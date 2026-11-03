# xHCI PCI capability probe

This standalone `no_std` crate matches PCI class `0c:03:30`, validates the
matched controller's non-prefetchable memory BAR0, and reads only the xHCI
capability registers. It checks the capability, operational, doorbell,
runtime-register, and extended-capability offsets against the granted BAR.
BARs smaller than one page are rejected because the current kernel PCI mapper
grants complete 4-KiB pages.

The probe does not take ownership from firmware, write controller registers,
enable bus mastering, allocate or expose DMA, set up interrupts, create rings,
enumerate USB devices, or start class drivers. It is not a working USB
host-controller driver. Ring setup must wait for the kernel-owned,
device-specific DMA and interrupt lifecycle; this crate defines no competing
resource model.

The crate is deliberately outside the root workspace and adds no third-party
dependency. Hardware support is not demonstrated by the host models or
freestanding compile check.

## Primary source

- Intel, *eXtensible Host Controller Interface for Universal Serial Bus (xHCI)
  Requirements Specification*, revision 1.2c, October 2025, §§5.2 and 5.3:
  <https://cdrdv2-public.intel.com/868295/xHCI__Rev1.2c.pdf>.

The PCI class tuple and register layout follow the specification. No
specification text or implementation code is copied.
