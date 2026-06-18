package server

import (
	"context"
	pb "github.com/sakistudio/sakissh-go/proto/sakissh"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

type SakiSshServer struct {
	pb.UnimplementedSakiSSHServer
}

func NewServer() *SakiSshServer {
	return &SakiSshServer{}
}

func (s *SakiSshServer) Ping(ctx context.Context, req *pb.PingRequest) (*pb.PingResponse, error) {
	return &pb.PingResponse{
		DaemonVersion: "5.0.0-go",
		Os:            "unknown",
		ShellType:     "sh",
		ShellPath:     "/bin/sh",
		UptimeSeconds:   0,
		ActiveProcesses: 0,
	}, nil
}

func (s *SakiSshServer) Authenticate(ctx context.Context, req *pb.AuthRequest) (*pb.AuthResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method Authenticate not implemented")
}

// NOTE: GetAuditPublicKey RPC 已從 proto 移除，此方法暫時停用
// 若需要恢復，需先在 sakissh.proto 中定義對應的 RPC 和 Message
// func (s *SakiSshServer) GetAuditPublicKey(ctx context.Context, req *pb.GetAuditPublicKeyRequest) (*pb.GetAuditPublicKeyResponse, error) {
// 	pubKeyHex := GetAuditPublicKeyPEM()
// 	if pubKeyHex == "" {
// 		return nil, status.Errorf(codes.Internal, "Audit logger not initialized")
// 	}
// 	return &pb.GetAuditPublicKeyResponse{
// 		PublicKeyHex:   pubKeyHex,
// 		KeyFingerprint: "SASS-Ed25519-PEM-Go",
// 	}, nil
// }

// v6: RawFileTransfer（§7.3 — 繞過 shell I/O 的 bit-perfect 傳輸）
func (s *SakiSshServer) RawFileTransfer(stream pb.SakiSSH_RawFileTransferServer) error {
	return status.Errorf(codes.Unimplemented, "RawFileTransfer not yet implemented")
}

// v6: RenewSession（§5.2 Session 續期）
func (s *SakiSshServer) RenewSession(ctx context.Context, req *pb.RenewSessionRequest) (*pb.RenewSessionResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "RenewSession not yet implemented")
}

