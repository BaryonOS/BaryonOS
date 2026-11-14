---
name: token-guard
description: "Reduce coding context and tool-output overhead while preserving contracts, permissions and test evidence. Use for development, debugging, reviews and BaryonOS tasks in ChatGPT, Codex, Claude, Copilot or OpenCode."
---

# Token Guard

Load this protocol once. System, user and repository instructions remain authoritative. Hosted Graphify supplies architecture and dependency navigation; this plugin supplies bounded local reads and command evidence. It changes neither model settings nor provider billing.

## Quality contract — always applies

- Preserve the complete requested scope, approved design, source contracts, model/reasoning settings and acceptance gates. Quality takes precedence over token savings; there is no fixed read/turn budget.
- Outlines and summaries are navigation, not evidence of complete behavior. Read required implementations/callers; recover omitted or changed evidence before editing or claiming success.
- For UI/UX, retain approved reference paths/hashes and compare actual before/after renders with the same viewport, scale, theme, fonts and data; check interaction, keyboard/focus, errors/recovery, contrast and motion. Label missing native or human validation.
- Preserve real exit codes, warnings, test results and full local logs. Failed, skipped, timed-out or stale checks never count as passing; source snippets are post-command context, not a tested snapshot.
- Preserve requirements, accepted/rejected decisions, source identity, open defects and next action at handoff. Use native compaction without forced resets, hidden model downgrades or invented cache/token/quality guarantees.

Do not open `references/quality-contract.md` during routine tasks: all mandatory rules are above. The extended reference is for an explicitly requested audit or matched quality evaluation, not a prerequisite read.

## Plan

- Check actual branch/HEAD and existing work; reuse implementations and PRs. Read required startup documents once, focusing on the current checkpoint and affected contracts. BaryonOS retains its `AGENTS.md`, `CODEX-WORKFLOW.md`, `PROMPT-VERVOLG.md` and `docs/PROJECT-STATE.md` route.
- State the bounded goal, affected files, acceptance command and evidence limit. Use hosted Graphify for symbols, callers, impact and tests; verify its indexed revision against the worktree. BaryonOS uses `BaryonOS/BaryonOS`, never a local replacement graph. If unavailable, report that limit and use targeted reads.
- Scope literal/configuration searches. Avoid repeated catalogs, generated-directory scans and unchanged reads unless the task needs them. Batch independent queries; expand context when a contract or failure requires it.

## Do

- Reuse repository helpers, then the standard library, then the smallest correct local change. Use targeted native edits; avoid speculative abstractions and full-file repetition in chat.
- Prefer active MCP `inspect_file`/`read_slice`; otherwise use `python3 -B tools/tokenguard.py` in BaryonOS or the installed skill's absolute `scripts/tokenguard.py` path. Pass `--root /absolute/project` before the subcommand. The MCP startup/discovery reports its project root; do not assume the plugin cache is the workspace.
- `inspect PATH` gives SHA-256 and declaration ranges: Python AST, other languages explicitly heuristic with omission markers. `slice PATH START END` reads at most 200 lines/12000 bytes; `--expect-sha HASH` detects changed source. Read actual bodies/contracts as needed.

## Check

- CLI `run [--timeout SECONDS] -- COMMAND ARG...` suits foreground text checks. It inherits stdin/environment with no implicit deadline, but captures combined output without a TTY. Run interactive, protocol, watch/background and TTY-sensitive work natively.
- MCP `run_compact` accepts literal `argv`, never an implicit shell string. Its stdin is closed and its configurable default timeout is 300 seconds, maximum 3600 per call. It can modify files or access network under host MCP approval; native shell permission rules do not transfer. Use native execution whenever that boundary or execution mode is unsuitable.
- Keep useful query output: CLI `run --output bounded --` or MCP `output: "bounded"`. Short output stays complete as sanitized text; large output is explicitly selected. `output_complete: false` requires recovering relevant omitted evidence from the full local log.
- Logs/receipts remain in `build/tokenguard/`. CLI `log PATH --start N --lines N` replays lines; `log PATH --offset N --length N` recovers raw-byte windows, including long lines. Keep command-produced sensitive logs local. RTK is an optional post-run filter, never the execution authority.
- Run the affected test, review the diff, then required integration gates. BaryonOS retains strict preflight, build and smoke requirements. Parallel writers need separate ownership/worktrees. Missing tools and pending CI stay explicit.

## Act and client boundaries

- Fix concrete failures and rerun affected checks. Update existing state/evidence at meaningful checkpoints; stop redundant verification once required evidence is complete. Report result, evidence, remaining limit and next action briefly.
- `stats` measures UTF-8 output bytes, not billed tokens, cache hits, credits or reasoning quality. No local model, automatic model routing or hosted Graphify reconfiguration is installed.
- Claude discovers the skill/hooks and uses its manifest's dedicated MCP configuration; OpenAI portable clients use `mcp.json`. Both require a local executor/Python for local tools. A saved account plugin alone does not grant web/mobile access to a computer. `allow_implicit_invocation` permits skill selection; it does not force every turn to use it.
- Claude command rewriting (`TOKENGUARD_AUTO_WRAP=1`) and native read guidance (`TOKENGUARD_READ_GUARD=1`) are opt-in for trusted workspaces. Read guidance may include a bounded outline in a denial, preserves bounded reads and required instruction files, and is not a permission boundary. Do not bypass a host denial through another tool.
- For workspace adapters, run `init --clients CLIENT...` to preview; add `--apply` to merge supported configuration without replacing other servers. Choose a stable plugin location first; rerun after relocating it. Do not register the same server through both a native plugin and workspace config.
