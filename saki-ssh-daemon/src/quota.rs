//! SASS v6.0 資源配額與排隊管理器 (Resource Quota & Queuing Manager)
//!
//! 防禦 Claude Code 的 Agent Teams 高併發請求，避免 OS 資源耗盡。
//! 【主攻防禦升級】：加入 Queue 深度上限，防禦記憶體爆破 (OOM Attack)。

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

/// 每個 ED25519 身分的配額狀態
#[derive(Debug)]
pub struct IdentityQuota {
    active_ptys: usize,
    max_ptys: usize,
    /// 等待佇列中的請求
    wait_queue: VecDeque<Arc<Notify>>,
}

/// 資源配額管理器 (Thread-safe)
#[derive(Clone)]
pub struct ResourceQuotaManager {
    quotas: Arc<Mutex<HashMap<String, IdentityQuota>>>,
    default_max_ptys: usize,
    /// 防禦 Agent 惡意 spam 導致佇列爆破記憶體的上限
    max_queue_depth: usize,
}

impl ResourceQuotaManager {
    pub fn new(default_max_ptys: usize, max_queue_depth: usize) -> Self {
        Self {
            quotas: Arc::new(Mutex::new(HashMap::new())),
            default_max_ptys,
            max_queue_depth,
        }
    }

    /// 申請執行權限。若超過配額，回傳的 Notify 將用於等待 (Queuing)。
    /// 【主攻防禦】：若排隊深度超過 `max_queue_depth`，回傳 Error (防禦 DDoS)。
    pub async fn acquire_pty(&self, identity_pubkey: &str) -> Result<Option<Arc<Notify>>, String> {
        let mut quotas = self.quotas.lock().await;
        let quota = quotas.entry(identity_pubkey.to_string()).or_insert(IdentityQuota {
            active_ptys: 0,
            max_ptys: self.default_max_ptys,
            wait_queue: VecDeque::new(),
        });

        if quota.active_ptys < quota.max_ptys {
            quota.active_ptys += 1;
            Ok(None) // 直接放行
        } else {
            if quota.wait_queue.len() >= self.max_queue_depth {
                // 惡意請求風暴 (Request Storm)，拒絕服務
                return Err("Quota Exceeded and Queue is Full. Possible DoS attack detected.".to_string());
            }
            let notify = Arc::new(Notify::new());
            quota.wait_queue.push_back(notify.clone());
            Ok(Some(notify)) // 需要等待
        }
    }

    /// 釋放執行權限，並喚醒 Queue 中的下一個請求
    pub async fn release_pty(&self, identity_pubkey: &str) {
        let mut quotas = self.quotas.lock().await;
        if let Some(quota) = quotas.get_mut(identity_pubkey) {
            if let Some(notify) = quota.wait_queue.pop_front() {
                // 有人在排隊，直接把名額轉讓給他並喚醒
                notify.notify_one();
            } else {
                if quota.active_ptys > 0 {
                    quota.active_ptys -= 1;
                }
            }
        }
    }

    /// 取得目前排隊的位置
    pub async fn get_queue_position(&self, identity_pubkey: &str) -> usize {
        let quotas = self.quotas.lock().await;
        quotas.get(identity_pubkey).map(|q| q.wait_queue.len()).unwrap_or(0)
    }
}
