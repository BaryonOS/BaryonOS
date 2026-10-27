use catalog_core::journal::{self, Error, Recovery, State, SECTOR, SLOTS};
use catalog_core::wire::{self, DELETE, PUT, RENAME};

fn apply(state: &mut State, object: u64, op: u16, name: &[u8], value: &[u8]) -> [u8; SECTOR] {
    let prepared = state
        .prepare(
            object,
            state.get(object).unwrap().revision,
            state.sequence() + 1,
            op,
            name,
            value,
        )
        .unwrap();
    let record = *prepared.record();
    state.commit(prepared).unwrap();
    record
}
fn recover(records: &[[u8; SECTOR]]) -> Result<State, Error> {
    let mut reader = Recovery::new();
    for i in 0..SLOTS {
        reader.push(i, records.get(i).unwrap_or(&[0; SECTOR]))?;
    }
    reader.finish()
}
fn resign(record: &mut [u8; SECTOR]) {
    record[504..].fill(0);
    let mut sum = 0xcbf2_9ce4_8422_2325u64;
    for byte in record.iter() {
        sum = (sum ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
    record[504..].copy_from_slice(&sum.to_le_bytes());
}

#[test]
fn empty_catalog_has_no_global_activity_or_invalid_domain() {
    let mut state = State::new();
    assert_eq!(state.catalog(0).unwrap(), wire::Catalog::empty());
    assert_eq!(state.catalog(2), Err(Error::Range));
    assert_eq!(state.catalog(usize::MAX), Err(Error::Range));
    apply(&mut state, 2, PUT, b"private B", b"never shown to A");
    assert_eq!(state.sequence(), 1);
    assert_eq!(state.catalog(0).unwrap(), wire::Catalog::empty());
}

#[test]
fn rename_is_atomic_preserves_all_content_and_has_no_early_commit() {
    let mut state = State::new();
    let value = core::array::from_fn::<_, 128, _>(|i| i as u8);
    let first = apply(&mut state, 0, PUT, "Café".as_bytes(), &value);
    let before = state;
    let prepared = state
        .prepare(0, 1, 2, RENAME, "Café nieuw".as_bytes(), &[])
        .unwrap();
    assert_eq!(state, before);
    let renamed = *prepared.record();
    assert_eq!(&renamed[64..192], &value);
    state.commit(prepared).unwrap();
    let object = state.get(0).unwrap();
    assert_eq!(object.revision, 2);
    assert_eq!(object.value, value);
    assert_eq!(&object.name[..object.name_length], "Café nieuw".as_bytes());
    assert_eq!(recover(&[first]).unwrap(), before);
    assert_eq!(recover(&[first, renamed]).unwrap(), state);
}

#[test]
fn catalog_sorted_filtered_and_tombstone_revision_is_local() {
    let mut state = State::new();
    apply(&mut state, 3, PUT, b"B3", b"secret3");
    apply(&mut state, 1, PUT, b"A1", b"a1");
    apply(&mut state, 0, PUT, b"A0", b"a0");
    apply(&mut state, 2, PUT, b"B2", b"secret2");
    let first = state.catalog(0).unwrap();
    assert_eq!(first.count, 2);
    assert_eq!(first.generation, 2);
    assert_eq!(first.entries.map(|entry| entry.object), [0, 1]);
    assert_eq!(&first.entries[0].name[..2], b"A0");
    apply(&mut state, 1, DELETE, &[], &[]);
    let catalog = state.catalog(0).unwrap();
    assert_eq!((catalog.count, catalog.generation), (1, 3));
    assert_eq!(catalog.entries[1], wire::CatalogEntry::EMPTY);
    assert_eq!(state.catalog(1).unwrap().generation, 2);
    let tombstone = state.get(1).unwrap();
    assert_eq!(
        (
            tombstone.revision,
            tombstone.present,
            tombstone.length,
            tombstone.name_length
        ),
        (2, false, 0, 0)
    );
    assert_eq!(tombstone.value, [0; 128]);
    assert_eq!(tombstone.name, [0; 32]);
}

#[test]
fn duplicate_labels_and_byte_distinct_unicode_keep_id_identity() {
    let mut state = State::new();
    apply(&mut state, 0, PUT, "café".as_bytes(), b"left");
    apply(&mut state, 1, PUT, "café".as_bytes(), b"right");
    assert_eq!(state.catalog(0).unwrap().count, 2);
    apply(&mut state, 1, RENAME, "cafe\u{301}".as_bytes(), &[]);
    assert_ne!(state.get(0).unwrap().name, state.get(1).unwrap().name);
    assert_eq!(&state.get(1).unwrap().value[..5], b"right");
}

#[test]
fn zero_length_content_is_a_live_named_object() {
    let mut state = State::new();
    let a = apply(&mut state, 0, PUT, b"empty", &[]);
    let b = apply(&mut state, 0, RENAME, b"still empty", &[]);
    assert!(state.get(0).unwrap().present);
    assert_eq!(state.catalog(0).unwrap().entries[0].length, 0);
    assert_eq!(recover(&[a, b]).unwrap(), state);
}

#[test]
fn stale_mutation_cannot_resurrect_deleted_object() {
    let mut state = State::new();
    apply(&mut state, 0, PUT, b"original", b"data");
    apply(&mut state, 0, DELETE, &[], &[]);
    let before = state;
    for op in [PUT, RENAME] {
        assert_eq!(state.prepare(0, 1, 4, op, b"old", &[]), Err(Error::Stale));
    }
    assert_eq!(
        state.prepare(0, 2, 4, RENAME, b"new", &[]),
        Err(Error::NotFound)
    );
    assert_eq!(state, before);
    apply(&mut state, 0, PUT, b"recreated", b"new data");
    assert_eq!(state.get(0).unwrap().revision, 3);
}

#[test]
fn state_rejects_bad_operation_id_bounds_name_and_rename_value() {
    let state = State::new();
    assert_eq!(state.prepare(0, 0, 0, PUT, b"x", &[]), Err(Error::Invalid));
    assert_eq!(state.prepare(0, 0, 1, 99, b"x", &[]), Err(Error::Invalid));
    assert_eq!(
        state.prepare(u64::MAX, 0, 1, PUT, b"x", &[]),
        Err(Error::Range)
    );
    assert_eq!(
        state.prepare(0, 0, 1, PUT, &[b'x'; 33], &[]),
        Err(Error::Range)
    );
    assert_eq!(
        state.prepare(0, 0, 1, PUT, b"x", &[0; 129]),
        Err(Error::Range)
    );
    assert_eq!(state.prepare(0, 0, 1, PUT, b"", &[]), Err(Error::Invalid));
    assert_eq!(
        state.prepare(0, 0, 1, PUT, &[0xff], &[]),
        Err(Error::Invalid)
    );
    assert_eq!(
        state.prepare(0, 0, 1, DELETE, b"x", &[]),
        Err(Error::Invalid)
    );
    assert_eq!(
        state.prepare(0, 0, 1, RENAME, b"x", &[0]),
        Err(Error::Invalid)
    );
    assert_eq!(
        state.prepare(0, 0, 1, DELETE, &[], &[]),
        Err(Error::NotFound)
    );
}

#[test]
fn prepared_commit_is_bound_to_complete_prior_state() {
    let mut state = State::new();
    let a = state.prepare(0, 0, 1, PUT, b"a", b"a").unwrap();
    let b = state.prepare(1, 0, 2, PUT, b"b", b"b").unwrap();
    state.commit(a.clone()).unwrap();
    assert_eq!(state.commit(a), Err(Error::Stale));
    assert_eq!(state.commit(b), Err(Error::Stale));
}

#[test]
fn full_preserves_latest_name_and_content_without_wrap() {
    let mut state = State::new();
    let mut records = Vec::new();
    for revision in 1..=32 {
        records.push(apply(&mut state, 0, PUT, b"last name", &[revision]));
    }
    let before = state;
    for op in [PUT, RENAME] {
        assert_eq!(
            state.prepare(0, 32, 33, op, b"must not replace", &[]),
            Err(Error::Full)
        );
    }
    assert_eq!(state.prepare(0, 32, 33, DELETE, &[], &[]), Err(Error::Full));
    assert_eq!(state, before);
    assert_eq!(recover(&records).unwrap(), state);
}

#[test]
fn recovery_rejects_all_single_byte_damage_and_noncanonical_rehashed_fields() {
    let mut state = State::new();
    let original = apply(&mut state, 0, PUT, b"short", b"value");
    for i in 0..SECTOR {
        let mut record = original;
        record[i] ^= 0x40;
        assert_eq!(
            journal::decode_record(&record, 0),
            Err(Error::Corrupt),
            "byte {i}"
        );
    }
    for i in (12..16).chain(54..64).chain(69..192).chain(197..504) {
        let mut record = original;
        record[i] = 1;
        resign(&mut record);
        assert_eq!(
            journal::decode_record(&record, 0),
            Err(Error::Corrupt),
            "padding {i}"
        );
    }
}

#[test]
fn recovery_checks_rename_predecessor_and_content_despite_valid_checksum() {
    let mut state = State::new();
    let first = apply(&mut state, 0, PUT, b"before", b"keep");
    let renamed = apply(&mut state, 0, RENAME, b"after", &[]);
    let mut changed = renamed;
    changed[64] ^= 1;
    resign(&mut changed);
    assert!(journal::decode_record(&changed, 1).is_ok());
    assert_eq!(recover(&[first, changed]), Err(Error::Corrupt));
    let mut no_predecessor = first;
    no_predecessor[48..50].copy_from_slice(&RENAME.to_le_bytes());
    resign(&mut no_predecessor);
    assert_eq!(recover(&[no_predecessor]), Err(Error::Corrupt));
    let mut deletion = first;
    deletion[48..50].copy_from_slice(&DELETE.to_le_bytes());
    deletion[50..54].fill(0);
    deletion[64..224].fill(0);
    resign(&mut deletion);
    assert_eq!(recover(&[deletion]), Err(Error::Corrupt));
}

#[test]
fn recovery_requires_complete_contiguous_scan_and_stays_failed() {
    let mut state = State::new();
    let first = apply(&mut state, 0, PUT, b"a", b"a");
    let second = apply(&mut state, 0, RENAME, b"b", &[]);
    assert_eq!(recover(&[[0; SECTOR], second]), Err(Error::Corrupt));
    let mut reader = Recovery::new();
    reader.push(0, &first).unwrap();
    assert_eq!(reader.finish(), Err(Error::Corrupt));
    let mut reader = Recovery::new();
    assert_eq!(reader.push(1, &first), Err(Error::Corrupt));
    assert_eq!(reader.push(0, &first), Err(Error::Corrupt));
    assert_eq!(reader.finish(), Err(Error::Corrupt));
    assert_eq!(
        journal::decode_record(&[0; SECTOR], usize::MAX),
        Err(Error::Corrupt)
    );
}

#[test]
fn recovery_rejects_revision_jump_and_invalid_utf8_with_new_checksum() {
    let mut state = State::new();
    let first = apply(&mut state, 0, PUT, b"name", b"data");
    let second = apply(&mut state, 0, RENAME, b"next", &[]);
    let mut jump = second;
    wire::put64(&mut jump, 40, 1);
    resign(&mut jump);
    assert_eq!(recover(&[first, jump]), Err(Error::Corrupt));
    let mut invalid = first;
    invalid[192] = 0xff;
    resign(&mut invalid);
    assert_eq!(journal::decode_record(&invalid, 0), Err(Error::Corrupt));
    for at in [50, 52] {
        let mut oversized = first;
        oversized[at..at + 2].copy_from_slice(&u16::MAX.to_le_bytes());
        resign(&mut oversized);
        assert_eq!(journal::decode_record(&oversized, 0), Err(Error::Corrupt));
    }
}

#[test]
fn valid_legacy_v1_record_is_rejected_without_modifying_it() {
    let mut legacy = [0; SECTOR];
    legacy[..8].copy_from_slice(b"NOSM6B01");
    legacy[8..12].copy_from_slice(&[1, 0, 64, 0]);
    for at in [16, 24, 40] {
        wire::put64(&mut legacy, at, 1);
    }
    legacy[48..50].copy_from_slice(&PUT.to_le_bytes());
    legacy[50..52].copy_from_slice(&4u16.to_le_bytes());
    legacy[64..68].copy_from_slice(b"old!");
    resign(&mut legacy);
    let before = legacy;
    assert_eq!(recover(&[legacy]), Err(Error::Corrupt));
    assert_eq!(legacy, before);
}

#[test]
fn mixed_lifecycle_roundtrips_all_records_and_deleted_metadata_is_cleared() {
    let mut state = State::new();
    let mut records = Vec::new();
    for id in 0..4 {
        records.push(apply(
            &mut state,
            id,
            PUT,
            &[b'A' + id as u8; 32],
            &[id as u8; 128],
        ));
    }
    for id in 0..4 {
        records.push(apply(&mut state, id, RENAME, &[b'a' + id as u8; 32], &[]));
    }
    for id in [1, 3] {
        records.push(apply(&mut state, id, DELETE, &[], &[]));
    }
    assert_eq!(state.sequence(), 10);
    assert_eq!(recover(&records).unwrap(), state);
    for record in &records[8..] {
        assert_eq!(record[50..54], [0; 4]);
        assert!(record[64..504].iter().all(|byte| *byte == 0));
    }
}
