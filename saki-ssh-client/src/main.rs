use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use sakissh::saki_ssh_client::SakiSshClient;
use sakissh::{
    CancelRequest, ExecuteRequest, FileChunk, FileDownloadRequest, PingRequest, SignalRequest,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod sakissh {
    tonic::include_proto!("sakissh");
}

// ============================================================
// CLI 定義
// ============================================================

#[derive(Parser, Debug)]
#[command(
    name = "sakissh",
    version,
    about = "SakiAgentSSH Client — Agent-native remote execution\nCopyright (c) 2026 Saki Studio. All rights reserved."
)]
struct Cli {
    /// Remote address(es), comma-separated for failover
    /// e.g. "http://192.168.50.10:19284,http://100.64.0.1:19284"
    #[arg(short, long, env = "SAKISSH_ADDR")]
    addr: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Execute a remote command
    Exec {
        /// Current working directory on remote
        #[arg(short, long)]
        cwd: Option<String>,

        /// Environment variables (KEY=VALUE)
        #[arg(short, long)]
        env: Vec<String>,

        /// Command and arguments
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// Check daemon connectivity and status
    Ping,
    /// Copy files to/from remote
    Cp {
        /// Source path (prefix with "remote:" for remote paths)
        source: String,
        /// Destination path (prefix with "remote:" for remote paths)
        dest: String,
    },
    /// Cancel a running execution
    Cancel {
        /// Execution ID to cancel
        execution_id: String,
    },
    /// Send a POSIX signal to a running execution
    Signal {
        /// Execution ID
        execution_id: String,
        /// Signal name: SIGINT, SIGTERM, SIGKILL, SIGHUP
        #[arg(default_value = "SIGTERM")]
        signal: String,
    },
}

// ============================================================
// 多路徑 Failover 連線
// ============================================================

async fn connect_with_failover(addrs: &str) -> Result<SakiSshClient<tonic::transport::Channel>> {
    let addresses: Vec<&str> = addrs.split(',').map(|s| s.trim()).collect();

    for (i, addr) in addresses.iter().enumerate() {
        let addr = if !addr.starts_with("http") {
            format!("http://{}", addr)
        } else {
            addr.to_string()
        };

        match tokio::time::timeout(
            std::time::Duration::from_secs(3),
            SakiSshClient::connect(addr.clone()),
        )
        .await
        {
            Ok(Ok(client)) => {
                if i > 0 {
                    eprintln!("[sakissh] Connected via fallback path: {}", addr);
                }
                return Ok(client);
            }
            Ok(Err(e)) => {
                eprintln!("[sakissh] Failed to connect to {}: {}", addr, e);
            }
            Err(_) => {
                eprintln!("[sakissh] Connection timeout for {}", addr);
            }
        }
    }

    anyhow::bail!(
        "Failed to connect to any address: {}",
        addrs
    )
}

// ============================================================
// 指令執行 (含 Ctrl+C 轉發)
// ============================================================

async fn exec_command(
    client: &mut SakiSshClient<tonic::transport::Channel>,
    command: Vec<String>,
    cwd: Option<String>,
    env_vars: Vec<String>,
) -> Result<i32> {
    if command.is_empty() {
        anyhow::bail!("No command specified.");
    }

    let execution_id = uuid::Uuid::new_v4().to_string();

    let mut env_map = std::collections::HashMap::new();
    for e in &env_vars {
        if let Some((k, v)) = e.split_once('=') {
            env_map.insert(k.to_string(), v.to_string());
        }
    }

    let full_command = command.join(" ");

    let request = tonic::Request::new(ExecuteRequest {
        command: full_command,
        args: vec![],
        cwd: cwd.unwrap_or_default(),
        env: env_map,
        execution_id: execution_id.clone(),
    });

    let mut stream = client.execute_stream(request).await?.into_inner();

    // 設定 Ctrl+C handler → 發送 Cancel RPC
    let cancel_client = client.clone();
    let cancel_id = execution_id.clone();
    let cancelled = Arc::new(Mutex::new(false));
    let cancelled_clone = cancelled.clone();

    tokio::spawn(async move {
        let mut cancel_client = cancel_client;
        // 等待 ctrlc 信號 (在 spawn 中用 channel 接)
        let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);
        let tx_clone = tx;
        if ctrlc::set_handler(move || {
            let _ = tx_clone.try_send(());
        })
        .is_ok()
        {
            if rx.recv().await.is_some() {
                let mut locked = cancelled_clone.lock().await;
                *locked = true;
                eprintln!("\n[sakissh] Ctrl+C → sending Cancel for {}", cancel_id);
                let _ = cancel_client
                    .cancel(tonic::Request::new(CancelRequest {
                        execution_id: cancel_id,
                    }))
                    .await;
            }
        }
    });

    let mut exit_code = 0;
    while let Some(response) = tokio_stream::StreamExt::next(&mut stream).await {
        let res = response?;
        if !res.data.is_empty() {
            if res.source == 0 {
                use std::io::Write;
                std::io::stdout().write_all(&res.data)?;
                std::io::stdout().flush()?;
            } else {
                use std::io::Write;
                std::io::stderr().write_all(&res.data)?;
                std::io::stderr().flush()?;
            }
        }
        if let Some(code) = res.exit_code {
            exit_code = code;
            break;
        }
    }

    Ok(exit_code)
}

// ============================================================
// Ping
// ============================================================

async fn ping_daemon(client: &mut SakiSshClient<tonic::transport::Channel>) -> Result<()> {
    let response = client
        .ping(tonic::Request::new(PingRequest {}))
        .await?
        .into_inner();

    println!("SakiSSH Daemon Status:");
    println!("  Version:     {}", response.daemon_version);
    println!("  OS:          {}", response.os);
    println!("  Shell:       {} ({})", response.shell_type, response.shell_path);
    println!("  Uptime:      {}s", response.uptime_seconds);
    println!("  Active Procs: {}", response.active_processes);

    Ok(())
}

// ============================================================
// 檔案傳輸
// ============================================================

async fn copy_file(
    client: &mut SakiSshClient<tonic::transport::Channel>,
    source: &str,
    dest: &str,
) -> Result<()> {
    let src_remote = source.starts_with("remote:");
    let dst_remote = dest.starts_with("remote:");

    match (src_remote, dst_remote) {
        (false, true) => {
            // 本地 → 遠端 (Upload)
            let local_path = source;
            let remote_path = dest.strip_prefix("remote:").unwrap();
            upload_file(client, local_path, remote_path).await?;
        }
        (true, false) => {
            // 遠端 → 本地 (Download)
            let remote_path = source.strip_prefix("remote:").unwrap();
            let local_path = dest;
            download_file(client, remote_path, local_path).await?;
        }
        (false, false) => {
            anyhow::bail!("Both paths are local. Use 'cp' directly.");
        }
        (true, true) => {
            anyhow::bail!("Both paths are remote. Not supported.");
        }
    }

    Ok(())
}

async fn upload_file(
    client: &mut SakiSshClient<tonic::transport::Channel>,
    local_path: &str,
    remote_path: &str,
) -> Result<()> {
    let metadata = std::fs::metadata(local_path)
        .with_context(|| format!("Cannot read {}", local_path))?;
    let total_size = metadata.len();

    eprintln!("[sakissh] Uploading {} → {} ({} bytes)", local_path, remote_path, total_size);

    let (tx, rx) = tokio::sync::mpsc::channel(32);

    // 送 metadata chunk
    tx.send(FileChunk {
        payload: Some(sakissh::file_chunk::Payload::Metadata(
            sakissh::FileMetadata {
                remote_path: remote_path.to_string(),
                total_size,
                offset: 0,
            },
        )),
    })
    .await?;

    // 讀檔案並分塊送出
    let local_path = local_path.to_string();
    let tx_data = tx;
    tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        let file = match tokio::fs::File::open(&local_path).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[sakissh] Failed to open {}: {}", local_path, e);
                return;
            }
        };
        let mut reader = tokio::io::BufReader::new(file);
        let mut buf = vec![0u8; 65536];
        loop {
            match reader.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if tx_data
                        .send(FileChunk {
                            payload: Some(sakissh::file_chunk::Payload::Data(
                                buf[..n].to_vec(),
                            )),
                        })
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("[sakissh] Read error: {}", e);
                    break;
                }
            }
        }
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let response = client
        .file_upload(tonic::Request::new(stream))
        .await?
        .into_inner();

    if response.success {
        eprintln!(
            "[sakissh] Upload complete: {} bytes written",
            response.bytes_written
        );
    } else {
        anyhow::bail!("Upload failed: {}", response.message);
    }

    Ok(())
}

async fn download_file(
    client: &mut SakiSshClient<tonic::transport::Channel>,
    remote_path: &str,
    local_path: &str,
) -> Result<()> {
    eprintln!("[sakissh] Downloading {} → {}", remote_path, local_path);

    let mut stream = client
        .file_download(tonic::Request::new(FileDownloadRequest {
            remote_path: remote_path.to_string(),
            offset: 0,
        }))
        .await?
        .into_inner();

    let mut file: Option<tokio::fs::File> = None;
    let mut bytes_received: u64 = 0;

    while let Some(chunk) = tokio_stream::StreamExt::next(&mut stream).await {
        let chunk = chunk?;
        match chunk.payload {
            Some(sakissh::file_chunk::Payload::Metadata(meta)) => {
                eprintln!(
                    "[sakissh] Remote file size: {} bytes",
                    meta.total_size
                );
                let path = std::path::Path::new(local_path);
                if let Some(parent) = path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                file = Some(tokio::fs::File::create(local_path).await?);
            }
            Some(sakissh::file_chunk::Payload::Data(data)) => {
                if let Some(ref mut f) = file {
                    use tokio::io::AsyncWriteExt;
                    f.write_all(&data).await?;
                    bytes_received += data.len() as u64;
                }
            }
            None => {}
        }
    }

    if let Some(mut f) = file {
        use tokio::io::AsyncWriteExt;
        f.flush().await?;
    }

    eprintln!(
        "[sakissh] Download complete: {} bytes received",
        bytes_received
    );

    Ok(())
}

// ============================================================
// main
// ============================================================

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut client = connect_with_failover(&cli.addr).await?;

    match cli.command {
        Commands::Exec { cwd, env, command } => {
            let exit_code = exec_command(&mut client, command, cwd, env).await?;
            std::process::exit(exit_code);
        }
        Commands::Ping => {
            ping_daemon(&mut client).await?;
        }
        Commands::Cp { source, dest } => {
            copy_file(&mut client, &source, &dest).await?;
        }
        Commands::Cancel { execution_id } => {
            let response = client
                .cancel(tonic::Request::new(CancelRequest { execution_id }))
                .await?
                .into_inner();
            println!("Cancel: success={}, message={}", response.success, response.message);
        }
        Commands::Signal {
            execution_id,
            signal,
        } => {
            let sig = match signal.to_uppercase().as_str() {
                "SIGINT" | "INT" | "2" => 0,
                "SIGTERM" | "TERM" | "15" => 1,
                "SIGKILL" | "KILL" | "9" => 2,
                "SIGHUP" | "HUP" | "1" => 3,
                _ => anyhow::bail!("Unknown signal: {}", signal),
            };
            let response = client
                .signal(tonic::Request::new(SignalRequest {
                    execution_id,
                    signal: sig,
                }))
                .await?
                .into_inner();
            println!(
                "Signal: success={}, message={}",
                response.success, response.message
            );
        }
    }

    Ok(())
}
