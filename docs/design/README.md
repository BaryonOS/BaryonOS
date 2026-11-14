# BaryonOS design

Design work for the BaryonOS visual identity and shell (started 13 November
2026, branch `claude/baryon-os-ui-ux-design-152adb`). Evidence levels are kept
apart: **design** (documents and boards), **simulation** (browser prototype,
labelled as such) and **native** (QEMU screenshots from the project's own
renderer). A design or simulation is never evidence of a working OS feature.

| Document | Content | Status |
|---|---|---|
| [AUDIT-2026-11-13.md](AUDIT-2026-11-13.md) | Phase A audit of the current native UI with QEMU screenshots | Done |
| [BRAND.md](BRAND.md) | BaryonOS mark, variants, wordmark, how the mark informs the system | Wordmark v2 (Saira 600 at 125 % width), selected by a pre-registered AI review, not a human test (route (a) was not blind; see the [record](../verification/logotype-review-20261114/README.md)); the original raster wordmark stays the reference and reversible fallback, and v1 (Archivo) is in Git history; the project owner can redirect it |
| [NAMING-AND-IA.md](NAMING-AND-IA.md) | Naming review, recommended names, IA, vocabulary, localisation, old → new mapping | Proposal P-NAMES-1, awaiting project-owner decision |
| [NATIVE-FEASIBILITY.md](NATIVE-FEASIBILITY.md) | Renderer capabilities, test oracles, plan for the native slice | Analysis |
| [CONCEPTS.md](CONCEPTS.md) | Three visual directions, rubric critique and selection | Orbitaal selected, v2 revision pending |
| [TYPOGRAPHY.md](TYPOGRAPHY.md) | Typeface research, measurements, specimens, selection, token specification, native assessment | Tie under the decision rule (Atkinson Hyperlegible Next 73.00, Inter 72.45 of 100); Inter selected by the lead on 14 November 2026 under the project owner's delegation, reversible. Tokens `type-1.0`. Native integration: font vendored; Inter atlas (nine records) behind the `inter-atlas` feature, off by default, host evidence only ([record](../verification/native-inter-atlas-20261114/README.md)); default build still Open Sans |
| DESIGN-SYSTEM.md | Tokens, components, motion, themes, accessibility | Pending |

Assets: `ui/design/brand/` (generated mark files), `ui/design/fonts/`
(OFL typefaces with [provenance](../../ui/design/fonts/FONTS.md)),
`ui/design/concepts/` (concept boards), `ui/design/typography/` (measurement
and specimen tooling).
