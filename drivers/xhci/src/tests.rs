use super::*;
use std::{cell::RefCell, rc::Rc, vec, vec::Vec};

#[test]
fn parses_and_bounds_a_supported_capability_layout() {
    let structural = 8 | (2 << 8) | (4 << 24);
    assert_eq!(
        ControllerCapabilities::parse(
            0x0100_0040,
            structural,
            0x0040_0000,
            0x1000,
            0x2000,
            0x4000,
        )
        .unwrap(),
        ControllerCapabilities {
            version: 0x0100,
            operational_offset: 0x40,
            max_slots: 8,
            max_interrupters: 2,
            max_ports: 4,
            doorbell_offset: 0x1000,
            runtime_offset: 0x2000,
            extended_capability_offset: Some(0x100),
        }
    );
}

#[test]
fn rejects_unsupported_revision_and_malformed_capability_length() {
    let structural = 1 | (1 << 8) | (1 << 24);
    assert_eq!(
        ControllerCapabilities::parse(0x0200_0040, structural, 0, 0x1000, 0x2000, 0x4000),
        Err(Error::Unsupported)
    );
    for cap_length in [0u8, 0x1c, 0x21] {
        assert_eq!(
            ControllerCapabilities::parse(
                u32::from(cap_length) | (0x0100 << 16),
                structural,
                0,
                0x1000,
                0x2000,
                0x4000,
            ),
            Err(Error::InvalidRegister)
        );
    }
}

#[test]
fn rejects_doorbell_runtime_and_extended_capability_ranges_outside_bar() {
    let structural = 8 | (2 << 8) | (4 << 24);
    let capability_word = 0x40 | (0x0100 << 16);
    assert_eq!(
        ControllerCapabilities::parse(capability_word, structural, 0, 0x3ff0, 0x2000, 0x4000),
        Err(Error::InvalidRegister)
    );
    assert_eq!(
        ControllerCapabilities::parse(capability_word, structural, 0, 0x1000, 0x3fe0, 0x4000),
        Err(Error::InvalidRegister)
    );
    assert_eq!(
        ControllerCapabilities::parse(
            capability_word,
            structural,
            0xffff_0000,
            0x1000,
            0x2000,
            0x4000,
        ),
        Err(Error::InvalidRegister)
    );
    assert_eq!(
        ControllerCapabilities::parse(capability_word, structural, 0, 0x1000, 0x2001, 0x4000,),
        Err(Error::InvalidRegister)
    );
}

#[test]
fn rejects_zero_structural_limits_and_port_register_overlap() {
    let capability_word = 0x40 | (0x0100 << 16);
    for structural in [(2 << 8) | (4 << 24), 8 | (4 << 24), 8 | (2 << 8)] {
        assert_eq!(
            ControllerCapabilities::parse(capability_word, structural, 0, 0x1000, 0x2000, 0x4000,),
            Err(Error::InvalidRegister)
        );
    }

    let structural = 8 | (2 << 8) | (4 << 24);
    assert_eq!(
        ControllerCapabilities::parse(capability_word, structural, 0, 0x440, 0x1000, 0x4000,),
        Err(Error::InvalidRegister)
    );
}

#[test]
fn rejects_doorbell_and_runtime_registers_in_capability_space() {
    let structural = 8 | (2 << 8) | (4 << 24);
    let capability_word = 0x40 | (0x0100 << 16);
    assert_eq!(
        ControllerCapabilities::parse(capability_word, structural, 0, 0x20, 0x2000, 0x4000,),
        Err(Error::InvalidRegister)
    );
    assert_eq!(
        ControllerCapabilities::parse(capability_word, structural, 0, 0x1000, 0x20, 0x4000,),
        Err(Error::InvalidRegister)
    );
}

#[test]
fn bounds_the_port_register_area_using_max_ports() {
    let structural = 1 | (1 << 8) | (0xff << 24);
    assert_eq!(
        ControllerCapabilities::parse(0x0100_0020, structural, 0, 0x400, 0x600, 0x1000,),
        Err(Error::InvalidRegister)
    );
}

#[test]
fn rejects_extended_capability_header_overlapping_advertised_regions() {
    let structural = 8 | (2 << 8) | (4 << 24);
    let capability_word = 0x40 | (0x0100 << 16);
    for capability_params in [0x0008_0000, 0x0014_0000, 0x0400_0000, 0x0800_0000] {
        assert_eq!(
            ControllerCapabilities::parse(
                capability_word,
                structural,
                capability_params,
                0x1000,
                0x2000,
                0x4000,
            ),
            Err(Error::InvalidRegister)
        );
    }
}

#[test]
fn rejects_unaligned_offsets_and_overlapping_register_regions() {
    let structural = 8 | (2 << 8) | (4 << 24);
    let capability_word = 0x40 | (0x0100 << 16);
    assert_eq!(
        ControllerCapabilities::parse(capability_word, structural, 0, 0x1001, 0x2000, 0x4000,),
        Err(Error::InvalidRegister)
    );
    assert_eq!(
        ControllerCapabilities::parse(capability_word, structural, 0, 0x1000, 0x1000, 0x4000,),
        Err(Error::InvalidRegister)
    );
}

#[test]
fn checks_register_access_ranges_and_mapping_overflow() {
    assert_eq!(register_offset(0x20, 0x1c, 4), Ok(0x1c));
    assert_eq!(register_offset(0x20, 0x20, 4), Err(Error::InvalidRegister));
    assert_eq!(register_offset(0x20, 2, 4), Err(Error::InvalidRegister));
    assert_eq!(
        register_offset(0x20, u64::MAX, 4),
        Err(Error::InvalidRegister)
    );
    assert_eq!(
        validate_mapping_range(usize::MAX - 0x7ff, 0x1000),
        Err(Error::InvalidBar)
    );
    assert_eq!(validate_mapping_range(0x1000, 0x20), Err(Error::InvalidBar));
    assert_eq!(
        validate_mapping_range(0x1000, 0x1800),
        Err(Error::InvalidBar)
    );
}

struct FakeRegisters {
    reads: Vec<u64>,
    values: [(u64, u32); 5],
}

impl RegisterAccess for FakeRegisters {
    fn read_u32(&mut self, offset: u64) -> Result<u32, Error> {
        self.reads.push(offset);
        self.values
            .iter()
            .find_map(|(register, value)| (*register == offset).then_some(*value))
            .ok_or(Error::InvalidRegister)
    }
}

#[test]
fn inspection_reads_only_the_required_capability_registers() {
    let mut registers = FakeRegisters {
        reads: Vec::new(),
        values: [
            (CAPABILITY_OFFSET, 0x0100_0040),
            (STRUCTURAL_PARAMS_1_OFFSET, 8 | (2 << 8) | (4 << 24)),
            (CAPABILITY_PARAMS_1_OFFSET, 0x0040_0000),
            (DOORBELL_OFFSET_REGISTER, 0x1000),
            (RUNTIME_OFFSET_REGISTER, 0x2000),
        ],
    };
    let parsed = ControllerCapabilities::inspect(&mut registers, 0x4000).unwrap();
    assert_eq!(parsed.version, 0x0100);
    assert_eq!(parsed.max_slots, 8);
    assert_eq!(
        registers.reads,
        [
            CAPABILITY_OFFSET,
            STRUCTURAL_PARAMS_1_OFFSET,
            CAPABILITY_PARAMS_1_OFFSET,
            DOORBELL_OFFSET_REGISTER,
            RUNTIME_OFFSET_REGISTER,
        ]
    );
}

#[derive(Clone, Copy)]
struct MockFunction {
    address: PciAddress,
    config: [u8; 256],
    bar_masks: [u32; 6],
    probing: [bool; 6],
}

impl MockFunction {
    fn new(address: PciAddress, class: u8, subclass: u8, interface: u8) -> Self {
        let mut config = [0; 256];
        config[0..2].copy_from_slice(&0x1234u16.to_le_bytes());
        config[2..4].copy_from_slice(&0x5678u16.to_le_bytes());
        config[4..6].copy_from_slice(&7u16.to_le_bytes());
        config[8] = 1;
        config[9] = interface;
        config[10] = subclass;
        config[11] = class;
        config[0x0e] = 0;
        config[0x10..0x14].copy_from_slice(&0xfebf_0004u32.to_le_bytes());
        Self {
            address,
            config,
            bar_masks: [0xffff_c004, 0xffff_ffff, 0, 0, 0, 0],
            probing: [false; 6],
        }
    }
}

#[derive(Default)]
struct MockPci {
    functions: Vec<MockFunction>,
    command_writes: Rc<RefCell<Vec<(PciAddress, u16)>>>,
}

impl MockPci {
    fn with_functions(functions: Vec<MockFunction>) -> Self {
        Self {
            functions,
            ..Self::default()
        }
    }

    fn with_command_writes(
        functions: Vec<MockFunction>,
        command_writes: Rc<RefCell<Vec<(PciAddress, u16)>>>,
    ) -> Self {
        Self {
            functions,
            command_writes,
        }
    }

    fn function_mut(&mut self, address: PciAddress) -> Option<&mut MockFunction> {
        self.functions
            .iter_mut()
            .find(|function| function.address == address)
    }
}

impl PciConfigAccess for MockPci {
    fn read_u32(&mut self, address: PciAddress, offset: u8) -> Result<u32, pci_core::ConfigError> {
        let Some(function) = self.function_mut(address) else {
            return Ok(if offset == 0 { u32::MAX } else { 0 });
        };
        if (0x10..=0x24).contains(&offset) && (offset - 0x10).is_multiple_of(4) {
            let index = usize::from((offset - 0x10) / 4);
            if function.probing[index] {
                return Ok(function.bar_masks[index]);
            }
        }
        let offset = usize::from(offset);
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
        if (0x10..=0x24).contains(&offset) && (offset - 0x10).is_multiple_of(4) {
            let index = usize::from((offset - 0x10) / 4);
            function.probing[index] = value == u32::MAX;
        }
        let offset = usize::from(offset);
        function.config[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        if offset == 4 {
            self.command_writes
                .borrow_mut()
                .push((address, value as u16));
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
        if offset == 4 {
            self.command_writes.borrow_mut().push((address, value));
        }
        Ok(())
    }
}

fn address(device: u8) -> PciAddress {
    PciAddress {
        segment: 0,
        bus: 0,
        device,
        function: 0,
    }
}

#[test]
fn discovers_one_xhci_function_and_keeps_activation_disabled() {
    let expected = address(3);
    let command_writes = Rc::new(RefCell::new(Vec::new()));
    let pci = MockPci::with_command_writes(
        vec![MockFunction::new(expected, 0x0c, 0x03, 0x30)],
        Rc::clone(&command_writes),
    );
    let found = discover(pci).unwrap();
    assert_eq!(found.address(), expected);
    assert_eq!(found.bar0().address(), expected);
    assert_eq!(found.bar0().driver_id(), DRIVER_REGISTRATION.id);
    assert_eq!(found.bar0().index(), 0);
    assert_eq!(found.bar0().base(), 0xfebf_0000);
    assert_eq!(found.bar0().size(), 0x4000);
    assert!(!command_writes.borrow().is_empty());
    assert!(command_writes
        .borrow()
        .iter()
        .all(|(device, command)| *device == expected && command & 0x7 == 0));
    assert_eq!(
        command_writes
            .borrow()
            .last()
            .map(|(_, command)| command & 0x7),
        Some(0)
    );
}

#[test]
fn accepts_a_32_bit_nonprefetchable_memory_bar() {
    let mut function = MockFunction::new(address(3), 0x0c, 0x03, 0x30);
    function.config[0x10..0x14].copy_from_slice(&0xfebf_0000u32.to_le_bytes());
    function.bar_masks[0] = 0xffff_c000;
    function.bar_masks[1] = 0;
    let found = discover(MockPci::with_functions(vec![function])).unwrap();
    assert_eq!(found.bar0().base(), 0xfebf_0000);
    assert_eq!(found.bar0().size(), 0x4000);
}

#[test]
fn safely_skips_non_xhci_and_rejects_multiple_matches() {
    assert_eq!(discover(MockPci::default()), Err(Error::NotFound));
    let other_usb_controller =
        MockPci::with_functions(vec![MockFunction::new(address(2), 0x0c, 0x03, 0x20)]);
    assert_eq!(discover(other_usb_controller), Err(Error::NotFound));
    let ambiguous = MockPci::with_functions(vec![
        MockFunction::new(address(2), 0x0c, 0x03, 0x30),
        MockFunction::new(address(3), 0x0c, 0x03, 0x30),
    ]);
    assert_eq!(discover(ambiguous), Err(Error::Ambiguous));
}

#[test]
fn skips_a_nonresponding_pci_function() {
    let mut nonresponding = MockFunction::new(address(3), 0x0c, 0x03, 0x30);
    nonresponding.config[0..4].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(
        discover(MockPci::with_functions(vec![nonresponding])),
        Err(Error::NotFound)
    );
}

#[test]
fn rejects_prefetchable_or_io_bar_shapes() {
    let mut prefetchable = MockFunction::new(address(3), 0x0c, 0x03, 0x30);
    prefetchable.config[0x10..0x14].copy_from_slice(&0xfebf_000cu32.to_le_bytes());
    assert_eq!(
        discover(MockPci::with_functions(vec![prefetchable])),
        Err(Error::InvalidBar)
    );

    let mut io_bar = MockFunction::new(address(3), 0x0c, 0x03, 0x30);
    io_bar.config[0x10..0x14].copy_from_slice(&0x0000_c001u32.to_le_bytes());
    assert_eq!(
        discover(MockPci::with_functions(vec![io_bar])),
        Err(Error::InvalidBar)
    );
}
