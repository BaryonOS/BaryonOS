# PR #355 immutable source receipt — 18 November 2026

This checkpoint binds the source-receipt milestone to commit
`b39948e4355b789faeffa9c92a92c1a443dd3480`, parent
`ef2db95f42f3883a3f85e5107971e2129291451f`.

The controller hashes exactly the selected regular source files before staging,
after staging and in the staged tree. It separately verifies HEAD before and after
the copy. A fixed `.baryonos-source-receipt.json` is then written with mode `0444`
and the staged workspace is mounted read-only. The receipt accepts only schema 1,
one lowercase 40-hex commit and the current v3 source hash. Redirected paths,
symbolic or hard links, special or oversized files, any write bit, duplicate or
unknown keys, source drift and commit drift are rejected before sandbox execution.

M6c and DocumentSession use the receipt in isolation while retaining their existing
Git fallback for ordinary host development. On the exact checkpoint commit, M6c
rebuilds and passes QEMU with raw exit 33. The DocumentSession normal/cold-recovery
gate passes two real VM runs and nine source-freeze checks. The security and complete
tools suites, applicable host tests, syntax compilation and strict preflight also
pass. Commands, log hashes and evidence limits are in [checks.json](checks.json).

This checkpoint removes the sandbox `.git` dependency only. It does not provide the
pinned offline runtime/cache, aggregate scratch limit, total workflow deadline,
`always()` evidence finalizer or authenticated client. The manifest and execution
hold remain disabled, and hosted exact-head checks remain mandatory.
