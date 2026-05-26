# Agent 工具執行能力之儲存邊界深度逆向研究

> **建立時間**：2026-03-27 22:40 (UTC+8)
> **研究者**：SakiAgentSSH 協議設計研究 Phase 3
> **狀態**：完成
> **標籤**：#逆向工程 #Agent工具 #沙箱 #儲存邊界

---

## 摘要 (Executive Summary)

**結論**：四大 Agent（Claude Code、Gemini CLI、Antigravity、Cursor）的工具邊界限制在「跨機執行」維度均為空白。本協議（SakiAgentSSH v3.0）的 capability-based permission model 可有效填補此空白，構成方法論上有效的第四層邊界限制。

---

## 1. Gemini CLI 工具邊界逆向分析

### 1.1 架構觀察

| 觀察點 | 發現 |
|--------|------|
| 可執行檔 | `/opt/homebrew/bin/gemini`（62 行 Node.js wrapper） |
| 核心模組 | `@google/gemini-cli-core`（npm minified dist） |
| 授權 | Google LLC 2025, Apache-2.0 |
| 配置路徑 | `~/.gemini/settings.json` + `~/.gemini/projects.json` |

### 1.2 工具邊界機制

| 機制 | CLI 參數 | 強制性 | 繞過方式 |
|------|----------|--------|----------|
| 沙箱模式 | `--sandbox` / `-s` | 可選 | 不帶此flag |
| YOLO 模式 | `--yolo` / `-y` | 可選 | 啟用即繞過所有 |
| 審批模式 | `--approval-mode` | 預設: `default` | 設為 `yolo` |
| 政策引擎 | `--policy` / `--admin-policy` | 可選 | 不載入 policy |
| 工具白名單 | `--allowed-tools`（已棄用→Policy Engine） | 可選 | N/A |
| Git 保護 | `settings.security.blockGitExtensions` | 可選 | 設為 false |
| 目錄信任 | `settings.security.folderTrust.enabled` | 可選 | 設為 false |

### 1.3 projects.json 分析

```json
{
  "/Users/hc1034/Saki_Studio/Claude": "claude",
  "/Users/hc1034": "hc1034",
  "/Users/hc1034/.ssh": "ssh"    // ⚠️ .ssh 被註冊為專案
}
```

**弱點**：`.ssh` 目錄被註冊為合法專案，Gemini CLI 可自由存取 SSH 金鑰。

### 1.4 rootDirectory 硬邊界

Gemini CLI 文件工具（view_file、replace_file_content 等）受 `rootDirectory` 限制（啟動目錄），超出範圍會 **hard-fail**（直接失敗，不提示使用者）。但 `bash` 工具（run_command）不受此限制——可透過 shell 命令存取任意路徑。

**結論**：Gemini CLI 的路徑限制 **僅對文件工具有效，對 shell 工具無效**。

---

## 2. Antigravity (Windsurf) 工具邊界逆向分析

### 2.1 架構觀察

| 觀察點 | 發現 |
|--------|------|
| LS gRPC 認證 | CSRF Token + Cookie（Port 52743） |
| MCP Server | 內建 SSE（Port 52773） |
| CDP Debug | WebSocket（Port 9000-9003） |
| Token Budget | MAX_BUDGET=750,000 + THINKING_BONUS=200,000 |

### 2.2 工具邊界機制

| 機制 | 強制性 | 繞過方式 |
|------|--------|----------|
| CSRF Token | 本地 HTTP | 直接存取 LS endpoint |
| CortexStepType | 配額控制（僅 Core 消耗） | 工具呼叫不計配額 |
| browser_allow_list.json | 瀏覽器域名白名單 | N/A |
| **OS 級沙箱** | **❌ 不存在** | — |
| **路徑限制** | **❌ 不存在** | — |

### 2.3 關鍵弱點

1. **無 OS 級沙箱**：所有工具可存取啟動使用者可存取的任何路徑
2. **無指令限制**：run_command 可執行任意指令（包括 `sudo`、`rm -rf`）
3. **對話加密金鑰硬編碼**：`conversations/*.pb` 使用 AES-256-GCM 但金鑰可逆向取得
4. **workspace 概念弱**：雖有 workspace 但不阻止跨 workspace 操作

---

## 3. Claude Code 工具邊界逆向分析

### 3.1 架構觀察

| 觀察點 | 發現 |
|--------|------|
| 可執行檔 | `~/.local/bin/claude`（Mach-O 64-bit ARM64） |
| 配置路徑 | `~/.claude/settings.json` + `~/.claude/projects/` |
| 專案目錄 | 以路徑 slug 命名（`-Users-hc1034-Saki-Studio-Claude`） |
| Session 格式 | `.jsonl`（JSON Lines，明文） |

### 3.2 工具邊界機制

| 機制 | CLI 參數 | 強制性 | 繞過方式 |
|------|----------|--------|----------|
| 工具拒絕名單 | `--disallowedTools` | 可選 | 不帶此flag |
| Seatbelt 沙箱 | `/sandbox` 命令 | 可選但推薦 | 不啟用 |
| 寫入限制 | CWD 寫入限制（沙箱啟用時） | OS 級 | 未啟用沙箱時無限制 |
| 網路隔離 | 代理路由 + 域名審批 | 沙箱內 | 未啟用沙箱時無限制 |
| 權限自動模式 | auto-allow mode | 沙箱內自動 | N/A |
| Git 限制 | 僅當前分支可 push | Web 版本 | CLI 版無限制 |

### 3.3 Seatbelt 沙箱分析

Claude Code 使用 macOS Seatbelt（`sandbox-exec`）：
- **寫入**：僅允許 CWD 及子目錄
- **讀取**：全系統可讀（可透過 `settings.json` deny 特定路徑）
- **網路**：透過代理路由，僅允許已批准域名
- **強制性**：kernel-enforced（無法繞過）

**但**：Seatbelt 是 **可選的**。使用者需手動啟用 `/sandbox`。預設安裝**不啟用沙箱**。

---

## 4. Cursor 工具邊界逆向分析（簡要）

| 機制 | 狀態 |
|------|------|
| .cursorignore | ✅ 可阻止 Agent 讀取特定檔案 |
| Workspace trust | ✅ restricted/normal 模式 |
| 沙箱模型（2025/11 後） | ⚠️ 從 allow-list 轉為直接 FS 存取 |
| 弱點 | 🔴 暴露 ~/.ssh / ~/.aws 等敏感路徑 |
| MicroVM 沙箱 | 🔵 2026 實驗中（NanoClaw + Docker） |

---

## 5. 統一工具能力矩陣（深度版）

| 維度 | Claude Code | Gemini CLI | Antigravity | Cursor |
|------|:-----------:|:----------:|:-----------:|:------:|
| **文件工具路徑限制** | ✅ CWD（沙箱時） | ✅ rootDir（hard-fail） | ❌ | ✅ workspace |
| **Shell 工具路徑限制** | ✅ CWD（沙箱時） | ❌ 可任意存取 | ❌ | ⚠️ |
| **指令黑名單** | ⚠️ --disallowedTools | ⚠️ --policy deny | ❌ | ❌ |
| **OS 級沙箱** | ⚠️ 可選 Seatbelt | ⚠️ 可選 Docker | ❌ | ⚠️ 轉型中 |
| **網路隔離** | ✅ 代理（沙箱時） | ⚠️ 可選容器 | ❌ | ❌ |
| **審計日誌** | ✅ history.jsonl | ✅ history/ | ⚠️ conversations.pb | ✅ |
| **預算控制** | ✅ --max-budget-usd | ❌ | ❌ | ❌ |
| **跨機執行限制** | ❌ | ❌ | ❌ | ❌ |

---

## 6. 方法論有效性確立

### 6.1 SakiAgentSSH v3.0 協議對各 Agent 的邊界限制有效性

| Agent | 自身弱點 | 本協議覆蓋 | 方法論有效性 |
|-------|----------|:----------:|:----------:|
| Claude Code（無沙箱） | Shell 可任意存取 FS | ✅ allowed_paths + allowed_commands | **有效** |
| Claude Code（有沙箱） | 讀取仍無限制 | ✅ denied_paths（daemon 強制） | **有效** |
| Gemini CLI | Shell 工具無邊界 | ✅ allowed_commands + allowed_cwd | **有效** |
| Antigravity | 全無限制 | ✅ 五維邊界全覆蓋 | **高度有效** |
| Cursor | 沙箱退化暴露敏感檔 | ✅ denied_paths + allowed_paths | **有效** |

### 6.2 方法論論證

**定理**：SakiAgentSSH v3.0 的 capability-based permission model 對任何具有工具執行能力之 Agent 的儲存空間邊界限制，在方法論上是有效的。

**證明**：

1. **獨立性**：Daemon 側限制獨立於 Agent client 任何安全機制運作
2. **完備性**：五維邊界（路徑/指令/環境/網路/時間）覆蓋所有已知攻擊向量
3. **不可繞過性**：限制在 daemon process 內強制執行，Agent 無法干預（unlike client-side sandbox 可被 prompt injection 繞過）
4. **可審計性**：所有操作記錄 audit log，異常可偵測
5. **最小權限**：每個 Agent key 僅授予必要的 capability set（POLP）

**Q.E.D.** — 任何 Agent 透過 SakiAgentSSH 跨機執行時，其儲存空間操作受限於 daemon 側 capability set 定義的邊界，此邊界獨立於 Agent 自身安全機制，因此方法論上有效。
