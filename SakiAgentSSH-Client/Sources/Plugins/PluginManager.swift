// PluginManager.swift
// SakiAgentSSH Client — 統一 Plugin 管理器
//
// RFC 章節引用：
//   draft-sakistudio-sass-00, Appendix C (anchor: plugins-reference)
//   "Saki Studio Plugins Reference Implementation"
//
// 管理的 Plugins：
//   #1 ChaCha20Solver         — 認知挑戰解密
//   #2 TlsExporterBinding     — TLS EKM 綁定
//   #4 AuditVerifier           — 稽核鏈驗證
//   #7 EnvInjectorClient       — 環境變數注入
//
// © 2026 Saki Studio. All rights reserved.

import Foundation
import CryptoKit
import os.log

// MARK: - Plugin 管理器

/// 統一管理 SASS 協議所有 Plugins 的生命週期與高階 API
///
/// 提供以下核心功能：
/// - `solveChallenge()`: 解密 ChaCha20 認知挑戰
/// - `verifyAudit()`: 驗證 ED25519 稽核鏈
/// - `bindTlsExporter()`: 綁定 TLS Exporter 金鑰材料
/// - `prepareEnv()`: 準備揮發性快取環境變數
@MainActor
final class PluginManager: ObservableObject {

    // MARK: - 單例

    /// 共用 PluginManager 實例
    static let shared = PluginManager()

    // MARK: - 日誌

    private let logger = Logger(
        subsystem: "tw.com.saki-studio.SakiAgentSSH-Client",
        category: "PluginManager"
    )

    // MARK: - 狀態

    /// 當前 session ID
    @Published private(set) var currentSessionID: String?

    /// Plugin 狀態
    @Published private(set) var pluginStatus: [PluginID: PluginState] = [
        .chacha20Solver: .idle,
        .tlsExporter: .idle,
        .auditVerifier: .idle,
        .envInjector: .idle,
    ]

    /// 最後一次挑戰結果
    @Published private(set) var lastChallengeResult: ChaCha20Solver.ChallengeResult?

    /// 最後一次稽核驗證結果
    @Published private(set) var lastAuditResult: AuditVerifier.VerificationResult?

    // MARK: - Plugin 識別

    /// Plugin 識別碼
    enum PluginID: String, CaseIterable, Identifiable {
        case chacha20Solver = "Plugin #1: ChaCha20 Solver"
        case tlsExporter = "Plugin #2: TLS Exporter"
        case auditVerifier = "Plugin #4: Audit Verifier"
        case envInjector = "Plugin #7: Env Injector"

        var id: String { rawValue }

        /// Plugin 圖示
        var systemImage: String {
            switch self {
            case .chacha20Solver: return "lock.shield"
            case .tlsExporter: return "key.fill"
            case .auditVerifier: return "checkmark.shield"
            case .envInjector: return "terminal"
            }
        }
    }

    /// Plugin 運行狀態
    enum PluginState: Equatable {
        case idle
        case running
        case success
        case failure(String)
    }

    // MARK: - 初始化

    private init() {
        logger.info("PluginManager 初始化完成")
    }

    // MARK: - Session 管理

    /// 設定當前 session ID
    func setSession(_ sessionID: String) {
        currentSessionID = sessionID
        logger.info("Session 設定：\(sessionID, privacy: .public)")
    }

    /// 清除 session
    func clearSession() {
        currentSessionID = nil
        lastChallengeResult = nil
        lastAuditResult = nil
        resetAllPluginStates()
        logger.info("Session 已清除")
    }

    // MARK: - Plugin #1: ChaCha20 認知挑戰

    /// 解密 daemon 送來的 ChaCha20-Poly1305 認知挑戰
    ///
    /// - Parameters:
    ///   - key: 預共享對稱金鑰（32 bytes）
    ///   - nonce: Daemon 送來的 nonce（12 bytes）
    ///   - ciphertext: Daemon 送來的密文（含 Poly1305 tag）
    ///   - receivedAt: 挑戰接收時間
    /// - Returns: 解密後的明文資料
    func solveChallenge(
        key: Data,
        nonce: Data,
        ciphertext: Data,
        receivedAt: Date = Date()
    ) throws -> Data {
        pluginStatus[.chacha20Solver] = .running
        logger.info("🔐 開始解密 ChaCha20 認知挑戰")

        do {
            let result = try ChaCha20Solver.solve(
                key: key,
                nonce: nonce,
                ciphertext: ciphertext,
                challengeReceivedAt: receivedAt
            )

            lastChallengeResult = result
            pluginStatus[.chacha20Solver] = .success

            if result.isNearExpiry {
                logger.warning(
                    "⚠️ 挑戰接近過期：剩餘 \(result.remainingTTL, privacy: .public) 秒"
                )
            }

            logger.info(
                "✅ 挑戰解密成功：\(result.plaintext.count) bytes，耗時 \(String(format: "%.3f", result.elapsedTime)) 秒"
            )

            return result.plaintext
        } catch {
            pluginStatus[.chacha20Solver] = .failure(error.localizedDescription)
            logger.error("❌ 挑戰解密失敗：\(error.localizedDescription)")
            throw error
        }
    }

    // MARK: - Plugin #2: TLS Exporter 綁定

    /// 計算 client_ekm_hmac（離線模式）
    ///
    /// 當無法直接存取 TLS metadata 時，使用預先取得的 EKM 金鑰。
    ///
    /// - Parameters:
    ///   - ekmKey: EKM 金鑰部分（32 bytes）
    ///   - sessionID: Session ID
    /// - Returns: HMAC-SHA256 結果
    func bindTlsExporter(ekmKey: Data, sessionID: String) throws -> Data {
        pluginStatus[.tlsExporter] = .running
        logger.info("🔑 開始計算 TLS Exporter HMAC")

        do {
            let hmac = try TlsExporterBinding.computeClientEkmHmac(
                ekmKey: ekmKey,
                sessionID: sessionID
            )

            pluginStatus[.tlsExporter] = .success
            logger.info("✅ TLS Exporter HMAC 計算成功：\(hmac.count) bytes")
            return hmac
        } catch {
            pluginStatus[.tlsExporter] = .failure(error.localizedDescription)
            logger.error("❌ TLS Exporter 計算失敗：\(error.localizedDescription)")
            throw error
        }
    }

    /// 拆分 EKM 原始材料為金鑰與 nonce
    func splitExportedKeyMaterial(_ rawEKM: Data) throws -> (key: Data, nonce: Data) {
        try TlsExporterBinding.splitEKM(rawEKM)
    }

    // MARK: - Plugin #4: 稽核鏈驗證

    /// 驗證 daemon 的 ED25519 稽核鏈
    ///
    /// - Parameters:
    ///   - records: 稽核記錄陣列
    ///   - publicKey: Daemon 的 ED25519 公鑰（32 bytes）
    /// - Returns: 驗證結果
    func verifyAudit(
        records: [AuditVerifier.AuditRecord],
        publicKey: Data
    ) -> AuditVerifier.VerificationResult {
        pluginStatus[.auditVerifier] = .running
        logger.info("🔍 開始驗證稽核鏈：\(records.count) 筆記錄")

        let result = AuditVerifier.verify(records: records, publicKey: publicKey)

        lastAuditResult = result

        if result.isValid {
            pluginStatus[.auditVerifier] = .success
            logger.info(
                "✅ 稽核鏈驗證通過：\(result.verifiedCount)/\(result.totalCount) 筆"
            )
        } else {
            let reason = result.failureReason?.localizedDescription ?? "未知原因"
            pluginStatus[.auditVerifier] = .failure(reason)
            logger.error("❌ 稽核鏈驗證失敗：\(reason)")
        }

        return result
    }

    // MARK: - Plugin #7: 環境變數注入

    /// 準備揮發性快取環境變數
    ///
    /// - Parameter session: Session ID（若為 nil，使用當前 session）
    /// - Returns: 環境變數字典
    func prepareEnv(session: String? = nil) -> [String: String] {
        let sessionID = session ?? currentSessionID ?? UUID().uuidString

        pluginStatus[.envInjector] = .running

        let env = EnvInjectorClient.prepareEnvironment(for: sessionID)

        pluginStatus[.envInjector] = .success
        logger.info("✅ 環境變數準備完成：\(env.count) 個")

        return env
    }

    /// 合併使用者環境變數與快取重導向
    ///
    /// - Parameters:
    ///   - userEnv: 使用者自訂環境變數
    ///   - session: Session ID
    /// - Returns: 合併後的環境變數
    func mergeEnv(userEnv: [String: String], session: String? = nil) -> [String: String] {
        let sessionID = session ?? currentSessionID ?? UUID().uuidString
        return EnvInjectorClient.mergeEnvironment(userEnv: userEnv, session: sessionID)
    }

    // MARK: - 全流程：認證 + 挑戰解答

    /// 完整認證流程（結合 Plugin #1 + #2）
    ///
    /// 1. 計算 TLS Exporter HMAC
    /// 2. 解密 ChaCha20 認知挑戰
    ///
    /// - Parameters:
    ///   - authResponse: Daemon 的認證回應（含 nonce + ciphertext）
    ///   - sharedKey: 預共享金鑰（32 bytes）
    ///   - ekmKey: EKM 金鑰（32 bytes，可選）
    ///   - sessionID: Session ID
    /// - Returns: (解密明文, client_ekm_hmac?)
    func performFullAuthentication(
        challengeNonce: Data,
        challengeCiphertext: Data,
        sharedKey: Data,
        ekmKey: Data?,
        sessionID: String
    ) throws -> (plaintext: Data, ekmHmac: Data?) {
        setSession(sessionID)

        // Plugin #2: TLS Exporter（可選）
        var ekmHmac: Data?
        if let ekmKey = ekmKey {
            ekmHmac = try bindTlsExporter(ekmKey: ekmKey, sessionID: sessionID)
        }

        // Plugin #1: 解密挑戰
        let plaintext = try solveChallenge(
            key: sharedKey,
            nonce: challengeNonce,
            ciphertext: challengeCiphertext
        )

        logger.info("✅ 完整認證流程完成")
        return (plaintext, ekmHmac)
    }

    // MARK: - 輔助方法

    /// 重設所有 Plugin 狀態
    private func resetAllPluginStates() {
        for id in PluginID.allCases {
            pluginStatus[id] = .idle
        }
    }

    /// 取得所有 Plugin 的摘要資訊
    func statusSummary() -> String {
        var lines: [String] = ["SASS Plugins Status:"]
        for id in PluginID.allCases {
            let state = pluginStatus[id] ?? .idle
            let stateStr: String
            switch state {
            case .idle: stateStr = "⏸ 待命"
            case .running: stateStr = "⏳ 執行中"
            case .success: stateStr = "✅ 成功"
            case .failure(let reason): stateStr = "❌ 失敗：\(reason)"
            }
            lines.append("  \(id.rawValue): \(stateStr)")
        }
        if let session = currentSessionID {
            lines.append("  Session: \(session)")
        }
        return lines.joined(separator: "\n")
    }
}
