# Agent 儲存空間邊界限制跨平台研究

> **建立時間**：2026-03-27 22:21 (UTC+8)
> **研究者**：SakiAgentSSH 協議設計研究
> **狀態**：Phase 1 完成
> **標籤**：#Agent安全 #儲存邊界 #沙箱 #gRPC #SSH

---

## 摘要 (Executive Summary)

**結論：有條件可行** — 現有 Agent 的儲存邊界限制機制分為三層：(1) 應用層工具審批 (2) OS 級沙箱 (3) 程序身份隔離。三者均有各自弱點，SakiAgentSSH 協議可在 **傳輸層** 加入 SSH 風格的 channel 權限模型，作為獨立於 Agent 自身安全機制的 **第四層** 限制，達到方法論上有效之邊界控制。

---

## 證據矩陣 (Evidence Matrix)

### 1. Claude Code (Anthropic)

| 維度 | 機制 | 有效性 | 來源 |
|------|------|--------|------|
| 寫入限制 | CWD 限制 — 僅可寫入目前工作目錄及子目錄 | ✅ OS 級強制（Seatbelt/bubblewrap） | [claude.com docs] |
| 讀取限制 | 全系統可讀，可透過 settings deny 特定路徑 | ⚠️ 預設寬鬆 | [claude.com docs] |
| 網路限制 | 沙箱內命令透過代理路由，僅允許已批准域名 | ✅ | [claude.com docs] |
| 工具審批 | 權限層 → 沙箱層（雙閘門），auto-allow + regular 兩模式 | ✅ | [webSearch 2026-03-27] |
| 沙箱技術 | macOS: Seatbelt；Linux/WSL2: bubblewrap | ✅ kernel-enforced | [claude.com docs] |
| 弱點 | 讀取無限制（預設）；dangerously-skip-permissions 旁路 | 🔴 | [claude.com docs] |

### 2. Gemini CLI (Google)

| 維度 | 機制 | 有效性 | 來源 |
|------|------|--------|------|
| 工作目錄限制 | rootDirectory — 所有工具限於啟動目錄 | ✅ 硬失敗（hard-fail） | [github.io docs] |
| 工具審批 | Policy Engine — allow/deny by glob 規則 | ✅ | [poncardas.com] |
| 沙箱選項 | Docker/Podman 容器 + macOS Seatbelt 支援 | ✅ 可選 | [mintlify.com] |
| 信任模型 | folderTrust — /permissions trust 手動授信 | ✅ | [poncardas.com] |
| 弱點 | 原生編輯工具 hard-fail 不提示；blockGitExtensions 可關閉 | ⚠️ | [github.com issue] |

### 3. Antigravity (Windsurf / Codeium)

| 維度 | 機制 | 有效性 | 來源 |
|------|------|--------|------|
| 認證 | LS gRPC-Connect — CSRF Token + Cookie | ✅ 本地 | [SakiAgentHistory Phase 1-10] |
| 工具配額 | CortexStepType — 僅 Core 類消耗配額 | ✅ 但無路徑限制 | [Agent內部架構速查 SKILL] |
| 瀏覽器沙箱 | browser_allow_list.json | ✅ | [.gemini/antigravity/] |
| 弱點 | **無 OS 級沙箱**；工具可存取全文件系統（受限於啟動使用者權限） | 🔴 關鍵弱點 | [逆向研究] |
| 弱點 | conversations/*.pb 使用 AES-256-GCM 加密但金鑰硬編碼 | 🔴 | [AgentContext SKILL] |

### 4. Cursor (Anysphere)

| 維度 | 機制 | 有效性 | 來源 |
|------|------|--------|------|
| 工作目錄 | workspace 級別限制 + .cursorignore | ✅ | [cursor.com docs] |
| 信任模型 | workspace trust — restricted/normal 雙模式 | ✅ | [cursor.com docs] |
| 沙箱 | 2025/11 從 allow-list 轉為直接 FS 存取（爭議） | ⚠️ 退化 | [luca-becker.me] |
| 弱點 | 沙箱模型變更暴露 ~/.ssh / ~/.aws 等敏感路徑 | 🔴 | [luca-becker.me] |
| 進階 | 2026 MicroVM 沙箱（NanoClaw + Docker 合作） | ✅ 實驗中 | [thenewstack.io] |

---

## 統一 Agent 能力矩陣

| 限制維度 | Claude Code | Gemini CLI | Antigravity | Cursor |
|----------|:-----------:|:----------:|:-----------:|:------:|
| CWD 寫入限制 | ✅ OS級 | ✅ 硬失敗 | ❌ | ✅ |
| 全 FS 讀取限制 | ⚠️ 可選 deny | ❌ 全域可讀 | ❌ | ⚠️ .cursorignore |
| 網路隔離 | ✅ 代理 | ⚠️ 可選容器 | ❌ | ❌ |
| OS 級沙箱 | ✅ Seatbelt/bwrap | ⚠️ 可選 | ❌ | ⚠️ 轉型中 |
| 工具審批 | ✅ 雙閘門 | ✅ Policy | ⚠️ 僅 CSRF | ✅ |
| Git 推送限制 | ⚠️ 當前分支限 | ✅ blockGit | ❌ | ❌ |
| 跨機執行限制 | N/A | N/A | N/A | N/A |

### 🔴 關鍵發現

**所有 Agent 均不具備「跨機執行時的傳輸層邊界限制」**。現有限制全在 client 端（Agent 本機），一旦 Agent 透過 SakiAgentSSH 的 gRPC 連線遠端執行指令，**daemon 側完全沒有基於 SSH key 的 channel 級權限隔離**。這正是本協議需要填補的空白。

---

## 風險評估 (Risk Assessment)

| 風險 | 程度 | 說明 |
|------|------|------|
| Agent 繞過 client 端沙箱 | 高 | prompt injection 可觸發 dangerously-skip-permissions |
| daemon 無路徑限制 | 高 | 現有 SakiAgentSSH ACL 僅限 IP + Token，不限路徑 |
| Token 竊取 | 中 | Cursor 沙箱變更暴露 ~/.ssh；Agent 可讀取 token file |
| 跨使用者提權 | 低 | daemon 以受限使用者運行（L3），但 config 無 chroot |

---

## 實作建議 (Actionable Advice)

### SakiAgentSSH 協議需新增的邊界限制層

1. **SSH Channel-Level Authorization**
   - 每個 gRPC session 綁定一組 SSH key pair
   - key 對應 `authorized_commands` 白名單（類似 SSH `authorized_keys` 的 `command=` 限制）
   - 支援路徑白名單（`chroot_dir`）

2. **Daemon 側強制邊界**
   - 檔案操作限於配置的 `allowed_paths` 前綴
   - 指令執行限於配置的 `allowed_commands` 清單
   - 環境變數注入限制（不繼承 daemon 環境）

3. **Session 級限制**
   - 每個 SSH channel 有獨立的 capability set
   - 超時自動銷毀 session
   - 並行 session 上限

4. **審計**
   - 所有操作寫入 audit log
   - 異常行為偵測（大量檔案讀取、敏感路徑存取）
