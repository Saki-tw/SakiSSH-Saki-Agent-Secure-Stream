# SakiAgentSSH v5.0 TaskMELIUS 01 — Phase 0 + Phase 1

> 建立時間：2026-05-22 11:58 (UTC+8)
> 對應 Task.md：Phase 0 + Phase 1
> 對應 Session：6cfbbcda-37b7-4e8c-a464-22b4de7e20b9

---

## 任務第一步：RFC v5.0 協議設計

1. 讀取現有 RFC 草案 `docs/pages/draft-saki-sakissh-protocol-00.md` 完整內容
2. 讀取現有 `proto/sakissh.proto` 完整內容
3. 設計 v5.0 五層協議棧（TCP → TLS 1.3 → gRPC/HTTP2 → Agent RPC → Threat Defense）
4. 擴充 proto 定義：新增 ChaChaCognitiveChallenge RPC、TlsConfig message、PolicyRule message
5. 撰寫 RFC draft-01 正式文件（IETF 風格）

## 任務第二步：TLS/mTLS Rust 實作

1. 讀取 Daemon 和 Client 的 Cargo.toml 確認現有依賴
2. 新增 rustls 相關依賴
3. 實作 config.rs TlsConfig 結構
4. Daemon main.rs 加入 ServerTlsConfig
5. 開始 Phase 1 其餘實作並展開下一份 TaskMELIUS

## 任務第三步：Client TLS + CA 工具

1. Client main.rs 加入 ClientTlsConfig
2. 建立 sakissh-ca 工具專案結構
3. 實作 CA init / issue / revoke 子命令
4. 產出測試憑證
5. M1 本機 TLS E2E 測試並確認下一步

## 任務第四步：驗證與歸檔

1. TLS 連線測試（openssl s_client 驗證）
2. mTLS 拒絕無憑證客戶端測試
3. 更新 ARCHITECTURE.md 反映 TLS 變更
4. 歸檔 Phase 0+1 成果至 Scientia
5. 開始 Phase 2 ChaCha20-13Policy 強化
