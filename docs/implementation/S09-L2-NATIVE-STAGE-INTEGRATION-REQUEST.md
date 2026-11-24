# S09 — Integration request: native L2 export and restore stages and volume gate pins (26 → 30, planned)

**To:** the integration coordinator.
**From:** the S09 workstream, branch `claude/s09-l2-native-export` (stacked on
#411, `f6a07fe3`).
**Status:** planned. The stages are designed in
[S09-L2-NATIVE-EXPORT.md](S09-L2-NATIVE-EXPORT.md) and not yet implemented.
The pins below are requested only for the head that adds the stages, and only
once the full local gate reports `passed=True` with 30 stages. No action is
needed before that head exists. The S09 workstream does not change protected
`.github/workflows/` files (project-owner instruction of 19 November 2026).

## Problem

`tests/test_ci_workflows.py` (`test_volume_gate_stage_counts_match_plan`)
requires two protected pins to equal `len(PLAN)` in `tools/test_volume.py`.
Four new `PLAN` entries (`l2-export`, `l2-restore`, `l2-corrupt`, `l2-used`)
take `len(PLAN)` from 26 to 30.

## Stages

| `PLAN` entry | Parent | Kernel feature | Case marker | Proves |
|---|---|---|---|---|
| `l2-export` | `archive-export` | `test-volume-l2-export` | `L2_EXPORT` | Writable source step, then v2 export to the device-1 disk, read back and verified |
| `l2-restore` | `l2-export` | `test-volume-l2-restore` | `L2_RESTORE` | Cold L2 restore without the source disk; provenance matches the source |
| `l2-corrupt` | `l2-export` | `test-volume-l2-corrupt` | `L2_CORRUPT` | A damaged archive is refused with `Corrupt` before any write |
| `l2-used` | `l2-restore` | `test-volume-l2-used` | `L2_USED` | Provenance survives a cold boot; a restore into a used domain returns `Stale` |

These stages add no new authority.

- **What they use:** the slot-1 backup portal and the test-only device-1 disk
  that the project owner approved on 19 November 2026 (bounds B1–B6 in
  [S09-DUAL-DISK-FIXTURE-REQUEST.md](S09-DUAL-DISK-FIXTURE-REQUEST.md)).
- **Rights:** the device-1 grant is R|W|F for `l2-export` and READ for the
  other three.
- **Unchanged:** no portal grant, capability semantics or device ownership
  beyond those bounds.

## The two protected pins

| File | Line at `f6a07fe3` | Current | Requested |
|---|---|---|---|
| `.github/workflows/kvm-pilot.yml` | 124 | `"volume": ("run/volume-*/result.json", 26, "volume-gate.json"),` | `"volume": ("run/volume-*/result.json", 30, "volume-gate.json"),` |
| `.github/workflows/volume-acceptance.yml` | 62 | `- name: M6e/S06 native volume gate (26 QEMU runs)` | `- name: M6e/S06 native volume gate (30 QEMU runs)` |

If the stage review changes the number of stages, the pins must equal
`len(PLAN)` at that head; this table does not bind them.

## Ordering and expected failure

- **When the pins land.** Land them together with the stage commit, or after
  it. Landed earlier, they fail the same test on the base the other way round
  (`['30'] != ['26']`).
- **Expected failure in between.** Until the pins land, hosted `host-quality`
  fails on that single test with `['26'] != ['30']`, as in the 15 → 22 and
  22 → 26 precedents.
- **Base PRs.** #407, #408 and #411 must be merged first, because the stages
  call `export_v2` (#408) and `restore_native_l2` (#411).
- **U05 (#436).** The U05 cases 23 and 24 run through `tools/run_settings.py`
  and do not change `PLAN`.

## Verification after the change

```bash
python3 -B -m unittest discover -s tests -p 'test_ci_workflows.py'
```

```bash
actionlint -shellcheck= -pyflakes= .github/workflows/kvm-pilot.yml .github/workflows/volume-acceptance.yml
```

```bash
python3 -B tools/test_volume.py
```

Expected results:

- the first two commands exit 0;
- the gate reports `passed=True` with 30 stages;
- hosted `host-quality` and `m6e-native-volume` are green on that head.

## Rollback

Revert the pin commit and the stage commit together. Reverting only one
recreates the count mismatch.
