# B workstream — current resume pointer

This file is the compact resume pointer for specialist B. It is not a backlog;
tasks live in `docs/roadmap-pipeline/ledger.json` and criteria in `docs/progress.json`.

## Scope and mandate (recorded once)

- Scope: finish B02 and B04–B06; guard B01/B03 (both accepted on main as of 1fc9b8ca).
- Mandate: specialist owner under ADR-MERGE-DELEGATION-20261123. May commit, push,
  open and repair PRs, resolve review threads with evidence and squash-merge
  (expected-head procedure). No direct main push, force-push, admin bypass,
  auto-merge without proven checks, or weakened tests. Old roadmap runner and
  merge queue stay disabled.
- At most one primary open PR (plus one managed child only when demonstrably needed).
- TEST keys/profiles grant no release signing, production anchors or new trust boundaries.
- Extra AI review is not external human audit (B05).

## Project metadata agreement (project owner, via the PR guardian, 24 Nov)

- Maintain project 5 membership, labels, Status, Subsystem, Work Type and milestone
  (B02: M6; B04–B06: M8). Priority/Start/Target are org issue fields; Start date only
  when proven, Target only for a real deadline. Effort (1/2/3/5/8) only on leaf issues.
- Record real issue–PR, parent/sub and blocked-by links; closing keywords only for the
  scoped issue a PR fully resolves. Merged = Done, other closure = Closed. A merge is not
  criterion acceptance. No schema/permission changes, no artificial issues.
- Gap on #486: the local gh token lacks `read:project`, so project 5 membership and
  fields are not set; owner action needed.

## Current cycle

- cycle_id: b04-repro-loader-20261124 (previous b02-key1-relay-20261124 closed: #486 merged as 93dabb65, CLEAR_READY)
- task: B04 build divergence (loader/image_payload) diagnosis and root-cause fix
- branch: claude/b04-repro-divergence-20261124
- PR: see the HANDOFF comment for the number and expected head
- state: RESUME_REQUIRED until the PR is squash-merged and verified on main.
- result: lld-link wall-clock COFF/debug timestamps and a PDB (run paths) feeding the CodeView GUID
  were the only differing bytes; `/Brepro` + `/DEBUG:NONE` on the UEFI target make two isolated
  default builds byte-identical on this host. Comparator unchanged, nothing normalised.

## Evidence / NOT RUN

- Evidence: `docs/verification/b04-repro-loader-20261124/` (red/red/green/green raw reports,
  byte diffs, Secure Boot TEST chain 5/5 at `0c590f13`, smoke raw 33, `tools/test.py` exit 0).
- NOT RUN: hosted CI; non-default profiles in repro_check; other host/path/toolchain;
  FAT image bytes; hardware; human audit. No CI job runs repro_check (protected `.github/**`).
- progress/PROJECT-STATE/ledger untouched (lead-owned). Their wording "loader/image_payload differ"
  becomes stale on merge; the lead should reassess B04-T2 text. No B04 credit: same-host only.

## Blockers (external)

- B02-T3 cold rollback anchor: owner ADR required.
- B05 external human audit: not commissioned.
- B06 native local AI / model loader rows: missing L/P/A components.
- B04 hosted Secure Boot job: protected `.github/**` request
  (docs/implementation/B04-SECURE-BOOT-CI-INTEGRATION-REQUEST.md).

## Parked local work (not pushed, not in this cycle)

- claude/b05-user-abi-20261123 e4e6a9aa — open medium verifier finding (quota off-by-one mutant survives).
- claude/b05-virtio-caps-20261123 5dc89767 — verifier MERGE.
- claude/b03-floor-crash-20261123 f18ba29c — verifier MERGE.

## Lessons

1. `tools.tests.test_roadmap_runner` has 5 pre-existing failures while build/roadmap-pipeline/STOP exists.
2. With `GIT_EDITOR=true` in the environment, git-driven `tools/tests` fail (2 failures, 9 errors); run them with it unset.
3. `tools/run_system.py` requires `NOS_BUILD_DIR` inside the checkout (`private_path`).

## Next executable task

- Finish the B04 repro PR: hosted checks green on the current head, expected-head squash merge,
  verify on main, post `HANDOFF B b04-repro-loader-20261124`.
- After that cycle: B05 parked branches (b05-user-abi quota off-by-one mutant first), or B04
  hosted job once the protected `.github/**` request is handled.
