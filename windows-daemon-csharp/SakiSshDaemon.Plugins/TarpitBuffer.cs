// =============================================================================
// SakiSshDaemon.Plugins — TarpitBuffer.cs
// SASS Plugin #3: Zero-Allocation Tarpit Static Buffer with Pseudo-ICMP
//
// 對應 Rust: tarpit_payload.rs (SakiTarpitGenerator)
// 對應 Go:   tarpit_payload.go (SakiTarpitGenerator)
// RFC 參考: draft-sakistudio-sass-05 Appendix C.3 (anchor: tarpit-buffer)
//           + C.3.1 Pseudo-ICMP Payload Generation (anchor: tarpit-icmp-gen)
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
// - Payload 使用 RandomNumberGenerator（非 ChaCha20）產生高熵內容
//   但封包結構對齊 Rust/Go 的 pseudo-ICMP 格式
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;
using System.Buffers;
using System.Security.Cryptography;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Logging;

namespace SakiSshDaemon.Plugins
{
    /// <summary>
    /// Pseudo-ICMP 封包標頭（8 bytes）。
    /// <para>對齊 Rust: IcmpHeader / Go: ICMPHeader</para>
    /// </summary>
    internal readonly struct IcmpHeader
    {
        /// <summary>ICMP 類型：{0=Echo Reply, 3=Dest Unreachable, 8=Echo Request, 11=Time Exceeded, 13=Timestamp, 14=Timestamp Reply}</summary>
        public byte Type { get; }

        /// <summary>ICMP 代碼（counter rotate-left 3, masked to 4 bits）</summary>
        public byte Code { get; }

        /// <summary>ICMP 識別碼（counter low 16 bits）</summary>
        public ushort Identifier { get; }

        /// <summary>ICMP 序列號（wrapping u16 counter）</summary>
        public ushort SeqNumber { get; }

        public IcmpHeader(byte type, byte code, ushort identifier, ushort seqNumber)
        {
            Type = type;
            Code = code;
            Identifier = identifier;
            SeqNumber = seqNumber;
        }

        /// <summary>
        /// 序列化為 8 bytes — 對齊 Rust IcmpHeader::to_bytes()
        /// Checksum 欄位（bytes [2:4]）初始為 0，待外部計算後回填。
        /// </summary>
        public void WriteTo(Span<byte> buffer)
        {
            buffer[0] = Type;
            buffer[1] = Code;
            // Checksum at [2:4] — 先設 0
            buffer[2] = 0;
            buffer[3] = 0;
            // Identifier (big-endian)
            buffer[4] = (byte)(Identifier >> 8);
            buffer[5] = (byte)(Identifier & 0xFF);
            // SeqNumber (big-endian)
            buffer[6] = (byte)(SeqNumber >> 8);
            buffer[7] = (byte)(SeqNumber & 0xFF);
        }
    }

    /// <summary>
    /// Pseudo-ICMP 焦油坑負載產生器。
    /// <para>
    /// 對齊 Rust: SakiTarpitGenerator / Go: SakiTarpitGenerator
    /// 產生結構化的偽 ICMP 封包，讓 Agent 誤以為是網路流量。
    /// C# 版使用 RandomNumberGenerator 產生 payload（非 ChaCha20），
    /// 但封包結構（header + checksum + 簽名）完全對齊 Rust/Go。
    /// </para>
    /// </summary>
    internal sealed class PseudoIcmpGenerator
    {
        /// <summary>可選 ICMP 類型 — 對齊 Rust/Go icmpTypes</summary>
        private static readonly byte[] IcmpTypes = { 0, 3, 8, 11, 13, 14 };

        /// <summary>Saki✰ 簽名（UTF-8）— 對齊 Rust SAKI_SIGNATURE / Go SakiSignature</summary>
        private static readonly byte[] SakiSignature = Encoding.UTF8.GetBytes("Saki✰");

        /// <summary>ICMP header 大小（bytes）</summary>
        private const int HeaderSize = 8;

        private ulong _counter;
        private ushort _seqNum;

        /// <summary>
        /// 計算 RFC 1071 校驗和 — 對齊 Rust compute_icmp_checksum / Go ComputeICMPChecksum
        /// </summary>
        public static ushort ComputeIcmpChecksum(ReadOnlySpan<byte> data)
        {
            uint sum = 0;
            int i = 0;
            while (i + 1 < data.Length)
            {
                sum += (uint)((data[i] << 8) | data[i + 1]);
                i += 2;
            }
            if (data.Length % 2 == 1)
            {
                sum += (uint)(data[data.Length - 1] << 8);
            }
            while ((sum >> 16) != 0)
            {
                sum = (sum & 0xFFFF) + (sum >> 16);
            }
            return (ushort)~sum;
        }

        /// <summary>
        /// 產生單一偽 ICMP 封包（header + 高熵 payload）。
        /// <para>對齊 Rust: generate_icmp_packet / Go: generateMixedICMPTypes</para>
        /// </summary>
        /// <param name="payloadSize">payload 大小（不含 8-byte header）</param>
        /// <param name="buffer">目標 buffer（必須 ≥ headerSize + payloadSize）</param>
        /// <returns>實際寫入的 bytes 數</returns>
        public int GenerateIcmpPacket(int payloadSize, Span<byte> buffer)
        {
            _counter++;
            _seqNum = unchecked((ushort)(_seqNum + 1));

            int totalSize = HeaderSize + payloadSize;
            if (buffer.Length < totalSize)
                totalSize = buffer.Length;

            int actualPayload = totalSize - HeaderSize;
            if (actualPayload < 0) return 0;

            // 根據 counter 選擇 ICMP 類型 — 對齊 Rust/Go
            byte icmpType = IcmpTypes[_counter % (ulong)IcmpTypes.Length];
            // Code: rotate-left 3, masked to 4 bits — 對齊 Rust/Go
            byte code = (byte)(((_counter << 3) | (_counter >> (64 - 3))) & 0x0F);

            var header = new IcmpHeader(
                icmpType,
                code,
                (ushort)(_counter & 0xFFFF),
                _seqNum);

            header.WriteTo(buffer);

            // 使用 RandomNumberGenerator 產生高熵 payload
            // （C# 版差異：非 ChaCha20，但同等高熵）
            if (actualPayload > 0)
            {
                RandomNumberGenerator.Fill(buffer.Slice(HeaderSize, actualPayload));
            }

            // 計算 RFC 1071 checksum — 對齊 Rust/Go
            ushort checksum = ComputeIcmpChecksum(buffer.Slice(0, totalSize));
            buffer[2] = (byte)(checksum >> 8);
            buffer[3] = (byte)(checksum & 0xFF);

            return totalSize;
        }

        /// <summary>
        /// 產生一個焦油坑 chunk，由多個偽 ICMP 封包串接。
        /// <para>
        /// 結構：[ICMP Packet 1][ICMP Packet 2]...[ICMP Packet N][Saki✰]
        /// 對齊 Rust: generate_chunk / Go: GenerateChunk
        /// </para>
        /// </summary>
        /// <param name="buffer">目標 buffer（chunk 大小由 buffer.Length 決定）</param>
        /// <param name="isFinal">是否為最後一個 chunk（附帶 Saki✰ 簽名）</param>
        /// <returns>實際寫入的 bytes 數</returns>
        public int GenerateChunk(Span<byte> buffer, bool isFinal)
        {
            int chunkSize = buffer.Length;
            int written = 0;

            while (written < chunkSize)
            {
                // payload 大小在 56~248 之間（193 是質數，避免週期）
                // — 對齊 Rust/Go
                int payloadVariation = 56 + (int)((_counter + 1) % 193);
                int remaining = chunkSize - written;

                if (isFinal && remaining <= SakiSignature.Length + HeaderSize)
                    break;

                int packetSize = HeaderSize + payloadVariation;
                if (packetSize > remaining)
                {
                    packetSize = remaining;
                    if (packetSize < HeaderSize)
                        break;
                }

                int bytesWritten = GenerateIcmpPacket(
                    packetSize - HeaderSize,
                    buffer.Slice(written, packetSize));
                written += bytesWritten;
            }

            // 最後一個 chunk 附帶 Saki✰ 簽名 — 對齊 Rust/Go
            if (isFinal)
            {
                // 零填充到簽名位置
                while (written < chunkSize - SakiSignature.Length)
                {
                    buffer[written++] = 0x00;
                }
                // 寫入 Saki✰ 簽名
                int sigLen = Math.Min(SakiSignature.Length, chunkSize - written);
                SakiSignature.AsSpan(0, sigLen).CopyTo(buffer.Slice(written));
                written += sigLen;
            }

            return Math.Min(written, chunkSize);
        }
    }

    /// <summary>
    /// SASS Plugin #3: Zero-Allocation Tarpit Static Buffer with Pseudo-ICMP。
    /// <para>
    /// RFC draft-sakistudio-sass-05 §C.3 + §C.3.1: 全域共用 64 KiB 靜態高熵 Buffer，
    /// 所有連線共享，空間開銷為 O(1)。
    /// 以慢速回傳 40 MiB 偽 ICMP 封包結構的垃圾資料，強制反向耗竭 Agent 的
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
        /// 全域靜態 64 KiB pseudo-ICMP Buffer。
        /// <para>
        /// 對齊 Rust: static STATIC_ENTROPY: OnceLock&lt;Vec&lt;u8&gt;&gt;
        /// 使用 Lazy&lt;T&gt; 實現執行緒安全的延遲初始化。
        /// 現在使用 PseudoIcmpGenerator 產生結構化的 ICMP 封包（非純隨機）。
        /// </para>
        /// </summary>
        private static readonly Lazy<byte[]> StaticEntropy = new Lazy<byte[]>(() =>
        {
            byte[] data = new byte[64 * 1024]; // 64 KiB
            // 使用 PseudoIcmpGenerator 產生結構化 pseudo-ICMP 封包
            var generator = new PseudoIcmpGenerator();
            generator.GenerateChunk(data.AsSpan(), isFinal: false);
            return data;
        }, LazyThreadSafetyMode.ExecutionAndPublication);

        private readonly ILogger<TarpitBuffer> _logger;
        private bool _disposed;

        public TarpitBuffer(ILogger<TarpitBuffer> logger)
        {
            _logger = logger ?? throw new ArgumentNullException(nameof(logger));
        }

        /// <inheritdoc />
        public string Name => "Zero-Allocation Tarpit Buffer (Pseudo-ICMP)";

        /// <inheritdoc />
        public string RfcSection => "Appendix C.3 + C.3.1 (tarpit-buffer + tarpit-icmp-gen)";

        /// <inheritdoc />
        public bool IsHealthy => !_disposed;

        /// <summary>目前啟用中的 Tarpit 連線數</summary>
        public static int ActiveCount => Volatile.Read(ref _activeTarpitCount);

        /// <inheritdoc />
        public Task<bool> InitializeAsync(CancellationToken cancellationToken = default)
        {
            // 觸發 Lazy 初始化，確保 pseudo-ICMP buffer 在啟動時就準備好
            _ = StaticEntropy.Value;

            _logger.LogInformation(
                "Plugin #3 ({Name}) 初始化完成 — 64 KiB pseudo-ICMP 靜態 Buffer 已就緒",
                Name);
            return Task.FromResult(true);
        }

        /// <summary>
        /// 將指定的 Session 拖入 Zero-Allocation 焦油坑。
        /// <para>
        /// 對齊 Rust: TarpitGenerator::engulf()
        /// 從全域靜態 64 KiB pseudo-ICMP buffer 循環切片發送，總量 40 MiB。
        /// 每個 chunk 包含結構化的偽 ICMP 封包（header + checksum + payload）。
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
                    "Tarpit Active Defense Engaged (Pseudo-ICMP) — {Chunks} chunks × {ChunkSize}B = {TotalBytes}B total",
                    totalChunks, config.ChunkSize, config.TotalBytes);

                // 為動態 chunk 建立 per-session 的 ICMP 產生器
                var sessionGenerator = new PseudoIcmpGenerator();

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
                            bool isFinal = (chunkIdx == totalChunks - 1);

                            // 產生結構化 pseudo-ICMP chunk
                            // 交替使用靜態 buffer（偶數 chunk）和動態產生（奇數 chunk）
                            if (chunkIdx % 2 == 0)
                            {
                                // 偶數 chunk：從靜態 buffer 複製（效能最佳）
                                Array.Copy(buffer, 0, chunk, 0,
                                    Math.Min(buffer.Length, config.ChunkSize));
                            }
                            else
                            {
                                // 奇數 chunk：動態產生新的 pseudo-ICMP 封包（避免 pattern）
                                sessionGenerator.GenerateChunk(
                                    chunk.AsSpan(0, config.ChunkSize), isFinal);
                            }

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

                _logger.LogWarning("Tarpit stream finished (Pseudo-ICMP mode)");
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
