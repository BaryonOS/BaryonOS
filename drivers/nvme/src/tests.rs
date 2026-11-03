use super::*;
use std::{vec, vec::Vec};

fn cap(mqes: u16, dstrd: u8, mpsmin: u8, mpsmax: u8, supports_nvm: bool) -> u64 {
    u64::from(mqes)
        | (2u64 << 24)
        | (u64::from(dstrd) << 32)
        | (u64::from(supports_nvm) << 37)
        | (u64::from(mpsmin) << 48)
        | (u64::from(mpsmax) << 52)
}

#[test]
fn parses_supported_controller_capabilities() {
    assert_eq!(
        ControllerCapabilities::parse(cap(63, 2, 0, 0, true)),
        Ok(ControllerCapabilities {
            max_queue_entries: 64,
            timeout_500ms: 2,
            doorbell_stride_bytes: 16,
            supports_nvm: true,
            minimum_page_exponent: 0,
            maximum_page_exponent: 0,
        })
    );
}

#[test]
fn rejects_empty_or_incompatible_controller_capabilities() {
    assert_eq!(
        ControllerCapabilities::parse(cap(0, 0, 0, 0, true)),
        Err(Error::Unsupported)
    );
    assert_eq!(
        ControllerCapabilities::parse(cap(7, 0, 0, 0, false)),
        Err(Error::Unsupported)
    );
    assert_eq!(
        ControllerCapabilities::parse(cap(7, 0, 1, 1, true)),
        Err(Error::Unsupported)
    );
    assert_eq!(
        ControllerCapabilities::parse(cap(7, 0, 2, 1, true)),
        Err(Error::Unsupported)
    );
}

#[test]
fn parses_only_supported_nvme_major_versions() {
    assert_eq!(
        Version::parse(0x0001_0402),
        Ok(Version {
            major: 1,
            minor: 4,
            tertiary: 2
        })
    );
    assert_eq!(Version::parse(0x0002_0200).unwrap().major, 2);
    assert_eq!(Version::parse(0x0003_0000), Err(Error::Unsupported));
    assert_eq!(Version::parse(0), Err(Error::Unsupported));
}

#[test]
fn bounds_register_ranges_without_overflow() {
    assert_eq!(register_offset(MINIMUM_BAR_BYTES, 0x1004, 4), Ok(0x1004));
    assert_eq!(
        register_offset(MINIMUM_BAR_BYTES, 0x1008, 4),
        Err(Error::InvalidRegister)
    );
    assert_eq!(
        register_offset(MINIMUM_BAR_BYTES, u64::MAX, 8),
        Err(Error::InvalidRegister)
    );
}

#[test]
fn rejects_mmio_mapping_ranges_that_cannot_be_addressed() {
    assert_eq!(
        validate_mapping_range(0x1000, MINIMUM_BAR_BYTES),
        Ok(MINIMUM_BAR_BYTES as usize)
    );
    assert_eq!(
        validate_mapping_range(usize::MAX - 0x7ff, 0x1000),
        Err(Error::InvalidBar)
    );
    assert_eq!(
        validate_mapping_range(0x1000, u64::MAX),
        Err(Error::InvalidBar)
    );
}

struct FakeRegisters {
    reads: Vec<(u64, u8)>,
    cap: u64,
    version: u32,
}

impl RegisterAccess for FakeRegisters {
    fn read_u32(&mut self, offset: u64) -> Result<u32, Error> {
        self.reads.push((offset, 4));
        match offset {
            VS_OFFSET => Ok(self.version),
            _ => Err(Error::InvalidRegister),
        }
    }

    fn read_u64(&mut self, offset: u64) -> Result<u64, Error> {
        self.reads.push((offset, 8));
        match offset {
            CAP_OFFSET => Ok(self.cap),
            _ => Err(Error::InvalidRegister),
        }
    }
}

#[test]
fn controller_inspection_reads_only_cap_and_version() {
    let mut registers = FakeRegisters {
        reads: Vec::new(),
        cap: cap(31, 1, 0, 0, true),
        version: 0x0001_0300,
    };
    let info = ControllerInfo::inspect(&mut registers).unwrap();
    assert_eq!(info.capabilities.max_queue_entries, 32);
    assert_eq!(info.version.major, 1);
    assert_eq!(registers.reads, [(CAP_OFFSET, 8), (VS_OFFSET, 4)]);
}

fn nvme_config(
    address: PciAddress,
    device_id: u16,
    class: u8,
    subclass: u8,
    prog_if: u8,
) -> MockFunction {
    let mut config = [0u8; 256];
    config[0..2].copy_from_slice(&0x1234u16.to_le_bytes());
    config[2..4].copy_from_slice(&device_id.to_le_bytes());
    config[8] = 1;
    config[9] = prog_if;
    config[10] = subclass;
    config[11] = class;
    config[0x0e] = 0;
    config[0x10..0x14].copy_from_slice(&0xfebf_0004u32.to_le_bytes());
    config[0x14..0x18].copy_from_slice(&0u32.to_le_bytes());
    MockFunction {
        address,
        config,
        probe_low: false,
        probe_high: false,
    }
}

struct MockFunction {
    address: PciAddress,
    config: [u8; 256],
    probe_low: bool,
    probe_high: bool,
}

#[derive(Default)]
struct MockPci {
    functions: Vec<MockFunction>,
}

impl MockPci {
    fn function_mut(&mut self, address: PciAddress) -> Option<&mut MockFunction> {
        self.functions.iter_mut().find(|f| f.address == address)
    }
}

impl PciConfigAccess for MockPci {
    fn read_u32(&mut self, address: PciAddress, offset: u8) -> Result<u32, pci_core::ConfigError> {
        let Some(function) = self.function_mut(address) else {
            return Ok(if offset == 0 { u32::MAX } else { 0 });
        };
        let offset = usize::from(offset);
        if offset == 0x10 && function.probe_low {
            return Ok(0xffff_c004);
        }
        if offset == 0x14 && function.probe_high {
            return Ok(u32::MAX);
        }
        Ok(u32::from_le_bytes(
            function.config[offset..offset + 4].try_into().unwrap(),
        ))
    }

    fn write_u32(
        &mut self,
        address: PciAddress,
        offset: u8,
        value: u32,
    ) -> Result<(), pci_core::ConfigError> {
        let Some(function) = self.function_mut(address) else {
            return Err(pci_core::ConfigError::WrongFunction);
        };
        match (offset, value) {
            (0x10, u32::MAX) => function.probe_low = true,
            (0x14, u32::MAX) => function.probe_high = true,
            (0x10, _) => {
                function.probe_low = false;
                function.config[0x10..0x14].copy_from_slice(&value.to_le_bytes());
            }
            (0x14, _) => {
                function.probe_high = false;
                function.config[0x14..0x18].copy_from_slice(&value.to_le_bytes());
            }
            _ => {
                let offset = usize::from(offset);
                function.config[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            }
        }
        Ok(())
    }

    fn write_u16(
        &mut self,
        address: PciAddress,
        offset: u8,
        value: u16,
    ) -> Result<(), pci_core::ConfigError> {
        let Some(function) = self.function_mut(address) else {
            return Err(pci_core::ConfigError::WrongFunction);
        };
        let offset = usize::from(offset);
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

#[test]
fn discovers_one_nvme_function_and_returns_its_bound_bar_grant() {
    let expected = address(3, 0);
    let pci = MockPci {
        functions: vec![
            nvme_config(expected, 0x5678, 0x01, 0x08, 0x02),
            nvme_config(address(4, 0), 0x9999, 0x02, 0x00, 0x00),
        ],
    };
    let found = discover(pci).unwrap();
    assert_eq!(found.address(), expected);
    assert_eq!(found.bar0().address(), expected);
    assert_eq!(found.bar0().driver_id(), DRIVER_REGISTRATION.id);
    assert_eq!(found.bar0().index(), 0);
    assert_eq!(found.bar0().base(), 0xfebf_0000);
    assert_eq!(found.bar0().size(), 0x4000);
}

#[test]
fn rejects_unsupported_absent_and_ambiguous_pci_functions() {
    assert_eq!(discover(MockPci::default()), Err(Error::NotFound));

    let unsupported = MockPci {
        functions: vec![nvme_config(address(3, 0), 0x5678, 0x01, 0x08, 0x01)],
    };
    assert_eq!(discover(unsupported), Err(Error::NotFound));

    let ambiguous = MockPci {
        functions: vec![
            nvme_config(address(3, 0), 0x5678, 0x01, 0x08, 0x02),
            nvme_config(address(4, 0), 0x5679, 0x01, 0x08, 0x02),
        ],
    };
    assert_eq!(discover(ambiguous), Err(Error::Ambiguous));
}

#[test]
fn rejects_invalid_nvme_bar_shapes() {
    let mut prefetchable = nvme_config(address(3, 0), 0x5678, 0x01, 0x08, 0x02);
    prefetchable.config[0x10..0x14].copy_from_slice(&0xfebf_000cu32.to_le_bytes());
    assert_eq!(
        discover(MockPci {
            functions: vec![prefetchable]
        }),
        Err(Error::InvalidBar)
    );

    let mut io_bar = nvme_config(address(3, 0), 0x5678, 0x01, 0x08, 0x02);
    io_bar.config[0x10..0x14].copy_from_slice(&0x0000_c001u32.to_le_bytes());
    assert_eq!(
        discover(MockPci {
            functions: vec![io_bar]
        }),
        Err(Error::InvalidBar)
    );
}
