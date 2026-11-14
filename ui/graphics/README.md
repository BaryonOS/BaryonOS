# Native graphics primitives

`no_std`, `forbid(unsafe_code)`, no allocator, float, device or IPC dependencies.
The caller owns the destination slice; every primitive is clipped to that
private rectangle. No source pointer or framebuffer mapping is retained.

`Canvas::new(Rect::new(x,y,width,height), &mut pixels)` validates dimensions and
buffer size before modifying pixels. Coordinates and dimensions are physical
pixels. The palette is caller-selected; colors carry no compositor authority.

- `fill_rect`, `rounded_rect(rect,radius,color)`, `stroke_rounded_rect(rect,radius,stroke,color)`.
- `line(x0,y0,x1,y1,thickness,color)`, `ellipse(rect,color)` and `Icon` on a 24-unit grid.
- `Font::new(Size::{Small,Body,Title}, Weight::{Regular,Semibold}, double)`.
  `font.measure(text)`, `font.height()`, `font.em()` use the actual proportional
  metrics, with separate physical rasters for 100/200%. Canvas `text` takes the
  top of the font line box; all advances use 26.6 fixed point. Small uses the
  regular face; Body has regular/semibold rasters. Title uses the deliberate
  semibold design weight at both scales (its Weight argument is ignored). Text is
  single-line; the caller owns wrapping and clipping decisions.
- `theme::wallpaper(canvas,screen,palette)` and `theme::MATERIAAL` are shared by
  app and compositor. They read only geometry/colors, never document data.

Fontcoverage: Basic Latin and Latin-1, ellipsis, Euro and replacement. Unsupported
codepoints use the visible replacement glyph and never rewrite stored bytes.
This atlas is not a general shaping/IME engine: no bidi, complex-script shaping,
ligatures or font fallback chain. The document editor deliberately retains its
existing mono/codepoint layout until a matching selection/caret implementation
is available. Proportional labels should not be treated as editable document cells.

The fixed RLE source and metadata are generated offline by
`scripts/generate_font.py` from hash-checked Open Sans 1.10. Every encoded row is
validated by tests. Native code has no TTF parser, filesystem access or host font
fallback. Antialias coverage has 16 levels; curves/icons use 4 samples per pixel.
All owned font data is immutable and belongs in R+NX pages. Approximately 273 KiB
of atlas/metadata requires a bounded 512 KiB RO image segment; it is not heap RAM.

Feature `inter-atlas` (off by default) swaps the atlas module for one generated
by `scripts/generate_inter_atlas.py` from the vendored Inter 4.001 (type-1.0
roles, `docs/design/TYPOGRAPHY.md` section 8): Small 13/26 px 400, Body 15/30 px
400 and 600, Title 22/44 px 600, display 48 px 400 (magnified 2x at 200%).
`cv05 cv08 ss01` are baked per character, digits 0-9 are tabular, and the
15/30 px 600 space is 2/4 px wider. `height()` is the role line height
(18/22/22/28/56, doubled at 200%); glyphs include the floored half-leading, so
centring on `height()` stays centred. Inter 4.001 maps no U+FFFD: the fallback
slot holds its visible .notdef glyph. The two atlases are never linked together:
Inter data and tables are 334,150 B (Open Sans 273,659 B); the linked
`materiaal-app` read-only segment measured 353,736 B with the feature and
293,248 B without, against 524,288 B (host build, 14 November 2026).
`ui/native` forwards the feature for measurement only. Specimens:
`cargo run --example type_specimen [--features inter-atlas] -- OUT_DIR`. Evidence and open
items: `docs/verification/native-inter-atlas-20261114/README.md`.

Run host tests with the existing Rust 1.90 toolchain:
`cargo +1.90.0 test --offline --locked --manifest-path ui/graphics/Cargo.toml`.
This proves geometry/font mechanics, not native execution, accessibility or U01.
