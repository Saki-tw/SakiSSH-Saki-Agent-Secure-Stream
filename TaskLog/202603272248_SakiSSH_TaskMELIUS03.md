# SakiAgentSSH v3.0 協議實作 TaskMELIUS 03

> 建立時間：2026-03-27 22:48 (UTC+8)
> 對應 task.md Phase 6

## 任務第一步：Proto 擴展

1. 讀取現有 proto/sakissh.proto 確認當前內容
2. 新增 Authenticate / GetCapabilities / RenewSession RPC
3. 新增 AuthRequest/AuthResponse/CapabilityRequest/CapabilityResponse 等 message
4. 擴展 AgentSshError enum（50-79 區段）
5. 確認 build.rs 正確引用 proto → 開始 auth.rs

## 任務第二步：auth.rs 實作

1. 建立 AgentAuthenticator 結構與 ED25519 驗證邏輯
2. 實作 verify() 方法（nonce challenge-response）
3. 實作 authorized_agents.json 載入
4. 與 config.rs 整合（新增 AuthConfig）
5. 確認下一步：capability.rs

## 任務第三步：capability.rs + session.rs 實作

1. 建立 CapabilitySet 結構（allowed/denied commands/paths）
2. 實作 check_command() / check_path() 方法
3. 建立 SessionManager（HashMap<session_id, AuthenticatedAgent>）
4. 實作 session 過期自動清除（tokio timer）
5. 確認下一步：audit.rs + 整合

## 任務第四步：audit.rs + daemon 整合

1. 建立 AuditLogger（append-only log file）
2. 實作 Tower interceptor：capability_interceptor
3. 整合 main.rs：加入 interceptor chain + 新 RPC handler
4. 整合所有新 RPC 到 SakiSSH service impl
5. 確認下一步：client 更新

## 任務第五步：Client 更新 + 編譯驗證

1. 更新 saki-ssh-client key 管理（~/.sakissh/id_ed25519）
2. 實作 authenticate() client 端流程
3. 更新 Cargo.toml 加入新依賴
4. 清除快取後完整編譯驗證
5. 產出結果：更新 Walkthrough + 下一步規劃
