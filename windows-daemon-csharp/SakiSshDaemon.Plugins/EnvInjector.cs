// =============================================================================
// SakiSshDaemon.Plugins — EnvInjector.cs
// SASS Plugin #7: Volatile Cache Redirection (Environment Variable Injection)
//
// 對應 Rust: env_injector.rs (EnvInjector::inject_volume_reduction_env)
// RFC 參考: draft-sakistudio-sass-00 Appendix C.7 (anchor: volatile-cache)
//
// 環境變數重導向表:
// +-----------------+---------------------+---------------------------+
// | Detected Tool   | Environment Variable | Redirect Target           |
// +-----------------+---------------------+---------------------------+
// | npm/yarn/pnpm   | npm_config_cache    | %TEMP%\sass_vol\npm       |
// | npm/yarn/pnpm   | YARN_CACHE_FOLDER   | %TEMP%\sass_vol\yarn      |
// | cargo/rustc     | CARGO_TARGET_DIR    | %TEMP%\sass_vol\ct        |
// | cargo/rustc     | CARGO_HOME          | %TEMP%\sass_vol\ch        |
// | pip             | PIP_CACHE_DIR       | %TEMP%\sass_vol\pip       |
// | (all commands)  | TMPDIR              | %TEMP%\sass_vol\tmp       |
// +-----------------+---------------------+---------------------------+
//
// Windows 差異:
// - 路徑前綴: %TEMP%\sass_vol\ (取代 /tmp/sass_volatile_cache/)
// - 額外注入 TEMP 和 TMP（Windows 專用臨時目錄變數）
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Logging;

namespace SakiSshDaemon.Plugins
{
    /// <summary>
    /// SASS Plugin #7: Volatile Cache Redirection (EnvInjector)。
    /// <para>
    /// RFC draft-sakistudio-sass-00 §C.7: 將 Agent 的垃圾與快取 I/O
    /// （如 npm cache, cargo build artifacts）動態卸載至臨時目錄，
    /// 避免污染儲存層的動態分支。
    /// </para>
    /// <para>
    /// Windows 差異: 使用 %TEMP%\sass_vol\ 取代 /tmp/sass_volatile_cache/。
    /// 額外注入 TEMP/TMP 環境變數（Windows 專用）。
    /// </para>
    /// </summary>
    public sealed class EnvInjector : IPlugin
    {
        private readonly ILogger<EnvInjector> _logger;
        private bool _disposed;

        /// <summary>
        /// Volatile cache 根路徑。
        /// <para>
        /// Windows: %TEMP%\sass_vol\
        /// 對齊 Rust: /tmp/sass_volatile_cache/
        /// </para>
        /// </summary>
        private static string VolatileRoot => Path.Combine(Path.GetTempPath(), "sass_vol");

        public EnvInjector(ILogger<EnvInjector> logger)
        {
            _logger = logger ?? throw new ArgumentNullException(nameof(logger));
        }

        /// <inheritdoc />
        public string Name => "Volatile Cache Redirection (EnvInjector)";

        /// <inheritdoc />
        public string RfcSection => "Appendix C.7 (volatile-cache)";

        /// <inheritdoc />
        public bool IsHealthy => !_disposed;

        /// <inheritdoc />
        public Task<bool> InitializeAsync(CancellationToken cancellationToken = default)
        {
            // 確保 volatile cache 根目錄存在
            try
            {
                Directory.CreateDirectory(VolatileRoot);
            }
            catch (Exception ex)
            {
                _logger.LogWarning(ex,
                    "無法建立 volatile cache 根目錄: {Path}", VolatileRoot);
            }

            _logger.LogInformation(
                "Plugin #7 ({Name}) 初始化完成 — VolatileRoot={Root}",
                Name, VolatileRoot);
            return Task.FromResult(true);
        }

        /// <summary>
        /// 分析指令意圖，並注入對應的減量環境變數。
        /// <para>
        /// 對齊 Rust: EnvInjector::inject_volume_reduction_env()
        /// 根據指令內容偵測 6 個工具，注入對應的環境變數重導向。
        /// </para>
        /// </summary>
        /// <param name="command">要執行的指令</param>
        /// <param name="env">現有的環境變數字典（會被修改並回傳）</param>
        /// <returns>注入減量環境變數後的字典</returns>
        public Dictionary<string, string> InjectVolumeReductionEnv(
            string command,
            Dictionary<string, string> env)
        {
            string cmdLower = command.ToLowerInvariant();

            // 判斷是否為高 I/O 消耗的快取/建置指令 — 對齊 Rust cmd_lower.contains()

            if (cmdLower.Contains("npm") ||
                cmdLower.Contains("yarn") ||
                cmdLower.Contains("pnpm"))
            {
                _logger.LogInformation(
                    "EnvInjector: 偵測到 Node.js 套件管理器。重導向快取至揮發性記憶體。");

                // npm_config_cache → %TEMP%\sass_vol\npm
                // 對齊 RFC §C.7: npm/yarn/pnpm → npm_config_cache → /tmp/sass_vol/npm
                env["npm_config_cache"] = Path.Combine(VolatileRoot, "npm");

                // YARN_CACHE_FOLDER → %TEMP%\sass_vol\yarn
                // 對齊 RFC §C.7: npm/yarn/pnpm → YARN_CACHE_FOLDER → /tmp/sass_vol/yarn
                env["YARN_CACHE_FOLDER"] = Path.Combine(VolatileRoot, "yarn");
            }
            else if (cmdLower.Contains("cargo") ||
                     cmdLower.Contains("rustc"))
            {
                _logger.LogInformation(
                    "EnvInjector: 偵測到 Rust 建置系統。重導向 target 目錄至揮發性記憶體。");

                // CARGO_TARGET_DIR → %TEMP%\sass_vol\ct
                // 對齊 RFC §C.7: cargo/rustc → CARGO_TARGET_DIR → /tmp/sass_vol/ct
                env["CARGO_TARGET_DIR"] = Path.Combine(VolatileRoot, "ct");

                // CARGO_HOME → %TEMP%\sass_vol\ch
                // 對齊 RFC §C.7: cargo/rustc → CARGO_HOME → /tmp/sass_vol/ch
                env["CARGO_HOME"] = Path.Combine(VolatileRoot, "ch");
            }
            else if (cmdLower.Contains("pip"))
            {
                _logger.LogInformation(
                    "EnvInjector: 偵測到 Python pip。重導向快取至揮發性記憶體。");

                // PIP_CACHE_DIR → %TEMP%\sass_vol\pip
                // 對齊 RFC §C.7: pip → PIP_CACHE_DIR → /tmp/sass_vol/pip
                env["PIP_CACHE_DIR"] = Path.Combine(VolatileRoot, "pip");
            }

            // 強制全局 TMPDIR 到隔離區域
            // 對齊 RFC §C.7: (all commands) → TMPDIR → /tmp/sass_vol/tmp
            // 對齊 Rust: env.insert("TMPDIR", "/tmp/sass_volatile_cache/tmp")
            string tmpPath = Path.Combine(VolatileRoot, "tmp");
            env["TMPDIR"] = tmpPath;

            // Windows 專用: 額外注入 TEMP 和 TMP 環境變數
            // Windows 應用程式通常使用 TEMP/TMP 而非 TMPDIR
            env["TEMP"] = tmpPath;
            env["TMP"] = tmpPath;

            return env;
        }

        /// <summary>
        /// 確保所有揮發性快取子目錄已建立。
        /// 在 Session 啟動時呼叫，避免指令執行時因目錄不存在而失敗。
        /// </summary>
        public void EnsureVolatileDirectories()
        {
            string[] subdirs = { "npm", "yarn", "ct", "ch", "pip", "tmp" };
            foreach (string sub in subdirs)
            {
                string path = Path.Combine(VolatileRoot, sub);
                try
                {
                    Directory.CreateDirectory(path);
                }
                catch (Exception ex)
                {
                    _logger.LogWarning(ex,
                        "無法建立揮發性快取子目錄: {Path}", path);
                }
            }
        }

        /// <inheritdoc />
        public void Dispose()
        {
            _disposed = true;
        }
    }
}
