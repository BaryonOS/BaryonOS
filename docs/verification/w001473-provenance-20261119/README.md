# W001473 provenance record — 19 November 2026

Scope: issue #283 under
[ADR-W001473-PROVENANCE-20261119](../../DECISIONS.md#adr-w001473-provenance-20261119--disposition-route-for-historical-source-record-w001473-issue-283).
This record is evidence for a project-owner decision. It is not a legal
assessment, grants no clearance and changes no source or licence file. The
protected reference corpus under `.agents/skills/os-dev/references/` is
unchanged.

## Evidence found

All facts below come from the repository's own copy of the MediaWiki export
metadata, `.agents/skills/os-dev/assets/wiki-revision-metadata.jsonl.gz`
(SHA-256 recorded in [provenance.json](provenance.json)), and from the
`source-index.json` entry of W001473.

| Item | Finding |
|---|---|
| Page | "Monotasking Systems", page ID 1473 |
| Included text | revision 11215 of 6 March 2011; text SHA-256 `55bd6aad…4c60d`, 1,468 bytes |
| First revision | 1773 of 3 December 2006 by `Telexicon`, comment **"Ported from the old OS Wiki"** |
| Later revisions | six edits by six other accounts between December 2006 and March 2011 |
| Original author | **not established**: the text originates on an earlier wiki whose author and terms are not in the export |
| Licence statements in the corpus | W003005 (OSDev Wiki:Copyrights) licenses content "added since June 6 of 2011" under CC0. W002031 (OSDev Wiki:License) requires public-domain compatibility for new content "as from 03-07-2011" and calls the wiki "of mixed licensing". Both pages are retained as written; they are not combined into a single claim |

**Conclusion:** every revision of the included text (latest: 6 March 2011)
predates the CC0 start date of 6 June 2011 and either reading of "03-07-2011", and its first revision is an explicit port from a third-party
wiki. No grant of reuse permission for the original text is established. This
is an unverified-provenance finding, not a claim of infringement.

## Primary sources that could not be reached

On 19 November 2026 automated requests from this session to `wiki.osdev.org`
were refused by a bot challenge (HTTP 403). The Internet Archive was blocked by
this environment's egress policy. No challenge was circumvented. The original
"old OS Wiki" (OSFAQ) page history, its author and its terms therefore remain
unverified. A person with ordinary browser access can check the page history
and the OSFAQ archive, or ask the wiki administrators directly.

## Class scope (publication impact)

W001473 is not unique. In the same export:

- 28 of 3,054 included wiki pages carry an explicit import comment
  ("Ported from the old OS Wiki", "rough import from OSFAQ" or
  "import to MediaWiki"); they are listed in [provenance.json](provenance.json);
- 641 pages have only revisions dated before 6 June 2011.

The repository is public, so the corpus is published. A disposition for
W001473 alone would leave the same provenance class in place for the other
pages. Whether the decision should cover only W001473, the 28 imported pages
or all pre-CC0 pages is a **legal and publication decision for the project
owner**; it is escalated, not decided here.

## Disposition options prepared

1. **Narrow exclusion.** Remove the W001473 excerpt (1,468 bytes) from
   `references/corpus/wiki-01.md` and its entries in the index files, recompute
   the later offsets in `source-index.json`, and keep this record as the reason.
   The exclusion must persist: a corpus rebuild has to skip excluded records
   (implemented in PR #397). Git history is not rewritten, so the text remains in
   earlier commits; the exclusion only stops further distribution at HEAD.
2. **Independently authored replacement.** Replace the excerpt with the text
   below, written for this project without reference to the wording of the
   original, and mark it as project-authored (MPL-2.0) rather than as a wiki
   source.
3. **Retain pending evidence.** Keep the excerpt unchanged while the original
   OSFAQ author and terms are sought by a person with browser access.

Candidate replacement text (project-authored; unreviewed draft — its technical
statements need primary-source references before it is used):

> A monotasking (single-tasking) operating system runs one program at a time.
> The running program effectively controls the machine; a small resident part of
> the system remains to service requests and to resume control when the program
> ends. Some designs keep several programs loaded but let only one interact with
> the user. Such systems are simpler to build but offer little isolation between
> programs, so general-purpose systems today are multitasking.

Each option touches a protected path (`.agents/skills/os-dev/references/`) and
therefore needs an explicitly scoped, reviewed PR after the owner selects it.
Issue #283 remains open.
