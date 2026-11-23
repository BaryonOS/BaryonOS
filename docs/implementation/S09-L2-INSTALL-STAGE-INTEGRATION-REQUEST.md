# S09 — Integration request: native INSTALL stages and volume gate pins (22 → 26, planned)

**Addressed to:** the Integration Manager. **From:** the S09 workstream, PR
#411 (branch `claude/s09-l2-provenance`, head `83443b21`, stacked on #408).
**Status:** open. The four stages are implemented on this branch (the
commit after `cc5fd5ca`). The full gate passed locally with 26 stages
(`tools/test_volume.py`, `pc-q35-8.2`, KVM, `passed=True`, host checks exit
0; `build/install-gate/volume-gate.json`). As expected,
`test_volume_gate_stage_counts_match_plan` now fails with
`AssertionError: Lists differ: ['22'] != ['26']` (observed locally, exit 1)
until the two protected pins below are changed. The S09 workstream does not
change protected `.github/workflows/` files (project-owner instruction,
19 November 2026).

## Problem

`83443b21` links the S09 L2 operations 19–23 (INSTALL_BEGIN, INSTALL_APPEND,
INSTALL_COMMIT, INSTALL_COLLECTION, PROVENANCE_STAT; see
`S09-L2-PROVENANCE.md`) into the native volume service. The 22-stage volume
gate passes with that binary, but no stage issues ops 19–23, so L2 has host
evidence only (unit, crash and mutation tests).

Native evidence needs new stages in `PLAN` (`tools/test_volume.py`). Every new
`PLAN` entry breaks two protected pins: `tests/test_ci_workflows.py`
(`test_volume_gate_stage_counts_match_plan`, line 400) requires both to equal
`len(PLAN)`.

## Planned stages

The four entries are appended to `PLAN` after the existing 22.

- **Domain.** Every stage runs in the slot-1 app's own domain 0, through that
  app's existing ingress endpoint.
- **No new authority.** No portal grant, archive range, second disk or
  capability changes.
- **By design.** INSTALL reaches only the caller's own domain and only slots
  that were never used (spec section 2).

| `PLAN` entry | Parent | Kernel feature | `CASE_MARKER` / cut marker | What the stage proves | Host oracle |
|---|---|---|---|---|---|
| `install` | none (fresh image, writable) | `test-volume-install` | `INSTALL` | INSTALL of one object with 3 fixture versions, and `INSTALL_COLLECTION`, into slots never used before.<br>Replies: STAT gives revision 3, VERSION_STAT gives 3..1, PROVENANCE_STAT returns the fixture's origin revisions and timestamps.<br>A second INSTALL_BEGIN into the now-used slot returns `Stale` with no table write. | `volume_disk.choose(..., allow_v3=True)` finds a v3 table with `installed = 3`.<br>Revisions and `origin_*` equal the fixture, and the clock is +1 (rule R).<br>Version contents are byte-exact against the fixture. |
| `install-recover` | `install` (read-only cold mount, same image) | `test-volume-install-recover` | `INSTALL_RECOVER` | Provenance survives a cold boot: STAT, VERSION_STAT and PROVENANCE_STAT give the same answers.<br>INSTALL_BEGIN on the read-only mount returns `Access`. | Image SHA-256 unchanged from `install` |
| `install-cut` | none (fresh image, writable) | `test-volume-install-cut` | `VOLUME:CUT:INSTALL` (real kill, raw `-9`) | Power cut during the INSTALL_COMMIT table write. | None at this stage; the next stage checks it |
| `recover-cut` | `install-cut` | `test-volume-recover-cut` (existing) | `RECOVER_CUT` (existing) | The object is either never-used (revision 0, no provenance) or fully installed; never partial. | `allow_v3=True` finds exactly one of the two states, from a valid table copy. |

Expected `len(PLAN)` at the head that adds the stages: **26**. If the stage
review changes the list, the pins must equal `len(PLAN)` at that head; this
table does not bind them.

Not in this request: native stages for archive v2 export and L1/L2 restore
(phase 3). They are not yet designed in this detail and need a separate
request.

## Write set requested for the S09 workstream (non-protected)

| File | Change |
|---|---|
| `tools/test_volume.py` | `PLAN`; the "twenty-two" in the module docstring |
| `tools/run_volume.py` | `FEATURE`, `CASE_MARKER`, `CUT_MARKER`, the allowed `(case, parent)` pairs, and the v3 oracle for the new cases. The existing `allow_v3` parameter of `tools/volume_disk.py` is used; that file needs no change. |
| `tools/build.py` | The case list that maps gate cases to kernel features |
| `kernel/Cargo.toml` | Three test-only features, each `["native-volume"]` |
| `kernel/src/m2/m3/volume.rs` | Test-profile code only: `case()` numbers, markers, expected counters and the INSTALL cut point. No change to the kernel ABI, capability semantics, grants or device ownership. |
| `sdk/examples/volume-docs/src/app.rs` | The three app flows, using the existing `Client::install` and `Client::provenance` |
| `tests/test_volume_*.py` | Host tests for the new runner logic |

The kernel file is listed explicitly because the project rules require an
agreed write area for kernel changes. A peer session currently holds
`services/volume`, `tools/volume_disk.py` and `tools/mutate_volume.py`. None of
them is in this write set; the S09 workstream coordinates with that session
before any change to them becomes necessary.

## The two protected pins

Both files are identical on PR head `83443b21` and `main` `035b44e5`.

| File | Line | Current | Requested |
|---|---|---|---|
| `.github/workflows/kvm-pilot.yml` | 124 | `"volume": ("run/volume-*/result.json", 22, "volume-gate.json"),` | `"volume": ("run/volume-*/result.json", 26, "volume-gate.json"),` |
| `.github/workflows/volume-acceptance.yml` | 62 | `- name: M6e/S06 native volume gate (22 QEMU runs)` | `- name: M6e/S06 native volume gate (26 QEMU runs)` |

## Expected hosted failure

If the stage commit lands without the pins, `host-quality` should fail on the
same single test as in the 15 → 22 precedent
(`S09-WORKFLOW-PIN-INTEGRATION-REQUEST.md`), with
`AssertionError: Lists differ: ['22'] != ['26']`. It has been reproduced locally; no hosted run of this head exists yet.

## Ordering

Land the pins together with the stage commit, or after it. Landed earlier,
they fail the same test on `main` the other way round (`['26'] != ['22']`).
Do not change `tests/test_ci_workflows.py`.

## Runtime budget

The 22-stage gate has passed twice:

- on hosted CI, in 6 min 21 s (run 44495342426 at `b6bb2cab`);
- locally at `83443b21` (`build/optS-gate3/volume-gate.json`, `passed=True`,
  22 stages).

The four extra stages each boot once. Together they should add under
2 minutes, well inside the job's 40-minute timeout; this is an estimate, not
a measurement.

## Risks to settle during the stage work

- **App size.** The volume-docs app binary is linked against the same 64 KiB
  `.text` bound as the service. Linking the INSTALL client flows into the app
  has not been measured.
- **Cut point (fixed).** The kernel kills after 4 table-sector writes of the
  INSTALL_COMMIT publish (as `cut-txn`). That copy is torn, so `recover-cut`
  natively finds the object never-used (generation 1). The fully-installed
  branch of the oracle is covered only by host tests
  (`tests/test_volume_runner.py`).

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
- the gate reports `passed=True` with 26 stages;
- hosted `host-quality` and `m6e-native-volume` are green on the new head.

## Rollback

Revert the pin commit and the stage commit together. Reverting only one
recreates the count mismatch in the other direction.
