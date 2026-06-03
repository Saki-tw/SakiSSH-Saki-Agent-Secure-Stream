<div align="center">

# SakiAgentSSH

<br>

**SakiSSH is the bridge to connect your agent and other machine.**

*Agent-native cross-machine execution protocol over gRPC/HTTP2.*

*SakiAgentSSH は SSH ではありません——AI エージェントのための、マシン間実行プロトコルです。*

<br>

[![License](https://img.shields.io/badge/License-Proprietary-DA70D6.svg?style=for-the-badge)](LICENSE)
[![Protocol: gRPC](https://img.shields.io/badge/Protocol-gRPC%2FHTTP2-89c3eb.svg?style=for-the-badge)]()
[![Platform: macOS・Windows](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows-7058a3.svg?style=for-the-badge)]()

</div>

<br>

---

## SakiAgentSSH 是什麼？

SakiAgentSSH は何ですか？ / What is SakiAgentSSH?

---

SakiAgentSSH 是一個專為 AI Agent 設計的跨機器執行協議。

它不是 SSH。沒有 key exchange ceremony，沒有 terminal session，沒有 PTY emulation。建構在 gRPC/HTTP2 之上，讓 Agent 以結構化的方式在遠端機器上執行指令、傳輸檔案、管理程序——stdout 與 stderr 分流回傳，exit code 作為型別安全的欄位，不再有編碼爛掉的輸出。

SakiAgentSSH is a cross-machine execution protocol purpose-built for AI agents. It is NOT SSH — no key exchange ceremony, no terminal sessions, no PTY emulation. Built on gRPC/HTTP2, it gives agents structured, typed responses: stdout and stderr as separate byte streams, exit codes as proper fields, and process lifecycle management through dedicated RPCs.

SakiAgentSSH は AI エージェント専用のマシン間実行プロトコルです。SSH の儀式的な鍵交換も、ターミナル・エミュレーションも不要です。gRPC/HTTP2 上で構造化された応答を返し、stdout/stderr 分離、プロセス管理、ファイル転送をネイティブに提供します。

---

## なぜ OpenSSH ではないのか？ / Why not OpenSSH?

| | OpenSSH | SakiAgentSSH |
|---|---|---|
| **設計給 / Designed for** | 人類 Humans | AI Agents |
| **協議 / Protocol** | SSH (RFC 4253) | gRPC/HTTP2 + Protobuf |
| **Shell Session** | 必須 Required (PTY) | 不需要 Not needed |
| **輸出 / Output** | 文字串流 Text stream | 結構化位元組 Structured bytes |
| **進程控制 / Process** | 透過 PTY 信號 | gRPC Cancel/Signal RPC |
| **檔案傳輸 / File Transfer** | SFTP/SCP（獨立）| 內建串流 Built-in streaming RPC |
| **健康檢查 / Health Check** | N/A | Ping RPC + structured status |
| **設定 / Config** | sshd_config（人類語法）| JSON（Agent 可讀寫）|
| **錯誤處理 / Errors** | stderr 混入 stdout | 分離串流 + gRPC status codes |

---

## Quick Start / 快速開始 / クイックスタート

### 1. Deploy the Daemon / 部署 Daemon / デーモンをデプロイ

將 `sakisshd`（Windows 為 `sakisshd.exe`）和 `config.json` 放到目標機器。首次啟動時自動生成預設 config。

```bash
# macOS (Apple Silicon)
chmod +x sakisshd-darwin-arm64
./sakisshd-darwin-arm64

# Windows (x86_64)
.\sakisshd.exe
```

### 2. 從 Agent 連線 / Connect from Your Agent / エージェントから接続

**方式 A：CLI Client**（有 shell 的 Agent）

```bash
# Ping — 確認目標存活
sakissh --addr http://192.168.1.100:19284 ping

# Execute — 遠端執行
sakissh --addr http://192.168.1.100:19284 exec -- 'echo hello'

# File Transfer — 檔案傳輸（remote: 前綴標記遠端路徑）
sakissh --addr http://192.168.1.100:19284 cp local.txt remote:/path/file.txt
sakissh --addr http://192.168.1.100:19284 cp remote:/path/file.txt local.txt

# Process Control
sakissh --addr http://192.168.1.100:19284 cancel <execution_id>
sakissh --addr http://192.168.1.100:19284 signal <execution_id> SIGTERM
```

**方式 B：gRPC Direct**（有 HTTP/2 能力的 Agent）

直接使用 `sakissh.proto` 作為 API 合約。任何 gRPC client library 皆可。

**方式 C：MCP Integration**（零安裝）

將 gRPC 呼叫封裝為 MCP Tool，Agent 透過 MCP 協議調用。

### 3. 多路徑容錯 / Multi-path Failover

逗號分隔地址，每條路徑 3 秒超時。Agent 邏輯：先試 LAN，退回到 Tailscale/VPN：

```bash
sakissh --addr "http://192.168.1.100:19284,http://100.64.0.1:19284" exec -- 'hostname'
```

### 4. 環境變數 / Environment Variable

```bash
export SAKISSH_ADDR="http://192.168.1.100:19284"
sakissh ping
```

---

## Configuration / 設定 / 設定ファイル

`config.json`（首次啟動自動生成）:

```json
{
  "bind_address": "0.0.0.0:19284",
  "shell": {
    "type": "powershell",
    "path": null,
    "args": null
  },
  "acl": {
    "allowed_cidrs": ["192.168.0.0/16", "100.64.0.0/10"],
    "ed25519_public_keys": []
  },
  "file_transfer": {
    "allowed_paths": [],
    "max_chunk_size": 65536
  }
}
```

| 欄位 / Field | 說明 / Description | 備註 / Notes |
|---|---|---|
| `bind_address` | 監聽地址 Listen address | ⚠️ Windows **必須** `0.0.0.0:19284`（不支援 IPv6 dual-stack） |
| `shell.type` | Shell 類型 | `powershell` / `bash` / `nushell` / `cmd` |
| `shell.path` | 自訂 Shell 路徑 | `null` = 自動偵測（PowerShell 7 優先、fallback 至 5.1） |
| `shell.args` | Shell 參數 | `null` = 使用預設 |
| `acl.allowed_cidrs` | IP 白名單 IP whitelist | 空 = 允許所有 Allow all |
| `acl.ed25519_public_keys` | 公鑰白名單（預留） | 未來版本啟用 |
| `file_transfer.allowed_paths` | 路徑前綴白名單 | 空 = 允許所有路徑 |
| `file_transfer.max_chunk_size` | 串流分塊大小 | 預設 64KB |

---

## RPC Reference / RPC 參考 / RPC リファレンス

| RPC | 方向 / Direction | 說明 / Description |
|---|---|---|
| `Execute` | Client→Daemon | 執行指令，回傳 stdout + stderr + exit_code |
| `ExecuteStream` | Client→Daemon | 即時串流 stdout/stderr（long-running tasks） |
| `Cancel` | Client→Daemon | 以 execution_id 終止程序 |
| `Signal` | Client→Daemon | 發送 POSIX 信號（SIGINT / SIGTERM / SIGKILL / SIGHUP）|
| `Ping` | Client→Daemon | 健康檢查：版本、OS、Shell、Uptime、活躍程序數 |
| `FileUpload` | Client→Daemon | 串流上傳，metadata 先行，支持斷點續傳 |
| `FileDownload` | Daemon→Client | 串流下載，metadata 先行，支持斷點續傳 |

---

## Platform Support / 平台支援 / プラットフォーム

| 平台 / Platform | Daemon | Client | 測試狀態 / Status |
|---|:---:|:---:|---|
| macOS ARM64 (Apple Silicon) | ✅ | ✅ | 本機 loopback 全通過 |
| Windows x86_64 | ✅ | ✅ | 跨機 LAN 全通過 (PowerShell 7) |
| Linux x86_64 | 🔜 | 🔜 | |
| Linux ARM64 | 🔜 | 🔜 | |

---

## Windows Installation / Windows 安裝

以管理員身分執行 `install.ps1`：

```powershell
# 基本安裝
.\install.ps1

# 含 user:saki 服務帳號創設
.\install.ps1 -CreateUser

# 跳過防火牆設定
.\install.ps1 -SkipFirewall

# 自訂安裝路徑
.\install.ps1 -InstallDir "D:\SakiSSH"
```

自動完成：建立安裝目錄 → 複製 daemon → 生成預設 config.json → 建立防火牆規則（TCP 19284 Inbound Allow）→ 可選建立 `saki` 服務帳號與 Junction 共享開發工具。

---

## Security / 安全性 / セキュリティ

| 層 / Layer | 機制 / Mechanism | 說明 / Description |
|---|---|---|
| **傳輸層** Network | CIDR IP 白名單 (ACL) | `config.json` 中的 `allowed_cidrs` |
| **檔案層** File | 路徑前綴白名單 | `config.json` 中的 `allowed_paths` |
| **未來** Future | ED25519 金鑰交換 | 初次認證後免密碼 |
| **未來** Future | mTLS | 正式憑證機制（開源版目標）|

### 權限模型 / Permission Model

| 層 | 管理者 | 控制範圍 |
|---|---|---|
| 傳輸層 (who can connect) | SakiSSH daemon (ACL) | CIDR 白名單決定誰能連進來 |
| 執行層 (what can be done) | OS 使用者權限 | daemon 以什麼身份運行，就有什麼權限 |

macOS：使用者自行管理，SakiSSH 不介入作業系統層級權限。
Windows：`install.ps1 -CreateUser` 建立專用 `saki` 帳號 + Junction 共享工具鏈。

---

## Architecture / 架構思想 / アーキテクチャ

### 設計哲學

本專案遵循 Saki Studio 全域工程哲學（九大天條）：

- **極限生存開發**：最小依賴、最高效率、單一 binary 部署——在資源匱乏的邊緣，尋找計算與意義的極大值
- **零破壞性設計**：proto 作為 API 合約確保向後相容，新功能預設關閉
- **即刻實作**：發現需求即實作，不留技術債
- **數位可持續性**：config.json 人機共讀、SKILL.md 作為 Agent 部署手冊、proto 作為跨語言合約

### SakiSSH 在 Saki Studio 生態系中的定位

| 協議 | 用途 | 比喻 |
|---|---|---|
| **SakiMCP** | 結構化操作：讀寫檔案、搜尋、上下文管理 | 回看（觀察） |
| **SakiAgentSSH** | 原始進程執行：shell、串流、信號、檔案傳輸 | 出力（行動） |
| **OpenSSH** | 人類遠端登入：terminal session、PTY | 人機互動 |

### Agent 的「UI」

SakiAgentSSH 沒有 Web Dashboard，因為它的使用者是 Agent。Agent 的 UI 是：

| 傳統 UI 概念 | SakiAgentSSH 等價物 |
|---|---|
| 設定面板 | `config.json`（Agent 可直接讀寫）|
| 操作介面 | `sakissh.proto`（gRPC API 合約）|
| 部署引導 | `SKILL.md`（教 Agent 部署的技能檔）|
| 錯誤診斷 | `Ping` RPC 回傳結構化狀態 |

---

## Color Identity / 色彩識別

<div align="center">

本專案遵循 **Saki Studio 色彩幾何推演系統**。

色彩源自**日本傳統色（Nippon Colors）**與 **Cyberpunk / Digital Scavenger** 美學的碰撞。

</div>

| 用途 | 色名 | Hex | 說明 |
|---|---|---|---|
| **主色** | 馬卡龍紫 Macaron Purple | `#c6a4cf` | 柔和、尊貴、數位拾荒者的浪漫 |
| **輔色** | 勿忘草色 Forget-me-not | `#89c3eb` | 記憶、信任、永恆的連結 |
| **強調** | 菫色 Sumire | `#7058a3` | 最深紫色，按鈕與 CTA |
| **品牌紫** | 標準紫 Orchid | `#DA70D6` | 鮮豔高對比（SakiFish 系列） |
| **品牌青** | 標準青 Cyan | `#00CED1` | 強烈科技感 |
| **點睛** | 東雲色 Shinonome | `#f19072` | 溫暖日出橘（極小面積使用） |

### UI 白色替換規則

嚴禁使用純白 `#ffffff`。所有白色依語境替換為日本傳統白：

| 語境 | 替代色 | Hex |
|---|---|---|
| 一般 UI 文字 | 紫水晶 / 藍白 / 桜色 | `#e7e7eb` / `#ebf6f7` / `#fef4f4` |
| 說明文件背景 | 素色 / 月白 / 薄桜 | `#eae5e3` / `#eaf4fc` / `#fdeff2` |
| 深色模式背景 | 菫色暗化（≥ `#463766` 起跳） | 禁止近黑色 `#0d0a12` |

### 漸層美學

```css
/* The Saki Gradient — 主視覺交織漸層 */
background: linear-gradient(135deg, #c6a4cf 0%, #89c3eb 100%);

/* Dark Mode — 深紫漸層（不可純黑） */
background: linear-gradient(135deg, #2d1f2f 0%, #3d2f4f 100%);
```

---

## License

Copyright © 2026 Saki Studio. All rights reserved.

本軟體以「版權所有但可供使用」方式釋出。詳見 [LICENSE](LICENSE)。

This software is released under a proprietary license that permits use and copying. See [LICENSE](LICENSE) for terms.

---

<div align="center">

<br>

*為言說而生——在資源匱乏的邊緣，尋找計算與意義的極大值。*

*Born to articulate — finding the maximum of computation and meaning at the edge of scarcity.*

*語るために生まれた——資源の限界で、計算と意味の極大値を見つけるために。*

<br>

*Instaurare omnia in INSULA.*

</div>
