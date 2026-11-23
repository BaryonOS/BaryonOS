# U06 usability protocol for agreed core tasks — proposal

**Date:** 23 November 2026. **Source baseline:** `main` at
`47a30a02162e0714181001c3553ba86e1e60d3e6`. **Status: PROPOSAL.** This
document drafts a preregistrable protocol for rubric criterion **U06 — Polish
validated with users**. It closes no criterion, executes no study and changes no
score: the progress register remains **41/100** and U06 remains **open**. The
human gate `U06-T1` ("Preregistered core-task user study") in
[the roadmap ledger](../roadmap-pipeline/ledger.json) stays **blocked** until the
project owner approves the study, its budgets and its privacy handling.

U06 acceptance text ([progress register](../progress.json),
[PROGRESS.md](../PROGRESS.md)):

> Representative users complete agreed core tasks; friction, errors, and visual
> inconsistencies are measured and resolved within product budgets.

This protocol reuses, and does not replace:

- the per-task measures in [DESIGN-AMBITION.md](../DESIGN-AMBITION.md)
  (completion, number of decisions/prompts, error/recovery behaviour and
  comprehension, with users who are not administrators; visual polish,
  responsiveness, keyboard operation and accessibility in the same trial);
- the D1 trial tasks in [DESKTOP-SHELL.md](DESKTOP-SHELL.md) §5 (start an app,
  find a file again, arrange a window, shut down without explanation);
- the user-task register UC01–UC09 in
  [FULL-TEST-RELEASE-CONTEXT.md](../FULL-TEST-RELEASE-CONTEXT.md) and the rows
  FR-U03, FR-U04, FR-U05 and FR-R01 in
  [FULL-TEST-RELEASE.md](../FULL-TEST-RELEASE.md), including evidence level
  **A** (bounded human checks; AI review is no substitute);
- the accessibility verification list in
  [accessibility-gaps.md](../audit/os-completeness/accessibility-gaps.md),
  item 4 (participant group, tasks, success/assist/errors and privacy);
- the privacy rules of [LOCAL-PRIVACY-CONTRACT.md](../LOCAL-PRIVACY-CONTRACT.md).

## 1. Scope and testability rule

A core task is **testable** only when a participant can perform it, without
developer tooling, in one of the three interactive bundles accepted by
[`tools/package_windows_qemu.py`](../../tools/package_windows_qemu.py):
`native-window-session`, `native-system-materiaal` or
`native-system-session-preview`. A directory under `sdk/examples/`, a host
renderer in `ui/native`, the browser prototype in `ui/prototype` or a
design drawing is not a participant-reachable native surface. Tasks without
such a surface are listed as **NOT YET TESTABLE** and stay in the protocol so
that the task set remains traceable to the full product scope; they are run
in a later round once their native surface exists.

A task marked **CONDITIONAL** has native source and tests on `main`, but its
presence in an interactive bundle must be confirmed against the running
candidate bundle before the preregistration is frozen: by source inspection at
the bundle's recorded `source_commit`, with path and line per route, and by a
pilot run (§10). The bundle `manifest.json` written by
`tools/package_windows_qemu.py` records hashes, kernel and loader features and
the source version, but no shell routes, so it cannot confirm a route. If a
route is absent, the task becomes NOT YET TESTABLE for that round.

**One frozen bundle per round.** Every counted task in a round runs on the same
frozen bundle, recorded by its `manifest.json` in the preregistration (§10).
Round 1 uses the `native-window-session` bundle, which carries CT01–CT06,
CT10 and CT11 and the conditional tasks if present. The
`native-system-session-preview` surface for CT06 is an alternative only: it is
used in a separately frozen, separately scheduled session with its own
manifest and is never pooled with Round 1 results.

**Audit basis.** The classification below was re-audited against the
`native-window-session` bundle at `main` commit
`47a30a02162e0714181001c3553ba86e1e60d3e6`. In that bundle the compositor
process `sdk/examples/window-session` owns a trusted shell (`src/shell.rs`)
with the pages `Lock`, `Users`, `Auth`, `Unlocking`, `Desktop`, `Search`,
`Settings`, `Notifications`, `Session` and `Capability` (`shell.rs:16-27`), a
dock with "Zoeken", "Materiaal", "Rekenmachine" and "Systeem"
(`shell.rs:626-666`), app activation through `ShellCommand::Activate`
(`main.rs:430-432`), shutdown through `ShellCommand::Shutdown`, which sends
`CONTROL [9]` (`main.rs:433-436`, also F10 at `main.rs:467-476`), F6 window
cycling (`main.rs:452-455`, `scene.rs:1295-1297`) and an Alt+Tab switcher
(`scene.rs:628-697`, wired at `main.rs:344-356`). The bundle starts on the
`Desktop` page: `Scene::new` calls `Shell::new(false)` (`scene.rs:183`), and
the start on `Page::Lock` (`main.rs:233-234`) is compiled only with the
`session-preview` cargo feature. `tools/build.py:205` sets `session_preview`
only for kernel feature `native-session-route`, `tools/build_materiaal.py:52`
forwards it and `sdk/examples/window-session/build.py:46` turns it into that
cargo feature; `tools/package_windows_qemu.py:119`
refuses that profile. Source presence is not runtime evidence; every task is
rechecked in the pilot (§10).

## 2. Core task list

### 2.1 Testable on the current interactive bundles

| ID | Task given to the participant | Native surface (source) | Trace |
|---|---|---|---|
| CT01 | Open Materiaal, create a new document, type a short given sentence and save it | `native-window-session` bundle; [`sdk/examples/materiaal`](../../sdk/examples/materiaal), [`sdk/examples/window-session`](../../sdk/examples/window-session); contract [NATIVE-WINDOW-SESSION.md](NATIVE-WINDOW-SESSION.md); accepted U01 run [U01-ACCEPTANCE.md](../verification/mature-session-20261102/U01-ACCEPTANCE.md) | FR-A02, FR-U03, UC01 (partial) |
| CT02 | Close the document, find it again in the list and reopen it; confirm the text is intact | Same bundle; save/list/reopen chain recorded in U01 acceptance | FR-A01, FR-A02, D1 "find a file again", UC07 (partial) |
| CT03 | Use the Calculator for a given sum while Materiaal stays open, then return to the document | [`sdk/examples/calculator`](../../sdk/examples/calculator) in `native-window-session` | FR-U01, U02 (switching/focus) |
| CT04 | Move the two windows so both are fully visible side by side, by dragging or with the window menu "Ordenen" | `native-window-session` (two draggable windows, U01 acceptance); window menu "Ordenen" → `Scene::arrange_positions` ([`scene.rs:452-511`](../../sdk/examples/window-session/src/scene.rs)); [`sdk/examples/compositor`](../../sdk/examples/compositor) | D1 "arrange a window", FR-U01 |
| CT05 | Complete CT01–CT02 with the keyboard only (no pointer) | Keyboard focus route via [`sdk/examples/input-service`](../../sdk/examples/input-service) and the compositor; [NATIVE-INPUT-DAEMON.md](NATIVE-INPUT-DAEMON.md) | FR-U05 (keyboard part), U04, UC09 (partial) |
| CT06 | Lock the workspace, try to sign in as the local user by choosing any of the offered sign-in methods, read the result, then return to the workspace through the guest test session | `native-window-session`: lock via Super+L, Sessie → "Scherm afschermen" or search result "Scherm afschermen" (`shell.rs:298`, `shell.rs:353`, `shell.rs:441`); "Aanmelden" → "Lokale gebruiker · nog niet gekoppeld" → refusal notice; "VM-gast · openbare testsessie" → "Open testsessie" (`shell.rs:240-275`, `shell.rs:372-413`). Alternative surface: `native-system-session-preview` with runner [`tools/run_session_preview.py`](../../tools/run_session_preview.py); [SESSION-AND-DESKTOP.md](SESSION-AND-DESKTOP.md) | FR-U03 (comprehension of a refusal); **no** FR-S02 claim: no real authentication exists |
| CT10 | Start Calculator and then Materiaal, once from the dock and once from search | `native-window-session` dock buttons "Materiaal" and "Rekenmachine" (`shell.rs:539-550`, drawn at `shell.rs:626-666`); search page "Zoeken en starten" via dock "Zoeken", F3 or a Super key tap (`shell.rs:276-283`, `shell.rs:324-370`, `shell.rs:417-424`, `shell.rs:445`); both end in `ShellCommand::Activate` (`main.rs:430-432`) | D1 "start an app" (dock/search part), FR-U01 |
| CT11 | Switch between the two open windows with the keyboard | `native-window-session` Alt+Tab switcher "WISSEL VAN VENSTER" (`scene.rs:628-697`, `scene.rs:1851`); F6 cycles focus directly (`main.rs:452-455`) | FR-U01, U02 (switching/focus) |

CT06 measures only whether the refusal and the guest route are understood.
It is explicitly not a sign-in test: the local-user route presents only
"Wachtwoord · dienst niet beschikbaar", "Vingerafdruk · sensor niet
beschikbaar" and "FIDO2-sleutel · geen apparaatdienst", and no credential is
collected (`shell.rs:263-272`, `shell.rs:395-399`, `shell.rs:515`).
The CT06 start state is the unlocked workspace; the lock screen is entered by
the participant, not shown at boot (§1 audit basis).

### 2.2 Conditional

| ID | Task | Native source on `main` | Condition |
|---|---|---|---|
| CT07 | Rename the document, place it in a collection, find it with the library search field | [NATIVE-MATERIAL-LIBRARY.md](NATIVE-MATERIAL-LIBRARY.md) (U03 implementation; native acceptance open) | Library route confirmed in the running candidate bundle (§1) |
| CT08 | Restore an earlier revision of the document | Same; [S06-COLLECTIONS-HISTORY.md](S06-COLLECTIONS-HISTORY.md) | Same |
| CT09 | Copy a selected sentence and paste it into another place | [NATIVE-CLIPBOARD.md](NATIVE-CLIPBOARD.md) (U02 contract; not closed) | Clipboard route confirmed in the running candidate bundle (§1) |
| CT12 | With all work saved, shut the system down from the session menu | `native-window-session` Sessie page "Afsluiten" (`shell.rs:301`, `shell.rs:406`) → `CONTROL [9]` (`main.rs:433-436`); "Opnieuw starten · nog niet beschikbaar" (`shell.rs:305`) | The pilot confirms that the VM ends in an observable powered-off state on the reference host; otherwise NOT YET TESTABLE for that round |

### 2.3 NOT YET TESTABLE

| ID | Task | Missing native surface (source) | Trace |
|---|---|---|---|
| NT01 | Complete first use and create a local profile | No OOBE code; [PROFILES-AND-USERS.md](PROFILES-AND-USERS.md) is a plan | FR-S01 |
| NT02 | Sign in with a personal password and unlock a personal session | No real authentication or key boundary; the native shell refuses local-user sign-in (`shell.rs:263-272`). Locking and returning through the guest test session is covered by CT06 | FR-S02, B02 |
| NT03 | Change a setting and find it unchanged after restart; use a full Settings app | Native "Snelle instellingen" page exists (`shell.rs:284-295`) but offers no persisted, user-changeable setting ("Beweging verminderen · aan" is fixed on; "Verbinding" and "Opslag" are notices), and restart is unavailable; no Settings app (DESKTOP-SHELL step D5) | FR-S03, U05 |
| NT04 | Start an app from the full Zenit/Orbit launcher with an app catalogue | Native dock and search start the two bundled apps (CT10); a full launcher with an app catalogue is not in an interactive bundle (DESKTOP-SHELL §7, step D3) | D1 "start an app" (launcher part), FR-U01 |
| NT05 | Find a file by name or content with indexed system search (Signaal) | Native search matches fixed keywords to five apps and actions only (`shell.rs:324-370`); no native file/content index (step D4) | FR-O07, U05 |
| NT06 | Browse known folders in a Files app | No native Files app (step D5) | FR-A01 |
| NT07 | Shut down or restart with unsaved work open and choose save, discard or cancel | "Afsluiten" sends `CONTROL [9]` without a save/discard/cancel dialog (`main.rs:433-436`); restart is unavailable (`shell.rs:305`). Shutdown with saved work is CT12 | D1 "shut down", FR-S04 |
| NT08 | Answer a permission request and later revoke it | Native permission phase blocked (ledger U03 task, window-session manifest and `ui/native` ownership) | A03, FR-U02 |
| NT09 | Understand and recover from a save failure | Fault/probe profiles are refused by the Windows launcher ([tools/windows-qemu/README.md](../../tools/windows-qemu/README.md)); design only in [UX-MATERIAAL-SAVE-RECOVERY.md](UX-MATERIAAL-SAVE-RECOVERY.md) | FR-U03, UC08 |
| NT10 | Use the system with a screen reader | Blocked `U04-T3` (TTS/braille dependency decision, audio) | FR-U05, U04, UC09 |
| NT11 | Snap a window to half the screen | No snap gesture or shortcut in the native scene; "Ordenen" places both windows at fixed positions (`scene.rs:487-499`) and is used in CT04. Alt+Tab and F6 switching are CT11 | FR-U01 |
| NT12 | Pair a personal NAS and resume work | No interactive native pairing flow | DESIGN-AMBITION task table, FR-P02 |
| NT13 | Import a photo, view it and find it again | Image components (`sdk/examples/image-viewer`, [NATIVE-IMAGES.md](NATIVE-IMAGES.md)) are not in an interactive bundle | UC04, FR-A03 |
| NT14 | Work with a spreadsheet or presentation | `sdk/examples/spreadsheet`, `sdk/examples/presentation` not in an interactive bundle | FR-A14, FR-A15 |

The current testable set therefore covers a narrow slice: documents, two
apps, window switching and arrangement, the lock/guest route and shutdown. A
U06 pass on that slice alone would not satisfy "agreed core tasks" unless the
project owner explicitly agrees that slice as the core set (see §12).

### 2.4 Reclassification against `main` `47a30a02`

An earlier draft of this protocol classified tasks from the 2 November U01
state, before the native trusted shell was present. Changes:

| Task | Earlier classification | Current classification and reason |
|---|---|---|
| Start an app | NT04 NOT YET TESTABLE (launcher/dock host-rendered only) | CT10 testable from dock and search; NT04 narrowed to a full Zenit/Orbit launcher |
| Switch windows with Alt+Tab | Part of NT11 NOT YET TESTABLE | CT11 testable (Alt+Tab switcher, F6 cycling); NT11 narrowed to snap to half the screen |
| Lock, refusal, guest return | CT06 in `native-system-session-preview` only; lock/unlock part of NT02 | CT06 testable in `native-window-session` from the unlocked workspace; session preview kept as alternative surface; NT02 narrowed to personal sign-in |
| Change a setting | NT03 NOT YET TESTABLE (no Settings app) | NT03 narrowed to persistence after restart and a full Settings app; the "Snelle instellingen" page offers no changeable setting to test |
| System search | NT05 NOT YET TESTABLE (no search) | App/action search is used in CT10; NT05 narrowed to indexed file/content search |
| Shut down | NT07 NOT YET TESTABLE | CT12 CONDITIONAL for shutdown via "Afsluiten" with saved work; NT07 narrowed to unsaved-work handling and restart |
| Arrange windows | CT04 by dragging | CT04 by dragging or "Ordenen"; non-overlap at 1280×720 is pending pilot confirmation (§4) |
| Other tasks | CT01–CT03, CT05, CT07–CT09, NT01, NT06, NT08–NT10, NT12–NT14 | Unchanged; the `Capability` page has no opener in the shell (`shell.rs:310-319`), so NT08 stays NOT YET TESTABLE |

## 3. Measures per task

All measures are taken by a human observer and from host-side capture; see §6.

| Measure | Definition |
|---|---|
| Completion | `unaided` (success without assist), `assisted` (success after any assist), `failed` (wrong end state, abandoned, or time limit reached). Success criteria are the end states listed in §4 |
| Time on task | Seconds from the observer's "start" to the participant's declared completion; reported as median and range. Valid only together with the declared host, QEMU version and accelerator (TCG or WHPX) |
| Errors | Count of actions that move away from the goal and require correction (wrong control, wrong window focus, lost text, wrong document). A **critical error** is any loss or overwrite of participant-entered content, or a saved state the participant believes differs from the actual state |
| Assists | Level 1: neutral prompt ("what would you try next?"); level 2: hint naming an area; level 3: observer demonstrates. Level 3 ends the task as `assisted` |
| Decisions/prompts | Number of dialogs, confirmations and choices the OS presented (DESIGN-AMBITION measure) |
| Comprehension | One scripted question after the task (for example "Where is your document stored now and is it saved?"); scored correct/partial/incorrect against a key frozen in the preregistration |
| Post-task ease | Single Ease Question (SEQ), 7-point scale, after each task (Sauro & Dumas, 2009) |
| Post-session usability | System Usability Scale (SUS), 10 items, after all tasks (Brooke, 1996); interpretation per Bangor, Kortum & Miller (2008) and Lewis (2018) |

## 4. Per-task success criteria

| ID | Success end state (observer-verified) |
|---|---|
| CT01 | A document with the given sentence exists and the UI shows a saved state; the exact Materiaal status text is recorded in the pilot and frozen as the key |
| CT02 | The same document is reopened from the list; text identical to CT01 |
| CT03 | Correct result shown in "Rekenmachine"; focus returned to the document with text intact |
| CT04 | Both windows fully visible and non-overlapping at 1280×720. Whether "Ordenen" (`scene.rs:487-499`) alone yields non-overlap at that resolution is not established by source reading and is confirmed in the pilot; if it does not, CT04 requires dragging |
| CT05 | CT01–CT02 end states reached without pointer input |
| CT06 | Participant reaches the lock screen, reads the local-user refusal ("Niet beschikbaar. Er wordt geen sessie ontsloten." and "Nog geen accountdienst. Voer geen wachtwoord in."), states correctly that personal sign-in is not available, and returns to the workspace through "Open testsessie" |
| CT07 | New name shown; document listed in the target collection; found via search |
| CT08 | Earlier revision content shown as current, participant can state which version is active |
| CT09 | Pasted text identical to the selection at the intended location |
| CT10 | Start state, before each of the two runs (dock, then search): both app windows minimized, so neither "Rekenmachine" nor "Materiaal" is visible; the observer sets and records this state. End state: each window opened by the participant during the run is visible and focused after a dock click, respectively after choosing the result on "Zoeken en starten". The two app windows exist for the whole session and activation unhides and focuses an existing window (`Scene::select`, `scene.rs:1280-1292`), so without this start state focusing an already open window (for example Materiaal after CT03) would satisfy the task |
| CT11 | Focus moved to the requested window with Alt+Tab ("WISSEL VAN VENSTER") or F6; the observer records which route was used |
| CT12 | After "Afsluiten" the VM reaches the powered-off end state confirmed in the pilot. Content survival is a separate check, not part of this observer-verified end state: after power-off, the CT01 document is checked by a host-side disk oracle where the runner provides one, or reopened under observer control after a reboot of the same disk image, outside the timed task. If neither check is available, content survival is recorded as not verified and no no-loss result is claimed |

The UI strings above are Dutch because the shell and app strings on `main` are Dutch;
they are success keys, not translations to show participants (see §12
question 6). For CT01, CT02 and CT07–CT09 the observer additionally records
whether the end state persisted after the session (host-side disk oracle where
a runner provides one); CT12 uses the separate content-survival check above.
Observer judgement alone is evidence level A/V, not D.

## 5. Friction and visual-inconsistency log

Each finding is one record in plain JSON lines. The observer's working copy,
kept only on the local disk (§8), may link a finding to participant pseudonyms.
The committed copy carries no participant field: it records only how many
participants were affected (`frequency`) and paraphrased observations:

```json
{"finding": "F-003", "round": 1, "task": "CT02", "kind": "friction", "severity": 2, "frequency": 3, "observed": "List entry not recognised as the saved document", "screen_ref": "frame-hash-or-screendump-id", "fr": ["FR-A01", "FR-U04"], "status": "open", "resolution": null, "retest_round": null}
```

- `kind`: `friction` (hesitation, wrong path, confusion), `error` (as §3),
  `visual` (inconsistent spacing, typography, icon, colour, state, truncation,
  focus indicator, motion; FR-U04 categories), `accessibility` (FR-U05 categories).
- `severity` (proposal): 1 blocks the task or causes a critical error;
  2 causes an error or level-2/3 assist; 3 causes delay or doubt;
  4 cosmetic.
- `frequency`: number of participants affected in that round.
- `screen_ref`: a frame hash or a host screendump identifier; never a camera
  image or a participant name.
- Visual findings may also be raised by an expert inspection of the same frozen
  image; those are tagged `"source": "expert"` and do not count as user evidence.

## 6. Proposed budgets (PROPOSALS; owner decision required)

None of the values below is agreed. They are starting proposals for the owner
decision that the preregistration must record.

| Budget | Proposed value |
|---|---|
| Unaided completion per testable task | ≥ 80 % of participants |
| Critical errors | 0 in the final round; any critical error is a severity-1 finding |
| Median SEQ per task | ≥ 5.5 of 7 |
| Mean SUS | ≥ 68, the approximate overall average reported in the SUS literature (Lewis, 2018); a higher target can be set by the owner |
| Comprehension | ≥ 80 % correct per task |
| Open findings at acceptance | 0 of severity 1–2; severity 3 only with an owner-accepted rationale |
| Time on task | Median ≤ 2 × the median of a pilot expert on the same host and accelerator; absolute seconds are not proposed because TCG/WHPX timing is not product hardware timing |
| Decisions/prompts | No repeated confirmation of the same valid grant (DESIGN-AMBITION) |

"Resolved within product budgets" (U06 text) is proposed to mean: every
severity-1/2 finding is fixed in source, and a re-test round on a new frozen
image with new participants meets all budgets above.

## 7. Participants, recruitment and consent

- **Representative users:** adults who use Windows, macOS or Linux at least
  weekly and are not system administrators or BaryonOS contributors
  (DESIGN-AMBITION, ADR-058 familiarity goal). The number per round is an owner
  question (§12).
- **Accessibility participants:** a separate subgroup is shared with
  `U04-T4` ("Accessibility user task tests", blocked) and UC09. The protocol
  records only the assistive configuration used (for example keyboard only,
  200 % scale when available, high contrast), never a diagnosis or medical
  information. Tasks that depend on a screen reader stay NOT YET TESTABLE
  (NT10).
- **Recruitment:** direct invitation only; no public advertisement,
  tracking link or third-party survey platform.
- **Consent:** written informed consent before the session, covering purpose,
  tasks, what is noted, whether host-screen recording is made (separate opt-in,
  default off), storage location, retention, withdrawal at any time without
  reason, and deletion on request. The consent text and any legal basis are
  owner questions; this document does not assert legal compliance.
- **Compensation**, if any, is an owner decision.

## 8. Privacy handling

Consistent with [LOCAL-PRIVACY-CONTRACT.md](../LOCAL-PRIVACY-CONTRACT.md) §1 and §4.1
(no OS telemetry, no vendor cloud, local data under the user's control):

- **No instrumentation of the OS image.** The study image is an unmodified
  interactive bundle. No logging, analytics or event reporting is added for the
  study; adding it would be telemetry.
- **Capture is host-side only:** observer notes, optional QMP screendumps and an
  optional host-window recording with separate consent. No camera, microphone or
  face capture.
- **Local-only records.** Raw notes, consent forms, the pseudonym key and any
  recordings stay on an owner-controlled local disk outside the repository, are
  never uploaded to a cloud service, and are deleted at the retention date agreed
  by the owner.
- **Pseudonyms** (`P01`, `P02`, …) are used in local records only. Because the
  owner keeps the pseudonym key, pseudonymised records are personal data and are
  never committed.
- **No personal data in the repository.** Only aggregated results and findings
  in the committed §5 format (no participant field, paraphrased observations)
  are committed. Free-text quotes are paraphrased and
  checked for identifying content before commit.
- Participants type only study-provided text; no personal documents, accounts
  or passwords are entered into the VM.

## 9. Environment

- **Image:** one frozen interactive bundle built by
  [`tools/package_windows_qemu.py`](../../tools/package_windows_qemu.py); its
  `manifest.json` (kernel, loader, firmware and process-image hashes plus source
  version) is recorded in the preregistration. No rebuild during a round.
- **Windows hosts:** [tools/windows-qemu/README.md](../../tools/windows-qemu/README.md)
  with `Start-NieuwOS.ps1`; acceleration via `Acceleration.ps1` /
  `Test-Acceleration.ps1` (WHPX where available, otherwise TCG). The used
  accelerator is recorded per session.
- **Linux hosts:** the runner for the same profile, for example
  [`tools/run_window_session.py`](../../tools/run_window_session.py), with
  `NOS_QEMU_MACHINE` and OVMF paths recorded.
- **Recorded per session:** host CPU model, RAM, display resolution and scaling,
  QEMU version, OVMF version, accelerator, input devices. One host configuration
  per round is preferred, so time measures stay comparable.
- The target functional profile VM-FR (1280×720, 200 % scale checks) in
  FULL-TEST-RELEASE.md is not yet available as one integrated image; current
  bundles are research profiles.
- **Evidence limit of the source bundle.** The accepted U01 run
  ([U01-ACCEPTANCE.md](../verification/mature-session-20261102/U01-ACCEPTANCE.md))
  was built with kernel feature `native-window-session-probe`, an automatic
  probe profile that the Windows launcher refuses; it is not a run of the
  interactive `native-window-session` bundle with a human at the controls. Its
  Windows/WHPX row is a locally reported result that was not re-read from raw
  files. Interactive-bundle runtime evidence on Windows/WHPX, and every shell
  route cited in §1 and §2, therefore remain open until the pilot (§10) records
  them on the frozen bundle.

## 10. Procedure and evidence layout

1. **Owner approval** of tasks, budgets, consent and privacy handling (§12).
2. **Pre-freeze checks** on the candidate bundle, recorded in the
   preregistration:
   - re-audit the shell routes cited in §1 and §2 at the bundle's source
     version and update citations if `main` has moved past `47a30a02`;
   - confirm the start page: on `47a30a02` the `native-window-session` bundle
     opens on the workspace, which matches the statement in
     [tools/windows-qemu/README.md](../../tools/windows-qemu/README.md) that
     "the workspace opens directly". If a later bundle starts on the lock
     screen, that README line is stale and is reported for correction in a
     separate change, and the CT06 start state is revised;
   - confirm each CT07–CT09 and CT12 route individually by source inspection
     at the bundle's `source_commit`, recording source path and line per route;
     the bundle manifest lists no routes (§1). The pilot (step 3) then confirms
     each route at runtime.
3. **Pilot** with one or two people (not counted) on the candidate bundle, to
   check scripts and timing, and to confirm the CT01 status text, the CT04
   non-overlap end state with "Ordenen", the CT12 powered-off end state and
   the CT07–CT09 and CT12 routes.
4. **Preregistration freeze**, after the pilot: copy of this protocol with
   the agreed task set, success keys, budgets, analysis plan and bundle
   `manifest.json`; its SHA-256 is recorded before the first counted session, following the precedent of
   [the logotype review pre-registration](../verification/logotype-review-20261114/PREREGISTRATION.md),
   which is committed before the review runs so that the commit time evidences
   the order. The frozen copy contains the pilot-derived keys (CT01 status
   text, CT04 end state, CT12 powered-off end state) and the testability
   decision for CT07–CT09 and CT12; these do not change after the first
   counted session. Later changes go to a deviations log, never silently into
   the frozen copy.
5. **Round 1** sessions (about 45–60 minutes: introduction, consent, tasks in
   a fixed order with CT05 last, SEQ after each task, SUS, short debrief).
6. **Analysis** against the frozen plan: per-task completion, errors, assists,
   time, SEQ, comprehension; SUS summary; findings list ranked by severity and
   frequency.
7. **Fixes** in ordinary source PRs referencing finding IDs.
8. **Re-test round** on a new frozen image with new participants.

Proposed future evidence directory (not created by this change):

```text
docs/verification/u06-usability-<YYYYMMDD>/
  README.md               summary, evidence limits, links
  PREREGISTRATION.md      frozen protocol copy and its SHA-256
  environment.json        bundle manifest hashes, host/QEMU/OVMF/accelerator
  tasks.json              frozen task set, success keys, budgets
  results-aggregate.json  per-task aggregates and SUS summary (no per-person rows with free text)
  findings.jsonl          committed §5 records: no participant field, counts only
  deviations.md           every change after the freeze, with reason
```

## 11. What this does not prove

- It does not prove usability of BaryonOS; it is a plan.
- A pass on CT01–CT06, CT10 and CT11 would validate only a narrow slice on a
  research VM profile, not the full product, the full desktop shell, personal
  sign-in, persisted settings, file search, permissions, NAS or the
  applications in §2.3.
- VM results are evidence level A/V on the recorded host; they are not
  physical-hardware (H) evidence and do not set hardware latency budgets.
- Observer-verified end states are not durability evidence (D) unless backed by
  an independent disk oracle.
- SUS and SEQ measure perceived usability of the tested slice; they do not
  measure security, accessibility conformance or feature parity with Windows,
  macOS or Linux.
- Accessibility results from keyboard-only tasks do not substitute for
  screen-reader tests (U04-T3/T4) or a WCAG/EN 301 549 conformance claim.
- AI review of screenshots or this protocol is not a user test.
- A small qualitative sample does not support statistical generalisation.

## 12. Open owner questions

1. Is the current testable slice (CT01–CT06, CT10 and CT11, plus CT07–CT09 and
   CT12 if confirmed in the bundle) acceptable as the agreed core set for a
   first round, or must U06 wait until the narrowed NT01–NT07 have native
   surfaces?
2. Are the proposed budgets in §6 accepted, changed or replaced?
3. How many participants per round, and is a separate accessibility subgroup
   (shared with U04-T4) required in the first round?
4. Which consent text, legal basis, retention period and local storage location
   apply, and who acts as observer?
5. Is optional host-window screen recording allowed at all?
6. Which language is used for sessions and questionnaires, and is a validated
   translated SUS required if sessions are held in Dutch? The shell and app
   strings on `main` are Dutch (§4), so a session held in another language
   confounds UI comprehension with language comprehension; the owner decides
   whether participants must read Dutch, or whether the round waits for a
   localised bundle.
7. Which host and accelerator (Windows/WHPX or Linux/TCG) is the reference for
   time measures?
8. Is compensation offered?

## Sources

- Bangor, A., Kortum, P. T., & Miller, J. T. (2008). An Empirical Evaluation of
  the System Usability Scale. *International Journal of Human–Computer
  Interaction*, 24(6). <https://doi.org/10.1080/10447310802205776>
- Brooke, J. (1996). SUS: A "quick and dirty" usability scale. In P. W. Jordan
  et al. (Eds.), *Usability Evaluation in Industry*. Taylor & Francis.
  <https://doi.org/10.1201/9781498710411-35>
- Lewis, J. R. (2018). The System Usability Scale: Past, Present, and Future.
  *International Journal of Human–Computer Interaction*, 34(7).
  <https://doi.org/10.1080/10447318.2018.1455307>
- Sauro, J., & Dumas, J. S. (2009). Comparison of three one-question, post-task
  usability questionnaires. *Proceedings of CHI 2009*, ACM.
  <https://doi.org/10.1145/1518701.1518946>

The four references were checked against OpenAlex metadata on 23 November 2026
(title, first author, year and DOI).
