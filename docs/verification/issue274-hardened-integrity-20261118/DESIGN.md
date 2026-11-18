# PR #358 hardened evidence checkpoint

This lead-owned verification child supersedes the validated failure modes in the
earlier protected issue #274 evidence without modifying those historical records.
The helpers, fixed inputs, fresh output and complete raw command records are bound
to the source identities below.

## Bound source state

- Repository commit: 4ae9a1c4f19bcf84ed06f0a8afcfdfc40fb237a8
- Repository tree: a6eb48f68f30cf443b96c7cfc9da636a2a179532
- Wiki publication source: 5eae79a2bb6a5b1d655de16c7a293be96a07a9a6
- Wiki baseline: ff7a4748a0b61bfb5de8598344464ac94080bc4d
- Wiki publication: 4a01b8b2ad2695da509a512ea688007c964f56da
- Wiki publication tree: f7bee30ee39abb60da294d48357af5b6c85f2fae
- Runtime: Git 2.53.0, Python 3.14.4, Node 22.22.1, marked 18.2.1.
- marked was reused from the existing isolated sec356-helper-fix Node cache;
  no dependency was installed.

The repository and Wiki references must be complete 40-character commit IDs.
Every source read uses the corresponding Git tree and regular blob. Git object,
repository, worktree, index, config and replacement-ref redirections are removed
from child-process environments.

## Helper design

- collect.cjs creates a repository/Wiki tree inventory. It uses the marked lexer,
  validates GitHub-style collision anchors and Wiki fragments, rejects nonregular
  blobs, bounds percent-decoding errors, discovers tracked Markdown and YAML
  community forms, and records extracted and locally classified link counts
  separately.
- check.py validates canonical evidence inputs and their SHA-256 manifest, then
  reruns collect.cjs against the exact repository and Wiki commits and compares
  all semantic output except the generation timestamp. It therefore reproduces
  counts, scope, reachability, protected exclusions, link findings and Wiki claims.
  The manifest requires named publication-record and published-receipt roles in
  addition to the inventory, collector output, review and complete helper set.
- verify.py validates the exact repository and Wiki snapshots without Python
  assertions. It checks baseline ancestry, the complete before/after Wiki page
  domain, exact before hashes, preservation, publication receipts, source
  transformations, local targets and fence-aware fragments.
- self_test.py builds disposable Git repositories beneath this evidence child.
  NFC/NFD names are inserted directly into the Git index so the control does not
  depend on filesystem Unicode normalization.

## Finding-to-control map

| Finding | Corrected mechanism | Positive or negative control |
| --- | --- | --- |
| 1 mutable repository target reads | exact source tree and blobs | immutable_git_tree_and_replacement_refs; checker_after_worktree_mutation exit 0 |
| 2 inherited Git redirection | stripped Git environment | inherited_git_redirection |
| 3 unchecked Wiki fragments | same-blob anchor validation outside fenced/inline code | wiki_fragment_duplicate_anchor and wiki_invalid_fragment, including hidden explicit anchors |
| 4 malformed percent sequences | strict bounded decoder | malformed_percent_encoding and strict_percent_decoder |
| 5 optimized-away assertions | explicit require errors; no Assert AST node | python_optimize_integrity_negative; tampered verifier exit 1 under -O |
| 6 nonregular Wiki targets | regular Git blob requirement | wiki_nonregular_blob and verifier_wiki_regular_blob |
| 7 duplicate anchor collision | used-anchor suffix loop | fence_aware_duplicate_anchors; A, A, A-1 yields a, a-1, a-1-1 |
| 8 replacement refs | GIT_NO_REPLACE_OBJECTS | immutable_git_tree_and_replacement_refs |
| 9 Unicode fixture portability | direct index entries | unicode_nfc_nfd_tree_entries |
| 10 noncanonical inventory | explicit canonical regular path | canonical_inventory_path; symlink negative exit 1 |
| 11 YAML form drift | exact issue/discussion YAML tree inventory | yaml_form_inventory and yaml_form_drift_negative exit 1 |
| 12 fence parser | character, length, indentation and info legality | python_fence_character_and_length and illegal_fence_openers |
| 13 unbound manifest | required roles and SHA-256 member set | manifest_sha_binding and manifest_required_wiki_roles; omission/tamper exits 1 |
| 14 mislabeled historical run | exact current commit in command/output | live collector and checker logs |
| 15 overloaded local-link count | extracted and classified fields | split_link_counts; live 7658 versus 5697 |
| 16 mutable Wiki input | exact Wiki commit/tree blobs | live collector output and full_sha_required |
| 17 unpinned primary reproduction | required full repository SHA | mutable_ref_negative exit 1 |
| 18 missing prerequisites | versions and marked module path recorded | this document and command logs |
| 19 overstated fence exclusion | CommonMark controls and bounded method text | illegal_fence_openers and fence-aware fragment control |
| 20 unpinned archive reproduction | new current-tree evidence supersedes, not rewrites, old archive | exact source/tree output and manifest |
| 21 mutable Wiki publication verification | four exact source/baseline/publication identities | live Wiki verifier and optimized verifier logs |
| 22 source provenance outside PR ancestry | current reconciled commit recorded | lead must retain 4ae9a1c as an ancestor of the published sibling or regenerate |
| 23 current-head CodeQL comment elision | remove HTML comments to a fixed point before tag extraction; truncate unmatched comments | fixed_point_html_comment_elision rejects an anchor exposed by one-pass removal |

Additional reviewer controls cover complete Wiki baseline inventory, exact
before/after preservation and publication ancestry. The negative cases are
wiki_baseline_inventory_completeness_negative,
wiki_before_after_preservation_negative and wiki_baseline_ancestry_negative.
Additional semantic and ref controls mutate the recorded link count and every
verifier commit field; each subprocess exits 1 as required.

## Reproduction commands and raw exits

All paths below are the exact paths used for this verification run. Logs contain the
complete command, stdout, stderr and raw exit code.

1. Collector: logs/01-collector.json, exit 0.

~~~text
NODE_PATH=/home/simon/projects/BaryonOS-worktrees/phase0-combined-local/build/sec356-helper-fix/node/node_modules node docs/verification/issue274-hardened-integrity-20261118/helpers/collect.cjs --root /home/simon/projects/BaryonOS-worktrees/pr358-review-fixes --ref 4ae9a1c4f19bcf84ed06f0a8afcfdfc40fb237a8 --wiki /home/simon/projects/BaryonOS-worktrees/phase0-combined-local/build/issue274-wiki-readonly --wiki-ref 4a01b8b2ad2695da509a512ea688007c964f56da --review /home/simon/projects/BaryonOS-worktrees/pr358-review-fixes/docs/verification/issue274-hardened-integrity-20261118/inputs/review.json --out /home/simon/projects/BaryonOS-worktrees/pr358-review-fixes/docs/verification/issue274-hardened-integrity-20261118/output
~~~

2. Immutable checker and semantic collector reproduction:
logs/02-checker.json, exit 0.

~~~text
python3 -B docs/verification/issue274-hardened-integrity-20261118/helpers/check.py --root /home/simon/projects/BaryonOS-worktrees/pr358-review-fixes --ref 4ae9a1c4f19bcf84ed06f0a8afcfdfc40fb237a8 --inventory docs/verification/issue274-hardened-integrity-20261118/output/coverage.json --review docs/verification/issue274-hardened-integrity-20261118/inputs/review.json --manifest docs/verification/issue274-hardened-integrity-20261118/manifest.json --wiki /home/simon/projects/BaryonOS-worktrees/phase0-combined-local/build/issue274-wiki-readonly --marked-node-path /home/simon/projects/BaryonOS-worktrees/phase0-combined-local/build/sec356-helper-fix/node/node_modules
~~~

3. Exact Wiki verifier: logs/03-wiki-verifier.json, exit 0. The same command
with PYTHONOPTIMIZE=1 and python3 -O is in
logs/04-wiki-verifier-optimize.json, exit 0.

~~~text
python3 -B docs/verification/issue274-hardened-integrity-20261118/helpers/verify.py --root /home/simon/projects/BaryonOS-worktrees/pr358-review-fixes --wiki /home/simon/projects/BaryonOS-worktrees/phase0-combined-local/build/issue274-wiki-readonly --record docs/verification/issue274-hardened-integrity-20261118/inputs/publication.json --published docs/verification/issue274-hardened-integrity-20261118/inputs/published.json
~~~

4. Disposable positive/negative controls: logs/05-self-test.json, exit 0.
The log records raw exit 0 for the collector and immutable positive checks and
raw exit 1 for every subprocess negative control.

~~~text
python3 -B docs/verification/issue274-hardened-integrity-20261118/helpers/self_test.py --marked-node-path /home/simon/projects/BaryonOS-worktrees/phase0-combined-local/build/sec356-helper-fix/node/node_modules --live-root /home/simon/projects/BaryonOS-worktrees/pr358-review-fixes --live-wiki /home/simon/projects/BaryonOS-worktrees/phase0-combined-local/build/issue274-wiki-readonly --record /home/simon/projects/BaryonOS-worktrees/pr358-review-fixes/docs/verification/issue274-hardened-integrity-20261118/inputs/publication.json --published /home/simon/projects/BaryonOS-worktrees/pr358-review-fixes/docs/verification/issue274-hardened-integrity-20261118/inputs/published.json
~~~

## Results and evidence limits

The live collector reports 840 tracked Markdown paths, 398 first-party candidate
documents, 442 protected exclusions, 179 normally reachable documents, 3,007
candidate lines, 7,658 extracted links, 5,697 locally classified links, 58 Wiki
references and eight community forms. The one unresolved target is the expected
generated build/toolchain-provenance.json reference; it is not suppressed.

The live Wiki verifier checks 15 source pages, 13 byte-identically preserved
baseline pages, 171 local targets and one fragment. The 34-case self-test is a host-level
deterministic fixture, not hosted CI, native QEMU, hardware or human language/UI
acceptance. No GitHub/Wiki state was mutated. Issue #274, UI localisation,
language-quality acceptance and full issue acceptance remain open. The lead must
rerun after any source-tree or helper change and must keep the recorded repository
commit in the published PR ancestry; otherwise the output is stale.
