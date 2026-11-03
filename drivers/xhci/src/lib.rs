#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
//! Read-only xHCI PCI discovery and capability-register inspection.
//!
//! This crate does not take controller ownership, enable bus mastering, or
//! allocate DMA.

use core::ptr::NonNull;
use driver_core::{
    DeviceClass, DriverId, DriverMatch, DriverRegistration, DriverRegistry, DriverVersion,
    PciAddress, PciMatch, RegistryError,
};
use pci_core::{MatchedPciFunction, PciBarGrant, PciConfigAccess, ScanError};

/// Registration for a standard PCI xHCI host controller.
pub const DRIVER_REGISTRATION: DriverRegistration = DriverRegistration {
    id: DriverId::new(0x4e4f_0004),
    name: "nieuw-os-xhci-probe",
    version: DriverVersion::new(0, 1, 0),
    class: DeviceClass::UsbHostController,
    pci: PciMatch {
        vendor_id: None,
        device_id: None,
        class: Some(0x0c),
        subclass: Some(0x03),
        programming_interface: Some(0x30),
        revision: None,
        subsystem_vendor_id: None,
        subsystem_device_id: None,
    },
};

/// Failure while discovering an xHCI controller or validating its register layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// PCI configuration scanning or matching failed.
    Pci,
    /// No registered xHCI controller was found.
    NotFound,
    /// More than one xHCI controller matched this single-controller probe.
    Ambiguous,
    /// The matched controller did not expose a supported BAR0.
    InvalidBar,
    /// A register offset, range, alignment, or encoding was invalid.
    InvalidRegister,
    /// The controller revision is outside the supported xHCI 1.x profile.
    Unsupported,
}

/// One uniquely matched xHCI function and its device-bound BAR0 grant.
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

/// Enumerates segment zero, matches the xHCI PCI class, and grants the unique
/// selected function's BAR0.
///
/// The generic scanner is read-only. BAR probing temporarily writes the
/// matched function's BAR registers, restores them, and leaves memory decoding
/// and bus mastering disabled.
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
    let bar_type = (raw_bar0 >> 1) & 3;

    if raw_bar0 & 1 != 0
        || raw_bar0 & 8 != 0
        || !matches!(bar_type, 0 | 2)
        || (bar_type == 2 && bars[1].is_some())
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

/// Minimum BAR size addressable by the kernel's page-granular PCI mapper.
pub const MINIMUM_BAR_BYTES: u64 = 0x1000;
const CAPABILITY_OFFSET: u64 = 0x00;
const STRUCTURAL_PARAMS_1_OFFSET: u64 = 0x04;
const CAPABILITY_PARAMS_1_OFFSET: u64 = 0x10;
const DOORBELL_OFFSET_REGISTER: u64 = 0x14;
const RUNTIME_OFFSET_REGISTER: u64 = 0x18;
const MINIMUM_CAPABILITY_LENGTH: u64 = 0x20;
const MINIMUM_REGISTER_OFFSET: u64 = 0x20;
const MINIMUM_OPERATIONAL_LENGTH: u64 = 0x3c;
const PORT_REGISTER_OFFSET: u64 = 0x400;
const PORT_REGISTER_LENGTH: u64 = 0x10;
const RUNTIME_INTERRUPTER_OFFSET: u64 = 0x20;
const RUNTIME_INTERRUPTER_LENGTH: u64 = 0x20;

/// Read-only register accessor, also usable by host-side protocol tests.
pub trait RegisterAccess {
    /// Reads an aligned 32-bit register.
    fn read_u32(&mut self, offset: u64) -> Result<u32, Error>;
}

/// Supervisor mapping of the matched xHCI BAR0.
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
    if end > length || !offset.is_multiple_of(4) {
        return Err(Error::InvalidRegister);
    }
    usize::try_from(offset).map_err(|_| Error::InvalidRegister)
}

fn validate_mapping_range(base: usize, length: u64) -> Result<usize, Error> {
    let length = usize::try_from(length).map_err(|_| Error::InvalidBar)?;
    if length < MINIMUM_BAR_BYTES as usize
        || !length.is_multiple_of(MINIMUM_BAR_BYTES as usize)
        || length > isize::MAX as usize
        || base.checked_add(length).is_none()
    {
        return Err(Error::InvalidBar);
    }
    Ok(length)
}

impl RegisterAccess for MmioBar {
    fn read_u32(&mut self, offset: u64) -> Result<u32, Error> {
        let offset = self.contains(offset, 4)?;
        // SAFETY: Construction requires a stable MMIO mapping for the grant;
        // the range and native 32-bit alignment were checked above.
        Ok(unsafe { core::ptr::read_volatile(self.base.as_ptr().add(offset).cast::<u32>()) })
    }
}

/// Validated, read-only xHCI register layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControllerCapabilities {
    /// Raw HCIVERSION value; the supported major revision is 1.
    pub version: u16,
    /// Byte offset from BAR0 to the operational register block.
    pub operational_offset: u64,
    /// Maximum device slots reported by HCSPARAMS1.
    pub max_slots: u8,
    /// Maximum interrupters reported by HCSPARAMS1.
    pub max_interrupters: u16,
    /// Maximum root-hub ports reported by HCSPARAMS1.
    pub max_ports: u8,
    /// Byte offset from BAR0 to the doorbell array.
    pub doorbell_offset: u64,
    /// Byte offset from BAR0 to the runtime register block.
    pub runtime_offset: u64,
    /// First extended capability offset, if advertised.
    pub extended_capability_offset: Option<u64>,
}

impl ControllerCapabilities {
    /// Parses standard capability registers and verifies every advertised
    /// register range fits inside the granted BAR without overlapping another
    /// advertised register area.
    pub fn parse(
        capability: u32,
        structural_params_1: u32,
        capability_params_1: u32,
        doorbell_offset: u32,
        runtime_offset: u32,
        bar_size: u64,
    ) -> Result<Self, Error> {
        let capability_length = u64::from((capability & 0xff) as u8);
        let version = (capability >> 16) as u16;
        let max_slots = structural_params_1 as u8;
        let max_interrupters = ((structural_params_1 >> 8) & 0x7ff) as u16;
        let max_ports = (structural_params_1 >> 24) as u8;
        let doorbell_register = u64::from(doorbell_offset);
        let runtime_register = u64::from(runtime_offset);
        let doorbell_offset = doorbell_register & !3;
        let runtime_offset = runtime_register & !0x1f;
        let extended_dword_offset = (capability_params_1 >> 16) as u16;
        let extended_capability_offset =
            (extended_dword_offset != 0).then_some(u64::from(extended_dword_offset) * 4);

        if capability_length < MINIMUM_CAPABILITY_LENGTH || !capability_length.is_multiple_of(4) {
            return Err(Error::InvalidRegister);
        }
        if version >> 8 != 1 {
            return Err(Error::Unsupported);
        }
        if max_slots == 0 || max_interrupters == 0 || max_ports == 0 {
            return Err(Error::InvalidRegister);
        }
        if doorbell_register & 3 != 0 || runtime_register & 0x1f != 0 {
            return Err(Error::InvalidRegister);
        }

        let operational_offset = capability_length;
        let operational_bytes = PORT_REGISTER_OFFSET
            .checked_add(
                u64::from(max_ports)
                    .checked_mul(PORT_REGISTER_LENGTH)
                    .ok_or(Error::InvalidRegister)?,
            )
            .ok_or(Error::InvalidRegister)?
            .max(MINIMUM_OPERATIONAL_LENGTH);
        let doorbell_bytes = (u64::from(max_slots) + 1)
            .checked_mul(4)
            .ok_or(Error::InvalidRegister)?;
        let runtime_bytes = RUNTIME_INTERRUPTER_OFFSET
            .checked_add(
                u64::from(max_interrupters)
                    .checked_mul(RUNTIME_INTERRUPTER_LENGTH)
                    .ok_or(Error::InvalidRegister)?,
            )
            .ok_or(Error::InvalidRegister)?;

        validate_region(bar_size, operational_offset, operational_bytes)?;
        validate_region(bar_size, doorbell_offset, doorbell_bytes)?;
        validate_region(bar_size, runtime_offset, runtime_bytes)?;

        if ranges_overlap(0, capability_length, doorbell_offset, doorbell_bytes)?
            || ranges_overlap(0, capability_length, runtime_offset, runtime_bytes)?
            || ranges_overlap(
                operational_offset,
                operational_bytes,
                doorbell_offset,
                doorbell_bytes,
            )?
            || ranges_overlap(
                operational_offset,
                operational_bytes,
                runtime_offset,
                runtime_bytes,
            )?
            || ranges_overlap(
                doorbell_offset,
                doorbell_bytes,
                runtime_offset,
                runtime_bytes,
            )?
        {
            return Err(Error::InvalidRegister);
        }

        if let Some(offset) = extended_capability_offset {
            if offset < MINIMUM_CAPABILITY_LENGTH {
                return Err(Error::InvalidRegister);
            }
            validate_region(bar_size, offset, 4)?;
            if ranges_overlap(offset, 4, 0, capability_length)?
                || ranges_overlap(offset, 4, operational_offset, operational_bytes)?
                || ranges_overlap(offset, 4, doorbell_offset, doorbell_bytes)?
                || ranges_overlap(offset, 4, runtime_offset, runtime_bytes)?
            {
                return Err(Error::InvalidRegister);
            }
        }

        Ok(Self {
            version,
            operational_offset,
            max_slots,
            max_interrupters,
            max_ports,
            doorbell_offset,
            runtime_offset,
            extended_capability_offset,
        })
    }

    /// Reads only the xHCI capability registers required to validate the
    /// controller layout. No controller register is written.
    pub fn inspect<R: RegisterAccess>(registers: &mut R, bar_size: u64) -> Result<Self, Error> {
        Self::parse(
            registers.read_u32(CAPABILITY_OFFSET)?,
            registers.read_u32(STRUCTURAL_PARAMS_1_OFFSET)?,
            registers.read_u32(CAPABILITY_PARAMS_1_OFFSET)?,
            registers.read_u32(DOORBELL_OFFSET_REGISTER)?,
            registers.read_u32(RUNTIME_OFFSET_REGISTER)?,
            bar_size,
        )
    }
}

fn validate_region(bar_size: u64, offset: u64, length: u64) -> Result<(), Error> {
    let end = offset.checked_add(length).ok_or(Error::InvalidRegister)?;
    if end > bar_size || offset < MINIMUM_REGISTER_OFFSET {
        return Err(Error::InvalidRegister);
    }
    Ok(())
}

fn ranges_overlap(first: u64, first_len: u64, second: u64, second_len: u64) -> Result<bool, Error> {
    let first_end = first.checked_add(first_len).ok_or(Error::InvalidRegister)?;
    let second_end = second
        .checked_add(second_len)
        .ok_or(Error::InvalidRegister)?;
    Ok(first < second_end && second < first_end)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
extern crate std;
