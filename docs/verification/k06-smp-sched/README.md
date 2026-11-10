# K06 step 5f — SMP scheduling acceptance evidence

**Status: acceptance open.** Each of the 64 cases must pass exactly 50/50 on one frozen source, including all 50 complete ten-minute combined soaks. The aggregate and the subsequent repository gate must also pass. A failed or missing repetition rejects acceptance.

**Current complete restart:** [42741185181](https://github.com/simonvanieperen/BaryonOS/actions/runs/42741185181), frozen commit `332275d97ed5f24f6d43174969a5667eb42f2afa`, source-v3 `0f5144e3a17b1b5d0a0e91b24fb8dcf4db8cf5e82965dc96d4f0d2db4d591a18`, is pending behind the rejected run. No previous repetition is reused.

**Latest rejected restart:** [42731994218](https://github.com/simonvanieperen/BaryonOS/actions/runs/42731994218), tested source `59a30f10ceb558d1e31408afece49ca5c331df20`, source-v3 `505c25139b4d23d78aa9da406108aad437ba7a763b1b1c67b0cc6074a4d8f2ae`, remains running but has a red case and cannot be accepted. The next candidate is described below; no repetition is reused.

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
