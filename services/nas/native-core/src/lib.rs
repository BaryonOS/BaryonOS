//! Bounded NAS authority and ciphertext store, independent of host APIs.
//!
//! The caller identity must come from an authenticated transport or the kernel,
//! never from a request body. This core does not authenticate peers, encrypt
//! documents, grant network access, or provide a native service by itself.
//! Each published state uses a serialized, atomic storage transaction. No
//! acknowledgement precedes its completion. An uncertain outcome retires the
//! instance; only recovery from storage can resume service.
#![no_std]
#![forbid(unsafe_code)]

mod snapshot;

pub use snapshot::SNAPSHOT_BYTES;

/// Validate the canonical storage schema, not its authenticity or freshness.
pub fn validate_snapshot(bytes: &[u8; SNAPSHOT_BYTES]) -> Result<(), Error> {
    let mut state = State::EMPTY;
    snapshot::decode(bytes, &mut state)
}

pub const NAMESPACES: usize = 4;
pub const OBJECTS: usize = 4;
pub const SESSIONS: usize = 16;
pub const UPLOADS: usize = 8;
pub const MAX_CIPHERTEXT: usize = 1024;
pub const MAX_QUOTA: u32 = (OBJECTS * MAX_CIPHERTEXT) as u32;

/// Public, cryptographically verified identity fingerprint. Possession of these
/// bytes is not identity proof. Only the trusted ingress supplies this value.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Principal(pub [u8; 32]);

impl Principal {
    fn valid(self) -> bool {
        self.0 != [0; 32]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Denied,
    Invalid,
    Full,
    Quota,
    Conflict,
    Expired,
    NotFound,
    Stale,
    Entropy,
    Corrupt,
    Io,
    /// Storage may have committed; never automatically replay the operation.
    Unknown,
    ReopenRequired,
}

/// Production adapters must use the existing entropy service, not a clock,
/// counter, test seed or NAS-supplied randomness.
pub trait Entropy {
    fn fill(&mut self, bytes: &mut [u8; 32]) -> Result<(), Error>;
}

/// Exclusive single-writer storage of one bounded snapshot. `replace` must
/// atomically retain either the previous snapshot or all of `snapshot`, and
/// return success only after durable commit. An empty store is `None`, never a
/// fallback for unreadable/corrupt data. No content keys belong in this store.
/// Any replacement error may follow a commit; the core reports `Unknown` and
/// retains the original error for diagnostics. No error code authorizes replay.
pub trait Storage {
    fn load(&mut self, out: &mut [u8; SNAPSHOT_BYTES]) -> Result<Option<usize>, Error>;
    fn replace(&mut self, snapshot: &[u8; SNAPSHOT_BYTES]) -> Result<(), Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NamespaceStatus {
    Offered = 1,
    Active = 2,
    Revoked = 3,
    Evicted = 4,
}

#[derive(Clone, Copy)]
struct Object {
    revision: u64,
    length: u16,
    bytes: [u8; MAX_CIPHERTEXT],
}

impl Object {
    const EMPTY: Self = Self {
        revision: 0,
        length: 0,
        bytes: [0; MAX_CIPHERTEXT],
    };
}

#[derive(Clone, Copy)]
struct Namespace {
    id: [u8; 32],
    status: NamespaceStatus,
    epoch: u64,
    quota: u32,
    owner: Principal,
    invitation: [u8; 32],
    expires: u64,
    objects: [Object; OBJECTS],
}

impl Namespace {
    fn used(&self) -> u32 {
        self.objects.iter().map(|o| u32::from(o.length)).sum()
    }
}

#[derive(Clone, Copy)]
struct State {
    generation: u64,
    admin: Principal,
    namespaces: [Option<Namespace>; NAMESPACES],
}

impl State {
    const EMPTY: Self = Self {
        generation: 0,
        admin: Principal([0; 32]),
        namespaces: [None; NAMESPACES],
    };
}

/// This is a local invitation identifier, not a document key or reusable login.
/// It matches the intended authenticated recipient in this authority store.
/// The transport envelope must separately bind the selected NAS identity.
/// No Debug implementation: callers must not log invitation material.
#[derive(Clone, Copy)]
pub struct Invitation {
    pub namespace: [u8; 32],
    pub nonce: [u8; 32],
    pub recipient: Principal,
    pub expires: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Session {
    boot: [u8; 32],
    serial: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Upload {
    boot: [u8; 32],
    serial: u64,
}

pub const TOKEN_BYTES: usize = 40;

fn encode_token(boot: [u8; 32], serial: u64) -> [u8; TOKEN_BYTES] {
    let mut bytes = [0; TOKEN_BYTES];
    bytes[..32].copy_from_slice(&boot);
    bytes[32..].copy_from_slice(&serial.to_le_bytes());
    bytes
}

fn decode_token(bytes: &[u8; TOKEN_BYTES]) -> Result<([u8; 32], u64), Error> {
    let boot = bytes[..32].try_into().unwrap();
    let serial = u64::from_le_bytes(bytes[32..].try_into().unwrap());
    if boot == [0; 32] || serial == 0 {
        return Err(Error::Invalid);
    }
    Ok((boot, serial))
}

impl Session {
    pub fn to_bytes(self) -> [u8; TOKEN_BYTES] {
        encode_token(self.boot, self.serial)
    }

    /// Canonical decoding is not authorization; every use checks caller,
    /// live session membership and namespace epoch again.
    pub fn from_bytes(bytes: &[u8; TOKEN_BYTES]) -> Result<Self, Error> {
        let (boot, serial) = decode_token(bytes)?;
        Ok(Self { boot, serial })
    }
}

impl Upload {
    pub fn to_bytes(self) -> [u8; TOKEN_BYTES] {
        encode_token(self.boot, self.serial)
    }

    pub fn from_bytes(bytes: &[u8; TOKEN_BYTES]) -> Result<Self, Error> {
        let (boot, serial) = decode_token(bytes)?;
        Ok(Self { boot, serial })
    }
}

#[derive(Clone, Copy)]
struct SessionState {
    token: Session,
    namespace: usize,
    epoch: u64,
    owner: Principal,
}

#[derive(Clone, Copy)]
struct UploadState {
    token: Upload,
    session: Session,
    namespace: usize,
    epoch: u64,
    object: usize,
    expected: u64,
    length: usize,
    bytes: [u8; MAX_CIPHERTEXT],
}

/// Administration deliberately exposes no object identifiers or ciphertext.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Usage {
    pub namespace: [u8; 32],
    pub status: NamespaceStatus,
    pub epoch: u64,
    pub quota: u32,
    pub used: u32,
    pub reserved: u32,
}

/// Caller-owned arena: reserve outside the native stack (static or scoped
/// mapped memory). The exclusive borrow prevents sharing live service state.
pub struct Workspace {
    state: State,
    next: State,
    snapshot: [u8; SNAPSHOT_BYTES],
    boot: [u8; 32],
    serial: u64,
    sessions: [Option<SessionState>; SESSIONS],
    uploads: [Option<UploadState>; UPLOADS],
    poisoned: bool,
    storage_error: Option<Error>,
}

impl Workspace {
    pub const fn new() -> Self {
        Self {
            state: State::EMPTY,
            next: State::EMPTY,
            snapshot: [0; SNAPSHOT_BYTES],
            boot: [0; 32],
            serial: 0,
            sessions: [None; SESSIONS],
            uploads: [None; UPLOADS],
            poisoned: false,
            storage_error: None,
        }
    }

    pub fn last_storage_error(&self) -> Option<Error> {
        self.storage_error
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Service<'a, S> {
    storage: S,
    work: &'a mut Workspace,
}

impl<'a, S: Storage> Service<'a, S> {
    /// Explicit initialization of an empty, designated store only.
    pub fn initialize(
        mut storage: S,
        admin: Principal,
        entropy: &mut impl Entropy,
        work: &'a mut Workspace,
    ) -> Result<Self, Error> {
        if !admin.valid() {
            return Err(Error::Invalid);
        }
        work.storage_error = None;
        if storage.load(&mut work.snapshot)?.is_some() {
            return Err(Error::Conflict);
        }
        let boot = random(entropy)?;
        work.state = State {
            generation: 1,
            admin,
            namespaces: [None; NAMESPACES],
        };
        snapshot::encode(&work.state, &mut work.snapshot);
        if let Err(error) = storage.replace(&work.snapshot) {
            work.storage_error = Some(error);
            return Err(Error::Unknown);
        }
        Ok(Self::new(storage, work, boot))
    }

    pub fn recover(
        mut storage: S,
        entropy: &mut impl Entropy,
        work: &'a mut Workspace,
    ) -> Result<Self, Error> {
        if storage.load(&mut work.snapshot)? != Some(SNAPSHOT_BYTES) {
            return Err(Error::Corrupt);
        }
        snapshot::decode(&work.snapshot, &mut work.state)?;
        let boot = random(entropy)?;
        Ok(Self::new(storage, work, boot))
    }

    fn new(storage: S, work: &'a mut Workspace, boot: [u8; 32]) -> Self {
        work.boot = boot;
        work.serial = 0;
        work.sessions.fill(None);
        work.uploads.fill(None);
        work.poisoned = false;
        work.storage_error = None;
        Self { storage, work }
    }

    pub fn into_storage(self) -> S {
        self.storage
    }

    pub fn is_poisoned(&self) -> bool {
        self.work.poisoned
    }

    pub fn last_storage_error(&self) -> Option<Error> {
        self.work.last_storage_error()
    }

    fn ready(&self) -> Result<(), Error> {
        if self.work.poisoned {
            Err(Error::ReopenRequired)
        } else {
            Ok(())
        }
    }

    fn admin(&self, caller: Principal) -> Result<(), Error> {
        self.ready()?;
        if caller == self.work.state.admin {
            Ok(())
        } else {
            Err(Error::Denied)
        }
    }

    fn namespace(&self, id: &[u8; 32]) -> Result<usize, Error> {
        self.work
            .state
            .namespaces
            .iter()
            .position(|n| n.as_ref().is_some_and(|n| &n.id == id))
            .ok_or(Error::NotFound)
    }

    fn next_serial(&mut self) -> Result<u64, Error> {
        self.work.serial = self.work.serial.checked_add(1).ok_or(Error::Full)?;
        Ok(self.work.serial)
    }

    fn persist(&mut self) -> Result<(), Error> {
        self.work.next.generation = self
            .work
            .state
            .generation
            .checked_add(1)
            .ok_or(Error::Full)?;
        snapshot::encode(&self.work.next, &mut self.work.snapshot);
        if let Err(error) = self.storage.replace(&self.work.snapshot) {
            self.work.poisoned = true;
            self.work.storage_error = Some(error);
            return Err(Error::Unknown);
        }
        core::mem::swap(&mut self.work.state, &mut self.work.next);
        Ok(())
    }

    pub fn invite(
        &mut self,
        caller: Principal,
        recipient: Principal,
        quota: u32,
        now: u64,
        lifetime: u64,
        entropy: &mut impl Entropy,
    ) -> Result<Invitation, Error> {
        self.admin(caller)?;
        if !recipient.valid()
            || recipient == self.work.state.admin
            || quota == 0
            || quota > MAX_QUOTA
            || lifetime == 0
        {
            return Err(Error::Invalid);
        }
        let expires = now.checked_add(lifetime).ok_or(Error::Invalid)?;
        let index = self
            .work
            .state
            .namespaces
            .iter()
            .position(Option::is_none)
            .ok_or(Error::Full)?;
        let id = random(entropy)?;
        // Never retry a broken/repeating entropy source indefinitely.
        if self.namespace(&id).is_ok() {
            return Err(Error::Entropy);
        }
        let nonce = random(entropy)?;
        if nonce == id {
            return Err(Error::Entropy);
        }
        self.work.next = self.work.state;
        self.work.next.namespaces[index] = Some(Namespace {
            id,
            status: NamespaceStatus::Offered,
            epoch: 1,
            quota,
            owner: recipient,
            invitation: nonce,
            expires,
            objects: [Object::EMPTY; OBJECTS],
        });
        self.persist()?;
        Ok(Invitation {
            namespace: id,
            nonce,
            recipient,
            expires,
        })
    }

    pub fn accept(
        &mut self,
        caller: Principal,
        invitation: &Invitation,
        now: u64,
    ) -> Result<(), Error> {
        self.ready()?;
        let index = self.namespace(&invitation.namespace)?;
        self.work.next = self.work.state;
        let ns = self.work.next.namespaces[index]
            .as_mut()
            .ok_or(Error::NotFound)?;
        if ns.status != NamespaceStatus::Offered
            || caller != ns.owner
            || invitation.recipient != caller
            || invitation.nonce != ns.invitation
            || invitation.expires != ns.expires
        {
            return Err(Error::Denied);
        }
        if now >= ns.expires {
            return Err(Error::Expired);
        }
        ns.status = NamespaceStatus::Active;
        ns.invitation = [0; 32];
        ns.expires = 0;
        self.persist()
    }

    pub fn open(&mut self, caller: Principal, namespace: &[u8; 32]) -> Result<Session, Error> {
        self.ready()?;
        let index = self.namespace(namespace)?;
        let ns = self.work.state.namespaces[index]
            .as_ref()
            .ok_or(Error::NotFound)?;
        if ns.status != NamespaceStatus::Active || ns.owner != caller {
            return Err(Error::Denied);
        }
        let slot = self
            .work
            .sessions
            .iter()
            .position(Option::is_none)
            .ok_or(Error::Full)?;
        let epoch = ns.epoch;
        let token = Session {
            boot: self.work.boot,
            serial: self.next_serial()?,
        };
        self.work.sessions[slot] = Some(SessionState {
            token,
            namespace: index,
            epoch,
            owner: caller,
        });
        Ok(token)
    }

    fn session(&self, caller: Principal, token: Session) -> Result<SessionState, Error> {
        self.ready()?;
        let session = self
            .work
            .sessions
            .iter()
            .flatten()
            .find(|s| s.token == token && s.owner == caller)
            .copied()
            .ok_or(Error::Stale)?;
        let ns = self.work.state.namespaces[session.namespace]
            .as_ref()
            .ok_or(Error::Stale)?;
        if ns.status != NamespaceStatus::Active || ns.epoch != session.epoch {
            return Err(Error::Stale);
        }
        Ok(session)
    }

    pub fn close(&mut self, caller: Principal, token: Session) -> Result<(), Error> {
        self.session(caller, token)?;
        for slot in &mut self.work.sessions {
            if slot.is_some_and(|s| s.token == token) {
                *slot = None;
            }
        }
        for slot in &mut self.work.uploads {
            if slot.is_some_and(|u| u.session == token) {
                *slot = None;
            }
        }
        Ok(())
    }

    fn reserved(&self, namespace: usize) -> u32 {
        self.work
            .uploads
            .iter()
            .flatten()
            .filter(|u| u.namespace == namespace)
            .map(|u| u.length as u32)
            .sum()
    }

    pub fn stage(
        &mut self,
        caller: Principal,
        token: Session,
        object: usize,
        expected: u64,
        ciphertext: &[u8],
    ) -> Result<Upload, Error> {
        let session = self.session(caller, token)?;
        if object >= OBJECTS || ciphertext.is_empty() || ciphertext.len() > MAX_CIPHERTEXT {
            return Err(Error::Invalid);
        }
        let ns = self.work.state.namespaces[session.namespace]
            .as_ref()
            .ok_or(Error::Stale)?;
        if expected == u64::MAX || ns.objects[object].revision != expected {
            return Err(Error::Conflict);
        }
        // Conservative copy-on-write reservation includes the old object.
        if ns.used() + self.reserved(session.namespace) + ciphertext.len() as u32 > ns.quota {
            return Err(Error::Quota);
        }
        let slot = self
            .work
            .uploads
            .iter()
            .position(Option::is_none)
            .ok_or(Error::Full)?;
        let upload = Upload {
            boot: self.work.boot,
            serial: self.next_serial()?,
        };
        let mut bytes = [0; MAX_CIPHERTEXT];
        bytes[..ciphertext.len()].copy_from_slice(ciphertext);
        self.work.uploads[slot] = Some(UploadState {
            token: upload,
            session: token,
            namespace: session.namespace,
            epoch: session.epoch,
            object,
            expected,
            length: ciphertext.len(),
            bytes,
        });
        Ok(upload)
    }

    fn upload(&self, session: SessionState, upload: Upload) -> Result<(usize, UploadState), Error> {
        self.work
            .uploads
            .iter()
            .enumerate()
            .filter_map(|(i, u)| u.map(|u| (i, u)))
            .find(|(_, u)| {
                u.token == upload
                    && u.session == session.token
                    && u.namespace == session.namespace
                    && u.epoch == session.epoch
            })
            .ok_or(Error::Stale)
    }

    pub fn cancel(
        &mut self,
        caller: Principal,
        session: Session,
        upload: Upload,
    ) -> Result<(), Error> {
        let session = self.session(caller, session)?;
        let (slot, _) = self.upload(session, upload)?;
        self.work.uploads[slot] = None;
        Ok(())
    }

    pub fn commit(
        &mut self,
        caller: Principal,
        session: Session,
        upload: Upload,
    ) -> Result<u64, Error> {
        let session = self.session(caller, session)?;
        let (slot, upload) = self.upload(session, upload)?;
        self.work.next = self.work.state;
        let ns = self.work.next.namespaces[session.namespace]
            .as_mut()
            .ok_or(Error::Stale)?;
        if ns.objects[upload.object].revision != upload.expected {
            self.work.uploads[slot] = None;
            return Err(Error::Conflict);
        }
        let revision = upload.expected.checked_add(1).ok_or(Error::Full)?;
        ns.objects[upload.object] = Object {
            revision,
            length: upload.length as u16,
            bytes: upload.bytes,
        };
        self.persist()?;
        self.work.uploads[slot] = None;
        Ok(revision)
    }

    /// Returns only opaque client ciphertext, never document plaintext.
    pub fn read(
        &self,
        caller: Principal,
        session: Session,
        object: usize,
        expected: u64,
        out: &mut [u8],
    ) -> Result<usize, Error> {
        let session = self.session(caller, session)?;
        let object = self.work.state.namespaces[session.namespace]
            .as_ref()
            .and_then(|n| n.objects.get(object))
            .ok_or(Error::Invalid)?;
        if object.length == 0 {
            return Err(Error::NotFound);
        }
        if object.revision != expected {
            return Err(Error::Conflict);
        }
        let length = usize::from(object.length);
        if out.len() < length {
            return Err(Error::Invalid);
        }
        out[..length].copy_from_slice(&object.bytes[..length]);
        Ok(length)
    }

    pub fn usage(&self, caller: Principal, namespace: &[u8; 32]) -> Result<Usage, Error> {
        self.admin(caller)?;
        let index = self.namespace(namespace)?;
        self.usage_at(caller, index)
    }

    /// Bounded administrator enumeration; exposes no object IDs or content.
    pub fn usage_at(&self, caller: Principal, index: usize) -> Result<Usage, Error> {
        self.admin(caller)?;
        let slot = self
            .work
            .state
            .namespaces
            .get(index)
            .ok_or(Error::Invalid)?;
        let ns = slot.as_ref().ok_or(Error::NotFound)?;
        Ok(Usage {
            namespace: ns.id,
            status: ns.status,
            epoch: ns.epoch,
            quota: ns.quota,
            used: ns.used(),
            reserved: self.reserved(index),
        })
    }

    pub fn set_quota(
        &mut self,
        caller: Principal,
        namespace: &[u8; 32],
        quota: u32,
    ) -> Result<(), Error> {
        self.admin(caller)?;
        let index = self.namespace(namespace)?;
        let reserved = self.reserved(index);
        self.work.next = self.work.state;
        let ns = self.work.next.namespaces[index]
            .as_mut()
            .ok_or(Error::NotFound)?;
        if matches!(
            ns.status,
            NamespaceStatus::Revoked | NamespaceStatus::Evicted
        ) {
            return Err(Error::Denied);
        }
        if quota == 0 || quota > MAX_QUOTA {
            return Err(Error::Invalid);
        }
        if quota < ns.used() + reserved {
            return Err(Error::Quota);
        }
        ns.quota = quota;
        self.persist()
    }

    fn drop_handles(&mut self, namespace: usize) {
        for session in &mut self.work.sessions {
            if session.is_some_and(|s| s.namespace == namespace) {
                *session = None;
            }
        }
        for upload in &mut self.work.uploads {
            if upload.is_some_and(|u| u.namespace == namespace) {
                *upload = None;
            }
        }
    }

    /// Revocation commits before cleanup. On failure the instance is retired;
    /// no old session can continue against uncertain durable authority.
    pub fn revoke(&mut self, caller: Principal, namespace: &[u8; 32]) -> Result<(), Error> {
        self.admin(caller)?;
        let index = self.namespace(namespace)?;
        self.work.next = self.work.state;
        let ns = self.work.next.namespaces[index]
            .as_mut()
            .ok_or(Error::NotFound)?;
        if matches!(
            ns.status,
            NamespaceStatus::Revoked | NamespaceStatus::Evicted
        ) {
            return Ok(());
        }
        ns.epoch = ns.epoch.checked_add(1).ok_or(Error::Full)?;
        ns.status = NamespaceStatus::Revoked;
        ns.invitation = [0; 32];
        ns.expires = 0;
        self.persist()?;
        self.drop_handles(index);
        Ok(())
    }

    /// Two durable transactions: revoke first, then purge this store's copies.
    /// A failed purge leaves a revoked namespace, never false deletion success.
    pub fn evict(&mut self, caller: Principal, namespace: &[u8; 32]) -> Result<(), Error> {
        self.revoke(caller, namespace)?;
        let index = self.namespace(namespace)?;
        self.work.next = self.work.state;
        let ns = self.work.next.namespaces[index]
            .as_mut()
            .ok_or(Error::NotFound)?;
        if ns.status == NamespaceStatus::Evicted {
            return Ok(());
        }
        ns.objects = [Object::EMPTY; OBJECTS];
        ns.status = NamespaceStatus::Evicted;
        self.persist()
    }
}

fn random(entropy: &mut impl Entropy) -> Result<[u8; 32], Error> {
    let mut bytes = [0; 32];
    entropy.fill(&mut bytes)?;
    if bytes == [0; 32] {
        return Err(Error::Entropy);
    }
    Ok(bytes)
}
