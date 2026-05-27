// =============================================================================
// SakiSshDaemon.Plugins — TlsExporterBinding.cs
// SASS Plugin #2: TLS Exporter Binding for Cognitive Challenge
//
// 對應 Rust: threat_defense.rs (TlsExporterProvider / derive_ekm / verify_ekm_hmac)
// 對應 Go:   defense/tls_exporter.go
// RFC 參考:
//   - RFC 5705: Keying Material Exporters for TLS
//   - RFC 9266: Channel Bindings for TLS 1.3
//   - RFC 8446 §7.5: Exporters
//   - draft-sakistudio-sass-00 Appendix C.2 (anchor: tls-exporter-binding)
//
// TLS Exporter Label: "EXPORTER-sakissh-chacha20-v14"
// Context: Session UUID (16 bytes)
// Length: 44 bytes (32-byte ChaCha20 key + 12-byte nonce)
//
// Windows/.NET 8 差異:
// - .NET 8 的 SslStream 不提供 ExportKeyingMaterial() API
// - SChannel (Windows TLS) 底層支援 SSL_export_keying_material，
//   但 .NET BCL 未封裝此功能 (截至 .NET 8.0)
// - 需要透過 P/Invoke 呼叫 OpenSSL 的 SSL_export_keying_material，
//   或等待 .NET 9+ 原生支援
// - 目前使用 HMAC-SHA256 stub 作為 fallback，與 Rust/Go 降級行為一致
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;
using System.Security.Cryptography;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Logging;

namespace SakiSshDaemon.Plugins
{
    /// <summary>
    /// TLS Exported Keying Material 結構。
    /// <para>封裝從 TLS session 匯出的 44 bytes 密鑰材料。</para>
    /// <para>對齊 Rust: ExportedKeyingMaterial struct</para>
    /// <para>對齊 Go: defense.ExportedKeyingMaterial</para>
    /// </summary>
    public sealed class ExportedKeyingMaterial
    {
        /// <summary>完整 44 bytes EKM 原始資料</summary>
        public byte[] Raw { get; }

        /// <summary>前 32 bytes: ChaCha20-Poly1305 加密金鑰</summary>
        public byte[] ChaChaKey { get; }

        /// <summary>後 12 bytes: ChaCha20-Poly1305 nonce</summary>
        public byte[] ChaChaNonce { get; }

        /// <summary>EKM 來源標記：true = 真實 TLS EKM，false = HMAC fallback stub</summary>
        public bool IsRealEkm { get; }

        public ExportedKeyingMaterial(byte[] raw, bool isRealEkm = false)
        {
            if (raw.Length != TlsExporterBinding.EkmLength)
                throw new ArgumentException(
                    $"EKM 必須為 {TlsExporterBinding.EkmLength} bytes，實際為 {raw.Length}",
                    nameof(raw));

            Raw = raw;
            ChaChaKey = new byte[32];
            ChaChaNonce = new byte[12];
            Array.Copy(raw, 0, ChaChaKey, 0, 32);
            Array.Copy(raw, 32, ChaChaNonce, 0, 12);
            IsRealEkm = isRealEkm;
        }
    }

    // =========================================================================
    // TLS Exporter Provider 介面 (RFC 5705 §2 / RFC 9266 §3)
    // =========================================================================

    /// <summary>
    /// TLS Exporter 提供者介面。
    /// <para>封裝 TLS session 的 EKM 匯出能力，允許不同的 TLS 後端實作。</para>
    /// <para>對齊 Rust: TlsExporterProvider trait</para>
    /// <para>對齊 Go: TLSExporterProvider interface</para>
    /// </summary>
    public interface ITlsExporterProvider
    {
        /// <summary>
        /// 從 TLS session 匯出 Keying Material。
        /// </summary>
        /// <param name="label">RFC 5705 exporter label</param>
        /// <param name="context">應用層上下文 (Session UUID)</param>
        /// <param name="length">要匯出的位元組數</param>
        /// <returns>匯出的位元組</returns>
        /// <exception cref="NotSupportedException">當底層 TLS 不支援 EKM 匯出時</exception>
        byte[] ExportKeyingMaterial(string label, byte[]? context, int length);
    }

    // =========================================================================
    // .NET SslStream TLS Exporter Provider (目前不支援)
    // =========================================================================

    /// <summary>
    /// .NET SslStream TLS Exporter 提供者。
    /// <para>
    /// ⚠️ .NET 8.0 的 SslStream 不提供 ExportKeyingMaterial() API。
    /// </para>
    /// <para>
    /// 技術分析 (Why .NET 8 不支援):
    /// </para>
    /// <list type="bullet">
    /// <item>
    /// <description>
    /// Windows SChannel: 底層透過 <c>NCryptExportKey</c> / <c>SslExportKeyingMaterial</c>
    /// 支援 RFC 5705，但 .NET BCL 的 <c>SslStream</c> 未封裝此功能。
    /// </description>
    /// </item>
    /// <item>
    /// <description>
    /// OpenSSL (Linux .NET): <c>SSL_export_keying_material()</c> 在 OpenSSL 1.0.1+
    /// 中可用，但 .NET 的 OpenSSL interop 層未暴露此 API。
    /// </description>
    /// </item>
    /// <item>
    /// <description>
    /// P/Invoke 路徑: 理論上可透過 <c>[DllImport("libssl")]</c> 直接呼叫
    /// <c>SSL_export_keying_material()</c>，但需要取得 <c>SslStream</c> 底層的
    /// <c>SSL*</c> 指標，而 .NET 將其封裝為 internal。
    /// </description>
    /// </item>
    /// <item>
    /// <description>
    /// .NET 9 提案: dotnet/runtime#97485 追蹤此功能，預計在 .NET 9 或更晚版本支援。
    /// </description>
    /// </item>
    /// </list>
    /// </summary>
    public sealed class SslStreamExporterProvider : ITlsExporterProvider
    {
        /// <summary>
        /// 嘗試從 SslStream 匯出 Keying Material。
        /// <para>⚠️ 始終拋出 <see cref="NotSupportedException"/>。</para>
        /// </summary>
        /// <exception cref="NotSupportedException">
        /// .NET 8.0 SslStream 不支援 RFC 5705 ExportKeyingMaterial API。
        /// 等待 .NET 9+ (dotnet/runtime#97485) 或使用 HMAC fallback。
        /// </exception>
        public byte[] ExportKeyingMaterial(string label, byte[]? context, int length)
        {
            // .NET 8.0 不支援 — 明確拋出帶有技術原因的 NotSupportedException
            // 未來實作路線:
            // 1. .NET 9+: 使用 SslStream.ExportKeyingMaterial() (若 API 加入)
            // 2. P/Invoke: 透過 reflection 取得 SslStream._securityContext → SafeHandle
            //    → NativeSSL → SSL_export_keying_material()
            // 3. OpenSSL interop: 在 Linux 上直接 P/Invoke libssl.so
            throw new NotSupportedException(
                ".NET 8.0 SslStream 不支援 RFC 5705 ExportKeyingMaterial() API。" +
                "Windows SChannel 底層支援 SslExportKeyingMaterial，但 .NET BCL 未封裝此功能。" +
                "請使用 HmacFallbackProvider 作為降級方案，或等待 .NET 9+ (dotnet/runtime#97485)。" +
                "HMAC fallback 已自動啟用，與 Rust/Go 降級行為一致。");
        }
    }

    // =========================================================================
    // HMAC Fallback Provider — 降級方案
    // =========================================================================

    /// <summary>
    /// HMAC Fallback 提供者 — 當無法取得 TLS 連線時使用。
    /// <para>
    /// 使用 HMAC-SHA256(session_uuid, label) 推導密鑰材料。
    /// 與 Rust HmacFallbackProvider / Go HmacFallbackProvider 行為完全一致。
    /// </para>
    /// <para>⚠️ 此模式不具備真正的 TLS 通道綁定安全性。</para>
    /// </summary>
    public sealed class HmacFallbackProvider : ITlsExporterProvider
    {
        private readonly byte[] _sessionUuid;

        public HmacFallbackProvider(byte[] sessionUuid)
        {
            if (sessionUuid.Length != TlsExporterBinding.SessionUuidLength)
                throw new ArgumentException(
                    $"Session UUID 必須為 {TlsExporterBinding.SessionUuidLength} bytes",
                    nameof(sessionUuid));
            _sessionUuid = sessionUuid;
        }

        /// <summary>
        /// HMAC-SHA256 模擬 EKM 推導 — 對齊 Rust/Go 的 HMAC fallback 行為。
        /// </summary>
        public byte[] ExportKeyingMaterial(string label, byte[]? context, int length)
        {
            // 步驟 1: HMAC-SHA256(session_uuid, label) → 32 bytes key
            byte[] keyMaterial;
            using (var hmac = new HMACSHA256(_sessionUuid))
            {
                keyMaterial = hmac.ComputeHash(
                    System.Text.Encoding.UTF8.GetBytes(label));
            }

            // 步驟 2: HMAC-SHA256(key_material, "nonce-derivation") → 取前 12 bytes 作為 nonce
            byte[] nonceSource;
            using (var hmac = new HMACSHA256(keyMaterial))
            {
                nonceSource = hmac.ComputeHash(
                    System.Text.Encoding.UTF8.GetBytes("nonce-derivation"));
            }

            // 組合至指定長度
            byte[] result = new byte[length];
            int keyLen = Math.Min(32, length);
            Array.Copy(keyMaterial, 0, result, 0, keyLen);
            if (length > 32)
            {
                int nonceLen = Math.Min(length - 32, nonceSource.Length);
                Array.Copy(nonceSource, 0, result, 32, nonceLen);
            }

            return result;
        }
    }

    // =========================================================================
    // SASS Plugin #2: TLS Exporter Binding
    // =========================================================================

    /// <summary>
    /// SASS Plugin #2: TLS Exporter Binding。
    /// <para>
    /// RFC draft-sakistudio-sass-00 §C.2: 從 TLS session 推導 Keying Material，
    /// 用於通道綁定 (Channel Binding)。確保認知挑戰在同一 TLS session 中完成。
    /// </para>
    /// <para>
    /// v1.4 升級: 新增 ITlsExporterProvider 介面支援，
    /// 當 .NET 支援 ExportKeyingMaterial 時可無縫切換至真實 TLS EKM。
    /// 目前在 .NET 8 上自動使用 HMAC fallback。
    /// </para>
    /// </summary>
    public sealed class TlsExporterBinding : IPlugin
    {
        /// <summary>
        /// TLS Exporter Label — RFC 5705 §4 格式。
        /// <para>對齊 Rust: TLS_EXPORTER_LABEL 常量</para>
        /// <para>對齊 Go: TLSExporterLabel 常量</para>
        /// </summary>
        public const string ExporterLabel = "EXPORTER-sakissh-chacha20-v14";

        /// <summary>
        /// TLS Exporter 輸出長度 (44 bytes = 32 key + 12 nonce)。
        /// <para>對齊 Rust: TLS_EXPORTER_LENGTH = 44</para>
        /// <para>對齊 Go: TLSExporterLength = 44</para>
        /// </summary>
        public const int EkmLength = 44;

        /// <summary>Session UUID 長度 (16 bytes)</summary>
        public const int SessionUuidLength = 16;

        private readonly ILogger<TlsExporterBinding> _logger;
        private bool _disposed;

        public TlsExporterBinding(ILogger<TlsExporterBinding> logger)
        {
            _logger = logger ?? throw new ArgumentNullException(nameof(logger));
        }

        /// <inheritdoc />
        public string Name => "TLS Exporter Binding";

        /// <inheritdoc />
        public string RfcSection => "Appendix C.2 (tls-exporter-binding)";

        /// <inheritdoc />
        public bool IsHealthy => !_disposed;

        /// <inheritdoc />
        public Task<bool> InitializeAsync(CancellationToken cancellationToken = default)
        {
            _logger.LogInformation(
                "Plugin #2 ({Name}) 初始化完成 — Label={Label}, Length={Length}, " +
                "模式=HMAC Fallback (.NET 8 不支援真實 TLS EKM)",
                Name, ExporterLabel, EkmLength);
            return Task.FromResult(true);
        }

        // =====================================================================
        // EKM 推導 — 透過 Provider 介面
        // =====================================================================

        /// <summary>
        /// 從 ITlsExporterProvider 推導 EKM (v1.4 推薦入口)。
        /// <para>對齊 Rust: derive_ekm()</para>
        /// <para>對齊 Go: DeriveEKM()</para>
        /// </summary>
        /// <param name="provider">TLS Exporter 提供者</param>
        /// <param name="sessionUuid">16-byte session UUID 作為 EKM context</param>
        /// <returns>44 bytes 的 ExportedKeyingMaterial</returns>
        public ExportedKeyingMaterial DeriveEkm(ITlsExporterProvider provider, byte[] sessionUuid)
        {
            if (sessionUuid.Length != SessionUuidLength)
                throw new ArgumentException(
                    $"Session UUID 必須為 {SessionUuidLength} bytes",
                    nameof(sessionUuid));

            try
            {
                byte[] raw = provider.ExportKeyingMaterial(
                    ExporterLabel, sessionUuid, EkmLength);

                bool isReal = provider is not HmacFallbackProvider;
                _logger.LogInformation(
                    "TLS EKM derived ({Mode}), label={Label}",
                    isReal ? "real TLS exporter" : "HMAC fallback",
                    ExporterLabel);

                return new ExportedKeyingMaterial(raw, isReal);
            }
            catch (NotSupportedException ex)
            {
                // .NET 8 SslStream 不支援 → 自動降級
                _logger.LogWarning(
                    "TLS EKM 匯出不支援 ({Reason}), 降級為 HMAC fallback",
                    ex.Message);
                return DeriveEkmFallback(sessionUuid);
            }
            catch (Exception ex)
            {
                _logger.LogWarning(
                    "TLS EKM 匯出失敗 ({Error}), 降級為 HMAC fallback",
                    ex.Message);
                return DeriveEkmFallback(sessionUuid);
            }
        }

        /// <summary>
        /// HMAC Fallback EKM 推導。
        /// <para>對齊 Rust: derive_ekm_fallback()</para>
        /// <para>對齊 Go: DeriveEKMFallback()</para>
        /// <para>當 .NET SslStream 不支援 ExportKeyingMaterial 時自動使用。</para>
        /// </summary>
        public ExportedKeyingMaterial DeriveEkmFallback(byte[] sessionUuid)
        {
            if (sessionUuid.Length != SessionUuidLength)
                throw new ArgumentException(
                    $"Session UUID 必須為 {SessionUuidLength} bytes",
                    nameof(sessionUuid));

            var provider = new HmacFallbackProvider(sessionUuid);
            byte[] raw = provider.ExportKeyingMaterial(
                ExporterLabel, sessionUuid, EkmLength);

            _logger.LogInformation(
                "TLS EKM fallback derived (HMAC-SHA256), label={Label}",
                ExporterLabel);

            return new ExportedKeyingMaterial(raw, isRealEkm: false);
        }

        /// <summary>
        /// 從 TLS session 推導 Exported Keying Material (EKM)。
        /// <para>向後相容入口 — 使用 HMAC fallback。</para>
        /// <para>⚠️ 建議遷移至 DeriveEkm(ITlsExporterProvider, byte[])</para>
        /// </summary>
        [Obsolete("請使用 DeriveEkm(ITlsExporterProvider, byte[]) 搭配 provider 介面")]
        public ExportedKeyingMaterial DeriveEkm(byte[] sessionUuid)
        {
            return DeriveEkmFallback(sessionUuid);
        }

        // =====================================================================
        // EKM HMAC 驗證
        // =====================================================================

        /// <summary>
        /// 驗證 Client 提供的 EKM HMAC 通道綁定。
        /// <para>
        /// 對齊 Rust: verify_ekm_hmac()
        /// 對齊 Go: VerifyEKMHmac()
        /// Client 以 HMAC-SHA256(ekm.raw, decrypted_plaintext) 計算 client_ekm_hmac，
        /// Daemon 端以相同方式驗證（constant-time）。
        /// </para>
        /// </summary>
        /// <param name="ekm">44 bytes EKM</param>
        /// <param name="decryptedPlaintext">解密後的明文</param>
        /// <param name="clientHmac">Client 提供的 HMAC</param>
        /// <returns>true 表示通道綁定一致</returns>
        public bool VerifyEkmHmac(
            ExportedKeyingMaterial ekm,
            byte[] decryptedPlaintext,
            byte[] clientHmac)
        {
            // 計算期望的 HMAC — 對齊 Rust verify_ekm_hmac
            byte[] expectedHmac;
            using (var hmac = new HMACSHA256(ekm.Raw))
            {
                expectedHmac = hmac.ComputeHash(decryptedPlaintext);
            }

            // constant-time 比對 — 對齊 Rust hmac crate 的 verify_slice
            bool result = CryptographicOperations.FixedTimeEquals(
                expectedHmac, clientHmac);

            if (result)
            {
                _logger.LogInformation("TLS EKM HMAC 通道綁定驗證成功");
            }
            else
            {
                _logger.LogWarning("TLS EKM HMAC 通道綁定驗證失敗");
            }

            return result;
        }

        /// <inheritdoc />
        public void Dispose()
        {
            _disposed = true;
        }
    }
}
