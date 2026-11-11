# NAS-CORE-1 — bounded Rust management core

**Implementation step for P04; not yet a native NAS service or rubric point.**
This standalone `no_std` crate has no external dependencies, allocator,
host API or cryptographic primitive. The existing Node probes remain
unchanged. The core stores only opaque client ciphertext and necessary
management metadata; it receives no document names or content keys.

## Contract before native connection

The trusted ingress supplies `Principal`, an already cryptographically
verified device/administrator identity. This public fingerprint is
**not** proof of authentication by itself; JSON or IPC input must not
promote it to sender on its own. The native adapter must obtain this from authenticated
transport and kernel-enforced endpoints. The constructor and public
Rust methods do not form a kernel security boundary.

Administrator and guest are separate principals. An invitation binds a
fresh namespace and single-use nonce to the intended recipient and expiry time.
The administrator sets the quota separately and may adjust it.
Creating and accepting are durable transactions. The adapter
additionally checks the NAS identity, selected LAN/internet rights and
trusted time; these are not simulated by this core.
`Invitation` is only the local store tuple, not an established interoperable
NAS token, signed network message or multi-device recovery contract.
The transport envelope must bind the selected NAS identity separately.
The B02 keyring fixture is not used here as a NAS contract.

The caller reserves a `Workspace` in advance outside the native stack,
for example in exclusive static or allocated process memory. Both
`initialize` and `recover` borrow that working memory exclusively. State,
transaction copy, snapshot and temporary uploads live there; the core adds
no allocator or `unsafe`. `validate_snapshot` checks only
schema, not authenticity or freshness. The optimized host probe
with a 64 KiB stack checks the existing native stack budget as
preliminary investigation; actual kernel execution remains separately required.

Sessions and uploads are bound to caller, namespace, epoch and a fresh boot ID.
They are never stored or restored after restart. Staging copies
the input, counts against the quota and publishes nothing yet. Commit re-checks
the original revision; a competing writer gets a
conflict, not a silent overwrite. Cancelling and closing release only
the caller's own reservations. Reading yields an unchanged
output buffer on every rejection.

`Storage::replace` must replace an entire snapshot atomically and durably;
an old or a complete new snapshot are the only permitted
crash outcomes. The adapter must guarantee exclusively one writer. Success
before flush or only a RAM copy is insufficient. Any error on replacement
blocks the instance; it does not repeat a mutation. Every error from `replace`
becomes `Unknown` for initialization and publication: the storage contract gives
no guarantee, even for `Io` for example, that the commit did not take place.
`last_storage_error` retains the original cause for local
diagnostics, without including content or tokens in it.
The caller must reopen and inspect the
durable state, not assume success again.

Revocation first records `Revoked` with a higher epoch and then removes
RAM handles. Eviction performs a **second** transaction that deletes the managed
ciphertext copies and stores `Evicted`. If that second step fails,
the already durable revocation remains in effect. For both
steps the administrator needs no connection, permission or decryption key from the guest.
A full guest quota does not block the administration transactions. Tombstones are
not reused: a new offer gets a new namespace.

This is logical deletion of this service copy, not physical secure
erase, deletion of independent backups or local client files.
The storage adapter must also clean up its own temporary snapshots and any
storage history according to that management contract; without that evidence
the product interface must not claim complete deletion.

## Snapshot and probe limits

`NOSNAS01` v1 has a canonical little-endian encoding with fixed length
`SNAPSHOT_BYTES` (17.216 bytes). Header, version, zero fields, namespace status,
ID uniqueness, quotas, revisions, lengths and unused payload space are
validated before recovery. A missing or damaged existing record never
silently becomes an empty store. Initialization is an explicitly different operation.

The snapshot provides **no checksum, MAC or anti-rollback anchor of its own**.
The storage adapter must provide integrity and crash consistency; a
malicious NAS administrator can replace their own authority database,
offer old ciphertext or end availability. The client must
verify AEAD binding and trusted revision history independently.
A schema validator does not detect every semantically valid bit change.

For this first build step: four namespaces including tombstones, four
object slots per namespace, 1.024 ciphertext bytes per object, sixteen sessions
and eight uploads. Copy-on-write reservation counts both the old and new bytes.
These are visible probe limits, not accepted daily capacity.
Streaming, production capacity, network service, client encryption, name/manifest
encoding, background sync, multiple devices, key rotation/recovery,
native UI and independent human review remain open.

## Targeted checks

With the existing Rust 1.90.0 toolchain and a dedicated target directory:

```sh
cargo test --offline --locked --manifest-path services/nas/native-core/Cargo.toml
cargo test --offline --locked --release --manifest-path services/nas/native-core/Cargo.toml
cargo clippy --offline --locked --manifest-path services/nas/native-core/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path services/nas/native-core/Cargo.toml --all -- --check
cargo build --offline --locked --manifest-path services/nas/native-core/Cargo.toml --release --target x86_64-unknown-none
```

The host tests inject storage outcomes before and after commit. They do not prove
power loss, QEMU execution, native transport authentication or complete
P04 acceptance. Current commands, exit codes and source binding belong in the existing
project status and verification bundle, not as an unexecuted green claim here.
