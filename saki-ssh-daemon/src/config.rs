use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// SakiSSH Daemon 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// 監聽地址 (預設: "[::0]:19284")
    #[serde(default = "default_bind_address")]
    pub bind_address: String,

    /// Shell 設定
    #[serde(default)]
    pub shell: ShellConfig,

    /// 存取控制
    #[serde(default)]
    pub acl: AclConfig,

    /// 檔案傳輸設定
    #[serde(default)]
    pub file_transfer: FileTransferConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    /// Shell 類型: "powershell", "nushell", "bash", "cmd"
    #[serde(default = "default_shell_type")]
    pub r#type: String,

    /// Shell 路徑 (null = 自動偵測)
    #[serde(default)]
    pub path: Option<String>,

    /// Shell 指令參數 (null = 依類型推算)
    #[serde(default)]
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclConfig {
    /// 允許的 CIDR 列表 (空 = 允許全部)
    #[serde(default)]
    pub allowed_cidrs: Vec<String>,

    /// ED25519 公鑰列表 (hex 編碼，未來擴展)
    #[serde(default)]
    pub ed25519_public_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferConfig {
    /// 允許的路徑前綴 (空 = 允許全部)
    #[serde(default)]
    pub allowed_paths: Vec<String>,

    /// 最大 chunk 大小 (bytes)
    #[serde(default = "default_max_chunk_size")]
    pub max_chunk_size: u64,
}

fn default_bind_address() -> String {
    "[::0]:19284".to_string()
}

fn default_shell_type() -> String {
    if cfg!(windows) {
        "powershell".to_string()
    } else {
        "bash".to_string()
    }
}

fn default_max_chunk_size() -> u64 {
    65536
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            bind_address: default_bind_address(),
            shell: ShellConfig::default(),
            acl: AclConfig::default(),
            file_transfer: FileTransferConfig::default(),
        }
    }
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            r#type: default_shell_type(),
            path: None,
            args: None,
        }
    }
}

impl Default for AclConfig {
    fn default() -> Self {
        Self {
            allowed_cidrs: vec![],
            ed25519_public_keys: vec![],
        }
    }
}

impl Default for FileTransferConfig {
    fn default() -> Self {
        Self {
            allowed_paths: vec![],
            max_chunk_size: default_max_chunk_size(),
        }
    }
}

impl ShellConfig {
    /// 取得 Shell 可執行路徑
    pub fn executable(&self) -> String {
        if let Some(ref p) = self.path {
            return p.clone();
        }
        match self.r#type.as_str() {
            "powershell" => {
                if cfg!(windows) {
                    // 優先使用 PowerShell 7+，回退至 Windows PowerShell
                    if Path::new("C:\\Program Files\\PowerShell\\7\\pwsh.exe").exists() {
                        "C:\\Program Files\\PowerShell\\7\\pwsh.exe".to_string()
                    } else {
                        "powershell.exe".to_string()
                    }
                } else {
                    "pwsh".to_string()
                }
            }
            "nushell" | "nu" => {
                if cfg!(windows) {
                    "C:\\Program Files\\nu\\bin\\nu.exe".to_string()
                } else {
                    "nu".to_string()
                }
            }
            "bash" => "bash".to_string(),
            "cmd" => "cmd.exe".to_string(),
            other => other.to_string(),
        }
    }

    /// 取得 Shell 指令包裝參數
    pub fn command_args(&self) -> Vec<String> {
        if let Some(ref a) = self.args {
            return a.clone();
        }
        match self.r#type.as_str() {
            "powershell" => vec![
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-Command".to_string(),
            ],
            "nushell" | "nu" => vec!["-c".to_string()],
            "bash" => vec!["-c".to_string()],
            "cmd" => vec!["/C".to_string()],
            _ => vec!["-c".to_string()],
        }
    }
}

impl DaemonConfig {
    /// 從檔案載入，不存在則建立預設
    pub fn load_or_create(config_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            let config: DaemonConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config = DaemonConfig::default();
            let content = serde_json::to_string_pretty(&config)?;
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(config_path, content)?;
            Ok(config)
        }
    }

    /// 取得 config 檔案路徑（同 exe 路徑下的 config.json）
    pub fn default_path() -> PathBuf {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        exe_dir.join("config.json")
    }
}
