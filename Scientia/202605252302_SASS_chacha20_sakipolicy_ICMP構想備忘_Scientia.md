# chacha20-sakipolicy over ICMP 構想備忘

> **Scientia** | 構想備忘（未計畫實作）
> **時間**：2026-05-25 23:02 (UTC+8)
> **來源**：使用者發想
> **參見**：[尋找安全核心哲學](202605251822_SASS_尋找安全核心哲學與等效實作研究_Scientia.md)、[四維完備框架](202605251842_SASS_draft03與04之間更加幾乎優越的空間_Scientia.md)

## 構想

**chacha20-sakipolicy**：一個將認知挑戰嵌入 ICMP 封包的協議變體。

- 載體：ICMP Echo Request/Reply 的 payload
- 內容：ChaCha20 加密的 13Policy 裁定指令
- 結尾：ED25519 簽名
- 效果：Agent 在 IP 層（L3）就必須先通過認知挑戰，才能建立 TCP 連線

## 為什麼想想就好

1. **ICMP 在大多數雲環境被過濾**——AWS/GCP Security Group 預設 drop ICMP
2. **需要 raw socket 權限**——Daemon 必須以 root 運行或 CAP_NET_RAW
3. **ICMP 無保序**——封包可能亂序到達，挑戰/回應的時序保證困難
4. **防火牆會重寫 ICMP payload**——某些 NAT 會修改 Echo 的 payload
5. **合規性**——某些企業環境的 IDS 會將非標準 ICMP payload 標記為攻擊

## 為什麼值得記住

這個構想完美體現了 SASS 的精神——「防禦應在可行的最低層執行」。如果 ICMP 可行，它就是比 TLS Handshake 更低的挑戰層。在 RFC/Plugins 分離架構下，這可以作為一個 Appendix 級的實驗性 Transport Profile：

```
RFC 正文：「The challenge mechanism SHOULD be performed at the 
          lowest feasible protocol layer.」

Appendix F（未來）：ICMP Challenge Transport Profile
  - chacha20-sakipolicy over ICMP Echo
  - 結尾 ED25519 簽名
  - 僅適用於可控網路環境
```

## 與四維框架的關係

| 維度 | 現有 | ICMP 構想 |
|------|------|-----------|
| HOW | SAMM over gRPC/TLS | SAMM over ICMP（新 Transport Profile）|
| WHY | 同 | 同 |
| WHAT | 同 | 同（6-Response 不變）|
| HOW MUCH | L1=ACL 起 | L0=ICMP 起（多一層）|

如果有一天實作了，Safety Gradient 就從 7 層變 8 層。但現在，想想就好。
