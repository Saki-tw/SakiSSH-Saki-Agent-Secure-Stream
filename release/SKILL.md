---
name: sakiagentssh
description: Deploy, configure, and troubleshoot SakiAgentSSH — an Agent-native cross-machine execution protocol over gRPC. Use this skill when an agent needs to (1) deploy sakisshd on a new machine, (2) execute commands on a remote machine, (3) transfer files cross-machine, (4) diagnose connection issues, or (5) manage remote processes (cancel/signal). SakiAgentSSH replaces OpenSSH for agent-to-machine communication.
---

# SakiAgentSSH

SakiSSH is the bridge to connect your agent and other machine. It is an Agent-native cross-machine execution protocol using gRPC/HTTP2.

## Architecture

- **Daemon** (`sakisshd`): Runs on the target machine, listens on port 19284
- **Client** (`sakissh`): CLI for agents with shell access (optional — gRPC direct calls also work)
- **Proto** (`sakissh.proto`): The API contract — 7 RPCs

## Deployment

### Step 1: Transfer daemon to target machine

Determine target OS and copy the appropriate binary:

- macOS ARM64: `sakisshd-darwin-arm64`
- Windows x86_64: `sakisshd.exe`

Transfer via SCP, existing SSH, or any file transfer method available.

### Step 2: Create config.json

Create `config.json` in the same directory as the daemon binary:

```json
{
  "bind_address": "0.0.0.0:19284",
  "shell": {
    "type": "powershell",
    "path": null,
    "args": null
  },
  "acl": {
    "allowed_cidrs": [],
    "ed25519_public_keys": []
  },
  "file_transfer": {
    "allowed_paths": [],
    "max_chunk_size": 65536
  }
}
```

Platform-specific `shell.type` values:
- Windows: `"powershell"` (auto-detects pwsh 7, falls back to powershell.exe)
- macOS/Linux: `"bash"`

**Critical for Windows**: `bind_address` MUST be `"0.0.0.0:19284"` (not `[::0]`). Windows does not enable IPv6 dual-stack by default.

### Step 3: Open firewall (Windows only)

```powershell
New-NetFirewallRule -DisplayName 'SakiAgentSSH' -Direction Inbound -Action Allow -Protocol TCP -LocalPort 19284
```

### Step 4: Start daemon

```bash
# Foreground (testing)
./sakisshd

# Windows background
Start-Process -FilePath '.\sakisshd.exe' -WindowStyle Hidden

# Windows as service (production) — use install.ps1
```

### Step 5: Verify from client

```bash
sakissh --addr http://<target-ip>:19284 ping
```

Expected response includes: daemon_version, os, shell_type, shell_path, uptime_seconds, active_processes.

## Usage

### Execute commands

```bash
# Single command
sakissh --addr http://192.168.1.100:19284 exec -- 'echo hello'

# With working directory
sakissh --addr http://192.168.1.100:19284 exec --cwd /tmp -- 'ls -la'

# With environment variables
sakissh --addr http://192.168.1.100:19284 exec --env RUST_LOG=debug -- 'cargo build'
```

### File transfer

Use `remote:` prefix to indicate remote paths:

```bash
# Upload (local → remote)
sakissh --addr <addr> cp local_file.txt remote:/path/on/remote.txt

# Download (remote → local)
sakissh --addr <addr> cp remote:/path/on/remote.txt local_file.txt
```

### Multi-path failover

Comma-separated addresses with 3-second timeout per path:

```bash
sakissh --addr "http://192.168.1.100:19284,http://100.64.0.1:19284" ping
```

Agent logic: try LAN first, fall back to Tailscale/VPN.

### Process management

```bash
# Cancel a running execution
sakissh --addr <addr> cancel <execution_id>

# Send POSIX signal
sakissh --addr <addr> signal <execution_id> SIGTERM
```

The `execution_id` is returned in ExecuteStream responses. Ctrl+C during `exec` auto-sends Cancel RPC.

### Environment variable for address

```bash
export SAKISSH_ADDR="http://192.168.1.100:19284"
sakissh ping
```

## Troubleshooting

### Connection timeout

1. Verify daemon running: `ssh target "netstat -an | findstr 19284"` (Windows) or `lsof -i :19284` (macOS)
2. Check bind_address in config.json — must be `0.0.0.0:19284` on Windows
3. Check firewall: `Get-NetFirewallRule -DisplayName 'SakiAgentSSH'` (Windows)
4. Try Tailscale IP if LAN blocked

### "Broken pipe" error

Daemon is listening on IPv6 only (`[::]:19284`) but client connects via IPv4. Fix: change `bind_address` to `"0.0.0.0:19284"`.

### PowerShell encoding issues (CP950/Shift-JIS)

Expected on CJK Windows. Command output is byte-accurate; display encoding is client-side. Agent should process raw bytes, not decoded text.

### Daemon exits immediately

Check config.json for JSON syntax errors. Run daemon in foreground to see error output.
