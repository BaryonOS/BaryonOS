# K06 step 5f — SMP scheduling acceptance evidence

**Status: acceptance open.** Each of the 64 cases must pass exactly 50/50 on one frozen source, including all 50 complete ten-minute combined soaks. The aggregate and the subsequent repository gate must also pass. A failed or missing repetition rejects acceptance.

## Rejected run 42723127310

[GitHub Actions run](https://github.com/simonvanieperen/BaryonOS/actions/runs/42723127310) tested `f808d59150466aee3de24452ea408250b8f4dca6`, source-v3 `759320893ea058b90d0bc764cd2b9534734827131a212ad5ba27f5501db6e8ac`. The `nine-capacity` case completed 49/50: iteration 16 reported `SMP:CLOCK:DRIFT:US=1010334`, `M1:FAIL:SMP_CLOCK_DRIFT`, raw exit 35, without a watchdog timeout. It is rejected even if every other job passes. The `shared-scratch` case also completed 49/50: iteration 10 reported `SMP:CLOCK:DRIFT:US=1010549`, raw exit 35 without a timeout. The original run and artifacts are retained.

The [incident receipt](incident-42723127310.json) records the artifact ID, original ZIP digest and raw-file digests. The compressed raw serial, stderr and guest result preserve the failing repetition. The full original ZIP is retained locally under `build/k06-handoff/run-42723127310/nine-capacity.zip` and on GitHub.

## Corrected source and full restart

Corrected source `ed5413691b30df302c4b304a4d2eab9945f15919` has source-v3 `d5ba44f6f596ecbe099e0143489c9c2fdfb1382dda163d3cc7ad757787cf19fd`. [Full restart 42727641812](https://github.com/simonvanieperen/BaryonOS/actions/runs/42727641812) was dispatched from zero with `full_matrix=true` and is pending behind the original run at this checkpoint. No earlier successful repetition is reused. Documentation commits after this source commit do not change its source fingerprint or retarget the dispatched run.

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

The pending restart on `ed541369` predates this FPU repair and must be replaced by a complete restart on the new source. The original run remains allowed to finish. No passing repetition from either older source is reused.
