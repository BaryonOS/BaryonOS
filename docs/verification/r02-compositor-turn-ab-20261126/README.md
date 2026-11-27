# R02 local KVM A/B: compositor clock once per pass (26 November 2026)

Diagnostic A/B for R02 candidate (2) from
[`../r02-window-baseline-20261126/`](../r02-window-baseline-20261126/): the
window compositor (`sdk/examples/window-session/src/main.rs`) reads the clock
once per pass for the shell clock and input polling, and once more for SUR1
expiry, commit stamps and reply deadlines of the ingress batch, instead of once
per message and per reply flush. This is **local KVM evidence, not hosted CI
acceptance**, physical-hardware evidence or a product-latency claim. No
criterion or progress point changes.

## Configuration

- A: `origin/main` `ffea3883870d0aa0f26170d8676015ae31067f53` (worktree
  `r-turn-a`). B: `5aedee77896060b80603e3d44c8ee25aad66be66` (branch
  `claude/r02-compositor-turn-20261126`). Profile `native-window-session-probe`;
  kernel ELF A `4d6c288a…`, B `1aaca144…`.
- [`ab5.sh`](ab5.sh): one build per side, then five window sessions per side
  with `tools/run_window_session.py --clipboard --timeout 1800`, SeaBIOS ROMs,
  `NOS_QEMU_ACCEL=kvm`, interleaved A1 B1 B2 A2 A3 B3 B4 A4 A5 B5; then the
  Materiaal normal, keyboard and recover probes on B.
- [`ab6.sh`](ab6.sh): one high-contrast session per side with the CI flags
  `--clipboard --workspace` (scale 1), order A1 B1 B2 A2; then the comment-only
  review fix on B and a rebuild to compare the kernel ELF.
- [`analyse.py`](analyse.py) derives [`summary.txt`](summary.txt) from the
  reports. QEMU 10.2.1, `-cpu qemu64 -smp 1 -m 256M` as fixed by the runner;
  shared host, load average 3.3–5.4 ([`meta.txt`](meta.txt),
  [`meta-ws.txt`](meta-ws.txt)). Raw reports, logs and screendumps stay local;
  their SHA-256 values are in [`report-sha256.txt`](report-sha256.txt).

## Results

Acceptance per run is `raw_qemu_exit` 33 and `passed` true in the report; the
runner exit is listed separately in the meta files.

| Run set | raw_qemu_exit | passed | runner exit |
|---|---|---|---|
| A1–A5 (`--clipboard`) | 33 ×5 | true ×5 | 0 ×5 |
| B1–B5 (`--clipboard`) | 33 ×5 | true ×5 | 0 ×5 |
| A1, A2, B1, B2 (`--clipboard --workspace`) | 33 ×4 | true ×4 (`app_high_contrast_gate` true) | 0 ×4 |
| Materiaal normal, keyboard, recover on B | 33 ×3 | true ×3 | 0 ×3 |

`last_input_to_verified_app_image_s` (host-side, 0.2 s screendump retry grid),
median (min–max) over five runs:

| Phase | A | B | B−A |
|---|---|---|---|
| `reopened` | 0.72 (0.70–0.73) | 0.43 (0.41–0.48) | −0.29 |
| `list` | 0.67 (0.65–0.67) | 0.42 (0.41–0.47) | −0.25 |
| `editor` | 0.65 (0.63–0.68) | 0.40 (0.40–0.45) | −0.25 |

- Whole run: duration median 19.25 s (A) and 17.21 s (B); QEMU host CPU median
  13.86 s and 11.45 s (host-process accounting, not guest vCPU utilisation).
- All other phases differ by at most ±0.05 s in the median, except
  `clipboard-reset`: its values fall on the retry grid (A 0.13–0.46, B
  0.07–0.31; means 0.27 and 0.26 s), so the median shift is not a measured
  change.
- Materiaal recover on B ends read-only with `ERRORS` 1, `SAVE_ACKS` 0,
  `DISK_WRITES` 0 and `DISK_FLUSHES` 0. These probes use `native-compositor`,
  so they guard against regressions but do not exercise this change.

## Pixel identity

- 26 of 31 phase frames (`--clipboard`) have one `frame_sha256` across all ten
  runs; the generated oracle files and the paint-isolation baseline are
  identical.
- High contrast: `document-menu-high-contrast`,
  `document-menu-contrast-restored`, `calculator-high-contrast` and
  `calculator-contrast-restored` have one `frame_sha256` across A1, A2, B1 and
  B2.
- `recovered` and `rebound-editor` differ only in run 2, identically on A and
  B.
- `clipboard-rejected`, `clipboard-pasted` and `clipboard-back` vary from run
  to run on A itself (and in the 26 November baseline on `ffea3883`). They are
  captured right after F6 moves focus while the taskbar and app body are still
  repainting; the wait predicate checks app state, not the taskbar. Split at
  y = 650 ([`summary.txt`](summary.txt)): after F6 to the calculator, A shows
  the previous taskbar state in all five runs, B the current one in four of
  five with identical body content in all five.
- The same transition effect explains the `--workspace` differences in
  `workspace-work`, `workspace-search` and `lock-veld-overlay-open`, which
  also differ between A1 and A2. In `lock-unlocked` only B1 differs (A1, A2
  and B2 are identical).
- Review fix (comment only): the rebuilt B kernel ELF is byte-identical
  (`1aaca1444a964b97…` before and after), so the results above apply to the
  final head.

## Not run

- Hosted `native-window-session` (scale 1 and 2) runs on the PR head.
- Scale 2 locally, TCG, WHPX, SMP profiles, physical hardware, guest-side or
  scanout timestamps. No R02-L1 budget is evaluated.
