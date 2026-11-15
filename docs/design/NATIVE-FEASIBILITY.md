# Native feasibility of the new visual identity

Status: **analysis** of source `c76daf7b`, 13 November 2026. It records what
the native renderer can draw, which tests pin the current look, and how a
restyle may change them legitimately. Line references are to that commit.

## 1. Renderer capabilities (`ui/graphics`)

| Available | Not available |
|---|---|
| Canvas up to 8192×8192, `0x00RRGGBB` | Backdrop blur of other windows |
| `fill_rect`, 2×2-supersampled `rounded_rect`, `stroke_rounded_rect`, `translucent_rounded_rect` (alpha over pixels already drawn) | Gradient primitive (gradients must be computed per pixel, as `theme::wallpaper` does) |
| `line`, `ellipse`, `blend`, `blend_pixel` | Image decoding or bitmap blit |
| `soft_shadow` with spread 1–64, quadratic falloff, no blur | Animation clock |
| Text from a prebuilt Open Sans atlas: 12 regular, 16 regular and semibold, 32 semibold, ×2 at 200 %; 194 Latin-1 glyphs; single line | Runtime font loading, letter-spacing, multi-line layout |
| 20 stroke icons on a 24-unit grid (`icon.rs`) | — |

All drawing is replayed per 32×8 tile, so every effect must be tile-invariant:
a pixel's colour may depend only on its coordinates and the scene, never on
tile order.

## 2. Two shells, split colour sources

- `ui/native` (Horizon, Orbit, search, Nebula pages): `Frame::render_tile`
  (`render.rs:436-472`) chooses `theme::MATERIAAL` or `HIGH_CONTRAST`, draws the
  desktop overlay on every page except Material, so Orbit is not drawn on the
  Material page. Geometry comes from `Desktop::controls` (`desktop.rs:157-316`).
- Veld (`sdk/examples/window-session`): wallpaper, window chrome, switcher,
  notifications and palette use `theme::MATERIAAL`; header, dock, lock cover and
  panels use literals in `shell.rs` (`8-14`, `601-623`, `670-831`).
- `ui/prototype/tokens.css` is compiled into `ui/native/src/design.rs` with a
  strict parser; only five values are used. It must not change on this branch.
- `theme.rs` is shared by `ui/native`, Veld, materiaal, calculator and the
  compositor. Editing `MATERIAAL` or `theme::wallpaper` in place changes the
  pixels of four other gated components.

## 3. Test oracles and how they may change

| Oracle | Legitimate update |
|---|---|
| High-contrast FNV golden array (`ui/native/tests/high_contrast.rs:161-168`) | Recompute in the same change when default-theme pixels change; the `HIGH_CONTRAST` role constants and the ≥7:1 / ≥3:1 checks stay |
| Card and dock geometry and pixel asserts (`render_regressions.rs:124-447`) | Real contracts: keep geometry; colour asserts follow named theme roles |
| Session-preview app rectangle (`run_session_preview.py`) | Regenerated from the same source each run; byte equality with the host oracle remains required |
| Veld contrast (`contrast.rs`), window-state tuples, opaque-cover checks | Re-derive from formulas if geometry changes; never lower thresholds, add masks or make the cover translucent |
| `ui/native/verification/checks.json` | Already stale at `c76daf7b` (hashes of `render.rs`, `desktop.rs`, `design.rs`, `controller.rs` differ); regenerate only through `verify.py` and say it was stale |

Weakening means shrinking regions, adding tolerances or masks, dropping
phases or scales, or lowering WCAG thresholds. None of that is permitted.

## 4. Plan for the native vertical slice

1. Add a new palette and wallpaper function **additively** in
   `ui/graphics/src/theme.rs`; leave `MATERIAAL` and `theme::wallpaper`
   untouched so Veld, materiaal, calculator and the compositor keep their pixels.
2. Add a native renderer for the BaryonOS mark from the brand constants
   (circles, polygons and clearance carving are per-pixel computable).
3. Switch only the `ui/native` shell pages (accounts, sign-in, desktop,
   search) and Orbit to the new palette and mark; restyle the Material page
   within its existing geometry.
4. Regenerate the high-contrast golden array; run the `ui/graphics` and
   `ui/native` suites, the session-preview QEMU gate and screenshots at 100 %
   and 200 %.
5. Out of scope on this branch: Veld restyle (shared literals and geometry
   oracles), a new glyph atlas (needs a provenance entry, a read-only segment
   budget check against 512 KiB and text-metric oracle review), animation.

## 5. Typeface constraint

The atlas generator (`ui/graphics/scripts/generate_font.py`) asserts the hashes
of the Open Sans source files, and those files are not on this host. A new
native typeface therefore needs a reviewed generator change and a provenance
entry. Until then, native text stays Open Sans while the design system
specifies the target typeface.

## Addendum, 14 November 2026

Sections 4.5 and 5 describe the state at `c76daf7b`. Since then a separate,
reviewed generator (`ui/graphics/scripts/generate_inter_atlas.py`) emits an
Inter 4.001 atlas with nine `type-1.0` records behind the cargo feature
`inter-atlas`, off by default; it replaces the Open Sans atlas in a build and
is never linked beside it. The linked `materiaal-app` read-only segment is
293 248 B by default and 353 736 B with the feature, of 512 KiB. This is host
evidence only: no QEMU run and no image with the feature. Open items are listed
in [native-inter-atlas-20261114](../verification/native-inter-atlas-20261114/README.md),
among them the `ui/native` licence and package entries and the Open Sans
pixel-hash oracle in `ui/native/tests/high_contrast.rs`.
