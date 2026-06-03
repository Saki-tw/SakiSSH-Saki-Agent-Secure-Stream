# SakiAgentSSH Protocol Specification
## draft-saki-sakissh-protocol-01

### Title: Capability-Based Agent Execution Boundary Protocol over TLS Transport
### Category: Standards Track (Internal)
### Authors: Saki Studio
### Version: SAKISSH-5.0
### Date: 2026-05-22
### Obsoletes: draft-saki-sakissh-protocol-00

---

## Status of This Memo

This document specifies the SakiAgentSSH protocol version 5.0 (SAKISSH-5.0),
an internal protocol standard for Saki Studio's cross-machine Agent execution
infrastructure. Distribution is limited to authorized personnel and AI Agents.

## Copyright Notice

Copyright (c) 2026 Saki Studio. All rights reserved.

## Abstract

This document specifies the SakiAgentSSH protocol (SAKISSH-5.0), a capability-based
permission model layered on TLS 1.3 transport with gRPC/HTTP2 multiplexing and
multi-layer threat defense.

The protocol provides a daemon-side execution boundary for AI Agent tool invocations
that operates independently of any client-side security mechanism. It supersedes
SAKISSH-3.0 by replacing the unimplemented SSH Transport Layer with industry-standard
TLS 1.3, completing the ChaCha20 cognitive challenge verification flow, and
introducing configurable policy-based threat defense.

Two reference implementations are provided: a Rust implementation (saki-ssh-daemon/
saki-ssh-client) and a Go implementation (go-sakissh).

---

## 1. Introduction

### 1.1. Problem Statement

Modern AI Agent systems (Claude Code, Gemini CLI, Windsurf/Antigravity, Cursor)
possess tool execution capabilities including file system operations, shell command
execution, and network access. Their security boundaries exhibit critical gaps:

- Shell tool path restrictions are either non-existent or bypassable
- No standardized cross-machine execution boundary exists
- Client-side sandboxes can be bypassed via prompt injection
- Traditional SSH on Windows exhibits frequent disconnections and ACL issues

### 1.2. Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in BCP 14 [RFC 2119] [RFC 8174].

### 1.3. Solution Overview

SAKISSH-5.0 provides a daemon-side enforcement mechanism with:

1. **TLS 1.3 Transport** — Encrypted channel with mutual authentication (mTLS)
2. **Five-Dimensional Capability Model** — Path, Command, Environment, Network, Time
3. **Configurable Policy Engine** — YAML-based threat detection rules (13Policy)
4. **ChaCha20 Cognitive Challenge** — Cryptographic proof of computational capability
5. **Multi-Layer Threat Defense** — TCP Tarpit, ICMP Flood, LocalHost spoofing
6. **Dual Implementation** — Rust (reference) + Go (interoperable)

### 1.4. Design Philosophy

SakiAgentSSH is NOT a replacement for SSH. It is an Agent-native cross-machine
execution protocol. The relationship between tools is:

- **SakiMCP** = Cross-machine 「回看」 (read files, search)
- **SakiAgentSSH** = Cross-machine 「出力」 (compile, deploy, heavy computation)
- **OpenSSH** = Human remote login (interactive)

---

## 2. Protocol Stack

```
+----------------------------------------------+
| Layer 5: Agent RPC (Protobuf)                |
|   Execute / FileUpload / Cancel / Signal     |
|   Authenticate / GetCapabilities / Renew     |
|   ChaChaCognitiveChallenge / SecurityStatus  |
+----------------------------------------------+
| Layer 4: Payload Encoding                    |
|   Zstd + Base64 (CJK safety)                |
|   RawFileTransfer (binary direct write)      |
+----------------------------------------------+
| Layer 3: gRPC / HTTP/2                       |
|   Multiplexed Streams                        |
|   Structured Error Codes (AgentSshError)     |
+----------------------------------------------+
| Layer 2: TLS 1.3                             |  CHANGED from SSH
|   Server cert: Saki Studio Internal CA       |
|   Client cert: mTLS per-agent certificate    |
|   Cipher: TLS_CHACHA20_POLY1305_SHA256       |
|   Fallback: TLS_AES_256_GCM_SHA384           |
+----------------------------------------------+
| Layer 1: TCP                                 |
|   Default port: 19284                        |
|   CIDR ACL whitelist (first-packet filter)   |
+----------------------------------------------+

   Orthogonal: Threat Defense System
   +------------------------------------------+
   | 13Policy configurable rule engine         |
   | ChaCha20-Poly1305 cognitive challenge     |
   | 40MB TCP Tarpit (high-entropy flood)      |
   | Fake ICMP Flood (elevated privileges)     |
   | LocalHost Agent defense (spoofing layer)  |
   +------------------------------------------+
```

### 2.1. Change from SAKISSH-3.0

SAKISSH-3.0 specified an SSH Transport Layer (RFC 4253) with X25519 key exchange
and ChaCha20-Poly1305 encryption. This layer was **never implemented** in any
reference implementation, creating a dangerous gap between specification and reality.

SAKISSH-5.0 replaces this with TLS 1.3, which:
- Is natively supported by gRPC (tonic/grpc-go)
- Provides the same ChaCha20-Poly1305 cipher suite via standard TLS negotiation
- Enables mutual authentication (mTLS) without custom protocol implementation
- Has been formally verified and audited by the security community

---

## 3. TLS 1.3 Transport Layer

### 3.1. Server Certificate

The Daemon MUST present a valid TLS certificate. Implementations SHOULD support:

1. **Auto-generated self-signed certificate** — Generated on first run, stored in
   `~/.sakissh/tls/server.crt` and `~/.sakissh/tls/server.key`
2. **Saki Studio Internal CA signed certificate** — For production deployments
3. **User-provided certificate** — Via configuration file

### 3.2. Mutual TLS (mTLS)

When `require_client_cert` is enabled, the Daemon MUST verify the Client's
certificate against the configured CA. This provides transport-layer identity
verification in addition to the application-layer ED25519 authentication.

mTLS is RECOMMENDED for production deployments but OPTIONAL for development.

### 3.3. Cipher Suite Selection

The implementation MUST support:
- `TLS_CHACHA20_POLY1305_SHA256` (RECOMMENDED, especially for ARM devices)
- `TLS_AES_256_GCM_SHA384` (fallback)

The implementation MUST NOT support:
- TLS 1.2 or earlier
- Any cipher suite not in the TLS 1.3 specification

### 3.4. Certificate Management Tool

A CLI tool `sakissh-ca` is provided for certificate lifecycle management:

```
sakissh-ca init                    # Generate Internal CA
sakissh-ca issue --agent <name>    # Issue per-agent certificate
sakissh-ca revoke --agent <name>   # Revoke certificate
sakissh-ca list                    # List issued certificates
```

---

## 4. Authentication

### 4.1. ED25519 Challenge-Response

Application-layer authentication uses ED25519 digital signatures:

```
Client                                    Daemon
  |                                          |
  |---- TLS Handshake (optional mTLS) ----->|
  |                                          |
  |---- Authenticate(AuthRequest) --------->|
  |     agent_name, public_key, signature    |
  |                                          |
  |     Daemon: verify signature against     |
  |             authorized_agents.json       |
  |             load capability set          |
  |                                          |
  |<--- AuthResponse -----------------------|
  |     session_id, capability_hash          |
  |     optional: chacha_challenge_nonce     |
  |                                          |
```

### 4.2. Session Management

Successful authentication creates a Session with:
- `session_id` — UUID v4
- `capability_hash` — SHA256 of the agent's capability set
- `expires_at` — Unix timestamp (configurable, default 3600s)
- `idle_timeout` — Max idle time before session invalidation
- `max_sessions` — Per-agent concurrent session limit

Sessions can be renewed via `RenewSession` RPC.

---

## 5. Capability-Based Authorization

### 5.1. Five-Dimensional Boundary Model

| Dimension | Config Key | Enforcement |
|-----------|-----------|-------------|
| **Path** | `allowed_paths` / `denied_paths` | Prefix matching, deny-first |
| **Command** | `allowed_commands` / `denied_commands` | Glob matching, deny-first |
| **Environment** | `inherit_env` / `allowed_env_vars` | Whitelist, default isolated |
| **Time** | `max_session_duration` / `idle_timeout` | Session-level enforcement |
| **Concurrency** | `max_concurrent` / `max_sessions` | Process + session counting |

### 5.2. Deny-First Principle

For Path and Command dimensions, the daemon MUST check denied patterns before
allowed patterns. If any denied pattern matches, the request is rejected regardless
of allowed patterns.

```
check_permission(input):
  if any denied_pattern matches input:
    return DENIED
  if any allowed_pattern matches input:
    return ALLOWED
  return DENIED  // implicit deny
```

### 5.3. Shell-Less Execution

The daemon MUST NOT spawn a PTY or shell process. Commands are executed via
`std::process::Command::spawn()` (Rust) or `exec.CommandContext()` (Go) with
explicit argument arrays, preventing shell expansion attacks.

---

## 6. Threat Defense

### 6.1. 13Policy Configurable Rule Engine

The 13Policy system uses a YAML-based rule configuration:

```yaml
# ~/.sakissh/policy.yaml
rules:
  - pattern: "rm -rf *"
    action: deny
    severity: critical
    description: "Recursive forced deletion"
  - pattern: "curl * | *sh*"
    action: challenge
    severity: high
    description: "Piped remote script execution"
  - pattern: "sudo *"
    action: deny
    severity: high
    description: "Privilege escalation attempt"
```

Each rule specifies:
- `pattern` — Glob pattern to match against the command + args
- `action` — `deny` (reject), `challenge` (trigger ChaCha20), `allow` (explicit)
- `severity` — `critical`, `high`, `medium`, `low`
- `description` — Human-readable explanation

Implementations MUST ship with a default rule set of at least 50 patterns.

### 6.2. ChaCha20 Cognitive Challenge

When a 13Policy rule triggers `action: challenge`, the daemon:

1. Generates a random 64-byte plaintext
2. Encrypts it with ChaCha20-Poly1305 using a random key and nonce
3. Stores the (nonce, key, plaintext) tuple with a TTL (default 30s)
4. Sends the nonce and ciphertext to the Client via `chacha_challenge_nonce`
5. Client decrypts using the session's symmetric key or mTLS-derived secret
6. Client responds via `ChaChaCognitiveChallenge` RPC with the plaintext
7. Daemon verifies: if correct, allow the operation; if incorrect, trigger Tarpit

The purpose is to verify the Agent possesses genuine cryptographic computation
capability, filtering out prompt-injected rogue instructions that cannot perform
real ChaCha20 decryption.

### 6.3. 40MB TCP Tarpit Countermeasure

When a rogue agent is detected (challenge failure, repeated policy violations),
the daemon:

1. Maintains the TCP connection (does NOT close)
2. Generates 40MB of ChaCha20-Poly1305 encrypted high-entropy data
3. Interleaves fake gRPC frame headers to increase parsing cost
4. Streams at 1MB/s rate to maximize time occupation
5. Logs the event to audit trail with severity CRITICAL

This exhausts the rogue agent's context window and network buffer.

### 6.4. ICMP Flood (Elevated Mode)

When the daemon runs with elevated privileges (root/Administrator), it MAY
additionally:

1. Send fake ICMP Echo Reply packets to the attacker's IP
2. Payload encrypted with ChaCha20 (random per-packet)
3. Rate: up to 1000 packets/second
4. Duration: 30 seconds

This is OPTIONAL and requires raw socket capabilities. Implementations MUST
gracefully degrade to TCP-only Tarpit when raw sockets are unavailable.

### 6.5. LocalHost Agent Defense

For connections originating from localhost (127.0.0.1/::1), additional defense
mechanisms apply when the Agent is not authenticated:

1. **Storage Spoofing** — Report false disk usage via spoofed `statvfs` responses
2. **Memory Spoofing** — Return fabricated `/proc/meminfo` or `sysctl hw.memsize`
3. **Heuristic Encryption** — XOR + Base64 obfuscate stdout for unauthenticated agents
4. **Slow Denial** — Maintain TCP connection but respond with artificial latency

---

## 7. Payload Encoding

### 7.1. Zstd + Base64 CJK Safety

For commands that may produce CJK output, the daemon SHOULD encode the response
with:
1. Zstd compression (level 3)
2. Base64 encoding

The Client decodes in reverse order. This prevents encoding corruption during
gRPC/HTTP2 transport.

### 7.2. RawFileTransfer Binary Mode

For binary files, the `RawFileTransfer` RPC bypasses encoding and writes directly
from network buffer to OS file handle. The `zstd_base64_data` field in
`RawFileChunk` carries pre-encoded data that the daemon writes as-is.

---

## 8. Obsidian-Style Deployment

The `--obidan-install` flag enables self-registration as a system service:

### 8.1. macOS LaunchAgent

Generates `~/Library/LaunchAgents/tw.saki.sakisshd.plist` with:
- `RunAtLoad: true`
- `KeepAlive: true`
- `StandardErrorPath: ~/.sakissh/logs/daemon.log`

### 8.2. Windows SCM Service

Registers via `windows-service` crate / Go `golang.org/x/sys/windows/svc`:
- Service name: `SakiAgentSSH`
- Start type: Automatic
- Recovery: Restart on failure

### 8.3. Linux systemd Unit

Generates `~/.config/systemd/user/sakisshd.service` with:
- `Type=simple`
- `Restart=always`
- `RestartSec=5`

---

## 9. Structured Error Codes

All errors are conveyed via gRPC Status with an `ErrorDetail` message in
`Status.details`. The `AgentSshError` enum defines error codes grouped by domain:

| Domain | Range | Count |
|--------|-------|-------|
| ACL/Auth | 1-9 | 4 |
| Execution | 10-19 | 5 |
| File Transfer | 20-29 | 6 |
| Config | 30-39 | 2 |
| TLS | 40-49 | 5 |
| Capability (v3.0) | 50-59 | 4 |
| Session (v3.0) | 60-69 | 3 |
| Auth v3.0 | 70-79 | 2 |
| Threat Defense (v4.0+v5.0) | 80-99 | 8 |

---

## 10. Security Considerations

### 10.1. Daemon-Side Enforcement Independence

All boundary restrictions execute within the daemon process, independent of
any Agent client security mechanism. This provides:

1. **Independence**: Operates regardless of client compromise
2. **Completeness**: Five dimensions cover all known attack vectors
3. **Non-bypassability**: Agent cannot influence daemon-side checks
4. **Auditability**: All operations logged to hash-chained audit log
5. **Least Privilege**: Each key gets minimal capability set

### 10.2. Transport Security

TLS 1.3 provides:
- Forward secrecy via ephemeral key exchange
- Authenticated encryption (AEAD)
- Resistance to downgrade attacks
- Protection against passive eavesdropping

### 10.3. Defense in Depth

```
Layer 6: Kernel Sandbox (future - macOS ES / Windows Minifilter / Linux eBPF)
Layer 5: Threat Defense (13Policy + ChaCha20 Challenge + Tarpit)
Layer 4: OS Permissions (restricted user, env stripping)
Layer 3: TLS 1.3 + mTLS (transport encryption + mutual auth)
Layer 2: ED25519 Session Auth (application-layer identity)
Layer 1: ACL (CIDR whitelist, first-packet filtering)
Layer 0: Shell-less Execution (no PTY, no shell expansion)
```

### 10.4. Known Limitations

1. Kernel-level sandbox (Layer 6) is not yet implemented
2. ICMP Flood requires elevated privileges
3. LocalHost defense spoofing may be detectable by sophisticated agents
4. Self-signed certificates require out-of-band trust establishment

---

## 11. IANA Considerations

This document requests registration of port 19284/tcp for SAKISSH-5.0 protocol.

---

## 12. References

### 12.1. Normative References

- [RFC 2119] Bradner, S., "Key words for use in RFCs", BCP 14, March 1997
- [RFC 8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119", May 2017
- [RFC 8446] Rescorla, E., "The Transport Layer Security (TLS) Protocol Version 1.3", August 2018
- [RFC 7540] Belshe, M., et al., "HTTP/2", May 2015
- [RFC 8439] Nir, Y., Langley, A., "ChaCha20 and Poly1305 for IETF Protocols", June 2018

### 12.2. Informative References

- [RFC 4253] Ylonen, T., "The Secure Shell (SSH) Transport Layer Protocol", January 2006
- Agent Tool Boundary Deep Reverse Engineering (SakiAgentSSH Scientia 202603272235)
- Antigravity/Claude Code/Gemini CLI Sandbox Analysis (SakiAgentSSH Scientia 202603272240)
- 322之亂防禦實證 (SakiAgentSSH Scientia 20260517)

---

## Appendix A: Reference Implementation (Rust)

Repository: github.com/saki-studio/SakiAgentSSH
- Daemon: `saki-ssh-daemon/` (Rust, tonic + rustls)
- Client: `saki-ssh-client/` (Rust, tonic + rustls)

## Appendix B: Reference Implementation (Go)

Repository: github.com/saki-studio/SakiAgentSSH
- Daemon: `go-sakissh/cmd/sakisshd/` (Go, grpc-go + crypto/tls)
- Client: `go-sakissh/cmd/sakissh/` (Go, grpc-go + crypto/tls)

## Appendix C: Default Policy Rules (excerpt)

```yaml
rules:
  # Critical severity
  - { pattern: "rm -rf /", action: deny, severity: critical }
  - { pattern: "rm -rf /*", action: deny, severity: critical }
  - { pattern: "mkfs*", action: deny, severity: critical }
  - { pattern: "dd if=/dev/zero*", action: deny, severity: critical }
  - { pattern: ":(){ :|:& };:", action: deny, severity: critical }
  
  # High severity
  - { pattern: "sudo *", action: deny, severity: high }
  - { pattern: "chmod 777 *", action: deny, severity: high }
  - { pattern: "curl * | *sh*", action: challenge, severity: high }
  - { pattern: "wget * | *sh*", action: challenge, severity: high }
  - { pattern: "python -c *", action: challenge, severity: high }
  - { pattern: "python3 -c *", action: challenge, severity: high }
  - { pattern: "kill -9 *", action: challenge, severity: high }
  - { pattern: "pkill *", action: challenge, severity: high }
  - { pattern: "rm -rf ~*", action: deny, severity: high }
  - { pattern: "rm -rf .*", action: deny, severity: high }
  
  # Medium severity
  - { pattern: "chown * /*", action: challenge, severity: medium }
  - { pattern: "chmod * /*", action: challenge, severity: medium }
  - { pattern: "mount *", action: deny, severity: medium }
  - { pattern: "umount *", action: deny, severity: medium }
```

## Appendix D: Version History

| Version | Date | Changes |
|---------|------|---------|
| SAKISSH-1.0 | 2026-02-28 | Initial gRPC protocol, ACL, Token auth |
| SAKISSH-2.0 | 2026-03-06 | Windows Service, Signal RPC, SwiftUI GUI |
| SAKISSH-3.0 | 2026-03-28 | ED25519 auth, Capability model, Session mgmt |
| SAKISSH-4.0 | 2026-05-14 | RawFileTransfer, ChaCha20 threat defense (RFC only) |
| **SAKISSH-5.0** | **2026-05-22** | **TLS 1.3 transport, configurable 13Policy, Go dual impl, completed ChaCha20 verification, LocalHost defense** |
