# R02 candidate (1): SUR1 reply wait with `ipc_wait` (27 November 2026)

Local evidence for R02 candidate (1) from
[`../r02-window-baseline-20261126/`](../r02-window-baseline-20261126/): the
SUR1 client (`ui/surface-protocol/src/client.rs`) waits for each reply with
`ipc_wait` and one absolute deadline of 150 PIT ticks (U's decision,
#589 issuecomment-7213649852), instead of spinning on `yield_now`.
Transports that cannot block keep the yield path. This is **local evidence,
not hosted CI acceptance**, physical-hardware evidence or a product-latency
claim. No criterion or progress point changes.

## Configuration

- A: `16071bdc9ca8cb40e7233f6e267529260fc46cb0` (branch
  `claude/r02-sur1-ipc-wait-20261126`, worktree `r-turn-a`). B: `main`
  `20d9089fe78a400143272aa34aa73cfb19f2e992` (after #586, worktree
  `r-base`). Both trees clean. Profile `native-window-session-probe`; kernel
  ELF A `95d22379…`, B `a5005cbc…` (the kernel embeds the changed app ELFs).
- `main` later moved to `bbf4df38`. Its kernel change (`ac495b2f`) is limited
  to `cfg(feature = "native-ai-job")`, which `compile_error!` excludes
  together with `ipc-wait` and `framebuffer`, so B represents the
  window-session kernel on current `main`; `git merge-tree` with `bbf4df38`
  is clean.
- [`ab7.sh`](ab7.sh): one build per side, then five window sessions per side
  with the CI flags `tools/run_window_session.py --clipboard --workspace
  --timeout 1800` (scale 1), SeaBIOS ROMs, `NOS_QEMU_ACCEL=kvm`, interleaved
  A1 B1 B2 A2 A3 B3 B4 A4 A5 B5.
- [`native.sh`](native.sh) and [`native-recover.sh`](native-recover.sh):
  native acceptance on A under TCG (`NOS_QEMU_ACCEL` unset, as in hosted CI):
  window session scale 1 and 2, then Materiaal normal, keyboard and recover.
  The first recover attempt used a separate build directory and the harness
  refused it before QEMU started (no validated parent in that `BUILD/run`);
  the rerun builds the recover feature in the directory that holds the
  passing normal parent, as in `docs/contracts/NATIVE-MATERIAAL-PROBE.md`.
- [`analyse.py`](analyse.py) derives [`summary.txt`](summary.txt) from the
  reports. QEMU 10.2.1, `-cpu qemu64 -smp 1 -m 256M` as fixed by the runner;
  shared 32-thread host, one-minute load 2.1–6.0 during the A/B; from about
  10:26Z another session ran short functional QEMU boots
  ([`meta.txt`](meta.txt), [`native-meta.txt`](native-meta.txt)). Raw
  reports, logs and screendumps stay local; their SHA-256 values are in
  [`report-sha256.txt`](report-sha256.txt).

## Results

Acceptance per run is `raw_qemu_exit` 33 and `passed` true in the report; the
runner exit is listed separately in the meta files.

| Run set | raw_qemu_exit | passed | runner exit |
|---|---|---|---|
| A1–A5, KVM (`--clipboard --workspace`) | 33 ×5 | true ×5 | 0 ×5 |
| B1–B5, KVM (`--clipboard --workspace`) | 33 ×5 | true ×5 | 0 ×5 |
| A, TCG window session scale 1 / scale 2 | 33 / 33 | true / true (122.7 s / 315.7 s) | 0 / 0 |
| A, Materiaal normal, keyboard, recover | 33 ×3 | true ×3 | 0 ×3 |

`last_input_to_verified_app_image_s` (host-side, 0.2 s screendump retry grid),
median (min–max) over five KVM runs:

| Phase | A | B | B−A |
|---|---|---|---|
| `document-menu-high-contrast` | 0.49 (0.49–0.70) | 0.70 (0.68–0.70) | +0.20 |
| `document-menu-contrast-restored` | 0.49 (0.47–0.70) | 0.70 (0.70–0.70) | +0.21 |
| `reopened` | 0.42 (0.39–0.44) | 0.44 (0.42–0.44) | +0.02 |
| `list` | 0.37 (0.37–0.42) | 0.42 (0.42–0.42) | +0.05 |
| `editor` | 0.40 (0.40–0.40) | 0.40 (0.40–0.45) | 0.00 |

- Whole run: duration median 36.86 s (A) and 37.60 s (B); QEMU host CPU median
  24.19 s and 24.93 s (host-process accounting, not guest vCPU utilisation).
- The two high-contrast phases move by one step of the retry grid: A verified
  on the earlier screendump in at least three of five runs, B in none. The
  resolution of this measurement is the 0.2 s grid.
- All other phases differ by at most ±0.05 s in the median, except
  `clipboard-rejected` (A 0.07–0.28, B 0.07–0.28), whose values fall on the
  retry grid on both sides, so the median shift is not a measured change.
- Recovery path (phases with elapsed `seconds` only): `calculated` →
  `recovered` 2.21 s (2.16–2.23) on A against 2.25 s (2.23–2.33) on B;
  `recovered` → `calculator-restored` 1.71 s on both. Retiring the Materiaal
  surface during recovery therefore wakes the waiting client; no run shows a
  1.5 s deadline stall.
- Materiaal recover on A ends read-only with `ERRORS` 1, `SAVE_ACKS` 0,
  `DISK_WRITES` 0 and `DISK_FLUSHES` 0.

## Pixel identity

- 60 of 66 phase frames have one `frame_sha256` across all ten KVM runs; the
  generated oracle files and the paint-isolation baseline are identical.
- `workspace-work`, `workspace-search`, `workspace-notice-copied`,
  `workspace-window-menu`, `lock-veld-overlay-open` and `lock-unlocked` vary
  from run to run on A itself and on B itself. Split at y = 650
  ([`summary.txt`](summary.txt)): the taskbar band is identical in all ten
  runs for each of these phases; only the app body differs. This is the
  transition effect described in
  [`../r02-compositor-turn-ab-20261126/`](../r02-compositor-turn-ab-20261126/):
  the wait predicate checks app state, not a settled repaint.

## Not run

- Hosted `native-window-session` (scale 1 and 2) and the Materiaal probe run
  on the PR head.
- WHPX, SMP profiles, physical hardware, guest-side or scanout timestamps. No
  R02-L1 budget is evaluated. The 150-tick deadline and its clock relations
  are U's record on #589; the soft-deadline follow-up is #589.
