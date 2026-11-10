# APP-SOURCE-RECOVERY-1 — durable publisher authority and recovery

30 October 2026; TASK-SEC-PUBLISHER-RECOVERY-CONTRACT-1.
**Adapter contract; no implemented issuer, backend, storage format,
capability or cold-recovery acceptance.** Base
`ca45a7dabe6a040921de03bdc2fad3e5045a75ba`.
Objective: connect the existing app verifier to authorised, durably bound
source state without building a second verifier/catalogue or new cryptography.
Rubric B04 and the associated app/release criteria remain open.

## 1. Existing interfaces and separate source versions

| Building block | Source | Available, but not delivered |
|---|---|---|
| `app_package_core::SourceRegistry`, `SourceVerifiedExecutable` | #73 `dce3b05aa358f4dbde6f2349b426217779b40052`, `crates/app-package-core/src/lib.rs` | Hybrid verification and live add/rotate/revoke; no actor authorisation or persistence |
| `PackageIdentity`, `catalog::InstallCatalog` | #80 `bc27ed0f20097a68c9bfdfeecc8867c7e7721211`, `sdk/app-management/src/{lib,catalog}.rs` | Provenance metadata and install/update/remove model; no issuer or durable journal |
| `ArtifactStore`, `ArtifactDescriptor`, `open_checked` | #78 `7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb`, `crates/artifact-store-core/src/lib.rs` | Immutable byte generation/read contract; no publisher trust or backend |
| `records::RecordContext`, `Service::{export_persistent,restore_persistent}` | `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`, `services/keyring/src/{records,persistent}.rs` | AEAD context and key-state building blocks; no independent cold rollback anchor |

These are separate work items, not a jointly integrated release image.
Apps owns the verifier/catalogue/protocol; release owns the store/backend,
authorised-image/kernel-copy/dynamic-launch integration. SEC owns this
source-recovery contract. Changes to shared/frozen code require its
owner; this document does not change that code.

`SourceRegistry::add(id, anchors)` creates generation 1.
`rotate(id, expected_generation, anchors)` and
`revoke(id, expected_generation)` require the current generation, increment
it with overflow checking and retain revoked slots as tombstones.
`verify_executable(id, expected_generation, package, current_abi)` rejects
unknown, revoked or stale sources and returns `SourceVerifiedExecutable`
only after the existing package/ELF verification.
This core has no public export/import interface for its private source list:
calling the current crate alone therefore does not provide a durable adapter.
Any controlled reconstruction/snapshot interfaces must be designed later with
Apps; copying the private layout or skipping revocations is prohibited.

## 2. Security invariant and source-management actor

A successful new installation/update may result only from:
an authorised management decision, current durably confirmed source state,
the existing package verifier and a separately authorised installation action.
A package, catalogue descriptor, download server or store locator cannot
issue a trusted source, keyset, epoch or grant.

A future source-management service receives a separate capability for
add/rotate/revoke. It binds every operation to the actual kernel sender,
user/policy authority, intended source domain and current
service instance. A payload PID, SourceId, `verified` boolean, UI text or
public constructor is not evidence of actor identity. The concrete issuer/bootstrap
capability has **not yet been specified or implemented**.

Installation grants no source-management right or general file, NAS, network,
key or launch authority. Automatic rechecking within an existing grant is
permitted; a new source or different trust authority requires an explicit
authorised decision. No regrant after Unknown.
App-publisher anchors remain separate from B03 release anchors, TLS identity
and B02 user keys.

## 3. State requiring durable binding, without a new byte ABI

The future storage transaction binds at least:

- the authorised source-management domain and store/volume identity;
- SourceId and the exact Ed25519/ML-DSA-65 pair;
- the checked source generation and active/revoked status, including tombstones;
- the expected previous state and new authenticated commit identity;
- sufficient authenticated recovery metadata to distinguish an uncertain
  mutation from unpublished work.

This is a semantic invariant, not a selected encoding, KDF, key-ID scheme,
journalling algorithm or cryptographic protocol. Use existing reviewed
cryptography only through an authorised adapter. A checksum or hash beside
a file does not make that file authoritative.

| Counter/identity | Meaning | Not a substitute for |
|---|---|---|
| SourceId | Source identity within an authorised management domain | Actor grant or content digest |
| Source generation | Sequence of key rotations/revocations for that source | App version or cold-boot anchor |
| App version | Authenticated package version | Source trust or access right |
| Immutable store generation | Exact stored bytes | Publisher epoch or key lifecycle |
| Service epoch | Separates live instance/request handles | Durable revocation history |
| Durable commit/anchor binding | Which source state is demonstrably current | Success boolean or separate rollback file |

No wraparound, history reset to zero or tombstone reuse.
Exhaustion rejects the operation before publication; a new domain is not
a silent workaround.

## 4. Mutation, durable ACK and linearisation

The following steps are mandatory adapter behaviour, not an existing service:

1. Authenticate the actor and grant; validate the request, source and expected
   generation before storage or cryptographic work. Serialise conflicting trust mutations.
2. Calculate/validate a candidate using the same rules as `SourceRegistry`,
   without making an advance change visible in the active registry. Preparing
   a private candidate is not equivalent to an active commit.
3. Record the intent/candidate and recovery binding according to the
   crash-consistent backend/anchor protocol that remains to be selected. Do
   not issue a durable ACK based on RAM, enqueueing, a successful write without
   the required flush, or caller-supplied commit evidence.
4. Publish the new source state and only then return success to the actor when
   the concrete backend and independent rollback anchor have confirmed their
   required commit guarantee. That protocol must define the exact
   linearisation point; two independent writes are not automatically atomic.
5. On revocation, the adapter closes new install/update acceptance for that
   source before the mutation entry point reports completion. If uncertainty
   arises after publication, that source remains closed to new acceptance
   until authoritative reconciliation, even if an older active snapshot exists.

A new rotation or revocation must cause already verified but uncommitted
install/update work to be reauthorised. Retaining only an old
`SourceVerifiedExecutable` to bypass the current source-generation check
is insufficient. Do not stop other independent sources globally unless the
backend demonstrably has no narrower safe boundary.

## 5. Crash windows and Unknown

| Window / outcome | Permitted conclusion | Required subsequent boundary |
|---|---|---|
| Before publication, demonstrably nothing issued/stored | No trust mutation confirmed | A new explicit request is possible; do not fabricate an ACK |
| Intent written, candidate not committed | Incomplete | Recovery validates the entire intent/commit chain; do not publish a partial registry |
| Candidate data durable, independent anchor not confirmed | Not demonstrably current | Unknown, source acceptance closed; do not trust the data hash alone |
| Anchor may have advanced, data/commit outcome uncertain | Unknown | Do not fall back to an older active state; reconcile or return a recoverable error |
| Commit completed, response lost | May already have executed | Investigate the outcome using the authoritative commit identity; no automatic mutation replay |
| Revoke durable, old cache/installer still active | Old acceptance is invalid | Cache/installer reauthorises; no new installation using the old generation |
| Cold boot without independent freshness evidence | No cold antirollback | An old authentic source list is not proven current; no production acceptance |
| Corrupt, conflicting, missing or wrong domain | No valid state | Closed failure with diagnostics, no empty Default registry as a success fallback |

Unknown is neither a successful revocation, installation or flush nor
permission to execute again. An authenticated snapshot must not authorise a
lower source generation, lost tombstone or reverted keyset.
A service epoch after reboot is not evidence that the stored list is current.

## 6. Cold recovery and anchor dependency

The recovery agent authenticates the complete source state, management domain
and store binding, checks them against independently trusted freshness and only
then publishes a complete registry. Reconciliation may derive only from that
authority, not from an app metadata record, stored `PackageIdentity`, previous
success message or snapshot that the same attacker can roll back.

A TPM-NV counter fixture on a different source version does not prove the
required source-state/lineage binding. B02 `minimum_generation=0` accepts
authenticity without guaranteeing cold freshness. B02 restore, firmware
release floor and artifact generation must not be repurposed as new app-publisher
anchors. A real anchor provider, durable backend, write-ahead/commit
coordination and reconstruction API are **OPEN**. Without those dependencies,
no positive production cold-recovery claim can be made.

## 7. Installer consumption and already installed apps

The installer binds authorisation to the exact SourceId, current source generation,
app ID, authenticated package version, ABI, image length and digest. It uses
the existing `SourceRegistry::verify_executable`, not a second verifier or
a publicly self-constructible verified token.

`PackageIdentity` is metadata; the immutable artifact descriptor proves only
byte binding after actual complete verification. Neither grants an installation,
source-management or launch capability. Recheck current source state and the
installation grant before commit; revoke/rotate after verification creates no
silent exception. The concrete transaction must serialise this check with
publication to prevent a check-then-commit race.

A catalogue commit follows only actually committed exact artifact bytes and
the authorised installation action. Store-Unknown makes the catalogue uncertain
under the existing catalogue rules; authoritative recovery is not a higher
generation number supplied by the caller.

**Existing policy remains:** revoking a source blocks new package verification
and updates, but already installed apps remain launchable; no silent deletion
of apps or user data. A valid installed artifact retains its existing separate
launch policy. This exception must not be used to label a new package as
already installed. Revoking an individual launch/network/object grant is a
different operation and remains enforceable.

## 8. Minimum implementation and acceptance boundary

Before a production claim, the following are separately required: an authorised
source-management actor, durable backend/anchor provider, controlled registry
reconstruction and installer/launcher wiring by their respective owners.
None is delivered by this document. The existing fixed core bounds are
experimental bounds, not product limits.

Required focused negative tests of the eventual implementation:

- wrong actor/domain/caller and direct package/store payload treated as authority;
- stale generation, wrong keyset, duplicate source/keys, overflow and tombstone reuse;
- rotate/revoke between verification and installation commit;
- every crash window above, a lost response and Unknown without replay;
- authentic old snapshot, anchor/data mismatch and cold boot without an anchor;
- correct source revoke: reject new install/update, retain launchability of an existing app;
- reject artifact substitution or catalogue reconstruction from unauthorised metadata.

A host model, cross-compilation, AI review and documentation do not establish
native execution, durable recovery or a human audit. Source/artifact binding
and required QEMU/human gates remain separate; this contract awards no points.
Apps and release have each performed one focused read-only interface review:
no concrete interface/policy defect was found within their existing
SourceRegistry/catalogue and store/launch boundaries respectively. This is not
a broad audit, new test execution or production acceptance.
Reverting this document alone requires no data migration and changes no code.

## 9. Project references

- [B02 records and key state](../implementation/B02-SECURE-RECORDS.md)
- [Privacy contract](../LOCAL-PRIVACY-CONTRACT.md)
- [`APP-PACKAGES.md` at the Apps review source](https://github.com/simonvanieperen/nieuw-os/blob/dce3b05aa358f4dbde6f2349b426217779b40052/docs/implementation/APP-PACKAGES.md)
- [`ARTIFACT-STORE-CONTRACT.md` at the root-store source](https://github.com/simonvanieperen/nieuw-os/blob/7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb/docs/implementation/ARTIFACT-STORE-CONTRACT.md)
- [Shared cryptographic authority on a separate SEC work item](https://github.com/simonvanieperen/nieuw-os/blob/c5fbb21a0e46e010d5e9b4748ae9b9307a67cef4/docs/contracts/NATIVE-CRYPTO-AUTHORITY.md)
