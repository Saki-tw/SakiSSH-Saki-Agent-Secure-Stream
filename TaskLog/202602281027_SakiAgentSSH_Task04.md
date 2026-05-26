# SakiAgentSSH Release 與上架研究

## Phase 1：混淆版二進位與專案圖示 (Release 實作)
- [x] macOS ARM64 sakisshd/sakissh strip+LTO Release 編譯
- [x] Windows x86_64 sakisshd.exe/sakissh.exe strip+LTO Release 編譯
- [x] CopyRight 下 daemon/client icon 已確認
- [x] 四個二進位已更新至 release/ 目錄
- [ ] GitHub Release 建立與上傳

## Phase 2：SakiAgentSSH 存取權限架構 (安全研究)
- [x] OpenSSH 權限模型分析
- [x] Windows install.ps1 -CreateUser 權限研究
- [x] macOS LaunchDaemon 權限研究
- [x] ED25519 金鑰交換整合研究
- [x] 安全指南撰寫 → Scientia ✅

## Phase 3：Brew 與 Winget 上架研究
- [x] Homebrew Formula/Tap 規格研究
- [x] Winget manifest 格式研究
- [x] 單平台下載目錄結構規劃
- [x] 自動化升級策略
- [x] 跨平台上架 SOP → Scientia ✅
