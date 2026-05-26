package defense

import (
	"crypto/rand"

	pb "github.com/sakistudio/sakissh-go/proto/sakissh"
)

func ExecuteTarpitStream(stream pb.SakiSSH_ExecuteStreamServer, tarpitSizeMB int) error {
	totalChunks := tarpitSizeMB
	for i := 0; i < totalChunks; i++ {
		chunk := make([]byte, 1024*1024) // 1MB
		rand.Read(chunk)

		res := &pb.StreamResponse{
			Source: pb.StreamResponse_STDOUT,
			Data:   chunk,
		}

		if i == totalChunks-1 {
			code := int32(-1)
			res.ExitCode = &code
		}

		if err := stream.Send(res); err != nil {
			return err
		}
	}
	return nil
}
