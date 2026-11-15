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

`tools/roadmap_runner.py` is a small deterministic controller. The long-lived part
is the controller process and the versioned ledger, not a growing chat. For each
READY task whose dependencies are verified it:

1. takes the exclusive controller lock and a write lease, and refuses to start when
   the checkout is dirty, on another branch, or a previous worker is still alive;
2. writes an immutable brief containing only the task fields, the binding rules and
   the trusted check commands;
3. starts a new headless `claude -p` process with a fresh `--session-id`, never
   `--continue`, `--resume` or `--fork-session`, and never `--bare`, bypass modes or
   skipped permissions. The worker has an allow-listed environment without GitHub,
   cloud or session-ingress credentials, `--permission-prompts none`, file edits
   (`acceptEdits`), read-only git, and exactly the task's check commands.
   `--autocompact 100000` is the context fail-safe, and `--max-budget-usd` sets a
   per-session cap on the CLI's cost estimate;
4. checks the result itself. Changed files must fall inside the task's write set;
   anything else is reverted and reported. Protected files (workflows, the
   pipeline, the progress register and its views, AGENTS.md/CLAUDE.md,
   PROJECT-STATE.md, DECISIONS.md) are never accepted from a worker. The ledger's
   check commands and `tools/progress.py --check` are re-run with raw exit codes.
   The worker's JSON report is recorded, never trusted;
5. starts a second fresh, read-only session that verifies the diff against the task
   without seeing the writer's self-report. A blocker or major finding sends the task
   back to a new worker session with only those findings, for at most two repair
   rounds. Two identical failures without new evidence block the task;
6. commits verified work with its receipt, pushes the dedicated branch, releases the
   lease and selects the next task for another fresh session.

Each session's transcript is checked so that it contains exactly one root user
message, equal to the brief, and no entries from another session. This is the
evidence that no earlier conversation was inherited.

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
