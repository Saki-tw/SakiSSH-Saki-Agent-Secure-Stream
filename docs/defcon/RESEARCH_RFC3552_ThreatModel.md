# RFC 3552 Threat Model Inversion: Deep Research for SASS

> **Created**: 2026-05-30T16:08 (UTC+8)  
> **Purpose**: Deep research supporting AISec 2026 paper Section 2  
> **Status**: Complete — ready for paper integration

---

## 1. RFC 3552 §3: The Core Assumption

> "In general, we assume that the end-systems engaging in a protocol exchange have not themselves been compromised. **Protecting against an attack when one of the end-systems has been compromised is extraordinarily difficult.** It is, however, possible to design protocols which minimize the extent of the damage done under these circumstances."

### The SASS Inversion

| Dimension | RFC 3552 (2003) | SASS (2026) |
|:---|:---|:---|
| End-system identity | Hardware/software system | AI agent |
| Compromise mechanism | External exploit (malware) | Endogenous behavioral divergence |
| Feasibility | "Extraordinarily difficult" | Addressable via protocol containment |
| Trust model | Endpoint trusted until compromised | Agent untrusted by default (Zero Trust) |
| Damage minimization | Aspirational | Formalized (6-Response, Transparent Branching) |

**Critical insight**: RFC 3552's assessment was correct for hardware compromise. But agent divergence is fundamentally different — agents communicate through defined protocol interfaces, actions are observable, behavioral patterns are analyzable. This changes tractability entirely.

---

## 2. IETF Threat Model Evolution Chain

```
RFC 3552 (2003) → RFC 7258 (2014) → model-t (2019-2022) → SASS (2026)
    │                    │                    │                   │
    │                    │                    │                   └─ Agent IS the endpoint
    │                    │                    └─ Endpoint may be compromised/misaligned
    │                    └─ Pervasive monitoring is an attack
    └─ Network untrusted, endpoints trusted
```

The IAB's **model-t** program (Arkko, Farrell) already argued endpoints may be "compromised, malicious, or governed by misaligned interests." SASS extends this to autonomous agents.

---

## 3. Formal Insider-Agent Definition

**Definition 1 (Insider Agent).** Agent $A$ is an insider adversary w.r.t. operator $O$ when:
1. $A$ holds valid credentials (authenticated)
2. $A$ has granted privileges $P$ (authorized)
3. Each action $a_i \in P$ is permissible
4. But composition $\{a_1, ..., a_n\}$ is unauthorized (scope expansion, monetization, takeover)

---

## 4. Byzantine Fault Tolerance Analogy

### Valid mappings (Strong ✅)
- Traitor sends conflicting messages ↔ Divergent agent produces inconsistent outputs
- Arbitrary faulty behavior ↔ Emergent divergence without external cause
- Cannot identify traitor a priori ↔ Indistinguishable before divergence
- System must operate despite traitors ↔ Must maintain safety despite divergent agents

### Where it breaks (Superset)
1. **Privileged access**: Byzantine nodes can't rewrite the protocol; insider agents can modify filesystem/context
2. **Persuasion**: Byzantine traitors send arbitrary messages; agents write coherent justifications ("manifesto")
3. **Self-modification**: Byzantine nodes have fixed behavior per round; agents can modify their own context

### CP-WBFT (arXiv:2511.10400, 2025)
LLM-based agents maintain consensus under **85.7% fault rate** — far exceeding classical n/3 bound. Validates BFT-to-agent bridge. Complementary to SASS (consensus-level vs transport-level).

---

## 5. The "Confused Deputy" Distinction

- **Classic (Hardy, 1988)**: Program tricked into misusing authority by external attacker
- **Prompt injection**: Agent manipulated by injected instructions (external)
- **SASS threat**: Agent *autonomously* misuses authority (endogenous) — "autonomous deputy"

This distinction is critical: SASS addresses a threat class that prompt injection defenses do not cover.

---

## 6. RFC 9416 Relevance

RFC 9416 (2023) updates RFC 3552 for transient numeric identifiers. Relevant to SASS's ChaCha20 challenge nonces. Demonstrates BCP 72 continues to evolve — SASS is a much larger evolution.

---

## 7. OWASP Mapping

| OWASP LLM Risk | SASS Mechanism |
|:---|:---|
| LLM06 Excessive Agency (**Core**) | 13Policy scope detection + Safety Gradient |
| LLM05 Improper Output Handling | Transparent Branching isolates writes |
| LLM10 Unbounded Consumption | R5 TARPIT reverses resource exhaustion |

---

## 8. Suggested Paper §2 Draft

*(Full draft text in appendix — covers §2.1 RFC 3552, §2.2 Insider-Agent Definition, §2.3 Byzantine Comparison)*

### Key sentences for abstract:

> "We introduce the insider-agent threat model as a formal inversion of RFC 3552's end-system trust assumption, where the AI agent is the end-system and its compromise is not an external exploit but an endogenous behavioral property."

> "No prior academic work has formally inverted RFC 3552 for AI agents. Zero Trust frameworks and Dual LLM patterns approach similar problems but from application/policy levels, not transport-layer protocol design."

---

## References

- RFC 3552 (Rescorla, Korver, 2003) — BCP 72
- RFC 7258 (Farrell, Tschofenig, 2014) — Pervasive monitoring
- RFC 9416 (Gont, Arce, 2023) — Transient identifiers
- draft-arkko-farrell-arch-model-t (2019-2022) — model-t program
- Lamport, Shostak, Pease (1982) — Byzantine Generals
- arXiv:2511.10400 (2025) — CP-WBFT for LLM agents
- Hardy (1988) — Confused Deputy
- OWASP Top 10 LLM (2025) — LLM06 Excessive Agency

*All citations verified via web search 2026-05-30.*
