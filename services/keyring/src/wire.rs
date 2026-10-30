//! IPC v1 of the Key Service (tag `KEY1`). Requests and replies travel in one
//! 256-byte payload: a 16-byte header plus a bounded body. Lengths are
//! validated before any byte is interpreted; the domain is never carried on
//! the wire because the ingress endpoint already fixes it.
use zeroize::Zeroizing;

use crate::{
    persistent::{SnapshotInfo, PERSISTENT_BLOB},
    records::{RecordContext, RecordKind, RECORD_SUITE, RECORD_VERSION},
    EntropySource, Service, Status, DOMAINS, MAX_PLAINTEXT, MAX_SEALED, NONCE_BYTES, RECOVERY_BLOB,
    SECRET_BYTES, TAG_BYTES,
};

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
pub const SEAL_RECORD: u8 = 8;
pub const UNSEAL_RECORD: u8 = 9;
pub const RANDOM: u8 = 10;
/// B02 persistence opcodes. They are only reachable through a separately
/// granted administrative ingress; domain endpoints and the RANDOM ingress
/// reject them. The secret in the body is caller-held and never stored.
pub const EXPORT_PERSISTENT: u8 = 11;
pub const RESTORE_PERSISTENT: u8 = 12;
pub const RANDOM_PREFIX: usize = 16;
pub const REPLY: u8 = 0x80;
pub const RECORD_CONTEXT: usize = 64;
pub const VOLUME_ID: usize = 16;
/// Nonsecret snapshot information: generation (u64), then created and
/// revoked counters (u16 each) per domain.
pub const SNAPSHOT_INFO: usize = 8 + DOMAINS * 4;
/// EXPORT body: volume id, caller secret. Reply: blob, snapshot info.
pub const EXPORT_PERSISTENT_BODY: usize = VOLUME_ID + SECRET_BYTES;
pub const EXPORT_PERSISTENT_REPLY: usize = PERSISTENT_BLOB + SNAPSHOT_INFO;
/// RESTORE body: volume id, caller secret, minimum generation (u64), blob.
pub const RESTORE_PERSISTENT_BODY: usize = VOLUME_ID + SECRET_BYTES + 8 + PERSISTENT_BLOB;

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
        SEAL_RECORD => Some((RECORD_CONTEXT, RECORD_CONTEXT + MAX_PLAINTEXT)),
        UNSEAL_RECORD => Some((
            RECORD_CONTEXT + NONCE_BYTES + TAG_BYTES,
            RECORD_CONTEXT + MAX_SEALED,
        )),
        RANDOM => Some((RANDOM_PREFIX, RANDOM_PREFIX)),
        EXPORT_PERSISTENT => Some((EXPORT_PERSISTENT_BODY, EXPORT_PERSISTENT_BODY)),
        RESTORE_PERSISTENT => Some((RESTORE_PERSISTENT_BODY, RESTORE_PERSISTENT_BODY)),
        _ => None,
    }
}

/// RANDOM v1 has no caller-selected principal, seed or crypto/key operation.
/// The eight reserved bytes must remain zero. Its authority comes exclusively
/// from a separately granted RANDOM-only ingress, not these requested bytes.
pub fn random_request(request_id: u64, length: usize) -> Result<[u8; SIZE], Status> {
    if request_id == 0 || !(1..=crate::random::MAX_BYTES).contains(&length) {
        return Err(Status::Invalid);
    }
    let mut body = [0u8; RANDOM_PREFIX];
    body[..4].copy_from_slice(b"RND1");
    body[4..6].copy_from_slice(&crate::random::VERSION.to_le_bytes());
    body[6..8].copy_from_slice(&(length as u16).to_le_bytes());
    request(RANDOM, 0, request_id, &body).ok_or(Status::Invalid)
}

pub fn random_length(header: &Header, body: &[u8]) -> Result<usize, Status> {
    if header.op != RANDOM
        || header.index != 0
        || header.status != 0
        || header.request == 0
        || usize::from(header.length) != RANDOM_PREFIX
        || body.len() != RANDOM_PREFIX
        || &body[..4] != b"RND1"
        || body[4..6] != crate::random::VERSION.to_le_bytes()
        || body[8..].iter().any(|byte| *byte != 0)
    {
        return Err(Status::Invalid);
    }
    let length = usize::from(u16::from_le_bytes([body[6], body[7]]));
    if !(1..=crate::random::MAX_BYTES).contains(&length) {
        return Err(Status::Invalid);
    }
    Ok(length)
}

/// Separate dispatch for the native RANDOM-only ingress. Passing a legacy
/// opcode here never invokes key generation, sealing, recovery or export.
pub fn dispatch_random(
    service: &mut Service,
    source: &mut impl EntropySource,
    domain: usize,
    now: u64,
    header: &Header,
    body: &[u8],
) -> [u8; SIZE] {
    let length = match random_length(header, body) {
        Ok(length) => length,
        Err(status) => return reply(header, status, &[]),
    };
    let mut bytes = [0u8; crate::random::MAX_BYTES];
    let result = match service.random_at(domain, header.request, length, now, source, &mut bytes) {
        Ok(length) => {
            let mut body_out = [0u8; BODY];
            body_out[..RANDOM_PREFIX].copy_from_slice(body);
            body_out[RANDOM_PREFIX..RANDOM_PREFIX + length].copy_from_slice(&bytes[..length]);
            let answer = reply(header, Status::Ok, &body_out[..RANDOM_PREFIX + length]);
            crate::random::clear_bytes(&mut body_out);
            answer
        }
        Err(status) => reply(header, status, &[]),
    };
    crate::random::clear_bytes(&mut bytes);
    result
}

/// Canonical context prefix for record requests. There is deliberately no
/// domain field: even a sender that edits these bytes cannot select its caller
/// principal. The receive endpoint determines that field during decode.
pub fn encode_record_context(context: RecordContext) -> Result<[u8; RECORD_CONTEXT], Status> {
    context.validate(usize::from(context.domain))?;
    let mut out = [0; RECORD_CONTEXT];
    out[..8].copy_from_slice(b"NOSCTX01");
    out[8..10].copy_from_slice(&RECORD_VERSION.to_le_bytes());
    out[10..12].copy_from_slice(&RECORD_SUITE.to_le_bytes());
    out[12] = context.kind as u8;
    out[16..32].copy_from_slice(&context.volume);
    out[32..40].copy_from_slice(&context.object.to_le_bytes());
    out[40..48].copy_from_slice(&context.revision.to_le_bytes());
    out[48..52].copy_from_slice(&context.chunk.to_le_bytes());
    out[52..54].copy_from_slice(&context.length.to_le_bytes());
    out[54..56].copy_from_slice(&context.schema.to_le_bytes());
    Ok(out)
}

pub fn decode_record_context(domain: usize, bytes: &[u8]) -> Result<RecordContext, Status> {
    if domain >= DOMAINS
        || bytes.len() != RECORD_CONTEXT
        || &bytes[..8] != b"NOSCTX01"
        || bytes[8..10] != RECORD_VERSION.to_le_bytes()
        || bytes[10..12] != RECORD_SUITE.to_le_bytes()
        || bytes[13..16].iter().any(|byte| *byte != 0)
        || bytes[56..].iter().any(|byte| *byte != 0)
    {
        return Err(Status::Invalid);
    }
    let context = RecordContext {
        volume: bytes[16..32].try_into().map_err(|_| Status::Invalid)?,
        domain: domain as u8,
        object: u64::from_le_bytes(bytes[32..40].try_into().map_err(|_| Status::Invalid)?),
        revision: u64::from_le_bytes(bytes[40..48].try_into().map_err(|_| Status::Invalid)?),
        chunk: u32::from_le_bytes(bytes[48..52].try_into().map_err(|_| Status::Invalid)?),
        length: u16::from_le_bytes([bytes[52], bytes[53]]),
        kind: RecordKind::from_code(bytes[12]).ok_or(Status::Invalid)?,
        schema: u16::from_le_bytes([bytes[54], bytes[55]]),
    };
    context.validate(domain)?;
    Ok(context)
}

/// Build a bounded record request without exposing snapshot/recovery authority.
/// The context is only requested data; a native broker must also authorize the
/// volume/object/action before granting access to its Key Service endpoint.
pub fn record_request(
    op: u8,
    index: u8,
    request_id: u64,
    context: RecordContext,
    data: &[u8],
) -> Result<[u8; SIZE], Status> {
    let expected = match op {
        SEAL_RECORD => usize::from(context.length),
        UNSEAL_RECORD => NONCE_BYTES + usize::from(context.length) + TAG_BYTES,
        _ => return Err(Status::Invalid),
    };
    if data.len() != expected {
        return Err(Status::Invalid);
    }
    let encoded = encode_record_context(context)?;
    let mut body = [0; BODY];
    let length = RECORD_CONTEXT + data.len();
    // encode_record_context validates the 128-byte plaintext ceiling, so both
    // operations fit within BODY (192 and 232 bytes respectively).
    body[..RECORD_CONTEXT].copy_from_slice(&encoded);
    body[RECORD_CONTEXT..length].copy_from_slice(data);
    request(op, index, request_id, &body[..length]).ok_or(Status::Invalid)
}

/// Record-only dispatch shared with the actual native service adapter. The
/// adapter supplies an authenticated ingress domain after checking sender,
/// message tag, payload size and absence of delegated capabilities. This
/// helper does not grant object rights and implements no persistence opcodes.
pub fn dispatch_record(
    service: &mut Service,
    source: &mut impl EntropySource,
    domain: usize,
    header: &Header,
    body: &[u8],
) -> [u8; SIZE] {
    let bounds = match header.op {
        SEAL_RECORD | UNSEAL_RECORD => body_bounds(header.op),
        _ => None,
    };
    let Some((low, high)) = bounds else {
        return reply(header, Status::Invalid, &[]);
    };
    if header.status != 0
        || usize::from(header.length) != body.len()
        || body.len() < low
        || body.len() > high
    {
        return reply(header, Status::Invalid, &[]);
    }
    let context = match decode_record_context(domain, &body[..RECORD_CONTEXT]) {
        Ok(context) => context,
        Err(status) => return reply(header, status, &[]),
    };
    let data = &body[RECORD_CONTEXT..];
    if header.op == SEAL_RECORD {
        let mut out = [0; MAX_SEALED];
        match service.seal_record(domain, header.index, context, data, source, &mut out) {
            Ok(length) => reply(header, Status::Ok, &out[..length]),
            Err(status) => reply(header, status, &[]),
        }
    } else {
        let mut out = [0; MAX_PLAINTEXT];
        match service.unseal_record(domain, header.index, context, data, source, &mut out) {
            Ok(length) => reply(header, Status::Ok, &out[..length]),
            Err(status) => reply(header, status, &[]),
        }
    }
}

pub fn encode_snapshot_info(info: SnapshotInfo) -> [u8; SNAPSHOT_INFO] {
    let mut out = [0u8; SNAPSHOT_INFO];
    out[..8].copy_from_slice(&info.generation.to_le_bytes());
    for domain in 0..DOMAINS {
        let at = 8 + domain * 4;
        out[at..at + 2].copy_from_slice(&info.created[domain].to_le_bytes());
        out[at + 2..at + 4].copy_from_slice(&info.revoked[domain].to_le_bytes());
    }
    out
}

pub fn decode_snapshot_info(bytes: &[u8]) -> Option<SnapshotInfo> {
    if bytes.len() != SNAPSHOT_INFO {
        return None;
    }
    let mut info = SnapshotInfo {
        generation: u64::from_le_bytes(bytes[..8].try_into().ok()?),
        created: [0; DOMAINS],
        revoked: [0; DOMAINS],
    };
    for domain in 0..DOMAINS {
        let at = 8 + domain * 4;
        info.created[domain] = u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        info.revoked[domain] = u16::from_le_bytes([bytes[at + 2], bytes[at + 3]]);
    }
    Some(info)
}

/// Build an EXPORT_PERSISTENT request. The caller owns `secret` and must wipe
/// its own copy of the returned message after sending it.
pub fn export_persistent_request(
    request_id: u64,
    volume: &[u8; VOLUME_ID],
    secret: &[u8; SECRET_BYTES],
) -> Option<[u8; SIZE]> {
    let mut body = Zeroizing::new([0u8; EXPORT_PERSISTENT_BODY]);
    body[..VOLUME_ID].copy_from_slice(volume);
    body[VOLUME_ID..].copy_from_slice(secret);
    request(EXPORT_PERSISTENT, 0, request_id, &*body)
}

/// Build a RESTORE_PERSISTENT request. `minimum_generation` must come from an
/// independently trusted source; zero accepts any authentic snapshot.
pub fn restore_persistent_request(
    request_id: u64,
    volume: &[u8; VOLUME_ID],
    secret: &[u8; SECRET_BYTES],
    minimum_generation: u64,
    blob: &[u8; PERSISTENT_BLOB],
) -> Option<[u8; SIZE]> {
    let mut body = Zeroizing::new([0u8; RESTORE_PERSISTENT_BODY]);
    let secret_end = VOLUME_ID + SECRET_BYTES;
    body[..VOLUME_ID].copy_from_slice(volume);
    body[VOLUME_ID..secret_end].copy_from_slice(secret);
    body[secret_end..secret_end + 8].copy_from_slice(&minimum_generation.to_le_bytes());
    body[secret_end + 8..].copy_from_slice(blob);
    request(RESTORE_PERSISTENT, 0, request_id, &*body)
}

/// Persistence-only dispatch for the separately granted administrative
/// ingress. It never serves key, record, recovery or RANDOM opcodes, and the
/// legacy/RANDOM dispatchers never reach `export_persistent`/`restore_persistent`.
/// Every format check precedes the cryptographic work; a failed call leaves the
/// namespace unchanged (see `Service::restore_persistent`).
pub fn dispatch_persistent(
    service: &mut Service,
    source: &mut impl EntropySource,
    header: &Header,
    body: &[u8],
) -> [u8; SIZE] {
    let bounds = match header.op {
        EXPORT_PERSISTENT | RESTORE_PERSISTENT => body_bounds(header.op),
        _ => None,
    };
    let Some((low, high)) = bounds else {
        return reply(header, Status::Invalid, &[]);
    };
    if header.index != 0
        || header.status != 0
        || usize::from(header.length) != body.len()
        || body.len() < low
        || body.len() > high
    {
        return reply(header, Status::Invalid, &[]);
    }
    let mut volume = [0u8; VOLUME_ID];
    volume.copy_from_slice(&body[..VOLUME_ID]);
    let mut secret = Zeroizing::new([0u8; SECRET_BYTES]);
    secret.copy_from_slice(&body[VOLUME_ID..VOLUME_ID + SECRET_BYTES]);
    if header.op == EXPORT_PERSISTENT {
        let mut out = [0u8; EXPORT_PERSISTENT_REPLY];
        let (blob, info) = out.split_at_mut(PERSISTENT_BLOB);
        let blob: &mut [u8; PERSISTENT_BLOB] = blob.try_into().expect("exact blob length");
        match service.export_persistent(&volume, &secret, source, blob) {
            Ok(snapshot) => {
                info.copy_from_slice(&encode_snapshot_info(snapshot));
                reply(header, Status::Ok, &out)
            }
            Err(status) => reply(header, status, &[]),
        }
    } else {
        let at = VOLUME_ID + SECRET_BYTES;
        let minimum = u64::from_le_bytes(body[at..at + 8].try_into().expect("eight bytes"));
        match service.restore_persistent(&volume, &secret, &body[at + 8..], minimum, source) {
            Ok(snapshot) => reply(header, Status::Ok, &encode_snapshot_info(snapshot)),
            Err(status) => reply(header, status, &[]),
        }
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
