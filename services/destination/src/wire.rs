//! Destination wire v1 (spec 2.2, 2.3): a 224-byte body inside `Message320`,
//! a 64-byte header and at most 160 data bytes. Check 2 of 2.2 is
//! [`validate`]; it never depends on service state.
use crate::{get32, get64, put32, put64, Status, ID, LABEL};

pub const SIZE: usize = 224;
pub const DATA: usize = 160;
pub const VERSION: u16 = 1;
/// LIST record size and records per reply (2.3).
pub const LIST_RECORD: usize = 80;
pub const LIST_MAX: usize = 2;
pub const INFO_LENGTH: usize = 48;

pub const INFO: u16 = 1;
pub const LIST: u16 = 2;
pub const CREATE: u16 = 3;
pub const APPEND: u16 = 4;
pub const SEAL: u16 = 5;
pub const ABORT: u16 = 6;
pub const VERIFY: u16 = 7;
pub const READ: u16 = 8;
pub const BIND: u16 = 9;
pub const UNBIND: u16 = 10;
pub const FORMAT: u16 = 11;

/// Lanes: 0 is the control lane, 1..=3 the operation lanes (4.3).
pub const CONTROL: usize = 0;
pub const LANES: usize = 4;

/// BIND operation codes in `name_length` (2.3).
pub const BIND_EXPORT: u32 = 1;
pub const BIND_VERIFY: u32 = 2;
pub const BIND_RESTORE: u32 = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Header {
    pub operation: u16,
    pub request_id: u64,
    pub epoch: u64,
    pub entry: u64,
    pub offset: u64,
    pub length: u32,
    pub status: i32,
    pub name_length: u32,
    pub count: u32,
    pub aux: u64,
}

impl Header {
    pub const fn new(operation: u16, request_id: u64, epoch: u64, entry: u64) -> Self {
        Self {
            operation,
            request_id,
            epoch,
            entry,
            offset: 0,
            length: 0,
            status: 0,
            name_length: 0,
            count: 0,
            aux: 0,
        }
    }
}

/// Decode the fixed fields without trusting them.
pub fn header(body: &[u8; SIZE]) -> Header {
    Header {
        operation: u16::from_le_bytes([body[2], body[3]]),
        request_id: get64(body, 8),
        epoch: get64(body, 16),
        entry: get64(body, 24),
        offset: get64(body, 32),
        length: get32(body, 40),
        status: get32(body, 44) as i32,
        name_length: get32(body, 48),
        count: get32(body, 52),
        aux: get64(body, 56),
    }
}

pub fn encode(header: &Header, data: &[u8]) -> [u8; SIZE] {
    assert!(data.len() <= DATA && data.len() == header.length as usize);
    let mut body = [0; SIZE];
    body[..2].copy_from_slice(&VERSION.to_le_bytes());
    body[2..4].copy_from_slice(&header.operation.to_le_bytes());
    put64(&mut body, 8, header.request_id);
    put64(&mut body, 16, header.epoch);
    put64(&mut body, 24, header.entry);
    put64(&mut body, 32, header.offset);
    put32(&mut body, 40, header.length);
    put32(&mut body, 44, header.status as u32);
    put32(&mut body, 48, header.name_length);
    put32(&mut body, 52, header.count);
    put64(&mut body, 56, header.aux);
    body[64..64 + data.len()].copy_from_slice(data);
    body
}

pub fn data(body: &[u8; SIZE]) -> &[u8] {
    let length = (get32(body, 40) as usize).min(DATA);
    &body[64..64 + length]
}

/// Check 2 of 2.2: version, then operation (`NoSys`), then every other field
/// per the field-rule table (`Invalid`). `epoch` and `request_id` are check 5.
pub fn validate(body: &[u8; SIZE]) -> Result<Header, Status> {
    if u16::from_le_bytes([body[0], body[1]]) != VERSION {
        return Err(Status::Invalid);
    }
    let h = header(body);
    if !(INFO..=FORMAT).contains(&h.operation) {
        return Err(Status::NoSys);
    }
    let length = h.length as usize;
    if body[4..8] != [0; 4]
        || h.status != 0
        || length > DATA
        || body[64 + length..].iter().any(|byte| *byte != 0)
    {
        return Err(Status::Invalid);
    }
    let (entry, offset, name, count, aux) = (h.entry, h.offset, h.name_length, h.count, h.aux);
    let lane = (1..=3).contains(&count);
    let ok = match h.operation {
        INFO | FORMAT => entry == 0 && offset == 0 && length == 0 && name == 0 && count == 0,
        LIST => entry == 0 && offset == 0 && length == 0 && name == 0 && count < 16,
        CREATE => {
            entry == 0
                && offset == 0
                && (1..=LABEL).contains(&length)
                && name as usize == length
                && count == 0
        }
        APPEND => entry != 0 && (1..=DATA).contains(&length) && name == 0 && count == 0,
        SEAL => entry != 0 && length == 0 && name == 0 && count == 0,
        ABORT | VERIFY => entry != 0 && offset == 0 && length == 0 && name == 0 && count == 0,
        READ => {
            entry != 0 && length == 0 && name == 0 && count == 0 && (1..=DATA as u64).contains(&aux)
        }
        BIND => {
            (if name == BIND_EXPORT {
                entry == 0
            } else {
                entry != 0
            }) && offset == 0
                && length == ID
                && (BIND_EXPORT..=BIND_RESTORE).contains(&name)
                && lane
        }
        UNBIND => entry == 0 && offset == 0 && length == 0 && name == 0 && lane,
        _ => false,
    };
    // `aux` must be 0 except for READ (byte count) and BIND (generation).
    let aux_ok = matches!(h.operation, READ | BIND) || aux == 0;
    if ok && aux_ok {
        Ok(h)
    } else {
        Err(Status::Invalid)
    }
}
