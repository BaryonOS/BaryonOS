# R02 local KVM A/B: Materiaal changed-region repaint (25 November 2026)

Before/after measurement for the Materiaal client paint-path change on branch
`claude/r02-dirty-repaint-20261125`. It is **local KVM evidence**, not hosted
CI acceptance, physical-hardware evidence or a product-latency claim. No
criterion, budget or progress point changes. Baseline that motivated the
change: `docs/verification/r02-window-baseline-20261125/` (PR #526).

## Change under test

In retained mode the existing sparse scan in
`sdk/examples/materiaal/src/lib.rs::paint_step` renders every tile from the
current view and confirms (SUR1 RETAIN) runs of tiles whose bytes match the
FNV-1a digest of what the surface already shows. Before the change, retained
mode disabled the digest skip, so the conservative (often whole-viewport)
controller damage re-sent about 1785 tiles after most view changes.

Safety conditions: a digest enters the cache only after the tile's blit is
acknowledged; RETAIN only after a complete frame and with a retained base;
the last tile always commits; tiles blitted by a superseded partial frame are
re-sent until the next complete frame (the receiver refuses RETAIN for tiles
changed since its base frame, which would fail the surface).

## Configuration

- A: base `3b9a94c52bb4d3ada5a9c6499817560b71d0e4d0` (`origin/main` after
  #519). B: `c37218944af8ab2c1223cc69d822df78d0c2a20a`.
- Command: [`ab.sh`](ab.sh): Materiaal normal probe on B, then ten interleaved
  `tools/run_window_session.py --clipboard --timeout 240` runs per side
  (profile `native-window-session-probe`, SeaBIOS ROMs,
  `NOS_QEMU_ACCEL=kvm`). The recorded batch reused A's image from the
  preceding batch at the same commit; the committed script also builds A.
- QEMU 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3), `-cpu qemu64 -smp 1 -m 256M`
  as fixed by the runner; host with 32 logical CPUs shared with other
  sessions. One-minute load average before each run: 1.13–2.65
  ([`meta.txt`](meta.txt)).
- Raw reports and logs stay local; SHA-256 values in
  [`report-sha256.txt`](report-sha256.txt). Full table:
  [`summary.txt`](summary.txt).

## Results

- Window session: A 10/10 and B 10/10 pass with `raw_qemu_exit` 33. Duration
  A median 26.93 s (25.68–27.58), B median 18.69 s (18.55–19.44).
- Materiaal normal probe on B: pass, raw 33.
- `last_input_to_verified_app_image_s` (median, host-side, 0.2 s retry grid):
  ten phases that repainted the app body drop from 0.85–1.08 s to 0.10–0.27 s,
  for example `document-menu` 0.86 → 0.23 s, `clipboard-back` 1.08 → 0.27 s
  and `saved` 0.87 → 0.10 s. `editor` (0.83 → 0.63 s), `list` (0.85 → 0.61 s)
  and `reopened` (0.85 → 0.67 s) change most of the body and gain less.
- Native `materiaal-app` `.text`: 0x27e93 at B against 0x27ee3 at A, under the
  0x28000 linker assertion (`sdk/examples/materiaal/linker.ld`).

## Findings

- `document-menu-cancelled`: A 0.03 s, B 0.25 s (median). The phase injects
  Shift+F10 then Esc. B blits the menu tiles before Esc supersedes that
  generation and then restores them; A has not reached those tiles when Esc
  arrives, so its screen never changes and the expected image equals the
  pre-input one. Guest counters at `a155749c` show GEN 24 → 26 with one
  completed frame on B. The A value is therefore not a latency measurement
  of the same work. Intermediate heads re-sent more: 0.44–0.66 s at
  `5cd55d1c` (full repaint after a supersede) and 0.64–0.66 s at `a155749c`
  (everything below the superseded position); `c3721894` re-sends only the
  hull of tiles that frame blitted.
- One B run at `a155749c` timed out in `calculator-restored` (Calculator
  `EVALUATIONS` stayed 2, compositor `FOCUSED` 0, guest still running). Over
  all local batches: base 30/30 passes, fix heads 21/22. Not reproduced in
  the ten runs at `c3721894`. Cause not established; it is a Calculator
  focus/input step, but a timing effect of the faster Materiaal paint is not
  excluded. Kept open.
- Pre-existing on `main`, outside this change: input loss on an idle view
  does not repaint, so the InputLost banner appears only on the next view
  change (issue #537, owner U).

## Not run

- Materiaal recover probe: fails on `main` until #523; to be run after
  merging `main` (required before merge).
- Hosted CI, TCG, WHPX, SMP profiles, physical hardware, guest-side or
  scanout timestamps, energy. No candidate budget is evaluated.
