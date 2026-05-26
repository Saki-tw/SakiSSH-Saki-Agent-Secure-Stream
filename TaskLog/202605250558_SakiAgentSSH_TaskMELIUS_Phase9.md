# SakiAgentSSH Phase 9 TaskMELIUS 01 (Architecture Rectification)

> **建立時間**：2026-05-25 05:58 (UTC+8)
> **專案**：SakiAgentSSH
> **屬性**：任務清單 (TaskMELIUS)
> **背景**：針對 v6.0 Phase 8 盤點中發現的矛盾與實作落差進行修補。

---

## 任務第一步：Rust Daemon 金鑰持久化與 Audit RPC
- [ ] 1. 編輯 `proto/sakissh.proto`，新增 `GetAuditPublicKey` RPC 與對應 Message。
- [ ] 2. 編輯 `saki-ssh-daemon/src/audit.rs`，加入讀寫 `~/.config/sass/audit_key.pem` 與 `audit_pub.pem` 的邏輯。若不存在則產生 Ed25519 金鑰對，權限 `chmod 600`。
- [ ] 3. 編輯 `saki-ssh-daemon/src/main.rs`，實作 `get_audit_public_key` 端點，限制為攜帶 token 且通過 ACL 驗證之請求。
- [ ] 4. 執行 `cargo check` 驗證 Rust 編譯。
- [ ] 5. 轉向下一階段：Rust Daemon 分支隔離機制的跨平台實作（任務第二步）。

## 任務第二步：Rust Daemon 微型分支與作業系統隔離
- [ ] 1. 編輯 `saki-ssh-daemon/src/branch_mgr.rs`。
- [ ] 2. 針對 macOS 實作純 Userspace Symlink Tree 隔離。
- [ ] 3. 針對 Linux 實作 User Namespaces 呼叫與 Rootless OverlayFS 掛載。
- [ ] 4. 執行 `cargo check` 確認跨 OS target (`#[cfg]`) 能否正確編譯。
- [ ] 5. 轉向下一階段：Go Daemon 實作對齊（任務第三步）。

## 任務第三步：Go Daemon 防禦層對齊 (Env, Branch, Audit, Tarpit)
- [ ] 1. 建立 `go-sakissh/internal/server/env_injector.go`，移植環境變數攔截。
- [ ] 2. 建立 `go-sakissh/internal/server/branch_mgr.go`，移植 Symlink Tree/OverlayFS 呼叫邏輯。
- [ ] 3. 建立 `go-sakissh/internal/server/audit.go`，移植 Ed25519 金鑰對產生、寫入 PEM，並與 Rust 版本完全相同的 SHA256 Hash Chain `audit.jsonl` 輸出格式。
- [ ] 4. 建立 `go-sakissh/internal/server/tarpit.go`，實作 13Policy 規則檢查與 40MB 焦油坑。
- [ ] 5. 轉向下一階段：Go Daemon RPC 整合與編譯驗證（任務第四步）。

## 任務第四步：Go Daemon Execute 整合與驗證
- [ ] 1. 修改 `go-sakissh/internal/server/execute.go` 引用上述套件，串接 Tarpit、Branch、Env、Audit 防禦鏈。
- [ ] 2. 實作 `GetAuditPublicKey` RPC。
- [ ] 3. 執行 `go build ./...` 驗證編譯。
- [ ] 4. 若可能，執行簡易單元測試或手工測試。
- [ ] 5. 轉向下一階段：成果歸檔與撰寫 Walkthrough（任務第五步）。

## 任務第五步：歸檔與報告
- [ ] 1. 產生 Walkthrough 報告至 `WalkthroughLog/`。
- [ ] 2. 將本 TaskMELIUS 與對應的 Implementation Plan 歸檔。
- [ ] 3. 更新 `saki-ssh-daemon/Cargo.toml` 與 `go-sakissh/go.mod` 版本號（若需要）。
- [ ] 4. 確認四大再現性目標已完全滿足。
- [ ] 5. 任務完成。
