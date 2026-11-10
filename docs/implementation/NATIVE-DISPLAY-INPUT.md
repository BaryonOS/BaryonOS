# Native display and input transport — execution

Baseline `61ea7278390d5235a8bc024cbf084222a7b0c658`, 30 October 2026.
The lead agent has reserved DISPLAY=4096, INPUT=8192 and opt-in ABI 1.8.
This task retains G0 as a separate regression and makes no release claim.

Owner of the new transports and capability engine: `git_build_audit`.
Root owns boot/framebuffer extraction, central syscall/SDK hooks, build,
workspaces, manifest grants and integration. `native_ui_apps` owns the ring-3
renderer/controller. The memory agent retains ownership of m3/paging/Sys4.

The kernel receives only fixed hardware transport: no fonts, GUI parser,
document policy, focus policy or keyboard layout. The framebuffer remains
supervisor-RW/NX/UC-MMIO, outside RAM/heap/DMA ownership. One typed,
non-delegable manifest capability gives the compositor transport access.
A second source receives no authority. Closing/exit and generation exhaustion
must not make old handles valid again.

Display: at most 32×8 pixels per operation, packed little-endian u32
`0x00RRGGBB` (bytes B,G,R,0). Geometry returns only width, height and epoch;
no physical address, window pointer or device stride. All descriptor and
payload plans are fully validated and copied to private RAM; all rows/colours
are checked before the first VRAM write.

Input: one protected hardware source receives only a fixed bounded reset,
status/data read and readiness report. No raw-port/command interface for apps.
The source configures both i8042 channels, Set 2 with controller translation
to Set 1 and mouse ID 0. Ready requires actual ACK/BAT/ID and a checked
controller mode. Output plans are validated before status reads, FIFO
consumption or commands. Reset is a bounded state machine; pending never
means Ready. AUX, parity, timeout, known lost bytes and epochs remain visible.
The ring-3 source then delivers actual IPC to the existing input-core; a
payload or locally constructed `Received` grants no source PID/focus permission.

Native acceptance remains open until root integrates the hooks and executes
real QMP keys/mouse, rendered pixels, rejection, stale handles, invalid
usercopy, reset errors and cleanup on the current integrated image. Pure
driver/codec tests do not replace that evidence.

## Fixed wire/syscall contract 1.8

All other register arguments are zero; errors return RDX=0 and do not change
the caller's output. The kernel selects the actual PID from the syscall
context. Source handles are not process IDs or physical addresses.

| Number | Arguments | Success RDX |
| --- | --- | --- |
| 0x80 DisplayInfo | handle, output pointer, 32, 0, 0, 0 | 32 |
| 0x81 DisplayBlit | handle, descriptor pointer, pixel pointer, pixel bytes, 0, 0 | width×height pixels |
| 0x82 InputResetBegin | handle, 0, 0, 0, 0, 0 | new ticket >0; started only |
| 0x83 InputResetPoll | handle, ticket, output pointer, 32, 0, 0 | 32, Ready only |
| 0x84 InputRead | handle, output pointer, 32, 0, 0, 0 | 32, one actual status/data sample |

`EAGAIN` on Read means no FIFO data; on ResetPoll it means an authorised reset
is still in progress. After valid output preflight, ResetPoll may execute
steps and drain bytes, but on Pending it writes neither Ready nor output.
There is no success fallback and no automatic second reset after an ambiguous
reply. A wrong ticket/owner cannot continue another reset.

The four fixed records are exactly 32 little-endian bytes:

- Info: width u32@0, height u32@4, epoch u64@8, maxwidth u16@16=32,
  maxheight u16@18=8, format u32@20=1, eight zero bytes from 24.
- Blit descriptor: epoch u64@0, x/y u32@8/12, width/height u16@16/18,
  flags u32@20=0, bytecount u32@24, reserved u32@28=0.
- ResetReady: ticket u64@0, epoch u64@8, discarded u32@16, operations u32@20,
  profile u32@24=1, flags u32@28=0.
- Raw input: epoch u64@0, sequence u64@8, status u8@16, data u8@17,
  zero bytes from 18. Status contains actual OBF/AUX/parity/timeout; the kernel
  does not rewrite a keyboard error as a printable sentinel or discard mouse
  bytes. An error status requires another reset.

The SDK crate shares the fixed pure `wire` module with the kernel. The kernel
uses `default-features=false`: no user runtime, allocator or GUI in that
dependency. The optional/default `client` layer uses the existing SDK runtime.
Only root adds the central native wrappers; the client does not duplicate
int80 assembly. The typed Source wrappers check representation, grant no
permissions and do not close borrowed/shared handles. The owner explicitly
closes the manifest grant. An app cannot create global source objects itself.

<a id="begrensde-ps2-reset"></a>

## Bounded PS/2 reset

The pure `drivers/ps2` core operates over four fixed typed port operations.
Each ResetPoll performs at most eight state-machine actions, hence at most
sixteen port operations. Global bounds are 8,192 operations, 256 drain bytes,
at most two RESEND retries, 64 monotonic PIT ticks per step and 1,000 per reset.
There is no kernel polling spin or TSC/host/wall-time fallback.

The closed command sequence disables interfaces and performs bounded draining,
disables IRQ1/IRQ12, initialises the keyboard with ACK+BAT, temporarily
disables scanning, selects Set2, initialises the mouse with ACK+BAT+ID0 and
defaults, enables controller translation to Set1, checks the command byte and
establishes scanning with F4 and actual ACKs before Ready. RESEND of a mouse
command also repeats D4. Unexpected replies, error status, missing ACK/BAT/ID,
wrong mode, budget/timeout and counter overflow do not produce Ready. Input
from the other channel during reset is explicitly counted as discarded and
covered by the new epoch/SYNC. There is no claim that i8042 reports every
possible undetected physical byte loss.

Tickets and epochs do not wrap. Raw sequences start at 1 after each successful
reset; exhaustion fails before FIFO read. Closing/exit or a new global object
generation must not inherit an old Ready/reset. `detach(pid)` is only for
proven process death/complete principal retirement, not a delayed old-grant
event that would erase a new session of the same living PID.

Native hook points to be connected by root:

- `display::prepare()` after the actual framebuffer map; `ps2::prepare()`
  performs no I/O.
- `display::syscall(state,slot,number,args)` for 0x80/81;
  `ps2::syscall(...)` for 0x82–84, before the generic M4 fallback.
- `display::detach(pid)` and `ps2::detach(pid)` on actual principal cleanup.
- Manifest-only `Engine::create_display(pid)` / `create_input(pid)` and
  `check_display` / `check_input` with the actual caller PID. Static UI tests
  cannot use the new permission as raw port I/O or a RAM-frame permission.
- Root builds/versions ABI1.8 and grants sources only to the compositor and
  hardware source process. That source delivers SYNC/SAMPLE through actual
  IPC to input-core; compositor and lifecycle ingresses remain separate
  authorised PIDs.

Kernel counters are display `[blits,pixels,denied]` and input
`[started,ready,samples,denied,discarded]`. Repeated polling of the same
successful reset ticket confirms the same Ready without additional hardware
work or an additional ready count. A retained image after compositor failure
is not yet a safe session-lock/recovery experience; that product gate remains
explicitly open.

## Checks and sources

The capability slice has 47 host tests, including globally exclusive quotas,
PID/type, rights expansion, derive/transfer/revoke, close/reuse/exit,
maximum-generation retirement and atomic slot/node quotas. A separate reviewer
repeated them and five additional counterexamples (52 passed), with no
concrete blocker.

PS/2 has eight host tests and passing Clippy/freestanding library build; the
SDK has five host codec tests and passing Clippy/pure-wire target build. The
complete native SDK layer and new kernel adapters await the central
hooks/dependencies managed by root. This text does not approve that native
boundary.

Commands from this worktree, with `LOCALAPPDATA=/workspace/nieuw-os/.tools`
and dedicated target directories:

```text
python3 -B tools/dev.py cargo +1.90.0 test --locked --offline -p capability-core
python3 -B tools/dev.py cargo +1.90.0 test --manifest-path drivers/ps2/Cargo.toml --locked --offline
python3 -B tools/dev.py cargo +1.90.0 clippy --manifest-path drivers/ps2/Cargo.toml --locked --offline --all-targets -- -D warnings
python3 -B tools/dev.py cargo +1.90.0 build --manifest-path drivers/ps2/Cargo.toml --locked --offline --release --target x86_64-unknown-none --lib
python3 -B tools/dev.py cargo +1.90.0 test --manifest-path sdk/display-client/Cargo.toml --locked --offline
python3 -B tools/dev.py cargo +1.90.0 clippy --manifest-path sdk/display-client/Cargo.toml --locked --offline --all-targets -- -D warnings
python3 -B tools/dev.py cargo +1.90.0 build --manifest-path sdk/display-client/Cargo.toml --no-default-features --locked --offline --release --target x86_64-unknown-none --lib
```

The checked primary emulator source is QEMU 10.0.13:
[`hw/input/ps2.c`](https://github.com/qemu/qemu/blob/v10.0.13/hw/input/ps2.c),
SHA-256 `fe898d87af4c87c138767b272c8ebcb0932927fcc889b4a82ac50e5086cda715`,
and [`hw/input/pckbd.c`](https://github.com/qemu/qemu/blob/v10.0.13/hw/input/pckbd.c),
SHA-256 `e48dc36f8ddd165877ac647408dd4f10ab782dc9061e25f78c20a975abb5eaa4`,
read 30 October 2026. No QEMU code was incorporated and no new external
dependency added. Physical PS/2/GPU hardware and universal device support
have not been tested. Root retains G0 and all existing release gates alongside
the new native operation trial; complete Materiaal and release acceptance
remain open.
