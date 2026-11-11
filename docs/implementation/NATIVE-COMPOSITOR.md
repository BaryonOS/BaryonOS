# TASK-NATIVE-COMPOSITOR — one assigned native surface

This ring-3 server owns only the DisplaySource, SurfaceRECV/AppReplySEND,
INP1FocusSEND/FocusReplyRECV and SupervisorControlRECV/RegistrySEND granted
by root. No personal volume, key service, network, rawInput or framebuffer
pointer. The project's kernel retains device/copy and capability mechanisms;
SUR1 parsing, focus correlation and drawing policy remain in this separate
userspace service.

## Contract and permissions

Root bootstrap is a private 1024-B startup page at DATA. All fields are u64:
32 `NOSCOMP1` little-endian, 40 BootGeneration>0, 48 DisplaySource,
56 SurfaceRECV, 64 AppReplySEND, 72 AppPID, 80 SurfaceEpoch,
88 INP1FocusSEND, 96 FocusReplyRECV, 104 PolicyPID, 112 PolicyEpoch,
120 SupervisorControlRECV, 128 RegistrySEND, 136 InitPID, 144 ClientID.
Unused words are initially 0. This is a bounded boot manifest, not a new
general product ABI. Handles/PIDs in payloads never grant new authority.
InitPID1 is the existing lifecycle supervisor; SourcePID6, CompositorPID5
and SupervisorPID1 remain distinct. PolicyPID7 is outside that actor
configuration.

The compositor checks the actual received kernel sender, tag and zero
capabilities before processing SUR1 or control. Unexpected received
capabilities are closed once on every path. Reply destinations are fixed
at bootstrap. A wrong app or a request cannot select its own reply capability.
The two closeable surface handles must alias neither each other nor the
display, focus or control routes; bootstrap rejects such overlap.

NativeDisplayEpoch comes only from the actual DisplayClient geometry call
and remains internal. SUR1 INFO exposes public GOP width/height, SurfaceEpoch
and one assigned global rectangle `(16,48,W-32,H-64)`. The checked minimum
is 352×304, giving the app at least 320×240. All subtractions/conversions and
bounds are checked beforehand. Input policy retains global GOP geometry;
the app adapter translates only within the rectangle and cancels local
pointer actions outside the surface. The compositor has no raw mouse hook.

## Actual display chain

SUR1 retains at most one 1024-B tile. Only the opaque CommitReady from accept
can be completed through commit_with with a mandatory callback. Production
uses only the actual DisplayClient.blit; no host backend supplies positive
evidence of device I/O. A callback error is a negative Blit status even with
Ok(Reply) and is not counted as successful pixels/a frame.

There is at most one pending surface reply. IPC EAGAIN retains exactly that
reply and stops new SurfaceRECV, but not control/focus processing. Each round
expires staging before accept; a timeout reply has the same retention duty.
If the reply deadline expires or the reply route is closed, only that surface
is revoked. Control/focus remain available; a display error is terminal.
Commit starts only after a fresh actual monotonic tick check; no token is
retained across wait rounds or arbitrary callbacks. Previously confirmed
tiles are visible writes, not an atomic page flip/vsync or rollback guarantee.

Four non-overlapping chrome strips lie entirely outside the app rectangle.
Their private 32×8 tiles use exactly Noto Sans Mono Bitmap 0.3.2 and the same
compile-time `ui/prototype/tokens.css` through the existing pure design module.
The compositor graph contains no font parser, document controller, media
decoder or document data. Font pointer tables and text/bitmap data receive
R+NX; the linker uses five separate LOAD segments. The entry is direct
externC_start with the existing C stack convention.

Native geometry, all pure runtime configuration and complete drawing of chrome
plus clearing of the assigned app rectangle must succeed through actual
DisplayClient blits before READY. During bootstrap, CPU time is yielded after
at most 32 tiles. This removes visible previous pixels in that area; it is
not a general VRAM erasure guarantee. READY tag `IPDREAD1`, 32 B:
`IPD1`@0, version u16=1@4, role u16=3@6, SurfaceEpoch u64@8,
BootGeneration@16 and actual NativeDisplayEpoch@24. Init binds kernel PID,
role, generation and epoch. A queued READY is not InitACK.

## CMP1 control

Init uses tag `u64::from_le_bytes(*b"CMPCTRL1")`, exactly 48 bytes:
`CMP1`@0, version u16=1@4, opcode u16@6, requestID u64@8,
SurfaceEpoch@16, BootGeneration@24, status u64@32, state u64@40.
Request status/state are 0. Op 1 FocusStart, 2 FocusStop, 3 Retire.
Request ID is non-zero and strictly monotonic. No client ID, rectangle, PID
or reply handle from the control payload is used.

Control replies travel through the fixed RegistrySEND to init, with
opcode|0x8000 and the same request/epoch/generation binding. Closed status set:
0 Ok, 1 Malformed, 2 Access, 3 Binding, 4 Replay, 5 Busy, 6 Input,
7 Uncertain, 8 Retired, 9 Exhausted, 10 Display. Only at status 0 does state
mean 0 Inactive, 1 Focused or 2 Retired; negative replies have state 0 without
declaring inactivity.

Init sends at most one unanswered control request. The compositor separately
retains one pending control reply alongside the surface reply. EAGAIN retains
the same bytes without applying the command again. FocusStart sends INP1
Focus with the bootstrapped ClientID; FocusStop and Retire send Focus0, as
the existing input core supports. PolicyEpoch differs from the surface/native
display epoch. Focused/Inactive/Retired success follows only after a real
canonical INP1 acknowledgement from the bound policy PID.

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:ede901fc3478f02c5afd36e48c878a21f2183f6326e58f58567b7934d737c8dd -->

After successful focus publication, the compositor waits for a reply without
resending the request. An incorrectly bound reply or expired deadline ends
the epoch as uncertain, without a rollback or success claim. Retire removes
staging, rejects a pending surface reply and closes new surfaces. After the
Focus0 acknowledgement, the compositor clears the complete assigned app
rectangle with bounded background blits. Only after completion may it
acknowledge Retire; a blit error is terminal. The 200-tick reply deadline
starts after clearing completes. The period between Focus0ACK and reply can
therefore exceed 200 ticks; init monitors the entire compositor operation
with a separate 500-tick deadline. Maximum clearing time has not yet been
measured natively. Previously valid published replies cannot be retrieved
from another process's kernel queue.

<!-- /coderabbit-ai-generated:v1 -->

## Execution and evidence boundary

Fixed round budgets: at most one control receive, one focus reply, one surface
message and one chrome tile; only during Retire, at most 32 clearing tiles.
App flooding or a full app reply queue does not stop protected control/focus
routes. Personal pixel/key payloads are not logged. Root owns init/kernel
hooks, manifest grants, final image/QMP capture and the test image. This
component gate first requires pure host error trials and an actual ELF target
build; it does not yet provide a positive fake-hardware or native desktop
claim. Actual init/process death, native compositor–Materiaal operation,
recovery, multiple windows and accessibility remain separate open release
criteria.

## Build and observation

`LOCALAPPDATA=/workspace/nieuw-os/.tools PATH=/workspace/.nieuw-os-env/bin:$PATH python3 sdk/examples/compositor/build.py`
builds the dedicated standalone Cargo workspace offline/locked with Rust 1.90.0,
scalar x86_64-unknown-none and the checked five-segment linker script. Root
must have integrated the actual ABI1.8 wrappers; component validation uses
only the approved temporary SDK patch and then restores it. `NOS_COMPOSITOR_ELF`
points to the output; this ELF build is not an executed OS test.

Private startup-page observation words are not authority: 400 actual chrome
blits including bootstrap, 408 received surfaces, 416 acknowledged actual
app blits, 424 pixels from those blits, 432 rejections, 440 surface-reply
EAGAIN, 448 published focus requests, 456 canonical focus ACKs, 464 completed
frame or 0, 472 Ready queued, 480 focus state, 488 fatal step code, 496 actual
app-rectangle clearing tiles including bootstrap. Root must correlate these
with real process/broker events, source/ELF/image hashes and the rendered
interface.
