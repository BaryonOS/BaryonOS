# Current project status

## NEXT AGENT HANDOFF — K06 step 5f acceptance repair — 10 November 2026

**Source:** exact existing branch `codex/k06-5f-complete-20261109`; incoming source `f808d59150466aee3de24452ea408250b8f4dca6`. The current host-checked IPC/scheduler candidate has source-v3 `a6ce4a41bea9d95462d9803abcff5bf690eb31c1431e4e7807cab0d312af02f9` on working-tree base `55ec046fb287cf6bf06b96a4421cf2aac0f4723a`. No new native acceptance is claimed. Previously dispatched source `332275d97ed5f24f6d43174969a5667eb42f2afa` / source-v3 `0f5144e3a17b1b5d0a0e91b24fb8dcf4db8cf5e82965dc96d4f0d2db4d591a18` is queued in full restart `42741185181`; it does not include the current IPC correction.

**Rejected evidence:** [run 42723127310](https://github.com/simonvanieperen/BaryonOS/actions/runs/42723127310) is completed with failure on incoming source `f808d591`, source-v3 `759320893ea058b90d0bc764cd2b9534734827131a212ad5ba27f5501db6e8ac`. Independent raw-artifact readback checks all 3200 guests: 2976 pass and 224 fail across ten ordinary cases (63 FPU-image mismatches, 159 watchdogs and two clock-drift failures). K07 M4/qsim each pass 50/50 and all 50 complete soaks pass on that rejected source. The aggregator fails; the final gate is skipped. All 74 original artifact ZIPs remain intact, with API digest/size, CRC and canonical bytes checked. Complete payloads and terminal metadata are retained locally; compact results and representative raw failures are committed. [Complete counts and preservation receipt](verification/k06-smp-sched/README.md). No old repetition counts toward the new run.

**Corrections under verification:** clock sampling retains actual PIT tick deltas instead of assuming that an idle return observes exactly the requested target. The five segments, median and exact 1% limit remain unchanged; diagnostics distinguish actual tick counts and TSC deltas on failure. The old transcript does not establish the cause of the observed drift. The missing executable final smoketest wrapper now invokes existing M2, K05 6/6, single-CPU K07 and RTC 5/5 regressions sequentially; host oracles, Clippy/fmt and nested evidence retention are restored. No quotas, guest timeouts, soak duration or repetition requirements are reduced.

**Progress:** **41/100 → 41/100**. K06 and the K07 SMP evidence extension remain open. Completion requires all 64 cases at exactly 50/50 (3200 unique guest runs), including 50 complete combined soaks, independent raw-artifact verification, a green aggregator and the subsequent executed `tools/gate.sh` with retained log. A new full workflow must start from zero on the corrected source; a failed-job rerun is insufficient.

**Executed before restart:** `native-smp` crossbuild, `cargo check --lib`, all 274 Python tool tests without skips, four orchestration tests, formatting, shell syntax, progress consistency and diff whitespace checks pass. Full Cargo tests also pass (525 tests), as does the existing SMP Clippy scope. An extra workspace-wide Clippy check reports an existing lint in unrelated `capability-core`; its failed log is retained without changing that code. [Host-check receipt](verification/k06-smp-sched/host-checks.json). No local guest was started.

**Additional FPU repair:** probe readiness now follows full initialization and the first successful GPR/x87/SSE checks. The stop condition waits for every live canary, while retaining the original non-canary soak exclusions and final register comparisons. M3/soak crossbuilds, 47 SMP host tests and 8/8 real-ELF mutation checks on each probe pass. [FPU host receipt](verification/k06-smp-sched/fpu-host-checks.json). The old artifact lacks slot/register values, so exact historical causality remains unproven.

**Observer correction under verification:** redundant all-process scratch scans on every entry are replaced by a check of the entering process before cleanup and a complete live-process scan after STOP. The user-loop pattern check, true return-permission walks, usercopy/shootdown/resource checks, locks, workload, 1000-switch minimum and deadlines remain intact. Observer sampling per process now follows its own entries. Crossbuilds, 47 SMP host tests and 8/8 artifact checks on each probe pass; no runtime speedup or timeout-recovery claim is made. [Observer receipt](verification/k06-smp-sched/observer-host-checks.json).

**Additional rejected restart:** [run 42731994218](https://github.com/simonvanieperen/BaryonOS/actions/runs/42731994218) is still running but cannot pass acceptance: `resched-deaf` is 46/50, with clock drift at iterations 5, 38, 40 and 43 (raw exit 35, no timeout). Every measured segment contains exactly 1000 IRQ0 ticks; the four medians exceed 1%. The exact original ZIP, API digest and four raw repetitions are preserved in the [incident receipt](verification/k06-smp-sched/incident-42731994218.json). Tick-edge overshoot does not explain these failures. No result from this source can count toward the next full restart.

**Per-AP observer candidate:** the clock fixture previously made all busy APs execute a locked RMW on one shared progress word. Each AP now maintains its own 64-byte-separated single-writer counter using ordinary atomic stores. The continuously busy AP loops, PAUSE, IRQ0 clock, five samples, 1% boundary and all deadlines remain intact; every online AP must show progress. The workload mix consequently loses shared observer contention; the separate synchronization/lock tests remain intact. This is a fixture correction, not proof of the historical cause or proof that clock drift is resolved. Crossbuild, 47 SMP host tests, fmt and diff checks pass; actual object code retains a per-AP store/PAUSE loop without the shared LOCK instruction. [Host/objectcode receipt](verification/k06-smp-sched/clock-per-ap-host-checks.json). [Complete restart 42741185181](https://github.com/simonvanieperen/BaryonOS/actions/runs/42741185181) is dispatched from zero on exactly `332275d97ed5f24f6d43174969a5667eb42f2afa` and pending behind the rejected run.

**Current scheduler correction (host-checked only):** ordinary nonblocking syscalls may retain their remaining quantum. The central selector rechecks actual deferred-timer/0xe8 generations after kill/wait refresh, removes only the caller from its temporary home queue without reordering peers, then uses the unchanged return validator, dispatch CAS, stack claim, FPU and CR3 path. Continuation does not renew event acknowledgements or fairness selections; yield, blocking, kill, migration, recovery and stop remain normal scheduling paths. Full Engine audits move from every unchanged timer entry to every completed capability syscall plus existing bootstrap/cleanup and final STOP boundaries. Snapshots, rollback/output checks and wipes are intact. [Host evidence](verification/k06-smp-sched/ipc-host-checks.json): 31 core tests plus one doctest, required SMP Clippy, 275 Python host tests, six CI tests, three matching crossbuilds and eight real-ELF checks per canary profile pass. Full workspace tests and native IPC throughput on this candidate are not yet established.

**Remaining native/runner failures:** the 59a run also has `m3-one-multi` 47/50 (three clock-drift failures on one vCPU), `m3-eight-single` 45/50 (five 60-second watchdogs) and `m3-migrate-eight` 49/50 (one earlier clock failure). Only 49/50 complete soaks pass: shard 7 repetition 5 times out at 660 seconds after K07 pressure evidence. Multiple late cases fail setup with signed Ubuntu Snapshot InRelease HTTP 503 before any build/guest; these are missing repetitions, not executed 0/50 outcomes. All available original artifacts and exact setup log are preserved. [Failure checkpoint](verification/k06-smp-sched/rejected-42731994218-checkpoint.json). Setup now retries transient requests three times within the existing job deadline, preserving URLs, package pins, signatures, hashes and guest limits; no native recovery is claimed.

**Clock boundary:** [primary-source investigation](verification/k06-smp-sched/CLOCK-INVESTIGATION.md) finds no established EOI/ExtINT/HLT defect. QEMU can process overdue PIT edges before a PIC acknowledgement; the PIC retains one pending bit. Historical causality still requires a trace. QEMU already requests 1 ns timer slack. ADR-053 explicitly permits only actually counted IRQ0s to increase the clock, so elapsed-PIT or TSC compensation is not silently introduced. The one-CPU failure remains unexplained by per-AP counters.

**Next:** commit/push the validated candidate, queue separately labelled clock diagnostics through the existing workflow, and dispatch the complete uninstrumented full matrix from zero on the same frozen source after diagnostics start. Diagnostics share the matrix concurrency lock, retain the unchanged 50 repetitions/60-second guest bounds and cannot enable acceptance jobs; trace results never count toward 3200. The existing pending full restart may be superseded before execution. Preserve the rejected run and independently verify every future matrix/soak/K07/aggregate/final gate result. No competing local QEMU matrix is permitted.

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
