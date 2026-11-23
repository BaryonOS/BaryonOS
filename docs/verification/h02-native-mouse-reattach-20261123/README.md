# H02-T4 native same-port USB mouse reattach

The opt-in `native-usb-hid-mouse-reattach` profile passes in QEMU. After the
unchanged `reports` sequence the guest retires the keyboard and unplugs the
generation-1 mouse with the left button held. It then re-enumerates a
generation-2 mouse on the same port, using the existing spare VT-d page, and
refuses generation-1 work after slot reuse. The `reattach` (keyboard),
`reports` and `readonly` cases still pass with unchanged markers. H02 remains
**open**. No score change is proposed.

## Source identity

- Source commit: `16cb2474d03f98f5d3b3a728b9ce1275a0238e3f` on
  `claude/h02-mouse-reattach`, base `297d7bfc`.
- Executed source-v3 fingerprint:
  `f398a13019da31fc1b125eb1737d43a87cb961590332ddafaf138078c313bc3e`.
  `docs/` is outside this fingerprint, so this evidence commit keeps it.
  Each runner report records `base_commit` `16cb2474…` and this fingerprint.
  The tree was clean apart from the untracked, fingerprint-excluded
  `drivers/usb/target/`. The fingerprint was equal before and after the runs.
- Toolchain: rustc 1.100.0 (0ef1bc386 2026-11-09), cargo 1.100.0
  (3acc79c0c 2026-11-05).
- QEMU: `QEMU emulator version 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3)`
  (`qemu-system-x86_64 --version`), machine `pc-q35-8.2`, qemu64,
  single-thread TCG, one CPU, 256 MiB, OVMF 4M from `/usr/share/OVMF`.
  Hosted CI uses QEMU 8.2.2; this case has **not** run there.
- Execution checkout: the worktree
  `BaryonOS-worktrees/h02-mouse`. It is not nested inside the main checkout,
  so the doubled-linker-script artifact described in the H02-T3 evidence does
  not apply.
- Isolated environment: `NOS_BUILD_DIR=$W/build/h02m/usb-<case>`,
  `NOS_TARGET_DIR=CARGO_TARGET_DIR=$W/target/h02m`,
  `NOS_QEMU_MACHINE=pc-q35-8.2`, `TMPDIR` on disk outside `/tmp`. Python ran
  as `env -u GIT_EDITOR python3 -B`. One QEMU guest ran at a time.

## Provenance

The builder's session ended after commit `16cb2474` without recording process
exit codes or evidence. Its logs had a matching fingerprint, but they were
not treated as proof. Every native case and host check in the tables below
was **re-executed on 23 November 2026 (Europe/Amsterdam) at the commit above**,
and its exit status was observed directly. The only exception is the mutation
control. It is taken from the builder's artifacts, which were checked as
described in its section.

## Native results

Each row is a separate boot. Each produced raw QEMU exit **33**, no timeout,
`passed: true` with artifact binding, and the fingerprint above.

| Case | Kernel feature | Build exit | Run exit | Raw QEMU exit | Duration | QMP replies |
|---|---|---:|---:|---:|---:|---:|
| `reattach-mouse` (run 1) | `native-usb-hid-mouse-reattach` | 0 | 0 | 33 | 3.31 s | 16 |
| `reattach-mouse` (run 2) | `native-usb-hid-mouse-reattach` | 0 | 0 | 33 | 3.35 s | 16 |
| `reattach` | `native-usb-hid-reattach` | 0 | 0 | 33 | 3.39 s | 13 |
| `reports` | `native-usb-hid-probe` | 0 | 0 | 33 | 1.79 s | 7 |
| `readonly` | `test-usb-hid-readonly-fault` | 0 | 0 | 33 | 1.73 s | 1 |

The QMP reply count includes `qmp_capabilities`. Commands per case:

```sh
NOS_BUILD_DIR=$W/build/h02m/usb-<case> python3 -B tools/build.py --kernel-feature <feature>
NOS_BUILD_DIR=$W/build/h02m/usb-<case> python3 -B tools/run_usb_hid.py --case <case>
```

[`native-report.json`](native-report.json) is the runner's report for
`reattach-mouse` run 1, kept verbatim except for one change. The absolute
worktree prefix in `guest.run_directory` was removed, so the path is relative
to the worktree. It contains the QEMU command line, the artifact and firmware
digests, the serial output, the observed controlled markers and the full QMP
request/response trace.

Observed mouse-profile controlled markers, each exactly once, in order: the
unchanged `reports` tuple (`USB:BEGIN` … `USB:REPORTS:PASS`), then
`USB:REATTACH:HELD_BUTTON`, `USB:REATTACH:DETACHED`,
`USB:REATTACH:ENUM:MOUSE`, `USB:REATTACH:SLOT_REUSED:KEYBOARD`,
`USB:REATTACH:STALE_DENIED`, `USB:REATTACH:READY`,
`USB:REATTACH:MOUSE_DOWN`, `USB:REATTACH:MOUSE_UP`,
`USB:REATTACH:UNPLUG:PASS`, `USB:ATTACH:BUDGET_DENIED`,
`USB:RETIRED:UNCHANGED`, `USB:GUARD:UNCHANGED`, `USB:QUIESCE:PASS`,
`USB:PAGES:RETAINED`, `USB:PASS`, framed by `XHCI:BEGIN`/`XHCI:PASS`.

### Observed QEMU 10.2.1 behaviour

- After the keyboard was retired (slot 1 disabled, DCBAA[1] cleared), Enable
  Slot for the generation-2 mouse returned slot 1. That slot is the retired
  keyboard's, not the generation-1 mouse's slot 2
  (`USB:REATTACH:SLOT_REUSED:KEYBOARD`). The replayed keyboard completion
  therefore matched the new slot and DCI. Only its TRB address rejected it.
  The generation-1 mouse completion was rejected by slot.
- No port-1 event arrived during the held-button and unplug step. The step
  refuses any such event (codes 52 to 54), so the passing runs confirm this
  for QEMU 10.2.1.
- The third identity on port 2 was refused with `Exhausted`, with no port
  reset and no Enable Slot, and no event arrived in the quiet window.

## Mutation control (local only, not committed)

`build/mutation.patch` replaces the TRB-address comparison in `accepts()`
(`kernel/src/m2/xhci/usb.rs`) with `event.parameter`, so that only slot, DCI,
completion code and length are checked. This is the spec's control (2).

- Evidence: the builder's run, `build/logs/mutation.run.log` and its report.
  It was not re-executed in this session.
- Checks performed: `git apply --check build/mutation.patch` exits 0 against
  HEAD, whose `usb.rs` blob is `839e93f8`. The mutation report records
  `base_commit` `16cb2474…` and a different fingerprint,
  `2061feb8f516775afcfb4c6c68e6b980669a1d4c809d787f3e099ef8ced083ac`. It
  records `passed: false` and raw QEMU exit **35**.
- Result: the mutated guest emitted `USB:REATTACH:SLOT_REUSED:KEYBOARD`, then
  `USB-DIAG:REJECT:000000000000002d` (code 45, cross-device replay accepted),
  then `M1:FAIL:KERNEL_PANIC`. The runner raised an assertion failure.
- Conclusion: with slot reuse across devices, the TRB address alone rejects
  the keyboard replay.

The unmutated source was restored, and the fingerprint returned to
`f398a130…` before the native runs above.

## Host checks

| Command | Exit | Result |
|---|---:|---|
| `cargo +1.100.0 test --locked --manifest-path drivers/usb/Cargo.toml` | 0 | 49 tests |
| `cargo +1.100.0 clippy --locked --manifest-path drivers/usb/Cargo.toml --all-targets -- -D warnings` | 0 | |
| `cargo +1.100.0 fmt --manifest-path drivers/usb/Cargo.toml -- --check` | 0 | |
| `cargo +1.100.0 fmt --all -- --check` | 0 | |
| `python3 -B -m unittest discover -s tools/tests -p 'test_usb_hid*.py' -v` | 0 | 19 tests, including 4 mouse-reattach oracle/harness tests |
| `python3 -m compileall -q tools` | 0 | |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` | 0 | 804 tests, OK (skipped=15) |
| `cargo +1.100.0 check --locked --release --target-dir build/check-usb -p os-kernel --target x86_64-unknown-none --features native-usb-hid-mouse-reattach,test-usb-hid-readonly-fault` | 101 | Expected: `USB reattach and the read-only DMA fault require separate boots` |
| `… --features native-usb-hid-reattach,native-usb-hid-mouse-reattach` | 101 | Expected: `USB keyboard and mouse reattach require separate boots` |
| `… --features native-usb-hid-reattach,test-usb-hid-readonly-fault` | 101 | Expected: widened guard regression |

## Not run

- `python3 -B tools/agent_gate.py --job native-xhci.yml#native-xhci --list`
  exits 1 with `uses step needs an explicit local adapter` (the `Checkout`
  step). The local gate subset therefore cannot be listed or run for this job.
  The native cases above were run directly instead.
- Hosted CI for `reattach-mouse`: `.github/workflows/native-xhci.yml` is
  protected. Adding the profile to the USB loop and its report to the
  required paths needs an integration request.

## What this does not prove

- **QEMU only.** All results come from emulated `qemu-xhci`, `usb-kbd` and
  `usb-mouse` devices with `intel-iommu`. No physical USB controller, device,
  cable or hub was used.
- **No physical hotplug.** Unplug and reattach are QMP `device_del` and
  `device_add`, not electrical events.
- **QEMU 10.2.1 only.** The case has not run on CI's QEMU 8.2.2. The slot
  reuse (`SLOT_REUSED:KEYBOARD`) and port-event behaviour were observed only
  on 10.2.1. The guest fails closed if 8.2.2 differs, but that is unverified.
- **The canary detects; it does not prevent.** Retired pages 5, 12, 6 and 13
  stay mapped in the fixed VT-d domain until VM exit. The 0x3c check
  detects a late device write after quiesce; it does not prevent one.
- **No hosted CI coverage yet**, because `.github` is protected (see Not run).
- No IRQ-driven delivery, IPC input broker, input-policy authority, mouse
  wheel bytes (the QEMU boot report is at most 4 bytes), human acceptance or
  independent audit. The mutation control is the builder's local run and is
  not committed code.
- Same-device rejection by TRB address alone remains keyboard-profile (H02-T3)
  evidence. This profile adds cross-device slot reuse.
