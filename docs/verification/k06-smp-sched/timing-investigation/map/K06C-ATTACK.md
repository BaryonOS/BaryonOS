# K06C-ATTACK — adversarial review of the timing-robustness change and proposals A and B

- **Task:** K06C-ATTACK (read-only red-team review). **Date:** 12 November 2026 (Europe/Amsterdam).
- **Baseline:** frozen worktree at commit `cc1043f`; change under attack `git diff 67e592c cc1043f -- . ':!docs'`.
  All `path:line` references are relative to that worktree. QEMU references are to the 10.2.1 excerpts in
  the session scratchpad (`qemu-src/v10.2.1/`).
- **Method:** source reading, light `git` queries and arithmetic on stored series only. Nothing was built,
  booted or executed. The hosted Graphify graph represents `main`, not this worktree, and was not used;
  the code was read directly.
- **Labels:** [S] verified in source, [D] verified in data, [I] inferred, [U] unverified.
- **Data sets:** local series on a 4-CPU host with QEMU 8.2.2: `a-*` at `ed05150` (margins and progress
  rule, no edge accounting): `four` 20 boots, `eight` 30, `nine-capacity` 30; `b-*` at `cc1043f`: `four` 30,
  `eight` 30. Hosted run 43187501026 (3 301 boots, pre-change source) and the hosted PR job at `ed05150`
  (43 guests, one boot per case). No hosted data exist for `cc1043f`.

## 1. Verdict

Proposal B is not acceptable as worded. Its "`extra` ticks stay fatal" clause fails healthy boots: the
shadow accounting reported `EXTRA=1` in 6 of 60 healthy local boots [D], through a first-sample path that
is visible in the source [S]. The `receptive` predicate, on which the whole host/kernel attribution rests,
only observes two short spans around each tick; every interrupts-off interval outside those spans, and
every loss caused by another CPU, is booked on the host [S]. Combined with the proposed 20–25 % per-boot
bound, the effective tolerance for such loss rises from 1 % to 20–25 % per boot, and the per-case median
rule has no power against a defect that affects fewer than half the boots and does not exist on
single-boot PR CI [I]. The same per-case rule rejects a healthy kernel on the local host for case `four`
[D]. The implemented sync rule converts the normative 2 s limit from a kernel-fatal condition into an
oracle check on an optional line whose limit is supplied by the kernel itself, with a verdict that depends
on the CPU count of whichever machine evaluates the log [S]. On the five oversubscribed cases a 1x–10x
slowdown now passes, and no acceptance environment enforces the limit for eight parallel CPUs [S]. The
shadow-mode sampling may itself degrade the unchanged 1 % rule: `SMP_CLOCK_DRIFT` rose from 4/80 boots at
`ed05150` to 23/60 at `cc1043f` on the same host [D]; causation is unverified [U] and must be settled by a
same-host A/B before any `cc1043f` clock data are used as evidence. A matrix pass probability of at least
99 % for 3 550 guests cannot be demonstrated statistically with the available or any practical sample
(Section C); the observed hosted rate is two orders of magnitude away from the required per-guest rate [D].

## 2. Ranked findings

| ID | Sev. | Target | Scenario | Trace | Minimal fix |
|---|---|---|---|---|---|
| F01 | BLOCKER | proposal B | A healthy boot reports `extra = 1`; B keeps `extra` fatal. | (1) The BSP enters `clock()` with IRQ0 already pending, so the first `sample()` only seeds the books: `Edges::new(pending)` with `opened = (1, false)` (`kernel/src/m2/smp.rs:2964-2967`, `crates/smp-lock/src/clock.rs:37`) [S]. (2) The first `hlt` delivers the old merged edge; the next sample is `periods=0, counted=1`: `gained = 0 + self.pending = 1 = used`, so `self.opened = (0, …)` and the first-sample allowance is erased (`clock.rs:49-58`) [S]. (3) QEMU raises the IRQ of the current period late: the counter is computed from the clock (`hw_timer_i8254.c:53-76`) while the IRQ comes from a main-loop timer (`:260-288`) [S]; the second IRQ0 in the same lattice period gives `back=1, late=min(1,0)=0, extra += 1` (`clock.rs:66-70`) [S]. (4) `EXTRA=1` in 4/30 (`four`) and 2/30 (`eight`) healthy boots at `cc1043f` [D]. A second path is documented by the crate's own test: `(2,0),(1,1),(0,1)` yields `(2,2,1)` (`clock.rs:172`, `:192`) [S]. | Make only the net invariant fatal (IRQ0 count greater than lattice periods + 1 over the whole window); keep prefix `extra` diagnostic. Keep the first-sample allowance open until a sample with `periods >= 1`. Add unit test `run(true, [(0,1,F,T),(0,1,F,T)])`. |
| F02 | MAJOR | proposal B | Kernel-caused IRQ0 loss is booked as `host`. | `receptive = !blind && ticks == self.ticks+1 && halted - entry < period/4 && tsc - entry < period/4` (`smp.rs:2975-2986`) [S]. (a) Another vector or NMI wakes the BSP from `hlt` and runs k periods with IF=0; edges merge in the PIC (`hw_intc_i8259.c:145`); at `iretq` the pending IRQ0 is taken before `cli` (`smp.rs:2737-2749`) [I]; the stub stamps a fresh entry TSC (`smp.rs:430-449`); the sample sees one tick and two short spans, so `missing = k` goes to `host` (`clock.rs:55`) [S]. (b) The stub reads the LAPIC ID by MMIO before `rdtsc` (`smp.rs:432-440`): time before the stamp is in neither span [S]. (c) `halt()` stamps `halted` before the `idle_halt()` preamble (`smp.rs:2991-2996`, `2737-2748`) [S]. (d) Loss induced by another CPU (PIC mask, ICW1, PIT rewrite, BQL-heavy loop) leaves both BSP spans short; APs are not asserted idle during the precision phase (`smp.rs:963-1090`) [S]. (e) IF=0 for up to 1/4 period per tick counts as fully receptive (`smp.rs:2916-2917`) [S]. | Require an unchanged BSP interrupt-entry/NMI counter (except the one IRQ0) for `receptive`; stamp the entry TSC before the MMIO read; stamp `halted` immediately before `sti`; assert AP idle state and unchanged PIC IMR/LINT0 across the window; derive the span threshold from measured data. |
| F03 | MAJOR | proposal B | Tolerance for host-booked loss rises from 1 % to 20–25 % per boot. | (1) Today any boot with raw median more than 1 % off fails (`smp.rs:3118`, `3134-3156`) [S]. (2) Under B a boot passes when `OWN_US` is within 1 % and host loss is under the bound; `OWN_US` removes exactly the host-booked periods (`smp.rs:3088-3090`) [S]. (3) The per-case median over 50 boots moves only when more than half of the boots are affected; PR CI runs one boot per case, so no per-case rule exists there [I]. (4) A defect of type F02 losing up to 20–25 % of ticks in fewer than 25 of 50 boots passes every rule [I]. | Bound host-booked loss far below 20 % (local maximum 2.96 % [D]); add a per-case bound on the upper quantile, not only the median; close F02 first; state explicitly that single-boot CI carries no clock-accuracy evidence under B. |
| F04 | MAJOR (BLOCKER if confirmed) | implemented (shadow mode) | The accounting may perturb the unchanged 1 % gate. | (1) `cc1043f` adds one `sample()` per loop iteration of `clock_at` (`smp.rs:3003-3037`): two `inb(0x20)`, `outb(0x43)`, two `inb(0x40)` and `rdtsc` reads with IF=0 before each `hlt` (`smp.rs:2937-2948`) [S]. (2) Same host, same rule: `SMP_CLOCK_DRIFT` 1/20 (`four`), 0/30 (`eight`), 3/30 (`nine-capacity`) at `ed05150`; 15/30 (`four`), 8/30 (`eight`) at `cc1043f` [D]. (3) The series ran about 4.5 h apart under unknown background load; causation is not established [U]. | Interleaved same-host A/B with the sampling compiled out; if confirmed, reduce the per-tick I/O (single latch, or sample every Nth tick) before relying on any `cc1043f` clock evidence. |
| F05 | MAJOR | implemented (oracle) | The 2 s sync limit is no longer enforced by anything mandatory. | (1) The kernel fails only on stall or at 10x (`smp.rs:1367-1427`, `1524-1563`) [S]. (2) `evaluate` checks `used > limit` only if a margin line is present, takes `limit` from that line, and does not know which key is the sync wait (`tools/run_smp.py:164`, `:544-548`) [S]. (3) `host_cpus` defaults to the evaluating machine's affinity and is not stored (`run_smp.py:502-503`) [S]; `verify_artifacts.py:246-247` replays `evaluate` without it, so the same log can pass on a 4-CPU verifier and fail on an 8-CPU one [S]. (4) ADR-082's own `margin-oracle` case ("margin line missing or malformed: host oracle rejects", `docs/DECISIONS.md:2680-2741`) is not implemented [S]. | Make a named sync margin line mandatory for every case that prints `SMP:SYNC:PASS`; pin site-to-limit in the oracle; record `host_cpus` (and its source) in the report and replay with the recorded value. |
| F06 | MAJOR | implemented + proposal A | A 1x–10x performance regression passes on oversubscribed cases; nothing tests eight parallel CPUs against 2 s (or 500 ms under A). | (1) Exemption: `margin[1] == "WAIT" and oversubscribed` (`run_smp.py:545`) [S]. (2) On a 4-CPU runner this covers `eight`, `nine-capacity`, `m3-eight`, `m3-migrate-eight`, `m3-load-eight` [S]. (3) The single-thread 8-vCPU cases have no real parallel contention [I]. (4) The exemption is by kind, not by site, so every future `within_progress` site inherits it [S]. | Run at least one non-oversubscribed 8-vCPU job (or a calibrated work-per-CPU-time check), and scope the exemption to named sites. |
| F07 | MAJOR | proposal B | The per-case median rule rejects a healthy kernel. | Local `four`, 30 boots: sorted raw medians have 1 009 787 and 1 010 220 µs in the middle (+0.994 % and +1.037 % of the kernel's reference 999 847 µs, `smp.rs:3118`); 15 of 30 exceed the threshold [D]. Local `eight` passes (+0.70/+0.73 %) [D]. The rule is undefined for even n, for boots without an EDGES line, for soak shards and for the single-boot PR job [S, absence]. The 50 boots of a case run sequentially on one runner, so host noise is correlated within the sample [I]. | Do not judge the raw median at 1 % on noisy hosts; if a per-case rule is kept, judge kernel-charged loss (`LOST - HOST`) and define the estimator, minimum n and missing-line handling. |
| F08 | MAJOR | proposal A | A leaves the same convoy class fatal at other sites and has no progress witness for half of the scratch work. | (1) Lock pair: `within_ms(2_000, …)` (`smp.rs:4214-4227`) is not covered; local maxima 386/2000 and 422/2000 at 8 on 4 [D]; series M recorded 2.71 s at 4 on 2 (`timing-investigation/map/K06B-LOCK.md:359`) [D]. (2) `LOCK_STUCK_MS` = 400 ms (`smp.rs:335`, `kernel/src/m2/lock.rs:251-264`) stays binding inside scratch [S]. (3) The 1 024 fill rounds advance no shared counter (`smp.rs:4081-4108`) [S]: a progress rule sees a stall while all CPUs fill. (4) `cpus > host_cpus` ignores the QEMU main loop, SMT, cgroup quota and steal time [I]. (5) The timeout path is `quarantine_all`, with different side effects from a marker-only failure [S]. | Apply the same progress rule to the lock pair; add a per-CPU scratch progress counter that also advances in the fill rounds; define the profile predicate on measured capacity rather than affinity count. |
| F09 | MINOR | implemented | `STALL_MS` (1 000 polls) is shorter than `SYNC_MS` (2 000): a new, tighter failure. | A tail with a complete 1–2 s stall that still ends within 2 s passed before and now fails `SMP:SYNC:STALLED` (`smp.rs:1379`, `1524-1563`) [S]. Local maxima of the no-progress run: 14 and 7 polls [D]. The window counts polls, so a whole-VM freeze does not trigger it, but poll time is not wall-clock time [S]. Progress is the sum over CPUs, so one dead AP is detected only after the others finish [S]; this differs from ADR-082 ("per-CPU … within W = 2 s"). | Record the decision and its evidence in ADR-082; consider per-CPU progress. |
| F10 | MINOR | implemented | New oracle rejection for a LOCK margin above 400. | A waiter descheduled across the deadline at the per-iteration pause finds the lock granted on resume, so `lock_stuck` is not reached, and `lock_waited` records `waited > 400` (`lock.rs:253-263`, `smp.rs:1497-1501`); the oracle rejects `used > limit` for kind LOCK (`run_smp.py:545`) [S]. Previously a silent pass. `lock_waited` also runs while the lock is held and scans the shared margin table, lengthening hold times [S]. | Clamp or exempt the benign overshoot explicitly, or record the margin after release. |
| F11 | MINOR | fixtures | Fixtures cover the cap and one span clause only; `sync-slow` is fragile. | See Section D. | See Section D. |
| F12 | MINOR | proposal B | A single host stall of 1–1.25 s becomes fatal. | 5 001 ticks × 20–25 % ≈ 1 000–1 250 periods of host loss in one boot; today the five-segment median absorbs a stall confined to one or two segments (`smp.rs:3099-3101`) [S]. | Bound host loss per segment-median, or accept the stall and bound the count of affected segments. |
| F13 | MINOR | proposal B | `OWN_US` carries systematic biases; kernel-charged loss is unbounded. | Calibration takes the minimum of ten samples (`smp.rs:341-351`), so `OWN_US` sits at −0.22…+0.16 % (`four`) and −0.19…+0.02 % (`eight`) [D]. Segment boundaries use `saturating_sub` (`smp.rs:3088-3089`) [S]. The median ignores up to two segments, so kernel-charged loss of any size in two segments passes, as today [S]. | Add a per-boot bound on `LOST - HOST` (local maximum 19 of 5 001 [D]). |
| F14 | NOTE | implemented (pre-existing) | A deadlock during the BSP's own stress share has no marker. | `stress()` on the BSP takes the raw `LOCKED_TOTAL.lock()` without a bound (`smp.rs:788-811`); the progress rule guards only the tail (`smp.rs:1367-1371`) [S]. Result: 60 s host watchdog (`run_smp.py:148`). | Bound the BSP share or run it under the same progress observer. |
| F15 | NOTE | implemented | "Shadow mode" is not purely diagnostic; the decision record is stale. | `SMP_CLOCK_PIT_COUNT` is fatal now (`smp.rs:2961`) and the oracle rejects a repeated or inconsistent EDGES line (`run_smp.py:553`) [S]. ADR-082 still reads "PROPOSED — not accepted … the committed source is unchanged" while kernel and oracle semantics changed [S]. 2 000 × `wait_us(1_000)` is longer in wall-clock time than 200 × `wait_us(10_000)` [I]. | Update ADR-082 before acceptance; list the new fatal paths. |
| F16 | MAJOR | statistics | A pass probability of at least 99 % is not demonstrable. | Section C. | Report an estimate with stated assumptions, not a demonstrated bound. |

## A. A real defect passes

### A.1 Implemented sync rule

- A sync tail up to 10x the limit (about 20 000 polls) passes the kernel [S] (`smp.rs:1526`, `1540-1563`).
  On non-oversubscribed profiles the oracle rejects it only if the margin line is present [S] (F05).
- A lost or suppressed margin line passes [S]: no line is required by `evaluate`, and `SMP:MARGIN:OVERFLOW`
  handling by the oracle was not traced [U]. Observed key counts are 43–48 of 64 slots [D].
- A kernel change of `SYNC_MS` is accepted by the oracle because the limit is read from the guest line [S].
- One starved or dead AP is masked by summed progress until the remaining CPUs finish; detection then
  takes a further 1 000 polls [S]. The defect is still detected, but later and without identifying the CPU.
- A livelock that keeps incrementing `ATOMIC_TOTAL` (which is incremented before the lock is taken,
  `smp.rs:788-811`) counts as progress until the 10x cap [S].
- A deadlock inside the BSP share is outside the rule (F14) [S].

### A.2 Proposal A

- Throughput collapse on oversubscribed profiles passes up to 10x by design [I]; no profile checks eight
  truly parallel CPUs against 500 ms (F06).
- A scratch defect that makes one CPU crawl while others progress passes until the cap, unless progress is
  tracked per CPU [I].
- The existing negative fixtures do not include a scratch stall or crawl (Section D) [S].
- The profile predicate can be satisfied or defeated by the environment alone (affinity, quota), so the same
  kernel is judged by different rules on different verifiers [S] (F05).

### A.3 Proposal B

**(i) Kernel-caused IRQ0 loss booked as `host`.** Yes, through F02 (a)–(e). In the healthy kernel at
`cc1043f` no other handler is expected on the BSP in the idle precision phase, so (a) needs a second defect
[I]; (b), (c) and (e) are present today and enlarge the non-receptive time that is treated as receptive [S].
Locally 96–97 % of all lost periods were booked on the host (2 034 of 2 116 on `four`, 1 561 of 1 608 on
`eight`) [D]; `SPAN_US` exceeded the 250 µs threshold at least once in every one of 60 boots [D], so the
threshold is exercised constantly and its value is not supported by a measured distribution [U].

**(ii) Systematic loss rate that passes.**

| Loss booked as | Pattern | Today | Under B |
|---|---|---|---|
| kernel | every boot, uniform rate r | fails above about 1.0 % | fails above about 1.0–1.2 % (baseline of `OWN_US` is negative) [I from D] |
| kernel | confined to at most 2 of 5 segments, any size | passes (median) | passes; no bound on `LOST - HOST` [S] |
| host (F02 paths) | every boot, uniform rate r | fails when r plus host loss exceeds 1 % | single boot: passes up to the 20–25 % bound; matrix: fails only when r plus the median host loss exceeds 1 % [I] |
| host (F02 paths) | fewer than half of the boots | every affected boot fails | passes entirely up to 20–25 % [I] |

**(iii) Arithmetic that hides loss.**
- First sample: the seed sample is not accounted; one doubled tick at the start is absorbed by
  `opened = (1, false)` [S]. The same construct causes F01.
- Blind samples (latch retries exhausted, `smp.rs:2951`) set `receptive = false`; loss is charged to the
  kernel, which is conservative [S].
- `lost.saturating_sub(late)` and the segment-boundary `saturating_sub` cannot raise `host` above `lost`
  [I from S]; they can leave up to one period of uncompensated credit per boundary [I].
- Netting: a doubled tick following an open loss booking (`(3,1),(1,2)`) is consumed as a late arrival and
  never reaches `extra` [S]. The net count stays within the lattice, so elapsed time is not overstated, but
  a spurious-tick defect can hide behind real loss [I].
- Divisor errors: a doubled divisor is caught only by the count check; an error of +x % below 50 % yields
  `periods = 1` per tick, `OWN_US` equals the raw value and the 1 % rule applies; a smaller divisor produces
  no `extra` until the period falls below half the nominal value [I from S].

**(iv) Corruptible inputs of `receptive`.** `CLOCK_EDGE_TSC` (written by the stub after an MMIO read),
`halted` (stamped before the halt preamble), the tick delta, and `period`, which is derived from the
calibrated `per_ms` (`smp.rs:2901-2917`) [S]. An overestimated `per_ms` widens the span threshold and
shifts the lattice; the lattice is re-anchored at every non-zero `periods`, so only long gaps (more than
roughly 100 periods at 0.5 % bias) can mis-round [I]. The PIT count is trusted for the edge phase [S].

## B. A healthy boot fails

- **`SMP:SYNC:STALLED`.** Needs 1 000 consecutive polls without any progress on any CPU [S]. Observed
  maxima are 14 polls (`nine-capacity`) and 7 (`eight`) [D]. A complete VM freeze does not count [S]. A
  lock-holder vCPU thread descheduled for about 1 s of BSP polling would trigger it; the probability on
  hosted runners is not measured [U]. The hosted `nine-capacity` incident (one counter sample,
  `DONE=0:TOTAL=157489`) cannot distinguish slow progress from a stall [U].
- **`SMP_CLOCK_PIT_COUNT`.** Cannot fire on QEMU mode 2: the count is in 1..divisor
  (`hw_timer_i8254.c:72`) [S]. A latch left by firmware survives the mode set (`hw_timer_i8254.c:121`,
  `:161-171`) and would poison the first read deterministically [I]; not seen in 60 local boots [D], OVMF
  on the hosted runner unverified [U].
- **Margin table overflow.** 43–48 of 64 keys over 43 hosted guests and the local series [D]; not all 71
  cases were counted [U].
- **LOCK margin above 400.** New rejection path (F10) [S]; local maxima 100/400 and 117/400 for the serial
  lock [D].
- **Oracle rejections from new lines.** `host > lost` and `own_us > us` cannot occur in a correct kernel:
  `own[i] <= samples[i]` element-wise, and the take-back never reduces `lost` below `host` [I from S]. The
  field widths of both regexes are sufficient [S]. The recomputed verdict can differ between verifiers
  (F05) [S].
- **`extra` fatal under B.** Fails about 10 % of healthy local boots (6/60; 95 % interval about 4–21 %) [D].
- **Per-boot host-loss bound under B.** Local maximum 2.96 % [D]; a single 1–1.25 s host stall exceeds a
  20–25 % bound (F12) [I]. Hosted distribution unknown [U].
- **Per-case median under B.** Fails on local `four` (F07) [D].
- **Kernel-charged false loss under B.** `LOST - HOST` reached 19 and 9 periods per boot (0.38 %, 0.18 %)
  on a healthy kernel [D]; these stay in `OWN_US`. A noisier host can push this above 1 % [I].
- **Host watchdog.** The `sync-slow` fixture uses about 30 s of the 60 s guest timeout [D]; `m3-load-eight`
  uses 46.0–47.3 s of 60 s on hosted runners [D].

## C. Statistics

- **Required rate.** (1 − p)^3550 >= 0.99 requires p <= 2.83e-6 per guest, about 1 in 353 000 [S, arithmetic].
- **Observed rate.** Hosted run 43187501026: 3 failures in 3 301 boots, p ≈ 9.1e-4 [D]. At that rate the
  matrix pass probability is about exp(−3.2) ≈ 4 % [I]. The required improvement is about 320x.
- **Sample needed.** Zero failures in n boots bounds p at 95 % confidence by 3/n; n >= ln(20)/2.83e-6 ≈
  1.06 million boots, that is about 298 consecutive clean matrices [S, arithmetic].
- **What k clean matrices show.** The 95 % upper bound on the matrix failure probability is
  1 − 0.05^(1/k): k = 1 gives 95 %, 10 gives 25.9 %, 30 gives 9.5 %, 59 gives 5.0 %, 298 gives 1.0 % [S].
- **Non-brute-force argument.** A structural argument (each known mechanism removed or given a measured
  margin) supports an engineering estimate, not a 95 % confidence statement [I]. It rests on three
  assumptions that the data do not carry: the mechanism list is exhaustive; tails can be extrapolated; and
  failures are independent. Local margins are heavy-tailed: the serial lock wait reached 117/400 ms, where
  an exponential tail gives P(> 400) of about 8e-6 per boot and a power-law tail about 5e-3 [I]. Failures
  cluster per runner, which makes boots within one job weakly informative about other runners [I]. A
  limit expressed as progress rather than wall-clock time removes a tail instead of extrapolating it; that
  part of the argument is sound where the progress witness is complete (see F08, F09) [I].
- **Defensible statement.** "No known mechanism with an estimated per-guest rate above X remains, under
  assumptions A1–A3; k clean matrices observed" [I]. A claim of ">= 99 %" without that qualification is
  not supported.

**Residual mechanisms, most plausible first** (after the implemented change, A and B as worded):

1. Clock under B: `extra` fatal (10 % locally) [D]; then the per-case median under correlated host noise
   [D]; kernel-charged false loss [D]; the single-stall bound [I].
2. Scratch under the unchanged 500 ms rule wherever A does not apply: 408/500 ms and 244/500 ms maxima
   at 8 on 4 [D]; series M 2.65 s at 4 on 2 [D].
3. `SMP:LOCKPAIR:TIMEOUT` at 2 s: 422/2000 and 386/2000 at 8 on 4 [D]; 2.71 s at 4 on 2 [D].
4. `SMP:LOCK:STUCK` at 400 ms (serial lock in the sync tail, kernel lock in scratch, pair lock): local
   maxima 100–117/400 [D]; plus the new LOCK-margin oracle path (F10).
5. `SMP:SYNC:STALLED`, the 10x cap and the 60 s watchdog behind it [I].
6. Fixed `within_ms` sites of 100/200/500 ms and the 100 ms shootdown rounds: local maxima 11/100 and
   8/100 [D]; 20 s wake and isolation deadlines at 1 486/20 000 [D].
7. Host watchdogs: `m3-load-eight` 47.3/60 s, soak 612/660 s, `m3-ipc-four` 209/660 s; the documentary
   264 s IPC rule sits at 79 % and is not enforced by a tool (`docs/DECISIONS.md:2654`) [D, S].
8. `SMP:TIMER:RATIO` band 50 000–75 000 (`run_smp.py:142`): 2 of 110 local boots rejected (78 060,
   78 661); local `eight` maximum 68 607; hosted maximum 65 138 [D].
9. TSC calibration bias (minimum of ten 10 ms samples) [S]; its effect on `OWN_US` at 8 vCPUs on hosted
   runners is unmeasured [U].
10. QEMU model effects (merged edges at timer catch-up, `util_qemu-timer.c:533-567`) [S]; margin table
    overflow [I].

## D. Negative fixtures

- **`test-smp-sync-stall`.** The first AP parks after counting, outside any lock (`smp.rs:898-908`) [S].
  It exercises the real stall path; a broken detector would end at the cap with the other marker [I]. It
  does not cover a deadlock, the death of a lock holder, a stall inside the BSP share, or the window length.
- **`test-smp-sync-slow`.** Reaches only the 10x cap. Nothing boots a guest in the 1x–10x band that only
  the oracle polices [S]. Fragile: CPU 1 needs about 40 s for its share, the cap is reached at about
  21–23 s; a sustained 1.8–2x stretch of poll time lets CPU 1 finish, and the oracle window
  `60_000 <= TOTAL < 80_000` (`run_smp.py:638`) or `SMP_TEST_SYNC_NOT_DETECTED` then fails [I]. Observed
  totals 70 552 and 70 411 [D].
- **`test-smp-sync-lost-update`.** Injected arithmetic on the pre-existing compare path; adequate for
  that path [S].
- **`test-smp-clock-masked`.** Inserts `wait_us(3_000)` with IF=0 between the sample and `halt()` on every
  32nd tick (`smp.rs:3029`) [S], that is inside the measured `halted − entry` span. It tests that clause
  only; not the `tsc − entry` clause, the tick-count clause, the blind clause, a PIC mask, or any F02 path.
  The pass criterion requires only that both medians are more than 1 % off (`smp.rs:3119-3129`) [S]: the
  fixture passed with `HOST=95` of `LOST=409` and `HOST=52` of `LOST=355` [D], so up to roughly 84 % of the
  kernel-masked loss could be booked on the host before it fails [I]. The comment "none of that loss may be
  booked on the host" (`smp.rs:3120-3121`) is not what the code checks [S].
- **`test-smp-clock-divisor`.** Doubled divisor only; ends at the count check on the first samples
  (`smp.rs:2957-2961`) [S]. No fixture for a non-integer error of 2–10 %.

**Defect classes without a fixture:** IRQ0 loss in an unmeasured span (other handler or NMI in the halt
window, time before the entry stamp); loss caused by another CPU or a PIC mask; a doubled or synthesised
tick (`extra`); host-booked systematic loss against the per-boot bound and the per-case rule; deadlock
during the BSP share; lock-holder death; one starved CPU; the 1x–10x performance band (`perf-gate`);
missing, malformed or overflowed margin lines (`margin-oracle`); LOCK margin above 400; scratch stall or
crawl; TSC miscalibration. ADR-082 itself lists `sync-deadlock`, `sync-starved`, `perf-gate`,
`clock-synthesised` and `margin-oracle` as required; these are absent [S].

## E. Not verified

- Nothing was built, booted or run; all behaviour statements are from source and stored data.
- No hosted data exist for `cc1043f`: EDGES, `OWN_US`, `extra`, host-loss fraction and margin
  distributions on QEMU 10.2.1 hosted runners are unknown. Local results use QEMU 8.2.2.
- Whether the six local `EXTRA=1` boots arose at the first samples (F01 path 1) or later (path 2); the
  line reports totals only.
- The cause of the rise in `SMP_CLOCK_DRIFT` between `ed05150` and `cc1043f` (F04).
- Interrupt acceptance at the `iretq`/`cli` boundary under TCG (F02 a), inferred from source.
- QEMU 8.2.2 device code was not re-read line by line; the 10.2.1 excerpts were used.
- OVMF's PIT and LAPIC state at hand-off, and main-loop lateness magnitudes on hosted runners.
- Whether the hosted `nine-capacity` sync incident was a stall or slow progress.
- How the oracle treats `SMP:MARGIN:OVERFLOW`; margin key counts for all 71 cases.
- The exact wording of proposals A and B exists only in the task statement; ADR-082 Part 3 differs.
- The numeric tail fits in Section C are illustrations, not estimates with stated uncertainty.
