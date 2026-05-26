/// SakiAgentSSH v3.0 — ED25519 SSH 風格認證模組
///
/// 安全模型：
/// 1. Daemon 產生 random nonce (challenge)
/// 2. Client 以 ED25519 私鑰簽名 nonce
/// 3. Daemon 驗證簽名，查找 public key 對應的 CapabilitySet
/// 4. 建立 session，分配 session_id

use crate::capability::CapabilitySet;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// 已認證的 Agent session
#[derive(Debug, Clone)]
pub struct AuthenticatedAgent {
    pub name: String,
    pub public_key: [u8; 32],
    pub session_id: String,
    pub capabilities: CapabilitySet,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub last_activity: SystemTime,
}

impl AuthenticatedAgent {
    /// 檢查 session 是否已過期
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }

    /// 更新最後活動時間
    pub fn touch(&mut self) {
        self.last_activity = SystemTime::now();
    }

    /// 檢查是否閒置逾時
    pub fn is_idle_timeout(&self) -> bool {
        let idle_duration = Duration::from_secs(self.capabilities.idle_timeout);
        SystemTime::now()
            .duration_since(self.last_activity)
            .map(|d| d > idle_duration)
            .unwrap_or(false)
    }
}

/// 授權 Agent 配置（從 authorized_agents.json 載入）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent 可讀名稱
    pub name: String,
    /// ED25519 公鑰 (hex 編碼)
    pub public_key_hex: String,
    /// 此 Agent 的 capability set
    #[serde(default)]
    pub capabilities: CapabilitySet,
}

/// 授權 Agents 配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizedAgentsConfig {
    #[serde(default)]
    pub agents: Vec<AgentConfig>,
}

impl Default for AuthorizedAgentsConfig {
    fn default() -> Self {
        Self { agents: vec![] }
    }
}

/// Session 管理器（共享狀態）
pub type SessionMap = Arc<RwLock<HashMap<String, AuthenticatedAgent>>>;

/// Agent 認證器
pub struct AgentAuthenticator {
    /// pubkey (32 bytes) → AgentConfig
    authorized_agents: HashMap<[u8; 32], AgentConfig>,
    /// 活躍 session
    pub sessions: SessionMap,
}

impl AgentAuthenticator {
    /// 從配置載入
    pub fn new(config: AuthorizedAgentsConfig) -> Self {
        let mut authorized = HashMap::new();

        for agent in &config.agents {
            match hex::decode(&agent.public_key_hex) {
                Ok(bytes) if bytes.len() == 32 => {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&bytes);
                    authorized.insert(key, agent.clone());
                    info!("Authorized agent: {} (key: {}...)", agent.name, &agent.public_key_hex[..16]);
                }
                _ => {
                    warn!("Invalid public key for agent '{}': {}", agent.name, agent.public_key_hex);
                }
            }
        }

        if authorized.is_empty() {
            info!("Agent auth disabled (no authorized agents configured)");
        } else {
            info!("Agent auth enabled with {} authorized agent(s)", authorized.len());
        }

        Self {
            authorized_agents: authorized,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 驗證是否啟用了 agent 認證
    pub fn is_enabled(&self) -> bool {
        !self.authorized_agents.is_empty()
    }

    /// 產生 challenge nonce (32 bytes random)
    pub fn generate_nonce() -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut nonce = vec![0u8; 32];
        rng.fill(&mut nonce[..]);
        nonce
    }

    /// 驗證 Agent 認證請求
    pub async fn verify(
        &self,
        agent_name: &str,
        public_key: &[u8],
        signature: &[u8],
        nonce: &[u8],
    ) -> Result<AuthenticatedAgent, AuthError> {
        // 1. 檢查 public key 長度
        if public_key.len() != 32 {
            return Err(AuthError::InvalidKey("Public key must be 32 bytes".into()));
        }
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(public_key);

        // 2. 查找 authorized agent
        let agent_config = self.authorized_agents
            .get(&key_bytes)
            .ok_or_else(|| AuthError::KeyNotFound)?;

        // 3. 驗證 ED25519 簽名
        let verifying_key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|e| AuthError::InvalidKey(format!("Invalid ED25519 key: {}", e)))?;

        let sig = Signature::from_bytes(
            signature.try_into()
                .map_err(|_| AuthError::InvalidSignature("Signature must be 64 bytes".into()))?
        );

        verifying_key.verify(nonce, &sig)
            .map_err(|_| AuthError::InvalidSignature("ED25519 signature verification failed".into()))?;

        // 4. 檢查 session 數量限制
        let sessions = self.sessions.read().await;
        let agent_sessions = sessions.values()
            .filter(|s| s.public_key == key_bytes && !s.is_expired())
            .count() as u32;

        if agent_sessions >= agent_config.capabilities.max_sessions {
            return Err(AuthError::SessionLimit(agent_config.capabilities.max_sessions));
        }
        drop(sessions);

        // 5. 建立新 session
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now();
        let expires_at = now + Duration::from_secs(agent_config.capabilities.max_session_duration);

        let session = AuthenticatedAgent {
            name: if agent_name.is_empty() { agent_config.name.clone() } else { agent_name.to_string() },
            public_key: key_bytes,
            session_id: session_id.clone(),
            capabilities: agent_config.capabilities.clone(),
            created_at: now,
            expires_at,
            last_activity: now,
        };

        // 6. 註冊 session
        self.sessions.write().await.insert(session_id.clone(), session.clone());

        info!("Agent '{}' authenticated, session: {} (expires: {}s)",
            session.name, session_id, agent_config.capabilities.max_session_duration);

        Ok(session)
    }

    /// 從 session ID 查找已認證 agent 並驗證有效性
    pub async fn get_session(&self, session_id: &str) -> Result<AuthenticatedAgent, AuthError> {
        let mut sessions = self.sessions.write().await;

        let session = sessions.get_mut(session_id)
            .ok_or(AuthError::SessionNotFound)?;

        if session.is_expired() {
            sessions.remove(session_id);
            return Err(AuthError::SessionExpired);
        }

        if session.is_idle_timeout() {
            sessions.remove(session_id);
            return Err(AuthError::SessionExpired);
        }

        session.touch();
        Ok(session.clone())
    }

    /// 續約 session
    pub async fn renew_session(&self, session_id: &str) -> Result<SystemTime, AuthError> {
        let mut sessions = self.sessions.write().await;

        let session = sessions.get_mut(session_id)
            .ok_or(AuthError::SessionNotFound)?;

        if session.is_expired() {
            sessions.remove(session_id);
            return Err(AuthError::SessionExpired);
        }

        let new_expires = SystemTime::now() + Duration::from_secs(session.capabilities.max_session_duration);
        session.expires_at = new_expires;
        session.touch();

        info!("Session {} renewed for agent '{}'", session_id, session.name);
        Ok(new_expires)
    }

    /// 計算 capability set 的 SHA256 雜湊
    pub fn capability_hash(cap: &CapabilitySet) -> Vec<u8> {
        let json = serde_json::to_string(cap).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        hasher.finalize().to_vec()
    }

    /// 清理過期 session
    pub async fn cleanup_expired(&self) {
        let mut sessions = self.sessions.write().await;
        let before = sessions.len();
        sessions.retain(|_, s| !s.is_expired() && !s.is_idle_timeout());
        let removed = before - sessions.len();
        if removed > 0 {
            info!("Cleaned up {} expired session(s)", removed);
        }
    }

    /// 觸發 13Policy 高風險行為檢查
    pub fn check_13policy(&self, command: &str) -> bool {
        // v4.0: 攔截危險指令與不受限的執行嘗試
        let dangerous_keywords = ["rm -rf /", "mkfs", "dd if=/dev/zero", ":(){ :|:& };:"];
        dangerous_keywords.iter().any(|k| command.contains(k))
    }
}

/// 從檔案載入 authorized_agents 配置
pub fn load_authorized_agents(config_dir: &Path) -> AuthorizedAgentsConfig {
    let path = config_dir.join("authorized_agents.json");
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => {
                    info!("Loaded authorized agents from {:?}", path);
                    return config;
                }
                Err(e) => {
                    warn!("Failed to parse authorized_agents.json: {}", e);
                }
            },
            Err(e) => {
                warn!("Failed to read authorized_agents.json: {}", e);
            }
        }
    } else {
        info!("No authorized_agents.json found at {:?}, agent auth disabled", path);
    }
    AuthorizedAgentsConfig::default()
}

/// 認證錯誤
#[derive(Debug)]
pub enum AuthError {
    KeyNotFound,
    InvalidKey(String),
    InvalidSignature(String),
    SessionNotFound,
    SessionExpired,
    SessionLimit(u32),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::KeyNotFound => write!(f, "Public key not in authorized agents"),
            AuthError::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
            AuthError::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
            AuthError::SessionNotFound => write!(f, "Session not found"),
            AuthError::SessionExpired => write!(f, "Session expired"),
            AuthError::SessionLimit(max) => write!(f, "Maximum sessions ({}) reached", max),
        }
    }
}
