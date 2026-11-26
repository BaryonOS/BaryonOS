# H05-T5c slice 1: virtio-blk lane 0 re-initialisation after an S3 wake

Date: 25 November 2026 (Europe/Amsterdam), re-run 26 November 2026 for the O13 latch
carryover. Branch `claude/h05-t5c-blk-reinit-20261125`. Tested source: `20c3a5dc`
(slice 1 plus the O13 check `c9236113`, with `origin/main` 2289a271 merged; #539 and #555
included). The first run on base c61a1fe0 is superseded. Decision record: amendment H05-T5c under
ADR-H05-S3-SYSTEM-20261123 item 3 in `docs/DECISIONS.md`.

## Scope

After a validated ACPI S3 wake and the virtio-rng re-init, the kernel re-validates and
re-initialises the virtio-blk primary lane (00:05.0). It then runs a discarded
kernel READ of canary sector 0. If a check refuses, lane 0 is quarantined, every later
block request returns -24, and the system continues to the S5 gate and cold recovery.
No ring-3 block capability is issued, and `block_objects == 0` and the SUSPEND storage
gate are unchanged.

Evidence tags:
- **emulator:** native QEMU/TCG.
- **injected:** a labelled kernel-side corruption of one input of a real check.
- **physical:** none.

## Environment

- QEMU 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3), machine `pc-q35-8.2`, OVMF, TCG.
- Rust toolchain `1.100.0`, from `rust-toolchain.toml` and CI.
- Build and target directories live inside the worktree. `run_system.py` refuses an
  artifact outside the checkout:
  - `NOS_BUILD_DIR=build/h05-t5c`
  - `NOS_TARGET_DIR=CARGO_TARGET_DIR=target/h05-t5c`

## Commands and raw exit codes

Logs of the 26 November re-run on `20c3a5dc` are under `~/projects/BaryonOS-scratch/h/h05-t5c/o13/`.
Run directories are under `build/h05-t5c-o13/run/` (volume gate: `build/h05-t5c-vol/`, smoke:
`build/h05-t5c-smoke/`). Local only; nothing here is a hosted result.

| Command | Raw exit | Log |
|---|---|---|
| `cargo +1.100.0 test --locked --manifest-path drivers/virtio-blk/Cargo.toml` (5 tests) | 0 | `blk-test-1100.log` |
| `cargo +1.100.0 clippy --locked --manifest-path drivers/virtio-blk/Cargo.toml --all-targets -- -D warnings` | 0 | `blk-clippy-1100.log` |
| `cargo +1.100.0 test --locked --manifest-path crates/virtio-core/Cargo.toml` | 0 | `core-test-1100.log` |
| `cargo +1.100.0 fmt --manifest-path <virtio-blk, kernel, sdk>/Cargo.toml -- --check` | 0, 0, 0 | (console) |
| `python3 -B -m unittest tests.test_system_harness` (64 tests) | 0 | `harness.log` |
| `python3 -B -m unittest discover -s tools/tests -p test_sleep.py` (20 tests) | 0 | `test-sleep.log` |
| `python3 -B tools/test_power_system.py` | 0 (`passed=True stages=22`) | `power.log`, `build/h05-t5c-o13/power-system-gate.json` |
| `python3 -B tools/build.py --kernel-feature native-acpi-s3 && python3 -B tools/run_sleep.py --case cycles` | 0 (QEMU raw 33, `POWER:S3:PASS`) | `sleep-cycles.log` |
| `python3 -B -u tools/test_volume.py` (M6e gate) | 0 (`passed=True stages=34`) | `volume.log` |
| `python3 -B tools/build.py && python3 -B tools/run.py --smoke` | 0 (QEMU raw 33, `M1:PASS`) | `smoke.log` |
| O13 mutant: fresh lane Policy before `BLK_DONE`, `power-s3-blk-reinit` | 1 (QEMU raw 35, `SYSTEM:FAIL:S3_BLK_LATCH_CARRIED`) | `mut-done.log` |
| O13 mutant: fresh lane Policy in `blk_reinit_quarantine`, `power-s3-blk-reinit-capacity` | 1 (QEMU raw 35, `SYSTEM:FAIL:S3_BLK_LATCH_CARRIED`) | `mut-quarantine.log` |
| The capacity stage after restoring the source | 0 | `restored.log` |
| `NOS_BUILD_DIR=$PWD/build/power-system-gate python3 -B -u tools/test_power_system.py` after the QMP socket fix (head after `66e49d4c`) | 0 (`passed=True stages=22`) | `power-cidir.log` |

The capacity mutant of the first run (capacity removed from `negotiated_matches`, raw 35,
then restored, raw 0) was run on base c61a1fe0 only; the driver check it covers is
unchanged since.

Power gate stages: the 11 power cases pass, each followed by a passing cold `recover`
(QEMU raw 33). Of those 11 cases, the new blk-reinit profiles are:
- `power-s3-blk-reinit`
- `-busy`
- `-identity`
- `-capacity`
- `-no-reset-ack`

The stage durations add up to about 90 s, well within the existing 45-minute job
timeout. `native-power.yml` was not changed: the gate step runs `test_power_system.py`,
which iterates over `POWER_FEATURE`, and no stage-count pin exists in tests or
workflows.

## Native markers (from the gate run's `serial.txt`)

| Case | Tag | Re-init marker | LOST_BLOCK | IO_AFTER_WAKE | ACCOUNTED_VOLUME |
|---|---|---|---|---|---|
| blk-reinit | emulator | `VOLUME:S3:REINIT:BARS=RESTORED:STATUS=15:CAPACITY=00000000000000002048:SELFTEST=READ` | 6 | 0 | 8 |
| busy | injected (`SYSTEM:FAULT_INJECTED:S3_BLK_BUSY`) | `VOLUME:S3:REINIT:QUARANTINE:BROKER_BUSY` | 24 | 0 | 0 |
| identity | injected (`S3_BLK_IDENTITY`) | `VOLUME:S3:REINIT:QUARANTINE:REDISCOVER` | 24 | 0 | 1 |
| capacity | injected (`S3_BLK_CAPACITY`) | `VOLUME:S3:REINIT:QUARANTINE:CAPACITY` | 24 | 0 | 5 |
| no-reset-ack | injected (`S3_BLK_NO_RESET_ACK`) | `VOLUME:S3:REINIT:QUARANTINE:RESET_ACK` | 24 | 0 | 731 (polling, timing dependent) |

What the markers show:
- **LOST_BLOCK:** init's post-wake 0x40 result as a magnitude. -6 means the request
  reached the authority check; -24 means the lane is quarantined.
- **Success case only:**
  - `SUSPEND_BLK_STALE_TICKET=1`: an old-epoch publish was refused. The pre-sleep
    ticket peek also returns -1, but only because nothing is pending, so it is not
    evidence; the real evidence is the old-epoch publish check plus TICKET_FLOOR.
  - `SUSPEND_BLK_QUARANTINED=0`.
- **Every injected case:** `SUSPEND_BLK_QUARANTINED=1`.
- **BUSY injection:** it forces the refusal and bypasses the quiescence predicate, so
  the predicate itself is source-only evidence.
- **Harness "global latch" mutant:** it only proves its own text substitution; the
  kill comes from the exact-line latch pin.
- **Accounted re-init accesses:** every one is counted (ACCOUNTED_VOLUME), and
  `IO_AFTER_WAKE` stays 0 over the keyring and volume accounted totals.

## PIC / INTx finding

On the success path, the INTx line is read from config 0x3c before the self-test. The
8259 masks are read with `in al, dx` from ports 0x21 and 0xa1:

- `SUSPEND_BLK_INTX_LINE=10`
- `SUSPEND_BLK_PIC_MASTER=254`
- `SUSPEND_BLK_PIC_SLAVE=255`

The post-wake `timer_init` in `kernel/src/m2/m3.rs` masks every line except IRQ0, so
IRQ10 is masked. INTx-disable is also set at the function, and MSI/MSI-X are
re-disabled. An unmasked line would quarantine with `PIC_UNMASKED`. This path is
implemented but was not provoked natively.

## Notes required by the coordinator

- **Quarantine code:** -24 is returned for a block request on a quarantined lane.
  It is distinct from -13, -14 and every existing status code.
- **Policy continuity:**
  - The same lane Policy object is used: `start_reset` → `reset_completed(true)`.
    `Policy::new()` and reassignment are never used.
  - Pinned structurally by `tests/test_system_harness.py`
    (`test_blk_reinit_is_accounted_latched_and_keeps_the_policy`, fresh-Policy mutant).
  - Pinned by the host test `policy_epoch_ticket_floor_survives_the_reinit_reset` in
    `drivers/virtio-blk`.
  - Checked natively: TICKET_FLOOR and the old-epoch publish refusal (STALE_TICKET;
    the peek part is tautological, see above).
- **O13 latch carryover (#522, #539):** in stage 0, before any check or reset, the blk
  re-init profiles register and latch the never-granted canary sector 0 on lane 0
  (`register_latch`, `latch_scope`) and snapshot the table. At settlement the kernel
  requires the same table and `write_latched(0)`; otherwise it stops with
  `SYSTEM:FAIL:S3_BLK_LATCH_CARRIED` (raw 35). It prints `SUSPEND_BLK_LATCH_CARRIED`,
  which `run_system.py` requires as 1 in every blk case. Observed: 1 in all five blk
  cases. Both fresh-Policy mutants above fail with that reason. Test profiles only; no
  production profile registers a latch.
- **Published, uncompleted WRITE at SUSPEND:** a device lost in this state is already
  refused by the existing SUSPEND storage gate, so no path was added.
- **volume.rs:** 5 hunks against c61a1fe0 (`git diff c61a1fe0 HEAD -- kernel/src/m2/m3/volume.rs | grep -c '^@@'`):
  - the `quarantine_code` wrapper in `syscall()`;
  - three hunks holding four one-token `resumed()` → `device_lost()` swaps, at four call
    sites in three functions (`device_call` twice, `reset`, `advance_lane`);
  - one contiguous block at the end of the file.

## Not run / not claimable

- Hosted `native-power` CI runs on the PR head; its result is recorded on the PR, not here.
- Hosted run on `66e49d4c` failed in `power-s3-blk-reinit-no-reset-ack` (raw 1 after 0.5 s):
  the relative QMP socket path under `build/power-system-gate` was 109 bytes, above the
  108-byte `AF_UNIX` limit, so QEMU refused to start. `run_system.py` now uses a 16-hex run-directory
  suffix and refuses an over-long socket path before launching QEMU.
- Physical S3 was not tested.
- The `BARS=KEPT` branch was not observed, because local QEMU restores the BARs.
- No real device fault was produced, such as a BAR, capacity or identity change.
  The injections are kernel-side labels.
- There is no IOMMU, so DMA isolation is not claimable.
- Re-init is one-shot: its state is never reset. It is safe only because SUSPEND is
  granted once per boot; a multi-cycle S3 slice must reset it on each wake.
- After quarantine, a process without block authority can tell the lane state apart
  (-24 vs -6). Accepted under owner decision Q4; awareness only.
- A failed `rediscover` may leave memory decode enabled (no MMIO is issued), and the
  canary sector stays in a kernel DMA page. Both are kernel-only; logic unchanged.
- The ring-3 re-grant, remount and key-erasure gate belong to slices 2-4 and are not
  implemented.
- The O13 latch is registered after the wake and before the reset. The check proves that
  re-init and quarantine keep the same lane Policy and latch table; it does not prove that
  a latch set before SUSPEND survives the sleep itself.
