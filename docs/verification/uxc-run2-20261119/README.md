# UX-C run 2 — informed-consent permission prompt (19 November 2026)

Status: **accepted on AI verification only** (no user study, no human audit, no native
enforcement). Base `a6afd1a2`. Design-system prototype evidence; it accepts no
`docs/progress.json` criterion.

## Scope

Concept b ("Eén vraag") of UX-C run 1, with grafts from concepts a and c, built into
`ui/design/system/` (`.ds-ask` in a native `<dialog>`, `.ds-grants`), the behaviour
model `ui/design/prototype/contract.js`, Dutch copy in `strings.js`, contract tests in
`ui/design/prototype/tests/`, `docs/design/DESIGN-SYSTEM.md` §11.1 and the prompt and
grant rows of `docs/design/suite/CAPABILITY-MATRIX.md`. Owner inputs: the permission
model of 15 November 2026, readings R2–R5 and decisions OQ-1..OQ-6 and MF-1..MF-6 of
18–19 November 2026. The draft ADR `PROPOSED-ADR.md` in this directory has status
**Proposed** and is not in `docs/DECISIONS.md`.

## Runs

| Run | Result |
|---|---|
| Run 2 `wf_ff2800a7-fd9` (writer, fresh verifier, at most 2 repairs) | `blocked_after_2_repairs`: 13 of 14 items met. One blocker: a remembered Toestaan could answer a later irreversible single-action external send with the same five key fields (OQ-1). |
| Run 2r `wf_043dc049-a87` (one targeted repair of `contract.js` and tests, one fresh verifier) | Blocker fixed; 29 of 29 tests pass. Mutation probes (ignore a key field, sticky remember, persisted temporary denial, and others) each make the suite fail. Two majors remained: U7 (simulation controls unlabelled) and U9 (documentation said the comparison line was in a disclosure). |
| Lead correction under owner decision A, then narrow fresh verifier `wf_70100a30-fbb` | U7 and U9 met; no regressions; gates pass; protected files unchanged. |

## Acceptance evidence (final verifier)

- U7: the five simulation controls form a named group (`aria-describedby` points to a
  visible "Simulatie, geen echte actie" note). The status table carries a "Gesimuleerd"
  row and a lifecycle row reading "voorgesteld, nog niet gebouwd". Note contrast is
  8.47:1 (light), 8.45:1 (dark) and 21:1 (contrast).
- U9: in the DOM, "Vergelijk: …" is in the first layer, while "Waarom niet" and the
  five-field key are in closed `<details>`. §11.1 matches, and the OQ-1 paragraph
  matches `contract.js`.
- WCAG review of three minors, none failed:
  - The app-reason frame (2 px dashed) measures 7.68:1, 10.11:1 and 21:1 from the
    token and from the rendered pixels (SC 1.4.11).
  - The path connector matches the same token, and the text and list order also
    carry the direction.
  - No WCAG 2.2 AA criterion requires a switch to turn explanations off.
- Gates:
  - `node --test --experimental-test-isolation=none ui/design/prototype/tests/*.test.mjs`
    exits 0 (29/29).
  - `node docs/verification/a11y-ds-20261115/check.mjs` exits 0 (0 failures).
  - `python3 -B tools/check_a11y_matrix.py .` exits 0.
- Protected: `ui/design/system/tokens.css` sha256 `c4513523…ee39` and
  `docs/verification/a11y-ds-20261115/check.mjs` sha256 `82582384…2675` are unchanged.
  Frozen `ui/prototype/` is unchanged, and the untracked O-P1 work in progress has
  matching hashes.

## Known limits and open items

- Remembered and temporary decisions are prototype behaviour only, and the gallery
  labels them so. Native enforcement needs an accepted ADR first.
- Because one decision is stored per five-field key, a remembered Weigeren on the
  irreversible send replaces a remembered Toestaan for an ongoing send-copy with the
  same five fields. Access only narrows, and the owner accepted this behaviour
  (decision A, 19 November 2026).
- Open minors:
  - There is no setting to turn explanations off.
  - The path reads as a vertical list rather than concept b's horizontal "path of
    light" (AI opinion).
  - The gallery page scrolls horizontally at 320 px because of the Knop,
    Lijstrij and Privacy-instrument cards. This is pre-existing at base and outside
    scope.
- Some 200 % and 320 px measurements used headless-render probe pages. 200 % is
  viewport emulation, not browser zoom.
- Raw evidence, run results and journals are in the ignored
  `build/design-evidence/uxc-run2/` and the coordinator checkpoint.
