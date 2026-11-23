//! Catalog copy codec, validator and mount rule (spec 2.6).
//!
//! Layout of one 2048-byte copy, little-endian: magic `NOSDST\0\x01` (0..8),
//! version u16 = 1 (8..10), sealed-entry count u8 <= 16 (10), five zero bytes
//! (11..16), destination ID (16..32), sequence u64 != 0 (32..40), next entry
//! ID (40..48), `next_free` (48..56), formatted sector count (56..64), then 16
//! records of 80 bytes (64..1344), zero padding (1344..2040) and FNV-64 over
//! 0..2040 (2040..2048). Copy A occupies sectors 0-3, copy B sectors 4-7.
use crate::{
    fnv64, get64, put64, sectors_for, DATA_START, FNV_OFFSET, ID, LABEL, MAX_ARCHIVE, MAX_ENTRIES,
    MIN_ARCHIVE, SECTOR,
};

pub const COPY: usize = 2048;
pub const COPY_SECTORS: u64 = 4;
/// Catalog area read at mount: both copies plus the reserved sectors 8-15.
pub const AREA_SECTORS: usize = 16;
pub const MAGIC: [u8; 8] = *b"NOSDST\x00\x01";
pub const VERSION: u16 = 1;
pub const RECORD: usize = 80;
pub const RECORDS_AT: usize = 64;
pub const RECORDS_END: usize = RECORDS_AT + MAX_ENTRIES * RECORD;
pub const SUM_AT: usize = 2040;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    pub id: u64,
    pub first: u64,
    pub length: u64,
    pub trailer: u64,
    pub sealed_sequence: u64,
    pub label_length: u8,
    pub label: [u8; LABEL],
}

impl Record {
    pub const EMPTY: Self = Self {
        id: 0,
        first: 0,
        length: 0,
        trailer: 0,
        sealed_sequence: 0,
        label_length: 0,
        label: [0; LABEL],
    };

    pub fn end(&self) -> u64 {
        self.first + sectors_for(self.length)
    }

    pub fn encode(&self, out: &mut [u8]) {
        out[..RECORD].fill(0);
        put64(out, 0, self.id);
        put64(out, 8, self.first);
        put64(out, 16, self.length);
        put64(out, 24, self.trailer);
        put64(out, 32, self.sealed_sequence);
        out[40] = self.label_length;
        out[48..80].copy_from_slice(&self.label);
    }

    fn decode(bytes: &[u8]) -> Self {
        let mut label = [0; LABEL];
        label.copy_from_slice(&bytes[48..80]);
        Self {
            id: get64(bytes, 0),
            first: get64(bytes, 8),
            length: get64(bytes, 16),
            trailer: get64(bytes, 24),
            sealed_sequence: get64(bytes, 32),
            label_length: bytes[40],
            label,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Catalog {
    pub count: usize,
    pub id: [u8; ID],
    pub sequence: u64,
    pub next_entry: u64,
    pub next_free: u64,
    pub formatted: u64,
    pub records: [Record; MAX_ENTRIES],
}

impl Catalog {
    /// The copy FORMAT writes: sequence 1, no entries, next entry ID 1.
    pub fn formatted(id: [u8; ID], formatted: u64) -> Self {
        Self {
            count: 0,
            id,
            sequence: 1,
            next_entry: 1,
            next_free: DATA_START,
            formatted,
            records: [Record::EMPTY; MAX_ENTRIES],
        }
    }

    pub fn entries(&self) -> &[Record] {
        &self.records[..self.count]
    }

    pub fn find(&self, id: u64) -> Option<&Record> {
        self.entries().iter().find(|record| record.id == id)
    }

    pub fn encode(&self) -> [u8; COPY] {
        let mut out = [0; COPY];
        out[..8].copy_from_slice(&MAGIC);
        out[8..10].copy_from_slice(&VERSION.to_le_bytes());
        out[10] = self.count as u8;
        out[16..32].copy_from_slice(&self.id);
        put64(&mut out, 32, self.sequence);
        put64(&mut out, 40, self.next_entry);
        put64(&mut out, 48, self.next_free);
        put64(&mut out, 56, self.formatted);
        for (index, record) in self.entries().iter().enumerate() {
            record.encode(&mut out[RECORDS_AT + index * RECORD..]);
        }
        let sum = fnv64(FNV_OFFSET, &out[..SUM_AT]);
        put64(&mut out, SUM_AT, sum);
        out
    }
}

/// Decode a copy and return it only if every rule of 2.6 holds for a scope of
/// `scope_sectors` sectors. This is the mount validator and the pre-write
/// validator of 2.7 step 4.
pub fn decode(bytes: &[u8; COPY], scope_sectors: u64) -> Option<Catalog> {
    if bytes[..8] != MAGIC
        || u16::from_le_bytes([bytes[8], bytes[9]]) != VERSION
        || fnv64(FNV_OFFSET, &bytes[..SUM_AT]) != get64(bytes, SUM_AT)
        || bytes[11..16] != [0; 5]
        || bytes[RECORDS_END..SUM_AT].iter().any(|byte| *byte != 0)
    {
        return None;
    }
    let count = usize::from(bytes[10]);
    let mut catalog = Catalog {
        count,
        id: [0; ID],
        sequence: get64(bytes, 32),
        next_entry: get64(bytes, 40),
        next_free: get64(bytes, 48),
        formatted: get64(bytes, 56),
        records: [Record::EMPTY; MAX_ENTRIES],
    };
    catalog.id.copy_from_slice(&bytes[16..32]);
    if count > MAX_ENTRIES
        || catalog.sequence == 0
        || catalog.next_entry == 0
        || catalog.formatted != scope_sectors
        || catalog.next_free < DATA_START
        || catalog.next_free > catalog.formatted
    {
        return None;
    }
    let mut end = DATA_START;
    let mut previous = 0;
    let mut previous_sealed = 0;
    for index in 0..MAX_ENTRIES {
        let raw = &bytes[RECORDS_AT + index * RECORD..RECORDS_AT + (index + 1) * RECORD];
        if index >= count {
            if raw.iter().any(|byte| *byte != 0) {
                return None;
            }
            continue;
        }
        let record = Record::decode(raw);
        let label_length = usize::from(record.label_length);
        // Entry IDs are never 0 (2.3) and strictly ascending; extents are
        // contiguous from sector 16, which also makes first sectors ascend.
        // Sealing sequences: FORMAT commits sequence 1 without records and
        // each SEAL adds one record at the new sequence, so they are >= 2,
        // <= the copy's sequence and strictly ascending (revision 3).
        if record.id <= previous
            || record.sealed_sequence < 2
            || record.sealed_sequence > catalog.sequence
            || record.sealed_sequence <= previous_sealed
            || record.id >= catalog.next_entry
            || record.first != end
            || record.length < MIN_ARCHIVE
            || record.length > MAX_ARCHIVE
            || label_length == 0
            || label_length > LABEL
            || raw[41..48].iter().any(|byte| *byte != 0)
            || record.label[label_length..].iter().any(|byte| *byte != 0)
        {
            return None;
        }
        end = record.end();
        previous = record.id;
        previous_sealed = record.sealed_sequence;
        catalog.records[index] = record;
    }
    (catalog.next_free == end).then_some(catalog)
}

/// The higher-sequence copy `higher` extends `lower` (2.6 mount rule).
pub fn extends(lower: &Catalog, higher: &Catalog) -> bool {
    higher.id == lower.id
        && higher.formatted == lower.formatted
        && higher.count >= lower.count
        && higher.records[..lower.count] == lower.records[..lower.count]
        && higher.next_free >= lower.next_free
        && higher.next_entry >= lower.next_entry
}

/// A mounted catalog with what step 4 of 2.7 needs to pick its target slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub catalog: Catalog,
    /// Copy A (index 0) and copy B (index 1) passed the validator.
    pub valid: [bool; 2],
    pub sequence: [u64; 2],
}

impl Selection {
    /// Both copies identical after FORMAT or a completed commit.
    pub fn committed(catalog: Catalog) -> Self {
        Self {
            catalog,
            valid: [true; 2],
            sequence: [catalog.sequence; 2],
        }
    }

    /// Step 4: the invalid slot, otherwise the lower sequence, otherwise A.
    pub fn target(&self) -> usize {
        match self.valid {
            [false, _] => 0,
            [_, false] => 1,
            _ if self.sequence[1] < self.sequence[0] => 1,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// No allocator: the selection is returned by value once per mount.
#[allow(clippy::large_enum_variant)]
pub enum Mount {
    Mounted(Selection),
    Blank,
    Corrupt,
}

/// Mount rule over sectors 0-15, all of which must have been read
/// successfully: a read error never reaches this function (2.6).
pub fn mount_rule(area: &[[u8; SECTOR]; AREA_SECTORS], scope_sectors: u64) -> Mount {
    let mut copies = [[0; COPY]; 2];
    for (slot, copy) in copies.iter_mut().enumerate() {
        for sector in 0..COPY_SECTORS as usize {
            copy[sector * SECTOR..(sector + 1) * SECTOR]
                .copy_from_slice(&area[slot * COPY_SECTORS as usize + sector]);
        }
    }
    let a = decode(&copies[0], scope_sectors);
    let b = decode(&copies[1], scope_sectors);
    let selection = |catalog: Catalog| Selection {
        catalog,
        valid: [a.is_some(), b.is_some()],
        sequence: [
            a.map_or(0, |copy| copy.sequence),
            b.map_or(0, |copy| copy.sequence),
        ],
    };
    match (a, b) {
        (Some(only), None) | (None, Some(only)) => Mount::Mounted(selection(only)),
        (Some(a_copy), Some(b_copy)) => {
            if copies[0] == copies[1] {
                return Mount::Mounted(selection(a_copy));
            }
            let (lower, higher) = match a_copy.sequence.cmp(&b_copy.sequence) {
                core::cmp::Ordering::Less => (a_copy, b_copy),
                core::cmp::Ordering::Greater => (b_copy, a_copy),
                core::cmp::Ordering::Equal => return Mount::Corrupt,
            };
            if extends(&lower, &higher) {
                Mount::Mounted(selection(higher))
            } else {
                Mount::Corrupt
            }
        }
        (None, None) => {
            if area
                .iter()
                .all(|sector| sector.iter().all(|byte| *byte == 0))
            {
                Mount::Blank
            } else {
                Mount::Corrupt
            }
        }
    }
}
