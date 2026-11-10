# Driver rules
Read the root instructions and hardware/security documentation. Target one precisely specified virtual or physical device.
Treat device data, descriptors and lengths as untrusted. Define explicit contracts for MMIO, DMA, memory ordering, interrupts, timeouts and reset paths. Use only the minimum mappings and rights. A userspace driver with unrestricted DMA is not sufficiently isolated.
Start with virtual devices. Do not attach raw host devices/disks to test guests. Identify which guarantees apply only to the emulator and which require IOMMU/hardware support.
Test absent or unresponsive devices, invalid ring lengths, and restart without leaking capabilities or buffers.
