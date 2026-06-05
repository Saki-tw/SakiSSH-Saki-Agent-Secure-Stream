// SakiSSHServiceStubs.swift
// SakiAgentSSH Client — v6 RPC Client Stubs
//
// 對應 proto service 定義：
//   rpc RawFileTransfer(stream RawFileChunk) returns (RawFileTransferResponse);
//   rpc RenewSession(RenewSessionRequest) returns (RenewSessionResponse);
//
// 目前為 stub 實作，返回 unimplemented 錯誤。
// 待 gRPC-Swift 整合後替換為實際實作。
//
// © 2026 Saki Studio. All rights reserved.

import Foundation
import os.log

// MARK: - Service Stub Errors

/// RPC 呼叫錯誤
enum SakiSSHServiceError: LocalizedError {
    case unimplemented(String)
    case connectionFailed(String)
    case invalidResponse(String)

    var errorDescription: String? {
        switch self {
        case .unimplemented(let method):
            return "\(method) not yet implemented"
        case .connectionFailed(let detail):
            return "Connection failed: \(detail)"
        case .invalidResponse(let detail):
            return "Invalid response: \(detail)"
        }
    }
}

// MARK: - v6 Service Stubs

/// v6 新增 RPC 的 Client Stub
///
/// 提供 `RawFileTransfer` 和 `RenewSession` 的呼叫介面。
/// 目前為 stub 實作，待 gRPC-Swift 整合後替換。
enum SakiSSHServiceStubs {

    private static let logger = Logger(
        subsystem: "tw.com.saki-studio.SakiAgentSSH-Client",
        category: "ServiceStubs"
    )

    // MARK: - RawFileTransfer (Client Streaming → Unary)

    /// 傳送原始檔案資料至 daemon（v6 §7.3）
    ///
    /// Client streaming: 逐步發送 RawFileChunk（首個為 metadata，後續為 data）
    /// Server 回傳單一 RawFileTransferResponse
    ///
    /// - Parameter chunks: RawFileChunk 序列
    /// - Returns: RawFileTransferResponse
    /// - Throws: SakiSSHServiceError
    static func rawFileTransfer(
        chunks: [RawFileChunk]
    ) async throws -> RawFileTransferResponse {
        logger.warning("⚠️ RawFileTransfer stub called — not yet implemented")
        throw SakiSSHServiceError.unimplemented("RawFileTransfer")
    }

    // MARK: - RenewSession (Unary)

    /// 續期 session（v6 §5.2）
    ///
    /// - Parameter request: RenewSessionRequest
    /// - Returns: RenewSessionResponse
    /// - Throws: SakiSSHServiceError
    static func renewSession(
        request: RenewSessionRequest
    ) async throws -> RenewSessionResponse {
        logger.warning("⚠️ RenewSession stub called — not yet implemented")
        throw SakiSSHServiceError.unimplemented("RenewSession")
    }
}
