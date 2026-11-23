//! Pure M6e service state machine. The native adapter supplies only IPC and a
//! scoped `Disk`; all authorization below uses the kernel-bound domain index.
//!
//! Durability order for every mutation: content sectors (only into chunks the
//! durable table does not reference) -> FLUSH -> next table copy -> FLUSH ->
//! live state -> ACK. A failure before the table write is a certain abort
//! (`Io`, durable state unchanged). A failure during or after the table write
//! is `Unknown`: this instance then refuses every later request and never
//! replays the write. A fresh instance recovers from disk.
use crate::{
    checksum_update, chunks_for, table, wire, Disk, Status, CHUNK, CHUNKS, CHUNK_SECTORS,
    COLLECTIONS, DATA_START, DOMAINS, FNV_OFFSET, HISTORY, MAX_CHUNKS, NAME, NO_CHUNK, PER_DOMAIN,
    QUOTA, SECTOR, TABLE_SECTORS, TRANSACT_MAX,
};
use table::{Collection, Entry, Next, Table, Version};
#[cfg(feature = "l2-install")]
use wire::Manifest;
use wire::{Header, INSTALL_MAX, SIZE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Session {
    /// v7 only: actual kernel-authenticated requester, never a payload field.
    pub principal: u64,
    /// v7 only: service-issued, never reused during this service instance.
    pub upload_token: u32,
    pub object: usize,
    pub expected: u64,
    pub request_id: u64,
    pub length: usize,
    pub received: usize,
    pub name: [u8; NAME],
    pub name_length: usize,
    pub chunks: [u16; MAX_CHUNKS],
    pub chunk_count: usize,
    pub sums: [u64; MAX_CHUNKS],
    pub content_sum: u64,
    sector: [u8; SECTOR],
}

/// One version of a staged install: its chunks are
/// `InstallSession::chunks[chunk_start..chunk_start + chunk_count]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstallVersion {
    pub origin: u64,
    pub length: usize,
    pub received: usize,
    pub name: [u8; NAME],
    pub name_length: usize,
    pub content_sum: u64,
    pub chunk_start: usize,
    pub chunk_count: usize,
}

/// L2 install staging (S09-L2-PROVENANCE 4.2). Never ACKed until one commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstallSession {
    pub principal: u64,
    pub upload_token: u32,
    pub object: usize,
    pub request_id: u64,
    pub n: usize,
    pub tags: u32,
    pub kind: u16,
    pub collection: u8,
    pub origin_revision: u64,
    pub origin_created: u64,
    pub origin_modified: u64,
    pub versions: [InstallVersion; INSTALL_MAX],
    pub chunks: [u16; INSTALL_MAX * MAX_CHUNKS],
    pub sums: [u64; INSTALL_MAX * MAX_CHUNKS],
    pub chunk_count: usize,
    /// One buffer: the APPEND that completes a version writes its last
    /// partial sector before it replies.
    sector: [u8; SECTOR],
}

#[cfg(feature = "l2-install")]
impl InstallSession {
    fn total(&self) -> usize {
        self.versions[..self.n].iter().map(|v| v.length).sum()
    }
    /// Lowest-index incomplete version (zero-length ones are complete).
    fn target(&self) -> Option<usize> {
        self.versions[..self.n]
            .iter()
            .position(|v| v.received < v.length)
    }
}

/// At most one staged session per domain (session matrix, 4.1).
// Both variants live inline in `Service` (no allocator); the size is pinned
// by a test against the native stack budget.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Staged {
    Upload(Session),
    #[cfg(feature = "l2-install")]
    Install(InstallSession),
}

impl Staged {
    pub fn principal(&self) -> u64 {
        match self {
            Self::Upload(s) => s.principal,
            #[cfg(feature = "l2-install")]
            Self::Install(s) => s.principal,
        }
    }
    pub fn upload_token(&self) -> u32 {
        match self {
            Self::Upload(s) => s.upload_token,
            #[cfg(feature = "l2-install")]
            Self::Install(s) => s.upload_token,
        }
    }
    fn bind(&mut self, principal: u64, token: u32) {
        match self {
            Self::Upload(s) => (s.principal, s.upload_token) = (principal, token),
            #[cfg(feature = "l2-install")]
            Self::Install(s) => (s.principal, s.upload_token) = (principal, token),
        }
    }
    fn chunks(&self) -> &[u16] {
        match self {
            Self::Upload(s) => &s.chunks[..s.chunk_count],
            #[cfg(feature = "l2-install")]
            Self::Install(s) => &s.chunks[..s.chunk_count],
        }
    }
}

struct Cache {
    valid: bool,
    object: usize,
    version: u64,
    chunk: usize,
    bytes: [u8; CHUNK],
}

/// Monotonic evidence counters; they describe this instance only.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    pub commits: u64,
    pub data_sectors: u64,
    pub table_writes: u64,
    pub flushes: u64,
    pub read_sectors: u64,
    pub chunk_allocations: u64,
    pub rejected: u64,
    pub aborted_sessions: u64,
    pub unknown: u64,
    pub transactions: u64,
    pub restores: u64,
}

/// Replacement objects/collections of one atomic commit.
struct Patch {
    objects: [(usize, Entry); TRANSACT_MAX],
    count: usize,
    collections: Option<[Collection; DOMAINS * COLLECTIONS]>,
    /// Minimum table version of the published copy (0 = keep the current).
    format: u16,
}

impl Patch {
    const fn new() -> Self {
        Self {
            objects: [(0, Entry::EMPTY); TRANSACT_MAX],
            count: 0,
            collections: None,
            format: 0,
        }
    }
    fn one(object: usize, entry: Entry) -> Self {
        let mut patch = Self::new();
        patch.push(object, entry);
        patch
    }
    fn push(&mut self, object: usize, entry: Entry) {
        self.objects[self.count] = (object, entry);
        self.count += 1;
    }
}

pub struct Service {
    epoch: u64,
    table: Table,
    sessions: [Option<Staged>; DOMAINS],
    cache: Cache,
    read_only: bool,
    format_initialized: bool,
    admission_checked: bool,
    uncertain: bool,
    // Separate counters avoid revealing another domain's upload activity.
    next_upload_token: [u32; DOMAINS],
    pub counters: Counters,
}

fn reply(
    request: &Header,
    epoch: u64,
    revision: u64,
    aux: u64,
    count: u32,
    data: &[u8],
) -> [u8; SIZE] {
    let mut h = Header::new(request.operation, request.request_id, epoch, request.object);
    h.revision = revision;
    h.aux = aux;
    h.count = count;
    h.length = data.len() as u32;
    wire::encode(&h, data)
}

type Outcome = Result<[u8; SIZE], (Status, u64)>;

impl Service {
    pub fn new(epoch: u64, table: Table, read_only: bool) -> Self {
        let format_initialized = table.generation != 0;
        Self {
            epoch,
            table,
            sessions: [None; DOMAINS],
            cache: Cache {
                valid: false,
                object: 0,
                version: 0,
                chunk: 0,
                bytes: [0; CHUNK],
            },
            read_only,
            format_initialized,
            admission_checked: false,
            uncertain: false,
            next_upload_token: [1; DOMAINS],
            counters: Counters::default(),
        }
    }

    /// Recover the durable table, decoded directly into the new service
    /// rather than through a separate table value. (This avoids one table
    /// copy; the native stack budget is still set by the whole binary.)
    ///
    /// A writable instance flushes before it serves any request: the chosen
    /// copy may so far exist only in the device cache, while later commits
    /// may free chunks that the other, fallback copy still references. If
    /// that flush fails, the instance does not start.
    pub fn recover(epoch: u64, read_only: bool, disk: &mut impl Disk) -> Result<Self, Status> {
        let mut service = Self::new(epoch, Table::EMPTY, read_only);
        service.format_initialized = table::recover_with_state(disk, &mut service.table)?;
        service.admission_checked = true;
        if !read_only {
            disk.flush()?;
            service.counters.flushes += 1;
        }
        Ok(service)
    }

    pub fn table(&self) -> &Table {
        &self.table
    }
    pub fn uncertain(&self) -> bool {
        self.uncertain
    }
    /// The staged upload of `domain` (None while an install is staged).
    pub fn session(&self, domain: usize) -> Option<&Session> {
        match &self.sessions[domain] {
            Some(Staged::Upload(session)) => Some(session),
            _ => None,
        }
    }
    pub fn staged(&self, domain: usize) -> Option<&Staged> {
        self.sessions[domain].as_ref()
    }

    /// Handle one already sender-authenticated request. `domain` must come from
    /// the kernel-bound endpoint the request arrived on, never from the body.
    pub fn handle(&mut self, domain: usize, body: &[u8; SIZE], disk: &mut impl Disk) -> [u8; SIZE] {
        assert!(domain < DOMAINS);
        // Legacy callers cannot replace, append, commit or cancel an active
        // token-bound upload, even with a colliding request ID/revision.
        if let Ok(request) = wire::validate_request(body, self.epoch, domain) {
            if matches!(
                request.operation,
                wire::BEGIN
                    | wire::APPEND
                    | wire::COMMIT
                    | wire::ABORT
                    | wire::INSTALL_BEGIN
                    | wire::INSTALL_APPEND
                    | wire::INSTALL_COMMIT
            ) && !self.uncertain
                && self.sessions[domain]
                    .as_ref()
                    .is_some_and(|s| s.upload_token() != 0)
            {
                self.counters.rejected += 1;
                return wire::error_reply(&request, self.epoch, Status::Busy, 0);
            }
        }
        self.handle_inner(domain, body, disk)
    }

    /// v7 entry point. `principal` MUST come from the kernel-delivered sender
    /// of the already domain-authorized ingress, never the untrusted payload.
    /// One domain retains its existing single staged upload. A second BEGIN
    /// is Busy; it cannot silently replace another editor's name/content.
    pub fn handle_for(
        &mut self,
        domain: usize,
        principal: u64,
        body: &[u8; SIZE],
        disk: &mut impl Disk,
    ) -> [u8; SIZE] {
        use crate::upload;
        assert!(domain < DOMAINS);
        let (request, token, canonical) = match upload::validate_request(body, self.epoch, domain) {
            Ok(parts) => parts,
            Err(status) => {
                self.counters.rejected += 1;
                return upload::wrap(
                    wire::error_reply(&wire::header(body), self.epoch, status, 0),
                    upload::token(body),
                );
            }
        };
        let rejected =
            |status| upload::wrap(wire::error_reply(&request, self.epoch, status, 0), token);
        if principal == 0 {
            self.counters.rejected += 1;
            return rejected(Status::Access);
        }
        if self.uncertain {
            return rejected(Status::Unknown);
        }
        if matches!(request.operation, wire::BEGIN | wire::INSTALL_BEGIN) {
            if self.sessions[domain].is_some() {
                self.counters.rejected += 1;
                return rejected(Status::Busy);
            }
            let Some(next) = self.next_upload_token[domain].checked_add(1) else {
                self.counters.rejected += 1;
                return rejected(Status::Quota);
            };
            let response = self.handle_inner(domain, &canonical, disk);
            if wire::header(&response).status != Status::Ok as i32 {
                return upload::wrap(response, 0);
            }
            let issued = self.next_upload_token[domain];
            self.next_upload_token[domain] = next;
            self.sessions[domain]
                .as_mut()
                .expect("successful BEGIN stages a session")
                .bind(principal, issued);
            return upload::wrap(response, issued);
        }
        if matches!(
            request.operation,
            wire::APPEND | wire::COMMIT | wire::ABORT | wire::INSTALL_APPEND | wire::INSTALL_COMMIT
        ) && !self.sessions[domain]
            .as_ref()
            .is_some_and(|s| s.principal() == principal && s.upload_token() == token)
        {
            self.counters.rejected += 1;
            return rejected(Status::Access);
        }
        upload::wrap(self.handle_inner(domain, &canonical, disk), token)
    }

    /// Trusted adapter/supervisor notification only, never a wire operation.
    /// End every staged upload of a requester proven dead, or after ALL its
    /// service grants have been authoritatively retired. Retirement of one old
    /// endpoint alone is insufficient: it could cancel a newer upload from the
    /// same living principal. Tokens are not reset/reused.
    /// Native owner-death event wiring is a separate integration requirement.
    pub fn detach(&mut self, principal: u64) -> usize {
        let mut detached = 0;
        for staged in &mut self.sessions {
            if staged
                .as_ref()
                .is_some_and(|s| s.upload_token() != 0 && s.principal() == principal)
            {
                *staged = None;
                self.counters.aborted_sessions += 1;
                detached += 1;
            }
        }
        detached
    }

    fn handle_inner(
        &mut self,
        domain: usize,
        body: &[u8; SIZE],
        disk: &mut impl Disk,
    ) -> [u8; SIZE] {
        assert!(domain < DOMAINS);
        let request = match wire::validate_request(body, self.epoch, domain) {
            Ok(request) => request,
            Err(status) => {
                self.counters.rejected += 1;
                return wire::error_reply(&wire::header(body), self.epoch, status, 0);
            }
        };
        if self.uncertain {
            // Validation first; Unknown then reveals no state and grants nothing.
            return wire::error_reply(&request, self.epoch, Status::Unknown, 0);
        }
        let mutating = !matches!(
            request.operation,
            wire::STAT
                | wire::READ
                | wire::LIST
                | wire::COLLECTION_STAT
                | wire::DOMAIN_STAT
                | wire::VERSION_STAT
                | wire::PROVENANCE_STAT
        );
        let result = if mutating && self.read_only {
            Err((Status::Access, 0))
        } else {
            match request.operation {
                wire::STAT => self.stat(&request),
                wire::BEGIN => self.begin(domain, &request, body, disk),
                wire::APPEND => self.append(domain, &request, body, disk),
                wire::COMMIT => self.commit(domain, &request, disk),
                wire::ABORT => self.abort(domain, &request),
                wire::READ => self.read(&request, disk),
                wire::LIST => Ok(self.list(domain, &request)),
                wire::COLLECTION_STAT => Ok(self.collection_stat(domain, &request)),
                wire::TRANSACT => self.transact(domain, &request, body, disk),
                wire::COLLECTION => self.collection(domain, &request, body, disk),
                wire::SHUTDOWN => self.shutdown(&request, disk),
                wire::DOMAIN_STAT => Ok(self.domain_stat(domain, &request)),
                wire::VERSION_STAT => self.version_stat(&request),
                #[cfg(feature = "l2-install")]
                wire::INSTALL_BEGIN => self.install_begin(domain, &request, body, disk),
                #[cfg(feature = "l2-install")]
                wire::INSTALL_APPEND => self.install_append(domain, &request, body, disk),
                #[cfg(feature = "l2-install")]
                wire::INSTALL_COMMIT => self.install_commit(domain, &request, disk),
                #[cfg(feature = "l2-install")]
                wire::INSTALL_COLLECTION => self.install_collection(domain, &request, body, disk),
                #[cfg(feature = "l2-install")]
                wire::PROVENANCE_STAT => self.provenance_stat(domain, &request),
                _ => self.object_operation(domain, &request, body, disk),
            }
        };
        match result {
            Ok(body) => body,
            Err((status, revision)) => {
                if status == Status::Unknown {
                    self.counters.unknown += 1;
                } else {
                    self.counters.rejected += 1;
                }
                wire::error_reply(&request, self.epoch, status, revision)
            }
        }
    }

    fn stat(&self, request: &Header) -> Outcome {
        let entry = &self.table.entries[request.object as usize];
        if !entry.present {
            return Err((Status::NotFound, entry.revision));
        }
        let mut data = [0; wire::STAT_LENGTH as usize];
        data[..NAME].copy_from_slice(&entry.current.name);
        data[32..40].copy_from_slice(&entry.current.content_sum.to_le_bytes());
        data[40..48].copy_from_slice(&entry.request_id.to_le_bytes());
        data[48..56].copy_from_slice(&entry.created.to_le_bytes());
        data[56..64].copy_from_slice(&entry.modified.to_le_bytes());
        data[64..68].copy_from_slice(&entry.tags.to_le_bytes());
        data[68..70].copy_from_slice(&entry.kind.to_le_bytes());
        data[70] = entry.collection;
        data[71] = entry.history_count as u8;
        data[72..80].copy_from_slice(&entry.current.revision.to_le_bytes());
        for (index, version) in entry.history[..entry.history_count].iter().enumerate() {
            let at = 80 + index * wire::STAT_HISTORY;
            data[at..at + 8].copy_from_slice(&version.revision.to_le_bytes());
            data[at + 8..at + 12].copy_from_slice(&(version.length as u32).to_le_bytes());
            data[at + 12..at + 20].copy_from_slice(&version.content_sum.to_le_bytes());
        }
        let mut body = reply(
            request,
            self.epoch,
            entry.revision,
            entry.current.length as u64,
            0,
            &data,
        );
        body[48..52].copy_from_slice(&(entry.current.name_length as u32).to_le_bytes());
        Ok(body)
    }

    fn reserved(&self, chunk: u16) -> bool {
        self.sessions
            .iter()
            .flatten()
            .any(|session| session.chunks().contains(&chunk))
    }

    fn begin(
        &mut self,
        domain: usize,
        request: &Header,
        body: &[u8; SIZE],
        disk: &mut impl Disk,
    ) -> Outcome {
        // One upload per domain. A new BEGIN from the same kernel-bound domain
        // replaces an abandoned one (e.g. after an app restart); no other
        // domain can influence this session. A staged install is never
        // replaced: the kind is checked before anything is taken (4.1).
        if self.installing(domain) {
            return Err((Status::Busy, 0));
        }
        if self.sessions[domain].take().is_some() {
            self.counters.aborted_sessions += 1;
        }
        let object = request.object as usize;
        let entry = &self.table.entries[object];
        if entry.revision != request.revision {
            return Err((Status::Stale, 0));
        }
        let length = request.aux as usize;
        let needed = chunks_for(length);
        // Conservative: the replaced version stays in history, so committed
        // usage plus the new chunks bounds the usage after commit.
        if self.table.domain_chunks(domain) + needed > QUOTA {
            return Err((Status::Quota, 0));
        }
        let mut session = Session {
            principal: 0,
            upload_token: 0,
            object,
            expected: request.revision,
            request_id: request.request_id,
            length,
            received: 0,
            name: [0; NAME],
            name_length: request.name_length as usize,
            chunks: [NO_CHUNK; MAX_CHUNKS],
            chunk_count: 0,
            sums: [0; MAX_CHUNKS],
            content_sum: FNV_OFFSET,
            sector: [0; SECTOR],
        };
        session.name[..session.name_length].copy_from_slice(wire::data(body, session.name_length));
        // Lowest free index first keeps allocation deterministic, so tests and
        // the disk oracle can observe real reuse of released chunks.
        for chunk in 0..CHUNKS as u16 {
            if session.chunk_count == needed {
                break;
            }
            if !self.table.references(chunk) && !self.reserved(chunk) {
                session.chunks[session.chunk_count] = chunk;
                session.sums[session.chunk_count] = FNV_OFFSET;
                session.chunk_count += 1;
            }
        }
        if session.chunk_count != needed {
            return Err((Status::Full, 0));
        }
        self.ensure_format_initialized(disk)?;
        self.counters.chunk_allocations += needed as u64;
        self.sessions[domain] = Some(Staged::Upload(session));
        Ok(reply(
            request,
            self.epoch,
            request.revision,
            length as u64,
            needed as u32,
            &[],
        ))
    }

    /// Is an install staged in `domain` (session matrix, 4.1)?
    fn installing(&self, domain: usize) -> bool {
        #[cfg(feature = "l2-install")]
        return matches!(self.sessions[domain], Some(Staged::Install(_)));
        #[cfg(not(feature = "l2-install"))]
        {
            let _ = domain;
            false
        }
    }

    fn matching(&self, domain: usize, request: &Header) -> Result<Session, (Status, u64)> {
        match self.sessions[domain] {
            Some(Staged::Upload(session))
                if session.object == request.object as usize
                    && session.request_id == request.request_id
                    && session.expected == request.revision =>
            {
                Ok(session)
            }
            _ => Err((Status::Invalid, 0)),
        }
    }

    fn data_sector(session: &Session, index: usize) -> u64 {
        let chunk = u64::from(session.chunks[index / CHUNK_SECTORS as usize]);
        DATA_START + chunk * CHUNK_SECTORS + (index as u64 % CHUNK_SECTORS)
    }

    fn append(
        &mut self,
        domain: usize,
        request: &Header,
        body: &[u8; SIZE],
        disk: &mut impl Disk,
    ) -> Outcome {
        let mut session = self.matching(domain, request)?;
        let bytes = wire::data(body, request.length as usize);
        if request.aux as usize != session.received
            || session.received + bytes.len() > session.length
        {
            return Err((Status::Range, 0));
        }
        for byte in bytes {
            let offset = session.received;
            session.sector[offset % SECTOR] = *byte;
            let chunk = offset / CHUNK;
            session.sums[chunk] = checksum_update(session.sums[chunk], &[*byte]);
            session.content_sum = checksum_update(session.content_sum, &[*byte]);
            session.received += 1;
            if session.received % SECTOR == 0 {
                let sector = Self::data_sector(&session, offset / SECTOR);
                if disk.write(sector, &session.sector).is_err() {
                    // Only unreferenced chunks were touched: durable state is
                    // unchanged, so this is a certain abort, not Unknown.
                    self.sessions[domain] = None;
                    self.counters.aborted_sessions += 1;
                    return Err((Status::Io, 0));
                }
                self.counters.data_sectors += 1;
                session.sector = [0; SECTOR];
            }
        }
        let received = session.received as u64;
        self.sessions[domain] = Some(Staged::Upload(session));
        Ok(reply(
            request,
            self.epoch,
            request.revision,
            received,
            0,
            &[],
        ))
    }

    fn abort(&mut self, domain: usize, request: &Header) -> Outcome {
        #[cfg(feature = "l2-install")]
        if self.installing(domain) {
            self.matching_install(domain, request)?;
        } else {
            self.matching(domain, request)?;
        }
        #[cfg(not(feature = "l2-install"))]
        self.matching(domain, request)?;
        self.sessions[domain] = None;
        self.counters.aborted_sessions += 1;
        Ok(reply(request, self.epoch, request.revision, 0, 0, &[]))
    }

    /// Staged uploads were never ACKed, so dropping them loses nothing durable.
    fn shutdown(&mut self, request: &Header, disk: &mut impl Disk) -> Outcome {
        for staged in &mut self.sessions {
            // Uploads and installs alike: never ACKed, nothing durable lost.
            if staged.take().is_some() {
                self.counters.aborted_sessions += 1;
            }
        }
        if disk.flush().is_err() {
            // The barrier's outcome is unknowable; never claim durability.
            self.uncertain = true;
            return Err((Status::Unknown, 0));
        }
        self.counters.flushes += 1;
        self.read_only = true;
        Ok(reply(request, self.epoch, 0, 0, 0, &[]))
    }

    fn commit(&mut self, domain: usize, request: &Header, disk: &mut impl Disk) -> Outcome {
        let session = self.matching(domain, request)?;
        if request.aux as usize != session.length || session.received != session.length {
            return Err((Status::Range, 0));
        }
        self.sessions[domain] = None;
        let current = self.table.entries[session.object];
        if current.revision != session.expected {
            self.counters.aborted_sessions += 1;
            return Err((Status::Stale, 0));
        }
        let staged = (|| {
            if session.received % SECTOR != 0 {
                let sector = Self::data_sector(&session, session.received / SECTOR);
                disk.write(sector, &session.sector)?;
                self.counters.data_sectors += 1;
            }
            disk.flush()?;
            self.counters.flushes += 1;
            Ok::<(), Status>(())
        })();
        if staged.is_err() {
            self.counters.aborted_sessions += 1;
            return Err((Status::Io, 0));
        }
        let clock = self.next_clock(domain)?;
        let mut entry = current;
        if !entry.present {
            // A re-created object never inherits the old incarnation's origin.
            entry.end_incarnation();
        }
        entry.push_history();
        entry.current = Version {
            // Rule R admits revision u64::MAX on a crafted image: fail closed.
            revision: current.revision.checked_add(1).ok_or((Status::Full, 0))?,
            length: session.length,
            chunk_count: session.chunk_count,
            name_length: session.name_length,
            name: session.name,
            content_sum: session.content_sum,
            chunks: session.chunks,
            sums: session.sums,
            origin: 0,
        };
        entry.current.chunks[session.chunk_count..].fill(NO_CHUNK);
        entry.current.sums[session.chunk_count..].fill(0);
        entry.present = true;
        self.touch(&mut entry, request.request_id, clock)?;
        self.publish(
            domain,
            &Patch::one(session.object, entry),
            request.request_id,
            disk,
        )?;
        Ok(reply(
            request,
            self.epoch,
            entry.revision,
            session.length as u64,
            0,
            &[],
        ))
    }

    fn next_generation(&self) -> Result<u64, (Status, u64)> {
        self.table
            .generation
            .checked_add(1)
            .ok_or((Status::Full, 0))
    }

    /// The committing domain's clock after this commit. Object stamps use it
    /// instead of the global generation, so a domain cannot observe how many
    /// commits another domain makes. Clocks never exceed the generation.
    fn next_clock(&self, domain: usize) -> Result<u64, (Status, u64)> {
        self.next_generation()?;
        Ok(self.table.clocks[domain] + 1)
    }

    /// Common bookkeeping of any object mutation. Recovery bounds a deleted
    /// object's history by `revision + 1`, so a tombstone can never take
    /// revision `u64::MAX`: refused as exhausted, like the generation, before
    /// anything is published.
    fn touch(&self, entry: &mut Entry, request_id: u64, clock: u64) -> Result<(), (Status, u64)> {
        let limit = if entry.present {
            u64::MAX
        } else {
            u64::MAX - 1
        };
        if entry.revision >= limit {
            return Err((Status::Full, 0));
        }
        entry.revision += 1;
        entry.request_id = request_id;
        if entry.created == 0 {
            entry.created = clock;
        }
        entry.modified = clock;
        Ok(())
    }

    /// DELETE, RENAME, RESTORE, PURGE and META on one object of `domain`.
    fn object_operation(
        &mut self,
        domain: usize,
        request: &Header,
        body: &[u8; SIZE],
        disk: &mut impl Disk,
    ) -> Outcome {
        let object = request.object as usize;
        let current = self.table.entries[object];
        let needs_present = matches!(request.operation, wire::DELETE | wire::RENAME | wire::META);
        if needs_present && !current.present
            || request.operation == wire::PURGE && current.revision == 0
        {
            return Err((Status::NotFound, current.revision));
        }
        if current.revision != request.revision {
            return Err((Status::Stale, 0));
        }
        let clock = self.next_clock(domain)?;
        let mut entry = current;
        match request.operation {
            wire::DELETE => Self::delete(&mut entry),
            wire::PURGE => Self::purge(&mut entry),
            wire::RENAME => {
                let length = request.name_length as usize;
                entry.current.name = [0; NAME];
                entry.current.name[..length].copy_from_slice(wire::data(body, length));
                entry.current.name_length = length;
            }
            wire::RESTORE => {
                let slot = request.aux as usize - 1;
                if slot >= entry.history_count {
                    return Err((Status::NotFound, current.revision));
                }
                // The restored version leaves the history and becomes the
                // current one; the replaced current version takes its place,
                // so no other history version is dropped.
                let restored = entry.take_history(slot);
                entry.push_history();
                entry.current = Version {
                    revision: current.revision.checked_add(1).ok_or((Status::Full, 0))?,
                    ..restored
                };
                entry.present = true;
            }
            wire::META => {
                let (tags, kind, collection, _) = wire::unmeta(request.aux);
                self.apply_meta(domain, &mut entry, tags, kind, collection)?;
            }
            _ => return Err((Status::NoSys, 0)),
        }
        self.touch(&mut entry, request.request_id, clock)?;
        self.publish(domain, &Patch::one(object, entry), request.request_id, disk)?;
        // Count only durable restores, like transactions.
        if request.operation == wire::RESTORE {
            self.counters.restores += 1;
        }
        Ok(reply(request, self.epoch, entry.revision, 0, 0, &[]))
    }

    /// Deletion keeps the content as the newest history version, so it is
    /// undoable with RESTORE until PURGE releases the space.
    fn delete(entry: &mut Entry) {
        entry.push_history();
        entry.present = false;
        entry.current = Version::EMPTY;
        entry.collection = 0;
        entry.tags = 0;
        entry.kind = 0;
    }

    fn purge(entry: &mut Entry) {
        entry.history = [Version::EMPTY; HISTORY];
        entry.history_count = 0;
        if !entry.present {
            // Nothing of the installed incarnation is left (2.2).
            entry.end_incarnation();
        }
    }

    fn apply_meta(
        &self,
        domain: usize,
        entry: &mut Entry,
        tags: u32,
        kind: u16,
        collection: u8,
    ) -> Result<(), (Status, u64)> {
        if collection != 0
            && !self
                .table
                .collection(domain, usize::from(collection))
                .present
        {
            return Err((Status::Range, 0));
        }
        entry.tags = tags;
        entry.kind = kind;
        entry.collection = collection;
        Ok(())
    }

    /// Read-only state of one collection of `domain`: revision (the CAS
    /// token), presence, name and the request that produced the revision, so
    /// a client can show its collections and resolve an Unknown COLLECTION
    /// commit by reading state. Never reveals another domain's collections.
    fn collection_stat(&self, domain: usize, request: &Header) -> [u8; SIZE] {
        let collection = self.table.collection(domain, request.aux as usize);
        let mut body = reply(
            request,
            self.epoch,
            collection.revision,
            collection.request_id,
            u32::from(collection.present),
            collection.name(),
        );
        body[48..52].copy_from_slice(&(collection.name_length as u32).to_le_bytes());
        body
    }

    /// The caller's own clock and generation only: never `table.generation`,
    /// the global sum of all clocks, which would reveal other domains' commits.
    fn domain_stat(&self, domain: usize, request: &Header) -> [u8; SIZE] {
        let generation = self.table.domain_generation(domain);
        reply(
            request,
            self.epoch,
            generation,
            self.table.clocks[domain],
            0,
            &[],
        )
    }

    /// Selector 0: the stamps of any own slot (present or not). Selector
    /// `s >= 1`: the name of history version `s` of a present object, only if
    /// current, history[0], .., history[s - 1] have consecutive revisions. A
    /// gap may hide a DELETE (S09-ARCHIVE-V2 section 3), so versions behind it,
    /// and every version of an absent slot, stay unnamed (R2).
    fn version_stat(&self, request: &Header) -> Outcome {
        let entry = &self.table.entries[request.object as usize];
        if entry.revision != request.revision {
            return if entry.revision == 0 {
                Err((Status::NotFound, 0))
            } else {
                Err((Status::Stale, 0))
            };
        }
        let selector = request.name_length as usize;
        if selector == 0 {
            let mut data = [0; 16];
            data[..8].copy_from_slice(&entry.created.to_le_bytes());
            data[8..].copy_from_slice(&entry.modified.to_le_bytes());
            let present = u32::from(entry.present);
            return Ok(reply(
                request,
                self.epoch,
                entry.revision,
                0,
                present,
                &data,
            ));
        }
        if !entry.present || selector > Self::prefix(entry) {
            return Err((Status::NotFound, entry.revision));
        }
        let version = &entry.history[selector - 1];
        let mut body = reply(
            request,
            self.epoch,
            entry.revision,
            version.revision,
            0,
            version.name(),
        );
        body[48..52].copy_from_slice(&(version.name_length as u32).to_le_bytes());
        Ok(body)
    }

    /// History versions of a present `entry` that follow the current version
    /// with consecutive revisions (VERSION_STAT's R2 rule).
    fn prefix(entry: &Entry) -> usize {
        let mut newer = entry.current.revision;
        entry.history[..entry.history_count]
            .iter()
            .take_while(|version| {
                let step = version.revision.checked_add(1) == Some(newer);
                newer = version.revision;
                step
            })
            .count()
    }

    /// Create, rename or delete (empty name) one collection of `domain`.
    fn collection(
        &mut self,
        domain: usize,
        request: &Header,
        body: &[u8; SIZE],
        disk: &mut impl Disk,
    ) -> Outcome {
        let index = request.aux as usize;
        let slot = domain * COLLECTIONS + index - 1;
        let current = self.table.collections[slot];
        if current.revision != request.revision {
            return Err((Status::Stale, 0));
        }
        let length = request.name_length as usize;
        let mut next = current;
        if length == 0 {
            if !current.present {
                return Err((Status::NotFound, current.revision));
            }
            let in_use = self.table.entries[domain * PER_DOMAIN..(domain + 1) * PER_DOMAIN]
                .iter()
                .any(|entry| usize::from(entry.collection) == index);
            if in_use {
                return Err((Status::Busy, 0));
            }
            next.present = false;
            next.name = [0; NAME];
            next.name_length = 0;
            next.origin_revision = 0;
        } else {
            next.present = true;
            next.name = [0; NAME];
            next.name[..length].copy_from_slice(wire::data(body, length));
            next.name_length = length;
        }
        // A crafted image may hold revision u64::MAX (clock = generation).
        next.revision = next.revision.checked_add(1).ok_or((Status::Full, 0))?;
        next.request_id = request.request_id;
        let mut collections = self.table.collections;
        collections[slot] = next;
        let mut patch = Patch::new();
        patch.collections = Some(collections);
        self.publish(domain, &patch, request.request_id, disk)?;
        Ok(reply(
            request,
            self.epoch,
            next.revision,
            index as u64,
            0,
            &[],
        ))
    }

    /// All-or-nothing META/DELETE/PURGE over up to four distinct objects of
    /// one domain: every precondition is checked before a single table write.
    fn transact(
        &mut self,
        domain: usize,
        request: &Header,
        body: &[u8; SIZE],
        disk: &mut impl Disk,
    ) -> Outcome {
        let clock = self.next_clock(domain)?;
        let mut patch = Patch::new();
        for index in 0..request.count as usize {
            let bytes = &body[64 + index * wire::SUBOP..64 + (index + 1) * wire::SUBOP];
            let sub = wire::decode_subop(bytes).ok_or((Status::Invalid, 0))?;
            let object = domain * PER_DOMAIN + usize::from(sub.index);
            let mut entry = self.table.entries[object];
            match sub.op {
                wire::SUB_META | wire::SUB_DELETE if !entry.present => {
                    return Err((Status::NotFound, 0));
                }
                wire::SUB_PURGE if entry.revision == 0 => return Err((Status::NotFound, 0)),
                _ => {}
            }
            if entry.revision != sub.expected {
                return Err((Status::Stale, 0));
            }
            match sub.op {
                wire::SUB_META => {
                    self.apply_meta(domain, &mut entry, sub.tags, sub.kind, sub.collection)?
                }
                wire::SUB_DELETE => Self::delete(&mut entry),
                _ => Self::purge(&mut entry),
            }
            self.touch(&mut entry, request.request_id, clock)?;
            patch.push(object, entry);
        }
        self.publish(domain, &patch, request.request_id, disk)?;
        self.counters.transactions += 1;
        Ok(reply(
            request,
            self.epoch,
            self.table.domain_generation(domain),
            0,
            request.count,
            &[],
        ))
    }

    /// Write the next table copy, a commit of `domain`, into the older slot.
    /// Never touches the slot holding the current durable generation.
    fn publish(
        &mut self,
        domain: usize,
        patch: &Patch,
        request_id: u64,
        disk: &mut impl Disk,
    ) -> Result<(), (Status, u64)> {
        self.ensure_format_initialized(disk)?;
        let generation = self.next_generation()?;
        let collections = patch.collections.unwrap_or(self.table.collections);
        let mut clocks = self.table.clocks;
        clocks[domain] += 1;
        let format = self.table.format.max(patch.format);
        let next = Next {
            format,
            generation,
            request_id,
            clocks,
            objects: &patch.objects[..patch.count],
            collections: &collections,
        };
        let base = Table::slot(generation) * TABLE_SECTORS;
        let attempt = self.table.attempt_digest(&next);
        let written = (|| {
            for index in 0..TABLE_SECTORS {
                let sector = self.table.encode_with(index, &next, attempt);
                disk.write(base + index, &sector)?;
            }
            self.counters.table_writes += 1;
            disk.flush()?;
            self.counters.flushes += 1;
            Ok::<(), Status>(())
        })();
        if written.is_err() {
            // The table copy may or may not be durable now. Close this
            // instance; recovery on disk decides, never an automatic retry.
            self.uncertain = true;
            return Err((Status::Unknown, 0));
        }
        self.table.format = format;
        self.table.generation = generation;
        self.table.request_id = request_id;
        self.table.clocks = clocks;
        self.table.collections = collections;
        for (object, entry) in next.objects {
            self.table.entries[*object] = *entry;
        }
        self.counters.commits += 1;
        Ok(())
    }

    fn ensure_format_initialized(&mut self, disk: &mut impl Disk) -> Result<(), (Status, u64)> {
        if !self.admission_checked {
            let mut recovered = Table::EMPTY;
            self.format_initialized =
                table::recover_with_state(disk, &mut recovered).map_err(|status| (status, 0))?;
            if recovered != self.table {
                self.uncertain = true;
                return Err((Status::Corrupt, 0));
            }
            self.admission_checked = true;
        }
        if self.format_initialized {
            return Ok(());
        }
        if table::write_format_marker(disk).is_err() {
            self.uncertain = true;
            return Err((Status::Unknown, 0));
        }
        self.format_initialized = true;
        self.counters.flushes += 1;
        Ok(())
    }

    fn read(&mut self, request: &Header, disk: &mut impl Disk) -> Outcome {
        let object = request.object as usize;
        let entry = self.table.entries[object];
        if entry.revision != request.revision {
            // Freshness first: the selector refers to this exact object state.
            return if entry.revision == 0 {
                Err((Status::NotFound, 0))
            } else {
                Err((Status::Stale, 0))
            };
        }
        let version = match request.name_length as usize {
            0 if entry.present => entry.current,
            0 => return Err((Status::NotFound, entry.revision)),
            slot if slot <= entry.history_count => entry.history[slot - 1],
            _ => return Err((Status::NotFound, entry.revision)),
        };
        let offset = request.aux as usize;
        if offset > version.length {
            return Err((Status::Range, 0));
        }
        if offset == version.length {
            return Ok(reply(
                request,
                self.epoch,
                entry.revision,
                offset as u64,
                0,
                &[],
            ));
        }
        let chunk = offset / CHUNK;
        if !(self.cache.valid
            && self.cache.object == object
            && self.cache.version == version.revision
            && self.cache.chunk == chunk)
        {
            self.load(&version, object, chunk, disk)?;
        }
        let end = core::cmp::min(
            core::cmp::min(version.length, (chunk + 1) * CHUNK),
            offset + request.count as usize,
        );
        let start = offset % CHUNK;
        let data = &self.cache.bytes[start..start + (end - offset)];
        Ok(reply(
            request,
            self.epoch,
            entry.revision,
            offset as u64,
            0,
            data,
        ))
    }

    /// Load and verify one content chunk. Bytes after the content inside the
    /// last written sector must be the canonical zero padding. Versions of one
    /// object have distinct version revisions, so the cache key is exact.
    fn load(
        &mut self,
        version: &Version,
        object: usize,
        chunk: usize,
        disk: &mut impl Disk,
    ) -> Result<(), (Status, u64)> {
        self.cache.valid = false;
        let used = core::cmp::min(CHUNK, version.length - chunk * CHUNK);
        let sectors = used.div_ceil(SECTOR);
        let base = DATA_START + u64::from(version.chunks[chunk]) * CHUNK_SECTORS;
        let mut buffer = [0; SECTOR];
        for index in 0..sectors {
            disk.read(base + index as u64, &mut buffer)
                .map_err(|_| (Status::Io, 0))?;
            self.counters.read_sectors += 1;
            self.cache.bytes[index * SECTOR..(index + 1) * SECTOR].copy_from_slice(&buffer);
        }
        if checksum_update(FNV_OFFSET, &self.cache.bytes[..used]) != version.sums[chunk]
            || self.cache.bytes[used..sectors * SECTOR]
                .iter()
                .any(|byte| *byte != 0)
        {
            return Err((Status::Corrupt, 0));
        }
        self.cache.valid = true;
        self.cache.object = object;
        self.cache.version = version.revision;
        self.cache.chunk = chunk;
        Ok(())
    }

    /// Paged listing of present objects; `count` filters by collection
    /// (0 = all, `c + 1` = collection `c`, where 0 means "no collection").
    fn list(&self, domain: usize, request: &Header) -> [u8; SIZE] {
        let filter = request.count as usize;
        let mut data = [0; wire::LIST_ENTRY * wire::LIST_MAX];
        let mut count = 0;
        let mut next = PER_DOMAIN as u64;
        for index in request.aux as usize..PER_DOMAIN {
            let object = domain * PER_DOMAIN + index;
            let entry = &self.table.entries[object];
            if !entry.present || filter != 0 && usize::from(entry.collection) + 1 != filter {
                continue;
            }
            if count == wire::LIST_MAX {
                next = index as u64;
                break;
            }
            wire::encode_list_entry(
                &wire::ListEntry {
                    object: object as u64,
                    revision: entry.revision,
                    length: entry.current.length,
                    name_length: entry.current.name_length,
                    name: entry.current.name,
                    collection: entry.collection,
                    history_count: entry.history_count as u8,
                },
                &mut data[count * wire::LIST_ENTRY..(count + 1) * wire::LIST_ENTRY],
            );
            count += 1;
        }
        let mut body = reply(
            request,
            self.epoch,
            self.table.domain_generation(domain),
            next,
            count as u32,
            &data[..count * wire::LIST_ENTRY],
        );
        // Echo the filter so the client can bind pages to one query.
        body[48..52].copy_from_slice(&(filter as u32).to_le_bytes());
        body
    }
}

/// L2 install and provenance reads (S09-L2-PROVENANCE section 4). Without
/// the `l2-install` feature the operations stay NoSys (`wire::LAST`); the
/// native volume-docs service is built that way until its text budget and
/// an integration request allow the native stage.
#[cfg(feature = "l2-install")]
impl Service {
    fn matching_install(
        &self,
        domain: usize,
        request: &Header,
    ) -> Result<InstallSession, (Status, u64)> {
        match self.sessions[domain] {
            Some(Staged::Install(session))
                if session.object == request.object as usize
                    && session.request_id == request.request_id
                    && request.revision == 0 =>
            {
                Ok(session)
            }
            _ => Err((Status::Invalid, 0)),
        }
    }

    fn collection_ok(&self, domain: usize, collection: u8) -> bool {
        collection == 0
            || self
                .table
                .collection(domain, usize::from(collection))
                .present
    }

    /// INSTALL_BEGIN (4.3): stage fresh chunks for every manifest version of
    /// a never-used own slot. Provenance is stored as asserted; it feeds no
    /// check below except its own shape (validated in `wire`).
    fn install_begin(
        &mut self,
        domain: usize,
        request: &Header,
        body: &[u8; SIZE],
        disk: &mut impl Disk,
    ) -> Outcome {
        if self.sessions[domain].is_some() {
            return Err((Status::Busy, 0));
        }
        let manifest = Manifest::decode(
            wire::data(body, request.length as usize),
            request.count as usize,
        )
        .ok_or((Status::Invalid, 0))?;
        let object = request.object as usize;
        if self.table.entries[object].revision != 0 {
            return Err((Status::Stale, 0));
        }
        let (tags, kind, collection, _) = wire::unmeta(request.aux);
        if !self.collection_ok(domain, collection) {
            return Err((Status::Range, 0));
        }
        let needed = manifest.chunks();
        if self.table.domain_chunks(domain) + needed > QUOTA {
            return Err((Status::Quota, 0));
        }
        let mut session = InstallSession {
            principal: 0,
            upload_token: 0,
            object,
            request_id: request.request_id,
            n: manifest.count,
            tags,
            kind,
            collection,
            origin_revision: manifest.origin_revision,
            origin_created: manifest.origin_created,
            origin_modified: manifest.origin_modified,
            versions: [InstallVersion {
                origin: 0,
                length: 0,
                received: 0,
                name: [0; NAME],
                name_length: 0,
                content_sum: FNV_OFFSET,
                chunk_start: 0,
                chunk_count: 0,
            }; INSTALL_MAX],
            chunks: [NO_CHUNK; INSTALL_MAX * MAX_CHUNKS],
            sums: [0; INSTALL_MAX * MAX_CHUNKS],
            chunk_count: 0,
            sector: [0; SECTOR],
        };
        let mut start = 0;
        for (staged, source) in session
            .versions
            .iter_mut()
            .zip(&manifest.versions[..manifest.count])
        {
            staged.origin = source.origin;
            staged.length = source.length;
            staged.name = source.name;
            staged.name_length = source.name_length;
            staged.chunk_start = start;
            staged.chunk_count = chunks_for(source.length);
            start += staged.chunk_count;
        }
        // Fresh chunks only (never deduplicated), lowest index first; staged
        // chunks of any domain are never handed out twice.
        for chunk in 0..CHUNKS as u16 {
            if session.chunk_count == needed {
                break;
            }
            if !self.table.references(chunk) && !self.reserved(chunk) {
                session.chunks[session.chunk_count] = chunk;
                session.sums[session.chunk_count] = FNV_OFFSET;
                session.chunk_count += 1;
            }
        }
        if session.chunk_count != needed {
            return Err((Status::Full, 0));
        }
        self.ensure_format_initialized(disk)?;
        self.counters.chunk_allocations += needed as u64;
        let total = session.total() as u64;
        self.sessions[domain] = Some(Staged::Install(session));
        Ok(reply(request, self.epoch, 0, total, needed as u32, &[]))
    }

    fn install_append(
        &mut self,
        domain: usize,
        request: &Header,
        body: &[u8; SIZE],
        disk: &mut impl Disk,
    ) -> Outcome {
        let mut session = self.matching_install(domain, request)?;
        let index = request.count as usize;
        let bytes = wire::data(body, request.length as usize);
        let Some(target) = session.target() else {
            return Err((Status::Range, 0));
        };
        let version = session.versions[target];
        if index != target
            || request.aux as usize != version.received
            || version.received + bytes.len() > version.length
        {
            return Err((Status::Range, 0));
        }
        let sector_of = |s: &InstallSession, offset: usize| {
            let chunk = u64::from(s.chunks[version.chunk_start + offset / CHUNK]);
            DATA_START + chunk * CHUNK_SECTORS + (offset / SECTOR) as u64 % CHUNK_SECTORS
        };
        for byte in bytes {
            let offset = session.versions[target].received;
            session.sector[offset % SECTOR] = *byte;
            let chunk = version.chunk_start + offset / CHUNK;
            session.sums[chunk] = checksum_update(session.sums[chunk], &[*byte]);
            let v = &mut session.versions[target];
            v.content_sum = checksum_update(v.content_sum, &[*byte]);
            v.received += 1;
            let (received, complete) = (v.received, v.received == v.length);
            // Full sectors as they fill; the last partial one when this APPEND
            // completes the version (zero padded), so one buffer suffices.
            if received % SECTOR == 0 || complete {
                let sector = sector_of(&session, offset);
                if disk.write(sector, &session.sector).is_err() {
                    // Only unreferenced reserved chunks: a certain abort.
                    self.sessions[domain] = None;
                    self.counters.aborted_sessions += 1;
                    return Err((Status::Io, 0));
                }
                self.counters.data_sectors += 1;
                session.sector = [0; SECTOR];
            }
        }
        let received = session.versions[target].received as u64;
        self.sessions[domain] = Some(Staged::Install(session));
        Ok(reply(request, self.epoch, 0, received, request.count, &[]))
    }

    /// One commit for the whole object (rule R): revision `n`, versions
    /// `n..1`, the clock raised by exactly one, `installed = n`. No `touch`.
    fn install_commit(&mut self, domain: usize, request: &Header, disk: &mut impl Disk) -> Outcome {
        let session = self.matching_install(domain, request)?;
        if request.count as usize != session.n || request.aux as usize != session.total() {
            return Err((Status::Invalid, 0));
        }
        if session.target().is_some() {
            return Err((Status::Range, 0));
        }
        self.sessions[domain] = None;
        if self.table.entries[session.object].revision != 0 {
            self.counters.aborted_sessions += 1;
            return Err((Status::Stale, 0));
        }
        if !self.collection_ok(domain, session.collection) {
            self.counters.aborted_sessions += 1;
            return Err((Status::Range, 0));
        }
        if disk.flush().is_err() {
            self.counters.aborted_sessions += 1;
            return Err((Status::Io, 0));
        }
        self.counters.flushes += 1;
        let clock = self.next_clock(domain)?;
        let n = session.n;
        let mut versions = [Version::EMPTY; INSTALL_MAX];
        for (index, (version, staged)) in
            versions.iter_mut().zip(&session.versions[..n]).enumerate()
        {
            let chunks = staged.chunk_start..staged.chunk_start + staged.chunk_count;
            *version = Version {
                revision: (n - index) as u64,
                length: staged.length,
                chunk_count: staged.chunk_count,
                name_length: staged.name_length,
                name: staged.name,
                content_sum: staged.content_sum,
                origin: staged.origin,
                ..Version::EMPTY
            };
            version.chunks[..staged.chunk_count].copy_from_slice(&session.chunks[chunks.clone()]);
            version.sums[..staged.chunk_count].copy_from_slice(&session.sums[chunks]);
        }
        let mut entry = Entry {
            revision: n as u64,
            request_id: request.request_id,
            created: clock,
            modified: clock,
            tags: session.tags,
            kind: session.kind,
            present: true,
            collection: session.collection,
            history_count: n - 1,
            current: versions[0],
            installed: n as u8,
            origin_revision: session.origin_revision,
            origin_created: session.origin_created,
            origin_modified: session.origin_modified,
            ..Entry::EMPTY
        };
        entry.history[..n - 1].copy_from_slice(&versions[1..n]);
        let mut patch = Patch::one(session.object, entry);
        patch.format = table::VERSION_V3;
        self.publish(domain, &patch, request.request_id, disk)?;
        Ok(reply(request, self.epoch, n as u64, request.aux, 0, &[]))
    }

    /// Create never-used collection `aux` with a recorded source revision.
    fn install_collection(
        &mut self,
        domain: usize,
        request: &Header,
        body: &[u8; SIZE],
        disk: &mut impl Disk,
    ) -> Outcome {
        let index = request.aux as usize;
        let slot = domain * COLLECTIONS + index - 1;
        if self.table.collections[slot].revision != 0 {
            return Err((Status::Stale, 0));
        }
        let length = request.name_length as usize;
        let mut next = Collection {
            revision: 1,
            request_id: request.request_id,
            present: true,
            name_length: length,
            origin_revision: crate::get64(body, 64 + length),
            ..Collection::EMPTY
        };
        next.name[..length].copy_from_slice(wire::data(body, length));
        let mut collections = self.table.collections;
        collections[slot] = next;
        let mut patch = Patch::new();
        patch.collections = Some(collections);
        patch.format = table::VERSION_V3;
        self.publish(domain, &patch, request.request_id, disk)?;
        Ok(reply(request, self.epoch, 1, index as u64, 0, &[]))
    }

    /// Provenance the own domain installed (4.4), under VERSION_STAT's
    /// presence and prefix rule: an absent slot reveals no version origin.
    fn provenance_stat(&self, domain: usize, request: &Header) -> Outcome {
        let stale = |revision: u64| {
            if revision == 0 {
                (Status::NotFound, 0)
            } else {
                (Status::Stale, 0)
            }
        };
        if request.count == 1 {
            let collection = self.table.collection(domain, request.aux as usize);
            if collection.revision != request.revision {
                return Err(stale(collection.revision));
            }
            let origin = collection.origin_revision;
            return Ok(reply(
                request,
                self.epoch,
                collection.revision,
                origin,
                u32::from(origin != 0),
                &[],
            ));
        }
        let entry = &self.table.entries[request.object as usize];
        if entry.revision != request.revision {
            return Err(stale(entry.revision));
        }
        let mut data = [0u8; 24 + 8 * INSTALL_MAX];
        let mut length = 0;
        let mut versions = 0;
        if entry.installed != 0 {
            crate::put64(&mut data, 0, entry.origin_revision);
            crate::put64(&mut data, 8, entry.origin_created);
            crate::put64(&mut data, 16, entry.origin_modified);
            length = 24;
            if entry.present {
                versions = 1 + Self::prefix(entry);
                let origins = core::iter::once(&entry.current).chain(&entry.history);
                for (index, version) in origins.take(versions).enumerate() {
                    crate::put64(&mut data, 24 + 8 * index, version.origin);
                }
                length += 8 * versions;
            }
        }
        let mut body = reply(
            request,
            self.epoch,
            entry.revision,
            0,
            u32::from(entry.installed),
            &data[..length],
        );
        body[48..52].copy_from_slice(&(versions as u32).to_le_bytes());
        Ok(body)
    }
}

#[cfg(test)]
mod upload_token_tests {
    use super::*;
    struct NoDisk;
    impl Disk for NoDisk {
        fn read(&mut self, _: u64, _: &mut [u8; SECTOR]) -> Result<(), Status> {
            panic!("no I/O before token allocation")
        }
        fn write(&mut self, _: u64, _: &[u8; SECTOR]) -> Result<(), Status> {
            panic!("no I/O before token allocation")
        }
        fn flush(&mut self) -> Result<(), Status> {
            panic!("no I/O before token allocation")
        }
    }

    #[test]
    fn exhausted_token_counter_never_wraps_or_stages_a_new_upload() {
        let mut service = Service::new(7, Table::EMPTY, false);
        service.next_upload_token[0] = u32::MAX;
        let mut h = Header::new(wire::BEGIN, 1, 7, 0);
        h.length = 4;
        h.name_length = 4;
        let request = crate::upload::encode(&h, b"Name", 0);
        for _ in 0..2 {
            let response = service.handle_for(0, 10, &request, &mut NoDisk);
            let reply = crate::upload::validate_reply(&h, 0, &response, 7).unwrap();
            assert_eq!(reply.status, Status::Quota as i32);
            assert!(service.session(0).is_none());
            assert_eq!(service.next_upload_token[0], u32::MAX);
        }
    }
}

#[cfg(all(test, feature = "l2-install"))]
mod install_commit_tests {
    use super::*;
    /// Accepts every write and reads zeros; nothing here reaches a table write.
    struct Sink;
    impl Disk for Sink {
        fn read(&mut self, _: u64, sector: &mut [u8; SECTOR]) -> Result<(), Status> {
            *sector = [0; SECTOR];
            Ok(())
        }
        fn write(&mut self, _: u64, _: &[u8; SECTOR]) -> Result<(), Status> {
            Ok(())
        }
        fn flush(&mut self) -> Result<(), Status> {
            Ok(())
        }
    }

    /// Defence in depth: no wire operation can make the slot used while an
    /// install is staged, so the INSTALL_COMMIT re-check is driven directly.
    #[test]
    fn commit_rechecks_that_the_slot_is_still_never_used() {
        let mut service = Service::new(7, Table::EMPTY, false);
        let mut manifest = Manifest {
            origin_revision: 5,
            origin_created: 1,
            origin_modified: 2,
            count: 1,
            ..Manifest::default()
        };
        manifest.versions[0].origin = 4;
        manifest.versions[0].name_length = 1;
        manifest.versions[0].name[0] = b'v';
        let mut data = [0; wire::DATA];
        manifest.encode(&mut data);
        let mut h = Header::new(wire::INSTALL_BEGIN, 3, 7, 1);
        h.count = 1;
        h.length = manifest.bytes() as u32;
        let begin = wire::encode(&h, &data[..manifest.bytes()]);
        let reply = wire::header(&service.handle(0, &begin, &mut Sink));
        assert_eq!(reply.status, Status::Ok as i32);
        service.table.entries[1].revision = 1;
        let mut commit = Header::new(wire::INSTALL_COMMIT, 3, 7, 1);
        commit.count = 1;
        let reply = wire::header(&service.handle(0, &wire::encode(&commit, &[]), &mut Sink));
        assert_eq!(reply.status, Status::Stale as i32);
        assert!(service.sessions[0].is_none());
        assert_eq!(service.counters.table_writes, 0);
    }
}
