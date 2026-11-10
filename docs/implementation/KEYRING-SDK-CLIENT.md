# KEY1 SDK client

The standalone crate `sdk/keyring-client` provides a `no_std`, allocator-free
client for the existing isolated key service. This is the reusable app channel
for key creation, revocation and encryption/decryption of small records.
Key material does not leave the key service through this API. The system app
does not yet use this new client; native client execution and the complete
encrypted storage chain remain open acceptance gates.

Implementation builds on
[B01](B01-ENTROPY-AND-KEYS.md), [B02](B02-SECURE-RECORDS.md),
[the storage contract](ENCRYPTED-OBJECT-STORAGE.md), `services/keyring/src/wire.rs`
and the existing SDK runtime. There are no kernel, grant, service or workspace
changes. The private `[workspace]` and lockfile let the integrator deliberately
connect the shared SDK later.

## Invariant and ownership

A `Session<T>` receives an already granted send/receive pair **assigned
exclusively to that session**, the expected service PID, `expected_domain`
and a bounded poll budget. The caller must not share the same pair, use it
outside the session or open a subsequent session on the same old handles.
`close()` and termination after protocol loss close both owned handles; there
are no hidden syscalls in `Drop`. The caller must execute `close()` before
dropping an unused session. Failed construction does not take ownership of
the handles. Close errors are returned; there is no claim that a rejected
kernel close released the object.

`expected_domain` checks only local context and response counts. It is not
evidence of rights. The server determines the actual domain through its
kernel-authorised ingress. Record opcodes 8/9 have **no domain field on the
wire**; the existing server decoder supplies that domain. Object, revision
and `RecordKind` are also requested context, not permission. A native broker
must authorise the object and action before granting a suitable channel.
The client does not remove the need for that missing product broker.

The client exposes only existing opcodes 1–5 and 8–9. There is no API for
recovery/snapshot export or restore (6/7), no new grant and no custom
cryptographic algorithm. These types do not form a server boundary: malicious
apps can construct wire bytes themselves. The native broker/server must
restrict existing broad development endpoints before third-party apps receive
access. Access to different record kinds must also be enforced there.

## Interface

`begin(Operation)` copies and validates the request; the returned `u64` is
only a correlation ID. `poll()` makes at most one IPC attempt and one yield,
returning `Poll::Pending` or a fully validated `Reply`.
`exchange(Operation)` uses the same bounded path synchronously.

| Operation | Typed response | Limits |
| --- | --- | --- |
| `Status` | `ServiceStatus` | Canonical state/reason, both key counters ≤16, entropy and counters do not decrease within this session. |
| `Generate` | `Generated(KeyIndex)` | Index 0–15; no reuse of an index already observed within this service lifetime. |
| `Revoke(key)` | `Revoked` | Empty successful body; a lost response is not a confirmed revocation. |
| `Seal` / `Unseal` | `Sealed` / `Plaintext` | Existing legacy context; plaintext 0–128 bytes, sealed exactly plaintext length +40. |
| `SealRecord` / `UnsealRecord` | `Sealed` / `Plaintext` | B02 context: volume, domain, object, revision, chunk, length, kind and schema; exact lengths. |

For names, the caller uses `RecordKind::Name`; for content,
`RecordKind::Content`. The key service authenticates these distinct contexts.
A name record therefore cannot be decrypted as content. The 128-byte limit is
the existing KEY1 record boundary, not a product limit for documents. The
integrated storage service must divide larger objects into records and obtain
the expected context from authenticated metadata. This does not by itself
protect against replay of an entirely old volume; an independent current
revision anchor and durable key recovery remain part of the storage contract.

Native use after the broker grants an exclusive connection (the original
comment means: use the key exclusively through this authorised service connection):

```rust,ignore
use nieuw_os_keyring_client::{NativeTransport, Operation, Reply, Session};

let mut keys = Session::new(
    NativeTransport, key_service_pid, expected_domain,
    granted_send, granted_receive, 4096,
)?;
let key = match keys.exchange(Operation::Generate)? {
    Reply::Generated(key) => key,
    _ => unreachable!("typed operation/reply contract"),
};
// Gebruik key uitsluitend via deze geautoriseerde dienstverbinding.
keys.close()?;
```

`KeyIndex` contains a public domain-local index, not a raw key or independent
access right. A new service lifetime requires a new broker connection and
session; there is no automatic reconnection.

## Errors, loss and cancellation

The parser requires the service PID, KEY1 tag, exactly 256 bytes and zero
attached capabilities. It then checks opcode, index, correlation ID, length,
zero padding, known status and the exact successful response shape. A
canonical service rejection becomes `Error::Service(Status)` and cannot yield
plaintext or ACK. Unknown status, forged sender, extra grant, incorrect ID or
malformed body terminates the connection. Unexpected received grants are
closed separately; close errors remain visible.

There is one pending operation. A second `begin` returns `Busy`. IDs increase
monotonically from 1; on exhaustion, the session closes before sending,
without wraparound. The poll budget persists across all polls and starts
separately for the send and receive phases. According to the SDK runtime,
`EAGAIN` on send means nothing entered the kernel queue. Other unclear send
outcomes receive `Delivery::Unknown`.

- Before enqueue: cancellation erases the owned request buffer and returns
  `NotPerformed`; the exclusive channel remains usable.
- After enqueue or uncertain publication: cancellation, timeout or protocol
  loss closes the connection and returns `MayHaveCompleted`. There is no
  KEY1 abort opcode; an already published Generate/Revoke may still execute.
  The client does not resend and reports no success without a valid response.
- A lost Generate or Revoke response requires recovery by the higher layer.
  Restarting the same action after a timeout may create a second key or
  encounter different state, and never happens silently here.

`Plaintext` has no `Clone`/`Copy`, shows only length in `Debug` and erases its
own buffer on drop. Owned request buffers are erased with `zeroize` on
destruction and immediately after a successful send. This does **not** prove
that all copies have been erased: caller buffers, temporary moves, SDK
`Outgoing`/`Received` and kernel/service buffers have their own lifetimes.
This client never receives key contents.

## Dependencies and evidence produced

Local dependencies are the existing SDK runtime and `keyring-core`; the
latter supplies only reused protocol/context types to the product client.
Host tests additionally use actual `capability-core` and the key-service core
with recognisable deterministic test entropy. That does not demonstrate
native entropy or native client execution.

The direct external dependency is the existing RustCrypto crate
`zeroize = 1.9.1`, default features disabled, Apache-2.0/MIT. The pinned registry
checksum is `e13084392c5e4bc371903e2935a5eaeed24905a7511356b883835e18a78f6879`.
Compared with baseline commit `3879c3288f3b005d9df3ed5e1e40aa99c3791521`, the
standalone lockfile adds no registry version/checksum. There is no new crypto
implementation or new build-script code. Origin is the same previously used
upstream RustCrypto/zeroize release:
<https://crates.io/crates/zeroize/1.9.1>, repository
<https://github.com/RustCrypto/utils>. The local package declares Rust ≥1.85,
no build script and Apache-2.0 OR MIT; this build uses Rust 1.90.0.

Executed on 30 October 2026 with Rust 1.90.0, offline and a private target directory:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-keyring-client-target \
  python3 -B tools/dev.py cargo +1.90.0 test \
  --manifest-path sdk/keyring-client/Cargo.toml --locked --offline
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-keyring-client-target \
  python3 -B tools/dev.py cargo +1.90.0 clippy \
  --manifest-path sdk/keyring-client/Cargo.toml --locked --offline --all-targets -- -D warnings
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-keyring-client-target \
  RUSTFLAGS='--cfg chacha20_force_soft --cfg chacha20_backend="soft" --cfg poly1305_backend="soft" --cfg sha2_backend="soft"' \
  python3 -B tools/dev.py cargo +1.90.0 build \
  --manifest-path sdk/keyring-client/Cargo.toml --locked --offline \
  --release --target x86_64-unknown-none --lib
```

Result: 3 unit tests and 12 host integration tests passed, no skipped tests;
Clippy and freestanding library build exited 0. Tests include actual key
creation/revocation, 0/1/128 bytes, 7 incorrect record-context/authentication
cases, malformed replies, enforced capability revocation, domain injection
through actual ingress, counter limits/regression, lost Generate/Revoke
responses and both cancellation phases. The bounded checks are recorded with
source/artefact hashes in `sdk/keyring-client/verification/`.

Not performed: native app integration, a new QEMU run of this client,
long-term storage/restart, physical hardware, complete object broker and
persistent key/snapshot/rollback integration. These remain explicitly open
and are not signed off by this SDK step. The lead agent manages release
acceptance and the progress meter.
