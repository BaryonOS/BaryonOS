# Token Guard

Token Guard reduces unnecessary source reads and command output while preserving
the command's exit status and access to its complete log. One plugin contains
the shared skill, a standard-library Python CLI and a native Claude hook.
Repository architecture, capability contracts and required gates remain binding.

Version 1.1 makes quality precedence explicit, adds an on-demand UI/UX contract,
preserves short text output and makes automatic Claude wrapping opt-in. Neither
the helper tests nor smaller logs establish equal model or design quality.
The [quality and options review](TOKEN-GUARD-QUALITY.md) separates implemented
mechanisms, host-controlled features, proposals and unverified outcomes.

## Components

| Component | Purpose |
| --- | --- |
| `plugins/token-guard/` | Portable plugin and Claude compatibility manifest. |
| `plugins/token-guard/skills/token-guard/SKILL.md` | Canonical workflow. |
| `.agents/skills/token-guard/SKILL.md` | Codex and OpenCode adapter. |
| `.github/skills/token-guard/SKILL.md` | Native Copilot skill adapter. |
| `.claude/skills/token-guard/SKILL.md` | Small adapter linking to the canonical skill. |
| `AGENTS.md` and `CLAUDE.md` | Existing project rules and Claude's `@AGENTS.md` import. |
| `.github/copilot-instructions.md` | Existing Copilot instructions and a skill reference. |
| `tools/tokenguard.py` | Launcher for the plugin's single CLI implementation. |
| `plugins/token-guard/hooks/hooks.json` | Conservative Claude build/test hook. |
| `plugins/token-guard/skills/token-guard/references/quality-contract.md` | On-demand UI/UX and comparison requirements. |
| `build/tokenguard/` | Ignored runtime logs and measurements; not source evidence. |

No additional MCP server, provider proxy or local Graphify index is required.
The hosted Graphify MCP remains the source of code relationships. Its results
represent `main` and must be checked against the working tree. The CLI does not
call Graphify or change authentication, sandbox policy or tool permissions.

## Client use

**Codex:** launch in the repository. Codex discovers `AGENTS.md` and repository
skills under `.agents/skills`; `$token-guard` can select the skill explicitly.
`allow_implicit_invocation: true` permits automatic selection; it does not
guarantee that the skill is active on every request.
The repository's `.agents/plugins/marketplace.json` exposes the complete plugin;
`.codex/config.toml` enables it when the checkout is already trusted. No trust,
approval or MCP settings are changed (OpenAI, n.d.-a, n.d.-b, n.d.-c).

**Claude Code:** the existing `CLAUDE.md` imports `AGENTS.md`. The adapter under
`.claude/skills` exposes `/token-guard` and loads the shared protocol on demand.
Load the complete plugin with `claude --plugin-dir ./plugins/token-guard` for
its components and `/token-guard:token-guard` command. The hook is a no-op unless
`TOKENGUARD_AUTO_WRAP=1` is explicitly set. For persistent installation,
run `claude plugin marketplace add .` from this checkout, then
`claude plugin install token-guard@baryon-tools`. The catalog declares one plugin;
it does not add another runtime or MCP server. The project adapter and installed
plugin refer to the same protocol; load its body only once (Anthropic, n.d.-a,
n.d.-b, n.d.-e). The installer does not automatically configure another computer.

**GitHub Copilot:** repository instructions and a native `.github/skills`
adapter reference the same skill. Use `/token-guard` where the client supports
slash invocation. VS Code distinguishes harness-specific skill locations; an
`.agents/skills` adapter alone is not claimed to register every Copilot client.
Ordinary inline completion is not evidence that the agent workflow has run
(GitHub, n.d.; Microsoft, n.d.).

**OpenCode V2:** `AGENTS.md` and `.agents/skills` are discovered natively; the skill
can be selected as `@token-guard`. Do not add a duplicate `instructions` entry
for `AGENTS.md`. No new `opencode.json` MCP configuration is required
(OpenCode, n.d.-a, n.d.-b).

**ChatGPT/Codex account plugin:** the private Token Guard package is saved through
Plugin Creator. Select that plugin for its bundled skill; no separate personal
skill is needed. Repository files alone do not activate an account plugin.
Execution requires a checkout, Python 3.10 or newer, and a terminal tool. A chat
with connector access alone must report local gates as not executed. No local
MCP process is advertised as a web-accessible service (OpenAI, n.d.-a).

**Other repositories:** the canonical skill already uses the absolute bundled
helper path supplied by the skill loader when a repository launcher is absent.
The Claude hook also uses its installed absolute helper path and the payload's
working directory. An account plugin install does not install local clients.
For a skill-only Copilot/OpenCode setup, place the complete `skills/token-guard/`
directory, including `scripts/` and `references/`, in that client's documented
skill directory. Do not copy only BaryonOS's thin adapter into another project.
No `init` command or automatic rewriting of existing MCP configuration is provided.

## Run checks and inspect evidence

Commands use an argument vector after `--`; Token Guard does not invoke a shell.
The child inherits stdin and environment. Stdout/stderr are combined into a
non-TTY file, and POSIX process-group handling is used for cancellation. These
are observable differences from arbitrary native execution. Use the wrapper only
for suitable foreground text checks; interactive, protocol, background/watch and
TTY-sensitive tasks keep the native execution tool. Pipes, redirections and
command chaining require an explicit shell/script under normal permissions.
There is no implicit helper timeout; `--timeout` sets an explicit finite deadline.
Do not wrap Git diffs, status output or other requested data in a summary
mode that would conceal the result needed for the task.

```sh
python3 -B tools/tokenguard.py run --timeout 600 -- python3 tools/preflight.py --strict
python3 -B tools/tokenguard.py run --timeout 600 -- python3 tools/build.py
python3 -B tools/tokenguard.py run --timeout 600 -- python3 tools/run.py --smoke
```

These examples show the standard gate interface. Select targeted tests first;
run the broader gates required by the actual change and repository instructions.
Each command must complete before a dependent command starts. A wrapper success
message establishes only that command's result, not OS acceptance or CI success.

`run` reports the exit status and saved log location. A failure adds bounded
diagnostic output and source context where a valid location is available.
Its process exit status preserves failure for callers and CI. Raw logs remain
available even when the model-facing result is shortened. RTK log filtering is
optional; its presence does not decide whether the original command passed.
Short output remains complete sanitized text in both modes and bypasses RTK.
Larger selected output carries an explicit incomplete indication and
`output_complete: false` in the receipt. Completeness refers to sanitized text;
terminal controls, binary bytes and independent stream identity are not preserved
in the displayed form. The original combined bytes remain in the raw log.
Automatic source snippets are labelled as current source read after the command;
they do not establish which source version the command tested.
The console prints only the compact receipt fields needed for the next decision;
the adjacent `.json` file retains capture, stdin, deadline and sanitization details.

```sh
python3 -B tools/tokenguard.py run --output bounded -- python3 tools/progress.py --check
python3 -B tools/tokenguard.py inspect tools/build.py
python3 -B tools/tokenguard.py slice tools/build.py 80 110
python3 -B tools/tokenguard.py stats
```

Use `--root DIR` before the subcommand when the working directory differs from
the repository root. Use `slice PATH START END --expect-sha HASH` with the hash
from a previous inspection to detect source changes before relying on its lines.
Read more context when a contract or diagnostic requires it; no fixed number of
reads is a correctness criterion. An outline is navigation, not a complete
semantic analysis. Python AST support does not imply AST support for all languages.

For a longer diagnostic, use `log PATH --start N --lines N` with the log path
reported by `run`. For a very long single line, use
`log PATH --offset N --length N` (raw byte offsets; at most 12,000 bytes per read).
The byte view reports its next offset and file size; text display is sanitized
and a UTF-8 character split at the boundary can be replaced. Raw files remain
authoritative for machine comparison. Do not mix line and byte options.
Keep runtime logs out of commits; record compact, versioned
evidence in the existing verification documents. Update `PROJECT-STATE.md` at
the normal checkpoint, including the source version, checks and next action.

## Claude plugin hook

Loading the complete Claude plugin registers a `PreToolUse` hook. By default it
returns no rewrite. `TOKENGUARD_AUTO_WRAP=1` explicitly enables wrapping for
suitable foreground text checks; for example, set that environment variable when
launching Claude after reviewing the capture limits above. The enabled hook returns
`hookSpecificOutput.updatedInput` to replace a tool's input. It must retain the
other input fields, use `hookEventName: "PreToolUse"`, and omit
`permissionDecision` so the normal permission flow remains applicable.
Claude's auto mode may reject a rewritten call; explicitly using the CLI is
the fallback, not adding an unconditional allow decision (Anthropic, n.d.-c).

The hook only wraps unambiguous, simple build/test commands. It leaves
Git commands, output reads, existing wrappers, shell operators, substitutions,
environment assignments, background/watch/interactive requests and unsupported
quoting untouched. It must preserve
argument boundaries, use the payload's `cwd` for the active worktree and avoid
running the command itself. Invalid input should produce no rewrite.
The manifest uses a quoted plugin-root path. It does not run the build inside
the hook or return an allow decision. Codex, Copilot and OpenCode use the explicit
CLI workflow; Claude's hook API is not assumed portable to those clients.
The OpenAI manifests explicitly select an empty hook set, so the Claude hook
is not accidentally registered through OpenAI's default hook-file discovery.
The helper has no cache-blocking rule, blanket allow decision, local Graphify
lookup or MCP-server mode. Registering `tokenguard.py mcp` would not create a server.

## Measurement and limits

`stats` measures locally observed output reduction, not billed provider tokens.
Compare equivalent tasks, source revisions and test outcomes; record extra reads,
retries and execution time as well as output size. No fixed saving percentage or
absence of quality loss is established by smaller logs alone. Small outputs can
grow because status/evidence metadata costs bytes. Both presentation modes retain
short sanitized output, including single-line JSON; `--output bounded` never
applies a generic RTK log filter.
Large output is explicitly sampled/truncated; the full log remains authoritative.

On POSIX, timeouts and handled cancellation stop the command's process group.
Uncatchable SIGKILL cannot run cleanup. Windows cleanup covers the direct child
only; descendant cleanup is not verified there. Raw log size is not capped and
the helper is not a sandbox. No secrets should be deliberately passed to checks.

Prompt caching belongs to the provider and client. Claude documents skill
invocation as an appended message that preserves the existing cached prefix;
loading a skill does not inherently invalidate the cache (Anthropic, n.d.-d).
A compact stable protocol can reduce repetition but cannot guarantee a cache hit
or subscription credit saving. Clearing context after every small change is not
required; use a checkpoint when changing tasks or handing work over.

Configuration compatibility is based on the official documentation reviewed on
14 November 2026. Claude Code, Copilot and OpenCode were not installed in the
implementation environment. Codex was available, but its presence alone does not
prove the complete workflow ran. Executed verification and remaining client
checks belong in the current project checkpoint; do not infer runtime evidence
from these configuration descriptions.

## References

- Anthropic. (n.d.-a). *How Claude remembers your project*. Retrieved November 14, 2026, from https://code.claude.com/docs/en/memory
- Anthropic. (n.d.-b). *Extend Claude with skills*. Retrieved November 14, 2026, from https://code.claude.com/docs/en/skills
- Anthropic. (n.d.-c). *Hooks reference*. Retrieved November 14, 2026, from https://code.claude.com/docs/en/hooks
- Anthropic. (n.d.-d). *How Claude Code uses prompt caching*. Retrieved November 14, 2026, from https://code.claude.com/docs/en/prompt-caching
- Anthropic. (n.d.-e). *Create a Claude Code plugin*. Retrieved November 14, 2026, from https://code.claude.com/docs/en/plugins/create
- GitHub. (n.d.). *Adding agent skills for GitHub Copilot*. Retrieved November 14, 2026, from https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/customize-cloud-agent/add-skills
- Microsoft. (n.d.). *Use Agent Skills in VS Code*. Retrieved November 14, 2026, from https://code.visualstudio.com/docs/agent-customization/agent-skills
- OpenAI. (n.d.-a). *Build skills*. Retrieved November 14, 2026, from https://learn.chatgpt.com/docs/build-skills
- OpenAI. (n.d.-b). *Model Context Protocol*. Retrieved November 14, 2026, from https://learn.chatgpt.com/docs/extend/mcp?surface=cli
- OpenAI. (n.d.-c). *Package your plugin*. Retrieved November 14, 2026, from https://developers.openai.com/plugins/build/plugins
- OpenCode. (n.d.-a). *Instructions*. Retrieved November 14, 2026, from https://opencode.ai/docs/instructions
- OpenCode. (n.d.-b). *Skills*. Retrieved November 14, 2026, from https://opencode.ai/docs/skills
