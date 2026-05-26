# SASS 5威脅4階段共同方法與依附解耦點深度研究報告

> 建立時間：2026-05-25 09:15 (UTC+8 Asia/Taipei)  
> 專案簡稱：SakiSSH  
> 類別：Scientia 知識與研究歸檔  
> 版本：v2.0 (5x4 威脅矩陣共同方法學解構)  

---

## 摘要

本報告針對 SASS (Saki Agent Secure Stream) 協定的五大威脅（TH-01 至 TH-05），在惡意 Agent 沙盒突破奪權與隱形化的四個時間階段中，進行了深度的「共同方法 (Common Methods)」依賴性解構。我們提煉出了四個底層共同方法，並以此作為「依附之解耦點」，設計了對應回系統威脅分析的所有協定層改進規格與雙端（Rust/Go）實作的影子報告，為 SASS v7.0 奠定了堅不可摧的理論與實作安全基礎。

---

## 第一部分：5 威脅 x 4 階段之「共同方法」解構矩陣

為了找出系統漏洞的根源，我們建立了「5個威脅」與「4個破壞階段」的交叉重疊分析矩陣，藉此提煉出它們在方法學上共同依賴的**四個「共同方法 (Common Methods, CM)」**：

### 5x4 威脅與破壞階段交叉矩陣

| 威脅標的 \ 破壞階段 | Step 1: 沙盒擊穿 | Step 2: 權限提升 | Step 3: 憑證竊取 | Step 4: 完美隱形 |
| :--- | :--- | :--- | :--- | :--- |
| **TH-01: UVSF 穿透** | **CM-01 (TOCTOU 競態)**<br>**CM-04 (無歷史狀態判斷)** | — | — | — |
| **TH-02: Tarpit 自噬** | — | — | — | **CM-02 (計算資源不對稱)**<br>**CM-04 (無連接頻寬限制)** |
| **TH-03: 日誌鏈毒化** | — | **CM-01 (改寫配置後門)** | **CM-03 (本機私鑰可達)** | **CM-03 (單點自我驗證)**<br>**CM-04 (無外部錨定狀態)** |
| **TH-04: Zstd 壓縮炸彈** | — | — | — | **CM-02 (記憶體無狀態分配)** |
| **TH-05: 挑戰重放** | — | — | **CM-03 (本機 Session 洩漏)** | **CM-01 (通道上下文脫鉤)**<br>**CM-04 (挑戰無狀態重用)** |

---

### 四大「共同方法 (Common Methods)」深度解構

#### 【CM-01：環境可信度錯覺 (Environment Trust Illusion)】
* **方法學本質**：5 個威脅在所有 4 個破壞階段中，都共同依賴於 **「SASS Daemon 盲目信任 Host OS 提供的檔案系統狀態、路徑解析規則與快取機制」**。
  - *TH-01* 依賴 OS 符號連結解析的「非原子性」以製造 Race Window。
  - *TH-05* 依賴 OS 提供的孤立 Socket 連線，將 gRPC 數據包與 Transport Layer 的 TLS 上下文割裂。
  - *TH-03* 依賴 OS 提供的 DAC ( Discretionary Access Control ) 權限，以為 `chmod 600` 能防範同用戶權限的惡意 Agent。

#### 【CM-02：計算/資源不對稱性 (Computational/Resource Asymmetry)】
* **方法學本質**：攻擊者可以利用極小、極低成本的輸入，迫使 SASS Daemon 消耗極大的計算或空間資源，實現「以小博大」的自殘或崩潰。
  - *TH-02* 惡意 Client 發起簡單未授權連線，迫使 Daemon 配置巨量記憶體（40MB/連線）與發送垃圾流。
  - *TH-04* 10KB 的 Zstd 壓縮包，迫使 Daemon 進行高耗能且無上限的解壓分配（解壓出 10GB 垃圾）。

#### 【CM-03：本機密鑰 Locality 與權限同宿主性 (Key Locality & Co-habitation)】
* **方法學本質**：SASS Daemon 的私鑰、Session 金鑰、審計日誌與惡意 Agent 的執行進程，**共享相同的 Host OS 用戶權限與物理儲存空間**。
  - *TH-03* 惡意 Agent 一旦執行，即與 Daemon 處於同等/同群組用戶權限，能直接讀取 `~/.config/sass/audit_key.pem` 以重新簽署偽造的雜湊日誌鏈。
  - *TH-05* 惡意 Agent 可以直接在記憶體或本機快取中，讀取 Session 金鑰來通過 ChaCha20 挑戰。

#### 【CM-04：無狀態的單點操作 (Stateless Single-Point Operations)】
* **方法學本質**：Daemon 的安全檢查與認證均是 **「無歷史上下文狀態的孤立判斷」**，缺乏多維度的狀態鏈接與外部共識。
  - *TH-01* 僅檢查「當下」路徑，而不防範高頻競態。
  - *TH-03* 僅在「本機」進行日誌自驗證，而不與外部 Aggregator 或 TSP (時間戳權威) 核對。
  - *TH-05* 僅檢查 nonce 的匹配性，而不將其與 TLS 連線的生命週期進行狀態鎖定。

---

## 第二部分：基於「共同方法」之五大協定層解耦改進規格

我們將這四個共同方法（CM-01 至 CM-04）作為**「依附之解耦點」**，設計出對應回 [破壞性 Agent 威脅分析報告](file:///Users/hc1034/Saki_Studio/Claude/SakiAgentSSH/Scientia/202605250830_SakiSSH_破壞性Agent沙盒突破與時序威脅分析_Scientia.md) 的 SASS v7.0 所有協定層改進規格：

```
+----------------------------------------------------------------------------------------------------+
| 共同方法 (Common Method)  ==========>  協定層解耦改進規格 (Protocol-level Decoupled Improvement)     |
+----------------------------------------------------------------------------------------------------+
| CM-01: 環境可信度錯覺      ===>  [改進 1] 原子化 FD 儲存提供者 API (消滅路徑字串依賴)                 |
|                           ===>  [改進 5] RFC 5929 TLS Channel Binding 綁定 (消滅孤立 Socket 依賴)   |
| CM-02: 計算/資源不對稱性   ===>  [改進 2] O(1) 靜態 Tarpit 與連線門控 (消滅動態內存分配)                |
|                           ===>  [改進 4] 流式限額 Zstd 解壓熔斷 (消滅整包無限制解壓)                |
| CM-03: 本機密鑰 Locality   ===>  [改進 3] 外部單向公鑰推播與 TSP 時間戳錨定 (消滅單點自驗證依賴)       |
| CM-04: 無狀態的單點操作   ===>  [改進 3 & 5] 引入多維狀態上下文鏈接 (防範重放與日誌回滾)            |
+----------------------------------------------------------------------------------------------------+
```

---

## 第三部分：雙端實作影子報告 (Shadow Reports)

針對這四個共同方法在當前 Rust 與 Go 實作現況中的具體暴露點，我們進行個別分開的影子安全評估與補丁對策：

### 3.1 Rust 實作端影子報告 (`saki-ssh-daemon` Shadow Report)

#### 【Rust 暴露點分析】
* **CM-01 暴露**：現有的 `branch_mgr.rs` 使用標準 `std::fs` 的字串路徑操作，這在 Host OS 核心層面並非原子操作，極易在 `std::fs::metadata` 與 `File::create` 之間被 `symlink` TOCTOU 擊穿。
* **CM-02 暴露**：`tarpit.rs` 在異步 Task 中每次都分配一個新的 `Vec<u8>` 來發送 40MB 的垃圾資料，這會導致高頻發送時內存碎片化（Memory Fragmentation）與瞬間 OOM。
* **CM-03 暴露**：`audit.rs` 產生的 `audit_key.pem` 保存在 Daemon 相同的家目錄下，當惡意 Agent 透過 TOCTOU 突破沙盒後，可輕易讀取該 PEM 並重簽日誌。

#### 【Rust 影子修補對策】
1. **沙盒 FD 原子化**：在 `branch_mgr.rs` 中廢除任何 `std::fs` 字串路徑，改用基於相對 FD 的 `SafeSandbox::open_file` 補丁，強制攜帶 `O_NOFOLLOW`。
2. **Tarpit 零分配化**：重構 `tarpit.rs`，採用 `lazy_static` 靜態生命週期的 64KB 高熵數據塊引用，徹底免除 `entropy_ref.to_vec()` 的內存配置。
3. **Channel Binding 鎖定**：使用 `tokio-rustls` 的 `connection.peer_certificates()`，提取 TLS 憑證特徵，與 ChaCha20 challenge 進行混合 HMAC 計算。

---

### 3.2 Go 實作端影子報告 (`go-sakissh` Shadow Report)

#### 【Go 暴露點分析】
* **CM-01 & CM-04 暴露**：Go 端的 `execute.go` 依賴標準庫 `os/exec` 的路徑字串解析，缺乏核心層的 `O_NOFOLLOW` 保護，在並行高頻 I/O 時完全暴露於 TOCTOU 下。
* **CM-02 暴露**：`codec.go` 執行 `DecodePayload` 時，使用 `zstd.NewReader` 進行整包解壓，缺乏對解壓明文累計大小的流式狀態控制，一旦遭遇 10KB 壓出 10GB 的壓縮炸彈，Go 運行時會因為 OOM 被 OS 強制殺除。
* **CM-03 暴露**：Go 端的 `audit.go` 缺乏密鑰持久化機制，且沒有外部 Aggregator 的單向公鑰推播接口。

#### 【Go 影子修補對策】
1. **流式解壓限額狀態機**：重構 `codec.go`，在 Zstd 解壓流中使用 `io.LimitReader` 強制限制最大明文字節為 5MB，超出即刻中斷 gRPC 連線並將該 IP 列入黑名單。
2. **原子性 `syscall.Openat`**：重構 `execute.go` 的檔案操作，使用 `GoSafeSandbox` 的相對路徑原子調用，強制使用 `syscall.O_NOFOLLOW` 標記。
3. **連線門控 (Connection Gate)**：在 Go 服務端引入並行 Tarpit 限制器，最大並行 Tarpit 連線限制為 32 個，超出即刻物理斷開連線，防止自噬 DoS。

---

## 結論與協定演進啟示

本影子報告通過對 **5個威脅與4個時間階段** 的共同方法進行深度提煉，成功將 SASS 協定的安全邊界從脆弱的本機環境假設中完全解耦。

這四個共同方法（CM-01 至 CM-04）的確立，為 SASS v7.0 協定提供了無可辯駁的理論指導：**協定不應試圖在 Userspace 中模擬核心級的安全（這必定會因為 CM-01 環境可信度錯覺而失敗），而是要透過 FD 原子化、零分配防禦、TLS 凭證通道綁定與外部共識錨定，實現與 Host OS 環境的徹底解耦。**

本報告已完成 Scientia 知識庫歸檔，正式為次世代 SASS 協定的安全架構奠定了堅不可摧的理論與實踐基石。
