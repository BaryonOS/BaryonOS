//! Client side of the v6 protocol: canonical request builders, strict reply
//! validation and bounded streaming helpers. A reply is never trusted beyond
//! what it echoes; revisions and catalog entries are not grants.
use crate::{
    get32, get64, valid_name, wire, Status, CHUNK, COLLECTIONS, HISTORY, MAX_LENGTH, NAME, OBJECTS,
    PER_DOMAIN, TRANSACT_MAX,
};
use wire::{Header, SubOp, DATA, SIZE};

/// One synchronous request/response exchange with the volume service.
pub trait Transport {
    fn call(&mut self, request: &[u8; SIZE]) -> Result<[u8; SIZE], Status>;
}

pub fn request(header: &Header, data: &[u8]) -> [u8; SIZE] {
    wire::encode(header, data)
}

fn named(body: &[u8; SIZE], name_length: usize) -> bool {
    name_length <= NAME
        && valid_name(&body[64..64 + name_length])
        && body[64 + name_length..64 + NAME]
            .iter()
            .all(|byte| *byte == 0)
}

/// Validate the structure of a reply to `request`. `Ok` carries the reply
/// header; the reply status may still be an error code. Any malformed reply
/// is `Protocol`, and the client must not interpret its data.
pub fn validate_reply(request: &Header, body: &[u8; SIZE], epoch: u64) -> Result<Header, Status> {
    let h = wire::header(body);
    let length = h.length as usize;
    if u16::from_le_bytes([body[0], body[1]]) != wire::VERSION
        || body[4..8] != [0; 4]
        || h.operation != request.operation
        || h.request_id != request.request_id
        || h.epoch != epoch
        || h.object != request.object
        || length > DATA
        || body[64 + length..].iter().any(|byte| *byte != 0)
    {
        return Err(Status::Protocol);
    }
    let status = Status::from_code(i64::from(h.status));
    if status == Status::Protocol {
        return Err(Status::Protocol);
    }
    if status != Status::Ok {
        let revision_ok = h.revision == 0 || status == Status::NotFound;
        if !revision_ok || length != 0 || h.name_length != 0 || h.count != 0 || h.aux != 0 {
            return Err(Status::Protocol);
        }
        return Ok(h);
    }
    let bare = length == 0 && h.name_length == 0;
    let ok = match h.operation {
        wire::STAT => {
            let data = &body[64..64 + wire::STAT_LENGTH as usize];
            let history = usize::from(data[71]);
            let (created, modified) = (get64(data, 48), get64(data, 56));
            let version = get64(data, 72);
            // Versions are strictly older than the current one, newest first.
            let mut newer = version;
            let ordered = (0..history.min(HISTORY)).all(|index| {
                let at = 80 + index * wire::STAT_HISTORY;
                let revision = get64(data, at);
                let fits =
                    revision != 0 && revision < newer && get32(data, at + 8) as usize <= MAX_LENGTH;
                newer = revision;
                fits
            });
            h.length == wire::STAT_LENGTH
                && h.revision != 0
                && h.count == 0
                && h.aux <= MAX_LENGTH as u64
                && named(body, h.name_length as usize)
                && usize::from(data[70]) <= COLLECTIONS
                && history <= HISTORY
                && version != 0
                && version <= h.revision
                && created != 0
                && created <= modified
                && ordered
                && data[80 + history * wire::STAT_HISTORY..]
                    .iter()
                    .all(|b| *b == 0)
        }
        wire::BEGIN => {
            h.revision == request.revision
                && h.aux == request.aux
                && h.count as usize == (request.aux as usize).div_ceil(CHUNK)
                && bare
        }
        wire::APPEND => {
            h.revision == request.revision
                && h.aux == request.aux + u64::from(request.length)
                && bare
                && h.count == 0
        }
        wire::COMMIT | wire::DELETE | wire::RENAME | wire::RESTORE | wire::PURGE | wire::META => {
            h.revision == request.revision + 1
                && bare
                && h.count == 0
                && h.aux
                    == if h.operation == wire::COMMIT {
                        request.aux
                    } else {
                        0
                    }
        }
        wire::COLLECTION => {
            h.revision == request.revision + 1 && bare && h.count == 0 && h.aux == request.aux
        }
        wire::TRANSACT => bare && h.count == request.count && h.aux == 0,
        wire::COLLECTION_STAT => {
            // Never created: all zero. Deleted: a revision but no name.
            h.name_length as usize == length
                && h.count <= 1
                && (h.revision == 0) == (h.aux == 0)
                && if h.count == 1 {
                    h.revision != 0 && length != 0 && named(body, length)
                } else {
                    length == 0
                }
        }
        wire::ABORT => h.revision == request.revision && bare && h.count == 0 && h.aux == 0,
        wire::READ => {
            h.revision == request.revision
                && h.aux == request.aux
                && h.length <= request.count
                && h.name_length == 0
                && h.count == 0
        }
        wire::LIST => {
            let count = h.count as usize;
            let mut ok = count <= wire::LIST_MAX
                && length == count * wire::LIST_ENTRY
                && h.name_length == request.count
                && h.aux > request.aux
                && h.aux <= PER_DOMAIN as u64;
            // Entries are only inspected once the header is canonical, so an
            // untrusted `aux` never enters the bound arithmetic below.
            let entries = if ok { count } else { 0 };
            let mut previous = None;
            for index in 0..entries {
                let bytes =
                    &body[64 + index * wire::LIST_ENTRY..64 + (index + 1) * wire::LIST_ENTRY];
                let entry = wire::decode_list_entry(bytes);
                let filter = request.count as usize;
                ok &= entry.object >= request.object + request.aux
                    && entry.object < request.object + h.aux
                    && entry.object < OBJECTS as u64
                    && previous.is_none_or(|p| entry.object > p)
                    && entry.revision != 0
                    && entry.length <= MAX_LENGTH
                    && usize::from(entry.collection) <= COLLECTIONS
                    && (filter == 0 || usize::from(entry.collection) + 1 == filter)
                    && usize::from(entry.history_count) <= HISTORY
                    && entry.name_length <= NAME
                    && valid_name(&entry.name[..entry.name_length])
                    && entry.name[entry.name_length..]
                        .iter()
                        .all(|byte| *byte == 0);
                previous = Some(entry.object);
            }
            ok
        }
        _ => false,
    };
    if ok {
        Ok(h)
    } else {
        Err(Status::Protocol)
    }
}

/// One retained earlier version as reported by STAT.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HistoryInfo {
    pub revision: u64,
    pub length: usize,
    pub content_sum: u64,
}

/// Metadata returned by STAT.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stat {
    pub revision: u64,
    pub version_revision: u64,
    pub length: usize,
    pub name_length: usize,
    pub name: [u8; NAME],
    pub content_sum: u64,
    pub request_id: u64,
    pub created: u64,
    pub modified: u64,
    pub tags: u32,
    pub kind: u16,
    pub collection: u8,
    pub history_count: usize,
    pub history: [HistoryInfo; HISTORY],
}

/// One collection of the caller's domain as reported by COLLECTION_STAT.
/// `revision == 0` means the collection was never created.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CollectionInfo {
    pub revision: u64,
    pub request_id: u64,
    pub present: bool,
    pub name_length: usize,
    pub name: [u8; NAME],
}

impl CollectionInfo {
    pub fn name(&self) -> &[u8] {
        &self.name[..self.name_length]
    }
}

pub struct Client<T: Transport> {
    pub transport: T,
    pub epoch: u64,
    next_request: u64,
}

impl<T: Transport> Client<T> {
    /// `first_request` must be nonzero; IDs are local correlation values only.
    pub fn new(transport: T, epoch: u64, first_request: u64) -> Self {
        Self {
            transport,
            epoch,
            next_request: first_request.max(1),
        }
    }

    pub fn next_request_id(&self) -> u64 {
        self.next_request
    }

    fn fresh(&mut self) -> u64 {
        let id = self.next_request;
        self.next_request = self.next_request.wrapping_add(1).max(1);
        id
    }

    /// Send one request and return the validated reply header and body.
    pub fn exchange(
        &mut self,
        header: &Header,
        data: &[u8],
    ) -> Result<(Header, [u8; SIZE]), Status> {
        let body = self.transport.call(&request(header, data))?;
        let reply = validate_reply(header, &body, self.epoch)?;
        Ok((reply, body))
    }

    fn status(reply: &Header) -> Result<(), Status> {
        match Status::from_code(i64::from(reply.status)) {
            Status::Ok => Ok(()),
            other => Err(other),
        }
    }

    fn simple(&mut self, header: &Header, data: &[u8]) -> Result<u64, (Status, u64)> {
        let (reply, _) = self.exchange(header, data).map_err(|status| (status, 0))?;
        Self::status(&reply).map_err(|status| (status, reply.revision))?;
        Ok(reply.revision)
    }

    /// Returns `Err((NotFound, tombstone revision))` for absent objects.
    pub fn stat(&mut self, object: u64) -> Result<Stat, (Status, u64)> {
        let header = Header::new(wire::STAT, self.fresh(), self.epoch, object);
        let (reply, body) = self.exchange(&header, &[]).map_err(|status| (status, 0))?;
        Self::status(&reply).map_err(|status| (status, reply.revision))?;
        let data = &body[64..64 + wire::STAT_LENGTH as usize];
        let mut name = [0; NAME];
        name.copy_from_slice(&data[..NAME]);
        let history_count = usize::from(data[71]);
        let mut history = [HistoryInfo::default(); HISTORY];
        for (index, item) in history.iter_mut().enumerate().take(history_count) {
            let at = 80 + index * wire::STAT_HISTORY;
            *item = HistoryInfo {
                revision: get64(data, at),
                length: get32(data, at + 8) as usize,
                content_sum: get64(data, at + 12),
            };
        }
        Ok(Stat {
            revision: reply.revision,
            version_revision: get64(data, 72),
            length: reply.aux as usize,
            name_length: reply.name_length as usize,
            name,
            content_sum: get64(data, 32),
            request_id: get64(data, 40),
            created: get64(data, 48),
            modified: get64(data, 56),
            tags: get32(data, 64),
            kind: u16::from_le_bytes([data[68], data[69]]),
            collection: data[70],
            history_count,
            history,
        })
    }

    fn begin(
        &mut self,
        object: u64,
        expected: u64,
        name: &[u8],
        length: usize,
    ) -> Result<u64, Status> {
        let id = self.fresh();
        let mut begin = Header::new(wire::BEGIN, id, self.epoch, object);
        begin.revision = expected;
        begin.aux = length as u64;
        begin.length = name.len() as u32;
        begin.name_length = name.len() as u32;
        let (reply, _) = self.exchange(&begin, name)?;
        Self::status(&reply)?;
        Ok(id)
    }

    fn stream(
        &mut self,
        object: u64,
        expected: u64,
        id: u64,
        until: usize,
        fill: &mut impl FnMut(usize, &mut [u8]),
    ) -> Result<(), Status> {
        let mut offset = 0;
        let mut buffer = [0; DATA];
        while offset < until {
            let amount = core::cmp::min(DATA, until - offset);
            fill(offset, &mut buffer[..amount]);
            let mut append = Header::new(wire::APPEND, id, self.epoch, object);
            append.revision = expected;
            append.aux = offset as u64;
            append.length = amount as u32;
            let (reply, _) = self.exchange(&append, &buffer[..amount])?;
            Self::status(&reply)?;
            offset += amount;
        }
        Ok(())
    }

    /// Stream a new revision. `fill(offset, buffer)` supplies content bytes so
    /// no client needs a document-sized buffer. On any error the upload is not
    /// committed unless the error is `Unknown` (resolve with `stat`, never by
    /// blindly repeating). Returns `(new revision, request ID)`.
    pub fn write(
        &mut self,
        object: u64,
        expected: u64,
        name: &[u8],
        length: usize,
        mut fill: impl FnMut(usize, &mut [u8]),
    ) -> Result<(u64, u64), Status> {
        let id = self.begin(object, expected, name, length)?;
        self.stream(object, expected, id, length, &mut fill)?;
        let mut commit = Header::new(wire::COMMIT, id, self.epoch, object);
        commit.revision = expected;
        commit.aux = length as u64;
        let (reply, _) = self.exchange(&commit, &[])?;
        Self::status(&reply)?;
        Ok((reply.revision, id))
    }

    /// Begin and stream only `partial` bytes, then stop (for abort/crash tests).
    pub fn write_partial(
        &mut self,
        object: u64,
        expected: u64,
        name: &[u8],
        length: usize,
        partial: usize,
        mut fill: impl FnMut(usize, &mut [u8]),
    ) -> Result<u64, Status> {
        let id = self.begin(object, expected, name, length)?;
        self.stream(object, expected, id, partial.min(length), &mut fill)?;
        Ok(id)
    }

    pub fn abort(&mut self, object: u64, expected: u64, id: u64) -> Result<(), Status> {
        let mut header = Header::new(wire::ABORT, id, self.epoch, object);
        header.revision = expected;
        let (reply, _) = self.exchange(&header, &[])?;
        Self::status(&reply)
    }

    /// Read a complete version in bounded pieces (`selector` 0 = current,
    /// 1..=HISTORY = history slot) of the object at `revision`.
    pub fn read_version(
        &mut self,
        object: u64,
        revision: u64,
        selector: usize,
        length: usize,
        mut sink: impl FnMut(usize, &[u8]),
    ) -> Result<usize, Status> {
        let mut offset = 0;
        while offset < length {
            let mut header = Header::new(wire::READ, self.fresh(), self.epoch, object);
            header.revision = revision;
            header.aux = offset as u64;
            header.count = DATA as u32;
            header.name_length = selector as u32;
            let (reply, body) = self.exchange(&header, &[])?;
            Self::status(&reply)?;
            let amount = reply.length as usize;
            if amount == 0 || offset + amount > length {
                return Err(Status::Protocol);
            }
            sink(offset, wire::data(&body, amount));
            offset += amount;
        }
        Ok(offset)
    }

    pub fn read(
        &mut self,
        object: u64,
        revision: u64,
        length: usize,
        sink: impl FnMut(usize, &[u8]),
    ) -> Result<usize, Status> {
        self.read_version(object, revision, 0, length, sink)
    }

    pub fn delete(&mut self, object: u64, expected: u64) -> Result<u64, (Status, u64)> {
        let mut header = Header::new(wire::DELETE, self.fresh(), self.epoch, object);
        header.revision = expected;
        self.simple(&header, &[])
    }

    pub fn purge(&mut self, object: u64, expected: u64) -> Result<u64, (Status, u64)> {
        let mut header = Header::new(wire::PURGE, self.fresh(), self.epoch, object);
        header.revision = expected;
        self.simple(&header, &[])
    }

    /// Make history slot `selector` (1 = newest) the current version again;
    /// also undeletes a deleted object.
    pub fn restore(
        &mut self,
        object: u64,
        expected: u64,
        selector: usize,
    ) -> Result<u64, (Status, u64)> {
        let mut header = Header::new(wire::RESTORE, self.fresh(), self.epoch, object);
        header.revision = expected;
        header.aux = selector as u64;
        self.simple(&header, &[])
    }

    pub fn rename(&mut self, object: u64, expected: u64, name: &[u8]) -> Result<u64, Status> {
        let mut header = Header::new(wire::RENAME, self.fresh(), self.epoch, object);
        header.revision = expected;
        header.length = name.len() as u32;
        header.name_length = name.len() as u32;
        self.simple(&header, name).map_err(|(status, _)| status)
    }

    pub fn meta(
        &mut self,
        object: u64,
        expected: u64,
        tags: u32,
        kind: u16,
        collection: u8,
    ) -> Result<u64, (Status, u64)> {
        let mut header = Header::new(wire::META, self.fresh(), self.epoch, object);
        header.revision = expected;
        header.aux = wire::meta(tags, kind, collection);
        self.simple(&header, &[])
    }

    /// Create or rename (non-empty `name`) or delete (empty `name`) collection
    /// `index` of `domain`. Returns the new collection revision.
    pub fn collection(
        &mut self,
        domain: usize,
        index: usize,
        expected: u64,
        name: &[u8],
    ) -> Result<u64, (Status, u64)> {
        let mut header = Header::new(
            wire::COLLECTION,
            self.fresh(),
            self.epoch,
            (domain * PER_DOMAIN) as u64,
        );
        header.revision = expected;
        header.aux = index as u64;
        header.length = name.len() as u32;
        header.name_length = name.len() as u32;
        self.simple(&header, name)
    }

    /// Read collection `index` (1..=COLLECTIONS) of `domain`; allowed on a
    /// read-only mount.
    pub fn collection_stat(
        &mut self,
        domain: usize,
        index: usize,
    ) -> Result<CollectionInfo, Status> {
        let mut header = Header::new(
            wire::COLLECTION_STAT,
            self.fresh(),
            self.epoch,
            (domain * PER_DOMAIN) as u64,
        );
        header.aux = index as u64;
        let (reply, body) = self.exchange(&header, &[])?;
        Self::status(&reply)?;
        let length = reply.length as usize;
        let mut name = [0; NAME];
        name[..length].copy_from_slice(&body[64..64 + length]);
        Ok(CollectionInfo {
            revision: reply.revision,
            request_id: reply.aux,
            present: reply.count == 1,
            name_length: length,
            name,
        })
    }

    /// Apply up to four sub-operations atomically. Returns the domain change
    /// counter after the commit.
    pub fn transact(&mut self, domain: usize, subs: &[SubOp]) -> Result<u64, Status> {
        if subs.is_empty() || subs.len() > TRANSACT_MAX {
            return Err(Status::Invalid);
        }
        let mut data = [0; wire::SUBOP * TRANSACT_MAX];
        for (index, sub) in subs.iter().enumerate() {
            wire::encode_subop(
                sub,
                &mut data[index * wire::SUBOP..(index + 1) * wire::SUBOP],
            );
        }
        let mut header = Header::new(
            wire::TRANSACT,
            self.fresh(),
            self.epoch,
            (domain * PER_DOMAIN) as u64,
        );
        header.count = subs.len() as u32;
        header.length = (subs.len() * wire::SUBOP) as u32;
        let (reply, _) = self.exchange(&header, &data[..subs.len() * wire::SUBOP])?;
        Self::status(&reply)?;
        Ok(reply.revision)
    }

    /// Complete paged listing of the caller's domain (`filter` 0 = all,
    /// `c + 1` = collection `c`). Pages must come from one unchanged domain
    /// change counter; a concurrent change returns `Stale`.
    pub fn list_filtered(
        &mut self,
        domain: usize,
        filter: usize,
        out: &mut [wire::ListEntry; PER_DOMAIN],
    ) -> Result<(usize, u64), Status> {
        let mut count = 0;
        let mut cursor = 0u64;
        let mut generation = None;
        while cursor < PER_DOMAIN as u64 {
            let mut header = Header::new(
                wire::LIST,
                self.fresh(),
                self.epoch,
                (domain * PER_DOMAIN) as u64,
            );
            header.aux = cursor;
            header.count = filter as u32;
            let (reply, body) = self.exchange(&header, &[])?;
            Self::status(&reply)?;
            if generation.is_some_and(|g| g != reply.revision) {
                return Err(Status::Stale);
            }
            generation = Some(reply.revision);
            for index in 0..reply.count as usize {
                if count == PER_DOMAIN {
                    return Err(Status::Protocol);
                }
                let bytes =
                    &body[64 + index * wire::LIST_ENTRY..64 + (index + 1) * wire::LIST_ENTRY];
                out[count] = wire::decode_list_entry(bytes);
                count += 1;
            }
            cursor = reply.aux;
        }
        Ok((count, generation.unwrap_or(0)))
    }

    pub fn list(
        &mut self,
        domain: usize,
        out: &mut [wire::ListEntry; PER_DOMAIN],
    ) -> Result<(usize, u64), Status> {
        self.list_filtered(domain, 0, out)
    }
}
