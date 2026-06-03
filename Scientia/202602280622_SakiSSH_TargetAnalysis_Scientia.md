# SakiAgentSSH 內部歷史考古與架構現況分析

> 建立時間：2026-02-28 06:22 (UTC+8)
> 執行協議：20260128_2245_內部歷史考古協議 / 20260128_2230_架構現況報告生成協議

## 1. 階段零：規模與模式判定
- **統計指令**：`find . -name "*.rs" -o -name "*.proto" | xargs wc -l`
- **規模判定**：總計約 1,860 行。屬於 **微型 (Micro)** 專案。
- **掃描策略**：全量讀取架構概述與關鍵檔案。

## 2. 階段一：事實盤點 (Fact Inventory)
- **核心架構**：Client-Daemon 模式，以 gRPC / HTTP2 取代傳統 SSH TTY 綁定。
- **功能模組**：
  - `saki-ssh-daemon/src/main.rs`: 負責指令串流 (ExecuteStream)、生命週期追蹤 (TrackedProcess)、CIDR IP 白名單檢查 (`check_acl`)。
  - `proto/sakissh.proto`: 定義 RPC，包含 `FileUpload`、`FileDownload` 及其使用的 `FileChunk` 串流機制，並支援 `offset` 斷點續傳。
- **文檔時間軸**：已具備 `ARCHITECTURE.md`，前版已更新至 v0.2（引入 ACL 與檔案傳輸）。

## 3. 階段二：差距分析 (Gap Analysis)
1. **代碼與文檔一致性**：`ARCHITECTURE.md` 描述的 `FileChunk` 與 ACL 機制皆在代碼中得到確認，無實作落差。
2. **遺留雜訊**：根目錄存在一個空目錄 `SakiSSH`，推測為開發過程中的重構殘留。
3. **文案缺失**：`docs/pages/` 內的 HTML 內容缺乏 Saki Studio 規定的三語系 Persona 風格與量化效益（如具體的效能提升數據）。

## 4. 階段三：報告生成計畫
- 確認事實無誤，將根據此分析結果，直接進入 `docs/pages/` 文案的替寫任務，不中斷自動化執行流程。