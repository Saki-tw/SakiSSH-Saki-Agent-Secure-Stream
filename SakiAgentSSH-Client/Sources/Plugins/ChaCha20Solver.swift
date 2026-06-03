// ChaCha20Solver.swift
// SakiAgentSSH Client — Plugin #1: ChaCha20-Poly1305 認知挑戰
//
// RFC 章節引用：
//   draft-sakistudio-sass-00, Appendix C.1 (anchor: chacha20-challenge)
//   "ChaCha20-Poly1305 Cognitive Challenge" — RFC 8439
//
// 流程：
//   1. Daemon 產生隨機 32-byte key、12-byte nonce、64-byte plaintext
//   2. 以 ChaCha20-Poly1305 加密後送出 (nonce, ciphertext)
//   3. Client 用預共享金鑰解密，回傳明文
//   4. Daemon 進行常數時間比對
//
// © 2026 Saki Studio. All rights reserved.

import Foundation
import CryptoKit
import os.log

// MARK: - ChaCha20 認知挑戰解密器

/// Plugin #1: ChaCha20-Poly1305 Cognitive Challenge Solver
///
/// 負責解密 daemon 送來的認知挑戰密文。
/// 使用 Apple CryptoKit 的 `ChaChaPoly` 實作，僅依賴原生框架。
final class ChaCha20Solver {

    // MARK: - 常數

    /// 挑戰 TTL（秒）— 超過此時間的挑戰視為過期
    /// RFC 規格：daemon 端以 60 秒 TTL 儲存 (key, nonce, plaintext)
    static let challengeTTLSeconds: TimeInterval = 60.0

    /// TTL 警告閾值（秒）— 接近過期時發出警告
    static let challengeTTLWarningThreshold: TimeInterval = 45.0

    /// 金鑰長度（bytes）
    static let keyLength = 32

    /// Nonce 長度（bytes）
    static let nonceLength = 12

    // MARK: - 日誌

    private static let logger = Logger(
        subsystem: "tw.com.saki-studio.SakiAgentSSH-Client",
        category: "ChaCha20Solver"
    )

    // MARK: - 挑戰結果

    /// 挑戰解密結果
    struct ChallengeResult {
        /// 解密後的明文
        let plaintext: Data

        /// 解密耗時（秒）
        let elapsedTime: TimeInterval

        /// 是否接近 TTL 過期
        let isNearExpiry: Bool

        /// TTL 剩餘時間（秒）
        let remainingTTL: TimeInterval
    }

    /// 挑戰解密錯誤
    enum SolverError: LocalizedError {
        /// 金鑰長度不正確
        case invalidKeyLength(expected: Int, actual: Int)

        /// Nonce 長度不正確
        case invalidNonceLength(expected: Int, actual: Int)

        /// 密文為空
        case emptyCiphertext

        /// 解密失敗（認證標籤驗證失敗）
        case decryptionFailed(underlying: Error)

        /// 挑戰已超過 TTL
        case challengeExpired(elapsed: TimeInterval)

        var errorDescription: String? {
            switch self {
            case .invalidKeyLength(let expected, let actual):
                return "金鑰長度錯誤：預期 \(expected) bytes，實際 \(actual) bytes"
            case .invalidNonceLength(let expected, let actual):
                return "Nonce 長度錯誤：預期 \(expected) bytes，實際 \(actual) bytes"
            case .emptyCiphertext:
                return "密文為空"
            case .decryptionFailed(let error):
                return "ChaCha20-Poly1305 解密失敗：\(error.localizedDescription)"
            case .challengeExpired(let elapsed):
                return "挑戰已過期：已過 \(String(format: "%.1f", elapsed)) 秒（TTL: \(challengeTTLSeconds) 秒）"
            }
        }
    }

    // MARK: - 核心解密

    /// 解密 daemon 送來的認知挑戰
    ///
    /// - Parameters:
    ///   - key: 預共享對稱金鑰（32 bytes）
    ///   - nonce: Daemon 送來的 nonce（12 bytes）
    ///   - ciphertext: Daemon 送來的密文（含 Poly1305 認證標籤）
    ///   - challengeReceivedAt: 挑戰接收時間（用於 TTL 檢查）
    /// - Returns: 解密結果，包含明文與計時資訊
    /// - Throws: `SolverError` 解密過程中的錯誤
    static func solve(
        key: Data,
        nonce: Data,
        ciphertext: Data,
        challengeReceivedAt: Date = Date()
    ) throws -> ChallengeResult {
        let startTime = CFAbsoluteTimeGetCurrent()

        // 驗證金鑰長度
        guard key.count == keyLength else {
            throw SolverError.invalidKeyLength(expected: keyLength, actual: key.count)
        }

        // 驗證 nonce 長度
        guard nonce.count == nonceLength else {
            throw SolverError.invalidNonceLength(expected: nonceLength, actual: nonce.count)
        }

        // 驗證密文非空
        guard !ciphertext.isEmpty else {
            throw SolverError.emptyCiphertext
        }

        // TTL 檢查
        let elapsed = Date().timeIntervalSince(challengeReceivedAt)
        if elapsed > Self.challengeTTLSeconds {
            logger.warning("⚠️ 挑戰已過期：\(elapsed, privacy: .public) 秒")
            throw SolverError.challengeExpired(elapsed: elapsed)
        }

        // 建立 CryptoKit 金鑰與 nonce
        let symmetricKey = SymmetricKey(data: key)
        let chachaNonce = try ChaChaPoly.Nonce(data: nonce)

        // 解密
        // ChaChaPoly.SealedBox 需要 nonce + ciphertext + tag 的組合
        // daemon 送來的 ciphertext 已包含 Poly1305 tag（最後 16 bytes）
        let sealedBox: ChaChaPoly.SealedBox
        do {
            sealedBox = try ChaChaPoly.SealedBox(
                nonce: chachaNonce,
                ciphertext: ciphertext.dropLast(16),
                tag: ciphertext.suffix(16)
            )
        } catch {
            // 若密文長度不足以分離 tag，嘗試直接用完整資料
            do {
                sealedBox = try ChaChaPoly.SealedBox(
                    nonce: chachaNonce,
                    ciphertext: ciphertext,
                    tag: Data(repeating: 0, count: 16)
                )
            } catch {
                throw SolverError.decryptionFailed(underlying: error)
            }
        }

        let plaintext: Data
        do {
            plaintext = try ChaChaPoly.open(sealedBox, using: symmetricKey)
        } catch {
            throw SolverError.decryptionFailed(underlying: error)
        }

        let endTime = CFAbsoluteTimeGetCurrent()
        let solveTime = endTime - startTime

        // TTL 警告
        let remainingTTL = Self.challengeTTLSeconds - elapsed
        let isNearExpiry = remainingTTL < (Self.challengeTTLSeconds - Self.challengeTTLWarningThreshold)

        if isNearExpiry {
            logger.warning(
                "⚠️ 挑戰接近過期：剩餘 \(remainingTTL, privacy: .public) 秒"
            )
        }

        logger.info(
            "✅ ChaCha20 挑戰解密成功：明文 \(plaintext.count) bytes，耗時 \(String(format: "%.3f", solveTime)) 秒"
        )

        return ChallengeResult(
            plaintext: plaintext,
            elapsedTime: solveTime,
            isNearExpiry: isNearExpiry,
            remainingTTL: remainingTTL
        )
    }

    /// 簡化版解密（不含計時，用於快速呼叫）
    ///
    /// - Parameters:
    ///   - key: 預共享對稱金鑰（32 bytes）
    ///   - nonce: Nonce（12 bytes）
    ///   - ciphertext: 密文（含 Poly1305 tag）
    /// - Returns: 解密後的明文
    static func decrypt(key: Data, nonce: Data, ciphertext: Data) throws -> Data {
        let result = try solve(key: key, nonce: nonce, ciphertext: ciphertext)
        return result.plaintext
    }
}
