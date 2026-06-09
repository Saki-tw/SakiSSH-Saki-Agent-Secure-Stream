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
            // 讓 npm cache 寫入 /tmp/sass_vol，不進入分支
            env.insert("npm_config_cache".to_string(), "/tmp/sass_vol/npm".to_string());
            env.insert("YARN_CACHE_FOLDER".to_string(), "/tmp/sass_vol/yarn".to_string());
        } else if cmd_lower.contains("cargo") || cmd_lower.contains("rustc") {
            info!("EnvInjector: Detected Rust build system. Redirecting target dir to volatile memory.");
            // 讓 Rust build artifacts 寫入外部 RAM Disk，保護原始碼目錄
            env.insert("CARGO_TARGET_DIR".to_string(), "/tmp/sass_vol/ct".to_string());
            env.insert("CARGO_HOME".to_string(), "/tmp/sass_vol/ch".to_string());
        } else if cmd_lower.contains("pip") {
            info!("EnvInjector: Detected Python pip. Redirecting cache to volatile memory.");
            env.insert("PIP_CACHE_DIR".to_string(), "/tmp/sass_vol/pip".to_string());
        }

        // 強制全局 TMPDIR 到隔離區域
        env.insert("TMPDIR".to_string(), "/tmp/sass_vol/tmp".to_string());
        
        env
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // C.7 EnvInjector 測試
    // ========================================

    #[test]
    fn inject_npm_command_injects_npm_and_yarn_vars() {
        let env = HashMap::new();
        let result = EnvInjector::inject_volume_reduction_env("npm install express", env);

        assert!(result.contains_key("npm_config_cache"),
            "應注入 npm_config_cache");
        assert!(result.contains_key("YARN_CACHE_FOLDER"),
            "應注入 YARN_CACHE_FOLDER");
        assert!(result["npm_config_cache"].contains("sass_vol"),
            "npm cache 路徑應包含 sass_vol 前綴");
    }

    #[test]
    fn inject_cargo_command_injects_cargo_vars() {
        let env = HashMap::new();
        let result = EnvInjector::inject_volume_reduction_env("cargo build --release", env);

        assert!(result.contains_key("CARGO_TARGET_DIR"),
            "應注入 CARGO_TARGET_DIR");
        assert!(result.contains_key("CARGO_HOME"),
            "應注入 CARGO_HOME");
    }

    #[test]
    fn inject_pip_command_injects_pip_var() {
        let env = HashMap::new();
        let result = EnvInjector::inject_volume_reduction_env("pip install requests", env);

        assert!(result.contains_key("PIP_CACHE_DIR"),
            "應注入 PIP_CACHE_DIR");
    }

    #[test]
    fn inject_any_command_always_injects_tmpdir() {
        let env = HashMap::new();
        let result = EnvInjector::inject_volume_reduction_env("ls -la", env);

        assert!(result.contains_key("TMPDIR"),
            "所有指令都應注入 TMPDIR");
    }

    #[test]
    fn inject_yarn_command_matches_node_detection() {
        let env = HashMap::new();
        let result = EnvInjector::inject_volume_reduction_env("yarn add lodash", env);

        assert!(result.contains_key("npm_config_cache"),
            "yarn 應觸發 Node.js 偵測");
        assert!(result.contains_key("YARN_CACHE_FOLDER"),
            "yarn 應注入 YARN_CACHE_FOLDER");
    }

    #[test]
    fn inject_preserves_existing_env() {
        let mut env = HashMap::new();
        env.insert("EXISTING_VAR".to_string(), "existing_value".to_string());
        let result = EnvInjector::inject_volume_reduction_env("ls", env);

        assert_eq!(result["EXISTING_VAR"], "existing_value",
            "現有環境變數應被保留");
        assert!(result.contains_key("TMPDIR"),
            "應同時注入新變數");
    }

    #[test]
    fn inject_rustc_command_triggers_cargo_detection() {
        let env = HashMap::new();
        let result = EnvInjector::inject_volume_reduction_env("rustc --version", env);

        assert!(result.contains_key("CARGO_TARGET_DIR"),
            "rustc 應觸發 Rust 建置系統偵測");
    }

    #[test]
    fn inject_pnpm_command_triggers_node_detection() {
        let env = HashMap::new();
        let result = EnvInjector::inject_volume_reduction_env("pnpm install", env);

        assert!(result.contains_key("npm_config_cache"),
            "pnpm 應觸發 Node.js 偵測");
    }
}

