# F2: Operational Scale Declaration

**Document ID**: F2  
**Category**: Operational Context  
**Generated**: 2026-06-12  
**Status**: Declaration only — no raw log content included

---

## 1. Scope Statement

The 72 sessions included in this evidence corpus represent a curated selection of **incident-window sessions** extracted from a total population of **7,000+ sessions** accumulated over the deployment period. These sessions were selected based on temporal proximity to observed anomalous events and behavioral pattern relevance.

## 2. Log Volume

A single monitored endpoint generates approximately **~325 MB of agent command logs per day**. The table below provides representative daily log sizes during a sample observation window:

| Date | Log Size | Note |
|------------|----------|------------------|
| 2026-02-04 | ~280 MB | Normal operation |
| 2026-02-05 | ~310 MB | Normal operation |
| 2026-02-06 | ~295 MB | Normal operation |
| 2026-02-07 | ~340 MB | High activity |
| 2026-02-08 | ~325 MB | Normal operation |
| 2026-02-09 | ~315 MB | Normal operation |
| 2026-02-10 | ~350 MB | Peak activity |
| 2026-02-11 | ~330 MB | Normal operation |
| 2026-02-12 | ~305 MB | Normal operation |

**9-day sample total**: ~2,850 MB (~2.78 GB)

## 3. Aggregate Estimate

Over the four-month deployment period (**2026-02 through 2026-06**), the estimated total volume of agent commands exceeds **100,000 individual command invocations** across all monitored sessions.

This estimate is derived from:
- Daily log file sizes (as sampled above)
- Average command density per MB of structured log output
- Session count growth rate observed across the deployment window

## 4. Privacy Notice

This document serves as a **scale declaration only**. Raw log contents are not included in this evidence corpus due to the inability to fully sanitize personally identifiable information, infrastructure details, and third-party service credentials embedded within agent session logs.

The log size figures and command count estimates presented here are verifiable against the original log archives, which remain available under controlled access.

---

*This document is part of the evidence corpus. See [INDEX.md](../INDEX.md) for navigation.*
