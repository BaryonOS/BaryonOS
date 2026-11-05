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
/// Enabled processor entries retained from one MADT; more fails the MADT closed.
pub const MAX_MADT_CPUS: usize = 32;
const MADT_FIXED_SIZE: usize = 44;
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
    Madt,
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

/// One enabled processor from the MADT. `apic_id` is the 8-bit xAPIC ID for
/// type-0 entries and the 32-bit x2APIC ID for type-9 entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MadtCpu {
    pub apic_id: u32,
    pub uid: u32,
    pub x2apic: bool,
}

const EMPTY_CPU: MadtCpu = MadtCpu {
    apic_id: 0,
    uid: 0,
    x2apic: false,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MadtStatus {
    Missing,
    /// Malformed, duplicate or over-capacity MADT. SMP stays unavailable; the
    /// other ACPI inventory is independent of this failure.
    Invalid,
    Valid,
}

/// Processor inventory only. Nothing here starts a processor or grants an
/// interrupt route; the kernel revalidates the base against IA32_APIC_BASE.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Madt {
    pub status: MadtStatus,
    pub local_apic_base: u64,
    pub pcat_compat: bool,
    pub cpus: [MadtCpu; MAX_MADT_CPUS],
    pub cpu_count: u8,
    /// Disabled but online-capable (hot-add) entries; never started at boot.
    pub online_capable: u8,
    /// Disabled entries that are not online-capable.
    pub disabled: u8,
    pub io_apic_count: u8,
}

impl Madt {
    const fn unavailable(status: MadtStatus) -> Self {
        Self {
            status,
            local_apic_base: 0,
            pcat_compat: false,
            cpus: [EMPTY_CPU; MAX_MADT_CPUS],
            cpu_count: 0,
            online_capable: 0,
            disabled: 0,
            io_apic_count: 0,
        }
    }

    pub fn cpus(&self) -> &[MadtCpu] {
        if self.status == MadtStatus::Valid {
            &self.cpus[..usize::from(self.cpu_count)]
        } else {
            &[]
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Discovery {
    pub availability: Availability,
    pub mcfg: [McfgRegion; MAX_MCFG_REGIONS],
    pub mcfg_count: u8,
    pub dmar: [DmarUnit; MAX_DMAR_UNITS],
    pub dmar_count: u8,
    pub madt: Madt,
}

impl Discovery {
    pub const fn empty(availability: Availability) -> Self {
        Self {
            availability,
            mcfg: [EMPTY_MCFG; MAX_MCFG_REGIONS],
            mcfg_count: 0,
            dmar: [EMPTY_DMAR; MAX_DMAR_UNITS],
            dmar_count: 0,
            madt: Madt::unavailable(MadtStatus::Missing),
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

/// Discover bounded MCFG and DMAR descriptors plus the MADT processor
/// inventory. All tables are checksummed, length-limited, and read through
/// `PhysicalRead`; unknown root entries are ignored after validating their
/// address shape. MADT content errors only mark `madt` invalid.
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
    let mut saw_madt = false;
    // One reusable table buffer keeps discovery within the 16-KiB kernel stack.
    let mut table = [0u8; MAX_TABLE_SIZE];
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
            reader
                .read(address, &mut table[..length])
                .map_err(|_| Error::Read)?;
            validate_table(&table[..length], b"DMAR")?;
            parse_dmar(&table[..length], &mut result)?;
        } else if signature == b"APIC" {
            // A second MADT is ambiguous: fail SMP closed, keep other inventory.
            // Parsed in place: no Madt temporaries on the 16-KiB kernel stack.
            let parsed = if saw_madt {
                Err(Error::DuplicateTable)
            } else {
                saw_madt = true;
                match table_length(&header) {
                    Ok(length) => {
                        reader
                            .read(address, &mut table[..length])
                            .map_err(|_| Error::Read)?;
                        validate_table(&table[..length], b"APIC")
                            .and_then(|()| parse_madt(&table[..length], &mut result.madt))
                    }
                    Err(error) => Err(error),
                }
            };
            if parsed.is_err() {
                result.madt.status = MadtStatus::Invalid;
                result.madt.cpu_count = 0;
            }
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

const MADT_LOCAL_APIC: u8 = 0;
const MADT_IO_APIC: u8 = 1;
const MADT_LOCAL_APIC_OVERRIDE: u8 = 5;
const MADT_LOCAL_X2APIC: u8 = 9;
const MADT_ENABLED: u32 = 1;
const MADT_ONLINE_CAPABLE: u32 = 2;

/// ACPI 6.5 section 5.2.12. Only interpreted structures need an exact length;
/// every structure must be at least two bytes and lie wholly inside the table.
/// Unknown/OEM structures are skipped. Reserved MADT and processor flag bits
/// are ignored because they do not change which processors are started.
///
/// Writes into `madt` in place (it stays `Invalid` until fully accepted) so
/// that no copy of the processor array lives on the kernel stack.
fn parse_madt(table: &[u8], madt: &mut Madt) -> Result<(), Error> {
    madt.status = MadtStatus::Invalid;
    madt.cpu_count = 0;
    madt.online_capable = 0;
    madt.disabled = 0;
    madt.io_apic_count = 0;
    madt.local_apic_base = 0;
    if table.len() < MADT_FIXED_SIZE {
        return Err(Error::Madt);
    }
    madt.pcat_compat = read_u32(table, 40)? & 1 != 0;
    let mut base = u64::from(read_u32(table, 36)?);
    let mut saw_override = false;
    let mut offset = MADT_FIXED_SIZE;
    while offset < table.len() {
        let kind = table[offset];
        let length = usize::from(*table.get(offset + 1).ok_or(Error::Madt)?);
        let end = offset.checked_add(length).ok_or(Error::Madt)?;
        if length < 2 || end > table.len() {
            return Err(Error::Madt);
        }
        let entry = &table[offset..end];
        let processor = match kind {
            MADT_LOCAL_APIC => {
                if length != 8 {
                    return Err(Error::Madt);
                }
                let apic_id = u32::from(entry[3]);
                // 0xff is the xAPIC broadcast destination, never one CPU.
                Some((
                    apic_id,
                    u32::from(entry[2]),
                    read_u32(entry, 4)?,
                    false,
                    0xff,
                ))
            }
            MADT_LOCAL_X2APIC => {
                if length != 16 {
                    return Err(Error::Madt);
                }
                let apic_id = read_u32(entry, 4)?;
                Some((
                    apic_id,
                    read_u32(entry, 12)?,
                    read_u32(entry, 8)?,
                    true,
                    u32::MAX,
                ))
            }
            MADT_IO_APIC => {
                if length != 12 {
                    return Err(Error::Madt);
                }
                madt.io_apic_count = madt.io_apic_count.checked_add(1).ok_or(Error::Madt)?;
                None
            }
            MADT_LOCAL_APIC_OVERRIDE => {
                if length != 12 || saw_override {
                    return Err(Error::Madt);
                }
                saw_override = true;
                base = read_u64(entry, 4)?;
                None
            }
            _ => None,
        };
        if let Some((apic_id, uid, flags, x2apic, broadcast)) = processor {
            let enabled = flags & MADT_ENABLED != 0;
            let online_capable = flags & MADT_ONLINE_CAPABLE != 0;
            if enabled && online_capable {
                // ACPI 6.3+: Online Capable is reserved-zero when Enabled is set.
                return Err(Error::Madt);
            }
            if enabled {
                if apic_id == broadcast
                    || madt.cpus[..usize::from(madt.cpu_count)]
                        .iter()
                        .any(|cpu| cpu.apic_id == apic_id)
                {
                    return Err(Error::Madt);
                }
                let index = usize::from(madt.cpu_count);
                if index == MAX_MADT_CPUS {
                    return Err(Error::Capacity);
                }
                madt.cpus[index] = MadtCpu {
                    apic_id,
                    uid,
                    x2apic,
                };
                madt.cpu_count += 1;
            } else if online_capable {
                madt.online_capable = madt.online_capable.checked_add(1).ok_or(Error::Madt)?;
            } else {
                madt.disabled = madt.disabled.checked_add(1).ok_or(Error::Madt)?;
            }
        }
        offset = end;
    }
    if base == 0 || !base.is_multiple_of(PAGE_SIZE) || base >= (1 << 52) || madt.cpu_count == 0 {
        return Err(Error::Madt);
    }
    madt.local_apic_base = base;
    madt.status = MadtStatus::Valid;
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

    fn lapic(uid: u8, apic_id: u8, flags: u32) -> Vec<u8> {
        let mut entry = vec![MADT_LOCAL_APIC, 8, uid, apic_id];
        entry.extend_from_slice(&flags.to_le_bytes());
        entry
    }

    fn x2apic(apic_id: u32, flags: u32, uid: u32) -> Vec<u8> {
        let mut entry = vec![MADT_LOCAL_X2APIC, 16, 0, 0];
        entry.extend_from_slice(&apic_id.to_le_bytes());
        entry.extend_from_slice(&flags.to_le_bytes());
        entry.extend_from_slice(&uid.to_le_bytes());
        entry
    }

    fn base_override(base: u64) -> Vec<u8> {
        let mut entry = vec![MADT_LOCAL_APIC_OVERRIDE, 12, 0, 0];
        entry.extend_from_slice(&base.to_le_bytes());
        entry
    }

    fn madt(base: u32, entries: &[Vec<u8>]) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(&base.to_le_bytes());
        body.extend_from_slice(&1u32.to_le_bytes());
        for entry in entries {
            body.extend_from_slice(entry);
        }
        table(b"APIC", &body)
    }

    /// QEMU-like inventory: two enabled xAPICs, one disabled, one hot-add,
    /// one enabled x2APIC, an I/O APIC, an override-free base and skipped
    /// ISO/NMI/OEM structures.
    fn qemu_like_entries() -> Vec<Vec<u8>> {
        let mut io_apic = vec![MADT_IO_APIC, 12, 0, 0];
        io_apic.extend_from_slice(&0xfec0_0000u32.to_le_bytes());
        io_apic.extend_from_slice(&0u32.to_le_bytes());
        vec![
            lapic(0, 0, MADT_ENABLED),
            lapic(1, 1, MADT_ENABLED),
            lapic(2, 2, 0),
            lapic(3, 3, MADT_ONLINE_CAPABLE),
            x2apic(300, MADT_ENABLED, 4),
            io_apic,
            vec![2, 10, 0, 0, 2, 0, 0, 0, 0, 0],
            vec![4, 6, 0xff, 0, 0, 1],
            vec![0x80, 3, 0x55],
        ]
    }

    /// MCFG at 0x2000 plus each MADT at 0x4000, 0x5000, ...
    fn madt_fixture(madts: Vec<Vec<u8>>) -> (AcpiHandoff, Fixture) {
        let (handoff, mut fixture) = valid_fixture(false);
        let mut xsdt_body = 0x2000u64.to_le_bytes().to_vec();
        for (index, bytes) in madts.into_iter().enumerate() {
            let address = 0x4000 + 0x1000 * index as u64;
            xsdt_body.extend_from_slice(&address.to_le_bytes());
            fixture.tables.push((address, bytes));
        }
        fixture.tables[0] = (0x1000, table(b"XSDT", &xsdt_body));
        (handoff, fixture)
    }

    fn discover_madt(madts: Vec<Vec<u8>>) -> Discovery {
        let (handoff, mut fixture) = madt_fixture(madts);
        discover(&handoff, &mut fixture)
    }

    /// MADT failure must not hide the independent PCI inventory.
    fn assert_only_madt_invalid(discovered: Discovery) {
        assert_eq!(discovered.madt.status, MadtStatus::Invalid);
        assert!(discovered.madt.cpus().is_empty());
        assert_eq!(discovered.availability, Availability::MissingDmar);
        assert_eq!(discovered.mcfg_count, 1);
    }

    fn rechecksum(mut table: Vec<u8>) -> Vec<u8> {
        let length = table.len() as u32;
        table[4..8].copy_from_slice(&length.to_le_bytes());
        checksum_byte(&mut table, 9);
        table
    }

    #[test]
    fn madt_inventory_keeps_only_enabled_unique_processors() {
        let discovered = discover_madt(vec![madt(0xfee0_0000, &qemu_like_entries())]);
        assert_eq!(discovered.availability, Availability::MissingDmar);
        let madt = discovered.madt;
        assert_eq!(madt.status, MadtStatus::Valid);
        assert_eq!(madt.local_apic_base, 0xfee0_0000);
        assert!(madt.pcat_compat);
        assert_eq!(madt.online_capable, 1);
        assert_eq!(madt.disabled, 1);
        assert_eq!(madt.io_apic_count, 1);
        let ids: Vec<(u32, u32, bool)> = madt
            .cpus()
            .iter()
            .map(|cpu| (cpu.apic_id, cpu.uid, cpu.x2apic))
            .collect();
        assert_eq!(ids, vec![(0, 0, false), (1, 1, false), (300, 4, true)]);
    }

    #[test]
    fn missing_madt_is_explicit_and_other_inventory_survives() {
        let discovered = discover_madt(vec![]);
        assert_eq!(discovered.madt.status, MadtStatus::Missing);
        assert!(discovered.madt.cpus().is_empty());
        assert_eq!(discovered.mcfg_count, 1);
        let (handoff, mut fixture) = valid_fixture(true);
        fixture.tables[0].1[9] ^= 1;
        assert_eq!(
            discover(&handoff, &mut fixture).madt.status,
            MadtStatus::Missing
        );
    }

    #[test]
    fn single_local_apic_address_override_replaces_the_base() {
        let mut entries = qemu_like_entries();
        entries.push(base_override(0x1_0000_0000));
        let discovered = discover_madt(vec![madt(0xfee0_0000, &entries)]);
        assert_eq!(discovered.madt.status, MadtStatus::Valid);
        assert_eq!(discovered.madt.local_apic_base, 0x1_0000_0000);
    }

    #[test]
    fn malformed_processor_inventory_fails_only_smp_closed() {
        let entries = qemu_like_entries;
        let mut cases: Vec<Vec<u8>> = Vec::new();
        let with = |extra: Vec<Vec<u8>>| {
            let mut all = entries();
            all.extend(extra);
            madt(0xfee0_0000, &all)
        };
        cases.push(with(vec![lapic(9, 1, MADT_ENABLED)]));
        cases.push(with(vec![x2apic(1, MADT_ENABLED, 9)]));
        cases.push(with(vec![lapic(9, 0xff, MADT_ENABLED)]));
        cases.push(with(vec![x2apic(u32::MAX, MADT_ENABLED, 9)]));
        cases.push(with(vec![lapic(9, 9, MADT_ENABLED | MADT_ONLINE_CAPABLE)]));
        cases.push(with(vec![
            base_override(0xfee0_0000),
            base_override(0xfee0_0000),
        ]));
        cases.push(with(vec![base_override(0)]));
        cases.push(with(vec![base_override(0xfee0_0800)]));
        cases.push(with(vec![base_override(1 << 52)]));
        cases.push(with(vec![vec![0x80, 0]]));
        cases.push(with(vec![vec![0x80, 1]]));
        cases.push(with(vec![vec![0x80]]));
        cases.push(with(vec![vec![0x80, 40, 0, 0]]));
        let mut short_lapic = lapic(9, 9, MADT_ENABLED);
        short_lapic[1] = 7;
        short_lapic.pop();
        cases.push(with(vec![short_lapic]));
        let mut long_x2apic = x2apic(400, MADT_ENABLED, 9);
        long_x2apic[1] = 17;
        long_x2apic.push(0);
        cases.push(with(vec![long_x2apic]));
        let mut short_io_apic = vec![MADT_IO_APIC, 8];
        short_io_apic.extend_from_slice(&[0; 6]);
        cases.push(with(vec![short_io_apic]));
        cases.push(madt(0, &entries()));
        cases.push(madt(0xfee0_0010, &entries()));
        cases.push(madt(
            0xfee0_0000,
            &[lapic(0, 0, 0), lapic(1, 1, MADT_ONLINE_CAPABLE)],
        ));
        let too_many: Vec<Vec<u8>> = (0..=MAX_MADT_CPUS as u8)
            .map(|id| lapic(id, id, MADT_ENABLED))
            .collect();
        cases.push(madt(0xfee0_0000, &too_many));
        let mut bad_checksum = madt(0xfee0_0000, &entries());
        bad_checksum[9] ^= 1;
        cases.push(bad_checksum);
        let mut too_short = madt(0xfee0_0000, &[]);
        too_short.truncate(40);
        cases.push(rechecksum(too_short));
        let mut oversized = madt(0xfee0_0000, &entries());
        oversized[4..8].copy_from_slice(&(MAX_TABLE_SIZE as u32 + 1).to_le_bytes());
        cases.push(oversized);
        for (index, case) in cases.into_iter().enumerate() {
            let discovered = discover_madt(vec![case]);
            assert_eq!(
                discovered.madt.status,
                MadtStatus::Invalid,
                "case {index} unexpectedly accepted"
            );
            assert_only_madt_invalid(discovered);
        }
        // Exactly the capacity is still accepted.
        let full: Vec<Vec<u8>> = (0..MAX_MADT_CPUS as u8)
            .map(|id| lapic(id, id, MADT_ENABLED))
            .collect();
        let accepted = discover_madt(vec![madt(0xfee0_0000, &full)]).madt;
        assert_eq!(accepted.status, MadtStatus::Valid);
        assert_eq!(accepted.cpus().len(), MAX_MADT_CPUS);
    }

    #[test]
    fn a_second_madt_makes_the_processor_inventory_ambiguous() {
        let first = madt(0xfee0_0000, &qemu_like_entries());
        for second in [
            first.clone(),
            madt(0xfee0_0000, &[lapic(0, 7, MADT_ENABLED)]),
        ] {
            assert_only_madt_invalid(discover_madt(vec![first.clone(), second]));
        }
        let mut bad = madt(0xfee0_0000, &[lapic(0, 7, MADT_ENABLED)]);
        bad[9] ^= 1;
        assert_only_madt_invalid(discover_madt(vec![bad, first]));
    }

    #[test]
    fn mutated_madt_never_panics_and_valid_results_keep_invariants() {
        let original = madt(0xfee0_0000, &qemu_like_entries());
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let (mut valid, mut invalid) = (0, 0);
        for _ in 0..4000 {
            let mut table = original.clone();
            for _ in 0..1 + next() % 4 {
                let index = ACPI_HEADER_SIZE + (next() as usize % (table.len() - ACPI_HEADER_SIZE));
                table[index] = next() as u8;
            }
            checksum_byte(&mut table, 9);
            // Reuse one (possibly previously valid) Madt: stale state must not survive.
            let mut madt = Madt::unavailable(MadtStatus::Valid);
            madt.cpu_count = MAX_MADT_CPUS as u8;
            let result = parse_madt(&table, &mut madt);
            assert_eq!(result.is_ok(), madt.status == MadtStatus::Valid);
            match result {
                Ok(()) => {
                    valid += 1;
                    let cpus = madt.cpus();
                    assert!(!cpus.is_empty() && cpus.len() <= MAX_MADT_CPUS);
                    assert!(
                        madt.local_apic_base != 0 && madt.local_apic_base.is_multiple_of(PAGE_SIZE)
                    );
                    for (i, cpu) in cpus.iter().enumerate() {
                        assert!(cpus[i + 1..]
                            .iter()
                            .all(|other| other.apic_id != cpu.apic_id));
                        assert!(cpu.x2apic || cpu.apic_id < 0xff);
                        assert!(!cpu.x2apic || cpu.apic_id != u32::MAX);
                    }
                }
                Err(error) => {
                    invalid += 1;
                    assert!(matches!(
                        error,
                        Error::Madt | Error::Capacity | Error::TableLength
                    ));
                }
            }
        }
        assert!(
            valid > 0 && invalid > 0,
            "mutation corpus must exercise both outcomes"
        );
    }
}
