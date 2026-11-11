# Presentation slides with the existing native raster library

TASK-PLATFORM-M11-SLIDE-RENDER-CONSUMER, base `ab85d633`.
A real host consumer: import a local text deck, select a slide,
next/previous, render pixels with `ui/graphics`, output a PPM frame.
No HTML/browser, new font/graphics engine or standalone model without a consumer.
The model/render path is `no_std+alloc`; the CLI is a host program.

```sh
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/presentation/Cargo.toml --features host-cli --bin slide-preview -- --slide 2 --scale 2 < sdk/examples/presentation/fixtures/deck.txt > dia.ppm
```

`--slide` is a 1-based slide number; the optional `--scale` is 1 or 2.
Frame 640x360 or 1280x720, 24-bit RGB Netpbm P6, no alpha.
This is the P6 variant with unchanged UI RGB sample values, **without**
conversion to the BT.709 transfer function of strictly colour-accurate PPM.
Primary format source: Netpbm, *PPM Format Specification*, 7 November 2025,
<https://netpbm.sourceforge.net/doc/ppm.html>, accessed 3 November 2026.
The shared MATERIAAL colours and existing Font presets are reused.
Scale 2 uses the existing larger font presets, no claim of exactly
doubled glyph dimensions throughout or validated high-DPI/accessibility UX.

## Small explicit text profile

UTF-8 with **LF** lines; the first line of each slide is the title,
following lines are body. The exact line `---`, with an LF before and after it,
separates slides. No Markdown, PPTX, ODP or Microsoft compatibility claim.
Example:

```text
Eerste titel
Een regel inhoud.
---
Tweede titel
Meer inhoud.
```

At most 32 KiB input, 32 slides, 256 bytes/title, twelve body lines,
512 bytes/body line. No empty titles/slides or control characters except LF.
CRLF, tabs and NUL are explicitly refused, not silently rewritten.
The model borrows the original UTF-8 source without copy/loss;
limited atlas glyph coverage uses a **visible replacement character**.
That preserves source bytes, but is not full Unicode shaping/font support.
No text edit/export round trip or interpretation of content.

## Render and resource invariant

For the selected slide, output scale, buffer, full title
and body width and total text height are checked **before every pixel write**.
Content that is too wide/long gives `TextOverflow`; no ellipsis, silent clipping,
hidden slide lines or successful output. It is not an automatic layout/
word-wrap engine; the user can split lines or create an extra slide.
A slide that does not fit remains selectable and the source remains intact.
Navigating past the beginning/end does not change the selection.

The deck vector fallibly reserves at most 32 entries; source bytes remain
with the caller. Canvas buffer size at most 921600 u32 pixels (approx. 3.52 MiB),
PPM output approx. 2.64 MiB plus header. The host consumer reserves fallibly;
no native hard heap quota/OOM recovery/performance claim.
The CLI only writes stdout after import/selection/render/output allocation succeed.
The stdout write itself is not atomic and may be partial on an I/O error.

Only a path dependency on the existing `ui/graphics`, no new
registry dependency or unsafe. No writes to the existing graphics/shared workspace,
General editor, kernel, driver, input, window, B02 or launch.
No file, network, launch or trusted UI authority obtained.
A native slideshow requires the existing app/surface/input/portal owners;
this does not build a parallel broker or process lifecycle.

## Executed acceptance

```sh
export CARGO_TARGET_DIR="$PWD/target/platform-presentation-host"
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/presentation/Cargo.toml --features host-cli
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/presentation/Cargo.toml --features host-cli --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/presentation/Cargo.toml -- --check
cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/presentation/Cargo.toml --target x86_64-unknown-none
python3 -B sdk/examples/presentation/tests/interop.py target/platform-presentation-host/debug/slide-preview build/platform-presentation-evidence
```

Eight Rust tests (five in `tests/deck.rs`, three real `slide-preview` processes in `tests/cli.rs`) check import,
navigation, source preservation, two real slide frames at both scales, limits,
glyph fallback and pixel buffers that remain unchanged on errors.
The standalone Python PPM oracle checks four real output frames,
RGB/header/pixel regions and generates PNG evidence images only outside Git.
That PNG helper path is not a product encoder and does not independently validate
the complete typography. Agent inspection of those images is not a human test.
No native executable, GUI operation, save portal, presentation editor,
QEMU, physical, human or release acceptance.
