# BaryonOS design system ds-1.0

Status: **design (proposal)**, version `ds-1.0`, 14 November 2026, branch
`claude/baryon-os-ui-ux-design-152adb`, baseline commit `573840e0`. ds-1.0 is
derived from the Orbitaal v2 board
(`ui/design/concepts/orbitaal-v2/index.html`). Simulation evidence (browser
renders of the gallery) and native evidence are recorded separately; neither
is part of this document's claim. The project owner can redirect any part of
it. Inter 4.001 is a reversible lead selection (`docs/design/TYPOGRAPHY.md`
§7 and §8.1); the Saira wordmark is the project owner's to redirect
(`docs/design/BRAND.md` §4). This is a design artefact, not a working OS.
Repair round 1 (same day) applied an independent verifier's findings: new
values for `--color-line-strong`, `--color-text-subtle`, `--color-text-disabled`
and their neutral twins (§4.1), the line height of weight-600 text (§6), a
compact and a neutral specimen per theme in the gallery, and this document.
Its evidence is in `build/design-evidence/ds-1.0/repair-1/`.
Repair round 2 (same day) applied a second independent verifier's findings: a
base `[hidden]` rule (§10), a 1 px gap between the contrast-theme focus ring
and white marks (§10, §13, O-K2), the ask-state privacy instrument in the
200 % strip (§14), and corrections to §6, §10, §15 and §16. Its evidence is in
`build/design-evidence/ds-1.0/repair-2/`.
Task UX-D-C1 (14 November 2026) applied the owner decision on app permissions:
two prompt kinds with three equal choices, durations defined once, a
four-field grant key, revocable grant rows and the Appmachtigingen path
(§11.1); its evidence is in `build/design-evidence/ds-1.0/c1/writer/`. Repair
round 1 (15 November 2026) added the destination to the grant scope, the grant
row's reach badge and duration chip (`.ds-grant-reach`, `.ds-grant-until`, a
760 px list cap), one `alertdialog` on the modal `<dialog>` and a race-free
prompt resolver; evidence in `build/design-evidence/ds-1.0/c1/repair-1/`.
UX-D-C1 repair round 2 (15 November 2026; not the ds-1.0 repair round 2
above, whose evidence is `build/design-evidence/ds-1.0/repair-2/`) made the
modal prompt scroll inside the viewport at 200 % (§10, §14), named Intrekken
by all four key fields (§11.1), gave the "Altijd" chip its own glyph `altijd`
(§8), aligned "herstarten" with the shell's "opnieuw opstarten" and recorded
open items O-G12 to O-G14; evidence in
`build/design-evidence/ds-1.0/c1/repair-2/`. Boundary gate for UX-D-C1:
`git diff --stat HEAD -- <protected paths>` (uncommitted edits; empty) plus
`git log --oneline 46957bac..HEAD -- <protected paths>`, which lists only the
merge of main `e186ec1b` (#335, `tools/package_token_guard.py`, not a C1
change); `git diff --stat 46957bac` over the same paths therefore prints that
one file (`c1/repair-2/boundary-start.txt`, `boundary-end.txt`).

Requirements input: [WORKSPACE-REQUIREMENTS.md](suite/WORKSPACE-REQUIREMENTS.md)
(S0, accepted 14 November 2026). ds-1.0 applies its decisions D1 (colour rule
scope, no status hue), D2 (densities, 24 × 24 targets), D3 (neutral surfaces;
values PROVISIONAL), D4 (region names) and D9 (single-pointer selection). S0
values marked PROVISIONAL stay provisional here.

## 1. Purpose, scope and evidence level

ds-1.0 is the shared visual and interaction vocabulary for the BaryonOS shell
and first-party apps: tokens, icons, components and their states, keyboard
rules, and the trust indicators that only the shell may draw.

| File | Content |
|---|---|
| `ui/design/system/tokens.css` | All tokens: type, space and density, radii, motion (`:root`); colour, place, material and elevation per theme (`:root, [data-theme="dark"]`, `[data-theme="light"]`, `[data-theme="contrast"]`); neutral-surface twins and the `[data-surface="neutral"]` rule; compact density; reduced motion. Custom properties and `@font-face` only. |
| `ui/design/system/components.css` | `.ds-` component classes, states, the focus ring, the 200 % rule, reduced-motion and high-contrast overrides. |
| `ui/design/system/icons.js` | `ICONS`: 53 keys of inner SVG markup on a 24-unit grid; re-exported unchanged by `ui/design/prototype/icons.js`. |
| `ui/design/system/index.html`, `gallery.js` | Gallery ("BaryonOS ontwerpsysteem ds-1.0"): every component in the three themes, a compact-density strip and a neutral-surface panel per theme (S0 D2, D3), a 200 % strip, the icon set, the type scale and token swatches. Keyboard behaviour is wired in `gallery.js` and is a simulation. |

**Not part of ds-1.0:** `ui/prototype/tokens.css` is the frozen Materiaal P02
token file compiled into `ui/native/src/design.rs`
(`docs/design/NATIVE-FEASIBILITY.md` §2). ds-1.0 does not read, extend or
replace it.

| Evidence level | What exists for ds-1.0 | Where |
|---|---|---|
| Design | This document, the four system files above, the v2 board | `ui/design/system/`, `ui/design/concepts/orbitaal-v2/` |
| Simulation | Headless Chrome renders of the gallery and the icon sheet; scripted key and pointer events; computed token and contrast checks. All runs by AI agents. | `build/design-evidence/ds-1.0/` (ignored by Git, untracked) |
| Native | None. No ds-1.0 token or component is drawn by `ui/graphics` or `ui/native`. | §17 |

At the time of writing `ui/design/system/` and `ui/design/prototype/icons.js`
are new and uncommitted on the branch.

## 2. Principles

| Principle | Rule in ds-1.0 | Source |
|---|---|---|
| Familiar conventions (ADR-058) | Desktop, Orbit (dock), launcher, search, windows with controls on the right, and the default shortcuts follow Windows, macOS and Linux conventions; additions never replace the familiar path. | `AGENTS.md` (ADR-058); `docs/implementation/DESKTOP-SHELL.md` §1, §2 |
| Materiaal is the data and permissions model | History, rights and storage status come from Materiaal (inspector sections Versies, Toegang, Opslag); Materiaal is not a mandatory main screen. | `AGENTS.md`; DESKTOP-SHELL.md §2 (Files row); `docs/design/NAMING-AND-IA.md` §4 |
| Colour means place | Green = this device, blue = NAS, red = internet; never decoration, never alone. Place and state are always shape + icon + text (dot device, ring with dot NAS, open ring internet, square error). Scope: chrome, trust surfaces and the place badge; user content in work regions is unrestricted. | `docs/design/CONCEPTS.md` (Selection, adopted grafts); S0 D1 |
| No status hue | Error, warning and success use `text.primary`, an icon, a word and (for error) the square `.ds-status-marker`. ds-1.0 ships no red status colour. | S0 D1; `tokens.css` (`--color-danger`, `--color-warning`, `--color-success`) |
| Calm, premium dark and light | Two first-class themes with their own values (not inversions); dark is the default. High contrast is a full third theme. | `tokens.css`; CONCEPTS.md Selection |
| Security Floor | Trust indicators (place badge in trust surfaces, privacy instrument, permission prompt) are drawn only by the shell. A familiar or trusted look grants no capability; secrets are entered only on the session-service route. | Drawing, secret entry and "a trusted look grants no capability": DESKTOP-SHELL.md §1 ("Rights remain rights"); `docs/implementation/PROFILES-AND-USERS.md` §3 (signing in, locking and signing out) and §4 invariant 2. Security Floor layering: `docs/LOCAL-PRIVACY-CONTRACT.md` §4.1. `docs/design/BRAND.md` §5 |

## 3. Tokens

### 3.1 Naming and selectors

| Prefix | Meaning |
|---|---|
| `--color-*` | Neutral roles: surfaces, lines, text, focus, selection, states, status aliases |
| `--place-device`, `--place-nas`, `--place-internet` (`--place-internet-line`) | Place colours |
| `--material-*` | Kern (trusted surface), glass, launcher, inspector, Orbit, app tiles |
| `--elev-*` | Shadows (per theme) |
| `--font-*`, `--type-<role>-size/-line/-weight` | Type-1.0 |
| `--space-*` | 4 px grid steps and density roles |
| `--radius-*` | Corner radii by role |
| `--motion-duration-*`, `--motion-ease-*` | Motion |

| Selector | Effect |
|---|---|
| `:root, [data-theme="dark"]` | Dark (default) |
| `[data-theme="light"]` | Light |
| `[data-theme="contrast"]` | High contrast |
| `[data-surface="neutral"]` | Re-points 30 roles at the achromatic twins of the nearest theme (S0 D3, values PROVISIONAL) |
| `[data-density="compact"]` | Compact sizes (S0 D2, PROVISIONAL) |
| `@media (prefers-reduced-motion: reduce)`, `[data-motion="reduce"]` | Every `--motion-duration-*` = 0ms |

Every theme block defines the same 88 themed names (87 before repair round 1
added `--color-surface-surround`), so an element with its own
`data-theme` re-resolves every colour, place, material and elevation token
(`build/design-evidence/ds-1.0/checks.log`, check 3a; re-run in `build/design-evidence/ds-1.0/repair-1/check-report.txt`). The neutral rule
resolves against the nearest theme; a nested `data-theme` panel inside a
neutral region gets the plain theme values back, which keeps trust surfaces
unchanged (S0 D3).

### 3.2 Themed tokens

Values are copied by script from `tokens.css`
(`build/design-evidence/ds-1.0/repair-1/tools/gen_tables.py`, a copy of `doc/gen_tables.py`). The last column is the
Orbitaal v2 variable or rule the dark value comes from (row by row in
`build/design-evidence/ds-1.0/repair-1/tools/mapping.tsv`, the tokens stage's mapping plus the repair-1 rows). † marks a value derived by
the tokens stage because v2 defines none for that theme, or changed in repair
round 1 (§4.1); derivations are
listed in §13 (contrast) and in `build/design-evidence/ds-1.0/work/mapping.tsv` (light).

<!-- BEGIN token-tables -->
**Neutral colour roles** (29)

| Token | Dark | Light | Contrast | Orbitaal v2 |
|---|---|---|---|---|
| `--color-bg` | `#070912` | `#E4E9F2` | `#000` | `--s0`; contrast: rule `.hc` (background) |
| `--color-surface-panel` | `#0D1020` | `#EDF0F7` | `#000` | `--s1` |
| `--color-surface` | `#12162A` | `#FBFCFE` | `#000` | `--s2` |
| `--color-surface-raised` | `#1A1F38` | `#E3E8F1` | `#000` | `--s3` |
| `--color-surface-inset` | `#080A16` | `#FFFFFF` | `#000` | `--sin` |
| `--color-surface-surround` | `#070912` † | `#E4E9F2` † | `#000` † | derived |
| `--color-line` | `#232A47` | `#D8DEE9` | `#FFFFFF` | `--ln` |
| `--color-line-strong` | `#7882A8` † | `#6B7590` † | `#FFFFFF` | derived from `--ln2` |
| `--color-text` | `#EEF1F8` | `#111726` | `#FFFFFF` | `--tx1` |
| `--color-text-muted` | `#A9B2CB` | `#424B62` | `#FFFFFF` | `--tx2` |
| `--color-text-subtle` | `#9BA2BC` † | `#515A73` † | `#F2F2F2` | derived from `--tx3` |
| `--color-text-disabled` | `#9BA2BC` † | `#515A73` † | `#F2F2F2` † | derived from `--tx3` |
| `--color-focus` | `#EEF1F8` | `#111726` | `#FFE34D` | `--focus` |
| `--color-selection` | `rgba(220,230,245,.085)` | `rgba(18,24,33,.065)` | `#333333` † | `--selbg` |
| `--color-hover` | `rgba(220,230,245,.07)` | `rgba(18,24,33,.05)` | `#1A1A1A` † | `--hov` |
| `--color-hover-strong` | `rgba(220,230,245,.12)` | `rgba(18,24,33,.09)` | `#262626` † | `--hov2` |
| `--color-rim-a` | `rgba(226,233,255,.46)` | `#FFFFFF` | `#FFFFFF` | `--rim-a` |
| `--color-rim-b` | `rgba(150,165,220,.10)` | `rgba(70,90,140,.16)` | `#FFFFFF` | `--rim-b` |
| `--color-rim-c` | `rgba(150,165,220,.20)` | `rgba(70,90,140,.30)` | `#FFFFFF` | `--rim-c` |
| `--color-silver` | `#C9D3E6` | `#5B667F` | `#FFFFFF` † | `--silver` |
| `--color-blade` | `#E6EBF5` | `#2B3245` | `#FFFFFF` | `--blade` |
| `--color-blade-glow` | `rgba(230,235,245,.35)` | `rgba(43,50,69,.18)` | `transparent` | `--blade-glow` |
| `--color-well` | `rgba(3,6,11,.42)` | `rgba(18,30,48,.045)` | `#000` | `--well` |
| `--color-scrim` | `rgba(3,5,9,.55)` | `rgba(222,229,238,.40)` | `#000000` † | `--scrim` |
| `--color-button-primary-bg` | `linear-gradient(180deg,#F4F6FA,#D9DFEA)` | `linear-gradient(180deg,#232B42,#121828)` | `#FFFFFF` † | `--pri-bg` |
| `--color-button-primary-bg-hover` | `linear-gradient(180deg,#FFFFFF,#E8ECF4)` | `linear-gradient(180deg,#2E3853,#1A2134)` | `#D9D9D9` † | `--pri-bg-h` |
| `--color-button-primary-text` | `#0A0D1A` | `#FFFFFF` | `#000000` † | `--pri-tx` |
| `--color-button-secondary-bg` | `rgba(220,230,245,.08)` | `#FFFFFF` | `#000000` † | `--sec-bg` |
| `--color-pill-bg` | `rgba(220,230,245,.06)` | `#FFFFFF` | `#000000` † | `--pill-bg` |

**Status (S0 D1: no status hue)** (3)

| Token | Dark | Light | Contrast | Orbitaal v2 |
|---|---|---|---|---|
| `--color-danger` | `#EEF1F8` † | `#111726` † | `#FFFFFF` † | derived from `--tx1` |
| `--color-warning` | `#EEF1F8` † | `#111726` † | `#FFFFFF` † | derived from `--tx1` |
| `--color-success` | `#EEF1F8` † | `#111726` † | `#FFFFFF` † | derived from `--tx1` |

**Place** (4)

| Token | Dark | Light | Contrast | Orbitaal v2 |
|---|---|---|---|---|
| `--place-device` | `#42D27F` | `#127A43` | `#7CF0A8` | `--green` |
| `--place-nas` | `#5AA2FF` | `#1061D6` | `#8EC5FF` | `--blue` |
| `--place-internet` | `#FF5C50` | `#CC3328` | `#FF8A80` | `--red` |
| `--place-internet-line` | `rgba(255,93,80,.5)` | `rgba(217,52,42,.5)` | `#FF8A80` † | `--ask-line` |

**Materials** (19)

| Token | Dark | Light | Contrast | Orbitaal v2 |
|---|---|---|---|---|
| `--material-kern-bg` | `#0B0E1B` | `#EEF1F5` | `#000000` † | `--kern-bg` |
| `--material-kern-tx` | `#EEF1F8` | `#111726` | `#FFFFFF` † | `--kern-tx` |
| `--material-kern-tx2` | `#B3BCD3` | `#424B62` | `#FFFFFF` † | `--kern-tx2` |
| `--material-kern-line` | `#262E4C` | `#D8DEE9` | `#FFFFFF` † | `--kern-line` |
| `--material-kern-rim` | `linear-gradient(135deg,#EEF2F8,#9AA5BE 35%,#56607E 70%,#7C88A6)` | `linear-gradient(135deg,#FFFFFF,#C9D3E6 40%,#78819A 75%,#AEB8C6)` | `#FFFFFF` † | `--kern-rim` |
| `--material-kern-warn-bg` | `#22151C` | `#FBEDEB` | `#000000` † | `--kern-warn-bg` |
| `--material-kern-warn-line` | `rgba(255,108,96,.45)` | `rgba(204,51,40,.45)` | `#FF8A80` † | `--kern-warn-line` |
| `--material-kern-warn-ic` | `#FF7A6E` | `#CC3328` | `#FF8A80` † | `--kern-warn-ic` |
| `--material-glass` | `rgba(10,15,25,.60)` | `rgba(250,252,255,.62)` | `#000` | `--glass` |
| `--material-glass-line` | `rgba(205,216,232,.14)` | `rgba(20,32,50,.10)` | `#FFFFFF` | `--glass-line` |
| `--material-launch` | `linear-gradient(180deg,rgba(20,24,46,.86),rgba(11,13,27,.92))` | `linear-gradient(180deg,rgba(255,255,255,.80),rgba(240,243,250,.86))` † | `#000000` † | rule `.launch .lbg` |
| `--material-launch-opaque` | `#0F1328` | `#F4F6FB` | `#000000` † | `--launch-opaque` |
| `--material-insp-bg` | `linear-gradient(180deg,#181D38,#12162B 70%)` | `#FFFFFF` | `#000000` † | `--insp-bg` |
| `--material-orbit-bg` | `linear-gradient(180deg,rgba(32,38,68,.74),rgba(13,16,31,.84))` | `linear-gradient(180deg,rgba(255,255,255,.80),rgba(240,243,250,.86))` | `#000` | `--dock-bg` |
| `--material-tile-bg` | `radial-gradient(120% 120% at 28% 0%,#2D3858 0%,#131B2F 55%,#080A12 100%)` | `radial-gradient(120% 120% at 28% 0%,#2D3858 0%,#131B2F 55%,#080A12 100%)` | `#000` | `--tile-bg` (:root); contrast: rule `.hc .tile,.hc .nuc` (background) |
| `--material-tile-fg` | `#DCE3F0` | `#DCE3F0` | `#FFFFFF` † | `--tile-fg` (:root) |
| `--material-tile-rim` | `inset 0 1px 0 rgba(255,255,255,.2),inset 0 0 0 1px rgba(170,188,255,.14)` | `inset 0 1px 0 rgba(255,255,255,.2),inset 0 0 0 1px rgba(170,188,255,.14)` | `inset 0 0 0 2px #FFFFFF` | `--tile-rim` (:root); contrast: rule `.hc .tile,.hc .nuc` (box-shadow) |
| `--material-tile-sh` | `0 4px 10px rgba(0,0,0,.28)` | `0 4px 10px rgba(0,0,0,.28)` | `none` † | `--tile-sh` (:root) |
| `--material-tile-pc` | `#E6EBF5` | `#E6EBF5` | `#FFFFFF` † | `--tile-pc` (:root) |

**Elevation** (7)

| Token | Dark | Light | Contrast | Orbitaal v2 |
|---|---|---|---|---|
| `--elev-window` | `22px 36px 64px rgba(0,0,0,.58),6px 12px 28px rgba(0,0,0,.42)` | `22px 36px 64px rgba(36,54,108,.22),6px 12px 28px rgba(36,54,108,.14)` | `none` † | `--wsh` |
| `--elev-window-inactive` | `14px 24px 48px rgba(0,0,0,.45)` | `14px 24px 48px rgba(36,54,108,.16)` | `none` † | `--wsh-off` |
| `--elev-orbit` | `0 18px 44px rgba(0,0,0,.55),0 2px 0 rgba(0,0,0,.3)` | `0 18px 44px rgba(36,54,108,.20),0 1px 0 rgba(255,255,255,.9) inset` | `none` | `--dock-sh` |
| `--elev-plate` | `0 18px 48px rgba(0,0,0,.28)` | `0 18px 48px rgba(36,54,108,.16)` † | `none` | rule `.plate-g` |
| `--elev-launcher` | `22px 36px 64px rgba(0,0,0,.6)` | `22px 36px 64px rgba(36,54,108,.22)` † | `none` † | rule `.launch .lbg` |
| `--elev-inspector` | `-14px 18px 40px rgba(0,0,0,.42)` | `-14px 18px 40px rgba(36,54,108,.16)` | `none` † | rule `.insp` |
| `--elev-trusted` | `0 34px 64px rgba(6,10,26,.42),0 10px 24px rgba(6,10,26,.30)` | `0 34px 64px rgba(6,10,26,.42),0 10px 24px rgba(6,10,26,.30)` | `none` † | rule `.kern` |

**Neutral-surface twins (S0 D3, PROVISIONAL)** (26)

| Token | Dark | Light | Contrast | Orbitaal v2 |
|---|---|---|---|---|
| `--color-neutral-surround` | `#090909` † | `#777777` † | `#000000` † | S0 D3 twin |
| `--color-neutral-surface-panel` | `#111111` † | `#F0F0F0` † | `#000` † | S0 D3 twin |
| `--color-neutral-surface` | `#171717` † | `#FCFCFC` † | `#000` † | S0 D3 twin |
| `--color-neutral-surface-raised` | `#212121` † | `#E8E8E8` † | `#000` † | S0 D3 twin |
| `--color-neutral-surface-inset` | `#0B0B0B` † | `#FFFFFF` † | `#000` † | S0 D3 twin |
| `--color-neutral-line` | `#2C2C2C` † | `#DEDEDE` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-line-strong` | `#858585` † | `#757575` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-text` | `#F1F1F1` † | `#171717` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-text-muted` | `#B2B2B2` † | `#4B4B4B` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-text-subtle` | `#A4A4A4` † | `#5B5B5B` † | `#F2F2F2` † | S0 D3 twin |
| `--color-neutral-selection` | `rgba(229,229,229,.085)` † | `rgba(24,24,24,.065)` † | `#333333` † | S0 D3 twin |
| `--color-neutral-hover` | `rgba(229,229,229,.07)` † | `rgba(24,24,24,.05)` † | `#1A1A1A` † | S0 D3 twin |
| `--color-neutral-hover-strong` | `rgba(229,229,229,.12)` † | `rgba(24,24,24,.09)` † | `#262626` † | S0 D3 twin |
| `--color-neutral-rim-a` | `rgba(233,233,233,.46)` † | `#FFFFFF` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-rim-b` | `rgba(167,167,167,.10)` † | `rgba(91,91,91,.16)` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-rim-c` | `rgba(167,167,167,.20)` † | `rgba(91,91,91,.30)` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-silver` | `#D2D2D2` † | `#666666` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-button-primary-bg` | `linear-gradient(180deg,#F6F6F6,#DFDFDF)` † | `linear-gradient(180deg,#2C2C2C,#181818)` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-button-primary-bg-hover` | `linear-gradient(180deg,#FFFFFF,#ECECEC)` † | `linear-gradient(180deg,#393939,#212121)` † | `#D9D9D9` † | S0 D3 twin |
| `--color-neutral-button-primary-text` | `#0E0E0E` † | `#FFFFFF` † | `#000000` † | S0 D3 twin |
| `--color-neutral-button-secondary-bg` | `rgba(229,229,229,.08)` † | `#FFFFFF` † | `#000000` † | S0 D3 twin |
| `--color-neutral-pill-bg` | `rgba(229,229,229,.06)` † | `#FFFFFF` † | `#000000` † | S0 D3 twin |
| `--elev-neutral-window` | `22px 36px 64px rgba(0,0,0,.58),6px 12px 28px rgba(0,0,0,.42)` † | `22px 36px 64px rgba(57,57,57,.22),6px 12px 28px rgba(57,57,57,.14)` † | `none` † | S0 D3 twin |
| `--elev-neutral-window-inactive` | `14px 24px 48px rgba(0,0,0,.45)` † | `14px 24px 48px rgba(57,57,57,.16)` † | `none` † | S0 D3 twin |
| `--color-neutral-focus` | `#F1F1F1` † | `#171717` † | `#FFE34D` † | S0 D3 twin |
| `--color-neutral-blade` | `#F1F1F1` † | `#171717` † | `#FFFFFF` † | S0 D3 twin |
<!-- END token-tables -->

`--color-surface-surround` is defined in every theme block (equal to
`--color-bg`; v2 has no surround) and re-pointed to `--color-neutral-surround`
under `data-surface="neutral"` (S0 D3; repair-1). Where the contrast value comes
from a v2 `.hc` rule rather than the dark source's variable, the Orbitaal v2
column names that rule ("contrast: rule …"); "derived from" names the v2
variable a † value departs from. The name changes `--dock-bg` → 
`--material-orbit-bg` and `--dock-sh` → `--elev-orbit` follow S0 D4 (avoid
"dock").

### 3.3 Type, space, radius and motion (theme-independent, `:root`)

| Token | Value | Source |
|---|---|---|
| `--font-family-ui` | `"Inter", system-ui, sans-serif` | TYPOGRAPHY.md §8.1 |
| `--font-features-ui` | `"cv05", "cv08", "ss01"` | §8.1 |
| `--font-numeric-digits` | `tabular-nums` | §8.1, §8.4 |
| `--font-weight-regular` / `-strong` | 400 / 600 | §8.2 |
| `--type-caption-*` | 13 / 18 / 400 | §8.2 |
| `--type-body-*` | 15 / 22 / 400 | §8.2 |
| `--type-body-dense-*` | 15 / 20 / 400 | §8.2 ("20 in dense rows"); S0 D2 |
| `--type-body-strong-*` | 15 / 22 / 600 | §8.2 |
| `--type-reading-*` | 16 / 26 / 400 | §8.2 |
| `--type-title-*` | 22 / 28 / 600 | §8.2 |
| `--type-display-*` | 48 / 56 / 400 | §8.2 |

| Token | Value | v2 source (tokens.css comment) |
|---|---|---|
| `--space-1` … `--space-6` | 4, 8, 12, 16, 20, 24 px | v2 `gap` and `padding` values |
| `--space-8`, `--space-10` | 32, 40 px | v2 `padding 24px 32px`, `.cap padding 0 40px` |
| `--space-target-min` | 24 px (compact 24) | S0 D2 |
| `--space-control-height` | 32 px (compact 24) | S0 D2, PROVISIONAL |
| `--space-control-inset` | 12 px (compact 8) | S0 D2, PROVISIONAL |
| `--space-gap` | 8 px (compact 4) | S0 D2, PROVISIONAL |
| `--space-panel` | 16 px (compact 8) | S0 D2, PROVISIONAL |

| Radius token | px | v2 rule | Radius token | px | v2 rule |
|---|---|---|---|---|---|
| `--radius-mark` | 2 | blades, bars | `--radius-tile` | 12 | `.tile` 40 px, `.kw`, `.tgl` |
| `--radius-keycap` | 6 | Enter keycap | `--radius-control-large` | 13 | `.btn.lg` |
| `--radius-tile-small` | 8 | `.tile.s28` | `--radius-card` | 14 | `.ob`, `.insp`, `.card` |
| `--radius-item` | 9 | `.tile.s32`, `.nb`, `.sr`, `.ni` | `--radius-window` | 16 | `.win`, `.plate` |
| `--radius-row` | 10 | `.row`, `.wc button` | `--radius-trusted` | 20 | `.kern` |
| `--radius-control` | 11 | `.btn`, `.inp` | `--radius-orbit` | 22 | `.orbit`, `.appic` |
| `--radius-pill` | 999px | `.pill` | `--radius-launcher` | 24 | `.launch .lbg` |
| `--radius-round` | 50% | nodes, dots, rings | | | |

Motion tokens are listed in §12. v2's off-grid spacings (2, 3, 5, 6, 9, 10,
11, 14, 18 px) and radii (1, 4, 7 px) are not tokenised; components snap to
the grid (`tokens.css` comments).

## 4. Themes and contrast pairs

Method: `build/design-evidence/ds-1.0/repair-1/tools/contrast_pairs.py` (copy of the documentation stage's `doc/contrast_pairs.py`), run with
`python3 -I` on `tokens.css` (WCAG 2.2 relative luminance, ratios floored to
two decimals; translucent tokens composited in 8-bit sRGB over the named base;
gradients judged at every stop, worst stop reported). Output:
`build/design-evidence/ds-1.0/repair-1/tools/contrast-pairs.md`. Thresholds: text 4.5:1
(7:1 in the contrast theme), UI components and focus 3:1, disabled text 3:1
(TYPOGRAPHY.md §8.3). `--color-line` is listed for information only: ds-1.0
uses it for dividers and for the edges of surfaces (window, menu, dialog,
toast, place badge, info banner) that are identified by fill, icon or text
rather than by that edge. The final base verifier measured these edges from
computed styles at 1440 px: 1.27:1 dark (`#232A47` on `#12162A`), 1.31:1 light
(`#D8DEE9` on `#FBFCFE`), place-badge edges 1.03:1; each of these components
is identified by its fill, icon or text instead (place badge: node shape,
icon and label), so the edges are not WCAG 1.4.11 boundaries. The new grant
row edge measures 1.27:1 dark and 1.32:1 light (identified by its text and
Intrekken button) and 21:1 in contrast (UX-D-C1 harness).

**Backdrop assumption.** Glass, Orbit material and the translucent launcher are
composited over `--color-bg`. Their contrast over the real wallpaper is not
known (§19).

<!-- BEGIN contrast-tables -->
#### dark — at rest

| Foreground | Need | bg `#070912` | panel `#0D1020` | surface `#12162A` | raised `#1A1F38` | inset `#080A16` |
|---|---|---|---|---|---|---|
| `--color-text` `#EEF1F8` | 4.5 | 17.57 | 16.70 | 15.82 | 14.31 | 17.43 |
| `--color-text-muted` `#A9B2CB` | 4.5 | 9.38 | 8.91 | 8.44 | 7.64 | 9.30 |
| `--color-text-subtle` `#9BA2BC` | 4.5 | 7.84 | 7.45 | 7.06 | 6.39 | 7.77 |
| `--color-text-disabled` `#9BA2BC` | 3.0 | 7.84 | 7.45 | 7.06 | 6.39 | 7.77 |
| `--color-line-strong` `#7882A8` | 3.0 | 5.25 | 4.99 | 4.73 | 4.28 | 5.21 |
| `--color-line` `#232A47` | info | 1.41 | 1.34 | 1.27 | 1.15 | 1.40 |
| `--color-focus` `#EEF1F8` | 3.0 | 17.57 | 16.70 | 15.82 | 14.31 | 17.43 |
| `--color-blade` `#E6EBF5` | 3.0 | 16.62 | 15.79 | 14.96 | 13.54 | 16.48 |
| `--place-device` `#42D27F` | 3.0 | 10.17 | 9.67 | 9.16 | 8.29 | 10.09 |
| `--place-nas` `#5AA2FF` | 3.0 | 7.61 | 7.23 | 6.85 | 6.20 | 7.55 |
| `--place-internet` `#FF5C50` | 3.0 | 6.52 | 6.20 | 5.87 | 5.31 | 6.47 |

#### light — at rest

| Foreground | Need | bg `#E4E9F2` | panel `#EDF0F7` | surface `#FBFCFE` | raised `#E3E8F1` | inset `#FFFFFF` |
|---|---|---|---|---|---|---|
| `--color-text` `#111726` | 4.5 | 14.67 | 15.67 | 17.41 | 14.54 | 17.88 |
| `--color-text-muted` `#424B62` | 4.5 | 7.13 | 7.62 | 8.47 | 7.07 | 8.69 |
| `--color-text-subtle` `#515A73` | 4.5 | 5.63 | 6.01 | 6.68 | 5.58 | 6.86 |
| `--color-text-disabled` `#515A73` | 3.0 | 5.63 | 6.01 | 6.68 | 5.58 | 6.86 |
| `--color-line-strong` `#6B7590` | 3.0 | 3.76 | 4.02 | 4.47 | 3.73 | 4.59 |
| `--color-line` `#D8DEE9` | info | 1.10 | 1.18 | 1.31 | 1.09 | 1.35 |
| `--color-focus` `#111726` | 3.0 | 14.67 | 15.67 | 17.41 | 14.54 | 17.88 |
| `--color-blade` `#2B3245` | 3.0 | 10.47 | 11.19 | 12.43 | 10.38 | 12.76 |
| `--place-device` `#127A43` | 3.0 | 4.43 | 4.73 | 5.25 | 4.38 | 5.39 |
| `--place-nas` `#1061D6` | 3.0 | 4.65 | 4.97 | 5.52 | 4.61 | 5.67 |
| `--place-internet` `#CC3328` | 3.0 | 4.23 | 4.52 | 5.02 | 4.19 | 5.15 |

#### contrast — at rest

| Foreground | Need | bg `#000000` | panel `#000000` | surface `#000000` | raised `#000000` | inset `#000000` |
|---|---|---|---|---|---|---|
| `--color-text` `#FFFFFF` | 7.0 | 21.00 | 21.00 | 21.00 | 21.00 | 21.00 |
| `--color-text-muted` `#FFFFFF` | 7.0 | 21.00 | 21.00 | 21.00 | 21.00 | 21.00 |
| `--color-text-subtle` `#F2F2F2` | 7.0 | 18.75 | 18.75 | 18.75 | 18.75 | 18.75 |
| `--color-text-disabled` `#F2F2F2` | 3.0 | 18.75 | 18.75 | 18.75 | 18.75 | 18.75 |
| `--color-line-strong` `#FFFFFF` | 3.0 | 21.00 | 21.00 | 21.00 | 21.00 | 21.00 |
| `--color-line` `#FFFFFF` | info | 21.00 | 21.00 | 21.00 | 21.00 | 21.00 |
| `--color-focus` `#FFE34D` | 3.0 | 16.34 | 16.34 | 16.34 | 16.34 | 16.34 |
| `--color-blade` `#FFFFFF` | 3.0 | 21.00 | 21.00 | 21.00 | 21.00 | 21.00 |
| `--place-device` `#7CF0A8` | 3.0 | 14.88 | 14.88 | 14.88 | 14.88 | 14.88 |
| `--place-nas` `#8EC5FF` | 3.0 | 11.58 | 11.58 | 11.58 | 11.58 | 11.58 |
| `--place-internet` `#FF8A80` | 3.0 | 9.19 | 9.19 | 9.19 | 9.19 | 9.19 |

#### Worst case under hover, strong hover and selection fills (every base)

| Foreground | Need | dark | light | contrast |
|---|---|---|---|---|
| `--color-text` | 4.5 / 7 | 10.37 (raised+hover-strong `#31374F`) | 12.13 (raised+hover-strong `#D0D5DE`) | 12.63 (bg+selection `#333333`) |
| `--color-text-muted` | 4.5 / 7 | 5.54 (raised+hover-strong `#31374F`) | 5.90 (raised+hover-strong `#D0D5DE`) | 12.63 (bg+selection `#333333`) |
| `--color-text-subtle` | 4.5 / 7 | 4.63 (raised+hover-strong `#31374F`) | 4.65 (raised+hover-strong `#D0D5DE`) | 11.28 (bg+selection `#333333`) |
| `--color-text-disabled` | 3.0 | 4.63 (raised+hover-strong `#31374F`) | 4.65 (raised+hover-strong `#D0D5DE`) | 11.28 (bg+selection `#333333`) |
| `--color-line-strong` | 3.0 | 3.10 (raised+hover-strong `#31374F`) | 3.11 (raised+hover-strong `#D0D5DE`) | 12.63 (bg+selection `#333333`) |
| `--color-focus` | 3.0 | 10.37 (raised+hover-strong `#31374F`) | 12.13 (raised+hover-strong `#D0D5DE`) | 9.83 (bg+selection `#333333`) |
| `--color-blade` | 3.0 | 9.81 (raised+hover-strong `#31374F`) | 8.66 (raised+hover-strong `#D0D5DE`) | 12.63 (bg+selection `#333333`) |
| `--place-device` | 3.0 | 6.00 (raised+hover-strong `#31374F`) | 3.66 (raised+hover-strong `#D0D5DE`) | 8.95 (bg+selection `#333333`) |
| `--place-nas` | 3.0 | 4.49 (raised+hover-strong `#31374F`) | 3.84 (raised+hover-strong `#D0D5DE`) | 6.96 (bg+selection `#333333`) |
| `--place-internet` | 3.0 | 3.85 (raised+hover-strong `#31374F`) | 3.50 (raised+hover-strong `#D0D5DE`) | 5.53 (bg+selection `#333333`) |

#### Component and material pairs (translucent materials composited over `--color-bg`)

| Pair | Need | dark | light | contrast |
|---|---|---|---|---|
| Primary button text on its fill | 4.5 / 7 | 14.45 | 14.04 | 21.00 |
| Primary button text on hover fill | 4.5 / 7 | 16.34 | 11.62 | 14.87 |
| Text on secondary button over raised | 4.5 / 7 | 11.63 | 17.88 | 21.00 |
| Secondary button edge over raised | 3.0 | 3.48 | 4.59 | 21.00 |
| Secondary button edge over surface | 3.0 | 3.91 | 4.59 | 21.00 |
| Input edge on inset | 3.0 | 5.21 | 4.59 | 21.00 |
| Tooltip / badge text (inset on text fill) | 4.5 / 7 | 17.43 | 17.88 | 21.00 |
| Kern text on Kern material | 4.5 / 7 | 16.99 | 15.78 | 21.00 |
| Kern secondary text on Kern material | 4.5 / 7 | 10.11 | 7.67 | 21.00 |
| Kern text on internet notice | 4.5 / 7 | 15.58 | 15.68 | 21.00 |
| Internet notice icon on notice | 3.0 | 6.93 | 4.52 | 9.19 |
| Tile glyph on tile (every stop) | 3.0 | 8.95 | 8.95 | 21.00 |
| Text on launcher, opaque fallback | 4.5 / 7 | 16.23 | 16.53 | 21.00 |
| Text on launcher, translucent over bg | 4.5 / 7 | 15.82 | 15.92 | 21.00 |
| Text on inspector (every stop) | 4.5 / 7 | 14.60 | 17.88 | 21.00 |
| Muted text on inspector | 4.5 / 7 | 7.79 | 8.69 | 21.00 |
| Text on glass over bg | 4.5 / 7 | 17.18 | 16.36 | 21.00 |
| Muted text on glass over bg | 4.5 / 7 | 9.17 | 7.95 | 21.00 |
| Text on Orbit material over bg | 4.5 / 7 | 14.45 | 15.92 | 21.00 |

#### MF1: Kern material against the page window it covers (UI, 3:1)

| Page theme | Kern theme | Kern bg | Window | Ratio |
|---|---|---|---|---|
| dark | light | `#EEF1F5` | `#12162A` | 15.78 |
| light | dark | `#0B0E1B` | `#FBFCFE` | 18.71 |
| contrast | contrast | `#000000` | `#000000` | 1.00 (edge carried by 2 px white `--material-kern-rim`) |

#### Neutral surface (`data-surface="neutral"`, PROVISIONAL) — worst case at rest over panel, surface, raised, inset; state and button fills: §4.1 (0 failures)

| Foreground | Need | dark | light | contrast |
|---|---|---|---|---|
| `--color-text` | 4.5 / 7 | 14.25 (raised) | 14.63 (raised) | 21.00 (inset) |
| `--color-text-muted` | 4.5 / 7 | 7.59 (raised) | 7.11 (raised) | 21.00 (inset) |
| `--color-text-subtle` | 4.5 / 7 | 6.45 (raised) | 5.54 (raised) | 18.75 (inset) |
| `--color-text-disabled` | 3.0 | 6.45 (raised) | 5.54 (raised) | 18.75 (inset) |
| `--color-line-strong` | 3.0 | 4.36 (raised) | 3.76 (raised) | 21.00 (inset) |
| `--color-focus` | 3.0 | 14.25 (raised) | 14.63 (raised) | 16.34 (inset) |
| `--color-blade` | 3.0 | 14.25 (raised) | 14.63 (raised) | 21.00 (inset) |
| `--place-device` | 3.0 | 8.24 (raised) | 4.40 (raised) | 14.88 (inset) |
| `--place-nas` | 3.0 | 6.17 (raised) | 4.62 (raised) | 11.58 (inset) |
| `--place-internet` | 3.0 | 5.28 (raised) | 4.20 (raised) | 9.19 (inset) |
<!-- END contrast-tables -->

### 4.1 S0 D3 enumeration and the repair-1 token change

The enumeration in the S0 D3 form (bases panel, surface, raised, inset; at rest
and under hover, strong hover, selection, secondary-button and pill fills; text
4.5:1, `line-strong`, focus and place 3:1) failed 29 plain and 17 neutral pairs
on the tokens stage's values, all `--color-line-strong` or `--color-text-subtle`.
One failure was at rest: dark `--color-line-strong` `#5D6890` on
`--color-surface-raised` `#1A1F38`, 2.96:1, the edge of inputs, progress bars and
secondary and disabled buttons. Lowering state alphas cannot clear an at-rest
failure, so repair round 1 did the other task S0 D3 assigns to this stage: it
raised `--color-line-strong` and `--color-text-subtle` (with
`--color-text-disabled`, which equals subtle) in dark and light, and their
neutral twins. State alphas keep their v2 values.

| Token | Theme | Before (v2 or S0 twin) | ds-1.0 after repair-1 | Worst D3 pair after |
|---|---|---|---|---|
| `--color-line-strong` | dark | `#5D6890` (`--ln2`) | `#7882A8` | 3.10 (raised + hover-strong `#31374F`) |
| `--color-line-strong` | light | `#78819A` (`--ln2`) | `#6B7590` | 3.11 (raised + hover-strong `#D0D5DE`) |
| `--color-text-subtle`, `--color-text-disabled` | dark | `#8C95B2` (`--tx3`) | `#9BA2BC` | 4.63 (raised + hover-strong) |
| `--color-text-subtle`, `--color-text-disabled` | light | `#58627D` (`--tx3`) | `#515A73` | 4.65 (raised + hover-strong) |
| `--color-neutral-line-strong` | dark / light | `#747474` / `#818181` | `#858585` / `#757575` | 3.12 / 3.13 |
| `--color-neutral-text-subtle` | dark / light | `#969696` / `#626262` | `#A4A4A4` / `#5B5B5B` | 4.63 / 4.62 |

Method (`build/design-evidence/ds-1.0/repair-1/tools/tune.py`): HSL lightness in steps of 0.25 %, hue and
saturation kept, smallest change that brings every D3 pair to at least 3.10 or
4.60 after flooring (0.10 above the threshold). A twin is the achromatic grey of
the same WCAG luminance as the new plain value, moved one 8-bit step where it
still fell short. The contrast theme is unchanged.

| Set | Dark | Light | Contrast |
|---|---|---|---|
| Plain values | 0 (was 20) | 0 (was 9) | 0 |
| Neutral twins (PROVISIONAL) | 0 (was 10) | 0 (was 7) | 0 |

<!-- BEGIN failure-list -->

<!-- END failure-list -->

The tables above and these counts are regenerated from the edited `tokens.css`
(`build/design-evidence/ds-1.0/repair-1/tools/contrast-pairs.md`); `build/design-evidence/ds-1.0/repair-1/tools/check_tokens.py` reports 0 hard
failures and 0 D3 open items (`build/design-evidence/ds-1.0/repair-1/check-report.txt`). Consequences:

- These values depart from the reviewed v2 board and are marked † in §3.2. This is a ds-1.0 design decision, not a revision of v2.
- The dark secondary-button edge is now 3.48:1 over raised and 3.91:1 over a
  window (O-C2 closed).
- Hierarchy: muted and subtle text are closer than in v2 (1.20:1 dark, was
  1.41:1; 1.27:1 light, was 1.43:1). Judgement: acceptable, because subtle text
  is metadata and never the only cue; review is open (O-C3).
- The neutral twin values stay PROVISIONAL (S0 D3).
- S0 D3 asks for the run under `docs/verification/`. This round could not write
  there, so the run is only in ignored evidence and the D3 check is not yet
  stored as accepted (O-C1).

## 5. Materials

| Material | Tokens | Use | Not for |
|---|---|---|---|
| Kern (trusted surface) | `--material-kern-bg/-tx/-tx2/-line/-rim`, `--material-kern-warn-*`, `--radius-trusted`, `--elev-trusted` | Permission prompts and other shell questions only. Opaque. Takes the **opposite** theme by setting `data-theme` on the Kern element (MF1); the contrast theme keeps contrast with a 2 px white rim. | Any app surface; any translucent treatment |
| Scrim | `--color-scrim` | Behind a modal dialog (`::backdrop`, `.ds-scrim`) and behind a trusted question (CONCEPTS.md, adopted grafts). Opaque `#000000` in contrast. | Decoration |
| Glass | `--material-glass`, `--material-glass-line` | Shell plates over the wallpaper only (v2 `.plate-g`). | Window interiors, text-heavy panels, anything over moving content |
| Orbit | `--material-orbit-bg`, `--elev-orbit`, `--radius-orbit` | The Orbit capsule; `.ds-orbit` adds `backdrop-filter: blur(24px)`, removed in contrast. | — |
| Launcher | `--material-launch`, `--material-launch-opaque`, `--elev-launcher`, `--radius-launcher` | Launcher panel; `--material-launch-opaque` is the required fallback over windows (MF11), because natively blur is feasible only over the static pre-blurred wallpaper. | — |
| Well | `--color-well` | Recessed areas inside a surface (v2 `--well`); in ds-1.0 only the privacy instrument's track. | Primary content |
| Inspector | `--material-insp-bg`, `--elev-inspector` | The inspector tethered to a selected row (MF8). | — |
| App tile | `--material-tile-bg/-fg/-rim/-sh/-pc`, `--radius-tile*` | Every app icon, first- or third-party, identical in both themes (MF9). Used by `.ds-ask-app`. | System controls (bare glyphs) |

Glass, launcher and inspector materials, `--elev-plate`, `--elev-launcher` and
`--elev-inspector` are tokens only: no ds-1.0 component uses them yet.

**Rule against indiscriminate glass.** Translucency is limited to the three
shell materials that sit on the wallpaper (glass plate, Orbit, launcher). Every
translucent material has an opaque value in the contrast theme, and the
launcher has an opaque fallback. Window bodies, trusted surfaces, menus,
dialogs and toasts are opaque. Contrast of translucent materials depends on
the backdrop and is unverified over real wallpaper pixels (§19).

## 6. Typography

ds-1.0 implements the `type-1.0` roles (deviations: `font-display: swap`
and the digits rule below); the specification is
[TYPOGRAPHY.md](TYPOGRAPHY.md) §8 and is not restated here.

- Roles and tokens: `--type-{caption,body,body-dense,body-strong,reading,title,display}-{size,line,weight}` (§3.3). Classes `.ds-t-caption`, `.ds-t-body`, `.ds-t-body-dense`, `.ds-t-body-strong`, `.ds-t-reading`, `.ds-t-title` and `.ds-t-display` apply them.
- Inter 4.001 is loaded by `@font-face` from `ui/design/fonts/Inter[opsz,wght].ttf` with `font-display: swap`; features `cv05 cv08 ss01` always on.
- Weights 400 and 600 only; tracking 0; no other sizes or line heights. Weight 600 at 15 px is always `body-strong` 15/22/600: buttons, the selected tab, the selected row name, lit privacy-instrument labels and the instrument's place line. The 20 px line is used only at weight 400 (`body-dense`, S0 D2); a selected compact row keeps its 24 px height with the 22 px line. Unlit instrument labels use `body` 15/22/400 so lit and unlit labels share one line box.
- Sweep: the gallery's own check (`gallery.js`, `?measure=1`) compares size, line height and weight together since repair round 1; the earlier check compared size and weight only and missed 15/20/600 on `.ds-btn`, `.ds-tab`, `.ds-row-name`, `.ds-privacy-lbl` and the instrument's place line. Repair-1 run: 1 314 visible text elements (200 % strip excluded), none outside the seven roles (`build/design-evidence/ds-1.0/repair-1/measure.json`).
- Tabular figures: `--font-numeric-digits` is set by four selectors: `.ds-num` on numeral spans (times, sizes, dates, percentages), and directly on `.ds-row-meta` (file size), `.ds-taskitem-badge` (count) and `.ds-toast-time` (time), three elements that hold only a short value; it is never set on a line of prose; the progress percentage is a `.ds-num` span and `.ds-progress-label` sets no figure style. TYPOGRAPHY.md §8.1 and §8.4 ask for tabular digits everywhere because the native atlas cannot select by context; the browser and native behaviours therefore differ for digits in prose (open item O-T1).
- Disabled text uses `--color-text-disabled` (= `--color-text-subtle`), at least 3:1 (§4), and is always paired with a non-colour cue (dashed edge, reason line).

## 7. Layout

| Aspect | ds-1.0 value | Source |
|---|---|---|
| Grid | 4 px; every `--space-*` is a multiple of 4 | `tokens.css`; tokens stage check 3d |
| Minimum target | 24 × 24 CSS px for every hit area (`--space-target-min`) | S0 D2; WCAG 2.2 SC 2.5.8 |
| Control height | 32 px standard, 24 px compact (PROVISIONAL); v2 used 36 px | S0 D2; v2 `.btn`, `.inp` |
| Large button | 44 px (`data-size="large"`, Kern and dialogs) | v2 `.btn.lg` |
| Window title bar | 52 px (`.ds-window-bar`) | v2 `.fm` grid: first row 52 px, holding the full-width `.bar` |
| Window controls | 40 × 32 px each (`.ds-window-ctl`) | v2 `.wc button` |
| Orbit | 64 px high, 48 × 48 items, 22 px from the bottom edge in v2 | v2 `.orbit`, `.ob` |
| Files window first column (sidebar) | 248 px | v2 `.fm` (`grid-template-columns: 248px 1fr`) |
| v2 artboard | 1440 × 900 | v2 `.ab` |
| v2 window examples | 916 × 640, 960 × 690, 1240 × 724 | v2 `.win` inline styles |
| v2 launcher | 860 × 608 | v2 `.launch` inline style |
| Permission prompt | 560 px (`.ds-ask`); v2 shows 600 px | `components.css`; v2 `.kern` inline style |
| Menu / toast / dialog | min 300 px / 440 px / 440 px | `components.css` |

Radii: §3.3. Elevation: `--elev-*` in §3.2; every elevation is `none` in the
contrast theme, where edges carry separation.

## 8. Iconography

| Rule | Value | Source |
|---|---|---|
| Grid | 24 units, `viewBox="0 0 24 24"` | `icons.js` header; `ui/design/prototype/contract.js` `icon()` |
| Stroke | 1.6, round caps and joins, `fill: none`, `stroke: currentColor` (48 px empty-state icons: 1.2) | `components.css` `.icon`, `.ds-empty > .icon` |
| Particle | One solid particle at a joint in app icons, `class="pc"`, `fill="currentColor"`; place colour only through `--ds-c` in `.ds-place`, lit or asking privacy columns and `.ds-ask-row` | `icons.js` header; `components.css` |
| Naming | v2 sprite ids without `i-`; app ids, file kinds and places from `ui/design/prototype/data.js`; fallback `dot` | `icons.js` |
| Markup | No inline style, no `var()`, no colour literals | tokens stage check 2 |

Icon keys (54; the gallery's `?measure=1` reports 54 keys, all used, none
unresolved, `c1/repair-2/gallery-measure.json`):

| Group | Keys |
|---|---|
| ds-1.0 addition, not in v2 (1) | `altijd` (UX-D-C1 repair-2): a closed loop with the particle on the top of its right lobe (r 2.1, as on `internet`'s edge; on the crossing it merged with the strokes at 16 px), drawn in the v2 stroke style for the "Altijd, tot je het intrekt" grant chip, so `lock` stays reserved for the Kern band and a revocable grant does not read as "locked". Kept in a separate block after the generated object in `icons.js`. That the loop reads as "until revoked" is AI design judgement, untested (O-G13) |
| v2 sprite (42) | `zoeken` `bestanden` `notities` `instellingen` `haven` `device` `nas` `versies` `toegang` `internet` `power` `lock` `assistent` `min` `max` `close` `back` `fwd` `plus` `check` `recent` `desk` `doc` `image` `download` `trash` `warn` `draw` `list` `pdf` `loc` `nofiles` `window` `chev` `coll` `huis` `archief` `web` `klok` `apps` `user` `a11y` |
| Drawn for ds-1.0 (4) | `rekenmachine`, `muziek`, `sheet`, `dot` (outlined diamond; a solid dot would read as "device") |
| Aliases (7) | `browser` = `web`, `fotos` = `image`, `note` = `doc`, `drawing` = `draw`, `desktop` = `desk`, `documents` = `doc`, `downloads` = `download` |

**Tiles versus glyphs (CONCEPTS.md MF9).** Every app icon, first- or
third-party, sits on the one app tile (`--material-tile-*`), identical in both
themes; system controls (menu, search, window controls) are bare glyphs. In
ds-1.0 only `.ds-ask-app` draws the tile; `.ds-taskitem` currently shows app
icons as bare glyphs, unlike v2's Orbit (open item O-K3).

## 9. Window chrome and desktop components

Implemented in `components.css` and shown in the gallery:

| Element | Class | Behaviour and states | DESKTOP-SHELL §2 |
|---|---|---|---|
| Window | `.ds-window`, `.ds-window-bar`, `.ds-window-title`, `.ds-window-body` | `data-state="active"` (top light blade, `--elev-window`), `inactive` (subtle title and controls, `--elev-window-inactive`), `maximised` (no radius, edge or shadow) | Title bar with minimise, maximise, close |
| Window controls | `.ds-window-ctls`, `.ds-window-ctl` | Right-hand order Minimaliseren, Maximaliseren (Vorige grootte when maximised), Sluiten; hover, pressed, inner focus ring; Sluiten carries a tooltip "Sluiten" with the keycap "Alt+F4" | Buttons on the right |
| Orbit | `.ds-orbit`, `.ds-orbit-sep` | Capsule; separator before third-party apps (CONCEPTS.md grafts); accessible name "Orbit, dock" (NAMING-AND-IA.md §4, S0 D4) | Orbit (dock) |
| Orbit item | `.ds-taskitem`, `.ds-taskitem-badge` | `data-state` `running` (short bar), `active` (blade on Orbit's top edge + bar, `aria-current`), `attention` (wide bar + numbered keycap badge; the count is in the accessible name) | Pinned and open apps with indicator |

Not in ds-1.0 (tokens exist for some): launcher (`--material-launch*`),
search results, the Alt+Tab switcher, system tray and clock, notification
centre, quick settings, desktop icons, window move, resize and snap handles.
These are open item O-K1.

Names: NAMING-AND-IA.md §1 keeps DESKTOP-SHELL.md §7 (Horizon, Orbit, Zenit,
Nebula, Signaal, Kern, Haven) binding until the project owner decides P-NAMES-1
(§8), while the design system on this branch uses the proposed names (Menu,
Zoeken, Bestanden, Instellingen). ds-1.0 token names use "Kern" for the v2
trusted-surface material; in DESKTOP-SHELL.md §7 "Kern" is the Settings app
(open item O-N1).

## 10. Component states

States are real pseudo-classes and ARIA attributes. `data-demo="hover|pressed|focus"`
forces a state on inert gallery specimens only. "—" means the component has no
such state in `components.css`; missing states that a product use would need
are listed in O-S1. Keyboard behaviour for listbox, tabs, menu, dialogs and the
tooltip is implemented by `ui/design/system/gallery.js` (simulation), not by
`components.css`.

**Focus ring (all interactive components).** `outline: 2px solid var(--color-focus)`,
offset `2px` outside for `.ds-btn`, `.ds-input`, `.ds-switch`, checkbox and
radio inputs and `.ds-tabpanel`; offset `-2px` inside for `.ds-row`,
`.ds-menu-item`, `.ds-tab`, `.ds-window-ctl` and `.ds-taskitem`. No ring on
pointer focus (`:focus:not(:focus-visible)`). In the contrast theme the ring is
`#FFE34D`, 1.28:1 against white, so two rules keep a 1 px black or `#333333`
gap between the ring and white marks: `.ds-tab` takes offset `-5px` (clear of
the 3 px selected underline and the white tab-list line), and a focused row's
blade moves to `left: 3px`, inside the ring (§13, O-K2).

**Hidden.** The base rule `[hidden] { display: none !important; }` in
`components.css` keeps the HTML `hidden` attribute effective on every `.ds-`
class that sets `display`. Before repair round 2 only `.ds-menu`,
`.ds-tabpanel` and `.ds-toast` had a guard, so the hidden "Melding opnieuw
tonen" `.ds-btn` rendered and took a Tab stop in each theme panel.

**Anatomy (part classes).** Every `.ds-` class in `components.css` is listed
here; the state tables below use the root names.

| Component | Root | Parts |
|---|---|---|
| Type role | — | `.ds-t-caption`, `.ds-t-body`, `.ds-t-body-dense`, `.ds-t-body-strong`, `.ds-t-reading`, `.ds-t-title`, `.ds-t-display` (§6) |
| Utilities | — | `.ds-num` (tabular digits on a numeral span), `.ds-muted` (`--color-text-muted`), `.ds-vh` (visually hidden), `.ds-status-marker` (square error marker) |
| Button | `.ds-btn` | `.ds-spinner` (busy) |
| Field | `.ds-field` | `.ds-field-label`, `.ds-input`, `.ds-input-wrap` (positions the leading search icon), `.ds-field-hint`, `.ds-field-error` |
| Switch | `.ds-switch` | `.ds-switch-track` (the 44 × 24 track; decorative, `aria-hidden`) |
| Checkbox, radio | `.ds-check`, `.ds-radio` | `.ds-fieldset` (group with legend) |
| List | `.ds-list` | `.ds-list-head` (caption column header, `aria-hidden`), `.ds-row`, `.ds-row-name`, `.ds-row-meta` (size, tabular), `.ds-row-date` |
| Place | `.ds-place` | `.ds-node` (shape: dot, ring with dot, open ring) |
| Privacy instrument | `.ds-privacy` | `.ds-privacy-track`, `.ds-privacy-col`, `.ds-privacy-lbl`, `.ds-privacy-text` |
| Permission prompt | `.ds-ask` | `.ds-ask-band`, `.ds-ask-body`, `.ds-ask-title`, `.ds-ask-bond`, `.ds-ask-row`, `.ds-ask-app`, `.ds-ask-node`, `.ds-ask-dur`, `.ds-ask-warn`, `.ds-ask-choices` |
| Window | `.ds-window` | `.ds-window-bar`, `.ds-window-title`, `.ds-window-ctls`, `.ds-window-ctl`, `.ds-window-body` |
| Orbit | `.ds-orbit` | `.ds-taskitem`, `.ds-taskitem-badge`, `.ds-orbit-sep` |
| Menu | `.ds-menu-anchor`, `.ds-menu` | `.ds-menu-item`, `.ds-menu-sep`, `.ds-menu-sub`, `.ds-menu-why` (reason line of an unavailable item), `.ds-chev-down` (trigger chevron) |
| Tabs | `.ds-tabs` | `.ds-tablist`, `.ds-tab`, `.ds-tabpanel` |
| Toast | `.ds-toast` | `.ds-toast-title`, `.ds-toast-body`, `.ds-toast-time`, `.ds-toast-actions` |
| Banner | `.ds-banner` | — |
| Dialog | `.ds-dialog`, `.ds-scrim` | `.ds-dialog-body`, `.ds-dialog-title`, `.ds-dialog-text`, `.ds-dialog-actions` |
| Progress | `.ds-progress-field` | `.ds-progress-label`, `.ds-progress`, `.ds-progress-bar` |
| Empty, loading, error | `.ds-empty`, `.ds-skeleton` | `.ds-empty-title`, `.ds-empty-hint`, `.ds-empty-actions`; `.ds-skeleton-row`, `.ds-skeleton-line` |
| Tooltip | `.ds-tip-anchor` | `.ds-tip` |
| Keyboard key | `.ds-kbd` | `.ds-kbd-combo` |

**Density and neutral surface.** `data-density="compact"` and
`data-surface="neutral"` (S0 D2, D3; values PROVISIONAL) change tokens; the only
component rule is `[data-density="compact"] .ds-place` (badge 20 px high). The gallery shows both per theme: compact buttons,
field, switch, check, radio, tabs and rows render 24 px high (targets at least
24 × 24; window controls keep 40 × 32), and the neutral panel renders a field,
secondary buttons (rest, hover, disabled), rows (selected, hover, rest) and a
progress bar on a raised surface, with a focusable canvas on
`--color-surface-surround`. Inside the neutral host every computed colour is
achromatic except place colours (135 elements, 0 tinted; `build/design-evidence/ds-1.0/repair-1/measure.json`).

### Button `.ds-btn` (`.ds-spinner`)

| State | Treatment |
|---|---|
| Default | `data-variant` `primary` (light fill in dark, dark fill in light), `secondary` (`--color-button-secondary-bg` + 1 px `--color-line-strong`), `quiet` or none (transparent), `danger` (2 px `--color-danger` edge and text, trash icon; no red, S0 D1). `data-size="large"` 44 px; `data-shape="icon"` square. |
| Hover | Primary: `--color-button-primary-bg-hover`; others: `--color-hover-strong` |
| Pressed (`:active`, `aria-pressed="true"`) | 1 px down; non-primary: `--color-selection` fill and `--color-text-muted` edge; primary: inset 1 px edge |
| Focus-visible | Outer ring |
| Disabled (`:disabled`, `aria-disabled="true"`) | Transparent, 1 px dashed `--color-line-strong`, `--color-text-disabled`; hover and pressed suppressed; no opacity |
| Loading (`aria-busy="true"`) | Spinner arc shown, pointer events off; static arc under reduced motion |
| Selected, error, empty | — |

### Text and search field `.ds-field`, `.ds-input`

| State | Treatment |
|---|---|
| Default | `--color-surface-inset` fill, 1 px `--color-line-strong`; label `.ds-field-label`, hint `.ds-field-hint`; placeholder `--color-text-subtle` |
| Hover | Edge `--color-text-muted` |
| Focus-visible | Outer ring |
| Error (`aria-invalid="true"`) | 2 px `--color-danger` edge; `.ds-field-error` with square marker, warn icon and message |
| Disabled | Dashed edge, transparent, `--color-text-disabled` |
| Search variant | `.ds-field[data-variant="search"]`: pill radius, leading search icon |
| Pressed, selected, loading, empty | — |

### Switch `.ds-switch` (`role="switch"`)

| State | Treatment |
|---|---|
| Off / on (`aria-checked`) | Outlined track with knob left / filled `--color-text` track, knob right (shape and position, not colour) |
| Hover | 4 px `--color-hover-strong` halo |
| Focus-visible | Outer ring |
| Disabled | Dashed `--color-line-strong` track, `--color-text-disabled` label |
| Pressed, error, loading | — |

### Checkbox and radio `.ds-check`, `.ds-radio`, `.ds-fieldset`

| State | Treatment |
|---|---|
| Unchecked | 20 px, 2 px `--color-text-muted` edge; checkbox rounded square (`--radius-keycap`, distinct from the error square), radio open ring |
| Checked (`:checked`) | Checkbox: filled with tick (`::after`); radio: filled disc with knocked-out centre (distinct from the NAS ring-with-dot) |
| Hover | 4 px halo |
| Focus-visible | Outer ring on the input |
| Disabled | Dashed edge, `--color-text-disabled` label |
| Indeterminate, pressed, error | — |

### List row `.ds-list`, `.ds-row` (`role="option"` in a listbox)

| State | Treatment |
|---|---|
| Default | Icon, name, place badge, size and date (`.ds-num` tabular) |
| Hover | `--color-hover` |
| Selected (`aria-selected="true"`) | `--color-selection` fill, name weight 600, tapered `--color-blade` grows from the row centre (`--motion-duration-selection`); contrast adds a 2 px white inset edge |
| Focus-visible | Inner ring |
| Single pointer (D9) | A plain click selects one row |
| Pressed, disabled, error, loading | — |

### Place badge `.ds-place`, node `.ds-node`

| State | Treatment |
|---|---|
| `data-place` `device` / `nas` / `internet` | Node `data-shape` dot / ring with dot / open ring, icon with place-coloured particle, text (gallery: "Dit apparaat", "Thuis-NAS", "Internet") |
| Node `data-state="off"` | Outline only, `--color-text-subtle` |
| Node `data-state="ask"` | 2 px ring plus a pulsing halo (`--motion-duration-halo`); static in contrast and under reduced motion |
| Contrast | Device dot gets a black and white halo |
| Not focusable (static label) | |

### Privacy instrument `.ds-privacy` (content and rules: §11)

| State | Treatment |
|---|---|
| Root `data-state="local"` / `"internet"` | No root rule; expressed by the columns: only Dit apparaat lit / all three lit with lit connector blades |
| Root `data-state="ask"` | Edge `--place-internet-line`; internet column `ask` with node halo; 2 px edge in contrast. The edge is a supplementary cue: at 50 % alpha it reaches 2.11:1 to 2.35:1 against the adjacent fills in dark and light (below 3:1; contrast theme `#FF8A80` on black, 9.20:1). The text "Webvenster vraagt internet", the `role="status"` label and the haloed internet node are the required indicators; the edge must never be the only cue. |
| Column `data-state="lit"` / `"ask"` / `"off"` | Label weight 600 and `--color-text`, place-coloured particle / same plus the asking node / outline node in `--color-text-subtle` |
| Hover, pressed, focus, disabled, error | — (not focusable, `role="status"`) |

### Permission prompt `.ds-ask` (content and rules: §11)

| State | Treatment |
|---|---|
| Shown | Kern material in the opposite theme, rim via `padding-box`/`border-box` layers, `--elev-trusted`; rises over `--motion-duration-ask` (none under reduced motion or in contrast) |
| Choices | Three `.ds-btn` `data-variant="secondary"` `data-size="large"`, full width, stacked; no primary choice (§11.1); hover, pressed, focus and disabled states are the button's. Duration lines `.ds-ask-terms` (caption) are each choice's `aria-describedby` |
| Title | `tabindex="-1"`, initial focus in the dialog (§11.1); 2 px focus ring. The gallery sets `autofocus` on the title before `showModal()`, so the dialog's own focusing step lands on the title and not on the scrollable body (C1 repair-2) |
| Modal copy | One `alertdialog`: the `<dialog>` carries `role="alertdialog"`, `aria-labelledby` (title) and `aria-describedby` (internet notice); the inner `.ds-ask` gets `role="none"` without its own name (repair-1; inline specimens keep `role="alertdialog"` on `.ds-ask`) |
| Modal fit (C1 repair-2) | The `<dialog>` keeps the UA `max-height` and `overflow: visible` (the Kern shadow is not clipped) and is a flex column while `[open]`; `.ds-ask` is a flex column with `overflow: clip` (the scrollbar stays inside the rounded edge); `.ds-ask-body` scrolls (`overflow-y: auto`, `overscroll-behavior: contain`, `scroll-padding-block: var(--space-2)`, `scrollbar-color: var(--material-kern-tx2) var(--material-kern-bg)`) under the fixed band, so the "Vraag van het systeem" band stays in view while Tab scrolls a choice into view. Inline specimens are not scoped and keep their full height |

### App grant row `.ds-grants`, `.ds-grant` (content and rules: §11.1)

| State | Treatment |
|---|---|
| Rest | `--color-surface-raised` fill, 1 px `--color-line` edge (2 px in contrast), app tile `.ds-grant-app`, name `.ds-grant-name` "<App> · <bron>" 15/22 600, meta `.ds-grant-meta` "<recht> · <reikwijdte>" 15/22 400 `--color-text-muted`; list at most 760 px wide so Intrekken stays near the row's content |
| Reach (repair-1, design proposal for UX-E) | `.ds-grant-reach` line: a `.ds-place` badge for where the granted access reaches (node shape + icon + text): "Internet · <bestemming>" for a sending or connecting right, else one badge per place where the resource is kept (Dit apparaat, Thuis-NAS; a collection takes the places of its files) |
| Duration (repair-1, design proposal for UX-E) | `.ds-grant-until[data-until]` chip, neutral colours: `close` = 1 px dashed `--color-line-strong` edge + `klok` + "Tot je <App> sluit"; `always` = 2 px solid `--color-line-strong` edge + `altijd` (closed loop; `lock` until C1 repair-2, now reserved for the Kern band) + "Altijd, tot je het intrekt". Shape + icon + text; text 14.3:1 (dark), 14.5:1 (light), 21:1 (contrast); edge 4.28:1, 3.74:1, 21:1 over the row |
| Intrekken | Secondary `.ds-btn`; its hover, pressed and focus states are the button's; accessible name "Intrekken: <App>, <bron>, <recht>, <reikwijdte>" from all four key fields, <bron> with "naar <bestemming>" for a sending right (§11.1, C1 repair-2) |
| Removed | Row leaves the list; focus moves to the next Intrekken, else the previous one, else `.ds-grant-empty` (`tabindex="-1"`, focus ring); `.ds-live` announces the removal |
| Empty | `.ds-grant-empty`: "Geen blijvende machtigingen. Apps vragen het je wanneer ze iets nodig hebben." |
| Loading, error, disabled | — (not designed; UX-E) |
| Contrast | 2 px white edge |

### Window `.ds-window`, `.ds-window-ctl`

| State | Treatment |
|---|---|
| `data-state="active"` / `"maximised"` | Top light blade (`::before`), `--elev-window` / no radius, edge or shadow |
| `data-state="inactive"` | Title, controls and title icon in `--color-text-subtle`; `--elev-window-inactive` |
| Control hover / pressed | `--color-hover-strong` and `--color-text` / `--color-selection` |
| Control focus-visible | Inner ring |
| Control disabled | — |

### Orbit item `.ds-taskitem`

| State | Treatment |
|---|---|
| `data-state="running"` / `"active"` / `"attention"` | 12 × 3 px bar in `--color-text-muted` / bar plus 36 px blade above the item / 24 px bar in `--color-text` plus `.ds-taskitem-badge` |
| Hover / pressed | `--color-hover-strong` / `--color-selection` |
| Focus-visible | Inner ring |
| Disabled | — |

### Menu `.ds-menu-anchor`, `.ds-menu`, `.ds-menu-item`

| State | Treatment |
|---|---|
| Trigger | `aria-haspopup="menu"`, `aria-expanded`; `.ds-chev-down` |
| Open | Rises 12 px over `--motion-duration-base`; none under reduced motion or contrast; `hidden` when closed |
| Item hover / focus-visible | `--color-hover-strong` + inner ring |
| Item disabled (`aria-disabled="true"`) | `--color-text-disabled`, no hover fill, visible reason line `.ds-menu-why` |
| Separator, shortcut, submenu marker | `.ds-menu-sep`; `.ds-kbd` hints; `.ds-menu-sub` chevron as a static visual specimen. The item carries no `aria-haspopup` or `aria-expanded`, because no submenu opens; ArrowRight moves to the next item. Submenus are not specified (O-S4). |
| Pressed, checked items | — |

### Tabs `.ds-tabs`, `.ds-tablist`, `.ds-tab`, `.ds-tabpanel`

| State | Treatment |
|---|---|
| Default / hover | `--color-text-muted` / `--color-hover` fill and `--color-text` |
| Selected (`aria-selected="true"`) | Weight 600 + 2 px `--color-blade` underline (3 px in contrast) |
| Focus-visible | Inner ring on the tab; outer ring on the panel |
| Pressed, disabled | — |

### Toast `.ds-toast`

| State | Treatment |
|---|---|
| Shown (`role="status"`) | Icon, title, body, time, actions ("Ongedaan maken" quiet, close icon); rises over `--motion-duration-base` |
| Hidden | `hidden`; the gallery offers "Melding opnieuw tonen" and moves focus to it |

### Banner `.ds-banner`

| `data-tone` | Treatment |
|---|---|
| `info` | `--color-line` edge, `dot` icon, word "Info." |
| `success` | Base edge (`--color-line-strong`), `check` icon, word "Gelukt." (its rule repeats the base edge style) |
| `warning` | No dedicated rule: base edge, `warn` icon, word "Let op." |
| `error` | 2 px `--color-danger` edge, square marker, `warn` icon, word "Fout." |

### Dialog `.ds-dialog`, `.ds-scrim`

| State | Treatment |
|---|---|
| Open (native `<dialog>` `showModal()`) | `::backdrop` `--color-scrim`; focus starts on the safest action (Annuleren); rises over `--motion-duration-ask` |
| Static specimen | `.ds-scrim` over an inert stage |
| `data-variant="ask"` | Transparent shell hosting a `.ds-ask`; the body scrolls inside the viewport under a fixed band (Permission prompt, Modal fit) |

### Progress `.ds-progress-field`, `.ds-progress`, `.ds-progress-bar`

| State | Treatment |
|---|---|
| Determinate | `role="progressbar"`, `aria-valuenow`, width from `--ds-value`, tabular percentage |
| Indeterminate (`data-state="indeterminate"`) | Sliding segment; under reduced motion a static full striped bar with the visible word "Bezig" |

### Empty, loading and error `.ds-empty`, `.ds-skeleton`

| State | Treatment |
|---|---|
| Empty | `nofiles` icon, `.ds-empty-title`, `.ds-empty-hint`, action |
| Loading | `.ds-skeleton` `role="status"` `aria-busy="true"`, visually hidden text (`.ds-vh`); shimmer only when motion is allowed; dashed outlines in contrast |
| Error (`data-tone="error"`, `role="alert"`) | 2 px `--color-danger` edge, square marker, retry (primary) and a quiet action |

### Tooltip `.ds-tip-anchor`, `.ds-tip` (`role="tooltip"`)

| State | Treatment |
|---|---|
| Hidden / shown | Shown on `:hover` and `:focus-within` of the anchor; inverted (`--color-text` fill, `--color-surface-inset` text); no fade in contrast |
| Dismissed | Escape sets `data-dismissed` until focus or pointer leaves |
| Static | `data-state="open"` |

### Keyboard key `.ds-kbd`, `.ds-kbd-combo`

Caption 13/18, 24 px high, keycap radius, 1 px `--color-text-subtle` inset
edge, `--color-text-muted` text. Not focusable.

## 11. Trust indicators

| Indicator | Class | Content | Behaviour |
|---|---|---|---|
| Place badge | `.ds-place` | Node shape + icon + storage label (NAMING-AND-IA.md §6) | Static; inside trust surfaces and in file rows |
| Privacy instrument | `.ds-privacy` (`.ds-privacy-track`, `-col`, `-lbl`, `-text`) | Three columns Dit apparaat, Thuis-NAS, Internet; each column `data-state` `lit`, `ask` or `off`; connectors are light blades between lit columns | `role="status"` with a full sentence `aria-label`. Root `data-state`: `local`, `ask` (internet-place edge `--place-internet-line`, a supplementary cue under 3:1 in dark and light, §10; halo on the internet node) and `internet` (all lit). Only `ask` has a root rule; `internet` is derived, v2 has no such state. |
| Permission prompt | `.ds-ask` (`-band`, `-body`, `-title`, `-bond`, `-row`, `-app`, `-node`, `-dur`, `-warn`, `-choices`) | Band "Vraag van het systeem · getoond boven alle apps"; title "<App> wil <wat> <actie>"; bonded rows App, Wat, Waarheen, Hoe lang; internet notice "Dit programma kan de toegestane inhoud naar een internetdienst sturen." | `role="alertdialog"` (in the modal on the `<dialog>`, §10), `data-kind` `ongoing` or `action`; three equal choices per kind (§11.1); initial focus on the title; Escape answers deny; the result is reported in a toast |
| App grant row | `.ds-grants`, `.ds-grant` (`-app`, `-name`, `-meta`, `-reach`, `-until`, `-empty`), `.ds-place`, `.ds-live` | App, resource, right, scope (with its destination), reach badge and duration chip; "Intrekken" | One revocable standing grant; removal announced in a polite `role="status"` region (§11.1) |

Rules and their sources:

- **Drawn only by the shell.** DESKTOP-SHELL.md §1: permission dialogs are drawn by the trusted shell in a layer that apps cannot draw over, and §4 asks for permission dialogs stating who, what and for how long. DESKTOP-SHELL.md §3 lists the protected system layer as **missing** natively, so this is a requirement, not a property of any build. In the browser the `.ds-` classes are available to any page; nothing in ds-1.0 enforces exclusivity.
- **Opposite theme and opacity.** Kern takes the opposite theme (MF1) and is always opaque; a scrim dims apps while a trusted question is open (CONCEPTS.md grafts).
- **Not proof of authenticity.** An app can imitate pixels; passwords, PINs and administrator passwords are entered only on the session-service surface (DESKTOP-SHELL.md §1; BRAND.md §5).
- **Separate, visible, revocable permissions.** LOCAL-PRIVACY-CONTRACT.md §4: installation grants nothing; LAN and internet are granted separately; combined permissions are displayed; an app allowed to read a file and contact a server may send that content. The prompt's internet notice states exactly this boundary.
- **Necessary decision versus explanation.** LOCAL-PRIVACY-CONTRACT.md §4.1: explanations may be turned off; "Privacyuitleg uit" never means "alles toestaan", and a valid grant need not be asked again for each normal repetition.
- **Location honesty.** LOCAL-PRIVACY-CONTRACT.md §1: an unreachable storage location is visibly paused and files are not forwarded elsewhere; the gallery's warning banner and error empty state use the NAMING-AND-IA.md §6 sentence, which expresses this behaviour without stating the pause and the non-forwarding explicitly (O-N3).

The Kern band shows the `lock` icon where BRAND.md §5 places the compact mark
(O-B1).

### 11.1 Permission model (owner decision of 14 November 2026, task UX-D-C1)

Source: three project-owner messages of 14 November 2026 (verbatim in the
coordinator checkpoint `owner-decision-uxd-permissions-20261114.md`); message 3
is the final rule for prompt labels. This is a UX decision. The prompts, the
grant list and revocation below are design and browser simulation
(`ui/design/prototype/contract.js`, the gallery); nothing here changes native
code, the kernel, `docs/implementation/**` or `docs/LOCAL-PRIVACY-CONTRACT.md`.

**Interim presentation.** Task UX-C (owner ambition on informed consent of
14 November 2026) replaces the prompt's presentation. The `.ds-ask` dialog in
ds-1.0 is a minimal, neutral carrier for the model, labels, contract, accessible
names, interactions and revocation defined here; it is not the final consent
design.

**Superseded by the owner decision of 15 November 2026.** The project owner
replaced the prompt choices below with two primary choices, "Toestaan" and
"Weigeren", plus an optional remember control (owner wording "Keuze
onthouden") that is off by default and applies to both choices. Without
remembering, an ongoing permission lasts until the app closes and a single
action is allowed for that action only; a denial without remembering records
nothing lasting. With remembering, the allow or deny decision applies to
future identical requests, keyed on app, right, data, destination and scope.
Task UX-C researches and verifies this model before it replaces the
description below. Until then the gallery and `contract.js` show the
14 November model, and its verification record
(`docs/verification/ds-1.0-c1-20261115/`) is not evidence for the new
model. Native persistent grants and remembered denials require an ADR first.

**Two prompt kinds, three visible choices each.**

| Kind | Meaning | Choices (`data-choice`, label) |
|---|---|---|
| Ongoing app permission (`data-kind="ongoing"`) | Access that lasts while the app runs: internet, LAN, a named NAS dataset, a folder, camera, microphone | `untilClose` "Toestaan tot afsluiten" · `always` "Altijd toestaan" · `deny` "Weigeren" |
| Single action (`data-kind="action"`) | One discrete operation whose permission lapses once carried out: exporting or sharing one file, one sensitive system operation | `once` "Eenmalig toestaan" · `always` "Altijd toestaan" · `deny` "Weigeren" |

"Eenmalig toestaan" never appears on an ongoing prompt, and different
authorisations are never merged into one choice. Labels and durations live
once in `ui/design/prototype/strings.js` (`trusted.choices`,
`trusted.duration`, `trusted.result`, `trusted.grant`); the gallery writes the
button labels and duration lines from there.

**Durations (each defined once).**

| Choice | Validity | Stored as |
|---|---|---|
| Toestaan tot afsluiten | Ends when the app closes, so also at logout and system restart (the owner's label; whether "afsluiten" reads as shutdown is open, O-G12) | Standing grant, `until: 'close'` |
| Eenmalig toestaan | Ends when the single action completes, fails or is cancelled | Never stored |
| Altijd toestaan | The same key is not asked again; survives closing and reopening the app and, as a future requirement, a system restart, until the user revokes it | Standing grant, `until: 'always'` |
| Weigeren (also Escape or closing the prompt) | Denies this request only | Nothing stored; the app may ask again |

The prompt text for "Altijd toestaan" says "ook als je <app> opnieuw opent"
and does not mention restart, because the prototype contract does not simulate
a restart (a reload loses all state) and no native store exists. The string
adds restart only when native persistence exists (O-G1).

**Grant key, no repeated consent, no broadening.** A standing grant is keyed
on four fields: app + resource + capability (right) + scope. The scope
includes the destination of a right that moves data off the device: "een
kopie sturen" (`send-copy`) of Tuinplan to tuinplanner.example and to
andere-dienst.example are two keys, and `ask()` rejects a `send-copy` request
without a destination (`to`). For "verbinding maken" (`connect`) the resource
is the destination itself (the domain). `destination()` in `contract.js` is
the one helper that names it for the key, the grant row and the texts. Treating
the destination as part of the owner's "reikwijdte" is an interpretation for
owner confirmation (O-G9); a verifier found that the earlier key without it let
a grant for one destination cover every destination (repair-1). A request whose
exact key holds a valid grant (until close while the app runs, or always)
never prompts again (`ask()` resolves without calling the prompt). A request
with another key (other resource, broader right, wider scope, other
destination) prompts again;
an existing grant never broadens. The prototype matches the exact key only
(no ordering such as "lezen en wijzigen covers lezen"; O-G3).

**Per-request check.** A grant suppresses the prompt only. The policy service
still checks capability, object scope and authorisation on every request
(`docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md`, around line 220).
Grounding: `docs/LOCAL-PRIVACY-CONTRACT.md` §4 (separate, visible, revocable
permissions; around line 117) and `docs/DESIGN-AMBITION.md` line 22 (repeated
granting is not the desired default).

**Contract (`ui/design/prototype/contract.js`).** `ask(request)` takes
`{ kind: 'ongoing' | 'action', app, resource, right, scope, to? }` (`to` is
required for `send-copy`) and, when it prompts, resolves to
`'untilClose' | 'always' | 'deny'` (ongoing) or `'once' | 'always' | 'deny'`
(action); any other answer, including Escape, becomes `'deny'`. A covered
request resolves without a prompt to the stored duration, `'untilClose'` or
`'always'`, whatever its kind: kind is not part of the key, so an ongoing grant
with the same key also covers an action request. Callers treat every result
except `'deny'` as permission to proceed. `setPrompt(fn)`
attaches the prompt UI. `grants.all()`, `grants.covering(request)`,
`grants.revoke(key)` and `grants.appClosed(app)` work per four-field key;
`appClosed` ends that app's `close` grants. Data seeds in
`ui/design/prototype/data.js` (`{app, rights, until}` on a file) are
normalised to the key on load. Contract test: 48 assertions pass
(`build/design-evidence/ds-1.0/c1/repair-1/contract-test.mjs` and `.txt`;
derived from the verifier's test, with destinations, revoking one of two
destinations and the covered-by-kind result added). The verifier's original
test (`verify-0/contract-test.mjs`) now stops at its `send-copy` request
without a destination, which `ask()` rejects by design.

**Initial focus (reviewable design choice).** On open, focus goes to the
prompt title (`tabindex="-1"`, `data-initial-focus`), not to any choice.

| Option | Stray Enter or Space at open | Screen reader on open | Nudging |
|---|---|---|---|
| Title (chosen) | Does nothing | Expected (no screen-reader test, §19): the title, then the bonded rows in order; `aria-describedby` adds the internet notice | No choice is marked as expected |
| Weigeren | Denies the request the user may have wanted | Expected: the button name and its description only | Mild nudge to refuse; the ring marks one choice as the expected one |
| First allow choice | Grants access: excluded by the invariants | Expected: button name only | Nudge to allow: excluded |

Measured effort from the title (real keys): first choice Tab + Enter, "Altijd
toestaan" Tab Tab + Enter, Weigeren Tab Tab Tab + Enter or Escape alone. The
order follows the owner's list; effort is therefore not identical, and
Weigeren stays the cheapest through Escape.

Sources (accessed 14 November 2026): the WAI-ARIA APG Dialog (Modal) pattern,
<https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/>, says focus generally goes to
the first focusable element, but advises focusing a static element at the
start of the content (`tabindex="-1"`) when the content has structure that
would be hard to follow "as a single unbroken string", and the least
destructive action for steps that are not easily reversible. A prompt grant is
reversible (revocable in Appmachtigingen), and the prompt is structured
(rows App, Wat, Waarheen, Hoe lang), so the title fits better than Weigeren.
The APG Alert and Message Dialogs pattern,
<https://www.w3.org/WAI/ARIA/apg/patterns/alertdialog/>, refers to the modal
dialog pattern for its keyboard interaction.

Invariants (checked with real key presses, §15): no single keystroke at open
grants a standing permission; "Altijd toestaan" is never preselected, never the
emphasised choice and never triggered by Enter on open; Escape or closing the
prompt equals Weigeren and stores nothing; Tab from the title reaches the
choices in their visual order.

**Equal weight.** The three choices use the same button variant
(`secondary`), size (`large`) and full width, stacked in the owner's order
(narrow grant, broad grant, deny). No choice is primary, Weigeren included.
Each has a visible duration line, referenced by `aria-describedby`. Stacking
keeps the long Dutch labels whole (measured on the inline specimens: 0
clipped buttons, equal 510 x 44 px, 1018 x 88 px at 200 %). The modal prompt
did not fit at 200 % until C1 repair-2 (verify-1 step 10: "Altijd toestaan"
and Weigeren below the viewport); its body now scrolls under the fixed band.
Measured with real Tab presses at `<html data-scale="2">` in 1440 x 1000 and
960 x 667 windows, dark, light and contrast, ongoing prompt in all six and
the action prompt at 960 x 667 in all three themes: every choice is inside
the viewport after its Tab, the band stays in view, the last duration line is
reachable at maximum scroll and no horizontal clipping
(`c1/repair-2/kbd-real-keys.jsonl`; before the fix `kbd-pre-fix.txt`). With
focus on the title, real ArrowDown, PageDown and End scroll the body to its
maximum (960 x 667, dark page). At 100 % in the 1440 x 1000 window the body
does not scroll and the Kern shadow is unchanged; other 100 % window sizes
were not measured. Sources: the
owner asks for "eenvoudig en gelijkwaardig" choices
(`owner-ambition-informed-consent-20261114.md`). That emphasising one choice
would nudge the decision is design judgement, not a sourced finding; user
effect is unproven (§19).

**Deviation from the v2 board.** The reviewed Orbitaal v2 board shows the
buttons once / session / deny ("Eenmalig", "Deze sessie", "Weigeren") with
Weigeren as primary. ds-1.0 deliberately deviates by this owner decision: two
kinds, the labels above, no primary choice and title focus. The frozen boards
under `ui/design/concepts/**` keep their labels.

**Visibility and revocation.** Every standing grant ("Toestaan tot afsluiten"
while the app runs, and "Altijd toestaan") is listed and revocable under
Instellingen › Privacy en beveiliging › Appmachtigingen (NAMING-AND-IA.md §5).
Single-action grants lapse at once and are not listed. ds-1.0 provides the
grant row `.ds-grants` / `.ds-grant`: app tile, "<App> · <bron>", "<recht> ·
<reikwijdte>", a reach line and an "Intrekken" button whose accessible name is
"Intrekken: <App>, <bron>, <recht>, <reikwijdte>", where <bron> adds "naar
<bestemming>" for a sending right (`trusted.grant.target`). The name carries
all four key fields (C1 repair-2; before, only app and resource), so no two
standing grants share a name: for example "Intrekken: Notities, Tuinplan,
lezen, alleen dit document" and "Intrekken: Notities, Tuinplan, lezen en
wijzigen, alleen dit document" (`c1/repair-2/revoke-names-test.txt`, 6/6
pass; in the gallery the four seeded rows have four distinct names).
Revoking removes the row, moves focus to the next
row's Intrekken (else the previous one, else the empty message) and announces
"Machtiging ingetrokken: <App>, <bron>, <recht>, <reikwijdte>. <App> moet het
opnieuw vragen." in a
polite `role="status"` region (`.ds-live`) that sits outside the rows. No undo:
the next request simply prompts again. The full Appmachtigingen screen is
task UX-E.

The reach line (repair-1) answers the two questions a person judges a grant
by: where the access can reach and for how long. Reach is the existing
`.ds-place` badge (node shape + icon + text): "Internet · <bestemming>" for a
sending or connecting right, otherwise every place where the resource is
kept ("Dit apparaat", "Thuis-NAS" or both; for a collection, the places of
its files). It states reach, not a
promise that data stays local: an app with a read grant and an internet grant
may combine them (LOCAL-PRIVACY-CONTRACT.md §4), which a later Appmachtigingen
screen should show per app. Duration is a neutral chip whose shape follows
`data-until`: a dashed edge with the clock for "Tot je <App> sluit", a solid
2 px edge with the closed loop `altijd` for "Altijd, tot je het intrekt"
(C1 repair-2: the lock commonly reads as "locked, cannot change", which works
against a revocable grant, and it already marks the Kern band; the new glyph
is a ds-1.0 addition, §8, O-G13). Evidence classes:
contrast, clipping and names are measured (§19); that the row was too
conventional, that place and duration chips make grants easier to scan and
that the loop reads better than the lock are design judgement by an AI
verifier and writer, not a user finding (U06). No token value of the v2 board
changes; the chip uses existing neutral tokens.

**Design proposal for UX-E (not built; future research, O-G14).** The ds-1.0
list repeats the app on every row and gives no overview of where one app can
reach as a whole (AI design judgement, verify-1). UX-E could group grants
under one app header (tile and name once) and draw that app's combined reach
as a compact Orbitaal privacy-instrument strip: the device, NAS and internet
nodes lit by the union of its grants, built from the existing `.ds-privacy`
and `.ds-place` (shape + icon + text, colour for place only). The strip would
answer "where can this app reach" at a glance and display combined
permissions as LOCAL-PRIVACY-CONTRACT.md §4 asks. It changes the row
structure, the index-based `data-revoke` and the verified focus order after
revocation, so it is left to UX-E and needs its own keyboard and
screen-reader check. Feasibility: works now in the browser prototype;
natively it depends on the Appmachtigingen screen (UX-E) and the policy
service's grant list (O-G1).

**Persistent denial (research only; open owner decision O-G2).** No fourth
prompt choice is added.

| Source (accessed 14 November 2026) | Behaviour |
|---|---|
| Android, Request runtime permissions, <https://developer.android.com/training/permissions/requesting> | If the user denies a permission more than once, the system dialog no longer appears; the action is treated as "don't ask again". Apps must accept the preference and not nag. "Only this time" is a one-time grant for location, microphone and camera. |
| Apple HIG, Privacy, <https://developer.apple.com/design/human-interface-guidelines/privacy> | Not re-verified: the page renders client-side and its body was not retrieved in the verification run (repair-1). The writer's earlier reading (the system shows the request alert; people can update their choice in Settings) is unconfirmed, and no "ask only once" claim is cited (O-G7). |
| Chromium Blog, Introducing quieter permission UI for notifications (January 2020), <https://blog.chromium.org/2020/01/introducing-quieter-permission-ui-for.html> | Not re-verified: only the publication date (7 January 2020) was confirmed; the post body was not retrieved in the verification run. The writer's earlier reading: Chrome switches to a quieter, non-interrupting notification prompt for users who usually block such requests and for sites with very low acceptance rates (O-G7). |
| W3C Permissions, Working Draft 26 October 2026, <https://www.w3.org/TR/permissions/> | States `granted`, `denied`, `prompt`; a stored decision is reused until its lifetime ends or the user revokes it. |

Options: (A) as now: Weigeren stores nothing, every new request prompts;
simple, but a persistent app can nag. (B) Automatic quieting after repeated
denials (Android, verified by verify-1; the Chrome behaviour is not cited as
support until the Chromium post body is retrieved and quoted, O-G7): after
two denials of the same key within the app's
installation, further requests for that key do not interrupt; the app gets
`deny` and a quiet, non-modal notice offers the prompt on the user's
initiative; the denial is listed and clearable in Appmachtigingen. (C) An
explicit "Altijd weigeren": rejected, because it adds a fourth choice.
Recommendation: B, keyed on the same four fields, never on the app as a whole,
cleared by revocation in Appmachtigingen, and visible there, so a denial is as
inspectable as a grant. This needs an owner decision (O-G2) and, natively, the
same ADR as persistent grants (O-G1).

**Evidence limits.** Design and prototype simulation only. Persistence across
a system restart and native enforcement are future implementation
requirements; native capability and security contracts change only after a
separate review, and a project ADR is required before native persistent grants
(O-G1). The browser gallery enforces nothing: any page can draw `.ds-` classes.

## 12. Motion

| Token | ms | Use in ds-1.0 | Board source |
|---|---|---|---|
| `--motion-duration-fast` | 140 | Hover fills, button press, input edge, window control and task item fills, tooltip fade | B1 menu close |
| `--motion-duration-selection` | 160 | Row blade grows from the centre; switch track and knob | B1 selection |
| `--motion-duration-base` | 220 | Menu and toast rise (12 px); determinate progress width | B1 menu open |
| `--motion-duration-ask` | 240 | Permission prompt and dialog rise, no overshoot | B1 system question |
| `--motion-duration-focus` | 260 | Not used by a component (focus blade sweep) | B1 |
| `--motion-duration-slow` | 320 | Active Orbit blade transform | B1 active blade |
| `--motion-duration-boot` | 1400 | Not used by a component (boot convergence) | B1 |
| `--motion-duration-loader` | 1400 | Spinner rotation; indeterminate progress slide | v2 `.ld` |
| `--motion-duration-halo` | 2400 | Internet node halo while asking | B1 |
| `--motion-duration-loader-pulse` | 2800 | Skeleton shimmer | v2 `.ld .bind` |
| `--motion-duration-stagger` | 80 | Not used by a component (boot blade stagger) | v2 `.bt-b2` |

Easing: `--motion-ease-orbital-out` `cubic-bezier(.16, .84, .24, 1)` for every
transition and rise; `--motion-ease-halo` `cubic-bezier(.2, .7, .2, 1)`;
`--motion-ease-linear` (spinner, shimmer); `--motion-ease-in-out` (progress
slide). Motion uses transform and opacity, plus background colour, border
colour and width.

**Reduced motion** (`prefers-reduced-motion: reduce` or `[data-motion="reduce"]`):
all 11 durations become 0ms (`tokens.css`); halo, spinner, progress slide,
skeleton shimmer and the rise of menu, toast, dialog and prompt are removed;
the indeterminate bar becomes a static full striped bar with the word "Bezig";
skeleton lines become a static `--color-hover-strong` fill; the spinner stays a
static arc. The gallery exposes a switch for `data-motion`.

## 13. High contrast

`[data-theme="contrast"]` is a complete theme (opaque surfaces and fills; the decorative row, window, Orbit and privacy-connector blades keep their faded gradient ends over black, with a solid centre at 21:1; black surfaces, white
text and lines, focus `#FFE34D`). Values come from v2 `.hc` where v2 defines
them (v2 applies `class="dk hc"`); the 24 names `.hc` leaves undefined are
derived as opaque, achromatic values anchored on `HIGH_CONTRAST` in
`ui/graphics/src/theme.rs` (black `0x000000`, white `0xffffff`):

| Derived token | Value | Reason |
|---|---|---|
| `--color-selection` / `--color-hover` / `--color-hover-strong` | `#333333` / `#1A1A1A` / `#262626` | Opaque fills; selection is carried by a white edge and weight 600 because `#333333` is only 1.66:1 on black |
| `--color-scrim` | `#000000` | Trusted surfaces stand alone on black with a 2 px white edge |
| `--color-button-primary-bg` / hover / text | `#FFFFFF` / `#D9D9D9` / `#000000` | 21.00:1 and 14.87:1 |
| `--color-button-secondary-bg`, `--color-pill-bg` | `#000000` | Edge from the line token |
| `--color-silver`, Kern text, text-2, line, rim; tile fg and particle | `#FFFFFF` | Solid rim instead of the metal gradient |
| Kern bg, Kern warn bg, inspector, launcher, launch-opaque | `#000000` | Opaque |
| Kern warn line and icon, `--place-internet-line` | `#FF8A80` | v2 `.hc --red` (internet place) |
| `--material-tile-sh`, every `--elev-*` | `none` | No shadows |
| `--color-surface-surround` (repair-1) | `#000` | Equal to `--color-bg` (v2 `.hc{background:#000}`); v2 has no surround. Under `data-surface="neutral"` the surround and surfaces are all black, so the canvas region carries a 1 px `--color-line-strong` edge (S0 D3; gallery `.gal-surround`) |

Rejected anchors: native `accent_soft` cyan `0x00ffff` as selection (a
non-place hue next to NAS blue) and native muted `0xb8b8b8` (v2 `.hc` defines
`#FFFFFF` and `#F2F2F2`). Source: tokens stage decisions,
`build/design-evidence/ds-1.0/work/mapping.tsv`.

Component overrides in contrast (`components.css`): Orbit and privacy edges
2 px, Orbit blur off; selected row 2 px white inset edge; selected tab 3 px
underline; focused tab ring offset `-5px` and focused row blade at `left: 3px`
(repair round 2); device node black and white halo; ask halo static; no entrance
animation for menu, prompt, dialog and toast; no tooltip fade; skeleton lines
dashed outlines; Kern edge 2 px.

Thresholds and results: text ≥ 7:1 (lowest 11.28:1, `#F2F2F2` on
`#333333`), UI ≥ 3:1 (lowest 5.53:1, `--place-internet` on `#333333`), no
translucent colour or gradient (§4; tokens stage check 3e). Constraint: focus
`#FFE34D` is 1.28:1 next to white and 1.09:1 next to `#D9D9D9`, so every ring
needs a black separator where it would touch a white fill; the outer ring's
2 px offset over the black base provides it. Inner rings next to white marks
were measured in repair round 2 with pixel profiles across the ring at device
scale 1 (`build/design-evidence/ds-1.0/repair-2/shots/ring-base-profile.txt`, `ring-final-profile.txt`,
`junction-base-final.json`; a junction pixel is a grey or white pixel that
touches a pure ring pixel at under 3:1; it does not count yellow-white blends
such as `#FFEF9C`). Before the §10 rules the selected-tab underline and the
tab-list line touched the ring (27 junction pixels, lowest 1.26:1) and the
white row blade covered the ring's left side. After them, the selected
standard tabs Versies, Toegang and Opslag and the compact tab Versies have 0
junction pixels and a 1 px black gap above the underline; the standard
selected row's left-edge profile reads ring, 1 px `#333333`, blade (the
junction count is 0 for that row before and after, so the profile is the
evidence). v2 defines no contrast-theme focus offset or blade position
(`.hc` rules of the board); both values are repair-2 derivations. Still open (O-K2): in compact rows the
20 px place badge's white edge touches the ring (258 pixels, lowest 1.06:1,
unchanged by the fix), the compact blade's rounded end blends with the ring
corner (2 pixels), a descender in a compact tab label (Toegang) touches the
inset ring (12 pixels), and the Orbit badge was not measured.

## 14. Scaling

| Scale | Browser | Native |
|---|---|---|
| 100 % | All sizes are logical CSS px | `type-1.0` records 13r, 15r, 15sb, 22sb, 48r (with `inter-atlas`) |
| 200 % | One mechanism shared by gallery, prototype and checks: an element (normally `<html>`) with `data-scale="2"` gets `[data-scale="2"] { zoom: 2 }` (`components.css`). The gallery renders a strip at 200 % (`#scale-host`) with buttons (primary, secondary, danger), a switch, a search field, one selected list row with a place badge, place badges, keycaps, a checkbox and, since repair round 2, the privacy instrument in the ask state. Menu, tabs, permission prompt, dialog, toast, window chrome and the Orbit are not in the strip (§19). | Doubled geometry and the 200 % atlas records 26r, 30r, 30sb, 44sb; display 48r magnified 2× (`ui/graphics/src/text.rs`; `docs/verification/native-inter-atlas-20261114/README.md`) |
| 125 %, 150 % | — | Open (TYPOGRAPHY.md §8.6) |

CSS `zoom` is not a device scale factor or a reflow. The components stage
checked one strip (computed zoom 2; a 32 px button measured 64 px tall).
Repair round 2 measured the strip with the instrument added: computed zoom 2,
83 rendered elements (SVG parts included), `#scale-host` scroll width equal to
its 680 px client width, no element beyond the strip's edges and no clipped
scroller (`build/design-evidence/ds-1.0/repair-2/flows.json`, `build/design-evidence/ds-1.0/repair-2/shots/scale-strip.png`).
UX-D-C1 added both permission prompts and the grant rows to the strip: stacked
choices 1018 x 88 px each, none clipped inside the prompt
(`build/design-evidence/ds-1.0/c1/writer/harness-result.json`). Those are the
inline specimens. The modal prompt (`#dlg-ask`, top layer) spilled out of the
viewport at 200 % (verify-1 step 10; reproduced in
`c1/repair-2/kbd-pre-fix.txt`: "Altijd toestaan" at 955–1043 and Weigeren at
1059–1147 in a 1000 px viewport). Since C1 repair-2 its body scrolls under a
fixed band (§10 Permission prompt, Modal fit): with `<html data-scale="2">`,
real Tab presses and windows of 1440 x 1000 and 960 x 667 in dark, light and
contrast, every choice is in view after its Tab, the band stays in view and
nothing clips horizontally (`c1/repair-2/kbd-real-keys.jsonl`). The page
behind still scrolls horizontally at 200 % from pre-existing gallery
elements (verify-1 step 13), not from the prompt or the grant rows.

## 15. Keyboard rules

Default shortcuts (copied from DESKTOP-SHELL.md §2, "Default keyboard
shortcuts"):

| Key | Action |
|---|---|
| Ctrl+C / Ctrl+X / Ctrl+V | Copy / cut / paste |
| Ctrl+Z / Ctrl+Y | Undo / redo |
| Ctrl+S, Ctrl+A, Ctrl+F, Ctrl+P | Save, select all, find, print |
| Alt+Tab, Alt+F4 | Switch, close window |
| OS key | Zenit with Signaal |
| OS key+E / +I / +L / +D | Files / Settings / lock / show desktop |
| Ctrl+Alt+Delete | Trusted system screen (lock, switch, sign out, task manager) |
| Ctrl+Shift+Esc | Task manager |

S0 D5 adds namespaces (OS-owned keys never reach apps; Ctrl+Alt+Delete only to
the session service) and proposes Ctrl+N/O/W, Shift+Ctrl+Z and Ctrl+plus,
minus and 0 as an amendment to DESKTOP-SHELL.md (S0 open item 5); ds-1.0 does
not implement a keymap.

| Rule | ds-1.0 |
|---|---|
| Focus order | DOM order; Tab and Shift+Tab move between components |
| Groups, one tab stop each | Listbox rows and tabs use a roving `tabindex`; radios are native groups |
| Arrow keys (gallery simulation) | Listbox: ArrowUp, ArrowDown, Home, End move and select (Left and Right ignored), Space selects; tabs: ArrowLeft, ArrowRight (also Up and Down), Home, End select and focus (automatic activation); menu: ArrowDown or ArrowUp on the trigger opens on the first or last item, arrows and Home and End move; Right and Left act as Down and Up, since no item opens a submenu. Arrow movement wraps at the ends. |
| Activation | Space and Enter on native buttons, switch and menu items |
| Escape | Closes the menu and returns focus to its trigger; closes dialogs and returns focus to the opener; answers a permission prompt with deny; hides a tooltip |
| Modal focus | `showModal()` makes the page behind the dialog inert; Tab cycles through the dialog controls and the browser UI, never page content; Escape always closes it, so there is no keyboard trap. Tab out of an open menu closes it. |
| Initial focus | Permission prompt: the title (`tabindex="-1"`); Enter and Space at open do nothing; Tab reaches the choices in visual order (§11.1; real key presses in `build/design-evidence/ds-1.0/c1/writer/kbd-real-keys.txt`). Delete dialog: Annuleren (the safe choice, NAMING-AND-IA.md §6) |
| Focus ring | §10: 2 px `--color-focus`, outer or inner. Real keyboard focus (repair-2, 176 Tab stops; repair-1 counted 179, three of them the hidden "Melding opnieuw tonen" buttons that rendered before the `[hidden]` rule): minimum 4.00:1, the light ring `#171717` on the neutral surround `#777777` (S0 D3 predicts 4.00); elsewhere at least 9.83:1 (contrast theme, `#FFE34D` inside a selected row `#333333`); these ratios are computed from styles by `m_focus.js`, not pixel-sampled in repair round 2 (§13 has the contrast-theme ring profiles). The 17 stops over gradient or translucent fills (Kern prompt buttons, Orbit items) were not pixel-sampled again; their markup and focus tokens are unchanged since the verifier's pass. |
| Shortcut display | `.ds-kbd` keycaps; S0 D5 names this `.ds-keycap` (O-S2) |

Evidence: the components stage sent real key and pointer events through the
DevTools protocol in the dark panel only (`build/design-evidence/ds-1.0/components/keys.json`,
`checks.log`). Repair round 1 sent real keys in all three
themes to the new compact and neutral specimens (tabs ArrowRight and Home;
listbox ArrowDown and End; switch Space; radio ArrowRight; Tab to the canvas)
and made one forward Tab pass over the whole gallery (179 stops, every stop
`:focus-visible` with a ring; `build/design-evidence/ds-1.0/repair-1/dn-keys.json`, `build/design-evidence/ds-1.0/repair-1/tools/kbd-raw.json`).
Repair round 2 repeated the pass after the `[hidden]` rule (176 stops, every
stop `:focus-visible` with a ring, no "Melding opnieuw tonen" stop;
`build/design-evidence/ds-1.0/repair-2/tools/kbd-raw.json`) and sent real keys in all three themes to the toast
(Tab to Melding sluiten, Enter hides the toast and focuses the now rendered
"Melding opnieuw tonen", Enter shows the toast, hides the button and returns
focus to Melding sluiten) and to the Versies tooltip (Tab shows it, Escape
hides it with focus kept, leaving and re-entering shows it again;
`build/design-evidence/ds-1.0/repair-2/flows.json`). The second verifier had already seen the light tooltip hide
on Escape.

## 16. Asset and font licensing

| Asset | Licence and provenance | Source |
|---|---|---|
| Inter 4.001 (`Inter[opsz,wght].ttf`) | SIL OFL 1.1; google/fonts commit `29b4ec6d`; SHA-256 `29160a80ff49ddcab2c97711247e08b1fab27a484a329ce8b813d820dc559031`; licence `OFL-Inter.txt` `5b9321a4…` | `ui/design/fonts/FONTS.md` |
| Saira (wordmark v2) | SIL OFL 1.1; SHA-256 `9d050fc5a01c85f74c4257c207d50b55d1e40c37308c642f974a2c5003231dde`; licence `OFL-Saira.txt` `f2665d47…`; traced to outline paths at build time, no run-time dependency; used on brand surfaces only | FONTS.md; BRAND.md §4 (the Saira hash is pinned in FONTS.md) |
| BaryonOS mark | Project mark; generated deterministically by `ui/design/brand/generate_mark.py`; `LICENSING.md` ("Trademarks"): "MPL-2.0 does not grant general rights to the **Baryon OS** name, logos, or other project marks." | BRAND.md intro, §6; `LICENSING.md` ("Trademarks") |
| Icons | Drawn for the project (v2 sprite plus four ds-1.0 drawings); no third-party icon set | `icons.js` header; tokens stage report |

FONTS.md states that the OFL forbids selling the fonts by themselves and that
bundling them with BaryonOS is permitted; no conclusion beyond that is drawn
here. Native shipping of Inter still needs the `ui/native` licence and package
entries (FONTS.md; inter-atlas record open items 6 and 7).

## 17. Native mapping

Native code is not changed by ds-1.0. The plan is a **new, additive** `Palette`
const for Orbitaal in `ui/graphics/src/theme.rs`
(`docs/design/NATIVE-FEASIBILITY.md` §4 step 1). `MATERIAAL` and
`theme::wallpaper` stay untouched because `ui/native`, Veld, materiaal,
calculator and the compositor share them (NATIVE-FEASIBILITY.md §2).
`HIGH_CONTRAST` is the anchor of the contrast theme, not its equal (see the
table). The default native build still draws Open Sans; Inter is drawn only
with the cargo feature `inter-atlas`, off by default (`ui/graphics/Cargo.toml`;
`text.rs` header).

| `Palette` field / constant | Proposed ds-1.0 source (dark / light) | `HIGH_CONTRAST` today vs ds contrast |
|---|---|---|
| `desktop_start`, `desktop_end`, `glow` | No token: the v2 wallpaper is artwork; nearest is `--color-bg` for `desktop_start` | `0` / `#000` |
| `surface` | `--color-surface` | `0` / `#000` |
| `background` | `--color-bg` (`--color-surface-panel` has no field) | `0` / `#000` |
| `text` | `--color-text` | `0xffffff` / `#FFFFFF` |
| `muted` | `--color-text-muted` | `0xb8b8b8` / `#FFFFFF` (differs) |
| `accent` | `--color-focus` (no accent hue in ds-1.0) | `0xffff00` / `#FFE34D` (differs) |
| `accent_soft` | `--color-selection`, composited to an opaque value per base | `0x00ffff` / `#333333` (cyan rejected) |
| `border` | `--color-line` (one field; `--color-line-strong` has none) | `0xffffff` / `#FFFFFF` |
| `SHELL_TEXT`, `SHELL_MUTED` | `--color-text`, `--color-text-muted` over the wallpaper | — |
| `SHELL_LINE` | `--material-glass-line` or `--color-line-strong` (undecided) | — |
| `DIVIDER` | `--color-line` | — |
| `SHADOW` | Colour of `--elev-*` (black in dark; `rgba(36,54,108,·)` in light) | none |
| `HIGH_CONTRAST_DANGER` (`0xffb4b4`) | No counterpart: S0 D1 has no status hue | — |

Text roles (`ui/graphics/src/text.rs`; inter-atlas record table in
`docs/verification/native-inter-atlas-20261114/README.md`; Open Sans sizes from
NATIVE-FEASIBILITY.md §1):

| ds role | `text.rs` API | `inter-atlas` record (100 / 200 %) | Default Open Sans record |
|---|---|---|---|
| caption 13/18 | `Size::Small` | 13r / 26r | 12 regular |
| body 15/22 | `Size::Body`, `Weight::Regular` | 15r / 30r | 16 regular |
| body-dense 15/20 | as body | no 20 px line record | — |
| body-strong 15/22 | `Size::Body`, `Weight::Semibold` | 15sb / 30sb | 16 semibold |
| reading 16/26 | none | not built (16r / 32r) | — |
| title 22/28 | `Size::Title` | 22sb / 44sb | 32 semibold |
| display 48/56 | `Font::display` | 48r, magnified 2× at 200 % | 64 px record |

Icons: `ui/graphics/src/icon.rs` has 20 stroked polyline icons on a 24-unit
grid, with no curves, fills or particles. Nearest counterparts only:

| ds key | `icon.rs` | ds key | `icon.rs` |
|---|---|---|---|
| `zoeken` | `Search` | `close`, `min`, `max` | `Close`, `Minimize`, `Maximize` |
| `bestanden` | `Folder` | `back`, `fwd` | `ChevronLeft`, `ChevronRight` |
| `instellingen` | `Settings` | `check`, `plus` | `Check`, `Add` |
| `power`, `lock`, `user`, `apps` | `Power`, `Lock`, `User`, `Apps` | `internet` | `Network` (nearest) |

No native counterpart: the remaining 39 keys (aliases and the ds-1.0 addition `altijd` included), among them `device`, `nas`,
`notities`, `haven`, `versies`, `toegang`, `warn`, `trash`, `dot` and every file
kind. Native icons without a ds key: `Material`, `Key`, `Fingerprint`,
`Refresh`, `Shield`.

No native counterpart yet: place colours (no `Palette` field), materials and
gradients (no gradient primitive; per-pixel computation only), backdrop blur
(only over the static pre-blurred wallpaper, MF11), dashed edges, every motion
token (no animation clock), the reading and dense type records, the mark and
the wordmark (BRAND.md §4), and 125 % and 150 % scale (NATIVE-FEASIBILITY.md §1;
TYPOGRAPHY.md §8.6).

## 18. Versioning and change rules

| Change | Version step |
|---|---|
| Rename or remove a token, class, attribute, theme selector or icon key; change a token's role; change keyboard behaviour; a value change that drops a listed pair below its threshold | Major (`ds-2.0`) |
| Add tokens, components, variants or icons; replace PROVISIONAL values within the thresholds; reserved S0 names becoming real (ds-1.1 per S0) | Minor (`ds-1.1`) |
| Documentation fixes; value corrections that keep every pair | Patch (`ds-1.0.1`) |

Repairs made before the project owner accepts ds-1.0, such as repair rounds 1
and 2, keep the version `ds-1.0`.

Rules: `tokens.css` is regenerated, not hand-edited
(`build/design-evidence/ds-1.0/work/gen_tokens.py`, see O-P1). Exception:
repair round 1 edited it by hand because it could write only `ui/design/system/`
and its evidence folder; the generator must take over the repair-1 values before
the next value change (O-P1). Every value keeps
its v2 source or a recorded derivation; the contrast script in §4 is re-run on
every colour change; `ui/prototype/tokens.css` is never touched. Architecture
changes are recorded in `docs/DECISIONS.md`. The project owner can redirect any
decision; the design pipeline may change values only within this document's
rules.

## 19. Not verified

- No human review, no user or task trial (U06), no screen-reader or
  assistive-technology test. All checks were run by AI agents.
- No native rendering of any ds-1.0 token, icon or component; no QEMU run.
- Contrast of glass, Orbit material and the translucent launcher over real
  wallpaper pixels; §4 assumes `--color-bg` as backdrop.
- `data-density="compact"` and `data-surface="neutral"` are rendered for a
  subset of components only (§10); menu, dialog, toast, window chrome and the
  privacy instrument were not rendered in either mode.
- Keyboard sequences and modals in the light and contrast themes (repair
  round 2 exercised only the toast and tooltip paths there); Firefox and Safari (the checkbox tick uses `::after`
  on `<input>`, which Firefox does not render).
- Colour-vision deficiency simulation of the place colours.
- 200 % only as CSS `zoom` on one strip; no device scale factor; no reflow at
  320 CSS px; menu, tabs, dialog, toast, window chrome and the Orbit are not
  in the strip (both permission prompts and the grant rows are, since UX-D-C1).
  The modal permission prompt was measured at 200 % with `zoom` on `<html>`
  only since C1 repair-2 (two window sizes, three themes); the delete dialog,
  menus and toasts were not measured at 200 %.
- UX-D-C1 (permission model): the contract self-check, the gallery
  self-measure, a harness for labels, sizes, clipping, contrast and flows, real
  key presses through the chrome-devtools MCP, and renders
  (`build/design-evidence/ds-1.0/c1/writer/`). The writer could not view the
  renders (image reads were blocked in this session); a visual review is
  outstanding (O-G6). The crops use y-ranges measured in a 1425 px-wide iframe
  applied to a 1440 px render, so their edges are approximate. The two contrast-ring rules for the Orbit bar and the
  menu offset were applied without a new pixel profile.
- UX-D-C1 repair-1 (`build/design-evidence/ds-1.0/c1/repair-1/`): contract
  test (48 assertions), a writer harness with fixed 80 ms waits (the verify-0
  failure mode) and a same-task race (answer one prompt and open the next
  before the first `close` event), reach and duration chips (contrast, 24 px
  height, no clipping at 1440 px and with the list at 320 px, unique revoke
  names), real Enter, Escape, Tab, Shift+Tab and Enter-on-a-choice presses and Chrome's accessibility tree through
  the chrome-devtools MCP (`real-keys-and-a11y.txt`). Expected screen-reader
  output on open, from that tree and not from a screen reader: "Webvenster wil
  internet gebruiken, alertdialog" (once), the description "Dit programma kan
  de toegestane inhoud naar een internetdienst sturen.", then the focused
  heading; each choice reads its name and its duration line. NVDA, JAWS,
  VoiceOver and Orca were not run. The writer viewed the dark and contrast
  grant cards in screenshots; light and 200 % renders of the new rows were
  checked by measurement only.
- UX-D-C1 repair-2 (`build/design-evidence/ds-1.0/c1/repair-2/`): real
  Enter, Tab, Shift+Tab and Escape presses through the chrome-devtools MCP on
  the modal prompt at 200 % (`kbd-real-keys.jsonl`, 12 runs, all pass; one
  run repeats the ongoing prompt because the asynchronous `close` event
  returned focus to its opener, recorded as such; the first four runs predate
  the `overflow: clip` and `scrollbar-color` lines, and only the last capture
  run uses the final scrollbar track (`var(--material-kern-bg)`); neither
  change alters measured geometry. The runs' in-view flags compare each
  focused choice with the window; their logged rectangles also lie between
  the band's bottom and the dialog's bottom, and a final check with real Tab
  presses on the final CSS found every choice inside the `.ds-ask-body`
  scrollport); a 100 % check (body does
  not scroll, `--elev-trusted` shadow present, dialog `overflow: visible`);
  real Enter on an Intrekken button (four distinct names, the announcement
  names all four fields); `revoke-names-test.txt` (6/6), the verify-1 and
  repair-1 contract tests (36/36, 48/48), the verify-1 consistency check (no
  change) and the verify-1 measure harness (re-run on the final CSS and
  icons: no difference from verify-1, 0 failures, normal and reduced motion);
  the gallery self-measure (54 icon keys, none unresolved; run before the
  final particle position, the key count re-checked on the final file); real ArrowDown, PageDown and End with focus on the
  title scroll the body to its maximum. The writer viewed, in chrome-devtools
  screenshots, the 200 % prompt in all three themes with focus on Weigeren,
  the 100 % dark-page prompt, the final-CSS 200 % prompt at 960 x 667 (light
  page, End pressed: scrollbar inside the rounded edge, band and last line in
  view) and the grant chips with the final `altijd` glyph in light (100 % and
  200 %) and contrast (100 %); the dark chip was viewed only with earlier
  particle positions. Saved captures: `r2-*.png` in the same folder (the
  `-final` ones use the final CSS and icons). The `altijd` and `klok` glyphs
  were compared side by side in those screenshots, not in a pixel crop.
- Animation was not watched or timed; reduced motion was checked by computed
  styles.
- Contrast theme: the compact place badge, the compact blade end, compact tab
  descenders and the Orbit badge beside inner focus rings (O-K2).
- Repair round 1 re-ran the gallery self-measure, the computed and
  pixel-sampled contrast sweep (0 computed failures, 0 controls under 3:1; the
  pixel sampler flags 13 glyphs, Orbit tile icons and contrast-theme place
  nodes, against 9 of the same kinds in the verifier's run; not analysed further), the keyboard checks above and clipped renders
  (`build/design-evidence/ds-1.0/repair-1/`). The other rendered and keyboard results are the tokens and
  components stages'. Renders and `SHA256SUMS` in `build/design-evidence/ds-1.0/`
  and `components/` predate repair-1 and are stale for the changed tokens,
  typography and gallery.
- Repair round 2 re-ran the gallery self-measure (0 errors, 53 icon keys
  resolved, 224 targets with none under 24 px, 1 311 visible text elements
  with none outside the seven roles), the forward Tab pass, the toast and
  tooltip key paths, the contrast-ring profiles and clipped renders
  (`build/design-evidence/ds-1.0/repair-2/`). Repair-1 renders and `SHA256SUMS` are stale for the changed
  `components.css` and `index.html`.

Evidence paths cited here under `build/design-evidence/ds-1.0/` are ignored by
Git; the lead must move what is needed to `docs/verification/` (O-P2).

## 20. Open items

| ID | Item |
|---|---|
| O-C1 | Store the repair-1 S0 D3 run (`build/design-evidence/ds-1.0/repair-1/check-report.txt`, `tools/contrast-pairs.md`, `tools/tune.py`, `tools/mapping.tsv`) under `docs/verification/` (S0 D3, S0 open item 2); this round had no write access there. |
| O-C2 | Closed in repair-1: dark secondary-button edge 3.48:1 over raised, 3.91:1 over a window (§4.1). |
| O-C4 | S0 D3's twin table (`docs/design/suite/WORKSPACE-REQUIREMENTS.md`) still lists the earlier twins `#747474`, `#969696`, `#818181` and `#626262`; ds-1.0 now uses `#858585`, `#A4A4A4`, `#757575` and `#5B5B5B` (§4.1). The drift from v2 is not yet in `docs/DECISIONS.md`. |
| O-C3 | Muted versus subtle text separation fell to 1.20:1 (dark) and 1.27:1 (light) with the repair-1 values; confirm the hierarchy in a human review. |
| O-T1 | Tabular digits: `.ds-num` spans only in the browser versus every digit in TYPOGRAPHY.md §8.1 and §8.4. |
| O-S1 | Missing states: pressed on row, tab, menu item, switch, checkbox and radio; disabled on row, tab, Orbit item and window control; loading only on the button and skeleton; checkbox indeterminate; checked menu items; error on switch, checkbox and radio; no dedicated warning-banner rule. |
| O-S2 | S0 reserved names not implemented: regions (`.ds-workspace`, `.ds-menu-bar`, `.ds-toolbar`, `.ds-tool-rail`, `.ds-canvas`, `.ds-inspector`, `.ds-panel`, `.ds-panel-zone`, `.ds-timeline`, `.ds-data-grid`, `.ds-status-bar`, `.ds-document-tabs`, `.ds-command-palette`) and components (`.ds-panel-header`, `.ds-track`, `.ds-ruler`, `.ds-playhead`, `.ds-cell`, `.ds-number-field`, `.ds-layer-row`, `.ds-expander`, `.ds-export-dialog`, `.ds-object-chip`, `.ds-version-indicator`); S0 names `.ds-keycap` and `.ds-place-badge` where ds-1.0 has `.ds-kbd` and `.ds-place`. |
| O-S3 | Attributes and hooks beyond the fixed naming list: `data-kind`, `data-until`, `data-initial-focus`, `data-revoke`, `.ds-live`, `.ds-ask-terms` (UX-D-C1), `data-size`, `data-shape`, `data-demo`, `data-choice`, `--ds-c`, `--ds-value`; sub-classes such as `.ds-status-marker`, `.ds-node`, `.ds-orbit`, `.ds-list`, `.ds-input`. |
| O-S4 | Submenus: `.ds-menu-sub` is a visual marker only; opening, ARIA (`aria-haspopup`, `aria-expanded`) and keys (ArrowRight opens, ArrowLeft and Escape close) are not specified. |
| O-K1 | Launcher, search results, switcher, tray, notification centre, quick settings and window move, resize and snap are not designed as components. |
| O-K2 | UX-D-C1 applied the base verifier's two one-line fixes (`.ds-taskitem::after` bottom 4 px and `.ds-menu` 5 px offset in the contrast theme); not re-profiled. Contrast theme, partly closed in repair round 2: tabs and the standard selected row now keep a 1 px gap between the inner ring and white marks (§13); the focused row's blade moves 3 px inward, a repair-2 derivation not in v2. Open: the compact place badge edge (258 junction pixels, lowest 1.06:1), the compact blade end (2 pixels) and compact tab descenders (12 pixels) touch the ring; the Orbit badge is not measured. Compact values are PROVISIONAL (S0 D2). |
| O-K3 | `.ds-taskitem` shows app icons as bare glyphs; MF9 puts every app on the tile. |
| O-N1 | "Kern" names the trusted-surface material in ds-1.0 and the Settings app in DESKTOP-SHELL.md §7; P-NAMES-1 awaits the project owner. |
| O-N2 | Closed in UX-D-C1: prompt labels, durations and results now come from `strings.js` `trusted.*` and match NAMING-AND-IA.md §6 (§11.1). |
| O-N3 | The NAS-unreachable sentence implies but does not state "paused" and "not forwarded elsewhere" (LOCAL-PRIVACY-CONTRACT.md §1); adding "en wordt niet naar een andere plek gestuurd" needs a copy decision. |
| O-N4 | The gallery names the browser app "Webvenster" (v2 board), since C1 repair-2 from `strings.js` `apps.webvenster`; `strings.js` also keeps `apps.browser` "Browser", so the product name of the browser app is still undecided. The prompts' "what" and "done" fragments in `gallery.js` are marked gallery sample data, not `strings.js` copy (O-D1). |
| O-G1 | A project ADR is required before native persistent grants; persistence of "Altijd toestaan" across a system restart and native enforcement are future implementation requirements. Native capability and security contracts change only after a separate review. When native persistence exists, the "Altijd toestaan" duration line adds restart. |
| O-G2 | Persistent denial: owner decision on the §11.1 proposal (recommended option B, quieting after two denials of the same key, listed and clearable in Appmachtigingen). |
| O-G3 | Grant matching is exact on the four-field key; whether a broader existing grant may cover a narrower request (for example read within read and write) needs a policy-service decision. |
| O-G4 | Interpretation recorded for owner confirmation: message 3 fixes the ongoing labels and restricts "Eenmalig toestaan" to single actions; single actions therefore offer "Eenmalig toestaan", "Altijd toestaan", "Weigeren". |
| O-G5 | UX-C replaces the prompt's presentation (informed consent); the initial-focus choice, the stacked equal choices and the duration lines are reviewable inputs to UX-C, and their user effect is untested. |
| O-G6 | The UX-D-C1 writer renders (`build/design-evidence/ds-1.0/c1/writer/c1-*.png`) are stale for the grant rows. In repair-1 the writer viewed the dark and contrast grant cards (chrome-devtools screenshots); the renders `c1/repair-1/r1-gallery-*-top3300.png` and their crops were saved, not viewed (image reads are blocked in the writer session). Visual judgement of light and 200 % is pending. C1 repair-2: the writer viewed the 200 % modal prompt in all three themes, and the grant rows in light (100 % and 200 %), contrast (100 %) and dark (200 %, earlier glyph) in chrome-devtools screenshots (`c1/repair-2/r2-*.png`). The `c1/writer` and `c1/repair-1` renders themselves were not re-viewed, and no independent visual review has been made. |
| O-B1 | Kern band uses `lock`; BRAND.md §5 places the compact mark on trusted surfaces. Since C1 repair-2 `lock` appears only on the Kern band and the prompt openers in the gallery, not on grant chips. |
| O-B2 | BRAND.md §5 describes red, green and blue as accents for highlights and app differentiation, and the mark's node colours differ from the place colours (BRAND.md §4); both sit against "colour means place" and need a project-owner decision. |
| O-I1 | No info or restore icon: the info banner uses `dot` and "Vorige grootte" uses `window`. |
| O-P1 | `tokens.css` names its generator under ignored `build/`; the generator and mapping must move into the repository for the file to be reproducible. `work/gen_tokens.py` and `work/mapping.tsv` predate repair-1, which edited `tokens.css` by hand; the current mapping is `build/design-evidence/ds-1.0/repair-1/tools/mapping.tsv`. |
| O-P2 | Move the tokens, components and §4 contrast evidence from `build/design-evidence/ds-1.0/` to `docs/verification/`. |
| O-P3 | DESKTOP-SHELL.md §7 still describes a mint "Lokaal" privacy light; CONCEPTS.md MF2 removes the mint tint, and ds-1.0 uses the three-node instrument. |
| O-D1 | Gallery sample data not in `ui/design/prototype/data.js` (file sizes, "Vakantiefoto's Lissabon"; the seeded "Altijd" grant Notities · collectie Tuin, created through `ask()` at gallery load; the two prompts' "what" and "done" fragments in `gallery.js`). |
| O-G7 | The Apple HIG Privacy and Chromium Blog rows in §11.1 are not re-verified (both render client-side; the verifier retrieved only the Chromium date). Apple's "ask once, then redirect to Settings" behaviour named in the task brief is not cited. Quote both from a retrieved copy, or from Apple's API documentation, before relying on them. Since C1 repair-2, option B cites only Android as support. |
| O-G9 | Interpretation for owner confirmation: the destination of a sending right is part of the scope ("reikwijdte") of the four-field key, carried as `to` in the contract (§11.1). The alternative is a fifth key field; the behaviour is the same. |
| O-G10 | Contrast theme exception, pre-existing since `46957bac`: the decorative row, window, Orbit and privacy-connector blades keep transparent gradient ends over black (§13), against the "no translucency" rule. Justification: their solid centre carries the cue at 21:1 and the ends are decoration; either confirm the exception or make the ends opaque in `[data-theme="contrast"]` and re-profile the focus-ring gaps (O-K2). |
| O-G11 | The grant row's reach badge and duration chip are a design proposal for UX-E (§11.1); their scanning benefit is AI judgement, untested with users (U06). |
| O-G12 | Owner question, label not changed: "Toestaan tot afsluiten" can be read as "until shutdown". The BaryonOS power menu itself uses "Afsluiten" for shutting down (`strings.js` `shell.shutdown`, NAMING-AND-IA.md Power row: "Aan/uit: Slaapstand, Opnieuw opstarten, Afsluiten"), and Windows NL uses the same word (ADR-058). The contract ends the grant earlier, when the app closes; the duration line "Geldt tot je <App> sluit" states this. Under owner rule M3 (no label may suggest another validity) the owner is asked to confirm "afsluiten" or clarify it, for example "Toestaan tot je de app sluit". The verify-1 note that BaryonOS has no internal clash is not correct. |
| O-G13 | The `altijd` glyph (closed loop) is a ds-1.0 addition, not in the reviewed v2 sprite (§8); it replaces `lock` on the "Altijd" grant chip. Brand review and a user check of its reading are pending. The final form (particle r 2.1 on the top of the right lobe) was viewed at 16 px in light and contrast and at 32 px in light; at 16 px the particle reads as a small dot on the loop, slight but present (AI judgement). |
| O-G14 | UX-E design proposal, not built: grants grouped per app with a combined-reach privacy strip (§11.1). Needs its own keyboard, focus-after-revoke and screen-reader design. |
| O-G8 | `docs/verification/suite-s1-capability-matrix-20261114/` may hold a 215-row summary; the matrix now has 216 rows (XC09) and the stored summary was not regenerated (no write access). |
