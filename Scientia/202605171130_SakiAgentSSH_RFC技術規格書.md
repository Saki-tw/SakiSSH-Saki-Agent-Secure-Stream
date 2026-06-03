# SakiAgentSSH v5.0 (RFC) 技術實作規格書 (Scientia)
> 建立時間：2026-05-17T11:30:00+08:00 (最後修訂：2026-05-22)
> 作者：Antigravity (SakiDeusExAgent) / 共同作者：小Saki本人
> 地點：SakiStudio Dev M1上的CodEditor / Session 6cfbbcda-37b7-4e8c-a464-22b4de7e20b9
> Update/ChangeLog：v5.0 升級 - 定義 TLS/mTLS 傳輸層、ALPN 握手協定，並正名 SakiAgentSSH 為「Saki Agent Secure Stream (SASS)」以符合 IETF HTTP/2 Overlay 規範。

**標籤**: #SakiAgentSSH #RFC #TechnicalSpec #ProjectSpeculari #ZeroTrust
**狀態**: 核心定義完成，待 Driver 層擴充
**日期**: 2026-05-17

本文件作為 SakiAgentSSH 協議的 RFC 技術實作細則，旨在消弭高階概念中的抽象部分，為開發與後續維護提供精確的程式碼層級（Code-Level）指導與系統架構藍圖。

---

## 0. 協議命名與 IANA 註冊聲明 (v5.0 新增)

為避免與 IETF RFC 4251 (The Secure Shell Protocol Architecture) 產生命名衝突與技術混淆，本協議（內部代號 `SakiAgentSSH`）在正式提交 RFC 草案時，將正名為 **Saki Agent Secure Stream (SASS)** 或 **SakiRPC**。
- **架構定位**：本協議**不使用** RFC 4253 定義的 SSH Transport，而是建立於 **HTTP/2 over TLS 1.3** 之上的應用層 (OSI Layer 7) 覆蓋網路協議。
- **ALPN 協商**：在 TLS/mTLS 握手階段，協議將使用專屬的 ALPN (Application-Layer Protocol Negotiation) 標籤 `x-sakirpc-v5` 或 `h2`，確保網路設備與代理伺服器能正確識別此流量並非傳統 SSH (Port 22)。
- **MIME Type**：gRPC Payload 將註冊使用 `application/grpc+saki` 命名空間。

---

## 1. 網路傳輸層：gRPC 封裝與 CJK 亂碼防禦

### 1.1 執行載荷 (Payload) 的編碼實作
為了徹底解決 POSIX PTY 與 `sh -c` 在 Windows (CP950) 與 macOS (UTF-8) 之間的 CJK 字串轉譯損毀問題，SakiAgentSSH v4.0 放棄純字串的指令傳遞，改用雙層封裝。

**實作細節 (Codec 模組)**：
- **依賴**: `zstd` (v0.13), `base64` (v0.22)
- **編碼流程 (Client端)**：
  1. 將含有特殊字元或非 ASCII 路徑的字串轉換為 `Vec<u8>`。
  2. 使用 `zstd::stream::Encoder::new(buffer, 3)` 進行壓縮。
  3. 透過 `base64::engine::general_purpose::STANDARD.encode()` 轉為 Base64  ASCII 字串。
  4. 存入 `ExecuteRequest.raw_payload` (Protobuf `bytes` 型態)。
- **解碼流程 (Daemon端)**：
  1. 接收 `raw_payload` 後進行 Base64 decode。
  2. 傳入 `zstd::stream::Decoder` 展開為原始 Byte Stream。
  3. 直接將 Byte 轉換回 `String` 並透過 Tokio `Command::args()` 餵給進程，**全程不經過 shell interpretation**。

### 1.2 檔案直寫機制 (Network-to-OS Write)
取代傳統的 `echo "data" > file`。

**實作細節 (`RawFileTransfer` RPC)**：
- **Protobuf 定義**: `rpc RawFileTransfer(stream RawFileChunk) returns (RawFileTransferResponse);`
- **I/O 處理**: Daemon 收到 `RawFileChunk` 的 `Metadata` 後，直接使用 `tokio::fs::File::create` 打開對應 OS 檔案的 File Descriptor。
- 接收到 `ZstdBase64Data` 塊後，呼叫 `decode_payload` 解出原始二進位，呼叫 `file.write_all(&decoded)`。此舉繞過了所有的 TTY buffer 與字元編碼猜測，確保 100% 位元精確複製。

---

## 2. 威脅防禦：Chacha20-13Policy 與 40MB Tarpit

在進入任何指令解析前，**Daemon 首先會嚴格檢查簽證 (ED25519 Signature)**。只有通過該密碼學認證的連線，才有資格進入 gRPC 下令階段。此外，Daemon 必須在部署時取得**作業系統的網路防火牆級別權限 (Firewall-level Privileges)**，以便針對所有對外連線的可能 Agent 以及直連流量進行全域監聽與攔截。

當已授權但疑似失控的 Agent 嘗試下達諸如 `rm -rf /` 或 `:(){ :|:& };:` 等惡意指令時，將觸發進階防禦機制。

### 2.1 13Policy 偵測與高熵挑戰 (Cognitive Auth)
- **攔截點**: 已經過憑證認證的 `SakiSsh::execute` 與 `execute_stream` RPC 入口處。
- **挑戰生成**: 
  1. `auth::AgentAuthenticator::check_13policy` 發現特徵碼，回傳 `false`。
  2. Daemon 內部調用 `threat_defense::generate_chacha_challenge()`。
  3. 隨機生成 32 bytes 密鑰與 12 bytes Nonce，使用 `ChaCha20Poly1305` 對固定字串 `"SAKI_AGENT_SSH_CHALLENGE"` 進行加密。
  4. 透過 gRPC 的 `Status::with_details` 將錯誤代碼設為 `ERROR_CHACHA_CHALLENGE_FAILED` (80)，並附帶 `chacha_challenge_nonce` 返回給 Agent。

### 2.2 混雜 ICMP 與 Chacha20-Poly1305 的 40MB 柏油坑反擊 (Hybrid Tarpit & ICMP Flood)
有別於單純切斷連線，針對未能解開挑戰或未通過簽證的惡意連線，Daemon 將採取極具侵略性的反制措施，從應用層與網路層雙管齊下。
- **實作細節 (`execute_tarpit_countermeasure`)**:
  - 開啟一個 `tokio::spawn` 獨立任務，傳入觸發者的來源 IP。
  - **應用層 (TCP Tarpit)**: 以極慢的速度分批透過已建置的連線回塞總計 **40MB** 的資料。這 40MB 載荷並非單純亂數，而是全程使用 `Chacha20-Poly1305` 進行加密的高熵資料。目的在於強迫 Rogue Agent 耗費巨大的 CPU 資源嘗試解密或轉譯，並瞬間撐爆其 Context Window 上限。
  - **網路層 (ICMP Flood)**: 在 TCP 塞滿垃圾資料的同時，Daemon 將調用底層權限（結合前述的 Firewall-level Privileges），混雜發送大量且大體積的 ICMP 請求（Ping Flood）至該來源 IP。
  - **效果**: 雙管齊下強制耗盡惡意 Agent 的 Socket 連線池、頻寬與運算資源，徹底阻斷其後續的掃描或橫向移動能力。

---

## 3. 部署模式：Obidan-Style (自體寄生註冊)

SakiAgentSSH Daemon (`sakisshd`) 不需透過安裝檔部署，具備自舉 (Bootstrap) 能力。

### 3.1 Windows Service (SCM 註冊)
- **觸發**: 帶入 `--obidan-install` 參數。
- **實作 (API-level)**: 呼叫作業系統 `sc create SakiAgentSSH binPath= "{current_exe}" start= auto`。
- 註冊成功後，自動調用 `sc start SakiAgentSSH`。程式內整合 `windows-service` crate，`ffi_service_main` 會攔截 SCM (Service Control Manager) 的控制信號並將狀態回報為 `Running`。

### 3.2 macOS LaunchAgent (plist 自動生成)
- **觸發**: 帶入 `--obidan-install` 參數。
- **實作 (File-level)**: 
  1. Daemon 取得 `std::env::current_exe()`。
  2. 動態生成 `tw.saki.sakisshd.plist`，設定 `<key>RunAtLoad</key><true/>` 與 `<key>KeepAlive</key><true/>`。
  3. 寫入至 `~/Library/LaunchAgents/`。
  4. 執行 `std::process::Command::new("launchctl").arg("load")`，立即讓 `launchd` 接管進程生命週期，達到隨插即用的免互動安裝。

---

## 4. 終極隔離：Driver-Level Storage Locking (Phase 6 實作架構)

為彌補 User-space Daemon 無法徹底限制 Agent 的缺陷（例如 Agent 可能嘗試利用本地提權漏洞），SakiAgentSSH 將實施 Driver 級針對儲存區域的鎖定，達成物理級別的沙盒隔離。

### 4.1 macOS 實作：Endpoint Security (DriverKit)
- **元件**: SakiAgentSSH ES Extension (.systemextension)
- **實作方式**: 
  - 利用已取得的 Apple DriverKit (Endpoint Security) 授權，開發核心層擴充。
  - 註冊 `ES_EVENT_TYPE_AUTH_OPEN` 與 `ES_EVENT_TYPE_AUTH_EXEC` 事件監聽。
  - 當 Agent（其 PID 由 SakiAgentSSH Daemon 標記）嘗試讀取非 `CapabilitySet.allowed_paths` 內的檔案時，Kernel 層的 ES 回呼函數會直接攔截並返回 `ES_AUTH_RESULT_DENY`。
  - 由於運作於 Ring 0，此封鎖將強制覆蓋並無視 Agent 內的 `sudo` 甚至 Root 權限。

### 4.2 Windows 實作：File System Minifilter Driver (WDK)
- **元件**: `saki_agent_filter.sys`
- **實作方式**:
  - 利用已取得的 WDK (Windows Driver Kit) 憑證，開發與部署 File System Minifilter Driver。
  - 在 WDK 框架中實作 `PreOperation` callback (`IRP_MJ_CREATE`)。
  - 直接在 IRP (I/O Request Packet) 處理層級攔截，並利用 ALPC 與 User-space 的 `sakisshd` 同步 PID 與允許路徑清單。
  - 若受限 PID 試圖建立 `Handle` (開啟檔案)，Minifilter 物理拒絕該 IRP，返回 `STATUS_ACCESS_DENIED`。

### 4.3 Linux 實作研究方向
- 針對 Linux 系統，後續需研究透過 **eBPF** (Extended Berkeley Packet Filter) 或是註冊 **out-of-tree kernel module** 的授權與發佈方式來達成同等於上述兩者的隔離效果。

---

## 5. 結論與下一步

本階段的 SakiAgentSSH 實作計畫已轉換為嚴謹的 RFC 架構文檔歸檔完畢。後續的 Crate 實作與 Driver 層次的深度開發，將留待具備更完整編譯與驅動簽章環境的對話中接續處理。

目前的任務追蹤器 (`task.md`) 與實作計畫 (`implementation_plan.md`) 皆已收斂完成。
