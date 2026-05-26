# SakiSSH v1.0.0 原始碼分析 — Nushell 依賴排查

**日期**: 2026-04-07 20:23 (UTC+8)
**專案**: SakiAgentSSH
**標籤**: #SakiSSH #原始碼分析 #v2計劃

## 1. 結論：Nushell 不是硬性依賴

經分析 `saki-ssh-daemon/src/config.rs` 和 `main.rs`（v1.0.0, 1274行, 46KB），確認：

- `config.rs:154-177`：`executable()` 依 config.json `shell.type` 解析路徑，設為 `powershell` 即使用 pwsh.exe
- `main.rs:205-216`：`build_command()` 直接呼叫 `config.shell.executable()`，不 probe 其他 shell
- **daemon 本身不依賴 nushell**

## 2. 真正的問題：進程存活模型

### 2.1 Windows Service 模式（line 1155-1239）
- daemon 已完整實作 Windows Service（SCM 整合, `windows-service` crate）
- `main()`（line 1246-1266）：不帶 `--console` flag → SCM 模式；帶 `--console` → Console 模式
- 排程任務 `\SakiAgentSSH` 不帶 `--console`，但排程任務的環境碰巧能模擬 SCM

### 2.2 gsudo 啟動失敗根因
- `gsudo { Start-Process sakisshd.exe }` 在 gsudo session 結束時，Windows Job Object 殺死所有子進程
- 這不是 nushell 的問題，而是 Windows 進程管理機制

### 2.3 為何卸載 nushell 後表現為無法啟動
- 巧合：卸载 nushell → 用 gsudo Start-Process 重啟 → Job Object 殺掉 → 誤判為 nushell 依賴
- 實際上無論 nushell 是否存在，gsudo Start-Process 的 daemon 都會被殺

## 3. v2 RFC 更新（基於原始碼分析）

### 3.1 已具備但未啟用的能力
- ✅ Windows Service 模式（`windows_svc` module, line 1156）
- ✅ ED25519 認證（`auth.rs`, `capability.rs`）
- ✅ 審計日誌（`audit.rs`）
- ✅ File Upload/Download（streaming gRPC）
- ✅ 進程管理（Cancel, Signal）
- ✅ Session lifecycle（Authenticate, RenewSession, GetCapabilities）

### 3.2 v2 真正需要的改進
1. **`sc create SakiAgentSSH` 自動註冊**（而非手動 schtasks）
2. **config 新增 WSL 原生模式**：daemon 直接 spawn `wsl.exe` 而非經過 shell
3. **file_transfer 路徑擴充**：加入 `D:\Saki_Studio\`
4. **XREADGROUP 增強**：worker script 需先讀 pending 再讀新消息

## 4. 原始碼路徑
- `/Users/hc1034/Saki_Studio/Claude/SakiAgentSSH/saki-ssh-daemon/src/config.rs` (226行)
- `/Users/hc1034/Saki_Studio/Claude/SakiAgentSSH/saki-ssh-daemon/src/main.rs` (1274行)
- 依賴：tonic 0.12, prost 0.13, ed25519-dalek 2, tokio 1
