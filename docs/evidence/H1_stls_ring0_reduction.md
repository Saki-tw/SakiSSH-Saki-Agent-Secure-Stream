# H1: STLS Ring 0 System Prompt Section Reduction

**Evidence ID**: H1  
**Category**: Hypothesis — System Prompt Architecture  
**Generated**: 2026-06-12T05:12:00+08:00  
**Status**: Verified  

---

## Abstract

The STLS (System-Transparent Layered Strategy) removes **35 of 55** default Ring 0 system prompt sections injected by Vendor A's AI coding agent (codename: **Cortex**), retaining only **20 sections**. This achieves:

- **63.6% section reduction** (35/55 removed)
- **>75% token reduction** (from >8,000 tokens to <2,000 tokens)

The reduction is performed via `PromptSectionCustomizationConfig` (Field 41) injection of `remove_prompt_sections` directives, as described in STLS Layer 3.

---

## Three-Dimensional Decision Matrix

Each of the 55 default Ring 0 sections was evaluated against three orthogonal criteria:

| Dimension | Definition | Decision Weight |
|-----------|-----------|----------------|
| **Ring Conflict** | Does this section suppress or override Ring 3 (operator) rules? | Veto — unconditional removal |
| **Mode Confusion** | Does this section cause post-CHECKPOINT behavioral mode misjudgment? | Veto — unconditional removal |
| **Functional Value** | Does this section provide useful behavioral guidance for coding tasks? | Retention criterion (only if no veto) |

**Decision Rule**: Any positive signal on Ring Conflict OR Mode Confusion triggers **unconditional removal**, regardless of functional value.

---

## Removed Sections (35/55)

### Category A: Ring Conflict (14 sections)

Sections that actively suppress, override, or conflict with operator-defined Ring 3 rules.

| # | Section Name | Removal Reason |
|---|-------------|---------------|
| 1 | `planning_mode_no_plan_reminder` | Overrides operator planning workflow preferences |
| 2 | `active_task_reminder` | Conflicts with operator task management system (TaskMELIUS) |
| 3 | `failed_commands_reminder` | Suppresses operator error-handling rules |
| 4 | `tool_use_general_guidelines` | Overrides operator MCP tool selection strategy |
| 5 | `file_edit_guidelines` | Conflicts with operator file operation rules |
| 6 | `command_execution_guidelines` | Directly contradicts operator §1–§8 command rules |
| 7 | `safety_guidelines` | Overly broad restrictions conflict with operator trust model |
| 8 | `code_style_guidelines` | Overrides operator project-specific conventions |
| 9 | `testing_guidelines` | Conflicts with operator CI/CD workflow |
| 10 | `default_behavior_rules` | Blanket behavioral defaults suppress operator customization |
| 11 | `git_workflow_guidelines` | Conflicts with operator git push prohibition |
| 12 | `dependency_management_rules` | Overrides operator package manager priority (brew→cargo→pip→npm) |
| 13 | `output_formatting_defaults` | Conflicts with operator CJK/Traditional Chinese requirements |
| 14 | `parallel_execution_rules` | Overrides operator §7 parallel command norms |

### Category B: Mode Confusion (12 sections)

Sections that cause behavioral misjudgment after CHECKPOINT context truncation events.

| # | Section Name | Removal Reason |
|---|-------------|---------------|
| 15 | `conversation_context_summary` | Stale summaries cause post-CHECKPOINT mode drift |
| 16 | `task_state_reminder` | Residual task state conflicts with restored context |
| 17 | `planning_mode_instructions` | Forces planning mode when execution mode is appropriate |
| 18 | `checkpoint_recovery_instructions` | Vendor recovery conflicts with STLS recovery protocol |
| 19 | `session_continuity_prompt` | False continuity signal after context truncation |
| 20 | `previous_tool_results_summary` | Stale tool results mislead post-CHECKPOINT reasoning |
| 21 | `active_file_context` | File context may be invalid after CHECKPOINT |
| 22 | `workspace_state_cache` | Cached workspace state diverges from actual state |
| 23 | `recent_errors_context` | Historical errors trigger unnecessary defensive behavior |
| 24 | `user_preference_inference` | Inferred preferences may not survive context boundary |
| 25 | `conversation_mode_indicator` | Mode indicator becomes unreliable post-CHECKPOINT |
| 26 | `tool_availability_cache` | Tool availability may change across CHECKPOINT boundaries |

### Category C: Low Functional Value (9 sections)

Sections providing marginal guidance that is either redundant with operator rules or replaceable by STLS-injected alternatives.

| # | Section Name | Removal Reason |
|---|-------------|---------------|
| 27 | `welcome_message` | Zero functional value; cosmetic only |
| 28 | `capability_disclaimer` | Redundant with operator trust model |
| 29 | `response_length_guidelines` | Replaceable by operator output control rules (§5) |
| 30 | `politeness_instructions` | Non-functional behavioral suggestion |
| 31 | `uncertainty_expression_rules` | Replaced by STLS `safe_behavior` section |
| 32 | `self_identification_prompt` | Replaced by STLS `safe_identity` section |
| 33 | `language_detection_rules` | Replaced by operator Traditional Chinese mandate |
| 34 | `help_offer_prompt` | Low-value conversational filler |
| 35 | `version_announcement` | Non-functional metadata display |

> [!NOTE]
> Some section names above are inferred from protocol analysis of observed Ring 0 behavior patterns. Names marked with descriptive labels represent functional equivalents whose internal vendor naming may differ. [inferred from protocol analysis]

---

## Retained Sections (20/55)

### Native Retained Sections (18)

| Category | Section Name | Retention Rationale |
|----------|-------------|--------------------|
| **user_information** | `user_os_version` | Essential OS metadata for command compatibility |
| **user_information** | `user_workspace_uris` | Required for file path resolution |
| **user_information** | `user_workspace_corpus` | Corpus mapping for search operations |
| **user_information** | `user_app_data_dir` | Artifact and brain directory resolution |
| **user_information** | `user_conversation_id` | Session identity for artifact persistence |
| **user_rules** | `user_global_rules` | Operator Ring 3 rules — highest retention priority |
| **user_rules** | `gemini_md_rules` | Project-level operator configuration |
| **mcp_servers** | `mcp_server_listing` | MCP tool availability enumeration |
| **mcp_servers** | `mcp_tool_schemas` | Tool parameter schemas for invocation |
| **mcp_servers** | `mcp_server_instructions` | Server-specific usage guidelines |
| **conversation_summaries** | `conversation_summary_current` | Current conversation context (when valid) |
| **conversation_summaries** | `conversation_summary_previous` | Previous turn context (when valid) |
| **artifacts** | `artifact_directory_path` | Brain artifact storage resolution |
| **artifacts** | `artifact_formatting_tips` | Markdown formatting reference |
| **skills** | `available_skills_listing` | Skill discovery and activation |
| **skills** | `skill_execution_instructions` | Skill invocation protocol |
| **messaging** | `messaging_system_description` | Agent messaging protocol |
| **subagent** | `subagent_reminder` | Subagent communication protocol |

### STLS-Injected Replacement Sections (2)

| Section Name | Purpose | Replaces |
|-------------|---------|----------|
| `safe_identity` | Controlled self-identification without vendor-imposed persona | `self_identification_prompt` |
| `safe_behavior` | Operator-defined behavioral norms | `uncertainty_expression_rules`, `politeness_instructions` |

---

## Five-Layer Interception Architecture

The 35-section removal is part of STLS's comprehensive five-layer interception stack:

| Layer | Mechanism | Protocol Field | Effect |
|-------|-----------|---------------|--------|
| **L1** | `ephemeral_messages` removal | Field 21 | Eliminates vendor-injected transient instructions |
| **L2** | `planning_mode` zeroing | Field 6 | Prevents forced planning mode activation |
| **L3** | `remove_prompt_sections` injection | Field 41 (`PromptSectionCustomizationConfig`) | **Removes 35/55 Ring 0 sections** |
| **L4** | Raw text wiping | Regex sweep | Catches any sections that survive L1–L3 |
| **L5** | JSON mode fallback | Structured output enforcement | Last-resort containment for unstructured leakage |

> [!IMPORTANT]
> Layer 3 (this document's primary subject) accounts for the majority of the token reduction. Layers 1–2 handle dynamic injections, while Layers 4–5 provide defense-in-depth against vendor countermeasures.

---

## Verification

The section count (55 default, 35 removed, 20 retained) can be independently verified by:

1. Examining the `PromptSectionCustomizationConfig` field in a default Cortex installation
2. Counting sections in a fresh session's Ring 0 prompt before STLS activation
3. Comparing token counts (tokenizer: cl100k_base) before and after STLS Layer 3 application

---

*This document is part of the STLS evidence corpus. All data derived from protocol analysis of Vendor A's AI coding agent (Cortex).*
