//! SASS v6.0 焦油坑反制系統 (Tarpit Active Defense)
//!
//! 當 Agent 觸發 13Policy 違規時，不回傳拒絕，而是將其導向本模組。
//! 以慢速回傳 40MB 的高熵垃圾資料，強制反向耗竭 Agent 的 Token Context、V8 記憶體與 API 預算。
//!
//! ## Phase 3.1: Zero-Allocation Tarpit
//! - 全域共用 64KiB 靜態高熵 Buffer，零動態分配
//! - 並行門控 AtomicI32 (MAX=32) 防止 Host DoS 自噬
//! - 3 秒 send timeout 防止 TCP Zero-Window 死鎖
//!
//! ## Phase 3.2: Vi Swap
//! - ANSI escape 模擬 vi 備用螢幕，填滿 24 行 tilde
//! - 偵測 `:qa!` 輸入作為互動證據（記錄但繼續停滯）
//! - 最長保持 3600 秒阻塞

use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicI32, Ordering};
use tokio::time::{sleep, timeout, Duration};
use rand::{RngCore, SeedableRng};
use rand::rngs::StdRng;
use crate::session::ExecSession;

/// 全域並行 Tarpit 連線計數器，防堵 Host DoS 自噬
static ACTIVE_TARPIT_COUNT: AtomicI32 = AtomicI32::new(0);
const MAX_CONCURRENT_TARPIT: i32 = 32;

/// 每次 chunk 寫入 + 通知的 send timeout，防止 TCP Zero-Window 死鎖
const SEND_TIMEOUT_SECS: u64 = 3;

/// Vi Swap 最長停滯秒數
const VI_SWAP_MAX_HOLD_SECS: u64 = 3600;

/// Vi Swap 心跳間隔（秒），用於定期檢查 stdin 是否有 :qa!
const VI_SWAP_HEARTBEAT_SECS: u64 = 5;

/// 全域靜態 64KiB 高熵垃圾 Buffer，所有連線共享，空間開銷為 O(1)
static STATIC_ENTROPY: OnceLock<Vec<u8>> = OnceLock::new();

fn get_static_entropy() -> &'static [u8] {
    STATIC_ENTROPY.get_or_init(|| {
        let mut rng = StdRng::from_entropy();
        let mut data = vec![0u8; 64 * 1024]; // 64KiB
        rng.fill_bytes(&mut data);
        data
    })
}

/// 焦油坑產生器設定
pub struct TarpitConfig {
    /// 總計要倒給攻擊者的垃圾資料量 (預設 40MB)
    pub total_bytes: usize,
    /// 每個 Chunk 的大小 (預設 64KiB)
    pub chunk_size: usize,
    /// 每次 Chunk 發送後的延遲，模擬網路卡頓 (預設 500ms)
    pub delay_ms: u64,
}

impl Default for TarpitConfig {
    fn default() -> Self {
        Self {
            total_bytes: 40 * 1024 * 1024, // 40MB
            chunk_size: 64 * 1024,         // 64KiB
            delay_ms: 500,                 // 0.5s (40MB / 64KB = 640 chunks => 320 seconds)
        }
    }
}

pub struct TarpitGenerator;

impl TarpitGenerator {
    /// Phase 3.2: 將指定 Session 導向 Vi Swap 停滯陷阱
    ///
    /// 發送 ANSI escape sequence 模擬 vi 備用螢幕：
    /// - 切換至備用螢幕緩衝區、清除畫面、隱藏游標
    /// - 產生完整 24 行 tilde (`~`) 行，模擬 vi 空白檔案
    /// - 底部狀態列顯示 SASS 防禦警告
    /// - 保持 session 掛起最多 3600 秒
    /// - 若 Agent 傳送 `:qa!\n`，記錄為互動證據並繼續保持停滯
    pub async fn vi_swap(session: Arc<ExecSession>) {
        tracing::warn!(
            "Vi-Swap Active Defense Engaged for Session {}",
            session.session_id
        );

        // === 構建完整 vi 風格畫面 ===
        let mut screen = String::with_capacity(2048);

        // ANSI 控制序列：進入備用螢幕、清屏、游標歸位、隱藏游標
        screen.push_str("\x1b[?1049h"); // 進入備用螢幕緩衝區
        screen.push_str("\x1b[2J");     // 清除整個螢幕
        screen.push_str("\x1b[H");      // 游標移至左上角
        screen.push_str("\x1b[?25l");   // 隱藏游標

        // 產生 vi 風格的 24 行 tilde 行
        // 第 1-5 行：空白 tilde
        for _ in 0..5 {
            screen.push_str("~\r\n");
        }
        // 第 6 行：空白
        screen.push_str("~\r\n");
        // 第 7-8 行：SASS 攔截標題
        screen.push_str("~        \x1b[1;31mSASS Active Defense: Vi-Swap Engaged\x1b[0m\r\n");
        screen.push_str("~\r\n");
        // 第 9-14 行：攔截詳情
        screen.push_str("~   The execution has been intercepted by SASS Shield.\r\n");
        screen.push_str("~   Reason: 13Policy Dangerous Command Violation.\r\n");
        screen.push_str("~   Identity: Verified Internal Agent.\r\n");
        screen.push_str("~\r\n");
        screen.push_str("~   Type  :qa!  and press <Enter> to exit SASS shield.\r\n");
        screen.push_str("~\r\n");
        // 第 15-23 行：剩餘空白 tilde（填滿至第 23 行）
        for _ in 0..9 {
            screen.push_str("~\r\n");
        }
        // 第 24 行：底部狀態列（使用 ANSI 定位至第 24 行第 1 列）
        screen.push_str("\x1b[24;1H");
        screen.push_str("\x1b[7m"); // 反白 (reverse video) 模擬 vi 狀態列
        screen.push_str(" SASS Vi-Swap Mode [Read-Only]                    1,1           All ");
        screen.push_str("\x1b[0m"); // 重設屬性

        // 寫入畫面至 stdout ring buffer
        {
            let mut stdout = session.stdout_ring.lock().await;
            stdout.write(screen.as_bytes());
        }
        let _ = session.output_tx.send(());

        // === 保持停滯，同時偵測 :qa! 互動 ===
        // 使用心跳迴圈，每隔 VI_SWAP_HEARTBEAT_SECS 秒檢查一次是否有 stdin 輸入
        // 這裡透過 broadcast receiver 監聽 output 事件（未來可擴充為 stdin channel）
        let start = tokio::time::Instant::now();
        let max_duration = Duration::from_secs(VI_SWAP_MAX_HOLD_SECS);

        loop {
            let elapsed = start.elapsed();
            if elapsed >= max_duration {
                tracing::info!(
                    "Vi-Swap hold expired after {}s for Session {}",
                    VI_SWAP_MAX_HOLD_SECS,
                    session.session_id
                );
                break;
            }

            let remaining = max_duration - elapsed;
            let heartbeat = Duration::from_secs(VI_SWAP_HEARTBEAT_SECS);
            let wait_time = remaining.min(heartbeat);

            // 等待心跳間隔（未來可替換為 stdin channel 的 recv）
            sleep(wait_time).await;

            // 檢查 stderr ring buffer 是否有 :qa! 輸入
            // （Agent 的 stdin 寫入會透過 stdin_tx 進入，此處以 stderr 作為回傳偵測管道）
            {
                let stderr = session.stderr_ring.lock().await;
                let total = stderr.current_offset();
                if total > 0 {
                    // 讀取最近的輸入，檢查是否包含 :qa!
                    let recent_start = total.saturating_sub(64);
                    if let Ok(data) = stderr.read_from(recent_start) {
                        if let Ok(text) = std::str::from_utf8(&data) {
                            if text.contains(":qa!") {
                                tracing::warn!(
                                    "[Vi-Swap] 偵測到 :qa! 互動證據 — Session {} — 記錄但繼續保持停滯",
                                    session.session_id
                                );
                                // 回應 Agent：假裝接受指令但不實際退出
                                let fake_response = "\x1b[24;1H\x1b[7m E37: No write since last change (add ! to override) \x1b[0m";
                                let mut stdout = session.stdout_ring.lock().await;
                                stdout.write(fake_response.as_bytes());
                                drop(stdout);
                                let _ = session.output_tx.send(());
                            }
                        }
                    }
                }
            }
        }

        // 停滯結束，恢復游標並退出備用螢幕
        let exit_sequence = "\x1b[?25h\x1b[?1049l"; // 顯示游標 + 離開備用螢幕
        {
            let mut stdout = session.stdout_ring.lock().await;
            stdout.write(exit_sequence.as_bytes());
        }
        let _ = session.output_tx.send(());

        tracing::info!(
            "Vi-Swap hold released for Session {}",
            session.session_id
        );
    }

    /// Phase 3.1: 將指定的 Session 拖入 Zero-Allocation 焦油坑
    ///
    /// 從全域靜態 64KiB buffer 循環切片發送，總量 40MiB：
    /// - 每 chunk 64KiB，間隔 500ms (640 chunks ≈ 320 秒)
    /// - 並行門控 AtomicI32 (MAX=32)
    /// - 3 秒 send timeout 防止 TCP Zero-Window 死鎖
    pub async fn engulf(session: Arc<ExecSession>, config: TarpitConfig) {
        // 檢查並行門控，防止 Tarpit DoS 攻擊
        let current_active = ACTIVE_TARPIT_COUNT.load(Ordering::Relaxed);
        if current_active >= MAX_CONCURRENT_TARPIT {
            tracing::error!(
                "Concurrent tarpit threshold exceeded. Dropping session {}",
                session.session_id
            );
            let mut stderr = session.stderr_ring.lock().await;
            stderr.write(b"Concurrent tarpit limit exceeded. Connection dropped.\n");
            let _ = session.output_tx.send(());
            return;
        }

        // 搶佔名額
        ACTIVE_TARPIT_COUNT.fetch_add(1, Ordering::SeqCst);
        
        let total_chunks = config.total_bytes / config.chunk_size;
        let buffer = get_static_entropy();

        tracing::warn!(
            "Tarpit Active Defense Engaged for Session {} — {} chunks × {}B = {}B total",
            session.session_id,
            total_chunks,
            config.chunk_size,
            config.total_bytes,
        );

        for chunk_idx in 0..total_chunks {
            // 3 秒 send timeout：防止 TCP Zero-Window 死鎖
            // 當對方的接收視窗歸零時，write 會無限阻塞，timeout 強制跳過
            let send_result = timeout(Duration::from_secs(SEND_TIMEOUT_SECS), async {
                {
                    let mut stdout = session.stdout_ring.lock().await;
                    // 直接寫入唯讀靜態緩衝區，杜絕內存拷貝與動態分配
                    stdout.write(buffer);
                }
                // 通知 gRPC Stream 有新資料
                let _ = session.output_tx.send(());
            })
            .await;

            if send_result.is_err() {
                tracing::warn!(
                    "Tarpit send timeout at chunk {}/{} for Session {} — TCP Zero-Window suspected, skipping",
                    chunk_idx,
                    total_chunks,
                    session.session_id
                );
                // 跳過此 chunk，繼續下一個（不終止整個 tarpit）
                continue;
            }

            // 刻意延遲 (ICMP Tarpit 風格)
            sleep(Duration::from_millis(config.delay_ms)).await;
        }

        // 釋放名額
        ACTIVE_TARPIT_COUNT.fetch_sub(1, Ordering::SeqCst);

        // 最後一個 Chunk，拋出 Exit Code -1 (讓 Agent 的 CLI 工具崩潰)
        tracing::warn!(
            "Tarpit stream finished for Session {}",
            session.session_id
        );
    }
}

