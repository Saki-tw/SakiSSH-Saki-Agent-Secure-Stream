# SASS v1.4 實作計畫 (Implementation Plan)

> 建立時間：2026-05-25 12:41 (UTC+8)
> 專案：SakiAgentSSH
> 目標：SASS v1.4 "幾乎處處優越" 正式落地實作

## 1. 任務背景與方法論概述
依據 Handoff 指示，我們需移除過時的 Protobuf 定義與 gRPC 層的 `ChaChaCognitiveChallenge`，將認知挑戰的層級「下沉」至 L4 (傳輸層) 或 TLS 握手層。同時實作「雙軌制防禦（Vi Swap / Tarpit）」與「XOR 本機溢位防禦」，將破壞成本完全外部化。

---

## 2. 實作架構設計

### 2.1 L4 TLS 握手攔截 (TLS-Exporter / Pre-handshake)
- **現狀**：目前 ChaCha20 挑戰在 `main.rs` 的 `cha_cha_cognitive_challenge` 透過 gRPC (L7) 進行，這意味著攻擊者已完成了 TCP 與 TLS 握手，並消耗了伺服器的 gRPC 解析資源。
- **改進方案**：實作 **L4 Pre-handshake Interceptor**。
  - 在 `run_server` 中，不再直接使用 `Server::builder().serve()`。
  - 我們將手動建立 `tokio::net::TcpListener::accept` loop。
  - 當新的 `TcpStream` 進入時，先不交給 `tonic`，而是進入 `Pre-handshake Challenge` 階段：
    - Daemon 送出一段隨機 Nonce。
    - 若 Client (在超時 500ms 內) 無法以約定的 ChaCha20 key 回覆正確密文，則直接進入 **Zero-Allocation Tarpit**，或者立即 Drop。
    - 若成功，則將該 Stream 包裝後傳遞給 `tokio_rustls::TlsAcceptor`，完成 TLS 握手後再交給 `tonic` router。
- **效益**：無算力的惡意探測在 L4 就被攔截，完全無法觸發 TLS 握手的非對稱運算負載。

### 2.2 雙軌制防禦 (Dual Standard)
- **目標**：針對不同威脅來源給予不同的反擊。
- **Zero-Allocation Tarpit (對外)**：
  - 觸發條件：L4 挑戰失敗、未認證的 TLS 連線、惡意端口掃描。
  - 實作：建立一個 `static ZERO_BUF: [u8; 65536] = [0; 65536];`。當觸發時，啟動一個 tokio task 無窮盡地向該 socket 寫入此 buf，不配置任何新記憶體，直到對方中斷。
- **Vi Swap 停滯機制 (對內)**：
  - 觸發條件：已通過認證的 Agent，但行為越界 (OOB)，例如嘗試訪問非授權路徑。
  - 實作：在 `check_acl` 或 `policy.rs` 攔截後，取代原本回傳 `tonic::Code::PermissionDenied`，我們回傳一段特殊的串流。若是檔案讀取，我們回傳 `\x1b[2J\x1b[H` (清空終端) 以及無意義的 Base64 垃圾，藉此「污染」該 Agent (LLM) 的上下文，使其陷入幻覺停滯。

### 2.3 XOR 本機溢位防禦
- **目標**：保護 LocalHost 資源不被異常提取。
- **實作**：
  - 修改 `localhost_defense.rs`。
  - 當檔案路徑命中 `/etc/passwd`, `/etc/shadow`, `~/.ssh/` 等特權區域時。
  - 檔案讀取時，不回傳原始資料，而是在資料流過時即時執行 XOR (`data[i] ^ 0x5A`)，使攻擊者拿到錯誤資料，進一步引導 LLM 解讀錯誤。

---

## 3. 執行步驟清單
1. [ ] **Phase 1-1**: 刪除 `main.rs` 的 `cha_cha_cognitive_challenge` RPC 邏輯。
2. [ ] **Phase 1-2**: 實作 L4 Listener (取代現有 `serve`)。
3. [ ] **Phase 1-3**: 實作 L4 ChaCha20 Pre-handshake 邏輯。
4. [ ] **Phase 2-1**: 實作 `tarpit.rs` 的 `zero_allocation_tarpit(stream)` 函式。
5. [ ] **Phase 2-2**: 實作 `policy.rs` 的 Vi Swap 攔截。
6. [ ] **Phase 3-1**: 實作 `localhost_defense.rs` 的 XOR stream 變換。
7. [ ] **Phase 3-2**: 整合所有防禦機制並測試編譯。

---

## 4. 預期效益與驗證
- **驗證方式**：
  1. 使用 `nc` 測試 Daemon port，應立刻斷線或卡在 Tarpit，不再直接收到 TLS Alert。
  2. 模擬 Agent 越界存取 `/etc/passwd`，預期會收到 XOR 混淆或 Vi Swap 清除碼，而非標準錯誤。
