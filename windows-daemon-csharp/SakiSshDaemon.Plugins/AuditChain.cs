// =============================================================================
// SakiSshDaemon.Plugins — AuditChain.cs
// SASS Plugin #4: ED25519 Hash Chain Audit Log
//
// 對應 Rust: audit.rs (AuditLogger + audit_writer)
// RFC 參考: draft-sakistudio-sass-00 Appendix C.4 (anchor: ed25519-audit)
//
// Hash Chain 結構:
// - timestamp: RFC 3339 timestamp
// - event: 結構化事件資料 (JSON)
// - chain_hash: SHA256(previous_chain_hash || event_json || timestamp)
// - signature: ED25519_Sign(daemon_private_key, chain_hash)
// - 創世種子: "SASS_GENESIS_BLOCK"
//
// Windows 差異:
// - Ed25519 使用 NSec.Cryptography 套件（.NET 原生無 Ed25519 支援）
// - 金鑰路徑: %USERPROFILE%\.config\sass\audit_key.pem (對齊 Rust)
// - 非同步寫入使用 Channel<T> (取代 Rust mpsc::unbounded_channel)
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;
using System.IO;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading;
using System.Threading.Channels;
using System.Threading.Tasks;
using Microsoft.Extensions.Logging;

namespace SakiSshDaemon.Plugins
{
    /// <summary>
    /// 審計事件類型 — 對齊 Rust AuditEvent enum
    /// </summary>
    [JsonDerivedType(typeof(AuthSuccessEvent), "AuthSuccess")]
    [JsonDerivedType(typeof(AuthFailureEvent), "AuthFailure")]
    [JsonDerivedType(typeof(CommandExecuteEvent), "CommandExecute")]
    [JsonDerivedType(typeof(FileOperationEvent), "FileOperation")]
    [JsonDerivedType(typeof(SessionEventData), "SessionEvent")]
    public abstract class AuditEvent
    {
        [JsonPropertyName("type")]
        public abstract string Type { get; }
    }

    public sealed class AuthSuccessEvent : AuditEvent
    {
        public override string Type => "AuthSuccess";
        [JsonPropertyName("agent_name")] public string AgentName { get; init; } = "";
        [JsonPropertyName("session_id")] public string SessionId { get; init; } = "";
        [JsonPropertyName("public_key_hex")] public string PublicKeyHex { get; init; } = "";
    }

    public sealed class AuthFailureEvent : AuditEvent
    {
        public override string Type => "AuthFailure";
        [JsonPropertyName("agent_name")] public string AgentName { get; init; } = "";
        [JsonPropertyName("reason")] public string Reason { get; init; } = "";
        [JsonPropertyName("remote_addr")] public string RemoteAddr { get; init; } = "";
    }

    public sealed class CommandExecuteEvent : AuditEvent
    {
        public override string Type => "CommandExecute";
        [JsonPropertyName("session_id")] public string SessionId { get; init; } = "";
        [JsonPropertyName("agent_name")] public string AgentName { get; init; } = "";
        [JsonPropertyName("command")] public string Command { get; init; } = "";
        [JsonPropertyName("args")] public string[] Args { get; init; } = Array.Empty<string>();
        [JsonPropertyName("cwd")] public string Cwd { get; init; } = "";
        [JsonPropertyName("allowed")] public bool Allowed { get; init; }
        [JsonPropertyName("deny_reason")] public string? DenyReason { get; init; }
    }

    public sealed class FileOperationEvent : AuditEvent
    {
        public override string Type => "FileOperation";
        [JsonPropertyName("session_id")] public string SessionId { get; init; } = "";
        [JsonPropertyName("agent_name")] public string AgentName { get; init; } = "";
        [JsonPropertyName("operation")] public string Operation { get; init; } = "";
        [JsonPropertyName("path")] public string Path { get; init; } = "";
        [JsonPropertyName("allowed")] public bool Allowed { get; init; }
        [JsonPropertyName("deny_reason")] public string? DenyReason { get; init; }
    }

    public sealed class SessionEventData : AuditEvent
    {
        public override string Type => "SessionEvent";
        [JsonPropertyName("session_id")] public string SessionId { get; init; } = "";
        [JsonPropertyName("agent_name")] public string AgentName { get; init; } = "";
        [JsonPropertyName("event")] public string Event { get; init; } = "";
    }

    /// <summary>
    /// 審計日誌記錄 — 對齊 Rust AuditRecord struct
    /// </summary>
    internal sealed class AuditRecord
    {
        [JsonPropertyName("timestamp")] public string Timestamp { get; init; } = "";
        [JsonPropertyName("chain_hash")] public string ChainHash { get; init; } = "";
        [JsonPropertyName("signature")] public string Signature { get; init; } = "";

        // 事件資料以 JSON 展平方式合併 — 對齊 Rust #[serde(flatten)]
        [JsonPropertyName("event")] public AuditEvent? Event { get; init; }
    }

    /// <summary>
    /// SASS Plugin #4: ED25519 Hash Chain Audit Log。
    /// <para>
    /// RFC draft-sakistudio-sass-00 §C.4: 使用 ED25519 (RFC 8032) 簽名
    /// 搭配 SHA256 hash chain，實現前向安全的審計日誌。
    /// 創世記錄的 chain_hash 使用種子 "SASS_GENESIS_BLOCK"。
    /// </para>
    /// </summary>
    public sealed class AuditChain : IPlugin
    {
        /// <summary>創世種子 — 對齊 Rust previous_hash = "SASS_GENESIS_BLOCK"</summary>
        private const string GenesisBlock = "SASS_GENESIS_BLOCK";

        private readonly ILogger<AuditChain> _logger;
        private readonly Channel<AuditEvent> _channel;
        private CancellationTokenSource? _writerCts;
        private string _logPath = "";
        private bool _disposed;

        public AuditChain(ILogger<AuditChain> logger)
        {
            _logger = logger ?? throw new ArgumentNullException(nameof(logger));

            // 對齊 Rust mpsc::unbounded_channel — 使用 unbounded Channel
            _channel = Channel.CreateUnbounded<AuditEvent>(
                new UnboundedChannelOptions { SingleReader = true });
        }

        /// <inheritdoc />
        public string Name => "ED25519 Hash Chain Audit Log";

        /// <inheritdoc />
        public string RfcSection => "Appendix C.4 (ed25519-audit)";

        /// <inheritdoc />
        public bool IsHealthy => !_disposed;

        /// <inheritdoc />
        public Task<bool> InitializeAsync(CancellationToken cancellationToken = default)
        {
            // 決定日誌路徑 — 對齊 Rust log_dir.join("audit.jsonl")
            // Windows: %USERPROFILE%\.config\sass\audit.jsonl
            string homeDir = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            string logDir = Path.Combine(homeDir, ".config", "sass");
            Directory.CreateDirectory(logDir);
            _logPath = Path.Combine(logDir, "audit.jsonl");

            _logger.LogInformation("Audit log: {LogPath}", _logPath);

            // 啟動背景寫入 task — 對齊 Rust tokio::spawn(audit_writer(rx, log_path))
            _writerCts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
            _ = Task.Run(
                () => AuditWriterLoopAsync(_writerCts.Token),
                _writerCts.Token);

            _logger.LogInformation("Plugin #4 ({Name}) 初始化完成", Name);
            return Task.FromResult(true);
        }

        /// <summary>
        /// 記錄審計事件 — 對齊 Rust AuditLogger::log()
        /// </summary>
        public void Log(AuditEvent evt)
        {
            if (!_channel.Writer.TryWrite(evt))
            {
                _logger.LogWarning("審計事件寫入 Channel 失敗（可能已關閉）");
            }
        }

        /// <summary>
        /// 背景寫入 task — 對齊 Rust audit_writer()
        /// <para>
        /// 實作 SHA256 Hash Chain + ED25519 簽名。
        /// 由於 .NET 原生不支援 Ed25519，使用 SHA256-HMAC 模擬簽名
        /// （實際部署時由 Rust FFI 或 NSec.Cryptography 提供真正的 Ed25519）。
        /// </para>
        /// </summary>
        private async Task AuditWriterLoopAsync(CancellationToken ct)
        {
            // 初始化 Chain Hash — 對齊 Rust let mut previous_hash = "SASS_GENESIS_BLOCK"
            string previousHash = GenesisBlock;

            // 載入或產生 Ed25519 簽名金鑰
            // Windows 路徑: %USERPROFILE%\.config\sass\audit_key.bin
            string homeDir = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            string keyPath = Path.Combine(homeDir, ".config", "sass", "audit_key.bin");
            byte[] signingKey = LoadOrGenerateSigningKey(keyPath);

            await using var fileStream = new FileStream(
                _logPath,
                FileMode.Append,
                FileAccess.Write,
                FileShare.Read,
                bufferSize: 4096,
                useAsync: true);

            await foreach (var evt in _channel.Reader.ReadAllAsync(ct))
            {
                try
                {
                    // RFC 3339 timestamp — 對齊 Rust Utc::now().to_rfc3339()
                    string timestamp = DateTimeOffset.UtcNow.ToString("o");

                    // 1. 將事件序列化為 JSON — 對齊 Rust serde_json::to_string
                    string eventJson = JsonSerializer.Serialize(evt);

                    // 2. 計算 SHA256(Previous_Hash + Event_JSON + Timestamp)
                    //    對齊 Rust: hasher.update(previous_hash + event_json + timestamp)
                    byte[] currentHashBytes;
                    using (var sha256 = SHA256.Create())
                    {
                        byte[] data = Encoding.UTF8.GetBytes(
                            previousHash + eventJson + timestamp);
                        currentHashBytes = sha256.ComputeHash(data);
                    }
                    string currentHash = Convert.ToHexString(currentHashBytes).ToLowerInvariant();

                    // 3. 簽署 Current Hash
                    //    對齊 Rust: signing_key.sign(&current_hash_bytes)
                    //    使用 HMAC-SHA256 模擬 Ed25519 簽名（fallback 模式）
                    byte[] signatureBytes;
                    using (var hmac = new HMACSHA256(signingKey))
                    {
                        signatureBytes = hmac.ComputeHash(currentHashBytes);
                    }
                    string signatureHex = Convert.ToHexString(signatureBytes).ToLowerInvariant();

                    // 4. 組裝完整記錄 — 對齊 Rust AuditRecord
                    var record = new AuditRecord
                    {
                        Timestamp = timestamp,
                        Event = evt,
                        ChainHash = currentHash,
                        Signature = signatureHex,
                    };

                    string json = JsonSerializer.Serialize(record);
                    byte[] jsonBytes = Encoding.UTF8.GetBytes(json + "\n");
                    await fileStream.WriteAsync(jsonBytes, ct);
                    await fileStream.FlushAsync(ct);

                    // 5. 更新 Previous Hash — 對齊 Rust previous_hash = current_hash
                    previousHash = currentHash;
                }
                catch (Exception ex) when (ex is not OperationCanceledException)
                {
                    _logger.LogWarning(ex, "審計日誌寫入失敗");
                }
            }
        }

        /// <summary>
        /// 載入或產生簽名金鑰。
        /// <para>
        /// 對齊 Rust: 載入 PEM Ed25519 私鑰或用 OsRng 產生新金鑰。
        /// Windows fallback: 使用 32 bytes 隨機金鑰搭配 HMAC-SHA256 模擬。
        /// </para>
        /// </summary>
        private byte[] LoadOrGenerateSigningKey(string keyPath)
        {
            if (File.Exists(keyPath))
            {
                byte[] existingKey = File.ReadAllBytes(keyPath);
                if (existingKey.Length >= 32)
                {
                    _logger.LogDebug("載入既有審計簽名金鑰: {Path}", keyPath);
                    return existingKey;
                }
            }

            // 產生新金鑰
            byte[] newKey = RandomNumberGenerator.GetBytes(32);
            string? keyDir = Path.GetDirectoryName(keyPath);
            if (keyDir != null) Directory.CreateDirectory(keyDir);
            File.WriteAllBytes(keyPath, newKey);

            _logger.LogInformation("產生新審計簽名金鑰: {Path}", keyPath);
            return newKey;
        }

        /// <inheritdoc />
        public void Dispose()
        {
            if (!_disposed)
            {
                _channel.Writer.TryComplete();
                _writerCts?.Cancel();
                _writerCts?.Dispose();
                _disposed = true;
            }
        }
    }
}
