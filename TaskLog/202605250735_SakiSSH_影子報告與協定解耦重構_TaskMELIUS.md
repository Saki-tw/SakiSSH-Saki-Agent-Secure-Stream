# SASS (Saki Agent Secure Stream) 影子報告與協定解耦重構任務佇列 (TaskMELIUS)

> 建立時間：2026-05-25 07:35 (UTC+8 Asia/Taipei)  
> 專案簡稱：SakiSSH  
> 類別：TaskMELIUS 任務清單歸檔  
> 版本：v1.0 (非同步協作動態展開)  

---

## 🎯 任務狀態總覽 (Pueue Queue)

本 TaskMELIUS 作為 SASS v7.0 協定重構與雙端影子報告（Shadow Report）的動態執行佇列，嚴格遍循「不中斷旅程」之連續執行鐵律。

```
[ ] Phase 1: 方法學依賴性解構與解耦點確立 ── 分析 TOCTOU、Tarpit、Audit 等底層方法學依賴。
[ ] Phase 2: 設計五大協定層改進規格 (Protocol-level Spec) ── 定義抽象對策。
[ ] Phase 3: Rust 實作端影子報告 (Rust Shadow Report) ── `saki-ssh-daemon` 針對性修補。
[ ] Phase 4: Go 實作端影子報告 (Go Shadow Report) ── `go-sakissh` 針對性修補。
[ ] Phase 5: 影子報告合龍與最終歸檔 ── 寫入 Scientia 知識庫並完成任務交接。
```

---

## 📝 1:5 細化實作步驟

### Phase 1: 方法學依賴性解構與解耦點確立
1. 分析 **TOCTOU 擊穿** 的方法學依賴：解構 Userspace 系統調用「非原子性（Non-Atomicity）」與「軟連結解析路徑不確定性」。
2. 分析 **Tarpit 自噬 DoS** 的方法學依賴：解構「動態內存配置（Dynamic Allocation）」與「未授權資源預先分配（Unauthenticated Resource Pre-allocation）」。
3. 分析 **日誌鏈毒化** 的方法學依責：解構「私鑰同權限本地可讀性（Key Locality）」與「單點日誌自我驗證（Self-Validation）」。
4. 分析 **Zstd 壓縮炸彈** 的方法學依賴：解構「非流式記憶體分配（Non-Streaming Buffer Allocation）」與「解壓縮過程無狀態上限控制」。
5. 分析 **ChaCha20 挑戰繞過** 的方法學依賴：解構「加密挑戰與傳輸通道狀態脫鉤（Channel Decoupling）」與「nonce 偽隨機性（PRNG Predictability）」。
6. **【動態展開 - 轉折點】**：確立這 5 個「依附之解耦點」，作為下一階段協定層改進的輸入，立即開始執行 Phase 2-1。

### Phase 2: 設計五大協定層改進規格
1. 設計 **原子性儲存提供者 API**：定義如何使用 fd 級操作消除 Path-Check 時間窗。
2. 設計 **靜態 Tarpit 限流與分配規則**：設計全域靜態垃圾 Ring Buffer 傳輸機制。
3. 設計 **外部一向公鑰推播與時間戳錨定** (RFC 3161) 協定接口。
4. 設計 **流式 Zstd 解壓防衛閘限額**（Max Decompressed Size Limit）協定規格。
5. 設計 **TLS Channel Binding (tls-unique)** 與 ChaCha20 Challenge HMAC 簽章規格。
6. **【動態展開 - 轉折點】**：完成協定層抽象規格設計，直接切入 Phase 3-1 進行 Rust 實作影子報告。

### Phase 3: Rust 實作端影子報告 (Rust Shadow Report)
1. 針對 `saki-ssh-daemon` 現有代碼（如 `branch_mgr.rs`, `tarpit.rs`, `challenge_store.rs`），指出當前的技術缺陷與漏洞隱患。
2. 提供 Rust 代碼級別的 `StorageProvider` 封裝，實作基於 `openat` 與 `O_NOFOLLOW` 的安全 sandbox path 讀寫。
3. 提供 Rust 代碼級別的 **Zero-Allocation Tarpit** 異步流傳輸實作，使用全域靜態高熵緩衝區。
4. 提供 Rust 代碼級別的 **TLS Channel Binding** 與 HMAC Challenge 驗證代碼。
5. 指出 Rust 端引入這些防禦後的性能影響與 OS 特化限制。
6. **【動態展開 - 轉折點】**：Rust 影子報告完成，立即開始 Phase 4-1 進行 Go 實作影子報告。

### Phase 4: Go 實作端影子報告 (Go Shadow Report)
1. 針對 `go-sakissh` 現有代碼（如 `execute.go`, `tarpit.go`, `codec.go`），指出 Go 端在 Zstd 解壓、路徑檢查與 Tarpit 上的安全落後。
2. 提供 Go 代碼級別的流式 Zstd 限額解壓縮 `SafeDecodePayload` 實作。
3. 提供 Go 代碼級別的 Tarpit 並行連線池與靜態緩衝區分配。
4. 提供 Go 代碼級別的相對路徑 `syscall.Openat` 與 `O_NOFOLLOW` 封裝。
5. 分析 Go 運行時（Goroutines）在面對高頻 Tarpit 阻斷時的調度與資源分配特徵。
6. **【動態展開 - 轉折點】**：Go 影子報告完成，立即開始 Phase 5-1 進行最終合龍歸檔。

### Phase 5: 影子報告合龍與最終歸檔
1. 收集 Phase 1~4 的所有產出，彙整為 `/Users/hc1034/Saki_Studio/Claude/SakiAgentSSH/Scientia/202605250900_SakiSSH_影子報告與協定解耦重構分析_Scientia.md`。
2. 更新介面端的 `task.md` 與 `walkthrough.md`。
3. 撰寫最終交接報告，引導人類用戶查閱，並依照「連續執行鐵律」與「禁止請示中斷條款」完美結束。
