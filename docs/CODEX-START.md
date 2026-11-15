# Portable Codex start and recovery prompt

Use this text as the **initial prompt** in an authenticated Codex session started from a BaryonOS repository checkout. This file is a stable startup guide, not live task state.

Before editing, load the user's global `~/.codex/AGENTS.md`, repository `AGENTS.md` and each applicable directory `AGENTS.md`. Confirm current branch, HEAD, dirty files and existing task owner. Read only `CODEX-WORKFLOW.md`, `PROMPT-VERVOLG.md` and the **latest relevant** `docs/PROJECT-STATE.md` checkpoint first; then follow the applicable ADRs, `docs/ROADMAP.md`, security contracts and focused subsystem skill. Historical directions do not override accepted work or newer owner decisions.

Find exact definitions and linked tests through the **hosted Graphify MCP** (`recall`, `graphify_rank_files`/`graphify_find`, `graphify_impact`, `graphify_tests_for`) before broad source inspection. Check the Graphify indexed commit against the checkout. If Graphify is unavailable, use small direct source reads, mark the limitation, and do not construct a substitute local index. Never perform broad recursive grep or dump entire logs as routine navigation.

Operate each task through the one shared loop: (1) orient, (2) claim one eligible task with exclusive write paths, (3) obtain bounded context, (4) specify invariants and acceptance, (5) make the smallest safe change following Ponytail, (6) run targeted tests then mandated native/hosted checks using a single suitable RTK **or** Token Guard wrapper per command, (7) inspect the entire diff and repair failures, (8) checkpoint branch/commit/test evidence and draft PR, (9) record one durable lesson in the right place, (10) continue the next safe task or leave a precise blocker. Raw exit codes and complete raw logs remain authoritative; do not downgrade acceptance to compensate for missing tools. Keep Git hooks enabled; do not use `--no-verify` or change `core.hooksPath` to bypass them.

Keep changing tasks/decisions/logs in `docs/PROJECT-STATE.md`, `docs/DECISIONS.md`, `docs/verification/`, ignored `build/` and GitHub; never in `AGENTS.md`. Only a lead may propose durable instruction updates in a separate reviewed `docs(agents):` change. Do not edit policy files, pipeline evaluators, state registers or other agents' write sets from an ordinary worker. Public repository prose and comments are in professional English.

If context is insufficient, construct a **read-only plan** using hosted Graphify and exact contracts; planning is not a request to change Codex's UI mode automatically. If no safe task remains, or a security/authority decision is missing, record one focused `OWNER:` question and continue only disjoint authorized work. Do not guess or change permissions, production keys, merge controls, disks or device firmware.

## Read-only canary instruction

Use only when requested for validation: **do not edit, commit, push, run network installers or merge**. Report the checkout HEAD, which instruction layers were loaded from this start directory, whether hosted Graphify tools and token controls are genuinely available, a relevant Graphify symbol/test lookup if possible, and one safe read-only command with its exact exit code. Report unavailable items as `NOT RUN`; do not claim a completed autonomous loop. Follow `docs/CODEX-PARITY.md` for the acceptance checklist.
