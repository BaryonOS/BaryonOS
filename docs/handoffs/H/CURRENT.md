# Specialist H — current handoff

Compact resume pointer for the H track. It is not a backlog: tasks live in
`docs/progress.json`, `docs/roadmap-pipeline/ledger.json`, the H
specifications and GitHub. This file plus the live handoff comment named
below are the canonical H state; no chat history, private memory, local
path or specific tool is needed to resume.

## Scope and mandate (recorded once)

- **Scope:** hardware and power lifecycle: H04 virtio-net TX and link edges,
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
- **Working rules:** every cycle starts on a fresh branch from `origin/main`
  with an isolated `NOS_BUILD_DIR` inside the checkout; native evidence is
  bound to the tested head and stored in a new `docs/verification/` child
  directory; a merged slice does not accept a criterion.

## Current cycle

- **cycle_id:** `h05-s3-noscrub-20261127`.
- **Landed on `main` (27 November 2026):** H05-T5d-T, S3 SUSPEND requires a
  LOCKED, erased keyring (KL2 -25; PR #597, `8b00a9c0`); the H04-TXDOWN ADR
  amendment, -27 TX_EDGE, documentation only (PR #601, `8ee0d501`);
  the S3 noscrub red control, where `test-system-s3-lock-noscrub` now gets the S3 TEST init and the power gate case `power-s3-noscrub` expects `LOCK_STACK_RESIDUE` (PR #613, `af7415df`; ledger H05-T5d-T `verified`). Earlier: H05-T5c slice 1, virtio-blk re-initialisation
  after S3 with quarantine -24 (PR #556, `945847d8`), and H02-T5 (PR #515,
  `744f1b75`).
- **Open PRs:** this handoff refresh only.
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

- KL2: SUSPEND (0x91) returns -25 unless the keyring is LOCKED and the
  instance erased (`ADR-KEYRING-LOCK-20261126`).
- KL4/F1: LOCKED latches only after the verified exit-path stack scrub;
  without it the kernel halts with `LOCK_STACK_RESIDUE`.
- TEST profiles only: the unwrap secret is a public TEST constant; no
  key-state durability is claimed; no production LOCK before owner
  decision O5.
- Error codes: -24 QUARANTINED, -25 KL2 SUSPEND, -26 reserved for the
  project owner's ADR (S proposes it for `RemountRefused`), -27 TX_EDGE
  (decided in #601, not implemented).

### Owner questions and blockers

- -26 value (owner ADR); O5 blocks H05-T5d-P (production LOCK).
- OD-HW-TARGETS, -BOOT, -CAPTURE, -SIGNERS, -INV-ROUTE
  (`docs/drivers/hardware-matrix.json`) before any physical HW-INV run; the
  GPT/FAT32 image layout is an assumption until OD-HW-BOOT.
- B04 loader reproducibility (PR #493) before an image hash can be
  reproduced from an independent clean checkout.
- H03-T3b audio broker waits for owner decisions B1/B2/B4.

### Agreements

- With N (27 November 2026, on #529): the H04-TXDOWN implementation PR adds
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
   variable: use `local` in step and mutant functions.
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
    108 bytes. Keep case names short; never shorten the 32-hex suffix.
13. The kernel embeds user ELFs with `include_bytes!` and Cargo freshness is
    mtime-based: switching the init variant for the same kernel feature in
    one target directory can leave a stale init. Touch the
    `system-{init,app,worker}` sources or use a fresh target directory before
    rebuilding. Hosted CI is unaffected.
14. In `tools/build.py`, a feature list derived from another list must be
    computed after every list it depends on is complete (the
    `test-system-s3-lock-noscrub` profile built the plain init until #613);
    `tests/test_system_harness.py` pins this plumbing.

## Next executable task

1. H04-TXDOWN implementation PR (#600): -27 `syscall()` exemption, txdown
   hold, TXDOWN-1/2 and control cases, `NET_TX_EDGE` in
   `sdk/runtime/src/native.rs`; N review on #529.
2. H05-T5c slices 2–4 and T5e/UNLOCK with B and S; H05-T5d-P after O5.
3. Physical H02–H06 work after the owner decisions above.

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
