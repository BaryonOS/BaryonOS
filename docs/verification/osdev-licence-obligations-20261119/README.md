# Upstream licence evidence for five OSDev records with licence obligations (#283)

Date: 19 November 2026 (Europe/Amsterdam). Scope: the five records that the
rights assessment
([`osdev-corpus-rights-20261119/FLAGGED-REVIEW.md`](../osdev-corpus-rights-20261119/FLAGGED-REVIEW.md),
category 2) lists as containing material from a named upstream source with
notice, attribution or share-alike obligations. This record collects the
upstream licence evidence that the assessment could not fetch. **It does not
decide whether any record is kept, changed or excluded, and it does not
establish a reuse right.** The disposition remains a project-owner decision
under ADR-W001473-PROVENANCE-20261119 (retain only on a verifiable licence,
original permission or an independently written replacement). The corpus is
not changed.

## Method

For each record, the statement in the record text (at the revision recorded in
`.agents/skills/os-dev/references/source-index.json`) was read, the named
upstream source was fetched where reachable, and the overlap between the record
and the upstream text was measured where the upstream text is code. Fetched
files are identified by SHA-256 in [`evidence.json`](evidence.json); they are
not committed. `wiki.osdev.org` and `web.archive.org` were not reachable from
this environment, so the OSDev revisions themselves were not re-fetched.

## Findings

| Record | Upstream named in the record | Upstream licence found | Overlap with upstream | Status |
|---|---|---|---|---|
| W004323 *User:Janworks/Intel Ethernet i219-V* | Intel's Linux `e1000e` driver, used to "deduce the needed information" | `SPDX-License-Identifier: GPL-2.0`, Copyright 1999–2018 Intel (`drivers/net/ethernet/intel/e1000e/regs.h`) | 15 code blocks (18,835 characters). No `#define` line is copied. Six lines of at least 25 characters occur verbatim in nine `e1000e` source files: two code statements (`rctl &= ~E1000_RCTL_SZ_4096;`, `rctl &= ~E1000_RCTL_BSEX;`) and four comments. Register names and values are facts from Intel's hardware interface. | Overlap is small; the two copied buffer-size comments show partial derivation from GPL-2.0 code. Uncertain. |
| W004744 *User:Isaccbarker/String Formatting* | HenipaOS; the record states the code "is thus licensed under SPDX: BSD-3-Clause-No-Nuclear-License" | The identifier exists in the SPDX License List (`BSD-3-Clause-No-Nuclear-License`, not OSI-approved). The HenipaOS source was not located (no public GitHub repository matched), so the stated licence could not be checked against the original. | Not measurable without the upstream source. | The licence adds a use restriction (no nuclear facilities) and requires the notice to be kept; it is incompatible with CC0 and with an unrestricted corpus. Uncertain. |
| W002279 *Detecting Memory (x86)* | GNU EFI; the `EFI_MEMORY_DESCRIPTOR` structure is "taken from GNU EFI" | GNU EFI `README.efilib`: files in `lib` and `inc` derive from Intel's EFI Application Toolkit, Copyright 1998–2000 Intel, under a two-condition BSD-style licence (retain the copyright notice and conditions). `inc/efidef.h` carries `Copyright (c) 1998 Intel Corporation`. | One structure (six fields with comments). The field layout is fixed by the UEFI specification; the comments match `efidef.h`. | Notice obligation applies to the copied text; an independent description from the UEFI specification would avoid it. Uncertain. |
| W001545 *Interrupt Descriptor Table* | A table "originally created for Wikipedia", Wikipedia template revision 1283313564, copied via `wiki.jwo.cz` | Wikipedia site licence: Creative Commons Attribution-ShareAlike 4.0 (MediaWiki `siteinfo`). Revision 1283313564 of *Template:X86 protected mode interrupts*: 31 March 2025, 5,418 bytes. | The table, as stated. | CC BY-SA 4.0 requires attribution and share-alike; incompatible with CC0 and with MPL-2.0 relicensing. Attribution may be given by linking the source page (Wikimedia Terms of Use, section 7, as cited in review; not re-fetched here), so a collected contributor list is not a prerequisite. Uncertain. |
| W004920 *Template:X86 protected mode interrupts* | Same table as W001545 | As W001545 | As W001545 | As W001545. Uncertain. |

Evidence gaps, kept separate: (1) the Wikipedia contributor list up to revision
1283313564 was not collected (the history query was refused during this session);
(2) the intermediate `wiki.jwo.cz` copy was not verified (unreachable). Upstream
files were fetched from mutable branch URLs; their content is identified by
SHA-256 in `evidence.json`, but the immutable upstream commit was not recorded.

## Options for the project owner (not decided here)

1. **Exclude** the record through the existing exclusion script and register,
   as for #397–#400.
2. **Retain with the upstream licence**: keep the text with its notice and
   licence (GPL-2.0, BSD-3-Clause-No-Nuclear-License, the Intel BSD-style
   notice, CC BY-SA 4.0 with attribution), marked as not covered by the CC0
   statement. This requires a licence-compatibility decision for the corpus.
3. **Replace** the copied fragment with an independently written description
   from the primary specification (Intel SDM, UEFI specification, Intel
   datasheets), with references, and keep the rest of the page under the CC0
   basis.

For W004323 and W002279 the copied fragment is small enough that option 3
removes the obligation without losing technical content. For W004744 the
licence's use restriction makes option 2 the least attractive. For W001545 and
W004920 the table is the core of W004920; option 3 would mean rewriting the
table from the Intel SDM.

## Limits

This is a technical provenance record, not legal advice. Overlap measurement
covers only the files listed in `evidence.json` and only verbatim lines of at
least 25 characters; paraphrase is not detected. No licence is claimed for any
record, and #283 stays open.
