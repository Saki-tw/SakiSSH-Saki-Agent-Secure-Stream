# go-sakissh — SASS Go 實作

> **SASS v5.0** · **RFC**: draft-sakistudio-sass-07
>
> [🇹🇼 繁體中文](#繁體中文) | [🇺🇸 English](#english) | [🇯🇵 日本語](#日本語)

---

# 繁體中文

## 簡介

`go-sakissh` 是 **SASS (Saki Agent Secure Stream)** 協議的 Go 語言實作，提供完整的 daemon 和 client，實作 RFC draft-sakistudio-sass-07 定義的所有 7 個 Plugin。作為 SASS 生態系中的次要跨平台參考實作。

Go daemon 使用 **goroutine-based 並行** 實現 Tarpit 緩慢滴注機制，並使用標準函式庫 `crypto/chacha20poly1305` 處理認知挑戰。

### 與 SASS RFC 的關係

本實作對應 RFC Appendix B 中的 "Go Implementation"，為次要跨平台參考實作 (secondary cross-platform reference)，支援 Linux、macOS、Windows 三平台。

## 目錄結構

```
go-sakissh/
├── cmd/
│   ├── sakisshd/    # Daemon 入口
│   └── sakissh/     # Client 入口
├── internal/
│   ├── defense/     # Plugin C.1, C.2 及防禦模組
│   ├── server/      # Plugin C.3-C.7 及伺服器核心
│   ├── codec/       # 編解碼器
│   └── config/      # 設定檔解析
└── go.mod
```

## 編譯

### 前置需求

| 工具 | 版本 | 安裝方式 |
|------|------|---------|
| Go | ≥ 1.21 | macOS: `brew install go` / Windows: `choco install golang` |
| protoc | ≥ 3.21 | macOS: `brew install protobuf` |

### 編譯指令

```bash
cd go-sakissh

# Daemon
go build -o bin/sakisshd ./cmd/sakisshd

# Client
go build -o bin/sakissh ./cmd/sakissh

# 或一次編譯所有
go build ./...
```

### 靜態連結

```bash
CGO_ENABLED=0 go build -o bin/sakisshd ./cmd/sakisshd
```

## 測試

```bash
cd go-sakissh
go test ./...
```

## 實作的 Plugin 列表

| Plugin # | RFC 錨點 | 名稱 | Go Package |
|----------|---------|------|-----------|
| C.1 | `chacha20-challenge` | ChaCha20 Cognitive Challenge | `internal/defense/challenge_store.go` |
| C.2 | `tls-exporter-binding` | TLS Exporter Binding | `internal/defense/tls_exporter.go` |
| C.3 | `tarpit-buffer` | Zero-Alloc Tarpit | `internal/server/tarpit.go` |
| C.4 | `ed25519-audit` | ED25519 Audit Chain | `internal/server/audit.go` |
| C.5 | `vi-swap-ansi` | Vi Swap ANSI | `internal/server/v6_integration.go` |
| C.6 | `symlink-tree` | Transparent Branching | `internal/server/branch_mgr.go` |
| C.7 | `volatile-cache` | EnvInjector | `internal/server/env_injector.go` |

### 其他核心模組

| 檔案 | 功能 |
|------|------|
| `internal/defense/capability.go` | 權限模型 |
| `internal/defense/policy_engine.go` | 策略引擎 |
| `internal/defense/watchdog.go` | 行程逾時監控 |
| `internal/defense/localhost_defense.go` | LocalHost 偽裝防禦 |
| `internal/server/auth.go` | 認證模組 |
| `internal/server/execute.go` | 指令執行 |
| `internal/server/server.go` | gRPC 伺服器 |

## 相依套件

| Module | 用途 |
|--------|------|
| `google.golang.org/grpc` | gRPC 伺服器 / 客戶端 |
| `google.golang.org/protobuf` | Protocol Buffers |
| `golang.org/x/crypto` | ED25519、ChaCha20-Poly1305 |

---

# English

## Introduction

`go-sakissh` is the Go language implementation of the **SASS (Saki Agent Secure Stream)** protocol, providing a full daemon and client with all 7 Plugins defined in RFC draft-sakistudio-sass-07. It serves as the secondary cross-platform reference implementation in the SASS ecosystem.

The Go daemon uses **goroutine-based concurrency** for the Tarpit slow-drip mechanism and the standard library `crypto/chacha20poly1305` for cognitive challenges.

### Relationship to SASS RFC

This implementation corresponds to the "Go Implementation" in RFC Appendix B, serving as the secondary cross-platform reference supporting Linux, macOS, and Windows.

## Build

### Prerequisites

| Tool | Version | Installation |
|------|---------|-------------|
| Go | ≥ 1.21 | macOS: `brew install go` / Windows: `choco install golang` |
| protoc | ≥ 3.21 | macOS: `brew install protobuf` |

### Build Commands

```bash
cd go-sakissh

# Daemon
go build -o bin/sakisshd ./cmd/sakisshd

# Client
go build -o bin/sakissh ./cmd/sakissh

# Or build all
go build ./...
```

### Static Linking

```bash
CGO_ENABLED=0 go build -o bin/sakisshd ./cmd/sakisshd
```

## Test

```bash
cd go-sakissh
go test ./...
```

## Implemented Plugins

| Plugin # | RFC Anchor | Name | Go Package |
|----------|-----------|------|-----------|
| C.1 | `chacha20-challenge` | ChaCha20 Cognitive Challenge | `internal/defense/challenge_store.go` |
| C.2 | `tls-exporter-binding` | TLS Exporter Binding | `internal/defense/tls_exporter.go` |
| C.3 | `tarpit-buffer` | Zero-Alloc Tarpit | `internal/server/tarpit.go` |
| C.4 | `ed25519-audit` | ED25519 Audit Chain | `internal/server/audit.go` |
| C.5 | `vi-swap-ansi` | Vi Swap ANSI | `internal/server/v6_integration.go` |
| C.6 | `symlink-tree` | Transparent Branching | `internal/server/branch_mgr.go` |
| C.7 | `volatile-cache` | EnvInjector | `internal/server/env_injector.go` |

## Dependencies

| Module | Purpose |
|--------|---------|
| `google.golang.org/grpc` | gRPC server/client |
| `google.golang.org/protobuf` | Protocol Buffers |
| `golang.org/x/crypto` | ED25519, ChaCha20-Poly1305 |

---

# 日本語

## はじめに

`go-sakissh` は **SASS (Saki Agent Secure Stream)** プロトコルの Go 言語実装です。完全な daemon と client を提供し、RFC draft-sakistudio-sass-07 で定義されたすべての 7 つの Plugin を実装しています。SASS エコシステムにおける二次的なクロスプラットフォーム参照実装です。

Go daemon は **goroutine ベースの並行処理** で Tarpit のスローディップメカニズムを実現し、標準ライブラリの `crypto/chacha20poly1305` で認知チャレンジを処理します。

### SASS RFC との関係

本実装は RFC Appendix B の "Go Implementation" に対応し、Linux、macOS、Windows をサポートする二次的クロスプラットフォーム参照実装です。

## ビルド

### 必要なもの

| ツール | バージョン | インストール |
|--------|-----------|------------|
| Go | ≥ 1.21 | macOS: `brew install go` / Windows: `choco install golang` |
| protoc | ≥ 3.21 | macOS: `brew install protobuf` |

### ビルドコマンド

```bash
cd go-sakissh

# Daemon
go build -o bin/sakisshd ./cmd/sakisshd

# Client
go build -o bin/sakissh ./cmd/sakissh

# すべてをビルド
go build ./...
```

## テスト

```bash
cd go-sakissh
go test ./...
```

## 実装された Plugin 一覧

| Plugin # | RFC アンカー | 名前 | Go パッケージ |
|----------|------------|------|-------------|
| C.1 | `chacha20-challenge` | ChaCha20 認知チャレンジ | `internal/defense/challenge_store.go` |
| C.2 | `tls-exporter-binding` | TLS Exporter バインディング | `internal/defense/tls_exporter.go` |
| C.3 | `tarpit-buffer` | ゼロアロケーション Tarpit | `internal/server/tarpit.go` |
| C.4 | `ed25519-audit` | ED25519 監査チェーン | `internal/server/audit.go` |
| C.5 | `vi-swap-ansi` | Vi Swap ANSI | `internal/server/v6_integration.go` |
| C.6 | `symlink-tree` | 透過的ブランチング | `internal/server/branch_mgr.go` |
| C.7 | `volatile-cache` | EnvInjector | `internal/server/env_injector.go` |

## 依存関係

| モジュール | 用途 |
|-----------|------|
| `google.golang.org/grpc` | gRPC サーバー / クライアント |
| `google.golang.org/protobuf` | Protocol Buffers |
| `golang.org/x/crypto` | ED25519、ChaCha20-Poly1305 |

---

*Saki Studio · SASS v5.0 · draft-sakistudio-sass-07*
