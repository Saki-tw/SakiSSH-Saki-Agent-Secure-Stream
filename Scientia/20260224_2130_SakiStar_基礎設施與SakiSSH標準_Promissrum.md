# 🌌 SakiStar 跨機基礎設施與安全架構標準 (Promissrum)

> **時間**：2026-02-24 21:30 (UTC+8)
> **級別**：專案標準 (Project Standard)
> **驅動哲學**：SakiConsultumUltimum

本標準奠定 Saki Studio 跨平台分散式計算（M1 ↔ Windows）的終極設計原則。徹底釐清帳號權限、資源浪費、服務共用與連線穩定性的根本解法。

---

## 💡 核心典範轉移

**舊思維**：將 Windows 視為供人登入的「伺服器工作站」，依賴 OpenSSH、Per-user 環境（如 WSL/AppX）。
**新典範**：**全機構皆為 Agent 控制域。** `saki`、`daubl`、`workASUS` 皆為 M1 LLM（大語言模型）的自動化執行身份，而非互斥的「不同人類」。

---

## 1. 資源管控：拔除全機構 Git 監控

由於本機構程式碼版本控制均由 M1 LLM 代管，**人類不直接操作 Git**。

### 🚨 痛點與解法
IDE（如 VS Code）預設的安全 Git Watcher 會對每一個開啟的資料夾進行密集的檔案 I/O 掃描。在擁有多節點、數萬個檔案（如 npm/cargo node_modules）的環境下，這是無意義的 CPU 與電力浪費。

### ✅ 標準實作
1. **全域禁用**：在所有機器的 VS Code `settings.json` 強制寫入：
   ```json
   "git.enabled": false,
   "git.autorefresh": false,
   "git.decorations.enabled": false
   ```
2. Agent 在操作 `git diff` 時，直接使用原生 `git` CLI，不受 IDE 影響。

---

## 2. 終結權限斷層：SakiSSH 的戰略本質

### 🚨 重大事實釐清
- **Windows 的 帳號皆為 Agent 的手足**：`daubl` (有 admin，無本機密碼) = 負責**「部署」**的 Agent 身份；`saki` (懸空 Users，有密碼) = 負責**「執行」**的 Agent 身份。
- **OpenSSH 是安全漏洞與功能斷層**：它迫使我們必須使用密碼登入 `saki`，卻把我們關在無法安裝軟體、無法直接觸發 `winget` 來源更新的牢籠。

### ✅ SakiSSH 作為「M1 神經元延伸」的標準
SakiSSH 不再是「一個更好的 OpenSSH」，而是**「M1 在 Windows 上的常駐代理人 (Daemon)」**。

| 設計 | 說明 |
|------|------|
| **運行身份** | 作為 Windows 系統服務，以 **`LocalSystem`** 運行。 |
| **權限穿透** | `LocalSystem` 擁有比 `daubl` 更高的本機權限，可**直接全域安裝/升級** любые winget 套件，Agent 不再受限於 `saki` 懸空帳號。 |
| **無狀態執行** | 不開啟 cmd/conpty，M1 Agent 透過 gRPC/TCP 直接觸發 `C:\Program Files\nu\bin\nu.exe -c "cargo build"`，回傳純淨 UTF-8。 |
| **單向憑證** | M1 透過 mTLS 憑證與 SakiSSH 連線，完全繞過 Windows 帳號密碼登入（沒有輸入密碼的環節 = 全自動化）。 |

---

## 3. 工具鏈與 Linux 基礎設施 (WSL2) 的全域共用

### 🚨 WSL2 的原罪：Per-User 隔離
在傳統框架下，`wsl --install` 是綁定當前使用者的（登錄檔 + 虛擬磁碟放在 `%USERPROFILE%\AppData`）。我們曾因此將 Ubuntu 砍除，這導致 `saki` 和 `daubl` 無法看到同一個 Linux 系統。

### ✅ 全域共用標準：Tarball Import 模式
放棄市集 (Store App) 的安裝法。WSL 必須被當作「全域容器」佈署：

1. **從根源共用**：
   在 `daubl` 身份下，將 Ubuntu 24.04 映像檔安裝到**全系統可見的路徑**（例如 `C:\WSL\Ubuntu2404`）。
   ```powershell
   # 1. 下載 rootfs
   # 2. 匯入為全域 WSL (資料存於 C:\WSL)
   wsl --import SAKI-Ubuntu C:\WSL\Ubuntu2404 .\ubuntu-24.04-server-cloudimg-amd64-wsl.rootfs.tar.gz
   ```
2. **免密碼執行**：
   由於 SakiSSH 以 `LocalSystem` 運行，它可以直接呼叫 `wsl.exe -d SAKI-Ubuntu`，徹底粉碎 `daubl` 和 `saki` 之間的看門狗。
3. **消除 Winget `--scope machine` 的痛點**：
   `saki` 無法安裝軟體的死結被繞過。SakiSSH 會直接調用 `winget install ... --scope machine`，保持全機器 Node/Rust/Nu 版本單一，不用再管哪個帳號裝了什麼。

---

## 4. Tailscale 雙向憑證存取架構重構

### 🚨 盲點
之前以為「Loser 網路三線全掛」，實情是：Loser 透過 IPv4/TS 是活著的，只是 WSL 內的 TS 掛了。如果我們推進到全域 WSL 與 SakiSSH 架構，根本不需要在 WSL 內單獨裝 Tailscale！

### ✅ 單層 Overlay 網路標準
1. **Host-Only 網路出口**：
   Loser PC 與 Trading PC 只允許啟動**唯一一個 Windows Host 層級的 Tailscale**。
2. **Agent 登入驗證**：
   SakiSSH 服務監聽 `0.0.0.0`，但內建 ACL，只允許來自 M1 的 Tailscale IP（`100.119.71.51`）或 LAN IP（`192.168.50.124`）連線入站。
3. **退役 WSL TS**：
   正式放棄「讓 saki 在 WSL 裡搞 TS」的複雜架構。WSL 只是純粹的計算容器（用來跑 Linux Node.js/Hugo），不負責對外網路通訊。

---

## ⚡ 總結與推進

SakiSSH 不只是一個工具，它是**「終結 Windows 用戶模型阻礙 AI 自動化的終極解方」**。
在它的實作完成前，Loser PC 可以暫時繼續透過現有的 `ssh saki@loser` 與 `--scope machine` 的 Nushell 撐著，但所有新基礎設施的發展方向，都將以這份典範為標竿。
