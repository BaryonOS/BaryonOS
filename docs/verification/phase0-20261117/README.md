# Phase 0 integration checkpoint — 17 November 2026

## Executed T0 reconciliation

- Task: establish the bounded integration and quality baseline before governance phases 1–6.
- Source main: `b718f6bafdfc4927d3b7acfe786e67dcb8f88eec`; shared local checkout: `c07506c85ce30ad9177e2e2db4118fe48e867a5c`.
- Sixteen existing worktrees were inspected. Three contain WIP: the shared main checkout, Orbitaal UI/UX, and the local pressure test. None was modified.
- No other Claude/Copilot/controller process was visible in this execution environment. This does not release ownership or establish that remote sessions are stopped.
- Four shared Git post-hooks invoke local Graphify, contrary to repository policy. They were preserved. A separate clone with ordinary Git configuration and no active Git hooks was created; no `--no-verify` or hooksPath override was used. Installed client hooks and required checks are not claimed to be verified by this isolation.
- Orbitaal (#339) and S09 (#340) remain Claude-owned under the existing handoff. Their source is reviewed read-only until an explicit transfer; a free lock does not grant ownership.
- The initial authenticated T0 inventory contained exactly PRs #333/#336/#339/#340/#343/#344 and issues #41/#42/#43/#274/#277/#283. New unrelated arrivals are post-baseline work.

## Initial T0 integration and issue dispositions

| Order | PR / issue | Current disposition and required next evidence |
| --- | --- | --- |
| 1 | #343 artifact download | Head `1c6842c4c73f42691fa553e76944bc9ebd795186`. Current PR SMP jobs pass but the changed manual aggregate is skipped. Add a bounded upload/download contract job; preserve all native gates. Independent upstream review found no concrete security regression. |
| 2 | #344 r-efi | Head `1cfaad3d8259147f05c6a7233ef932c6c2670a72`. P1: update audited dependency records; package provenance/licence review must accompany the pin. |
| 3 | #336 / #41 | Head `6b2f3d2698cd525904ec69f35ab3081d5cad18d9`. TEST authentication is a bounded slice. Persistence, production custody and human review remain open; no production key/signing authority is inferred. |
| 4 | #333 / #42 | Head `b0aac21645d1f9b178dcc076360a4c016ffda8fd`. DMA/USB and synthetic HID proofs remain distinct. End-to-end handoff, general lifecycle and authorised physical matrix remain open. |
| 5 | #340 | Head `93797f3a6075af9cc4fdd533dad288bfa7894ffe`. Preserve S09 ownership. Portal Clippy failure and review findings need disposition; D1–D5 require the owner. |
| 6 | #339 | Head `ca3534f32b8bd80373143542e14b8d3afe0ea298`. Preserve dirty worktree and design owner. Research/checkpoint scope is not native product acceptance. |
| — | #43 | 50/100 remains an open long-term assignment; baseline B does not require reaching it. No automatic points or issue closure. |
| — | #274 / #277 | Translation and live Wiki coverage require separate evidence. Complete triage and record access/ownership limitations; do not claim completed coverage. |
| — | #283 | Classify preserved historical proposals by actual source inclusion, replacement, rejection or deferral. Closed is not merged. |

At initial T0, each merge required separate owner approval. The updated scope-limited Phase 0 authority below supersedes that approval workflow; current verified source, checks, merge-hold and review clearance remain mandatory. No merge or auto-merge has been performed. Ready/draft metadata conflicts with several PR descriptions; metadata alone grants no merge authority.

## Current task contract

Goal: retain this executed T0 checkpoint and prepare #343/#344 for evidence-based review.
Invariant: preserve all WIP, workstream authority, accepted criteria, Security Floor and existing gates.
Write set for this checkpoint: this new evidence directory, the lead-owned top handoff in `docs/PROJECT-STATE.md`, and the linked progress assessment/generated views. Ordinary workers do not own these files.
Acceptance: `python3 -B tools/progress.py --write`, `python3 -B tools/progress.py --check`, `git diff --check`, and required hosted checks on the checkpoint PR. Implementation PRs retain their own targeted/native checks.
Evidence limit: T0 is inventory, not integrated main acceptance, a completed security audit, runtime parity or an OS progress criterion. Baseline B is NOT ESTABLISHED; governance phases 1–6 have NOT STARTED.

## Logs and resume

Raw sanitized T0 records: `/tmp/baryonos-phase0-20261117/T0-local.json`, `T0-github.json`, `T0-checks-343-344.json`. Their hashes are recorded separately. Full command logs remain outside Git.
Resume from branch `codex/phase0-baseline` in `/tmp/baryonos-phase0-20261117/repo`; confirm the remote head and existing ownership before writes. The current resume section below supersedes the initial T0 next actions. The complete integration/quality assessment and final source freeze remain outstanding.

## Follow-up execution checkpoint

- PR343 correction published as aa7a0387aa34c2f9ead9a46d9a3ee7df32f5a3a5. All 349 tools tests pass with the installed rustup binary (exit 0). Hosted artifact contract succeeds; its synthetic checkout has the exact PR-head tree. Other affected/required jobs are individually tracked, not implied passed.
- PR344 audit correction published as 57a9e5f3a1abe6c6f2364d340d765fcaa198aa35. The ten-group loader/M1 gate and all 18 native update stages pass locally (exit 0). Independent re-review confirms the inventory finding addressed; its thread is resolved. Current-head hosted checks remain separate.
- PR336 Rust/Python and native TEST-policy probe pass locally (exit 0); independent scoped security review found no concrete blocker. Three coupling advisories were independently dispositioned as nonblocking bounded orchestration, with rationale in [the review record](pr336-review-dispositions.json); their threads were resolved. PR333 and336 are BEHIND current main and require reconciliation before merge readiness.
- PR333's existing xHCI/HID local CI subsets pass (exit 0): five USB/xHCI profiles plus the separate synthetic HID IPC probe, with all 45 raw child logs retained. [Native receipts](pr333-native.json). Independent scoped security review found no concrete blocker but does not establish H02 or physical acceptance.
- [Follow-up raw-code/log receipts](followup-checks.json), [hosted fixture receipt](pr343-hosted-contract.json), [historical PR dispositions](issue283-triage.json), [documentation gaps](documentation-triage.md).
- Preliminary authenticated read access to CodeQL/dependency alerts works. This is audit readiness, not the fresh final integrated security assessment required after queue processing.

Baseline B is still not established. No merge, auto-merge, issue closure or governance migration has occurred. Current owner authority and exact resume steps are recorded below.

## Earlier bounded owner authority and completed repairs

The project owner explicitly approved squash merge of PR343 head aa7a0387aa34c2f9ead9a46d9a3ee7df32f5a3a5 **only after the remaining window-session checks are green**. [Exact approval scope](pr343-owner-approval.json). Required checks and the affected artifact/SMP jobs already pass; pending jobs are not reported as passed. Recheck exact head, current base, review status and every approval condition before the atomic head-matched merge. At that observation, other PR merges had no separate approval. The updated Phase 0 authority below now applies; auto-merge remains prohibited.

[PR340 read-only ownership/review disposition](pr340-ownership-review.md): retain the S09 owner; a bounded portal-lint repair and fresh fingerprint/gate remain. The owner granted only this limited repair authority. Source90816de39460b76e35cd626721d4dac9bde905e2 fixes the portal lint and wording in a separate clone; existing worktrees/WIP/runners and broader S09 ownership remain unchanged. Exact Clippy101→0,93 Rust tests,53 Python tests and the full15-stage native volume gate pass. [New native receipt](pr340-native-receipt.json) and [full bounded report](pr340-volume-gate.json). No merge authority was granted. D1–D5 and full product acceptance are not preconditions for that limited D6(a) slice.

## PR339 published checkpoint verification

On exact head ca3534f3 the read-only checker passes (exit 0), output equals the archived result and all five hashes match. The independent harness rejects30 in-memory mutations and two CLI negatives; the known row62 classification gap and ten published minors remain open. [Verification and actual scope finding](pr339-scope-review.json).

The verification README calls the work a systematic literature review, but the research explicitly describes a targeted, abstract-level, non-systematic review. Preserve the systematic owner requirement and obtain a source-owner correction or explicit owner disposition; checker success does not establish research completion. The published checkpoint and dirty future design WIP remain distinct. No Orbitaal source, runner or review thread was changed.

## Current resume — 17 November 2026

- PR343: owner approval remains conditional on both window-session jobs passing; the last observed pending job is `(1)`. No merge has occurred. Recheck the exact approved head, base and review/check gates immediately before merging.
- PR340: the explicitly released two-hunk repair is complete with fresh native evidence. This grants no broader S09 ownership or merge authority.
- PR339: the owner has now [released only the incorrect completion-claim correction](pr339-limited-owner-release.json). The bounded correction is published as `7bea10d96348b6d5990f9ed1fb53ab94940b3ae1`; independent review, all five hashes and the byte-identical checker output pass (exit 0). [Exact reviewed scope and commands](pr339-claim-correction-review.md). Research/matrix/archives and all design WIP/runners remain untouched. The systematic-review requirement remains open.
- Draft PR346 (`cece3041187e1b7d08ec7708c36b84c728d6e2d2`) translates only the remaining CI guide as a bounded contribution to the triaged #274 task. Semantic/anchor/command checks and all 26 workflow tests pass. Required current-head hosted checks remain in progress. Full #274 completion is not a baseline-B prerequisite.

Baseline B and the final integrated assessment remain outstanding. Governance phases 1–6 have not started. After PR343, reconcile PR344 with the new main before its own current-head merge verification; retain the remaining approved queue order and explicit dispositions.

## Updated owner-controlled Phase 0 merge authority

The project owner explicitly authorized the lead to independently review and squash-merge eligible original Phase 0 PRs, narrow PRs for the six inventoried issues, and necessary integration/security/quality fixes. [Exact scope and unchanged gates](owner-phase0-merge-authority.json). Separate per-PR approval is no longer required inside that scope. No auto-merge or repository-wide policy change is authorized. New trust/product/architecture decisions, unavailable mandatory evidence, destructive/hardware/key operations and unreconciled ownership still require escalation. PR343 retains its specific outstanding-window-check condition.

After a successful PR343 merge, reconcile PR344 with the new main, verify its complete integrated diff and current-head checks, and merge only when every applicable gate and review thread is clear. No merge has occurred as of this checkpoint.
