# Token Guard: quality and option review

Reviewed on 14 November 2026. The objective is less unnecessary context with the
same assignment and acceptance criteria. Equal execution, equal model decisions,
equal UI/UX quality and a fixed billing saving are not established guarantees.

## What the first release proved and did not prove

Version 1.0 had 27 helper regressions and 25 workflow tests passing, a small
synthetic coding trial and an output-byte measurement. Those results concern
the tested helper/workflow behaviors. They do not establish visual quality,
usability, client activation on another computer or unchanged reasoning.

A focused review reproduced five limits: changed stdin/environment/TTY conditions;
an implicit 1,800-second deadline; an unlabelled omitted UI measurement in a
successful log; an inaccessible long-line tail through line-based replay; and
post-command source context being newer than the source used by the command.
Version 1.1 addresses the avoidable differences and makes remaining limits explicit.

## Quality controls in version 1.1

- Quality, required scope, chosen model/reasoning settings and acceptance criteria
  take precedence over savings. No two-read budget, forced session reset or
  automatic model downgrade is introduced.
- Claude automatic wrapping is disabled unless `TOKENGUARD_AUTO_WRAP=1` is set.
  Explicit CLI use remains available. The child inherits stdin and environment,
  with no implicit helper deadline; an explicit timeout remains supported.
- Capture still combines stdout/stderr into a non-TTY file and handles process
  groups on POSIX. Interactive, protocol, watch/background and TTY-sensitive work
  retains native execution. No universal transparency claim is made.
- Short output is retained as sanitized text. Selection from larger output is
  always labelled and recorded as incomplete. Raw combined logs remain local;
  byte-range replay makes omitted long-line content reachable.
- Source snippets explicitly describe the current file after execution. Important
  test evidence must separately identify the tested source; a successful exit is
  not a source-version attestation.
- UI/UX tasks preserve reference images/design decisions and require visual,
  interaction and accessibility evidence. Since 1.2, the mandatory contract is
  inline in five skill bullets. The extended reference is only for an explicit
  audit/evaluation. These are instructions, not completed UI-quality evidence.

## Version 1.2: disposition of the binary-archive audit

The inspected source was the stored 1.1.1 package and merged repository source,
not an unspecified archive inferred from compressed sizes. File size or unchanged
hook JSON cannot establish runtime behavior. The earlier helper did not implement
the audit's `run_compact()`/MCP functions; transport defects in illustrative code
were therefore new-feature requirements rather than reproduced 1.1.1 defects.

| Audit item | Implemented resolution | Deliberate boundary |
| --- | --- | --- |
| Missing native MCP entry | Real stdio server, portable `mcp.json`, root compatibility `.mcp.json` and a dedicated Claude override; three tools use the same helper. | Local execution is required; account storage is not web access to a computer. |
| Identical explicit manifests | Synchronize identity/version/description, preserve each native schema and default discovery. | Claude merges hook defaults; duplicate references can double-load. Portable manifests prohibit the proposed top-level fields. |
| Repeated quality reference read | Five mandatory bullets inline; routine tasks do not open the extended reference. | Explicit audits/evaluations may use the full rubric; original task evidence remains required. |
| Stdin theft and hanging MCP | Isolated command stdin, bounded MCP deadline, responsive protocol loop, cancellation/EOF cleanup. | Native CLI retains inherited stdin and no implicit deadline; TTY/interactive work stays native. |
| Notifications/ping/errors | Legacy initialization/ping plus modern 2026-07-28 discovery and metadata. Notifications do not reply; actual tool failures are `isError`. | A null ID is invalid, not a notification. Unsupported pre-June-2025 batch versions are not advertised. |
| Windows Unicode | CLI UTF-8 stream configuration; MCP input reads UTF-8 bytes and emits safe JSON escaping. | Native Windows/client/process-tree behavior remains unverified. Child environment is not globally rewritten. |
| Multiline signatures/data | Python AST retained; Rust/TS heuristic scans balanced delimiters, where clauses and bounded long headers, with explicit omissions. | A fixed four-line limit or colon delimiter cannot preserve all contracts; no universal AST/100% claim. |
| Compiler progress | Recognizable Cargo progress filtered only in selected large failed summaries; raw logs/diagnostics retained. | Complete short output and data mode remain intact; 25 lines cannot guarantee every error is represented. |
| Expanded read hooks | Opt-in Claude `Bash|Read|Grep|Glob`; a blocked large source read includes its outline. | Not a zero-turn guarantee, permission boundary or portable hook API. Bounded/required reads remain available. |
| Self-block exception | Actual helper invocations do not match the literal-read guard. | No blanket token-guard substring allowlist and no absolute plugin-cache ban. |
| Client bridges | Explicit targets, dry-run plan, staged validated merges, per-file atomic replace, rollback/backups. | Preserve other servers; fail on conflicting/unsupported config. No multi-file atomicity or global trust changes are claimed. |

The [setup guide](TOKEN-GUARD.md) records actual client shapes and protocol versions.
Current protocol and manifest primary sources are listed there. Mandatory quality
rules are never removed to meet a line-count target; adding tool definitions also
has overhead, so an MCP server alone is not a measured token saving.

## Original token-saving proposals

| Proposal | Actual integration or disposition | Quality boundary |
| --- | --- | --- |
| Compile repeated plugin instructions into one small protocol | One canonical skill and thin adapters; mandatory quality rules inline, extended audit material only on explicit request. | Preserve mandatory project/security/design contracts. The full behavior of separate Ponytail/ECC plugins is not claimed to have been cloned. |
| Reduce tool turns and add fault context automatically | Independent reads may be batched by the agent; the helper supplies diagnostic context in the same result. | Dependent steps stay sequential. Current source context is labelled; required follow-up reads remain allowed. |
| AST body stripping and data masking | Python AST outlines and bounded source reads are implemented. Other-language outlines are explicitly heuristic. Generic data masking is not enabled. | An outline is navigation. Values, labels, dimensions, colors, examples and error details must not be erased when they affect decisions. |
| One-shot Graphify plus source inspection | The agent may batch independent hosted Graphify and local inspection calls when the host supports it. The helper does not call Graphify itself. | Repository policy requires the hosted graph and current-source checks; no local graph or duplicate authentication route is created. |
| Hard prohibitions through pre-tool hooks | Opt-in, narrow foreground wrapping and bounded-read guidance, including immediate source outlines. | Cache paths and command substrings are not permission boundaries; required source/instruction reads remain possible. |
| Cheap executor plus expensive advisor and system flags | No automatic model/effort downgrade. Platform-specific settings may be evaluated separately. | A test can pass while design, maintainability or reasoning quality deteriorates; escalation cannot depend only on test failure. |
| RTK/log compression | Optional post-execution filtering for sampled output, with exit codes and original diagnostic evidence preserved. | Complete short output and bounded data bypass RTK; a saved raw log does not mean all its contents were seen by the agent. |
| YAGNI, minimal diffs and verification loop | Implemented as reuse, limited scope, relevant tests, required gates and correction of concrete failures. | Minimum code size and minimum token count are subordinate to correct behavior and adequate design. |
| Stable prefix and provider prompt cache | Stable instructions avoid needless changes; actual cache matching, retention and billing belong to the client/provider. | Skill invocation does not inherently destroy Claude's earlier cached prefix. No guaranteed discount or full-context per-turn cost is assumed. |
| Compact handoff and automatic compaction | Versioned checkpoints preserve goals, decisions, references, source, evidence and open work. Native host compaction is preferred. | The plugin does not control ChatGPT context thresholds. No invented universal token threshold or manual editing of opaque compaction items. |
| Deferred tool schemas and skill discovery | Use the client's native on-demand discovery where available. | Adding another MCP server is not automatically a saving; discovery is not a guarantee of invocation. |
| Compact reports and structured measurements | Implemented as receipts and concise status. | Product copy, accessibility labels, units, critical caveats and required documentation are not reduced to telegram style. |

The proposal's extra 60–80% saving, universally intact cache and identical quality
cannot be verified from these mechanisms. Output bytes, cache usage, billed
tokens and subscription quota are different measurements (Anthropic, n.d.-a,
n.d.-b; OpenAI, n.d.-b, n.d.-c).

## Review of the five follow-up suggestions

1. **Plugin path and MCP.** The absolute bundled-script fallback was already in
   the canonical skill; the Claude hook already uses its plugin-root path.
   A regression test exercises the helper installed outside the target project.
   Version 1.2 implements the actual `mcp` subcommand and its native launch files,
   rather than adding a config for an absent entrypoint. A stdio configuration is
   not a network service for ChatGPT. The terminal workflow remains available.
2. **Self-blocking.** The helper never implemented the proposed blanket cache
   blocker. It returns no permission decision and adds no substring-based allow
   exception. A command mentioning the plugin name must not bypass client checks.
3. **Copilot/OpenCode entry points.** BaryonOS has the shared `.agents/skills`
   adapter; version 1.1 adds `.github/skills/token-guard` for native Copilot
   discovery. OpenCode supports its own and compatibility skill directories.
   Version 1.2 adds explicit, non-destructive workspace MCP merges. Outside BaryonOS, install
   the complete skill directory; an adapter alone lacks its referenced source.
   Native end-to-end activation in unavailable clients remains unverified.
4. **Hosted Graphify.** Already required by the skill and project. The helper has
   no `lookup_graphify()` function and does not inspect a local `graph.json`.
   Missing required hosted context must be reported and recovered through scoped
   source reads, not silently treated as complete evidence.
5. **Implicit skill invocation.** `allow_implicit_invocation: true` was already
   present. It permits selection; it does not force activation or hook execution
   on every request. Explicit invocation and the short project instructions remain
   available (OpenAI, n.d.-a; Microsoft, n.d.; OpenCode, n.d.).

Agent Skills interoperability and client-specific plugin manifests are distinct.
The inspected folder layout does not certify every client or establish an
official combined Ponytail/ECC standard.

## Further ideas: evaluate before making them defaults

| Idea | Feasible next experiment | Why it is not claimed as implemented |
| --- | --- | --- |
| Content-addressed cross-session context cache | Reuse exact snippets only with matching file hashes, relevant source/dependency versions and task scope. | Existing hashes/stale-read checks are present; a durable cache with complete invalidation rules is not. |
| Native language-server inspection for Rust/TypeScript | Reuse an already installed language server or hosted semantic index for one demonstrably expensive navigation task. | Adding and maintaining parsers without measured need would create dependencies and more context. |
| Adaptive context expansion | Start with a relevant contract; retrieve callers, examples and implementation when an identified uncertainty remains. | The workflow already permits expansion; an automated predictor of sufficiency is not a proven quality oracle. |
| Quality-aware compression evaluation | Compare retained requirements and actual task outcomes in matched baseline/Token Guard sessions. | The evaluation protocol is packaged; an agent-level UI A/B study and human usability evaluation have not been executed. |
| Provider usage aggregation | Read usage exposed by each authorized client and aggregate input, output, cached tokens, agents, retries and time per equivalent task. | The local helper observes bytes, not inaccessible provider billing or subscription counters. |
| Automatic model routing or semantic summarization | Test one task class with unchanged acceptance criteria and measured total cost, including retries. | Both can change reasoning or remove relevant content. They remain experimental rather than default savings. |

The evaluation protocol in the packaged quality contract fixes source, task,
model/effort, tools, references and criteria; compares separate matched sessions;
and requires all mandatory criteria plus no material observed regression. The
UI pilot uses an isolated permission dialog with known hierarchy, focus, contrast
and recovery defects. It does not change approved project boards. Even a passed
pilot would only support its examined tasks, not every future execution.

## References

- Anthropic. (n.d.-a). *How Claude Code uses prompt caching*. Retrieved November 14, 2026, from https://code.claude.com/docs/en/prompt-caching
- Anthropic. (n.d.-b). *Manage costs effectively*. Retrieved November 14, 2026, from https://code.claude.com/docs/en/costs
- Microsoft. (n.d.). *Use Agent Skills in VS Code*. Retrieved November 14, 2026, from https://code.visualstudio.com/docs/agent-customization/agent-skills
- OpenAI. (n.d.-a). *Build skills*. Retrieved November 14, 2026, from https://learn.chatgpt.com/docs/build-skills
- OpenAI. (n.d.-b). *Prompt caching*. Retrieved November 14, 2026, from https://developers.openai.com/api/docs/guides/prompt-caching
- OpenAI. (n.d.-c). *Compaction*. Retrieved November 14, 2026, from https://developers.openai.com/api/docs/guides/compaction
- OpenCode. (n.d.). *Skills*. Retrieved November 14, 2026, from https://opencode.ai/docs/skills/
