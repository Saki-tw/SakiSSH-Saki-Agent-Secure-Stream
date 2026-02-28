mod config;

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

pub mod sakissh {
    tonic::include_proto!("sakissh");
}

use sakissh::saki_ssh_server::{SakiSsh, SakiSshServer};
use sakissh::{
    CancelRequest, CancelResponse, ExecuteRequest, ExecuteResponse, FileChunk,
    FileDownloadRequest, FileTransferResponse, PingRequest, PingResponse, PosixSignal,
    SignalRequest, SignalResponse, StreamResponse,
};

// ============================================================
// CIDR ACL 攔截器
// ============================================================

fn check_acl(
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

        Self {
            config,
            processes: Arc::new(RwLock::new(HashMap::new())),
            start_time: chrono::Utc::now(),
            parsed_cidrs,
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
    // --------------------------------------------------------
    type ExecuteStreamStream = ReceiverStream<Result<StreamResponse, Status>>;

    async fn execute_stream(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<Self::ExecuteStreamStream>, Status> {
        let remote_addr = request.remote_addr();
        check_acl(remote_addr, &self.parsed_cidrs)?;

        let req = request.into_inner();
        let exec_id = if req.execution_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            req.execution_id.clone()
        };

        let (mut cmd, cmd_desc) = self.build_command(&req.command);
        cmd.args(&req.args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if !req.cwd.is_empty() {
            cmd.current_dir(&req.cwd);
        }
        for (k, v) in &req.env {
            cmd.env(k, v);
        }

        info!("[{}] ExecuteStream: {}", exec_id, cmd_desc);

        let child = cmd
            .spawn()
            .map_err(|e| Status::internal(format!("Failed to spawn '{}': {}", cmd_desc, e)))?;

        let (tx, rx) = tokio::sync::mpsc::channel(128);
        let exec_id_clone = exec_id.clone();
        let processes = self.processes.clone();

        // 拆出 stdout/stderr，保留 child 用於追蹤
        let mut child = child;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // 註冊進程
        {
            let mut map = processes.write().await;
            map.insert(exec_id.clone(), TrackedProcess { child });
        }

        // stdout 串流
        if let Some(mut stdout) = stdout {
            let tx_stdout = tx.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                while let Ok(n) = stdout.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    let _ = tx_stdout
                        .send(Ok(StreamResponse {
                            source: 0, // STDOUT
                            data: buf[..n].to_vec(),
                            exit_code: None,
                        }))
                        .await;
                }
            });
        }

        // stderr 串流
        if let Some(mut stderr) = stderr {
            let tx_stderr = tx.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                while let Ok(n) = stderr.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    let _ = tx_stderr
                        .send(Ok(StreamResponse {
                            source: 1, // STDERR
                            data: buf[..n].to_vec(),
                            exit_code: None,
                        }))
                        .await;
                }
            });
        }

        // 等待進程結束
        let processes_wait = processes.clone();
        let exec_id_wait = exec_id_clone;
        tokio::spawn(async move {
            let exit_code = {
                let mut map = processes_wait.write().await;
                if let Some(tracked) = map.get_mut(&exec_id_wait) {
                    let status = tracked.child.wait().await;
                    let code = status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
                    map.remove(&exec_id_wait);
                    code
                } else {
                    -1
                }
            };

            let _ = tx
                .send(Ok(StreamResponse {
                    source: 0,
                    data: Vec::new(),
                    exit_code: Some(exit_code),
                }))
                .await;
        });

        Ok(Response::new(ReceiverStream::new(rx)))
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
    let ssh = MySsh::new(config);

    info!("SakiAgentSSH Daemon v{} — Copyright (c) 2026 Saki Studio. All rights reserved.", env!("CARGO_PKG_VERSION"));
    info!("Listening on {}", addr);

    Server::builder()
        .add_service(SakiSshServer::new(ssh))
        .serve(addr)
        .await?;

    Ok(())
}
