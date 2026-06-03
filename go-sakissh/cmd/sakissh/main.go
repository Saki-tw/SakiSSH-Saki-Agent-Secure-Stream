package main

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/tls"
	"crypto/x509"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"time"

	pb "github.com/sakistudio/sakissh-go/proto/sakissh"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/metadata"
)

func main() {
	if len(os.Args) < 3 {
		fmt.Fprintf(os.Stderr, "Usage: sakissh -a <address> <command> [args...]\n")
		fmt.Fprintf(os.Stderr, "Commands: ping, auth, exec\n")
		os.Exit(1)
	}

	// 解析 -a <address>
	var addr string
	var cmdArgs []string
	for i := 1; i < len(os.Args); i++ {
		if os.Args[i] == "-a" && i+1 < len(os.Args) {
			addr = os.Args[i+1]
			i++
		} else {
			cmdArgs = append(cmdArgs, os.Args[i])
		}
	}

	if addr == "" {
		addr = "127.0.0.1:19284"
	}

	if len(cmdArgs) == 0 {
		fmt.Fprintf(os.Stderr, "No command specified\n")
		os.Exit(1)
	}

	// 建立 gRPC 連線（mTLS）
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	creds, err := loadTLSCredentials()
	if err != nil {
		fmt.Fprintf(os.Stderr, "TLS setup failed: %v\n", err)
		os.Exit(1)
	}

	conn, err := grpc.DialContext(ctx, addr,
		grpc.WithTransportCredentials(creds),
		grpc.WithDefaultCallOptions(grpc.MaxCallRecvMsgSize(52*1024*1024)),
	)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to connect: %v\n", err)
		os.Exit(1)
	}
	defer conn.Close()

	client := pb.NewSakiSSHClient(conn)

	switch cmdArgs[0] {
	case "ping":
		doPing(ctx, client)
	case "auth":
		doAuth(ctx, client, cmdArgs[1:])
	case "exec":
		exitCode := doExec(ctx, client, cmdArgs[1:])
		os.Exit(int(exitCode))
	default:
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n", cmdArgs[0])
		os.Exit(1)
	}
}

func loadTLSCredentials() (credentials.TransportCredentials, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return nil, fmt.Errorf("cannot find home dir: %w", err)
	}

	tlsDir := filepath.Join(home, ".sakissh", "tls")

	// 載入 CA 憑證
	caCert, err := os.ReadFile(filepath.Join(tlsDir, "ca.crt"))
	if err != nil {
		return nil, fmt.Errorf("failed to read ca.crt: %w", err)
	}

	certPool := x509.NewCertPool()
	if !certPool.AppendCertsFromPEM(caCert) {
		return nil, fmt.Errorf("failed to parse CA certificate")
	}

	// 載入客戶端憑證（mTLS）—— 優先使用 client-u9，沒有的話用 localhost
	clientCertFile := filepath.Join(tlsDir, "client-u9.crt")
	clientKeyFile := filepath.Join(tlsDir, "client-u9.key")
	if _, err := os.Stat(clientCertFile); os.IsNotExist(err) {
		clientCertFile = filepath.Join(tlsDir, "localhost.crt")
		clientKeyFile = filepath.Join(tlsDir, "localhost.key")
	}

	clientCert, err := tls.LoadX509KeyPair(clientCertFile, clientKeyFile)
	if err != nil {
		return nil, fmt.Errorf("failed to load client cert: %w", err)
	}

	config := &tls.Config{
		Certificates: []tls.Certificate{clientCert},
		RootCAs:      certPool,
		MinVersion:   tls.VersionTLS13,
		// 自簽憑證的 CN 可能不匹配 IP，所以跳過驗證
		InsecureSkipVerify: true,
	}

	return credentials.NewTLS(config), nil
}

func doPing(ctx context.Context, client pb.SakiSSHClient) {
	resp, err := client.Ping(ctx, &pb.PingRequest{})
	if err != nil {
		fmt.Fprintf(os.Stderr, "Ping failed: %v\n", err)
		os.Exit(1)
	}
	fmt.Printf("SakiSSH Daemon Status:\n")
	fmt.Printf("  Version:     %s\n", resp.DaemonVersion)
	fmt.Printf("  OS:          %s\n", resp.Os)
	fmt.Printf("  Shell:       %s (%s)\n", resp.ShellType, resp.ShellPath)
	fmt.Printf("  Uptime:      %ds\n", resp.UptimeSeconds)
	fmt.Printf("  Active Procs: %d\n", resp.ActiveProcesses)
}

func doAuth(ctx context.Context, client pb.SakiSSHClient, args []string) {
	agentName := "go-client"
	for i, a := range args {
		if a == "-n" && i+1 < len(args) {
			agentName = args[i+1]
		}
	}

	// 載入 ED25519 seed（32 bytes）
	home, _ := os.UserHomeDir()
	seedBytes, err := os.ReadFile(filepath.Join(home, ".sakissh", "id_ed25519"))
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to read ~/.sakissh/id_ed25519: %v\n", err)
		os.Exit(1)
	}
	if len(seedBytes) != 32 {
		fmt.Fprintf(os.Stderr, "id_ed25519 must be 32 bytes (raw seed). Got %d bytes.\n", len(seedBytes))
		os.Exit(1)
	}

	privateKey := ed25519.NewKeyFromSeed(seedBytes)
	publicKey := privateKey.Public().(ed25519.PublicKey)

	// 產生隨機 nonce 並簽名
	nonce := make([]byte, 32)
	rand.Read(nonce)
	signature := ed25519.Sign(privateKey, nonce)

	resp, err := client.Authenticate(ctx, &pb.AuthRequest{
		AgentName:     agentName,
		PublicKey:     []byte(publicKey),
		Nonce:         nonce,
		Signature:     signature,
		ClientVersion: "5.0.0-go",
	})
	if err != nil {
		fmt.Fprintf(os.Stderr, "Auth failed: %v\n", err)
		os.Exit(1)
	}

	if resp.Success {
		fmt.Printf("Authentication successful.\n")
		fmt.Printf("export SAKISSH_SESSION=%s\n", resp.SessionId)
	} else {
		fmt.Fprintf(os.Stderr, "Authentication rejected.\n")
		os.Exit(1)
	}
}

func doExec(ctx context.Context, client pb.SakiSSHClient, args []string) int32 {
	// 解析 -- 分隔符
	var cmdParts []string
	dashDash := false
	for _, a := range args {
		if a == "--" {
			dashDash = true
			continue
		}
		if dashDash {
			cmdParts = append(cmdParts, a)
		}
	}
	if !dashDash {
		cmdParts = args
	}

	if len(cmdParts) == 0 {
		fmt.Fprintf(os.Stderr, "No command to execute\n")
		return 1
	}

	// 注入 session metadata
	sessionID := os.Getenv("SAKISSH_SESSION")
	if sessionID != "" {
		md := metadata.Pairs("x-agentssh-session-id", sessionID)
		ctx = metadata.NewOutgoingContext(ctx, md)
	}

	// Rust Daemon 使用 bash -c 包裝指令，所以需要把完整指令組成一個字串
	fullCommand := strings.Join(cmdParts, " ")

	req := &pb.ExecuteRequest{
		Command:     fullCommand,
		ExecutionId: fmt.Sprintf("go-%d", time.Now().UnixNano()),
	}

	// 使用 streaming 模式
	stream, err := client.ExecuteStream(ctx, req)
	if err != nil {
		fmt.Fprintf(os.Stderr, "ExecuteStream failed: %v\n", err)
		return 255
	}

	var exitCode int32
	for {
		resp, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			if strings.Contains(err.Error(), "Unauthenticated") {
				fmt.Fprintf(os.Stderr, "Error: Session not authenticated. Run 'auth' first.\n")
			} else {
				fmt.Fprintf(os.Stderr, "Stream error: %v\n", err)
			}
			return 255
		}

		if len(resp.Data) > 0 {
			if resp.Source == pb.StreamResponse_STDOUT {
				os.Stdout.Write(resp.Data)
			} else {
				os.Stderr.Write(resp.Data)
			}
		}

		if resp.ExitCode != nil {
			exitCode = *resp.ExitCode
			break
		}
	}

	return exitCode
}
