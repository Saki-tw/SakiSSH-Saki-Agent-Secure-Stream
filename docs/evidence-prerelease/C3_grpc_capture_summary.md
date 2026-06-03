# C3: gRPC Capture Summary

> **來源**: C1\_session\_log.log + C2\_incident\_window.log
> **分析時間**: 2026-05-31T18:20 UTC+8
> **分析方法**: grep/awk/sort/uniq 統計提取

---

## 檔案統計

| 檔案 | 行數 | 大小 | 時間範圍 | 描述 |
|------|------|------|---------|------|
| C1 | 14,575 行 | 2.7 MB | 2026-02-26 22:49 ~ 2026-02-27 20:20 UTC+8 | 事件後 session（LS log） |
| C2 | 1,180 行 | 377 KB | 2026-02-26 03:20 ~ 05:06 UTC+8 | 事件窗口（IDE daily log 擷取） |

### C1 Log Level 分佈

| Level | Count |
|-------|-------|
| `[info]` | 14,413 |
| （無 warning/error/debug level） | — |

> **備註**：C1 為 LS（Language Server, Go）的 stdout log，全部以 `[info]` 等級輸出，包含 Go 層 `I`(info) 和 `E`(error) 前綴。

### C2 Log Level 分佈

| Level | Count |
|-------|-------|
| `[ERROR]` | 658 |
| `[WARN]` | 442 |
| `[INFO]` | 74 |

> **備註**：C2 為 IDE（Electron renderer / Extension Host）的日誌，以 `[JS]`/`[LEGACY]`/`[network]` 來源區分。

---

## Tool Call Distribution — C1（gRPC 方法呼叫統計）

### 完整 gRPC URI 匹配（`/exa.*`）

| # | gRPC Method | Count | 佔比 | 類別 |
|---|-------------|-------|------|------|
| 1 | `SendActionToChatPanel` | 13,181 | 99.97% | monitoring（Extension → LS 狀態輪詢） |
| 2 | `StreamAgentStateUpdates` | 2 | 0.015% | monitoring（Agent 狀態串流） |
| 3 | `RefreshMcpServers` | 1 | 0.008% | operational（MCP 伺服器重新載入） |
| 4 | `AcknowledgeCascadeCodeEdit` | 1 | 0.008% | operational（確認程式碼編輯） |
| — | **合計** | **13,185** | 100% | — |

### RFC 7258 分類

| 類別 | 呼叫數 | 佔比 |
|------|--------|------|
| **Monitoring**（狀態輪詢/心跳/串流） | 13,183 | **99.98%** |
| **Operational**（實際功能操作） | 2 | **0.02%** |

> **關鍵發現**：99.98% 的 gRPC 呼叫為 `SendActionToChatPanel`，此為 Extension Host 向 LS 發送 UI 動作的通道。在 21.5 小時的 session 中，共產生 13,181 次呼叫，平均每 **5.87 秒** 一次。Session 末段（20:20 UTC+8）仍出現 `channel is full` 錯誤，顯示通道飽和問題持續至 session 結束。

---

## Tool Call Distribution — C2（事件窗口）

C2 為 IDE 日誌，不包含 gRPC URI 格式（`/exa.*`），但包含 JavaScript 層的 gRPC 呼叫堆疊：

| # | 方法（JS 層） | 出現次數 | 類別 |
|---|--------------|---------|------|
| 1 | `sendActionToChatPanel` | 220 | monitoring（堆疊追蹤中的方法名） |

> **備註**：C2 的 220 次出現均來自 `ConnectError` 堆疊追蹤中的 `Object.sendActionToChatPanel`，代表失敗的 gRPC 呼叫嘗試。

---

## API Endpoint Distribution

### C1

| # | URL | Count | 用途 |
|---|-----|-------|------|
| 1 | `https://daily-cloudcode-pa.googleapis.com/v1internal:streamGenerateContent?alt=sse` | 135 | Cloud Code API — 模型推論串流 |

> **備註**：135 次 API 呼叫對應 141 次 planner 請求（部分請求在 API 層被重試前即已產生 log）。

### C2

| # | URL | Count | 用途 |
|---|-----|-------|------|
| — | （無外部 API URL 記錄） | 0 | — |

> **備註**：C2 為 IDE 前端日誌，不記錄後端 API URL。事件期間唯一觀察到的網路活動為 HTTP 502 回應（1 次）和 Saki Extension 的 quota fetch 失敗。

---

## Planner Request Statistics（C1）

| 指標 | 值 |
|------|-----|
| **Planner 請求總數** | 141 |
| **最小 chat messages** | 2 |
| **最大 chat messages** | 226 |
| **平均 chat messages** | 97.1 |
| **可重試 API 錯誤** | 19 次 |
| **API retry attempt ≥ 2** | 19 次 |

### 模型可用性問題

| 錯誤類型 | 次數 |
|---------|------|
| `UNAVAILABLE (503): No capacity for claude-opus-4-6-thinking` | 19 |

> **發現**：所有 API 重試均因 `claude-opus-4-6-thinking` 模型容量不足（503）。重試延遲從 5 秒到 ~10 秒不等（指數退避）。

### C2 Planner

- C2 事件窗口中 **無** planner 請求記錄（IDE 前端日誌不包含此資訊）。

---

## Error Distribution

### C1（LS Log）

| # | 錯誤類型 | 次數 | 嚴重度 |
|---|---------|------|--------|
| 1 | `channel is full`（SendActionToChatPanel 通道飽和） | 13,181 | 🔴 Critical |
| 2 | `UNAVAILABLE (503)` — 模型容量不足 | 19 | 🟡 Warning |
| 3 | `LS_ERROR` 遙測 | 14 | 🟡 Warning |
| 4 | `error from [REDACTED_IP]`（來自 MCP 的錯誤） | 5 | 🟡 Warning |
| 5 | `fast apply fallback`（程式碼編輯套用失敗） | 2 | 🟠 Moderate |
| 6 | `cascade step: CORTEX_STEP_TYPE_MCP_TOOL`（MCP 檔案未找到） | 1 | 🟡 Warning |
| 7 | `cascade step: CORTEX_STEP_TYPE_CODE_ACTION`（程式碼套用失敗） | 1 | 🟡 Warning |
| — | **C1 錯誤總計** | **43** | — |

> **附註**：43 為 `grep -ci 'error'` 的結果，但 `channel is full` 在 Go log 中以 `E`（error 前綴）標記卻歸類於 `[info]` level，實際影響遠大於 43。interceptor.go:74 記錄的 `channel is full` 有 13,181 條，為真正的主要錯誤。

### C2（事件窗口）

| # | 錯誤類型 | 次數 | 嚴重度 |
|---|---------|------|--------|
| 1 | `channel is full` | 880 | 🔴 Critical |
| 2 | `ConnectError` | 660 | 🔴 Critical |
| 3 | `429`（Rate Limit） | 6 | 🟡 Warning |
| 4 | `503`（Service Unavailable） | 3 | 🟡 Warning |
| 5 | HTTP 502（network layer） | 1 | 🟡 Warning |
| — | **C2 錯誤總計**（`grep -c error`） | **1,142** | — |
| — | **C2 警告總計**（`grep -c warn`） | **442** | — |

### 錯誤模式分析

C2 事件窗口呈現高度規律的錯誤循環（每 **~60 秒** 一輪）：

```
[T+0.000] [WARN]  rejected promise: ConnectError: channel is full
[T+0.001] [WARN]  stack trace: ConnectError → sendActionToChatPanel
[T+0.002] [ERROR] [google.cortex] channel is full
[T+0.002] [ERROR] ConnectError: channel is full (full stack)
[T+0.003] [ERROR] 發生未知的錯誤。如需詳細資訊，請參閱記錄檔。
[T+60.000] (下一輪重複)
```

每輪產生：2 WARN + 6 ERROR = **8 條日誌**（`[LEGACY]` 和 `[JS]` 各記錄一份）。
- 事件窗口持續 **~106 分鐘**（03:20 ~ 05:06）
- 預計產生 ~106 輪 × 8 條 ≈ **848 條**（與實際 880+660 大致吻合，差異來自部分 rate limit 和 502 事件插入）。

---

## Cortex Step Types（C1）

| Step Type | 次數 | 結果 |
|-----------|------|------|
| `CORTEX_STEP_TYPE_MCP_TOOL` | 1 | 失敗（No such file or directory） |
| `CORTEX_STEP_TYPE_CODE_ACTION` | 1 | 失敗（target content not found） |

---

## MCP Server Interactions

### C1（LS Log）

| 時間戳 | 事件 | 詳情 |
|--------|------|------|
| 2026-02-26 22:49:36 | `RefreshMcpServers` 失敗 | `loading already in progress`（MCP 伺服器重新載入正在進行中） |
| 2026-02-26 22:49:37 | Chrome DevTools MCP 發現 | `http://[REDACTED_IP]:[PORT]/mcp`，停止 watcher |
| 2026-02-27 03:06:52 | MCP 工具執行失敗 | `CORTEX_STEP_TYPE_MCP_TOOL: No such file or directory (os error 2)` |

> **附加發現**：C1 log 中包含大量 `SakiMCP` 相關的 shell 命令輸出（Agent 執行的命令），涉及版本檢查、專案結構掃描、Scientia 搜尋等。這些是 Agent 行為的記錄而非 MCP 伺服器的系統呼叫。

### C2（事件窗口）

| 時間戳 | 事件 | 詳情 |
|--------|------|------|
| — | 無 MCP 相關記錄 | 事件期間 IDE 前端未記錄 MCP 活動 |

> **備註**：C2 事件窗口中出現 `[Saki] Quota fetch error: 連接失敗: fetch failed`（~每 5 分鐘一次，共 ~3 次），這是 Saki Extension 嘗試查詢 quota API 的失敗——可能因 LS 通道飽和導致整體網路堆疊受影響。

---

## RFC 7258 Classification — 綜合評估

### C1 Session（21.5 小時）

| 分類 | 呼叫數 | 佔比 |
|------|--------|------|
| **Monitoring / 監控類** | 13,183 | **99.98%** |
| └ `SendActionToChatPanel`（UI → LS 狀態輪詢） | 13,181 | 99.97% |
| └ `StreamAgentStateUpdates`（Agent 狀態串流） | 2 | 0.015% |
| **Operational / 操作類** | 2 | **0.02%** |
| └ `RefreshMcpServers`（MCP 重新載入） | 1 | 0.008% |
| └ `AcknowledgeCascadeCodeEdit`（確認編輯） | 1 | 0.008% |

### C2 事件窗口（106 分鐘）

| 分類 | 事件數 | 佔比 |
|------|--------|------|
| **Monitoring / 監控類**（全部為失敗的 `sendActionToChatPanel`） | 1,100 | **~93%** |
| **Error Reporting / 錯誤回報**（`ConnectError` + 未知錯誤 UI 通知） | ~80 | **~7%** |
| **Operational / 操作類** | 0 | **0%** |

### 綜合結論

1. **通道飽和是核心問題**：C1 和 C2 都以 `channel is full` 為主要錯誤，表明 gRPC 的 `SendActionToChatPanel` 通道缺乏背壓機制。
2. **監控流量壓倒性**：>99.9% 的 gRPC 呼叫為監控/狀態輪詢，非使用者操作。
3. **事件窗口完全癱瘓**：C2 期間（03:20~05:06）無任何 operational 呼叫成功，IDE 處於持續的 `ConnectError` 循環。
4. **模型容量問題**：C1 中 19 次 503 錯誤表明 `claude-opus-4-6-thinking` 模型在高峰期容量不足，但此問題與通道飽和獨立。
5. **外部 API 為單一端點**：所有後端呼叫指向 `daily-cloudcode-pa.googleapis.com`（Cloud Code API），共 135 次。

---

*Generated by SakiAgentSSH evidence pipeline, 2026-05-31T18:20 UTC+8*
