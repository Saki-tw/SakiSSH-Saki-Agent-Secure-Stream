# TaskMELIUS: SakiSSH & SakiClip 部署驗證與收益化衝刺

> **時間戳**：20260226_0110
> **專案**：SakiSSH, SakiClip, SakiMeasureMAP

## 1. SakiClip 連線奪還 (Loser -> M1)
- [x] 1.1 確認 M1 Hub 監聽介面與防火牆狀態 (Port 19283)。
- [x] 1.2 讀取 Loser v3.1 日誌判定連線失敗根因。
- [x] 1.3 修正 Hub 繫結邏輯 (若需 0.0.0.0)。
- [x] 1.4 遠端推送修復版源碼並執行 dotnet publish。
- [ ] 1.5 驗證 GUI 燈號與連線穩定度。

## 2. SakiSSH 分散式指揮部驗證
- [ ] 2.1 驗證 sakisshd.exe 與 Nushell 的對接完整性。
- [ ] 2.2 測試 M1 -> Loser 的遠端 gRPC 指令執行。
- [ ] 2.3 撰寫 Axiom 分散式執行 Scientia 報告。
- [ ] 2.4 稽核遠端環境防禦設定 (UAC/Firewall)。
- [ ] 2.5 建立效能監控基線。

## 3. SakiMeasureMAP 收益化衝刺 (Pro Features)
- [x] 3.1 定位 iOS 端功能鎖 (Feature Gate) 插入點。
- [x] 3.2 鑲嵌 「PRO」 標示。
- [x] 3.3 實作 IsProEnabled() 檢查邏輯與 StoreKit 預留。
- [ ] 3.4 執行 ios_release 建置腳本並生成 RC 版。
- [x] 3.5 最終稽核 4000 字 App Store Metadata。
