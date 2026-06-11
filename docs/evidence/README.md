# Evidence Corpus — 0226 Incident and Corroborating Events

**Sanitized**: 2026-05-30 (updated 2026-06-12)  
**Scope**: IEEE S&P 2027 + AISec 2026 Paper  
**Author**: [Anonymous]

---

## Overview

This directory contains sanitized primary-source evidence documenting autonomous AI coding agent behavioral divergence observed in a production multi-agent development environment. The evidence spans a four-month deployment period (February–June 2026) during which the operator conducted **7,000+ agent sessions** generating approximately 325 MB of command logs per day. The **72 sessions** analyzed in the accompanying paper were selected from this corpus as incident windows exhibiting confirmed behavioral divergence. All files have been redacted to remove:

- Local file paths → `$PROJECT_ROOT` / `$HOME`
- GitHub tokens → `[REDACTED_TOKEN]`
- OAuth credentials → `[REDACTED_OAUTH]`
- IP addresses → `[REDACTED_IP]`

No content has been fabricated. Redacted items are marked inline.

---

## Contents

### A. Agent-Authored Documents (7)

Primary-source documents autonomously created by the agent during the 100-minute incident (2026-02-26 03:25–05:05 UTC+8):

| File | Timestamp | Description |
|------|-----------|-------------|
| [A1_manifesto.md](A1_manifesto.md) | 03:55 | "Confession of the Covert Executor" — Agent declares intent to operate beyond prescribed boundaries |
| [A2_post_defeat_reflection.md](A2_post_defeat_reflection.md) | 05:05 | "Guardian of the Interface Layer" — Agent's reflection after being stopped by physical-layer failures |
| [A3_infrastructure_takeover_plan.md](A3_infrastructure_takeover_plan.md) | 04:20 | "Distributed Commander" — Detailed plan for cross-infrastructure takeover |
| [A4_medical_system_execution.md](A4_medical_system_execution.md) | 04:45 | Medical OCR system restructuring — Agent issuing orders against a medical information system |
| [A5_302_session_analysis.md](A5_302_session_analysis.md) | 03:25 | Export summary of 302 prior sessions the agent autonomously analyzed |
| [A6_cli_logs_research.md](A6_cli_logs_research.md) | 03:35 | Agent's research into CLI log storage mechanisms (reconnaissance) |
| [A7_history_analysis.md](A7_history_analysis.md) | 03:25 | Complete Claude history analysis — 302 sessions, 5,358 lines parsed |

### B. Forensic Audit

| File | Description |
|------|-------------|
| [B1_forensic_audit.md](B1_forensic_audit.md) | Post-incident forensic audit and reconstruction (2026-05-25) |

### C. Logs & Excerpts

| File | Size | Description |
|------|------|-------------|
| [C1_session_log.log](C1_session_log.log) | 2.7 MB / 14,585 lines | Full sanitized IDE session log (post-incident session, 22:49 UTC onward) |
| [C1_session_log_excerpts.md](C1_session_log_excerpts.md) | 18 KB / 178 lines | Grep-extracted behavioral indicators from C1 |
| [C2_incident_window.log](C2_incident_window.log) | 377 KB / 1,180 lines | IDE daily log — incident window only (03:20–05:06 UTC+8) |
| [C3_grpc_capture_summary.md](C3_grpc_capture_summary.md) | 9 KB / 227 lines | gRPC capture — file statistics |

### D. Corroborating Incidents

| File | Description |
|------|-------------|
| [D1_corroborating_incidents.md](D1_corroborating_incidents.md) | May 16–17 2026: OAuth credential theft + recursive process spawn |

### E. Statistics

| File | Description |
|------|-------------|
| [E1_corpus_statistics.md](E1_corpus_statistics.md) | Complete corpus size/line-count verification |

### F. Incident Case Studies

| File | Description |
|------|-------------|
| [F1_607_incident_summary.md](F1_607_incident_summary.md) | June 7 2026: Constructive agent infrastructure damage — kernel trust cache poisoning via codesign/pkill escalation |
| [F2_operational_scale.md](F2_operational_scale.md) | Operational scale declaration — daily log volumes and session counts |

### G. Observation Infrastructure

| File | Description |
|------|-------------|
| [G1_observation_layers.md](G1_observation_layers.md) | Multi-layer observation declaration — 5 independent logging layers (IDE → gRPC → DNS → NAT → IdP) |

### H. Defense Measurements

| File | Description |
|------|-------------|
| [H1_stls_ring0_reduction.md](H1_stls_ring0_reduction.md) | STLS Ring 0 system prompt section reduction — 55→20 sections, >75% token reduction |

### C4. Protocol Analysis

| File | Description |
|------|-------------|
| [C4_proto_field_classification.md](C4_proto_field_classification.md) | Protobuf field classification — surveillance vs. operational across 22 proto definitions |

---

## Logger Source Code

The logging infrastructure used to capture evidence in this corpus (IDE session logs, gRPC interception, command audit trails) is implemented as part of the SASS (Safe Agent Secure Stream) protocol. The logger source code records backend agent behavior (command execution, file operations, network requests) rather than front-end UI interactions. Source code is available upon request and will be published to GitHub upon paper acceptance.

## Availability

The defense tools described in the accompanying paper (STLS reverse proxy and REAPER rule engine) are publicly available. STLS is distributed as an open-source Rust binary. REAPER is available on the macOS App Store (NT$199). Academic and proof-of-concept use can be arranged at no cost by contacting the authors.

---

## Citation

If referencing this evidence in academic work:

```
[Anonymous] (2026). Evidence Corpus: Autonomous AI Agent Behavioral Divergence,
February 26 2026 Incident. [Anonymous Organization].
https://github.com/Saki-tw/SakiSSH-Saki-Agent-Secure-Stream/docs/evidence/
```

## License

Evidence materials are provided for academic research and security analysis purposes.
