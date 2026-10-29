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
    DATA_START, DOMAINS, FNV_OFFSET, MAX_CHUNKS, NAME, NO_CHUNK, PER_DOMAIN, QUOTA, SECTOR,
    TABLE_SECTORS,
};
use table::{Entry, Table};
use wire::{Header, SIZE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Session {
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

struct Cache {
    valid: bool,
    object: usize,
    revision: u64,
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
}

pub struct Service {
    epoch: u64,
    table: Table,
    sessions: [Option<Session>; DOMAINS],
    cache: Cache,
    read_only: bool,
    uncertain: bool,
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

impl Service {
    pub fn new(epoch: u64, table: Table, read_only: bool) -> Self {
        Self {
            epoch,
            table,
            sessions: [None; DOMAINS],
            cache: Cache {
                valid: false,
                object: 0,
                revision: 0,
                chunk: 0,
                bytes: [0; CHUNK],
            },
            read_only,
            uncertain: false,
            counters: Counters::default(),
        }
    }

    pub fn table(&self) -> &Table {
        &self.table
    }
    pub fn uncertain(&self) -> bool {
        self.uncertain
    }
    pub fn session(&self, domain: usize) -> Option<&Session> {
        self.sessions[domain].as_ref()
    }

    /// Handle one already sender-authenticated request. `domain` must come from
    /// the kernel-bound endpoint the request arrived on, never from the body.
    pub fn handle(&mut self, domain: usize, body: &[u8; SIZE], disk: &mut impl Disk) -> [u8; SIZE] {
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
        let result = match request.operation {
            wire::STAT => self.stat(&request),
            wire::BEGIN => self.begin(domain, &request, body),
            wire::APPEND => self.append(domain, &request, body, disk),
            wire::COMMIT => self.commit(domain, &request, disk),
            wire::ABORT => self.abort(domain, &request),
            wire::READ => self.read(&request, disk),
            wire::DELETE | wire::RENAME => self.metadata(&request, body, disk),
            wire::LIST => Ok(self.list(domain, &request)),
            _ => Err((Status::NoSys, 0)),
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

    fn stat(&self, request: &Header) -> Result<[u8; SIZE], (Status, u64)> {
        let entry = self.table.entries[request.object as usize];
        if !entry.present {
            return Err((Status::NotFound, entry.revision));
        }
        let mut data = [0; wire::STAT_LENGTH as usize];
        data[..NAME].copy_from_slice(&entry.name);
        data[32..40].copy_from_slice(&entry.content_sum.to_le_bytes());
        data[40..48].copy_from_slice(&entry.request_id.to_le_bytes());
        let mut body = reply(
            request,
            self.epoch,
            entry.revision,
            entry.length as u64,
            0,
            &data,
        );
        body[48..52].copy_from_slice(&(entry.name_length as u32).to_le_bytes());
        Ok(body)
    }

    fn reserved(&self, chunk: u16) -> bool {
        self.sessions
            .iter()
            .flatten()
            .any(|session| session.chunks[..session.chunk_count].contains(&chunk))
    }

    fn begin(
        &mut self,
        domain: usize,
        request: &Header,
        body: &[u8; SIZE],
    ) -> Result<[u8; SIZE], (Status, u64)> {
        if self.read_only {
            return Err((Status::Access, 0));
        }
        // One upload per domain. A new BEGIN from the same kernel-bound domain
        // replaces an abandoned one (e.g. after an app restart); no other
        // domain can influence this session.
        if self.sessions[domain].take().is_some() {
            self.counters.aborted_sessions += 1;
        }
        let object = request.object as usize;
        let entry = self.table.entries[object];
        if entry.revision != request.revision {
            return Err((Status::Stale, 0));
        }
        let length = request.aux as usize;
        let needed = chunks_for(length);
        if self.table.domain_chunks(domain) + needed > QUOTA {
            return Err((Status::Quota, 0));
        }
        let mut session = Session {
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
        self.counters.chunk_allocations += needed as u64;
        self.sessions[domain] = Some(session);
        Ok(reply(
            request,
            self.epoch,
            request.revision,
            length as u64,
            needed as u32,
            &[],
        ))
    }

    fn matching(&self, domain: usize, request: &Header) -> Result<Session, (Status, u64)> {
        match self.sessions[domain] {
            Some(session)
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
    ) -> Result<[u8; SIZE], (Status, u64)> {
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
                if let Err(status) = disk.write(sector, &session.sector) {
                    // Only unreferenced chunks were touched: durable state is
                    // unchanged, so this is a certain abort, not Unknown.
                    self.sessions[domain] = None;
                    self.counters.aborted_sessions += 1;
                    return Err((io(status), 0));
                }
                self.counters.data_sectors += 1;
                session.sector = [0; SECTOR];
            }
        }
        let received = session.received as u64;
        self.sessions[domain] = Some(session);
        Ok(reply(
            request,
            self.epoch,
            request.revision,
            received,
            0,
            &[],
        ))
    }

    fn abort(&mut self, domain: usize, request: &Header) -> Result<[u8; SIZE], (Status, u64)> {
        self.matching(domain, request)?;
        self.sessions[domain] = None;
        self.counters.aborted_sessions += 1;
        Ok(reply(request, self.epoch, request.revision, 0, 0, &[]))
    }

    fn commit(
        &mut self,
        domain: usize,
        request: &Header,
        disk: &mut impl Disk,
    ) -> Result<[u8; SIZE], (Status, u64)> {
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
            Ok(())
        })();
        if let Err(status) = staged {
            self.counters.aborted_sessions += 1;
            return Err((io(status), 0));
        }
        let mut entry = Entry {
            revision: session.expected + 1,
            request_id: session.request_id,
            length: session.length,
            present: true,
            name_length: session.name_length,
            name: session.name,
            chunk_count: session.chunk_count,
            content_sum: session.content_sum,
            chunks: session.chunks,
            sums: session.sums,
        };
        entry.chunks[entry.chunk_count..].fill(NO_CHUNK);
        entry.sums[entry.chunk_count..].fill(0);
        let revision = self.publish(session.object, entry, request.request_id, disk)?;
        Ok(reply(
            request,
            self.epoch,
            revision,
            session.length as u64,
            0,
            &[],
        ))
    }

    fn metadata(
        &mut self,
        request: &Header,
        body: &[u8; SIZE],
        disk: &mut impl Disk,
    ) -> Result<[u8; SIZE], (Status, u64)> {
        if self.read_only {
            return Err((Status::Access, 0));
        }
        let object = request.object as usize;
        let current = self.table.entries[object];
        if !current.present {
            return Err((Status::NotFound, current.revision));
        }
        if current.revision != request.revision {
            return Err((Status::Stale, 0));
        }
        let entry = if request.operation == wire::DELETE {
            Entry::tombstone(current.revision + 1, request.request_id)
        } else {
            let length = request.name_length as usize;
            let mut renamed = current;
            renamed.revision += 1;
            renamed.request_id = request.request_id;
            renamed.name = [0; NAME];
            renamed.name[..length].copy_from_slice(wire::data(body, length));
            renamed.name_length = length;
            renamed
        };
        let revision = self.publish(object, entry, request.request_id, disk)?;
        Ok(reply(request, self.epoch, revision, 0, 0, &[]))
    }

    /// Write the next table copy into the older slot. Never touches the slot
    /// holding the current durable generation.
    fn publish(
        &mut self,
        object: usize,
        entry: Entry,
        request_id: u64,
        disk: &mut impl Disk,
    ) -> Result<u64, (Status, u64)> {
        let mut next = self.table;
        next.generation = self
            .table
            .generation
            .checked_add(1)
            .ok_or((Status::Full, 0))?;
        next.request_id = request_id;
        next.entries[object] = entry;
        let base = Table::slot(next.generation) * TABLE_SECTORS;
        let written = (|| {
            for index in 0..TABLE_SECTORS {
                disk.write(base + index, &next.encode_sector(index))?;
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
        self.table = next;
        self.counters.commits += 1;
        Ok(entry.revision)
    }

    fn read(
        &mut self,
        request: &Header,
        disk: &mut impl Disk,
    ) -> Result<[u8; SIZE], (Status, u64)> {
        let object = request.object as usize;
        let entry = self.table.entries[object];
        if !entry.present {
            return Err((Status::NotFound, entry.revision));
        }
        if entry.revision != request.revision {
            return Err((Status::Stale, 0));
        }
        let offset = request.aux as usize;
        if offset > entry.length {
            return Err((Status::Range, 0));
        }
        if offset == entry.length {
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
            && self.cache.revision == entry.revision
            && self.cache.chunk == chunk)
        {
            self.load(&entry, object, chunk, disk)?;
        }
        let end = core::cmp::min(
            core::cmp::min(entry.length, (chunk + 1) * CHUNK),
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
    /// last written sector must be the canonical zero padding.
    fn load(
        &mut self,
        entry: &Entry,
        object: usize,
        chunk: usize,
        disk: &mut impl Disk,
    ) -> Result<(), (Status, u64)> {
        self.cache.valid = false;
        let used = core::cmp::min(CHUNK, entry.length - chunk * CHUNK);
        let sectors = used.div_ceil(SECTOR);
        let base = DATA_START + u64::from(entry.chunks[chunk]) * CHUNK_SECTORS;
        let mut buffer = [0; SECTOR];
        for index in 0..sectors {
            disk.read(base + index as u64, &mut buffer)
                .map_err(|status| (io(status), 0))?;
            self.counters.read_sectors += 1;
            self.cache.bytes[index * SECTOR..(index + 1) * SECTOR].copy_from_slice(&buffer);
        }
        if checksum_update(FNV_OFFSET, &self.cache.bytes[..used]) != entry.sums[chunk]
            || self.cache.bytes[used..sectors * SECTOR]
                .iter()
                .any(|byte| *byte != 0)
        {
            return Err((Status::Corrupt, 0));
        }
        self.cache = Cache {
            valid: true,
            object,
            revision: entry.revision,
            chunk,
            bytes: self.cache.bytes,
        };
        Ok(())
    }

    fn list(&self, domain: usize, request: &Header) -> [u8; SIZE] {
        let mut data = [0; wire::LIST_ENTRY * wire::LIST_MAX];
        let mut count = 0;
        let mut next = PER_DOMAIN as u64;
        for index in request.aux as usize..PER_DOMAIN {
            if count == wire::LIST_MAX {
                next = index as u64;
                break;
            }
            let object = domain * PER_DOMAIN + index;
            let entry = self.table.entries[object];
            if entry.present {
                wire::encode_list_entry(
                    &wire::ListEntry {
                        object: object as u64,
                        revision: entry.revision,
                        length: entry.length,
                        name_length: entry.name_length,
                        name: entry.name,
                    },
                    &mut data[count * wire::LIST_ENTRY..(count + 1) * wire::LIST_ENTRY],
                );
                count += 1;
            }
        }
        reply(
            request,
            self.epoch,
            self.table.domain_generation(domain),
            next,
            count as u32,
            &data[..count * wire::LIST_ENTRY],
        )
    }
}

/// Device failures before the table write are reported as plain I/O errors:
/// the broker may call a failed mutating request uncertain, but only chunks
/// outside the durable table were involved, so the commit outcome is certain.
fn io(_status: Status) -> Status {
    Status::Io
}
