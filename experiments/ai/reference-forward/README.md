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
execution or numerical quality on the project's OS. The approved engine/model route and
the full model graph are described below (L01-T5/T6); native execution, the tokenizer
and task-quality acceptance remain open.
CI coverage must be added with integration; the existing shared workflow is owned
by pending CI PR391 and is deliberately not changed by this independent leaf.

RMS normalization and SwiGLU are additional numerical building blocks; the full
model graph is described under L01-T6. RMS rejects overflowing square accumulation explicitly.
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

## Attention, RoPE and KV cache (L01-T4)

`rope` applies rotary position embedding to one head in the rotate-half layout
of Llama/Qwen2-style checkpoints. `KvCache` stores one layer's keys and values in
caller-owned buffers and refuses a token once it is full (`ContextLimit`); it
never evicts, truncates or allocates. `attention` computes causal grouped-query
attention of one new query over every cached token with max-subtracted softmax
and a `1/sqrt(head_dim)` scale. Attention work is charged to a per-request `Budget` (with cooperative `Cancel`);
output is published only on success. `kv_cache_bytes` applies the proposal's formula
`2 × layers × KV_heads × head_dimension × tokens × bytes_per_element` with checked
arithmetic against the proposed 128 MiB ceiling (`MAX_KV_BYTES`), before any
allocation. That ceiling is a proposal value, not an approved or measured budget.

Tests compare against hand-computed rotations and softmax weights. Host tests and
an `x86_64-unknown-none` release library build were the evidence for this step.
(Historical: at this step the pinned configuration was still unread. It has since
been inspected (L01-T5), and the multi-layer graph with attention biases is wired
and compared with the real model (L01-T6). The tokenizer is not implemented.)

## Decoder layer, work budget and pinned configuration (L01-T5)

`DecoderLayer` composes one Qwen2-style layer: RMSNorm → Q/K/V (with biases) →
per-head RoPE → KV append → grouped-query attention → O → residual → RMSNorm →
SwiGLU → residual. Weights are borrowed `Bf16` tensors read in place and widened
per element (lossless, not quantization): an f32 copy of the pinned model would
need about 1.84 GiB and not fit the approved 2 GiB VM, BF16 needs 942 MiB. A
`Budget` charges every row before it runs and polls `Cancel`; exhausting it or
cancelling returns `WorkLimit`/`Cancelled`. On any failure the token's K/V append
is rolled back and the hidden state is unchanged; a full cache is refused.
The fixed `MAX_MACS` still bounds the original f32 `Dense` slice only, because one
pinned projection (4,864 × 896) already exceeds it.

`tests/decoder.rs` checks three tokens against an independent f64 reference.
Two mutants (no rollback; wrong row offset) each fail a test.

Pinned configuration, read at revision `7ae557604adf67be50417f59c2c2f167def9a775`
(owner approved 25 November 2026): `Qwen2ForCausalLM`, 24 layers, hidden 896,
14 heads, 2 KV heads, head dimension 64, intermediate 4,864, RMS ε 1e-6, RoPE θ
1,000,000, vocabulary 151,936, tied embeddings, BF16, no sliding window. The
safetensors header (32,280 bytes) lists 290 BF16 tensors totalling 494,032,768
elements; with the header that is exactly the pinned 988,097,824 bytes.

| File | Bytes | SHA-256 |
|---|---|---|
| config.json | 659 | `18e18afcaccafade98daf13a54092927904649e1dd4eba8299ab717d5d94ff45` |
| generation_config.json | 242 | `e558847a8b4402616f1273797b015104dc266fe4b520056fca88823ba8f8ebe6` |
| tokenizer_config.json | 7,305 | `5b5d4f65d0acd3b2d56a35b56d374a36cbc1c8fa5cf3b3febbbfabf22f359583` |
| tokenizer.json | 7,031,645 | `c0382117ea329cdf097041132f6d735924b697924d6f6fc3945713e96ce87539` |
| vocab.json | 2,776,833 | `ca10d7e9fb3ed18575dd1e277a2579c16d108e32f27439684afa0e10b1440910` |
| merges.txt | 1,671,839 | `599bab54075088774b1733fde865d5bd747cbcc7a547c5bc12610e874e26f5e3` |
| LICENSE | 11,343 | `832dd9e00a68dd83b3c3fb9f5588dad7dcf337a0db50f7d9483f310cd292e92e` |
| model.safetensors header | 32,280 | `bd808652be103f0e4e29786b9aa8829e0f38607880486926c7246ee2de577a76` |

Tokenizer facts: byte-level BPE, 151,643 vocabulary entries plus 22 added tokens
(valid ids below 151,665), 151,387 merges, NFC normalization, a Unicode-category
split regex (`\p{L}`, `\p{N}`) and the ChatML template. A native tokenizer must
reproduce reference ids exactly and refuse input it cannot handle.

Native limit found: `x86_64-unknown-none` is soft-float, as is the accepted L02
qsim job. One token costs roughly 360 million multiply/accumulates (about 500
million when generating, with the tied 151,936 × 896 head), so soft-float native
inference is expected to be far too slow; enabling SSE for the AI process is a
new CPU-feature decision. This is an estimate, not a measurement.

## Full model and Candle oracle (L01-T6)

`Model` runs a token through the embedding, every `DecoderLayer` and the final
norm with the tied output head; a failure in any layer rolls back every cache.
`tests/qwen_oracle.rs` loads the real pinned model through
`open_pinned_model` (length and SHA-256 verified), parses it with the repository
`model-format` parser, reads tensor data through the verified artifact and
compares with the Candle fixture: the top-10 final logits for the first prompt
match within 1e-2 and two greedy steps give the same tokens. Run it with
`NOS_QWEN_DIR=<model dir> cargo +1.100.0 test --release --locked --test qwen_oracle`
(2.8 s, 1.9 GB peak RSS on the development host); without the variable it is
skipped and proves nothing. Reversing the RoPE rotation makes it fail.
This is host evidence of numerical agreement only; native execution, the
tokenizer and task quality remain open.
