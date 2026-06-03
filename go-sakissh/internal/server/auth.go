package server

import (
	"context"

	"github.com/sakistudio/sakissh-go/internal/defense"
	pb "github.com/sakistudio/sakissh-go/proto/sakissh"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

type AgentAuthenticator struct {
	ChallengeStore *defense.ChallengeStore
	// To be expanded with full ED25519 auth verification logic
}

func NewAuthenticator() *AgentAuthenticator {
	return &AgentAuthenticator{
		ChallengeStore: defense.NewChallengeStore(60),
	}
}

func (s *SakiSshServer) AuthCognitiveChallenge(ctx context.Context, req *pb.CognitiveChallengeRequest) (*pb.CognitiveChallengeResponse, error) {
	// For Go implementation demo
	return nil, status.Errorf(codes.Unimplemented, "cognitive challenge auth not fully implemented in Go skeleton")
}
