# Design typefaces: provenance and licences

All faces below are variable TrueType fonts under the SIL Open Font License 1.1.
Each licence text is stored next to its font. The files were retrieved on
13 November 2026 from the `google/fonts` repository, branch `main`
(`https://raw.githubusercontent.com/google/fonts/main/ofl/<family>/<file>`).
They are used by the design concepts, the interactive prototype and the brand
generator (`ui/design/brand/generate_mark.py`). The OFL forbids selling the
fonts by themselves and reserves the font names; bundling them with BaryonOS
is permitted.

| File | Family path | SHA-256 | Use |
|---|---|---|---|
| `Archivo[wdth,wght].ttf` | `ofl/archivo/` | `0e094a7d3c7c4c25cf1310c4b30014f1dae9332220b1c2c88f4fa996f0b05053` | Wordmark; concept A |
| `InstrumentSans[wdth,wght].ttf` | `ofl/instrumentsans/` | `b24f1812584816958afcf22e22d08e44318c5e51651e25d2438efdde389b33b1` | Concept C |
| `FunnelDisplay[wght].ttf` | `ofl/funneldisplay/` | `b4151c9c4b7b07eb74320096b4ff4156cca8821f5adfab34af9fd9a2d6c1179d` | Concept B (display) |
| `FunnelSans[wght].ttf` | `ofl/funnelsans/` | `652c9834434bc01835c4e75a73d3c7e42ff2e4beb261cc851cb911e889af6a77` | Concept B (text) |

| Licence file | SHA-256 |
|---|---|
| `OFL-Archivo.txt` | `108b4e57c9c796d3d38d0428ca7ee39de47ad93187302718d9b2d8864b9b716b` |
| `OFL-InstrumentSans.txt` | `9e27a72ed30eb49a08678f6a5d6ed98ec7ba5368f541637ee0683ec9134ef966` |
| `OFL-FunnelDisplay.txt`, `OFL-FunnelSans.txt` | `a32303f6ea3542927121212a9e4f3fe3548741e6d448d9dc6f29e97cf50102e6` |

The native renderer does not load these files. It draws text from a prebuilt
Open Sans atlas (`ui/graphics/assets/font-provenance.json`,
`ui/native/THIRD-PARTY.md`). Moving the native UI to a design typeface requires
a new atlas, a new provenance entry with hashes, a text-metric oracle review and
a budget check against the 512 KiB read-only segment. That work is recorded as
open in `docs/design/NATIVE-FEASIBILITY.md`.
