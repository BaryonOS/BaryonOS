# Native text sources

The proportional system text, vector icons and geometry come from the
local, dependency-free `nieuw-os-graphics` crate. The hashed Open Sans 1.10
source, offline raster generation and Apache 2.0 notices are in
[graphics/THIRD-PARTY.md](../graphics/THIRD-PARTY.md). The app parses no TTF.
The existing Noto Sans Mono subset below remains exclusively for the
editable document text, so that caret/selection metrics match the text.

# Noto Sans Mono bitmap backend

- Crate `noto-sans-mono-bitmap = 0.3.2`, defaults off; regular, size16/32,
  unicode-basic-latin, unicode-latin-1-supplement, unicode-specials.
- Sparse index version published 2026-01-07; current non-yanked version at
  source check on 2026-10-30. MSRV 1.56.1, edition 2021, no_std, no runtime
  dependencies/build script/alloc/floats. The minifb host example is exclusively an
  upstream dev-dependency and is not included in this graph.
- Tarball https://static.crates.io/crates/noto-sans-mono-bitmap/noto-sans-mono-bitmap-0.3.2.crate
  SHA-256 `f102c477fd4ec864c149803bf606ecc23bc9329602ec0f99073a728abf454abc`.
- Upstream git `636fb222507237549c20e9b23dae54fc9d639ae3` from the actual
  `.cargo_vcs_info.json`. Codegen font `codegen/src/res/NotoSansMono-Regular.ttf`
  at that commit: 405892 bytes, version 2.014, SHA-256
  `56134342e3d5137fbfcb25361039d4a98def7df75db7dd821ef47c3ae03127b6`.
- TTF name-table0: Copyright 2022 The Noto Project Authors
  (https://github.com/notofonts/latin-greek-cyrillic); name-table13/14 explicitly
  declare SIL Open Font License 1.1. The original TTF file has not been
  modified, redistributed or included in Git as an ordinary asset.
- Crate library MIT notice: [licenses/noto-sans-mono-bitmap-MIT.txt](licenses/noto-sans-mono-bitmap-MIT.txt),
  upstream SHA `d084589aca58e74853c2d03f8a1f7b6f3a13f1fcc4b0c451f30455da09c624b7`.
- Font notice with full OFL1.1 text: [licenses/NotoSansMono-OFL.txt](licenses/NotoSansMono-OFL.txt),
  primary source https://raw.githubusercontent.com/notofonts/latin-greek-cyrillic/main/OFL.txt,
  original SHA `cee9892f9f0cc8fe882c9e9537ee6a89621d86ee7ceaf70b02e2b2b1c25c061a`.
  That copyright notice matches the pinned font metadata. No trademark or
  product name derived from font authors; the font backend is separate from OS identity.

This targeted compatibility/provenance check is not a legal audit or
guarantee of maintenance. The upstream API/codegen features are not modified;
the chosen subset has an explicit glyph fallback. Later shaping/font switching
can be integrated behind the same native render/component boundary.

The OFL notice has been preserved in full in substance; only one upstream
trailing space has been removed for the repository whitespace check. The
original file hash above is kept as provenance.
