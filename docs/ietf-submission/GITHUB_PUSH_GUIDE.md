# SakiAgentSSH GitHub 提交教學

> **時間**：2026-05-26 11:22 (UTC+8)
> **前提**：repo 已存在 `https://github.com/Saki-tw/SakiAgentSSH.git`，本地 ahead 6 commits + 大量未追蹤檔案

---

## 步驟一：確認 .gitignore 生效

已經幫你建好 `.gitignore`，排除了 `.DS_Store`、`.env`、`target/`、`.pem`、`.key` 等垃圾。

確認不會洩漏 secrets：

```bash
cd /Users/hc1034/Saki_Studio/Claude/SakiAgentSSH
cat .gitignore
```

## 步驟二：清除已追蹤的垃圾（如果有的話）

```bash
# 如果 .DS_Store 已經被追蹤過，先移除
git rm -r --cached .DS_Store 2>/dev/null
git rm -r --cached '*.DS_Store' 2>/dev/null
git rm -r --cached .env 2>/dev/null
```

## 步驟三：加入所有檔案

```bash
git add -A
```

## 步驟四：確認不會推 secrets

```bash
# 必須檢查！確認沒有 .env 或 .pem 在 staged 裡
git diff --cached --name-only | grep -E '\.env|\.pem|\.key|\.p12|ghp_'
# 如果有輸出 → 有問題，不要繼續
# 如果沒輸出 → 安全，繼續
```

## 步驟五：Commit

```bash
git commit -m "feat: SASS v1.4 — 全 Phase 實作 + RFC draft-sakistudio-sass-00

Phase 0: 17 模組整合 + 13 依賴
Phase 1: mTLS (TLS 1.3 + 向後相容)
Phase 2: ChaCha20 認知挑戰 + TLS Exporter 綁定
Phase 3: Zero-Alloc Tarpit + Vi Swap + PolicyVerdict 五級裁定
Phase 4: XOR 本機防禦
Phase 5: CJK 安全傳輸 + PTY Ring Buffer
Phase 6: RFC draft-sakistudio-sass-00 (IETF I-D 已提交)

AES → MAS (Martingale Almost-Surely Superior)
Aumann-Serrano (2008) SSD 理論基礎

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

## 步驟六：推

```bash
git push origin main
```

如果要求認證：

```bash
# 用 token 推（token 在 .env 裡）
git push https://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@github.com/Saki-tw/SakiAgentSSH.git main
```

## 步驟七：確認

去 https://github.com/Saki-tw/SakiAgentSSH 看到新 commit 就完成了。

## 步驟八：回來更新 RFC 的 Appendix B

推完後告訴我 repo URL，我把它補進 `draft-sakistudio-sass-00.xml` 的 Appendix B，重新提交。

---

## 懶人一鍵版

直接複製貼上整段：

```bash
cd /Users/hc1034/Saki_Studio/Claude/SakiAgentSSH && \
git rm -r --cached .DS_Store 2>/dev/null; \
git rm -r --cached .env 2>/dev/null; \
git add -A && \
git diff --cached --name-only | grep -E '\.env|\.pem|\.key|\.p12|ghp_' && \
echo "⚠️ 有 secrets！不要繼續！" || \
git commit -m "feat: SASS v1.4 — RFC draft-sakistudio-sass-00 (IETF I-D)

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>" && \
git push origin main
```
