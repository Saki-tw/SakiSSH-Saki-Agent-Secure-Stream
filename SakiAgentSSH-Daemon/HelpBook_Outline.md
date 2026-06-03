# SakiAgentSSH Help Book 大綱

> **狀態**：大綱，待後續完整實作
> **格式**：Apple Help Book（`.help` bundle，HTML-based）
> **語系**：繁體中文 / English / 日本語

---

## Help Book Bundle 結構

```
SakiAgentSSHHelp.help/
├── Contents/
│   ├── Info.plist
│   └── Resources/
│       ├── en.lproj/
│       │   ├── index.html          # EN main page
│       │   └── pages/
│       │       ├── installation.html
│       │       ├── usage.html
│       │       └── troubleshooting.html
│       ├── zh-Hant.lproj/
│       │   ├── index.html          # 繁中主頁
│       │   └── pages/
│       │       ├── installation.html
│       │       ├── usage.html
│       │       └── troubleshooting.html
│       ├── ja.lproj/
│       │   ├── index.html          # 日文主頁
│       │   └── pages/
│       │       ├── installation.html
│       │       ├── usage.html
│       │       └── troubleshooting.html
│       ├── shared/
│       │   ├── style.css           # 全域樣式（馬卡龍紫 #DA70D6 / 勿忘草青 #00CED1）
│       │   ├── GenJyuuGothicX-Regular.ttf
│       │   └── images/
│       │       ├── daemon_icon.png
│       │       └── client_icon.png
│       └── SakiAgentSSH.helpindex  # Apple Help 索引
```

---

## Info.plist 必要 Key

```xml
<key>CFBundleIdentifier</key>
<string>tw.com.saki-studio.SakiAgentSSH.help</string>
<key>CFBundleName</key>
<string>SakiAgentSSH Help</string>
<key>CFBundlePackageType</key>
<string>BNDL</string>
<key>HPDBookTitle</key>
<string>SakiAgentSSH Help</string>
<key>HPDBookAccessPath</key>
<string>index.html</string>
```

App 側 Info.plist 需加入：
```xml
<key>CFBundleHelpBookFolder</key>
<string>SakiAgentSSHHelp.help</string>
<key>CFBundleHelpBookName</key>
<string>tw.com.saki-studio.SakiAgentSSH.help</string>
```

---

## 頁面內容大綱

### 1. index.html — 首頁
- App icon + 名稱（Daemon / Client）
- 版本號
- 快速導航：安裝 | 使用方式 | 故障排除
- © 2026 Saki Studio

### 2. installation.html — 安裝指南
- **macOS**：App Store 安裝或直接下載
- **Windows**：
  - Daemon: `https://github.com/saki-tw/SakiAgentSSH/releases/download/v0.2.0/sakisshd.exe`
  - Client: `https://github.com/saki-tw/SakiAgentSSH/releases/download/v0.2.0/sakissh.exe`
- 系統需求（macOS 13+, Windows 10+ x64）
- 安全設定（CIDR 白名單）

### 3. usage.html — 使用方式
- Daemon 啟動流程
  - macOS: App Store 版直接啟動
  - Windows: `sakisshd.exe --config config.toml`
- Client 連線
  - macOS: App Store 版啟動
  - Windows: `sakissh.exe <host>:<port> <command>`
- gRPC 端口設定
- 指令代理範例

### 4. troubleshooting.html — 故障排除
- 連線失敗 → 檢查 CIDR 白名單
- 防火牆設定（macOS / Windows）
- gRPC port 衝突

---

## CSS 色彩哲學

```css
:root {
    --saki-purple: #DA70D6;      /* 馬卡龍紫 */
    --saki-blue: #00CED1;        /* 勿忘草青 */
    --bg-gradient: linear-gradient(135deg, rgba(218,112,214,0.05), rgba(0,206,209,0.05));
    --font-family: 'GenJyuuGothicX-Regular', -apple-system, sans-serif;
}
```

---

## 建置指令

```bash
# 生成 .helpindex（Apple Help Indexer）
hiutil -C -a -f SakiAgentSSHHelp.help/Contents/Resources/SakiAgentSSH.helpindex \
    SakiAgentSSHHelp.help/Contents/Resources/en.lproj/

# 複製到 app bundle
cp -R SakiAgentSSHHelp.help \
    SakiAgentSSHDaemon.app/Contents/Resources/
```
