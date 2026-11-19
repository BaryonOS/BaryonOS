# Source-specific English assessment entrypoints — 18 November 2026

The [case manifest](sources.json) maps all thirty-four records identified in the earlier language inventory. The sixteen completeness records explicitly assess `380111b`; the coordinating multidisciplinary report and sixteen specialist records explicitly assess `beafdedf`. Their read-only methods, proposal status and statements preserving normative release criteria establish their reporting roles. Classification does not rely solely on their dates.

The completeness and multidisciplinary English guides provide source-specific summaries and reading entrypoints. They preserve detailed original dossiers, which are not represented as full line-by-line translations. Session-route research has a full historical English equivalent, retaining its source identifiers, URLs, date, proposed acceptance conditions and 41/100 boundary. All thirty-four original hashes remain unchanged. Current normative contracts, accepted decisions and checkpoint remain active.

The source ambiguity in the original release-profile opening is explicitly recorded: it uses `wijzigen` without a negation while the introduction/map preserve all criteria. The original wording remains unchanged; the English guide adopts no weakened release gate.

The [new snapshot](coverage.json) records 396 first-party Markdown documents and 179 normal-navigation documents at `221d0322fe4a53070eec72982d8382f4ee1cf47f`. It does not overwrite or retag the earlier 392-document snapshot. Reproduction uses:

```sh
python3 -B docs/verification/issue274-language-coverage-20261118/check.py --inventory docs/verification/issue274-archive-entrypoints-20261118/coverage.json
python3 -B docs/verification/issue274-language-coverage-20261118/check.py --inventory docs/verification/issue274-archive-entrypoints-20261118/coverage.json --self-test
```

The checker retains the thirty-three detailed Dutch originals as such while separately reporting thirty-four available English entrypoints. Semantic acceptance of the summaries requires independent final review. Proper UI/runtime localisation remains open and untouched; issue #274 is not closed. Progress remains 41/100.

Independent review also identified two P2 helper defects in the earlier checkpoint: default Git path quoting could omit Unicode/special filenames, and the collector could read a tracked escaping symlink from the working tree. The corrected helpers use NUL-delimited UTF-8 paths; the collector reads regular immutable Git blobs and rejects nonregular sources. POSIX fixture controls cover composed/decomposed Unicode, tab, LF, CR, quotes and backslashes in index/tree paths, plus a synthetic escaping symlink. Git metadata reads explicitly disable lazy fetching. The [helper-fix receipt](helper-fix.json) records exact tool hashes, raw results and the unchanged older snapshot's reproduction. No private outside-file content or real user file is used in those tests.

Final integration must create a new reviewed snapshot for the combined candidate's ten Rust-guidance changes, two A11Y documents and new verification entrypoints. The changed-source/scope guard remains intact; this branch's snapshot is not silently relabelled as that integrated result.
