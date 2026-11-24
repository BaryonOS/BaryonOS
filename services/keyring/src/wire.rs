//! IPC v1 of the Key Service (tag `KEY1`). Requests and replies travel in one
//! 256-byte payload: a 16-byte header plus a bounded body. Lengths are
//! validated before any byte is interpreted; the domain is never carried on
//! the wire because the ingress endpoint already fixes it.
use zeroize::Zeroizing;

use crate::{
    persistent::{KeyStateStore, SnapshotInfo, StoreError, WriteAheadError, PERSISTENT_BLOB},
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
/// reject them. The secret in their body is caller-held and never stored.
pub const EXPORT_PERSISTENT: u8 = 11;
pub const RESTORE_PERSISTENT: u8 = 12;
/// ADR-B02-KEY1-RELAY (test profile): administrative-only supply of the
/// external secret, once per instance. The service holds it only in zeroizing
/// memory, never returns, logs, exports or persists it. Not in `body_bounds`,
/// so every domain, persistent and management dispatch replies `Invalid`.
pub const SUPPLY_SECRET: u8 = 13;
/// Key Service -> init commit request on the key-state commit ingress. Never
/// valid on any Key Service ingress (not in `body_bounds`).
pub const COMMIT_KEYSTATE: u8 = 14;
/// Bound of one relay commit in 100 Hz ticks (15 s): 3x the volume service's
/// 500-tick completion budget, below the kernel's 20,000-tick system bound.
pub const COMMIT_TICKS: u64 = 1500;
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

/// Ordinary app ingress. Recovery export/import and persistent snapshots are
/// deliberately excluded; only the separately authorized management ingress
/// may dispatch those operations. GENERATE and REVOKE here are the legacy,
/// non-durable operations and are refused with `Invalid` once the instance is
/// in write-ahead mode; `dispatch_domain_durable` is the write-ahead variant.
pub fn dispatch_domain(
    service: &mut Service,
    source: &mut impl EntropySource,
    domain: usize,
    header: &Header,
    body: &[u8],
) -> [u8; SIZE] {
    let Some((low, high)) = body_bounds(header.op) else {
        return reply(header, Status::Invalid, &[]);
    };
    if header.status != 0
        || usize::from(header.length) != body.len()
        || body.len() < low
        || body.len() > high
    {
        return reply(header, Status::Invalid, &[]);
    }
    match header.op {
        STATUS => {
            let status = status_body(
                service.state() as u8,
                service.reason() as u8,
                [service.keys_created(0), service.keys_created(1)],
                service.counters.entropy_bytes,
            );
            reply(header, Status::Ok, &status)
        }
        GENERATE => match service.generate(domain, source) {
            Ok(index) => reply(header, Status::Ok, &[index]),
            Err(status) => reply(header, status, &[]),
        },
        SEAL => {
            let mut out = [0u8; MAX_SEALED];
            match service.seal(domain, header.index, body, source, &mut out) {
                Ok(length) => reply(header, Status::Ok, &out[..length]),
                Err(status) => reply(header, status, &[]),
            }
        }
        UNSEAL => {
            let mut out = [0u8; MAX_PLAINTEXT];
            match service.unseal(domain, header.index, body, source, &mut out) {
                Ok(length) => reply(header, Status::Ok, &out[..length]),
                Err(status) => reply(header, status, &[]),
            }
        }
        SEAL_RECORD | UNSEAL_RECORD => dispatch_record(service, source, domain, header, body),
        REVOKE => match service.revoke(domain, header.index) {
            Ok(()) => reply(header, Status::Ok, &[]),
            Err(status) => reply(header, status, &[]),
        },
        // Recovery and persistence are management-only, even though their
        // request formats remain part of KEY1 for compatible clients.
        EXPORT_RECOVERY | RESTORE | EXPORT_PERSISTENT | RESTORE_PERSISTENT => {
            reply(header, Status::Invalid, &[])
        }
        _ => reply(header, Status::Invalid, &[]),
    }
}

/// Write-ahead context of one domain ingress, held by the service adapter
/// from its own configuration. Never built from request bytes.
pub struct Durable<'a, S: KeyStateStore> {
    pub volume: &'a [u8; VOLUME_ID],
    pub secret: &'a [u8; SECRET_BYTES],
    pub store: &'a mut S,
}

/// Status for a write-ahead failure: `Failed` is certainly not durable and an
/// explicit retry is allowed (`NotReady`); `Unknown` has latched the instance
/// (`Degraded`); a key-service refusal keeps its own status. An adapter must
/// report an IPC timeout or ambiguous transport result as `StoreError::Unknown`.
pub const fn durable_status(error: WriteAheadError) -> Status {
    match error {
        WriteAheadError::Key(status) => status,
        WriteAheadError::Failed => Status::NotReady,
        WriteAheadError::Unknown => Status::Degraded,
    }
}

/// Domain ingress whose GENERATE and REVOKE are write-ahead: `Ok` is replied
/// only after `durable.store` committed the snapshot in this call. Every other
/// opcode is exactly `dispatch_domain`. Never falls back to the legacy
/// generate/revoke, and never serves persistence or recovery opcodes.
pub fn dispatch_domain_durable<S: KeyStateStore>(
    service: &mut Service,
    source: &mut impl EntropySource,
    domain: usize,
    header: &Header,
    body: &[u8],
    durable: &mut Durable<'_, S>,
) -> [u8; SIZE] {
    if !matches!(header.op, GENERATE | REVOKE) {
        return dispatch_domain(service, source, domain, header, body);
    }
    if header.status != 0 || usize::from(header.length) != body.len() || !body.is_empty() {
        return reply(header, Status::Invalid, &[]);
    }
    if header.op == GENERATE {
        match service.generate_durable(
            domain,
            durable.volume,
            durable.secret,
            durable.store,
            source,
        ) {
            Ok(index) => reply(header, Status::Ok, &[index]),
            Err(error) => reply(header, durable_status(error), &[]),
        }
    } else {
        match service.revoke_durable(
            domain,
            header.index,
            durable.volume,
            durable.secret,
            durable.store,
            source,
        ) {
            Ok(()) => reply(header, Status::Ok, &[]),
            Err(error) => reply(header, durable_status(error), &[]),
        }
    }
}

/// External secret held by a relay-mode service instance. No `Clone`, no
/// `Debug`; the bytes are wiped when the value is dropped.
pub struct HeldSecret(Zeroizing<[u8; SECRET_BYTES]>);

impl HeldSecret {
    pub fn secret(&self) -> &[u8; SECRET_BYTES] {
        &self.0
    }
}

/// Administrative SUPPLY_SECRET request. The caller wipes its own copy.
pub fn supply_secret_request(request_id: u64, secret: &[u8; SECRET_BYTES]) -> [u8; SIZE] {
    request(SUPPLY_SECRET, 0, request_id, secret).expect("32-byte body fits")
}

pub fn decode_supply_secret(header: &Header, body: &[u8]) -> Result<HeldSecret, Status> {
    if header.op != SUPPLY_SECRET
        || header.index != 0
        || header.status != 0
        || usize::from(header.length) != SECRET_BYTES
        || body.len() != SECRET_BYTES
    {
        return Err(Status::Invalid);
    }
    let mut held = Zeroizing::new([0u8; SECRET_BYTES]);
    held.copy_from_slice(body);
    Ok(HeldSecret(held))
}

/// Commit request to init: the request id is the generation (never 0).
pub fn commit_request(generation: u64, blob: &[u8; PERSISTENT_BLOB]) -> Option<[u8; SIZE]> {
    if generation == 0 {
        return None;
    }
    request(COMMIT_KEYSTATE, 0, generation, blob)
}

/// Init side: a canonical COMMIT_KEYSTATE request, or `None`.
pub fn decode_commit_request(payload: &[u8]) -> Option<(u64, [u8; PERSISTENT_BLOB])> {
    let header = Header::decode(payload)?;
    if header.op != COMMIT_KEYSTATE
        || header.index != 0
        || header.status != 0
        || header.request == 0
        || usize::from(header.length) != PERSISTENT_BLOB
    {
        return None;
    }
    let mut blob = [0u8; PERSISTENT_BLOB];
    blob.copy_from_slice(&payload[HEADER..HEADER + PERSISTENT_BLOB]);
    Some((header.request, blob))
}

/// Init side: `COMMITTED` (Ok) or `NOT_COMMITTED` (NotReady) for a request.
pub fn commit_reply(request_header: &Header, committed: bool) -> [u8; SIZE] {
    let status = if committed {
        Status::Ok
    } else {
        Status::NotReady
    };
    reply(request_header, status, &[])
}

/// Service side: map init's reply to the store outcome. Only a canonical,
/// empty reply to this generation's COMMIT_KEYSTATE counts; `NotReady` is the
/// only certain failure, everything else (including malformed) is `Unknown`.
pub fn commit_outcome(generation: u64, payload: &[u8]) -> Result<(), StoreError> {
    let Some(header) = Header::decode(payload) else {
        return Err(StoreError::Unknown);
    };
    if header.op != COMMIT_KEYSTATE | REPLY
        || header.index != 0
        || header.request != generation
        || generation == 0
        || header.length != 0
    {
        return Err(StoreError::Unknown);
    }
    match header.status {
        0 => Ok(()),
        status if status == Status::NotReady.code() => Err(StoreError::Failed),
        _ => Err(StoreError::Unknown),
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
/// ingress. It never serves key, record, recovery or RANDOM opcodes.
/// `dispatch_domain`, `dispatch_domain_durable` and `dispatch_random` never call
/// `export_persistent`/`restore_persistent` or return a snapshot blob; in the
/// durable dispatch the blob goes only to `KeyStateStore::commit`.
/// Every format check precedes the cryptographic work. A refused call leaves
/// the key namespace and the volume binding unchanged (a failed entropy reseed
/// can still degrade the service); a successful EXPORT or RESTORE binds the
/// instance to the request's volume. In write-ahead mode EXPORT is refused
/// (`NotReady`) until state is settled and RESTORE is refused (`Invalid`).
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

/// Dispatch key recovery or persistent snapshots for a management ingress.
/// The service adapter must authenticate the endpoint sender against its
/// kernel manifest before calling this function.
pub fn dispatch_management(
    service: &mut Service,
    source: &mut impl EntropySource,
    header: &Header,
    body: &[u8],
) -> [u8; SIZE] {
    if matches!(header.op, EXPORT_PERSISTENT | RESTORE_PERSISTENT) {
        return dispatch_persistent(service, source, header, body);
    }
    let Some((low, high)) = (match header.op {
        EXPORT_RECOVERY | RESTORE => body_bounds(header.op),
        _ => None,
    }) else {
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
    if header.op == EXPORT_RECOVERY {
        let mut secret = Zeroizing::new([0u8; SECRET_BYTES]);
        let mut blob = Zeroizing::new([0u8; RECOVERY_BLOB]);
        match service.export_recovery(source, &mut secret, &mut blob) {
            Ok(()) => {
                let mut out = Zeroizing::new([0u8; SECRET_BYTES + RECOVERY_BLOB]);
                out[..SECRET_BYTES].copy_from_slice(&*secret);
                out[SECRET_BYTES..].copy_from_slice(&*blob);
                reply(header, Status::Ok, &*out)
            }
            Err(status) => reply(header, status, &[]),
        }
    } else {
        let mut secret = Zeroizing::new([0u8; SECRET_BYTES]);
        secret.copy_from_slice(&body[..SECRET_BYTES]);
        let blob: &[u8; RECOVERY_BLOB] = body[SECRET_BYTES..]
            .try_into()
            .expect("validated recovery body length");
        match service.restore(&secret, blob, source) {
            Ok(domains) => reply(header, Status::Ok, &[domains as u8]),
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

/// ADR-B02-KEY1-RELAY host checks of the relay wire and its store mapping.
/// Deterministic entropy is a test fixture; no native IPC is exercised.
#[cfg(test)]
mod relay_tests {
    use super::*;

    const VOLUME: [u8; VOLUME_ID] = *b"NOS-SYSTEM-VOL01";
    const SECRET: [u8; SECRET_BYTES] = [0x5a; SECRET_BYTES];

    struct Entropy(u64);
    impl EntropySource for Entropy {
        fn read(&mut self, out: &mut [u8]) -> Result<(), Status> {
            for byte in out {
                self.0 ^= self.0 << 13;
                self.0 ^= self.0 >> 7;
                self.0 ^= self.0 << 17;
                *byte = (self.0 >> 24) as u8;
            }
            Ok(())
        }
    }

    /// Scripted init: each commit pops the next outcome; records generations.
    struct Script {
        outcomes: [Result<(), StoreError>; 4],
        calls: usize,
        generations: [u64; 4],
    }
    impl KeyStateStore for Script {
        fn commit(
            &mut self,
            generation: u64,
            blob: &[u8; PERSISTENT_BLOB],
        ) -> Result<(), StoreError> {
            // The blob travels through the real relay wire both ways.
            let request = commit_request(generation, blob).expect("nonzero generation");
            let (seen, carried) = decode_commit_request(&request).expect("canonical request");
            assert_eq!((seen, &carried), (generation, blob));
            self.generations[self.calls] = generation;
            let outcome = self.outcomes[self.calls];
            self.calls += 1;
            let header = Header::decode(&request).unwrap();
            let answer = match outcome {
                Ok(()) => commit_reply(&header, true),
                Err(StoreError::Failed) => commit_reply(&header, false),
                Err(StoreError::Unknown) => reply(&header, Status::Degraded, &[]),
            };
            commit_outcome(generation, &answer)
        }
    }

    fn ready() -> (Service, Entropy) {
        let mut service = Service::new();
        let mut source = Entropy(0x1234_5678_9abc_def1);
        service.start(&mut source).unwrap();
        (service, source)
    }

    fn call(
        service: &mut Service,
        source: &mut Entropy,
        store: &mut Script,
        op: u8,
        index: u8,
    ) -> Header {
        let request = request(op, index, 7, &[]).unwrap();
        let header = Header::decode(&request).unwrap();
        let mut durable = Durable {
            volume: &VOLUME,
            secret: &SECRET,
            store,
        };
        let answer = dispatch_domain_durable(service, source, 0, &header, &[], &mut durable);
        Header::decode(&answer).unwrap()
    }

    #[test]
    fn relay_opcodes_are_refused_by_every_service_dispatch_without_state_change() {
        let (mut service, mut source) = ready();
        for op in [SUPPLY_SECRET, COMMIT_KEYSTATE] {
            assert_eq!(body_bounds(op), None);
            let bodies: [&[u8]; 2] = [&SECRET, &[0u8; PERSISTENT_BLOB]];
            for body in bodies {
                let message = request(op, 0, 9, body).unwrap();
                let header = Header::decode(&message).unwrap();
                let before = (service.snapshot_info(), service.counters.entropy_bytes);
                for answer in [
                    dispatch_domain(&mut service, &mut source, 0, &header, body),
                    dispatch_persistent(&mut service, &mut source, &header, body),
                    dispatch_management(&mut service, &mut source, &header, body),
                ] {
                    let decoded = Header::decode(&answer).unwrap();
                    assert_eq!(decoded.status, Status::Invalid.code());
                    assert_eq!(decoded.length, 0);
                }
                assert_eq!(
                    (service.snapshot_info(), service.counters.entropy_bytes),
                    before
                );
            }
        }
    }

    #[test]
    fn supply_decode_refuses_noncanonical_requests() {
        let good = supply_secret_request(3, &SECRET);
        let header = Header::decode(&good).unwrap();
        let held = decode_supply_secret(&header, &good[HEADER..HEADER + SECRET_BYTES]).unwrap();
        assert_eq!(held.secret(), &SECRET);
        let body = &good[HEADER..HEADER + SECRET_BYTES];
        for bad in [
            Header {
                length: 31,
                ..header
            },
            Header { index: 1, ..header },
            Header {
                status: -1,
                ..header
            },
            Header {
                op: RESTORE_PERSISTENT,
                ..header
            },
        ] {
            assert_eq!(
                decode_supply_secret(&bad, body).err(),
                Some(Status::Invalid)
            );
        }
        assert_eq!(
            decode_supply_secret(&header, &body[..31]).err(),
            Some(Status::Invalid)
        );
        // A dirty tail never decodes to a header in the first place.
        let mut dirty = good;
        dirty[SIZE - 1] = 1;
        assert!(Header::decode(&dirty).is_none());
    }

    #[test]
    fn commit_request_round_trips_and_refuses_zero_or_dirty() {
        let blob = core::array::from_fn::<u8, PERSISTENT_BLOB, _>(|i| i as u8);
        let message = commit_request(4, &blob).unwrap();
        assert_eq!(decode_commit_request(&message), Some((4, blob)));
        assert!(commit_request(0, &blob).is_none());
        let mut dirty = message;
        dirty[HEADER + PERSISTENT_BLOB] = 1;
        assert!(decode_commit_request(&dirty).is_none());
        for (at, value) in [(0usize, SUPPLY_SECRET), (1, 1), (12, 0xff)] {
            let mut bad = message;
            bad[at] = value;
            assert!(decode_commit_request(&bad).is_none(), "byte {at}");
        }
        let short = request(COMMIT_KEYSTATE, 0, 4, &blob[..119]).unwrap();
        assert!(decode_commit_request(&short).is_none());
        let zero = request(COMMIT_KEYSTATE, 0, 0, &blob).unwrap();
        assert!(decode_commit_request(&zero).is_none());
    }

    #[test]
    fn commit_outcome_maps_only_ok_and_not_ready_to_certain_results() {
        let blob = [7u8; PERSISTENT_BLOB];
        let header = Header::decode(&commit_request(5, &blob).unwrap()).unwrap();
        assert_eq!(commit_outcome(5, &commit_reply(&header, true)), Ok(()));
        assert_eq!(
            commit_outcome(5, &commit_reply(&header, false)),
            Err(StoreError::Failed)
        );
        for status in [Status::Degraded, Status::Invalid, Status::Auth] {
            assert_eq!(
                commit_outcome(5, &reply(&header, status, &[])),
                Err(StoreError::Unknown)
            );
        }
        // Wrong request id (another generation), wrong op, a body, garbage.
        assert_eq!(
            commit_outcome(6, &commit_reply(&header, true)),
            Err(StoreError::Unknown)
        );
        assert_eq!(
            commit_outcome(
                0,
                &commit_reply(
                    &Header {
                        request: 0,
                        ..header
                    },
                    true
                )
            ),
            Err(StoreError::Unknown)
        );
        let wrong_op = Header {
            op: GENERATE,
            ..header
        };
        assert_eq!(
            commit_outcome(5, &commit_reply(&wrong_op, true)),
            Err(StoreError::Unknown)
        );
        let with_body = reply(&header, Status::Ok, &[1]);
        assert_eq!(commit_outcome(5, &with_body), Err(StoreError::Unknown));
        assert_eq!(commit_outcome(5, &[0xff; SIZE]), Err(StoreError::Unknown));
        assert_eq!(commit_outcome(5, &[0; 16]), Err(StoreError::Unknown));
    }

    #[test]
    fn failed_commit_replies_not_ready_and_a_retry_commits() {
        let (mut service, mut source) = ready();
        let mut store = Script {
            outcomes: [
                Err(StoreError::Failed),
                Ok(()),
                Err(StoreError::Failed),
                Ok(()),
            ],
            calls: 0,
            generations: [0; 4],
        };
        let first = call(&mut service, &mut source, &mut store, GENERATE, 0);
        assert_eq!(first.status, Status::NotReady.code());
        // GENERATE allocates again: the next generation with a new index.
        let second = call(&mut service, &mut source, &mut store, GENERATE, 0);
        assert_eq!(second.status, Status::Ok.code());
        assert_eq!(store.generations[..2], [1, 2]);
        // REVOKE retried after NOT_COMMITTED re-commits the same generation.
        let third = call(&mut service, &mut source, &mut store, REVOKE, 1);
        assert_eq!(third.status, Status::NotReady.code());
        let fourth = call(&mut service, &mut source, &mut store, REVOKE, 1);
        assert_eq!(fourth.status, Status::Ok.code());
        assert_eq!(store.generations, [1, 2, 3, 3]);
        assert!(!service.write_ahead_uncertain());
    }

    #[test]
    fn unknown_commit_latches_degraded_without_further_store_calls() {
        let (mut service, mut source) = ready();
        let mut store = Script {
            outcomes: [Ok(()), Err(StoreError::Unknown), Ok(()), Ok(())],
            calls: 0,
            generations: [0; 4],
        };
        let ok = call(&mut service, &mut source, &mut store, GENERATE, 0);
        assert_eq!((ok.status, ok.length), (Status::Ok.code(), 1));
        assert_eq!(store.calls, 1);
        let unknown = call(&mut service, &mut source, &mut store, GENERATE, 0);
        assert_eq!(unknown.status, Status::Degraded.code());
        assert_eq!(store.calls, 2);
        for (op, index) in [(GENERATE, 0), (REVOKE, 0)] {
            let answer = call(&mut service, &mut source, &mut store, op, index);
            assert_eq!(answer.status, Status::Degraded.code());
        }
        assert_eq!(store.calls, 2);
        assert!(service.write_ahead_uncertain());
    }

    #[test]
    fn first_bind_fixes_the_relay_volume() {
        let (mut service, mut source) = ready();
        let mut store = Script {
            outcomes: [Ok(()); 4],
            calls: 0,
            generations: [0; 4],
        };
        assert_eq!(
            call(&mut service, &mut source, &mut store, GENERATE, 0).status,
            0
        );
        let other = *b"other-volume-id!";
        let mut durable = Durable {
            volume: &other,
            secret: &SECRET,
            store: &mut store,
        };
        let message = request(GENERATE, 0, 8, &[]).unwrap();
        let header = Header::decode(&message).unwrap();
        let answer =
            dispatch_domain_durable(&mut service, &mut source, 0, &header, &[], &mut durable);
        assert_eq!(
            Header::decode(&answer).unwrap().status,
            Status::Invalid.code()
        );
        assert_eq!(store.calls, 1);
    }
}
