# Accessibility matrix successor checker — 18 November 2026

Executable source: `a2792a2c1ee8e8eeb604549ac8a04272ed70168a`, based on PR #339
head `5e222780c916205bb9aa4e54ba9ccaa983ce61e7`. This checkpoint repairs two
structural checker gaps. It does not complete the systematic literature review,
validate research quality, establish UI conformance or accept a native criterion.

`python3 -B tools/check_a11y_matrix.py <repository-root>` reuses the archived
parser and structural validator only after verifying the original checker's exact
SHA-256. The verified bytes are compiled directly; an altered archive cannot run.
The successor does not call the archived named-file validator. The historical
checker, matrix, research and evidence archives remain unchanged.

Named files in prototype and native status cells must resolve to regular files
inside the supplied repository. Absolute paths, parent traversal and Windows path
syntax are rejected. Nested repository-relative paths are supported; bare names
retain the documented search under `ui/` and `docs/`, counting only confined files.
An internal symlink may reference an internal file; an escaping file or directory
symlink cannot supply evidence. The matrix itself must also remain inside the root.
The recognized suffixes remain CSS, HTML, JS, Rust, Markdown, Python, JSON and TOML.
Line numbers, selectors, scientific support and the contents of cited files are
not validated by this structural check.

Independent review found that the first successor candidate could truncate a
punctuated outside path into an existing inside basename. The correction retains
whole quoted locations and lexical tokens, and rejects unsafe location prefixes
before basename lookup. Sixteen added negative cases cover existing absolute and
parent paths containing `@`, `%`, spaces or parentheses, both quoted and unquoted,
while a matching inside basename exists. The regression fails on the earlier
candidate and passes after correction; the unchanged current matrix still passes.

[`baseline-contract.json`](baseline-contract.json) records the exact Need and
Adaptation identities of all 51 baseline rows in matrix SHA-256
`e12d566f069a56e1066689e598f5bfbad7ac646c6347e14da2dc94b0a5a6e6b5` at the
source checkpoint. No baseline choice is added or reclassified. Each identity must
occur exactly once, retain tier `baseline` and retain an axis starting with
`none (baseline)`. Reordering is allowed; removal, duplication, changed identity or
a consistent tier-and-axis demotion is rejected. The contract's own digest is
pinned in the checker. Future intentional identity changes require a reviewed
contract and pin update; validation never regenerates the contract from its input.

Fifteen focused tests pass, including demotion and removal of every baseline row
(102 mutation subtests), real existing outside files, file and directory symlink
escapes, valid nested paths and bare names, Windows-looking paths, an escaping
matrix, archive tampering and manifest tampering. The tests also demonstrate that
the pinned historical validator accepts the outside absolute and parent paths and
misses some consistent non-safety baseline demotions.

The complete activated tools suite passes 399 tests with no skips. Strict preflight,
the normal native build and QEMU smoke pass on the committed source; QEMU returns
33 with the required boot markers. These generic gates provide no new design or
accessibility acceptance. Python compilation and the protected-source comparison
also pass. Exact command records, raw exit statuses and local log digests are in
[`checks.json`](checks.json). Independent candidate review and current-head hosted
checks remain separate requirements. Score: 41/100, unchanged.
