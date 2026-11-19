# OSDev reference corpus — rights assessment, 19 November 2026

Scope: issue #283, owner route of 19 November 2026 under
[ADR-W001473-PROVENANCE-20261119](../../DECISIONS.md). This is a risk-ranked
inventory of the 3,054 wiki records in `.agents/skills/os-dev/references/`
(before the W001473 exclusion), for owner and reviewer decisions. It is **not a
legal assessment**, grants no clearance, applies no licence and changes no
corpus file. Per-record results are in [classification.json](classification.json).

## Method

- **Revision history:** the repository's own export metadata
  (`.agents/skills/os-dev/assets/wiki-revision-metadata.jsonl.gz`), giving
  every revision's date, contributor and edit comment.
- **Licence statements in the corpus:** W003005 (OSDev Wiki:Copyrights) licenses
  content "added since June 6 of 2011" under CC0; W002031 (OSDev Wiki:License)
  calls the wiki "of mixed licensing". Both are cited as written.
- **Text screening:** each record's text was matched for copyright notices,
  third-party licence names, copy markers ("copied from", "taken from",
  "from Wikipedia" and similar) and quotation blocks. A match is a reason for
  review, not a finding.
- **Not available:** older revision texts (the export holds the latest text only)
  and the live wiki or its archives, which refused automated access from this
  environment. No access control was circumvented.

## Classes

| Class | Definition | Records | Of which redirects | Flagged by screening |
|---|---|---:|---:|---:|
| A | all revisions on or after 6 June 2011 | 1,815 | 331 | 73 |
| B | revisions before and after 6 June 2011 | 574 | 20 | 54 |
| C | only revisions before 6 June 2011 | 637 | 173 | 22 |
| D | explicit import comment in the history | 28 | 4 | 4 |

Class D takes precedence over B and C; the 641 pre-CC0-only pages reported
earlier are the 637 in C plus 4 imported pages.

## Result

- **Reusable on a verifiable licence statement (1,742 records):** class A records
  without a screening flag. Their entire recorded history falls under the CC0
  statement in W003005. This relies on the site's own statement; individual
  contributor grants were not verified separately.
- **Uncertain, under review (1,284 records):** the 73 flagged class A records
  (embedded third-party text may carry its own terms), all 574 class B records
  (a later edit does not license older text that may survive) and all 637 class C
  records. They are **not removed blindly**. Next step: individual review in
  descending risk order — flagged records first, then non-redirect class C, then
  non-redirect class B — keeping a record only on a verifiable licence, original
  permission or an independently written replacement, and otherwise excluding
  the affected text with `scripts/exclude_sources.py`.
- **Excluded or to be excluded (28 records):** W001473 (PR #397) and the other 27
  class D records (separate scoped PR), because an explicit import from an earlier
  wiki without established terms cannot be shown to be reusable. Redirect pages in
  class D are excluded too; no claim is made about the protectability of short
  texts.

## Residual risk in the public Git history

The corpus is in the public repository since at least commit `500422c`
(11 November 2026) under its current path. Exclusion stops distribution at HEAD
only: earlier commits, forks and clones keep the text. Removing it from history
would need a history rewrite, which is not authorized. No project licence
(MPL-2.0) is claimed for any wiki text.

Issue #283 remains open until the review of classes A-flagged, B and C is
complete and the resulting measures are merged. The product score is unchanged.
