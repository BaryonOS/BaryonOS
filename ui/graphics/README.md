# Native graphics primitives

`no_std`, `forbid(unsafe_code)`, no allocator, float, device or IPC dependencies.
The caller owns the destination slice; every primitive is clipped to that
private rectangle. No source pointer or framebuffer mapping is retained.

`Canvas::new(Rect::new(x,y,width,height), &mut pixels)` validates dimensions and
buffer size before modifying pixels. Coordinates and dimensions are physical
pixels. The palette is caller-selected; colors carry no compositor authority.

- `fill_rect`, `rounded_rect(rect,radius,color)`, `stroke_rounded_rect(rect,radius,stroke,color)`.
- `line(x0,y0,x1,y1,thickness,color)`, `ellipse(rect,color)` and `Icon` on a24unit grid.
- `Font::new(Size::{Small,Body,Title}, Weight::{Regular,Semibold}, double)`.
  `font.measure(text)`, `font.height()`, `font.em()` use the actual proportional
  metrics, with separate physical rasters for100/200%. Canvas `text` takes the
  top of the font line box; all advances use26.6fixed point. Small uses the
  regular face; Body and Title have actual regular/semibold rasters. Text is
  single-line; the caller owns wrapping and clipping decisions.
- `theme::wallpaper(canvas,screen,palette)` and `theme::MATERIAAL` are shared by
  app and compositor. They read only geometry/colors, never document data.

Fontcoverage: BasicLatin andLatin1, ellipsis, Euro and replacement. Unsupported
codepoints use the visible replacement glyph and never rewrite stored bytes.
This atlas is not a general shaping/IME engine: no bidi, complex-script shaping,
ligatures or font fallback chain. The document editor deliberately retains its
existing mono/codepoint layout until a matching selection/caret implementation
is available. Proportional labels should not be treated as editable document cells.

The fixed RLE source and metadata are generated offline by
`scripts/generate_font.py` from hash-checked OpenSans1.10. Every encoded row is
validated by tests. Native code has no TTF parser, filesystem access or host font
fallback. Antialiascoverage has16levels; curves/icons use4samples per pixel.
All owned font data is immutable and belongs in R+NX pages. Approximately365KiB
of atlas/metadata requires a bounded512KiB RO image segment; it is not heap RAM.

Run hosttests with the existing Rust1.90 toolchain:
`cargo +1.90.0 test --offline --locked --manifest-path ui/graphics/Cargo.toml`.
This proves geometry/font mechanics, not native execution, accessibility or U01.
