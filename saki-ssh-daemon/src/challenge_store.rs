//! ChaCha20 認知挑戰狀態儲存器
//! 
//! 管理待驗證的挑戰，包含 TTL 自動清理機制。
//! v5.0 新增 — 解決 v4.0 「生成但不驗證」的缺陷。

use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, aead::{Aead, KeyInit}};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use rand::Rng;
use std::fs;
use std::path::PathBuf;
use subtle::ConstantTimeEq;

/// 單個挑戰的內部狀態
struct ChallengeEntry {
    nonce: [u8; 12],
    plaintext: Vec<u8>,
    created_at: Instant,
    ttl: Duration,
}

/// 挑戰狀態儲存器（執行緒安全）
#[derive(Clone)]
pub struct ChallengeStore {
    entries: Arc<RwLock<HashMap<Vec<u8>, ChallengeEntry>>>,
    default_ttl: Duration,
    static_key: [u8; 32],
}

impl ChallengeStore {
    /// 載入或產生固定的 ChaCha20 Key
    fn load_or_generate_key() -> [u8; 32] {
        let path = dirs::home_dir().unwrap().join(".sakissh/chacha20.key");
        if path.exists() {
            let data = fs::read(&path).expect("Failed to read chacha20.key");
            if data.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&data[0..32]);
                return key;
            }
        }
        
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, &key).expect("Failed to write chacha20.key");
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        
        key
    }

    /// 建立新的 ChallengeStore
    pub fn new(default_ttl_secs: u64) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Duration::from_secs(default_ttl_secs),
            static_key: Self::load_or_generate_key(),
        }
    }

    /// 產生新的 ChaCha20-Poly1305 認知挑戰
    /// 
    /// 返回 (nonce, encrypted_challenge) 給 Client
    /// Client 必須解密並回傳 plaintext 來證明計算能力
    pub async fn generate_challenge(&self) -> (Vec<u8>, Vec<u8>) {
        let (nonce_bytes, plaintext, ciphertext) = {
            let mut rng = rand::thread_rng();
            
            // 產生隨機 nonce
            let mut nonce_bytes = [0u8; 12];
            rng.fill(&mut nonce_bytes);
            
            // 產生隨機明文（64 bytes）
            let mut plaintext = vec![0u8; 64];
            rng.fill(&mut plaintext[..]);
            
            // 加密
            let key = Key::from_slice(&self.static_key);
            let nonce = Nonce::from_slice(&nonce_bytes);
            let cipher = ChaCha20Poly1305::new(key);
            let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
                .expect("ChaCha20-Poly1305 encryption should never fail");

            (nonce_bytes, plaintext, ciphertext)
        };
        
        // 儲存挑戰狀態
        let entry = ChallengeEntry {
            nonce: nonce_bytes,
            plaintext,
            created_at: Instant::now(),
            ttl: self.default_ttl,
        };
        
        let nonce_vec = nonce_bytes.to_vec();
        self.entries.write().await.insert(nonce_vec.clone(), entry);
        
        // 回傳 nonce + 加密資料
        (nonce_vec, ciphertext)
    }

    /// 驗證 Client 的挑戰回應
    /// 
    /// 返回 true 表示 Agent 具備真實計算能力
    /// Phase 2: 使用 constant-time 比對防止 timing side-channel 攻擊
    pub async fn verify_response(&self, nonce: &[u8], response: &[u8]) -> bool {
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.remove(nonce) {
            // 檢查 TTL
            if entry.created_at.elapsed() > entry.ttl {
                tracing::warn!("ChaCha20 challenge expired (TTL exceeded)");
                return false;
            }
            
            // Phase 2: constant-time 比對，防止 timing side-channel 攻擊
            // 長度不同時仍需避免提早返回洩漏資訊
            if response.len() != entry.plaintext.len() {
                tracing::warn!("ChaCha20 challenge response length mismatch");
                return false;
            }
            response.ct_eq(&entry.plaintext).into()
        } else {
            tracing::warn!("ChaCha20 challenge nonce not found (possible replay attack)");
            false
        }
    }

    /// 遍歷所有待驗證挑戰，嘗試以 constant-time 比對找到匹配的明文
    ///
    /// 由於 ChallengeRequest Proto 目前未包含 nonce 欄位，
    /// 無法精確查找對應挑戰，因此需遍歷所有 pending 條目。
    /// 複雜度為 O(n)，但 n 極小（TTL 60s，實務上 pending 數量 < 10）。
    ///
    /// 找到匹配後，該條目會被移除（一次性驗證，防止重放攻擊）。
    ///
    /// # TODO
    /// Proto 應新增 challenge_nonce 欄位到 ChallengeRequest，
    /// 以實現 O(1) 查找，避免 O(n) 遍歷。
    pub async fn try_verify_any(&self, response: &[u8]) -> bool {
        let mut entries = self.entries.write().await;

        // 先清理過期條目
        entries.retain(|_, entry| entry.created_at.elapsed() < entry.ttl);

        // 遍歷所有 pending 挑戰，constant-time 比對
        let mut matched_key: Option<Vec<u8>> = None;
        for (nonce_key, entry) in entries.iter() {
            if response.len() == entry.plaintext.len()
                && bool::from(response.ct_eq(&entry.plaintext))
            {
                matched_key = Some(nonce_key.clone());
                break;
            }
        }

        if let Some(key) = matched_key {
            entries.remove(&key);
            tracing::info!("ChaCha20 認知挑戰驗證成功（try_verify_any）");
            true
        } else {
            tracing::warn!("try_verify_any: 無匹配的挑戰（可能已過期或為重放攻擊）");
            false
        }
    }

    /// 清理過期的挑戰條目
    pub async fn cleanup_expired(&self) {
        let mut entries = self.entries.write().await;
        entries.retain(|_, entry| entry.created_at.elapsed() < entry.ttl);
    }

    /// 啟動背景清理任務（每 60 秒清理一次）
    pub fn spawn_cleanup_task(self) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                self.cleanup_expired().await;
            }
        });
    }
}
