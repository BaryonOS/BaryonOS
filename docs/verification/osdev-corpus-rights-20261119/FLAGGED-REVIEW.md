# Review of screening-flagged records — 19 November 2026

Individual review of the 149 records flagged by text screening in classes A–C
(the four flagged class D records are excluded with their class). For each
record, up to two matching passages were read with about 60 characters of context
on each side; a record with further unread matches may need a second pass. Categories are review outcomes, not legal
findings.

## 1. Third-party text with unknown terms — exclude (5)

| Record | Title | Included third-party material |
|---|---|---|
| W001553 | Text UI | Example console class "taken from Brandon's Kernel Tutorial" |
| W001845 | Shutdown | Quoted forum post; W002031 states forum posts are owned by their authors |
| W002332 | ARM Overview | Table "copied from the ARM7TDI datasheet" |
| W003703 | User:Combuster/Alloca | Quoted Usenet post by a named third party |
| W003774 | CPU Registers x86-64 | Table taken from wiki.jwo.cz, licence not stated |

These pages are excluded with `scripts/exclude_sources.py` in a separate scoped PR.

## 2. Third-party material under a verifiable licence with obligations — owner decision (5)

| Record | Title | Material and licence |
|---|---|---|
| W004323 | User:Janworks/Intel Ethernet i219-V | Register definitions the page says were taken from the Linux e1000e driver (Linux is distributed under GPL-2.0) |
| W004744 | User:Isaccbarker/String Formatting | Code the page says comes from HenipaOS, with a BSD-3-Clause variant SPDX identifier |
| W002279 | Detecting Memory (x86) | Structure definition the page says is taken from GNU EFI |
| W001545 | Interrupt Descriptor Table | Table the page says was copied from wiki.jwo.cz, which cites a Wikipedia template revision |
| W004920 | Template:X86 protected mode interrupts | Same table as W001545 |

Source of each determination: the statement in the record itself, at the
revision recorded in `.agents/skills/os-dev/references/source-index.json`
(`revision_url`). The upstream licences (Linux, HenipaOS, GNU EFI, wiki.jwo.cz,
Wikipedia) were **not** fetched or verified from this environment; "verifiable"
means the page names a licence or licensed source that can be checked, not that
it was checked. If confirmed, the terms carry notice, attribution or share-alike
obligations that the CC0-based corpus statement does not meet. Option: keep with
explicit attribution and licence notes, or exclude.

## 3. Short quotations from specifications and manuals — owner decision (12)

W001852, W002246, W002603, W002803, W003231, W003378, W004215, W004362,
W004461, W004520, W004615, W004993. Each quotes a few sentences from a vendor
manual, a specification or a product manual with attribution. Whether a
quotation exception applies depends on jurisdiction and purpose; this is a legal
question and is escalated, not decided.

## 4. No third-party text identified (127)

The remaining flagged records only mention licences as subject matter (for
example "released under the GPL"), contain the author's own code with their own
copyright string, quote trivial program banners, describe images that are not in
the corpus (`File:` pages; the image files themselves are not included), or
discuss copyright on talk pages. This covers only the passages read; the
records keep their **uncertain** status, including the class A records among
them, until their individual review is complete.

## Limits

Unflagged records in classes B and C were not read individually. Two risks stay
unresolved for them: older wiki text under unclear terms, and embedded
third-party material that pattern screening cannot rule out. Their individual
review must cover both. Establishing rights needs the contributors' grants, a
verifiable licence for any third-party material, or an independent replacement,
which this environment cannot obtain.
