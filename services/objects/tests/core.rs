use object_core::journal::{self, Error, Recovery, State, SECTOR, SLOTS};
use object_core::wire::{self, DELETE, PUT};

fn append(
    state: &mut State,
    records: &mut [[u8; SECTOR]; SLOTS],
    object: u64,
    op: u16,
    data: &[u8],
) {
    let revision = state.get(object).unwrap().revision;
    let prepared = state
        .prepare(object, revision, state.sequence() + 1, op, data)
        .unwrap();
    records[state.sequence() as usize] = *prepared.record();
    state.commit(prepared).unwrap();
}
fn recover(records: &[[u8; SECTOR]; SLOTS]) -> Result<State, Error> {
    let mut recovery = Recovery::new();
    for (index, record) in records.iter().enumerate() {
        recovery.push(index, record)?;
    }
    recovery.finish()
}
fn reseal(record: &mut [u8; SECTOR]) {
    record[504..].fill(0);
    let sum = wire::checksum_update(wire::FNV_OFFSET, record);
    wire::put64(record, 504, sum);
}

#[test]
fn four_object_values_and_revisions_survive_complete_streaming_scan() {
    let mut state = State::new();
    let mut records = [[0; SECTOR]; SLOTS];
    for object in 0..4 {
        append(&mut state, &mut records, object, PUT, &[object as u8; 128]);
    }
    append(&mut state, &mut records, 1, PUT, b"changed");
    assert_eq!(state.sequence(), 5);
    assert_eq!(recover(&records), Ok(state));
    for object in 0..4 {
        let actual = state.get(object).unwrap();
        assert_eq!(actual.revision, if object == 1 { 2 } else { 1 });
        assert!(actual.present);
        if object != 1 {
            assert_eq!(actual.value, [object as u8; 128]);
        }
    }
}
#[test]
fn empty_value_is_a_real_object_distinct_from_absent_and_tombstone() {
    let mut state = State::new();
    let mut records = [[0; SECTOR]; SLOTS];
    assert!(!state.get(0).unwrap().present);
    append(&mut state, &mut records, 0, PUT, &[]);
    let empty = state.get(0).unwrap();
    assert!(empty.present);
    assert_eq!(empty.length, 0);
    assert_eq!(empty.revision, 1);
    append(&mut state, &mut records, 0, DELETE, &[]);
    let deleted = state.get(0).unwrap();
    assert!(!deleted.present);
    assert_eq!(deleted.revision, 2);
    assert_eq!(deleted.value, [0; 128]);
    assert_eq!(recover(&records), Ok(state));
}
#[test]
fn tombstone_revision_prevents_replay_and_allows_explicit_cas_recreation() {
    let mut state = State::new();
    let mut records = [[0; SECTOR]; SLOTS];
    append(&mut state, &mut records, 0, PUT, b"original");
    append(&mut state, &mut records, 0, DELETE, &[]);
    assert_eq!(
        state.prepare(0, 0, 1, PUT, b"old request"),
        Err(Error::Stale)
    );
    assert_eq!(state.prepare(0, 1, 2, DELETE, &[]), Err(Error::Stale));
    assert_eq!(state.prepare(0, 2, 3, DELETE, &[]), Err(Error::NotFound));
    append(&mut state, &mut records, 0, PUT, b"recreated");
    assert_eq!(state.get(0).unwrap().revision, 3);
    assert_eq!(recover(&records), Ok(state));
}
#[test]
fn prepared_mutation_does_not_change_live_state_and_cannot_commit_twice() {
    let mut state = State::new();
    let before = state;
    let prepared = state.prepare(0, 0, 1, PUT, b"new").unwrap();
    assert_eq!(state, before);
    let duplicate = prepared.clone();
    state.commit(prepared).unwrap();
    let after = state;
    assert_eq!(state.commit(duplicate), Err(Error::Stale));
    assert_eq!(state, after);
}
#[test]
fn prepared_mutation_cannot_be_committed_to_a_different_equal_sequence_state() {
    let mut a = State::new();
    let mut b = State::new();
    let a1 = a.prepare(0, 0, 1, PUT, b"a").unwrap();
    a.commit(a1).unwrap();
    let b1 = b.prepare(0, 0, 1, PUT, b"b").unwrap();
    b.commit(b1).unwrap();
    let a2 = a.prepare(0, 1, 2, PUT, b"c").unwrap();
    let before = b;
    assert_eq!(b.commit(a2), Err(Error::Stale));
    assert_eq!(b, before);
}
#[test]
fn full_log_never_overwrites_an_old_record_or_wraps_revisions() {
    let mut state = State::new();
    let mut records = [[0; SECTOR]; SLOTS];
    for seq in 0..SLOTS {
        append(&mut state, &mut records, 0, PUT, &[seq as u8]);
    }
    let before = state;
    let original_records = records;
    assert_eq!(state.prepare(0, 32, 33, PUT, b"overflow"), Err(Error::Full));
    assert_eq!(state.prepare(0, 32, 33, DELETE, &[]), Err(Error::Full));
    assert_eq!(state, before);
    assert_eq!(records, original_records);
    assert_eq!(recover(&records), Ok(state));
}
#[test]
fn malformed_commands_and_stale_replays_leave_every_object_unchanged() {
    let state = State::new();
    let before = state;
    assert_eq!(state.prepare(4, 0, 1, PUT, b"x"), Err(Error::Range));
    assert_eq!(state.prepare(u64::MAX, 0, 1, PUT, b"x"), Err(Error::Range));
    assert_eq!(state.prepare(0, u64::MAX, 1, PUT, b"x"), Err(Error::Stale));
    assert_eq!(state.prepare(0, 0, 0, PUT, b"x"), Err(Error::Invalid));
    assert_eq!(state.prepare(0, 0, 1, 99, b"x"), Err(Error::Invalid));
    assert_eq!(state.prepare(0, 0, 1, PUT, &[0; 129]), Err(Error::Range));
    assert_eq!(state.prepare(0, 0, 1, DELETE, b"x"), Err(Error::Range));
    assert_eq!(state.prepare(0, 0, 1, DELETE, &[]), Err(Error::NotFound));
    assert_eq!(state, before);
}
#[test]
fn every_single_byte_corruption_in_an_occupied_record_is_rejected() {
    let record = *State::new()
        .prepare(0, 0, 1, PUT, b"document")
        .unwrap()
        .record();
    for index in 0..SECTOR {
        let mut bad = record;
        bad[index] ^= 1;
        assert_eq!(
            journal::decode_record(&bad, 0),
            Err(Error::Corrupt),
            "byte{index}"
        );
    }
}
#[test]
fn valid_checksum_never_replaces_structure_sequence_or_revision_validation() {
    let record = *State::new()
        .prepare(0, 0, 1, PUT, b"document")
        .unwrap()
        .record();
    for index in [
        0, 8, 9, 10, 11, 12, 15, 16, 32, 40, 48, 49, 50, 51, 52, 63, 191, 192, 503,
    ] {
        let mut bad = record;
        bad[index] ^= 0x80;
        reseal(&mut bad);
        assert_eq!(
            journal::decode_record(&bad, 0),
            Err(Error::Corrupt),
            "field{index}"
        );
    }
    let mut no_request = record;
    wire::put64(&mut no_request, 24, 0);
    reseal(&mut no_request);
    assert_eq!(journal::decode_record(&no_request, 0), Err(Error::Corrupt));
}
#[test]
fn gaps_duplicate_slots_wrong_order_and_incomplete_scan_fail_closed() {
    let mut state = State::new();
    let mut records = [[0; SECTOR]; SLOTS];
    append(&mut state, &mut records, 0, PUT, b"first");
    append(&mut state, &mut records, 0, PUT, b"second");
    let mut gap = records;
    gap[0] = [0; SECTOR];
    assert_eq!(recover(&gap), Err(Error::Corrupt));
    let mut reordered = records;
    reordered.swap(0, 1);
    assert_eq!(recover(&reordered), Err(Error::Corrupt));
    let mut recovery = Recovery::new();
    recovery.push(0, &records[0]).unwrap();
    assert_eq!(recovery.push(0, &records[0]), Err(Error::Corrupt));
    assert_eq!(recovery.finish(), Err(Error::Corrupt));
    assert_eq!(Recovery::new().finish(), Err(Error::Corrupt));
}
#[test]
fn malformed_tail_poison_is_sticky_and_never_guesses_an_older_success() {
    let mut state = State::new();
    let mut records = [[0; SECTOR]; SLOTS];
    append(&mut state, &mut records, 0, PUT, b"first");
    let next = state.prepare(0, 1, 2, PUT, b"second").unwrap();
    records[1][..256].copy_from_slice(&next.record()[..256]);
    assert_eq!(recover(&records), Err(Error::Corrupt));
    let mut recovery = Recovery::new();
    recovery.push(0, &records[0]).unwrap();
    assert_eq!(recovery.push(1, &records[1]), Err(Error::Corrupt));
    assert_eq!(recovery.push(1, &[0; SECTOR]), Err(Error::Corrupt));
    assert_eq!(recovery.finish(), Err(Error::Corrupt));
}
#[test]
fn resealed_revision_skip_and_delete_of_never_created_object_are_rejected() {
    let mut state = State::new();
    let mut records = [[0; SECTOR]; SLOTS];
    append(&mut state, &mut records, 0, PUT, b"first");
    append(&mut state, &mut records, 0, DELETE, &[]);
    let mut wrong_revision = records;
    wire::put64(&mut wrong_revision[1], 40, 1);
    reseal(&mut wrong_revision[1]);
    assert_eq!(recover(&wrong_revision), Err(Error::Corrupt));
    let mut no_predecessor = records;
    wire::put64(&mut no_predecessor[1], 32, 1);
    wire::put64(&mut no_predecessor[1], 40, 1);
    reseal(&mut no_predecessor[1]);
    assert_eq!(recover(&no_predecessor), Err(Error::Corrupt));
}
#[test]
fn zero_volume_is_valid_empty_state_but_out_of_range_scan_is_invalid() {
    assert_eq!(recover(&[[0; SECTOR]; SLOTS]), Ok(State::new()));
    assert_eq!(
        journal::decode_record(&[0; SECTOR], SLOTS),
        Err(Error::Corrupt)
    );
}
