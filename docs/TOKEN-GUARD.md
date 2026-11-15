# Token Guard

Token Guard 1.2 provides one coding workflow, a Python CLI and three native MCP
tools, while preserving real command exits and complete local logs. The required
quality contract is inline in the skill. Optional Claude hooks guide reads and
adapt suitable build/test commands. Hosted Graphify remains the architecture and
dependency source; no local replacement graph or provider proxy is installed.
Neither compact output nor helper tests establish equal model or UI/UX quality.
The [options review](TOKEN-GUARD-QUALITY.md) records implemented and unproven ideas.

## Components

| Component | Purpose |
| --- | --- |
| `plugins/token-guard/plugin.json` | Portable Agent Plugins identity and OpenAI presentation. |
| `.claude-plugin/plugin.json`, `.codex-plugin/plugin.json` inside the plugin | Host-specific compatibility manifests; not identical schemas. |
| Plugin `mcp.json` | OpenAI portable stdio launch configuration. |
| Plugin `.mcp.json` | Generated-compatible OpenAI legacy stdio configuration. |
| Plugin `config/claude-mcp.json` | Explicit Claude override with Claude plugin/project variables. |
| Plugin `skills/token-guard/SKILL.md` | One canonical workflow with five mandatory quality rules. |
| Plugin `scripts/tokenguard*.py` inside the skill | Shared command evidence, MCP transport, installer and optional read guidance. |
| `tools/tokenguard.py` | Repository launcher for the same implementation. |
| Plugin `hooks/hooks.json` | Claude `Bash|Read|Grep|Glob` matcher; behavior is opt-in. |
| `.agents/skills`, `.claude/skills`, `.github/skills` adapters | Host-specific discovery of the shared workflow. |
| Plugin `references/quality-contract.md` inside the skill | Extended explicit audit/evaluation reference; no routine prerequisite read. |
| `build/tokenguard/` | Ignored local logs and receipts. |

Claude automatically discovers its standard skill/hook locations. Its manifest
explicitly references the dedicated Claude MCP file. Readback of account release
1.2.0 showed that the OpenAI service regenerates `.mcp.json` from portable
`mcp.json`; version 1.2.1 keeps Claude's variables in a separate override under
the same server name. Claude merges its default MCP map first and later definitions
replace a matching server entry (Anthropic, n.d.-f). Explicitly
referencing the same default hook file can merge it twice; a small Claude manifest
is therefore not evidence of an inactive hook. Root portable `plugin.json` must
not receive unsupported top-level `skills`/`mcpServers` fields. The OpenAI extension
and Codex overlay deliberately select an empty hook set, preventing Claude's hook
from being loaded through OpenAI's default hook discovery (Anthropic, n.d.-f;
OpenAI, n.d.-c). Names, versions and descriptions are synchronized across manifests.

## Client setup

**BaryonOS/Codex:** launch in a trusted checkout. `AGENTS.md` and the skill adapter
remain authoritative; `$token-guard` selects the skill explicitly. Repository
marketplace/configuration enables the plugin; current portable clients discover
`mcp.json`. `allow_implicit_invocation: true` permits selection, not permanent
activation. The older Codex overlay is retained for skill compatibility; legacy
MCP loading from that overlay alone is not claimed. Use `init` when needed
(OpenAI, n.d.-a, n.d.-b, n.d.-c).

**Claude Code:** use `claude --plugin-dir ./plugins/token-guard` in this checkout,
then `/token-guard:token-guard`. For persistent installation, run
`claude plugin marketplace add .`, then `claude plugin install token-guard@baryon-tools`.
The installed plugin uses absolute plugin/project variables. Existing `CLAUDE.md`
and `.claude/skills` adapters remain usable; load the canonical skill only once.
Hooks do nothing by default. Enable `TOKENGUARD_AUTO_WRAP=1` for suitable command
rewrites and/or `TOKENGUARD_READ_GUARD=1` for bounded-read guidance in a trusted
workspace (Anthropic, n.d.-a, n.d.-b, n.d.-c, n.d.-e, n.d.-f).

**GitHub Copilot/VS Code:** the `.github/skills` adapter supplies the workflow.
Current Copilot workspace MCP and editor-managed VS Code MCP have distinct setup
targets below; do not assume all Copilot surfaces use the same loader. Inline
completion is not evidence of agent-workflow execution (GitHub, n.d.; Microsoft, n.d.).

**OpenCode V2:** `AGENTS.md` and `.agents/skills` provide the workflow. Use the
`opencode` init target for local MCP tools; do not duplicate `AGENTS.md` in an
`instructions` array (OpenCode, n.d.-a, n.d.-b, n.d.-c).

**ChatGPT/Codex account plugin:** select the saved private Token Guard plugin for
its bundled skill. Local tools need a checkout, Python and a local executor;
saving a package does not make a local stdio server accessible to web/mobile.
Connector access alone is insufficient for local gates. Do not claim activation
on a user's computer from account storage alone (OpenAI, n.d.-c).

**Other repositories:** use the complete installed plugin or copy the complete
`skills/token-guard/` directory into the client's supported skill location.
Do not copy only BaryonOS's thin adapter. Use the bundled helper's absolute path
and `--root /absolute/project`; do not assume `tools/tokenguard.py` exists there.

## Preview and apply workspace configuration

Use a stable extracted plugin location. The CLI needs Python 3.10 or newer;
Codex TOML merging uses Python 3.11's standard `tomllib`. On Windows, the initial
command may be `py -3` or `python`; generated configs use `sys.executable`.

```sh
python3 -B tools/tokenguard.py --root /absolute/BaryonOS init --clients claude codex copilot vscode opencode
python3 -B tools/tokenguard.py --root /absolute/BaryonOS init --clients claude codex copilot vscode opencode --apply
```

Without `--apply`, this is a read-only plan. Apply validates all selected files
before writing, preserves hosted Graphify and other entries, and uses staged
per-file atomic replacement with backups/rollback. This is not a multi-file
filesystem transaction. Unsupported syntax, duplicate keys, conflicting Token
Guard entries and path escapes fail without silent overwrites. Existing JSONC
comments/configuration remain intact. New targets use atomic no-overwrite creation;
existing-file updates use optimistic snapshot checks and backups. Do not edit the
same configs concurrently: non-cooperating editors can race between checking and
replacing an existing file. Legacy OpenCode V1 server maps are refused rather than
silently mixed with the V2 schema. Re-run setup after relocating a plugin;
a versioned cache path can disappear after an upgrade. No trust/approval is enabled.
Do not register the same server through both native plugin and workspace config.

| Target | Configuration |
| --- | --- |
| `claude` | Project `.mcp.json`, `mcpServers`; not `.claude/settings.json`. |
| `copilot` | Current workspace `.mcp.json`, common `mcpServers` stdio entry. |
| `vscode` | Editor-managed `.vscode/mcp.json`, `servers`. |
| `codex` | `.codex/config.toml`, `mcp_servers.token-guard`. |
| `opencode` | Current V2 `opencode.json`/`.jsonc`, `mcp.servers`. |

The shared Claude/Copilot entry contains common stdio fields; outer client
timeouts still apply. The installer only adds documented timeout settings.

## MCP protocol and execution boundary

`mcp` serves `inspect_file`, `read_slice` and `run_compact` as UTF-8 newline-delimited
JSON-RPC. Versions `2025-11-25` and `2025-06-18` use `initialize`, id-less
`notifications/initialized` and `ping`. Version `2026-07-28` uses per-request
metadata, `server/discover`, `resultType: complete` and list cache hints. Older
initialization requests negotiate an offered version; pre-June-2025 batch support
is not advertised. Notifications never execute tools or receive replies.
Explicit `id: null` is invalid, not a notification (Model Context Protocol, 2026a, 2026b).

`run_compact` accepts a literal **argv array**, such as
`["python3", "tools/preflight.py", "--strict"]`, without an implicit shell.
MCP children receive `stdin=DEVNULL` and a finite deadline: 300 seconds by default,
configurable via `TOKENGUARD_TIMEOUT` or `timeout_seconds`, maximum 3600 per call.
Native CLI keeps inherited stdin/environment and no implicit deadline. RTK's
optional post-run filter has its own three-second timeout and closed stdin.

Only one MCP command runs at a time; protocol pings/discovery/read tools remain
responsive. Cancellation or EOF stops active work and suppresses its response.
A failed command returns `isError: true` plus its actual exit status and log path;
malformed requests use JSON-RPC errors. Frames and source reads are bounded.

**Token Guard is not a sandbox.** An argv command can modify files, access network
or explicitly invoke a shell, and its MCP annotations declare this. Keep host MCP
approval and sandbox controls; native shell allow/deny rules do not automatically
transfer to MCP. Use native execution when that permission boundary or execution
mode is required. Read tools are confined to the configured project root.

Claude supplies `${CLAUDE_PROJECT_DIR}` explicitly. OpenAI portable configuration
uses `${PLUGIN_ROOT}` for the executable and inherits launch cwd; no undocumented
Codex project-root variable is assumed. Initialization/discovery reports the actual
root. Use `init` for an explicit absolute workspace binding when necessary.

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

## Claude hooks

The optional command hook returns `hookSpecificOutput.updatedInput` while retaining
all other input fields. It never returns an unconditional permission allow decision.
It recognizes only unambiguous foreground build/test argv. Git/data commands,
existing wrappers, substitutions, operators, background/watch/interactive requests
and unsupported quoting retain native execution. Invalid input produces no rewrite.
A host refusal must not be bypassed through another tool (Anthropic, n.d.-c).

The optional read guard pauses a recognized source file over 150 lines when the
read has no suitable bound. Exit 2 includes its bounded outline, removing the need
for a separate inspect call. The blocked call still occurs; this is not literally
zero turns. `Read` with offset/limit up to 160, instruction Markdown, images, scoped
dependency queries and exact plugin files remain accessible. Simple literal `cat`
and `Get-Content` reads inside a Bash payload are recognized; arbitrary shell
programs are not parsed. This does not intercept a separate native PowerShell tool.

No blanket `token-guard` substring grants a bypass and no absolute plugin-cache
ban exists. Those approaches would be easy to evade or prevent required instructions
from loading. Broad generated/cache scans receive guidance only with the opt-in
guard; targeted reads remain possible. PreToolUse itself can read source before
normal tool approval, so enable it only in trusted workspaces with authorized
read access. It is guidance, not access control. Claude hooks are not assumed
portable to Codex, Copilot or OpenCode.

## Measurement and limits

`stats` measures raw and helper-output UTF-8 bytes, excluding MCP JSON framing,
tool definitions and client history. These are not billed tokens, credits, cache
hits or quality. Compare equivalent tasks, revisions and outcomes, including extra
reads/retries/time. Small outputs can grow due to evidence metadata. Short output
is complete sanitized text; large output is selected and always labelled incomplete.
An outline is navigation: Python uses AST; Rust/TypeScript use a bounded heuristic,
not macro expansion or full syntax/contract verification. Read needed bodies.

POSIX timeouts/cancellation stop the command process group; SIGKILL cannot run
cleanup. Native CLI Windows cleanup covers the direct child. MCP cancellation
attempts Windows `taskkill /T`, with direct-process fallback; native Windows
cleanup is unverified here. Raw logs are uncapped local files and may contain
command-produced sensitive data. They are not uploaded automatically.

Prompt caching belongs to provider/client. Claude documents skill invocation as
appending a message while preserving the existing cached prefix; a skill read does
not inherently destroy a 90% discount (Anthropic, n.d.-d). This plugin does not
control model routing, hidden reasoning, context windows or subscription charges.

Compatibility follows official documentation reviewed on 14 November 2026.
Protocol/helper tests are not native-client acceptance. Claude, Copilot, OpenCode,
real RTK and Windows were unavailable in this environment. Codex's installed CLI
and valid packaging alone do not establish a full client workflow. No paired
UI/UX agent evaluation, human usability test or provider billing study has run.
The current verification record identifies the actual tested source and limits.

## Reproduce the package

The reproducible package command is `python3 -B tools/package_token_guard.py`.
It validates component paths, schema placement, synchronized metadata and runtime
version, then writes `build/plugin.zip` plus a SHA-256 manifest. The archive uses
an explicit component allowlist, deterministic metadata and executable entrypoint
permissions; bytecode, temporary logs and marketplaces are never included. The
same validation runs in the host-quality CI step.

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
- Anthropic. (n.d.-f). *Plugin manifest reference*. Retrieved November 14, 2026, from https://code.claude.com/docs/en/plugins-reference
- OpenCode. (n.d.-c). *MCP servers*. Retrieved November 14, 2026, from https://opencode.ai/docs/mcp-servers
- Model Context Protocol. (2026a). *Base protocol overview (2026-07-28)*. https://modelcontextprotocol.io/specification/2026-07-28/basic
- Model Context Protocol. (2026b). *Discovery (2026-07-28)*. https://modelcontextprotocol.io/specification/2026-07-28/server/discover
