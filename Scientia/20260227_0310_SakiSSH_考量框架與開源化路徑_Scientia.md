# SakiSSH 考量框架摘要（Scientia 歸檔）

> **建立時間**：2026-02-27 03:10 (UTC+8)
> **完整文件**：Brain artifact `sakissh_consideration_framework.md`
> **標籤**：#SakiSSH #POSIX #開源化 #SakiMeasureMAP #架構共鳴

## 研究結論

### 1. POSIX 障礙已由 gRPC 架構消除
- TTY 互動、編碼地獄、PATH 繼承、ACL 衝突、殭屍進程 → 全部由現有 SakiSSH 設計解決

### 2. 雙軌架構安全確認
- SakiClip: Port 19283 (Human-facing, TCP Binary)
- SakiSSH: Port 19284 (Agent-facing, gRPC/HTTP2)
- 兩者共用物理網卡但端口完全隔離，互不干擾

### 3. 多管道連接方案
- 物理端口只有 19284 一個
- Client 端實現 LAN (192.168.50.x) → Tailscale (100.x.x.x) Failover
- Daemon 端 bind `[::0]:19284` 無需修改
- gRPC HTTP/2 天生支援串流多路復用

### 4. 開源化路徑
- Core 層（proto + daemon + client + auth）可開源
- Ext 層（orchestrator + MeasureMAP 整合）保持私有
- 開源前需抽離 Nushell 硬編碼與 Saki 內部 IP

### 5. 演進四階段
- Phase 1：安全加固（mTLS / IP ACL）
- Phase 2：協議擴展（FileTransfer / SystemInfo / 信號轉譯）
- Phase 3：開源準備（可配置化 + proto v1 命名空間）
- Phase 4：SakiMeasureMAP 對接（圖資串流 + 多節點調度）

### 6. Antigravity 架構共鳴
- ConnectRPC ≅ gRPC、jetskiAgent 指令降維 ≅ ExecuteRequest 最小化
- 證明 SakiSSH 與業界先進 Agent 系統同構
