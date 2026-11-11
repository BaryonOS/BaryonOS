# TASK-NATIVE-INPUT-SERVICE — real source and policy service

This execution contract builds on input-core4/INP1 and PS/2 adapter `b2ad7c1`.
The dedicated worktree baseline is `b6074c3`, followed by dependency integration
with `985ed4da` that preserves its substantive content. The lead agent owns
bootstrap, capability grants, kernel/SDK hooks and the final native VM gate.
This task writes only new `sdk/examples/input-service/**` files and this
document. No separate kernel replacement, new privileged parser or host decoder.

## Process boundaries

Two new ring-3 executables form the input chain; this does not mean that the
OS has only two processes in total:

| Process | Required owned authority | Data and output |
| --- | --- | --- |
| input-source | One non-delegable InputSource, Source-ingressSEND, own replyRECV, restricted RegistrySEND to init | Actual status/data from the broker, real reset results, INP1 Sync/Sample |
| input-policy | Exactly four RECV ingresses: source, compositor, supervisor, apps; pre-bound replySENDs; RegistrySEND | Existing input-core4, focus/generations/cancel and PEEK/ACK |
| compositor, separate from both services | DisplaySource and restricted focusSEND/replyRECV | Actual surfaces/focus; no source injection |
| init/supervisor | Process-status/management grants and lifecycleSEND/replyRECV | Actual process death to INP1 lifecycle |
| ordinary Materiaal app | Document, surface and appINP1 SEND/replyRECV | Only its own focused events and its own document bytes |

Source, compositor and supervisor PIDs remain mutually distinct, as
`Service::new` requires. An ordinary app receives no raw input/display grant.
The policy receives no device, volume, network or entropy capability. A
payload, guessed PID or local Handle constructor grants no authority. In
production code, all received `Received` envelopes come only from the actual
SDK `native::receive`; the core verifies ingress and kernel sender.

## Bootstrap and actual readiness

The lead agent fixes the restricted startup metadata before embedding the
ELFs: schema/role, own PID, service epoch, source/policy/compositor/supervisor
PIDs, four different policyRECV handles, fixed reply routes, at most eight
client-ID/PID/replySEND bindings, public width/height and RegistrySEND to init.
No request may select an arbitrary reply handle. The startup page is owned
RAM, not a runtime configuration permission for other apps. The fixed IPD1
boot manifest uses the following u64 fields at DATA. This is not a new general
product ABI. Prefix 0–31 remains the existing kernel bootstrap.

| Offset | Source | Policy |
| --- | --- | --- |
|32|`NOSISRC1` little-endian|`NOSIPOL1` little-endian|
|40|BootGeneration >0|BootGeneration >0|
|48|InputSource|sourceRECV|
|56|INP1 SourceSEND|focusRECV|
|64|own replyRECV|lifecycleRECV|
|72|PolicyPID|appsRECV|
|80|ServiceEpoch|SourcePID|
|88|RegistrySEND|CompositorPID|
|96|InitPID|Supervisor/InitPID|
|104|reserved0|ServiceEpoch|
|112|reserved0|width, upper 32 bits zero|
|120|reserved0|height, upper 32 bits zero|
|128|reserved0|SourceReplySEND|
|136|reserved0|CompositorReplySEND|
|144|reserved0|SupervisorReplySEND|
|152|reserved0|RegistrySEND|
|160|reserved0|clientCount1–8|
|168+24×i|reserved0|client ID, client PID, replySEND in three consecutive u64s|

All unused bootstrap words up to 1024 are initially zero. The daemon reads
them once before writing diagnostics. Source must therefore not receive the
old generic ownPID at 176; that is a reserved word here. Policy 176 is the
first client PID. Root supplies the actual identities and public GOP geometry,
without handle/PID lookup based on an app request.

READY uses tag `u64::from_le_bytes(*b"IPDREAD1")` and exactly 32 payload bytes:
`IPD1` at 0, version u16=1 at 4, role u16 at 6 (1 source, 2 policy),
ServiceEpoch u64 at 8, BootGeneration at 16 and RawEpoch at 24. Source reports
the actually confirmed rawEpoch; policy reports 0. Init matches the kernel
sender to the fixed role, generation and epoch. A tag or payload grants no
identity. Registry-send EAGAIN preserves the same message; successful send
means queued, not InitACK. The local handshake has a deadline of 200 actual
PIT ticks, with no polling-counter fallback.

Policy reports ready only after valid configuration and actual validation of
its receive permissions. Source reports ready only after the real PS/2 broker
has reported Ready **and** policy has acknowledged a real authenticated INP1
Sync. Registry readiness travels to init as actual IPC; a private diagnostic
word or host success does not replace that dependency handshake.

The typed InputClient uses only the five previously agreed SDK display/input
wrappers, of which source needs only input 0x82–0x84. ResetBegin returns a
ticket and does not mean Ready. Ready follows only after real ACK/BAT/ID0/
configuration readback by the separate hardware adapter. The 32-byte rawEpoch
belongs to a different domain from the 96-byte INP1 serviceEpoch. INP1 source
request numbers remain strictly monotonic across hardware resets; raw sequence
restarts at 1 within the new rawEpoch.

## Source loop and errors

After every actual Ready, source sends Sync(profile1) and waits for the
correct policy PID, tag, opcode/reply bit, request ID, serviceEpoch, status
and fully canonical reply. Exactly one real raw sample is then retained until
policy has replied to it. The next raw sample is not read while this request
is pending.

Request-send EAGAIN means the same owned bytes with the same number may be
offered again later. After sendOk, receive EAGAIN means only waiting: never
publish the request again. On reply-send EAGAIN, policy may already have
processed the sample; it is not decoded a second time. Timeouts use the actual
100-Hz boot-monotonic SDK clock, not a polling counter or wall clock.

An actual parity/timeout status reaches the core unchanged. QueueFull and
decoder/controller errors are explicitly acknowledged as errors, after which
source performs a real broker reset and new Sync. No fictional error bytes,
prewritten keys or Sync before hardware Ready. Uncertain publication,
wrong/missing reply identity and unexplained raw epoch/sequence errors
terminate source in a fail-closed state; the real supervisor reports SourceDead
and renews the service epoch and grants if necessary. Between detection
without a sample and that real lifecycle report, an explicit supervisor window
exists; without a new loss opcode, the core cannot know about it earlier.
There is no claim of immediate synchronous core cancellation during that
window. A separate new source PID cannot silently take over the old lifetime
binding.

## Bounded policy loop

There are eleven fixed route slots: three actors and at most eight clients.
Each pending slot owns the actual received envelope, ingress and one coreReply.
At most one unanswered request is allowed per peer. Lifecycle, focus and
source are served separately in each round when their own reply slot is free;
shared apps receive a fixed small budget. A second request for an occupied
slot neither replaces the first nor reaches the core. App errors do not close
the shared appRECV.

`handle` is called once. Ordinary core errors are answered where possible
with `error_reply` for the same bound canonical envelope, never as Empty or
success. Unexpected delegated capabilities are closed exactly once on every
path. An unbound/non-canonical request receives no invented reply. All output
handles derive only from bootstrap route binding.

Each round attempts to send each pending slot at most once, in rotating order.
An obsolete event is converted to a negative StaleRequest using the original
envelope. `reply_is_current` is rechecked immediately before the real send,
with no intervening core mutation. EAGAIN preserves the slot; only sendOk
clears it. A dead client route is closed; a stopped core ends the epoch.
`handle` is not repeated to recreate an acknowledgement. Only a real valid
ACK consumes an offered event; PEEK and failed sends do not.

These measures do not prove perfect multi-app admission fairness: with one
shared kernel FIFO, a flooding app can repeatedly occupy a free position.
An authorised supervisor must be able to revoke individual ingress grants
for that purpose. The first native vertical slice uses one Materiaal client;
general resource/flood isolation remains explicitly open. Policy also cannot
retrieve an event already successfully sent to the kernel queue after a later
focus change. The app adapter must correctly apply generation/cancel watermarks
and duplicate PEEK after a failed ACK, without executing an event twice.

## Acceptance and evidence boundary

Host tests use actual capability-engine envelopes and bounded transport
errors for the adapter logic. They do not prove hardware or native sender
provenance. The target build must contain the actual SDK calls; no separate
ASM syscall implementation. Kernel/shared-build/workspace changes remain
owned by the lead agent and are not included in this daemon commit.

The native gate requires the real source and policy processes, actual boot
grants/PIDs/ingresses, successful hardware Ready+Sync and authenticated
PEEK→send→ACK; QMP keys and mouse must control the focused Materiaal app.
Rejection of raw input for the app, source status error/actual reset, focus
cancel, reply backpressure, known process death and bounded cleanup are
recorded with the exact commit/configuration/image hash. This is not yet
complete desktop, input-device, accessibility or full-release acceptance.

## Build interface and private diagnostics

`python3 -B sdk/examples/input-service/build.py` builds the actual
`input-source` and `input-policy` for `x86_64-unknown-none` with pinned
Rust 1.90.0. The caller sets a dedicated CARGO_TARGET_DIR. JSON provides
`NOS_INPUT_SOURCE_ELF`, `NOS_INPUT_POLICY_ELF` and per-ELF hash/segment
inspection; `native_runtime_performed` remains false. Both entry points are
direct `extern "C" _start`; the compiler handles call-site alignment from
the existing kernel RSP mod16=8. There is no parallel ASM syscall layer.

Source writes only private counters: 400 resetStarted, 408 resetReady,
416 SyncACKs, 424 actual samples read, 432 SampleACKs, 440 sendEAGAIN,
448 acknowledged error/reset recovery, 456 first RegistryReady queued,
464 last confirmed rawEpoch, 472 fatal step code and 480 last actual tick.
Policy writes 400 received, 408 successfully corehandled, 416 rejected,
424 pipelined, 432 replyEAGAIN, 440 replies queued, 448 stale replies replaced,
456 client routes retired, 464 actual RECV check mask 15, 472 RegistryReady
queued and 480 fatal step code. These are neither authority fields nor
independent acceptance: init must receive real IPC, and native/provenance
evidence remains bound to the complete lead-agent image. No raw keystrokes
or document content are logged.

## First executed component checks

Thirteen host scenarios actually passed: three bootstrap/registry byte
contracts and ten adapter/lifetime cases using the real capability engine.
They check, among other things, real FIFO EAGAIN, retention of a pending PEEK
across a focus change, no republication after sendOk, a wrong kernel sender,
closing two actually delegated capabilities once, error status/reset/monotonic
numbers, terminal raw loss and a deadline. The host Raw fixture proves only
sequencing; it does not replace a PS/2 broker. The existing large host engine
receives the same explicit 16-MiB test-thread stack as other SDK host tests;
the native 64-KiB stack remains unchanged.

Host and native-target Clippy with `-D warnings`, fmt and both actual
`x86_64-unknown-none` ELF builds pass. The linked executables were also
inspected with objdump/nm: source 2370 and policy 4719 instructions, no
SIMD/FPU, raw in/out, host syscall or unresolved symbol. The direct C entry
contains the compiler push before CALL and preserves the existing SysV stack
alignment. The large initialisation prologues total approximately 46 KiB;
this is targeted artifact inspection, not a complete static stack proof or
native stack acceptance.

The first Clippy run reported the large sourceState enum variant. The pending
owned message was then moved into one fixed optional field outside the small
phase enum, without a heap or increased budget; all 13 tests and both lints
then passed again. No safety check was disabled.

The temporary SDK ABI1.8 wrapper patch supplied by root was used for the
target check and then fully restored. Those shared dependencies are integrated
separately; this daemon commit does not contain them. Exact commands,
source/dependency/patch/ELF hashes and the limited source review are under
`sdk/examples/input-service/verification/`. No daemon QEMU run has yet been
performed: real grants, hardware, registry/init, lifecycle and Materiaal
operation remain the next integrated native gate.
