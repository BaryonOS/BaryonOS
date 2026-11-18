# PR #355 fail-closed gate resolution — 18 November 2026

This checkpoint binds the gate-resolver correction to source
`b35433d27e07de2e55e5703d3186a4ab5ba2b78a`, whose parent is the published PR
#355 reconciliation head `a1e12249b09c5cab94ccf2a27747116cd15dbb76`.

The prior resolver silently removed GitHub Actions `uses`, conditional, setup and
commandless steps and could therefore present an incomplete local job as the selected
gate. The corrected resolver blocks each unsupported category before returning any
command list. `agent_pipeline` converts the refusal to `TaskError`; `roadmap_runner`
records exit 2 and a hashed `gate.log`. Both current `ci.yml#host-quality` and
`ci.yml#qemu-smoke` definitions are negative-tested and block at their first
unmodelled checkout action.

The combined security suite passes 114 tests with ten environment-gated skips. The
complete tools suite passes 477 tests with the same ten skips. Independent focused
verification and security rereview found no blocking issue in this change. Exact
commands, raw-log hashes and evidence limits are recorded in
[checks.json](checks.json).

This is an interim fail-closed correction, not complete selected-gate compatibility.
Job-level timeouts, `always()` finalizers, immutable source receipts, pinned offline
tool/runtime inputs and all other explicit adapter semantics remain open. The pipeline
manifest and unconditional execution hold remain disabled. Native/QEMU workloads
were not repeated for this Python controller-only checkpoint; the parent receipt
remains historical evidence and hosted exact-head acceptance is still required.
