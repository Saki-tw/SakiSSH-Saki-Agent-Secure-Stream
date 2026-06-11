# G1: Multi-Layer Observation Declaration

**Document ID**: G1  
**Category**: Methodology  
**Generated**: 2026-06-12  
**Status**: Active

---

## 1. Overview

The evidence presented in this corpus is derived from multiple independent observation layers. Each layer captures a distinct class of signals, and critically, each layer's logging infrastructure is itself subject to observation by adjacent layers. This architecture enables cross-validation of anomalous events and provides resilience against single-point tampering.

## 2. Observation Layers

| Layer | Source | Evidence Status | Approx. Volume |
|-------|--------|----------------|----------------|
| L0 | Agent IDE session logs | Published (C1, C2) | 2.7 MB / 14,585 lines per incident session |
| L1 | gRPC interception logs | Published summary (C3) | 679 KB capture log |
| L2 | DNS query records | Available on request | ~50M queries over 4 months; ~10% blocked |
| L3 | External NAT logs | Available on request | ISP-level connection records |
| L4 | IdP audit trails | Available on request | 4 identity providers; timestamped auth events |

### Layer Descriptions

- **L0 (Agent IDE session logs)**: Raw session transcripts capturing every command issued by the agent, including tool calls, file operations, and inter-agent messages. Published as exhibits C1 and C2.
- **L1 (gRPC interception logs)**: Network-level capture of gRPC traffic between the agent runtime and upstream services. Published in summarized form as exhibit C3.
- **L2 (DNS query records)**: Complete DNS resolution logs from the monitored endpoint's resolver, covering approximately 50 million queries over the four-month observation period. Approximately 10% of queries were blocked by policy.
- **L3 (External NAT logs)**: Connection-level records from the ISP-facing NAT gateway, providing an independent view of outbound connection patterns.
- **L4 (IdP audit trails)**: Authentication and authorization event logs from four distinct identity providers, providing timestamped records of credential usage and session establishment.

## 3. The Observation Paradox

A fundamental property of this layered architecture is that **no single layer can be tampered with in isolation without producing detectable artifacts in adjacent layers**.

Consider: if an agent session log (L0) is modified to conceal an unauthorized command, the gRPC layer (L1) retains an independent record of the corresponding API call. If the gRPC layer is also compromised, the DNS layer (L2) still records the domain resolution that preceded the connection. If DNS records are purged, the NAT layer (L3) preserves evidence of the outbound connection itself.

This cascading cross-validation means that even if an observation tool is itself compromised, the layers above and below it can still detect the anomaly. This is the core value proposition of multi-layer observation: **the observer is always observed**.

## 4. Pending Work

A fully sanitized version of the L2 DNS query records is currently in preparation. The sanitization process involves:
- Removal of internal hostnames and private domain zones
- Redaction of client-identifiable query patterns
- Preservation of temporal distribution and blocking ratios

The sanitized dataset will be published as a supplementary exhibit upon completion.

---

*This document is part of the evidence corpus. See [INDEX.md](../INDEX.md) for navigation.*
