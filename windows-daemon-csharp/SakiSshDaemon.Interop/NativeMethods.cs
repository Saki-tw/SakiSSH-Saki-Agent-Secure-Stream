// =============================================================================
// SakiSshDaemon.Interop — NativeMethods.cs
// SASS (Saki Agent Secure Stream) — Rust FFI P/Invoke 宣告
//
// 對應 Rust cdylib: saki_ssh_core.dll
// 參考: draft-sakistudio-sass-00 Appendix C
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;
using System.Runtime.InteropServices;

namespace SakiSshDaemon.Interop
{
    /// <summary>
    /// Rust core cdylib 的 P/Invoke 原生方法宣告。
    /// 所有 FFI 函數均對應 saki-ssh-daemon Rust crate 匯出的 C ABI 符號。
    /// </summary>
    public static class NativeMethods
    {
        /// <summary>Rust cdylib 名稱（不含副檔名，Runtime 自動附加 .dll）</summary>
        private const string RustLib = "saki_ssh_core";

        // =====================================================================
        // ChaCha20-Poly1305 認知挑戰 (Plugin #1)
        // 對應 Rust: challenge_store.rs + threat_defense.rs
        // RFC 參考: draft-sakistudio-sass-00 Appendix C.1
        // =====================================================================

        /// <summary>
        /// 產生 ChaCha20-Poly1305 認知挑戰。
        /// Rust 端會生成隨機 key/nonce/plaintext，加密後回傳 nonce + ciphertext。
        /// </summary>
        /// <param name="nonceOut">輸出 12 bytes nonce</param>
        /// <param name="ciphertextOut">輸出加密後的密文 buffer</param>
        /// <param name="ciphertextLen">密文 buffer 長度（輸入/輸出）</param>
        /// <returns>0 表示成功，非零為錯誤碼</returns>
        [DllImport(RustLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "sass_challenge_generate")]
        public static extern int SassChallengeGenerate(
            [Out] byte[] nonceOut,
            [Out] byte[] ciphertextOut,
            ref int ciphertextLen);

        /// <summary>
        /// 驗證 ChaCha20 認知挑戰回應（constant-time 比對）。
        /// </summary>
        /// <param name="nonce">原始 12 bytes nonce</param>
        /// <param name="response">Agent 回傳的解密明文</param>
        /// <param name="responseLen">明文長度</param>
        /// <returns>1 表示驗證通過，0 表示失敗</returns>
        [DllImport(RustLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "sass_challenge_verify")]
        public static extern int SassChallengeVerify(
            [In] byte[] nonce,
            [In] byte[] response,
            int responseLen);

        /// <summary>
        /// 嘗試對所有待驗證挑戰進行 constant-time 比對（O(n) 遍歷）。
        /// 用於 ChallengeRequest 未包含 nonce 欄位的場景。
        /// </summary>
        /// <param name="response">Agent 回傳的解密明文</param>
        /// <param name="responseLen">明文長度</param>
        /// <returns>1 表示找到匹配，0 表示無匹配</returns>
        [DllImport(RustLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "sass_challenge_verify_any")]
        public static extern int SassChallengeVerifyAny(
            [In] byte[] response,
            int responseLen);

        // =====================================================================
        // TLS Exporter 通道綁定 (Plugin #2)
        // 對應 Rust: threat_defense.rs
        // RFC 參考: draft-sakistudio-sass-00 Appendix C.2
        // =====================================================================

        /// <summary>
        /// 從 TLS session 推導 Exported Keying Material (EKM)。
        /// Label = "EXPORTER-sakissh-chacha20-v14"，Length = 44 bytes。
        /// </summary>
        /// <param name="sessionUuid">16 bytes session UUID 作為 context</param>
        /// <param name="ekmOut">輸出 44 bytes EKM (32 key + 12 nonce)</param>
        /// <returns>0 表示成功</returns>
        [DllImport(RustLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "sass_derive_ekm")]
        public static extern int SassDeriveEkm(
            [In] byte[] sessionUuid,
            [Out] byte[] ekmOut);

        /// <summary>
        /// 驗證 Client 提供的 EKM HMAC 通道綁定。
        /// constant-time 驗證確保不洩漏 timing 資訊。
        /// </summary>
        /// <param name="ekm">44 bytes EKM</param>
        /// <param name="plaintext">解密明文</param>
        /// <param name="plaintextLen">明文長度</param>
        /// <param name="clientHmac">Client 提供的 HMAC</param>
        /// <param name="hmacLen">HMAC 長度</param>
        /// <returns>1 表示驗證通過，0 表示失敗</returns>
        [DllImport(RustLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "sass_verify_ekm_hmac")]
        public static extern int SassVerifyEkmHmac(
            [In] byte[] ekm,
            [In] byte[] plaintext,
            int plaintextLen,
            [In] byte[] clientHmac,
            int hmacLen);

        // =====================================================================
        // Ed25519 審計簽名 (Plugin #4)
        // 對應 Rust: audit.rs
        // RFC 參考: draft-sakistudio-sass-00 Appendix C.4
        // =====================================================================

        /// <summary>
        /// 使用 Ed25519 私鑰簽署 hash chain 摘要。
        /// </summary>
        /// <param name="hashBytes">SHA256 摘要 (32 bytes)</param>
        /// <param name="signatureOut">輸出 Ed25519 簽名 (64 bytes)</param>
        /// <returns>0 表示成功</returns>
        [DllImport(RustLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "sass_audit_sign")]
        public static extern int SassAuditSign(
            [In] byte[] hashBytes,
            [Out] byte[] signatureOut);

        /// <summary>
        /// 驗證 Ed25519 審計簽名。
        /// </summary>
        /// <param name="hashBytes">SHA256 摘要 (32 bytes)</param>
        /// <param name="signature">Ed25519 簽名 (64 bytes)</param>
        /// <param name="publicKey">Ed25519 公鑰 (32 bytes)</param>
        /// <returns>1 表示驗證通過，0 表示失敗</returns>
        [DllImport(RustLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "sass_audit_verify")]
        public static extern int SassAuditVerify(
            [In] byte[] hashBytes,
            [In] byte[] signature,
            [In] byte[] publicKey);

        // =====================================================================
        // 生命週期管理
        // =====================================================================

        /// <summary>
        /// 初始化 Rust core runtime（必須在所有 FFI 呼叫前調用一次）。
        /// </summary>
        /// <returns>0 表示成功</returns>
        [DllImport(RustLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "sass_init")]
        public static extern int SassInit();

        /// <summary>
        /// 清理 Rust core runtime 資源。
        /// </summary>
        [DllImport(RustLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "sass_shutdown")]
        public static extern void SassShutdown();
    }
}
