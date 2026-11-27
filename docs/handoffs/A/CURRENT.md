# Workstream A handoff (Applications, SDK, Installation and Ecosystem)

Durable, tool-neutral handoff for criteria A01–A04. Any agent can resume from
this file plus the live HANDOFF comment it links to. It does not depend on chat
history, private memory or a specific MCP; Graphify is optional context. Local
paths are optional convenience only.

- **Live state:** the newest comment beginning with `HANDOFF A` on issue #536.
  The current one is https://github.com/BaryonOS/BaryonOS/issues/536#issuecomment-7217812075.
- **Last updated:** 2026-11-27 (UTC), main 20c7e18a.

## Resume procedure

1. Verify `git fetch`, the branch, HEAD, `git status --short`, any open A PRs
   (`gh pr list --search "Part of #536"`) and the merge queue.
2. Compare them with the live HANDOFF comment. Never redo work that has a
   landed SHA.
3. Read the root `AGENTS.md`, the `AGENTS.md` files of the directories you
   change, `docs/handoffs/A/T2C-SPEC.md` and `docs/FULL-TEST-RELEASE.md`.

## Ownership and write set

- **Owner role:** Specialist A. Coordination goes through the PR guard role;
  `.github` changes go to the CI owner (#494).
- **A writes:**
  - `sdk/examples/task-grant-broker/**`;
  - the A03 kernel profiles `kernel/src/m2/m3/task_grant.rs` and
    `kernel/src/m2/m3/task_grant_volume.rs`;
  - in `kernel/Cargo.toml`, only the task-grant feature lines;
  - in `kernel/src/m2/m3.rs`, only the task-grant `cfg` lines;
  - `tools/run_task_grant.py` and `tools/build_task_grant_volume.py`;
  - the A lines in `tools/build.py` and `tools/run.py`;
  - `docs/handoffs/A/**`;
  - A assessment files under `docs/progress/assessments/`.
- Report the kernel write set on #536 before committing under `kernel/`. S and H
  own `volume.rs`.
- **Never edit:** `.github/**`, any `AGENTS.md` or `CLAUDE.md`,
  `docs/progress.json`, generated progress views, `docs/PROJECT-STATE.md` or
  `docs/DECISIONS.md`.

## Mandates and decisions (with source)

- **Merge delegation:** A squash-merges its own PRs. Sources:
  ADR-MERGE-DELEGATION-20261123, extended to A by ADR-SPECIALIST-TEAM-20261125
  clause 2 (#527, 9cc1b86f).
- **Autonomous merge queue** (project owner, 26 Nov; ADR in #576): A enqueues its
  own merge-ready PR through GraphQL `enqueuePullRequest` with
  `expectedHeadOid`, and only when all of the following hold:
  - mergeStateStatus is CLEAN;
  - autoMergeRequest is null;
  - there are 0 unresolved review threads;
  - no run is in progress or queued on the head.
- **Merge prohibitions:** never `gh pr merge` on BLOCKED (it silently enables
  auto-merge); never `--admin`; never auto-merge.
- **Review scope stop:** a non-blocking review point is answered, resolved and
  recorded on #536 without a push. Real blockers get one bundled push.
  Infrastructure failures get one rerun only.
- **T2c-3 exit notification:** the project owner chose option A
  (reply-endpoint closure) on 25 Nov 2026. It is recorded in
  `docs/handoffs/A/T2C-SPEC.md` ("Owner decision for T2c-3"). The capability-core
  pin landed as #579 (95553a22).

## Criteria status

| Criterion | Status | Ledger |
|---|---|---|
| A01 SDK and API | accepted | — |
| A02 install/update | open | A02-T1 verified (host); A02-T2 blocked (owner: source backend/anchor/actor, APP-SOURCE-RECOVERY.md §6/§8); A02-T3 blocked (kernel dynamic-launch ADR, B02 store) |
| A03 permissions | open | T1 host core verified; T2a #531 aa6d1c70; T2b #535 a004e3dc; T2c-1 #583 28e7ee1a; T2c-3 pin #579 95553a22; T2c-2 #593 7ee91d92; trusted dialog (U) still missing |
| A04 everyday apps | open | A04-T2 blocked (owner engine decision; N02–N04); A04-T3 blocked (H03 audio, S09, P01) |

Score: 43/100, unchanged by A since 25 Nov. Assessments
`2026-11-25-a03-volume-read-broker-host.json` and
`2026-11-26-a03-volume-native.json`.

## A03-T2c-2 summary (landed as #593, 7ee91d92)

The native broker is the sole client of a read-only volume domain 0
(`native-task-grant-volume`, `test-task-grant-volume-denied`). Both modes take
the boot epoch from startup word 32. The hosted gate is
`nas-core.yml#task-grant-volume`, added by CI as 2529d6bc.

Acceptance commands (sequential, one checkout, raw QEMU exit 33 each):

```
python3 -B tools/build.py --kernel-feature native-task-grant-volume && python3 -B tools/run_task_grant.py --case volume
python3 -B tools/build.py --kernel-feature test-task-grant-volume-denied && python3 -B tools/run_task_grant.py --case volume-denied
python3 -B tools/build.py --kernel-feature native-task-grant && python3 -B tools/run_task_grant.py --case normal
python3 -B tools/build.py --kernel-feature test-task-grant-denied && python3 -B tools/run_task_grant.py --case denied
python3 -B tools/build.py --kernel-feature native-sdk-app && python3 -B tools/run_sdk_app.py --case normal
python3 -B -u tools/test_nas.py
```

Evidence (recorded on #593):

- At head 5b2c7e3a: every case raw 33; disk reads 544 → 548 (volume) and
  544 → 544 (volume-denied); NAS gate 6/6; the seeded image is byte-identical
  before and after.
- Hosted: 75 checks pass.
- Mutations (skip authorise, skip retire, touch volume then deny, uncounted
  calls) failed with raw 35 on 1af1a516.

Evidence limits:

- Kernel epoch 1 equals the host constant `EPOCH`, so the word-32 change cannot
  be told apart natively.
- UNKNOWN_OBJECT_DENIED covers an absent object only.
- The Stale path is tested on the host only.
- The dialog and the exit notice are labelled fixtures.
- Only QEMU x86_64 was tested; there is no hardware evidence or human audit.

## Open follow-ups (non-blocking)

- The broker's `volume-core` dependency is not gated behind the `native`
  feature (Graphify note on #583).

## Next safe action

Plan A03-T2c-3 under owner option A, as specified in `T2C-SPEC.md`:

1. The broker retires a client's grants when its reply SEND fails with Revoked
   (-10) or Closed (-9), replacing the `observe_exit` fixture.
2. Add a native fault/kill case.
3. The profile must be a `milestone-m4` profile without the `native-smp-m3-ipc`
   slot restriction.

Report the kernel write set on #536 first. If the owner decision is ever
disputed, confirm it with the project owner before kernel work.
