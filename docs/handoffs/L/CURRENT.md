# Specialist L — resume pointer

Compact resume pointer for the local system AI and quantum runtime specialist
(L01, L02). This file is not a backlog: tasks come from `docs/progress.json`,
issues and pull requests. Newer ADRs and live evidence take precedence over it.

## Scope, mandate and working agreements (recorded once)

- Scope: L01 useful local system AI; L02 local quantum execution (accepted,
  `docs/verification/L02-ACCEPTANCE.md`; keep the native simulator and its
  evidence); future hybrid/physical QPU research, which never blocks the
  consumer release. Post-quantum cryptography belongs to B with N.
- Mandate (project owner, 25 November 2026): ADR-MERGE-DELEGATION-20261123 is
  extended to L; operational coordination by "BaryonOS - PR Bewaker". Merge only
  after that extension is durably recorded and all merge conditions hold.
- Owner decision, 25 November 2026: the L01 proposal is approved for the first
  experiment (summarize/clarify tasks, Qwen2.5-0.5B-Instruct pin, Candle route,
  2 GiB VM). Open: the numeric profile (SSE in the AI process). Missing CI
  coverage is coordinated with specialist CI.

## Current cycle

- cycle_id: `L-20261125-decoder-layer`
- tasks: L01-T4 (RoPE, KV cache, GQA attention) and L01-T5 (per-request work
  budget with cancellation, in-place BF16 weights, full decoder layer, pinned
  configuration and tokenizer verification) in `experiments/ai/reference-forward`
- branch: `claude/l01-attention-kv-20261125`, worktree
  `~/projects/BaryonOS-worktrees/l01-attention-kv`
- pull request, tested source SHA and hosted runs: recorded in the pull request
- evidence: `tests/attention.rs`, `tests/decoder.rs` (f64 reference, two caught
  mutants); `x86_64-unknown-none` library build. Host evidence only.
- local data (not committed): `~/projects/BaryonOS-scratch/l/qwen-7ae5576/`
  holds the pinned config/tokenizer files and the safetensors header; hashes are
  in the crate README.

## Next action

1. Candle host oracle: pin a release, review dependencies, download the pinned
   model (988,097,824 bytes, verify SHA-256) and emit small golden fixtures
   (token ids, layer-0 hidden state, final top-k logits).
2. Full-model graph (embedding, 24 layers, final norm, tied head) checked against
   those fixtures; then a no_std byte-level BPE tokenizer checked id-for-id.
3. Native: needs the numeric-profile decision; meanwhile the LP-05 network-denial
   probe does not depend on the engine.
4. L02: no reopen without a concrete technical trigger.
