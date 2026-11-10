# FR-U01 — bounded native input components

Task contract, 29 October 2026. Owner: `native_ui_apps`; integration: lead agent.
Write scope: only `crates/ui-core/**` and this document. No new dependency,
syscall, kernel permission or claim of graphical product success.

## Result and boundary

The `no_std` core decodes a translated PS/2 Set-1 keyboard stream and standard
three-byte PS/2 mouse packets into physical events. A separate router retains
the focus generation and the owner of pressed keys/buttons. The existing
`Keyboard`, `KeyEvent`, `Modifiers` and ASCII `TextLine` remain available for
G0. The core adds no device I/O, user pointers, text layout or storage. This
is not a compositor, input service or completed U01/U02 acceptance.

Profile: QEMU q35, i8042, keys translated to Set 1 and PS/2 device ID 0;
no wheel/five-button mode, USB, IME or physical hardware claim. The adapter
must actually configure the profile, check controller status, correctly
separate AUX and keyboard, and keep command replies outside the event stream.

## Interfaces and invariants

- `Set1Keyboard::feed`: physical `KeyboardEvent`s with Press/Repeat/Release and
  left/right modifier distinction; text translation remains separate. Actual
  break bytes remain visible as Release after a decoder reset, even without
  local down state. The router never delivers a release without a valid owner.
  Unknown codes and broken prefixes produce an explicit error. The adapter
  resets both decoder and router for lost/controller-faulted input and reports
  input reset to the affected client; no silently stuck modifier.
- `Ps2Mouse::feed`: exactly three bytes per packet; bit 3 marks the first byte.
  Payload bytes may have any bit pattern. Movement values are signed nine-bit
  values; an overflowing axis does not move, while button transitions remain
  available. After known byte loss/timeout, the adapter resets the packet
  boundary. The bit marker alone does not prove recovery after arbitrary
  undetected byte loss.
- `Cursor`: typed screen bounds, clamping and conversion of PS/2 Y to screen Y;
  no pointer acceleration or scaling policy.
- `InputRouter`: only the trusted compositor changes focus. Events carry an
  owner and monotonic generation; delivery rechecks both. Old repeats/releases
  and already pressed modifiers/buttons are not transferred to new focus.
  Generation overflow closes the route and does not reuse a token. Physical
  down state is separate from optional delivery owners: a key pressed without
  focus also remains known. Input reset preserves that state and old owners
  until a real release; a reset invents no click or shortcut. The real release
  immediately clears the marker, even if the decoder itself no longer knows
  of the preceding make. The adapter reports the reset clearly. Router tokens
  are not kernel capabilities: source, process and surface authority must be
  checked at the eventual kernel/service boundary.

## Acceptance of this component

Host tests: fixed packet/scancode vectors, signed extremes, overflow, split
packets, invalid headers/prefixes, Pause/PrintScreen, modifier release and
repeat, cursor clamping, focus changes with held keys/buttons, old queued
events and generation exhaustion. Existing G0 tests must continue to pass.
Compile the library with the specified Rust 1.90.0 for
`x86_64-unknown-none`; run targeted Clippy and format checks in a dedicated
target directory.

Native follow-up acceptance remains open: ring-3 display/input service, kernel
capability boundary, real QMP mouse/keys, two surface owners, focus and
compositor recovery on the final image. Host tests or library compilation
do not prove these scenarios.

## Executed checks — 29 October 2026

On base commit `2409708`, with the corrected source hashes from
[`input-acceptance.json`](../../crates/ui-core/verification/input-acceptance.json),
the following were actually executed (all exit 0):

```text
cargo +1.90.0 test -p ui-core --locked --offline
cargo +1.90.0 clippy -p ui-core --all-targets --locked --offline -- -D warnings
cargo +1.90.0 build -p ui-core --lib --release --target x86_64-unknown-none --locked --offline
cargo +1.90.0 fmt -p ui-core --check
git diff --check
```

Result after review corrections: **30 host tests passed** (25 core tests and
five public-API reset regressions); native library compilation, Clippy and
formatting passed. The first implementation on `2409708` demonstrably failed
two new counterexamples (exit 101, 0/2 passed): a key held without focus was
delivered again after decoder reset; a real release immediately after decoder
reset did not clear the old marker. The same first two regressions pass after
the fix, alongside tests for modifier isolation, no-focus release and
orphan-release/Pause. The compact report retains the exact failing test source,
source commit and log hashes. Physical down state and delivery owners are now
separate; real break bytes reach the router even after decoder reset.
Tool selection uses the existing `tools/common.py::environment()`, with
`LOCALAPPDATA=/workspace/nieuw-os/.tools` and dedicated
`CARGO_TARGET_DIR=/workspace/nieuw-os-worktrees/ui/build/input-target`.
Complete local stdout/stderr: `build/input-acceptance/`; the compact report
contains commands, exit statuses, compiler, source/log hashes and the target
artifact hash. No QEMU/native input trial or human operation was performed
for this slice.

## Sources

- OSDev Wiki, *PS/2 Mouse*, W003157, revision 30841 of 2 July 2026:
  standard packet, sign/overflow bits and device ID. Local text checked.
- OSDev Wiki, *PS/2 Keyboard*, W001900, revision 30840 of 2 July 2026:
  Set-1 make/break, prefixes, command/reply boundary. Local text checked.
- QEMU Project, `hw/input/ps2.c`, tag `v10.0.2`, functions
  `ps2_mouse_send_packet`, `ps2_mouse_event` and `ps2_keyboard_event`:
  https://github.com/qemu/qemu/blob/v10.0.2/hw/input/ps2.c.
  Primary source for the emulator profile, read 29 October 2026; no code
  incorporated. QEMU limits standard mouse deltas there to ±127; the core
  also decodes the broader standard nine-bit format. No claim that a complete
  IBM hardware manual was checked locally.
