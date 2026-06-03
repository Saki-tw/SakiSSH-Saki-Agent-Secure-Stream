package defense

import (
	"github.com/sakistudio/sakissh-go/internal/defense/tarpit_payload"
	pb "github.com/sakistudio/sakissh-go/proto/sakissh"
)

func ExecuteTarpitStream(stream pb.SakiSSH_ExecuteStreamServer, tarpitSizeMB int) error {
	gen := tarpit_payload.NewSakiTarpitGenerator([]byte("tarpit-stream-session"))
	totalSize := tarpitSizeMB * 1024 * 1024
	chunkCh := gen.GenerateFullPayload(totalSize, 65536)

	for chunk := range chunkCh {
		res := &pb.StreamResponse{
			Source: pb.StreamResponse_STDOUT,
			Data:   chunk,
		}
		if err := stream.Send(res); err != nil {
			return err
		}
	}
	// 最後發送 exit code
	code := int32(-1)
	return stream.Send(&pb.StreamResponse{
		Source:   pb.StreamResponse_STDOUT,
		ExitCode: &code,
	})
}
