# NAS sealed-record consumer seam

**PROPOSED — no NAS key grant, new trust root, escrow or wire change is adopted.**
Baseline main `69d82075`. This is a consumer contract for existing cryptography;
it is not an implemented encryption path or new native acceptance.

## Existing pieces and missing join

[Keyring records](../../services/keyring/src/records.rs) provide `RecordContext`,
`RecordKind`, `Service::seal_record` and `Service::unseal_record`. The context binds
volume, domain, object, revision, chunk, exact plaintext length, kind and schema.
The caller domain and expected context must come from authenticated ingress/state,
never from ciphertext. The [KEY1 codec](../../services/keyring/src/wire.rs) encodes
that context; [B02](B02-SECURE-RECORDS.md) records the existing allocation/revocation
and cold rollback limits.

The [NAS core](../../services/nas/native-core/src/lib.rs) accepts opaque ciphertext
through `Service::stage`/`commit`/`read`; administrator `revoke`/`evict` needs no guest
content key. `Principal` is a verified fingerprint supplied by trusted ingress,
not identity established merely by possessing its bytes. The
[NASP codec](../../services/nas/native-probe/src/lib.rs) has a 104-byte data field.
It is a bounded native fixture, not an established production sealed-record ABI.

## Proposed ownership and context mapping

Only an authorized client-side consumer holds access to its Key-Service domain.
The administrator/provider stores opaque bytes and manages space; neither gets
that domain key, a recovery snapshot capability or a decryption service on behalf
of the guest. Names, manifests and sensitive indexes are encrypted as well as content.

An authenticated local manifest must bind the full 32-byte NAS namespace and peer
identity to an existing 16-byte volume identity and the user's object identity.
No truncation of the NAS identifier, server-supplied domain, or physical slot
number becomes a decryption identity. NAS slots/revisions are checked against
this manifest; object/revision/chunk/length/kind/schema are fixed before unseal.
Namespace-to-volume assignment, collision handling and manifest publication are
**OWNER/SECURITY DECISION REQUIRED** before implementation; no allocator or new
identity construction is introduced here.

## Bounded chunking decision

The existing maximum 128-byte plaintext seals to 168 bytes: 24-byte nonce plus
ciphertext plus 16-byte tag. It cannot fit in one 104-byte NASP data field.
The NAS core's 1024-byte ciphertext bound does not enlarge that wire field.

A minimal proposed probe can use at most 64 plaintext bytes per independently
context-bound record, producing at most 104 sealed bytes. This reuses the existing
codec below its limit, without truncation or custom cryptography. Chunk position,
exact final length and total object membership must be authenticated by the local
manifest. The four-object NAS fixture is still finite: it does not become arbitrary
file storage by relabelling slots as chunks. Larger objects need an explicitly
reviewed bounded fragmentation/manifest protocol. Neither wire alternative is
selected as the product format by this proposal.

## Lifecycle and uncertainty

Seal before handing bytes to the NAS transport. Publish the new authenticated
manifest only after the intended ciphertext commit/durability is established.
Unseal only after current authorization and manifest checks; a tag/context failure
yields no partially accepted plaintext. Revocation cancels future consumer use and
stale uploads; it cannot recall copies already disclosed to an authorized endpoint.

Device loss does not transfer a key to the administrator. Recovery uses only an
explicitly authorized existing recovery route. A wrong/missing secret, revoked
allocation, failed authentication or unavailable freshness state refuses access.
An unknown publication is not silently repeated as success: preserve the prior
manifest and expose uncertainty until the original operation can be reconciled.
Service restart and trusted cold freshness are distinct. Replaying both expected
context and ciphertext defeats the codec's revision check without an independent
anchor; cold antirollback remains unresolved, with no provider-derived floor.

## Required negative acceptance

| Contract | Test owner | Required future refusal/observation |
|---|---|---|
| LP-15 / EO-01 | Consumer plus NAS integration | Administrator/provider captures contain no names, plaintext, keys or sensitive manifest; unavoidable size/timing metadata listed |
| LP-22 / EO-02 | Keyring and consumer | Changed tag/header, missing/duplicated/swapped chunks, length/kind/schema mismatch; no partial output or manifest advance |
| EO-03 | Consumer/identity ingress | Other namespace/peer/object/revision, wrong domain/key index and reused physical slot fail before plaintext |
| EO-10 | Consumer/persistence | Crash before/after ciphertext and manifest publication, key rotation and unknown completion never expose mixed plaintext or false success |
| Revocation/recovery | NAS and Key-Service adapters | Retained session/upload/key handle, device loss, wrong secret, torn restore and stale authenticated state remain denied or explicitly unavailable |

These tests are proposed, not executed here. Existing partial source-bound results
remain with their original criteria. The [privacy contract](../LOCAL-PRIVACY-CONTRACT.md)
and [encrypted-storage contract](ENCRYPTED-OBJECT-STORAGE.md) govern the full outcome.
Owner decisions precede broader authority or product-format changes; P01, physical
hardware, human audit and the 41/100 roadmap score remain unaccepted/unchanged.
