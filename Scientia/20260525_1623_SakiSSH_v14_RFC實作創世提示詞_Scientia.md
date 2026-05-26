# SASS v1.4 RFC 實作 創世提示詞

> **生成時間**：20260525_1623 (UTC+8)
> **前 Session ID**：25a3dcaa-a700-422b-b0cd-2001c13e485d
> **生成原因**：CHECKPOINT 4 + 前 Session 全部實作為幻覺產物，已 git checkout 恢復。等同零進度重新開始。

---

## 任務目標 (Mission Statement)

將 SASS (Saki Agent Secure Stream) v1.4 的「幾乎處處優越 (AES)」正式實作於 Rust 守護行程中，並完成 RFC 協定文件正式化。

---

## 專案位置與關鍵文檔 (Context Map)

| 項目 | 路徑 | 說明 |
|------|------|------|
| 專案根目錄 | `/Users/hc1034/Saki_Studio/Claude/SakiAgentSSH/` | SASS 主目錄 |
| Daemon 原始碼 | `saki-ssh-daemon/src/` | Rust daemon 核心 |
| Client 原始碼 | `saki-ssh-client/src/` | Rust client |
| Proto 定義 | `proto/sakissh.proto` | v1.4 已清除 ChaChaCognitiveChallenge |
| RFC 草案 v0 | `docs/pages/draft-saki-sakissh-protocol-00.md` | 初版 |
| RFC 草案 v1 | `docs/pages/draft-saki-sakissh-protocol-01.md` | 最新草案（468行） |
| 架構文件 | `ARCHITECTURE.md` | 架構概覽 |
| 建置文件 | `BUILDING.md` / `BUILDING_zh-TW.md` | 編譯指引 |
| CA 工具 | `sakissh-ca/` | 憑證管理 |

### 必讀文件（按優先級）

1. `ARCHITECTURE.md`（~100行）— 架構概覽
2. `saki-ssh-daemon/src/main.rs` — Daemon 核心邏輯
3. `proto/sakissh.proto` — 已清理的 proto 定義
4. `docs/pages/draft-saki-sakissh-protocol-01.md`（468行）— 最新 RFC

### ⚠️ 紅線警告

**SakiAgentSSH 與 STLS Proxy（`SakiDeusExAgent/DeusExAntigravityTrueLanguageServer/saki-stls-proxy/`）是完全無關的兩個專案。** 經 rg 搜尋驗證：零程式碼引用、零共用 crate、零共用 proto、零架構關聯。**絕對不可混淆。**

| 專案 | 本質 | 技術棧 |
|------|------|--------|
| SakiAgentSSH (SASS) | gRPC/mTLS 遠端執行框架 | ed25519 + tonic + rustls |
| STLS Proxy | Antigravity IDE ConnectRPC 中間人 | ConnectRPC + ephemeral_message 攔截 |

---

## 已完成事項 (Completed Work)

| 項目 | 狀態 | 證據 | 驗證指令 |
|------|:---:|------|---------| 
| 千點流變考古 | ✅ | ChatMelius/320 | — |
| SASS v1.4 理論定案 | ✅ | ChatMelius/320 | — |
| Protobuf 拔除 ChaChaCognitiveChallenge | ✅ | `proto/sakissh.proto` | `grep ChaChaCognitive proto/sakissh.proto`（應零命中） |
| 兩專案無關性證明 | ✅ | rg 搜尋（本 Session） | `rg -i stls saki-ssh-daemon/`（應零命中） |

---

## 未完成事項 (Remaining Work)

> 🔴 以下全部為未開始狀態。前 Session 的「實作」全部是幻覺，已被 git checkout 恢復。

### Phase 1：Daemon 程式碼清理 🟢

- [ ] 進入 `saki-ssh-daemon/src/`，盤點所有 `.rs` 檔案
- [ ] 清除所有與舊版 `ChaChaCognitiveChallenge` 相關的 gRPC handler 邏輯
- [ ] 確認 `Cargo.toml` 依賴與 proto 定義一致
- [ ] 驗證：`cargo check`（daemon 子 crate）

### Phase 2：L4 TLS 握手攔截 🟡

- [ ] 研究在 `tonic`/`rustls` 框架下如何進行 TLS Custom Extension
- [ ] 實作 TLS-Exporter 機制，強制 ChaCha20 挑戰
- [ ] 無算力 → 立即 Drop 連線
- [ ] 修改 `saki-ssh-daemon/src/main.rs` 或新增 `tls_challenge.rs`
- [ ] 驗證：單元測試 + 手動連線測試

### Phase 3：雙軌制防禦落地 🟢

- [ ] 修改 `policy.rs` 或 `session.rs` 中的越界裁定邏輯
- [ ] **已認證但越界的內部 Agent** → Vi Swap 停滯機制（回傳靜態 terminal escape sequence 卡死 LLM）
- [ ] **未認證/惡意外部 Agent** → Zero-Allocation Tarpit（64KiB 無限切片餿水）
- [ ] 驗證：tarpit 測試 + Vi Swap 行為測試

### Phase 4：XOR 本機溢位防禦 🟢

- [ ] 將 localhost 防禦從「假檔案系統」升級為「直接 XOR 混淆回傳」
- [ ] 修改相關 handler
- [ ] 驗證：本機連線測試

### Phase 5：RFC 協定文件正式化 🟢

- [ ] 基於 `draft-saki-sakissh-protocol-01.md` 更新至 v1.4 最終版
- [ ] 確保 RFC 與實作完全對應
- [ ] 更新 `ARCHITECTURE.md` 反映新增的防禦機制

---

## 技術棧速查 (Tech Stack Quick Ref)

| 項目 | 版本/規格 |
|------|----------|
| Rust | 確認 `rustc --version`（預期 1.93+） |
| tonic | gRPC 框架 |
| rustls | TLS 實作 |
| ed25519 | 簽章演算法 |
| proto | `proto/sakissh.proto`（已清理 v1.4） |

---

## 建議執行順序 (Recommended Execution Order)

1. **Phase 1** → 清理 daemon 程式碼（無依賴，基礎工作）
2. **Phase 4** → XOR 本機溢位防禦（獨立模組，可先做）
3. **Phase 3** → 雙軌制防禦（依賴 policy 架構理解）
4. **Phase 2** → L4 TLS 攔截（最複雜，需研究 tonic/rustls extension）
5. **Phase 5** → RFC 文件更新（收尾，需所有實作完成後對照）

---

## 踩坑清單與限制事項 (Gotchas & Constraints)

1. **不要碰 STLS Proxy** — `SakiDeusExAgent/` 下的東西跟本專案完全無關
2. **proto 已清理** — `ChaChaCognitiveChallenge` 已從 proto 移除，daemon 程式碼中的殘留需手動清除
3. **`language_server.real`** — 目前 LS 是 `.real` 版本（STLS 代理層已部署），但這與 SASS 開發無關
4. **Git 狀態乾淨** — 所有追蹤檔案已恢復，untracked 檔案（config、logs、dSYM 等）保持原樣
5. **非同步協作架構** — 完成每個 Phase 後應歸檔至 TaskLog/ImplementationLog/Scientia
6. **所有編譯先清快取** — `cargo clean && cargo build`

---

## 錯誤統計

| 類型 | 數量 | 說明 |
|------|------|------|
| 專案混淆幻覺 | 1（致命） | 將 SASS 與 STLS Proxy 混為一體，所有實作無效 |
| 無效 artifact | 3+ | task.md, walkthrough.md, implementation_plan.md 全部基於錯誤前提 |
| 使用者糾正次數 | 4+ | 使用者多次明確指出錯誤 |
