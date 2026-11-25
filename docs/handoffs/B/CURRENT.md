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

- Session: local "BaryonOS - Specialist B [3d498e]" (25 Nov, resumed ~20:15Z from this file at b4aa7304 and #524
  CHECKPOINT 7173038286; registered with the PR guard [a1de4b], acknowledged; [2a403f] retired).
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
  `docs/verification/b03-rel05-hauth-seeded-20261125/`, local commits 4e2eda89, e1c749a3, 38453a5f), the owner squat
  under the empty ownerAuth (removed, boots identical to `loader-hauth`) and DA-counter sensitivity (seeded `0x20E` = 1,
  loader `DA:1:1`, all other provisioned boots 0; `docs/verification/b03-rel05-hauth-da-squat-20261125/`, local commit 06b7528a),
  PCR 13 read-back of every active bank (SHA-1/256/384/512 equal `H(0^n || H(CLOSED))` on OVMF 2025.11;
  thirteen native cases at source 16ff8409; `docs/verification/b03-rel05-pcr13-banks-20261125/`, local commit 2c9fdec9),
  the seeded cut between the ownerAuth and lockoutAuth changes (`loader-hauth-lockout-cut`: `MODE:EXISTING`, `BOUND`,
  `LOCKOUT:SET` at PERMANENT `0x401`, only lockoutAuthSet changes, boot 1 identical to `loader-hauth`; fourteen native
  cases at source 6817b4e7; `docs/verification/b03-rel05-hauth-lockout-cut-20261125/`, local commit 94acc222),
  the remaining cut points (`HAUTH` defined, `HAUTH` written, `LREC` written: each re-provisions with `loader-hauth`
  output, and `check_state` requires a fresh `LREC`; seventeen native cases at source 2816266d;
  `docs/verification/b03-rel05-hauth-cut-points-20261125/`, local commit fa2d7f58).
- NOT RUN: a fault-injected interruption of a real loader boot and a torn NV write (every provisioning step is
  covered by a host-seeded cut), the TCG event log for PCR 13, S3b/genesis/ACCEPTED/UEFI copy, U22 write atomicity, native S5 and joint N04 tests, hosted CI for
  the loader-* cases (the noble OVMF 2024.02 image is not on the local host), physical hardware, human audit.
- Licensing register: `boot/loader/src/tpm_probe.rs`, `tpm_hauth.rs` and `tools/tpm_seed.py` are not listed in
  `MPL-2.0-FILES.txt` (no test enforces it); the register is an owner/lead decision, so it is reported, not changed.
- CI risks for the seeded slice: `TpmSeedTests` and the seeds use `--server`, `--ctrl ...,mode=0600` and
  `--flags not-need-init,startup-clear`, verified only on swtpm 0.10.1. `ci.yml#host-quality` installs no swtpm, so
  they skip there, but `native-system.yml` runs `tools.tests.test_tpm` with the pinned swtpm 0.7.3, so they run on
  the first lane push; the post-boot state checks
  are unit-tested through `run_tpm.check_state()`; ci-impact selects host-quality through the `tools/**` glob.
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
- Done locally: 06b7528a `loader-hauth-owner-squat-empty` and `loader-hauth-da-counted` (seed `observe()` records
  `0x20E`; `evaluate` pins 0 on every other provisioned boot; seed `server()` socket-path guard), then the evidence/handoff
  commit. At source 99347dfa: thirteen native cases pass, the eleven earlier ones with unchanged stdout hashes; loader
  `5bbcc2fa…` and kernel `ba349c3b…` unchanged; `tools/tests` 1181 OK (15 skipped); `test_tpm` 10 OK; `compileall`;
  `progress.py --check` 43/100.
- Done locally: 2c9fdec9 PCR 13 bank read-back (codec `PcrReadBank`, loader `LOADER-HAUTH:PCR13:BANK:` lines for the
  `GetActivePcrBanks` banks before the `0x20E` reads, `run_tpm.closed_banks`), then the evidence/handoff commit. Results
  and hashes: `docs/verification/b03-rel05-pcr13-banks-20261125/` (thirteen cases pass at source 16ff8409; provisioned
  boots now `7a4c0d5c…`/`08a3f833…`, da-counted `14145a63…`, all halt/probe/kernel-only hashes unchanged; loader
  `da716d06…`, kernel `c6cef6ab…`; tpm-crb 1.100.0 test/clippy/fmt/scalar, `tools/tests` 1181 OK, `progress.py --check` 43/100).
- Done locally: 94acc222 `loader-hauth-lockout-cut` (seed `lockout-cut`: host TEST `HAUTH` secret, BOUND `LREC` through
  the PCR 13 = 0 write policy with the `trial.rs` encoding pinned in `test_tpm`, ownerAuth = `HAUTH[:32]`, lockoutAuth
  empty; runner `REPAIRED` table and an after-boot check that only lockoutAuthSet changed), then the evidence/handoff
  commit, then the assessment/spec commit (CHECKPOINT 7173740570, ~20:30Z). Fourteen native cases at source
  6817b4e7; results in the new evidence directory. #524 checks on 5e126571: 69 pass, 10 skipped, only merge-hold
  red (BEHIND); 0 unresolved review threads.
- Done locally: fa2d7f58 remaining cut points (seeds `hauth-defined`, `hauth-written`, `lrec-written` as one helper
  chain; cases `loader-hauth-cut-*`; `check_state()` with unit tests and the fresh-`LREC` rule), then the
  evidence/assessment commit. Seventeen native cases at source 2816266d; `tools/tests` 1184 OK.
- Next on #524, decision-free: the TCG event log for PCR 13 (the loader reads the firmware log and checks one
  `EV_IPL` `CLOSED` entry for PCR 13), unless the lane arrives first. Then, only after the owner
  answers (both are in the "reported to the owner, not accepted" list): (b) S3/U27 and the section 7.9 halt; (c) native
  S5 with N's joint tests. Genesis, ACCEPTED and the UEFI copy wait for S3b and the OWNER answer on U13c record staging.
- When the PR guard grants the lane: merge `main` into #524 (no rebase), push, rerun the applicable gates, refresh the PR
  body and post a HANDOFF comment. Merging `main` changes `source_hash` (#544 touched `tools/`): rerun the seventeen
  `run_tpm.py` cases (six + eleven seeded) on the merged head, or state in the PR body that the loader evidence is bound to 2816266d; #524 changes `docs/DECISIONS.md`, so D2 owner review still applies.

Lane: the owner lifted the main freeze at 20:19Z (main 0fe66b51, #524 BEHIND by 8, no TPM-file overlap). Lane order from the
PR guard: #515 (H) -> #539 -> #533 -> #543 -> #535 -> #525 / #523 / #540 / #524 / #538; one lane holder at a time, no
catch-up outside the lane; the PR guard announces the grant explicitly. The repository moved to GitHub Enterprise Cloud
(hosted job limit 60 -> 500, being verified by CI): report runner, required-check, permission, billing or hosted swtpm
anomalies to the PR guard; merge as soon as the lane is held and the applicable gates and reviews are done.
Earlier: owner correction ~18:05Z; #548 at 19:58Z. #548 changes merge-hold to decide at once and
re-check after the gates finish, so the 25 Nov merge-hold lesson below may be outdated; re-read it before the lane merge. CI keeps priority until every CI batch is resolved; the main freeze ended at 20:19Z.
#524 (draft, BEHIND main) does not catch up or push until the PR guard grants the lane. Infra reruns on 5e126571
(runs 45649628744, 45649628788, 45649628817) all succeeded; merge-hold stays red only because the branch is BEHIND.
