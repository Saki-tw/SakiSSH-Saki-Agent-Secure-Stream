# SakiAgentSSH v3.0 協議實作方案設計

> **建立時間**：2026-03-27 22:45 (UTC+8)
> **版本**：Draft 0.1
> **對應**：Phase 4 協議實作方案
> **標籤**：#實作設計 #proto #daemon #client

---

## 1. Proto 擴展設計

### 1.1 擴展原則

- 向後相容：現有 7 RPC 不變動
- 新增 3 RPC + 4 message + 9 error 碼（50-69 區段）
- 保持 proto3 語法

### 1.2 新增 Proto 定義

```protobuf
// ============================================================
// SSH 認證與 Capability（v3.0 新增）
// ============================================================

// SSH 風格認證
rpc Authenticate(AuthRequest) returns (AuthResponse);
// Capability 自查
rpc GetCapabilities(CapabilityRequest) returns (CapabilityResponse);
// Session 續約
rpc RenewSession(SessionRenewRequest) returns (SessionRenewResponse);

message AuthRequest {
  string agent_name = 1;
  bytes public_key = 2;       // ED25519 public key (32 bytes)
  bytes signature = 3;        // ED25519 signature of nonce
  bytes nonce = 4;            // daemon 提供的 challenge nonce
  string client_version = 5;  // SAKISSH-3.0-...
}

message AuthResponse {
  bool success = 1;
  string session_id = 2;
  bytes capability_hash = 3;  // SHA256(capability_set)
  uint64 expires_at = 4;      // Unix timestamp
  string daemon_version = 5;
}

message CapabilityRequest {}

message CapabilityResponse {
  repeated string allowed_commands = 1;
  repeated string denied_commands = 2;
  repeated string allowed_paths = 3;
  repeated string denied_paths = 4;
  uint32 max_concurrent = 5;
  uint32 timeout_seconds = 6;
  uint64 max_file_size_bytes = 7;
}

message SessionRenewRequest {
  string session_id = 1;
}

message SessionRenewResponse {
  bool success = 1;
  uint64 new_expires_at = 2;
}

// 擴展 AgentSshError enum
// Capability (50-59)
// ERROR_CAPABILITY_DENIED = 50;
// ERROR_CAPABILITY_PATH_DENIED = 51;
// ERROR_CAPABILITY_CMD_DENIED = 52;
// ERROR_CAPABILITY_EXPIRED = 53;
// Session (60-69)
// ERROR_SESSION_NOT_FOUND = 60;
// ERROR_SESSION_EXPIRED = 61;
// ERROR_SESSION_LIMIT = 62;
// Auth (70-79)
// ERROR_AUTH_KEY_NOT_FOUND = 70;
// ERROR_AUTH_SIGNATURE_INVALID = 71;
```

---

## 2. Daemon 側權限管理模組設計

### 2.1 模組結構

```
saki-ssh-daemon/src/
├── main.rs           (現有 → 加入 auth interceptor 呼叫)
├── config.rs         (現有 → 擴展 CapabilityConfig)
├── auth.rs           (🔴 新增：ED25519 認證模組)
├── capability.rs     (🔴 新增：Capability 檢查模組)
├── session.rs        (🔴 新增：Session 管理模組)
└── audit.rs          (🔴 新增：審計日誌模組)
```

### 2.2 auth.rs 設計

```rust
// 關鍵結構
pub struct AuthenticatedAgent {
    pub name: String,
    pub public_key: ed25519_dalek::VerifyingKey,
    pub session_id: String,
    pub capabilities: CapabilitySet,
    pub expires_at: SystemTime,
}

pub struct AgentAuthenticator {
    authorized_agents: HashMap<[u8; 32], AgentConfig>,  // pubkey → config
    active_sessions: Arc<RwLock<HashMap<String, AuthenticatedAgent>>>,
}

impl AgentAuthenticator {
    pub fn verify(&self, req: &AuthRequest) -> Result<AuthResponse, Status> {
        // 1. 查找 public_key 對應的 AgentConfig
        // 2. 驗證 ED25519 signature of nonce
        // 3. 建立 session，設定 expires_at
        // 4. 返回 session_id + capability_hash
    }
}
```

### 2.3 capability.rs 設計

```rust
#[derive(Clone)]
pub struct CapabilitySet {
    pub allowed_commands: Vec<glob::Pattern>,
    pub denied_commands: Vec<glob::Pattern>,
    pub allowed_paths: Vec<PathBuf>,       // 前綴匹配
    pub denied_paths: Vec<PathBuf>,        // 前綴匹配（優先）
    pub max_concurrent: u32,
    pub timeout_seconds: u32,
    pub max_file_size: u64,
    pub inherit_env: bool,
    pub allowed_env_vars: Vec<String>,
}

impl CapabilitySet {
    pub fn check_command(&self, cmd: &str) -> Result<(), AgentSshError> {
        // deny 優先 → 再 check allow
    }

    pub fn check_path(&self, path: &Path) -> Result<(), AgentSshError> {
        // denied_paths 優先 → 再 check allowed_paths
    }
}
```

### 2.4 gRPC Interceptor 設計

```rust
// Tower interceptor — 在每個 RPC 前檢查 session + capability
pub fn capability_interceptor(
    sessions: Arc<RwLock<HashMap<String, AuthenticatedAgent>>>,
) -> impl Fn(Request<()>) -> Result<Request<()>, Status> {
    move |mut req: Request<()>| {
        // 1. 從 metadata 取得 x-agentssh-session-id
        // 2. 查找 session，檢查未過期
        // 3. 注入 capability_set 到 request extensions
        // 4. 各 RPC handler 內部再 check command/path
    }
}
```

---

## 3. Client 側認證模組設計

### 3.1 Key 管理

```
~/.sakissh/
├── id_ed25519          # 私鑰（chmod 600）
├── id_ed25519.pub      # 公鑰
└── known_daemons       # 已知 daemon host key（TOFU）
```

### 3.2 認證流程（Client 端）

```rust
pub async fn authenticate(
    client: &mut SakiSshClient<Channel>,
    key_path: &Path,
) -> Result<String, Error> {
    // 1. 讀取 ED25519 私鑰
    // 2. Ping daemon 取得 nonce
    // 3. 簽名 nonce
    // 4. 呼叫 Authenticate RPC
    // 5. 保存 session_id 供後續 RPC 使用
}
```

---

## 4. 跨 OS 依賴最小化方案

### 4.1 新增 Cargo 依賴

```toml
[dependencies]
# 現有依賴保持不變...

# v3.0 新增（全部純 Rust，零 C 依賴）
ed25519-dalek = { version = "2", features = ["std"] }
x25519-dalek = { version = "2", features = ["static_secrets"] }
chacha20poly1305 = "0.10"
hkdf = "0.12"
sha2 = "0.10"
glob = "0.3"
```

### 4.2 跨平台驗證點

| 平台 | Rust toolchain | 已知問題 |
|------|---------------|----------|
| macOS (aarch64) | stable | ✅ 無 |
| macOS (x86_64) | stable | ✅ 無 |
| Windows (x86_64) | stable-msvc | ⚠️ POSIX Signal 降級 |
| Linux (x86_64) | stable | ✅ 無 |
| Linux (aarch64) | stable | ⚠️ 需驗證 |

### 4.3 authorized_agents.json 位置規範

| OS | 路徑 |
|----|------|
| macOS | `~/.config/sakissh/authorized_agents.json` |
| Linux | `~/.config/sakissh/authorized_agents.json` |
| Windows | `%APPDATA%\SakiSSH\authorized_agents.json` |
