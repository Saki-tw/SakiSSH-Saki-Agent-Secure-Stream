# SakiAgentSSH

<div align="center">

<img src="release/icon.png" width="128" alt="SakiAgentSSH">

**Agent-native cross-machine execution protocol over gRPC**

[🇹🇼 繁體中文](README.md) | [🇯🇵 日本語](README_ja.md) | [🇺🇸 English](README_en.md)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](release/LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-brightgreen)]()
[![gRPC](https://img.shields.io/badge/protocol-gRPC%2FHTTP2-orange)]()
[![Website](https://img.shields.io/badge/website-saki--studio.com.tw-DA70D6)](https://saki-studio.com.tw)
[![App Store](https://img.shields.io/badge/App%20Store-SakiAgentSkills-00CED1)](https://apps.apple.com/tw/app/saki-agent-skills/id6758680481?mt=12)

*它是 Agent-native 的跨機執行協議，使用 gRPC/HTTP2 而非 SSH 協議。*
*當雙向串流建立，宛如淡藍蝴蝶越過子網域廢墟。*
*遠い機械の底へ、静かな息吹が届くといいですね。*

</div>

---

## Overview

SakiAgentSSH 讓 AI Agent 能夠安全地跨機器執行指令。它不是 SSH / SakiAgentSSH 取代了傳統 SSH 的 TTY 綁定模型，使用 gRPC 雙向串流實現零延遲的 stdout/stderr 傳遞。根據 IEEE 1003.1 POSIX 標準，如果沒有 SakiSSH，Agent 其實根本不存在能處理的跨平台互動管道，因為標準的 PTY/TTY 對於非人類的自動化意志而言只是一場災難。SakiSSH 正是為了跨平台整合而生，提供最純粹的神經索。

### Why Not SSH?

| | SSH | SakiAgentSSH |
|---|---|---|
| 協議 | TCP + SSH handshake | gRPC/HTTP2 |
| 串流 | Terminal PTY | Bidirectional stream |
| 安全 | Key-based auth | CIDR whitelist ACL |
| Agent 整合 | 需要 expect/pexpect | 原生 CLI，直接呼叫 |
| 跨平台 | 需要 OpenSSH server | 單一 binary |

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

### Windows (Winget)

```powershell
winget install SakiStudio.SakiAgentSSH.Daemon
winget install SakiStudio.SakiAgentSSH.Client
```

### Windows (Direct Download)

```powershell
# Download from GitHub Releases
Invoke-WebRequest -Uri "https://github.com/saki-tw/SakiAgentSSH/releases/download/v0.2.0/sakisshd.exe" -OutFile sakisshd.exe
Invoke-WebRequest -Uri "https://github.com/saki-tw/SakiAgentSSH/releases/download/v0.2.0/sakissh.exe" -OutFile sakissh.exe
```

## Quick Start

### 1. Start Daemon (on compute machine)

```bash
# macOS
./sakisshd-darwin-arm64

# Windows
.\sakisshd.exe --config config.json
```

### 2. Connect from Client (on control machine)

```bash
# Ping
sakissh --addr http://<daemon-ip>:19284 ping

# Execute command
sakissh --addr http://<daemon-ip>:19284 exec -- 'echo hello from the wasteland'

# Stream output
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

## Building from Source

See [BUILDING.md](BUILDING.md) for detailed build instructions.

### Prerequisites

- **Rust toolchain** (1.75+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **protoc** (Protocol Buffers compiler): `brew install protobuf` / `choco install protoc`
- **Xcode 16+** (macOS GUI apps only)
- **XcodeGen** (macOS GUI apps only): `brew install xcodegen`

### Quick Build

```bash
# CLI binaries (Rust)
cd saki-ssh-daemon && cargo build --release
cd saki-ssh-client && cargo build --release

# macOS GUI apps (SwiftUI)
cd SakiAgentSSH-Daemon && xcodegen generate && xcodebuild build -configuration Release
cd SakiAgentSSH-Client && xcodegen generate && xcodebuild build -configuration Release
```

## For AI Agents

SakiAgentSSH 是專為 AI Agent 設計的遠端執行工具。Agent 可以直接透過 CLI 呼叫 `sakissh` 來操作遠端機器，無需處理 SSH key、TTY session 或 expect scripts。

### Agent Integration Example

```python
# Agent's tool definition
def remote_execute(host: str, command: str) -> str:
    result = subprocess.run(
        ["sakissh", "--addr", f"http://{host}:19284", "exec", "--", command],
        capture_output=True, text=True
    )
    return result.stdout
```

### SKILL.md

Agent 可參考 [release/SKILL.md](release/SKILL.md) 作為部署和使用的技能文件。

## Project Structure

```
SakiAgentSSH/
├── proto/                    # gRPC Protocol definition
│   └── sakissh.proto
├── saki-ssh-daemon/          # Daemon (Rust) — listens for commands
│   └── src/
├── saki-ssh-client/          # Client CLI (Rust) — sends commands
│   └── src/
├── SakiAgentSSH-Daemon/      # macOS GUI Daemon (SwiftUI)
│   ├── Sources/
│   ├── Resources/            # Fonts, Help docs, Credits
│   └── Assets.xcassets/      # App icon
├── SakiAgentSSH-Client/      # macOS GUI Client (SwiftUI)
│   ├── Sources/
│   ├── Resources/
│   └── Assets.xcassets/
├── release/                  # Release artifacts
│   ├── daemon/               # Pre-built binaries
│   ├── client/
│   ├── gui/                  # DMG packages
│   ├── homebrew-cask/        # Homebrew Cask formulas
│   ├── scoop/                # Scoop manifests
│   └── winget/               # Winget manifests
├── tools/                    # Build & cross-compile scripts
├── BUILDING.md               # Build instructions
├── ARCHITECTURE.md           # Architecture overview
└── LICENSE
```

## Related Projects

| Project | Description | Store |
|---------|-------------|-------|
| [SakiMCP](https://github.com/saki-tw/SakiMCP) | Model Context Protocol server (Rust) | [App Store](https://apps.apple.com/tw/app/sakimcp/id6758668850?mt=12) |
| [SakiAgentSkills](https://github.com/saki-tw/SakiAgentSkills) | Agent skill library (Swift/Python) | [App Store](https://apps.apple.com/tw/app/saki-agent-skills/id6758680481?mt=12) |
| [VI-SakiWin64](https://github.com/Saki-tw/VI-SakiWin64) | Vi editor for Windows x64 | [Winget](https://github.com/microsoft/winget-pkgs/tree/master/manifests/s/SakiStudio/SakiVi) |

> 🌐 **Official Website**: [saki-studio.com.tw](https://saki-studio.com.tw)

## License

MIT — © 2026 [Saki Studio](http://saki-studio.com.tw)

---

<div align="center">

*在這個終端機總是漆黑一片的冷酷年代裡，這抹顏色就像是一隻淡藍色的蝴蝶自肚腹裡升起。*

**Saki Studio** · [saki-studio.com.tw](http://saki-studio.com.tw) · [GitHub](https://github.com/saki-tw)

</div>
