# SakiAgentSSH Architecture Status Report

> **Created**: 2026-02-28 06:22 (UTC+8)
> **Version**: 1.0
> **Status**: ✅ Implemented
> **Scale**: Micro (approx. 1,608 LOC) [1]

[🇹🇼 繁體中文](ARCHITECTURE.md) | [🇯🇵 日本語](ARCHITECTURE_ja.md) | [🇺🇸 English](ARCHITECTURE_en.md)

## 1. Current Architecture

SakiAgentSSH is a gRPC-based cross-machine Agent execution bridge. Adopting a Client-Daemon architecture, it replaces legacy SSH tunnels and is designed specifically for high-frequency, non-interactive Agent automation. It bypasses The Corporation's bloated protocols for direct node-to-node neural links.

```mermaid
graph TD
    A[Agent (Gemini CLI / Claude)] -->|CLI Command| B(saki-ssh-client)
    B -->|gRPC / HTTP2| C(saki-ssh-daemon)
    C -->|Spawn & Track| D[Local Shell / Processes]
    C -->|Stream Read/Write| E[Local File System]
```

### Directory Mapping
- `saki-ssh-daemon/`: The guardian process residing on the controlled node. Handles ACLs, request translation, process tracking, and I/O streaming.
- `saki-ssh-client/`: The command-line client. Responsible for issuing requests and forwarding Ctrl+C (cancel signals).
- `proto/`: gRPC protocol definitions (`sakissh.proto`).

## 2. Technical Implementation

- **Core Language**: Rust 2021 Edition
- **Networking**: `tonic` (v0.12), `prost` (v0.13) [2]
- **Asynchronous Runtime**: `tokio` (v1.0), `tokio-stream`
- **CLI Parsing**: `clap` (v4.4)
- **Other Dependencies**: `serde` (config), `ipnet` (ACL), `uuid` (execution identity)

## 3. Core Methods & Mechanisms

### 3.1 Execution Tracking Module (`saki-ssh-daemon/src/main.rs`)
- **Function**: Command streaming and lifecycle management.
- **Key Structures**: `TrackedProcess` and `MySsh::execute_stream`.
- **Logic Summary**: 
  1. Daemon receives an `ExecuteRequest`.
  2. Registers `tokio::process::Child` into an in-memory `RwLock<HashMap>` using a UUID.
  3. Splits stdout/stderr into two asynchronous tasks, returning them to the Client as a `StreamResponse`.
  4. Supports immediate process termination via `CancelRequest`.

### 3.2 ACL Security Module (`saki-ssh-daemon/src/main.rs`)
- **Function**: Access Control List (Vault-grade perimeter).
- **Key Function**: `check_acl`.
- **Logic Summary**: Parses the client's connecting IP based on `ipnet` and compares it against `allowed_cidrs`. If it fails the check, it immediately drops the connection with `Status::permission_denied`.

### 3.3 File Transfer Module (`proto/sakissh.proto` & `main.rs`)
- **Function**: gRPC streaming file transfer.
- **Key Definitions**: `FileUpload`, `FileDownload`, `FileChunk`.
- **Logic Summary**: Uses `Stream<FileChunk>`. The first packet transmits `FileMetadata` (containing filename and size), followed by `bytes data`. Supports resume operations via the `offset` parameter.

## 4. Original Expectations vs. Reality

- ✅ **Non-blocking Interaction**: Replaced the SSH tunnel, avoiding UI freezes during long builds.
- ✅ **Large File Support**: Implemented chunked streaming to prevent OOM errors.
- ✅ **POSIX Signal Translation**: Supports forwarding Client-side Ctrl+C as a Cancel RPC to terminate remote processes.

## 5. Evolutionary Timeline

- **v0.1**: Established basic `tonic` gRPC bidirectional communication.
- **v0.2**: Introduced file upload/download (`FileChunk`). Implemented Vault-grade ACL control, replacing the initial open-door design.

---
**Sources**:
[1] `find . -name "*.rs" -o -name "*.proto" | xargs wc -l` statistics
[2] `saki-ssh-client/Cargo.toml` and `saki-ssh-daemon/Cargo.toml`