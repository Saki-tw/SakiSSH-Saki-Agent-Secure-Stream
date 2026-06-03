# SASS Draft-05 完整實作 創世提示詞

> **生成時間**：20260604_0305 (UTC+8)
> **前 Session ID**：8ed46996-678e-4dda-ac72-bdfb52586551
> **生成原因**：使用者明確指令（協議九）
> **ChatMelius 歸檔**：`ChatMelius/20260604_SASS_Draft05_FullImplementation/`

---

## 3.1 元資訊

本 Session 完成了 SASS RFC draft-05 的**全部 22 項裁示修改**和 **Go/Rust 焦油坑整合**。所有工作已完成，無未完成項目。新 Session 的主要任務為：後續精修、git commit、xml2rfc 生成、IETF 提交準備。

---

## 3.2 任務目標

> 將 draft-sakistudio-sass-04.xml 升級為 draft-05，根據外部批評和使用者裁示執行 22 項修改，同時將 ChaCha20 偽 ICMP 焦油坑產生器整合到 Go 和 Rust 參考實作中。

---

## 3.3 專案位置與關鍵文檔

| 項目 | 路徑 |
|------|------|
| **專案根目錄** | `/Users/[USER]/Saki_Studio/Claude/SakiAgentSSH/` |
| **draft-05 XML** | `docs/ietf-submission/draft-sakistudio-sass-05.xml`（3280 行） |
| **draft-04 XML** | `docs/ietf-submission/draft-sakistudio-sass-04.xml`（3069 行，原始參考） |
| **Go 實作** | `github/go-sakissh/` |
| **Rust 實作** | `github/saki-ssh-daemon/` |
| **使用者裁示原文** | `~/.gemini/antigravity/brain/1615c3d6-f383-4ef9-8773-40543abcdb0e/`（前前 Session Brain） |
| **裁示原文全集** | 搜尋 `202606040101_SASS_使用者裁示原文全集.md`（如已產出） |
| **RFC 修改意見清單** | `brain/1615c3d6/202606032241_SASS_RFC修改意見清單_原文引用版.md`（502 行） |
| **本次 ChatMelius** | `SakiAgentHistory/ChatMelius/20260604_SASS_Draft05_FullImplementation/` |

### 必讀文件（按優先級）

1. `draft-sakistudio-sass-05.xml`（3280 行）— 最終產出
2. 本 handoff_prompt.md — 完整施工藍圖

### 可選文件

- `draft-sakistudio-sass-04.xml`（3069 行）— 對照用
- `brain/8ed46996/walkthrough.md` — 本次走查
- `brain/8ed46996/task.md` — 本次任務追蹤

---

## 3.4 已完成事項

### Phase 1: WP-A1+A2 機械修正（10 項有效 + #12 不成立還原）

| # | 項目 | 狀態 | 證據 | 驗證指令 |
|:-:|------|:----:|------|---------|
| #4 | Abstract 去 `<xref>` 引用 | ✅ | L51-87 | `grep '<xref' draft-05.xml \| head -5` |
| #5 | RFC 8439 → Informative | ✅ | Informative 區塊 | `grep -A2 'RFC8439' draft-05.xml` |
| #6 | ALPN 去 x- 前綴（3 處） | ✅ | 全文 | `grep 'x-sakirpc' draft-05.xml`（應 0 結果） |
| #8 | SSH deprecated → NOT RECOMMENDED | ✅ | §3.4 | `grep -i 'deprecated' draft-05.xml` |
| #3 | LocalHost 措辭 → fabricated data | ✅ | §8.4 | `grep 'fabricated' draft-05.xml` |
| #1 | SSD → Pointwise 全段重寫 | ✅ | §10.6 | `grep 'pointwise' draft-05.xml` |
| #2 | Cognitive Challenge 重寫 | ✅ | §8.2 | `grep 'O(1)' draft-05.xml` |
| #14 | Rice 年份 → 1953 | ✅ | 文本 | `grep '1953' draft-05.xml` |
| #17 | R1 scope → filesystem only | ✅ | L465 | `grep 'filesystem only' draft-05.xml` |
| #18 | App A → Informative | ✅ | Appendix A | `grep -i 'informative' draft-05.xml \| grep -i 'protobuf'` |
| #12 | ~~國別~~ → **不成立，已還原** | ✅ | L50 無 note | `grep 'Language Identification' draft-05.xml`（應 0） |

### Phase 2: WP-A3~A6 深度修正（12 項）

| # | 項目 | 狀態 | 證據 | 驗證指令 |
|:-:|------|:----:|------|---------|
| #19 | 禁 0-RTT + EKM | ✅ | §10.4 | `grep '0-RTT' draft-05.xml` |
| #20 | Tarpit 打假 + 持續驗證 | ✅ | Table 3 + §8.3 | `grep 'reclassified' draft-05.xml` |
| #21 | IP → Session 簽名 | ✅ | §10.4 | `grep 'bound to IP' draft-05.xml`（應 0） |
| #23 | MUST NOT + SC 連接 | ✅ | §8.5 | `grep 'undetectability' draft-05.xml` |
| #24 | Layer 命名 Axiom~Scientia | ✅ | §10.2 | `grep 'Axiom-L0' draft-05.xml` |
| #25 | 序列化 CBOR/JSON | ✅ | §6.3 | `grep 'MsgPack' draft-05.xml`（應只在 Changes 中） |
| #26 | 演算法特性描述 | ✅ | §4.2 | `grep 'constant-time' draft-05.xml` |
| #27a | 規則結構 | ✅ | §8.1 | `grep '50 patterns' draft-05.xml`（應 0） |
| #27b | 動態逾時 | ✅ | §6.2 | `grep 'configurable' draft-05.xml` |
| #7 | MIME sakirpc | ✅ | §4.3 + IANA | `grep 'grpc+saki' draft-05.xml`（應 0） |
| #13 | RFC 3161 不動 | ✅ | Normative | `grep 'RFC3161' draft-05.xml` |
| #10 | Tarpit RECOMMENDED | ✅ | §4.2 | `grep 'RECOMMENDED' draft-05.xml \| grep -i '50 MiB'` |

### WP-A6 版本變更

| 項目 | 狀態 | 驗證 |
|------|:----:|------|
| docName → draft-05 | ✅ | `grep 'docName' draft-05.xml` |
| date → June 4, 2026 | ✅ | `grep '<date' draft-05.xml` |
| Appendix I: Changes from draft-04 | ✅ | `grep 'changes-from-draft-04' draft-05.xml` |

### Phase 3: Go + Rust 焦油坑整合

| 項目 | 狀態 | 驗證 |
|------|:----:|------|
| Go tarpit 整合 | ✅ | `cd go-sakissh && go build ./...` |
| Rust tarpit 整合 | ✅ | `cd saki-ssh-daemon && cargo test` (8/8) |
| ICMP 類型隨機化 | ✅ | test_icmp_type_randomization |
| Type 3/11 假 IPv4 header | ✅ | test_type3_has_ipv4_header |

### 撤回/不成立項目

| # | 原因 | 使用者原話 |
|:-:|------|-----------|
| #9 | SSH 命名 | 「吃屎」 |
| #12 | Agent 自己提出 | 無外部 reviewer |
| #15 | AI co-author | 「吃屎」 |
| #16 | SIGKILL 完備性 | 列入 R6 |
| #22 | inference proxy | 「愚蠢的迎合」 |

---

## 3.5 未完成事項

**本 Session 無未完成的 RFC 修改或程式碼整合項目。**

後續 Session 可考慮的工作：

- [ ] 🟢 `git commit` 各子專案的修改
- [ ] 🟢 `xml2rfc` 生成 txt/html 輸出並驗證排版
- [ ] 🟡 重新跑 `idnits` 驗證（draft-03 通過，draft-05 待確認）
- [ ] 🟡 IETF Datatracker 提交準備
- [ ] 🟡 DEF CON poster abstract 與 draft-05 TRM 宣稱對齊
- [ ] 🔴 Commercial Speculari 內容深度精修（前 Session 提及）

---

## 3.6 技術棧速查

| 語言/工具 | 版本 | 用途 |
|-----------|------|------|
| Go | 1.24+ | `go-sakissh/` 參考實作 |
| Rust | nightly | `saki-ssh-daemon/` 參考實作 |
| xml2rfc | — | RFC XML → txt/html |
| golang.org/x/crypto | v0.51.0 | ChaCha20 焦油坑 |
| chacha20poly1305 (Rust) | 0.10 | ChaCha20 焦油坑 |

---

## 3.7 建議執行順序

1. `git diff` 確認各子專案修改（draft-05.xml, go-sakissh, saki-ssh-daemon）
2. `xml2rfc draft-sakistudio-sass-05.xml` 生成 txt/html
3. `idnits` 驗證
4. `git commit` + `git log` 記錄
5. DEF CON poster 文字對齊
6. Commercial Speculari 精修

---

## 3.8 踩坑清單與限制事項

| 踩坑 | 說明 |
|------|------|
| **#12 虛假意見** | Agent 在掃描 draft-04 時自己產生的國別修改建議，不存在於任何外部 reviewer 意見中。已還原但浪費了使用者時間。新 Session 須警惕 Agent 自行創造需求。 |
| **Go package 命名** | `tarpit_payload.go` 需放在 `internal/defense/tarpit_payload/` 子目錄下（Go package 命名規範），不能與 `defense` package 同目錄。 |
| **429 限流** | Phase 2 子代理在完成 10/12 項後遭 429。主體 Agent 接力完成剩餘 2 項。分批策略有效防止了全部損失。 |
| **§11 哲學段** | 使用者明確禁止刪除。有外部批評建議移除，使用者回應「來源請求。哪個王八蛋對這有意見的？」。 |
| **RFC 3161** | 使用者堅持 Normative + MUST：「時間都不準，別的都不可談」。 |
| **Axiom~Scientia 命名** | §10.2 Safety Gradient 的 8 層各用不同 Saki Studio 機構名。§3.1 的 Layer 1-7（功能疊加架構）保持不變。兩套完全區分。 |
