# Current overview

Updated 18 November 2026. Source baseline: main
`6f29b6720b51aecad5714faa48b445afd6e2f5a9` (#355), preceded by #359 at `1c10cdb5`.
Current development: N04-T1 prototype blocked on native throughput; next B05-T2.
[Single lead checkpoint and continuation](PROJECT-STATE.md).
Integration/merges/closure remain with the separate Integration Manager.

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
