# Native boundary trial for display and PS/2

This separate fixture is an execution test for ABI 1.8 on the named
development profile `display-input-dev-800x600`: QEMU 10.0.13, `pc-q35-8.2`,
qemu64/TCG/1 CPU/256 MiB, UEFI framebuffer strictly 800×600, the project's
own kernel and three actual CPL3 programs. It is not acceptance of the
Materiaal desktop, input daemon, focus broker or basic programs. The product
scope and all open criteria from `FULL-TEST-RELEASE.md` remain applicable.

The primary geometry source is the existing
`boot/loader/src/graphics.rs::prepare`: bounded GOP QueryMode selection
requires `horizontal_resolution == 800`, `vertical_resolution == 600` and
`pixel_format <= 1`; if that mode is absent, `GOP_800_600` follows. The runner
records this source path and the actual source hash. This small development
profile does not reduce the mandatory full-release profiles (VM-FR, 1280×720
and 1920×1080/200%, with their own resource requirements).

The first real run on `fc9515d66d6d65662de14860f7dc1e3173d05465`
reached both actual generation-1 Ready markers and produced a real 800×600
PPM. The original harness incorrectly expected 1280×720 and aborted the host
run with raw -9/`passed=false` before QMP input. That was a harness error.
That failed evidence is retained; a new run must still pass the complete
pixel, input, permission, sequence and cleanup checks.

## Ownership and integration

The new source files are the standalone crate
`sdk/examples/display-input-probe/**` and `kernel/src/m2/m3/display_fixture.rs`.
The lead agent owns kernel Cargo, shared M3/M4 hooks, SDK ABI/wrappers, image
builder and QMP runner. The fixture grants no general new permissions. The
existing `DISPLAY`/`INPUT` objects remain manifest-only permissions.

The new profile is named `native-display-probe = ["display-input"]`. It must
not be selected together with another native fixture. The M4 capability
engine, IPC dispatch and usercopy remain actually active; the old M4 trial
manifests, observers and their counters are excluded for this profile. The
system trial is not embedded as an alternative program.

`process_elf(slot)` uses only these actually built ELFs:

| Slot / initial PID | Program | Compile-time path |
| --- | --- | --- |
| 0 / 1 | `display-probe` | `NOS_DISPLAY_PROBE_ELF` |
| 1 / 2 | `input-probe` | `NOS_INPUT_PROBE_ELF` |
| 2 / 3 | `display-intruder-probe` | `NOS_DISPLAY_INTRUDER_ELF` |

The kernel calls the new observer with `manifest(&State)`,
`observe_exit(&State, usize, u64)`,
`observe_fault(&State, usize, &[u64;22], u64)` and `poll(&mut State)`.
After the existing framebuffer preparation, `display::prepare()` and
`ps2::prepare()` remain in place; existing cleanup calls both `detach(pid)`
and `m4::process_exit(pid)`. The observer itself performs no display or port I/O.

The private startup/evidence page uses `src/contract.rs`. This page is not a
published application ABI. `SOURCE=48`, `WRONG_TYPE=56`, `PEER_DISPLAY=64`,
`PEER_INPUT=72`, `GENERATION=80`, `CONTROL=88` and `OLD_SOURCE=96`. Results
start at 400; the failing step is at 240. Each process has its own endpoint
as a wrong object type. The input source also has a second endpoint so that
its source handle has a different slot index from the display handle. A
copied number must not become a source permission.

## Actual operations and counterexamples

The display owner obtains the actual geometry through the typed SDK and draws
one tile of 32×8 pixels. Deliberately invalid descriptors, epoch, geometry,
alpha, lengths, other registers and pointers then pass through the real
kernel boundary as raw `int 0x80` calls. Local SDK validation therefore
cannot mask a missing kernel check. Every error result also requires `RDX=0`.

For the last alpha byte, the other pixels have a different colour. For the
copy boundary, the payload points to `0x60010000-320`: the first 320 bytes
belong to the actual private heap; the next page is unmapped. The error must
occur before any framebuffer store. The kernel observer does not read back
invented pixels; the external runner compares the real QMP screenshot.

Round 1 draws at (64,64), round 2 at (112,80). For generation `g` and local
pixel `(x,y)`, the colour is:

```text
((0x20 + g*0x20 + x*3) << 16) | ((0x30 + y*0x13) << 8) | (0x70 + x + y)
```

These are packed `0x00RRGGBB` pixels. The independent runner calculates the
same published pattern definition without taking app results as its image
source. It checks the first tile before new input, then both tiles. The
second round does not overwrite the first tile.

The input source starts a reset itself and waits for the actual ACK/BAT/ID0/
Set2 and controller-translation chain. A startup ticket is not Ready. Before
the first reset poll, an invalid output pointer receives an error without a
prefix change. Only the `Ready` reply with the actual epoch activates the
source. The empty Ready queue returns EAGAIN without changing the output
sentinel.

For each generation, the observer reports
`DISPINPUT:DISPLAY_READY:<20 cijfers>` and `DISPINPUT:INPUT_READY:<20 cijfers>`
(the placeholder denotes 20 digits). The QMP runner waits for both steps and
takes the screenshot. It then sends a real `a` make/break, an
`input-send-event` batch with relative x=7/y=-3 and left button down, followed
by left button up. The source requires exactly these eight actual bytes:

```text
1e 9e 09 07 03 08 00 00
```

The first two have keyboard status; the last six have AUX status. OBF must
be on and parity/timeout must be off. Raw samples remain raw bytes; this trial
is neither a host decoder nor native input policy. The source requires actual
sequences 1..8 and the current reset epoch. Before **every** valid read, it
tries an output buffer crossing the heap guard: that rejection must consume
neither a FIFO byte nor a sequence. All eight status/byte combinations are
emitted on actual exit as `DISPINPUT:RAW:<20 cijfers>`, each preceded by
`DISPINPUT:SEQ:<20 cijfers>` from the actually received `Sample.sequence`
(the placeholders denote 20 digits). Private sequence fields are at 640..696.
The host validates both transcripts separately as sequence 1..8; it does not
construct a sequence solely from the position in the output.

The unauthorised third app tries both copied source integers, wrong object
types and zero handles. The two owners try derivation, zero-rights derivation,
revocation and IPC transfer of their source; these remain rejected. Invalid
output calls preserve their sentinels. Both owners close their source and
then check the stale handle through a real syscall. For DERIVE/TRANSFER/REVOKE,
the existing error code is `Invalid` (-1): these operations are not supported
rights of the source object type. Two targeted host contract tests use the
real capability engine to check these error codes, complete failure atomicity
and copied/closed/old-handle expectations before the native trial.

When all results are ready, the observer sets CONTROL=1, observes actual exits
and allows normal cleanup to run. Only after all processes are Dead does it
check engine invariants, empty object/queue/handle and process counts, and
recovered physical frames, accounting for explicitly retained kernel page
tables. It then reuses all three slots with new PIDs 4..6 and new manifest
capabilities. Old source integers remain invalid; display epoch, input epoch
and ticket must increase. The new input source must not inherit an old Ready
and performs hardware initialisation again.

## Counters and evidence boundary

After two rounds, the fixed numbers are: BLITS=2, PIXELS=512,
DISPLAY_DENIED=22, CAP_DENIED=44, RESETS=2, READY=2, SAMPLES=16, CLOSED=4,
OLD_DENIED=2, EXITS=6, SLOT_REUSES=3, LIVE_PROCESSES=0 and
FRAMES_RECOVERED=1. POINTER_DENIED is at least 18; OUTPUT_UNCHANGED is
POINTER_DENIED+27. RESET_PENDING is at least 2. More empty reads are permitted,
but each additional pointer rejection must also produce an unchanged prefix.
The independent adapter counters are display `[2,512,35]` and input
`[2,2,16,POINTER_DENIED+17,werkelijk_discarded]` (the final field is the
actual discarded count).

Only after actual execution, an exactly validated marker/counter transcript,
QMP input and independent pixel checks may the final markers
`DISPINPUT:DISPLAY:PASS`, `INPUT:PASS`, `ISOLATION:PASS`, `CLEANUP:PASS` and
`M1:PASS` count as native evidence. The raw VM exit must be 33. Compilation,
a host test, a populated startup value or a previously displayed screenshot
does not replace that evidence. The observer bounds the trial to 10000 ticks
and 2000000 syscalls.

## Executed preparatory checks

`LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-display-probe-target python3 -B sdk/examples/display-input-probe/build.py`
built the three ELFs with Rust 1.90.0, the project's linker, an offline lockfile
and disabled SSE/AVX code generation. `objdump -d` found no FPU/SIMD
instructions or registers in those actual binaries. The entry is an ordinary
extern-C function: the compiler inserts a `push rax` before the call to main,
matching the existing kernel entry with RSP mod 16 = 8.

The new crate was checked by Clippy with the native feature and none target,
using `-D warnings`; fmt and Python syntax checks pass. The current existing
PS/2 test set (8 tests) and display-client test set (5 tests) pass in dedicated
targets. This preparatory evidence was built using the temporary ABI1.8 SDK
patch supplied by the lead agent; that patch is not included in the fixture
commit. The combined native kernel build and QMP operation follow in the
integration branch. This text does not yet claim that execution.
