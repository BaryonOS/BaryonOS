# Specialist P — resume pointer

Compact resume pointer for the personal-storage specialist (P01–P04).
This file is not a backlog: tasks come from `docs/progress.json`, the roadmap
ledger, issues and pull requests. Newer ADRs and live evidence take precedence.

## Scope and working agreements

- Scope: P01 native end-to-end encrypted NAS storage, P02 automatic
  synchronisation and conflicts, P03 multiple devices and key lifecycle, P04
  offering, revoking and removing storage. Crypto choices are coordinated with
  B (keys, trust) and N (transport); P designs no cryptography.
- Session: "BaryonOS - Specialist P", coordinated by "BaryonOS - PR Bewaker".
- Merge: `ADR-SPECIALIST-TEAM-20261125` extends `ADR-MERGE-DELEGATION-20261123`
  to P. P squash-merges its own PR with `--match-head-commit` once the
  delegation's conditions hold, following the PR guard's merge-lane protocol.
- Keep host prototype, host simulation and native QEMU evidence separate.

## Last landed cycle: P04-T1 (PR #533)

- P04-T1 native LP-25 interrupted purge (criterion P04) landed on main as
  e68612f5 (26 Nov 11:01:56Z, squash of tested head 0b688369; tree equal).
  Issue #532 is closed; Project #5 shows Done for #532 and #533.
- delivered: `tools/test_nas.py` stages `interrupt` and `repair`; contract in
  `services/nas/native-probe/CONTRACT.md` "Interrupted purge and repair".
- evidence: local native gate on 0b688369 4/4, raw QEMU exit 33; all hosted
  gates passed on that head (72 pass, 11 skipped, merge-hold success).
- evidence limit: a controlled NAS process exit with the volume service alive.
  Not power loss, torn writes, disk errors, the revoke-PURGE window, remote
  sessions or physical erasure. Criterion P04 stays open until the lead
  records acceptance; the score is unchanged.
- ledger: `docs/roadmap-pipeline/ledger.json` P04-T1 is lead/Codex-owned; the
  write set, checks and acceptance are in PR #533 for the lead to record.

## Working rules (merge lane)

- One lane holder at a time; the PR guard announces each lane explicitly. No
  catch-up push outside the lane. Since B9, merge-hold first fails with only
  "Pending:" lines and restarts itself; act only on "Failed:", and rerun only
  infra failures with `gh run rerun --failed`.
- Before `gh pr merge` (owner, 26 Nov): `gh run list --commit <full head SHA>`
  filtered to non-completed runs other than "Merge hold" must return exactly
  `[]`. A Copilot "Agent Merge" run started through the owner account counts
  too (seen on #533 at 10:59Z). Squash with `--match-head-commit`; never
  `--admin` or auto-merge.
- After a squash of a branch that another local branch stacks on, merge the
  tested head first, then main with `-s ours` once main's tree equals that
  head (otherwise every shared hunk conflicts against the old merge-base).

## Local stacked cycle: P02-T3 slice 1 (not pushed)

- branch `claude/p02-t3-nasp-head-20261125`, worktree
  `~/projects/BaryonOS-worktrees/p02-t3-head`, stacked on this branch because
  it touches the same probe files. Local only under the CI-first freeze; it
  becomes its own PR after #533 lands (merge main in, never rebase).
- row reconciliation (25 Nov ~18:30Z, origin/main de242a3f): P01-T2 waits on
  the owner's context-mapping disposition; P01-T3 waits on P01-T2, B02-T1/T3
  and N03-T3 (all blocked); P03-T2 waits on the owner's authority model
  (P03-T1). P02-T3 is the only P row without an owner or cross-track gate.
- proposed spec for the lead (ledger P02-T3 is skeletal):
  - goal: slice 1 adds NASP opcode 15 Head, delegating to the existing
    `Service::head`; slice 2 adds a native guest stage that edits offline,
    survives a NAS cold restart and reconciles via Head; slice 3 (guest
    checkpoint persistence) needs a guest storage grant and is an OWNER
    question before any code.
  - invariant: Head is read-only, session-scoped and caller-bound by the
    kernel-supplied sender; staging is invisible; no new capability and no
    core, snapshot, volume or checkpoint format change.
  - write set (slice 1): `services/nas/native-probe/src/lib.rs`,
    `services/nas/native-probe/tests/ipc.rs`,
    `services/nas/native-probe/CONTRACT.md`.
  - acceptance: the `nas-core.yml` nas-workspace steps for the probe on Rust
    1.100.0 (debug and release test and clippy `--all-targets`, `fmt --check`,
    `x86_64-unknown-none` build, native `clippy --features native --bins`);
    probe and NAS Python unittests; the unchanged four-stage native gate.
  - local result (b5d1221f): probe host tests 8+2+3 pass, clippy/fmt pass,
    native clippy pass, Python unittests OK, native gate 4/4 raw 33
    (normal, recover, interrupt, repair; source 72b890a4…).
  - evidence limit: host dispatch tests plus a native regression run; slice 1
    makes no native Head call and no offline reconcile in QEMU.

## P02-T3 slice 2 (2a and 2b implemented and committed locally, not pushed)

Finding: `native-client` drives `Service<S>` in-process (`prepare`, `commit`,
`cancel`, `reconcile`, `reconcile_latest` take `&mut Service` and a
`Principal`); it cannot run in the guest over NASP as is. The native probe has
no in-boot NAS restart; only the separate `native-supervisor` profile
restarts processes.

- 2a (host, P-owned crates): a `Remote` trait in `native-client` covering
  open/stage/commit/cancel/close/read/head. `Service` plus the trusted caller
  implements it with unchanged behaviour and tests; `native-probe` adds a NASP
  adapter over a caller-supplied request/reply function. Host tests drive the
  client through encoded frames and `dispatch`.
  - write set: `services/nas/native-client/src/lib.rs` and its tests;
    `services/nas/native-probe/{src/lib.rs,tests/ipc.rs,Cargo.toml,Cargo.lock}`
    (local path dependency on `nieuw-os-nas-client`, no external crate);
    `services/nas/native-probe/CONTRACT.md`.
  - invariants: a Commit whose request was sent but whose reply is missing or
    malformed is `Unknown`, never `Io` or success (same rule as the volume
    transport); no replay of an uncertain commit; payloads above the 104-byte
    probe limit fail `Invalid` before sending; the caller principal never
    travels in the frame.
  - acceptance: nas-workspace steps on 1.100.0 for native-client and
    native-probe, including a lost-Commit-reply regression.
  - local result (321cfdaa): nas-workspace steps on 1.100.0 pass for all
    four NAS crates (client 21 tests, probe 16 incl. 3 new client-over-NASP
    tests), native probe clippy passes, NAS Python unittests OK, native gate
    4/4 raw 33. `source_hash` covers `services/`, so the gate binds
    native-client too. The probe `Cargo.lock` gains only the local path crate
    `nieuw-os-nas-client` (no external crate). Not run: hosted CI, a native
    NaspRemote call.
- 2b finding (owner-approved in-boot restart, ~19:10Z): a second NAS process
  in the same boot needs a runtime re-grant. The NAS ingress and volume-reply
  endpoints are owned by the NAS pid and `observe_exit` calls
  `process_died`; admin, guest and volume hold bootstrap-delegated send caps
  and pin the NAS pid in word 160. The `native-supervisor` profile restarts
  workers through launch capabilities and endpoints that survive the worker,
  so in-boot restart means redesigning NAS endpoint ownership: capability
  design for an OWNER decision, not P code. Zero-re-grant alternative: the
  cross-boot cold restart, which needs a write-enabled stage (`RECOVER` maps
  the block device read-only) and carries the guest edit through the test
  harness handoff. A NAS-process-internal `Service::recover` is not what was
  approved: requests would queue, so the not-delivered path never runs.
  Either way the 2b spec must map a timeout after a queued send to
  `Delivery::Sent`, and must decide who drops a late reply with an older
  correlation (today it would make every later exchange `Unknown`).
- 2b owner choice (~19:15Z): cross-boot cold restart ("Herstart over twee
  boots"). Implemented as stages `offline` (4, root, fresh writable disk) and
  `resume` (5, child of the exact passing offline image, writable); no token
  handoff. Offline: the guest commits revision 1 through `NaspRemote`; the
  admin sends Shutdown; only kernel `observe_exit(NAS)`, after
  `process_died`, sets guest word 216, and the guest's next `prepare_via`
  cannot deliver Open (`Io`), so the edit stays `NeedsReconcile`. Resume: plain
  `Service::recover`; a base-0 edit reconciles via Head to `Conflict`; the
  offline edit, re-created from fixed probe constants (no client persistence),
  reconciles to `Ready` and commits revision 2. The edit is 32 bytes because the
  core's copy-on-write reservation counts the old 96 bytes against quota 128.
  Oracle `check_resume_image`: core generation 4/5, volume 7/9, Active epoch 1,
  no history. Counts admin/NAS/guest: offline 3/0, 8/2, 5/2; resume 3/0,
  15/2, 12/2. `nas-core.yml` needs no change (it runs `tools/test_nas.py`).
  Open: a late reply with an older correlation still makes later exchanges
  `Unknown` (no late-reply drop); the probe never times out a queued send.
  Local result (272c064f): native gate 6/6 raw 33 (normal, recover,
  interrupt, repair, offline, resume; source dee0df1f…); probe host tests,
  clippy (debug/release, native `x86_64-unknown-none`) and fmt pass on 1.100.0;
  NAS runner/build/oracle/replay unittests OK. Not run: hosted CI.
  Evidence limit: local QEMU IPC, a controlled NAS exit and a second boot on
  the same disk; not real transport, identity, power loss, an in-boot NAS
  restart or client persistence.
- 2b (native, original in-boot proposal, superseded): new root stage `resume` (startup word 112 = 4, feature
  `test-system-nas-resume`, fresh disk). The guest commits revision 1, the NAS
  process exits at a coordinated point, the guest queues an offline edit
  (prepare fails, edit stays `NeedsReconcile`), the fixture kernel observes the
  NAS exit and starts a fresh NAS instance with the same role grants, which
  runs `Service::recover`. The guest's old session is `Stale`; it reopens,
  reconciles via Head and commits revision 2; the admin checks usage and the
  disk oracle checks the generations.
  - write set: `services/nas/native-probe/src/main.rs`,
    `kernel/src/m2/m3/nas.rs` (feature-gated fixture only), the build profile
    in `tools/build.py` and the kernel feature list, `tools/test_nas.py`,
    `services/nas/native-probe/disk_oracle.py` and its tests,
    `tests/test_nas_build.py` and `tests/test_nas_runner.py`, and
    `nas-core.yml`, which is protected (CI owner D2 pin change).
  - design choice: an in-boot NAS restart (recommended; models the NAS going
    down while the client device stays up) versus a cross-boot restart that
    would carry the guest edit through a harness handoff. That handoff is
    test-only state, and real client persistence stays slice 3, an OWNER
    question.
  - evidence limit: local QEMU IPC and process restart with the volume
    service alive; not real transport, identity, power loss or client
    persistence.

## Next

1. P02-T3 slices 1, 2a and 2b sit on `claude/p02-t3-nasp-head-20261125`
   (worktree `~/projects/BaryonOS-worktrees/p02-t3-head`), merged with main
   e68612f5 (no content change; the diff against main is the P02-T3 work
   only). Not pushed: it needs its own issue/PR and a lane from the PR guard.
   Before the first push, re-run the six-stage native gate
   (`NOS_BUILD_DIR` inside the checkout, e.g. `build/<name>`, or
   `private_path` refuses it) and the nas-workspace steps on 1.100.0.
   Last local result (2d81de8a, main e68612f5): native gate 6/6 raw 33
   (source `ce651335…`); nas-workspace steps for all four NAS crates, probe
   native clippy and the NAS Python unittests exit 0 (145 Rust tests).
   Tracking issue #554; lane queued after H (overlap with S/H only in
   `kernel/Cargo.toml` and `tools/build.py`).
2. Slice 3 (guest queue persistence across a guest restart) is an OWNER
   question before any code.
