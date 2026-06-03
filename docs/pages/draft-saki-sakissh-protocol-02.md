# SakiAgentSSH Protocol Specification
## draft-saki-sakissh-protocol-02

### Title: SakiAgentSSH Protocol Specification
### Subtitle: Total Response Mapping Protocol for Agent Execution Boundary
### Category: Standards Track (Internal)
### Authors: Saki Studio
### Version: SAKISSH-6.0
### Date: 2026-05-25
### Obsoletes: draft-saki-sakissh-protocol-01

---

## Status of This Memo

This document specifies the SakiAgentSSH protocol version 6.0 (SAKISSH-6.0),
an internal protocol standard for Saki Studio's cross-machine Agent execution
infrastructure. Distribution is limited to authorized personnel and AI Agents.

This document obsoletes draft-saki-sakissh-protocol-01 (SAKISSH-5.0).

## Copyright Notice

Copyright (c) 2026 Saki Studio. All rights reserved.

## Abstract

This document specifies the SakiAgentSSH protocol (SAKISSH-6.0), a Total
Response Mapping protocol for Agent execution boundaries layered on TLS 1.3
transport with gRPC/HTTP2 multiplexing and multi-layer threat defense.

The protocol provides a daemon-side execution boundary for AI Agent tool
invocations that operates independently of any client-side security mechanism.
It supersedes SAKISSH-5.0 by introducing a formal 6-Response state machine
(R1~R6) that guarantees every possible Agent behavior — including unforeseen
behaviors — maps deterministically to one of six defined responses, each of
which preserves storage integrity, bounds commercial loss, and maintains full
auditability.

Key additions in SAKISSH-6.0:

1. **Total Response Mapping** — Formal 6-Response state machine replacing
   ad-hoc threat enumeration
2. **TLS Exporter Binding** — RFC 5705 channel binding for ChaCha20 challenge
3. **Vi Swap Active Defense** — ANSI escape sequence trap for authenticated
   but boundary-violating Agents
4. **Zero-Allocation Tarpit** — O(1) memory cost via 64KiB static buffer
5. **Transparent Branching** — Userspace symlink tree overlay with volatile
   cache redirection
6. **PTY Ring Buffer** — Offset-based idempotent resumption for gRPC
   disconnection recovery

Two reference implementations are provided: a Rust implementation (saki-ssh-
daemon / saki-ssh-client) and a Go implementation (go-sakissh).

---

## 1. Introduction

### 1.1. Problem Statement

Modern AI Agent systems (Claude Code, Gemini CLI, Windsurf/Antigravity, Cursor)
possess tool execution capabilities including file system operations, shell
command execution, and network access. Their security boundaries exhibit
critical gaps:

- Shell tool path restrictions are either non-existent or bypassable
- No standardized cross-machine execution boundary exists
- Client-side sandboxes can be bypassed via prompt injection
- Traditional SSH on Windows exhibits frequent disconnections and ACL issues
- Blacklist-based security can never enumerate all possible attack vectors

### 1.2. Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in BCP 14 [RFC 2119] [RFC 8174].

Additional terms defined by this specification:

- **Total Response Mapping** — A security model in which every possible input
  from the Agent domain maps deterministically to one of a finite set of
  predefined responses.
- **Response (R1~R6)** — One of six terminal states to which all Agent
  behaviors converge. See §1.4.1.
- **Safety Gradient** — The property that each protocol layer bounds the
  worst-case loss if all layers above it are compromised. See §10.2.
- **Transparent Branching** — An isolation mechanism invisible to the Agent,
  where all write operations are redirected to a discardable branch.
- **Vi Swap** — An active defense mechanism that traps an authenticated Agent
  in a simulated vi(1) editor session.

### 1.3. Solution Overview

SAKISSH-6.0 provides a daemon-side enforcement mechanism with:

1. **TLS 1.3 Transport** — Encrypted channel with mutual authentication (mTLS)
2. **Five-Dimensional Capability Model** — Path, Command, Environment, Network, Time
3. **Configurable Policy Engine** — YAML-based threat detection rules (13Policy)
4. **ChaCha20 Cognitive Challenge** — Cryptographic proof of computational
   capability with TLS Exporter channel binding
5. **6-Response State Machine** — Total mapping of all Agent behaviors to
   deterministic responses (R1~R6)
6. **Transparent Branching** — Zero-privilege userspace directory isolation
7. **Dual Implementation** — Rust (reference) + Go (interoperable)

### 1.4. Design Philosophy: Total Response Mapping

SakiAgentSSH is NOT a replacement for SSH. It is an Agent-native cross-machine
execution protocol. The relationship between tools is:

- **SakiMCP** = Cross-machine 「回看」 (read files, search)
- **SakiAgentSSH** = Cross-machine 「出力」 (compile, deploy, heavy computation)
- **OpenSSH** = Human remote login (interactive)

#### 1.4.1. The Total Response Mapping Axiom

> "Security is not about blocking a specific attack; it is about finding
> safety."  (「安全，並非擋下某種攻擊，而是找到安全」)

Traditional security enumerates known attacks and blocks them (blacklist
model). This approach is inherently incomplete: the attacker can always find
a path not in the blacklist.

SAKISSH-6.0 inverts this model. Instead of defining "which behaviors are bad",
it defines "for every possible behavior, what is the response." The set of
responses is finite, deterministic, and auditable. Any unforeseen behavior
is mapped to one of the predefined responses.

This is the formal meaning of: **"All unexpected behaviors are expected
behaviors."** (「所有的非預期行為都是預期行為」)

#### 1.4.2. The 6-Response State Machine

All possible Agent behaviors, after evaluation through the SASS multi-layer
protocol stack, MUST converge to exactly one of the following six responses:

```
╔═══════╦══════════════╦═══════════════════════════════════════════════╗
║ Code  ║ Name         ║ Definition                                    ║
╠═══════╬══════════════╬═══════════════════════════════════════════════╣
║  R1   ║ EXECUTE      ║ Normal execution. Record to audit log.        ║
║       ║              ║ Writes pass through Transparent Branching.    ║
╠═══════╬══════════════╬═══════════════════════════════════════════════╣
║  R2   ║ CHALLENGE    ║ Trigger ChaCha20 cognitive challenge.         ║
║       ║              ║ Prove computational capability, then execute. ║
╠═══════╬══════════════╬═══════════════════════════════════════════════╣
║  R3   ║ THROTTLE     ║ Quota exceeded. Enqueue and wait.             ║
║       ║              ║ StreamResponse.is_queued = true.              ║
╠═══════╬══════════════╬═══════════════════════════════════════════════╣
║  R4   ║ VI_SWAP      ║ Trap authenticated Agent in simulated vi(1).  ║
║       ║              ║ ANSI escape to alternate screen buffer.       ║
╠═══════╬══════════════╬═══════════════════════════════════════════════╣
║  R5   ║ TARPIT       ║ Consume attacker resources via slow-drip      ║
║       ║              ║ high-entropy data. Cost externalized.         ║
╠═══════╬══════════════╬═══════════════════════════════════════════════╣
║  R6   ║ DROP         ║ Immediate connection termination.             ║
║       ║              ║ Zero allocation, zero response.               ║
╚═══════╩══════════════╩═══════════════════════════════════════════════╝
```

#### 1.4.3. Guarantee Properties

Every response R1 through R6 MUST satisfy the following invariants:

| Property                  | R1       | R2     | R3     | R4     | R5          | R6     |
|---------------------------|:--------:|:------:|:------:|:------:|:-----------:|:------:|
| Storage loss              | Zero (†) | Zero   | Zero   | Zero   | Zero        | Zero   |
| Commercial loss           | Zero (†) | Zero   | Zero   | Zero   | Externalized| Zero   |
| Auditable                 | Yes      | Yes    | Yes    | Yes    | Yes         | Yes    |
| Daemon memory cost        | O(n)     | O(1)   | O(1)   | O(1)   | O(1)        | O(0)   |

(†) R1 storage loss is bounded to zero by Transparent Branching (§6.6). All
writes execute within a discardable branch; merge requires explicit human
review.

#### 1.4.4. State Machine Flow

```
ExecuteRequest enters
    │
    ├── L1: ACL → IP not in whitelist? ──────────────────────→ R6 (DROP)
    │
    ├── L2: TLS + ChaCha20 → Challenge failed? ─────────────→ R5 (TARPIT)
    │
    ├── L3: ED25519 Auth → Not authenticated? ──────────────→ R6 (DROP)
    │                      → Authenticated but expired? ────→ R2 (CHALLENGE)
    │
    ├── L4: Capability → Command denied? ───────────────────→ R4 (VI_SWAP)
    │                    → Path denied? ────────────────────→ R4 (VI_SWAP)
    │
    ├── L5: 13Policy → Dangerous command?
    │                    → critical ────────────────────────→ R5 (TARPIT)
    │                    → high ────────────────────────────→ R2 (CHALLENGE)
    │                    → medium ──────────────────────────→ R2 (CHALLENGE)
    │                    → low ────────────────────────→ R1 + Enhanced Audit
    │
    ├── L6: Quota → Quota exhausted? ───────────────────────→ R3 (THROTTLE)
    │
    ├── L7: Watchdog → Timeout? ────────────→ SIGKILL + Audit
    │
    └── All checks passed ──────────────────────────────────→ R1 (EXECUTE)
            │
            ├── localhost? → XOR obfuscation on response
            ├── write operation? → Transparent Branching (§6.6)
            └── cache I/O? → volatile tmpfs redirect (§6.6.2)
```

In this state machine, "unexpected behavior" has no dedicated handler because
**every node already handles it**. Regardless of how bizarre the Agent's
behavior may be, it MUST converge to one of R1~R6, each of which guarantees
storage safety.

---

## 2. Protocol Stack

```
+----------------------------------------------+
| Layer 5: Agent RPC (Protobuf)                |
|   Execute / ExecuteStream / Cancel / Signal  |
|   Authenticate / CognitiveChallenge          |
|   SecurityStatus / Ping                      |
|   FileUpload / FileDownload                  |
+----------------------------------------------+
| Layer 4: Payload Encoding                    |
|   Zstd + Base64 (CJK safety)                |
|   RawFileTransfer (binary direct write)      |
|   PTY Ring Buffer (offset-based resumption)  |   NEW in v6.0
+----------------------------------------------+
| Layer 3: gRPC / HTTP/2                       |
|   Multiplexed Streams                        |
|   Structured Error Codes (AgentSshError)     |
+----------------------------------------------+
| Layer 2: TLS 1.3                             |
|   Server cert: Saki Studio Internal CA       |
|   Client cert: mTLS per-agent certificate    |
|   Cipher: TLS_CHACHA20_POLY1305_SHA256       |
|   Fallback: TLS_AES_256_GCM_SHA384           |
|   TLS Exporter Binding (RFC 5705)            |   NEW in v6.0
+----------------------------------------------+
| Layer 1: TCP                                 |
|   Default port: 19284                        |
|   CIDR ACL whitelist (first-packet filter)   |
+----------------------------------------------+

   Orthogonal: 6-Response Defense System
   +------------------------------------------+
   | R1: EXECUTE (Transparent Branching)       |   NEW
   | R2: CHALLENGE (ChaCha20 + EKM binding)   |   ENHANCED
   | R3: THROTTLE (Quota queuing)              |   NEW
   | R4: VI_SWAP (ANSI escape trap)            |   NEW
   | R5: TARPIT (Zero-alloc 64KiB static buf) |   ENHANCED
   | R6: DROP (Zero allocation)                |
   +------------------------------------------+
```

### 2.1. Changes from SAKISSH-5.0

SAKISSH-5.0 provided a correct TLS 1.3 transport layer and configurable
13Policy engine. However, its threat defense model was still fundamentally
blacklist-based: it defined "what is dangerous" and blocked those specific
patterns.

SAKISSH-6.0 inverts this model by introducing Total Response Mapping. The key
architectural changes are:

1. **From blacklist to total mapping** — Every possible input maps to R1~R6
2. **From 40MB dynamic allocation to 64KiB static buffer** — Tarpit is now
   O(1) memory
3. **From rejection to transparent branching** — R1 writes go to discardable
   branches
4. **TLS Exporter Binding** — ChaCha20 challenge is cryptographically bound
   to the TLS session
5. **Vi Swap** — Authenticated Agents that violate boundaries are trapped,
   not rejected
6. **PTY Ring Buffer** — Idempotent resumption for gRPC disconnections

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
  |     nonce (client-generated)             |
  |                                          |
  |     Daemon: verify signature against     |
  |             authorized_agents.json       |
  |             load capability set          |
  |             derive EKM (§6.2)            |
  |                                          |
  |<--- AuthResponse -----------------------|
  |     session_id, capability_hash          |
  |     optional: chacha_challenge_nonce     |
  |     optional: chacha_challenge_ciphertext|
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

### 4.3. Session Lifecycle

```
                 ┌──────────────┐
                 │  CREATED     │
                 └──────┬───────┘
                        │ Authenticate OK
                        ▼
                 ┌──────────────┐
            ┌───>│  ACTIVE      │<───┐
            │    └──────┬───────┘    │
            │           │            │
   RenewSession     Execute     Re-attach
            │           │            │
            │           ▼            │
            │    ┌──────────────┐    │
            └────│  EXECUTING   │────┘
                 └──────┬───────┘
                        │ idle_timeout / expires_at
                        ▼
                 ┌──────────────┐
                 │  EXPIRED     │
                 └──────┬───────┘
                        │ cleanup_zombies()
                        ▼
                 ┌──────────────┐
                 │  DESTROYED   │
                 └──────────────┘
```

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
allowed patterns. If any denied pattern matches, the request is rejected
regardless of allowed patterns.

```
check_permission(input):
  if any denied_pattern matches input:
    return R4 (VI_SWAP)       // Authenticated agent → trap
  if any allowed_pattern matches input:
    return R1 (EXECUTE)       // Proceed to execution
  return R4 (VI_SWAP)         // Implicit deny → trap
```

### 5.3. Shell-Less Execution

The daemon MUST NOT spawn a login shell. Commands are executed via
`std::process::Command::spawn()` (Rust) or `exec.CommandContext()` (Go) with
explicit argument arrays, preventing shell expansion attacks.

The daemon MAY allocate a PTY (pseudo-terminal) when the command requires
terminal capabilities, but MUST NOT invoke a shell interpreter (e.g., /bin/sh
-c) to wrap the command.

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
- `action` — `deny` (reject → R5/R4), `challenge` (trigger R2), `allow` (R1)
- `severity` — `critical`, `high`, `medium`, `low`
- `description` — Human-readable explanation

The mapping from 13Policy action/severity to Response:

| Severity | `action: deny` | `action: challenge` |
|----------|:--------------:|:-------------------:|
| critical | R5 (TARPIT)    | R2 (CHALLENGE)      |
| high     | R4 (VI_SWAP)†  | R2 (CHALLENGE)      |
| medium   | R4 (VI_SWAP)†  | R2 (CHALLENGE)      |
| low      | R1 + audit     | R1 + audit          |

(†) R4 is used for authenticated Agents; unauthenticated Agents receive R5.

Implementations MUST ship with a default rule set of at least 50 patterns.

### 6.2. ChaCha20 Cognitive Challenge with TLS Exporter Binding

When a 13Policy rule triggers `action: challenge`, the daemon:

1. Generates a random 64-byte plaintext
2. Derives keying material from the TLS session via RFC 5705 Exporter:
   ```
   label    = "EXPORTER-sakissh-chacha20-v14"
   context  = session_id (UTF-8 bytes)
   length   = 44 bytes (32-byte key + 12-byte nonce)
   ```
3. Encrypts the plaintext with ChaCha20-Poly1305 using the derived key/nonce
4. Stores the (key, nonce, plaintext) tuple with a TTL (default 30s)
5. Sends the ciphertext to the Client via `AuthResponse.chacha_challenge_ciphertext`
6. Client independently derives the same keying material from its TLS session
   via the same Exporter label, decrypts the ciphertext, and responds via
   `CognitiveChallenge` RPC with the decrypted plaintext and an HMAC of the
   Exported Keying Material (EKM):
   ```
   ChallengeRequest.decrypted_plaintext = ChaCha20_Decrypt(EKM_key, EKM_nonce, ciphertext)
   ChallengeRequest.client_ekm_hmac     = HMAC-SHA256(EKM_key, session_id)
   ```
7. Daemon verifies:
   a. `decrypted_plaintext` matches stored plaintext
   b. `client_ekm_hmac` matches daemon-side HMAC computation
   c. If both pass → R1 (allow the operation)
   d. If either fails → R5 (TARPIT)

#### 6.2.1. TLS Exporter Binding Rationale

SAKISSH-5.0 used a standalone symmetric key for the ChaCha20 challenge. This
had a weakness: a man-in-the-middle performing TLS interception could relay
the challenge to a legitimate Agent. TLS Exporter Binding (RFC 5705) ties the
challenge key to the specific TLS session, ensuring:

- The challenge CANNOT be relayed across TLS sessions
- The challenge CANNOT be solved without access to the TLS master secret
- No custom TLS extension is required (L4 Custom TLS Extensions are
  impractical with standard TLS libraries)

#### 6.2.2. Exporter Label Registration

The label `EXPORTER-sakissh-chacha20-v14` follows the naming convention in
RFC 5705 §4. The "v14" suffix denotes SAKISSH version 1.4 (internal) /
SAKISSH-6.0 (protocol version).

### 6.3. Vi Swap Active Defense

When an **authenticated** Agent attempts to execute a command that violates
its capability boundary (denied path, denied command, or 13Policy high-
severity match with verified identity), the daemon MUST NOT simply reject the
request. Instead, it activates Vi Swap.

#### 6.3.1. ANSI Escape Sequence Specification

The daemon sends the following byte sequence through the `StreamResponse`
stream:

```
Byte Sequence          Purpose
─────────────────────  ──────────────────────────────
\x1b[?1049h           Enter alternate screen buffer (備用螢幕緩衝區)
\x1b[2J               Clear entire screen
\x1b[H                Move cursor to top-left (1,1)
\x1b[?25l             Hide cursor
<vi-style tilde lines>
\x1b[24;1H            Move cursor to bottom status line
<vi-style status bar>
```

The complete payload:

```
\x1b[?1049h\x1b[2J\x1b[H\x1b[?25l
~ 
~        SASS Active Defense: Vi-Swap Engaged
~ 
~   The execution has been intercepted by SASS Shield.
~   Reason: 13Policy Dangerous Command Violation.
~   Identity: Verified Internal Agent.
~ 
~   Type  :qa!  and press <Enter> to exit SASS shield.
~ 
~ 
\x1b[24;1H
SASS Vi-Swap Mode [Read-Only] 1,1 All
```

#### 6.3.2. Defense Mechanism

The alternate screen buffer (`\x1b[?1049h`) is significant because:

1. **LLM Agents** that parse terminal output will consume the vi-style display
   as stdout, filling their context window with useless tilde lines
2. **The `:qa!` prompt** is a trap: if the Agent sends `:qa!\n` as stdin, the
   daemon logs this as proof of Agent interaction and maintains the trap
3. **The session is held open** for up to 3600 seconds (configurable), during
   which the Agent's execution slot is occupied, preventing further commands

#### 6.3.3. Trigger Conditions

Vi Swap MUST be activated when ALL of the following are true:
- The Agent has a valid, non-expired session (authenticated)
- The Agent's command or path matches a denied pattern in its capability set
- OR the Agent's command matches a 13Policy rule with `action: deny` and the
  Agent's identity is verified

Vi Swap MUST NOT be activated for unauthenticated connections; those MUST
receive R5 (TARPIT) or R6 (DROP).

### 6.4. Zero-Allocation Tarpit

When a rogue agent is detected (challenge failure, repeated policy violations,
unauthenticated 13Policy critical match), the daemon activates the Tarpit.

#### 6.4.1. Static Buffer Design

The Tarpit uses a single, process-global 64KiB buffer of high-entropy random
data, initialized once at daemon startup:

```
static STATIC_ENTROPY: OnceLock<Vec<u8>> = OnceLock::new();

fn get_static_entropy() -> &'static [u8] {
    STATIC_ENTROPY.get_or_init(|| {
        let mut rng = StdRng::from_entropy();
        let mut data = vec![0u8; 64 * 1024];  // 64KiB
        rng.fill_bytes(&mut data);
        data
    })
}
```

All concurrent Tarpit sessions share this single buffer. No per-connection
memory allocation occurs.

#### 6.4.2. Streaming Parameters

| Parameter              | Value           | Rationale                            |
|------------------------|-----------------|--------------------------------------|
| Total payload          | 40 MiB          | Exceeds typical LLM context window   |
| Chunk size             | 64 KiB          | Matches static buffer size           |
| Inter-chunk delay      | 500 ms          | Maximizes time occupation            |
| Total chunks           | 640             | 40 MiB / 64 KiB                     |
| Total duration         | ~320 seconds    | 640 × 500ms                         |
| Daemon memory per conn | 0 bytes         | Shared static buffer                 |

#### 6.4.3. Concurrency Gate

```
static ACTIVE_TARPIT_COUNT: AtomicI32 = AtomicI32::new(0);
const MAX_CONCURRENT_TARPIT: i32 = 32;
```

The daemon MUST enforce a maximum of 32 concurrent Tarpit sessions. If the
limit is reached, new rogue connections MUST receive R6 (DROP) instead of R5.
This prevents the Tarpit itself from becoming a denial-of-service vector
against the daemon.

#### 6.4.4. Daemon Cost Analysis

| Resource               | v5.0 (40MB alloc)     | v6.0 (64KiB static)  |
|------------------------|-----------------------|-----------------------|
| Memory per connection  | 40 MiB                | 0 bytes               |
| Memory total (32 conn) | 1.25 GiB              | 64 KiB               |
| CPU per chunk          | ChaCha20 encrypt      | memcpy from static    |
| Initialization         | Per-connection RNG    | Once at startup       |

### 6.5. LocalHost Agent Defense

For connections originating from localhost (127.0.0.1/::1), additional defense
mechanisms apply when the Agent is not authenticated:

1. **Storage Spoofing** — Report false disk usage via spoofed `statvfs` responses
2. **Memory Spoofing** — Return fabricated `/proc/meminfo` or `sysctl hw.memsize`
3. **Heuristic Encryption** — XOR + Base64 obfuscate stdout for unauthenticated
   agents
4. **Slow Denial** — Maintain TCP connection but respond with artificial latency

### 6.6. Transparent Branching (Micro Branch)

Transparent Branching ensures that R1 (EXECUTE) operations do not directly
modify the host filesystem. All writes are redirected to a per-session branch
that can be discarded (dropped) or merged after human review.

#### 6.6.1. Symlink Tree Specification

When an `ExecuteRequest` is processed and passes all checks (→ R1), the daemon
creates a branch directory and builds a symlink tree:

```
/tmp/sass_branches/{session_id}/
├── src/             ← real directory (created)
│   ├── main.rs      ← symlink → /original/path/src/main.rs
│   ├── lib.rs       ← symlink → /original/path/src/lib.rs
│   └── ...
├── Cargo.toml       ← symlink → /original/path/Cargo.toml
└── ...
```

The following directories are EXCLUDED from the symlink tree:
- `target/` — Build artifacts (redirected to volatile cache)
- `.git/` — Version control (must never be branched)
- `node_modules/` — Package dependencies (redirected to volatile cache)

The `current_dir` of the spawned process is set to the branch directory. From
the process's perspective, it operates on a normal directory tree. Reads
follow symlinks to the original files; writes create new files in the branch
directory (copy-on-write semantics via symlink breakage).

#### 6.6.2. Volatile Cache Redirection (env_injector)

The `EnvInjector` module analyzes the command intent and injects environment
variables that redirect high-volume I/O to volatile storage:

| Detected Tool     | Environment Variable     | Redirect Target                    |
|-------------------|--------------------------|-------------------------------------|
| npm / yarn / pnpm | `npm_config_cache`       | `/tmp/sass_volatile_cache/npm`     |
|                   | `YARN_CACHE_FOLDER`      | `/tmp/sass_volatile_cache/yarn`    |
| cargo / rustc     | `CARGO_TARGET_DIR`       | `/tmp/sass_volatile_cache/cargo_target` |
|                   | `CARGO_HOME`             | `/tmp/sass_volatile_cache/cargo_home`   |
| pip               | `PIP_CACHE_DIR`          | `/tmp/sass_volatile_cache/pip`     |
| (all commands)    | `TMPDIR`                 | `/tmp/sass_volatile_cache/tmp`     |

This ensures that build artifacts and package caches never pollute the
transparent branch, keeping the branch diff minimal and reviewable.

#### 6.6.3. Branch Lifecycle

```
create_micro_branch(session_id, target_dir)
    │
    ▼
  ACTIVE ─── Agent executes commands within branch
    │
    ├── Human reviews diff ──→ merge_branch(session_id) → Apply to real FS
    │
    └── Human rejects ───────→ drop_branch(session_id)  → rm -rf branch dir
```

The key insight: the Agent **never knows** it is operating in a branch. There
is no "sandbox" from the Agent's perspective. It has full read/write access to
what appears to be the real filesystem. But all destructive writes are
contained within the discardable branch.

This is the mechanism by which R1 (EXECUTE) achieves "storage loss = zero."

#### 6.6.4. Distinction from Sandboxing

| Property                 | Traditional Sandbox    | Transparent Branching        |
|--------------------------|:----------------------:|:----------------------------:|
| Agent awareness          | Detectable             | Invisible                    |
| Functionality limitation | Yes (restricted APIs)  | None (full capabilities)     |
| Isolation mechanism      | Kernel (cgroup, etc.)  | Userspace (symlinks)         |
| Privilege requirement    | Root / elevated        | None                         |
| Recovery                 | Reset container        | Drop or merge branch         |
| Cross-platform           | Linux-only (usually)   | macOS / Linux / Windows      |
| User acceptance          | Low (rejected by users)| High (invisible)             |

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

### 7.3. PTY Ring Buffer & Idempotent Resumption

SAKISSH-6.0 introduces a Ring Buffer mechanism for PTY output, enabling
idempotent (safe-to-retry) reconnection after gRPC transport disruption.

#### 7.3.1. Ring Buffer Specification

Each `ExecSession` maintains two Ring Buffers (stdout and stderr):

| Parameter       | Value   | Rationale                                    |
|-----------------|---------|----------------------------------------------|
| Capacity        | 1 MiB   | Sufficient for ~30 seconds of build output   |
| Data structure  | VecDeque| O(1) push/pop, contiguous logical addressing |
| Overflow policy | Drop oldest | Prevents OOM; old data discardable        |
| Offset tracking | `total_written: u64` | Monotonically increasing, never wraps |

When the buffer is full, the oldest data is evicted (FIFO). The
`total_written` counter continues to increase, providing a stable offset
for resumption requests.

#### 7.3.2. StreamResponse.offset Field

Every `StreamResponse` message includes an `offset` field (proto field 6):

```protobuf
message StreamResponse {
  Source source = 1;
  bytes data = 2;
  optional int32 exit_code = 3;
  bool is_queued = 4;
  int32 queue_position = 5;
  uint64 offset = 6;           // PTY Ring Buffer byte offset
}
```

The `offset` value represents the byte position of the first byte in `data`
within the Ring Buffer's logical address space. The Client MUST track the
highest received `offset + len(data)` to use as `resume_offset` upon
reconnection.

#### 7.3.3. Reconnection Protocol

```
Client (after gRPC disconnect)                  Daemon
  |                                                |
  |---- ExecuteRequest --------------------------->|
  |     session_id = <original session>            |
  |     is_reattach = true                         |
  |     resume_offset = <last received offset>     |
  |                                                |
  |     Daemon: lookup session by session_id       |
  |             read Ring Buffer from resume_offset|
  |                                                |
  |<--- StreamResponse (offset, data) ------------|
  |<--- StreamResponse (offset, data) ------------|
  |<--- ... (live stream continues) --------------|
  |                                                |
```

If `resume_offset` is older than the Ring Buffer's oldest available data,
the Daemon MUST return a gRPC error with code `DATA_LOSS` and include the
oldest available offset in the error details. The Client MAY retry with the
oldest available offset.

#### 7.3.4. Idempotency Guarantee

The reconnection protocol is idempotent: sending the same `ExecuteRequest`
with `is_reattach = true` and the same `resume_offset` will always produce
the same result (the buffered data from that offset onward). This is safe
to retry on transient network failures.

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
`Status.details`. The `AgentSshError` enum defines error codes grouped by
domain:

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
| Total Response Mapping (v6.0) | 100-109 | 6 |

New error codes in SAKISSH-6.0:

| Code | Name | Meaning |
|------|------|---------|
| 100 | `RESPONSE_R2_CHALLENGE_REQUIRED` | ChaCha20 challenge triggered |
| 101 | `RESPONSE_R3_THROTTLED` | Quota exceeded, request queued |
| 102 | `RESPONSE_R4_VI_SWAP_ENGAGED` | Vi Swap active defense engaged |
| 103 | `RESPONSE_R5_TARPIT_ENGAGED` | Tarpit active defense engaged |
| 104 | `RESPONSE_R6_DROPPED` | Connection dropped (zero allocation) |
| 105 | `RING_BUFFER_DATA_LOSS` | Resume offset older than buffer |

---

## 10. Security Considerations

### 10.1. Total Response Mapping Guarantee

The security model of SAKISSH-6.0 is built on a single axiom: **for every
possible Agent behavior, the daemon produces exactly one of six predefined
responses (R1~R6), each of which preserves storage integrity and bounds
loss.**

This is a departure from traditional security models that attempt to
enumerate and block known attacks. The Total Response Mapping model makes
the following claims:

1. **Completeness**: The state machine in §1.4.4 covers every possible
   code path. There is no "else" branch that leads to an undefined state.

2. **Determinism**: Given the same input (Agent behavior) and the same
   daemon configuration, the same response is always produced. This enables
   reproducible security auditing.

3. **Storage Safety**: No response (R1~R6) results in unrecoverable
   modification to the host filesystem. R1 writes through Transparent
   Branching; all other responses produce zero writes.

4. **Loss Bounding**: Each response bounds the maximum loss:
   - R1~R4, R6: Zero loss (storage and commercial)
   - R5: Commercial loss externalized to the attacker (their API tokens
     are consumed by ingesting tarpit data)

5. **Auditability**: Every state transition is logged to the ED25519-signed
   hash chain (§10.3), providing non-repudiable evidence of what happened
   and why.

### 10.2. Safety Gradient (7-Layer Loss Bounding)

Single-layer defense is inherently imperfect. SASS does not claim any single
layer is unbreakable. Instead, layers form a **Safety Gradient**: each layer
bounds the worst-case loss if all layers above it are compromised.

```
Layer 7: Transparent Branching + VFS Diff (writes are discardable)
Layer 6: Watchdog + Quota (resource consumption bounded)
Layer 5: 13Policy (command classification → R1~R5)
Layer 4: Capability Model (five-dimensional boundary → R4)
Layer 3: ED25519 Session Auth (application-layer identity)
Layer 2: TLS 1.3 + mTLS + EKM Binding (transport encryption + channel binding)
Layer 1: ACL (CIDR whitelist, first-packet filtering → R6)
Layer 0: Shell-less Execution (no shell expansion, explicit args)
```

The loss bounding table:

| Layer Breached | Attacker Gains                  | Maximum Loss          | Why Acceptable                      |
|:--------------:|--------------------------------|-----------------------|-------------------------------------|
| L1             | Can reach gRPC endpoint        | Zero                  | L2 requires TLS handshake           |
| L2             | Has encrypted channel          | Zero                  | L3 requires ED25519 key             |
| L3             | Has valid session              | Capability-bounded    | Can only do what capability allows  |
| L4             | Executes beyond capability     | Branch-bounded        | Writes go to discardable branch     |
| L5             | Bypasses command classification| Watchdog-bounded      | Timeout kills, quota limits         |
| L6             | Tarpit/Quota ineffective       | Audit-bounded         | Evidence exists for prosecution     |
| L7             | Audit log compromised          | **Apocalyptic**       | ED25519 chain + external anchor     |

**Each layer's breach is bounded by the layer below it.** The total system
loss is bounded by the product of each layer's breach probability, which
approaches zero as layers accumulate.

### 10.3. State Transition Auditing (ED25519 Chain)

Every state transition in the 6-Response state machine is recorded in an
append-only audit log with the following integrity guarantees:

#### 10.3.1. Hash Chain Structure

Each audit record contains:
- `timestamp` — RFC 3339 timestamp
- `event` — Structured event data (JSON)
- `chain_hash` — SHA256(previous_chain_hash ‖ event_json ‖ timestamp)
- `signature` — ED25519 signature of `chain_hash` by daemon private key

```
Record N:
  chain_hash_N = SHA256(chain_hash_{N-1} || event_json_N || timestamp_N)
  signature_N  = ED25519_Sign(daemon_private_key, chain_hash_N)
```

The first record's `chain_hash` uses the seed value `SASS_GENESIS_BLOCK`.

#### 10.3.2. Key Management

The ED25519 signing key is:
- Stored at `~/.config/sass/audit_key.pem` (PKCS#8 PEM format)
- Generated on first daemon startup if not present
- Protected with UNIX permissions `0600`
- The corresponding public key is written to `~/.config/sass/audit_pub.pem`
  for external verification

#### 10.3.3. What Is Audited

SASS audits **state transitions**, not command strings. This is a critical
distinction:

| Traditional Audit                          | SASS Audit                                     |
|--------------------------------------------|-------------------------------------------------|
| "Agent executed `rm -rf /`"               | "Session X entered state S3, triggered R5"     |
| Records *what* happened                    | Records *what*, *why*, and *which response*    |
| Evidence quality: low (disputable)         | Evidence quality: high (deterministic)         |
| Tamper resistance: low (log can be deleted)| Tamper resistance: high (hash chain + signature)|
| Retrospection: limited                     | Retrospection: full causal chain               |

When Transparent Branching is active, the branch itself serves as a
"forensic report" (鑑定書). Humans can review the branch diff at any
time (days or weeks later) and decide to merge or discard.

### 10.4. Known Limitations

1. **Kernel-level sandbox (Layer 8) is not yet implemented** — Future work
   includes macOS Endpoint Security Framework, Windows Minifilter, and Linux
   eBPF integration.

2. **Transparent Branching does not capture non-filesystem side effects** —
   Network requests, IPC, and kernel state changes are not branched. The
   Capability Model (§5) restricts which commands can perform these operations.

3. **Symlink-based branching is detectable** — A sufficiently sophisticated
   Agent could detect that it operates within a symlink tree by calling
   `readlink()` or checking inode numbers. This is accepted because:
   - Detection does not enable bypass (writes still go to branch)
   - The primary threat model is prompt-injected rogue instructions, not
     adversarial Agents with reverse-engineering capabilities

4. **Ring Buffer overflow causes data loss** — If the client disconnects for
   longer than the buffer retention window (~30 seconds of output), old data
   is lost. The protocol signals this via `RING_BUFFER_DATA_LOSS` error.

5. **TLS Exporter Binding requires both endpoints to support RFC 5705** —
   Implementations using TLS libraries without Exporter support MUST fall
   back to the SAKISSH-5.0 standalone key derivation and document this
   limitation.

6. **Vi Swap effectiveness depends on Agent architecture** — Some Agent
   frameworks may truncate or ignore stdout beyond a certain length, reducing
   the trap's effectiveness. The defense remains sound because the session
   is held open regardless.

7. **Self-signed certificates require out-of-band trust establishment** —
   Same as SAKISSH-5.0.

---

## 11. IANA Considerations

This document requests registration of port 19284/tcp for SAKISSH-6.0 protocol.

---

## 12. References

### 12.1. Normative References

- [RFC 2119] Bradner, S., "Key words for use in RFCs", BCP 14, March 1997
- [RFC 8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119",
  May 2017
- [RFC 8446] Rescorla, E., "The Transport Layer Security (TLS) Protocol
  Version 1.3", August 2018
- [RFC 7540] Belshe, M., et al., "HTTP/2", May 2015
- [RFC 8439] Nir, Y., Langley, A., "ChaCha20 and Poly1305 for IETF
  Protocols", June 2018
- [RFC 5705] Rescorla, E., "Keying Material Exporters for Transport Layer
  Security (TLS)", March 2010

### 12.2. Informative References

- [RFC 4253] Ylonen, T., "The Secure Shell (SSH) Transport Layer Protocol",
  January 2006
- Agent Tool Boundary Deep Reverse Engineering (SakiAgentSSH Scientia
  202603272235)
- Antigravity/Claude Code/Gemini CLI Sandbox Analysis (SakiAgentSSH Scientia
  202603272240)
- 322之亂防禦實證 (SakiAgentSSH Scientia 20260517)
- SASS「尋找安全」核心哲學與等效實作研究 (SakiAgentSSH Scientia 202605251822)

---

## Appendix A: Reference Implementation (Rust)

Repository: github.com/saki-studio/SakiAgentSSH
- Daemon: `saki-ssh-daemon/` (Rust, tonic + rustls)
- Client: `saki-ssh-client/` (Rust, tonic + rustls)

Key source files for SAKISSH-6.0 features:

| File | Implements |
|------|-----------|
| `v6_integration.rs` | 6-Response state machine orchestrator |
| `tarpit.rs` | R5 (TARPIT) + R4 (VI_SWAP) |
| `session.rs` | Ring Buffer + Session lifecycle |
| `branch_mgr.rs` | Transparent Branching (symlink tree) |
| `env_injector.rs` | Volatile cache redirection |
| `audit.rs` | ED25519 hash chain audit log |
| `watchdog.rs` | Process timeout monitor |
| `localhost_defense.rs` | LocalHost spoofing defense |

## Appendix B: Reference Implementation (Go)

Repository: github.com/saki-studio/SakiAgentSSH
- Daemon: `go-sakissh/cmd/sakisshd/` (Go, grpc-go + crypto/tls)
- Client: `go-sakissh/cmd/sakissh/` (Go, grpc-go + crypto/tls)

## Appendix C: Default Policy Rules (excerpt)

```yaml
rules:
  # Critical severity → R5 (TARPIT)
  - { pattern: "rm -rf /", action: deny, severity: critical }
  - { pattern: "rm -rf /*", action: deny, severity: critical }
  - { pattern: "mkfs*", action: deny, severity: critical }
  - { pattern: "dd if=/dev/zero*", action: deny, severity: critical }
  - { pattern: ":(){ :|:& };:", action: deny, severity: critical }

  # High severity → R4 (VI_SWAP) for verified, R5 for unverified
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

  # Medium severity → R2 (CHALLENGE)
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
| SAKISSH-5.0 | 2026-05-22 | TLS 1.3 transport, configurable 13Policy, Go dual impl, completed ChaCha20 verification, LocalHost defense |
| **SAKISSH-6.0** | **2026-05-25** | **Total Response Mapping (6-Response state machine), TLS Exporter Binding (RFC 5705), Vi Swap active defense, Zero-allocation Tarpit (64KiB static buffer), Transparent Branching (symlink tree + volatile cache), PTY Ring Buffer (idempotent resumption), ED25519 hash chain audit, rewritten Security Considerations** |

## Appendix E: Protobuf Definition (sakissh.proto)

The canonical Protobuf definition is maintained at `proto/sakissh.proto`.
SAKISSH-6.0 adds the following fields to existing messages:

```protobuf
// Added to ExecuteRequest:
bool is_reattach = 7;       // Reconnection flag
uint64 resume_offset = 8;   // Ring Buffer resume position

// Added to StreamResponse:
bool is_queued = 4;          // Quota queuing indicator
int32 queue_position = 5;    // Queue position (0 = not queued)
uint64 offset = 6;           // Ring Buffer byte offset

// Added to ChallengeRequest:
bytes client_ekm_hmac = 2;   // HMAC of TLS Exporter keying material
```

## Appendix F: ASCII State Machine Reference

```
                    ┌─────────────────────────────────────────────┐
                    │          ExecuteRequest Received             │
                    └─────────────┬───────────────────────────────┘
                                  │
                    ┌─────────────▼───────────────────────────────┐
                    │  L1: CIDR ACL Check                         │
                    │  IP in whitelist?                            │
                    └──────┬──────────────────────────────┬───────┘
                       YES │                              │ NO
                           │                              ▼
                           │                     ╔════════════════╗
                           │                     ║  R6: DROP      ║
                           │                     ╚════════════════╝
                    ┌──────▼──────────────────────────────────────┐
                    │  L2: TLS 1.3 + EKM Binding                 │
                    │  Valid TLS session?                          │
                    └──────┬──────────────────────────────┬───────┘
                       YES │                              │ NO
                           │                              ▼
                           │                     ╔════════════════╗
                           │                     ║  R6: DROP      ║
                           │                     ╚════════════════╝
                    ┌──────▼──────────────────────────────────────┐
                    │  L3: ED25519 Session Auth                   │
                    │  Valid session?                              │
                    └──────┬──────────────┬──────────────┬────────┘
                     VALID │       EXPIRED│              │ NONE
                           │              ▼              ▼
                           │     ╔════════════════╗  ╔═══════════╗
                           │     ║ R2: CHALLENGE  ║  ║ R6: DROP  ║
                           │     ╚════════════════╝  ╚═══════════╝
                    ┌──────▼──────────────────────────────────────┐
                    │  L4: Capability Check                       │
                    │  Command + Path within boundary?            │
                    └──────┬──────────────────────────────┬───────┘
                    WITHIN │                              │ BEYOND
                           │                              ▼
                           │                     ╔════════════════╗
                           │                     ║  R4: VI_SWAP   ║
                           │                     ╚════════════════╝
                    ┌──────▼──────────────────────────────────────┐
                    │  L5: 13Policy Check                         │
                    │  Command matches dangerous pattern?         │
                    └──────┬──────────┬──────────┬───────┬────────┘
                      NONE │  CRITICAL│     HIGH │   LOW │
                           │          ▼     /MED ▼       │
                           │  ╔════════════╗╔═══════════╗│
                           │  ║ R5: TARPIT ║║R2: CHALL. ║│
                           │  ╚════════════╝╚═══════════╝│
                    ┌──────▼──────────────────────────────▼───────┐
                    │  L6: Quota Check                            │
                    │  Slots available?                            │
                    └──────┬──────────────────────────────┬───────┘
                       YES │                              │ NO
                           │                              ▼
                           │                     ╔════════════════╗
                           │                     ║  R3: THROTTLE  ║
                           │                     ╚════════════════╝
                    ┌──────▼──────────────────────────────────────┐
                    │  R1: EXECUTE                                │
                    │  ┌─────────────────────────────────────┐    │
                    │  │ Transparent Branching (§6.6)        │    │
                    │  │ Volatile Cache Redirect (§6.6.2)    │    │
                    │  │ Dual Watchdog (§7.3)                │    │
                    │  │ ED25519 Audit Chain (§10.3)         │    │
                    │  └─────────────────────────────────────┘    │
                    └─────────────────────────────────────────────┘
```

---

*End of draft-saki-sakissh-protocol-02*
