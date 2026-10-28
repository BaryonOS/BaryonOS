//! Volume table v1 and fail-closed recovery.
//!
//! Generation `g` is always written to slot `g % 2`. Every table sector ends
//! with the table generation and an FNV-64 over slot, sector index and the
//! first 504 bytes, so a torn table write is visible per sector. Recovery may
//! only pick the newest completely valid copy when the other copy is either
//! the previous generation or an interrupted attempt at the next generation.
use crate::{
    checksum_update, chunks_for, domain_of, get16, get32, get64, put16, put32, put64, valid_name,
    Disk, Status, CHUNKS, DOMAINS, FNV_OFFSET, MAX_CHUNKS, MAX_LENGTH, NAME, NO_CHUNK, OBJECTS,
    PER_DOMAIN, QUOTA, SECTOR, TABLE_SECTORS,
};

pub const MAGIC: &[u8; 8] = b"NOSVOL01";
pub const VERSION: u16 = 1;
pub const ENTRY: usize = 240;
pub const TABLE_SECTORS_USIZE: usize = TABLE_SECTORS as usize;
const TRAILER: usize = 496;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub revision: u64,
    /// Request that produced this revision; lets a client resolve an earlier
    /// Unknown outcome by reading state instead of replaying a write.
    pub request_id: u64,
    pub length: usize,
    pub present: bool,
    pub name_length: usize,
    pub name: [u8; NAME],
    pub chunk_count: usize,
    pub content_sum: u64,
    pub chunks: [u16; MAX_CHUNKS],
    pub sums: [u64; MAX_CHUNKS],
}

impl Entry {
    pub const EMPTY: Self = Self {
        revision: 0,
        request_id: 0,
        length: 0,
        present: false,
        name_length: 0,
        name: [0; NAME],
        chunk_count: 0,
        content_sum: 0,
        chunks: [NO_CHUNK; MAX_CHUNKS],
        sums: [0; MAX_CHUNKS],
    };

    pub fn name(&self) -> &[u8] {
        &self.name[..self.name_length]
    }

    /// A tombstone keeps only its revision and the deleting request.
    pub fn tombstone(revision: u64, request_id: u64) -> Self {
        Self {
            revision,
            request_id,
            ..Self::EMPTY
        }
    }

    fn encode(&self, out: &mut [u8]) {
        put64(out, 0, self.revision);
        put64(out, 8, self.request_id);
        put32(out, 16, self.length as u32);
        out[20] = u8::from(self.present);
        out[21] = self.name_length as u8;
        put16(out, 22, self.chunk_count as u16);
        put64(out, 24, self.content_sum);
        out[32..64].copy_from_slice(&self.name);
        for (index, chunk) in self.chunks.iter().enumerate() {
            put16(out, 64 + index * 2, *chunk);
        }
        for (index, sum) in self.sums.iter().enumerate() {
            put64(out, 96 + index * 8, *sum);
        }
    }

    fn decode(bytes: &[u8], generation: u64) -> Result<Self, Status> {
        let mut entry = Self {
            revision: get64(bytes, 0),
            request_id: get64(bytes, 8),
            length: get32(bytes, 16) as usize,
            present: false,
            name_length: usize::from(bytes[21]),
            name: [0; NAME],
            chunk_count: usize::from(get16(bytes, 22)),
            content_sum: get64(bytes, 24),
            chunks: [NO_CHUNK; MAX_CHUNKS],
            sums: [0; MAX_CHUNKS],
        };
        entry.name.copy_from_slice(&bytes[32..64]);
        for index in 0..MAX_CHUNKS {
            entry.chunks[index] = get16(bytes, 64 + index * 2);
            entry.sums[index] = get64(bytes, 96 + index * 8);
        }
        let present = match bytes[20] {
            0 => false,
            1 => true,
            _ => return Err(Status::Corrupt),
        };
        entry.present = present;
        let canonical = bytes[224..ENTRY].iter().all(|byte| *byte == 0)
            && entry.revision <= generation
            && (entry.revision == 0) == (entry.request_id == 0)
            && entry.name_length <= NAME
            && entry.name[entry.name_length..]
                .iter()
                .all(|byte| *byte == 0)
            && entry.chunk_count <= MAX_CHUNKS
            && entry.chunks[entry.chunk_count..]
                .iter()
                .all(|chunk| *chunk == NO_CHUNK)
            && entry.sums[entry.chunk_count..].iter().all(|sum| *sum == 0)
            && entry.chunks[..entry.chunk_count]
                .iter()
                .all(|chunk| usize::from(*chunk) < CHUNKS);
        let shape = if present {
            entry.revision != 0
                && valid_name(entry.name())
                && entry.length <= MAX_LENGTH
                && entry.chunk_count == chunks_for(entry.length)
        } else {
            entry.length == 0
                && entry.name_length == 0
                && entry.chunk_count == 0
                && entry.content_sum == 0
        };
        if canonical && shape {
            Ok(entry)
        } else {
            Err(Status::Corrupt)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Table {
    pub generation: u64,
    pub request_id: u64,
    pub entries: [Entry; OBJECTS],
}

impl Default for Table {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Table {
    /// An all-zero volume is the empty generation-0 state; nothing is formatted.
    pub const EMPTY: Self = Self {
        generation: 0,
        request_id: 0,
        entries: [Entry::EMPTY; OBJECTS],
    };

    pub const fn slot(generation: u64) -> u64 {
        generation % 2
    }

    pub fn domain_chunks(&self, domain: usize) -> usize {
        self.entries[domain * PER_DOMAIN..(domain + 1) * PER_DOMAIN]
            .iter()
            .map(|entry| entry.chunk_count)
            .sum()
    }

    /// Sum of the domain's own object revisions (including tombstones). It is
    /// a change detector for paged listings, never a grant.
    pub fn domain_generation(&self, domain: usize) -> u64 {
        self.entries[domain * PER_DOMAIN..(domain + 1) * PER_DOMAIN]
            .iter()
            .fold(0u64, |sum, entry| sum.wrapping_add(entry.revision))
    }

    pub fn references(&self, chunk: u16) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.chunks[..entry.chunk_count].contains(&chunk))
    }

    pub fn encode_sector(&self, index: u64) -> [u8; SECTOR] {
        assert!(index < TABLE_SECTORS);
        let mut sector = [0; SECTOR];
        if index == 0 {
            sector[..8].copy_from_slice(MAGIC);
            put16(&mut sector, 8, VERSION);
            put16(&mut sector, 10, OBJECTS as u16);
            put16(&mut sector, 12, CHUNKS as u16);
            put16(&mut sector, 14, crate::CHUNK_SECTORS as u16);
            put64(&mut sector, 16, self.generation);
            put64(&mut sector, 24, self.request_id);
            put16(&mut sector, 32, MAX_CHUNKS as u16);
            put16(&mut sector, 34, QUOTA as u16);
        } else {
            let first = (index as usize - 1) * 2;
            for half in 0..2 {
                self.entries[first + half].encode(&mut sector[half * ENTRY..(half + 1) * ENTRY]);
            }
        }
        seal(
            &mut sector,
            Self::slot(self.generation),
            index,
            self.generation,
        );
        sector
    }

    /// Validate one complete table copy. Every cross-entry invariant is
    /// checked here so a decoded table can never alias or overcommit chunks.
    pub fn decode(
        slot: u64,
        sectors: &[[u8; SECTOR]; TABLE_SECTORS as usize],
    ) -> Result<Self, Status> {
        let mut generation = None;
        for (index, sector) in sectors.iter().enumerate() {
            match classify(slot, index as u64, sector) {
                SectorState::Valid(found) if generation.is_none_or(|g| g == found) => {
                    generation = Some(found)
                }
                _ => return Err(Status::Corrupt),
            }
        }
        let generation = generation.ok_or(Status::Corrupt)?;
        let header = &sectors[0];
        if generation == 0
            || Self::slot(generation) != slot
            || &header[..8] != MAGIC
            || get16(header, 8) != VERSION
            || get16(header, 10) != OBJECTS as u16
            || get16(header, 12) != CHUNKS as u16
            || get16(header, 14) != crate::CHUNK_SECTORS as u16
            || get64(header, 16) != generation
            || get64(header, 24) == 0
            || get16(header, 32) != MAX_CHUNKS as u16
            || get16(header, 34) != QUOTA as u16
            || header[36..TRAILER].iter().any(|byte| *byte != 0)
        {
            return Err(Status::Corrupt);
        }
        let mut table = Self {
            generation,
            request_id: get64(header, 24),
            entries: [Entry::EMPTY; OBJECTS],
        };
        for (index, sector) in sectors.iter().enumerate().skip(1) {
            if sector[2 * ENTRY..TRAILER].iter().any(|byte| *byte != 0) {
                return Err(Status::Corrupt);
            }
            for half in 0..2 {
                table.entries[(index - 1) * 2 + half] =
                    Entry::decode(&sector[half * ENTRY..(half + 1) * ENTRY], generation)?;
            }
        }
        let mut used = [false; CHUNKS];
        for entry in &table.entries {
            for chunk in &entry.chunks[..entry.chunk_count] {
                let chunk = usize::from(*chunk);
                if used[chunk] {
                    return Err(Status::Corrupt);
                }
                used[chunk] = true;
            }
        }
        if (0..DOMAINS).any(|domain| table.domain_chunks(domain) > QUOTA) {
            return Err(Status::Corrupt);
        }
        // The last committing request must own at least one current revision:
        // every commit changes exactly one object and records its request ID.
        if !table
            .entries
            .iter()
            .any(|entry| entry.revision != 0 && entry.request_id == table.request_id)
        {
            return Err(Status::Corrupt);
        }
        Ok(table)
    }

    pub fn object(&self, object: usize) -> &Entry {
        &self.entries[object]
    }

    pub fn owner_domain(object: usize) -> usize {
        domain_of(object)
    }
}

fn sector_sum(slot: u64, index: u64, sector: &[u8; SECTOR]) -> u64 {
    let value = checksum_update(FNV_OFFSET, &[slot as u8, index as u8]);
    checksum_update(value, &sector[..504])
}

fn seal(sector: &mut [u8; SECTOR], slot: u64, index: u64, generation: u64) {
    put64(sector, TRAILER, generation);
    let sum = sector_sum(slot, index, sector);
    put64(sector, 504, sum);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectorState {
    /// Never written: the whole sector is zero.
    Zero,
    /// Trailer and checksum match this slot/index; carries the generation.
    Valid(u64),
    /// Garbage, misplaced or partially written.
    Invalid,
}

pub fn classify(slot: u64, index: u64, sector: &[u8; SECTOR]) -> SectorState {
    if sector.iter().all(|byte| *byte == 0) {
        return SectorState::Zero;
    }
    let generation = get64(sector, TRAILER);
    if generation == 0 || get64(sector, 504) != sector_sum(slot, index, sector) {
        SectorState::Invalid
    } else {
        SectorState::Valid(generation)
    }
}

/// One table copy as read from disk, before the cross-copy decision.
pub struct SlotScan {
    pub states: [SectorState; TABLE_SECTORS as usize],
    pub table: Option<Table>,
}

impl SlotScan {
    pub fn scan(slot: u64, sectors: &[[u8; SECTOR]; TABLE_SECTORS as usize]) -> Self {
        let mut states = [SectorState::Zero; TABLE_SECTORS as usize];
        for (index, sector) in sectors.iter().enumerate() {
            states[index] = classify(slot, index as u64, sector);
        }
        Self {
            states,
            table: Table::decode(slot, sectors).ok(),
        }
    }

    fn all_zero(&self) -> bool {
        self.states.iter().all(|state| *state == SectorState::Zero)
    }

    /// Every sector carries the same valid generation. Such a copy looks
    /// complete; if it nevertheless failed to decode it is corruption, not a
    /// torn write, and recovery must stop rather than silently skip it.
    fn uniform(&self) -> bool {
        match self.states[0] {
            SectorState::Valid(g) => self.states.iter().all(|s| *s == SectorState::Valid(g)),
            _ => false,
        }
    }

    /// Acceptable companion of a durable generation `g`: each sector is the
    /// previous copy `g-1` (zero when `g == 1`), a written sector of the
    /// interrupted next attempt `g+1`, or a torn sector.
    fn compatible_with(&self, g: u64) -> bool {
        if self.uniform() && self.table.is_none() {
            return false;
        }
        self.states.iter().all(|state| match *state {
            SectorState::Zero => g == 1,
            SectorState::Valid(found) => found + 1 == g || found == g + 1,
            SectorState::Invalid => true,
        })
    }
}

/// Decide the durable table from both scanned copies.
pub fn choose(slots: [SlotScan; 2]) -> Result<Table, Status> {
    let [zero, one] = slots;
    match (zero.table, one.table) {
        (Some(a), Some(b)) => {
            let (newer, older) = if a.generation > b.generation {
                (a, b)
            } else {
                (b, a)
            };
            if newer.generation == older.generation + 1 {
                Ok(newer)
            } else {
                Err(Status::Corrupt)
            }
        }
        (Some(table), None) => {
            if one.compatible_with(table.generation) {
                Ok(table)
            } else {
                Err(Status::Corrupt)
            }
        }
        (None, Some(table)) => {
            if zero.compatible_with(table.generation) {
                Ok(table)
            } else {
                Err(Status::Corrupt)
            }
        }
        (None, None) => {
            // Only "never committed" is acceptable: slot 0 untouched and slot 1
            // either untouched or a torn attempt at generation 1.
            let first_attempt = !one.uniform()
                && one.states.iter().all(|state| {
                    matches!(
                        state,
                        SectorState::Zero | SectorState::Valid(1) | SectorState::Invalid
                    )
                });
            if zero.all_zero() && (one.all_zero() || first_attempt) {
                Ok(Table::EMPTY)
            } else {
                Err(Status::Corrupt)
            }
        }
    }
}

/// Read both table copies through the service's own disk capability. Only
/// table sectors are read; content chunks are verified lazily on access.
pub fn recover(disk: &mut impl Disk) -> Result<Table, Status> {
    let mut sectors = [[0; SECTOR]; TABLE_SECTORS as usize];
    let mut scans = [None, None];
    for slot in 0..2u64 {
        for index in 0..TABLE_SECTORS {
            disk.read(slot * TABLE_SECTORS + index, &mut sectors[index as usize])?;
        }
        scans[slot as usize] = Some(SlotScan::scan(slot, &sectors));
    }
    let [Some(zero), Some(one)] = scans else {
        return Err(Status::Corrupt);
    };
    choose([zero, one])
}
