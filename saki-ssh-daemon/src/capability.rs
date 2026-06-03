/// SakiAgentSSH v3.0 — Capability-Based Permission Model
///
/// 每個 SSH key 綁定一組 CapabilitySet，定義五維邊界：
/// 1. 路徑 (Path) — allowed_paths / denied_paths 前綴匹配
/// 2. 指令 (Command) — allowed_commands / denied_commands glob 匹配
/// 3. 環境 (Environment) — 不繼承 daemon env + 注入 session metadata
/// 4. 時間 (Time) — max_duration + idle_timeout
/// 5. 並行 (Concurrency) — max_concurrent sessions/processes

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Agent 的 capability set，定義該 agent 可執行的操作邊界
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySet {
    /// 允許執行的指令 glob 模式 (空 = 允許全部)
    #[serde(default)]
    pub allowed_commands: Vec<String>,

    /// 拒絕執行的指令 glob 模式 (deny 優先於 allow)
    #[serde(default)]
    pub denied_commands: Vec<String>,

    /// 允許存取的路徑前綴 (空 = 允許全部)
    #[serde(default)]
    pub allowed_paths: Vec<String>,

    /// 拒絕存取的路徑前綴 (deny 優先)
    #[serde(default)]
    pub denied_paths: Vec<String>,

    /// 最大並行進程數
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: u32,

    /// 單次執行逾時 (秒)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,

    /// 最大檔案傳輸大小 (bytes)
    #[serde(default = "default_max_file_size")]
    pub max_file_size_bytes: u64,

    /// 是否繼承 daemon 環境變數
    #[serde(default)]
    pub inherit_env: bool,

    /// 允許的環境變數名稱（僅在 inherit_env=false 時生效）
    #[serde(default = "default_allowed_env_vars")]
    pub allowed_env_vars: Vec<String>,

    /// Session 最大持續時間 (秒)
    #[serde(default = "default_session_duration")]
    pub max_session_duration: u64,

    /// Session 閒置逾時 (秒)
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,

    /// 最大同時 session 數
    #[serde(default = "default_max_sessions")]
    pub max_sessions: u32,
}

fn default_max_concurrent() -> u32 { 5 }
fn default_timeout() -> u32 { 300 }
fn default_max_file_size() -> u64 { 100 * 1024 * 1024 } // 100MB
fn default_session_duration() -> u64 { 3600 }
fn default_idle_timeout() -> u64 { 600 }
fn default_max_sessions() -> u32 { 3 }

fn default_allowed_env_vars() -> Vec<String> {
    vec![
        "PATH".to_string(),
        "HOME".to_string(),
        "LANG".to_string(),
        "TERM".to_string(),
    ]
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self {
            allowed_commands: vec![],
            denied_commands: vec![],
            allowed_paths: vec![],
            denied_paths: vec![],
            max_concurrent: default_max_concurrent(),
            timeout_seconds: default_timeout(),
            max_file_size_bytes: default_max_file_size(),
            inherit_env: false,
            allowed_env_vars: default_allowed_env_vars(),
            max_session_duration: default_session_duration(),
            idle_timeout: default_idle_timeout(),
            max_sessions: default_max_sessions(),
        }
    }
}

/// Capability 檢查錯誤
#[derive(Debug)]
pub enum CapabilityError {
    CommandDenied(String),
    PathDenied(String),
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilityError::CommandDenied(cmd) => {
                write!(f, "Command '{}' not permitted by capability set", cmd)
            }
            CapabilityError::PathDenied(path) => {
                write!(f, "Path '{}' not permitted by capability set", path)
            }
        }
    }
}

impl CapabilitySet {
    /// 檢查指令是否在 capability 允許範圍內
    ///
    /// deny 優先 → 再 check allow
    /// 若 allowed_commands 為空，視為允許全部（但仍受 denied 限制）
    pub fn check_command(&self, command: &str) -> Result<(), CapabilityError> {
        // 提取指令的第一個 token（不含參數）
        let cmd_name = command.split_whitespace().next().unwrap_or(command);

        // 1. 先檢查 deny list（deny 優先）
        for pattern in &self.denied_commands {
            if let Ok(glob) = glob::Pattern::new(pattern) {
                if glob.matches(command) || glob.matches(cmd_name) {
                    return Err(CapabilityError::CommandDenied(format!(
                        "'{}' matched deny pattern '{}'", cmd_name, pattern
                    )));
                }
            }
        }

        // 2. 若有 allow list，檢查是否在其中
        if !self.allowed_commands.is_empty() {
            let allowed = self.allowed_commands.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|g| g.matches(command) || g.matches(cmd_name))
                    .unwrap_or(false)
            });
            if !allowed {
                return Err(CapabilityError::CommandDenied(format!(
                    "'{}' not in allowed commands", cmd_name
                )));
            }
        }

        Ok(())
    }

    /// 檢查路徑是否在 capability 允許範圍內
    ///
    /// denied_paths 優先 → 再 check allowed_paths 前綴匹配
    pub fn check_path(&self, path: &str) -> Result<(), CapabilityError> {
        let target = Path::new(path);

        // 1. 先檢查 deny list
        for denied in &self.denied_paths {
            let denied_path = PathBuf::from(denied);
            if target.starts_with(&denied_path) {
                return Err(CapabilityError::PathDenied(format!(
                    "'{}' under denied path '{}'", path, denied
                )));
            }
        }

        // 2. 若有 allow list，檢查是否在其中
        if !self.allowed_paths.is_empty() {
            let allowed = self.allowed_paths.iter().any(|allowed| {
                target.starts_with(Path::new(allowed))
            });
            if !allowed {
                return Err(CapabilityError::PathDenied(format!(
                    "'{}' not under any allowed path", path
                )));
            }
        }

        Ok(())
    }
}
