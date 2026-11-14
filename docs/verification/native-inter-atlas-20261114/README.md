# Native Inter atlas behind a feature, 14 November 2026

Phase G step 1 of the design pipeline. It adds a second, offline-generated text
atlas (Inter 4.001 at the `type-1.0` role sizes, [TYPOGRAPHY.md](../../design/TYPOGRAPHY.md)
section 8) to `ui/graphics`. The atlas sits behind the cargo feature
`inter-atlas`, which is off by default.

**What this is not.** This is host-side evidence only: builds, tests and synthetic
raster images drawn through the real `Canvas::text` path. No QEMU run, no
image with the feature, no user test. Inter is the proposed face; the project
owner can still redirect it.

Base commit `4a52d756`. Built by one agent and checked by an independent
verifier agent that re-ran every command. Both are AI agents; this is not a
human review.

## What changed

| File | Change |
|---|---|
| `ui/graphics/scripts/generate_inter_atlas.py` | Build-time generator (Pillow with raqm, `python3 -I`). Reads only `ui/design/fonts/Inter[opsz,wght].ttf` and asserts its SHA-256 against FONTS.md. Sets `wght` and `opsz` per record, bakes `cv05 cv08 ss01`, and applies `tnum` to 0–9 only. Widens the space by 2 px (15sb) and 4 px (30sb). Uses the same RLE encoding and the same 96 × 96 glyph bound as `generate_font.py`. |
| `ui/graphics/src/atlas_inter.rs`, `assets/inter-atlas.rle`, `assets/inter-atlas-provenance.json` | Generated output. |
| `ui/graphics/src/lib.rs`, `src/text.rs` | `cfg(feature = "inter-atlas")` swaps the atlas module and the record mapping. Five new tests. |
| `ui/graphics/Cargo.toml`, `ui/native/Cargo.toml` | Feature `inter-atlas`; `ui/native` forwards it for measurement only. Lockfiles unchanged. |
| `ui/graphics/licenses/OFL-Inter.txt`, `THIRD-PARTY.md`, `README.md` | OFL-1.1 text (byte copy, SHA-256 `5b9321a4…`), provenance and usage notes. |
| `ui/graphics/examples/type_specimen.rs` | Writes PPM specimens per record for visual checks. |

Record mapping with the feature:

| API | Record | Line height |
|---|---|---|
| `Size::Small` | 13 px 400 (26 px at 200 %) | 18 (36) |
| `Size::Body`, `Weight::Regular` | 15 px 400 (30) | 22 (44) |
| `Size::Body`, `Weight::Semibold` | 15 px 600 (30) | 22 (44) |
| `Size::Title` | 22 px 600 (44) | 28 (56) |
| `Font::display` | 48 px 400, magnified 2× at 200 % | 56 (112) |

The generator bakes half the leading into each glyph's top offset. Existing
callers centre text with `Font::height()`, so text sits in the same place
relative to that height.

## Results

| Check | Result |
|---|---|
| Default outputs (`atlas.rs`, `open-sans-atlas.rle`, `font-provenance.json`, `generate_font.py`, all lockfiles) | Byte-identical to `4a52d756` |
| Generator run twice in separate scratch trees | Byte-identical outputs; one flipped font byte stops it |
| `cargo +1.90.0 test --offline --locked` (`ui/graphics`), default / `--features inter-atlas` | Exit 0 / exit 0 (lib 9 / 12, clipping 7, mark 3) |
| `clippy --all-targets -D warnings`, both feature sets; `fmt --check` | Exit 0 |
| `python3 -B ui/native/verify.py` (default features) | Exit 0, run by the verifier in a scratch copy, so tracked evidence files were not rewritten |
| `ui/native` host tests with `--features inter-atlas` (informational) | 13 of 14 targets pass. `high_contrast::default_source_fingerprints` fails because its pixel hashes are tied to the Open Sans rasters |

Atlas size: 204 characters × 9 records. DATA 228 940 B, ROWS 74 982 B, GLYPHS
29 376 B, CHARS 816 B, FONTS 36 B, 334 150 B in total. Open Sans totals
273 659 B, so Inter adds 60 491 B.

Read-only bytes against the explicit 512 KiB limit
([ro-measurement.json](ro-measurement.json), [measure_ro.py](measure_ro.py)):

| Build | Default | `inter-atlas` |
|---|---:|---:|
| Linked `materiaal-app` ELF, read-only PT_LOAD MemSiz (its `linker.ld` asserts `.rodata` ≤ 512 KiB; passed in both) | 293 248 B | 353 736 B (margin 170 552 B) |
| Unlinked scalar `ui/native` rlibs, sum of `.rodata*` (upper bound) | 398 713 B | 459 204 B |

The two atlases are never linked together. Keeping both would need about
627 398 B (arithmetic, not measured), which is over the limit. With the
feature, the code segment uses 157 865 B of its 163 840 B limit.

SHA-256: `atlas_inter.rs` `41f90d7a…`, `inter-atlas.rle` `18de7e6f…`,
`inter-atlas-provenance.json` `48f5d26e…`, `generate_inter_atlas.py` `7e9475b1…`.

## Visual check (AI viewing, host images)

[zoom3x-inter-body-regular-100.png](zoom3x-inter-body-regular-100.png) and
[zoom3x-open-sans-body-regular-100.png](zoom3x-open-sans-body-regular-100.png)
show the 15 px regular record at 3×. Both agents viewed the full specimen set.

- `cv05`/`cv08` separate I, l and 1; in Open Sans, I and l are identical bars.
  The em dash is a real glyph; Open Sans draws U+FFFD in its place.
- 13 and 15 px text is soft: unhinted 4-bit coverage spreads stems over about
  2 px, as Open Sans does today. 22 px and all 200 % records look crisp.
- The renderer floors the pen per glyph, so some letter gaps are 1 px uneven.
  There is no kerning ("Ve" in "Versie" is slightly loose). Both are shared
  renderer behaviour.
- Tabular digits make "0.9.3" and "1.234,56" look gappy in running text.

## Open items

1. Owner decision on Inter. Until then the feature stays off and nothing ships with it.
2. Inter 4.001 has no U+FFFD glyph, so the fallback slot holds `.notdef` (a
   grey bar at UI sizes). TYPOGRAPHY.md 8.1 promises U+FFFD. Options: draw an
   original replacement glyph, or accept `.notdef` and change the text.
3. Word space: only 15sb and 30sb are widened. Whether 15r/30r and NBSP need it
   follows the corrected TYPOGRAPHY.md 8.2.
4. Inter's `calt` raises the colon between digits in the browser; the atlas
   stores the default colon, so a native clock's colon sits lower.
5. Tabular digits in running text versus tables and clocks: the atlas cannot
   switch per context without a second digit set.
6. The pixel-hash oracle in `ui/native/tests/high_contrast.rs` and the
   text-metric oracles of `ui/native`, Veld and the SDK examples need review
   before any app migrates.
7. `ui/native/THIRD-PARTY.md` and the Windows QEMU package script have no Inter
   or OFL entry yet. Both are required before an image is built with the feature.
8. The reading role (16/32 px) and the dense 20 px body line height are not
   built. The 48 px record has negative leading (ascent + descent 59 > 56).
9. The 200 % records are cut at the doubled optical size (`opsz` 26/30/32), and
   13 px clamps to `opsz` 14.
10. Reproducibility depends on the host's Pillow, FreeType, raqm and HarfBuzz
    versions (recorded in the provenance). The HarfBuzz version is a probe of
    the system library and may differ from the copy raqm loaded.
11. New files under `ui/graphics` change the common source-v3 hash, so stored
    verification records that bind the old hash are stale until regenerated.
