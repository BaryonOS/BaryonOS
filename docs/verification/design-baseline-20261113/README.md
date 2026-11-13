# Design baseline capture, 13 November 2026

Purpose: genuine native QEMU screenshots of the UI before the redesign, for
[docs/design/AUDIT-2026-11-13.md](../../design/AUDIT-2026-11-13.md). No source
file was changed for this capture; `git diff --exit-code` exited 0 afterwards.

- Source commit `c76daf7b7d75d90df1942e4c5b107834e16be9ff`, source fingerprint v3
  `e693759d89705970ecaa9c315b0423261fdc7924235af566beb363419eabd255` (all reports).
- QEMU 10.2.1 (`qemu_sha256 fa552a45…44cb`), machine `pc-q35-10.0`, TCG,
  1 vCPU, 256 MiB, disposable volume and firmware-variable copies.
- Separate build and target directories per route under
  `build/design-audit/<route>` and `target/design-audit-<route>` (ignored).

| Route | Command (abridged; full lines in `commands.tsv`) | Exit | Report | Frames |
|---|---|---|---|---|
| Session preview | `tools/run_session_preview.py` with `native-system-session-preview` | 0 | `passed=true`, 8/8 app rectangles byte-equal to host oracle | 8 at 800×600 |
| Smoke | `tools/run_window_session.py --smoke-only` | 0 | `passed=true` | 1 at 1280×720 |
| Session route | `tools/run_window_session.py --session-route` | 0 | `passed=true`, guest pass exit 33 | 19 at 1280×720 |
| Switcher | `tools/run_window_session.py --switcher --scale 1` | 0 | `passed=true`, `switcher_gate=true` | 18 at 1280×720 |
| Full gate | `tools/run_window_session.py --clipboard --workspace` | 0 | `passed=true`, `native_gate=true` | 66 at 1280×720 |

`png-index.tsv` lists every converted PNG with its size and SHA-256 (paths
abbreviated: `$AUDIT` = the scratch capture directory). PNGs were converted
from the runners' PPM screendumps with `tools/capture_graphics.encode_png`.
Sixteen representative frames and a contact sheet are stored in
`docs/design/screenshots/baseline-2026-11-13/` with their own `SHA256SUMS`
(re-encoded with Pillow, so their hashes differ from `png-index.tsv`).

Evidence limits:

- Session-route and smoke frames have no pixel oracle; capture is triggered by
  serial shell state only.
- Full-gate `workspace-*` phases compare shell regions only; 33 other phases
  compare the full app body.
- The 200 % profile and the library walkthrough were not captured.
- Timings are single samples from overlapping runs, not measurements.
- Full reports and logs were kept outside the repository (up to 644 KB each).
