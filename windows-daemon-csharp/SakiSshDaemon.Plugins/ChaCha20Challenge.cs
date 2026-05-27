// =============================================================================
// SakiSshDaemon.Plugins — ChaCha20Challenge.cs
// SASS Plugin #1: ChaCha20-Poly1305 認知挑戰
//
// 對應 Rust: challenge_store.rs + threat_defense.rs
// RFC 參考: draft-sakistudio-sass-00 Appendix C.1 (anchor: chacha20-challenge)
//
// 實作流程:
// 1. Daemon 產生隨機 32-byte key、12-byte nonce、64-byte plaintext
// 2. 使用 ChaCha20-Poly1305 加密 plaintext
// 3. 儲存 (key, nonce, plaintext) 並設定 60 秒 TTL
// 4. 傳送 (nonce, ciphertext) 給 Agent
// 5. Agent 解密後回傳 plaintext
// 6. Daemon 以 constant-time 比對驗證
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;
using System.Collections.Concurrent;
using System.Security.Cryptography;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Logging;

namespace SakiSshDaemon.Plugins
{
    /// <summary>
    /// SASS Plugin #1: ChaCha20-Poly1305 認知挑戰。
    /// <para>
    /// RFC draft-sakistudio-sass-00 §C.1: 使用 ChaCha20-Poly1305 (RFC 8439)
    /// 作為認知挑戰機制。挑戰密文對缺乏共享金鑰的觀察者而言，
    /// 必須與隨機資料不可區分。
    /// </para>
    /// <para>
    /// Windows 差異: 使用 System.Security.Cryptography.ChaCha20Poly1305
    /// （.NET 8 內建），取代 Rust 的 chacha20poly1305 crate。
    /// </para>
    /// </summary>
    public sealed class ChaCha20Challenge : IPlugin
    {
        /// <summary>認知挑戰預設 TTL（秒）— 對齊 Rust ChallengeStore::new(60)</summary>
        private const int DefaultTtlSeconds = 60;

        /// <summary>隨機明文長度 — 對齊 Rust generate_challenge() 的 64 bytes</summary>
        private const int PlaintextLength = 64;

        /// <summary>ChaCha20-Poly1305 Nonce 長度 (12 bytes)</summary>
        private const int NonceLength = 12;

        /// <summary>ChaCha20-Poly1305 Key 長度 (32 bytes)</summary>
        private const int KeyLength = 32;

        /// <summary>ChaCha20-Poly1305 Tag 長度 (16 bytes)</summary>
        private const int TagLength = 16;

        /// <summary>背景清理間隔（秒）— 對齊 Rust spawn_cleanup_task 的 60 秒</summary>
        private const int CleanupIntervalSeconds = 60;

        private readonly ILogger<ChaCha20Challenge> _logger;
        private readonly ConcurrentDictionary<string, ChallengeEntry> _entries;
        private readonly byte[] _staticKey;
        private CancellationTokenSource? _cleanupCts;
        private bool _disposed;

        /// <summary>單個挑戰的內部狀態 — 對齊 Rust ChallengeEntry</summary>
        private sealed class ChallengeEntry
        {
            public byte[] Nonce { get; init; } = Array.Empty<byte>();
            public byte[] Plaintext { get; init; } = Array.Empty<byte>();
            public DateTime CreatedAt { get; init; }
            public TimeSpan Ttl { get; init; }
            public bool IsExpired => DateTime.UtcNow - CreatedAt > Ttl;
        }

        public ChaCha20Challenge(ILogger<ChaCha20Challenge> logger)
        {
            _logger = logger ?? throw new ArgumentNullException(nameof(logger));
            _entries = new ConcurrentDictionary<string, ChallengeEntry>();

            // 載入或產生固定的 ChaCha20 Key — 對齊 Rust load_or_generate_key()
            _staticKey = LoadOrGenerateKey();
        }

        /// <inheritdoc />
        public string Name => "ChaCha20-Poly1305 Cognitive Challenge";

        /// <inheritdoc />
        public string RfcSection => "Appendix C.1 (chacha20-challenge)";

        /// <inheritdoc />
        public bool IsHealthy => !_disposed;

        /// <inheritdoc />
        public Task<bool> InitializeAsync(CancellationToken cancellationToken = default)
        {
            // 啟動背景清理任務 — 對齊 Rust spawn_cleanup_task()
            _cleanupCts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
            _ = Task.Run(() => CleanupLoopAsync(_cleanupCts.Token), _cleanupCts.Token);

            _logger.LogInformation(
                "Plugin #1 ({Name}) 初始化完成 (TTL={Ttl}s)",
                Name, DefaultTtlSeconds);
            return Task.FromResult(true);
        }

        /// <summary>
        /// 產生新的 ChaCha20-Poly1305 認知挑戰。
        /// <para>對齊 Rust: ChallengeStore::generate_challenge()</para>
        /// </summary>
        /// <returns>包含 (nonce, ciphertext) 的元組，用於 AuthResponse</returns>
        public (byte[] Nonce, byte[] Ciphertext) GenerateChallenge()
        {
            // 產生隨機 nonce (12 bytes)
            byte[] nonce = RandomNumberGenerator.GetBytes(NonceLength);

            // 產生隨機明文 (64 bytes)
            byte[] plaintext = RandomNumberGenerator.GetBytes(PlaintextLength);

            // ChaCha20-Poly1305 加密
            byte[] ciphertext = new byte[PlaintextLength];
            byte[] tag = new byte[TagLength];

            using (var cipher = new ChaCha20Poly1305(_staticKey))
            {
                cipher.Encrypt(nonce, plaintext, ciphertext, tag);
            }

            // 合併 ciphertext + tag
            byte[] fullCiphertext = new byte[PlaintextLength + TagLength];
            Array.Copy(ciphertext, 0, fullCiphertext, 0, PlaintextLength);
            Array.Copy(tag, 0, fullCiphertext, PlaintextLength, TagLength);

            // 儲存挑戰狀態 — 使用 nonce 的 hex 作為 key
            string nonceHex = Convert.ToHexString(nonce);
            var entry = new ChallengeEntry
            {
                Nonce = nonce,
                Plaintext = plaintext,
                CreatedAt = DateTime.UtcNow,
                Ttl = TimeSpan.FromSeconds(DefaultTtlSeconds),
            };
            _entries[nonceHex] = entry;

            _logger.LogDebug("產生 ChaCha20 認知挑戰: nonce={NonceHex}", nonceHex);
            return (nonce, fullCiphertext);
        }

        /// <summary>
        /// 驗證 Agent 的挑戰回應。
        /// <para>對齊 Rust: ChallengeStore::verify_response() — constant-time 比對</para>
        /// </summary>
        /// <param name="nonce">原始 nonce</param>
        /// <param name="response">Agent 解密後的明文</param>
        /// <returns>驗證是否通過</returns>
        public bool VerifyResponse(byte[] nonce, byte[] response)
        {
            string nonceHex = Convert.ToHexString(nonce);

            if (!_entries.TryRemove(nonceHex, out var entry))
            {
                _logger.LogWarning("ChaCha20 challenge nonce 未找到（可能為重放攻擊）");
                return false;
            }

            // 檢查 TTL — 對齊 Rust verify_response 的 TTL 檢查
            if (entry.IsExpired)
            {
                _logger.LogWarning("ChaCha20 challenge 已過期 (TTL 超時)");
                return false;
            }

            // 長度不同時仍需避免提早返回洩漏資訊
            if (response.Length != entry.Plaintext.Length)
            {
                _logger.LogWarning("ChaCha20 challenge 回應長度不匹配");
                return false;
            }

            // constant-time 比對 — 對齊 Rust subtle::ConstantTimeEq
            bool passed = CryptographicOperations.FixedTimeEquals(
                response, entry.Plaintext);

            if (passed)
            {
                _logger.LogInformation("ChaCha20 認知挑戰驗證成功");
            }
            else
            {
                _logger.LogWarning("ChaCha20 認知挑戰驗證失敗：明文不匹配");
            }

            return passed;
        }

        /// <summary>
        /// 遍歷所有待驗證挑戰，嘗試 constant-time 比對找到匹配。
        /// <para>對齊 Rust: ChallengeStore::try_verify_any()</para>
        /// <para>
        /// 用於 ChallengeRequest Proto 未包含 nonce 欄位的場景。
        /// 複雜度 O(n)，但 n 極小（TTL 60s，實務上 pending 數量 &lt; 10）。
        /// </para>
        /// </summary>
        public bool TryVerifyAny(byte[] response)
        {
            // 先清理過期條目
            CleanupExpired();

            foreach (var kvp in _entries)
            {
                var entry = kvp.Value;
                if (response.Length == entry.Plaintext.Length &&
                    CryptographicOperations.FixedTimeEquals(response, entry.Plaintext))
                {
                    _entries.TryRemove(kvp.Key, out _);
                    _logger.LogInformation("ChaCha20 認知挑戰驗證成功（try_verify_any）");
                    return true;
                }
            }

            _logger.LogWarning("try_verify_any: 無匹配的挑戰（可能已過期或為重放攻擊）");
            return false;
        }

        /// <summary>
        /// 載入或產生固定的 ChaCha20 Key。
        /// <para>
        /// 對齊 Rust: ChallengeStore::load_or_generate_key()
        /// Windows 路徑: %USERPROFILE%\.sakissh\chacha20.key
        /// </para>
        /// </summary>
        private byte[] LoadOrGenerateKey()
        {
            // Windows 路徑對應 — Rust 使用 dirs::home_dir().join(".sakissh/chacha20.key")
            string homeDir = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            string keyDir = System.IO.Path.Combine(homeDir, ".sakissh");
            string keyPath = System.IO.Path.Combine(keyDir, "chacha20.key");

            if (System.IO.File.Exists(keyPath))
            {
                byte[] existingKey = System.IO.File.ReadAllBytes(keyPath);
                if (existingKey.Length == KeyLength)
                {
                    _logger.LogDebug("載入既有 ChaCha20 key: {Path}", keyPath);
                    return existingKey;
                }
            }

            // 產生新 key
            byte[] newKey = RandomNumberGenerator.GetBytes(KeyLength);
            System.IO.Directory.CreateDirectory(keyDir);
            System.IO.File.WriteAllBytes(keyPath, newKey);

            _logger.LogInformation("產生新 ChaCha20 key: {Path}", keyPath);
            return newKey;
        }

        /// <summary>清理過期的挑戰條目 — 對齊 Rust cleanup_expired()</summary>
        private void CleanupExpired()
        {
            foreach (var kvp in _entries)
            {
                if (kvp.Value.IsExpired)
                {
                    _entries.TryRemove(kvp.Key, out _);
                }
            }
        }

        /// <summary>背景清理迴圈 — 對齊 Rust spawn_cleanup_task()</summary>
        private async Task CleanupLoopAsync(CancellationToken ct)
        {
            while (!ct.IsCancellationRequested)
            {
                try
                {
                    await Task.Delay(
                        TimeSpan.FromSeconds(CleanupIntervalSeconds), ct);
                    CleanupExpired();
                }
                catch (OperationCanceledException)
                {
                    break;
                }
            }
        }

        /// <inheritdoc />
        public void Dispose()
        {
            if (!_disposed)
            {
                _cleanupCts?.Cancel();
                _cleanupCts?.Dispose();
                _entries.Clear();
                _disposed = true;
            }
        }
    }
}
