# Shared artefact-store contract

**Status:** contract proposal agreed by Apps and AI owners; root integration
and native execution remain open.
**Owner:** release integration. **Date:** 30 October 2026.
**Evidence boundary:** this document describes a contract, not a native store,
kernel ABI, serialised Message320 operation, dynamic launcher or durability
guarantee.

This contract separates large immutable artefacts from the document volume.
The document volume remains for documents; its current object/domain limits
are not artefact capacity. The same read-only path may later serve app images
and AI models, but their trust authorities remain separate.

## 1. Invariants and non-goals

- A read handle is opaque, immutable and bound to the authenticated client
  context. The concrete service checks that context, store epoch and current
  access on every operation. The handle token itself grants no rights;
  revocation makes subsequent reads stale or unauthorised.
- Descriptor fields and SHA-256 are metadata and content-integrity input,
  not evidence of provenance, publisher trust, model safety or executability.
  No `verified` boolean.
- Reads are read-only. This contract grants no network, document, key,
  launch or administrative capability.
- Read API chunks are at most 4096 bytes. This is not an IPC wire format or
  a product limit on artefact size. Existing Message320 payload remains at
  most 256 bytes; a future adapter must split large reads without bypassing
  the IPC limit or capability boundary. [AS-IPC1] [AS-IPC2]
- Storage capacity and quotas are provider/namespace values in bytes.
  A candidate model, app fixture or document-volume value does not become
  global OS capacity.
- Installation/publication is a separate write transaction. Prior reservation
  is not a promise of durable storage. Only the concrete backend can specify
  and demonstrate its commit/flush/recovery guarantees.

## 2. Logical artefact identity and descriptor

Shared store identity is separate from package/model provenance:

| Field | Type | Meaning |
|---|---|---|
| `artifact_id` | `[u8; 32]` | Opaque ID assigned within the store |
| `immutable_generation` | `u64` | Exact immutable byte generation |
| `length` | `u64` | Exact byte count |
| `sha256` | `[u8; 32]` | Expected complete-content digest |

`ArtifactRef` is the pair `(artifact_id, immutable_generation)`;
`ArtifactDescriptor` adds `length` and `sha256`. This is a logical in-memory
form, not `repr(C)`, canonical byte encoding or an IPC wire ABI. Descriptor
fields are untrusted metadata until `open` compares them with the
store-authoritative descriptor. This contract prohibits zero-length
artefacts: `length == 0` is `Invalid`. The EOF rule applies only to a valid,
non-empty descriptor and `offset == length`.

The current Apps leaf uses a different 116-byte package descriptor with
`artifact_id`, `source_id`, `source_generation`, `version`, `length` and
`sha256`; the AI leaf uses a local numeric ID and `u64` generation. These are
consumer/leaf models, not two shared store ABIs. Store mapping keeps package
source/revocation generation and package version separate from immutable store
generation. Apps and AI have agreed this separation; their leaf adapters have
not yet been integrated into this root baseline. No caller or model metadata
may fabricate store authority.

`open` compares the requested descriptor with the store-authoritative
descriptor and opens only that immutable generation. A digest field does not
prove that all bytes have already been read or hashed. A consumer that will
parse or use bytes first verifies the entire stream against `length` and
`sha256`; partial plaintext/data are not released as a valid artefact after
an integrity-check failure. A later native adapter must specify exactly where
that verification occurs.

## 3. Read-only API and bounds

The logical API is:

```text
open(descriptor) -> opaque ReadHandle
read_at(handle, offset: u64, output: &mut [u8]) -> byte_count
capacity() -> Capacity
```

Internally, `ReadHandle` contains at least descriptor binding and store epoch.
Callers cannot construct or mutate its fields. The concrete service binds the
handle to the kernel-authenticated sender; a caller ID in payload, numeric
locator or public epoch is not authority. Every `read_at` rechecks sender,
rights, revocation and current epoch.

For `read_at`:

1. `output.len()` must be between 1 and 4096; otherwise `Invalid`.
2. `offset == length` explicitly returns `EndOfArtifact`, without backend I/O.
   `offset > length` returns `OutOfRange`.
3. Use checked arithmetic for `remaining = length - offset` and
   `requested = min(output.len(), remaining)`. Overflow or inconsistent
   descriptor/store state fails closed.
4. The backend may return between 1 and `requested` bytes. A positive short
   read before `length` is valid; the caller continues at `offset + byte_count`.
   Zero bytes before declared EOF is `Truncated`; more bytes than `requested`
   is `Protocol`.
5. The caller calculates the new offset with checked addition. The API never
   returns empty success data as an EOF fallback.
6. A stale epoch, revoked handle or changed generation returns `Stale` or
   `Unauthorized`; it does not reopen under an implicitly new generation.
7. Every error leaves the result explicitly invalid. A backend error is not
   converted into success, EOF, zero bytes or another generation.

The logical error set is `Invalid`, `Unauthorized`, `Stale`, `Corrupt`,
`Truncated`, `Overflow`, `EndOfArtifact`, `OutOfRange`, `NotFound`,
`Protocol` and provider I/O error. The service/wire adapter may map these only
to an existing explicit status; it must not silently discard cause or uncertainty.

## 4. Capacity and quotas

A store reports `capacity_bytes`, `used_bytes`, `reserved_bytes` and
`quota_bytes`, each as `u64`, for the same account/namespace and snapshot time.
Values from different scopes must not be combined; if the backend cannot
provide a coherent snapshot, it returns an explicit error. These are snapshots,
not reservations or commit guarantees. Available space is:

```text
limit = min(capacity_bytes, quota_bytes)
committed = checked_add(used_bytes, reserved_bytes)
available = checked_sub(limit, committed)
```

Addition overflow returns `Overflow`; `committed > limit` returns `Corrupt`.
No saturating arithmetic, negative/clamped value or success-shaped fallback.
The operational `begin` remains authoritative and may still return `Full`
or `Quota` after an earlier capacity snapshot.

## 5. Separate write transaction

Write operations are exclusive to an explicitly authorised install/publish
principal; read-only consumers do not receive this interface. The service
obtains caller identity from kernel-authenticated message metadata. A
transaction binds at least `(caller, request_id, service_epoch)` to one descriptor.

```text
begin(key, descriptor) -> reserve full length
write_chunk(key, exact_next_offset, nonempty_bytes <= 4096) -> next_offset
commit(key, descriptor) -> published immutable handle
cancel(key) -> cancelled
outcome(key) -> Pending | Committed | Cancelled | Unknown
```

- `begin` atomically reserves the full declared length before accepting any
  chunk. A definitive error (`Full`, `Quota`, invalid, unauthorised, busy)
  means no mutation. `Unknown` means the mutation outcome is uncertain and
  the transaction becomes poisoned.
- Chunks are non-empty, at most 4096 bytes, follow exactly the expected offset
  and never exceed `length`. Replay, gap, overlap, overflow or an
  acknowledgement with a different next offset fails; a mismatch makes the
  outcome `Unknown` when a write may already have occurred.
- `commit` is allowed only after exactly `length` bytes. The backend rechecks
  length and complete SHA-256 before atomic publication. A digest/length
  error publishes nothing. `Ok` is not a general power-loss, crash-recovery
  or durable-media guarantee.
- `Unknown` blocks new chunks, commit, cancel and automatic replay.
  `outcome` may be queried only within the same service epoch. After service
  restart, the old epoch is stale; transaction recovery requires a separately
  designed and tested persistent journal. There is no success-shaped fallback
  or assumption here that restart equals cancellation.
- `cancel` applies before the commit boundary. An uncertain outcome remains
  `Unknown`; cancel must not turn it into `Cancelled`.

Full reservation thus remains separate from durability. A native store must
later demonstrate storage I/O errors, flush order, crash/cold recovery, orphan
cleanup, rollback and the precise meaning of `Committed` as separate acceptance.

## 6. Trust authorities and consumer boundaries

- **Apps:** `source_id`, `source_generation` and package version are
  package/source identity, not immutable store generation. SHA-256 binds
  bytes; it does not make the current in-memory SourceRegistry a persistent
  publisher trust authority. App installation grants no dynamic launch
  authority. A kernel- or broker-verified launch capability is a separate
  future contract.
- **AI:** model pin, format validation, complete stream digest, publisher
  trust and suitability for inference are separate checks. The AI owner
  adapts the numeric trial ID to the store-assigned `artifact_id` form;
  that leaf change is not yet integrated into this root baseline. This reader
  grants no network or personal-document rights. AI inference remains
  without a network capability.
- **Shared store:** manages immutable bytes, version and read/transaction
  status. The document volume, publisher keyring, B02/B03 root or a
  caller-populated boolean is not silently reused as store authority.

## 7. Implementation and acceptance boundary

The first shared code step is a dependency-free `no_std` contract crate with
pure descriptor, range, read-handle and capacity validators plus negative
host tests. It implements no writable transaction engine: the write API above
remains an interface contract for a later step. This slice must add no kernel
ABI, syscall, Message320 layout, backend, package launch, model download or
claim of successful installation/inference.

Current read-only tests check: zero/incorrect ID and generation; descriptor
mismatch on open; stale/revoked and unauthorised reads; read buffer 0/4097;
exact EOF and offset beyond EOF; positive short read, zero before EOF and
impossible backend byte count; and capacity overflow plus
`used + reserved > limit`. A later writable implementation requires separate
negative tests for quota/full before writes; empty/large chunk,
out-of-order/replay/overrun; mismatch acknowledgement; incomplete/digest-invalid
commit; Unknown on begin/chunk/commit/cancel; outcome with incorrect
transaction or epoch; and restart invalidation without automatic retry.

Host tests check only contract logic. Target build, native store, dynamic
launch, publisher trust, model bytes, inference, flush durability, recovery,
performance and full FR/rubric acceptance remain open.

## 8. Primary sources for the IPC boundary

- **[AS-IPC1]** Baryon OS project. (2026, October 26). *Executable kernel and security contract M2–M4*, Message format and transaction section (source version `7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb`). [Kernel and security contract](https://github.com/simonvanieperen/nieuw-os/blob/7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb/docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md).
- **[AS-IPC2]** Baryon OS project. (n.d.). *SDK Message320 envelope: MESSAGE_SIZE, MAX_PAYLOAD and length validation* (source version `7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb`). [SDK implementation](https://github.com/simonvanieperen/nieuw-os/blob/7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb/sdk/runtime/src/message.rs).
