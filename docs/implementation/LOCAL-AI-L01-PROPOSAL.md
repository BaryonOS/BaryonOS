# First local AI experiment

**APPROVED by the project owner on 25 November 2026** for the first experiment: the
task set, the Qwen2.5-0.5B-Instruct pin, the Candle route and the initial 2 GiB VM
budget. These limits apply to the first experiment only and are not permanent
product limits. On the same day the project owner approved SSE2 hardware floating
point for the native ring-3 AI process as the initial accelerated baseline (not a
ceiling); see "Numeric profile" below.
Baseline main `69d82075`.

## Preserve the existing candidate and authority path

The [model-store contract](../../experiments/ai/model-store-contract/README.md)
already pins Qwen2.5-0.5B-Instruct revision
`7ae557604adf67be50417f59c2c2f167def9a775`, Apache-2.0 candidate,
988,097,824-byte model.safetensors, SHA-256
`fdf756fa7fcbe7404d5c60e26bff1a0c8b8aa1f72ced49e7dd0210fe288fb7fe`.
This proposal preserves that candidate rather than selecting a second model.
Provenance, Dutch quality, tokenizer/configuration identity and native usability
are not approved by that pin. The revision-specific configuration endpoint was
unavailable during this evaluation; no layer count or observed KV usage is claimed.

Use the existing `VerifiedArtifact` path and the
[shared artifact contract](ARTIFACT-STORE-CONTRACT.md): verify a complete immutable
generation before parsing, retain the same opaque handle, and fail on stale
reads/corruption/cancellation. Integrity is separate from publisher/model safety.
The standalone safetensors parser delivered in PR374 is not an inference engine
or acceptance of this real model; native store/IPC and tokenizer/configuration
binding still need implementation. No remote executable tokenizer or model code.

## Proposed first tasks and engine experiment

The [product contract](PRODUCT-SPEC.md) offers local AI on an explicit selection
for shortening, clarification, summarization, comparison and translation. A first
bounded trial should test summarizing a selected paragraph and clarifying its
wording. Other required tasks stay on the product roadmap; the trial does not
narrow the product scope. Normal editing remains usable without AI.

The [Candle primary project](https://github.com/huggingface/candle) is a Rust
reference-engine candidate (MIT OR Apache-2.0), not a demonstrated no_std/Baryon
port. Pin/review a release, selected features, build scripts, tokenizer and CPU
requirements before any host reference build. A native route additionally needs
a bounded allocator and supported numeric/runtime operations; SSE2 is the only
approved CPU extension and no permission expansion is authorized here. Do not replace the OS with a host/Linux
runtime or implement an unreviewed model engine merely to obtain a demo.

## Proposed VM-AI resource bounds

| Resource | Initial experiment ceiling; not a measured requirement |
|---|---|
| Guest | 2 GiB RAM, one vCPU, no GPU or new CPU feature |
| Artifact | Existing exact byte count/digest; sequential bounded verified reads |
| Context | 1,024 total tokens including at most 128 generated tokens; explicit refusal/selection reduction if exceeded |
| KV cache | 128 MiB fixed ceiling; validate `2 × layers × KV_heads × head_dimension × tokens × bytes_per_element` with checked arithmetic before allocation |
| Other engine/tokenizer scratch | 512 MiB ceiling, including temporary conversions; allocator failure returns an explicit local limitation |
| Scheduling | Bounded cancellation/deadline per request; no automatic retry or background context collection |

These ceilings are proposed limits, not proof that the candidate fits or produces
useful answers. Read the pinned architecture/dtypes first; refuse an incompatible
shape, overflow or memory requirement. No silent quantization, context truncation,
cloud fallback or unrecorded format conversion. Measure memory, stack, latency,
cancellation and answer quality before proposing a larger budget.

## Positive and negative native acceptance design

| Requirement | Test owner | Required experiment |
|---|---|---|
| LP-04 | AI/store/native integration | With no network capability, complete each approved task from a verified installed generation; record actual output, memory and latency |
| LP-05 | Kernel/AI boundary | Direct socket, delegated helper and subprocess upload attempts fail on actual caller identity; packet observation finds no egress |
| LP-06 | Allocator/model adapter | Missing model, oversized context/KV, OOM and cancellation report a local limitation; no provider request or false answer success |
| LP-07 | Downloader/permission boundary | Separately authorized downloader cannot obtain selected document, history, index, NAS or content-key capabilities |
| Model safety | Parser/tokenizer/engine | Tampered/stale generation, malformed tensors/configuration and hostile input fail before untrusted code or out-of-bounds allocation |

The [privacy contract](../LOCAL-PRIVACY-CONTRACT.md) keeps inference exclusively
on-device. Downloader authority is separate, explicitly scoped and never inherited
by the inference process. No prompt or model output creates permissions, invokes a
shell or enrolls a helper. Capability-absence denial must be tested on the native
kernel; a host boolean or disconnected mock is insufficient.

## Numeric profile

Owner decision, 25 November 2026: SSE2 hardware floating point is approved for the
native ring-3 AI process. Conditions:

- Reuse the existing eager FXSAVE64/FXRSTOR64 context management (ADR-045,
  `fpu-context`), including initialization, register isolation and secure cleanup.
  No new FPU mechanism, and no floating point in kernel code.
- Verify state preservation across scheduling, preemption, IPC, blocking, interrupts,
  process termination and slot reuse. Enable SMP and sleep/wake combinations only
  after their own verification.
- Verify CPU feature availability and compiler-generated instructions. AVX, AVX2,
  AVX-512 and FMA stay disabled until context management supports and verifies them.
- Measure native performance, numerical correctness, latency, memory and desktop
  responsiveness. A soft-float comparison is optional, used only when it adds evidence.

Implementation route: `x86_64-unknown-none` is a soft-float target, and enabling
SSE there is being phased out by rustc. The AI job therefore builds for the
hard-float `x86_64-unknown-linux-gnu` ABI (x86-64 baseline: SSE/SSE2 only) as a
freestanding static ELF without libc or Linux syscalls. An allowlist gate inspects
the linked ELF. See [the native job](../../experiments/ai/native-job/README.md).

**OWNER DECISIONS:** approved 25 November 2026 (see top and the numeric profile). Native
inference, usability/Dutch-quality tests, hardware and external review are NOT RUN.
L01 remains open and the roadmap score stays 41/100.
