# Current overview

Updated18November2026. Current source baseline `fac5a864`; Rust1.100 is pinned.
[Single lead checkpoint and continuation](PROJECT-STATE.md).
Current development branch: `codex/ci-power-lifecycle`. CI-L1 local checks pass;
mandatory hosted checks remain separate. Score **41 → 41/100**.

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
