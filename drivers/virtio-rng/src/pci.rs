use super::{Discovery, DriverFactory, DEVICE_ID_ENTROPY, DRIVER_FACTORY, DRIVER_REGISTRATION};
use driver_core::{DriverFactoryMatch, PciAddress, PciDeviceDescriptor, PciHeaderError};
use pci_core::{MatchedPciFunction, Mechanism1, PciConfigAccess};
use virtio_core::{parse_capabilities_for, Bar, Error};

fn config_error(_: pci_core::ConfigError) -> Error {
    Error::Protocol
}

fn bar_error(error: pci_core::PciBarError) -> Error {
    match error {
        pci_core::PciBarError::Config(_) | pci_core::PciBarError::Restore => Error::Protocol,
        pci_core::PciBarError::Unsupported => Error::Unsupported,
        pci_core::PciBarError::Range => Error::Range,
    }
}

fn header_error(error: PciHeaderError) -> Error {
    match error {
        PciHeaderError::NotPresent => Error::Offline,
        PciHeaderError::InvalidAddress | PciHeaderError::UnsupportedHeaderType => Error::Protocol,
    }
}

fn validate_registration(
    address: PciAddress,
    config: &[u8; 256],
) -> Result<DriverFactoryMatch<DriverFactory>, Error> {
    let descriptor =
        PciDeviceDescriptor::from_type0_config(address, config).map_err(header_error)?;
    let mut registry = driver_core::DriverRegistry::<1, DriverFactory>::with_factories();
    registry
        .register_with_factory(DRIVER_REGISTRATION, DRIVER_FACTORY)
        .map_err(|_| Error::Protocol)?;
    match registry.best_match_with_factory(&descriptor) {
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
    let mut registry = driver_core::DriverRegistry::<1, DriverFactory>::with_factories();
    registry
        .register_with_factory(DRIVER_REGISTRATION, DRIVER_FACTORY)
        .map_err(|_| Error::Protocol)?;
    let mut selected = None;
    let mut ambiguous = false;
    // SAFETY: Initialization is serialized on the current single CPU, before
    // device services or interrupts can access PCI configuration.
    let mut access = unsafe { Mechanism1::new() };
    pci_core::enumerate(&mut access, |descriptor| {
        match registry.best_match_with_factory(&descriptor) {
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

pub(super) fn discover() -> Result<Discovery, Error> {
    discover_at(select_device()?)
}

pub(super) fn discover_at(address: PciAddress) -> Result<Discovery, Error> {
    // SAFETY: Initialization is serialized on the current single CPU, before
    // device services or interrupts can access PCI configuration.
    let mut access = unsafe { Mechanism1::new() };
    let mut config = [0; 256];
    for offset in (0..256).step_by(4) {
        config[offset..offset + 4].copy_from_slice(
            &access
                .read_u32(address, offset as u8)
                .map_err(config_error)?
                .to_le_bytes(),
        );
    }
    let descriptor =
        PciDeviceDescriptor::from_type0_config(address, &config).map_err(header_error)?;
    let matched = validate_registration(address, &config)?;
    let mut pci =
        MatchedPciFunction::bind(access, descriptor, matched.driver_match).map_err(bind_error)?;
    let original = pci.read_u32(address, 4).map_err(config_error)? as u16;
    let bar_grants = pci.probe_memory_bars().map_err(bar_error)?;
    let mut bars = [Bar::NONE; 6];
    for (index, grant) in bar_grants.iter().enumerate() {
        if let Some(grant) = grant {
            if grant.size() > 65536 || !grant.base().is_multiple_of(4096) {
                return Err(Error::Range);
            }
            bars[index] = Bar {
                base: grant.base(),
                size: grant.size(),
                is_memory: true,
            };
        }
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
    if matched_after_probe.driver_match.registration.id != matched.driver_match.registration.id {
        return Err(Error::Protocol);
    }
    let regions = parse_capabilities_for(&config, &bars, DEVICE_ID_ENTROPY)?;
    let mut at = config[0x34] as usize;
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
        bar_grants,
        regions,
        driver_factory: matched.factory,
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
    // SAFETY: The caller is the serialized single-CPU kernel initialization
    // path; PCI config access remains exclusive and interrupts stay disabled.
    let access = unsafe { Mechanism1::new() };
    let pci =
        MatchedPciFunction::bind(access, descriptor, matched.driver_match).map_err(bind_error)?;
    Ok(Hardware { address, pci })
}

impl Hardware {
    pub(super) fn bus_master(&mut self, enabled: bool) -> Result<(), Error> {
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

    pub(super) fn bus_master_enabled(&mut self) -> Result<bool, Error> {
        self.pci
            .read_u32(self.address, 4)
            .map(|command| command & 4 != 0)
            .map_err(config_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn address() -> PciAddress {
        PciAddress {
            segment: 0,
            bus: 0,
            device: 6,
            function: 0,
        }
    }

    fn config(vendor: u16, device: u16) -> [u8; 256] {
        let mut config = [0; 256];
        config[0..2].copy_from_slice(&vendor.to_le_bytes());
        config[2..4].copy_from_slice(&device.to_le_bytes());
        config
    }

    #[test]
    fn registry_matches_rng_and_keeps_its_initializer() {
        let matched = validate_registration(address(), &config(0x1af4, DEVICE_ID_ENTROPY)).unwrap();
        assert_eq!(matched.driver_match.registration.id, DRIVER_REGISTRATION.id);
        assert!(core::ptr::fn_addr_eq(matched.factory, DRIVER_FACTORY));
    }

    #[test]
    fn registry_skips_other_devices_and_rejects_non_type0_headers() {
        assert!(matches!(
            validate_registration(address(), &config(0x1af4, 0x1042)),
            Err(Error::Offline)
        ));
        let mut bridge = config(0x1af4, DEVICE_ID_ENTROPY);
        bridge[0x0e] = 1;
        assert!(matches!(
            validate_registration(address(), &bridge),
            Err(Error::Protocol)
        ));
    }
}
