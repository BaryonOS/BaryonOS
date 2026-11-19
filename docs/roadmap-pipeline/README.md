# H/U/N/B/A/P/L/R clean-session roadmap pipeline

This directory holds the execution queue and receipts of the roadmap pipeline that
advances the open criteria of tracks H, U, N, B, A, P, L and R in
[`docs/progress.json`](../progress.json). It is **not** a second roadmap. Milestone
exit criteria remain in [`docs/ROADMAP.md`](../ROADMAP.md), the fixed 100-point
register remains in `docs/progress.json`, and the current handoff remains in
[`docs/PROJECT-STATE.md`](../PROJECT-STATE.md). Every task in the ledger names
exactly one existing criterion. A verified task is an engineering checkpoint. It
does not award points: acceptance follows [`docs/PROGRESS.md`](../PROGRESS.md) and is
recorded only by the lead, with the required evidence level.

## Files

| Path | Purpose |
|---|---|
| `ledger.json` | Dependency-aware task register: stable IDs, criterion, milestone, status (`ready`, `active`, `verified`, `blocked`, `superseded`), dependencies, write and read sets, trusted check commands, risk, evidence level, non-claimable parts and rollback. Ledger order is the documented rank. |
| `receipts/<task>.json` | Compact, versioned receipt per verified task: base commit, file hashes, fresh session IDs, controller-run check exit codes and log hashes, verifier findings, and transcript checks. |
| `build/roadmap-pipeline/` (ignored) | Runtime state: controller lock, lease with heartbeat, `STOP`/`PAUSE` files, worker briefs, raw session output, full check logs. |

## Controller

### Separate issue-driven controller: default-deny execution

`tools/agent_pipeline.py` is a separate issue-driven controller. Its execution and
`tools/agent_gate.py` are disabled by the protected
`.github/agent-pipeline.json` until reviewed authorization and a verified offline
runtime exist. Labels and model-produced specifications grant no write authority.

The manifest has schema `2`, an `enabled` boolean, an exact ordered `engines` list,
a `sandbox` object and a `tasks` mapping keyed by issue number. CLI engine selection
and failover order must equal the reviewed manifest before any controller or sandbox
operation. Each task grants exact `targets` and `reads` file lists, `criterion`,
`mode`, `gates`, a full reviewed `base` SHA and `issue_sha256` from
`agent_pipeline.issue_digest`. The grant binds the issue's title, body, dependencies
and execution fields before L0. Between that base and the main commit containing its
grant, only the manifest may differ. A subsequent source change blocks the task until
its authorization is renewed. L0 may narrow these sets; lockfiles require explicit
authority too. Existing ownership and claims still apply.

The sandbox object requires pinned backend, rootfs, user resource-manager and
resource-control paths and SHA-256 values, `timeout_seconds` (1–7200), complete
`agent`/`gate` resource profiles and a complete scratch policy. It binds a
nonprivileged, maintained bubblewrap release (minimum 0.13.1), systemd user resource
scope and a dedicated credential-free offline Linux runtime, fingerprinted by
`agent_sandbox.tree_digest`. Runtime provenance,
licensing, advisories, required tools, dependencies and matching firmware must be
verified before enablement. The fingerprint establishes integrity; it does not
establish that runtime content is trustworthy. Host homes, credentials, sockets,
Git metadata, caches and devices must not be included in that runtime.

The scratch policy names one to three dedicated tmpfs roots and exact total byte and
inode caps, plus pre-copy source and export byte/file budgets. Each concurrent task
needs its own root. The controller locks the opened mount directory, binds its mount
ID and exact caps from kernel metadata, and accepts only an active owned lease as a
workspace. Source, `build/`, `target/`, `/dev/shm` and per-command temporary/home
directories stay inside that lease; the remainder of `/dev` is read-only. Missing,
shared, aliased, non-tmpfs or incorrectly capped roots fail before execution. Cleanup
failure leaves the root nonempty and therefore unavailable rather than silently
reusing it; a dirty root does not prevent selection of a later clean pool member.

Every L0/executor attempt receives a disposable source copy, explicit environment,
read-only runtime and private user, process, mount and network namespaces. Only
validated regular-file changes in its frozen write set return to the controller.
Gates mount source and evaluators read-only, with separate writable `build/` and
`target/` directories inside the same cap. The disposable tree is removed after
verified process cleanup. One combined output budget covers stdout and stderr across
the complete agent transaction or gate plan, including retries, steps and finalizers.
Captured payload bytes remain once in bounded raw files rather than being duplicated in
the controller text log; exit codes, source fingerprints and the scratch cleanup receipt
remain in the ignored controller task record.
Native gates require suitable offline tooling and disposable images. Missing
tools, network access, namespaces or required checks cause failure, never a skip
or an authenticated fallback. Unsupported active Git hooks or executable helpers
also block publication; they are not disabled. Authenticated GitHub commands run
only in the trusted controller checkout after source and head verification.

The offline fixture verifies boundary behavior, not authenticated Claude, Codex,
Copilot, Graphify or native-client compatibility. Those environments require
separate evidence and remain unverified until tested. No credential/network broker
is introduced. The roadmap controller described below retains its own ownership
model but now uses the same sandbox contract for workers, checks and verification.
Both controllers fail closed without the protected complete runtime configuration;
their task sources, authorization records and publication flows remain distinct.

**Enablement remains blocked.** The committed disabled policy must remain in place
until real dedicated mounts with measured workload headroom, the complete resource
profiles and selected client/gate transactions have been verified natively. The
public contract and mocked host tests do not provision a mount or prove ENOSPC,
inode-exhaustion, sibling-task or cleanup behavior on the target host. This is an
explicit follow-up owned by the pipeline maintainer, not an accepted availability-risk
exception. In particular, `ci.yml#host-quality` currently needs
network setup and Git metadata that this offline runtime deliberately does not
provide; it must fail rather than omit those checks. Native fixtures and local
boundary checks do not establish that workflow's compatibility.

### Roadmap controller

`tools/roadmap_runner.py` is a small deterministic controller. The long-lived part
is the controller process and the versioned ledger, not a growing chat. For each
READY task whose dependencies are verified it:

1. takes the exclusive controller lock and a write lease, rejects executable Git
   callbacks, and refuses to start when the checkout is dirty, on another branch,
   a previous worker is still alive or the protected sandbox is unavailable;
2. writes an immutable brief containing only the task fields, the binding rules and
   the trusted check commands;
3. copies reviewed regular source files into a disposable Git-free sandbox and starts
   a new headless `claude -p` process there with a fresh `--session-id`, never
   `--continue`, `--resume` or `--fork-session`, and never `--bare`, bypass modes or
   skipped permissions. The worker has a private home and network namespace without
   host Git metadata, credentials or sockets, `--permission-prompts none`, file edits
   (`acceptEdits`), read-only git, and exactly the task's check commands.
   A session-scoped `--autocompact` window is the context fail-safe (see the
   measured limit below), `--max-budget-usd` sets a per-session cap on the
   CLI's cost estimate, and the sandbox enforces the configured timeout and resource
   profile for the complete process group;
4. checks the result inside the same sandbox. Changed files must fall inside the
   task's frozen write set; anything else remains in the disposable copy and is
   discarded. Protected files (workflows, the
   pipeline, the progress register and its views, AGENTS.md/CLAUDE.md,
   PROJECT-STATE.md, DECISIONS.md) are never accepted from a worker. The ledger's
   check commands, modelled gates and `tools/progress.py --check` run with source
   read-only and private build/target/tmp paths. Only a successful regular-file delta
   is exported to the controller checkout. The worker's JSON report is recorded,
   never trusted;
5. starts a second fresh, sandboxed read-only session that verifies the candidate
   without seeing the writer's self-report. A blocker or major finding sends the task
   back to a new worker session with only those findings, for at most two repair
   rounds. Two identical failures without new evidence block the task;
6. commits verified work with its receipt, pushes the dedicated branch, releases the
   lease and selects the next task for another fresh session.

Each session receives one prompt with a fresh session ID and an empty private sandbox
home. The private home is destroyed with the command, so no host transcript or prior
conversation can enter the next session; the receipt records the prompt hash and the
controller-generated session ID instead of retaining sandbox-home transcript files.

### Controls

```sh
python3 -B tools/roadmap_runner.py status      # counts, next task, lease, stop/pause
python3 -B tools/roadmap_runner.py validate    # ledger schema and safety checks
python3 -B tools/roadmap_runner.py run --init-check --max-tasks 3
python3 -B tools/roadmap_runner.py pause       # wait before the next session
python3 -B tools/roadmap_runner.py stop        # exit before the next session
python3 -B tools/roadmap_runner.py resume      # clear stop/pause
```

A lease whose owner process is gone marks its task `blocked` with the dirty paths
recorded. The controller then stops and leaves any partial diff in place for
review; it never resets, cleans or stashes on its own.

## Known limits (measured 15 November 2026)

- **Graphify:** the hosted Graphify MCP reaches the interactive lead session through
  the host's connector delivery, and `claude mcp list` in a fresh headless process
  reports no MCP servers. Workers therefore read code directly and report that
  limitation. The lead performs the Graphify impact map (recall, definitions,
  callers, impact, linked tests) before a task enters the ledger and records the
  resulting read set and notes in the task. This matches the AGENTS.md rule that
  pipeline scripts do not call Graphify themselves.
- **Context window:** a fresh worker starts at about 28k tokens. With the requested
  100k window, the first real task (A02-T1, session `bc7d4dd4`) compacted 18 times in
  34 minutes, re-read the same files 87 times and made no edit; the lead stopped it
  through the STOP file. Workers therefore use a session-scoped 300k window, still
  as a fail-safe (one task per session remains the normal boundary), plus the
  sandbox timeout and resource profile. Host-global and user settings are not mounted.
- **Reasoning effort and quota:** security and high-risk tasks and their verifiers run at
  `xhigh`; low- and medium-risk tasks run at `high`. The first runs reached the account's
  5-hour usage limit within about an hour (inventory plus workers); the controller stops on
  that message without partial edits, and the lead resumes it after the reset.
- **Skills:** fresh sessions list `ponytail`, `verification-loop` and `token-guard`;
  the repository skill `.agents/skills/os-dev/SKILL.md` is read as a file.
- **Cost telemetry:** with subscription authentication, `total_cost_usd` is the CLI's
  estimate, not billing. The cumulative cap uses that estimate, and a rate or usage
  limit message stops the controller.
- **Lifecycle:** in a cloud container the controller is a background process of the
  container. It survives closing the interactive chat only as long as the container
  runs. The ledger and receipts are versioned, so a new container resumes with one
  command after `git pull`. Boot-time services are not installed.
- **Evidence:** this container has QEMU 8.2 with TCG only (no KVM), no physical
  devices and no human participants. Physical-hardware, user-study and
  external-audit clauses stay open regardless of pipeline results.
