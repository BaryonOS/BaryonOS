# Codex–Claude instruction and workflow parity

**Evidence scope:** this checklist concerns client runtime parity, separately from
repository governance. Historical NOT RUN observations below are not prerequisites
for independent implementation. Shared instructions and adapters do not establish
client tool access, isolation, hardware support or acceptance. Record new observations
with source/environment; never infer all-surface parity from a single session.

**Evidence status: NOT ACCEPTED (15 November 2026).** This checklist is a mutable verification record, not an agent policy and not an OS acceptance criterion. GitHub source review cannot establish the user's on-device Codex configuration or run the local Git hooks.

## What is implemented in this repository

- Repository root and subsystem `AGENTS.md` files contain durable rules; only client-level working methods belong in `~/.codex/AGENTS.md`. Runtime state, owner decisions, logs and evidence remain outside `AGENTS.md`.
- `CODEX-WORKFLOW.md` and `docs/CODEX-START.md` describe the ten-step loop; `tools/agent_pipeline.py` implements the issue-driven L0 / execution / local gate route, while `tools/roadmap_runner.py` implements a **Claude-only** clean-session roadmap runner. These controllers are not interchangeable and no Codex end-to-end equivalence has been demonstrated.
- Issue-driven execution and the shared local gate now require a reviewed owner-controlled manifest and an isolated offline runtime; the committed configuration is disabled. [Execution constraints](roadmap-pipeline/README.md#separate-issue-driven-controller-default-deny-execution) describe the boundary and its unverified client surfaces. Offline fixtures do not establish model-client parity, and the roadmap runner's direct commands are not covered.
- Hosted Graphify is mandatory for architectural navigation **when available**; fallback is a bounded direct source inspection with the limitation recorded. `docs/roadmap-pipeline/README.md` reports no Graphify MCP servers in fresh headless Claude processes on the measured container.
- Token Guard and RTK are output-filtering tools, not test oracles. Ponytail guides minimal changes. ECC/Git hooks must remain enabled in the actual development checkout; an API-created commit does not verify local hooks.

## Verification matrix

| Requirement | Evidence available | Status |
| --- | --- | --- |
| Three instruction layers exist in repository/client | Root and 12 directory files on PR #341; global home file reported by Claude but inaccessible to this review | PARTIAL |
| Root/subdirectory instruction loading | Claude reported `codex debug prompt-input` from five directories; no independently retained transcript | REPORTED, NOT REPRODUCED |
| Tool-safe unattended changes | [PR #342](https://github.com/BaryonOS/BaryonOS/pull/342) merged as `ca50fa8f` after passing required host-quality, qemu-smoke and merge-hold; both controllers share the guarded path policy | REPOSITORY CHECKS PASSED; CODEX CANARY NOT RUN |
| Hosted Graphify connection inside Codex | No genuine Codex worker transcript | NOT RUN |
| RTK, Token Guard, Ponytail, ECC active | No end-to-end Codex execution artifact; some repository tool tests previously reported by Claude | NOT RUN |
| Real task: plan → edit → test → repair → checkpoint | Issue pipeline unit tests and Claude-only runner exist; Codex canary not executed | NOT RUN |
| Relevant native gates | This documentation change does not alter native code; no new local native test was executed | NOT RUN |

## Read-only Codex canary (on the authenticated development machine)

Run only in a clean checkout and **do not allow source writes** during the canary:

1. Record `git status --short`, branch and full HEAD. Inspect `~/.codex/AGENTS.md` without copying personal settings or secrets into Git.
2. Run `codex debug prompt-input` from the repository root and representative nested start directories (`services`, `ui`, `crates`, `tools`). Confirm that the global and root rules and the corresponding subtree rules are loaded without truncation. Preserve exact CLI version and a sanitized proof in ignored `build/codex-canary/`.
3. Start a genuinely new read-only Codex task using the canary instruction in `docs/CODEX-START.md`. Verify access to hosted Graphify, accurate current SHA, bounded read set, RTK/Token Guard availability and no write attempt. Never claim a plugin is active merely because its documentation exists.
4. Then execute one explicitly scoped, disposable test task on an isolated branch/worktree with hooks enabled. Require exact test commands, raw exit codes, relevant log hashes, complete diff review and a draft PR. Verify clean rollback for a rejected edit and never bypass protected-path policies.
5. Record the transcript/check identifiers, source SHA, active tool versions, checks not run and unresolved blockers here or in a new `docs/verification/` child. Only mark parity verified after all required observations pass on the same source and environment.

**Historical runtime-parity limitation:** This GitHub-connected review cannot access or execute the user's home-directory Codex runtime, its authentication, CLI session, or local hooks. The Codex canary therefore remains NOT RUN. Do not merge this documentation as evidence of completed parity or native acceptance.

## Safe continuation

Merge requires current-head host checks, the applicable acceptance hold and the project owner's explicit approval. The project owner may authorize a specific PR merge, but that does not authorize a general auto-merge policy. If the canary cannot run, leave parity open, record the exact boundary and continue independent code work without inventing results.
