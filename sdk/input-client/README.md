# Typed INP1 app client

Allocation-free `no_std` client for the existing 96-byte INP1 app interface.
This library provides the actual channel to the isolated input service;
it does not decode PS/2 hardware bytes or grant focus/input permissions.
`NativeTransport` uses existing native SDK IPC. Production source does not
construct `Received` from invented sender information.

## Interface and ownership

On success, `Session::new(transport, Binding { epoch, service_pid, client_id,
width, height, send, receive }, poll_budget)` takes over one exclusive endpoint
pair. The bootstrap must already have granted that pair. Do not pass borrowed
or shared handles. A failed constructor does not take ownership of the handles.
IDs/PID/geometry in `Binding` are expected correlation data; only the kernel
and service ingress determine actual authority. Width/height describe the
global GOP geometry, not the local app rectangle.

`peek()` returns `Peek::Empty` or `Peek::Offer(offer)`. An offer has typed
`epoch()`, `sequence()`, `generation()` and `event()` getters. `Event` directly
uses existing input-core/ui-core types. The app processes an offer once and
calls `ack(&offer)`. Until that exact ACK, the session rejects a new PEEK;
a second ACK of the same offer is also rejected locally.

`AckOutcome::Acknowledged` confirms an actual validated service ACK. A canonical
`StaleAck`, for example after losing focus, returns `AckOutcome::Superseded`.
The old offer is then obsolete. This outcome does not increment an ACK counter
or read/acknowledge a new cancellation. The app can cancel transient interaction
and receive the actual new cancellation message through the next PEEK.
`Cancel.generation` is a through-watermark: it may be older than the known
focus generation. The controller determines which old state expires.
Only key/mouse generations must not decrease; every event sequence must
strictly increase within the bound service epoch.

`close()` and `cancel()` close only this owned channel pair, each handle once;
both closes are attempted if the first fails. `Drop` attempts the same cleanup
without an implicit event ACK. Use explicit `close()` to observe a close error.
None of these functions is a global focus command. The trusted supervisor/broker
must also handle the service lifecycle when a binding expires; local close
does not claim that native integration.

## Protocol and failure contract

The client reuses the INP1 version/tag/opcodes/requests and status/event types
from the existing service crate. Replies undergo strict revalidation:
kernel sender, tag, no capabilities, exact length, magic/version/opcode,
requestID/epoch/clientID, status, all reserved bytes, enum values,
modifier/button bits, global coordinates, movement limits and overflow fields.
The 107 supported physical-key variants are explicitly enumerated; there are
no unsafe enum casts or decoder tricks. Unknown messages never become Empty,
an event or an ACK. Unexpected actual received capabilities are cleaned up
separately before the owned channel pair closes.

RequestIDs start at 1, are never reused and fail before wrapping. Only a known
kernel EAGAIN before enqueue may repeat the same unpublished call. After
enqueue, only receive/yield follows, never another send. Polling is finite
and bounded by `MAX_POLL_BUDGET`.

A lost or invalid reply closes the session. `Error::Interrupted` retains
`Delivery::NotEnqueued`, `Enqueued` or `Unknown`, plus transport, protocol
and cleanup errors where available. In particular, a lost ACK may already
have been executed; the client does not report success or replay the event.
A new session after uncertainty requires explicit recovery arrangements,
new grants and reconciliation by the app/broker. There is no automatic reset
or cloud/host fallback.

## Executed evidence and remaining limits

This standalone crate builds offline with its own lockfile. The only new
type dependency is the safe public `MouseButtons::checked` constructor
provided by the UI owner, source commit `5f887648`.

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=build/input-client-target python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/input-client/Cargo.toml
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=build/input-client-target python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked --manifest-path sdk/input-client/Cargo.toml --all-targets -- -D warnings
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=build/input-client-target RUSTFLAGS='-C no-redzone=yes -C target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2' python3 -B tools/dev.py cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/input-client/Cargo.toml --target x86_64-unknown-none --message-format=json-render-diagnostics
```

Thirteen tests pass: one reachable internal exhaustion boundary and twelve
targeted exchanges through the actual capability engine and input service,
including actual queue EAGAIN, focus cancellation between PEEK/ACK, lost
PEEK/ACK replies, invalid sender/tag/wire, replay, invalid mouse fields,
actual received capabilities and a failed close. The host adapter uses test
data; this is not native keyboard, compositor or Materiaal acceptance.
Clippy/fmt and the scalar none-target build pass. Concrete source/artifact
hashes are recorded in `verification/pre-native.json`.

The native app caller, actual input-service IPC in the combined image,
supervisor lifecycle and QMP/interaction evidence remain separately required.
This SDK foundation does not complete the full test release.
