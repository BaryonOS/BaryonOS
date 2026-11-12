# K06 step 5f — timing repair: instrumentation, measurements and review, 12 November 2026

Diagnostic only. Nothing in this note is native acceptance, and no acceptance matrix has
run on the source described here. The decision record is ADR-082 in
[DECISIONS.md](../../DECISIONS.md); its contract parts are proposals. The preceding
investigation is [TIMING-INVESTIGATION.md](TIMING-INVESTIGATION.md).

## 1. What the source contains

Commits `ed05150`, `cc1043f` and `4616fb3` on PR #317 add measurement and fail-closed
checks to the K06 fixture. Every acceptance limit is unchanged.

| Part | Guest (`kernel/src/m2/smp.rs`, `crates/smp-lock/src/clock.rs`) | Host oracle (`tools/run_smp.py`) |
|---|---|---|
| Margin records | Largest used part of each bounded wait, contended lock wait and shootdown round: `SMP:MARGIN:<WAIT\|LOCK\|ROUND>:<id>:<used>/<limit>` | Stored as `guest_waits`; no verdict; a full table is rejected |
| Sync wait | 10 ms polls as before; fatal on 2 s without progress (`SMP:SYNC:STALLED`) or after 20 s (`SMP:SYNC:TIMEOUT`); prints `SMP:SYNC:WAIT_MS=:STALL_MS=` | Line required at a fixed position; a wait of 2 000 ms or more is rejected on every profile; limit pinned in the oracle |
| Clock edges (shadow) | PIC IRR read and PIT channel 0 latch after every idle wake; periods booked as counted, pending, late or lost; lost periods charged to the host only with a complete witness; prints `SMP:CLOCK:EDGES` | Stored as `clock_edges`; the verdict stays the guest's raw 1 % rule |
| Negative fixtures | `sync-stall`, `sync-slow`, `sync-lost-update`, `clock-masked`, `clock-divisor` | Exact terminal sequence per case; matrix 71 × 50 |

## 2. Data

One row per boot: [`timing-investigation/repair/boots.tsv`](timing-investigation/repair/boots.tsv)
(columns in [`repair/README.md`](timing-investigation/repair/README.md)).

| Series | Source | Host | QEMU | Boots |
|---|---|---|---|---|
| local-A | `ed05150` (margins, progress wait; no clock sampling) | this development container, 4 CPUs | 8.2.2 | `four` 20, `eight` 30, `nine-capacity` 30 |
| local-B | `cc1043f` | same, CPUs 0–3 and CPUs 0–1 | 8.2.2 | 4 CPUs: 30 each; 2 CPUs: `four` 12, `eight` 3, `nine-capacity` 3 (host watchdog raised to 600 s for the 2-CPU series only) |
| hosted | `cc1043f`, run [43286858831](https://github.com/BaryonOS/BaryonOS/actions/runs/43286858831) (`margin_diagnostic`) | GitHub-hosted `ubuntu-26.04`, 4 vCPUs | 10.2.1 | `four`, `eight`, `nine-capacity`, `m3-load-two`, `m3-load-eight` × 50 |
| local-AB | `67e592c` against `4616fb3`, interleaved | this container, 4 CPUs | 8.2.2 | `four`, 40 per arm |

The development container is itself a virtual machine with other tenants. Its absolute
failure rates are not hosted rates. During the 2-CPU series, builds ran on the other two
CPUs; those boots were therefore measured under additional memory and cache load.

## 3. Results

### 3.1 Verdicts under the unchanged rules

| Series | Case (host CPUs) | Passed | Failures |
|---|---|---|---|
| hosted | `four`, `eight`, `nine-capacity`, `m3-load-two`, `m3-load-eight` (4) | 50/50 each, 250/250 | none |
| local-A | `four` (4) | 19/20 | 1 `SMP_CLOCK_DRIFT` |
| local-A | `eight` (4) | 30/30 | none |
| local-A | `nine-capacity` (4) | 26/30 | 3 `SMP_CLOCK_DRIFT`, 1 `SMP:TIMER:RATIO` out of band |
| local-B | `four` (4) | 14/30 | 15 `SMP_CLOCK_DRIFT`, 1 `SMP:TIMER:RATIO` out of band |
| local-B | `eight` (4) | 22/30 | 8 `SMP_CLOCK_DRIFT` |
| local-B | `nine-capacity` (4) | 19/30 | 11 `SMP_CLOCK_DRIFT` |
| local-B | `four` (2) | 10/12 | 1 `SMP_CLOCK_DRIFT`, 1 `SMP:TIMER:RATIO` out of band |
| local-B | `eight` (2) | 1/3 | 1 `SMP:SYNC:TIMEOUT` at the 20 s cap (93 278 of 160 000 increments done), 1 `SMP:TIMER:RATIO` out of band |
| local-B | `nine-capacity` (2) | 1/3 | 1 `SMP:LOCKPAIR:TIMEOUT`, 1 `SMP_ICR` |

No boot in any series reported a lost update, a lock violation, a stall or `SMP:LOCK:STUCK`.
Every local failure on 4 host CPUs is the unchanged 1 % clock rule or the unchanged
`SMP:TIMER:RATIO` band.

### 3.2 Sync, scratch and lock waits (ms; median / 90th percentile / maximum)

| Series, case (host CPUs) | Sync wait (limit 2 000) | Longest run without sync progress | Scratch (limit 500) | Lock pair (limit 2 000) | Longest lock wait (limit 400) |
|---|---|---|---|---|---|
| hosted `four` (4) | 0 / 0 / 2 | 0 | 0 / 1 / 5 | 0 / 1 / 1 | 6 / 7 / 8 |
| hosted `m3-load-two` (4) | 0 / 0 / 0 | 0 | 0 / 1 / 1 | 0 / 0 / 0 | 2 / 3 / 3 |
| hosted `eight` (4) | 2 / 6 / 11 | 0 | 31 / 149 / 218 | 1 / 1 / 2 | 18 / 22 / 27 |
| hosted `nine-capacity` (4) | 1 / 4 / 4 | 0 | 15 / 77 / 263 | 1 / 1 / 1 | 19 / 23 / 28 |
| hosted `m3-load-eight` (4) | 1 / 4 / 901 | 1 | 56 / 177 / 199 | 1 / 1 / 4 | 18 / 24 / 26 |
| local-A `eight` (4) | 1 / 16 / 349 | 7 | 11 / 139 / 244 | 1 / 4 / 386 | 35 / 91 / 117 |
| local-A `nine-capacity` (4) | 2 / 15 / 1 492 | 14 | 12 / 39 / 179 | 1 / 3 / 5 | 29 / 46 / 63 |
| local-B `eight` (4) | 1 / 9 / 18 | 5 | 20 / 140 / 408 | 1 / 2 / 422 | 30 / 71 / 100 |
| local-B `nine-capacity` (4) | 1 / 5 / 423 | 9 | 20 / 53 / 129 | 1 / 406 / 407 | 33 / 44 / 63 |
| local-B `four` (2) | 1 / 106 / 119 | 6 | 6 / 29 / 215 | 4 / 7 / 7 | 19 / 23 / 23 |
| local-B `eight`, `nine-capacity` (2), completed boots | 212–2 569 | 12 | 0–4 | 0–2 000 | 146–158 |

- The waits are heavy-tailed only where the guest has more vCPUs than the host has CPUs:
  the median is 1–3 % of the limit and single boots reach 43–82 % of it. With four vCPUs
  on four CPUs the maximum stays at or below 1 % of each limit on the hosted runner.
- The scratch limit is the closest: 263 ms of 500 ms in 150 hosted eight-vCPU boots and
  408 ms locally. The hosted incident of run 43125206729 belongs to this tail.
- The sync wait reached 901 ms once in 150 hosted eight-vCPU boots while the median was
  1 ms; the hosted incident of run 43187501026 belongs to this tail. In no boot did sync
  progress pause for longer than 14 ms, against the 2 s liveness window.
- Lock waits stay below 30 ms hosted and reach 117 ms locally on 4 CPUs (158 ms on 2).
- Every other recorded wait stays at or below 7 % of its limit on the hosted runner. The
  host watchdog is the exception: `m3-load-eight` needs 46.0–48.3 s of 60 s, of which
  30 s is the fixed load phase.

### 3.3 Clock

| Series (host CPUs) | Boots with accounting | Raw median segment | Boots beyond ±1 % | Median without host-charged periods | Lost periods per boot, maximum | Kernel-charged, maximum | Boots with `EXTRA` |
|---|---|---|---|---|---|---|---|
| hosted (4) | 250 | −0.14 … +0.55 % | 0 | −0.14 … −0.01 % | 29 of 5 001 | 1 | 3 |
| local-B (4) | 90 | +0.13 … +6.22 % | 34 | −0.22 … +0.16 % | 367 of 5 001 | 19 | 6 |
| local-B (2) | 17 | +0.17 … +1.16 % | 1 | −0.11 … −0.02 % | 99 of 5 001 | 16 | 1 |
| local-AB, final source (4) | 40 | +0.15 … +1.12 % | 2 | −0.14 … −0.02 % | 73 of 5 001 | 12 | 2 |

- Each boot counted exactly 5 001 IRQ0s. The raw deviation is one-sided and equals the
  lost periods; removing the host-charged periods leaves −0.22 … +0.16 % in all 397 boots.
  The remaining negative offset matches the TSC calibration, which takes the shortest of
  ten samples.
- Hosted, 59 of 250 boots lost at least one period and 29 had one period charged to the
  kernel; the largest loss was 29 periods (+0.55 %). The three hosted incidents lost more
  than 1 %.
- `EXTRA` (a counted IRQ0 beyond the elapsed periods) is one tick in 12 of 397 boots that
  show no other anomaly. The emulator presents an edge late after a sample that had
  already balanced its books. It reduces the host credit of its segment.
- `SMP:TIMER:RATIO`: 62 574–65 378 hosted (band 50 000–75 000); four local boots on the
  loaded development host left the band (77 263–78 784).

### 3.4 Cost of the instrumentation

Interleaved boots of case `four` on 4 host CPUs, base `67e592c` (arm A) against the final
source (arm B), 40 boots per arm in the order A B B A:

| Arm | Passed | Failures | Median boot time |
|---|---|---|---|
| A, `67e592c` | 36/40 | 4 `SMP_CLOCK_DRIFT` | 12.82 s |
| B, final source | 38/40 | 2 `SMP_CLOCK_DRIFT` | 12.81 s |

No cost of the margin records or the clock sampling is visible (Fisher exact p = 0.68).
Forty boots per arm exclude only large effects. The failure rate of the unchanged clock
rule on this host moved between 5 % (local-A), 38 % (local-B) and 5–10 % (local-AB) on
one day; that variation is host load. The base fails at the same rate as the
instrumented source when both are measured at the same time.

### 3.5 Negative fixtures and regular cases on the final source

One boot per case on `4616fb3` (QEMU 8.2.2, `pc-q35-8.2`, 4 host CPUs). One boot shows that a
path exists and ends as specified; it is not a 50/50 result.

| Case | Oracle | Sync wait / longest run without progress (ms) | Lost / host-charged periods | `EXTRA` | Raw median | Median without host-charged periods |
|---|---|---|---|---|---|---|
| `four` | pass | 0 / 0 | 34 / 33 | 0 | +0.48 % | −0.02 % |
| `eight` | pass | 20 / 0 | 25 / 24 | 1 | +0.43 % | −0.06 % |
| `nine-capacity` | pass | 530 / 0 | 53 / 53 | 0 | +0.84 % | −0.06 % |
| `one` | pass | 0 / 0 | 40 / 40 | 0 | +0.78 % | −0.02 % |
| `two-single` | pass | 10 / 0 | 50 / 49 | 0 | +0.48 % | −0.02 % |
| `m3-two` | pass | 0 / 0 | 39 / 36 | 0 | +0.37 % | −0.03 % |

| Negative case | Oracle | Terminal marker observed |
|---|---|---|
| `lock-stuck` (existing case) | expected sequence | `SMP:LOCK:STUCK:10:CPU=1`, after the new sync line |
| `sync-stall` | expected sequence | `SMP:SYNC:STALLED:DONE=2:TOTAL=80000` |
| `sync-slow` | expected sequence | `SMP:SYNC:TIMEOUT:DONE=2:TOTAL=70286` at the 20 s cap |
| `sync-lost-update` | expected sequence | `SMP:SYNC:LOST_UPDATE` |
| `clock-masked` | expected sequence | `SMP:CLOCK:DRIFT:KERNEL`; 328 periods lost, 21 charged to the host, raw median +6.13 %, without host-charged periods +5.91 % |
| `clock-divisor` | expected sequence | `SMP:CLOCK:PIT_COUNT` |

All six regular boots passed the unchanged 1 % clock rule; each lost 25–53 PIT periods,
of which 24–53 were charged to the host. In `clock-masked` the periods that the kernel masked stayed in the judged
value, and the 21 host-charged periods are of the size that the regular boots lost on
the same host.

## 4. Review findings and their disposition

Two read-only reviews by subagents examined `67e592c..cc1043f`; their reports are stored
as working notes in [`timing-investigation/map/`](timing-investigation/map/README.md)
(`K06C-CONTRACT.md`, `K06C-ATTACK.md`). A review by an AI is not an independent human audit.

| Finding | Disposition in `4616fb3` |
|---|---|
| C1/F06: the sync bound was raised to "stall or 20 s" and overruns were accepted on oversubscribed profiles without an ADR in force | Fixed. The oracle rejects a wait of 2 000 ms or more on every profile. The exemption is decision D1 |
| C2/F05: the 2 s figure was enforced through an optional line with a guest-supplied limit | Fixed. Required line at a fixed position; limit pinned in the oracle |
| C3/F05: the verdict depended on the CPU affinity of the evaluating process | Fixed. `evaluate()` has no host-dependent input |
| C4/F10: a recorded lock wait above 400 ms was rejected by the host only | Removed. Records carry no verdict (see ADR-082, "considered and not adopted") |
| C5/F09: a 1 s stall limit was stricter than the former rule | Fixed. The stall window equals the 2 s limit; such a wait failed before as well |
| C6/F04: unmeasured observer effect of the clock sampling | Measured, section 3.4 |
| C7: margin bookkeeping runs inside contended critical sections | Open. Covered only indirectly by section 3.4 |
| C8: margin keys are source line numbers | Open (records only). The sync wait has a stable name |
| C9: 36 of 40 `within_ms` limits remain unnamed literals; the aggregate does not summarise margins | Open AC-3 work |
| C10: margin and edge lines are accepted at any position | Unchanged; they carry no verdict |
| C11: the gate rehearsal job has no refuse step | Open; every acceptance job is skipped in that mode |
| C12/F15: stale comments; ADR-082 said the source was unchanged | Fixed |
| F01: a fatal `EXTRA` would fail healthy boots | Confirmed in data (section 3.3). ADR-082 D3 does not make `EXTRA` fatal |
| F02: the receptive witness saw only two spans | Partly closed: a sample is receptive only if exactly one IRQ0 was counted and no other interrupt entered the BSP. Remaining gaps are listed in ADR-082 D3 |
| F03/F12/F13: size and shape of the host-loss bound | Open; part of decision D3 |
| F07: a per-case median rule rejects healthy boots | The rule is not proposed |
| F08: scratch has no per-CPU progress counter; the lock-pair wait has the same shape | Open; part of decision D2 |
| F11: fixture coverage | Partly open; required cases are listed in ADR-082 |
| F14: a deadlock inside the BSP's own sync share ends in the 60 s watchdog without a marker | Open (pre-existing) |
| F16: a 99 % pass probability cannot be demonstrated by sampling | Agreed; ADR-082, "Statistics" |

## 5. Verification executed on `4616fb3`

Local, development container, 4 host CPUs, QEMU 8.2.2, `NOS_QEMU_MACHINE=pc-q35-8.2`; run
on the working tree that was committed unchanged as `4616fb3`. Every command exited 0. The step
summary is stored as [`repair/final-verification.txt`](timing-investigation/repair/final-verification.txt).

| Command | Result | Time |
|---|---|---|
| `cargo fmt --all -- --check` | exit 0 | 2 s |
| `cargo clippy --locked -p boot-protocol -p smp-lock --all-targets -- -D warnings` | exit 0 | cached |
| `cargo test --locked -p boot-protocol -p smp-lock` | exit 0 | 432 s |
| `python3 -m compileall -q tools tests userspace/tests experiments/quantum/native experiments/quantum/native/job .agents/skills/os-dev` | exit 0 | 1 s |
| `python3 -B -m unittest discover -s tests -p 'test_ci_evidence*.py'` | 31 tests, OK | 14 s |
| `python3 -B -m unittest discover -s tests -p 'test_ci_workflows.py'` | 23 tests, OK | <1 s |
| `python3 -m unittest discover -s tests -p 'test_document_session_*.py'` | 35 tests, OK | 1 s |
| `python3 -m unittest discover -s tests -p 'test_volume_*.py'` | 37 tests, OK | 2 s |
| `python3 -m unittest discover -s tests -p 'test_system_*.py'` | 53 tests, OK | 4 s |
| `python3 -m unittest discover -s tools/tests -p 'test_*.py'` | 313 tests, OK | 33 s |
| `python3 .agents/skills/os-dev/scripts/validate_skill.py` | exit 0 | 1 s |
| `python3 -m unittest discover -s .agents/skills/os-dev/tests -p 'test_*.py'` | 6 tests, OK | 1 s |
| `python3 -B -m unittest discover -s tests -p <pattern>` for `test_common.py`, `test_preflight.py`, `test_build_paths.py`, `test_guest_harness.py`, `test_m1_artifact_binding.py`, `test_m6c_*.py` | 10, 24, 3, 7, 8 and 49 tests, OK | 2 s |
| `python3 -B tools/build.py --kernel-feature <feature>` for `native-smp`, `native-smp-m3` and the six fixture features | exit 0 each | 2–4 s each |
| `python3 -B tools/run_smp.py --case <case>` for the twelve cases of section 3.5 | exit 0 each | 5–25 s each |

The clippy step reported an up-to-date build: the same command had run on the same source
in the preparation worktree shortly before. In the preparation run, `cargo test` pinned to
two CPUs did not finish `mutual_exclusion_under_contention` (eight threads on a FIFO
ticket lock) within ten CPU-minutes and was stopped; on four CPUs it passes, as above
and in the hosted gate rehearsal (259 s for the host-test stage).

## 6. Not verified

- No acceptance matrix, no soak and no final gate ran on this source.
- The hosted series ran on `cc1043f`, not on `4616fb3`. There the sync wait polled every 1 ms and
  the host witness lacked the other-interrupt condition, so hosted host-charged counts are
  upper bounds for the final source. The final witness was measured locally only (46 boots:
  −0.14 … −0.02 %).
- Hosted margins exist for five of the 71 cases only.
- The five negative fixtures were booted locally (QEMU 8.2.2) and not on the hosted runner; none has run 50/50.
- KVM and physical hardware.
- The remaining gaps of the receptive witness (ADR-082 D3) have no negative case.
