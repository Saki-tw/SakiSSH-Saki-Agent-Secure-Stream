// RawFileTransferMessages.swift
// SakiAgentSSH Client — v6 Proto Message Types: Raw File Transfer
//
// 對應 proto 定義：
//   rpc RawFileTransfer(stream RawFileChunk) returns (RawFileTransferResponse);
//   message RawFileChunk { oneof payload { RawFileMetadata metadata = 1; bytes data = 2; } }
//   message RawFileMetadata { ... }
//   message RawFileTransferResponse { ... }
//
// © 2026 Saki Studio. All rights reserved.

import Foundation

// MARK: - RawFileMetadata

/// v6 §7.3 Raw File Transfer 的檔案元資料
///
/// 對應 proto: `message RawFileMetadata`
struct RawFileMetadata: Codable, Sendable {
    /// 遠端檔案路徑（proto field 1）
    let remotePath: String

    /// 檔案總大小（bytes）（proto field 2）
    let totalSize: UInt64

    /// 斷點續傳起始 offset（proto field 3）
    let offset: UInt64

    /// 完整性驗證 SHA-256 hex（proto field 4）
    let checksumSha256: String

    enum CodingKeys: String, CodingKey {
        case remotePath = "remote_path"
        case totalSize = "total_size"
        case offset
        case checksumSha256 = "checksum_sha256"
    }
}

// MARK: - RawFileChunk

/// v6 §7.3 Raw File Transfer 的資料區塊
///
/// 模擬 proto oneof 語義：首個 chunk 為 metadata，後續為 data
///
/// 對應 proto: `message RawFileChunk`
enum RawFileChunk: Sendable {
    /// 首個 chunk：檔案元資料（proto field 1）
    case metadata(RawFileMetadata)

    /// 後續 chunks：原始二進位資料（proto field 2）
    case data(Data)
}

// MARK: - RawFileTransferResponse

/// v6 §7.3 Raw File Transfer 的回應
///
/// 對應 proto: `message RawFileTransferResponse`
struct RawFileTransferResponse: Codable, Sendable {
    /// 是否成功（proto field 1）
    let success: Bool

    /// 訊息（proto field 2）
    let message: String

    /// 已寫入位元組數（proto field 3）
    let bytesWritten: UInt64

    /// 寫入後校驗 SHA-256 hex（proto field 4）
    let checksumSha256: String

    enum CodingKeys: String, CodingKey {
        case success
        case message
        case bytesWritten = "bytes_written"
        case checksumSha256 = "checksum_sha256"
    }
}
