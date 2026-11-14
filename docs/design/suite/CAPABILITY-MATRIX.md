# Professional application capability matrix (S1)

## 1. Status, scope and definitions

Status: design research and proposal, 14 November 2026 (Europe/Amsterdam).
Not an ADR and not a normative decision; no user test was performed. This is
deliverable S1 of the project owner's professional-ecosystem addendum, written on branch
`claude/baryon-os-ui-ux-design-152adb` at source `ec8b1d08`. Microsoft 365, Adobe Creative Cloud and the other tools
named below serve as **functional and workflow benchmarks only**: no trade
dress, layout, iconography or documentation text is reproduced. Names and
extensions for a BaryonOS-native file-format family, and the native object
architecture, are out of scope here (deliverables S3 and S4); a "native project
container" is referred to only generically. External formats and open standards
are named only to state what a stored prototype does or which standard applies.
The feature-level interoperability matrix is deliverable S5.

**Columns.** *Capability*: row identifier and capability; for a row above
*proposed* the name is the exact tested feature, the expectation and depth
columns state the full professional capability, and where the gap is large a separate
proposed row carries it (for example WP24, PR13, NK11, GR17 and GR38).
*Professional expectation*: what a professional must
be able to do (judgement, see section 3). *Beginner path*: the simplest route
for a first-time user. *Professional depth*: the expert control the capability
needs; it never lists prototype limits. *Shared dependency*: engines from the
controlled vocabulary in section 4.1. *BaryonOS support today*, *Evidence
level* and *Priority*: defined below. *Evidence*: repository paths for every
row above *proposed*, with the tested scope and limits in brackets; "—" when no
repository evidence exists, with a bracketed note when related code or an open
criterion is relevant.

**Evidence levels** (column "Evidence level"):

- **passed (native)**: the row's own capability matches the acceptance sentence
  of a criterion recorded as `accepted` in `docs/progress.json`, with stored
  native QEMU evidence. A passed dependency does not lift an application row.
- **prototyped (host)**: a host-process prototype or library test exercises
  exactly the feature named in the row. It proves that feature only, never the
  discipline.
- **proposed**: design or research only; no repository code exercises the
  row's capability as named. Related partial code is noted in brackets in the
  Evidence cell.

A native QEMU run whose criterion is still open never raises a row: the row
keeps the level its host evidence supports, and the Evidence cell adds
"native QEMU run, criterion open".

**BaryonOS support today** describes what repository code demonstrably does
for the row's capability at the stated evidence level; for host rows it states
the prototype's fidelity, not availability on the native system. Values:
*full*; *with limitations* (the feature works within stated bounds); *lossy*
(the operation succeeds only by discarding stated content of its input, such
as formatting, metadata, colour profiles, bit depth or alpha, even when the
user must confirm the loss); *unsupported*; *proprietary research* (meaningful
support depends on undocumented or proprietary behaviour and needs research
first). Tie-break: an operation that preserves everything it accepts and
rejects input it cannot preserve is *with limitations*, not *lossy*. Every
proposed row is *unsupported* or *proprietary research*.

**Priority**: P0 shared infrastructure that many rows need; P1 core
application capability; P2 professional depth; P3 later. Priorities are
proposals for the phased plan (deliverable S10), not commitments.

**Identifiers.** Matrix rows carry two-letter discipline prefixes (WP word
processing, SS spreadsheets, PR presentations, NK notebooks and knowledge, EM
email and calendar, DB databases and workflows, PD PDF, GR graphics, VI video,
AU audio, UX interface design, TD 3D, XC cross-cutting suite capabilities).
Identifiers are stable: rows added after the first version are appended to
their table, never renumbered. A single letter followed by two digits,
such as K03 or U03, denotes a criterion in `docs/progress.json`; the one
exception, deliverable S10, is always written with the word "deliverable". S0
to S9 denote addendum deliverables. Numbers in square brackets denote the sources
in section 7. Statements marked "judgement" are the author's assessment, not a
sourced fact.

**Result in one line.** 215 capability rows; 0 *passed (native)*; 14
*prototyped (host)*; 201 *proposed*; no row has *full* support. The repository
proves native OS services that the suite will depend on (section 4), but no
professional application capability has native acceptance.

The host evidence for every row above *proposed* is cited in that row's
Evidence cell: nine platform receipts and the Materiaal library record
(README and host test logs). An evidence inventory and a script that validates
this document's tables and counts its dependencies are stored in
`docs/verification/suite-s1-capability-matrix-20261114/`: `evidence-inventory.md` and
`python3 -I docs/verification/suite-s1-capability-matrix-20261114/check.py . docs/design/suite/CAPABILITY-MATRIX.md`
(see open item 13).

## 2. Why users stay with mature suites

Each driver below is a judgement about switching cost, supported by an opened
primary source that shows the mechanism. "Sequencing (judgement)" states whether the driver
changes what BaryonOS builds first.

**File fidelity and round trips.** Users exchange files with people who do not
switch. Microsoft's own comparison of its word-processing format with
OpenDocument Text lists track changes as not supported (changes are accepted
on save) and tables as only partially supported [1]. The UK government's ODF
guidance warns that some functions may be lost when converting spreadsheets to
ODF [2]. Office Open XML itself is a large standard (ECMA-376, ISO/IEC 29500)
[3]. Sequencing (judgement): decisive. Isolated converters with an explicit fidelity
report are P0 infrastructure; a converter that opens a file is not a round trip
(see WP22–WP24).

**Calculation semantics.** Spreadsheet results are trusted because colleagues
get the same numbers. Microsoft documents that its spreadsheet stores numbers
as IEEE 754 double precision with 15 significant digits, yet deviates from
IEEE 754 in places: no denormalised numbers, no infinities or NaN (errors
instead), and a near-zero compensation for addition and subtraction since Excel 97
[4]. OpenFormula defines formula syntax, semantics and functions as an open
standard [5]. Sequencing (judgement): decisive. The existing decimal prototype (SS02) uses
nine fractional digits and must not be presented as compatible; matching
semantics is a research item (SS04).

**Muscle memory and keyboard shortcuts.** Experts work through shortcuts and
customised commands. Word lets users bind shortcuts to commands, macros, fonts,
styles and symbols and store them in a document or template [6]. Sequencing (judgement):
early and cheap. A shared command and shortcut system with familiar defaults
follows ADR-058 and belongs in P0 (Input & command system); customisation,
keymap presets and configurable workspaces are rows XC03–XC05.

**Reliability and recovery.** Users trust a suite that does not lose work.
LibreOffice's general load and save options offer periodic recovery data for
all modified documents in case of a crash, an option to save the document
itself at that interval instead, and a backup copy of the previous version on
every save [36]. Sequencing (judgement): decisive for trust and cheap to design
in. The storage journal and cold recovery are accepted (S02), and host tests
keep in-session drafts on conflict and I/O failure (WP01), but no application
autosaves or restores unsaved work after a crash (XC01, XC02).

**Precision tools.** Professionals depend on exact placement. LibreOffice's
grid options define snap-to-grid with resolution, subdivision and a modifier
to override snapping for a single action [7]. Sequencing (judgement): shared 2D vector
engine capability; needed before any layout, illustration or slide editor is
credible (PR05, GR21).

**Performance on large documents.** Mature tools set high, documented limits.
The current spreadsheet limit is a 1,048,576-row by 16,384-column worksheet,
32,767 characters per cell and 64 nested function levels, with documented pivot
table limits [8]. Sequencing (judgement): affects data structures early (columnar,
incremental), but measured performance is P2 depth; current prototypes are
deliberately small (64 formula cells, 4096 CSV rows).

**Plugins and extensions.** Organisations rely on add-ins. Microsoft's add-in
platform uses web technologies that run sandboxed in a webview, are declared
through a manifest with permission levels, deploy centrally and are distributed
through a marketplace; the older COM and VSTO add-ins run only on Windows [9].
Audio work depends on plugin standards such as CLAP and LV2 [10, 11].
Sequencing (judgement): decisive for the security model. Plugins must run in isolated
processes under capabilities; the kernel side (K03, K04, K07) is accepted, the
application permission layer (A03) is open.

**Collaboration.** Real-time co-authoring shows each author's presence and
edits; users on older versions or without the required subscription see
others' changes only after saving [12]. Sequencing (judgement): depends on the open NAS,
sync and network criteria (P01–P04, N02–N04) and on the privacy contract (no
vendor cloud); comments and history come first, real-time editing later (P2).

**Colour management.** Print and photography depend on ICC profiles; the
current ICC specification is ICC.1:2022 (profile version 4.4), and the v4
specification was also published as ISO 15076-1 in 2005 [13]. Motion-picture work uses colour-management frameworks such
as OpenColorIO [14] and HDR parameters in ITU-R BT.2100 [15]. Sequencing (judgement):
decisive for every graphics and video row; colour must be designed into the
pixel pipeline from the start (GR09, GR10, VI09). All current image prototypes
explicitly run unmanaged.

**Training and certification.** Skills are credentialed. Microsoft's Excel
Expert exam expects about 150 hours of instruction and hands-on experience [16]; Adobe's
certification recommends about 150 hours of application experience and tests
hands-on tasks inside the application [17]. Sequencing (judgement): not a build item. It
argues for familiar concepts and terminology so existing training transfers
(judgement), and for later learning material (P3).

**Ecosystem integration.** Suites are tied together by shared identity and data
services: Microsoft Graph exposes mail, calendar, contacts, files, notes,
planning, tasks and Teams data through one endpoint [18]. Sequencing (judgement): BaryonOS
should reach comparable integration through Materiaal objects and open
protocols such as IMAP, iCalendar, CalDAV and vCard [19–22], not through a
mandatory vendor cloud (privacy contract). This is a P1 requirement for the
communication rows.

## 3. Capability matrix by discipline

Column meanings are fixed in section 1. Professional expectations are functional
judgements informed by the sources in section 7 and by the reference tools named
in the addendum; they describe what a professional must be able to do, never how
a vendor's interface looks. Shared-dependency names come from the controlled
vocabulary in section 4.

### 3.1 Office and productivity

#### 3.1.1 Word processing, typography, references, revision tracking, publishing and complex layout

| Capability | Professional expectation | Beginner path | Professional depth | Shared dependency | BaryonOS support today | Evidence level | Evidence | Priority |
|---|---|---|---|---|---|---|---|---|
| WP01 Plain-text document edit, save and reopen | Reliable editing with explicit save; no silent loss on conflict or I/O failure | Type, save, reopen from Materiaal | Read-only, conflict and uncertain-commit states preserved as drafts | Materiaal history; Text & font engine | with limitations | prototyped (host) | `docs/verification/materiaal-library-20261102/checks/app-host.stdout.txt`; `docs/implementation/NATIVE-MATERIAAL.md` (native QEMU run, criterion U03 open) | P1 |
| WP02 Rich-text formatting | Character and paragraph formatting, lists, alignment, spacing | Formatting bar with presets | Exact numeric values, format inspection, clear formatting | Text & font engine | unsupported | proposed | — | P1 |
| WP03 Style system | Paragraph, character, list and table styles with inheritance | Built-in style sets | Inheritance, update from selection, style transfer between documents | Text & font engine; Asset library & linking | unsupported | proposed | — | P1 |
| WP04 Advanced typography | OpenType layout features, kerning, ligatures, variable fonts, hyphenation, justification | Language-aware defaults | Per-feature control, hyphenation zones, widow/orphan rules, baseline grid | Text & font engine | unsupported | proposed | — | P0 |
| WP05 Complex scripts and bidirectional text | Correct shaping and bidi ordering for RTL, Indic and CJK; vertical text | Automatic by language | Per-run language tags, vertical layout, ruby annotation | Text & font engine | unsupported | proposed | — (current renderers use bitmap atlas fallback without shaping) | P0 |
| WP06 Pagination and page setup | Sections, columns, headers/footers, page numbering, mixed orientation | Page templates | Section breaks, column balancing, keep-with-next rules | Text & font engine; PDF & print engine | unsupported | proposed | — | P1 |
| WP07 Document tables | Merge/split cells, repeating headers, tables across pages, cell styles | Insert grid | Break rules, decimal alignment, nested tables | Text & font engine | unsupported | proposed | — | P1 |
| WP08 Images and floating objects | Anchoring, text wrap modes, captions, linked or embedded images | Drag and drop | Exact anchor offsets, linked-image status | Text & font engine; Asset library & linking | unsupported | proposed | — | P1 |
| WP09 Fields and cross-references | Live page, figure and heading references; document properties | Insert page number | Cross-references, field formatting, update control | Text & font engine | unsupported | proposed | — (DOCX import rejects fields) | P2 |
| WP10 Multilevel numbering and outline navigation | Numbering linked to heading styles; navigation by outline | Click a heading in an outline panel | Legal numbering, restart rules, outline reordering | Text & font engine | unsupported | proposed | — | P1 |
| WP11 Tables of contents, indexes and lists of figures | Generated from styles and updatable | Generate contents in one step | Multiple indexes, concordance files | Text & font engine | unsupported | proposed | — | P2 |
| WP12 Footnotes and endnotes | Auto-numbered notes with section restart | Insert footnote | Numbering formats, continuation notices | Text & font engine | unsupported | proposed | — | P1 |
| WP13 Citations and bibliography | Reference library, citation styles, generated bibliography | Insert citation from library | Style switching, shared libraries, identifier lookup | Text & font engine; Search & index; Asset library & linking | unsupported | proposed | — | P2 |
| WP14 Equations | Structured mathematical editing with standard interchange | Equation templates | Linear input, numbering, interchange | Text & font engine | unsupported | proposed | — | P2 |
| WP15 Comments and threaded review | Anchored threads with resolve states and author filters | Add comment | Threads, resolve, filter, export of review | Collaboration & sync; Materiaal history | unsupported | proposed | — | P1 |
| WP16 Revision tracking and document comparison | Attributed insertions, deletions and format changes; accept/reject; compare two versions | Turn tracking on, accept all | Per-change review, compare and merge, tracking lock | Materiaal history; Collaboration & sync | unsupported | proposed | — (DOCX import rejects tracked changes; S06 revisions are whole-object) | P1 |
| WP17 Version history preview and restore | Browse, preview and restore earlier versions without losing current work | Restore from a history list | Named versions, visual comparison (WP16), branches | Materiaal history | with limitations | prototyped (host) | `docs/verification/materiaal-library-20261102/checks/sdk-host.stdout.txt` and `docs/verification/materiaal-library-20261102/checks/ui-host-corrected.stdout.txt` (passing host tests: read-only preview, restore as a new revision, stale selection cannot restore another revision, restore of a large document; plain text, whole-document revisions, no comparison); `docs/verification/materiaal-library-20261102/README.md` (native QEMU run, criterion U03 open; two history versions) | P1 |
| WP18 Proofing tools | Multilingual spelling, grammar, thesaurus, custom dictionaries, offline | Inline suggestions | Per-language dictionaries, rule sets | Text & font engine; Search & index | unsupported | proposed | — | P1 |
| WP19 Templates and mail merge | Templates with placeholders; merge from data sources | Start from template | Conditional merge fields, data-source mapping | Automation runtime; Asset library & linking | unsupported | proposed | — | P2 |
| WP20 Long-document publishing | Chapters as separate files with shared numbering and contents | Single long document | Book or master-document assembly | Materiaal history; Text & font engine | unsupported | proposed | — | P3 |
| WP21 Accessible document authoring | Heading structure, alt text, reading order, accessibility checker | Prompts for alt text | Checker rules, tagged export | Accessibility service; PDF & print engine | unsupported | proposed | — | P1 |
| WP22 DOCX import (plain text) | Open word-processor files with layout preserved | Open file; formatting loss is stated | Styles, tables, fields, images, numbering and tracked changes imported, with a fidelity report | Interchange converters | lossy | prototyped (host) | `docs/verification/platform-docx-import-20261103.md` (paragraphs and runs to UTF-8 text; mandatory plain-text option drops run formatting; tables, fields, images, links, numbering, tracked changes and headers or footers are rejected) | P1 |
| WP23 DOCX export (plain text) | Share editable files with other suites | Export | Styles, tables, images and fields written; output opens without repair in other suites | Interchange converters | with limitations | prototyped (host) | `docs/verification/platform-docx-export-20261103.md` (three fixed parts, plain text; Unicode, tabs and empty paragraphs read back exactly by an independent reader; CR, CRLF, NUL and XML-invalid characters rejected; never opened in an office suite) | P1 |
| WP24 Full-fidelity DOCX and ODF round trip | Open, edit and save without layout or feature drift | Open and save | Feature-level fidelity report per document | Interchange converters | unsupported | proposed | — (import and export slices are independent; no round trip tested) | P1 |
| WP25 Large-document performance | Responsive editing of documents with hundreds of pages and images | — | Incremental layout, background repagination | Text & font engine; Materiaal history | unsupported | proposed | — (storage criterion S05 passes as a dependency only) | P2 |
| WP26 Document automation | Recorded and scripted automation | Recorded actions | Scripting API under capability limits | Automation runtime; Plugin sandbox | unsupported | proposed | — | P2 |
| WP27 Print and PDF output of documents | Faithful print and PDF with embedded fonts, links, bookmarks and tags | Print or export | Archival and accessible PDF profiles | PDF & print engine | unsupported | proposed | — | P1 |
| WP28 Find and replace | Find and replace text, formatting and special characters, with case, whole-word and pattern options | Find box | Pattern matching, replace formatting, search within a selection | Text & font engine | unsupported | proposed | — | P1 |

#### 3.1.2 Spreadsheets

| Capability | Professional expectation | Beginner path | Professional depth | Shared dependency | BaryonOS support today | Evidence level | Evidence | Priority |
|---|---|---|---|---|---|---|---|---|
| SS01 CSV table import, cell edit and export | Delimited text opened and saved exactly | Open CSV, edit cells, save | Dialect and encoding options, type inference | Interchange converters | with limitations | prototyped (host) | `docs/verification/platform-csv-table-20261103.md` (RFC 4180 profile; 4096 x 256; no inference, no undo) | P1 |
| SS02 Formula entry and dependency recalculation | Automatic recalculation on edit with cycle detection | Type a formula referring to cells | Manual or iterative calculation, dependency inspection | Calculation engine | with limitations | prototyped (host) | `docs/verification/platform-spreadsheet-recalc-20261103.md` (four operators, 64 cells, decimal arithmetic) | P0 |
| SS03 Function library | Hundreds of mathematical, statistical, financial, text, date and lookup functions | Function picker with help | Array formulas, user-defined functions | Calculation engine | unsupported | proposed | — (prototype has no functions) | P1 |
| SS04 Compatible calculation semantics | Same results as established tools: binary64 numbers, 15-digit precision, date systems, error values, documented deviations | Same answers as colleagues | Compatibility mode, conformance to an open formula standard, deviation report | Calculation engine; Interchange converters | proprietary research | proposed | — (prototype uses 9-digit decimal; no parity claim) | P0 |
| SS05 Reference model | Ranges, relative and absolute references, named ranges, multiple sheets | Click to reference | Structured and cross-sheet references | Calculation engine | unsupported | proposed | — | P1 |
| SS06 Number, date and currency formats | Locale-aware display formats | Format presets | Custom format codes | Calculation engine; Text & font engine | unsupported | proposed | — | P1 |
| SS07 Typed cells and data validation | Text, number and date types; validation rules and lists | Drop-down lists | Rule-based validation with messages | Calculation engine | unsupported | proposed | — | P1 |
| SS08 Sort, filter and conditional formatting | Multi-key sorting, filters and rule-based formatting | Sort by a column | Formula-based rules, custom sort orders | Calculation engine | unsupported | proposed | — | P1 |
| SS09 Large datasets | Responsive scrolling and recalculation near a million rows | — | Multi-threaded recalculation, columnar storage | Calculation engine; Materiaal history; Background job queue | unsupported | proposed | — (prototypes: 64 formula cells; 4096 CSV rows) | P2 |
| SS10 Pivot tables and summarisation | Field-based pivots, grouping, calculated fields, refresh | Suggested pivots | Calculated fields, filters, source refresh | Calculation engine | unsupported | proposed | — | P2 |
| SS11 Charts | Standard types, combination charts, secondary axes, linked to ranges | Suggested chart | Trendlines, templates, exact axis control | 2D vector engine; Calculation engine | unsupported | proposed | — | P1 |
| SS12 Data import and refreshable connections | Repeatable import and transformation from files, databases and web data | Import wizard | Transformation steps, scheduled refresh | Interchange converters; Internet protocols; Automation runtime | unsupported | proposed | — | P2 |
| SS13 What-if analysis and statistics | Solve a cell for a target value, scenarios, optimisation, statistical tools | Solve a cell for a target value | Constraint solver, regression | Calculation engine | unsupported | proposed | — | P3 |
| SS14 Spreadsheet automation | Recorded macros, scripts, custom functions | Record actions | Script editor, sandboxed custom functions | Automation runtime; Plugin sandbox | unsupported | proposed | — | P2 |
| SS15 XLSX and ODS round trip | Formulas, formats and charts preserved through open and save | Open file | Per-feature fidelity report | Interchange converters; Calculation engine | unsupported | proposed | — | P1 |
| SS16 Undo and redo | Multi-level undo across edits | Ctrl+Z | Per-sheet history | Materiaal history; Input & command system | unsupported | proposed | — (calculator undo exists; its native gate on the merged tree is unknown) | P1 |
| SS17 Workbook collaboration | Comments, concurrent editing, change history, protected ranges | Share | Cell history, permissions per range | Collaboration & sync | unsupported | proposed | — | P2 |
| SS18 Formula auditing and protection | Trace precedents and dependents, step evaluation, sheet protection | Error hints | Dependency tracing, step-by-step evaluation | Calculation engine | unsupported | proposed | — (prototype reports explicit cycle and reference errors) | P2 |
| SS19 Formula-injection safety on exchange | Warn about formula-like values from untrusted sources | Safe defaults | Policy per source | Interchange converters | unsupported | proposed | — (CSV prototype keeps formula-like text literal and states it is not injection-safe) | P1 |
| SS20 Find and replace in workbooks | Find and replace in values and formulas across sheets | Find box | Search in formulas or values, whole-cell and pattern matching, replace across sheets | Calculation engine | unsupported | proposed | — | P1 |

#### 3.1.3 Presentations

| Capability | Professional expectation | Beginner path | Professional depth | Shared dependency | BaryonOS support today | Evidence level | Evidence | Priority |
|---|---|---|---|---|---|---|---|---|
| PR01 Slide navigation and preview | Present slides with next, previous and select | Arrow keys | Speaker view on a second display, slide overview grid | GPU compositing; Text & font engine | with limitations | prototyped (host) | `docs/verification/platform-presentation-preview-20261103.md` (own text-deck profile; host raster only) | P1 |
| PR02 Slide raster frame export (PPM) | Export slides at chosen resolutions | Export | Resolution, colour and notes-page options | Pixel pipeline; Text & font engine | with limitations | prototyped (host) | `docs/verification/platform-presentation-preview-20261103.md` (PPM at 640x360 or 1280x720; no colour-fidelity claim) | P2 |
| PR03 Slide editing with layouts and placeholders | Masters, layouts and placeholders | Choose a layout | Master editing, custom placeholders | Text & font engine; 2D vector engine | unsupported | proposed | — | P1 |
| PR04 Themes and templates | Theme colours and fonts applied globally | Pick a theme | Custom and brand themes | Asset library & linking | unsupported | proposed | — | P1 |
| PR05 Precise layout controls | Guides, snapping, align and distribute, numeric position and size | Dynamic alignment guides | Numeric transforms, grids | 2D vector engine; Input & command system | unsupported | proposed | — | P1 |
| PR06 Animations and transitions | Entrance, emphasis and exit effects, timing, motion paths, transitions | Presets | Timeline, triggers, motion paths | GPU compositing | unsupported | proposed | — | P2 |
| PR07 Embedded media | Linked or embedded video and audio with trimming | Insert video | Trim, poster frame, linked media | Media framework; Asset library & linking | unsupported | proposed | — | P2 |
| PR08 Presenter tools | Notes, timer, rehearsal, speaker view on a second display | Start show | Recorded slide timings, recording | GPU compositing; Input & command system | unsupported | proposed | — | P2 |
| PR09 Charts, tables and diagrams in slides | Native objects, optionally linked to data | Insert | Live link to spreadsheets | 2D vector engine; Calculation engine | unsupported | proposed | — | P2 |
| PR10 Deck export to PDF and video | Faithful PDF and video export | Export | Handouts, video encoding settings | PDF & print engine; Media framework | unsupported | proposed | — | P2 |
| PR11 PPTX and ODP import and export | Layout and animation fidelity through open and save | Open file | Per-feature fidelity report | Interchange converters | unsupported | proposed | — (preview uses its own text format) | P1 |
| PR12 Accessible slides | Reading order, alt text, captions, checker | Prompts | Reading-order editing | Accessibility service | unsupported | proposed | — | P2 |
| PR13 Slide image export at chosen resolution and colour space | Export slides as images at any resolution, in delivery formats, with a colour profile | Export slides as images | Resolution, format, colour-space and notes-page options | Pixel pipeline; Colour management; Interchange converters | unsupported | proposed | — (only fixed-size PPM frames exist; see PR02) | P2 |

#### 3.1.4 Notebooks, knowledge management, diagrams, whiteboards and research organisation

| Capability | Professional expectation | Beginner path | Professional depth | Shared dependency | BaryonOS support today | Evidence level | Evidence | Priority |
|---|---|---|---|---|---|---|---|---|
| NK01 Plain-text notes in collections | Notebooks and sections or collections, tags | Create a note in a collection | Nested structure, tags, templates | Materiaal history; Search & index | with limitations | prototyped (host) | `docs/verification/materiaal-library-20261102/checks/sdk-host.stdout.txt` and `docs/verification/materiaal-library-20261102/checks/ui-host-corrected.stdout.txt` (passing host tests: collections with metadata filters, empty-only collection delete, missing or stale collection preserves the document; plain text; no tags or nesting); `docs/verification/materiaal-library-20261102/README.md` (native QEMU run, criterion U03 open; four collections) | P1 |
| NK02 Rich notes | Mixed text, images, ink, attachments and audio on one page | Type or paste anything | Ink, recordings, attachments | Text & font engine; Pixel pipeline; Input & command system | unsupported | proposed | — | P2 |
| NK03 Local document-name search | Find a note or document by name in the local library | Search box | Combined with collection and metadata filters | Search & index | with limitations | prototyped (host) | `docs/verification/materiaal-library-20261102/checks/ui-host-corrected.stdout.txt` (passing host test: search reads real document names only; no full-text or metadata search); `docs/verification/materiaal-library-20261102/README.md` (native QEMU run, criterion U03 open) | P1 |
| NK04 Links and backlinks | Links between notes, backlinks, graph view | Link while typing | Transclusion, graph view | Materiaal history; Search & index | unsupported | proposed | — | P2 |
| NK05 Research organisation | Captured sources with provenance, annotations, citation export | Save a clip | Annotation sync, citation export | Asset library & linking; Search & index | unsupported | proposed | — | P2 |
| NK06 Diagrams | Attached connectors, shape libraries, automatic layout | Templates | Auto-layout, data-linked diagrams | 2D vector engine | unsupported | proposed | — | P2 |
| NK07 Whiteboards | Infinite canvas, sticky notes, ink, multi-user sessions | Sticky notes | Frames, voting, templates, presence | 2D vector engine; Collaboration & sync; Input & command system | unsupported | proposed | — | P2 |
| NK08 Handwriting and pen input in notes | Pressure-sensitive ink with recognition | Draw | Ink-to-text, shape recognition | Input & command system | unsupported | proposed | — | P3 |
| NK09 Offline-first notebook sync | Works offline and merges later without loss | Automatic | Conflict review | Collaboration & sync | unsupported | proposed | — (sync criteria P01–P04 open) | P1 |
| NK10 Note import and export | Migration from text formats and other note tools | Import | Mapping report | Interchange converters | unsupported | proposed | — | P2 |
| NK11 Full-text and metadata search across notes and documents | Search inside content and metadata of every note and document | Search box | Operators, filters, recognised text | Search & index | unsupported | proposed | — (only name search exists, NK03; U05 open) | P1 |

#### 3.1.5 Email, calendars, contacts, scheduling, tasks and collaboration

| Capability | Professional expectation | Beginner path | Professional depth | Shared dependency | BaryonOS support today | Evidence level | Evidence | Priority |
|---|---|---|---|---|---|---|---|---|
| EM01 Mail accounts and mailbox sync | Standards-based mail access with offline cache | Set up by address | Multiple accounts, server-side rules, delegation | Internet protocols; Signing & identity | unsupported | proposed | — (N02–N04 open) | P1 |
| EM02 Reading, threading, search and rules | Conversation view, fast search, filters | Inbox | Rules, saved searches, shared mailboxes | Search & index; Internet protocols | unsupported | proposed | — | P1 |
| EM03 Composing | Rich and plain text, attachments, signatures, templates | Compose | Templates, scheduled send | Text & font engine; Internet protocols | unsupported | proposed | — | P1 |
| EM04 Mail security | Signed and encrypted mail, sender verification, phishing warnings | Automatic indicators | Key management, organisational policy | Signing & identity | unsupported | proposed | — (B01 open) | P2 |
| EM05 Calendars | Events with recurrence, time zones and invitations | Add event | Shared calendars, free/busy | Internet protocols; Collaboration & sync | unsupported | proposed | — | P1 |
| EM06 Scheduling across people and resources | Free/busy lookup, time proposals, room booking | Suggest a time | Delegates, resources | Internet protocols; Collaboration & sync | unsupported | proposed | — | P2 |
| EM07 Contacts | Contact cards with sync and groups | Add contact | Duplicate merge, directory lookup | Internet protocols; Search & index | unsupported | proposed | — | P1 |
| EM08 Tasks and to-dos | Due dates, reminders, recurring tasks, lists | Add task | Subtasks, assignment | Collaboration & sync | unsupported | proposed | — | P2 |
| EM09 Reminders and notifications | Reliable reminders even when apps are closed | Default reminders | Quiet hours, per-source rules | Background job queue | unsupported | proposed | — | P1 |
| EM10 Sharing and team collaboration | Share with people, permission levels, activity history, revocation | Share button | Granular permissions, revocation, audit history | Collaboration & sync; Signing & identity | unsupported | proposed | — (P01–P04 open) | P1 |
| EM11 Real-time co-editing | Simultaneous editing with presence and merge | Automatic | Offline merge, conflict review | Collaboration & sync; Materiaal history | unsupported | proposed | — | P2 |

#### 3.1.6 Databases, forms, project management and business workflows

| Capability | Professional expectation | Beginner path | Professional depth | Shared dependency | BaryonOS support today | Evidence level | Evidence | Priority |
|---|---|---|---|---|---|---|---|---|
| DB01 Desktop database | Relational tables, queries and reports | Templates | Visual and text queries, indexes, constraints | Search & index; Calculation engine | unsupported | proposed | — | P2 |
| DB02 Forms and surveys | Form builder with validation and branching | Template forms | Conditional logic, validation | Accessibility service | unsupported | proposed | — | P2 |
| DB03 Reports from data | Grouped reports with totals and export | Report wizard | Layout control, export | PDF & print engine | unsupported | proposed | — | P3 |
| DB04 List and tracker views | Table, board, calendar and timeline views over the same records | Choose a view | Filters, grouping, computed fields | Calculation engine; Materiaal history | unsupported | proposed | — | P2 |
| DB05 Project scheduling | Task dependencies, Gantt view, resources, critical path | Simple task list | Critical path, levelling, baselines | Calculation engine | unsupported | proposed | — | P3 |
| DB06 Business workflows and approvals | Triggers, approvals, notifications, audit log | Templates | Conditional flows, audit history | Automation runtime; Background job queue; Signing & identity | unsupported | proposed | — | P3 |
| DB07 External data connectors | Connections to databases and web services with isolated credentials | Connect wizard | Scheduled refresh | Internet protocols; Plugin sandbox | unsupported | proposed | — | P3 |

#### 3.1.7 PDF creation, annotation, editing, signing and export

| Capability | Professional expectation | Beginner path | Professional depth | Shared dependency | BaryonOS support today | Evidence level | Evidence | Priority |
|---|---|---|---|---|---|---|---|---|
| PD01 PDF viewing | Fast, accurate rendering with search, bookmarks and layers | Open | Measurement, output preview | PDF & print engine; Text & font engine; Colour management | unsupported | proposed | — | P1 |
| PD02 PDF creation from any application | Fonts embedded, links and tags preserved | Save as PDF | Presets for archive, print and accessibility | PDF & print engine | unsupported | proposed | — | P1 |
| PD03 PDF annotation | Interoperable highlights, notes, stamps and drawings | Highlight | Review summaries, flattening | PDF & print engine; Collaboration & sync | unsupported | proposed | — | P1 |
| PD04 Page organisation | Merge, split, reorder, rotate, extract | Drag thumbnails | Batch operations | PDF & print engine | unsupported | proposed | — | P1 |
| PD05 PDF content editing | Edit existing text and images | Click to edit | Reflow, font-substitution warnings | PDF & print engine; Text & font engine | unsupported | proposed | — | P2 |
| PD06 PDF forms | Fill, create and calculate interactive forms | Fill form | Field calculation and validation | PDF & print engine; Calculation engine | unsupported | proposed | — | P2 |
| PD07 Digital signatures | Certificate-based signing, validation, timestamps, long-term validation | Sign | Certificate validation, timestamps | Signing & identity; PDF & print engine | unsupported | proposed | — (B01 open) | P2 |
| PD08 Redaction and sanitisation | Verifiable removal of content and hidden data | Redact tool | Pattern-based redaction, metadata scrub | PDF & print engine | unsupported | proposed | — | P2 |
| PD09 Text recognition (OCR) | Searchable text from scans, on-device | Automatic | Language packs | Pixel pipeline; Search & index | unsupported | proposed | — | P2 |
| PD10 Standards-conformant professional export | Archival, print and accessible PDF profiles with preflight | Presets | Preflight profiles and fixes | PDF & print engine; Colour management; Accessibility service | unsupported | proposed | — | P2 |
| PD11 PDF comparison | Visual and text differences between versions | Compare | Filtered difference report | PDF & print engine | unsupported | proposed | — | P3 |

### 3.2 Graphics, photography and publishing

| Capability | Professional expectation | Beginner path | Professional depth | Shared dependency | BaryonOS support today | Evidence level | Evidence | Priority |
|---|---|---|---|---|---|---|---|---|
| GR01 Image viewing with orientation, fit, zoom and pan | Accurate display, fast zoom, 1:1 pixel inspection | Open an image | Pixel inspection, high-bit-depth samples | Pixel pipeline; GPU compositing | with limitations | prototyped (host) | `docs/verification/platform-m7-exif-viewer-20261103.md` (viewer host-tested; the native VM run tested the decoder only; no colour management) | P1 |
| GR02 Rectangular crop to a new file | Crop with aspect presets, straightening, non-destructive | Drag a crop box | Numeric crop, straighten, re-editable crop | Pixel pipeline | lossy | prototyped (host) | `docs/verification/platform-image-crop-20261103.md` (PNG or JPEG to a new 8-bit PNG, source unchanged; 16-bit PNG rejected; ICC, gamma and metadata always dropped; crop not re-editable) | P1 |
| GR03 Raster export to BMP | Export with explicit quality, bit depth, profile and metadata options | Export | Bit depth, profile embedding, metadata control | Pixel pipeline; Interchange converters | lossy | prototyped (host) | `docs/verification/platform-image-export-20261103.md` (PNG or JPEG to 24-bit BMP only; 16-bit quantised to 8; alpha flattened; no ICC or metadata) | P1 |
| GR04 Layers and groups | Many layers, groups, opacity, clipping | Layers list | Groups, clipping, linked or embedded editable layers | Pixel pipeline; Materiaal history | unsupported | proposed | — | P1 |
| GR05 Masks | Layer, vector and clipping masks; selection-based masks | Brush a mask | Vector masks, density and feather | Pixel pipeline | unsupported | proposed | — | P1 |
| GR06 Blend modes and compositing | Standard blend modes with correct colour arithmetic | Mode list | Blend ranges, linear-light compositing | Pixel pipeline; Colour management; GPU compositing | unsupported | proposed | — | P1 |
| GR07 Selection tools | Geometric, freehand, colour-range and assisted subject selection | Automatic subject selection | Edge refinement, channels, paint a selection as a temporary mask | Pixel pipeline | unsupported | proposed | — | P1 |
| GR08 Non-destructive adjustments and effects | Adjustment layers and live, re-editable filters | Presets | Stackable, maskable, re-editable effects | Pixel pipeline; Node-graph engine; GPU compositing | unsupported | proposed | — | P1 |
| GR09 High bit depth and HDR images | 16-bit integer and 32-bit float pipelines end to end | Automatic | Linear float working space, HDR display | Pixel pipeline; Colour management | unsupported | proposed | — (the codec decodes 16-bit PNG with exact sample preservation and the host viewer rounds to 8 bit only for display, per `docs/implementation/NATIVE-IMAGES.md`; crop rejects 16-bit PNG; BMP export quantises 16 to 8 bit; no end-to-end 16-bit or float pipeline) | P0 |
| GR10 Colour profiles and soft-proofing | Profile-aware display and conversion, rendering intents, CMYK, gamut warning | Automatic | Rendering intents, separations, soft proof | Colour management | unsupported | proposed | — (prototypes run with unmanaged colour) | P0 |
| GR11 Retouching | Healing, cloning, filling a selection from surrounding pixels, warping | Remove a blemish with one stroke | Frequency-separation style workflows | Pixel pipeline | unsupported | proposed | — | P2 |
| GR12 Undo history and snapshots | Deep undo, history list, snapshots | Ctrl+Z | Snapshots, non-linear history | Materiaal history; Input & command system | unsupported | proposed | — | P1 |
| GR13 Large-canvas performance | Tiled, accelerated editing of very large images | — | Out-of-core tiles, scratch storage | Pixel pipeline; GPU compositing; Background job queue | unsupported | proposed | — (crop prototype: 4096 px per side, 1 Mpixel; BMP export prototype: 16384 px per axis, 4 Mpixel) | P2 |
| GR14 Layered-file interchange | Open and save layered raster files with fidelity | Open file | Per-feature fidelity report | Interchange converters | proprietary research | proposed | — | P1 |
| GR15 RAW decoding and development | Demosaicing, white balance, exposure, lens and noise correction | Automatic | Camera profiles, lens profiles, noise reduction | Pixel pipeline; Colour management | proprietary research | proposed | — (vendor RAW formats are largely undocumented; one published raw format exists) | P2 |
| GR16 Photo cataloguing | Import, ratings, collections, keywords, previews for large libraries | Automatic import | Rule-based collections, hierarchical keywords | Materiaal history; Search & index; Asset library & linking | unsupported | proposed | — | P2 |
| GR17 Photo metadata read, write and preserve | Read, write and preserve camera and descriptive metadata | Show information | Batch templates, sidecar handling | Interchange converters; Search & index | unsupported | proposed | — (only EXIF orientation is read, for display, which is counted under GR01; crop and BMP export drop all metadata) | P1 |
| GR18 Batch processing and presets | Apply edits and exports to many images | Copy settings | Recorded actions, scripted batches | Automation runtime; Background job queue | unsupported | proposed | — | P2 |
| GR19 Advanced colour correction | Curves, levels, hue/saturation, local adjustments, look tables | Automatic tonal correction | Per-channel curves, masked corrections | Pixel pipeline; Colour management | unsupported | proposed | — | P1 |
| GR20 Vector paths and shapes | Precise Bezier editing and boolean path operations | Shape tools | Node editing, offsets, path operations | 2D vector engine | unsupported | proposed | — | P1 |
| GR21 Precision geometry | Snapping to grids and points, guides, numeric transforms, units | Dynamic alignment guides | Align and distribute, constraints, numeric entry | 2D vector engine; Input & command system | unsupported | proposed | — | P1 |
| GR22 Vector typography | Type on paths, outline conversion, OpenType and variable fonts | Text tool | Glyph access, variable axes | Text & font engine; 2D vector engine | unsupported | proposed | — | P1 |
| GR23 Symbols in illustration | Reusable symbols whose instances update | Insert symbol | Overrides, shared libraries | Asset library & linking; 2D vector engine | unsupported | proposed | — | P2 |
| GR24 Illustration tools | Gradient meshes, patterns, vector brushes, image tracing | Presets | Mesh editing, tracing parameters | 2D vector engine | unsupported | proposed | — | P2 |
| GR25 Vector interchange | SVG, PDF and other vector formats with fidelity | Open file | Per-feature fidelity report | Interchange converters | proprietary research | proposed | — | P1 |
| GR26 Painting brush engine | Natural-media brushes with dynamics and texture | Preset brushes | Brush designer, dynamics curves | Pixel pipeline; GPU compositing | unsupported | proposed | — | P2 |
| GR27 Pen and tablet input | Pressure, tilt and rotation with low stroke latency | Plug in a tablet | Pressure curves, per-tool mapping | Input & command system; GPU compositing | unsupported | proposed | — (H02 open) | P0 |
| GR28 Painting canvas navigation | Rotate and mirror canvas, reference views | Gestures | Reference panels, mirrored drawing | GPU compositing; Input & command system | unsupported | proposed | — | P2 |
| GR29 Page layout | Frames, master pages, threaded text, baseline grid | Templates | Masters, threading, grids | Text & font engine; 2D vector engine | unsupported | proposed | — | P2 |
| GR30 Publishing typography | Multi-line composition, optical margins, hyphenation, nested styles | Defaults | Composer settings, pattern-based styles | Text & font engine | unsupported | proposed | — | P2 |
| GR31 Prepress and print production | Preflight, separations, overprint preview, bleed, print-standard PDF | Print preset | Ink management, overprint preview | PDF & print engine; Colour management | unsupported | proposed | — | P2 |
| GR32 Linked resources and relinking | Linked files with status, relink, and gathering of all linked files for handoff | Automatic relink | Gather linked files and fonts for handoff, link status overview | Asset library & linking | unsupported | proposed | — | P1 |
| GR33 Digital publishing | Reflowable e-book and interactive document export | Export | Fixed versus reflowable output, accessibility metadata | Interchange converters; Accessibility service | unsupported | proposed | — | P3 |
| GR34 Shared palettes and swatches | Shared colour palettes across applications, spot colours | Pick from palette | Named swatch libraries, spot inks | Asset library & linking; Colour management | unsupported | proposed | — | P1 |
| GR35 Font management | Activate fonts per project, detect and resolve missing fonts | Automatic | Font collections, substitution warnings | Text & font engine; Asset library & linking | unsupported | proposed | — (native UI uses one bitmap font backend) | P0 |
| GR36 Reusable components across applications | Insert library items in any app; linked updates | Drag from library | Versioned components | Asset library & linking; Materiaal history | unsupported | proposed | — | P1 |
| GR37 Brand kits across applications | One brand set of logos, colours, fonts and templates available in every application | Apply the brand | Locked brand items, versioned updates, brand-usage check | Asset library & linking; Colour management | unsupported | proposed | — | P2 |
| GR38 Raster export with format, bit-depth, profile and metadata control | Export to common delivery formats with explicit quality, bit depth, colour profile and metadata choices | Export presets for screen and print | Profile embedding, metadata filtering, batch export | Pixel pipeline; Interchange converters; Colour management | unsupported | proposed | — (only BMP export exists; see GR03) | P1 |

### 3.3 Video, motion graphics and animation

| Capability | Professional expectation | Beginner path | Professional depth | Shared dependency | BaryonOS support today | Evidence level | Evidence | Priority |
|---|---|---|---|---|---|---|---|---|
| VI01 Media import and frame-accurate playback | Frame-accurate playback of common camera and delivery formats | Drag clips in | Variable frame rate handling, timecode | Media framework; GPU compositing | unsupported | proposed | — (H03 open) | P0 |
| VI02 Multitrack timeline editing | Multitrack video and audio with ripple, roll, slip and slide trimming | Simple timeline that closes gaps | Trim modes, track targeting, keyboard trimming | Media framework; Input & command system | unsupported | proposed | — | P1 |
| VI03 Multicamera editing | Synchronise angles by timecode or audio; switch angles | Automatic sync | Angle editing during playback | Media framework; Low-latency audio | unsupported | proposed | — | P2 |
| VI04 Timeline precision | Frame and sub-frame positions, timecode entry, sample-accurate audio | Snap to frames | Timecode entry, sub-frame audio edits | Media framework | unsupported | proposed | — | P1 |
| VI05 Proxies and optimised media | Proxy generation, toggle and relink | Automatic proxies | Proxy formats and resolution control | Media framework; Background job queue; Asset library & linking | unsupported | proposed | — | P1 |
| VI06 Transitions and keyframes | Transitions and keyframed parameters with curves | Presets | Curve editor, easing control | Media framework; Node-graph engine | unsupported | proposed | — | P1 |
| VI07 Non-destructive projects and media relinking | Source media never modified; relink after moves | Automatic relink | Relink by metadata, consolidate project | Asset library & linking; Materiaal history | unsupported | proposed | — | P1 |
| VI08 Colour grading | Primary and secondary correction, curves, qualifiers, windows, look tables, scopes | Automatic colour | Node-based grading, waveform and vector scopes | Colour management; GPU compositing; Node-graph engine | unsupported | proposed | — | P2 |
| VI09 HDR and colour-managed video pipeline | Scene-referred working spaces, HDR transfer functions, display mapping | Automatic | Working-space selection, HDR mastering | Colour management; GPU compositing | unsupported | proposed | — | P2 |
| VI10 Professional export and delivery | Delivery presets with explicit codec, bit depth, captions and loudness | Export preset | Custom codec settings, loudness normalisation | Media framework; Background job queue; Low-latency audio | unsupported | proposed | — | P1 |
| VI11 Captions and subtitles | Create, edit, import and burn in captions | On-device automatic captions | Caption styles and formats | Text & font engine; Media framework | unsupported | proposed | — | P2 |
| VI12 Motion graphics | Animated titles, shapes and reusable templates | Templates | Expressions, parameter linking | 2D vector engine; Node-graph engine; GPU compositing | unsupported | proposed | — | P2 |
| VI13 Compositing | Layer or node compositing, keying, tracking, rotoscoping masks | Presets | Keyer, planar tracking, rotoscoping | Node-graph engine; GPU compositing; Pixel pipeline | unsupported | proposed | — | P2 |
| VI14 Video effects and third-party effects | Effects library and isolated effect plugins | Drag an effect | Effect SDK, GPU effects | Plugin sandbox; GPU compositing | unsupported | proposed | — | P2 |
| VI15 2D character animation | Rigged and frame-by-frame animation with onion skinning | Preset 2D rigs | Bone rigs, onion skinning | 2D vector engine; Node-graph engine | unsupported | proposed | — | P3 |
| VI16 2D and 3D integration | 3D layers, cameras and imported scenes in compositions | Add 3D text | Imported scenes, camera tracking | 3D scene engine; GPU compositing | unsupported | proposed | — | P3 |
| VI17 Encoding and transcoding | Batch transcodes with presets | Convert | Hardware encoders, bitrate control | Media framework; Background job queue | unsupported | proposed | — | P1 |
| VI18 Project interchange | Exchange timelines with other editors and finishing tools | Export for another editor | Interchange with fidelity report | Interchange converters | proprietary research | proposed | — | P2 |
| VI19 Render queues | Queue, pause, resume and monitor renders in the background | Render | Watch folders, distributed rendering | Background job queue | unsupported | proposed | — | P1 |
| VI20 Audio for video | Mixing, synchronisation and loudness metering inside the editor | Automatic levels | Bus routing, meters | Low-latency audio; Media framework | unsupported | proposed | — | P1 |
| VI21 Vertical and social-format delivery | Reframe one edit for vertical, square and widescreen delivery with platform presets, safe areas and captions | Pick a delivery format | Subject-aware reframing, per-format edits, thumbnail export | Media framework; Background job queue | unsupported | proposed | — | P2 |

### 3.4 Audio production

| Capability | Professional expectation | Beginner path | Professional depth | Shared dependency | BaryonOS support today | Evidence level | Evidence | Priority |
|---|---|---|---|---|---|---|---|---|
| AU01 Sample-exact clip extraction | Sample-exact edits preserving format and channel order | Select a range and save | Metadata-aware edits | Interchange converters; Media framework | lossy | prototyped (host) | `docs/verification/platform-wav-clip-20261103.md` (PCM16 WAV only; samples, channel order and rate preserved exactly; extra chunks such as cue and loop metadata are dropped only on explicit confirmation, otherwise refused; no playback) | P1 |
| AU02 Playback and monitoring | Glitch-free low-latency playback | Play | Device and buffer selection | Low-latency audio | unsupported | proposed | — (no playback exists; H03 open) | P0 |
| AU03 Multitrack arrangement | Non-destructive multitrack editing with fades, crossfades and comping | Drag clips | Comping, grid editing, ripple | Low-latency audio; Materiaal history | unsupported | proposed | — | P1 |
| AU04 Recording | Multi-input recording with monitoring, punch-in and loop takes | Record button | Punch recording, take lanes | Low-latency audio | unsupported | proposed | — | P1 |
| AU05 Mixing | Faders, pan, sends, buses and metering | Simple mixer | Sends, groups, control groups | Low-latency audio | unsupported | proposed | — | P1 |
| AU06 Automation | Automation lanes for volume, pan and plugin parameters | Draw a curve | Read, write, touch and latch modes | Low-latency audio; Input & command system | unsupported | proposed | — | P2 |
| AU07 Routing | Flexible input/output, sidechains, multichannel and immersive formats | Defaults | Sidechains, surround and immersive buses | Low-latency audio | unsupported | proposed | — | P2 |
| AU08 Effects | High-quality equalisers, dynamics, reverbs and delays | Presets | Linear-phase processing, oversampling | Low-latency audio | unsupported | proposed | — | P1 |
| AU09 Mastering | Limiting, loudness targets, dithering and true-peak control | Target presets | True-peak limiting, dither choice, loudness metering | Low-latency audio | unsupported | proposed | — | P2 |
| AU10 Waveform editing | Sample-accurate destructive and non-destructive edits, fades | Select and delete | Zero-crossing edits, sample pencil | Low-latency audio | unsupported | proposed | — (AU01 covers range extraction only; no fades) | P1 |
| AU11 Spectral editing and repair | Spectral view, noise reduction, click and hum repair | Automatic denoise | Spectral selection and repair | Low-latency audio; GPU compositing | unsupported | proposed | — | P3 |
| AU12 MIDI sequencing and editing | Piano roll, quantisation, controller lanes, high-resolution MIDI | Draw notes | Controller lanes, per-note expression | Input & command system; Low-latency audio | unsupported | proposed | — | P2 |
| AU13 Virtual instruments | Built-in synthesisers and samplers | Presets | Multi-sampled instruments | Low-latency audio; Plugin sandbox | unsupported | proposed | — | P2 |
| AU14 Audio plugin hosting | Third-party plugins with delay compensation and crash isolation | Browse plugins | Per-plugin isolation, crash recovery | Plugin sandbox; Low-latency audio | unsupported | proposed | — | P1 |
| AU15 Sample-rate and sample-format handling | High-quality conversion; 24-bit and 32-bit float sessions | Automatic | Conversion quality, dither | Low-latency audio; Interchange converters | unsupported | proposed | — (AU01 is PCM16 only, without resampling) | P0 |
| AU16 Latency management | Measured round-trip latency and automatic compensation | Automatic | Manual offsets | Low-latency audio | unsupported | proposed | — | P0 |
| AU17 Audio export and delivery | Mixes and stems with metadata and loudness targets | Export | Stems, broadcast metadata | Interchange converters; Background job queue | unsupported | proposed | — | P1 |
| AU18 Session interchange | Exchange multitrack sessions with other audio workstations | Export stems | Session interchange with fidelity report | Interchange converters | proprietary research | proposed | — | P3 |
| AU19 Control surfaces and controllers | Map hardware controllers to functions | Automatic detection | Mapping editor | Input & command system | unsupported | proposed | — | P3 |

### 3.5 UI/UX and interactive design

| Capability | Professional expectation | Beginner path | Professional depth | Shared dependency | BaryonOS support today | Evidence level | Evidence | Priority |
|---|---|---|---|---|---|---|---|---|
| UX01 Wireframing | Fast low-fidelity layouts and annotations | Templates and basic components | Annotations, flow maps | 2D vector engine | unsupported | proposed | — | P2 |
| UX02 High-fidelity interface design | Pixel-precise frames with vector, text and effects | Templates | Effects, exact constraints | 2D vector engine; Text & font engine | unsupported | proposed | — | P2 |
| UX03 Responsive layouts | Constraints, stack layouts and breakpoints that adapt to size | Presets | Constraint and stack layouts, breakpoints | 2D vector engine | unsupported | proposed | — | P2 |
| UX04 Reusable components | Components, instances, variants and overrides | Insert a component | Variants, properties, overrides | Asset library & linking; Materiaal history | unsupported | proposed | — | P1 |
| UX05 Interactive prototyping and transitions | Clickable flows, transitions, conditional logic, variables | Link frames | Conditions, variables, animated transitions | GPU compositing; Node-graph engine | unsupported | proposed | — | P2 |
| UX06 Design tokens and variables | Colour, spacing and type tokens with modes and aliases | Pick a token | Aliases, modes, token exchange | Asset library & linking | unsupported | proposed | — (the repository's own token source is not a user feature) | P1 |
| UX07 Component libraries and design systems | Shared libraries with published, versioned updates | Subscribe to a library | Versioned publishing, change review | Asset library & linking; Collaboration & sync | unsupported | proposed | — | P2 |
| UX08 Accessibility inspection | Contrast, focus order, target size and semantic checks | Contrast warnings | Focus-order annotation for handoff | Accessibility service; Colour management | unsupported | proposed | — | P1 |
| UX09 Developer handoff | Measurements, specifications, asset export, token and code export | Export assets | Token and code export | Interchange converters | unsupported | proposed | — | P2 |
| UX10 Design collaboration | Comments and multi-user editing | Share | Presence, review mode | Collaboration & sync | unsupported | proposed | — | P2 |
| UX11 Design version management | History, branches and merge review | Restore | Branch and merge with review | Materiaal history; Collaboration & sync | unsupported | proposed | — (S06 storage revisions are linear) | P2 |
| UX12 Design-file interchange | Import and export of design files and vector assets | Export SVG | Fidelity report | Interchange converters | proprietary research | proposed | — | P3 |

### 3.6 3D and spatial creation

| Capability | Professional expectation | Beginner path | Professional depth | Shared dependency | BaryonOS support today | Evidence level | Evidence | Priority |
|---|---|---|---|---|---|---|---|---|
| TD01 Mesh modelling | Polygon modelling with non-destructive modifier stacks | Primitives | Modifier stacks, topology tools | 3D scene engine | unsupported | proposed | — | P2 |
| TD02 Precision modelling | Exact dimensions, snapping, units and constraints | Snapping | Numeric transforms, constraints | 3D scene engine; Input & command system | unsupported | proposed | — | P3 |
| TD03 Sculpting | Multiresolution and dynamic-topology sculpting | Brushes | Multiresolution, remeshing | 3D scene engine; GPU compositing; Input & command system | unsupported | proposed | — | P3 |
| TD04 Scene management | Hierarchies, collections, instancing, references for large scenes | Scene outline | Instancing, external references | 3D scene engine; Asset library & linking | unsupported | proposed | — | P2 |
| TD05 UV editing | Unwrapping, seams and packing | Automatic unwrap | Seams, packing control | 3D scene engine | unsupported | proposed | — | P3 |
| TD06 Materials and physically based rendering | Physically based materials with real-time and path-traced preview | Material presets | Node-based materials | 3D scene engine; Node-graph engine; GPU compositing | unsupported | proposed | — | P2 |
| TD07 Textures and baking | Texture maps and baking | Automatic baking | Bake sets | 3D scene engine; Pixel pipeline | unsupported | proposed | — | P3 |
| TD08 Lighting and cameras | Physical lights, environment lighting, camera models | Presets | Physical camera, depth of field | 3D scene engine; Colour management | unsupported | proposed | — | P2 |
| TD09 3D animation | Keyframes, curve editing, non-linear clips | Automatic keying | Curve editor, non-linear animation | 3D scene engine; Node-graph engine | unsupported | proposed | — | P3 |
| TD10 Rigging | Skeletons, skin weights, inverse kinematics, constraints | Automatic rig | Constraints, drivers | 3D scene engine | unsupported | proposed | — | P3 |
| TD11 Rendering | Offline and real-time rendering with passes and distribution | Render | Render passes, distributed rendering | 3D scene engine; GPU compositing; Background job queue | unsupported | proposed | — | P2 |
| TD12 Procedural and node-based workflows | Node graphs for materials and geometry | Node presets | Custom node groups | Node-graph engine | unsupported | proposed | — | P2 |
| TD13 3D painting and material authoring | Painting across surfaces, layered materials | Procedural material presets that adapt to geometry | Layered materials with masks | 3D scene engine; Pixel pipeline; Input & command system | unsupported | proposed | — | P3 |
| TD14 3D asset management | Asset browser with previews, tags and versions | Drag an asset | Versioning, metadata | Asset library & linking; Search & index | unsupported | proposed | — | P2 |
| TD15 Pipeline interoperability | Exchange scenes, runtime assets and materials with graphics and game pipelines | Export preset | Layered scenes, fidelity report | Interchange converters; 3D scene engine | proprietary research | proposed | — | P1 |
| TD16 Spatial preview | Preview scenes on spatial or head-mounted displays | — | Device-specific preview | 3D scene engine | unsupported | proposed | — | P3 |

### 3.7 Cross-cutting suite capabilities

These rows apply to every application in sections 3.1 to 3.6. They answer
addendum sections 1, 5 and 6 (reliability, configurable workspaces, familiar
shortcuts, clear recovery, accessibility users, students and teachers).

| Capability | Professional expectation | Beginner path | Professional depth | Shared dependency | BaryonOS support today | Evidence level | Evidence | Priority |
|---|---|---|---|---|---|---|---|---|
| XC01 Autosave and backup copies | Periodic recovery data or automatic save at a chosen interval; backup of the previous version on save | On by default | Interval, location and retention control; per-document choice between recovery data and saving the document | Materiaal history; Background job queue | unsupported | proposed | — (the Materiaal app refuses to stop while a draft is dirty and never autosaves; host tests keep in-session drafts on failure, see WP01) | P1 |
| XC02 Crash and power-loss recovery of open work | After a crash or power loss, unsaved work is offered for restore together with its relation to the last saved version | Restore prompt after restart | Compare recovered and saved versions; recover per document | Materiaal history | unsupported | proposed | — (S02 journal and cold recovery are accepted for stored data only; no restore of unsaved application state) | P1 |
| XC03 Configurable workspaces | Arrange, save, switch and reset panel layouts per task | Task-based default layouts | Saved and shared layouts, per-display arrangements | Input & command system | unsupported | proposed | — | P1 |
| XC04 Command and shortcut customisation | Rebind any command, find commands by name, detect conflicts, store bindings per user or per template | Familiar defaults (ADR-058) | Multi-key sequences, per-context keymaps, keymap export and import | Input & command system | unsupported | proposed | — (native keyboard runs exist under open U02; no user-defined bindings) | P0 |
| XC05 Familiar keymap presets and import | Shortcut presets matching conventions users already know; import from other tools where the format is documented | Choose a preset at first start | Per-application presets, report of unmatched commands | Input & command system; Interchange converters | unsupported | proposed | — | P1 |
| XC06 Accessible suite applications | Every editor usable with a screen reader, by keyboard only, at large scale, in high contrast and with reduced motion | System settings apply automatically | Accessible canvases, grids and timelines; keyboard navigation of complex views | Accessibility service; Input & command system | unsupported | proposed | — (an app-local semantic tree has host tests but no screen-reader output; U04 open; see section 4.1) | P1 |
| XC07 In-app learning and contextual help | Contextual help, guided first tasks and practice material that let existing training transfer | Guided first project | Offline searchable reference for every command | Search & index | unsupported | proposed | — | P2 |
| XC08 Education and classroom use | Teachers distribute, collect and return work with feedback; students work offline | Hand in a document | Class rosters, deadlines, rubric-based feedback | Collaboration & sync; Signing & identity | unsupported | proposed | — | P3 |

## 4. Shared infrastructure summary

### 4.1 Dependencies ranked by row count

The 23 names below are the controlled vocabulary of the "Shared dependency"
column; every name is used at least once. "Rows" counts the rows that name the
dependency explicitly. It measures breadth of reuse, not engineering size or
importance (judgement). Thirteen rows are themselves P0 because they define a
shared engine or the shared command system (for example GR09, GR10, AU02,
AU15, AU16, XC04).

| Rank | Shared dependency | Rows | What the repository shows today | Open criteria and key standards |
|---|---|---|---|---|
| 1 | Text & font engine | 32 | Native UI draws text through one bitmap font backend with an atlas fallback; no shaping, bidirectional text or variable fonts | No criterion yet; OpenType [23], UAX #9 [24] |
| 2 | Interchange converters | 26 | Five host-prototyped rows (WP22, WP23, SS01, GR03, AU01), each bounded or lossy; none native | Native parsing must run in isolated processes (K03, K04 accepted); ECMA-376 [3], OpenFormula [5] |
| 3 | Materiaal history | 22 | S06 accepted (collections, metadata, edit history; two retained versions in the tested profile), with S02, S05 and S08; application-level plain-text history restore and collections host-tested (WP17, NK01) and run natively under open U03 | U03 open |
| 3 | GPU compositing | 22 | U01 accepted: native compositor and session; GPU acceleration not demonstrated | H03 open |
| 3 | Pixel pipeline | 22 | Host crop, export and viewer prototypes (8-bit output, unmanaged colour); the codec decodes 16-bit PNG exactly; a native decoder fixture run outside any criterion | H03 open; OpenEXR [27], DNG [35] |
| 6 | Asset library & linking | 21 | Nothing beyond S06 collections and metadata | Design Tokens draft [33] |
| 6 | Input & command system | 21 | Native keyboard and clipboard runs (U02 open); calculator undo integrated, native gate not executed; no user-defined bindings | U02, H02 open |
| 8 | Calculation engine | 19 | Host decimal recalculation prototype: 64 cells, four operators, nine fractional digits | OpenFormula [5]; semantics research SS04 |
| 9 | Low-latency audio | 18 | No playback; host PCM16 clip extraction only | H03 open; EBU R 128 [32], MIDI 2.0 [31] |
| 10 | 2D vector engine | 17 | Nothing | SVG 2 [34] |
| 11 | PDF & print engine | 16 | Nothing | PDF and archival profiles not checked against an opened source (section 6) |
| 11 | Colour management | 16 | Nothing; every image prototype drops or ignores profiles | ICC [13], OpenColorIO [14] |
| 13 | Search & index | 15 | Local document-name search host-tested (NK03) and run natively under open U03; no full-text index | U05 open |
| 13 | Collaboration & sync | 15 | Nothing | P01–P04, N02–N04 open; privacy contract excludes a vendor cloud |
| 13 | 3D scene engine | 15 | Nothing | OpenUSD [28], glTF 2.0 [29], MaterialX [30] |
| 16 | Media framework | 14 | Nothing | H03 open; OpenTimelineIO [26], BT.2100 [15] |
| 17 | Background job queue | 13 | K05 accepted (process and service management) as a base | — |
| 18 | Node-graph engine | 10 | Nothing | MaterialX [30] |
| 19 | Internet protocols | 8 | N01 accepted (packet and transport stack) | N02–N04 open; IMAP, iCalendar, CalDAV, vCard [19–22] |
| 20 | Accessibility service | 7 | App-local semantic tree with host tests; its native probe failed (timeout, no save) and the later tree records no native execution; a native keyboard baseline passed on other code without it; no passing native run of the tree | U04 open; WCAG 2.2 [25] |
| 21 | Automation runtime | 6 | Nothing | A03 open |
| 21 | Plugin sandbox | 6 | K03, K04, K07 and A01 accepted (isolation, capabilities, resource limits, SDK) | A02, A03 open; CLAP [10], LV2 [11] |
| 21 | Signing & identity | 6 | B03 accepted (signed updates and rollback) | B01 open |

Sequencing consequence (judgement): four broad engines (text, converters,
compositing with a path to GPU, Materiaal history) together appear in 102 row mentions across all six disciplines and
the cross-cutting rows; the pixel pipeline is equally broad (22 rows). Colour
management has fewer rows but blocks every professional graphics and video row
and is cheap to design in early and expensive to retrofit.

### 4.2 Accepted native services the suite can build on

These criteria are `accepted` in `docs/progress.json`. They prove operating
system services, not application capabilities, and none of them lifts a
matrix row above *proposed*.

| Criterion | Accepted capability | Evidence | Use in the suite |
|---|---|---|---|
| K03 | Isolated preempted processes | `docs/verification/m2-m4/ACCEPTANCE.md` | One process per application, converter and plugin |
| K04 | Capabilities and bounded IPC | `docs/verification/m2-m4/ACCEPTANCE.md` | Per-document and per-plugin access |
| K05 | Process and service management | `docs/verification/K05-ACCEPTANCE.md` | Background jobs, render and export queues |
| K07 | Resource management under pressure | `docs/verification/K07-ACCEPTANCE.md` | Large documents, plugin limits |
| S02 | Journal and cold recovery | `docs/verification/m6b/README.md` | Crash-safe saving |
| S04, S05 | Scalable volumes; documents of useful size | `docs/verification/M6E-ACCEPTANCE.md` | Media and project storage |
| S06 | Collections, metadata and edit history | `docs/verification/S06-ACCEPTANCE.md` | Base of Materiaal history and libraries |
| S08 | Storage recovery after failures | `docs/verification/S08-ACCEPTANCE.md` | Project integrity |
| U01 | Native compositor and session | `docs/verification/mature-session-20261102/U01-ACCEPTANCE.md` | Window and surface base; GPU acceleration not demonstrated |
| A01 | Public application API and SDK | `docs/verification/A01-ACCEPTANCE.md` | Application and plugin development |
| B03 | Signed updates and rollback | `docs/verification/B03-ACCEPTANCE.md` | Authentic application, plugin and font delivery |
| N01 | Packet and transport stack | `docs/verification/mature-session-20261102/README.md` | Base for mail, calendar and sync protocols |

## 5. Specialised applications and shared workspaces (proposal)

This section is a proposal for discussion in deliverables S4 (object
architecture) and S10 (phased plan). It is not a decision and has not been tested with users.

**Criteria (judgement).** An activity deserves its own application when at
least one of these holds: it has hard real-time constraints (playback,
recording); its data model diverges (timeline, scene graph, pixel layers,
flowing text); its tool density would crowd a shared surface; or professionals
expect a dedicated environment with its own keyboard map. Activities share a
workspace when they use the same object kinds and editing model and users
switch between them often.

| Proposed surface | Kind | Rows | Main engines | Reason |
|---|---|---|---|---|
| Document workspace | Shared | WP, NK01–NK05, NK08–NK11, PD | Text & font, PDF & print, Materiaal history | Flowing text, notes, references and PDF review share one editing model |
| Data workspace | Shared | SS, DB | Calculation, Search & index | Grids, records, forms and trackers share one record model |
| Canvas workspace | Shared | PR, NK06–NK07 | 2D vector, Text & font | Free placement on slides, diagrams and boards |
| Communication hub | Shared | EM | Internet protocols, Search & index | Mail, calendar, contacts and tasks; tasks are the same Materiaal objects as tracker items in the data workspace |
| Photo and raster editor | Specialised, with development and catalogue modes | GR01–GR19, GR38 | Pixel pipeline, Colour management | Layer model and tool density |
| Vector illustration | Specialised | GR20–GR25 | 2D vector | Path-editing model |
| Page layout and prepress | Specialised | GR29–GR33 | Text & font, PDF & print, Colour management | Threaded frames, print production |
| Painting | Specialised workspace on the raster engine | GR26–GR28 | Pixel pipeline, Input & command | Low-latency stroke input |
| Video editor, including grading, audio for video and social delivery | Specialised | VI01–VI11, VI17–VI21 | Media framework, GPU, Low-latency audio | Real-time timeline playback |
| Motion and compositing | Specialised | VI12–VI16 | Node-graph, GPU | Node or layer compositing model |
| Audio workstation | Specialised | AU | Low-latency audio, Plugin sandbox | Hard real-time audio graph |
| Interface design tool | Specialised | UX | 2D vector, Asset library | Components, constraints and prototypes |
| 3D suite | Specialised | TD | 3D scene engine, Node-graph | Scene-graph model |

**Shared by every surface (proposal).** Materiaal objects and history; one
asset library for swatches, fonts, brand kits and components (GR34–GR37,
UX06–UX07); colour management; the text engine; the command and shortcut
system with configurable workspaces (XC03–XC05); autosave and recovery (XC01,
XC02); accessibility (XC06); learning and classroom functions (XC07, XC08);
isolated converters with fidelity reports. A file should move between surfaces as a
Materiaal object rather than through export and import (judgement; subject to
S4).

## 6. Open items

1. No user test and no ADR. Every priority is a proposal for deliverable S10.
2. U03 discrepancy: `docs/verification/materiaal-library-20261102/README.md`
   records U03 as accepted (score 41 to 44) on another branch, while
   `docs/progress.json` on this branch records U03 open. The matrix follows
   `docs/progress.json`; the discrepancy is not resolved here.
3. Five native runs under open criteria or no criterion are counted below
   *passed (native)*: the document scenario, the Materiaal keyboard run, the
   clipboard run (U02), the Materiaal library run (U03) and the image-decoder
   fixture. The calculator undo integration has no executed native gate on the
   merged tree.
4. WP01, WP17, NK01 and NK03 have passing host tests and a native run, but
   their criterion U03 is open; they are *prototyped (host)* on the host
   evidence only. The JSON sidecars of the Materiaal host logs record the
   command and return code 0, not the source commit, so the logs are not bound
   to a commit by themselves. U01 acceptance includes a saved and reopened
   14-byte document as evidence, but its criterion covers the compositor and
   session, not document editing.
5. No host prototype was opened in a mature office application; DOCX export
   was checked only by an independent reader. Round-trip fidelity is unknown.
6. The spreadsheet recalculation oracle does not record the Python micro
   version. Judgement: the decimal prototype cannot reproduce the documented
   binary floating-point behaviour [4]; SS04 needs research before any
   compatibility claim.
7. Format-level detail is deferred: file formats to S3, the native object
   architecture to S4, the feature-level interoperability matrix to S5.
8. Sources not opened and therefore not cited: the Adobe Photoshop shortcut
   page (HTTP 403), the PDF Association and ISO pages on PDF (HTTP 403), the
   ICC specification PDF (unreadable), the Blender snapping page (body not
   extracted) and the ACES page (HTTP 404). Claims about PDF/A, PDF/X, PDF/UA
   and ACES therefore remain unverified.
9. Not covered: engineering simulation, game-engine authoring, and scientific
   and statistical packages. The addendum's user groups are covered by the
   Beginner path column and by XC06 (accessibility users), XC07 and XC08
   (students and teachers) and VI21 (video creators), but the needs of specific
   accessibility groups (for example switch access, braille output or
   cognitive accessibility) are not yet broken down into rows.
10. GPU acceleration and audio output are not demonstrated natively (H03 open);
    every real-time row depends on them.
11. The host records were written in Dutch; their content was read in full and
    restated in English here.
12. Terminology was not reconciled with S0
    (`docs/design/suite/WORKSPACE-REQUIREMENTS.md`).
13. The evidence inventory and validation script are stored in
    `docs/verification/suite-s1-capability-matrix-20261114/`. Both predate
    this revision: the inventory does not list the host logs cited by WP17,
    NK01 and NK03, and the script still allows a proposed row with limited
    support when a native run is named, which no row now uses.
14. The column set of section 3 is defined in section 1 of this document; no
    separate column specification exists in the addendum or in S0.

## 7. Sources

All sources were opened on 14 November 2026. Microsoft and Adobe pages are
used as functional and workflow benchmarks only.

| No. | Source | Supports |
|---|---|---|
| 1 | https://support.microsoft.com/en-US/Word/differences-between-the-opendocument-text-odt-format-and-the-word-docx-format | Feature-level loss between word-processing formats: track changes not supported, tables partial |
| 2 | https://www.gov.uk/guidance/using-open-document-formats-odf-in-your-organisation | Government guidance (18 December 2018): functions may be lost converting spreadsheets; migration advice |
| 3 | https://ecma-international.org/publications-and-standards/standards/ecma-376/ | Office Open XML standard, fifth edition |
| 4 | https://learn.microsoft.com/en-us/troubleshoot/microsoft-365-apps/excel/floating-point-arithmetic-inaccurate-result | IEEE 754 double, 15-digit precision, documented deviations, near-zero compensation since Excel 97 |
| 5 | https://docs.oasis-open.org/office/OpenDocument/v1.3/os/part4-formula/OpenDocument-v1.3-os-part4-formula.html | OpenFormula 1.3: open formula syntax, semantics and functions |
| 6 | https://support.microsoft.com/en-us/word/customize-keyboard-shortcuts | Custom shortcuts for commands, macros, fonts, styles and symbols, stored in a document or template |
| 7 | https://help.libreoffice.org/latest/en-US/text/shared/optionen/01050100.html | Snap to grid with resolution, subdivision and a one-action override |
| 8 | https://support.microsoft.com/en-us/office/excel-specifications-and-limits-1672b34d-7043-467e-8e27-269d656771c3 | Worksheet, cell, formula, nesting and pivot limits |
| 9 | https://learn.microsoft.com/en-us/office/dev/add-ins/overview/office-add-ins | Sandboxed web add-ins, manifest permissions, central deployment; COM and VSTO Windows-only |
| 10 | https://cleveraudio.org/ | CLAP open audio plugin standard |
| 11 | https://lv2plug.in/ | LV2 open audio plugin standard |
| 12 | https://support.microsoft.com/en-us/topic/7dd3040c-3f30-4fdd-bab0-8586492a1f1d | Real-time co-authoring, presence, subscription requirement |
| 13 | https://www.color.org/icc_specs2.xalter | ICC.1:2022, profile version 4.4; ISO 15076-1 (2005); 2025 amendment |
| 14 | https://opencolorio.org/ | Colour management for motion pictures |
| 15 | https://www.itu.int/rec/R-REC-BT.2100 | BT.2100 image parameters for HDR television |
| 16 | https://learn.microsoft.com/certifications/exams/mo-211 | Excel Expert certification; about 150 hours of instruction and hands-on experience |
| 17 | https://certifiedprofessional.adobe.com/en/how-it-works | Certification delivery, hands-on tasks, about 150 hours of experience |
| 18 | https://learn.microsoft.com/en-us/graph/overview | One API endpoint across mail, calendar, contacts, files, notes, planning, tasks and Teams |
| 19 | https://www.rfc-editor.org/rfc/rfc9051.html | IMAP4rev2 |
| 20 | https://www.rfc-editor.org/rfc/rfc5545.html | iCalendar |
| 21 | https://www.rfc-editor.org/rfc/rfc4791.html | CalDAV |
| 22 | https://www.rfc-editor.org/rfc/rfc6350.html | vCard 4.0 |
| 23 | https://learn.microsoft.com/en-us/typography/opentype/spec/ | OpenType 1.9.1 |
| 24 | https://www.unicode.org/reports/tr9/ | Unicode Bidirectional Algorithm (UAX #9), Unicode 18.0.0 |
| 25 | https://www.w3.org/TR/WCAG22/ | WCAG 2.2, W3C Recommendation |
| 26 | https://github.com/AcademySoftwareFoundation/OpenTimelineIO | Editorial timeline interchange |
| 27 | https://openexr.com/en/latest/ | High-dynamic-range scene-linear and multi-part, multi-channel images; deep compositing named as a use case |
| 28 | https://openusd.org/release/intro.html | Layered scene description and interchange (USD 26.11) |
| 29 | https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html | glTF 2.0.1 runtime 3D asset delivery |
| 30 | https://materialx.org/ | Material and look-development interchange |
| 31 | https://midi.org/midi-2-0 | MIDI 2.0: higher resolution, per-note control, capability negotiation |
| 32 | https://tech.ebu.ch/publications/r128 | EBU R 128 loudness normalisation and true-peak level |
| 33 | https://www.designtokens.org/tr/drafts/format/ | Design Tokens Format Module, draft Community Group report, not a W3C Standard |
| 34 | https://www.w3.org/TR/SVG2/ | SVG 2, Candidate Recommendation Snapshot |
| 35 | https://www.loc.gov/preservation/digital/formats/fdd/fdd000628.shtml | DNG 1.6: a published raw format owned by a vendor |
| 36 | https://help.libreoffice.org/latest/en-US/text/shared/optionen/01010200.html | Load and save options: periodic recovery information for modified documents, optional automatic save of the document instead, backup copy on save |
