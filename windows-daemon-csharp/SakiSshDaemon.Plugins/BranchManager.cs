// =============================================================================
// SakiSshDaemon.Plugins — BranchManager.cs
// SASS Plugin #6: Transparent Branching via Symlink Tree
//
// 對應 Rust: branch_mgr.rs (BranchMgr)
// RFC 參考: draft-sakistudio-sass-00 Appendix C.6 (anchor: symlink-tree)
//
// 分支結構:
// /tmp/sass_branches/{session_id}/
// +-- src/         <- real directory (created)
// |   +-- main.rs  <- symlink -> /orig/src/main.rs
// |   +-- lib.rs   <- symlink -> /orig/src/lib.rs
// +-- Cargo.toml   <- symlink -> /orig/Cargo.toml
//
// 排除目錄: target/, .git/, node_modules/
//
// Windows 差異:
// - 路徑: %TEMP%\sass_branches\{session_id}\（取代 /tmp/sass_branches/）
// - 使用 NTFS Junction Point（取代 Unix symlink，避免 SeCreateSymbolicLinkPrivilege 權限問題）
// - Junction Point 僅適用於目錄；檔案使用 hardlink 或複製
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Logging;

namespace SakiSshDaemon.Plugins
{
    /// <summary>
    /// SASS Plugin #6: Transparent Branching via Symlink Tree。
    /// <para>
    /// RFC draft-sakistudio-sass-00 §C.6: 在 Agent 執行修改前，
    /// 將目標目錄動態隔離。Agent 的所有寫入都會進入容量極小的高層分支，
    /// 而不會污染底層系統。
    /// </para>
    /// <para>
    /// Windows 差異: 使用 NTFS Junction Point 取代 Unix symlink。
    /// Junction Point 不需要 SeCreateSymbolicLinkPrivilege 權限（Windows 10 1703+），
    /// 但僅適用於目錄。檔案層級使用 hardlink。
    /// </para>
    /// </summary>
    public sealed class BranchManager : IPlugin
    {
        /// <summary>排除的目錄名稱 — 對齊 Rust build_symlink_tree 的排除列表</summary>
        private static readonly string[] ExcludedDirectories = { "target", ".git", "node_modules" };

        private readonly ILogger<BranchManager> _logger;
        private bool _disposed;

        public BranchManager(ILogger<BranchManager> logger)
        {
            _logger = logger ?? throw new ArgumentNullException(nameof(logger));
        }

        /// <inheritdoc />
        public string Name => "Transparent Branching (Branch Manager)";

        /// <inheritdoc />
        public string RfcSection => "Appendix C.6 (symlink-tree)";

        /// <inheritdoc />
        public bool IsHealthy => !_disposed;

        /// <inheritdoc />
        public Task<bool> InitializeAsync(CancellationToken cancellationToken = default)
        {
            _logger.LogInformation(
                "Plugin #6 ({Name}) 初始化完成 — BranchRoot={BranchRoot}",
                Name, GetBranchRoot());
            return Task.FromResult(true);
        }

        /// <summary>
        /// 為特定的執行階段建立儲存層微型分支。
        /// <para>
        /// 對齊 Rust: BranchMgr::create_micro_branch()
        /// Windows: 使用 %TEMP%\sass_branches\{session_id}\ 作為分支目錄。
        /// </para>
        /// </summary>
        /// <param name="sessionId">Session 識別碼</param>
        /// <param name="targetDir">目標目錄的絕對路徑</param>
        /// <returns>分支目錄路徑，null 表示建立失敗</returns>
        public string? CreateMicroBranch(string sessionId, string targetDir)
        {
            if (!Directory.Exists(targetDir))
            {
                _logger.LogWarning(
                    "BranchMgr: 目標目錄 {TargetDir} 不存在。跳過分支建立。",
                    targetDir);
                return null;
            }

            // Windows 路徑 — 對齊 Rust /tmp/sass_branches/{session_id}
            // Windows 使用 %TEMP%\sass_branches\{session_id}\
            string branchDir = Path.Combine(GetBranchRoot(), sessionId);

            try
            {
                Directory.CreateDirectory(branchDir);
            }
            catch (Exception ex)
            {
                _logger.LogWarning(ex, "BranchMgr: 無法建立分支目錄");
                return null;
            }

            _logger.LogInformation(
                "BranchMgr: Created micro overlay branch for Session {SessionId} targeting {TargetDir}",
                sessionId, targetDir);

            // 建立 Symlink Tree — 對齊 Rust build_symlink_tree
            // Windows 差異: 目錄使用 Junction Point，檔案使用 hardlink
            try
            {
                BuildLinkTree(targetDir, branchDir);
            }
            catch (Exception ex)
            {
                _logger.LogWarning(ex, "BranchMgr: 無法建立 link tree");
            }

            return branchDir;
        }

        /// <summary>
        /// 在 Human Review 失敗時捨棄分支。
        /// <para>對齊 Rust: BranchMgr::drop_branch()</para>
        /// </summary>
        public void DropBranch(string sessionId)
        {
            string branchDir = Path.Combine(GetBranchRoot(), sessionId);

            if (Directory.Exists(branchDir))
            {
                _logger.LogInformation(
                    "BranchMgr: Dropping branch for Session {SessionId}",
                    sessionId);
                Directory.Delete(branchDir, recursive: true);
            }
        }

        /// <summary>
        /// 在 Human Review 成功時合併分支。
        /// <para>對齊 Rust: BranchMgr::merge_branch() — 目前為模擬實作</para>
        /// </summary>
        public void MergeBranch(string sessionId, string targetDir)
        {
            _logger.LogInformation(
                "BranchMgr: Merging branch for Session {SessionId} (Simulated)",
                sessionId);
            // 在真實實作中，此處應將分支內的差異複製回 targetDir
            DropBranch(sessionId);
        }

        /// <summary>
        /// 遞迴建立 Link Tree。
        /// <para>
        /// 對齊 Rust: BranchMgr::build_symlink_tree()
        /// Windows 差異:
        /// - 目錄: 建立實體目錄並遞迴（Junction Point 會導致循環參照問題）
        /// - 檔案: 使用 File.CreateSymbolicLink()（.NET 6+）
        ///         若失敗（權限不足），fallback 到 hardlink
        ///         若仍失敗，最終 fallback 到檔案複製
        /// </para>
        /// </summary>
        private void BuildLinkTree(string srcDir, string dstDir)
        {
            foreach (var entry in new DirectoryInfo(srcDir).EnumerateFileSystemInfos())
            {
                string destPath = Path.Combine(dstDir, entry.Name);

                if (entry is DirectoryInfo dirInfo)
                {
                    // 排除重量級目錄 — 對齊 Rust: target, .git, node_modules
                    if (Array.Exists(ExcludedDirectories,
                        d => d.Equals(dirInfo.Name, StringComparison.OrdinalIgnoreCase)))
                    {
                        continue;
                    }

                    Directory.CreateDirectory(destPath);
                    BuildLinkTree(dirInfo.FullName, destPath);
                }
                else if (entry is FileInfo fileInfo)
                {
                    CreateFileLink(fileInfo.FullName, destPath);
                }
            }
        }

        /// <summary>
        /// 建立檔案連結（symlink → hardlink → copy 三級降級）。
        /// <para>
        /// Windows 差異: symlink 需要 SeCreateSymbolicLinkPrivilege，
        /// 在非 Developer Mode 的 Windows 上可能失敗。
        /// 降級策略確保零權限也能正常運作。
        /// </para>
        /// </summary>
        private void CreateFileLink(string sourcePath, string destPath)
        {
            // 優先嘗試 symlink (.NET 7+)
            try
            {
                File.CreateSymbolicLink(destPath, sourcePath);
                return;
            }
            catch
            {
                // 權限不足，嘗試 hardlink
            }

            // Fallback: 嘗試 hardlink
            try
            {
                if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
                {
                    // Windows CreateHardLink API
                    if (CreateHardLinkW(destPath, sourcePath, IntPtr.Zero))
                        return;
                }
            }
            catch
            {
                // Hardlink 也失敗
            }

            // 最終 fallback: 檔案複製
            try
            {
                File.Copy(sourcePath, destPath, overwrite: true);
            }
            catch (Exception ex)
            {
                _logger.LogWarning(ex,
                    "BranchMgr: 無法建立連結或複製 {Source} → {Dest}",
                    sourcePath, destPath);
            }
        }

        /// <summary>
        /// 取得分支根目錄。
        /// <para>
        /// Windows: %TEMP%\sass_branches\
        /// 對齊 Rust: /tmp/sass_branches/
        /// </para>
        /// </summary>
        private static string GetBranchRoot()
        {
            return Path.Combine(Path.GetTempPath(), "sass_branches");
        }

        // =====================================================================
        // Windows API P/Invoke — NTFS Hardlink
        // =====================================================================

        /// <summary>Windows CreateHardLink API for file hardlinks</summary>
        [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool CreateHardLinkW(
            string lpFileName,
            string lpExistingFileName,
            IntPtr lpSecurityAttributes);

        /// <inheritdoc />
        public void Dispose()
        {
            _disposed = true;
        }
    }
}
