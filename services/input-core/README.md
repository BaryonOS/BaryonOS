# Nieuw OS input service core

Allocator-free `no_std` core with `forbid(unsafe_code)`. Reuses the existing
`ui-core` PS/2 decoders, USB boot-report parsers and `InputRouter`; no device I/O, syscalls, new
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

## Boot-HID source profile

The first authenticated `SYNC` selects profile 1 (PS/2) or profile 2 (one boot
keyboard and one boot mouse). The profile stays fixed for the service epoch;
switching requires a new bootstrap lifetime. Existing PS/2 operations retain
their encoding. The following additional INP1 operations are source-only:

| Operation | Opcode | arg0 | arg1 |
|---|---|---|---|
| HID keyboard | 3 | Eight completed report bytes, little-endian | Zero |
| HID mouse | 4 | Completed bytes, little-endian, zero padding | Actual length 3–8 |
| HID detach | 5 | Mask: keyboard=1, mouse=2, both=3 | Zero |

The transport owner must authenticate device generations and publish only
matching successful boot-session completions. Numeric identifiers and report
bytes grant no authority. The service independently parses reports and retains
the same sender/endpoint, source sequence, epoch, focus and queue checks.

HID held state survives stream loss and `SYNC`: the first new absolute report
can release old suppression markers without inventing a new press. Actual
detach cancels queued/delivered state and clears only the detached role's known
holds. A complete keyboard batch updates physical ownership before publication;
overflow cannot leave half-applied releases or bind later presses to the new
cancel generation. Mouse screen-down Y is converted to the shared up-positive
event convention. Keyboard modifiers and current `PhysicalKey` positions are
mapped explicitly; usages outside that contract produce no key event. US
backslash/non-US hash aliases are coalesced, and simultaneous aliases are rejected.
Repeat timing, layouts, LED control, extra mouse buttons and wheels remain outside
this boot profile. No xHCI, DMA containment or physical-device support is implied.

Sources: USB-IF. (2001). *Device class definition for human interface devices
(HID), version 1.11*, Appendix B. https://www.usb.org/sites/default/files/documents/hid1_11.pdf
USB-IF. (2022). *HID usage tables, version 1.3*, Keyboard/Keypad page.
https://www.usb.org/sites/default/files/hut1_3_0.pdf
Atmel. (2009). *AT91 USB HID driver implementation* (6273B), p. 10.
https://ww1.microchip.com/downloads/aemDocuments/documents/OTH/ApplicationNotes/ApplicationNotes/doc6273.pdf

```sh
cargo +1.90.0 test --manifest-path services/input-core/Cargo.toml --locked --offline
cargo +1.90.0 clippy --manifest-path services/input-core/Cargo.toml --all-targets --locked --offline -- -D warnings
cargo +1.90.0 build --manifest-path services/input-core/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline
```

Use a dedicated `CARGO_TARGET_DIR` and the existing tool environment.
[Contract and native dependencies](../../docs/implementation/NATIVE-INPUT-SERVICE.md).
[Verifiable host/target report](verification/host-target.json).
