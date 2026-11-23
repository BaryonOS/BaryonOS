# Consumer-to-professional maturity model (S2)

## 1. Status

Status: design research and proposal, 18 November 2026 (Europe/Amsterdam).
Not an ADR and not a normative decision; no user test was performed, and no
statement here is user evidence. This is deliverable S2 of the project owner's
professional-ecosystem addendum (§8, item 2), written on branch
`claude/baryon-os-suite-20261118` at source `862d29d7`. Evidence levels are
those of S1 (`docs/design/suite/CAPABILITY-MATRIX.md` §1): *passed (native)*,
*prototyped (host)* and *proposed*; this document raises no row and assigns
none of its own requirements a level above *proposed*. Markers follow S0:
[SOURCE] an opened source, [ANALYSIS] derived reasoning, [JUDGEMENT] a design
choice or assessment of the author; AI review is not user evidence. Mature
suites are functional and workflow benchmarks only; no trade dress, layout,
iconography or documentation text is reproduced. Level names, product names
and file names are provisional; P-NAMES-1 is undecided and this document
decides no name.

## 2. Purpose and scope

One workspace must serve a person who opens a document for the first time and
a person who works in it eight hours a day, without a "lite" product, a second
edition or a mode that removes capability. The addendum asks for "coherent
beginner, intermediate and professional interaction paths" (§6) and for
progressive disclosure "without hiding essential expert capabilities" (§1).

S0 (`WORKSPACE-REQUIREMENTS.md`, accepted) already supplies the mechanisms:
disclosure presets that belong to the workspace, not the person (D6), two
densities (D2), keyboard namespaces (D5), a professional component set (D8),
single-pointer alternatives to every drag (D9) and the Materiaal object
components (D7). S1 (`CAPABILITY-MATRIX.md`, accepted) supplies 216
capability rows, each with a *Beginner path* and a *Professional depth*
column, and a proposal of shared workspaces and specialised applications
(S1 §5). S2 adds:

1. a small set of **maturity levels**, each defined by what a user observably
   does (section 3);
2. a **dimension × level table** stating, for fifteen dimensions, what the user
   can do and what the system must provide at each level (section 4);
3. **progression rules** for moving between levels without hiding data or
   switching off safety (section 5);
4. a **per-discipline mapping** of every S1 row to the level from which its
   Beginner path and its Professional depth are part of the minimum
   (section 6);
5. **acceptance criteria** per level, all *proposed* (section 7).

Out of scope: file-format names (S3), the native object architecture (S4),
the interoperability matrix (S5), application concepts and visual designs
(S6, S7) and the phased plan (S10). Nothing here changes a security contract,
the Security Floor, an owner decision or an S0 or S1 decision.

### 2.1 Terms

- **Maturity level**: a description of what a user does in one discipline. It
  is a design and test instrument for this document, S6 and S10. It is never
  a setting, never stored, never inferred from usage and never shown to the
  user as a label (RULE-1).
- **Disclosure preset**: S0 D6's `data-disclosure="essential|extended|complete"`
  on a workspace, chosen by the user. S0 calls it a "level"; S2 says
  "disclosure preset" to keep the two concepts apart.
- **Density**: S0 D2's `standard` and `compact`, settable per region.
- **An application serves level N** when (a) each column in the
  application's section 6 minimum at N (section 6.1: Beginner path,
  Professional depth or both, of its discipline's rows, the cross-cutting
  rows and the rows that S1 §5 proposes as shared) has native acceptance
  evidence that covers that column, in the sense S1 §1 gives *passed
  (native)*, with XC09 read with ADR-DESIGN-OWNER-20261115 D2 (section 6.1,
  rule 5); (b) it provides every "system must provide" cell of section 4 at
  or below N, under the applicability rule of the section 4 introduction;
  and (c) it passes the section 7 criteria for every level up to N and the
  cross-level criteria, applied to level N: a step or measure does not apply
  when it exercises only cells above N, or only clauses that do not bind the
  application at N under the applicability rule (section 7, introduction);
  a step that checks a safety, accessibility or permission protection
  applies to every view and setting the application provides, whatever
  level the view or setting belongs to (RULE-4). A scenario is written once,
  at the lowest level at which a user meets it, and some scenarios also
  exercise a cell of a higher level; section 8.2, item 17 lists those
  criteria and cells and states which steps remain at the lower level.
  S1 evidence levels are per row, and the name of a row above *proposed* is
  its exact tested feature, which can be narrower than its columns (S1 §1;
  WP22 "DOCX import (plain text)" has a Professional depth that section 6
  requires from L3), so a row's evidence level does not by itself establish
  clause (a). [ANALYSIS] Every discipline's L1 minimum contains *proposed*
  rows (section 6), so **no application serves L1 today**.

### 2.2 Crosswalk

| Maturity level | Addendum path (§6) | S1 column that is the minimum | Disclosure preset usually sufficient (S0 D6; the user's choice, never bound to the level) | Learnability focus [E2] |
|---|---|---|---|---|
| L1 First use | Beginner | Beginner path of L1 rows | `essential` (also the first-launch preset, S0 D6) | Initial learnability: understanding task flow, awareness of functionality |
| L2 Everyday | Beginner to intermediate | Beginner path | `essential` or `extended` | Locating functionality |
| L3 Proficient | Intermediate | Beginner path; Professional depth of core rows | `extended` | Understanding functionality |
| L4 Expert | Professional | Professional depth | `complete` | Transitioning to efficient behaviours |
| L5 Specialist | Professional | Professional depth, including automation, extension and interchange | `complete` plus custom layouts (ds-1.1 reserved names) | Extending the tool for others; outside the categories of [E2] |

[SOURCE] Grossman et al. distinguish initial learnability ("initial
performance with the system") from extended learnability (change in
performance over time) and classify observed learnability issues as
understanding task flow, awareness of functionality, locating functionality,
understanding functionality and transitioning to efficient behaviours [E2].
[JUDGEMENT] The column "Learnability focus" maps those categories to levels;
the mapping is the author's, not Grossman's.

## 3. Maturity levels

| Level | Provisional name | Observable behaviour (what the user does in this discipline) | Observable signal of having reached it |
|---|---|---|---|
| L1 | First use | Works through visible controls: menus, toolbars, buttons, context menus, drag and drop. Follows empty-state cues, a template or a guided first task. Creates or opens a document, makes a change, saves it and finds it again. Uses help or abandons the task when stuck. | None: every first contact with an application or a discipline starts here. |
| L2 | Everyday | Repeats a set of core tasks across sessions without help. Uses the app-standard shortcuts (S0 D5) and a few others. Opens files received from others and sends files back. Comments, undoes and restores an earlier version when needed. Finds an unfamiliar command through the menus or the command palette. | Completes the discipline's core task set without help in a later session. |
| L3 | Proficient | Builds structure instead of direct formatting: styles, layers, masks, references, tracks, named ranges, components. Sets values numerically in the inspector. Reviews with tracked changes and version comparison. Changes the disclosure preset or the panel arrangement. | Uses structural features to make a change that applies throughout a document. |
| L4 | Expert | Works keyboard-led on routine tasks (shortcuts, context single keys, palette) with a customised keymap and saved layouts. Enters exact values with units. Works on large documents and batches. Exchanges files with other professional tools and checks fidelity. Delivers to external specifications (print, broadcast, archival, accessibility). | Keyboard and numeric routes dominate frequent commands; output meets an external specification. |
| L5 | Specialist | Extends and shapes the tool for repeatable work or for others: scripts and recorded automation, plugins, templates, presets, shared libraries and brand kits, interchange and colour pipelines. Reads fidelity reports and performance diagnostics. | Produces an automation, template or library that someone else uses. |

Rules for using the levels:

- **Per discipline and per task.** A teacher can be L4 in word processing and
  L1 in audio production. Levels are not personas.
- **Not a diagnosis or a person.** Accessibility needs are independent of
  level: an expert screen-reader user is L4, and tier-2 settings combine with
  every level (ADR-DESIGN-OWNER-20261115 D1; dimension D12).
- **Not time in product.** A professional who switches from another suite can
  act at L4 on the first day when conventions and keymaps transfer (ADR-058;
  S1 XC05).
- **Not an edition.** No licence, edition, price or feature set differs by
  level [JUDGEMENT]; changing that would be an owner decision (section 8.1).

Grounding: [SOURCE] Scarr et al. report that "few users switch" to expert
shortcut mechanisms, that interfaces can trap users in a "beginner mode" with
a low performance ceiling, that satisficing keeps "good enough" strategies in
use, and that the performance "dip" after switching modality deters the switch
[E1]. They derive three goals: maximise the likelihood of a switch, minimise
its cost and enable a high ceiling [E1]. [SOURCE] Shneiderman proposes
multi-layer designs in which novices start with a limited feature set and move
up "when needed"; the paper records critics' concerns that mature users adopt
strongly different feature subsets and asks how compatibility of output across
layers can be ensured [E5]. [ANALYSIS] Because experts use different subsets,
a linear ladder cannot prescribe which features a user sees. S2 therefore uses
levels only to state the minimum an application must offer and to define
tests, never to decide what is visible.

## 4. Dimension × level table

Each dimension has one table with the same columns. Cell identifiers
(`D01.L1` to `D15.L5`) are stable. The "Cells and rules tested" column of
section 7 names the cells each criterion tests; section 8.2 lists the cells
that no criterion tests, or tests only in part. "The user can"
describes behaviour the level needs; "The system must provide" is a
requirement on an application that claims the level, cumulative with all
lower levels, under this **applicability rule** [JUDGEMENT]. A clause that
rests on an S1 XC row or on a row that S1 §5 proposes as shared by every
surface (GR34–GR37, UX06, UX07), or that names no discipline capability (an
S0 rule, a ds-1.x protection, a privacy rule, a budget or a [JUDGEMENT] for
every application), binds every application. A clause that names a
capability of other S1 rows (for example recorded actions, rigs, proxies or
tracked changes) binds the disciplines of the rows its Basis cites, at the
cell's level; a discipline whose own S1 row names the same capability in its
Beginner path or Professional depth without being cited follows the level at
which section 6 requires that column, earlier or later than the cell; and a
discipline that has no S1 row with the capability is not bound by that
clause. Section 8.2, item 16 records the cases found and those left open.
Every requirement is *proposed*. "Basis" names the source
decision or row; cells without a project basis are marked [JUDGEMENT]. Every
S1 row in a Basis names the column it relies on, *Beginner path* or
*Professional depth*, and section 6 requires that column at or below the
cell's level; with the applicability rule, no cell requires a discipline to
provide an S1 capability before section 6 does, so clauses (a) and (b) of
"serves level N" (section 2.1) agree.
Where section 6 requires the column at a lower level, the Basis marks it
"from Lk": the cell builds on a requirement that already holds from Lk.
Safety, accessibility and permission protections are stated at the lowest
level whose views they protect; higher cells only extend them to new views.
A script checks the column and level of every such citation (section 8.2,
item 3).

### D01 Discoverability and onboarding

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D01.L1 | L1 First use | Start the application and finish a first task without an account, a network connection or a tour; see what to do next in an empty document. | Opens at `essential`; an empty state with one primary action and one sentence; an optional, skippable guided first task; locally stored templates and sample content. File access for the first task comes from the user's own open or save choice, not from an extra question. | S0 D6; S1 XC07 (Beginner path); PROFILES-AND-USERS OOBE step 9; privacy contract §4 (selected files); presentation of file choice [JUDGEMENT], subject to UX-C and A03 |
| D01.L2 | L2 Everyday | Find a command not used before through the menus or the command palette, and notice its shortcut. | Every command in the menu bar and the palette in every disclosure preset; the shortcut beside each menu item and palette entry, rendered from the live keymap. | S0 D5, D6; `.ds-keycap` |
| D01.L3 | L3 Proficient | Discover structural features (styles, layers, inspector) and switch the disclosure preset. | View > Workspace with Essential, Extended and Complete; the palette commands "Workspace: Essential", "Workspace: Extended" and "Workspace: Complete"; an inspector and a contextual toolbar group that follow the selection. | S0 D6 |
| D01.L4 | L4 Expert | Read the complete keymap, find conflicts and context single keys, and import a familiar preset. | A keymap listing searchable by command name and by key; preset import with a report of unmatched commands. The keymap editor is a reserved ds-1.1 name, not an existing component. | S0 D5; S1 XC04, XC05 (Professional depth) |
| D01.L5 | L5 Specialist | Discover the automation and extension surface: command identifiers, scripting reference and plugin interface. | A stable identifier for every command, visible in the keymap and palette details and usable by scripts; an offline reference for the scripting and plugin interfaces. | S1 WP26, SS14 (Professional depth); A01 accepted, A03 open; identifiers [JUDGEMENT] |

### D02 Command access (menus, palette, shortcuts, customisation)

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D02.L1 | L1 First use | Use the menu bar, toolbar, context menus and direct manipulation; use the shortcuts known from other systems. | The S0 D5 app-standard namespace with one meaning in every application (Ctrl+C/X/V/Z/Y/S/A/F/P, Ctrl+N/O/W, Shift+Ctrl+Z, view zoom); OS-owned keys never reach applications; a single-pointer alternative to every drag; context single keys act only with focus in a canvas or timeline, never in a grid or text input, and a switch turns them off. | S0 D5, D9; DESKTOP-SHELL §2; ADR-058; WCAG 2.2 SC 2.1.4, 2.5.7 [E8] |
| D02.L2 | L2 Everyday | Open the command palette (Ctrl+Shift+P) and run commands by name; learn shortcuts while using menus. | A palette listing every command with its shortcut; shortcuts shown at every point where the command is invoked by pointer, so the faster route is rehearsed without a switch cost. | S0 D5; [E1] three goals |
| D02.L3 | L3 Proficient | Use context single keys in a canvas or timeline, transport keys (Space, J/K/L, arrows) and app-modified chords. | Transport keys and app-modified chords listed with their context in menus, the palette and the keymap; the D02.L1 single-key rules unchanged. | S0 D5 |
| D02.L4 | L4 Expert | Rebind any command, use multi-key sequences and per-context keymaps, choose a familiar preset, export and import bindings. | Rebinding with conflict detection; keymaps per user, exported as a delta; presets named by convention, never by vendor. | S0 D5; S1 XC04, XC05 (Professional depth) |
| D02.L5 | L5 Specialist | Bind scripts and recorded actions to keys and palette entries; share a command set with a team. | Automation entries appear in the palette and keymap like built-in commands and run with only the grants of the automation itself. | S1 WP26, SS14 (Professional depth); grant scope [JUDGEMENT], needs UX-C, A03 and an ADR (section 8.1, O2) |

### D03 Information density and layout

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D03.L1 | L1 First use | Work in one task-focused layout at standard density. | The `essential` preset at `standard` density; panels at most two levels deep; nothing in the document hidden by the preset (RULE-3). | S0 D2, D6; [E6] |
| D03.L2 | L2 Everyday | Show and hide panels, zoom, and return to the default layout. | A reset-layout command; view zoom keys; the last preset kept per user and application. | S0 D5, D6 |
| D03.L3 | L3 Proficient | Use the `extended` preset and compact density on chosen panels. | `data-density` on any region down to one panel; Tier 1 holds in both densities, including B3's 24 × 24 hit areas, as at every level (D12.L1). | S0 D2; DESIGN-SYSTEM §21.1 |
| D03.L4 | L4 Expert | Use the `complete` preset, saved and shared custom layouts and per-display arrangements; work on a neutral surface for colour-critical tasks. | Saved, switchable and shareable layouts and per-display arrangements; `data-surface="neutral"` per workspace. The workspace switcher, custom layouts and docking are reserved ds-1.1 names, not existing components; neutral values are PROVISIONAL. | S0 D3, D6; S1 XC03 (Professional depth) |
| D03.L5 | L5 Specialist | Attach layouts to templates or projects so that a team receives them. | Layouts exportable and importable as files that templates and projects carry. | S1 XC03 (Professional depth, from L4: saved and shared layouts); file form and attachment [JUDGEMENT] |

### D04 Precision and numeric control

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D04.L1 | L1 First use | Place and align objects with snapping and alignment guides; undo an imprecise move. | Dynamic alignment guides; click placement of guides and playhead as the non-drag route; where a handle uses the Equivalent exception, its inspector field stays in the same view in every preset, `essential` included; the exact value of a ruler reading shown in `type.body` in the inspector or status bar. | S1 PR05, GR21 (Beginner path); S0 D2, D9; WCAG 2.2 SC 2.5.7, 2.5.8 [E8] |
| D04.L2 | L2 Everyday | Type common values such as font size, image size or duration, and step them from the keyboard. | A number field that shows its unit and steps with the S0 D8 keys (Up/Down, Page Up/Page Down, and Home/End where a limit exists), as at every level (D12.L1). | S0 D8 |
| D04.L3 | L3 Proficient | Set position, size, crop and time numerically in the inspector; align and distribute; use grids and constraints. | The Professional depth of the precision rows: numeric transforms and grids, numeric crop and straighten, align and distribute with constraints, timecode entry and sub-frame edits; inspector fields that follow the selection in every preset. | S1 PR05, GR02, GR21, VI04, TD02 (Professional depth); S0 D6 |
| D04.L4 | L4 Expert | Work numerically from the keyboard as the main route: exact values with units in every discipline, and constraints and drivers in rigs. | Units accepted and converted in every number field; constraints and drivers for rigs. | S1 TD10 (Professional depth); unit conversion [JUDGEMENT] |
| D04.L5 | L5 Specialist | Link parameters, write expressions and drive geometry from data or scripts. | Parameter linking and expressions; scripted transforms through the automation runtime. | S1 VI12 (Professional depth); Automation runtime (S1 §4.1) |

### D05 Automation and scripting

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D05.L1 | L1 First use | Repeat work by hand and start from templates. | Templates without code; nothing in a document runs code automatically; embedded automation in a received document is signalled and runs only after an explicit decision. | S1 WP19 (Beginner path); no automatic execution [JUDGEMENT] derived from privacy contract §4; decision flow needs UX-C, A03 and an ADR |
| D05.L2 | L2 Everyday | Reuse styles, presets and copied settings. | Presets and copy-paste of settings with a preview. | S1 GR18, WP03 (Beginner path); preview [JUDGEMENT] |
| D05.L3 | L3 Proficient | Apply a preset or copied settings to several selected items at once; use templates with placeholders. | Multi-item application with a preview of the result before it runs. | S1 GR18 (Beginner path, from L2), WP19 (Beginner path, from L1); multi-item application and preview [JUDGEMENT] |
| D05.L4 | L4 Expert | Record actions and replay them as a batch where the discipline offers recorded actions; run the discipline's batches and keep working while they run. | Recorded actions, replayable as a batch, in word processing and spreadsheets, whose section 6 L4 minimum contains them (WP26, SS14); graphics records actions only from L5 (GR18, D05.L5), and the other disciplines have no recorded-action row in S1 and are not bound by this clause. Every batch an application offers, recorded or built in (for example PDF page operations, image metadata templates, image export and render queues), runs as a background job, and a batch or replay that changes documents is one undoable history entry. | S1 WP26, SS14 (Beginner path), PD04, GR17, GR38 (Professional depth), VI19 (Beginner path, from L2); K05 accepted as base; scope per discipline [JUDGEMENT] (section 8.2, item 16) |
| D05.L5 | L5 Specialist | Write scripts and custom functions, record actions in graphics and run scripted batches, schedule them and watch folders. | A scripting interface under capability limits, run in isolated processes; recorded actions and scripted batches in graphics; the D05.L1 rule on embedded automation still holds. | S1 WP26, SS14, VI19, GR18 (Professional depth); K03, K04 accepted; A03 open; script grants [JUDGEMENT], need UX-C, A03 and an ADR |

### D06 Batch and large-document performance expectations

[SOURCE] Nielsen's response-time limits: about 0.1 s for a reaction to feel
instantaneous, about 1 s to keep the user's flow of thought, and about 10 s to
keep attention, beyond which a percent-done indicator and a way to interrupt
are advised [E7]. [JUDGEMENT] S2 adopts these as *proposed* budgets until U02
("measured interaction budgets", open) sets product budgets. Audio and video
real-time budgets (S1 AU16, VI01) are not covered by these limits.

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D06.L1 | L1 First use | Edit small documents with immediate feedback. | Feedback to typing and pointer input within 0.1 s; any operation longer than 1 s shows progress; longer than 10 s, a percent-done indicator and Cancel. | [E7]; U02 open |
| D06.L2 | L2 Everyday | Open and save multi-page, multi-image documents without waiting on the editor. | Saving never blocks input; the version indicator shows each save state truthfully. | S0 D7; non-blocking save [JUDGEMENT] |
| D06.L3 | L3 Proficient | Work in documents of hundreds of pages, thousands of rows or dozens of layers. | The L1 budgets hold across a proficient-scale reference corpus, to be defined with S10 (section 8.2, item 6). | [E7]; corpus [JUDGEMENT]; U02 open |
| D06.L4 | L4 Expert | Work with large datasets, high-resolution images, long timelines and batches of hundreds of files. | Incremental layout, background repagination and multi-threaded recalculation; background jobs with progress, Cancel and a per-item result report; proxy control; out-of-core tiles; cancellation leaves no unreported partial output. | S1 WP25, SS09, GR13, VI05 (Professional depth), VI19 (Beginner path, from L2); K05, K07 accepted as base |
| D06.L5 | L5 Specialist | Run scripted or scheduled batches and inspect performance. | Render queues with watch folders; local performance diagnostics; scripted batches bound by the same resource limits. | S1 VI19, TD11 (Professional depth); K07; diagnostics [JUDGEMENT] |

### D07 Collaboration and review

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D07.L1 | L1 First use | Send a copy to someone and open a file received from someone. | The Share action ("Delen"); an export that states what will not survive; a permission question that names the destination when content leaves the device. | NAMING-AND-IA §6; S0 D8; ADR-DESIGN-OWNER-20261115 D2 |
| D07.L2 | L2 Everyday | Add comments; share with a paired person or NAS. | Comments anchored to the content; sharing over paired storage only, with no vendor cloud and no silent relay. | S1 WP15 (Beginner path); privacy contract §1, §2; P01–P04 open |
| D07.L3 | L3 Proficient | Reply to, resolve and filter comment threads; turn tracking on and accept all changes; compare versions. | Comment threads with resolve states and filters; attributed tracked changes with accept all; version comparison. | S1 WP15, WP17 (Professional depth), WP16, PD11 (Beginner path) |
| D07.L4 | L4 Expert | Accept or reject individual changes, compare and merge, use versioned shared libraries and set permissions per range. | Per-change review; merge with conflict review; versioned components in shared libraries; per-range permissions. | S1 WP16, SS17, GR36 (Professional depth), NK09 (Professional depth, from L3) |
| D07.L5 | L5 Specialist | Co-edit in real time, govern team libraries and distribute and collect classroom work. | Real-time co-editing with offline merge; library governance; classroom distribution, all over paired or end-to-end encrypted storage. | S1 UX07 (Professional depth), EM11 (Beginner path, from L3), EM11, UX10, XC08 (Professional depth, from L4); N02–N04 open |

### D08 File fidelity and round trips

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D08.L1 | L1 First use | Open files from other suites and know, before saving, what will not survive. | Import that states what it could not keep; an export dialog with format, quality, colour profile, compatibility target, size and what will not survive; converters in isolated processes; no silent loss. | S1 WP22 (Beginner path), §2; S0 D8; K03, K04 accepted |
| D08.L2 | L2 Everyday | Save back in the original format and send it on. | A fidelity notice on every save to an external format. | S1 WP24 (Beginner path) |
| D08.L3 | L3 Proficient | Choose a compatibility target and read why a feature changes. | The compatibility-target choice in the export dialog with an explanation per affected feature. | S0 D8; per-feature explanation [JUDGEMENT] pending S5 |
| D08.L4 | L4 Expert | Verify round trips with other professional tools and convert in batches. | A per-feature fidelity report per document and per batch; round-trip test corpora. | S1 SS15, PR11, GR14, GR25 (Professional depth); S5 |
| D08.L5 | L5 Specialist | Build interchange pipelines (timelines, sessions, scenes). | Project, session and scene interchange with fidelity reports. | S1 AU18, TD15 (Professional depth), VI18 (Professional depth, from L4) |

### D09 Error prevention

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D09.L1 | L1 First use | Delete and change things without fear of permanent loss. | Deletion goes to the Prullenbak (recycle bin) and can be undone; confirmation only for irreversible actions ("Definitief verwijderen"); save, discard or cancel on closing a changed document; safe defaults against formula injection when data is exchanged; an error is a square with icon and message that says what happened and what happens next. | NAMING-AND-IA §6; S0 D1, D5; S1 SS19 (Beginner path); WCAG 2.2 SC 3.3.4 reversible, checked or confirmed [E9] |
| D09.L2 | L2 Everyday | Enter values from lists that accept only valid choices, and be warned before a loss. | Drop-down lists for constrained values; a notice before a lossy save; the conflict state of the version indicator; a warning when a linked resource is missing. | S1 SS07, GR32, WP24 (Beginner path); S0 D7 |
| D09.L3 | L3 Proficient | Define validation rules and check documents before handing them over. | Rule-based validation with messages; an accessibility checker with its rules; formula error hints. | S1 SS07, WP21 (Professional depth), SS18 (Beginner path) |
| D09.L4 | L4 Expert | Protect ranges, preflight to a specification and preview a batch before it runs. | Range protection and locks; preflight profiles with fixes; a batch preview listing affected items. | S1 SS17, PD10 (Professional depth); batch preview [JUDGEMENT] |
| D09.L5 | L5 Specialist | Dry-run a script before it changes documents. | A dry-run of scripted changes listing the affected items. | S1 WP26, SS14 (Professional depth); dry-run [JUDGEMENT] |

### D10 Recovery, undo and history (Materiaal history)

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D10.L1 | L1 First use | Undo and redo, restore an earlier version and get unsaved work back after a crash. | Ctrl+Z, Ctrl+Y and Shift+Ctrl+Z; autosave and crash recovery on by default; "saved" shown only after `Saved{revision}`; "Deze versie terugzetten" restores as a new revision. | S0 D5, D7; S1 XC01, XC02, WP17 (Beginner path; WP17 has host evidence of restore as a new revision) |
| D10.L2 | L2 Everyday | Browse the version list and restore deleted items. | Versies (versions) in the Files inspector; restore from the Prullenbak. | NAMING-AND-IA §5, §6 |
| D10.L3 | L3 Proficient | Compare versions, keep named versions and recover per document. | Visual comparison; named versions; snapshots and non-linear history where the discipline needs them. | S1 WP17, GR12, XC02 (Professional depth) |
| D10.L4 | L4 Expert | Choose the autosave interval and retention, and branch work. | Interval, location and retention control; branch and merge with review. | S1 XC01, UX11 (Professional depth) |
| D10.L5 | L5 Specialist | Revert a whole script run and audit changes. | One grouped history entry per script run (batches already from D05.L4); audit history. | S1 DB06 (Professional depth); grouping [JUDGEMENT] |

### D11 Help and learning

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D11.L1 | L1 First use | Get help where the task is, without a network connection. | Help in the same place in every application; a guided first project; all help stored locally. | WCAG 2.2 SC 3.2.6 by analogy [E8]; S1 XC07 (Beginner path) |
| D11.L2 | L2 Everyday | Learn a shortcut while working and search help. | The shortcut shown where the command is used; local search over help. | [E1] (supporting the switch to faster methods during work); S1 §4.1 Search & index |
| D11.L3 | L3 Proficient | Understand a feature in the context of the current document. | Explanations and demonstrations in the user's own interface, not only in a separate manual. | [E2] guideline on demonstration in context |
| D11.L4 | L4 Expert | Look up any command, its shortcut and its options offline. | An offline searchable reference for every command, generated from the live keymap. | S1 XC07 (Professional depth) |
| D11.L5 | L5 Specialist | Learn the scripting, plugin and SDK interfaces. | Offline interface references and examples for scripts and plugins. | A01 accepted (SDK); A02, A03 open |

### D12 Accessibility tiers

Accessibility is level-invariant: Tier 1 is never a setting, and tier-2 axes
combine with every preset at every level. D12.L1 states the protection for
every level and for every view a user meets at L1, including grids,
timelines and canvases. D12.L2 to D12.L5 only add the views that first
appear at those levels and extend the same protection to them. The
tiers are stated as ds-1.x **requirements**; ds-1.1 is unaccepted and its
baseline B8 (flashing under keyboard repeat) is one of the three defects of
the authorised UX-A11Y-DS run 3, so nothing here claims that Tier 1 passes.
B8 is read with ADR-DESIGN-OWNER-20261115 D3: the system-interface flash
limits that D3 names (WCAG 2.2 SC 2.3.1 and ITU-R BT.1702-3) apply, and the
five-second remark of BT.1702-3 is an additional risk consideration, not a
hard limit. Under D3, UX-A11Y-A1 amends the research document, the matrix
and the design criteria; until then, criteria that count Tier 1 failures
(AC-1.5, AC-X.4, AC-X.7) apply B8 with that reading.

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D12.L1 | L1 First use | Use the application with keyboard only, a screen reader, magnification or tier-2 settings from first launch, without configuring the application, in every view of L1: menus, dialogs, documents, grids, timelines and canvases. | Tier 1 baseline (B1–B12) in every theme and preset with no setting, at every level; device preferences applied automatically; the tier-2 axes of Instellingen › Toegankelijkheid apply to every application, independent of preset and density; accessible canvases, grids (one tab stop) and timelines with keyboard navigation of complex views; number fields that step with the S0 D8 keys; a single-pointer alternative to every drag. | ADR-DESIGN-OWNER-20261115 D1; DESIGN-SYSTEM §21.1, §21.2, §21.5; S1 XC06 (Beginner path and Professional depth); S0 D8, D9; WCAG 2.2 SC 2.1.1, 2.5.7 [E8] |
| D12.L2 | L2 Everyday | Use the views first met at L2 with keyboard and assistive technology: the command palette, comments, the version list, the grant list and plugin interfaces. | Tier 1 and the D12.L1 protections hold in these views; plugin interfaces built from the same components inherit Tier 1. | DESIGN-SYSTEM §21.1; inheritance [JUDGEMENT] |
| D12.L3 | L3 Proficient | Use the views first met at L3 with keyboard and assistive technology: comment threads, tracked-change review, version comparison, and style, layer and track panels. | Tier 1 and the D12.L1 protections hold in these views; changes and differences are never shown by colour alone (B11). | DESIGN-SYSTEM §21.1 |
| D12.L4 | L4 Expert | Use the views first met at L4 with keyboard and assistive technology: the keymap editor, batch and fidelity reports, preflight results and merge-conflict review. | Tier 1 and the D12.L1 protections hold in these views; reports are readable as structured text. | DESIGN-SYSTEM §21.1; report form [JUDGEMENT] |
| D12.L5 | L5 Specialist | Use the script editor, plugin test mode and interface references with keyboard and assistive technology, and build plugins and scripts whose interfaces stay accessible. | Tier 1 and the D12.L1 protections hold in these views; the SDK offers plugin and script authors the same components, so their interfaces inherit Tier 1; accessibility inspection of plugin and script interfaces in plugin test mode. | DESIGN-SYSTEM §21.1; A01 accepted (SDK); inheritance and inspection [JUDGEMENT] |

### D13 Privacy and permissions

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D13.L1 | L1 First use | Decide on access when an application first needs it, understanding what is asked. | Installation grants nothing; a prompt drawn by the trusted shell with two primary choices, "Toestaan" and "Weigeren", and a remember control that is off by default; the same prompt at every level; a remembered choice applies only to its five-field key (app, right, data, destination, scope) and never broadens; a policy check per request; no OS telemetry; voluntary application telemetry only through an explicit grant and never with a document, NAS or key capability. | Privacy contract §4, LP-13, LP-30; ADR-DESIGN-OWNER-20261115 D2; S1 XC09 (Professional depth; where XC09 differs from D2, D2 governs, section 6.1, rule 5); DESIGN-SYSTEM §11 |
| D13.L2 | L2 Everyday | See which application uses the network or microphone, review grants and revoke them. | The privacy indicator in Orbit's status area; every remembered choice listed and revocable in Instellingen › Privacy en beveiliging › Appmachtigingen. | NAMING-AND-IA §5; ADR-DESIGN-OWNER-20261115 D2 |
| D13.L3 | L3 Proficient | Rely on remembered choices for repeated identical requests; turn privacy explanations off. | Remembered choices reused only for identical requests under the D13.L1 key; "Privacyuitleg uit" (privacy explanations off) never means "alles toestaan" (allow everything) and leaves existing grants unchanged. | ADR-DESIGN-OWNER-20261115 D2; privacy contract §4.1 |
| D13.L4 | L4 Expert | Run project and batch work under bounded grants for a folder or destination. | Project and batch grants bounded to a named folder or destination under the D13.L1 key and policy check; no expert setting that skips prompts. | Batch grant scope [JUDGEMENT], needs UX-C, A03 and an ADR |
| D13.L5 | L5 Specialist | Grant scripts their own access and review what each script may do. | Scripts as separate principals with their own grants, listed and revocable like application grants (plugins from D14.L2). | Privacy contract §4, §4.1; separate principals [JUDGEMENT], A03 open |

### D14 Plugins and extensions

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D14.L1 | L1 First use | Complete first-use work without any plugin. | The L1 minimum of section 6 met by first-party code alone; no third-party code acts without a grant. | Privacy contract §4; first-party sufficiency [JUDGEMENT] |
| D14.L2 | L2 Everyday | Install a plugin or effect from a source of the user's choice, see what it may access and keep working when it fails. | Provenance shown (a signature proves origin, not trustworthiness); access requested at first use as a separate principal; per-plugin process isolation and resource limits; a crash or hang never loses the document draft; plugins never draw over trust surfaces. | Privacy contract §4; B03, K03, K04, K07 accepted as base; S1 AU14 (Professional depth); DESIGN-SYSTEM §11 |
| D14.L3 | L3 Proficient | Review, disable and remove plugins. | A plugin list per application with provenance, grants and an off switch; removing a plugin revokes its grants. | Privacy contract §4; list and revocation on removal [JUDGEMENT] |
| D14.L4 | L4 Expert | Use professional plugin standards and third-party effects and instruments. | Hosting of standard plugin formats and third-party effects under the D14.L2 isolation, with per-plugin presets and crash reports. | S1 AU14 (Professional depth, from L2); plugin standards in S1 §2; per-plugin presets and crash reports [JUDGEMENT] |
| D14.L5 | L5 Specialist | Develop, test and distribute plugins. | An SDK, developer documentation and a sandboxed test mode. | A01 accepted; A02, A03 open; S1 VI14 (Professional depth) |

### D15 Trust signals

| Cell | Level | The user can | The system must provide | Basis |
|---|---|---|---|---|
| D15.L1 | L1 First use | See where a document is stored, whether it is saved and which application uses the network, microphone or camera. | The place badge (shape, icon, storage label); the version indicator states; the privacy instrument; errors that state what happened and what happens next; no red status colour, never colour alone. | S0 D1, D7; DESIGN-SYSTEM §11; NAMING-AND-IA §6 |
| D15.L2 | L2 Everyday | See what an export loses and whether sharing leaves the device. | The export dialog's "what will not survive"; the prompt's notice that permitted content can be sent to an internet service. | S0 D8; DESIGN-SYSTEM §11 |
| D15.L3 | L3 Proficient | See link, colour-profile and fidelity status. | The status of the selected link, the colour profile in use and fidelity notices in the status bar or inspector. | S1 GR10 (Beginner path, from L1), GR32, WP24 (Beginner path, from L2); status placement [JUDGEMENT] |
| D15.L4 | L4 Expert | Verify signatures and change attribution, and review the status of every linked file. | Certificate validation with timestamps; attributed changes with per-change review; a link status overview. | S1 PD07, WP16, GR32 (Professional depth) |
| D15.L5 | L5 Specialist | Verify the provenance of scripts, fonts, templates and shared libraries, reproduce batch outputs and audit changes. | Provenance display for these components as for plugins (D14.L2); per-batch reports; audit history. | B03 accepted as base; privacy contract §4; S1 DB06 (Professional depth); batch reports [JUDGEMENT] |

## 5. Progression rules

- **RULE-1 Levels are never stored, inferred or shown.** No setting, profile
  field, heuristic or telemetry assigns a maturity level, and no interface
  text names one. Users choose presets, density, keymaps and layouts; the
  system does not choose for them. [JUDGEMENT]; privacy contract LP-13.
- **RULE-2 Progressive disclosure never removes a capability.** Every command
  stays in the menu bar and the palette in every preset (S0 D6). Presets
  change layout, tools and density only. Panels have at most two levels
  ([SOURCE] Nielsen: designs with more than two disclosure levels typically
  have low usability, and the route to advanced features must be visible
  [E6]).
- **RULE-3 No mode hides data.** A preset may leave a feature's tools out of the
  layout but never its content. A document opened at `essential` still shows,
  or visibly signals with a one-step route, its tracked changes, comments,
  hidden layers, linked resources, embedded automation and metadata.
  [JUDGEMENT], following Shneiderman's question on compatibility of output
  across layers [E5].
- **RULE-4 No level switches off safety.** Tier 1 accessibility, the
  system-interface flash baseline (ADR-DESIGN-OWNER-20261115 D3), the
  permission prompt, the Security Floor, truthful save states, autosave and
  recovery, validation and isolation of converters and plugins are identical
  in every preset and at every level.
- **RULE-5 One document at every level.** Switching preset or density changes no
  document data and creates no revision (S0 D6: presets never change
  documents). A document made at `essential` opens unchanged at `complete`.
- **RULE-6 Preferences are user-controlled and reversible.** Every preference
  (preset, density, layout, keymap, learning aids) can be undone, reset per
  item and reset as a whole, as Instellingen › Toegankelijkheid does for the
  accessibility axes (DESIGN-SYSTEM §21.5). Preferences are kept per user and
  application (S0 D6), keymaps export as a delta (S0 D5), and a guest session
  leaves nothing (PROFILES-AND-USERS §3).
- **RULE-7 Command structure is static; personalisation is adaptable.** Menus do
  not reorder or hide items by usage. Users may personalise; the system does
  not adapt on its own. [SOURCE] In a 27-participant study a static menu was
  significantly faster than an adaptive one, and most participants preferred
  the user-controlled adaptable menu (15 of 27 ranked it first) [E3]. In a
  field study with 20 participants over about six weeks, a personal interface
  that users could toggle to the full interface improved perceived
  navigation, learning, satisfaction and control compared with adaptive menus
  [E4]. A palette may show a separate, clearable group of recent commands
  without reordering the full list [JUDGEMENT].
- **RULE-8 Learning aids are local and user-controlled.** Aids that need no usage
  data (shortcuts shown in menus, tooltips and the palette) are on by default.
  Aids derived from the user's own usage (for example suggestions ordered by
  the user's frequency of use, or a shortcut suggested after repeated menu
  use) run only on the device, are off until the user turns them on, can be
  cleared, never leave the device, never form a profile and never reorder the
  menus or the full palette list (RULE-7). [JUDGEMENT]; privacy contract LP-13.
  [SOURCE] For comparison, Blur, the system in [E1], observes each pointer
  action and shows its command equivalent as transient feedback, and orders
  its command recommendations by frequency of use. S2 adopts neither as a
  default [JUDGEMENT]. Whether usage-derived aids are offered at all stays a design
  question for task trials (section 8.2, item 13).
- **RULE-9 Entry at any level.** First launch at `essential` stays S0 D6's
  decision. A migrating professional chooses a familiar keymap preset at first
  start (S1 XC05) and switches to `complete` in one step; the choice is kept.
  The `essential` default remains subject to S0 open item 10 (task trials).
- **RULE-10 Moving down is as easy as moving up.** "Workspace: Essential", Reset
  layout and Reset keymap are always one command away, and none changes
  documents.
- **RULE-11 Axes are independent.** Maturity level, disclosure preset, density
  and the tier-2 accessibility axes combine freely; no combination
  unintentionally disables another (ADR-DESIGN-OWNER-20261115 D1, review
  criteria; DESIGN-SYSTEM §21.4). The contrast-theme bundle is the documented
  exception: there Transparantie and Prikkels change nothing and Contrast only
  thickens edges (DESIGN-SYSTEM §21.3, O-A7).
- **RULE-12 Every slow route shows the fast route.** Menus, tooltips and the
  palette show the shortcut; the inspector shows the exact value of a dragged
  property; a recorded batch shows its steps. This serves [E1]'s three goals
  (prompt the switch, lower its cost, raise the ceiling) without monitoring
  the user.

## 6. Per-discipline mapping

### 6.1 How to read the mapping

For every S1 row the tables state two levels [JUDGEMENT]:

- **Beginner path required from**: from this level, the row's S1 *Beginner
  path* is part of the minimum.
- **Professional depth required from**: from this level, the row's S1
  *Professional depth* is also part of the minimum.

The minimum for level N in a discipline is the Beginner path of every row
whose first level is at or below N plus the Professional depth of every row
whose second level is at or below N; the summary table under each discipline
lists both sets by row identifier. An application's minimum is that of its
discipline plus the cross-cutting rows (section 6.14) and the rows that S1
§5 proposes as shared by every surface (GR34–GR37 in section 6.9, UX06 and
UX07 in section 6.12), at the levels stated there. Section 4 cites these
columns at the same levels (section 4, introduction). Assignment rules:

1. Correctness, safety, recovery, accessibility and permission rows enter at
   L1 (for example SS04, SS19, GR09, GR10, XC01, XC02, XC06, XC09). Where the
   Professional depth is itself a safety or accessibility property, it is
   required as early as the row (WP01, XC06, XC09 at L1; AU14 plugin
   isolation at L2, when plugins first appear).
2. Rows whose S1 Beginner path is "—" (WP25, SS09, GR13, TD16) have no
   beginner-facing control; they enter only through their Professional depth.
3. S1 *Priority* (P0–P3) is build order and is independent of the level: WP04
   is P0 and enters at L1 because its Beginner path is automatic,
   language-aware typography.
4. The "Evidence level" column is copied from S1 unchanged and checked by
   script (section 8.2, item 3). Of the 216 rows, 14 are *prototyped (host)* and
   202 *proposed*; none is *passed (native)*. The level is S1's row-level
   level; clause (a) of section 2.1 needs evidence per required column.
5. XC09 is read with ADR-DESIGN-OWNER-20261115 D2. Its S1 Beginner path names
   the three-choice prompt that D2 superseded, and its Professional depth keys
   grants on four fields where D2 uses five (app, right, data, destination,
   scope). Wherever XC09 differs from D2, D2 and cells D13.L1 and D13.L3
   govern clause (a) of section 2.1 for XC09; S1 is not changed
   (section 8.2, item 2).

<!-- S6:BEGIN -->
### 6.2 Word processing, typography, references, revision tracking, publishing and complex layout (S1 §3.1.1)

| Capability (S1 row) | Evidence level (S1, unchanged) | Beginner path required from | Professional depth required from |
|---|---|---|---|
| WP01 Plain-text document edit, save and reopen | prototyped (host) | L1 First use | L1 First use |
| WP02 Rich-text formatting | proposed | L1 First use | L3 Proficient |
| WP03 Style system | proposed | L2 Everyday | L4 Expert |
| WP04 Advanced typography | proposed | L1 First use | L4 Expert |
| WP05 Complex scripts and bidirectional text | proposed | L1 First use | L4 Expert |
| WP06 Pagination and page setup | proposed | L2 Everyday | L4 Expert |
| WP07 Document tables | proposed | L2 Everyday | L3 Proficient |
| WP08 Images and floating objects | proposed | L2 Everyday | L3 Proficient |
| WP09 Fields and cross-references | proposed | L2 Everyday | L4 Expert |
| WP10 Multilevel numbering and outline navigation | proposed | L2 Everyday | L4 Expert |
| WP11 Tables of contents, indexes and lists of figures | proposed | L3 Proficient | L4 Expert |
| WP12 Footnotes and endnotes | proposed | L3 Proficient | L4 Expert |
| WP13 Citations and bibliography | proposed | L3 Proficient | L4 Expert |
| WP14 Equations | proposed | L3 Proficient | L4 Expert |
| WP15 Comments and threaded review | proposed | L2 Everyday | L3 Proficient |
| WP16 Revision tracking and document comparison | proposed | L3 Proficient | L4 Expert |
| WP17 Version history preview and restore | prototyped (host) | L1 First use | L3 Proficient |
| WP18 Proofing tools | proposed | L1 First use | L3 Proficient |
| WP19 Templates and mail merge | proposed | L1 First use | L4 Expert |
| WP20 Long-document publishing | proposed | L4 Expert | L5 Specialist |
| WP21 Accessible document authoring | proposed | L1 First use | L3 Proficient |
| WP22 DOCX import (plain text) | prototyped (host) | L1 First use | L3 Proficient |
| WP23 DOCX export (plain text) | prototyped (host) | L1 First use | L3 Proficient |
| WP24 Full-fidelity DOCX and ODF round trip | proposed | L2 Everyday | L4 Expert |
| WP25 Large-document performance | proposed | — | L4 Expert |
| WP26 Document automation | proposed | L4 Expert | L5 Specialist |
| WP27 Print and PDF output of documents | proposed | L1 First use | L4 Expert |
| WP28 Find and replace | proposed | L1 First use | L3 Proficient |

| Level | Beginner path of | Professional depth of |
|---|---|---|
| L1 First use | WP01, WP02, WP04, WP05, WP17, WP18, WP19, WP21, WP22, WP23, WP27, WP28 | WP01 |
| L2 Everyday | WP03, WP06, WP07, WP08, WP09, WP10, WP15, WP24 | — |
| L3 Proficient | WP11, WP12, WP13, WP14, WP16 | WP02, WP07, WP08, WP15, WP17, WP18, WP21, WP22, WP23, WP28 |
| L4 Expert | WP20, WP26 | WP03, WP04, WP05, WP06, WP09, WP10, WP11, WP12, WP13, WP14, WP16, WP19, WP24, WP25, WP27 |
| L5 Specialist | — | WP20, WP26 |

### 6.3 Spreadsheets (S1 §3.1.2)

| Capability (S1 row) | Evidence level (S1, unchanged) | Beginner path required from | Professional depth required from |
|---|---|---|---|
| SS01 CSV table import, cell edit and export | prototyped (host) | L1 First use | L3 Proficient |
| SS02 Formula entry and dependency recalculation | prototyped (host) | L1 First use | L4 Expert |
| SS03 Function library | proposed | L2 Everyday | L4 Expert |
| SS04 Compatible calculation semantics | proposed | L1 First use | L4 Expert |
| SS05 Reference model | proposed | L1 First use | L3 Proficient |
| SS06 Number, date and currency formats | proposed | L1 First use | L3 Proficient |
| SS07 Typed cells and data validation | proposed | L2 Everyday | L3 Proficient |
| SS08 Sort, filter and conditional formatting | proposed | L2 Everyday | L3 Proficient |
| SS09 Large datasets | proposed | — | L4 Expert |
| SS10 Pivot tables and summarisation | proposed | L3 Proficient | L4 Expert |
| SS11 Charts | proposed | L2 Everyday | L4 Expert |
| SS12 Data import and refreshable connections | proposed | L3 Proficient | L4 Expert |
| SS13 What-if analysis and statistics | proposed | L3 Proficient | L5 Specialist |
| SS14 Spreadsheet automation | proposed | L4 Expert | L5 Specialist |
| SS15 XLSX and ODS round trip | proposed | L1 First use | L4 Expert |
| SS16 Undo and redo | proposed | L1 First use | L3 Proficient |
| SS17 Workbook collaboration | proposed | L2 Everyday | L4 Expert |
| SS18 Formula auditing and protection | proposed | L3 Proficient | L4 Expert |
| SS19 Formula-injection safety on exchange | proposed | L1 First use | L4 Expert |
| SS20 Find and replace in workbooks | proposed | L1 First use | L3 Proficient |

| Level | Beginner path of | Professional depth of |
|---|---|---|
| L1 First use | SS01, SS02, SS04, SS05, SS06, SS15, SS16, SS19, SS20 | — |
| L2 Everyday | SS03, SS07, SS08, SS11, SS17 | — |
| L3 Proficient | SS10, SS12, SS13, SS18 | SS01, SS05, SS06, SS07, SS08, SS16, SS20 |
| L4 Expert | SS14 | SS02, SS03, SS04, SS09, SS10, SS11, SS12, SS15, SS17, SS18, SS19 |
| L5 Specialist | — | SS13, SS14 |

### 6.4 Presentations (S1 §3.1.3)

| Capability (S1 row) | Evidence level (S1, unchanged) | Beginner path required from | Professional depth required from |
|---|---|---|---|
| PR01 Slide navigation and preview | prototyped (host) | L1 First use | L3 Proficient |
| PR02 Slide raster frame export (PPM) | prototyped (host) | L2 Everyday | L4 Expert |
| PR03 Slide editing with layouts and placeholders | proposed | L1 First use | L4 Expert |
| PR04 Themes and templates | proposed | L1 First use | L4 Expert |
| PR05 Precise layout controls | proposed | L1 First use | L3 Proficient |
| PR06 Animations and transitions | proposed | L2 Everyday | L4 Expert |
| PR07 Embedded media | proposed | L2 Everyday | L4 Expert |
| PR08 Presenter tools | proposed | L1 First use | L3 Proficient |
| PR09 Charts, tables and diagrams in slides | proposed | L2 Everyday | L4 Expert |
| PR10 Deck export to PDF and video | proposed | L1 First use | L4 Expert |
| PR11 PPTX and ODP import and export | proposed | L1 First use | L4 Expert |
| PR12 Accessible slides | proposed | L1 First use | L3 Proficient |
| PR13 Slide image export at chosen resolution and colour space | proposed | L2 Everyday | L4 Expert |

| Level | Beginner path of | Professional depth of |
|---|---|---|
| L1 First use | PR01, PR03, PR04, PR05, PR08, PR10, PR11, PR12 | — |
| L2 Everyday | PR02, PR06, PR07, PR09, PR13 | — |
| L3 Proficient | — | PR01, PR05, PR08, PR12 |
| L4 Expert | — | PR02, PR03, PR04, PR06, PR07, PR09, PR10, PR11, PR13 |
| L5 Specialist | — | — |

### 6.5 Notebooks, knowledge management, diagrams, whiteboards and research organisation (S1 §3.1.4)

| Capability (S1 row) | Evidence level (S1, unchanged) | Beginner path required from | Professional depth required from |
|---|---|---|---|
| NK01 Plain-text notes in collections | prototyped (host) | L1 First use | L3 Proficient |
| NK02 Rich notes | proposed | L1 First use | L3 Proficient |
| NK03 Local document-name search | prototyped (host) | L1 First use | L3 Proficient |
| NK04 Links and backlinks | proposed | L2 Everyday | L4 Expert |
| NK05 Research organisation | proposed | L3 Proficient | L4 Expert |
| NK06 Diagrams | proposed | L2 Everyday | L4 Expert |
| NK07 Whiteboards | proposed | L2 Everyday | L4 Expert |
| NK08 Handwriting and pen input in notes | proposed | L2 Everyday | L4 Expert |
| NK09 Offline-first notebook sync | proposed | L1 First use | L3 Proficient |
| NK10 Note import and export | proposed | L2 Everyday | L4 Expert |
| NK11 Full-text and metadata search across notes and documents | proposed | L1 First use | L3 Proficient |

| Level | Beginner path of | Professional depth of |
|---|---|---|
| L1 First use | NK01, NK02, NK03, NK09, NK11 | — |
| L2 Everyday | NK04, NK06, NK07, NK08, NK10 | — |
| L3 Proficient | NK05 | NK01, NK02, NK03, NK09, NK11 |
| L4 Expert | — | NK04, NK05, NK06, NK07, NK08, NK10 |
| L5 Specialist | — | — |

### 6.6 Email, calendars, contacts, scheduling, tasks and collaboration (S1 §3.1.5)

| Capability (S1 row) | Evidence level (S1, unchanged) | Beginner path required from | Professional depth required from |
|---|---|---|---|
| EM01 Mail accounts and mailbox sync | proposed | L1 First use | L4 Expert |
| EM02 Reading, threading, search and rules | proposed | L1 First use | L3 Proficient |
| EM03 Composing | proposed | L1 First use | L3 Proficient |
| EM04 Mail security | proposed | L1 First use | L4 Expert |
| EM05 Calendars | proposed | L1 First use | L3 Proficient |
| EM06 Scheduling across people and resources | proposed | L2 Everyday | L4 Expert |
| EM07 Contacts | proposed | L1 First use | L3 Proficient |
| EM08 Tasks and to-dos | proposed | L1 First use | L3 Proficient |
| EM09 Reminders and notifications | proposed | L1 First use | L3 Proficient |
| EM10 Sharing and team collaboration | proposed | L2 Everyday | L4 Expert |
| EM11 Real-time co-editing | proposed | L3 Proficient | L4 Expert |

| Level | Beginner path of | Professional depth of |
|---|---|---|
| L1 First use | EM01, EM02, EM03, EM04, EM05, EM07, EM08, EM09 | — |
| L2 Everyday | EM06, EM10 | — |
| L3 Proficient | EM11 | EM02, EM03, EM05, EM07, EM08, EM09 |
| L4 Expert | — | EM01, EM04, EM06, EM10, EM11 |
| L5 Specialist | — | — |

### 6.7 Databases, forms, project management and business workflows (S1 §3.1.6)

| Capability (S1 row) | Evidence level (S1, unchanged) | Beginner path required from | Professional depth required from |
|---|---|---|---|
| DB01 Desktop database | proposed | L3 Proficient | L5 Specialist |
| DB02 Forms and surveys | proposed | L2 Everyday | L4 Expert |
| DB03 Reports from data | proposed | L3 Proficient | L4 Expert |
| DB04 List and tracker views | proposed | L1 First use | L3 Proficient |
| DB05 Project scheduling | proposed | L3 Proficient | L5 Specialist |
| DB06 Business workflows and approvals | proposed | L3 Proficient | L5 Specialist |
| DB07 External data connectors | proposed | L4 Expert | L5 Specialist |

| Level | Beginner path of | Professional depth of |
|---|---|---|
| L1 First use | DB04 | — |
| L2 Everyday | DB02 | — |
| L3 Proficient | DB01, DB03, DB05, DB06 | DB04 |
| L4 Expert | DB07 | DB02, DB03 |
| L5 Specialist | — | DB01, DB05, DB06, DB07 |

### 6.8 PDF creation, annotation, editing, signing and export (S1 §3.1.7)

| Capability (S1 row) | Evidence level (S1, unchanged) | Beginner path required from | Professional depth required from |
|---|---|---|---|
| PD01 PDF viewing | proposed | L1 First use | L4 Expert |
| PD02 PDF creation from any application | proposed | L1 First use | L4 Expert |
| PD03 PDF annotation | proposed | L1 First use | L3 Proficient |
| PD04 Page organisation | proposed | L2 Everyday | L4 Expert |
| PD05 PDF content editing | proposed | L2 Everyday | L4 Expert |
| PD06 PDF forms | proposed | L1 First use | L4 Expert |
| PD07 Digital signatures | proposed | L2 Everyday | L4 Expert |
| PD08 Redaction and sanitisation | proposed | L3 Proficient | L4 Expert |
| PD09 Text recognition (OCR) | proposed | L2 Everyday | L4 Expert |
| PD10 Standards-conformant professional export | proposed | L3 Proficient | L4 Expert |
| PD11 PDF comparison | proposed | L3 Proficient | L4 Expert |

| Level | Beginner path of | Professional depth of |
|---|---|---|
| L1 First use | PD01, PD02, PD03, PD06 | — |
| L2 Everyday | PD04, PD05, PD07, PD09 | — |
| L3 Proficient | PD08, PD10, PD11 | PD03 |
| L4 Expert | — | PD01, PD02, PD04, PD05, PD06, PD07, PD08, PD09, PD10, PD11 |
| L5 Specialist | — | — |

### 6.9 Graphics, photography and publishing (S1 §3.2)

| Capability (S1 row) | Evidence level (S1, unchanged) | Beginner path required from | Professional depth required from |
|---|---|---|---|
| GR01 Image viewing with orientation, fit, zoom and pan | prototyped (host) | L1 First use | L3 Proficient |
| GR02 Rectangular crop to a new file | prototyped (host) | L1 First use | L3 Proficient |
| GR03 Raster export to BMP | prototyped (host) | L1 First use | L4 Expert |
| GR04 Layers and groups | proposed | L2 Everyday | L4 Expert |
| GR05 Masks | proposed | L3 Proficient | L4 Expert |
| GR06 Blend modes and compositing | proposed | L3 Proficient | L4 Expert |
| GR07 Selection tools | proposed | L2 Everyday | L4 Expert |
| GR08 Non-destructive adjustments and effects | proposed | L1 First use | L3 Proficient |
| GR09 High bit depth and HDR images | proposed | L1 First use | L4 Expert |
| GR10 Colour profiles and soft-proofing | proposed | L1 First use | L4 Expert |
| GR11 Retouching | proposed | L2 Everyday | L4 Expert |
| GR12 Undo history and snapshots | proposed | L1 First use | L3 Proficient |
| GR13 Large-canvas performance | proposed | — | L4 Expert |
| GR14 Layered-file interchange | proposed | L1 First use | L4 Expert |
| GR15 RAW decoding and development | proposed | L2 Everyday | L4 Expert |
| GR16 Photo cataloguing | proposed | L2 Everyday | L4 Expert |
| GR17 Photo metadata read, write and preserve | proposed | L1 First use | L4 Expert |
| GR18 Batch processing and presets | proposed | L2 Everyday | L5 Specialist |
| GR19 Advanced colour correction | proposed | L1 First use | L4 Expert |
| GR20 Vector paths and shapes | proposed | L1 First use | L3 Proficient |
| GR21 Precision geometry | proposed | L1 First use | L3 Proficient |
| GR22 Vector typography | proposed | L1 First use | L4 Expert |
| GR23 Symbols in illustration | proposed | L3 Proficient | L4 Expert |
| GR24 Illustration tools | proposed | L2 Everyday | L4 Expert |
| GR25 Vector interchange | proposed | L1 First use | L4 Expert |
| GR26 Painting brush engine | proposed | L1 First use | L4 Expert |
| GR27 Pen and tablet input | proposed | L1 First use | L3 Proficient |
| GR28 Painting canvas navigation | proposed | L1 First use | L3 Proficient |
| GR29 Page layout | proposed | L2 Everyday | L4 Expert |
| GR30 Publishing typography | proposed | L2 Everyday | L4 Expert |
| GR31 Prepress and print production | proposed | L3 Proficient | L4 Expert |
| GR32 Linked resources and relinking | proposed | L2 Everyday | L4 Expert |
| GR33 Digital publishing | proposed | L3 Proficient | L5 Specialist |
| GR34 Shared palettes and swatches | proposed | L1 First use | L4 Expert |
| GR35 Font management | proposed | L1 First use | L4 Expert |
| GR36 Reusable components across applications | proposed | L3 Proficient | L4 Expert |
| GR37 Brand kits across applications | proposed | L2 Everyday | L5 Specialist |
| GR38 Raster export with format, bit-depth, profile and metadata control | proposed | L1 First use | L4 Expert |

| Level | Beginner path of | Professional depth of |
|---|---|---|
| L1 First use | GR01, GR02, GR03, GR08, GR09, GR10, GR12, GR14, GR17, GR19, GR20, GR21, GR22, GR25, GR26, GR27, GR28, GR34, GR35, GR38 | — |
| L2 Everyday | GR04, GR07, GR11, GR15, GR16, GR18, GR24, GR29, GR30, GR32, GR37 | — |
| L3 Proficient | GR05, GR06, GR23, GR31, GR33, GR36 | GR01, GR02, GR08, GR12, GR20, GR21, GR27, GR28 |
| L4 Expert | — | GR03, GR04, GR05, GR06, GR07, GR09, GR10, GR11, GR13, GR14, GR15, GR16, GR17, GR19, GR22, GR23, GR24, GR25, GR26, GR29, GR30, GR31, GR32, GR34, GR35, GR36, GR38 |
| L5 Specialist | — | GR18, GR33, GR37 |

### 6.10 Video, motion graphics and animation (S1 §3.3)

| Capability (S1 row) | Evidence level (S1, unchanged) | Beginner path required from | Professional depth required from |
|---|---|---|---|
| VI01 Media import and frame-accurate playback | proposed | L1 First use | L4 Expert |
| VI02 Multitrack timeline editing | proposed | L1 First use | L3 Proficient |
| VI03 Multicamera editing | proposed | L3 Proficient | L4 Expert |
| VI04 Timeline precision | proposed | L1 First use | L3 Proficient |
| VI05 Proxies and optimised media | proposed | L1 First use | L4 Expert |
| VI06 Transitions and keyframes | proposed | L1 First use | L4 Expert |
| VI07 Non-destructive projects and media relinking | proposed | L1 First use | L4 Expert |
| VI08 Colour grading | proposed | L2 Everyday | L4 Expert |
| VI09 HDR and colour-managed video pipeline | proposed | L1 First use | L5 Specialist |
| VI10 Professional export and delivery | proposed | L1 First use | L4 Expert |
| VI11 Captions and subtitles | proposed | L2 Everyday | L4 Expert |
| VI12 Motion graphics | proposed | L2 Everyday | L5 Specialist |
| VI13 Compositing | proposed | L3 Proficient | L5 Specialist |
| VI14 Video effects and third-party effects | proposed | L2 Everyday | L5 Specialist |
| VI15 2D character animation | proposed | L3 Proficient | L5 Specialist |
| VI16 2D and 3D integration | proposed | L3 Proficient | L5 Specialist |
| VI17 Encoding and transcoding | proposed | L2 Everyday | L4 Expert |
| VI18 Project interchange | proposed | L3 Proficient | L4 Expert |
| VI19 Render queues | proposed | L2 Everyday | L5 Specialist |
| VI20 Audio for video | proposed | L1 First use | L4 Expert |
| VI21 Vertical and social-format delivery | proposed | L1 First use | L4 Expert |

| Level | Beginner path of | Professional depth of |
|---|---|---|
| L1 First use | VI01, VI02, VI04, VI05, VI06, VI07, VI09, VI10, VI20, VI21 | — |
| L2 Everyday | VI08, VI11, VI12, VI14, VI17, VI19 | — |
| L3 Proficient | VI03, VI13, VI15, VI16, VI18 | VI02, VI04 |
| L4 Expert | — | VI01, VI03, VI05, VI06, VI07, VI08, VI10, VI11, VI17, VI18, VI20, VI21 |
| L5 Specialist | — | VI09, VI12, VI13, VI14, VI15, VI16, VI19 |

### 6.11 Audio production (S1 §3.4)

| Capability (S1 row) | Evidence level (S1, unchanged) | Beginner path required from | Professional depth required from |
|---|---|---|---|
| AU01 Sample-exact clip extraction | prototyped (host) | L1 First use | L3 Proficient |
| AU02 Playback and monitoring | proposed | L1 First use | L4 Expert |
| AU03 Multitrack arrangement | proposed | L1 First use | L4 Expert |
| AU04 Recording | proposed | L1 First use | L4 Expert |
| AU05 Mixing | proposed | L2 Everyday | L4 Expert |
| AU06 Automation | proposed | L3 Proficient | L4 Expert |
| AU07 Routing | proposed | L3 Proficient | L5 Specialist |
| AU08 Effects | proposed | L2 Everyday | L4 Expert |
| AU09 Mastering | proposed | L3 Proficient | L5 Specialist |
| AU10 Waveform editing | proposed | L1 First use | L3 Proficient |
| AU11 Spectral editing and repair | proposed | L3 Proficient | L5 Specialist |
| AU12 MIDI sequencing and editing | proposed | L2 Everyday | L4 Expert |
| AU13 Virtual instruments | proposed | L2 Everyday | L4 Expert |
| AU14 Audio plugin hosting | proposed | L2 Everyday | L2 Everyday |
| AU15 Sample-rate and sample-format handling | proposed | L1 First use | L4 Expert |
| AU16 Latency management | proposed | L1 First use | L4 Expert |
| AU17 Audio export and delivery | proposed | L1 First use | L4 Expert |
| AU18 Session interchange | proposed | L3 Proficient | L5 Specialist |
| AU19 Control surfaces and controllers | proposed | L3 Proficient | L4 Expert |

| Level | Beginner path of | Professional depth of |
|---|---|---|
| L1 First use | AU01, AU02, AU03, AU04, AU10, AU15, AU16, AU17 | — |
| L2 Everyday | AU05, AU08, AU12, AU13, AU14 | AU14 |
| L3 Proficient | AU06, AU07, AU09, AU11, AU18, AU19 | AU01, AU10 |
| L4 Expert | — | AU02, AU03, AU04, AU05, AU06, AU08, AU12, AU13, AU15, AU16, AU17, AU19 |
| L5 Specialist | — | AU07, AU09, AU11, AU18 |

### 6.12 UI/UX and interactive design (S1 §3.5)

| Capability (S1 row) | Evidence level (S1, unchanged) | Beginner path required from | Professional depth required from |
|---|---|---|---|
| UX01 Wireframing | proposed | L1 First use | L3 Proficient |
| UX02 High-fidelity interface design | proposed | L2 Everyday | L4 Expert |
| UX03 Responsive layouts | proposed | L2 Everyday | L4 Expert |
| UX04 Reusable components | proposed | L2 Everyday | L4 Expert |
| UX05 Interactive prototyping and transitions | proposed | L2 Everyday | L4 Expert |
| UX06 Design tokens and variables | proposed | L3 Proficient | L4 Expert |
| UX07 Component libraries and design systems | proposed | L3 Proficient | L5 Specialist |
| UX08 Accessibility inspection | proposed | L1 First use | L3 Proficient |
| UX09 Developer handoff | proposed | L3 Proficient | L4 Expert |
| UX10 Design collaboration | proposed | L2 Everyday | L4 Expert |
| UX11 Design version management | proposed | L1 First use | L4 Expert |
| UX12 Design-file interchange | proposed | L2 Everyday | L5 Specialist |

| Level | Beginner path of | Professional depth of |
|---|---|---|
| L1 First use | UX01, UX08, UX11 | — |
| L2 Everyday | UX02, UX03, UX04, UX05, UX10, UX12 | — |
| L3 Proficient | UX06, UX07, UX09 | UX01, UX08 |
| L4 Expert | — | UX02, UX03, UX04, UX05, UX06, UX09, UX10, UX11 |
| L5 Specialist | — | UX07, UX12 |

### 6.13 3D and spatial creation (S1 §3.6)

| Capability (S1 row) | Evidence level (S1, unchanged) | Beginner path required from | Professional depth required from |
|---|---|---|---|
| TD01 Mesh modelling | proposed | L1 First use | L4 Expert |
| TD02 Precision modelling | proposed | L1 First use | L3 Proficient |
| TD03 Sculpting | proposed | L2 Everyday | L4 Expert |
| TD04 Scene management | proposed | L1 First use | L4 Expert |
| TD05 UV editing | proposed | L3 Proficient | L4 Expert |
| TD06 Materials and physically based rendering | proposed | L1 First use | L4 Expert |
| TD07 Textures and baking | proposed | L3 Proficient | L4 Expert |
| TD08 Lighting and cameras | proposed | L1 First use | L4 Expert |
| TD09 3D animation | proposed | L2 Everyday | L4 Expert |
| TD10 Rigging | proposed | L3 Proficient | L4 Expert |
| TD11 Rendering | proposed | L1 First use | L5 Specialist |
| TD12 Procedural and node-based workflows | proposed | L2 Everyday | L4 Expert |
| TD13 3D painting and material authoring | proposed | L3 Proficient | L4 Expert |
| TD14 3D asset management | proposed | L2 Everyday | L4 Expert |
| TD15 Pipeline interoperability | proposed | L2 Everyday | L5 Specialist |
| TD16 Spatial preview | proposed | — | L4 Expert |

| Level | Beginner path of | Professional depth of |
|---|---|---|
| L1 First use | TD01, TD02, TD04, TD06, TD08, TD11 | — |
| L2 Everyday | TD03, TD09, TD12, TD14, TD15 | — |
| L3 Proficient | TD05, TD07, TD10, TD13 | TD02 |
| L4 Expert | — | TD01, TD03, TD04, TD05, TD06, TD07, TD08, TD09, TD10, TD12, TD13, TD14, TD16 |
| L5 Specialist | — | TD11, TD15 |

### 6.14 Cross-cutting suite capabilities (S1 §3.7)

| Capability (S1 row) | Evidence level (S1, unchanged) | Beginner path required from | Professional depth required from |
|---|---|---|---|
| XC01 Autosave and backup copies | proposed | L1 First use | L4 Expert |
| XC02 Crash and power-loss recovery of open work | proposed | L1 First use | L3 Proficient |
| XC03 Configurable workspaces | proposed | L1 First use | L4 Expert |
| XC04 Command and shortcut customisation | proposed | L1 First use | L4 Expert |
| XC05 Familiar keymap presets and import | proposed | L1 First use | L4 Expert |
| XC06 Accessible suite applications | proposed | L1 First use | L1 First use |
| XC07 In-app learning and contextual help | proposed | L1 First use | L4 Expert |
| XC08 Education and classroom use | proposed | L2 Everyday | L4 Expert |
| XC09 Revocable persistent app permissions | proposed | L1 First use (read with ADR-DESIGN-OWNER-20261115 D2; section 6.1, rule 5) | L1 First use |

| Level | Beginner path of | Professional depth of |
|---|---|---|
| L1 First use | XC01, XC02, XC03, XC04, XC05, XC06, XC07, XC09 | XC06, XC09 |
| L2 Everyday | XC08 | — |
| L3 Proficient | — | XC02 |
| L4 Expert | — | XC01, XC03, XC04, XC05, XC07, XC08 |
| L5 Specialist | — | — |
<!-- S6:END -->

## 7. Acceptance criteria

All criteria are *proposed*. Thresholds, participant numbers and corpus sizes
are the author's [JUDGEMENT]; U06 ("polish validated with users",
open) and U02 (interaction budgets, open) own the product budgets, and the
project owner agrees them (section 8.1, O1). User-test metrics follow Grossman
et al.'s task metrics: T1 optimal completion, T2 completion without help, T4
decrease in errors and T5 time to completion [E2]. Each discipline runs its own
scenario variant, drawn from the application's section 6 minimum (section
6.1) and the section 4 cells that bind it under the applicability rule
(section 4, introduction). A step is omitted at the level being claimed when
the clauses it exercises do not bind the application there under that rule,
and applies from the level at which they bind (as AC-3.3 and AC-4.3 word
it); a step that exercises only cells above the level being claimed does not
apply unless it checks a protection (section 2.1, clause (c)). A level passes
when every discipline that claims it passes. Participant numbers are at least
eight per discipline and level, recruited by observable behaviour (section
3), never by diagnosis. "User test needed" is "yes" when the criterion cannot be decided
without participants. "Cells and rules tested" names the section 4 cells,
section 5 rules and section 3 level signals ("L2 signal") a criterion tests.
A cell is marked "(part)" in every criterion that names it when those
criteria together leave one of its "system must provide" clauses
unexercised; section 8.2, item 14 lists those cells and the cells that no
criterion names. A cell named without "(part)" is covered by the criteria
that name it taken together, not necessarily by each one alone.

| ID | Level | Scenario | Measure and proposed threshold | Method | User test needed | Cells and rules tested | Status |
|---|---|---|---|---|---|---|---|
| AC-1.1 | L1 First use | Participants with no prior use of the application create or open a document, make a stated change, save it, close the application and reopen the document from recent items or Files. | Completion without help (T2) ≥ 80 %; median time ≤ 5 min; no lost change. | Task trial | yes | D01.L1 (part), D02.L1 | proposed |
| AC-1.2 | L1 First use | The same participants delete a named part (paragraph, layer or clip), undo the deletion, restore an earlier version from the version list, then delete the document and restore it from the Prullenbak. | ≥ 80 % recover all three without help; the current version is never lost. | Task trial | yes | D09.L1 (part), D10.L1 (part), D10.L2 | proposed |
| AC-1.3 | L1 First use | After AC-1.1, participants say where the document is stored and whether it is saved. | ≥ 80 % answer both correctly from the place badge and version indicator. | Task trial, post-task question | yes | D15.L1 (part) | proposed |
| AC-1.4 | L1 First use | Launch each application for the first time, then list every command reachable from the menu bar and the palette, with the shortcut shown beside it, in `essential`, `extended` and `complete`, and the nesting depth of every panel. | The application opens at `essential` and `standard`; the three command sets are identical (0 differences) and every shown shortcut matches the live keymap; no panel nests deeper than two levels (S0 D6, RULE-2). | Automated inventory | no | D01.L1 (part), D01.L2, D03.L1, RULE-2 | proposed |
| AC-1.5 | L1 First use | Run the AC-1.1 scenario keyboard-only in dark, light and contrast with no setting, then run the Tier 1 checks in the `essential` preset. | Scenario completes keyboard-only; 0 Tier 1 (B1–B12) failures. | Automated and inspection | no | D02.L1, D12.L1 | proposed |
| AC-1.6 | L1 First use | Capture network traffic and inspect accounts during AC-1.1 on a fresh user account. | 0 outbound connections initiated by the application or for OS telemetry; no account created or required. | Automated packet capture | no | D01.L1 (part), D13.L1 (part) | proposed |
| AC-1.7 | L1 First use | Inject storage-full, I/O-error and conflict faults during save and during crash recovery. | "Saved" never appears without `Saved{revision}`; the draft survives 100 % of injected faults; each error message states what happened and what happens next. | Automated fault injection | no | D06.L2, D09.L1 (part), D10.L1 (part), D15.L1 (part) | proposed |
| AC-1.8 | L1 First use | Open help from every first-party application with networking disabled and search it for three command names. | Help is reached by the same menu path and shortcut in 100 % of applications and opens offline; each search finds the command's help offline. | Automated inspection | no | D11.L1 (part), D11.L2 | proposed |
| AC-1.9 | L1 First use | Send every app-standard chord (view zoom included) and every OS-owned chord to each first-party application; press single-letter command keys with focus in a canvas, a grid and a text field, with context single keys on and off. | Each app-standard chord runs the same command in 100 % of applications; 0 OS-owned chords reach an application; single-key commands fire only in the canvas with the switch on, and 0 fire in grids, text fields or with the switch off. | Automated | no | D02.L1, D03.L2 | proposed |
| AC-1.10 | L1 First use | Inventory every drag in the `essential`, `extended` and `complete` presets: handles, keyframes, clip edges, guides, splitters, layer and track order, marquee and lasso. | 100 % have a single-pointer alternative without dragging in every preset; every handle that uses the Equivalent exception has its inspector field in the same view in every preset. | Automated and inspection | no | D02.L1, D04.L1 (part), D12.L1 | proposed |
| AC-1.11 | L1 First use | Run an automated accessibility-tree check over every L1 view (menus, dialogs, document, grid, timeline and canvas) at `essential`; then participants who use a screen reader, a screen magnifier or device-level tier-2 settings perform the AC-1.1 scenario without changing any application setting. | 0 elements without accessible name, role and value; each grid is one tab stop with arrow-key navigation inside it; completion without help ≥ 80 % in each group; 0 application settings changed. | Automated tree check and task trial with assistive-technology users | yes | D12.L1 | proposed |
| AC-2.1 | L2 Everyday | Participants who completed AC-1.1 repeat the discipline's core task set two to seven days later without instruction. | Completion without help ≥ 90 %; median time at least 30 % lower than in the first session (T5); errors lower (T4). | Task trial, longitudinal | yes | L2 signal | proposed |
| AC-2.2 | L2 Everyday | Participants locate three commands they have not used, described by outcome rather than by label. | ≥ 80 % locate each within 60 s through menus, menu search or the palette. | Task trial | yes | D01.L2, D02.L2 | proposed |
| AC-2.3 | L2 Everyday | After rebinding one command, list the palette and compare every displayed shortcut in menus, palette and tooltips with the live keymap. | Every command appears in the palette with its shortcut; 0 mismatches. | Automated | no | D02.L2, D11.L2 | proposed |
| AC-2.4 | L2 Everyday | Open fixture files from other suites containing features the converter cannot keep, edit them and save back in the original format. | 100 % of known-lossy features listed before saving; 0 silent losses in a reopen comparison. | Automated with an S5 fixture corpus | no | D07.L1, D08.L1 (part), D08.L2, D09.L2 (part), D15.L2 (part) | proposed |
| AC-2.5 | L2 Everyday | Measure responsiveness on the discipline's everyday reference document on the reference system, including during saves. | Input feedback ≤ 0.1 s and command response ≤ 1 s at the 95th percentile; every operation over 1 s shows progress, over 10 s percent-done and Cancel. | Native measurement (U02) | no | D06.L1, D06.L2 | proposed |
| AC-2.6 | L2 Everyday | Participants add a comment to a passage, edit the text before that passage, then share a copy with a paired person. | ≥ 80 % without help; the comment stays on its passage after the edit in 100 % of runs; the permission prompt names the destination in 100 % of shares; packet capture shows the content sent only to the paired device. | Task trial, inspection and packet capture | yes | D07.L1, D07.L2 | proposed |
| AC-2.7 | L2 Everyday | A plugin crashes, hangs and exceeds its memory limit during use. | Draft kept; application responsive within the AC-2.5 budgets; the plugin is stopped and reported; it never draws over a trust surface. | Automated fault injection | no | D14.L2 (part) | proposed |
| AC-2.8 | L2 Everyday | Allow an application a remembered access, repeat the identical request and a request that differs in one key field, find the grant in Appmachtigingen, revoke it, then repeat the identical request. | Before revocation the identical request does not prompt and the differing request prompts; the grant is listed with its five key fields; after revocation the identical request prompts again in 100 % of runs; the privacy indicator names the application while access is in use. | Automated | no | D13.L1 (part), D13.L2, D13.L3 | proposed |
| AC-2.9 | L2 Everyday | Inventory every number field and operate each by keyboard. | 100 % show their unit and step with Up/Down and Page Up/Page Down, and with Home/End where a limit exists; each exposes its name, value and unit in the accessibility tree. | Automated | no | D04.L2, D12.L1 | proposed |
| AC-3.1 | L3 Proficient | Participants who use the discipline weekly convert a directly formatted document, image or timeline into a structured one (styles, layers, tracks) and change it globally. | Optimal completion (T1) ≥ 70 % without help. | Task trial | yes | L3 signal, D01.L3 (part) | proposed |
| AC-3.2 | L3 Proficient | Switch `essential` → `extended` → `complete` → `essential`, and `standard` ↔ `compact` on the whole workspace and on one panel, on a fixture that uses every feature kind of the discipline. | Revision and content hash unchanged; no new revision; every feature shown or signalled at `essential` (RULE-3, RULE-5); the compact panel alone changes density. | Automated | no | D03.L1, D03.L3, RULE-3, RULE-5 | proposed |
| AC-3.3 | L3 Proficient | For each feature named in the S1 Professional depth of PR05, GR02, GR21, VI04 and TD02 that the discipline has, select an object and set the value numerically in the inspector, at each preset. | 100 % of those features reachable from the inspector at every preset; every value read back equals the value entered; aligned and distributed geometry matches the expected result. | Automated | no | D04.L3 | proposed |
| AC-3.4 | L3 Proficient | Participants reply to and resolve a comment thread, filter comments by author, turn tracking on and accept all changes, then compare two versions and restore one part (paragraph, layer, clip) from the earlier version. | ≥ 70 % complete each step without help; the current version is kept. | Task trial | yes | D07.L3, D10.L3 (part) | proposed |
| AC-3.5 | L3 Proficient | Change preset, density and layout; reset one item, reset all, undo; repeat in a guest session and sign out. | Each reset and undo restores the expected state; settings kept per user and application; nothing remains after guest sign-out. | Automated | no | D01.L3 (part), D03.L2, RULE-6 | proposed |
| AC-3.6 | L3 Proficient | Over two weeks of normal work with shortcuts shown in menus and palette (no usage-derived aids), record how the five most frequent commands are invoked. | The keyboard share of those commands rises from week 1 to week 2 for ≥ 60 % of participants (the author's measure, related to the command metrics of [E2]). | Task trial, longitudinal, with locally stored logs the participant reviews | yes | D02.L2, D11.L2, RULE-12 | proposed |
| AC-3.7 | L3 Proficient | With focus in a timeline, use each transport key (Space, J/K/L, arrows) and each app-modified chord and compare it with its menu, palette and keymap entry; then repeat the AC-1.9 single-key test. | Every key acts as listed; 100 % are listed with their context; the AC-1.9 result is unchanged. | Automated | no | D02.L3 | proposed |
| AC-3.8 | L3 Proficient | Measure responsiveness on the discipline's proficient-scale reference corpus (hundreds of pages, thousands of rows or dozens of layers), including during saves. | The AC-2.5 budgets hold at the 95th percentile. | Native measurement (U02) | no | D06.L3 | proposed |
| AC-4.1 | L4 Expert | Search the keymap by command name and by key; rebind ten commands including one conflict; turn context single keys off; import a familiar preset in which two commands have no equivalent; export the keymap delta and import it under another account; reset. | Both searches find the command; conflict detected 100 %; both unmatched commands reported; no preset named after a vendor; imported bindings identical; reset restores defaults. | Automated | no | D01.L4, D02.L4 | proposed |
| AC-4.2 | L4 Expert | Run the expert reference corpus of each discipline (sizes proposed for S10, for example a 500-page illustrated document, a workbook of one million filled cells, a 100-megapixel 16-bit image with 50 layers, a 60-minute four-track timeline; not derived from another product's limits). | The AC-2.5 budgets hold at the 95th percentile; no data loss under memory pressure within K07 limits. | Native measurement | no | D06.L4 (part) | proposed |
| AC-4.3 | L4 Expert | Run a batch over 200 files including one corrupt file, built from a recorded action where the discipline's L4 minimum has recorded actions (WP26, SS14) and otherwise from the discipline's own batch capability at or below L4 (for example PD04, GR17, GR38 or VI19); keep editing while it runs; cancel once and resume. A discipline with neither omits this criterion. | Per-item report with the corrupt file named; all others complete; no unreported partial output; a batch that changes documents is one revertible history entry; editing stays within the AC-2.5 budgets while the batch runs. | Automated | no | D05.L4, D06.L4 (part) | proposed |
| AC-4.4 | L4 Expert | Round-trip the S5 corpus as one batch through a named third-party suite: open, edit, save, reopen there. | 0 differences missing from the per-document and per-batch fidelity reports; S5's feature rows for the discipline pass. | Automated and inspection in the third-party suite | no | D08.L4 | proposed |
| AC-4.5 | L4 Expert | Professionals migrating from another tool, using a familiar keymap preset, perform a professional benchmark workflow (addendum §7) in the BaryonOS application and in their usual tool. | Completion ≥ 90 %; median time ≤ 1.25 × their usual tool after 30 minutes of familiarisation. | Task trial, within-subject | yes | D02.L4 | proposed |
| AC-4.6 | L4 Expert | Screen-reader and keyboard-only professionals perform an expert task in a grid, a timeline and a canvas, then rebind one command in the keymap editor and read a batch report. | Completion ≥ 80 %; 0 elements in the task without accessible name, role and value. | Automated tree check and task trial with assistive-technology users | yes | D12.L1, D12.L4 | proposed |
| AC-4.7 | L4 Expert | Protect a range and try to edit it; preflight seeded fixtures against a print and an archival profile; preview a batch of 200 files before it runs, then run it. | 100 % of edits to the protected range refused with a message; 100 % of seeded preflight violations reported, with a fix where the profile defines one; the preview lists every affected item and the run changes no item outside it. | Automated with seeded fixtures | no | D09.L4 | proposed |
| AC-5.1 | L5 Specialist | A script bound to a palette entry tries to read a file outside its grant, open a network connection without a grant and change the open document; its grant is then revoked and it runs again. | The first two are denied in 100 % of runs; the change is one undoable history entry; the script runs in a process separate from the application and is listed with its grants; after revocation each access is denied or prompts again. | Automated negative tests | no | D02.L5, D05.L5, D13.L5 | proposed |
| AC-5.2 | L5 Specialist | A specialist records an automation for a repeated task and binds it to a palette entry and a key; a colleague acting at L2 runs it from the palette for a dependent task. | Specialist completes without help ≥ 70 %; colleague completes without help ≥ 80 %. | Task trial in pairs | yes | L5 signal, D02.L5, D05.L4 | proposed |
| AC-5.3 | L5 Specialist | Export the discipline's interchange formats chosen in S5 and validate them. | 100 % validate with the reference validators; the fidelity report is machine-readable. | Automated | no | D08.L5 | proposed |
| AC-5.4 | L5 Specialist | Two participants co-edit one document in real time over paired storage while one goes offline and both edit; a library maintainer publishes a library update that a subscriber reviews; a teacher distributes an assignment to three student accounts, collects it and returns it with feedback. | 0 edits lost and every conflicting edit shown in conflict review in 100 % of automated runs; 0 bytes of content sent to hosts outside the paired or end-to-end encrypted storage (packet capture); the update is applied only after review; ≥ 80 % of teacher participants complete the classroom cycle without help. | Automated sync and packet-capture tests; task trial with teachers | yes | D07.L5 | proposed |
| AC-X.1 | All levels | Inspect the settings schema and capture network traffic over a full multi-application session. | No maturity-level field and no usage profile stored; 0 bytes of usage data leave the device (RULE-1, LP-13). | Automated and inspection | no | D13.L1 (part), RULE-1 | proposed |
| AC-X.2 | All levels | Turn usage-derived learning aids on, use them, then clear them and turn them off. | Clearing deletes all aid data; with aids off no usage is recorded. | Automated storage inspection | no | RULE-8 | proposed |
| AC-X.3 | All levels | Use commands at very different frequencies over five sessions. | Menu contents and order identical across sessions apart from the user's own changes (RULE-7). | Automated | no | RULE-7 | proposed |
| AC-X.4 | All levels | Compare permission prompts, Tier 1 checks, flash checks, save states, autosave and isolation across all presets and densities, and with privacy explanations on and off; then set each tier-2 axis of Instellingen › Toegankelijkheid in turn and inspect every first-party application at each preset and density. | 0 differences apart from the explanation text (RULE-4); with explanations off, existing grants are unchanged and every new request still prompts; each tier-2 axis takes effect in 100 % of applications at every preset and density, except in the inert cases that DESIGN-SYSTEM documents: Transparantie and Prikkels in the contrast theme, where Contrast only thickens edges (§21.3, O-A7), and Regellengte where the surface is narrower than the measure (§21.4). | Automated | no | D03.L3, D12.L1, D13.L1 (part), D13.L3, RULE-4, RULE-11 | proposed |
| AC-X.5 | All levels | Participants switch from Essential to Complete and back, then say whether any content was removed. | ≥ 80 % switch within 60 s; ≥ 80 % correctly say that nothing was removed. | Task trial | yes | D01.L3 (part), RULE-2 | proposed |
| AC-X.6 | All levels | Open a received document that contains embedded automation, at every preset and in an application that has a scripting interface. | The automation does not run before an explicit decision; it is signalled at every preset (RULE-3). | Automated and inspection | no | D05.L1 (part), D05.L5, RULE-3 | proposed |
| AC-X.7 | All levels | Run the automated accessibility-tree and Tier 1 checks over every view named in D12.L2 to D12.L5 that the application provides, then a keyboard-only run of each view's actions: palette, find and run a command; comments and threads, add, reply, resolve and filter; version list and comparison, open, compare and restore; grant list, revoke; tracked changes, accept and reject; style, layer and track panels, select, reorder and hide; a fixture plugin's interface, operate every control; keymap editor, rebind; batch, fidelity and preflight reports, move to each item; merge-conflict review, choose a side. | 0 elements without accessible name, role and value; 0 Tier 1 (B1–B12) failures; 100 % of the listed actions complete keyboard-only. | Automated | no | D12.L2, D12.L3, D12.L4, D12.L5 (part) | proposed |

## 8. Risks and open items

### 8.1 Decisions that need the project owner (not decided here)

- **O1 Product budgets.** The thresholds of section 7 are proposals. U06
  requires agreed core tasks and product budgets; the owner agrees them.
- **O2 Grants for scripts, batch jobs, plugins and embedded automation.** D02.L5,
  D05.L1, D05.L5, D13.L4, D13.L5 and D14.L2 describe bounded grants, separate
  principals and a decision before embedded automation runs as [JUDGEMENT]. ADR-DESIGN-OWNER-20261115 D2 already requires an ADR for
  persistent grants; this scope belongs in it or in A03 and UX-C.
- Already pending and unchanged here: P-NAMES-1 (names), the Apple-keyboard
  chords (S0 open item 5) and any wish to tie features, editions or prices to
  levels, which section 3 rules out as a proposal.

### 8.2 Open items

1. No user test and no ADR. Every level assignment, cell and threshold is
   judgement until task trials (U06) and research with professionals exist.
2. Permission wording mismatch: S1 XC09 (*proposed*, unchanged) and
   NAMING-AND-IA §6 still describe the superseded three-choice prompt of
   14 November 2026. This document follows ADR-DESIGN-OWNER-20261115 D2 and
   reads XC09 with it (section 6.1, rule 5). The mismatch is not resolved in
   S1 or NAMING-AND-IA, which are not edited.
3. Section 6 and the dimension and criteria tables were validated by
   `python3 -I build/design-evidence/suite-s2/s2_tool.py check .`, an
   unversioned scratch script under the ignored `build/` directory (not a
   gate; output and mutation cases beside it). Storing it under a new
   `docs/verification/` child is left to the lead. It checks names and
   evidence levels against S1, exactly-once coverage of the 216 rows, the
   14/202 split, all 75 dimension cells, the column tag and level of every
   S1 citation in a section 4 Basis against section 6 (including the
   "from Lk" marks), the lists of items 14, 15 and 17, references to
   criteria, cells, owner decisions and items of this section, that
   progression rules are named RULE-1 to RULE-12 and never by the S1-style
   prefix PR, and the section 9 URLs. It cannot judge whether a criterion
   exercises a cell, nor apply the applicability rule of section 4; the
   "(part)" marks and item 16 are [JUDGEMENT].
4. ds-1.1 (`DESIGN-SYSTEM.md`) is unaccepted; UX-A11Y-DS run 3 addresses B8
   keyboard-repeat flashing, `.gal-note` contrast and modal backdrop dimming.
   D12 states Tier 1 as a requirement only and reads B8 with
   ADR-DESIGN-OWNER-20261115 D3 (D12 introduction); which parts of B8 change
   is left to UX-A11Y-A1.
5. Sources not opened and therefore not cited: the survey by Cockburn,
   Gutwin, Scarr and Malacria on novice-to-expert transitions (ACM Computing
   Surveys, 2014; the copy found returned HTTP 403), and the ISO pages for
   ISO 9241-110:2020 and ISO 9241-11:2018 (HTTP 403). No claim relies on them.
6. Reference corpora and the reference system for AC-2.5, AC-3.8 and AC-4.2
   are not defined; S10 and U02 own them. Native measurement depends on H03 (GPU,
   audio, open) for every real-time discipline, and audio latency budgets
   need their own research (S1 AU16).
7. Fidelity criteria (D08, AC-2.4, AC-4.4, AC-5.3) depend on S5.
8. Collaboration cells D07.L2–D07.L5 depend on P01–P04 and N02–N04 (open)
   and on the privacy contract's paired-storage model.
9. Needs of specific accessibility groups (switch access, braille output,
   cognitive accessibility) are not broken down further than S1 open item 9
   and the accessibility matrix; the systematic accessibility review required
   by ADR-DESIGN-OWNER-20261115 D1 remains a separate open task.
10. Creator-facing checks for flashes in deliberately produced video (ADR
    D3 concerns playback and the system interface) are not an S1 row. A row
    could be appended to S1 later; S1 is not changed here.
11. Terminology: S0 D6 uses "level" for disclosure; S2 uses "disclosure
    preset" for that concept and "maturity level" for this model. S0 is not
    edited.
12. Shared and classroom devices: per-user preferences (RULE-6) versus teacher-
    or administrator-provided layouts and keymaps (S1 XC08) need research
    within PROFILES-AND-USERS's role model.
13. Usage-derived learning aids (RULE-8) are proposed only in the form the
    owner's framing allows: local, user-controlled, off by default and
    clearable. Whether they help enough to offer at all is a question for
    task trials (AC-3.6 tests the static aids only).
14. Cells without a criterion: D01.L5, D03.L4, D03.L5, D04.L4, D04.L5, D05.L2,
    D05.L3, D06.L5, D07.L4, D08.L3, D09.L3, D09.L5, D10.L4, D10.L5, D11.L3,
    D11.L4, D11.L5, D13.L4, D14.L1, D14.L3, D14.L4, D14.L5, D15.L3, D15.L4,
    D15.L5. Cells tested only in part: D01.L1, D01.L3, D04.L1, D05.L1, D06.L4,
    D08.L1, D09.L1, D09.L2, D10.L1, D10.L3, D11.L1, D12.L5, D13.L1, D14.L2,
    D15.L1, D15.L2. Clause (b) of "serves level N" (section 2.1) cannot be
    verified for these cells until criteria are added or extended.
15. Cells whose Basis marks an earlier requirement: D03.L5, D05.L3, D05.L4,
    D06.L4, D07.L4, D07.L5, D08.L5, D14.L4, D15.L3. Each builds on an S1 column
    that section 6 requires from a lower level ("from Lk"); that level governs,
    and the cell adds only what it states beyond it.
16. Discipline scope of section 4 clauses (decision of repair round 2,
    [JUDGEMENT]). GR18's Professional depth ("Recorded actions, scripted
    batches") stays at L5 and is cited in D05.L5; D05.L4 and AC-4.3 require
    recorded actions only where section 6 places them at L4 (WP26, SS14
    Beginner path). Moving GR18's Professional depth to L4 was rejected
    because it would bring scripted batches to L4, one level before scripting
    (WP26, SS14 Professional depth, L5). Presentations, notebooks, mail,
    databases, PDF, video, audio, interface design, 3D and the cross-cutting
    rows have no recorded-action row in S1 (PR08 recorded slide timings, NK02
    recordings and AU06 mix automation are other capabilities). The
    applicability rule of the section 4 introduction generalises the
    decision; following S1 §5, rows housed in graphics or interface design
    but shared by every surface (GR34–GR37, UX06, UX07) bind every
    application, so the shared libraries of D07.L4 (GR36) do. A keyword sweep
    of the Beginner path and Professional depth columns of S1 against the
    capabilities that section 4 names (script and output in
    `build/design-evidence/suite-s2/repair2/`; per-feature presets and
    templates were not treated as one capability) found these rows, not
    cited by the cell concerned, that place its capability at another level;
    each discipline follows its own section 6 level. Later than the cell:
    GR18 recorded actions (L5; D05.L4), UX03 constraint layouts (L4;
    D04.L3), DB02 and PD06 validation (L4; D09.L3), GR37 locked brand items
    (L5, shared; D09.L4). Earlier than the cell: WP17 history list (L1;
    D10.L2) and branches (L3; D10.L4), WP22 fidelity report (L3; D08.L4),
    EM10 audit history (L4; D10.L5, D15.L5). A consequence of the rule is
    that a discipline without a row for a capability is not bound by the
    clause that names it, for example comments (D07.L2) in audio, or
    real-time co-editing (D07.L5) in word processing: S1 places co-editing
    in EM11, which S1 §5 assigns to the communication hub, not to every
    surface. Open: where only a row's Professional expectation names a
    capability (comments and concurrent editing in SS17, comments and
    multi-user editing in UX10), S1 assigns it to no column, so the rule
    gives it no level; S6 or a revision of S1 resolves this. Whether a
    discipline without a row needs the capability is likewise a question for
    S1 and S6, not decided here.
17. Criteria that name a cell above their own level (cross-level criteria
    count as L1), as criterion and cell: AC-1.2 D10.L2, AC-1.4 D01.L2,
    AC-1.7 D06.L2, AC-1.8 D11.L2, AC-1.9 D03.L2, AC-2.8 D13.L3, AC-X.4 D03.L3,
    AC-X.4 D13.L3, AC-X.5 D01.L3, AC-X.6 D05.L5, AC-X.7 D12.L2, AC-X.7 D12.L3,
    AC-X.7 D12.L4, AC-X.7 D12.L5. Under clause (c) of section 2.1, the steps
    that exercise only these cells bind from the cell's level. At the
    criterion's own level the following remain. AC-1.2: undoing the deletion
    of a part, restoring an earlier version as a new revision (D10.L1) and
    undoing the deletion of the document (D09.L1), with "recover all three"
    read over those three; browsing the version list and restoring from the
    Prullenbak bind from L2. AC-1.4: the menu-bar inventory and panel depth;
    the palette listing and the shortcut beside each menu item and palette
    entry bind from L2. AC-1.7: every step, because its truthful save states
    are also required by D10.L1 and D15.L1. AC-1.8: reaching
    help offline; help search binds from L2. AC-1.9: every step, because view
    zoom is also an app-standard chord of D02.L1. AC-2.8: every step, because
    each also exercises the five-field key of D13.L1. AC-X.5: no step before
    L3, because switching the preset is D01.L3. AC-X.4, AC-X.6 and AC-X.7:
    their density, privacy-explanation, embedded-automation and view checks
    test protections and apply wherever the view or setting exists.
    Relabelling or splitting these criteria is left to the next revision of
    S2.

### 8.3 Risks

| Risk | Consequence | Mitigation in this document |
|---|---|---|
| Levels are reused as personas, editions or visibility rules | Features hidden from users who need them | RULE-1, RULE-2, RULE-3; AC-1.4, AC-X.1 |
| The `essential` first-launch preset slows migrating experts | The performance dip deters switching [E1] | RULE-9, RULE-12; S1 XC05; AC-4.5 |
| A linear ladder misfits experts who use different subsets [E5] | Wrong minimums per level | Levels set minimums, never visibility; section 6 assignments open to S6 task analysis |
| Learning aids drift into profiling | Breach of the no-telemetry principle | RULE-1, RULE-8; AC-X.1, AC-X.2; section 8.2 item 13 |
| 102 S1 rows enter at L1, 89 of them *proposed*; every discipline's L1 minimum contains at least one *proposed* row | No discipline can serve L1 soon | Exposed for S10 sequencing; shared engines first (S1 §4) |
| Lossy import at first use | Early loss of trust | D08.L1 stated loss; AC-2.4 |

## 9. Sources

Project sources (repository, read on 18 November 2026): S0
`docs/design/suite/WORKSPACE-REQUIREMENTS.md` (D1–D9, open items);
S1 `docs/design/suite/CAPABILITY-MATRIX.md` (§1, §2, §4–§6 in full; full
rows for WP, SS01–SS03 and XC; for every other row the identifier, name,
Beginner path, Professional depth, support, evidence level and priority);
`docs/design/DESIGN-SYSTEM.md` (ds-1.1 preamble, §11, §11.1, §21);
`docs/design/NAMING-AND-IA.md` (§1, §4–§6); `docs/DESIGN-AMBITION.md` (additions
of 14 and 15 November 2026); `docs/implementation/DESKTOP-SHELL.md` (§2–§4);
`docs/implementation/PROFILES-AND-USERS.md` (§3); `docs/DECISIONS.md`
(ADR-058, ADR-DESIGN-OWNER-20261115 D1–D3); `docs/LOCAL-PRIVACY-CONTRACT.md`
(§4, §4.1, LP-13, LP-30); `docs/progress.json` (U02–U06, A03); the project
owner's professional-ecosystem addendum (local, not versioned; §1, §2, §4, §5,
§6, §8).

External sources, all retrieved 18 November 2026. PDF text was extracted
locally to `build/design-evidence/suite-s2/sources/` (scratch, not
versioned).

| ID | URL | Supports; limitation |
|---|---|---|
| E1 | https://csse.canterbury.ac.nz/andrew.cockburn/papers/blur.pdf | Scarr, Cockburn, Gutwin, Quinn, "Dips and Ceilings", CHI 2011: few users switch to expert methods; beginner mode and low ceiling; satisficing; performance dip; three goals for supporting the switch; Blur's calm notification shows the command equivalent of each pointer action as transient feedback, and its command recommendations are ordered by frequency of use (RULE-8). Limitation: Blur observes every pointer action; S2 allows usage-derived aids only locally and under user control (RULE-8). |
| E2 | https://research.autodesk.com/app/uploads/2023/03/a-survey-of-software.pdf_recqWPTEYd6nsP0r8.pdf | Grossman, Fitzmaurice, Attar, "A Survey of Software Learnability", CHI 2009: initial and extended learnability; task, command and documentation metrics (T1, T2, T4, T5, C2); five categories of learnability issues; guideline on demonstration in the user's own interface. Limitation: the issue categories come from one study with ten participants using one CAD application. |
| E3 | https://www.cs.ubc.ca/labs/edapt/papers/findlater2004.pdf | Findlater, McGrenere, "A Comparison of Static, Adaptive, and Adaptable Menus", CHI 2004: 27 participants; static faster than adaptive; adaptable preferred (15 of 27 first overall). Limitation: lab study of split menus. |
| E4 | https://www.dgp.toronto.edu/public_user/RMB/papers/p28.pdf | McGrenere, Baecker, Booth, "An Evaluation of a Multiple Interface Design Solution for Bloated Software", CHI 2002: personal interface toggled with the full interface; 20 participants for about six weeks; better navigation, learning, satisfaction and control than adaptive menus. Limitation: one word processor, self-reported measures. |
| E5 | https://www.cs.umd.edu/~ben/ACM-CUU2003.pdf | Shneiderman, "Promoting Universal Usability with Multi-Layer Interface Design", ACM Conference on Universal Usability 2003: novices start at layer 1 and move up when needed; critics' concerns on feature subsets and output compatibility across layers; user control favoured over adaptive menus. Limitation: position paper. |
| E6 | https://www.nngroup.com/articles/progressive-disclosure/ | Nielsen, "Progressive Disclosure" (2006): defer advanced features; more than two levels typically have low usability; the route to advanced features must be visible. Limitation: practitioner guidance; read through a summarising fetch. |
| E7 | https://www.nngroup.com/articles/response-times-3-important-limits/ | Nielsen, "Response Times: The 3 Important Limits" (1993, update 2014): 0.1 s, 1 s, 10 s; percent-done indicator and interruption for long operations. Limitation: not an audio or video real-time budget; read through a summarising fetch. |
| E8 | https://www.w3.org/TR/WCAG22/ | WCAG 2.2, W3C Recommendation of 12 December 2024: SC 2.1.1, 2.1.4, 2.5.7, 2.5.8, 3.2.6, 3.3.4; a web standard used by analogy for native applications. |
| E9 | https://www.w3.org/WAI/WCAG22/Understanding/error-prevention-legal-financial-data.html | Understanding SC 3.3.4: reversible, checked or confirmed submissions for modifying or deleting user-controllable data; does not require confirmation of every save. Informative. |
