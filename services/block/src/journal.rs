//! Four immutable-slot synthetic journal. FNV detects tested corruption only;
//! it provides neither authentication nor a general torn-write guarantee.
use crate::wire::{checksum_update, get32, get64, put32, put64, FNV_OFFSET};
pub const SECTOR: usize = 512;
pub const SLOTS: usize = 4;
pub const VALUE: usize = 128;
pub const MAGIC: &[u8; 8] = b"NOSM6A01";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub generation: u64,
    pub request_id: u64,
    pub value: [u8; VALUE],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Corrupt,
    Stale,
    Full,
}
fn checksum(record: &[u8; SECTOR]) -> u64 {
    let mut copy = *record;
    copy[504..].fill(0);
    checksum_update(FNV_OFFSET, &copy)
}
pub fn encode(slot: usize, request_id: u64, value: &[u8; VALUE]) -> Result<[u8; SECTOR], Error> {
    if slot >= SLOTS || request_id == 0 {
        return Err(Error::Corrupt);
    }
    let mut record = [0; SECTOR];
    record[..8].copy_from_slice(MAGIC);
    record[8] = 1;
    record[10] = 64;
    put64(&mut record, 16, slot as u64 + 1);
    put64(&mut record, 24, request_id);
    put32(&mut record, 32, VALUE as u32);
    put32(&mut record, 36, slot as u32);
    record[64..192].copy_from_slice(value);
    let sum = checksum(&record);
    put64(&mut record, 504, sum);
    Ok(record)
}
pub fn decode(record: &[u8; SECTOR], slot: usize) -> Result<State, Error> {
    if slot >= SLOTS
        || &record[..8] != MAGIC
        || record[8..12] != [1, 0, 64, 0]
        || get32(record, 12) != 0
        || get64(record, 16) != slot as u64 + 1
        || get64(record, 24) == 0
        || get32(record, 32) != VALUE as u32
        || get32(record, 36) != slot as u32
        || record[40..64].iter().any(|b| *b != 0)
        || record[192..504].iter().any(|b| *b != 0)
        || get64(record, 504) != checksum(record)
    {
        return Err(Error::Corrupt);
    }
    let mut value = [0; VALUE];
    value.copy_from_slice(&record[64..192]);
    Ok(State {
        generation: slot as u64 + 1,
        request_id: get64(record, 24),
        value,
    })
}
pub fn recover(records: &[[u8; SECTOR]; SLOTS]) -> Result<State, Error> {
    let mut state = None;
    for (slot, record) in records.iter().enumerate() {
        if let Ok(valid) = decode(record, slot) {
            state = Some(valid);
        }
    }
    state.ok_or(Error::Corrupt)
}
pub fn prepare(
    current: &State,
    expected: u64,
    id: u64,
    value: &[u8; VALUE],
) -> Result<[u8; SECTOR], Error> {
    if current.generation == 0 || current.generation > SLOTS as u64 {
        return Err(Error::Corrupt);
    }
    if expected != current.generation {
        return Err(Error::Stale);
    }
    if current.generation == SLOTS as u64 {
        return Err(Error::Full);
    }
    encode(current.generation as usize, id, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn value() -> [u8; VALUE] {
        core::array::from_fn(|i| (i as u8).wrapping_mul(17))
    }
    fn sealed(mut record: [u8; SECTOR]) -> [u8; SECTOR] {
        let sum = checksum(&record);
        put64(&mut record, 504, sum);
        record
    }
    #[test]
    fn exact_format_and_roundtrip() {
        let r = encode(2, 7, &value()).unwrap();
        assert_eq!(&r[..8], MAGIC);
        assert_eq!(get64(&r, 16), 3);
        assert_eq!(get64(&r, 24), 7);
        assert_eq!(get32(&r, 36), 2);
        assert!(r[40..64].iter().chain(r[192..504].iter()).all(|b| *b == 0));
        assert_eq!(
            decode(&r, 2),
            Ok(State {
                generation: 3,
                request_id: 7,
                value: value()
            })
        );
    }
    #[test]
    fn every_single_byte_corruption_is_detected_for_this_record() {
        let r = encode(1, 2, &value()).unwrap();
        for i in 0..SECTOR {
            let mut bad = r;
            bad[i] ^= 1;
            assert_eq!(decode(&bad, 1), Err(Error::Corrupt), "byte {i}");
        }
    }
    #[test]
    fn checksum_does_not_replace_structural_validation() {
        let r = encode(1, 2, &value()).unwrap();
        for i in [0, 8, 9, 10, 11, 12, 16, 24, 32, 36, 40, 63, 192, 503] {
            let mut bad = r;
            bad[i] ^= 2;
            assert_eq!(decode(&sealed(bad), 1), Err(Error::Corrupt), "field {i}");
        }
        assert_eq!(decode(&r, 0), Err(Error::Corrupt));
    }
    #[test]
    fn prefix256_tail_recovers_old_while_full_record_recovers_new() {
        let mut records = [[0; SECTOR]; SLOTS];
        records[0] = encode(0, 1, &value()).unwrap();
        let next = encode(1, 2, &[0xa7; VALUE]).unwrap();
        records[1][..256].copy_from_slice(&next[..256]);
        assert_eq!(recover(&records).unwrap().generation, 1);
        records[1] = next;
        assert_eq!(recover(&records).unwrap().value, [0xa7; VALUE]);
    }
    #[test]
    fn no_implicit_format_for_empty_or_corrupt_volume() {
        let mut records = [[0; SECTOR]; SLOTS];
        assert_eq!(recover(&records), Err(Error::Corrupt));
        records[0] = encode(0, 1, &value()).unwrap();
        records[0][64] ^= 1;
        assert_eq!(recover(&records), Err(Error::Corrupt));
    }
    #[test]
    fn cas_full_and_append_preserve_previous_record() {
        let mut records = [[0; SECTOR]; SLOTS];
        records[0] = encode(0, 1, &value()).unwrap();
        let first = records[0];
        for gen in 1..4 {
            let s = recover(&records).unwrap();
            assert_eq!(prepare(&s, 0, 2, &value()), Err(Error::Stale));
            records[gen] = prepare(&s, gen as u64, gen as u64 + 1, &[gen as u8; VALUE]).unwrap();
        }
        assert_eq!(
            prepare(&recover(&records).unwrap(), 4, 5, &value()),
            Err(Error::Full)
        );
        assert_eq!(records[0], first);
        assert_eq!(recover(&records).unwrap().generation, 4);
    }
    #[test]
    fn invalid_tail_can_be_replaced_after_recovery() {
        let mut r = [[0; SECTOR]; SLOTS];
        r[0] = encode(0, 1, &value()).unwrap();
        r[1].fill(0x91);
        let s = recover(&r).unwrap();
        r[1] = prepare(&s, 1, 2, &[4; VALUE]).unwrap();
        assert_eq!(recover(&r).unwrap().generation, 2);
    }
}
