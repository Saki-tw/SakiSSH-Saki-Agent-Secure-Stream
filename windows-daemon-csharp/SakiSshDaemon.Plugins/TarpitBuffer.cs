// =============================================================================
// SakiSshDaemon.Plugins — TarpitBuffer.cs
// SASS Plugin #3: Zero-Allocation Tarpit Static Buffer
//
// 對應 Rust: tarpit.rs (TarpitGenerator::engulf)
// RFC 參考: draft-sakistudio-sass-00 Appendix C.3 (anchor: tarpit-buffer)
//
// 串流參數:
// - 總負載: 40 MiB
// - Chunk 大小: 64 KiB
// - Chunk 間隔: 500 ms
// - 總 chunks: 640
// - 總時長: ~320 秒
// - 並行門控: AtomicI32 (Interlocked), max 32
//
// Windows 差異:
// - 使用 ArrayPool<byte>.Shared.Rent(65536) 實現零分配
// - AtomicI32 → Interlocked.Increment/Decrement
// - 3 秒 send timeout 使用 CancellationTokenSource
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;
using System.Buffers;
using System.Security.Cryptography;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Logging;

namespace SakiSshDaemon.Plugins
{
    /// <summary>
    /// SASS Plugin #3: Zero-Allocation Tarpit Static Buffer。
    /// <para>
    /// RFC draft-sakistudio-sass-00 §C.3: 全域共用 64 KiB 靜態高熵 Buffer，
    /// 所有連線共享，空間開銷為 O(1)。
    /// 以慢速回傳 40 MiB 高熵垃圾資料，強制反向耗竭 Agent 的
    /// Token Context、V8 記憶體與 API 預算。
    /// </para>
    /// </summary>
    public sealed class TarpitBuffer : IPlugin
    {
        /// <summary>全域並行 Tarpit 連線計數器 — 對齊 Rust ACTIVE_TARPIT_COUNT</summary>
        private static int _activeTarpitCount;

        /// <summary>最大並行 Tarpit 數 — 對齊 Rust MAX_CONCURRENT_TARPIT = 32</summary>
        private const int MaxConcurrentTarpit = 32;

        /// <summary>每次 chunk 寫入的 send timeout — 對齊 Rust SEND_TIMEOUT_SECS = 3</summary>
        private const int SendTimeoutSeconds = 3;

        /// <summary>
        /// 全域靜態 64 KiB 高熵垃圾 Buffer。
        /// <para>
        /// 對齊 Rust: static STATIC_ENTROPY: OnceLock&lt;Vec&lt;u8&gt;&gt;
        /// 使用 Lazy&lt;T&gt; 實現執行緒安全的延遲初始化。
        /// </para>
        /// </summary>
        private static readonly Lazy<byte[]> StaticEntropy = new Lazy<byte[]>(() =>
        {
            byte[] data = new byte[64 * 1024]; // 64 KiB
            RandomNumberGenerator.Fill(data);
            return data;
        }, LazyThreadSafetyMode.ExecutionAndPublication);

        private readonly ILogger<TarpitBuffer> _logger;
        private bool _disposed;

        public TarpitBuffer(ILogger<TarpitBuffer> logger)
        {
            _logger = logger ?? throw new ArgumentNullException(nameof(logger));
        }

        /// <inheritdoc />
        public string Name => "Zero-Allocation Tarpit Buffer";

        /// <inheritdoc />
        public string RfcSection => "Appendix C.3 (tarpit-buffer)";

        /// <inheritdoc />
        public bool IsHealthy => !_disposed;

        /// <summary>目前啟用中的 Tarpit 連線數</summary>
        public static int ActiveCount => Volatile.Read(ref _activeTarpitCount);

        /// <inheritdoc />
        public Task<bool> InitializeAsync(CancellationToken cancellationToken = default)
        {
            // 觸發 Lazy 初始化，確保高熵 buffer 在啟動時就準備好
            _ = StaticEntropy.Value;

            _logger.LogInformation(
                "Plugin #3 ({Name}) 初始化完成 — 64 KiB 靜態高熵 Buffer 已就緒",
                Name);
            return Task.FromResult(true);
        }

        /// <summary>
        /// 將指定的 Session 拖入 Zero-Allocation 焦油坑。
        /// <para>
        /// 對齊 Rust: TarpitGenerator::engulf()
        /// 從全域靜態 64 KiB buffer 循環切片發送，總量 40 MiB。
        /// </para>
        /// </summary>
        /// <param name="writeCallback">
        /// 寫入回呼：每次被呼叫時傳入一個 chunk 的 byte[]。
        /// 此回呼對應 Rust 中的 session.stdout_ring.lock().write(buffer)。
        /// </param>
        /// <param name="config">可選的 Tarpit 配置，null 時使用預設值</param>
        /// <param name="cancellationToken">取消令牌</param>
        public async Task EngulfAsync(
            Func<byte[], int, Task> writeCallback,
            TarpitConfig? config = null,
            CancellationToken cancellationToken = default)
        {
            config ??= TarpitConfig.Default;

            // 檢查並行門控 — 對齊 Rust ACTIVE_TARPIT_COUNT.load + fetch_add
            int current = Volatile.Read(ref _activeTarpitCount);
            if (current >= MaxConcurrentTarpit)
            {
                _logger.LogError(
                    "並行 Tarpit 門檻已超過。丟棄連線。(Current={Current}, Max={Max})",
                    current, MaxConcurrentTarpit);
                return;
            }

            // 搶佔名額 — 對齊 Rust fetch_add(1, SeqCst)
            Interlocked.Increment(ref _activeTarpitCount);

            try
            {
                int totalChunks = config.TotalBytes / config.ChunkSize;
                byte[] buffer = StaticEntropy.Value;

                _logger.LogWarning(
                    "Tarpit Active Defense Engaged — {Chunks} chunks × {ChunkSize}B = {TotalBytes}B total",
                    totalChunks, config.ChunkSize, config.TotalBytes);

                for (int chunkIdx = 0; chunkIdx < totalChunks; chunkIdx++)
                {
                    if (cancellationToken.IsCancellationRequested) break;

                    // 3 秒 send timeout — 對齊 Rust timeout(SEND_TIMEOUT_SECS)
                    using var timeoutCts = CancellationTokenSource.CreateLinkedTokenSource(
                        cancellationToken);
                    timeoutCts.CancelAfter(TimeSpan.FromSeconds(SendTimeoutSeconds));

                    try
                    {
                        // 使用 ArrayPool 租借 buffer，實現零分配寫入
                        byte[] chunk = ArrayPool<byte>.Shared.Rent(config.ChunkSize);
                        try
                        {
                            // 從靜態高熵 buffer 複製（循環切片）
                            Array.Copy(buffer, 0, chunk, 0,
                                Math.Min(buffer.Length, config.ChunkSize));

                            await writeCallback(chunk, config.ChunkSize)
                                .WaitAsync(timeoutCts.Token);
                        }
                        finally
                        {
                            ArrayPool<byte>.Shared.Return(chunk);
                        }
                    }
                    catch (OperationCanceledException) when (timeoutCts.IsCancellationRequested
                        && !cancellationToken.IsCancellationRequested)
                    {
                        // Send timeout — 對齊 Rust "TCP Zero-Window suspected, skipping"
                        _logger.LogWarning(
                            "Tarpit send timeout at chunk {ChunkIdx}/{TotalChunks} — " +
                            "TCP Zero-Window suspected, skipping",
                            chunkIdx, totalChunks);
                        continue; // 跳過此 chunk，繼續下一個（不終止整個 tarpit）
                    }

                    // 刻意延遲 — 對齊 Rust sleep(Duration::from_millis(config.delay_ms))
                    await Task.Delay(
                        TimeSpan.FromMilliseconds(config.DelayMs),
                        cancellationToken);
                }

                _logger.LogWarning("Tarpit stream finished");
            }
            finally
            {
                // 釋放名額 — 對齊 Rust fetch_sub(1, SeqCst)
                Interlocked.Decrement(ref _activeTarpitCount);
            }
        }

        /// <inheritdoc />
        public void Dispose()
        {
            _disposed = true;
        }
    }

    /// <summary>
    /// 焦油坑產生器設定。
    /// <para>對齊 Rust: TarpitConfig struct</para>
    /// </summary>
    public sealed class TarpitConfig
    {
        /// <summary>總計要倒給攻擊者的垃圾資料量（預設 40 MiB）</summary>
        public int TotalBytes { get; init; } = 40 * 1024 * 1024;

        /// <summary>每個 Chunk 的大小（預設 64 KiB）</summary>
        public int ChunkSize { get; init; } = 64 * 1024;

        /// <summary>每次 Chunk 發送後的延遲毫秒（預設 500ms）</summary>
        public int DelayMs { get; init; } = 500;

        /// <summary>預設配置 — 對齊 Rust TarpitConfig::default()</summary>
        public static TarpitConfig Default => new TarpitConfig();
    }
}
