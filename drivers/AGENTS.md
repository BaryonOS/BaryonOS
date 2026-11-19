# Driver rules
Read the root instructions and hardware/security documentation. Target one precisely specified virtual or physical device.
Treat device data, descriptors and lengths as untrusted. Define explicit contracts for MMIO, DMA, memory ordering, interrupts, timeouts and reset paths. Use only the minimum mappings and rights. A userspace driver with unrestricted DMA is not sufficiently isolated.
Start with virtual devices. Do not attach raw host devices/disks to test guests. Identify which guarantees apply only to the emulator and which require IOMMU/hardware support.
Test absent or unresponsive devices, invalid ring lengths, and restart without leaking capabilities or buffers.
Commands: workspace members `virtio-blk-driver`, `virtio-rng-driver` and `virtio-net-driver` use `cargo +1.100.0 test --locked -p <crate>` and clippy `-D warnings`; the other drivers use `--manifest-path drivers/<driver>/Cargo.toml` with test, clippy, fmt check and `build --locked --release --target x86_64-unknown-none`. Native gates: `native-rtc.yml`, `native-power.yml`, `native-display-input.yml` and `native-system.yml` (TPM).
