# PR #355 bounded gate finalizer — 18 November 2026

This checkpoint binds the gate-finalizer milestone to commit
`364b94964cc7f3fbb8ff2377e87861e858ef03ad`, parent
`467e25b6394fcffc97d85a7be025bd5814b0806e`.

The controller can now represent one gate job with one total deadline and at most
one reserved finalizer. Ordinary steps cannot consume the finalizer reserve, and
the finalizer itself is capped to that reserve. It runs after ordinary non-zero
results, verified OOM/resource results, cleaned timeouts and bounded-output
failures. A generic sandbox, guard, termination or cleanup failure aborts without
starting another process. Primary and finalizer streams use distinct indexed files,
and a finalizer failure cannot replace the primary failure status.

The focused controller suite and complete tools suite pass on the exact commit.
Independent security review initially found three controller defects and then a
cleanup-boundary defect; all four received regression tests before the final PASS.
Commands, raw-log hashes and evidence limits are in [checks.json](checks.json).

This is controller semantics only. The current `ci.yml#qemu-smoke` definition still
fails closed at its first unmodelled action. The private rootfs, offline Cargo cache
and tool receipts were not transferred, so no runtime layout was invented and no
partial `qemu-smoke` acceptance is claimed. The manifest and execution hold remain
disabled.
