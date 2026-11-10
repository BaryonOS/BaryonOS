# APP-SOURCE-SNAPSHOT-1 - exporting and reconstructing source state

30 October 2026; TASK-APP-SOURCE-SNAPSHOT-CONTRACT-1.
**DESIGN for coordination, not an existing API or implementation authorisation.**
Only this document is new. No verifier, issuer, backend, storage encoding,
anchor, capability or native recovery is delivered. Apps remains 2/8, total 34%.

## 1. Sources, task boundary and invariant

This design addresses only the private-registry boundary in
[APP-SOURCE-RECOVERY, section 1][recovery]. Sources are pinned separately:

| Source | Version and meaning |
|---|---|
| Apps core | [`dce3b05aa358f4dbde6f2349b426217779b40052`][core]: actual `SourceRegistry`, `SourceSlot`, `TrustAnchors`, `add/rotate/revoke/verify_executable` |
| SEC recovery contract | [`05784243d3d1ea273b588de82069596648fbde4d`][recovery]: actor, durable publication, Unknown, cold freshness and installer reauthorisation |
| Exclusive assignment | [`82664b56a03dad5f2cdd64314234c406dd74a299`][queue]: only this new documentation work item; no core, backend or frozen-PR writes |

The future interface must export and reconstruct every state reachable through
the existing core without loss: the same SourceIds, exact public key pairs,
exact generations, active/revoked status and tombstones.
It must not reject an existing valid state through stricter cryptographic policy.
Reconstruction provides **no evidence** that the state is authentic, authorised
or current. Only the future trusted adapter may actively publish a candidate
after the separate recovery checks.

This is a focused API agreement, not a byte ABI, definitive architecture ADR
or permission to add the specified functions now. The names below are
proposed Rust signatures for later coordination with the owners.

## 2. Existing semantics that must be retained

The current core contains at most `MAX_SOURCES = 8` slots. `add` uses an
empty slot, starts at generation 1 and rejects an existing SourceId or
identical complete `TrustAnchors` pair, including in revoked slots.
`rotate` replaces the current pair and increments the generation with overflow checking.
`revoke` increments the generation with overflow checking and retains the source
with its final key pair as a tombstone. A revoked source cannot be rotated,
revoked or added again. There is no delete/compact/regrant.

`valid_anchors` uses exactly the existing parse checks:
`EdKey::from_bytes(...).is_ok()` and
`EncodedVerifyingKey::<MlDsa65>::try_from(...).is_ok()`.
This is not an additional claim about key strength, nonzero values or
independent cryptographic validity. The snapshot interface will reuse
the same internal helper; it does not invent a second key validator.

Duplicate keys currently means equality of the **entire pair**, not that
each individual Ed25519 or ML-DSA component must be unique. A rotation removes
the previous pair from that slot state; the registry is not a complete key history.
A previously used pair that is no longer present anywhere is not retrospectively
prohibited. Retain this policy; stricter historical key exclusion requires
a separate explicit decision and different stored state.

`u32::MAX` is a valid stored final generation. A subsequent mutation is
rejected with `GenerationExhausted`; export or reconstruction must not reject,
increment, truncate or reset the existing final state to 1.

## 3. Proposed logical record and API behaviour

A logical `SourceRecord` contains only:

| Field | Meaning |
|---|---|
| `id: SourceId` | Exactly the 32 source-identity bytes, not a derived hash or display name |
| `anchors: TrustAnchors` | Exactly Ed25519 32 bytes and ML-DSA-65 1952 bytes; public keys only |
| `generation: u32` | Exact existing source generation, not service epoch or artifact generation |
| `state: Active / Revoked` | Explicit semantic state; Revoked remains an occupied slot |

No `Empty` record, private Rust layout, discriminant bytes, padding,
pointer, secret, verified flag or capability is included in this exchange.
Label/URL/licence, user domain, store identity and durable
commit/anchor binding are not existing `SourceSlot` fields. They remain
separate authenticated adapter metadata, associated with these records
under the SEC contract; the core must not invent them.

### Export

Proposed: `SourceRegistry::export_snapshot(&self, out: &mut [SourceRecord])
-> Result<usize, SnapshotError>`.

- Export all occupied slots, including tombstones, in existing slot order.
  An immutable borrow keeps this copy consistent with a single core state;
  the service serialises it with mutations.
- Return the exact number of records. Unused output elements are not part
  of the snapshot; do not read unwritten trailing bytes.
- An undersized buffer returns `BufferTooSmall` without partially changing
  the output or registry. Count/validate first, then copy. No truncated success.
- No newly generated IDs, key pairs or generations, and no lost tombstones.
  No registration, signing, I/O or actor-check side effects.
- Core export is not a durable ACK. Service export in Unknown may at most
  return explicitly diagnostic candidate data, never an authoritative
  recovery source or successful production export.

### Reconstruction

Proposed: `SourceRegistry::from_snapshot(records: &[SourceRecord])
-> Result<SourceRegistry, SnapshotError>`.

This function builds a **private candidate**; it neither replaces an existing
registry nor grants trust. The trusted service keeps the result outside its
active `verify_executable`/installer path until publication is permitted.
The current core has no Unknown or authenticated-provider state of its own:
`Ok(SourceRegistry)` therefore can never be treated as a trust token.

Complete prevalidation is mandatory:

1. At most `MAX_SOURCES` records; zero records is structurally valid for
   a registry that has never been configured, but is not an authorised rollback fallback.
2. Every SourceId has at least one nonzero byte; every keyset meets exactly
   the existing `valid_anchors` semantics.
3. No duplicate SourceId or duplicate complete key pair anywhere in the
   record set, irrespective of active/revoked state. No deduplication or last-record-wins.
4. Active has generation at least 1; Revoked at least 2, because the existing
   `revoke` increments an existing generation. `u32::MAX` remains valid
   for both; the future decoder already explicitly rejects unknown states
   or incompatible field widths.
5. Transfer all validated records into occupied slots in the specified
   order; initialise only the remaining slots as Empty.

Do not reconstruct by replaying public `add` and repeated `rotate/revoke`:
that could distort generations/keys, create temporary verification rights
and perform unnecessary work. In the later core implementation, construct
from validated semantic fields; no transmute, raw memory or layout copying.
No partial result on error. The active registry and any export output buffer
remain unchanged.

Proposed concrete structural errors are `BufferTooSmall`, `Full`,
`InvalidSourceId`, `InvalidKeys`, `DuplicateSource`, `DuplicateKeys`,
`InvalidGeneration` and `InvalidState`. The precise Rust error representation
remains subject to owner coordination; no silent defaults or success-shaped
error substitution. `Stale`, `Unknown`, `Unauthenticated`, `WrongDomain` and
`Rollback` are separate adapter/recovery outcomes, not facts proven by this core.

## 4. Freshness, publication and existing active state

A structurally valid snapshot remains untrusted input. The adapter checks
actor/source-management domain, store binding, authenticated complete commit
content and independent freshness under [SEC sections 3-7][recovery].
Caller generations, service epochs, hashes or successful parsing do not replace
any of those checks. A signed old list can still constitute rollback.

Live replacement with a known previous active state additionally requires:

- No existing SourceId or tombstone may disappear; there is no deletion policy.
- A lower generation is rejected. An equal generation requires the same
  key pair and status, even if the overall snapshot label is newer.
- An active source at a higher generation may have a different active state
  after rotation or be Revoked after revocation. An identical current pair
  at a higher generation is not automatically invalid: intermediate rotations
  may have returned to that pair.
- An existing Revoked record must remain exactly equal. A higher generation
  or reactivation of that tombstone does not exist in the current mutation
  semantics. New sources may appear only as additions confirmed by the
  outer authority.
- These comparisons do not prove any missing intermediate history.
  A higher generation also requires the complete outer authority check.

At cold boot, the old RAM state is absent. That does not make an empty or older
snapshot acceptable: the independent freshness/lineage binding must detect
missing sources, tombstones or rotations. If that evidence is absent,
recovery remains Unknown/blocked; no `SourceRegistry::new()` as a success
fallback and no production installation/update using a candidate.

The complete candidate, adapter metadata and authenticated commit must switch
under a single service-publication boundary. On error, the previous known
state is retained; if durable publication may have occurred, the affected
scope remains closed until authoritative reconciliation. The exact crash/anchor
protocol remains open and is not delivered by this in-memory API.

An export is a copy, not a lease on current authority. Rotate/revoke after
export or package verification requires reauthorisation and serialisation
with installation commit. An old `SourceVerifiedExecutable` must not bypass
a generation check. Source revoke blocks new verification/update, but does
not remove existing apps/data or change their separate launch policy.

## 5. Mandatory subsequent focused negative tests

These are acceptance cases for a future implementation, **not tests
executed** in this documentation work item.

| Case | Required result |
|---|---|
| Empty, one source, full registry, mixed active/revoked | Export/reconstruct/export preserves all records and their order exactly; identical subsequent `add/rotate/revoke` outcomes |
| Generation 1 active, 2 revoked, both at `u32::MAX` | Valid final states are retained; mutation at max is rejected without changes |
| Return to an earlier pair after several rotations | Current valid state remains acceptable; no invented historical key ban |
| Different pairs sharing one key component | Same acceptance as the existing core; no stricter component uniqueness |
| Zero/one-too-small buffer; output larger than necessary | Error without a partial write; success counts only written records |
| Zero ID, malformed keys, duplicate ID/pair including a tombstone | Explicit error; no repair, skipping or deduplication |
| Active generation 0; Revoked 0/1; unknown state | Explicit error; no automatic generation/status conversion |
| Invalid final entry after a valid prefix; too many records | No partial candidate or active replacement |
| Same generation with changed keys/status; missing tombstone | Adapter rejects; active snapshot unchanged |
| Lower generation; Revoked to Active or a higher Revoked generation | Adapter rejects under the retained mutation policy |
| Valid parse but wrong actor/domain/commit/anchor | No publication or new package acceptance |
| Authentic old list; cold start without freshness evidence | Rollback/Unknown, no empty list or service epoch as a substitute |
| Crash during restore/publication; only a prefix read | No partial active registry, no success ACK |
| Rotate/revoke between export/verification and installation commit | Reauthorisation; old snapshot/token does not bypass current source state |
| Source revoke with an already installed app | New install/update rejected; existing separate launch policy retained |

## 6. Delivery and remaining open boundaries

Document review and consistency checking are not compilation, host testing,
native execution, durable recovery or a human audit. This design closes no
criterion; the central progress owner retains 34 -> 34.
Core export/reconstruction, actor issuer, backend, rollback anchor, decoder,
authoritative adapter and installation/launch wiring remain unimplemented
by this work item. No automatic subsequent claim on those write scopes.

Implementation requires separate owner coordination, code authorisation,
the negative tests above and the prescribed native/durable gates.
A scoped SEC interface review must check this concrete frozen document
against the existing recovery contract; it does not replace an independent human audit.
Reverting this document alone changes no storage or runtime behaviour.

[core]: https://github.com/simonvanieperen/nieuw-os/blob/dce3b05aa358f4dbde6f2349b426217779b40052/crates/app-package-core/src/lib.rs
[recovery]: https://github.com/simonvanieperen/nieuw-os/blob/05784243d3d1ea273b588de82069596648fbde4d/docs/contracts/APP-SOURCE-RECOVERY.md
[queue]: https://github.com/simonvanieperen/nieuw-os/blob/82664b56a03dad5f2cdd64314234c406dd74a299/docs/WORK-QUEUE.md
