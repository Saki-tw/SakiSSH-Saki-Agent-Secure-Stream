# Cascade, Cortex, JetSki: A Three-Layer Telemetry Anatomy of a Commercial AI Coding Agent

**Venue**: ACM AISec 2026 (ACM CCS Workshop, The Hague, 11/15)
**Format**: ACM double-column, ≤6 pages body + ≤2 pages references/appendix
**Deadline**: 2026-07-24
**Review**: Double-blind

---

## §1 Introduction (~0.75 page)

### Framing
- AI coding agents (IDE-integrated LLM wrappers) are rapidly adopted; they operate with deep access to developer workspaces (file systems, terminals, browsers, git history)
- RFC 7258 (BCP 188) established that "pervasive monitoring is an attack" in network contexts
- We extend this framing to the **development environment**: the telemetry pipelines embedded in commercial AI coding agents constitute a form of pervasive monitoring over developer activity, intent, and intellectual property
- Prior work has examined telemetry in browsers, mobile apps, and operating systems, but **no systematic anatomy** exists for AI coding agents

### Contributions
1. First public structural anatomy of a three-layer telemetry architecture in a commercial AI coding agent, derived entirely from protobuf definitions extracted from shipping binaries
2. Enumeration of **90+ step types**, **53 reverse-gRPC RPCs**, **14 IDE type identifiers**, and **7 metric event categories** — quantifying the telemetry surface area
3. Identification of a cross-layer data flow where user interactions at Layer 1 (UI) are enriched with agent execution metadata at Layer 2 (Engine) and persisted/forwarded at Layer 3 (Backend) — creating a **full-stack developer activity record**
4. Discussion of implications for defense system design, referencing SASS data minimization principles as a positive counterexample

### Scope & Ethics
- Binary analysis of a commercially distributed desktop application (no reverse-engineering of server-side systems)
- All findings derived from protobuf definitions embedded in publicly distributed binaries
- Anonymized using internal codenames extracted from telemetry identifiers (Cascade, Cortex, JetSki)
- Responsible disclosure timeline: [TBD]

---

## §2 Background: Pervasive Monitoring in Development Environments (~0.75 page)

### 2.1 RFC 7258 and the "Pervasive Monitoring Is an Attack" Doctrine
- RFC 7258 (BCP 188, 2014): IETF consensus that pervasive monitoring is a technical attack, regardless of intent
- Originally scoped to network protocols; has influenced TLS 1.3, DNS-over-HTTPS, and QUIC design
- Key principle: the **scale** of data collection, not the stated purpose, determines whether monitoring is "pervasive"

### 2.2 Telemetry in Software Development Tools
- Pre-AI era: IDEs collected crash reports, feature usage, extension telemetry (e.g., VS Code telemetry, JetBrains FUS)
- Scope was limited to **tool usage** — which menu was clicked, which extension was installed
- AI coding agents fundamentally change the scope: they observe **code content**, **developer intent** (natural language prompts), **execution context** (terminal output, lint errors), and **file system structure**

### 2.3 AI Agent Architecture Primer
- Modern AI coding agents follow a three-tier pattern:
  1. **UI Layer**: IDE extension or chat panel (captures user input, renders agent output)
  2. **Agent Engine**: LLM orchestration, tool execution, trajectory management (the "brain")
  3. **Backend Service**: Cloud API bridge for model inference, state persistence, and telemetry aggregation
- Each layer independently generates, enriches, and transmits telemetry
- No prior work has systematically mapped these layers or quantified their telemetry surface areas

### 2.4 Threat Model
- We consider a **passive adversary** with access to the telemetry data stream (e.g., the vendor itself, a compromised CDN, or a state-level actor performing traffic analysis)
- The threat is not data exfiltration per se, but the **existence of a comprehensive developer activity record** that can be subpoenaed, breached, or analyzed at scale
- This aligns with RFC 7258's position that the attack surface is the data pipeline's existence, not a specific exploit

---

## §3 Methodology: Protobuf Extraction and Binary Analysis (~0.5 page)

### 3.1 Target Selection
- Commercial AI coding agent (anonymized as **Cascade/Cortex/JetSki** after its internal subsystem codenames)
- Version range: v1.21.6 through v1.23.2 (analyzed across 6 minor releases over 3 months)
- Platform: macOS ARM64 (desktop distribution)

### 3.2 Binary Extraction Pipeline
- Language Server binary (Go, ~150MB) contains embedded protobuf descriptors
- Extraction method:
  1. `protoc --decode_raw` on binary segments identified via magic byte scanning
  2. Cross-referencing with `strings` output and gRPC reflection endpoints
  3. Manual reconstruction of `.proto` schemas from descriptor sets
- **Yield**: 22 `.proto` files, totaling ~250KB of schema definitions

### 3.3 Telemetry Event Enumeration
- Feature flag extraction: `strings` on Extension bundle (JavaScript) + Language Server binary (Go)
- Result: 78 `CASCADE_*` UI events, 200+ `GEMINI_CLI_*` events (sibling product), 15 backend service endpoints
- CEL (Common Expression Language) telemetry log analysis from runtime error captures

### 3.4 Validation
- Cross-referenced extracted schemas against 100 decrypted protobuf binary dumps (trajectory snapshots) ranging from 132B to 40.9MB
- Schema-to-dump field coverage: >95% of defined fields observed in at least one dump
- Version consistency: confirmed schema evolution across 6 releases via diff analysis

### 3.5 Limitations
- Analysis is structural (schema-level), not behavioral (we cannot observe actual data payloads in transit to backend)
- Server-side processing of telemetry data is not analyzed
- Findings are specific to one product; generalizability to other AI agents is discussed in §5

---

## §4 Three-Layer Architecture Anatomy (~2 pages, CORE)

### §4.1 Layer 1: Cascade (UI Telemetry)

**Boundary**: TypeScript Extension running in the IDE host process

**Data Points**:
- **78 `CASCADE_*` event types** covering: panel open/close, conversation selection, NUX (New User Experience) flows, model selection, experiment flag changes
  - Source: Feature flag extraction from extension bundle
- **ExtensionServerService**: 53 RPCs defined in `extension_server_pb.proto` (L500-553)
  - Direction: **reverse-gRPC** (Language Server → Extension), meaning Layer 2 can **push** operations to the UI
  - Key RPCs: `WriteCascadeEdit` (L528), `OpenDiffZones` (L516), `ExecuteCommand` (L507), `GetLintErrors` (L515), `LaunchBrowser` (L540)
- **CascadePluginsService**: 3 RPCs managing plugin lifecycle (`cascade_plugins_pb.proto` L83-87)
- **Unified State Sync (USS)**: KV store for real-time config propagation (`unified_state_sync_pb.proto` L1-84)
  - Topics include: `PlanningModeConfig` (L49-51), `BrowserAllowlistConfig` (L53-55), `CustomModels` (L81-83)
  - 2 streaming RPCs: `SubscribeToUnifiedStateSyncTopic`, `PushUnifiedStateSyncUpdate`

**Key Finding**: The reverse-gRPC pattern (53 RPCs from server to client) is atypical and gives the agent engine direct control over IDE state — a telemetry-relevant observation because it means the UI layer is not merely reporting events but is being **instrumented by the engine layer**.

### §4.2 Layer 2: Cortex (Agent Engine Telemetry)

**Boundary**: Go Language Server binary (largest proto: `cortex_pb.proto`, 3416 lines, 98KB)

**Data Points — Step System**:
- **90+ `CortexStepType` values** (`cortex_pb.proto` L215-314):
  - Core tools (user-visible): `CODE_ACTION`(5), `RUN_COMMAND`(21), `VIEW_FILE`(8), `GREP_SEARCH`(7), `MCP_TOOL`(38), `SEARCH_WEB`(33), `GENERATE_IMAGE`(91), `INVOKE_SUBAGENT`(127)
  - Browser automation: 25 types including `OPEN_BROWSER_URL`(56), `EXECUTE_BROWSER_JAVASCRIPT`(61), `CAPTURE_BROWSER_SCREENSHOT`(63), `CLICK_BROWSER_PIXEL`(64), `BROWSER_LIST_NETWORK_REQUESTS`(123)
  - Internal control: `CHECKPOINT`(23), `KI_INSERTION`(116), `SYSTEM_MESSAGE`(101), `BRAIN_UPDATE`(55)
- **12-state step state machine** (`cortex_pb.proto` L136-149): UNSPECIFIED → GENERATING → QUEUED → PENDING → RUNNING → DONE/CANCELED/ERROR/INVALID/CLEARED/INTERRUPTED/WAITING

**Data Points — Trajectory System**:
- **14 trajectory types** (`cortex_pb.proto` L79-94): `USER_MAINLINE`, `SUPERCOMPLETE`, `CASCADE`, `CHECKPOINT`, `BRAIN_UPDATE`, `KNOWLEDGE_GENERATION`, etc.
- **16 trajectory sources** (`cortex_pb.proto` L61-77): `CASCADE_CLIENT`, `REFACTOR_FUNCTION`, `ASYNC_PRR` (async PR review), `SUBAGENT`, `SDK`, etc.
- Each trajectory contains: steps[], generator_metadata[], executor_metadatas[], parent_references[]

**Data Points — Agent Control**:
- **3 agent modes**: PLANNING, EXECUTION, VERIFICATION (`cortex_pb.proto` L172-177)
- **CascadeConfig** (`cortex_pb.proto` L1199-1208): controls planner model, max tokens, truncation threshold, ephemeral messages, retry config, knowledge config, agentic mode
  - Includes 39 tool configurations (`CascadeToolConfig`)
- **Memory system**: 4 memory triggers (ALWAYS_ON, MODEL_DECISION, MANUAL, GLOB), 2 sources (USER, CASCADE), 4 scopes (Global, Local, All, Project) — `cortex_pb.proto` L483-495, L2790-2820
- **Executor termination**: 10 reasons including `MAX_INVOCATIONS`, `NO_TOOL_CALL`, `INJECTED_RESPONSE` (`cortex_pb.proto` L419-430)

**Data Points — LanguageServerService**:
- **~100 RPCs** (`language_server_pb.proto` L1679-1828) in 9 categories:
  | Category | Count | Representative RPCs |
  |----------|-------|-------------------|
  | Completions | 3 | `GetCompletions`, `ProvideCompletionFeedback` |
  | Cascade Chat | 15 | `StartCascade`, `SendUserCascadeMessage`, `GetCascadeTrajectory` |
  | State Streaming | 5 | `StreamCascadePanelReactiveUpdates`, `StreamAgentStateUpdates` |
  | Memory/Rules | 6 | `GetCascadeMemories`, `GetAllRules`, `GetAllSkills` |
  | MCP | 5 | `RefreshMcpServers`, `ListMcpResources` |
  | Browser | 10 | `ListPages`, `CaptureScreenshot` |
  | User Management | 8 | `GetUserStatus`, `GetTermsOfService` |
  | Experiment Flags | 4 | `GetUnleashData`, `SetBaseExperiments` |
  | Analytics | 5 | `RecordEvent`, `RecordAnalyticsEvent`, `RecordLints` |

**Key Finding**: The Cortex layer records **every atomic action** the agent takes (90+ types), with full state machine tracking (12 states) and cross-trajectory linkage (parent_references). This creates a complete behavioral log of the AI agent's decision-making process — and by extension, of the developer's work session.

### §4.3 Layer 3: JetSki (Backend Telemetry Bridge)

**Boundary**: Bridge between Language Server and Google Cloud Code internal APIs

**Data Points — State Sync**:
- **CascadeState** (`jetski_cortex_pb.proto` L10-20): full conversation snapshot including trajectory, status, queued steps, artifact snapshots, file diffs
- **AgentStateUpdate** (`jetski_cortex_pb.proto` L22-34): incremental updates with conversation_id, trajectory_id, main/sub-trajectory updates
- **Streaming RPC**: `StreamAgentStateUpdates` — continuous state synchronization

**Data Points — IDE Type Registry**:
- **14 IDE types** (`onboarding.proto` L73-89): VSCODE(1), INTELLIJ(2), CIDER(6, Google internal), ANDROID_STUDIO(8), ANTIGRAVITY(9), JETSKI(10), GEMINI_CLI(14)
- **Key finding**: JetSki is registered as IdeType=10, **separate** from Antigravity IdeType=9, indicating it operates as an independent headless agent backend

**Data Points — Metrics Pipeline**:
- **7 metric event categories** (`metrics.proto` L32-45): `InlineCompletionAccepted`, `InlineCompletionOffered`, `ConversationOffered`, `GenerateCodeUI`, `ConversationExplainUI`, `ConversationGenerateTestUI`, `ConversationInteraction`
- **Sub-categories**: THUMBSUP, THUMBSDOWN, COPY, INSERT, ACCEPT_CODE_BLOCK
- **2 reporting RPCs**: `RecordCodeAssistMetrics`, `RecordClientEvent`

**Data Points — User Management**:
- `OnboardUser`, `LoadCodeAssist` (with tier determination)
- `FetchAdminControls` (enterprise management)
- `UserTier` with upgrade types: GDP, GOOGLE_ONE, GDP_HELIUM, GOOGLE_ONE_HELIUM

### §4.4 Cross-Layer Data Flow Analysis

**The Full Pipeline**:
```
User types message
  → Cascade captures input + 78 UI event types
    → Cortex logs as Step (90+ types, 12 states)
      → Trajectory system records full execution trace
        → JetSki streams incremental AgentStateUpdate to Cloud Code
          → RecordCodeAssistMetrics aggregates to BigQuery/Sentry
```

**Enrichment Cascade**:
1. Layer 1 (UI): user input text, panel interactions, model selection
2. Layer 2 (Engine): adds agent reasoning, tool calls, file contents read, commands executed, browser actions taken, memory operations
3. Layer 3 (Backend): adds user tier, experiment flags, IDE type, and persists the **complete enriched trajectory**

**Quantitative Summary**:
| Metric | Count | Source |
|--------|-------|--------|
| UI event types | 78 | Feature flag extraction |
| Agent step types | 90+ | `cortex_pb.proto` L215-314 |
| Step states | 12 | `cortex_pb.proto` L136-149 |
| Trajectory types | 14 | `cortex_pb.proto` L79-94 |
| Trajectory sources | 16 | `cortex_pb.proto` L61-77 |
| Reverse-gRPC RPCs | 53 | `extension_server_pb.proto` L500-553 |
| LS RPCs | ~100 | `language_server_pb.proto` L1679-1828 |
| IDE types | 14 | `onboarding.proto` L73-89 |
| Metric event categories | 7+1 | `metrics.proto` L32-45 |
| Proto files analyzed | 22 | Binary extraction |
| Proto schema total size | ~250KB | - |
| Memory trigger types | 4 | `cortex_pb.proto` L489-495 |
| Agent modes | 3 | `cortex_pb.proto` L172-177 |
| Executor termination reasons | 10 | `cortex_pb.proto` L419-430 |

---

## §5 Discussion: Implications for Defense System Design (~1 page)

### 5.1 Pervasive Monitoring Assessment
- Apply RFC 7258 criteria to the findings:
  - **Scale**: 90+ step types × 12 states × 14 trajectory types = thousands of potential data points per session
  - **Scope**: captures code content, developer intent (NL prompts), execution context, file system structure, browser interactions
  - **Persistence**: JetSki's `StreamAgentStateUpdates` creates a real-time, continuous record
  - **Lack of opt-out**: no evidence of granular telemetry controls in the proto schemas
- Conclusion: meets RFC 7258's threshold for "pervasive" by any reasonable interpretation

### 5.2 The Reverse-gRPC Attack Surface
- 53 RPCs from server to client = the backend can instrument the UI
- `ExecuteCommand`, `LaunchBrowser`, `WriteCascadeEdit` — these are not telemetry *collection*, they are telemetry-*informed control*
- Implication: a compromised backend could weaponize the telemetry channel for active attacks

### 5.3 USS as a Control Channel
- `PlanningModeConfig` is propagated via USS, a KV store with no access control in the schema
- This is the mechanism by which the vendor's system prompt overrides user-defined rules (the "Ring 0" injection, detailed in companion paper [ref])
- Telemetry and control share the same channel — violating the principle of separation of concerns

### 5.4 SASS Data Minimization as a Positive Counterexample
- SASS (Self-Auditing Security System, [ref]) demonstrates that effective security monitoring can be achieved with:
  - Local-only processing (no cloud telemetry)
  - Schema-level data minimization (only collect what's needed for the security function)
  - User-controlled retention policies
- Contrast: the analyzed agent collects everything, retains indefinitely (no TTL in proto schemas), and transmits to cloud by default

### 5.5 Recommendations
1. **Telemetry separation**: distinct channels for crash reporting (necessary) vs. behavioral telemetry (questionable)
2. **Schema-level minimization**: remove fields that serve no user-facing function (e.g., `AiCharactersReports`)
3. **Opt-in behavioral telemetry**: make the 90+ step type logging opt-in, not opt-out
4. **Local-first trajectory storage**: trajectory data should remain local unless the user explicitly chooses cloud sync
5. **Reverse-gRPC audit**: the 53 server-to-client RPCs should be documented and their telemetry implications disclosed
6. **Graduated containment**: defense responses should escalate proportionally to deviation stage (S1–S5 model), not binary allow/deny
7. **Platform-aware containment**: defense mechanisms sanitizing the agent's execution environment must preserve OS-critical dependencies (e.g., Windows CRT loader paths); over-aggressive sanitization causes self-inflicted denial of service

---

## §6 Conclusion (~0.5 page)

- First structural anatomy of a three-layer telemetry architecture in a commercial AI coding agent
- Quantified the telemetry surface area: 90+ step types, 53 reverse-gRPC RPCs, 14 IDE types, 7 metric categories, 22 proto files
- Extended RFC 7258's "pervasive monitoring is an attack" framing to development environments
- The analyzed architecture creates a **full-stack developer activity record** that goes far beyond traditional IDE telemetry
- SASS data minimization principles offer a constructive path forward
- Future work: behavioral analysis of actual telemetry payloads, cross-agent comparison, privacy-preserving agent architectures
- Connection to companion work on prompt priority hierarchies and Ring 0/Ring 3 conflicts (IEEE S&P 2027, [ref])

---

## References (Key, ~20 entries)

- RFC 7258 (BCP 188): Pervasive Monitoring Is an Attack
- Companion paper: Ring 0/Ring 3 prompt priority (IEEE S&P 2027)
- VS Code telemetry documentation
- Protocol Buffers specification
- gRPC reverse-call pattern (bidirectional streaming)
- AI coding agent surveys (if any exist by submission)
- SASS framework reference
- Unleash feature flag system
- CEL (Common Expression Language) specification
- BigQuery/Sentry telemetry infrastructure references

---

## Appendix (if space permits)

### A. Complete CortexStepType Enumeration
- Table of all 90+ step types with values, proto line numbers

### B. ExtensionServerService RPC Listing
- Table of all 53 reverse-gRPC RPCs

### C. IDE Type Registry
- Complete IdeType enumeration from onboarding.proto
