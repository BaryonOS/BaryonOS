# S09 — Integration request: volume gate stage-count pins (15 → 22)

**Addressed to:** the Integration Manager. **From:** the S09 workstream, PR
#396 (branch `claude/s09-independent-recovery`, head `b6bb2cab`).
**Status:** open. The S09 workstream does not change protected
`.github/workflows/` files (project-owner instruction, 19 November 2026).

## Problem

PR #396 adds seven native stages to the volume gate, under the approved
`ADR-S09-DUALDISK-20261119`. Two protected workflow files still pin the old
count of 15. The contract test that binds them to the gate therefore fails,
and `host-quality` stops at that step.

## The two pins

| File | Line | Current | Required |
|---|---|---|---|
| `.github/workflows/kvm-pilot.yml` | 124 | `"volume": ("run/volume-*/result.json", 15, "volume-gate.json"),` | `"volume": ("run/volume-*/result.json", 22, "volume-gate.json"),` |
| `.github/workflows/volume-acceptance.yml` | 62 | `- name: M6e/S06 native volume gate (15 QEMU runs)` | `- name: M6e/S06 native volume gate (22 QEMU runs)` |

Line numbers refer to the files at PR head `b6bb2cab`; neither file differs
from `main` `8f970866`.

## Source of the expected count

`PLAN` in `tools/test_volume.py` (line 49) is the single source. It has 22
entries: the 15 existing stages, unchanged, plus `archive-disk-export`,
`archive-disk-absent`, `archive-disk-small`, `archive-disk-restore`,
`archive-disk-wrong`, `archive-disk-cut` and `archive-disk-verify`
(commit `13674aac`). `tests/test_ci_workflows.py`
(`test_volume_gate_stage_counts_match_plan`, line 400) parses `PLAN` with
`ast` and requires both pins to equal `len(PLAN)`. The test needs no change.

## Failing test and actual output

Hosted CI, workflow `ci.yml`, job `host-quality`, step "Python syntax and
DocumentSession host tests", run
[44495342613](https://github.com/BaryonOS/BaryonOS/actions/runs/44495342613)
at `b6bb2cab`:

```text
FAIL: test_volume_gate_stage_counts_match_plan (test_ci_workflows.WorkflowContractTests.test_volume_gate_stage_counts_match_plan)
AssertionError: Lists differ: ['15'] != ['22']
FAILED (failures=1)
```

It is the only failing test. Steps 12–24 of `host-quality` are skipped after
it, and the three "Preserve … coverage artifact" steps fail only because those
steps produced no coverage. The same result occurred at heads `b48e8bc3`,
`ec397afd` and `dcc5a783`. It reproduces locally with
`python3 -B -m unittest discover -s tests -p test_ci_workflows.py` (exit 1, the
same single failure).

The gate itself already runs 22 stages on hosted CI: `volume-acceptance.yml`
job `m6e-native-volume`, run
[44495342426](https://github.com/BaryonOS/BaryonOS/actions/runs/44495342426)
at `b6bb2cab`, passed with `M6e gate passed=True stages=22` in 6 min 21 s,
inside the job's 40-minute timeout. Only the label and the KVM pilot's
expected report count are stale.

## Proposed correction

Change only the two values above, `15` → `22`, in one commit, either on
PR #396 or in a separate PR that lands with or after it. Do not change
`tests/test_ci_workflows.py`, the `host-quality` job or any other workflow.
If the pins land before PR #396, the test fails on `main` with `['22'] != ['15']`;
land them together with PR #396 or after it.

## Verification

```bash
python3 -B -m unittest discover -s tests -p 'test_ci_workflows.py'
```

```bash
actionlint -shellcheck= -pyflakes= .github/workflows/kvm-pilot.yml .github/workflows/volume-acceptance.yml
```

Expected: exit 0 for both, then a green hosted `host-quality` on the new head.

Trial run by the S09 workstream: the two edits were applied in a throwaway
detached worktree at `b6bb2cab`, outside this branch, and the worktree was
removed afterwards. The `test_ci_workflows.py` command above then exited 0
(all tests OK). `actionlint` is not installed on that host, so it was **not
run**.
The KVM pilot is `workflow_dispatch`-only. Its count is checked when it is
next dispatched with `gate: volume`; it then expects 22 `run/volume-*/result.json` files.

## Rollback

Revert the pin commit. If PR #396 is also reverted, `PLAN` returns to 15 and
the reverted pins match again. Reverting only one of them recreates this
mismatch in the other direction.
