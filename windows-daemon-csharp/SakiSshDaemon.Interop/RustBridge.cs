// =============================================================================
// SakiSshDaemon.Interop — RustBridge.cs
// SASS (Saki Agent Secure Stream) — Rust FFI 高階 Wrapper
//
// 將原始 P/Invoke 呼叫封裝為安全的 C# 介面，處理記憶體管理與錯誤轉譯。
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;
using Microsoft.Extensions.Logging;

namespace SakiSshDaemon.Interop
{
    /// <summary>
    /// Rust core 高階 wrapper，封裝 <see cref="NativeMethods"/> 的 P/Invoke 呼叫。
    /// 提供型別安全的 C# 介面，處理 byte[] 分配、錯誤碼轉換與生命週期管理。
    /// </summary>
    public sealed class RustBridge : IDisposable
    {
        private readonly ILogger<RustBridge> _logger;
        private bool _initialized;
        private bool _disposed;

        public RustBridge(ILogger<RustBridge> logger)
        {
            _logger = logger ?? throw new ArgumentNullException(nameof(logger));
        }

        /// <summary>
        /// 初始化 Rust core runtime。
        /// 必須在任何 FFI 呼叫前調用一次。
        /// </summary>
        /// <exception cref="InvalidOperationException">當 Rust core 初始化失敗時</exception>
        public void Initialize()
        {
            if (_initialized)
            {
                _logger.LogWarning("RustBridge 已經初始化，跳過重複呼叫");
                return;
            }

            try
            {
                int result = NativeMethods.SassInit();
                if (result != 0)
                {
                    throw new InvalidOperationException(
                        $"Rust core 初始化失敗，錯誤碼: {result}");
                }
                _initialized = true;
                _logger.LogInformation("Rust core runtime 初始化成功");
            }
            catch (DllNotFoundException ex)
            {
                _logger.LogError(ex,
                    "找不到 Rust cdylib (saki_ssh_core.dll)。" +
                    "請確認 DLL 已放置於應用程式目錄或 PATH 中。" +
                    "Windows C# Daemon 將使用純 C# fallback 模式運行。");
                // 不拋出異常 — 允許純 C# fallback 模式
            }
        }

        /// <summary>
        /// 產生 ChaCha20-Poly1305 認知挑戰。
        /// </summary>
        /// <returns>包含 (nonce, ciphertext) 的元組</returns>
        public (byte[] Nonce, byte[] Ciphertext) GenerateChallenge()
        {
            EnsureInitialized();

            byte[] nonce = new byte[12];
            byte[] ciphertext = new byte[128]; // 64 bytes plaintext + 16 bytes tag = 80，預留空間
            int ciphertextLen = ciphertext.Length;

            int result = NativeMethods.SassChallengeGenerate(nonce, ciphertext, ref ciphertextLen);
            if (result != 0)
            {
                throw new InvalidOperationException($"ChaCha20 挑戰產生失敗，錯誤碼: {result}");
            }

            // 截取實際長度
            byte[] actualCiphertext = new byte[ciphertextLen];
            Array.Copy(ciphertext, actualCiphertext, ciphertextLen);

            return (nonce, actualCiphertext);
        }

        /// <summary>
        /// 驗證 ChaCha20 認知挑戰回應。
        /// </summary>
        public bool VerifyChallenge(byte[] nonce, byte[] response)
        {
            EnsureInitialized();
            return NativeMethods.SassChallengeVerify(nonce, response, response.Length) == 1;
        }

        /// <summary>
        /// 遍歷所有待驗證挑戰進行驗證（無 nonce 時使用）。
        /// </summary>
        public bool VerifyChallengeAny(byte[] response)
        {
            EnsureInitialized();
            return NativeMethods.SassChallengeVerifyAny(response, response.Length) == 1;
        }

        /// <summary>
        /// 推導 TLS Exported Keying Material。
        /// </summary>
        /// <param name="sessionUuid">16 bytes session UUID</param>
        /// <returns>44 bytes EKM (32 key + 12 nonce)</returns>
        public byte[] DeriveEkm(byte[] sessionUuid)
        {
            EnsureInitialized();
            if (sessionUuid.Length != 16)
                throw new ArgumentException("Session UUID 必須為 16 bytes", nameof(sessionUuid));

            byte[] ekm = new byte[44];
            int result = NativeMethods.SassDeriveEkm(sessionUuid, ekm);
            if (result != 0)
            {
                throw new InvalidOperationException($"EKM 推導失敗，錯誤碼: {result}");
            }
            return ekm;
        }

        /// <summary>
        /// 驗證 EKM HMAC 通道綁定。
        /// </summary>
        public bool VerifyEkmHmac(byte[] ekm, byte[] plaintext, byte[] clientHmac)
        {
            EnsureInitialized();
            return NativeMethods.SassVerifyEkmHmac(
                ekm, plaintext, plaintext.Length, clientHmac, clientHmac.Length) == 1;
        }

        /// <summary>
        /// 使用 Ed25519 簽署審計 hash chain 摘要。
        /// </summary>
        public byte[] AuditSign(byte[] hashBytes)
        {
            EnsureInitialized();
            byte[] signature = new byte[64];
            int result = NativeMethods.SassAuditSign(hashBytes, signature);
            if (result != 0)
            {
                throw new InvalidOperationException($"Ed25519 審計簽名失敗，錯誤碼: {result}");
            }
            return signature;
        }

        /// <summary>是否已成功初始化（含 fallback 模式判斷）</summary>
        public bool IsNativeAvailable => _initialized;

        private void EnsureInitialized()
        {
            if (!_initialized)
            {
                throw new InvalidOperationException(
                    "Rust core 尚未初始化。請先呼叫 Initialize() 或確認 saki_ssh_core.dll 可用。");
            }
        }

        public void Dispose()
        {
            if (!_disposed)
            {
                if (_initialized)
                {
                    try
                    {
                        NativeMethods.SassShutdown();
                        _logger.LogInformation("Rust core runtime 已關閉");
                    }
                    catch (Exception ex)
                    {
                        _logger.LogError(ex, "Rust core shutdown 過程發生錯誤");
                    }
                }
                _disposed = true;
            }
        }
    }
}
