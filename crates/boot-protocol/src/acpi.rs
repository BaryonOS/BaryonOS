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
    pub register_bytes: u64,
    pub segment: u16,
    pub include_all: bool,
    /// Direct endpoint scopes on bus zero. Bridge paths confer no membership.
    pub direct_bus0: [u64; 4],
    pub unknown_scopes: bool,
}

const EMPTY_MCFG: McfgRegion = McfgRegion {
    base: 0,
    segment: 0,
    start_bus: 0,
    end_bus: 0,
};
const EMPTY_DMAR: DmarUnit = DmarUnit {
    register_base: 0,
    register_bytes: 0,
    segment: 0,
    include_all: false,
    direct_bus0: [0; 4],
    unknown_scopes: false,
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
    pub dmar_host_address_width: u8,
    pub dmar_has_reserved_memory: bool,
    pub dmar_has_other_structures: bool,
    pub madt: Madt,
    /// Physical address of the single FADT (`FACP`) root entry, 0 when absent
    /// or duplicated. Only the header signature is checked here; consumers
    /// must read and validate the table themselves.
    pub fadt: u64,
}

impl Discovery {
    pub const fn empty(availability: Availability) -> Self {
        Self {
            availability,
            mcfg: [EMPTY_MCFG; MAX_MCFG_REGIONS],
            mcfg_count: 0,
            dmar: [EMPTY_DMAR; MAX_DMAR_UNITS],
            dmar_count: 0,
            dmar_host_address_width: 0,
            dmar_has_reserved_memory: false,
            dmar_has_other_structures: false,
            madt: Madt::unavailable(MadtStatus::Missing),
            fadt: 0,
        }
    }

    /// ACPI discovery is only inventory; no IOMMU is enabled by this interface.
    pub const fn dma_execution_allowed(self) -> bool {
        false
    }

    /// Inventory profile for the opt-in, direct bus-zero native DMA fixture.
    /// This is not DMA authority: hardware activation and mapped pinned pages
    /// remain mandatory. QEMU 10 reports explicit scopes, not INCLUDE_PCI_ALL.
    pub fn native_dma_unit(&self, device: u8, function: u8) -> Option<DmarUnit> {
        if self.availability != Availability::DiscoveryComplete
            || self.dmar_count != 1
            || !(39..=52).contains(&self.dmar_host_address_width)
            || self.dmar_has_reserved_memory
            || self.dmar_has_other_structures
            || device > 31
            || function > 7
        {
            return None;
        }
        let unit = self.dmar[0];
        let devfn = usize::from(device) * 8 + usize::from(function);
        if unit.segment != 0
            || unit.unknown_scopes
            || unit.register_base < 0x8000_0000
            || !(PAGE_SIZE..=4 * PAGE_SIZE).contains(&unit.register_bytes)
            || !unit.register_base.is_multiple_of(PAGE_SIZE)
            || !unit.register_bytes.is_power_of_two()
            || unit
                .register_base
                .checked_add(unit.register_bytes)
                .is_none_or(|end| end > (1 << 52))
            || (!unit.include_all && unit.direct_bus0[devfn / 64] & (1 << (devfn % 64)) == 0)
        {
            None
        } else {
            Some(unit)
        }
    }

    /// In-place equivalent of `empty`: every count gates its array, so no
    /// stale entry stays observable and no large temporary is built.
    fn reset(&mut self, availability: Availability) {
        self.availability = availability;
        self.mcfg_count = 0;
        self.dmar_count = 0;
        self.dmar_host_address_width = 0;
        self.dmar_has_reserved_memory = false;
        self.dmar_has_other_structures = false;
        self.madt.status = MadtStatus::Missing;
        self.madt.cpu_count = 0;
        self.madt.online_capable = 0;
        self.madt.disabled = 0;
        self.madt.io_apic_count = 0;
        self.madt.local_apic_base = 0;
        self.madt.pcat_compat = false;
        self.fadt = 0;
    }
}

/// Caller-owned table buffers (8 KiB). The kernel keeps one in static
/// bootstrap memory so discovery never puts them on its 16-KiB stack.
pub struct Scratch {
    root: [u8; MAX_TABLE_SIZE],
    table: [u8; MAX_TABLE_SIZE],
}

impl Scratch {
    pub const fn new() -> Self {
        Self {
            root: [0; MAX_TABLE_SIZE],
            table: [0; MAX_TABLE_SIZE],
        }
    }
}

impl Default for Scratch {
    fn default() -> Self {
        Self::new()
    }
}

/// Reads only the physical spans requested by the parser. Kernel implementations
/// must validate each span against the firmware memory map before mapping it.
pub trait PhysicalRead {
    fn read(&mut self, address: u64, output: &mut [u8]) -> Result<(), Error>;
}

/// Discover bounded MCFG and DMAR descriptors, the MADT processor
/// inventory and the single FADT address. All tables are checksummed, length-limited, and read through
/// `PhysicalRead`; unknown root entries are ignored after validating their
/// address shape. MADT content errors only mark `madt` invalid.
pub fn discover(handoff: &AcpiHandoff, reader: &mut impl PhysicalRead) -> Discovery {
    let mut scratch = Scratch::new();
    let mut result = Discovery::empty(Availability::InvalidTables);
    discover_into(handoff, reader, &mut scratch, &mut result);
    result
}

/// `discover` with caller-provided scratch and result, for small stacks.
pub fn discover_into(
    handoff: &AcpiHandoff,
    reader: &mut impl PhysicalRead,
    scratch: &mut Scratch,
    result: &mut Discovery,
) {
    match handoff.validate() {
        Ok(Some(rsdp)) => {
            result.reset(Availability::InvalidTables);
            if discover_tables(rsdp, reader, scratch, result).is_err() {
                result.reset(Availability::InvalidTables);
            }
        }
        Ok(None) => result.reset(Availability::MissingHandoff),
        Err(_) => result.reset(Availability::RejectedHandoff),
    }
}

fn discover_tables(
    rsdp: &[u8],
    reader: &mut impl PhysicalRead,
    scratch: &mut Scratch,
    result: &mut Discovery,
) -> Result<(), Error> {
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
    let Scratch { root, table } = scratch;
    reader
        .read(root_address, &mut root[..root_length])
        .map_err(|_| Error::Read)?;
    validate_table(&root[..root_length], root_signature)?;

    let mut saw_mcfg = false;
    let mut saw_dmar = false;
    let mut saw_madt = false;
    let mut saw_fadt = false;
    let mut offset = ACPI_HEADER_SIZE;
    while offset < root_length {
        let address = if entry_size == 8 {
            read_u64(&root[..], offset)?
        } else {
            u64::from(read_u32(&root[..], offset)?)
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
            parse_mcfg(&table[..length], result)?;
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
            parse_dmar(&table[..length], result)?;
        } else if signature == b"FACP" {
            // A second FADT is ambiguous: no S5 source, other inventory stays.
            result.fadt = if saw_fadt { 0 } else { address };
            saw_fadt = true;
        } else if signature == b"APIC" {
            // A second MADT is ambiguous: fail SMP closed, keep other inventory.
            // Parsed in place: no Madt temporaries on the 16-KiB kernel stack.
            let parsed = if saw_madt {
                Err(Error::DuplicateTable)
            } else {
                saw_madt = true;
                match table_length(&header) {
                    // Even a rejected read only invalidates the MADT: MCFG/DMAR
                    // discovery must not depend on the optional SMP inventory.
                    Ok(length) => reader
                        .read(address, &mut table[..length])
                        .map_err(|_| Error::Read)
                        .and_then(|()| validate_table(&table[..length], b"APIC"))
                        .and_then(|()| parse_madt(&table[..length], &mut result.madt)),
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
    Ok(())
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
        // INTR_REMAP, X2APIC_OPT_OUT and DMA_CTRL_PLATFORM_OPT_IN; bits 3-7 reserved.
        || table[37] & !7 != 0
        || read_u16(table, 38)? != 0
        || table[40..48] != [0; 8]
    {
        return Err(Error::Dmar);
    }
    result.dmar_host_address_width = table[36] + 1;
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
            // DRHD byte 5: bits 3:0 are the register-set size exponent
            // (2^n pages); bits 7:4 are reserved.
            if length < 16 || structure[4] & !1 != 0 || structure[5] & 0xf0 != 0 {
                return Err(Error::Dmar);
            }
            let aperture = PAGE_SIZE << (structure[5] & 0x0f);
            if result.dmar_count as usize == MAX_DMAR_UNITS {
                return Err(Error::Capacity);
            }
            let register_base = read_u64(structure, 8)?;
            if register_base == 0
                || !register_base.is_multiple_of(PAGE_SIZE)
                || register_base
                    .checked_add(aperture)
                    .is_none_or(|end| end > (1 << 52))
            {
                return Err(Error::Dmar);
            }
            let mut unit = DmarUnit {
                register_base,
                register_bytes: aperture,
                segment: read_u16(structure, 6)?,
                include_all: structure[4] & 1 != 0,
                direct_bus0: [0; 4],
                unknown_scopes: false,
            };
            let mut scope = 16;
            while scope < length {
                let remaining = &structure[scope..];
                if remaining.len() < 8 {
                    return Err(Error::Dmar);
                }
                let size = usize::from(remaining[1]);
                if size < 8 || size > remaining.len() || size % 2 != 0 || remaining[2..4] != [0; 2]
                {
                    return Err(Error::Dmar);
                }
                for path in remaining[6..size].chunks_exact(2) {
                    if path[0] > 31 || path[1] > 7 {
                        return Err(Error::Dmar);
                    }
                }
                match remaining[0] {
                    1 if size == 8 && remaining[5] == 0 => {
                        let devfn = usize::from(remaining[6]) * 8 + usize::from(remaining[7]);
                        unit.direct_bus0[devfn / 64] |= 1 << (devfn % 64);
                    }
                    1..=4 => {} // Other endpoints, bridges, IOAPIC and HPET are inventory only.
                    _ => unit.unknown_scopes = true,
                }
                scope += size;
            }
            result.dmar[result.dmar_count as usize] = unit;
            result.dmar_count += 1;
        } else if kind == 1 {
            result.dmar_has_reserved_memory = true;
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
        } else {
            result.dmar_has_other_structures = true;
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

    /// Owned-result form of `discover_tables` for the RSDP-level tests:
    /// fresh scratch and result per call, so no state carries over.
    fn owned_tables(rsdp: &[u8], reader: &mut impl PhysicalRead) -> Result<Discovery, Error> {
        let mut scratch = Scratch::new();
        let mut result = Discovery::empty(Availability::InvalidTables);
        discover_tables(rsdp, reader, &mut scratch, &mut result).map(|()| result)
    }

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
    fn single_fadt_address_is_recorded_and_duplicates_fail_closed() {
        let (handoff, mut fixture) = valid_fixture(false);
        assert_eq!(discover(&handoff, &mut fixture).fadt, 0);
        let mut xsdt_body = 0x2000u64.to_le_bytes().to_vec();
        xsdt_body.extend_from_slice(&0x4000u64.to_le_bytes());
        fixture.tables[0].1 = table(b"XSDT", &xsdt_body);
        fixture.tables.push((0x4000, table(b"FACP", &[0; 80])));
        let discovered = discover(&handoff, &mut fixture);
        assert_eq!(discovered.fadt, 0x4000);
        assert_eq!(discovered.mcfg_count, 1);
        xsdt_body.extend_from_slice(&0x4000u64.to_le_bytes());
        fixture.tables[0].1 = table(b"XSDT", &xsdt_body);
        let discovered = discover(&handoff, &mut fixture);
        assert_eq!(discovered.fadt, 0);
        assert_eq!(discovered.mcfg_count, 1);
    }

    #[test]
    fn no_dmar_is_an_explicit_unavailable_state() {
        let (handoff, mut fixture) = valid_fixture(false);
        let discovered = discover(&handoff, &mut fixture);
        assert_eq!(discovered.availability, Availability::MissingDmar);
        assert!(!discovered.dma_execution_allowed());
    }

    fn direct_dma_fixture(scopes: &[u8], flags: u8, size: u8) -> Vec<u8> {
        let mut body = vec![0; 12];
        body[0] = 38; // 39-bit QEMU address width.
        let mut unit = vec![0; 16];
        unit[2..4].copy_from_slice(&(16u16 + scopes.len() as u16).to_le_bytes());
        unit[4] = flags;
        unit[5] = size;
        unit[8..16].copy_from_slice(&0xfed9_0000u64.to_le_bytes());
        body.extend_from_slice(&unit);
        body.extend_from_slice(scopes);
        table(b"DMAR", &body)
    }

    #[test]
    fn native_dma_profile_requires_actual_requester_coverage() {
        let mut result = Discovery::empty(Availability::DiscoveryComplete);
        let scope = [1, 8, 0, 0, 0, 0, 5, 0];
        parse_dmar(&direct_dma_fixture(&scope, 0, 0), &mut result).unwrap();
        let unit = result.native_dma_unit(5, 0).unwrap();
        assert_eq!(unit.register_bytes, PAGE_SIZE);
        assert_eq!(unit.register_base, 0xfed9_0000);
        assert_eq!(result.dmar_host_address_width, 39);
        assert!(result.native_dma_unit(5, 1).is_none());
        assert!(result.native_dma_unit(6, 0).is_none());
        assert!(result.native_dma_unit(32, 0).is_none());
        assert!(!result.dma_execution_allowed());

        // An endpoint on another bus, a bridge or a nested path is not the
        // direct bus-zero requester, even when its final device number matches.
        for scope in [
            vec![1, 8, 0, 0, 0, 1, 5, 0],
            vec![2, 8, 0, 0, 0, 0, 5, 0],
            vec![1, 10, 0, 0, 0, 0, 1, 0, 5, 0],
        ] {
            result.reset(Availability::DiscoveryComplete);
            parse_dmar(&direct_dma_fixture(&scope, 0, 0), &mut result).unwrap();
            assert!(result.native_dma_unit(5, 0).is_none());
        }
    }

    #[test]
    fn native_dma_profile_rejects_unsupported_or_ambiguous_inventory() {
        let valid = direct_dma_fixture(&[], 1, 0);
        let mut result = Discovery::empty(Availability::DiscoveryComplete);
        parse_dmar(&valid, &mut result).unwrap();
        assert!(result.native_dma_unit(5, 0).is_some());
        for mutate in [
            |d: &mut Discovery| d.dmar_has_reserved_memory = true,
            |d: &mut Discovery| d.dmar_has_other_structures = true,
            |d: &mut Discovery| d.dmar_count = 2,
            |d: &mut Discovery| d.dmar[0].segment = 1,
            |d: &mut Discovery| d.dmar[0].register_bytes = 8 * PAGE_SIZE,
            |d: &mut Discovery| d.dmar[0].register_base = 0x1000,
            |d: &mut Discovery| d.dmar_host_address_width = 38,
            |d: &mut Discovery| d.dmar_host_address_width = 53,
            |d: &mut Discovery| d.availability = Availability::InvalidTables,
        ] {
            let mut changed = result;
            mutate(&mut changed);
            assert!(changed.native_dma_unit(5, 0).is_none());
        }
        result.reset(Availability::DiscoveryComplete);
        parse_dmar(
            &direct_dma_fixture(&[9, 8, 0, 0, 0, 0, 5, 0], 1, 0),
            &mut result,
        )
        .unwrap();
        assert!(result.native_dma_unit(5, 0).is_none());
        result.reset(Availability::MissingHandoff);
        assert_eq!(result.dmar_host_address_width, 0);
        assert!(!result.dmar_has_reserved_memory);
        assert!(!result.dmar_has_other_structures);
        assert!(result.native_dma_unit(5, 0).is_none());
    }

    #[test]
    fn malformed_dmar_scope_never_establishes_requester_coverage() {
        for scope in [
            vec![1, 7, 0, 0, 0, 0, 5],
            vec![1, 10, 0, 0, 0, 0, 5, 0],
            vec![1, 8, 1, 0, 0, 0, 5, 0],
            vec![1, 8, 0, 0, 0, 0, 32, 0],
            vec![1, 8, 0, 0, 0, 0, 5, 8],
        ] {
            let mut result = Discovery::empty(Availability::DiscoveryComplete);
            assert_eq!(
                parse_dmar(&direct_dma_fixture(&scope, 0, 0), &mut result),
                Err(Error::Dmar)
            );
            assert!(result.native_dma_unit(5, 0).is_none());
        }
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
            owned_tables(&rsdp, &mut fixture).err(),
            Some(Error::RootShape)
        );

        let (_, mut fixture) = valid_fixture(true);
        fixture.tables[1].1[4..8].copy_from_slice(&35u32.to_le_bytes());
        assert_eq!(
            owned_tables(&rsdp, &mut fixture).err(),
            Some(Error::TableLength)
        );

        let (_, mut fixture) = valid_fixture(false);
        fixture.tables[0]
            .1
            .extend_from_slice(&0x2000u64.to_le_bytes());
        set_length_and_checksum(&mut fixture.tables[0].1);
        assert_eq!(
            owned_tables(&rsdp, &mut fixture).err(),
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
        let discovered = owned_tables(&rsdp, &mut fixture).unwrap();
        assert_eq!(discovered.availability, Availability::MissingDmar);
        assert_eq!(discovered.mcfg_count, 1);

        fixture.tables[0].1.extend_from_slice(&[0; 2]);
        set_length_and_checksum(&mut fixture.tables[0].1);
        assert_eq!(
            owned_tables(&rsdp, &mut fixture).err(),
            Some(Error::RootShape)
        );
    }

    #[test]
    fn accepts_valid_dmar_flags_and_drhd_size_but_rejects_reserved_bits() {
        let rsdp = valid_fixture(true).0.validate().unwrap().unwrap().to_vec();
        let with = |patch: &dyn Fn(&mut Vec<u8>)| {
            let (_, mut fixture) = valid_fixture(true);
            patch(&mut fixture.tables[2].1);
            checksum_byte(&mut fixture.tables[2].1, 9);
            owned_tables(&rsdp, &mut fixture)
        };
        // DMA_CTRL_PLATFORM_OPT_IN (bit 2) and a 2^3-page register set are valid.
        let ok = with(&|t| {
            t[37] = 0b111;
            t[53] = 3;
        })
        .unwrap();
        assert_eq!(ok.availability, Availability::DiscoveryComplete);
        assert_eq!(with(&|t| t[37] = 0b1000).err(), Some(Error::Dmar));
        assert_eq!(with(&|t| t[53] = 0x10).err(), Some(Error::Dmar));
        // The whole aperture, not only its first page, must stay below 2^52.
        assert_eq!(
            with(&|t| {
                t[53] = 15;
                t[56..64].copy_from_slice(&((1u64 << 52) - 0x8000).to_le_bytes());
            })
            .err(),
            Some(Error::Dmar)
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
    fn reused_result_never_keeps_a_stale_inventory() {
        // The kernel reuses one static Discovery and Scratch.
        let mut scratch = Scratch::new();
        let mut result = Discovery::empty(Availability::MissingHandoff);
        let (handoff, mut fixture) = madt_fixture(vec![madt(0xfee0_0000, &qemu_like_entries())]);
        discover_into(&handoff, &mut fixture, &mut scratch, &mut result);
        assert_eq!(result.madt.status, MadtStatus::Valid);
        assert_eq!(result.mcfg_count, 1);
        assert_eq!(result, discover(&handoff, &mut fixture));
        for (next, availability) in [
            (AcpiHandoff::rejected(), Availability::RejectedHandoff),
            (AcpiHandoff::unavailable(), Availability::MissingHandoff),
        ] {
            discover_into(&next, &mut fixture, &mut scratch, &mut result);
            assert_eq!(result.availability, availability);
            assert_eq!((result.mcfg_count, result.dmar_count), (0, 0));
            assert_eq!(result.madt.status, MadtStatus::Missing);
            assert!(result.madt.cpus().is_empty());
            discover_into(&handoff, &mut fixture, &mut scratch, &mut result);
        }
        fixture.tables[0].1[9] ^= 1; // corrupt XSDT checksum
        discover_into(&handoff, &mut fixture, &mut scratch, &mut result);
        assert_eq!(result.availability, Availability::InvalidTables);
        assert_eq!(result.mcfg_count, 0);
        assert!(result.madt.cpus().is_empty());
    }

    #[test]
    fn unreadable_madt_body_fails_only_smp_closed() {
        // Header readable, body outside what the reader accepts.
        let mut header_only = madt(0xfee0_0000, &qemu_like_entries());
        header_only.truncate(ACPI_HEADER_SIZE);
        assert_only_madt_invalid(discover_madt(vec![header_only]));
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
