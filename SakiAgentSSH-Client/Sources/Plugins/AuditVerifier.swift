// AuditVerifier.swift
// SakiAgentSSH Client — Plugin #4: ED25519 Hash Chain Audit 驗證器
//
// RFC 章節引用：
//   draft-sakistudio-sass-00, Appendix C.4 (anchor: ed25519-audit)
//   "ED25519 Hash Chain Audit Log" — RFC 8032
//
// Audit Chain 結構：
//   - timestamp: RFC 3339 時間戳
//   - event: 結構化事件資料 (JSON)
//   - chain_hash: SHA256(previous_chain_hash || event_json || timestamp)
//   - signature: ED25519_Sign(daemon_private_key, chain_hash)
//
// 創世種子：SASS_GENESIS_BLOCK
//
// © 2026 Saki Studio. All rights reserved.

import Foundation
import CryptoKit
import os.log

// MARK: - Audit Chain 驗證器

/// Plugin #4: ED25519 Hash Chain Audit Verifier
///
/// 驗證 daemon 送來的稽核鏈完整性：
/// 1. SHA256 hash chain 連續性
/// 2. ED25519 簽名有效性
final class AuditVerifier {

    // MARK: - 常數

    /// 創世區塊種子（第一筆記錄的 previous_chain_hash）
    static let genesisBlockSeed = "SASS_GENESIS_BLOCK"

    /// 創世區塊 hash = SHA256("SASS_GENESIS_BLOCK")
    static var genesisHash: Data {
        let seedData = genesisBlockSeed.data(using: .utf8)!
        let digest = SHA256.hash(data: seedData)
        return Data(digest)
    }

    // MARK: - 日誌

    private static let logger = Logger(
        subsystem: "tw.com.saki-studio.SakiAgentSSH-Client",
        category: "AuditVerifier"
    )

    // MARK: - 資料結構

    /// 單筆稽核記錄
    struct AuditRecord: Codable {
        /// RFC 3339 時間戳
        let timestamp: String

        /// 結構化事件資料（JSON 字串）
        let event: String

        /// 鏈式 hash = SHA256(previous_chain_hash || event || timestamp)
        let chainHash: Data

        /// ED25519 簽名 = Sign(daemon_private_key, chain_hash)
        let signature: Data

        enum CodingKeys: String, CodingKey {
            case timestamp
            case event
            case chainHash = "chain_hash"
            case signature
        }
    }

    /// 驗證結果
    struct VerificationResult {
        /// 是否全部通過驗證
        let isValid: Bool

        /// 已驗證的記錄數量
        let verifiedCount: Int

        /// 總記錄數量
        let totalCount: Int

        /// 第一筆失敗的記錄索引（若有）
        let firstFailureIndex: Int?

        /// 失敗原因（若有）
        let failureReason: VerificationFailure?

        /// 驗證耗時（秒）
        let elapsedTime: TimeInterval
    }

    /// 驗證失敗原因
    enum VerificationFailure: LocalizedError {
        /// Hash chain 斷裂
        case hashChainBroken(index: Int, expected: Data, actual: Data)

        /// ED25519 簽名無效
        case invalidSignature(index: Int)

        /// 時間戳格式錯誤
        case invalidTimestamp(index: Int, value: String)

        /// 時間戳非單調遞增
        case timestampNotMonotonic(index: Int, previous: String, current: String)

        /// 空稽核鏈
        case emptyChain

        /// 公鑰格式錯誤
        case invalidPublicKey(reason: String)

        var errorDescription: String? {
            switch self {
            case .hashChainBroken(let index, _, _):
                return "Hash chain 在第 \(index) 筆記錄斷裂"
            case .invalidSignature(let index):
                return "第 \(index) 筆記錄的 ED25519 簽名無效"
            case .invalidTimestamp(let index, let value):
                return "第 \(index) 筆記錄的時間戳格式錯誤：\(value)"
            case .timestampNotMonotonic(let index, let previous, let current):
                return "第 \(index) 筆記錄時間戳非單調遞增：\(previous) → \(current)"
            case .emptyChain:
                return "稽核鏈為空"
            case .invalidPublicKey(let reason):
                return "公鑰格式錯誤：\(reason)"
            }
        }
    }

    // MARK: - 核心驗證

    /// 驗證完整的稽核鏈
    ///
    /// - Parameters:
    ///   - records: 稽核記錄陣列（按時間順序排列）
    ///   - publicKey: Daemon 的 ED25519 公鑰（32 bytes）
    /// - Returns: 驗證結果
    static func verify(
        records: [AuditRecord],
        publicKey: Data
    ) -> VerificationResult {
        let startTime = CFAbsoluteTimeGetCurrent()

        // 空鏈檢查
        guard !records.isEmpty else {
            return VerificationResult(
                isValid: false,
                verifiedCount: 0,
                totalCount: 0,
                firstFailureIndex: nil,
                failureReason: .emptyChain,
                elapsedTime: 0
            )
        }

        // 解析 ED25519 公鑰
        let ed25519PublicKey: Curve25519.Signing.PublicKey
        do {
            ed25519PublicKey = try Curve25519.Signing.PublicKey(rawRepresentation: publicKey)
        } catch {
            return VerificationResult(
                isValid: false,
                verifiedCount: 0,
                totalCount: records.count,
                firstFailureIndex: 0,
                failureReason: .invalidPublicKey(reason: error.localizedDescription),
                elapsedTime: CFAbsoluteTimeGetCurrent() - startTime
            )
        }

        // 逐筆驗證
        var previousChainHash = genesisHash

        for (index, record) in records.enumerated() {
            // 1. 驗證時間戳格式（RFC 3339）
            if !isValidRFC3339Timestamp(record.timestamp) {
                return makeFailureResult(
                    index: index,
                    total: records.count,
                    reason: .invalidTimestamp(index: index, value: record.timestamp),
                    startTime: startTime
                )
            }

            // 2. 驗證時間戳單調遞增
            if index > 0 {
                let prevTimestamp = records[index - 1].timestamp
                if record.timestamp < prevTimestamp {
                    return makeFailureResult(
                        index: index,
                        total: records.count,
                        reason: .timestampNotMonotonic(
                            index: index,
                            previous: prevTimestamp,
                            current: record.timestamp
                        ),
                        startTime: startTime
                    )
                }
            }

            // 3. 驗證 hash chain 連續性
            //    chain_hash = SHA256(previous_chain_hash || event_json || timestamp)
            let expectedHash = computeChainHash(
                previousHash: previousChainHash,
                event: record.event,
                timestamp: record.timestamp
            )

            if expectedHash != record.chainHash {
                logger.error(
                    "❌ Hash chain 在第 \(index) 筆斷裂"
                )
                return makeFailureResult(
                    index: index,
                    total: records.count,
                    reason: .hashChainBroken(
                        index: index,
                        expected: expectedHash,
                        actual: record.chainHash
                    ),
                    startTime: startTime
                )
            }

            // 4. 驗證 ED25519 簽名
            //    signature = ED25519_Sign(daemon_private_key, chain_hash)
            guard ed25519PublicKey.isValidSignature(record.signature, for: record.chainHash) else {
                logger.error(
                    "❌ 第 \(index) 筆記錄的 ED25519 簽名無效"
                )
                return makeFailureResult(
                    index: index,
                    total: records.count,
                    reason: .invalidSignature(index: index),
                    startTime: startTime
                )
            }

            previousChainHash = record.chainHash
        }

        let elapsed = CFAbsoluteTimeGetCurrent() - startTime
        logger.info(
            "✅ 稽核鏈驗證通過：\(records.count) 筆記錄，耗時 \(String(format: "%.3f", elapsed)) 秒"
        )

        return VerificationResult(
            isValid: true,
            verifiedCount: records.count,
            totalCount: records.count,
            firstFailureIndex: nil,
            failureReason: nil,
            elapsedTime: elapsed
        )
    }

    /// 驗證單筆記錄（增量驗證）
    ///
    /// - Parameters:
    ///   - record: 新的稽核記錄
    ///   - previousChainHash: 前一筆的 chain_hash（若為首筆則用 genesis hash）
    ///   - publicKey: Daemon 的 ED25519 公鑰
    /// - Returns: 驗證是否通過
    static func verifyIncremental(
        record: AuditRecord,
        previousChainHash: Data?,
        publicKey: Curve25519.Signing.PublicKey
    ) -> Bool {
        let prevHash = previousChainHash ?? genesisHash

        // 驗證 hash chain
        let expectedHash = computeChainHash(
            previousHash: prevHash,
            event: record.event,
            timestamp: record.timestamp
        )

        guard expectedHash == record.chainHash else {
            logger.error("❌ 增量驗證：hash chain 不符")
            return false
        }

        // 驗證簽名
        guard publicKey.isValidSignature(record.signature, for: record.chainHash) else {
            logger.error("❌ 增量驗證：簽名無效")
            return false
        }

        return true
    }

    // MARK: - 輔助方法

    /// 計算 chain_hash = SHA256(previous_chain_hash || event_json || timestamp)
    private static func computeChainHash(
        previousHash: Data,
        event: String,
        timestamp: String
    ) -> Data {
        var hasher = SHA256()
        hasher.update(data: previousHash)
        hasher.update(data: event.data(using: .utf8) ?? Data())
        hasher.update(data: timestamp.data(using: .utf8) ?? Data())
        return Data(hasher.finalize())
    }

    /// 驗證 RFC 3339 時間戳格式
    private static func isValidRFC3339Timestamp(_ timestamp: String) -> Bool {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]

        if formatter.date(from: timestamp) != nil {
            return true
        }

        // 嘗試不含小數秒的格式
        formatter.formatOptions = [.withInternetDateTime]
        return formatter.date(from: timestamp) != nil
    }

    /// 建立失敗結果
    private static func makeFailureResult(
        index: Int,
        total: Int,
        reason: VerificationFailure,
        startTime: CFAbsoluteTime
    ) -> VerificationResult {
        VerificationResult(
            isValid: false,
            verifiedCount: index,
            totalCount: total,
            firstFailureIndex: index,
            failureReason: reason,
            elapsedTime: CFAbsoluteTimeGetCurrent() - startTime
        )
    }
}
