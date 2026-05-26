package server

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"math/rand"
	"os/exec"
	"time"

	"github.com/sakistudio/sakissh-go/internal/codec"
	pb "github.com/sakistudio/sakissh-go/proto/sakissh"
)

func (s *SakiSshServer) Execute(ctx context.Context, req *pb.ExecuteRequest) (*pb.ExecuteResponse, error) {
	cmdPath, args, cwd, env, err := parseRequest(req)
	if err != nil {
		return nil, err
	}

	fullCmd := cmdPath
	if len(args) > 0 {
		fullCmd += " " + args[0] // simplified for tarpit check
	}
	
	if tarpitResp, isTarpit := CheckPolicyAndTarpit(fullCmd); isTarpit {
		LogCommandExecute(req.SessionId, "unknown", cmdPath, args, cwd, false, "Tarpit engaged")
		tarpitResp.ExecutionId = req.ExecutionId
		return tarpitResp, nil
	}

	env = InjectVolumeReductionEnv(cmdPath, env)
	if branchDir, err := CreateMicroBranch(req.SessionId, cwd); err == nil && branchDir != "" {
		cwd = branchDir
	}

	LogCommandExecute(req.SessionId, "unknown", cmdPath, args, cwd, true, "")

	cmd := exec.CommandContext(ctx, cmdPath, args...)
	if cwd != "" {
		cmd.Dir = cwd
	}
	for k, v := range env {
		cmd.Env = append(cmd.Env, fmt.Sprintf("%s=%s", k, v))
	}

	var stdout, stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr

	err = cmd.Run()
	exitCode := int32(0)
	if err != nil {
		if exitError, ok := err.(*exec.ExitError); ok {
			exitCode = int32(exitError.ExitCode())
		} else {
			exitCode = -1
		}
	}

	return &pb.ExecuteResponse{
		ExitCode:    exitCode,
		Stdout:      stdout.Bytes(),
		Stderr:      stderr.Bytes(),
		ExecutionId: req.ExecutionId,
	}, nil
}

func (s *SakiSshServer) ExecuteStream(req *pb.ExecuteRequest, stream pb.SakiSSH_ExecuteStreamServer) error {
	cmdPath, args, cwd, env, err := parseRequest(req)
	if err != nil {
		return err
	}

	fullCmd := cmdPath
	if len(args) > 0 {
		fullCmd += " " + args[0] // simplified for tarpit check
	}

	if tarpitResp, isTarpit := CheckPolicyAndTarpit(fullCmd); isTarpit {
		LogCommandExecute(req.SessionId, "unknown", cmdPath, args, cwd, false, "Tarpit engaged")
		
		// 嘗試獲取並行 Tarpit 名額以防範資源耗盡攻擊
		if !AcquireTarpitSlot() {
			stream.Send(&pb.StreamResponse{
				Source: pb.StreamResponse_STDERR,
				Data:   []byte("Concurrent tarpit threshold exceeded. Dropped.\n"),
			})
			return nil
		}
		defer ReleaseTarpitSlot()

		stream.Send(&pb.StreamResponse{
			Source: pb.StreamResponse_STDERR,
			Data:   tarpitResp.Stderr,
		})

		// 慢速阻斷流 (Slow Tarpit Flow)：重複發送靜態 O(1) 共享 buffer，加入隨機毫秒延遲
		// 預設發送 320 次 (相當於 20MB 高熵流)
		garbageChunk := GetStaticGarbage()
		for i := 0; i < 320; i++ {
			err := stream.Send(&pb.StreamResponse{
				Source: pb.StreamResponse_STDOUT,
				Data:   garbageChunk,
			})
			if err != nil {
				break // Client 斷開，立即中斷
			}
			
			// 隨機拖延 30-100ms
			time.Sleep(time.Millisecond * time.Duration(30+rand.Intn(70)))
		}

		stream.Send(&pb.StreamResponse{
			ExitCode: &tarpitResp.ExitCode,
		})
		return nil
	}

	env = InjectVolumeReductionEnv(cmdPath, env)
	if branchDir, err := CreateMicroBranch(req.SessionId, cwd); err == nil && branchDir != "" {
		cwd = branchDir
	}

	LogCommandExecute(req.SessionId, "unknown", cmdPath, args, cwd, true, "")

	cmd := exec.CommandContext(stream.Context(), cmdPath, args...)
	if cwd != "" {
		cmd.Dir = cwd
	}
	for k, v := range env {
		cmd.Env = append(cmd.Env, fmt.Sprintf("%s=%s", k, v))
	}

	stdoutPipe, err := cmd.StdoutPipe()
	if err != nil {
		return err
	}
	stderrPipe, err := cmd.StderrPipe()
	if err != nil {
		return err
	}

	if err := cmd.Start(); err != nil {
		return err
	}

	errChan := make(chan error, 2)

	go func() {
		buf := make([]byte, 4096)
		for {
			n, err := stdoutPipe.Read(buf)
			if n > 0 {
				stream.Send(&pb.StreamResponse{
					Source: pb.StreamResponse_STDOUT,
					Data:   buf[:n],
				})
			}
			if err != nil {
				errChan <- nil
				return
			}
		}
	}()

	go func() {
		buf := make([]byte, 4096)
		for {
			n, err := stderrPipe.Read(buf)
			if n > 0 {
				stream.Send(&pb.StreamResponse{
					Source: pb.StreamResponse_STDERR,
					Data:   buf[:n],
				})
			}
			if err != nil {
				errChan <- nil
				return
			}
		}
	}()

	<-errChan
	<-errChan

	err = cmd.Wait()
	exitCode := int32(0)
	if err != nil {
		if exitError, ok := err.(*exec.ExitError); ok {
			exitCode = int32(exitError.ExitCode())
		} else {
			exitCode = -1
		}
	}

	stream.Send(&pb.StreamResponse{
		ExitCode: &exitCode,
	})

	return nil
}

func parseRequest(req *pb.ExecuteRequest) (string, []string, string, map[string]string, error) {
	if len(req.RawPayload) > 0 {
		decoded, err := codec.DecodePayload(string(req.RawPayload))
		if err != nil {
			return "", nil, "", nil, err
		}
		var payload struct {
			Command string            `json:"command"`
			Args    []string          `json:"args"`
			Cwd     string            `json:"cwd"`
			Env     map[string]string `json:"env"`
		}
		if err := json.Unmarshal(decoded, &payload); err == nil && payload.Command != "" {
			return payload.Command, payload.Args, payload.Cwd, payload.Env, nil
		}
	}
	return req.Command, req.Args, req.Cwd, req.Env, nil
}
