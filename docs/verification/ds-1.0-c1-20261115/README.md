# ds-1.0 correction UX-D-C1 — intermediate verification record (15 November 2026)

**Status: intermediate, not accepted.** Correction run UX-D-C1 applied the
project owner's permission decision of 14 November 2026 to ds-1.0 and the
prototype. Its third independent verification left 0 blockers, 1 major and
7 minors, so the runner ended `blocked_after_2_repairs` and UX-D stays not
Accepted.

## Run

| Item | Value |
|---|---|
| Base | ds-1.0 base `46957bac` ([record](../ds-1.0-base-20261114/README.md)); branch head `572a943a` at the end of the run |
| Workflow run | `wf_72e6bcf3-50d`: 1 writer, 3 independent verifiers, 2 repair writers; 0 errors |
| Duration and size | 2 h 01 min; about 1.81 M subagent tokens |
| Final verdict | 0 blockers, 1 major (design quality, AI opinion), 7 minors ([findings-final.json](findings-final.json)) |

## What the verifiers confirmed

- **Owner conformance:** ongoing prompts offer exactly "Toestaan tot afsluiten",
  "Altijd toestaan" and "Weigeren"; single actions offer "Eenmalig toestaan",
  "Altijd toestaan" and "Weigeren". No earlier label remains, and every
  duration line matches the enforced validity.
- **Grant semantics:** a grant is keyed on app, resource, right and scope,
  plus the destination for rights that send data off the device. A grant
  never broadens, and a covered request is not asked again.
  `contract-test.mjs` runs as `node contract-test.mjs <repo-root>`;
  48 assertions passed on this commit.
- **Interaction:** with real key presses, focus opens on the prompt title, so
  no single keystroke grants anything, and Escape denies and stores nothing.
- **Blockers closed:** the first verifier's blocker (a send-copy grant without
  its destination in the key) is fixed. So is the second verifier's blocker:
  at 200 % the modal prompt did not scroll. Verifier 2 re-tested the modal at
  960×667 with real keys and every choice is in view.
- **Boundaries:** against HEAD, no protected or native path changed.
  `tools/package_token_guard.py` appears against the base only because of the
  merge of main `e186ec1b` (PR #335), which landed before the last two
  verifications.

## Open items

- **Major (AI opinion, not user evidence).** The empty, loading and error
  states (`.ds-empty`, `.ds-skeleton`) follow a generic web-app template
  without the Orbitaal place or Triade identity. These states are unchanged
  since the base; verifier 0 had not viewed them. This is carried into
  UX-A11Y-DS as a must-fix: the project owner requires accessibility research
  before further design work, and loading states are motion and stimulus-load
  territory.
- **Minors (see `findings-final.json`):**
  - Overlapping `ask()` calls can store a duplicate grant, and `revoke()`
    removes one entry at a time (must-fix for the next writer: revocability).
  - The duration line has too little weight in the contrast theme.
  - The gallery's covered-request specimen stays interactive.
  - Evidence for capability row XC09 sits in ignored `build/`; the contract
    test is now committed here.
  - Design-quality notes on `.ds-progress`.
- **Owner questions raised by the run:** O-G12 (does "tot afsluiten" mean
  closing the app, or shutting down; the current reading is closing the app),
  O-G4 (the single-action labels) and O-G2 (persistent denial, recommended
  option B). An ADR is required before native persistent grants.
- **Stale counts:** the S1 matrix now has 216 rows; the S1 record describes
  commit `8e4abd8d` (215 rows).

## Limitations

A hook blocked image reads, so visual judgement used inline viewport
screenshots. Several sections, including the light grant rows and the 200 %
renders, were judged from DOM measurement only. No screen-reader test was
run, and there was no user testing. Review by AI agents is not an independent
human audit, and no native gate applies to these design files. `renders/`
holds WebP copies (quality 82) of verifier 2's renders, with the original PNG
hashes in `renders/ORIGINAL-PNG-SHA256SUMS`.

## Hashes

`SHA256SUMS` covers this record and the design files at this commit. Check it
from the repository root with
`sha256sum -c docs/verification/ds-1.0-c1-20261115/SHA256SUMS`.
