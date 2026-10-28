//! Four independent object slots, 32 immutable append records, no allocator/GC.
//! A checksum detects the exercised corruptions; it is not authentication.
use crate::wire::{checksum_update, get64, put64, DELETE, FNV_OFFSET, PUT};

pub const SECTOR: usize = 512;
pub const SLOTS: usize = 32;
pub const OBJECTS: usize = 4;
pub const VALUE: usize = 128;
pub const MAGIC: &[u8; 8] = b"NOSM6B01";

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
}
impl Object {
    const EMPTY: Self = Self {
        revision: 0,
        present: false,
        length: 0,
        value: [0; VALUE],
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
    /// Preparation is pure. Caller must complete WRITE then FLUSH before commit
    /// or ACK. request_id correlates replies; CAS prevents replay of old writes.
    pub fn prepare(
        &self,
        object: u64,
        expected_revision: u64,
        request_id: u64,
        operation: u16,
        value: &[u8],
    ) -> Result<Prepared, Error> {
        if request_id == 0 || !matches!(operation, PUT | DELETE) {
            return Err(Error::Invalid);
        }
        if value.len() > VALUE || (operation == DELETE && !value.is_empty()) {
            return Err(Error::Range);
        }
        let current = self.get(object)?;
        if current.revision != expected_revision {
            return Err(Error::Stale);
        }
        if operation == DELETE && !current.present {
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
        if operation == PUT {
            entry.present = true;
            entry.length = value.len();
            entry.value[..value.len()].copy_from_slice(value);
        }
        next.objects[object as usize] = entry;
        next.sequence = sequence;
        let mut record = [0; SECTOR];
        record[..8].copy_from_slice(MAGIC);
        record[8..10].copy_from_slice(&1_u16.to_le_bytes());
        record[10..12].copy_from_slice(&64_u16.to_le_bytes());
        put64(&mut record, 16, sequence);
        put64(&mut record, 24, request_id);
        put64(&mut record, 32, object);
        put64(&mut record, 40, revision);
        record[48..50].copy_from_slice(&operation.to_le_bytes());
        record[50..52].copy_from_slice(&(entry.length as u16).to_le_bytes());
        record[64..192].copy_from_slice(&entry.value);
        let sum = checksum(&record);
        put64(&mut record, 504, sum);
        Ok(Prepared {
            before: *self,
            after: next,
            record,
        })
    }
    /// This method cannot observe disk durability. Only the ring-3 I/O adapter
    /// calls it after both completions. A prepared record belongs to one state.
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
}

fn checksum(record: &[u8; SECTOR]) -> u64 {
    let value = checksum_update(FNV_OFFSET, &record[..504]);
    checksum_update(value, &[0; 8])
}

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
    if &record[..8] != MAGIC
        || record[8..16] != [1, 0, 64, 0, 0, 0, 0, 0]
        || sequence != index as u64 + 1
        || request_id == 0
        || object >= OBJECTS as u64
        || revision == 0
        || revision > sequence
        || !matches!(operation, PUT | DELETE)
        || length > VALUE
        || (operation == DELETE && length != 0)
        || record[52..64]
            .iter()
            .chain(record[192..504].iter())
            .any(|byte| *byte != 0)
        || record[64 + length..192].iter().any(|byte| *byte != 0)
        || get64(record, 504) != checksum(record)
    {
        return Err(Error::Corrupt);
    }
    let mut value = [0; VALUE];
    value.copy_from_slice(&record[64..192]);
    Ok(Some(Record {
        sequence,
        request_id,
        object,
        revision,
        operation,
        length,
        value,
    }))
}

/// Sticky fail-closed, strictly sequential scan of every journal sector.
/// Any malformed occupied record or hole before later data invalidates recovery.
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
            None => {
                self.unused = true;
            }
            Some(decoded) => {
                if self.unused || decoded.sequence != self.state.sequence + 1 {
                    return Err(Error::Corrupt);
                }
                let object = self.state.get(decoded.object).map_err(|_| Error::Corrupt)?;
                if object.revision.checked_add(1) != Some(decoded.revision)
                    || (decoded.operation == DELETE && !object.present)
                {
                    return Err(Error::Corrupt);
                }
                self.state.objects[decoded.object as usize] = Object {
                    revision: decoded.revision,
                    present: decoded.operation == PUT,
                    length: decoded.length,
                    value: decoded.value,
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
