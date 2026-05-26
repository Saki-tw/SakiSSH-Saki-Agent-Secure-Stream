# SakiSSH 深度架構分析（v2.0）：Windows 懸空用戶、OpenSSH 安全邊界與套件共用

> **研究時間**：2026-02-24 21:10 (UTC+8)
> **驅動哲學**：SakiConsultumUltimum (數位可持續性)
> **前置**：取代 v1.0（草率版），整合機構全域約束
> **相關 Scientia**：SSH 公鑰修復 (20260220)、WSL/Nushell 評估 (20260220)、Loser 架構重構 (20260224)

---

## 1. 全貌：M1 的控制面拓撲

M1 Mac Mini 是整個基礎設施的唯一「大腦」。它同時擁有：

| 介面 | IP | 角色 |
|------|-----|------|
| en0 (WiFi 5G) | 192.168.50.124 | 內網開發（Hugo, Git, SSH to Loser/Trading） |
| ppp0 (PPPoE) | 218.161.13.103 | **公網出口**，Cloudflare Tunnel 經此出 |
| Tailscale | 100.x.x.x | VPN fallback |

**Cloudflare Tunnel** 從 ppp0 出去，將 5 個域名對接回 M1 的 localhost 服務。
**SSH 出站** 從 en0 或 Tailscale 出去，連向 Loser 或 Trading。

當 M1 透過 SSH 對 Loser/Trading 下達編譯指令時，整個鏈路為：

```
M1 Agent / remote-build.sh
  ↓ SSH (IPv4 / IPv6 / Tailscale failover)
Windows OpenSSH Server (sshd_config)
  ↓ 啟動 shell (cmd.exe / nu.exe)
    ↓ 執行指令 (cargo / npm / hugo)
      ↓ stdout/stderr → SSH tunnel → M1 解析
```

**此鏈路的每一層都有可被破壞的斷點。** 以下逐層剖析。

---

## 2. 懸空用戶模型：saki 的真實身份

### 2.1 saki 不是正常使用者

在 Loser PC 與 Trading PC 上，`saki` 帳號具備以下異常特徵：

| 屬性 | 狀態 | 影響 |
|------|------|------|
| Administrators 群組 | ❌ 不是成員 | 無法安裝軟體、修改系統設定 |
| Users 群組 | ❓ 存疑（可能不在） | 若非 Users，則為「懸空帳號」 |
| Home 目錄 | `C:\Users\saki` | 存在，但內含 Junction 指向 daubl |
| .ssh/ 擁有者 | **前 Agent 以 daubl 建立** | ACL Owner = daubl → StrictModes 拒認 |
| 密碼 | 9528（已知） | 可用 `sshpass` 登入修復 |
| 環境變數 PATH | **不繼承 daubl 安裝的工具** | Nu, fd, rg 等全不可見（除非全域安裝） |

### 2.2 daubl 不是正常管理員

| 屬性 | 狀態 | 影響 |
|------|------|------|
| 密碼類型 | **Microsoft Account（雲端密碼）** | 無本機密碼，無法透過 SSH 以密碼登入 |
| 權限 | Administrators | 可安裝軟體、修改系統設定 |
| 存取 saki 的 .ssh/ | 需 `takeown /F` 改 Owner | 改完 Owner 變 daubl，反而破壞 StrictModes |
| 遠端操控性 | **極低** | M1 的 Agent 無法以 daubl 身份 SSH |

### 2.3 Trading PC 的 workASUS 情況類似

- `workASUS` = admin（有 Microsoft 密碼，非本機密碼）
- `saki` = SSH 代理帳號（懸空，同樣的 PATH / ACL 問題）

### 2.4 核心矛盾

> **擁有工具安裝權的 daubl/workASUS 無法被 Agent SSH 控制；**
> **能被 Agent SSH 控制的 saki 沒有任何安裝權。**

這不是技術問題，而是**身份模型的結構性衝突**。

---

## 3. OpenSSH 的「強制互動」安全問題

OpenSSH 本質上是為**人類 SSH 會話**設計的。當作為 Agent 自動化的通道時，存在以下安全性與可靠性衝突：

### 3.1 Host Key 強制互動

首次連線必須人工確認 `yes`（`StrictHostKeyChecking`），無法由 Agent 決策是否安全。
現有解法（`-o StrictHostKeyChecking=no`）**直接關閉安全檢查**，為中間人攻擊敞開大門。

### 3.2 StrictModes 與 ACL 地獄

Windows OpenSSH 的 `StrictModes` 要求 `authorized_keys` 的 Owner 必須是本用戶。
但懸空帳戶 saki **無法自行建立此檔案**（需要 daubl 代建，代建後 Owner 又變 daubl）。
唯一可行解法：`sshpass -p '9528' ssh loser "takeown /F ..."` — 即用**明碼密碼**修復**公鑰認證**。

### 3.3 DefaultShell 陷阱

Windows OpenSSH 的 `DefaultShell` 指向不存在的路徑時直接拒絕全部登入
（daubl 設 PS7 Preview MSIX 路徑就引爆過）。**一個管理員的日常操作就能癱瘓所有 SSH。**

### 3.4 編碼與 PTY

已在 v1.0 評估中陳述。重點是：OpenSSH 層面無解，因為它不是問題的來源，Windows 的 conpty + 混編碼才是。

---

## 4. 套件共用方案：winget `--scope machine` 的現況

### 4.1 已確認可行

```powershell
# 以 daubl (admin) 執行
winget install nushell --scope machine
# → 安裝至 C:\Program Files\nu\bin\nu.exe
# → saki 可存取（Program Files 對所有用戶可讀取）
```

**Loser PC 目前現況**：
- `cargo` → `C:\Users\saki\.cargo\bin\cargo.exe` (per-user, ✅ saki 可存取)
- `nu` → `C:\Program Files\nu\bin\nu.exe` (machine scope, ✅ saki 可存取)

### 4.2 卡住的問題

**saki 無法原始呼叫 `winget` 本身。**

原因：`winget` 是 MSIX 封裝的 App (`Microsoft.DesktopAppInstaller`)，它的 executable 位於：
```
C:\Users\daubl\AppData\Local\Microsoft\WindowsApps\winget.exe
```

這是 **per-user** 的 MSIX registration。`saki` 的 session 根本看不到 `winget` 這個指令。
因此：

- ✅ daubl 可以 `winget install X --scope machine` → 軟體裝進 `C:\Program Files\`
- ✅ saki 可以直接使用已安裝的軟體（如 nu.exe, cargo, node）
- ❌ saki 無法自行呼叫 `winget` 安裝或更新任何東西
- ❌ M1 Agent 透過 SSH (saki session) **永遠無法觸發 winget**

### 4.3 實際解法：一次性 daubl 部署腳本

```powershell
# daubl 本地執行一次（Admin PowerShell）：
winget install nushell --scope machine --accept-package-agreements --accept-source-agreements
winget install OpenJS.NodeJS.LTS --scope machine --accept-package-agreements --accept-source-agreements
winget install Hugo.Hugo.Extended --scope machine --accept-package-agreements --accept-source-agreements
# Rust 已透過 per-user rustup-init 安裝在 saki 名下，不需 machine scope
```

**安裝後，saki 的 PATH 中是否能自動看到這些工具？**

答案是：**不一定。** `winget --scope machine` 安裝後，安裝程式會在 `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment\Path` 中加入路徑。但 saki 的 SSH session 繼承的 PATH 可能不包含這些。

**建議的補救措施**：在 `sshd_config` 或 saki 的 `~\.bashrc` 等效位置（Windows 上通常無效）中，由 `remote-build.sh` 在 SSH 指令前端加入完整路徑。目前 `remote-build.sh` 已經使用了 `NU_EXE='C:\Program Files\nu\bin\nu.exe'` 絕對路徑呼叫，這正是正確做法。

---

## 5. SakiSSH：重新定義為「Agent RPC Daemon」

基於上述所有約束，SakiSSH 不應只是「替代 SSH 的遠端 shell」，而是定義為：

> **一支安裝在 Windows Service 層級的 Rust Daemon，
> 由 daubl 一次性安裝，以 SYSTEM 或 saki 身份運行，
> 透過 gRPC 接收 M1 Agent 的結構化指令，
> 直接呼叫 `nu.exe` 執行並回傳純 UTF-8 串流。**

### 5.1 為什麼 Windows Service？

| 屬性 | OpenSSH (sshd) | SakiSSH (Windows Service) |
|------|---------------|---------------------------|
| 安裝者 | daubl (admin) | daubl (admin) — 一次性 |
| 運行身份 | SYSTEM → 降權至 saki | 可配為 `LocalSystem` 或 `saki` |
| 認證 | SSH key / password | mTLS 憑證 或 Tailscale AuthKey |
| 編碼保證 | 無（受 conpty/cmd 干擾） | **保證 UTF-8**（直接讀 process stdout） |
| 套件依賴 | saki 的 PATH 必須有工具 | SakiSSH 知道工具的絕對路徑 |
| 進程管理 | 網路斷 → 殭屍進程 | Rust `Child::kill` + timeout |

### 5.2 認證方案

OpenSSH 的 Host Key 強制互動被人詬病。SakiSSH 可以：

1. **Tailscale 白名單**：SakiSSH 只接受來自 Tailscale 100.x.x.x 網段的連線。因為 Tailscale 本身就是 WireGuard 加密 + 節點認證，所以 SakiSSH 不需要額外的 key exchange。
2. **mTLS**：在首次安裝時由 M1 生成 CA → 發行 Loser/Trading 的 cert。比 SSH key ACL 地獄簡單得多。

### 5.3 架構佈局

```
M1 Mac Mini                              Loser PC (Windows)
┌─────────────────┐                      ┌──────────────────────┐
│ remote-build.sh │  gRPC over           │ SakiSSH Service      │
│   or            │  Tailscale/LAN       │  (Windows Service)   │
│ saki-orchestrator├─────────────────────►│  運行身份: SYSTEM    │
│                 │  ExecuteCommand()     │                      │
│                 │  ← StreamOutput()     │  ┌──────────────┐    │
└─────────────────┘                      │  │ nu.exe       │    │
                                         │  │ cargo.exe    │    │
                                         │  │ node.exe     │    │
                                         │  │ hugo.exe     │    │
                                         │  └──────────────┘    │
                                         └──────────────────────┘
```

### 5.4 套件更新問題的終極解決

SakiSSH 如果以 **SYSTEM** 或 **daubl** 身份運行，它就有權限呼叫 winget（透過完整路徑指定 MSIX App）。
這意味著 M1 Agent 可以透過 SakiSSH 遠端觸發 `winget upgrade --all --scope machine`，
解決了「saki 永遠無法觸發 winget」的死結。

---

## 6. 總結：SakiSSH 的三層效益

| 層次 | 解決的問題 |
|------|-----------|
| **編碼與 PTY** | 繞過 OpenSSH + conpty，直接二進位串流 |
| **安全與認證** | 消除 SSH Key ACL 地獄，以 Tailscale 白名單或 mTLS 替代 |
| **權限與套件** | 以 SYSTEM 運行，跳過 saki 懸空帳號限制，可觸發 winget |

**實作投資**：約 300~500 行 Rust。可內嵌或伴隨 `saki-orchestrator` 部署。
**風險**：需 daubl 做一次性 Windows Service 安裝（`sc create`），之後完全由 M1 控制。
