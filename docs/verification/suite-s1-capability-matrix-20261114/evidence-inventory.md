# S1 evidence inventory — what the repository proves for suite capabilities

> Note added at assembly: this scratch inventory was gathered at source `d17f75f6`; the
> published matrix cites `ec8b1d08`, which differs only in `docs/PROJECT-STATE.md`. Row
> identifiers here use the pre-rename scheme; the published matrix maps W to WP, X to SS,
> P to PR, K to NK, E to EM, D to DB, F to PD, G to GR, V to VI, A to AU, U to UX and T to TD
> for rows only. Criterion identifiers are unchanged.


Scratch record for deliverable S1 (capability matrix). Compiled 14 November 2026
on branch `claude/baryon-os-ui-ux-design-152adb`, worktree head `d17f75f6`.
Read in full: all nine `docs/verification/platform-*-20261103.md` records,
`docs/progress.json`, `docs/PROGRESS.md`, `docs/implementation/NATIVE-DOCUMENT-SCENARIO.md`,
`docs/implementation/NATIVE-MATERIAAL.md` (lines 200–394), the S06, U01 and A01
acceptance records, and the heads of the clipboard, Materiaal-library,
calculator-undo, image-integration and native-image records.

## Evidence-level rules applied

- **passed (native)**: only when the matrix row's *own* capability matches the
  acceptance sentence of a criterion that is `accepted` in `docs/progress.json`.
  A passed dependency (for example S06 revision history) does not lift an
  application row. Accepted criteria are OS services and appear in matrix
  section 4, not as discipline rows.
- **prototyped (host)**: a host-process prototype or library test that exercises
  exactly the feature named in the row. The record proves that exact feature only.
- **proposed**: everything else, including native QEMU runs whose criterion is
  still open (noted in the Evidence cell as "native QEMU run, criterion open"),
  unless the same record also stores a host test of that feature.

## A. Host prototypes (`sdk/examples/*`, host-cli) — all nine records

All nine share these limits: host CLI over stdin/stdout only; no native
executable, window, GUI, input, save portal or authorised volume save; stdout
writes are not atomic; no Microsoft Office / LibreOffice / Adobe application
open-or-render test; no QEMU, hardware, human user test, independent audit or
release acceptance; each proposes score 41 -> 41 (no criterion closed). The
no_std library additionally builds for `x86_64-unknown-none` (compilation only,
not execution).

| # | Record | Exact feature proven | Level | Explicit limits / not demonstrated / lossy |
|---|---|---|---|---|
| A1 | `docs/verification/platform-csv-table-20261103.md` (`sdk/examples/table-editor`) | UTF-8 CSV (RFC 4180 profile: comma, quote, CRLF/LF) import into a bounded rectangular table, transactional editing of selected cells, canonical CSV export, re-import; checked against Python `csv.reader` (81 oracles after review, incl. leading U+FEFF fix). | prototyped (host) | Max 4 MiB input, 4096 rows, 256 columns, 64 KiB/cell. No type inference, no formula evaluation (formula-like text kept literally and **not** safe against formula injection in external spreadsheets). No undo, typed cells, XLSX, other CSV dialects or Office round-trip parity. |
| A2 | `docs/verification/platform-docx-export-20261103.md` (`sdk/examples/document-export`) | UTF-8 plain text -> DOCX: ZIP/OPC package with exactly three fixed parts, WordprocessingML paragraphs/runs/text; Unicode, spaces, tabs and empty paragraphs read back exactly by an independent Python ZIP/XML reader (133 exports). Input markup is escaped; no metadata, external relationships or macros. | prototyped (host) | No formatting, styles, fonts, pagination, tables, images. No full OOXML schema validation; **not opened or rendered in Word or LibreOffice**. Independent of the import slice: no round trip proven. ECMA landing page returned HTTP 403 (not read). |
| A3 | `docs/verification/platform-docx-import-20261103.md` (`sdk/examples/document-import`) | DOCX (Transitional `word/document.xml`) -> UTF-8 text with mandatory `--plain-text` flag making formatting loss explicit; strict ZIP32/OPC admission, CRC per part, namespace-aware parsing (roxmltree 0.21.2); 83 text-exact imports, 1432 rejections incl. DEFLATE bomb, traversal names, spoofed main type. | prototyped (host) | Fields, tables, images, hyperlinks, numbering, tracked changes, unknown content and header/footer references are **rejected**, not imported. Linked styles not interpreted. No ZIP64/encryption. Upstream XML/DEFLATE allocations infallible. No Office/LibreOffice interop, no export round trip. |
| A4 | `docs/verification/platform-image-crop-20261103.md` (`sdk/examples/image-crop`) | PNG/JPEG decode (existing codec) -> display-oriented RGBA8 -> rectangular crop -> PNG encode (host-only `png` 0.18.1); byte-exact crop including alpha over all eight EXIF orientations (288 crops, 325 outputs, 111 denials). | prototyped (host) | Mandatory `--drop-metadata --unmanaged-colour`: **ICC, gamma and metadata dropped**; no colour management. **PNG16 rejected**, CMYK rejected. No resampling. PNG encoder not ported to the OS. Max 4096 px/dimension, 1 Mpixel. Host stdio is not a sandbox. |
| A5 | `docs/verification/platform-image-export-20261103.md` (`sdk/examples/image-export`) | PNG/JPEG -> uncompressed RGB24 BMP with explicit background for alpha flattening; display-oriented pixels; checked by independent Python reader (30 outputs; PNG exact, JPEG within tolerance 3). | prototyped (host) | **RGBA16 quantised to 8 bit (lossy)**; alpha flattened; no EXIF/ICC/metadata in output; no ICC/gamma/linear-light transform; CMYK rejected. Not a photo-fidelity claim. |
| A6 | `docs/verification/platform-m7-exif-viewer-20261103.md` (`sdk/examples/image-viewer`, `services/image-codec`) | Host: viewer fit/zoom/pan and tile rendering in display axes for all eight EXIF orientations, RGBA8/RGBA16 samples (19 viewer + 24 codec tests). Native: separate QEMU decoder probe (8 PNG, 3 JPEG, 10 negative cases, cancellation) passed at `30c4effe`. | prototyped (host) for the viewer; decoder VM run is native but tied to no accepted criterion | The **VM tested the decoder chain, not the viewer**. No native viewer window, object portal, input, cold save, OOM/revoke; ICC/gamma colour transforms and PNG eXIf out of scope. FR-A03/A04 open. |
| A7 | `docs/verification/platform-presentation-preview-20261103.md` (`sdk/examples/presentation`) | Import of a small project-specific UTF-8 LF text deck; next/previous/select; rasterised slide frame (title, body, page counter) at 640x360 or 1280x720 via existing `ui/graphics` atlas/Canvas and Materiaal palette; P6 PPM output; overflow is an error, not clipping. | prototyped (host) | **Own text profile only — no PPTX, ODP or Markdown**. No editor, animation, transitions, media, templates. No full Unicode font coverage/shaping. No BT.709/colour fidelity claim. No accessibility/high-DPI/performance UX. M11/FR-A15 open. |
| A8 | `docs/verification/platform-spreadsheet-recalc-20261103.md` (`sdk/examples/spreadsheet`) | Cell model (max 64 cells A1..Z64) with formulas using A1 references, dependency recalculation after edit (diamond DAG, 30-cell DAG), cycle/missing-ref/divide-by-zero/range errors explicit; arithmetic by existing `nieuw-os-calculator::decimal` (9 fraction digits, ties-to-even), checked against Python `decimal`. | prototyped (host) | **No functions, ranges, absolute references, macros**; ASCII only, 256 bytes/cell; no text/typed cells, no CSV/XLSX/ODS, no undo, no performance gate. **No Office/IEEE calculation-parity claim.** Python micro-version of the oracle run unknown. M10/FR-A14 open. |
| A9 | `docs/verification/platform-wav-clip-20261103.md` (`sdk/examples/audio-clip`) | RIFF/WAVE PCM16 mono/stereo, 8–192 kHz -> explicit frame range -> new WAV with identical sample bytes, channel order and rate; independent check with Python `wave` (48 outputs). | prototyped (host) | **No playback / audible output, no audio device**. No PCM8/24/32, float, extensible, RF64, compressed codecs. Unknown chunks only with explicit `--drop-metadata` (cue/loop positions dropped). No fades, gain or resampling; hard cut may click. FR-A06 open. |

## B. Native QEMU runs relevant to suite features whose criterion is OPEN

| # | Record | Exact feature demonstrated natively | Level for the matrix | Limits |
|---|---|---|---|---|
| B1 | `docs/implementation/NATIVE-DOCUMENT-SCENARIO.md` (29 Oct 2026) | SDK v7 `DocumentSession`: create draft, replace-range 4096 fixture bytes, token-bound BEGIN/APPEND/ABORT, SAVE/COMMIT to revision 1, OPEN reread; cold read-only recovery in a new VM; independent disk oracle. | proposed (native QEMU run, criterion open; storage part underpins S05/S06) | Fixture bytes, not a user editor; plaintext on disk (no B02); no GUI. |
| B2 | `docs/implementation/NATIVE-MATERIAAL.md` keyboard QMP baseline | Materiaal native app: ready, editor, dirty, saved, list, reopened phases via 21 QMP keys; disk oracle 14 bytes revision 1. | proposed (native QEMU run, criterion open) | No cold restart, screen reader, speech output; new semantic tree not linked into the tested image. Closes no U criterion. |
| B3 | `docs/verification/native-clipboard-20261102/README.md` | Ctrl+C/Ctrl+V of text between two isolated native apps (Materiaal and Calculator), one gesture per request, Ctrl+Shift+Delete clears; empty after compositor restart. 22 phases, raw exit 33. | proposed (native QEMU run; listed as evidence under U02, which is open) | Interaction budgets, capacity and account authentication open. Plain text only. |
| B4 | `docs/verification/materiaal-library-20261102/README.md` | Materiaal library: create, type, rename, save, local name search, collections, view earlier version and restore it as a new revision; cold reopen byte-identical; 64 KiB/document, two history versions, eight objects per domain. | proposed (native QEMU run; **U03 is open in progress.json**) | README states "U03 accepted, 41 -> 44" on draft PR 130 branch, but `docs/progress.json` on this branch records U03 open at 41/100. Discrepancy recorded as an open item; not resolved here. No shaping, IME, screen reader, human tests. |
| B5 | `docs/verification/full-release-images-844ec40/README.md`, `NATIVE-IMAGES.md` | Native PNG/JPEG decoder fixture in QEMU (8 PNG + 3 JPEG, 149,647 pixels, 10 negative cases, cancellation before decode). | proposed (native QEMU run, no criterion) | Decode fixture only; no viewer, full format coverage, mid-decode cancellation or native OOM recovery. |
| B6 | `docs/verification/veld-calculator-undo-20261103/README.md` | Calculator undo/redo (Ctrl+Z, Ctrl+Y/Ctrl+Shift+Z) and recall; integrated via PR #209. | proposed | Native gate on the merged tree is recorded as **unknown (not executed)**. Calculator is not a spreadsheet. |

## C. Accepted native criteria (`docs/progress.json`, 41/100) relevant as shared infrastructure

None of these accepts an application-level suite capability. They are cited in
matrix section 4 as passed (native) OS services.

| Criterion | Acceptance (abridged) | Evidence | Relevance to the suite |
|---|---|---|---|
| K03 Isolated preempted processes | Ring-3 processes with separate address spaces; app fault isolation. | `docs/verification/m2-m4/ACCEPTANCE.md` | Parser/plugin/codec process isolation. |
| K04 Capabilities and bounded IPC | Rights reduction, revocation, bounded Message320 IPC. | `docs/verification/m2-m4/ACCEPTANCE.md` | Plugin sandbox, file/object grants. |
| K07 Resource management under pressure | Memory/CPU/IPC/handle exhaustion bounded per app. | `docs/verification/K07-ACCEPTANCE.md` | Large documents, render workers. |
| S02 Journal and cold recovery | Bounded object journal survives interruption. | `docs/verification/m6b/README.md` | Crash consistency of saves. |
| S04 Space reuse and scalable volumes | Compaction, quotas beyond 32 records. | `docs/verification/M6E-ACCEPTANCE.md` | Large projects, media libraries. |
| S05 Documents of useful size | Large documents and streams beyond 128-byte probe. | `docs/verification/M6E-ACCEPTANCE.md` | Large datasets, media streams (bounded profile). |
| S06 Collections, metadata, edit history | Native collections, persistent metadata, recoverable revisions, transactions. | `docs/verification/S06-ACCEPTANCE.md` | Materiaal history, version restore, multi-object transactions. Two history versions in the tested profile. |
| S08 General storage recovery | Responsive during disk failure; explicit recovery choices. | `docs/verification/S08-ACCEPTANCE.md` | Actionable recovery in apps. |
| U01 Native compositor and session | Interactive graphical session, window/surface isolation, compositor recovery; Materiaal and Calculator as separate CPL3 processes; a 14-byte document saved, listed, reopened. | `docs/verification/mature-session-20261102/U01-ACCEPTANCE.md` | GPU/CPU compositing base for every app. The document save is part of the session evidence, not an accepted editor capability (U03 open). |
| A01 Application API and SDK | Reproducible SDK, documented ABI, sandbox behaviour. | `docs/verification/A01-ACCEPTANCE.md` | Foundation for first-party and third-party apps and plugins. |
| B03 Signed updates and rollback | Authentic updates, downgrade protection. | `docs/verification/B03-ACCEPTANCE.md` | Basis for authentic plugin/app/font/model distribution (A02 open). |
| N01 Native packet and transport stack | Real packets, bounded TCP/UDP. | `docs/verification/mature-session-20261102/README.md` | Collaboration and mail transport base (N02–N04 open). |
| L02 Local quantum execution | Not relevant to the suite. | — | — |

Open criteria that gate suite rows: U02 interaction, U03 Material open-to-save,
U04 accessibility, U05 settings/search/recovery, A02 install/update, A03 app
permissions and tool integration, A04 useful everyday applications, H03 graphics
output and audio, N02–N04 internet, B02 authenticated storage, P01–P04 NAS and
sync, S09 backup/export.

## D. Consequences for the matrix

1. No discipline row reaches "passed (native)"; expected `native_passed_rows` = [].
2. Host-prototyped rows (exact scope only): CSV table import/edit/export;
   plain-text DOCX export; plain-text DOCX import; four-operator decimal formula
   recalculation on 64 cells; image crop (8-bit, unmanaged colour); BMP export
   (lossy 16->8); image viewing with EXIF orientation, fit/zoom/pan; text-deck
   slide raster preview; PCM16 WAV clip.
3. "Support today" for those rows is at most "with limitations" or "lossy";
   no row is "full".
4. Every other row is "proposed" with support "unsupported" or "proprietary research".
