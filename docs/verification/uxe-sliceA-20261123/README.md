# UX-E slice A — connected prototype journey, stations 1–4 (23 November 2026)

Status: **slice accepted on AI verification only** (no user study, no human audit, no
real screen reader, no native implementation). Base `ec459716`. Browser simulation in
`ui/design/prototype/`; it accepts no `docs/progress.json` criterion. UX-E as a whole
stays open until slices B and C are accepted.

## Scope

Slice A of the UX-E plan (ignored `build/design-evidence/uxe/PLAN.md`): a shell entry
(`ui/design/prototype/index.html`, `journey.js`, `journey.css`, `journey/*.js`) with
station 1 Aanmelden (lock screen, profile choice, secret entered only on the Kern
session-service surface, recovery route, compact first start), station 2 Bureaublad
(Verder waar je was, Orbit dock, Menu, status area, clock), station 3 Apps en vensters
(open, focus, minimise, maximise, close, snapping, switcher, stack mode) and station 4
Zoeken (grouped local search; assistant row labelled Gesimuleerd). Also: a journey
sweep (`sweep.html`, `tests/run-sweep.mjs`), journey tests and
`docs/design/PROTOTYPE.md`. `strings.js` was only appended to. `contract.js`, the
contract tests, `data.js`, the design-system files, `tokens.css` and `check.mjs` are
unchanged.

## Run

Run `wf_0760de51-dca`: one planner, one writer, then fresh independent verifiers with
at most two repairs. The run was interrupted twice by session cutoffs and resumed from
its journal. The interrupted partial evidence was kept apart, in directories with an
`.interrupted-*` suffix.

| Round | Result |
|---|---|
| Verifier 0 | Not accepted; repair 1 |
| Verifier 1 | Not accepted. Blockers: a transparent window layer took pointer input over the whole desktop, and the session card clipped at 320×640 and 720×450 (WCAG 1.4.10). Majors: a literal "nullnull" in the search preview, sweep coverage gaps, and PROTOTYPE.md inconsistencies. Repair 2 followed |
| Verifier 2 | **Accepted**: E1, E2 and E4–E10 met (E3 belongs to slice C). 0 blockers, 0 majors, 7 minors |

## Acceptance evidence

- Stations 1–4 were walked with the keyboard only at 1440×900 and 320×640, and with the
  pointer, using CDP input events in headless Chrome.
- Text contrast is at least 7.07:1 in dark, light, T1 and T2, and 21:1 in the contrast
  theme. Inputs measure 4.59–21:1 and focus rings about 15.8:1 or more.
- The sweep `node ui/design/prototype/tests/run-sweep.mjs` covers 378 cells (1440×900,
  320×640 and 720×450; dark, light and contrast; tier-1 and tier-2 axes; 200 %
  scale) with 0 failures.
- Lead re-run on 23 November 2026, after the verifier:
  - `node --test --experimental-test-isolation=none ui/design/prototype/tests/*.test.mjs`
    exits 0 (49/49: 29 contract and 20 journey tests).
  - `node docs/verification/a11y-ds-20261115/check.mjs` exits 0 (0 failures).
  - `python3 -B tools/check_a11y_matrix.py .` exits 0.
  - `tokens.css` sha256 `c4513523…ee39` and `check.mjs` sha256 `82582384…2675` are
    unchanged. The untracked O-P1 work in progress matches its launch hashes.

## Open minors (none blocks the slice)

1. When Thuis-NAS is set to Niet bereikbaar, the Orbit status and the privacy
   instrument change, but the Verder, Menu and search rows still read only
   "Thuis-NAS".
2. `.jr-continue` (overflow-y: auto) clips the plates' shadows at the scroll edge in
   the light theme (AI opinion).
3. Rekenmachine opens in a fixed window that is mostly empty (AI opinion).
4. Existing `DATA.notifications` copy states encryption and recovery as fact next to
   only a small "Gesimuleerd" chip. This is part of the known lead follow-up on
   over-claiming existing strings.
5. The lock-screen region "Aanmelden" has no heading, so heading navigation skips it
   until the sign-in surface opens.
6. Several journey tests grep source text instead of testing behaviour.
7. Outside UX-E scope, in ds-1.1: the `.ds-window` border measures 1.27–1.32:1 and the
   `.ds-orbit` border 1.36–1.53:1 against the desk. The boundary relies on elevation
   shadow and the title bar. Whether this fails WCAG 1.4.11 depends on whether the
   border is the information needed to identify the component. It is recorded as a
   design-system follow-up and is not dismissed.

## Limits

- Headless Chrome with focus emulation only. Browser-reserved accelerators (such as
  Alt+` and Ctrl+Esc) are mapped to stand-ins and were not tested end to end.
- Only the CDP accessibility tree was checked, not a real screen reader.
- Media features were emulated. No human trials took place.
- Raw evidence and journals are in the ignored `build/design-evidence/uxe/` and the
  coordinator checkpoint.
