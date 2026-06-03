# SakiAgentSSH v0.2.0 協議擴展與跨機部署 — 研究知識

> 建立時間：20260228_0430 (UTC+8)
> 專案：SakiAgentSSH
> 類型：架構決策 + 跨機部署研究

## 一、專案定位決策

**結論**：SakiAgentSSH ≠ SSH。它是 Agent-native 的跨機執行協議。

| 層 | 協議 | 用途 |
|---|---|---|
| SakiMCP | MCP (跨機) | 結構化操作：讀寫檔案、搜尋、上下文管理 |
| SakiAgentSSH | gRPC/HTTP2 | 原始進程執行：shell 指令、串流 stdout、信號管理 |
| OpenSSH | SSH (RFC 4253) | 人類互動：terminal session、PTY |

**核心差異**：
- SakiMCP = 跨機「回看」（讀文件）
- SakiAgentSSH = 跨機「出力」（編譯、部署、重運算）
- OpenSSH = 人類遠端登入

## 二、Windows IPv4/IPv6 Dual-Stack 問題

**發現**：Windows 預設不啟用 IPv6 dual-stack socket。bind `[::0]:19284` 只接受 IPv6 連線。

**修正**：Windows config 必須使用 `0.0.0.0:19284`（IPv4）。

**根因分析**：Linux/macOS 預設 IPv6 socket 可接受 IPv4（dual-stack），Windows 需要明確設定 `IPV6_V6ONLY=0`。Tonic 目前不支援設定此 socket option。

## 三、Orchestrator 覆蓋分析

SakiSSH proto 已完整覆蓋 SakiStarCommunication orchestrator 的核心算力功能：

| 功能 | 覆蓋 RPC |
|---|---|
| 心跳探測 | Ping |
| 任務調度 | Execute/ExecuteStream |
| 任務狀態 | Ping.active_processes |
| 節點列表 | 多路徑 config |

**未覆蓋**：訊息傳遞（上層功能）、Web UI（Presentation layer）→ 列入開源化重大事項。

## 四、Agent 的「UI」= Prompt/Skill

SakiAgentSSH 的「UI」不是 Web Dashboard，而是：
- **SKILL.md** = Agent 的 man page
- **Proto** = Agent 的 API contract
- **Config.json** = Agent 的 Settings panel

## 五、權限模型

| 層 | 管理者 | 控制 |
|---|---|---|
| 傳輸層 | SakiSSH (CIDR ACL) | 誰能連進來 |
| 執行層 | OS (daemon 身份) | 能做什麼 |

macOS：使用者自行管理，不處理權限。
Windows：ps1 安裝腳本處理 user:saki 創設 + Junction。
