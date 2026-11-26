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

## Current cycle

- task: P04-T1 native LP-25 interrupted purge (criterion P04)
- branch: `claude/p04-t1-interrupted-purge-20261125`; PR number, tested
  source SHA and hosted runs are in the pull request description
- delivered: `tools/test_nas.py` stages `interrupt` (NAS exits between evict's
  volume WRITE and its history PURGE; one Revoked 96-byte copy retained) and
  `repair` (first recover refuses with `ReopenRequired`, then
  `VolumeStorage::finish_cleanup` purges history, then recovery). The repair
  stage boots only on the exact passing interrupt image. Contract:
  `services/nas/native-probe/CONTRACT.md` "Interrupted purge and repair".
- evidence limit: a controlled NAS process exit with the volume service alive.
  Not power loss, torn writes, disk errors, the revoke-PURGE window, remote
  sessions or physical erasure.
- ledger: `docs/roadmap-pipeline/ledger.json` P04-T1 is lead/Codex-owned; the
  proposed write set, checks and acceptance are in the pull request for the
  lead to record.

- state (25 Nov ~17:30Z): PR #533 is ready for review at f7fa786f (main
  a571d87c), and the local native gate passes there (4/4, raw exit 33). The
  GitHub incident reruns were accepted at 16:34Z (runs 45644871049 coverage,
  45644871306 CodeQL). merge-hold failed only on a wait timeout (unchanged
  head). At ~18:10Z main was de242a3f (#542 merged) and #533 was BEHIND, so
  merge-hold is not rerun on f7fa786f: the catch-up head re-runs it with the
  #542 poll fix. Reviews are handled:
  the CodeRabbit race is fixed, and the AI review and Graphify findings are
  answered. Codex and Copilot have no quota, so they are not retriggered.
- Freeze lifted (PR guard, 20:19Z; main 0fe66b51): the merge-lane protocol
  still holds (one lane holder; no catch-up outside the lane). Order #515 →
  #539 → #533; the guard announces the lane explicitly. Since B9, merge-hold
  first fails with only "Pending:" lines and restarts itself; only "Failed:"
  needs action, then `gh run rerun --failed` if it is an infra failure.
- GitHub Enterprise Cloud (PR guard, ~20:30Z): the hosted job limit is now
  up to 500; the merge lane stays until the merge queue is authorized. Merge
  promptly once holding the lane with applicable gates and reviews done.
  Report upgrade-related problems (unusual runners, missing required checks,
  permission or billing errors) to the PR guard.
- Superseded P0 CI-first mode (owner, ~17:25Z; corrected ~18:05Z): CI keeps priority
  until all CI batches are resolved, so the main freeze continues after #544
  lands. #533 gets the lane only when a CI PR is genuinely blocked (non-infra
  failure, 15 min without progress, or 60 min) or the CI exit criterion is
  met; order #515 → #539 → #533. No push that triggers hosted CI. Local
  handoff commits stay local and go out in the single catch-up push when the
  PR guard grants #533 the lane.

- Lane (26 Nov): the PR guard granted #533 the lane at 10:20Z. The catch-up
  head 8a8348e0 (main 29a159cc) passed the local native gate 4/4, raw exit
  33. #552 (CI priority) then moved main to c44134c6; the lane was reset to
  11:29Z and main is merged in once more.

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

1. #533 holds the merge lane (granted 26 Nov 10:20Z, reset to 11:29Z after
   #552 moved main to c44134c6). The live checkpoint is the PR: its head
   already merges main, so do not merge or push again unless main moves and
   the PR guard says so. Wait for the gates on that head; act only on
   "Failed:" (rerun infra failures with `gh run rerun --failed`). The
   open-run check must return exactly `[]`; then squash with
   `--match-head-commit <head>`, check the tree, set Project #5 to Done and
   report the landing to the PR guard. A native gate rerun needs
   `NOS_BUILD_DIR` inside the checkout, e.g. `build/<name>`, or
   `private_path` refuses it.
2. P02-T3 slices 1, 2a and 2b are committed locally on
   `claude/p02-t3-nasp-head-20261125` (stacked on this branch). It becomes its
   own PR after #533 lands; merge main in, never rebase. Slice 3 (guest queue
   persistence) is an OWNER question before any code.
