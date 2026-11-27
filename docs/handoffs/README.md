# Handoffs

Any capable agent, whatever its client or model, must be able to resume a
workstream from the repository and GitHub alone. It must not need chat history,
a client's private memory or one specific MCP server. These rules apply that
owner requirement of 27 November 2026 and extend item 4 of
ADR-SPECIALIST-TEAM-20261125.

## Canonical handoff

A handoff has two parts. Both are plain Markdown, with JSON where it helps.

1. `docs/handoffs/<X>/CURRENT.md` is the durable resume pointer.
   - It records the scope, mandate, limits, durable locations and resume procedure once.
   - Under ADR-SPECIALIST-TEAM-20261125 it holds no live head SHAs, locks, PIDs or cursors.
   - It links to the thread that carries the live handoff.
2. The newest comment starting with `HANDOFF <X>`, where `<X>` is the role (for
   example `HANDOFF G`), in that linked pull request or issue holds the live
   state. The role tag keeps handoffs on a shared thread apart. The comment uses
   the required fields below.

Client-specific files are adapters only: Codex start prompts, Claude or Ollama
usage notes, and `.agents/roles/*` adapters generated for `.claude`, `.codex`
and `.github`. They hold no task state.

## Required fields of a live handoff

1. Task and intended outcome.
2. Invariants and security boundaries.
3. Owner and exact write set.
4. Repository, branch and exact HEAD.
5. Dirty or uncommitted work, and who owns it. Before a transfer, the work is
   committed and pushed, or attached as a non-secret patch, or recorded as
   abandoned; a description alone does not transfer it.
6. Decisions, each with a source reference (an ADR ID, or a linked owner decision with its UTC time).
7. Actions taken, including Graphify record or version IDs.
8. Commands with exact exit codes, and evidence or log paths.
9. What was not run.
10. Risks, blockers and owner questions.
11. The single next safe action.
12. Acceptance commands and evidence limits.

## Rules

- Graphify memory is optional context. Every binding fact must be reproducible
  from the repository or the handoff, and every resume procedure that uses
  Graphify names a fallback that needs only the repository and GitHub.
- The canonical state contains no model-specific or client-specific instructions.
- A fact that exists only in one client's private state counts as not done.
- Update the canonical handoff before an ownership change, a context rotation
  or a stop.
- A resuming agent first verifies branch, HEAD, worktree status, ownership and
  the freshness of the handoff, and only then writes.
- Task state stays out of every `AGENTS.md` file.
- Public text contains no secrets, tokens or local session IDs. Local paths
  may appear only as an optional addition, never as the only location of a fact.
- Times are UTC, taken from `date -u` when written.
