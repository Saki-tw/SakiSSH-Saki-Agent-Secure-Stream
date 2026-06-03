//! SASS v6.0 狀態駐留與冪等恢復管理器 (Idempotent Resumption Manager)
//!
//! 防禦 Antigravity 的 Checkpoint 截斷，支援 gRPC 斷線後無縫重連。
//! 【主攻防禦升級】：Ring Buffer 容量上限與殭屍 Session 清理機制。

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use std::time::{Instant, Duration};

/// Ring Buffer 存放 Process 輸出，具備容量上限
#[derive(Debug)]
pub struct RingBuffer {
    buffer: VecDeque<u8>,
    max_size: usize,
    total_written: u64,
}

impl RingBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_size),
            max_size,
            total_written: 0,
        }
    }

    /// 寫入資料，若超過上限則丟棄最舊的資料 (防禦 OOM)
    pub fn write(&mut self, data: &[u8]) {
        for &byte in data {
            if self.buffer.len() >= self.max_size {
                self.buffer.pop_front();
            }
            self.buffer.push_back(byte);
            self.total_written += 1;
        }
    }

    /// 從指定 offset 讀取，若 offset 太舊已被覆蓋，回傳 Err
    pub fn read_from(&self, offset: u64) -> Result<Vec<u8>, String> {
        if offset > self.total_written {
            return Ok(vec![]); // 還沒寫到那裡
        }
        let oldest_available_offset = self.total_written.saturating_sub(self.buffer.len() as u64);
        if offset < oldest_available_offset {
            return Err("Offset is too old and has been overwritten by Ring Buffer".to_string());
        }
        let index = (offset - oldest_available_offset) as usize;
        let mut result = Vec::with_capacity(self.buffer.len() - index);
        for i in index..self.buffer.len() {
            result.push(self.buffer[i]);
        }
        Ok(result)
    }

    /// 回傳目前已寫入的總位元組數（即下一筆資料的 offset）
    pub fn current_offset(&self) -> u64 {
        self.total_written
    }
}

/// 獨立的執行 Session
pub struct ExecSession {
    pub session_id: String,
    pub stdout_ring: Arc<Mutex<RingBuffer>>,
    pub stderr_ring: Arc<Mutex<RingBuffer>>,
    /// 用於通知 Re-attach 的 Client 有新資料
    pub output_tx: broadcast::Sender<()>,
    pub created_at: Instant,
}

impl ExecSession {
    /// 從指定 offset 回放 stdout Ring Buffer 中的歷史資料
    /// 以 4KiB 為單位分批發送，避免一次性塞爆 channel
    pub async fn replay_from(
        &self,
        offset: u64,
        tx: &tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> Result<(), String> {
        let ring = self.stdout_ring.lock().await;
        let data = ring.read_from(offset)?;
        // 分批發送，每批最多 4096 bytes
        for chunk in data.chunks(4096) {
            tx.send(chunk.to_vec()).await
                .map_err(|e| format!("回放傳送失敗: {}", e))?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Arc<ExecSession>>>>,
    max_sessions_per_agent: usize,
    max_buffer_size: usize,
}

impl SessionManager {
    pub fn new(max_sessions: usize, max_buffer_size: usize) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            max_sessions_per_agent: max_sessions,
            max_buffer_size,
        }
    }

    /// 註冊新 Session，若超過上限則拒絕 (防禦惡意耗盡)
    pub async fn create_session(&self, session_id: String) -> Result<Arc<ExecSession>, String> {
        let mut sessions = self.sessions.lock().await;
        if sessions.len() >= self.max_sessions_per_agent * 10 {
            // 全局防禦限制
            return Err("Global Session limit reached. Possible DoS attack.".to_string());
        }
        
        let (tx, _) = broadcast::channel(16);
        let session = Arc::new(ExecSession {
            session_id: session_id.clone(),
            stdout_ring: Arc::new(Mutex::new(RingBuffer::new(self.max_buffer_size))),
            stderr_ring: Arc::new(Mutex::new(RingBuffer::new(self.max_buffer_size))),
            output_tx: tx,
            created_at: Instant::now(),
        });
        
        sessions.insert(session_id, session.clone());
        Ok(session)
    }

    pub async fn get_session(&self, session_id: &str) -> Option<Arc<ExecSession>> {
        let sessions = self.sessions.lock().await;
        sessions.get(session_id).cloned()
    }

    /// 定期清理殭屍 Session (斷線且超時未重連)
    pub async fn cleanup_zombies(&self, ttl: Duration) {
        let mut sessions = self.sessions.lock().await;
        let now = Instant::now();
        sessions.retain(|_, session| now.duration_since(session.created_at) < ttl);
    }
}
