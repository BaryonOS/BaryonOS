# Native model artifact reader contract

This isolated `no_std` experiment depends directly on the shared
`artifact-store-core` package at `crates/artifact-store-core`; it does not
define a second store trait, IPC encoding or kernel ABI. The experimental
VM-AI candidate is Qwen2.5-0.5B-Instruct from Hugging Face
revision
[`7ae557604adf67be50417f59c2c2f167def9a775`](https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct/tree/7ae557604adf67be50417f59c2c2f167def9a775):
`model.safetensors`, Apache-2.0, 988,097,824 bytes, SHA-256
`fdf756fa7fcbe7404d5c60e26bff1a0c8b8aa1f72ced49e7dd0210fe288fb7fe`.
This is a candidate pin, not a download or Dutch-quality approval.

The leaf re-exports the root `ArtifactRef` and `ArtifactDescriptor` types, so
callers use the store-assigned opaque `[u8; 32]` ID and immutable `u64`
generation directly. The old numeric leaf ID is gone; no cast or
digest-derived identity is used. Model repository/revision/license metadata
remains separate from `ArtifactDescriptor { reference, length, sha256 }`.
`open` uses root `open_checked` to compare the requested and authoritative
descriptor and retain the root opaque `ArtifactReadHandle`; there is no
parallel store ABI or caller-supplied `integrity=true` field. Reads use root
`read_bounded`, including its 4 KiB bound, positive-short-read, EOF and
provider-count checks. Dropping the opaque handle delegates cleanup to the
concrete store adapter; the root contract itself supplies no provider.

Before any bytes are exposed to a parser, the verifier hashes a complete read
pass with pinned `sha2 0.11.1` (`default-features = false`). A positive
short-read is hashed and advances by its returned count; zero before declared
EOF, stale generation, corruption, overflow, oversized counts, cancellation,
or digest mismatch aborts and drops the handle. Only a matching full-stream
digest produces a `VerifiedArtifact`, which reads from that same immutable
generation. This proves content identity against the requested SHA-256, not
publisher authority for executable apps or model safety. The catalog that
maps model metadata to a store-issued `ArtifactRef` is not implemented here.
The SHA-2 crate is RustCrypto's pinned MIT/Apache-2.0 no-std implementation;
version 0.11.1 requires Rust 1.88 and has no crate build script. The 0.11
dependency graph replaces `generic-array` with `hybrid-array` 0.4.15 and
`typenum` 1.20.1. The resolved SHA-2 dependency closure has no build scripts,
and the bare-metal dependency tree excludes `libc`. The lockfile also resolves
`cpufeatures` 0.3.1 for x86 runtime dispatch. The scalar bare-metal gate below
selects the portable implementation explicitly with
`--cfg sha2_backend="soft"`; disabling target features alone is not treated as
backend-selection evidence.

Each store read uses the root 4 KiB storage bound and addresses the same opaque
handle and generation. That is not a new IPC wire chunk: the existing 320-byte
IPC transport retains its maximum 256-byte payload, including any protocol
headers. A future adapter must subtract its headers from that bound and
split/stream store reads through the existing transport; this leaf defines
neither an IPC header nor a wire chunk size. The
caller-supplied byte budget is charged for requested bytes before I/O,
including failed attempts, and is not refunded; this bounds retries as well
as successful reads. Output is valid only on success. Stale, corrupt,
truncated, overflowing, out-of-bounds, budget-exhausted and EOF conditions
are explicit errors. Capacity snapshots and provider quota remain a separate
u64-based shared root contract; this leaf defines no product-wide capacity
cap. The store must keep generations immutable and report read failures.

The interface accepts no network or personal-document capability and does not
implement downloading, model installation, the model catalog, storage
capacity, model parsing, or inference. The complete integrity pass followed
by model reads can read artifact bytes twice; its time and I/O cost are
unmeasured. The current document volume is not a model store: its limits are
65,536 bytes per object and 163,840 bytes per domain. Release's shared contract
owns provider-reported `u64` capacity/quota snapshots; this experiment defines
no capacity ABI or product-wide cap.

The 8 GiB VM-AI memory profile remains an investigation target. The pinned
[`config.json`](https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct/raw/7ae557604adf67be50417f59c2c2f167def9a775/config.json)
reports 24 layers, hidden size 896, 14 attention heads, 2 KV heads, BF16
weights and a 32,768-position architecture limit. With head dimension
896/14 = 64 and two-byte KV elements, the K+V cache formula is
`24 * 2 * 64 * 2 * 2 = 12,288 bytes/token`: 48 MiB at 4,096 tokens or
384 MiB at the architecture maximum. Those are arithmetic estimates, not a
chosen product context or a measured runtime. The 988,097,824-byte BF16
candidate file is about 942 MiB; full FP32 expansion would be about 1.84 GiB,
before activations, tokenizer, workspace, and runtime overhead. The 8 GiB
profile's remaining margin is therefore unproven. No model bytes were
downloaded and no resource or inference measurement was made. Host tests use
small synthetic streams to test complete short-read verification, full-hash
rejection and handle cleanup on incomplete verification, descriptor mismatch,
revocation, quota/I/O errors and cancellation before or during a verified read.
These fixtures do not prove model loading, installation, or inference. A
bare-metal build proves only compilation for `x86_64-unknown-none`.

The bounded gates for this leaf, run against root-contract branch
`simon/native-artifact-store-contract` at `7f4f7bf`, are:

```sh
cargo +1.90.0 test --offline --locked --manifest-path experiments/ai/model-store-contract/Cargo.toml
cargo +1.90.0 clippy --offline --locked --manifest-path experiments/ai/model-store-contract/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path experiments/ai/model-store-contract/Cargo.toml -- --check
RUSTFLAGS='--cfg sha2_backend="soft" -C target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2' CARGO_TARGET_DIR=target/ai-model-store-contract-none-sha2-scalar cargo +1.90.0 build --offline --locked --manifest-path experiments/ai/model-store-contract/Cargo.toml --release --lib --target x86_64-unknown-none
```

An earlier `x86_64-unknown-none` build with default target features exited 101
with an LLVM internal error in `sha2`. That failure did not reproduce in two
subsequent isolated default-feature release builds; both and the scalar
release library build above exited 0. This compile-only evidence does not
validate a runtime instruction profile. The host test run passed 4 unit tests
and 18 contract tests; no native OS/QEMU run was performed.

The lifecycle follow-up starts from frozen adapter commit
`b51a8093320e6182afce9b189e89760fc8c99116`; it changes tests and this README
only. Production provider/catalog integration remains unassigned to this leaf.
The next native dependency is a store-issued reference and generation bound to
an authorized read handle with provider-owned cleanup and durable catalog data.
It must not be replaced with a test-generated identity or a model download.

Lifecycle follow-up, 30 October 2026: the `test`, `clippy` and `fmt -- --check`
commands above and `git diff --check` exited 0 (4 unit and 18 contract tests).
An initial formatting check exited 1; rustfmt corrected it before the successful
rerun. Bare-metal compilation and native/QEMU execution were not repeated for
this tests/documentation-only change. The earlier compile results remain bound
to the adapter source, not a new inference or runtime acceptance claim.

Dependency update verification, 10 November 2026: the pinned `sha2 0.11.1`
tree passed the 4 unit and 18 negative-contract tests, warning-denying Clippy,
formatting, and the explicit software-backend bare-metal build above. Strict
preflight, the standard OS build, and QEMU smoke also passed. No model artifact
was downloaded, and no native model-store execution or performance claim was
established.
