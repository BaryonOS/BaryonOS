//! Codec for one catalog v2 copy (S09-CATALOG-V2 §§5.1–5.2).

use super::{decode_header, decode_record, Catalog, COPY, RECORDS_END, SUM_AT};
use crate::{fnv64, get64, put64, DATA_START, FNV_OFFSET, ID, MAX_ENTRIES};

const VERSION: u16 = 2;
const METADATA: usize = 24;
const OWNERS_AT: usize = RECORDS_END + MAX_ENTRIES * METADATA;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Attributes {
    pub backup_time: i64,
    pub verified_sequence: u64,
    pub time_source: u8,
    pub deleting: bool,
    /// Opaque equality key. All-zero identifies an unowned entry.
    pub owner: [u8; ID],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CopyV2 {
    pub catalog: Catalog,
    pub attributes: [Attributes; MAX_ENTRIES],
}

impl CopyV2 {
    /// Encode only canonical copies, using the same validator as the reader.
    pub fn encode(&self, scope_sectors: u64) -> Option<[u8; COPY]> {
        if self.catalog.count > MAX_ENTRIES {
            return None;
        }
        let mut bytes = self.catalog.encode();
        bytes[8..10].copy_from_slice(&VERSION.to_le_bytes());
        for (index, entry) in self.attributes[..self.catalog.count].iter().enumerate() {
            let metadata = RECORDS_END + index * METADATA;
            let owner = OWNERS_AT + index * ID;
            bytes[metadata..metadata + 8].copy_from_slice(&entry.backup_time.to_le_bytes());
            put64(&mut bytes, metadata + 8, entry.verified_sequence);
            bytes[metadata + 16] = entry.time_source;
            bytes[metadata + 17] = u8::from(entry.deleting);
            bytes[owner..owner + ID].copy_from_slice(&entry.owner);
        }
        let sum = fnv64(FNV_OFFSET, &bytes[..SUM_AT]);
        put64(&mut bytes, SUM_AT, sum);
        (decode(&bytes, scope_sectors).as_ref() == Some(self)).then_some(bytes)
    }
}

/// Validate a single v2 copy against the actual device scope.
pub fn decode(bytes: &[u8; COPY], scope_sectors: u64) -> Option<CopyV2> {
    let mut catalog = decode_header(bytes, scope_sectors, VERSION)?;
    let count = catalog.count;
    if bytes[RECORDS_END + count * METADATA..OWNERS_AT]
        .iter()
        .chain(&bytes[OWNERS_AT + count * ID..SUM_AT])
        .any(|byte| *byte != 0)
    {
        return None;
    }
    let mut attributes = [Attributes::default(); MAX_ENTRIES];
    let mut next_free = DATA_START;
    let mut previous = 0;
    let mut previous_sealed = 0;
    let mut has_deleting = false;
    for (index, entry) in attributes[..count].iter_mut().enumerate() {
        let record = decode_record(bytes, index, &catalog, previous, previous_sealed)?;
        let end = record
            .first
            .checked_add(crate::sectors_for(record.length))?;
        if record.first < DATA_START
            || end > catalog.formatted
            || catalog.records[..index]
                .iter()
                .any(|other| record.first < other.end() && other.first < end)
        {
            return None;
        }
        let metadata = RECORDS_END + index * METADATA;
        let owner = OWNERS_AT + index * ID;
        let backup_time = get64(bytes, metadata) as i64;
        let verified_sequence = get64(bytes, metadata + 8);
        let time_source = bytes[metadata + 16];
        let state = bytes[metadata + 17];
        if backup_time < 0
            || time_source > 2
            || (time_source == 0) != (backup_time == 0)
            || state > 1
            || (state == 1 && has_deleting)
            || (verified_sequence != 0
                && !(record.sealed_sequence..=catalog.sequence).contains(&verified_sequence))
            || bytes[metadata + 18..metadata + METADATA] != [0; 6]
        {
            return None;
        }
        has_deleting |= state == 1;
        *entry = Attributes {
            backup_time,
            verified_sequence,
            time_source,
            deleting: state == 1,
            owner: bytes[owner..owner + ID].try_into().ok()?,
        };
        next_free = next_free.max(end);
        previous = record.id;
        previous_sealed = record.sealed_sequence;
        catalog.records[index] = record;
    }
    (catalog.next_free == next_free).then_some(CopyV2 {
        catalog,
        attributes,
    })
}
