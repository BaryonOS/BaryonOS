# Current overview

Updated 18 November 2026. Source baseline: main
`6f29b6720b51aecad5714faa48b445afd6e2f5a9` (#355), preceded by #359 at `1c10cdb5`.
Latest development checkpoint: `codex/b04-repro-check` (B04-T2 comparator delivered;
real build comparison RED: loader/ESP differ). [Executable continuation](PROJECT-STATE.md).
This implementation session delivered PR372 USB, PR374 model parser, PR375 native
NAS quota/offline eviction, PR377 settings and PR378 scoped search. SDK-core membership
remains with the integration owner; no native U05 acceptance. Overall41/100 unchanged.
Integration/merge/closure remains owned by the separate Integration Manager.

| Workstream | Current boundary | Next step |
|---|---|---|
| Codex H/U/N/B/A/P/L/R | Released READY queue processed; bounded code/native slices and supporting proposals preserved in draft PRs | N02-T3 after PR370 integration/write ownership release; actual source blockers in checkpoint |
| Integration | Separate Integration Manager owns existing PR review, merge and closure | No Codex development duplicate integration or CI polling |
| ClaudeS09/UI | Claude-owned worktrees preserved; main includes S09 #340 | Follow those workstreams' own handoffs; no Codex writes |
| Shared governance | Contracts/adapters/roles/skills integrated in main #362 | Runtime parity and held automation remain separate |
| Unsafe unattended runner | Source hold and disabled policy preserved | No activation; existing Daybreak/model-client/runtime findings remain open |
| Former #355 WIP | Original preserved checkout and recovery-only #357 remain unincorporated | Do not discard or publish blindly |

Branch-bound host/QEMU evidence does not establish current-main acceptance,
physical support, a human audit or release readiness. Proposed owner/security
boundaries remain proposed. No wiki or new project-management system was created.
