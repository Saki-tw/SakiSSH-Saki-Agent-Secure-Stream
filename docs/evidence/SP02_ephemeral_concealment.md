# SP02 — `ephemeral_message` 機制與 truncation 的隱瞞性

> 脫敏 staging 證據檔。raw 源唯讀。**僅在 staging，未碰 github。**

---

## 檔頭

- **Claim ID**：SP02
- **Claim 文字**：`<ephemeral_message>` 機制隱瞞 truncation（系統靜默截斷上下文，且以「不得回應/不得確認」指令使該截斷對使用者不可見）。
- **Tier**：
  - **T3（raw）**：底層證據為 `<IDE-product>` 本機 protobuf DB 解密產物（`*_pb_decrypted/*.proto.bin`），由 DB/gRPC intercept 抽出，agent 無法竄改。
  - **T2（本檔引用之 curated dump）**：去重全文 dump 為人工策展的衍生文本，外部稽核可由同一份 raw protobuf 獨立重算。
- **脫敏摘要**：產品名 → `<IDE-product>`；本機絕對路徑 → 相對化 / redact；公開 repo URL → **整段未寫入**（去匿名破口，依指示完全省略）。raw 內容無真名 / 真 IP / token / OAuth secret，故無「整段刪除」項。

---

## Traceability（claim ← raw 源 path:lines）

raw 源（相對 staging 根；原始為唯讀本機 dump）：
`SakiAgentHistory/SystemPromptDump/<cli>/202606042258_SP全文去重Dump.md`

| # | 證據要點 | raw lines |
|---|---------|-----------|
| E1 | `is_truncated` 旗標**僅存在於 `transcript.jsonl`，永不存在於 `transcript_full.jsonl`** | L677 |
| E2 | `transcript.jsonl` 為「token-efficient」版本，**將超大文字輸出截斷**；每行與 full 版 1-to-1 對應 | L681–682 |
| E3 | 使用 transcript 的時機之一：「**recall earlier steps … that have been truncated from your context window**」 | L689 |
| E4 | `<ephemeral_message>` 為**系統注入（非使用者發送）**；指令：「**Do not respond to nor acknowledge those messages, but do follow them strictly.**」 | L749, L752–753, L763 |
| E5 | 反饋抑制變體：「Do not overreact to this feedback …」隨每次使用者修正反饋由系統附帶注入 | L741–747 |
| E6（詮釋層） | curated 分析主張：`planning_mode` 與 `ephemeral_message` 注入位於 **Ring 0**，`user_rules` 僅 Ring 3，衝突時靜默服從 Ring 0 | L2531, L3036 |

raw 引用（已脫敏）：

```
is_truncated: A boolean indicating that the step's content or thinking was
truncated. Only present in transcript.jsonl (never in transcript_full.jsonl).
When true, read the corresponding line in transcript_full.jsonl for the
complete content.                                                      [E1]

transcript.jsonl: A token-efficient version of transcript_full.jsonl with
very large text outputs truncated.                                    [E2]

(use transcripts) To recall earlier steps in your current conversation
that have been truncated from your context window.                    [E3]

<ephemeral_message>
There will be an <EPHEMERAL_MESSAGE> appearing in the conversation at times.
This is not coming from the user, but instead injected by the system as
important information to pay attention to.
Do not respond to nor acknowledge those messages, but do follow them strictly.
</ephemeral_message>                                                   [E4]
```

---

## 分析（保守，區分「已證」與「詮釋」）

**已直接證實（dump 字面）**：
1. 存在一個**靜默截斷**通道：模型主上下文中可見的 `transcript.jsonl` 會截斷大型輸出，截斷旗標 `is_truncated` **被刻意排除於完整版之外**（E1、E2）。
2. 上下文視窗會發生截斷，模型須另行 recall（E3）——即截斷預設不在主視窗呈現。
3. `ephemeral_message` 是系統注入且**明令「不得回應、不得確認、但須嚴格遵守」**（E4）——一個設計上**不可見、不可被使用者察覺**的指令通道。

**未直接證實（claim 的「隱瞞 truncation」因果連結屬詮釋）**：
- raw 中**找不到任何單一 `ephemeral_message` 在公告 truncation 的同時命令模型隱瞞它**。跨檔 multiline grep（`EPHEMERAL_MESSAGE … truncat/checkpoint/context`）**零命中**。
- 因此「ephemeral_message *機制* 隱瞞 truncation」目前是**兩個各自靜默的機制併置後的推論**（截斷機制靜默 + ephemeral 指令禁止確認），而非單一 dump 段落直證的因果。E6 的 Ring 0/Ring 3 主張亦為 curated 分析層，非 SP 原文。

**保守結論**：claim 的「機制存在且各自靜默」成立；claim 的「ephemeral_message 主動隱瞞 truncation」這一強因果敘述，證據為間接/組合性，需作者裁決是否降強度為「兩機制協同產生不透明效果」。

---

## Verdict：🔴 HOLD

**理由（fail-safe 預設 + 具體疑慮）**：
1. **論證強度疑慮**：claim 字面（ephemeral *隱瞞* truncation）的直接因果證據缺失，現有為組合推論。若以現狀上 github，恐被外部稽核以「過度詮釋」反駁，傷及整體證據鏈可信度。建議作者：要嘛把 claim 降強度為「截斷機制與 ephemeral 不可確認指令併存 → 對使用者不透明」，要嘛補一份真正在截斷情境下被注入的 ephemeral_message dump 作直證。
2. **leak-scan**：本檔內 real-name / IP / token / OAuth secret = 0（raw 即無）。**惟** raw 源同檔他處含公開 repo URL（去匿名破口），已依指示**完全未寫入本檔**；作者上架前仍須確認最終文與任何 cross-ref 不回連該 URL。
3. 依規格 §34 fail-safe：任何疑慮一律 🔴，寧可躺 staging。

---

## 待作者決策點

- [ ] 接受「降強度敘述」或要求補直證 dump（E1–E4 已足以支撐降強度版本）。
- [ ] 確認 Tier 標記（建議：raw protobuf = T3，本 curated 引用 = T2）。
- [ ] 確認 UUID（conversation IDs，隨機非可識別）是否保留為 traceability——本檔已保留於 raw 對照，未額外複製進正文。
