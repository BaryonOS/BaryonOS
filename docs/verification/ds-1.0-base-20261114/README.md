# ds-1.0 base — intermediate verification record (14 November 2026)

**Status: intermediate, not accepted.** This record freezes the ds-1.0 base
(tokens, icons, components, gallery and `docs/design/DESIGN-SYSTEM.md`) so
that the follow-up correction can be reviewed as a separate diff. The base was
verified against the permission-prompt model that applied when the run
started (three choices: once, session, deny). The project owner's decisions of
14 November 2026 supersede that model and add two design ambitions:

- permission labels and grant semantics: ongoing app permissions offer
  "Toestaan tot afsluiten", "Altijd toestaan" and "Weigeren"; "Eenmalig
  toestaan" is used only for single actions whose permission lapses after
  execution; standing grants are revocable under Instellingen → Privacy en
  beveiliging → Appmachtigingen;
- an informed-consent experience that goes beyond a conventional permission
  dialog (task UX-C);
- a next-generation design ambition for the whole UI/UX pipeline, recorded in
  [DESIGN-AMBITION.md](../../DESIGN-AMBITION.md).

UX-D is accepted only after the correction run UX-D-C1 passes an independent
verification that includes these decisions and a design-quality review.

## Run

| Item | Value |
|---|---|
| Source | branch `claude/baryon-os-ui-ux-design-152adb`, HEAD `573840e0` at start |
| Workflow run | `wf_f6f80a6f-a9d` (serial runner, one writer at a time) |
| Workers | 3 writers (tokens and icons; components and gallery; document), 3 independent verifiers, 2 repair writers; 0 errors |
| Duration and size | 3 h 27 min; about 2.37 M subagent tokens |
| Final verdict | round 2: 0 blockers, 0 majors, 8 minors ([findings-final.json](findings-final.json)) |

The verifiers did not read the writers' reports. They checked token and class
consistency against the code and the Orbitaal v2 board, WCAG contrast from
computed styles and sampled pixels in the dark, light and high-contrast
panels, keyboard paths, reduced motion, the 200 % strip, fonts and icons, at
least 15 document claims against their sources, and that protected files
(`ui/design/concepts/`, `ui/prototype/`, `ui/design/brand/`,
`ui/design/fonts/`) were unchanged.

## Renders

`renders/` holds WebP copies (quality 82) of the final verifier's dark, light
and high-contrast gallery panels at 1440 px and the 200 % strip;
`renders/ORIGINAL-PNG-SHA256SUMS` lists the hashes of the original PNG files,
which stay in the ignored evidence directory.

## Open items carried into UX-D-C1

The eight minors in `findings-final.json` and the document's own open items
(contrast of `--color-line-strong` and `--color-text-subtle` under state
fills, missing states and reserved S0 names, shell components that are not yet
components, the "Kern" naming question P-NAMES-1) go to the correction run as
input. The document still cites ignored `build/` evidence paths; final
integration moves the needed scripts into a committed verification record.

## Limitations

Headless Chrome through a raw DevTools-protocol driver at 1440 px; only the
200 % scale was tested; no real screen reader, no other browser engine and no
user testing. Review by AI agents is not an independent human audit. No native
build, QEMU run or native gate applies to these design files.

## Hashes

`SHA256SUMS` covers the frozen design files at commit `46957bac` and this
record (later commits such as UX-D-C1 change those files, so check it in a
checkout of `46957bac`); check with
`sha256sum -c docs/verification/ds-1.0-base-20261114/SHA256SUMS` from the
repository root.
