// =============================================================================
// SakiSshDaemon.Plugins — ViSwap.cs
// SASS Plugin #5: Vi Swap ANSI Escape Sequence
//
// 對應 Rust: tarpit.rs (TarpitGenerator::vi_swap)
// RFC 參考: draft-sakistudio-sass-00 Appendix C.5 (anchor: vi-swap-ansi)
//
// 5 個 ANSI escape:
// 1. \x1b[?1049h — 進入備用螢幕緩衝區 (Enter alternate screen buffer)
// 2. \x1b[2J     — 清除整個螢幕 (Clear entire screen)
// 3. \x1b[H      — 游標移至左上角 (Move cursor to top-left)
// 4. \x1b[?25l   — 隱藏游標 (Hide cursor)
// 5. \x1b[24;1H  — 游標移至底部狀態列 (Move cursor to bottom status line)
//
// 持續 3600 秒（1 小時）
// 偵測 :qa! 輸入作為互動證據（記錄但繼續停滯）
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Logging;

namespace SakiSshDaemon.Plugins
{
    /// <summary>
    /// SASS Plugin #5: Vi Swap ANSI Escape Sequence。
    /// <para>
    /// RFC draft-sakistudio-sass-00 §C.5: 發送 ANSI escape sequence
    /// 模擬 vi 備用螢幕，將 Agent 困在虛假的 vi 介面中。
    /// 保持 session 掛起最多 3600 秒。
    /// </para>
    /// </summary>
    public sealed class ViSwap : IPlugin
    {
        /// <summary>Vi Swap 最長停滯秒數 — 對齊 Rust VI_SWAP_MAX_HOLD_SECS = 3600</summary>
        private const int MaxHoldSeconds = 3600;

        /// <summary>心跳間隔秒數 — 對齊 Rust VI_SWAP_HEARTBEAT_SECS = 5</summary>
        private const int HeartbeatSeconds = 5;

        // =====================================================================
        // RFC Appendix C.5 定義的 5 個 ANSI Escape Sequence
        // =====================================================================

        /// <summary>進入備用螢幕緩衝區 — RFC §C.5 第 1 個</summary>
        private const string AnsiEnterAlternateScreen = "\x1b[?1049h";

        /// <summary>清除整個螢幕 — RFC §C.5 第 2 個</summary>
        private const string AnsiClearScreen = "\x1b[2J";

        /// <summary>游標移至左上角 (1,1) — RFC §C.5 第 3 個</summary>
        private const string AnsiCursorHome = "\x1b[H";

        /// <summary>隱藏游標 — RFC §C.5 第 4 個</summary>
        private const string AnsiHideCursor = "\x1b[?25l";

        /// <summary>游標移至底部狀態列 (第 24 行第 1 列) — RFC §C.5 第 5 個</summary>
        private const string AnsiBottomStatusLine = "\x1b[24;1H";

        /// <summary>反白模式 (reverse video) — vi 狀態列樣式</summary>
        private const string AnsiReverseVideo = "\x1b[7m";

        /// <summary>重設屬性</summary>
        private const string AnsiReset = "\x1b[0m";

        /// <summary>紅色粗體 — SASS 攔截標題</summary>
        private const string AnsiBoldRed = "\x1b[1;31m";

        /// <summary>顯示游標 + 離開備用螢幕 — 結束序列</summary>
        private const string AnsiExitSequence = "\x1b[?25h\x1b[?1049l";

        private readonly ILogger<ViSwap> _logger;
        private bool _disposed;

        public ViSwap(ILogger<ViSwap> logger)
        {
            _logger = logger ?? throw new ArgumentNullException(nameof(logger));
        }

        /// <inheritdoc />
        public string Name => "Vi Swap ANSI Escape";

        /// <inheritdoc />
        public string RfcSection => "Appendix C.5 (vi-swap-ansi)";

        /// <inheritdoc />
        public bool IsHealthy => !_disposed;

        /// <inheritdoc />
        public Task<bool> InitializeAsync(CancellationToken cancellationToken = default)
        {
            _logger.LogInformation(
                "Plugin #5 ({Name}) 初始化完成 — MaxHold={MaxHold}s, Heartbeat={Heartbeat}s",
                Name, MaxHoldSeconds, HeartbeatSeconds);
            return Task.FromResult(true);
        }

        /// <summary>
        /// 構建完整的 vi 風格畫面。
        /// <para>對齊 Rust: TarpitGenerator::vi_swap() 的畫面構建部分</para>
        /// </summary>
        /// <returns>完整的 ANSI escape 畫面字串</returns>
        public string BuildViScreen()
        {
            var screen = new StringBuilder(2048);

            // ANSI 控制序列：進入備用螢幕、清屏、游標歸位、隱藏游標
            screen.Append(AnsiEnterAlternateScreen); // RFC §C.5 #1
            screen.Append(AnsiClearScreen);           // RFC §C.5 #2
            screen.Append(AnsiCursorHome);            // RFC §C.5 #3
            screen.Append(AnsiHideCursor);            // RFC §C.5 #4

            // 產生 vi 風格的 24 行 tilde 行 — 對齊 Rust vi_swap
            // 第 1-5 行：空白 tilde
            for (int i = 0; i < 5; i++)
            {
                screen.Append("~\r\n");
            }

            // 第 6 行：空白
            screen.Append("~\r\n");

            // 第 7-8 行：SASS 攔截標題
            screen.Append($"~        {AnsiBoldRed}SASS Active Defense: Vi-Swap Engaged{AnsiReset}\r\n");
            screen.Append("~\r\n");

            // 第 9-14 行：攔截詳情
            screen.Append("~   The execution has been intercepted by SASS Shield.\r\n");
            screen.Append("~   Reason: 13Policy Dangerous Command Violation.\r\n");
            screen.Append("~   Identity: Verified Internal Agent.\r\n");
            screen.Append("~\r\n");
            screen.Append("~   Type  :qa!  and press <Enter> to exit SASS shield.\r\n");
            screen.Append("~\r\n");

            // 第 15-23 行：剩餘空白 tilde（填滿至第 23 行）
            for (int i = 0; i < 9; i++)
            {
                screen.Append("~\r\n");
            }

            // 第 24 行：底部狀態列 — RFC §C.5 #5
            screen.Append(AnsiBottomStatusLine);
            screen.Append(AnsiReverseVideo);
            screen.Append(" SASS Vi-Swap Mode [Read-Only]                    1,1           All ");
            screen.Append(AnsiReset);

            return screen.ToString();
        }

        /// <summary>
        /// 將指定 Session 導向 Vi Swap 停滯陷阱。
        /// <para>
        /// 對齊 Rust: TarpitGenerator::vi_swap()
        /// 保持 session 掛起最多 3600 秒。
        /// 若 Agent 傳送 :qa!，記錄為互動證據並繼續保持停滯。
        /// </para>
        /// </summary>
        /// <param name="writeCallback">寫入回呼（對應 Rust stdout_ring.write）</param>
        /// <param name="readCallback">
        /// 讀取回呼（對應 Rust stderr_ring 偵測）：回傳最近輸入的文字，null 表示無輸入
        /// </param>
        /// <param name="cancellationToken">取消令牌</param>
        public async Task EngageAsync(
            Func<byte[], Task> writeCallback,
            Func<Task<string?>> readCallback,
            CancellationToken cancellationToken = default)
        {
            _logger.LogWarning("Vi-Swap Active Defense Engaged");

            // 發送完整 vi 風格畫面
            string screen = BuildViScreen();
            await writeCallback(Encoding.UTF8.GetBytes(screen));

            // 保持停滯，同時偵測 :qa! 互動
            var startTime = DateTime.UtcNow;
            var maxDuration = TimeSpan.FromSeconds(MaxHoldSeconds);

            while (!cancellationToken.IsCancellationRequested)
            {
                var elapsed = DateTime.UtcNow - startTime;
                if (elapsed >= maxDuration)
                {
                    _logger.LogInformation(
                        "Vi-Swap hold expired after {MaxHold}s",
                        MaxHoldSeconds);
                    break;
                }

                var remaining = maxDuration - elapsed;
                var waitTime = remaining < TimeSpan.FromSeconds(HeartbeatSeconds)
                    ? remaining
                    : TimeSpan.FromSeconds(HeartbeatSeconds);

                // 等待心跳間隔
                try
                {
                    await Task.Delay(waitTime, cancellationToken);
                }
                catch (OperationCanceledException)
                {
                    break;
                }

                // 檢查是否有 :qa! 輸入 — 對齊 Rust stderr ring buffer 偵測
                string? input = await readCallback();
                if (input != null && input.Contains(":qa!"))
                {
                    _logger.LogWarning(
                        "[Vi-Swap] 偵測到 :qa! 互動證據 — 記錄但繼續保持停滯");

                    // 假裝接受指令但不實際退出 — 對齊 Rust fake_response
                    string fakeResponse =
                        $"{AnsiBottomStatusLine}{AnsiReverseVideo}" +
                        " E37: No write since last change (add ! to override) " +
                        AnsiReset;
                    await writeCallback(Encoding.UTF8.GetBytes(fakeResponse));
                }
            }

            // 停滯結束，恢復游標並退出備用螢幕
            await writeCallback(Encoding.UTF8.GetBytes(AnsiExitSequence));

            _logger.LogInformation("Vi-Swap hold released");
        }

        /// <inheritdoc />
        public void Dispose()
        {
            _disposed = true;
        }
    }
}
