# Current project status

## PR #167 integration — 9 November 2026

The VELD-08 window-switcher correction and synchronization with main were committed as `d14253d1c5e8d169116e59174ca1ed193e96f5bd` on `veld/next-shell-20261103`. The patch adds blocked-page and minimized-window regression coverage, requires a fully repainted native screen before fixture capture, makes key-edge input counting exact, and requests F10 cleanup. Historical native two-scale evidence is bound to `537fee48c1c31155a51a997cd7a2236b159e9ac7` and does **not** establish a native pass for the revised source. CI and review for the new head are pending at this checkpoint. Progress remains **41/100**, with no newly accepted native criterion. Resume with current-head CI/review and the two native switcher scale gates before claiming native acceptance.

## PR #284 refresh — 9 November 2026

Dependabot coverage is extended to the remaining independent Cargo workspaces. The separate SDK entry from #285 is retained; its directory is excluded from the grouped entry to prevent duplicates. Progress remains 41/100. Local preflight, build and QEMU smoke passed (exit 0 each); all 11 workflow/Dependabot contract tests passed in WSL with the Windows worktree Git directory explicitly selected.

## PR #285 refresh — 9 November 2026

SDK monthly Dependabot coverage is retained on current main. The current documentation hierarchy is preserved. Progress remains 41/100; no new product acceptance. Local `python tools/preflight.py --strict`, `python tools/build.py`, and `python tools/run.py --smoke` passed (exit 0 each; QEMU raw exit 33, M1:PASS). No physical-hardware acceptance is claimed.

## NEXT AGENT HANDOFF — documentation hierarchy and governance — 9 November 2026

**Source and scope:** base `main` `f704f078213170e72292a5a491f0ee83bd588955`, branch `docs/restructure-project-docs-20261109`. The active README, contribution guidance, security policy, licensing documentation, and status layers have been reorganized so that each information type has one primary source. Historical content was preserved under `docs/history/` before the active files were shortened.

**Progress:** **41/100 → 41/100**. This is documentation and governance work; no new product criterion is accepted and no native OS functionality is changed.

**Open repository administration:** GitHub Wiki pages, GitHub Milestone objects, and a real private vulnerability-reporting channel require repository-level actions that are not exposed by the current connector. They are tracked in issues [#277](https://github.com/simonvanieperen/BaryonOS/issues/277), [#276](https://github.com/simonvanieperen/BaryonOS/issues/276), and [#278](https://github.com/simonvanieperen/BaryonOS/issues/278), respectively. [ROADMAP.md](ROADMAP.md) remains the authoritative milestone source until the Milestone objects exist.

**Resume:** review this branch through CI and automated review. Confirm that README, `tools/progress.py`, `docs/progress.json`, `docs/PROGRESS.md`, and `docs/assets/progress.svg` remain consistent with `python3 tools/progress.py --check`.

## Authoritative sources

| Question | Primary source |
|---|---|
| What is Baryon OS? | [README.md](../README.md) |
| What is the current technical state? | This file |
| How is the 0–100 score calculated? | [PROGRESS.md](PROGRESS.md) and [progress.json](progress.json) |
| What must ultimately be achieved? | [ROADMAP.md](ROADMAP.md) and [FULL-TEST-RELEASE.md](FULL-TEST-RELEASE.md) |
| What concrete work is open? | GitHub Issues and [WORK-QUEUE.md](WORK-QUEUE.md) where still applicable |
| Which architecture decisions apply? | [DECISIONS.md](DECISIONS.md) and `docs/implementation/` |
| How is the system built and tested? | [DEVELOPMENT.md](DEVELOPMENT.md) and [TEST-STRATEGY.md](TEST-STRATEGY.md) |
| Where is source-bound test evidence stored? | [verification/](verification/) |
| Where is historical status stored? | [history/](history/) |
| How can someone contribute? | [CONTRIBUTING.md](../CONTRIBUTING.md) |
| How should a vulnerability be reported? | [SECURITY.md](../SECURITY.md) |
| Which software license applies? | [LICENSE](../LICENSE), [LICENSING.md](../LICENSING.md), and [MPL-2.0-FILES.txt](../MPL-2.0-FILES.txt) |

## Current development state

The fixed progress score is **41/100**. Detailed accepted and open criteria are maintained only in [PROGRESS.md](PROGRESS.md) and [progress.json](progress.json). Recent source-bound implementation and CI evidence remains available under [verification/](verification/).

The former checkpoint sequence is preserved in [PROJECT-STATE-through-2026-11-09.md](history/PROJECT-STATE-through-2026-11-09.md). That snapshot is historical context, not a second current-status source.
