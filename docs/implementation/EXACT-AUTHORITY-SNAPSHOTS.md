# Exact authority snapshots

2 November 2026; implementation baseline `4638c7c`. Bounded optimisation of
the existing M4 and IPC-wait checks. No new permissions, mutation cache, UI
acceptance or U02 points.

Before each request, `m4::syscall` calculated complete `Stats`; an error
required a second scan. The diagnostic digest traverses approximately 74,569
typed words, including unused queue cells. `State.audit()` is specifically
absent from this ordinary IPC path. The existing `stats()`, `state_digest()`,
reference digest and all reporting remain unchanged fresh scans.

## Invariant and storage

The new `AuthoritySnapshot` contains a private, complete `Engine`. A
preallocated kernel static receives all ten `Copy` arrays and `last_pid`
directly through `snapshot_into`. Exhaustive destructuring without `..` and
`deny(unused_variables)` cause field growth to fail compilation until the
copy is deliberately extended. The existing derived `Engine::eq` compares
all typed fields, including generations, dead slots and inactive message
cells. Padding is not compared. This error check has no digest collision or
dependency on mutation hooks; the copy is never restored or used as active
authority.

On the checked x86-64 profile, the snapshot costs 461,816 B, alignment 8.
`AuthoritySnapshot::new()` exists only in static kernel storage, not as a
large return value, clone or temporary `Engine` on the 16 KiB service stack.
The scalar `native-ipc-wait` build shows 40 B of stack space plus 48 B of saved
registers for `snapshot_into`; the wiped snapshot lease uses 344 B plus 48 B.
This is inspection of these functions, not a complete maximum kernel stack
analysis.

One shared `SnapshotLock` provides a non-blocking exclusive lease. The kernel
checks IF=0 and acquires the lease before a mutable buffer borrow. Re-entry
stops; it never reuses a snapshot that is still active. No borrow of the live
`Engine` survives `dispatch`. M4 compares on every error; IPC-wait compares
on **every** result, including successful parking. All existing pointer,
capability, return and separate user-output checks remain present.

A snapshot temporarily duplicates IPC content. Before release, the kernel
therefore overwrites all 1024 copied `Message` values with a valid empty
message through typed volatile writes: headers, descriptors and all payloads.
No Rust struct padding is interpreted. This also occurs before an assertion
about a detected authority change: panic-abort does not execute Drop. The
lease is then released, before observers or scheduling. Node/object metadata
that does not contain content remains private kernel storage and is
overwritten at the next capture. The wipe adds 320 KiB of bounded writes per
observation; these costs must not disappear from a performance measurement.

## Targeted verification

`cargo +1.90.0 test --offline --locked -p capability-core`: 66 tests passed,
one existing benchmark ignored, plus 7 IPC-wait tests passed. Five new tests
compare snapshots with complete host clones and fresh oracles, change every
authority field including slot 0 and the last inactive queue cell, check error
paths and unexpected writes, erase all message fields and reject re-entry.
The clones exist only on a large host-test stack. Clippy for capability-core
with `--all-targets -- -D warnings` passes. An additional kernel Clippy check
with `--features native-ipc-wait --target x86_64-unknown-none -- -D warnings`
stops on seven existing diagnostics in unchanged `m3.rs`, `paging.rs` and
`main.rs` (useless_conversion, needless_return, manual_is_multiple_of,
too_many_arguments and not_unsafe_ptr_arg_deref). No diagnostic concerns this
diff; the native release build and QEMU gate pass. Full strict kernel Clippy
is therefore not reported as passing; log: `kernel-clippy.txt`.

The existing `native-ipc-wait` kernel and ELF inspection build. The real QEMU
run passes: raw exit 33, runner exit 0, 5.22344 s, both phase masks 63/60,
14 blocked waits, 7 ready wakes, 6 timeouts, 1 error wake, 181 idle IRQs and
complete cleanup. Profile: QEMU 10.0.13/q35-10.0/TCG/qemu64/1 CPU/256 MiB,
no network or physical disks. Source-v3:
`2cc2e7604a669660fd6b79ec70f8c9af8b4586f060b8330a258b01f7fc7dc73f`;
kernel SHA256:
`4104dba1c521142325d74de57ea06f6903adefadc8bc37e287d595950b89756b`.
This demonstrates native IPC/wait progress within this gate; it does not
measure comparable syscall latency, desktop latency or the complete U01 chain.

Execution from the dedicated worktree, with the existing activated toolchain:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools \
NOS_BUILD_DIR="$PWD/build/exact-snapshot/native" \
NOS_TARGET_DIR="$PWD/build/exact-snapshot/native-target" \
python3 -B tools/run_ipc_wait.py
```

Raw logs: `build/exact-snapshot/logs/{host,clippy,native-wait}.txt`;
source/artifact-bound report: `build/exact-snapshot/native/ipc-wait-report.json`.
The independent second-agent diff review found two issues that were corrected:
erasing before a possible panic-abort and removing an incorrect repr(C)
comment claim. Rechecking found no remaining blocker within this scope;
this is not a human audit.

A preliminary host trial with 9×128 repetitions on eight processes, one
endpoint and eight messages produced medians of 114,102 ns for one Stats scan,
214,314 ns for two scans, 45,032 ns for snapshot+wipe and 134,736 ns for
snapshot+exactEq+wipe. It used Rust 1.90/O3, the project's kernel memory
intrinsics and scalar-generated trial code; the host target warns that SSE2
is normally part of its ABI. This is only a research signal, not a reliable
native latency or hardware comparison. Raw trial files are temporarily under
`/tmp/nieuw-os-authority-snapshot-design/`; appropriate acceptance relies on
the real native gates and source-bound artifacts.

## Limits and rollback

This covers only the existing single-CPU/IF0 contract. SMP, re-entrant
callbacks or new interrupt paths require redesign. A failing IF0 check or
hardware/kernel crash can stop the VM; it grants no new authority. The storage
is never a userspace mapping or DMA buffer. Rollback restores only the M4/wait
Stats comparisons; the old fresh oracles remain available. There is no
persistent format migration.

## SW-1 — direct volatile word wipe

Addition on source `03e81d4`, 2 November 2026. The native proof and stack
figures above belong to the first snapshot implementation. Its real window
session became slower. Inspection of the exact live release ELF showed that
`write_volatile(Message::new())` first initialises 288 stack bytes per message
through the still bytewise `memset`, copies those 288 bytes and then writes
headers. Wordwise memcpy does not resolve those temporary initialisations.

Only erasure changes now: `snapshot_wipe::wipe_message` writes 40 aligned,
individual volatile `u64` zeros directly to the exclusively borrowed Message
object. The same 320 bytes per message and 1024 messages are fully erased
before the existing lease release and before a possible failure-atomicity
assertion. There is no new authority, cache or changed comparison, and no
Message temporary, callback or padding read. The unchanged capability,
pointer, return and output checks remain in force.

The constant size 320, Cap size 16, alignment and sum of field sizes are
checked. Exhaustive Message and Cap patterns without `..` pin all primitive
**values** and array lengths. Deref coercions therefore cannot silently accept
new wrapper types. Because only integer/byte fields are written, every
partial zero-write step also remains a valid Rust representation; the
snapshot is exclusive throughout the wipe.

Targeted evidence: two Python checks pass. The actual helper passes four
Rust/O3 tests covering complete headers/descriptors/payloads, access bounds
with inaccessible guard pages, canaries and neighbouring messages, plus all
1024 snapshot messages with the lease retained. Five separate negative
compilation tests reject Message/Cap growth, NonZero fields and a coercing
wrapper; an unchanged schema check compiles. Canary tests check their own
alignment precondition before any potentially invalid cast use.

The dedicated scalar native-ipc-wait release build and ELF inspection pass;
no VM has been started for this candidate. The actual release IR contains
40 `store volatile i64 0, align 8` operations per message, with no
memcpy/memset in the normal wipe/release path. Only the existing IF0 error
formatter, unreachable after panic, has a 16-byte LLVM memset. Release
assembly has no copy/wipe helper call in the wipe path, no vector registers
and 56 B of stack space for Drop. The lease is released after the complete
loop. This replaces the previous 344 B plus 48 B register save.

Logs and IR/assembly: `build/snapshot-word-wipe/logs/`; checked kernel:
`86279bc94f08d5898a1b8db887df3b277d85e937c3385b85b22a24b31343a283`.
Independent code/IR review and the final integrated native window test
determine acceptance. Compilation and fewer instructions do not demonstrate
actual latency improvement; no U01/U02 points are awarded.

The final independent review read the actual IR and release assembly and,
after strengthening by-value type checks and test alignment, found no
remaining blocker within this diff. Definitive source-v3:
`622d1f5a0e329c701656eb8b478d98e79fd2804bae6993c1ec21780b32f86818`.
Helper SHA256:
`4202f5644d61fa9118cd924e96b301986a54ff68f69a46583a3ef12120d39987`;
M4 SHA256:
`8680502f9deae2f71a4c363ff7f4eed3f87f324ed2b1941c231e6e7a34b8b183`.
The rebuilt source-bound manifest refers to exactly the same kernel bytes
as the inspected IR/assembly. No candidate VM or latency claim.
