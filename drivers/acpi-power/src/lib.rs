#![no_std]

//! Pure, bounded extraction of the ACPI S5 sleep settings.
//!
//! This crate validates the supplied FADT and DSDT byte snapshots itself. It
//! does not discover/map firmware tables, interpret AML methods, authorize a
//! shutdown request, or perform hardware I/O.

const SDT_HEADER_LEN: usize = 36;
const FADT_MIN_LEN: usize = 116;
const MAX_TABLE_LEN: usize = 1024 * 1024;
const MAX_AML_TERMS: usize = 65_536;
const PM1_CNT_LEN_OFFSET: usize = 89;
const FADT_FLAGS_OFFSET: usize = 112;
const FADT_DSDT_OFFSET: usize = 40;
const FADT_X_DSDT_OFFSET: usize = 140;
const FADT_PM1A_CNT_OFFSET: usize = 64;
const FADT_PM1B_CNT_OFFSET: usize = 68;
const FADT_X_PM1A_CNT_OFFSET: usize = 172;
const FADT_X_PM1B_CNT_OFFSET: usize = 184;
const FADT_HW_REDUCED_ACPI: u32 = 1 << 20;
const AML_NAME_OP: u8 = 0x08;
const AML_SCOPE_OP: u8 = 0x10;
const AML_METHOD_OP: u8 = 0x14;
const AML_EXT_OP: u8 = 0x5b;
const AML_PACKAGE_OP: u8 = 0x12;
const AML_VAR_PACKAGE_OP: u8 = 0x13;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    TableTooShort,
    TableTooLarge,
    TableLengthMismatch,
    InvalidSignature,
    InvalidChecksum,
    InvalidFadtLength,
    UnsupportedHardwareReducedAcpi,
    MissingDsdtAddress,
    InvalidTableSourceAddress,
    DsdtAddressMismatch,
    InvalidPm1Length,
    MissingPm1aRegister,
    InvalidGas,
    InvalidRegisterAddress,
    AliasedRegisters,
    MalformedAml,
    UnsupportedAml,
    MissingS5Object,
    DuplicateS5Object,
    InvalidS5Package,
    SleepTypeOutOfRange,
}

/// One PM1 control-register setting extracted from the validated tables.
///
/// The fields and constructor are private. This value contains parsed data,
/// not authority to perform I/O; it must never be accepted from an IPC caller
/// as a shutdown command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pm1Setting {
    address: u16,
    sleep_type: u8,
}

impl Pm1Setting {
    pub const fn address(self) -> u16 {
        self.address
    }

    pub const fn sleep_type(self) -> u8 {
        self.sleep_type
    }

    /// Returns the masked register value after a caller has read the current
    /// register value. No register access is performed by this crate.
    pub const fn apply_to(self, current: u16) -> u16 {
        let fields = (0b111 << 10) | (1 << 13);
        (current & !fields) | ((self.sleep_type as u16) << 10) | (1 << 13)
    }
}

/// Opaque S5 settings derived only from a validated FADT and its DSDT.
///
/// The addresses are read-only observations, not a generic port-I/O API.
/// Kernel integration must independently validate its ACPI snapshot and must
/// not accept this structure, or its fields, from an untrusted process.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct S5Plan {
    pm1a: Pm1Setting,
    pm1b: Option<Pm1Setting>,
}

impl S5Plan {
    pub const fn pm1a(self) -> Pm1Setting {
        self.pm1a
    }

    pub const fn pm1b(self) -> Option<Pm1Setting> {
        self.pm1b
    }
}

/// One selected table from the bounded, kernel-owned ACPI snapshot.
///
/// The metadata is not an authorization token: callers must obtain it from the
/// kernel snapshot handoff, and the kernel must validate the same ranges
/// independently. This crate still verifies the bytes and FADT/DSDT binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TableSnapshot<'a> {
    bytes: &'a [u8],
    source_address: u64,
}

impl<'a> TableSnapshot<'a> {
    pub const fn new(bytes: &'a [u8], source_address: u64) -> Self {
        Self {
            bytes,
            source_address,
        }
    }
}

/// The selected FADT and DSDT extents from one versioned kernel snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcpiTableSnapshot<'a> {
    fadt: TableSnapshot<'a>,
    dsdt: TableSnapshot<'a>,
}

impl<'a> AcpiTableSnapshot<'a> {
    pub const fn new(fadt: TableSnapshot<'a>, dsdt: TableSnapshot<'a>) -> Self {
        Self { fadt, dsdt }
    }
}

/// Validates raw SDT snapshots and extracts the legacy x86 S5 settings.
///
/// The source addresses in `tables` must originate in the trusted, bounded
/// kernel snapshot handoff, not caller-chosen IPC fields. The table slices
/// must contain exactly the respective SDT-declared lengths.
pub fn parse_s5(tables: AcpiTableSnapshot<'_>) -> Result<S5Plan, Error> {
    let fadt = tables.fadt.bytes;
    let dsdt = tables.dsdt.bytes;
    validate_sdt(fadt, *b"FACP")?;
    validate_sdt(dsdt, *b"DSDT")?;
    if tables.fadt.source_address == 0 {
        return Err(Error::InvalidTableSourceAddress);
    }
    if fadt.len() < FADT_MIN_LEN {
        return Err(Error::InvalidFadtLength);
    }

    let declared_dsdt_address = fadt_dsdt_address(fadt)?;
    if tables.dsdt.source_address == 0 || declared_dsdt_address != tables.dsdt.source_address {
        return Err(Error::DsdtAddressMismatch);
    }

    let flags = read_u32(fadt, FADT_FLAGS_OFFSET).ok_or(Error::InvalidFadtLength)?;
    if flags & FADT_HW_REDUCED_ACPI != 0 {
        return Err(Error::UnsupportedHardwareReducedAcpi);
    }

    let pm1_length = *fadt
        .get(PM1_CNT_LEN_OFFSET)
        .ok_or(Error::InvalidFadtLength)?;
    if pm1_length < 2 {
        return Err(Error::InvalidPm1Length);
    }

    let pm1a_address = pm1_register_address(
        fadt,
        FADT_PM1A_CNT_OFFSET,
        FADT_X_PM1A_CNT_OFFSET,
        pm1_length,
    )?
    .ok_or(Error::MissingPm1aRegister)?;
    let pm1b_address = pm1_register_address(
        fadt,
        FADT_PM1B_CNT_OFFSET,
        FADT_X_PM1B_CNT_OFFSET,
        pm1_length,
    )?;
    if let Some(pm1b) = pm1b_address {
        if ranges_overlap(pm1a_address, pm1b) {
            return Err(Error::AliasedRegisters);
        }
    }

    let (sleep_type_a, sleep_type_b) = parse_aml_s5(&dsdt[SDT_HEADER_LEN..])?;
    Ok(S5Plan {
        pm1a: Pm1Setting {
            address: pm1a_address,
            sleep_type: sleep_type_a,
        },
        pm1b: pm1b_address.map(|address| Pm1Setting {
            address,
            sleep_type: sleep_type_b,
        }),
    })
}

fn validate_sdt(bytes: &[u8], signature: [u8; 4]) -> Result<(), Error> {
    if bytes.len() < SDT_HEADER_LEN {
        return Err(Error::TableTooShort);
    }
    if bytes.len() > MAX_TABLE_LEN {
        return Err(Error::TableTooLarge);
    }
    if bytes.get(..4) != Some(signature.as_slice()) {
        return Err(Error::InvalidSignature);
    }
    let declared = read_u32(bytes, 4).ok_or(Error::TableTooShort)? as usize;
    if declared != bytes.len() {
        return Err(Error::TableLengthMismatch);
    }
    if bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)) != 0 {
        return Err(Error::InvalidChecksum);
    }
    Ok(())
}

fn fadt_dsdt_address(fadt: &[u8]) -> Result<u64, Error> {
    if fadt.len() > FADT_X_DSDT_OFFSET && fadt.len() < FADT_X_DSDT_OFFSET + 8 {
        return Err(Error::InvalidFadtLength);
    }
    if fadt.len() >= FADT_X_DSDT_OFFSET + 8 {
        let extended = read_u64(fadt, FADT_X_DSDT_OFFSET).ok_or(Error::InvalidFadtLength)?;
        if extended != 0 {
            return Ok(extended);
        }
    }
    let legacy = read_u32(fadt, FADT_DSDT_OFFSET).ok_or(Error::InvalidFadtLength)?;
    if legacy == 0 {
        return Err(Error::MissingDsdtAddress);
    }
    Ok(u64::from(legacy))
}

fn pm1_register_address(
    fadt: &[u8],
    legacy_offset: usize,
    gas_offset: usize,
    block_length: u8,
) -> Result<Option<u16>, Error> {
    if fadt.len() > gas_offset && fadt.len() < gas_offset + 12 {
        return Err(Error::InvalidFadtLength);
    }
    let legacy = read_u32(fadt, legacy_offset).ok_or(Error::InvalidFadtLength)?;

    let address = if fadt.len() >= gas_offset + 12 {
        let gas = &fadt[gas_offset..gas_offset + 12];
        let extended_address = read_u64(gas, 4).ok_or(Error::InvalidGas)?;
        if extended_address == 0 {
            u64::from(legacy)
        } else {
            validate_pm1_gas(gas, block_length)?;
            extended_address
        }
    } else {
        u64::from(legacy)
    };

    if address == 0 {
        return Ok(None);
    }
    validate_pm1_address(address, block_length).map(Some)
}

fn validate_pm1_gas(gas: &[u8], block_length: u8) -> Result<(), Error> {
    let space_id = *gas.first().ok_or(Error::InvalidGas)?;
    let bit_width = *gas.get(1).ok_or(Error::InvalidGas)?;
    let bit_offset = *gas.get(2).ok_or(Error::InvalidGas)?;
    let access_size = *gas.get(3).ok_or(Error::InvalidGas)?;
    if space_id != 1
        || bit_width != 16
        || bit_offset != 0
        || !(access_size == 0 || access_size == 2)
        || block_length < 2
    {
        return Err(Error::InvalidGas);
    }
    Ok(())
}

fn validate_pm1_address(address: u64, block_length: u8) -> Result<u16, Error> {
    if block_length < 2 {
        return Err(Error::InvalidPm1Length);
    }
    let last = address
        .checked_add(1)
        .ok_or(Error::InvalidRegisterAddress)?;
    if address == 0 || last > u64::from(u16::MAX) {
        return Err(Error::InvalidRegisterAddress);
    }
    Ok(address as u16)
}

fn ranges_overlap(a: u16, b: u16) -> bool {
    let a_start = u32::from(a);
    let b_start = u32::from(b);
    a_start < b_start + 2 && b_start < a_start + 2
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let field = bytes.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes([field[0], field[1], field[2], field[3]]))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    let field = bytes.get(offset..offset.checked_add(8)?)?;
    Some(u64::from_le_bytes([
        field[0], field[1], field[2], field[3], field[4], field[5], field[6], field[7],
    ]))
}

fn parse_aml_s5(aml: &[u8]) -> Result<(u8, u8), Error> {
    let mut cursor = 0usize;
    let mut terms = 0usize;
    let mut found = None;
    while cursor < aml.len() {
        terms += 1;
        if terms > MAX_AML_TERMS {
            return Err(Error::MalformedAml);
        }
        match aml[cursor] {
            AML_NAME_OP => {
                cursor += 1;
                let (name, name_len, is_root_s5) = parse_name_string(&aml[cursor..])?;
                cursor = cursor.checked_add(name_len).ok_or(Error::MalformedAml)?;
                if is_root_s5 {
                    if found.is_some() {
                        return Err(Error::DuplicateS5Object);
                    }
                    let (types, object_len) = parse_s5_package(&aml[cursor..])?;
                    cursor = cursor.checked_add(object_len).ok_or(Error::MalformedAml)?;
                    found = Some(types);
                } else {
                    let object_len = skip_data_object(&aml[cursor..])?;
                    cursor = cursor.checked_add(object_len).ok_or(Error::MalformedAml)?;
                }
                let _ = name;
            }
            AML_SCOPE_OP | AML_METHOD_OP => {
                let kind = if aml[cursor] == AML_SCOPE_OP {
                    PackageHeader::Name
                } else {
                    PackageHeader::NameAndFlags
                };
                cursor = skip_namespace_package(aml, cursor + 1, kind)?;
            }
            AML_EXT_OP => {
                let opcode = *aml.get(cursor + 1).ok_or(Error::MalformedAml)?;
                match opcode {
                    // AML namespace containers are skipped as bounded packages:
                    // bytes inside another scope/method cannot impersonate the
                    // root-level _S5 declaration.
                    0x81..=0x87 => {
                        let kind = match opcode {
                            0x81 => PackageHeader::NameAndFlags,
                            0x82 | 0x85 => PackageHeader::Name,
                            0x83 => PackageHeader::NameAndFixedBytes(6),
                            0x84 => PackageHeader::NameAndFixedBytes(3),
                            0x86 => PackageHeader::TwoNamesAndFlags,
                            0x87 => PackageHeader::TwoNamesTermArgAndFlags,
                            _ => return Err(Error::UnsupportedAml),
                        };
                        cursor = skip_namespace_package(aml, cursor + 2, kind)?;
                    }
                    // Mutex and Event have a NameString and no package body.
                    0x01 | 0x02 => {
                        let (_, name_len, _) =
                            parse_name_string(aml.get(cursor + 2..).ok_or(Error::MalformedAml)?)?;
                        cursor = cursor
                            .checked_add(2 + name_len)
                            .ok_or(Error::MalformedAml)?;
                        if opcode == 0x01 {
                            cursor = cursor.checked_add(1).ok_or(Error::MalformedAml)?;
                        }
                        if cursor > aml.len() {
                            return Err(Error::MalformedAml);
                        }
                    }
                    // OperationRegion is name, space ID, and two TermArgs.
                    0x80 => {
                        let (_, name_len, _) = parse_name_string(&aml[cursor + 2..])?;
                        cursor = cursor
                            .checked_add(2 + name_len)
                            .ok_or(Error::MalformedAml)?;
                        cursor = cursor.checked_add(1).ok_or(Error::MalformedAml)?;
                        if cursor > aml.len() {
                            return Err(Error::MalformedAml);
                        }
                        for _ in 0..2 {
                            let term_len = skip_term_arg(&aml[cursor..])?;
                            cursor = cursor.checked_add(term_len).ok_or(Error::MalformedAml)?;
                        }
                    }
                    _ => return Err(Error::UnsupportedAml),
                }
            }
            0x06 => {
                // AliasOp: SourceObject followed by AliasObject.
                cursor += 1;
                for _ in 0..2 {
                    let (_, name_len, _) = parse_name_string(&aml[cursor..])?;
                    cursor = cursor.checked_add(name_len).ok_or(Error::MalformedAml)?;
                }
            }
            0x15 => {
                // ExternalOp: NameString, ObjectType, and ArgumentCount.
                cursor += 1;
                let (_, name_len, _) = parse_name_string(&aml[cursor..])?;
                cursor = cursor
                    .checked_add(name_len + 2)
                    .ok_or(Error::MalformedAml)?;
                if cursor > aml.len() {
                    return Err(Error::MalformedAml);
                }
            }
            0xa3 => cursor += 1, // NoopOp
            _ => return Err(Error::UnsupportedAml),
        }
    }
    found.ok_or(Error::MissingS5Object)
}

fn parse_name_string(bytes: &[u8]) -> Result<(&[u8], usize, bool), Error> {
    let mut cursor = 0usize;
    let mut root = false;
    let mut parents = 0usize;
    if bytes.get(cursor) == Some(&0x5c) {
        root = true;
        cursor += 1;
    } else {
        while bytes.get(cursor) == Some(&0x5e) {
            parents += 1;
            cursor += 1;
        }
    }
    let prefix_end = cursor;
    let count = match bytes.get(cursor).copied().ok_or(Error::MalformedAml)? {
        0x00 => return Ok((&[], cursor + 1, false)),
        0x2e => {
            cursor += 1;
            2
        }
        0x2f => {
            let count = usize::from(*bytes.get(cursor + 1).ok_or(Error::MalformedAml)?);
            cursor += 2;
            if count == 0 {
                return Err(Error::MalformedAml);
            }
            count
        }
        _ => 1,
    };
    let names_len = count.checked_mul(4).ok_or(Error::MalformedAml)?;
    let end = cursor.checked_add(names_len).ok_or(Error::MalformedAml)?;
    let segments = bytes.get(cursor..end).ok_or(Error::MalformedAml)?;
    for segment in segments.chunks_exact(4) {
        if !valid_name_seg(segment) {
            return Err(Error::MalformedAml);
        }
    }
    let one_root_segment =
        count == 1 && parents == 0 && (root || prefix_end == 0) && segments == b"_S5_";
    Ok((segments, end, one_root_segment))
}

fn valid_name_seg(segment: &[u8]) -> bool {
    segment.len() == 4
        && (segment[0] == b'_' || segment[0].is_ascii_uppercase())
        && segment[1..]
            .iter()
            .all(|byte| *byte == b'_' || byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

fn parse_s5_package(bytes: &[u8]) -> Result<((u8, u8), usize), Error> {
    if bytes.first() != Some(&AML_PACKAGE_OP) {
        return Err(Error::InvalidS5Package);
    }
    let (end, body_start) = package_bounds(bytes, 1)?;
    let count = *bytes.get(body_start).ok_or(Error::MalformedAml)?;
    if count != 2 {
        return Err(Error::InvalidS5Package);
    }
    let mut cursor = body_start + 1;
    let first = parse_integer(bytes.get(cursor..end).ok_or(Error::MalformedAml)?)?;
    cursor += first.1;
    let second = parse_integer(bytes.get(cursor..end).ok_or(Error::MalformedAml)?)?;
    cursor += second.1;
    if cursor != end {
        return Err(Error::InvalidS5Package);
    }
    if first.0 > 7 || second.0 > 7 {
        return Err(Error::SleepTypeOutOfRange);
    }
    Ok(((first.0 as u8, second.0 as u8), end))
}

fn parse_integer(bytes: &[u8]) -> Result<(u64, usize), Error> {
    let opcode = *bytes.first().ok_or(Error::MalformedAml)?;
    match opcode {
        0x00 => Ok((0, 1)),        // ZeroOp
        0x01 => Ok((1, 1)),        // OneOp
        0xff => Ok((u64::MAX, 1)), // OnesOp
        0x0a => Ok((u64::from(*bytes.get(1).ok_or(Error::MalformedAml)?), 2)),
        0x0b => Ok((u64::from(read_u16(bytes, 1).ok_or(Error::MalformedAml)?), 3)),
        0x0c => Ok((u64::from(read_u32(bytes, 1).ok_or(Error::MalformedAml)?), 5)),
        0x0e => Ok((read_u64(bytes, 1).ok_or(Error::MalformedAml)?, 9)),
        _ => Err(Error::InvalidS5Package),
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let field = bytes.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes([field[0], field[1]]))
}

fn skip_data_object(bytes: &[u8]) -> Result<usize, Error> {
    let opcode = *bytes.first().ok_or(Error::MalformedAml)?;
    match opcode {
        0x00 | 0x01 | 0xff => Ok(1),
        0x0a => bytes.get(1).map(|_| 2).ok_or(Error::MalformedAml),
        0x0b => bytes.get(1..3).map(|_| 3).ok_or(Error::MalformedAml),
        0x0c => bytes.get(1..5).map(|_| 5).ok_or(Error::MalformedAml),
        0x0e => bytes.get(1..9).map(|_| 9).ok_or(Error::MalformedAml),
        0x0d => {
            let terminator = bytes[1..]
                .iter()
                .position(|byte| *byte == 0)
                .ok_or(Error::MalformedAml)?;
            Ok(terminator + 2)
        }
        0x11 | AML_PACKAGE_OP | AML_VAR_PACKAGE_OP => {
            let (end, _) = package_bounds(bytes, 1)?;
            Ok(end)
        }
        _ => {
            let (_, name_len, _) = parse_name_string(bytes)?;
            Ok(name_len)
        }
    }
}

fn skip_term_arg(bytes: &[u8]) -> Result<usize, Error> {
    // The FADT-related DSDTs used by this narrow parser express OperationRegion
    // addresses/lengths as integer literals or NameStrings.
    skip_data_object(bytes)
}

#[derive(Clone, Copy)]
enum PackageHeader {
    Name,
    NameAndFlags,
    NameAndFixedBytes(usize),
    TwoNamesAndFlags,
    TwoNamesTermArgAndFlags,
}

fn skip_namespace_package(
    bytes: &[u8],
    pkg_length_offset: usize,
    header: PackageHeader,
) -> Result<usize, Error> {
    let (end, mut cursor) = package_bounds(bytes, pkg_length_offset)?;
    let (_, first_len, _) = parse_name_string(bytes.get(cursor..end).ok_or(Error::MalformedAml)?)?;
    if first_len == 1 && bytes[cursor] == 0 {
        return Err(Error::MalformedAml);
    }
    cursor = cursor.checked_add(first_len).ok_or(Error::MalformedAml)?;
    match header {
        PackageHeader::Name => {}
        PackageHeader::NameAndFlags => {
            cursor = cursor.checked_add(1).ok_or(Error::MalformedAml)?;
        }
        PackageHeader::NameAndFixedBytes(count) => {
            cursor = cursor.checked_add(count).ok_or(Error::MalformedAml)?;
        }
        PackageHeader::TwoNamesAndFlags => {
            cursor = skip_nonempty_name(bytes, cursor, end)?;
            cursor = cursor.checked_add(1).ok_or(Error::MalformedAml)?;
        }
        PackageHeader::TwoNamesTermArgAndFlags => {
            cursor = skip_nonempty_name(bytes, cursor, end)?;
            let term_len = skip_term_arg(bytes.get(cursor..end).ok_or(Error::MalformedAml)?)?;
            cursor = cursor
                .checked_add(term_len)
                .and_then(|value| value.checked_add(1))
                .ok_or(Error::MalformedAml)?;
        }
    }
    if cursor > end {
        return Err(Error::MalformedAml);
    }
    Ok(end)
}

fn skip_nonempty_name(bytes: &[u8], cursor: usize, end: usize) -> Result<usize, Error> {
    let (name, name_len, _) =
        parse_name_string(bytes.get(cursor..end).ok_or(Error::MalformedAml)?)?;
    if name.is_empty() {
        return Err(Error::MalformedAml);
    }
    cursor.checked_add(name_len).ok_or(Error::MalformedAml)
}

fn package_bounds(bytes: &[u8], pkg_length_offset: usize) -> Result<(usize, usize), Error> {
    let lead = *bytes.get(pkg_length_offset).ok_or(Error::MalformedAml)?;
    let follow = usize::from((lead >> 6) & 0x03);
    if follow != 0 && lead & 0x30 != 0 {
        return Err(Error::MalformedAml);
    }
    let encoded_len = 1usize + follow;
    let mut package_len = if follow == 0 {
        usize::from(lead & 0x3f)
    } else {
        usize::from(lead & 0x0f)
    };
    for index in 0..follow {
        let byte = usize::from(
            *bytes
                .get(pkg_length_offset + 1 + index)
                .ok_or(Error::MalformedAml)?,
        );
        package_len |= byte << (4 + 8 * index);
    }
    if package_len < encoded_len {
        return Err(Error::MalformedAml);
    }
    let end = pkg_length_offset
        .checked_add(package_len)
        .ok_or(Error::MalformedAml)?;
    let body_start = pkg_length_offset
        .checked_add(encoded_len)
        .ok_or(Error::MalformedAml)?;
    if end > bytes.len() || body_start > end {
        return Err(Error::MalformedAml);
    }
    Ok((end, body_start))
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::{parse_s5, AcpiTableSnapshot, Error, S5Plan, TableSnapshot};
    use std::vec;
    use std::vec::Vec;

    const DSDT_ADDRESS: u64 = 0x1234_5000;

    fn finish_table(mut bytes: Vec<u8>, signature: &[u8; 4]) -> Vec<u8> {
        bytes[..4].copy_from_slice(signature);
        let len = bytes.len() as u32;
        bytes[4..8].copy_from_slice(&len.to_le_bytes());
        bytes[9] = 0;
        let checksum = bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte));
        bytes[9] = 0u8.wrapping_sub(checksum);
        bytes
    }

    fn dsdt(aml: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0; 36];
        bytes.extend_from_slice(aml);
        finish_table(bytes, b"DSDT")
    }

    fn fadt(pm1a: u32, pm1b: u32) -> Vec<u8> {
        let mut bytes = vec![0; 116];
        bytes[40..44].copy_from_slice(&(DSDT_ADDRESS as u32).to_le_bytes());
        bytes[64..68].copy_from_slice(&pm1a.to_le_bytes());
        bytes[68..72].copy_from_slice(&pm1b.to_le_bytes());
        bytes[89] = 2;
        finish_table(bytes, b"FACP")
    }

    fn package(bytes: &[u8]) -> Vec<u8> {
        assert!(bytes.len() < 0x3f);
        let mut out = vec![0x12, (bytes.len() + 1) as u8];
        out.extend_from_slice(bytes);
        out
    }

    fn integer(value: u8) -> Vec<u8> {
        match value {
            0 => vec![0x00],
            1 => vec![0x01],
            value => vec![0x0a, value],
        }
    }

    fn root_s5(first: u8, second: u8) -> Vec<u8> {
        let first = integer(first);
        let second = integer(second);
        let mut package_body = vec![2];
        package_body.extend_from_slice(&first);
        package_body.extend_from_slice(&second);
        let mut aml = vec![0x08];
        aml.extend_from_slice(b"_S5_");
        aml.extend_from_slice(&package(&package_body));
        aml
    }

    fn parse(fadt_bytes: &[u8], dsdt_bytes: &[u8]) -> Result<S5Plan, Error> {
        parse_s5(AcpiTableSnapshot::new(
            TableSnapshot::new(fadt_bytes, 0x1234_4000),
            TableSnapshot::new(dsdt_bytes, DSDT_ADDRESS),
        ))
    }

    #[test]
    fn extracts_root_s5_and_creates_read_only_register_settings() {
        let plan = parse(&fadt(0x404, 0x408), &dsdt(&root_s5(0, 1))).unwrap();
        assert_eq!(plan.pm1a().address(), 0x404);
        assert_eq!(plan.pm1a().sleep_type(), 0);
        assert_eq!(plan.pm1a().apply_to(0xffff), 0xe3ff);
        let pm1b = plan.pm1b().unwrap();
        assert_eq!(pm1b.address(), 0x408);
        assert_eq!(pm1b.sleep_type(), 1);
    }

    #[test]
    fn accepts_the_legacy_pm1a_register_without_pm1b() {
        let plan = parse(&fadt(0x404, 0), &dsdt(&root_s5(5, 6))).unwrap();
        assert_eq!(plan.pm1a().sleep_type(), 5);
        assert_eq!(plan.pm1b(), None);
    }

    #[test]
    fn validates_sdt_signatures_lengths_and_checksums() {
        let valid_fadt = fadt(0x404, 0);
        let valid_dsdt = dsdt(&root_s5(0, 0));
        let mut bad = valid_fadt.clone();
        bad[0] = b'X';
        assert_eq!(parse(&bad, &valid_dsdt), Err(Error::InvalidSignature));
        let mut bad = valid_dsdt.clone();
        bad[20] ^= 1;
        assert_eq!(parse(&valid_fadt, &bad), Err(Error::InvalidChecksum));
        let mut bad = valid_fadt.clone();
        let declared_len = bad.len() as u32 + 1;
        bad[4..8].copy_from_slice(&declared_len.to_le_bytes());
        assert_eq!(parse(&bad, &valid_dsdt), Err(Error::TableLengthMismatch));
        let mut bad = valid_dsdt.clone();
        bad.push(0);
        assert_eq!(parse(&valid_fadt, &bad), Err(Error::TableLengthMismatch));
    }

    #[test]
    fn rejects_wrong_dsdt_address_and_missing_aml_s5() {
        assert_eq!(
            parse_s5(AcpiTableSnapshot::new(
                TableSnapshot::new(&fadt(0x404, 0), 0x1234_4000),
                TableSnapshot::new(&dsdt(&root_s5(0, 0)), DSDT_ADDRESS + 1),
            )),
            Err(Error::DsdtAddressMismatch)
        );
        assert_eq!(
            parse_s5(AcpiTableSnapshot::new(
                TableSnapshot::new(&fadt(0x404, 0), 0),
                TableSnapshot::new(&dsdt(&root_s5(0, 0)), DSDT_ADDRESS),
            )),
            Err(Error::InvalidTableSourceAddress)
        );
        assert_eq!(
            parse(&fadt(0x404, 0), &dsdt(&[])),
            Err(Error::MissingS5Object)
        );
    }

    #[test]
    fn rejects_duplicate_or_non_root_s5_declarations() {
        let mut duplicate = root_s5(0, 0);
        duplicate.extend_from_slice(&root_s5(1, 1));
        assert_eq!(
            parse(&fadt(0x404, 0), &dsdt(&duplicate)),
            Err(Error::DuplicateS5Object)
        );

        // A bounded Scope package containing bytes that look like _S5 is not a
        // root NameOp and must not be discovered by searching for byte strings.
        let nested = root_s5(0, 0);
        let mut scoped = vec![0x10, (nested.len() + 5) as u8];
        scoped.extend_from_slice(b"_SB_");
        scoped.extend_from_slice(&nested);
        assert_eq!(
            parse(&fadt(0x404, 0), &dsdt(&scoped)),
            Err(Error::MissingS5Object)
        );
    }

    #[test]
    fn rejects_malformed_and_ambiguous_s5_packages() {
        let mut wrong_count = vec![0x08];
        wrong_count.extend_from_slice(b"_S5_");
        wrong_count.extend_from_slice(&package(&[3, 0, 0, 0]));
        assert_eq!(
            parse(&fadt(0x404, 0), &dsdt(&wrong_count)),
            Err(Error::InvalidS5Package)
        );

        let mut truncated = vec![0x08];
        truncated.extend_from_slice(b"_S5_");
        truncated.extend_from_slice(&[0x12, 0x3f, 2, 0, 0]);
        assert_eq!(
            parse(&fadt(0x404, 0), &dsdt(&truncated)),
            Err(Error::MalformedAml)
        );

        let mut extra = vec![0x08];
        extra.extend_from_slice(b"_S5_");
        extra.extend_from_slice(&package(&[2, 0, 0, 0]));
        assert_eq!(
            parse(&fadt(0x404, 0), &dsdt(&extra)),
            Err(Error::InvalidS5Package)
        );
    }

    #[test]
    fn rejects_unsupported_aml_and_sleep_types_outside_three_bits() {
        assert_eq!(
            parse(&fadt(0x404, 0), &dsdt(&[0x99])),
            Err(Error::UnsupportedAml)
        );
        assert_eq!(
            parse(&fadt(0x404, 0), &dsdt(&root_s5(0x0a, 0))),
            Err(Error::SleepTypeOutOfRange)
        );
    }

    #[test]
    fn rejects_bad_pm1_ranges_lengths_aliases_and_hw_reduced_mode() {
        assert_eq!(
            parse(&fadt(0, 0), &dsdt(&root_s5(0, 0))),
            Err(Error::MissingPm1aRegister)
        );
        assert_eq!(
            parse(&fadt(0xffff, 0), &dsdt(&root_s5(0, 0))),
            Err(Error::InvalidRegisterAddress)
        );
        assert_eq!(
            parse(&fadt(0x404, 0x405), &dsdt(&root_s5(0, 0))),
            Err(Error::AliasedRegisters)
        );
        let mut short = fadt(0x404, 0);
        short[89] = 1;
        let short = finish_table(short, b"FACP");
        assert_eq!(
            parse(&short, &dsdt(&root_s5(0, 0))),
            Err(Error::InvalidPm1Length)
        );
        let mut reduced = fadt(0x404, 0);
        reduced[112..116].copy_from_slice(&(1u32 << 20).to_le_bytes());
        let reduced = finish_table(reduced, b"FACP");
        assert_eq!(
            parse(&reduced, &dsdt(&root_s5(0, 0))),
            Err(Error::UnsupportedHardwareReducedAcpi)
        );
    }

    #[test]
    fn prefers_valid_extended_pm1_gas_and_dsdt_address() {
        let mut fadt = vec![0; 196];
        fadt[40..44].copy_from_slice(&1u32.to_le_bytes());
        fadt[64..68].copy_from_slice(&0x404u32.to_le_bytes());
        fadt[89] = 2;
        fadt[140..148].copy_from_slice(&DSDT_ADDRESS.to_le_bytes());
        fadt[172] = 1;
        fadt[173] = 16;
        fadt[176..184].copy_from_slice(&0x504u64.to_le_bytes());
        fadt[184] = 1;
        fadt[185] = 16;
        fadt[188..196].copy_from_slice(&0x508u64.to_le_bytes());
        let fadt = finish_table(fadt, b"FACP");
        let plan = parse(&fadt, &dsdt(&root_s5(3, 4))).unwrap();
        assert_eq!(plan.pm1a().address(), 0x504);
        assert_eq!(plan.pm1b().unwrap().address(), 0x508);
    }

    #[test]
    fn rejects_invalid_extended_gas_and_partial_fadt_extensions() {
        let mut fadt = vec![0; 184];
        fadt[40..44].copy_from_slice(&(DSDT_ADDRESS as u32).to_le_bytes());
        fadt[64..68].copy_from_slice(&0x404u32.to_le_bytes());
        fadt[89] = 2;
        fadt[172] = 0; // System Memory is not accepted for legacy PM1 I/O.
        fadt[173] = 16;
        fadt[176..184].copy_from_slice(&0x504u64.to_le_bytes());
        let fadt = finish_table(fadt, b"FACP");
        assert_eq!(parse(&fadt, &dsdt(&root_s5(0, 0))), Err(Error::InvalidGas));

        let mut partial = vec![0; 180];
        partial[40..44].copy_from_slice(&(DSDT_ADDRESS as u32).to_le_bytes());
        partial[64..68].copy_from_slice(&0x404u32.to_le_bytes());
        partial[89] = 2;
        let partial = finish_table(partial, b"FACP");
        assert_eq!(
            parse(&partial, &dsdt(&root_s5(0, 0))),
            Err(Error::InvalidFadtLength)
        );
    }

    #[test]
    fn rejects_non_package_s5_and_malformed_pkg_length() {
        let mut non_package = vec![0x08];
        non_package.extend_from_slice(b"_S5_");
        non_package.extend_from_slice(&[0x01]);
        assert_eq!(
            parse(&fadt(0x404, 0), &dsdt(&non_package)),
            Err(Error::InvalidS5Package)
        );

        let mut bad_length = vec![0x08];
        bad_length.extend_from_slice(b"_S5_");
        bad_length.extend_from_slice(&[0x12, 0x01]);
        assert_eq!(
            parse(&fadt(0x404, 0), &dsdt(&bad_length)),
            Err(Error::MalformedAml)
        );
    }

    #[test]
    fn validates_all_pkg_length_widths_and_rejects_reserved_lead_bits() {
        let firmware = fadt(0x404, 0);
        for follow in 1..=3u8 {
            let mut aml = vec![0x08];
            aml.extend_from_slice(b"_S5_");
            aml.push(0x12);
            let length_offset = aml.len();
            aml.push((follow << 6) | (follow + 4));
            aml.extend_from_slice(&[0; 3][..usize::from(follow)]);
            aml.extend_from_slice(&[2, 0, 1]);
            let plan = parse(&firmware, &dsdt(&aml)).unwrap();
            assert_eq!(plan.pm1a().sleep_type(), 0);

            for reserved in [0x10, 0x20, 0x30] {
                let mut malformed = aml.clone();
                malformed[length_offset] |= reserved;
                assert_eq!(
                    parse(&firmware, &dsdt(&malformed)),
                    Err(Error::MalformedAml),
                    "follow={follow}, reserved={reserved:#x}"
                );
            }
        }
    }

    #[test]
    fn rejects_all_truncated_length_encodings_and_out_of_bounds_packages() {
        let firmware = fadt(0x404, 0);
        for follow in 1..=3u8 {
            for available in 0..usize::from(follow) {
                let mut aml = vec![0x08];
                aml.extend_from_slice(b"_S5_");
                aml.extend_from_slice(&[0x12, follow << 6]);
                aml.extend_from_slice(&[0xff; 3][..available]);
                assert_eq!(parse(&firmware, &dsdt(&aml)), Err(Error::MalformedAml));
            }
        }
        for length in [
            &[0x00][..],
            &[0x40, 0x00],
            &[0x81, 0x00, 0x00],
            &[0xc2, 0x00, 0x00, 0x00],
            &[0x3f],
            &[0x4f, 0xff],
            &[0x8f, 0xff, 0xff],
            &[0xcf, 0xff, 0xff, 0xff],
        ] {
            let mut aml = vec![0x08];
            aml.extend_from_slice(b"_S5_");
            aml.push(0x12);
            aml.extend_from_slice(length);
            aml.extend_from_slice(&[2, 0, 0]);
            assert_eq!(parse(&firmware, &dsdt(&aml)), Err(Error::MalformedAml));
        }
    }

    #[test]
    fn rejects_truncated_objects_even_after_a_valid_s5() {
        let firmware = fadt(0x404, 0);
        let mut named_string = vec![0x08];
        named_string.extend_from_slice(b"TEXT");
        named_string.extend_from_slice(&[0x0d, b'x', 0]);
        let objects = [
            root_s5(5, 6),
            named_string,
            vec![0x10, 5, b'_', b'S', b'B', b'_'],
            vec![0x14, 6, b'T', b'E', b'S', b'T', 0],
            vec![0x5b, 0x82, 5, b'D', b'E', b'V', b'0'],
            vec![0x5b, 0x01, b'M', b'T', b'X', b'0', 0],
            vec![0x5b, 0x02, b'E', b'V', b'T', b'0'],
            vec![0x15, b'E', b'X', b'T', b'0', 1, 0],
        ];
        for object in objects {
            for cut in 1..object.len() {
                let truncated = &object[..cut];
                assert!(parse(&firmware, &dsdt(truncated)).is_err());
                let mut after_s5 = root_s5(0, 0);
                after_s5.extend_from_slice(truncated);
                assert!(
                    parse(&firmware, &dsdt(&after_s5)).is_err(),
                    "object={object:02x?}, cut={cut}"
                );
            }
        }
    }

    #[test]
    fn rejects_every_truncated_table_slice_without_panicking() {
        let firmware = fadt(0x404, 0);
        let definition = dsdt(&root_s5(0, 0));
        for cut in 0..firmware.len() {
            assert!(parse(&firmware[..cut], &definition).is_err());
        }
        for cut in 0..definition.len() {
            assert!(parse(&firmware, &definition[..cut]).is_err());
        }
    }
}
