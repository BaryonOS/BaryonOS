# NAS-IPC-PROBE-1 — local native acceptance interface

This is a **separate P04 probe**, not the integrated release, an
interoperable NAS network protocol, authenticated device pairing or
E2EE client. The client sends recognizably synthetic opaque bytes,
no real documents or keys. P01–P04 remain open until their required
native evidence exists; an ELF build earns no point.

Source writer: NAS track, exclusively `services/nas/**`. Kernel manifest,
syscall selection, root build, QEMU runner and final evidence binding remain
with TL-REL. They are not set by this builder or these binaries.

## Artefacts and memory

Rust 1.90.0, `x86_64-unknown-none`, `no_std`, static ELF64, no allocator
or external package. The single `nas-probe.elf` contains three roles. The separate
process address spaces and their kernel grants determine authority, not the code
or a role field from a request. `nas-volume.elf` builds the **unmodified**
existing `sdk/examples/volume-docs/src/service.rs` with its own linker.

```sh
NOS_BUILD_DIR=build/nas-probe python3 -B services/nas/native-probe/build.py
```

Dedicated targets `target/nas-native/{nas-probe,nas-volume}`; inspected
ELFs and `build-report.json` under `build/nas-probe/users`. Binaries stay
out of Git. The builder uses the existing `inspect_user`, logging and
hash helpers; no second ELF loader or filesystem.

Both images keep startup data `0x40010000`, at most one RW/NX page.
The probe uses the same 64-KiB stack `0x7ffef000..0x7ffff000` and the existing
64-KiB private heap `0x60000000..0x60010000`. Only the NAS role places one
valid `Workspace` in the heap, without heap growth, allocator or second
user. The normal Rust constructor is used; no assumption that
all-zero bits form a valid Rust layout. Core/volume remain `unsafe`-free.
The native adapter has only the documented heap placement,
startup/intrinsics reuse and stack measurement as local unsafe.

The entry marks the pre-mapped stack before the first Rust frame.
On success startup word **448** yields the touched stack high-water mark.
For this only initialized, inactive stack memory below the
current RSP is read; no live Rust borrow, red zone or anyone else's memory.
This is **not** a formal upper bound for unused stack reservations or
error paths not yet executed. Native values remain open until QEMU execution.
The ELF footprint/hash and host measurements are in the accompanying evidence bundle.

## Processes, endpoints and rights

Proposed slots, still to be bound by the root integrator:

| Slot | Role / image | Only functional grants |
|---|---|---|
| 0 | administrator / `nas-probe.elf` | SEND to NAS admin ingress, RECV on own NAS reply, SEND/RECV to guest coordination |
| 1 | NAS / `nas-probe.elf` | two RECV ingresses, two SEND replies, SEND/RECV to own volume domain, entropy source |
| 2 | existing volume service / `nas-volume.elf` | scoped block READ/WRITE/FLUSH (cold READ-only), two own RECV ingresses and fixed SEND replies |
| 3 | guest / `nas-probe.elf` | SEND to NAS guest ingress, RECV on own NAS reply, SEND/RECV to admin coordination |

Only the trusted kernel manifest mints/delegates these existing
process-bound capabilities. No NAS/guest request can create capabilities.
Reply endpoints belong fixed to their actual receiver; there are no
reply capabilities or access rights in the wire. Every receive checks the
actual `Received.sender`, tag, length and absence of capability attachments.
A role field or PID in a payload grants no authority.

NAS owns volume domain 0, object 0. The second volume ingress remains
isolated; assign a different manifest PID and close unused delegated
SEND handles before start. Give administrator/guest no NAS volume, block,
entropy, launch or network grant. Handles are slot/generation **within one process**; equal numbers
in different processes are no proof of identity or delegation.

For this probe `LocalBinding` makes a public `NOSPROC1 + PID` principal
from the actually checked sender. This proves only local
process origin, **not** a cryptographic device identity or persistent
account identity. Manifest PIDs must stably denote the same
roles in the cold probe; product identity/restart delegation is not derived here.
Transport/TLS, peer pins and LAN/internet rights remain NET ownership.

## Startup words

All offsets are bytes in the existing private startup page. Not listed:
zero. The existing volume service uses its **own unmodified** layout.

| Offset | Administrator (role 0) | NAS (role 1) | Guest (role 3) |
|---|---|---|---|
| 32 | 1 | 1 | 1 |
| 48 | 0 | entropy handle | 0 |
| 56 | NAS admin SEND | admin RECV | NAS guest SEND |
| 64 | own NAS reply RECV | guest RECV | own NAS reply RECV |
| 72 | guest coordination SEND | admin reply SEND | admin coordination SEND |
| 80 | guest coordination RECV | guest reply SEND | admin coordination RECV |
| 88 | 0 | volume SEND | 0 |
| 96 | 0 | volume reply RECV | 0 |
| 112 | normal 0, cold 1 | normal 0, cold 1 | normal 0, cold 1 |
| 120 | 0 | 1 | 3 |
| 160 | NAS PID | admin PID | NAS PID |
| 168 | guest PID | guest PID | admin PID |
| 176 | own PID | own PID | own PID |
| 184 | 0 | volume PID | 0 |

The syscall set must include existing IPC, entropy and monotonic clock.
No new syscall, general cap rights or trusted RTC claim.
The initially empty synthetic disk uses the existing
`tools/volume_disk.py::create_image` fixture: 2.048 sectors, of which only
the 580 sectors from sector 4 fall within the volume capability; canaries
outside it. Never use a host disk.
Cold reopening happens on the same hashed image with a READ-only block grant;
expect zero disk mutations. No automatic empty initialization or cleanup.

## Wire v1: NASP / 256 bytes

Tag `0x4e41535000000001`, exactly 256 bytes, little-endian. Coordination has
the separate tag `TAG+1` and a fixed peer check; this is not a tunnel for
NAS or volume authority.

| Bytes | Meaning |
|---|---|
| 0..4 | `NASP` |
| 4..6 | version 1 |
| 6..8 | opcode 1..13; only a reply sets bit `0x8000` |
| 8..16 | non-zero request correlation, not a capability |
| 16..20 | status: 0 or fixed negative error code; requests always 0 |
| 20..22 | payload length 0..104 |
| 22..24 | zero |
| 24..56 | namespace where the opcode requires it, otherwise zero |
| 56..96 | 40-byte session handle where required, otherwise zero |
| 96..136 | 40-byte upload handle where required, otherwise zero |
| 136..144 | expected/returned revision or epoch |
| 144..152 | quota, object index, expiry or enumeration index depending on opcode |
| 152..256 | payload; all bytes after the length zero |

Opcodes: 1 Invite, 2 Accept, 3 Open, 4 Stage, 5 Commit, 6 Read, 7 Usage,
8 SetQuota, 9 Revoke, 10 Evict, 11 Shutdown, 12 UsageAt, 13 Close.
The parser also checks **unused fields per opcode**, token form,
reply form and empty error replies. Session decoding grants no rights:
the core re-checks caller, live handle, namespace and epoch.
No principal, key, mutable client buffer or capability in the wire.

The 104-byte payload and one selected guest are visible probe limits,
not production capacity. The core has other explicit probe caps.
Staging reserves quota and does not confirm durable publication; COMMIT
confirms only after snapshot and history purge. Every storage replacement error
becomes `Unknown`, also on initialization; the original cause remains local
diagnostics. The instance blocks further use. No silent
re-execution, success fallback, old snapshot or ACK before durability.

The monotonic clock is only for offers within the same boot.
The cold probe accepts no old offer on reset PIT ticks.
This is a fail-closed probe limitation, not a finished persistent-pairing UX
or trusted time provider; RTC must not silently replace this.

## Normal and cold assertions

Normal: admin offers 128 bytes; guest accepts and opens, stages/commits/reads
96 synthetic bytes; reserves 32 replacement bytes. Admin sees used 96,
reserved 32, rejects quota 127, revokes without guest consent. Guest gets
`Stale` on old upload/read and `Denied` on reopening. Guest administration via
guest ingress is rejected; receiving via its own SEND-only NAS ingress
gives kernel `Access`. That checks concrete rights, not the incorrect
assumption that a numeric handle is globally unique. Afterwards
admin deletes the ciphertext, Usage shows
`Evicted`/used 0/reserved 0; NAS shutdown only via admin.

Cold: the service restores exclusively the existing canonical snapshot with fresh
real entropy; admin enumerates `Evicted`, epoch 2, used/reserved 0; guest
cannot open, perform no administration or use an old/synthetic
session/upload handle. For this the cold guest probe reads only the
two bounded test inputs below; they are not included in the NAS wire,
snapshot, ordinary logs or reports. Idempotent eviction/shutdown
write nothing.
No content key is delivered to the NAS or administrator.

## Exact boot-1 token replay — leaf requirement, root binding still open

The cold probe accepts no filled-in product credential or home-made
token. Only a private test handoff with the **exact** 40-byte
`Session::to_bytes()` and 40-byte `Upload::to_bytes()` from a completed
normal native guest run is used. The source must be the real `Stage` reply
from that boot. A synthetic marker, arbitrary valid token form,
different PID, `Invitation`, request ID, hash alone or `PIT` reset does not replace this
evidence.

Cold guest startup uses the existing private 1.024-byte startup page;
this introduces no syscall, endpoint, wire version, capability or general
bootstrap ABI:

| Guest startup bytes | Cold input |
|---|---|
| 256..296 | Exact 40-byte normal-run session handle |
| 296..336 | Exact 40-byte normal-run upload handle |

The leaf reads exclusively ten existing aligned 64-bit startup words and
reconstructs their canonical little-endian bytes. `Session::from_bytes` and
`Upload::from_bytes` check only canonical form, not identity or
authority. On a normal run both cold input ranges must be zero; on
cold, missing/zero/invalid input fails before replay as error step 47/48.
Cold gives the original session to `READ` (expects `Stale`) and exactly
`(oude sessie, oude upload)` (old session, old upload) to `COMMIT` (expects `Stale`). The second
native reply proves that the combination yields no commit after recovery; due to
the existing core call order the old upload token is not authorized there separately from
the expired session. An isolated host core probe separately uses
a fresh active session to also reject the old upload token on its own.
Those two outcomes remain labelled as different evidence.

The cold runner must fill the startup attachment exclusively from the private artifact
of the successful normal guest run and must not execute the cold image
without the complete provenance binding. Required manifest: schema 1,
verified source commit, normal origin run ID, role `guest`, origin stage
`normal`, capture stage `normal-after-stage`, `completed=true`,
payload length 80 and SHA-256 of the exact raw payload. The manifest contains no
token bytes. Run directory and artifact are outside the Git checkout, mode 0700/0600,
no symlinks or extra hard links; the artifact is not copied to serial, ordinary
logs, CI artifacts or Git. For the cold boot the runner makes a
separate read-only 0400 input copy, compares SHA-256 before injection and
after the run and rejects any deviation. Here the hash is a provenance/integrity binding,
not a cryptographic identity or product credential.

**Root integration (#110), active since 30 October 2026.** The normal guest
writes the exact pair from its real `Stage` reply (session and the never
committed pending upload) after issuance into startup bytes 256..336. When
the guest exits, only the kernel acceptance harness
(`kernel/src/m2/m3/nas.rs`) reads those ten words, rejects a zero session or
zero upload (`REPLAY_EXPORT`) and writes `NOSRPL01` followed by the 80 bytes
to a dedicated `isa-debugcon` port `0x4f8`. The runner (`tools/test_nas.py`)
connects that port to a pre-created 0600 file in a private
0700 directory outside the checkout, requires exactly one frame of 88 bytes and writes
payload and manifest via `tests/replay_handoff.py` with full source/run/role/stage
binding. Leak check (#119, normal and cold run): before anything from the
QEMU output enters the run directory, the runner searches the exact raw stdout
(COM1 serial) and stderr for every contiguous 16-byte window of each of
the two 40-byte tokens, raw and as hex regardless of case. The reference is,
in the normal run, the debugcon capture, in the cold run the pair bound before
start. On a hit or an unreadable capture the run fails and
`serial.txt` and `stderr.txt` get only a placeholder with the
findings (labels, no bytes), length and SHA-256 of the original bytes.
A capture that is not exactly one frame of 88 bytes is searched in its
entirety; an empty capture exported nothing and remains readable. On a
passed leak check `serial.txt` contains exclusively allowlist markers,
fixed error labels and exactly 20-digit diagnostics. Evaluation uses the
original output; filtering does not make errors invisible to the gate. Fragments
shorter than 16 bytes and other encodings (base64, decimal, hex with
separators, UTF-16) are not detected. For the cold run the
runner validates again, makes a read-only
0400 copy and passes it as `-fw_cfg name=opt/nieuw-os/nas-replay`. The
kernel reads the `fw_cfg` file (signature `QEMU`, unique name, exactly 80
bytes). In the cold run it rejects missing or wrongly framed input
(`REPLAY_INPUT`) and an all-zero pair (`REPLAY_INPUT_ZERO`); it does not compare
the tokens itself with the normal run. In the normal run it rejects any
present or unreadable replay input (`REPLAY_INPUT_IN_NORMAL`). The runner
checks that the payload matches the SHA-256 in the bound manifest,
makes a read-only input copy and compares that hash again after the run. Then
it removes the private directory. Reports, serial and CI artifacts contain only
the SHA-256 of the pair.
Firmware has no device or driver on port 0x4f8 and reads no
`opt/nieuw-os/*` files; no process gets port I/O or a new
capability. Recovery counters are now `[(4,0),(4,5),(0,5)]` for
administrator/NAS/guest: the extra error reply is the `COMMIT` replay.

The host test creates handles exclusively by calling the existing real NAS core API:
a session and upload from a host normal model are kept
in RAM, then core recovery performs a new boot. Read of the old session
fails `Stale`; to test the upload separately a new, active
session is opened and the old upload committed with it. That call too fails
`Stale`, without storage write/ACK. This is a host test; it fills no native
handoff artifact and yields no QEMU result.

Diagnostics: startup word 40 = 1 only after complete role assertions; 240 =
error step; 400 = successful RPCs, 408 = error replies, 416 = wrong caller
(NAS) or SEND-as-RECV rejection (guest), 424 = invalid envelopes, 448 =
touched stack high-water mark. 192 = 1 after successful NAS recovery/initialization.
Every error stops the relevant probe role with exit 1; no green from a
timeout alone. The kernel/root runner must independently validate exit 0 of administrator/NAS/guest, the still
running existing volume service, all counters, disk bytes and the cold
recovery run. The volume service has no new
shutdown opcode and need not exit successfully as a fourth process.

**Open native gates:** the cold run without or with deviating input is
covered only by code and runner host tests, not as a separate native VM run; no claim of
power loss, physical secure erase, anti-rollback, network/TLS, E2EE,
background sync, device rotation/recovery, UI or full P04 acceptance.
