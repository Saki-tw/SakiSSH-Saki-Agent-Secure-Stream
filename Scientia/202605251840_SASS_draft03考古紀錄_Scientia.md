# draft-saki-sass-03 考古紀錄

> **Scientia** | 文獻考古
> **時間**：2026-05-25 18:40 (UTC+8)
> **來源**：ChatMelius Session（前任 Claude 47 / Gemini Star 的協作成果）
>
> **參見**：
> - [202605251822_SASS_尋找安全核心哲學與等效實作研究_Scientia.md](./202605251822_SASS_尋找安全核心哲學與等效實作研究_Scientia.md) — 本篇「draft-04 需新增內容」清單中的 6-Response 狀態機、Safety Gradient、Transparent Branching 的理論基礎
> - [202605251842_SASS_draft03與04之間更加幾乎優越的空間_Scientia.md](./202605251842_SASS_draft03與04之間更加幾乎優越的空間_Scientia.md) — 以本篇的考古事實為基礎，進一步分析 03/04 裂隙中的未發掘洞見
> - **RFC**：[draft-saki-sass-03](../docs/draft-saki-sass-03.txt)（本篇分析對象）→ [draft-saki-sass-04](../docs/draft-saki-sass-04.txt)（繼承目標）
> - **索引**：[Scientia INDEX](./INDEX.md)

## 文件定位

`/Users/hc1034/Saki_Studio/Claude/SakiAgentSSH/docs/draft-saki-sass-03.txt` 是前任 Agent 在 Session 320~622 期間產出的 RFC 草案。版本為 SASS v1.3。

## 前任在 03 → 04 之間崩潰的原因分析

根據使用者描述「在 03 跟 04 之間把自己逼瘋了」，推測原因為：

### 矛盾點：同一文件中的雙重身份
draft-03 試圖在**同一份文件**中同時扮演兩個角色：
1. **抽象約定**（SAMM、Transport Decoupling、CBOR baseline）
2. **具體實作**（ChaCha20-Poly1305、TLS Exporter、Zero-Alloc Tarpit）

當前任嘗試將這兩者「統一」時，發現：
- 寫得太抽象 → 失去可實作性
- 寫得太具體 → 失去通用性
- 兩者交織 → 文件結構崩潰

### 解決方案（使用者 2026-05-25 確認）
**RFC vs Plugins 雙版本架構**：
- RFC Version（正文）= 純約定，零專利，任何人可實作
- Plugins Version（Appendix C）= Saki Studio 商業加值實作

這個分離讓 draft-03 的矛盾不再存在。

## draft-03 的精華（已繼承至 draft-04）

| 章節 | 精華 | 繼承狀態 |
|------|------|---------|
| §3.2 SAMM | 傳輸無關的抽象訊息模型 | ✅ 繼承 |
| §4.1 Transport Decoupling | 控制平面與傳輸解耦 | ✅ 繼承 |
| §4.3 ALPN mitigation | 封包拆分防禦 | ✅ 繼承 |
| §4.4 tls-exporter | RFC 9266 Channel Binding | → 移至 Appendix C |
| §5.3 TOCTOU defense | openat + O_NOFOLLOW | ✅ 繼承 |
| §6.2 Decompression Bomb | Huffman collision 50ms gate | ✅ 繼承 |
| §8.2 ChaCha20 | 具體認知挑戰實作 | → 移至 Appendix C |
| §8.3 Zero-Alloc Tarpit | 64KiB 靜態 buffer + 3s timeout | ✅ 繼承（正文保留抽象定義）|

## draft-04 需要新增的內容（draft-03 中不存在）

1. **6-Response 狀態機** (R1~R6) — 全域回應映射
2. **Safety Gradient** — 7 層損失界定
3. **Transparent Branching** — Micro Branch 透明分流
4. **PTY Ring Buffer** — 斷線冪等續傳
5. **State Transition Auditing** — 狀態變遷審計（非指令審計）
