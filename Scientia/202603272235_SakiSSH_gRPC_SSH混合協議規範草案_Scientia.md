# SakiAgentSSH 協議規範 — gRPC over TCP + SSH 權限管理之跨 OS Agent 協議

> **建立時間**：2026-03-27 22:35 (UTC+8)
> **版本**：Draft 0.1
> **狀態**：Phase 2 設計草案
> **標籤**：#協議設計 #gRPC #SSH #RFC4253 #Agent邊界

---

## 1. 協議總覽

### 1.1 設計目標

建立一個以 **gRPC over TCP** 為基礎，融合 **SSH (RFC 4253)** 權限管理能力的跨作業系統 Agent 執行協議。此協議：

1. 對 Agent 工具執行的**儲存空間邊界**實施方法論上有效之限制
2. 跨 OS 最低依賴（除 Rust 外零外部 runtime）
3. 向後相容現有 SakiAgentSSH v2.0 協議

### 1.2 協議棧

```
┌──────────────────────────────────────┐
│ Layer 5: Agent RPC (Protobuf)        │  ← 現有 sakissh.proto
│   Execute / FileUpload / Cancel etc. │
├──────────────────────────────────────┤
│ Layer 4: gRPC / HTTP/2               │  ← tonic 0.12+
│   Multiplexed Streams                │
├──────────────────────────────────────┤
│ Layer 3: SSH Channel Authorization   │  ← 🔴 新增層
│   Capability-Based Permission        │
│   Key Exchange + Session Key         │
├──────────────────────────────────────┤
│ Layer 2: SSH Transport (RFC 4253)    │  ← 🔴 新增層
│   Encryption + MAC + Compression     │
│   Binary Packet Protocol             │
├──────────────────────────────────────┤
│ Layer 1: TCP                         │  ← 現有 19284 port
└──────────────────────────────────────┘
```

### 1.3 與現有架構的關係

| 功能 | v2.0 現有 | v3.0 新增 |
|------|-----------|-----------|
| 傳輸加密 | 無（明文 gRPC）/ 可選 TLS | SSH Transport 強制加密 |
| 認證 | Token header + CIDR ACL | SSH Key Pair (ED25519) |
| 路徑限制 | FileTransferConfig 白名單 | **Capability Set 路徑限制** |
| 指令限制 | 無 | **Capability Set 指令白名單** |
| Session 管理 | 無（無狀態） | **SSH Channel 有狀態 Session** |
| 審計 | 無 | **Audit Log 強制** |

---

## 2. SSH Transport 層設計 (Layer 2)

### 2.1 協議版本識別

遵循 RFC 4253 §4.2，連線建立後交換版本識別字串：

```
SAKISSH-3.0-{softwareversion} {comments}\r\n
```

- `SAKISSH` 前綴區別於標準 SSH（避免被 SSH scanner 誤判）
- `3.0` 為協議主版本
- `softwareversion` = `SakiAgentSSH/{daemon-version}`

### 2.2 Binary Packet Protocol

遵循 RFC 4253 §6 的 Binary Packet 格式：

```
uint32    packet_length
byte      padding_length
byte[n1]  payload        ; n1 = packet_length - padding_length - 1
byte[n2]  random_padding ; n2 = padding_length
byte[m]   mac            ; m = mac_length (0 before key exchange)
```

### 2.3 Key Exchange（簡化版 Diffie-Hellman）

基於 RFC 4253 §8，但簡化為：

1. **演算法協商**：固定使用 `curve25519-sha256`（無協商，最低依賴）
2. **Key Exchange**：X25519 Diffie-Hellman
3. **Host Key**：ED25519（daemon 端固定密鑰對）
4. **Session Key 衍生**：HKDF-SHA256
5. **加密**：ChaCha20-Poly1305（現代、高效、Rust 生態豐富）
6. **MAC**：Poly1305（與 ChaCha20 綁定）
7. **Rekey**：每 1GB 或 1 小時觸發

### 2.4 與 TLS 的比較

| 面向 | SSH Transport | TLS 1.3 | 選擇 SSH 的理由 |
|------|:------------:|:-------:|:---------------|
| 模型差異 | 信任首次使用 (TOFU) | CA 鏈 | Agent 場景無 CA |
| 金鑰管理 | authorized_keys 文件 | 證書+CA | 更適合 admin 手動管理 |
| Channel 多工 | ✅ 原生 | ❌（需 HTTP/2） | SSH channel 正好映射 gRPC stream |
| 外部依賴 | 零（純 Rust） | rustls/openssl | SSH 可內嵌 |
| 權限攜帶 | ✅ key→capability | ❌ | 核心差異 |

---

## 3. SSH Channel Authorization 層設計 (Layer 3)

### 3.1 Capability-Based Permission Model

每個 SSH key 綁定一組 **Capability Set**，存儲於 daemon 的 `authorized_agents.json`：

```json
{
  "agents": [
    {
      "name": "gemini-cli@m1-mac",
      "public_key": "ssh-ed25519 AAAA...",
      "capabilities": {
        "execute": {
          "allowed_commands": ["ls", "cat", "grep", "cargo", "npm"],
          "denied_commands": ["rm -rf", "sudo", "chmod 777"],
          "allowed_cwd": ["/Users/hc1034/Saki_Studio/"],
          "max_concurrent": 5,
          "timeout_seconds": 300
        },
        "file_transfer": {
          "allowed_paths": ["/Users/hc1034/Saki_Studio/"],
          "denied_paths": ["~/.ssh", "~/.aws", "~/.gnupg"],
          "max_file_size_mb": 100,
          "allowed_directions": ["upload", "download"]
        },
        "environment": {
          "inherit_daemon_env": false,
          "allowed_env_vars": ["PATH", "HOME", "LANG"],
          "inject_env": {
            "SAKISSH_AGENT": "gemini-cli",
            "SAKISSH_SESSION": "${SESSION_ID}"
          }
        },
        "session": {
          "max_duration_seconds": 3600,
          "max_sessions": 3,
          "idle_timeout_seconds": 600
        }
      }
    }
  ]
}
```

### 3.2 五維邊界限制模型

| 維度 | 限制機制 | 強制層級 |
|------|----------|----------|
| **路徑 (Path)** | `allowed_paths` / `denied_paths` 前綴匹配 | Daemon 側強制 |
| **指令 (Command)** | `allowed_commands` glob 匹配 + deny 優先 | Daemon 側強制 |
| **環境 (Environment)** | 不繼承 daemon env + 注入 session metadata | Daemon 側強制 |
| **網路 (Network)** | SSH channel 本身無直接網路授權；透過指令限制間接控制 | 間接 |
| **時間 (Time)** | `max_duration` + `idle_timeout` + rekey 週期 | Daemon 側強制 |

### 3.3 認證流程時序圖

```
Client (Agent)                              Daemon
    |                                          |
    |──── TCP Connect (port 19284) ──────────►|
    |                                          |
    |◄──── SAKISSH-3.0-... version string ────|
    |──── SAKISSH-3.0-... version string ────►|
    |                                          |
    |◄──── SSH_MSG_KEXINIT ───────────────────|
    |──── SSH_MSG_KEXINIT ───────────────────►|
    |                                          |
    |   ┌── X25519 Key Exchange ──┐            |
    |   │  derive session keys     │           |
    |   └──────────────────────────┘           |
    |                                          |
    |──── SSH_MSG_NEWKEYS ───────────────────►|
    |◄──── SSH_MSG_NEWKEYS ───────────────────|
    |                                          |
    |   [加密通道已建立]                        |
    |                                          |
    |──── UserAuth (ED25519 signature) ──────►|
    |                                          |
    |   Daemon: lookup public_key              |
    |           → 載入 capability set          |
    |                                          |
    |◄──── UserAuth SUCCESS ──────────────────|
    |       (capability_hash in response)      |
    |                                          |
    |   [SSH Channel 開啟 → gRPC HTTP/2]       |
    |                                          |
    |──── gRPC Execute("ls /tmp") ───────────►|
    |   Daemon: check capability               |
    |     ✅ "ls" in allowed_commands          |
    |     ✅ "/tmp" under allowed_cwd          |
    |◄──── gRPC ExecuteResponse ──────────────|
    |                                          |
    |──── gRPC Execute("sudo rm -rf /") ─────►|
    |   Daemon: check capability               |
    |     🔴 "sudo" in denied_commands         |
    |◄──── gRPC Status(PERMISSION_DENIED) ────|
    |       ErrorDetail: ERROR_CAPABILITY_DENIED|
    |                                          |
```

---

## 4. 跨 OS 最低依賴架構

### 4.1 Rust 依賴清單

| Crate | 用途 | 跨平台 |
|-------|------|--------|
| `tonic` 0.12 | gRPC 框架 | ✅ |
| `prost` 0.13 | Protobuf | ✅ |
| `tokio` 1.0 | 非同步 runtime | ✅ |
| `ed25519-dalek` | ED25519 簽名 | ✅ 純 Rust |
| `x25519-dalek` | X25519 key exchange | ✅ 純 Rust |
| `chacha20poly1305` | 加密 | ✅ 純 Rust (RustCrypto) |
| `hkdf` + `sha2` | Key derivation | ✅ 純 Rust (RustCrypto) |
| `ipnet` | CIDR ACL（保留） | ✅ |

### 4.2 零外部依賴聲明

- **不依賴 OpenSSL**：全部使用 RustCrypto 純 Rust 實作
- **不依賴 libssh**：自行實作 SSH Transport 子集
- **不依賴 OS 沙箱**：邊界限制在 daemon application 層強制
- **不依賴 CA/PKI**：使用 SSH 風格 TOFU 模型

### 4.3 OS 特定降級策略

| OS | 完整功能 | 降級項目 |
|----|----------|----------|
| macOS | ✅ 全功能 | 無 |
| Linux | ✅ 全功能 | 無 |
| Windows | ✅ 核心功能 | POSIX Signal → TerminateProcess 映射 |
| FreeBSD | ⚠️ 未測試 | 需驗證 tokio 相容性 |

---

## 5. Proto 擴展規範（向後相容）

### 5.1 新增 RPC 端點

```protobuf
service SakiSSH {
  // 現有 7 個 RPC 保持不變...

  // 🔴 v3.0 新增
  // SSH 認證（在 gRPC 層暴露 SSH 認證狀態）
  rpc Authenticate(AuthRequest) returns (AuthResponse);

  // Capability 查詢（client 可查詢自身權限）
  rpc GetCapabilities(CapabilityRequest) returns (CapabilityResponse);

  // Session 管理
  rpc RenewSession(SessionRenewRequest) returns (SessionRenewResponse);
}
```

### 5.2 新增 Message 類型

```protobuf
message AuthRequest {
  string agent_name = 1;
  bytes public_key = 2;
  bytes signature = 3;        // ED25519 簽名 of nonce
  bytes nonce = 4;
}

message AuthResponse {
  bool success = 1;
  string session_id = 2;
  bytes capability_hash = 3;  // SHA256 of capability set
  uint64 expires_at = 4;      // Unix timestamp
}

message CapabilityRequest {}

message CapabilityResponse {
  repeated string allowed_commands = 1;
  repeated string allowed_paths = 2;
  repeated string denied_paths = 3;
  uint32 max_concurrent = 4;
  uint32 timeout_seconds = 5;
}

message SessionRenewRequest {
  string session_id = 1;
}

message SessionRenewResponse {
  bool success = 1;
  uint64 new_expires_at = 2;
}

// 新增錯誤碼
enum AgentSshError {
  // 現有 0-49 保留...

  // Capability (50-59) 🔴 v3.0 新增
  ERROR_CAPABILITY_DENIED = 50;       // 操作不在 capability set 內
  ERROR_CAPABILITY_PATH_DENIED = 51;  // 路徑不在允許清單
  ERROR_CAPABILITY_CMD_DENIED = 52;   // 指令不在允許清單
  ERROR_CAPABILITY_EXPIRED = 53;      // Session 已過期

  // Session (60-69) 🔴 v3.0 新增
  ERROR_SESSION_NOT_FOUND = 60;
  ERROR_SESSION_EXPIRED = 61;
  ERROR_SESSION_LIMIT = 62;           // 超過最大 session 數
}
```

---

## 6. 對各 Agent 適配方案

### 6.1 Gemini CLI 適配

```
Gemini CLI → saki-ssh-client → SakiSSH daemon
           ED25519 key: ~/.gemini/sakissh/id_ed25519
           Capability: 受限於 Saki_Studio 工作目錄
```

### 6.2 Antigravity 適配

```
Antigravity → saki-ssh-client (via LS command)
            ED25519 key: ~/.gemini/antigravity/sakissh/id_ed25519
            Capability: 完整開發權限（trusted agent）
```

### 6.3 Claude Code 適配

```
Claude Code → saki-ssh-client → SakiSSH daemon
            ED25519 key: ~/.claude/sakissh/id_ed25519
            Capability: 受限於 CWD（與 Claude Code 自身沙箱一致）
```

---

## 7. 方法論有效性論證

### 7.1 本協議對 Agent 儲存邊界限制的有效性

| 攻擊向量 | Agent 自身防禦 | 本協議防禦 | 有效性 |
|----------|:------------:|:---------:|:------:|
| Agent 繞過 client 沙箱直接操作 | ⚠️ prompt injection 可繞過 | ✅ daemon 側強制 | **有效** |
| Agent 讀取敏感 dotfiles | ❌（Cursor 沙箱退化） | ✅ denied_paths | **有效** |
| Agent 執行危險指令 | ⚠️ 依賴 client 審批 | ✅ allowed_commands | **有效** |
| Agent 長期佔用資源 | ❌ | ✅ session timeout | **有效** |
| Agent 跨使用者提權 | ✅ OS 身份隔離 | ✅ + capability 限制 | **雙重有效** |
| Agent 竊取 SSH key | ⚠️ 依賴 OS 權限 | 🔶 key 存取獨立於 daemon | **外部風險** |

### 7.2 方法論結論

本協議在 **daemon 側** 提供了獨立於任何 Agent client 安全機制的 **第四層邊界限制**。由於所有限制在 daemon 側強制執行，即使 Agent client 被完全妥協（prompt injection、dangerously-skip-permissions、沙箱繞過），daemon 仍能有效：

1. **限制可存取路徑** — denied_paths 阻擋敏感目錄
2. **限制可執行指令** — allowed_commands 白名單
3. **限制存取時間** — session 自動過期
4. **審計所有操作** — 不可繞過的 audit log

這構成了方法論上的「**零信任 Agent 執行模型**」— 不信任任何 Agent 自稱的安全保證，僅信任 daemon 側強制執行的 capability 限制。
