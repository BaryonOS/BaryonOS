# First-party English coverage inventory — 18 November 2026

The [source-bound inventory](coverage.json) records 392 first-party Markdown candidates at parent source `68757cd2` and 434 protected/source exclusions, for 826 tracked Markdown files. Normal README/contributing/development/Wiki-source navigation reaches 140 repository documents in the recorded graph. Source hashes, exclusion reasons, language-candidate line numbers, manual dispositions, compatibility anchors and open work are retained without copying historical source prose into this receipt.

The current audit reuses the scope and exception rules of the existing 9 November English/link records, plus the reviewed Phase 0 translations. The exact older `a462e0d` Git tree is unavailable locally; a single direct fetch returned `not our ref`. Consequently, no old exact-blob language acceptance is claimed. The counts above describe the current recorded source, not the older report's 280-document claim.

## What was checked

- The collector uses the existing Markdown lexer approach, with the already available Node `v24.19.0` and `marked` `17.0.5`. No dependency, installation, hook, paid service or mandatory CI gate is added. The Python snapshot checker uses only the standard library.
- Dutch-word candidate detection excludes fenced code, inline code, URLs and HTML metadata. It identifies possible prose for review; absence of a match does not prove professional English or localisation quality.
- All detected normal-navigation candidates were reviewed. They are preserved original quotations, actual interface/name vocabulary, multilingual search synonyms or historical session names, except the Dutch RB-01 baseline, which now has a complete English equivalent and explicit archived-original navigation. Twenty candidate documents have a recorded preservation disposition. Other navigation nodes received automated scanning, not a complete human semantic audit.
- The unabridged Dutch release assignment remains hash-bound and has an existing English translation. Actual UI strings and fixtures are deliberately preserved; retaining those strings in a source explanation does not accept runtime localisation.
- All eleven previously required compatibility anchors remain present. The graph reports one pre-existing missing generated `build/toolchain-provenance.json` reference. This is recorded rather than represented as a newly passing link.
- Wiki-context references resolve in the separate actual Wiki checkout. They are not falsely reported missing because the six operational guides are absent from the repository's concept-source directory. Publication and remote byte equality are recorded in the [Wiki publication receipt](../issue274-wiki-publication-20261118/PUBLISHED.md).
- Both legacy repository names return GitHub API `Moved Permanently` responses pointing to repository ID `1435549142`. Historical pinned URLs remain unchanged. A complete browser/render/redirect or historical deep-link HTTP audit is not claimed.
- Default community-form prose and all six current Discussion category names/descriptions are English. Stable IDs, category slugs, parser aliases and form behaviour are unchanged.

## Reproduction and maintenance

From the repository root, reproduce the exact recorded source/candidate dispositions with:

```sh
python3 -B docs/verification/issue274-language-coverage-20261118/check.py
```

An optional `--ref <full-commit-SHA>` reads the listed files from an existing immutable Git snapshot. Later source changes require a fresh reviewed inventory; the checker reports changed or unclassified tracked documents rather than silently accepting them. Use `--inventory <repository-relative-path>` for a later versioned inventory. A source-bound receipt remains historical evidence after a later documentation update.

`--self-test` copies only the recorded documentation/form inputs into a temporary ignored `build/` directory and creates a local Git index without a remote. The positive control returns 0; changed prose/source identity and a newly tracked unclassified document each return the expected raw exit 1. The real source files remain untouched.

Regenerate navigation and language candidates locally, using an existing installation of the recorded Markdown lexer and a checkout of the actual Wiki:

```sh
node docs/verification/issue274-language-coverage-20261118/collect.cjs --wiki /absolute/path/to/BaryonOS.wiki
```

The collector writes only ignored `build/english-coverage-20261118/` reports. It performs no network request or publication and does not fetch missing history. Refresh the versioned inventory only after reviewing new candidates, scope and source identity. A zero exit from the snapshot checker means that the recorded source and dispositions were reproduced; it is explicitly **not** full English/localisation acceptance.

## Remaining acceptance work

Issue #274 remains **open**. Thirty-four dated Dutch audit/research originals still need appropriate English archive entrypoints/equivalents; none was rewritten or falsely classified as translated. The inventory lists them individually. Native `menu_label` and `status_label` at source `67d5859d` return fixed Dutch user-visible strings without a locale parameter. This is a concrete localisation residual, not a complete audit of every application's localisation architecture. The native/UI workstream owns that implementation; this documentation change does not replace actual strings, fixtures or storage identifiers.

Full public UI/runtime localisation acceptance remains open. Browser/deep-link and language-quality review limits above are evidence limits, not newly introduced human-audit prerequisites. Progress remains 41/100. No native feature or security requirement is accepted by the inventory or translation.
