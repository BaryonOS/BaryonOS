# S09 requests to the integration coordinator (24 November 2026)

Source: owner decision packages 1 and 2 of 24 November 2026, recorded in
`docs/implementation/S09-DESTINATION-SERVICE.md` (header note and 8.1). S
cannot make these changes itself: they touch protected or lead-owned files.
Each request states the interface, the change, the acceptance and the
dependencies. S stays available for review of each change.

## C1 — ADR for the owner decisions (lead-owned `docs/DECISIONS.md`)

- **Change:** add at the top of `docs/DECISIONS.md`:
  `ADR-S09-DESTINATION-DECISIONS-20261124 — Owner decisions O1–O13 for the
  S09 destination service`. Status: decided by the project owner on 24
  November 2026 in two packages; amends ADR-S09-DUALDISK-20261119 B3 and B5
  for the R5 test profile only (destination service holds the whole disk 1
  with READ|WRITE|FLUSH, no DERIVE, TRANSFER or REVOKE; a scripted test
  dialog is permitted; B4 and B6 unchanged). Decision: as recorded in 8.1 of
  the specification, items O1–O13; O14–O18, D1–D6 and the restore-fidelity
  decisions unchanged. Consequences: a design decision is not test evidence;
  S09 stays open; R5 needs K5, the hosted R3 checks and its native stages;
  R8 needs the O12 owner-binding design, the O13 kernel latch with its own
  native test, and native evidence of the entropy route.
- **Acceptance:** the ADR text matches 8.1; ADR-S09-DUALDISK carries a
  pointer to the amendment.
- **Dependency:** none.

## C2 — Hosted R3 cross-check and reverse differential (O6; R5 precondition)

- **Interface:** both directions exist on the host (PR #509):
  - forward: `DEST_ORACLE_OUT=<dir> cargo +1.100.0 test --locked -p
    destination-core --test oracle_images`, then `DEST_ORACLE_OUT=<dir>
    python3 -B -m unittest tests.test_volume_destination_disk.RustCrossCheck`;
  - reverse: `DEST_REVERSE_OUT=<dir> python3 -B -m unittest
    tests.test_volume_destination_disk.ReverseDifferential`, then
    `DEST_REVERSE_IN=<dir> cargo +1.100.0 test --locked -p destination-core
    --test reverse_differential`.
  `<dir>` must be an absolute path. Without the variables both Rust tests
  pass without reading anything, so the order above is mandatory.
- **Change:** one step in `ci.yml#host-quality` (or the new destination gate
  of C4) that runs the four commands in that order with fresh directories.
- **Acceptance:** the hosted log shows 30 forward images (RustCrossCheck OK,
  not skipped) and 182 reverse images (`reverse_differential ... ok` with
  `DEST_REVERSE_IN` set); a skipped test is a failed step.
- **Dependency:** PR #509 merged.

## C3 — P10: lint of the `no_std` library as a native profile links it

- **Change:** add to a hosted gate:
  `cargo +1.100.0 clippy --locked -p destination-core --lib
  --no-default-features --target x86_64-unknown-none -- -D warnings`.
- **Acceptance:** the step runs and exits 0. Local run on `7019a1f6`: exit 0.
- **Dependency:** none.

## C4 — Separate destination acceptance gate (O6)

- **Change:** a separately named workflow job for the native destination
  stages of R5 (`dest-format`, `dest-export`, `dest-export-2`,
  `dest-restore`, `dest-grant-once`) and later stages, beside the existing
  volume regression tests, which stay unchanged. Stage-count expectations
  come from the stage plan in the runner, not from historical counts.
- **Acceptance:** the job exists and fails when a stage is missing or
  skipped; the volume gate's own pins stay.
- **Dependency:** the R5 stages, which do not exist yet. S will supply the
  runner and the stage plan first; this request is advance notice.

## C5 — K5 per-device broker state (shared kernel interface)

- **Interface:** `kernel/src/m2/m3/volume.rs` broker state (`Policy`,
  reset state, reset deadline, error state, active request, data buffer),
  spec 2.7 broker contract and 4.3 K5, including property 3 (a further poll
  after a read ended by a reset returns -1).
- **Change:** S prepares the K5 design and a small first kernel step on its
  branch; the coordinator decides on routing and review because the broker
  is shared with the volume service.
- **Acceptance:** spec 2.7 properties per device; disk-0 progress during a
  disk-1 timeout shown by a native fixture; existing volume stages pass.
- **Dependency:** owner approval O4 (a), given on 24 November 2026.
