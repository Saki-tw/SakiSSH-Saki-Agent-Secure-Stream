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

func (s *SakiSshServer) Auth(ctx context.Context, req *pb.AuthRequest) (*pb.AuthResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method Auth not implemented")
}

func (s *SakiSshServer) GetAuditPublicKey(ctx context.Context, req *pb.GetAuditPublicKeyRequest) (*pb.GetAuditPublicKeyResponse, error) {
	pubKeyHex := GetAuditPublicKeyPEM()
	if pubKeyHex == "" {
		return nil, status.Errorf(codes.Internal, "Audit logger not initialized")
	}
	return &pb.GetAuditPublicKeyResponse{
		PublicKeyHex:   pubKeyHex,
		KeyFingerprint: "SASS-Ed25519-PEM-Go",
	}, nil
}
