# SakiAgentSSH 專案架構與知識深度分析報告

> **Scientia #21** | 專案逆向與領域知識提取
> **分析時間**：2026-05-14 12:25 (UTC+8)
> **對應工具**：ProjectSpeculari Method

---

## 一、研究邊界與資料源探索 (Phase 0)

SakiAgentSSH 的誕生源於 Saki Studio 在 Windows 節點上遇到的原生 SSH 不穩定性（特別是 `sshd` 的 ACL 與 IPv4Mapped 路由問題），其目標是打造一個專門為 Antigravity AI Agent 服務的跨機執行通道。

**核心資料源與通訊端點**：
- **通訊協議**：基於 HTTP/2 的 gRPC (`http://{ip}:19284`)。
- **認證機制**：摒棄了傳統的 SSH Key 交換，改用純 Rust 實作的 `ed25519-dalek` 進行 SSH 風格的非對稱加密認證，達成零 C 語言依賴。

## 二、專案結構與架構掃描 (Phase 1 & 2)

**模組架構**：
本專案採用典型的 C/S 架構，並輔以 macOS / Windows GUI 封裝：
1. **`saki-ssh-daemon`** (Rust):
   - 基於 `tonic` 實作的 gRPC Server。
   - 作為背景服務運行（Windows 透過 `windows-service`，macOS 透過 LaunchDaemon）。
   - 負責接收命令、驗證 Ed25519 簽章、執行 shell 指令並串流回傳 stdout/stderr。
2. **`saki-ssh-client`** (Rust):
   - 命令列客戶端，作為 Agent 呼叫遠端節點的入口 (`sakissh --addr ... run "..."`)。
3. **`SakiAgentSSH-Client` / `SakiAgentSSH-Daemon`** (Swift):
   - 提供 macOS/iOS 友善的圖形化介面，便於手動管理憑證與守護行程狀態。

## 三、核心知識提取 (Phase 3)

1. **Agent-Native 的執行抽象**
   - 傳統 SSH 在執行互動式命令或長時命令時容易引發 PTY 卡死，SakiAgentSSH 從 gRPC 層面解決了串流控制的問題，提供了符合 Agent 預期的清晰退出碼與獨立 stderr 通道。
2. **零 C 依賴的認證協定**
   - 為了最大化跨平台相容性（特別是避免 Windows 上編譯 OpenSSL/libssh 的痛苦），v3.0 版本自主實作了基於 Ed25519 + SHA2 的挑戰-回應 (Challenge-Response) 認證機制。
3. **跨平台權限模型**
   - **Windows**：Daemon 需註冊為系統服務或以 Administrator 權限運行，確保 Agent 擁有足夠的權限進行編譯與檔案操作。
   - **macOS**：需處理 Sandbox 穿透與 `Full Disk Access`，確保 Daemon 能夠存取整個 `/Users/hc1034/` 空間。

## 四、歷史演進與生態洞察 (Phase 4)

從 `Scientia/` 內的 20 份文檔可以看出本專案的跌宕起伏：
- 初期嘗試修復 Windows OpenSSH 的 ACL 權限問題（2026/02/20 - 2026/02/24）。
- 在歷經多次 SSH 斷連導致 Agent Session 截斷後，決定自主研發 `SakiAgentSSH` (2026/02/28 創世提示詞)。
- v1/v2 版本仍存在邊界限制，至 v3.0 終於確立了「純 Rust、gRPC、Ed25519」的理想型態，並成為 SakiStarCommuncation 多節點編排中最高優先級的健康探針 (Probe)。

## 結論

SakiAgentSSH 是由「痛點」催生的防禦性專案。它成功將不穩定的傳統系統管理工具 (SSH)，替換為現代化、可觀測、專為 AI Agent 設計的 RPC 呼叫協定。它不僅消除了 Windows 節點的環境地雷，也大幅提升了 Agent 執行 `run_command` 時的穩定度。
