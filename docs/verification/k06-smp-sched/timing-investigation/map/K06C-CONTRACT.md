# K06C-CONTRACT — independent contract review of `67e592c..cc1043f`

**Scope.** `git diff 67e592c cc1043f -- . ':!docs'` (commits `ed05150`, `cc1043f`; 11 files, +947/−49) in the
frozen worktree `snap-cc1043f`. The two verifiers under `docs/verification/k06-smp-sched/` were read because
the oracle chain depends on them.
**Method.** Static reading only (Read, Grep, `git diff`, `git show`). Nothing was built, booted, tested or
executed. A second AI review is not an independent human audit.
**Labels.** [S] verified in source, [I] inferred, [U] unverified. Line numbers refer to `cc1043f`; `base:`
means `67e592c`. `spec NN` means the L0 copy under `scratchpad/specs/docs/implementation/k06-steps/`.
**Correction of the task premise.** The pre-change sync wait was
`(0..200).any(|_| all_done || { wait_us(10_000); false })` (base: `kernel/src/m2/smp.rs:1352-1354`), not
`within_ms(2_000, …)` [S].

## 1. Verdict

No change was found that weakens an isolation, capability, memory-protection or lock-order invariant, and no
path was found by which a failing boot becomes a pass through a parsing error, a diagnostic workflow mode or a
negative fixture [S]. The normative limits named in the assignment are unchanged in their guest and host
verdict logic: scratch 500 ms, clock median within 1 %, `LOCK_STUCK_MS` 400 ms, shootdown 100 ms, the
`SMP:TIMER:RATIO` band and the 60/660/660 s host watchdogs [S]. The diff does change the acceptance contract
in one place and leaves the oracle weak around it. The sync wait's fatal bound moves from about 2 s to "no
progress for 1 000 polls, or 20 000 polls", while the only decision record (ADR-082) is marked PROPOSED and
spec 09 forbids raising a deadline without an ADR (C1). The 2 s figure survives only as an optional serial
line that the oracle never requires, whose limit it does not pin, and which about sixteen terminal cases never
print (C2). The overrun exemption depends on the CPU affinity of whichever process evaluates the transcript;
that value is not recorded and is recomputed on the verifier's machine (C3). These three findings are MAJOR:
they block the use of `cc1043f` for an acceptance matrix, not its use as a labelled diagnostic. The remaining
findings are MINOR or NOTE, mostly new stricter failure modes and unmeasured instrumentation overhead. No
BLOCKER was identified. Missing evidence is listed in section 4 as open, not as approval.

## 2. Findings

| ID | Sev. | Claim | Evidence | How verified | Suggested minimal fix |
|---|---|---|---|---|---|
| C1 | MAJOR | The sync deadline is raised with no ADR in force. Guest-fatal bound: was 200 polls of 10 ms; is `Stalled` after 1 000 consecutive 1 ms polls without progress or `Late` after `SYNC_MS * PROGRESS_CAP` = 20 000 polls. The host accepts `used > 2000` when `threads == "multi" and cpus > host_cpus` [S]. On a 4-CPU runner that exempts `eight`, `nine-capacity`, `m3-eight`, `m3-migrate-eight`, `m3-load-eight` [S]. A boot like the hosted failure `SMP:SYNC:TIMEOUT:DONE=0:TOTAL=157489` would now pass if progress continued [I]. | `kernel/src/m2/smp.rs:782,1371-1403,1524-1563`; base `smp.rs:1352-1354`; `tools/run_smp.py:501-503,545`; `run_smp.py:27-28,72,78,90` (case table); spec 09 `:52-53` ("De deadline wordt niet verhoogd zonder ADR"); `docs/DECISIONS.md:2680-2682` (ADR-082 "PROPOSED — not accepted … No deadline … has been changed; the committed source is unchanged") | Read both versions; read spec and ADR text | Do not run an acceptance matrix on this code until the project owner has accepted a rewritten ADR-082 that states `STALL_MS`, the cap and the exemption rule. Until then either keep `Late` at 1 x `limit`, or keep the code and label every run diagnostic. Correct the ADR sentence that the source is unchanged. |
| C2 | MAJOR | The 2 s performance figure is enforced only through an optional line. (a) `evaluate()` strips `SMP:MARGIN` lines and never requires one [S]. (b) `margins()` prints only after `smp::start` returns, so terminal cases that pass the sync phase and exit inside `start` print none; for them a 2–20 s sync wait is accepted on any host [S mechanism; I list: `double-run`, `shared-scratch`, `lock-in-handler`, `lockpair-order`, `bsp-spin-irq-off-control`, `icr-race-control`, `proc-no-shootdown-control`, `lock-stuck`, `held-across-switch`, `quarantine-lock-held`, `quarantine-lock-pending`, `ap-nmi-ist`, `gs-iret-fault`, `gs-nmi-user`, `clock-masked`, `clock-divisor`]. (c) The oracle pins no limit value: `…:5000/20000` passes, so a later edit of `SYNC_MS`, `STALL_MS` or the ROUND limit is invisible to the "independent" oracle [S]. (d) Waits after `margins()` (M3 phases, e.g. `smp.rs:5336`) are recorded but never printed [S]. | `run_smp.py:164,539-549,644-651`; `kernel/src/m2.rs:169-171`; `smp.rs:1475-1493` | Read; traced which expected sequences end inside `start` | Print the sync margin immediately after the wait under a stable name (not a line number); require exactly one in every case whose expected sequence passes `SMP:SYNC:PASS`; pin the limits in the oracle (sync 2000, STALL 1000, LOCK 400, ROUND 100), as is already done for `LIMIT_MS=400:LIMIT_TICKS=40` at `run_smp.py:630`. |
| C3 | MAJOR | The exemption is a property of the evaluating process, not of the recorded run. `host_cpus=None` defaults to `len(os.sched_getaffinity(0))`; `run()` neither passes nor stores it; `verify_artifacts.py` re-evaluates on its own machine and demands equality with the stored result [S]. Consequences: an overrun accepted on a 4-CPU runner is rejected when verified on a host with 8 or more CPUs; restricting affinity (`taskset`) makes any multi-thread case "oversubscribed" and widens acceptance; the call is Linux-only (uncaught `AttributeError` elsewhere, fail-closed) [I]. The value is host-derived only; no guest output feeds it [S]. | `run_smp.py:492-503,746-748`; `docs/verification/k06-smp-sched/verify_artifacts.py:246-247` | Read | Make the default fail-closed (`None` means no exemption). Let `run()` measure `host_cpus` once, store it in each item and pass it explicitly. Let the verifier pass the stored value and compare it with the runner size recorded in the environment evidence. |
| C4 | MINOR | New host-only rejection: a LOCK margin above 400 ms without a guest `SMP:LOCK:STUCK`. `TicketLock::lock_with` tests the grant after `relax()`, so a grant observed after the last deadline check is not fatal in the guest, but `lock_waited` then reports more than 400 and the host raises [S code; I scenario]. Stricter; a new flake source under vCPU preemption. | `kernel/src/m2/lock.rs:253-263`; `smp.rs:1497-1501`; `crates/smp-lock/src/lib.rs:67-74`; `run_smp.py:545` | Read | Decide deliberately: keep (then document it as a DG-1 tightening in the ADR) or record without judging LOCK overruns in the host. |
| C5 | MINOR | `STALL_MS = 1000` is a new guest-fatal limit with no normative source. A boot with a 1 s gap in aggregate progress that still finished inside the old 2 s passed before and now fails `SMP_SYNC_STALLED` [I]. Progress is one sum over all CPUs, so a single starved CPU is detected only after every other CPU has finished plus 1 s (bounded by the cap) [S]. The code comment calls the wait "ADR-082", whose text is not in force and, per `docs/PROJECT-STATE.md`, not yet aligned with these constants [S]. | `smp.rs:1373,1521-1526,1547-1558` | Read | State `STALL_MS` and its derivation in the ADR; consider per-CPU progress words if starvation of one CPU must be distinguished from slowness. |
| C6 | MINOR | Clock shadow accounting does not alter the verdict logic, but its observer effect is unmeasured. Every wake in the idle phase now performs 4 `inb` + 1 `outb` (up to 3 tries) with IF=0; before it performed none [S]. Under TCG each port access leaves translated code; any added host blocking can only merge more IRQ0 edges, i.e. more `SMP_CLOCK_DRIFT`, never fewer [I]. Consequently every failure rate of the 1 % rule measured on `cc1043f` includes the instrumentation; no control at `67e592c` was seen [U]. A new fatal marker `SMP_CLOCK_PIT_COUNT` exists in every `native-smp` boot [S]. | `smp.rs:2937-2946,2949-2962,3015-3018,3028-3035` | Read | Run a like-for-like control series on `67e592c` before quoting failure rates of the unchanged rule; record in the ADR that the PIT-count check is a new fatal path. |
| C7 | MINOR | `lock_waited` and `margin` run inside the critical section of every contended `IrqLock`, after the grant and before the owner is published: one `rdtsc`, two divisions and a scan of up to 64 atomics [S]. This lengthens contended hold times and the IQ-3 transit window in all acceptance images, including the normative scratch phase (500 ms), in the strict direction [I]; magnitude unmeasured [U]. | `lock.rs:250-269`; `smp.rs:1452-1471,1497-1501` | Read | Record LOCK waits in a direct per-level slot (`fetch_max` on one atomic), or take the timestamp at grant and book it after release. |
| C8 | MINOR | Margin keys are source line numbers. Keys change with any edit above a wait, so margins are not comparable across commits; two waits formatted onto one line would share a key and break "one key, one limit"; `id & 0x3fff` aliases lines above 16 383 [S]. The host exemption applies to every WAIT key: converting another wait (for example scratch, `smp.rs:4130`) to `within_progress` would be exempted with no oracle change [S]. Today only the sync wait can report `used > limit` for WAIT [S]. | `smp.rs:1453,1504-1519,1539-1541,2695-2733`; `run_smp.py:545` | Read; call-site census by Grep | Give each wait a stable name; restrict the exemption to the one named sync wait. |
| C9 | NOTE | AC-3 is only partly met: 36 of 40 `within_ms` limits are still unnamed literals; the aggregate does not summarise margins; the table has 64 slots for about 47–52 keys per boot [S census; I count]. Overflow is fail-closed (`SMP:MARGIN:OVERFLOW` is an unknown line) only in cases that reach `margins()` [S]. | `smp.rs:1441,1469-1470,1490-1492`; spec 09 `:20-21` | Grep census | Track as open AC-3 work; assert the key count in a host test. |
| C10 | NOTE | The oracle is no longer strictly exact-sequence for two line shapes: well-formed `SMP:MARGIN` lines (unique names) and one `SMP:CLOCK:EDGES` line are accepted at any position after the boot prefix [S]. Not security-relevant; relevant if either line later carries a verdict. | `run_smp.py:538-557` | Read | When a line becomes normative, add it to the expected sequence at a fixed position. |
| C11 | NOTE | Workflow separation holds; `diagnostic-gate-rehearsal` has no "refuse other modes" step (harmless, because every acceptance job is excluded). Diagnostic artefacts contain acceptance-shaped `result.json`/`serial.txt`; only deliberate manual relabelling could misuse them [S]. | `.github/workflows/native-smp.yml:465-598` | Read | Optional: add the refuse step to the rehearsal job. |
| C12 | NOTE | Documentation drift: the oracle comment says ROUND carries "the shootdown generation" but the guest always sends id 0; the `inb`/`outb` SAFETY comment does not mention the new PIC port 0x20 read; PR job timeout raised from 20 to 30 min [S]. | `run_smp.py:161-163`; `smp.rs:739,746,1980,1987,2939`; `native-smp.yml:54` | Read | Correct the comments. |

## 3. Answers to the questions

### Q1 — Normative limits

| Limit | Old | New | Normative source | Can a verdict differ? |
|---|---|---|---|---|
| Sync wait | 200 x 10 ms polls, fatal | `SYNC_MS = 2000` as logged figure; fatal at stall 1000 or 20 000 polls; host rejects `used > 2000` unless exempt | No normative value (ADR-070 asks only for a bounded test, comment `smp.rs:779-781`); raising needs an ADR (spec 09 `:52-53`) | Yes, both directions (C1, C2, C5) [S] |
| `STALL_MS` 1000, `PROGRESS_CAP` 10 | absent | new | None; ADR-082 is proposed | Yes [S] |
| Scratch | `within_ms(500, …)` | unchanged, `smp.rs:4130` | spec 01 `:52-57` | No in logic [S]; perturbed by C7 [I] |
| Clock | median of five segments within 1 % | unchanged expression, `smp.rs:3099-3101,3134,3156` | spec 06 `:48-49`, BT-1 `:23-25`; decisions of 10/11 Nov 2026 | No in logic [S]; observer effect [U] |
| Clock 4x observation bound | unchanged | `smp.rs:3050` | 10 Nov 2026 decision | No [S] |
| `LOCK_STUCK_MS` 400 | guest fatal | guest path unchanged, `smp.rs:335,354-360`, `lock.rs:253-259`; plus host rejection above 400 | spec 03 DG-1 `:18-22` | Stricter only (C4) [S] |
| Shootdown 100 ms | `rdtsc() > end` | unchanged, `smp.rs:313,1979-1981`; ROUND margin cannot exceed its limit | ADR-K06-5F; spec 06 `:52-53` | No [S] |
| `SMP:TIMER:RATIO` band | 50 000..75 000 | unchanged, `run_smp.py:142,563-565` | spec 06 BT-5 `:34-36`, ADR-080 | No [S] |
| Host watchdogs | 60 / 660 / 660 s | unchanged, `run_smp.py:148,151,157,700` | ADR-081 addenda | No [S]; `sync-slow` uses at least 20 s of 60 [I] |
| Idle wake 100 ms | unchanged | `run_smp.py:145` | — | No [S] |
| PIT count 1..=1193 | absent | new fatal `SMP_CLOCK_PIT_COUNT` | None | New failure mode (C6) [S] |
| Receptive span 1/4 period; latch shorter than 1/8 period; 3 tries | absent | `smp.rs:2917,2938,2942` | None | Diagnostic only [S] |
| Margin table 64 slots | absent | `smp.rs:1441` | None | Overflow rejects [S] |
| `sync-slow` TOTAL band 60 000..79 999; masked fixture 3 ms per 32 ticks; divisor x2 | absent | `run_smp.py:127,638`; `smp.rs:3029-3031,2811-2814` | None; fixtures | Fixture cases only [S] |
| Matrix size | 66 x 50 = 3300 | 71 x 50 = 3550 | spec 09 AC-3 (50/50 per case) | Aggregate and verifier counts updated consistently [S] |
| PR job timeout | 20 min | 30 min | None | No oracle effect [S] |

Confirmed unchanged in effect: scratch 500 ms, clock 1 %, shootdown 100 ms, RATIO band, host watchdogs [S].
`LOCK_STUCK_MS`: the guest effect is unchanged; the host adds a stricter check [S].

### Q2 — Sync wait

- **Passed before, fail now.** (1) A boot with 1000 consecutive polls without aggregate progress that still
  finished inside 2 s: `SMP_SYNC_STALLED` [I]. (2) A LOCK margin above 400 ms (C4) [I]. (3) A PIT count out
  of range or more than 64 margin keys [S]. No functional end-state behaviour is newly rejected.
- **Failed before, pass now.** A sync phase lasting from about 2 s up to 20 000 polls with progress at least
  every 1000 polls: (a) in the five oversubscribed multi-thread cases; (b) on any host in the terminal cases
  that never print margins (C2) [S]. In all other cases the same boot fails at the host with
  `Guest wait wait_<line> used N of 2000 ms` instead of a guest `FAIL` marker [S].
- **Poll granularity.** The old bound was 199 effective 10 ms waits; the new figure is 2000 one-millisecond
  waits, each with its own PIT channel 2 programming overhead that `used` does not count. In wall time the
  non-exempt limit is therefore slightly more lenient than before [I]; size unmeasured [U].
- **End-state checks.** Byte-for-byte as strict: `ATOMIC_TOTAL != expected || *LOCKED_TOTAL.lock() != expected`
  and `VIOLATIONS != 0` are unchanged; only fixture branches were added in front of the same `serial::fail`
  calls (`smp.rs:1404-1420`) [S]. `finished_stress` (DONE or IPI_READY) is unchanged (`smp.rs:1431-1433`) [S].
- **Progress value.** `ATOMIC_TOTAL + SERIAL_DONE`; both are written only by `fetch_add(1)` (`smp.rs:375,794`),
  so the sum is monotone and bounded by `(online + 1) * (20 000 + 200)` [S]. The wait always ends: ready,
  1000 stalled polls, or 20 000 polls [S]. `worst <= STALL_MS` always, so the STALL margin can never exceed its
  limit; a stall is reported by the guest, not by the margin [S].

### Q3 — Host oracle `evaluate()`

- **Exact sequence.** Still required: `tail` is compared for equality with the expected list in all three
  branches (`run_smp.py:644-651`); boot prefix, ACPI inventory, serial fixture lines, exit code and
  `FORBIDDEN = ("FAIL", "PANIC")` are unchanged [S].
- **Newly accepted.** Well-formed MARGIN lines with unique names and `used <= limit` (or a WAIT overrun when
  exempt) and one EDGES line, at any position (C10); the five new terminal sequences, each only under its own
  case name [S].
- **Newly rejected.** `limit == 0`; a repeated margin name; `used > limit` unless WAIT and exempt; a second
  EDGES line; `host > lost`; `own_us > us`; `SMP:MARGIN:OVERFLOW` and any malformed variant, which fall into
  `tail` and fail the sequence comparison [S]. The two EDGES sanity checks cannot fire on output of the
  committed guest: `host <= lost` is an invariant of `Edges::account`, and each `own` sample is at most its
  `sample` (`crates/smp-lock/src/clock.rs:47-72`; `smp.rs:3083,3090`) [I].
- **Forge or omit.** Omission of either line is accepted (C2). A guest that prints a smaller `used` or a larger
  `limit` is accepted; the oracle has no independent time base for a guest-internal wait and pins no limit
  (C2c) [S]. The guest cannot produce duplicate keys by a race (Q5) [I].
- **Malformed line into a pass.** Not found. Only fully matched digit groups reach `int()`; every other shape
  ends in `ValueError`, which `run()` records as a failed item (`run_smp.py:760-761`); other exceptions abort
  the run with `passed: False` already saved (`:684,697,762-765`) [S].
- **`host_cpus`.** Host-derived only [S]. The default when `None` is not safe (C3).

### Q4 — Clock shadow accounting

- **(i) No writes to the clock.** `CLOCK_TICKS` is written only by `lock inc` in the IRQ0 stub and
  `CLOCK_EDGE_TSC` only there; Rust code only loads them. `smp_lock::clock` is pure bookkeeping in a separate
  crate with no access to kernel statics [S]. `samples[]` uses the same `(ticks, edge TSC)` pair, loaded before
  `sample()` runs, and the same arithmetic; `OWN_US` is printed, and judged only in the masked fixture
  (`smp.rs:3010-3018,3079-3090,3119-3130`) [S]. The load phase passes `None` (`smp.rs:3176`) [S]. ADR-053 is
  respected at this commit; promoting `OWN_US` to the verdict would change BT-1's refutation criterion and
  needs a decision [I].
- **(ii) Port I/O.** `outb(0x43, 0x00)` is a counter-latch command for channel 0; it changes neither mode nor
  reload. Both count bytes are always read before the result is judged (`smp.rs:2943`), so no latch is left
  half-read for the later M3 re-initialisation [S]. `wait_us` uses ports 0x61, 0x43 (control word 0xb0) and
  0x42 only (`smp.rs:755-776`), so channel 2 is separate [S]. No OCW3 is written to port 0x20 anywhere in the
  non-legacy kernel, so `inb(0x20)` returns IRR after ICW1 [S for the absence of writes; I for the PIC default,
  taken from QEMU i8259 semantics and K06B-CLOCK, not re-verified against QEMU source]. A latch left pending by
  firmware on channel 0 would make the first sample read a stale count [U]; if present it is deterministic and
  the first hosted boot shows it.
- **(iii) `SMP_CLOCK_PIT_COUNT`.** Fires only for `count == 0 || count > 1193` on a sample that passed the
  IRR-stable, short-latch filter [S]. Under QEMU's mode 2 the readable range is 1..=reload [I, from memory of
  `hw/timer/i8254.c` and K06B-CLOCK P1].
- **(iv) Lost edges and BT-3.** It cannot be shown statically that the added work never costs an edge. An edge
  is lost only when two periods pass while IRQ0 is pending; the added window is five port accesses per wake, so
  a loss needs host-side blocking of about a period [I]. The effect is in the strict direction and a long
  window makes the sample non-receptive, so the loss is charged to the kernel and not hidden in `OWN_US`
  (`smp.rs:2981-2984`) [S]. BT-3 holds: sampling takes no lock and `idle_halt` with its lockdep and CR3 check
  is unchanged (`smp.rs:2737-2749`) [S].

### Q5 — Margin table

- **Races.** Insert is a CAS on the first empty slot; a loser re-reads the winner's word and continues;
  slots are never cleared. Two CPUs with the same key cannot occupy two slots, and `fetch_max` keeps the
  maximum [I, by case analysis of `smp.rs:1455-1468`]. `margins()` may print a LOCK value that later grows
  and omits keys inserted after its scan; both are benign [I].
- **Locks and contexts.** `margin()` takes no lock and uses atomics only, so it is safe on APs, with IF=0 and
  inside the `IrqLock` slow path; no lockdep interaction [S]. `margins()` prints through the per-CPU line
  buffer under `SERIAL`, like every other marker [S].
- **Hot path.** One call per finished wait or shootdown round; one per contended `IrqLock` acquisition, inside
  the critical section (C7) [S].
- **Overflow.** A 65th key sets `MARGIN_LOST`; the resulting line is rejected (C9). Values saturate at
  16 777 215 [S]. `lock_waited` is accurate to about 1 ms and would be distorted by a `TSC_PER_MS`
  recalibration during a wait [I].
- **`#[track_caller]`.** All callers are in `smp.rs`, so the line is unambiguous today; the coupling of
  evidence keys to source lines is finding C8 [S].

### Q6 — Negative fixtures

- **Reachability.** All five are `cfg!(feature = …)` branches; the features imply only `native-smp`, `default`
  is empty and nothing else enables them (`kernel/Cargo.toml:9,53-57`). `tools/build.py:59-60` only adds them to
  the permitted `--kernel-feature` values [S].
- **Exit only on own marker.** `sync-stall` exits 0x10 only when the outcome is `Stalled`, `sync-slow` only
  when `Late`, `sync-lost-update` only in the totals branch, `clock-masked` only when both medians exceed 1 %,
  `clock-divisor` only in the PIT-count branch; any other failure reaches `serial::fail` [S].
- **Fail-closed.** `SMP_TEST_SYNC_NOT_DETECTED` (`smp.rs:1411-1417`), `SMP_TEST_CLOCK_MASK_NOT_CHARGED`
  (`:3129`), `SMP_TEST_CLOCK_DIVISOR_NOT_DETECTED` (`:3131-3133`) [S].
- **Consistency.** Cargo features, `build.py`, `CASES` (71), `TERMINAL`, `SYNC_WRITERS`, expected sequences,
  the PR step (`native-smp.yml:221-228`), 70 matrix rows plus the soak job, both verifiers (71, 3550) and the
  aggregate job name `SMP section 9 aggregate 71x50` agree [S].
- **Limits.** The set is smaller than the negatives ADR-082 proposes (no lock-holding deadlock, no single-CPU
  starvation, no case for the host performance gate itself) [I]. No fixture proves that a slow but alive boot
  is rejected by the host on a non-exempt profile; that path is covered by a host unit test only [S, not run].
  The watchdog margin of `sync-slow` and the hosted behaviour of all five are unmeasured [U].

### Q7 — Workflow

- **Acceptance output.** None. Every acceptance and PR job carries `!inputs.margin_diagnostic`
  (`native-smp.yml:51,255,314,602,671,727,778,1034`). With `margin_diagnostic=true` combined with `full_matrix`
  or `clock_diagnostic`, those jobs stay skipped and `diagnostic-margin` exits 2 in its refuse step
  (`:487-494`) [S]. The report is renamed `diagnostic/margin-report.json` and the artefact is named
  `DIAGNOSTIC-NOT-ACCEPTANCE-…`; the aggregate downloads `smp-*` and both it and the verifier search for
  `smp-report.json`; the final-gate verifier requires the two named jobs to have succeeded [S; I that skipped
  jobs are rejected by the verifier, not executed].
- **Injection.** None found. `margin_cases` reaches only `fromJSON` (matrix), `name:`, `env:` and `with:`;
  `margin_repeat` reaches only `env:`. No `run:` block interpolates an input or matrix value; the shell reads
  quoted environment variables (`:488-494,513-535`) [S]. An unknown case fails with `KeyError` under
  `bash -eo pipefail`; `--repeat` is `type=int` (`run_smp.py:788`) [S]. That the `fromJSON` matrix of a skipped
  job is not evaluated on push and pull-request events follows GitHub's documented order [I].

### Q8 — Anything else

- No change to paging, capabilities, IPC, user-mode entry or lockdep order; the only non-fixture kernel change
  is the call inside `IrqLock::take` (C7) and the two lines in `m2.rs` [S].
- `within_ms` is semantically identical (`any` became `position` plus `is_some`) [S].
- The observer TSC returned by `clock_at` is now read after sampling, so the lag figures printed on a drift
  failure include sampling time; diagnostic only [S].
- No test was removed or relaxed in `tools/tests/`; both files only gain tests [S]. They were not executed [U].
- The repository states that this work is unaccepted and provisional (`docs/PROJECT-STATE.md`), and the owner
  authorisation it cites exists only as that text [U]. The proposed ADR still asserts that nothing changed (C1).

## 4. Not verified (open items, not approvals)

1. No build, host test, boot or workflow run was executed; every behavioural statement is from reading.
2. QEMU i8254 and i8259 behaviour (latch, mode 2 count range, IRR default, edge merging) was taken from memory
   and K06B-CLOCK; QEMU 8.2.2 and 10.2.1 sources were not consulted.
3. Firmware state of PIT channel 0 at hand-off (pending latch) on the hosted OVMF build.
4. The observer effect of the clock sampling on the lost-edge rate; no control series at `67e592c` was seen.
5. The cost of the margin instrumentation on contended `IrqLock` paths and on the scratch phase.
6. Hosted behaviour of the five fixtures, the `sync-slow` margin against the 60 s watchdog, and whether the
   divisor fixture passes every phase before the clock phase on QEMU 10.2.1.
7. The exact list of terminal cases in C2(b) was derived from expected sequences, not from boots.
8. `tests/test_ci_workflows.py` and other tests outside the audited paths; GitHub's evaluation order for the
   matrix of a skipped job.
9. Size of the evidence file with `guest_waits` for fifty boots against the 1 MiB limit.
10. The owner's authorisation for the implementation, and behaviour under KVM or on physical hardware.
