//! Canonical M6c v4 messages. Only the kernel-bound endpoint supplies authority.

pub const SIZE: usize = 224;
pub const VERSION: u16 = 4;
pub const GET: u16 = 1;
pub const PUT: u16 = 2;
pub const DELETE: u16 = 3;
pub const RENAME: u16 = 4;
pub const LIST: u16 = 5;
pub const TAG: u64 = 0x4d36_434c_4b00_0004;
pub const NAME: usize = 32;
pub const VALUE: usize = 128;
pub const CATALOG_ENTRY: usize = 56;
pub const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    Ok = 0,
    Invalid = -1,
    NoSys = -2,
    Range = -3,
    Access = -5,
    Stale = -10,
    Io = -12,
    Timeout = -13,
    NoDevice = -14,
    Protocol = -15,
    Unknown = -16,
    Full = -17,
    Corrupt = -18,
    NotFound = -19,
}
impl Status {
    pub fn from_code(code: i64) -> Self {
        match code {
            0 => Self::Ok,
            -1 => Self::Invalid,
            -2 => Self::NoSys,
            -3 => Self::Range,
            -5 => Self::Access,
            -10 => Self::Stale,
            -12 => Self::Io,
            -13 => Self::Timeout,
            -14 => Self::NoDevice,
            -16 => Self::Unknown,
            -17 => Self::Full,
            -18 => Self::Corrupt,
            -19 => Self::NotFound,
            _ => Self::Protocol,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub operation: u16,
    pub request_id: u64,
    pub epoch: u64,
    pub object: u64,
    pub revision: u64,
    pub length: u32,
    pub status: i32,
    pub name_length: u32,
    pub count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CatalogEntry {
    pub object: u64,
    pub revision: u64,
    pub length: usize,
    pub name_length: usize,
    pub name: [u8; NAME],
}
impl CatalogEntry {
    pub const EMPTY: Self = Self {
        object: 0,
        revision: 0,
        length: 0,
        name_length: 0,
        name: [0; NAME],
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Catalog {
    pub generation: u64,
    pub count: usize,
    pub entries: [CatalogEntry; 2],
}
impl Catalog {
    pub const EMPTY: Self = Self::empty();
    pub const fn empty() -> Self {
        Self {
            generation: 0,
            count: 0,
            entries: [CatalogEntry::EMPTY; 2],
        }
    }
}

// Helpers use fixed bounds-checked offsets; external bytes are never cast to
// aligned Rust structures or interpreted as pointers/authority.
pub fn get32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}
pub fn get64(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes([
        bytes[at],
        bytes[at + 1],
        bytes[at + 2],
        bytes[at + 3],
        bytes[at + 4],
        bytes[at + 5],
        bytes[at + 6],
        bytes[at + 7],
    ])
}
pub fn put32(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
pub fn put64(bytes: &mut [u8], at: usize, value: u64) {
    bytes[at..at + 8].copy_from_slice(&value.to_le_bytes());
}
/// Diagnostic checksum only; not authentication or rollback protection.
pub fn checksum_update(mut value: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        value = (value ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
    value
}

pub fn valid_name(bytes: &[u8]) -> bool {
    if bytes.is_empty() || bytes.len() > NAME {
        return false;
    }
    let Ok(name) = core::str::from_utf8(bytes) else {
        return false;
    };
    name != "."
        && name != ".."
        && name.chars().all(|ch| {
            !ch.is_control()
                && ch != '/'
                && ch != '\\'
                && !matches!(ch, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
}

pub fn header(bytes: &[u8; SIZE]) -> Header {
    Header {
        operation: u16::from_le_bytes([bytes[2], bytes[3]]),
        request_id: get64(bytes, 8),
        epoch: get64(bytes, 16),
        object: get64(bytes, 24),
        revision: get64(bytes, 32),
        length: get32(bytes, 40),
        status: get32(bytes, 44) as i32,
        name_length: get32(bytes, 48),
        count: get32(bytes, 52),
    }
}
fn common(bytes: &[u8; SIZE]) -> bool {
    bytes[..2] == VERSION.to_le_bytes()
        && get32(bytes, 4) == 0
        && bytes[56..64].iter().all(|byte| *byte == 0)
}
fn canonical_payload(bytes: &[u8; SIZE], h: Header) -> bool {
    h.length <= VALUE as u32
        && h.name_length <= NAME as u32
        && bytes[64 + h.length as usize..192]
            .iter()
            .all(|byte| *byte == 0)
        && bytes[192 + h.name_length as usize..]
            .iter()
            .all(|byte| *byte == 0)
}

/// Oversized inputs become an invalid bounded message, never a truncated name
/// or a panic. Validation, not constructor success, determines acceptance.
pub fn request(
    id: u64,
    epoch: u64,
    object: u64,
    revision: u64,
    op: u16,
    name: Option<&[u8]>,
    value: Option<&[u8]>,
) -> [u8; SIZE] {
    let mut bytes = [0; SIZE];
    bytes[..2].copy_from_slice(&VERSION.to_le_bytes());
    bytes[2..4].copy_from_slice(&op.to_le_bytes());
    put64(&mut bytes, 8, id);
    put64(&mut bytes, 16, epoch);
    put64(&mut bytes, 24, object);
    put64(&mut bytes, 32, revision);
    if let Some(value) = value {
        if value.len() > VALUE {
            put32(&mut bytes, 40, u32::MAX);
        } else {
            put32(&mut bytes, 40, value.len() as u32);
            bytes[64..64 + value.len()].copy_from_slice(value);
        }
    }
    if let Some(name) = name {
        if name.len() > NAME {
            put32(&mut bytes, 48, u32::MAX);
        } else {
            put32(&mut bytes, 48, name.len() as u32);
            bytes[192..192 + name.len()].copy_from_slice(name);
        }
    }
    bytes
}

pub fn validate_request(bytes: &[u8; SIZE], epoch: u64, domain: usize) -> Result<Header, Status> {
    let h = header(bytes);
    if !common(bytes) || h.request_id == 0 || h.status != 0 || h.count != 0 {
        return Err(Status::Invalid);
    }
    if epoch == 0 || h.epoch != epoch {
        return Err(Status::Stale);
    }
    if h.object >= 4 {
        return Err(Status::Range);
    }
    if domain >= 2 || h.object / 2 != domain as u64 {
        return Err(Status::Access);
    }
    if h.length > VALUE as u32 || h.name_length > NAME as u32 || h.revision > 32 {
        return Err(Status::Range);
    }
    if !canonical_payload(bytes, h) {
        return Err(Status::Invalid);
    }
    match h.operation {
        GET if h.revision == 0 && h.length == 0 && h.name_length == 0 => {}
        PUT if valid_name(&bytes[192..192 + h.name_length as usize]) => {}
        DELETE if h.length == 0 && h.name_length == 0 => {}
        RENAME if h.length == 0 && valid_name(&bytes[192..192 + h.name_length as usize]) => {}
        LIST if h.object.is_multiple_of(2)
            && h.revision == 0
            && h.length == 0
            && h.name_length == 0 => {}
        GET | PUT | DELETE | RENAME | LIST => return Err(Status::Invalid),
        _ => return Err(Status::NoSys),
    }
    Ok(h)
}

/// Only successful GET returns content/name. NotFound exposes only the already
/// authorized object's revision; every other error clears all metadata.
pub fn reply(
    h: Header,
    epoch: u64,
    revision: u64,
    status: Status,
    name: Option<&[u8]>,
    value: Option<&[u8]>,
) -> [u8; SIZE] {
    let revision = if matches!(status, Status::Ok | Status::NotFound) {
        revision
    } else {
        0
    };
    let include = status == Status::Ok && h.operation == GET;
    let mut bytes = request(
        h.request_id,
        epoch,
        h.object,
        revision,
        h.operation,
        if include { name } else { None },
        if include { value } else { None },
    );
    put32(&mut bytes, 44, status as i32 as u32);
    bytes
}

pub fn catalog_reply(
    h: Header,
    epoch: u64,
    status: Status,
    catalog: Option<&Catalog>,
) -> [u8; SIZE] {
    let mut bytes = reply(h, epoch, 0, status, None, None);
    if status != Status::Ok {
        return bytes;
    }
    let Some(catalog) = catalog else {
        return reply(h, epoch, 0, Status::Protocol, None, None);
    };
    if h.operation != LIST
        || h.object >= 4
        || !h.object.is_multiple_of(2)
        || !valid_catalog(catalog, h.object / 2)
    {
        return reply(h, epoch, 0, Status::Protocol, None, None);
    }
    put64(&mut bytes, 32, catalog.generation);
    put32(&mut bytes, 40, (catalog.count * CATALOG_ENTRY) as u32);
    put32(&mut bytes, 52, catalog.count as u32);
    for (i, entry) in catalog.entries[..catalog.count].iter().enumerate() {
        let at = 64 + i * CATALOG_ENTRY;
        put64(&mut bytes, at, entry.object);
        put64(&mut bytes, at + 8, entry.revision);
        put32(&mut bytes, at + 16, entry.length as u32);
        put32(&mut bytes, at + 20, entry.name_length as u32);
        bytes[at + 24..at + 56].copy_from_slice(&entry.name);
    }
    bytes
}

fn valid_catalog(catalog: &Catalog, domain: u64) -> bool {
    if domain >= 2 || catalog.count > 2 || catalog.generation > 32 {
        return false;
    }
    let mut revisions = 0u64;
    let mut previous = None;
    for entry in &catalog.entries[..catalog.count] {
        if entry.object >= 4
            || entry.object / 2 != domain
            || previous.is_some_and(|object| object >= entry.object)
            || entry.revision == 0
            || entry.revision > 32
            || entry.length > VALUE
            || entry.name_length > NAME
            || !valid_name(&entry.name[..entry.name_length])
            || entry.name[entry.name_length..]
                .iter()
                .any(|byte| *byte != 0)
        {
            return false;
        }
        previous = Some(entry.object);
        revisions += entry.revision;
    }
    // Deleted objects can contribute revisions without appearing. With two
    // live objects the exact domain total is known; with fewer it is a bound.
    revisions <= catalog.generation
        && (catalog.count != 2 || revisions == catalog.generation)
        && catalog.entries[catalog.count..]
            .iter()
            .all(|entry| *entry == CatalogEntry::EMPTY)
}

fn read_catalog(bytes: &[u8; SIZE], h: Header) -> Result<Catalog, Status> {
    if h.object >= 4
        || !h.object.is_multiple_of(2)
        || h.count > 2
        || h.name_length != 0
        || h.length != h.count * CATALOG_ENTRY as u32
    {
        return Err(Status::Invalid);
    }
    let mut catalog = Catalog {
        generation: h.revision,
        count: h.count as usize,
        ..Catalog::empty()
    };
    for i in 0..catalog.count {
        let at = 64 + i * CATALOG_ENTRY;
        let mut name = [0; NAME];
        name.copy_from_slice(&bytes[at + 24..at + 56]);
        catalog.entries[i] = CatalogEntry {
            object: get64(bytes, at),
            revision: get64(bytes, at + 8),
            length: get32(bytes, at + 16) as usize,
            name_length: get32(bytes, at + 20) as usize,
            name,
        };
    }
    if !valid_catalog(&catalog, h.object / 2) {
        return Err(Status::Invalid);
    }
    Ok(catalog)
}

pub fn validate_reply(bytes: &[u8; SIZE], req: Header, epoch: u64) -> Result<Header, Status> {
    let h = header(bytes);
    if !common(bytes)
        || h.request_id != req.request_id
        || h.operation != req.operation
        || h.object != req.object
        || h.epoch != epoch
        || epoch == 0
        || !canonical_payload(bytes, h)
    {
        return Err(Status::Invalid);
    }
    if h.status == 0 {
        if req.request_id == 0 || req.epoch != epoch || h.object >= 4 || req.count != 0 {
            return Err(Status::Invalid);
        }
        match h.operation {
            LIST if req.revision == 0 && req.length == 0 && req.name_length == 0 => {
                read_catalog(bytes, h)?;
            }
            GET if (1..=32).contains(&h.revision)
                && h.count == 0
                && req.revision == 0
                && req.length == 0
                && req.name_length == 0
                && valid_name(&bytes[192..192 + h.name_length as usize]) => {}
            PUT | DELETE | RENAME
                if (1..=32).contains(&h.revision)
                    && h.length == 0
                    && h.name_length == 0
                    && h.count == 0
                    && req.revision.checked_add(1) == Some(h.revision) => {}
            _ => return Err(Status::Invalid),
        }
    } else {
        if h.length != 0
            || h.name_length != 0
            || h.count != 0
            || !matches!(
                h.status,
                -1 | -2 | -3 | -5 | -10 | -12 | -13 | -14 | -15 | -16 | -17 | -18 | -19
            )
        {
            return Err(Status::Invalid);
        }
        if h.status == Status::NotFound as i32 {
            if !matches!(h.operation, GET | DELETE | RENAME) || h.revision > 32 {
                return Err(Status::Invalid);
            }
        } else if h.revision != 0 {
            return Err(Status::Invalid);
        }
    }
    Ok(h)
}

/// Fully validates correlation, domain, order, bounds and canonical padding.
/// Error replies are returned as errors, never interpreted as empty catalogs.
pub fn decode_catalog(bytes: &[u8; SIZE], req: Header, epoch: u64) -> Result<Catalog, Status> {
    let h = validate_reply(bytes, req, epoch)?;
    if h.status != 0 {
        return Err(Status::from_code(i64::from(h.status)));
    }
    if h.operation != LIST {
        return Err(Status::Invalid);
    }
    read_catalog(bytes, h)
}
