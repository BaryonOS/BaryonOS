# B03/B04 production signing policy — decision proposal

Date: 14 November 2026. Source baseline: `b8a7e83658bdbcd4970c26f753dda1b342fce0d4`.
Status: **PRODUCTION PROPOSAL — UNAPPROVED; bounded synthetic experiment approved under ADR-B03K3.**
Tracker: [issue #41](https://github.com/BaryonOS/BaryonOS/issues/41).

This document makes the remaining release-policy choices reviewable. It does not
change an ADR, grant signing authority, create keys, enable a production profile,
approve firmware changes or close B03/B04/#41. The Security Floor and explicit
TEST-feature guards remain binding. B03's existing points are unchanged. [1–4]

The [transition specification](B03-POLICY-TRANSITION-PROPOSAL.md), prepared on
`1260de8b5b0bb5330422ab9d5a63a64f5e2603b7`, now makes the canonical envelopes,
exact operation identity, anchor commit/readback, revocation history and cut
outcomes concrete for review. Its symbolic model is a specification exercise,
not a native or production implementation. D1–D7 production prerequisites remain
unresolved. After the initial review, the project owner explicitly
approved publication and the TEST/QEMU experiment; ADR-B03K3 records that limited
authorisation. Native implementation and its evidence are separate checkpoints.

## Existing boundary

The current policy accepts at most eight exact Ed25519/ML-DSA-65 pairs, checks
release-version windows and rejects revoked pairs. TEST successors are pre-pinned;
policy distribution, policy rollback protection and trusted calendar expiry are
absent. ADR-049 assigns production rotation/revocation to B04; newer ADR-B03K1 and
ADR-B03K2 explicitly retain the additional #41 requirements. [1–3]

The existing TEST workflow passed on the source baseline in
[GitHub Actions run 43594560820](https://github.com/BaryonOS/BaryonOS/actions/runs/43594560820).
That result verifies the existing implementation under its gate, not this
proposal, production keys or a production release chain.

## Recommended bounded route

Specify a separate, versioned production policy envelope and verify it with an
offline root role. Delegate image signing to a distinct release role. Reuse the
existing hybrid verification and bounded signer-table concepts; do not invent a
cryptographic primitive. Start with synthetic keys and disposable QEMU state.
Production activation remains disabled until B04 and persistent rollback
protection have separately demonstrated the assumptions below.

The proposed envelope binds its format version, target/profile, policy version
and complete signer table in one canonical message verified by both root
signatures. Image-signing keys cannot change the trusted root, authorize their
own successor or replace the policy. An explicit format specification must bind
the image bundle to its intended policy before implementation; existing manifest
v1, TEST fixtures and on-disk layouts are not silently reinterpreted.

## Decisions requiring approval

| ID | Recommended choice | Required decision or prerequisite |
|---|---|---|
| D1 — roles and custody | A distinct offline root pair authorizes policy; a separate release pair signs images. Private material stays outside the repository, CI, prompts and logs. CI supplies unsigned candidate artifacts and hashes. | Name custodians, approval records, controlled signing environment, backup and loss procedures. Two signature algorithms do not constitute two independent human approvers. |
| D2 — policy distribution | Accept bounded, root-authenticated policy packages through an explicit update/recovery ingress. Distribution media and network transport confer no trust. An installed policy version identifies one exact policy digest. | Specify canonical bytes, size bounds, target binding, parser failures and the production ingress's existing capability or separately reviewed new contract. |
| D3 — persistent rollback protection | Bind the committed policy version and digest to a persistent state mechanism whose rollback resistance is demonstrated for the supported profile. | Select and validate a concrete backend and threat model. Ordinary disk files, duplicate slots and the current writable-OVMF assumptions are insufficient against whole-state replacement; no TPM or commodity monotonic store is presumed available or suitable. |
| D4 — revocation and recovery | The offline root authorizes release-key replacement and revocation. Recovery accepts only images and policy permitted by committed trust state; it cannot waive signatures, restore revoked authority or lower security floors. | Define recovery after a lost release key, compromised release key, interrupted transition and unavailable valid image. If all root authority is lost/compromised, automatic recovery is not claimed; a separately authenticated B04 re-enrolment procedure is required. |
| D5 — root and boot chain | Keep the initial root fixed in the first proposed profile; authenticate the loader and its embedded root through a defined firmware-to-loader chain. | B04 must specify initial enrolment, loader verification, loader/policy rollback resistance and root replacement/recovery. Host approval or a QEMU boot alone cannot establish that chain. |
| D6 — time and freshness | Retain authenticated release-version windows for the first revision; do not interpret RTC time as trusted expiry. | Document that an offline client learns a new revocation only when it receives an authorized policy. Calendar expiry or protection against indefinitely withheld updates needs a separately justified freshness/time mechanism. |
| D7 — activation scope | Approve specification and synthetic-key implementation separately from real key generation, production signing and release publication. | Record which decisions are approved. An accepted proposal alone authorizes none of those consequential operations or physical-device changes. |

## Required transition invariants

1. Read and validate committed policy state before selecting a candidate. Reject
   an older policy, an equal version with a different digest, an unknown root,
   malformed tables and any incomplete hybrid signature. A same-version,
   same-digest package may only resume the identical interrupted operation.
   Identity also binds the predecessor anchor, target, image version/digests,
   length and destination slot; policy identity alone is insufficient.
2. Verify the candidate policy and image before activation. A policy that revokes
   the running release key requires an eligible replacement or a separately
   authenticated recovery route; staging bytes alone grants no authority.
3. Specify the persistent commit protocol jointly with D3. Power loss at every
   write/flush/anchor transition must produce one documented policy/image state.
   Never silently lower a committed policy floor to make an older image boot.
4. Retain fallback only to an image permitted by committed policy, the image
   version floor and the authenticated boot chain. If no such state exists,
   fail closed with an actionable reason; an unverified recovery path is not used.
5. Loader and update service enforce the same policy semantics independently.
   The microkernel acquires no key material, policy parser or new implicit right.
   Ordinary apps cannot mutate trust state or obtain update/signing authority.

These are proposed acceptance requirements, not claims that an atomic protocol,
persistent backend, recovery UI or production ingress currently exists. [1–4]

## Synthetic-key native acceptance matrix

All rows below are **NOT RUN** and require a frozen source version, isolated disk
and firmware state, exact binaries/configuration and independent state inspection.
Retain the existing 18-stage `tools/test_update.py` gate; extensions must preserve
its refusal, write-count, cold-boot and interruption oracles. [2, 5]

| Case | Required native observation |
|---|---|
| Authorized successor | Root-approved replacement policy and release image activate; cold boot uses the intended policy digest and eligible image. |
| Unauthorized policy | Wrong root, one missing/invalid signature, wrong target, altered bytes, oversize and duplicate signer entries grant no authority or active-state change. |
| Role separation | A valid release signature cannot authorize a policy, self-rotation or a root change; ordinary app requests remain denied. |
| Policy replay/equivocation | Older policy and equal-version/different-digest policy are rejected, including after cold boot and whole-disk replacement under D3's declared threat model. |
| Interrupted activation | Cuts at each durable transition recover only the specified valid state; neither old revoked authority nor a partially written policy becomes active. |
| Revoked/compromised release key | Both loader and service reject newly signed and replayed images from that key under the committed revocation; permitted recovery remains available. |
| Recovery and exhaustion | Missing/corrupt policy, storage/full-write failure, unavailable anchor state and no eligible image fail closed without weakening security floors. |
| Time semantics | Untrusted RTC changes do not alter release-window authorization; withheld revocation remains a documented freshness limitation. |
| Boot-chain substitution | Replaced loader/root or rolled-back policy-bearing loader is rejected by the separately specified B04 chain; ordinary OVMF execution is insufficient evidence. |
| TEST separation | Production-profile builds reject public TEST anchors and absent policy configuration; current TEST profiles remain explicit and reproducible. |

Host tests precede native tests. QEMU establishes only the documented emulator
model; the selected physical persistence/firmware boundary and independent
release-chain review retain their own evidence requirements. No score changes
follow from drafting this proposal, increasing test counts or passing host tests.

## Review outcome and sources

The D2/D3/D4 transition specification is linked above. ADR-B03K3 records the
bounded synthetic-experiment approval and separates it from outstanding
D1–D7 production prerequisites. The exact native backend/encoding/ingress contract
must precede native implementation. Unresolved decisions leave production
activation disabled. Approval and resulting architecture changes belong in
`DECISIONS.md`; this proposal is not a second roadmap or an approved contract.

1. Baryon OS. (2026). [Architecture decisions](../DECISIONS.md), ADR-049 decision 6, ADR-B03K1 and ADR-B03K2.
2. Baryon OS. (2026). [B03 signed updates](B03-SIGNED-UPDATES.md), §2 invariants 7–8, §7 and B03-KEY-POLICY-1 evidence boundary.
3. Baryon OS. (2026). [TEST anchors for B03](../../tools/update-keys/README.md), policy table and production/time limitations.
4. Baryon OS. (2026). [Project instructions](../../AGENTS.md), Security Floor and Execution; [criterion register](../progress.json), B03/B04.
5. Baryon OS. (2026). [Native signing-key lifecycle evidence](../verification/B03-KEY-POLICY-1.md), native outcomes and open boundaries. Historical evidence does not verify this proposal.
