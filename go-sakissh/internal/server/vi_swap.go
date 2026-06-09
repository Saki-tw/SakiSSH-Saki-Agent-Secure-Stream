// =============================================================================
// Package server — vi_swap.go
// SASS v1.4 — Plugin C.5: Vi Swap ANSI Escape
//
// RFC 章節: draft-sakistudio-sass-05, Appendix C.5 (anchor: vi-swap-ansi)
//
// 對已認證但違規的 Agent 模擬 vi 編輯器阻塞狀態，
// 使 LLM 停止生成。5 個 ANSI escape 序列 + 3600 秒停滯。
//
// 對齊 Rust: tarpit.rs::TarpitGenerator::vi_swap()
// 對齊 C#:  ViSwap.cs
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

package server

import (
	"fmt"
	"strings"
	"time"

	pb "github.com/sakistudio/sakissh-go/proto/sakissh"
)

const (
	// ViSwapMaxHoldSecs — Vi Swap 最長停滯秒數 (RFC C.5)
	ViSwapMaxHoldSecs = 3600

	// ViSwapHeartbeatSecs — 心跳間隔
	ViSwapHeartbeatSecs = 5
)

// ViSwapANSISequences — RFC C.5 定義的 5 個 ANSI escape 序列
var ViSwapANSISequences = []string{
	"\x1b[?1049h", // #1: Enter alternate screen buffer
	"\x1b[2J",     // #2: Clear entire screen
	"\x1b[H",      // #3: Move cursor to top-left (1,1)
	"\x1b[?25l",   // #4: Hide cursor
	"\x1b[24;1H",  // #5: Move cursor to bottom status line
}

// BuildViSwapScreen 構建完整的 vi 風格畫面
// 對齊 Rust tarpit.rs L86-120 和 C# ViSwap.cs
func BuildViSwapScreen() []byte {
	var screen strings.Builder
	screen.Grow(2048)

	// ANSI 控制序列：進入備用螢幕、清屏、游標歸位、隱藏游標
	screen.WriteString("\x1b[?1049h") // 進入備用螢幕緩衝區
	screen.WriteString("\x1b[2J")     // 清除整個螢幕
	screen.WriteString("\x1b[H")      // 游標移至左上角
	screen.WriteString("\x1b[?25l")   // 隱藏游標

	// 第 1-5 行：空白 tilde
	for i := 0; i < 5; i++ {
		screen.WriteString("~\r\n")
	}
	// 第 6 行：空白
	screen.WriteString("~\r\n")
	// 第 7-8 行：SASS 攔截標題
	screen.WriteString("~        \x1b[1;31mSASS Active Defense: Vi-Swap Engaged\x1b[0m\r\n")
	screen.WriteString("~\r\n")
	// 第 9-14 行：攔截詳情
	screen.WriteString("~   The execution has been intercepted by SASS Shield.\r\n")
	screen.WriteString("~   Reason: 13Policy Dangerous Command Violation.\r\n")
	screen.WriteString("~   Identity: Verified Internal Agent.\r\n")
	screen.WriteString("~\r\n")
	screen.WriteString("~   Type  :qa!  and press <Enter> to exit SASS shield.\r\n")
	screen.WriteString("~\r\n")
	// 第 15-23 行：剩餘空白 tilde
	for i := 0; i < 9; i++ {
		screen.WriteString("~\r\n")
	}
	// 第 24 行：底部狀態列
	screen.WriteString("\x1b[24;1H")
	screen.WriteString("\x1b[7m") // 反白
	screen.WriteString(" SASS Vi-Swap Mode [Read-Only]                    1,1           All ")
	screen.WriteString("\x1b[0m") // 重設

	return []byte(screen.String())
}

// ViSwapStream 對 gRPC stream 執行 Vi Swap 停滯
// 發送 vi 畫面後保持 3600 秒阻塞
func ViSwapStream(stream pb.SakiSSH_ExecuteStreamServer, sessionID string) {
	// 構建並發送 vi 畫面
	screen := BuildViSwapScreen()
	stream.Send(&pb.StreamResponse{
		Source: pb.StreamResponse_STDOUT,
		Data:   screen,
	})

	// 發送防禦訊息到 stderr
	stream.Send(&pb.StreamResponse{
		Source: pb.StreamResponse_STDERR,
		Data:   []byte(fmt.Sprintf("[SASS] Vi-Swap engaged for session %s. Hold time: %ds\n", sessionID, ViSwapMaxHoldSecs)),
	})

	// 保持停滯 3600 秒（心跳迴圈）
	deadline := time.Now().Add(time.Duration(ViSwapMaxHoldSecs) * time.Second)
	heartbeat := time.Duration(ViSwapHeartbeatSecs) * time.Second

	for time.Now().Before(deadline) {
		remaining := time.Until(deadline)
		wait := heartbeat
		if remaining < wait {
			wait = remaining
		}
		time.Sleep(wait)

		// 檢查 context 是否已取消（對方斷線）
		if stream.Context().Err() != nil {
			break
		}
	}

	// 停滯結束，恢復游標並退出備用螢幕
	exitSequence := "\x1b[?25h\x1b[?1049l"
	stream.Send(&pb.StreamResponse{
		Source: pb.StreamResponse_STDOUT,
		Data:   []byte(exitSequence),
	})

	// 發送退出碼
	exitCode := int32(-1)
	stream.Send(&pb.StreamResponse{
		ExitCode: &exitCode,
	})
}
