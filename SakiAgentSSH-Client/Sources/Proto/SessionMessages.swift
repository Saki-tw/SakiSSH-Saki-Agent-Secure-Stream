// SessionMessages.swift
// SakiAgentSSH Client — v6 Proto Message Types: Session Renewal
//
// 對應 proto 定義：
//   rpc RenewSession(RenewSessionRequest) returns (RenewSessionResponse);
//   message RenewSessionRequest { ... }
//   message RenewSessionResponse { ... }
//
// © 2026 Saki Studio. All rights reserved.

import Foundation

// MARK: - RenewSessionRequest

/// v6 §5.2 Session 續期請求
///
/// 對應 proto: `message RenewSessionRequest`
struct RenewSessionRequest: Codable, Sendable {
    /// Session ID（proto field 1）
    let sessionId: String

    /// Client hint: 請求的續期秒數（proto field 2）
    /// Daemon MAY honor, cap, or ignore。
    /// 若為 0 或未設定，daemon SHOULD 使用預設續期時長。
    let requestedExtensionSeconds: UInt32

    enum CodingKeys: String, CodingKey {
        case sessionId = "session_id"
        case requestedExtensionSeconds = "requested_extension_seconds"
    }
}

// MARK: - RenewSessionResponse

/// v6 §5.2 Session 續期回應
///
/// 對應 proto: `message RenewSessionResponse`
struct RenewSessionResponse: Codable, Sendable {
    /// 是否成功（proto field 1）
    let success: Bool

    /// 新的過期時間（RFC 3339 timestamp）（proto field 2）
    let newExpiresAt: String

    /// 訊息（proto field 3）
    let message: String

    /// 實際授予的續期秒數（daemon 最終裁決）（proto field 4）
    let grantedExtensionSeconds: UInt32

    enum CodingKeys: String, CodingKey {
        case success
        case newExpiresAt = "new_expires_at"
        case message
        case grantedExtensionSeconds = "granted_extension_seconds"
    }
}
