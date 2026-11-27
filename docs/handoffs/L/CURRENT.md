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
  extended to L; operational coordination by "BaryonOS - PR Bewaker". The
  extension is recorded on main (ADR-SPECIALIST-TEAM-20261125, #527). Merge
  with `--match-head-commit` once all merge conditions hold, following the
  PR guard's merge-lane protocol (first-green-first, one lane holder catches up,
  drafts do not catch up, temporary CI-PR priority).
- Owner decision, 25 November 2026: the L01 proposal is approved for the first
  experiment (summarize/clarify tasks, Qwen2.5-0.5B-Instruct pin, Candle route,
  2 GiB VM). SSE2 in the native ring-3 AI process is approved (same day) under
  the existing eager FXSAVE64 context; no AVX/FMA, no kernel FP, SMP and
  sleep/wake only after their own verification. CI coverage is coordinated with
  specialist CI.

## Current cycle

- cycle_id: `L-20261125-decoder-layer`
- tasks: L01-T4 (RoPE, KV cache, GQA attention), L01-T5 (work budget with
  cancellation, in-place BF16 weights, decoder layer, pinned config/tokenizer
  hashes) and L01-T6 (full `Model`, host Candle oracle `experiments/ai/candle-oracle`,
  opt-in real-model test `tests/qwen_oracle.rs`, which matches)
- branch: `claude/l01-attention-kv-20261125`, worktree
  `~/projects/BaryonOS-worktrees/l01-attention-kv`
- pull request, tested source SHA and hosted runs: recorded in the pull request
- evidence: `tests/attention.rs`, `tests/decoder.rs` (f64 reference, two caught
  mutants); `x86_64-unknown-none` library build. Host evidence only.
- local data (not committed): `~/projects/BaryonOS-scratch/l/qwen-7ae5576/`
  holds the pinned config/tokenizer files and the safetensors header; hashes are
  in the crate README.

## Next action

1. no_std byte-level BPE tokenizer, checked id-for-id against the oracle
   fixture ids; refuse input it cannot normalize or classify.
2. Model file: `~/projects/BaryonOS-scratch/l/qwen-7ae5576/model.safetensors`
   (SHA-256 verified); run the oracle test with `NOS_QWEN_DIR` set.
3. Native SSE2 job: draft PR #538, stacked on #525, branch
   `claude/l01-native-ai-job-20261125`, worktree
   `~/projects/BaryonOS-worktrees/l01-native-ai-job`. `native-ai-job` passes natively
   (QEMU TCG, one CPU, raw exit 33, exact host-oracle bits, slot reuse, canary,
   context wipe). The MXCSR mutation was caught.
   Warden review fixed (allowlist gate, source-tree binding, per-run preemption);
   CI lead commits added the dependabot entry, the native job and clippy (head
   0c4c3a83, D2 GO, D3 file present). Merge order: #525 first (head 9c0d4c56,
   waiting for GitHub to recover from the 25 November Actions outage to rerun
   coverage uploads), then retarget #538 to main and catch up once it holds the
   lane. If A's #531 lands first, keep both sides' lines in kernel/Cargo.toml,
   tools/build.py and m3.rs and rerun native-ai-job.
   Next: a verified artifact-mapping design for full Qwen, which is blocked by
   USER_QUOTA (8 MiB per process); the tokenizer (item 1); LP-05 network denial.
4. L02: no reopen without a concrete technical trigger.
