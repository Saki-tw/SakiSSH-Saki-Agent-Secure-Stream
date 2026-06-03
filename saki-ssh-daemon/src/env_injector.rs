//! Phase 8: 協議層的意圖過濾與 I/O 減量 (Protocol Layer Reduction)
//! 
//! 將 Agent 的垃圾與快取 I/O (如 npm cache, cargo build artifacts)
//! 動態卸載至 RAM Disk 或獨立的 tmpfs，避免污染儲存層的動態分支。

use std::collections::HashMap;
use tracing::info;

pub struct EnvInjector;

impl EnvInjector {
    /// 分析指令意圖，並注入對應的減量環境變數
    pub fn inject_volume_reduction_env(command: &str, mut env: HashMap<String, String>) -> HashMap<String, String> {
        let cmd_lower = command.to_lowercase();
        
        // 判斷是否為高 I/O 消耗的快取/建置指令
        if cmd_lower.contains("npm") || cmd_lower.contains("yarn") || cmd_lower.contains("pnpm") {
            info!("EnvInjector: Detected Node.js package manager. Redirecting cache to volatile memory.");
            // 讓 npm cache 寫入 /tmp/sass_volatile_cache，不進入分支
            env.insert("npm_config_cache".to_string(), "/tmp/sass_volatile_cache/npm".to_string());
            env.insert("YARN_CACHE_FOLDER".to_string(), "/tmp/sass_volatile_cache/yarn".to_string());
        } else if cmd_lower.contains("cargo") || cmd_lower.contains("rustc") {
            info!("EnvInjector: Detected Rust build system. Redirecting target dir to volatile memory.");
            // 讓 Rust build artifacts 寫入外部 RAM Disk，保護原始碼目錄
            env.insert("CARGO_TARGET_DIR".to_string(), "/tmp/sass_volatile_cache/cargo_target".to_string());
            env.insert("CARGO_HOME".to_string(), "/tmp/sass_volatile_cache/cargo_home".to_string());
        } else if cmd_lower.contains("pip") {
            info!("EnvInjector: Detected Python pip. Redirecting cache to volatile memory.");
            env.insert("PIP_CACHE_DIR".to_string(), "/tmp/sass_volatile_cache/pip".to_string());
        }

        // 強制全局 TMPDIR 到隔離區域
        env.insert("TMPDIR".to_string(), "/tmp/sass_volatile_cache/tmp".to_string());
        
        env
    }
}
