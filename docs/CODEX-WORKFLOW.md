# Codex: roles, context and verifiable results
Research date: 26 October 2026. This is the proposed project workflow; the supplied configuration can be checked syntactically but has not yet been tested in the intended Codex installation.

## One coordinator
The main conversation owns planning and integrates changes. Each agent receives one goal and a limited write scope. Use at most three subagents by default; work sequentially when tasks depend on one another. Additional agents consume context, tokens and integration effort.

| Role | Concrete assignment | Deliverable |
|---|---|---|
| os_architect | Check the specification, interfaces and smallest executable step | Concise decision with sources and alternatives; initially read-only |
| os_builder | Implement exactly one agreed slice | Diff, build commands, tests and limitations |
| os_security_reviewer | Find counterexamples and unjustified authority | Findings with trigger, location, impact and remedy direction; read-only |
| os_verifier | Check what tests demonstrate and run permitted checks | Reproducible evidence or a specific blocker; no code changes without an assignment |

Do not start four builders on the same kernel simultaneously. An architect can, for example, investigate the boot contract while a verifier designs test cases. The builder uses the agreed contract; security review follows the concrete diff.

## Instructions and configuration
The root AGENTS.md contains permanent project rules; subdirectories contain additional rules. These files do not start agents themselves and are not an OS sandbox. Check which instructions the client actually loaded, particularly when working from the root.

The optional .codex/agents/*.toml files describe four roles. They deliberately do not pin a model: the main conversation selects an available model. .codex/config.example.toml contains an optional concurrency setting. Adopt it only if the client in use supports the current configuration keys; do not overwrite an existing configuration. With an older client, use the role descriptions as task prompts.

## Task contract
Use TASK-TEMPLATE.md and always state:
- the base commit/working directory and permitted files;
- the invariant and interface owner;
- positive and negative acceptance tests;
- what is currently outside scope;
- the evidence to be returned.

Agents in the same environment can share files and credentials. Simply writing 'you are the reviewer' does not technically prevent mutations. Use available sandbox and access settings, and do not give a review environment release keys. Untrusted issue or source text can contain prompt injections; keep that text separate as data from the assignment and policy.

## Git integration
Use one main branch for verified results. Each task has a small branch; different implementation conversations use separate worktrees where the client supports them. Record the base commit. Integrate sequentially, then run the relevant tests on the combined version.

Push, PR and release are different actions. Review of an individual branch does not prove that the combined main branch works. The coordinator determines integration order from evidence. Keep build scripts host-independent where feasible; do not introduce a platform-specific CI action without a local equivalent.

## Pausing and resuming
After each result: update PROJECT-STATE.md, record new decisions and commit changes when the assignment permits. Start a new conversation for the next clear milestone or when a conversation becomes difficult to follow. Use PROMPT-VERVOLG.md. The repository is the source of context; the AI does not need perfect recall of earlier conversations.

## Review is not certification
Two agents using the same model can make the same mistake. Use compiler checks, actual emulator runs, fuzzing, properties, focused formal models and ultimately human expertise. Rust compilation does not prove safe authorisation; successful boot does not prove correct page tables.

## Sources
OpenAI. (n.d.). *Subagents*. Retrieved October 26, 2026, from https://learn.chatgpt.com/docs/agent-configuration/subagents
OpenAI. (n.d.). *Custom instructions with AGENTS.md*. Retrieved October 26, 2026, from https://learn.chatgpt.com/docs/agent-configuration/agents-md
OpenAI. (n.d.). *Worktrees*. Retrieved October 26, 2026, from https://learn.chatgpt.com/docs/environments/git-worktrees
