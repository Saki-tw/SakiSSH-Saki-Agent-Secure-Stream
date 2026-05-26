# TaskMELIUS: SakiAgentSSH ProjectSpeculari Research

**建立時間**：2026-05-14 12:25 (UTC+8)
**專案名稱**：SakiAgentSSH
**目標**：執行 ProjectSpeculari Phase 0 ~ 5 深度研究與歸檔。

## 執行進度
- [x] **Phase 0: 資料源探索**
  - 確認核心端點：`http://{ip}:19284` (gRPC)。
  - 確認依賴庫：Rust (`tonic`, `ed25519-dalek`, `tokio`)，無 C 依賴。
- [x] **Phase 1: 結構掃描**
  - 專案包含 `saki-ssh-daemon` (Rust server), `saki-ssh-client` (Rust CLI), 以及 `SakiAgentSSH-Client` / `SakiAgentSSH-Daemon` (Swift GUI)。
- [x] **Phase 2: 架構分析**
  - 基於 HTTP/2 gRPC 實作的跨機指令執行協定。
  - v3.0 引入了基於 Ed25519 的 SSH 風格認證，達成零 C 依賴的純 Rust 實作。
- [x] **Phase 3: 核心知識提取**
  - 提取了 Agent-native 的執行能力：專為 AI Agent 跨機執行 `run_command` 所設計的封裝。
  - 提取了針對 Windows UAC 提權與 macOS Sandbox 的穿透與權限模型設計。
- [x] **Phase 4: 研究目錄分析**
  - `Scientia/` 包含 20 份報告，詳細記錄了從原生的 SSH 連線不穩定到自主開發 gRPC 替代方案的決策過程與 RFC 草案。
- [x] **Phase 5: 歸檔與報告**
  - 生成 `Scientia/202605141225_SakiAgentSSH_Speculari_Scientia.md`。

## 結論
SakiAgentSSH 解決了 Saki Studio 分散式架構中最核心的「連線穩定度」問題。原生的 SSH 在 Windows 上由於 ACL 與權限問題屢次中斷 Agent 的執行，而 SakiAgentSSH 透過 gRPC + Ed25519 提供了一個專為 Agent 打造的高效、可靠跨機執行橋樑。
