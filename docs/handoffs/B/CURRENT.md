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

## Current cycle — 2026-11-27T22:17:32Z

- #649 landed at `7fceb5fd494c5c56126f0cf14a778779523a3fec` on
  27 November 2026. Exact merge-tree and patch comparisons passed; the
  landing evidence is on #649. It remains a host-only S7 foundation, not
  REL-05 or N04 acceptance. The score remains 43/100.
- Active primary B PR: #665, branch `codex/b03-s7-native-20261127`, worktree
  `/home/simon/projects/BaryonOS-worktrees/b03-s7-native`. Its first pushed
  head is `8b7710bf4393536d29d5bafd697d8586a0254da1`, based on #649's
  landed main. It records the project owner's approved U13c TEST staged-record
  route and the fail-closed both-copies-lost rule in the two implementation
  specifications. `docs/progress/assessments/2026-11-27-b03-u13c-approved-route.json`
  records the unchanged 43/100 assessment. JSON parsing, `progress.py --check`
  and `git diff --check` exited 0. Native tests are NOT RUN because #665
  changes specifications only. Hosted CI and exact-head review remain open.
- K confirmed on #529 that `kernel/src/m2/m3/tpm_fixture.rs` is free for B's
  scoped TEST S7 slice. Preserve K's live scheduler/return hunks in `m3.rs`;
  compare exact diffs on integration. No native S7 code has been written in
  this branch.

## Prior cycle — 2026-11-27T21:15:04Z

- Active primary B PR: #649, branch codex/b03-s7-anchor-20261127,
  worktree /home/simon/projects/BaryonOS-worktrees/b03-s7-anchor.
  The source baseline before this handoff refresh is
  7d7e8d6f3f8156bc1879e157b80b334483c8b8aa; it merged
  origin/main dc29612c before push. The only dirty file at this
  checkpoint is this handoff. Fetch and recheck the live head and status
  before acting. The newest B HANDOFF comment on #649 gives
  detailed exact-head evidence and review state.
- #649 is a bounded host-only REL-05 S7 foundation: ANCHOR capability,
  nondelegable authority model, private completion mailbox, one-way
  Running-to-Writing fence and four attempted extends per domain per boot.
  The planned syscall ABI is documented; native syscalls, CRB operations,
  authenticated ENROLL/BootInfo handoff and joint N04 evidence remain open.
  N04 stays option 5A. No REL-05 or N04 criterion is accepted by #649.
  The score stays 43/100.
- Source-bound checks on the last S7 code change: capability-core tests
  exit 0 (96 unit pass, one ignored, 11 integration pass), clippy exit 0,
  cargo fmt check exit 0, progress.py --check exit 0, git diff --check
  exit 0. Raw logs: ignored build/s7-evidence/anchor-completion-test-fix.log,
  capability-clippy-fix.log, fmt-check-fix.log, and
  progress-handoff-refresh.log. The handoff-only progress check and diff
  check both exit 0. Hosted exact-head CI is pending after the main merge.
  Native S7 syscalls/CRB/joint N04, hardware and human audit: NOT RUN.
- The owner approved the U13c TEST route on #529: stage a signed ENROLL
  or recovery record at a second ESP path, validate both records, choose
  the newest accepted one, and promote primary only after TPM and UEFI
  accepted-serial copies advance. Loss of both copies stays fail closed
  until independently authorized recovery. Production keys and anchors
  remain out of scope.
- Landed B PRs: #524 REL-05 base (ce562ddc), #573 LOCK L1 (40c10d3a),
  #590 S3 LOCK noscrub (9fe65ba9), #608 M1-HAUTH unenrolled halt
  (29735929), and #631 owner ADRs (ee29246b). Their handoffs are
  historical; #649 is the active source.

## Standing owner rules for B (27 Nov, relayed by the PR guard)

- Project integrity: per own PR, B maintains labels, milestone, Project fields (Status, Subsystem, Work Type; the
  local token lacks `read:project`, so the PR guard sets them), `Part of`/`Refs`, source-bound evidence, a D3
  assessment in `docs/progress/assessments/` (also for docs or handoff PRs), this file and a Graphify `remember`
  note for durable knowledge. The PR body names the affected criteria and the evidence behind any progress claim. A
  merge is not acceptance.
- `docs/progress.json` `delivery_review`: B keeps the implementation and verification rows of B01–B06 current
  without asking per update: own rows only, current source-bound evidence, rubric, weights and acceptance text
  unchanged, the global snapshot `date`/`source_commit` untouched (date and SHA in the row text), then
  `tools/progress.py --write` and `--check`.
- Criterion acceptance (owner, 27 Nov 13:14:07Z, correcting 13:12:47Z): B is the technical owner of its criteria
  and delivers implementation, snapshot-row refreshes (score unchanged), verification and the acceptance evidence.
  When B believes every acceptance condition of a B criterion is met and the required independent verification is
  complete, B reports the criterion ID and evidence links to the PR guard. Formal acceptance and the score change are
  done by the PR guard, never by B; conditions requiring human acceptance or an owner decision keep that requirement.
  No B criterion qualifies on 27 Nov.
- B01 stays accepted (reassessment of 24 Nov). LOCK/UNLOCK (ADR-KEYRING-LOCK, issue #564) lies outside its
  acceptance text and counts towards H05; the earlier "B01 stays open" wording is rectified by
  `docs/progress/assessments/2026-11-27-b01-criterion-wording.json`.

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
  state unchanged, on a never-provisioned and on a cleared TPM; nineteen native cases at source 5b392b72, repeated at aa95a441 after a main merge;
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

- B02-T3 cold rollback anchor: owner ADR approved in #631; implementation and independent evidence remain open.
- B05 external human audit: not commissioned.
- B06 native local AI / model loader rows: missing L/P/A components.
- B04 hosted Secure Boot job: protected `.github/**` request
  (docs/implementation/B04-SECURE-BOOT-CI-INTEGRATION-REQUEST.md).

## Next executable task

1. On #665, inspect exact-head CI and reviews. Repair substantive failures;
   after all applicable checks pass, require open runs `[]`, zero unresolved
   threads, `mergeStateStatus` CLEAN/HAS_HOOKS/BEHIND and null
   `autoMergeRequest`, then enqueue with expected head. Verify landing.
2. Start a new B PR for the native S7 path and owner-approved U13c TEST
   record flow. K's #529 hunk ownership applies. Keep REL-05 section 7.9
   fail closed; do not count host or documentation evidence as native N04.

## External dependencies and limits

- B02-T3 cold rollback anchor consumers require independent evidence;
  B05 external human audit is not commissioned; B06 depends on L/P/A
  native components. B04 hosted Secure Boot integration needs the
  protected CI workflow owner.
- B01 LOCK/UNLOCK order is #563, then #573, then SUSPEND/UNLOCK;
  O2/O5/O8 binding remains. No production LOCK before O5.
- Project #5 fields remain inaccessible to B's current token; the
  PR guard owns that metadata update. B does not issue D1-D3 verdicts.
