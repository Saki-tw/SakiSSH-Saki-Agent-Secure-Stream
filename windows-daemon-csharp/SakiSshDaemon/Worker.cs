// =============================================================================
// SakiSshDaemon — Worker.cs
// SASS Windows Service BackgroundService 入口
//
// 負責初始化所有 7 個 Plugins 與 Rust FFI Bridge，
// 管理 Windows Service 生命週期。
//
// 參考: draft-sakistudio-sass-00 Appendix C
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using SakiSshDaemon.Interop;
using SakiSshDaemon.Plugins;

namespace SakiSshDaemon
{
    /// <summary>
    /// SASS Daemon 的 BackgroundService 主體。
    /// <para>
    /// 管理 7 個 Plugins 的生命週期，以及 Rust FFI Bridge 的初始化與清理。
    /// 作為 Windows Service 時，由 SCM 控制啟動/停止。
    /// </para>
    /// </summary>
    public sealed class Worker : BackgroundService
    {
        private readonly ILogger<Worker> _logger;
        private readonly RustBridge _rustBridge;
        private readonly IEnumerable<IPlugin> _plugins;

        public Worker(
            ILogger<Worker> logger,
            RustBridge rustBridge,
            IEnumerable<IPlugin> plugins)
        {
            _logger = logger ?? throw new ArgumentNullException(nameof(logger));
            _rustBridge = rustBridge ?? throw new ArgumentNullException(nameof(rustBridge));
            _plugins = plugins ?? throw new ArgumentNullException(nameof(plugins));
        }

        /// <summary>
        /// Service 啟動邏輯。
        /// </summary>
        public override async Task StartAsync(CancellationToken cancellationToken)
        {
            _logger.LogInformation("=== SASS Windows Daemon 啟動中 ===");
            _logger.LogInformation("OS: {OS}", Environment.OSVersion);
            _logger.LogInformation(".NET: {Runtime}", System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription);

            // 1. 初始化 Rust FFI Bridge
            _logger.LogInformation("初始化 Rust FFI Bridge...");
            _rustBridge.Initialize();
            if (_rustBridge.IsNativeAvailable)
            {
                _logger.LogInformation("Rust core 可用 — 使用原生加密");
            }
            else
            {
                _logger.LogWarning("Rust core 不可用 — 使用 C# fallback 模式");
            }

            // 2. 初始化所有 Plugins
            int pluginCount = 0;
            int failCount = 0;
            foreach (var plugin in _plugins)
            {
                try
                {
                    bool success = await plugin.InitializeAsync(cancellationToken);
                    if (success)
                    {
                        pluginCount++;
                        _logger.LogInformation(
                            "  ✓ Plugin 初始化成功: {Name} ({Section})",
                            plugin.Name, plugin.RfcSection);
                    }
                    else
                    {
                        failCount++;
                        _logger.LogWarning(
                            "  ✗ Plugin 初始化失敗: {Name}",
                            plugin.Name);
                    }
                }
                catch (Exception ex)
                {
                    failCount++;
                    _logger.LogError(ex,
                        "  ✗ Plugin 初始化異常: {Name}",
                        plugin.Name);
                }
            }

            _logger.LogInformation(
                "=== SASS Daemon 啟動完成 — {Success}/{Total} Plugins 就緒 ===",
                pluginCount, pluginCount + failCount);

            await base.StartAsync(cancellationToken);
        }

        /// <summary>
        /// Service 主要執行迴圈。
        /// 維持 Service 運行，並定期進行健康檢查。
        /// </summary>
        protected override async Task ExecuteAsync(CancellationToken stoppingToken)
        {
            _logger.LogInformation("SASS Daemon 主迴圈已啟動");

            while (!stoppingToken.IsCancellationRequested)
            {
                try
                {
                    // 定期健康檢查（每 60 秒）
                    await Task.Delay(TimeSpan.FromSeconds(60), stoppingToken);

                    // 檢查所有 Plugins 健康狀態
                    var unhealthy = _plugins.Where(p => !p.IsHealthy).ToList();
                    if (unhealthy.Count > 0)
                    {
                        foreach (var p in unhealthy)
                        {
                            _logger.LogWarning(
                                "Plugin 健康檢查失敗: {Name}",
                                p.Name);
                        }
                    }
                }
                catch (OperationCanceledException)
                {
                    break;
                }
                catch (Exception ex)
                {
                    _logger.LogError(ex, "SASS Daemon 主迴圈異常");
                }
            }
        }

        /// <summary>
        /// Service 停止邏輯。
        /// </summary>
        public override async Task StopAsync(CancellationToken cancellationToken)
        {
            _logger.LogInformation("=== SASS Daemon 停止中 ===");

            // 釋放所有 Plugins
            foreach (var plugin in _plugins)
            {
                try
                {
                    plugin.Dispose();
                    _logger.LogInformation("  Plugin 已釋放: {Name}", plugin.Name);
                }
                catch (Exception ex)
                {
                    _logger.LogError(ex,
                        "  Plugin 釋放失敗: {Name}",
                        plugin.Name);
                }
            }

            // 釋放 Rust FFI Bridge
            _rustBridge.Dispose();

            _logger.LogInformation("=== SASS Daemon 已停止 ===");
            await base.StopAsync(cancellationToken);
        }
    }
}
