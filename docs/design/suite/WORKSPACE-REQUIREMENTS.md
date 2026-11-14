# Professional workspace requirements (S0)

Status: design proposal, 14 November 2026, repaired after independent
verification; the lead's reversible design decisions under the project
owner's delegation, not ADRs; no user test was run. [SOURCE] marks an opened
source, [ANALYSIS] derived reasoning, [JUDGEMENT] a design choice. Colour
figures: `python3 -I docs/verification/suite-s0-workspace-20261114/contrast.py`
(`contrast.txt` and `SHA256SUMS` beside it; WCAG 2.2 formula, floored,
self-checked; overlays composited to 8 bits with ties rounded to even), from
Orbitaal v1 (`ui/design/concepts/orbitaal/index.html` lines 40, 49, 320).
All neutral colour values are PROVISIONAL (D3).

## Decisions

### D1 Colour rule scope and error status — ds-1.0

**Decision.** "Colour means place" governs chrome and trust surfaces: the
shell, ds components outside work regions, and the place badge anywhere. In
work regions user content is unrestricted (place hues may occur in
documents); `content.label.*` (named track, layer, clip and series labels,
bar swatch) and `overlay.*` (selection outline, guides, mask, playhead,
meters, record-arm) never borrow place values or shapes; both are
valued in ds-1.1. An error, in chrome and in workspaces, is a
square (`.ds-status-marker`) filled with `text.primary`, with an icon and a
message in `text.primary`; TYPOGRAPHY §8.3's `status.error` resolves to
`text.primary`.
Warnings use icon and text. ds-1.0 ships no red status colour: not the
`place.internet` red, not v1's `.pill.err` fill (#C9352A), nor any
red-tinted warning token. **Rationale.** [SOURCE]
WCAG 1.4.1; Apple: one meaning per colour; Excel scales colour data green to
red; CONCEPTS.md adopts Materie's shape code: dot device, ring with dot NAS,
open ring internet, square error. [ANALYSIS] The shared red as text fails 4.5:1 in 12 neutral cases
(light #CC3328 on raised #E8E8E8 4.20 FAIL). [JUDGEMENT] CONCEPTS and
TYPOGRAPHY §8.3 state no fill; `text.primary` keeps the square neutral and
above 4.5:1 on every base and state (`contrast.txt` sections 5, 6b and 9).
**Sources:** W1, P2, B1.

### D2 Density, type and targets — ds-1.0

**Decision.** Densities `standard` and `compact`, set by `data-density` on
any region down to one panel, change only size and spacing tokens. Compact
rows use `type.body` at its 20 px dense line (24 = 20 + 2 × 2; standard
32 = 22 + 2 × 5) and grow with wider text spacing. Every component hit area
is at least 24 × 24 without the Spacing exception. Content handles
(keyframes, clip edges, guides, splitters) get a 24 px hit band. Only where
neighbouring handles sit closer than 24 px may a handle use the Equivalent
exception, and its inspector field then stays in the same view in every
preset, `essential` included; otherwise the handle uses the band. Ruler
numerals and timecode use `type.caption`; the exact value appears in
`type.body` in the inspector or status bar. No `type-1.0` size is added.
**Rationale.** [SOURCE] SC 2.5.8: Equivalent needs a control "on the same
page"; its Understanding page names density control as an option; WinUI has
two densities, scopable per panel. [JUDGEMENT] S7 tests 13 px rulers. **Sources:** W1, W2, P4.

### D3 Neutral surfaces — rule ds-1.0, values PROVISIONAL

**Decision.** `data-surface="neutral"` replaces the surface, line, text and
focus roles with achromatic twins (same luminance, C\*ab ≤ 0.01; dark
`line.strong` is lifted to #747474 because its twin #696969 is 2.93 FAIL on
raised) and adds `surface.surround`; only `data-surface` switches it. Twinning moves contrast by at most 0.09 (measured 0.089);
no threshold flips at rest. Other v1 values in the app window are twinned at
the same alpha or removed (state table). Hover and selection fills may appear
on panel, window, raised and inset, never on the surround; hover does not
stack on a selected row, which also shows the selection bar and weight 600.
`focus.ring` takes `text.primary`. The surround carries only the focus ring,
plus a 1 px `line.strong` canvas edge in high contrast, where surround and
surfaces are both #000000. The override covers the app window; trust
surfaces, scrim and Orbit never change; photo, RAW, grading and proofing
presets turn it on; users toggle it per workspace. **Acceptance check:** every
text pair ≥ 4.5:1 and every `line.strong`, `focus.ring` and place-glyph pair
≥ 3:1 on each allowed base at rest, under each hover fill and under the
selection fill, in dark, light and high contrast. The v1-derived values only
illustrate while the v2 board is unaccepted; they fail 12 pairs and v1 itself
fails 20. Button fills (the `--sec-bg` and `--pill-bg` twins) are not yet
bases of the check; on the `--sec-bg` twin over raised (#2E2E2E) dark
`line.strong` is 2.90 FAIL at rest, PROVISIONAL and outside the 12
(`contrast.txt` section 7b). The
ds-1.0 tokens stage adds them as bases, computes final values from the
accepted board, raising `line.strong` and `text.tertiary` or lowering state
alphas until the check passes, and stores the run under
`docs/verification/`. **Rationale.**
[ANALYSIS] v1 chrome is tinted (dark surfaces C\*ab 4.4–18.0). [SOURCE]
ISO 3664:2009: the surround affects adaptation; a monitor border is normally
dark grey or black; BT.2100-3: neutral grey room. [ANALYSIS] Applying both to
chrome is extrapolation. [JUDGEMENT] Surrounds: dark #090909; light L\* 50
(#777777), departing from the ISO note to limit adaptation swings beside light
chrome; S7 tests it. **Sources:** C1, C2, W1.

### D4 Region names — ds-1.0

**Decision.** Reserve the `.ds-` classes and attributes in the names table;
avoid "dock" (Orbit is "Orbit, dock"), bare "palette" and bare "grid".
Visible names come from the NAMING-AND-IA string table, which still lacks
them (open item 8). [ANALYSIS] Renaming after S7 touches every consumer.

### D5 Keyboard namespaces — ds-1.0 (keymap editor, presets ds-1.1)

**Decision.** Four namespaces in priority order. OS-owned: Ctrl+Alt+Delete
goes only to the session service, never to the shell or an app; OS-key
chords, Alt+Tab, Alt+F4, Ctrl+Shift+Esc and Print Screen go to the shell. No
OS key reaches an app or is bindable; for Alt+F4 the shell sends a close
request, so the app's save/discard/cancel dialog still runs. App standard:
DESKTOP-SHELL's Ctrl+C/X/V/Z/Y/S/A/F/P plus Ctrl+N/O/W, Shift+Ctrl+Z and
Ctrl+plus/minus/0 (view zoom), one meaning everywhere (amendment in open item
5). App modified: other chords. Context single keys (letters, Space, J/K/L):
only with focus in a canvas or timeline, never in a grid or text input,
switchable off. Transport: Space, J/K/L, arrows. Command palette:
Ctrl+Shift+P. Keymaps are per user, exported as a delta; presets are named by
convention, never by vendor; `.ds-keycap` renders shortcuts from the live
keymap.
**Rationale.** [SOURCE] PROFILES-AND-USERS §3 and invariant 8, DESKTOP-SHELL
§1 and §3: the input router sends Ctrl+Alt+Delete exclusively to the session
service. [SOURCE] NATIVE-MATERIAAL: a dirty document asks save/discard/cancel.
[SOURCE] GNOME lists Ctrl+N/O/W, Shift+Ctrl+Z and the zoom keys as
conventions; Apple lists the core editing set and says not to repurpose it;
SC 2.1.4 accepts focus-only activation; Krita, Final Cut Pro, Blender and VS
Code show single keys, J/K/L, keymap deltas and Ctrl+Shift+P.
**Sources:** W1, W3, P1, P3, B2–B5.

### D6 Progressive disclosure — ds-1.0

**Decision.** Disclosure belongs to the workspace, not the person: presets
(`data-workspace`) at `data-disclosure="essential|extended|complete"` change
layout, tools and density, never documents, permissions, shortcuts, the menu
bar or the palette. Every app opens at `essential` on first launch; the last
level is kept per user and app. Route: View > Workspace lists Essential,
Extended and Complete as radio items plus named presets; the palette offers
"Workspace: Essential", "Workspace: Extended" and "Workspace: Complete".
Every level keeps every command in the menu bar and palette and the fields D2
and D9 require; panels have at most two levels; inspector and a contextual
toolbar group follow the selection. The switcher control and custom layouts
follow in ds-1.1. **Rationale.** [SOURCE] NN/g: defer
advanced features behind an obvious path; more than two levels usually
fails; the owner's addendum: approachable without hiding expert
capabilities. [JUDGEMENT] `essential` is safe as default because no command
leaves the menu bar or palette. **Sources:** R1.

### D7 Materiaal object components — ds-1.0

**Decision.** `.ds-object-chip` (icon, name, place glyph; bound to the object
ID); `.ds-place-badge` (shape, icon, NAMING-AND-IA §6 storage label);
`.ds-version-indicator` with the UX-MATERIAAL-SAVE-RECOVERY states: unsaved,
saving (with "Cancel saving"), confirming storage, saved (only after
`Saved{revision}`), saving cancelled (draft stays unsaved), conflict, storage
full or I/O error, read-only or denied (save disabled with a reason),
outcome unknown (no "Try again"); it opens Versies with "Deze versie
terugzetten". `.ds-link-badge` is a reserved name until S4 defines linking.
Timeline, compare and relink follow in ds-1.1.
**Rationale.** [SOURCE] NATIVE-MATERIAAL and UX-MATERIAAL-SAVE-RECOVERY
confirm storage only through `Saved{revision}` and keep the draft in every
failure state.

### D8 Professional component set — ds-1.0

**Decision.** The components in the names table. The number field shows its
unit, steps with Up/Down and Page Up/Down, uses Home/End when a minimum or
maximum exists, and has no stacked arrows; the grid has one tab stop; the
export dialog shows format, quality, colour profile, compatibility target,
size and what will not survive. Docking, colour picker, curves, scopes,
meters, node and curve editors follow in ds-1.1; S7 shows colour picker and
meters as labelled placeholders. **Rationale.** [SOURCE] APG: Up/Down required, Page
Up/Down optional, Home/End with a limit; one grid tab stop; the addendum asks
for explicit export settings. [JUDGEMENT] This set covers the four S7 designs.
**Sources:** W4, W5.

### D9 Dragging alternatives (SC 2.5.7) — ds-1.0

**Decision.** Every drag has a single-pointer alternative without dragging at
every disclosure level. Keyframe, clip-edge and guide positions: inspector
number fields, and a click on the ruler places the playhead or a guide. Layer
and track order: Move up, Move down, Move to top and Move to bottom in the
context menu and menu bar. Splitters: Narrow, Standard, Wide and Place
splitter (the next click sets its position) in the panel menu, plus a width
`.ds-number-field` there; double-click resets. Docking (ds-1.1): "Move panel
to…". Marquee and lasso selection: the commands Select all, Toggle selection,
Select area, Polygon lasso, Close lasso and Cancel lasso sit in the menu bar,
the palette and the tool options. With Toggle selection on, a plain click
adds or removes one item, and Select area and Polygon lasso add to the
selection instead of replacing it. Select area: a first click sets one
corner, a second click the opposite corner, and the items inside are
selected. Polygon lasso: each click adds a vertex; a click on the first
vertex (24 × 24 hit area, D2) or Close lasso closes it; Cancel lasso discards
it. Shift+click and Ctrl+click stay as keyboard shortcuts for the same
toggling. Sliders: a click on the track sets the value. Only freehand
strokes, where the drawn path is the content, are the "dragging is
essential" case.
**Rationale.** [SOURCE] SC 2.5.7 (AA); the W1 glossary calls functionality
essential only if it "cannot be achieved in another way that would conform".
[ANALYSIS] Keyboard alternatives alone do not satisfy it, so a modifier key
cannot be the only route to an arbitrary subset. Two clicked corners and a
click-placed polygon select the same items as a marquee or lasso drag, so
neither drag is essential. **Sources:** W1.

## Tokens to add (values PROVISIONAL, v1-derived)

Neutral rows apply under `data-surface="neutral"`; minima are at rest over
panel, window, raised and inset (state fills: failure table).

| Token | Purpose | Dark | Light | High contrast | Minimum at rest |
|---|---|---|---|---|---|
| `size.target.min` | Hit area | 24 × 24 | same | same | — |
| `size.control` | Control height, standard / compact | 32 / 24 | same | same | — |
| `space.control` | Control inset | 12 / 8 | same | same | — |
| `space.gap` | Control gap | 8 / 4 | same | same | — |
| `space.panel` | Panel inset | 16 / 8 | same | same | — |
| `surface.surround` | Canvas surround | #090909 | #777777 | #000000 | focus 17.62 / 4.00 / 16.34 |
| `surface.panel` (s1) | Panel | #111111 | #F0F0F0 | unchanged | — |
| `surface.window` (s2) | Window | #171717 | #FCFCFC | unchanged | — |
| `surface.raised` (s3) | Raised | #212121 | #E8E8E8 | unchanged | — |
| `surface.inset` (sin) | Field | #0B0B0B | #FFFFFF | unchanged | — |
| `line.subtle` (ln) | Divider | #2C2C2C | #DEDEDE | unchanged | decorative |
| `line.strong` (ln2) | Boundary | #747474 (twin #696969: 2.93 FAIL on raised) | #818181 | unchanged | 3.44 / 3.17 (raised) |
| `text.primary` (tx1) | Text | #F1F1F1 | #171717 | unchanged | 14.25 / 14.63 (raised) |
| `text.secondary` (tx2) | Secondary | #B2B2B2 | #4B4B4B | unchanged | 7.59 / 7.11 (raised) |
| `text.tertiary` (tx3) | Metadata | #969696 | #626262 | unchanged | 5.44 / 4.97 (raised) |
| `focus.ring` | Focus | #F1F1F1 | #171717 | #FFE34D | 14.25 / 14.63 / 16.34 |

Place glyphs pass 3:1 on every allowed base and state (lowest: light
#CC3328 on hovered raised #D7D7D7, 3.58); high contrast passes every pair.

**State values under neutral** (twin = same luminance, same alpha):

| v1 value | Treatment | Dark | Light |
|---|---|---|---|
| `--selbg` selection | twin | #A0A0A0 at 15 % | #676767 at 10 % |
| `--hov`, `--hov2` hover | twin | #BEBEBE at 7 % / 12 % | #2F2F2F at 5 % / 9 % |
| `--rim-a`, `--rim-b`, `--rim-c` | twin | #E9E9E9 at 46 %; #A7A7A7 at 10 % / 20 % | #FFFFFF; #5B5B5B at 16 % / 30 % |
| `--sec-bg`, `--pill-bg` | twin | #BEBEBE at 8 % / 6 % | #FFFFFF |
| `--pri-bg`, `--pri-bg-h` gradients | stops twinned | #F6F6F6→#DFDFDF, #FFFFFF→#ECECEC; text #0E0E0E ≥ 14.48 | #2C2C2C→#181818, #393939→#212121; text #FFFFFF ≥ 11.54 |
| `--silver` | twin | #D2D2D2 | #666666 |
| `--wsh`, `--wsh-off` window shadows | twin | #000000 (already neutral) | #393939 at 22 % / 14 %; 16 % |
| `--focus`, selection bar | `text.primary` | #F1F1F1 | #171717 |
| `--glowc` glow, `.pill.err` | removed | — | — |

Unchanged: `--priv-bg` (place badge) and everything outside the app window.

**Acceptance-check failures of the provisional twins** (v1 = same pair in
Orbitaal v1, inherited):

| Theme | Foreground | Over | Row | Ratio | Needed | v1 |
|---|---|---|---|---|---|---|
| dark | `text.tertiary` #969696 | strong hover, raised | #343434 | 4.20 FAIL | 4.5 | 4.21 FAIL |
| dark | `text.tertiary` #969696 | selection, raised | #343434 | 4.20 FAIL | 4.5 | 4.19 FAIL |
| dark | `line.strong` #747474 | selection, window | #2C2C2C | 2.98 FAIL | 3 | 2.57 FAIL |
| dark | `line.strong` #747474 | hover, raised | #2C2C2C | 2.98 FAIL | 3 | 2.57 FAIL |
| dark | `line.strong` #747474 | strong hover, raised | #343434 | 2.66 FAIL | 3 | 2.29 FAIL |
| dark | `line.strong` #747474 | selection, raised | #343434 | 2.66 FAIL | 3 | 2.28 FAIL |
| light | `text.tertiary` #626262 | strong hover, raised | #D7D7D7 | 4.23 FAIL | 4.5 | 4.20 FAIL |
| light | `text.tertiary` #626262 | selection, raised | #DBDBDB | 4.40 FAIL | 4.5 | 4.30 FAIL |
| light | `line.strong` #818181 | strong hover, panel | #DFDFDF | 2.92 FAIL | 3 | 2.88 FAIL |
| light | `line.strong` #818181 | hover, raised | #DFDFDF | 2.92 FAIL | 3 | 2.90 FAIL |
| light | `line.strong` #818181 | strong hover, raised | #D7D7D7 | 2.70 FAIL | 3 | 2.69 FAIL |
| light | `line.strong` #818181 | selection, raised | #DBDBDB | 2.81 FAIL | 3 | 2.75 FAIL |

v1 fails 20 pairs (`contrast.txt` section 10): `line.strong` 2.28–2.98 FAIL
and `text.tertiary` 4.19–4.30 FAIL under state fills, and dark `line.strong`
#5D6890 on raised #1A1F38 at rest, 2.96 FAIL. With the
unaccepted v2 selection alpha (white 8.5 %, black 6.5 %): #747474 on #343434
2.66 FAIL, #969696 on #343434 4.20 FAIL, #818181 on #E0E0E0 2.95 FAIL,
#818181 on #D9D9D9 2.75 FAIL, #626262 on #D9D9D9 4.32 FAIL. Rounding ties up
instead of to even moves only the light v1 selection fill over window and
raised by one 8-bit step: 4.30 becomes 4.33 and 2.75 becomes 2.77, still
FAIL, and no verdict changes (`contrast.txt` section 8b).

## Names to reserve

| Name | Kind | Version |
|---|---|---|
| `.ds-workspace` `.ds-menu-bar` `.ds-toolbar` `.ds-tool-rail` `.ds-canvas` `.ds-inspector` `.ds-panel` `.ds-panel-zone` `.ds-timeline` `.ds-data-grid` `.ds-status-bar` `.ds-document-tabs` `.ds-command-palette` | Regions | ds-1.0 |
| `.ds-panel-header` `.ds-track` `.ds-ruler` `.ds-playhead` `.ds-cell` `.ds-number-field` `.ds-layer-row` `.ds-keycap` `.ds-expander` `.ds-export-dialog` `.ds-object-chip` `.ds-place-badge` `.ds-version-indicator` `.ds-status-marker` | Components | ds-1.0 |
| `.ds-link-badge` | Component, name only | ds-1.0 name; defined after S4 |
| `data-density` `data-surface` `data-workspace` `data-disclosure` | Attributes | ds-1.0 |
| `content.label.*` `overlay.*` | Token namespaces | ds-1.1 values |
| `.ds-panel-float` `.ds-workspace-switcher` `.ds-keymap-editor` `.ds-version-timeline` `.ds-color-picker` `.ds-scope` `.ds-meter` | Components | ds-1.1 |

## Rules for ds-1.0

- Use place hues, dots and rings only for place, the square only for errors; ship no red status colour.
- Give every target 24 × 24, or Equivalent with its field in view in every preset; give every drag a non-drag alternative.
- Pass the acceptance check with hover and selection fills on every allowed base.
- Keep trust surfaces unchanged; draw only the focus ring (and the high-contrast edge) on the surround.
- Route Ctrl+Alt+Delete only to the session service, other OS keys only to the shell.
- Keep every command in the menu bar and palette at every level; show "saved" only after `Saved{revision}`.

## Open items

1. 11–12 px labels lie outside `type-1.0`; only a new type revision could add
   them. The dense 20 px line and `type.reading` have no native record.
2. Final neutral and state values: ds-1.0 tokens stage, against the accepted
   v2 board, clearing the 12 failures and the button-fill pair (2.90, D3).
3. A non-place status hue, with a colour-vision check (UX-D).
4. Saturated wallpaper around a non-maximised neutral window; ISO 3664 clause
   4.5 and ISO 12646 unread.
5. DESKTOP-SHELL amendment: add Ctrl+N/O/W, Shift+Ctrl+Z and
   Ctrl+plus/minus/0; Apple-keyboard app chords (Cmd is the OS key) need a
   project-owner decision.
6. Non-US layouts and assistive-technology modifiers.
7. Ctrl+Shift+P collision review; legal check of familiar presets.
8. Localised names for workspace regions in the NAMING-AND-IA string table.
9. Label and overlay values, with a colour-vision check, before S7.
10. Task trials (U06) for every decision, the `essential` default included.

## Sources

Retrieved 14 November 2026. Project: DESKTOP-SHELL §1–3, PROFILES-AND-USERS
§3–4, NAMING-AND-IA §4–6, TYPOGRAPHY §8, NATIVE-MATERIAAL,
UX-MATERIAAL-SAVE-RECOVERY, CONCEPTS; the owner's addendum on the
professional ecosystem (local file).

| ID | URL | Supports; limitation |
|---|---|---|
| W1 | https://www.w3.org/TR/2024/REC-WCAG22-20241212/ | SC 1.4.1, 1.4.3, 1.4.11, 2.1.4, 2.5.7, 2.5.8, contrast formula; web standard used by analogy |
| W2 | https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html | Spacing, density; informative |
| W3 | https://www.w3.org/WAI/WCAG22/Understanding/character-key-shortcuts.html | Speech-input intent; informative |
| W4 | https://www.w3.org/WAI/ARIA/apg/patterns/spinbutton/ | Number-field keys |
| W5 | https://www.w3.org/WAI/ARIA/apg/patterns/grid/ | Grid keys |
| C1 | https://cdn.standards.iteh.ai/samples/43234/d3de25012f6b433e886679d3a450499f/ISO-3664-2009.pdf | 3.16, 4.1.2; preview ends before 4.5 |
| C2 | https://www.itu.int/dms_pubrec/itu-r/rec/bt/R-REC-BT.2100-3-202502-I!!PDF-E.pdf | Table 3; room, not UI |
| P1 | https://developer.apple.com/tutorials/data/design/human-interface-guidelines/keyboards.json | Core editing set; no Command-0 or view zoom |
| P2 | https://developer.apple.com/tutorials/data/design/human-interface-guidelines/color.json | One meaning per colour |
| P3 | https://developer.gnome.org/hig/reference/keyboard.html | Standard, zoom and legacy system shortcuts |
| P4 | https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/compact-sizing | Two densities; no pixel values |
| B1 | https://support.microsoft.com/en-us/office/use-conditional-formatting-to-highlight-information-in-excel-fed60dfa-1d3f-4e13-9ecb-f1951ff89d7f | Colour as data |
| B2 | https://support.apple.com/guide/final-cut-pro/play-media-ver90ba4ef0/mac | J/K/L |
| B3 | https://docs.krita.org/en/reference_manual/tools/freehand_brush.html | Single-key tool |
| B4 | https://docs.blender.org/manual/en/latest/editors/preferences/keymap.html | Keymap presets, delta export |
| B5 | https://code.visualstudio.com/docs/getstarted/userinterface | Ctrl+Shift+P |
| R1 | https://www.nngroup.com/articles/progressive-disclosure/ | Two levels; 2006 |
