# SASS Protocol I-D 自審報告

> **文件**: `draft-sakistudio-sass-00`
> **日期**: 2026-05-25
> **審閱層級**: RFC 出版級（IESG 審查標準）
> **狀態**: Experimental (Independent Submission Stream)

---

## 一、格式檢查清單

| # | 檢查項目 | 狀態 | 說明 |
|---|---------|:----:|------|
| 1 | 行寬 ≤ 72 字元 | ✅ | 所有行已重排至 72 字元以內 |
| 2 | Header 格式（作者/組織/日期/到期日） | ✅ | `H. Chen / Saki Studio / 25 May 2026 / Expires: 26 November 2026` |
| 3 | Draft 名稱格式 | ✅ | `draft-sakistudio-sass-00`（符合 `draft-{stream}-{name}-{version}` 慣例） |
| 4 | Status of This Memo (BCP 78/79 boilerplate) | ✅ | 已使用 IETF 標準 boilerplate 文字 |
| 5 | Copyright Notice (IETF Trust) | ✅ | 改用 `IETF Trust` 而非原稿的 `Saki Studio` |
| 6 | Intended Status: Experimental | ✅ | 符合 Independent Submission 慣例 |
| 7 | Table of Contents | ✅ | 完整涵蓋所有章節，頁碼已更新 |
| 8 | 頁尾格式 | ✅ | `Chen    Expires 26 November 2026    [Page X]` |
| 9 | 頁首格式 | ✅ | `Internet-Draft    SASS Protocol    May 2026` |
| 10 | 分頁 (Form Feed) | ⚠️ | TXT 版使用空行分隔；XML 版由 xml2rfc 自動處理 |
| 11 | IANA Considerations 章節 | ✅ | Section 11，列出 ALPN ID、MIME type、TCP port |
| 12 | Security Considerations 章節 | ✅ | Section 10，共 6 個子章節 |
| 13 | References 分離（Normative/Informative） | ✅ | 12.1 Normative (12 篇)，12.2 Informative (5 篇) |
| 14 | RFC 2119/8174 引用與聲明 | ✅ | Section 1 尾段 + Section 2.1 獨立子章節 |
| 15 | xml2rfc v3 格式 (RFC 7991) | ✅ | 已產出 `.xml` 檔案，含 `<bcp14>` 標籤 |
| 16 | 章節編號連續 | ✅ | 原稿 `3.1a` 已改為 `3.2`，後續章節重新編號 |
| 17 | DOI 引用 | ✅ | 所有 RFC 引用已加入 DOI 與 URL |
| 18 | Authors' Addresses | ✅ | 包含全名、組織、城市、國家、Email |

### 格式修正摘要

| 修正項 | 原稿問題 | 修正方式 |
|--------|---------|---------|
| Copyright | `Copyright (c) 2026 Saki Studio` | 改為 `IETF Trust`（BCP 78 要求） |
| Status | `Intended status: Informational` | 改為 `Experimental`（任務要求） |
| Boilerplate | 不完整，缺 Internet-Drafts working documents 段落 | 補齊完整 boilerplate |
| Section `3.1a` | 非標準子章節編號 | 改為 `3.2`，後續重新編號 |
| 作者名稱 | 僅列 `Saki Studio` | 改為 `H. Chen`（IETF 要求自然人） |
| RFC 引用格式 | 簡略格式 | 補齊完整作者、DOI、URL |
| BCP 14 聲明 | 無獨立聲明段落 | 在 Section 1 尾段 + Section 2.1 加入 |
| 行寬 | 多處超過 72 字元 | 全文重排 |

---

## 二、內容自審清單（RFC 出版級）

### 2.1 Security Considerations 檢查 (RFC 3552 合規)

RFC 3552 要求 Security Considerations 必須討論以下面向：

| # | RFC 3552 要求 | 涵蓋狀態 | 對應章節 | 評估 |
|---|-------------|:--------:|---------|------|
| 1 | 通訊安全（竊聽、篡改） | ✅ | §10.4 (TLS 1.3 強制) | TLS 1.3 + EKM binding 已涵蓋 |
| 2 | 身份驗證 | ✅ | §5.1 (三階段驗證) | 傳輸層 mTLS + 應用層 ED25519 + 認知挑戰 |
| 3 | 授權 | ✅ | §5.3 (五維能力模型) | deny-first + TOCTOU 防護 |
| 4 | 完整性 | ✅ | §10.3 (Hash Chain 審計) | ED25519 + SHA256 鏈式審計 |
| 5 | 可用性 / DoS | ✅ | §8.3.2, §8.3.3 | Tarpit 並發閘 + Zero-Window 防護 |
| 6 | 密碼學假設 | ✅ | §4.2 (TLS 1.3 密碼套件) | AES-256-GCM + ChaCha20-Poly1305 |
| 7 | 已知攻擊向量 | ✅ | §10.4 (6 項已知威脅) | ALPN stripping, Huffman DoS, 0-RTT replay 等 |
| 8 | 殘餘風險 | ✅ | §10.5 (6 項已知限制) | Kernel sandbox 未實作、非 FS 副作用 |
| 9 | 隱私考量 | ⚠️ | 未明確討論 | **建議新增**：Agent 命令日誌的隱私影響 |

**RFC 3552 額外建議**：
- ⚠️ 未討論 **privacy considerations**（Agent 的命令紀錄是否構成 PII）
- ⚠️ 未討論 **downgrade attack** 的完整場景（雖已禁止 TLS 1.2 降級）
- ✅ 已涵蓋 **timing side-channels**（constant-time comparison, §8.2）

### 2.2 RFC 2119 / RFC 8174 術語檢查

| 檢查項 | 狀態 | 說明 |
|--------|:----:|------|
| 引用 RFC 2119 + RFC 8174 | ✅ | BCP 14 雙引用 |
| 獨立 Requirements Language 子章節 | ✅ | Section 2.1 |
| 正文中包含完整 boilerplate | ✅ | Section 1 尾段 |
| MUST/SHOULD/MAY 一致使用 | ✅ | 全文大寫使用，非規範性用法用小寫 |
| `<bcp14>` XML 標籤 | ✅ | xml2rfc XML 中所有關鍵詞已標記 |

### 2.3 References 檢查

| 引用 | 分類 | 存在性 | 格式 |
|------|------|:------:|:----:|
| RFC 2119 | Normative | ✅ 存在 | ✅ |
| RFC 3161 | Normative | ✅ 存在 | ✅ |
| RFC 4648 | Normative | ✅ 存在 | ✅ |
| RFC 5705 | Normative | ✅ 存在 | ✅ |
| RFC 7301 | Normative | ✅ 存在 | ✅ |
| RFC 8032 | Normative | ✅ 存在 | ✅ |
| RFC 8174 | Normative | ✅ 存在 | ✅ |
| RFC 8439 | Normative | ✅ 存在 | ✅ |
| RFC 8446 | Normative | ✅ 存在 | ✅ |
| RFC 8878 | Normative | ✅ 存在 | ✅ |
| RFC 8949 | Normative | ✅ 存在 | ✅ |
| RFC 9266 | Normative | ✅ 存在 | ✅ |
| [AS2008] | Informative | ✅ 存在 | ✅ (含 DOI) |
| RFC 4251-4254 | Informative | ✅ 存在 | ✅ |

**分類問題**：
- ⚠️ RFC 3161 (TSP) 目前列為 Normative，但正文中僅在 §10.4 提及「Mandates ... external TSP anchors」。若 TSP 非強制實作要求，建議降為 Informative。
- ⚠️ RFC 8439 (ChaCha20) 列為 Normative，但正文中 ChaCha20 是 Appendix C 的 OPTIONAL 實作。若正文無 MUST 使用 ChaCha20 的要求，建議降為 Informative。
- ⚠️ RFC 8032 (ED25519) 同上——正文均使用「MAY use ED25519」。

### 2.4 IANA Considerations 檢查

目前 Section 11 請求三項註冊：

| 請求項 | IANA Registry | 問題 |
|--------|--------------|------|
| ALPN ID: `x-sakirpc-v5` | TLS ALPN Protocol IDs | ⚠️ `x-` 前綴在 RFC 6838 後已被棄用。IANA 可能要求移除 `x-` 前綴 |
| MIME: `application/grpc+saki` | Media Types | ⚠️ 應提供完整的 Media Type 註冊模板（RFC 6838 §5.6） |
| TCP Port: 19284 | Service Name and Transport Protocol Port Number | ⚠️ 需提供 Service Name、Transport Protocol、Assignment Notes |

**建議**：
1. 若不打算正式向 IANA 註冊，應改為 "This document has no IANA actions" 並說明使用私有/實驗值
2. 若要正式註冊，需補充完整的 IANA registration templates

---

## 三、RFC 出版路線分析

### 3.1 提交路線比較

| 路線 | 要求 | 可行性 | 風險 |
|------|------|:------:|------|
| **Independent Submission (ISE)** | RFC Editor 審查 + IESG 衝突檢查 | ⭐⭐⭐⭐ | 最可行路線，不需 WG 共識 |
| **IETF Standards Track (secsh WG)** | WG 採納 + WG Last Call + IESG 審查 | ⭐ | secsh WG 已休眠；需建立新 WG |
| **secdispatch → 新 WG** | secdispatch 分流 → BOF → WG 成立 | ⭐⭐ | 需要 AD sponsorship + 社群支持 |
| **Experimental (IETF Stream)** | AD sponsor + IETF Last Call | ⭐⭐ | 需找到願意 sponsor 的 Security AD |

### 3.2 RFC 5743 合規 (Independent Submission)

| 要求 | 狀態 | 說明 |
|------|:----:|------|
| 不與現有 IETF 工作衝突 | ✅ | SASS 與 SSH 無線格式重疊 |
| 技術上合理 | ✅ | 基於標準密碼學原語 |
| 不含 IETF Standards Track 術語 | ⚠️ | 使用 BCP 14 關鍵詞是允許的，但 ISE 文件通常較保守 |
| IPR 聲明 | ❓ | **需作者確認**（見第四章） |
| 不請求 IETF 共識 | ✅ | `consensus="false"` 已設定 |
| 不創建新的 IETF 名字空間 | ⚠️ | ALPN ID 和 TCP port 請求可能需要 IANA Designated Expert 審查 |

### 3.3 IESG 常見退回原因

| # | 常見退回原因 | 本文狀態 | 風險等級 |
|---|------------|:--------:|:--------:|
| 1 | Security Considerations 不充分 | ✅ 通過 | 低 |
| 2 | 缺少 IANA Considerations | ✅ 有 | 中（格式需完善） |
| 3 | Normative/Informative 分類不當 | ⚠️ 需修正 | 中 |
| 4 | BCP 14 用法不一致 | ✅ 一致 | 低 |
| 5 | 與現有標準衝突 | ✅ §3.4 已聲明 | 低 |
| 6 | 技術正確性存疑 | ⚠️ AES 宣稱需審查 | 高 |
| 7 | 文件過長/結構不清 | ✅ 結構清晰 | 低 |
| 8 | 缺少互操作性考量 | ⚠️ Appendix B 較簡略 | 中 |
| 9 | 作者參與歷史 | ⚠️ 見下方分析 | 中 |
| 10 | 過度使用行銷語言 | ⚠️ 需人工審閱 | 中 |

### 3.4 Area Director Sponsorship 策略

**最佳路線**：Independent Submission Stream (ISE)

理由：
1. SASS 是全新協議，不修改現有標準——適合 Independent Stream
2. 不需要 AD sponsor（ISE 有自己的 Editor）
3. Experimental 類別降低審查門檻
4. 可作為未來 Standards Track 的前驅文件

**若選擇 IETF Stream**：
- 目標 Area：Security Area (sec)
- 潛在 AD：Security Area Director
- 建議先在 secdispatch 做 5 分鐘 lightning talk 測試反應
- 風險：「為什麼不用 OpenSSH ForceCommand」將是第一個問題

### 3.5 作者資格

| 項目 | 說明 |
|------|------|
| IETF 參與歷史 | Independent Stream 不嚴格要求 |
| Datatracker 帳號 | ✅ 需要（提交時必須） |
| IPR 聲明 | ✅ 必須提交 IPR disclosure |
| 組織歸屬 | 以個人名義提交更簡單（不需組織授權書） |

---

## 四、需要作者人工審閱的項目

> 以下項目涉及法律判斷、個人意圖、或部署經驗，AI 無法代替人類做出決定。

### 4.1 身份與法律

#### 4.1.1 IPR (Intellectual Property Rights) 聲明
- **問題**：此協議是否涉及任何專利（已申請或擬申請）？
- **為何 AI 無法代替**：專利持有狀態是法律事實，只有作者本人知道
- **建議**：若無專利，提交時選擇 "no known IPR"
- **對應章節**：整份文件
- **後果**：若有隱瞞 IPR 且被發現，文件將被撤回

#### 4.1.2 提交身份
- **問題**：以個人名義還是 Saki Studio 組織名義提交？
- **為何 AI 無法代替**：涉及組織內部授權流程
- **建議**：以個人名義提交最簡單（避免組織授權書）。XML 中 author 已設為 `H. Chen / Saki Studio`
- **對應章節**：Authors' Addresses

#### 4.1.3 IETF Trust 授權條款
- **問題**：是否接受 BCP 78 的授權條款？（將著作權部分權利授予 IETF Trust）
- **為何 AI 無法代替**：這是法律授權決定
- **建議**：標準做法是接受。若不接受，無法作為 I-D 提交
- **對應章節**：Copyright Notice

### 4.2 技術宣稱的人類判斷

#### 4.2.1 AES (Almost Everywhere Superior) 宣稱
- **問題**：Aumann-Serrano [AS2008] 的引用是否準確反映您的意圖？SSD 的定義在經濟學中有嚴格的數學意義——您的用法是否構成類比還是嚴格證明？
- **為何 AI 無法代替**：這涉及原作者的數學意圖和嚴謹度判斷
- **建議方向**：若為類比，建議加 "by analogy with" 修飾語；若為嚴格證明，需提供形式化證明
- **對應章節**：§10.6
- **IESG 風險**：數學宣稱的嚴謹度是常見退回原因

#### 4.2.2 6-Response 狀態機完整性
- **問題**：是否存在您知道但文件未提及的第 7 種回應？例如：RenewSession 失敗時的行為、Ping timeout 的行為、FileUpload 中途斷線的行為
- **為何 AI 無法代替**：實際部署中可能存在文件未覆蓋的邊界情況
- **建議方向**：逐一列舉所有 RPC method × 所有失敗模式，確認都映射到 R1~R6
- **對應章節**：§3.2

#### 4.2.3 13Policy 預設規則集
- **問題**：「至少 50 條模式」的要求是否符合您的實際部署場景？預設的 dangerous_commands 清單是否過於寬鬆或過於嚴格？
- **為何 AI 無法代替**：規則集的適當性取決於實際部署環境和使用者反饋
- **建議方向**：考慮在 Appendix 中列出推薦的最小規則集範例
- **對應章節**：§8.1

#### 4.2.4 Safety Gradient 七層存在性
- **問題**：七層中每一層是否都在您的實際部署中存在並運作？特別是 Layer 6 (Storage Sandbox - UVSF Core | KFS Kernel) 和 Layer 7 (Transparent Branching) 是否已完整實作？
- **為何 AI 無法代替**：實作狀態只有開發者知道
- **建議方向**：若某些層尚未實作，應在 §10.5 Known Limitations 中說明
- **對應章節**：§10.2

### 4.3 IETF 審查者可能問的問題

> 以下是 IESG 審查或 IETF Last Call 中很可能出現的問題。作者應準備回答。

#### 4.3.1 「為什麼不直接用 OpenSSH 的 ForceCommand？」
- **對應章節**：§1, §3.4
- **為何 AI 無法代替**：需要作者基於實際經驗解釋 ForceCommand 的不足
- **建議回答方向**：ForceCommand 是命令白名單，不提供 (a) 主動威脅防禦 (b) 能力模型 (c) 透明分支 (d) 串流 I/O。但應在文件中加入更明確的比較

#### 4.3.2 「與 Tailscale SSH / Teleport / Boundary 的差異？」
- **對應章節**：§1
- **為何 AI 無法代替**：需要作者評估競品並做出公正比較
- **建議回答方向**：SASS 專注於 AI Agent 威脅模型，而非人類使用者。考慮在 §1 或 Appendix 加入 Related Work 比較

#### 4.3.3 「為什麼選擇 gRPC 而非原生 SSH Channel？」
- **對應章節**：§4.2
- **為何 AI 無法代替**：這是架構設計決策，涉及作者的技術判斷
- **建議回答方向**：SASS 使用 Transport Decoupling，gRPC 只是一個 profile。SAMM 是核心，gRPC 是可替換的。文件中已有此論述

#### 4.3.4 「ChaCha20 認知挑戰是否構成 security theater？」
- **對應章節**：§8.2, Appendix C.1
- **為何 AI 無法代替**：需要作者解釋認知挑戰的實際威脅模型和有效性證據
- **建議回答方向**：認知挑戰不是 PoW——它驗證的是 Agent 持有的 pre-shared key 和 TLS session binding，不是算力。但 "cognitive challenge" 這個名稱可能誤導審查者

#### 4.3.5 「Vi Swap 是否違反 RFC 4251 §11.1？」
- **對應章節**：§8.3.1
- **為何 AI 無法代替**：這是倫理/設計哲學問題
- **建議回答方向**：RFC 4251 §11.1 討論的是人類使用者不應被欺騙。SASS 的目標是 Agent，而非人類。且 §3.4 已聲明 SASS 與 SSH 無關。但審查者可能不接受這個論點
- **風險等級**：🔴 高——可能成為 IESG 投票否決的原因

#### 4.3.6 「Tarpit 是否可能被用於 DoS 攻擊己方？」
- **對應章節**：§8.3.2
- **為何 AI 無法代替**：需要作者基於部署經驗評估風險
- **建議回答方向**：文件已在 §8.3.2 設計了並發閘（max 32）和 §8.3.3 的 send timeout。但應明確量化：32 個並發 Tarpit session × 320 秒 × 64 KiB = daemon 端最大資源消耗

### 4.4 措辭與語氣

#### 4.4.1 RFC 語氣調整
- **問題**：RFC 風格通常乾燥且精確。以下段落的語氣可能被視為過度「哲學」或「行銷」：
  - §1.2 "All unexpected behaviors are expected behaviors"
  - §8.1 "An Agent's actions are fundamentally non-special"
  - §10.6 "Almost Everywhere Superior" 名稱本身
- **為何 AI 無法代替**：語氣的可接受度是主觀判斷
- **建議方向**：
  - "All unexpected behaviors are expected behaviors" → 保留，這是形式化定義
  - "fundamentally non-special" → 考慮改為更技術性的表述
  - "AES" → 名稱可能引起與 AES 加密演算法的混淆，考慮改名
- **對應章節**：§1.2, §8.1, §10.6

#### 4.4.2 「Actions are non-special」論述
- **問題**：此論述是保留在正文（§8.1）還是移到 Appendix？
- **為何 AI 無法代替**：這是文件架構決策
- **建議方向**：保留在正文——它是理解 13Policy 設計的關鍵前提。但措辭可以更技術化
- **對應章節**：§1.2, §8.1

---

## 五、潛在問題與建議修正

### 5.1 高優先級（可能導致退回）

| # | 問題 | 建議修正 | 章節 |
|---|------|---------|------|
| H1 | RFC 8439、RFC 8032 列為 Normative 但正文僅 MAY 使用 | 降為 Informative，或在正文中加入 MUST 要求 | §12 |
| H2 | ALPN ID 使用 `x-` 前綴（RFC 6838 後已棄用） | 改為 `sakirpc-v5` 或聲明為實驗用途 | §4.2, §11 |
| H3 | IANA 請求缺少完整 registration template | 補充 RFC 6838 §5.6 Media Type template | §11 |
| H4 | Privacy Considerations 缺失 | 新增 §10.7 Privacy Considerations 或在 §10 中加入 | §10 |
| H5 | AES 名稱與 AES 加密混淆 | 考慮在首次出現時加註 "(not to be confused with the Advanced Encryption Standard)" | §10.6 |

### 5.2 中優先級（可能收到 DISCUSS）

| # | 問題 | 建議修正 | 章節 |
|---|------|---------|------|
| M1 | 未提及 Related Work（Tailscale SSH、Teleport 等） | 在 §1 或 Appendix 新增 Related Work 比較 | §1 |
| M2 | 互操作性測試描述過於簡略 | Appendix B 擴充測試矩陣描述 | App B |
| M3 | 「Cognitive Challenge」命名可能誤導 | 考慮改為 "Keying Material Verification" 或保留但加註解 | §8.2 |
| M4 | Error Code 範圍未明確說明預留空間 | 加入 "Codes 110-127 are reserved for future use" | §9 |
| M5 | Session UUID v4 的隨機性來源未指定 | 加入 "generated using a cryptographically secure random source" | §5.2 |

### 5.3 低優先級（建議改善）

| # | 問題 | 建議修正 | 章節 |
|---|------|---------|------|
| L1 | 原稿 `plugable` 拼寫錯誤 | 已修正為 `pluggable` | §1, §4.2 |
| L2 | Appendix 中引用 `~/.config/sass/` 路徑 | 加入 XDG_CONFIG_HOME 支援說明 | App C.4 |
| L3 | 未指定 JSON 序列化版本 | 加入 "JSON as defined in [RFC8259]" 並新增引用 | §3.3 |
| L4 | 版本歷史表與 I-D 版本號不對應 | 說明 draft-00 對應 SASS v1.4 | App D |

---

## 六、產出檔案清單

| 檔案 | 路徑 | 說明 |
|------|------|------|
| TXT 版 I-D | `docs/draft-sakistudio-sass-00.txt` | 手工排版，72 字元行寬 |
| XML 版 I-D | `docs/ietf-submission/draft-sakistudio-sass-00.xml` | xml2rfc v3 格式，可用 `xml2rfc` 工具生成合規 TXT/HTML |
| 自審報告 | `docs/IETF_SELF_REVIEW.md` | 本文件 |

### 下一步建議

1. **人工審閱**：完成第四章所有需要人類判斷的項目
2. **修正高優先級問題**：至少處理 H1~H5
3. **xml2rfc 驗證**：執行 `xml2rfc --v3 draft-sakistudio-sass-00.xml` 驗證 XML 格式
4. **idnits 檢查**：使用 IETF 的 `idnits` 工具進行最終格式驗證
5. **提交前**：在 [Datatracker](https://datatracker.ietf.org/) 建立帳號並提交 IPR 聲明
