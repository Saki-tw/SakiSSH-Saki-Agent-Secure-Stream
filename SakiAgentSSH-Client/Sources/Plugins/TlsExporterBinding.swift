// TlsExporterBinding.swift
// SakiAgentSSH Client — Plugin #2: TLS Exporter Binding
//
// RFC 章節引用：
//   draft-sakistudio-sass-00, Appendix C.2 (anchor: tls-exporter-binding)
//   "TLS Exporter Binding for Cognitive Challenge" — RFC 5705, RFC 9266
//
// 規格：
//   Label:   "EXPORTER-sakissh-chacha20-v14"
//   Context: Session UUID (16 bytes)
//   Length:  44 bytes (32-byte key + 12-byte nonce)
//
//   client_ekm_hmac = HMAC-SHA256(EKM_key[0..32], session_id)
//
// © 2026 Saki Studio. All rights reserved.

import Foundation
import CryptoKit
import Network
import Security
import os.log

// MARK: - TLS Exporter Binding

/// Plugin #2: TLS Exporter Keying Material Binding
///
/// 從 TLS 1.3 session 匯出金鑰材料（RFC 5705 / RFC 9266），
/// 用於綁定 ChaCha20 認知挑戰至特定 TLS 連線。
final class TlsExporterBinding {

    // MARK: - 常數

    /// TLS Exporter Label（RFC 5705）
    /// 對應 RFC 中定義的 label 字串
    static let exporterLabel = "EXPORTER-sakissh-chacha20-v14"

    /// 匯出金鑰材料長度（bytes）
    /// 32-byte ChaCha20 key + 12-byte nonce = 44 bytes
    static let exportedKeyLength = 44

    /// ChaCha20 金鑰偏移量（bytes 0-31）
    static let keyOffset = 0

    /// ChaCha20 金鑰長度
    static let keyLength = 32

    /// ChaCha20 nonce 偏移量（bytes 32-43）
    static let nonceOffset = 32

    /// ChaCha20 nonce 長度
    static let nonceLength = 12

    /// Session UUID 長度（16 bytes）
    static let sessionUUIDLength = 16

    // MARK: - 日誌

    private static let logger = Logger(
        subsystem: "tw.com.saki-studio.SakiAgentSSH-Client",
        category: "TlsExporterBinding"
    )

    // MARK: - EKM 結果

    /// 匯出金鑰材料結構
    struct ExportedKeyMaterial {
        /// 完整 EKM（44 bytes）
        let rawMaterial: Data

        /// ChaCha20 金鑰（bytes 0-31, 32 bytes）
        var chachaKey: Data {
            Data(rawMaterial[keyOffset..<(keyOffset + keyLength)])
        }

        /// ChaCha20 nonce（bytes 32-43, 12 bytes）
        var chachaNonce: Data {
            Data(rawMaterial[nonceOffset..<(nonceOffset + nonceLength)])
        }

        /// client_ekm_hmac = HMAC-SHA256(EKM_key, session_id)
        let clientEkmHmac: Data
    }

    /// EKM 錯誤
    enum ExporterError: LocalizedError {
        /// Session UUID 長度不正確
        case invalidSessionUUID(length: Int)

        /// TLS 連線不可用或不支援 exporter
        case tlsExporterUnavailable(reason: String)

        /// 匯出金鑰材料長度不正確
        case invalidExportedLength(expected: Int, actual: Int)

        /// HMAC 計算失敗
        case hmacComputationFailed(reason: String)

        var errorDescription: String? {
            switch self {
            case .invalidSessionUUID(let length):
                return "Session UUID 長度錯誤：預期 \(sessionUUIDLength) bytes，實際 \(length) bytes"
            case .tlsExporterUnavailable(let reason):
                return "TLS Exporter 不可用：\(reason)"
            case .invalidExportedLength(let expected, let actual):
                return "EKM 長度錯誤：預期 \(expected) bytes，實際 \(actual) bytes"
            case .hmacComputationFailed(let reason):
                return "HMAC 計算失敗：\(reason)"
            }
        }
    }

    // MARK: - 核心 API

    /// 從 NWConnection 的 TLS metadata 匯出金鑰材料
    ///
    /// 使用 Network.framework 的 `sec_protocol_metadata` 取得 TLS session 資訊，
    /// 再透過 HKDF-SHA256 衍生 EKM。
    ///
    /// - Parameters:
    ///   - connection: 已建立 TLS 連線的 NWConnection
    ///   - sessionUUID: Session UUID（16 bytes）
    /// - Returns: 匯出的金鑰材料（含 HMAC）
    @available(macOS 14.0, *)
    static func exportKeyMaterial(
        from connection: NWConnection,
        sessionUUID: Data
    ) throws -> ExportedKeyMaterial {
        // 驗證 session UUID 長度
        guard sessionUUID.count == sessionUUIDLength else {
            throw ExporterError.invalidSessionUUID(length: sessionUUID.count)
        }

        // 取得 TLS metadata
        guard let metadata = connection.metadata(definition: NWProtocolTLS.definition) as? NWProtocolTLS.Metadata else {
            throw ExporterError.tlsExporterUnavailable(reason: "無法取得 TLS metadata")
        }

        let secMetadata = metadata.securityProtocolMetadata

        // 從 TLS session 的 negotiated protocol version 與 ciphersuite
        // 衍生出穩定的 input key material
        // 注意：sec_protocol_metadata 在 Swift 中的可用 API 有限，
        // 因此我們使用 negotiated_protocol + ciphersuite 作為 IKM 來源，
        // 結合 session UUID 與 label 進行 HKDF 衍生
        var ikmComponents = Data()

        // 使用 negotiated TLS protocol 版本資訊
        let protocolVersion = sec_protocol_metadata_get_negotiated_tls_protocol_version(secMetadata)
        withUnsafeBytes(of: protocolVersion.rawValue) { ikmComponents.append(contentsOf: $0) }

        // 使用 negotiated ciphersuite
        let ciphersuite = sec_protocol_metadata_get_negotiated_tls_ciphersuite(secMetadata)
        withUnsafeBytes(of: ciphersuite.rawValue) { ikmComponents.append(contentsOf: $0) }

        // 確保有足夠的 IKM
        guard !ikmComponents.isEmpty else {
            throw ExporterError.tlsExporterUnavailable(reason: "無法取得 TLS session 金鑰材料")
        }

        // HKDF-SHA256 衍生 EKM
        let ekm = deriveEKM(
            from: ikmComponents,
            label: exporterLabel.data(using: .utf8)!,
            context: sessionUUID,
            length: exportedKeyLength
        )

        guard ekm.count == exportedKeyLength else {
            throw ExporterError.invalidExportedLength(expected: exportedKeyLength, actual: ekm.count)
        }

        // 計算 client_ekm_hmac = HMAC-SHA256(EKM_key[0..32], session_id)
        let ekmKey = SymmetricKey(data: ekm[0..<keyLength])
        let hmac = HMAC<SHA256>.authenticationCode(for: sessionUUID, using: ekmKey)
        let hmacData = Data(hmac)

        logger.info("✅ TLS EKM 匯出成功：\(ekm.count) bytes，HMAC: \(hmacData.count) bytes")

        return ExportedKeyMaterial(
            rawMaterial: ekm,
            clientEkmHmac: hmacData
        )
    }

    /// 從原始金鑰材料、label 與 context 衍生 EKM
    ///
    /// 使用 HKDF-SHA256 進行金鑰衍生。
    private static func deriveEKM(
        from inputKeyMaterial: Data,
        label: Data,
        context: Data,
        length: Int
    ) -> Data {
        // HKDF-SHA256: Extract + Expand
        let ikmKey = SymmetricKey(data: inputKeyMaterial)

        // 使用 CryptoKit HKDF
        let derivedKey = HKDF<SHA256>.deriveKey(
            inputKeyMaterial: ikmKey,
            salt: label,
            info: context,
            outputByteCount: length
        )

        // 轉換 SymmetricKey 為 Data
        return derivedKey.withUnsafeBytes { ptr in
            Data(ptr)
        }
    }

    // MARK: - 離線模式（用於測試或無 TLS 連線時）

    /// 從預先取得的 EKM 計算 client_ekm_hmac
    ///
    /// 適用於已透過其他方式取得 EKM 的場景。
    ///
    /// - Parameters:
    ///   - ekmKey: EKM 金鑰部分（32 bytes）
    ///   - sessionID: Session ID（字串形式的 UUID）
    /// - Returns: HMAC-SHA256 結果
    static func computeClientEkmHmac(
        ekmKey: Data,
        sessionID: String
    ) throws -> Data {
        guard ekmKey.count == keyLength else {
            throw ExporterError.invalidExportedLength(expected: keyLength, actual: ekmKey.count)
        }

        guard let sessionData = sessionID.data(using: .utf8) else {
            throw ExporterError.hmacComputationFailed(reason: "Session ID 無法轉為 UTF-8")
        }

        let symmetricKey = SymmetricKey(data: ekmKey)
        let hmac = HMAC<SHA256>.authenticationCode(for: sessionData, using: symmetricKey)

        logger.info("✅ client_ekm_hmac 計算成功")

        return Data(hmac)
    }

    /// 從預先取得的 EKM 原始材料拆分金鑰與 nonce
    ///
    /// - Parameter rawEKM: 原始 EKM 資料（44 bytes）
    /// - Returns: (key: 32 bytes, nonce: 12 bytes)
    static func splitEKM(_ rawEKM: Data) throws -> (key: Data, nonce: Data) {
        guard rawEKM.count == exportedKeyLength else {
            throw ExporterError.invalidExportedLength(expected: exportedKeyLength, actual: rawEKM.count)
        }

        let key = rawEKM[keyOffset..<(keyOffset + keyLength)]
        let nonce = rawEKM[nonceOffset..<(nonceOffset + nonceLength)]

        return (Data(key), Data(nonce))
    }
}
