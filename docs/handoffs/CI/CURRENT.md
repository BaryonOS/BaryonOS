# Specialist CI — resume pointer

This is the compact resume pointer for the CI specialist ("BaryonOS - Specialist CI").
It is not a backlog. Tasks come from issue #494, open pull requests and
`docs/progress.json`. Newer ADRs and live GitHub state take precedence over
this file.

## Scope, mandate and working agreements (recorded once)

- **Scope:** CI workflows (`.github/**`), the merge-hold and CI-impact tooling
  (`tools/merge_hold.py`, `tools/ci_impact.py`), the gates' test pins in
  `tests/test_ci_workflows.py`, and CI cost and flake reduction.
  - Subsystem behaviour belongs to the owning specialist.
  - The CI specialist is not the PR guard. It feeds new heads to "BaryonOS - PR Bewaker".
- **Mandate:** the standing CI maintenance mandate is issue #494, under
  ADR-MERGE-DELEGATION-20261123 and ADR-PROGRESS-ASSESSMENTS-20261125 (D1–D3).
  - Under D2, for `.github/**`, `tools/merge_hold.py` and `tools/ci_impact.py`,
    an independent AI review is enough. It must come from a separate agent
    without write access, after the bot findings are resolved.
  - Policy, register and security files need the project owner's review on the pull request.
- **Never:** bypass protections, weaken gates or tests, change a ruleset or
  enable the merge queue. Enabling the queue needs the owner's explicit
  confirmation at that moment (D1).
- **Merge on applicable checks:** required checks plus the gates that apply to
  the change. Batch pushes. Do not wait for native suites the change does not
  affect.
- **Context rotation:** follow ADR-SPECIALIST-TEAM-20261125. The project owner
  runs `/clear`; this session never does.

## Durable locations

| Content | Location |
|---|---|
| Live checkpoint: open pull requests, heads, runs, next task | the newest "HANDOFF — Specialist CI" comment on #494 |
| Owner decisions D1–D3 | #494, comments 7155783661 and 7155944783; ADR-PROGRESS-ASSESSMENTS-20261125 |
| Pull request head, tested SHA, hosted runs | the pull request description |

## Lessons

1. Run the `roadmap_runner` tests with `env -u GIT_EDITOR`. Otherwise the
   session's GIT_EDITOR causes false "Git environment overrides" failures.
2. merge-hold waits for running gates with a cheap poll (#517). Without a
   merge queue, a long-gated pull request needs one catch-up round for each
   landing on `main`. This is the evidence for D1.

## Resume procedure

1. Read this file and the newest #494 handoff comment.
2. Fetch `origin/main`. Check the CI branches, worktrees, open pull requests,
   heads, reviews and gates, and any running processes.
3. Report the session to the PR guard. Resume the next task in the
   checkpoint, and do not repeat work that is already proven.
