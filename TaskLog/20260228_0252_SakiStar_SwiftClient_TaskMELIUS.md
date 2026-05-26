# SakiClip Swift Client — TaskMELIUS

> 生成時間：20260228 0252 (UTC+8)
> 專案簡稱：SakiStar
> 原始任務：實作 SakiClip Client macOS arm64 版本

## 原始任務拆解

### 1. 分析 Hub 端 Swift 可複用架構
1. 讀取 SakiClipApp.swift 的 SakiProto 協議實作
2. 確認 XOR scrambling / zstd / hash 的 Swift 原生方法
3. 確認 Hub 端 TCP 連線機制（NWConnection vs Socket）
4. 提取可直接複用的 SakiProto/SakiColors 類別
5. 確認 macOS 剪貼簿 API（NSPasteboard）與 Client 需求差異

### 2. 建立 Swift Client 專案結構
1. 確認目標路徑 SakiClip/ClientMac/
2. 建立 Swift 單檔 CLI+GUI 架構（SwiftUI / AppKit）
3. 決策：SwiftUI vs AppKit（考慮 NSPasteboard、系統匣、視窗佈局）
4. 建立 Package.swift 或直接 swiftc 編譯
5. 確認 macOS sandbox 權限（Network + Clipboard）

### 3. 實作核心網路層
1. 實作 TCP Client 用 NWConnection（從 Hub 端提取）
2. 實作 ConnLoop 自動重連
3. 實作 ReadLoop 接收與解析 SakiProto
4. 實作 CLIP/FILE/PING 訊息處理
5. 驗證與 Hub 端的雙向通訊

### 4. 實作 UI 層
1. 實作主視窗（發送區/接收區/檔案區/日誌區）
2. 實作 Saki Studio 色彩主題
3. 實作 i18n 三語系（zh-TW/ja-JP/en-US）
4. 實作系統匣（NSStatusItem）
5. 實作設定頁面 + 版權頁

### 5. 實作剪貼簿監控
1. 實作 NSPasteboard polling（macOS 無事件通知）
2. 實作剪貼簿變更偵測與自動發送
3. 實作接收時寫入剪貼簿
4. 實作檔案拖放接收
5. 驗證完整剪貼簿雙向同步

### 6. Config + 部署
1. 實作 settings.json 讀寫（~/Library/Application Support/SakiClipClient/）
2. 實作首次語言偵測
3. 編譯為 arm64 binary
4. 測試與 Hub 連線
5. 確認 Receive 路徑設為 ~/Receive
