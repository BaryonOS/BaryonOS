# B02 — authenticated records and exact key state

Date: 29 October 2026; extended 30 October 2026 (FR-B02P). Task FR-B02,
building on B01/ADR-043 and `services/volume`. Status: **core, wire and
native persistence slice implemented and proven under QEMU in the FR-I01
profile (see FR-B02P below); cold rollback anchoring, write-ahead in the
service and full B02 acceptance remain open**.
A host-only write-ahead contract and a durable KEY1 domain dispatch exist
(B02-T2, below); no native key-state store is wired to the service.
There is no new kernel cryptography or dependency. The existing
KEY1 opcodes/ABI are preserved; FR-B02N adds compatible record opcodes.

## Goal and security boundary

The existing B01 SEAL binds only domain and key index and is not a
storage record format. The existing recovery blob has no exact
allocation counter: legacy RESTORE marks all sixteen indices as reserved.
The new operations are therefore placed in the same `keyring_core::Service`,
with the same maintained XChaCha20-Poly1305/HKDF-SHA-256 implementations.
The existing B01 operations and their tests remain usable.

The new core contains no disk I/O, no broker that grants rights and no
trusted cold-boot clock. The service adapter must take domain, object access and
expected revision from authorized system state. An app or
storage server must not obtain that authority by filling in context
fields. Ordinary apps get no master key or persistent snapshot authority.

Invariants:

- Records authenticate volume, requester domain, key index, object,
  revision, chunk position, exact length, record kind, encoding schema, suite and
  format version. Equal-length name/metadata/content records are not interchangeable.
- An error yields no partial plaintext. Output buffers and the
  key namespace do not change on rejection. Entropy monitoring may deliberately put the
  service into `Degraded` on source failure; diagnostic counters
  keep counting as in B01.
- Exact allocations and revocations are encrypted together with the master.
  Import validates the entire record before the namespace commit. Under the same
  master, allocations are merged by maximum and revocations by bitwise union;
  a previously issued index is not issued again.
- A used or volume-bound instance does not switch to a
  different master. Even an empty valid persistent snapshot binds the lineage;
  legacy RESTORE offers no bypass.
- Generation overflow returns `Quota` before any partial namespace change.
  Temporary key/plaintext buffers use `Zeroizing`; this is
  memory hygiene and not a claim about all compiler/register copies.

## Core interfaces and formats

`records::RecordContext` contains `volume: [u8; 16]`, `domain: u8`,
`object: u64`, `revision: u64`, `chunk: u32`, `length: u16`,
`kind: RecordKind` and `schema: u16` (zero invalid).
`Service::seal_record` and `unseal_record` additionally receive the
kernel-bound domain, the key index and the existing entropy source.
The domains are exclusively `0..DOMAINS`; context and requester must have
the same domain. Object ID zero and empty content are valid values.

A record contains `nonce[24] | ciphertext[length] | tag[16]`, at most
168 bytes for the existing 128-byte chunk. The total document length is
not thereby limited to 128 bytes: the native adapter must stream.
The expected context is not stored in the record as readable document metadata.
Names, tags, collections, sensitive indexes and manifests must themselves be stored in
authenticated encrypted records.

The exactly 64-byte AAD is canonical little-endian:

| Offset | Content |
|---|---|
| 0 | `NOSREC01` (8 bytes) |
| 8, 10 | Format version 1, suite 1 (both u16) |
| 12, 13 | Domain, key index (both u8) |
| 14 | Record kind u8: 1 content, 2 name, 3 object metadata, 4 volume manifest, 5 search index |
| 15 | Reserved zero byte |
| 16 | Volume ID (16 bytes) |
| 32, 40 | Object ID and revision (both u64) |
| 48 | Chunk position (u32) |
| 52 | Plaintext length (u16) |
| 54 | Canonical encoding version within the record kind (u16, non-zero) |
| 56 | Eight reserved zero bytes |

Suite 1 uses the existing derived B01 domain/index key and
XChaCha20-Poly1305 with a fresh DRBG nonce. The separate AAD purpose
prevents interchange with legacy SEAL. Length must match exactly; truncation,
extra bytes and a changed expected length are rejected.

`Service::export_persistent(volume, secret, source, out)` receives an
externally held random 32-byte secret; it returns `SnapshotInfo`.
It keeps no copy of this secret. This is not a password KDF.
`restore_persistent(volume, secret, blob, minimum_generation, source)`
returns the same information after authentication, schema and lineage checks.
`snapshot_info()` and `key_generation()` contain no key bytes; their
management/metadata output likewise belongs behind appropriate capabilities.

The persistent blob is exactly 120 bytes:

| Component | Format |
|---|---|
| Header, 32 bytes | `NOSKEY01`, version u16=1, suite u16=1, volume ID[16], zero[4] |
| Nonce, 24 bytes | Fresh DRBG output |
| Ciphertext, 48 bytes | master[32], generation u64, per domain created u16 + revoked u16 |
| Tag, 16 bytes | AEAD tag over ciphertext and the entire header |

The wrap key comes from HKDF-SHA-256 with salt
`nieuw-os persistent key-state v1`, the user secret as IKM and info
`wrap`. No home-made crypto primitive. Only format, suite and
opaque volume ID are outside the encryption; observed sizes and
access patterns are not hidden metadata.

GENERATE and REVOKE increment the generation on success. Legacy RESTORE
increments it as well. A persistent import keeps a sufficiently new
generation or increments the maximum if a merged namespace is not yet
described by that generation. Repeatedly importing the same
current state does not change the generation.

## Native integration contract and open acceptance

1. Start the existing Key Service with the real entropy broker. A separate
   user/recovery agent supplies the external secret under an explicit
   management authority. No built-in fixture secret in the product path.
2. The existing native Key Service adapter dispatches the new operations
   8/9 via `wire::dispatch_record`. The earlier KEY1 codes, lengths and
   error meanings remain intact. Host tests execute the same dispatch function;
   native execution via the integrated app remains to be proven.
3. Take the domain from the receiving capability endpoint and check, on every
   record operation, the object, the action and the current grant. Take context and
   revision from the authenticated volume, not from untrusted ciphertext.
   Bind the record kind and the encoding schema to the permitted operation:
   reading names grants no right to a content, manifest or index record.
4. Store key allocations/revocations with the crash-consistent volume transaction
   before a confirmed handle or confirmed revocation. Do not release any ciphertext
   or document commit under an allocation that has not yet been durably recorded.
   On uncertain I/O, no false confirmation or automatic retry; preserve
   the existing `Unknown` and recovery boundary. This adapter is still to be built.
5. Prove with real VM writes, shutdown and a new VM that content,
   names, exact allocations and revocations are preserved. Read the real
   images independently and test bit flips, swaps, full storage, wrong
   key/domain, interruption and key loss. This native evidence is open.

`minimum_generation` can reject an old valid blob, **only when that
lower bound is independently trusted**. A second file next to the blob,
a UI success message or a restored VM snapshot does not provide that guarantee.
A cold instance with lower bound zero cannot distinguish an old valid snapshot
from the current one. Likewise, record decryption accepts replay if
the old expected manifest context is also restored. Full cold
rollback anchoring, persistent nonce/crash analysis and multi-writer key lifecycle
remain visibly open for B02; no criterion is signed off for these.

The existing two domains/sixteen indices are B01 trial limits. This work
does not declare those limits suitable for the full product release;
scalable key/object integration remains a release requirement.

## Checks performed

On branch `codex/full-release-storage`, base `73ebc1e`, with the existing
Rust 1.90.0 tool environment. The checks use exclusively synthetic
test data, no real disk, NAS, user secrets or production keys.

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-fr-storage-target python3 -B tools/dev.py cargo +1.90.0 test --locked -p keyring-core
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-fr-storage-target python3 -B tools/dev.py cargo +1.90.0 clippy --locked -p keyring-core --all-targets -- -D warnings
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B tools/dev.py cargo +1.90.0 fmt -p keyring-core -- --check
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-fr-storage-target RUSTFLAGS='--cfg chacha20_backend="soft" --cfg poly1305_backend="soft" --cfg sha2_backend="soft"' python3 -B tools/dev.py cargo +1.90.0 build --locked -p keyring-core --release --target x86_64-unknown-none
```

The host tests execute the real maintained AEAD/HKDF code, including
all supported record lengths, all bit flips of complete records and
snapshots, truncation, context swaps, generation limit, exact allocation,
revocation union and an explicitly demonstrated unprotected cold replay case.
The existing B01 tests continue to check the standard test vectors and earlier
recovery/entropy cases. Host/no_std evidence does not replace
native service/disk execution or an independent security audit.

Outcome on the final source: **15 existing + 23 new host tests
passed**, Clippy with `-D warnings`, formatting check, diff check and
the portable `x86_64-unknown-none` release build, all exit 0. The new
record kind/schema binding came from a separate agent review: before integration,
the concrete interchangeability of equal-length name/content records was resolved.
No VM run, actually stored encrypted volume or full cold
anti-rollback is claimed by this partial batch.

## FR-B02N — connected KEY1 record operations

`SEAL_RECORD = 8` and `UNSEAL_RECORD = 9` use the same KEY1 tag,
256-byte payload, 16-byte header and normal error responses. The body is a
canonical context of exactly 64 bytes followed by plaintext or
`nonce | ciphertext | tag` respectively. SEAL body: 64..192 bytes; UNSEAL body:
104..232 bytes. Both fit within the existing 240-byte body.

The context prefix has this form, distinct from the internal record AAD:

| Offset | Content |
|---|---|
| 0 | `NOSCTX01` (8 bytes) |
| 8, 10 | Record format version 1 and suite 1 (u16) |
| 12 | Record kind u8, exclusively the five values listed |
| 13 | Three reserved zero bytes |
| 16 | Volume ID[16] |
| 32, 40 | Object ID and revision (u64) |
| 48 | Chunk position (u32) |
| 52, 54 | Plaintext length and non-zero encoding schema (u16) |
| 56 | Eight reserved zero bytes |

There is **no domain in the wire context**. The native adapter determines it
exclusively from the receiving endpoint. `decode_record_context` fills in
`RecordContext.domain` with it. Thus ciphertext under domain A, when
received on domain B, is evaluated with a different AAD/key and rejected.
The wire context is not an object grant; the object/action check described earlier
remains the responsibility of the native access adapter.

Usage for a native caller:

```rust
let payload = keyring_core::wire::record_request(
    keyring_core::wire::SEAL_RECORD, key_index, request_id, context, plaintext,
)?;
// Send payload under KEY1 via the already granted endpoint.
// Validate sender, tag, zero received capabilities, headerop|REPLY,
// key_index, request_id, status, exact length and zero padding of the response.
```

UNSEAL uses the same builder with opcode 9 and the received sealed record.
A SEAL response contains at most 168 body bytes; an UNSEAL response at most
128 bytes. Every error has length zero and a zero-filled body. The dispatcher
checks exact context/data length, version, suite, kinds, schema,
reservations, request status and ingress domain before cryptographic processing.
No persistent snapshot opcodes or new recovery authorities have been added for
ordinary apps.

Executed on the final FR-B02N source: **46 host tests (15 legacy,
23 secure-record and 8 wire/dispatch)**, Clippy with `-D warnings`, formatting
and the real native service/app build for `x86_64-unknown-none`, all exit 0.
The existing `sdk/examples/keyring/build.py` explicitly uses portable
crypto backends and checks the native ELF budgets via the linker script.
Command for that build:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-fr-storage-target/native-service python3 -B sdk/examples/keyring/build.py
```

This build evidence does not run a VM. The main integrator must invoke the new opcodes
natively and observe cross-domain/type/schema/tampering errors;
the persistent storage/rollback gates remain open.

## FR-B02P — persistent key state and sealed record natively

30 October 2026, ADR-051, branch `codex/b02-persistent-keystate-20261030`.
This completes contract steps 1–3 and a bounded form of 4–5 above;
it is not full B02 acceptance.

### Wire

`EXPORT_PERSISTENT = 11` and `RESTORE_PERSISTENT = 12` use the same
KEY1 tag, header and error responses. Key index must be 0.

| Opcode | Request body (exact) | Ok response body |
|---|---|---|
| 11 | volume ID[16] \| secret[32] = 48 bytes | blob[120] \| snapshotinfo[16] = 136 bytes |
| 12 | volume ID[16] \| secret[32] \| minimum generation u64 \| blob[120] = 176 bytes | snapshotinfo[16] |

Snapshotinfo is little-endian: generation u64, then per domain created u16 and
revoked u16. `wire::dispatch_persistent` serves exclusively these two opcodes;
`dispatch_record`, `dispatch_random` and the native service's legacy dispatch
reject 11/12 with `Invalid` before any cryptographic step. Host tests:
`services/keyring/tests/persistent_wire.rs` (5 tests: round trip to a cold
instance preserving revocation and record keys, wrong secret,
generation floor, foreign volume, all other opcodes/lengths/index/status,
readiness).

### Native adapter

The Key Service optionally reads a third ingress from start words 96 (receive),
104 (reply-send) and 168 (permitted sender PID). If word 96 is absent, the
B01 profile remains exactly unchanged. Every request is checked for sender,
capability count, tag and header; evidence words 384–416 count
seen/exports/restores/rejected/wrong sender. The secret from the
request is processed in `Zeroizing` buffers. The SDK wipes the temporary
native receive buffer on valid envelope transfer and wipes `Outgoing` and
`Received` envelopes on cleanup; their `Debug` representation contains no payload.
This leaves caller-owned arrays, possible compiler/register copies and
crash memory outside the guarantee. The underlying received IPC copy is
memory hygiene, not a key storage or full key isolation guarantee.
At startup the service rejects an admin sender equal to the ordinary
key client or to itself, and admin and RANDOM ingress must not share a
handle. Init wipes its own request copies after every exchange. A SEND
to this ingress is equivalent to the full key namespace (EXPORT
under a self-chosen secret), and for a cold instance the
external secret authenticates the lineage; see ADR-051.

### Integrated profile (FR-I01, `native-system`/`test-system-recover`)

- Manifest: admin channel Key Service ← init, reply channel init ← Key
  Service, volume domain 1 for init; none of the six app handles refers to
  the admin ingress or its reply channel (`APP_ADMIN_HANDLE`, via the new
  read-only `Engine::endpoint_object`), and init's new handles carry no
  entropy/block/launch/network right (`INIT_AUTHORITY`). Since INT3-FU-116
  that comparison covers the full app handle table: `APP_HANDLE_COUNT` requires
  that the six handles are mutually distinct and that
  `Engine::available_slots(app)` is exactly `MAX_SLOTS` − 1 (reserved
  zero slot) − 6, so that a seventh handle (for example to the
  admin ingress) makes the manifest fail. This holds at manifest time, in
  every profile that executes the manifest from `kernel/src/m2/m3/system.rs`;
  the NAS profiles load `system` from `m3/nas.rs` and do not have this
  check. Later transfer via IPC is
  excluded because no admin handle carries `TRANSFER` (code reading, no
  separate native trial).
- Normal run: the app generates key 0, seals/unseals, performs the ten
  context rejections, revokes key 0, generates key 1 and seals
  a 128-byte record in RAM (context: volume `NOS-SYSTEM-VOL01`, object 1,
  revision 1, chunk 0, schema 1). After `KEYS_READY` init exports the
  key state (created [2,0], revoked [1,0], generation 3), rejects wrong
  secret, generation floor and GENERATE-on-admin, commits the blob as object 8
  (`sleutelstatus`, 120 bytes) in domain 1, re-imports idempotently and
  sends `PERSISTED`. Only then does the app store the record as document 1
  (`verzegeld`, 168 bytes) via the ordinary DocumentSession.
- Cold restart (same image, read-only block right): init reads object 8,
  rejects wrong secret and floor `u64::MAX`, restores with floor 0 the exact
  state before `GO`. On GENERATE the app gets index 2 (no reissue),
  opens document 1 and unseals it with key 1; key 0 gives `Revoked`,
  a modified last byte gives `Auth`.
- Independent disk oracle (`tools/volume_disk.py`, `run_system.check_disk`):
  objects 1 and 8 are explicitly opaque; the record does not contain the known plaintext,
  the blob carries only `NOSKEY01`/version/suite/volume ID outside the
  AEAD body; generation 3, three chunks, domain clocks [2, 1].

Exact execution, hashes and limits: PROJECT-STATE.md,
`docs/verification/full-release-b02-keystate-*` and, for the review fix
of the admin ingress check (#116), `docs/verification/full-release-b02-review-followup/`.
Not proven: trusted
generation floor, write-ahead per GENERATE/REVOKE within the service, loss or
compromise of the external secret (= attacker-chosen master at cold
start), multiple writers, a native counterexample for the
admin sender check, physical hardware.

B02-T2 (host only): `Service::generate_durable`/`revoke_durable` commit
the sealed key-state snapshot through a caller-supplied `KeyStateStore`
before confirming, and `wire::dispatch_domain_durable` routes KEY1
GENERATE/REVOKE of a domain ingress through them. `Ok` is replied only
after a successful commit in that call; a certain store failure replies
`NotReady` (explicit retry allowed) and an uncertain one replies
`Degraded` and latches the instance. The volume, secret and store come
from the service adapter, never from the request. Host tests:
`services/keyring/tests/write_ahead.rs`. Still open: a native store
for the keyring service (ADR-051 gives it no volume right and keeps the
secret outside the service), so no QEMU profile exercises this path.

## FR-B02R — KEY1 key-state commits relayed through init (B02-T4)

24 November 2026, ADR-B02-KEY1-RELAY-20261123, branch
`claude/b02-key1-relay-native-20261124`. TEST profile only
(`test-system-keystate-relay*`); the `native-system` and `test-system-recover`
profiles are unchanged. This closes the "no native store" gap of B02-T2 for a
single instance; it is not full B02 acceptance.

### Wire (`services/keyring/src/wire.rs`)

| Opcode | Direction | Request body | Reply |
|---|---|---|---|
| `SUPPLY_SECRET = 13` | init → Key Service admin ingress | secret[32], index 0, status 0 | `Ok`, empty |
| `COMMIT_KEYSTATE = 14` | Key Service → init commit ingress | blob[120]; header `request` = generation ≠ 0 | op `0x8E`, index 0, `request` = generation, length 0; status `Ok` (`COMMITTED`) or `NotReady` (`NOT_COMMITTED`) |

Neither opcode is in `body_bounds`. The host test
`relay_opcodes_are_refused_by_every_service_dispatch_without_state_change`
shows that `dispatch_domain`, `dispatch_persistent` and `dispatch_management`
answer `Invalid` without state change. RANDOM ingress never reaches
`dispatch_random` with them, because `poll_random` refuses any opcode other
than RANDOM first (`sdk/examples/keyring/src/service.rs`). The native probe
still sends only opcodes 11 and 12 on the domain endpoint; a native refusal of
13 and 14 there is not measured.
`commit_outcome` maps only a canonical `Ok` reply to success and only a
canonical `NotReady` reply to `StoreError::Failed`; every other reply,
including a different generation, a dirty tail, a nonzero index or any other
status, is `StoreError::Unknown`. `COMMIT_TICKS = 1500` monotonic ticks bounds
one commit (send plus reply).

### Reply matching by generation

The commit reply channel carries one reply per request and has no separate
request identifier, so the service matches the reply to its own call by the
generation in the header `request` field. This is sufficient because:

- A generation is sent at most once while no certain answer exists. A
  `GENERATE` after `NOT_COMMITTED` allocates the next generation; a `REVOKE`
  retried after `NOT_COMMITTED` re-commits the same generation, but the earlier
  attempt was answered with a certain `NotReady`, so no reply for it is still in
  flight (host test `failed_commit_replies_not_ready_and_a_retry_commits`,
  generations `[1, 2, 3, 3]`).
- The only outcome that can leave a late reply in the channel is `Unknown`
  (timeout, transport error, malformed reply). `Unknown` latches the instance in
  `Degraded` before any further store call
  (`unknown_commit_latches_degraded_without_further_store_calls`), so a late
  reply is never read as the answer to a later commit.
- A reply from a sender other than init's PID (startup word 512, which must
  equal the admin caller), with capabilities, with a different tag or with a
  non-256-byte payload is `Unknown` before decoding.

### Volume binding

The Key Service binds write-ahead state to `RELAY_VOLUME =
b"NOS-SYSTEM-VOL01"`, a compile-time constant in
`sdk/examples/keyring/src/service.rs`, never to a request field. Edge cases:

- In relay mode the admin ingress serves only one `SUPPLY_SECRET`; `EXPORT`,
  `RESTORE`, a second supply and every other opcode are answered `Invalid`
  without reaching `dispatch_management`. No admin request can therefore bind
  `persistent_volume` to another volume before the first durable commit.
- The first durable `GENERATE`/`REVOKE` binds `persistent_volume` to
  `RELAY_VOLUME` before sealing; `check_write_ahead` (unchanged
  `persistent.rs`) refuses any later call with a different volume with
  `Invalid` and no store call (`first_bind_fixes_the_relay_volume`).
- Init independently checks that the blob header carries `NOSKEY01`, version 1,
  suite 1 and the same volume ID before it begins a transaction, and refuses
  the commit with `NOT_COMMITTED` otherwise.

### Native adapter and manifest

- Key Service startup words 496 (commit SEND), 504 (commit reply RECEIVE) and
  512 (init PID). All zero keeps every existing path. A partial set, RANDOM
  ingress present, equal handles, a relay handle equal to any other ingress
  handle (words 48/56/80/88/96/104), an init PID different from the admin
  caller, a missing admin ingress or an unavailable monotonic clock stops the
  service with startup error 12 before it serves anything.
- Until init supplies the secret the service polls only the admin ingress, so
  no domain request can be served without a store. Each domain
  `GENERATE`/`REVOKE` then goes through `dispatch_domain_durable` with the held
  secret, `RELAY_VOLUME` and the `RelayStore`. The secret is held in
  `Zeroizing` memory (`HeldSecret`, no `Clone`/`Debug`).
- Init (words 192/200/208) receives commits by the kernel endpoint object (it
  alone holds RECEIVE on the commit ingress); it additionally checks the sender
  PID against the Key Service. It writes the blob to object 8 in domain 1 with
  `expected_revision` = its last acknowledged revision, and replies `COMMITTED`
  only when the volume returns exactly revision + 1; any other volume result
  after the transaction began is answered `Degraded`, which the service maps to
  `Unknown`. Init makes no blocking call into the Key Service while commits are
  outstanding: after `GO` it waits on the registry with a tick-bounded loop that
  polls the commit ingress.
- Manifest checks (`kernel/src/m2/m3/system.rs`), in order:
  `APP_COMMIT_HANDLE` (no app handle refers to the commit ingress or its reply
  channel, by endpoint object), `VOLUME_COMMIT_HANDLE`, `VOLUME_HANDLE_COUNT`
  (exactly 5), `KEY_COMMIT_HANDLE`, `KEY_HANDLE_COUNT` (exactly 8),
  `KEY_COMMIT_RIGHTS` and `INIT_COMMIT_RIGHTS` (the Key Service holds only SEND
  on the commit ingress and only RECEIVE on the reply channel, init the
  converse; none of the four handles is delegable). Init's commit handles also
  pass `INIT_AUTHORITY`. The exact counts list every handle: the volume service
  holds block, its ingress, the second ingress, the reply SEND and the second
  reply endpoint (5); the Key Service holds entropy, domain ingress, second
  ingress, reply SEND, admin ingress, admin reply, commit SEND and commit reply
  RECEIVE (8). Workers are spawned at runtime and receive only handles that init
  delegates; init's commit handles carry no TRANSFER, so no worker can obtain
  them.
- Durability before `COMMITTED`: init replies only after the volume service has
  returned the new revision of object 8, and the volume service acknowledges a
  write only after its crash-consistent transaction (data flush, table write,
  flush) completed (`services/volume/src/lib.rs`).

### Pinned expected values and derivation

Common to the three passing cases (`tools/run_system.py` `RELAY_EXACT`,
`PERSIST_EXACT["relay"]`, kernel `relay_check_and_print`):

| Value | Expected | Derivation |
|---|---|---|
| `OK`, `INIT_COMMITTED`, `INIT_BEGUN` | 3 | The app's key flow is GENERATE key 0, REVOKE key 0, GENERATE key 1: three durable operations, each committed once. |
| `LAST_GENERATION`, `INIT_GENERATION`, `KEYSTATE_GENERATION` | 3 | Generations 1, 2, 3 for those three operations; word 472 changes only on `Ok`. |
| `INIT_REVISION` | 3 | Object 8 is new; each commit uses expected revision n and yields n + 1. |
| `SECRET_SUPPLIED` / `SECRET_REFUSED` | 1 / 1 | Init supplies once and then sends a second supply with a wrong secret, which must be refused. |
| `ADMIN_REQUESTS` / `ADMIN_DENIED` / `KEYSTATE_ADMIN_OP_DENIED` | 3 / 2 / 1 | Supply, refused second supply, refused admin GENERATE. |
| `KEYSTATE_EXPORTS`, `KEYSTATE_RESTORES`, `CRYPTO_RESTORES`, wrong-secret and floor denials | 0 | Relay mode never exports or restores. |
| `CRYPTO_TAMPER_DENIED` | 9 | The normal profile's 10 includes one AEAD rejection from the wrong-secret RESTORE, which relay mode does not perform. |
| `COMMIT_TICKS` | 1500 | Bound for one commit including the volume write. The measured answered commit takes 5 ticks, so the bound leaves a margin of about 300× for a slower host while keeping the timeout case under 20 s of TCG time. |
| `MAX_COMMIT_TICKS` | 0..1499 (answered; 0 occurs under KVM when every commit finishes within one tick), 1500..2999 (timeout) | An answered commit returns before `COMMIT_TICKS`; a withheld one returns at the first tick check at or after the deadline (measured exactly 1500). The upper bound 2 × `COMMIT_TICKS` rejects a wait that ignored the deadline while tolerating scheduling delay after it. |
| `SUM` | ≠ 0, equals FNV of the current object 8 version on disk | Init's checksum of the last acknowledged blob; random nonces make the value differ per run. |
| Disk: object 8 | revision 3, versions [3, 2, 1], 120 bytes, `sleutelstatus`, header bound to `NOS-SYSTEM-VOL01`, three distinct nonzero nonces | Three commits; volume `HISTORY = 2` keeps all three. |
| Disk: volume state | generation 5, 5 chunks, domain clocks [2, 3] | Five volume commits: document and sealed record in domain 0, three key-state versions in domain 1. |

Per case:

| Case | `REQUESTS` = `INIT_RECEIVED` | `NOT_COMMITTED` = `INIT_REFUSED` | `UNKNOWN` = `INIT_WITHHELD` | `APP_FIRST`, `APP_SECOND` | Derivation |
|---|---|---|---|---|---|
| `relay` | 3 | 0 | 0 | 0, 0 | No fault probe. |
| `relay-not-committed` | 5 | 2 | 0 | 1, 1 | After the record, the app sends two raw GENERATEs; init refuses commits 4 and 5 before any write, so both answer `NotReady` (1) and the volume is unchanged. |
| `relay-timeout` | 4 | 0 | 1 | 7, 7 | Init withholds the reply to commit 4; the service reaches `COMMIT_TICKS`, latches, answers `Degraded` (7), and answers the second GENERATE `Degraded` without a store call. |

Red case `relay-leak`: the manifest also gives the app a SEND to the commit
ingress. Exactly one `SYSTEM:FAIL:APP_COMMIT_HANDLE`, no
`SYSTEM:MANIFEST:PASS`, no relay markers, the three boot markers, raw QEMU exit
35 and an unchanged disk image are required (`run_system.evaluate_red`).

### Execution

Source `d4db823e`, QEMU 10.2.1 TCG, `docs/verification/b02-key1-relay-20261124/`:
`relay` 4.6 s, `relay-not-committed` 4.6 s, `relay-timeout` 19.6 s (raw 33
each), `relay-leak` 1.8 s (raw 35). Maximum commit latency 5 ticks for answered
commits and 1500 for the withheld one. `tools/test_system.py` and
`tools/test_keyring.py` pass. The `agent_gate.py` prefilter could not resolve
either job locally (NOT RUN); hosted CI is not run.

### Residuals

- Cold rollback anchoring and a trusted generation floor remain open (B02-T3).
  Deviation from ADR item 2: in relay mode the admin ingress refuses
  `EXPORT_PERSISTENT` and `RESTORE_PERSISTENT`, so a relay instance cannot
  cold-start with RESTORE plus secret supply. Only the fresh-instance supply
  (`SUPPLY_SECRET`) is implemented; the cold-start half depends on B02-T3.
- The disk oracle proves that the current object 8 bytes equal the blob init
  acknowledged and that the header binds `NOS-SYSTEM-VOL01`; it does not prove
  that generation 3 inside the AEAD restores.
- The wait for the secret supply before domain traffic has no deadline. It
  fails closed (no domain request is served), and the kernel's system bound ends
  the run; the ADR's bounded-wait rule is applied to commit waits.
- Only `relay-leak` (commit SEND in the app) is a red manifest control; the
  volume-held, rights and handle-count conditions have no red case.
- The secret is the compile-time test constant in init (ADR-051). The service
  now retains it, so a reader of Key Service memory can forge lineage (ADR
  consequence). `main` never returns, so `HeldSecret`'s
  drop does not run; the wipe at instance end comes from the kernel zeroing
  freed frames (`paging::zero_dynamic` via the heap release path); wiping
  of the SDK `Received` envelope copy of the supply request is not separately
  verified.
- The timeout case withholds the reply before any write, so a timeout after
  the volume transaction began is not exercised natively (it maps to the same
  `Unknown` by construction; init replies `Degraded` for a failed write).
- One writer, one instance, one volume identity. The init PID check is defence
  in depth; authority comes from the endpoint object.
- Evidence is QEMU TCG only; no hardware, no human audit.
