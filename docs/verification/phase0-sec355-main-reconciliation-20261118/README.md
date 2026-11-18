# PR #355 current-main reconciliation — 18 November 2026

This checkpoint reconciles PR #355 source
`2cb7d9366425ee1c43ac5857be2101b43b4dcb67` with verified main
`1c10cdb5b30df2f3d30bea74430383f59c398931`. The tested merge is
`fa4895daf92eaf48bf7b9ecd07751d50f5537490`, tree
`d36212c7e3e98db85cd8d7760a40cab716817430`.

All PR #359 containment calls remain before issue selection, every transaction
ingress and CLI construction. PR #355 resource/OOM source is unchanged. Test-only
patches bypass the hold solely to reach lower authorization, revocation, publication
and opt-in transaction contracts; separate quarantine tests prove the production
hold. The committed manifest remains disabled.

The complete tools suite passes 474 tests with ten documented environment-gated
skips. Strict preflight and the current `qemu-smoke` workload set pass locally:
M1, M6c, DocumentSession, the native quantum-job interface, and normal plus denied
SDK applications. Each executed guest returns the expected raw exit 33.

The exact tested source also passed Rust tests, normal and M6c builds, and M1/M6c
QEMU inside a real cgroup-v2 scope with 6 GiB high, 8 GiB maximum, zero swap,
512 tasks and 400% CPU. Peak memory was 699,133,952 bytes, with zero high, max,
OOM or OOM-kill events. The 64 MiB allocation profile remains an intentional
negative enforcement probe, not a development profile.

Raw logs remain under ignored `build/tokenguard/` and
`build/sec355-reconcile-resource/`; their hashes are in [checks.json](checks.json).
The private opt-in offline fixture was not present. Local coverage/actionlint tooling
was also unavailable; hosted exact-head `host-quality`, CodeQL and review remain
mandatory. Authenticated model clients, aggregate scratch/staging/export, complete
selected gate compatibility and final Daybreak acceptance remain open.
