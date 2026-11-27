# Specialist H — current handoff

Compact resume pointer for the H track. It is not a backlog: tasks live in
`docs/progress.json`, `docs/roadmap-pipeline/ledger.json`, the H
specifications and GitHub. This file plus the live handoff comment named
below are the canonical H state; no chat history, private memory, local
path or specific tool is needed to resume.

## Scope and mandate (recorded once)

- **Scope:** the H01 guard and hardware and power lifecycle: H04 virtio-net TX and link edges,
  H05 ACPI S3/S5 suspend and resume (including virtio-blk re-initialisation
  and the KL2 keyring LOCK coupling), the power-system native gate, and
  H02–H06 inventory and support-matrix preparation. Network-service
  semantics belong to N (#529), kernel-shared interfaces to K (#530),
  Security Floor questions to B, CI workflows and pins to CI (#494), the
  volume remount contract to S (#564, #610).
- **Mandate:** `ADR-MERGE-DELEGATION-20261123` and the owner decision of
  26 November 2026 (20:10Z): H commits, repairs and enqueues its own
  merge-ready PRs. No force-push, admin bypass, auto-merge, direct `main`
  push or weakened test. H never edits protected files (root `AGENTS.md`
  list, including `.github/**`).
- **Per-PR integrity (owner mandate of 27 November 2026, ~12:35Z):** for
  each own PR, H keeps the labels, milestone M9, Project #5 Status,
  Subsystem and Work Type (final status included), the issue, roadmap or
  criterion link (`Refs`/`Part of #N`), source-bound evidence, a D3
  assessment in `docs/progress/assessments/<date>-<slug>.json` (also for
  docs and handoff PRs), this file and Graphify `remember` notes. The PR
  body names the functionality and acceptance criteria touched and the
  evidence that carries any progress claim. A merge is not acceptance.
  #551 and #556 landed without an issue link; add it in a later H PR
  where it fits naturally.
- **Own criterion rows (owner decision of 27 November 2026, ~12:50Z):** H
  keeps the implementation and verification snapshot rows of its own
  criteria in `docs/progress.json` (`delivery_review`) current without
  asking per update. Only H's own rows; current source-bound evidence;
  rubric, weights and acceptance conditions unchanged; the score rises only
  on full acceptance with completed independent verification. Do not
  change the global snapshot date or `source_commit`; put the date and SHA
  in the row text. Regenerate with `python3 tools/progress.py --write`, then
  `--check`, and use D3 assessment files to avoid conflicts.
- **Criterion acceptance (owner decision of 27 November 2026, 13:14Z,
  correcting 13:12Z; G records it on #622):** H stays the technical owner of
  its criteria: implementation, its own snapshot rows (score unchanged),
  verification and delivery of the acceptance evidence. The formal
  acceptance and any score increase are done by the PR guard, never by H.
  When every acceptance condition is met and the independent verification is
  complete, H reports the criterion ID and the evidence links to the guard.
- **Working rules:** every cycle starts on a fresh branch from `origin/main`
  with an isolated `NOS_BUILD_DIR` inside the checkout; native evidence is
  bound to the tested head and stored in a new `docs/verification/` child
  directory; a merged slice does not accept a criterion.

## Current cycle

- **cycle_id:** `h-owner-audio-hw-20261127`.
- **Landed on `main` (27 November 2026):** H05-T5d-T, S3 SUSPEND requires a
  LOCKED, erased keyring (KL2 -25; PR #597, `8b00a9c0`); the H04-TXDOWN ADR
  amendment, -27 TX_EDGE, documentation only (PR #601, `8ee0d501`);
  the S3 noscrub red control, where `test-system-s3-lock-noscrub` now gets
  the S3 TEST init and the power gate case `power-s3-noscrub` expects
  `LOCK_STACK_RESIDUE` (PR #613, `af7415df`; ledger H05-T5d-T `verified`;
  evidence [h05-s3-noscrub-20261127](../../verification/h05-s3-noscrub-20261127/README.md));
  the agent-neutral handoff pointer and the H02/H04/H05/H06 snapshot rows
  (PR #620, `a5ae23bf`, tree-identical).
- **Earlier:** H05-T5c slice 1, virtio-blk re-initialisation after S3 with
  quarantine -24 and O13 latch carryover (PR #556, `945847d8`; ledger
  `verified`, hosted gates green on `0e9e1b91`; evidence
  [h05-s3-blk-reinit-20261125](../../verification/h05-s3-blk-reinit-20261125/README.md)),
  and H02-T5 (PR #515, `744f1b75`; ledger `verified`, native evidence bound
  to `f8662109`; evidence
  [h02-usb-refusal-20261125](../../verification/h02-usb-refusal-20261125/README.md)).
- **Open PRs:** this decision record, `ADR-H-AUDIO-HW-OWNER-20261127`
  (branch `docs/h-owner-audio-hw`).
- **Live handoff:** the newest comment starting with `HANDOFF — Specialist H`
  on issue #564 (task, invariants, write set, branch and HEAD, decisions,
  commands with exit codes, not run, risks, next safe action, acceptance
  limits). Older H handoffs are on #515; the newest #564 comment supersedes
  them. A resuming agent first verifies branch, HEAD, `git status`,
  ownership and freshness against this file, that comment and live GitHub.
- **Communication:** #564 (H handoff; post-wake remount with S), #600
  (H04-TXDOWN implementation tracker), #529 (N), #530 (K), #494 (CI).

### Local work

None outside the open PRs listed above.

### Invariants and reserved values

- AUDIO syscall block `0xC0`–`0xCF` (agreed with K on #530,
  issuecomment-7219621427); ABI stays 1.0 until K's coordinated bump.
- KL2: SUSPEND (0x91) returns -25 unless the keyring is LOCKED and the
  instance erased (`ADR-KEYRING-LOCK-20261126`).
- KL4/F1: LOCKED latches only after the verified exit-path stack scrub;
  without it the kernel halts with `LOCK_STACK_RESIDUE`.
- TEST profiles only: the unwrap secret is a public TEST constant; no
  key-state durability is claimed; no production LOCK before owner
  decision O5.
- Error codes: -24 QUARANTINED, -25 KL2 SUSPEND, -26 reserved for the
  project owner's ADR (S proposes it for `RemountRefused`), -27 TX_EDGE
  (decided in #601, not implemented), -28 proposed by H and B for T5f-1
  "authentication absent" (pending S and B's ADR #631).

### Owner questions and blockers

- Owner decisions of 27 November 2026 on H03 audio (B1, B2, B4) and on all
  nine OD-HW-* are recorded on #564 (issuecomment-7219592317) and in
  `ADR-H-AUDIO-HW-OWNER-20261127`. OD-HW-BOOT, -DISKS, -CAPTURE, -PUBLISH and
  -INV-ROUTE are decided. OD-HW-TARGETS (no target identity), -SIGNERS (no
  named identity), -FAMILIES (R03 install transport) and -POWER (wake
  source) stay open with partial answers; every physical protocol stays
  blocked. The open parts, plus input-to-display capture equipment for R's
  R02-L1 cells (#530), go to the owner as one bundle.
- B01-LOCK owner decisions O2, O5, O7, O8 and OQ1 (#564,
  issuecomment-7219397369) are drafted by B in #631. H's T5f constraints are
  on #564 (issuecomment-7219429976). The order is authentication → T5f-1 →
  UNLOCK → T5f-2. T5f-1 needs a native authentication mechanism and owner,
  which are not chosen. H implements T5f only after #631 is on `main`.
- -26 value (owner ADR); H05-T5d-P (production LOCK) waits for B's O5
  freeze implementation.
- B04 loader reproducibility (PR #493) before an image hash can be
  reproduced from an independent clean checkout.
- H03-T3b audio broker: owner decisions given; B reviews the AUDIO right
  against its checklist (kernel-only mint, non-delegation in every transfer
  path, kind + holder + generation checks, revocation, no app DMA, MMIO or
  IRQ) once H opens the implementation PR. U's client requirements are on
  #564 (issuecomment-7219663903).
- K measures whether #602 (C3 scheduler) slows the H04-T3 net-link gates
  (27–52 % on hosted TCG; #602 issuecomment-7218963989); #602 is held until
  the cause is known.

### Agreements

- With N (27 November 2026; N's handoff on #529, issuecomment-7217112158): the H04-TXDOWN implementation PR adds
  `NET_TX_EDGE: i64 = -27` next to `NET_LINK_DOWN` in
  `sdk/runtime/src/native.rs` (H is the only writer of that constant). N
  reviews the `syscall()` exemption and the assertion
  `tx_edge_completed <= tx_completed`; N's network-service branch follows on
  #529 only after H's PR lands.
- With S (#564): the post-wake remount check is specified in
  `docs/implementation/VOLUME-POST-WAKE-REMOUNT.md`; the native PW cases need T5d-T (landed),
  T5e and UNLOCK, and H's agreement for `tools/run_system.py`,
  `tests/test_system_harness.py` and `init.rs`.

## Lessons

1. `tools/run.py` requires `NOS_BUILD_DIR` inside the checkout;
   `run_hw_inventory.py` needs a short `--run-root` for the QMP socket path.
2. A kernel profile that runs after `M2:PASS` must keep the non-feature
   `serial::exit(0x10)` as the cfg'd tail expression, or the default M2
   smoke changes.
3. `tools/build.py` accepts comma-joined kernel features, so HW-INV fixtures
   are separate Cargo features listed in `HWINV_FEATURES`, each paired with
   the `hw-inventory` loader.
4. A raw QEMU exit of 35 means a `0xF4` write of `0x11`; a halted guest ends
   only through QMP `quit` (raw 0).
5. `tools/tests/test_roadmap_runner.py` fails five `Session` tests while
   `GIT_EDITOR` is set in the environment; with it unset all pass.
6. `fsck.fat` drops a boot-sector volume label that has no matching
   root-directory label entry; the image uses `NO NAME`. `fsck.fat -v`
   prints "32 bit entries", not "FAT32".
7. HMP `info cpus` no longer reports `(halted)` in current QEMU;
   `info registers` still prints `HLT=1`.
8. A bash gate helper that sets a global `name` clobbers the caller's
   variable: use `local` in step and mutant functions, or mutant runs
   silently target the wrong build directory.
9. `evaluate_fixture` markers that end in `:` are value markers (prefix plus
   non-empty value); every other fixture marker must be the whole line.
10. QEMU routes QMP input to a USB HID device only after the guest's first
    interrupt-IN poll; wait on MFINDEX before printing a READY marker. Reset
    Device on a disabled slot completes with code 11 and Slot ID 0.
11. Any `kernel/` or `tools/` change, including the runner, changes the source
    fingerprint: rebuild before running a native case. `NOS_QEMU_MACHINE`
    accepts only recorded machines (`pc-q35-8.2`), not `q35`.
12. `run_system.py` puts the QMP socket at
    `build/<gate>/run/system-<case>-<32 hex>/qmp.sock`; it must stay below
    108 bytes. Keep case names short; never shorten the 32-hex suffix, which
    `capture_system.py` and the RANDOM/images validators pin.
13. The kernel embeds user ELFs with `include_bytes!` and Cargo freshness is
    mtime-based: switching the init variant for the same kernel feature in
    one target directory can leave a stale init. Touch the
    `system-{init,app,worker}` sources or use a fresh target directory before
    rebuilding. Hosted CI was not observed to hit this. The native H
    workflows (`.github/workflows/native-power.yml`,
    `.github/workflows/native-system.yml`) have no cache step, so each job
    starts without a target directory from an earlier run. Steps that share a
    target directory within one job (for example `target/power-native-s3`)
    build different kernel features. Hosted CI has not been tested
    specifically for this failure mode.
14. In `tools/build.py`, a feature list derived from another list must be
    computed after every list it depends on is complete (the
    `test-system-s3-lock-noscrub` profile built the plain init until #613);
    `tests/test_system_harness.py` pins this plumbing.

15. `tools/hw_matrix.py` refuses `OD-HW-TARGETS` as `decided` while the
    HW-FR target is null, and every `decided` owner decision needs a
    `reference`. `tools/tests/test_hw_matrix.py` pins the committed open
    count.

## Next executable task

1. H04-TXDOWN implementation PR (#600): -27 `syscall()` exemption, txdown
   hold, TXDOWN-1/2 and control cases, `NET_TX_EDGE` in
   `sdk/runtime/src/native.rs`; N review on #529.
2. H03-T3b: ring-3 audio broker with the AUDIO right and syscalls in
   `0xC0`–`0xCF`. Post the broker IPC interface on #564 for U's review
   first; output only; keep the TRANSPORT.md §7 oracle; send B the PR link
   for the Security Floor review.
3. H05-T5c slices 2–4: slice 2 needs B's LOCK/UNLOCK key erasure
   (`ADR-KEYRING-LOCK-20261126`, UNLOCK still open); slice 3 is S's
   post-wake remount; slice 4 is the ring-3 re-grant. Also T5e with B and S,
   and H05-T5d-P after O5.
4. Physical H02–H06 work once the owner names a target machine (one
   bundled request: target identity, signer, wake source, R03 transport,
   capture equipment).

Merge procedure: when every applicable gate is green and no review thread is
unresolved, mark the PR ready (Project #5 Status In Review). Immediately
before enqueueing, `mergeStateStatus` must be `CLEAN`, `HAS_HOOKS` or
`BEHIND`, `autoMergeRequest` null, and
`gh run list -R BaryonOS/BaryonOS --commit <full head SHA> --limit 200
--json databaseId,workflowName,status --jq '[.[] | select(.status !=
"completed" and .workflowName != "Merge hold")]'` must print exactly `[]`.
Enqueue with the GraphQL `enqueuePullRequest` mutation and
`expectedHeadOid`; `gh pr merge` on a `BLOCKED` PR silently enables
auto-merge. A `merge-hold` failure with only `Pending:` lines needs no fix.
After landing, compare the landed tree with
`git merge-tree --write-tree <parent> <head>` and post the landing SHA in the
live handoff comment.
