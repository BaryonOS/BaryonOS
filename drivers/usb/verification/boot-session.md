# H02 — bounded HID boot-session completion

Date: 14 November 2026. Issue: #42. Continuation of PR #328.

## Implemented contract

`boot_session::Session` joins the existing configuration selector, HID setup
encoder and keyboard/mouse state parsers. Construction selects from the complete
configuration bytes; an arbitrary public `BootInterface` value cannot replace the
private binding. One operation can be pending. Input submission is denied until
the matching `SET_PROTOCOL(boot)` completion reports success and zero data bytes.

Opaque correlation tokens bind the owner-supplied device identity, attach
generation, interface, operation kind and checked sequence. An incorrect, stale
or duplicate token cannot consume current work. Matching failed or malformed
completions are consumed without changing held input state. A sequence never
wraps or restarts after reset; exhaustion refuses new submissions.

Reset invalidates pending work, releases known keys/modifiers/buttons and requires
protocol selection again. Detach is terminal. Repeated clearing emits no duplicate
releases. The owner must quiesce transport separately and restore the configured
device and alternate zero after a bus reset. Changed descriptors or device identity
require a newly selected session.

The initial component handles one packet per input transfer, bounded by
`min(8, max_packet_size)`. It rejects keyboard endpoints below eight bytes and
mouse endpoints below three bytes. This is a deliberately limited implementation
profile, not a statement that USB forbids other endpoint/report arrangements.
The existing boot-mouse displacement policy is unchanged.

## Trust and evidence limits

The enumeration owner must supply a nonreused lifetime identity, bind the
descriptor snapshot to the configured device, authenticate completion routing,
and provide exactly the completed bytes. Numeric identities and tokens grant no
device or input authority. A failed input transfer preserves held state; the
owner must reset or detach on terminal loss. This module does not provide timers,
keyboard repeat, `SET_IDLE`, focus policy or event delivery.

No controller, DMA buffer, MMIO mapping, IRQ, kernel API or dependency is added.
The xHCI crate still has no execution/ring adapter. Kernel bus-master enablement
remains test-only pending enforced DMA containment. Native USB enumeration,
HID transport, hotplug/reset/unplug, focus/stuck-key acceptance, physical reference
hardware and H02 acceptance remain open. The ordinary OS smoke gate does not
execute this standalone USB component.

## Verification

Published implementation: `95871d00da8300bd542609930643af7e96e179dd`.
Locally tested commit: `c4cb446ec78bed810df0e2a73cc75d51e3d42df6`.
The complete Git trees are identical (`e282f702fe09c9871b08e1cf7a3c93d24e5b7e70`);
publication through the GitHub connector changes commit metadata only.
Source-v3: `b541866d534338fe367e8cdf965504267317f546f24b323f5de27bf5c7074896`.

All 44 USB tests pass, including 12 new integration tests and one new unit test.
The new cases cover protocol gating, nonzero control lengths, wrong identities
and operation kinds, stale/duplicate completions, reset/detach during pending
work, reattach with a new generation, malformed and oversized reports, independent
sessions, once-only releases and sequence exhaustion. Formatting, strict Clippy
and the scalar freestanding library build also pass. No test is ignored.

The freestanding build uses the existing component CI restrictions on red-zone,
SSE and AVX features. These checks were rerun in the integrating checkout on the
exact source above. A separate read-only AI review found no blocking issue on
that source; this is not an independent human security audit.

Exact commands, exit codes, timings, source hashes and log hashes are stored in
[the component receipt](../../../docs/verification/issue42-boot-session-20261114/component.json).
The complete short [USB test log](../../../docs/verification/issue42-boot-session-20261114/usb-tests.txt)
is retained. Other raw local logs are under `build/issue42-checks/` in the
integrating checkout.

The standard preflight/build/QEMU smoke gate on the integrated source is pending
at this first component checkpoint; no native USB result is claimed.

## Primary source and project policy

USB Implementers Forum. (2001). *Device class definition for human interface
devices (HID)* (Version 1.11, section 7.2.6 and appendices B and F.3).
<https://www.usb.org/sites/default/files/documents/hid1_11.pdf>.

The standard defines protocol selection, reset behavior and boot-report layout.
Correlation tokens, single-packet bounds and lifecycle transitions are project
implementation choices. No specification text or implementation code is copied.

## Reversal

Reverting the three session source/test changes removes this component without
changing the earlier descriptor, boot-report or HID request encoders. Existing
kernel capabilities and the H02 criterion remain unchanged.
