# SASS (Saki Agent Secure Stream) RFC 全協定威脅建模前置分析

> 建立時間：2026-05-25 07:15 (UTC+8 Asia/Taipei)  
> 專案簡稱：SakiSSH  
> 類別：Scientia 知識與研究歸檔  
> 版本：v1.0 (協定自我打擊與威脅建模前置研究)  

---

## 一、方法論與研究目的

在 SASS (Saki Agent Secure Stream) 朝向 IETF RFC 提交標準化的過程中，協定必須面臨全球最頂尖密碼學家與系統安全審查員的「嚴苛審視」。

本研究旨在依據 **STRIDE 威脅建模模型** 與 **RFC 網路協議安全規範基準**，針對 SASS v7.0 七層防禦架構進行全面的自我打擊（Self-Attacking）。我們將以最挑剔、最不留情面的「攻擊者視角」，挖掘 SASS 在邊界解碼、Userspace 沙盒重導向、Tarpit 反制、認知挑戰以及 Hash Chain 稽核等處可能出現的最嚴重未預期行為，並為其設計無懈可擊的防禦對策，奠定 RFC 標準的學術與工程安全基礎。

---

## 二、評估指標與打擊領域

我們將針對以下五個核心威脅領域進行深度解構：

| 領域編號 | 威脅標的 | STRIDE 分類 | 攻擊者動機與手段 |
| :--- | :--- | :--- | :--- |
| **01** | **UVSF 沙盒穿透** | T (Tampering) / E (Elevation of Privilege) | 利用 Userspace 符號連結的操作時差 (TOCTOU) 或 Race Condition，穿透沙盒讀寫家目錄敏感密鑰。 |
| **02** | **Tarpit 自我反噬** | D (Denial of Service) | 惡意 Client 並行發起大量觸發 13Policy 的垃圾資料請求，藉此耗盡 Daemon 記憶體與頻寬資源，引發 Host DoS。 |
| **03** | **日誌鏈回滾與覆蓋** | T (Tampering) / R (Repudiation) | 攻擊者在取得同等進程權限後，竊取本機 Ed25519 私鑰，對歷史 JSONL 審計日誌鏈進行重簽與覆蓋，掩蓋入侵痕跡。 |
| **04** | **Zstd 解壓壓縮炸彈** | D (Denial of Service) | 發送精心構造的高壓縮率 Zstd Payload (Decompression Bomb)，在 Userspace 解碼時引發 OOM Crash 或 CPU 耗盡。 |
| **05** | **認知挑戰重放與預計算** | S (Spoofing) / E (Elevation of Privilege) | 在並行連線中監聽並重放 ChaCha20 Challenge 答案，或利用 PRNG 漏洞預計算隨機 nonce，偽造合法 Client。 |

---

## 三、協定七層防禦架構基準線 (SASS-7 Layer Model)

在進行自我打擊前，我們確立 SASS v7.0 的防禦層級規格：

```
+-----------------------------------------------------------------------+
|  Layer 6: I/O Sandbox — UVSF Engine (Symlink) | Kernel Engine (DEXT)   |
+-----------------------------------------------------------------------+
|  Layer 5: Audit Trail — Forward-Secure Hash Chain & Ed25519 PEM Sign  |
+-----------------------------------------------------------------------+
|  Layer 4: Session & Cap — ED25519 Token & 5-Dimensional ACL Control  |
+-----------------------------------------------------------------------+
|  Layer 3: Threat Defense — 13Policy Engine, ChaCha20, Tarpit, Local   |
+-----------------------------------------------------------------------+
|  Layer 2: Payload Encoding — Zstd + Base64 Envelope                   |
+-----------------------------------------------------------------------+
|  Layer 1: Transport Security — TLS 1.3 mTLS (ALPN: x-sakirpc-v5)      |
+-----------------------------------------------------------------------+
|  Layer 0: ACL — CIDR Network Filtering                                |
+-----------------------------------------------------------------------+
```

本前置研究將直接指導隨後展開的「全協議擴寫與最嚴苛 RFC Review」任務，確保我們提出的每一項修正都具備可再現性與極高的學術安全水準。
