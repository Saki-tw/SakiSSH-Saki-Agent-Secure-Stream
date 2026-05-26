# SakiAgentSSH Release 與 macOS App — Walkthrough

## macOS App Store 上架

兩個獨立 App — **ARCHIVE SUCCEEDED** + **codesign verified** ✅

| App | Bundle ID | 狀態 |
|-----|-----------|------|
| **Daemon** | `tw.com.saki-studio.SakiAgentSSH-Daemon-GUIapp` | ✅ Archive + codesign |
| **Client** | `tw.com.saki-studio.SakiAgentSSH-Client-GUIApp` | ✅ Archive + codesign |

### 內含功能
- **Icon**: Assets.xcassets 完整 1x+2x 配對（16~1024px）
- **字型**: GenJyuuGothicX-Regular.ttf 綁定所有 UI 文字
- **色彩**: 馬卡龍紫 `#DA70D6` / 勿忘草青 `#00CED1` 漸層主題
- **版權頁**: © 2026 Saki Studio + links
- **Help 選單**: Cmd+? 連結 GitHub README
- **Win64 下載**: GitHub Release 連結
- **Entitlements**: Sandbox + Network Client（+ Server for Daemon）
- **ITSAppUsesNonExemptEncryption**: false

### Archives 位置
```
~/Library/Developer/Xcode/Archives/2026-02-28/
├── SakiAgentSSHDaemon.xcarchive
└── SakiAgentSSHClient.xcarchive
```

> Xcode → Window → Organizer → Distribute App

### 待辦
- [ ] 確認 GitHub Release Win64 路徑
- [ ] i18n 說明書（含 Help Book 整合）
