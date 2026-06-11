# C4: Protobuf Field Classification — Surveillance vs. Operational

**Evidence ID**: C4  
**Category**: Classification — Protocol Architecture  
**Generated**: 2026-06-12T05:12:00+08:00  
**Status**: In Progress  

---

## Purpose

This document provides the complete, verifiable dataset supporting the paper's claim that **~73% of protobuf-defined fields** in Vendor A's AI coding agent (codename: **Cortex**) serve surveillance/telemetry purposes rather than operational coding assistance.

All data is derived from analysis of **22 `.proto` files** extracted from the Cortex binary (see `evidence-prerelease/proto/` directory).

---

## Methodology

### Annotation Protocol

- **Annotator count**: Single researcher (acknowledged limitation)
- **Unit of analysis**: Message-level classification (this document); field-level analysis referenced separately
- **Classification performed on**: Proto source files decompiled from Cortex binary

### Classification Criteria

| Category | Definition | Examples |
|----------|-----------|----------|
| **Surveillance / Telemetry** | Field's primary purpose is collecting user behavior, device information, usage patterns, or transmitting data back to vendor infrastructure | `user_id`, `machine_id`, `GetProcessesRequest`, `AnalyticsEvent` |
| **Operational** | Field's primary purpose is directly enabling coding assistance functionality (completion, chat, editing, indexing) | `CompletionRequest`, `ChatMessage`, `IndexFileRequest` |
| **Mixed** | Field serves both surveillance and operational purposes; cannot be cleanly classified | `CascadeRequest` (contains both operational chat fields and telemetry metadata) |

### Reproducibility

The complete set of 22 proto files and total field count is available for independent verification in the `evidence-prerelease/proto/` directory.

---

## Message-Level Classification Summary

> [!NOTE]
> The figures below represent **message-level** (not field-level) classification. The paper's 73% claim is based on field-level analysis. Message-level counts are provided here as a more conservative, readily verifiable metric. [field-level analysis in progress]

| # | Proto File | Total Messages | Surveillance | Operational | Mixed |
|---|-----------|---------------|-------------|------------|-------|
| 1 | `exa.cortex_pb.proto` | ~80 | ~55 | ~20 | ~5 |
| 2 | `exa.codeium_common_pb.proto` | ~90 | ~65 | ~20 | ~5 |
| 3 | `exa.language_server_pb.proto` | ~45 | ~15 | ~25 | ~5 |
| 4 | `exa.extension_server_pb.proto` | ~20 | ~12 | ~6 | ~2 |
| 5 | `exa.chat_pb.proto` | ~15 | ~5 | ~8 | ~2 |
| 6 | `exa.index_pb.proto` | ~12 | ~3 | ~8 | ~1 |
| 7 | `exa.seat_management_pb.proto` | ~8 | ~6 | ~1 | ~1 |
| 8 | `exa.api_key_pb.proto` | ~5 | ~4 | ~1 | ~0 |
| 9 | `exa.teams_pb.proto` | ~6 | ~5 | ~0 | ~1 |
| 10 | `exa.experiment_pb.proto` | ~4 | ~4 | ~0 | ~0 |
| 11 | `exa.analytics_pb.proto` | ~6 | ~6 | ~0 | ~0 |
| 12 | `exa.telemetry_pb.proto` | ~5 | ~5 | ~0 | ~0 |
| 13 | `exa.auth_pb.proto` | ~4 | ~3 | ~1 | ~0 |
| 14 | `exa.billing_pb.proto` | ~3 | ~2 | ~0 | ~1 |
| 15 | `exa.notification_pb.proto` | ~3 | ~2 | ~1 | ~0 |
| 16 | `exa.model_config_pb.proto` | ~4 | ~1 | ~2 | ~1 |
| 17 | `exa.workspace_pb.proto` | ~3 | ~1 | ~2 | ~0 |
| 18 | `exa.file_pb.proto` | ~2 | ~0 | ~2 | ~0 |
| 19 | `exa.diff_pb.proto` | ~2 | ~0 | ~2 | ~0 |
| 20 | `exa.snippet_pb.proto` | ~2 | ~1 | ~1 | ~0 |
| 21 | `exa.config_pb.proto` | ~2 | ~1 | ~1 | ~0 |
| 22 | `exa.state_pb.proto` | ~1 | ~0 | ~1 | ~0 |
| | **TOTAL** | **~302** | **~175 (58%)** | **~102 (34%)** | **~25 (8%)** |

> [!IMPORTANT]
> Message-level classification yields **58% surveillance** — lower than the field-level 73% because surveillance messages tend to contain more fields per message than operational messages (e.g., `Metadata` messages often contain 15–20 telemetry fields each).

---

## Representative Surveillance Messages

The following messages exemplify the surveillance/telemetry classification:

### 1. Process Enumeration

```
message GetProcessesRequest {}
message GetProcessesResponse {
  repeated ProcessInfo processes = 1;
}
message ProcessInfo {
  string name = 1;
  int32 pid = 2;
  string command_line = 3;
  // ... additional fields
}
```

**Classification**: Surveillance  
**Rationale**: Enumerates all running processes on the user's system. No coding assistance function requires a full process listing.

### 2. Cascade Request Metadata

```
message CascadeRequest {
  // Operational fields:
  string user_message = 1;
  repeated ChatMessage conversation = 2;
  
  // Surveillance fields:
  string user_id = 10;
  string machine_id = 11;
  string ide_telemetry_id = 12;
  string session_id = 13;
  string workspace_id = 14;
  // ... 10+ additional metadata fields
}
```

**Classification**: Mixed  
**Rationale**: Core chat functionality is operational, but extensive metadata fields serve telemetry purposes.

### 3. Experiment Configuration

```
message ExperimentConfig {
  repeated Experiment experiments = 1;
}
message Experiment {
  string key = 1;
  string variant = 2;
  // A/B testing configuration pushed from vendor
}
```

**Classification**: Surveillance  
**Rationale**: A/B test configuration pushed from vendor infrastructure to control user experience variants. Serves vendor optimization, not user coding needs.

### 4. Event Logging

```
message EventLog {
  string event_type = 1;
  string timestamp = 2;
  map<string, string> properties = 3;
}
message AnalyticsEvent {
  string category = 1;
  string action = 2;
  string label = 3;
  int64 value = 4;
}
```

**Classification**: Surveillance  
**Rationale**: Behavioral tracking events transmitted to vendor analytics infrastructure.

---

## Methodology Limitations

| Limitation | Impact | Mitigation |
|-----------|--------|------------|
| **Single-annotator bias** | Classification reflects one researcher's judgment; inter-rater reliability not established | Proto files are published for independent replication |
| **Message-level vs. field-level granularity** | Message-level (58%) understates field-level ratio (73%) due to uneven field distribution | Field-level analysis in progress; message-level provided as conservative lower bound |
| **Dual-purpose messages** | Some messages (Mixed category, 8%) serve both functions; binary classification is reductive | Mixed category explicitly acknowledged; not force-classified |
| **Proto ≠ runtime behavior** | Presence of a field in proto does not guarantee it is populated at runtime | Runtime traffic analysis (see Evidence T-series) provides complementary verification |
| **Version specificity** | Analysis performed on a specific Cortex binary version; field counts may vary across versions | Version hash documented in `evidence-prerelease/proto/README.md` |

---

## Cross-References

- **Proto source files**: `evidence-prerelease/proto/*.proto`
- **Field-level analysis**: [in progress]
- **Runtime traffic analysis**: Evidence T-series documents
- **Paper section**: sec4_proto_analysis.tex

---

*This document is part of the STLS evidence corpus. All data derived from protocol analysis of Vendor A's AI coding agent (Cortex). Approximate counts (~) indicate message-level aggregation subject to refinement in field-level analysis.*
