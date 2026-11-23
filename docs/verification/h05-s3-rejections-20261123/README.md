# H05-T6b S3 plan rejections and wake fixtures: local evidence (23 November 2026)

**Scope.** Three review follow-ups from `docs/verification/h05-hardening-20261123/`,
implemented on branch `claude/h05-t6b-20261123` from base `7c5201b6`. Code commits:
`34afae5a` (items A and B) and `ef355dad` (item C). Local QEMU 10.2.1 with
`-M pc-q35-8.2`, OVMF, TCG. CI pins QEMU 8.2.2, which this run did not use. No
hosted CI, no physical hardware and no human audit. This is a builder's local
record, not acceptance of any criterion.

## Changes

**A. Exit code for refused plans (`kernel/src/m2/sleep.rs`).** In `native-acpi-s3`,
`sleep::run` now stops the VM with isa-debug value `0x13` (raw 39) when `resolve()`
returns `None` or after `POWER:S3:REJECTED:FacsStore:DISARMED`. Before, both returned
to the M2 exit `0x10` (raw 33, the pass code). `run` now returns only after
`POWER:S3:PASS`.

Before 0x13 was chosen, `kernel/src`, `boot/loader/src`, `tools/`, `tests/`,
`.github/`, `userspace/` and `sdk/` were searched for isa-debug-exit values. The
only values in use were `0x10` and `0x11` (kernel and loader), `0x12` (the
rejected-wake exit) and the S5 poweroff (raw 0). No harness pinned raw 39.

*Feature closure.* Only `native-acpi-s3` (with its `test-acpi-s3-*` children) and
`test-system-s3` enable `acpi-s3-core`. `sleep::run` is compiled only for
`native-acpi-s3`. In `test-system-s3`, `stash` stores a `None` plan, `ready()` is
then false, and syscall 0x91 returns -14 (`system.rs`). That profile's exit is
unchanged.

*Profiles whose exit changes:*
- `test-acpi-s3-corrupt-facs`: raw 33 → 39 (observed).
- `native-acpi-s3`: only on its rejection paths (`UNAVAILABLE`, `UNREADABLE`,
  `FacsNotNvs`, `FacsExtent`, a `wake_plan` error, `Pm1bUnsupported`,
  `FacsStore:DISARMED`): raw 33 → 39. This holds by construction. Natively, only
  `InvalidSignature` (corrupt-facs) and `FacsStore:DISARMED` (facs-end) were
  exercised. The pass path stays raw 33.
- New profiles: `test-acpi-s3-unarmed-wake` (raw 37) and `test-acpi-s3-facs-end`
  (raw 39).

**B. Fixtures (`kernel/Cargo.toml`, `tools/build.py`, `tools/run.py`,
`tools/run_sleep.py`).**
1. `test-acpi-s3-unarmed-wake` (case `unarmed-wake`). `cycle` writes an all-ones
   `SAVED`, leaves `ARMED` at 0 and sets the page token to the cycle (1). Expected
   tail: PLAN, `POWER:S3:ENTER:CYCLE=1:SCI_EN=1`,
   `POWER:S3:WAKE:REJECTED:TOKEN:ARMED=0:PAGE=0x1`; one QMP SUSPEND/WAKEUP pair; raw 37.
2. `test-acpi-s3-facs-end` (case `facs-end`). In place of the wake stores, `cycle`
   writes one byte at `facs+length` through `write_firmware`. Any result other than
   `Err(Permission)` fails the VM with `S3_FACS_END_NOT_REFUSED`. The kernel then
   disarms, as for any store failure. Expected tail: PLAN,
   `POWER:S3:STORE:FACS_END:REFUSED:Permission`,
   `POWER:S3:REJECTED:FacsStore:DISARMED`; no sleep; raw 39. This exit was chosen
   because no PM1 write follows. The fixture also exercises item A's DISARMED branch.

**C. CR3 from the kernel image (`kernel/src/m2/sleep.rs`).** The 32-bit stage now
loads CR3 from `NOS_S3_CR3`, a u32 static in the identity-mapped kernel image that
`prepare` writes once. It no longer reads offset 8 of the low wake page, which is
now reserved (zero). The `root < 4 GiB` check is kept. C is a separate commit.

## Commands (raw exit codes)

Run sequentially in the worktree with `NOS_BUILD_DIR=$PWD/build/h05t6b`,
`CARGO_TARGET_DIR=$PWD/target/h05t6b` and `NOS_QEMU_MACHINE=pc-q35-8.2`. Logs are in
the ignored `build/h05t6b/logs/{ab,c}/`.

| Command | Exit on `34afae5a` (A+B) | Exit on `ef355dad` (A+B+C) |
|---|---|---|
| `build.py --kernel-feature native-acpi-s3` + `run_sleep.py --case cycles` | 0 / 0 (raw 33) | 0 / 0 (raw 33) |
| `... test-acpi-s3-stale-token` + `--case stale-token` | 0 / 0 (raw 37) | 0 / 0 (raw 37) |
| `... test-acpi-s3-corrupt-facs` + `--case corrupt-facs` | 0 / 0 (raw 39) | 0 / 0 (raw 39) |
| `... test-acpi-s3-unarmed-wake` + `--case unarmed-wake` | 0 / 0 (raw 37) | 0 / 0 (raw 37) |
| `... test-acpi-s3-facs-end` + `--case facs-end` | 0 / 0 (raw 39) | 0 / 0 (raw 39) |
| `python3 -B -u tools/test_power_system.py` (8 stages; power-s3 raw 0) | not run | 0 |
| `unittest discover -s tools/tests -p 'test_sleep.py'` (20 tests) | 0 | 0 |
| `unittest discover -s tests -p 'test_ci_*.py'` (66 tests) | 0 | 0 |
| `unittest discover -s tools/tests -p 'test_*.py'` (670 tests) | 1, see below | not rerun (no tool change in C) |
| `cargo +1.90.0 test --locked` / `clippy --all-targets -- -D warnings` (`--manifest-path drivers/acpi-power/Cargo.toml`) | n/a | 0 / 0 |
| `cargo +1.90.0 fmt --all -- --check` | 0 | 0 |
| `python3 -m compileall -q tools` | n/a | 0 |
| `python3 -B tools/preflight.py --strict` | n/a | 0 |

**Source binding.** Every native run used the working tree before it was
committed, so each report's `base_commit` names the parent commit. The tree was
then committed unchanged: `source_hash()` at `ef355dad` is `0988db5f…d5f`, and it
equals `source_sha256` in all five `build/h05t6b/sleep-*-report.json` files. The
C run overwrote the A+B reports; for the A+B column only `logs/ab/` remains.

In the full `tools/tests` run, six tests in `test_agent_isolation` and
`test_agent_pipeline` errored because the session exports `GIT_EDITOR=true`. With
`env -u GIT_EDITOR`, both modules pass (68 tests, OK). Neither module touches S3.

**Mutation check (uncommitted).** In `s3_wake`, `SAVED` was read and CR4/CR0 were
restored before the `ARMED` check. The `unarmed-wake` fixture then failed. The guest
printed ENTER and no rejection marker, and the run ended at the 180 s timeout
(host kill, raw -9). `run_sleep.py` exited 1. The mutation was reverted with
`git checkout`. The fixture therefore detects a restore before the authority check,
which the stale-token fixture could not.

**Mutation check 2 (uncommitted).** In `paging::write_firmware`, the
`store_within_extent` refusal was disabled. The `facs-end` fixture then failed:
`M1:FAIL:S3_FACS_END_NOT_REFUSED` and raw 35, not 39, and `run_sleep.py` exited 1.
The mutation was reverted with `git checkout`. The `Permission` the fixture
observes therefore comes from the extent check, not from a later
`firmware_span` refusal.

## Limits

- Not exercised natively: `UNAVAILABLE`, `UNREADABLE`, `FacsNotNvs`, `FacsExtent`,
  a `wake_plan` error and `Pm1bUnsupported`. Their raw 39 follows from the single
  `resolve()` → `None` exit, not from a run.
- `facs-end` checks only the end boundary. `store_within_extent` host tests cover
  the other bounds.
- C is verified only by the passing cycle and system-profile wake. No fixture
  corrupts the wake page's offset 8.
- Hosted CI (`.github/workflows/native-power.yml`) runs `cycles`, `corrupt-facs`
  and `stale-token`. It does not run `unarmed-wake` or `facs-end`. Adding them
  needs an integration request, because `.github/**` is outside this slice.
- QEMU 10.2.1 locally, not the CI QEMU 8.2.2; TCG only; no physical firmware.
