#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
//! Read-only NVMe PCI discovery and controller capability inspection.
//!
//! This crate does not enable bus mastering, allocate DMA, or submit commands.

use core::ptr::NonNull;
use driver_core::{
    DeviceClass, DriverId, DriverMatch, DriverRegistration, DriverRegistry, DriverVersion,
    PciAddress, PciMatch, RegistryError,
};
use pci_core::{MatchedPciFunction, PciBarGrant, PciConfigAccess, ScanError};

/// PCI registration for the standard NVM Express controller programming interface.
pub const DRIVER_REGISTRATION: DriverRegistration = DriverRegistration {
    id: DriverId::new(0x4e4f_0003),
    name: "nieuw-os-nvme",
    version: DriverVersion::new(0, 1, 0),
    class: DeviceClass::StorageController,
    pci: PciMatch {
        vendor_id: None,
        device_id: None,
        class: Some(0x01),
        subclass: Some(0x08),
        programming_interface: Some(0x02),
        revision: None,
        subsystem_vendor_id: None,
        subsystem_device_id: None,
    },
};

/// Failure while discovering an NVMe controller or inspecting its registers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// PCI configuration scanning or matching failed.
    Pci,
    /// No registered NVMe controller was found.
    NotFound,
    /// More than one NVMe controller matched this single-device probe.
    Ambiguous,
    /// The matched controller did not expose a supported BAR0.
    InvalidBar,
    /// A register offset, range, alignment, or encoding was invalid.
    InvalidRegister,
    /// The controller does not support the minimum NVM/4-KiB profile.
    Unsupported,
}

/// One uniquely matched NVMe function and its device-bound BAR0 grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Discovery {
    address: PciAddress,
    driver_match: DriverMatch,
    bar0: PciBarGrant,
}

impl Discovery {
    /// Returns the matched PCI address.
    pub const fn address(self) -> PciAddress {
        self.address
    }

    /// Returns the selected driver registration and specificity.
    pub const fn driver_match(self) -> DriverMatch {
        self.driver_match
    }

    /// Returns the BAR0 grant bound to this function and driver.
    pub const fn bar0(self) -> PciBarGrant {
        self.bar0
    }

    /// Creates a read-only MMIO view over the exact BAR0 grant.
    ///
    /// # Safety
    /// `virtual_base` must point to a stable, supervisor-only uncached mapping
    /// of the complete BAR0 range in this discovery. The kernel must have
    /// enabled PCI memory decoding for this function. The mapping must not
    /// expose memory outside the grant or permit an untrusted process access.
    pub unsafe fn map_bar0(self, virtual_base: NonNull<u8>) -> Result<MmioBar, Error> {
        let length = validate_mapping_range(virtual_base.as_ptr() as usize, self.bar0.size())?;
        if !self
            .bar0
            .matches_device(self.address, DRIVER_REGISTRATION.id, 0)
            || self.bar0.base() == 0
            || !self.bar0.base().is_multiple_of(4096)
            || !(virtual_base.as_ptr() as usize).is_multiple_of(4096)
        {
            return Err(Error::InvalidBar);
        }
        Ok(MmioBar {
            base: virtual_base,
            length: length as u64,
        })
    }
}

/// Enumerates segment zero, matches the standard NVMe PCI class, and grants
/// only BAR0 of the unique selected function.
///
/// The generic scanner is read-only. BAR probing temporarily writes the
/// matched function's BAR registers, restores them, and leaves PCI memory
/// decoding and bus mastering disabled.
pub fn discover<A: PciConfigAccess>(access: A) -> Result<Discovery, Error> {
    let mut access = access;
    let mut registry = DriverRegistry::<1>::new();
    registry
        .register(DRIVER_REGISTRATION)
        .map_err(|_| Error::Pci)?;

    let mut selected = None;
    let mut ambiguous = false;
    pci_core::enumerate(&mut access, |descriptor| {
        match registry.best_match(&descriptor) {
            Ok(Some(driver_match)) => {
                if selected.replace((descriptor, driver_match)).is_some() {
                    ambiguous = true;
                }
            }
            Err(RegistryError::AmbiguousMatch) => ambiguous = true,
            Err(_) | Ok(None) => {}
        }
    })
    .map_err(map_scan_error)?;

    if ambiguous {
        return Err(Error::Ambiguous);
    }
    let (descriptor, driver_match) = selected.ok_or(Error::NotFound)?;
    let mut matched =
        MatchedPciFunction::bind(access, descriptor, driver_match).map_err(|_| Error::Pci)?;
    let bars = matched.probe_memory_bars().map_err(|_| Error::InvalidBar)?;
    let bar0 = bars[0].ok_or(Error::InvalidBar)?;
    let raw_bar0 = matched
        .read_u32(descriptor.address, 0x10)
        .map_err(|_| Error::Pci)?;

    // NVMe PCI BAR0 is a non-prefetchable 64-bit memory BAR; BAR1 carries
    // its high address bits and therefore is not a separate grant.
    if raw_bar0 & 1 != 0
        || (raw_bar0 >> 1) & 3 != 2
        || raw_bar0 & 8 != 0
        || bars[1].is_some()
        || bar0.base() == 0
        || !bar0.base().is_multiple_of(4096)
        || bar0.size() < MINIMUM_BAR_BYTES
        || bar0.base().checked_add(bar0.size()).is_none()
        || !bar0.matches_device(descriptor.address, DRIVER_REGISTRATION.id, 0)
    {
        return Err(Error::InvalidBar);
    }

    Ok(Discovery {
        address: descriptor.address,
        driver_match,
        bar0,
    })
}

fn map_scan_error(_error: ScanError) -> Error {
    Error::Pci
}

/// Minimum BAR size needed to cover the controller registers and admin doorbells.
pub const MINIMUM_BAR_BYTES: u64 = 0x1008;
const CAP_OFFSET: u64 = 0x00;
const VS_OFFSET: u64 = 0x08;

/// Read-only register accessor, also usable by host-side protocol tests.
pub trait RegisterAccess {
    /// Reads an aligned 32-bit register.
    fn read_u32(&mut self, offset: u64) -> Result<u32, Error>;
    /// Reads an aligned 64-bit register.
    fn read_u64(&mut self, offset: u64) -> Result<u64, Error>;
}

/// Supervisor mapping of the matched NVMe BAR0.
pub struct MmioBar {
    base: NonNull<u8>,
    length: u64,
}

impl MmioBar {
    fn contains(&self, offset: u64, size: u64) -> Result<usize, Error> {
        register_offset(self.length, offset, size)
    }
}

fn register_offset(length: u64, offset: u64, size: u64) -> Result<usize, Error> {
    let end = offset.checked_add(size).ok_or(Error::InvalidRegister)?;
    if end > length {
        return Err(Error::InvalidRegister);
    }
    usize::try_from(offset).map_err(|_| Error::InvalidRegister)
}

fn validate_mapping_range(base: usize, length: u64) -> Result<usize, Error> {
    let length = usize::try_from(length).map_err(|_| Error::InvalidBar)?;
    if length < MINIMUM_BAR_BYTES as usize
        || length > isize::MAX as usize
        || base.checked_add(length).is_none()
    {
        return Err(Error::InvalidBar);
    }
    Ok(length)
}

impl RegisterAccess for MmioBar {
    fn read_u32(&mut self, offset: u64) -> Result<u32, Error> {
        if !offset.is_multiple_of(4) {
            return Err(Error::InvalidRegister);
        }
        let offset = self.contains(offset, 4)?;
        // SAFETY: Construction requires a stable MMIO mapping for the grant;
        // the range and native 32-bit alignment were checked above.
        Ok(unsafe { core::ptr::read_volatile(self.base.as_ptr().add(offset).cast::<u32>()) })
    }

    fn read_u64(&mut self, offset: u64) -> Result<u64, Error> {
        if !offset.is_multiple_of(8) {
            return Err(Error::InvalidRegister);
        }
        let offset = self.contains(offset, 8)?;
        // SAFETY: Construction requires a stable MMIO mapping for the grant;
        // the range and native 64-bit alignment were checked above.
        Ok(unsafe { core::ptr::read_volatile(self.base.as_ptr().add(offset).cast::<u64>()) })
    }
}

/// Parsed controller capabilities required to plan an NVMe 1.x/2.x admin queue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControllerCapabilities {
    /// Maximum submission/completion queue entries, decoded from the zero-based MQES.
    pub max_queue_entries: u32,
    /// Advertised controller timeout in units of 500 milliseconds.
    pub timeout_500ms: u8,
    /// Doorbell spacing in bytes.
    pub doorbell_stride_bytes: u32,
    /// Controller advertises the NVM command set.
    pub supports_nvm: bool,
    /// Minimum memory page size exponent, where zero means 4 KiB.
    pub minimum_page_exponent: u8,
    /// Maximum memory page size exponent, where zero means 4 KiB.
    pub maximum_page_exponent: u8,
}

impl ControllerCapabilities {
    /// Parses CAP and rejects controllers outside the initial NVM/4-KiB profile.
    pub fn parse(raw: u64) -> Result<Self, Error> {
        let mqes = (raw & 0xffff) as u16;
        let timeout_500ms = ((raw >> 24) & 0xff) as u8;
        let dstrd = ((raw >> 32) & 0x0f) as u32;
        let supports_nvm = raw & (1 << 37) != 0;
        let minimum_page_exponent = ((raw >> 48) & 0x0f) as u8;
        let maximum_page_exponent = ((raw >> 52) & 0x0f) as u8;

        if mqes == 0
            || !supports_nvm
            || minimum_page_exponent > 0
            || minimum_page_exponent > maximum_page_exponent
        {
            return Err(Error::Unsupported);
        }
        let doorbell_stride_bytes = 4u32.checked_shl(dstrd).ok_or(Error::Unsupported)?;
        Ok(Self {
            max_queue_entries: u32::from(mqes) + 1,
            timeout_500ms,
            doorbell_stride_bytes,
            supports_nvm,
            minimum_page_exponent,
            maximum_page_exponent,
        })
    }
}

/// NVMe revision in the controller's VS register.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Version {
    /// Major revision.
    pub major: u16,
    /// Minor revision.
    pub minor: u8,
    /// Tertiary revision.
    pub tertiary: u8,
}

impl Version {
    /// Parses VS and accepts only major revisions 1 and 2.
    pub fn parse(raw: u32) -> Result<Self, Error> {
        let version = Self {
            major: (raw >> 16) as u16,
            minor: (raw >> 8) as u8,
            tertiary: raw as u8,
        };
        if !(1..=2).contains(&version.major) {
            return Err(Error::Unsupported);
        }
        Ok(version)
    }
}

/// Read-only summary of the controller's standard registers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControllerInfo {
    /// Parsed CAP values.
    pub capabilities: ControllerCapabilities,
    /// Parsed VS version.
    pub version: Version,
}

impl ControllerInfo {
    /// Reads and validates CAP and VS without changing controller state.
    pub fn inspect<R: RegisterAccess>(registers: &mut R) -> Result<Self, Error> {
        let capabilities = ControllerCapabilities::parse(registers.read_u64(CAP_OFFSET)?)?;
        let version = Version::parse(registers.read_u32(VS_OFFSET)?)?;
        Ok(Self {
            capabilities,
            version,
        })
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
extern crate std;
