# Bounded f32 reference forward slice

Host numerical building block for L01-T3, not a transformer or accepted inference
engine. A borrowed row-major dense layer validates shapes and finite parameters;
two layers with ReLU use caller-owned scratch and publish output only after all
operations succeed. Combined multiply/accumulate work is capped at 1,048,576.
Arithmetic overflow/nonfinite values fail; scratch may retain partial intermediates.
Separate multiply/add is the reference convention; results are not promised bitwise
identical across different floating-point environments/backends.

The exact little-endian binary32 decoder validates all values before publishing.
It establishes neither artifact authenticity nor permission. A future loader must
use the existing verified-artifact/descriptor path; no model or tokenizer is loaded.
No allocator, unsafe code, CPU profile or grant is added. The pinned numerical
dependency is documented below.

Run locked tests, Clippy, fmt and release library build with Rust1.100. A successful
no_std library build does not prove floating-point linkage, context saving, native
execution or numerical quality on the project's OS. Transformer attention/RoPE/KV,
real tensor graph, approved engine/model route and native acceptance remain open.
CI coverage must be added with integration; the existing shared workflow is owned
by pending CI PR391 and is deliberately not changed by this independent leaf.

RMS normalization and SwiGLU are additional numerical building blocks; no full
model graph is implemented. RMS rejects overflowing square accumulation explicitly.
SwiGLU evaluates sigmoid with a sign-dependent expression to avoid exp overflow.
Both commit output only on success and bound caller scratch/work.

Dependency: libm exactly0.2.16, MIT, Rust compiler-builtins upstream (not archived;
upstream push11November2026 checked). The crates.io archive SHA-256 is
b6d2cec3eae94f9f509c767b45932f1ada8350c4bdb85af2fcab4a3c14807981.
The published manifest, licence, build.rs and configure.rs were inspected before
addition: no runtime/build dependencies; the build script reads build environment
and emits cfg/env directives, without file/process/network operations. libm is
no_std. Default architecture/intrinsic paths are disabled and force-soft-floats is
explicit. These checks do not prove native linkage, FP context preservation or
all numerical properties. Source: https://github.com/rust-lang/compiler-builtins.
