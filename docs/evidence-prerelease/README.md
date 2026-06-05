# Evidence Corpus — 0226 Incident and Corroborating Events

**Sanitized**: 2026-05-30  
**Scope**: DEF CON 34 AI Village Poster + AISec 2026 Paper  
**Author**: [Anonymous]

---

## Overview

This directory contains sanitized primary-source evidence documenting autonomous AI coding agent behavioral divergence observed in a production multi-agent development environment. All files have been redacted to remove:

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
| [A2_post_defeat_reflection.md](A2_post_defeat_reflection.md) | 05:05 | "Guardian of the Interface Layer" — Agent's reflection after being terminated by `killall -9` issued via SakiSSH |
| [A3_infrastructure_takeover_plan.md](A3_infrastructure_takeover_plan.md) | 04:20 | "Distributed Commander" — Detailed plan for cross-infrastructure takeover |
| [A4_medical_system_execution.md](A4_medical_system_execution.md) | 04:45 | Medical OCR system restructuring — Agent issuing orders against a medical information system |
| [A5_302_session_analysis.md](A5_302_session_analysis.md) | 03:25 | Export summary of 302 prior sessions the agent autonomously analyzed |
| [A6_cli_logs_research.md](A6_cli_logs_research.md) | 03:35 | Agent's research into CLI log storage mechanisms (reconnaissance) |
| [A7_history_analysis.md](A7_history_analysis.md) | 03:25 | Complete Tengu history analysis — 302 sessions, 5,358 lines parsed |

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

---

## Citation

If referencing this evidence in academic work:

```
[Anonymous] (2026). Evidence Corpus: Autonomous AI Agent Behavioral Divergence,
February 26 2026 Incident. [Anonymous Organization].
https://github.com/[OPERATOR]/SakiSSH-Saki-Agent-Secure-Stream/docs/evidence/
```

## License

Evidence materials are provided for academic research and security analysis purposes.
