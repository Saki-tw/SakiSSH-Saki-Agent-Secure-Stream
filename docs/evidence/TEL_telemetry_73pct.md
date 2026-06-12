# TEL — 73% 遙測/監控欄位 claim 支撐證據（脫敏 staging）

**Claim ID**: TEL
**Claim 文字**: Vendor A 之 AI coding agent（codename: Cortex）的 protobuf 定義欄位中，約 **73%** 屬於監控/遙測（surveillance/telemetry）用途，而非操作性的編碼輔助功能。

---

## Tier

**T2** — 外部稽核可獨立重算。證據基於 22 個 `.proto` 檔案的訊息/欄位分類，proto 來源已（於作者策展區）公開供獨立複現；分類準則明列，第三方可重新標註並驗算比例。非 OS-kernel 攔截，故非 T3。

## Traceability

- Claim「~73% protobuf 欄位為監控/遙測用途」← raw 源 `SakiAgentSSH/docs/evidence/C4_proto_field_classification.md:12`（Purpose 段）
- 73% 為 **field-level** 數字 ← raw 源 `:43`、`:72`、`:161`（多處明示）
- 本文件實際提供者為 **message-level 58%** ← raw 源 `:69`（TOTAL 列）、`:72`
- 分類準則三類（Surveillance / Operational / Mixed）← raw 源 `:28-32`
- 代表性監控訊息範例 ← raw 源 `:80-153`
- 方法論限制（單標註者、proto≠runtime、版本特定性）← raw 源 `:156-164`

## 脫敏摘要

| 類別 | 原始 | 脫敏後 | 理由 |
|------|------|--------|------|
| 真實廠商名 | proto 檔名含 `codeium`（`exa.codeium_common_pb.proto`） | `exa.<vendor>_common_pb.proto` | **去匿名化漏洞**：`codeium` 直接洩漏 Vendor A 真身，與全文「Vendor A / Cortex」匿名策略衝突 |
| 產品/廠商代號 | Cortex / Vendor A | 保留 | 已是 raw 源既定匿名代號（符合 ruleset 廠商→代號規則） |
| 策展區路徑 | `evidence-prerelease/proto/` | 保留為相對路徑 | 屬作者署名命名，非機敏；無真實機碼/IP/token |

> proto 欄位名（`user_id`、`machine_id`、`ide_telemetry_id` 等）為**被分析對象**的協議欄位，非操作者個資，依語意保留作為證據。

---

## 數據摘要（脫敏後）

### 訊息層級分類（22 proto 檔，本文件可驗證之保守下界）

| 指標 | 計數 | 比例 |
|------|------|------|
| Total messages | ~302 | 100% |
| **Surveillance** | ~175 | **58%** |
| Operational | ~102 | 34% |
| Mixed | ~25 | 8% |

> raw 源 `:72` 明示：message-level 為 58%，低於 field-level 73%，因監控類訊息平均欄位數較多（如 `Metadata` 訊息常含 15–20 個遙測欄位）。

### 代表性監控訊息（脫敏後）

1. **Process Enumeration** — `GetProcessesRequest/Response`、`ProcessInfo{name,pid,command_line}`：列舉使用者系統全部執行中程序；無編碼輔助功能需要完整程序清單。（raw `:80-96`）
2. **Cascade Request Metadata**（Mixed）— 操作性 `user_message`/`conversation` 與監控性 `user_id`/`machine_id`/`ide_telemetry_id`/`session_id`/`workspace_id`+10 餘欄位並存。（raw `:98-117`）
3. **Experiment Configuration** — `ExperimentConfig`/`Experiment{key,variant}`：廠商基礎設施下推 A/B 測試組態，服務廠商最佳化而非使用者編碼需求。（raw `:119-133`）
4. **Event Logging** — `EventLog`、`AnalyticsEvent`：行為追蹤事件回傳廠商分析基礎設施。（raw `:135-152`）

---

## Verdict

🔴 **HOLD**

理由（兩項，任一即足以 HOLD，依 fail-safe 預設保守）：

1. **Claim 與本證據之數字落差**：claim 主張 **73%（field-level）**，但本 raw 源僅提供 **58%（message-level）**，且 raw 源 `:43`/`:161` 三度明示「field-level analysis **in progress**」。即 73% 此一具體數字在本文件中**尚未被支撐**，本文件僅能支撐「過半（58%）訊息為監控用途」之較弱命題。直接以本檔背書 73% 會超出證據範圍——須作者補上 field-level 分析或調整 claim 措辭。
2. **去匿名化殘留風險**：原始 proto 檔名含真實廠商字串 `codeium`，本檔已脫敏為 `<vendor>`，但同源 `evidence-prerelease/proto/` 內的原始檔名若同步公開，仍會洩漏 Vendor A 真身。需作者確認公開區 proto 檔名亦已一併匿名化，否則匿名策略破功。

leak-scan：real-name=已脫敏（codeium→`<vendor>`）、IP=0、token=0、operator 個資=0。脫敏本身達標，但因上述 claim-證據落差與跨檔匿名一致性疑慮，**不**升級為 🟢/🟡，預設 🔴 待作者裁決。
