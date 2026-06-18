// =============================================================================
// SakiSshDaemon.Interop — Messages/SessionMessages.cs
// SASS (Saki Agent Secure Stream) — Session 管理 Message 定義
//
// 對應 sakissh.proto §5.2 Session Lifecycle 的 2 個 message：
//   - RenewSessionRequest (client 請求續期)
//   - RenewSessionResponse (daemon 回應)
//
// 注意：本專案不使用 protoc 自動產出，而是手動定義 POCO class。
// 欄位名稱與編號對應 proto 定義，序列化由 Rust core 處理。
//
// 參考: draft-sakistudio-sass-07 §5.2
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;

namespace SakiSshDaemon.Interop.Messages
{
    /// <summary>
    /// Session 續期請求。
    /// <para>
    /// 對應 proto: <c>message RenewSessionRequest</c>
    /// Client 請求延長 session 的有效期限。Daemon MAY honor, cap, or ignore。
    /// </para>
    /// </summary>
    public sealed class RenewSessionRequest
    {
        /// <summary>
        /// 要續期的 session ID。
        /// 對應 proto field: session_id = 1
        /// </summary>
        public string SessionId { get; set; } = string.Empty;

        /// <summary>
        /// Client hint: 請求的續期秒數。
        /// Daemon MAY honor, cap, or ignore。
        /// 若為 0 或未設定，daemon SHOULD 使用預設續期時長。
        /// 對應 proto field: requested_extension_seconds = 2
        /// </summary>
        public uint RequestedExtensionSeconds { get; set; }
    }

    /// <summary>
    /// Session 續期回應。
    /// <para>
    /// 對應 proto: <c>message RenewSessionResponse</c>
    /// Daemon 回報續期結果，包含新的過期時間與實際授予的秒數。
    /// </para>
    /// </summary>
    public sealed class RenewSessionResponse
    {
        /// <summary>
        /// 續期是否成功。
        /// 對應 proto field: success = 1
        /// </summary>
        public bool Success { get; set; }

        /// <summary>
        /// 新的過期時間（RFC 3339 timestamp）。
        /// 對應 proto field: new_expires_at = 2
        /// </summary>
        public string NewExpiresAt { get; set; } = string.Empty;

        /// <summary>
        /// 回應訊息。
        /// 對應 proto field: message = 3
        /// </summary>
        public string Message { get; set; } = string.Empty;

        /// <summary>
        /// 實際授予的續期秒數（daemon 最終裁決）。
        /// 對應 proto field: granted_extension_seconds = 4
        /// </summary>
        public uint GrantedExtensionSeconds { get; set; }
    }
}
