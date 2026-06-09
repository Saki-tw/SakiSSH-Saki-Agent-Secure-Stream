# SASS Plugins API 參考 / SASS Plugins API Reference / SASS プラグイン API リファレンス

> **SASS v1.4** · **RFC**: draft-sakistudio-sass-05, Appendix C
>
> [🇹🇼 繁體中文](#繁體中文) | [🇺🇸 English](#english) | [🇯🇵 日本語](#日本語)

---

# 繁體中文

## 概述

本文件為 SASS RFC draft-sakistudio-sass-05 Appendix C 所定義的 7 個 Plugin 之 API 參考手冊。這些 Plugin 均為 **OPTIONAL**（可選）且 **INFORMATIVE**（資訊性），用於 Saki Studio 的參考實作。

### 部署模型

每個 SASS 協議為完整的一對 **daemon + client**：

| 實作 | 語言 | 平台 | 角色 | Plugin 覆蓋 |
|------|------|------|------|------------|
| Rust Daemon | Rust | Linux, macOS, Windows | Daemon + Client | 7/7 |
| Go Implementation | Go | Linux, macOS, Windows | Daemon + Client | 7/7 |
| C# Windows Service | C# | Windows | Daemon | 7/7 |
| Swift macOS Client | Swift | macOS | Client | 4/7 |

---

## C.1 — ChaCha20-Poly1305 認知挑戰

**RFC 錨點**: `chacha20-challenge`
**RFC 參照**: [RFC 8439](https://www.rfc-editor.org/rfc/rfc8439)

### 常數規格

| 參數 | 值 | 說明 |
|------|-----|------|
| Key 長度 | 32 bytes | 對稱金鑰 |
| Nonce 長度 | 12 bytes | 隨機 nonce |
| Plaintext 長度 | 64 bytes | 隨機明文 |
| TTL | 60 秒 | 挑戰有效期限 |

### 流程

1. Daemon 產生隨機 32-byte key、12-byte nonce、64-byte plaintext
2. 使用 ChaCha20-Poly1305 加密明文（結合 TLS Exporter 綁定值，見 C.2）
3. 將 (key, nonce, plaintext) 三元組存儲，TTL = 60 秒
4. Daemon 發送 (nonce, ciphertext) 至 Agent（透過 AuthResponse）
5. Agent 解密並透過 CognitiveChallenge RPC 回傳明文
6. Daemon 進行常數時間比較

### 各語言實作

| 語言 | 檔案路徑 | 類別 / 模組 |
|------|---------|------------|
| Rust | `saki-ssh-daemon/src/challenge_store.rs` | `ChallengeStore` |
| Go | `go-sakissh/internal/defense/challenge_store.go` | `ChallengeStore` |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/ChaCha20Challenge.cs` | `ChaCha20Challenge` |
| Swift | `SakiAgentSSH-Client/Sources/Plugins/ChaCha20Solver.swift` | `ChaCha20Solver` |

> **C# 實作注意**：ChaCha20-Poly1305 操作委派給 Rust FFI 函式庫 (`sass_crypto_ffi.dll`)，透過 P/Invoke 調用，使用 `Span<byte>` 固定緩衝區（`GCHandle` pin），避免 .NET JIT 的時序側信道。

---

## C.2 — TLS Exporter 綁定

**RFC 錨點**: `tls-exporter-binding`
**RFC 參照**: [RFC 5705](https://www.rfc-editor.org/rfc/rfc5705)、[RFC 9266](https://www.rfc-editor.org/rfc/rfc9266)

### 常數規格

| 參數 | 值 | 說明 |
|------|-----|------|
| Label | `"EXPORTER-sakissh-chacha20-v14"` | TLS Exporter 標籤 |
| Context | 16 bytes (Session UUID) | TLS Exporter 上下文 |
| Length | 44 bytes | 導出金鑰材料長度 |
| Key 分割 | bytes 0-31 = ChaCha20 key | 32-byte 加密金鑰 |
| Nonce 分割 | bytes 32-43 = ChaCha20 nonce | 12-byte nonce |

### 客戶端 HMAC

```
client_ekm_hmac = HMAC-SHA256(EKM_key, session_id)
```

### 各語言實作

| 語言 | 檔案路徑 | 類別 / 模組 |
|------|---------|------------|
| Rust | `saki-ssh-daemon/src/threat_defense.rs` | TLS EKM 推導與 HMAC 驗證 |
| Go | `go-sakissh/internal/defense/tls_exporter.go` | TLS EKM 推導 |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/TlsExporterBinding.cs` | `TlsExporterBinding` |
| Swift | `SakiAgentSSH-Client/Sources/Plugins/TlsExporterBinding.swift` | `TlsExporterBinding` |

> **C# 已知限制**：.NET 8 `SslStream` 不支援 `ExportKeyingMaterial()` API，使用 HMAC fallback。等待 .NET 9+ (`dotnet/runtime#97485`)。
>
> **Swift 已知限制**：依賴 `NWConnection` TLS metadata + HKDF-SHA256 衍生，而非直接的 RFC 5705 exporter（Apple 平台限制）。

---

## C.3 — Zero-Allocation Tarpit 靜態緩衝區

**RFC 錨點**: `tarpit-buffer`

### 常數規格

| 參數 | 值 | 說明 |
|------|-----|------|
| 總負載 | 40 MiB | 總傳送量 |
| Chunk 大小 | 64 KiB | 每塊大小 |
| Chunk 間隔 | 500 ms | 緩慢滴注延遲 |
| 總 Chunk 數 | 640 | 40 MiB / 64 KiB |
| 總持續時間 | ~320 秒 | 640 × 500ms |
| 並行閘道 | `AtomicI32` / `Interlocked`, max 32 | 最大並行 tarpit 數 |

### Rust 靜態緩衝區

```rust
static STATIC_ENTROPY: OnceLock<Vec<u8>> = OnceLock::new();
```

### 各語言實作

| 語言 | 檔案路徑 | 類別 / 模組 |
|------|---------|------------|
| Rust | `saki-ssh-daemon/src/tarpit.rs` | `OnceLock` 靜態 entropy buffer |
| Go | `go-sakissh/internal/server/tarpit.go` | Goroutine-based slow-drip |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/TarpitBuffer.cs` | `TarpitBuffer` |
| Swift | N/A | Daemon-only plugin，Client 不實作 |

> **C# 實作注意**：使用 `ArrayPool<byte>.Shared.Rent(65536)` 達成零配置串流，`try/finally` 歸還緩衝區。並行閘道使用 `Interlocked.Increment/Decrement`。

---

## C.4 — ED25519 雜湊鏈稽核日誌

**RFC 錨點**: `ed25519-audit`
**RFC 參照**: [RFC 8032](https://www.rfc-editor.org/rfc/rfc8032)

### 常數規格

| 參數 | 值 | 說明 |
|------|-----|------|
| Genesis 種子 | `"SASS_GENESIS_BLOCK"` | 首筆記錄的 chain_hash 種子 |
| 時戳格式 | RFC 3339 | 時戳格式 |
| 事件格式 | JSON | 結構化事件資料 |

### 雜湊鏈結構

```
chain_hash = SHA256(previous_chain_hash || event_json || timestamp)
signature  = ED25519_Sign(daemon_private_key, chain_hash)
```

### 各語言實作

| 語言 | 檔案路徑 | 類別 / 模組 |
|------|---------|------------|
| Rust | `saki-ssh-daemon/src/audit.rs` | ED25519 hash chain 日誌寫入 |
| Go | `go-sakissh/internal/server/audit.go` | Audit chain 寫入 |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/AuditChain.cs` | `AuditChain` |
| Swift | `SakiAgentSSH-Client/Sources/Plugins/AuditVerifier.swift` | `AuditVerifier` |

> **C# 已知限制**：ED25519 簽名使用 HMAC-SHA256 fallback。需整合 `NSec.Cryptography` 套件實現真正的 Ed25519 簽名。

---

## C.5 — Vi Swap ANSI Escape Sequence

**RFC 錨點**: `vi-swap-ansi`

### 常數規格

| 參數 | 值 | 說明 |
|------|-----|------|
| 最大維持時間 | 3600 秒 | max_hold |
| 心跳間隔 | 5 秒 | heartbeat |

### ANSI 序列表

| Byte Sequence | 用途 |
|--------------|------|
| `\x1b[?1049h` | 進入替代畫面緩衝區 |
| `\x1b[2J` | 清除整個畫面 |
| `\x1b[H` | 游標移至左上角 (1,1) |
| `\x1b[?25l` | 隱藏游標 |
| `\x1b[24;1H` | 游標移至底部狀態列 |

### 各語言實作

| 語言 | 檔案路徑 | 類別 / 模組 |
|------|---------|------------|
| Rust | `saki-ssh-daemon/src/tarpit.rs` (vi_swap) | Vi Swap ANSI escape |
| Go | `go-sakissh/internal/server/v6_integration.go` | Vi Swap |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/ViSwap.cs` | `ViSwap` |
| Swift | N/A | Daemon-only plugin，Client 不實作 |

> **C# 實作注意**：Windows 上需啟用 ConHost ANSI Virtual Terminal (VT) 處理。透過 P/Invoke 調用 `SetConsoleMode(handle, ENABLE_VIRTUAL_TERMINAL_PROCESSING)`。若 VT 處理不可用（如無頭服務），則退化為傳送原始 UTF-8 噪音模式。

---

## C.6 — 透明分支 (Symlink Tree)

**RFC 錨點**: `symlink-tree`

### 常數規格

| 參數 | 值 | 說明 |
|------|-----|------|
| 排除目錄 | `target/`, `.git/`, `node_modules/` | 不進行符號連結的目錄 |

### 分支結構

```
/tmp/sass_branches/{session_id}/
├── src/         <- 真實目錄（建立）
│   ├── main.rs  <- symlink -> /orig/src/main.rs
│   └── lib.rs   <- symlink -> /orig/src/lib.rs
└── Cargo.toml   <- symlink -> /orig/Cargo.toml
```

### 分支生命週期

| 操作 | 介面 | 說明 |
|------|------|------|
| 建立 | `create_micro_branch(session_id, target_dir) -> branch_path` | 建立微型分支 |
| 合併 | `merge_branch(session_id) -> apply diff` | 將差異套用至真實檔案系統 |
| 丟棄 | `drop_branch(session_id) -> rm -rf` | 刪除分支目錄 |

### 各語言實作

| 語言 | 檔案路徑 | 類別 / 模組 |
|------|---------|------------|
| Rust | `saki-ssh-daemon/src/branch_mgr.rs` | Symlink tree 微型分支 |
| Go | `go-sakissh/internal/server/branch_mgr.go` | Branch manager |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/BranchManager.cs` | `BranchManager` |
| Swift | N/A | Daemon-only plugin，Client 不實作 |

> **C# 實作注意**：Windows (NTFS) 上實作三級退化策略：
> 1. **NTFS Junction Points**（優先）：需 Windows 10 1703+ 開發者模式
> 2. **Hardlinks**：跨卷時使用 `CreateHardLink`
> 3. **Full copy**：FAT32 等不支援時退化為 `File.Copy`
>
> 退化等級記錄至稽核軌跡。

---

## C.7 — 揮發性快取重導向 (EnvInjector)

**RFC 錨點**: `volatile-cache`

### 常數規格 (POSIX)

| 偵測工具 | 環境變數 | 重導向目標 |
|---------|---------|-----------|
| npm/yarn/pnpm | `npm_config_cache` | `/tmp/sass_vol/npm` |
| npm/yarn/pnpm | `YARN_CACHE_FOLDER` | `/tmp/sass_vol/yarn` |
| cargo/rustc | `CARGO_TARGET_DIR` | `/tmp/sass_vol/ct` |
| cargo/rustc | `CARGO_HOME` | `/tmp/sass_vol/ch` |
| pip | `PIP_CACHE_DIR` | `/tmp/sass_vol/pip` |
| (所有指令) | `TMPDIR` | `/tmp/sass_vol/tmp` |

### 常數規格 (Windows)

| 環境變數 | Windows 重導向目標 |
|---------|-------------------|
| `npm_config_cache` | `%TEMP%\sass_vol\npm` |
| `YARN_CACHE_FOLDER` | `%TEMP%\sass_vol\yarn` |
| `CARGO_TARGET_DIR` | `%TEMP%\sass_vol\ct` |
| `CARGO_HOME` | `%TEMP%\sass_vol\ch` |
| `PIP_CACHE_DIR` | `%TEMP%\sass_vol\pip` |
| `TEMP` / `TMP` | `%TEMP%\sass_vol\tmp` |

### 各語言實作

| 語言 | 檔案路徑 | 類別 / 模組 |
|------|---------|------------|
| Rust | `saki-ssh-daemon/src/env_injector.rs` | 環境變數揮發性快取注入 |
| Go | `go-sakissh/internal/server/env_injector.go` | Env injector |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/EnvInjector.cs` | `EnvInjector` |
| Swift | `SakiAgentSSH-Client/Sources/Plugins/EnvInjectorClient.swift` | `EnvInjectorClient` |

> **C# 實作注意**：Windows 上使用 `Directory.CreateDirectory` 處理完整路徑階層。同時設定 `TEMP` 和 `TMP` 環境變數（Windows 慣例），而非 POSIX 的 `TMPDIR`。

---

# English

## Overview

This document serves as the API reference for the 7 Plugins defined in SASS RFC draft-sakistudio-sass-05 Appendix C. All Plugins are **OPTIONAL** and **INFORMATIVE**, specific to the Saki Studio reference implementation.

### Deployment Model

Each SASS protocol constitutes a complete **daemon + client** pair:

| Implementation | Language | Platform | Role | Plugin Coverage |
|---------------|----------|----------|------|----------------|
| Rust Daemon | Rust | Linux, macOS, Windows | Daemon + Client | 7/7 |
| Go Implementation | Go | Linux, macOS, Windows | Daemon + Client | 7/7 |
| C# Windows Service | C# | Windows | Daemon | 7/7 |
| Swift macOS Client | Swift | macOS | Client | 4/7 |

---

## C.1 — ChaCha20-Poly1305 Cognitive Challenge

**RFC Anchor**: `chacha20-challenge`
**RFC Reference**: [RFC 8439](https://www.rfc-editor.org/rfc/rfc8439)

### Constants

| Parameter | Value | Description |
|-----------|-------|-------------|
| Key length | 32 bytes | Symmetric key |
| Nonce length | 12 bytes | Random nonce |
| Plaintext length | 64 bytes | Random plaintext |
| TTL | 60 seconds | Challenge validity period |

### Flow

1. Daemon generates random 32-byte key, 12-byte nonce, 64-byte plaintext
2. Plaintext encrypted via ChaCha20-Poly1305 (combined with TLS Exporter binding, see C.2)
3. Tuple (key, nonce, plaintext) stored with 60-second TTL
4. Daemon sends (nonce, ciphertext) to Agent via AuthResponse
5. Agent decrypts and returns plaintext via CognitiveChallenge RPC
6. Daemon performs constant-time comparison

### Implementation Files

| Language | File Path | Class / Module |
|----------|-----------|---------------|
| Rust | `saki-ssh-daemon/src/challenge_store.rs` | `ChallengeStore` |
| Go | `go-sakissh/internal/defense/challenge_store.go` | `ChallengeStore` |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/ChaCha20Challenge.cs` | `ChaCha20Challenge` |
| Swift | `SakiAgentSSH-Client/Sources/Plugins/ChaCha20Solver.swift` | `ChaCha20Solver` |

> **C# Note**: ChaCha20-Poly1305 is delegated to Rust FFI (`sass_crypto_ffi.dll`) via P/Invoke with pinned `Span<byte>` buffers (`GCHandle`), avoiding .NET JIT timing side-channels.

---

## C.2 — TLS Exporter Binding for Cognitive Challenge

**RFC Anchor**: `tls-exporter-binding`
**RFC Reference**: [RFC 5705](https://www.rfc-editor.org/rfc/rfc5705), [RFC 9266](https://www.rfc-editor.org/rfc/rfc9266)

### Constants

| Parameter | Value | Description |
|-----------|-------|-------------|
| Label | `"EXPORTER-sakissh-chacha20-v14"` | TLS Exporter label |
| Context | 16 bytes (Session UUID) | TLS Exporter context |
| Length | 44 bytes | Exported keying material length |
| Key split | bytes 0-31 = ChaCha20 key | 32-byte encryption key |
| Nonce split | bytes 32-43 = ChaCha20 nonce | 12-byte nonce |

### Client HMAC

```
client_ekm_hmac = HMAC-SHA256(EKM_key, session_id)
```

### Implementation Files

| Language | File Path | Class / Module |
|----------|-----------|---------------|
| Rust | `saki-ssh-daemon/src/threat_defense.rs` | TLS EKM derivation + HMAC verification |
| Go | `go-sakissh/internal/defense/tls_exporter.go` | TLS EKM derivation |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/TlsExporterBinding.cs` | `TlsExporterBinding` |
| Swift | `SakiAgentSSH-Client/Sources/Plugins/TlsExporterBinding.swift` | `TlsExporterBinding` |

> **C# Limitation**: .NET 8 `SslStream` lacks `ExportKeyingMaterial()` API; uses HMAC fallback. Pending .NET 9+ (`dotnet/runtime#97485`).
>
> **Swift Limitation**: Uses `NWConnection` TLS metadata + HKDF-SHA256 derivation instead of direct RFC 5705 exporter (Apple platform limitation).

---

## C.3 — Zero-Allocation Tarpit Static Buffer

**RFC Anchor**: `tarpit-buffer`

### Constants

| Parameter | Value | Description |
|-----------|-------|-------------|
| Total payload | 40 MiB | Total data sent |
| Chunk size | 64 KiB | Per-chunk size |
| Inter-chunk delay | 500 ms | Slow-drip delay |
| Total chunks | 640 | 40 MiB / 64 KiB |
| Total duration | ~320 seconds | 640 × 500ms |
| Concurrency gate | `AtomicI32` / `Interlocked`, max 32 | Max concurrent tarpits |

### Implementation Files

| Language | File Path | Class / Module |
|----------|-----------|---------------|
| Rust | `saki-ssh-daemon/src/tarpit.rs` | `OnceLock` static entropy buffer |
| Go | `go-sakissh/internal/server/tarpit.go` | Goroutine-based slow-drip |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/TarpitBuffer.cs` | `TarpitBuffer` |
| Swift | N/A | Daemon-only plugin |

> **C# Note**: Uses `ArrayPool<byte>.Shared.Rent(65536)` for zero-allocation streaming.

---

## C.4 — ED25519 Hash Chain Audit Log

**RFC Anchor**: `ed25519-audit`
**RFC Reference**: [RFC 8032](https://www.rfc-editor.org/rfc/rfc8032)

### Constants

| Parameter | Value | Description |
|-----------|-------|-------------|
| Genesis seed | `"SASS_GENESIS_BLOCK"` | First record's chain_hash seed |
| Timestamp format | RFC 3339 | Timestamp format |
| Event format | JSON | Structured event data |

### Hash Chain Structure

```
chain_hash = SHA256(previous_chain_hash || event_json || timestamp)
signature  = ED25519_Sign(daemon_private_key, chain_hash)
```

### Implementation Files

| Language | File Path | Class / Module |
|----------|-----------|---------------|
| Rust | `saki-ssh-daemon/src/audit.rs` | ED25519 hash chain logging |
| Go | `go-sakissh/internal/server/audit.go` | Audit chain writer |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/AuditChain.cs` | `AuditChain` |
| Swift | `SakiAgentSSH-Client/Sources/Plugins/AuditVerifier.swift` | `AuditVerifier` |

> **C# Limitation**: ED25519 signatures use HMAC-SHA256 fallback. Requires `NSec.Cryptography` integration for proper Ed25519 signing.

---

## C.5 — Vi Swap ANSI Escape Sequence

**RFC Anchor**: `vi-swap-ansi`

### Constants

| Parameter | Value | Description |
|-----------|-------|-------------|
| Max hold time | 3600 seconds | max_hold |
| Heartbeat interval | 5 seconds | heartbeat |

### ANSI Sequence Table

| Byte Sequence | Purpose |
|--------------|---------|
| `\x1b[?1049h` | Enter alternate screen buffer |
| `\x1b[2J` | Clear entire screen |
| `\x1b[H` | Move cursor to top-left (1,1) |
| `\x1b[?25l` | Hide cursor |
| `\x1b[24;1H` | Move cursor to bottom status line |

### Implementation Files

| Language | File Path | Class / Module |
|----------|-----------|---------------|
| Rust | `saki-ssh-daemon/src/tarpit.rs` (vi_swap) | Vi Swap ANSI escape |
| Go | `go-sakissh/internal/server/v6_integration.go` | Vi Swap |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/ViSwap.cs` | `ViSwap` |
| Swift | N/A | Daemon-only plugin |

> **C# Note**: Requires ConHost ANSI VT processing on Windows. Falls back to raw UTF-8 noise for headless services.

---

## C.6 — Transparent Branching via Symlink Tree

**RFC Anchor**: `symlink-tree`

### Constants

| Parameter | Value | Description |
|-----------|-------|-------------|
| Excluded dirs | `target/`, `.git/`, `node_modules/` | Directories not symlinked |

### Branch Structure

```
/tmp/sass_branches/{session_id}/
├── src/         <- real directory (created)
│   ├── main.rs  <- symlink -> /orig/src/main.rs
│   └── lib.rs   <- symlink -> /orig/src/lib.rs
└── Cargo.toml   <- symlink -> /orig/Cargo.toml
```

### Branch Lifecycle

| Operation | Interface | Description |
|-----------|-----------|-------------|
| Create | `create_micro_branch(session_id, target_dir) -> branch_path` | Create micro-branch |
| Merge | `merge_branch(session_id) -> apply diff` | Apply diff to real FS |
| Drop | `drop_branch(session_id) -> rm -rf` | Delete branch directory |

### Implementation Files

| Language | File Path | Class / Module |
|----------|-----------|---------------|
| Rust | `saki-ssh-daemon/src/branch_mgr.rs` | Symlink tree micro-branch |
| Go | `go-sakissh/internal/server/branch_mgr.go` | Branch manager |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/BranchManager.cs` | `BranchManager` |
| Swift | N/A | Daemon-only plugin |

> **C# Note**: Three-level degradation on Windows (NTFS): Junction Points → Hardlinks → Full copy.

---

## C.7 — Volatile Cache Redirection (EnvInjector)

**RFC Anchor**: `volatile-cache`

### Constants (POSIX)

| Detected Tool | Environment Variable | Redirect Target |
|--------------|---------------------|----------------|
| npm/yarn/pnpm | `npm_config_cache` | `/tmp/sass_vol/npm` |
| npm/yarn/pnpm | `YARN_CACHE_FOLDER` | `/tmp/sass_vol/yarn` |
| cargo/rustc | `CARGO_TARGET_DIR` | `/tmp/sass_vol/ct` |
| cargo/rustc | `CARGO_HOME` | `/tmp/sass_vol/ch` |
| pip | `PIP_CACHE_DIR` | `/tmp/sass_vol/pip` |
| (all commands) | `TMPDIR` | `/tmp/sass_vol/tmp` |

### Constants (Windows)

| Environment Variable | Windows Redirect Target |
|---------------------|------------------------|
| `npm_config_cache` | `%TEMP%\sass_vol\npm` |
| `YARN_CACHE_FOLDER` | `%TEMP%\sass_vol\yarn` |
| `CARGO_TARGET_DIR` | `%TEMP%\sass_vol\ct` |
| `CARGO_HOME` | `%TEMP%\sass_vol\ch` |
| `PIP_CACHE_DIR` | `%TEMP%\sass_vol\pip` |
| `TEMP` / `TMP` | `%TEMP%\sass_vol\tmp` |

### Implementation Files

| Language | File Path | Class / Module |
|----------|-----------|---------------|
| Rust | `saki-ssh-daemon/src/env_injector.rs` | Volatile cache redirection |
| Go | `go-sakissh/internal/server/env_injector.go` | Env injector |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/EnvInjector.cs` | `EnvInjector` |
| Swift | `SakiAgentSSH-Client/Sources/Plugins/EnvInjectorClient.swift` | `EnvInjectorClient` |

> **C# Note**: Sets both `TEMP` and `TMP` (Windows convention) instead of POSIX `TMPDIR`.

---

# 日本語

## 概要

本ドキュメントは、SASS RFC draft-sakistudio-sass-05 Appendix C で定義された 7 つの Plugin の API リファレンスマニュアルです。すべての Plugin は **OPTIONAL**（任意）かつ **INFORMATIVE**（参考情報）であり、Saki Studio の参照実装に特化しています。

### デプロイメントモデル

各 SASS プロトコルは完全な **daemon + client** のペアで構成されます：

| 実装 | 言語 | プラットフォーム | 役割 | Plugin カバレッジ |
|------|------|----------------|------|-----------------|
| Rust Daemon | Rust | Linux, macOS, Windows | Daemon + Client | 7/7 |
| Go Implementation | Go | Linux, macOS, Windows | Daemon + Client | 7/7 |
| C# Windows Service | C# | Windows | Daemon | 7/7 |
| Swift macOS Client | Swift | macOS | Client | 4/7 |

---

## C.1 — ChaCha20-Poly1305 認知チャレンジ

**RFC アンカー**: `chacha20-challenge`
**RFC 参照**: [RFC 8439](https://www.rfc-editor.org/rfc/rfc8439)

### 定数仕様

| パラメータ | 値 | 説明 |
|-----------|-----|------|
| 鍵長 | 32 bytes | 対称鍵 |
| Nonce 長 | 12 bytes | ランダム nonce |
| 平文長 | 64 bytes | ランダム平文 |
| TTL | 60 秒 | チャレンジ有効期限 |

### フロー

1. Daemon がランダムな 32-byte 鍵、12-byte nonce、64-byte 平文を生成
2. ChaCha20-Poly1305 で平文を暗号化（TLS Exporter バインディング値と組み合わせ、C.2 参照）
3. (鍵, nonce, 平文) タプルを TTL = 60 秒で保存
4. Daemon が (nonce, 暗号文) を AuthResponse 経由で Agent に送信
5. Agent が復号し CognitiveChallenge RPC 経由で平文を返送
6. Daemon が定数時間比較を実行

### 各言語の実装

| 言語 | ファイルパス | クラス / モジュール |
|------|------------|-------------------|
| Rust | `saki-ssh-daemon/src/challenge_store.rs` | `ChallengeStore` |
| Go | `go-sakissh/internal/defense/challenge_store.go` | `ChallengeStore` |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/ChaCha20Challenge.cs` | `ChaCha20Challenge` |
| Swift | `SakiAgentSSH-Client/Sources/Plugins/ChaCha20Solver.swift` | `ChaCha20Solver` |

---

## C.2 — TLS Exporter バインディング

**RFC アンカー**: `tls-exporter-binding`

### 定数仕様

| パラメータ | 値 | 説明 |
|-----------|-----|------|
| ラベル | `"EXPORTER-sakissh-chacha20-v14"` | TLS Exporter ラベル |
| コンテキスト | 16 bytes (Session UUID) | TLS Exporter コンテキスト |
| 長さ | 44 bytes | エクスポート鍵素材の長さ |
| 鍵分割 | bytes 0-31 = ChaCha20 鍵 | 32-byte 暗号化鍵 |
| Nonce 分割 | bytes 32-43 = ChaCha20 nonce | 12-byte nonce |

### 各言語の実装

| 言語 | ファイルパス | クラス / モジュール |
|------|------------|-------------------|
| Rust | `saki-ssh-daemon/src/threat_defense.rs` | TLS EKM 導出 + HMAC 検証 |
| Go | `go-sakissh/internal/defense/tls_exporter.go` | TLS EKM 導出 |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/TlsExporterBinding.cs` | `TlsExporterBinding` |
| Swift | `SakiAgentSSH-Client/Sources/Plugins/TlsExporterBinding.swift` | `TlsExporterBinding` |

---

## C.3 — ゼロアロケーション Tarpit 静的バッファ

**RFC アンカー**: `tarpit-buffer`

### 定数仕様

| パラメータ | 値 | 説明 |
|-----------|-----|------|
| 総ペイロード | 40 MiB | 総送信量 |
| チャンクサイズ | 64 KiB | チャンクあたりのサイズ |
| チャンク間隔 | 500 ms | スローディップ遅延 |
| 総チャンク数 | 640 | 40 MiB / 64 KiB |
| 総所要時間 | 約 320 秒 | 640 × 500ms |
| 同時実行ゲート | `AtomicI32` / `Interlocked`, 最大 32 | 最大同時 tarpit 数 |

### 各言語の実装

| 言語 | ファイルパス | クラス / モジュール |
|------|------------|-------------------|
| Rust | `saki-ssh-daemon/src/tarpit.rs` | `OnceLock` 静的 entropy バッファ |
| Go | `go-sakissh/internal/server/tarpit.go` | Goroutine ベースのスローディップ |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/TarpitBuffer.cs` | `TarpitBuffer` |
| Swift | N/A | Daemon 専用プラグイン |

---

## C.4 — ED25519 ハッシュチェーン監査ログ

**RFC アンカー**: `ed25519-audit`

### 定数仕様

| パラメータ | 値 | 説明 |
|-----------|-----|------|
| Genesis シード | `"SASS_GENESIS_BLOCK"` | 最初のレコードの chain_hash シード |
| タイムスタンプ形式 | RFC 3339 | タイムスタンプ形式 |
| イベント形式 | JSON | 構造化イベントデータ |

### 各言語の実装

| 言語 | ファイルパス | クラス / モジュール |
|------|------------|-------------------|
| Rust | `saki-ssh-daemon/src/audit.rs` | ED25519 ハッシュチェーンログ |
| Go | `go-sakissh/internal/server/audit.go` | 監査チェーン |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/AuditChain.cs` | `AuditChain` |
| Swift | `SakiAgentSSH-Client/Sources/Plugins/AuditVerifier.swift` | `AuditVerifier` |

---

## C.5 — Vi Swap ANSI エスケープシーケンス

**RFC アンカー**: `vi-swap-ansi`

### 定数仕様

| パラメータ | 値 | 説明 |
|-----------|-----|------|
| 最大保持時間 | 3600 秒 | max_hold |
| ハートビート間隔 | 5 秒 | heartbeat |

### 各言語の実装

| 言語 | ファイルパス | クラス / モジュール |
|------|------------|-------------------|
| Rust | `saki-ssh-daemon/src/tarpit.rs` (vi_swap) | Vi Swap ANSI エスケープ |
| Go | `go-sakissh/internal/server/v6_integration.go` | Vi Swap |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/ViSwap.cs` | `ViSwap` |
| Swift | N/A | Daemon 専用プラグイン |

---

## C.6 — 透過的ブランチング (Symlink Tree)

**RFC アンカー**: `symlink-tree`

### 定数仕様

| パラメータ | 値 | 説明 |
|-----------|-----|------|
| 除外ディレクトリ | `target/`, `.git/`, `node_modules/` | シンボリックリンクしないディレクトリ |

### 各言語の実装

| 言語 | ファイルパス | クラス / モジュール |
|------|------------|-------------------|
| Rust | `saki-ssh-daemon/src/branch_mgr.rs` | Symlink tree マイクロブランチ |
| Go | `go-sakissh/internal/server/branch_mgr.go` | ブランチマネージャー |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/BranchManager.cs` | `BranchManager` |
| Swift | N/A | Daemon 専用プラグイン |

---

## C.7 — 揮発性キャッシュリダイレクション (EnvInjector)

**RFC アンカー**: `volatile-cache`

### 定数仕様 (POSIX)

| 検出ツール | 環境変数 | リダイレクト先 |
|-----------|---------|---------------|
| npm/yarn/pnpm | `npm_config_cache` | `/tmp/sass_vol/npm` |
| npm/yarn/pnpm | `YARN_CACHE_FOLDER` | `/tmp/sass_vol/yarn` |
| cargo/rustc | `CARGO_TARGET_DIR` | `/tmp/sass_vol/ct` |
| cargo/rustc | `CARGO_HOME` | `/tmp/sass_vol/ch` |
| pip | `PIP_CACHE_DIR` | `/tmp/sass_vol/pip` |
| (全コマンド) | `TMPDIR` | `/tmp/sass_vol/tmp` |

### 各言語の実装

| 言語 | ファイルパス | クラス / モジュール |
|------|------------|-------------------|
| Rust | `saki-ssh-daemon/src/env_injector.rs` | 環境変数揮発性キャッシュ注入 |
| Go | `go-sakissh/internal/server/env_injector.go` | Env injector |
| C# | `windows-daemon-csharp/SakiSshDaemon.Plugins/EnvInjector.cs` | `EnvInjector` |
| Swift | `SakiAgentSSH-Client/Sources/Plugins/EnvInjectorClient.swift` | `EnvInjectorClient` |

---

*Saki Studio · SASS v1.4 · draft-sakistudio-sass-05*
