# windows-daemon-csharp — SASS C# Windows 服務 Daemon

> **SASS v5.0** · **RFC**: draft-sakistudio-sass-07
>
> [🇹🇼 繁體中文](#繁體中文) | [🇺🇸 English](#english) | [🇯🇵 日本語](#日本語)

---

# 繁體中文

## 簡介

`windows-daemon-csharp` 是 **SASS (Saki Agent Secure Stream)** 協議的 C# 實作，提供原生 Windows daemon，以 .NET 8 Worker Service 架構運行。它實作 RFC draft-sakistudio-sass-07 定義的所有 7 個 Plugin，並透過 **Rust FFI interop** (P/Invoke) 處理效能關鍵的加密操作（ChaCha20-Poly1305 和 ED25519）。

### 與 SASS RFC 的關係

本實作對應 RFC Appendix B 中的 "C# Windows Service Daemon"，為 Windows 平台的原生 daemon 實作。

### 架構特色

- **Service 生命週期**：透過 `Microsoft.Extensions.Hosting.BackgroundService` 管理
- **gRPC 傳輸**：使用 `Grpc.Net.Client` + `SslCredentials` 支援 TLS 1.3
- **Rust FFI**：ChaCha20 和 ED25519 操作透過 `[DllImport("sass_crypto_ffi")]` 連結原生 Rust 函式庫
- **Tarpit 零配置**：使用 `ArrayPool<byte>.Shared` 實現零記憶體配置串流
- **透明分支**：使用 NTFS Junction Points，含 symlink→hardlink→copy 三級退化策略
- **環境變數注入**：目標為 `%TEMP%\sass_vol\`，符合 Windows 路徑慣例

## 編譯

### 前置需求

| 工具 | 版本 | 安裝方式 |
|------|------|---------|
| .NET SDK | ≥ 8.0 | macOS: `brew install dotnet` / Windows: [dot.net](https://dot.net) |
| MSBuild | ≥ 17.0 | 隨 .NET SDK 附帶 |

### Windows 原生編譯

```powershell
cd windows-daemon-csharp
dotnet build -c Release
# 產出: SakiSshDaemon/bin/Release/net8.0/SakiSshDaemon.exe
```

### 從 macOS 交叉編譯

```bash
cd windows-daemon-csharp
dotnet publish -c Release -r win-x64 --self-contained
# 產出: SakiSshDaemon/bin/Release/net8.0/win-x64/publish/SakiSshDaemon.exe
```

## 測試

```bash
cd windows-daemon-csharp
dotnet test
```

## 安裝為 Windows 服務

```powershell
sc.exe create SakiSshDaemon binPath= "C:\path\to\SakiSshDaemon.exe"
sc.exe start SakiSshDaemon
```

## 實作的 Plugin 列表

| Plugin # | RFC 錨點 | 名稱 | C# 類別 | 備註 |
|----------|---------|------|--------|------|
| C.1 | `chacha20-challenge` | ChaCha20 Cognitive Challenge | `ChaCha20Challenge` | Rust FFI interop |
| C.2 | `tls-exporter-binding` | TLS Exporter Binding | `TlsExporterBinding` | HMAC fallback (待 .NET 9 EKM API) |
| C.3 | `tarpit-buffer` | Zero-Alloc Tarpit | `TarpitBuffer` | ArrayPool zero-alloc |
| C.4 | `ed25519-audit` | ED25519 Audit Chain | `AuditChain` | HMAC-SHA256 fallback (待 NSec) |
| C.5 | `vi-swap-ansi` | Vi Swap ANSI | `ViSwap` | ConHost ANSI VT |
| C.6 | `symlink-tree` | Transparent Branching | `BranchManager` | NTFS Junction |
| C.7 | `volatile-cache` | EnvInjector | `EnvInjector` | %TEMP%\sass_vol\ |

所有 Plugin 實作 `IPlugin` 介面，檔案位於 `SakiSshDaemon.Plugins/` 目錄。

### 已知限制

1. **AuditChain**: ED25519 簽名使用 HMAC-SHA256 fallback（.NET 無原生 Ed25519，待整合 `NSec.Cryptography`）
2. **TlsExporterBinding**: .NET 8 `SslStream` 不支援 `ExportKeyingMaterial()` API，使用 HMAC fallback（追蹤 `dotnet/runtime#97485`）

## 相依套件

| NuGet Package | 用途 |
|---------------|------|
| `Grpc.AspNetCore` | gRPC 伺服器 |
| `Microsoft.Extensions.Hosting.WindowsServices` | Windows Service 託管 |
| `System.Security.Cryptography` | SHA256、HMAC、AES |

---

# English

## Introduction

`windows-daemon-csharp` is the C# implementation of the **SASS (Saki Agent Secure Stream)** protocol, providing a native Windows daemon running as a .NET 8 Worker Service. It implements all 7 Plugins defined in RFC draft-sakistudio-sass-07, using **Rust FFI interop** (P/Invoke) for performance-critical cryptographic operations (ChaCha20-Poly1305 and ED25519).

### Relationship to SASS RFC

This implementation corresponds to "C# Windows Service Daemon" in RFC Appendix B, providing the native Windows daemon.

### Architecture Highlights

- **Service lifecycle**: Managed via `Microsoft.Extensions.Hosting.BackgroundService`
- **gRPC transport**: `Grpc.Net.Client` + `SslCredentials` for TLS 1.3
- **Rust FFI**: Native ChaCha20 and ED25519 via `[DllImport("sass_crypto_ffi")]`
- **Tarpit zero-alloc**: `ArrayPool<byte>.Shared` for zero-allocation streaming
- **Transparent Branching**: NTFS Junction Points with three-level degradation
- **EnvInjector**: Targets `%TEMP%\sass_vol\` for Windows path conventions

## Build

### Prerequisites

| Tool | Version | Installation |
|------|---------|-------------|
| .NET SDK | ≥ 8.0 | macOS: `brew install dotnet` / Windows: [dot.net](https://dot.net) |
| MSBuild | ≥ 17.0 | Bundled with .NET SDK |

### Build (Windows Native)

```powershell
cd windows-daemon-csharp
dotnet build -c Release
# Output: SakiSshDaemon/bin/Release/net8.0/SakiSshDaemon.exe
```

### Cross-Compile from macOS

```bash
cd windows-daemon-csharp
dotnet publish -c Release -r win-x64 --self-contained
# Output: SakiSshDaemon/bin/Release/net8.0/win-x64/publish/SakiSshDaemon.exe
```

## Test

```bash
cd windows-daemon-csharp
dotnet test
```

## Install as Windows Service

```powershell
sc.exe create SakiSshDaemon binPath= "C:\path\to\SakiSshDaemon.exe"
sc.exe start SakiSshDaemon
```

## Implemented Plugins

| Plugin # | RFC Anchor | Name | C# Class | Notes |
|----------|-----------|------|----------|-------|
| C.1 | `chacha20-challenge` | ChaCha20 Cognitive Challenge | `ChaCha20Challenge` | Rust FFI interop |
| C.2 | `tls-exporter-binding` | TLS Exporter Binding | `TlsExporterBinding` | HMAC fallback |
| C.3 | `tarpit-buffer` | Zero-Alloc Tarpit | `TarpitBuffer` | ArrayPool zero-alloc |
| C.4 | `ed25519-audit` | ED25519 Audit Chain | `AuditChain` | HMAC-SHA256 fallback |
| C.5 | `vi-swap-ansi` | Vi Swap ANSI | `ViSwap` | ConHost ANSI VT |
| C.6 | `symlink-tree` | Transparent Branching | `BranchManager` | NTFS Junction |
| C.7 | `volatile-cache` | EnvInjector | `EnvInjector` | %TEMP%\sass_vol\ |

All Plugins implement the `IPlugin` interface, located in `SakiSshDaemon.Plugins/`.

### Known Limitations

1. **AuditChain**: ED25519 signatures use HMAC-SHA256 fallback (pending `NSec.Cryptography` integration)
2. **TlsExporterBinding**: .NET 8 `SslStream` lacks `ExportKeyingMaterial()` API (tracking `dotnet/runtime#97485`)

## Dependencies

| NuGet Package | Purpose |
|---------------|---------|
| `Grpc.AspNetCore` | gRPC server |
| `Microsoft.Extensions.Hosting.WindowsServices` | Windows Service hosting |
| `System.Security.Cryptography` | SHA256, HMAC, AES |

---

# 日本語

## はじめに

`windows-daemon-csharp` は **SASS (Saki Agent Secure Stream)** プロトコルの C# 実装で、.NET 8 Worker Service として実行されるネイティブ Windows daemon を提供します。RFC draft-sakistudio-sass-07 で定義されたすべての 7 つの Plugin を実装し、パフォーマンスが重要な暗号操作（ChaCha20-Poly1305 と ED25519）には **Rust FFI interop** (P/Invoke) を使用します。

### SASS RFC との関係

本実装は RFC Appendix B の "C# Windows Service Daemon" に対応し、Windows プラットフォーム向けのネイティブ daemon 実装を提供します。

### アーキテクチャの特徴

- **サービスライフサイクル**: `Microsoft.Extensions.Hosting.BackgroundService` で管理
- **gRPC トランスポート**: TLS 1.3 対応の `Grpc.Net.Client` + `SslCredentials`
- **Rust FFI**: `[DllImport("sass_crypto_ffi")]` 経由のネイティブ暗号操作
- **Tarpit ゼロアロケーション**: `ArrayPool<byte>.Shared` によるゼロ割り当てストリーミング
- **透過的ブランチング**: NTFS ジャンクションポイント（3 段階劣化戦略付き）
- **環境変数注入**: Windows パス規約に従い `%TEMP%\sass_vol\` を使用

## ビルド

### 必要なもの

| ツール | バージョン | インストール |
|--------|-----------|------------|
| .NET SDK | ≥ 8.0 | macOS: `brew install dotnet` / Windows: [dot.net](https://dot.net) |

### ビルドコマンド（Windows ネイティブ）

```powershell
cd windows-daemon-csharp
dotnet build -c Release
# 出力: SakiSshDaemon/bin/Release/net8.0/SakiSshDaemon.exe
```

### macOS からのクロスコンパイル

```bash
cd windows-daemon-csharp
dotnet publish -c Release -r win-x64 --self-contained
# 出力: SakiSshDaemon/bin/Release/net8.0/win-x64/publish/SakiSshDaemon.exe
```

## テスト

```bash
cd windows-daemon-csharp
dotnet test
```

## 実装された Plugin 一覧

| Plugin # | RFC アンカー | 名前 | C# クラス | 備考 |
|----------|------------|------|----------|------|
| C.1 | `chacha20-challenge` | ChaCha20 認知チャレンジ | `ChaCha20Challenge` | Rust FFI |
| C.2 | `tls-exporter-binding` | TLS Exporter バインディング | `TlsExporterBinding` | HMAC フォールバック |
| C.3 | `tarpit-buffer` | ゼロアロケーション Tarpit | `TarpitBuffer` | ArrayPool |
| C.4 | `ed25519-audit` | ED25519 監査チェーン | `AuditChain` | HMAC-SHA256 フォールバック |
| C.5 | `vi-swap-ansi` | Vi Swap ANSI | `ViSwap` | ConHost ANSI VT |
| C.6 | `symlink-tree` | 透過的ブランチング | `BranchManager` | NTFS ジャンクション |
| C.7 | `volatile-cache` | EnvInjector | `EnvInjector` | %TEMP%\sass_vol\ |

すべての Plugin は `IPlugin` インターフェースを実装し、`SakiSshDaemon.Plugins/` ディレクトリに配置されています。

## 依存関係

| NuGet パッケージ | 用途 |
|----------------|------|
| `Grpc.AspNetCore` | gRPC サーバー |
| `Microsoft.Extensions.Hosting.WindowsServices` | Windows Service ホスティング |
| `System.Security.Cryptography` | SHA256、HMAC、AES |

---

*Saki Studio · SASS v5.0 · draft-sakistudio-sass-07*
