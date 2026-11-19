# Task grant policy core (host model)

This dependency-free, allocation-free `no_std` model binds explicit read, write
and export rights to an app, a specific process incarnation (PID and epoch),
and a registered object revision. Installation, internet permission and privacy
explanation preferences confer no rights. There is no delegation, inheritance,
IPC message, serialization or capability transfer API.

## Trusted adapter contract

The future trusted adapter exclusively owns `Grants` and all model handles.
It authenticates the app principal and the current PID/epoch from the execution
boundary, supplies authoritative object IDs/revisions, and invokes `grant` only
after an explicit trusted permission decision. App-supplied identity fields,
object numbers or handles never authenticate an operation. The adapter checks
`authorize` at every protected operation, serializes lifecycle events with those
operations, and prevents a check/use race with revision changes and revocation.

Each table represents one session. Its session ID must be globally unique across
table instances, restarts and retained handles; PID/epoch pairs must not repeat
while grants can survive. These identifiers are trusted inputs, not credentials
or cryptographic tokens. Never clone/reconstruct a live table or reuse its session
ID. `end_session` permanently closes it; a new session requires a fresh table ID.

`register_object` explicitly introduces authoritative objects; grants and checks
cannot register an unknown object or revision. Revisions advance monotonically
through `advance_revision`, which retires existing grants for that object. The
adapter must report deletion as an authoritative revision change before any
further access and must not grant deleted objects. Object slots are not evicted.

`close(actor, object)` ends all until-close grants for that actor/object, while
session grants remain until session end or explicit revocation. A subsequent
trusted grant represents a new open. Process exit/restart must be reported with
`retire_process` and authenticated operations use the new epoch. All lifetimes
are event bounded; this model implements no wall-clock timeout.

Handles stay inside the trusted adapter and carry a session, slot and generation.
Revoke, close, process retirement, revision advance and session end invalidate
retained copies. Successful issuance consumes a monotonically increasing
generation; overflow permanently prevents further issuance in that table, without
disabling revocation or existing valid grants. Full tables reject insertion without
eviction. Zero-capacity tables reject insertion as well.

## Task contract and verification

Goal: implement A03-T1 only. Invariant: no operation succeeds without a live,
matching actor/object/revision/right grant; stale handles never regain authority.
Write set: this crate, `sdk/Cargo.toml` and the local member entry in
`sdk/Cargo.lock`. Every existing registry dependency version remains unchanged.

Acceptance commands:

```sh
cargo +1.90.0 test --manifest-path sdk/Cargo.toml --locked
cargo +1.90.0 clippy --manifest-path sdk/Cargo.toml --locked --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml --all -- --check
cargo +1.90.0 build --manifest-path sdk/Cargo.toml --locked -p nieuw-os-task-grant-core --release --target x86_64-unknown-none
```

Evidence limit: host policy tests and freestanding compilation do not demonstrate
native enforcement, trusted UI, IPC identity authentication, volume per-object
enforcement or end-to-end LP acceptance. Revocation cannot erase copies already
read. Read plus network permission may permit exfiltration even without the
separate export operation. A03 remains open.
