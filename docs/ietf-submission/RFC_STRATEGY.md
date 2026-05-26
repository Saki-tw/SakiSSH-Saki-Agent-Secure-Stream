# SASS RFC 推進策略指南

> 從 Internet-Draft 到正式 RFC 的完整路線圖與策略建議。

---

## 一、三條路線比較

| 項目 | secsh Working Group | secdispatch → 新 WG | Independent Stream (ISE) |
|------|--------------------|--------------------|------------------------|
| **適用情境** | SSH 擴展協定，最對口 | 若 secsh 不採納，可請求分配 | 不需 WG 共識，獨立提交 |
| **優勢** | 社群認可度最高、reviewers 最熟悉 SSH | 有機會成立專門 WG | 流程最簡單、不需政治操作 |
| **劣勢** | 需取得 WG 共識、可能被要求大改 | 流程漫長、需要說服 Area Director | RFC 標記為 `Informational`，非 Standards Track |
| **預估時程** | 12-24 個月 | 18-36 個月 | 6-12 個月 |
| **最終 RFC 狀態** | Proposed Standard | Proposed Standard | Informational |
| **推薦度** | ★★★★★ | ★★ | ★★★★ |

### 推薦路線

**優先走 secsh WG → 若不採納則轉 Independent Stream**

理由：
1. SASS 本質是 SSH Agent 的簽章擴展，secsh 是最對口的 WG
2. 即使 secsh 不採納，Independent Stream 的 Informational RFC 仍有實質參考價值
3. 很多重要協定（如 SSH 自身的早期 RFC）都是先以 Informational 發布

---

## 二、Area Director 聯繫範本

SASS 屬於 **Security Area (SEC)**，需聯繫 SEC Area Director。

> 可在 <https://www.ietf.org/about/groups/iesg/> 查詢當前 SEC Area Director。

### Email 範本

```
Subject: New Individual I-D: draft-sakistudio-sass-00 (SSH Agent Signature Scheme)

Dear [SEC Area Director Name],

I have submitted an Individual Internet-Draft titled "SASS: SSH Agent 
Signature Scheme" (draft-sakistudio-sass-00) and would appreciate your 
guidance on the appropriate venue for its progression.

SASS defines a signature scheme that leverages existing SSH Agent 
infrastructure (RFC 4253 key pairs) for general-purpose document and 
message signing, without requiring additional key management overhead. 
The protocol introduces an envelope format with domain separation, 
timestamp binding, and algorithm agility while maintaining backward 
compatibility with existing SSH deployments.

Key aspects:
- Extends SSH Agent Protocol (draft-miller-ssh-agent) for signing 
  arbitrary data
- Provides domain separation to prevent cross-protocol signature reuse
- Supports Ed25519, ECDSA (P-256/P-384), and RSA key types
- Defines a compact binary envelope format

I believe this draft would be most appropriate for the secsh Working 
Group, as it directly extends SSH Agent functionality. However, I am 
open to your recommendation on the best path forward.

The draft is available at:
https://datatracker.ietf.org/doc/draft-sakistudio-sass/

I would welcome any feedback or direction you can provide.

Best regards,
[Your Name]
Saki Studio
[Your Email]
```

---

## 三、WG Adoption 流程

### 步驟 1：加入 secsh 郵件列表

- 列表地址：<https://www.ietf.org/mailman/listinfo/curdle>
  > 注意：目前 SSH 相關討論多在 `curdle` 列表（Ciphersuites for DTLS, TLS, SSH）
- 訂閱後先潛水觀察 1-2 週，了解社群討論風格

### 步驟 2：在郵件列表發布 Draft

```
Subject: [curdle] New I-D: draft-sakistudio-sass-00 (SSH Agent Signature Scheme)

Hi all,

I'd like to introduce a new Individual I-D that defines a signature 
scheme leveraging existing SSH Agent infrastructure.

draft-sakistudio-sass-00: SASS: SSH Agent Signature Scheme
https://datatracker.ietf.org/doc/draft-sakistudio-sass/

Problem statement:
SSH key pairs are already widely deployed and managed, but there is no 
standardized way to use SSH Agent for general-purpose signing beyond 
SSH authentication. Users who want to sign documents or messages must 
maintain a separate key management infrastructure (GPG, etc.).

SASS addresses this by defining a protocol for SSH Agent-based signing 
with proper domain separation, timestamp binding, and a compact 
envelope format.

I welcome any feedback and would be interested in whether the WG 
considers this in scope for adoption.

Thanks,
[Your Name]
```

### 步驟 3：WG Adoption Call

- 若 WG 有興趣，Chair 會發起 **adoption call**（通常 2 週投票）
- 需要足夠的支持者（不需全體一致，但需明確多數）
- 若通過，Draft 名稱會從 `draft-sakistudio-sass` 改為 `draft-ietf-curdle-sass`

### 步驟 4：WG Last Call → IETF Last Call → RFC

- WG 內部 review 與修訂（可能數輪）
- WG Last Call（2 週）
- IESG Review
- IETF Last Call（2 週）
- RFC Editor 編輯
- 發布為 RFC

---

## 四、IETF Meeting 策略

### 是否需要出席 IETF Meeting？

**強烈建議至少遠端參加一次**，理由：

1. 面對面（或視訊）的 presentation 對 WG adoption 有顯著幫助
2. 可以即時回應社群疑問，避免郵件列表上的延遲溝通
3. 展現作者對 Draft 的認真態度

### IETF Meeting 時程

- IETF 每年舉辦 **三次** 會議（約 3月、7月、11月）
- 會議排程：<https://www.ietf.org/how/meetings/>
- 遠端參加費用約 USD $125（現場約 USD $800）

### Presentation 準備

1. 在會議前 **6 週** 向 WG Chair 申請議程時段
2. 準備 **10-15 分鐘** 的簡報（含 Q&A 時間）
3. 簡報重點：
   - 問題定義（為什麼需要 SASS？）
   - 技術方案概述
   - 安全性考量
   - 與現有方案的比較（GPG, Signify, Minisign）
   - 實作狀態

---

## 五、時程預估

### 理想路線（secsh WG 採納）

```
Month 0     ──  提交 I-D (draft-sakistudio-sass-00)
Month 1-2   ──  郵件列表討論、收集反饋
Month 3     ──  參加 IETF Meeting，做 presentation
Month 4-6   ──  根據反饋更新 Draft (-01)
Month 6-8   ──  WG Adoption Call
Month 8-14  ──  WG 內部 review 與修訂 (-02, -03, ...)
Month 14-16 ──  WG Last Call
Month 16-18 ──  IESG Review + IETF Last Call
Month 18-22 ──  RFC Editor 排程與編輯
Month 22-24 ──  發布為 RFC
```

### 備選路線（Independent Stream）

```
Month 0     ──  提交 I-D
Month 1-3   ──  社群反饋、確認不走 WG 路線
Month 3-4   ──  聯繫 ISE (Independent Submissions Editor)
Month 4-8   ──  ISE Review + 外部 Review
Month 8-10  ──  IESG Conflict Review
Month 10-12 ──  RFC Editor 編輯 + 發布
```

### 影響時程的因素

| 因素 | 加速 | 減速 |
|------|------|------|
| 現有實作 | 有 reference implementation ✅ | 僅有規格無實作 |
| 安全性 review | 已有外部安全分析 | 需要額外安全審查 |
| 社群支持 | 多人表示支持採納 | 社群質疑必要性 |
| Draft 品質 | 格式規範、無歧義 | 需要大量編輯修訂 |
| 與現有 RFC 衝突 | 無衝突、純擴展 ✅ | 需修改現有 RFC |

---

## 六、實用資源

| 資源 | 連結 |
|------|------|
| IETF Datatracker | <https://datatracker.ietf.org/> |
| I-D 作者工具 | <https://author-tools.ietf.org/> |
| RFC 格式指南 | <https://www.rfc-editor.org/rfc/rfc7322> |
| IETF 新手指南 | <https://www.ietf.org/about/participate/tao/> |
| secsh / curdle WG | <https://datatracker.ietf.org/wg/curdle/about/> |
| IETF Meeting 排程 | <https://www.ietf.org/how/meetings/> |
| Independent Stream | <https://www.rfc-editor.org/about/independent/> |
