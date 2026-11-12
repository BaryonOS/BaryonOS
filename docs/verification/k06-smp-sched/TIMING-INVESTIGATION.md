# K06 step 5f — timing incident investigation, 12 November 2026

Diagnostic only. Nothing in this note is native acceptance. No normative deadline,
tolerance or acceptance contract was changed. Measurements used uncommitted diagnostic
images; the patches are stored next to the raw data in
[`timing-investigation/`](timing-investigation/). The decision proposal that follows
from this note is ADR-082 in [DECISIONS.md](../../DECISIONS.md) (status: proposed).

## 1. Incidents examined

All on GitHub-hosted `ubuntu-26.04` runners (4 vCPUs = 2 cores with SMT, QEMU 10.2.1,
`pc-q35-10.0`, TCG). Each is one or two guests out of about 3 300 per matrix.

| Run | Case (guest vCPUs) | Iteration | Marker | Deadline | Observed |
|---|---|---|---|---|---|
| 43057843393 | soak shard 9 | 1 | `SMP_CLOCK_DRIFT` (idle, APs halted) | 1 % | median +1.05 % |
| 43125206729 | `eight` (8) | r050 | `SMP:SCRATCH:TIMEOUT` | 500 ms | not finished in 500 ms |
| 43187501026 | `nine-capacity` (9, 8 started) | r029 | `SMP:SYNC:TIMEOUT:DONE=0:TOTAL=157489` | 2 s | 98.4 % of 160 000 increments done |
| 43187501026 | `m3-load-two` (2) | r003, r006 | `SMP_CLOCK_DRIFT` (idle, AP halted) | 1 % | median +1.08 %, +1.27 % |

The fourth row arrived during this investigation. It matters because that guest has two
vCPUs on a four-vCPU runner: the guest does not oversubscribe the runner, and the AP is
halted during the measurement. Every failing segment still counted exactly 1 000 IRQ0s.

Run 43187501026 was not cancelled and no job was retried.

## 2. Method

- Host: this development container, 4 CPUs (Intel Xeon 2.1 GHz, itself a virtual
  machine; host steal 0.7–1.5 %), QEMU 8.2.2, `pc-q35-8.2`, OVMF, `-cpu qemu64`,
  256 MiB, the same device set as `tools/run_smp.py`. QEMU 8.2.2 and this host differ
  from the hosted runner, so absolute rates below are **not** hosted rates.
- `timing.py` boots one image N times under `taskset`, timestamps every serial marker
  on the host, and samples `/proc/<pid>/task/*/schedstat` for the QEMU threads.
- Image M (`diag-long.patch`): source `2543861` with the sync and scratch deadlines
  multiplied by ten and one extra line with the clock median, so that tails are
  observed rather than cut off. The 1 % clock limit is unchanged in this image.
- Image P (`diag-progress.patch`): as M, but the BSP does not take part in the sync
  stress; it polls every 1 ms and prints the shared counter every 50 polls. This is the
  configuration of the failed tail (APs contend, BSP polls PIT channel 2) for the whole
  phase.
- Guest capacity was varied with `-smp`; host capacity with `taskset -c 0-3` or `0-1`.
  Series ran one at a time.

Limits: the per-thread scheduler sample is missing for series M (the sampler matched a
thread name that QEMU sets only with `debug-threads=on`; corrected for series P).
Series M `eight` on two host CPUs was stopped after nine boots because every boot hit
the 120 s kill timer. Small samples (4–30 boots) show the shape of a distribution, not
a failure rate.

## 3. Results

### 3.1 Series M — phase durations (host wall clock, seconds)

`sync tail` is the time from the BSP's last own serial line to `SMP:SERIAL:PASS`; it
approximates what the 2 s deadline measures. `scratch` is `SMP:KILL:PASS` →
`SMP:SCRATCH:PASS` and includes the BSP's own share.

| Guest vCPUs on host CPUs | Boots | Sync phase median | Sync tail median / max | Scratch median / p90 / max | Scratch > 0.5 s | `SMP_CLOCK_DRIFT` |
|---|---|---|---|---|---|---|
| 4 on 4 | 30 | 1.15 | 0.001 / 0.17 | 0.042 / 0.047 / 0.050 | 0 of 27 | 3 |
| 8 on 4 | 30 | 7.49 | 0.011 / 0.039 | 0.071 / 0.486 / 0.558 | 2 of 26 | 4 |
| 9 on 4 (8 started) | 30 | 7.91 | 0.011 / 0.51 | 0.070 / 0.500 / 0.548 | 2 of 23 | 7 |
| 4 on 2 | 20 | 42.0 | 0.010 / 5.60 | 0.062 / 2.27 / 2.65 | 3 of 18 | 2 |
| 8 on 2 | 9 | not finished in 120 s (9 of 9, all inside the sync phase) | — | — | — | — |

- No boot reported a lost update, a lock violation or any failure other than
  `SMP_CLOCK_DRIFT`. Every boot that was not killed by the 120 s timer completed the
  sync phase with the exact expected total.
- With the committed deadlines, the 4-on-2 boot with a 5.6 s tail would have failed
  `SMP_SYNC_TIMEOUT`, and up to seven boots would have failed `SMP:SCRATCH:TIMEOUT` (the host-side interval is slightly wider than the guest's deadline window).
- The scratch distribution is bimodal once the guest has more runnable vCPUs than the
  host has CPUs: most boots finish in about 70 ms, a tail takes 0.2–0.56 s (4 host
  CPUs) or 1.3–2.7 s (2 host CPUs). With 4 vCPUs on 4 host CPUs there is no tail.
- Halving host capacity for the same four-vCPU guest lengthens the sync phase about
  37-fold (1.15 s → 42 s), not two-fold.

### 3.2 Series P — progress under contention

| Guest vCPUs on host CPUs | TCG threads | Boots | Sync phase | Counter rate median (min) per s | Longest 50 ms window without progress | vCPU thread run / runqueue wait |
|---|---|---|---|---|---|---|
| 4 on 4 | multi | 6 | 0.62–0.81 s | 85 000–116 000 (52 800) | none | 12 s / 0.5 s |
| 9 on 4 (7 APs contend) | multi | 10 | 6.1–8.0 s | 19 500–25 500 (3 250) | none | 40 s / 32 s |
| 4 on 2 | multi | 4 | 24.8–26.5 s | 2 200–2 280 (936) | none | 58 s / 55 s |
| 9 on 4 (7 APs contend) | single | 4 | 0.33–0.46 s | 406 000–496 000 (283 000) | none | not sampled |

- In all 24 boots the counter advanced in every 50 ms window until it reached its exact
  final value. The lowest rate in any window was 936 increments per second.
- The same guest work with seven contending APs runs about 20 times faster under
  single-threaded TCG (round-robin, one host thread) than under multi-threaded TCG on
  four host CPUs. The guest code is identical; only host scheduling of vCPU threads
  differs.
- Runqueue wait of the vCPU threads is about 4 % of run time at 4 on 4, and 80–95 % at
  9 on 4 and 4 on 2.
- After the counter is complete, the APs still print 200 serial lines each before they
  report `DONE`; at 9 on 4 this takes 0.38–0.51 s, about a quarter of the 2 s deadline.
- The hosted failure left 2 511 increments undone after more than 2 s, a rate below
  1 256 per second. That is inside the range measured locally at 4 on 2 (936–2 280 per
  second) and far below the local 9-on-4 median. The hosted record holds one counter
  sample, so continued progress after the deadline is inferred, not observed.

### 3.3 Clock precision (idle phase, APs halted)

- 20 of 134 local boots failed the 1 % limit (16 of 110 in series M, 4 of 24 in series
  P), at every guest size including 4 on 4 and single-threaded TCG.
- 109 of 110 medians in series M lie above the nominal 999 847 µs; the failing medians
  range from +1.0 % to +3.8 %. The TSC calibration of the failing boots varies by
  0.12 % (2 100 558–2 102 981 per ms), so calibration does not explain a deviation of
  1 % or more.
- The hosted `m3-load-two` failures show the same picture: 1 000 counted IRQ0s per
  segment, segments between −0.02 % and +3.4 %.
- A one-sided, positive deviation with an exact IRQ count is what the source-confirmed
  QEMU mechanism in [CLOCK-INVESTIGATION.md](CLOCK-INVESTIGATION.md) predicts: overdue
  PIT edges merge into one pending interrupt, so 1 000 delivered interrupts span more
  than 1 000 periods. No PIC trace was captured, so the mechanism remains consistent
  with the data rather than directly observed.

## 4. Assessment

| Hypothesis | Evidence for | Evidence against | Verdict |
|---|---|---|---|
| Kernel or synchronisation defect (deadlock, lost update, missed wake-up) | — | 134 completed local boots: exact totals, no violation, no window without progress; the hosted failure shows 98.4 % progress; `eight` passed 50/50 in the next run on unchanged code | Not demonstrated |
| Insufficient host capacity for the guest | 37-fold slow-down when host CPUs are halved; scratch tail only when runnable vCPUs exceed host CPUs; runqueue wait ≈ run time | Clock failures also at 2 vCPUs on a 4-vCPU runner and at 4 on 4 | Cause of the sync and scratch incidents; not the whole cause of the clock incidents |
| QEMU/TCG scheduling | Single-threaded TCG is 20 times faster on the same work; the kernel's FIFO ticket lock hands the lock to a specific waiter, whose vCPU thread may be descheduled by the host; lost PIT edges are a QEMU timer and interrupt-controller property | — | Mechanism for both classes |
| Deadline not robust | Each deadline compares guest progress with host wall-clock time (PIT channel 2 and TSC follow host time under TCG); at least 36 of the 40 `within_ms` deadlines and the sync deadline are unnamed literals and none logs its margin, although spec 09-5f AC-3 requires both | The deadlines hold with a wide margin when the guest is not oversubscribed (4 on 4: sync tail ≤ 0.17 s of 2 s, scratch ≤ 0.05 s of 0.5 s) | Demonstrated for oversubscribed profiles |

Common cause: the fixture measures guest progress against host wall-clock time while
QEMU's progress (vCPU threads, timer thread) depends on a host scheduler that neither
the guest nor the workflow controls. Two mechanisms follow from it:

1. **Lock handoff to a descheduled vCPU.** `crates/smp-lock` is a FIFO ticket lock. The
   next owner is fixed; if the host has descheduled that vCPU thread, every other
   contender spins until the host runs it again. Throughput then follows host
   scheduling latency, not guest work. This is the known behaviour of FIFO spinlocks
   under vCPU overcommit and is already recorded as risk R02 in spec 09-5f AC-2. It
   explains the sync and scratch incidents.
2. **Merged PIT edges.** Described in CLOCK-INVESTIGATION.md. It needs no guest
   oversubscription; any delay of the QEMU timer or vCPU thread on a shared host is
   enough. It explains the clock incidents.

Rough exposure, from very small counts: at least five incidental timing failures in
three full matrices (about 9 800 guests) is about one per 2 000 guests. A matrix of
3 300 guests then passes with a probability of roughly 20 %, which matches three
rejected matrices in a row. The estimate is an order of magnitude, not a measurement.

## 5. Not examined

- Hosted distributions: only pass/fail and total duration per guest exist for hosted
  runs. The proposed margin logging (ADR-082, part 1) would supply them.
- QEMU 10.2.1 behaviour of the same series, KVM, and physical hardware.
- A per-CPU progress trace for the scratch phase and a PIC/PIT trace for the clock.
- The remaining 39 wall-clock deadlines; none has failed in the three matrices.
