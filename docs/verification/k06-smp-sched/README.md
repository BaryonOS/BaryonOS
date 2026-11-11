# K06 step 5f — SMP scheduling acceptance evidence

**Status: acceptance open.** Each of the 64 cases must pass exactly 50/50 on one frozen source, including all 50 complete ten-minute combined soaks. The aggregate and the subsequent repository gate must also pass. A failed or missing repetition rejects acceptance.

[Copilot continuation prompt](COPILOT-HANDOFF.md) contains the exact frozen runs, existing branch, evidence commands and failure policy. Both read-only verifiers are tracked here, including [final-gate readback](verify_final_gate.py).

**Current frozen restart:** [42818153989](https://github.com/BaryonOS/BaryonOS/actions/runs/42818153989), commit `54a9d5c17291b862e30240e49ed4cde0d479315b`, source-v3 `727676734c47e4efabedb49831763d71c1cd08c5698704de4e1c12a26e57232b`, is pending behind the rejected run in the unchanged branch concurrency lock. No new guest has started at dispatch checkpoint and no prior repetition is reused. [Receipt](restart-42818153989.json).

**Rejected active restart:** [42809499884](https://github.com/BaryonOS/BaryonOS/actions/runs/42809499884), frozen commit `3dd6194f02231e8f1db61940242d8d90318daf1d`, source-v3 `05f680b768def10a2495e5958fa3af2fb95b50be19c755cce728f8c850ac887a`, remains running but cannot be accepted. [Exact failure checkpoint](rejected-42809499884-checkpoint.json): `proc-shootdown-deaf` is 48/50 and `bsp-spin-irq-off-control` is 49/50. [Independent partial readback](rejected-42809499884-checkpoint-verification.json.gz) verifies 1547/1550 attempts, including all 50 complete soaks; remaining cases and the gate are not accepted. [Three exact raw failures](rejected-42809499884-known-raw-failures.json.gz). No repetition is reused.

**Restored execution:** the same Cloud workspace is available. Fetch, push dry-run and workflow dispatch succeed after organization app installation 180498948. The repository moved to `BaryonOS/BaryonOS` without changing repository ID 1435549142. Previous attempt `42803221546` on this source was cancelled: eight ordinary jobs and the probe report success, 55 ordinary jobs and ten soak shards are cancelled, with no aggregate or gate. All nine original ZIPs are retained locally and match API SHA-256/size and ZIP CRC. API status alone is not independently verified 50/50 evidence. The complete new raw readback remains open; 41/100 is unchanged. Original GitHub artifacts are retained.

**Diagnostic boundary:** run `42746512322` on preceding source `3b6c9a68` completed with 50/50 traced successes, independently read back before the outage. Ordered PIC traces in two repetitions show multiple IRQ0 rises before an acknowledgement. The text traces have no timestamps, do not reproduce a clock failure and do not establish historical host-scheduling causality. They supply none of the required 3200 uninstrumented runs. The complete diagnostic receipt remains in the restored workspace and has not yet been committed.

**Latest rejected restart:** [42731994218](https://github.com/simonvanieperen/BaryonOS/actions/runs/42731994218), tested source `59a30f10ceb558d1e31408afece49ca5c331df20`, source-v3 `505c25139b4d23d78aa9da406108aad437ba7a763b1b1c67b0cc6074a4d8f2ae`, is complete with failure and cannot be accepted. Final counts are preserved below; no repetition is reused.

## Completed rejected run

Original run `42723127310` is completed with failure. [All case counts](rejected-42723127310-rejected-run-summary.json) and the [complete diagnostic readback](rejected-42723127310-final-verification.json.gz) record **2976/3200 passes and 224 failures**, with all 64 cases attempted exactly 50 times. Ten ordinary cases fail; 53 ordinary cases and the combined soak pass 50/50. The observed failures comprise 63 FPU-image mismatches, 159 watchdog timeouts and two clock-drift failures. These observations do not establish one cause for every failure.

| Rejected case | Passed |
|---|---:|
| `nine-capacity` | 49/50 |
| `shared-scratch` | 49/50 |
| `m3-four-single` | 49/50 |
| `m3-eight-single` | 15/50 |
| `m3-migrate-eight-single` | 17/50 |
| `m3-ipc-four-single` | 0/50 |
| `m3-ipc-four` | 0/50 |
| `m3-load-two-single` | 46/50 |
| `m3-load-two` | 49/50 |
| `m3-load-eight-single` | 2/50 |

K07 M4 quotas and qsim pressure each pass 50/50, and ten soak shards each pass five full ten-minute repetitions. The aggregate fails without an aggregate report; the final repository gate is skipped and has no gate log. No result from this source is reused by the current full restart.

[Original ZIP inventory](rejected-42723127310-zip-digest-inventory.json) records all 74 original artifacts and their verified API SHA-256, size and CRC. Every canonical extracted byte matches its ZIP. The [preservation receipt](rejected-42723127310-preservation.json) identifies a complete local payload/metadata archive. All original ZIPs remain locally and on GitHub. Compact results, the complete diagnostic verifier result and [additional representative raw failures](rejected-42723127310-additional-raw-failures.json.gz) are committed alongside the earlier raw incident evidence.

## Initial incidents on rejected run 42723127310

[GitHub Actions run](https://github.com/simonvanieperen/BaryonOS/actions/runs/42723127310) tested `f808d59150466aee3de24452ea408250b8f4dca6`, source-v3 `759320893ea058b90d0bc764cd2b9534734827131a212ad5ba27f5501db6e8ac`. The `nine-capacity` case completed 49/50: iteration 16 reported `SMP:CLOCK:DRIFT:US=1010334`, `M1:FAIL:SMP_CLOCK_DRIFT`, raw exit 35, without a watchdog timeout. It is rejected even if every other job passes. The `shared-scratch` case also completed 49/50: iteration 10 reported `SMP:CLOCK:DRIFT:US=1010549`, raw exit 35 without a timeout. The original run and artifacts are retained.

The [incident receipt](incident-42723127310.json) records the artifact ID, original ZIP digest and raw-file digests. The compressed raw serial, stderr and guest result preserve the failing repetition. The full original ZIP is retained locally under `build/k06-handoff/run-42723127310/nine-capacity.zip` and on GitHub.

## Corrected source and full restart

Corrected source `ed5413691b30df302c4b304a4d2eab9945f15919` has source-v3 `d5ba44f6f596ecbe099e0143489c9c2fdfb1382dda163d3cc7ad757787cf19fd`. [Full restart 42727641812](https://github.com/simonvanieperen/BaryonOS/actions/runs/42727641812) was dispatched from zero with `full_matrix=true`; it was subsequently superseded while pending. No earlier successful repetition is reused. Documentation commits after this source commit do not change its source fingerprint or retarget the dispatched run.

Clock observations now retain the actual IRQ0 tick count alongside their TSC timestamp. A sample that returns after its requested edge is compared using its actual tick delta, rather than assuming exactly 1000 interrupts. Five one-second target segments, the median, the existing observation deadline and the exact 1% limit remain unchanged. Failure diagnostics report calibration and every raw tick/TSC delta. This corrects a sampling defect; the old transcript cannot establish that this defect caused the failed repetition. Actual lost IRQ0 ticks still fail.

The final gate requires an executable `tools/qemu-smoke-test.sh`, which was absent on the rejected source. The restored wrapper runs existing M2, K05, single-CPU K07 and RTC gates sequentially after the full SMP matrix, together with host checks. It keeps existing guest timeouts and uses separate fresh evidence/build directories. Nested reports, raw transcripts and logs are retained by the final gate artifact.

No physical-hardware, x2APIC, IOAPIC device routing, power-management or independent human-audit claim is made. One-CPU cases remain regression evidence.

## Local checks before restart

The [host-check receipt](host-checks.json) records a successful `native-smp` crossbuild, `cargo check --lib`, 274/274 Python tool tests without skips and 4/4 gate-orchestration tests. Formatting, shell syntax, progress consistency and diff whitespace checks also pass. Full Cargo tests subsequently pass (525 tests). The existing SMP Clippy scope (`boot-protocol`, `smp-lock`, all targets, warnings denied) passes. An additional workspace-wide Clippy run reports an existing `manual_div_ceil` in unrelated `capability-core` code; its failed log is preserved, and that code is unchanged. No local QEMU guest was started. These checks do not replace the required native evidence.

The [independent raw-artifact recheck](partial-verification-42723127310.json) rejects the two archived cases: 98/100 guests pass, with both clock failures reproduced by the current byte-identical tested oracles. [Verifier](verify_artifacts.py) never starts a guest; diagnostic partial mode always returns failure. Full mode requires all 3200 unique guests and the complete aggregator.

## Additional FPU fixture repair

The original run also rejects `m3-four-single` at iteration 33 (49/50): `SMP:M3:FPU:IMAGE_MISMATCH`, raw exit 35, without timeout; its clock check passes. Raw evidence and the original ZIP digest are recorded in the incident receipt. The old log does not identify the mismatched register.

The probe previously published DATA+128 readiness before x87/SSE initialization, and its general loop-progress word increased before any canary check. The kernel stop condition could therefore accept a first, timer-interrupted initialization as completed progress. Readiness is now published only after full initialization and successful GPR/x87/SSE comparisons; every live canary must publish it before the fixture stops. The existing M4/qsim soak exclusions, final FPU-image comparisons and negative controls remain intact. This fixes a reachable premature-stop defect, without claiming that the old transcript proves its exact historical cause.

[Host evidence](fpu-host-checks.json): M3 and soak crossbuilds share the candidate fingerprint; 47 SMP host tests and eight real-ELF artifact/mutation tests on each probe pass. The workflow runs and archives that artifact check before each relevant M3 case and soak. The two unmap profiles use their distinct `smp_unmap.elf` fixture and are explicitly excluded from this probe-specific check. No native guest is executed locally.

[Previous complete restart 42730151923](https://github.com/simonvanieperen/BaryonOS/actions/runs/42730151923) tests `a0b27619abc79f0afcf588078d62c357d337fe93`, source-v3 `cc19201b30d3a9108de30cbc82af2a0d68816afd3fb43eed1cf7c394ae57887c`, from zero. The earlier pending restart on `ed541369` was superseded and cancelled before matrix execution. The original run was allowed to finish. No passing repetition from either older source is reused.

## Eight-CPU single-TCG failures and observer work

The original `m3-eight-single` report is 15/50 (26 FPU-image failures and nine 60-second watchdog timeouts). `m3-migrate-eight-single` is 17/50 (32 FPU-image failures and one watchdog timeout). Their complete original reports, representative raw failures, artifact IDs and ZIP hashes are retained. The original ten soak shards all completed successfully (50 complete soaks); the run remains rejected by its failing cases.

The fixture scanned all sixteen live processes on every M3 kernel entry. Each private-word observation takes an address-space lock and maps/unmaps a scratch page, including two INVLPG instructions. About 8000 entries can therefore add 128000 observer reads and 256000 invalidations under KERNEL, apart from genuine return-permission checks. The observer now checks the entering process before cleanup and performs a full live-process scan after stop and before cleanup. The observer frequency per process consequently follows that process's own entries. The actual ring-3 private-pattern check still executes on every user loop; all user-return permission walks, usercopy/shootdown/resource checks and locks remain intact. This removes redundant observer work without changing user workloads, 1000 switches per CPU, watchdogs or safety limits.

[Observer host evidence](observer-host-checks.json) contains matching M3/soak crossbuilds, 47 passing SMP host tests and eight passing real-ELF checks per probe. This is not a measured speedup or proof that the ten historical timeouts are resolved. The pending `a0b27619` restart was superseded and cancelled before matrix execution. The current complete restart is bound above to the final observer-corrected source.

## Rejected restart and per-AP progress observer

[Incident receipt](incident-42731994218.json): `resched-deaf` is 46/50, with clock drift at iterations 5, 38, 40 and 43. The original artifact (13729275038, ZIP SHA-256 `d84ba730f96a9f01c989d301db4f339495913e719781d4118d3829e820c019d7`) is retained and verified. [All four raw failures and the manifest](incident-42731994218-resched-deaf-raw.json.gz) show five exact 1000-tick segments each; the medians are 1010544, 1010633, 1012463 and 1010492 microseconds. All fail the unchanged 1% limit, raw exit 35, without a timeout. Observation overshoot is not the cause of these four failures.

Candidate source-v3 `0f5144e3a17b1b5d0a0e91b24fb8dcf4db8cf5e82965dc96d4f0d2db4d591a18` removes shared progress-observer contention: continuously busy APs now store their own counters, separated by 64 bytes. The BSP checks every online AP rather than any progress on one global word. The old built loop contains a shared LOCK RMW; the new [actual object-code excerpt](clock-per-ap-objectcode.txt) contains a per-AP store, PAUSE and phase poll. This changes the fixture's workload mix; it does not reduce CPU occupancy, remove synchronization tests or alter the PIT clock, sample count, exact 1% bound or deadlines. L0 step 06 section 4 requires SMP load without prescribing a shared progress RMW. No causal link from cache-line contention to the historical IRQ0 deficit is claimed; genuine IRQ loss may still fail.

[Host receipt](clock-per-ap-host-checks.json): crossbuild and 47 SMP host tests pass, as do fmt and diff checks. No local guest is started. The current complete restart is bound above to the committed candidate and must pass from zero before any native pass claim.

## Current IPC correction and diagnostic mode

The selector now permits an ordinary nonblocking syscall to continue its remaining quantum only after checking current per-CPU deferred timer and reschedule generations. It uses the same full return validator, READY dispatch CAS, stack ownership, FPU and CR3 sequence. Continuation preserves peer FIFO order and does not renew quantum acknowledgements or fairness selections. Explicit yield/exit/block, kill, migration, recovery and stop do not obtain this shortcut. Wake placement remains FIFO; 10000 IPC rounds within 60 seconds are not yet proved.

Run 42748931069 disproves the throughput candidate: both ordinary IPC cases time out in all 50 repetitions. Each 10000-round pair requires at least 40000 successful capability syscalls. The old path copied and observably wiped a 461816-byte exact authority snapshot before every successful capability-free operation even though successful results were never compared with it, then performed a complete structural Engine scan. The soak does not prove this quota: its duration profile requires only positive IPC progress.

The replacement first proves, under KERNEL and the process ASPACE lease, that a completely validated capability-free SEND or RECV has a ready queue and no remaining fallible step. Only that proven-success path omits the otherwise unused snapshot. Every operation that may fail, every capability transfer and every malformed/usercopy path retains the complete exact snapshot, comparison and wipe. Complete Engine audits remain at bootstrap, process exit/cleanup and final quiescent STOP; syscall-local validation, exact failure rollback checks, usercopy/output validation and observable erasure remain intact. The mode20 loop also receives the same PAUSE host-vCPU hint as ordinary canaries, without guest Yield, blocking, queue priority or fewer IPC calls. Full native throughput remains open until a new complete matrix passes.

The [known-failure checkpoint](rejected-42731994218-checkpoint.json) and [retained raw failures](rejected-42731994218-known-raw-failures.json.gz) distinguish clock drift, eight-CPU timeouts, the incomplete fifth soak on shard 7, and HTTP 503 setup failures with no guest attempt. [Original setup log](setup-failure-129569906578.log.gz) preserves the snapshot outage. Three download retries retain every signed URL, package/version/hash and existing execution bound. The snapshot returned HTTP 200 at a later local observation; this is not a completed native setup proof.

The [clock investigation](CLOCK-INVESTIGATION.md) preserves the explicit counted-IRQ0 contract and separates the source-confirmed coalescing mechanism from unproved historical causality. The new `clock_diagnostic` input on the existing workflow is manual-only, default false, mutually exclusive with all acceptance guest jobs and rejects simultaneous full-matrix input before setup/build/guests. It traces the unchanged one-CPU multi-thread profile 50 times with its original 60-second watchdog. Actual trace flags remain in each raw executed command. Trace/host-context files are diagnostic evidence only; they cannot supply any of the required 3200 uninstrumented matrix runs. [Workflow mode/guard checks](ipc-ci-hostchecks.log.gz) and [command-spy checks](clock-trace-wrapper-hostchecks.log.gz) execute no QEMU guests. Full runs retain the 63 ordinary cases, ten five-run soak shards, exact aggregate and subsequent repository gate.

## Final rejected 59a restart

[Final counts](rejected-42731994218-rejected-run-summary.json): 2500 executed matrix attempts, 2386 verified passes and 114 native failures; 700 required repetitions are missing after 14 setup failures. There are 106 native watchdogs and eight clock-drift failures. Only 43 cases pass exactly 50/50; seven cases are native-red and 14 have no native evidence. Both IPC cases time out in all 50 repetitions. All 50 soaks are attempted but only 49 complete successfully. Regular K07 M4/qsim cases are missing, although the separate environment probe passes. The aggregator fails and tools/gate.sh is skipped, without a gate artifact.

All [60 original ZIPs](rejected-42731994218-zip-digest-inventory.json) remain intact and independently match API SHA-256/size, ZIP CRC and canonical bytes. The [complete diagnostic readback](rejected-42731994218-final-verification.json.gz) explicitly rejects this run. These results do not count toward the current frozen-source full restart.

## Ordinary canary CPU hint

[Host receipt](pause-host-checks.json): the ordinary mode19 path now contains one PAUSE after all GPR/x87/SSE comparisons and the readiness store. The IPC and migration-Yield branches bypass it; the legacy mode0 loop remains intact. [Actual ELF disassembly](pause-probe-objectcode.txt) and independent source review confirm this layout; eight real-ELF tests per M3/soak profile, 48 SMP host tests and M3/soak/supervisor crossbuilds pass on the candidate fingerprint.

QEMU v10.2.1 [helper_pause](https://github.com/qemu/qemu/blob/v10.2.1/target/i386/tcg/misc_helper.c#L91) exits the emulator vCPU loop without guest HLT or Yield. [Round-robin TCG](https://github.com/qemu/qemu/blob/v10.2.1/accel/tcg/tcg-accel-ops-rr.c#L281) then rotates vCPUs; its normal [kick period is 100 ms](https://github.com/qemu/qemu/blob/v10.2.1/accel/tcg/tcg-accel-ops-rr.h#L16). This does not establish infinite starvation without PAUSE. The guest thread remains Running, and only a real kernel selection of another process increases cpu_switches. Thus the original 1000-switch check still proves process scheduling; no CPU count, workload process count, clock criterion, deadline or quota is reduced. Emulator interleaving changes, host CPU occupancy is unmeasured, and MTTCG may pay additional BQL overhead. A native speedup or explanation of the five historical watchdogs is not claimed. The complete matrix restarts from zero on the committed hint source.

## Rejected 22df restart and IRQ-entry clock boundary

[Run 42748931069](https://github.com/simonvanieperen/BaryonOS/actions/runs/42748931069) is rejected on source `22dfceaf1fe4b1c4a499440b4199b951d85025f9`, source-v3 `58f7cc8a20db1c0f3afe30241fb14c125e7c75b62b984c6412056a19923a3eae`. Sixty of 63 ordinary jobs and all ten soak shards pass. Both IPC cases are 0/50 with genuine 60-second watchdogs after `SMP:M3:GO:CPUS=4`; `m3-canary-control` is 49/50 because iteration 25 reports a five-segment median of 1011862 microseconds. The environment/K07 probe and both regular K07 jobs pass. The aggregator fails and the final repository gate is skipped. This run supplies no acceptance repetitions.

The failed clock sample counts exactly 1000 IRQ0 entries in every segment, but the old measurement reads TSC only after the IRQ has returned to the observer. The IRQ0 stub now captures TSC immediately before publishing each actually counted tick. The five one-second segments, actual tick deltas, median, exact 1% limit and deadline remain unchanged; no elapsed tick is synthesized. Failure output also records the observer lag at both ends of every segment. Lost or coalesced IRQ0 entries still lengthen the interval between 1000 actual entry timestamps. Host/crossbuild evidence is diagnostic only; the complete matrix must restart from zero.

## Lock-free deaf fixture and bounded clock observer work

On source `3dd6194f`, process-shootdown-deaf iterations 18 and 21 correctly stop with `SMP:QUARANTINE:LOCK_HELD:40:CPU=1`. The scheduler continued to enter RUNQ during the deliberately unacknowledged round. This violates the test's own precondition in [L0 step 07 section 4](https://github.com/BaryonOS/BaryonOS/blob/docs/k06-step-specs/docs/implementation/k06-steps/07-5e-process-shootdown.md): the deaf AP holds no lock. IQ-3's fail-closed stop is correct and remains unchanged.

The replacement uses a separate real kernel shootdown as a setup barrier. Each AP closes this test's scheduler admission before its valid setup ack and masks its process timer. A pending timer/reschedule interrupt resumes the same stack/root without taking RUNQ. The BSP requires all setup acks, lock-free APs, new reader progress after setup and the exact AP1 ACTIVE bit before changing A's PTE. The later A-round still performs no flush/ack and retains its 100-ms deadline. All frame-retention, held-owner checks before/after INIT and quarantine handling remain intact. The serial oracle additionally requires `SMP:PROC_DEAF:LOCK_FREE:CPUS=4` at the exact setup boundary and rejects missing or incorrect CPU evidence.

Clock-control iteration 45 still fails with five exact 1000-tick samples and median 1014857 microseconds. Later observer lags are approximately 2 microseconds; they do not explain the sustained drift. In QEMU 10.2.1, [PAUSE exits the TCG CPU loop](https://github.com/qemu/qemu/blob/v10.2.1/target/i386/tcg/misc_helper.c#L91), and [MTTCG reacquires BQL on return](https://github.com/qemu/qemu/blob/v10.2.1/accel/tcg/tcg-accel-ops-mttcg.c#L87). The clock observer previously did this after only an increment/store. Each continuously busy AP now performs 64 dependent integer steps before its separate progress increment/store and PAUSE. This explicitly changes the fixture instruction mix and exit frequency per work step; it proves neither a historical cause nor a measured speedup. All APs remain runnable and busy, with the same real IRQ0 counting, five consecutive segments, median, exact 1% bound and deadlines.

[Host checks](fixture-repair-host-checks.json) record four matching crossbuilds, 275 host tests, 48 focused oracle tests, fmt/diff checks and [actual ELF instructions](fixture-clock-batch-objectcode.txt). No local native guest is executed. Native recovery requires a new complete 64×50 matrix and executed gate from zero.

## Rejected full run 43057843393 (head `e398039e`)

[Incident receipt](incident-43057843393.json): 72 of 79 jobs pass, three fail and four are skipped. Two primary failures:

- `m3-soak-four`, shard 9, iteration 5: `SMP_CLOCK_DRIFT` in the idle precision phase with every AP halted. All five segments count exactly 1000 IRQ0; the median is 1 010 389 µs against 999 847 µs (1.05%; limit 1%, unchanged). The other 49 soaks and every other case pass the same check. The pattern matches the lost-edge mechanism in [CLOCK-INVESTIGATION.md](CLOCK-INVESTIGATION.md). No guest defect is demonstrated, and no code changes in response.
- `m3-ipc-four`: 0/50, every repetition stopped by the 180 s host watchdog after `SMP:M3:GO:CPUS=4`, without a FAIL marker. `m3-ipc-four-single` passes 50/50 at 71–84 s per boot on the same runner class.

Local reproduction of the IPC case (QEMU 8.2.2, multi-thread TCG, `taskset`): 116 s on 4 host CPUs, 130 s on 3 and 327 s on 2; single-thread TCG takes 103–109 s at 2 or 4 CPUs. Diagnostic counters (uncommitted) show the unchanged oracle passing with zero wait timeouts. The IPC phase runs at about one round per scheduler tick on 4 CPUs and about 0.37 round per tick on 2; most of the time goes to endpoints that a tick preempts and requeues behind three CPU-bound canaries. A variant that gave the canary PAUSE hint on one pass in 64 was slower (432 s on 2 CPUs) and was discarded. The 180 s watchdog therefore sits below the completion time on the hosted 4-vCPU runner; changing it or the runner is an owner decision. The aggregate rejects the run and the final gate is skipped. The compact artifacts stay on GitHub for their 3-day retention only.
