// =============================================================================
// Package defense — capability.go
// SASS v1.4 Capability-Based Permission Model (5 維度 Capability)
//
// 對應 Rust: capability.rs (CapabilitySet)
// RFC 參考: draft-sakistudio-sass-00 §3.2 (capability-model)
//
// 每個 SSH key 綁定一組 CapabilitySet，定義五維邊界:
//   1. 路徑 (Path) — allowed_paths / denied_paths 前綴匹配
//   2. 指令 (Command) — allowed_commands / denied_commands glob 匹配
//   3. 環境 (Environment) — 不繼承 daemon env + 注入 session metadata
//   4. 時間 (Time) — max_duration + idle_timeout
//   5. 並行 (Concurrency) — max_concurrent sessions/processes
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

package defense

import (
	"fmt"
	"path/filepath"
	"strings"
)

// CapabilitySet — Agent 的 capability set，定義該 agent 可執行的操作邊界
// 對齊 Rust: CapabilitySet struct
type CapabilitySet struct {
	// AllowedCommands — 允許執行的指令 glob 模式 (空 = 允許全部)
	AllowedCommands []string `json:"allowed_commands" yaml:"allowed_commands"`

	// DeniedCommands — 拒絕執行的指令 glob 模式 (deny 優先於 allow)
	DeniedCommands []string `json:"denied_commands" yaml:"denied_commands"`

	// AllowedPaths — 允許存取的路徑前綴 (空 = 允許全部)
	AllowedPaths []string `json:"allowed_paths" yaml:"allowed_paths"`

	// DeniedPaths — 拒絕存取的路徑前綴 (deny 優先)
	DeniedPaths []string `json:"denied_paths" yaml:"denied_paths"`

	// MaxConcurrent — 最大並行進程數
	MaxConcurrent int `json:"max_concurrent" yaml:"max_concurrent"`

	// TimeoutSeconds — 單次執行逾時 (秒)
	TimeoutSeconds int `json:"timeout_seconds" yaml:"timeout_seconds"`

	// MaxFileSizeBytes — 最大檔案傳輸大小 (bytes)
	MaxFileSizeBytes int64 `json:"max_file_size_bytes" yaml:"max_file_size_bytes"`

	// InheritEnv — 是否繼承 daemon 環境變數
	InheritEnv bool `json:"inherit_env" yaml:"inherit_env"`

	// AllowedEnvVars — 允許的環境變數名稱（僅在 InheritEnv=false 時生效）
	AllowedEnvVars []string `json:"allowed_env_vars" yaml:"allowed_env_vars"`

	// MaxSessionDuration — Session 最大持續時間 (秒)
	MaxSessionDuration int64 `json:"max_session_duration" yaml:"max_session_duration"`

	// IdleTimeout — Session 閒置逾時 (秒)
	IdleTimeout int64 `json:"idle_timeout" yaml:"idle_timeout"`

	// MaxSessions — 最大同時 session 數
	MaxSessions int `json:"max_sessions" yaml:"max_sessions"`
}

// DefaultCapabilitySet 回傳預設的 CapabilitySet
// 對齊 Rust: impl Default for CapabilitySet
func DefaultCapabilitySet() *CapabilitySet {
	return &CapabilitySet{
		AllowedCommands:    []string{},
		DeniedCommands:     []string{},
		AllowedPaths:       []string{},
		DeniedPaths:        []string{},
		MaxConcurrent:      5,
		TimeoutSeconds:     300,
		MaxFileSizeBytes:   100 * 1024 * 1024, // 100MB
		InheritEnv:         false,
		AllowedEnvVars:     []string{"PATH", "HOME", "LANG", "TERM"},
		MaxSessionDuration: 3600,
		IdleTimeout:        600,
		MaxSessions:        3,
	}
}

// CapabilityError — Capability 檢查錯誤
// 對齊 Rust: CapabilityError enum
type CapabilityError struct {
	Kind    string // "CommandDenied" 或 "PathDenied"
	Message string
}

func (e *CapabilityError) Error() string {
	return fmt.Sprintf("%s: %s", e.Kind, e.Message)
}

// CheckCommand 檢查指令是否在 capability 允許範圍內
// 對齊 Rust: CapabilitySet::check_command()
//
// deny 優先 → 再 check allow
// 若 AllowedCommands 為空，視為允許全部（但仍受 DeniedCommands 限制）
func (c *CapabilitySet) CheckCommand(command string) error {
	// 提取指令的第一個 token（不含參數）
	cmdName := command
	if idx := strings.IndexByte(command, ' '); idx >= 0 {
		cmdName = command[:idx]
	}

	// 1. 先檢查 deny list（deny 優先）
	for _, pattern := range c.DeniedCommands {
		matchedFull, _ := filepath.Match(pattern, command)
		matchedName, _ := filepath.Match(pattern, cmdName)
		if matchedFull || matchedName {
			return &CapabilityError{
				Kind:    "CommandDenied",
				Message: fmt.Sprintf("'%s' matched deny pattern '%s'", cmdName, pattern),
			}
		}
	}

	// 2. 若有 allow list，檢查是否在其中
	if len(c.AllowedCommands) > 0 {
		allowed := false
		for _, pattern := range c.AllowedCommands {
			matchedFull, _ := filepath.Match(pattern, command)
			matchedName, _ := filepath.Match(pattern, cmdName)
			if matchedFull || matchedName {
				allowed = true
				break
			}
		}
		if !allowed {
			return &CapabilityError{
				Kind:    "CommandDenied",
				Message: fmt.Sprintf("'%s' not in allowed commands", cmdName),
			}
		}
	}

	return nil
}

// CheckPath 檢查路徑是否在 capability 允許範圍內
// 對齊 Rust: CapabilitySet::check_path()
//
// DeniedPaths 優先 → 再 check AllowedPaths 前綴匹配
func (c *CapabilitySet) CheckPath(path string) error {
	// 1. 先檢查 deny list
	for _, denied := range c.DeniedPaths {
		if strings.HasPrefix(path, denied) {
			return &CapabilityError{
				Kind:    "PathDenied",
				Message: fmt.Sprintf("'%s' under denied path '%s'", path, denied),
			}
		}
	}

	// 2. 若有 allow list，檢查是否在其中
	if len(c.AllowedPaths) > 0 {
		allowed := false
		for _, allowedPath := range c.AllowedPaths {
			if strings.HasPrefix(path, allowedPath) {
				allowed = true
				break
			}
		}
		if !allowed {
			return &CapabilityError{
				Kind:    "PathDenied",
				Message: fmt.Sprintf("'%s' not under any allowed path", path),
			}
		}
	}

	return nil
}
