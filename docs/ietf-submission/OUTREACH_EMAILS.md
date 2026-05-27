# SASS RFC 審查推動信件

> **用途**：提交後主動推動審查
> **時間**：2026-05-26
> **Draft**：draft-sakistudio-sass-00

---

## 信件一：寄給 sshm WG 郵件列表

**收件人**：`sshm@ietf.org`
**主旨**：`[sshm] New Individual I-D: draft-sakistudio-sass-00 — Agent-oriented secure remote execution protocol`

```
Hi all,

I'd like to introduce a new Individual Internet-Draft that addresses
a gap in secure remote execution for autonomous AI agents:

  draft-sakistudio-sass-00: SakiAgentSSH Secure Protocol Specification
  https://datatracker.ietf.org/doc/draft-sakistudio-sass/

Problem statement:

The proliferation of autonomous AI-powered coding agents operating on
remote machines introduces a critical threat model — the Rogue Agent.
Unlike traditional SSH clients controlled by human operators, agents
may autonomously execute destructive commands, exfiltrate credentials,
or pivot laterally across networks without explicit human
authorization.

Existing remote execution protocols such as SSH (RFC 4251-4254) were
designed for human-operated terminals and lack the fine-grained
capability controls, active defenses, and binary-safe encoding
schemes required for agent management.

What SASS provides:

- Total Response Mapping: a 6-Response state machine (R1-R6) that
  maps every possible agent behavior to a deterministic, bounded
  response — eliminating the concept of "unexpected behavior"

- Safety Gradient: 7-layer loss bounding where each protocol layer
  independently guarantees storage integrity even if all layers
  above are compromised

- Active defense mechanisms: Vi Swap (traps authenticated agents
  in simulated terminal state), Zero-Allocation Tarpit (exhausts
  attacker resources at O(1) daemon cost)

- Transparent Branching: zero-loss write isolation invisible to the
  agent, requiring explicit human review for merge

- Control-Transport Decoupling: transport-agnostic messaging core
  (SAMM) with pluggable transport profiles (gRPC/h2, WebSocket,
  raw TCP/CBOR)

- MAS (Martingale Almost-Surely Superior): formal version comparison
  based on Second-order Stochastic Dominance (Aumann-Serrano, 2008)

Relationship to SSH:

SASS shares NO wire format, key exchange, channel multiplexing, or
subsystem architecture with SSH. The name "SakiAgentSSH" is a
historical development codename. However, the threat model SASS
addresses — autonomous agents on remote machines — is directly
relevant to the SSH ecosystem, as SSH is currently the most common
transport used by coding agents (GitHub Copilot, Claude Code,
Cursor, etc.) despite its unsuitability for non-human operators.

Reference implementation (Rust + Go) is available at:
https://github.com/Saki-tw/SakiSSH-Saki-Agent-Secure-Stream

The implementation has been deployed across four protocol versions
(v1.1-v1.4) and verified via cargo check with 0 errors.

I welcome any feedback and would be interested in whether the WG
considers this relevant to its scope.

Best regards,
Hua Chang
Saki Studio
Saki@saki-studio.com.tw
https://saki-studio.com.tw
```

---

## 信件二：寄給 SEC Area Directors

**收件人**：Security Area Directors（查 https://www.ietf.org/about/groups/iesg/ 取最新 email）
- Deb Cooley
- Christopher Inacio

**主旨**：`New Individual I-D: draft-sakistudio-sass-00 — Seeking venue guidance`

```
Dear Deb and Christopher,

I have submitted an Individual Internet-Draft,
draft-sakistudio-sass-00, and would appreciate your guidance on
the most appropriate venue for its progression.

The draft defines the Saki Agent Secure Stream (SASS) protocol, a
security framework specifically designed for autonomous AI agent
remote execution — a threat model that existing protocols (SSH,
Mosh, etc.) do not address.

Key contributions:

1. Total Response Mapping: Every possible agent behavior maps
   deterministically to one of six bounded responses. This is
   grounded in Rice's Theorem (1953) — since static analysis
   cannot determine whether an arbitrary command sequence is
   "safe," SASS guarantees bounded responses instead.

2. Formal version comparison (MAS) based on Aumann-Serrano (2008)
   Second-order Stochastic Dominance, providing a rigorous
   framework for protocol evolution claims.

3. Privacy Considerations addressing the unique position of AI
   agents as entities that simultaneously exist as userspace
   processes (with privacy expectations) and components of
   commercial hyperscale services (with GDPR/CCPA recoverability
   constraints).

The draft is intended as Experimental status. A reference
implementation in Rust and Go is publicly available.

I believe this may be relevant to the sshm Working Group given its
relationship to agent-based remote execution, but I am open to your
recommendation — including Independent Submission if you consider
that more appropriate.

Draft: https://datatracker.ietf.org/doc/draft-sakistudio-sass/
Implementation: https://github.com/Saki-tw/SakiSSH-Saki-Agent-Secure-Stream
Website: https://saki-studio.com.tw/sakiagentssh/

Best regards,
Hua Chang
Saki Studio
Saki@saki-studio.com.tw
+886-988-403-884
```

---

## 信件三（備用）：寄給 ISE（走 Independent Stream 時用）

**收件人**：`rfc-ise@rfc-editor.org`
**主旨**：`Request for Independent Stream review: draft-sakistudio-sass-00`

```
Dear Independent Submissions Editor,

I would like to request Independent Stream review for
draft-sakistudio-sass-00, "SakiAgentSSH Secure Protocol
Specification."

This document defines a security protocol for autonomous AI agent
remote execution, addressing the Rogue Agent threat model. It
introduces Total Response Mapping (a 6-state deterministic response
machine), formal protocol version comparison based on Second-order
Stochastic Dominance, and active defense mechanisms including
Zero-Allocation Tarpit and Vi Swap.

The draft is intended as Experimental status. A reference
implementation in Rust and Go is publicly available at:
https://github.com/Saki-tw/SakiSSH-Saki-Agent-Secure-Stream

I am pursuing this via Independent Stream as the protocol, while
related to the SSH agent ecosystem, defines an entirely new wire
format and does not extend SSH directly.

Draft: https://datatracker.ietf.org/doc/draft-sakistudio-sass/

I welcome your assessment of whether this document is suitable for
the Independent Stream.

Best regards,
Hua Chang
Saki Studio
Saki@saki-studio.com.tw
```

---

## 推薦發送順序

| 順序 | 對象 | 時機 |
|:----:|------|------|
| 1 | sshm WG (`sshm@ietf.org`) | Draft 公告後立即 |
| 2 | SEC Area Directors | 同時或隔天 |
| 3 | ISE (`rfc-ise@rfc-editor.org`) | 若 2 週內 WG 無回應 |

> **注意**：寄信前先訂閱 sshm 郵件列表 https://www.ietf.org/mailman/listinfo/sshm
> 否則你的信會被 moderation queue 擋住。
