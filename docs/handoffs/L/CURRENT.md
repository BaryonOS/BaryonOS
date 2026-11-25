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
- Owner-gated, unchanged: every decision listed in
  `docs/implementation/LOCAL-AI-L01-PROPOSAL.md` (first task/quality set,
  approval and provenance of the Qwen2.5-0.5B-Instruct pin, engine/port route,
  VM budget, numeric profile). No model download, engine dependency or new
  capability before those decisions.

## Current cycle

- cycle_id: `L-20261125-attention-kv`
- task: L01-T4 — bounded RoPE, fixed-capacity KV cache, causal grouped-query
  attention and the checked KV-budget formula in
  `experiments/ai/reference-forward` (own code, no new dependency)
- branch: `claude/l01-attention-kv-20261125`, worktree
  `~/projects/BaryonOS-worktrees/l01-attention-kv`
- pull request, tested source SHA and hosted runs: recorded in the pull request
  description, not in this file
- evidence: `experiments/ai/reference-forward/tests/attention.rs` (host);
  `x86_64-unknown-none` release library build. Host evidence only; no native
  inference, model or quality claim.
- gap: the crate is not covered by any workflow (README states CI is added with
  integration); a CI job needs a protected `.github/**` change.

## Next action

1. After the owner decides the proposal items, read the pinned checkpoint's
   configuration through the verified-artifact path and check the RoPE/GQA
   layout and KV budget against it.
2. Without decisions: a single decoder-layer composition (RMSNorm → QKV →
   RoPE → attention → output projection → residual → SwiGLU) over these
   primitives with a hand-checked fixture.
3. L02: no reopen without a concrete technical trigger.
