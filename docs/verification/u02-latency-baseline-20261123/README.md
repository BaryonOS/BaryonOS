# U02 interaction-latency baseline, 23 November 2026

**TCG host-observed baseline, not U02 acceptance, no budget.** `baseline.json`
carries `kind: "baseline"`, `accepted: null` and `budget: null`. No interaction
budget has been preregistered; that decision remains with the project owner.
The U02 product target in `docs/implementation/NATIVE-CLIPBOARD.md` refers to a
later WHPX reference; TCG figures are not comparable with it
(`docs/TEST-STRATEGY.md`, Performance).

## What was measured

- Source: commit `bc1f76b856900a3a6b0563c4c83ea4bce5d6c8de` (runner fix plus
  summariser), source fingerprint
  `dc8e67dfb268c9cedef7ff0aa9684797ad7aaa4ef4d864861e188a8d6daec823`.
  The later evidence commit changes only the summariser's `run_directory`
  output (base name instead of absolute path), not the runner.
- Profile: `native-window-session-probe`, scale 1 (1280x720), `--clipboard
  --workspace`, QEMU 10.2.1 TCG, machine `pc-q35-8.2`, SeaBIOS ROMs as in
  `.github/workflows/native-window-session.yml`; QEMU binary SHA-256
  `fa552a45c337a88fe71e1e1af9ebace875f30e88685a6986d9406c92efaf44cb`.
  Local Linux host (32 logical CPUs), not a hosted runner.
- Three sequential cold starts, each a fresh disposable run directory. Each
  run passed: raw QEMU exit 33, `passed`, `native_gate`, `clipboard_gate`,
  `workspace_gate` and every other assertion of the CI native-gate step.
- Interval per phase: `last_input_to_verified_app_image_s`, from the start of
  the last injected QMP input command (key, pointer or button) to the first
  exact full app-body oracle match. It includes QMP, serial polling (20-50 ms),
  oracle retries (200 ms) and screendump capture. It is neither a guest
  timestamp nor scanout latency. Only the 33 oracle-bearing phases carry it;
  workspace/switcher phases use other image checks and record no interval.

## Results (milliseconds, n = 3 per phase)

With three samples, p95 equals the maximum; these are not distribution
estimates and do not meet the 20-samples-per-stratum rule.

| Phase | n | min | p50 | p95 | max | last input |
|---|---|---|---|---|---|---|
| initial | 3 | 72 | 73 | 74 | 74 | pointer |
| editor | 3 | 11277 | 11369 | 11938 | 11938 | key |
| dirty | 3 | 668 | 689 | 716 | 716 | key |
| read-tool | 3 | 3356 | 3356 | 3564 | 3564 | key |
| read-tool-blocked | 3 | 113 | 113 | 133 | 133 | key |
| write-tool-restored | 3 | 3193 | 3393 | 3585 | 3585 | key |
| document-menu | 3 | 9491 | 9898 | 10119 | 10119 | key |
| document-menu-next | 3 | 6818 | 7465 | 7500 | 7500 | key |
| document-menu-read | 3 | 8446 | 9734 | 9910 | 9910 | key |
| document-menu-blocked | 3 | 111 | 112 | 133 | 133 | key |
| document-menu-cancelled | 3 | 91 | 112 | 112 | 112 | key |
| document-menu-write | 3 | 3149 | 3380 | 3568 | 3568 | key |
| clipboard-copied | 3 | 10433 | 11308 | 11711 | 11711 | key |
| clipboard-rejected | 3 | 1258 | 1286 | 1306 | 1306 | key |
| clipboard-numeric | 3 | 12446 | 13085 | 13288 | 13288 | key |
| clipboard-pasted | 3 | 1279 | 1305 | 1386 | 1386 | key |
| clipboard-calculator-copied | 3 | 1711 | 1897 | 1967 | 1967 | key |
| clipboard-back | 3 | 13013 | 13378 | 14092 | 14092 | key |
| clipboard-empty | 3 | 10827 | 11273 | 12078 | 12078 | key |
| clipboard-reset | 3 | 10384 | 11188 | 11333 | 11333 | key |
| rebound-editor | 3 | 96 | 100 | 100 | 100 | pointer |
| restored-editor | 3 | 12343 | 12682 | 12985 | 12985 | key |
| clipboard-recovered-empty | 3 | 11094 | 11478 | 12303 | 12303 | key |
| saved | 3 | 11183 | 11317 | 12198 | 12198 | key |
| list | 3 | 10933 | 10991 | 11472 | 11472 | key |
| reopened | 3 | 10830 | 11323 | 11780 | 11780 | key |
| window-restored | 3 | 11403 | 11833 | 12325 | 12325 | pointer |
| document-menu-mouse | 3 | 7491 | 7499 | 8986 | 8986 | pointer |
| document-menu-high-contrast | 3 | 11454 | 12286 | 13331 | 13331 | key |
| document-menu-contrast-restored | 3 | 11645 | 12105 | 12310 | 12310 | key |
| document-menu-mouse-cancelled | 3 | 7485 | 7492 | 7913 | 7913 | key |
| calculator-high-contrast | 3 | 9568 | 9617 | 10249 | 10249 | key |
| calculator-contrast-restored | 3 | 7282 | 7282 | 7529 | 7529 | key |

## Interpretation limits

- Before this change only `key()` set the interval start, so `rebound-editor`,
  `window-restored` and `document-menu-mouse` (reached by pointer, button or
  shell input) inherited an earlier key timestamp, and `initial` had none.
- Pointer-ended phases start at the last pointer event, which is often a
  parking move after a click or after recovery. `initial` and `rebound-editor`
  (about 70-100 ms) mainly show that the image was already complete when the
  parking move ended; they are not click-to-image or boot figures.
- Several key phases follow a multi-key chord sequence (for example Ctrl+A,
  Ctrl+C) or a TCG app repaint; the interval covers only the last key command.
- Values of many seconds reflect TCG emulation, full-body oracle comparison
  and 200 ms retry polling; they are not optimisation targets by themselves.

## Files and raw data

- `baseline.json`: output of `python3 -B tools/window_latency_baseline.py
  build/u02-logs/report-{1,2,3}.json --output ...` (exit 0), including each
  report's SHA-256.
- Raw reports (about 610 KB each), serial logs, frames and QEMU output remain
  in ignored `build/` (`build/u02-logs/`, `build/nos/run/window-*`):

| Run | Report SHA-256 | Duration (s) |
|---|---|---|
| 1 | `2796fafdcfe6a1ed2c21ed5d4e0c2bd9a03c876d4866fd19da3fdcc0945416b2` | 358.4 |
| 2 | `e3a51cbeb41c13c153470e69d5432abeaf6d8a1b3e15c3117afec4d90b28e7d6` | 374.3 |
| 3 | `c8dddedb95a37193df910199bf99b95b27b0196231ef4862e181ff75f27dcd07` | 359.8 |

## Not run

- Scale 2 (`native-window-session-hidpi-probe`) and the switcher part gate:
  switcher phases carry no oracle interval, so they add no samples.
- WHPX reference, hosted CI runs and the 20-sample, no-burst U02 protocol of
  `tools/window_latency.py`.
