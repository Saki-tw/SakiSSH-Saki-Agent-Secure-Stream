// EnvInjectorClient.swift
// SakiAgentSSH Client — Plugin #7: Volatile Cache Redirection 客戶端
//
// RFC 章節引用：
//   draft-sakistudio-sass-00, Appendix C.6 (anchor: volatile-cache)
//   "Volatile Cache Redirection"
//
// 環境變數重導向表：
//   | 偵測工具         | 環境變數             | 重導向目標           |
//   |-----------------|---------------------|---------------------|
//   | npm/yarn/pnpm   | npm_config_cache    | /tmp/sass_vol/npm   |
//   | npm/yarn/pnpm   | YARN_CACHE_FOLDER   | /tmp/sass_vol/yarn  |
//   | cargo/rustc     | CARGO_TARGET_DIR    | /tmp/sass_vol/ct    |
//   | cargo/rustc     | CARGO_HOME          | /tmp/sass_vol/ch    |
//   | pip             | PIP_CACHE_DIR       | /tmp/sass_vol/pip   |
//   | (all commands)  | TMPDIR              | /tmp/sass_vol/tmp   |
//
// © 2026 Saki Studio. All rights reserved.

import Foundation
import os.log

// MARK: - 客戶端環境變數注入器

/// Plugin #7: Volatile Cache Redirection Client
///
/// 為遠端執行的指令準備環境變數，將各工具的快取目錄
/// 重導向至 `/tmp/sass_vol/` 下的揮發性儲存空間。
///
/// 此設計確保：
/// 1. 不同 session 的快取互不干擾
/// 2. Session 結束後快取自動清除（tmpfs）
/// 3. 避免寫入宿主機的永久儲存空間
final class EnvInjectorClient {

    // MARK: - 常數

    /// 揮發性快取基礎路徑
    static let volatileCacheBase = "/tmp/sass_vol"

    // MARK: - 日誌

    private static let logger = Logger(
        subsystem: "tw.com.saki-studio.SakiAgentSSH-Client",
        category: "EnvInjectorClient"
    )

    // MARK: - 快取目錄定義

    /// 快取重導向項目
    struct CacheRedirect {
        /// 偵測的工具名稱
        let toolName: String

        /// 環境變數名稱
        let envVar: String

        /// 重導向子目錄（相對於 volatileCacheBase）
        let subdirectory: String

        /// 完整路徑（含 session 隔離）
        func fullPath(session: String) -> String {
            "\(EnvInjectorClient.volatileCacheBase)/\(session)/\(subdirectory)"
        }

        /// 不含 session 隔離的通用路徑
        var genericPath: String {
            "\(EnvInjectorClient.volatileCacheBase)/\(subdirectory)"
        }
    }

    /// RFC 定義的所有快取重導向規則
    static let cacheRedirects: [CacheRedirect] = [
        // npm/yarn/pnpm
        CacheRedirect(toolName: "npm/yarn/pnpm", envVar: "npm_config_cache", subdirectory: "npm"),
        CacheRedirect(toolName: "npm/yarn/pnpm", envVar: "YARN_CACHE_FOLDER", subdirectory: "yarn"),

        // cargo/rustc
        CacheRedirect(toolName: "cargo/rustc", envVar: "CARGO_TARGET_DIR", subdirectory: "ct"),
        CacheRedirect(toolName: "cargo/rustc", envVar: "CARGO_HOME", subdirectory: "ch"),

        // pip
        CacheRedirect(toolName: "pip", envVar: "PIP_CACHE_DIR", subdirectory: "pip"),

        // 通用 TMPDIR
        CacheRedirect(toolName: "(all)", envVar: "TMPDIR", subdirectory: "tmp"),
    ]

    // MARK: - 核心 API

    /// 為指定 session 準備完整的環境變數字典
    ///
    /// 產生所有快取重導向的環境變數，用於附加到 ExecuteRequest 的 env 欄位。
    ///
    /// - Parameter session: Session ID（用於目錄隔離）
    /// - Returns: 環境變數字典 `[String: String]`
    static func prepareEnvironment(for session: String) -> [String: String] {
        var env: [String: String] = [:]

        for redirect in cacheRedirects {
            let path = redirect.fullPath(session: session)
            env[redirect.envVar] = path
        }

        logger.info(
            "✅ 環境變數準備完成：\(env.count) 個變數，session: \(session, privacy: .public)"
        )

        return env
    }

    /// 為指定 session 準備環境變數（不含 session 目錄隔離）
    ///
    /// 使用通用路徑（不依賴 session ID），適用於共用快取場景。
    ///
    /// - Returns: 環境變數字典 `[String: String]`
    static func prepareGenericEnvironment() -> [String: String] {
        var env: [String: String] = [:]

        for redirect in cacheRedirects {
            env[redirect.envVar] = redirect.genericPath
        }

        logger.info("✅ 通用環境變數準備完成：\(env.count) 個變數")

        return env
    }

    /// 取得特定工具的環境變數
    ///
    /// - Parameter toolName: 工具名稱（如 "npm", "cargo", "pip"）
    /// - Returns: 匹配的環境變數字典
    static func environmentForTool(_ toolName: String, session: String) -> [String: String] {
        var env: [String: String] = [:]
        let lowered = toolName.lowercased()

        for redirect in cacheRedirects {
            if redirect.toolName.lowercased().contains(lowered) || redirect.toolName == "(all)" {
                env[redirect.envVar] = redirect.fullPath(session: session)
            }
        }

        return env
    }

    /// 合併環境變數（使用者自訂 + 快取重導向）
    ///
    /// 使用者自訂的環境變數優先級高於快取重導向。
    ///
    /// - Parameters:
    ///   - userEnv: 使用者自訂的環境變數
    ///   - session: Session ID
    /// - Returns: 合併後的環境變數
    static func mergeEnvironment(
        userEnv: [String: String],
        session: String
    ) -> [String: String] {
        var merged = prepareEnvironment(for: session)

        // 使用者自訂覆蓋預設值
        for (key, value) in userEnv {
            merged[key] = value
        }

        return merged
    }

    // MARK: - 目錄管理

    /// 列出某 session 的所有快取目錄路徑
    ///
    /// - Parameter session: Session ID
    /// - Returns: 快取目錄路徑陣列
    static func listCacheDirectories(for session: String) -> [String] {
        cacheRedirects.map { $0.fullPath(session: session) }
    }

    /// 取得清理指令（用於 session 結束時）
    ///
    /// - Parameter session: Session ID
    /// - Returns: rm -rf 指令字串
    static func cleanupCommand(for session: String) -> String {
        "rm -rf \(volatileCacheBase)/\(session)"
    }
}
