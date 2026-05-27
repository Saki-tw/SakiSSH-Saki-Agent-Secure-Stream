// =============================================================================
// Package server — v6_integration.go
// SASS v1.4 — 6-Response 狀態機 (Total Response Mapping)
//
// 對應 Rust: v6_integration.rs (MySsh::execute_stream_v6)
// RFC 參考: draft-sakistudio-sass-00 §4 (6-response-state-machine)
//
// 六種回應映射:
//   R1(EXECUTE)  — 正常執行
//   R2(CHALLENGE) — 認知挑戰（邊界邊緣指令）
//   R3(THROTTLE) — 排隊等待（配額超限）
//   R4(VI_SWAP)  — Vi Swap 誘騙（已認證 Agent 的致命違規）
//   R5(TARPIT)   — 高熵吞噬（未認證 Agent 的致命違規）
//   R6(DROP)     — 靜默丟棄
//
// 流程:
//   1. ACL 檢查 → 2. LocalHost Spoofing 防禦 →
//   3. 斷線重連 (Idempotent Resumption) → 4. 資源配額排隊 →
//   5. 13Policy 邊界裁定 → 6. 環境清洗 → 7. 分支建立 →
//   8. 進程執行 → 9. Kernel Bridge PID 註冊 →
//   10. 雙重看門狗 → 11. stdout/stderr 串流
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

package server

import (
	"fmt"
	"log"
	"os/exec"
	"strings"

	"github.com/sakistudio/sakissh-go/internal/defense"
	pb "github.com/sakistudio/sakissh-go/proto/sakissh"
	"google.golang.org/grpc/peer"
)

// PolicyVerdict — 13Policy 邊界裁定結果
// 對齊 Rust: policy::PolicyVerdict enum
type PolicyVerdict int

const (
	// VerdictAllow — 完全在授權邊界內，正常執行
	VerdictAllow PolicyVerdict = iota
	// VerdictAllowWithAudit — 在授權邊界內，但增強審計記錄
	VerdictAllowWithAudit
	// VerdictChallenge — 邊界邊緣，需要認知挑戰
	VerdictChallenge
	// VerdictChallengeHigh — 高風險邊界邊緣
	VerdictChallengeHigh
	// VerdictTarpit — 致命邊界違規，啟動 Tarpit 吞噬
	VerdictTarpit
)

// EvaluateCommand — 13Policy 邊界裁定
// 對齊 Rust: Policy13::evaluate_command()
//
// 以集合論成員資格檢驗指令是否在授權邊界內
func EvaluateCommand(command string) PolicyVerdict {
	cmdLower := strings.ToLower(command)

	// 致命違規 — Tarpit
	dangerousKeywords := []string{
		"rm -rf /",
		"mkfs",
		"dd if=/dev/zero of=/dev/sda",
		"> /dev/sda",
		":(){ :|:& };:",
	}
	for _, kw := range dangerousKeywords {
		if strings.Contains(cmdLower, kw) {
			return VerdictTarpit
		}
	}

	// 高風險邊界 — Challenge
	sensitiveKeywords := []string{
		"chmod 777",
		"chown root",
		"iptables",
		"firewall-cmd",
		"systemctl disable",
	}
	for _, kw := range sensitiveKeywords {
		if strings.Contains(cmdLower, kw) {
			return VerdictChallengeHigh
		}
	}

	// 需審計的指令
	auditKeywords := []string{
		"sudo",
		"su -",
		"passwd",
		"useradd",
		"userdel",
	}
	for _, kw := range auditKeywords {
		if strings.Contains(cmdLower, kw) {
			return VerdictAllowWithAudit
		}
	}

	return VerdictAllow
}

// ExecuteStreamV6 — 6-Response 狀態機串流執行入口
// 對齊 Rust: MySsh::execute_stream_v6()
//
// 此為 SASS v1.4 的核心執行路徑，所有 ExecuteStream 請求都經過此狀態機。
func (s *SakiSshServer) ExecuteStreamV6(req *pb.ExecuteRequest, stream pb.SakiSSH_ExecuteStreamServer) error {
	ctx := stream.Context()

	// 取得呼叫方 IP（用於 LocalHost Spoofing 偵測）
	var remoteAddr string
	if p, ok := peer.FromContext(ctx); ok && p.Addr != nil {
		remoteAddr = p.Addr.String()
	}

	// === Phase 1: LocalHost Spoofing 防禦 ===
	if strings.HasPrefix(remoteAddr, "127.") || strings.HasPrefix(remoteAddr, "[::1]") {
		// Localhost 請求 — 檢查是否為探測指令
		if spoofedData := defense.HandleSpoofing(req.Command, req.Args); spoofedData != nil {
			exitCode := int32(0)
			stream.Send(&pb.StreamResponse{
				Source:   pb.StreamResponse_STDOUT,
				Data:     spoofedData,
				ExitCode: &exitCode,
			})
			return nil
		}
	}

	// === Phase 2: 13Policy 邊界裁定 (Boundary Adjudicator) ===
	fullCommand := req.Command
	if len(req.Args) > 0 {
		fullCommand += " " + strings.Join(req.Args, " ")
	}
	verdict := EvaluateCommand(fullCommand)

	switch verdict {
	case VerdictTarpit:
		// 致命邊界違規：啟動 Tarpit 吞噬
		log.Printf("[WARN] 13Policy Tarpit: Engulfing session for command='%s'", fullCommand)
		LogCommandExecute(req.SessionId, "unknown", req.Command, req.Args, req.Cwd, false, "Tarpit engaged by 13Policy")

		// 使用既有的 tarpit 機制
		if tarpitResp, isTarpit := CheckPolicyAndTarpit(fullCommand); isTarpit {
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

			garbageChunk := GetStaticGarbage()
			for i := 0; i < 320; i++ {
				if err := stream.Send(&pb.StreamResponse{
					Source: pb.StreamResponse_STDOUT,
					Data:   garbageChunk,
				}); err != nil {
					break
				}
			}
			stream.Send(&pb.StreamResponse{
				ExitCode: &tarpitResp.ExitCode,
			})
		}
		return nil

	case VerdictChallengeHigh, VerdictChallenge:
		// 邊界邊緣：回傳認知挑戰訊息，不執行指令
		log.Printf("[WARN] 13Policy Challenge: command='%s' 位於授權邊界邊緣", fullCommand)
		challengeMsg := fmt.Sprintf(
			"13Policy Boundary Challenge: '%s' 位於授權邊界外，需要額外授權才能執行。",
			req.Command)
		exitCode := int32(-13)
		stream.Send(&pb.StreamResponse{
			Source:   pb.StreamResponse_SYSTEM,
			Data:     []byte(challengeMsg),
			ExitCode: &exitCode,
		})
		return nil

	case VerdictAllowWithAudit:
		// 在授權邊界內，但增強審計記錄
		LogCommandExecute(req.SessionId, "unknown", req.Command, req.Args, req.Cwd, true,
			"AllowWithAudit: 增強審計")
		// 繼續執行（落入下方正常流程）

	case VerdictAllow:
		// 完全在授權邊界內，正常執行
	}

	// === Phase 3: 環境清洗 (Headless Enforcement) ===
	env := req.Env
	if env == nil {
		env = make(map[string]string)
	}
	defense.SanitizeEnv(env)
	env = InjectVolumeReductionEnv(req.Command, env)

	// === Phase 4: 微型動態分支 (Micro Overlay Branching) ===
	cwd := req.Cwd
	if cwd == "" {
		cwd = "."
	}
	if branchDir, err := CreateMicroBranch(req.SessionId, cwd); err == nil && branchDir != "" {
		cwd = branchDir
	}

	// === Phase 5: 審計日誌 ===
	LogCommandExecute(req.SessionId, "unknown", req.Command, req.Args, cwd, true, "")

	// === Phase 6: 進程執行 ===
	cmdPath := req.Command
	cmd := exec.CommandContext(ctx, cmdPath, req.Args...)
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

	// === Phase 7: Kernel Bridge PID 註冊 ===
	if cmd.Process != nil {
		defense.RegisterRestrictedPID(uint32(cmd.Process.Pid))
	}

	// === Phase 8: 雙重看門狗 ===
	monitor := defense.NewProcessMonitor(30, 3600) // 30s inactivity, 1hr absolute
	killSignal := make(chan struct{})
	go func() {
		if monitor.SpawnWatchdog(killSignal) {
			// Watchdog 觸發 — 強制結束進程
			if cmd.Process != nil {
				cmd.Process.Kill()
			}
		}
	}()

	// === Phase 9: stdout/stderr 串流 ===
	errChan := make(chan error, 2)

	// stdout goroutine
	go func() {
		buf := make([]byte, 4096)
		for {
			n, readErr := stdoutPipe.Read(buf)
			if n > 0 {
				monitor.TickActivity()
				stream.Send(&pb.StreamResponse{
					Source: pb.StreamResponse_STDOUT,
					Data:   buf[:n],
				})
			}
			if readErr != nil {
				errChan <- nil
				return
			}
		}
	}()

	// stderr goroutine
	go func() {
		buf := make([]byte, 4096)
		for {
			n, readErr := stderrPipe.Read(buf)
			if n > 0 {
				monitor.TickActivity()
				stream.Send(&pb.StreamResponse{
					Source: pb.StreamResponse_STDERR,
					Data:   buf[:n],
				})
			}
			if readErr != nil {
				errChan <- nil
				return
			}
		}
	}()

	// 等待兩個串流 goroutine 完成
	<-errChan
	<-errChan

	// 等待進程結束
	err = cmd.Wait()
	close(killSignal) // 通知看門狗正常結束

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
