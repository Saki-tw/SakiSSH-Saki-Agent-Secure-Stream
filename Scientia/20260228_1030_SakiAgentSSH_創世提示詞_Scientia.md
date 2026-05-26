# SakiAgentSSH Release發布與上架 創世提示詞

> **生成時間**：20260228_1033 (UTC+8)
> **前 Session ID**：325224df-6cdc-4e12-b6b4-98e5f15b4cbe
> **生成原因**：用戶指令「即刻執行協議九」/ Step Id > 1000

## 任務目標 (Mission Statement)
完成 SakiAgentSSH 最後一哩路的發布與上架作業：包含建立 GitHub Release、將 Archive 後的 App 送審至 App Store Connect、以及提交套件管理員 (Homebrew / Winget) 的發布 PR。

## 專案位置與關鍵文檔 (Context Map)

| 項目 | 要求 |
|------|------|
| 專案根目錄 | `/Users/hc1034/Saki_Studio/Claude/SakiAgentSSH` |
| 必讀文件清單 | `TaskLog/202602281001_SakiAgentSSH_TaskMELIUS.md` |
| 可選文件 | `release/homebrew-cask/PR_TEMPLATE.md` (Brew 提交證據) |

## 已完成事項 (Completed Work)

| 項目 | 狀態 | 證據 | 驗證指令 |
|------|:---:|------|---------|
| Scoop Repository 建置與推送 | ✅ | `https://github.com/Saki-tw/Scoop-SakiStudio` | `curl -O https://raw.githubusercontent.com/Saki-tw/Scoop-SakiStudio/main/bucket/sakiagentssh-daemon.json` |
| SakiWeb 下載連結更新 | ✅ | `SakiWeb/content/SakiAgentSSH/_index.md` | `cat ../SakiWeb/content/SakiAgentSSH/_index.md` |
| XcodeGen Entitlements 修復 | ✅ | `SakiAgentSSH-Daemon/project.yml` | `grep "app-sandbox" SakiAgentSSH-Daemon/project.yml` |
| AppIcon icns 生成與設定 | ✅ | `SakiAgentSSH-Daemon/Resources/AppIcon.icns` | `file SakiAgentSSH-Daemon/Resources/AppIcon.icns` |
| Asset Catalog 修正 | ✅ | `SakiAgentSSH-Daemon/Assets.xcassets/AppIcon.appiconset/Contents.json` | `cat SakiAgentSSH-Daemon/Assets.xcassets/AppIcon.appiconset/Contents.json` |

## 未完成事項 (Remaining Work)

### Phase 4：Release & Distribution
- [ ] 🟢 **App Store 上架：Xcode Distribute**
  - 使用 Xcode GUI 對 Daemon 和 Client 進行 Product -> Archive。
  - 在 Xcode Organizer 中 Validate App 並 Distribute App 至 App Store Connect。
- [ ] 🟢 **建立 GitHub Release v0.2.0**
  - 在 `Saki-tw/SakiAgentSSH` (GitHub) 建立 Release `v0.2.0`。
  - 填寫 Release Note 附加編譯好之 Daemon/Client macOS & Windows 二進位 (`release/` 下的檔案)。
- [ ] 🟡 **提交 Homebrew Cask PR (待 Release 生效)**
  - 取 GitHub Release URL 二進位的 SHA256。
  - 建立 Homebrew Cask 公式並對 `Homebrew/homebrew-cask` 提 PR，附上 `release/homebrew-cask/PR_TEMPLATE.md` 的 Notability 證據。
- [ ] 🟡 **提交 Winget PR**
  - 對 `microsoft/winget-pkgs` 提 PR 放入 `SakiStudio.SakiAgentSSH.Daemon.yaml` 與 `Client` 的 manifests。

## 技術棧速查 (Tech Stack Quick Ref)
- **核心架構**: xcodeproj 取代直接 SPM package 結構 (macOS)
- **依賴管理**: XcodeGen (`project.yml`)
- **發布工具**: Scoop (自管 bucket), Homebrew (官方 tap), Winget (官方 repo)
- **CI/CD**: Xcode Organizer手動打包 + GitHub Releases 託管二進位

## 建議執行順序 (Recommended Execution Order)
1. **GitHub Release (優先)**：因為後續所有套件管理工具的 url 都必須指向此處。
2. **Xcode App Store 送審**：此部分操作可能需時處理沙盒設定（目前已預先排錯完畢），讓審核機制開始運轉。
3. **套件管理員 PR 建立**：確認 url 和 SHA 生效後提出 PR。

## 踩坑清單與限制事項 (Gotchas & Constraints)
- **XcodeGen 生成空 Entitlements 覆寫**: 曾因使用 `entitlements: path` 導致 xcodegen 每次 generate 將 `.entitlements` 檔案覆寫為空的。必須使用 `properties` 屬性直接指定 `com.apple.security.app-sandbox`。
- **App Store Connect 圖示缺失**: 使用 XcodeGen 時 Asset Catalog 無法直接讓 Xcode 正確將 AppIcon 綁定至主程式，解法為透過 `iconutil` 生成獨立 `.icns`，修改 project.yml `INFOPLIST_KEY_CFBundleIconFile` 為 AppIcon 並將實體 `.icns` 作為 resource 拷貝。
- **macOS Asset Catalog 1024x1024_1x 不合法**: 不能嘗試在 `Contents.json` 新增 `1024x1024` 的 `1x` 條目，這不符 macOS 規範，將導致 Xcode 將整體圖標判定為 Unassigned child。App Store 行銷必須藉由上傳 App Store Connect 網頁端或 Xcode 內正確匹配的最大 `512x512@2x` 自動解決。
