# SakiStarCommuncation 匯流排建構 — TaskMELIUS (Session 接續用)

> 建立時間：20260303 2113 (UTC+8)
> 專案簡稱：SakiStar
> 原始任務：機構匯流排完整實作與建構（核心產出：SakiAgentSSH）
> 狀態：Phase 1-4 全部完成，以下為後續可展開任務

---

## ✅ 已完成（本 Session）

### A. SakiAgentSSH 編譯與部署
1. ✅ Mac 本機 daemon/client release 編譯
2. ✅ Windows x86_64 交叉編譯（mingw-w64）→ 16MB PE32+
3. ✅ 本機 gRPC Ping + Execute 驗證
4. ✅ loser 部署 → gRPC Ping (v1.0.0, pwsh 7.5.4) + Execute 成功
5. ✅ trading-v4 部署 → gRPC Ping (v1.0.0, powershell.exe) 成功

### B. saki-orchestrator 整合
1. ✅ 新增 BusInterface struct + sakissh_probe() gRPC 探針
2. ✅ heartbeat 改為 gRPC 優先 SSH 備援
3. ✅ 兩節點（loser + trading）均啟用 gRPC
4. ✅ API /api/nodes 正確回傳匯流排資訊
5. ✅ 重新編譯成功

### C. 基礎設施
1. ✅ 路由修正：192.168.50.0/24 → en5 (USB 2.5G)
2. ✅ bus-diagnosis.sh 建立並驗證
3. ✅ ARCHITECTURE.md 新增 §3.7 匯流排拓撲
4. ✅ service-manifest.yaml 更新 grpc+ssh
5. ✅ 全部歸檔 TaskMELIUS + Scientia + WalkthroughLog

---

## 🔶 後續可展開任務

### D. Windows Service 原生支持
1. 在 saki-ssh-daemon 加入 `windows-service` crate 依賴
2. 實作 ServiceMain + service control handler
3. 條件編譯 `#[cfg(windows)]` 與 `#[cfg(not(windows))]`
4. 重新交叉編譯並部署
5. 驗證 `sc.exe create` + `sc.exe start` 不再報 Error 1053

### E. route 持久化
1. 研究 macOS 路由持久化方案（networksetup / launchd / pf）
2. 建立 launchd plist 在開機時自動 route add
3. 驗證重開機後路由仍走 en5
4. 歸檔至 Scientia
5. 更新 bus-diagnosis.sh 加入持久化檢查

### F. trading-v4 防火牆與常駐
1. 確認 trading-v4 防火牆已開放 19284
2. 設定 Task Scheduler / NSSM 常駐
3. 從 orchestrator 驗證 trading 持續在線
4. 更新 service-manifest.yaml trading 為 grpc+ssh
5. 歸檔

### G. SakiAgentSSH 開源化準備
1. 確認 winget manifest 安裝路徑為 C:\Program Files\SakiAgentSSH\
2. 更新 install.ps1 預設路徑
3. App Store Review 回覆（移除 network.server entitlement）
4. GitHub Release 準備（但禁止 push）
5. 歸檔
