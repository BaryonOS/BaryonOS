# Current overview

Updated 18 November 2026. Source baseline: main `69d82075` (#362 governance and #340 S09 included).
Current task branch: `codex/unified-governance`. Details and executable resume step:
[lead checkpoint](PROJECT-STATE.md#next-agent-handoff--unified-governance-implementation--18-november-2026).

| Workstream | State and owner | Next boundary |
|---|---|---|
| Unified governance | Codex lead, shared contract/adapters/roles/skills and compact recovery implemented; 521 host tests, 15 skips, exit 0 | Local contract/protection regressions pass; draft PR and mandatory hosted checks next; governance is separate from runtime parity |
| Issue-pipeline containment | #359/#355 merged; source execution hold and disabled manifest preserved | Authenticated model clients, offline gate runtime and Daybreak acceptance remain open; no activation |
| Former #355 WIP | `sec355-local` at `124a679c`, dirty gate/tests and untracked offline-QEMU evidence | Preserved in place, not published or incorporated; #357 remains recovery-only/unaccepted |
| Claude UI/UX and suite | Claude-owned worktrees; UI/UX PR #360 | Follow those workstreams' own handoffs; no Codex writes |
| S09 | Claude-owned `s09-native-archive`, PR #340 | Preserve ownership and outstanding owner decisions |
| Roadmap | Existing fixed [register](PROGRESS.md) and task ledger | Continue one eligible disjoint task or straightforward PR after governance checks; do not use held issue execution |

Progress **41 → 41/100**: governance and containment establish no new OS criterion.
There is no zero-open-items or baseline-B prerequisite for governance. Product
acceptance and Security Floor remain unchanged. See
[governance decision](DECISIONS.md#adr-agent-governance-20261118--shared-contracts-and-risk-based-execution).

Local host checks and repository contracts do not establish client runtime parity,
resolved Daybreak findings, hardware support or a human audit. Unrelated blockers
are tracked separately rather than stopping independent implementation.
