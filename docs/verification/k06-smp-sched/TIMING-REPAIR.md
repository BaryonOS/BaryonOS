# K06 step 5f — timing repair: instrumentation, measurements and review, 12 November 2026

Diagnostic only. Nothing in this note is native acceptance, and no acceptance matrix has
run on the source described here. The decision record is ADR-082 in
[DECISIONS.md](../../DECISIONS.md). Sections 1–6 describe the state of 12 November, when its
contract parts were proposals; the project owner decided them on 13 November 2026, and
section 7 records their implementation and verification. The preceding investigation is
[TIMING-INVESTIGATION.md](TIMING-INVESTIGATION.md).

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
| C7: margin bookkeeping runs inside contended critical sections | Fixed (section 7): booked after the owner is published, in one slot per lock level, without a table scan |
| C8: margin keys are source line numbers | Not a blocker (section 7): the waits that carry a verdict have stable names (`sync_wait`, `scratch_wait`); the other records carry no verdict and are compared within one source hash, which the evidence binds |
| C9: 36 of 40 `within_ms` limits remain unnamed literals; the aggregate does not summarise margins | Fixed (section 7): every deadline is a named constant with its reason, values unchanged; the aggregate keeps the largest margin per wait and case |
| C10: margin and edge lines are accepted at any position | Unchanged; they carry no verdict |
| C11: the gate rehearsal job has no refuse step | Fixed (section 7) |
| C12/F15: stale comments; ADR-082 said the source was unchanged | Fixed |
| F01: a fatal `EXTRA` would fail healthy boots | Confirmed in data (section 3.3). ADR-082 D3 does not make `EXTRA` fatal |
| F02: the receptive witness saw only two spans | Closed (section 7): entry stamp before the LAPIC read, halt stamp right before `sti`, every AP wake and every AP outside its `hlt` voids the host credit; fixture `clock-other-cpu` |
| F03/F12/F13: size and shape of the host-loss bound | Open; part of decision D3 |
| F07: a per-case median rule rejects healthy boots | The rule is not proposed |
| F08: scratch has no per-CPU progress counter; the lock-pair wait has the same shape | Fixed for scratch with D2 (section 7). The lock pair keeps its 2 s bound: hosted maximum 4 ms |
| F11: fixture coverage | Closed (section 7): `scratch-stall`, `scratch-slow`, `clock-other-cpu`; `sync-slow` made independent of poll stretch; oracle and host tests for D1–D3 |
| F14: a deadlock inside the BSP's own sync share ends in the 60 s watchdog without a marker | Not a blocker (section 7): such a deadlock still fails the boot through the host watchdog; only the marker is missing |
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

## 7. Implementation of the decision of 13 November 2026

Diagnostic evidence; no acceptance matrix has run. Decision text: ADR-082, "Decision of
13 November 2026 and its implementation". Source: `c47ab74`, `d1cf67e`, `10d1dc7` and
`4eb18b1` on PR #317; the candidate is `4eb18b1`.

### 7.1 Changes and reasons

| Part | Change | Reason |
|---|---|---|
| D1 sync | Single-writer progress word per CPU; an AP without own progress for 2 s stops the guest (`SMP:SYNC:STALLED:CPU=<n>`); cap 20 s. The oracle judges 2 000 ms off the five overcommitted cases (fixed list `OVERCOMMITTED`) | Liveness apart from host performance; per-CPU detection names the stalled CPU instead of waiting for the others to finish |
| D2 scratch | Progress word per CPU in every fill and KERNEL round; 500 ms without own progress stops the guest (`SMP:SCRATCH:STALLED:CPU=<n>`), cap 5 s, both quarantine; PASS line with `WAIT_MS`/`STALL_MS`; the oracle judges spec 01's 500 ms off the five cases | As D1; spec 01 stays measurable on every other case and on a suitable environment |
| D3 clock | Entry stamp before the LAPIC MMIO read; halt stamp right before `sti`; AP wakes and APs outside `hlt` void the host credit; the verdict is the median without host-charged periods within 1 % on both sides, with a majority bound of 10 % host credit per segment (`smp_lock::clock::judge`); the oracle requires the edge line before the verdict and rechecks the judged median | Close F02 (b)–(d); (e) needs no fit: a span below a quarter period cannot merge two edges |
| D5 | `m3-load` host watchdog 125 s | 2.5 × 48.3 s (ADR-081 rule); guest checks unchanged |
| ICR (F19 fixture) | Each acknowledgement bounded by 2 s of TSC instead of 65 536 PAUSEs | The loop count failed a healthy local `eight` boot (`SMP:ICR:LOST`); a lost or misrouted 0xe8 still never arrives |
| C7 | Lock waits booked after the owner is published, one slot per level | No table scan inside contended critical sections; IQ-3 transit window unchanged |
| C9 / AC-3 | Named deadline constants `ACK_MS`, `STEP_MS`, `WORK_MS`, `READY_MS`, `FATAL_MS`, `PAIR_MS` (values unchanged); the aggregate keeps the largest margin per wait and case | Spec 09-5f AC-3 |
| C11 | Refuse step in the gate rehearsal | Mode separation |
| Oracle | A guest line containing the placeholder character `*` is rejected | A literal placeholder could otherwise match an expected line |
| Fixtures | `scratch-stall`, `scratch-slow`, `clock-other-cpu`; `sync-slow` crawls 5 ms per round (100 s share); matrix 74 × 50 | F11; each relaxation has a fault that it must still reject |

Two read-only AI reviews of `c47ab74` (kernel correctness; oracle and contract) found five
items, all fixed in `10d1dc7` and `4eb18b1`: K1 (a 10 % bound on the median segment alone
could fail a healthy boot whose single host-stalled segment lands on the median; now a
majority rule), K2 (exact `DONE` count of `sync-stall`), F1 (`scratch-slow` could finish
before the cap under slow polls), F2 (the edge line was optional, so the oracle recheck could
be skipped), F3 (the watchdog test did not run `run()`). A review by an AI is not an
independent human audit.

### 7.2 Local verification (QEMU 8.2.2, `pc-q35-8.2`)

69 boots, one row each in
[`repair/adr082-local-boots.tsv`](timing-investigation/repair/adr082-local-boots.tsv); raw
rows and scripts in `repair/adr082-local-runs.tar.gz`.

| Series | Source | Host CPUs | Boots | Passed |
|---|---|---|---|---|
| r1 | `c47ab74` | 4 | 22 | 21 (`eight`: `SMP:ICR:LOST`, the PAUSE-count ack; repaired in `d1cf67e`) |
| r2 | `d1cf67e` | 4 | 9 (`icr-race-control`, `eight` × 5, `nine-capacity` × 3) | 9 |
| r2 | `d1cf67e` | 3 (`taskset`) | 8 (`eight` × 3, `nine-capacity` × 3, `four` × 2) | 8 |
| r3 | `10d1dc7` | 4 | 4 (`sync-stall`, `four`, `clock-masked`, `clock-other-cpu`) | 4 |
| final | `4eb18b1` | 4 | 26: `one`, `two-single`, `four`, `eight`, `nine-capacity`, `m3-two`, `m3-eight`, `m3-load-eight`, `m3-recovery-four`, every sync, clock and scratch fixture, `lock-stuck`, `shared-scratch`, `lockpair-order`, `icr-race-control`, `ap-ipi-deaf`, `ap-shootdown-deaf`, `resched-deaf`, `quarantine-sched-busy`, `quarantine-lock-held` | 26 |

- Every negative case ended on its own marker; every regular case passed the oracle of its
  source. `m3-load-eight` took 49.8–50.2 s (watchdog 125 s).
- Margins (ms, maximum): on the five overcommitted cases, sync 510 (4 CPUs) and 1 540
  (3 CPUs), scratch 266; the longest run without own progress of any CPU was 20 (sync,
  window 2 000) and 14 (scratch, window 500). On every other case sync stayed at or below
  100 and scratch at or below 14.
- Clock, 47 regular boots: raw median −0.16 … +2.01 % (one boot above 1 %, which the old
  rule would have failed: 96 periods lost, 94 charged to the host), judged median
  −0.19 … −0.01 %; at most 10 kernel-charged periods per boot. `clock-masked` and
  `clock-other-cpu` (six boots) lost 313–342 periods, of which at most 26 were credited to
  the host, and stopped with a judged median of +6.1 %.
- Host and oracle tests on `4eb18b1`: `tools/tests` 317 OK; `tests/test_ci_*` OK;
  `cargo fmt --check`; `cargo clippy -p boot-protocol -p smp-lock -D warnings`;
  `cargo test --locked -p boot-protocol -p smp-lock` (70 tests, 358 s, 4 CPUs); every
  affected profile builds with the warnings of the base.
  `tools/preflight.py`: nothing missing.

### 7.3 Hosted diagnostic on the candidate

Run [43352859422](https://github.com/BaryonOS/BaryonOS/actions/runs/43352859422)
(`margin_diagnostic`, source `4eb18b1`, matrix runner `ubuntu-26.04`, 4 vCPUs, QEMU 10.2.1,
`pc-q35-10.0`), 20 boots per case. One row per boot in
[`repair/adr082-hosted-boots.tsv`](timing-investigation/repair/adr082-hosted-boots.tsv);
artifact digests in `repair/adr082-hosted-artifacts.txt`. Not acceptance evidence.

| Case | Passed | Sync wait / longest stall (ms) | Scratch wait / longest stall (ms) | Raw median | Judged median | Lost / host-charged, max per boot | Longest lock wait (ms, limit 400) | Host time, max (s) |
|---|---|---|---|---|---|---|---|---|
| `four` | 20/20 | 40 / 0 | 3 / 0 | −0.05 … −0.02 % | −0.05 … −0.02 % | 2 / 1 | 8 | 10.1 of 60 |
| `eight` | 20/20 | 220 / 0 | 201 / 8 | −0.08 … −0.02 % | −0.08 … −0.02 % | 1 / 1 | 21 | 15.0 of 60 |
| `nine-capacity` | 20/20 | 260 / 0 | 60 / 9 | −0.10 … −0.02 % | −0.10 … −0.02 % | 2 / 1 | 21 | 16.7 of 60 |
| `m3-eight` | 20/20 | 260 / 0 | 202 / 7 | −0.16 … +0.03 % | −0.16 … −0.03 % | 6 / 6 | 26 | 28.4 of 60 |
| `m3-migrate-eight` | 20/20 | 270 / 0 | 185 / 8 | −0.11 … −0.02 % | −0.11 … −0.02 % | 1 / 0 | 24 | 24.4 of 60 |
| `m3-load-two` | 20/20 | 0 / 0 | 1 / 0 | −0.03 … −0.01 % | −0.03 … −0.01 % | 1 / 0 | 3 | 39.7 of 125 |
| `m3-load-eight` | 20/20 | 280 / 0 | 120 / 5 | −0.09 … −0.03 % | −0.09 … −0.03 % | 2 / 2 | 23 | 47.3 of 125 |
| `m3-ipc-four` | 20/20 | 50 / 0 | 15 / 1 | −0.05 … −0.02 % | −0.05 … −0.02 % | 1 / 0 | 11 | 211.8 of 660 (ADR-081 documentary bound 264) |
| `icr-race-control` | 20/20 | 30 / 0 | 6 / 0 | −0.06 … −0.02 % | −0.06 … −0.02 % | 2 / 1 | — | 12.1 of 60 |
| `scratch-stall` | 20/20 | 50 / 0 | stops at the stall | −0.08 … +1.37 % | −0.08 … −0.02 % | 66 / 66 | — | 10.8 of 60 |
| `scratch-slow` | 20/20 | 70 / 0 | stops at the cap | −0.05 … −0.02 % | −0.05 … −0.02 % | 1 / 1 | — | 15.4 of 60 |
| `clock-other-cpu` | 20/20 | 50 / 0 | — | +6.13 … +6.18 % | +6.13 … +6.18 % | 316 / 0 | — | 8.0 of 60 |

- 240 of 240 boots ended as their oracle requires. Two `scratch-stall` boots lost 66 PIT
  periods, all while the BSP was demonstrably halted; their raw medians (up to +1.37 %) would
  have failed the old rule, the judged medians stay at −0.08 … −0.02 %. In `clock-other-cpu`
  none of the 316 lost periods per boot was credited to the host.
- Every other recorded wait used at most 3.3 % of its limit. Sync and scratch waits on the
  overcommitted cases stayed at or below 14 % (sync) and 41 % (scratch) of the limits that the
  oracle no longer judges there; no CPU went without own progress for more than 9 ms.
- `m3-ipc-four` took 198.7–211.8 s per boot (median 203.5 s), as in run 43187501026
  (198–209 s): 32 % of the 660 s watchdog and 80 % of the 264 s bound that ADR-081 R3-5
  addendum 2 sets for acceptance. That bound is documentary; no tool checks it.
- Gate rehearsal on the final-gate runner: success in 14 min; regression stages `host-tests`,
  `clippy`, `fmt`, `m2`, `k05`, `k05-smp`, `k07`, `rtc-build` and `rtc` all exit 0.
- Pull-request run [43353618663](https://github.com/BaryonOS/BaryonOS/actions/runs/43353618663)
  on `3dbc62c` (documentation only on top of `4eb18b1`): `native-smp` success, every profile
  and every fixture once on `ubuntu-24.04` with `pc-q35-8.2`.

### 7.4 GO/NO-GO for one full matrix

**GO**, to be started only with the project owner's explicit permission.

| Check | Result |
|---|---|
| Every known failure mechanism covered | Sync and scratch tails on overcommitted guests (D1, D2), host-lost PIT edges (D3), the `m3-load` watchdog margin (D5) and the ICR loop-count bound are removed by construction; the five matrix incidents of runs 43057843393, 43125206729 and 43187501026 belong to these mechanisms |
| Negative tests still detect real faults | Every fixture stops on its own marker: locally on `4eb18b1`, in PR run 43353618663, and 20/20 on the matrix runner for `scratch-stall`, `scratch-slow`, `clock-other-cpu` and `icr-race-control`; host tests cover the oracle rules and `judge` |
| Margins and runner capacity | Judged waits used at most 3.3 % of their limits on the matrix runner, lock waits at most 26 of 400 ms, the longest run without own progress 9 ms; host watchdogs: `m3-load-eight` 47.3 of 125 s, `m3-ipc-four` 211.8 of 660 s |
| Conformity with the approved contracts | D1–D5 as decided on 13 November 2026; the ICR repair touches no normative limit; no security or correctness check was relaxed (deadlock, lost update, lock violation and missing progress stay fatal, every wait is bounded) |

Pass probability for one matrix of 3 700 guests, stated with its assumptions: the five
known failures in about 9 850 earlier guests are removed by construction, and no other
timing class failed in those guests. With a Jeffreys estimate of the remaining per-guest
rate (0.5 / 9 850) the point estimate is about 83 %; the 95 % upper bound on that rate
(3 / 9 850) gives a lower bound of about 32 %. The new code itself has 240 hosted, 69 local
and one full PR pass behind it. 99 % is not demonstrable: it needs a per-guest rate of at
most 2.8 × 10⁻⁶, about one million clean boots.

Residual risks: an unknown rare class; the soak watchdog (612 of 660 s, 600 s of it the
fixed load); the documentary 264 s IPC bound (80 % used, while run 43125206729 needed only
121 s, so runner speed varies by a factor of about 1.7); DG-1's 400 ms lock waits on
overcommitted guests (26 ms hosted). On the five overcommitted cases the 2 s and 500 ms
limits stay testable through the recorded `WAIT_MS` values (per boot and in the aggregate's
margin summary) but are not judged; judging them needs an environment with at least as many
dedicated cores as vCPUs, or KVM or hardware (H06).

