# SakiAgentSSH TaskMELIUS (Phase 4: Release & Distribution)

> 任務拆解依據非同步協作架構 Protocol 9，將剩餘之「GitHub Release 建立與上架/提交」任務以 1:5 比例細化。

## 第 1 步：App Store 上架完成
- [ ] 1. 開啟 Xcode，對 SakiAgentSSHDaemon 執行 Product -> Archive (確認 10 條目 icon set 正常無 unassigned child)
- [ ] 2. 對 SakiAgentSSHClient 執行 Product -> Archive
- [ ] 3. 在 Xcode Organizer 中 Validate App 並 Distribute App 至 App Store Connect
- [ ] 4. 登入 App Store Connect 確認 TestFlight/審核狀態
- [ ] 5. 如遇任何憑證或 entitlements 錯誤，參閱 `apple_app_store_submission_guide` KI 並修正

## 第 2 步：GitHub Release 建立
- [ ] 1. 在 `Saki-tw/SakiAgentSSH` 建立 Tag `v0.2.0`
- [ ] 2. 撰寫 Release Note (包含 gRPC/HTTP2 特性、多語系支援等)
- [ ] 3. 附加 `sakisshd` (macOS ARM64), `sakissh` (macOS ARM64), `sakisshd.exe` (Win x64), `sakissh.exe` (Win x64)
- [ ] 4. 附加可能產生的 DMG 打包檔 (若有的話)
- [ ] 5. 發布 Release

## 第 3 步：套件管理員 PR 提交
- [ ] 1. 確保 GitHub Release 已生效，並取得二進位檔的 SHA256 checksum
- [ ] 2. 填寫並提交 Homebrew Cask PR (使用 `release/homebrew-cask/PR_TEMPLATE.md` 內預防被拒絕之 Notability 證據)
- [ ] 3. Fork `microsoft/winget-pkgs`
- [ ] 4. 將 `SakiStudio.SakiAgentSSH.Daemon.yaml` 與 Client 相關 yaml 加入對應目錄
- [ ] 5. 提交 Winget PR 並跟隨自動測試流程
