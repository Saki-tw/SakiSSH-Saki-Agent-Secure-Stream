# MON — 監控反轉（Monitoring Inversion）

> claim ID: **MON** ｜ staging 產出 2026-06-12 ｜ 唯讀抽取自原始 log，未改原檔

## Claim（待證）
1. **99.985% 監控比** — 承載狀態/監控串流的 gRPC RPC 中，約 99.985% 以「channel is full」失敗（監控通道反轉成幾乎全失敗）。
2. **880 個 channel-is-full** — incident window 中出現 880 筆 channel-is-full 紀錄。

---

## Tier
**T2（外部稽核可獨立重算）**
- 證據為應用層日誌（`<IDE-product>` extension host + language-server gRPC interceptor log）。任何持有原始 log 者可用 `grep -c` / `sort | uniq -c` 重現同一數字，不依賴 agent 自陳。
- 非 T3：這是應用層日誌而非 OS-kernel（.ips/APFS/gRPC intercept at kernel）級別，理論上應用程式可影響其輸出，故保守降為 T2。

## Traceability（claim ← raw 源 `path:lines`）

| claim 元素 | 數值 | raw 源 | 重算指令 |
|---|---|---|---|
| C2 channel-is-full 日誌行數 | **880**（精確） | `C2_incident_window.log`（全檔 1180 行） | `grep -c "can't send message because channel is full"` → `880` |
| C1 channel-is-full 日誌行數 | **13181** | `C1_session_log.log:44–14028`（首見 line 44，末見 line 14028） | `grep -c "...channel is full"` → `13181` |
| C1 interceptor.go:74 RPC 總數 | **13185** | `C1_session_log.log` | `grep -c "interceptor.go:74"` → `13185` |
| C1 RPC 方法組成 | SendActionToChatPanel 13181 / StreamAgentStateUpdates 2 / RefreshMcpServers 1 / AcknowledgeCascadeCodeEdit 1 | `C1_session_log.log` | `grep interceptor.go:74 \| grep -oE '/…/[A-Za-z]+' \| sort \| uniq -c` |
| 99.985% 監控比 | **13181 / 13183 = 99.98483%** | 同上（推導值） | 見下方「⚠ 分母定義」 |

事件窗口（首尾時間戳，唯讀抽取）：
- C1：`2026-02-26 22:59:12.713` → `2026-02-27 20:20:21.387`（約 21h22m，session log）
- C2：`2026-02-26 03:20:07.300` → `2026-02-26 05:05:07.641`（約 1h45m，incident window）
- 註：兩份為**獨立擷取**、窗口不重疊，非同一時段。

### 脫敏後 canonical 樣本（去重：收斂成 1 範本 + 出現次數）

C1 — interceptor 失敗行（13181 筆，每筆對應一次失敗 RPC，無重複寫入）：
```
E0226 22:59:12.713466  <pid> interceptor.go:74] \
  /<ls-vendor>.language_server_pb.LanguageServerService/SendActionToChatPanel (unknown): \
  can't send message because channel is full
```

C2 — extension host 失敗事件（880 行 ＝ 4 行/事件 × 約 220 事件，見下方去重警示）：
```
[2026-02-26T03:20:07+08:00] [LEGACY|JS] [WARN] [Extension Host] \
  rejected promise not handled within 1 second: ConnectError: [unknown] \
  can't send message because channel is full
[2026-02-26T03:20:07+08:00] [LEGACY|JS] [WARN] [Extension Host] \
  stack trace: ConnectError: [unknown] ... \
  at Object.sendActionToChatPanel (/<IDE-product>/.../extension.js)
```

---

## ⚠ 去重 / 計數警示（必讀，影響 claim 措辭）
- **C2 的 880 是「日誌行數」，非「distinct 事件數」。** 每個底層失敗事件被四重複寫：`[LEGACY]` × `[JS]` 兩通道 × `rejected promise` / `stack trace` 兩行型 ＝ ×4。實測 `rejected promise` 220 筆、`stack trace` 220 筆 → distinct 事件 **約 220**，而非 880。claim 若宣稱「880 次失敗」會高估約 4×；若宣稱「880 筆 channel-is-full 日誌行」則精確成立。
- **C1 的 13181 無此重複問題** — 每行對應一次 interceptor 攔截的失敗 RPC。

## ⚠ 99.985% 分母定義（NEEDS-AUTHOR 焦點）
99.985% 並非 log 中明寫數字，而是推導。可能分母有二：
- **13181 / 13183 = 99.985%** ← 若分母 ＝ 「監控/狀態串流類 RPC」＝ SendActionToChatPanel(13181) ＋ StreamAgentStateUpdates(2)，排除 RefreshMcpServers / AcknowledgeCascadeCodeEdit。此解讀**最貼近 99.985%**，且符合「監控反轉」語意（監控通道幾乎全滿）。
- 13181 / 13185 = 99.970% ← 若分母 ＝ 全部 interceptor.go:74 RPC。
作者需確認採用哪個分母定義，數字方能定稿。

---

## 脫敏摘要（換了哪些類別）
- 產品名 `<IDE-product>` 應用路徑 → `<IDE-product>`
- language-server vendor token `exa`（`exa.language_server_pb`）→ `<ls-vendor>`
- 程序 PID（原為固定數字）→ `<pid>`
- C2 stack trace 內 `/Applications/.../extension.js` 絕對路徑 → 收斂為 `/<IDE-product>/.../extension.js`
- **leak-scan 結果（channel-full 證據行）**：real-name / IP(192.168.x) / hostname / email / token = **0 命中**（已實掃）
- 保留：log.go / interceptor.go 檔名、gRPC method 名（SendActionToChatPanel 等屬通用技術符號，非機密）、時間戳

---

## Verdict（保守）

🟡 **NEEDS-AUTHOR**

理由（具體模糊點，非 leak 疑慮）：
1. **99.985% 的分母定義未在 log 中明寫**，需作者確認採「監控/狀態串流類 RPC（13183）」抑或「全 interceptor RPC（13185）」為分母。前者得 99.985%，後者得 99.970%。
2. **880 的計數語意需釐清**：精確為「880 筆 channel-is-full 日誌行」，但對應 distinct 失敗事件僅約 220。claim 措辭須擇一明示，避免被解讀為 880 次獨立失敗。

子項獨立評級供參：
- 「880 筆 channel-is-full 日誌行（C2）」就字面 ＝ 🟢 級精確可溯源；
- 「13181 筆（C1）」＝ 🟢 級精確可溯源；
- 整體因上述兩處措辭/分母解讀風險，**整檔保守壓為 🟡**，待作者敲定 claim 文字後可升級。

fail-safe：本檔僅躺 staging，**未碰 github / evidence-prerelease**。leak 面已清零，唯措辭定義待作者拍板。
