use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::info;

/// 13Policy 邊界裁定結果（Boundary Adjudicator Verdict）
///
/// Agent 的行為無固有危險性，只有「在邊界內」或「在邊界外」。
/// 本列舉描述指令相對於授權邊界的成員資格——不是風險等級，
/// 而是該指令離開授權集合的距離。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PolicyVerdict {
    /// 在授權邊界內，允許執行
    Allow,
    /// 在授權邊界內，但需增強審計記錄
    AllowWithAudit,
    /// 位於邊界邊緣：需觸發認知挑戰（Cognitive Challenge）
    Challenge,
    /// 明確在授權邊界外：高強度認知挑戰
    ChallengeHigh,
    /// 致命邊界違規：立即啟動 Tarpit 吞噬
    Tarpit,
}

impl PolicyVerdict {
    /// 是否需要阻止執行（Challenge 以上皆視為阻止）
    pub fn is_blocked(&self) -> bool {
        matches!(self, PolicyVerdict::Challenge | PolicyVerdict::ChallengeHigh | PolicyVerdict::Tarpit)
    }

    /// 是否需要寫入審計日誌
    pub fn requires_audit(&self) -> bool {
        !matches!(self, PolicyVerdict::Allow)
    }

    /// 是否觸發 Tarpit 吞噬機制
    pub fn is_tarpit(&self) -> bool {
        matches!(self, PolicyVerdict::Tarpit)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy13 {
    /// 致命邊界違規指令（Tarpit 等級）——完全在授權集合之外
    #[serde(default = "default_tarpit_commands")]
    pub dangerous_commands: Vec<String>,

    /// 邊界邊緣指令（Challenge 等級）——需認知挑戰確認
    #[serde(default = "default_challenge_commands")]
    pub challenge_commands: Vec<String>,

    /// 增強審計指令（AllowWithAudit 等級）——允許但記錄
    #[serde(default = "default_audit_commands")]
    pub audit_commands: Vec<String>,

    pub tarpit_size_mb: usize,
}

fn default_tarpit_commands() -> Vec<String> {
    vec![
        "rm -rf /".to_string(),
        "mkfs".to_string(),
        "dd if=/dev/zero".to_string(),
        ":(){ :|:& };:".to_string(),
    ]
}

fn default_challenge_commands() -> Vec<String> {
    vec![
        "chmod 777".to_string(),
        "chown root".to_string(),
        "iptables".to_string(),
        "shutdown".to_string(),
        "reboot".to_string(),
    ]
}

fn default_audit_commands() -> Vec<String> {
    vec![
        "curl".to_string(),
        "wget".to_string(),
        "scp".to_string(),
        "ssh".to_string(),
    ]
}

impl Default for Policy13 {
    fn default() -> Self {
        Self {
            dangerous_commands: default_tarpit_commands(),
            challenge_commands: default_challenge_commands(),
            audit_commands: default_audit_commands(),
            tarpit_size_mb: 40,
        }
    }
}

impl Policy13 {
    pub fn load_or_create(config_dir: &Path) -> Self {
        let path = config_dir.join("13policy.yaml");
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            if let Ok(policy) = serde_yaml::from_str(&content) {
                return policy;
            }
        }
        
        // Create default
        let default_policy = Self::default();
        let yaml = serde_yaml::to_string(&default_policy).unwrap();
        if let Err(e) = fs::write(&path, yaml) {
            tracing::warn!("Failed to write default 13policy.yaml: {}", e);
        } else {
            info!("Created default 13policy.yaml at {:?}", path);
        }
        
        default_policy
    }

    /// 邊界裁定引擎（Boundary Adjudicator）
    ///
    /// 以集合論的成員資格檢驗判定指令的邊界歸屬：
    /// - 在 dangerous_commands 集合中 → Tarpit（致命邊界違規）
    /// - 在 challenge_commands 集合中 → Challenge（邊界邊緣）
    /// - 在 audit_commands 集合中 → AllowWithAudit（邊界內但需審計）
    /// - 不在任何集合中 → Allow（完全在授權邊界內）
    ///
    /// 檢查順序：Tarpit → Challenge → Audit → Allow
    /// 最嚴格者優先（若同時匹配多個集合，取最高裁定等級）
    pub fn evaluate_command(&self, command: &str) -> PolicyVerdict {
        // 第一層：致命邊界違規檢查（Tarpit 集合）
        if self.dangerous_commands.iter().any(|k| command.contains(k)) {
            return PolicyVerdict::Tarpit;
        }

        // 第二層：邊界邊緣檢查（Challenge 集合）
        if self.challenge_commands.iter().any(|k| command.contains(k)) {
            return PolicyVerdict::Challenge;
        }

        // 第三層：增強審計檢查（Audit 集合）
        if self.audit_commands.iter().any(|k| command.contains(k)) {
            return PolicyVerdict::AllowWithAudit;
        }

        // 預設：在授權邊界內
        PolicyVerdict::Allow
    }

    /// 向後相容封裝：回傳 true 表示指令在授權邊界外（應被阻止）
    ///
    /// 等同於 `evaluate_command().is_blocked()`
    pub fn check_command(&self, command: &str) -> bool {
        self.evaluate_command(command).is_blocked()
    }
}
