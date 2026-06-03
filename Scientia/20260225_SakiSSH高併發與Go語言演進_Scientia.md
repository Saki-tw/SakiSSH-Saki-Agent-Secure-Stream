# SakiSSH 架構演進：高併發與多語言考量

## 核心觀點
當前 SakiSSH 基於 Rust (Tonic) 實作，具備優異的記憶體安全與執行效能。對於現階段的分散式編譯任務與 Agent 指令傳遞，Rust 方案已能滿足需求。

## 未來演進：Go 語言的可能性
- **高併發場景**: 若未來 SakiSSH 需處理極大流量的 gRPC 請求或複雜的並行調度，可考慮將 Daemon 改用 **Go (Golang)** 實作。
- **優勢**: Go 的協程 (Goroutine) 模型在處理數萬計的併發連線時具備極高的開發效率與 runtime 低開銷。
- **現狀**: 目前無此急迫需求，優先維持 Rust 工具鏈以利於交叉編譯與單一 binary 分發。

## 跨機自動化 (Axiom 預研)
- **Gemini 調用**: SakiSSH 應作為 `gemini` 指令的透明傳輸層。
- **Nushell 整合**: 所有的 Gemini 指令皆透過 Nushell 進行輸出過濾與結構化處理。
