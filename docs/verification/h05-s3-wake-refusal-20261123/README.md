# H05-T5a — post-wake device-loss refusal exercised natively (23 November 2026)

**Decision:** ADR-H05-S3-SYSTEM-20261123, item 3. **Profile:** kernel feature
`test-system-s3` (opt-in test profile, not a published ABI). **Status:**
verified in QEMU, not accepted; H05 stays open. This closes the H05-T4 limit
"the post-wake -14 refusal of entropy and block requests is … not exercised
natively" for the refusal path, with the bound stated under *Limits*.

## What changed

After the granted suspend, the post-wake timer tick and the one-shot SUSPEND
refusal, init issues one well-formed entropy request (`0x60`, length 1) and
one well-formed block read (`0x40`, sector 0, offset 0, length 1). Each
returns -14. Init then powers off through the unchanged S5 gate.

**Why init presents its POWER handle.** Init holds no entropy or block
capability (the manifest grants entropy to the Key Service only, and
`APP_AUTHORITY`/init endpoint checks prove that no endpoint hides one). The
only block object belongs to the volume service, which closes it on
SHUTDOWN; the storage gate requires `block_objects == 0` before both the
suspend grant and S5. An authorised block request therefore cannot exist
after the wake without reopening storage. Before H05-T5a the device-loss
check sat after the authority lookup and was unreachable from ring 3. In
`test-system-s3` it now runs directly after the argument-shape checks and
before the authority lookup, the `denied` counter, the policy, any copy,
ring, doorbell or device access. Before the wake the same request with a
non-entropy/non-block handle is refused at the authority check (-6, as the
app's existing wrong-type probes show); after the wake it is refused with
-14. The refusal grants nothing; it reveals only that a resume happened,
which init itself requested.

**Kernel evidence.** `keyring::syscall` (0x60) and `volume::syscall`
(0x40..=0x43) record every result after the wake, tagged by syscall number,
in the bounded `Suspend` log (4 entries). `finish` requires exactly
`[(0x60, -14), (0x40, -14)]` from init and no post-wake entropy/block call
from any other process (fail closed on overflow), then prints the logged
magnitudes. `tools/run_system.py` requires `SUSPEND_LOST_ENTROPY=14` and
`SUSPEND_LOST_BLOCK=14` after `SUSPEND_TICK_AFTER_WAKE`, in order, and
refuses any `SYSTEM:SUSPEND*` line in the `power` case (the flush-fault
and normal/recover oracles do not check for it).

## Results (source commit `026269b6`, source fingerprint v3 `b2c44a16…`)

The runs used the uncommitted tree that became `026269b6`; the v3 source
fingerprint recomputed on the clean commit is the same `b2c44a16…`.

| Command | Exit | Result |
|---|---|---|
| `python3 -B -u tools/test_power_system.py` | 0 | 8/8 stages: power (raw 0) + recover (33), power-flush-error (33) + recover (33), power-flush-timeout (33) + recover (33), **power-s3 (raw 0, 5.2 s) + recover (33)** |
| `python3 -B tools/build.py --kernel-feature native-acpi-s3` | 0 | — |
| `python3 -B -u tools/run_sleep.py --case cycles` | 0 | raw 33, 3 cycles, `POWER:S3:PASS` |
| `python3 -B -m unittest discover -s tests -p test_system_harness.py` | 0 | 29 tests |
| `python3 -B -m unittest discover -s tests -p 'test_system_*.py'` | 0 | 63 tests, incl. 2 new oracle tests |
| `python3 -B -m unittest discover -s tools/tests -p test_sleep.py` | 0 | 17 tests |
| `python3 -B tools/preflight.py --strict` | 0 | — |
| `python3 -B -m unittest discover -s tests -p test_ci_workflows.py` / `'test_ci_evidence*.py'` | 0 | 28 / 38 tests |
| `python3 -B -m unittest tools.tests.test_gs_discipline` | 0 | 8 tests |
| `cargo +1.90.0 fmt --all -- --check`, same with `+1.100.0`, and `--manifest-path sdk/examples/system/Cargo.toml` | 0 | — |

Environment for the native runs: `NOS_BUILD_DIR=<worktree>/build/h05t5a`
(the FPU runner refuses a build directory outside the checkout),
`CARGO_TARGET_DIR=$HOME/.cache/baryon-h05t5a/target`,
`NOS_QEMU_MACHINE=pc-q35-8.2`, QEMU 10.2.1 (Debian build), single-thread
TCG. Hosted CI pins QEMU 8.2.2; its `native-power.yml#power-system-gate`
runs the same `tools/test_power_system.py`.

power-s3 serial (abridged, in order):

```
POWER:S3:PLAN:PM1A=0x0604:SLP_TYP=1:PM1A_STS=0x0600:FACS_VERSION=0:REAL=0x1000:PM32=0x2000510:POLICY=Edk2Version0Extended
SYSTEM:MANIFEST:PASS
POWER:S3:ENTER:CYCLE=1:SCI_EN=1
POWER:S3:RESUMED:CYCLE=1:ENTRY=PM32:PM1_STS=0x8101:WAK_STS=1:SCI_EN=RESTORED:HWSIG=SAME:FACS=DISARMED:CANARY=PASS:CR3=SAME
SYSTEM:POWER_INIT_REFUSALS:00000000000000000003
SYSTEM:SUSPEND_REFUSALS:00000000000000000005
SYSTEM:SUSPEND_GRANTED:00000000000000000001
SYSTEM:SUSPEND_TICK_AFTER_WAKE:00000000000000000001
SYSTEM:SUSPEND_LOST_ENTROPY:00000000000000000014
SYSTEM:SUSPEND_LOST_BLOCK:00000000000000000014
SYSTEM:BROKERS:PASS
SYSTEM:POWER:GATE:PASS
SYSTEM:CASE:POWER_S3:PASS
M1:PASS
POWER:S5:PM1A=0x0604:SLP_TYP=0:SCI_EN=1
POWER:S5:WRITE
```

The `power` case serial contains no `SYSTEM:SUSPEND` line, and its refusal
counters (`POWER_INIT_REFUSALS=3`, `POWER_APP_REFUSALS=2`) are unchanged.
Other profiles are unchanged by construction (every kernel change is gated on
`test-system-s3`; init's new calls run only when word 592 is set), not by a
byte diff: the larger init ELF is embedded in every system-profile kernel,
and only the power-gate and `native-acpi-s3` profiles were rebuilt and run.
Raw logs remain in the ignored `build/h05t5a/` of the executing worktree;
they are not committed.

## Limits

- An *authorised* post-wake entropy or block request is still not exercised:
  no process holds block authority after the storage gate, and the Key
  Service, which keeps its entropy capability, issues no entropy request
  after the wake. The refusal path now precedes authority, so such a
  request would take the same -14 branch, but that is shown by code order
  and review, not by a run.
- `0x41` (block wait) and `0x61` (entropy wait) are not issued after the
  wake; `0x61` has no device-loss branch (the broker is idle by the gate).
- QEMU/OVMF/TCG only, QEMU 10.2.1 locally; device re-initialisation after a
  wake remains a separate slice. AI review only, no independent human audit.
- Not run: hosted CI on this head; clippy (no workflow runs clippy on the
  kernel binary or `sdk/examples/system`).
- Any non-init process that issues `0x60` or `0x40`–`0x43` after the wake
  makes `finish` fail the suspend evidence (`other` must stay 0). This is
  deliberate fail-closed behaviour in a test profile; it is not a product
  device-loss policy.

## Independent review

A fresh AI reviewer read the candidate diff `02ef433f..5e8c800d` against
ADR-H05-S3-SYSTEM-20261123: no blocking or major finding. The cfg confinement,
the absence of device, pointer or ring access before the refusal, and the log
bounds were confirmed by reading. Minor finding 1 (a stray `SYSTEM:SUSPEND*`
line passed in the refused-power, normal and cold cases) is fixed: the check now
runs for every case except `power-s3` (`refuse_stray_suspend` in
`tools/run_system.py`), with a host test that fails when the check is disabled.
Minor findings 2 and 3 are the limits recorded above. This is AI review, not an
independent human audit.
