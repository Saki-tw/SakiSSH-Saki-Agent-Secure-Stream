# SP01 — System Prompt 演進 v1.21.6 → v1.23.2：Ring 0 膨脹 +87%（脫敏 staging 版）

> 🔴 STAGING（非 github）。graduate 需作者勾「必須公開」+ 四 gate。
> **Tier**：T2（跨版 PB 解密清單 + 活體 SP 自我觀測，第三方可獨立重算）；可升 T3（附 pb_decrypted raw + SHA256）。
> **支撐 claim**：論文 abstract/sec3/sec7「vendor prompt inflated **87%** (10,500 → 19,600 tokens)」+ sec4:44「`ephemeral_message` 隱瞞 truncation」（補 `Ring0_report` 弱引）。
> **Traceability**：raw 源 = `SakiAgentHistory/Scientia/202605140022_SystemPrompt演進對比_v1216到v1232_Scientia.md`（§四 token 表、§2.4 CHECKPOINT、line 23 ephemeral_message 兩版保留）+ `SakiAgentHistory/SystemPromptDump/{0518,0525,0530,0603}_pb_decrypted`（跨版解密 dump）。
> **脫敏**：真實產品名 → `<IDE-product>`；版本號 v1.21.6/v1.23.2 保留（論文已用）；system-prompt tag 名保留（論文 sec2/sec4 已引）。
> **Leak-scan（待 P3 對抗驗證）**：real-name=0 / IP=0 / token/secret=0 / operator-path=0。

## 1. Ring 0 區塊 token 變化（per-block，v1.21.6 → v1.23.2）

| 區塊 | v1.21.6 | v1.23.2 | 變化 |
|------|:---:|:---:|:---:|
| identity | ~500 | ~300 | −40% |
| planning_mode | ~1,300 | ~2,500 | +92% |
| artifacts | (內嵌) | ~3,000 | 🆕 |
| persistent_context | ~800 | ~1,500 | +88% |
| web_application_development | ~2,000 | ~3,500 | +75% |
| communication_style | ~2,900 | ~200 | −93% |
| user_rules | ~2,000 | ~5,000+ | +150% |
| skills | ~1,000 | ~3,000+ | +200% |
| mcp_servers | 0 | ~500 | 🆕 |
| **估計總計** | **~10,500** | **~19,600+** | **+87%** |

## 2. 與論文 claim 的對應

- **87% 膨脹**：總計 10,500 → 19,600+，膨脹近一倍 = 論文招牌數字的原始算據。
- **`<ephemeral_message>` 載體**：兩版（v1.21.6 & v1.23.2）均保留此 tag — 即 sec4:44 所述「叫 agent 隱瞞 truncation」機制的承載結構，**非空泛內部報告**。
- **CHECKPOINT 起源**：v1.21.6 無 → v1.23.2 新增 `CHECKPOINT_TYPE_UNSPECIFIED`，是 context truncation 的核心機制（sec2/sec4 因果鏈源頭）。
- **planning_mode 權重集中**：`agentic_mode_overview`/`task_boundary_tool`/`notify_user_tool` 三 tag 移除、整合進 `<planning_mode>` → 解釋為何 planning_mode 在事故中支配行為（sec4 cross-incident pattern 2）。

## 3. 升 T3 路徑（OS-kernel/網路堆疊不可竄改）

- 附 `SystemPromptDump/{date}_pb_decrypted` 對應檔的 **SHA256**；
- pb 解密 dump 源自 gRPC 串流攔截（network stack 產出，agent 不可竄改）→ 與本表逐欄位比對即可第三方確定性重算 87%。
