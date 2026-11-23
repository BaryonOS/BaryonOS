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
    checksum_update, chunks_for, get16, get32, get64, put16, put32, put64, valid_name, Disk,
    Status, CHUNK, CHUNKS, COLLECTIONS, DOMAINS, FNV_OFFSET, HISTORY, MAX_LENGTH, NAME, PER_DOMAIN,
    QUOTA, SECTOR,
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

// --- Format v2 (`docs/implementation/S09-ARCHIVE-V2.md`): writer over the
// own-domain reads, reader and L1 history replay. v1 above is unchanged.

pub const VERSION_V2: u16 = 2;
pub const HEADER_V2: usize = 64;
pub const COLLECTION_RECORD_V2: usize = 64;
pub const OBJECT_RECORD_V2: usize = 128;
pub const VERSION_RECORD_V2: usize = 64;
/// Version records per archive: every slot with its current version and history.
pub const VERSIONS_V2: usize = PER_DOMAIN * (HISTORY + 1);
pub const TOMBSTONES: u8 = 0x01;
pub const ACCOUNTED: u8 = 0x02;
/// Every version's chunks lie within its object's, and a domain holds at most `QUOTA` chunks.
pub const CONTENT_V2: u64 = ((HISTORY + 1) * QUOTA * CHUNK) as u64;
pub const MAX_ARCHIVE_V2: u64 = (HEADER_V2
    + COLLECTIONS * COLLECTION_RECORD_V2
    + PER_DOMAIN * OBJECT_RECORD_V2
    + VERSIONS_V2 * VERSION_RECORD_V2
    + TRAILER) as u64
    + CONTENT_V2;

/// One validated version record (its content is checked against `content_sum`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VersionV2 {
    pub revision: u64,
    pub length: usize,
    pub content_sum: u64,
    pub name_length: usize,
    pub name: [u8; NAME],
}

impl VersionV2 {
    pub fn name(&self) -> &[u8] {
        &self.name[..self.name_length]
    }
}

/// One validated object record with its versions, current first (present
/// objects), then history newest first. Revisions and stamps are recorded
/// source values only.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObjectV2 {
    pub slot: usize,
    pub present: bool,
    pub collection: u8,
    pub kind: u16,
    pub tags: u32,
    pub revision: u64,
    pub created: u64,
    pub modified: u64,
    pub version_count: usize,
    pub versions: [VersionV2; HISTORY + 1],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CollectionV2 {
    pub index: usize,
    pub present: bool,
    pub revision: u64,
    pub name_length: usize,
    pub name: [u8; NAME],
}

impl CollectionV2 {
    pub fn name(&self) -> &[u8] {
        &self.name[..self.name_length]
    }
}

/// Everything a verified v2 archive records except content, in archive order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArchiveV2 {
    pub tombstones: bool,
    /// With `accounted` clear, `generation` is an unchecked recorded value
    /// (S9 not performed) and must be reported as such.
    pub accounted: bool,
    pub generation: u64,
    /// The exporting domain's own clock: a validation bound, never installed.
    pub clock: u64,
    pub content_bytes: u64,
    pub collection_count: usize,
    pub collections: [CollectionV2; COLLECTIONS],
    pub object_count: usize,
    pub objects: [ObjectV2; PER_DOMAIN],
}

impl ArchiveV2 {
    pub fn object(&self, slot: usize) -> Option<&ObjectV2> {
        self.objects[..self.object_count]
            .iter()
            .find(|object| object.slot == slot)
    }
}

/// Parse and check a complete v2 archive (every rule of section 4, S1–S10).
/// `version(object, i, content)` is called for version `i` of `object` once
/// its record and content checked out (`object.versions[..=i]` filled). The
/// per-object chunk rule, the trailer and S9 are checked later, so callers
/// that act on versions must verify the archive first.
pub fn walk_v2<S: Source>(
    source: &mut S,
    scratch: &mut [u8; MAX_LENGTH],
    mut version: impl FnMut(&ObjectV2, usize, &[u8]) -> Result<(), Status>,
) -> Result<ArchiveV2, Status> {
    let mut input = In {
        source,
        sum: FNV_OFFSET,
    };
    let mut header = [0; HEADER_V2];
    input.exact(&mut header)?;
    let flags = header[10];
    let count = usize::from(header[12]);
    let collection_count = usize::from(header[13]);
    let version_count = usize::from(get16(&header, 14));
    let content_bytes = get64(&header, 24);
    let clock = get64(&header, 32);
    if header[..8] != MAGIC
        || get16(&header, 8) != VERSION_V2
        || flags & !(TOMBSTONES | ACCOUNTED) != 0
        || header[11] != 0
        || !zero(&header[40..56])
        || count > PER_DOMAIN
        || collection_count > COLLECTIONS
        || version_count > VERSIONS_V2
        || content_bytes > (version_count * MAX_LENGTH) as u64
        || content_bytes > CONTENT_V2
        || !sealed(&header)
    {
        return Err(Status::Corrupt);
    }
    let mut archive = ArchiveV2 {
        tombstones: flags & TOMBSTONES != 0,
        accounted: flags & ACCOUNTED != 0,
        generation: get64(&header, 16),
        clock,
        content_bytes,
        ..ArchiveV2::default()
    };
    let mut present_collections = [false; COLLECTIONS];
    let mut previous = 0;
    let mut revisions = 0u64;
    for position in 0..collection_count {
        let mut record = [0; COLLECTION_RECORD_V2];
        input.exact(&mut record)?;
        let index = usize::from(record[0]);
        let present = record[1];
        let length = usize::from(record[2]);
        let revision = get64(&record, 8);
        let valid = sealed(&record)
            && index > previous
            && index <= COLLECTIONS
            && present <= 1
            && length <= NAME
            && zero(&record[3..8])
            && zero(&record[16 + length..56])
            && revision != 0
            && if present == 1 {
                valid_name(&record[16..16 + length])
            } else {
                length == 0
            };
        // Deleted collections exist only for the S9 check (R2).
        if !valid || (present == 0 && !archive.accounted) || revision > clock {
            return Err(Status::Corrupt);
        }
        previous = index;
        present_collections[index - 1] = present == 1;
        revisions = revisions.wrapping_add(revision);
        let mut name = [0; NAME];
        name[..length].copy_from_slice(&record[16..16 + length]);
        archive.collections[position] = CollectionV2 {
            index,
            present: present == 1,
            revision,
            name_length: length,
            name,
        };
    }
    archive.collection_count = collection_count;
    let mut next_slot = 0;
    let mut total = 0u64;
    let mut versions_seen = 0usize;
    let mut chunks = 0usize;
    for position in 0..count {
        let mut record = [0; OBJECT_RECORD_V2];
        input.exact(&mut record)?;
        let present = record[1];
        let collection = usize::from(record[2]);
        let history_count = usize::from(record[3]);
        let mut object = ObjectV2 {
            slot: usize::from(record[0]),
            present: present == 1,
            collection: record[2],
            kind: get16(&record, 4),
            tags: get32(&record, 8),
            revision: get64(&record, 16),
            created: get64(&record, 24),
            modified: get64(&record, 32),
            ..ObjectV2::default()
        };
        let valid = sealed(&record)
            && object.slot >= next_slot
            && object.slot < PER_DOMAIN
            && present <= 1
            && history_count <= HISTORY
            && zero(&record[6..8])
            && zero(&record[12..16])
            && zero(&record[40..120])
            && object.revision != 0
            && (collection == 0
                || (collection <= COLLECTIONS && present_collections[collection - 1]))
            && (object.present || (collection == 0 && object.kind == 0 && object.tags == 0));
        let tombstone = !object.present && history_count != 0;
        let content_free = !object.present && history_count == 0;
        // Flag gating (section 3), then S4, S5, S6 and S7.
        if !valid
            || (tombstone && !archive.tombstones)
            || (content_free && !archive.accounted)
            || object.created == 0
            || object.created > object.modified
            || object.modified > clock
            || object.revision > clock
        {
            return Err(Status::Corrupt);
        }
        next_slot = object.slot + 1;
        object.version_count = usize::from(present) + history_count;
        let mut largest = 0;
        for index in 0..object.version_count {
            let mut record = [0; VERSION_RECORD_V2];
            input.exact(&mut record)?;
            let name_length = usize::from(record[0]);
            let length = get32(&record, 4) as usize;
            let revision = get64(&record, 8);
            let valid = sealed(&record)
                && name_length <= NAME
                && valid_name(&record[24..24 + name_length])
                && zero(&record[1..4])
                && zero(&record[24 + name_length..56])
                && length <= MAX_LENGTH;
            // S2 bounds the newest version by the object revision; S3 makes
            // the chain strictly decreasing (S1 keeps the subtraction in range).
            let bound = if index == 0 {
                object.revision
            } else {
                object.versions[index - 1].revision - 1
            };
            versions_seen += 1;
            total = total.checked_add(length as u64).ok_or(Status::Corrupt)?;
            if !valid
                || revision == 0
                || revision > bound
                || versions_seen > version_count
                || total > content_bytes
            {
                return Err(Status::Corrupt);
            }
            let mut name = [0; NAME];
            name[..name_length].copy_from_slice(&record[24..24 + name_length]);
            object.versions[index] = VersionV2 {
                revision,
                length,
                content_sum: get64(&record, 16),
                name_length,
                name,
            };
            let content = &mut scratch[..length];
            input.exact(content)?;
            if checksum_update(FNV_OFFSET, content) != object.versions[index].content_sum {
                return Err(Status::Corrupt);
            }
            version(&object, index, content)?;
            largest = largest.max(chunks_for(length));
        }
        // S10: an object's unique chunks are at least its largest version's.
        chunks += largest;
        if chunks > QUOTA {
            return Err(Status::Corrupt);
        }
        revisions = revisions.wrapping_add(object.revision);
        archive.objects[position] = object;
    }
    archive.object_count = count;
    let expected = input.sum;
    let mut trailer = [0; TRAILER];
    input.exact(&mut trailer)?;
    if total != content_bytes
        || versions_seen != version_count
        || trailer[..8] != END
        || get64(&trailer, 8) != expected
    {
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
    // S9: a wrapping sum like the table's; detects accidental omission only.
    if archive.accounted && revisions != archive.generation {
        return Err(Status::Corrupt);
    }
    Ok(archive)
}

/// Check a complete v2 archive without side effects.
pub fn verify_v2<S: Source>(
    source: &mut S,
    scratch: &mut [u8; MAX_LENGTH],
) -> Result<ArchiveV2, Status> {
    walk_v2(source, scratch, |_, _, _| Ok(()))
}

/// Chunks free for a never-used domain whatever the other domains hold:
/// BEGIN keeps every domain's committed plus staged chunks within `QUOTA`.
pub const GUARANTEED_FREE: usize = CHUNKS - (DOMAINS - 1) * QUOTA;
const _: () = assert!(GUARANTEED_FREE > 0);

/// Result of [`restore_native`]. A lost reply is resolved by reading state
/// back and counted only when the request demonstrably committed; counts are
/// a lower bound when the status is still `Unknown` or is `Protocol` (a
/// malformed reply): that request may have committed without a valid reply.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NativeReport {
    /// Bit `s`: archive slot `s` fully replayed (its replayable versions,
    /// see [`replayable`], and its metadata). Set once the last needed
    /// commit is confirmed: after the WRITE of its current version for a
    /// slot without metadata, after its META otherwise, so a later failure,
    /// also during the WRITE phase, keeps completed slots.
    pub restored: u8,
    /// Committed WRITEs this restore confirmed (a lower bound after
    /// `Unknown` or `Protocol`, see above).
    pub writes: usize,
    /// Every commit this restore made (WRITE, COLLECTION, META). Zero after
    /// an error other than `Unknown` or `Protocol` means this restore
    /// committed nothing: the domain's committed state is as it was, never
    /// used only if it was before (not after `Stale`, which reports an
    /// already used domain). A failed BEGIN whose reply was lost, or an
    /// upload interrupted before its COMMIT, may still leave a staged upload
    /// that reserves chunks until it is replaced; [`never_used`] observes
    /// committed revisions only, not staged uploads. Non-zero
    /// means this restore made the domain used and a retry is refused. After
    /// an unresolved `Unknown`, or a malformed reply (`Protocol`), this
    /// restore may have committed even at zero: [`never_used`] decides on
    /// retry.
    pub committed: usize,
    /// Versions of present objects not replayed because they follow a
    /// revision gap and may predate a DELETE (owner decision, 19 November
    /// 2026). Content-free records are excluded separately and not counted.
    /// Set as soon as the archive is verified, so a later refusal (`Invalid`,
    /// `Stale`, `Quota`, `Full`) reports it too.
    pub skipped: usize,
}

/// Number of `object`'s versions (current first, then history newest first)
/// that L1 replays: the longest prefix whose consecutive revisions differ by
/// exactly 1. A committed version's revision equals the entry revision after
/// that commit; DELETE, RENAME, META and TRANSACT raise the entry revision
/// without a version, while RESTORE creates a current version at the next
/// revision. A step of 1 therefore excludes an intervening DELETE (or any
/// other revision-only operation); it does not prove that no operation
/// occurred, since the newer version may come from a RESTORE. The archive
/// cannot tell a DELETE gap from a META or RENAME gap; both stop the prefix
/// (conservative). Absent objects replay nothing.
pub fn replayable(object: &ObjectV2) -> usize {
    if !object.present {
        return 0;
    }
    let versions = &object.versions[..object.version_count];
    1 + versions
        .windows(2)
        .take_while(|pair| pair[1].revision.checked_add(1) == Some(pair[0].revision))
        .count()
}

/// Is `domain` demonstrably never used (R3)? Every publish path raises at
/// least one object or collection revision of the domain and revisions never
/// decrease, so all revisions zero means nothing was ever committed there.
/// STAT reports the revision of absent objects too: `(NotFound, 0)` only.
pub fn never_used<T: Transport>(client: &mut Client<T>, domain: usize) -> Result<bool, Status> {
    for slot in 0..PER_DOMAIN {
        match client.stat((domain * PER_DOMAIN + slot) as u64) {
            Err((Status::NotFound, 0)) => {}
            Ok(_) | Err((Status::NotFound, _)) => return Ok(false),
            Err((status, _)) => return Err(status),
        }
    }
    for index in 1..=COLLECTIONS {
        if client.collection_stat(domain, index)?.revision != 0 {
            return Ok(false);
        }
    }
    Ok(true)
}

/// L1 native restore: replay a v2 archive's history into a never-used
/// `domain` through ordinary WRITE/META/COLLECTION requests (spec section 7).
///
/// `open` must return a fresh reader of the same archive on each call: the
/// archive is verified completely in a first pass, then read once per history
/// depth so every object is written oldest version first. In every replay
/// pass each object record is compared with the verified plan before its
/// WRITE; the rest of the pass (header, collections, trailer, trailing bytes)
/// is compared only after that pass's WRITEs, so such a mismatch returns
/// `Corrupt` with `committed > 0` and leaves the domain used.
/// Refused before the first write: a corrupt or v1 archive (`Corrupt`), an
/// archive with `TOMBSTONES` set (`Invalid`, R2), a domain that was ever used
/// (`Stale`, R3) and a replay that exceeds the domain quota (`Quota`; L1
/// allocates fresh chunks for every WRITE). Content-free records and deleted
/// collections are never replayed, nor are versions after the first
/// revision gap of an object ([`replayable`], counted in `skipped`); the
/// capacity checks count replayed versions only. The volume-wide free chunk count is not
/// observable through the protocol, so a replay needing more than
/// [`GUARANTEED_FREE`] chunks is refused with `Full` before the first write.
/// Every WRITE comes first, then the collections, then each object's META
/// from the verified plan, so the first mutation is a BEGIN whenever a
/// version is replayed; a token-bound upload staged in the domain then makes
/// it `Busy` before anything commits (a legacy, token-0 staged upload is
/// instead replaced by that BEGIN, as for any legacy BEGIN). A restore with no replayable version
/// but a present collection starts with COLLECTION, which a staged upload
/// does not refuse: it succeeds, the domain becomes used, and the staged
/// upload can still COMMIT afterwards (its object revision is unchanged).
/// No read operation exposes staged uploads, so this is not detected. A lost
/// reply (`Unknown`) is resolved by STAT or COLLECTION_STAT and accepted only
/// when the state is exactly what was sent. A failure with
/// `report.committed == 0` and a status other than `Unknown` or `Protocol`
/// means this restore committed nothing (after `Stale` the domain was
/// already used), although a failed or interrupted upload may leave a staged
/// upload that reserves chunks until it is replaced; an unresolved `Unknown` or a malformed reply (`Protocol`) may have
/// committed, so the domain must then be treated as used ([`never_used`]
/// decides on retry). Any other later failure leaves it used, and a retry is
/// then refused. The caller must be the only writer of the destination
/// domain meanwhile: a concurrent write to a slot the archive does not use,
/// after the never-used check, is not detected.
///
/// `source` receives the verified archive before anything is written: the
/// recorded source revisions, stamps, clock and generation are reported only
/// there; the destination assigns its own revisions and clock.
pub fn restore_native<S: Source, T: Transport>(
    mut open: impl FnMut() -> S,
    scratch: &mut [u8; MAX_LENGTH],
    client: &mut Client<T>,
    domain: usize,
    source: &mut ArchiveV2,
) -> Result<NativeReport, (Status, NativeReport)> {
    let mut report = NativeReport::default();
    if domain >= DOMAINS {
        return Err((Status::Invalid, report));
    }
    let plan = verify_v2(&mut open(), scratch).map_err(|status| (status, report))?;
    *source = plan;
    report.skipped = plan.objects[..plan.object_count]
        .iter()
        .filter(|object| object.present)
        .map(|object| object.version_count - replayable(object))
        .sum();
    if plan.tombstones {
        return Err((Status::Invalid, report));
    }
    match never_used(client, domain) {
        Ok(true) => {}
        Ok(false) => return Err((Status::Stale, report)),
        Err(status) => return Err((status, report)),
    }
    // A never-used domain holds no chunks, and at most HISTORY + 1 versions
    // per object means every replayed version stays referenced.
    let needed: usize = plan.objects[..plan.object_count]
        .iter()
        .flat_map(|object| object.versions[..replayable(object)].iter())
        .map(|version| chunks_for(version.length))
        .sum();
    if needed > QUOTA {
        return Err((Status::Quota, report));
    }
    // The volume-wide free count is not observable. shortcut: refuses
    // replays of 25..=40 chunks that might fit; lift once a reviewed
    // free-count read exists.
    if needed > GUARANTEED_FREE {
        return Err((Status::Full, report));
    }
    let mut revisions = [0u64; PER_DOMAIN];
    let needs_meta = |o: &&ObjectV2| o.tags != 0 || o.kind != 0 || o.collection != 0;
    for depth in (0..=HISTORY).rev() {
        let mut writes = report.writes;
        let mut restored = report.restored;
        let replayed = walk_v2(&mut open(), scratch, |object, index, content| {
            let Some(planned) = plan.object(object.slot) else {
                return Err(Status::Corrupt);
            };
            // The source is read again: act only on what the first pass verified.
            let same = ObjectV2 {
                versions: object.versions,
                ..*planned
            } == *object
                && planned.versions[..=index] == object.versions[..=index];
            if !same {
                return Err(Status::Corrupt);
            }
            if index != depth || index >= replayable(planned) {
                return Ok(());
            }
            let version = &object.versions[index];
            let target = (domain * PER_DOMAIN + object.slot) as u64;
            let expected = revisions[object.slot];
            let written = client.write(
                target,
                expected,
                version.name(),
                content.len(),
                |offset, chunk| chunk.copy_from_slice(&content[offset..offset + chunk.len()]),
            );
            let revision = match written {
                Ok((revision, _)) => revision,
                // The commit may have landed: accept it only if it is exactly ours.
                Err(Status::Unknown) => match client.stat(target) {
                    Ok(stat)
                        if Some(stat.revision) == expected.checked_add(1)
                            && stat.name[..stat.name_length] == *version.name()
                            && stat.length == content.len()
                            && stat.content_sum == version.content_sum =>
                    {
                        stat.revision
                    }
                    _ => return Err(Status::Unknown),
                },
                Err(status) => return Err(status),
            };
            writes += 1;
            revisions[object.slot] = revision;
            // Index 0, the current version, is a slot's last WRITE: without
            // metadata the slot is complete, even if a later request fails.
            if index == 0 && !needs_meta(&planned) {
                restored |= 1 << object.slot;
            }
            Ok(())
        });
        report.writes = writes;
        report.restored = restored;
        report.committed = writes;
        match replayed {
            Ok(again) if again == plan => {}
            Ok(_) => return Err((Status::Corrupt, report)),
            Err(status) => return Err((status, report)),
        }
    }
    // Collections before META: META with a collection index needs it to exist.
    for collection in &plan.collections[..plan.collection_count] {
        if collection.present {
            match client.collection(domain, collection.index, 0, collection.name()) {
                Ok(_) => {}
                // Created from revision 0: exactly ours means revision 1.
                Err((Status::Unknown, _)) => match client.collection_stat(domain, collection.index)
                {
                    Ok(info)
                        if info.present
                            && info.revision == 1
                            && info.name() == collection.name() => {}
                    _ => return Err((Status::Unknown, report)),
                },
                Err((status, _)) => return Err((status, report)),
            }
            report.committed += 1;
        }
    }
    for object in plan.objects[..plan.object_count]
        .iter()
        .filter(|o| o.present && needs_meta(o))
    {
        let target = (domain * PER_DOMAIN + object.slot) as u64;
        let expected = revisions[object.slot];
        let meta = (object.tags, object.kind, object.collection);
        match client.meta(target, expected, meta.0, meta.1, meta.2) {
            Ok(_) => {}
            Err((Status::Unknown, _)) => match client.stat(target) {
                Ok(stat)
                    if Some(stat.revision) == expected.checked_add(1)
                        && (stat.tags, stat.kind, stat.collection) == meta => {}
                _ => return Err((Status::Unknown, report)),
            },
            Err((status, _)) => return Err((status, report)),
        }
        report.committed += 1;
        report.restored |= 1 << object.slot;
    }
    Ok(report)
}

/// Result of [`export_v2`]: what the archive records (no content), and the
/// number of retained history versions left out because they follow a
/// revision gap and may predate a DELETE (R2, owner decision).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExportV2 {
    pub archive: ArchiveV2,
    pub omitted: usize,
}

fn put_version_v2<S: Sink>(out: &mut Out<'_, S>, version: &VersionV2) -> Result<(), Status> {
    let mut record = [0; VERSION_RECORD_V2];
    record[0] = version.name_length as u8;
    put32(&mut record, 4, version.length as u32);
    put64(&mut record, 8, version.revision);
    put64(&mut record, 16, version.content_sum);
    record[24..24 + version.name_length].copy_from_slice(version.name());
    seal(&mut record);
    out.put(&record)
}

/// Export `domain` (the client must be bound to it) as a v2 archive.
///
/// Policy (S09-ARCHIVE-V2 section 3, S09-OWN-DOMAIN-READS section 3):
/// `TOMBSTONES` is never set (R2); `ACCOUNTED` is always set, and the only
/// content-free records are those S9 needs: one per absent slot with a
/// non-zero revision (deleted, tombstoned or purged; stamps from
/// VERSION_STAT, no content, no name) and every deleted collection. A present
/// object carries only its consecutive-revision prefix of versions, the same
/// prefix L1 replays; later versions are counted in `omitted`. Every recorded
/// value is a source value read from the service; nothing is synthesised and
/// no source identifier exists. DOMAIN_STAT is the first request, so an older
/// service (`NoSys`) gets nothing written. The final DOMAIN_STAT must equal
/// the first, otherwise `Stale` and no trailer: on any error the caller must
/// discard the output (and may retry).
pub fn export_v2<T: Transport>(
    client: &mut Client<T>,
    domain: usize,
    sink: &mut impl Sink,
) -> Result<ExportV2, Status> {
    if domain >= DOMAINS {
        return Err(Status::Invalid);
    }
    let first = client.domain_stat(domain)?;
    let (generation, clock) = first;
    let mut archive = ArchiveV2 {
        accounted: true,
        generation,
        clock,
        ..ArchiveV2::default()
    };
    let mut omitted = 0;
    for index in 1..=COLLECTIONS {
        let info = client.collection_stat(domain, index)?;
        if info.revision == 0 {
            continue;
        }
        // A bound violation under a fixed clock means a concurrent change.
        if info.revision > clock {
            return Err(Status::Stale);
        }
        archive.collections[archive.collection_count] = CollectionV2 {
            index,
            present: info.present,
            revision: info.revision,
            name_length: info.name_length,
            name: info.name,
        };
        archive.collection_count += 1;
    }
    for slot in 0..PER_DOMAIN {
        let object = (domain * PER_DOMAIN + slot) as u64;
        let mut o = ObjectV2 {
            slot,
            ..ObjectV2::default()
        };
        match client.stat(object) {
            Ok(stat) => {
                let collected = stat.collection == 0
                    || archive.collections[..archive.collection_count]
                        .iter()
                        .any(|c| c.present && c.index == usize::from(stat.collection));
                if !collected {
                    return Err(Status::Stale);
                }
                o.present = true;
                o.collection = stat.collection;
                o.kind = stat.kind;
                o.tags = stat.tags;
                o.revision = stat.revision;
                o.created = stat.created;
                o.modified = stat.modified;
                o.versions[0] = VersionV2 {
                    revision: stat.version_revision,
                    length: stat.length,
                    content_sum: stat.content_sum,
                    name_length: stat.name_length,
                    name: stat.name,
                };
                for (version, info) in o.versions[1..].iter_mut().zip(&stat.history) {
                    version.revision = info.revision;
                    version.length = info.length;
                    version.content_sum = info.content_sum;
                }
                o.version_count = 1 + stat.history_count;
                let keep = replayable(&o);
                omitted += o.version_count - keep;
                o.version_count = keep;
                o.versions[keep..].fill(VersionV2::default());
                for (selector, version) in o.versions[..keep].iter_mut().enumerate().skip(1) {
                    let named = client
                        .version_stat(object, stat.revision, selector)
                        .map_err(|(status, _)| status)?;
                    if named.version_revision != version.revision {
                        return Err(Status::Corrupt);
                    }
                    version.name_length = named.name_length;
                    version.name = named.name;
                }
            }
            Err((Status::NotFound, 0)) => continue,
            Err((Status::NotFound, revision)) => {
                let stamps = client
                    .version_stat(object, revision, 0)
                    .map_err(|(status, _)| status)?;
                if stamps.present {
                    return Err(Status::Corrupt);
                }
                o.revision = revision;
                o.created = stamps.created;
                o.modified = stamps.modified;
            }
            Err((status, _)) => return Err(status),
        }
        if o.revision > clock || o.modified > clock {
            return Err(Status::Stale);
        }
        archive.objects[archive.object_count] = o;
        archive.object_count += 1;
    }
    let objects = &archive.objects[..archive.object_count];
    let version_count: usize = objects.iter().map(|o| o.version_count).sum();
    archive.content_bytes = objects
        .iter()
        .flat_map(|o| &o.versions[..o.version_count])
        .map(|v| v.length as u64)
        .sum();

    let mut out = Out {
        sink,
        sum: FNV_OFFSET,
    };
    let mut header = [0; HEADER_V2];
    header[..8].copy_from_slice(&MAGIC);
    put16(&mut header, 8, VERSION_V2);
    header[10] = ACCOUNTED;
    header[12] = archive.object_count as u8;
    header[13] = archive.collection_count as u8;
    put16(&mut header, 14, version_count as u16);
    put64(&mut header, 16, generation);
    put64(&mut header, 24, archive.content_bytes);
    put64(&mut header, 32, clock);
    seal(&mut header);
    out.put(&header)?;
    for c in &archive.collections[..archive.collection_count] {
        let mut record = [0; COLLECTION_RECORD_V2];
        record[0] = c.index as u8;
        record[1] = u8::from(c.present);
        record[2] = c.name_length as u8;
        put64(&mut record, 8, c.revision);
        record[16..16 + c.name_length].copy_from_slice(c.name());
        seal(&mut record);
        out.put(&record)?;
    }
    for o in &archive.objects[..archive.object_count] {
        let mut record = [0; OBJECT_RECORD_V2];
        record[0] = o.slot as u8;
        record[1] = u8::from(o.present);
        record[2] = o.collection;
        record[3] = (o.version_count - usize::from(o.present)) as u8;
        put16(&mut record, 4, o.kind);
        put32(&mut record, 8, o.tags);
        put64(&mut record, 16, o.revision);
        put64(&mut record, 24, o.created);
        put64(&mut record, 32, o.modified);
        seal(&mut record);
        out.put(&record)?;
        let object = (domain * PER_DOMAIN + o.slot) as u64;
        for (selector, version) in o.versions[..o.version_count].iter().enumerate() {
            put_version_v2(&mut out, version)?;
            let mut failed = None;
            let mut sum = FNV_OFFSET;
            let read =
                client.read_version(object, o.revision, selector, version.length, |_, chunk| {
                    sum = checksum_update(sum, chunk);
                    if failed.is_none() {
                        failed = out.put(chunk).err();
                    }
                })?;
            if let Some(status) = failed {
                return Err(status);
            }
            if read != version.length || sum != version.content_sum {
                return Err(Status::Corrupt);
            }
        }
    }
    // Snapshot rule: an unchanged own clock proves no own commit happened.
    if client.domain_stat(domain)? != first {
        return Err(Status::Stale);
    }
    let mut trailer = [0; TRAILER];
    trailer[..8].copy_from_slice(&END);
    put64(&mut trailer, 8, out.sum);
    out.sink.put(&trailer)?;
    Ok(ExportV2 { archive, omitted })
}
