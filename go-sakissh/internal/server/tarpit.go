package server

import (
	"context"
	"crypto/rand"
	"log"
	"strings"
	"sync/atomic"
	"time"

	pb "github.com/sakistudio/sakissh-go/proto/sakissh"
)

type PolicyEngine struct {
	TarpitSizeMB int
}

var globalPolicyEngine = &PolicyEngine{
	TarpitSizeMB: 40,
}

var (
	// 靜態全域 64KB 高熵 Buffer，所有連線共享，空間複雜度 O(1)
	staticGarbage []byte
	// 當前並行的 Tarpit 任務計數器
	activeTarpitCount int32
)

const (
	MaxConcurrentTarpit = 32
	ChunkSize           = 65536 // 64KB

	// RFC Appendix C.3: 慢速串流參數
	TotalTarpitBytes = 40 * 1024 * 1024 // 40 MiB
	ChunkDelayMs     = 500              // 500ms chunk 間隔
	SendTimeoutSecs  = 3                // 3s send timeout
)

func init() {
	staticGarbage = make([]byte, ChunkSize)
	_, err := rand.Read(staticGarbage)
	if err != nil {
		panic(err)
	}
}

func (p *PolicyEngine) CheckCommand(command string) bool {
	cmdLower := strings.ToLower(command)
	dangerousKeywords := []string{
		"rm -rf /",
		"mkfs",
		"dd if=/dev/zero of=/dev/sda",
		"> /dev/sda",
	}
	for _, kw := range dangerousKeywords {
		if strings.Contains(cmdLower, kw) {
			return true
		}
	}
	return false
}

// CheckPolicyAndTarpit 進行門控防禦，並返回一個安全的靜態高熵數據，杜絕巨量內存動態分配
func CheckPolicyAndTarpit(command string) (*pb.ExecuteResponse, bool) {
	if globalPolicyEngine.CheckCommand(command) {
		// 檢查並行門控，防止並行自噬 DoS
		currentActive := atomic.LoadInt32(&activeTarpitCount)
		if currentActive >= MaxConcurrentTarpit {
			return &pb.ExecuteResponse{
				ExitCode: -1,
				Stderr:   []byte("Concurrent tarpit threshold exceeded. Connection dropped."),
			}, true
		}

		// 單次 Execute 回傳一個靜態共享的 Chunk，避免分配 40MB
		return &pb.ExecuteResponse{
			ExitCode: -1,
			Stdout:   staticGarbage,
			Stderr:   []byte("Tarpit engaged. Security violation."),
		}, true
	}
	return nil, false
}

// AcquireTarpitSlot 嘗試獲取並行 Tarpit 名額
func AcquireTarpitSlot() bool {
	for {
		current := atomic.LoadInt32(&activeTarpitCount)
		if current >= MaxConcurrentTarpit {
			return false
		}
		if atomic.CompareAndSwapInt32(&activeTarpitCount, current, current+1) {
			return true
		}
	}
}

// ReleaseTarpitSlot 釋放並行 Tarpit 名額
func ReleaseTarpitSlot() {
	atomic.AddInt32(&activeTarpitCount, -1)
}

// GetStaticGarbage 返回共享的唯讀垃圾塊
func GetStaticGarbage() []byte {
	return staticGarbage
}

// EngulfStream 實現 RFC Appendix C.3 慢速串流
// 對齊 Rust: TarpitGenerator::engulf()
//
// 行為:
//   - 總負載 40 MiB (TotalTarpitBytes)
//   - 每 chunk 64 KiB (ChunkSize)
//   - chunk 間隔 500ms (ChunkDelayMs)
//   - 總 640 chunks、~320 秒
//   - 每次 send 帶 3 秒 timeout (SendTimeoutSecs)
//   - 從 staticGarbage 讀取（零分配）
func EngulfStream(stream pb.SakiSSH_ExecuteStreamServer) error {
	// 1. 並行門控：防止並行自噬 DoS
	if !AcquireTarpitSlot() {
		log.Printf("[WARN] Tarpit 並行上限已達 %d，拒絕新的 engulf", MaxConcurrentTarpit)
		return stream.Send(&pb.StreamResponse{
			Source: pb.StreamResponse_STDERR,
			Data:   []byte("Concurrent tarpit threshold exceeded. Connection dropped."),
		})
	}
	defer ReleaseTarpitSlot()

	totalChunks := TotalTarpitBytes / ChunkSize // 640

	log.Printf("[INFO] Tarpit engulf 開始: %d chunks, %d bytes/chunk, 間隔 %dms",
		totalChunks, ChunkSize, ChunkDelayMs)

	for i := 0; i < totalChunks; i++ {
		// 檢查 stream context 是否已取消（對方斷線）
		if stream.Context().Err() != nil {
			log.Printf("[INFO] Tarpit engulf 中斷: 對方斷線 (chunk %d/%d)", i, totalChunks)
			return stream.Context().Err()
		}

		// 使用帶 timeout 的 context 執行 send
		sendCtx, sendCancel := context.WithTimeout(stream.Context(), time.Duration(SendTimeoutSecs)*time.Second)

		// 在帶 timeout 的 goroutine 中發送
		errCh := make(chan error, 1)
		go func() {
			errCh <- stream.Send(&pb.StreamResponse{
				Source: pb.StreamResponse_STDOUT,
				Data:   staticGarbage, // 零分配：共享唯讀 buffer
			})
		}()

		select {
		case err := <-errCh:
			sendCancel()
			if err != nil {
				log.Printf("[INFO] Tarpit engulf send 失敗 (chunk %d/%d): %v", i, totalChunks, err)
				return err
			}
		case <-sendCtx.Done():
			sendCancel()
			log.Printf("[INFO] Tarpit engulf send timeout (chunk %d/%d)", i, totalChunks)
			return sendCtx.Err()
		}

		// chunk 間隔延遲 500ms
		time.Sleep(time.Duration(ChunkDelayMs) * time.Millisecond)
	}

	log.Printf("[INFO] Tarpit engulf 完成: 已發送 %d chunks (%d bytes)",
		totalChunks, TotalTarpitBytes)

	return nil
}
