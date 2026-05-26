# SakiStarCommuncation 匯流排建構完成 Walkthrough

> 時間：2026-03-03 21:12 (TST)
> 專案簡稱：SakiStar
> 對應 Task.md Phase 1-4 + 後續展開

## 完成摘要

### SakiAgentSSH 部署 (核心成果)
- Mac 本機 + Windows x86_64 交叉編譯 ✅
- loser (HP 8A4F) gRPC Ping/Execute ✅ (v1.0.0, pwsh 7.5.4)
- trading-v4 (ASUS) gRPC Ping ✅ (v1.0.0, powershell.exe)
- 兩節點均已部署 sakisshd.exe + config.json (ACL: LAN + Tailscale)

### saki-orchestrator 整合
- heartbeat 改為 gRPC 優先 SSH 備援
- BusInterface struct 新增各節點匯流排資訊
- loser-win: SakiAgentSSH=active (gRPC)
- trading: SakiAgentSSH=active (gRPC)（已更新配置）

### 路由修正
- en5 (USB 2.5G) 已在網路服務順序第一位
- `sudo route add 192.168.50.0/24 -interface en5` 修正路由快取 ✅

### 文檔更新
- ARCHITECTURE.md §3.7 匯流排拓撲章節
- service-manifest.yaml loser-win 改 grpc+ssh
- bus-diagnosis.sh 新建（bash 3 相容）

## 修改檔案清單
- `SakiStarCommuncation/saki-orchestrator/src/main.rs`
- `SakiStarCommuncation/scripts/bus-diagnosis.sh` (新建)
- `SakiStarCommuncation/ARCHITECTURE.md`
- `SakiStarCommuncation/service-manifest.yaml`

## 後續
- Windows Service 支持需加入 `windows-service` crate（Error 1053 問題）
- 用戶需在兩台 Windows 端設定 Task Scheduler/NSSM 常駐
