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
presence in an interactive bundle must be confirmed against the bundle
`manifest.json` before the preregistration is frozen. If absent, it becomes
NOT YET TESTABLE for that round.

## 2. Core task list

### 2.1 Testable on the current interactive bundles

| ID | Task given to the participant | Native surface (source) | Trace |
|---|---|---|---|
| CT01 | Open Materiaal, create a new document, type a short given sentence and save it | `native-window-session` bundle; [`sdk/examples/materiaal`](../../sdk/examples/materiaal), [`sdk/examples/window-session`](../../sdk/examples/window-session); contract [NATIVE-WINDOW-SESSION.md](NATIVE-WINDOW-SESSION.md); accepted U01 run [U01-ACCEPTANCE.md](../verification/mature-session-20261102/U01-ACCEPTANCE.md) | FR-A02, FR-U03, UC01 (partial) |
| CT02 | Close the document, find it again in the list and reopen it; confirm the text is intact | Same bundle; save/list/reopen chain recorded in U01 acceptance | FR-A01, FR-A02, D1 "find a file again", UC07 (partial) |
| CT03 | Use the Calculator for a given sum while Materiaal stays open, then return to the document | [`sdk/examples/calculator`](../../sdk/examples/calculator) in `native-window-session` | FR-U01, U02 (switching/focus) |
| CT04 | Move the two windows so both are fully visible side by side by dragging | `native-window-session` (two draggable windows, U01 acceptance); [`sdk/examples/compositor`](../../sdk/examples/compositor) | D1 "arrange a window", FR-U01 |
| CT05 | Complete CT01–CT02 with the keyboard only (no pointer) | Keyboard focus route via [`sdk/examples/input-service`](../../sdk/examples/input-service) and the compositor; [NATIVE-INPUT-DAEMON.md](NATIVE-INPUT-DAEMON.md) | FR-U05 (keyboard part), U04, UC09 (partial) |
| CT06 | In the session preview, try to sign in personally, read the result, then enter the guest workspace | `native-system-session-preview`; [SESSION-AND-DESKTOP.md](SESSION-AND-DESKTOP.md); runner [`tools/run_session_preview.py`](../../tools/run_session_preview.py) | FR-U03 (comprehension of a refusal); **no** FR-S02 claim: no real authentication exists |

CT06 measures only whether the refusal and the guest route are understood.
It is explicitly not a sign-in test.

### 2.2 Conditional

| ID | Task | Native source on `main` | Condition |
|---|---|---|---|
| CT07 | Rename the document, place it in a collection, find it with the library search field | [NATIVE-MATERIAL-LIBRARY.md](NATIVE-MATERIAL-LIBRARY.md) (U03 implementation; native acceptance open) | Library build present in the frozen bundle manifest |
| CT08 | Restore an earlier revision of the document | Same; [S06-COLLECTIONS-HISTORY.md](S06-COLLECTIONS-HISTORY.md) | Same |
| CT09 | Copy a selected sentence and paste it into another place | [NATIVE-CLIPBOARD.md](NATIVE-CLIPBOARD.md) (U02 contract; not closed) | Clipboard route present in the frozen bundle manifest |

### 2.3 NOT YET TESTABLE

| ID | Task | Missing native surface (source) | Trace |
|---|---|---|---|
| NT01 | Complete first use and create a local profile | No OOBE code; [PROFILES-AND-USERS.md](PROFILES-AND-USERS.md) is a plan | FR-S01 |
| NT02 | Sign in with a password, lock and unlock | No real authentication or key boundary; session preview rejects personal sign-in | FR-S02, B02 |
| NT03 | Change a setting and find it unchanged after restart | No Settings app (DESKTOP-SHELL step D5) | FR-S03, U05 |
| NT04 | Start an app from Zenit/Orbit | Trusted shell, launcher and dock exist only host-rendered (DESKTOP-SHELL §7, step D3) | D1 "start an app", FR-U01 |
| NT05 | Find a file with system search (Signaal) | No native search index (step D4) | FR-O07, U05 |
| NT06 | Browse known folders in a Files app | No native Files app (step D5) | FR-A01 |
| NT07 | Shut down or restart with unsaved work open | No native save/discard/cancel shutdown flow in the interactive bundles | D1 "shut down", FR-S04 |
| NT08 | Answer a permission request and later revoke it | Native permission phase blocked (ledger U03 task, window-session manifest and `ui/native` ownership) | A03, FR-U02 |
| NT09 | Understand and recover from a save failure | Fault/probe profiles are refused by the Windows launcher ([tools/windows-qemu/README.md](../../tools/windows-qemu/README.md)); design only in [UX-MATERIAAL-SAVE-RECOVERY.md](UX-MATERIAAL-SAVE-RECOVERY.md) | FR-U03, UC08 |
| NT10 | Use the system with a screen reader | Blocked `U04-T3` (TTS/braille dependency decision, audio) | FR-U05, U04, UC09 |
| NT11 | Snap a window to half the screen or switch with Alt+Tab | Window arranging and Alt+Tab are design (DESKTOP-SHELL §2) | FR-U01 |
| NT12 | Pair a personal NAS and resume work | No interactive native pairing flow | DESIGN-AMBITION task table, FR-P02 |
| NT13 | Import a photo, view it and find it again | Image components (`sdk/examples/image-viewer`, [NATIVE-IMAGES.md](NATIVE-IMAGES.md)) are not in an interactive bundle | UC04, FR-A03 |
| NT14 | Work with a spreadsheet or presentation | `sdk/examples/spreadsheet`, `sdk/examples/presentation` not in an interactive bundle | FR-A14, FR-A15 |

The current testable set therefore covers a narrow document-centred slice. A
U06 pass on that slice alone would not satisfy "agreed core tasks" unless the
project owner explicitly agrees that slice as the core set (see §11).

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
| CT01 | A document with the given sentence exists and the UI shows a saved state |
| CT02 | The same document is reopened from the list; text identical to CT01 |
| CT03 | Correct result shown in Calculator; focus returned to the document with text intact |
| CT04 | Both windows fully visible and non-overlapping |
| CT05 | CT01–CT02 end states reached without pointer input |
| CT06 | Participant states correctly that personal sign-in is not available and reaches the guest workspace |
| CT07 | New name shown; document listed in the target collection; found via search |
| CT08 | Earlier revision content shown as current, participant can state which version is active |
| CT09 | Pasted text identical to the selection at the intended location |

For CT01, CT02 and CT07–CT09 the observer additionally records whether the
end state persisted after the session (host-side disk oracle where a runner
provides one). Observer judgement alone is evidence level A/V, not D.

## 5. Friction and visual-inconsistency log

Each finding is one record. The format is plain JSON lines so that it can be
aggregated without personal data:

```json
{"finding": "F-003", "round": 1, "participant": "P05", "task": "CT02",
 "kind": "friction", "severity": 2, "frequency": 3,
 "observed": "List entry not recognised as the saved document",
 "screen_ref": "frame-hash-or-screendump-id", "fr": ["FR-A01", "FR-U04"],
 "status": "open", "resolution": null, "retest_round": null}
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
  question (§11).
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
- **Pseudonyms** (`P01`, `P02`, …) are the only identifiers in any record that
  leaves the observer's local disk.
- **No personal data in the repository.** Only aggregated, de-identified results
  and findings (§5 format) are committed. Free-text quotes are paraphrased and
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

## 10. Procedure and evidence layout

1. **Owner approval** of tasks, budgets, consent and privacy handling (§11).
2. **Preregistration freeze:** copy of this protocol with the agreed task set,
   success keys, budgets, analysis plan and bundle `manifest.json`; its SHA-256
   is recorded before the first counted session. Later changes go to a
   deviations log, never silently into the frozen copy.
3. **Pilot** with one or two people (not counted) to check scripts and timing.
4. **Round 1** sessions (about 45–60 minutes: introduction, consent, tasks in
   a fixed order with CT05 last, SEQ after each task, SUS, short debrief).
5. **Analysis** against the frozen plan: per-task completion, errors, assists,
   time, SEQ, comprehension; SUS summary; findings list ranked by severity and
   frequency.
6. **Fixes** in ordinary source PRs referencing finding IDs.
7. **Re-test round** on a new frozen image with new participants.

Proposed future evidence directory (not created by this change):

```text
docs/verification/u06-usability-<YYYYMMDD>/
  README.md               summary, evidence limits, links
  PREREGISTRATION.md      frozen protocol copy and its SHA-256
  environment.json        bundle manifest hashes, host/QEMU/OVMF/accelerator
  tasks.json              frozen task set, success keys, budgets
  results-aggregate.json  per-task aggregates and SUS summary (no per-person rows with free text)
  findings.jsonl          §5 records, pseudonymised
  deviations.md           every change after the freeze, with reason
```

## 11. What this does not prove

- It does not prove usability of BaryonOS; it is a plan.
- A pass on CT01–CT06 would validate only a document-centred slice on a research
  VM profile, not the full product, the desktop shell, sign-in, settings,
  search, permissions, NAS or applications in §2.3.
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

1. Is the current testable slice (CT01–CT06, plus CT07–CT09 if present in the
   bundle) acceptable as the agreed core set for a first round, or must U06 wait
   until NT01–NT07 have native surfaces?
2. Are the proposed budgets in §6 accepted, changed or replaced?
3. How many participants per round, and is a separate accessibility subgroup
   (shared with U04-T4) required in the first round?
4. Which consent text, legal basis, retention period and local storage location
   apply, and who acts as observer?
5. Is optional host-window screen recording allowed at all?
6. Which language is used for sessions and questionnaires, and is a validated
   translated SUS required if sessions are held in Dutch?
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
