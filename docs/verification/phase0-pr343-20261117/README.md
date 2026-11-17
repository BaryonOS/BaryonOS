# PR343 — bounded artifact-action contract

The changed download action previously ran only in the manual full-matrix aggregate. Ordinary PR CI did not execute it. A fixture-only composite now uploads two matching and one excluded ZIP artifacts with the existing v4.6.2 pin, downloads with exactly the aggregate pin/inputs, and rejects missing, extra, misplaced or modified files. This job is excluded from every workflow_dispatch; fixture artifacts cannot contaminate the full matrix. Existing native jobs, permissions, digest defaults and historical evidence checks are unchanged.

The host regression exercises valid and four invalid layouts. The installed, checksum-verified actionlint and existing workflow/evidence checks pass. A complete tools run initially failed because PATH selected snap launchers inside this environment; a rerun selects the already installed rustup binary, without disabling sandboxing or installing a toolchain. The failed log is retained.

[Exact source hashes, commands and raw-log hashes](checks.json). Independent AI review is not a human audit. Hosted action execution and required CI on the published head remain pending; no merge or new native criterion is accepted. The unchanged 74x50 matrix is NOT RUN for this action-only correction.
