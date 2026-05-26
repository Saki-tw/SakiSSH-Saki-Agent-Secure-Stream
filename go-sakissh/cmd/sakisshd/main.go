package main

import (
	"fmt"
	"log"
	"net"

	"github.com/sakistudio/sakissh-go/internal/config"
	"github.com/sakistudio/sakissh-go/internal/server"
	pb "github.com/sakistudio/sakissh-go/proto/sakissh"
	"google.golang.org/grpc"
)

func main() {
	cfg, err := config.LoadOrCreate(config.DefaultPath())
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	lis, err := net.Listen("tcp", cfg.BindAddress)
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}

	fmt.Printf("Starting SakiSSH Daemon on %s...\n", cfg.BindAddress)
	
	// In a real implementation, we would add TLS credentials here
	grpcServer := grpc.NewServer()
	pb.RegisterSakiSSHServer(grpcServer, server.NewServer())
	
	if err := grpcServer.Serve(lis); err != nil {
		log.Fatalf("Failed to serve: %v", err)
	}
}
