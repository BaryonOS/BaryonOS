# Native memory growth test

This architecture test performs real ABI1.7 memory growth in ring3 on
the own kernel. The normal route uses `native::heap_grow`; only the
seven deliberately invalid arguments use an explicit test `int80`.
`Vec`, `Box` and `String` use the maintained `sdk/heap-allocator`, exclusively
on top of a new dedicated RW/NX arena. There is no host allocator or automatic
memory growth. Every new byte is read and checked for zero before being handed
to the allocator.

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools \
CARGO_TARGET_DIR=build/sdk-memory-probe-rust190 \
python3 -B sdk/examples/memory/build.py
```

The build explicitly uses Rust1.90.0, its own pinned lockfile and
`x86_64-unknown-none`. The JSON result gives `NOS_MEMORY_ELF` and the actually
inspected ELF/hash. All processes use the same executable with their own
frames and a separate private start page. The protocol is in
`src/contract.rs`; it is test metadata, not a new public startup contract.

The main agent integrates feature selection, ELF embedding and
`kernel/src/m2/m3/memory_fixture.rs` via the shared build and VM procedures.
The features are `native-memory` and the derived `native-memory-nx`,
`native-memory-guard`, `native-memory-peer`, `native-memory-oom`.
`native-memory` requires `process-memory` and `fpu-context`. Legacy profiles
remain separate. No other M3/M4 demo feature belongs in this fixture.

Normal, NX, guard and peer use two processes on the existing
pc-q35-8.2/qemu64/TCG/1CPU/256MiB regression profile. Three lives include a
new PID in the reused first process slot. Every life checks
513 pages of growth across a page table boundary, argument errors, a refused
oversized growth followed by exactly contiguous successful growth, the existing
total 2048-frame quota, its own content and the preserved old heap. In negative
cases only the first process then executes a real faulting
instruction: NX `#PF` with error21, writing the unwritten suffix with error6,
or reading a physical frame of the other live process with error5.
The observer checks vector14, error code, CR2 and RIP and lets the other
process demonstrably continue working before the slot is reused.

OOM is a separate negative 128MiB profile with sixteen processes.
Each requests bounded portions of64 pages until the real kernel reports Quota or
OutOfMemory and keeps those frames occupied. No fixture artificially withdraws
frames from the pool. At least one process must report real physical exhaustion.
Only when all sixteen are blocked may fifteen exit. The
remaining OOM process must then receive one new zero-filled page exactly at
the old suffix boundary. This does not widen the existing quota.

Before process cleanup the kernel observer checks the actual owner,
RW/NX/user PTE, supervisor alias, alias counts and the unwritten suffix boundary.
Directly after cleanup and before reuse it checks all released
heap frames independently: ownerFree, no aliases/PTE and all bytes zero.
Permanent new kernel page tables are explicitly subtracted from the recovered
frame balance and counted separately.

The nineteen `MEMORY:` counters are fixed in order in the observer.
For every case:

- `ZERO_BYTES = GROWN_PAGES * 4096`;
- `CLEANUP_ZERO_BYTES = ZERO_BYTES + CLEAN_STARTS * 65536`;
- `RETRY_ZERO_BYTES` is already included in `ZERO_BYTES`;
- `ARG_DENIED = CLEAN_STARTS * 7`, all collection/private checks once
  per life, at least two real PIT interrupts;
- no live processes and `FRAMES_RECOVERED = 1`.

Normal and fault tests require three clean lives,21 argument refusals,
6 quota refusals and one slot reuse. Normally three processes exit;
a fault test counts one fault and two normal exits. OOM requires sixteen lives,
sixteen exits,112 argument refusals, at least one physical OOM, one recovery,
4096 recovered zero bytes and no slot reuse. The number of physical OOMs plus
quota refusals must be sixteen; package layout or VM allocation details may
not be used to precook this distinction.

A source-bound successful VM test proves only these concrete properties
on this regression/negative profile. The complete 8GiB product release,
arbitrary application compatibility, physical hardware, SMP, heap shrinking and
shared memory buffers remain separate acceptance criteria. Moreover, the local
development tests with temporary integration hooks are not evidence
for a different composed main branch commit.
