# B03 policy transition — proposed synthetic experiment

Date: 14 November 2026. Baseline: `1260de8b5b0bb5330422ab9d5a63a64f5e2603b7`.
Status: **SYNTHETIC EXPERIMENT APPROVED under ADR-B03K3; implementation in progress; no production authority.**
Parent: [D1–D7 decision proposal](B03-PRODUCTION-POLICY-PROPOSAL.md).
Tracker: [issue #41](https://github.com/BaryonOS/BaryonOS/issues/41).

This specification records the bounded experimental design approved by the
project owner under [ADR-B03K3](../DECISIONS.md#adr-b03k3--authorised-synthetic-policy-transition-experiment).
The accompanying [symbolic check](../verification/b03-policy-transition-20261114/model.py)
explores a finite abstraction of the persistence rules. It does not implement
the formats below, verify signatures, execute Baryon OS, or establish a storage
backend's guarantees. ADR-049, ADR-B03K1 and ADR-B03K2 remain unchanged. [1–3]

## 1. Approval scope and unresolved prerequisites

The received approval is limited to a separate **synthetic-key QEMU
experiment** with a fixed TEST root role, a distinct TEST release role, the
formats and state machine below, and negative native tests. Neither an existing
TEST seed nor a new experimental seed may acquire production authority.

| Decision | Concrete proposed disposition | Still required |
|---|---|---|
| D1 | Separate fixed root and release key pairs in the experiment; no production private material. | Production custodians, human approval process, backup and loss procedure. |
| D2 | Bounded, domain-separated policy and image-binding envelopes; authenticate both signatures independently in loader and service. | Exact native ingress/capability contract before integration; production activation remains unresolved. |
| D3 | Explore an abstract atomic anchor isolated from the replaceable update disk. | A concrete backend, its failure semantics and demonstrated rollback resistance. A host-controlled test anchor is only a fixture. |
| D4 | Commit only a fully verified replacement; retain revocation tombstones; never recover by decreasing a floor or restoring revoked authority. | Native cut/recovery evidence and the separate production recovery contract. |
| D5 | Keep the experimental root fixed. | Separate B04 firmware-to-loader authentication, loader rollback protection, initial enrolment and root-loss recovery. |
| D6 | Keep authenticated release-version windows. RTC changes confer no authority. | Any later calendar-expiry or update-freshness mechanism. Offline revocation delivery remains an explicit limitation. |
| D7 | Approve the synthetic experiment separately. | Separate authorisation for real keys, production signing, release publication or physical-device changes. |

The experimental dispositions are approved under ADR-B03K3. The production
prerequisites in the last column remain unresolved. Native adapter/anchor
encoding/ingress details must be fixed before their integration. Approval of
the synthetic experiment does not approve the outstanding production choices.

## 2. Proposed canonical messages

All integers are unsigned little-endian. There are no optional fields,
alternate encodings, trailing bytes or nonzero reserved bytes. Bounds and
checked length arithmetic precede allocation or signature verification.
`SHA-256` and the existing Ed25519/ML-DSA-65 implementations are reused;
no new cryptographic primitive is proposed. Existing manifest/slot formats
retain their current meaning. [2, 4]

### Policy body and signatures

| Offset | Bytes | Meaning |
|---|---:|---|
| 0 | 8 | Magic `B03POL01`; not `NOSMAN01`. |
| 8 | 2 | Format version, exactly 1. |
| 10 | 2 | Signer count `n`, 1–8 including revoked entries. |
| 12 | 4 | Body length, exactly `112 + 2008*n`. |
| 16 | 16 | Exact target/profile identifier; experimental value `B03-QEMU-TEST01\0`. |
| 32 | 8 | Policy version, positive and strictly increasing for a new transition. |
| 40 | 8 | Root-authorised minimum image version, positive. |
| 48 | 32 | Digest of the immediately preceding policy body. |
| 80 | 32 | Fixed root identifier: SHA-256 of `B03ROOT1` followed by the full Ed25519 and ML-DSA-65 public keys, in that order. |
| 112 | `2008*n` | Complete ordered signer table. |

Each 2,008-byte signer entry contains the full 32-byte Ed25519 public key,
the full 1,952-byte ML-DSA-65 public key, an 8-byte first release version,
an 8-byte last release version, a one-byte revoked flag (0 or 1), and seven
zero bytes. Windows are inclusive, start above zero and are nonempty.
Entries are strictly ordered by the concatenated full public-key bytes.
Duplicate full pairs and duplicate pairs of the existing eight-byte key IDs
are rejected. The current key-ID derivation remains unchanged. [4]

Exactly 64 Ed25519 signature bytes and 3,309 ML-DSA-65 signature bytes follow
the body. Both signatures cover the same complete body using the existing
library verification operations. Maximum envelope length is 19,549 bytes.
The root identifier is compared with the independently pinned root; a key
provided by the package cannot choose its own root. A release pair cannot reuse
either public-key component of the fixed root pair. The policy digest is
SHA-256 of the canonical **body**,
excluding signature bytes, so another valid signature encoding of the same
body does not create a different policy identity.

The initial experimental policy is an explicit trusted fixture with zero
predecessor digest. This is not a general enrolment path. A successor must
name the committed policy digest, even when its version skips numbers.
Version overflow, unknown targets/versions and ambiguous tables fail closed.

### Binding the image to its intended policy

The existing signed 3,500-byte `NOSMAN01` manifest is retained unchanged.
The new experimental bundle additionally carries an 88-byte message:
`B03BND01` (8), target identifier (16), policy-body SHA-256 (32), and exact
manifest SHA-256 (32). The same release pair signs that message with both
algorithms; the 64-byte and 3,309-byte signatures follow in that order.

Acceptance requires the policy's two root signatures, the binding's two
release signatures, the existing manifest's two signatures, the policy
release window, and the exact image length/digest and ELF-loadability checks.
The release pair in the binding must equal the manifest's policy-selected
pair. A standalone v1 manifest cannot authorize a policy transition; a policy
signature cannot be replayed as an image signature. Manifest v1 and current
TEST entry points are not silently reinterpreted. [2, 4]

The symbolic check does not implement the native parser, signature integration,
resource budget or cross-format tests. Implementation evidence is recorded
separately as each layer is completed.

## 3. Trusted anchor and exact operation identity

The proposed persistent anchor `A` contains a format/target, a non-wrapping
revision, committed policy version and body digest, monotonic image floor,
and the last committed transition identity. Its canonical representation
must be fixed with the selected backend before native implementation.
Missing, unreadable, torn or conflicting anchor state is not initial state.

The transition identity binds a fixed `B03TXN01` domain, the complete expected
predecessor anchor, target, candidate policy version/digest, candidate image
version/manifest digest/image digest/length, and destination slot. These are
fixed-width fields; the backend specification must freeze their encoding.
Identity means the full tuple or its domain-separated SHA-256 digest, not a
policy number, slot number or disk header alone.

- A new transition requires the exact current predecessor anchor, a higher
  policy version, the correct predecessor policy digest, and a verified image
  permitted by the candidate policy and version at least
  `max(old_floor, candidate_policy.minimum_image_version)`. The candidate
  image version must also exceed the independently established running version,
  preserving the existing update-service rule. Same-version policy-only rotation
  is outside this first experiment and needs a separate decision.
- An equal policy version with a different body digest is equivocation and
  is rejected. An older policy is rejected even if correctly signed.
- An already committed request is idempotent only when its complete identity
  equals the anchored last transition. A different image or slot under the
  same policy is a different operation, not an interrupted-operation resume.
  An idempotent acknowledgement does not reinstall that image, lower a later
  boot floor or authorize a handover that current policy/floors would reject.
- Subsequent image-only updates are a separate operation under the committed
  policy. They cannot bypass the current downgrade rules or mutate policy.

### Required backend contract — an assumption, not an implementation

The backend must provide an atomic compare-and-swap of the **complete** anchor
against the expected predecessor. A mismatch cannot change any anchor field.
Successful readback must establish the exact durable successor. An interrupted
write may leave the exact predecessor or exact successor; an unsupported,
torn, unavailable or uncertain result stops activation. A disk journal or an
unprotected counter plus independent digest file does not supply this contract.

All anchor mutations, including boot-floor advancement, are serialized and
increment the revision without wrap. An old anchor cannot be restored under
the backend's declared threat model. D3 must demonstrate that property and
B04 must authenticate the code enforcing it. The symbolic model assumes both;
it deliberately exhibits the replay failure when the anchor assumption is
removed. No claim is made that a TPM, firmware variable, ordinary disk file or
existing emulator configuration supplies these properties.

## 4. Commit and restart protocol

The proposed first experiment has one serialized updater. The service verifies
and stages; the loader revalidates and is the sole anchor writer before kernel
handover. This division is proposed, not an existing production capability.
The running slot remains independently identified as required by ADR-049. [1]

1. Read the trusted anchor and its exact committed policy. Verify the complete
   incoming policy, binding, manifest, image and transition identity before
   the first persistent write. Invalid requests do not change active state.
2. Write the candidate policy and image/manifest/binding to separate staging
   storage and the non-running image slot. Preserve all objects needed to
   boot the committed state. Flush and independently read back every required
   byte; revalidate digests, signatures, windows and loadability.
3. Before kernel handover, re-read the anchor and revalidate the staged bytes.
   A predecessor mismatch or invalid/readback-failed candidate aborts without
   committing. Disk headers are hints only; they cannot select trust state.
4. Compare-and-swap the complete anchor to the new policy, last transition
   identity and `max(old_floor, candidate_policy.minimum_image_version)`.
   **This atomic durable anchor change is the policy commit point.**
5. Read the anchor back. Continue only with an exact known predecessor or
   successor; unknown, inconsistent or unreadable state stops boot. A lost
   acknowledgement alone is never interpreted as rollback or success.
6. Independently verify eligible boot images under the committed policy and
   floor. Prefer the committed transition's image; fallback is permitted only
   to another fully verified image under that same policy and floor. Before
   any handover, including fallback, require that image's valid binding to the
   **committed** policy digest. A binding to the predecessor policy is
   insufficient. An intended fallback therefore needs an additional signed
   binding to the candidate policy, durably stored without replacing its old
   binding or writing the running image slot before commit. Before
   handing over to a higher image version, durably advance/read back the
   image floor with a separate serialized anchor operation, retaining policy
   identity and last transition. A floor-write failure stops that handover.
7. Clean up obsolete staging only after the committed state is known. Cleanup
   neither changes authority nor permits deletion of the only eligible image.

The previously running kernel cannot continue across the policy commit in this
proposed loader-owned sequence. Every later handover, including an older
still-eligible fallback, is newly verified under committed state. A later
service-owned commit would need a separate quiescence and
handover argument. A native implementation must demonstrate that its actual
I/O and loader checks refine these rules; a Python state tuple is insufficient.

## 5. Required cut and refusal outcomes

| Interruption or fault | Required outcome |
|---|---|
| Before/during staging or its flush/readback | Anchor remains old. Old eligible image remains bootable; partial staging grants no authority. |
| Candidate validation, storage-full or write/readback failure | No anchor commit; report failure. Existing refused-inbox write-count checks remain required. |
| Compare-and-swap finds another predecessor | No commit by this request; re-read and fully reassess. Never overwrite the newer anchor. |
| Anchor write loses acknowledgement; readback is exact old state | Old policy remains authoritative; retry only after revalidation. |
| Anchor write loses acknowledgement; readback is exact new state | New policy is authoritative; identical retry is idempotent. |
| Torn, missing, unavailable or unexpected anchor/readback | Stop with an actionable recovery reason; do not infer authority from disk. |
| Commit succeeds, disk header remains old | New anchored policy governs selection; old header cannot undo revocation. |
| Old update-disk snapshot is restored after commit | No old policy substitution. Boot only if the new anchored policy and an eligible image are actually available; otherwise stop. |
| New image is missing/corrupt after commit | Use another image only if the committed policy and floor permit it; otherwise authenticated recovery remains required. |
| Image-floor write/acknowledgement fails | Re-read the complete anchor; no higher-version handover without a known durable floor. Never lower it for fallback. |
| Both trusted anchor and disk are restored | Outside D3's assumed guarantee; the model demonstrates that old authority can return. This must not be reported as rollback resistance. |

The finite model groups writes by their durable old/new/partial outcomes. It
does not enumerate byte-level storage/controller behaviour, all histories,
or concurrent native processes. Each native write/flush/anchor boundary needs
its own future interruption test and independent disk/anchor inspection.

## 6. Revocation, exhaustion and availability

For this proposed first experiment, revocation is permanent within the fixed
root lineage. Each successor retains every revoked pair as a revoked tombstone.
Neither removal nor unrevocation is accepted. No active pair in the candidate
table may share either public-key component with any candidate tombstone,
including a newly revoked pair; this also checks retained active entries.
Root-authorised recovery cannot
waive these rules. Later changes to this policy require a separate decision.

The eight-entry bound includes tombstones. When it is exhausted, reject the
transition; do not discard revocation history or silently enlarge the bound.
This is an explicit experiment limit, not a production lifecycle solution.
Long-term authenticated revocation history remains part of D3/D4 design.

A transition that revokes the old image's signer can remove its fallback.
The old image also needs its own binding to the new policy to remain a fallback,
even when its signer is retained. The replacement must be fully verified and
durable before commit, but later
loss of that sole eligible image may still stop boot. Availability is not
guaranteed by keeping an A/B slot. Recovery must restore an image eligible
under the committed policy/floor; if none exists, fail closed. The proposed
experiment must show both an allowed fallback and this refused fallback.

## 7. Acceptance sequence and evidence boundary

1. Experimental approval is recorded in ADR-B03K3; the unresolved production
   decisions remain explicit.
2. Specify the real native adapter, exact anchor/transition encoding and ingress
   rights before implementation. Preserve the existing TEST compile guards.
3. Implement parsers and cryptographic checks with malformed-input, role,
   target, size, equivocation and revocation tests. Run host tests first.
4. Preserve all existing 18 native update stages. Add independently inspected
   policy/image interruption, restart, replay, exhaustion and refusal cases
   under the selected experimental backend, with source-bound evidence.
5. Obtain the required independent review. D3 physical persistence, B04 boot
   trust and production custody/ceremony remain separate acceptance gates.

The current symbolic check supplies only a bounded specification exercise.
It cannot close #41 or award product points. No native transition case,
real signature for these new formats, physical device or production release
is accepted by this document.

## Sources

1. Baryon OS. (2026). [Architecture decisions](../DECISIONS.md), ADR-049,
   ADR-B03K1 and ADR-B03K2; inspected at the baseline above.
2. Baryon OS. (2026). [B03 signed updates](B03-SIGNED-UPDATES.md).
3. Baryon OS. (2026). [Project instructions](../../AGENTS.md) and
   [test strategy](../TEST-STRATEGY.md).
4. Baryon OS. (2026). [Update core](../../crates/update-core/src/lib.rs) and
   [pinned policy validation](../../crates/update-core/src/policy.rs), at the
   baseline above. Public-key/signature sizes, manifest length and key-ID
   derivation are implementation facts; the new envelopes and protocol are
   proposals, not existing behaviour or external security certification.
