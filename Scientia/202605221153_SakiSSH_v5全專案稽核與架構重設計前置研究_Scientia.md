# SakiAgentSSH v5.0 全專案稽核與架構重設計 — 前置研究

> 建立時間：2026-05-22 11:53 (UTC+8)
> 研究方法：三路平行研究（Codebase 分析 + 歷史考古 + 安全/協議評估）
> 研究目的：為 ConsultumUltimum 全任務提供決策基礎

---

## 一、研究方法論

本研究採用三路平行子任務架構，分別對 SakiAgentSSH 進行：
1. **代碼庫深度分析**：架構、語言、模組、依賴、部署通路
2. **歷史演進考古**：11 個 ChatMelius Session、26 篇 Scientia、27 篇 TaskLog
3. **安全與協議評估**：Proto 定義、加密機制、認證流程、RFC 與實作落差

---

## 二、關鍵發現

### 2.1 致命安全缺陷
- **明文傳輸**：所有 gRPC 通訊為裸 HTTP/2，Token、指令、檔案可被中間人截取
- **RFC 幻覺**：規格書聲稱有 SSH Transport Layer（X25519 + ChaCha20），但代碼中完全沒有
- **13Policy 僅 4 個硬編碼關鍵字**：`rm -rf /`, `mkfs`, `dd if=/dev/zero`, fork bomb
- **ChaCha20 挑戰生成但不驗證回應**：`threat_defense.rs` 註解明確寫 "In a real implementation..."

### 2.2 傳輸層安全方案評估

| 方案 | 與 gRPC 整合 | 實作複雜度 | 安全性 | 推薦 |
|------|-------------|-----------|--------|------|
| 自建 SSH Transport | 差（需包裝 HTTP/2） | 極高 | 取決於實作品質 | ❌ |
| TLS 1.3 + mTLS | 完美（tonic 原生） | 低 | 業界標準 | ✅ |
| 自訂 CFR | 未知 | 極高 | 未經審計 | ❌ |

結論：TLS 1.3 是唯一合理選擇。ChaCha20-Poly1305 作為 TLS cipher suite 自然整合。

### 2.3 Go 雙實作可行性
- Go 交叉編譯可一次解決 U9 的 gnullvm linker 問題
- Go 標準庫內建 `crypto/ed25519`、`crypto/tls`，零外部依賴
- gRPC-Go 為官方維護，品質可靠
- 預估工作量 ~3,000 行 Go

### 2.4 部署歷史教訓
- U9：gnullvm linker 失敗、PATH 環境變數未生效
- Loser：cmd.exe 編譯 39 分鐘無輸出（protoc 可能缺失）
- Trading：完全無 dev tools
- **結論**：遠端機器應推送預編譯 binary，避免在遠端編譯

---

## 三、決策建議（供使用者確認）

1. 傳輸層改用 TLS 1.3 + mTLS（棄用 SSH Transport 幻影）
2. Go 完整雙實作（Daemon + Client），Rust 保持為參考實作
3. U9 以 Go 預編譯 binary 部署（繞過 Rust 交叉編譯問題）
4. 13Policy 從硬編碼改為可配置 YAML 規則引擎
5. ChaCha20 認知挑戰補完驗證流程

---

## 四、參考來源

- `SakiAgentSSH/ARCHITECTURE.md`（已過時，需更新）
- `SakiAgentSSH/proto/sakissh.proto`（305 行，v4.0）
- `SakiAgentSSH/docs/pages/draft-saki-sakissh-protocol-00.md`（RFC 草案）
- ChatMelius #294（v3.0 Protocol + 四節點部署）
- ChatMelius #20260331（全面稽核）
- ChatMelius #20260332（遺留資料同步 + 安全強化）
- 全域工程哲學 `202603010730_GlobalPhilosophy.AGENDUM`
