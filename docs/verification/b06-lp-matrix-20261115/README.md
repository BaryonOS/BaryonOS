# LP evidence reference matrix

Created 18 November 2026 in the task-named directory. Run
`python3 -B tools/lp_matrix.py --check` to validate the 31 contract identifiers,
reference paths/digests and residual coverage. The JSON output publishes the
remaining requirement for every row. A green exit validates structure, not privacy
compliance, test authenticity or complete scenario coverage.

`none` means no complete-scenario evidence has been mapped by this change. It does
not mean the feature is absent or discard previous partial evidence. No historical
record was re-audited and no AI, NAS or provider row receives native acceptance.
The fixed criterion register remains authoritative. Mapping evidence requires
review of its scope and source against the complete contract scenario.

A `host` reference must be an existing bounded repository file with its SHA-256.
A `native` reference additionally requires an archived `docs/verification/` JSON
record using the existing system-runner fields: `passed: true`, `timed_out: false`,
`raw_qemu_exit: 33` and a 64-character source fingerprint. Other evidence formats
remain unmapped until explicitly supported; a README or host result is insufficient.
These structural fields and a digest do not establish record authenticity or
prove the mapped LP scenario. Every row retains explicit residual coverage.
