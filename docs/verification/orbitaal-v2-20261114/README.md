# Orbitaal v2 design board — verification record (14 November 2026)

Board: `ui/design/concepts/orbitaal-v2/index.html` (SHA-256 prefix `2a658658ed2c181a`),
a static design board with six 1440×900 artboards. It is a design, not a
working OS. Evidence level: headless browser renders (viewed) and pixel
measurements; no native run, no participant test, no human audit.

## Reproduction

`python3 -I docs/verification/orbitaal-v2-20261114/generator/build_v2.py`, run
from the repository root, regenerates the board byte for byte from the v1 page
(read-only slices), `generator/v2.css` and `generator/geo.json`.

## Independent recheck (AI review)

`independent-recheck.json`: verdict ACCEPT; all 21 planner items (MF1–MF11,
G1–G8, T, W) resolved; 0 blockers, 0 majors, 6 minor findings that remain open.
Render matrix: 100 % and 200 % (device scale factor 2 at a 1440 CSS px
viewport, not a 720 CSS px reflow); dark artboards 1–3 and 6, light artboards
4–5; high contrast only as the artboard 6 specimen (the board defines no
`prefers-contrast` or `forced-colors` theme); reduced motion via
`--force-prefers-reduced-motion`; keyboard focus on three controls at 200 %;
text and component contrast sampled from rendered pixels; v1 comparison.

## Files

| Path | Content |
|---|---|
| `generator/` | Board generator and its inputs |
| `independent-recheck.json` | Recheck report (per-item verdicts, findings, checks) |
| `renders/*.webp` | Lossy WebP copies for viewing (quality 82); 200 % renders are cropped |
| `renders/ORIGINAL-PNG-SHA256SUMS` | Hashes of the full-resolution PNG renders kept outside Git |
| `SHA256SUMS` | Hashes of every file in this folder |

Not covered: a 720 CSS px reflow at 200 %, a real high-contrast theme,
watched animation timing, other browsers, native rendering.
