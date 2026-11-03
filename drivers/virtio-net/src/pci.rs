//! Registry-selected PCI discovery and the sole volatile/port-I/O boundary.
use super::{Discovery, Io, DEVICE_ID_NET, DRIVER_REGISTRATION};
use driver_core::{DriverMatch, PciAddress, PciDeviceDescriptor, PciHeaderError};
use pci_core::{MatchedPciFunction, Mechanism1, PciConfigAccess};
use virtio_core::{parse_capabilities_for, Bar, Error};

fn config_error(_: pci_core::ConfigError) -> Error {
    Error::Protocol
}

fn validate_registration(address: PciAddress, config: &[u8; 256]) -> Result<DriverMatch, Error> {
    let descriptor =
        PciDeviceDescriptor::from_type0_config(address, config).map_err(|error| match error {
            PciHeaderError::NotPresent => Error::Offline,
            PciHeaderError::InvalidAddress | PciHeaderError::UnsupportedHeaderType => {
                Error::Protocol
            }
        })?;
    let mut registry = driver_core::DriverRegistry::<1>::new();
    registry
        .register(DRIVER_REGISTRATION)
        .map_err(|_| Error::Protocol)?;
    match registry.best_match(&descriptor) {
        Ok(Some(matched)) => Ok(matched),
        Ok(None) => Err(Error::Offline),
        Err(_) => Err(Error::Protocol),
    }
}

fn bind_error(error: pci_core::BindError) -> Error {
    match error {
        pci_core::BindError::Config(error) => config_error(error),
        pci_core::BindError::Header(_)
        | pci_core::BindError::DescriptorChanged
        | pci_core::BindError::DriverMismatch => Error::Protocol,
    }
}

fn select_device() -> Result<PciAddress, Error> {
    let mut registry = driver_core::DriverRegistry::<1>::new();
    registry
        .register(DRIVER_REGISTRATION)
        .map_err(|_| Error::Protocol)?;
    let mut selected = None;
    let mut ambiguous = false;
    // SAFETY: Called synchronously during single-CPU kernel initialization
    // before device services or interrupts can access PCI configuration.
    let mut access = unsafe { Mechanism1::new() };
    pci_core::enumerate(&mut access, |descriptor| {
        match registry.best_match(&descriptor) {
            Ok(Some(_)) => {
                if selected.replace(descriptor.address).is_some() {
                    ambiguous = true;
                }
            }
            Err(_) => ambiguous = true,
            _ => {}
        }
    })
    .map_err(|_| Error::Protocol)?;
    if ambiguous {
        return Err(Error::Protocol);
    }
    selected.ok_or(Error::Offline)
}

pub fn discover() -> Result<Discovery, Error> {
    discover_at(select_device()?)
}

/// Probes one registry-selected type-0 VirtIO network function.
pub fn discover_at(address: PciAddress) -> Result<Discovery, Error> {
    // SAFETY: Called synchronously during single-CPU kernel initialization
    // before device services or interrupts can access PCI configuration.
    let mut pci = unsafe { Mechanism1::new() };
    if pci.read_u32(address, 0).map_err(config_error)? != (u32::from(DEVICE_ID_NET) << 16) | 0x1af4
    {
        return Err(Error::Offline);
    }
    let mut config = [0; 256];
    for offset in (0..256).step_by(4) {
        config[offset..offset + 4].copy_from_slice(
            &pci.read_u32(address, offset as u8)
                .map_err(config_error)?
                .to_le_bytes(),
        );
    }
    let descriptor =
        PciDeviceDescriptor::from_type0_config(address, &config).map_err(|_| Error::Protocol)?;
    let matched = validate_registration(address, &config)?;
    let mut pci = MatchedPciFunction::bind(pci, descriptor, matched).map_err(bind_error)?;
    let original = pci.read_u32(address, 4).map_err(config_error)? as u16;
    pci.write_u16(address, 4, (original & !7) | (1 << 10))
        .map_err(config_error)?;
    let mut bars = [Bar::NONE; 6];
    let mut index = 0;
    while index < 6 {
        let offset = 0x10 + index as u8 * 4;
        let low = pci.read_u32(address, offset).map_err(config_error)?;
        if low & 1 != 0 {
            index += 1;
            continue;
        }
        let wide = low & 6 == 4;
        if low & 6 != 0 && !wide || wide && index == 5 {
            return Err(Error::Unsupported);
        }
        let high = if wide {
            pci.read_u32(address, offset + 4).map_err(config_error)?
        } else {
            0
        };
        pci.write_u32(address, offset, u32::MAX)
            .map_err(config_error)?;
        if wide {
            pci.write_u32(address, offset + 4, u32::MAX)
                .map_err(config_error)?;
        }
        let mask_low = pci.read_u32(address, offset).map_err(config_error)?;
        let mask_high = if wide {
            pci.read_u32(address, offset + 4).map_err(config_error)?
        } else {
            u32::MAX
        };
        pci.write_u32(address, offset, low).map_err(config_error)?;
        if wide {
            pci.write_u32(address, offset + 4, high)
                .map_err(config_error)?;
        }
        let base = u64::from(low & !15) | u64::from(high) << 32;
        let mask = u64::from(mask_low & !15) | u64::from(mask_high) << 32;
        let size = (!mask).wrapping_add(1);
        if base != 0 && size != 0 {
            if !size.is_power_of_two() || size > 65536 || !base.is_multiple_of(4096) {
                return Err(Error::Range);
            }
            bars[index] = Bar {
                base,
                size,
                is_memory: true,
            };
        }
        index += if wide { 2 } else { 1 };
    }
    pci.write_u16(address, 4, (original & !5) | 2 | (1 << 10))
        .map_err(config_error)?;
    for offset in (0..256).step_by(4) {
        config[offset..offset + 4].copy_from_slice(
            &pci.read_u32(address, offset as u8)
                .map_err(config_error)?
                .to_le_bytes(),
        );
    }
    let matched_after_probe = validate_registration(address, &config)?;
    if matched_after_probe.registration.id != matched.registration.id {
        return Err(Error::Protocol);
    }
    let regions = parse_capabilities_for(&config, &bars, DEVICE_ID_NET)?;
    let mut at = config[0x34] as usize;
    // parse_capabilities_for has already validated the complete capability walk.
    while at != 0 {
        let value = pci.read_u32(address, at as u8).map_err(config_error)?;
        if config[at] == 5 {
            pci.write_u32(address, at as u8, value & !(1 << 16))
                .map_err(config_error)?;
        }
        if config[at] == 0x11 {
            pci.write_u32(address, at as u8, (value & !(1 << 31)) | (1 << 30))
                .map_err(config_error)?;
        }
        at = config[at + 1] as usize;
    }
    Ok(Discovery {
        address,
        config,
        bars,
        regions,
    })
}

pub(super) struct Hardware {
    address: PciAddress,
    pci: MatchedPciFunction<Mechanism1>,
}

pub(super) unsafe fn hardware(address: PciAddress, config: &[u8; 256]) -> Result<Hardware, Error> {
    let descriptor =
        PciDeviceDescriptor::from_type0_config(address, config).map_err(|_| Error::Protocol)?;
    let matched = validate_registration(address, config)?;
    // SAFETY: The caller is the synchronous, single-CPU kernel initialization
    // path; it serializes PCI config access and keeps interrupts disabled.
    let access = unsafe { Mechanism1::new() };
    let pci = MatchedPciFunction::bind(access, descriptor, matched).map_err(bind_error)?;
    Ok(Hardware { address, pci })
}

impl Io for Hardware {
    fn read8(&self, address: u64) -> u8 {
        // SAFETY: Engine derives only bounded byte addresses in permanent UC
        // BAR mappings or exclusive pinned DMA pages; no references alias DMA.
        unsafe { (address as *const u8).read_volatile() }
    }
    fn read16(&self, address: u64) -> u16 {
        // SAFETY: Engine uses naturally aligned registers/ring words in the
        // same permanent, exclusively owned mapped regions.
        unsafe { (address as *const u16).read_volatile() }
    }
    fn read32(&self, address: u64) -> u32 {
        // SAFETY: Aligned validated common-register or used-element dword.
        unsafe { (address as *const u32).read_volatile() }
    }
    fn write8(&mut self, address: u64, value: u8) {
        // SAFETY: Validated exclusive MMIO/DMA byte, with publication ordering
        // handled by Engine; a posted bounce buffer is never overwritten.
        unsafe {
            (address as *mut u8).write_volatile(value);
        }
    }
    fn write16(&mut self, address: u64, value: u16) {
        // SAFETY: Aligned validated common, notify or driver-owned avail word.
        unsafe {
            (address as *mut u16).write_volatile(value);
        }
    }
    fn write32(&mut self, address: u64, value: u32) {
        // SAFETY: Aligned common config register in caller's permanent UC map.
        unsafe {
            (address as *mut u32).write_volatile(value);
        }
    }
    fn write64(&mut self, address: u64, value: u64) {
        // SAFETY: Naturally aligned common queue-address register; fixed DMA
        // address derives only from the validated pinned pool.
        unsafe {
            (address as *mut u64).write_volatile(value);
        }
    }
    fn bus_master(&mut self, enabled: bool) -> Result<(), Error> {
        let current = self.pci.read_u32(self.address, 4).map_err(config_error)? as u16;
        self.pci
            .write_u16(
                self.address,
                4,
                if enabled {
                    current | 6 | (1 << 10)
                } else {
                    (current & !4) | (1 << 10)
                },
            )
            .map_err(config_error)
    }
    fn bus_master_enabled(&mut self) -> Result<bool, Error> {
        self.pci
            .read_u32(self.address, 4)
            .map(|command| command & 4 != 0)
            .map_err(config_error)
    }
}
