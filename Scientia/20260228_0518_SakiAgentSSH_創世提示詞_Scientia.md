# SakiAgentSSH Release與上架研究 創世提示詞

> **生成時間**：20260228_0518 (UTC+8)
> **前 Session ID**：afd8dadd-7c9d-4ecb-9940-6e3de374f15f
> **生成原因**：使用者指令 (動作太慢了。即刻執行協議九)

## 任務目標 (Mission Statement)
完成 SakiAgentSSH 從編譯到正式釋出的最後一哩路，包含將未完成的 Windows daemon LTO 混淆編譯與 Icon 放回 Release，執行 openSSH 等級權限研究，以及 Winget 與 Brew 上架研究，單平台發佈檔案分離結構。

## 專案位置與關鍵文檔 (Context Map)
| 項目 | 要求 |
|------|------|
| 專案根目錄 | `/Users/hc1034/Saki_Studio/Claude/SakiAgentSSH` |
| 必讀文件清單 | 1. `release/` 目錄內容<br>2. `/Users/hc1034/Saki_Studio/Claude/SakiAgentSSH/TaskLog/20260228_0515_SakiAgentSSH_Release與上架_TaskMELIUS.md` |

## 已完成事項 (Completed Work)
| 項目 | 狀態 | 證據 | 驗證指令 |
|------|:---:|------|---------|
| 基礎 gRPC 整合與本機跨機測試 | ✅ | `saki-ssh-daemon/src/main.rs` | `cargo run` 或本機連線測試 |
| 專案全面更名與重構 | ✅ | `SakiAgentSSH` 目錄 | `ls /Users/hc1034/Saki_Studio/Claude/SakiAgentSSH` |
| GitHub 推送零內部文件清理 | ✅ | `.gitignore` | `git status` |
| Daemon / Client Icon 設計 | ✅ | `/Users/hc1034/Saki_Studio/Claude/CopyRight/*.png` | |

## 未完成事項 (Remaining Work)
### Phase 1：混淆版二進位與專案圖示 (Release 實作)
- [ ] 🔴 等待 Windows x86_64 `sakisshd.exe` 含 winres `.ico` 與混淆配置編譯完成（LTO 極耗時）
- [ ] 🟢 確認 CopyRight 目錄下之 `SakiAgentSSH_daemon_icon.png` 存放正確與轉為 Icon
- [ ] 🟢 將所有編譯完成的二進位檔複製並隔離至乾淨的 `release/` 目錄，引導上傳 GitHub Release

### Phase 2：SakiAgentSSH 存取權限架構 (安全研究)
- [ ] 🟡 分析 OpenSSH 既有權限模型（User/Group、chroot、key-based auth）
- [ ] 🟡 研究 Windows 上透過 `install.ps1 -CreateUser` 建立 `saki` 帳號的權限邊界
- [ ] 🟡 研究 macOS 上做為 LaunchDaemon 與特定 User 執行時的操作權限差異
- [ ] 🟡 探討 ED25519 金鑰交換機制的 Agent 整合方式
- [ ] 🟢 制定 SakiAgentSSH 最佳實踐安全指南（寫入 Scientia）

### Phase 3：Brew 與 Winget 跨平台上架 (發布研究)
- [ ] 🟡 研究 Homebrew Cask/Formula 上架的規格與檔案分離策略
- [ ] 🟡 研究 Winget 提交 `.exe` 與安裝腳本的 manifest 格式
- [ ] 🟡 規劃單一平台下載時的目錄結構與依賴處理方式
- [ ] 🟡 規劃套件管理器中的自動化升級策略與版本號判定基準
- [ ] 🟢 撰寫跨平台上架標準作業流程 SOP（寫入 Protocol/Workflow）

## 技術棧速查 (Tech Stack Quick Ref)
| 組件 | 技術/語言 | 重點細節 |
|------|----------|---------|
| Daemon/Client | Rust (Cargo) | edition = "2021", tonic, prost, tokio |
| 跨平台打包 | PowerShell / Bash | Windows `install.ps1`, macOS `.pkg` 或 brew |
| 圖示工具 | ImageMagick, winres | 用於轉換 PNG 到 ICO 並嵌入 Windows EXE |

## 建議執行順序 (Recommended Execution Order)
1. 先確認 Phase 1 耗時極長的 Windows 交叉編譯是否完成，若完成立刻複製出 binaries 到 release，並讓用戶上傳 GitHub Release。
2. 進入 Phase 2 權限模型研究，提出 SakiAgentSSH 的安全防護最佳實踐並寫入 Scientia。
3. 進入 Phase 3 進行 Homebrew 與 Winget 分析，產生單平台發佈檔案分離結構。

## 踩坑清單與限制事項 (Gotchas & Constraints)
- **Windows LTO 交叉編譯極慢**：請耐心等待或監控 `target/x86_64-pc-windows-gnu/release/sakisshd.exe`，不應隨意中斷。
- **Git Push 零污染原則**：SakiAgentSSH 當前已做到零內部文件污染的 GitHub 同步。任何新編譯產生的東西都不該污染 tracked file，只能使用 `.gitignore` 排除的 release 二進位位置。
- **Winres 嵌入**：Windows 二進位編譯前確保 `icon.ico` 存在並在 `build.rs` 處理。
