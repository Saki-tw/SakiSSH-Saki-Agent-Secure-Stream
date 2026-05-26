# SakiSSH 實作與跨機整合 (Phase 13-15) 實作計畫

> **建立時間**：2026-02-25 19:57 (UTC+8)
> **專案**：SakiStarCommuncation

## 1. 架構決策
- **放棄方案**: 在 Windows 上安裝 LLVM/GNU 工具鏈（太重）。
- **採用方案**: 使用 \`cargo-xwin\` 在 macOS M1 上交叉編譯 Windows 執行檔。
- **通訊協議**: gRPC (Tonic)，解決 SSH 編碼與多路復用效能問題。
- **殼層策略**: **核心對接方僅限 Nushell**。Daemon 預設使用 \`nu -c\` 包裝指令。
- **擴展目標**: 支援直接透過 SakiSSH 呼叫 \`gemini\` CLI，實現跨機 Agent 指令傳遞。

## 2. 實作進度
- [x] **交叉編譯環境建立**: 已安裝 \`cargo-xwin\` 並成功下載 Windows SDK。
- [x] **Daemon (Windows)**: \`sakisshd.exe\` 已編譯成功，**強制使用 Nushell 執行**。
- [x] **Client (macOS)**: \`sakissh\` (Rust 版) 已編譯並支援 SSH 別名解析。
- [x] **整合至 remote-build.sh**: 已實作 \`resolve_ssh_host\` 並完成 \`remote_exec\` 替換。
- [ ] **部署與連通性測試**: 將 \`.exe\` 推送到 Loser PC 並啟動服務。

## 3. 待辦事項 (Phase 15 續行)
1. 將 `sakisshd.exe` 複製到 `SakiStarCommuncation/bin/windows/` 歸檔。
2. 修改 `remote-build.sh` 邏輯，加入 `SAKISSH_EXE` 呼叫分支。
3. 在目標 Windows 機器設定防火牆規則 (Port 19284)。
