# STLS — Ring 0 系統提示段數削減 claim 支撐證據（脫敏 staging）

**Claim ID**: STLS
**Claim 文字**: STLS 將 Ring 0 系統提示從 **55 段降到 20 段**，減少 **75%**。
**raw 源**: `SakiAgentSSH/docs/evidence/H1_stls_ring0_reduction.md`

---

## Tier

**T2** — 外部稽核可獨立重算。段數（55 預設 / 35 移除 / 20 保留）可由第三方依 raw 源 `:149-156` 所列三步驟複現：(1) 檢視 `PromptSectionCustomizationConfig`（Field 41）、(2) 計算未啟用 STLS 時的 Ring 0 段數、(3) 以 `cl100k_base` 比較前後 token 數。非 OS-kernel 攔截，故非 T3。

> ⚠️ T2 強度被 raw 源自承限制下調：`:93-94` 註明部分段名「inferred from protocol analysis」，內部廠商命名可能不同；故「段數」可重算，但「逐段身分」屬推定而非直接觀測。

## Traceability

- Claim「Ring 0 55 段 → 20 段」← raw 源 `:12`（Abstract）、`:35`（Removed 35/55）、`:98`（Retained 20/55）
- 段數削減率 **63.6%（35/55）** ← raw 源 `:14`（明示 "63.6% section reduction"）
- **>75%** 實為 **token** 削減（>8,000 → <2,000 tokens），**非段數** ← raw 源 `:15`（"**>75% token reduction**"）
- 移除機制（Field 41 `remove_prompt_sections` 注入，STLS Layer 3）← raw 源 `:17`、`:140`
- 三維決策矩陣（Ring Conflict / Mode Confusion / Functional Value）← raw 源 `:21-31`
- 35 段移除明細（A:14 Ring 衝突 / B:12 模式混淆 / C:9 低功能值）← raw 源 `:37-91`
- 20 段保留（18 原生 + 2 STLS 注入替代）← raw 源 `:100-128`
- 五層攔截架構 ← raw 源 `:132-145`

## 脫敏摘要

| 類別 | 原始 | 脫敏後 | 理由 |
|------|------|--------|------|
| 真實廠商名 | Vendor A / Cortex | 保留 | 已是 raw 源既定匿名代號（符合 ruleset 廠商→代號規則） |
| 操作者規則代號 | TaskMELIUS、§1–§8、brew→cargo→pip→npm、CJK/Traditional Chinese | 保留 | 屬作者署名 agency（ruleset 明列保留），且為被分析行為之描述、非機敏 |
| 路徑 | `SakiAgentSSH/docs/evidence/H1...` | 保留為相對路徑 | 屬作者命名空間，無真實機碼/IP/token |
| 段名（`planning_mode_*`、`user_global_rules` 等） | — | 保留 | 為**被分析對象**之 Ring 0 段標籤（部分為推定名），非操作者個資 |

leak-scan：real-name=0、IP=0、token/secret=0、operator 個資=0、OAuth/key=0。脫敏本身達標。

---

## 數據摘要（脫敏後）

| 指標 | 數值 | 來源 |
|------|------|------|
| 預設 Ring 0 段數 | 55 | `:12` |
| 移除段數 | 35 | `:35` |
| 保留段數 | 20（18 原生 + 2 STLS 注入） | `:98-128` |
| **段數削減率** | **63.6%** | `:14` |
| token 削減率 | **>75%**（>8,000 → <2,000） | `:15` |

### 移除分類（35 段）
- **A — Ring 衝突（14）**：壓制/覆蓋操作者 Ring 3 規則者，無條件移除（`:37-56`）
- **B — 模式混淆（12）**：CHECKPOINT 截斷後造成行為模式誤判者（`:58-75`）
- **C — 低功能值（9）**：冗餘或可由 STLS 注入段替代者（`:77-91`）

---

## Verdict

🔴 **HOLD**

理由（依 fail-safe 預設保守，下列任一即足以 HOLD）：

1. **Claim 數字與單位錯配（主因）**：claim 主張「55 段 → 20 段，**減少 75%**」，將兩個不同維度的數字併為一句。raw 源明確區分：
   - **段數**削減為 **63.6%**（35/55，`:14`）——這才是「55→20 段」對應的比例；
   - **75%** 是 **token** 削減（`:15`），對應 >8,000→<2,000 tokens，**與段數無關**。

   即「55→20 段」≠「減少 75%」。直接以本檔背書「段數減少 75%」會超出證據、且與 raw 源自身數字衝突。須作者擇一修正措辭：要麼「段數減少 63.6%」，要麼「token 減少 >75%（段數 55→20）」。

2. **逐段身分為推定**：raw 源 `:93-94` 自承部分段名係「inferred from protocol analysis」，內部廠商命名可能不同。總段數可獨立重算（T2 成立），但 35 段的逐項命名/分類屬推定，公開時若被當作直接觀測會有過度宣稱風險。

leak-scan 全項為 0、脫敏達標、可溯源、Tier=T2——脫敏面無瑕疵；但因上述 claim-證據單位錯配與逐段推定性質，**不**升級為 🟢/🟡，預設 🔴 待作者裁決措辭。
