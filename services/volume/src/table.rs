//! Volume table v2 (S06) and fail-closed recovery.
//!
//! Generation `g` is always written to slot `g % 2`. Every table sector ends
//! with the table generation, an attempt digest (FNV-64 over the payload of
//! all sectors of that copy) and an FNV-64 over slot, sector index and the
//! first 504 bytes. A torn write is visible per sector, and sectors of two
//! different write attempts for the same generation never form one copy. Recovery may
//! only pick the newest completely valid copy when the other copy is either
//! the previous generation or an interrupted attempt at the next generation.
//!
//! v2 adds per-domain collections, object metadata (kind, tags, logical
//! created/modified stamps from a per-domain clock) and a two-deep version
//! history per object that makes earlier content, and deleted objects,
//! restorable.
use crate::{
    checksum_update, chunks_for, domain_of, get16, get32, get64, put16, put32, put64, valid_name,
    Disk, Status, CHUNKS, COLLECTIONS, DATA_START, DOMAINS, FNV_OFFSET, HISTORY, MAX_CHUNKS,
    MAX_LENGTH, NAME, NO_CHUNK, OBJECTS, PER_DOMAIN, QUOTA, SECTOR, TABLE_SECTORS, VOLUME_SECTORS,
};

pub const MAGIC: &[u8; 8] = b"NOSVOL02";
const FORMAT_MARKER_MAGIC: &[u8; 8] = b"NOSFMT02";
const FORMAT_MARKER_VERSION: u16 = 1;
pub const VERSION: u16 = 2;
pub const VERSION_BYTES: usize = 216;
pub const COLLECTION_BYTES: usize = 56;
pub const TABLE_SECTORS_USIZE: usize = TABLE_SECTORS as usize;
const TRAILER: usize = 488;
const HEAD: usize = 64;

/// One immutable content + name snapshot of an object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Version {
    /// Object revision at which this snapshot was created (write/restore).
    pub revision: u64,
    pub length: usize,
    pub chunk_count: usize,
    pub name_length: usize,
    pub name: [u8; NAME],
    pub content_sum: u64,
    pub chunks: [u16; MAX_CHUNKS],
    pub sums: [u64; MAX_CHUNKS],
}

impl Version {
    pub const EMPTY: Self = Self {
        revision: 0,
        length: 0,
        chunk_count: 0,
        name_length: 0,
        name: [0; NAME],
        content_sum: 0,
        chunks: [NO_CHUNK; MAX_CHUNKS],
        sums: [0; MAX_CHUNKS],
    };

    pub fn name(&self) -> &[u8] {
        &self.name[..self.name_length]
    }

    pub fn chunks(&self) -> &[u16] {
        &self.chunks[..self.chunk_count]
    }

    fn encode(&self, out: &mut [u8]) {
        put64(out, 0, self.revision);
        put32(out, 8, self.length as u32);
        put16(out, 12, self.chunk_count as u16);
        out[14] = self.name_length as u8;
        put64(out, 16, self.content_sum);
        out[24..56].copy_from_slice(&self.name);
        for (index, chunk) in self.chunks.iter().enumerate() {
            put16(out, 56 + index * 2, *chunk);
        }
        for (index, sum) in self.sums.iter().enumerate() {
            put64(out, 88 + index * 8, *sum);
        }
    }

    fn decode(bytes: &[u8]) -> Result<Self, Status> {
        let mut version = Self {
            revision: get64(bytes, 0),
            length: get32(bytes, 8) as usize,
            chunk_count: usize::from(get16(bytes, 12)),
            name_length: usize::from(bytes[14]),
            name: [0; NAME],
            content_sum: get64(bytes, 16),
            chunks: [NO_CHUNK; MAX_CHUNKS],
            sums: [0; MAX_CHUNKS],
        };
        version.name.copy_from_slice(&bytes[24..56]);
        for index in 0..MAX_CHUNKS {
            version.chunks[index] = get16(bytes, 56 + index * 2);
            version.sums[index] = get64(bytes, 88 + index * 8);
        }
        if bytes[15] != 0 || version.chunk_count > MAX_CHUNKS || version.name_length > NAME {
            return Err(Status::Corrupt);
        }
        let canonical = version.name[version.name_length..]
            .iter()
            .all(|byte| *byte == 0)
            && version.chunks[version.chunk_count..]
                .iter()
                .all(|chunk| *chunk == NO_CHUNK)
            && version.sums[version.chunk_count..]
                .iter()
                .all(|sum| *sum == 0)
            && version
                .chunks()
                .iter()
                .all(|chunk| usize::from(*chunk) < CHUNKS);
        let shape = if version.revision == 0 {
            version == Self::EMPTY
        } else {
            valid_name(version.name())
                && version.length <= MAX_LENGTH
                && version.chunk_count == chunks_for(version.length)
        };
        if canonical && shape {
            Ok(version)
        } else {
            Err(Status::Corrupt)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Object revision; increments on every mutation of this object.
    pub revision: u64,
    /// Request that produced this revision; lets a client resolve an earlier
    /// Unknown outcome by reading state instead of replaying a write.
    pub request_id: u64,
    /// Logical time: the owning domain's clock at creation / last mutation.
    pub created: u64,
    pub modified: u64,
    pub tags: u32,
    pub kind: u16,
    pub present: bool,
    /// 0 = no collection, 1..=COLLECTIONS within the owning domain.
    pub collection: u8,
    pub history_count: usize,
    pub current: Version,
    /// Newest first. Deleting moves `current` here, so deletes are undoable.
    pub history: [Version; HISTORY],
}

impl Entry {
    pub const EMPTY: Self = Self {
        revision: 0,
        request_id: 0,
        created: 0,
        modified: 0,
        tags: 0,
        kind: 0,
        present: false,
        collection: 0,
        history_count: 0,
        current: Version::EMPTY,
        history: [Version::EMPTY; HISTORY],
    };

    pub fn name(&self) -> &[u8] {
        self.current.name()
    }
    pub fn length(&self) -> usize {
        self.current.length
    }

    /// Every version this object retains, current first.
    pub fn versions(&self) -> impl Iterator<Item = &Version> {
        core::iter::once(&self.current)
            .filter(move |_| self.present)
            .chain(self.history[..self.history_count].iter())
    }

    /// Unique chunks retained by this object (versions may share chunks).
    pub fn chunk_usage(&self) -> usize {
        let mut seen = [false; CHUNKS];
        let mut count = 0;
        for version in self.versions() {
            for chunk in version.chunks() {
                let chunk = usize::from(*chunk);
                if !seen[chunk] {
                    seen[chunk] = true;
                    count += 1;
                }
            }
        }
        count
    }

    pub fn references(&self, chunk: u16) -> bool {
        self.versions()
            .any(|version| version.chunks().contains(&chunk))
    }

    /// Push the current snapshot onto the history, dropping the oldest.
    pub fn push_history(&mut self) {
        if self.present {
            for index in (1..HISTORY).rev() {
                self.history[index] = self.history[index - 1];
            }
            self.history[0] = self.current;
            self.history_count = core::cmp::min(self.history_count + 1, HISTORY);
        }
    }

    /// Remove history version `slot` (< history_count), keeping the order of
    /// the others, and return it.
    pub fn take_history(&mut self, slot: usize) -> Version {
        assert!(slot < self.history_count);
        let taken = self.history[slot];
        for index in slot..HISTORY - 1 {
            self.history[index] = self.history[index + 1];
        }
        self.history_count -= 1;
        self.history[self.history_count..].fill(Version::EMPTY);
        taken
    }

    fn encode_head(&self, out: &mut [u8]) {
        put64(out, 0, self.revision);
        put64(out, 8, self.request_id);
        put64(out, 16, self.created);
        put64(out, 24, self.modified);
        put32(out, 32, self.tags);
        put16(out, 36, self.kind);
        out[38] = u8::from(self.present);
        out[39] = self.collection;
        out[40] = self.history_count as u8;
        self.current.encode(&mut out[HEAD..HEAD + VERSION_BYTES]);
    }

    fn encode_history(&self, out: &mut [u8]) {
        for (index, version) in self.history.iter().enumerate() {
            version.encode(&mut out[index * VERSION_BYTES..(index + 1) * VERSION_BYTES]);
        }
    }

    fn decode_head(&mut self, bytes: &[u8], generation: u64) -> Result<(), Status> {
        self.revision = get64(bytes, 0);
        self.request_id = get64(bytes, 8);
        self.created = get64(bytes, 16);
        self.modified = get64(bytes, 24);
        self.tags = get32(bytes, 32);
        self.kind = get16(bytes, 36);
        self.present = match bytes[38] {
            0 => false,
            1 => true,
            _ => return Err(Status::Corrupt),
        };
        self.collection = bytes[39];
        self.history_count = usize::from(bytes[40]);
        self.current = Version::decode(&bytes[HEAD..HEAD + VERSION_BYTES])?;
        if bytes[41..HEAD].iter().any(|byte| *byte != 0)
            || bytes[HEAD + VERSION_BYTES..TRAILER]
                .iter()
                .any(|byte| *byte != 0)
            || self.history_count > HISTORY
            || usize::from(self.collection) > COLLECTIONS
            || self.revision > generation
            || self.modified > generation
            || self.created > self.modified
        {
            return Err(Status::Corrupt);
        }
        Ok(())
    }

    fn decode_history(&mut self, bytes: &[u8]) -> Result<(), Status> {
        if bytes[HISTORY * VERSION_BYTES..TRAILER]
            .iter()
            .any(|byte| *byte != 0)
        {
            return Err(Status::Corrupt);
        }
        for index in 0..HISTORY {
            self.history[index] =
                Version::decode(&bytes[index * VERSION_BYTES..(index + 1) * VERSION_BYTES])?;
        }
        Ok(())
    }

    /// Per-object invariants that need both sectors.
    fn validate(&self) -> bool {
        if self.revision == 0 {
            return *self == Self::EMPTY;
        }
        let mut ok = self.request_id != 0 && self.created != 0 && self.modified != 0;
        if self.present {
            ok &= self.current.revision != 0 && self.current.revision <= self.revision;
        } else {
            ok &= self.current == Version::EMPTY
                && self.collection == 0
                && self.tags == 0
                && self.kind == 0;
        }
        let mut newer = if self.present {
            self.current.revision
        } else {
            self.revision + 1
        };
        for (index, version) in self.history.iter().enumerate() {
            if index < self.history_count {
                ok &= version.revision != 0 && version.revision < newer;
                newer = version.revision;
            } else {
                ok &= *version == Version::EMPTY;
            }
        }
        // Versions of one object may share a chunk only at the same position
        // with the same checksum (a restore re-references, never rewrites).
        let mut positions = [(u8::MAX, 0u64); CHUNKS];
        for version in self.versions() {
            for (position, chunk) in version.chunks().iter().enumerate() {
                let slot = &mut positions[usize::from(*chunk)];
                let sum = version.sums[position];
                if slot.0 == u8::MAX {
                    *slot = (position as u8, sum);
                } else if *slot != (position as u8, sum) {
                    ok = false;
                }
            }
        }
        ok
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Collection {
    pub revision: u64,
    pub request_id: u64,
    pub present: bool,
    pub name_length: usize,
    pub name: [u8; NAME],
}

impl Collection {
    pub const EMPTY: Self = Self {
        revision: 0,
        request_id: 0,
        present: false,
        name_length: 0,
        name: [0; NAME],
    };

    pub fn name(&self) -> &[u8] {
        &self.name[..self.name_length]
    }

    fn encode(&self, out: &mut [u8]) {
        put64(out, 0, self.revision);
        put64(out, 8, self.request_id);
        out[16] = self.name_length as u8;
        out[17] = u8::from(self.present);
        out[24..56].copy_from_slice(&self.name);
    }

    fn decode(bytes: &[u8], generation: u64) -> Result<Self, Status> {
        let mut collection = Self {
            revision: get64(bytes, 0),
            request_id: get64(bytes, 8),
            present: false,
            name_length: usize::from(bytes[16]),
            name: [0; NAME],
        };
        collection.name.copy_from_slice(&bytes[24..56]);
        collection.present = match bytes[17] {
            0 => false,
            1 => true,
            _ => return Err(Status::Corrupt),
        };
        let ok = bytes[18..24].iter().all(|byte| *byte == 0)
            && collection.revision <= generation
            && (collection.revision == 0) == (collection.request_id == 0)
            && collection.name_length <= NAME
            && collection.name[collection.name_length..]
                .iter()
                .all(|byte| *byte == 0)
            && if collection.present {
                collection.revision != 0 && valid_name(collection.name())
            } else {
                collection.name_length == 0
            };
        if ok {
            Ok(collection)
        } else {
            Err(Status::Corrupt)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Table {
    pub generation: u64,
    pub request_id: u64,
    /// Per-domain logical clock: the number of commits made by that domain.
    /// Object `created`/`modified` stamps use the owning domain's clock, so
    /// one domain cannot count another domain's commits. Every commit comes
    /// from exactly one domain, hence the clocks sum to `generation`.
    pub clocks: [u64; DOMAINS],
    pub entries: [Entry; OBJECTS],
    /// `domain * COLLECTIONS + (index - 1)`.
    pub collections: [Collection; DOMAINS * COLLECTIONS],
}

/// Header offset of the per-domain clocks (one u64 per domain).
const CLOCKS: usize = 40;

/// The table one commit publishes: its generation, request and domain
/// clocks plus the replacement objects and collections.
#[derive(Clone, Copy)]
pub struct Next<'a> {
    pub generation: u64,
    pub request_id: u64,
    pub clocks: [u64; DOMAINS],
    pub objects: &'a [(usize, Entry)],
    pub collections: &'a [Collection; DOMAINS * COLLECTIONS],
}

impl Default for Table {
    fn default() -> Self {
        Self::EMPTY
    }
}

/// Sector layout: 0 header, 1 collections, then per object `2 + 2o` (head +
/// current version) and `3 + 2o` (history).
pub const fn object_sectors(object: usize) -> (u64, u64) {
    (2 + 2 * object as u64, 3 + 2 * object as u64)
}

impl Table {
    /// An all-zero volume is the empty generation-0 state; nothing is formatted.
    pub const EMPTY: Self = Self {
        generation: 0,
        request_id: 0,
        clocks: [0; DOMAINS],
        entries: [Entry::EMPTY; OBJECTS],
        collections: [Collection::EMPTY; DOMAINS * COLLECTIONS],
    };

    pub const fn slot(generation: u64) -> u64 {
        generation % 2
    }

    pub fn collection(&self, domain: usize, index: usize) -> &Collection {
        &self.collections[domain * COLLECTIONS + index - 1]
    }

    pub fn domain_chunks(&self, domain: usize) -> usize {
        self.entries[domain * PER_DOMAIN..(domain + 1) * PER_DOMAIN]
            .iter()
            .map(Entry::chunk_usage)
            .sum()
    }

    /// Sum of the domain's own object and collection revisions. It is a
    /// change detector for paged listings, never a grant.
    pub fn domain_generation(&self, domain: usize) -> u64 {
        let objects = self.entries[domain * PER_DOMAIN..(domain + 1) * PER_DOMAIN]
            .iter()
            .fold(0u64, |sum, entry| sum.wrapping_add(entry.revision));
        self.collections[domain * COLLECTIONS..(domain + 1) * COLLECTIONS]
            .iter()
            .fold(objects, |sum, collection| {
                sum.wrapping_add(collection.revision)
            })
    }

    pub fn references(&self, chunk: u16) -> bool {
        self.entries.iter().any(|entry| entry.references(chunk))
    }

    pub fn encode_sector(&self, index: u64) -> [u8; SECTOR] {
        let next = Next {
            generation: self.generation,
            request_id: self.request_id,
            clocks: self.clocks,
            objects: &[],
            collections: &self.collections,
        };
        self.encode_with(index, &next, self.attempt_digest(&next))
    }

    /// Digest binding all sectors of one write attempt: FNV-64 over the
    /// payload (bytes before the trailer) of every sector of the copy.
    pub fn attempt_digest(&self, next: &Next) -> u64 {
        (0..TABLE_SECTORS).fold(FNV_OFFSET, |digest, index| {
            let payload = self.payload_with(index, next);
            checksum_update(digest, &payload[..TRAILER])
        })
    }

    /// Encode sector `index` of the table `next` describes. This lets a
    /// commit stream its next copy without materializing a second table.
    pub fn encode_with(&self, index: u64, next: &Next, attempt: u64) -> [u8; SECTOR] {
        let mut sector = self.payload_with(index, next);
        seal(
            &mut sector,
            Self::slot(next.generation),
            index,
            next.generation,
            attempt,
        );
        sector
    }

    fn payload_with(&self, index: u64, next: &Next) -> [u8; SECTOR] {
        assert!(index < TABLE_SECTORS);
        let (objects, collections) = (next.objects, next.collections);
        let mut sector = [0; SECTOR];
        match index {
            0 => {
                sector[..8].copy_from_slice(MAGIC);
                put16(&mut sector, 8, VERSION);
                put16(&mut sector, 10, OBJECTS as u16);
                put16(&mut sector, 12, CHUNKS as u16);
                put16(&mut sector, 14, crate::CHUNK_SECTORS as u16);
                put64(&mut sector, 16, next.generation);
                put64(&mut sector, 24, next.request_id);
                put16(&mut sector, 32, MAX_CHUNKS as u16);
                put16(&mut sector, 34, QUOTA as u16);
                sector[36] = HISTORY as u8;
                sector[37] = COLLECTIONS as u8;
                for (domain, clock) in next.clocks.iter().enumerate() {
                    put64(&mut sector, CLOCKS + 8 * domain, *clock);
                }
            }
            1 => {
                for (index, collection) in collections.iter().enumerate() {
                    collection.encode(
                        &mut sector[index * COLLECTION_BYTES..(index + 1) * COLLECTION_BYTES],
                    );
                }
            }
            _ => {
                let object = (index as usize - 2) / 2;
                let entry = objects
                    .iter()
                    .find(|(id, _)| *id == object)
                    .map_or(&self.entries[object], |(_, entry)| entry);
                if index.is_multiple_of(2) {
                    entry.encode_head(&mut sector);
                } else {
                    entry.encode_history(&mut sector);
                }
            }
        }
        sector
    }

    /// Decode one table copy from a sector source, validating every field and
    /// every cross-entry invariant. Streaming keeps the native stack bounded.
    pub fn decode_from(
        slot: u64,
        mut sector: impl FnMut(u64) -> Result<[u8; SECTOR], Status>,
        out: &mut Self,
    ) -> Result<(), Status> {
        *out = Self::EMPTY;
        let mut sealed = (0, 0);
        let mut digest = FNV_OFFSET;
        for index in 0..TABLE_SECTORS {
            let bytes = sector(index)?;
            match classify(slot, index, &bytes) {
                SectorState::Valid(found, attempt) if index == 0 || (found, attempt) == sealed => {
                    sealed = (found, attempt)
                }
                _ => return Err(Status::Corrupt),
            }
            digest = checksum_update(digest, &bytes[..TRAILER]);
            out.decode_sector(slot, index, &bytes, sealed.0)?;
        }
        // All sectors must come from the same write attempt.
        if digest != sealed.1 {
            return Err(Status::Corrupt);
        }
        out.validate()
    }

    fn decode_sector(
        &mut self,
        slot: u64,
        index: u64,
        bytes: &[u8; SECTOR],
        generation: u64,
    ) -> Result<(), Status> {
        match index {
            0 => {
                if Self::slot(generation) != slot
                    || &bytes[..8] != MAGIC
                    || get16(bytes, 8) != VERSION
                    || get16(bytes, 10) != OBJECTS as u16
                    || get16(bytes, 12) != CHUNKS as u16
                    || get16(bytes, 14) != crate::CHUNK_SECTORS as u16
                    || get64(bytes, 16) != generation
                    || get64(bytes, 24) == 0
                    || get16(bytes, 32) != MAX_CHUNKS as u16
                    || get16(bytes, 34) != QUOTA as u16
                    || usize::from(bytes[36]) != HISTORY
                    || usize::from(bytes[37]) != COLLECTIONS
                    || bytes[38..CLOCKS].iter().any(|byte| *byte != 0)
                    || bytes[CLOCKS + 8 * DOMAINS..TRAILER]
                        .iter()
                        .any(|byte| *byte != 0)
                {
                    return Err(Status::Corrupt);
                }
                self.generation = generation;
                self.request_id = get64(bytes, 24);
                for (domain, clock) in self.clocks.iter_mut().enumerate() {
                    *clock = get64(bytes, CLOCKS + 8 * domain);
                }
            }
            1 => {
                let used = DOMAINS * COLLECTIONS * COLLECTION_BYTES;
                if bytes[used..TRAILER].iter().any(|byte| *byte != 0) {
                    return Err(Status::Corrupt);
                }
                for (index, collection) in self.collections.iter_mut().enumerate() {
                    *collection = Collection::decode(
                        &bytes[index * COLLECTION_BYTES..(index + 1) * COLLECTION_BYTES],
                        generation,
                    )?;
                }
            }
            _ => {
                let object = (index as usize - 2) / 2;
                if index.is_multiple_of(2) {
                    self.entries[object].decode_head(bytes, generation)?;
                } else {
                    self.entries[object].decode_history(bytes)?;
                }
            }
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), Status> {
        // Every commit advances exactly one domain clock. Each commit bumps a
        // given object or collection at most once, so no revision or stamp
        // can exceed its own domain's clock.
        let total = self
            .clocks
            .iter()
            .try_fold(0u64, |sum, clock| sum.checked_add(*clock));
        if total != Some(self.generation)
            || self
                .collections
                .iter()
                .enumerate()
                .any(|(index, collection)| collection.revision > self.clocks[index / COLLECTIONS])
        {
            return Err(Status::Corrupt);
        }
        let mut owner = [u8::MAX; CHUNKS];
        for (object, entry) in self.entries.iter().enumerate() {
            let clock = self.clocks[domain_of(object)];
            if !entry.validate() || entry.revision > clock || entry.modified > clock {
                return Err(Status::Corrupt);
            }
            if entry.collection != 0
                && !self
                    .collection(domain_of(object), usize::from(entry.collection))
                    .present
            {
                return Err(Status::Corrupt);
            }
            for version in entry.versions() {
                for chunk in version.chunks() {
                    let slot = &mut owner[usize::from(*chunk)];
                    if *slot != u8::MAX && *slot != object as u8 {
                        return Err(Status::Corrupt);
                    }
                    *slot = object as u8;
                }
            }
        }
        if (0..DOMAINS).any(|domain| self.domain_chunks(domain) > QUOTA) {
            return Err(Status::Corrupt);
        }
        // The last commit must be reflected by at least one object or
        // collection it changed (every commit records its request ID).
        let reflected = self
            .entries
            .iter()
            .any(|entry| entry.revision != 0 && entry.request_id == self.request_id)
            || self.collections.iter().any(|collection| {
                collection.revision != 0 && collection.request_id == self.request_id
            });
        if !reflected {
            return Err(Status::Corrupt);
        }
        Ok(())
    }
}

fn sector_sum(slot: u64, index: u64, sector: &[u8; SECTOR]) -> u64 {
    let value = checksum_update(FNV_OFFSET, &[slot as u8, index as u8]);
    checksum_update(value, &sector[..504])
}

fn seal(sector: &mut [u8; SECTOR], slot: u64, index: u64, generation: u64, attempt: u64) {
    put64(sector, TRAILER, generation);
    put64(sector, TRAILER + 8, attempt);
    let sum = sector_sum(slot, index, sector);
    put64(sector, 504, sum);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectorState {
    /// Never written: the whole sector is zero.
    Zero,
    /// Trailer and checksum match this slot/index; carries the generation
    /// and the digest of the write attempt that produced the sector.
    Valid(u64, u64),
    /// Garbage, misplaced or partially written.
    Invalid,
}

fn is_format_marker(sector: &[u8; SECTOR]) -> bool {
    &sector[..8] == FORMAT_MARKER_MAGIC
        && get16(sector, 8) == FORMAT_MARKER_VERSION
        && get16(sector, 10) == VERSION
        && get32(sector, 12) == DATA_START as u32
        && get32(sector, 16) == VOLUME_SECTORS as u32
        && get32(sector, 20) == CHUNKS as u32
        && sector[24..504].iter().all(|byte| *byte == 0)
        && get64(sector, 504) == checksum_update(FNV_OFFSET, &sector[..504])
}

pub fn classify(slot: u64, index: u64, sector: &[u8; SECTOR]) -> SectorState {
    if sector.iter().all(|byte| *byte == 0) {
        return SectorState::Zero;
    }
    let generation = get64(sector, TRAILER);
    if get64(sector, 504) != sector_sum(slot, index, sector) {
        SectorState::Invalid
    } else {
        SectorState::Valid(generation, get64(sector, TRAILER + 8))
    }
}

/// One table copy as read from disk, before the cross-copy decision. Only
/// sector states and validity are kept; the chosen copy is decoded again.
#[derive(Clone, Copy, Debug)]
pub struct SlotScan {
    pub states: [SectorState; TABLE_SECTORS_USIZE],
    format_marker: bool,
    /// Generation of a completely valid copy.
    pub valid: Option<u64>,
}

impl SlotScan {
    /// Classify every sector of `slot`; only a uniformly sealed copy is then
    /// fully decoded (into `scratch`) to decide validity.
    pub fn scan(
        slot: u64,
        mut sector: impl FnMut(u64) -> Result<[u8; SECTOR], Status>,
        scratch: &mut Table,
    ) -> Result<Self, Status> {
        let mut states = [SectorState::Zero; TABLE_SECTORS_USIZE];
        let mut format_marker = false;
        for index in 0..TABLE_SECTORS {
            let bytes = sector(index)?;
            if slot == 0 && index == 0 {
                format_marker = is_format_marker(&bytes);
            }
            states[index as usize] = classify(slot, index, &bytes);
        }
        let mut scan = Self {
            states,
            format_marker,
            valid: None,
        };
        if scan.uniform() {
            let mut io = Ok(());
            let decoded = Table::decode_from(
                slot,
                |index| {
                    let bytes = sector(index);
                    if let Err(status) = bytes {
                        io = Err(status);
                    }
                    bytes
                },
                scratch,
            );
            io?;
            scan.valid = decoded.ok().map(|()| scratch.generation);
        }
        Ok(scan)
    }

    fn all_zero(&self) -> bool {
        self.states.iter().all(|state| *state == SectorState::Zero)
    }

    fn format_marker_only(&self) -> bool {
        self.format_marker
            && self.states[1..]
                .iter()
                .all(|state| *state == SectorState::Zero)
    }

    /// Every sector carries the same valid generation. Such a copy looks
    /// complete; if it nevertheless failed to decode it is corruption, not a
    /// torn write, and recovery must stop rather than silently skip it.
    fn uniform(&self) -> bool {
        match self.states[0] {
            SectorState::Valid(g, a) => self.states.iter().all(|s| *s == SectorState::Valid(g, a)),
            _ => false,
        }
    }

    /// Acceptable companion of a durable generation `g`: each sector is the
    /// previous copy `g-1` (zero when `g <= 1`), a written sector of the
    /// interrupted next attempt `g+1`, or a torn sector.
    fn compatible_with(&self, g: u64) -> bool {
        if self.uniform() && self.valid.is_none() {
            return false;
        }
        self.states.iter().all(|state| match *state {
            SectorState::Zero => g == 1,
            SectorState::Valid(found, _) => {
                found.checked_add(1) == Some(g) || g.checked_add(1) == Some(found)
            }
            SectorState::Invalid => true,
        })
    }
}

/// Select a fully validated durable copy, or an uncommitted empty volume.
pub fn choose(slots: [SlotScan; 2]) -> Result<Option<u64>, Status> {
    let [zero, one] = slots;
    match (zero.valid, one.valid) {
        (Some(a), Some(b)) => {
            if a.min(b).checked_add(1) == Some(a.max(b)) {
                Ok(Some(Table::slot(a.max(b))))
            } else {
                Err(Status::Corrupt)
            }
        }
        (Some(g), None) if one.compatible_with(g) => Ok(Some(0)),
        (None, Some(g)) if zero.compatible_with(g) => Ok(Some(1)),
        (None, None) => {
            // Only "never committed" is acceptable: slot 0 untouched and slot 1
            // either untouched or a torn attempt at generation 1.
            let first_attempt = !one.uniform()
                && one.states.iter().all(|state| {
                    matches!(
                        state,
                        SectorState::Zero | SectorState::Valid(1, _) | SectorState::Invalid
                    )
                });
            if (zero.all_zero() || zero.format_marker_only()) && (one.all_zero() || first_attempt) {
                Ok(None)
            } else {
                Err(Status::Corrupt)
            }
        }
        _ => Err(Status::Corrupt),
    }
}

/// Recover the durable table through the service's own disk capability into
/// `out`. Unmarked empty volumes are checked in full; committed content is
/// verified lazily on access.
pub fn recover(disk: &mut impl Disk, out: &mut Table) -> Result<(), Status> {
    recover_with_state(disk, out).map(|_| ())
}

/// Recover the durable table and report whether this volume has ever been
/// explicitly claimed by the NOSVOL02 writer. An unmarked empty volume is
/// accepted only when every sector in the scoped volume is zero.
pub fn recover_with_state(disk: &mut impl Disk, out: &mut Table) -> Result<bool, Status> {
    let zero = SlotScan::scan(0, |index| read_sector(disk, index), out)?;
    let one = SlotScan::scan(1, |index| read_sector(disk, TABLE_SECTORS + index), out)?;
    match choose([zero, one])? {
        None => {
            let initialized = zero.format_marker_only();
            if !initialized {
                if !zero.all_zero() || !one.all_zero() {
                    return Err(Status::Corrupt);
                }
                for index in DATA_START..VOLUME_SECTORS {
                    if read_sector(disk, index)?.iter().any(|byte| *byte != 0) {
                        return Err(Status::Corrupt);
                    }
                }
            }
            *out = Table::EMPTY;
            Ok(initialized)
        }
        Some(slot) => {
            Table::decode_from(
                slot,
                |index| read_sector(disk, slot * TABLE_SECTORS + index),
                out,
            )?;
            Ok(true)
        }
    }
}

/// Persist a recognizable format marker before the first staged content or
/// metadata write. The checksum detects damage; this marker is not an
/// authentication mechanism.
pub fn write_format_marker(disk: &mut impl Disk) -> Result<(), Status> {
    let mut sector = [0; SECTOR];
    sector[..8].copy_from_slice(FORMAT_MARKER_MAGIC);
    put16(&mut sector, 8, FORMAT_MARKER_VERSION);
    put16(&mut sector, 10, VERSION);
    put32(&mut sector, 12, DATA_START as u32);
    put32(&mut sector, 16, VOLUME_SECTORS as u32);
    put32(&mut sector, 20, CHUNKS as u32);
    let checksum = checksum_update(FNV_OFFSET, &sector[..504]);
    put64(&mut sector, 504, checksum);
    disk.write(0, &sector)?;
    disk.flush()
}

fn read_sector(disk: &mut impl Disk, sector: u64) -> Result<[u8; SECTOR], Status> {
    let mut bytes = [0; SECTOR];
    disk.read(sector, &mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::{choose, SectorState, SlotScan, TABLE_SECTORS_USIZE};
    use crate::Status;

    #[test]
    fn adjacent_generation_checks_do_not_wrap() {
        let valid_zero = SlotScan {
            states: [SectorState::Valid(0, 7); TABLE_SECTORS_USIZE],
            format_marker: false,
            valid: Some(0),
        };
        let mut mixed = SlotScan {
            states: [SectorState::Zero; TABLE_SECTORS_USIZE],
            format_marker: false,
            valid: None,
        };
        mixed.states[0] = SectorState::Valid(u64::MAX, 8);
        assert_eq!(choose([valid_zero, mixed]), Err(Status::Corrupt));
        let maximum = SlotScan {
            states: [SectorState::Valid(u64::MAX, 8); TABLE_SECTORS_USIZE],
            format_marker: false,
            valid: Some(u64::MAX),
        };
        assert_eq!(choose([maximum, maximum]), Err(Status::Corrupt));
        mixed.states.fill(SectorState::Invalid);
        mixed.states[0] = SectorState::Valid(0, 7);
        assert_eq!(choose([mixed, maximum]), Err(Status::Corrupt));
    }
}
