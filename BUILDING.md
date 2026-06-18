# SASS 建置指南 / SASS Building Guide

> **SASS v5.0** · **RFC**: draft-sakistudio-sass-07
>
> [🇹🇼 繁體中文](#繁體中文) | [🇺🇸 English](#english)

---

# 繁體中文

## 概述

SakiAgentSSH（SASS）參考實作涵蓋四種語言生態系，每個協議皆為完整的一對 daemon + client。本文件說明各實作的編譯、測試與部署方式。

## 前置需求

### 通用工具

| 工具 | 用途 | 安裝方式 |
|------|------|---------|
| `protoc` (≥ 3.21) | Protocol Buffers 編譯 | macOS: `brew install protobuf` / Windows: `choco install protoc` |
| `git` | 版本控制 | macOS: `xcode-select --install` |

### Rust 工具鏈 (≥ 1.75)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Go 工具鏈 (≥ 1.21)

```bash
# macOS
brew install go

# Windows
choco install golang
```

### .NET SDK (≥ 8.0)

```bash
# macOS
brew install dotnet

# Windows: https://dot.net
```

### Swift / Xcode (macOS 專屬)

| 工具 | 版本 | 安裝方式 |
|------|------|---------|
| Xcode | ≥ 16.0 | Mac App Store |
| XcodeGen | latest | `brew install xcodegen` |
| macOS SDK | ≥ 13.0 | 隨 Xcode 附帶 |

### Windows 交叉編譯工具鏈 (從 macOS 交叉編譯)

```bash
rustup target add x86_64-pc-windows-gnu
brew install mingw-w64
```

---

## 一、Rust 實作 (`saki-ssh-daemon/`)

**角色**：Daemon + Client（主要參考實作）
**平台**：Linux、macOS、Windows
**Plugin 覆蓋率**：7/7

### 編譯

```bash
# Daemon（守護進程）
cd saki-ssh-daemon
cargo build --release
# 產出: target/release/sakisshd

# Client（客戶端）
cd saki-ssh-client
cargo build --release
# 產出: target/release/sakissh
```

### 測試

```bash
cd saki-ssh-daemon
cargo test --workspace
```

### Windows 交叉編譯

```bash
cd saki-ssh-daemon
cargo build --release --target x86_64-pc-windows-gnu
# 產出: target/x86_64-pc-windows-gnu/release/sakisshd.exe

cd ../saki-ssh-client
cargo build --release --target x86_64-pc-windows-gnu
# 產出: target/x86_64-pc-windows-gnu/release/sakissh.exe
```

### Windows 原生編譯

```powershell
# 需要 Visual Studio Build Tools 或 GNU toolchain
cd saki-ssh-daemon
cargo build --release
# 產出: target\release\sakisshd.exe
```

### 靜態連結 (建議用於 SASS 管理的 session)

```bash
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release
```

### 相依套件

| Crate | 用途 |
|-------|------|
| `tonic` | gRPC 伺服器 / 客戶端 |
| `prost` | Protocol Buffers |
| `tokio` | 非同步執行期 |
| `ipnetwork` | CIDR ACL |
| `chacha20poly1305` | ChaCha20-Poly1305 認知挑戰 (Plugin C.1) |
| `ed25519-dalek` | ED25519 稽核鏈簽章 (Plugin C.4) |
| `serde` / `serde_json` | 設定檔解析 |
| `clap` | CLI 參數解析 |

---

## 二、Go 實作 (`go-sakissh/`)

**角色**：Daemon + Client（次要跨平台參考）
**平台**：Linux、macOS、Windows
**Plugin 覆蓋率**：7/7

### 編譯

```bash
cd go-sakissh

# Daemon
go build -o bin/sakisshd ./cmd/sakisshd

# Client
go build -o bin/sakissh ./cmd/sakissh

# 或使用 go build ./... 編譯所有套件
go build ./...
```

### 測試

```bash
cd go-sakissh
go test ./...
```

### 靜態連結

```bash
CGO_ENABLED=0 go build -o bin/sakisshd ./cmd/sakisshd
```

### 相依套件

| Module | 用途 |
|--------|------|
| `google.golang.org/grpc` | gRPC 伺服器 / 客戶端 |
| `google.golang.org/protobuf` | Protocol Buffers |
| `golang.org/x/crypto` | ED25519、ChaCha20-Poly1305 |

---

## 三、C# 實作 (`windows-daemon-csharp/`)

**角色**：Daemon（Windows 原生服務）
**平台**：Windows
**Plugin 覆蓋率**：7/7
**架構**：.NET 8 Worker Service + Rust FFI (P/Invoke)

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

### 測試

```bash
cd windows-daemon-csharp
dotnet test
```

### 安裝為 Windows 服務

```powershell
sc.exe create SakiSshDaemon binPath= "C:\path\to\SakiSshDaemon.exe"
sc.exe start SakiSshDaemon
```

### 相依套件

| NuGet Package | 用途 |
|---------------|------|
| `Grpc.AspNetCore` | gRPC 伺服器 |
| `Microsoft.Extensions.Hosting.WindowsServices` | Windows Service 託管 |
| `System.Security.Cryptography` | SHA256、HMAC、AES |

> **注意**：ChaCha20-Poly1305 與 ED25519 操作委派給 Rust FFI 函式庫 (`sass_crypto_ffi.dll`)，透過 P/Invoke 調用，以確保常數時間操作，避免 .NET JIT 編譯帶來的時序側信道。

---

## 四、Swift 實作 (`SakiAgentSSH-Client/`)

**角色**：Client（macOS 原生客戶端）
**平台**：macOS
**Plugin 覆蓋率**：4/7（ChaCha20、TLS Exporter、ED25519 Audit、EnvInjector）

### 編譯

```bash
cd SakiAgentSSH-Client
xcodegen generate
xcodebuild build -project SakiAgentSSHClient.xcodeproj \
    -scheme SakiAgentSSHClient \
    -configuration Release \
    SYMROOT=$(pwd)/build_out \
    ONLY_ACTIVE_ARCH=NO
# 產出: build_out/Release/SakiAgentSSHClient.app
```

### 測試

```bash
cd SakiAgentSSH-Client
xcodegen generate
xcodebuild test -project SakiAgentSSHClient.xcodeproj \
    -scheme SakiAgentSSHClientTests \
    -destination 'platform=macOS'
# 測試檔案: Tests/PluginTests.swift
```

### 相依框架

| Framework | 用途 |
|-----------|------|
| SwiftUI | 使用者介面 |
| CryptoKit | ChaCha20-Poly1305、HKDF、HMAC、Curve25519 (Plugins) |
| Network | NWConnection TLS metadata (Plugin C.2) |
| Combine | 響應式事件處理 |

> **注意**：
> - `.xcodeproj` 不應加入 Git 追蹤（已在 `.gitignore` 中排除），請始終透過 `project.yml` + `xcodegen generate` 產生。
> - 未實作的 3 個 Plugin（Tarpit、Vi Swap、Transparent Branching）為 daemon 端機制，client 無需實作。

---

## macOS GUI 應用程式 (SwiftUI)

除了 CLI 工具外，我們為 SakiAgentSSH 提供了帶有馬卡龍紫 (`#DA70D6`) 與勿忘草青 (`#00CED1`) 色系的 SwiftUI 介面包裝。

### 編譯 Daemon App

```bash
cd SakiAgentSSH-Daemon
xcodegen generate
xcodebuild build -project SakiAgentSSHDaemon.xcodeproj \
    -scheme SakiAgentSSHDaemon \
    -configuration Release \
    SYMROOT=$(pwd)/build_out \
    ONLY_ACTIVE_ARCH=NO
# 產出: build_out/Release/SakiAgentSSHDaemon.app
```

### Archive for App Store

```bash
xattr -dr com.apple.quarantine .
xcodebuild -project SakiAgentSSHDaemon.xcodeproj \
    -scheme SakiAgentSSHDaemon \
    -configuration Release \
    archive -archivePath ./build/SakiAgentSSHDaemon.xcarchive
```

### DMG 封裝

```bash
hdiutil create -volname "SakiAgentSSH Daemon" \
    -srcfolder build_out/Release/SakiAgentSSHDaemon.app \
    -ov -format UDZO \
    SakiAgentSSHDaemon.dmg
```

---

## RFC 編譯與驗證

```bash
# 編譯 RFC Internet-Draft
xml2rfc docs/ietf-submission/draft-sakistudio-sass-07.xml --text --html

# 格式檢查
idnits docs/ietf-submission/draft-sakistudio-sass-07.txt
```

---

## 發行檢查清單

```
[ ] cargo build --release (macOS ARM64)
[ ] cargo build --release --target x86_64-pc-windows-gnu (Windows)
[ ] strip binaries
[ ] go build ./... (go-sakissh daemon + client)
[ ] dotnet publish -r win-x64 (C# daemon)
[ ] xcodegen generate (Daemon + Client)
[ ] xcodebuild archive (or Xcode GUI)
[ ] hdiutil create DMG
[ ] xcodebuild test (Swift Plugin 測試)
[ ] cargo test --workspace (Rust 測試)
[ ] go test ./... (Go 測試)
[ ] dotnet test (C# 測試)
[ ] shasum -a 256 *.dmg *.exe
[ ] Update Scoop manifest hash
[ ] Update Homebrew Cask hash
[ ] Update Winget manifest hash
[ ] git tag vX.Y.Z
[ ] Upload to GitHub Releases
```

---

## 故障排除

- **`protoc` 找不到**：`cargo build` 出現 `tonic-build` / `prost` 相關錯誤時，請確認 `protoc` 在 PATH 中。
- **macOS App 閃退**：手動複製 `.app` 後，請移除隔離屬性：`xattr -dr com.apple.quarantine /Applications/SakiAgentSSHDaemon.app`
- **Windows C# daemon 啟動失敗**：確認 `sass_crypto_ffi.dll` 與 `SakiSshDaemon.exe` 位於同一目錄。

---

# English

## Overview

SakiAgentSSH (SASS) reference implementation spans four language ecosystems. Each protocol is a complete daemon + client pair. This document covers building, testing, and deploying all implementations.

## Prerequisites

### Common Tools

| Tool | Purpose | Installation |
|------|---------|-------------|
| `protoc` (≥ 3.21) | Protocol Buffers compilation | macOS: `brew install protobuf` / Windows: `choco install protoc` |
| `git` | Version control | macOS: `xcode-select --install` |

### Rust Toolchain (≥ 1.75)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Go Toolchain (≥ 1.21)

```bash
# macOS
brew install go

# Windows
choco install golang
```

### .NET SDK (≥ 8.0)

```bash
# macOS
brew install dotnet

# Windows: https://dot.net
```

### Swift / Xcode (macOS only)

| Tool | Version | Installation |
|------|---------|-------------|
| Xcode | ≥ 16.0 | Mac App Store |
| XcodeGen | latest | `brew install xcodegen` |
| macOS SDK | ≥ 13.0 | Bundled with Xcode |

### Windows Cross-Compilation Toolchain (from macOS)

```bash
rustup target add x86_64-pc-windows-gnu
brew install mingw-w64
```

---

## 1. Rust Implementation (`saki-ssh-daemon/`)

**Role**: Daemon + Client (Primary Reference Implementation)
**Platform**: Linux, macOS, Windows
**Plugin Coverage**: 7/7

### Build

```bash
# Daemon
cd saki-ssh-daemon
cargo build --release
# Output: target/release/sakisshd

# Client
cd saki-ssh-client
cargo build --release
# Output: target/release/sakissh
```

### Test

```bash
cd saki-ssh-daemon
cargo test --workspace
```

### Windows Cross-Compilation

```bash
cd saki-ssh-daemon
cargo build --release --target x86_64-pc-windows-gnu
# Output: target/x86_64-pc-windows-gnu/release/sakisshd.exe

cd ../saki-ssh-client
cargo build --release --target x86_64-pc-windows-gnu
# Output: target/x86_64-pc-windows-gnu/release/sakissh.exe
```

### Static Linking (Recommended for SASS-managed sessions)

```bash
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release
```

### Dependencies

| Crate | Purpose |
|-------|---------|
| `tonic` | gRPC server/client |
| `prost` | Protocol Buffers |
| `tokio` | Async runtime |
| `ipnetwork` | CIDR ACL |
| `chacha20poly1305` | ChaCha20-Poly1305 cognitive challenge (Plugin C.1) |
| `ed25519-dalek` | ED25519 audit chain signatures (Plugin C.4) |
| `serde` / `serde_json` | Config parsing |
| `clap` | CLI argument parsing |

---

## 2. Go Implementation (`go-sakissh/`)

**Role**: Daemon + Client (Secondary Cross-Platform Reference)
**Platform**: Linux, macOS, Windows
**Plugin Coverage**: 7/7

### Build

```bash
cd go-sakissh

# Daemon
go build -o bin/sakisshd ./cmd/sakisshd

# Client
go build -o bin/sakissh ./cmd/sakissh

# Or build all packages
go build ./...
```

### Test

```bash
cd go-sakissh
go test ./...
```

### Static Linking

```bash
CGO_ENABLED=0 go build -o bin/sakisshd ./cmd/sakisshd
```

### Dependencies

| Module | Purpose |
|--------|---------|
| `google.golang.org/grpc` | gRPC server/client |
| `google.golang.org/protobuf` | Protocol Buffers |
| `golang.org/x/crypto` | ED25519, ChaCha20-Poly1305 |

---

## 3. C# Implementation (`windows-daemon-csharp/`)

**Role**: Daemon (Windows Native Service)
**Platform**: Windows
**Plugin Coverage**: 7/7
**Architecture**: .NET 8 Worker Service + Rust FFI (P/Invoke)

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

### Test

```bash
cd windows-daemon-csharp
dotnet test
```

### Install as Windows Service

```powershell
sc.exe create SakiSshDaemon binPath= "C:\path\to\SakiSshDaemon.exe"
sc.exe start SakiSshDaemon
```

### Dependencies

| NuGet Package | Purpose |
|---------------|---------|
| `Grpc.AspNetCore` | gRPC server |
| `Microsoft.Extensions.Hosting.WindowsServices` | Windows Service hosting |
| `System.Security.Cryptography` | SHA256, HMAC, AES |

> **Note**: ChaCha20-Poly1305 and ED25519 operations are delegated to a Rust FFI library (`sass_crypto_ffi.dll`) via P/Invoke for constant-time operations.

---

## 4. Swift Implementation (`SakiAgentSSH-Client/`)

**Role**: Client (macOS Native)
**Platform**: macOS
**Plugin Coverage**: 4/7 (ChaCha20, TLS Exporter, ED25519 Audit, EnvInjector)

### Build

```bash
cd SakiAgentSSH-Client
xcodegen generate
xcodebuild build -project SakiAgentSSHClient.xcodeproj \
    -scheme SakiAgentSSHClient \
    -configuration Release \
    SYMROOT=$(pwd)/build_out \
    ONLY_ACTIVE_ARCH=NO
```

### Test

```bash
cd SakiAgentSSH-Client
xcodegen generate
xcodebuild test -project SakiAgentSSHClient.xcodeproj \
    -scheme SakiAgentSSHClientTests \
    -destination 'platform=macOS'
# Test file: Tests/PluginTests.swift
```

### Dependencies

| Framework | Purpose |
|-----------|---------|
| SwiftUI | User interface |
| CryptoKit | ChaCha20-Poly1305, HKDF, HMAC, Curve25519 (Plugins) |
| Network | NWConnection TLS metadata (Plugin C.2) |
| Combine | Reactive event handling |

> **Note**: The 3 unimplemented Plugins (Tarpit, Vi Swap, Transparent Branching) are daemon-side mechanisms and not required for client implementations.

---

## RFC Compilation & Validation

```bash
xml2rfc docs/ietf-submission/draft-sakistudio-sass-07.xml --text --html
idnits docs/ietf-submission/draft-sakistudio-sass-07.txt
```

---

## Release Checklist

```
[ ] cargo build --release (macOS ARM64)
[ ] cargo build --release --target x86_64-pc-windows-gnu (Windows)
[ ] strip binaries
[ ] go build ./... (go-sakissh daemon + client)
[ ] dotnet publish -r win-x64 (C# daemon)
[ ] xcodegen generate (Daemon + Client)
[ ] xcodebuild archive (or Xcode GUI)
[ ] hdiutil create DMG
[ ] xcodebuild test (Swift plugin tests)
[ ] cargo test --workspace (Rust tests)
[ ] go test ./... (Go tests)
[ ] dotnet test (C# tests)
[ ] shasum -a 256 *.dmg *.exe
[ ] Update Scoop manifest hash
[ ] Update Homebrew Cask hash
[ ] Update Winget manifest hash
[ ] git tag vX.Y.Z
[ ] Upload to GitHub Releases
```

---

## Troubleshooting

- **`protoc` not found**: If `cargo build` fails with `tonic-build` / `prost` errors, ensure `protoc` is in your PATH.
- **macOS App crashes on launch**: After manually copying `.app`, remove quarantine: `xattr -dr com.apple.quarantine /Applications/SakiAgentSSHDaemon.app`
- **Windows C# daemon fails to start**: Ensure `sass_crypto_ffi.dll` is in the same directory as `SakiSshDaemon.exe`.

---

*Saki Studio · SASS v5.0 · draft-sakistudio-sass-07*
