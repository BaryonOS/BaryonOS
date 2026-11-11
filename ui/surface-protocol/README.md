# SUR1 — app to separate compositor

Pure `no_std`, safe Rust, no allocator/runtime/device I/O. Root grants only
the compositor a DISPLAY cap; the app has exclusively its assigned
Surface-SEND and its own reply-RECV. The adapter checks the actually received
kernel PID, tag `0x5355524641430001` and zero caps before it calls `Receiver`.
A payload or host-constructed envelope proves no sender or authority.

The 48-byte little-endian header contains `SUR1`, version1 u16@4, opcode u16@6,
request ID u64@8, surface epoch u64@16, Controller frame generation u64@24,
tile index u64@32 and two u32 arguments@40/44. IDs/epoch are non-zero. No
Rust struct casts or loose padding material. Op1=INFO,2=BEGIN,3=CHUNK,4=COMMIT.

| Operation | Arguments | Body |
|---|---|---|
| INFO | frame/tile/args zero | empty; total48B |
| BEGIN | local x/y | width u16, height u16, pixelbytes u32; total56B |
| CHUNK | exact next offset / data length | at most208 bytes; total≤256B |
| COMMIT | args zero | empty; total48B |

One request ID applies to the whole ordered tile transaction; at most five chunks
for1024 bytes. Each chunk uses exactly min(208, remaining), without overlap,
gap, repetition or extra bytes. BEGIN/CHUNK send no positive ACK. INFO is
a separate RPC with its own increasing request ID. A published
transaction is not automatically resent on an uncertain reply.

A reply has the same binding and opcode|0x8000. Status u32@40 is0 on success,
otherwise an `Error` number1..14; pixels u32@44 only on a successful COMMIT.
The INFO reply is80B: body32 with screen width/height u32@48/52, assigned global
x/y u32@56/60, assigned width/height u32@64/68, tilemax32/8 u16@72/74,
format1 u32@76. The COMMIT reply is48B and reports exactly the number of written
pixels. Negative replies have pixels0 and no non-zero body.

`Geometry::rectangle(tile_index)` determines canonical local row-major32×8 tiles,
with exactly smaller edges. The assignment fits entirely within at most1920×1080;
the compositor only adds the global offset after complete validation. No app
can thereby paint the mandatory system border. Pixel bytes are packed
LEu32 `0x00RRGGBB`; every fourth byte is zero. The whole tile is validated before I/O.

`Receiver::new(peer, geometry, timeout_ticks)` keeps only one1024B tile.
`accept(kernel_sender, monotone_tick, payload)` returns Staged, Info(Reply) or an
opaque CommitReady. Only `commit_with(token, callback)` can produce a positive
tile reply and mandatorily executes that callback with the validated
global rect/the complete pixels. Root uses **the real**
`DisplayClient::blit` here; a host recorder/error callback is exclusively a test,
not native evidence. There is no public positive response builder.
The exact signature is `commit_with(token, now_tick, callback)`: the trusted
native adapter samples the actual monotonic SDK tick directly at commit, not a
payload/host/wall-clock value. Expiry happens before payload decode and again
at commit admission before the first blit. This promises no vsync deadline across
later preemption and does not retroactively undo written pixels.

A newer frame generation starts only at tile0 and replaces an unfinished frame;
older/overlapping tiles are forbidden. `completed_frame()` is only Some
after all canonical tiles have actually been confirmed via callbacks. App-side
`Controller::presented` likewise remains bound to the current generation/epoch.
This is no atomic page flip, vsync or desktop acceptance.

Root samples existing native monotonic time and calls `expire(now)` in its
service loop. Timeout/clock rollback/retire never yield success. `reject`
produces at most one negative reply for a canonical current actor/epoch
transaction and clears that staging; further fragments of the same failed ID
are ignored without a reply storm. The adapter retains a reply still to be sent
on a real IPC EAGAIN and then receives/processes no new app request. It
closes unexpectedly received caps. Source grants are not shared or
taken over from app payloads. A new service lifetime requires a new epoch and
grants; an equal reused epoch proves no new authority.

From the repository root with Rust1.90.0:

```sh
cargo +1.90.0 test --offline --locked --manifest-path ui/surface-protocol/Cargo.toml
cargo +1.90.0 clippy --offline --locked --manifest-path ui/surface-protocol/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 build --offline --locked --manifest-path ui/surface-protocol/Cargo.toml --release --lib --target x86_64-unknown-none
```

Host tests actually check this state machine, malformed payloads,
actor/epoch/replay, chunk order, edges, callback failure and timeout. They prove
no kernel sender, capability enforcement, pixels on GOP or native input/app operation.

## Optional app client

Feature `client` adds only the existing SDK runtime/catalog Transport;
the default remains the pure dependency-free assembler. `client::Session::new` takes
ownership of one Surface-SEND/reply-RECV pair after valid construction. `geometry`
verifies the real compositor reply; `blit(frame,tile,rect,pixels)` publishes
BEGIN/five ordered chunks/COMMIT and accepts exclusively a fully
bound reply with the exact pixel count. Only not-yet-published sends may
be retried after kernel EAGAIN. Lost reply/unknown send/protocol error closes both own
channels without republishing a tile. `close` is explicit and
idempotent. Draft/dirty/presented remain the responsibility of the app;
an error may never sign off that state as confirmed.

`verification/freeze.json` keeps the original 8-test freeze. The current
timeout/client evidence is separately in `verification/deadline-fixed.json`:
the new regression actually failed on the old source, the correction has
15 host tests and independent targeted re-review. No native compositor claim.
