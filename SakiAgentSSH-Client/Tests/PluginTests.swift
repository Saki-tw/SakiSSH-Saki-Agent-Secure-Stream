// PluginTests.swift
// SakiAgentSSH Client — Plugin 單元測試
//
// 測試覆蓋：
//   C.1 ChaCha20Solver（4 個測試）
//   C.2 TlsExporterBinding（5 個測試）
//   C.4 AuditVerifier（5 個測試）
//   C.7 EnvInjectorClient（6 個測試）
//
// © 2026 Saki Studio. All rights reserved.

import XCTest
import CryptoKit
@testable import SakiAgentSSHClient

// MARK: - C.1 ChaCha20Solver Tests

final class ChaCha20SolverTests: XCTestCase {

    /// 正確金鑰/nonce/密文應成功解密
    func testSolveWithValidInput() throws {
        // 產生測試資料
        let key = SymmetricKey(size: .bits256)
        let keyData = key.withUnsafeBytes { Data($0) }
        let plaintext = Data((0..<64).map { _ in UInt8.random(in: 0...255) })
        let nonce = ChaChaPoly.Nonce()
        let nonceData = nonce.withUnsafeBytes { Data($0) }

        // 加密
        let sealedBox = try ChaChaPoly.seal(plaintext, using: key, nonce: nonce)
        // ChaChaPoly.SealedBox.combined = nonce + ciphertext + tag
        // 但 daemon 送出 ciphertext + tag（不含 nonce）
        let ciphertextWithTag = sealedBox.ciphertext + sealedBox.tag

        // 解密
        let result = try ChaCha20Solver.solve(
            key: keyData,
            nonce: nonceData,
            ciphertext: ciphertextWithTag
        )

        XCTAssertEqual(result.plaintext, plaintext, "解密後明文應與原始明文相同")
        XCTAssertFalse(result.isNearExpiry, "新產生的挑戰不應接近過期")
    }

    /// 金鑰長度不為 32 bytes 應拋出錯誤
    func testSolveRejectsInvalidKeyLength() {
        let shortKey = Data(repeating: 0xAB, count: 16)
        let nonce = Data(repeating: 0, count: 12)
        let ciphertext = Data(repeating: 0, count: 80)

        XCTAssertThrowsError(
            try ChaCha20Solver.solve(key: shortKey, nonce: nonce, ciphertext: ciphertext)
        ) { error in
            guard case ChaCha20Solver.SolverError.invalidKeyLength(let expected, let actual) = error else {
                XCTFail("預期 invalidKeyLength 錯誤，實際：\(error)")
                return
            }
            XCTAssertEqual(expected, 32)
            XCTAssertEqual(actual, 16)
        }
    }

    /// Nonce 長度不為 12 bytes 應拋出錯誤
    func testSolveRejectsInvalidNonceLength() {
        let key = Data(repeating: 0xAB, count: 32)
        let badNonce = Data(repeating: 0, count: 8)
        let ciphertext = Data(repeating: 0, count: 80)

        XCTAssertThrowsError(
            try ChaCha20Solver.solve(key: key, nonce: badNonce, ciphertext: ciphertext)
        ) { error in
            guard case ChaCha20Solver.SolverError.invalidNonceLength(_, _) = error else {
                XCTFail("預期 invalidNonceLength 錯誤")
                return
            }
        }
    }

    /// 空密文應拋出錯誤
    func testSolveRejectsEmptyCiphertext() {
        let key = Data(repeating: 0xAB, count: 32)
        let nonce = Data(repeating: 0, count: 12)

        XCTAssertThrowsError(
            try ChaCha20Solver.solve(key: key, nonce: nonce, ciphertext: Data())
        ) { error in
            guard case ChaCha20Solver.SolverError.emptyCiphertext = error else {
                XCTFail("預期 emptyCiphertext 錯誤")
                return
            }
        }
    }
}

// MARK: - C.2 TlsExporterBinding Tests

final class TlsExporterBindingTests: XCTestCase {

    /// Exporter Label 應符合 RFC 規格
    func testExporterLabelMatchesRFC() {
        XCTAssertEqual(
            TlsExporterBinding.exporterLabel,
            "EXPORTER-sakissh-chacha20-v15",
            "Label 必須與 RFC 定義一致"
        )
    }

    /// 匯出金鑰材料長度應為 44 bytes
    func testExportedKeyLengthIs44() {
        XCTAssertEqual(TlsExporterBinding.exportedKeyLength, 44)
        XCTAssertEqual(TlsExporterBinding.keyLength + TlsExporterBinding.nonceLength, 44)
    }

    /// splitEKM 應正確拆分 key（32B）和 nonce（12B）
    func testSplitEKMCorrectly() throws {
        let rawEKM = Data((0..<44).map { UInt8($0) })
        let (key, nonce) = try TlsExporterBinding.splitEKM(rawEKM)

        XCTAssertEqual(key.count, 32, "Key 應為 32 bytes")
        XCTAssertEqual(nonce.count, 12, "Nonce 應為 12 bytes")
        XCTAssertEqual(key, Data(rawEKM[0..<32]))
        XCTAssertEqual(nonce, Data(rawEKM[32..<44]))
    }

    /// splitEKM 長度不為 44 應拋出錯誤
    func testSplitEKMRejectsWrongLength() {
        let badData = Data(repeating: 0, count: 40)
        XCTAssertThrowsError(try TlsExporterBinding.splitEKM(badData))
    }

    /// computeClientEkmHmac 應具確定性（相同輸入相同輸出）
    func testHmacIsDeterministic() throws {
        let ekmKey = Data(repeating: 0xAA, count: 32)
        let sessionID = "test-session-uuid-12345678"

        let hmac1 = try TlsExporterBinding.computeClientEkmHmac(ekmKey: ekmKey, sessionID: sessionID)
        let hmac2 = try TlsExporterBinding.computeClientEkmHmac(ekmKey: ekmKey, sessionID: sessionID)

        XCTAssertEqual(hmac1, hmac2, "相同輸入應產生相同 HMAC")
        XCTAssertEqual(hmac1.count, 32, "HMAC-SHA256 輸出應為 32 bytes")
    }
}

// MARK: - C.4 AuditVerifier Tests

final class AuditVerifierTests: XCTestCase {

    /// Genesis block seed 應為 "SASS_GENESIS_BLOCK"
    func testGenesisBlockSeed() {
        XCTAssertEqual(AuditVerifier.genesisBlockSeed, "SASS_GENESIS_BLOCK")
    }

    /// Genesis hash 應為 SHA256("SASS_GENESIS_BLOCK")
    func testGenesisHashMatchesRFC() {
        let expectedHash = SHA256.hash(data: "SASS_GENESIS_BLOCK".data(using: .utf8)!)
        XCTAssertEqual(AuditVerifier.genesisHash, Data(expectedHash))
    }

    /// Genesis hash 應具確定性
    func testGenesisHashIsDeterministic() {
        let hash1 = AuditVerifier.genesisHash
        let hash2 = AuditVerifier.genesisHash
        XCTAssertEqual(hash1, hash2)
        XCTAssertEqual(hash1.count, 32, "SHA256 輸出應為 32 bytes")
    }

    /// 空稽核鏈應回傳失敗
    func testEmptyChainFails() {
        let privateKey = Curve25519.Signing.PrivateKey()
        let publicKey = privateKey.publicKey

        let result = AuditVerifier.verify(
            records: [],
            publicKey: publicKey.rawRepresentation
        )

        XCTAssertFalse(result.isValid)
        XCTAssertEqual(result.totalCount, 0)
        if case .emptyChain = result.failureReason {
            // 正確
        } else {
            XCTFail("預期 emptyChain 錯誤")
        }
    }

    /// 有效的單筆稽核記錄應驗證通過
    func testValidSingleRecordPasses() {
        let privateKey = Curve25519.Signing.PrivateKey()
        let publicKey = privateKey.publicKey

        // 建立有效記錄
        let timestamp = ISO8601DateFormatter().string(from: Date())
        let event = "{\"type\":\"auth_success\",\"agent\":\"test\"}"

        // 計算 chain_hash
        var hasher = SHA256()
        hasher.update(data: AuditVerifier.genesisHash)
        hasher.update(data: event.data(using: .utf8)!)
        hasher.update(data: timestamp.data(using: .utf8)!)
        let chainHash = Data(hasher.finalize())

        // 簽名
        let signature = try! privateKey.signature(for: chainHash)

        let record = AuditVerifier.AuditRecord(
            timestamp: timestamp,
            event: event,
            chainHash: chainHash,
            signature: Data(signature)
        )

        let result = AuditVerifier.verify(
            records: [record],
            publicKey: publicKey.rawRepresentation
        )

        XCTAssertTrue(result.isValid, "有效記錄應通過驗證")
        XCTAssertEqual(result.verifiedCount, 1)
    }
}

// MARK: - C.7 EnvInjectorClient Tests

final class EnvInjectorClientTests: XCTestCase {

    /// 揮發性快取基礎路徑應為 /tmp/sass_vol
    func testVolatileCacheBasePath() {
        XCTAssertEqual(EnvInjectorClient.volatileCacheBase, "/tmp/sass_vol")
    }

    /// prepareEnvironment 應回傳 6 個環境變數
    func testPrepareEnvironmentReturns6Vars() {
        let env = EnvInjectorClient.prepareEnvironment(for: "test-session")
        XCTAssertEqual(env.count, 6, "應有 6 個環境變數（npm, yarn, cargo_target, cargo_home, pip, tmpdir）")
    }

    /// npm_config_cache 應指向正確的揮發性路徑
    func testNpmCacheRedirect() {
        let env = EnvInjectorClient.prepareEnvironment(for: "session-123")
        XCTAssertEqual(env["npm_config_cache"], "/tmp/sass_vol/session-123/npm")
    }

    /// CARGO_TARGET_DIR 應指向正確的揮發性路徑
    func testCargoTargetDirRedirect() {
        let env = EnvInjectorClient.prepareEnvironment(for: "session-abc")
        XCTAssertEqual(env["CARGO_TARGET_DIR"], "/tmp/sass_vol/session-abc/ct")
    }

    /// TMPDIR 應對所有指令強制注入
    func testTmpdirAlwaysInjected() {
        let env = EnvInjectorClient.prepareEnvironment(for: "s1")
        XCTAssertNotNil(env["TMPDIR"], "TMPDIR 應始終存在")
        XCTAssertEqual(env["TMPDIR"], "/tmp/sass_vol/s1/tmp")
    }

    /// mergeEnvironment 中使用者自訂應覆蓋預設值
    func testMergeEnvironmentUserOverrides() {
        let userEnv = ["CARGO_TARGET_DIR": "/custom/path"]
        let merged = EnvInjectorClient.mergeEnvironment(userEnv: userEnv, session: "s1")

        XCTAssertEqual(merged["CARGO_TARGET_DIR"], "/custom/path", "使用者自訂應覆蓋預設")
        XCTAssertNotNil(merged["TMPDIR"], "非覆蓋的變數應保留")
    }
}
