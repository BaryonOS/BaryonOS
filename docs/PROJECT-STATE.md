# Current project status

## NEXT AGENT HANDOFF — K06 step 5f review findings fixed; full matrix pending — 12 November 2026

**Source:** branch `codex/k06-5f-complete-20261109` (PR #296); the commit carrying this entry holds the fixes. Decision and limits: [DECISIONS.md](DECISIONS.md), "review findings closed before acceptance". Run 43118230329 on `ae3e985` was cancelled because the fixes change kernel and tool sources; its 25 completed cases (50/50 each) and the K07 probe are diagnostics only ([receipt](verification/k06-smp-sched/cancelled-43118230329.json)).

**Fixes (scope verified against `docs/k06-step-specs`):**
- **(c) IQ-3 transitional locks:** `CpuLocal.transit` records the `IrqLock::id()` from before the ticket until the owner is published, and from clearing the owner until the ticket is released; `held_by()` stops on owner or transit for `KERNEL`/`ASPACE`/`RUNQ`/`SHOOTDOWN`. New negative case `quarantine-lock-pending`; the matrix is now 65×50 (3 250 guests).
- **(b) DG-2 harness:** the `!` terminal fallback is accepted by the harness verdict as by the oracle; other `!` lines and artifact-binding failures still fail (new host test, mutation-checked).
- **(d) K05 on four vCPUs:** explicit profile `native-smp-supervisor` plus five test variants; `run_supervisor.py --smp`; final-gate stage `k05-smp`. On SMP, `WAIT_IDLE_IRQS` counts BSP IRQ0s during idle, as on one CPU.
- **(e) Recovery from idle:** `smp_idle()` dispatches a pending recovery even with an empty run queue.

**Executed locally (QEMU 8.2.2, 4 CPUs; not acceptance):** `tools/tests` 310, `test_ci_workflows` 21, `cargo test -p smp-lock` 33, `fmt`; one boot each of `quarantine-lock-pending` and `quarantine-lock-held` (both `LOCK_HELD:10:CPU=2`), `spin-irq-off`, `ap-ipi-deaf`, `ap-shootdown-deaf`, `ap-timer-dead`, `resched-deaf`, `proc-shootdown-deaf`, `bsp-spin-irq-off`, `lock-stuck`, `native-smp` (6 cases), `m3-recovery-four`, `m3-kill-four`, `m3-ipc-four`, `m3-ipc-four-single`; `test_supervisor.py` 6/6 on one CPU and `test_supervisor.py --smp` 6/6 on four vCPUs (`SMP_HOMES` 6, 6, 6, 6, 15, 2; `wait` 83 idle IRQ0 ≤ 88 ticks). A first level-based version of (c) turned `spin-irq-off` into a false `LOCK_HELD` and was replaced.

**Next:** full `native-smp.yml` dispatch (`full_matrix=true`) on this head; all 65 cases 50/50, 50 soaks, K07, aggregate and final gate (now including `k05-smp`) must pass; then resolve the review threads and merge PR #296.

**Progress:** 39/100 → 39/100.

## Previous checkpoint — K06 step 5f run 43057843393, main merge and IPC watchdog — 11 November 2026

**Source:** branch `codex/k06-5f-complete-20261109` (PR #296). Merge commit `24d0d09a` brings in main `c520703b`, including the #307 squash `aeaf3cb7`; `.github/actions/setup-qemu` is byte-identical to main, so the signed QEMU/OVMF package bundle stays the install path. The merge conflicted only in this file and `docs/progress.json`. Kernel, tools and workflows are unchanged on this branch since `e398039e`.

**Run [43057843393](https://github.com/BaryonOS/BaryonOS/actions/runs/43057843393) on `e398039e` (rejected):** 72 of 79 jobs pass, the aggregate fails and the final gate is skipped. Details, artifact IDs and ZIP digests are in [the incident receipt](verification/k06-smp-sched/incident-43057843393.json).

- **Soak shard 9, iteration 5:** `SMP_CLOCK_DRIFT` in the idle precision phase (all APs halted). Five segments of exactly 1000 IRQ0, median 1 010 389 µs against 999 847 µs (1.05%, limit 1%). The other 49 soaks and every other case pass the same check. This matches the lost-edge mechanism in [CLOCK-INVESTIGATION.md](verification/k06-smp-sched/CLOCK-INVESTIGATION.md); no guest defect is demonstrated, so the kernel is unchanged. One failure in roughly 3 200 idle checks can recur in a new 64×50 run.
- **`m3-ipc-four`: 0/50**, each repetition stopped by the 180 s watchdog after `SMP:M3:GO`, with no FAIL marker. `m3-ipc-four-single` passes 50/50 at 71–84 s per boot.

**IPC diagnosis (local, QEMU 8.2.2, not acceptance):** multi-thread TCG takes 116 s on 4 host CPUs, 130 s on 3 and 327 s on 2 (`taskset`); single-thread TCG takes 103–109 s on 2 or 4. Uncommitted counters show the unchanged oracle passing with zero wait timeouts. The IPC phase runs at about one round per scheduler tick on 4 CPUs and 0.37 on 2; most time goes to endpoints that a tick preempts and requeues behind three CPU-bound canaries. Giving the canary PAUSE hint on one pass in 64 instead of every pass made the 2-CPU run slower (432 s) and was discarded. No kernel defect is demonstrated. The 180 s watchdog (ADR-081 R3-5: at least 2.5 times the measured time) was derived from 71.8 s on a faster host and is below the completion time on the hosted 4-vCPU runner.

**Decision (Simon, 11 November 2026):** `IPC_TIMEOUT_S` 180 → 660 s (`tools/run_smp.py`, ADR-081 R3-5 addendum 2 in [DECISIONS.md](DECISIONS.md)). The 2.5× rule stays: a hosted per-boot time above 264 s for `m3-ipc-four` keeps K06 open. The idle clock flake gets no code change; the next full run starts from zero and a recurrence rejects it. Rounds, oracle, FAIR limits, guest deadlines, `GUEST_TIMEOUT_S` and the 1% clock limit are unchanged.

**Executed:** merge and conflict resolution; the local runs above; after the decision `tools/tests` (309) and `tests/test_ci_workflows.py` pass, and one native `m3-ipc-four` boot passes in 108.6 s with `timeout_s` 660 (local QEMU 8.2.2, 4 CPUs, not acceptance); `python3 tools/progress.py --write` and `--check`. Pending at this checkpoint: the full 64×50 matrix dispatch on this commit, K07, the aggregate and `tools/gate.sh`.

**Next:** dispatch `native-smp.yml` with `full_matrix=true` on this head; read every failed job; record the hosted `m3-ipc-four` per-boot times against the 264 s bound; merge PR #296 only after the matrix, aggregate and final gate pass.

**Progress:** 39/100 → 39/100.

## Previous checkpoint — K06 step 5f clock check redesign — 11 November 2026

**Source:** branch `codex/k06-5f-complete-20261109` (PR #296), on top of `9698c379`. The commit that carries this entry holds the change; the full-matrix dispatch is recorded in the PR.

**Decision:** Simon chose to redesign the clock measurement instead of a paid 8-vCPU runner (option b below). The decision and its limits are in [DECISIONS.md](DECISIONS.md), "clock check split into idle precision and load progress". The 1% check is relaxed under load by decision; the hosted drift is not fixed. `clock()` in `kernel/src/m2/smp.rs` now runs two phases. First, with all APs halted, the existing 5-segment median must stay within 1% of the PIT period. Then every AP spins, and the idle BSP must count `CLOCK_SEGMENT` more IRQ0s before a 4× deadline; every AP must show work. Markers, the 1% limit, the repetitions and ADR-053 (no synthesized ticks) are unchanged.

**Local evidence (QEMU 10.2.1, TCG, not acceptance):** `tools/run_smp.py --repeat 5`, QEMU confined with `taskset`. The old check failed 0/5 on the same CPU counts:

| Case | Host CPUs | Result |
|---|---|---|
| `four` | 3 | 5/5 pass |
| `m4-quotas-four` | 3 | 5/5 pass |
| `four` | 2 | clock 5/5 pass; run 4/5 (r002 `SMP:ICR:LOST`) |
| `m4-quotas-four` | 2 | clock 5/5 pass; run 4/5 (r004 `SMP:ICR:LOST`) |
Also: `cargo fmt --all -- --check`, `test_smp_serial.py` (33 tests) and `test_gs_discipline.py` pass, and both kernel profiles build.

**2-CPU observation (open, not part of this change):** with 4 guest vCPUs on 2 host CPUs, the unchanged ICR phase (`icr_work`, ack wait bounded at 2^16 spin iterations) reported `SMP:ICR:LOST` once per case. The clock check passed in all ten 2-CPU runs. Before this change the 2-CPU runs stopped at the clock check, so they never reached ICR. The hosted runner profile has 4 vCPUs; if the full matrix shows `SMP_ICR`, it is the next diagnostic target. The ICR bound was not changed.

**CI on `9698c379`:** `host-quality` passes. About twenty QEMU jobs (including `qemu-smoke`, `native-smp` and `native-window-session`) failed in the step "Pinned QEMU and OVMF": runner DNS could not resolve the snapshot or `archive.ubuntu.com` (14:04–14:12 UTC, run 43034888367). That is a runner network fault, not a code result; the new push re-runs these jobs.

**CI on `ff2aad23`:** the same fault recurred at 14:26–14:35 UTC. All QEMU push jobs and all three attempts of matrix dispatch run 43038515659 failed in the apt step on `Temporary failure resolving` for `snapshot.ubuntu.com` and `archive.ubuntu.com`, on both `ubuntu-24.04` and `ubuntu-26.04`. crates.io and GitHub traffic in the same runs succeeded. The 64×50 body never executed, so that run is no SMP result. No pin or check was changed on this branch in response.

**Shared CI fix:** #307 makes the setup install from a pinned, signed package bundle, so CI no longer needs the Ubuntu hosts (hosted canary run 43057156214; see the #307 checkpoint below). That branch is merged into this one. The 64×50 matrix is dispatched again on the merge commit; its run ID and result are not recorded here yet.

**Open:** full 64×50 matrix, soaks, K07, aggregate and final gate on GitHub for this head. Host-quality on `2b6e81d6`/`9698c379`, `native-window-session` run 43031085350 and Copilot dynamic failure 43031108744 are not yet reviewed. Mirror the §4 "Klok" revision onto branch `docs/k06-step-specs` (`06-5c-2-bsp-timer-idle.md`). Load-phase clock precision remains unproven until it is measured on KVM or hardware.

**Progress:** 39/100 → 39/100 until the hosted matrix and final gate are green.

## Previous checkpoint — K06 step 5f QEMU 8.2 spin window and hosted clock drift — 11 November 2026

**Source:** branch `codex/k06-5f-complete-20261109` (PR #296). Earlier commits: `e875a30e` (R3-5 remote readiness wake, `IPC_TIMEOUT_S = 180`), `02e7c4d9` (merge; legacy `native-smp-legacy-m3`), `5a391465` (preserve-evidence and snapshot fallback), `a2f517a6` (local matrix runner script; no local full matrix is used as acceptance — verification runs on GitHub-hosted runners).

**Spin window fix:** the PR job `native-smp` on QEMU 8.2.2 failed with `SMP:LOCK:STUCK:10:CPU=1` because QEMU 8.2 `PAUSE` leaves the STI interrupt shadow set. All IF=1 spin windows are now `sti; nop; pause; cli` (see the ADR-081 addendum in [DECISIONS.md](DECISIONS.md)). Locally on QEMU 10.2.1: `two-single` 2/2, `cargo test -p smp-lock`, SMP host tests, `cargo fmt --check`. Proof on 8.2.2 comes from the PR job.

**Artifact contract (#294):** all K06 uploads use `./.github/actions/preserve-evidence` with `required-paths`; `smp-report.json` stores `stdout_sha256` instead of the duplicated transcript, so eight-CPU reports stay below 1 MiB. `setup-qemu` keeps the pinned snapshot first and falls back to `archive.ubuntu.com` with the same keyring, exact pins, `dpkg` check and firmware hashes; run 43022743756 passed that step.

**Hosted clock drift (blocker, open):** on head `2e77e2e3` both `native-smp` runs (43031091677, 43031085708; QEMU 8.2.2 and 10.2.1) still fail only with `M1:FAIL:SMP_CLOCK_DRIFT` (`smp-four`, about 1.8% against 1%). `SMP:LOCK:STUCK` no longer occurs, so the STI-shadow fix holds on 8.2.2. Stopping background services had no effect (load average 0.53 during the failure). Local reproduction, `m4-quotas-four`, 5 repetitions per row, QEMU confined with `taskset`:

| Host CPUs | Result | Drift |
|---|---|---|
| 4 | 5/5 pass | — |
| 3 | 0/5 | 2.0–4.0% |
| 2 | 0/5 | 20–27% |

Renicing the AP vCPU threads does not help: nice 10 on 3 CPUs gives 2.2–4.7%, nice 19 gives 1.9–2.9%. The cause is host CPU capacity for TCG emulation. A 4-vCPU guest on the 4-vCPU hosted runner (2 cores × SMT) loses IRQ0 edges. The 1% limit, the test content and the repetitions stay unchanged, and ADR-053 forbids synthesized ticks.

**Host quality:** `2b6e81d6` fixes clippy `manual_div_ceil` in `capability-core` (`MAX_NODES.div_ceil(64)`). Locally the CI clippy scope, `cargo test -p capability-core` and `cargo fmt --check` pass.

**Next (Simon's decision):** either (a) approve a larger hosted runner with at least 8 vCPUs (paid), or (b) design a separate clock measurement within ADR-053 (for example, the BSP measures while the APs idle, and CLOCK_LOAD gets its own progress criterion). This is a decision record, not a silent change. Do not re-dispatch the full matrix without a new code change. Acceptance requires all 64 cases at 50/50, 50 soaks, K07, a green aggregate and the final gate.

**Progress:** 39/100 → 39/100 until the hosted matrix and final gate are green.

## Previous checkpoint — K06 step 5f acceptance repair — 10 November 2026

**Copilot execution handoff:** [self-contained continuation prompt](verification/k06-smp-sched/COPILOT-HANDOFF.md). Continue the existing two GitHub runs and exact branch; do not dispatch a duplicate of pending run `42818153989`. The final-gate verifier is now tracked beside the matrix verifier, so continuation does not require the previous Cloud workspace. At the handoff check the old run has 29 ordinary successes, two known failed cases and ten successful soak shards; both IPC jobs are running. The candidate run is pending with zero jobs. These live counts must be refreshed before acting. GitHub workflows remain untouched by this handoff.

**Current frozen source and queued restart:** `54a9d5c17291b862e30240e49ed4cde0d479315b`, source-v3 `727676734c47e4efabedb49831763d71c1cd08c5698704de4e1c12a26e57232b`, on the same existing branch. [Full run 42818153989](https://github.com/BaryonOS/BaryonOS/actions/runs/42818153989) was dispatched from zero with `full_matrix=true`, diagnostics off, and is pending behind rejected run `42809499884` in the unchanged branch concurrency group. Its jobs have not started at this checkpoint. No old repetition is reused. [Frozen restart receipt](verification/k06-smp-sched/restart-42818153989.json).

**Source and rejected active run:** existing branch `codex/k06-5f-complete-20261109`, tested native source `3dd6194f02231e8f1db61940242d8d90318daf1d`, source-v3 `05f680b768def10a2495e5958fa3af2fb95b50be19c755cce728f8c850ac887a`. [Full restart 42809499884](https://github.com/BaryonOS/BaryonOS/actions/runs/42809499884) remains running but is rejected: `proc-shootdown-deaf` is 48/50 with RUNQ held at quarantine; `bsp-spin-irq-off-control` is 49/50 with clock drift. Independent partial readback verifies 1547/1550 attempts, including all 50 complete soaks; no repetition is reused. The repository is now `BaryonOS/BaryonOS` with the same repository ID 1435549142. After organization installation 180498948, fetch, push and workflow dispatch succeed. [Exact failures and retained ZIP bindings](verification/k06-smp-sched/rejected-42809499884-checkpoint.json).

**Corrected fixture candidate:** source-v3 `727676734c47e4efabedb49831763d71c1cd08c5698704de4e1c12a26e57232b`. The deaf-process test first proves its L0-required lock-free precondition through a real kernel setup round and scheduler-admission closure; pending timer/reschedule entries cannot acquire RUNQ afterwards. The A-round still receives no flush/ack, and frame quarantine, IQ-3 checks and INIT remain intact. Clock-load APs execute 64 dependent integer steps before each separate progress store and PAUSE, preserving continuous work and every clock criterion while reducing TCG CPU-loop exits per work step. Historical drift causality and native recovery remain unproved. Four matching crossbuilds, 275 host tests, 48 focused oracles, fmt/diff and actual ELF checks pass. [Host receipt](verification/k06-smp-sched/fixture-repair-host-checks.json).

**Cancelled attempt:** run `42803221546` on the same source ended cancelled: eight ordinary jobs and the environment probe report success, 55 ordinary jobs and all ten soak shards are cancelled; aggregate and final gate are absent. All nine original ZIPs are retained locally with API SHA-256/size and ZIP CRC verified. No repetition is reused, and no native acceptance is inferred from API statuses alone.

**Rejected preceding source:** [Full restart 42748931069](https://github.com/BaryonOS/BaryonOS/actions/runs/42748931069) is rejected on frozen source `22dfceaf1fe4b1c4a499440b4199b951d85025f9`, source-v3 `58f7cc8a20db1c0f3afe30241fb14c125e7c75b62b984c6412056a19923a3eae`: 60/63 ordinary jobs and all ten soak shards pass, both IPC jobs are 0/50, and `m3-canary-control` is 49/50. Environment/K07 and both regular K07 jobs pass, but the aggregator fails and the final gate is skipped. None of its repetitions is reused. Diagnostic `42746512322` on `3b6c9a68` passed its 50 traced repetitions and was independently read back; it also supplies zero acceptance repetitions.

**Rejected evidence:** [run 42723127310](https://github.com/simonvanieperen/BaryonOS/actions/runs/42723127310) is completed with failure on incoming source `f808d591`, source-v3 `759320893ea058b90d0bc764cd2b9534734827131a212ad5ba27f5501db6e8ac`. Independent raw-artifact readback checks all 3200 guests: 2976 pass and 224 fail across ten ordinary cases (63 FPU-image mismatches, 159 watchdogs and two clock-drift failures). K07 M4/qsim each pass 50/50 and all 50 complete soaks pass on that rejected source. The aggregator fails; the final gate is skipped. All 74 original artifact ZIPs remain intact, with API digest/size, CRC and canonical bytes checked. Complete payloads and terminal metadata are retained locally; compact results and representative raw failures are committed. [Complete counts and preservation receipt](verification/k06-smp-sched/README.md). No old repetition counts toward the new run.

**Corrections under verification:** clock sampling retains actual PIT tick deltas instead of assuming that an idle return observes exactly the requested target. The five segments, median and exact 1% limit remain unchanged; diagnostics distinguish actual tick counts and TSC deltas on failure. The old transcript does not establish the cause of the observed drift. The missing executable final smoketest wrapper now invokes existing M2, K05 6/6, single-CPU K07 and RTC 5/5 regressions sequentially; host oracles, Clippy/fmt and nested evidence retention are restored. No quotas, guest timeouts, soak duration or repetition requirements are reduced.

**Progress:** **41/100 → 41/100**. K06 and the K07 SMP evidence extension remain open. Completion requires all 64 cases at exactly 50/50 (3200 unique guest runs), including 50 complete combined soaks, independent raw-artifact verification, a green aggregator and the subsequent executed `tools/gate.sh` with retained log. A new full workflow must start from zero on the corrected source; a failed-job rerun is insufficient.

**Executed before restart:** `native-smp` crossbuild, `cargo check --lib`, all 274 Python tool tests without skips, four orchestration tests, formatting, shell syntax, progress consistency and diff whitespace checks pass. Full Cargo tests also pass (525 tests), as does the existing SMP Clippy scope. An extra workspace-wide Clippy check reports an existing lint in unrelated `capability-core`; its failed log is retained without changing that code. [Host-check receipt](verification/k06-smp-sched/host-checks.json). No local guest was started.

**Additional FPU repair:** probe readiness now follows full initialization and the first successful GPR/x87/SSE checks. The stop condition waits for every live canary, while retaining the original non-canary soak exclusions and final register comparisons. M3/soak crossbuilds, 47 SMP host tests and 8/8 real-ELF mutation checks on each probe pass. [FPU host receipt](verification/k06-smp-sched/fpu-host-checks.json). The old artifact lacks slot/register values, so exact historical causality remains unproven.

**Observer correction under verification:** redundant all-process scratch scans on every entry are replaced by a check of the entering process before cleanup and a complete live-process scan after STOP. The user-loop pattern check, true return-permission walks, usercopy/shootdown/resource checks, locks, workload, 1000-switch minimum and deadlines remain intact. Observer sampling per process now follows its own entries. Crossbuilds, 47 SMP host tests and 8/8 artifact checks on each probe pass; no runtime speedup or timeout-recovery claim is made. [Observer receipt](verification/k06-smp-sched/observer-host-checks.json).

**Additional rejected restart:** [run 42731994218](https://github.com/simonvanieperen/BaryonOS/actions/runs/42731994218) is still running but cannot pass acceptance: `resched-deaf` is 46/50, with clock drift at iterations 5, 38, 40 and 43 (raw exit 35, no timeout). Every measured segment contains exactly 1000 IRQ0 ticks; the four medians exceed 1%. The exact original ZIP, API digest and four raw repetitions are preserved in the [incident receipt](verification/k06-smp-sched/incident-42731994218.json). Tick-edge overshoot does not explain these failures. No result from this source can count toward the next full restart.

**Per-AP observer candidate:** the clock fixture previously made all busy APs execute a locked RMW on one shared progress word. Each AP now maintains its own 64-byte-separated single-writer counter using ordinary atomic stores. The continuously busy AP loops, PAUSE, IRQ0 clock, five samples, 1% boundary and all deadlines remain intact; every online AP must show progress. The workload mix consequently loses shared observer contention; the separate synchronization/lock tests remain intact. This is a fixture correction, not proof of the historical cause or proof that clock drift is resolved. Crossbuild, 47 SMP host tests, fmt and diff checks pass; actual object code retains a per-AP store/PAUSE loop without the shared LOCK instruction. [Host/objectcode receipt](verification/k06-smp-sched/clock-per-ap-host-checks.json). The corresponding earlier full restart `42741185181` was superseded while pending and supplied no native acceptance.

**Current scheduler/IPC correction (host-checked only):** ordinary nonblocking syscalls may retain their remaining quantum. Run 42748931069 shows that this alone does not meet the 10000-round IPC bound. The replacement proves capability-free SEND/RECV success under KERNEL and ASPACE before omitting a snapshot that successful calls never compare; every possibly failing, capability-bearing, malformed or usercopy path keeps the exact snapshot, comparison and observable wipe. Complete Engine audits remain at bootstrap, exit/cleanup and final STOP instead of running a full-table observer scan after every successful syscall. Mode20 receives a PAUSE host-vCPU hint without guest Yield, blocking, FIFO priority or reduced IPC count. Capability tests, focused Python oracles, matching IPC/soak crossbuilds and actual-ELF checks pass; native throughput remains unproved.

**Completed rejected restart:** the 59a run ended with failure. Independent raw readback verifies 2500 executed attempts: 2386 pass and 114 fail (106 watchdogs, eight clock-drift failures). Another 700 repetitions are missing after 14 HTTP 503 setup failures. Exactly 43 cases pass 50/50; seven cases have native failures. Both IPC cases are 0/50 with 60-second watchdogs; `m3-eight-single` is 45/50; one-CPU `m3-one-multi` is 47/50; only 49/50 full soaks pass, and regular K07 quota/qsim cases are missing. Aggregate fails and the final gate is skipped. All 60 original ZIP artifacts are API-hash/size/CRC/canonical-byte verified and retained. [Final counts](verification/k06-smp-sched/rejected-42731994218-rejected-run-summary.json) and [complete rejected readback](verification/k06-smp-sched/rejected-42731994218-final-verification.json.gz). Three APT retries preserve the same signed snapshot, pins, hashes and execution bounds; recovery must still be demonstrated natively.

**Clock boundary:** [primary-source investigation](verification/k06-smp-sched/CLOCK-INVESTIGATION.md) finds no established EOI/ExtINT/HLT defect. QEMU can process overdue PIT edges before a PIC acknowledgement; the PIC retains one pending bit. Historical causality still requires a trace. QEMU already requests 1 ns timer slack. ADR-053 explicitly permits only actually counted IRQ0s to increase the clock, so elapsed-PIT or TSC compensation is not silently introduced. The one-CPU failure remains unexplained by per-AP counters.

**Canary hint (host-checked only):** a single PAUSE follows the complete ordinary mode19 GPR/x87/SSE check and readiness store. The earlier mode20 IPC and explicit migration-yield branches bypass it; the legacy mode0 loop is unchanged. This changes QEMU host-vCPU interleaving without guest Yield/HLT/block or extra process switches. The 1000 real-switch requirement, loads, deadlines and final checks remain. Three matching crossbuilds, 48 SMP host tests and eight real-ELF tests per canary profile pass; actual disassembly and independent source review confirm placement. [Receipt](verification/k06-smp-sched/pause-host-checks.json). Runtime improvement and historical causality remain unproved; MTTCG may incur BQL overhead.

**Clock correction under verification:** the failed canary-control repetition counted exactly 1000 IRQ0 entries in every segment, while the old observer read TSC only after returning from the final interrupt. The IRQ0 stub now records TSC immediately before publishing each real counted tick. Segment count, actual tick normalization, median, exact 1% limit and deadline remain unchanged; no elapsed tick is synthesized. Failure diagnostics retain observer lag. This removes only post-interrupt observation latency and still makes missing/coalesced IRQ0 entries lengthen a 1000-entry interval. Native recovery remains unproved.

**Next:** monitor queued run `42818153989` and preserve the completing rejected run `42809499884`. Do not cancel either or run a local QEMU matrix. For the corrected source require all 64 cases exactly 50/50, 3200 raw verified guests, regular K07, all 50 complete soaks, aggregate success and the subsequent actually executed tools/gate.sh with retained log. Compare aggregate and gate artifacts independently. Any red or missing repetition keeps K06 open.

## Checkpoint (PR #307) — CI QEMU/OVMF package bundle — 11 November 2026

**Source:** branch `ci/qemu-apt-bundle` (PR #307) from main `c07506c8`;
builder and action commit `f26cd05623368c1a834be538e38a2aa30f5e1df1`.

**Problem:** every QEMU workflow (#296, #304 and the backlog) failed in
"Pinned QEMU and OVMF". snapshot.ubuntu.com timed out over HTTPS, and DNS for
the Ubuntu mirrors failed on hosted runners.

**Change:** `.github/actions/setup-qemu` installs first from a hash-pinned
release bundle (`ci-qemu-apt-20261111-r2`). APT still verifies the Ubuntu
signature, index hashes and package hashes. Pins and firmware checks are
unchanged, and the signed snapshot and archive remain warning-level fallbacks.
The Ubuntu 26.04 profile from #296 is included. See
ADR-CI-QEMU-BUNDLE-20261111 and
[the bundle report](verification/ci-qemu-bundle-20261111.json).

**Evidence:**
- Both bundles were built twice from the signed archive with identical
  results, checked with gpgv and installed download-only from the bundle alone:
  - noble: 89 debs, sha256 `f54c031b…`
  - resolute: 113 debs, sha256 `45026b96…`
- The release is published; anonymous downloads match the pins.
- Hosted canary run 43057156214 passed on ubuntu-24.04 (with swtpm) and
  ubuntu-26.04. The Ubuntu mirrors were blocked, the bundle was the APT source
  and no fallback warning appeared.
- 21 CI-workflow tests pass.

**Corrected on the way** (three failed canary runs, recorded in the report):
- The action now limits APT to `Targets: Packages`.
- The first release `ci-qemu-apt-20261111` had percent-encoded `.deb` names
  and is superseded; its assets are unchanged.
- The 26.04 bundle now carries `ibverbs-providers`.

**Open:**
- The affected workflows have not been rerun on this change beyond the canary.
- The K06 64×50 matrix on #296 has not been rerun.
- A later runner image can need a package the bundle lacks. The action then
  warns and falls back to the signed network path.

**Acceptance:** 39/100 → 39/100; CI infrastructure only.

**Next:** merge #307 when its required checks are green, merge main into #296
and rerun the K06 matrix there.

## NEXT AGENT HANDOFF — PR #210 Veld input-ownership repairs — 11 November 2026

**Source:** PR head `a8624b8` merged with `main` `c520703b`, then with `main`
`1a3feb92` (after #296 and #314). Only `docs/PROJECT-STATE.md` conflicted;
both sides were kept and the duplicated #299 heading collapsed onto
byte-identical content. Repair commits `eb34cfb` and `cdbd187`. After the
second merge the default, `native-window-session-probe` and
`native-window-session` kernel builds passed locally.

**Repairs of the five open review threads:**
- Alt+Tab during an Alt+F7 keyboard move now rolls the move back before the
  switcher opens, so the Alt release can no longer reach `fail(17)`.
- `Scene::veld_owns_pointer` keeps a dismissed overlay's button-up tail inside
  Veld, so the next press reaches the app; a trusted cover or lock still takes
  priority (`!shell.blocks_apps()`).
- The document menu owns every key it consumed until that key's release
  (`menu_owned_keys`, indexed by the `repr(u8)` `PhysicalKey`), fixing both the
  Enter-repeat draft replacement and the Backspace-repeat deletion after
  dismissal.
- The CodeQL cleartext-logging alert in `trusted_guard.rs` is closed by
  dropping the formatted trusted key from an assertion message.

**Local evidence:** window-session 112/112 and `ui/native` 110/110 Rust tests,
warning-denying Clippy and rustfmt for both crates, `progress.py --check` and
`git diff --check` passed (builder). Each of the four new regressions fails
when its fix is reverted. The integrator independently re-ran `trusted_guard`
(7/7), `document_menu` (14/14), rustfmt and the diff check. Earlier repairs
(distinct 100%/200% evidence artifacts, Materiaal RX 157,161 bytes in 39 pages
under the 160-KiB cap) are unchanged.

**Acceptance:** **39/100 → 39/100**. No native runtime evidence is claimed;
both hosted 1800-second native-window-session profiles must be green on the
published head before a squash merge.

**Next:** evaluate the current-head hosted checks; squash-merge with the
expected head only after both native profiles and the required gates pass.

## Checkpoint (main) — PR #299 ChaCha20 production unification — 10 November 2026

**Source:** this local candidate is based directly on the actual PR #297
squash `2d17a3870c0f22254af0a749d3f29128fe2e11ae` (tree
`563b3ec57355ca124672afe94de79808f9e4fc70`), which follows the PR #257
squash. Its semantic donor is local commit
`5ba29a843d7087ff637fb2856677cfd17da6f74a` on reviewed #297 candidate
`3cb1e16d0f5f1a91784d0761ec0b5c5d7f60606f`. This replay has not been
published or treated as hosted evidence.

**Implementation:** production now resolves one pinned `chacha20=0.10.2` for
the DRBG, maintained RNG and AEAD closure. The direct pinned `cipher=0.5.2`
dependency enables its `zeroize` feature so the stream wrapper and buffered
keystream implement `ZeroizeOnDrop`; compile-time assertions retain this
boundary. `chacha20=0.9.1` remains only as the independent dev-test alias
`chacha20-legacy` for the byte-exact old DRBG oracle. Product builders use the
single portable `chacha20_backend="soft"` selector.

**Compatibility and provenance:** the new implementation reproduces the old
DRBG bytes and cold-restores a fixed pre-migration persisted blob, including
decryption of its existing record key. All four locks and crate checksums are
contract-tested; the three consumer product locks contain 0.10.2 but not
0.9.1. The reviewed upstream crates are `no_std`, require Rust 1.85, use MIT
or Apache-2.0 and have no build script.

**Local evidence:** all 71 keyring tests and four crypto-contract tests passed.
Warning-denying Clippy, formatting, strict preflight, progress consistency and
`git diff --check` passed. Locked offline fetches, consumer tests and Clippy,
and the keyring, SDK-client, example, system and random-probe freestanding
release builds passed. Independent read-only review found no security,
compatibility, lockgraph or provenance blocker.

**Acceptance:** **39/100 → 39/100**. No native B01, RANDOM, FR-I01 or B03 gate,
QEMU run, physical entropy result or new product criterion is claimed.

**Next:** finish this replay on the actual #297 squash, publish with an exact
lease and evaluate only current-head hosted checks and review. Squash-merge
with the expected head only after they are green.

## Historical checkpoint — PR #257 fail-closed publication inventory — 10 November 2026

**Source:** exact reviewed head `590d9c3b589435fff6bf794c85908bbb37a98382`
was squash-merged as `1e3062fc1258ba6d098e70bcb13d64de1fbd90a2`;
the trees are byte-identical at `17696fb42e386366db39257517908acf80ae03fc`.
The audit remains a local evidence tool and never grants publication clearance
by itself.

**Implementation:** the three-phase scanner binds the exact advertised Git
tips, Git history and metadata, Actions inventory, scanner identity, retained
bytes and resource receipts to one immutable snapshot. Git replacement refs
are disabled, public ref discovery receives no inherited credentials, and the
private route exposes its token only to a transient askpass process. Retained
state uses reconciled O(1) accounting with bounded atomic replacements. A
final audit-success record is now invalidated before every retry and is written
only after the command deadline and persistent usage ledger have closed
successfully; a failing retry cannot leave an earlier success active. The
canonical repository binding is now `BaryonOS/BaryonOS` at immutable repository
ID `1435549142`; both historical owner/name routes are accepted only through a
GitHub redirect to that exact ID and current metadata.

**Validation:** all 86 publication-audit unit tests pass, including real Git
replacement-ref isolation, exact advertised-HEAD binding, retained-file
tampering, repository-transfer binding, deadline and state-quota failures,
failed re-finalization and the ordered success commit. The live public API
confirms the canonical name, clone URL and unchanged repository ID; both old
routes return the expected ID redirect. Python bytecode compilation, the strict
preflight and the staged-diff check pass. No live GitHub/Gitleaks inventory was
executed for this integration, so there is no claim that the repository is
clear for publication and no historical receipt is promoted to current evidence.

**Acceptance:** **39/100 → 39/100**. This strengthens audit tooling and its
fail-closed evidence boundary but completes no OS product criterion and adds
no native runtime, physical-hardware or human acceptance.

At merge time 45 hosted checks were green and the native-window job was still
in progress; that unfinished job is not positive evidence. Any real publication
audit must run against its exact advertised source tip and must keep
`publication_clearance` false.

## Checkpoint (main) — PR #218 H05 step-2 replay after #269 — 10 November 2026

**Source:** local branch `codex/pr218-final-20261110`, based directly on
`main` `1864c283a8fe0e732044e361981b9c01d342e00b` (tree
`f5ed6ad39f32505b8215d9c65e8dc91582d56dc1`). The semantic donor is PR
#218 head `f1901125b34a2543571a47b937c992d18aeda7cc`; the bounded local port was
replayed from `ae5bf09b12db83ce198772e8f788ce83d46cec22`. The already merged PR
#213 ACPI/FADT/DSDT/S5 implementation was not duplicated. No remote branch,
review thread or pull request was mutated.

**Implementation:** the port adds the sole non-delegable POWER capability,
volume IPC v6 `SHUTDOWN`, syscall `0x90`, the final storage/keyring gate,
bounded fail-closed refusal evidence, SDK/example support and three native
POWER profiles. The former default-to-POWER selector branch now rejects every
unknown or composite selector before state changes; a full-engine atomicity
test covers `POWER|TRANSFER`, an unknown high bit and `u64::MAX`. Current
main's newer keyring manifest is retained. Its capability benchmark expectations
were corrected from 21/29/9 to the observed 25 handles, 35 nodes and 11
endpoints instead of reverting the new administrative channels.

**Integration:** #243's fail-closed `qemu_accel()` selector remains the only
way to opt into KVM; the POWER runner no longer hard-codes TCG. The evidence
verifier retains the KVM pilot and both H05 jobs, for 18 archive workflows and
20 preservation calls. The English documentation from #269 remains English.
Its progress register and generated README/SVG remain byte-unchanged at 39/100.

**Current replay validation:** capability-core passed 87 tests with one ignored
benchmark; volume-core passed all 72 tests; the SDK workspace passed 116 tests.
Warning-denying Clippy and formatting passed for the affected Rust workspaces.
The 19 system-harness, eight CI-evidence, 21 CI-workflow, ten KVM-selector,
34 system-capture and 18 cap-benchmark harness tests passed. Pinned actionlint
1.7.12, the staged-diff check and the progress consistency check also passed.
The current candidate source fingerprint is
`2859165807d303b8dbe2f977899b96700c7eccbc8bc2348f055fe5aed8d37394`.

**Native evidence:** strict preflight, the default build and the TCG smoke
passed (`M1:PASS`, raw exit 33). All three POWER profiles built with three
ELF load segments. The source-bound six-stage POWER matrix passed: normal S5
exited raw 0; flush-error and flush-timeout exited raw 33 without S5; all three
cold recoveries exited raw 33 with unchanged recovery-image hashes. Its report
is `build/pr218-final-power-gate/power-system-gate.json`. The native capability
benchmark passed raw 33 with digest `0x77cd1386b23a996b` for all variants and
an unchanged disk; its report is
`build/pr218-final-cap-bench/cap-bench-report.json`. The standalone #213 path
passed five real S5 exits and two corrupt-DSDT rejections.

The separately requested `smp-lock` suite is not green evidence. With no QEMU
or competing heavy task, `mutual_exclusion_under_contention` still consumed
approximately four CPUs for more than three minutes. The candidate suite was
interrupted with exit 130. The exact test was then repeated in a clean detached
worktree at parent `1864c283a8fe0e732044e361981b9c01d342e00b`; it reproduced
the same ten-thread, approximately 397% CPU hang and hit the bounded 180-second
timeout with exit 124. The replay changes neither `crates/smp-lock` nor
`kernel/src/m2/lock.rs`, so this is a pre-existing parent-baseline failure,
not passing evidence and not a POWER-port regression.

**Acceptance:** **39/100 → 39/100**. This is QEMU/TCG and host evidence for a
bounded H05 substep. It does not establish complete coordinated production
shutdown, S3 sleep/wake, thermal/battery behavior, independent human security
review or physical-hardware support; H05 remains open.

**Next:** review the final diff and identities, create the single local noreply
commit and publish only through the parent integration flow. Current-head
hosted checks and review remain required before any guarded squash merge.

## Checkpoint (main) — Issue #292 target-flag repair — 10 November 2026

**Source:** PR #295 local integration commit
`16d2fb3d6c3d2e66b66888fcf5577b316a9f580a`, rebased onto the #287 squash
base `e1eefaf8e0ab061c90e9fefe89ef16f80917a343`. The artifact-store workflow
uses additive Cargo configuration so the freestanding target retains its code
model, relocation model, linker arguments, and scalar CPU-feature restrictions.
`.cargo/config.toml`, kernel runtime behavior, capabilities, and dependencies
are unchanged.

**Local integration evidence:** twelve workflow contract tests, nine
artifact-store contract tests, warning-denying Clippy, formatting, the
freestanding library build, strict preflight, the standard build, QEMU smoke,
and the progress check passed on tree
`2c04a809d890e7eb8e4f6041281aeda41f9e1c7b`. The verbose compiler invocation
retained the required code model, relocation model, no-red-zone setting, four
linker arguments, and scalar target-feature restrictions. Source hashes,
commands, log hashes, and evidence limits are recorded in
[`verification/pr295-main-integration-20261110.json`](verification/pr295-main-integration-20261110.json).
The original focused record remains in
[`verification/artifact-store-target-flags.md`](verification/artifact-store-target-flags.md).

**Acceptance:** **39/100 → 39/100**. This CI correction closes no OS product
criterion and is not native runtime evidence.

**Next:** publish with an expected old-head lease. Require current-head hosted
review and checks before a guarded squash. Issue #292 remains subject to the
repository's normal issue lifecycle.

## Historical checkpoint — PR #167 switcher review repair — 10 November 2026

**Proposal:** VELD-08 remains a bounded Alt+Tab subgate. It does not accept the
complete native session, recovery, accessibility, or interaction-budget criteria.

**Implementation:** PR #167 now includes current `main` `5456296` and binds each
tested key edge to an acknowledged pointer marker with an exact input-counter
delta. The native work-frame reference additionally requires `SHELL_READY` and
two consecutive byte-identical full-screen captures. No capability, kernel ABI,
dependency, storage right, or default gate was changed.

**Test evidence:** `python3 -B tools/test_window_session.py` passed 23 host tests;
`python3 tools/progress.py --check` and `git diff --check` passed. Strict preflight
reported missing Rust, QEMU, and OVMF; the build and smoke commands consequently
failed before execution. Exact source, commands, exit codes, and evidence limits
are recorded in
[`verification/pr167-switcher-review-fix-20261110.json`](verification/pr167-switcher-review-fix-20261110.json).

**Unknown:** The final PR head still requires current-head CodeRabbit/CodeQL and
required checks, including native switcher runs at 100% and 200%. Historical
native receipts do not verify this repair. **Progress remains 39/100.**

**Next:** evaluate only the final published head checks, resolve the verified
review thread, and squash-merge with an expected head SHA when all required
gates are green.

## H05 native ACPI S5 integration — PR #213 (9 November 2026)

The opt-in `native-acpi-s5` kernel profile connects the existing
`drivers/acpi-power` parser to bounded FADT/DSDT discovery and ACPI S5 port
writes in the fixed QEMU/OVMF configuration. Invalid firmware performs no PM1 control-port writes; serial diagnostics
and debug-exit I/O remain active. The change adds a reset-sensitive QEMU oracle,
five positive and two negative native runs, and the `native-power.yml`
workflow. Source-bound original evidence is retained in
[`verification/h05-acpi-s5/`](verification/h05-acpi-s5/).

CI evidence now retains 17 tracked workflow files and 18 preservation
calls, alongside the English ADR-076 and verification record. The 22 green
workflow results on the former PR head `9d88326` predate integration with
main and are not evidence for the new merge commit. No physical-hardware,
service-initiated power-off, S3 sleep/wake, independently traced negative
PM1 writes or complete ACPI support is claimed.
**The current acceptance score remains 39/100; H05 is still open.**

**Historical handoff (PR #213):** Main `04c8b224e3062400288a353056b75abdd3bcbba5`
and the prior PR source `9d88326be637adccfeccc743f31b3c67185e16a8`
were integrated at `16d8346a2968f0315334e8190eb66e8103b14cf6`;
`ee047c512c8637f57b72528515c564f9092f164a` fixes the CI evidence counts.
The SMP legacy profiles and 17-workflow/18-call evidence contract are retained.
No local Rust, QEMU, strict preflight, build or smoke gate ran in this connector
session. Confirm all applicable CI and security-review results on the latest
source, including native-power, host-quality and qemu-smoke, before squash merge.
Physical-hardware evidence remains missing, so H05 stays open.

## PR #167 — VELD-08 window-switcher integration — 9 November 2026

**Source:** Existing PR #167 at pre-integration head `fdaef7acce36d1865a79406314a4c1159de172fd`, merged in the current integration branch with `main` at `316d4f10470efe16bfdf314304a410f8b1150833`. The implementation preserves compositor-owned window thumbnails, focus-on-confirmation, bounded input handling and explicit opt-in 1x/2x native probes. The latest corrections retain trusted-shell routing, remove minimized-window hiding after selection, require complete app frames before the native assertion, preserve key-edge baselines, and release pointer capture after the full outside-click gesture.

**Evidence:** Local integration commit `09f982c348a9ba98baded4f3401defb2a38cfd0b` passed `tools/progress.py --check`, strict preflight, `tools/test_window_session.py`, the complete window-session host suite, format and warning-denying Clippy, `tools/build.py`, and `tools/run.py --smoke`. Native switcher runs at scales 1 and 2 each passed all 18 phases with raw QEMU exit 33, `switcher_gate=true` and intentionally `native_gate=false`; both reports bind source fingerprint `3e669760446ce83a47d45cf4e5e2d3584860731d9043c53e8a7bc17a6fe56dae` to that commit. GitHub email privacy required a metadata-only rewrite to publication commit `b25a7d10b50240f00b44569ffafba65ae0a064d2`; both commits have identical tree `f1cb8de749cb58b7209b77f7c5271e52f0ca726f`. Compact evidence is stored in [`veld-switcher-20261103/current-main-integration-20261109.json`](verification/veld-switcher-20261103/current-main-integration-20261109.json).

**Evidence boundary:** GitHub reported 47 checks on pre-integration head `fdaef7acce36d1865a79406314a4c1159de172fd`: 46 successful and one neutral (`Graphify Formal Verification`). CodeRabbit completed successfully and no unresolved review thread was present. Those GitHub results do **not** verify publication commit `b25a7d10b50240f00b44569ffafba65ae0a064d2`; current-head CI and review remain required after publication. Historical two-scale receipts remain bound to `537fee48c1c31155a51a997cd7a2236b159e9ac7`. No physical-device, human usability or independent security acceptance is claimed.

**Progress:** **39/100 → 39/100.** A successful switcher subgate does not close the complete native integration, accessibility or recovery criteria.

**Historical handoff (PR #167):** Publish the integrated source without rewriting history. Confirm current-head CodeRabbit, CodeQL and required GitHub checks, then squash-merge with the expected head SHA.

## PR #154 — reviewed K06 and experimental M3/M4 SMP integration — 9 November 2026

**Source:** existing PR #154 `simonvanieperen-studious-spoon` at tested source `fcaa2ef44b7bdfcabbc6368149105f978cade27a` against `main` `540f7286cea57f9fe49c55b896aa9a42fd4b4b35`. K06 MADT (up to eight CPUs) remains the default `native-smp` profile. The distinct SMP Info/M3-M4 (up to four CPUs) remains opt-in under `native-smp-legacy` and its M3/M4 profiles, without changing the default boot ABI. Duplicate UEFI ACPI 2.0 configuration entries are rejected; per-CPU safety and bounded memory-pressure recovery assertions are retained.

**Evidence on source `fcaa2ef44b7bdfcabbc6368149105f978cade27a`:** GitHub Actions [K06 native SMP and experimental M3/M4 SMP](https://github.com/simonvanieperen/BaryonOS/actions/runs/42667770606) completed successfully, including the full 15-stage experimental native and negative matrix. [Native system integration](https://github.com/simonvanieperen/BaryonOS/actions/runs/42667770871) completed successfully, including normal/NX/guard/peer/OOM physical memory tests and actual recovery. [CI](https://github.com/simonvanieperen/BaryonOS/actions/runs/42667770942) passed `qemu-smoke`, but `host-quality` failed a 16-versus-17 evidence invocation count because the new experimental SMP evidence job is now included. The isolated contract-test correction follows in this PR checkpoint. Earlier checks are not acceptance evidence for this later documentation/test HEAD.

**Acceptance:** **39/100 → 39/100.** K06 remains open: the experimental M3/M4 profile is still a separate opt-in implementation and does not establish the full shared concurrent production scheduling/address-space model or physical hardware support. B01 positive native management recovery also remains open. No safety or product acceptance point is added by CI success alone.

**NEXT AGENT HANDOFF:** verify fresh `host-quality`, `qemu-smoke`, K06 and experimental SMP, native-system and CodeRabbit/CodeQL checks after the evidence-test and status commit. Run or confirm `python3 tools/progress.py --check`, `python3 tools/preflight.py --strict`, `python3 tools/build.py`, and `python3 tools/run.py --smoke` on the final source. Resolve valid remaining security review findings; then squash-merge this existing PR with expected HEAD SHA. Do not grant authority, accept K06 or suppress negative cases without supporting evidence.

## PR #154 — K06 and experimental M3/M4 SMP integration checkpoint — 9 November 2026

**Source:** existing PR #154 at `0ac938095908ecaf8e42218823bea97367160a65`, integrated against main `2ed4c087746021da3c8b96511af3209a35d0f441`. The conflicting ACPI MADT/K06 and SMP Info/M3-M4 paths are retained as **separate opt-in build profiles** instead of an unconditional ours/theirs replacement. K06 stays the default `native-smp` profile (up to eight CPUs); the historical SMP Info ABI and its native M3/M4 probes are isolated under `native-smp-legacy` and the dependent `native-smp-m3`/`native-smp-m4` features (up to four CPUs). Stable boot ABI/graphics and legacy SMP wire layouts are feature-gated separately; the legacy test runner has a distinct module name.

**Verification boundary:** source-level integration only. The branch implementation is not yet verified by Cargo, `tools/progress.py --check`, strict preflight, the native build, QEMU smoke, full SMP matrix or the current-head GitHub Actions checks. Historical PR evidence and the older K06 tests do **not** establish merged-runtime acceptance. No criterion is newly accepted: **39/100 → 39/100**. Do not squash-merge before required checks pass and review findings are resolved.

**NEXT AGENT HANDOFF:** run the standard host, formatting, preflight, build and QEMU smoke gates on the actual integrated PR SHA, then `python3 -B -u tools/test_smp.py` and all current K06 `tools/run_smp.py` cases. Repair any compiler, ABI, oracle, or negative-case failures; check CodeRabbit review and GitHub `host-quality`/`qemu-smoke` on the latest head before squash-merge using an expected SHA. Preserve the separate B01 native positive recovery test as open.



## PR #214 — Horizon desktop integration — 9 November 2026

**Source:** PR #214 (`ui/desktop-2030`), integrating main `c663f0b6386203e17e28d52c2c79da161205ee26`
while retaining PR #155's keyring authorisation fix and 39/100 assessment.
The Horizon implementation remains host-rendered; no native UI acceptance is
claimed. Existing CI workflows and capability boundaries are unchanged.

**Fixes:** the SDK protocol-chain semantic heading now expects the visible
`Nebula` label; the graphics wallpaper matches rustfmt; the Orbit dock avoids
integer underflow on tiny widths and has a focused regression test. Previous
positive fade guards and focus-ring behavior are preserved. Desktop-shell
documentation distinguishes the planned running-app indicator from the
implemented view, where the dock is not rendered while Nebula is open.

**Evidence:** on prior head `d54b174005338d82fa5222a7eee770a373fe5c80`, GitHub
`host-quality` failed on the old heading assertion; `native-display-input`
failed on rustfmt, and `native-window-session` failed in the same
protocol-chain test. Prior `qemu-smoke` passed, but is not current-head
evidence. No local Rust, progress, preflight, build or QEMU commands ran in
this GitHub-connector environment. The integrated PR HEAD requires fresh CI.
Progress stays **39/100**; no criterion is newly accepted. Native UX and
user-task observations remain open.

**NEXT AGENT HANDOFF:** verify `python3 tools/progress.py --check`,
`python3 tools/preflight.py --strict`, `python3 tools/build.py`, and
`python3 tools/run.py --smoke` on the integrated head; review CodeRabbit
and the required GitHub `host-quality`/`qemu-smoke` statuses before
squash-merge. The separate B01 positive native recovery test remains open
under PR #155's checkpoint.

## PR #155 keyring recovery authorization — 9 November 2026

**Source:** PR #155 security code at `ef391def0a4bcd825f4664ab21de844ccd4f7ee4` was integrated with main `9554d677c0ab8dd9acfce47f9abe43e22c0215c8` on the existing PR branch. The current main CI workflows and concise documentation hierarchy were preserved. Ordinary applications can no longer export or restore keyring recovery material; only a separately authenticated management ingress can dispatch these operations. No new native management capability was granted.

**Acceptance:** **41/100 → 39/100.** B01 is open because the earlier positive native recovery test used ordinary app authority. The historical five-case native QEMU gate and positive management host tests do not establish native positive management recovery. Checks on the combined HEAD have **not yet been executed or verified**. No other accepted criterion changed.

**NEXT AGENT HANDOFF:** run `python3 tools/progress.py --check`, `python3 tools/preflight.py --strict`, `python3 tools/build.py`, `python3 tools/run.py --smoke`, and `python3 -B -u tools/test_keyring.py` on the combined PR HEAD. Review CodeRabbit and required CI results before squash merge. Physical-hardware and human-security audits remain outstanding.

## PR #181 — PCI device lifecycle policy checkpoint — 9 November 2026

The new `kernel-core` device execution lease registry models PCI function/generation binding, bounded config and BAR access, directional DMA grants, interrupt teardown, quarantine and release. It is a host-testable policy model only; no production kernel driver adapter is connected. Bus-master enablement is test-only until a kernel-issued, enforced DMA/IOMMU-containment proof exists. Live DMA frames are not pinned against the generic allocator's free path. Do not treat this as native PCI, DMA/IOMMU or physical-hardware acceptance.

The CI-blocking rustfmt regression was corrected on PR [#181](https://github.com/simonvanieperen/BaryonOS/pull/181) in `edb5ac77e3dac7e511c664e04e52d009175b6e23`; current `main` was integrated into the PR at `157182c068c62533bf06e4e8a1c994e7e72d2842` without replacing its two source-file changes. The current PR-head CI still must be evaluated; historical tests from earlier source commits are not proof for this checkpoint. Progress: **41/100 → 41/100**; no new criterion is accepted. Follow-up: add enforced DMA pinning/containment, then a fail-closed native adapter and negative runtime tests before exposing drivers to this model.

## PR #284 refresh — 9 November 2026

Dependabot coverage is extended to the remaining independent Cargo workspaces. The separate SDK entry from #285 is retained; its directory is excluded from the grouped entry to prevent duplicates. Progress remains 41/100. Local preflight, build and QEMU smoke passed (exit 0 each); all 11 workflow/Dependabot contract tests passed in WSL with the Windows worktree Git directory explicitly selected.

## PR #285 refresh — 9 November 2026

SDK monthly Dependabot coverage is retained on current main. The current documentation hierarchy is preserved. Progress remains 41/100; no new product acceptance. Local `python tools/preflight.py --strict`, `python tools/build.py`, and `python tools/run.py --smoke` passed (exit 0 each; QEMU raw exit 33, M1:PASS). No physical-hardware acceptance is claimed.

## Previous checkpoint — documentation hierarchy and governance — 9 November 2026

**Source and scope:** base `main` `f704f078213170e72292a5a491f0ee83bd588955`, branch `docs/restructure-project-docs-20261109`. The active README, contribution guidance, security policy, licensing documentation, and status layers have been reorganized so that each information type has one primary source. Historical content was preserved under `docs/history/` before the active files were shortened.

**Progress:** **41/100 → 41/100**. This is documentation and governance work; no new product criterion is accepted and no native OS functionality is changed.

**Open repository administration:** GitHub Wiki pages, GitHub Milestone objects, and a real private vulnerability-reporting channel require repository-level actions that are not exposed by the current connector. They are tracked in issues [#277](https://github.com/simonvanieperen/BaryonOS/issues/277), [#276](https://github.com/simonvanieperen/BaryonOS/issues/276), and [#278](https://github.com/simonvanieperen/BaryonOS/issues/278), respectively. [ROADMAP.md](ROADMAP.md) remains the authoritative milestone source until the Milestone objects exist.

**Resume:** review this branch through CI and automated review. Confirm that README, `tools/progress.py`, `docs/progress.json`, `docs/PROGRESS.md`, and `docs/assets/progress.svg` remain consistent with `python3 tools/progress.py --check`.

## PR #185 — USB HID control setup encoding — 9 November 2026

**Source:** Existing PR #185, branch `copilot/driver-usb-hid-control` at `ad2d6d63c24d95174c083e2cbbc6ff7da263f09e`, integrated against `main` `75fbaa2dfdd9e14c0b2d64ad78e6e081bc283592`. The merge resolution retains the current main tree and the USB module/test changes. All 21 CI workflow files changed in the historical PR already match main; outdated README, project-status and progress snapshots are not reinstated. The PR's source-bound SMP validation receipt is retained as historical evidence, not as verification of this integration.

**Result:** `drivers/usb/src/hid_control.rs` encodes HID 1.11 class/interface `SET_PROTOCOL(boot)` and `SET_IDLE(report ID 0)` setup packets. Its exhaustive host tests cover interface numbers and idle durations. No USB transport, device identity, control completion or new authority is implemented; the caller must validate actual transfer completion.

**Verification boundary:** The previous branch reported passing targeted USB host tests and historical QEMU/SMP runs. This integration has **not** executed `cargo +1.90.0 test --locked --manifest-path drivers/usb/Cargo.toml`, strict Clippy, formatting, `python3 tools/preflight.py --strict`, `python3 tools/build.py` or `python3 tools/run.py --smoke` in the current session; the connected execution environment lacks Rust/QEMU and the project's RTK/Ponytail tooling. Hosted CI and review on the new PR head require separate confirmation. The earlier SMP receipt under `docs/verification/` is bound to its recorded source and does not demonstrate HID hardware support.

**Progress:** **41/100 → 41/100**; no fixed acceptance criterion is newly satisfied.

**Next:** confirm the current PR-head CI/review gate and squash-merge this existing PR; implement and test the HID control-transfer transport and completion boundary separately.


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

The fixed progress score is **39/100**. Detailed accepted and open criteria are maintained only in [PROGRESS.md](PROGRESS.md) and [progress.json](progress.json). Recent source-bound implementation and CI evidence remains available under [verification/](verification/).

The former checkpoint sequence is preserved in [PROJECT-STATE-through-2026-11-09.md](history/PROJECT-STATE-through-2026-11-09.md). That snapshot is historical context, not a second current-status source.
