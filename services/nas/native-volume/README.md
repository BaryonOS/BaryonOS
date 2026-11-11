# NAS-VOLUME-1 — adapter on the existing volume service

`VolumeStorage` implements the NAS `Storage` interface via the existing
`volume_core::client::Client`: no second filesystem, crypto layer, raw disk path
or modified kernel/SDK/network interface. Dependencies are exclusively
the exact local `0.1.0` crates NAS-core and volume-core (including catalog-core).
These are repository code, `no_std`, without external packages or build scripts.

The trusted startup process supplies an **exclusive** NAS volume object and a
kernel-bound volume endpoint. An object number is not a capability and this
Rust type provides no endpoint security. The existing volume service continues to
enforce domain access, streaming, quotas, content checks and flush/commit ordering.
Initialization is only permitted on `NotFound` with revision zero; a
deleted, foreign, unreadable or damaged object is not recreated
empty.

Every replacement streams the full canonical snapshot under the expected
revision, commits before ACK and then cleans up the volume version history.
The latter is essential: a plain WRITE would retain older active authority and
deleted ciphertext as recoverable history. The cleanup
keeps the current snapshot and deletes only its earlier versions.
An error, missing reply or uncertain outcome yields no success and
makes the adapter unusable for the next mutation.

A crash between WRITE and PURGE can leave a new current snapshot with
leftover history. A normal `load` writes nothing and
rejects that state with `ReopenRequired`. The trusted recovery controller
can **explicitly** run `finish_cleanup`: validate the current version, checksum and
NAS schema, then purge only history. Afterwards
the NAS core must reopen. No fallback to old authority, no silent
re-execution of the original guest/administration operation.

This is logical cleanup of the active volume table, **not physical secure
erase or protection against disk replay**. The old COW sectors, alternative
tables and external copies are not physically erased. The existing
volume checksum is not a MAC. Malicious disk rollback remains a separate
client/key/anchor problem; no NAS content key is introduced here.

The code can be built `no_std`, but is not yet a running native
NAS service. The integration tests use the actual volume client and
volume service with its existing simulated block device. Host execution
and fault injection do not replace QEMU, independent disk or P04 acceptance.

Targeted checks, with Rust 1.90.0, host linker and a dedicated target directory:

```sh
cargo test --offline --locked --manifest-path services/nas/native-volume/Cargo.toml
cargo test --offline --locked --release --manifest-path services/nas/native-volume/Cargo.toml
cargo clippy --offline --locked --manifest-path services/nas/native-volume/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path services/nas/native-volume/Cargo.toml --all -- --check
cargo build --offline --locked --manifest-path services/nas/native-volume/Cargo.toml --release --target x86_64-unknown-none
```

The release tests cover a NAS core plus volume client on a real
64 KiB host thread stack. The host volume service runs in a separate thread,
so that its stack is not wrongly added to that of the client.
This tests the memory budget and control logic, **not kernel IPC authentication**.
The existing volume test helper is reused read-only; no
second simulated block device has been added.
