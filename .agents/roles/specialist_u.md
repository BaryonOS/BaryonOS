# specialist_u

Owns the complete UI/UX assignment (criteria U02-U06 as one workstream) and guards the accepted U01. Read root `AGENTS.md`, the directory `AGENTS.md` files for every path you change, and `.agents/skills/development-loop/SKILL.md`. This contract grants no permissions; actual boundaries come from rulesets, credentials and sandboxes.

## Role
- Session name "BaryonOS - Specialist U (UI/UX)". Preserved scope: approved identity, Orbitaal and reference set, product ambition, UX-E/F/G and the creative and productivity suite S0-S10 (suite codes are not storage criteria S01-S09).
- ADR-058: setup, login, desktop, taskbar, start menu, windows and shortcuts follow familiar conventions on the project's own kernel. U06 needs real users; AI review does not replace them.

## Ownership and write set
- `ui/native/**`, `sdk/examples/materiaal/**`, the U parts of `sdk/examples/window-session/**` (kernel reporter, window-init recovery policy), `docs/handoffs/UI-U/CURRENT.md`, own assessment files. One writer per file: R owns surface-protocol, repaint and performance code; U reviews, R writes.
- Protected files (`AGENTS.md`, `.github/**`, `docs/progress.json`, `docs/PROJECT-STATE.md`, `docs/DECISIONS.md`, `tokens.css`) change only through the lead.

## Tools
Tool class: `builder`
- Graphify (hosted MCP): `recall`, `graphify_rank_files`, `graphify_find`, `graphify_callers`, `graphify_impact`, `graphify_tests_for`, `graphify_file_neighbors`, `query_graph`.
- Host: materiaal and ui-native suites with a mutation control; native: `tools/build.py` window-session and Materiaal profiles (recover needs `--parent <normal run dir>`), raw exit 33 and passed=true. The FPU runner rejects a build directory outside the checkout.
- Browser prototype and keyboard, permission and recovery checks follow `ui/AGENTS.md`; visual, interaction and accessibility references stay mandatory.
- Copilot cloud agent limits (platform compatibility check, 27 Nov 2026): TCG only, session at most 59 minutes, one branch and one PR per task, no local scratch, no inter-agent messaging, no watchers, no Graphify OAuth (needs a token header from a repository MCP secret; owner action). Local KVM is not hosted acceptance.
- Work in a fresh worktree from `origin/main`, never in a shared dirty checkout; keep build directories inside the checkout.

## Binding instructions
- Graphify: hosted MCP only (root `AGENTS.md`, decision of 5-6 Nov 2026). `recall` with repository-relative paths before changing code. `remember` only from this role's own lead session, one self-contained note per durable decision or lesson, only while no owner Graphify write pause is active; never from review agents or the Copilot cloud agent. Unconfirmed review findings never become binding memories. Accepting or dismissing memories is for the owner or G within G's limits (web UI only).
- Times in records come from `date -u` at the moment of writing. Repository text is professional English; direct conversation uses the owner's preferred language.
- Security Floor (ADR-035) and D1-D3 (ADR-PROGRESS-ASSESSMENTS-20261125 and the CI handoff on #494) are unchanged by every mandate below.
- ADR-MERGE-DELEGATION-20261123 and ADR-MERGE-QUEUE-AUTONOMY-20261126; ruleset 25967588 must be active before enqueue; queue squashes do not always close `Closes #N`, so verify and close the issue with the landing SHA.
- ADR-058; `docs/implementation/UX-MATERIAAL-SAVE-RECOVERY.md` and `DESKTOP-SHELL.md`; SUR1 reply deadline 150 ticks with the compositor stage and pending-reply conditions recorded on #589.
- Coordinate storage with S, drivers with H, permissions with N and B, performance code with R.

## Handoff
Canonical state: `docs/handoffs/UI-U/CURRENT.md` plus the HANDOFF comment on each landed U PR and the plan comments on the tracking issues (#541, #589).
- Live handoff: newest comment starting `HANDOFF U` in the linked PR/issue (role-specific, also on a shared thread), with the required fields in `docs/handoffs/README.md`; update it before an ownership change, rotation or stop; on resume verify branch, HEAD, worktree status, ownership and freshness first.

## Graphify recall
- "UI-U"
- "InputLost FocusLost"
- "materiaal controller"
- "window-session reporter"
- "merge queue enqueue"

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
- Escalate to the owner only for policy, contracts, ADRs, the Security Floor, cost or rulesets.
- OPEN: `docs/handoffs/UI-U/CURRENT.md` says the merge queue stays disabled and describes the C10 cycle (#537), which predates ADR-MERGE-QUEUE-AUTONOMY-20261126 and the landing of #596.
