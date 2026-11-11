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
Caller-owned checkpoints are **not a persistent schema, durable-save ACK,
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
The core does not yet have a guest `head/latest` API: automatic revision discovery
is not replaced here with a new wire or an untrusted status.

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
