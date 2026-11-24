# S09 phase 3 slices C and D: native L2 stages

Evidence for slices C and D of
[S09-L2-NATIVE-EXPORT.md](../../implementation/S09-L2-NATIVE-EXPORT.md)
(section 9).

**Evidence level:** local native QEMU runs of a test-only fixture, plus host
tests. This is not hosted acceptance, not production behaviour and not a
human audit. S09 stays open.

## Source and environment

- Tested source: `61349b1fd051c6b1954cf8aac1a37c92a331ff0b` on
  `claude/s09-l2-slice-cd` (base `origin/main` `16ae48a8`). Source
  fingerprint v3: `3599458143a153fb9ff5360d25310708a7a0fb0bf6d7bae51fb57ef648906cd6`.
- QEMU 10.2.1, machine `pc-q35-10.0`, TCG, rustc 1.100.0.
- The exact pull-request head is recorded in the pull request; later commits
  touch documentation only.

## Native gate

`NOS_BUILD_DIR=build/gate2 python3 -B -u tools/test_volume.py`: exit 0,
`M6e gate passed=True stages=30`. The four new stages:

| Stage | Case | Raw QEMU exit | Result |
|---|---|---|---|
| `l2-export` | 25 | 33 | S1–S5 (11 commits), v2 export to device 1, 92 697 bytes; `omitted = 1`; 26 compared pairs, none equal |
| `l2-restore` | 26 | 33 | cold restore into an empty domain without the source disk: `restored = 0x3E`, `collections = 0b111`, `committed = 8`, 0 provenance mismatches |
| `l2-corrupt` | 27 | 33 | one content byte flipped (offset 59 849, object 5): `Corrupt`, `committed = 0`, disk 0 unchanged |
| `l2-used` | 28 | 33 | both disks read-only at device level: provenance survives the cold boot (0 mismatches); a second restore returns `Stale`, `committed = 0` |

All four print `VOLUME:ARCHIVE:RIGHTS_REFUSALS` as specified: 0 for
`l2-export` and 1 (the refused device-1 write, step 702) for the others.
The other 26 stages still pass.

### First run (failed, fixed)

The first run at `d9285ca3` passed 29 of 30 stages. `l2-used` failed with
`VOLUME:SERVICE_ERROR` = `Protocol` (-15): the portal read the archive disk
while the read-only service was still recovering, and the broker, which
serves one block request across both disks, failed the service's read. A
diagnostic run (temporary kernel prints, not committed) showed the archive
lane active during the service's recovery. `l2_used` now lists the restored
domain before its first archive read, as `l2_restore` does. Whether the
broker should answer `Busy` there is recorded as an open kernel question in
section 10 of the design document.

## Host checks

| Command | Exit |
|---|---|
| `python3 -B -m unittest discover -s tests -p 'test_volume_*.py'` | 0 (195 tests, 1 skipped) |
| `python3 -B -m unittest discover -s tests -p 'test_ci_workflows.py'` | 0 (39 tests; both stage-count pins at 30) |
| `python3 -m compileall -q tools tests` | 0 |
| `cargo +1.100.0 fmt --manifest-path sdk/Cargo.toml --all -- --check` | 0 |
| `python3 tools/progress.py --check` | 0 |

`actionlint` is not installed on this host: NOT RUN. The workflow change
replaces two numbers only.

**Limit of the host tests:** `tests/test_volume_l2_native.py` builds its
synthetic after-images from the oracle's own model (`expected_source`,
`expected_l2`). It proves that the oracle accepts the modelled state and
refuses each listed mutation. It does not prove that the model matches the
service. That correspondence rests on the native runs above.

## Review

An independent AI review of `d9285ca3` (read-only) gave APPROVE with these
items:

- the design document did not yet describe S5 (fixed);
- the rights-refusal counter was gated on `not(native-system)` instead of
  the archive-device test profile (fixed);
- the `l2-export` table writes were not pinned (fixed: 11 and 11 · 34);
- the host tests are self-consistent rather than independent (recorded
  above).

An AI review is not a human audit.

## Pins

The project owner instructed on 24 November 2026 that no integration
coordinator exists and that S moves its own pins. Both stage-count pins
(`.github/workflows/kvm-pilot.yml`, `.github/workflows/volume-acceptance.yml`)
are set to 30 in the same pull request.
