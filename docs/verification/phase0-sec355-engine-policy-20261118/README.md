# PR #355 owner-bound engine policy — 18 November 2026

This checkpoint binds the engine-policy milestone to commit
`47835c4ebc5dad0576c0feaa3ec3654f3679f14d`, parent
`79d8674324d14d0b7608fb15f19701f33ddc939c`.

Manifest schema 2 contains the exact reviewed engine set and failover order. The
pipeline rejects any reordered, narrowed or expanded CLI selection before controller
verification, Git, sandbox or model-client access. Retries retain that order instead
of rotating it. Standalone gates and the pipeline use one validator for schema,
enablement, non-empty known string names and uniqueness.

Focused and complete tools suites pass on the exact commit. Independent security
review found that the first implementation allowed standalone gates to accept a
schema-2 manifest which the pipeline rejected. The shared validator and a negative
regression corrected that path; rereview reports no actionable finding. Commands,
raw-log hashes and evidence limits are recorded in [checks.json](checks.json).

The checked-in manifest remains disabled and the source execution hold remains
unconditional. This checkpoint does not supply the missing private authenticated
model-client/runtime contract, complete selected-gate adapter or aggregate
scratch/staging/export filesystem cap, and it does not resolve the Daybreak finding.
