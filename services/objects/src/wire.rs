//! Canonical M6b wire format. Endpoint identity supplies the domain authority.

pub const SIZE: usize = 224;
pub const VERSION: u16 = 3;
pub const GET: u16 = 1;
pub const PUT: u16 = 2;
pub const DELETE: u16 = 3;
pub const TAG: u64 = 0x4d36_424c_4b00_0003;
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
}

// Callers use fixed, bounds-checked offsets in canonical arrays. These helpers
// never cast wire bytes to Rust structs or assume aligned external pointers.
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
/// Diagnostic checksum only; no cryptographic authentication claim.
pub fn checksum_update(mut value: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        value = (value ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
    value
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
    }
}
fn common(bytes: &[u8; SIZE]) -> bool {
    bytes[..2] == VERSION.to_le_bytes()
        && get32(bytes, 4) == 0
        && bytes[48..64]
            .iter()
            .chain(bytes[192..].iter())
            .all(|byte| *byte == 0)
}
fn canonical_payload(bytes: &[u8; SIZE], length: u32) -> bool {
    length <= 128
        && bytes[64 + length as usize..192]
            .iter()
            .all(|byte| *byte == 0)
}

/// Oversized constructor input produces an invalid bounded message, not a panic
/// or a silently truncated value. The receiver rejects its u32::MAX length.
pub fn request(
    id: u64,
    epoch: u64,
    object: u64,
    revision: u64,
    op: u16,
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
        if value.len() > 128 {
            put32(&mut bytes, 40, u32::MAX);
        } else {
            put32(&mut bytes, 40, value.len() as u32);
            bytes[64..64 + value.len()].copy_from_slice(value);
        }
    }
    bytes
}

pub fn validate_request(bytes: &[u8; SIZE], epoch: u64, domain: usize) -> Result<Header, Status> {
    let h = header(bytes);
    if !common(bytes) || h.request_id == 0 || h.status != 0 {
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
    if h.length > 128 || h.revision > 32 {
        return Err(Status::Range);
    }
    if !canonical_payload(bytes, h.length) {
        return Err(Status::Invalid);
    }
    match h.operation {
        GET if h.revision == 0 && h.length == 0 => {}
        PUT => {}
        DELETE if h.length == 0 => {}
        GET | DELETE => return Err(Status::Invalid),
        _ => return Err(Status::NoSys),
    }
    Ok(h)
}

/// NotFound exposes only the already-authorized object's current revision.
/// This permits CAS recreation of a tombstone without allowing an ABA reset.
pub fn reply(
    h: Header,
    epoch: u64,
    revision: u64,
    status: Status,
    value: Option<&[u8]>,
) -> [u8; SIZE] {
    let revision = if matches!(status, Status::Ok | Status::NotFound) {
        revision
    } else {
        0
    };
    let mut bytes = request(
        h.request_id,
        epoch,
        h.object,
        revision,
        h.operation,
        if status == Status::Ok { value } else { None },
    );
    put32(&mut bytes, 44, status as i32 as u32);
    bytes
}

pub fn validate_reply(bytes: &[u8; SIZE], req: Header, epoch: u64) -> Result<Header, Status> {
    let h = header(bytes);
    if !common(bytes)
        || h.request_id != req.request_id
        || h.operation != req.operation
        || h.object != req.object
        || h.epoch != epoch
        || epoch == 0
        || !canonical_payload(bytes, h.length)
    {
        return Err(Status::Invalid);
    }
    if h.status == 0 {
        if !(1..=32).contains(&h.revision) {
            return Err(Status::Invalid);
        }
        match h.operation {
            GET => {}
            PUT | DELETE if h.length == 0 && req.revision.checked_add(1) == Some(h.revision) => {}
            _ => return Err(Status::Invalid),
        }
    } else {
        if h.length != 0
            || !matches!(
                h.status,
                -1 | -2 | -3 | -5 | -10 | -12 | -13 | -14 | -15 | -16 | -17 | -18 | -19
            )
        {
            return Err(Status::Invalid);
        }
        if h.status == Status::NotFound as i32 {
            if !matches!(h.operation, GET | DELETE) || h.revision > 32 {
                return Err(Status::Invalid);
            }
        } else if h.revision != 0 {
            return Err(Status::Invalid);
        }
    }
    Ok(h)
}
