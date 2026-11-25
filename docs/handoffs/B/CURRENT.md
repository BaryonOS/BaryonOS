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

- Session: local "BaryonOS - Specialist B [d5840a]" (25 Nov ~19:15Z, resumed from #524 HANDOFF 7171449464;
  registered with the PR guard).
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
  frozen Names, swtpm probe `docs/verification/b03-rel05-lrec-probe-20261125/`), TEST loader M1-HAUTH BOUND provisioning
  (`tpm-hauth-provision-test`, `run_tpm.py --case loader-hauth`: provision, then verify on a second boot of the same swtpm
  state; PCR 13 `CLOSED` through `HashLogExtendEvent`, SHA-256 bank = `P_CLOSED`; policy read and empty-ownerAuth refused;
  no secret residue in the CRB buffer; `docs/verification/b03-rel05-hauth-bound-20261125/`, local commits 8f45c07e, 1a46fa21), DA lockout counter `0x20E` unchanged around the refused owner command, native halt case `loader-hauth-no-rng`
  (CLOSED extend, then `RNG_ABSENT`, raw 35, no kernel; `docs/verification/b03-rel05-hauth-halt-20261125/`, local commit 89d00679),
  pre-seeded cases via `tools/tpm_seed.py` (platform `HAUTH` squat `0x149`, foreign ownerAuth `0x18B`, owner `HAUTH`
  squat under a foreign ownerAuth halts `PUBLIC`, unbound `LREC` after a provisioning boot, power-cut leftovers repaired;
  `docs/verification/b03-rel05-hauth-seeded-20261125/`, local commits 4e2eda89, e1c749a3, 38453a5f).
- NOT RUN: a provisioning-path halt after an NV write (fault injection), a cut between the ownerAuth and lockoutAuth
  changes, an owner-created squat under an empty ownerAuth as a separate case, a DA-counter sensitivity check, PCR 13 read-back of the
  SHA-1/384/512 banks, S3b/genesis/ACCEPTED/UEFI copy, U22 write atomicity, native S5 and joint N04 tests, hosted CI for
  the loader-* cases (the noble OVMF 2024.02 image is not on the local host), physical hardware, human audit.
- Licensing register: `boot/loader/src/tpm_probe.rs`, `tpm_hauth.rs` and `tools/tpm_seed.py` are not listed in
  `MPL-2.0-FILES.txt` (no test enforces it); the register is an owner/lead decision, so it is reported, not changed.
- CI risks for the seeded slice: `TpmSeedTests` (skipped without swtpm) and the seeds use `--server`, `--ctrl` and
  `--flags not-need-init,startup-clear`, verified only on swtpm 0.10.1 (hosted CI has 0.7.3); the halt-unchanged check
  in `run_tpm.run()` has no unit test; ci-impact selects host-quality through the `tools/**` glob.
- Open, reported to the owner, not accepted: boot-path revival at PCR 13 = 0 before B04; S3/U27; U13c record
  staging and both-copies-lost revival; REL-05 section 7.9 DEGRADED versus halt; the TEST loader does not check
  ownerAuth against `HAUTH[..32]` (a foreign owner planting a well-formed `HAUTH`/`LREC` would bind); U25 on the hosted OVMF image and real firmware.

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

- Done locally (25 Nov, not pushed): M1-HAUTH BOUND provisioning slice (spec steps 1-5). Commits 8f45c07e (frozen Names,
  frame sizes), 1a46fa21 (loader, runner, tests), fe21707e (evidence), then an evidence-wording commit. Gate parity at
  1a46fa21 source (Rust 1.100.0, `GIT_EDITOR` unset): root fmt; tpm-crb test/clippy/fmt/scalar `x86_64-unknown-none`
  build; update-core test/clippy; `compileall`; `tools/tests` (1178 OK, 15 skipped); the ci.yml `tests/` patterns;
  `tools/test.py` (boot acceptance). `agent_gate.py --job ci.yml#host-quality` cannot resolve locally (`uses:` step). Observed: undefine of a missing index =
  `0x28B`; refused owner command = `0x9A2` (`BAD_AUTH`; lockout counter `0x20E` unchanged at 0, sensitivity unproven); policy read after the extend =
  `0x99D`.
- Done locally: 89d00679 halt path + DA counter (`loader-hauth-no-rng`), fbe0e81c evidence, then this handoff update.
  At source 1438acd9: six native cases pass (`loader-hauth` repeated, identical); `tools/tests` 1179 OK; `compileall`;
  loader build, fmt, clippy (only the known `main_default.rs:818` errors).
- Done locally: 4e2eda89 pre-seeded cases (`tools/tpm_seed.py`, four `run_tpm.py` cases, halts keyed per (case, boot)),
  e1c749a3 evidence. At source a9bd4cef: ten native cases pass; the six existing cases keep their stdout hashes, so the
  1438acd9 loader evidence carries over; `tools/tests` 1181 OK (15 skipped); `test_tpm` 10 OK; `compileall`.
- Done locally: 38453a5f `loader-hauth-owner-squat` (owner AUTHREAD | AUTHWRITE `HAUTH` + foreign ownerAuth, halt
  `FAIL:PUBLIC:0100b304`, seeded state unchanged), then the evidence/handoff commit. At source bb091002: eleven native
  cases pass, the ten earlier ones with unchanged stdout hashes; gate results are in the newest #524 CHECKPOINT comment.
- Next on #524, in order: (b) S3/U27 and the section 7.9 halt, (c) native S5 with N's joint
  tests. Genesis, ACCEPTED and the UEFI copy wait for S3b and the OWNER answer on U13c record staging.
- When the PR guard grants the lane: merge `main` into #524 (no rebase), push, rerun the applicable gates, refresh the PR
  body and post a HANDOFF comment. Merging `main` changes `source_hash` (#544 touched `tools/`): rerun the eleven
  `run_tpm.py` cases (six + five seeded) on the merged head, or state in the PR body that the loader evidence is bound to bb091002; #524 changes `docs/DECISIONS.md`, so D2 owner review still applies.

Lane: owner correction ~18:05Z; origin/main 9dcbcc69 (#547) unchanged at 19:23Z, #524 BEHIND by 6; no lane grant yet. CI keeps priority until every CI batch is resolved, and the main freeze holds after #544.
#524 (draft, BEHIND main) does not catch up or push until the PR guard grants the lane. Infra reruns on 5e126571
(runs 45649628744, 45649628788, 45649628817) all succeeded; merge-hold stays red only because the branch is BEHIND.
