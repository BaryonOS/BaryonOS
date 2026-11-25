# H06-T8 exact-line fixture marker matching — 24 November 2026

Scope: host-oracle hardening of `evaluate_fixture` in `tools/run_hw_inventory.py`,
plus a native QEMU/TCG rerun of every HW-INV case and terminal fixture on the new
source fingerprint. This is not physical hardware, not a signed hardware record
and not H06 acceptance.

| Item | Value |
|---|---|
| Source commit | `a4b4d55` (branch `claude/sweet-franklin-ljra1f` after merging main `a2595a3a`, which contains #500 and #504), clean tree. The first run on `013b60ad` (fingerprint `74c732d2…`) also passed; this rerun rebinds the evidence because main changed `tools/`. |
| Source fingerprint (v3) | `56ff972a05ed7cc1ae038fc54a7d4d869cafb8d62229b5c1ceb0310a6eab8583` |
| Emulator | QEMU 8.2.2 (cloud container), machine `pc-q35-8.2` (the hosted CI pin), TCG, one CPU, 256 MiB, OVMF without enrolled keys, `isa-debug-exit` at `0xf4` |
| Gate | the steps of `.github/workflows/native-hw-inventory.yml`, run in order; every step's raw exit code was 0 |

## Change

`evaluate_fixture` found each fixture marker with `line.startswith(marker)`. A line
such as `M2:PASSED`, `HWINV:BEGIN_X` or `HWINV:FAIL:FIXTURE2` therefore satisfied
`M2:PASS`, `HWINV:BEGIN` or `HWINV:FAIL:FIXTURE`. The new `marker_matches` keeps
prefix matching only for markers that end in `:` and carry a value
(`M2:PANIC_LINE:`), and then requires a non-empty value. Every other marker must be
the whole line. This closes the CodeRabbit findings 6047166675, 6047799391 and
6048042333 on #500.

All fixture markers in the committed H06-T7 logs are exact lines, except
`M2:PANIC_LINE:0000000310`; the stricter rule therefore changes no recorded
verdict.

## Results

| Case | Result | Raw QEMU exit |
|---|---|---|
| `plain`, `iommu` | passed; QMP `query-pci` equal | 0 after QMP `quit` |
| `image` | passed; image SHA-256 `7a646569…` (see Limits) | 0 |
| `missing-kernel` | passed; `HLT=1` | 0 |
| `panic`, `m2-fault`, `hwinv-fail`, `serial-timeout` | passed; `HLT=1` | 0 |

Host and static steps: `test_hw_*.py` 98 tests OK (34 skipped), `hw_evidence.py
--check`, `hw_matrix.py --check`, `cargo test`/`clippy -D warnings` for `pci-core`
and `boot-protocol`, and `cargo fmt --check` all exited 0. The result files are in
this directory; run directories were under the ignored `build/r/`.

## Mutant

Reverting `marker_matches` to `line.startswith(marker)` fails the two new host
tests (`test_full_line_markers_need_an_exact_line`,
`test_value_markers_need_a_value`); the source was restored afterwards.

## Limits

- The image hash differs from the `a4ca34eb…` recorded for H06-T6. The loader is not
  bit-reproducible across build environments until B04 (#493) lands; the kernel
  and image layout are unchanged by this task.
- Not run: physical hardware, local `actionlint` (no workflow change), default and
  M2 smoke (no kernel or loader change in this task).
