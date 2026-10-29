//! IPC v1 of the Key Service (tag `KEY1`). Requests and replies travel in one
//! 256-byte payload: a 16-byte header plus a bounded body. Lengths are
//! validated before any byte is interpreted; the domain is never carried on
//! the wire because the ingress endpoint already fixes it.
use crate::{Status, MAX_PLAINTEXT, MAX_SEALED, RECOVERY_BLOB, SECRET_BYTES};

pub const TAG: u64 = 0x4b45_5931;
pub const SIZE: usize = 256;
pub const HEADER: usize = 16;
pub const BODY: usize = SIZE - HEADER;

pub const STATUS: u8 = 1;
pub const GENERATE: u8 = 2;
pub const SEAL: u8 = 3;
pub const UNSEAL: u8 = 4;
pub const REVOKE: u8 = 5;
pub const EXPORT_RECOVERY: u8 = 6;
pub const RESTORE: u8 = 7;
pub const REPLY: u8 = 0x80;

/// Header: op (1), key index (1), body length (2 LE), request id (8), reserved (4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub op: u8,
    pub index: u8,
    pub length: u16,
    pub request: u64,
    /// Reply only: status code (i32); requests must carry 0.
    pub status: i32,
}

impl Header {
    pub fn encode(self) -> [u8; HEADER] {
        let mut bytes = [0u8; HEADER];
        bytes[0] = self.op;
        bytes[1] = self.index;
        bytes[2..4].copy_from_slice(&self.length.to_le_bytes());
        bytes[4..12].copy_from_slice(&self.request.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.status.to_le_bytes());
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE {
            return None;
        }
        let length = u16::from_le_bytes([bytes[2], bytes[3]]);
        if usize::from(length) > BODY {
            return None;
        }
        // Everything after the body must be zero: canonical payloads only.
        if bytes[HEADER + usize::from(length)..]
            .iter()
            .any(|b| *b != 0)
        {
            return None;
        }
        Some(Self {
            op: bytes[0],
            index: bytes[1],
            length,
            request: u64::from_le_bytes(bytes[4..12].try_into().ok()?),
            status: i32::from_le_bytes(bytes[12..16].try_into().ok()?),
        })
    }
}

pub fn message(header: Header, body: &[u8]) -> Option<[u8; SIZE]> {
    if body.len() > BODY || usize::from(header.length) != body.len() {
        return None;
    }
    let mut out = [0u8; SIZE];
    out[..HEADER].copy_from_slice(&header.encode());
    out[HEADER..HEADER + body.len()].copy_from_slice(body);
    Some(out)
}

pub fn request(op: u8, index: u8, request: u64, body: &[u8]) -> Option<[u8; SIZE]> {
    message(
        Header {
            op,
            index,
            length: body.len() as u16,
            request,
            status: 0,
        },
        body,
    )
}

pub fn reply(request: &Header, status: Status, body: &[u8]) -> [u8; SIZE] {
    let body = if status == Status::Ok { body } else { &[] };
    message(
        Header {
            op: request.op | REPLY,
            index: request.index,
            length: body.len() as u16,
            request: request.request,
            status: status.code(),
        },
        body,
    )
    .expect("reply bodies are bounded by construction")
}

/// Valid request body lengths per operation.
pub fn body_bounds(op: u8) -> Option<(usize, usize)> {
    match op {
        STATUS | GENERATE | REVOKE | EXPORT_RECOVERY => Some((0, 0)),
        SEAL => Some((0, MAX_PLAINTEXT)),
        UNSEAL => Some((24 + 16, MAX_SEALED)),
        RESTORE => Some((SECRET_BYTES + RECOVERY_BLOB, SECRET_BYTES + RECOVERY_BLOB)),
        _ => None,
    }
}

/// STATUS reply body: state, reason, keys per domain (2), entropy bytes (8).
pub fn status_body(state: u8, reason: u8, keys: [u16; 2], entropy_bytes: u64) -> [u8; 14] {
    let mut body = [0u8; 14];
    body[0] = state;
    body[1] = reason;
    body[2..4].copy_from_slice(&keys[0].to_le_bytes());
    body[4..6].copy_from_slice(&keys[1].to_le_bytes());
    body[6..14].copy_from_slice(&entropy_bytes.to_le_bytes());
    body
}
