# NAS-RESUME-1 — bounded client queue, host model

`no_std`, allocator-free, `forbid(unsafe_code)`. Reuses only the
local `nieuw-os-nas-core` interface, `Session`, `Upload`, revisions and errors.
No new identity/wire/crypto ABI, transport or provider implementation.
Input is **already encrypted opaque data**; this model encrypts nothing.
The only dependency is the existing local core v0.1.0 without a build script.

## Invariants and task boundary

Four pending edits, each with at most the existing 1.024 ciphertext bytes;
these are probe limits, not fixed product quotas. Full/duplicate/overflow explicitly
rejects the new edit. The existing edit is never replaced. Conflicts
retain local bytes, the server version and, after explicit observation, also a
verified copy of that remote ciphertext. `take` hands data over to the
caller and is not a silent deletion. `Debug` prints no ciphertext/identifiers.

The queue grants no authority. `prepare` **itself opens the selected namespace**,
after which existing core authentication, epoch binding and CAS are enforced.
`commit` uses the real staged ticket. A PID, route, boolean or
checkpoint grants no right. `Principal` may only come from trusted ingress;
this is the same assumption as the core, not added authentication.
A real remote adapter must identify the selected NAS cryptographically.
The caller stored at staging prevents a misrouted
commit/cancel from taking the ticket away; this comparison itself grants no right.

Offline queueing makes no service calls. An in-memory `Checkpoint` contains
no live Session/Upload. `restore` sets all unfinished edits to
`NeedsReconcile`, even when the checkpoint was taken before the dispatch.
`Unknown`, stale/revoke and conflicts produce no automatic replay.
`Checkpoint::encode`/`decode` provide a canonical versioned schema, described below.
Caller-owned checkpoints are **not a durable-save ACK, authenticated storage,
anti-rollback anchor or actual crash-resistant local storage**.
Old server RAM tickets must be cleaned up by the existing
server lifecycle on connection loss; the host recovery tests open a new core.

`reconcile` is explicit and read-only. A supplied revision hint is validated by
`Service::read`; caller/session/namespace/object remain bound.
An unchanged base can then be retried; exactly observed
target bytes yield `Observed`, **not** the original commit ACK.
A different version retains both sides, without overwriting or merging.
A deliberately chosen new revision must be re-encrypted under that context by the shared object/Key Service;
old AEAD bytes are not
silently reused under a different revision. `cancel` cleans up a not-yet-published
staging via the existing core, but retains the local edit.
Lower revisions remain blocked. Cleanup errors are returned separately.
`reconcile_latest` discovers the committed revision through `Service::head` and
reads it within the same authenticated session. It shares the existing reconciliation
state machine: uncertain writes are observed, never automatically replayed. Both
operations are synchronous within one exclusively borrowed local core; a future
remote adapter must retain revision-checked reads to detect concurrent changes.
No NASP wire opcode or transport is added.

## Checkpoint format v1

`CHECKPOINT_BYTES` is 8,464 bytes: a 16-byte header followed by four fixed
2,112-byte slot records. Header magic is `NOSNCP01`, followed by little-endian
u16 version 1 and six zero reserved bytes. An absent slot is entirely zero;
holes retain their slot positions. All integers below are little-endian.

| Slot offset | Field |
|---|---|
| 0, 1 | State 1 (NeedsReconcile), remote-present flag (0 or 1) |
| 2–3, 4–7 | u16 object index, zero reserved |
| 8–39, 40–47 | Namespace, u64 base revision |
| 48–55 | u64 observed remote revision (zero when absent) |
| 56–57, 58–59, 60–63 | u16 local/remote lengths, zero reserved |
| 64–1087, 1088–2111 | Local/remote ciphertext with zero unused tails |

Local revision is derived as base + 1 with checked overflow. Local ciphertext
must be nonempty; a present remote revision is zero exactly when its length is
zero (an observed unpublished object). Duplicate namespace/object pairs, invalid
indices, lengths, states, flags, padding, versions and nonexact sizes reject with
`Corrupt`. Encoding clears padding; decoding returns no partial checkpoint.

Every encoded occupied slot restores as `NeedsReconcile`, including previously
published/observed edits. No session, upload ticket or commit acknowledgement is
serialized. Both local and observed remote bytes survive for conflict handling.
The adapter must protect these bytes and provide authenticated, atomic and
rollback-aware storage. The codec provides no checksum, MAC, freshness proof or
disk I/O. Structurally valid modified or stale bytes remain untrusted; decoding
does not grant permission to send them or replace newer data.

## Evidence and open dependencies

Host tests use real core CAS/authority/storage recovery and synthetic
opaque bytes. Cross-compilation is library evidence, not runtime in Nieuw OS.
The optimized maximum queue (four objects of 1.024 bytes each)
runs through staging, RAM checkpoint, core recovery, reconciliation and publication on
a host thread with a 64 KiB stack. Host sizes: `Client` 9.088 bytes,
`Checkpoint` 8.608 bytes; the core arena is separate on caller-owned heap.
This does not measure native stack high-water or the full native call chain.
P02/P03/S09 and FR-P03/P04/P05 remain open. No E2EE, real network outage,
LAN/internet transition, native sync, device pairing or product claim.
The frozen probe ce52 and the separate disk oracle are not modified.

For the full chain the following are still missing here: authenticated shared transport
and peer binding; Security's object/key lifecycle interface; durable local
queue/version anchoring; remote latest-revision discovery; actual
Materiaal operation and native client/NAS execution. Provider choice remains
explicit in the shared contracts; this model does not select/change a provider,
receives no provider credentials/content keys and has no fallback.

Source-pinned alignment with the owners:

- NET `4a0ce99ff6b88adcd0e2c6be0571b30390d8d1c8` provides only
  `KeyServiceRandom` and certificate-time building blocks, **no** native handshake
  or TLS-peer-to-NAS-principal interface.
- Security `7f67471` internally provides `RecordContext` and
  `Service::seal_record/unseal_record`, not an authorized NAS app API.
  Mapping from NAS namespace to volume/lineage/epoch authority and recovery/
  independent rollback anchoring has not been established. The queue does not derive these
  rights/context from request ID, invitation or B02 fixture.

Targeted commands (Rust 1.90; dedicated `CARGO_TARGET_DIR`):

```sh
cargo test --offline --locked --manifest-path services/nas/native-client/Cargo.toml
cargo test --offline --locked --release --manifest-path services/nas/native-client/Cargo.toml
cargo clippy --offline --locked --manifest-path services/nas/native-client/Cargo.toml --all-targets -- -D warnings
cargo clippy --offline --locked --release --manifest-path services/nas/native-client/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path services/nas/native-client/Cargo.toml --all -- --check
cargo build --offline --locked --release --manifest-path services/nas/native-client/Cargo.toml --target x86_64-unknown-none
```
