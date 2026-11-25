#![no_std]
#![forbid(unsafe_code)]
//! A03 grant broker fixture: a fixed wire codec plus the broker decision.
//! The broker is the trusted adapter of `nieuw-os-task-grant-core`: the only
//! identity it accepts is the kernel-supplied sender PID of each IPC message.
//! The request's claimed actor field exists solely to prove it is ignored.

use nieuw_os_task_grant_core::{Actor, Duration, Error, Grants, Handle, Object, Operation, Rights};

/// Message tag of every broker request and reply ("A3TG").
pub const TAG: u64 = 0x4133_5447;
/// Request: operation, object id, claimed actor PID (three little-endian u64).
pub const REQUEST_SIZE: usize = 24;
/// Reply: status, data length, then at most `DATA_MAX` data bytes.
pub const REPLY_SIZE: usize = 32;
pub const DATA_MAX: usize = 16;

pub const OP_READ: u64 = 1;
pub const OP_WRITE: u64 = 2;
pub const OP_CLOSE: u64 = 3;

pub const STATUS_OK: u64 = 1;
pub const STATUS_DENIED: u64 = 2;
pub const STATUS_BAD_REQUEST: u64 = 3;
/// Broker-to-helper ordering signal: the owner's first phase is served.
pub const STATUS_GO: u64 = 4;
/// Broker-to-owner ordering signal: the helper's phase is served.
pub const STATUS_RESUME: u64 = 5;

pub const OBJECT_ONE: u64 = 1;
pub const OBJECT_TWO: u64 = 2;
pub const REVISION: u64 = 1;
/// Fixed, known content of object 1.
pub const CONTENT: &[u8; 12] = b"A03 grant ok";
/// Boot-scoped epoch. PIDs are never reused within a boot (see README), so
/// the PID alone distinguishes process incarnations in this fixture.
pub const EPOCH: u64 = 1;

/// Fixture rights bits and duration code of the kernel bootstrap grant word.
pub const RIGHT_READ: u64 = 1;
pub const RIGHT_WRITE: u64 = 2;
pub const RIGHT_EXPORT: u64 = 4;
pub const DURATION_UNTIL_CLOSE: u64 = 1;
pub const DURATION_SESSION: u64 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub op: u64,
    pub object: u64,
    /// App-supplied and untrusted; never used for any decision.
    pub claimed_actor: u64,
}

fn get(bytes: &[u8], index: usize) -> u64 {
    let mut word = [0; 8];
    word.copy_from_slice(&bytes[index * 8..index * 8 + 8]);
    u64::from_le_bytes(word)
}

/// Rejects a wrong length or unknown operation.
pub fn decode_request(payload: &[u8]) -> Option<Request> {
    if payload.len() != REQUEST_SIZE {
        return None;
    }
    let request = Request {
        op: get(payload, 0),
        object: get(payload, 1),
        claimed_actor: get(payload, 2),
    };
    matches!(request.op, OP_READ | OP_WRITE | OP_CLOSE).then_some(request)
}

pub fn encode_request(request: Request) -> [u8; REQUEST_SIZE] {
    let mut bytes = [0; REQUEST_SIZE];
    bytes[..8].copy_from_slice(&request.op.to_le_bytes());
    bytes[8..16].copy_from_slice(&request.object.to_le_bytes());
    bytes[16..].copy_from_slice(&request.claimed_actor.to_le_bytes());
    bytes
}

pub fn encode_reply(status: u64, data: &[u8]) -> [u8; REPLY_SIZE] {
    let length = data.len().min(DATA_MAX);
    let mut bytes = [0; REPLY_SIZE];
    bytes[..8].copy_from_slice(&status.to_le_bytes());
    bytes[8..16].copy_from_slice(&(length as u64).to_le_bytes());
    bytes[16..16 + length].copy_from_slice(&data[..length]);
    bytes
}

/// Returns (status, data, length); rejects a wrong size or oversized length.
pub fn decode_reply(payload: &[u8]) -> Option<(u64, [u8; DATA_MAX], usize)> {
    if payload.len() != REPLY_SIZE {
        return None;
    }
    let length = usize::try_from(get(payload, 1)).ok()?;
    if length > DATA_MAX {
        return None;
    }
    let mut data = [0; DATA_MAX];
    data[..length].copy_from_slice(&payload[16..16 + length]);
    Some((get(payload, 0), data, length))
}

/// The authenticated actor for a kernel-supplied sender PID. There is no app
/// principal registry yet, so the PID also stands in for the app identity.
pub fn actor(pid: u64) -> Actor {
    Actor {
        app: pid,
        pid,
        epoch: EPOCH,
    }
}

const HANDLES: usize = 4;

pub struct Broker {
    grants: Grants<2, HANDLES>,
    handles: [Option<Handle>; HANDLES],
}

impl Default for Broker {
    fn default() -> Self {
        Self::new()
    }
}

impl Broker {
    /// Registers the two authoritative objects; object 2 exists but is never granted.
    pub fn new() -> Self {
        let mut grants = Grants::new(1);
        for id in [OBJECT_ONE, OBJECT_TWO] {
            // Fresh two-slot table with distinct ids: registration cannot fail.
            let _ = grants.register_object(Object {
                id,
                revision: REVISION,
            });
        }
        Self {
            grants,
            handles: [None; HANDLES],
        }
    }

    /// FIXTURE: stands in for the trusted permission dialog (U-owned, not
    /// built). Only the trusted bootstrap may call this, never a request.
    pub fn trusted_grant(
        &mut self,
        pid: u64,
        object: u64,
        revision: u64,
        rights: u64,
        duration: u64,
    ) -> Result<(), Error> {
        if pid == 0 || rights & !(RIGHT_READ | RIGHT_WRITE | RIGHT_EXPORT) != 0 {
            return Err(Error::Denied);
        }
        let duration = match duration {
            DURATION_UNTIL_CLOSE => Duration::UntilClose,
            DURATION_SESSION => Duration::Session,
            _ => return Err(Error::Denied),
        };
        let slot = self
            .handles
            .iter()
            .position(Option::is_none)
            .ok_or(Error::GrantTableFull)?;
        let handle = self.grants.grant(
            actor(pid),
            Object {
                id: object,
                revision,
            },
            Rights {
                read: rights & RIGHT_READ != 0,
                write: rights & RIGHT_WRITE != 0,
                export: rights & RIGHT_EXPORT != 0,
            },
            duration,
        )?;
        self.handles[slot] = Some(handle);
        Ok(())
    }

    fn object(id: u64) -> Option<Object> {
        matches!(id, OBJECT_ONE | OBJECT_TWO).then_some(Object {
            id,
            revision: REVISION,
        })
    }

    fn permitted(&self, actor: Actor, object: Object, operation: Operation) -> bool {
        self.handles.iter().flatten().any(|handle| {
            self.grants
                .authorize(*handle, actor, object, operation)
                .is_ok()
        })
    }

    /// Decides one request. `sender` must be the kernel-supplied sender PID.
    /// Returns the reply status and the data to return with it.
    pub fn handle(&mut self, sender: u64, request: Request) -> (u64, &'static [u8]) {
        let actor = actor(sender);
        let Some(object) = Self::object(request.object) else {
            return (STATUS_DENIED, &[]);
        };
        match request.op {
            OP_READ if self.permitted(actor, object, Operation::Read) => {
                let data: &'static [u8] = if object.id == OBJECT_ONE {
                    CONTENT
                } else {
                    &[]
                };
                (STATUS_OK, data)
            }
            // No write path exists; an authorized write would only be acknowledged.
            OP_WRITE if self.permitted(actor, object, Operation::Write) => (STATUS_OK, &[]),
            // close() succeeds for a non-holder without retiring anything, so
            // only an actor holding a live grant on the object may close it.
            OP_CLOSE
                if [Operation::Read, Operation::Write, Operation::Export]
                    .into_iter()
                    .any(|operation| self.permitted(actor, object, operation)) =>
            {
                match self.grants.close(actor, object) {
                    Ok(()) => (STATUS_OK, &[]),
                    Err(_) => (STATUS_DENIED, &[]),
                }
            }
            _ => (STATUS_DENIED, &[]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: u64 = 3;
    const HELPER: u64 = 4;

    fn granted() -> Broker {
        let mut broker = Broker::new();
        broker
            .trusted_grant(
                OWNER,
                OBJECT_ONE,
                REVISION,
                RIGHT_READ,
                DURATION_UNTIL_CLOSE,
            )
            .unwrap();
        broker
    }

    fn request(op: u64, object: u64, claimed_actor: u64) -> Request {
        Request {
            op,
            object,
            claimed_actor,
        }
    }

    #[test]
    fn owner_reads_only_what_was_granted() {
        let mut broker = granted();
        assert_eq!(
            broker.handle(OWNER, request(OP_READ, OBJECT_ONE, OWNER)),
            (STATUS_OK, &CONTENT[..])
        );
        assert_eq!(
            broker.handle(OWNER, request(OP_WRITE, OBJECT_ONE, OWNER)).0,
            STATUS_DENIED
        );
        assert_eq!(
            broker.handle(OWNER, request(OP_READ, OBJECT_TWO, OWNER)).0,
            STATUS_DENIED
        );
        assert_eq!(
            broker.handle(OWNER, request(OP_READ, 99, OWNER)).0,
            STATUS_DENIED
        );
    }

    #[test]
    fn claimed_actor_is_ignored() {
        let mut broker = granted();
        assert_eq!(
            broker.handle(HELPER, request(OP_READ, OBJECT_ONE, OWNER)).0,
            STATUS_DENIED
        );
        // A false claim by the owner itself also changes nothing.
        assert_eq!(
            broker.handle(OWNER, request(OP_READ, OBJECT_ONE, HELPER)).0,
            STATUS_OK
        );
    }

    #[test]
    fn helper_cannot_close_owner_grant() {
        let mut broker = granted();
        assert_eq!(
            broker
                .handle(HELPER, request(OP_CLOSE, OBJECT_ONE, OWNER))
                .0,
            STATUS_DENIED
        );
        assert_eq!(
            broker.handle(OWNER, request(OP_READ, OBJECT_ONE, OWNER)).0,
            STATUS_OK
        );
    }

    #[test]
    fn owner_close_revokes() {
        let mut broker = granted();
        assert_eq!(
            broker.handle(OWNER, request(OP_CLOSE, OBJECT_ONE, OWNER)).0,
            STATUS_OK
        );
        assert_eq!(
            broker.handle(OWNER, request(OP_READ, OBJECT_ONE, OWNER)).0,
            STATUS_DENIED
        );
        assert_eq!(
            broker.handle(OWNER, request(OP_CLOSE, OBJECT_ONE, OWNER)).0,
            STATUS_DENIED
        );
    }

    #[test]
    fn no_fixture_grant_denies_everything() {
        let mut broker = Broker::new();
        for op in [OP_READ, OP_WRITE, OP_CLOSE] {
            assert_eq!(
                broker.handle(OWNER, request(op, OBJECT_ONE, OWNER)).0,
                STATUS_DENIED
            );
        }
    }

    #[test]
    fn invalid_fixture_is_rejected() {
        let mut broker = Broker::new();
        assert!(broker
            .trusted_grant(OWNER, 99, REVISION, RIGHT_READ, DURATION_UNTIL_CLOSE)
            .is_err());
        assert!(broker
            .trusted_grant(OWNER, OBJECT_ONE, 2, RIGHT_READ, DURATION_UNTIL_CLOSE)
            .is_err());
        assert!(broker
            .trusted_grant(OWNER, OBJECT_ONE, REVISION, 0, DURATION_UNTIL_CLOSE)
            .is_err());
        assert!(broker
            .trusted_grant(OWNER, OBJECT_ONE, REVISION, 8, DURATION_UNTIL_CLOSE)
            .is_err());
        assert!(broker
            .trusted_grant(OWNER, OBJECT_ONE, REVISION, RIGHT_READ, 0)
            .is_err());
        assert!(broker
            .trusted_grant(0, OBJECT_ONE, REVISION, RIGHT_READ, DURATION_UNTIL_CLOSE)
            .is_err());
    }

    #[test]
    fn wire_rejects_wrong_length_and_unknown_op() {
        let good = encode_request(request(OP_READ, OBJECT_ONE, OWNER));
        assert_eq!(
            decode_request(&good),
            Some(request(OP_READ, OBJECT_ONE, OWNER))
        );
        assert_eq!(decode_request(&good[..23]), None);
        let mut long = [0u8; REQUEST_SIZE + 1];
        long[..REQUEST_SIZE].copy_from_slice(&good);
        assert_eq!(decode_request(&long), None);
        for op in [0, 4, u64::MAX] {
            assert_eq!(
                decode_request(&encode_request(request(op, OBJECT_ONE, OWNER))),
                None
            );
        }
        let reply = encode_reply(STATUS_OK, CONTENT);
        let (status, data, length) = decode_reply(&reply).unwrap();
        assert_eq!((status, &data[..length]), (STATUS_OK, &CONTENT[..]));
        assert_eq!(decode_reply(&reply[..31]), None);
        let mut oversized = reply;
        oversized[8..16].copy_from_slice(&17u64.to_le_bytes());
        assert_eq!(decode_reply(&oversized), None);
    }
}
