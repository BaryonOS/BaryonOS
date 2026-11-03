use crate::serial;
use driver_core::{DriverRegistry, PciDeviceDescriptor, RegistryError};
use pci_core::{Mechanism1, ScanError};

const REGISTRY_CAPACITY: usize =
    1 + cfg!(feature = "native-system-net") as usize + cfg!(feature = "native-keyring") as usize;

pub(super) fn report_unsupported_devices() {
    let mut registry = DriverRegistry::<REGISTRY_CAPACITY>::new();
    register(&mut registry, virtio_blk_driver::DRIVER_REGISTRATION);
    #[cfg(feature = "native-system-net")]
    register(&mut registry, virtio_net_driver::DRIVER_REGISTRATION);
    #[cfg(feature = "native-keyring")]
    register(&mut registry, virtio_rng_driver::DRIVER_REGISTRATION);

    // SAFETY: M3 bootstrap runs on the single CPU with interrupts disabled;
    // this scan precedes all device setup and no other CF8/CFC owner exists.
    let mut access = unsafe { Mechanism1::new() };
    match pci_core::enumerate(&mut access, |descriptor| {
        match registry.best_match(&descriptor) {
            Ok(Some(_)) => {}
            Ok(None) => write_device(b"UNSUPPORTED:PCI", descriptor),
            Err(RegistryError::AmbiguousMatch) => write_device(b"PCI:AMBIGUOUS", descriptor),
            Err(
                RegistryError::Full
                | RegistryError::DuplicateDriverId
                | RegistryError::InvalidPciMatch,
            ) => serial::fail(b"PCI_REGISTRY_INVALID"),
        }
    }) {
        Ok(summary) => {
            if summary.unsupported_headers != 0 {
                serial::write(b"PCI:SKIPPED:NON_TYPE0:COUNT=");
                write_hex_u32(summary.unsupported_headers);
                serial::write(b"\n");
            }
        }
        Err(ScanError::Config(_)) => serial::write(b"PCI:SCAN:INCOMPLETE:CONFIG\n"),
        Err(ScanError::Header(_)) => serial::write(b"PCI:SCAN:INCOMPLETE:HEADER\n"),
    }
}

fn register<const N: usize>(
    registry: &mut DriverRegistry<N>,
    registration: driver_core::DriverRegistration,
) {
    if registry.register(registration).is_err() {
        serial::fail(b"PCI_REGISTRY_INVALID");
    }
}

fn write_device(status: &[u8], descriptor: PciDeviceDescriptor) {
    serial::write(status);
    serial::write(b":SEG=");
    write_hex_u16(descriptor.address.segment);
    serial::write(b":BUS=");
    write_hex_u8(descriptor.address.bus);
    serial::write(b":BDF=");
    write_hex_u8(descriptor.address.bus);
    serial::write(b":");
    write_hex_u8(descriptor.address.device);
    serial::write(b".");
    serial::write(&[hex_digit(descriptor.address.function)]);
    serial::write(b":VID=");
    write_hex_u16(descriptor.vendor_id);
    serial::write(b":DID=");
    write_hex_u16(descriptor.device_id);
    serial::write(b":CLASS=");
    write_hex_u8(descriptor.class);
    serial::write(b":SUBCLASS=");
    write_hex_u8(descriptor.subclass);
    serial::write(b":PROGIF=");
    write_hex_u8(descriptor.programming_interface);
    serial::write(b":REV=");
    write_hex_u8(descriptor.revision);
    serial::write(b":SUBSYS=");
    match (
        descriptor.subsystem_vendor_id,
        descriptor.subsystem_device_id,
    ) {
        (Some(vendor), Some(device)) => {
            write_hex_u16(vendor);
            serial::write(b":");
            write_hex_u16(device);
        }
        _ => serial::write(b"NONE"),
    }
    serial::write(b"\n");
}

fn write_hex_u8(value: u8) {
    serial::write(&[hex_digit(value >> 4), hex_digit(value)]);
}

fn write_hex_u16(value: u16) {
    serial::write(&[
        hex_digit((value >> 12) as u8),
        hex_digit((value >> 8) as u8),
        hex_digit((value >> 4) as u8),
        hex_digit(value as u8),
    ]);
}

fn write_hex_u32(value: u32) {
    serial::write(&[
        hex_digit((value >> 28) as u8),
        hex_digit((value >> 24) as u8),
        hex_digit((value >> 20) as u8),
        hex_digit((value >> 16) as u8),
        hex_digit((value >> 12) as u8),
        hex_digit((value >> 8) as u8),
        hex_digit((value >> 4) as u8),
        hex_digit(value as u8),
    ]);
}

fn hex_digit(value: u8) -> u8 {
    b"0123456789ABCDEF"[(value & 0x0f) as usize]
}
