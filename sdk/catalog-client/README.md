# G03 — native transport for the existing catalogue client

`nieuw-os-catalog-client` connects the existing
[`catalog_core::client`](../../services/catalog/src/client.rs) to the
[SDK runtime](../runtime/README.md). The crate is allocation-free, `no_std`
and forbids `unsafe`. `NativeTransport` uses the actual int80 wrappers and is
available only on `x86_64-unknown-none`. There is no new syscall, capability
type, IPC version or storage format.

`Session::new` takes the service epoch, expected kernel PID, domain and two
already delegated endpoint handles. The session assumes responsibility for
those handles; do not use them outside the session. Construction grants no
rights. The kernel and service remain the actual authority boundary.

`exchange` supports existing `Operation::{List, Get, Put, Rename, Delete}`
and returns `Reply::{Catalog, Document, Mutation, NotFound, ServiceError}`.
Document names/contents are bounded slices of the validated B01 document.
PUT/RENAME/DELETE retain the existing expected-revision check. A
`ServiceError`, including `Unknown`, never confirms a mutation.

## Bounded waiting and outcomes

The caller selects a poll budget from 1 through 1,000,000 per phase. The send
and receive phases each have that maximum; the app yields the CPU between
attempts. Only `EAGAIN` before enqueue permits retrying send. Once enqueue is
confirmed, only receive is performed; no timeout, error reply or cleanup
causes the request to be resent.

| Delivery on error | Meaning |
|---|---|
| `NotEnqueued` | The adapter has published no request; for example, a full queue or ordinary negative syscall return |
| `Enqueued` | The kernel has confirmed enqueue; processing may already have occurred |
| `Unknown` | The syscall return is invalid; the adapter cannot rule out enqueue |

The pure B01 client conservatively treats an already prepared mutation as
potentially uncertain. The adapter additionally retains the more precise
transport outcome above. None of these errors demonstrates disk rollback,
cancellation or the absence of subsequent service I/O.

On timeout, transport failure, incorrect identity/correlation, unwanted
capabilities or a terminal service reply, the session closes. It attempts to
close both owned endpoints. Received capabilities are closed only after B01
has seen the **original** envelope; their presence is not hidden by clearing
`cap_count` first. Every cleanup operation is attempted even if an earlier
close fails; the first cleanup error remains visible in the result. A closed
session makes no further transport calls.

`close()` explicitly ends a healthy session. There is no hidden Drop syscall
or automatic restart. After uncertainty, a new session requires independently
established recovery and new grants; simply invoking the constructor again
with old handles is not a recovery strategy.

## Checks and example

```sh
cargo test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-catalog-client -p nieuw-os-sdk-runtime
cargo clippy --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-catalog-client --all-targets -- -D warnings
python3 -B sdk/examples/catalog-read/build.py
```

The [catalogue reader app](../examples/catalog-read/README.md) uses this typed
client for LIST→GET. Host tests use the actual pure capability model for
queues, PIDs, delegation, revocation and capability cleanup, and the existing
catalogue core for CAS and document results. A commit in that host model is
only an in-memory change: it is not native WRITE/FLUSH or durability evidence.
The native G02 runtime experiment and future ordinary app startup retain
their separate acceptance boundaries.

Source contracts: [M6c](../../docs/implementation/M6C-NAMED-CATALOG.md),
[M6d](../../docs/implementation/M6D-IO-UNCERTAINTY.md) and
[ABI/capabilities](../../docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md).
Source-bound checks are recorded under
[`parallel-g03`](../../docs/verification/parallel-g03/).
