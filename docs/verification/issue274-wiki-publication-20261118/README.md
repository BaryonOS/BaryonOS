# English Wiki concept publication — 18 November 2026

The existing `docs/wiki/` source set links thirteen explanatory English concept pages that were absent from the separate live Wiki at `ff7a4748a0b61bfb5de8598344464ac94080bc4d`. This change prepares those existing pages for publication, adds their navigation to the existing English Home and sidebar, and uses the canonical `BaryonOS/BaryonOS` owner URLs. Licensing pointers use the current `LICENSING.md`. Earlier source versions remain available in Git history.

The six existing English guides, six Dutch-title compatibility redirects and footer remain byte-identical. Every original line of the live Home and sidebar remains in order. No existing Wiki is replaced. Changeable product status and acceptance remain in the repository checkpoint and fixed criterion register; this slice adds no product acceptance points. The current main source at preparation is `67d5859d735836331b6fb971eae89162b7fa8465`.

The [publication record](publication.json) identifies the fifteen exact prepared source files, original hashes, existing-page preservation, raw-log digests and publication state. A missing `wiki_after` means publication is pending; preparation is not a claim of a live update. The source/publication comparison can be reproduced from the repository root with:

```sh
python3 -B docs/verification/issue274-wiki-publication-20261118/verify.py --wiki /absolute/path/to/BaryonOS.wiki
```

The read-only verifier checks original concept-source equivalence after the documented URL/licensing-pointer edits, byte equality with the prepared Wiki files, all 171 referenced local targets, the preserved legacy README fragment, unchanged existing guides/redirects/footer, and original-line preservation for Home/sidebar. Two negative checks reject parent-directory and absolute path escapes. This is a bounded source/navigation check, not an automatic determination of language quality or a full-site browser audit.

| Check | Result |
| --- | --- |
| Source/publication preservation and local targets | exit 0; 15 source files, 13 new concept pages, 13 existing files byte-identical, 171 local targets and one compatibility fragment |
| Path escape negative controls | exit 0; both attempted escapes rejected |
| Strict preflight in the existing trusted environment | exit 0 |
| Standard build | exit 0; existing unused `serial::hex` warning retained |
| Standard smoke | exit 0; QEMU raw 33 and `M1:PASS` |
| Existing progress calculation/reference check | exit 0; 41/100 unchanged |
| Repository and Wiki whitespace | exit 0 |

The six live Discussion categories have English names and descriptions; their current GraphQL connection reports `hasNextPage: false`. Their IDs and slugs were not changed. The existing live Wiki was already in English and already referred to canonical status sources; the older documentation triage is historical rather than evidence of a current stale-score defect.

Issue #274 remains open. Complete current first-party navigation/language coverage, user-visible UI/runtime localisation acceptance and a maintained full documentation coverage inventory remain outside this publication slice. Literal UI strings, fixtures, source archives, historical evidence, vendor material and the OSDev corpus are untouched. Generic M1 smoke supplies no new native acceptance for a documented feature. Independent review and mandatory current-head hosted checks remain separate from these local results.
