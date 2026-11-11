# I01 — bounded native qsim port

This adapter builds the existing `qsim-core` as a separate Rust program
for `x86_64-unknown-none`. The simulator algorithms remain unchanged. The
program has no block, document, network or administration rights. It performs
the computations in ring 3 as soon as the opt-in kernel test manifest loads this
ELF. A successful `build.py` is solely compilation/artifact evidence;
native execution gets separate QEMU evidence from the integrator.

## Scope

- At most six qubits, 64 gates, 4,096 shots per job.
- The existing simulator seed remains deterministic and is not entropy or
  a cryptographic randomness source.
- The existing `Control` interface passes on cancellation and a bounded
  work budget. The test exercises deterministic deadline injection; there is
  no new native clock or IPC cancel capability.
- The allocator uses only the existing private 64 KiB process heap.
  In this finite test it returns memory only at process exit. This is
  not yet an allocator for a permanently reusable quantum service.
- All heap requests use checked additions and alignment.
  The test checks both a refused allocator request and the real
  `ResourceExhausted` outcome of the existing simulator core.
- `f64` is executed with scalar instructions by Rust's existing target/compiler builtins.
  The kernel does not need to add any x87/SIMD context.
  `build.py` checks the decoded full `.text` and rejects x87,
  MMX, SSE, AVX, opmask registers and unknown instruction lines. This is an
  artifact check, not a formal proof or physical hardware qualification.

## Reproduction

Required: Rust 1.90.0 with `x86_64-unknown-none`, `rust-lld`, Python 3,
GNU `objdump` and `nm`. There are no new external Rust dependencies. The
package's own `Cargo.lock` contains only this adapter and the existing local core.
Use a separate target and output directory per concurrent builder.

```bash
CARGO_TARGET_DIR=/tmp/nieuw-os-i01-target cargo test --manifest-path experiments/quantum/native/Cargo.toml --locked --offline
CARGO_TARGET_DIR=/tmp/nieuw-os-i01-target cargo clippy --manifest-path experiments/quantum/native/Cargo.toml --locked --offline --all-targets -- -D warnings
python3 experiments/quantum/native/tests/test_build_gate.py
python3 experiments/quantum/native/build.py --output-dir /tmp/nieuw-os-i01-build --target-dir /tmp/nieuw-os-i01-target
```

`build-report.json` binds source hashes, commands, Rust version, ELF segments,
binary hash and disassembly hash. `native_runtime_performed` remains
explicitly `false` in it: this task does not start an emulator. The ELF is called
`qsim-native-probe.elf`. Existing kernel/boot/capability interfaces are
not changed by this directory.

## Native test interface for the integrator

Entry `0x40000000`, private RW report page `0x40010000`, separate NX rodata
from `0x40011000`. The existing 64 KiB heap starts at `0x60000000`.
The kernel manifest initialises case to zero and a pattern of its own. Only
`abi_info` and `yield` of the existing `int 0x80` ABI are called.
After completion the program keeps yielding, so that the kernel can assess the outcome and
the progress of other processes.

| Offset | Meaning |
|---:|---|
| +0 | Kernel case; only 0 supported |
| +8 | App checkpoint counter |
| +16 | Kernel pattern; left unchanged by the app |
| +24 | Reserved; left unchanged by the app |
| +32 | 0 busy, 1 completed, `0x100 + code` on error |
| +40 | Successful scenarios as a bitmask; requires 127 |
| +48 / +56 | Bell4096, seed 0: count0=2082, count3=2014 |
| +64 | Bell shots: 4096 |
| +72 | Arena bytes used before the exhaustion test; 1..65536 |
| +80 | Largest work distance between checkpoints; at most 448 |
| +88 | Six-qubit X circuit: count63=4096 |
| +96 | Arena reservation after checked exhaustion: 65536 |
| +104 | Real core returns `ResourceExhausted`: 1 |

The seven scenarios are Bell, six qubits, cancel beforehand, cancel during
execution, injected deadline, invalid input limits and heap exhaustion.
The kernel must compare their values independently and treat user errors,
timeout and unexpected termination as failure. A copied app field alone
is not an independent kernel acceptance claim.

## Next steps

A real quantum service requires a reusable allocator or an explicit
job lifetime, bounded IPC input, a revocable cancel object and
process-bound quota. No physical QPU, quantum speedup, local AI,
NAS execution or cloud fallback has been added in this step. The fixed
roadmap awards no point solely for compilation or host tests.

Rollback: omit the opt-in quantum feature from the kernel test manifest;
the existing QEMU profiles and host simulator remain available.
