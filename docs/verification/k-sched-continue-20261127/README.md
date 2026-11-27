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

Commits after `478b4ba3` on the pull request add documentation and merge
`main` `3de6c87e` (#606: `tools/merge_hold.py`, its tests and CI
documentation only); none of them changes kernel, userspace or gate code.

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

## Not run

- Hosted CI on the final head, including the hosted `native-window-session`
  durations and the hosted `JOB_IRQS` range.
- A cold-cache TCG timing of the full verification list.
- Physical hardware.
