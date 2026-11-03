#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
//! Bounded PCI configuration access and conventional segment-zero enumeration.

pub use driver_core::PciAddress;
use driver_core::{DriverMatch, PciDeviceDescriptor, PciHeaderError};

/// Maximum conventional PCI buses scanned on segment zero.
pub const BUS_COUNT: u16 = 256;
/// Maximum devices on one conventional PCI bus.
pub const DEVICES_PER_BUS: u8 = 32;
/// Maximum functions on a multifunction PCI device.
pub const FUNCTIONS_PER_DEVICE: u8 = 8;

/// PCI configuration-space access used by the enumerator and device adapters.
pub trait PciConfigAccess {
    /// Reads one aligned 32-bit configuration register.
    fn read_u32(&mut self, address: PciAddress, offset: u8) -> Result<u32, ConfigError>;
    /// Writes one aligned 32-bit configuration register.
    fn write_u32(&mut self, address: PciAddress, offset: u8, value: u32)
        -> Result<(), ConfigError>;
    /// Writes one 16-bit configuration register.
    fn write_u16(&mut self, address: PciAddress, offset: u8, value: u16)
        -> Result<(), ConfigError>;
}

/// Configuration-access failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigError {
    /// Register offset is unaligned or outside the conventional 256-byte space.
    InvalidOffset,
    /// The segment is not supported by this configuration mechanism.
    UnsupportedSegment,
    /// The caller attempted to access a different PCI function than the one matched.
    WrongFunction,
}

/// PCI enumeration failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanError {
    /// A configuration read failed; the partial scan must not be used as complete.
    Config(ConfigError),
    /// A present function reported a malformed or unsupported header snapshot.
    Header(PciHeaderError),
}

/// Summary of one complete bounded scan.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScanSummary {
    /// Present PCI functions whose type-0 descriptors were delivered.
    pub type0_functions: u32,
    /// Present functions with a non-type-0 header, safely skipped.
    pub unsupported_headers: u32,
}

/// Enumerates segment zero, all conventional bus/device addresses, and
/// multifunction functions. Only present type-0 endpoints are sent to `visit`;
/// bridges and other header types are counted and skipped. No device is
/// enabled, mapped, or started by enumeration.
///
/// The function scans all 256 possible buses because this interface does not
/// yet receive a firmware root-bridge topology. Access is read-only and bounded.
pub fn enumerate<A, F>(access: &mut A, mut visit: F) -> Result<ScanSummary, ScanError>
where
    A: PciConfigAccess,
    F: FnMut(PciDeviceDescriptor),
{
    let mut summary = ScanSummary::default();
    for bus in 0..BUS_COUNT {
        for device in 0..DEVICES_PER_BUS {
            let function0 = PciAddress {
                segment: 0,
                bus: bus as u8,
                device,
                function: 0,
            };
            let id = access.read_u32(function0, 0).map_err(ScanError::Config)?;
            if id as u16 == u16::MAX {
                continue;
            }

            let header_word = access
                .read_u32(function0, 0x0c)
                .map_err(ScanError::Config)?;
            let multifunction = ((header_word >> 16) as u8 & 0x80) != 0;
            scan_function(access, function0, &mut summary, &mut visit)?;

            if multifunction {
                for function in 1..FUNCTIONS_PER_DEVICE {
                    let address = PciAddress {
                        function,
                        ..function0
                    };
                    let id = access.read_u32(address, 0).map_err(ScanError::Config)?;
                    if id as u16 == u16::MAX {
                        continue;
                    }
                    scan_function(access, address, &mut summary, &mut visit)?;
                }
            }
        }
    }
    Ok(summary)
}

fn scan_function<A, F>(
    access: &mut A,
    address: PciAddress,
    summary: &mut ScanSummary,
    visit: &mut F,
) -> Result<(), ScanError>
where
    A: PciConfigAccess,
    F: FnMut(PciDeviceDescriptor),
{
    let mut config = [0; 256];
    for offset in (0..256).step_by(4) {
        let word = access
            .read_u32(address, offset as u8)
            .map_err(ScanError::Config)?;
        config[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
    }
    if config[0x0e] & 0x7f != 0 {
        summary.unsupported_headers = summary.unsupported_headers.saturating_add(1);
        return Ok(());
    }
    let descriptor =
        PciDeviceDescriptor::from_type0_config(address, &config).map_err(ScanError::Header)?;
    summary.type0_functions = summary.type0_functions.saturating_add(1);
    visit(descriptor);
    Ok(())
}

/// Legacy x86 PCI configuration mechanism 1 (CF8/CFC), segment zero only.
///
/// The object does not serialize access against other CF8/CFC users. Construct
/// it only during single-core kernel initialization with interrupts disabled,
/// while no other code can access PCI configuration ports.
pub struct Mechanism1 {
    _private: (),
}

impl Mechanism1 {
    /// Creates the privileged port-I/O adapter.
    ///
    /// # Safety
    /// Caller is executing at CPL0 on x86, interrupts are disabled, and no
    /// concurrent code accesses CF8/CFC until this adapter is no longer used.
    pub unsafe fn new() -> Self {
        Self { _private: () }
    }

    fn validate(address: PciAddress, offset: u8) -> Result<(), ConfigError> {
        if address.segment != 0 {
            return Err(ConfigError::UnsupportedSegment);
        }
        if address.device >= DEVICES_PER_BUS
            || address.function >= FUNCTIONS_PER_DEVICE
            || offset & 3 != 0
        {
            return Err(ConfigError::InvalidOffset);
        }
        Ok(())
    }

    fn select(address: PciAddress, offset: u8) -> u32 {
        0x8000_0000
            | (u32::from(address.bus) << 16)
            | (u32::from(address.device) << 11)
            | (u32::from(address.function) << 8)
            | (u32::from(offset) & 0xfc)
    }
}

impl PciConfigAccess for Mechanism1 {
    fn read_u32(&mut self, address: PciAddress, offset: u8) -> Result<u32, ConfigError> {
        Self::validate(address, offset)?;
        let value: u32;
        // SAFETY: The constructor's CPL0/serialization contract holds. The
        // address and aligned 8-bit register offset were checked above.
        unsafe {
            core::arch::asm!(
                "out dx, eax",
                in("dx") 0xcf8u16,
                in("eax") Self::select(address, offset),
                options(nomem, nostack)
            );
            core::arch::asm!(
                "in eax, dx",
                in("dx") 0xcfcu16,
                out("eax") value,
                options(nomem, nostack)
            );
        }
        Ok(value)
    }

    fn write_u32(
        &mut self,
        address: PciAddress,
        offset: u8,
        value: u32,
    ) -> Result<(), ConfigError> {
        Self::validate(address, offset)?;
        // SAFETY: The constructor's CPL0/serialization contract holds. The
        // address and aligned 8-bit register offset were checked above.
        unsafe {
            core::arch::asm!(
                "out dx, eax",
                in("dx") 0xcf8u16,
                in("eax") Self::select(address, offset),
                options(nomem, nostack)
            );
            core::arch::asm!(
                "out dx, eax",
                in("dx") 0xcfcu16,
                in("eax") value,
                options(nomem, nostack)
            );
        }
        Ok(())
    }

    fn write_u16(
        &mut self,
        address: PciAddress,
        offset: u8,
        value: u16,
    ) -> Result<(), ConfigError> {
        Self::validate(address, offset & !3)?;
        if offset & 1 != 0 {
            return Err(ConfigError::InvalidOffset);
        }
        let port = 0xcfcu16 + u16::from(offset & 2);
        // SAFETY: The constructor's CPL0/serialization contract holds. The
        // address is valid and the 16-bit register fits within this dword.
        unsafe {
            core::arch::asm!(
                "out dx, eax",
                in("dx") 0xcf8u16,
                in("eax") Self::select(address, offset),
                options(nomem, nostack)
            );
            core::arch::asm!(
                "out dx, ax",
                in("dx") port,
                in("ax") value,
                options(nomem, nostack)
            );
        }
        Ok(())
    }
}

/// PCI configuration access permanently bound to one matched function.
pub struct MatchedPciFunction<A> {
    access: A,
    descriptor: PciDeviceDescriptor,
    driver: DriverMatch,
}

/// One memory BAR probed through a registry-matched PCI function.
///
/// The value is not constructible outside this crate. It binds the physical
/// window to the PCI function, matched driver, and BAR index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PciBarGrant {
    address: PciAddress,
    driver_id: driver_core::DriverId,
    index: u8,
    base: u64,
    size: u64,
}

impl PciBarGrant {
    /// Returns the matched PCI function.
    pub const fn address(self) -> PciAddress {
        self.address
    }

    /// Returns the matched driver identifier.
    pub const fn driver_id(self) -> driver_core::DriverId {
        self.driver_id
    }

    /// Returns the BAR index.
    pub const fn index(self) -> u8 {
        self.index
    }

    /// Returns the assigned physical base address.
    pub const fn base(self) -> u64 {
        self.base
    }

    /// Returns the BAR's decoded byte length.
    pub const fn size(self) -> u64 {
        self.size
    }

    /// Checks the grant's device, driver, and BAR identity.
    pub const fn matches_device(
        self,
        address: PciAddress,
        driver_id: driver_core::DriverId,
        index: u8,
    ) -> bool {
        self.address.segment == address.segment
            && self.address.bus == address.bus
            && self.address.device == address.device
            && self.address.function == address.function
            && self.driver_id.get() == driver_id.get()
            && self.index == index
    }

    /// Resolves one complete 4-KiB page contained in this BAR.
    pub fn page_address(self, offset: u64) -> Result<u64, PciBarError> {
        const PAGE_SIZE: u64 = 4096;
        if !offset.is_multiple_of(PAGE_SIZE)
            || offset
                .checked_add(PAGE_SIZE)
                .is_none_or(|end| end > self.size)
        {
            return Err(PciBarError::Range);
        }
        self.base.checked_add(offset).ok_or(PciBarError::Range)
    }
}

/// Failure while probing a PCI memory BAR.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PciBarError {
    /// PCI configuration access failed.
    Config(ConfigError),
    /// BAR index or memory encoding is unsupported.
    Unsupported,
    /// BAR is unassigned or its decoded range is malformed.
    Range,
    /// A probed BAR register could not be restored.
    Restore,
}

impl<A: PciConfigAccess> MatchedPciFunction<A> {
    /// Binds configuration access after validating the current descriptor and
    /// the selected driver's match rule against that descriptor.
    pub fn bind(
        mut access: A,
        descriptor: PciDeviceDescriptor,
        driver: DriverMatch,
    ) -> Result<Self, BindError> {
        let mut config = [0; 256];
        for offset in (0..256).step_by(4) {
            let word = access
                .read_u32(descriptor.address, offset as u8)
                .map_err(BindError::Config)?;
            config[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
        }
        let current = PciDeviceDescriptor::from_type0_config(descriptor.address, &config)
            .map_err(BindError::Header)?;
        if current != descriptor {
            return Err(BindError::DescriptorChanged);
        }
        if !driver.registration.pci.matches(&current) {
            return Err(BindError::DriverMismatch);
        }
        Ok(Self {
            access,
            descriptor,
            driver,
        })
    }

    /// Returns the matched device identity.
    pub const fn descriptor(&self) -> PciDeviceDescriptor {
        self.descriptor
    }

    /// Returns the registry result bound to this function.
    pub const fn driver_match(&self) -> DriverMatch {
        self.driver
    }

    /// Probes conventional memory BARs for this matched function.
    ///
    /// I/O BARs are omitted. Only 32-bit and 64-bit non-prefetchable memory
    /// BARs are accepted by this initial device profile. PCI I/O decode,
    /// memory decode, and bus mastering are disabled before probing and remain
    /// disabled on success or failure; callers may enable only the exact
    /// features needed after their resources have been validated.
    pub fn probe_memory_bars(&mut self) -> Result<[Option<PciBarGrant>; 6], PciBarError> {
        let address = self.descriptor.address;
        let command = self.read_u32(address, 4).map_err(PciBarError::Config)? as u16;
        self.write_u16(address, 4, command & !7)
            .map_err(PciBarError::Config)?;

        let mut bars = [None; 6];
        let mut index = 0usize;
        while index < bars.len() {
            let (grant, wide) = self.probe_memory_bar(index as u8)?;
            bars[index] = grant;
            index += if wide { 2 } else { 1 };
        }
        Ok(bars)
    }

    fn probe_memory_bar(&mut self, index: u8) -> Result<(Option<PciBarGrant>, bool), PciBarError> {
        if index >= 6 {
            return Err(PciBarError::Unsupported);
        }
        let address = self.descriptor.address;
        let offset = 0x10 + index * 4;
        let low = self
            .read_u32(address, offset)
            .map_err(PciBarError::Config)?;
        if low & 1 != 0 {
            return Ok((None, false));
        }

        let encoding = low & 6;
        let wide = encoding == 4;
        if encoding != 0 && !wide || wide && index == 5 {
            return Err(PciBarError::Unsupported);
        }
        let high = if wide {
            self.read_u32(address, offset + 4)
                .map_err(PciBarError::Config)?
        } else {
            0
        };

        let probed = (|| {
            self.write_u32(address, offset, u32::MAX)
                .map_err(PciBarError::Config)?;
            if wide {
                self.write_u32(address, offset + 4, u32::MAX)
                    .map_err(PciBarError::Config)?;
            }
            let mask_low = self
                .read_u32(address, offset)
                .map_err(PciBarError::Config)?;
            let mask_high = if wide {
                self.read_u32(address, offset + 4)
                    .map_err(PciBarError::Config)?
            } else {
                u32::MAX
            };
            Ok::<_, PciBarError>((mask_low, mask_high))
        })();

        let restore_high = if wide {
            self.write_u32(address, offset + 4, high)
        } else {
            Ok(())
        };
        let restore_low = self.write_u32(address, offset, low);
        if restore_high.is_err() || restore_low.is_err() {
            return Err(PciBarError::Restore);
        }
        let (mask_low, mask_high) = probed?;
        let masked_low = mask_low & !15;
        if masked_low == 0 && (!wide || mask_high == 0) {
            return Ok((None, wide));
        }
        let base = u64::from(low & !15) | (u64::from(high) << 32);
        let mask = u64::from(masked_low) | (u64::from(mask_high) << 32);
        let size = (!mask).wrapping_add(1);
        if size == 0
            || !size.is_power_of_two()
            || base & (size - 1) != 0
            || base.checked_add(size).is_none()
        {
            return Err(PciBarError::Range);
        }
        if base == 0 {
            return Err(PciBarError::Range);
        }
        Ok((
            Some(PciBarGrant {
                address,
                driver_id: self.driver.registration.id,
                index,
                base,
                size,
            }),
            wide,
        ))
    }
}

impl<A: PciConfigAccess> PciConfigAccess for MatchedPciFunction<A> {
    fn read_u32(&mut self, address: PciAddress, offset: u8) -> Result<u32, ConfigError> {
        if address != self.descriptor.address {
            return Err(ConfigError::WrongFunction);
        }
        self.access.read_u32(address, offset)
    }

    fn write_u32(
        &mut self,
        address: PciAddress,
        offset: u8,
        value: u32,
    ) -> Result<(), ConfigError> {
        if address != self.descriptor.address {
            return Err(ConfigError::WrongFunction);
        }
        self.access.write_u32(address, offset, value)
    }

    fn write_u16(
        &mut self,
        address: PciAddress,
        offset: u8,
        value: u16,
    ) -> Result<(), ConfigError> {
        if address != self.descriptor.address {
            return Err(ConfigError::WrongFunction);
        }
        self.access.write_u16(address, offset, value)
    }
}

/// Failure to bind access to a registry-matched PCI function.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindError {
    /// Configuration access failed while validating the device.
    Config(ConfigError),
    /// The current header could not be decoded as a type-0 endpoint.
    Header(PciHeaderError),
    /// The device changed between enumeration and resource binding.
    DescriptorChanged,
    /// The selected registration does not match the current descriptor.
    DriverMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use driver_core::{
        DeviceClass, DriverId, DriverRegistration, DriverRegistry, DriverVersion, PciMatch,
    };

    #[derive(Clone, Copy)]
    struct MockFunction {
        address: PciAddress,
        config: [u8; 256],
        bar_masks: [u32; 6],
        bar_probing: [bool; 6],
    }

    #[derive(Clone)]
    struct MockConfig {
        functions: [Option<MockFunction>; 3],
        reads: u32,
    }

    impl MockConfig {
        fn new() -> Self {
            Self {
                functions: [None; 3],
                reads: 0,
            }
        }

        fn add(&mut self, address: PciAddress, vendor: u16, device: u16) -> &mut [u8; 256] {
            let slot = self
                .functions
                .iter_mut()
                .find(|entry| entry.is_none())
                .unwrap();
            let mut config = [0; 256];
            config[..4].copy_from_slice(
                &u32::from(vendor)
                    .wrapping_add(u32::from(device) << 16)
                    .to_le_bytes(),
            );
            *slot = Some(MockFunction {
                address,
                config,
                bar_masks: [0; 6],
                bar_probing: [false; 6],
            });
            &mut slot.as_mut().unwrap().config
        }

        fn set_bar(&mut self, address: PciAddress, index: usize, value: u32, mask: u32) {
            let function = self
                .functions
                .iter_mut()
                .flatten()
                .find(|function| function.address == address)
                .unwrap();
            let offset = 0x10 + index * 4;
            function.config[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            function.bar_masks[index] = mask;
        }

        fn word(&self, address: PciAddress, offset: u8) -> u32 {
            let Some(function) = self
                .functions
                .iter()
                .flatten()
                .find(|function| function.address == address)
            else {
                return u32::MAX;
            };
            u32::from_le_bytes([
                function.config[offset as usize],
                function.config[offset as usize + 1],
                function.config[offset as usize + 2],
                function.config[offset as usize + 3],
            ])
        }
    }

    impl PciConfigAccess for MockConfig {
        fn read_u32(&mut self, address: PciAddress, offset: u8) -> Result<u32, ConfigError> {
            if address.device >= DEVICES_PER_BUS || address.function >= FUNCTIONS_PER_DEVICE {
                return Err(ConfigError::InvalidOffset);
            }
            if offset & 3 != 0 {
                return Err(ConfigError::InvalidOffset);
            }
            self.reads += 1;
            if (0x10..=0x24).contains(&offset) {
                let index = usize::from((offset - 0x10) / 4);
                if let Some(function) = self
                    .functions
                    .iter()
                    .flatten()
                    .find(|function| function.address == address)
                {
                    if function.bar_probing[index] {
                        return Ok(function.bar_masks[index]);
                    }
                }
            }
            Ok(self.word(address, offset))
        }

        fn write_u32(
            &mut self,
            address: PciAddress,
            offset: u8,
            value: u32,
        ) -> Result<(), ConfigError> {
            let function = self
                .functions
                .iter_mut()
                .flatten()
                .find(|function| function.address == address)
                .ok_or(ConfigError::WrongFunction)?;
            if (0x10..=0x24).contains(&offset) {
                let index = usize::from((offset - 0x10) / 4);
                if value == u32::MAX {
                    function.bar_probing[index] = true;
                } else {
                    function.bar_probing[index] = false;
                    let offset = offset as usize;
                    function.config[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
                }
            } else {
                let offset = offset as usize;
                function.config[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            }
            Ok(())
        }

        fn write_u16(
            &mut self,
            address: PciAddress,
            offset: u8,
            value: u16,
        ) -> Result<(), ConfigError> {
            let function = self
                .functions
                .iter_mut()
                .flatten()
                .find(|function| function.address == address)
                .ok_or(ConfigError::WrongFunction)?;
            let offset = offset as usize;
            function.config[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
            Ok(())
        }
    }

    fn address(device: u8, function: u8) -> PciAddress {
        PciAddress {
            segment: 0,
            bus: 0,
            device,
            function,
        }
    }

    fn matched_block_function(
        mock: MockConfig,
        address: PciAddress,
    ) -> MatchedPciFunction<MockConfig> {
        let mut snapshot = [0; 256];
        for offset in (0..256).step_by(4) {
            snapshot[offset..offset + 4]
                .copy_from_slice(&mock.word(address, offset as u8).to_le_bytes());
        }
        let descriptor = PciDeviceDescriptor::from_type0_config(address, &snapshot).unwrap();
        let registration = DriverRegistration {
            id: DriverId::new(1),
            name: "test-block",
            version: DriverVersion::new(1, 0, 0),
            class: DeviceClass::BlockDevice,
            pci: PciMatch {
                vendor_id: Some(0x1af4),
                device_id: Some(0x1042),
                class: None,
                subclass: None,
                programming_interface: None,
                revision: None,
                subsystem_vendor_id: None,
                subsystem_device_id: None,
            },
        };
        let mut registry = DriverRegistry::<1>::new();
        registry.register(registration).unwrap();
        let matched = registry.best_match(&descriptor).unwrap().unwrap();
        MatchedPciFunction::bind(mock, descriptor, matched).unwrap()
    }

    #[test]
    fn matched_bar_probe_returns_bounded_device_and_driver_grant() {
        let device = address(5, 0);
        let mut mock = MockConfig::new();
        let config = mock.add(device, 0x1af4, 0x1042);
        config[0x0b] = 1;
        config[4..6].copy_from_slice(&7u16.to_le_bytes());
        mock.set_bar(device, 0, 0x8000_0000, 0xffff_f000);
        let mut matched = matched_block_function(mock, device);

        let bars = matched.probe_memory_bars().unwrap();
        let bar = bars[0].unwrap();

        assert_eq!(bar.address(), device);
        assert_eq!(bar.driver_id(), DriverId::new(1));
        assert_eq!(bar.index(), 0);
        assert_eq!(bar.base(), 0x8000_0000);
        assert_eq!(bar.size(), 4096);
        assert!(bar.matches_device(device, DriverId::new(1), 0));
        assert!(!bar.matches_device(address(6, 0), DriverId::new(1), 0));
        assert!(!bar.matches_device(device, DriverId::new(2), 0));
        assert!(!bar.matches_device(device, DriverId::new(1), 1));
        assert_eq!(bar.page_address(0), Ok(0x8000_0000));
        assert_eq!(bar.page_address(4096), Err(PciBarError::Range));
        assert_eq!(bar.page_address(1), Err(PciBarError::Range));
        assert_eq!(matched.read_u32(device, 0x10), Ok(0x8000_0000));
        assert_eq!(matched.read_u32(device, 4).unwrap() as u16 & 7, 0);
    }

    #[test]
    fn matched_bar_probe_handles_64_bit_bars_and_skips_unimplemented_bars() {
        let device = address(5, 0);
        let mut mock = MockConfig::new();
        mock.add(device, 0x1af4, 0x1042)[0x0b] = 1;
        mock.set_bar(device, 0, 0x8000_0004, 0xffff_f004);
        mock.set_bar(device, 1, 1, u32::MAX);
        let mut matched = matched_block_function(mock, device);

        let bars = matched.probe_memory_bars().unwrap();

        assert_eq!(bars[0].unwrap().base(), 0x1_8000_0000);
        assert_eq!(bars[0].unwrap().size(), 4096);
        assert_eq!(bars[1], None);
        assert_eq!(bars[2], None);
    }

    #[test]
    fn matched_bar_probe_rejects_malformed_ranges_and_restores_bar_registers() {
        let device = address(5, 0);
        let mut mock = MockConfig::new();
        mock.add(device, 0x1af4, 0x1042)[0x0b] = 1;
        mock.set_bar(device, 0, 0x8000_0000, 0xffff_d000);
        let mut matched = matched_block_function(mock, device);

        assert_eq!(matched.probe_memory_bars(), Err(PciBarError::Range));
        assert_eq!(matched.read_u32(device, 0x10), Ok(0x8000_0000));
        assert_eq!(matched.read_u32(device, 4).unwrap() as u16 & 7, 0);
    }

    #[test]
    fn enumerates_multiple_devices_and_multifunction_endpoints() {
        let mut mock = MockConfig::new();
        let first = mock.add(address(4, 0), 0x1af4, 0x1042);
        first[0x0e] = 0x80;
        first[0x0b] = 1;
        let second = mock.add(address(4, 1), 0x1af4, 0x1041);
        second[0x0a] = 0;
        second[0x0b] = 2;
        mock.add(address(9, 0), 0x1234, 0xabcd);

        let mut seen = [None; 3];
        let mut len = 0;
        let summary = enumerate(&mut mock, |descriptor| {
            seen[len] = Some(descriptor);
            len += 1;
        })
        .unwrap();

        assert_eq!(len, 3);
        assert_eq!(summary.type0_functions, 3);
        assert_eq!(seen[0].unwrap().address, address(4, 0));
        assert_eq!(seen[1].unwrap().address, address(4, 1));
        assert_eq!(seen[2].unwrap().address, address(9, 0));
        assert!(mock.reads < 9_000);
    }

    #[test]
    fn does_not_probe_nonzero_functions_without_multifunction_header() {
        let mut mock = MockConfig::new();
        mock.add(address(2, 0), 0x1af4, 0x1042);
        mock.add(address(2, 1), 0x1af4, 0x1041);
        let mut seen = 0;
        let summary = enumerate(&mut mock, |_| seen += 1).unwrap();
        assert_eq!(seen, 1);
        assert_eq!(summary.type0_functions, 1);
    }

    #[test]
    fn safely_skips_non_type0_headers() {
        let mut mock = MockConfig::new();
        mock.add(address(1, 0), 0x1234, 0x5678)[0x0e] = 1;
        let mut seen = 0;
        let summary = enumerate(&mut mock, |_| seen += 1).unwrap();
        assert_eq!(seen, 0);
        assert_eq!(summary.unsupported_headers, 1);
    }

    #[test]
    fn matched_configuration_access_is_bound_to_the_selected_function() {
        let device_address = address(3, 0);
        let mut mock = MockConfig::new();
        let config = mock.add(device_address, 0x1af4, 0x1042);
        config[0x0b] = 1;
        let mut snapshot = [0; 256];
        for offset in (0..256).step_by(4) {
            snapshot[offset..offset + 4]
                .copy_from_slice(&mock.word(device_address, offset as u8).to_le_bytes());
        }
        let descriptor = PciDeviceDescriptor::from_type0_config(device_address, &snapshot).unwrap();
        let registration = DriverRegistration {
            id: DriverId::new(1),
            name: "test-block",
            version: DriverVersion::new(1, 0, 0),
            class: DeviceClass::BlockDevice,
            pci: PciMatch {
                vendor_id: Some(0x1af4),
                device_id: Some(0x1042),
                class: None,
                subclass: None,
                programming_interface: None,
                revision: None,
                subsystem_vendor_id: None,
                subsystem_device_id: None,
            },
        };
        let mut registry = DriverRegistry::<1>::new();
        registry.register(registration).unwrap();
        let matched = registry.best_match(&descriptor).unwrap().unwrap();
        let mut bound = MatchedPciFunction::bind(mock, descriptor, matched).unwrap();

        assert_eq!(bound.descriptor(), descriptor);
        assert_eq!(bound.driver_match(), matched);
        assert_eq!(bound.read_u32(device_address, 0), Ok(0x1042_1af4));
        assert_eq!(
            bound.read_u32(address(4, 0), 0),
            Err(ConfigError::WrongFunction)
        );
        assert_eq!(
            bound.write_u32(address(4, 0), 4, u32::MAX),
            Err(ConfigError::WrongFunction)
        );
        assert_eq!(
            bound.write_u16(address(4, 0), 4, u16::MAX),
            Err(ConfigError::WrongFunction)
        );
    }

    #[test]
    fn matched_configuration_binding_rejects_stale_or_unmatched_identity() {
        let device_address = address(3, 0);
        let mut mock = MockConfig::new();
        let config = mock.add(device_address, 0x1af4, 0x1042);
        config[0x0b] = 1;
        let mut snapshot = [0; 256];
        for offset in (0..256).step_by(4) {
            snapshot[offset..offset + 4]
                .copy_from_slice(&mock.word(device_address, offset as u8).to_le_bytes());
        }
        let descriptor = PciDeviceDescriptor::from_type0_config(device_address, &snapshot).unwrap();
        let registration = DriverRegistration {
            id: DriverId::new(1),
            name: "test-block",
            version: DriverVersion::new(1, 0, 0),
            class: DeviceClass::BlockDevice,
            pci: PciMatch {
                vendor_id: Some(0x1af4),
                device_id: Some(0x1043),
                class: None,
                subclass: None,
                programming_interface: None,
                revision: None,
                subsystem_vendor_id: None,
                subsystem_device_id: None,
            },
        };
        let matched = DriverMatch {
            registration,
            specificity: driver_core::PciSpecificity {
                subsystem: 0,
                identity: 2,
                class: 0,
                revision: false,
            },
        };
        assert_eq!(
            MatchedPciFunction::bind(mock.clone(), descriptor, matched).err(),
            Some(BindError::DriverMismatch)
        );
        assert_eq!(
            MatchedPciFunction::bind(
                mock,
                PciDeviceDescriptor {
                    device_id: 0x1043,
                    ..descriptor
                },
                DriverMatch {
                    registration: DriverRegistration {
                        pci: PciMatch {
                            device_id: Some(0x1043),
                            ..registration.pci
                        },
                        ..registration
                    },
                    ..matched
                },
            )
            .err(),
            Some(BindError::DescriptorChanged)
        );
    }
}
