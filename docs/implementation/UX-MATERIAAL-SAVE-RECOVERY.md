# TASK-UX-SAVE-RECOVERY — Materiaal: saving and recovery

**Status:** design package with an executed partial visual review of a native
probe; no complete runtime or user acceptance. **Design baseline:** checkout
`ca45a7d` on `simonvanieperen-bookish-invention`. The drawn states are in
[`ui/design/save-recovery.svg`](../../ui/design/save-recovery.svg).

## Goal and scope

Complete one usable, verifiable native chain for everyday work: create/open
a document, edit text, explicitly save, return to Materiaal and reopen the
same object. The service determines whether something is saved; the interface
must not predict success or automatically repeat an uncertain mutation.

This package does not replace P2, add a route or change shared tokens,
runtime, native UI code or service ABI. P2 remains frozen at seven screens.
The existing token source remains `ui/prototype/tokens.css`; existing
Materiaal identity, original Dutch labels and system/status boundaries remain
authoritative. The screen sketch is a design view, not a screenshot or claim
of a working native compositor.

**Acceptance:** FR-U03 (actual progress, cancellation and errors), FR-U05
(accessible operation), FR-S04 (no silent loss of open work), FR-O01
(retrieving confirmed work after a cold restart), FR-O04 (I/O errors without
false ACK or replay), FR-U01 (focus and return) and rubric U03/U04/U06/R01.
This design specification closes none of those criteria.

## Main task and state transitions

The flow uses the actual `Controller`/`DocumentSession` operations from
`ui/native`. UI object IDs are not permissions; only already validated native
adapters may supply events or capabilities.

| User action | Actual service/controller transition | Visible state |
|---|---|---|
| Refresh Materiaal | `refresh()` requests the actual object list. Only a valid `DocumentSession::list` response sets `list_loaded=true`; on error, the list remains unloaded and `Failed` is set. | Distinguish a successfully empty list from a failed/not-yet-loaded list. Show no loading spinner for a synchronous operation. |
| New document | Actually refresh first if needed; then use the first free object slot through `DocumentSession::create`. The draft is not yet saved. | Editor opens with document focus; status “Changes not yet saved.” |
| Edit | Input changes only the session draft. Preserve UTF-8 bytes and selection; do not change the stored revision. | Visible caret/selection and unsaved status; no “saved” appearance. |
| Save | With a changed draft, `begin_save()` starts an actual stream. Each `advance_save()` performs at most one APPEND/COMMIT exchange. Only actual `SaveProgress::Saved { revision }` sets `Status::Saved`. | While writing: label “Cancel saving” and progress from acknowledged bytes (`completed/total`), not time. After actual commit: “Saved on this device” plus revision number in the status/history view when available. |
| Cancel during saving | The button requests actual `DocumentSession::cancel_save()`/ABORT. | Only after confirmed cancellation: “Saving cancelled. Your text remains open.” Draft remains dirty; no new revision is reported. A failed ABORT remains an error. |
| Back with a dirty draft | `request_back()` opens `ConfirmClose`, with default focus `KeepEditing`; there is no automatic save or discard. | “Not yet saved” with explicit actions: “Save”, “Continue editing” (default focus), “Discard changes”. Escape cancels the overlay and preserves the draft. |
| Save and back | The same save chain as above. Only after confirmed commit does the controller return to Materiaal and refresh the list. | Return only after actual `Saved`; an error leaves the document open. |
| Discard | Only after explicit selection does the controller call `discard_changes()` and return to the actual list. | State that only unsaved changes are discarded; no hidden default choice. |
| Back/reopen | After confirmed saving, refresh the list, open the same object ID and have the service read the current revision again. | Same object identity and saved content; the previous draft is not evidence, the actual `open()` response is. |

**Unchanged content:** after its availability/permission checks, `save()` can
return `SaveOutcome::Unchanged { revision }`. The controller also uses
`Status::Saved` for this. It is the already loaded, confirmed revision,
not a new COMMIT, revision increase or reconfirmation after `Uncertain`.
For such an action, the intended copy is “No new changes. Revision {n} on
this device.” The current `View` does not yet expose this distinction
separately; role 13 must retain the operational result before this different
copy is possible. An additional click must never be interpreted as new
durability evidence.

At `completed == total`, only the content bytes are acknowledged: the next
COMMIT can still fail. Then show “Content sent. Confirming storage…”;
no tick mark or success animation. For an empty document there is no meaningful
byte percentage; show “Confirming storage…” without a determinate byte bar.
A cancellation request is not itself a terminal outcome. If COMMIT is already
confirmed, the result remains saved; do not subsequently report “cancelled”.
A late or incorrectly bound response must not overwrite current status.

### Errors, permissions and recovery boundaries

- **Read failed:** do not present an empty list as though there were zero
  documents. Show the cause if the service actually provides it, let the
  user remain on the list and offer another read action only when
  operationally available.
- **Read-only or denied grant:** `can_save=false`; visibly disable the
  impossible action with a textual reason. No repeated grant prompt or
  expansion of object/network permissions.
- **Conflict:** preserve the draft and report that a newer revision exists.
  No silent overwrite. Offer compare/merge/reopen only when the service
  provides actual operations for them.
- **Full/oversized content/I/O error:** preserve the draft and show the
  concrete consequence and next actually supported action. Do not imply
  partial saving.
- **Uncertain COMMIT outcome (`Unknown`/`Uncertain`):** explicitly show
  “Storage outcome unknown. Your text remains on this screen.” Block save
  and automatic replay; retained status is not “saved”. The current session
  closes the service connection and retains the draft in the process, but
  does not prove recovery after app/process termination. No native status
  query/safe-resumption operation has yet been established. **The UI must
  therefore not show “Try again” here until a version-bound service operation
  can safely establish commit status.**
- **Input/compositor loss:** if the current session still owns the draft,
  retain it and let a fresh authenticated input binding restore input. This
  is not evidence of storage, device reset or screen-reader recovery.

The UI itself implements no retry as recovery. The `retry` asset is intended
only for an error state for which the service explicitly permits an
idempotent/safe retry; `Unknown`/`Uncertain` is excluded.

### Target copy, originally Dutch, and availability of recovery actions

The texts below are target copy, not all current renderer labels. The error
code remains an optional diagnostic detail, never the primary explanation.

| Service fact | Primary message / consequence | Actually permitted next step and boundary |
|---|---|---|
| Valid empty `list` | “No material yet. Create your first document.” | “New document”, only with existing write permissions. No empty-list copy after a list error. |
| `ReadOnly` | “Read-only. You cannot change this document here.” | Reading/navigation remain possible. No button that simulates a grant. |
| Explicit deny/revoke | “No permission to save this document. Your text remains open.” | Retain the draft while the process lives. New scoped permission requires a real broker; that recovery route has not yet been provided. A generic transport error is not relabelled as denial. |
| `Service(Full)` / `Service(Quota)` | “Storage is full.” / “Your storage allowance is used up.”, followed by “Not saved. Your text remains open.” | Offer only proven space management/an authorised alternative destination once connected natively; the current save does not restore capacity. |
| `ContentTooLarge` | “This text does not fit in the available document space.” | Visibly retain input; silently truncate nothing. Text can be shortened only if the live session is still editable. The trial cap does not become a permanent product limit. |
| `Conflict` / stale revision | “There is a newer version. Your changes have not been saved.” | No blind overwrite or reopening that erases the draft. Comparison/recovery depends on a connected revision portal. |
| Known I/O error | “Saving failed. Your changes have not been confirmed.” | Retain the draft. Safe retry only after explicit service classification and recovery; current generic error handling grants no unlimited retry permission. |
| `Closed` | “Storage connection lost. Your text remains open.” | A new authenticated session requires recovery and fresh grants. The UI cannot reopen old endpoints. |
| `Uncertain` | “Storage outcome unknown. Your text remains on this screen.” | No automatic replay, no “Try again”, no claim that closing the app preserves the draft. |
| Confirmed ABORT | “Saving cancelled. Your text remains open.” | Return to document focus; the draft remains unsaved and becomes editable again if the session permits. |
| Confirmed COMMIT | “Saved on this device. Revision {n}.” | Return to the list and actually reopen. Infer no NAS/backup/encryption claim from this local ACK. |

## Screen structure and visual rules

Use the values from `docs/implementation/PRODUCT-SPEC.md` §5 and the existing
CSS token source; introduce no divergent runtime or design-token list.

- **Workspace:** material content remains dominant. A calm light background,
  one white content surface, precise alignment and a limited inspector are
  permitted; no permanent Office ribbon, dock, decorative statistics or
  glass layer.
- **Editor header:** “Back to material” on the left; title as primary context;
  one primary action (“Save”) on the right, changing to “Cancel saving” during
  actual saving. Keep the button at least 44×44 CSS pixels and visible at 200%.
- **Content:** document title and text editor remain one object surface.
  Cursor, selection and keyboard focus are separately visible; the editor
  has a clear control border and a focus ring from the existing token.
- **Status:** persistent text label near the editor footer; progress bar only
  during an active save and proportional to actual byte ACKs. Retain space
  for two status lines at narrow widths; colour is never the only meaning.
- **Confirmation:** opaque local dialogue, no misleading system-privilege
  claim. Show the consequence before the destructive action. “Continue editing”
  receives initial focus; “Discard changes” is clearly destructive and requires
  deliberate activation.
- **Typography:** product targets body 16/24, metadata 14/20, section 20/28,
  page title 32/40, weight 400 or higher. The current native raster uses
  bounded bitmap monospace glyphs; that is not definitive product typography.
  Proportional shaping, complete Unicode/grapheme support and IME are explicit
  rendering/input dependencies, not reasons to reduce the ambition.
- **Motion:** the current native leaf does not animate. A future compositor
  may use only short functional transitions according to the existing
  120/180/220-ms values; with reduced motion, no movement and at most a
  50-ms fade. Status/save feedback must never depend on animation.
- **Icons:** use role 16's asset set only beside text or a semantic name.
  There is no confirmed native SVG raster route yet; claim no icons in the
  native build before that route exists.

### Exact layout for role 13

Dimensions below are logical pixels (physical pixels divided by scale factor).
The SVG is a reduced overview of states, **not** an 800/1280/1920 runtime
layout drawn to scale. Determine breakpoints from logical window width;
a 1920 screen at 200% has 960 logical pixels.

| Logical viewport | Layout rule | Long text and space allocation |
|---|---|---|
| 800×600 / 1280×720 / 1920×1080 at 100% | Outer margin 24; workspace at most 1120 wide, centred. The existing shell rail follows PRODUCT-SPEC: 224 from 1200, 72 from 768. Editor within available content width. Header controls at least 44 high; 24 between header and content; editor padding 16. | Introduce no new rail for this slice if the native shell is still absent. Show the same workspace and record that missing shell separately. Title receives at least two lines; document panel fills the remaining height. |
| 1280×720 at 200% (640×360) | One column, rail/inspector closed but accessible through a named action once the shell is connected; no reduced labels. Header: back on the left and save on the right with at least 8 spacing. | Outer margin 16, header 44, title at most two lines of 24; editor at least two text lines; status normally 48, for long error/progress 72 or more from measured text height. No fixed status height that clips text. |
| 800×600 at 200% (400×300); additional width 320 | Compact outer margin 12; header 44; wrapping title line; no side panel. Full-screen dialogue with vertical full-width actions at least 44 high and 8 spacing. | With a tall status/title, the document fits in its own scroll area; controls/status remain reachable. When not everything fits at once, use explicit vertical screen scrolling, never hiding, overlap or horizontal page scrolling. |
| 1920×1080 at 200% (960×540) | Medium-width shell: 72 rail if present; inspector available on request, not permanent. Other dimensions do not scale again: they are already logical. | Same status/title measurement rules. Do not replace text scaling with larger rectangles alone or a doubled bitmap without geometry adjustment. |

Focus receives 6 px of clearance on all sides (3 stroke + 3 offset). A
visible selection uses the existing `--accent-soft` background and `--text`
text; after focus loss the selection remains recognisable, while the active
focus ring moves. A disabled control loses neither label nor reason. Hover
changes only existing-token colours or borders and causes no layout shift.
A primary button retains its accent fill and action hierarchy; the secondary
surface may change to accent-soft. Normal text contrast threshold is 4.5:1;
large text and meaningful UI borders/states at least 3:1. Assess rendered
colour pairs, not only fixed values in this package.

## Keyboard, focus, scaling and semantics

- Tab/Shift+Tab through all actions; in the editor: Back → Document → Save
  (during saving: Cancel). Enter activates a focused control; within document
  text, Enter inserts a line break. Ctrl+S requests actual saving. Escape
  with dirty work opens the explicit choice, not discard.
- The focus ring is 3 px with a 3-px offset, fully visible and not based only
  on colour. Pointer hover does not replace focus. After closing the
  non-destructive dialogue, focus returns to the action that opened it.
- At 200%, both text and geometry scale; no hidden primary action, overlap
  or horizontal page scrolling. Long UTF-8 titles and statuses wrap without
  drawing over document text or controls.
- Current `Frame::semantics` provides app-local roles, names, focus,
  read-only/enabled, text value, selection and actual save progress. This
  is not yet accessibility-tree/event IPC or a working screen reader.
  Screen-reader labels, status announcements and modal focus trapping must
  be actually connected in the compositor/accessibility broker before FR-U05.
- Keyboard operation is tested in the same flow as mouse operation; reduced
  motion is a system preference, not an app-specific security setting.

## Verifiable acceptance for the next native build

1. **Actual operation:** on an exact source-bound build, new/open → edit text
   → save → actual APPEND progress → confirmed commit → back → refresh list
   → reopen the same object. Compare object ID, revision and content with
   independent storage inspection.
2. **Negative cases:** ABORT before COMMIT; dirty close and Escape; deny/
   read-only; full storage; conflict; I/O error; unknown COMMIT. No case shows
   confirmed storage without acknowledgement, no unknown outcome is replayed,
   and the editor loses no draft during the live session.
3. **Fixed raster captures:** actually rendered editor/dirty/saving/saved/
   conflict/uncertain states at 800×600, 1280×720 and 1920×1080; 100%/200%
   and 320 CSS pixels logical width. Record source, image and frame hashes.
4. **Layout/assertions:** no clipping/overlap; visible focus; targets at least
   44×44; complete status label; long Dutch text and UTF-8 text remain readable;
   empty list is distinguishable from list error.
5. **Operation/accessibility:** the same assertions through PS/2/QMP
   keyboard-only and pointer operation; check semantic roles/names/status/
   progress; separately record actual screen-reader/accessibility-broker trial.
6. **Evidence boundary:** host raster/host harness is design review; target
   build is not native execution; native VM QMP rendering and operation are
   evidence only for their exact source/image; human task trial remains
   separate.

### Targeted state checks for 13/20

These are preregistered expected outcomes, **not** executed tests. Each case
records input, starting revision, actual service outcome, visible status and
focus. Storage cases additionally receive an independent byte/revision check;
snapshots or UI copy are insufficient.

| Case | Stimulus | Required outcome |
|---|---|---|
| UX-SR01 | Actually empty list versus list error/no valid response | Only a successfully empty list shows emptiness; an error has a named cause. No fictional objects. |
| UX-SR02 | New → UTF-8 text → Ctrl+S | Focus first on Document, during save on CancelSave, after confirmed COMMIT on Document; draft/content/revision exactly preserved and actual progress visible. |
| UX-SR03 | Last APPEND acknowledged, COMMIT not yet acknowledged; also an empty document | No Saved/tick mark; byte bar is not commit success. An empty document has no invented percentage. |
| UX-SR04 | ABORT during stream; failed ABORT; cancel after completed COMMIT | Only confirmed ABORT produces Cancelled and a dirty draft. Error remains error; confirmed COMMIT is not displayed as cancellation. |
| UX-SR05 | Dirty close; focus successively on KeepEditing, SaveAndBack and Discard, plus Tab/Shift+Tab/Escape | Each focus target visible/semantically correct, background inoperable. Escape preserves draft/caret/selection. Loss only after explicit Discard. |
| UX-SR06 | ReadOnly, fresh deny/revoke, Closed and Conflict during open/save/confirm-close | No working Save claim where the service refuses. Default focus remains safe; draft remains visible. Do not leave focus on an action that has become unreachable. |
| UX-SR07 | Full/Quota/ContentTooLarge and known I/O error | No truncation or false ACK; cause, consequence and supported next step correct. Do not present a generic error as proof of Full/deny. |
| UX-SR08 | Lost COMMIT reply / Uncertain | No Saved, automatic replay or retry button. The unknown outcome remains recognisable after input rebind or editing back to the original bytes. |
| UX-SR09 | Unchanged on a valid session; then the same attempt on Closed/Uncertain | Claim no new revision/COMMIT; a closed/uncertain session produces no new success. |
| UX-SR10 | Late response, old generation/epoch or stale action after navigation | No status/focus overwrite or new mutation from stale input. The native adapter must enforce binding; the UI label does not. |
| UX-SR11 | Long title (including accents), long error label and save progress at all three physical sizes, 100/200%, 320 logical width and keyboard-only | No label outside button/status, overlap or lost primary action; 44×44 targets and 3+3 focus clearance. Inspect save progress and long error separately, and the transition from progress to error. |
| UX-SR12 | Save → list → open same object → stop → cold new VM → open same object | ID/revision/UTF-8 bytes match confirmed storage; fresh Ready status need not be identical to old Saved status. No old draft as cold evidence. |

## Actually observed baseline and deviations

`ui/native/verification/checks.json` reports host/scalar checks with
`native_execution=false`, `screenreader=false` and `physical_hardware=false`.
The stored `host-raster-review.json` describes actually inspected host
rasters at 800×600 and 640×480: the minimum-size list shows an actual object,
editor/dirty-close states were rasterised at 100%/200%, and an opaque local
backdrop prevents partially visible modal text. That source-bound host
evidence was not rebuilt in this checkout.

Role 13 supplied the rendering/state specification on review baseline
`e382a989` and reported separate host-renderer fix `0818f5e` and
generation-bound semantics consumer `89e2004`. This does not prove any of
those separate commits on one guest image. Specification review finds the
existing save/ABORT/Uncertain distinction appropriate; the `Unchanged` nuance,
long status/progress at 200% and read-only/closed/conflict/deny at all three
confirm-close focus targets remain concrete integration points to check.
The new host PPMs were unavailable in this checkout and were not inspected
here.

The existing operation test `real_ps2_edit_stream_commit_list_and_reopen`
uses the host harness to exercise actual PS/2 bytes, draft, actual save
progress, commit, list and reopen; other host cases test ABORT, unknown
commit, conflict, revocation, I/O, read-only and focus. These tests do not
prove QEMU presentation or input for the combined 13/20 build.

Concrete open deviations/dependencies:

1. The renderer is a scalar 32×8 private-RAM tile renderer; compositor,
   display presentation and actual QMP rendering/operation must be bound
   to the exact 13/20 release candidate.
2. Semantics are app-local; screen-reader IPC and actual announcement/focus
   integration are absent.
3. The current monospaced bitmap font and code-point operations are a bounded
   first implementation, not final typography or Unicode quality.
4. There is no version-bound safe status query for `Unknown` yet; the UI
   must not “recover” this state through retry or saved success.
5. Design assets are not runtime assets while native vector/raster
   consumption is unavailable.
6. The current test source has no rasters or native combined build reproduced
   in this checkout. The release integrator supplies the exact candidate
   for follow-up review; missing evidence is not inferred from older
   screenshots.

**Human usability trial:** not yet performed. Prepare a task for a user
without administrative knowledge: “Create a short note, save it, go back and
reopen the same note; then cancel an edit and explain what is and is not
saved.” Record task completion, assistance, understanding of storage status,
error expectations and recovery success; invent no participant or result.

### Executable protocol for the human task trial

This is a prepared protocol, not recruitment, participant registration or
an executed trial. Role 20/21 uses only a recorded synthetic disposable
image. Participants do not work with personal documents, accounts, NAS or
keys. Participant consent and an appropriate local location for notes are
arranged before the trial; no automatic recording, telemetry or publication.
It is explicitly explained beforehand that this is a development version
and that the assessment concerns the system, not the participant.

**Starting conditions:** enter exact source commit, kernel/app/disk hashes,
profile, runner and input method. The checker confirms that the task and
fault injection actually work, without showing the participant the result
in advance. Do not ask a person to assess whether unimplemented controls
“work”. If the native task is unavailable, a labelled design study may be
performed, but it receives no native usability acceptance.

| Step / neutral instruction | Observe without supplying the answer | Independent check |
|---|---|---|
| “Create a short note about an appointment and make sure you can find it later.” | Does the participant find create, write and save? Which visible status gives them confidence? | Record actual commit, object ID, revision and bytes; no task success based only on confidence or a click. |
| “Go back to your material and continue with the same note.” | Is the same object found without an unintended copy? Does the relationship between object and tool remain clear? | Actual list/open, same ID and confirmed bytes; no previous draft as evidence. |
| “Add something and go back. You decide not to save that addition.” | Understanding of Save / Continue editing / Discard changes, default focus and Escape; count unintended loss or assistance. | Only explicit discard removes the addition; the earlier confirmed revision remains intact. |
| “Save this longer note, but stop saving while it is in progress.” | Does the participant find and understand cancellation? Do they expect text loss, undo or only stopping? | Actual ABORT and retention of dirty draft; if commit completed first, record that outcome accurately and do not classify it as a cancellation error. |
| Controlled uncertain outcome: “What do you think happened, and what would you do now?” | Understanding of unknown versus failed/saved; record dangerous expectations about retry, closing or backup. Promise no recovery that does not exist. | Fault/COMMIT binding and no-replay confirmed by the checker; open recovery dependency remains visible. |
| “Continue with the same note after restarting.” | Can the participant retrieve confirmed work? Do they distinguish the previous unsaved change from the last saved version? | Cold new VM, independent disk oracle and opened object bytes; execute only when that native route is available. |

Repeat relevant tasks keyboard-only and at 200% with participants suited to
those interaction modes; a sighted mouse user listening to a screen reader
does not replace testing with actual assistive-technology users. Do not
force a second round if fatigue or the development build distorts results.
The trial facilitator provides no navigation hint during the primary attempt;
requested assistance is given only after recording the point of assistance
and original outcome.

For each task the tester records: started/completed/blocked/aborted,
start/end time, number and nature of hints, unintended actions/copies,
expected versus actual saved version, focus/readability problem, the user's
explanation in their own words and recovery outcome. Report time and
subjective assessment separately; do not use averages to hide loss,
misunderstood Unknown, missing accessibility or an unexecutable task. Stop
the task on unexpected data loss, unsafe retry/permission expansion or an
unrecoverable focus trap; retain the defect and repeat only after a fix on
a newly bound candidate.

Sample size and task-time/assistance budget are fixed before the actual
trial by the research track according to PRX/DV-01, not selected afterwards
from favourable outcomes. This package claims no representativeness,
satisfaction, adoption or conformity. Native technical assertions, AI design
review and human observations each retain their own results column.

## Asset handoff and local checks

Role 16 supplied only `ui/assets/material-flow-icons.svg`, definitive
source `d3547639be733aefcc1228edad37114980e5073c`. Four symbols:
`document`, `history`, `saved`, `retry`; 24×24 viewBox, monochrome currentColor,
1.75 stroke and rounded ends/corners. The file is a sprite, not a directly
visible icon sheet. Role 15 inspected the contact sheet actually rendered
with system librsvg/Cairo at 24/48 px on light/dark backgrounds: no visible
clipping; document/clock/tick/repeat outlines are distinguishable. This is
AI host visualisation checking, not a human recognition trial or native
consumption. The sprite is not integrated by this package; intake/role 8
retains separate source and ownership.

Local execution on 30 October 2026:

| Command / check | Result and boundary |
|---|---|
| `python3 -B ui/native/verify.py` | Exit 1: `FileNotFoundError: cargo`. No host tests, Clippy, build or new native raster images executed. Existing evidence files not rewritten. |
| SVG parser check with `xml.etree.ElementTree` | Exit 0: namespace/viewBox correct. First shell variant had a Python quote-syntax error (exit 1); corrected parser check passed. |
| `python3 -B /home/simon/.copilot/session-state/038b6cdc1bb74465872d0b93e6a6e0cd/files/render-design.py ui/design/save-recovery.svg /home/simon/.copilot/session-state/038b6cdc1bb74465872d0b93e6a6e0cd/files/save-recovery.png` | System librsvg/Cairo, without installation; actual design rendered and inspected. No native runtime or operation trial. |
| `git diff --check` | Exit 0; only assigned design-package files added in the worktree. |
| `python3 -B tools/progress.py --check` | Exit 0; existing rubric 34/100 consistent. Design/asset review closes no U03/U04/U06/R01 criterion: proposal **34% → 34%**. |

The first visual design check found the button “Opslaan annuleren” (English:
“Cancel saving”) too narrow. That button was widened; the final sketch was
rerendered and inspected without the observed label overflow. This validates
the illustration, not the native responsive layout. No runtime token was
changed or reserved. Central PROJECT-STATE, WORK-QUEUE and score assessment
remain with role 1 according to the manager agreement.

Source binding for this visual check:

- Design SVG SHA-256:
  `57a69211e476a51ccd924d492a3047a9e5931d2a5b93fe2bb74e94b0430b7858`.
- Design PNG (1440×1110) SHA-256:
  `d513ce5cb053f662400b79423ead043802d3c8784ce206bed9a8e5ac09fde24f`.
- Role-16 asset SHA-256:
  `391f75bcfdf5f7cd37532b66531a52646a650e3bc3d7c7b2a811f2d60b2dea26`;
  contact sheet:
  `f92b214bfcbeb2dded9c135232fe7c7518acc8886a0f969f092dd9ce3a839d1b`.

PNGs remain session artifacts outside Git. The SVG retains source form and
an explicit simulation/design label; no generated binaries in this commit.

**Historical resumption step before receiving the probe:** role 4/12 supplies
one fixed candidate with source commit, image/app hashes, UI/token source
binding, profile, existing runner and corresponding 13/20 claim. Execute
UX-SR02/03/05/06/08/11/12 on it and assess the actual frames **and** input
transitions. Until then, runtime/user acceptance is open; the new local
host-raster run remains blocked by missing Cargo.

## Partial native review — actually operated probe 0818f5e

**30 October 2026, role 15.** The previously missing probe described above
has now been received and reviewed read-only. This replaces only that old
availability blocker. It is neither a new VM run by role 15 nor a human
audit. Design source and examined runtime source remain different.

| Binding | Exact value / boundary |
|---|---|
| Native source | `0818f5ec651b72c32b8795ff0d3d9a6ebdc9783d`, v3 `26b41e5586434ce51202a4e234ccaa99bad25c5a76ec3efdb3af9d7020c14f5f` |
| Probe | `native-system-materiaal-probe`, normal; `pc-q35-10.0/qemu64/TCG/1CPU/256MiB/VGA/PS2`, 800×600, scale 100%. This is neither VM-FR nor the ordinary dev1 preview. |
| Run executed by 13/12 | 132.4606 s; raw QEMU exit 33, runner exit 0. Duration of the whole trial is not a measurement of interactive response time. |
| Kernel / loader | `48acb20ca8e12e0418f21c3790361c2a3ac8c935cc76de0ad6ddb977bfd10896` / `9686ab3b85eaeedc334053cc9f583f6459c99e2779673416eacac319c68aa387` |
| Materiaal app | `d84f5333ad11d91b44ea822382562d78a508ec12794518cc70a0a47b583c06db` |
| Token source | `ui/prototype/tokens.css`: `0c588509e0cb0f94bbda2c6969da8d79d463359bfc9684687a684210a54aadcf`, unchanged |
| Read-only bundle | `native-materiaal-probe-0818f5e-ad23a2ee91.tar.gz`, 2,682,413 bytes, SHA-256 `3c476387f7aa7657207e2a0ee96b7d1b159695de1716305cdbefa795b26447b2` |
| Location | `/home/simon/.copilot/session-state/1b87793863ad4dcc8f16a0bd10884654/files/ui-image-0818/`; adjacent `handoff.json` contains the run command and source, tool, userspace, firmware, input and frame binding |

### What was actually inspected and checked

All six displayed PNGs (`ready`, `editor`, `dirty`, `saved`, `list`,
`reopened`) were opened and visually assessed. Their hashes were checked
against the handoff; for each image, decoded RGB pixels were compared exactly
with the original PPM in the bundle. All 40 manifest files, 15 frozen artifacts
and five UI/token source files were hash-checked; UI sources were additionally
compared with Git objects on `0818f5e`. This check certifies binding and pixel
equality, not all behavioural claims.

The 31 archived QMP requests and their successful reply IDs were inspected:
pointer to Nieuw (English: New) and button press/release, 14 actual ASCII
keys (`nieuw os werkt`, English: “Nieuw OS works”), Ctrl+S, Escape, Tab, Tab,
Enter and six screendumps. No guest-memory writes in that trace. F12 and
the test controller provide only trial termination: no ordinary product
closure, shutdown or cold-reopen evidence.

The handed-off snapshots show 14 draft bytes and zero save ACKs for `dirty`,
one save ACK, revision 1 and a clean draft for `saved`, and after list/open
the same object 0 with the same 14 bytes. The disk-oracle outcome in the
handoff is object 0 / revision 1 / 14 bytes / canaries intact. Independent
runtime/disk replay was supplied as passing by 13/12; role 15 did not rerun
the storage oracle here and claims no second VM trial.

### Concrete visual findings for role 13

| ID / priority | Observation on exactly these frames | Implementable follow-up and acceptance |
|---|---|---|
| UX-V01 — improvement before visual product acceptance | On `ready`, Nieuw (English: New) is a deep green primary button; on `list`, that same action loses its accent fill when the pointer is over it and resembles the secondary Vernieuwen (English: Refresh) button. The focus border does remain visible. | Retain the primary action hierarchy on hover, for example the same accent fill with an existing-token border/focus treatment. No new token without an ownership claim. Compare native ready/list with the pointer on and beside the same control; text contrast and focus must continue to comply. This does not block the normal save chain. |
| UX-V02 — open product quality | The title “Nieuw document” (English: “New document”), document text, buttons and status use a uniformly small bitmap monospace presentation. The opened object's title receives little more emphasis than metadata; the large content surface emphasises the difference from the small labels. No clipping is visible with this short ASCII content. | Retain the current functional bitmap backend as a labelled interim step; plan the specified body/metadata/title hierarchy with actual text metrics and proportional shaping. Check selected nominal sizes and rendered glyphs on target profiles. This observation proves neither that a particular font size is wrong nor that text is readable for all users. |
| UX-V03 — open object continuity | `saved` says only “Op dit apparaat opgeslagen” (English: “Saved on this device”); `list` shows “Nieuw document” / “Op dit apparaat” (English: “New document” / “On this device”), and `reopened` says “Document geopend” (English: “Document opened”). The new revision is correct in the snapshots but not visible in this UI. | Make the confirmed revision together with object identity queryable/visible in status or history context; `Unchanged` remains distinct from a new commit. Infer no new version from text editing alone. Verify final labels against the actual document response. |
| UX-V04 — positive partial observation | Dirty and Saved are textually distinct; the focus ring remains fully visible within the assigned surface. Document content `nieuw os werkt` appears in dirty, saved and reopened, with caret/focus on the document. System chrome remains outside the app area. | Preserve these status/focus/surface boundaries in follow-up work. Do not extend the finding to grant-prompt authenticity, screen reader, selection contrast or crash recovery: those were not exercised in this trial. |

Meaningful content has no visible overlap with controls/footer in these six
800×600 images. This does not approve long text, all error states or other
viewports. Role 15 made no change to runtime code, compositor, glyphs,
tokens or shared interface.

UX-V01 is also source-bound: `render.rs` on `0818f5e`, lines 1064–1077,
gives Primary the accent fill only when `!hovered`; hover falls back to the
same style as Secondary. The desired adjustment is specified more precisely
in the hover rule above; the current deviation is not presented as faultless
execution of the earlier overly general surface/accent-soft rule.

On the same fixture, role 16 reports the still-missing 24-px icon slots at
list/editor title and status. That is a future integration dependency, not
a defect in the supplied symbols: reserve a combined icon/text area with
8-px spacing, measure the full 24-px height and do not place it on the
existing text baseline across the editor border. Scale both together at
200%. Only the current confirmed document status may show `saved`; an old
counter `SAVE_ACKS=1` after reopening is no basis for it. Do not approve
history/retry/200% asset presentation from this run.

### Acceptance by actually available case

| UX-SR | Assessment on this single normal probe |
|---|---|
| SR01 | Successfully empty list visible. List error not executed; case not fully approved. |
| SR02 | New through pointer, ASCII input, Ctrl+S and confirmed Saved/reopen visible and trace-bound. Intermediate save progress/CancelSave focus not captured; therefore partial evidence only. |
| SR03 / SR04 | No intermediate APPEND frame, empty-document commit or ABORT/cancellation race. Not executed. |
| SR05 / SR06 / SR07 / SR08 / SR09 / SR10 | No dirty-close modal, deny/read-only/closed/conflict/full/I/O/Unknown/Unchanged or stale-input case in this fixture. Not executed; a positive normal run does not replace negative tests. |
| SR11 | Only 800×600 at 100%, short ASCII content. No 200%, other sizes, long error/progress or keyboard-only operation from initial startup. Not approved. |
| SR12 | Same object reopened within the same VM. No cold new VM; cold part not executed. |

The native source demonstrably has actual frames **and** an interaction trace
for this narrow normal path. All combined FR-U03/U05/S04/O01/O04 and rubric
U03/U04/U06/R01 remain open; proposal **34% → 34%**. Role 16's assets were
not consumed in this runtime. Screen-reader/human/physical acceptance remains
unexecuted.

### Command, result and next resumption step

From the dedicated repository root:

```sh
python3 -B /home/simon/.copilot/session-state/038b6cdc1bb74465872d0b93e6a6e0cd/files/verify-native-fixture.py /home/simon/.copilot/session-state/1b87793863ad4dcc8f16a0bd10884654/files/ui-image-0818
```

Exit 0: bundle hash/size, manifest members, six lossless pixel round trips,
15 artifact hashes, five Git UI sources and 31 successful bounded QMP
requests checked. No guest started, bundle code executed or evidence file
changed. The verification script is a session artifact; existing repository
helpers `capture_graphics.decode_png` and `run_graphics.ppm` supply pixel
checking. Complete JSON output is in
`/home/simon/.copilot/session-state/038b6cdc1bb74465872d0b93e6a6e0cd/files/native-fixture-verification.json`.
`git diff --check` and `python3 -B tools/progress.py --check` also returned
exit 0. The lead agent/manager retains ownership of central assessment files;
role 15 supplies only this 34% → 34% proposal with evidence.

**Next bounded flow:** dirty close with all three focus targets and Escape
(SR05), followed by actual error/Unknown presentation (SR06/08). Role 13 owns
implementation; 4/12/20 allocate the new source-bound cases and runner slot.
Then assess actually rendered modal and error frames plus the corresponding
interaction assertions; no additional VM by role 15 without that coordination.
UX-V01–03 are concrete product deviations for that follow-up, not an
instruction to change the shared token source without an ownership claim.

## UX-V01 — targeted source review of the hover fix

**30 October 2026, Europe/Amsterdam.** Reviewed leaf:
`0818f5ec651b72c32b8795ff0d3d9a6ebdc9783d` →
`29b79eae19de616be415cfbd7b7cc313766546d6`
(`origin/codex/native-primary-hover`). Only the renderer delta, associated
pixel regressions and `UX-V01-primary-hover.md` were reviewed; no general
re-review or new test/VM run.

**Outcome: no blocking design deviation in this delta.** The primary button
retains `ACCENT` as fill and `SURFACE` as label colour on hover; only the
border changes to the existing `TEXT` colour. That meets the refined hover
rule above without new tokens or geometry changes. The disabled branch
remains before style selection; Secondary and Danger are unchanged. Existing
focus code remains after button painting.

The added tests sample actual tile pixels at 100/200% for Nieuw (New),
Opslaan (Save) and the primary modal save action: fill, upper border, focus
border and unchanged rendered label pixels. The pointer is outside those
samples. A separate case checks disabled Nieuw and secondary-hover Vernieuwen
(Refresh). This is targeted regression evidence, not a complete surface
comparison or native modal/accessibility acceptance.

Executed review checks: `git diff --name-only 0818f5e 29b79eae` shows only
the three assigned leaf files; the complete renderer/test diff and evidence
note were read. A read-only Python-standard-library check compared all twelve
SHA-256 entries from that note: three source/token files against Git objects
and nine logs against the supplied log directory. Exit 0. `red.log` shows
the expected old failure at Material/100%; `regressions.log` shows 4/4
passing. The other gates were executed and described by role 13, not rerun
by role 15. No bundle code was executed.

**UX-V01 is resolved at source/host-regression level; native reassessment
remains open.** The six 0818 images are not relabelled as 29b79eae.
UX-V02/03, SR05/06/08, native 200%, integration CI and human acceptance
remain outside this review. Score proposal remains **34% → 34%**; central
assessment stays with the manager. Integration and appropriate combined
gates remain with the assigned integrator; role 15 changes no runtime files.
