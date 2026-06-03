// =============================================================================
// SakiSshDaemon — Program.cs
// SASS Windows C# Plugins Daemon 入口點
//
// 使用 .NET 8 Worker Service 模板，作為 Windows Service 運行。
// 核心加密走 Rust FFI，Plugin 邏輯由 C# 管理。
//
// 參考: draft-sakistudio-sass-00 Appendix C
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using Serilog;
using SakiSshDaemon.Interop;
using SakiSshDaemon.Plugins;

namespace SakiSshDaemon
{
    public class Program
    {
        public static void Main(string[] args)
        {
            // Serilog 配置
            Log.Logger = new Serilog.LoggerConfiguration()
                .MinimumLevel.Information()
                .WriteTo.Console(
                    outputTemplate: "[{Timestamp:HH:mm:ss} {Level:u3}] {Message:lj}{NewLine}{Exception}")
                .WriteTo.File(
                    path: System.IO.Path.Combine(
                        Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
                        ".config", "sass", "daemon.log"),
                    rollingInterval: Serilog.RollingInterval.Day)
                .CreateLogger();

            try
            {
                Log.Information(
                    "SASS Windows Daemon v1.0.0 — Copyright (c) 2026 Saki Studio. All rights reserved.");

                var builder = Host.CreateDefaultBuilder(args)
                    .UseWindowsService(options =>
                    {
                        options.ServiceName = "SakiSshDaemon";
                    })
                    .UseSerilog()
                    .ConfigureServices((context, services) =>
                    {
                        // Rust FFI Bridge（Singleton 生命週期）
                        services.AddSingleton<RustBridge>();

                        // 7 Plugins（全部 Singleton）
                        services.AddSingleton<ChaCha20Challenge>();
                        services.AddSingleton<TlsExporterBinding>();
                        services.AddSingleton<TarpitBuffer>();
                        services.AddSingleton<AuditChain>();
                        services.AddSingleton<ViSwap>();
                        services.AddSingleton<BranchManager>();
                        services.AddSingleton<EnvInjector>();

                        // 將所有 Plugin 註冊為 IPlugin 介面
                        services.AddSingleton<IPlugin>(sp => sp.GetRequiredService<ChaCha20Challenge>());
                        services.AddSingleton<IPlugin>(sp => sp.GetRequiredService<TlsExporterBinding>());
                        services.AddSingleton<IPlugin>(sp => sp.GetRequiredService<TarpitBuffer>());
                        services.AddSingleton<IPlugin>(sp => sp.GetRequiredService<AuditChain>());
                        services.AddSingleton<IPlugin>(sp => sp.GetRequiredService<ViSwap>());
                        services.AddSingleton<IPlugin>(sp => sp.GetRequiredService<BranchManager>());
                        services.AddSingleton<IPlugin>(sp => sp.GetRequiredService<EnvInjector>());

                        // Worker Service 主體
                        services.AddHostedService<Worker>();
                    });

                var host = builder.Build();
                host.Run();
            }
            catch (Exception ex)
            {
                Log.Fatal(ex, "SASS Daemon 啟動失敗");
            }
            finally
            {
                Log.CloseAndFlush();
            }
        }
    }
}
