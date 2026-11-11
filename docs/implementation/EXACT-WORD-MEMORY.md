# Exact bounded x86-64 memory intrinsics

2 November 2026; base `ecb9964de727561a900ed93bc86ad3d5847dfe6b`.
Only `kernel/src/memory.rs`, a targeted host harness and this note. No
snapshot, capability, syscall, cache or user interface change.

The real window session gave a serious slowdown signal with the new
exact snapshots. Overlapping VM load prevents a precise ratio.
One possible cause is repeated byte-wise emulation in TCG; this is not yet a
demonstrated attribution of all the slowdown.

## Contract

`memcpy` requires the existing C intrinsic preconditions: `n` readable and writable
bytes in valid, non-overlapping normal RAM regions. The routine copies
exactly `n/8` full words with `rep movsq`, followed by exactly `n%8`
bytes with `rep movsb`. x86-64 permits this unaligned word access. RSI,
RDI and RCX are explicit inout registers; the tail register cannot overlap with these
early-clobbers. There is no overread, overwrite or read-ahead
padding. The original destination pointer is returned. `cld`
also runs at length zero.

`memcmp` only reads a full word if at least eight valid bytes
remain. Inline MOV instructions prevent an optimizer-generated recursive
memcmp call. On a difference, XOR plus `trailing_zeros` on little-endian
x86-64 determines the **first** differing byte in address order. The result is the
difference between unsigned bytes, not a numeric comparison of whole words.
The remaining zero to seven bytes are read individually. This routine too
leaves DF=0. No SIMD instructions or CPU extensions are added.

`memset` and the overlap-safe `memmove` are unchanged. This does not widen their
contract; in particular the existing zero-length path of `memmove` does not clear DF.
No MMIO, shared-memory or atomicity guarantee has been added.

## Verification and evidence boundary

`tools/tests/test_kernel_memory.py` compiles the **actual** kernel source with
Rust1.90/O3 and runs five Linux/x86-64 tests. Passed: all64 combinations
of source/destination alignment0..7 at lengths0..1025, original pointer,
canaries, read-only source, actually inaccessible guard pages and exactly
placed range ends. Lexical tests vary every first differing
byte position, unsigned80/7f and00/ff, with an opposite following difference.
A single asm trampoline keeps STD→CALL→flags measurement→CLD together; Rust
resumes exclusively with DF=0. Overlapping memmove ranges are tested separately.

A second agent read the actual diff and checked Rust1.90/O3
assembly/IR: correct clobbers, correct stack alignment in the test trampoline,
no live redzone storage and no back-reference from the lexical test oracle
to the tested memcmp. No concrete blocker found; this is not a human
audit or formal proof.

Native release build/ELF inspection and the targeted scalar instruction check are
stored with the commit in `build/word-memory/logs/`. Real M4, network and
window gates follow on the integrated source. No native
speed gain is claimed here.

A separate, identically instrumented scalar QEMU trial of snapshot,
exactEq and mandatory clearing produced **no** valid measurement: the baseline stopped
before M2:PASS. The red reports and generated sources remain stored under
`build/word-memory/benchmark/e8b28780e470459e89b5f976eec03fd3/`. A
bounded diagnostic repetition already rejected the missing
code leaf before the CR3 switch; neither outcome is a successful benchmark. The experimental
driver lives outside the production source, under `build/word-memory/`. The relevant
follow-up measurement is the full real window session, with the same pixel oracles and
without widened security checks.

Rollback is a targeted revert of these two intrinsics. The snapshot and
error atomicity checks were not changed in this step.
