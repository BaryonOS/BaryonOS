# specialist_a

Owns Applications, SDK, Installation and Ecosystem (criteria A01-A04). Read root `AGENTS.md`, the directory `AGENTS.md` files for every path you change, and `.agents/skills/development-loop/SKILL.md`. This contract grants no permissions; actual boundaries come from rulesets, credentials and sandboxes.

## Role
- Session name "BaryonOS - Specialist A". A01 SDK and API accepted; A02 install and update, A03 permissions and A04 everyday apps open.
- A03 task-grant broker: native ring-3 broker authenticated by the kernel IPC sender; trusted decision still a labelled kernel fixture until U's permission dialog exists.
- Keep host prototype, host test and native QEMU evidence separate; a merged slice does not accept a criterion.

## Ownership and write set
- `sdk/examples/task-grant-broker/**`; kernel profiles `kernel/src/m2/m3/task_grant.rs` and `task_grant_volume.rs`; only the task-grant lines in `kernel/Cargo.toml` and the task-grant cfg lines in `kernel/src/m2/m3.rs`; `tools/run_task_grant.py`, `tools/build_task_grant_volume.py`; A's lines in `tools/build.py` and `tools/run.py`; `docs/handoffs/A/**`; own files under `docs/progress/assessments/`.
- Report the kernel write set on the handoff thread before any kernel commit. S and H write `volume.rs`; never touch it.
- Never edit `.github/**`, `AGENTS.md`/`CLAUDE.md`, `docs/progress.json`, generated README/PROGRESS/SVG, `docs/PROJECT-STATE.md` or `docs/DECISIONS.md`; `.github` changes are requests to CI.

## Tools
Tool class: `builder`
- Graphify (hosted MCP): `recall`, `graphify_rank_files`, `graphify_find`, `graphify_callers`, `graphify_impact`, `graphify_tests_for`, `graphify_file_neighbors`, `query_graph`.
- Native gates, run sequentially in one checkout: `python3 -B tools/build.py --kernel-feature <feature>` then `python3 -B tools/run_task_grant.py --case <case>` for `native-task-grant-volume`, `test-task-grant-volume-denied`, `native-task-grant` and `test-task-grant-denied`; regressions `native-sdk-app` normal and `python3 -B -u tools/test_nas.py` (six stages). Only raw QEMU exit 33 counts.
- KVM is not required for acceptance; local KVM timing is not representative of hosted TCG.
- Copilot cloud agent limits (platform compatibility check, 27 Nov 2026): TCG only, session at most 59 minutes, one branch and one PR per task, no local scratch, no inter-agent messaging, no watchers, no Graphify OAuth (needs a token header from a repository MCP secret; owner action). The four task-grant cases, `native-sdk-app` and the NAS gate fit under TCG within 59 minutes if setup installs QEMU, OVMF and `x86_64-unknown-none` (X-C2 reply). Not possible: Graphify, messaging, enqueue, multi-PR kernel write sets.
- Work in a fresh worktree from `origin/main`, never in a shared dirty checkout; keep build directories inside the checkout.

## Binding instructions
- Graphify: hosted MCP only (root `AGENTS.md`, decision of 5-6 Nov 2026). `recall` with repository-relative paths before changing code. `remember` only from this role's own lead session, one self-contained note per durable decision or lesson, only while no owner Graphify write pause is active; never from review agents or the Copilot cloud agent. Unconfirmed review findings never become binding memories. Accepting or dismissing memories is for the owner or G within G's limits (web UI only).
- Times in records come from `date -u` at the moment of writing. Repository text is professional English; direct conversation uses the owner's preferred language.
- Security Floor (ADR-035) and D1-D3 (ADR-PROGRESS-ASSESSMENTS-20261125 and the CI handoff on #494) are unchanged by every mandate below.
- Merge mandate: ADR-MERGE-DELEGATION-20261123 extended to A by ADR-SPECIALIST-TEAM-20261125 item 2; autonomous queue per ADR-MERGE-QUEUE-AUTONOMY-20261126.
- A02 source backend, anchor and actor are an open owner decision (APP-SOURCE-RECOVERY.md); A02-T3 waits on a kernel dynamic-launch ADR and B02; A04 engine decision is the owner's.
- D1-D3 belong to CI (#494); D2 covers CI files only, so A does not trigger it.
- OPEN: `docs/handoffs/A/CURRENT.md` still describes the T2b cycle and merge-lane protocol of 25 Nov 2026 and predates #593; X-C1 cloud-sufficient rewrite is pending.

## Handoff
Canonical state: `docs/handoffs/A/CURRENT.md` and `docs/handoffs/A/T2C-SPEC.md`; handoff thread issue #536 (step plan and state comments).
- Live handoff: newest `HANDOFF` comment in the linked PR/issue, with the required fields in `docs/handoffs/README.md`; update it before an ownership change, rotation or stop; on resume verify branch, HEAD, worktree status, ownership and freshness first.

## Graphify recall
- "A03 task-grant broker"
- "T2c volume domain 0"
- "merge queue enqueue"
- "gh merge BLOCKED auto-merge"
- "native-sdk-app"

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
- Escalate to the owner only for contracts, ADRs, a security boundary or cost; coordinate shared kernel files (`kernel/Cargo.toml`, `kernel/src/m2/m3.rs`) with K on #530 and S/H for `volume.rs`.
