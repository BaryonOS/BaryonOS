//! Test-only model of the five v2-to-v2 transitions in S09-CATALOG-V2 §5.3.
//! Inputs model already validated v2 copies. Device and service checks
//! (including ownership, VERIFY and erase prerequisites) run outside this model.

use crate::{
    catalog::{Catalog, Record},
    sectors_for, DATA_START, ID, MAX_ENTRIES,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Metadata {
    time: i64,
    source: u8,
    verified_sequence: u64,
    deleting: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CopyV2 {
    catalog: Catalog,
    metadata: [Metadata; MAX_ENTRIES],
    owners: [[u8; ID]; MAX_ENTRIES],
}

enum Transition {
    Sync,
    Seal {
        record: Record,
        time: i64,
        source: u8,
    },
    Mark(usize),
    Remove(usize),
    RecordVerify(usize),
}

fn apply(committed: &CopyV2, transition: Transition) -> Option<CopyV2> {
    let count = committed.catalog.count;
    if count > MAX_ENTRIES || committed.owners != [[0; ID]; MAX_ENTRIES] {
        return None;
    }
    let mut candidate = *committed;
    candidate.catalog.sequence = committed.catalog.sequence.checked_add(1)?;

    match transition {
        Transition::Sync => {}
        Transition::Seal {
            record,
            time,
            source,
        } => {
            if count == MAX_ENTRIES
                || record.id != committed.catalog.next_entry
                || record.sealed_sequence != candidate.catalog.sequence
                || time < 0
                || source > 2
                || (source == 0) != (time == 0)
            {
                return None;
            }
            let end = record.first.checked_add(sectors_for(record.length))?;
            candidate.catalog.records[count] = record;
            candidate.metadata[count] = Metadata {
                time,
                source,
                ..Metadata::default()
            };
            candidate.catalog.count += 1;
            candidate.catalog.next_entry = committed.catalog.next_entry.checked_add(1)?;
            candidate.catalog.next_free = committed.catalog.next_free.max(end);
        }
        Transition::Mark(index) if index < count => {
            if committed.metadata[..count]
                .iter()
                .any(|entry| entry.deleting)
            {
                return None;
            }
            candidate.metadata[index].deleting = true;
        }
        Transition::Remove(index) if index < count && committed.metadata[index].deleting => {
            let last = count - 1;
            candidate
                .catalog
                .records
                .copy_within(index + 1..count, index);
            candidate.metadata.copy_within(index + 1..count, index);
            candidate.catalog.records[last] = Record::EMPTY;
            candidate.metadata[last] = Metadata::default();
            candidate.catalog.count = last;
            let mut next_free = DATA_START;
            for record in candidate.catalog.entries() {
                let end = record.first.checked_add(sectors_for(record.length))?;
                next_free = next_free.max(end);
            }
            candidate.catalog.next_free = next_free;
        }
        Transition::RecordVerify(index) if index < count && !committed.metadata[index].deleting => {
            candidate.metadata[index].verified_sequence = candidate.catalog.sequence;
        }
        Transition::Mark(_) | Transition::Remove(_) | Transition::RecordVerify(_) => return None,
    }
    Some(candidate)
}

fn matches_one(committed: &CopyV2, candidate: &CopyV2) -> bool {
    let count = committed.catalog.count;
    let matches = |transition| apply(committed, transition).as_ref() == Some(candidate);
    if matches(Transition::Sync) {
        return true;
    }
    if let Some(record) = candidate.catalog.records.get(count) {
        let meta = candidate.metadata[count];
        if matches(Transition::Seal {
            record: *record,
            time: meta.time,
            source: meta.source,
        }) {
            return true;
        }
    }
    (0..count.min(MAX_ENTRIES)).any(|index| {
        matches(Transition::Mark(index))
            || matches(Transition::Remove(index))
            || matches(Transition::RecordVerify(index))
    })
}

fn catalog() -> CopyV2 {
    let mut catalog = Catalog::formatted([1; ID], 1500);
    catalog.count = 2;
    catalog.next_entry = 4; // Entry 3 was removed in an earlier commit.
    catalog.sequence = 8;
    catalog.records[0] = record(1, 900, 2);
    catalog.records[1] = record(2, 300, 3);
    catalog.next_free = 901;
    let mut metadata = [Metadata::default(); MAX_ENTRIES];
    metadata[0] = Metadata {
        time: 100,
        source: 1,
        verified_sequence: 4,
        deleting: false,
    };
    metadata[1] = Metadata {
        time: 80,
        source: 2,
        verified_sequence: 5,
        deleting: false,
    };
    CopyV2 {
        catalog,
        metadata,
        owners: [[0; ID]; MAX_ENTRIES],
    }
}

fn record(id: u64, first: u64, sealed_sequence: u64) -> Record {
    let mut label = [0; crate::LABEL];
    label[0] = b'v';
    Record {
        id,
        first,
        length: 512,
        trailer: 432,
        sealed_sequence,
        label_length: 1,
        label,
    }
}

#[test]
fn accepts_each_exact_transition() {
    let committed = catalog();
    let mut sync = committed;
    sync.catalog.sequence = 9;
    assert_eq!(apply(&committed, Transition::Sync), Some(sync));
    assert!(matches_one(&committed, &sync));

    let mut sealed = sync;
    let entry = record(4, 950, 9);
    sealed.catalog.records[2] = entry;
    sealed.catalog.count = 3;
    sealed.catalog.next_entry = 5;
    sealed.catalog.next_free = 951;
    sealed.metadata[2] = Metadata {
        time: 90,
        source: 2,
        ..Metadata::default()
    };
    assert_eq!(
        apply(
            &committed,
            Transition::Seal {
                record: entry,
                time: 90,
                source: 2
            }
        ),
        Some(sealed)
    );
    assert!(matches_one(&committed, &sealed));

    let mut recorded = sync;
    recorded.metadata[1].verified_sequence = 9;
    assert_eq!(
        apply(&committed, Transition::RecordVerify(1)),
        Some(recorded)
    );
    assert!(matches_one(&committed, &recorded));

    let mut marked = sync;
    marked.metadata[0].deleting = true;
    assert_eq!(apply(&committed, Transition::Mark(0)), Some(marked));
    assert!(matches_one(&committed, &marked));
    let mut removed = marked;
    removed.catalog.sequence = 10;
    removed.catalog.count = 1;
    removed.catalog.next_free = 301;
    removed.catalog.records[0] = committed.catalog.records[1];
    removed.catalog.records[1] = Record::EMPTY;
    removed.metadata[0] = committed.metadata[1];
    removed.metadata[1] = Metadata::default();
    assert_eq!(apply(&marked, Transition::Remove(0)), Some(removed));
    assert!(matches_one(&marked, &removed));
}

#[test]
fn rejects_retired_entry_id_and_counters_that_skip_or_wrap() {
    let committed = catalog();
    let seal = |id| Transition::Seal {
        record: record(id, 200, 9),
        time: 0,
        source: 0,
    };
    let sealed = apply(&committed, seal(4)).unwrap();
    assert_eq!(sealed.catalog.next_free, committed.catalog.next_free);
    assert!(matches_one(&committed, &sealed));
    assert_eq!(apply(&committed, seal(3)), None);
    let mut retired_id = sealed;
    retired_id.catalog.records[2].id = 3;
    assert!(!matches_one(&committed, &retired_id));
    let mut skipped_id = sealed;
    skipped_id.catalog.next_entry += 1;
    assert!(!matches_one(&committed, &skipped_id));
    let mut skipped_sequence = committed;
    skipped_sequence.catalog.sequence += 2;
    assert!(!matches_one(&committed, &skipped_sequence));
    let mut exhausted = committed;
    exhausted.catalog.next_entry = u64::MAX;
    assert_eq!(apply(&exhausted, seal(u64::MAX)), None);
    exhausted.catalog.sequence = u64::MAX;
    assert_eq!(apply(&exhausted, Transition::Sync), None);
}

#[test]
fn rejects_combined_transitions_and_changes_to_unrelated_fields() {
    let committed = catalog();
    let marked = apply(&committed, Transition::Mark(0)).unwrap();
    assert_eq!(apply(&marked, Transition::Mark(1)), None);
    assert_eq!(apply(&marked, Transition::RecordVerify(0)), None);
    assert_eq!(apply(&committed, Transition::Remove(0)), None);
    let mut both = marked;
    both.metadata[1].deleting = true;
    assert!(!matches_one(&committed, &both));
    both.metadata[1] = Metadata::default();
    assert!(!matches_one(&committed, &both));
    let removed = apply(&marked, Transition::Remove(0)).unwrap();
    let mut dropped_twice = removed;
    dropped_twice.catalog.count = 0;
    dropped_twice.catalog.records[0] = Record::EMPTY;
    dropped_twice.catalog.next_free = DATA_START;
    dropped_twice.metadata[0] = Metadata::default();
    assert!(!matches_one(&marked, &dropped_twice));
    let mut owner_changed = removed;
    owner_changed.owners[0][0] = 1;
    assert!(!matches_one(&marked, &owner_changed));
}
