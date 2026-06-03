package server

import (
	"crypto/rand"
	"strings"
	"sync/atomic"

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

