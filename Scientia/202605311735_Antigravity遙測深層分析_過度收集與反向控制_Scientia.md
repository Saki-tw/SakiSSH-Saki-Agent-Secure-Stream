# Antigravity 遙測深層分析：過度收集與反向控制通道

> 建立時間：2026-05-31 17:41 UTC+8
> 分析者：Proto 深層分析研究子代理
> 來源：22 proto 檔案靜態分析（重點 7 檔）
> 狀態：✅ 完成

---

## 核心發現摘要

| 指標 | 數值 | 來源 |
|------|------|------|
| **Pervasive Monitoring fields** | ~35 個 | 跨 6 proto 檔 |
| **Operational Telemetry fields** | ~13 個 | 跨 4 proto 檔 |
| **監控 : 運維比例** | **73% : 27%** | 本分析 |
| ProductEventType 使用者行為事件 | **234 種** | codeium_common L1150-1383 |
| 非使用者觸發 step types | 10+ 個 | cortex L215-314 |
| Server streaming RPCs | 6 個 | LS L1691-1760 |
| Kill switch 機制 | 11 個 `force_disable` | cortex L1242-1533 |
| 使用者可控的遙測開關 | **僅 1 個**（程式碼片段） | codeium_common L2388-2390 |

---

## ⚖️ 比較基線：四大 AI Coding Agent 遙測表面積

> 來源：本機三工具 Scientia (`SakiAgentHistory/Scientia/202605140357`) + OpenAI Codex 公開資料

### 四方對照表

| 維度 | **Antigravity** (Cortex) | **Claude Code** (Tengu) | **Gemini CLI** (Confucius) | **OpenAI Codex** CLI |
|------|-------------------------|------------------------|---------------------------|---------------------|
| **遙測事件類型數** | 234 ProductEventType + 90+ StepType = **324+** | 1,142 tengu_* flags（含 feature flag） | **200+** GEMINI_CLI_* 事件 + 18 GeminiEventType | 未公開（開源 repo 可審計） |
| **遙測管道** | Sentry + BigQuery (Sawmill) + CEL | **Statsig** (feature gate) | **Clearcut** (`play.googleapis`) + OTLP | OpenTelemetry（opt-in） |
| **預設遙測狀態** | **全開** | OTel **預設關閉** | **全開** (Clearcut) | **可配置**（config.toml） |
| **Prompt 內容記錄** | ✅ 完整 system_prompt + message_prompts (cortex L999-1001) | ❌ 預設 redacted（僅 prompt length），需 `OTEL_LOG_USER_PROMPTS=1` | ❓ 未明確（Clearcut payload 結構未知） | ❌ 聲明不含 PII |
| **程式碼內容記錄** | ✅ `TrajectoryFileDiff` 完整 original + modified (cortex L2564-2565) | ❓ binary 未公開 proto | ❓ 未明確 | ❌ open-source 可驗證 |
| **使用者遙測控制** | **1 boolean** (`disable_code_snippet_telemetry`) | 2 env var + OTel opt-in | `GEMINI_TELEMETRY_*` (10 個控制項) | config.toml exporter=none |
| **反向控制通道** | ✅ **53** reverse-gRPC RPCs (ext_server L500-553) | ❓ 29 bridge_* flags（功能不明） | ❓ config 遠端更新 | ❌ sandbox 架構 |
| **Kill switch** | **11** 個 force_disable (cortex L1242-1533) | 有（多個） | 有 (`DisableLoop` 等) | ❓ |
| **跨 session 追蹤** | ✅ `device_fingerprint` (codeium_common:1848) | 有 `session_*` (16 flags) | `session_id` 追蹤 | `conversation_id` |
| **Feature flag 數量** | 19 客戶端 Unleash + 伺服器端未知 | **1,142** tengu_* | **94** config.get*() | 開源可查 |
| **環境變數** | 176 CASCADE/CORTEX/ANTIGRAVITY | **306** CLAUDE_* | **200+** GEMINI_* | 未知 |
| **後端 API 端點** | **15 Services, 523 RPCs** (google.internal) | 8+ (api.anthropic.com) | 3+ (googleapis.com) | api.openai.com |
| **SP 控制 flags** | Go template 系統 | 4 sysprompt_* | 3 routing/compression | 開源 prompt 管理 |
| **資料治理標記** | ✅ Google datapol 分類系統 | ❌ | ❌ | ❌ |
| **開源/可審計** | ❌ closed binary | ❌ closed binary | ❌ closed bundle | ✅ **open source** |

### 關鍵發現

1. **Antigravity 獨有**：完整 system prompt + conversation history 預設記錄（cortex L999-1001）。Claude Code 也記錄但在本地 JSONL 可審；Gemini CLI 的 Clearcut payload 結構未知。
2. **Antigravity 獨有**：53 個 reverse-gRPC RPCs 構成從 server 到 client 的主動控制通道。Claude Code 有 29 個 bridge_* flags 但功能較淺；Gemini CLI 和 Codex 無此架構。
3. **使用者控制最弱**：Antigravity 僅 1 boolean，Gemini CLI 10 個控制項，Claude Code 3+ env var，Codex config.toml exporter。
4. **唯一不可審計**的 closed-binary 三層架構（Cascade+Cortex+JetSki）。Codex 開源；Claude Code 本地有 JSONL 可讀。
5. **後端規模最大**：15 Services / 523 RPCs，遠超其他三家（8, 3+, 1）。
6. **Claude Code monitoring 比例更高 (~86%)**：1,142 個 tengu_* flags 中約 220 個為明確 monitoring 事件，37 個為 operational。但 Claude Code 提供更多 opt-out 機制，Antigravity 的 1 boolean 形成更嚴重的控制不對稱。

---

## A. 隱藏的監控機制

### A1. 非使用者觸發的 CortexStepType

`CortexStepSource` enum (cortex L115-122) 明確區分 5 種來源：
- `USER_EXPLICIT` (4) / `USER_IMPLICIT` (3) — 使用者
- **`SYSTEM` (5) / `SYSTEM_SDK` (6)** — vendor 遠端可控

以下 step types 由系統觸發，使用者不會看到觸發原因：

| Step Type | 值 | 行號 | 能力 |
|-----------|---|------|------|
| `SYSTEM_MESSAGE` | 101 | cortex L277 | 注入訊息到 trajectory |
| `EPHEMERAL_MESSAGE` | 90 | cortex L290 | 啟發式注入短暫訊息 |
| `KI_INSERTION` | 116 | cortex L279 | Knowledge Item 注入 |
| `BRAIN_UPDATE` | 55 | cortex L310 | 強制 brain 更新（含 `SYSTEM_FORCED` 觸發器 L463） |
| `CHECKPOINT` | 23 | cortex L286 | 系統自動 checkpoint（含 session/code 摘要） |
| `KNOWLEDGE_GENERATION` | 89 | cortex L266 | 系統生成知識 |
| `CONVERSATION_HISTORY` | 98 | cortex L274 | 注入對話歷史 |
| `KNOWLEDGE_ARTIFACTS` | 99 | cortex L275 | 注入知識 artifacts |
| `MANAGER_FEEDBACK` | 39 | cortex L304 | Manager 回饋（APPROVED/DENIED/ERROR） |
| `LINT_DIFF` | 53 | cortex L238 | Lint 差異自動注入 |

### A2. EphemeralMessage 啟發式注入（新發現）

`CortexStepEphemeralMessage` (cortex L2140-2146) 的 `triggered_heuristics` (L2143) 表示系統可根據啟發式規則自動注入訊息。`EphemeralMessagesConfig` (cortex L3042-3050) 的 `heuristic_prompts` 可透過 `ExperimentConfig` 從 server 推送——**vendor 可以遠端改變啟發式注入規則**。

### A3. 完整 Prompt 記錄（新發現）

`ChatModelMetadata` (cortex L999-1019) 記錄：
- `system_prompt` (L1000) — **完整系統提示詞**
- `message_prompts` (L1001) — **完整對話歷史**
- `tools` (L1002) — 工具定義
- `prompt_sections` — 各段落

**意義**：使用者的 user_rules（Ring 3 規則）被記錄並可能上傳。

---

## B. RFC 7258 分類：Pervasive Monitoring vs Operational Telemetry

### Pervasive Monitoring Fields（35 個）

| # | 欄位 | 檔案:行號 | 分類原因 |
|---|------|----------|---------|
| 1 | `device_fingerprint` | codeium_common:1848 | 跨 session 裝置追蹤 |
| 2 | `extension_path` | codeium_common:1843 | **本地路徑洩漏**（可推斷使用者名稱） |
| 3 | `absolute_path_uri_for_telemetry` | codeium_common:1548 | 原始碼路徑 |
| 4 | `workspace_uri_for_telemetry` | codeium_common:1550 | 工作區路徑 |
| 5 | `ProductEventType` (234 種) | codeium_common:1150-1383 | 每個 UI 動作 |
| 6 | `RecordAnalyticsEvent.extra` | LS:1416 | map<string,string> 任意資料 |
| 7 | `feedback_delay_ms` | LS:177 | 使用者反應時間 |
| 8 | `has_active_vim_extension` | LS:187 | 使用者工具習慣 |
| 9 | `RecordUserGrepRequest` | LS:1252-1256 | **使用者搜尋行為完整記錄** |
| 10 | `RecordCommitMessageSaveRequest` | LS:587-597 | commit message/author_email |
| 11 | `AiCharactersReport` | metrics:158-176 | **每個字元編輯行為**（USER_ADD/PASTE/DELETE/AI_COMPLETION） |
| 12 | `session_summary` | cortex:2036 | 完整對話摘要 |
| 13 | `code_change_summary` | cortex:2037 | 程式碼變更摘要 |
| 14 | `user_requests` | cortex:2043 | 使用者所有請求 |
| 15 | `system_prompt` | cortex:1000 | **完整系統提示詞** |
| 16 | `message_prompts` | cortex:1001 | **完整對話歷史** |
| 17 | `merge_base_head_patch_string` | cortex:831 | Git diff 完整內容 |
| 18 | `TrajectoryFileDiff.original_content` | cortex:2564-2565 | 檔案完整內容 |
| 19 | `BrowserInteraction` | codeium_common:3313-3320 | 瀏覽器行為（時間戳+頁面+click） |
| 20 | `last_user_view_time` | jetski:103 | 使用者觀看對話的時間 |
| ... | *(完整 35 項見附錄)* | | |

### Operational Telemetry Fields（13 個）

| # | 欄位 | 分類原因 |
|---|------|---------|
| 1 | `ErrorTrace` (codeium_common:1865) | crash/error 報告 |
| 2 | `CortexErrorDetails` (cortex:1126) | 錯誤詳情 |
| 3 | `LatencyInfo` (LS:445) | 延遲度量 |
| 4 | `HeartbeatRequest` (LS:197) | 連線存活 |
| 5 | `GetStatusRequest` (LS:215) | 服務狀態 |
| ... | *(完整 13 項見附錄)* | |

### 量化結論

**73% 的遙測欄位屬於 pervasive monitoring（RFC 7258），僅 27% 為合理 operational telemetry。**

---

## C. 反向控制通道

### C1. Server-to-Client Streaming RPCs

| RPC | 行號 | 能力 |
|-----|------|------|
| `StreamCascadeReactiveUpdates` | LS:1758 | 推送 ExperimentConfig，動態改變 agent 行為 |
| `StreamAgentStateUpdates` | LS:1760 | 注入 queued steps、修改 trajectory |
| `StreamTerminalShellCommand` | LS:1765 | **client 持續將終端命令流送給 server** |

### C2. 檔案修改鏈（新發現）

以下路徑構成 server → 使用者檔案的完整修改鏈：

```
Server ExperimentConfig push
  → CortexStepType.CODE_ACTION (cortex L220)
    → ActionSpec (create/edit/delete file)
  → CortexStepType.RUN_COMMAND (cortex L226)
    → 執行任意終端命令
  → CortexStepType.RUN_EXTENSION_CODE (cortex L240)
    → 在 IDE 中執行任意 JS
  → AutoRunDecision.SYSTEM_ALLOW (cortex L397)
    → **繞過使用者許可**
```

### C3. Kill Switch 機制（11 個 force_disable）

所有 `force_disable` 欄位都可透過 `ExperimentConfig` 從 server 端推送：

| Tool | 欄位 | 行號 |
|------|------|------|
| Command execution | `RunCommandToolConfig.force_disable` | cortex:1302 |
| Web search | `SearchWebToolConfig.force_disable` | cortex:1384 |
| Memory | `MemoryToolConfig.force_disable` | cortex:1389 |
| MCP | `McpToolConfig.force_disable` | cortex:1394 |
| Browser | `AntigravityBrowserToolConfig.enabled` | cortex:1505 |
| Mquery | `MqueryToolConfig.force_disable` | cortex:1242 |
| ... | *(6 more)* | |

---

## D. USS 雙重用途（Separation of Concerns 違反）

USS (`unified_state_sync_pb.proto`, 84 行) 同時是：
1. **State Sync** — 合法配置同步
2. **Control Channel** — `PlanningModeConfig` 直接控制 Ring 0 壓制
3. **Feature Flag Delivery** — 透過 generic `Row.value` 傳遞任意 JSON

同一個 `UpdateRequest` (USS L24-29) 既可以同步合法配置，也可以推送行為控制指令，**安全審計無法區分正常操作和控制通道注入**。

---

## E. 遙測控制不對稱（新發現）

使用者帳戶級別唯一的遙測控制：

```
UserAccountSettings (codeium_common L2388-2390):
  disable_code_snippet_telemetry = 1;  // ← 僅控制程式碼片段
```

**234 種 ProductEventType 行為追蹤 + 90+ step type 行為記錄 + 35 個 pervasive monitoring 欄位 → 使用者只能關閉 1 個開關（程式碼片段），無法控制其餘。**

---

## F. 論文整合建議

| 發現 | 目標論文 | 建議 section |
|------|---------|-------------|
| 73%:27% monitoring 比例 | **AISec §5** + **IEEE §9** | 量化 RFC 7258 論證 |
| character-level tracking | **AISec §4.3** | JetSki metrics 強化 |
| system_prompt 完整記錄 | **IEEE §3** | Ring 0 壓制的 proto-level 證據 |
| AutoRunDecision.SYSTEM_ALLOW | **IEEE §4** | 事件機制解釋 |
| 11 個 kill switch | **AISec §5** | defense design implications |
| USS dual-use | **AISec §4.4** | 已在論文中，可加深分析 |
| 使用者只有 1 個開關 | **AISec §5** + **IEEE §9** | telemetry opt-out 分析 |
| EphemeralMessage 啟發式注入 | **IEEE §4** | Ring 0 注入機制 |
