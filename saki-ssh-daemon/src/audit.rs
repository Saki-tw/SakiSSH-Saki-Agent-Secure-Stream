/// SakiAgentSSH v3.0 — 審計日誌模組
///
/// 所有透過 SakiAgentSSH 執行的操作均記錄至 append-only audit log。
/// 日誌格式：JSON Lines（每行一筆事件）

use chrono::Utc;
use ed25519_dalek::{Signer, SigningKey, KEYPAIR_LENGTH, SECRET_KEY_LENGTH};
use rand::rngs::OsRng;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tracing::{info, warn};

/// 審計事件類型
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum AuditEvent {
    /// Agent 認證成功
    AuthSuccess {
        agent_name: String,
        session_id: String,
        public_key_hex: String,
    },
    /// Agent 認證失敗
    AuthFailure {
        agent_name: String,
        reason: String,
        /// Phase 2: 記錄來源 IP 供威脅分析
        remote_addr: String,
    },
    /// 指令執行
    CommandExecute {
        session_id: String,
        agent_name: String,
        command: String,
        args: Vec<String>,
        cwd: String,
        allowed: bool,
        deny_reason: Option<String>,
    },
    /// 檔案操作
    FileOperation {
        session_id: String,
        agent_name: String,
        operation: String, // "upload" or "download"
        path: String,
        allowed: bool,
        deny_reason: Option<String>,
    },
    /// Session 事件
    SessionEvent {
        session_id: String,
        agent_name: String,
        event: String, // "created", "renewed", "expired", "closed"
    },
}

/// 審計日誌記錄 (加入前向安全防護)
#[derive(Debug, Serialize)]
struct AuditRecord {
    timestamp: String,
    #[serde(flatten)]
    event: AuditEvent,
    /// Phase 8: Cryptographic Audit - 本次事件與前一筆 Hash 的 SHA256
    chain_hash: String,
    /// Phase 8: Cryptographic Audit - 由 SASS 伺服器私鑰簽署的 Hash
    signature: String,
}

/// 審計日誌器（透過 channel 非同步寫入）
#[derive(Clone)]
pub struct AuditLogger {
    tx: mpsc::UnboundedSender<AuditEvent>,
}

impl AuditLogger {
    /// 建立新的 AuditLogger，啟動背景寫入 task
    pub fn new(log_dir: &Path) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let log_path = log_dir.join("audit.jsonl");

        // 確保日誌目錄存在
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        info!("Audit log: {:?}", log_path);

        // 啟動背景寫入 task
        tokio::spawn(audit_writer(rx, log_path));

        Self { tx }
    }

    /// 建立不寫入任何內容的 no-op logger
    pub fn noop() -> Self {
        let (tx, _rx) = mpsc::unbounded_channel();
        Self { tx }
    }

    /// 記錄審計事件
    pub fn log(&self, event: AuditEvent) {
        let _ = self.tx.send(event);
    }
}

/// 背景寫入 task（append-only + Hash Chaining）
async fn audit_writer(mut rx: mpsc::UnboundedReceiver<AuditEvent>, log_path: PathBuf) {
    let mut file = match tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .await
    {
        Ok(f) => f,
        Err(e) => {
            warn!("Failed to open audit log {:?}: {}", log_path, e);
            return;
        }
    };

    // Phase 9: 載入或產生持久化 Ed25519 簽章金鑰 (PEM 格式)
    let key_path = dirs::home_dir().unwrap_or_default().join(".config/sass/audit_key.pem");
    if let Some(parent) = key_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let signing_key = if key_path.exists() {
        use ed25519_dalek::pkcs8::DecodePrivateKey;
        let pem_str = std::fs::read_to_string(&key_path).unwrap_or_default();
        SigningKey::from_pkcs8_pem(&pem_str).unwrap_or_else(|_| {
            warn!("Failed to load existing audit_key.pem. Generating a new one.");
            SigningKey::generate(&mut OsRng)
        })
    } else {
        use ed25519_dalek::pkcs8::EncodePrivateKey;
        let key = SigningKey::generate(&mut OsRng);
        if let Ok(pem) = key.to_pkcs8_pem(ed25519_dalek::pkcs8::spki::der::pem::LineEnding::LF) {
            let _ = std::fs::write(&key_path, pem.as_bytes());
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600));
            }
        }
        key
    };

    // 寫入 Public Key 供 GetAuditPublicKey RPC 讀取
    let pub_path = dirs::home_dir().unwrap_or_default().join(".config/sass/audit_pub.pem");
    use ed25519_dalek::pkcs8::EncodePublicKey;
    if let Ok(pub_pem) = signing_key.verifying_key().to_public_key_pem(ed25519_dalek::pkcs8::spki::der::pem::LineEnding::LF) {
        let _ = std::fs::write(&pub_path, pub_pem.as_bytes());
    }
    
    // 初始化 Chain Hash
    let mut previous_hash = String::from("SASS_GENESIS_BLOCK");

    while let Some(event) = rx.recv().await {
        let timestamp = Utc::now().to_rfc3339();
        
        // 1. 將事件序列化為字串，準備計算 Hash
        let event_json = serde_json::to_string(&event).unwrap_or_default();
        
        // 2. 計算 SHA256(Previous_Hash + Event_JSON + Timestamp)
        let mut hasher = Sha256::new();
        hasher.update(previous_hash.as_bytes());
        hasher.update(event_json.as_bytes());
        hasher.update(timestamp.as_bytes());
        let current_hash_bytes = hasher.finalize();
        let current_hash = hex::encode(current_hash_bytes);
        
        // 3. 使用 Server Private Key 簽署 Current Hash
        let signature = signing_key.sign(&current_hash_bytes);
        let signature_hex = hex::encode(signature.to_bytes());

        let record = AuditRecord {
            timestamp,
            event,
            chain_hash: current_hash.clone(),
            signature: signature_hex,
        };

        if let Ok(mut json) = serde_json::to_string(&record) {
            json.push('\n');
            if let Err(e) = file.write_all(json.as_bytes()).await {
                warn!("Failed to write audit log: {}", e);
            }
        }
        
        // 更新 Previous Hash，維持 Hash Chain 不斷鏈
        previous_hash = current_hash;
    }
}
