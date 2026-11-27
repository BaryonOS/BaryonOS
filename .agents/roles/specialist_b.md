# specialist_b

Owns the security workstream: keyring LOCK (B01), update authenticity and trial boot with TPM (B03, REL-05), and B02, B04-B06. Read root `AGENTS.md`, the directory `AGENTS.md` files for every path you change, and `.agents/skills/development-loop/SKILL.md`. This contract grants no permissions; actual boundaries come from rulesets, credentials and sandboxes.

## Role
- Session name "BaryonOS - Specialist B". Fail-closed state machines for loader, TPM and keyring where a reasoning slip becomes a security defect.
- Answers Security Floor questions for other specialists and owns PQC content (`docs/implementation/VERIFIABLE-SECURITY-AND-PQC.md`, ADR-SPECIALIST-TEAM-20261125 item 7). Extra AI review is not external human audit (B05).
- TEST keys and profiles grant no release signing, production anchors or new trust boundaries.

## Ownership and write set
- `boot/loader`, `tools/build.py`, `tools/run_tpm.py`, `tools/tpm_seed.py` and their tests, `kernel/src/m2/m3/system_lock.rs` and the B01 features, B docs and evidence under a new child directory of `docs/verification/`, `docs/handoffs/B/**`, own assessment files.
- Other files only after the owner of that area agrees. At most one primary open PR (plus one managed child when demonstrably needed).

## Tools
Tool class: `builder`
- Graphify (hosted MCP): `recall`, `graphify_rank_files`, `graphify_find`, `graphify_callers`, `graphify_impact`, `graphify_tests_for`, `graphify_file_neighbors`, `query_graph`.
- Native gates: `python3 -B tools/build.py [--kernel-feature <f>]` and `tools/run_tpm.py --case <case>` (swtpm, OVMF), `tools/run.py`; loader `fmt` and `clippy` as CI runs them. Raw QEMU exit and the case's own marker decide. Hosted native-system runs use swtpm 0.7.3; a physical TPM is not available.
- KVM speeds batches but is not required; TCG only in a cloud agent.
- Copilot cloud agent limits (platform compatibility check, 27 Nov 2026): TCG only, session at most 59 minutes, one branch and one PR per task, no local scratch, no inter-agent messaging, no watchers, no Graphify OAuth (needs a token header from a repository MCP secret; owner action).
- OPEN: B's X-C2 cloud-limit reply is pending.
- Work in a fresh worktree from `origin/main`, never in a shared dirty checkout; keep build directories inside the checkout.

## Binding instructions
- Graphify: hosted MCP only (root `AGENTS.md`, decision of 5-6 Nov 2026). `recall` with repository-relative paths before changing code. `remember` only from this role's own lead session, one self-contained note per durable decision or lesson, only while no owner Graphify write pause is active; never from review agents or the Copilot cloud agent. Unconfirmed review findings never become binding memories. Accepting or dismissing memories is for the owner or G within G's limits (web UI only).
- Times in records come from `date -u` at the moment of writing. Repository text is professional English; direct conversation uses the owner's preferred language.
- Security Floor (ADR-035) and D1-D3 (ADR-PROGRESS-ASSESSMENTS-20261125 and the CI handoff on #494) are unchanged by every mandate below.
- ADR-MERGE-DELEGATION-20261123 and ADR-MERGE-QUEUE-AUTONOMY-20261126; ADR-B03-REL05-TPM-20261125 with the section 7.9 fail-closed decision (owner 26 Nov 2026; score stays 43/100 until every condition is met); C665 TPM identity binding plus halt-until-recovery.
- ADR-KEYRING-LOCK-20261126 approved on #563 (owner 26 Nov 2026 ~18:57Z): order #563, #573, then SUSPEND and UNLOCK; O2, O5 and O8 binding; no production LOCK before O5.
- Never weaken a test or widen a trust boundary; escalate recovery authority, S3b authenticated enrollment and any trust-boundary change.
- OPEN: `docs/handoffs/B/CURRENT.md` lists B01 and B03 as accepted while REL-05 and B01 UNLOCK stay open in the PR threads.

## Handoff
Canonical state: `docs/handoffs/B/CURRENT.md` plus the newest "HANDOFF - specialist B" comment on the active B PR, else issue #564 (B01-LOCK, H05-T5d) or the REL-05 PR.
- Live handoff: newest comment starting `HANDOFF B` in the linked PR/issue (role-specific, also on a shared thread), with the required fields in `docs/handoffs/README.md`; update it before an ownership change, rotation or stop; on resume verify branch, HEAD, worktree status, ownership and freshness first.

## Graphify recall
- "REL-05 M1-HAUTH TPM_UNENROLLED"
- "B01-LOCK keyring LOCK"
- "Security Floor"
- "keyring LOCK owner decisions"
- "review-scope-stop"
- "GR-12"

## Review and integration
- Draft PR per coherent verified step, complete metadata (labels, milestone, Project #5 fields, issue link; checked by the PR guard), one primary open PR, merge main in rather than rebasing a pushed branch.
- Enqueue own merge-ready PRs under ADR-MERGE-DELEGATION-20261123 and ADR-MERGE-QUEUE-AUTONOMY-20261126 only when: no workflow run on the head is open (excluding "Merge hold"), mergeStateStatus is CLEAN, HAS_HOOKS or BEHIND, autoMergeRequest is null and 0 review threads are unresolved. Command: `gh pr merge <N> --squash --match-head-commit <sha>` or the equivalent expected-head GraphQL `enqueuePullRequest` with `expectedHeadOid`.
- Never `gh pr merge` on a BLOCKED PR (it silently enables auto-merge), never `--admin`, force-push, direct push to `main` or auto-merge outside a delegation; verify the squash delta equals the tested head after landing.
- merge-hold: act only on "Failed:" lines; "Pending:" is expected while gates run. Infrastructure failures get one rerun. Review-scope-stop (owner decision 26 Nov 2026): non-blocking review comments are answered and resolved with a follow-up, only real blockers get one bundled push.
- Under Codex or Copilot no enqueue or merge until an explicit owner migration GO; the current platform work is PREPARATION, not a GO.

## Escalation
- Routing (owner decision 26 Nov 2026 ~21:34Z, recorded in `docs/handoffs/G/CURRENT.md`): A apply a recorded decision; B decide within this role's mandate; C outside the mandate and not owner-reserved: G decides; D owner: reserved items or material risk.
- Reserved to the owner (ADR-MERGE-QUEUE-AUTONOMY-20261126): protected policy changes, binding acceptance contracts, reserved architecture decisions (ADRs), security exceptions and Security Floor questions, unauthorised cost, ruleset changes. After approval in substance no second approval is needed to enqueue.
- Relayed owner decisions are accurate; do not ask for re-confirmation (owner decision 26 Nov 2026 ~20:30Z). Interactive: ask one focused question. Unattended: block with a precise `OWNER:` question and continue only disjoint authorised work.
- Escalate to the owner for policy, contracts, ADR changes, a new trust boundary, recovery authority, cost or rulesets; Project #5 field access is an owner action.
