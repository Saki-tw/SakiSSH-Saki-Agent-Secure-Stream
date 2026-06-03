// =============================================================================
// SakiSshDaemon.Plugins — IPlugin.cs
// SASS (Saki Agent Secure Stream) — Plugin 介面定義
//
// 所有 7 個 Plugins 必須實作此介面。
// 參考: draft-sakistudio-sass-00 Appendix C
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;
using System.Threading;
using System.Threading.Tasks;

namespace SakiSshDaemon.Plugins
{
    /// <summary>
    /// SASS Plugin 統一介面。
    /// <para>
    /// 對應 RFC draft-sakistudio-sass-00 Appendix C 定義的 7 個 Plugins：
    /// <list type="number">
    ///   <item><description>ChaCha20-Poly1305 Cognitive Challenge (§C.1)</description></item>
    ///   <item><description>TLS Exporter Binding (§C.2)</description></item>
    ///   <item><description>Zero-Allocation Tarpit Buffer (§C.3)</description></item>
    ///   <item><description>ED25519 Hash Chain Audit Log (§C.4)</description></item>
    ///   <item><description>Vi Swap ANSI Escape (§C.5)</description></item>
    ///   <item><description>Transparent Branching via Symlink Tree (§C.6)</description></item>
    ///   <item><description>Volatile Cache Redirection (§C.7)</description></item>
    /// </list>
    /// </para>
    /// </summary>
    public interface IPlugin : IDisposable
    {
        /// <summary>
        /// Plugin 名稱（用於日誌與識別）。
        /// </summary>
        string Name { get; }

        /// <summary>
        /// 對應的 RFC 章節編號。
        /// </summary>
        string RfcSection { get; }

        /// <summary>
        /// 初始化 Plugin。
        /// 在 Windows Service 啟動時由 Worker 調用。
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>是否初始化成功</returns>
        Task<bool> InitializeAsync(CancellationToken cancellationToken = default);

        /// <summary>
        /// 檢查 Plugin 是否處於健康狀態。
        /// </summary>
        bool IsHealthy { get; }
    }
}
