# Native model artifact reader contract

This isolated `no_std` experiment adapts the shared read-only artifact-store
contract; it does not define an IPC encoding or kernel ABI. The experimental
VM-AI candidate is Qwen2.5-0.5B-Instruct from Hugging Face
revision
[`7ae557604adf67be50417f59c2c2f167def9a775`](https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct/tree/7ae557604adf67be50417f59c2c2f167def9a775):
`model.safetensors`, Apache-2.0, 988,097,824 bytes, SHA-256
`fdf756fa7fcbe7404d5c60e26bff1a0c8b8aa1f72ced49e7dd0210fe288fb7fe`.
This is a candidate pin, not a download or Dutch-quality approval.

`ArtifactRef` directly uses the store-assigned opaque `[u8; 32]` ID and its
immutable `u64` generation. The old numeric leaf ID is gone; no cast or
digest-derived identity is used. Model repository/revision/license metadata
remains separate from `ArtifactDescriptor { reference, length, sha256 }`.
`open` passes that untrusted descriptor to the store for authoritative
comparison and receives an opaque handle. There is no caller-supplied
`integrity=true` field.

Before any bytes are exposed to a parser, the verifier hashes a complete read
pass with pinned `sha2 0.10.9` (`default-features = false`). A positive
short-read is hashed and advances by its returned count; zero before declared
EOF, stale generation, corruption, overflow, oversized counts, cancellation,
or digest mismatch aborts and drops the handle. Only a matching full-stream
digest produces a `VerifiedArtifact`, which reads from that same immutable
generation. This proves content identity against the requested SHA-256, not
publisher authority for executable apps or model safety. The catalog that
maps model metadata to a store-issued `ArtifactRef` is not implemented here.
The SHA-2 crate is RustCrypto's pinned MIT/Apache-2.0 no-std implementation
with no crate build script or optional assembly feature. Its
`generic-array` dependency has a build script that only checks the rustc
version. The bare-metal dependency tree excludes `libc`; the lockfile also
resolves a target-specific `cpufeatures` edge for OS targets that use it.

Each store read is bounded to 4 KiB and addresses the same opaque handle and
generation. That is a storage API bound, not a new IPC wire chunk: the existing
320-byte IPC transport retains its maximum 256-byte data payload, so any future
IPC adapter must split/stream store reads through that existing layer. The
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
a short tampered synthetic stream to test full-hash rejection and negative
reader cases; they do not prove model loading, installation, or inference. A
bare-metal build proves only compilation for `x86_64-unknown-none`.

The bounded gates for this leaf are:

```sh
cargo +1.90.0 test --offline --locked --manifest-path experiments/ai/model-store-contract/Cargo.toml
cargo +1.90.0 clippy --offline --locked --manifest-path experiments/ai/model-store-contract/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path experiments/ai/model-store-contract/Cargo.toml -- --check
RUSTFLAGS='-C target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2' CARGO_TARGET_DIR=target/ai-model-store-contract-none-sha2-scalar cargo +1.90.0 build --offline --locked --manifest-path experiments/ai/model-store-contract/Cargo.toml --release --lib --target x86_64-unknown-none
```

In this Rust 1.90.0 environment the `x86_64-unknown-none` build with default
target features exited 101 with an LLVM internal error in `sha2`; the scalar
release library build above exited 0. That is a restricted compile check, not
an accepted runtime instruction profile. The host test run passed 3 unit
tests and 12 contract tests; no native OS/QEMU run was performed.
