<a id="wait-probe-1--twee-echte-cpl3-processen-één-elf"></a>

# WAIT-PROBE-1 — two actual CPL3 processes, one ELF

Standalone `no_std` acceptance program for the opt-in kernel profile
`native-ipc-wait` (ABI **1.10**, not RTC revision 1.9). No allocator, new
dependencies, privileged I/O, display, entropy, network or personal data.
The integrator supplies the manifest, scheduler fixture, cleanup check and QEMU gate.
A successful build/host test is **not native runtime evidence**.

Initial source baseline: `2a596f4df94573a9a8d74e9cbef668dedc30cf38`,
branch `simon/native-test-version`. The probe reuses
`../materiaal/linker.ld`: RX entry at `0x40000000`, private RW startup at
`0x40010000`, separate R/RW/RX segments. `_start` executes `cld`, checks
SysV entry `RSP % 16 == 8` and jumps to Rust without an additional return address.
`support.rs` follows the existing scalar/volatile memory intrinsics.

<a id="startupcontract"></a>

## Startup contract

All numbers below are **byte offsets of u64 words** from the
private data base address `0x40010000`, not word indices.

| Offset | Kernel input |
|---|---|
| 32 | Role: 0 = initiator/sender, 1 = peer/receiver |
| 56 | Own endpoint root, `ALL_RIGHTS`; RECV/SEND/DERIVE/REVOKE remain kernel permissions |
| 64 | SEND-only descendant of the peer's endpoint root |
| 72 | SEND-only descendant of the process's own endpoint root, **no RECV** |
| 80 | Not read; may be zero |
| 176 | Actual peer PID, required to be nonzero |

PID values 1/2 are not assumed. Each peer message must exactly match the supplied
sender PID, tag, payload length, bytes and zero delegated capabilities.
All ordinary handles pass through `Handle::from_raw`; endpoint wrappers claim
no authority. The negative raw ABI tests deliberately also use handle zero.
The SEND-only handle at 72 must not simply be the same ALL_RIGHTS handle as
56: the negative RECV permission test must receive `-5` for that handle.

<a id="evidencecontract"></a>

## Evidence contract

| Offset | Output |
|---|---|
| 8 | Heartbeat, incremented at the start of each phase |
| 400 | `done=1`, **written last**, on both success and error |
| 408 | Number of successful explicit assertions |
| 416 | Error code; zero only after all role phases |
| 424 | Actual measured total PIT ticks since start, including on error |
| 432 | Completed-phase bitmask, see below |
| 440 | Start tick |
| 448 | Active phase (1–6), 7 after full role acceptance |
| 464 | Role0: measured ticks in the empty-RECV wait |
| 472 | Role0: measured ticks in the full-SEND wait |
| 480 | Both roles: measured ticks of the shared count0 wait |
| 488 | Role0: measured ticks in the SEND wait that returns `-10` |
| 496 | Role1: exactly 17 consumed FULL-phase messages |
| 504 | Role1: 16 final messages not drained after root revocation |

`432` has fixed bits, set only after all local phase assertions:

| Bit | Meaning | Role0 | Role1 |
|---|---|---|---|
| 0 (`0x01`) | Negative raw-wait ABI, all items validated | Yes | No |
| 1 (`0x02`) | Immediate readiness and non-consuming masks | Yes | No |
| 2 (`0x04`) | Empty RECV → delayed peer-SEND → bytes | Waits/receives | Delays/sends |
| 3 (`0x08`) | Shared absolute count0 timer wait | Yes | Yes |
| 4 (`0x10`) | Full SEND → peer drain → wake, exact FIFO without replay | Fills/waits/sends | Drains/checks |
| 5 (`0x20`) | Full queue → root revocation → sender `-10`, no drain | Waits/checks | Revokes/checks |

**Exact final mask for role0: `0x3f`; role1: `0x3c`.** The program checks
these exact masks itself. Success writes `done=1` and calls `exit(0)`.
Every error writes evidence and calls `exit(nonzero)`; the panic code is 255.
The kernel must check both exits, both exact masks, zero error codes, assertions,
ticks, `blocked/wake_ready/wake_timeout/idle_irqs > 0` and an empty authority engine
after cleanup. Evidence is process data, not a substitute for these
kernel observations.

<a id="protocol-en-syscallbewijs-per-fase"></a>

## Protocol and syscall evidence per phase

All messages are SDK `Outgoing`/`Received`, tag `0x5741495450520001`.
The payload is exactly 16 bytes: kind:u8 at 0, three zero bytes, sequence:u32-LE
at 4 and at:u64-LE at 8. No capabilities are sent in messages.
The phase deadline is set once to `now+200`; preparation/drain/revoke timers are
`now+40`. All calculations check for overflow. An `EAGAIN` **never** renews
the deadline. A successful SEND is never repeated.

1. **Negative tests (role0, phase1).** Raw `0x22` checks count17/MAX,
   count0 with a nonzero pointer, each reserved argument and a deadline beyond
   the horizon: `-1`. A null pointer, a kernel pointer and a 16-byte record crossing
   the startup page boundary at `0x40010ff8`: `-3`. Interests0/3/MAX:
   `-1`. RECV on own SEND-only: `-5`. Ready SEND followed by handle0
   must still return `-4`; ready SEND followed by unauthorized RECV must still
   return `-5`. Both array orders are tested. Count0 with an expired
   deadline returns zero; SEND readiness returns SDK `u16` 1.
2. **Immediate (role0, phase2).** `0x21` first returns `EAGAIN`. `0x20`
   sends one actual self-message. Two consecutive `0x22` calls with
   SEND+RECV both return mask3: no reservation/consumption by
   wait. Then one receive with exact bytes, followed by `EAGAIN`.
   Sixteen ready SEND items return `u16::MAX`.
3. **Empty RECV (both, phase3).** Role0 sends EMPTY, checks that its
   own queue is empty and waits for RECV. Role1 receives EMPTY, waits 40 actual ticks
   with count0 and sends DATA with a nontrivial sequence/payload.
   Role0 requires mask1, a positive measured wait interval, exactly one DATA and
   an empty queue afterward. Error codes 160–163/200.
4. **Both Waiting (both, phase4).** IDLE carries one shared absolute
   deadline. Role1 sends IDLE_ACK and performs a count0 wait until that deadline.
   Role0 receives ACK and performs the same count0 wait. Both check that
   the deadline was in the future, mask0 is returned and the tick count reaches
   at least the deadline. With exactly two processes, the kernel must observe actual
   PIT idle via `sti; hlt`; the app cannot certify that instruction
   itself. Error codes 170/201–203.
5. **Full SEND (both, phase5).** FULL carries a future drain tick;
   role1 ACKs and sleeps without receiving. Role0 sends sequences0–15;
   an attempt with 16 returns `EAGAIN`; it waits for SEND before the drain tick.
   Role1 drains at that tick. Role0 requires ready1 and positive wait ticks,
   and sends sequence16 **exactly once** after wake. Role1 receives exactly
   0–16 in order, then checks `EAGAIN` and ACKs DRAINED17.
   Replay, incorrect bytes and a missing message each cause failure.
   Error codes 150–154/180–185/210–212.
6. **Revoke without drain (both, phase6; last).** REVOKE/ACK agree on
   a revoke tick. Role0 again fills exactly 16, receives `EAGAIN` and
   waits for SEND. Role1 sleeps without draining, checks RECV readiness and
   executes `0x12(root)`; the root itself remains valid. RECV is still ready
   afterward; its own SEND descendant is now `-10`. There is **no receive** on
   the full queue. Role0 requires `-10` from the wait, positive measured ticks,
   and again `-10` from both SEND and WAIT; never ready/false success.
   Role1 sends REVOKED over the still-valid opposite direction.
   Role0 receives that confirmation before exit. Role1 remains alive for another
   40 ticks to avoid using root exit as a substitute for revocation.
   The kernel must supply the actual wake-error/queue/cleanup observations;
   the 40-tick hold time is not general evidence of scheduler fairness.
   Error codes 190–195/220–226.

General errors: ABI/startup 1–8, clock 90–91, encoding 92,
bounded SEND100–103, bounded RECV/envelope 110–115. The syscall checks in
phase 1 use 120–138 and immediate readiness 140–148.

The current SDK lacks a public `native::syscall`; therefore, negative `0x22`
tests alone use a local raw int80 wrapper in `main.rs`, with
the same register contract and the same `decode_return` as SDK-native.
No Rust code dereferences the deliberately invalid test pointers.

<a id="bouwen-en-controles"></a>

## Build and checks

No toolchain installation. Use the existing isolated environment:

```sh
export PATH="$PWD/.tools/nieuw-os:$PATH"
export LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu"
export CARGO_TARGET_DIR="$PWD/target/ipc-wait-probe-host"
python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/ipc-wait-probe/Cargo.toml
python3 -B tools/dev.py cargo +1.90.0 check --offline --locked --manifest-path sdk/examples/ipc-wait-probe/Cargo.toml
python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path sdk/examples/ipc-wait-probe/Cargo.toml --all -- --check
python3 -B sdk/examples/ipc-wait-probe/build.py
```

`build.py` respects an explicit `CARGO_TARGET_DIR`; relative paths
are resolved from the repository root. Without that variable it uses
`target/ipc-wait-probe-host`. The integrator supplies an isolated per-app directory
(for example, `$NOS_TARGET_DIR/ipc-wait-app`), never a shared Cargo directory.
The builder uses pinned compiler 1.90.0, matching rust-lld and the existing
scalar flags. It calls the existing
`tools/build_users.py::inspect_user` (also used by
`tools/build_materiaal.py`) and prints ELF segments, hash and
`NOS_IPC_WAIT_PROBE_ELF`. Artifact:
`target/ipc-wait-probe-host/x86_64-unknown-none/release/ipc-wait-probe`.
With an explicit target directory the artifact is always at
`$CARGO_TARGET_DIR/x86_64-unknown-none/release/ipc-wait-probe`, and both the
inspection and `NOS_IPC_WAIT_PROBE_ELF` refer to that same path.
Integration check with a different target directory:

```sh
CARGO_TARGET_DIR="$PWD/target/ipc-wait-probe-explicit" python3 -B sdk/examples/ipc-wait-probe/build.py
```

This is an actual native release build plus ELF inspection; no QEMU run.
The scalar build path avoids SSE/XMM context; the ELF disassembly performed
contained no XMM/YMM/ZMM register references.

Executed during implementation: lockfile generated via Cargo; host tests
3/3, host check, formatter/check, native release build and ELF inspection:
**exit0**. The last four commands above were all executed successfully.
Latest ELF-SHA256:
`3889333416ba3259853caa345aa0809f51b1f9fe5df0cb6ee51e194005aabf2a`.
`cargo fmt` initially could not run: the `rustfmt` component was missing
from the initially supplied toolchain. After the environment owner made it
available, the formatter was run and the check repeated: exit0.
No component was added by this task.
QEMU/native execution, kernel cleanup and full-release acceptance:
**not performed by this subagent**. The integrator owns the
status/score register; this source/build result awards no progress points.
