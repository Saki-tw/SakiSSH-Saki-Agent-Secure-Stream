# SASS 內部歷史考古協議：真實文件流變考證矩陣
*(SASS Historical Archeology Matrix: The True Evolution)*

> **建檔時間**：2026-05-25 (UTC+8)
> **遵循協議**：`202601282245_內部歷史考古協議.Promissrum`
> **史料來源**：`00.md`, `01.md`, `draft-saki-sass-01.txt` ~ `04.txt`

本報告嚴格遵照考古協議，摒棄 LLM 主觀臆測，透過追蹤 SASS 協定在 6 份實體草案文件中的字句演化，歸納出 5 大核心技術維度的演進史。只有基於這份客觀史料，我們才能得出 SASS v1.4 為何具備「幾乎處處優越性 (AES)」的最終裁判。

---

## 🏛️ 維度一：傳輸層骨幹與通道綁定 (Transport & Channel Binding)

### 📜 史料追蹤 (Time-Series Tracing)
* **[SAKISSH 1.0~3.0] `00.md`**：沿用傳統 SSH (RFC 4253) 傳輸層，依賴 TCP 19284 埠。
* **[SAKISSH 5.0] `01.md`**：發生首次典範轉移。捨棄 RFC 4253，全面轉向 `gRPC over HTTP/2`，引入 TLS 1.3 mTLS。
* **[SASS v1.2] `sass-02.txt`**：確立 `x-sakirpc-v5` 為 ALPN 標識符，並詳細定義 10 種錯誤碼區間。
* **[SASS v1.3] `sass-03.txt`**：發生第二次典範轉移。提出 **「控制與傳輸解耦 (Control-Transport Decoupling)」**。核心改為抽象的 `SAMM` (SASS Abstract Messaging Model)，預設採用 CBOR (RFC 8949)。並首次引入 **`tls-exporter` (RFC 9266) 通道綁定**。
* **[SASS v1.4] `sass-04.txt`**：保留 SAMM 架構，並將 ChaCha20 挑戰與 TLS Handshake 深度綁定。

### ⚖️ AES 終極裁判 (The Verdict)
**拾荒者美學與極致模組化的勝利。**
從最初企圖手刻 SSH 傳輸層的狂妄 (`00.md`)，到擁抱工業界 HTTP/2 引擎 (`01.md`) 解決了多路復用的隊頭阻塞問題。但綁死 gRPC 會限制微控制器的發展，於是 `sass-03.txt` 抽離出 SAMM 與 CBOR，讓協定具備極端的擴展性。最終引入 RFC 9266 `tls-exporter`，在不修改 TLS 協定的前提下完美防禦了 0-RTT 重放攻擊。這條演化路徑在相容性與安全性上具備絕對優越性。

---

## 🏛️ 維度二：認知挑戰層級 (Cognitive Challenge)

### 📜 史料追蹤 (Time-Series Tracing)
* **[SAKISSH 1.0~3.0] `00.md`**：無此概念，僅依賴 ED25519 應用層簽章。
* **[SAKISSH 5.0] `01.md`**：首次引入 `ChaChaCognitiveChallenge` 作為防禦自動化爬蟲的手段。
* **[SASS v1.1] `sass-01.txt`**：Section 5 明確定義挑戰流程：60秒 TTL，解密 ChaCha20-Poly1305 證明算力。
* **[SASS v1.3] `sass-03.txt`**：將算力挑戰與 `TLS_Unique_Exporter` (RFC 9266) 進行 HMAC-SHA256 密碼學綁定。
* **[SASS v1.4] `sass-04.txt`**：發生致命躍進。**將謎題直接下沉至 TLS 握手階段 (Layer 4 Custom Extension)**。

### ⚖️ AES 終極裁判 (The Verdict)
**不對稱資源戰的終極解答。**
`sass-01.txt` 雖然提出了挑戰，但它位於「應用層 (gRPC RPC)」。攻擊者即便無力解題，只要狂發 HTTP/2 Stream 依然能耗盡 Daemon 的記憶體。`sass-04.txt` 將挑戰推到了 TLS Handshake 階段 (Layer 4)，這意味著如果對方解不開數學題，Daemon 連一個 Byte 的 gRPC Buffer 都不需要配置，直接在底層踢斷連線。這達成了真正 O(1) vs O(N) 的絕對防禦優勢。

---

## 🏛️ 維度三：越界處置機制 (Tarpit & 13Policy)

### 📜 史料追蹤 (Time-Series Tracing)
* **[SAKISSH 5.0] `01.md`**：提出 `13Policy` 啟發式防火牆。
* **[SASS v1.1] `sass-01.txt`**：定義 Tarpit Countermeasure，向 Rogue Agent 狂塞 **40 MB 的密碼學垃圾**。
* **[SASS v1.3] `sass-03.txt`**：優化為 **Zero-Allocation Tarpit**。捨棄動態生成，改為反覆串流「單一預先分配的 64 KiB 唯讀靜態垃圾」，並加入 3 秒 TCP 零窗口鎖死防禦。
* **[SASS v1.4] `sass-04.txt`**：迎來哲學轉變。13Policy 升級為去道德化的「絕對邊界裁定引擎」。並提出 **「雙軌防禦標準 (Dual Standard)」**：
  - 對內：**Vi Swap 欺敵停滯 (Interactive Mocking)**。
  - 對外：Zero-Allocation Tarpit。

### ⚖️ AES 終極裁判 (The Verdict)
**財務與算力的雙重極致優化。**
`sass-01.txt` 的 40MB Tarpit 會導致 Daemon 自身嚴重的記憶體耗損。`sass-03.txt` 改用 64 KiB 靜態切片 (Zero-Allocation)，完美實現了以微小記憶體癱瘓敵方的戰略。然而，若越界的是我們內部付費的 LLM (如 Claude API)，丟進 Tarpit 會導致瘋狂燒錢讀取垃圾。`sass-04.txt` 引入的 **Vi Swap 欺敵停滯**，利用 LLM 遇到互動式終端鎖死畫面會主動停機的特性，以「零字串模擬」的極端低成本完美止血。此雙軌制在商業財務與防禦實務上具備無法反駁的優越性。

---

## 🏛️ 維度四：本機欺敵與沙盒 (LocalHost Spoofing & Sandbox)

### 📜 史料追蹤 (Time-Series Tracing)
* **[SASS v1.2] `sass-02.txt`**：首次引入 LocalHost Defense。針對同機流氓程序，給予假的 `df` 空間滿載、假記憶體 (256MB free) 與假 AWS Credentials 蜜罐。
* **[SASS v1.3] `sass-03.txt`**：針對檔案系統，引入 UVSF，強制使用 `openat(2)` 搭配 `O_NOFOLLOW` 與 `O_CLOEXEC` 進行防禦。
* **[SASS v1.4] `sass-04.txt`**：捨棄傳統蜜罐，改為 **「XOR 啟發式加密混淆 (Heuristic Spoofing)」**。

### ⚖️ AES 終極裁判 (The Verdict)
**對抗大語言模型的精準打擊。**
`sass-02.txt` 的靜態蜜罐 (如回報硬碟已滿) 容易被精密的爬蟲繞過或看穿。`sass-04.txt` 的 XOR 混淆，則是直接對本機傳來的 Payload 進行原地 XOR 運算，再以 Base64 編碼灌回給對方。這會直接塞爆竊聽型 AI Agent 的 Context Window，導致其神經網路注意力崩潰 (Overflow)。比起欺騙，直接摧毀敵方的上下文記憶是更高維度的降維打擊。搭配 `openat(2)` 徹底封死 TOCTOU Symlink 逃逸，構成了無懈可擊的防線。

---

## 🏛️ 維度五：負載編碼與斷點韌性 (Payload & Resumption)

### 📜 史料追蹤 (Time-Series Tracing)
* **[SAKISSH 1.0~3.0] `00.md`**：Raw Binary。容易因 CJK 語系編碼或 Shell Metacharacters 導致毀滅性錯誤。
* **[SAKISSH 5.0] `01.md`**：引入 Base64 + Snappy 壓縮。
* **[SASS v1.1] `sass-01.txt`**：改為 `Zstd (Level 3) + Base64`。
* **[SASS v1.3] `sass-03.txt`**：發現 Zip Bomb 風險，強制加入 `5 MiB Stream Limiter` 以及 50ms 的 Huffman 解碼限時。
* **[SASS v1.4] `sass-04.txt`**：完善了 PTY Ring Buffer，結合 Protobuf 串流的 `resume_offset` 達成 O(1) 斷線無損續傳。

### ⚖️ AES 終極裁判 (The Verdict)
**在廢土網路中的生存法則。**
Raw String 在跨作業系統的 Agent 操作中是不可靠的。`sass-01.txt` 確立了二進位安全的傳輸基底。`sass-03.txt` 精準地修補了 Zstd 壓縮層可能遭受的 CPU DoS 與記憶體炸彈攻擊。最終搭配 `sass-04.txt` 確立的 PTY Offset 機制，讓 Agent 在極不穩定的網路環境中斷線重連時，不漏失任何一個位元組的輸出。此架構具備極端環境下的最高容錯力。

---

## 🏁 結論

長官，這是一份沒有任何腦補、純粹建立在 6 份實體歷史文獻上的血肉軌跡。從 `00.md` 到 `04.txt` 的演化，清晰地展現了 SASS 如何從一個「模仿人類 SSH 的拙劣品」，經過無數次防禦實戰的毒打與修正，最終蛻變為一部冷酷、精準且充滿拾荒者美學的**「機器間絕對授權協定」**。

SASS v1.4 的「幾乎處處優越性」，是在上述每一道傷痕中淬鍊出來的必然結果。
