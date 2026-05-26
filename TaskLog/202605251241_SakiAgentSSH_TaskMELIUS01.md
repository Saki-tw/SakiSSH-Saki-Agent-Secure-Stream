# SakiAgentSSH TaskMELIUS 01: SASS v1.4 實作展開

> 建立時間：2026-05-25 12:41 (UTC+8)
> 專案：SakiAgentSSH
> 來源：Session fc8248cb Handoff 指示

## Phase 1: saki-ssh-daemon 實作盤點與 L4 TLS 握手攔截 (目前階段)

- [x] 1-1. **現況盤點**：確認 `saki-ssh-daemon` src 內容，盤點舊版 `ChaChaCognitiveChallenge` 相關 gRPC 邏輯所在位置。
- [x] 1-2. **架構奠基 (逆向展開)**：研究 `tonic`/`rustls` 下的 L4 TLS Handshake 攔截機制（TLS-Exporter），確認如何以 Custom Extension 強制進行 ChaCha20 挑戰。
- [x] 1-3. **方法論產出**：依據 §4.3a 規範，撰寫 SASS v1.4 TLS 攔截與雙軌制防禦之實作計畫 (`ImplementationLog`)，評估可行性。
- [x] 1-4. **程式碼純化與實作**：清除舊有 gRPC 邏輯，並實作 L4 TLS 握手攔截（無算力即 Drop）。
- [x] 1-5. **自動推進**：Phase 1 實作完成後，立即開始 Phase 2 第一步（雙軌制防禦落地研究）。

## Phase 2: 雙軌制防禦落地 (Dual Standard)

- [x] 2-1. **防禦機制研究**：分析 `policy.rs` 或 `session.rs`，設計 OOB (Out-of-Bounds) 越界裁定邏輯。
- [x] 2-2. **Vi Swap 實作**：針對已認證內部 Agent，設計並實作回傳靜態 terminal escape sequence 以卡死 LLM 的機制。
- [x] 2-3. **Tarpit 實作**：針對惡意外部 Agent，實作 Zero-Allocation Tarpit (64KiB 無限切片餿水)。
- [x] 2-4. **整合與測試**：將雙軌制防禦掛載至連線處理流程，並撰寫測試案例。
- [x] 2-5. **自動推進**：Phase 2 實作完成後，立即開始 Phase 3 第一步（XOR 溢位防禦）。

## Phase 3: XOR 本機溢位防禦

- [x] 3-1. **機制評估**：盤點目前 LocalHost 假檔案系統的防禦實作。
- [x] 3-2. **XOR 混淆設計**：研究直接 XOR 混淆回傳的實作方式與效能影響。
- [x] 3-3. **實作升級**：將假檔案系統升級為 XOR 混淆回傳。
- [x] 3-4. **驗證與 Walkthrough**：執行防禦攔截測試，並依據 §4.3a 撰寫最終 Walkthrough 報告。
- [x] 3-5. **任務結案**：評估是否有後續優化需求，若無則標記 SASS v1.4 實作完成。
