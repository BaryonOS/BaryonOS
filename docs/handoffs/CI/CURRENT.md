# Specialist CI — resume pointer

This is the compact resume pointer for the CI specialist ("BaryonOS - Specialist CI").
It is not a backlog. Tasks come from issue #494, open pull requests and
`docs/progress.json`. Newer ADRs and live GitHub state take precedence over
this file. The role contract is `.agents/roles/specialist_ci.md`; the shared
handoff rules and per-PR integrity duties are in `docs/handoffs/README.md`.

## Scope, mandate and working agreements (recorded once)

- **Scope:** CI workflows (`.github/**`), the merge-hold and CI-impact tooling
  (`tools/merge_hold.py`, `tools/ci_impact.py`), the gates' test pins in
  `tests/test_ci_workflows.py`, the agent adapter generator
  (`tools/agent_adapters.py`, tasks CI-C1 to CI-C3), and CI cost and flake reduction.
  - Subsystem behaviour belongs to the owning specialist.
  - The CI specialist is not the PR guard. It feeds new heads to "BaryonOS - PR Bewaker".
- **Mandate:** the standing CI maintenance mandate is issue #494, under
  ADR-MERGE-DELEGATION-20261123 and ADR-PROGRESS-ASSESSMENTS-20261125 (D1–D3).
  - CI squash-merges its regular CI pull requests itself (owner, 25 November
    2026 16:55 UTC) and enqueues its own merge-ready pull requests without a
    lane grant (ADR-MERGE-QUEUE-AUTONOMY-20261126). CI pull requests go first
    (owner, 26 November 2026).
  - Under D2, for `.github/**`, `tools/merge_hold.py` and `tools/ci_impact.py`,
    an independent AI review is enough. It must come from a separate agent
    without write access, after the bot findings are resolved.
  - Policy, register and security files need the project owner's review on the pull request.
  - Proportional review (owner, 25 November 2026, every CI acceleration batch):
    a valid independent D2 GO leads to integration once the applicable
    conditions are met, not to another general review round. The CI specialist
    checks small textual corrections and non-blocking findings itself with a
    targeted diff check. An extra independent delta review is needed only when
    a change materially affects implementation, security, test coverage,
    permissions or acceptance conditions, or when repository policy requires
    one. Real blocking findings are always fixed; non-essential improvements
    wait for regular maintenance.
- **Phase 5 (owner, 25 November 2026):** a permanent cycle of integrated CI
  optimisation. It minimises the time from a verified change to its landing
  on `main`, with the least necessary runner use and no loss of quality,
  security, functional acceptance or reliability. The P0 performance contract
  (owner, 26 November 2026 ~21:15 UTC) sets the current targets.
  - Work areas: A, selective native tests; B, less duplicate work;
    C, the critical path, with a KVM pilot alongside Specialist R; D, the merge flow.
  - The KVM pilot does not replace TCG acceptance without evidence.
  - Selection stays fail-closed. An unknown or incomplete dependency means a
    full run. A central change runs every applicable gate. A PR never shrinks
    its own required set, and a skipped gate is never acceptance evidence.
  - Every gain is measured, with its sample size and uncertainty. No timeout,
    sleep, test duration or criterion is shortened.
- **Cost:** the $0 Actions budget hard stop stays. Heavy runners use only the
  included compute credit and must fall back to standard runners, never block
  the queue.
- **Never:** bypass protections, weaken gates or tests, or change a ruleset,
  organisation setting or runner group; use `--admin`, auto-merge or a force
  push; run `gh pr merge` on a BLOCKED pull request (it silently enables
  auto-merge). Owner-reserved changes need the owner's explicit yes at that moment.
- **Merge on applicable checks:** required checks plus the gates that apply to
  the change. Batch pushes. Do not wait for native suites the change does not
  affect. Update a branch by merging `main` in; never rebase a pushed branch.
- **Enqueue pre-check:** no queued or in-progress run on the head except
  merge-hold; `mergeStateStatus` CLEAN, HAS_HOOKS or BEHIND; `autoMergeRequest`
  null; zero unresolved review threads. Then GraphQL `enqueuePullRequest` with
  `expectedHeadOid`.
- **Pull request metadata:** label `maintenance`, Project #5 fields CI/CD and
  Maintenance, `Refs #494`, no milestone (CI exemption), the template's
  verification table, and a D3 assessment file per pull request (the per-PR
  integrity duties in `docs/handoffs/README.md`). A missing assessment is
  repaired in the next own pull request, never as a merge block.
- **Context rotation:** follow ADR-SPECIALIST-TEAM-20261125. The project owner
  runs `/clear`; this session never does.

## Durable locations

| Content | Location |
|---|---|
| Live checkpoint: open pull requests, heads, runs, next task | the newest comment starting with `HANDOFF CI` on #494, from an author with write access |
| Owner decisions D1–D3 | #494, comments 7155783661 and 7155944783; ADR-PROGRESS-ASSESSMENTS-20261125 |
| Phase 5 mandate: objective, work areas, measurement, quality contract | #494, comment 7161891267 |
| Pull request head, tested SHA, hosted runs | the pull request description |

## Lessons

1. Run the `roadmap_runner` tests with `env -u GIT_EDITOR`. Otherwise the
   session's GIT_EDITOR causes false "Git environment overrides" failures.
2. merge-hold waits for running gates with a cheap poll (#517). The merge
   queue (D1) removed the catch-up round per landing on `main`.
3. `.github/ci-impact.candidate.toml` (C3) carries its own line citations;
   re-anchor it together with the catalogue after every test shift.
4. After B9, a merge-hold failure with only "Pending:" lines is expected while
   gates run. Do not push a fix; the re-check re-runs it. If it is still
   failed once every gate has finished (the re-check can miss a merge-hold
   that was still running), re-run it by hand: `gh run rerun <run id> --failed`.
5. While its header says `# Candidate increment: none`,
   `.github/ci-impact.candidate.toml` is its own header plus a verbatim copy of
   the catalogue, and a host-quality test enforces that. Every catalogue change,
   including a revision bump, is copied into it in the same pull request. An
   intentional increment replaces `none` with its name and #494 reference.
6. The catalogue `revision` is the revision on `main` plus one. Re-read it
   immediately before every push, because another landing may have taken it.
7. After a change to a specialist contract's heading, first intro sentence or
   `Tool class:` line, regenerate the adapters with
   `python3 -B tools/agent_adapters.py --write`. The drift test in
   `tools/tests/test_agent_governance.py` fails on stale or orphaned adapters.

## Resume procedure

1. Read this file and the newest `HANDOFF CI` comment on #494.
2. Fetch `origin/main`. Check the CI branches, worktrees, open pull requests,
   heads, reviews and gates, the merge queue, and any running processes.
3. Use hosted Graphify `recall` for the paths involved when it is available.
   Without it, read the code, the ADRs and #494 directly and state that limitation.
4. Report the session to the PR guard. Resume the next task in the
   checkpoint, and do not repeat work that is already proven.
