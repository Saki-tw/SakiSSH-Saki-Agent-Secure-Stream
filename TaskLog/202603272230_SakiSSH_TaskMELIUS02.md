# SakiAgentSSH 協議研究 TaskMELIUS 02：gRPC+SSH 混合協議架構設計

> 建立時間：2026-03-27 22:30 (UTC+8)
> 對應 task.md Phase 2

## 任務第一步：gRPC over TCP + SSH 認證混合協議棧設計

1. 分析 RFC 4253 SSH Transport Layer 如何映射到 gRPC/HTTP2
2. 確定協議分層：TCP → SSH Transport → gRPC → Protobuf
3. 確認 tonic (Rust gRPC) 是否支援 Custom Transport
4. 設計協議版本協商機制
5. 確認下一步：key exchange 機制

## 任務第二步：SSH 風格 Key Exchange 機制設計

1. 研究 RFC 4253 §8 Diffie-Hellman Key Exchange 簡化方案
2. 設計 ED25519 key pair 認證流程（取代 RSA/DSA）
3. 確定 key exchange 對 gRPC TLS 的關係（互補 vs 替代）
4. 設計 session key 衍生與 rekey 機制
5. 確認下一步：Agent 邊界限制方法論

## 任務第三步：Agent 儲存邊界限制方法論設計

1. 基於 Phase 1 研究定義五維邊界：路徑/指令/環境/網路/時間
2. 設計 capability-based 權限模型（每 key 綁定 capability set）
3. 設計 daemon 側的 chroot-like 隔離（不依賴 OS chroot）
4. 設計行為審計與異常偵測框架
5. 確認下一步：跨 OS 依賴最小化

## 任務第四步：跨 OS 最低依賴架構設計

1. 確定 Rust 為唯一硬依賴（零外部 runtime）
2. 設計 OS 特定功能降級策略（Windows 無 POSIX、macOS 無 namespace）
3. 確認 protobuf/tonic 的跨平台狀態
4. 設計配置格式標準化（JSON5 or TOML）
5. 確認下一步：撰寫協議規範

## 任務第五步：撰寫協議規範 Scientia

1. 撰寫協議棧全貌圖
2. 撰寫認證流程時序圖
3. 撰寫 proto 擴展規範
4. 撰寫跨 Agent 適配層規範
5. 產出至 Scientia/ 並開始 Phase 3
