# Nieuw OS input service core

Allocator-free `no_std` core with `forbid(unsafe_code)`. Reuses the existing
`ui-core` PS/2 decoders and `InputRouter`; no device I/O, syscalls, new
kernel rights, native service loop or compositor. Four bootstrap-bound
RECV ingresses separate source, focus controller, lifecycle and app requests.

`Service::handle(endpoint, &Received)` validates endpoint, kernel sender,
tag/caps and the canonical INP1 payload. The native adapter must obtain `Received` directly
from the real kernel and explicitly close unexpectedly delegated caps.
`Received::from_bytes` in a host test does not by itself prove provenance.

A `Reply` selects only a pre-known `Route`; the caller maps it
to the pre-granted reply-SEND capability. `reply_is_current` is checked in
the same service thread immediately before send. PEEK retains input until
a valid ACK follows; focus/death/reset/overflow invalidate old event/cancel replies.
Empty and command acknowledgements are observations at handling time and
grant no focus; they do not clear a later pending cancel.
A separate cancel watermark takes precedence over ordinary events. Errors can
be handled with `error_reply` as a canonical negative status; there is
no fallback to a success or Empty reply.

Wire v1: 96 little-endian payload bytes; header `INP1`, version, opcode,
request number, epoch, arg0 and arg1. Replies use opcode|0x8000,
status in arg0 and the bound client ID in arg1. Status 0=Ok, 1=Empty,
2–18=`Error`. The event starts at byte 40: kind/action/key or buttons at
40–43, modifiers or signed delta at 44–47, event number at 48, focus generation
or cancel watermark at 56, pointer X/Y at 64/68. Bytes 72–95 remain zero.
A PEEK without an event has a zero-filled event body. All request bytes from
40 onward are zero; unused arguments are likewise zero.

```sh
cargo +1.90.0 test --manifest-path services/input-core/Cargo.toml --locked --offline
cargo +1.90.0 clippy --manifest-path services/input-core/Cargo.toml --all-targets --locked --offline -- -D warnings
cargo +1.90.0 build --manifest-path services/input-core/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline
```

Use a dedicated `CARGO_TARGET_DIR` and the existing tool environment.
[Contract and native dependencies](../../docs/implementation/NATIVE-INPUT-SERVICE.md).
[Verifiable host/target report](verification/host-target.json).
