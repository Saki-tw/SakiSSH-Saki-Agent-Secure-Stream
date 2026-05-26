# SakiAgentSSH macOS App × 2（App Store 上架用）

## 目標
建立兩個最小 macOS SwiftUI App，純粹作為 App Store 上架保底，各自包含對應的 Rust 二進位。

## Proposed Changes

---

### App 1：SakiAgentSSH Daemon

#### [NEW] SakiAgentSSH-Daemon/ (Xcode 專案)
- **Bundle ID**: `tw.saki.SakiAgentSSH-Daemon`
- **Icon**: `SakiAgentSSH_daemon_icon.png`（640→1024 upscale）
- **SwiftUI**：單視窗，頂部 About 文字欄位
- **系統「關於」**：顯示 Saki Studio 版權
- **內嵌**: `release/daemon/sakisshd-darwin-arm64` → `Contents/MacOS/sakisshd`
- **Entitlements**: App Sandbox + Network Client

---

### App 2：SakiAgentSSH Client

#### [NEW] SakiAgentSSH-Client/ (Xcode 專案)
- **Bundle ID**: `tw.saki.SakiAgentSSH-Client`
- **Icon**: `SakiAgentSSH_client_icon.png`（640→1024 upscale）
- **SwiftUI**：單視窗，頂部 About 文字欄位
- **系統「關於」**：顯示 Saki Studio 版權
- **內嵌**: `release/client/sakissh-darwin-arm64` → `Contents/MacOS/sakissh`
- **Entitlements**: App Sandbox + Network Client

---

### 共用結構（每個 App）
```
SakiAgentSSH-{Daemon|Client}/
├── Package.swift
├── Sources/
│   └── SakiAgentSSH{Daemon|Client}App.swift  # @main + AboutView
├── Resources/
│   ├── AppIcon.icns             # 從 CopyRight PNG 轉換
│   └── Credits.rtf              # 「關於」視窗的 credits
├── SakiAgentSSH{Daemon|Client}.entitlements
└── Info.plist (embedded via Package.swift)
```

### 關鍵技術細節
| 項目 | 值 |
|------|-----|
| 最低部署版本 | macOS 13.0 |
| 簽名 | Apple Distribution: Chang Hua (36HPTNN8NU) |
| Team ID | 36HPTNN8NU |
| Copyright | © 2026 Saki Studio. All rights reserved. |
| Icon 原始尺寸 | 640×640 → sips upscale 至 1024×1024 |

## Verification Plan
- `xcodebuild archive` 成功
- `codesign --verify --deep` 通過
- 系統「關於」選單正確顯示
