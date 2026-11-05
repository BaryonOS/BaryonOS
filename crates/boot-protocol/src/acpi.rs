//! Bounded ACPI handoff and firmware-table discovery.
//!
//! The loader copies only the fixed-size UEFI RSDP into the retained BootInfo
//! page. Kernel callers provide a physical reader that validates firmware-memory
//! ownership before mapping each table.
use crate::PAGE_SIZE;

pub const HANDOFF_OFFSET: u64 = 192;
pub const HANDOFF_SIZE: usize = 64;
pub const HANDOFF_MAGIC: [u8; 8] = *b"NOSACPI\0";
pub const HANDOFF_VERSION: u16 = 1;
pub const MAX_TABLE_SIZE: usize = 4096;
pub const MAX_MCFG_REGIONS: usize = 32;
pub const MAX_DMAR_UNITS: usize = 16;
const ACPI_HEADER_SIZE: usize = 36;

const SOURCE_NONE: u8 = 0;
pub const SOURCE_ACPI_10: u8 = 1;
pub const SOURCE_ACPI_20: u8 = 2;
const SOURCE_REJECTED: u8 = 3;
const HANDOFF_CAPTURED: u8 = 1;
const HANDOFF_UNAVAILABLE: u8 = 0;
const HANDOFF_REJECTED: u8 = 2;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AcpiHandoff {
    pub magic: [u8; 8],
    pub version: u16,
    pub size: u16,
    pub state: u8,
    pub source: u8,
    pub rsdp_len: u8,
    pub reserved0: u8,
    pub rsdp: [u8; 36],
    pub reserved1: [u8; 12],
}

const _: () = assert!(core::mem::size_of::<AcpiHandoff>() == HANDOFF_SIZE);

impl AcpiHandoff {
    pub const fn unavailable() -> Self {
        Self {
            magic: HANDOFF_MAGIC,
            version: HANDOFF_VERSION,
            size: HANDOFF_SIZE as u16,
            state: HANDOFF_UNAVAILABLE,
            source: SOURCE_NONE,
            rsdp_len: 0,
            reserved0: 0,
            rsdp: [0; 36],
            reserved1: [0; 12],
        }
    }

    pub const fn rejected() -> Self {
        Self {
            magic: HANDOFF_MAGIC,
            version: HANDOFF_VERSION,
            size: HANDOFF_SIZE as u16,
            state: HANDOFF_REJECTED,
            source: SOURCE_REJECTED,
            rsdp_len: 0,
            reserved0: 0,
            rsdp: [0; 36],
            reserved1: [0; 12],
        }
    }

    pub fn captured(source: u8, rsdp: &[u8]) -> Result<Self, Error> {
        if !matches!(source, SOURCE_ACPI_10 | SOURCE_ACPI_20)
            || !matches!(rsdp.len(), 20 | 36)
            || (source == SOURCE_ACPI_10) != (rsdp.len() == 20)
        {
            return Err(Error::Handoff);
        }
        let mut result = Self::unavailable();
        result.state = HANDOFF_CAPTURED;
        result.source = source;
        result.rsdp_len = rsdp.len() as u8;
        result.rsdp[..rsdp.len()].copy_from_slice(rsdp);
        Ok(result)
    }

    pub fn validate(&self) -> Result<Option<&[u8]>, Error> {
        if self.magic != HANDOFF_MAGIC
            || self.version != HANDOFF_VERSION
            || usize::from(self.size) != HANDOFF_SIZE
            || self.reserved0 != 0
            || self.reserved1 != [0; 12]
        {
            return Err(Error::Handoff);
        }
        match (self.state, self.source, self.rsdp_len) {
            (HANDOFF_UNAVAILABLE, SOURCE_NONE, 0) if self.rsdp == [0; 36] => Ok(None),
            (HANDOFF_REJECTED, SOURCE_REJECTED, 0) if self.rsdp == [0; 36] => Err(Error::Handoff),
            (HANDOFF_CAPTURED, SOURCE_ACPI_10, 20) if self.rsdp[20..] == [0; 16] => {
                Ok(Some(&self.rsdp[..20]))
            }
            (HANDOFF_CAPTURED, SOURCE_ACPI_20, 36) => Ok(Some(&self.rsdp)),
            _ => Err(Error::Handoff),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Handoff,
    Rsdp,
    Address,
    Read,
    TableHeader,
    TableLength,
    Checksum,
    RootShape,
    DuplicateTable,
    Mcfg,
    Dmar,
    Capacity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Availability {
    MissingHandoff,
    RejectedHandoff,
    InvalidTables,
    MissingMcfg,
    MissingDmar,
    DmarWithoutUnits,
    DiscoveryComplete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct McfgRegion {
    pub base: u64,
    pub segment: u16,
    pub start_bus: u8,
    pub end_bus: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DmarUnit {
    pub register_base: u64,
    pub segment: u16,
    pub include_all: bool,
}

const EMPTY_MCFG: McfgRegion = McfgRegion {
    base: 0,
    segment: 0,
    start_bus: 0,
    end_bus: 0,
};
const EMPTY_DMAR: DmarUnit = DmarUnit {
    register_base: 0,
    segment: 0,
    include_all: false,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Discovery {
    pub availability: Availability,
    pub mcfg: [McfgRegion; MAX_MCFG_REGIONS],
    pub mcfg_count: u8,
    pub dmar: [DmarUnit; MAX_DMAR_UNITS],
    pub dmar_count: u8,
}

impl Discovery {
    pub const fn empty(availability: Availability) -> Self {
        Self {
            availability,
            mcfg: [EMPTY_MCFG; MAX_MCFG_REGIONS],
            mcfg_count: 0,
            dmar: [EMPTY_DMAR; MAX_DMAR_UNITS],
            dmar_count: 0,
        }
    }

    /// ACPI discovery is only inventory; no IOMMU is enabled by this interface.
    pub const fn dma_execution_allowed(self) -> bool {
        false
    }
}

/// Reads only the physical spans requested by the parser. Kernel implementations
/// must validate each span against the firmware memory map before mapping it.
pub trait PhysicalRead {
    fn read(&mut self, address: u64, output: &mut [u8]) -> Result<(), Error>;
}

/// Discover bounded MCFG and DMAR descriptors. All tables are checksummed,
/// length-limited, and read through `PhysicalRead`; unknown root entries are
/// ignored after validating their address shape.
pub fn discover(handoff: &AcpiHandoff, reader: &mut impl PhysicalRead) -> Discovery {
    let rsdp = match handoff.validate() {
        Ok(Some(rsdp)) => rsdp,
        Ok(None) => return Discovery::empty(Availability::MissingHandoff),
        Err(_) => return Discovery::empty(Availability::RejectedHandoff),
    };
    discover_tables(rsdp, reader).unwrap_or_else(|_| Discovery::empty(Availability::InvalidTables))
}

fn discover_tables(rsdp: &[u8], reader: &mut impl PhysicalRead) -> Result<Discovery, Error> {
    let root_address = parse_rsdp(rsdp)?;
    let mut root_header = [0u8; ACPI_HEADER_SIZE];
    reader
        .read(root_address, &mut root_header)
        .map_err(|_| Error::Read)?;
    let root_signature: &[u8; 4] = root_header[..4]
        .try_into()
        .map_err(|_| Error::TableHeader)?;
    if root_signature != b"XSDT" && root_signature != b"RSDT" {
        return Err(Error::TableHeader);
    }
    let root_length = table_length(&root_header)?;
    let entry_size = if root_signature == b"XSDT" { 8 } else { 4 };
    if root_length < ACPI_HEADER_SIZE
        || !(root_length - ACPI_HEADER_SIZE).is_multiple_of(entry_size)
    {
        return Err(Error::RootShape);
    }
    let mut root = [0u8; MAX_TABLE_SIZE];
    reader
        .read(root_address, &mut root[..root_length])
        .map_err(|_| Error::Read)?;
    validate_table(&root[..root_length], root_signature)?;

    let mut result = Discovery::empty(Availability::InvalidTables);
    let mut saw_mcfg = false;
    let mut saw_dmar = false;
    let mut offset = ACPI_HEADER_SIZE;
    while offset < root_length {
        let address = if entry_size == 8 {
            read_u64(&root, offset)?
        } else {
            u64::from(read_u32(&root, offset)?)
        };
        if address == 0 || !address.is_multiple_of(entry_size as u64) {
            return Err(Error::Address);
        }
        let mut header = [0u8; ACPI_HEADER_SIZE];
        reader.read(address, &mut header).map_err(|_| Error::Read)?;
        let signature = &header[..4];
        if signature == b"MCFG" {
            if saw_mcfg {
                return Err(Error::DuplicateTable);
            }
            saw_mcfg = true;
            let length = table_length(&header)?;
            if length > MAX_TABLE_SIZE {
                return Err(Error::TableLength);
            }
            let mut table = [0u8; MAX_TABLE_SIZE];
            reader
                .read(address, &mut table[..length])
                .map_err(|_| Error::Read)?;
            validate_table(&table[..length], b"MCFG")?;
            parse_mcfg(&table[..length], &mut result)?;
        } else if signature == b"DMAR" {
            if saw_dmar {
                return Err(Error::DuplicateTable);
            }
            saw_dmar = true;
            let length = table_length(&header)?;
            if length > MAX_TABLE_SIZE {
                return Err(Error::TableLength);
            }
            let mut table = [0u8; MAX_TABLE_SIZE];
            reader
                .read(address, &mut table[..length])
                .map_err(|_| Error::Read)?;
            validate_table(&table[..length], b"DMAR")?;
            parse_dmar(&table[..length], &mut result)?;
        }
        offset += entry_size;
    }
    result.availability = if result.mcfg_count == 0 {
        Availability::MissingMcfg
    } else if !saw_dmar {
        Availability::MissingDmar
    } else if result.dmar_count == 0 {
        Availability::DmarWithoutUnits
    } else {
        Availability::DiscoveryComplete
    };
    Ok(result)
}

fn parse_rsdp(rsdp: &[u8]) -> Result<u64, Error> {
    if !(rsdp.len() == 20 || rsdp.len() == 36)
        || &rsdp[..8] != b"RSD PTR "
        || checksum(&rsdp[..20]) != 0
    {
        return Err(Error::Rsdp);
    }
    let revision = rsdp[15];
    if revision < 2 {
        if rsdp.len() != 20 {
            return Err(Error::Rsdp);
        }
        let root = u64::from(read_u32(rsdp, 16)?);
        if root == 0 || !root.is_multiple_of(4) {
            return Err(Error::Address);
        }
        return Ok(root);
    }
    if rsdp.len() != 36
        || read_u32(rsdp, 20)? != 36
        || checksum(rsdp) != 0
        || rsdp[33..36] != [0; 3]
    {
        return Err(Error::Rsdp);
    }
    let root = read_u64(rsdp, 24)?;
    if root == 0 || !root.is_multiple_of(8) {
        return Err(Error::Address);
    }
    Ok(root)
}

fn parse_mcfg(table: &[u8], result: &mut Discovery) -> Result<(), Error> {
    if table.len() < 44 || read_u64(table, 36)? != 0 || !(table.len() - 44).is_multiple_of(16) {
        return Err(Error::Mcfg);
    }
    for entry in table[44..].chunks_exact(16) {
        if result.mcfg_count as usize == MAX_MCFG_REGIONS {
            return Err(Error::Capacity);
        }
        let base = read_u64(entry, 0)?;
        let segment = read_u16(entry, 8)?;
        let start_bus = entry[10];
        let end_bus = entry[11];
        if base == 0
            || !base.is_multiple_of(1 << 20)
            || start_bus > end_bus
            || read_u32(entry, 12)? != 0
        {
            return Err(Error::Mcfg);
        }
        let span = (u64::from(end_bus) - u64::from(start_bus) + 1)
            .checked_mul(1 << 20)
            .ok_or(Error::Mcfg)?;
        let end = base.checked_add(span).ok_or(Error::Mcfg)?;
        if end > (1 << 52) {
            return Err(Error::Mcfg);
        }
        for existing in &result.mcfg[..result.mcfg_count as usize] {
            if existing.segment == segment
                && u16::from(start_bus) <= u16::from(existing.end_bus)
                && u16::from(existing.start_bus) <= u16::from(end_bus)
            {
                return Err(Error::Mcfg);
            }
            let existing_span =
                (u64::from(existing.end_bus) - u64::from(existing.start_bus) + 1) << 20;
            let existing_end = existing
                .base
                .checked_add(existing_span)
                .ok_or(Error::Mcfg)?;
            if base < existing_end && existing.base < end {
                return Err(Error::Mcfg);
            }
        }
        result.mcfg[result.mcfg_count as usize] = McfgRegion {
            base,
            segment,
            start_bus,
            end_bus,
        };
        result.mcfg_count += 1;
    }
    Ok(())
}

fn parse_dmar(table: &[u8], result: &mut Discovery) -> Result<(), Error> {
    if table.len() < 48
        || table[36] > 63
        || table[37] & !3 != 0
        || read_u16(table, 38)? != 0
        || table[40..48] != [0; 8]
    {
        return Err(Error::Dmar);
    }
    let mut offset = 48;
    while offset < table.len() {
        let kind = read_u16(table, offset)?;
        let length = usize::from(read_u16(table, offset + 2)?);
        if length < 4
            || offset
                .checked_add(length)
                .is_none_or(|end| end > table.len())
        {
            return Err(Error::Dmar);
        }
        let structure = &table[offset..offset + length];
        if kind == 0 {
            if length < 16 || structure[4] & !1 != 0 || structure[5] != 0 {
                return Err(Error::Dmar);
            }
            if result.dmar_count as usize == MAX_DMAR_UNITS {
                return Err(Error::Capacity);
            }
            let register_base = read_u64(structure, 8)?;
            if register_base == 0
                || !register_base.is_multiple_of(PAGE_SIZE)
                || register_base
                    .checked_add(PAGE_SIZE)
                    .is_none_or(|end| end > (1 << 52))
            {
                return Err(Error::Dmar);
            }
            result.dmar[result.dmar_count as usize] = DmarUnit {
                register_base,
                segment: read_u16(structure, 6)?,
                include_all: structure[4] & 1 != 0,
            };
            result.dmar_count += 1;
        } else if kind == 1 {
            if length < 24 || read_u16(structure, 4)? != 0 {
                return Err(Error::Dmar);
            }
            let base = read_u64(structure, 8)?;
            let limit = read_u64(structure, 16)?;
            if !base.is_multiple_of(PAGE_SIZE)
                || base >= (1 << 52)
                || base > limit
                || limit >= (1 << 52)
                || !limit
                    .checked_add(1)
                    .is_some_and(|end| end.is_multiple_of(PAGE_SIZE))
            {
                return Err(Error::Dmar);
            }
        }
        offset += length;
    }
    Ok(())
}

fn table_length(header: &[u8; ACPI_HEADER_SIZE]) -> Result<usize, Error> {
    let length = usize::try_from(read_u32(header, 4)?).map_err(|_| Error::TableLength)?;
    if !(ACPI_HEADER_SIZE..=MAX_TABLE_SIZE).contains(&length) {
        return Err(Error::TableLength);
    }
    Ok(length)
}

fn validate_table(table: &[u8], signature: &[u8; 4]) -> Result<(), Error> {
    if table.len() < ACPI_HEADER_SIZE || &table[..4] != signature {
        return Err(Error::TableHeader);
    }
    if read_u32(table, 4)? as usize != table.len() {
        return Err(Error::TableLength);
    }
    if checksum(table) != 0 {
        return Err(Error::Checksum);
    }
    Ok(())
}

fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, Error> {
    let bytes = bytes
        .get(offset..offset.checked_add(2).ok_or(Error::TableLength)?)
        .ok_or(Error::TableLength)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let bytes = bytes
        .get(offset..offset.checked_add(4).ok_or(Error::TableLength)?)
        .ok_or(Error::TableLength)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, Error> {
    let bytes = bytes
        .get(offset..offset.checked_add(8).ok_or(Error::TableLength)?)
        .ok_or(Error::TableLength)?;
    Ok(u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::{vec, vec::Vec};

    struct Fixture {
        tables: Vec<(u64, Vec<u8>)>,
    }

    impl PhysicalRead for Fixture {
        fn read(&mut self, address: u64, output: &mut [u8]) -> Result<(), Error> {
            let (base, table) = self
                .tables
                .iter()
                .find(|(base, table)| {
                    address >= *base
                        && address
                            .checked_add(output.len() as u64)
                            .is_some_and(|end| end <= *base + table.len() as u64)
                })
                .ok_or(Error::Read)?;
            let start = (address - *base) as usize;
            output.copy_from_slice(&table[start..start + output.len()]);
            Ok(())
        }
    }

    fn checksum_byte(bytes: &mut [u8], index: usize) {
        bytes[index] = 0;
        bytes[index] = 0u8.wrapping_sub(checksum(bytes));
    }

    fn table(signature: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0u8; ACPI_HEADER_SIZE + body.len()];
        let length = bytes.len() as u32;
        bytes[..4].copy_from_slice(signature);
        bytes[4..8].copy_from_slice(&length.to_le_bytes());
        bytes[8] = 1;
        bytes[10..16].copy_from_slice(b"NOSOS ");
        bytes[16..24].copy_from_slice(b"TEST    ");
        bytes[24..28].copy_from_slice(&1u32.to_le_bytes());
        bytes[28..32].copy_from_slice(b"NOS ");
        bytes[32..36].copy_from_slice(&1u32.to_le_bytes());
        bytes[36..].copy_from_slice(body);
        checksum_byte(&mut bytes, 9);
        bytes
    }

    fn valid_fixture(include_dmar: bool) -> (AcpiHandoff, Fixture) {
        let mcfg_entry = [
            0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x00, 0x00,
            0x00, 0x00,
        ];
        let mcfg = table(b"MCFG", &[&[0; 8][..], &mcfg_entry].concat());
        let mut dmar_body = [0u8; 12];
        dmar_body[0] = 47;
        let mut rmrr = [0u8; 24];
        rmrr[..2].copy_from_slice(&1u16.to_le_bytes());
        rmrr[2..4].copy_from_slice(&24u16.to_le_bytes());
        rmrr[8..16].copy_from_slice(&0x2000u64.to_le_bytes());
        rmrr[16..24].copy_from_slice(&0x2fffu64.to_le_bytes());
        let dmar = table(
            b"DMAR",
            &[
                &dmar_body[..],
                &[0, 0, 16, 0, 1, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0],
                &rmrr,
            ]
            .concat(),
        );
        let mut xsdt_body = Vec::new();
        xsdt_body.extend_from_slice(&0x2000u64.to_le_bytes());
        if include_dmar {
            xsdt_body.extend_from_slice(&0x3000u64.to_le_bytes());
        }
        let xsdt = table(b"XSDT", &xsdt_body);
        let mut rsdp = [0u8; 36];
        rsdp[..8].copy_from_slice(b"RSD PTR ");
        rsdp[9..15].copy_from_slice(b"NOSOS ");
        rsdp[15] = 2;
        rsdp[20..24].copy_from_slice(&36u32.to_le_bytes());
        rsdp[24..32].copy_from_slice(&0x1000u64.to_le_bytes());
        rsdp[8] = 0u8.wrapping_sub(checksum(&rsdp[..20]));
        rsdp[32] = 0u8.wrapping_sub(checksum(&rsdp));
        let handoff = AcpiHandoff::captured(SOURCE_ACPI_20, &rsdp).unwrap();
        let mut tables = vec![(0x1000, xsdt), (0x2000, mcfg)];
        if include_dmar {
            tables.push((0x3000, dmar));
        }
        (handoff, Fixture { tables })
    }

    #[test]
    fn handoff_states_and_fixed_wire_shape_are_checked() {
        assert_eq!(AcpiHandoff::unavailable().validate(), Ok(None));
        assert_eq!(
            AcpiHandoff::captured(SOURCE_ACPI_10, &[0; 20])
                .unwrap()
                .validate()
                .unwrap()
                .unwrap()
                .len(),
            20
        );
        let mut bad = AcpiHandoff::unavailable();
        bad.reserved1[0] = 1;
        assert_eq!(bad.validate(), Err(Error::Handoff));
        let mut bad = AcpiHandoff::rejected();
        bad.state = HANDOFF_CAPTURED;
        assert_eq!(bad.validate(), Err(Error::Handoff));
    }

    #[test]
    fn discovers_bounded_mcfg_and_dmar_but_never_enables_dma() {
        let (handoff, mut fixture) = valid_fixture(true);
        assert_eq!(handoff.validate().unwrap().unwrap().len(), 36);
        assert_eq!(checksum(&handoff.rsdp[..20]), 0);
        assert_eq!(checksum(&handoff.rsdp), 0);
        assert_eq!(read_u32(&handoff.rsdp, 20), Ok(36));
        assert_eq!(handoff.rsdp[33..36], [0; 3]);
        let discovered = discover(&handoff, &mut fixture);
        assert_eq!(discovered.availability, Availability::DiscoveryComplete);
        assert_eq!(discovered.mcfg_count, 1);
        assert_eq!(discovered.mcfg[0].base, 0x8000_0000);
        assert_eq!(discovered.mcfg[0].segment, 0);
        assert_eq!(discovered.dmar_count, 1);
        assert_eq!(discovered.dmar[0].register_base, 0x1000);
        assert!(!discovered.dma_execution_allowed());
    }

    #[test]
    fn no_dmar_is_an_explicit_unavailable_state() {
        let (handoff, mut fixture) = valid_fixture(false);
        let discovered = discover(&handoff, &mut fixture);
        assert_eq!(discovered.availability, Availability::MissingDmar);
        assert!(!discovered.dma_execution_allowed());
    }

    #[test]
    fn rejects_bad_checksums_lengths_addresses_and_mcfg_overlap() {
        let (handoff, mut fixture) = valid_fixture(true);
        fixture.tables[0].1[9] ^= 1;
        assert_eq!(
            discover(&handoff, &mut fixture).availability,
            Availability::InvalidTables
        );
        let (handoff, mut fixture) = valid_fixture(true);
        fixture.tables[2].1[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            discover(&handoff, &mut fixture).availability,
            Availability::InvalidTables
        );
        let (handoff, mut fixture) = valid_fixture(true);
        let duplicate = fixture.tables[1].1[44..60].to_vec();
        fixture.tables[1].1.extend_from_slice(&duplicate);
        let new_length = fixture.tables[1].1.len() as u32;
        fixture.tables[1].1[4..8].copy_from_slice(&new_length.to_le_bytes());
        checksum_byte(&mut fixture.tables[1].1, 9);
        assert_eq!(
            discover(&handoff, &mut fixture).availability,
            Availability::InvalidTables
        );
    }

    #[test]
    fn rejects_dmar_register_and_reserved_memory_ranges_outside_bounds() {
        let (handoff, mut fixture) = valid_fixture(true);
        fixture.tables[2].1[56..64].copy_from_slice(&(1u64 << 52).to_le_bytes());
        checksum_byte(&mut fixture.tables[2].1, 9);
        assert_eq!(
            discover(&handoff, &mut fixture).availability,
            Availability::InvalidTables
        );

        let (handoff, mut fixture) = valid_fixture(true);
        fixture.tables[2].1[72..80].copy_from_slice(&0x2001u64.to_le_bytes());
        checksum_byte(&mut fixture.tables[2].1, 9);
        assert_eq!(
            discover(&handoff, &mut fixture).availability,
            Availability::InvalidTables
        );
    }

    fn set_length_and_checksum(table: &mut [u8]) {
        let length = table.len() as u32;
        table[4..8].copy_from_slice(&length.to_le_bytes());
        checksum_byte(table, 9);
    }

    #[test]
    fn rejects_root_shape_short_headers_duplicate_mcfg_and_extended_checksum() {
        let (handoff, _) = valid_fixture(true);
        let rsdp = handoff.validate().unwrap().unwrap().to_vec();

        let (_, mut fixture) = valid_fixture(true);
        fixture.tables[0].1.push(0);
        set_length_and_checksum(&mut fixture.tables[0].1);
        assert_eq!(
            discover_tables(&rsdp, &mut fixture).err(),
            Some(Error::RootShape)
        );

        let (_, mut fixture) = valid_fixture(true);
        fixture.tables[1].1[4..8].copy_from_slice(&35u32.to_le_bytes());
        assert_eq!(
            discover_tables(&rsdp, &mut fixture).err(),
            Some(Error::TableLength)
        );

        let (_, mut fixture) = valid_fixture(false);
        fixture.tables[0]
            .1
            .extend_from_slice(&0x2000u64.to_le_bytes());
        set_length_and_checksum(&mut fixture.tables[0].1);
        assert_eq!(
            discover_tables(&rsdp, &mut fixture).err(),
            Some(Error::DuplicateTable)
        );

        let mut bad = rsdp.clone();
        bad[32] ^= 1;
        assert_eq!(parse_rsdp(&bad), Err(Error::Rsdp));
    }

    #[test]
    fn rsdt_root_uses_four_byte_entries_and_checks_their_shape() {
        let mut rsdp = [0u8; 20];
        rsdp[..8].copy_from_slice(b"RSD PTR ");
        rsdp[16..20].copy_from_slice(&0x1000u32.to_le_bytes());
        rsdp[8] = 0u8.wrapping_sub(checksum(&rsdp));

        let (_, mut fixture) = valid_fixture(false);
        fixture.tables[0].1 = table(b"RSDT", &0x2000u32.to_le_bytes());
        let discovered = discover_tables(&rsdp, &mut fixture).unwrap();
        assert_eq!(discovered.availability, Availability::MissingDmar);
        assert_eq!(discovered.mcfg_count, 1);

        fixture.tables[0].1.extend_from_slice(&[0; 2]);
        set_length_and_checksum(&mut fixture.tables[0].1);
        assert_eq!(
            discover_tables(&rsdp, &mut fixture).err(),
            Some(Error::RootShape)
        );
    }

    #[test]
    fn rejects_bad_rsdp_checksum_and_untrusted_handoff() {
        let (mut handoff, mut fixture) = valid_fixture(true);
        handoff.rsdp[8] ^= 1;
        assert_eq!(
            discover(&handoff, &mut fixture).availability,
            Availability::InvalidTables
        );
        handoff = AcpiHandoff::rejected();
        assert_eq!(
            discover(&handoff, &mut fixture).availability,
            Availability::RejectedHandoff
        );
    }
}
