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
- **Licence statements in the corpus:** OSDev Wiki contributors, "OSDev Wiki:Copyrights"
  (W003005), revision 11734 of 7 July 2011,
  <https://wiki.osdev.org/index.php?title=OSDev_Wiki%3ACopyrights&oldid=11734>,
  licenses content "added since June 6 of 2011" under CC0. OSDev Wiki
  contributors, "OSDev Wiki:License" (W002031), revision 11735 of 7 July 2011,
  <https://wiki.osdev.org/index.php?title=OSDev_Wiki%3ALicense&oldid=11735>,
  requires public-domain compatibility for new content "as from 03-07-2011",
  states that authors of existing material are to be informed, and calls the
  wiki "of mixed licensing". Both are cited as written from the export copy; the
  live revisions were not re-fetched. **Assumption:** the CC0 statement governs
  content from 6 June 2011; because W002031 names a later date, the 14 class A
  records with a revision between 6 June and 2 July 2011 are treated as uncertain
  rather than reusable (listed under `cc0_statement_window` in
  classification.json).
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

- **Reusable on a verifiable licence statement (1,728 records):** class A records
  without a screening flag and without a revision between 6 June and 2 July 2011. Their entire recorded history falls under the CC0
  statement in W003005. This relies on the site's own statement; individual
  contributor grants were not verified separately. Two further limits apply:
  pattern screening is not exhaustive and can miss embedded third-party text
  that a contributor could not license under CC0, and these records were not
  read individually. The result is reuse on the stated licence basis, not a
  cleared rights position.
- **Uncertain, under review (1,298 records):** the 14 class A records in the
  6 June – 2 July 2011 window, the 73 flagged class A records
  (embedded third-party text may carry its own terms), all 574 class B records
  (a later edit does not license older text that may survive) and all 637 class C
  records. They are **not removed blindly**. Next step: individual review in
  descending risk order — flagged records first (149), then non-redirect class C
  (442), then non-redirect class B (500), then the unflagged redirects of classes
  B and C (193; for example W001387 and W001388), then the 14 window records — keeping a record only on a
  verifiable licence, original permission or an independently written
  replacement, and otherwise excluding the affected text with
  `scripts/exclude_sources.py`. The review is complete only when every one of the
  1,298 records has a recorded disposition.
- **Excluded or to be excluded (28 records):** W001473 (PR #397) and the other 27
  class D records (separate scoped PR), because an explicit import from an earlier
  wiki without established terms cannot be shown to be reusable. Redirect pages in
  class D are excluded too; no claim is made about the protectability of short
  texts.

## Review of flagged records

[FLAGGED-REVIEW.md](FLAGGED-REVIEW.md) records the individual review of the 149
flagged class A–C records: 5 contain third-party text with unknown terms and are
excluded; 5 carry third-party material under a verifiable licence with
obligations and 12 contain short quotations, both escalated for an owner
decision; 127 contain no identified third-party text.

## Residual risk in the public Git history

The corpus is in the public repository since at least commit `500422c`
(11 November 2026) under its current path. Exclusion stops distribution at HEAD
only: earlier commits, forks and clones keep the text. Removing it from history
would need a history rewrite, which is not authorized. No project licence
(MPL-2.0) is claimed for any wiki text.

Issue #283 remains open until all of the following hold:

1. W001473 is resolved under the project owner's decision of 19 November 2026:
   it is excluded from the active corpus (PR #397) and the decision is recorded
   as ADR-W001473-PROVENANCE-20261119 (PR #395). Without a verifiable original
   grant, exclusion is the disposition; no reuse right is assumed.
2. The 27 other imported records and the 5 third-party records are excluded
   (PRs #399 and #400), or each has a recorded owner exception.
3. Every one of the 1,298 uncertain records has a recorded disposition, and the
   resulting measures are merged.

`docs/PROJECT-STATE.md` is lead-owned and is aligned with this condition when
these PRs are accepted. The product score is unchanged.
