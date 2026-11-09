# Current project status

## NEXT AGENT HANDOFF — K06 transferred contracts and step 5f — 9 November 2026

**Source:** branch `codex/k06-5f-acceptance-20261109`, base `main` `1db01c4a1fe9e433396c58af4e6b3732034ee2e9`. Selected prerequisites are taken from `feat/k06-5e-proc-shootdown` `f141995ce6d5a0032cb90dea1466a24dc71b2f4d`, including the step-5d-2 checkpoint `4030c4f4`. N04 and current main documentation are retained; unrelated runner migrations are excluded. Earlier local work remains in its original worktree.

**Transferred scope:** cross-CPU IPC (R3-5), address-space-locked usercopy (R3-6), K05/resource contracts under SMP (R3-7), explicit task migration, the complete L0 step-5f matrix with 50 repetitions per case, and the ten-minute soak. These requirements are open until source-bound native evidence exists.

**Implemented, under verification:** selected K06 prerequisites; `--repeat` with exact per-case counts and retained failure evidence; the 1/2/4/8-vCPU single/multi-TCG M3 matrix; explicit 1→2→0 migration with GPR/x87/SSE canaries; cross-CPU IPC and remote `0xe8`; real remote kill/unmap/recovery fixtures; KERNEL-wait fairness measurements with unchanged 400-ms/40-tick bounds; separate four-vCPU M4/qsim proofs; and one combined ten-minute soak manifest containing M4 quota, qsim pressure, IPC and CPU-bound load. None is accepted until the frozen 50/50 matrix and regressions complete. The [decision record](DECISIONS.md#adr-k06-5f-20261109--bound-and-measure-smp-acceptance-without-widening-authority) defines the acceptance limits and evidence boundary.

**Executed:** developmental single-run native smokes passed for migration, 30-second load, remote kill, real unmap and recovery before the combined-soak increment. The current host oracle suite has 268/268 passing tests; the full non-kernel Cargo workspace is green. A combined-soak development run remains non-acceptance evidence, and any run made while source changes is deliberately rejected by the runner. No 50/50 case, K07-SMP extension or final soak result is accepted yet.

**Progress:** **41/100 → 41/100**; K06 remains open and K07 retains its existing single-CPU evidence boundary. Native source must remain frozen throughout each repeated run. The next step is to compile the completed explicit IPC/RUNQ/lifecycle increment, execute one native positive and negative case, and repair any failure before starting repeated acceptance runs.

## Previous checkpoint — documentation hierarchy and governance — 9 November 2026

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
