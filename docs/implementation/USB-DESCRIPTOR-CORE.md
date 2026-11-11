# USB descriptor framing core

## Task contract

Goal of this first USB step: a small dependency-free `no_std` parser that
checks the boundaries of consecutive USB descriptors. The bytes will come
later from a bounded control transfer; this core performs no device
I/O itself.

The USB 2.0 descriptor structure begins with `bLength` and `bDescriptorType`.
`bLength` includes the descriptor header itself. The parser accepts only lengths
from two bytes upward that fall entirely within the supplied slice. An
incomplete header, too small a length or a truncated descriptor yields one explicit
error; after that the iterator stops and does not try to
resynchronise in untrusted bytes. Unknown descriptor types remain available unchanged
for a later policy layer.

## Boundaries and invariants

- The caller validates the control transfer status and bounds the received bytes
  before this core gets the slice. The core knows no device pointer, DMA buffer,
  controller status or capability.
- The core does not allocate, keeps no global state and uses no
  `unsafe`. Descriptors borrow their raw bytes from the input slice.
- Only generic framing is checked. This is not USB enumeration,
  configuration/interface validation, a HID driver, a USB host controller or an xHCI
  driver. Descriptor order, type fields and device policy remain for later
  separate layers.
- A framing error ends the iteration; partially validated subsequent
  descriptors are not published.
- Removing this standalone component changes no kernel, IPC,
  capability or existing driver interface.

## Acceptance and evidence boundary

Host tests cover empty input, valid and unknown types, descriptor boundaries,
lengths 0/1/2/255, missing header bytes, truncated descriptors and terminal
error handling. In addition, check formatting, Clippy and an
`x86_64-unknown-none` library build with the pinned Rust 1.90.0 toolchain.

This building block yields no H02 point: there is not yet a USB host controller that
enumerates a device or delivers events to a process. xHCI rings,
MMIO/DMA policy, timeouts/reset, hotplug, rights, QEMU devices and physical
reference hardware remain separate follow-up steps.

## Primary source

- USB Implementers Forum, *USB 2.0 Specification*, descriptor description
  §9.5. The official document page was updated on 3 June 2025 and links the
  specification with associated errata/ECNs:
  <https://www.usb.org/document-library/usb-20-specification>.
  The page links the official archive
  <https://www.usb.org/sites/default/files/usb_20_20250603.zip>.
  No specification text or implementation code has been copied.

## Executed check — 30 October 2026

On source commit `f8e7490b1dd8fdeb3cd8e8eb8578535bc49c5cc6` the host tests
(7/7), Clippy, formatting and the release library build for
`x86_64-unknown-none` passed. Exact commands, logs and the hash of the
built library are in
[`verification/usb-descriptor-core-f8e7490/`](../verification/usb-descriptor-core-f8e7490/).
No native QEMU USB run or physical hardware test has been performed.
