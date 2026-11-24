# H02-T4 native same-port USB mouse reattach

The opt-in `native-usb-hid-mouse-reattach` profile passes in QEMU. After the
unchanged `reports` sequence the guest retires the keyboard and unplugs the
generation-1 mouse with the left button held. It then re-enumerates a
generation-2 mouse on the same port, using the existing spare VT-d page, and
refuses generation-1 work after slot reuse. The `reattach` (keyboard),
`reports` and `readonly` cases still pass with unchanged markers. H02 remains
**open**. No score change is proposed.

## Source identity

- Source commit: `fe8959dd8327cbce393d395755f956da226b07c5` on
  `claude/h02-mouse-reattach`, base `297d7bfc`. It applies the review fixes
  to the H02-T4 implementation commit `16cb2474`.
- Executed source-v3 fingerprint:
  `e39b303443d5f763bededcf323bfd2d920b1635f790734d618de1ff47871ebe9`.
  `docs/` is outside this fingerprint, so this evidence commit keeps it.
  Each runner report records `base_commit` `fe8959dd…` and this fingerprint.
  The tree was clean apart from the untracked, fingerprint-excluded
  `drivers/usb/target/`. The fingerprint was equal before and after the runs.
- Toolchain: rustc 1.100.0 (0ef1bc386 2026-11-09), cargo 1.100.0
  (3acc79c0c 2026-11-05).
- QEMU: `QEMU emulator version 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3)`
  (`qemu-system-x86_64 --version`), machine `pc-q35-8.2`, qemu64,
  single-thread TCG, one CPU, 256 MiB, OVMF 4M from `/usr/share/OVMF`.
  Hosted CI uses QEMU 8.2.2; this case has **not** run there.
- Execution checkout: the worktree `BaryonOS-worktrees/h02-mouse`, written
  `$W` below. It is not nested inside the main checkout,
  so the doubled-linker-script artifact described in the H02-T3 evidence does
  not apply.
- Isolated environment: `NOS_BUILD_DIR=$W/build/h02m/usb-<case>`,
  `NOS_TARGET_DIR=CARGO_TARGET_DIR=$W/target/h02m`,
  `NOS_QEMU_MACHINE=pc-q35-8.2`, `TMPDIR` on disk outside `/tmp`. Python ran
  as `env -u GIT_EDITOR python3 -B`. One QEMU guest ran at a time.

## Provenance

Results recorded earlier at `16cb2474` are superseded. Every native case,
the mutation control and every host check in the tables below was
**re-executed on 23 November 2026 (Europe/Amsterdam) at the commit above**,
and its exit status was observed directly.

## Native results

Each row is a separate boot. Each produced raw QEMU exit **33**, no timeout,
`passed: true` with artifact binding, and the fingerprint above.

| Case | Kernel feature | Build exit | Run exit | Raw QEMU exit | Duration | QMP replies |
|---|---|---:|---:|---:|---:|---:|
| `reattach-mouse` (run 1) | `native-usb-hid-mouse-reattach` | 0 | 0 | 33 | 3.44 s | 16 |
| `reattach-mouse` (run 2) | `native-usb-hid-mouse-reattach` | 0 | 0 | 33 | 3.48 s | 16 |
| `reattach` | `native-usb-hid-reattach` | 0 | 0 | 33 | 3.32 s | 13 |
| `reports` | `native-usb-hid-probe` | 0 | 0 | 33 | 1.75 s | 7 |
| `readonly` | `test-usb-hid-readonly-fault` | 0 | 0 | 33 | 1.69 s | 1 |

The QMP reply count includes `qmp_capabilities`. Commands per case:

```sh
NOS_BUILD_DIR=$W/build/h02m/usb-<case> python3 -B tools/build.py --kernel-feature <feature>
NOS_BUILD_DIR=$W/build/h02m/usb-<case> python3 -B tools/run_usb_hid.py --case <case>
```

[`native-report.json`](native-report.json) is the runner's report for
`reattach-mouse` run 2, kept verbatim except for one change. The absolute
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

### Guest-asserted behaviour on QEMU 10.2.1

The serial output prints no slot numbers or port events. Each point below
follows from a marker that the guest prints only when the stated condition
holds. In every other case the guest fails closed (`kernel/src/m2/xhci/usb.rs`
at `fe8959dd`).

- `USB:REATTACH:SLOT_REUSED:KEYBOARD` was emitted. In the mouse profile this
  marker is printed only when the generation-2 mouse's slot equals the
  source slot of the first replayed event, which is the retired keyboard's
  slot. Otherwise the guest prints `SLOT_REUSED:MOUSE` or `SLOT_NEW`, and the
  oracle refuses both. The replay check that follows requires each replayed
  event to carry its source slot and to be refused by `accepts()`. For the
  event from the reused slot it also requires `accepts()` to pass once the
  event's TRB address is substituted for the pending one: slot, DCI,
  completion code 1 or 13, remaining length within the four-byte mouse
  transfer and the control-flag mask all match. The keyboard completion was
  therefore rejected by its TRB address alone, and the generation-1 mouse
  completion by its slot.
- `USB:REATTACH:HELD_BUTTON` was followed by `USB:REATTACH:DETACHED`. In
  `hold_and_unplug_mouse()` the following fail closed: any port event before
  the held button or on another port, a connect (code 52), a transfer event
  from another slot (54), and any change other than one left press (53). The
  passing runs show that none of these occurred on QEMU 10.2.1.
- `USB:ATTACH:BUDGET_DENIED` was emitted. It is printed only after the
  checks that the port index is `Detached`, that `attach()` returns
  `Exhausted` and that PED=0. No event may arrive in the bounded quiet
  window, and the port must end with `PORTSC & 3 == 1`; otherwise the guest
  fails with code 46 or the event's own rejection.

## Mutation control (local only, not committed)

`build/mutation.patch` replaces the TRB-address comparison in `accepts()`
(`kernel/src/m2/xhci/usb.rs`) with `event.parameter`, so that only slot, DCI,
completion code and length are checked. This is the spec's control (2). It
corroborates the guest assertion above; it is not its only support.

- Execution: `git apply build/mutation.patch` on `fe8959dd`, then
  `tools/build.py --kernel-feature native-usb-hid-mouse-reattach` (exit 0) and
  `tools/run_usb_hid.py --case reattach-mouse` (exit **1**) with a separate
  `NOS_BUILD_DIR=$W/build/h02m/usb-mutation-r2`, after the native runs above.
- Report: `base_commit` `fe8959dd…`, mutated fingerprint
  `7b0adc19fb3d28d7c09efba4dc8e1c70dcd92367d1abf74807140de49e0f0bbb`,
  `passed: false`, raw QEMU exit **35**, no timeout.
- Result: the mutated guest emitted `USB:REATTACH:SLOT_REUSED:KEYBOARD`, then
  `USB-DIAG:REJECT:000000000000002d` (code 45, cross-device replay accepted),
  then `M1:FAIL:KERNEL_PANIC`.
- Conclusion: with slot reuse across devices, the TRB address alone rejects
  the keyboard replay.

`git checkout kernel/src/m2/xhci/usb.rs` restored the source, and the
fingerprint returned to `e39b3034…`.

## Host checks

| Command | Exit | Result |
|---|---:|---|
| `cargo +1.100.0 test --locked --manifest-path drivers/usb/Cargo.toml` | 0 | 49 tests |
| `cargo +1.100.0 clippy --locked --manifest-path drivers/usb/Cargo.toml --all-targets -- -D warnings` | 0 | |
| `cargo +1.100.0 fmt --manifest-path drivers/usb/Cargo.toml -- --check` | 0 | |
| `cargo +1.100.0 fmt --all -- --check` | 0 | |
| `python3 -B -m unittest discover -s tools/tests -p 'test_usb_hid*.py'` | 0 | 19 tests, including 4 mouse-reattach oracle/harness tests |
| `python3 -m compileall -q tools` | 0 | |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` | 0 | 804 tests, OK (skipped=15) |
| `cargo +1.100.0 check --locked --release --target-dir build/check-usb -p os-kernel --target x86_64-unknown-none --features native-usb-hid-mouse-reattach,test-usb-hid-readonly-fault` | 101 | Expected: `USB reattach and the read-only DMA fault require separate boots` |
| `… --features native-usb-hid-reattach,native-usb-hid-mouse-reattach` | 101 | Expected: `USB keyboard and mouse reattach require separate boots` |
| `… --features native-usb-hid-reattach,test-usb-hid-readonly-fault` | 101 | Expected: widened guard regression |
| `… --features usb-hid-reattach-core` | 101 | Expected: `usb-hid-reattach-core is internal; select a reattach profile` |

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
- **Port events during command waits are dropped, not refused.**
  `Engine::next()` acknowledges every port-status event while a command or
  enumeration waits. A port-1 connect in that window, while port 1 is
  `Detached`, would be ignored rather than fail closed. No attach follows, so
  no authority is widened; see `docs/implementation/NATIVE-USB-HID.md`.
- Same-device rejection by TRB address alone remains keyboard-profile (H02-T3)
  evidence. This profile adds cross-device slot reuse.
