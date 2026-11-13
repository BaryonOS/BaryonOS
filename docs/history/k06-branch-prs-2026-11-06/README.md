# K06 branch pull requests #227, #229, #232, #238 and #246 — archived documentation

> Historical archive, created on 13 November 2026. The five pull requests below were written on the K06 (SMP)
> branches between 6 and 11 November 2026 and were never merged. Their code reached `main` through #296
> (`4e8e235`) and was later revised there; their documentation did not. This directory preserves that
> documentation unchanged so that the pull requests can be closed without losing content. The originals are in
> Dutch and are kept verbatim as historical evidence; English translations are in the active documents listed
> below. Evidence, scores and branch names describe the branch commits, not `main`.

| PR | Title | Head | Merge base |
|---|---|---|---|
| #227 | K06 steps 5b/5c: IRQ-aware spinning and lock-safe preemption | `1bbd85d` | `c663f0b` |
| #229 | K06 step 5d-1: per-AP runqueues, 0xe8 wakeups and CR3 switching with active_cpus | `803ea3d` | `5ae0757` |
| #232 | docs(k06): L0 specifications for the remaining K06 steps (5b-2 through 5f) | `dcd80d1` | `342194f` |
| #238 | K06 F5/F7: STUCK diagnostics in release builds and per-line SERIAL locking | `5b864e2` | `76c67f2` (base branch of #229) |
| #246 | Feat/k06 5d2 ring3 smp | `273b2eb` | `5ae0757` |

## Contents

| Path | Content | Source |
|---|---|---|
| `verification/` | 42 evidence files of the branch steps (5b-2, 5c-2, 5d-1, 5d-2a/b, 5e, audit F1/F2, F3/F4, F5/F7, F8, F16/F17, F19/F15) | added files of #246 (the 21 files of #238 are identical) |
| `k06-steps/` | ten L0 step specifications (5b-2 to 5f) | added files of #232 |
| `patches/pr246-docs.patch` | changes to `docs/DECISIONS.md`, `docs/implementation/K06-SMP.md`, `docs/PROJECT-STATE.md`, `docs/progress.json`, `docs/assets/progress.svg`, `README.md`, `.gitignore` | `git diff 5ae0757 273b2eb` |
| `patches/pr229-docs-relative-to-pr246.patch` | the only documentation difference of #229 against #246 | `git diff 273b2eb 803ea3d` |
| `patches/pr238-docs.patch`, `pr232-docs.patch`, `pr227-docs.patch` | the same document set for those pull requests | `git diff <merge base> <head>` |
| `superseded-tools/` | `tools/auto-loop.sh` as in #229/#246 and as in #227 | added files |
| `disposition.json` | per pull request and per changed file: where its content is now | generated |

Every copied file is byte-identical to the pull request blob; `disposition.json` records the git blob ids.
Each patch applied to its merge base reproduces the pull request's documentation exactly. Two evidence
manifests (`verification/k06-5d2a-ring3-smp/SHA256SUMS`, `verification/k06-5d2b-fpu/SHA256SUMS`) list
repository-root paths of the time (`docs/verification/...`); the other manifests are relative to their directory.
All listed hashes match. Original paths of branches, worktrees and run directories are kept unchanged.

## Where the content is now

- **Code, tests and workflows:** on `main` through #296; all kernel features and `run_smp.py` profiles of these
  pull requests exist on `main`. `disposition.json` gives, per file, how many added lines occur verbatim on
  `main`; the rest was revised there (for example the interruptible spin window and the M3 SMP scope).
- **Decisions:** English in `docs/DECISIONS.md`: ADR-075, ADR-077 to ADR-081, and ADR-083, which was ADR-076 on
  the branches (`main` already uses ADR-076 for ACPI S5); the branch addenda to ADR-074 and ADR-072; the
  revision note on ADR-024.
- **Specifications:** English in `docs/implementation/k06-steps/` and in the section "Branch additions,
  6–11 November 2026" of `docs/implementation/K06-SMP.md`.
- **Superseded:** status handoffs, progress assessments and scores (later state on `main`; K06 accepted at
  41/100 on 13 November 2026, `docs/verification/K06-ACCEPTANCE.md`); `tools/auto-loop.sh` (replaced by
  `tools/agent_pipeline.py`).
- **Not adopted:** the `.claude/` line that #227, #229 and #246 added to `.gitignore`; it is kept in the patches.
