# H05-T5b — virtio-rng re-initialised after an S3 wake (23 November 2026)

**Decision:** ADR-H05-S3-SYSTEM-20261123, item 3 (devices are lost after a
wake); this slice adds an opt-in exception for one device. **Profile:** kernel
feature `test-system-s3-reinit = ["test-system-s3"]` (opt-in test profile, not
a published ABI). **Status:** verified in QEMU, not accepted; H05 stays open.
Every kernel change is gated on the new feature. Two shared changes are not:
`discover()` now calls the extracted `quiesce_interrupts` (same writes, same
order), and init reads word 624, which only this profile sets (the init ELF
is embedded in every system-profile kernel). Rebuilt and run here: the four
power-gate profiles, `test-system-recover` and `native-acpi-s3`; the other
profiles are unchanged by construction, not by a run or a byte diff.

## Probe (step 1)

The temporary probe printed the PCI configuration of the rng function
(00:06.0) as saved at discovery and as read right after the validated wake.
QEMU 10.2.1 (Debian build), machine `pc-q35-8.2`, OVMF 2025.11, single-thread
TCG:

| Offset | Saved at discovery | After the wake |
|---|---|---|
| 0x00 vendor/device | `0x10441af4` | `0x10441af4` (responds) |
| 0x04 status/command | `0x00100402` | `0x00100000` (command 0) |
| 0x10 BAR0 | `0x00000000` | `0x00000000` |
| 0x14 BAR1 | `0x80001000` | `0x00000000` |
| 0x18, 0x1c BAR2/3 | `0x00000000` | `0x00000000` |
| 0x20 BAR4 low (64-bit, prefetchable) | `0x0000800c` | `0x0000000c` |
| 0x24 BAR4 high | `0x000000c0` | `0x00000000` |

The device responds and every BAR lost its address while keeping its
read-only type bits; no different non-zero base appeared. This is the
restore branch, not a stop condition. Hosted CI pins QEMU 8.2.2, which was
not measured; the oracle therefore accepts `KEPT` and `RESTORED`.

## What changed

- `drivers/virtio-rng`: `rediscover(&Discovery)` checks the IDs, disables
  decode and bus mastering, classifies the six BAR dwords with the pure
  `classify_bars` (all unchanged = `Kept`; all addresses cleared with intact
  type bits = written back and read back equal, `Restored`; anything else is
  refused), restores memory decode without bus mastering, re-parses the
  capability regions against the saved BARs (must equal discovery exactly)
  and repeats discovery's MSI/MSI-X quiesce. No BAR is sized. `Device::status`
  reads the status register. Unit tests cover the probe values, mixtures,
  different bases, changed type bits and an absent device.
- Kernel (`keyring.rs`, `system.rs`): after a validated wake `system::poll`
  drives `keyring::reinit_poll` with tick deadlines. It requires an idle
  broker, rediscovers, resets the device and waits for the acknowledgement
  (status 0, bus mastering off), checks that the three pool pages are still
  kernel-owned, clears pending/completed/bytes, initialises (bus mastering is
  set only there; ACCESS_PLATFORM stays refused) and polls a 32-byte
  self-test whose bytes are discarded (cleared in the driver's bounce page,
never copied to the broker) and never counted. `next_ticket` is kept.
  Word 616 (the post-wake tick for init) is set only after this completes.
  Until then 0x60 still returns -14; block requests always do.
- **Served post-wake request (labelled kernel-driven fixture).** No ring-3
  process requests entropy after the wake in this profile: init holds no
  entropy capability, and the Key Service reseeds only every 64 operations.
  As B01's lease fixture does, the kernel therefore issues one request with
  the Key Service's own entropy capability through the real 0x60/0x61
  handlers. The first post-wake ticket continues the sequence; the last
  pre-sleep ticket is refused (-1) against the post-wake answer; 32 bytes are
  delivered into the last 32 bytes of the Key Service's initial heap (mapped
  at load time, unused by its binary), checked non-zero and wiped. The broker
  counter `delivered_bytes` grows by exactly 32 across this step.
- Init: with word 624 (set only by this profile) its post-wake 0x60 with the
  POWER handle reaches the authority check and expects -6; otherwise -14 as
  before. The kernel requires the exact log `[(0x60,-6),(0x40,-14)]`.
- Any failure prints `KEYRING:S3:REINIT:FAIL:<reason>` and
  `SYSTEM:FAIL:S3_REINIT`, stops the device and exits 0x11.
- `tools/run_system.py`: case `power-s3-reinit` (so `test_power_system.py`
  gains the stage); exactly one `KEYRING:S3:REINIT:BARS=(KEPT|RESTORED):STATUS=15:SELFTEST=32`
  between RESUMED and BROKERS; exact suspend evidence in order; any
  `KEYRING:S3:` line in another case is refused.

## Results (source commit `b51f04d8`, source fingerprint v3 `15c1d7ee…`)

Environment: `NOS_BUILD_DIR=<worktree>/build/h05t5b`,
`CARGO_TARGET_DIR=$HOME/.cache/baryon-h05t5b/target`,
`NOS_QEMU_MACHINE=pc-q35-8.2`, QEMU 10.2.1, single-thread TCG. The runs used
the tree that became `b51f04d8`; its recomputed fingerprint is the same.

| Command | Exit | Result |
|---|---|---|
| `python3 -B -u tools/test_power_system.py` | 0 | 10/10 stages: power (raw 0) + recover (33), power-flush-error (33) + recover, power-flush-timeout (33) + recover, power-s3 (raw 0, 5.2 s) + recover, **power-s3-reinit (raw 0, 5.2 s) + recover (33)** |
| `python3 -B tools/build.py --kernel-feature native-acpi-s3` | 0 | — |
| `python3 -B -u tools/run_sleep.py --case cycles` | 0 | raw 33, 3 cycles, `POWER:S3:PASS` |
| `python3 -B -m unittest tests.test_system_harness` | 0 | 35 tests, incl. 6 re-init oracle tests (re-run on the final tree) |
| `python3 -B -m unittest discover -s tests -p 'test_system_*.py'` | 0 | 69 tests |
| `python3 -B -m unittest discover -s tools/tests -p test_sleep.py` | 0 | 17 tests |
| `python3 -B -m unittest discover -s tests -p test_ci_workflows.py` / `'test_ci_evidence*.py'` | 0 | 28 / 38 tests |
| `cargo +1.90.0 test --locked -p virtio-rng-driver` (also `+1.100.0`) | 0 | 6 tests (re-run after rustfmt) |
| `cargo +1.90.0 clippy --locked -p virtio-rng-driver --all-targets -- -D warnings` (also `+1.100.0`) | 0 | — |
| `cargo +1.90.0 fmt --all -- --check` (also `+1.100.0`), and `--manifest-path sdk/examples/system/Cargo.toml` | 0 | — |
| `python3 -B tools/preflight.py --strict` | 0 | — |

power-s3-reinit serial (abridged, in order):

```
POWER:S3:PLAN:PM1A=0x0604:SLP_TYP=1:PM1A_STS=0x0600:FACS_VERSION=0:REAL=0x1000:PM32=0x2000510:POLICY=Edk2Version0Extended
SYSTEM:MANIFEST:PASS
POWER:S3:ENTER:CYCLE=1:SCI_EN=1
POWER:S3:RESUMED:CYCLE=1:ENTRY=PM32:PM1_STS=0x8101:WAK_STS=1:SCI_EN=RESTORED:HWSIG=SAME:FACS=DISARMED:CANARY=PASS:CR3=SAME
KEYRING:S3:REINIT:BARS=RESTORED:STATUS=15:SELFTEST=32
SYSTEM:POWER_INIT_REFUSALS:00000000000000000003
SYSTEM:SUSPEND_REFUSALS:00000000000000000005
SYSTEM:SUSPEND_GRANTED:00000000000000000001
SYSTEM:SUSPEND_TICK_AFTER_WAKE:00000000000000000001
SYSTEM:SUSPEND_REINIT_ENTROPY:00000000000000000032
SYSTEM:SUSPEND_REINIT_STALE_TICKET:00000000000000000001
SYSTEM:SUSPEND_REINIT_ENTROPY_AUTH:00000000000000000006
SYSTEM:SUSPEND_LOST_BLOCK:00000000000000000014
SYSTEM:ENTROPY_BYTES:00000000000000001056
SYSTEM:BROKERS:PASS
SYSTEM:POWER:GATE:PASS
SYSTEM:CASE:POWER_S3_REINIT:PASS
M1:PASS
POWER:S5:PM1A=0x0604:SLP_TYP=0:SCI_EN=1
POWER:S5:WRITE
```

QMP control: trace `query-status, query-status, system_wakeup, query-status`;
events `SUSPEND, WAKEUP`. The power-s3 stage of the same gate still prints
`SUSPEND_LOST_ENTROPY=14` and `SUSPEND_LOST_BLOCK=14` and no `KEYRING:S3:`
line. Raw logs remain in the ignored `build/h05t5b/` of the executing
worktree; they are not committed.

## Limits

- The served request is kernel-driven with the Key Service's capability, not
  issued by the Key Service binary; init's own post-wake entropy request is
  refused by authority (-6), not served. A ring-3-issued post-wake delivery
  needs a process that requests entropy after the wake.
- Only virtio-rng is re-initialised. virtio-blk stays lost (-14); the volume
  stays shut down after the storage gate.
- The failure paths (`SYSTEM:FAIL:S3_REINIT`: busy broker, refused BARs,
  missing acknowledgement, foreign pool page, self-test timeout, stale ticket)
  are enforced in code and the BAR classification is unit-tested, but no
  native fault fixture exercises them.
- QEMU 10.2.1 locally (BARs cleared, `RESTORED`); the CI QEMU 8.2.2 result is
  not measured. QEMU/OVMF/TCG only; no physical firmware, IOMMU or SMP.
- Not run: hosted CI on this head; clippy on the kernel binary and
  `sdk/examples/system` (no workflow runs it). AI work only, no independent
  human audit.
