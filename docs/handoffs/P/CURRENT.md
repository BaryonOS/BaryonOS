# Specialist P — resume pointer

Compact resume pointer for the personal-storage specialist (P01–P04). This
file is the canonical P state and stands on its own: a session with only the
repository and GitHub can resume from it. Local worktree paths are optional
conveniences and never the source of truth. Tasks come from
`docs/progress.json`, the roadmap ledger, issues and pull requests; newer ADRs
and live evidence take precedence.

State as of 27 Nov 2026 14:05Z (main 10091f5b).

**Live handoff:** the newest "HANDOFF" comment on issue #592, currently
https://github.com/BaryonOS/BaryonOS/issues/592#issuecomment-7217363475
(HEADs, commands and exit codes, not-run list, next safe action). Graphify is
optional context only; every fact here and in that comment can be reproduced
from the repository and GitHub. On resume, first verify branch, HEAD,
worktree status, ownership and freshness against that comment.

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
- Project integrity (owner, 27 Nov 2026):
  - every P PR carries its own labels, milestone, Project #5 fields (Status,
    Subsystem, Work Type, final status), an issue or criterion link and a D3
    assessment in `docs/progress/assessments/` (also for docs and handoff
    PRs). The PR body names the affected criteria and the evidence behind
    any progress claim. A merge is not an acceptance;
  - P keeps its own `delivery_review` implementation and verification rows
    in `docs/progress.json` current without asking. The rows carry their own
    date and SHA; the global snapshot date and `source_commit`, the rubric,
    the weights and the acceptance conditions stay unchanged. Then run
    `python3 tools/progress.py --write` and `--check`;
  - formal criterion acceptance and any score increase are done by the PR
    guard. When all acceptance conditions of a P criterion are met and the
    required independent verification is complete, P reports the criterion
    ID and the evidence links to the guard.
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
- Python unittests:
  - `test_disk_oracle.py` and `test_replay_handoff.py` (in
    `services/nas/native-probe/tests`);
  - `test_nas_build.py` and `test_nas_runner.py` (in `tests/`).
- Six-stage native gate. On a fresh Cargo cache, first fetch the locked
  dependencies:
  `cargo +1.100.0 fetch --locked`, then the same with
  `--manifest-path sdk/Cargo.toml` and with
  `--manifest-path services/nas/native-probe/Cargo.toml`
  (the native job's setup in `nas-core.yml`). Then run:
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
- P02-T3 follow-ups from the #577 review: #595 → be1fd9ac (issue #592,
  closed).
  - `NaspRemote` drops a late reply with an older correlation after a receive
    timeout. The drop is bounded by earlier `Unknown` outcomes and never
    resends.
  - The `native-client` README limits `Unknown` to a Commit that was sent.

  Verified IDENTICAL to the tested head 5a8f5263. `@graphify good` is posted
  on the #577 thread.
- Portable P resume pointer (this file): #607 → 10091f5b, IDENTICAL to the
  tested head e41e1d0b.

## Open and blocked

- Owner decisions of 27 Nov 2026 (record:
  https://github.com/BaryonOS/BaryonOS/issues/592#issuecomment-7219530664).
  They grant GO for ADRs, implementation and verification within their
  limits; bounded technical choices need no further owner approval.
  - P01-T2 context mapping: an authenticated local manifest binds the full
    NAS namespace, the verified peer, the local volume, the user and the
    object. No truncated identifiers, no decryption authority or freshness
    floor for the NAS provider, encrypted names and indexes, B02
    seal/unseal, publication only after a durable ciphertext commit. Bounded
    chunking is allowed for the first native fixture only.
  - P02-T3 slice 3 (#628): GO for a separate, minimally privileged,
    encrypted local store so the guest offline queue survives a guest
    restart and reconciles via Head. No implicit replay of `Unknown`, no
    silent loss or overwrite, checkpoints bound to user, namespace, object,
    revision and authorization, revocation with the grant or identity.
  - P03 device authority and key lifecycle: fingerprint-verified first
    pairing, enrollment only by an authorized device with user intent,
    device-bound wrapping, sessions revoked before cleanup, rotation,
    recovery only via a device or an offline high-entropy secret, no
    escrow. The key hierarchy and freshness binding are agreed with B and N.
- In-boot NAS restart: parked by G (27 Nov, routing C). Restart stays across
  boots; in-boot recovery follows with the process-supervision track. Raise
  it again only if a P02 or P04 acceptance condition requires it.
- P02-T3 slice 3 status: design. The host invariant already exists
  (`restore` turns every unfinished edit into `NeedsReconcile`). The native
  store needs a guest volume grant (changes `CLIENT_AUTHORITY` in
  `kernel/src/m2/m3/nas.rs` and `CONTRACT.md`, so an ADR first) and a
  keyring in the NAS probe. Open questions to B on #628: record size (the
  8464-byte checkpoint against the 128-byte seal limit), the holder of the
  key-state export secret (slot 0 is the NAS administrator, who holds no user
  keys) and full NAS identifiers in `RecordContext`.
- Waiting on dependencies:
  - B02-T3 cold rollback anchor (ADR on #631): the NAS lineage/epoch anchor
    gets its own anchor domain with separate authority (confirmed by P);
    B proposes the domain id in the B02-T3 implementation PR.
  - P01-T3: B02-T1/T3 and N03-T3 (TLS peer to NAS principal).
  - P03-T2: implementation follows the P03 decision, with B and N.
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

1. P02-T3 slice 3 (#628): after B answers, write the ADR for the guest
   store, then the native wiring.
2. P01-T2 and P03: design and ADRs with B and N within the owner decisions.

Handoff threads: #592 (live HANDOFF), #554 (P02-T3). Update
the live handoff before an ownership change or a stop.
