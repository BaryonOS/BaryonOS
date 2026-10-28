//! Client side of the v5 protocol: canonical request builders, strict reply
//! validation and bounded streaming helpers. A reply is never trusted beyond
//! what it echoes; revisions and catalog entries are not grants.
use crate::{get64, valid_name, wire, Status, CHUNK, MAX_LENGTH, NAME, OBJECTS, PER_DOMAIN};
use wire::{Header, DATA, SIZE};

/// One synchronous request/response exchange with the volume service.
pub trait Transport {
    fn call(&mut self, request: &[u8; SIZE]) -> Result<[u8; SIZE], Status>;
}

pub fn request(header: &Header, data: &[u8]) -> [u8; SIZE] {
    wire::encode(header, data)
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
    let ok = match h.operation {
        wire::STAT => {
            let name_length = h.name_length as usize;
            h.length == wire::STAT_LENGTH
                && h.revision != 0
                && h.count == 0
                && h.aux <= MAX_LENGTH as u64
                && name_length <= NAME
                && valid_name(&body[64..64 + name_length])
                && body[64 + name_length..64 + NAME]
                    .iter()
                    .all(|byte| *byte == 0)
        }
        wire::BEGIN => {
            h.revision == request.revision
                && h.aux == request.aux
                && h.count as usize == (request.aux as usize).div_ceil(CHUNK)
                && length == 0
                && h.name_length == 0
        }
        wire::APPEND => {
            h.revision == request.revision
                && h.aux == request.aux + u64::from(request.length)
                && length == 0
                && h.name_length == 0
                && h.count == 0
        }
        wire::COMMIT | wire::DELETE | wire::RENAME => {
            h.revision == request.revision + 1
                && length == 0
                && h.name_length == 0
                && h.count == 0
                && h.aux
                    == if h.operation == wire::COMMIT {
                        request.aux
                    } else {
                        0
                    }
        }
        wire::ABORT => {
            h.revision == request.revision
                && length == 0
                && h.name_length == 0
                && h.count == 0
                && h.aux == 0
        }
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
                && h.name_length == 0
                && h.aux > request.aux
                && h.aux <= PER_DOMAIN as u64;
            let mut previous = None;
            for index in 0..count.min(wire::LIST_MAX) {
                let bytes =
                    &body[64 + index * wire::LIST_ENTRY..64 + (index + 1) * wire::LIST_ENTRY];
                let entry = wire::decode_list_entry(bytes);
                let first = request.object + request.aux;
                ok &= entry.object >= first
                    && entry.object < request.object + h.aux
                    && entry.object < OBJECTS as u64
                    && previous.is_none_or(|p| entry.object > p)
                    && entry.revision != 0
                    && entry.length <= MAX_LENGTH
                    && bytes[2..4] == [0, 0]
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

/// Metadata returned by STAT.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stat {
    pub revision: u64,
    pub length: usize,
    pub name_length: usize,
    pub name: [u8; NAME],
    pub content_sum: u64,
    pub request_id: u64,
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

    /// Returns `Err(NotFound)` with no data for tombstones/absent objects;
    /// `tombstone` then reports the authorized tombstone revision.
    pub fn stat(&mut self, object: u64) -> Result<Stat, (Status, u64)> {
        let header = Header::new(wire::STAT, self.fresh(), self.epoch, object);
        let (reply, body) = self.exchange(&header, &[]).map_err(|status| (status, 0))?;
        Self::status(&reply).map_err(|status| (status, reply.revision))?;
        let mut name = [0; NAME];
        name.copy_from_slice(&body[64..64 + NAME]);
        Ok(Stat {
            revision: reply.revision,
            length: reply.aux as usize,
            name_length: reply.name_length as usize,
            name,
            content_sum: get64(&body, 96),
            request_id: get64(&body, 104),
        })
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
        let id = self.fresh();
        let mut begin = Header::new(wire::BEGIN, id, self.epoch, object);
        begin.revision = expected;
        begin.aux = length as u64;
        begin.length = name.len() as u32;
        begin.name_length = name.len() as u32;
        let (reply, _) = self.exchange(&begin, name)?;
        Self::status(&reply)?;
        let mut offset = 0;
        let mut buffer = [0; DATA];
        while offset < length {
            let amount = core::cmp::min(DATA, length - offset);
            fill(offset, &mut buffer[..amount]);
            let mut append = Header::new(wire::APPEND, id, self.epoch, object);
            append.revision = expected;
            append.aux = offset as u64;
            append.length = amount as u32;
            let (reply, _) = self.exchange(&append, &buffer[..amount])?;
            Self::status(&reply)?;
            offset += amount;
        }
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
        let id = self.fresh();
        let mut begin = Header::new(wire::BEGIN, id, self.epoch, object);
        begin.revision = expected;
        begin.aux = length as u64;
        begin.length = name.len() as u32;
        begin.name_length = name.len() as u32;
        let (reply, _) = self.exchange(&begin, name)?;
        Self::status(&reply)?;
        let mut offset = 0;
        let mut buffer = [0; DATA];
        while offset < partial.min(length) {
            let amount = core::cmp::min(DATA, partial.min(length) - offset);
            fill(offset, &mut buffer[..amount]);
            let mut append = Header::new(wire::APPEND, id, self.epoch, object);
            append.revision = expected;
            append.aux = offset as u64;
            append.length = amount as u32;
            let (reply, _) = self.exchange(&append, &buffer[..amount])?;
            Self::status(&reply)?;
            offset += amount;
        }
        Ok(id)
    }

    pub fn abort(&mut self, object: u64, expected: u64, id: u64) -> Result<(), Status> {
        let mut header = Header::new(wire::ABORT, id, self.epoch, object);
        header.revision = expected;
        let (reply, _) = self.exchange(&header, &[])?;
        Self::status(&reply)
    }

    /// Read the complete current revision in bounded pieces, delivering each
    /// piece to `sink(offset, bytes)`. Returns the number of bytes read.
    pub fn read(
        &mut self,
        object: u64,
        revision: u64,
        length: usize,
        mut sink: impl FnMut(usize, &[u8]),
    ) -> Result<usize, Status> {
        let mut offset = 0;
        while offset < length {
            let mut header = Header::new(wire::READ, self.fresh(), self.epoch, object);
            header.revision = revision;
            header.aux = offset as u64;
            header.count = DATA as u32;
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

    pub fn delete(&mut self, object: u64, expected: u64) -> Result<u64, (Status, u64)> {
        let mut header = Header::new(wire::DELETE, self.fresh(), self.epoch, object);
        header.revision = expected;
        let (reply, _) = self.exchange(&header, &[]).map_err(|status| (status, 0))?;
        Self::status(&reply).map_err(|status| (status, reply.revision))?;
        Ok(reply.revision)
    }

    pub fn rename(&mut self, object: u64, expected: u64, name: &[u8]) -> Result<u64, Status> {
        let mut header = Header::new(wire::RENAME, self.fresh(), self.epoch, object);
        header.revision = expected;
        header.length = name.len() as u32;
        header.name_length = name.len() as u32;
        let (reply, _) = self.exchange(&header, name)?;
        Self::status(&reply)?;
        Ok(reply.revision)
    }

    /// Complete paged listing of the caller's domain. Pages must come from one
    /// unchanged domain generation; a concurrent change returns `Stale`.
    pub fn list(
        &mut self,
        domain: usize,
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
            let (reply, body) = self.exchange(&header, &[])?;
            Self::status(&reply)?;
            if generation.is_some_and(|g| g != reply.revision) {
                return Err(Status::Stale);
            }
            generation = Some(reply.revision);
            for index in 0..reply.count as usize {
                let bytes =
                    &body[64 + index * wire::LIST_ENTRY..64 + (index + 1) * wire::LIST_ENTRY];
                out[count] = wire::decode_list_entry(bytes);
                count += 1;
            }
            cursor = reply.aux;
        }
        Ok((count, generation.unwrap_or(0)))
    }
}
