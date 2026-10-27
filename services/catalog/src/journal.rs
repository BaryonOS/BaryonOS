//! Four named objects and 32 immutable append records, without an allocator.
//! FNV detects accidental corruption only; it is not authentication.
use crate::wire::{
    checksum_update, get64, put64, valid_name, Catalog, CatalogEntry, DELETE, FNV_OFFSET, PUT,
    RENAME,
};

pub const SECTOR: usize = 512;
pub const SLOTS: usize = 32;
pub const OBJECTS: usize = 4;
pub const VALUE: usize = 128;
pub const NAME: usize = 32;
pub const MAGIC: &[u8; 8] = b"NOSM6C02";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Range,
    Stale,
    Full,
    Corrupt,
    NotFound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Object {
    pub revision: u64,
    pub present: bool,
    pub length: usize,
    pub value: [u8; VALUE],
    pub name_length: usize,
    pub name: [u8; NAME],
}
impl Object {
    pub const EMPTY: Self = Self {
        revision: 0,
        present: false,
        length: 0,
        value: [0; VALUE],
        name_length: 0,
        name: [0; NAME],
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    sequence: u64,
    objects: [Object; OBJECTS],
}
impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
impl State {
    pub const fn new() -> Self {
        Self {
            sequence: 0,
            objects: [Object::EMPTY; OBJECTS],
        }
    }
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    pub fn get(&self, object: u64) -> Result<Object, Error> {
        if object >= OBJECTS as u64 {
            return Err(Error::Range);
        }
        Ok(self.objects[object as usize])
    }
    /// The adapter passes the kernel-bound endpoint domain, never an app field.
    /// Only local revisions (including tombstones) contribute to generation.
    pub fn catalog(&self, domain: usize) -> Result<Catalog, Error> {
        if domain >= 2 {
            return Err(Error::Range);
        }
        let mut catalog = Catalog::empty();
        for id in domain * 2..domain * 2 + 2 {
            let object = self.objects[id];
            catalog.generation += object.revision;
            if object.present {
                catalog.entries[catalog.count] = CatalogEntry {
                    object: id as u64,
                    revision: object.revision,
                    length: object.length,
                    name_length: object.name_length,
                    name: object.name,
                };
                catalog.count += 1;
            }
        }
        Ok(catalog)
    }

    /// Pure preparation. The I/O adapter must complete WRITE and FLUSH before
    /// commit/ACK. RENAME has no supplied value: existing bytes stay identical.
    pub fn prepare(
        &self,
        object: u64,
        expected_revision: u64,
        request_id: u64,
        operation: u16,
        name: &[u8],
        value: &[u8],
    ) -> Result<Prepared, Error> {
        if request_id == 0 || !matches!(operation, PUT | DELETE | RENAME) {
            return Err(Error::Invalid);
        }
        if value.len() > VALUE || name.len() > NAME {
            return Err(Error::Range);
        }
        match operation {
            PUT if valid_name(name) => {}
            RENAME if valid_name(name) && value.is_empty() => {}
            DELETE if name.is_empty() && value.is_empty() => {}
            _ => return Err(Error::Invalid),
        }
        let current = self.get(object)?;
        if current.revision != expected_revision {
            return Err(Error::Stale);
        }
        if matches!(operation, DELETE | RENAME) && !current.present {
            return Err(Error::NotFound);
        }
        if self.sequence >= SLOTS as u64 {
            return Err(Error::Full);
        }
        let sequence = self.sequence.checked_add(1).ok_or(Error::Full)?;
        let revision = current.revision.checked_add(1).ok_or(Error::Full)?;
        let mut next = *self;
        let mut entry = Object {
            revision,
            ..Object::EMPTY
        };
        if matches!(operation, PUT | RENAME) {
            entry.present = true;
            entry.name_length = name.len();
            entry.name[..name.len()].copy_from_slice(name);
            if operation == RENAME {
                entry.length = current.length;
                entry.value = current.value;
            } else {
                entry.length = value.len();
                entry.value[..value.len()].copy_from_slice(value);
            }
        }
        next.objects[object as usize] = entry;
        next.sequence = sequence;
        let mut record = [0; SECTOR];
        record[..8].copy_from_slice(MAGIC);
        record[8..10].copy_from_slice(&2_u16.to_le_bytes());
        record[10..12].copy_from_slice(&64_u16.to_le_bytes());
        put64(&mut record, 16, sequence);
        put64(&mut record, 24, request_id);
        put64(&mut record, 32, object);
        put64(&mut record, 40, revision);
        record[48..50].copy_from_slice(&operation.to_le_bytes());
        record[50..52].copy_from_slice(&(entry.length as u16).to_le_bytes());
        record[52..54].copy_from_slice(&(entry.name_length as u16).to_le_bytes());
        record[64..192].copy_from_slice(&entry.value);
        record[192..224].copy_from_slice(&entry.name);
        let sum = checksum(&record);
        put64(&mut record, 504, sum);
        Ok(Prepared {
            before: *self,
            after: next,
            record,
        })
    }

    /// Pure state update; this function cannot observe durability. The native
    /// adapter alone sequences it after successful WRITE and FLUSH completions.
    pub fn commit(&mut self, prepared: Prepared) -> Result<(), Error> {
        if *self != prepared.before {
            return Err(Error::Stale);
        }
        *self = prepared.after;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prepared {
    before: State,
    after: State,
    record: [u8; SECTOR],
}
impl Prepared {
    pub fn record(&self) -> &[u8; SECTOR] {
        &self.record
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    pub sequence: u64,
    pub request_id: u64,
    pub object: u64,
    pub revision: u64,
    pub operation: u16,
    pub length: usize,
    pub value: [u8; VALUE],
    pub name_length: usize,
    pub name: [u8; NAME],
}

fn checksum(record: &[u8; SECTOR]) -> u64 {
    let value = checksum_update(FNV_OFFSET, &record[..504]);
    checksum_update(value, &[0; 8])
}

/// Strict v2 decoder: old v1 occupied records are rejected, never reformatted
/// or interpreted as empty. All reserved and unused bytes must be canonical.
pub fn decode_record(record: &[u8; SECTOR], index: usize) -> Result<Option<Record>, Error> {
    if index >= SLOTS {
        return Err(Error::Corrupt);
    }
    if record.iter().all(|byte| *byte == 0) {
        return Ok(None);
    }
    let sequence = get64(record, 16);
    let request_id = get64(record, 24);
    let object = get64(record, 32);
    let revision = get64(record, 40);
    let operation = u16::from_le_bytes([record[48], record[49]]);
    let length = usize::from(u16::from_le_bytes([record[50], record[51]]));
    let name_length = usize::from(u16::from_le_bytes([record[52], record[53]]));
    if &record[..8] != MAGIC
        || record[8..16] != [2, 0, 64, 0, 0, 0, 0, 0]
        || sequence != index as u64 + 1
        || request_id == 0
        || object >= OBJECTS as u64
        || revision == 0
        || revision > sequence
        || !matches!(operation, PUT | DELETE | RENAME)
        || length > VALUE
        || name_length > NAME
        || (operation == DELETE && (length != 0 || name_length != 0))
        || (operation != DELETE && !valid_name(&record[192..192 + name_length]))
        || record[54..64]
            .iter()
            .chain(record[224..504].iter())
            .any(|byte| *byte != 0)
        || record[64 + length..192].iter().any(|byte| *byte != 0)
        || record[192 + name_length..224].iter().any(|byte| *byte != 0)
        || get64(record, 504) != checksum(record)
    {
        return Err(Error::Corrupt);
    }
    let mut value = [0; VALUE];
    value.copy_from_slice(&record[64..192]);
    let mut name = [0; NAME];
    name.copy_from_slice(&record[192..224]);
    Ok(Some(Record {
        sequence,
        request_id,
        object,
        revision,
        operation,
        length,
        value,
        name_length,
        name,
    }))
}

/// Sticky fail-closed scan. Invalid occupied records, skipped scan indices or
/// gaps invalidate recovery; only a complete scan may expose the resulting state.
pub struct Recovery {
    state: State,
    next_index: usize,
    unused: bool,
    failed: bool,
}
impl Default for Recovery {
    fn default() -> Self {
        Self::new()
    }
}
impl Recovery {
    pub const fn new() -> Self {
        Self {
            state: State::new(),
            next_index: 0,
            unused: false,
            failed: false,
        }
    }
    pub fn push(&mut self, index: usize, record: &[u8; SECTOR]) -> Result<(), Error> {
        let result = self.ingest(index, record);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn ingest(&mut self, index: usize, record: &[u8; SECTOR]) -> Result<(), Error> {
        if self.failed || index != self.next_index || index >= SLOTS {
            return Err(Error::Corrupt);
        }
        match decode_record(record, index)? {
            None => self.unused = true,
            Some(decoded) => {
                if self.unused || decoded.sequence != self.state.sequence + 1 {
                    return Err(Error::Corrupt);
                }
                let object = self.state.get(decoded.object).map_err(|_| Error::Corrupt)?;
                if object.revision.checked_add(1) != Some(decoded.revision)
                    || (matches!(decoded.operation, DELETE | RENAME) && !object.present)
                    || (decoded.operation == RENAME
                        && (decoded.length != object.length || decoded.value != object.value))
                {
                    return Err(Error::Corrupt);
                }
                self.state.objects[decoded.object as usize] = Object {
                    revision: decoded.revision,
                    present: decoded.operation != DELETE,
                    length: decoded.length,
                    value: decoded.value,
                    name_length: decoded.name_length,
                    name: decoded.name,
                };
                self.state.sequence = decoded.sequence;
            }
        }
        self.next_index += 1;
        Ok(())
    }
    pub fn finish(self) -> Result<State, Error> {
        if self.failed || self.next_index != SLOTS {
            Err(Error::Corrupt)
        } else {
            Ok(self.state)
        }
    }
}
