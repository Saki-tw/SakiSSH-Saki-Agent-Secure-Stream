# draft-03 與 draft-04 之間的「更加幾乎優越」

> **Scientia** | 哲學研究
> **時間**：2026-05-25 18:42 (UTC+8)
> **主題**：尋找 03/04 裂隙中未被說出的洞見
>
> **參見**：
> - [202605251822_SASS_尋找安全核心哲學與等效實作研究_Scientia.md](./202605251822_SASS_尋找安全核心哲學與等效實作研究_Scientia.md) — 本篇四維完備框架中 WHAT（Total Response Mapping）與 HOW MUCH（Safety Gradient）的理論來源
> - [202605251840_SASS_draft03考古紀錄_Scientia.md](./202605251840_SASS_draft03考古紀錄_Scientia.md) — 本篇 03/04 裂隙分析所依賴的考古事實基礎
> - **RFC**：[draft-saki-sass-03](../docs/draft-saki-sass-03.txt)（HOW 來源）→ [draft-saki-sass-04](../docs/draft-saki-sass-04.txt)（WHY 來源 + 新版承載四維完備框架）
> - **實作模組**：`thirteen_policy.rs`（Partition 語義）、`challenge.rs`（TLS Pushdown 精神）、`audit.rs`（狀態變遷審計）
> - **索引**：[Scientia INDEX](./INDEX.md)

---

## 1. 三份文件各自看到了什麼

| 文件 | 看到了 | 沒看到 |
|------|--------|--------|
| **draft-03** | **HOW** — 訊息如何流動（SAMM、Transport Decoupling） | 為什麼不枚舉攻擊 |
| **draft-04** | **WHY** — 行為無固有危險（Actions are non-special） | 如何保持形式完整 |
| **我們的 Scientia** | **WHAT** — 所有輸入收斂至 6 回應（Total Response Mapping） | — |

前任在 03 和 04 之間崩潰，是因為他**同時看到了 HOW 和 WHY，但找不到 WHAT** 來統一它們。

---

## 2. 裂隙中的三個未被說出的洞見

### 洞見 A：Boundary Adjudicator ≠ Firewall，而是 Partition

draft-03 §8.1 稱 13Policy 為「heuristic firewall」。draft-04 §7.1 改稱為「Boundary Adjudicator」並寫下了一句精準的話：

> *"An Agent's actions are fundamentally non-special; they lack inherent 'danger' or 'malice.'"*

但他沒有走到底。如果行為**真的**無固有危險，那 13Policy 不是防火牆，不是啟發式引擎，甚至不是裁定器——它是一個**測度論分割（measure-theoretic partition）**：

```
Ω（所有可能的 Agent 輸入）= B ∪ B^c

B  = 授權邊界內的集合（→ R1: EXECUTE）
B^c = 授權邊界外的集合（→ R2~R6 之一）
```

「`sudo rm -rf /` 在 B 內就執行，`pwd` 在 B^c 就拒絕」——這不是啟發式判斷，這是**集合論的成員資格檢驗**。沒有灰色地帶。

**這比 draft-04 更優越之處**：draft-04 仍然把 13Policy 當作「規則引擎」，暗示規則可以有優先序、可以衝突、需要啟發式解析。但如果它是 partition，就不存在衝突——每個輸入**恰好屬於一個**分區。

### 洞見 B：TLS Pushdown 的精神是對的

draft-04 §4.2 寫 ChaCha20 MUST 在 TLS Handshake 執行。實作不可行（rustls 限制），但**精神是對的**：

> 防禦應在**可行的最低層**執行。

這個精神在 RFC/Plugins 分離下可以這樣表達：

- **RFC（約定）**：「The challenge mechanism SHOULD be performed at the lowest feasible protocol layer to minimize resource allocation before authentication.」
- **Plugins（實作）**：具體在 Application Layer + TLS Exporter 綁定實作。

RFC 不說「在 L4」，也不說「在 L7」——它說「在你能做到的最低層」。這讓：
- rustls 使用者在 Application Layer 做（合規）
- OpenSSL 使用者在 TLS Extension 做（也合規，且更優）
- 未來如果 rustls 支援 custom extension，升級無需改 RFC

**這比 draft-03 和 draft-04 都更優越**：03 沒提挑戰的層級；04 寫死了 L4。我們的表述讓協議在時間維度上也「幾乎處處優越」——不會因為某個 TLS 庫的限制而過時。

### 洞見 C：「幾乎處處」是 Itô 引理上的那個幾乎

> **勘誤**（2026-05-25 22:51）：原版本錯誤地將「幾乎處處」解釋為勒貝格測度上的實分析概念，隨後又過度修正為「僅是修辭」。使用者指出：這是 **Itô 隨機微積分**意義上的「幾乎必然」（almost surely, a.s.）。

「所有的預期行為幾乎處處優越」——這裡的「幾乎處處」定義在一個**概率空間** (Ω, F, {F_t}, P) 上：

- **Ω** = 所有可能的 Agent 行為序列的空間（樣本空間）
- **F** = 可觀測事件的 σ-代數（指令、認證狀態、網路來源等）
- **{F_t}** = 濾網（filtration）——Daemon 在時刻 t 已觀測到的歷史
- **P** = Agent 行為上的概率測度

Agent 的行為是一個**隨機過程**——你不知道它下一步要做什麼。每條樣本路徑 ω ∈ Ω 是一串行為序列（連線 → 認證 → 執行指令 → ...）。

6-Response 狀態機是一個**適應過程**（adapted process）：在每個時刻 t，Daemon 根據 F_t（已觀測的歷史）將當前行為映射到 R1~R6 之一。這個映射是 F_t-可測的——它只依賴已知資訊，不需要預知未來。

```
對於 P-a.s. 所有 ω ∈ Ω：
  ∀t：Response(ω, t) ∈ {R1, R2, R3, R4, R5, R6}
  且每個 Response 滿足：
    儲存體損失 = 0
    商業損失 ≤ 可界定值
    可被完整稽核
```

**「幾乎處處優越」= 對 P-a.s. 所有可能的行為路徑，協議回應都維持安全不變量。**

「不成立的零測集」是什麼？是那些**超出模型假設**的路徑——例如 ED25519 被量子計算破解、或硬體級旁通道攻擊繞過所有軟體層。這些事件在當前技術條件下的概率測度為零，但我們承認它們的存在。

這比勒貝格測度的解釋更自然，因為安全協議的本質**就是**與一個隨機對手的博弈——它天然是一個隨機過程問題，不是實分析問題。

### 洞見 C 補充：AES 是比較級，不是絕對級

> **使用者補充**（2026-05-25 22:53）：「我們專案裡的『這個行為幾乎更優越』也是這樣比出來的。」

「幾乎處處優越」的精確語義來自 **SSD（Second-order Stochastic Dominance，二階隨機優越性）**：

- SSD 的定義：給定兩個期望值相同的分佈 F 和 G，若 F 在二階積分意義上處處 ≤ G，則 F SSD 於 G。直覺上：F 的「風險更低」。
- 你**沒辦法量化**「安全性 = 87 分」——這不是一個基數（cardinal）概念。
- 你只能拿出兩個東西——SASS 和 raw SSH——兩者都能讓 Agent 遠端執行指令（期望值相同），然後說：**SASS 的損失分佈 SSD 於 raw SSH**。

```
損失分佈比較：

              raw SSH (F_ssh)          SASS v1.4 (F_sass)
              ─────────────           ──────────────────
期望值:        E[L] = μ                E[L] = μ          （功能等效）
損失變異:      Var[L] = 高             Var[L] = 低        
尾端風險:      P(L > 災難) = 顯著      P(L > 災難) ≈ 0   

∴ F_sass SSD F_ssh
∴ SASS 的行為「幾乎處處優越」於 SSH
```

這意味著 AES 不是一個安全性的**絕對聲明**（「我們安全」），而是一個**比較聲明**（「我們比替代方案在損失分佈的二階意義上更優越」）。這也解釋了為什麼 SASS 的 RFC 不需要「證明安全」——它只需要展示在每個可比較的維度上，損失的隨機性都更低。

---

## 3. 統一框架：四維完備性

03 和 04 之間的裂隙，在加入 Total Response Mapping 後，被填補為一個**四維完備框架**：

| 維度 | 回答 | 來源 |
|------|------|------|
| **HOW**（形式） | SAMM — 傳輸無關的抽象訊息模型 | draft-03 |
| **WHY**（哲學） | Actions are non-special — 行為無固有危險 | draft-04 |
| **WHAT**（映射） | Total Response Mapping — 6 回應全域映射 | 本 Session |
| **HOW MUCH**（損失界定） | Safety Gradient — 7 層逐層損失界定 | 本 Session |

**任何一個維度缺席，協議都不完備**：
- 缺 HOW → 知道做什麼但不知道怎麼傳遞
- 缺 WHY → 知道怎麼做但不知道為什麼這樣做
- 缺 WHAT → 知道為什麼但不知道做什麼
- 缺 HOW MUCH → 知道做什麼但不知道能承受多少損失

前任在 03 有 HOW，在 04 新增了 WHY，但他缺 WHAT 和 HOW MUCH。他試圖用 HOW + WHY 去產生 WHAT，但那需要的是**研究**（Scientia），不是**更多的 RFC 條文**。這就是他崩潰的原因——他試圖用寫作解決需要思考的問題。

---

## 4. 對 draft-saki-sass-04（新版）的影響

新的 draft-04 應該在 Introduction 中明確宣告四維完備性，並在正文中分別用四個章節承載：

```
§3  Protocol Architecture (HOW) — SAMM 繼承自 03
§4  Design Philosophy (WHY) — Partition 不是 Firewall
§5  Total Response Mapping (WHAT) — 6-Response 狀態機
§10 Safety Gradient (HOW MUCH) — ε^7 → 0 損失界定
```

這讓 SASS v1.4 不只是「比 v1.3 多了幾個功能」，而是**在架構完備性上比任何前版本都更優越**。

而這個完備性本身，幾乎處處成立。
