// =============================================================================
// Package server — tarpit_test.go
// SASS Plugin #3: Tarpit 單元測試 (Server 層)
//
// 對照 C# PluginTests.cs: TarpitBufferTests
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

package server

import (
	"sync/atomic"
	"testing"
)

// --- PolicyEngine.CheckCommand ---

func TestCheckCommand_DangerousCommands_ReturnsTrue(t *testing.T) {
	t.Parallel()
	engine := &PolicyEngine{TarpitSizeMB: 40}

	dangerous := []string{
		"rm -rf /",
		"sudo mkfs /dev/sdb1",
		"dd if=/dev/zero of=/dev/sda bs=1M",
		"cat > /dev/sda",
	}
	for _, cmd := range dangerous {
		if !engine.CheckCommand(cmd) {
			t.Errorf("危險指令 %q 應被偵測", cmd)
		}
	}
}

func TestCheckCommand_SafeCommands_ReturnsFalse(t *testing.T) {
	t.Parallel()
	engine := &PolicyEngine{TarpitSizeMB: 40}

	safe := []string{
		"ls -la",
		"cat /etc/hostname",
		"echo hello",
		"pwd",
		"whoami",
	}
	for _, cmd := range safe {
		if engine.CheckCommand(cmd) {
			t.Errorf("安全指令 %q 不應被偵測為危險", cmd)
		}
	}
}

func TestCheckCommand_CaseInsensitive(t *testing.T) {
	t.Parallel()
	engine := &PolicyEngine{TarpitSizeMB: 40}
	if !engine.CheckCommand("RM -RF /") {
		t.Error("危險指令偵測應不分大小寫")
	}
}

// --- CheckPolicyAndTarpit ---

func TestCheckPolicyAndTarpit_DangerousCommand_ReturnsTarpitResponse(t *testing.T) {
	t.Parallel()
	resp, isTarpit := CheckPolicyAndTarpit("rm -rf /")
	if !isTarpit {
		t.Fatal("危險指令應觸發 tarpit")
	}
	if resp == nil {
		t.Fatal("tarpit response 不應為 nil")
	}
	if resp.ExitCode != -1 {
		t.Errorf("tarpit ExitCode 應為 -1，實際為 %d", resp.ExitCode)
	}
}

func TestCheckPolicyAndTarpit_SafeCommand_ReturnsNil(t *testing.T) {
	t.Parallel()
	resp, isTarpit := CheckPolicyAndTarpit("ls -la")
	if isTarpit {
		t.Error("安全指令不應觸發 tarpit")
	}
	if resp != nil {
		t.Error("安全指令的 response 應為 nil")
	}
}

func TestCheckPolicyAndTarpit_DangerousCommand_StdoutIsGarbage(t *testing.T) {
	t.Parallel()
	resp, isTarpit := CheckPolicyAndTarpit("rm -rf /")
	if !isTarpit || resp == nil {
		t.Skip("tarpit 未觸發")
	}
	if len(resp.Stdout) != ChunkSize {
		t.Errorf("tarpit stdout 應為 %d bytes (staticGarbage)，實際為 %d", ChunkSize, len(resp.Stdout))
	}
}

// --- GetStaticGarbage ---

func TestGetStaticGarbage_Returns64KB(t *testing.T) {
	t.Parallel()
	garbage := GetStaticGarbage()
	if len(garbage) != ChunkSize {
		t.Errorf("staticGarbage 應為 %d bytes，實際為 %d", ChunkSize, len(garbage))
	}
}

func TestGetStaticGarbage_IsNotAllZeros(t *testing.T) {
	t.Parallel()
	garbage := GetStaticGarbage()
	allZero := true
	for _, b := range garbage {
		if b != 0 {
			allZero = false
			break
		}
	}
	if allZero {
		t.Error("staticGarbage 應為高熵隨機資料，不應全為零")
	}
}

// --- AcquireTarpitSlot / ReleaseTarpitSlot ---

func TestAcquireAndReleaseTarpitSlot(t *testing.T) {
	// 不使用 t.Parallel()，因為操作全域計數器
	initial := atomic.LoadInt32(&activeTarpitCount)

	if !AcquireTarpitSlot() {
		t.Fatal("初始狀態應能獲取 tarpit slot")
	}
	current := atomic.LoadInt32(&activeTarpitCount)
	if current != initial+1 {
		t.Errorf("獲取後計數器應為 %d，實際為 %d", initial+1, current)
	}

	ReleaseTarpitSlot()
	after := atomic.LoadInt32(&activeTarpitCount)
	if after != initial {
		t.Errorf("釋放後計數器應回到 %d，實際為 %d", initial, after)
	}
}

// --- 常量驗證 ---

func TestMaxConcurrentTarpit_Is32(t *testing.T) {
	t.Parallel()
	if MaxConcurrentTarpit != 32 {
		t.Errorf("MaxConcurrentTarpit 應為 32，實際為 %d", MaxConcurrentTarpit)
	}
}

func TestChunkSize_Is65536(t *testing.T) {
	t.Parallel()
	if ChunkSize != 65536 {
		t.Errorf("ChunkSize 應為 65536 (64KB)，實際為 %d", ChunkSize)
	}
}

// --- RFC Appendix C.3 慢速串流常量驗證 ---

func TestTotalTarpitBytes_Is40MiB(t *testing.T) {
	t.Parallel()
	expected := 40 * 1024 * 1024
	if TotalTarpitBytes != expected {
		t.Errorf("TotalTarpitBytes 應為 %d (40 MiB)，實際為 %d", expected, TotalTarpitBytes)
	}
}

func TestChunkDelayMs_Is500(t *testing.T) {
	t.Parallel()
	if ChunkDelayMs != 500 {
		t.Errorf("ChunkDelayMs 應為 500，實際為 %d", ChunkDelayMs)
	}
}

func TestSendTimeoutSecs_Is3(t *testing.T) {
	t.Parallel()
	if SendTimeoutSecs != 3 {
		t.Errorf("SendTimeoutSecs 應為 3，實際為 %d", SendTimeoutSecs)
	}
}

func TestTotalChunks_Is640(t *testing.T) {
	t.Parallel()
	totalChunks := TotalTarpitBytes / ChunkSize
	if totalChunks != 640 {
		t.Errorf("TotalTarpitBytes/ChunkSize 應為 640，實際為 %d", totalChunks)
	}
}
