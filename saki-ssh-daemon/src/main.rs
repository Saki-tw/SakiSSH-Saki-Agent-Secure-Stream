mod config;
mod auth;
mod audit;
mod capability;
mod challenge_store;
mod codec;
mod env_injector;
mod kernel_bridge;
mod localhost_defense;
mod policy;
mod quota;
mod session;
mod snapshot;
mod tarpit;
mod threat_defense;
mod v6_integration;
mod watchdog;
mod branch_mgr;

use config::DaemonConfig;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status, Streaming};
use tracing::{info, warn};

use auth::{AuthenticatedAgent, AgentAuthenticator, load_authorized_agents};
use audit::AuditLogger;
use challenge_store::ChallengeStore;
use policy::Policy13;
use quota::ResourceQuotaManager;
use session::SessionManager;

// ============================================================
// SASS 錯誤型別
// ============================================================

#[derive(Debug)]
pub enum AgentSshError {
    Auth(String),
    Capability(String),
    Session(String),
    Internal(String),
}

impl std::fmt::Display for AgentSshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentSshError::Auth(msg) => write!(f, "Auth error: {}", msg),
            AgentSshError::Capability(msg) => write!(f, "Capability error: {}", msg),
            AgentSshError::Session(msg) => write!(f, "Session error: {}", msg),
            AgentSshError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl From<AgentSshError> for Status {
    fn from(e: AgentSshError) -> Self {
        match e {
            AgentSshError::Auth(msg) => Status::unauthenticated(msg),
            AgentSshError::Capability(msg) => Status::permission_denied(msg),
            AgentSshError::Session(msg) => Status::not_found(msg),
            AgentSshError::Internal(msg) => Status::internal(msg),
        }
    }
}

#[macro_export]
macro_rules! agentssh_error {
    ($variant:ident, $($arg:tt)*) => {
        AgentSshError::$variant(format!($($arg)*))
    };
}

pub mod sakissh {
    tonic::include_proto!("sakissh");
}

use sakissh::saki_ssh_server::{SakiSsh, SakiSshServer};
use sakissh::{
    AuthRequest, AuthResponse, CancelRequest, CancelResponse,
    ChallengeRequest, ChallengeResponse, ExecuteRequest, ExecuteResponse,
    FileChunk, FileDownloadRequest, FileTransferResponse, PingRequest,
    PingResponse, PosixSignal, SecurityStatusRequest, SecurityStatusResponse,
    SignalRequest, SignalResponse, StreamResponse,
};

// ============================================================
// CIDR ACL 攔截器
// ============================================================

#[allow(dead_code)]
pub fn check_acl(
    remote_addr: Option<SocketAddr>,
    allowed_cidrs: &[ipnet::IpNet],
) -> Result<(), Status> {
    if allowed_cidrs.is_empty() {
        return Ok(()); // 空白名單 = 允許全部
    }
    let addr = remote_addr
        .ok_or_else(|| Status::unauthenticated("Cannot determine remote address"))?;
    let ip = addr.ip();
    for cidr in allowed_cidrs {
        if cidr.contains(&ip) {
            return Ok(());
        }
    }
    warn!("ACL rejected connection from {}", addr);
    Err(Status::permission_denied(format!(
        "IP {} not in allowed CIDR list",
        ip
    )))
}

// ============================================================
// 進程追蹤器
// ============================================================

struct TrackedProcess {
    child: tokio::process::Child,
}

type ProcessMap = Arc<RwLock<HashMap<String, TrackedProcess>>>;

// ============================================================
// SakiSSH Service 實作
// ============================================================

pub struct MySsh {
    config: DaemonConfig,
    processes: ProcessMap,
    start_time: chrono::DateTime<chrono::Utc>,
    parsed_cidrs: Vec<ipnet::IpNet>,
    // === SASS v1.4: 6-Response 狀態機所需的元件 ===
    pub session_mgr: SessionManager,
    pub quota_mgr: ResourceQuotaManager,
    pub policy13: Policy13,
    pub audit: AuditLogger,
    // === Phase 2: 認知挑戰與 Agent 認證 ===
    pub challenge_store: ChallengeStore,
    pub authenticator: AgentAuthenticator,
}

impl MySsh {
    fn new(config: DaemonConfig) -> Self {
        let parsed_cidrs: Vec<ipnet::IpNet> = config
            .acl
            .allowed_cidrs
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();

        if !parsed_cidrs.is_empty() {
            info!("ACL enabled with {} CIDR rules", parsed_cidrs.len());
        } else {
            info!("ACL disabled (empty allowed_cidrs = allow all)");
        }

        // 載入 13Policy
        let config_dir = DaemonConfig::default_path();
        let config_parent = config_dir.parent().unwrap_or(Path::new("."));
        let policy13 = Policy13::load_or_create(config_parent);

        // 初始化 Session Manager
        let session_mgr = SessionManager::new(100, 1024 * 1024); // 最大 100 sessions, 1MiB ring buffer

        // 初始化 Quota Manager
        let quota_mgr = ResourceQuotaManager::new(4, 64); // 每 identity 最多 4 PTY, 佇列深度 64

        // 初始化 Audit Logger（同步）
        let audit = AuditLogger::new(config_parent);

        // Phase 2: 初始化 ChaCha20 認知挑戰儲存器（TTL 60 秒）
        let challenge_store = ChallengeStore::new(60);
        // 啟動背景清理任務
        challenge_store.clone().spawn_cleanup_task();
        info!("Phase 2: ChallengeStore 初始化完成 (TTL=60s)");

        // Phase 2: 載入 Agent 認證配置
        let agents_config = load_authorized_agents(config_parent);
        let authenticator = AgentAuthenticator::new(agents_config);
        info!("Phase 2: AgentAuthenticator 初始化完成");

        Self {
            config,
            processes: Arc::new(RwLock::new(HashMap::new())),
            start_time: chrono::Utc::now(),
            parsed_cidrs,
            session_mgr,
            quota_mgr,
            policy13,
            audit,
            challenge_store,
            authenticator,
        }
    }

    fn build_command(&self, command: &str) -> (Command, String) {
        let shell_exe = self.config.shell.executable();
        let shell_args = self.config.shell.command_args();

        let mut cmd = Command::new(&shell_exe);
        for arg in &shell_args {
            cmd.arg(arg);
        }
        cmd.arg(command);

        let cmd_desc = format!("{} {} '{}'", shell_exe, shell_args.join(" "), command);
        (cmd, cmd_desc)
    }
}

impl std::fmt::Debug for MySsh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MySsh")
            .field("shell", &self.config.shell.r#type)
            .finish()
    }
}

impl MySsh {
    /// Stub: 檢查 gRPC metadata 中的 session token
    #[allow(dead_code)]
    fn check_token<T>(&self, _request: &Request<T>) -> Result<(), Status> {
        // Phase 2 實作：從 metadata 取出 token 驗證
        Ok(())
    }

    /// Stub: 檢查並取得已認證的 Agent Session
    #[allow(dead_code)]
    async fn check_session<T>(&self, _request: &Request<T>) -> Result<Option<Arc<AuthenticatedAgent>>, Status> {
        // Phase 2 實作：從 metadata 取出 session_id 查詢 SessionMap
        Ok(None)
    }
}

#[tonic::async_trait]
impl SakiSsh for MySsh {
    // --------------------------------------------------------
    // Execute (單次回傳)
    // --------------------------------------------------------
    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        let remote_addr = request.remote_addr();
        check_acl(remote_addr, &self.parsed_cidrs)?;

        let req = request.into_inner();
        let exec_id = if req.execution_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            req.execution_id.clone()
        };

        let (mut cmd, cmd_desc) = self.build_command(&req.command);
        cmd.args(&req.args);

        if !req.cwd.is_empty() {
            cmd.current_dir(&req.cwd);
        }
        for (k, v) in &req.env {
            cmd.env(k, v);
        }

        info!("[{}] Execute: {}", exec_id, cmd_desc);

        let output = cmd
            .output()
            .await
            .map_err(|e| Status::internal(format!("Failed to execute '{}': {}", cmd_desc, e)))?;

        Ok(Response::new(ExecuteResponse {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
            execution_id: exec_id,
        }))
    }

    // --------------------------------------------------------
    // ExecuteStream (串流回傳)
    // SASS v1.4: 預設走 v6_integration 狀態機 (6-Response Total Response Mapping)
    // --------------------------------------------------------
    type ExecuteStreamStream = ReceiverStream<Result<StreamResponse, Status>>;

    async fn execute_stream(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<Self::ExecuteStreamStream>, Status> {
        // SASS v1.4: 所有請求經過 6-Response 狀態機
        // R1(EXECUTE) / R2(CHALLENGE) / R3(THROTTLE) / R4(VI_SWAP) / R5(TARPIT) / R6(DROP)
        self.execute_stream_v6(request).await
    }

    // --------------------------------------------------------
    // Cancel (取消進程，無 timeout，取消即殺)
    // --------------------------------------------------------
    async fn cancel(
        &self,
        request: Request<CancelRequest>,
    ) -> Result<Response<CancelResponse>, Status> {
        let remote_addr = request.remote_addr();
        check_acl(remote_addr, &self.parsed_cidrs)?;

        let req = request.into_inner();
        info!("[{}] Cancel requested", req.execution_id);

        let mut map = self.processes.write().await;
        if let Some(mut tracked) = map.remove(&req.execution_id) {
            match tracked.child.kill().await {
                Ok(_) => Ok(Response::new(CancelResponse {
                    success: true,
                    message: format!("Process {} killed", req.execution_id),
                })),
                Err(e) => Ok(Response::new(CancelResponse {
                    success: false,
                    message: format!("Failed to kill {}: {}", req.execution_id, e),
                })),
            }
        } else {
            Ok(Response::new(CancelResponse {
                success: false,
                message: format!("No process found with id {}", req.execution_id),
            }))
        }
    }

    // --------------------------------------------------------
    // Signal (POSIX 信號轉譯)
    // --------------------------------------------------------
    async fn signal(
        &self,
        request: Request<SignalRequest>,
    ) -> Result<Response<SignalResponse>, Status> {
        let remote_addr = request.remote_addr();
        check_acl(remote_addr, &self.parsed_cidrs)?;

        let req = request.into_inner();
        let signal = PosixSignal::try_from(req.signal)
            .unwrap_or(PosixSignal::Sigterm);

        info!(
            "[{}] Signal {:?} requested",
            req.execution_id, signal
        );

        let mut map = self.processes.write().await;
        if let Some(tracked) = map.get_mut(&req.execution_id) {
            let result = match signal {
                PosixSignal::Sigint | PosixSignal::Sigterm | PosixSignal::Sigkill => {
                    // Windows 不區分 SIGINT/SIGTERM/SIGKILL，統一 kill
                    // Unix 上可以 signal(2) 細分, 但 tokio::process::Child 僅提供 kill()
                    tracked.child.kill().await
                }
                PosixSignal::Sighup => {
                    // HUP: 通常用於重載設定，在我們的場景直接 kill
                    tracked.child.kill().await
                }
            };

            // 如果是 SIGKILL，從追蹤表移除
            if matches!(signal, PosixSignal::Sigkill) {
                map.remove(&req.execution_id);
            }

            match result {
                Ok(_) => Ok(Response::new(SignalResponse {
                    success: true,
                    message: format!("Signal {:?} sent to {}", signal, req.execution_id),
                })),
                Err(e) => Ok(Response::new(SignalResponse {
                    success: false,
                    message: format!("Failed: {}", e),
                })),
            }
        } else {
            Ok(Response::new(SignalResponse {
                success: false,
                message: format!("No process found with id {}", req.execution_id),
            }))
        }
    }

    // --------------------------------------------------------
    // Ping (連線自檢)
    // --------------------------------------------------------
    async fn ping(
        &self,
        request: Request<PingRequest>,
    ) -> Result<Response<PingResponse>, Status> {
        let remote_addr = request.remote_addr();
        check_acl(remote_addr, &self.parsed_cidrs)?;

        let _req = request.into_inner();
        let uptime = chrono::Utc::now()
            .signed_duration_since(self.start_time)
            .num_seconds() as u64;

        let active = self.processes.read().await.len() as u32;

        Ok(Response::new(PingResponse {
            daemon_version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: uptime,
            shell_type: self.config.shell.r#type.clone(),
            shell_path: self.config.shell.executable(),
            os: std::env::consts::OS.to_string(),
            active_processes: active,
        }))
    }

    // --------------------------------------------------------
    // FileUpload (Client → Daemon 串流上傳)
    // --------------------------------------------------------
    async fn file_upload(
        &self,
        request: Request<Streaming<FileChunk>>,
    ) -> Result<Response<FileTransferResponse>, Status> {
        let remote_addr = request.remote_addr();
        check_acl(remote_addr, &self.parsed_cidrs)?;

        let mut stream = request.into_inner();
        let mut file: Option<tokio::fs::File> = None;
        let mut bytes_written: u64 = 0;
        let mut remote_path = String::new();

        while let Some(chunk) = tokio_stream::StreamExt::next(&mut stream).await {
            let chunk = chunk?;
            match chunk.payload {
                Some(sakissh::file_chunk::Payload::Metadata(meta)) => {
                    remote_path = meta.remote_path.clone();
                    info!("FileUpload: {} (size: {})", remote_path, meta.total_size);

                    // 安全檢查：路徑白名單
                    if !self.config.file_transfer.allowed_paths.is_empty() {
                        let allowed = self
                            .config
                            .file_transfer
                            .allowed_paths
                            .iter()
                            .any(|p| remote_path.starts_with(p));
                        if !allowed {
                            return Err(Status::permission_denied(format!(
                                "Path {} not in allowed paths",
                                remote_path
                            )));
                        }
                    }

                    let path = Path::new(&remote_path);
                    if let Some(parent) = path.parent() {
                        tokio::fs::create_dir_all(parent).await.map_err(|e| {
                            Status::internal(format!("Failed to create directory: {}", e))
                        })?;
                    }

                    let f = if meta.offset > 0 {
                        let f = tokio::fs::OpenOptions::new()
                            .write(true)
                            .open(path)
                            .await
                            .map_err(|e| {
                                Status::internal(format!("Failed to open file: {}", e))
                            })?;
                        f.set_len(meta.offset).await.map_err(|e| {
                            Status::internal(format!("Failed to seek: {}", e))
                        })?;
                        f
                    } else {
                        tokio::fs::File::create(path).await.map_err(|e| {
                            Status::internal(format!("Failed to create file: {}", e))
                        })?
                    };
                    file = Some(f);
                }
                Some(sakissh::file_chunk::Payload::Data(data)) => {
                    if let Some(ref mut f) = file {
                        f.write_all(&data).await.map_err(|e| {
                            Status::internal(format!("Failed to write: {}", e))
                        })?;
                        bytes_written += data.len() as u64;
                    } else {
                        return Err(Status::invalid_argument(
                            "Data chunk received before metadata",
                        ));
                    }
                }
                None => {}
            }
        }

        if let Some(mut f) = file {
            f.flush().await.map_err(|e| {
                Status::internal(format!("Failed to flush: {}", e))
            })?;
        }

        info!(
            "FileUpload complete: {} ({} bytes)",
            remote_path, bytes_written
        );

        Ok(Response::new(FileTransferResponse {
            success: true,
            message: format!("Uploaded {} bytes to {}", bytes_written, remote_path),
            bytes_written,
        }))
    }

    // --------------------------------------------------------
    // FileDownload (Daemon → Client 串流下載)
    // --------------------------------------------------------
    type FileDownloadStream = ReceiverStream<Result<FileChunk, Status>>;

    async fn file_download(
        &self,
        request: Request<FileDownloadRequest>,
    ) -> Result<Response<Self::FileDownloadStream>, Status> {
        let remote_addr = request.remote_addr();
        check_acl(remote_addr, &self.parsed_cidrs)?;

        let req = request.into_inner();
        let remote_path = req.remote_path.clone();

        info!("FileDownload: {} (offset: {})", remote_path, req.offset);

        // 安全檢查
        if !self.config.file_transfer.allowed_paths.is_empty() {
            let allowed = self
                .config
                .file_transfer
                .allowed_paths
                .iter()
                .any(|p| remote_path.starts_with(p));
            if !allowed {
                return Err(Status::permission_denied(format!(
                    "Path {} not in allowed paths",
                    remote_path
                )));
            }
        }

        let metadata = tokio::fs::metadata(&remote_path).await.map_err(|e| {
            Status::not_found(format!("File not found: {}", e))
        })?;

        let total_size = metadata.len();
        let chunk_size = self.config.file_transfer.max_chunk_size as usize;
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        let offset = req.offset;

        tokio::spawn(async move {
            // 首先傳送 metadata
            let _ = tx
                .send(Ok(FileChunk {
                    payload: Some(sakissh::file_chunk::Payload::Metadata(
                        sakissh::FileMetadata {
                            remote_path: remote_path.clone(),
                            total_size,
                            offset,
                        },
                    )),
                }))
                .await;

            let file = match tokio::fs::File::open(&remote_path).await {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx
                        .send(Err(Status::internal(format!(
                            "Failed to open file: {}",
                            e
                        ))))
                        .await;
                    return;
                }
            };

            let mut reader = tokio::io::BufReader::new(file);

            // seek to offset
            if offset > 0 {
                use tokio::io::AsyncSeekExt;
                if let Err(e) = reader.seek(std::io::SeekFrom::Start(offset)).await {
                    let _ = tx
                        .send(Err(Status::internal(format!("Failed to seek: {}", e))))
                        .await;
                    return;
                }
            }

            let mut buf = vec![0u8; chunk_size];
            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx
                            .send(Ok(FileChunk {
                                payload: Some(sakissh::file_chunk::Payload::Data(
                                    buf[..n].to_vec(),
                                )),
                            }))
                            .await
                            .is_err()
                        {
                            break; // Client 端斷線
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Err(Status::internal(format!("Read error: {}", e))))
                            .await;
                        break;
                    }
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    // --------------------------------------------------------
    // v6: Authenticate (Phase 2: Ed25519 驗簽 + ChaCha20 挑戰生成)
    // --------------------------------------------------------
    async fn authenticate(
        &self,
        request: Request<AuthRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let remote_addr = request.remote_addr();
        check_acl(remote_addr, &self.parsed_cidrs)?;

        let req = request.into_inner();

        // Phase 2: Ed25519 簽章驗證 → 建立 session → 產生 ChaCha20 認知挑戰
        //
        // 流程：
        // 1. Client 以 Ed25519 私鑰簽署 nonce，連同公鑰一併送出
        // 2. Daemon 驗證簽章，查找對應的 CapabilitySet
        // 3. 建立 session，分配 session_id
        // 4. 產生 ChaCha20-Poly1305 認知挑戰，要求 Client 解密證明計算能力

        // 步驟 1-3: Ed25519 驗簽 + Session 建立
        let session = match self.authenticator.verify(
            &req.agent_name,
            &req.public_key,
            &req.signature,
            &req.nonce,
        ).await {
            Ok(session) => session,
            Err(e) => {
                warn!("Phase 2 Authenticate 失敗: {}", e);
                // 記錄審計日誌
                self.audit.log(crate::audit::AuditEvent::AuthFailure {
                    agent_name: req.agent_name.clone(),
                    reason: format!("{}", e),
                    remote_addr: remote_addr.map(|a| a.to_string()).unwrap_or_default(),
                });
                return Ok(Response::new(AuthResponse {
                    success: false,
                    session_id: String::new(),
                    capability_hash: String::new(),
                    chacha_challenge_nonce: Vec::new(),
                    chacha_challenge_ciphertext: Vec::new(),
                    message: format!("認證失敗: {}", e),
                }));
            }
        };

        // 步驟 4: 產生 ChaCha20-Poly1305 認知挑戰
        let (challenge_nonce, challenge_ciphertext) = self.challenge_store.generate_challenge().await;

        // 計算 capability hash 供 Client 校驗
        let cap_hash = AgentAuthenticator::capability_hash(&session.capabilities);
        let cap_hash_hex = hex::encode(&cap_hash);

        info!(
            "Phase 2 Authenticate 成功: agent='{}', session='{}', 等待認知挑戰回應",
            session.name, session.session_id
        );

        Ok(Response::new(AuthResponse {
            success: true,
            session_id: session.session_id,
            capability_hash: cap_hash_hex,
            chacha_challenge_nonce: challenge_nonce,
            chacha_challenge_ciphertext: challenge_ciphertext,
            message: "認證成功，請完成 ChaCha20 認知挑戰".to_string(),
        }))
    }

    // --------------------------------------------------------
    // v6: CognitiveChallenge (Phase 2: ChaCha20 驗證 + TLS EKM 綁定)
    // --------------------------------------------------------
    async fn cognitive_challenge(
        &self,
        request: Request<ChallengeRequest>,
    ) -> Result<Response<ChallengeResponse>, Status> {
        let remote_addr = request.remote_addr();
        check_acl(remote_addr, &self.parsed_cidrs)?;

        let req = request.into_inner();

        // Phase 2: 驗證 ChaCha20 解密結果 + TLS Exporter EKM HMAC 通道綁定
        //
        // 流程：
        // 1. Client 收到 Authenticate 回應中的 (nonce, ciphertext)
        // 2. Client 以共享的 ChaCha20 key 解密 ciphertext → 得到 plaintext
        // 3. Client 計算 HMAC-SHA256(ekm, plaintext) 作為通道綁定證據
        // 4. Daemon 驗證 plaintext（constant-time）+ EKM HMAC

        // 從 ChallengeRequest 中取得 Client 回傳的解密明文
        // Proto 定義: decrypted_plaintext = field 1, client_ekm_hmac = field 2
        let decrypted = &req.decrypted_plaintext;

        if decrypted.is_empty() {
            return Ok(Response::new(ChallengeResponse {
                passed: false,
                session_token: String::new(),
                message: "decrypted_plaintext 不可為空".to_string(),
            }));
        }

        // 步驟 1: 驗證 ChaCha20 解密結果（constant-time comparison via ChallengeStore）
        // ChallengeStore 使用 nonce 作為查找 key，但 ChallengeRequest 中沒有 nonce 欄位
        // 因此我們需要遍歷所有待驗證的挑戰 — 或者使用明文的前 12 bytes 作為辨識
        // 設計決策：使用 ChallengeStore 的全量搜索模式
        //
        // TODO: Proto 應新增 challenge_nonce 欄位到 ChallengeRequest，
        //       以避免 O(n) 搜索。目前以 try_verify_any 替代。
        let challenge_passed = self.challenge_store.try_verify_any(decrypted).await;

        if !challenge_passed {
            warn!("Phase 2 CognitiveChallenge 失敗: ChaCha20 解密結果不匹配");
            return Ok(Response::new(ChallengeResponse {
                passed: false,
                session_token: String::new(),
                message: "認知挑戰失敗：解密結果不正確或已過期".to_string(),
            }));
        }

        // 步驟 2: TLS Exporter EKM HMAC 通道綁定驗證 (Plugins 版本)
        // 若 Client 提供了 client_ekm_hmac，進行通道綁定驗證
        if !req.client_ekm_hmac.is_empty() {
            // 產生 stub EKM（未來替換為真實 TLS session EKM）
            // 使用 session UUID 的前 16 bytes 作為 context
            let mut session_uuid = [0u8; 16];
            // 以 remote_addr hash 作為 stub session context
            let addr_str = remote_addr.map(|a| a.to_string()).unwrap_or_default();
            let addr_hash = sha2::Digest::finalize(sha2::Digest::chain_update(
                sha2::Sha256::default(),
                addr_str.as_bytes(),
            ));
            session_uuid.copy_from_slice(&addr_hash[..16]);

            let ekm = threat_defense::derive_ekm_stub(&session_uuid);
            if !threat_defense::verify_ekm_hmac(&ekm, decrypted, &req.client_ekm_hmac) {
                warn!("Phase 2 CognitiveChallenge: TLS EKM HMAC 通道綁定失敗");
                return Ok(Response::new(ChallengeResponse {
                    passed: false,
                    session_token: String::new(),
                    message: "TLS 通道綁定失敗：EKM HMAC 不匹配".to_string(),
                }));
            }
            info!("Phase 2 CognitiveChallenge: TLS EKM HMAC 通道綁定成功");
        } else {
            info!("Phase 2 CognitiveChallenge: Client 未提供 EKM HMAC（跳過通道綁定）");
        }

        // 步驟 3: 產生 session token
        let session_token = uuid::Uuid::new_v4().to_string();
        info!("Phase 2 CognitiveChallenge 成功: session_token={}", session_token);

        Ok(Response::new(ChallengeResponse {
            passed: true,
            session_token,
            message: "認知挑戰通過，Session 已啟動".to_string(),
        }))
    }

    // --------------------------------------------------------
    // v6: SecurityStatus (安全狀態查詢 stub)
    // --------------------------------------------------------
    async fn security_status(
        &self,
        request: Request<SecurityStatusRequest>,
    ) -> Result<Response<SecurityStatusResponse>, Status> {
        let remote_addr = request.remote_addr();
        check_acl(remote_addr, &self.parsed_cidrs)?;

        let _req = request.into_inner();
        Ok(Response::new(SecurityStatusResponse {
            active_tarpits: 0,
            active_sessions: self.processes.read().await.len() as u32,
            blocked_ips: 0,
            policy_version: "13Policy-v1.0".to_string(),
        }))
    }
}

// ============================================================
// main
// ============================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // 載入配置
    let config_path = DaemonConfig::default_path();
    let config = DaemonConfig::load_or_create(&config_path)?;

    info!("Config loaded from {:?}", config_path);
    info!("Shell: {} ({})", config.shell.r#type, config.shell.executable());
    info!("Bind: {}", config.bind_address);

    let addr = config.bind_address.parse()?;
    let tls_config = config.tls.clone();
    let ssh = MySsh::new(config);

    info!("SakiAgentSSH Daemon v{} — Copyright (c) 2026 Saki Studio. All rights reserved.", env!("CARGO_PKG_VERSION"));

    // === Phase 1: TLS 傳輸層配置 ===
    match tls_config {
        Some(tls) => {
            use tonic::transport::{Certificate, Identity, ServerTlsConfig};

            info!("TLS 啟用: cert={}, key={}", tls.cert_path, tls.key_path);

            // 載入伺服器證書與私鑰
            let cert_pem = std::fs::read_to_string(&tls.cert_path)
                .map_err(|e| format!("無法讀取伺服器證書 {}: {}", tls.cert_path, e))?;
            let key_pem = std::fs::read_to_string(&tls.key_path)
                .map_err(|e| format!("無法讀取伺服器私鑰 {}: {}", tls.key_path, e))?;
            let server_identity = Identity::from_pem(cert_pem, key_pem);

            let mut server_tls = ServerTlsConfig::new().identity(server_identity);

            // mTLS: 若提供 CA 證書且要求客戶端證書，啟用雙向驗證
            if let Some(ref ca_path) = tls.ca_cert_path {
                if tls.require_client_cert {
                    let ca_pem = std::fs::read_to_string(ca_path)
                        .map_err(|e| format!("無法讀取 CA 證書 {}: {}", ca_path, e))?;
                    let ca_cert = Certificate::from_pem(ca_pem);
                    server_tls = server_tls.client_ca_root(ca_cert);
                    info!("mTLS 啟用: 要求客戶端證書驗證 (CA: {})", ca_path);
                } else {
                    info!("TLS 模式: 單向 TLS（CA 已配置但未要求客戶端證書）");
                }
            } else {
                info!("TLS 模式: 單向 TLS（無客戶端證書驗證）");
            }

            info!("Listening on {} (TLS)", addr);

            Server::builder()
                .tls_config(server_tls)?
                .add_service(SakiSshServer::new(ssh))
                .serve(addr)
                .await?;
        }
        None => {
            // 向後相容：無 TLS 配置時維持明文 gRPC
            info!("Listening on {} (plaintext)", addr);
            warn!("⚠ TLS 未啟用 — 通訊為明文 gRPC，僅建議用於受信任的內網環境");

            Server::builder()
                .add_service(SakiSshServer::new(ssh))
                .serve(addr)
                .await?;
        }
    }

    Ok(())
}
