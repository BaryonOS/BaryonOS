# Boot-HID publication adapter

`hid::Source` connects validated `nieuw_os_usb_core::boot_session::Session`
completions to the existing protected INP1 source ingress. It is a software
adapter, not an xHCI driver or evidence of physical USB operation.

1. Construct `Source` from the existing bootstrap `source::Configuration`.
2. Call `synchronize(now)` and drive `step(now, &mut Ipc)` until the matching
   profile-2 acknowledgement. `NativeIpc` uses the existing kernel SEND/RECV calls.
3. The enumeration/transport owner selects each interface, issues its boot
   protocol request, and authenticates completion in its `Session`.
4. Submit input through that Session and pass its token, transport status and
   exact completed bytes to `Source::complete_input`. Failed, stale, malformed
   or oversized completions create no publication.
5. Drive `step` until acknowledgement before submitting another report. EAGAIN
   retains the original message and absolute 200-tick deadline. Sender, request,
   epoch, operation and zero-capability reply shape are checked; fatal errors
   require controlled lifetime recovery, never a fabricated success.

On disconnect, `detach(now, &mut Session)` invalidates USB work immediately.
If publication is busy, retain and retry that detach notification after the
previous acknowledgement. No later input may be published for the detached
Session. Reattach needs a fresh owner-assigned generation and protocol selection.
The owner must receive the old role's detach acknowledgement before publishing
its replacement, and must never replay that old detach against the replacement.
Do not share a role between two live devices. The adapter cannot establish
physical identity, DMA/IRQ isolation, configuration completion or focus rights.

For bus reset/loss, quiesce and reset the affected Sessions separately, then
synchronize. The input policy retains prior physical suppression until fresh
absolute reports release it; synchronization does not manufacture releases.
Actual detach explicitly clears only the lost role. The existing PS/2 source
implementation and its native binary remain unchanged.

Host tests run through the real capability-engine model and policy adapter.
They do not prove real kernel execution, USB enumeration or hardware support.
A separate native fixture must exercise this adapter, actual IPC and client
observations on the own-kernel image, with synthetic reports labelled as such.

## Dedicated own-kernel fixture

The opt-in `native-hid-input-probe` kernel profile selects `input-hid-source`,
the unchanged production `input-policy`, and `window-hid-probe`. It retains the
existing window-session manifest and its protected source, focus and client-100
channels. Other fixed actor slots park without publishing input. The test source
supplies synthetic completions to actual USB `Session` objects; both publication
and client observations use real kernel IPC. This is not USB transport evidence.

The bounded script checks keyboard modifiers, mouse displacement/button mapping,
focus-loss cancellation and a superseded ACK, reset/resync suppression, keyboard
detach while a mouse button remains held, reattach with a fresh generation, and
mouse detach while a keyboard key remains held. Failed, malformed, oversized and
stale completions must not become publications. The controller also sends a HID
operation through its focus ingress and requires the actual `WrongRole` reply.
Every expected cancellation is consumed through PEEK and acknowledged through
the client API. The kernel observer coordinates steps using existing private
support pages; those numeric observations never select identities or grants.
The terminal pass also requires capability-engine cleanup and frame conservation.

`sdk/examples/{input-service,window-session}/build.py --hid-probe` builds and
inspects the two feature-gated fixture ELFs. A complete runtime verdict requires
the separately recorded own-kernel build and `tools/run_hid_input.py` execution;
the component's host/target receipt does not establish that result.

### Observed integration run, 2026-11-14

The own-kernel QEMU gate completed the boot, keyboard, mouse, focus, reset,
detach, reattach and rejection stages, followed by `HID:CLEANUP:PASS` and
`HID:CASE:PASS`. The raw QEMU exit was 33 with no timeout. Its generated
`build/hid-input-report.json` records these exact observations:

| Counter | Value | Meaning |
|---|---:|---|
| Source publications | 11 | Authenticated source requests acknowledged by the real policy process |
| Source rejections | 6 | Stale, failed, short, oversized or duplicate-usage completions rejected before publication |
| Client events | 10 | Events observed through the real client PEEK channel |
| Client cancellations | 4 | Focus loss, resync, keyboard detach and mouse detach, each followed by ACK |
| Superseded ACK | 1 | A previously offered mouse event invalidated by focus loss |
| Role denial | 1 | HID operation on the controller ingress rejected with `WrongRole` |
| Policy rejections | 2 | The corresponding `StaleAck` and `WrongRole` responses |
| Checked receive rights | 15 | All four production ingresses checked; bitmask `0b1111` |

The recorded integration receipt binds kernel SHA-256
`9b655ccc2028ce4f7100baef0fa0edb1e31c65c93ee2bea270257cb9c7e3d732`
and source fingerprint
`77ea25f517012b934a39d29502673bc43a92b90854cbf175e07dd790d874d65a`.
This identifies the observed run, before the final integrated revision is frozen.
The report explicitly records synthetic reports and native IPC assertions, with
USB enumeration, physical hardware and rendering outside this result.
