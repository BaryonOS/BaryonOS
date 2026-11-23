# Native USB HID transport fixture

The opt-in `native-usb-hid-probe` executes actual xHCI enumeration and interrupt
transfers in the kernel. QEMU input events produce device reports that enter the
existing `nieuw_os_usb_core::boot_session::Session`. This is a bounded transport
fixture, not the production USB-to-input-service path or H02 acceptance.

## Supported reference profile

The runner selects one QEMU `1b36:000d` controller at PCI `00:05.0`, two direct
USB2 ports, a keyboard on port 1 and a mouse on port 2. Both devices explicitly
use `usb_version=1`. The existing Low/Full-speed session contract remains in
force; the fixture requires the supported eight-byte endpoint-zero packet size.
High-speed devices, hubs, additional interfaces, unexpected completions and
out-of-profile descriptors fail closed. The CI machine is `pc-q35-8.2`.

The controller and VT-d register/authority checks are shared with
[the DMA fixture](NATIVE-XHCI-DMA.md). Initialization uses one bootstrap CPU with
interrupts disabled. Device interrupts remain masked, and completions are polled.
The USB profile cannot be combined with legacy xHCI negative profiles, the
application scheduler, SMP or other conflicting bootstrap consumers.

## DMA permissions and lifetime

The same real requester-bound domain maps fifteen pinned pages. The direction
comes from each private DMA record and cannot be supplied by a completion or
an untrusted numeric handle. Duplicate physical pages are rejected.

| Allocation | Pages | Device permission |
|---|---:|---|
| Command ring, ERST and DCBAA | 3 | Read |
| Event ring | 1 | Write |
| Shared input context | 1 | Read |
| Per-device output contexts | 2 | Read and write |
| Per-device endpoint-zero rings | 2 | Read |
| Per-device interrupt rings | 2 | Read |
| Shared descriptor buffer | 1 | Write |
| Per-device report buffers | 2 | Write |
| Independent permission guard | 1 | Read |

Only the two output contexts need the new `DeviceReadsWrites` direction. The
existing read-only and write-only mappings retain their permissions. Generic
frame release, reallocation, executable aliases and user aliases remain denied.

The opt-in `native-usb-hid-reattach` profile adds exactly one sixteenth page,
the limit of both the VT-d fixture and the device-execution model. It is
granted before the domain is established; no grant is ever added afterwards.

| Allocation (reattach profile only) | Pages | Device permission |
|---|---:|---|
| Generation-2 device: output context at offset 0, report buffer at `0x800` | 1 | Read and write |

The `native-usb-hid-mouse-reattach` profile reuses this same sixteenth page
for the generation-2 mouse; it adds no page, grant or direction. Both reattach
profiles select the internal `usb-hid-reattach-core` Cargo feature and are
mutually exclusive, and both are exclusive with the read-only fault profile
(`compile_error!`, cargo exit 101).

This page holds only zeroed memory, xHC-maintained context and report bytes.
The 1 KiB output context requires the 32-byte context size, which the fixture
already enforces (`HCCPARAMS1.CSZ=0`). Report submission clears only the
eight-byte report region, never the live context in the same page.

## Transfer and unplug behavior

Enumeration performs port reset, Enable Slot, Address Device, bounded device
and configuration descriptor requests, SET_CONFIGURATION, Configure Endpoint
and SET_PROTOCOL. The existing descriptor selector and session encoder are used.
Successful control completion requires the matching Status event; a short Data
event only determines the validated received length.

Every command and transfer completion is matched to its event type, slot,
endpoint and exact TRB address. The fixture never wraps or reuses a submitted
TRB. Control-data and input-report residual lengths must not exceed the submitted
request. Only a matched
completion with a valid actual length is passed to the pending Session token.

After enumeration, the host waits for guest readiness before sending QMP input.
The positive case verifies A press/release, relative mouse movement of +7/-3,
left-button press/release, and B held while the keyboard is removed. Actual port
disconnect is followed by correlated Stop Endpoint and Disable Slot completions.
Session detach then releases B once, rejects the old token and produces no
second release on repeated detach.

Device lifetimes come from the existing `hotplug::Ports` tracker: device IDs
1 and 2 are the two root ports, the instance attach budget is three and each
attach permits one enumeration attempt. Each Session uses the lifetime of its
attach ticket, and only a real, matched enumeration result completes the
ticket. In every profile both boot-time devices therefore use generation 1.
The earlier literal generation 2 for the mouse was internal only; markers,
QMP actions and exit status are unchanged.

Polling, descriptor storage, transfer rings and host output are bounded. The
host controller uses a shared deadline and QMP pipes, records its actual control
actions, rejects control errors and reaps the process on every outcome.
An unrelated event during the fixed unplug sequence terminates the fixture;
this implementation does not provide a general event distributor.

Every execution outcome after domain establishment attempts controller stop,
PCI bus-master clear and domain denial. Initialization failures retain their
allocations, and the outer fixture attempts controller stop before a terminal
failure. Success requires completed halt/invalidation and an unchanged guard.
All DMA and translation-table pages remain retained until the VM exits,
including after a failed or uncertain hardware transition.

## Same-port keyboard reattach

`native-usb-hid-reattach` extends the positive sequence after
`USB:STALE_TOKEN:DENIED`. It is exclusive with the read-only fault profile.

1. **Retire generation 1.** Disable Slot has already completed. `Ports` then
   detaches port 1, and the slot's DCBAA entry is cleared. The generation-1
   output context and report pages stay mapped in the fixed domain. They are
   filled with a canary. The final comparison *detects* a later stray device
   write; it does not prevent one. `USB:REATTACH:DETACHED` follows.
2. **Wait for a connect.** Only a port-status event for port 1 with CCS=1,
   PED=0 and `Ports` phase `Detached` is accepted. Any other event fails closed.
3. **Enumerate generation 2.** `Ports::attach` issues a generation-2 ticket.
   Port reset, Enable Slot, Address Device, descriptors, SET_CONFIGURATION,
   Configure Endpoint and SET_PROTOCOL run again with a fresh Session. The
   endpoint-zero and interrupt rings continue: each new dequeue pointer is the
   ring base plus the next unused TRB index, with DCS=1. Generation-1 TRBs keep
   cycle=1 and are never reused or wrapped onto.
4. **Reject generation-1 work after slot reuse.** QEMU 8.2.2 reuses the
   disabled slot (`USB:REATTACH:SLOT_REUSED`; `SLOT_NEW` would be reported
   otherwise and is not accepted by the oracle). With generation-2 input
   pending, three checks must fail:
   - a *replayed captured completion*: the last real generation-1 Transfer
     Event is presented again to the generation-2 correlation. It is not a
     hardware-generated event. Slot and endpoint match, so only the
     never-reused TRB address rejects it;
   - the generation-1 Session token, presented to the generation-2 Session,
     returns `UnexpectedCompletion`;
   - the generation-1 attach ticket returns `Stale`. This check is
     phase-based: generation 2 is already `Ready`, so any ticket would be
     refused here.
   The generation-specific ticket check runs earlier, inside the generation-2
   attach while that port is still `Enumerating`. The phase test then passes,
   so only the generation-1 lifetime and attempt refuse the old ticket. The
   port must stay `Enumerating`, and the generation-2 ticket is accepted in
   that same phase after enumeration. A failure reports
   `USB-DIAG:REJECT:0000000000000033` (code 51). The host test
   `detach_during_enumeration_retires_old_completion_and_releases_port`
   covers the same rule at library level.
   The generation-2 pending transfer remains unchanged and later accepts its
   real completion. `USB:REATTACH:STALE_DENIED` and `USB:REATTACH:READY`
   follow.
5. **Use and unplug generation 2.** The C press and release produce
   `USB:REATTACH:KEY_DOWN` and `USB:REATTACH:KEY_UP`. Removal uses the same
   correlated Stop Endpoint and Disable Slot sequence, an empty expected
   release set, stale-token rejection, `Ports` detach and DCBAA clearing.
6. **Refuse a third identity.** A further connect on port 1 must fail with the
   attach budget error `Exhausted`. The port is never reset, no slot is
   enabled and PORTSC must read CCS=1 and PED=0. A bounded quiet window then
   requires no further event and an unchanged CCS=1/PED=0 state. This window
   also lets QEMU answer `device_add` before the guest exits; without it, the
   reply was lost in the first local run.
7. **End.** The controller halts, bus mastering is cleared and the domain is
   denied. The two retired canaries (`USB:RETIRED:UNCHANGED`) and the guard
   must be unchanged. All pages remain retained until the VM exits.

**Fixture-only pattern.** The retired generation-1 read/write pages stay
mapped in the VT-d domain until the VM exits. The canaries show only that
their bytes were unchanged at the final check; a write that leaves or restores
the canary bytes would pass, so the fixture does not prove that no write
occurred after retirement. This is acceptable for this fixture, not as
a production reattach pattern: a product driver must unmap or reassign retired
DMA pages and invalidate the IOTLB before reuse (integration review of #409,
23 November 2026).

Every PORTSC write preserves PP and writes only write-1-to-clear change bits
or PR; PED=1 is never written. Interrupts remain masked. A connect event
during the report phase fails closed in this profile, whereas the earlier
profile still ignores one. The event ring remains a single non-wrapping
256-entry segment; the complete reattach sequence stays well below it.

The harness drives QEMU as follows: release B after
`USB:STALE_TOKEN:DENIED`; after `USB:REATTACH:DETACHED`, issue `device_add`
for `usb-kbd` `hid-kbd-g2` on `xhci.0` port `1` with `usb_version=1`; press and
release C; `device_del hid-kbd-g2`; and after `USB:REATTACH:UNPLUG:PASS`,
`device_add hid-kbd-g3`, which the guest must refuse. Every command reply is
required, held keys are released on every error path and the process is
always reaped. QEMU device addition and removal are emulated hotplug, not
physical hotplug.

## Same-port mouse reattach

`native-usb-hid-mouse-reattach` is a separate boot. It runs the complete,
unchanged `reports` sequence, including the keyboard unplug with B held, and
then re-enumerates the mouse on root port 2 instead of the keyboard.

1. **Retire the keyboard first.** Directly after `USB:REPORTS:PASS`, `Ports`
   detaches port 1 and DCBAA[keyboard slot] is cleared. The generation-1
   keyboard output context (page 5) and report page (page 12) are filled with
   the canary before any later Enable Slot can hand that slot out again.
2. **Unplug with a button held.** The guest waits for a generation-1 left
   press without movement (`USB:REATTACH:HELD_BUTTON`). Movement-free empty
   changes are ignored and resubmitted; any other mouse change (code 53), an
   event on another slot (code 54), or a port event that is not the port-2
   disconnect after the press (code 52) fails closed. On the port-2 CCS=0
   event the correlated Stop Endpoint and Disable Slot run, and detach must
   release exactly `buttons_released = 1`. The stale token is refused and a
   repeated detach must be the mouse's empty change.
3. **Reattach.** The shared `reattach()` step is the single owner of retiring
   the mouse: `Ports` detach of port 2, DCBAA[2] cleared, the generation-1
   mouse output context (page 6) and report page (page 13) filled, and
   `USB:REATTACH:DETACHED`. Only a port-2 connect with PED=0 while port 2 is
   `Detached` starts the generation-2 attach into the spare page; the old
   mouse ticket must be `Stale` while the new one is `Enumerating` (code 51).
   The new device must again be a boot mouse (code 44),
   `USB:REATTACH:ENUM:MOUSE`.
4. **Slot reuse is checked, not assumed.** QEMU's Enable Slot is expected to
   return the lowest free slot, so the generation-2 mouse receives the retired
   keyboard's slot (`USB:REATTACH:SLOT_REUSED:KEYBOARD`). QEMU 10.2.1 on the
   local host showed this; hosted CI (QEMU 8.2.2) does not run this case yet.
   `SLOT_REUSED:MOUSE` and `SLOT_NEW` are printed if QEMU differs, and the
   oracle refuses both.
5. **Reject generation-1 work by named source.** Both real generation-1
   completions are replayed against the generation-2 pending transfer:
   the keyboard's last completion (its source slot is the reused slot, so
   slot and endpoint must match and only the never-reused TRB address, input
   ring page 9 versus page 10, rejects it) and the mouse's last completion
   (refused by slot). Each event must carry its recorded source slot (code 45).
   The generation-1 mouse token and the generation-1 keyboard token must each
   return exactly `UnexpectedCompletion`, the old mouse ticket `Stale`, and the
   pending transfer must remain unchanged.
6. **Use, unplug and budget.** One QEMU event with x −4, y +6 and left down
   gives `USB:REATTACH:MOUSE_DOWN`; the release gives `USB:REATTACH:MOUSE_UP`
   (code 55 otherwise). The generation-2 unplug expects an empty release set,
   then `USB:REATTACH:UNPLUG:PASS`. The third mouse identity on port 2 is
   refused by the instance attach budget (keyboard g1, mouse g1, mouse g2),
   `USB:ATTACH:BUDGET_DENIED`.
7. **End.** After halt, bus-master clear and domain denial, pages 5, 12, 6
   and 13 must still hold the canary (`USB:RETIRED:UNCHANGED`); the guard must
   be unchanged.

**Evidence limits.** In this profile the generation-1 mouse event is refused
by slot; same-device TRB-address-only rejection remains keyboard-profile
evidence. This profile adds the cross-device case in which only the TRB
address rejects the keyboard event in the reused slot. The token checks do
not isolate the Session lifetime field: the generation-1 tokens have a higher
sequence number than the generation-2 pending token, so they would be refused
even without it. Lifetime isolation is covered by the `drivers/usb` host
tests; only the in-attach ticket check (code 51) is generation-specific.
The QEMU usb-mouse boot report is at most four bytes; wheel bytes are not
exercised. The canary pattern is fixture-only, as described above.

The harness sequence is: release B after `USB:STALE_TOKEN:DENIED`; press left
after `USB:REPORTS:PASS`; `device_del hid-mouse` after
`USB:REATTACH:HELD_BUTTON`; after `USB:REATTACH:DETACHED`, release left (it
reaches only QEMU's PS/2 mouse) and `device_add` `usb-mouse` `hid-mouse-g2` on
`xhci.0` port `2` with `usb_version=1`; after `USB:REATTACH:READY`, send x −4,
y +6 and left down in one event, then left up; `device_del hid-mouse-g2`; and
after `USB:REATTACH:UNPLUG:PASS`, `device_add hid-mouse-g3`. That is fifteen
QMP commands; every reply is required and held keys and buttons are released
on every error path.

## Separate evidence cases

`tools/run_usb_hid.py --case reports` requires the exact ordered native USB
markers, QMP actions, successful artifact/source binding and raw QEMU exit 33.
`--case readonly` runs `test-usb-hid-readonly-fault`: xHCI attempts an event write
to the mapped read-only guard. The oracle requires the correct requester,
address, write direction and VT-d permission-fault reason 5. All 4,096 guard
bytes must remain unchanged. The original No-Op, unmapped-write and missing-DMAR
cases remain separate boots in `tools/run_xhci.py`. `--case reattach` runs
`native-usb-hid-reattach` and requires the complete `reports` marker sequence,
then the reattach markers above in exact order, every QMP reply and raw exit 33.
Its source-bound results are in
[the H02-T3 evidence](../verification/h02-native-reattach-20261119/README.md).
`--case reattach-mouse` runs `native-usb-hid-mouse-reattach` with the same
rules and the mouse reattach markers; its results are in
[the H02-T4 evidence](../verification/h02-native-mouse-reattach-20261123/README.md).

The actual input-policy IPC experiment is documented in
[HID-INPUT.md](../../sdk/examples/input-service/HID-INPUT.md). It uses real
process isolation, authentication, focus and cancellation, with synthetic HID
completions. These experiments establish two separate paths: actual USB to
Session, and a validated Session producer to native input-policy IPC. Their
combination is not evidence of an operational USB-to-UI broker.

The next integration requires a bounded kernel/device-service completion
handoff, a long-lived requester lifecycle and an explicit coexistence contract
for other DMA consumers. QEMU same-port keyboard and mouse reattach now exist
as separate profiles. Real IRQ delivery, physical reset/reattach, general hotplug
(other ports, hubs, retries), the physical reference matrix and
human acceptance remain open.
Source-bound results and the standard gates are recorded in
[the checkpoint evidence](../verification/issue42-native-input-20261114/README.md).
H02 and the 41/100 assessment remain unchanged.

## References

Intel Corporation. (2025). *Extensible host controller interface for Universal
Serial Bus* (Revision 1.2c).
https://cdrdv2-public.intel.com/868295/xHCI__Rev1.2c.pdf

USB Implementers Forum. (2001). *Device class definition for human interface
devices (HID)* (Version 1.11).
https://www.usb.org/sites/default/files/documents/hid1_11.pdf

QEMU Project. (2025). *USB HID and xHCI device implementations* (Version 10.0.0)
[Source code].
https://github.com/qemu/qemu/blob/v10.0.0/hw/usb/dev-hid.c
https://github.com/qemu/qemu/blob/v10.0.0/hw/usb/hcd-xhci.c
