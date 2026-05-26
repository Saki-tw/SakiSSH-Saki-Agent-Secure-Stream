# SakiAgentSSH v0.2.0 發布補完與上架研究 - TaskMELIUS

## Melius 核心目標
完成 SakiAgentSSH 從編譯到正式釋出的最後一哩路，包含權限架構研究、混淆二進制發布與跨平台套件管理員（Winget/Brew）上架準備。針對剩餘未完成項目進行 1:5 深度拆解。

---

## 任務清單 (1:5 深化拆解)

### 1. 混淆版二進位與專案圖示 (Release 實作)
- [x] 確認 macOS ARM64 `sakisshd` 與 `sakissh` 含 strip/LTO/opt-level=z 配置之 Release 編譯成功
- [x] 確認 Windows x86_64 `sakisshd.exe` 與 `sakissh.exe` 含 strip/LTO/opt-level=z 配置之 Release 編譯成功
- [x] 確認 CopyRight 目錄下之 `SakiAgentSSH_daemon_icon.png` 與 `SakiAgentSSH_client_icon.png` 意象分離與存放正確
- [x] 將四個編譯完成之二進位檔複製並隔離至乾淨的 `release/` 目錄
- [ ] 確認 GitHub 倉庫 `SakiAgentSSH` 對應，建立 GitHub Release 並上傳二進位檔與安裝腳本

### 2. SakiAgentSSH 存取權限架構 (安全研究)
- [x] 分析 OpenSSH 既有權限模型（User/Group、chroot、key-based auth）
- [x] 研究 SakiAgentSSH Daemon 在 Windows 上透過 `install.ps1 -CreateUser` 建立 `saki` 專屬帳號的實務權限邊界
- [x] 研究 SakiAgentSSH Daemon 在 macOS 上做為 LaunchDaemon 與特定 User 執行時的操作權限差異
- [x] 探討未來引進 ED25519 金鑰交換機制的架構可行性與 Agent 端整合方式
- [x] 制定 SakiAgentSSH 最佳實踐安全指南（寫入 Scientia）

### 3. Brew 與 Winget 跨平台上架 (發布研究)
- [x] 研究 Homebrew Cask（或 Formula）上架 SakiAgentSSH client 與 daemon 的規格與檔案分離策略
- [x] 研究 Windows Package Manager (Winget) 提交 SakiAgentSSH `.exe` 與安裝腳本的 manifest 格式
- [x] 規劃單一平台（如僅下載 Windows 版）時的目錄結構與依賴處理方式
- [x] 規劃 SakiAgentSSH 在套件管理器中的自動化升級策略與版本號判定基準
- [x] 撰寫跨平台上架標準作業流程 SOP（寫入 Protocol/Workflow）
