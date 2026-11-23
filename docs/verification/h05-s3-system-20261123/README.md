# H05-T4 — test-profile S3 from the running system (23 November 2026)

**Decision:** ADR-H05-S3-SYSTEM-20261123. **Profile:** kernel feature
`test-system-s3` (opt-in test profile, not a published ABI). **Status:**
verified in QEMU, not accepted; H05 stays open (physical acceptance and
owner decisions OD-HW-* outstanding).

## What was exercised

Init's non-delegable POWER capability requests syscall `0x91` SUSPEND in the
running native system (key service, volume service, app, init in ring 3). The
kernel refuses it while the volume holds block authority (-8), for a launch
handle (-6), for a forged generation (-4) and for a nonzero reserved argument
(-1). It then grants one S3 cycle after the volume's SHUTDOWN, resumes through
the shared H05-T3 wake path, reprograms the PIC/PIT, audits every live process
address space and returns 0 to init. Init checks a ring-3 stack pattern across
the sleep, waits for the kernel's first post-wake timer tick, receives the
one-shot refusal (-14) and powers off through the unchanged S5 gate. A cold
recovery boot of the same image then verifies that the data survived.

## Results (source commit `e02ff67c`, source fingerprint v3 `01fc98ba…`)

| Command | Exit | Result |
|---|---|---|
| `NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B tools/test_power_system.py` | 0 | 8/8 stages pass: power (raw 0) + recover (33), power-flush-error (33) + recover (33), power-flush-timeout (33) + recover (33), **power-s3 (raw 0, 5.2 s) + recover (33)** |
| `python3 -B tools/run_sleep.py --case cycles` (native-acpi-s3) | 0 | raw 33, 3 cycles, markers unchanged after the `sleep.rs` split |
| `python3 -B tools/run_sleep.py --case corrupt-facs` | 0 | raw 33, rejection unchanged |
| `python3 -B tools/run_sleep.py --case stale-token` | 0 | raw 33, `POWER:S3:WAKE:REJECTED:TOKEN:ARMED=1:PAGE=0x5a5a0001` unchanged |
| `python3 -B -m unittest discover -s tests -p 'test_system_*.py'` | 0 | 61 tests, incl. 6 power-s3 oracle tests |
| `python3 -B -m unittest discover -s tools/tests -p test_sleep.py` | 0 | 17 tests, incl. SystemWake and import-order tests |
| `python3 -B -m unittest discover -s tests -p test_ci_workflows.py` / `test_ci_evidence*.py` | 0 | 28 / 38 tests |
| `python3 -B -m unittest tools.tests.test_gs_discipline` | 0 | GS/iretq discipline unchanged |
| `cargo +1.90.0 fmt --all -- --check`, SDK `fmt --check` | 0 | — |

power-s3 serial (abridged, in order):

```
POWER:S3:PLAN:PM1A=0x0604:SLP_TYP=1:PM1A_STS=0x0600:FACS_VERSION=0:REAL=0x1000:PM32=0x2000510:POLICY=Edk2Version0Extended
SYSTEM:MANIFEST:PASS
POWER:S3:ENTER:CYCLE=1:SCI_EN=1
POWER:S3:RESUMED:CYCLE=1:ENTRY=PM32:PM1_STS=0x8101:WAK_STS=1:SCI_EN=RESTORED:HWSIG=SAME:FACS=DISARMED:CANARY=PASS:CR3=SAME
SYSTEM:SUSPEND_REFUSALS:00000000000000000005
SYSTEM:SUSPEND_GRANTED:00000000000000000001
SYSTEM:SUSPEND_TICK_AFTER_WAKE:00000000000000000001
SYSTEM:BROKERS:PASS … SYSTEM:CLEANUP:PASS
SYSTEM:POWER:GATE:PASS
SYSTEM:CASE:POWER_S3:PASS
M1:PASS
POWER:S5:PM1A=0x0604:SLP_TYP=0:SCI_EN=1
POWER:S5:WRITE
```

QMP control (harness, private socket): trace `query-status, query-status,
system_wakeup, query-status`; events `SUSPEND, WAKEUP`; `suspended` before and
after a 0.5 s dwell, `running` after the single wake request.

Environment: QEMU 10.2.1 (Debian build) with machine `pc-q35-8.2`, OVMF
2025.11, single-thread TCG. Hosted `native-power.yml#power-system-gate`
(QEMU 8.2.2) runs the same `test_power_system.py` and therefore includes
power-s3; its result is the hosted evidence once it completes.

Raw logs remain in the ignored `build/h05t4-evidence/` of the executing
worktree; they are not committed.

## Review

An adversarial review in three lenses (kernel resume and security, oracle
integrity, regressions in accepted profiles) found no kernel or regression
defect and four minor harness defects (failure-path evidence, early profile
pin, an accelerator test that a machine check could mask). All four were
fixed in `e02ff67c`. This is AI review, not an independent human audit.

## Limits

- QEMU/OVMF/TCG only; wake only through QMP `system_wakeup`; no AML
  `_PTS`/`_WAK`, SMP, FPU state or physical firmware.
- Devices are lost across the wake. The post-wake -14 refusal of entropy and
  block requests is enforced in code and covered by review, but not exercised
  natively (no ring-3 process issues such a request after the wake).
- Only process address spaces are audited after the wake under M3; the H05-T3
  whole-kernel-root alias audit is not applicable there. The 64-frame canary
  covers 256 KiB of the lowest free RAM.
- Sleep time does not advance the tick clock. TSC-based diagnostics across
  the wake are saturating and unbounded.
- Not run: the ENOSYS answer of `0x91` in other profiles (no caller exists);
  hosted CI on this head at the time of writing.
