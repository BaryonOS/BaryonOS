# Third-party material for the native compositor

The compositor uses exactly `noto-sans-mono-bitmap =0.3.2` with the same static raster features as `ui/native`: regular, size_16, size_32, unicode-basic-latin, unicode-latin-1-supplement and unicode-specials. The crate is available from the recorded offline Cargo cache, has no build script or font parser and is built without default features. This is the existing vetted asset; no runtime font file is opened.

The Rust bitmap packaging is MIT; the derived Noto Sans Mono font data is SIL OFL 1.1. Both notices are kept unchanged under `licenses/`. The directly included pure `ui/native/src/design.rs` shares the colour tokens with `ui/prototype/tokens.css`; there is no dependency on the document controller or storage client. All path dependencies are in its own Cargo.lock.

This is not evidence of native rendering. The actual compositor ELF must be run through the dedicated DisplaySource broker.
