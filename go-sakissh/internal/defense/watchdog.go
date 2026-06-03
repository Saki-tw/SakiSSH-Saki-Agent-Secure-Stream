// =============================================================================
// Package defense — watchdog.go
// SASS v1.4 靜默超時與無頭環境看門狗 (Headless Watchdog & Anti-Hang)
//
// 對應 Rust: watchdog.rs (ProcessMonitor)
// RFC 參考: draft-sakistudio-sass-00 §6.3 (dual-watchdog)
//
// 防禦 Codex 等 Agent 的 Computer Use 或惡意佔用 (Slowloris/Hang)。
// 同時檢查「靜默超時 (Inactivity)」與「絕對超時 (Absolute)」。
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

package defense

import (
	"log"
	"sync"
	"time"
)

// ProcessMonitor — 雙重看門狗監控執行狀態
// 對齊 Rust: ProcessMonitor struct
//
// 設計原則:
//  1. Inactivity Timeout: 防禦 GUI hang 或 interactive prompt（Codex 防禦）
//  2. Absolute Timeout: 防禦 Slowloris 攻擊（Agent 每 29 秒送 1 byte）
//
// RFC 參考: draft-sakistudio-sass-00 §6.3 (dual-watchdog)
type ProcessMonitor struct {
	mu                sync.RWMutex
	lastActivity      time.Time
	startTime         time.Time
	inactivityTimeout time.Duration
	absoluteTimeout   time.Duration
}

// NewProcessMonitor 建立雙重看門狗
// 對齊 Rust: ProcessMonitor::new()
//
// # 參數
//   - inactivitySecs: 靜默超時秒數（無 I/O 輸出即觸發）
//   - absoluteSecs: 絕對超時秒數（無論活動與否，超過即強制結束）
func NewProcessMonitor(inactivitySecs, absoluteSecs int64) *ProcessMonitor {
	now := time.Now()
	return &ProcessMonitor{
		lastActivity:      now,
		startTime:         now,
		inactivityTimeout: time.Duration(inactivitySecs) * time.Second,
		absoluteTimeout:   time.Duration(absoluteSecs) * time.Second,
	}
}

// TickActivity 更新活動時間戳
// 對齊 Rust: ProcessMonitor::tick_activity()
// 每當 stdout/stderr 有輸出時呼叫
func (m *ProcessMonitor) TickActivity() {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.lastActivity = time.Now()
}

// SpawnWatchdog 啟動雙重看門狗 goroutine
// 對齊 Rust: ProcessMonitor::spawn_watchdog()
//
// 回傳 true 表示被超時斬殺，false 表示正常結束（外部通知 Cancel）
//
// # 防禦機制
//  1. 每 5 秒檢查一次靜默超時（Inactivity）
//  2. 同時檢查絕對超時（Absolute），防禦 Slowloris
func (m *ProcessMonitor) SpawnWatchdog(killSignal <-chan struct{}) bool {
	ticker := time.NewTicker(5 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			now := time.Now()

			// 1. 檢查靜默超時 (Inactivity Hang - Codex 防禦)
			m.mu.RLock()
			lastAct := m.lastActivity
			m.mu.RUnlock()

			if now.Sub(lastAct) > m.inactivityTimeout {
				log.Printf("[WARN] Watchdog triggered: Inactivity Timeout (%ds). "+
					"Possible GUI hang or Interactive prompt.",
					int(m.inactivityTimeout.Seconds()))
				return true
			}

			// 2. 檢查絕對超時 (Slowloris/Resource Hog - 惡意 Agent 防禦)
			if now.Sub(m.startTime) > m.absoluteTimeout {
				log.Printf("[WARN] Watchdog triggered: Absolute Timeout (%ds). "+
					"Preventing infinite execution.",
					int(m.absoluteTimeout.Seconds()))
				return true
			}

		case <-killSignal:
			// Process 正常結束或被 Client 主動 Cancel
			return false
		}
	}
}

// SanitizeEnv 清洗環境變數，防禦 GUI 呼叫
// 對齊 Rust: ProcessMonitor::sanitize_env()
//
// 移除的環境變數:
//   - DISPLAY: X11 顯示器
//   - WAYLAND_DISPLAY: Wayland 顯示器
//   - XAUTHORITY: X11 認證
func SanitizeEnv(env map[string]string) {
	blocklist := []string{"DISPLAY", "WAYLAND_DISPLAY", "XAUTHORITY"}
	for _, key := range blocklist {
		if _, exists := env[key]; exists {
			log.Printf("[WARN] Agent attempted to inject GUI variable %s. Stripping it.", key)
			delete(env, key)
		}
	}
}
