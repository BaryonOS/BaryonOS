# TASK-NATIVE-INPUT-PREP — input service core before native integration

Task contract, 30 October 2026, baseline `309ff53`. Owner: `native_ui_apps`;
the lead agent owns kernel/driver/compositor integration. Only new files under
`services/input-core/` and this document. No syscall, kernel permission,
workspace membership or existing SDK/UI interface is changed.

## Actual starting point

`ui-core` provides tested Set-1/PS2-ID-0 decoders and physical-down/focus
routing. G0 provides only kernel polling and framebuffer diagnostics. A native
InputSource broker, protected ring-3 ingress, compositor and input service
loop are still absent. This core reuses `ui-core` and the existing SDK
envelopes; it performs no I/O and does not prove a working native input service
or GUI.

## Authority, lifetime and error behaviour

- Bootstrap binds the service epoch, source PID, compositor PID, supervisor PID
  and at most eight client-ID/PID pairs. Four different RECV endpoints fit
  within the existing kernel quota: source, focus, lifecycle and shared apps.
  Only pre-granted SEND capabilities reach the respective ingress.
- `Received.sender()` comes only from real `native::receive`; a host-side
  `Received::from_bytes` does not prove provenance. The core checks both the
  actual receiving endpoint and the kernel PID. Payload, tag and guessed IDs
  grant no authority. An app cannot inject input, change focus, choose another
  client PID or report process death.
- The compositor changes only focus; the supervisor reports client/source/
  compositor death based on actual kernel observation. A dead client ID is
  not rebound within this service lifetime. Source/compositor death blocks
  input until the agreed protected reinitialisation is complete.
- On a focus change, input loss or a full event queue, old input becomes
  invalid. A separate cancel watermark takes precedence over events; it clears
  only generations up to and including the specified bound. Known pressed
  keys/buttons do not transfer to new focus through the existing router.
- Apps read with PEEK and acknowledge with ACK for exactly the offered event
  number. A failed IPC send consumes no event. Old ACKs cannot remove a new
  cancel or event. The native adapter checks reply validity immediately before
  send and handles it without an intervening focus/lifecycle mutation in the
  same service thread.
- The source actually configures Set 1 and mouse ID 0 before SYNC. Samples
  carry monotonic source sequence numbers and actual i8042 status/data. A gap,
  controller error or defective packet stream closes samples until explicit
  SYNC after draining/reinitialisation. No byte heuristic claims recovery
  after undetected loss.
- Counters do not wrap. Global event/focus or protected source-counter
  exhaustion closes the core; an exhausted app request number blocks only
  that client. Otherwise, an app could stop all input with one MAX request.
  When the core stops, the adapter closes connections and reports cancellation.
  Unexpected received delegated capabilities are rejected and must be closed
  by the adapter with `native::close_received_caps`; the core owns no syscall
  I/O.

## Wire v1

Dedicated protocol payload of 96 bytes within the existing Message320,
little-endian; no Rust struct cast. Header: `INP1`, version u16=1, opcode u16,
request ID u64, service epoch u64, arg0 u64 and arg1 u64. Other request bytes
are zero. Tag `0x4e49_4e50_5554_0001`; no transferred capabilities.

Source: SYNC (profile 1), SAMPLE (status byte/data byte). Focus: FOCUS
(client ID, 0 clears focus). Lifecycle: CLIENT_DEAD, SOURCE_DEAD,
COMPOSITOR_DEAD. Apps: PEEK and ACK(event number). Roles cannot use each
other's opcodes. A wrong epoch, non-canonical fields, unknown opcode,
missing/extra bytes and old request numbers are rejected before any authority
change.

Replies bind opcode+reply bit, request ID and service epoch. A PEEK contains
Empty or one key/mouse/cancel event, with event number and focus generation.
Key ID follows the explicit `PhysicalKey` u8 representation of this source
contract; compatible extension requires retaining existing numbers or a new
wire version. Arrow keys are physical keys; Unicode/text layout and hit
testing remain separate compositor/text-layer interfaces.
Status 0 is success and 1 is Empty; negative statuses 2–18 are the explicit
`Error` numbers. `error_reply` creates a negative reply only for an already
bound sender and a canonical current-epoch request. Other protocol errors
are rejected and reported locally; an app error must not close the shared
app ingress for other clients. The adapter closes unexpected capabilities
and can revoke only the affected client grant. A core that has actually
stopped ends its entire epoch and causes all affected clients to cancel input.
No success or empty input is issued in place of an error. Empty and ordinary
command acknowledgements describe state at handling time; they grant no
current focus. Only replies with an input/cancel event number become invalid
on a focus/lifecycle change. An Empty received later does not clear input
state or a pending cancel watermark.

## Acceptance of this preparation

Host tests use the real capability engine for kernel-envelope provenance and
rejection of unprotected routes, alongside fixed malformed-wire cases. Also:
focus change/death, old ACKs, queue overflow/cancel, stream gaps, controller
errors, PS/2 mouse/keyboard, no-focus reset regression, counter exhaustion and
bounded memory. Actually compile the library for `x86_64-unknown-none`; run
Clippy and fmt in a dedicated target directory. The host model and target
build receive no native U01/U02 points.

The native follow-up gate remains open: protected source capability and driver
loop, kernel provenance and real process-death observation, four ingresses
and reply routes, QMP keys/mouse in two isolated clients, EAGAIN/restart and
compositor focus. Any restarted service receives a new epoch and new endpoint
generations; old local tokens are not reusable authority.

## Executed evidence — 30 October 2026

On baseline `309ff53` plus this new core, the following actually passed (exit 0):

```sh
cargo +1.90.0 test --manifest-path services/input-core/Cargo.toml --locked --offline -- --nocapture
cargo +1.90.0 clippy --manifest-path services/input-core/Cargo.toml --all-targets --locked --offline -- -D warnings
cargo +1.90.0 build --manifest-path services/input-core/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline
cargo +1.90.0 fmt --manifest-path services/input-core/Cargo.toml --check
git diff --check
```

**20 host tests** (19 boundary/lifetime/wire cases plus counter exhaustion)
pass. In these host trials, the real capability engine provides sender PID
and scoped receive, blocks a forged sender envelope and exercises actual
queue backpressure. This remains a host model of that kernel component,
not evidence of a native input service. Measured `size_of::<Service>()` is
4256 bytes on x86-64, below the local 8-KiB core budget; the library uses no
heap or external dependencies. Production dependencies are only the existing
local ui-core and SDK; capability-core is a dev dependency only. Shared
Cargo/kernel/SDK/UI files were not changed.
A separate agent independently repeated the original 18 tests, Clippy and
freestanding build on `e228f0d`, with no concrete blocker. Review
clarifications on client-local counter exhaustion and Empty as an observation
were then recorded in two additional regressions; the IPC queue trial now
requires exactly kernel EAGAIN. Production code and wire behaviour were not
changed. This is an AI review within the stated scope, not a human audit.

Compiler and source/log/target hashes are in
[`services/input-core/verification/host-target.json`](../../services/input-core/verification/host-target.json).
Complete local logs: `build/input-core-checks/`; target directory:
`target/input-core/`. Tool selection used the existing
`LOCALAPPDATA=/workspace/nieuw-os/.tools` through `tools/common.py::environment()`.
An initial test compilation required an explicit u64 annotation in the test
helper; the checks above were then repeated on the final source.

Still **not executed**: native hardware ingress, real source capability,
driver and compositor processes, process-death notifications, QMP keys/mouse,
focus on real surfaces, native service restart and product accessibility.
The other conditions from FULL-TEST-RELEASE remain open. The source baseline
remains the project's own microkernel and the existing
[input decoder agreements](NATIVE-INPUT.md) and Message320 contracts.
