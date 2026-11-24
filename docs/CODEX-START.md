# Portable Codex start and recovery prompt

Use this text as the **initial prompt** in an authenticated Codex session started from a BaryonOS repository checkout. This file is a stable startup guide, not live task state.

Read root `AGENTS.md`, applicable directory instructions, `docs/STATUS.md` and
the latest relevant `docs/PROJECT-STATE.md` checkpoint. Confirm branch, full HEAD,
short status and exclusive task ownership. Check cross-agent handoffs before writing;
preserve unrelated dirty work. Load only missing contracts and relevant ADRs.

Execute `.agents/skills/development-loop/SKILL.md`; use the actual roles
`.codex/agents/os_architect.toml`, `os_builder.toml`, `os_verifier.toml` and
`os_security_reviewer.toml`, whose contracts are shared under `.agents/roles/`.
No nonexistent optional role or unfinished parity canary blocks ordinary implementation.
Default to one primary agent and at most one reviewer when risk warrants it.

Use hosted Graphify first for targeted navigation, comparing indexed commit with HEAD.
If unavailable, report it and read the owning source directly; never build a local graph.
Select RTK or Token Guard when useful, preserving raw logs and exit codes. Use risk-based
targeted checks and reuse valid source-bound evidence. Required hosted protections and
security regressions remain mandatory. Do not request optional Copilot reviews while
quota is exhausted or repeatedly retry external failures.

Keep hooks enabled. Legacy local Graphify indexing hooks conflict with hosted-only policy;
record that conflict and preserve required validation hooks when using a safe hook path.
Do not silently disable validation hooks or run a forbidden local index.
Update the existing compact checkpoint and progress assessment; commit/push coherent work
and open/update a draft PR. Manual squash requires actual mandatory checks and owner
mandate (the six workstreams hold it for their own PRs under ADR-MERGE-DELEGATION-20261123).
Use the guarded squash/auto-merge procedure in AGENTS.md; disable auto-merge before
changing a head or base and reassess it afterwards. Never bypass protections,
weaken permissions or disturb foreign WIP. Continue independent or correctly
stacked work while a submitted PR waits; verify its actual landing later.
Governance does not require zero open items or prove runtime parity/Daybreak remediation.

## Read-only canary instruction

Use only when requested for validation: **do not edit, commit, push, run network installers or merge**. Report the checkout HEAD, which instruction layers were loaded from this start directory, whether hosted Graphify tools and token controls are genuinely available, a relevant Graphify symbol/test lookup if possible, and one safe read-only command with its exact exit code. Report unavailable items as `NOT RUN`; do not claim a completed autonomous loop. Follow `docs/CODEX-PARITY.md` for the acceptance checklist.
