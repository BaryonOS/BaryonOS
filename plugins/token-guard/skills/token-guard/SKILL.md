---
name: token-guard
description: "Reduce coding context and tool-output overhead while preserving source contracts, permissions and test evidence. Use for token-conscious development, debugging, reviews and BaryonOS tasks in ChatGPT, Codex, Claude, Copilot or OpenCode."
---

# Token Guard

Use one copy of this protocol per task. Existing system, user and repository instructions remain authoritative. This plugin supplies workflow guidance and a local Python helper; it does not change model settings, provider billing, permissions or connected services.

## Plan

- Start from the requested task, actual branch/HEAD and working tree. Preserve existing changes.
- Read the repository's required startup documents once, focusing on the current checkpoint and affected contracts. In BaryonOS, preserve `AGENTS.md`, `CODEX-WORKFLOW.md`, `PROMPT-VERVOLG.md` and `docs/PROJECT-STATE.md` requirements. Extend context when correctness or security requires it.
- State one bounded goal, touched files, acceptance command and evidence limit. Reuse an existing implementation or PR before creating another.
- Use available hosted Graphify for definitions, relationships, impact and tests. Verify its indexed revision against the worktree. For BaryonOS, use `BaryonOS/BaryonOS`; never create a local substitute graph. Report unavailable access and use targeted reads.
- Search literal/configuration text with a scoped query. Avoid repeated plugin-catalog reads and generated-directory scans unless a specific task requires them. Batch independent reads; do not repeatedly fetch unchanged results.

## Do

- Reuse existing helpers, then the standard library, then the smallest correct local change. Do not add speculative abstractions, extra services or unrelated refactors.
- Read the needed implementation and its callers before editing; there is no arbitrary read-count limit.
- Use native targeted edits. Do not repeat full files in chat. Preserve identifiers and security-relevant details.
- Prefer the repository's `tools/tokenguard.py` when present. Otherwise run this skill's bundled `scripts/tokenguard.py` using its absolute path supplied by the skill loader. Run from the project root, or pass `--root /absolute/project` before the subcommand.
- `inspect PATH` returns a source hash and outline. Python uses AST; other-language outlines are heuristic, never a substitute for Graphify or the actual contract. `slice PATH START END` reads up to 200 lines; `--expect-sha HASH` detects a changed source version.

## Check

- Run noisy checks as `python3 -B tools/tokenguard.py run -- COMMAND ARG...`. Arguments are passed directly, without an implicit shell. For an explicitly needed shell, supply that shell and its arguments under the normal client permission flow.
- Use `run --output bounded -- COMMAND ARG...` for data-producing commands such as Git status/diffs. Do not reduce useful query results to an OK marker. An explicitly truncated diff is incomplete; retrieve the remaining relevant ranges before reviewing it.
- Inspect the real exit status, warnings, test summary and source context. Full combined output and receipts remain in `build/tokenguard/`; `log PATH --start N --lines N` retrieves a bounded excerpt. Logs may contain command-produced sensitive data: keep them local and do not upload without authorization.
- The helper uses an available RTK log filter only after execution, retains original diagnostics and never lets a filter turn failure into success. Do not wrap an already-compressed command again. A missing RTK executable does not prevent the built-in bounded output.
- Run the relevant targeted test, review the diff, then the required integration gates. Missing tools, timeouts, skipped checks and external CI still pending are explicit limitations. Never weaken a gate, reuse stale evidence or replace a security check to save tokens.
- In BaryonOS, preserve strict preflight, build and smoke requirements. Parallelize independent bounded work only when useful; give each writer separate ownership/worktrees and keep integration responsibility with the lead.

## Act

- Correct concrete failures and repeat only affected checks. Stop redundant verification once the required evidence is complete.
- Update the existing state/evidence files at meaningful checkpoints; retain unresolved risks, source identity, commands, exit codes and the next action. Do not create a competing roadmap or force a conversation reset after every small step.
- Use `stats` for raw-versus-displayed byte measurements. Bytes are not provider tokens, costs, quota savings or proof of unchanged reasoning quality. Inspect actual client usage before claiming a saving.
- Keep progress and final reports brief: result, evidence, remaining limit and next step. Do not claim a plugin controls hidden reasoning, guarantees a cache discount or makes unavailable tools accessible.

## Client boundaries

Claude's bundled hook wraps only recognised simple build/test commands and preserves all tool input fields. Shell expressions, reads, Git commands and existing wrappers remain unchanged. It returns no permission decision. Other clients use the same CLI through this skill; a shell and Python 3 are required. In a chat without execution access, use available connectors and report checks as not executed. Existing Graphify authentication and client permissions remain in place.
