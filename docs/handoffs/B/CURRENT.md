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
- Gap: neither the local gh token (no `read:project`) nor the cloud session (repository-scoped) can reach project 5, so membership and
  fields are not set; owner action needed.

## Current cycle

- Session: local "BaryonOS - Specialist B" (25 Nov, resumed from the PR #524 HANDOFF comment).
- cycle: B03-REL05 trial boot with a TPM anchor, M1-HAUTH under owner decision option 1 (TEST profile,
  QEMU/swtpm only; ADR-B03-REL05-TPM-20261125, C665 identity binding).
- branch: claude/b03-rel05-tpm-20261125, worktree ~/projects/BaryonOS-worktrees/b03-rel05-tpm.
- PR: #524 (draft). Live head, checks and evidence: the newest HANDOFF/CHECKPOINT comment on #524.
- Merge order: #518, then N's N03/N04 ADR PR, then #524 (after merging `main`). #524 changes
  `docs/DECISIONS.md`, so it needs the project owner's review under D2 (ADR-PROGRESS-ASSESSMENTS-20261125).
- B03-F1 (#514) is merged; its REL-05 characterization is the input to this cycle.

## Evidence / NOT RUN

- Evidence: host model `crates/update-core/tests/anchor_chain_model.rs`, `drivers/tpm-crb` codec and transport
  tests, swtpm probes under `docs/verification/b03-rel05-*-20261125/`, native `tools/run_tpm.py` normal/absent and
  loader-probe (S6 step 1: firmware TCG2 works through the loader's own binding; `EFI_RNG_PROTOCOL` absent on the
  current TEST QEMU line), loader-rng (S6 step 4: present with an opt-in TEST virtio-rng device on OVMF 2025.11;
  `docs/verification/b03-rel05-loader-rng-20261125/`), U26 loader record `LREC` (`0x0100B305`, `NV_LREC` with
  frozen Names, swtpm probe `docs/verification/b03-rel05-lrec-probe-20261125/`).
- NOT RUN: `HAUTH` provisioning, loader code that reads or writes `LREC`, U22 write atomicity, native S5 and joint N04 tests, hosted CI for the loader-probe
  and loader-rng cases (the noble OVMF 2024.02 image is not on the local host), physical hardware, human audit.
- Open, reported to the owner, not accepted: boot-path revival at PCR 13 = 0 before B04; S3/U27; U13c record
  staging and both-copies-lost revival; REL-05 section 7.9 DEGRADED versus halt; U25 on the hosted OVMF image and real firmware.

## Merge procedure lesson (25 Nov)

- With `pull_request_wait_minutes = 0`, merge-hold fails at once while gates run. After the applicable gates finish
  (ignore `upload-coverage-*`, `copilot*`, Graphify), re-run the merge-hold job, then squash-merge with the expected
  head before `main` moves. #493 needed six update rounds; signalled on #503.

## Blockers (external)

- B02-T3 cold rollback anchor: owner ADR required.
- B05 external human audit: not commissioned.
- B06 native local AI / model loader rows: missing L/P/A components.
- B04 hosted Secure Boot job: protected `.github/**` request
  (docs/implementation/B04-SECURE-BOOT-CI-INTEGRATION-REQUEST.md).

## Queued B work (no PR yet)

- After #524: B02-T3 cold rollback anchor consumers and B04 production custody (both need owner decisions).

## Lessons

1. `tools.tests.test_roadmap_runner` has 5 pre-existing failures while build/roadmap-pipeline/STOP exists.
2. With `GIT_EDITOR=true` in the environment, git-driven `tools/tests` fail (2 failures, 9 errors); run them with it unset.
3. `tools/run_system.py` requires `NOS_BUILD_DIR` inside the checkout (`private_path`).

## Next executable task

- S6 step 4 (U25) is done locally: `run_guest(virtio_rng=True)` + `run_tpm.py --case loader-rng`; provisioning
  cases must use that device. U26 is designed: `LREC` layout in REL-05 section 6.2, trusted only with the written
  Name; the section 6.3 versus 10.5 UEFI-copy wording is flagged under U28. The U13c record-staging rule is
  *proposed* (REL-05 section 10.5 "Record staging", host model `staged_record_replacement_converges`); OWNER
  question open: adopt it in place of the halt-after-cut clause. The `LREC` codec is in `update-core`
  (`trial::LoaderRecord`, `hauth_binding`; N confirmed no conflict with the N04 issuer). Next on #524: loader
  `LREC`/`HAUTH` TPM I/O and `HAUTH`
  provisioning (wipe, CRB buffer overwrite, PCR 13 extend last, all four banks), S3/U27, the section 7.9 halt,
  native S5 with N's joint tests. Resume from the newest HANDOFF comment on #524.
