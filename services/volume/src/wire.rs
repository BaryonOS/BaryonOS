//! Canonical M6e v6 volume messages (224 payload bytes inside Message320).
//! Authority comes only from the kernel-bound endpoint; object IDs select
//! within the caller's own domain and are re-authorized on every request.
//!
//! v6 (volume format v2) adds RESTORE, PURGE, META, COLLECTION, TRANSACT and
//! COLLECTION_STAT, history selectors for READ, collection filters for LIST
//! and an extended STAT. The generic header field at offset 48 is a per-operation parameter:
//! the name length for BEGIN/RENAME/COLLECTION and the version selector for
//! READ (0 = current, 1..=HISTORY = history slot).
use crate::{
    domain_of, get32, get64, put32, put64, valid_name, Status, COLLECTIONS, HISTORY, MAX_LENGTH,
    NAME, OBJECTS, PER_DOMAIN, TRANSACT_MAX,
};

pub const SIZE: usize = 224;
pub const VERSION: u16 = 6;
pub const TAG: u64 = 0x4d36_454c_4b00_0006;
pub const DATA: usize = 160;
pub const LIST_ENTRY: usize = 48;
pub const LIST_MAX: usize = 3;
pub const STAT_LENGTH: u32 = 120;
/// Per history version in STAT: revision u64, length u32, content sum u64.
pub const STAT_HISTORY: usize = 20;
pub const SUBOP: usize = 24;

pub const STAT: u16 = 1;
pub const BEGIN: u16 = 2;
pub const APPEND: u16 = 3;
pub const COMMIT: u16 = 4;
pub const ABORT: u16 = 5;
pub const READ: u16 = 6;
pub const DELETE: u16 = 7;
pub const RENAME: u16 = 8;
pub const LIST: u16 = 9;
pub const RESTORE: u16 = 10;
pub const PURGE: u16 = 11;
pub const META: u16 = 12;
pub const COLLECTION: u16 = 13;
pub const TRANSACT: u16 = 14;
/// Read-only state of one collection of the caller's domain.
pub const COLLECTION_STAT: u16 = 15;

/// TRANSACT sub-operations.
pub const SUB_META: u8 = 1;
pub const SUB_DELETE: u8 = 2;
pub const SUB_PURGE: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub operation: u16,
    pub request_id: u64,
    pub epoch: u64,
    pub object: u64,
    pub revision: u64,
    pub length: u32,
    pub status: i32,
    /// Name length (BEGIN/RENAME/COLLECTION/STAT reply) or READ selector.
    pub name_length: u32,
    pub count: u32,
    pub aux: u64,
}

impl Header {
    pub const fn new(operation: u16, request_id: u64, epoch: u64, object: u64) -> Self {
        Self {
            operation,
            request_id,
            epoch,
            object,
            revision: 0,
            length: 0,
            status: 0,
            name_length: 0,
            count: 0,
            aux: 0,
        }
    }
}

/// META payload in `aux`: tags (bits 0-31), kind (32-47), collection (48-55).
pub const fn meta(tags: u32, kind: u16, collection: u8) -> u64 {
    tags as u64 | (kind as u64) << 32 | (collection as u64) << 48
}

/// Split a META payload; the flag is false for non-canonical values.
pub const fn unmeta(aux: u64) -> (u32, u16, u8, bool) {
    (
        aux as u32,
        (aux >> 32) as u16,
        (aux >> 48) as u8,
        aux >> 56 == 0 && ((aux >> 48) as u8) as usize <= COLLECTIONS,
    )
}

/// One TRANSACT sub-operation (24 bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubOp {
    /// Index within the caller's domain (0..PER_DOMAIN).
    pub index: u8,
    pub op: u8,
    pub collection: u8,
    pub tags: u32,
    pub expected: u64,
    pub kind: u16,
}

pub fn encode_subop(sub: &SubOp, out: &mut [u8]) {
    out[0] = sub.index;
    out[1] = sub.op;
    out[2] = sub.collection;
    put32(out, 4, sub.tags);
    put64(out, 8, sub.expected);
    out[16..18].copy_from_slice(&sub.kind.to_le_bytes());
}

/// Decode and validate the shape (not the state) of one sub-operation.
pub fn decode_subop(bytes: &[u8]) -> Option<SubOp> {
    let sub = SubOp {
        index: bytes[0],
        op: bytes[1],
        collection: bytes[2],
        tags: get32(bytes, 4),
        expected: get64(bytes, 8),
        kind: u16::from_le_bytes([bytes[16], bytes[17]]),
    };
    let structural = bytes[3] == 0
        && bytes[18..SUBOP].iter().all(|byte| *byte == 0)
        && usize::from(sub.index) < PER_DOMAIN;
    let shaped = match sub.op {
        SUB_META => usize::from(sub.collection) <= COLLECTIONS,
        SUB_DELETE | SUB_PURGE => sub.collection == 0 && sub.tags == 0 && sub.kind == 0,
        _ => false,
    };
    (structural && shaped).then_some(sub)
}

/// Decode fixed header fields without trusting them; callers validate.
pub fn header(body: &[u8; SIZE]) -> Header {
    Header {
        operation: u16::from_le_bytes([body[2], body[3]]),
        request_id: get64(body, 8),
        epoch: get64(body, 16),
        object: get64(body, 24),
        revision: get64(body, 32),
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
    put64(&mut body, 24, header.object);
    put64(&mut body, 32, header.revision);
    put32(&mut body, 40, header.length);
    put32(&mut body, 44, header.status as u32);
    put32(&mut body, 48, header.name_length);
    put32(&mut body, 52, header.count);
    put64(&mut body, 56, header.aux);
    body[64..64 + data.len()].copy_from_slice(data);
    body
}

pub fn data(body: &[u8; SIZE], length: usize) -> &[u8] {
    &body[64..64 + length]
}

/// Strict structural validation of a request from `domain`. Returns the
/// header only if every field is canonical for its operation. Cross-domain
/// object selectors are rejected as Access before any state is consulted.
pub fn validate_request(body: &[u8; SIZE], epoch: u64, domain: usize) -> Result<Header, Status> {
    let h = header(body);
    if u16::from_le_bytes([body[0], body[1]]) != VERSION || body[4..8] != [0; 4] {
        return Err(Status::Invalid);
    }
    if !(STAT..=COLLECTION_STAT).contains(&h.operation) {
        return Err(Status::NoSys);
    }
    if h.epoch != epoch || h.status != 0 || h.request_id == 0 {
        return Err(Status::Invalid);
    }
    if h.object >= OBJECTS as u64 {
        return Err(Status::Range);
    }
    if domain_of(h.object as usize) != domain {
        return Err(Status::Access);
    }
    let length = h.length as usize;
    if length > DATA || body[64 + length..].iter().any(|byte| *byte != 0) {
        return Err(Status::Invalid);
    }
    let base = h.object as usize == domain * PER_DOMAIN;
    let named =
        || h.name_length as usize == length && length <= NAME && valid_name(&body[64..64 + length]);
    let ok = match h.operation {
        STAT => h.revision == 0 && length == 0 && h.name_length == 0 && h.count == 0 && h.aux == 0,
        BEGIN => h.count == 0 && h.aux as usize <= MAX_LENGTH && named(),
        APPEND => h.count == 0 && h.name_length == 0 && length != 0 && h.aux < MAX_LENGTH as u64,
        COMMIT => h.count == 0 && h.name_length == 0 && length == 0 && h.aux <= MAX_LENGTH as u64,
        ABORT | DELETE | PURGE => h.count == 0 && h.name_length == 0 && length == 0 && h.aux == 0,
        READ => {
            // The requested maximum travels in `count`; no data is supplied.
            h.name_length as usize <= HISTORY
                && length == 0
                && (1..=DATA as u32).contains(&h.count)
                && h.aux <= MAX_LENGTH as u64
        }
        RENAME => h.count == 0 && h.aux == 0 && named(),
        LIST => {
            base && h.revision == 0
                && length == 0
                && h.name_length == 0
                && h.count as usize <= COLLECTIONS + 1
                && h.aux < PER_DOMAIN as u64
        }
        RESTORE => {
            h.count == 0
                && h.name_length == 0
                && length == 0
                && (1..=HISTORY as u64).contains(&h.aux)
        }
        META => h.count == 0 && h.name_length == 0 && length == 0 && unmeta(h.aux).3,
        COLLECTION => {
            base && h.count == 0
                && (1..=COLLECTIONS as u64).contains(&h.aux)
                && (length == 0 && h.name_length == 0 || named())
        }
        COLLECTION_STAT => {
            base && h.revision == 0
                && length == 0
                && h.name_length == 0
                && h.count == 0
                && (1..=COLLECTIONS as u64).contains(&h.aux)
        }
        TRANSACT => {
            let count = h.count as usize;
            base && h.revision == 0
                && h.aux == 0
                && h.name_length == 0
                && (1..=TRANSACT_MAX).contains(&count)
                && length == count * SUBOP
                && (0..count).all(|index| {
                    let bytes = &body[64 + index * SUBOP..64 + (index + 1) * SUBOP];
                    decode_subop(bytes).is_some_and(|sub| {
                        // Each object appears at most once per transaction.
                        (0..index).all(|other| body[64 + other * SUBOP] != sub.index)
                    })
                })
        }
        _ => false,
    };
    if ok {
        Ok(h)
    } else {
        Err(Status::Invalid)
    }
}

/// Canonical error reply: echo identity, never data, names or revisions,
/// except the authorized tombstone revision for NotFound.
pub fn error_reply(request: &Header, epoch: u64, status: Status, revision: u64) -> [u8; SIZE] {
    let mut h = Header::new(request.operation, request.request_id, epoch, request.object);
    h.status = status as i32;
    h.revision = if status == Status::NotFound {
        revision
    } else {
        0
    };
    encode(&h, &[])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListEntry {
    pub object: u64,
    pub revision: u64,
    pub length: usize,
    pub name_length: usize,
    pub name: [u8; NAME],
    pub collection: u8,
    pub history_count: u8,
}

impl ListEntry {
    pub const EMPTY: Self = Self {
        object: 0,
        revision: 0,
        length: 0,
        name_length: 0,
        name: [0; NAME],
        collection: 0,
        history_count: 0,
    };
}

pub fn encode_list_entry(entry: &ListEntry, out: &mut [u8]) {
    out[0] = entry.object as u8;
    out[1] = entry.name_length as u8;
    out[2] = entry.collection;
    out[3] = entry.history_count;
    put32(out, 4, entry.length as u32);
    put64(out, 8, entry.revision);
    out[16..48].copy_from_slice(&entry.name);
}

pub fn decode_list_entry(bytes: &[u8]) -> ListEntry {
    let mut name = [0; NAME];
    name.copy_from_slice(&bytes[16..48]);
    ListEntry {
        object: u64::from(bytes[0]),
        revision: get64(bytes, 8),
        length: get32(bytes, 4) as usize,
        name_length: usize::from(bytes[1]),
        name,
        collection: bytes[2],
        history_count: bytes[3],
    }
}
