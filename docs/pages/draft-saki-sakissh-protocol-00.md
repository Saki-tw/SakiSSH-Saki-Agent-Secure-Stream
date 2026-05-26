# SakiAgentSSH Protocol Specification
## draft-saki-sakissh-protocol-00

### Title: Capability-Based Agent Execution Boundary Protocol over SSH Transport
### Category: Informational
### Authors: Saki Studio

---

## Abstract

This document specifies the SakiAgentSSH protocol (SAKISSH-3.0), a capability-based
permission model layered on SSH transport (RFC 4253) with gRPC/HTTP2 multiplexing.
The protocol provides a fourth-layer execution boundary for AI Agent tool invocations
that operates independently of any client-side security mechanism.

Two reference implementations are provided: a Rust implementation (saki-ssh-daemon)
and a [Go/Swift] implementation (saki-ssh-client-alt).

---

## 1. Introduction

### 1.1. Problem Statement

Modern AI Agent systems (Claude Code, Gemini CLI, Windsurf/Antigravity, Cursor)
possess tool execution capabilities including file system operations, shell command
execution, and network access. However, their security boundaries exhibit critical
gaps:

- Shell tool path restrictions are either non-existent or bypassable
- No standardized cross-machine execution boundary exists
- Client-side sandboxes can be bypassed via prompt injection

### 1.2. Solution Overview

SAKISSH-3.0 provides a daemon-side enforcement mechanism with five-dimensional
boundary control:

1. Path (allowed_paths / denied_paths prefix matching)
2. Command (allowed_commands glob matching with deny-priority)
3. Environment (isolated env with session metadata injection)
4. Network (indirect control via command restrictions)
5. Time (session expiry + idle timeout + rekey interval)

---

## 2. Protocol Stack

```
+--------------------------------------+
| Layer 5: Agent RPC (Protobuf)        |
|   Execute / FileUpload / Cancel etc. |
+--------------------------------------+
| Layer 4: gRPC / HTTP/2               |
|   Multiplexed Streams                |
+--------------------------------------+
| Layer 3: SSH Channel Authorization   |  NEW
|   Capability-Based Permission        |
|   Key Exchange + Session Key         |
+--------------------------------------+
| Layer 2: SSH Transport (RFC 4253)    |  NEW
|   Encryption + MAC + Compression     |
|   Binary Packet Protocol             |
+--------------------------------------+
| Layer 1: TCP                         |
|   Default port: 19284               |
+--------------------------------------+
```

---

## 3. SSH Transport Layer

### 3.1. Version Exchange

Upon connection establishment, both parties exchange version strings:

```
SAKISSH-3.0-{softwareversion} {comments}\r\n
```

The "SAKISSH" prefix distinguishes from standard SSH to prevent scanner misidentification.

### 3.2. Key Exchange

Fixed algorithm set (no negotiation for minimal dependency):
- Key Exchange: curve25519-sha256 (X25519 Diffie-Hellman)
- Host Key: ED25519
- Session Key Derivation: HKDF-SHA256
- Encryption: ChaCha20-Poly1305
- MAC: Poly1305 (bound to ChaCha20)
- Rekey: Every 1GB or 1 hour

### 3.3. Binary Packet Protocol

Per RFC 4253 Section 6:
```
uint32    packet_length
byte      padding_length
byte[n1]  payload
byte[n2]  random_padding
byte[m]   mac
```

---

## 4. Capability-Based Authorization

### 4.1. Capability Set

Each ED25519 public key is bound to a Capability Set stored in the daemon's
`authorized_agents.json`:

```json
{
  "agents": [{
    "name": "gemini-cli@m1-mac",
    "public_key": "ssh-ed25519 AAAA...",
    "capabilities": {
      "execute": {
        "allowed_commands": ["ls", "cat", "grep", "cargo"],
        "denied_commands": ["rm -rf", "sudo", "chmod 777"],
        "allowed_cwd": ["/home/user/project/"],
        "max_concurrent": 5,
        "timeout_seconds": 300
      },
      "file_transfer": {
        "allowed_paths": ["/home/user/project/"],
        "denied_paths": ["~/.ssh", "~/.aws", "~/.gnupg"],
        "max_file_size_mb": 100
      },
      "session": {
        "max_duration_seconds": 3600,
        "max_sessions": 3,
        "idle_timeout_seconds": 600
      }
    }
  }]
}
```

### 4.2. Authorization Flow

```
Client (Agent)                              Daemon
    |                                          |
    |---- TCP Connect (port 19284) ---------->|
    |                                          |
    |<--- SAKISSH-3.0 version string ---------|
    |---- SAKISSH-3.0 version string -------->|
    |                                          |
    |    [X25519 Key Exchange]                 |
    |    [derive session keys]                |
    |                                          |
    |---- SSH_MSG_NEWKEYS ------------------>|
    |<--- SSH_MSG_NEWKEYS --------------------|
    |                                          |
    |    [Encrypted channel established]       |
    |                                          |
    |---- UserAuth (ED25519 signature) ----->|
    |                                          |
    |    Daemon: lookup public_key             |
    |            -> load capability set        |
    |                                          |
    |<--- UserAuth SUCCESS -------------------|
    |     (capability_hash in response)        |
    |                                          |
    |    [SSH Channel -> gRPC HTTP/2]          |
    |                                          |
    |---- gRPC Execute("ls /tmp") ---------->|
    |    Daemon: check capability              |
    |      OK "ls" in allowed_commands         |
    |      OK "/tmp" under allowed_cwd         |
    |<--- gRPC ExecuteResponse ----------------|
```

---

## 5. Security Considerations

### 5.1. Daemon-Side Enforcement Independence

All boundary restrictions execute within the daemon process, independent of
any Agent client security mechanism. This provides:

1. Independence: Operates regardless of client compromise
2. Completeness: Five dimensions cover all known attack vectors
3. Non-bypassability: Agent cannot influence daemon-side checks
4. Auditability: All operations logged to tamper-resistant audit log
5. Principle of Least Privilege: Each key gets minimal capability set

### 5.2. Formal Proof of Effectiveness

Theorem: The capability-based permission model is methodologically effective
against any Agent with tool execution capability, regardless of client-side
security state.

Proof sketch: Since all restrictions are enforced in the daemon process space,
and the Agent communicates only through the defined RPC interface, the Agent
cannot directly manipulate the enforcement logic. Q.E.D.

---

## 6. IANA Considerations

This document requests registration of port 19284/tcp for SAKISSH-3.0 protocol.

---

## 7. References

### 7.1. Normative References

- RFC 4253: The Secure Shell (SSH) Transport Layer Protocol
- RFC 4254: The Secure Shell (SSH) Connection Protocol
- RFC 7540: Hypertext Transfer Protocol Version 2 (HTTP/2)

### 7.2. Informative References

- ReazonSpeech v2 Security Analysis (SakiAgentSSH Scientia 202603272240)
- Agent Tool Boundary Deep Reverse Engineering (SakiAgentSSH Scientia 202603272235)

---

## Appendix A: Reference Implementation (Rust)

Repository: github.com/saki-studio/saki-ssh-daemon (pending)
Crate: saki-ssh (pending crates.io publication)

## Appendix B: Reference Implementation (Go/Swift)

[To be determined based on interoperability testing requirements]
