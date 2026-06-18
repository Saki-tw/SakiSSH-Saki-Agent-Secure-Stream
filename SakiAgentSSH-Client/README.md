# SakiAgentSSH-Client — SASS Swift macOS 客戶端

> **SASS v5.0** · **RFC**: draft-sakistudio-sass-07
>
> [🇹🇼 繁體中文](#繁體中文) | [🇺🇸 English](#english) | [🇯🇵 日本語](#日本語)

---

# 繁體中文

## 簡介

`SakiAgentSSH-Client` 是 **SASS (Saki Agent Secure Stream)** 協議的 Swift 實作，提供原生 macOS 客戶端。使用 Apple 的 **CryptoKit** 框架和 **Network.framework** 處理 TLS 1.3 傳輸，實作 4 個 client 端相關的 Plugin。

本應用程式包含：
- **CLI 客戶端**：透過終端機的 `sakissh` 指令執行遠端操作
- **GUI 應用程式**：帶有馬卡龍紫 (`#DA70D6`) 與勿忘草青 (`#00CED1`) 色系的 SwiftUI 介面

### 與 SASS RFC 的關係

本實作對應 RFC Appendix B 中的 "Swift macOS Plugins Client"。由於 Tarpit、Vi Swap、Transparent Branching 為 daemon 端機制，client 端僅需實作 4 個 Plugin。

## 編譯

### 前置需求

| 工具 | 版本 | 安裝方式 |
|------|------|---------|
| Xcode | ≥ 16.0 | Mac App Store |
| XcodeGen | latest | `brew install xcodegen` |
| macOS SDK | ≥ 13.0 | 隨 Xcode 附帶 |

### 編譯指令

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

> **注意**：`.xcodeproj` 不應加入 Git 追蹤（已在 `.gitignore` 中排除），請始終透過 `project.yml` + `xcodegen generate` 產生。

## 測試

```bash
cd SakiAgentSSH-Client
xcodegen generate
xcodebuild test -project SakiAgentSSHClient.xcodeproj \
    -scheme SakiAgentSSHClientTests \
    -destination 'platform=macOS'
# 測試檔案: Tests/PluginTests.swift
```

測試項目包含 Plugin 合規性驗證，確認所有常數與 RFC 規格一致。

## 實作的 Plugin 列表

| Plugin # | RFC 錨點 | 名稱 | Swift 模組 | Framework |
|----------|---------|------|-----------|-----------|
| C.1 | `chacha20-challenge` | ChaCha20 Cognitive Challenge | `ChaCha20Solver.swift` | CryptoKit `ChaChaPoly` |
| C.2 | `tls-exporter-binding` | TLS Exporter Binding | `TlsExporterBinding.swift` | Network.framework `sec_protocol_metadata` |
| C.4 | `ed25519-audit` | ED25519 Audit Chain | `AuditVerifier.swift` | CryptoKit `Curve25519.Signing` |
| C.7 | `volatile-cache` | EnvInjector | `EnvInjectorClient.swift` | Foundation `ProcessInfo` |

### 未實作的 Plugin（Daemon-only）

| Plugin # | 名稱 | 原因 |
|----------|------|------|
| C.3 | Zero-Alloc Tarpit | Daemon 端機制 |
| C.5 | Vi Swap ANSI | Daemon 端機制 |
| C.6 | Transparent Branching | Daemon 端機制 |

### Plugin 管理器

`Sources/Plugins/PluginManager.swift` 負責統一管理所有 Plugin 的初始化與生命週期。

### 已知限制

- **TlsExporterBinding**: 依賴 `NWConnection` TLS metadata + HKDF-SHA256 衍生，而非直接的 RFC 5705 exporter（Apple 平台限制）

## 多語言說明書

本應用程式內建三語系說明書，透過 `Cmd+?` 開啟 Help Book：

| 語言 | 檔案 |
|------|------|
| 🇹🇼 繁體中文 | `Resources/help_zh-Hant.md` |
| 🇺🇸 English | `Resources/help_en-US.md` |
| 🇯🇵 日本語 | `Resources/help_ja-JP.md` |

## 相依框架

| Framework | 用途 |
|-----------|------|
| SwiftUI | 使用者介面 |
| CryptoKit | ChaCha20-Poly1305、HKDF、HMAC、Curve25519 |
| Network | NWConnection TLS metadata |
| Combine | 響應式事件處理 |

## 設計美學

- **色系**：馬卡龍紫 (`#DA70D6`) 與勿忘草青 (`#00CED1`)
- **字型**：`GenJyuuGothicX-Regular`
- **背景**：雙色漸層 (`--bg-gradient`)

---

# English

## Introduction

`SakiAgentSSH-Client` is the Swift implementation of the **SASS (Saki Agent Secure Stream)** protocol, providing a native macOS client. It uses Apple's **CryptoKit** framework and **Network.framework** for TLS 1.3 transport, implementing 4 client-relevant Plugins.

This application includes:
- **CLI client**: Remote execution via `sakissh` terminal command
- **GUI application**: SwiftUI interface with Macaron Purple (`#DA70D6`) and Forget-me-not Blue (`#00CED1`) theme

### Relationship to SASS RFC

This implementation corresponds to "Swift macOS Plugins Client" in RFC Appendix B. Since Tarpit, Vi Swap, and Transparent Branching are daemon-side mechanisms, the client only implements 4 Plugins.

## Build

### Prerequisites

| Tool | Version | Installation |
|------|---------|-------------|
| Xcode | ≥ 16.0 | Mac App Store |
| XcodeGen | latest | `brew install xcodegen` |
| macOS SDK | ≥ 13.0 | Bundled with Xcode |

### Build Commands

```bash
cd SakiAgentSSH-Client
xcodegen generate
xcodebuild build -project SakiAgentSSHClient.xcodeproj \
    -scheme SakiAgentSSHClient \
    -configuration Release \
    SYMROOT=$(pwd)/build_out \
    ONLY_ACTIVE_ARCH=NO
```

> **Note**: `.xcodeproj` should not be tracked in Git (already in `.gitignore`). Always generate via `project.yml` + `xcodegen generate`.

## Test

```bash
cd SakiAgentSSH-Client
xcodegen generate
xcodebuild test -project SakiAgentSSHClient.xcodeproj \
    -scheme SakiAgentSSHClientTests \
    -destination 'platform=macOS'
# Test file: Tests/PluginTests.swift
```

## Implemented Plugins

| Plugin # | RFC Anchor | Name | Swift Module | Framework |
|----------|-----------|------|-------------|-----------|
| C.1 | `chacha20-challenge` | ChaCha20 Cognitive Challenge | `ChaCha20Solver.swift` | CryptoKit `ChaChaPoly` |
| C.2 | `tls-exporter-binding` | TLS Exporter Binding | `TlsExporterBinding.swift` | Network.framework |
| C.4 | `ed25519-audit` | ED25519 Audit Chain | `AuditVerifier.swift` | CryptoKit `Curve25519.Signing` |
| C.7 | `volatile-cache` | EnvInjector | `EnvInjectorClient.swift` | Foundation `ProcessInfo` |

### Unimplemented Plugins (Daemon-only)

| Plugin # | Name | Reason |
|----------|------|--------|
| C.3 | Zero-Alloc Tarpit | Daemon-side mechanism |
| C.5 | Vi Swap ANSI | Daemon-side mechanism |
| C.6 | Transparent Branching | Daemon-side mechanism |

## Built-in Multilingual Help

| Language | File |
|----------|------|
| 🇹🇼 Traditional Chinese | `Resources/help_zh-Hant.md` |
| 🇺🇸 English | `Resources/help_en-US.md` |
| 🇯🇵 Japanese | `Resources/help_ja-JP.md` |

## Dependencies

| Framework | Purpose |
|-----------|---------|
| SwiftUI | User interface |
| CryptoKit | ChaCha20-Poly1305, HKDF, HMAC, Curve25519 |
| Network | NWConnection TLS metadata |
| Combine | Reactive event handling |

---

# 日本語

## はじめに

`SakiAgentSSH-Client` は **SASS (Saki Agent Secure Stream)** プロトコルの Swift 実装で、ネイティブ macOS クライアントを提供します。Apple の **CryptoKit** フレームワークと **Network.framework** を使用して TLS 1.3 トランスポートを処理し、4 つのクライアント関連 Plugin を実装しています。

このアプリケーションには以下が含まれます：
- **CLI クライアント**: ターミナルの `sakissh` コマンドによるリモート実行
- **GUI アプリケーション**: マカロンパープル (`#DA70D6`) と忘れな草のブルー (`#00CED1`) をテーマにした SwiftUI インターフェース

### SASS RFC との関係

本実装は RFC Appendix B の "Swift macOS Plugins Client" に対応します。Tarpit、Vi Swap、透過的ブランチングは daemon 側のメカニズムであるため、クライアントは 4 つの Plugin のみを実装しています。

## ビルド

### 必要なもの

| ツール | バージョン | インストール |
|--------|-----------|------------|
| Xcode | ≥ 16.0 | Mac App Store |
| XcodeGen | 最新版 | `brew install xcodegen` |
| macOS SDK | ≥ 13.0 | Xcode に同梱 |

### ビルドコマンド

```bash
cd SakiAgentSSH-Client
xcodegen generate
xcodebuild build -project SakiAgentSSHClient.xcodeproj \
    -scheme SakiAgentSSHClient \
    -configuration Release \
    SYMROOT=$(pwd)/build_out \
    ONLY_ACTIVE_ARCH=NO
```

> **お願い**: `.xcodeproj` は Git に追跡させないでください（`.gitignore` に登録済み）。常に `project.yml` + `xcodegen generate` で生成してください。

## テスト

```bash
cd SakiAgentSSH-Client
xcodegen generate
xcodebuild test -project SakiAgentSSHClient.xcodeproj \
    -scheme SakiAgentSSHClientTests \
    -destination 'platform=macOS'
# テストファイル: Tests/PluginTests.swift
```

## 実装された Plugin 一覧

| Plugin # | RFC アンカー | 名前 | Swift モジュール | フレームワーク |
|----------|------------|------|----------------|--------------|
| C.1 | `chacha20-challenge` | ChaCha20 認知チャレンジ | `ChaCha20Solver.swift` | CryptoKit `ChaChaPoly` |
| C.2 | `tls-exporter-binding` | TLS Exporter バインディング | `TlsExporterBinding.swift` | Network.framework |
| C.4 | `ed25519-audit` | ED25519 監査チェーン | `AuditVerifier.swift` | CryptoKit `Curve25519.Signing` |
| C.7 | `volatile-cache` | EnvInjector | `EnvInjectorClient.swift` | Foundation `ProcessInfo` |

### 未実装の Plugin（Daemon 専用）

| Plugin # | 名前 | 理由 |
|----------|------|------|
| C.3 | ゼロアロケーション Tarpit | Daemon 側メカニズム |
| C.5 | Vi Swap ANSI | Daemon 側メカニズム |
| C.6 | 透過的ブランチング | Daemon 側メカニズム |

## 内蔵多言語ヘルプ

| 言語 | ファイル |
|------|--------|
| 🇹🇼 繁體中文 | `Resources/help_zh-Hant.md` |
| 🇺🇸 English | `Resources/help_en-US.md` |
| 🇯🇵 日本語 | `Resources/help_ja-JP.md` |

## 依存関係

| フレームワーク | 用途 |
|--------------|------|
| SwiftUI | ユーザーインターフェース |
| CryptoKit | ChaCha20-Poly1305、HKDF、HMAC、Curve25519 |
| Network | NWConnection TLS メタデータ |
| Combine | リアクティブイベント処理 |

## デザイン美学

- **カラーテーマ**: マカロンパープル (`#DA70D6`) と忘れな草のブルー (`#00CED1`)
- **フォント**: `GenJyuuGothicX-Regular`
- **背景**: ダブルカラーグラデーション (`--bg-gradient`)

---

*Saki Studio · SASS v5.0 · draft-sakistudio-sass-07*
