//! S09 domain backup archive: export a domain's current documents into a
//! documented byte stream, verify such a stream, and restore it into another
//! volume without ever overwriting existing objects.
//!
//! Format v1 (all integers little-endian; spec in
//! `docs/implementation/S09-BACKUP-ARCHIVE.md`):
//! header (64) · collection records (48 each) · per object a record (96)
//! followed by its content · trailer (16). Every fixed record carries its own
//! FNV-64; the trailer carries FNV-64 over every preceding byte.
//!
//! FNV-64 detects accidental corruption, truncation and torn transfers only.
//! It is not authentication or encryption: an archive is plaintext personal
//! data and must only go to a destination the user chose (B02/P01 open).
//! Only current versions are exported; local history is not a backup.
use crate::client::{Client, CollectionInfo, Transport};
use crate::wire::ListEntry;
use crate::{
    checksum_update, get16, get32, get64, put16, put32, put64, valid_name, Disk, Status,
    COLLECTIONS, DOMAINS, FNV_OFFSET, MAX_LENGTH, NAME, PER_DOMAIN, SECTOR,
};

pub const MAGIC: [u8; 8] = *b"NOSBAK\x00\x01";
pub const END: [u8; 8] = *b"NOSBAKZ\x00";
pub const VERSION: u16 = 1;
pub const HEADER: usize = 64;
pub const COLLECTION_RECORD: usize = 48;
pub const OBJECT_RECORD: usize = 96;
pub const TRAILER: usize = 16;

/// Destination of an export. An error (e.g. `Full`, `Io`) aborts the export;
/// the bytes already written are then not a valid archive.
pub trait Sink {
    fn put(&mut self, bytes: &[u8]) -> Result<(), Status>;
}

/// Origin of an archive. Returns the number of bytes copied into `buffer`
/// (0 = end of stream). Untrusted: a count above `buffer.len()` is rejected.
pub trait Source {
    fn get(&mut self, buffer: &mut [u8]) -> Result<usize, Status>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub generation: u64,
    pub objects: usize,
    pub collections: usize,
    pub content_bytes: u64,
}

/// One validated object record (content is checked separately).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub slot: usize,
    pub name_length: usize,
    pub name: [u8; NAME],
    pub collection: u8,
    pub kind: u16,
    pub tags: u32,
    pub length: usize,
    pub revision: u64,
    pub created: u64,
    pub modified: u64,
    pub content_sum: u64,
}

impl Entry {
    pub fn name(&self) -> &[u8] {
        &self.name[..self.name_length]
    }
}

/// Result of a restore. Bit `s` of a mask refers to archive slot `s`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub restored: u8,
    /// Destination slot occupied or name already used: left untouched.
    pub conflicts: u8,
    /// Restored without its collection: destination index holds another name.
    pub uncollected: u8,
    /// Content restored, but tags/kind/collection could not be set (META
    /// failed; the restore stopped with that error).
    pub bare: u8,
}

struct Out<'a, S: Sink> {
    sink: &'a mut S,
    sum: u64,
}

impl<S: Sink> Out<'_, S> {
    fn put(&mut self, bytes: &[u8]) -> Result<(), Status> {
        self.sink.put(bytes)?;
        self.sum = checksum_update(self.sum, bytes);
        Ok(())
    }
}

fn seal(record: &mut [u8]) {
    let at = record.len() - 8;
    let sum = checksum_update(FNV_OFFSET, &record[..at]);
    put64(record, at, sum);
}

fn sealed(record: &[u8]) -> bool {
    let at = record.len() - 8;
    checksum_update(FNV_OFFSET, &record[..at]) == get64(record, at)
}

fn zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| *byte == 0)
}

/// Export every current document and collection of `domain` (the client must
/// be bound to it). Fails with `Stale` if the domain changed meanwhile, and
/// with `Corrupt` if a read does not match the service's own content sum; in
/// both cases the partial output must be discarded.
pub fn export<T: Transport>(
    client: &mut Client<T>,
    domain: usize,
    sink: &mut impl Sink,
) -> Result<Summary, Status> {
    if domain >= DOMAINS {
        return Err(Status::Invalid);
    }
    let mut entries = [ListEntry::EMPTY; PER_DOMAIN];
    let (count, generation) = client.list(domain, &mut entries)?;
    let entries = &mut entries[..count];
    entries.sort_unstable_by_key(|entry| entry.object);
    let mut collections = [(0usize, CollectionInfo::default()); COLLECTIONS];
    let mut collection_count = 0;
    for index in 1..=COLLECTIONS {
        let info = client.collection_stat(domain, index)?;
        if info.present {
            collections[collection_count] = (index, info);
            collection_count += 1;
        }
    }
    let collections = &collections[..collection_count];
    let content_bytes: u64 = entries.iter().map(|entry| entry.length as u64).sum();

    let mut out = Out {
        sink,
        sum: FNV_OFFSET,
    };
    let mut header = [0; HEADER];
    header[..8].copy_from_slice(&MAGIC);
    put16(&mut header, 8, VERSION);
    header[12] = count as u8;
    header[13] = collection_count as u8;
    put64(&mut header, 16, generation);
    put64(&mut header, 24, content_bytes);
    seal(&mut header);
    out.put(&header)?;
    for (index, info) in collections {
        let mut record = [0; COLLECTION_RECORD];
        record[0] = *index as u8;
        record[1] = info.name_length as u8;
        record[8..8 + info.name_length].copy_from_slice(info.name());
        seal(&mut record);
        out.put(&record)?;
    }
    for entry in entries.iter() {
        let object = entry.object;
        let stat = client.stat(object).map_err(|(status, _)| status)?;
        let slot = (object as usize).wrapping_sub(domain * PER_DOMAIN);
        let name = &stat.name[..stat.name_length.min(NAME)];
        if stat.revision != entry.revision {
            return Err(Status::Stale);
        }
        if stat.length != entry.length {
            return Err(Status::Corrupt);
        }
        let collection_known = stat.collection == 0
            || collections
                .iter()
                .any(|(index, _)| *index == usize::from(stat.collection));
        if slot >= PER_DOMAIN || !valid_name(name) || stat.length > MAX_LENGTH || !collection_known
        {
            return Err(Status::Corrupt);
        }
        let mut record = [0; OBJECT_RECORD];
        record[0] = slot as u8;
        record[1] = name.len() as u8;
        record[2] = stat.collection;
        put16(&mut record, 4, stat.kind);
        put32(&mut record, 8, stat.tags);
        put32(&mut record, 12, stat.length as u32);
        put64(&mut record, 16, stat.revision);
        put64(&mut record, 24, stat.created);
        put64(&mut record, 32, stat.modified);
        put64(&mut record, 40, stat.content_sum);
        record[48..48 + name.len()].copy_from_slice(name);
        seal(&mut record);
        out.put(&record)?;
        let mut failed = None;
        let mut sum = FNV_OFFSET;
        client.read(object, stat.revision, stat.length, |_, chunk| {
            sum = checksum_update(sum, chunk);
            if failed.is_none() {
                failed = out.put(chunk).err();
            }
        })?;
        if let Some(status) = failed {
            return Err(status);
        }
        if sum != stat.content_sum {
            return Err(Status::Corrupt);
        }
    }
    // The snapshot is only consistent if nothing changed since the listing.
    let mut again = [ListEntry::EMPTY; PER_DOMAIN];
    if client.list(domain, &mut again)?.1 != generation {
        return Err(Status::Stale);
    }
    let mut trailer = [0; TRAILER];
    trailer[..8].copy_from_slice(&END);
    put64(&mut trailer, 8, out.sum);
    out.sink.put(&trailer)?;
    Ok(Summary {
        generation,
        objects: count,
        collections: collection_count,
        content_bytes,
    })
}

struct In<'a, S: Source> {
    source: &'a mut S,
    sum: u64,
}

impl<S: Source> In<'_, S> {
    fn exact(&mut self, buffer: &mut [u8]) -> Result<(), Status> {
        let mut filled = 0;
        while filled < buffer.len() {
            let got = self.source.get(&mut buffer[filled..])?;
            if got == 0 {
                return Err(Status::Corrupt);
            }
            if got > buffer.len() - filled {
                return Err(Status::Protocol);
            }
            filled += got;
        }
        self.sum = checksum_update(self.sum, buffer);
        Ok(())
    }
}

/// Collections declared by an archive: index `i + 1` lives at position `i`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Collections {
    pub present: [bool; COLLECTIONS],
    pub lengths: [usize; COLLECTIONS],
    pub names: [[u8; NAME]; COLLECTIONS],
}

impl Collections {
    pub fn name(&self, index: usize) -> &[u8] {
        &self.names[index - 1][..self.lengths[index - 1]]
    }
}

/// Parse and check a complete archive. `object` is called with each fully
/// checked object (record and content) in slot order; an error from it stops
/// the walk. The trailer and end of stream are checked after the last object,
/// so callers that act on objects must verify the archive first.
pub fn walk<S: Source>(
    source: &mut S,
    scratch: &mut [u8; MAX_LENGTH],
    mut object: impl FnMut(&Entry, &Collections, &[u8]) -> Result<(), Status>,
) -> Result<Summary, Status> {
    let mut input = In {
        source,
        sum: FNV_OFFSET,
    };
    let mut header = [0; HEADER];
    input.exact(&mut header)?;
    let count = usize::from(header[12]);
    let collection_count = usize::from(header[13]);
    let content_bytes = get64(&header, 24);
    if header[..8] != MAGIC
        || get16(&header, 8) != VERSION
        || !zero(&header[10..12])
        || !zero(&header[14..16])
        || !zero(&header[32..56])
        || count > PER_DOMAIN
        || collection_count > COLLECTIONS
        || content_bytes > (count * MAX_LENGTH) as u64
        || !sealed(&header)
    {
        return Err(Status::Corrupt);
    }
    let mut collections = Collections::default();
    let mut previous = 0;
    for _ in 0..collection_count {
        let mut record = [0; COLLECTION_RECORD];
        input.exact(&mut record)?;
        let index = usize::from(record[0]);
        let length = usize::from(record[1]);
        if !sealed(&record)
            || index <= previous
            || index > COLLECTIONS
            || length > NAME
            || !valid_name(&record[8..8 + length])
            || !zero(&record[2..8])
            || !zero(&record[8 + length..40])
        {
            return Err(Status::Corrupt);
        }
        previous = index;
        collections.present[index - 1] = true;
        collections.lengths[index - 1] = length;
        collections.names[index - 1][..length].copy_from_slice(&record[8..8 + length]);
    }
    let mut total = 0u64;
    let mut next_slot = 0;
    for _ in 0..count {
        let mut record = [0; OBJECT_RECORD];
        input.exact(&mut record)?;
        let mut name = [0; NAME];
        name.copy_from_slice(&record[48..80]);
        let entry = Entry {
            slot: usize::from(record[0]),
            name_length: usize::from(record[1]),
            name,
            collection: record[2],
            kind: get16(&record, 4),
            tags: get32(&record, 8),
            length: get32(&record, 12) as usize,
            revision: get64(&record, 16),
            created: get64(&record, 24),
            modified: get64(&record, 32),
            content_sum: get64(&record, 40),
        };
        let collection = usize::from(entry.collection);
        let valid = sealed(&record)
            && entry.slot >= next_slot
            && entry.slot < PER_DOMAIN
            && entry.name_length <= NAME
            && valid_name(entry.name())
            && zero(&entry.name[entry.name_length..])
            && collection <= COLLECTIONS
            && (collection == 0 || collections.present[collection - 1])
            && entry.length <= MAX_LENGTH
            && entry.revision != 0
            && record[3] == 0
            && zero(&record[6..8])
            && zero(&record[80..88]);
        if !valid {
            return Err(Status::Corrupt);
        }
        next_slot = entry.slot + 1;
        total += entry.length as u64;
        if total > content_bytes {
            return Err(Status::Corrupt);
        }
        let content = &mut scratch[..entry.length];
        input.exact(content)?;
        if checksum_update(FNV_OFFSET, content) != entry.content_sum {
            return Err(Status::Corrupt);
        }
        object(&entry, &collections, content)?;
    }
    let expected = input.sum;
    let mut trailer = [0; TRAILER];
    input.exact(&mut trailer)?;
    if total != content_bytes || trailer[..8] != END || get64(&trailer, 8) != expected {
        return Err(Status::Corrupt);
    }
    let mut extra = [0; 1];
    let got = input.source.get(&mut extra)?;
    if got > extra.len() {
        return Err(Status::Protocol);
    }
    if got != 0 {
        return Err(Status::Corrupt);
    }
    Ok(Summary {
        generation: get64(&header, 16),
        objects: count,
        collections: collection_count,
        content_bytes,
    })
}

/// Check a complete archive without side effects.
pub fn verify<S: Source>(
    source: &mut S,
    scratch: &mut [u8; MAX_LENGTH],
) -> Result<Summary, Status> {
    walk(source, scratch, |_, _, _| Ok(()))
}

/// Restore an archive into `domain` of the client's volume. Archive slot `s`
/// becomes object `domain * PER_DOMAIN + s`. Existing objects and names are
/// never overwritten (reported as conflicts); a destination collection index
/// holding a different name is never renamed. Each object is committed only
/// after its record and content checked out. On error the report so far is
/// returned with the status: earlier objects stay restored, so callers must
/// `verify` the same bytes first. The caller must be the only writer of the
/// destination domain meanwhile: a concurrent slot change aborts with
/// `Stale`, but a concurrently created duplicate name is not detected.
pub fn restore<S: Source, T: Transport>(
    source: &mut S,
    scratch: &mut [u8; MAX_LENGTH],
    client: &mut Client<T>,
    domain: usize,
) -> Result<Report, (Status, Report)> {
    let mut report = Report::default();
    if domain >= DOMAINS {
        return Err((Status::Invalid, report));
    }
    let mut existing = [ListEntry::EMPTY; PER_DOMAIN];
    let (existing_count, _) = client
        .list(domain, &mut existing)
        .map_err(|status| (status, report))?;
    let existing = &existing[..existing_count];
    // 0 = not yet resolved, 1 = usable, 2 = destination holds another name.
    let mut mapping = [0u8; COLLECTIONS];
    let walked = walk(source, scratch, |entry, collections, content| {
        let object = (domain * PER_DOMAIN + entry.slot) as u64;
        let bit = 1u8 << entry.slot;
        let name_taken = existing
            .iter()
            .any(|e| e.name[..e.name_length] == *entry.name());
        let expected = match client.stat(object) {
            Ok(_) => None,
            Err((Status::NotFound, tombstone)) => Some(tombstone),
            Err((status, _)) => return Err(status),
        };
        let Some(expected) = expected.filter(|_| !name_taken) else {
            report.conflicts |= bit;
            return Ok(());
        };
        let mut collection = usize::from(entry.collection);
        if collection != 0 {
            let wanted = collections.name(collection);
            let state = &mut mapping[collection - 1];
            if *state == 0 {
                let info = client.collection_stat(domain, collection)?;
                *state = if !info.present {
                    client
                        .collection(domain, collection, info.revision, wanted)
                        .map_err(|(status, _)| status)?;
                    1
                } else if info.name() == wanted {
                    1
                } else {
                    2
                };
            }
            if *state == 2 {
                collection = 0;
            }
        }
        let written = client.write(
            object,
            expected,
            entry.name(),
            content.len(),
            |offset, chunk| chunk.copy_from_slice(&content[offset..offset + chunk.len()]),
        );
        let revision = match written {
            Ok((revision, _)) => revision,
            // The commit may have landed: accept it only if it is exactly ours.
            Err(Status::Unknown) => match client.stat(object) {
                Ok(stat)
                    if Some(stat.revision) == expected.checked_add(1)
                        && stat.name[..stat.name_length] == *entry.name()
                        && stat.length == content.len()
                        && stat.content_sum == entry.content_sum =>
                {
                    stat.revision
                }
                _ => return Err(Status::Unknown),
            },
            Err(status) => return Err(status),
        };
        report.restored |= bit;
        if entry.collection != 0 && collection == 0 {
            report.uncollected |= bit;
        }
        if entry.tags != 0 || entry.kind != 0 || collection != 0 {
            if let Err((status, _)) =
                client.meta(object, revision, entry.tags, entry.kind, collection as u8)
            {
                report.bare |= bit;
                return Err(status);
            }
        }
        Ok(())
    });
    match walked {
        Ok(_) => Ok(report),
        Err(status) => Err((status, report)),
    }
}

/// Archive stream stored from sector 0 of a dedicated raw block range (the
/// S09 native portal target). The range carries no further framing: the
/// stored length follows from the header counts and `verify` stays the only
/// acceptance gate. Sectors after the archive keep their previous bytes.
pub struct DiskSink<'a, D: Disk> {
    disk: &'a mut D,
    sectors: u64,
    next: u64,
    buffer: [u8; SECTOR],
    filled: usize,
    written: u64,
}

impl<'a, D: Disk> DiskSink<'a, D> {
    pub fn new(disk: &'a mut D, sectors: u64) -> Self {
        Self {
            disk,
            sectors,
            next: 0,
            buffer: [0; SECTOR],
            filled: 0,
            written: 0,
        }
    }

    /// Write the zero-padded last sector and flush; returns the archive
    /// length. The archive is durable only after this returns `Ok`.
    pub fn finish(mut self) -> Result<u64, Status> {
        if self.filled != 0 {
            self.buffer[self.filled..].fill(0);
            self.write_sector()?;
        }
        self.disk.flush()?;
        Ok(self.written)
    }

    fn write_sector(&mut self) -> Result<(), Status> {
        if self.next >= self.sectors {
            return Err(Status::Full);
        }
        self.disk.write(self.next, &self.buffer)?;
        self.next += 1;
        self.filled = 0;
        Ok(())
    }
}

impl<D: Disk> Sink for DiskSink<'_, D> {
    fn put(&mut self, mut bytes: &[u8]) -> Result<(), Status> {
        while !bytes.is_empty() {
            let take = (SECTOR - self.filled).min(bytes.len());
            self.buffer[self.filled..self.filled + take].copy_from_slice(&bytes[..take]);
            self.filled += take;
            self.written += take as u64;
            bytes = &bytes[take..];
            if self.filled == SECTOR {
                self.write_sector()?;
            }
        }
        Ok(())
    }
}

/// Reads an archive written by `DiskSink`. The range is untrusted: a header
/// whose counts imply more bytes than the range holds is `Corrupt`; any other
/// deviation is left to `walk`.
pub struct DiskSource<'a, D: Disk> {
    disk: &'a mut D,
    sectors: u64,
    length: Option<u64>,
    position: u64,
    buffer: [u8; SECTOR],
    loaded: Option<u64>,
}

impl<'a, D: Disk> DiskSource<'a, D> {
    pub fn new(disk: &'a mut D, sectors: u64) -> Self {
        Self {
            disk,
            sectors,
            length: None,
            position: 0,
            buffer: [0; SECTOR],
            loaded: None,
        }
    }

    fn load(&mut self, sector: u64) -> Result<(), Status> {
        if self.loaded != Some(sector) {
            self.loaded = None;
            self.disk.read(sector, &mut self.buffer)?;
            self.loaded = Some(sector);
        }
        Ok(())
    }
}

impl<D: Disk> Source for DiskSource<'_, D> {
    fn get(&mut self, out: &mut [u8]) -> Result<usize, Status> {
        let length = match self.length {
            Some(length) => length,
            None => {
                if self.sectors == 0 {
                    return Err(Status::Corrupt);
                }
                self.load(0)?;
                let header = &self.buffer;
                let records = u64::from(header[13]) * COLLECTION_RECORD as u64
                    + u64::from(header[12]) * OBJECT_RECORD as u64;
                let length = ((HEADER + TRAILER) as u64 + records)
                    .checked_add(get64(header, 24))
                    .filter(|length| *length <= self.sectors.saturating_mul(SECTOR as u64))
                    .ok_or(Status::Corrupt)?;
                self.length = Some(length);
                length
            }
        };
        let take = (length - self.position).min(out.len() as u64) as usize;
        if take == 0 {
            return Ok(0);
        }
        self.load(self.position / SECTOR as u64)?;
        let at = (self.position % SECTOR as u64) as usize;
        let take = take.min(SECTOR - at);
        out[..take].copy_from_slice(&self.buffer[at..at + take]);
        self.position += take as u64;
        Ok(take)
    }
}
