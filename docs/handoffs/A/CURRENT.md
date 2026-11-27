# Workstream A handoff (Applications, SDK, Installation and Ecosystem)

Durable, tool-neutral handoff for criteria A01–A04. Any agent can resume from
this file plus the live HANDOFF comment it links to. It does not depend on chat
history, private memory or a specific MCP; Graphify is optional context. Local
paths are optional convenience only.

- **Live state:** the newest comment beginning with `HANDOFF A` on issue #536.
  The current one is https://github.com/BaryonOS/BaryonOS/issues/536#issuecomment-7217812075.
- **Last updated:** 2026-11-27 (UTC), main 9efd5739 (A03-T2c-3 branch).

## Resume procedure

1. Verify `git fetch`, the branch, HEAD, `git status --short`, any open A PRs
   and the merge queue. Search widely, because a PR may say `Part of`, `Refs`
   or only name a criterion: `gh pr list --state open --search "536 in:body"`
   plus `gh pr list --state open --search "A03 OR A02 OR A04 in:title"`.
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
- **A also writes** its own A01–A04 `delivery_review` rows in
  `docs/progress.json` and the views `progress.py --write` regenerates (owner
  decision 27 Nov ~12:50Z, below).
- **Never edit:** `.github/**`, any `AGENTS.md` or `CLAUDE.md`, other
  criteria's rows, the `delivery_review` header date and source_commit, scores,
  `docs/PROJECT-STATE.md` or `docs/DECISIONS.md`.

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
  (reply-endpoint closure) on 25 Nov 2026 and confirmed it on 27 Nov as final
  (never re-ask). It is recorded in `docs/handoffs/A/T2C-SPEC.md`. The
  capability-core pin landed as #579 (95553a22). The product path is built with
  K on the existing capability lifecycle; no general kernel event channel.
- **Own progress rows** (owner, 27 Nov ~12:50Z, via the PR guard; #536
  issuecomment-7218586123): A keeps its own `delivery_review` implementation
  and verification rows current without per-update permission. Conditions:
  current source-bound evidence, rubric and weights unchanged, header date and
  source_commit unchanged (date and sha go in the row text), `progress.py
  --write` then `--check`, a D3 assessment. Evidence paths render at the header
  commit, so link files that are newer as blob URLs pinned to a commit.
- **Criterion acceptance** (owner, clarified 27 Nov 13:14Z, guard log): A
  delivers the evidence. When all acceptance conditions and the required
  independent verification are met, A reports the criterion ID and evidence
  links to the PR guard. The guard does the formal acceptance and the score
  change. A never changes a score.
- **A02–A04 decisions** (owner, 27 Nov ~13:30Z; #536
  issuecomment-7219526385). These are binding; do not re-ask.
  - A02-T2: durable authenticated source registry.
    - Uses a B-managed freshness-anchor domain on TPM, separate from release,
      TLS and user keys.
    - A bounded add, rotate and revoke capability.
    - Transactional, fail closed.
    - Revoking a source blocks only new installs and updates.
  - A02-T3: GO with K and B. Uses a one-shot `AuthorizedImage` capability.
  - A03: build with K; coordinate the trusted dialog with U.
  - A04-T2: BR0 comparison of Chromium and Gecko. Chromium by default; the
    final choice is A's, in consultation with B, N and G.

## Per-PR progress duties (owner, 27 Nov)

Every A PR carries its own:
- labels, milestone and Project #5 fields (Status, Subsystem, Work Type), plus
  the final status;
- `Part of #N` or `Refs #N`;
- a D3 assessment in `docs/progress/assessments/<date>-<slug>.json`, also for
  docs and handoff PRs;
- source-bound evidence;
- an update of this file.

The PR body names the functionality and acceptance criteria it touches, and the
evidence that carries any progress claim. The score rises only for a fully
accepted criterion; a merge is not acceptance. A missing assessment is repaired
in the next own PR.

## Criteria status

| Criterion | Status | Ledger |
|---|---|---|
| A01 SDK and API | accepted | — |
| A02 install/update | open | A02-T1 verified (host); A02-T2 unblocked (owner 27 Nov; needs B freshness-anchor interface); A02-T3 GO with K and B (owner 27 Nov) |
| A03 permissions | open | T1 host core verified; T2a #531 aa6d1c70; T2b #535 a004e3dc; T2c-1 #583 28e7ee1a; T2c-3 pin #579 95553a22; T2c-2 #593 7ee91d92; T2c-3 exit/fault retirement (this branch); trusted dialog (U) still missing |
| A04 everyday apps | open | A04-T2 BR0 Chromium/Gecko comparison GO (owner 27 Nov); A04-T3 blocked (H03 audio, S09, P01) |

Score: 43/100, unchanged by A since 25 Nov. Assessments
`2026-11-25-a03-volume-read-broker-host.json`,
`2026-11-26-a03-volume-native.json` and
`2026-11-27-a03-t2c3-exit-closure.json`.

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

## A03-T2c-3 summary (owner option A, reply-endpoint closure)

- **Exit detection.** The broker no longer reads a kernel-written exit notice.
  - It probes each client-owned reply endpoint with `ipc_wait`: a SEND
    condition at deadline 0, which delivers and reserves nothing.
  - It retires all of a client's grants on Revoked (-10) or Closed (-9)
    (`client_gone`).
  - A reply SEND failing with those codes retires the same way.
  - Any other error is a broker fault.
- **Fault case.** `test-task-grant-fault` ends the owner with a ring-3 #UD.
- **Gate checks.**
  - The broker retired nothing before the owner's end (word 152 is 0 at that
    point).
  - The probe saw -10 for the owner PID (words 448 and 456).
  - No reply-send retirement occurred (word 440 is 0).
- **Profiles.** `native-task-grant` gains `ipc-wait`. Both task-grant profiles
  are `milestone-m4` without `native-smp-m3-ipc`, so `retire_authority` always
  runs `m4::process_exit`.
- **Evidence limits.**
  - Detection is polled.
  - Kill, orphan kill and invalid-RSP share the cleanup path by code reading
    only.
  - On SMP kill, grants end at the actual exit, not at the KILL request.
- **CI.** The hosted gate step for `--case fault` is requested from CI.

## Open follow-ups (non-blocking)

- The broker's `volume-core` dependency is not gated behind the `native`
  feature (Graphify note on #583).

## Next safe action

1. Land A03-T2c-3. Then deliver the A03 evidence to the PR guard only after U's
   trusted dialog replaces the fixture and independent verification is done.
2. A02-T2 with B: agree the freshness-anchor and rollback-floor interfaces, then
   specify the source registry against `APP-SOURCE-RECOVERY.md`.
3. A02-T3 with K and B: draft the `AuthorizedImage` launch ADR.
4. A04-T2: run the scoped BR0 Chromium/Gecko comparison with B, N and G.

Report any kernel write set on #536 before committing under `kernel/`.
