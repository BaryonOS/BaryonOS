# B workstream — current resume pointer

This file is the compact resume pointer for specialist B. It is not a backlog;
tasks live in `docs/roadmap-pipeline/ledger.json` and criteria in `docs/progress.json`.

## Scope and mandate (recorded once)

- Scope: finish B02 and B04–B06; guard B01/B03 (both accepted on main as of 1fc9b8ca).
- Mandate: specialist owner under ADR-MERGE-DELEGATION-20261123. May commit, push,
  open and repair PRs, resolve review threads with evidence and squash-merge
  (expected-head procedure). No direct main push, force-push, admin bypass,
  auto-merge without proven checks, or weakened tests. The old roadmap runner stays
  disabled; the merge queue is active (ADR-MERGE-QUEUE-AUTONOMY-20261126).
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

- Session: local "BaryonOS - Specialist B" (27 Nov, resumed from the #590 HANDOFF comment 7213772692). Merge queue
  autonomy (ADR-MERGE-QUEUE-AUTONOMY-20261126): B enqueues its own merge-ready PRs; the PR guard coordinates and
  does not approve.
- Landed: #524 (B03-REL05, ADR-B03-REL05-TPM-20261125 with C665 and the §7.9 fail-closed decision) as ce562ddc;
  #573 (B01-LOCK L1, issue #564) as 40c10d3a; #590 (S3 LOCK noscrub red-control profile for H05-T5d) as 9fe65ba9
  on 27 Nov. H adds the `power-s3-noscrub` case after #597 (pitfall reported: `S3_SYSTEM_FEATURES` must include
  the profile).
- cycle: REL-05 §7.9 M1-HAUTH halt. branch: claude/b03-rel05-hauth-unenrolled-20261127, worktree
  ~/projects/BaryonOS-worktrees/b03-hauth-unenrolled. Live head, checks and evidence: the PR body and the newest
  HANDOFF comment on that PR. The verify-only TEST loader halts `TPM_UNENROLLED` with `RECOVERY:REQUIRED` on an
  unset ownerAuth (never-provisioned or cleared); the enrollment build is the unauthenticated TEST stand-in for S3b.

## Evidence / NOT RUN (B03-REL05, landed via #524)

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
  `docs/verification/b03-rel05-hauth-cut-points-20261125/`, local commit fa2d7f58),
  the firmware event log check (the last entry of the crypto-agile log is the loader's PCR 13 `EV_IPL` `CLOSED`
  event with one `H(CLOSED)` digest per active bank, no earlier PCR 13 entry; seventeen native cases at source
  42e48d3c; `docs/verification/b03-rel05-hauth-eventlog-20261126/`, local commit 477674bf),
  the unenrolled halt (verify-only `tpm-hauth-verify-test` halts `TPM_UNENROLLED` + `RECOVERY:REQUIRED`, raw 35,
  state unchanged, on a never-provisioned and on a cleared TPM; nineteen native cases at source 5b392b72;
  `docs/verification/b03-rel05-hauth-unenrolled-20261127/`).
- NOT RUN: a fault-injected interruption of a real loader boot and a torn NV write (every provisioning step is
  covered by a host-seeded cut), the event log on halt paths, the SHA-1-format log and the final events table, S3b/genesis/ACCEPTED/UEFI copy, U22 write atomicity, native S5 and joint N04 tests, hosted CI for
  the loader-* cases (the noble OVMF 2024.02 image is not on the local host), physical hardware, human audit.
- Licensing register: `boot/loader/src/tpm_probe.rs`, `tpm_hauth.rs`, `tools/tpm_seed.py` and `drivers/tpm-crb/src/event_log.rs`,
  `tests/event_log.rs` are not listed in
  `MPL-2.0-FILES.txt` (no test enforces it); the register is an owner/lead decision, so it is reported, not changed.
- CI risks for the seeded slice: `TpmSeedTests` and the seeds use `--server`, `--ctrl ...,mode=0600` and
  `--flags not-need-init,startup-clear`, verified only on swtpm 0.10.1. `ci.yml#host-quality` installs no swtpm, so
  they skip there, but `native-system.yml` runs `tools.tests.test_tpm` with the pinned swtpm 0.7.3, so they run on
  the first lane push; the post-boot state checks
  are unit-tested through `run_tpm.check_state()`; ci-impact selects host-quality through the `tools/**` glob.
- Open, reported to the owner, not accepted: boot-path revival at PCR 13 = 0 before B04; S3/U27; U13c record
  staging and both-copies-lost revival (now a binding open condition of REL-05 §7.9); the §7.9 loader conditions
  (authenticated first provisioning via S3b: the TEST loader halts `TPM_UNENROLLED` after a clear since 27 Nov, but
  its enrollment stand-in is unauthenticated; recovery currency, recovery authority, user-visible recovery text;
  owner decided §7.9 fail closed on 26 Nov, host model done); the TEST loader does not check
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

- Later: B02-T3 cold rollback anchor consumers and B04 production custody (both need owner decisions); SUSPEND and
  UNLOCK for B01-LOCK (UNLOCK blocked on O2, O5 and O8; no production LOCK before O5).

## Lessons

1. `tools.tests.test_roadmap_runner` has 5 pre-existing failures while build/roadmap-pipeline/STOP exists.
2. With `GIT_EDITOR=true` in the environment, git-driven `tools/tests` fail (2 failures, 9 errors); run them with it unset.
3. `tools/run_system.py` requires `NOS_BUILD_DIR` inside the checkout (`private_path`).

## Next executable task

- The M1-HAUTH halt PR: after the hosted gates are green (first candidate failure: the `clear` seed on swtpm 0.7.3 in
  `native-system.yml`), 0 unresolved threads, `mergeStateStatus` CLEAN/HAS_HOOKS/BEHIND, `autoMergeRequest` null and
  the open-run check `[]`: ready, then enqueue with GraphQL `enqueuePullRequest` (never `gh pr merge` while BLOCKED).
  Report the landing SHA to the PR guard.
- Then the queued items above, one PR at a time. TEST/QEMU/swtpm only; the progress score stays 43/100.
- The detailed #524 slice history (M1-HAUTH provisioning, seeded cases, PCR 13 banks, event log) is in the #524
  HANDOFF/CHECKPOINT comments and the `docs/verification/b03-rel05-*` directories.
