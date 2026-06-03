//! SASS v6.0 靜默超時與無頭環境看門狗 (Headless Watchdog & Anti-Hang)
//!
//! 防禦 Codex 等 Agent 的 Computer Use 或惡意佔用 (Slowloris/Hang)。

use std::time::Duration;
use tokio::time::{sleep, Instant};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 監控執行狀態
#[derive(Debug)]
pub struct ProcessMonitor {
    pub last_activity: Arc<RwLock<Instant>>,
    pub start_time: Instant,
    pub inactivity_timeout: Duration,
    pub absolute_timeout: Duration,
}

impl ProcessMonitor {
    pub fn new(inactivity_secs: u64, absolute_secs: u64) -> Self {
        Self {
            last_activity: Arc::new(RwLock::new(Instant::now())),
            start_time: Instant::now(),
            inactivity_timeout: Duration::from_secs(inactivity_secs),
            absolute_timeout: Duration::from_secs(absolute_secs),
        }
    }

    /// 更新活動時間戳
    pub async fn tick_activity(&self) {
        *self.last_activity.write().await = Instant::now();
    }

    /// 啟動看門狗。回傳 true 代表被超時斬殺，回傳 false 代表正常結束（外部需通知 Cancel）。
    /// 【主攻防禦】：同時檢查「絕對時間 (Absolute Timeout)」，防禦 Agent 每 29 秒送 1 byte 的 Slowloris 攻擊。
    pub async fn spawn_watchdog(monitor: Arc<ProcessMonitor>, mut kill_signal: tokio::sync::mpsc::Receiver<()>) -> bool {
        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(5)) => {
                    let now = Instant::now();
                    let last = *monitor.last_activity.read().await;
                    
                    // 1. 檢查靜默超時 (Inactivity Hang - Codex 防禦)
                    if now.duration_since(last) > monitor.inactivity_timeout {
                        tracing::warn!("Watchdog triggered: Inactivity Timeout ({}s). Possible GUI hang or Interactive prompt.", monitor.inactivity_timeout.as_secs());
                        return true;
                    }
                    
                    // 2. 檢查絕對超時 (Slowloris/Resource Hog - 惡意 Agent 防禦)
                    if now.duration_since(monitor.start_time) > monitor.absolute_timeout {
                        tracing::warn!("Watchdog triggered: Absolute Timeout ({}s). Preventing infinite execution.", monitor.absolute_timeout.as_secs());
                        return true;
                    }
                }
                _ = kill_signal.recv() => {
                    // Process 正常結束或被 Client 主動 Cancel
                    return false;
                }
            }
        }
    }

    /// 清洗環境變數，防禦 GUI 呼叫
    pub fn sanitize_env(env: &mut std::collections::HashMap<String, String>) {
        let blocklist = ["DISPLAY", "WAYLAND_DISPLAY", "XAUTHORITY"];
        for key in blocklist.iter() {
            if env.contains_key(*key) {
                tracing::warn!("Agent attempted to inject GUI variable {}. Stripping it.", key);
                env.remove(*key);
            }
        }
    }
}
