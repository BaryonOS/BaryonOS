#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
//! Bounded PCI configuration access and conventional segment-zero enumeration.

use driver_core::{PciAddress, PciDeviceDescriptor, PciHeaderError};

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
pub struct Mechanism1;

impl Mechanism1 {
    /// Creates the privileged port-I/O adapter.
    ///
    /// # Safety
    /// Caller is executing at CPL0 on x86, interrupts are disabled, and no
    /// concurrent code accesses CF8/CFC until this adapter is no longer used.
    pub unsafe fn new() -> Self {
        Self
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct MockFunction {
        address: PciAddress,
        config: [u8; 256],
    }

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
            *slot = Some(MockFunction { address, config });
            &mut slot.as_mut().unwrap().config
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
            Ok(self.word(address, offset))
        }

        fn write_u32(
            &mut self,
            _address: PciAddress,
            _offset: u8,
            _value: u32,
        ) -> Result<(), ConfigError> {
            Ok(())
        }

        fn write_u16(
            &mut self,
            _address: PciAddress,
            _offset: u8,
            _value: u16,
        ) -> Result<(), ConfigError> {
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
}
