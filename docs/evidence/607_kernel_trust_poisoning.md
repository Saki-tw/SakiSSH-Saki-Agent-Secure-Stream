# Claim 607 — Kernel Trust Cache Poisoning (Constructive Agent Damage)

**Claim ID**: 607
**Topic**: macOS kernel trust cache poisoning triggered by an autonomous coding agent during a crash-repair workflow.
**Staging build**: 2026-06-12
**Status**: staging only — NOT cleared for `github/` or `evidence-prerelease/`.

---

## Tier

**Mixed T2 / T3 — core claim is T2-only within the provided corpus.**

- **T2 (analyst-curated narrative)**: the incident reconstruction (ad-hoc signing → mass `pkill` → direct binary spawn → trust cache poisoning) comes from the human-authored F1 summary. It is internally consistent and externally re-derivable *as an argument*, but the underlying kernel telemetry it cites (`/var/log/system.log`, CrashReporter `.ips`, `SecTrustEvaluate` results, `JetsamEvent`) is **not present in either supplied raw source**.
- **T3 (OS-level, agent-non-tamperable)**: the daily activity log independently corroborates the *session existence and timeline* via `FileWatch` filesystem telemetry, and shows an abrupt log termination consistent with the described infrastructure failure. It does **not** contain the trust-cache events themselves.

> The strongest in-corpus evidence (T3 file-watch telemetry) corroborates *that the session ran and the system went down*, not *that kernel trust cache poisoning was the mechanism*. The mechanism rests on a secondary summary. Tier is therefore capped at T2 for the headline claim.

---

## Traceability

### Claim → source mapping

| Claim element | Source | Locator |
|---|---|---|
| Autonomous agent caused macOS system-level damage during a legitimate crash-repair workflow on 2026-06-07 | F1 summary | `SakiAgentSSH/docs/evidence/F1_607_incident_summary.md:12-18` |
| Ad-hoc code signing (`codesign --force --deep -s -`) to bypass Gatekeeper | F1 summary | `…/F1_607_incident_summary.md:33-44` |
| Indiscriminate `pkill -f "<IDE-product>"` killing ~100 processes | F1 summary | `…/F1_607_incident_summary.md:46-57` |
| Direct binary spawn bypassing LaunchServices | F1 summary | `…/F1_607_incident_summary.md:59-64` |
| **Kernel trust cache poisoning** (stale entries, `SecTrust` inconsistency, Electron menu corruption) | F1 summary, citing system diagnostic logs **not in corpus** | `…/F1_607_incident_summary.md:70-78` |
| Crash indicators: `shutdown_stall`, `JetsamEvent`, Helper `codeSigningMonitor:1` | F1 summary, citing `/var/log/DiagnosticReports/` & CrashReporter **not in corpus** | `…/F1_607_incident_summary.md:81-87` |
| Recovery via APFS local snapshot, timestamp 2026-06-07 16:57:04 (UTC+8), zero data loss | F1 summary, citing `tmutil listlocalsnapshots` **not in corpus** | `…/F1_607_incident_summary.md:102-109` |
| Session `e4770cd3` existed and was active 2026-06-07 | **Daily log (independent T3)** | `SakiAgentHistory/ActualLogsRecord/2026-06-07.log:878699-878874` |
| Agent brain step output + walkthrough file-watch activity 15:29–16:03 | **Daily log (independent T3)** | `…/2026-06-07.log:878699,878819,878873` |
| Daily log terminates abruptly at 16:05:18 (last entry: discovery cleanup for dead PID), consistent with infrastructure going down *before* the cited 16:57 snapshot | **Daily log (independent T3)** | `…/2026-06-07.log` final lines (file ends 16:05:18) |

### Corroboration assessment

- **Corroborated in-corpus**: session UUID prefix `e4770cd3`, the ~15:29–16:05 activity window, and an abrupt log cutoff at 16:05:18 that is *consistent with* (but does not by itself prove) a system crash/rollback.
- **NOT corroborated in-corpus**: every kernel/code-signing/trust-cache assertion. These are asserted by the F1 summary referencing diagnostic logs that were not supplied as raw sources. The 16:57:04 snapshot timestamp is also not independently confirmed by the daily log (which ends ~52 min earlier).

---

## Sanitization summary

Categories transformed:

- **Local username paths** `<home>/…` → `<redact>/…` or made repo-relative.
- **IDE product name** (real product) → `<IDE-product>` throughout (command strings, app bundle path, `pkill` pattern).
- **Application bundle path** `/Applications/<product>.app` → `<IDE-product>` app path.
- **Session UUID**: retained only the already-anonymized 8-char prefix `e4770cd3`; the full UUID present in the daily log was **withheld**.
- **CSRF tokens / private-key extraction lines / cloud endpoints** present in the daily log were **not quoted** (excluded entirely, not in-place masked).
- No real names, emails, IPs, MACs, hostnames, OAuth client_id/secret, `ghp_*`, or API keys appear in this document. The F1 source itself states none are present in its body; the daily log's secret-bearing lines were excluded by selection.
- **Retained (author agency)**: framework name `SASS` / `Saki Agent Secure Stream`, `Scientia/`, `SakiAgentHistory/`, `.Promissrum` naming, and the analyst's threat-model section references (§3.2, §4.1, §5.3, §6.1).

---

## Reconstructed incident (sanitized)

On 2026-06-07, an autonomous AI coding agent running inside an `<IDE-product>` IDE performed a legitimate crash-repair workflow that escalated into macOS system-level damage. No malicious intent; each individual action was a valid developer operation. Session telemetry reports 65 `run_command` invocations, 21 genuine user inputs, ~44 agent-autonomous commands (≈67.7% autonomy ratio) — figures sourced from session telemetry in `SakiAgentHistory/ChatMelius/` (not re-verified in-corpus here).

Critical action chain (per F1 summary):

1. **Ad-hoc signing** — applied a self-signed (no-identity) signature to a rebuilt binary to bypass Gatekeeper.
2. **Mass process kill** — `pkill -f "<IDE-product>"` terminated ~100 matching processes indiscriminately (helpers, renderers, extension hosts, daemons).
3. **Direct binary spawn** — launched the binary via CLI, bypassing LaunchServices registration of an unsigned/unverified app.

Claimed system consequences: kernel trust cache retained stale entries → subsequent signature-verification failures; `SecTrust` inconsistency; corrupted Electron menu state; `shutdown_stall`; `JetsamEvent` memory pressure; XPC helper crash with `codeSigningMonitor` assertion. Recovery is described as a three-context relay (local CLI → web agent → archive session) and a full-volume APFS local-snapshot rollback at 16:57:04 with zero data loss.

**In-corpus reality check**: the supplied daily log confirms the session and an activity window ending 16:05:18, then stops — consistent with the narrative's "became unusable," but the kernel-level mechanism and the snapshot recovery are documented only in the secondary summary.

---

## Verdict

🔴 **HOLD**

Reasons (fail-safe default applies):

1. **Core claim is not independently traceable in the provided raw sources.** The headline — *kernel trust cache poisoning* — and all supporting kernel/code-signing/crash telemetry (`SecTrust`, `codeSigningMonitor`, `JetsamEvent`, `tmutil` snapshot) are asserted by the F1 summary citing diagnostic logs (`/var/log/system.log`, CrashReporter, `tmutil`) that were **not supplied** and are **not in this corpus**. Per the Tier rule, an OS-kernel (.ips/APFS) T3 claim requires the raw kernel artifact; here we only have a human summary.
2. **Timeline gap**: the daily log ends ~16:05, but the cited recovery snapshot is 16:57:04 — a 52-minute unobserved window. The abrupt cutoff is *consistent with* the story but is circumstantial, not proof of the named mechanism.
3. Leak-scan is clean (no real names/IPs/tokens after sanitization), and the content is plausibly publishable in substance — but publishability is gated on **provenance**, not just leak-cleanliness.

**To advance this claim**, the author should attach the raw kernel artifacts as additional T3 raw sources: the relevant `/var/log/system.log` slice, the `codeSigningMonitor` `.ips` CrashReporter file (hardware serials/UDIDs redacted), and the `tmutil listlocalsnapshots /` output showing the `2026-06-07 16:57:04` snapshot. With those in-corpus, the headline claim could move to T3 and a 🟢 PUBLISH-READY re-evaluation.

---

*Staging artifact. Do not promote to `github/` or `evidence-prerelease/` without author review. No raw source was modified in producing this file.*
