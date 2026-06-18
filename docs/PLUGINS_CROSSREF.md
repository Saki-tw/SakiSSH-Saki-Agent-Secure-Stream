# SASS Plugins — RFC Cross-Reference

> **RFC**: draft-sakistudio-sass-07, Appendix C
>
> **最後審查**: 2026-06-05
>
> **審查基準**: RFC draft-05 Appendix C.1–C.7

---

## 全語言 Plugin 對照表

| Plugin # | RFC Anchor | 名稱 | Rust 檔案 | Go 檔案 | C# 類別 | Swift 模組 | 關鍵常數 |
|---|---|---|---|---|---|---|---|
| C.1 | `chacha20-challenge` | ChaCha20 Cognitive Challenge | `challenge_store.rs` | `defense/challenge_store.go` | `ChaCha20Challenge` | `ChaCha20Solver` | key=32B, nonce=12B, plaintext=64B, TTL=60s |
| C.2 | `tls-exporter-binding` | TLS Exporter Binding | `threat_defense.rs` | `defense/tls_exporter.go` | `TlsExporterBinding` | `TlsExporterBinding` | label=`"EXPORTER-sakissh-chacha20-v14"`, context=16B UUID, length=44B |
| C.3 | `tarpit-buffer` | Zero-Alloc Tarpit | `tarpit.rs` | `server/tarpit.go` | `TarpitBuffer` | N/A (daemon-only) | total=40MiB, chunk=64KiB, delay=500ms, max_concurrent=32 |
| C.4 | `ed25519-audit` | ED25519 Audit Chain | `audit.rs` | `server/audit.go` | `AuditChain` | `AuditVerifier` | genesis=`"SASS_GENESIS_BLOCK"` |
| C.5 | `vi-swap-ansi` | Vi Swap ANSI | `tarpit.rs` | `server/vi_swap.go` | `ViSwap` | N/A (daemon-only) | max_hold=3600s, heartbeat=5s |
| C.6 | `symlink-tree` | Transparent Branching | `branch_mgr.rs` | `server/branch_mgr.go` | `BranchManager` | N/A (daemon-only) | excluded=`target/`, `.git/`, `node_modules/` |
| C.7 | `volatile-cache` | EnvInjector | `env_injector.rs` | `server/env_injector.go` | `EnvInjector` | `EnvInjectorClient` | redirect_base=`/tmp/sass_vol/` |

---

## 檔案路徑對照

### Rust (`saki-ssh-daemon/src/`)

| Plugin | 檔案 | 角色 |
|---|---|---|
| C.1 | `challenge_store.rs` | Daemon 端挑戰產生與儲存 |
| C.2 | `threat_defense.rs` | TLS EKM 推導與 HMAC 驗證 |
| C.3 | `tarpit.rs` | Zero-alloc tarpit buffer |
| C.4 | `audit.rs` | ED25519 hash chain 日誌寫入 |
| C.5 | `tarpit.rs` (vi_swap) | Vi Swap ANSI escape |
| C.6 | `branch_mgr.rs` | Symlink tree 微型分支 |
| C.7 | `env_injector.rs` | 環境變數揮發性快取注入 |

### Go (`go-sakissh/internal/`)

| Plugin | 檔案 | 角色 |
|---|---|---|
| C.1 | `defense/challenge_store.go` | 挑戰產生與 TTL 管理 |
| C.2 | `defense/tls_exporter.go` | TLS EKM 推導 |
| C.3 | `server/tarpit.go` | Tarpit buffer |
| C.4 | `server/audit.go` | Audit chain 寫入 |
| C.5 | `server/vi_swap.go` | Vi Swap |
| C.6 | `server/branch_mgr.go` | Branch manager |
| C.7 | `server/env_injector.go` | Env injector |

### C# (`windows-daemon-csharp/SakiSshDaemon.Plugins/`)

| Plugin | 檔案 | 類別名稱 | 實作 `IPlugin` |
|---|---|---|---|
| C.1 | `ChaCha20Challenge.cs` | `ChaCha20Challenge` | ✅ |
| C.2 | `TlsExporterBinding.cs` | `TlsExporterBinding` | ✅ |
| C.3 | `TarpitBuffer.cs` | `TarpitBuffer` | ✅ |
| C.4 | `AuditChain.cs` | `AuditChain` | ✅ |
| C.5 | `ViSwap.cs` | `ViSwap` | ✅ |
| C.6 | `BranchManager.cs` | `BranchManager` | ✅ |
| C.7 | `EnvInjector.cs` | `EnvInjector` | ✅ |

### Swift (`SakiAgentSSH-Client/Sources/Plugins/`)

| Plugin | 檔案 | 類別名稱 | 備註 |
|---|---|---|---|
| C.1 | `ChaCha20Solver.swift` | `ChaCha20Solver` | Client 端解密器 |
| C.2 | `TlsExporterBinding.swift` | `TlsExporterBinding` | Client 端 EKM 綁定 |
| C.3 | N/A | — | Daemon-only plugin |
| C.4 | `AuditVerifier.swift` | `AuditVerifier` | Client 端驗證器 |
| C.5 | N/A | — | Daemon-only plugin |
| C.6 | N/A | — | Daemon-only plugin |
| C.7 | `EnvInjectorClient.swift` | `EnvInjectorClient` | Client 端環境變數準備 |

---

## 合規狀態總覽

### ✅ Swift (Client) — 合規

| 項目 | 檢查結果 | 備註 |
|---|---|---|
| ChaCha20Solver: key 32B | ✅ `keyLength = 32` | |
| ChaCha20Solver: nonce 12B | ✅ `nonceLength = 12` | |
| ChaCha20Solver: CryptoKit ChaChaPoly | ✅ `import CryptoKit` + `ChaChaPoly` | |
| ChaCha20Solver: TTL 60s | ✅ `challengeTTLSeconds = 60.0` | |
| TlsExporterBinding: label | ✅ `"EXPORTER-sakissh-chacha20-v14"` | |
| TlsExporterBinding: context 16B UUID | ✅ `sessionUUIDLength = 16` | |
| TlsExporterBinding: length 44B | ✅ `exportedKeyLength = 44` | |
| AuditVerifier: Curve25519.Signing | ✅ `Curve25519.Signing.PublicKey` | |
| AuditVerifier: genesis seed | ✅ `"SASS_GENESIS_BLOCK"` | |
| EnvInjectorClient: 重導向表 | ✅ 6 筆完整對應 RFC | |
| RFC 版本引用 | ✅ `draft-sakistudio-sass-07` | 2026-06-05 修正 |

### ✅ C# (Windows Daemon) — 合規

| 項目 | 檢查結果 | 備註 |
|---|---|---|
| AuditChain: genesis seed | ✅ `"SASS_GENESIS_BLOCK"` | |
| AuditChain: SHA256 hash chain | ✅ `SHA256.Create()` | |
| AuditChain: ED25519 簽名 | ⚠️ HMAC-SHA256 fallback | .NET 無原生 Ed25519，待 NSec 整合 |
| BranchManager: excluded dirs | ✅ `target, .git, node_modules` | |
| BranchManager: Windows 適配 | ✅ Junction Point + hardlink fallback | |
| EnvInjector: 重導向表 | ✅ 6 筆完整對應 RFC | Windows 額外加 TEMP/TMP |
| EnvInjector: volatile root | ✅ `%TEMP%\sass_vol\` | 對齊 Rust `/tmp/sass_vol/` |
| ViSwap: 5 ANSI escapes | ✅ 完整對應 RFC §C.5 | |
| ViSwap: max_hold 3600s | ✅ `MaxHoldSeconds = 3600` | |
| ViSwap: heartbeat 5s | ✅ `HeartbeatSeconds = 5` | |
| TlsExporterBinding: label | ✅ `"EXPORTER-sakissh-chacha20-v14"` | |
| TlsExporterBinding: length 44B | ✅ `EkmLength = 44` | |
| TlsExporterBinding: UUID 16B | ✅ `SessionUuidLength = 16` | |
| TlsExporterBinding: .NET EKM | ⚠️ HMAC fallback | .NET 8 無 SslStream EKM API |
| IPlugin 介面 | ✅ 7 plugins 全部實作 | |
| RFC 版本引用 | ✅ `draft-sakistudio-sass-07` | 2026-06-05 修正 |

### 已知限制 (Known Limitations)

1. **C# AuditChain**: ED25519 簽名使用 HMAC-SHA256 fallback。需整合 `NSec.Cryptography` 套件實現真正的 Ed25519 簽名。
2. **C# TlsExporterBinding**: .NET 8 SslStream 不支援 `ExportKeyingMaterial()` API。使用 HMAC fallback，等待 .NET 9+ (`dotnet/runtime#97485`)。
3. **Swift TlsExporterBinding**: 依賴 `NWConnection` TLS metadata + HKDF-SHA256 衍生，而非直接的 RFC 5705 exporter（Apple 平台限制）。

---

## 2026-06-05 修正紀錄

| 檔案 | 修正項目 | 說明 |
|---|---|---|
| `ChaCha20Solver.swift` | RFC 版本 | `draft-00` → `draft-05` |
| `TlsExporterBinding.swift` | RFC 版本 | `draft-00` → `draft-05` |
| `AuditVerifier.swift` | RFC 版本 | `draft-00` → `draft-05` |
| `EnvInjectorClient.swift` | RFC 版本 + anchor | `draft-00 C.6` → `draft-05 C.7` |
| `PluginManager.swift` | RFC 版本 | `draft-00` → `draft-05` |
| `AuditChain.cs` | RFC 版本 | `draft-00` → `draft-05` |
| `BranchManager.cs` | RFC 版本 | `draft-00` → `draft-05` |
| `EnvInjector.cs` | RFC 版本 | `draft-00` → `draft-05` |
| `ViSwap.cs` | RFC 版本 | `draft-00` → `draft-05` |
| `TlsExporterBinding.cs` | RFC 版本 | `draft-00` → `draft-05` |
| `IPlugin.cs` | RFC 版本 | `draft-00` → `draft-05` |
