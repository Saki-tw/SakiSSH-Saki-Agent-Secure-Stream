# saki-ssh-daemon — SASS Rust 參考實作

> **SASS v1.4** · **RFC**: draft-sakistudio-sass-05
>
> [🇹🇼 繁體中文](#繁體中文) | [🇺🇸 English](#english) | [🇯🇵 日本語](#日本語)

---

# 繁體中文

## 簡介

`saki-ssh-daemon` 是 **SASS (Saki Agent Secure Stream)** 協議的主要參考實作，使用 Rust 編寫。它同時包含 daemon 和 client，實作 RFC draft-sakistudio-sass-05 定義的所有 7 個 Plugin，是 SASS 生態系中涵蓋最完整的實作。

本目錄為 Cargo workspace，包含：
- **Daemon** (`sakisshd`)：守護進程，部署於運算端
- **Client** (`sakissh`)：位於 `../saki-ssh-client/`

### 與 SASS RFC 的關係

本實作為 SASS RFC 的 **canonical reference implementation**（典範參考實作），RFC Appendix B 中標記為 "Rust Daemon (Primary)"。所有 Plugin 的行為規格以本實作為準。

## 編譯

### 前置需求

| 工具 | 版本 | 安裝方式 |
|------|------|---------|
| Rust toolchain | ≥ 1.75 | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| protoc | ≥ 3.21 | macOS: `brew install protobuf` / Windows: `choco install protoc` |

### 編譯指令

```bash
cd saki-ssh-daemon
cargo build --release
# 產出: target/release/sakisshd
```

### Windows 交叉編譯

```bash
rustup target add x86_64-pc-windows-gnu
brew install mingw-w64
cargo build --release --target x86_64-pc-windows-gnu
# 產出: target/x86_64-pc-windows-gnu/release/sakisshd.exe
```

### 靜態連結

```bash
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release
```

## 測試

```bash
cargo test --workspace
```

## 實作的 Plugin 列表

| Plugin # | RFC 錨點 | 名稱 | 實作檔案 |
|----------|---------|------|---------|
| C.1 | `chacha20-challenge` | ChaCha20 Cognitive Challenge | `src/challenge_store.rs` |
| C.2 | `tls-exporter-binding` | TLS Exporter Binding | `src/threat_defense.rs` |
| C.3 | `tarpit-buffer` | Zero-Alloc Tarpit | `src/tarpit.rs` |
| C.4 | `ed25519-audit` | ED25519 Audit Chain | `src/audit.rs` |
| C.5 | `vi-swap-ansi` | Vi Swap ANSI | `src/tarpit.rs` (vi_swap) |
| C.6 | `symlink-tree` | Transparent Branching | `src/branch_mgr.rs` |
| C.7 | `volatile-cache` | EnvInjector | `src/env_injector.rs` |

### 其他核心模組

| 檔案 | 功能 |
|------|------|
| `src/v6_integration.rs` | 6-Response 狀態機 |
| `src/session.rs` | Ring Buffer + Session 生命週期 |
| `src/watchdog.rs` | 行程逾時監控 |
| `src/localhost_defense.rs` | LocalHost 偽裝防禦 |
| `src/auth.rs` | 認證模組 |
| `src/policy.rs` | 策略引擎 |
| `src/capability.rs` | 權限模型 |

## 相依套件

| Crate | 用途 |
|-------|------|
| `tonic` | gRPC 伺服器 / 客戶端 |
| `prost` | Protocol Buffers |
| `tokio` | 非同步執行期 |
| `ipnetwork` | CIDR ACL |
| `chacha20poly1305` | Plugin C.1 |
| `ed25519-dalek` | Plugin C.4 |
| `serde` / `serde_json` | 設定檔解析 |
| `clap` | CLI 參數解析 |

---

# English

## Introduction

`saki-ssh-daemon` is the primary reference implementation of the **SASS (Saki Agent Secure Stream)** protocol, written in Rust. It includes both daemon and client, implementing all 7 Plugins defined in RFC draft-sakistudio-sass-05 — the most complete implementation in the SASS ecosystem.

This directory is a Cargo workspace containing:
- **Daemon** (`sakisshd`): deployed on the compute plane
- **Client** (`sakissh`): located at `../saki-ssh-client/`

### Relationship to SASS RFC

This is the **canonical reference implementation** of the SASS RFC, labeled "Rust Daemon (Primary)" in Appendix B. All Plugin behavioral specifications are authoritative from this implementation.

## Build

### Prerequisites

| Tool | Version | Installation |
|------|---------|-------------|
| Rust toolchain | ≥ 1.75 | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| protoc | ≥ 3.21 | macOS: `brew install protobuf` / Windows: `choco install protoc` |

### Build Commands

```bash
cd saki-ssh-daemon
cargo build --release
# Output: target/release/sakisshd
```

### Windows Cross-Compilation

```bash
rustup target add x86_64-pc-windows-gnu
brew install mingw-w64
cargo build --release --target x86_64-pc-windows-gnu
# Output: target/x86_64-pc-windows-gnu/release/sakisshd.exe
```

## Test

```bash
cargo test --workspace
```

## Implemented Plugins

| Plugin # | RFC Anchor | Name | Implementation File |
|----------|-----------|------|-------------------|
| C.1 | `chacha20-challenge` | ChaCha20 Cognitive Challenge | `src/challenge_store.rs` |
| C.2 | `tls-exporter-binding` | TLS Exporter Binding | `src/threat_defense.rs` |
| C.3 | `tarpit-buffer` | Zero-Alloc Tarpit | `src/tarpit.rs` |
| C.4 | `ed25519-audit` | ED25519 Audit Chain | `src/audit.rs` |
| C.5 | `vi-swap-ansi` | Vi Swap ANSI | `src/tarpit.rs` (vi_swap) |
| C.6 | `symlink-tree` | Transparent Branching | `src/branch_mgr.rs` |
| C.7 | `volatile-cache` | EnvInjector | `src/env_injector.rs` |

---

# 日本語

## はじめに

`saki-ssh-daemon` は **SASS (Saki Agent Secure Stream)** プロトコルの主要な参照実装であり、Rust で書かれています。daemon と client の両方を含み、RFC draft-sakistudio-sass-05 で定義されたすべての 7 つの Plugin を実装しています。

このディレクトリは Cargo ワークスペースであり、以下を含みます：
- **Daemon** (`sakisshd`)：計算プレーンにデプロイ
- **Client** (`sakissh`)：`../saki-ssh-client/` に配置

### SASS RFC との関係

本実装は SASS RFC の **canonical reference implementation**（標準参照実装）です。RFC Appendix B では "Rust Daemon (Primary)" として分類されています。

## ビルド

### 必要なもの

| ツール | バージョン | インストール |
|--------|-----------|------------|
| Rust toolchain | ≥ 1.75 | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| protoc | ≥ 3.21 | macOS: `brew install protobuf` / Windows: `choco install protoc` |

### ビルドコマンド

```bash
cd saki-ssh-daemon
cargo build --release
# 出力: target/release/sakisshd
```

## テスト

```bash
cargo test --workspace
```

## 実装された Plugin 一覧

| Plugin # | RFC アンカー | 名前 | 実装ファイル |
|----------|------------|------|------------|
| C.1 | `chacha20-challenge` | ChaCha20 認知チャレンジ | `src/challenge_store.rs` |
| C.2 | `tls-exporter-binding` | TLS Exporter バインディング | `src/threat_defense.rs` |
| C.3 | `tarpit-buffer` | ゼロアロケーション Tarpit | `src/tarpit.rs` |
| C.4 | `ed25519-audit` | ED25519 監査チェーン | `src/audit.rs` |
| C.5 | `vi-swap-ansi` | Vi Swap ANSI | `src/tarpit.rs` (vi_swap) |
| C.6 | `symlink-tree` | 透過的ブランチング | `src/branch_mgr.rs` |
| C.7 | `volatile-cache` | EnvInjector | `src/env_injector.rs` |

---

*Saki Studio · SASS v1.4 · draft-sakistudio-sass-05*
