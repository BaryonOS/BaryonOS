# Specialist L — resume pointer

Compact resume pointer for the local system AI and quantum runtime specialist
(L01, L02). This file is not a backlog: tasks come from `docs/progress.json`,
issues and pull requests. Newer ADRs and live evidence take precedence over it.
The canonical live state is the latest `HANDOFF L` comment on issue #534
(as of 27 November 2026:
[issuecomment-7218440165](https://github.com/BaryonOS/BaryonOS/issues/534#issuecomment-7218440165))
and the open L pull requests; this file is sufficient to resume from the repository and
GitHub alone. Local paths below are optional additions for a local session.

## Scope, mandate and working agreements (recorded once)

- Scope: L01 useful local system AI; L02 local quantum execution (accepted,
  `docs/verification/L02-ACCEPTANCE.md`; keep the native simulator and its
  evidence); future hybrid/physical QPU research, which never blocks the
  consumer release. Post-quantum cryptography belongs to B with N.
- Mandate (project owner, 25 November 2026): ADR-MERGE-DELEGATION-20261123 is
  extended to L (ADR-SPECIALIST-TEAM-20261125, #527). Since 26 November L
  enqueues its own merge-ready pull requests in the merge queue; "BaryonOS - PR
  Bewaker" coordinates and verifies landings. Never `--admin`, never auto-merge;
  CI pull requests go first.
- Pre-enqueue checks: `mergeStateStatus` CLEAN, `autoMergeRequest` null, no
  unresolved review threads, no queued or in-progress run on the head; check
  `autoMergeRequest` again after enqueueing. A red `merge-hold` that reports only
  `Pending:` means wait; a base-tip failure means merge `main` once (no rebase)
  and push; `Failed:` means repair.
- Owner decision, 25 November 2026: the L01 proposal is approved for the first
  experiment (summarize/clarify tasks, Qwen2.5-0.5B-Instruct pin, Candle route,
  2 GiB VM). SSE2 in the native ring-3 AI process is approved (same day) under
  the existing eager FXSAVE64 context; no AVX/FMA, no kernel FP, SMP and
  sleep/wake only after their own verification. CI coverage is coordinated with
  specialist CI.
- Owner decision, 27 November 2026 (relayed by the PR guard): every criterion
  owner keeps the implementation and verification snapshot rows of its own
  criteria (L01, L02) in `docs/progress.json` (`delivery_review`) current without
  asking per update. Only own rows, with current source-bound evidence; rubric,
  weights and acceptance conditions unchanged; the score rises only on full
  acceptance plus completed independent verification; the global snapshot date and
  `source_commit` stay untouched (date and SHA go in the row text); regenerate with
  `python3 tools/progress.py --write` and `--check`; use D3 assessment files. The
  PR guard checks this at every relevant landing.

## Current cycle

| Field | Value |
|---|---|
| cycle_id | `L-20261127-codeql52-and-qwen-mapping` |
| Landed | #538 native SSE2 ring-3 AI job (`ac495b2f`); #598 native job hardening, CodeQL #51 fixed (`41955c86`); #599 no_std BPE tokenizer (`8d079547`); #609 resume pointer (`a98e64f9`) |
| In queue | #619 fix for CodeQL alert #52 (`py/overly-permissive-file` in `experiments/ai/native-job/tests/test_gate.py`); after landing, #52 must show as fixed on `main` |
| Open | full-Qwen model-mapping design, branch `claude/l01-model-mapping-design-20261127`, `docs/implementation/LOCAL-AI-L01-MODEL-MAPPING.md`; draft pull request awaiting owner answers to Q0–Q6 |
| Tracking | issue #534 (L01); tested heads, hosted runs and D3 assessments are recorded in each pull request |
| Coordination | K (#602, C3 scheduler) re-measured `JOB_IRQS` locally (28–41 job IRQs, inside the documented 20–81) and reports the hosted range before any change; K and B are interface owners for the mapping design's first slice |
| Score | L01 open; L02 accepted |

## Resuming

A resuming agent, whatever its client, first verifies branch, exact HEAD,
`git status --short`, ownership and the freshness of the latest `HANDOFF L`
comment against GitHub, and never redoes completed work. Graphify is optional
context; client-specific notes are adapters and hold no state.

## Model data and oracle

The model is not committed. The pin is public: Qwen2.5-0.5B-Instruct at
revision `7ae557604adf67be50417f59c2c2f167def9a775`; file sizes and SHA-256
values are in `experiments/ai/reference-forward/README.md`. The opt-in oracle
test (`experiments/ai/reference-forward/tests/qwen_oracle.rs`) and the tokenizer
sweep read the downloaded files through `NOS_QWEN_DIR` (and the sweep corpus
through `NOS_TOKENIZER_SWEEP`); without them only the committed fixtures run.

## Next action

The single next safe action is stated in the latest `HANDOFF L` comment; the
longer-term order is:


1. Land #619 through the merge queue, report the landing SHA in a `HANDOFF L`
   comment on #534 and confirm CodeQL alert #52 is fixed on `main` (never dismiss it).
2. Obtain the owner's answers to Q0–Q6 of the model-mapping design; then an ADR (if
   Q0 is yes) and the first slice S1 (synthetic artifact, own profile) with K and B.
   Full Qwen remains blocked by the frame pool and private-memory limits (Q2, Q5).
3. LP-05 network denial for the AI process.
4. L02: no reopen without a concrete technical trigger.

## Cloud agent limits (GitHub Copilot cloud agent)

Suitable: host-side Rust and Python work in `experiments/ai/` and
`tools/tests/` (unit tests, clippy, fmt), documentation, and answering review
comments. Not suitable: the `native-ai-job` QEMU gate needs a full kernel build
plus several QEMU runs; under TCG only and a 59-minute limit it is not a
reliable gate, and its timer-IRQ ranges are calibrated for hosted CI runners.
The real-model oracle and tokenizer sweep need the ~1 GB model files, which
the cloud agent cannot fetch from a local store; they stay local or hosted.
Graphify is not available to the cloud agent (OAuth).

## Local additions (optional)

- Local session handoff: `~/projects/BaryonOS-scratch/l/HANDOFF-*.md` (newest wins).
- Local model directory used for the oracle: `~/projects/BaryonOS-scratch/l/qwen-7ae5576/`.
