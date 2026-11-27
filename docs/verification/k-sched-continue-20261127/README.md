# R02-C3 single-CPU scheduler continuation: local evidence (27 November 2026)

Local evidence for ADR-K-SCHED-CONTINUE-20261126 (PR #602, issue #530, task
R02-C3): after an ordinary `int 0x80` the single-CPU M3 scheduler dispatches
the caller again within its quantum instead of selecting the next process.
Everything below is **local evidence, not hosted CI acceptance**,
physical-hardware evidence or a product-latency claim. No criterion or
progress point changes.

The evidence was collected on three heads of branch
`claude/k-sched-continue-20261126`. Each later head only merges `main` into
the earlier one; the C3 scheduler hunks in `kernel/src/m2/m3.rs` are the
same on all three.

| Evidence | Head | Base `main` | Host |
|---|---|---|---|
| Window-session A/B | `ddbb7c13` | `be1fd9ac` | local KVM |
| Native verification list (45 steps) | `0623ddf1` | `20c7e18a` | local QEMU TCG |
| `JOB_IRQS` re-measure, `qsim-k07` re-run, supervisor gates, host tests | `478b4ba3` | `7c32e6c7` | local QEMU TCG |

Commits after `478b4ba3` up to `6a74ad67` add documentation and merge
`main` `3de6c87e` (#606: `tools/merge_hold.py`, its tests and CI
documentation only); none of them changes kernel, userspace or gate code.
The again amendment that follows (`8b580497`, `d71dc04f`) has its own
evidence in the last section.

## Window-session A/B (D3 kill criterion)

- A: `main` `be1fd9ac9d96952dbd7ab7b21139800bb5151b9c`. B:
  `ddbb7c134df0e11aff961c893f488a63fe5f358b` (that `main` plus C3). Both
  worktrees clean ([`meta.txt`](meta.txt)). Profile
  `native-window-session-probe`; kernel ELF A `3f46542a…`, B `51079a87…`.
- [`ab-k.sh`](ab-k.sh): one build per side, then five window sessions per side
  with `tools/run_window_session.py --clipboard --timeout 1800`, SeaBIOS ROMs,
  `NOS_QEMU_ACCEL=kvm`, interleaved A1 B1 B2 A2 A3 B3 B4 A4 A5 B5; then one
  high-contrast session per side with the CI flags `--clipboard --workspace`,
  order A1 B1 B2 A2. It is derived from
  [`../r02-compositor-turn-ab-20261126/ab5.sh`](../r02-compositor-turn-ab-20261126/ab5.sh)
  and `ab6.sh`.
- [`summary.txt`](summary.txt) is the output of
  [`../r02-compositor-turn-ab-20261126/analyse.py`](../r02-compositor-turn-ab-20261126/analyse.py)
  (exit 0) on the raw reports, whose hashes are in
  [`report-sha256.txt`](report-sha256.txt). The raw reports and logs stay on
  the local host.
- The host was shared and not quiet: the load average was 3.2–6.7 during the
  runs ([`meta.txt`](meta.txt), [`meta-ws.txt`](meta-ws.txt)). Interleaving
  spreads load drift over both sides.

### Results

All ten clipboard sessions and all four workspace sessions ended with raw
QEMU exit 33 and `passed` true.

| Metric | A median (range) | B median (range) | Change |
|---|---|---|---|
| Whole session duration | 17.02 s (17.00–17.49) | 16.50 s (15.90–16.61) | −3.1 %, ranges disjoint |
| QEMU host CPU time | 11.79 s (11.77–12.00) | 10.78 s (10.10–10.93) | −8.6 %, ranges disjoint |

- Per phase, no median of `last_input_to_verified_app_image_s` moves by more
  than its own A or B range. This host-side metric sits on a coarse
  screendump retry grid.
- All 31 clipboard-session phase frames are identical across the ten runs,
  and the oracle file hashes form one set.
- Workspace loop: the four high-contrast frames are identical in all four
  runs. Five of 66 workspace phases have differing frame hashes
  (`workspace-work`, `workspace-search`, `workspace-window-menu`,
  `lock-veld-overlay-open`, `lock-unlocked`). Each of them also differs
  between A1 and A2, so this is the taskbar transition timing effect already
  recorded in
  [`../r02-compositor-turn-ab-20261126/README.md`](../r02-compositor-turn-ab-20261126/README.md),
  not a change caused by C3.

### D3 reading

The literal per-phase reading of D3 is not met. The project owner decided on
27 November 2026 at about 12:03 UTC that the disjoint whole-session and CPU
gains satisfy D3, and that hosted native job durations on #602 must still not
regress against recent `main`. That hosted comparison is recorded on the pull
request, not here.

## Native verification list on `0623ddf1`

[`native-0623ddf1.txt`](native-0623ddf1.txt) lists 45 steps (QEMU TCG as in
CI, `NOS_QEMU_MACHINE=pc-q35-8.2`, separate build and target directories per
step). It covers the M5, M6c, IPC-wait, FPU, memory, qsim, SDK, window
session (scale 1 and 2), NAS, Materiaal, keyring lock, legacy SMP (all 16
`tools/test_smp.py` profiles), K05 supervisor and task-grant gates, plus
`tools/test_milestones.py --through m4` with every `test-m3-*` case,
including `test-m3-syscall-spin`.

- 44 steps exit 0. `qsim-k07-gate` exited 1 without running QEMU: the runner
  refuses a reused build directory ("resource-pressure output already exists;
  use a fresh NOS_BUILD_DIR"). It passes with a fresh directory on
  `478b4ba3` (next section).
- The fixtures that guard `syscall_calls < 2_000_000` (FPU, display, memory)
  pass under the higher system-call rate of a continued caller.

## Re-measure and re-runs on `478b4ba3`

`main` `7c32e6c7` (with #598, the L01 native AI job hardening) merged
cleanly. Each gate used fresh `NOS_BUILD_DIR`, `NOS_TARGET_DIR` and
`CARGO_TARGET_DIR` directories and the default QEMU machine `pc-q35-10.0`
(CI pins `pc-q35-8.2`); see
[`native-478b4ba3.txt`](native-478b4ba3.txt).

- `native-ai-job` (the `native-system.yml` step: `tools/build.py
  --kernel-feature native-ai-job`, then `tools/run_ai_job.py`): five passes,
  each with raw QEMU exit 33 and `AIJOB:PASS`. Timer IRQs that landed on the
  job per run (two runs per pass): 41/34, 35/35, 35/35, 28/41, 37/37, so
  28–41 locally. This lies inside the local range of 20–81 that the comments
  in `kernel/src/m2/m3/ai_job.rs` and `tools/run_ai_job.py` record, so
  `MIN_JOB_TIMER_IRQS = 4` and both comments stay unchanged. The hosted range
  of 8–65 is re-read from the hosted `native-system` job on #602.
- `qsim-k07-gate` (`tools/test_resource_pressure.py`): raw QEMU exit 33,
  `QSIM:PASS`, `K07 resource-pressure accepted=True`.
- `tools/test_supervisor.py` without and with `--smp`: both report
  `K05 gate passed=True` over six stages, each stage with raw QEMU exit 33.
  Hosted CI runs only the variant without `--smp`.
- Host suites touched by the pull request: `tests/test_capture_gate.py`,
  `tools/tests/test_smp_host.py` (23 tests, `PYTHONPATH=tools`),
  `userspace/tests/test_probe.py` (13 tests on a fresh `milestone-m3` probe
  ELF) and `tools/tests/test_run_ai_job.py` all pass.

## Red control

The new case `test-m3-syscall-spin` requires `M3:SYSCALL_SPIN:PASS` once
before `KS-04` and once before `KS-07`, in that order. On a tree with the
fixture but without the scheduler change, the kernel stops at the fixture's
assertion with raw QEMU exit 35. With the scheduler change it passes with raw
exit 33 and both rounds report the spin marker. `tests/test_capture_gate.py`
pins the ordered marker tuple and rejects a transcript that has only the
first-round verdict.


## Network regression and the again amendment (27 November 2026)

Hosted CI on `6a74ad67` passed the job-level D3 rule, but the networked
system guests ran 27–52 % longer than in same-day non-C3 controls (the
numbers are on #602 and in the ADR). The project owner kept #602 out of the
merge queue until a structural fix is verified, without longer timeouts or a
relaxed D3. Root cause and amendment: see "Network regression and
amendment" in ADR-K-SCHED-CONTINUE-20261126. A system call that returns -8
(Again) now selects like a yield instead of continuing.

Three trees, all local; QEMU TCG with `NOS_QEMU_MACHINE=pc-q35-8.2` unless
stated; fresh build and target directories inside each checkout (the net
runners refuse directories outside it):

| Side | Tree |
|---|---|
| A | `main` `af7415df6aa64f0f293c5e88b59970c43a2f3ff6` (newer than B's base `3de6c87e`; the difference adds power and lock test tooling only and touches no kernel, userspace, SDK or crate code) |
| B | C3 `6a74ad67d5f56ba4e256ae792f9c6e670ac52f4f` |
| C | B plus the again amendment, committed unchanged as `8b580497` |

C ran as an uncommitted tree on B, so the scripts and summaries label it
with B's head `6a74ad67`. The SHA-256 of its `git diff -- kernel` against
B, `52e93f8edc16b410…`, was recorded with the pre-registered criteria
(#602 issuecomment-7219897682) and equals that of
`git diff 6a74ad67 8b580497 -- kernel`.

### `tools/test_net_system.py`

[`net-ab-run.sh`](net-ab-run.sh): one discarded warm-up per side, then three
interleaved rounds (A B C, C B A, B C A). Every run ended with raw QEMU exit
33 and `passed` true; [`net-ab-summary.txt`](net-ab-summary.txt) holds the
raw lines.

| Guest | A | B | C |
|---|---|---|---|
| `normal` | 54.8–56.8 s | 66.9–68.4 s | 42.9–43.5 s |
| `recover` | 42.2–42.8 s | 65.4–65.9 s | 42.1–42.2 s |

### Net-link gate (H04-T3)

[`net-link-ab-run.sh`](net-link-ab-run.sh): per side `tools/build.py
--kernel-feature test-system-net-link`, `tools/run_net_link.py --case normal`
and `--case nostatus`, then the `test-system-net-link-storm` build and
`--case storm`; one discarded warm-up per side, then two rounds (A B C,
C B A). Raw lines: [`net-link-ab-summary.txt`](net-link-ab-summary.txt);
each line lists `normal`, `nostatus` and `storm` in that order (the runner
labels every case `SYSTEM normal`).

Guest duration per run; every run ended with raw QEMU exit 33.

| Case | A | B | C |
|---|---|---|---|
| `normal` | 54.3–55.9 s | 66.9–67.8 s | 43.3–44.6 s |
| `nostatus` | 54.5–54.8 s | 65.1–65.6 s | 43.0–43.1 s |
| `storm` | 45.8–46.4 s | 68.4–71.2 s | 46.4–46.8 s |

C's `storm` range lies up to 0.4 s above A's, within the pre-registered
limit of 1.05 × A's maximum.

### Window session (D3 kill criterion, re-checked)

[`win-abc-run.sh`](win-abc-run.sh): local KVM, profile
`native-window-session-probe`, five `--clipboard` sessions per side in the
orders A B C, C B A, B C A, A C B, C A B, with SeaBIOS ROMs as in
[`ab-k.sh`](ab-k.sh). Summary: [`win-abc-summary.txt`](win-abc-summary.txt).

All 15 sessions ended with raw QEMU exit 33 and `passed` true. None of the
31 clipboard phases has more than one frame hash within a side.

| Metric | A median (range) | B median (range) | C median (range) |
|---|---|---|---|
| Whole session duration | 16.77 s (16.69–17.52) | 16.13 s (15.75–16.33) | 15.56 s (15.35–15.72) |
| QEMU host CPU time | 11.52 s (11.26–12.07) | 10.16 s (9.87–10.55) | 10.39 s (10.26–10.52) |

C keeps the window-session gain: both of its ranges lie below A's and do
not overlap them. C's CPU median lies 0.23 s above B's; the B and C
ranges overlap.

### Again gate, red control and native list on `d71dc04f`

QEMU TCG with `NOS_QEMU_MACHINE=pc-q35-8.2`, fresh build and target
directories inside the checkout. The list, the first `native-ai-job` passes
and the network-service cases ran on `d71dc04f`, before `main` `917d3406`
was merged; after the merge only the spin gate, its red control, the
`native-ai-job` passes and the host tests ran locally.

- `test-m3-syscall-spin` (`tools/build.py --kernel-feature
  test-m3-syscall-spin`, then the gate case
  `syscall-spin-continues-within-quantum`): raw QEMU exit 33 and
  `M3:SYSCALL_SPIN:PASS` in both rounds.
- Red control: head `503148ff` (after the merge of `main` `917d3406`) with
  only the `-8` condition removed from the single-CPU candidate stops at
  the poller assertion (`M2:PANIC_LINE:0000002713`, which is
  `kernel/src/m2/m3.rs:2713` in that tree, `M1:FAIL:KERNEL_PANIC`) with raw
  QEMU exit 35.
- [`native-d71dc04f.txt`](native-d71dc04f.txt): the 45-step list of the
  section on `0623ddf1`, with fresh build directories. All 45 steps exit 0,
  including `qsim-k07-gate`, `tools/test_smp.py` (16 profiles; the spin row
  reports both markers) and `tools/test_milestones.py --through m4`.
- `native-ai-job`, five passes with fresh directories: raw QEMU exit 33 and
  `AIJOB:PASS` each; timer IRQs that landed on the job per run 43/40, 41/46,
  50/46, 36/41, 35/40, inside the documented local range of 20–81.
- After the merge of `main` `917d3406` (which brings `c6e8cb5c`, a change
  to the AI-job test), `native-ai-job` again five passes on `503148ff`:
  raw QEMU exit 33 and `AIJOB:PASS` each; job timer IRQs per run 34/41,
  33/34, 34/38, 35/34, 33/31. The spin gate on `503148ff` also passes
  (raw 33, both markers).
- `net-broker-quality.yml#native-network-service` on `d71dc04f`: the six
  `tools/run_network_service.py` cases (`normal`, `client-exit`, `revoke`,
  `helper`, `device-reset`, `exhaustion`) each pass with raw QEMU exit 33.

### Limits

- Local TCG timings are not performance evidence on their own, and local
  KVM is not hosted CI. The hosted rule and the step-level guest comparison
  on the new head are recorded on #602.
- The host was shared with other specialists' runs; interleaving spreads
  load drift over the sides.


## Not run

- Hosted CI on the final head, including the hosted `native-window-session`
  durations and the hosted `JOB_IRQS` range.
- A cold-cache TCG timing of the full verification list.
- Locally against this branch, the gates that `main` `917d3406` brought
  in: the K5 per-disk volume fixtures (#623), the loader `TPM_UNENROLLED`
  halt and the `tools/run_window_session.py` change; hosted CI runs them.
- Physical hardware.
