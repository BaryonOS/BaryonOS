# Design typefaces: provenance and licences

All faces below are variable TrueType fonts under the SIL Open Font License 1.1.
Each licence text is stored next to its font. All files come from the
`google/fonts` repository: Archivo, Instrument Sans and Funnel from branch
`main`, retrieved on 13 November 2026
(`https://raw.githubusercontent.com/google/fonts/main/ofl/<family>/<file>`);
Inter and Saira from commit `29b4ec6d`, matching the SHA-256 pins in
`ui/design/typography/fetch_candidates.py`.
They are used by the design concepts, the interactive prototype and the brand
generator (`ui/design/brand/generate_mark.py`). The OFL forbids selling the
fonts by themselves; bundling them with BaryonOS is permitted. None of the
copyright lines below declares a Reserved Font Name.

| File | Family path | SHA-256 | Use |
|---|---|---|---|
| `Archivo[wdth,wght].ttf` | `ofl/archivo/` | `0e094a7d3c7c4c25cf1310c4b30014f1dae9332220b1c2c88f4fa996f0b05053` | Wordmark v1 (Git history) and logotype study; concept A |
| `InstrumentSans[wdth,wght].ttf` | `ofl/instrumentsans/` | `b24f1812584816958afcf22e22d08e44318c5e51651e25d2438efdde389b33b1` | Concept C |
| `FunnelDisplay[wght].ttf` | `ofl/funneldisplay/` | `b4151c9c4b7b07eb74320096b4ff4156cca8821f5adfab34af9fd9a2d6c1179d` | Concept B (display) |
| `FunnelSans[wght].ttf` | `ofl/funnelsans/` | `652c9834434bc01835c4e75a73d3c7e42ff2e4beb261cc851cb911e889af6a77` | Concept B (text) |
| `Inter[opsz,wght].ttf` | `ofl/inter/`, google/fonts commit `29b4ec6d` (`ui/design/typography/fetch_candidates.py`); font version 4.001 | `29160a80ff49ddcab2c97711247e08b1fab27a484a329ce8b813d820dc559031` | UI and reading typeface of `type-1.0` (`docs/design/TYPOGRAPHY.md` sections 7 and 8): tie under the decision rule (Atkinson Hyperlegible Next 73.00, Inter 72.45 of 100); lead selection of 14 November 2026 under the project owner's delegation, reversible; the project owner can redirect it |
| `Saira[wdth,wght].ttf` | `ofl/saira/`, google/fonts commit `29b4ec6d` (`ui/design/typography/fetch_candidates.py`) | `9d050fc5a01c85f74c4257c207d50b55d1e40c37308c642f974a2c5003231dde` | Wordmark v2 (`docs/design/BRAND.md`), traced to outline paths at build time |

| Licence file | SHA-256 |
|---|---|
| `OFL-Archivo.txt` | `108b4e57c9c796d3d38d0428ca7ee39de47ad93187302718d9b2d8864b9b716b` |
| `OFL-InstrumentSans.txt` | `9e27a72ed30eb49a08678f6a5d6ed98ec7ba5368f541637ee0683ec9134ef966` |
| `OFL-FunnelDisplay.txt`, `OFL-FunnelSans.txt` | `a32303f6ea3542927121212a9e4f3fe3548741e6d448d9dc6f29e97cf50102e6` |
| `OFL-Inter.txt` | `5b9321a4298cfeb6b34354164a1c3afc3db114569984c502b9b35d988fd58c57` |
| `OFL-Saira.txt` | `f2665d4718b452b3818a877191355ac884a6b9b419d35408fe7ee487e9e8f30f` |

The native renderer does not load these files. By default it draws text from a
prebuilt Open Sans atlas (`ui/graphics/assets/font-provenance.json`,
`ui/native/THIRD-PARTY.md`). An Inter atlas generated offline from
`Inter[opsz,wght].ttf` (nine records, provenance in
`ui/graphics/assets/inter-atlas-provenance.json`) sits behind the cargo feature
`inter-atlas`, off by default; it replaces the Open Sans atlas and is never
linked beside it. Its evidence is host-only, with no QEMU run
(`docs/verification/native-inter-atlas-20261114/`: the linked `materiaal-app`
read-only segment is 293 248 B by default and 353 736 B with the feature, of
512 KiB). Shipping it still needs the `ui/native` licence and package entries,
a text-metric oracle review and QEMU evidence (open items 6 and 7 of that
record).
