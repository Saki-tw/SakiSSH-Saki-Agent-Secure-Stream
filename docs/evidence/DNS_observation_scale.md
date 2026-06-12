# DNS 觀測規模證據 — claim: 「四個月 5000 萬筆 DNS 查詢與 10% 封鎖率」

- **Claim ID / topic**: DNS / observation_scale
- **Tier**: **T2**（外部稽核可獨立重算：raw 為 JSON Lines log，任何稽核者用 `wc -l` / `grep -c` 即可復現以下全部聚合數字）
- **raw 源**: `SakiAgentHistory/dns_query_logs.json`（1.20 GB，5,414,014 行，逐行一筆 JSON DNS 事件；唯讀，未改）
- **抽取方式**: 全程 bash `wc` / `grep -c` / `grep -o | sort | uniq -c` 聚合，未將整檔讀入 context（遵 §大檔處理）

---

## Traceability（claim 文字 ← raw 源聚合）

| claim 子主張 | raw 源證據 | 數值 | 是否支撐 |
|---|---|---|---|
| 「DNS 查詢」觀測管線存在且具規模 | `dns_query_logs.json` 全檔行數 | **5,414,014 筆** | ✅ 多百萬級 |
| 「10% 封鎖率」 | `grep -c REQUEST_BLOCKED` ÷ 總行數 | **11.05%**（僅 request-blocked）<br>**13.92%**（含 response-blocked） | 🟡 同量級、定義依賴 |
| 「四個月」 | `time_iso` 最早 / 最晚 | **2026-05-02 → 2026-06-12（約 41 天）** | 🔴 **僅 ~1.4 個月** |
| 「5000 萬筆」 | 全檔行數 | **541 萬筆**（本檔） | 🔴 **差約一個數量級** |

### 封鎖率計算明細（denominator 敏感，故全列）
```
總事件                       5,414,014
  REQUEST_BLOCKED              598,042   ← 請求階段封鎖
  RESPONSE_BLOCKED             155,589   ← 回應階段封鎖（CNAME/IP 類）
  REQUEST_ALLOWED            1,684,730
  RESPONSE_ALLOWED             119,977
  NONE                       2,855,546   ← 未套用過濾決策（PTR 反查 / 內部 / 直通）
  MODIFIED                        131

封鎖率（request-blocked ÷ 總） = 598,042 / 5,414,014 = 11.05%   ← 最接近 claim 之「10%」
封鎖率（全封鎖 ÷ 總）        = 753,631 / 5,414,014 = 13.92%
封鎖率（全封鎖 ÷ 已評估）    = 753,631 / 2,558,338 = 29.46%   ← 排除 NONE 後
```
> claim 之「10%」在「request-blocked ÷ 總事件」定義下成立（11.05%，保守同量級）。但若採其他 denominator 可達 29%，**封鎖率的成立與否高度依賴定義**，原 claim 未指明分母。

### 其他維度聚合（佐證觀測管線真實性）
```
協議分布   DNS 3,010,344 / DOH 1,304,390 / DOQ 1,040,180 / DOT 59,101
記錄型別   A 2,345,334 / AAAA 2,197,648 / HTTPS 524,648 / PTR 305,233 / SRV / TXT / SOA / NS
DNSSEC     false 5,062,251 / true 351,764（驗證比例 6.5%）
觀測端點   5 個（全部 client_country=TW）
唯一封鎖域名數  3,006
回應來源國  QN 1,855,975 / US 1,812,030 / TW 551,740 / (空) / SG / HK / JP / DE …
```

封鎖目標**類別**（literal 域名已脫敏；類別為公開企業遙測，無機敏）：
可觀測管線主要封鎖 **遙測/可觀測性服務**、**分析/歸因 SDK**、**廣告**三類流量；
top 封鎖目標所屬公司（公開法人）依量序為 Apple / Google / Microsoft / Akamai / AdGuard(過濾清單來源) / Meta / Xiaomi / OpenAI / SoftBank / Cloudflare。

---

## 脫敏摘要（換了哪些類別）

| 類別 | 處理 |
|---|---|
| 觀測端點裝置名（5 個：手機型號、ASUS 路由器型號、UniFi 交換器型號、內網 SSID `work@home`） | → `<endpoint-1..5>`，literal 一律不寫入本檔 |
| 所有 literal 域名（含 `*.in-addr.arpa` 反查內含 IP） | → 一律脫敏，僅保留**類別**與**所屬公開法人**聚合計數 |
| 所有 IP（PTR 反查段內含） | → redact |
| 操作者 / 本機路徑 | 本聚合未含，無需處理 |
| 機密 token / key | raw 全檔 `grep` 未見 `ghp_*` / API key / OAuth secret 欄位（log schema 不含認證欄位）；leak-scan = 0 |

> 註：top 封鎖 literal 域名（如 DataDog log intake、各家 analytics endpoint）本質為**公開廣告/追蹤域名清單**、非機密，但因主理人明確指示「所有域名與 IP 脫敏」，故一律以類別取代 literal。

---

## Verdict：🔴 **HOLD**

**理由（fail-safe，保守）**：
1. **規模對不上 claim**。raw 源僅涵蓋 **41 天 / 541 萬筆 / 5 端點**，而 claim 主張「**四個月 / 5000 萬筆**」。即便以本檔速率（~13.2 萬筆/日）外推至 120 天也僅 ~1,580 萬筆，**距 5000 萬筆差約一個數量級**。本單一檔案**無法**獨立支撐 claim 的時間跨度與總量。
2. **封鎖率定義未鎖定**。「10%」僅在「request-blocked ÷ 總事件 = 11.05%」這一特定分母下成立；改分母可得 13.9%~29.5%。claim 須補上分母定義才能稱「精確」。
3. 觀測管線本身**真實且具多百萬級規模、封鎖機制確實運作**（此點 ✅），但這支撐的是「存在大規模 DNS 觀測與過濾」的**弱主張**，非 claim 字面的「50M/4mo/10%」**強主張**。

**交還作者待確認事項（NEEDS-AUTHOR 具體點）**：
- (a) 是否另有 log 檔覆蓋 2026-02~04 三個月，能補足「四個月 / 5000 萬筆」？若有，請併入重算。
- (b) claim 之「10% 封鎖率」採何分母？建議於論文明寫為「request-blocked ÷ 總查詢 = 11%」以可溯源。
- (c) 5 端點是否即全部觀測範圍？若 claim 之 50M 來自更多端點，本檔不具代表性。

> 在 (a)(b)(c) 釐清前，本證據**留置 staging，禁止上 github / evidence-prerelease**。
