#![no_std]
#![forbid(unsafe_code)]
//! A03 grant broker fixture: a fixed wire codec plus the broker decision.
//! The broker is the trusted adapter of `nieuw-os-task-grant-core`: the only
//! identity it accepts is the kernel-supplied sender PID of each IPC message.
//! The request's claimed actor field exists solely to prove it is ignored.
//!
//! Security invariant (A03-T2c): a client's request causes a volume READ of
//! object O only while the broker holds a live task-grant-core grant with
//! actor == the kernel-supplied sender, object (O, rev) where rev is O's
//! current volume revision, and right READ. Every grant-decision denial is
//! decided before, and makes zero, transport calls. The broker builds every
//! volume request itself and only ever sends STAT (startup registration,
//! re-registration after a Stale READ, and the revision check that replaces
//! READ for an empty object) and READ with selector 0; it has no write path
//! and never relays client bytes to the volume.

use nieuw_os_task_grant_core::{Actor, Duration, Error, Grants, Handle, Object, Operation, Rights};
use volume_core::client::{Client, Transport};
use volume_core::{wire, Status};

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
/// A request claiming to report a process exit. Always denied: retirement is
/// driven only by the kernel's lifecycle notice, never by an app message.
pub const OP_RETIRE: u64 = 4;

pub const STATUS_OK: u64 = 1;
pub const STATUS_DENIED: u64 = 2;
pub const STATUS_BAD_REQUEST: u64 = 3;
/// Broker-to-helper ordering signal: the owner's first phase is served.
pub const STATUS_GO: u64 = 4;
/// Broker-to-owner ordering signal: the helper's phase is served.
pub const STATUS_RESUME: u64 = 5;

pub const OBJECT_ONE: u64 = 1;
pub const OBJECT_TWO: u64 = 2;
/// The volume objects the broker registers at startup (domain 0 ids).
pub const OBJECTS: [u64; 2] = [OBJECT_ONE, OBJECT_TWO];
/// Revision of the fixture objects (`FixtureVolume`, first commit in tests).
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
    matches!(request.op, OP_READ | OP_WRITE | OP_CLOSE | OP_RETIRE).then_some(request)
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

pub struct Broker<T: Transport> {
    /// Sole client of volume domain 0; only STAT and READ are ever sent.
    volume: Client<T>,
    grants: Grants<2, HANDLES>,
    /// Mirror of the core's registered objects with each one's exact length
    /// at that revision (`Client::read` needs it), indexed like `OBJECTS`.
    registered: [Option<(Object, usize)>; 2],
    /// Broker-side index of issued grants: holder, object and core handle.
    handles: [Option<(Actor, Object, Handle)>; HANDLES],
}

impl<T: Transport> Broker<T> {
    /// Registers each of `OBJECTS` at its current volume revision (STAT). An
    /// object whose STAT fails stays unregistered and can never be granted.
    pub fn new(transport: T, epoch: u64) -> Self {
        let mut broker = Self {
            volume: Client::new(transport, epoch, 1),
            grants: Grants::new(1),
            registered: [None; 2],
            handles: [None; HANDLES],
        };
        for (slot, id) in OBJECTS.into_iter().enumerate() {
            if let Ok(stat) = broker.volume.stat(id) {
                let object = Object {
                    id,
                    revision: stat.revision,
                };
                // Fresh two-slot table with distinct ids: registration cannot fail.
                if broker.grants.register_object(object).is_ok() {
                    broker.registered[slot] = Some((object, stat.length));
                }
            }
        }
        broker
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
        self.reclaim();
        let slot = self
            .handles
            .iter()
            .position(Option::is_none)
            .ok_or(Error::GrantTableFull)?;
        let (holder, object) = (
            actor(pid),
            Object {
                id: object,
                revision,
            },
        );
        let handle = self.grants.grant(
            holder,
            object,
            Rights {
                read: rights & RIGHT_READ != 0,
                write: rights & RIGHT_WRITE != 0,
                export: rights & RIGHT_EXPORT != 0,
            },
            duration,
        )?;
        self.handles[slot] = Some((holder, object, handle));
        Ok(())
    }

    /// Frees index entries whose core grant no longer authorizes anything
    /// (closed, revoked or retired). Live session grants keep their entry.
    fn reclaim(&mut self) {
        for slot in &mut self.handles {
            if let Some((holder, object, handle)) = *slot {
                let live = [Operation::Read, Operation::Write, Operation::Export]
                    .into_iter()
                    .any(|operation| {
                        self.grants
                            .authorize(handle, holder, object, operation)
                            .is_ok()
                    });
                if !live {
                    *slot = None;
                }
            }
        }
    }

    fn slot(id: u64) -> Option<usize> {
        OBJECTS.iter().position(|known| *known == id)
    }

    /// The object as currently registered (id and authoritative revision).
    fn object(&self, id: u64) -> Option<Object> {
        Some(self.registered[Self::slot(id)?]?.0)
    }

    /// The whole READ decision; sends nothing. `sender` must be the
    /// kernel-supplied sender PID. Returns the granted (O, rev) to fetch.
    pub fn authorize_read(&self, sender: u64, id: u64) -> Option<Object> {
        let object = self.object(id)?;
        self.permitted(actor(sender), object, Operation::Read)
            .then_some(object)
    }

    /// Fetches at most `DATA_MAX` bytes of an authorised object. Every non-OK
    /// outcome answers DENIED without data and retires all grants on it.
    fn read(&mut self, granted: Object) -> (u64, [u8; DATA_MAX], usize) {
        let Some(slot) = Self::slot(granted.id) else {
            return (STATUS_DENIED, [0; DATA_MAX], 0);
        };
        let length = self.registered[slot].map_or(0, |(_, length)| length);
        let mut data = [0; DATA_MAX];
        // shortcut: streams the whole object to return its first DATA_MAX
        // bytes (`Client::read` needs the exact length); add a bounded read
        // when objects grow beyond fixture size.
        let result = if length == 0 {
            // `Client::read` sends nothing for an empty object; STAT still
            // checks that (O, rev) is O's current state.
            match self.volume.stat(granted.id) {
                Ok(stat) if stat.revision == granted.revision => Ok(0),
                Ok(_) => Err(Status::Stale),
                Err((status, _)) => Err(status),
            }
        } else {
            self.volume
                .read(granted.id, granted.revision, length, |offset, bytes| {
                    if offset < DATA_MAX {
                        let count = bytes.len().min(DATA_MAX - offset);
                        data[offset..offset + count].copy_from_slice(&bytes[..count]);
                    }
                })
        };
        match result {
            Ok(_) => (STATUS_OK, data, length.min(DATA_MAX)),
            Err(status) => {
                self.retire_object(slot, granted, status);
                (STATUS_DENIED, [0; DATA_MAX], 0)
            }
        }
    }

    /// Fail closed after a failed READ of `granted`. Stale: re-STAT and
    /// advance the registered revision, which retires every grant on the
    /// object. Any other failure (NotFound, protocol or transport error, or a
    /// failed re-STAT) revokes every grant on the object while keeping its
    /// registration: the broker can no longer vouch that (O, rev) is O's
    /// current state (deleted, volume replaced, reply lost), and the cost of a
    /// transient failure is only a fresh trusted grant.
    fn retire_object(&mut self, slot: usize, granted: Object, status: Status) {
        if status == Status::Stale {
            if let Ok(stat) = self.volume.stat(granted.id) {
                let object = Object {
                    id: granted.id,
                    revision: stat.revision,
                };
                if self.grants.advance_revision(object).is_ok() {
                    self.registered[slot] = Some((object, stat.length));
                    self.reclaim();
                    return;
                }
            }
        }
        for (_, object, handle) in self.handles.iter().flatten() {
            if object.id == granted.id {
                // A stale handle is already dead; nothing else to retire.
                let _ = self.grants.revoke(*handle);
            }
        }
        self.reclaim();
    }

    fn permitted(&self, actor: Actor, object: Object, operation: Operation) -> bool {
        self.handles.iter().flatten().any(|(_, _, handle)| {
            self.grants
                .authorize(*handle, actor, object, operation)
                .is_ok()
        })
    }

    /// Live grants held by `pid` on any known object, derived from the table.
    pub fn live_grants(&self, pid: u64) -> u64 {
        let actor = actor(pid);
        self.handles
            .iter()
            .flatten()
            .filter(|(holder, object, handle)| {
                *holder == actor
                    && [Operation::Read, Operation::Write, Operation::Export]
                        .into_iter()
                        .any(|operation| {
                            self.grants
                                .authorize(*handle, actor, *object, operation)
                                .is_ok()
                        })
            })
            .count() as u64
    }

    /// Trusted lifecycle operation: `pid` must come from an authenticated
    /// kernel exit notice, never from a request. Returns the grants retired.
    pub fn retire_process(&mut self, pid: u64) -> Result<u64, Error> {
        let before = self.live_grants(pid);
        self.grants.retire_process(actor(pid))?;
        let after = self.live_grants(pid);
        self.reclaim();
        Ok(before - after)
    }

    /// Decides one request. `sender` must be the kernel-supplied sender PID.
    /// Returns the reply status, data and data length (as `decode_reply`).
    pub fn handle(&mut self, sender: u64, request: Request) -> (u64, [u8; DATA_MAX], usize) {
        const DENIED: (u64, [u8; DATA_MAX], usize) = (STATUS_DENIED, [0; DATA_MAX], 0);
        const OK: (u64, [u8; DATA_MAX], usize) = (STATUS_OK, [0; DATA_MAX], 0);
        let actor = actor(sender);
        let Some(object) = self.object(request.object) else {
            return DENIED;
        };
        match request.op {
            OP_READ => match self.authorize_read(sender, request.object) {
                Some(granted) => self.read(granted),
                None => DENIED,
            },
            // No write path exists: an authorized write is only acknowledged
            // and never reaches the volume.
            OP_WRITE if self.permitted(actor, object, Operation::Write) => OK,
            // close() succeeds for a non-holder without retiring anything, so
            // only an actor holding a live grant on the object may close it.
            OP_CLOSE
                if [Operation::Read, Operation::Write, Operation::Export]
                    .into_iter()
                    .any(|operation| self.permitted(actor, object, operation)) =>
            {
                match self.grants.close(actor, object) {
                    Ok(()) => OK,
                    Err(_) => DENIED,
                }
            }
            _ => DENIED,
        }
    }
}

/// FIXTURE for the native bin until T2c-2 wires a real volume domain: a
/// read-only stand-in that answers STAT and READ (selector 0) for `OBJECTS`
/// at `REVISION` (object 1 holds `CONTENT`, object 2 is empty) and refuses
/// every other request. Replies still pass `Client`'s strict validation.
pub struct FixtureVolume {
    pub epoch: u64,
}

impl Transport for FixtureVolume {
    fn call(&mut self, body: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        let request = wire::header(body);
        let content: &[u8] = match request.object {
            OBJECT_ONE => CONTENT,
            OBJECT_TWO => &[],
            _ => return Ok(wire::error_reply(&request, self.epoch, Status::NotFound, 0)),
        };
        let mut reply = wire::Header::new(
            request.operation,
            request.request_id,
            self.epoch,
            request.object,
        );
        reply.revision = REVISION;
        match request.operation {
            wire::STAT => {
                const NAME: &[u8] = b"fixture";
                let mut data = [0; wire::STAT_LENGTH as usize];
                data[..NAME.len()].copy_from_slice(NAME);
                data[48..56].copy_from_slice(&1u64.to_le_bytes()); // created
                data[56..64].copy_from_slice(&1u64.to_le_bytes()); // modified
                data[72..80].copy_from_slice(&REVISION.to_le_bytes()); // version
                reply.length = wire::STAT_LENGTH;
                reply.name_length = NAME.len() as u32;
                reply.aux = content.len() as u64;
                Ok(wire::encode(&reply, &data))
            }
            wire::READ
                if request.revision == REVISION
                    && request.name_length == 0
                    && request.aux <= content.len() as u64 =>
            {
                let data = &content[request.aux as usize..];
                let data = &data[..data.len().min(request.count as usize)];
                reply.aux = request.aux;
                reply.length = data.len() as u32;
                Ok(wire::encode(&reply, data))
            }
            wire::READ => Ok(wire::error_reply(&request, self.epoch, Status::Stale, 0)),
            _ => Ok(wire::error_reply(&request, self.epoch, Status::NoSys, 0)),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::{cell::RefCell, rc::Rc, vec, vec::Vec};

    use super::*;
    use volume_core::{service::Service, table::Table, Disk, SECTOR, VOLUME_SECTORS};

    const OWNER: u64 = 3;
    const HELPER: u64 = 4;
    const VOLUME_EPOCH: u64 = 7;

    struct Memory(Vec<[u8; SECTOR]>);

    impl Disk for Memory {
        fn read(&mut self, sector: u64, buffer: &mut [u8; SECTOR]) -> Result<(), Status> {
            *buffer = *self.0.get(sector as usize).ok_or(Status::Range)?;
            Ok(())
        }
        fn write(&mut self, sector: u64, buffer: &[u8; SECTOR]) -> Result<(), Status> {
            *self.0.get_mut(sector as usize).ok_or(Status::Range)? = *buffer;
            Ok(())
        }
        fn flush(&mut self) -> Result<(), Status> {
            Ok(())
        }
    }

    /// The real volume service on a RAM disk, reached as domain 0.
    type Volume = Rc<RefCell<(Service, Memory)>>;

    fn serve(volume: &Volume, request: &[u8; wire::SIZE]) -> [u8; wire::SIZE] {
        let (service, disk) = &mut *volume.borrow_mut();
        service.handle(0, request, disk)
    }

    /// The broker's transport: records every request operation. `fault`
    /// replaces the service with a transport error or a NotFound reply.
    struct Recorder {
        volume: Volume,
        ops: Vec<u16>,
        fault: Option<Status>,
        /// 1-based call number answered with a Stale reply.
        stale_at: Option<usize>,
        /// 1-based call number failed with a transport error.
        fail_at: Option<usize>,
    }

    impl Transport for Recorder {
        fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
            let header = wire::header(request);
            // Pin the request shape: only STAT or READ, a known object, and
            // READ selector 0 (current content, never history).
            assert!(matches!(header.operation, wire::STAT | wire::READ));
            assert!(OBJECTS.contains(&header.object));
            if header.operation == wire::READ {
                assert_eq!(header.name_length, 0);
            }
            self.ops.push(header.operation);
            let call = Some(self.ops.len());
            if call == self.fail_at {
                return Err(Status::Io);
            }
            if call == self.stale_at {
                return Ok(wire::error_reply(&header, VOLUME_EPOCH, Status::Stale, 0));
            }
            match self.fault {
                Some(Status::NotFound) => Ok(wire::error_reply(
                    &header,
                    VOLUME_EPOCH,
                    Status::NotFound,
                    0,
                )),
                Some(status) => Err(status),
                None => Ok(serve(&self.volume, request)),
            }
        }
    }

    /// Test-harness client that seeds content; never the broker, not recorded.
    struct Seeder(Volume);

    impl Transport for Seeder {
        fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
            Ok(serve(&self.0, request))
        }
    }

    fn seed(seeder: &mut Client<Seeder>, object: u64, expected: u64, content: &[u8]) -> u64 {
        seeder
            .write(
                object,
                expected,
                b"seed",
                content.len(),
                |offset, buffer| buffer.copy_from_slice(&content[offset..offset + buffer.len()]),
            )
            .unwrap()
            .0
    }

    /// Seeds objects 1 and 2 (first commit is `REVISION`), then starts the broker.
    fn setup_with(one: &[u8]) -> (Broker<Recorder>, Client<Seeder>) {
        let volume = Rc::new(RefCell::new((
            Service::new(VOLUME_EPOCH, Table::EMPTY, false),
            Memory(vec![[0; SECTOR]; VOLUME_SECTORS as usize]),
        )));
        let mut seeder = Client::new(Seeder(volume.clone()), VOLUME_EPOCH, 50_000);
        assert_eq!(seed(&mut seeder, OBJECT_ONE, 0, one), REVISION);
        assert_eq!(seed(&mut seeder, OBJECT_TWO, 0, b"two"), REVISION);
        let recorder = Recorder {
            volume,
            ops: Vec::new(),
            fault: None,
            stale_at: None,
            fail_at: None,
        };
        let broker = Broker::new(recorder, VOLUME_EPOCH);
        assert_eq!(ops(&broker), [wire::STAT, wire::STAT]);
        (broker, seeder)
    }

    fn broker() -> Broker<Recorder> {
        setup_with(CONTENT).0
    }

    fn ops(broker: &Broker<Recorder>) -> &[u16] {
        &broker.volume.transport.ops
    }

    /// The broker never sends anything but STAT and READ (selector 0).
    fn assert_read_only(broker: &Broker<Recorder>) {
        assert!(
            ops(broker)
                .iter()
                .all(|op| matches!(*op, wire::STAT | wire::READ)),
            "{:?}",
            ops(broker)
        );
    }

    fn read(broker: &mut Broker<Recorder>, sender: u64, object: u64) -> (u64, Vec<u8>) {
        let (status, data, length) = broker.handle(sender, request(OP_READ, object, sender));
        (status, data[..length].to_vec())
    }

    fn granted() -> Broker<Recorder> {
        let mut broker = broker();
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

    #[test]
    fn closed_grants_free_their_index_entries() {
        let mut broker = broker();
        for _ in 0..HANDLES + 2 {
            broker
                .trusted_grant(
                    OWNER,
                    OBJECT_ONE,
                    REVISION,
                    RIGHT_READ,
                    DURATION_UNTIL_CLOSE,
                )
                .unwrap();
            assert_eq!(
                broker.handle(OWNER, request(OP_CLOSE, OBJECT_ONE, OWNER)).0,
                STATUS_OK
            );
        }
        for _ in 0..HANDLES {
            broker
                .trusted_grant(OWNER, OBJECT_ONE, REVISION, RIGHT_READ, DURATION_SESSION)
                .unwrap();
        }
        // Live session grants are never reclaimed.
        assert_eq!(
            broker.trusted_grant(OWNER, OBJECT_ONE, REVISION, RIGHT_READ, DURATION_SESSION),
            Err(Error::GrantTableFull)
        );
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
            read(&mut broker, OWNER, OBJECT_ONE),
            (STATUS_OK, CONTENT.to_vec())
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
    fn forged_retire_is_denied_and_retires_nothing() {
        let mut broker = granted();
        for sender in [HELPER, OWNER] {
            assert_eq!(
                broker
                    .handle(sender, request(OP_RETIRE, OBJECT_ONE, OWNER))
                    .0,
                STATUS_DENIED
            );
        }
        assert_eq!(broker.live_grants(OWNER), 1);
        assert_eq!(
            broker.handle(OWNER, request(OP_READ, OBJECT_ONE, OWNER)).0,
            STATUS_OK
        );
    }

    #[test]
    fn exit_retires_only_that_process() {
        let mut broker = granted();
        broker
            .trusted_grant(HELPER, OBJECT_TWO, REVISION, RIGHT_READ, DURATION_SESSION)
            .unwrap();
        assert_eq!(broker.retire_process(OWNER), Ok(1));
        assert_eq!(broker.live_grants(OWNER), 0);
        assert_eq!(broker.live_grants(HELPER), 1);
        assert_eq!(
            broker.handle(HELPER, request(OP_READ, OBJECT_ONE, OWNER)).0,
            STATUS_DENIED
        );
        assert_eq!(
            broker.handle(OWNER, request(OP_READ, OBJECT_ONE, OWNER)).0,
            STATUS_DENIED
        );
        assert_eq!(broker.retire_process(OWNER), Ok(0));
    }

    #[test]
    fn no_fixture_grant_denies_everything() {
        let mut broker = broker();
        for op in [OP_READ, OP_WRITE, OP_CLOSE] {
            assert_eq!(
                broker.handle(OWNER, request(op, OBJECT_ONE, OWNER)).0,
                STATUS_DENIED
            );
        }
    }

    #[test]
    fn invalid_fixture_is_rejected() {
        let mut broker = broker();
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
        for op in [0, 5, u64::MAX] {
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

    // A03-T2c-1: READ of real volume objects through the broker.

    #[test]
    fn granted_read_returns_seeded_bytes_via_stat_and_read_only() {
        let mut broker = granted();
        assert_eq!(
            read(&mut broker, OWNER, OBJECT_ONE),
            (STATUS_OK, CONTENT.to_vec())
        );
        assert_eq!(ops(&broker), [wire::STAT, wire::STAT, wire::READ]);
    }

    #[test]
    fn long_object_returns_its_first_data_max_bytes() {
        // Longer than one READ reply, so several READs; still nothing else.
        let long: Vec<u8> = (0..300u32).map(|byte| byte as u8).collect();
        let (mut broker, _) = setup_with(&long);
        broker
            .trusted_grant(OWNER, OBJECT_ONE, REVISION, RIGHT_READ, DURATION_SESSION)
            .unwrap();
        assert_eq!(
            read(&mut broker, OWNER, OBJECT_ONE),
            (STATUS_OK, long[..DATA_MAX].to_vec())
        );
        assert!(ops(&broker)[2..].iter().all(|op| *op == wire::READ));
        assert!(ops(&broker).len() > 3);
    }

    #[test]
    fn ungranted_object_is_denied_without_transport_calls() {
        let mut broker = granted();
        for object in [OBJECT_TWO, 0, 99] {
            assert_eq!(read(&mut broker, OWNER, object), (STATUS_DENIED, vec![]));
        }
        assert_eq!(ops(&broker), [wire::STAT, wire::STAT]);
    }

    #[test]
    fn forged_actor_is_denied_without_transport_calls() {
        let mut broker = granted();
        let (status, _, length) = broker.handle(HELPER, request(OP_READ, OBJECT_ONE, OWNER));
        assert_eq!((status, length), (STATUS_DENIED, 0));
        assert_eq!(ops(&broker), [wire::STAT, wire::STAT]);
    }

    #[test]
    fn read_after_close_is_denied_without_transport_calls() {
        let mut broker = granted();
        assert_eq!(
            broker.handle(OWNER, request(OP_CLOSE, OBJECT_ONE, OWNER)).0,
            STATUS_OK
        );
        assert_eq!(
            read(&mut broker, OWNER, OBJECT_ONE),
            (STATUS_DENIED, vec![])
        );
        assert_eq!(ops(&broker), [wire::STAT, wire::STAT]);
    }

    #[test]
    fn read_after_retirement_is_denied_without_transport_calls() {
        let mut broker = granted();
        assert_eq!(broker.retire_process(OWNER), Ok(1));
        assert_eq!(
            read(&mut broker, OWNER, OBJECT_ONE),
            (STATUS_DENIED, vec![])
        );
        assert_eq!(ops(&broker), [wire::STAT, wire::STAT]);
    }

    #[test]
    fn rewritten_object_is_stale_denied_and_retires_the_grant() {
        let (mut broker, mut seeder) = setup_with(CONTENT);
        broker
            .trusted_grant(OWNER, OBJECT_ONE, REVISION, RIGHT_READ, DURATION_SESSION)
            .unwrap();
        let revision = seed(&mut seeder, OBJECT_ONE, REVISION, b"rewritten");
        assert!(revision > REVISION);
        assert_eq!(
            read(&mut broker, OWNER, OBJECT_ONE),
            (STATUS_DENIED, vec![])
        );
        // READ answered Stale; the re-STAT re-registered the new revision.
        assert_eq!(
            ops(&broker),
            [wire::STAT, wire::STAT, wire::READ, wire::STAT]
        );
        assert_eq!(broker.live_grants(OWNER), 0);
        assert_eq!(
            read(&mut broker, OWNER, OBJECT_ONE),
            (STATUS_DENIED, vec![])
        );
        assert_eq!(ops(&broker).len(), 4);
        // Only a fresh decision about the current revision reads again.
        assert!(broker
            .trusted_grant(OWNER, OBJECT_ONE, REVISION, RIGHT_READ, DURATION_SESSION)
            .is_err());
        broker
            .trusted_grant(OWNER, OBJECT_ONE, revision, RIGHT_READ, DURATION_SESSION)
            .unwrap();
        assert_eq!(
            read(&mut broker, OWNER, OBJECT_ONE),
            (STATUS_OK, b"rewritten".to_vec())
        );
        assert_read_only(&broker);
    }

    #[test]
    fn empty_object_read_is_revision_checked() {
        let (mut broker, mut seeder) = setup_with(b"");
        broker
            .trusted_grant(OWNER, OBJECT_ONE, REVISION, RIGHT_READ, DURATION_SESSION)
            .unwrap();
        assert_eq!(read(&mut broker, OWNER, OBJECT_ONE), (STATUS_OK, vec![]));
        assert_eq!(ops(&broker), [wire::STAT, wire::STAT, wire::STAT]);
        let revision = seed(&mut seeder, OBJECT_ONE, REVISION, b"rewritten");
        assert!(revision > REVISION);
        assert_eq!(
            read(&mut broker, OWNER, OBJECT_ONE),
            (STATUS_DENIED, vec![])
        );
        assert_eq!(broker.live_grants(OWNER), 0);
        broker
            .trusted_grant(OWNER, OBJECT_ONE, revision, RIGHT_READ, DURATION_SESSION)
            .unwrap();
        assert_eq!(
            read(&mut broker, OWNER, OBJECT_ONE),
            (STATUS_OK, b"rewritten".to_vec())
        );
        assert_read_only(&broker);
    }

    #[test]
    fn not_found_and_transport_errors_are_denied_and_retire_the_grant() {
        for fault in [Status::NotFound, Status::Io, Status::Unknown] {
            let mut broker = granted();
            broker.volume.transport.fault = Some(fault);
            assert_eq!(
                read(&mut broker, OWNER, OBJECT_ONE),
                (STATUS_DENIED, vec![])
            );
            assert_eq!(ops(&broker), [wire::STAT, wire::STAT, wire::READ]);
            assert_eq!(broker.live_grants(OWNER), 0);
            broker.volume.transport.fault = None;
            assert_eq!(
                read(&mut broker, OWNER, OBJECT_ONE),
                (STATUS_DENIED, vec![])
            );
            assert_eq!(ops(&broker).len(), 3);
            // Registration is kept: a fresh grant on the same revision reads.
            broker
                .trusted_grant(OWNER, OBJECT_ONE, REVISION, RIGHT_READ, DURATION_SESSION)
                .unwrap();
            assert_eq!(
                read(&mut broker, OWNER, OBJECT_ONE),
                (STATUS_OK, CONTENT.to_vec())
            );
        }
    }

    #[test]
    fn only_stat_and_read_are_ever_sent() {
        let (mut broker, mut seeder) = setup_with(CONTENT);
        let all = RIGHT_READ | RIGHT_WRITE | RIGHT_EXPORT;
        broker
            .trusted_grant(OWNER, OBJECT_ONE, REVISION, all, DURATION_UNTIL_CLOSE)
            .unwrap();
        broker
            .trusted_grant(HELPER, OBJECT_TWO, REVISION, all, DURATION_SESSION)
            .unwrap();
        for sender in [OWNER, HELPER] {
            for object in [OBJECT_ONE, OBJECT_TWO, 99] {
                for op in [OP_READ, OP_WRITE, OP_RETIRE] {
                    broker.handle(sender, request(op, object, OWNER));
                }
            }
        }
        // An authorised write is acknowledged without reaching the volume.
        let before = ops(&broker).len();
        assert_eq!(
            broker.handle(OWNER, request(OP_WRITE, OBJECT_ONE, OWNER)).0,
            STATUS_OK
        );
        assert_eq!(ops(&broker).len(), before);
        seed(&mut seeder, OBJECT_TWO, REVISION, b"stale");
        broker.handle(HELPER, request(OP_READ, OBJECT_TWO, HELPER));
        broker.volume.transport.fault = Some(Status::Io);
        broker.handle(OWNER, request(OP_READ, OBJECT_ONE, OWNER));
        broker.volume.transport.fault = None;
        for sender in [OWNER, HELPER] {
            broker.handle(sender, request(OP_CLOSE, OBJECT_ONE, sender));
            broker.retire_process(sender).unwrap();
        }
        assert!(ops(&broker).contains(&wire::READ));
        assert_read_only(&broker);
    }

    #[test]
    fn fixture_volume_serves_the_native_script() {
        let mut broker = Broker::new(FixtureVolume { epoch: EPOCH }, EPOCH);
        broker
            .trusted_grant(OWNER, OBJECT_ONE, REVISION, RIGHT_READ, DURATION_SESSION)
            .unwrap();
        broker
            .trusted_grant(OWNER, OBJECT_TWO, REVISION, RIGHT_READ, DURATION_SESSION)
            .unwrap();
        let (status, data, length) = broker.handle(OWNER, request(OP_READ, OBJECT_ONE, OWNER));
        assert_eq!((status, &data[..length]), (STATUS_OK, &CONTENT[..]));
        assert_eq!(
            broker.handle(OWNER, request(OP_READ, OBJECT_TWO, OWNER)),
            (STATUS_OK, [0; DATA_MAX], 0)
        );
        assert_eq!(
            broker.handle(HELPER, request(OP_READ, OBJECT_ONE, OWNER)).0,
            STATUS_DENIED
        );
    }

    /// A Stale READ (call 3) whose re-STAT (call 4) fails or is not newer
    /// must revoke every grant on the object yet keep its registration.
    fn stale_without_newer_revision_revokes(fail_restat: bool) {
        let mut broker = granted();
        broker
            .trusted_grant(HELPER, OBJECT_ONE, REVISION, RIGHT_READ, DURATION_SESSION)
            .unwrap();
        broker.volume.transport.stale_at = Some(3);
        broker.volume.transport.fail_at = fail_restat.then_some(4);
        assert_eq!(
            read(&mut broker, OWNER, OBJECT_ONE),
            (STATUS_DENIED, vec![])
        );
        assert_eq!(
            ops(&broker),
            [wire::STAT, wire::STAT, wire::READ, wire::STAT]
        );
        assert_eq!(
            (broker.live_grants(OWNER), broker.live_grants(HELPER)),
            (0, 0)
        );
        for sender in [OWNER, HELPER] {
            assert_eq!(
                read(&mut broker, sender, OBJECT_ONE),
                (STATUS_DENIED, vec![])
            );
        }
        assert_eq!(ops(&broker).len(), 4);
        // Registration stays at the volume's current revision.
        broker
            .trusted_grant(OWNER, OBJECT_ONE, REVISION, RIGHT_READ, DURATION_SESSION)
            .unwrap();
        assert_eq!(
            read(&mut broker, OWNER, OBJECT_ONE),
            (STATUS_OK, CONTENT.to_vec())
        );
    }

    #[test]
    fn stale_then_failed_restat_revokes_and_keeps_registration() {
        stale_without_newer_revision_revokes(true);
    }

    #[test]
    fn stale_then_not_newer_restat_revokes_and_keeps_registration() {
        stale_without_newer_revision_revokes(false);
    }
}
