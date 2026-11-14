# Token Guard 1.2

One coding workflow for bounded context and preserved command evidence. Python
3.10+ is required; the Codex configuration installer requires Python 3.11+.
No pip package, local model or local Graphify index is required.

## Choose one registration route

**Claude Code:** `claude --plugin-dir /absolute/token-guard` loads the skill,
standard hook file and `config/claude-mcp.json` through its manifest. Use
`/token-guard:token-guard`. The dedicated file preserves Claude variables when
an OpenAI package service regenerates the root compatibility `.mcp.json`.

**ChatGPT/Codex:** select/install the Token Guard plugin through the supported
host. Current portable clients use `mcp.json` with `${PLUGIN_ROOT}`. A local
executor is required for local tools; a saved package does not connect a web chat
to your computer. Initialization/discovery reports the actual project root.

**Workspace MCP:** extract into a stable directory and preview a configuration
merge for the clients you use. Replace both absolute paths below.

```sh
python3 -B /absolute/token-guard/skills/token-guard/scripts/tokenguard.py --root /absolute/project init --clients claude codex copilot vscode opencode
```

Add `--apply` to apply the validated plan. On Windows the first command may use
`py -3` or `python`; generated entries use the actual Python executable. Existing
Graphify/other servers are retained. Unsupported/conflicting configuration fails
without silent replacement. Do not register the same MCP server through both a
native plugin and a workspace entry. Update the binding if the plugin path moves.

## Runtime behavior

MCP tools: `inspect_file`, `read_slice`, `run_compact` (literal `argv` array).
MCP command stdin is closed; the default deadline is 300 seconds, configurable up
to 3600. CLI `run -- COMMAND ARG...` retains stdin/environment and no implicit
deadline. Both preserve exits and full logs in the project's `build/tokenguard/`.
Commands are not sandboxed by this plugin; host MCP approval still applies.

Claude hooks are opt-in: `TOKENGUARD_AUTO_WRAP=1` and/or
`TOKENGUARD_READ_GUARD=1`, only for a trusted workspace. Native shell permissions
do not automatically transfer to MCP. Interactive and TTY-sensitive commands
use their native execution path. Hosted Graphify remains the architecture source.

Quality rules are inline in the skill. Smaller output is not a guarantee of equal
UI/UX, reasoning or provider billing. Extended setup, evidence and limitations:
https://github.com/BaryonOS/BaryonOS/blob/main/docs/TOKEN-GUARD.md
