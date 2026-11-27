# Specialist P — resume pointer

Compact resume pointer for the personal-storage specialist (P01–P04). This
file is the canonical P state and stands on its own: a session with only the
repository and GitHub can resume from it. Local worktree paths are optional
conveniences and never the source of truth. Tasks come from
`docs/progress.json`, the roadmap ledger, issues and pull requests; newer ADRs
and live evidence take precedence.

State as of 27 Nov 2026 11:30Z (main 8ee0d501).

## Scope and working agreements

- Scope: P01 native end-to-end encrypted NAS storage, P02 automatic
  synchronisation and conflicts, P03 multiple devices and key lifecycle, P04
  offering, revoking and removing storage. Crypto choices are coordinated with
  B (keys, trust) and N (transport); P designs no cryptography.
- Keep host prototype, host simulation and native QEMU evidence separate.
- Write set: `services/nas/**`, `tools/test_nas.py` and its runner tests,
  this file, and new files under `docs/progress/assessments/`. Protected and
  lead-owned files stay out of P PRs (see `AGENTS.md`).
- Merge: `ADR-SPECIALIST-TEAM-20261125` extends
  `ADR-MERGE-DELEGATION-20261123` to P. Under the owner's autonomous merge
  queue decision (26 Nov ~20:10Z), P enqueues its own merge-ready PR. Before
  `gh pr merge <n> --squash --match-head-commit <sha>`:
  - the paginated open-run check prints nothing:
    `gh api --paginate "repos/BaryonOS/BaryonOS/actions/runs?head_sha=<sha>&per_page=100" --jq '.workflow_runs[] | select(.status != "completed" and .name != "Merge hold") | .id'`;
  - `mergeStateStatus` is CLEAN;
  - `autoMergeRequest` is null (`gh pr merge` on a BLOCKED PR silently enables
    auto-merge);
  - there are 0 unresolved review threads.

  Never use `--admin` or auto-merge. A merge-hold failure with only `Pending:`
  lines restarts itself; act only on `Failed:`. A queue squash does not
  auto-close `Closes #N` issues, so check them after landing.
- Review scope stop: a non-blocking review comment gets a reply, a resolve and
  a follow-up on the tracking issue, with no push. Only real blockers get one
  bundled push. Graphify feedback follows GR-12 (`@graphify wrong` with a
  one-line reason, or `@graphify good` once fixed).

## Gates

- `services/nas/*` crates (`nas-core.yml` nas-workspace) on Rust 1.100.0:
  - test, debug and release;
  - clippy `--all-targets` with `-D warnings`, debug and release;
  - `fmt --check`;
  - release build for `x86_64-unknown-none`;
  - for native-probe also
    `clippy --release --features native --bins --target x86_64-unknown-none`.
- Probe Python unittests: `test_disk_oracle.py`, `test_replay_handoff.py`.
- Six-stage native gate:
  `NOS_BUILD_DIR=$PWD/build/nas-native-ci CARGO_NET_OFFLINE=true python3 -B -u tools/test_nas.py`
  (stages: normal, recover, interrupt, repair, offline, resume).
  `NOS_BUILD_DIR` must lie inside the checkout. Only raw QEMU exit 33 per
  stage counts.
- Pitfall: `services/nas/native-probe/target/` is not ignored. Set
  `CARGO_TARGET_DIR` under `build/` for host cargo runs and never stage
  `services/nas` wholesale.

## Landed

- P04-T1 native LP-25 interrupted purge and repair: #533 → e68612f5.
  Stages `interrupt` and `repair`; contract "Interrupted purge and repair" in
  `services/nas/native-probe/CONTRACT.md`.
- P02-T3 slices 1, 2a and 2b: #577 → d5022f9b (issue #554).
  - NASP opcode 15 Head;
  - the `native-client` `Remote` trait and the `NaspRemote` adapter;
  - native stages `offline` and `resume` (cross-boot NAS cold restart,
    reconcile via Head).

  Native gate 6/6 raw 33; P02 stays open.

## In flight

- #595 (issue #592), the follow-ups from the #577 review:
  - `NaspRemote` drops a late reply with an older correlation after a receive
    timeout. The drop is bounded by earlier `Unknown` outcomes and never
    resends.
  - The `native-client` README limits `Unknown` to a Commit that was sent.

  Head 5a8f5263, all checks green, in the merge queue since 11:14Z. After
  landing:
  1. verify the squash against that head;
  2. close #592 if it is still open;
  3. post `@graphify good` on the #577 thread `PRRT_kwDOVZC91s68kBB8`;
  4. post a HANDOFF comment on #592.

## Open and blocked

- OWNER questions (no code before an answer):
  - P02-T3 slice 3: persisting the client queue across a guest restart. This
    needs a guest storage grant.
  - In-boot NAS restart: the NAS ingress and volume-reply endpoints are owned
    by the NAS pid. A second NAS process in the same boot therefore means
    redesigning endpoint ownership, which is a capability decision.
- Waiting on dependencies:
  - P01-T2: the owner's context-mapping disposition.
  - P01-T3: P01-T2, B02-T1/T3 and N03-T3.
  - P03-T2: the owner's authority model (P03-T1).
- Not implemented: a native late-reply stage (host test only), client
  persistence, a real transport, identity, power loss.

## Durable pitfalls

- The nas-core `stage()` reserves copy-on-write space for the old object too:
  a 96-byte revision under quota 128 allows at most a 32-byte replacement.
- After a kernel `process_died(NAS)`, a guest send on its NAS ingress fails
  with a non-again error, so `NaspRemote` maps it to `Delivery::NotSent(Io)`,
  never `Unknown`.
- A new `tools/test_nas.py` stage or failure label must extend the
  `safe_serial_evidence` allowlists.

## Next

1. Finish the #595 landing steps above.
2. Pick the next P row only if it has no owner or cross-track gate. Otherwise
   raise the OWNER questions on the tracking issue and wait.

Handoff threads: #592 (current slice), #554 (P02-T3).
