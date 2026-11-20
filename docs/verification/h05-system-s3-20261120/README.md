# H05-T4 — ACPI S3 requested from ring 3 through init's POWER capability

**Date:** 20 November 2026. **Evidence level:** native QEMU/TCG
(`pc-q35-8.2`, QEMU 8.2.2+ds-0ubuntu1.18, Ubuntu `OVMF_CODE_4M.fd`), not
physical hardware. **Profiles:** `native-system-s3`,
`test-system-s3-stale-token`, `test-system-s3-not-durable` (single CPU, the
base `native-system-power` services). **Score:** no criterion is accepted by
this builder step; H05 stays open (its acceptance requires measured,
recoverable sleep/wake, shutdown and thermal/battery behaviour on supported
hardware). The lead owns the ADR, the progress assessment and
`native-power.yml`.

Source binding: base `3ef2a4d` (branch `h05-t4-s3-system`), source
fingerprint v3
`851a982da1467480b7cb5c9b08b4b54b472ee7ff60bc7144f6d92306fbce1e6e` for every
native stage below. The runs executed on the working tree that was then
committed unchanged on `h05-t4-s3-system`, so the reports' `source_commit` and
`base_commit` name the parent `3ef2a4d`. [native-report.json](native-report.json)
holds, per stage, the kernel feature, raw QEMU exit, duration, QEMU command,
firmware/ESP/kernel/loader SHA-256, disk SHA-256 before and after, the
kernel's numeric evidence, the QMP events, trace and dwell observations, the
volume oracle summary and the control/POWER/SLEEP serial lines. It replaces
the absolute checkout prefix with `<CHECKOUT>` and keeps only those serial
lines; nothing else was changed.

## What was demonstrated

Init, a ring-3 process, requests one ACPI S3 cycle with syscall `0x91
SUSPEND(power)`. The kernel authorises it with the existing non-delegable
POWER capability and the unchanged storage gate, then suspends to RAM inside
the system call while the init, Key Service and Volume Service processes
stay live; after the harness wakes the guest, init continues and powers off
through S5:

1. Refusal order before any I/O: nonzero arguments (-1), `check_power`
   (-4/-5/-6/-10), no validated S3 plan (-14), storage gate (-8 retry, -16
   never this boot). Init observed -8 (volume still holds BLOCK authority),
   then -6 (its launch handle) and -4 (forged generation); the app observed
   -4 or -6 for init's raw handle number and -6 for its registry endpoint.
   The kernel logs these codes itself (`SLEEP_INIT_REFUSALS=3`,
   `SLEEP_APP_REFUSALS=2`); ring-3 words are not trusted for them.
2. The transition reuses the H05-T3 wake path (`sleep::suspend_once`, now
   shared through the internal `acpi-s3` feature): FACS armed with a fresh
   token, own 32-bit entry, token check, FACS disarm and read-back, canary,
   FACS signature and CR3 checks. Before arming, the kernel requires the
   kernel CR3 to be live and CR4.FSGSBASE/OSXSAVE to be clear.
3. After the validated wake the kernel latches `devices_lost`, resets the
   debug registers, reprograms the PIC and PIT and reads the IMRs back
   (`0xfe`/`0xff`), drops the TSC tick-gap baseline, then waits for two
   real IRQ0s (`SLEEP_TIMER_IRQS=2`) before any process runs again, and
   audits every live process (mappings, owners, W^X, pattern word).
4. Device submissions after the wake (keyring `0x60`, volume
   `0x40`/`0x42`/`0x43`) return -14 after their argument and authority
   checks and before any broker or device state; `offline` is not reused,
   so the power gate keeps its -16 meaning. While latched, `system::poll`
   requires both brokers idle. No device I/O was published after the wake
   (`SLEEP_DEVICE_IO_AFTER_WAKE=0`, `SLEEP_DEVICE_REFUSALS=0`).
5. S5 after S3: the gate is re-checked on the final broker state and the
   kernel writes PM1 (`SCI_EN=1`, restored by the wake path).

## Cases (`python3 -B -u tools/test_power_system.py`, one frozen source)

| Case | Kernel feature | Oracle | Result |
|---|---|---|---|
| `power` (unchanged H05) | `native-system-power` | gated S5 tail, raw exit 0 | pass, raw 0, 10.3 s |
| `power-flush-error` (unchanged) | `test-system-power-flush-error` | no PM1 write, -16, raw 33 | pass, raw 33, 10.5 s |
| `power-flush-timeout` (unchanged) | `test-system-power-flush-timeout` | no PM1 write, -16, raw 33 | pass, raw 33, 15.9 s |
| `s3` | `native-system-s3` | after `SYSTEM:MANIFEST:PASS`, before `SYSTEM:BROKERS:PASS`: PLAN, `ENTER:CYCLE=1` directly followed by `RESUMED:CYCLE=1:ENTRY=PM32:…:WAK_STS=1:…:CANARY=PASS:CR3=SAME`, `SYSTEM:SLEEP:TIMER:WAIT`, `SYSTEM:SLEEP:RESUMED:PASS`; exact SLEEP/POWER counters; QMP `suspended` with a 0.5 s serial-silent dwell; harness-only `system_wakeup`; QMP events `SUSPEND`, `WAKEUP`, then exactly one `SHUTDOWN`; one EBS; S5 tail; raw exit 0; disk oracle | pass, raw 0, 11.7 s |
| `s3-stale-token` | `test-system-s3-stale-token` | PLAN, ENTER, then `POWER:S3:WAKE:REJECTED:TOKEN:ARMED=1:PAGE=0x5a5a0001` as the last line, no kernel evidence after it, QMP `SUSPEND`, `WAKEUP`, no `SHUTDOWN`, raw exit 33, disk oracle | pass, raw 33, 10.7 s |
| `s3-not-durable` | `test-system-s3-not-durable` | injected flush error, SUSPEND ends -16 (`SLEEP_INIT_REFUSALS=4`, `SLEEP_GENERATION=0`), no `POWER:` line, no QMP `SUSPEND`/`SHUTDOWN`, raw exit 33 | pass, raw 33, 10.5 s |

Each of the six cases is followed by the read-only `recover` stage
(`test-system-recover`) on exactly that image: all six passed (raw 33,
5.8–6.2 s, disk unchanged). The stale-token image therefore recovered after
a sleep whose wake was rejected.

## H05-T3 regressions and smoke (same fingerprint)

`native-acpi-s3` (`cycles`), `test-acpi-s3-corrupt-facs` and
`test-acpi-s3-stale-token` passed with raw exit 33; their `POWER:` serial
lines are byte-identical to the H05-T3 evidence
(`docs/verification/h05-native-s3-20261119/`), including
`PM32=0x2000460`. The standard smoke (`tools/build.py`, `tools/run.py
--smoke`) passed with raw exit 33.

## Measured items the design left unverified

- **PIC interrupts after the OVMF S3 resume:** with only the PIC and PIT
  reprogrammed by the kernel (no LAPIC access), IRQ0 reaches the CPU: two
  IRQ0s were counted in every `s3` run. The LAPIC LINT0 state after resume
  was not read; the observation holds for this OVMF build and QEMU TCG only.
- **Wake page in the M3 profile:** first-fit still yields `REAL=0x1000`,
  below `0xA0000`; the 32-bit entry is `PM32=0x2000510` in this image.
- **QMP `SHUTDOWN` after the wake:** captured in every `s3` run, so the
  harness requires it (exactly once, after `WAKEUP`) in addition to raw
  exit 0.

## Commands

```bash
export NOS_QEMU_MACHINE=pc-q35-8.2
cargo +1.100.0 fmt --all -- --check
cargo +1.100.0 test --locked --manifest-path drivers/acpi-power/Cargo.toml
python3 -B -m unittest discover -s tools/tests -p 'test_sleep.py'
python3 -B -m unittest discover -s tools/tests -p 'test_gs_discipline.py'
python3 -B -m unittest discover -s tests -p 'test_system_harness.py'
python3 -m compileall -q tools
NOS_BUILD_DIR=build/power-gate python3 -B -u tools/test_power_system.py
NOS_BUILD_DIR=build/sleep-cycles python3 -B tools/build.py --kernel-feature native-acpi-s3
NOS_BUILD_DIR=build/sleep-cycles python3 -B tools/run_sleep.py --case cycles
NOS_BUILD_DIR=build/sleep-corrupt-facs python3 -B tools/build.py --kernel-feature test-acpi-s3-corrupt-facs
NOS_BUILD_DIR=build/sleep-corrupt-facs python3 -B tools/run_sleep.py --case corrupt-facs
NOS_BUILD_DIR=build/sleep-stale-token python3 -B tools/build.py --kernel-feature test-acpi-s3-stale-token
NOS_BUILD_DIR=build/sleep-stale-token python3 -B tools/run_sleep.py --case stale-token
NOS_BUILD_DIR=build/smoke python3 -B tools/build.py
NOS_BUILD_DIR=build/smoke python3 -B tools/run.py --smoke
```

`tools/test_power_system.py` now runs the three POWER and the three S3
cases by default; `--case <name>` (repeatable) selects a subset. All
commands above exited 0. `cargo +1.100.0 clippy --manifest-path
sdk/Cargo.toml --locked --all-targets -- -D warnings` also exited 0 (it does
not compile the native-only syscall wrappers or system binaries; the kernel
builds above do).

## Not demonstrated / open

- **Not product S3.** Devices are not reinitialized after the wake; the
  latch keeps the services alive but without storage or entropy until the
  next boot. Only a sleep immediately before poweroff is exercised.
- The -14 refusal paths (device latch, missing S3 plan) and the -1 argument
  refusal of SUSPEND are not exercised natively: no process submits device
  I/O after the wake in this profile, and the plan is always present.
- Suspended time is excluded from `ticks`; there is no wall-clock or
  suspend-duration accounting, and the TSC is not trusted across the wake.
- Physical firmware and hardware, wake sources other than QMP
  `system_wakeup`, AML `_PTS`/`_WAK`, the real-mode entry, SMP, and
  profiles with FPU context, IPC wait, the monotonic clock, display/input,
  process memory, networking, random, images or the wall clock (all
  excluded by `compile_error`).
- Independent PM1 I/O tracing; PM1 writes are inferred from QEMU's
  `suspended` state, events and exit.
- No AI or human review of this step has been performed yet; one focused
  security review is warranted (new syscall, sleep inside a system call,
  device latch).
