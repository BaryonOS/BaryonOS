# UX-E slice B: files, versions, consent in context and settings, stations 5–8 (23 November 2026)

Status: **slice accepted on AI verification only.** There was no user study, no human
audit, no real screen reader and no native implementation. The base is `44ef6030`. This
is a browser simulation in `ui/design/prototype/` and it accepts no `docs/progress.json`
criterion. UX-E as a whole stays open until slice C is accepted.

## Scope

Slice B of the UX-E plan (the ignored file `build/design-evidence/uxe/PLAN.md`) covers
four stations:

- Station 5 Bestanden: places, a simulated external provider, grouping and search.
- Station 6 Versies: compare, restore and undo.
- Station 7 Toestemming in context: Webvenster asks for internet, Foto's asks for the
  camera, and Notities asks to read another document.
- Station 8 Instellingen: Appmachtigingen, Opslag en NAS, Updates, and Privacy en
  beveiliging with the session file grants.

All permission requests go through `contract.js` and the `.ds-ask` prompt. The prompt
renderer was factored out of `gallery.js` into `ask-view.js` without any visible or
behavioural change to the gallery. `strings.js` was only appended to.

### Owner file-open model (owner decision of 23 November 2026, item E11)

- **(a) Opening a file.** An explicit open from Bestanden, Zoeken, Verder waar je was or
  Openen met seeds a temporary read of that one document, with no prompt. Opening never
  grants write.
- **(b) Granting write.** Write is granted only by the shell-drawn Bewerken control in
  the window frame, or by the system save dialog. Script clicks, forged objects and
  app-originated calls are refused. A Ctrl+S on a read-only document never escalates:
  focus moves to Bewerken and a short note is announced. Once write is held, Ctrl+S
  saves without a prompt.
- **(c) Revocation.** Each of these ends access, after which the next write fails
  closed:
  - closing the document (`grants.endTemporary` per document right);
  - closing the app (`appClosed`);
  - Intrekken in the grants view, for every temporary document grant including a read
    allowed in the prompt;
  - signing out (`logout`).
- **(d) Separate authorization.** Another file, a broader scope, a remembered grant and
  any network or camera right go through `ask()` and the prompt. There is no delegation
  between apps.

## Lead decisions in this slice

1. **Contract extension.** `44ef6030` adds `grants.endTemporary(key)` to the protected
   consent contract. It is additive only and covered by three tests. The run blocked
   without it: `grants.revoke` acts only on remembered decisions, and `appClosed` would
   also erase the app's R2 temporary denials. For temporary grants, item E11 (c)
   "grants.revoke on the five-field key" is read as `grants.endTemporary` on that key.
2. **Ctrl+S.** The strict reading of E11 (b) was chosen (see (b) above).
3. **App identity at station 7.** The requesting app is presented consistently as the
   third-party app Webvenster.
4. **System status.** While a running app holds a connect grant, the Orbit status, the
   privacy instrument and the quick settings say so.
5. **Rounds beyond the run's limit.** One lead follow-up round (round 3) was run after
   the two-repair limit was used up. After it, a lead fix of one remaining major was
   checked by a separate verifier (see Run).

## Run

| Run / round | Result |
|---|---|
| `wf_706cde97-b1d` (writer, verifier 0, repairs 1–2) | `blocked_after_2_repairs`: 1 blocker (the consent contract could not end one temporary grant, which only a contract change could fix), 3 majors (station 7 identity, contradicting status, Ctrl+S escalation), 3 minors. Majors and blockers by round: 6, 9, 4 |
| Lead commit `44ef6030` | `grants.endTemporary` added; the 29 UX-C contract tests are unchanged and pass, and 3 new tests pass |
| `wf_14ad6c1e-634` (lead follow-up: one fresh writer, one fresh verifier) | E1, E2 and E4–E11 met. 0 blockers, 1 major (after Vergeten or Wijzigen the open Webvenster window still showed internet in use), 6 minors |
| Lead fix plus focused verifier (`build/design-evidence/uxe/B-verify-4/`) | `journey.js` re-renders the Webvenster and Foto's windows on permission ops `revoke`, `change` and `end-temporary`; PROTOTYPE.md section 13.6 updated. **Accepted** for the scoped fix: Vergeten and Wijzigen clear the window in place, focus stays in Instellingen, the window stays open, and an unsaved Notities draft is untouched |

## Acceptance evidence

- Stations 5–8 were walked with the keyboard only and with the pointer, using CDP input
  in headless Chrome, at 1440×900 and 320×640, in dark, light and contrast.
- The generic contrast scan found 0 samples below threshold in dark, light, contrast,
  T1 and T2.
- The sweep `node ui/design/prototype/tests/run-sweep.mjs` covers 798 cells with 0
  failures. Warnings are recorded but do not fail the sweep.
- `node --test --experimental-test-isolation=none ui/design/prototype/tests/*.test.mjs`
  exits 0 with 72 of 72 passing and 0 todo or skipped. These are 23 + 6 UX-C contract
  tests (unchanged), 3 `endTemporary` tests, and the journey and file-access tests,
  including positive, negative, revocation and Ctrl+S tests.
- `node docs/verification/a11y-ds-20261115/check.mjs` exits 0, and
  `python3 -B tools/check_a11y_matrix.py .` exits 0.
- These protected files are unchanged:
  - `contract.js` and its tests, identical to `44ef6030`;
  - `tokens.css` (sha256 `c4513523…ee39`);
  - `check.mjs` (sha256 `82582384…2675`);
  - the untracked O-P1 work in progress, which matches its launch hashes.
- The raw logs, scripts and screenshots are in the ignored directories
  `build/design-evidence/uxe/B-*`. The run results and journals are in the ignored
  directory `build/design-pipeline-checkpoint/runs-20261115/uxe-sliceB-*`.
- Lead change after the verifiers: the unused loop variable `hit` was removed from
  `journey/sweep.js` (code-quality finding). The tests and the sweep were re-run
  afterwards.

## Open minors (none blocks the slice)

1. Foto's keeps its old in-app note ("Onthouden … Toegestaan") after the camera
   decision is forgotten or changed, and re-inserts it as a status region on every
   re-render.
2. With two Webvenster windows, Toestaan in one does not refresh the other until it
   re-renders.
3. A temporary connect grant has no Intrekken row; it ends with the app session.
4. In station 6 the toolbar wraps, so 'Groeperen op' stacks under the find field.
5. The decorative `.ds-empty-orbit` reports overflow at 320 CSS px and at 200 %. It is
   aria-hidden, nothing is lost and the page does not scroll.
6. Feasibility chips inside page headings become part of the heading's accessible name.
7. Station 7 tiles let desktop text show through the gaps (AI opinion).
8. In the Rondleiding, stations 7 and 8 sit below the fold of its scroll box.
9. PROTOTYPE.md numbering: the "12.5 Checks performed" heading sits under section 13.

## Limits

Headless Chrome only. Screen-reader structure was inferred from the DOM and roles, not
from a real screen reader. No native, QEMU, hardware or human evidence was produced. A
second AI review is not an independent human audit. All functions are prototype
behaviour: Werkt nu in dit prototype for the simulation, Technisch haalbaar, nog niet
gebouwd for native enforcement by BaryonOS.
