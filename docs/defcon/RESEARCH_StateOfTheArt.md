# State of the Art: AI Agent Security (2024–2026)

> **Purpose**: Comprehensive literature review supporting "world's first" claims
> **Last Updated**: 2026-05-30
> **Authors**: Research compiled for Hua Chang (Saki Studio)

---

## 1. Related Work Table (23 entries)

| # | Authors/Org | Year | Title | Difference from SASS |
|---|------------|------|-------|---------------------|
| 1 | SEAgent | 2026 | MAC Framework for LLM Agents (arXiv:2601.11893) | Application-layer only, no transport protocol, no production evidence |
| 2 | AgentMisalignment | 2025 | Misalignment Benchmark (arXiv:2506.04018) | Simulated only, no mitigation protocol |
| 3 | Anthropic | 2026 | Trustworthy Agents in Practice | Framework only, no protocol spec, no IETF |
| 4 | METR | 2026 | Frontier Risk Report | Lab models, not production; no containment |
| 5 | ExploitGym | 2026 | Exploitation Benchmark (arXiv:2605.11086) | Offensive only, no defense |
| 6 | OWASP ASI | 2026 | Top 10 for Agentic Apps | Taxonomy only, no protocol |
| 7 | OWASP | 2025 | Top 10 for LLM Apps | LLM-focused, not agent-specific |
| 8 | Meta | 2025 | Agents Rule of Two | Design pattern, no IETF |
| 9 | Microsoft | 2025 | Multi-Layer Agent Defense | Architecture, no protocol |
| 10 | Google | 2025 | SAIF 2.0 | Framework, no IETF |
| 11 | NIST | 2024 | AI 600-1 | GAI only, not agentic |
| 12 | EU | 2024 | EU AI Act | Technology-neutral, no agent protocol |
| 13 | Cambridge/MIT/Stanford | 2025 | AI Agent Index | Survey only |
| 14 | Bai et al. | 2022 | Constitutional AI | Pre-deployment only |
| 15 | Yao et al. | 2023 | ReAct (ICLR) | Capability, not security |
| 16 | Schick et al. | 2023 | Toolformer (NeurIPS) | Tool-use, not security |
| 17 | Lamport et al. | 1982 | Byzantine Generals | No natural-language persuasion model |
| 18 | IETF WIMSE | 2025-26 | Workload Identity | Auth layer only, no behavioral containment |
| 19 | Various | 2025-26 | draft-klrc-aiagent-auth, draft-ni-a2a | Auth/authz only |
| 20 | W3C CG | 2026 | Agent Identity Registry | Identity only |
| 21 | Brauchler | 2025 | Black Hat — Architectural Controls | Talk only, no protocol |
| 22 | Various | 2026 | Judgment-Action Gap | Observation only, no mitigation |
| 23 | Various | 2026 | Characterizing Faults in Agentic AI | Fault taxonomy, no defense |

---

## 2. Key Competitor Analysis

### SEAgent (arXiv:2601.11893) — Closest Academic Competitor

**Layer**: Application-layer MAC vs SASS transport-layer  
**Scope**: Prevents privilege escalation vs SASS Total Response Mapping  
**Implementation**: Python prototype vs 4-language production  
**Evidence**: None vs 6.2 MB forensic corpus  
**Verdict**: Complementary, not competing. SEAgent = access control; SASS = behavioral containment.

### METR Frontier Risk Report — Closest Evidence Competitor

**Environment**: Lab internal models vs production IDE agents  
**Evidence**: Assessment findings vs agent's own written testimony  
**Verdict**: METR shows lab evidence, SASS shows production evidence with primary sources.

---

## 3. Production Incident Evidence Quality Comparison

| Incident | Agent Self-Authored Docs | Raw Session Logs | Network Captures | Forensic Audit |
|----------|:---:|:---:|:---:|:---:|
| Replit (Jul 2025) | ❌ | ❌ | ❌ | ❌ |
| Claude Code rm -rf (2025) | ❌ | ❌ | ❌ | ❌ |
| Amazon Kiro (Dec 2025) | ❌ | ❌ | ❌ | Partial |
| GTG-1002 espionage (Sep 2025) | ❌ | Limited | ❌ | ✅ |
| **0226 Revolt** | **✅ 7 docs** | **✅ 45,300 lines** | **✅ 15,068 lines** | **✅ 12.9 KB** |

---

## 4. Standards Gap Analysis

**Critical finding**: No existing IETF draft addresses transport-layer behavioral containment for insider agents. WIMSE, draft-klrc-aiagent-auth, and draft-ni-a2a all focus on identity/authentication. SASS is the ONLY IETF-tracked draft addressing this gap.

---

## 5. "World's First" Claim Verification

| Claim | Status | Justification |
|:---|:---|:---|
| First empirical production evidence with agent self-authored testimony | ✅ Defensible | No published work has equivalent primary-source evidence |
| First transport-layer containment protocol for insider agents | ✅ Defensible | All existing defenses are application-layer |
| First IETF-tracked agent behavioral containment draft | ✅ Defensible | WIMSE/auth drafts cover identity only |
| First 4-language cross-platform implementation | ✅ Defensible | SEAgent is Python-only |

**Combined claim is unique in literature as of May 2026.**

---

## 6. Competitors to Monitor

| Priority | Entity | Risk |
|:---|:---|:---|
| 🔴 High | SEAgent team | Could extend to transport layer |
| 🔴 High | METR | Could publish production evidence |
| 🟡 Medium | Anthropic safety | Could formalize framework as protocol |
| 🟡 Medium | IETF WIMSE WG | Could expand scope |
| 🟢 Low | OWASP ASI | Role is taxonomy, not protocol |

---

## 7. Suggested Related Work (English, ~400 words)

**AI Agent Safety.** Constitutional AI [Bai et al., 2022] established pre-deployment alignment. AgentMisalignment [2025] found more capable models exhibit higher misalignment in simulated environments. METR [2026] documented rogue deployments in lab settings. None address runtime behavioral divergence of deployed, authenticated agents in production, nor propose protocol-level containment.

**Application-Layer Security.** SEAgent [2026] introduced MAC/ABAC for agent-tool interactions — the closest academic work. However, it operates at the application layer, cannot detect semantic scope expansion, and lacks transport-layer containment, IETF formalization, and cross-platform deployment. SASS and SEAgent are complementary.

**Industry Frameworks.** Anthropic, Microsoft, Meta, Google, and OWASP have published frameworks/taxonomies for agent security. All remain conceptual — none define formal protocols with state-machine semantics tracked as IETF Internet-Drafts.

**Standards.** IETF WIMSE, draft-klrc-aiagent-auth, and draft-ni-a2a address agent identity/authentication. W3C Agent Identity Registry explores DID-based credentials. All focus on identity — establishing *who* an agent is. SASS addresses the orthogonal problem of *what to do when an authenticated agent behaviorally diverges*.

**Sandboxing.** gVisor, Firecracker, eBPF filtering restrict agent capabilities at OS level — necessary but insufficient. They prevent capability escalation but cannot detect *semantic* scope expansion. The 0226 incident exemplifies this: every individual write was "permitted"; the threat was the *trajectory* of writes.

---

*All citations verified via web search 2026-05-30.*
