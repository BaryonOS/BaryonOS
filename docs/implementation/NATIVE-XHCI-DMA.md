# Native xHCI DMA containment fixture

This opt-in H02 prerequisite exercises the project's kernel against QEMU's
xHCI and Intel VT-d devices. It is a bounded controller-execution experiment.
Actual USB enumeration and held-key unplug have a separate
[native HID transport fixture](NATIVE-USB-HID.md). Interrupt delivery, reusable
native grants, general hotplug and physical acceptance remain open.

## Platform and authority

The fixture accepts exactly one QEMU `1b36:000d` xHCI function directly on PCI
bus zero. The ACPI inventory must contain one segment-zero DMA-remapping unit,
with a bounded register aperture and explicit coverage of that function.
`INCLUDE_PCI_ALL` or a one-hop endpoint scope starting at bus zero supplies that
coverage. A bridge path is insufficient. QEMU 10 describes endpoints explicitly
rather than setting `INCLUDE_PCI_ALL` (QEMU Project, 2025a).

Reserved-memory mappings, unknown remapping structures/scopes, multiple units,
register overlap and unsupported address widths fail closed. The discovery
method remains inventory only: `dma_execution_allowed()` still returns `false`.
The ordinary `DeviceLease` bus-master model remains test-only.

The native adapter holds the real lease and its opaque DMA grants. Its private
domain constructor derives the requester from that lease, checks live pinned
ownership, installs actual translation tables, waits for hardware completion
and checks translation-enable readback. The xHCI bus-master write additionally
compares the controller requester with that domain. No userspace endpoint,
integer or firmware-table flag independently grants DMA authority.

## Memory, interrupts and lifetime

Initialization runs on the bootstrap processor with interrupts disabled, before
the scheduler and application services. Five kernel-owned pages contain the
root, context and three-level translation tables. All other requester contexts
remain absent. At most sixteen DMA pages are mapped from IOVA `0x100000`;
duplicate physical pages and out-of-profile requests are rejected. Command,
ERST and DCBAA pages have device-read permission; the event page has device-write
permission. DMA pinning prevents ordinary free, reallocation, user aliases and
executable aliases while the lease is live.

Register mappings are supervisor-only, uncached and non-executable. Table
updates use bounded scratch mappings, verified `CLFLUSH` support and fences;
the QEMU profile does not advertise coherent translation-table walks. Native
activation uses legacy root/context structures, completed global context-cache
and IOTLB invalidations, and translation-enable readback. Invalidation requests
require and use read/write draining. The corresponding register fields and
ordering requirements are specified by Intel (2022, §§ 3.4.2, 6.5, 6.7, 11.4).

PCI INTx, MSI, MSI-X, xHCI interrupt generation and remapper fault interrupts stay
masked; the fixture polls completion and fault records. This does not establish
interrupt-remapping or IRQ-delivery support.

A domain immutably borrows its lease, preventing lease mutation through safe
Rust while translation remains associated with it. Successful termination
halts xHCI, clears and verifies PCI bus mastering, removes the context and waits
for global invalidation. Tables and DMA pages remain retained until the VM
exits. Failed initialization or teardown never returns those pages to the
allocator, disables the IOMMU or retries uncertain hardware state.

## Acceptance and evidence limits

The dedicated runner `tools/run_xhci.py` defines separate cases:

- `noop`: xHCI fetches a No-Op command and writes the matching successful
  command-completion event through its bounded DMA domain.
- `fault`: the event destination is an unmapped, independently initialized
  guard page. The oracle requires a matching requester, address and write fault;
  every guard byte must remain unchanged after quiescence.
- `no-dmar`: the same controller remains unable to enable bus mastering when
  the required firmware inventory is absent.

Each case must pass its own native serial/exit oracle. Host parser tests and
cross-compilation do not substitute for these runs. The default smoke oracle
retains its existing unavailable-DMA markers. Executed results and exact source
identity belong in the checkpoint evidence, not in claims of general hardware
support. Missing DMAR, malformed inventory and failed hardware transitions are
failures, not a request to bypass isolation.

## References

Intel Corporation. (2022, June). *Intel virtualization technology for directed
I/O architecture specification* (Revision 4.0, Order No. D51397-015).
https://cdrdv2-public.intel.com/671081/vt-directed-io-spec.pdf

QEMU Project. (2025a). *hw/i386/acpi-build.c* (Version 10.0.0) [Source code].
https://github.com/qemu/qemu/blob/v10.0.0/hw/i386/acpi-build.c

QEMU Project. (2025b). *Intel VT-d device implementation and register definitions*
(Version 10.0.0) [Source code].
https://github.com/qemu/qemu/blob/v10.0.0/hw/i386/intel_iommu.c
https://github.com/qemu/qemu/blob/v10.0.0/hw/i386/intel_iommu_internal.h

The reviewed Intel PDF has SHA-256
`e55c50169100cb9a13b719349204c6da92e9d3e75cb24f9e4a23097516b11784`.
