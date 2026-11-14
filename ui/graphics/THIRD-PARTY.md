# Open Sans font data

Open Sans Regular and Semibold, version 1.10. Original glyph outlines are by
Steve Matteson; digitized data copyright (c) 2010–2011 Google Corporation.
Font name/license metadata was read from both actual source TTF name tables.
Both declare Apache License 2.0; complete license is in `licenses/Apache-2.0.txt`.
The exact installed source files and SHA-256 values are recorded in
`assets/font-provenance.json` and enforced by the offline generator.

Source: Debian fonts-open-sans package, upstream https://www.opensans.com/.
Regular SHA-256 e64e508b2aa2880f907e470c4550980ec4c0694d103a43f36150ac3f93189bee.
Semibold SHA-256 aa3b0ef53db12e3d45094030cac0e69d384e44cc5978643dd4390041cad546e2.

The TTF binaries are not bundled or parsed at runtime. The derived coverage
atlas retains the copyright/license notices. This is a fixed offline asset,
not a dynamic external library or a claim of current upstream maintenance.
Pillow/FreeType generated the asset on the host; neither is linked into Nieuw OS.
All drawing/vector icon code is original project code; no platform icon set or
existing OS implementation has been copied.

# Inter font data (feature `inter-atlas` only)

Inter, variable font `Inter[opsz,wght].ttf`, version 4.001. Copyright 2020 The
Inter Project Authors (https://github.com/rsms/inter). Licence: SIL Open Font
License 1.1 (OFL-1.1); the complete text is in `licenses/OFL-Inter.txt`
(SHA-256 5b9321a4298cfeb6b34354164a1c3afc3db114569984c502b9b35d988fd58c57,
identical to `ui/design/fonts/OFL-Inter.txt`). The copyright line declares no
Reserved Font Name.

Source: the vendored `ui/design/fonts/Inter[opsz,wght].ttf`, retrieved from
google/fonts commit `29b4ec6d` and recorded in `ui/design/fonts/FONTS.md`.
SHA-256 29160a80ff49ddcab2c97711247e08b1fab27a484a329ce8b813d820dc559031.

`scripts/generate_inter_atlas.py` rasterises it offline (Pillow, FreeType,
raqm/HarfBuzz) into `assets/inter-atlas.rle` and `src/atlas_inter.rs`; the
records, features and tool versions are in `assets/inter-atlas-provenance.json`.
The atlas is a rasterised derivative of the font software and is distributed
under the same OFL-1.1 terms; this is a conservative reading, not legal advice.
It is compiled only with the cargo feature `inter-atlas`; default builds contain
no Inter data. The TTF is not bundled or parsed at runtime.
