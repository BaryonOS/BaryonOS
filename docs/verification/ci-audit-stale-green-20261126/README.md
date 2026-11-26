# Independent CI audit evidence — stale-green gate reruns

Date: 26 November 2026. Baseline:
`04fd62453a1720fb295bb21e3e3eb8ca27955790` (`origin/main`, merged PR
#550). Values are derived from GitHub Actions REST data. Runner minutes are
sums of job wall-clock durations, not billing figures.

## Reproduce the committed evidence

From the repository root, run:

```sh
python3 -B docs/verification/ci-audit-stale-green-20261126/verify.py
```

The standard-library verifier checks the committed snapshot hashes and
recomputes the queue, targeted job-median, 60-PR baseline and stale-green
claims. It also checks that the capacity and branch-filter analyses describe
the shared 100-run capture consistently. The PR #550, same-tree duplication
and branch-filter totals still depend on the external raw responses whose
paths and SHA-256 digests are recorded in `sources.json`; a clean checkout
cannot independently recompute those three values.

## Reproduced correctness gap

PR #515 contained head
`3ce662a4e307c22e8f087bd1f7b1df44c2ce8b36`. Its merge-hold
[run 45644771961](https://github.com/BaryonOS/BaryonOS/actions/runs/45644771961)
completed successfully at 16:18:47Z. The same head's CI
[run 45644771873](https://github.com/BaryonOS/BaryonOS/actions/runs/45644771873)
then ran attempt 2 from 16:34:19Z to 16:34:43Z. A head-filtered API listing
contained one merge-hold run and no hold newer than that CI attempt. The later
attempt passed, so this observation is not a missed failure; it proves that the
required green decision can remain bound to an older gate attempt.

Reproduction:

```sh
gh api --method GET repos/BaryonOS/BaryonOS/actions/runs \
  -f head_sha=3ce662a4e307c22e8f087bd1f7b1df44c2ce8b36 \
  -f event=pull_request -f per_page=100
gh api repos/BaryonOS/BaryonOS/actions/runs/45644771961
gh api repos/BaryonOS/BaryonOS/actions/runs/45644771873
```

The repair listens to `workflow_run.in_progress` for attempts greater than one
because GitHub does not emit `requested` for re-runs. It re-runs older passing
(`success`, `neutral` or `skipped`) evidence for a catalogue gate, using the
whole-workflow `/rerun` endpoint. An older active hold is cancelled and polled
by exact run identity; regular cancellation falls back to force-cancel, while a
passing completion race is re-run. A higher attempt is evidence only when it
started strictly after the gate boundary. The `completed` activity repeats
stale-passing invalidation even while another gate is open; only retrying an
already red hold waits for all gates to close. Initial attempts, non-gates,
forks, moved heads and newer holds are negative cases in
`tools/tests/test_merge_hold_recheck.py`.

The distinct start and completion handlers can race the same re-run request. A
409 or an explicit "already running" response is not accepted by itself: up to
six exact-run reads must prove a higher `run_attempt` that started strictly
after the applicable gate boundary. An attempt that started on or before that
boundary is followed to completion and then re-run once. Missing timestamps,
unknown states and a second unproven conflict fail the re-check and preserve the
stale result for investigation.

The pull request head is re-read before every cancel, force-cancel and re-run
POST. A 409 is never sufficient evidence; exact state must converge to a newer
post-boundary attempt or a terminal conclusion. Both cancel stages are bounded
under the five-minute job timeout, and an unconfirmed state is a hard error.

This is asynchronous. A small interval remains between a gate re-run starting
and GitHub scheduling the invalidating re-check. Host tests prove decision
logic, not GitHub scheduler latency; the first hosted gate re-run after merge is
the live acceptance test.

## Timing and capacity baseline

The existing 60-PR reference measured 11.1 minutes median to required-green,
6.42 CI rounds and 910.1 job wall-clock minutes per PR. It predates PR #550 and
is not treated as a post-change measurement.

For PR #550, 73 of 83 Actions checks executed and summed to 215.0 job
wall-clock minutes. The native-window workflow took 40.7 minutes end to end.
Median execution among the current source-bound samples was 535.5 seconds for
host-quality, 130 seconds for qemu-smoke, 635 seconds for CodeQL Rust, 949
seconds for native SMP, 646 seconds for native system and 1,238 seconds for the
native-window scale-2 job. Artifact upload normally took 1–4 seconds; the
acceptance scenarios, not transfer, dominate those critical paths.

During the 07:15–07:45Z burst, the observed peak was 20 running jobs. The
per-minute running count had median 18 and p90 19. The 101
jobs outside known matrix caps waited 183 seconds median and 544 seconds at
p90; 26 matrix-limited jobs waited 812 seconds median. This is an observed
repository peak, not proof of the organization limit. In particular, the
documented Enterprise value 500 is not treated as measured available capacity.

PR #515 head `33b1d7be3613dfca41529d93614183e283381989` and squash-main
commit `744f1b75b68dc45dd2ab4d150a65cc827402e381` share tree
`2face506e98d17231ee65b4089a1086c8748e266`, but their PR and main runs
consumed 260.3 and 206.8 job wall-clock minutes respectively. This is measured
duplication, not permission to reuse acceptance evidence across commit SHAs.
Avoiding the second full pass requires proven merge-group acceptance and the
project owner's separate D1 activation decision.

## Branch-filter counterfactual

In the 07:15–07:55Z capture, 47 upstream completions mapped uniquely to
47 re-check runs: 21 main-push activities produced skipped jobs, while 26 PR
activities remained (16 cancelled in the queue and 10 successful). Excluding
the exact `main` branch would therefore remove 21 workflow objects and 21
skipped job records, but no runner allocation, runner minute or script API
call. The added `in_progress` correctness trigger creates administrative runs
for started PR workflows, so the combined patch must not be reported as a
runner-time reduction.

The machine-readable values and explicit `NOT RUN` limits are in
`metrics.json`. The bounded capture manifest in `sources.json` records the REST
queries, retained raw-file sets, extraction scripts and SHA-256 digests behind
those values. The capacity analysis filters the shared 100-run response to 99
runs created through 07:45Z; the branch-filter analysis also uses the one later
07:50Z source/re-check pair. A current replay of the 07:15–07:55Z query returned
the same 100 run IDs. Large raw API responses remain in the named external
scratch location and are not repository evidence by themselves. Live merge-group validation was not possible because the queue
is inactive. Enterprise capacity settings were not visible with the available
token. No native OS acceptance follows from this CI-only evidence.
