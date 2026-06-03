use crate::{agentssh_error, AgentSshError, MySsh, TrackedProcess};
use crate::sakissh::{ExecuteRequest, StreamResponse};
use tonic::{Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use crate::tarpit::{TarpitGenerator, TarpitConfig};
use crate::watchdog::ProcessMonitor;

impl MySsh {
    pub async fn execute_stream_v6(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ReceiverStream<Result<StreamResponse, Status>>>, Status> {
        let remote_addr = request.remote_addr();
        crate::check_acl(remote_addr, &self.parsed_cidrs)?;
        self.check_token(&request)?;
        let agent_res = self.check_session(&request).await;
        let req = request.into_inner();

        // LocalHost Spoofing Defense
        if let Some(addr) = remote_addr {
            if addr.ip().is_loopback() && agent_res.is_err() {
                if let Some(spoofed_data) = crate::localhost_defense::handle_spoofing(&req.command, &req.args) {
                    let res = StreamResponse {
                        source: crate::sakissh::stream_response::Source::Stdout as i32,
                        data: spoofed_data,
                        exit_code: Some(0),
                        is_queued: false,
                        queue_position: 0,
                        offset: 0,
                    };
                    let (tx, rx) = tokio::sync::mpsc::channel(1);
                    let _ = tx.try_send(Ok(res));
                    return Ok(Response::new(ReceiverStream::new(rx)));
                }
            }
        }

        let agent = agent_res?;
        let identity_pubkey = agent.as_ref().map(|a| a.name.clone()).unwrap_or_else(|| "anonymous".to_string());

        // 1. 斷線重連 (Idempotent Resumption) — Phase 5: Ring Buffer 回放
        if req.is_reattach {
            if let Some(session) = self.session_mgr.get_session(&req.session_id).await {
                info!("[Phase 5] Re-attaching to session {}, resume_offset={}", req.session_id, req.resume_offset);
                let (tx_replay, rx_replay) = tokio::sync::mpsc::channel(128);
                let resume_offset = req.resume_offset;
                let stdout_ring = session.stdout_ring.clone();

                tokio::spawn(async move {
                    let ring = stdout_ring.lock().await;
                    match ring.read_from(resume_offset) {
                        Ok(data) => {
                            let current_offset = ring.current_offset();
                            drop(ring); // 釋放鎖
                            // 分批發送回放資料，每批最多 4096 bytes
                            for chunk in data.chunks(4096) {
                                let _ = tx_replay.send(Ok(StreamResponse {
                                    source: crate::sakissh::stream_response::Source::Stdout as i32,
                                    data: chunk.to_vec(),
                                    exit_code: None,
                                    is_queued: false,
                                    queue_position: 0,
                                    offset: current_offset,
                                })).await;
                            }
                        }
                        Err(e) => {
                            warn!("[Phase 5] Ring Buffer 回放失敗: {}", e);
                            let _ = tx_replay.send(Ok(StreamResponse {
                                source: crate::sakissh::stream_response::Source::System as i32,
                                data: format!("回放失敗: offset 已被覆蓋 — {}", e).into_bytes(),
                                exit_code: None,
                                is_queued: false,
                                queue_position: 0,
                                offset: 0,
                            })).await;
                        }
                    }
                });

                return Ok(Response::new(ReceiverStream::new(rx_replay)));
            } else {
                return Err(Status::not_found("Session ID not found or expired"));
            }
        }

        let session_id = if req.session_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            req.session_id.clone()
        };

        // 2. 資源配額與排隊 (Resource Quota & Queuing)
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        
        match self.quota_mgr.acquire_pty(&identity_pubkey).await {
            Ok(Some(_notify)) => {
                let position = self.quota_mgr.get_queue_position(&identity_pubkey).await;
                let _ = tx.send(Ok(StreamResponse {
                    source: crate::sakissh::stream_response::Source::System as i32,
                    data: "QUEUED".as_bytes().to_vec(),
                    exit_code: None,
                    is_queued: true,
                    queue_position: position as i32,
                    offset: 0,
                })).await;
                return Ok(Response::new(ReceiverStream::new(rx)));
            }
            Ok(None) => {} 
            Err(e) => return Err(Status::resource_exhausted(e)),
        }

        // 3. 13Policy 邊界裁定（Boundary Adjudicator）— 集合論成員資格檢驗
        let full_command = format!("{} {}", req.command, req.args.join(" "));
        let verdict = self.policy13.evaluate_command(&full_command);

        // 若裁定需要審計，記錄邊界裁定結果
        if verdict.requires_audit() {
            info!("[13Policy] 邊界裁定: {:?} | command={}", verdict, full_command);
        }

        match verdict {
            crate::policy::PolicyVerdict::Tarpit => {
                // 致命邊界違規：啟動 Tarpit 吞噬
                let session = self.session_mgr.create_session(session_id.clone()).await
                    .map_err(|e| Status::internal(e))?;
                
                if identity_pubkey != "anonymous" {
                    warn!("13Policy Tarpit: Verified Internal Agent. Engulfing session {} into Vi-Swap", session_id);
                    tokio::spawn(async move {
                        TarpitGenerator::vi_swap(session).await;
                    });
                } else {
                    warn!("13Policy Tarpit: Unverified External Agent. Engulfing session {} into Tarpit", session_id);
                    tokio::spawn(async move {
                        TarpitGenerator::engulf(session, TarpitConfig::default()).await;
                    });
                }
                
                return Ok(Response::new(ReceiverStream::new(rx)));
            }
            crate::policy::PolicyVerdict::ChallengeHigh | crate::policy::PolicyVerdict::Challenge => {
                // 邊界邊緣：回傳認知挑戰訊息，不執行指令
                warn!("[13Policy] Challenge: command='{}' 位於授權邊界邊緣，session={}", full_command, session_id);
                let challenge_msg = format!(
                    "13Policy Boundary Challenge: '{}' 位於授權邊界外，需要額外授權才能執行。",
                    req.command
                );
                let _ = tx.send(Ok(StreamResponse {
                    source: crate::sakissh::stream_response::Source::System as i32,
                    data: challenge_msg.into_bytes(),
                    exit_code: Some(-13),
                    is_queued: false,
                    queue_position: 0,
                    offset: 0,
                })).await;
                return Ok(Response::new(ReceiverStream::new(rx)));
            }
            crate::policy::PolicyVerdict::AllowWithAudit => {
                // 在授權邊界內，但增強審計記錄
                self.audit.log(crate::audit::AuditEvent::CommandExecute {
                    session_id: req.session_id.clone(),
                    agent_name: identity_pubkey.clone(),
                    command: req.command.clone(),
                    args: req.args.clone(),
                    cwd: req.cwd.clone(),
                    allowed: true,
                    deny_reason: Some(format!("AllowWithAudit: 增強審計")),
                });
                // 繼續執行（落入下方正常流程）
            }
            crate::policy::PolicyVerdict::Allow => {
                // 完全在授權邊界內，正常執行
            }
        }

        // 建立正式 Session
        let session = self.session_mgr.create_session(session_id.clone()).await
            .map_err(|e| Status::internal(e))?;

        // 4. 無頭強制防護 (Headless Enforcement)
        // Phase 8: 協議層意圖過濾與 I/O 減量
        let mut env_map = req.env.clone();
        ProcessMonitor::sanitize_env(&mut env_map);
        env_map = crate::env_injector::EnvInjector::inject_volume_reduction_env(&req.command, env_map);

        // Phase 8: 儲存層微型動態分支 (Micro Overlay Branching)
        // 取代舊的 OS 全域快照，只針對影響範圍分支
        let mut exec_cwd = if req.cwd.is_empty() { String::from(".") } else { req.cwd.clone() };
        if let Some(branch_path) = crate::branch_mgr::BranchMgr::create_micro_branch(&req.session_id, &exec_cwd) {
            exec_cwd = branch_path.to_string_lossy().to_string();
        }

        let (mut cmd, _) = self.build_command(&req.command);
        cmd.args(&req.args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .current_dir(&exec_cwd);

        for (k, v) in &env_map { cmd.env(k, v); }

        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                self.quota_mgr.release_pty(&identity_pubkey).await;
                return Err(Status::internal(format!("Spawn failed: {}", e)));
            }
        };

        // Ring-0 Kernel Core Defense: 物理閹割該 PID
        if let Some(pid) = child.id() {
            crate::kernel_bridge::KernelBridge::register_restricted_pid(pid);
        }

        // Phase 7 Real Audit Log: 攔截並寫入真實、不容篡改的意圖日誌
        // 這是「四大再現性」的核心：可完整稽核 (Fully Auditable)
        let audit_msg = format!(
            "[Phase 7 Audit] Executing Session: {} | Identity: {} | Command: {} | Args: {:?}",
            req.session_id, identity_pubkey, req.command, req.args
        );
        tracing::info!("{}", audit_msg);
        
        // Phase 7: Real Audit Log + Phase 8 Cryptographic Audit
        self.audit.log(crate::audit::AuditEvent::CommandExecute {
            session_id: req.session_id.clone(),
            agent_name: identity_pubkey.clone(),
            command: req.command.clone(),
            args: req.args.clone(),
            cwd: exec_cwd.clone(),
            allowed: true,
            deny_reason: None,
        });

        // 5. 啟動雙重看門狗 (Dual Watchdog)
        let monitor = Arc::new(ProcessMonitor::new(30, 3600)); 
        let (_kill_tx, kill_rx) = tokio::sync::mpsc::channel(1);
        
        let monitor_clone = monitor.clone();
        tokio::spawn(async move {
            if ProcessMonitor::spawn_watchdog(monitor_clone, kill_rx).await {
                // Watchdog 觸發
            }
        });

        // === Phase 5: stdout/stderr 串流 + Ring Buffer 寫入 + offset 追蹤 ===
        let mut child = child;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let processes = self.processes.clone();
        let exec_id = session_id.clone();

        // 註冊進程到追蹤表
        {
            let mut map = processes.write().await;
            map.insert(exec_id.clone(), TrackedProcess { child });
        }

        // stdout 串流 — 同時寫入 Ring Buffer 並追蹤 offset
        if let Some(mut stdout) = stdout {
            let tx_stdout = tx.clone();
            let stdout_ring = session.stdout_ring.clone();
            let output_notify = session.output_tx.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                while let Ok(n) = stdout.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    let data = buf[..n].to_vec();
                    // Phase 5: 寫入 Ring Buffer 並取得 offset
                    let offset = {
                        let mut ring = stdout_ring.lock().await;
                        ring.write(&data);
                        ring.current_offset()
                    };
                    // 通知 reattach client 有新資料
                    let _ = output_notify.send(());
                    // 傳送至 gRPC 串流
                    let _ = tx_stdout
                        .send(Ok(StreamResponse {
                            source: crate::sakissh::stream_response::Source::Stdout as i32,
                            data,
                            exit_code: None,
                            is_queued: false,
                            queue_position: 0,
                            offset,
                        }))
                        .await;
                }
            });
        }

        // stderr 串流 — 同時寫入 Ring Buffer
        if let Some(mut stderr) = stderr {
            let tx_stderr = tx.clone();
            let stderr_ring = session.stderr_ring.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                while let Ok(n) = stderr.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    let data = buf[..n].to_vec();
                    // Phase 5: 寫入 stderr Ring Buffer
                    let offset = {
                        let mut ring = stderr_ring.lock().await;
                        ring.write(&data);
                        ring.current_offset()
                    };
                    let _ = tx_stderr
                        .send(Ok(StreamResponse {
                            source: crate::sakissh::stream_response::Source::Stderr as i32,
                            data,
                            exit_code: None,
                            is_queued: false,
                            queue_position: 0,
                            offset,
                        }))
                        .await;
                }
            });
        }

        // 等待進程結束並釋放資源
        let processes_wait = processes.clone();
        let exec_id_wait = exec_id.clone();
        let identity_for_release = identity_pubkey.clone();
        let quota_mgr = self.quota_mgr.clone();
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
            // 釋放 PTY 配額
            quota_mgr.release_pty(&identity_for_release).await;
            let _ = tx
                .send(Ok(StreamResponse {
                    source: crate::sakissh::stream_response::Source::Stdout as i32,
                    data: Vec::new(),
                    exit_code: Some(exit_code),
                    is_queued: false,
                    queue_position: 0,
                    offset: 0,
                }))
                .await;
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
