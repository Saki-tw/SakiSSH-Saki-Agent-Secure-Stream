# SakiAgentSSH

<div align="center">

<img src="release/icon.png" width="128" alt="SakiAgentSSH">

**Agent-native cross-machine execution protocol over gRPC**

[🇹🇼 繁體中文](README.md) | [🇯🇵 日本語](README_ja.md) | [🇺🇸 English](README_en.md)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](release/LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-brightgreen)]()
[![gRPC](https://img.shields.io/badge/protocol-gRPC%2FHTTP2-orange)]()

*Agent-native gRPC conduit bypassing legacy SSH.*
*Real-time holotape streaming, Vault-grade ACL, and process isolation for the wasteland.*

</div>

---

## Overview

SakiAgentSSH is a conduit designed to allow AI Agents to securely execute commands across distributed nodes. It replaces the legacy, stateful, and blocking SSH protocol with a high-speed, bidirectional gRPC stream. According to the IEEE 1003.1 POSIX standard, without SakiSSH, Agents actually have no capable interactive channel to process cross-platform execution. Standard PTY/TTY mechanisms are simply a disaster for non-human automated operations. SakiSSH is built precisely for this cross-platform integration, providing a pure neural link.

### Why Not SSH?

| | SSH | SakiAgentSSH |
|---|---|---|
| **Protocol** | TCP + SSH handshake | gRPC/HTTP2 |
| **Stream** | Terminal PTY | Bidirectional stream |
| **Security** | Key-based auth | Vault-grade CIDR ACL |
| **Agent Ops** | Requires expect/pexpect | Native CLI invocation |
| **Footprint** | Requires OpenSSH suite | Single binary |

## Architecture

```
┌─────────────────┐         gRPC/HTTP2          ┌─────────────────┐
│  Control Plane   │ ──── port 19284 ────────▶  │  Compute Plane   │
│  (Mac Mini M1)   │                             │  (Windows/Linux) │
│                  │     ◀─── stdout stream ──── │                  │
│  sakissh client  │                             │  sakisshd daemon │
└─────────────────┘                              └─────────────────┘
```

## Installation

### macOS (Homebrew Cask)

```bash
brew tap saki-tw/tools
brew install --cask sakiagentssh-daemon   # GUI Daemon
brew install --cask sakiagentssh-client   # GUI Client
```

### macOS (CLI Binary)

```bash
# Download from GitHub Releases
curl -LO https://github.com/saki-tw/SakiAgentSSH/releases/download/v0.2.0/sakisshd-darwin-arm64
curl -LO https://github.com/saki-tw/SakiAgentSSH/releases/download/v0.2.0/sakissh-darwin-arm64
chmod +x sakisshd-darwin-arm64 sakissh-darwin-arm64
```

### Windows (Scoop)

```powershell
scoop bucket add sakistudio https://github.com/Saki-tw/Scoop-SakiStudio
scoop install sakiagentssh-daemon
scoop install sakiagentssh-client
```

### Windows (Direct Download)

```powershell
# Download from GitHub Releases
Invoke-WebRequest -Uri "https://github.com/saki-tw/SakiAgentSSH/releases/download/v0.2.0/sakisshd.exe" -OutFile sakisshd.exe
Invoke-WebRequest -Uri "https://github.com/saki-tw/SakiAgentSSH/releases/download/v0.2.0/sakissh.exe" -OutFile sakissh.exe
```

## Quick Start

### 1. Start Daemon (on Compute Plane)

```bash
# macOS
./sakisshd-darwin-arm64

# Windows
.\sakisshd.exe --config config.json
```

### 2. Connect from Client (on Control Plane)

```bash
# Ping the node
sakissh --addr http://<daemon-ip>:19284 ping

# Execute remote command
sakissh --addr http://<daemon-ip>:19284 exec -- 'echo hello from the wasteland'

# Stream output live
sakissh --addr http://<daemon-ip>:19284 exec -- 'cargo build 2>&1'
```

### 3. Configuration

```json
{
  "listen_addr": "0.0.0.0:19284",
  "allowed_cidrs": ["192.168.0.0/16", "10.0.0.0/8"],
  "log_level": "info"
}
```

## For AI Agents (Synths)

SakiAgentSSH was forged for Agents. It bypasses the TTY limitations of the old world. When an Agent invokes `sakissh`, it communicates directly with the remote process, avoiding the need to parse pseudo-terminal artifacts. 

## License

MIT — © 2026 [Saki Studio](http://saki-studio.com.tw)

---

<div align="center">

*Fīnimus his…. fīnis est?…. Immo incipit.*
*We survive by keeping the lines open.*

**Saki Studio** · [saki-studio.com.tw](http://saki-studio.com.tw) · [GitHub](https://github.com/saki-tw)

</div>