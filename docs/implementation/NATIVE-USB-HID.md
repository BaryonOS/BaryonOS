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

## Separate evidence cases

`tools/run_usb_hid.py --case reports` requires the exact ordered native USB
markers, QMP actions, successful artifact/source binding and raw QEMU exit 33.
`--case readonly` runs `test-usb-hid-readonly-fault`: xHCI attempts an event write
to the mapped read-only guard. The oracle requires the correct requester,
address, write direction and VT-d permission-fault reason 5. All 4,096 guard
bytes must remain unchanged. The original No-Op, unmapped-write and missing-DMAR
cases remain separate boots in `tools/run_xhci.py`.

The actual input-policy IPC experiment is documented in
[HID-INPUT.md](../../sdk/examples/input-service/HID-INPUT.md). It uses real
process isolation, authentication, focus and cancellation, with synthetic HID
completions. These experiments establish two separate paths: actual USB to
Session, and a validated Session producer to native input-policy IPC. Their
combination is not evidence of an operational USB-to-UI broker.

The next integration requires a bounded kernel/device-service completion
handoff, a long-lived requester lifecycle and an explicit coexistence contract
for other DMA consumers. Real IRQ delivery, hardware reset/reattach, general
hotplug, the physical reference matrix and human acceptance remain open.
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
