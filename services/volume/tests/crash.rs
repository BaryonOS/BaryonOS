//! Exhaustive crash-point simulation: every mutating disk operation of every
//! step in a mixed scenario is interrupted, and several durable images are
//! derived from the unflushed write cache (none, prefixes, all, torn sector).
//! Each image must recover to exactly the state before or after the step,
//! and every present object must read back with its expected content.
mod common;

use common::{fill, pattern, Local, SimDisk};
use volume_core::{
    client::Client,
    service::Service,
    table::{self, Table},
    Status, CHUNK, MAX_LENGTH, SECTOR,
};

const EPOCH: u64 = 3;

#[derive(Clone, Copy, Debug)]
enum Step {
    Write { object: u64, length: usize },
    Delete { object: u64 },
    Rename { object: u64 },
}

const SCENARIO: &[Step] = &[
    Step::Write {
        object: 0,
        length: 700,
    },
    Step::Write {
        object: 8,
        length: 2 * CHUNK + 5,
    },
    Step::Write {
        object: 0,
        length: MAX_LENGTH,
    },
    Step::Rename { object: 8 },
    Step::Write {
        object: 1,
        length: 0,
    },
    Step::Delete { object: 0 },
    Step::Write {
        object: 9,
        length: 3 * CHUNK,
    },
    Step::Write {
        object: 0,
        length: 5 * CHUNK + 1,
    },
    Step::Delete { object: 8 },
    Step::Write {
        object: 8,
        length: 513,
    },
];

fn domain(object: u64) -> usize {
    (object / 8) as usize
}

/// Apply one step through the real client/service path.
fn apply(service: &mut Service, disk: &mut SimDisk, step: Step) -> Result<(), Status> {
    let object = match step {
        Step::Write { object, .. } | Step::Delete { object } | Step::Rename { object } => object,
    };
    let current = service.table().entries[object as usize].revision;
    let mut c = Client::new(
        Local {
            service,
            disk,
            domain: domain(object),
        },
        EPOCH,
        1 + current * 100 + object,
    );
    match step {
        Step::Write { object, length } => c
            .write(object, current, b"Stap", length, fill(object, current + 1))
            .map(|_| ()),
        Step::Delete { object } => c.delete(object, current).map(|_| ()).map_err(|(s, _)| s),
        Step::Rename { object } => c
            .rename(object, current, "Hernoemd ✓".as_bytes())
            .map(|_| ()),
    }
}

/// Every present object must be fully readable with the expected content.
fn verify_content(table: &Table, image: Vec<[u8; SECTOR]>) {
    let mut disk = SimDisk::from_image(image);
    let mut service = Service::new(EPOCH, *table, true);
    for (object, entry) in table.entries.iter().enumerate() {
        if !entry.present {
            continue;
        }
        // Content revision: renames keep the bytes of the earlier write.
        let content_revision = (1..=entry.revision)
            .rev()
            .find(|r| entry.content_sum == content_sum(object as u64, *r, entry.length))
            .expect("content does not match any written revision");
        let mut c = Client::new(
            Local {
                service: &mut service,
                disk: &mut disk,
                domain: object / 8,
            },
            EPOCH,
            1,
        );
        let mut seen = 0;
        c.read(
            object as u64,
            entry.revision,
            entry.length,
            |offset, bytes| {
                for (index, byte) in bytes.iter().enumerate() {
                    assert_eq!(
                        *byte,
                        pattern(object as u64, content_revision, offset + index)
                    );
                }
                seen += bytes.len();
            },
        )
        .unwrap();
        assert_eq!(seen, entry.length);
    }
}

fn content_sum(object: u64, revision: u64, length: usize) -> u64 {
    let bytes: Vec<u8> = (0..length).map(|i| pattern(object, revision, i)).collect();
    volume_core::checksum_update(volume_core::FNV_OFFSET, &bytes)
}

#[test]
fn every_crash_point_recovers_to_before_or_after() {
    // Reference run: durable images and tables after each step.
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut images = vec![disk.settled()];
    let mut tables = vec![Table::EMPTY];
    let mut mutating = vec![];
    for step in SCENARIO {
        let start = disk.mutating_ops;
        apply(&mut service, &mut disk, *step).unwrap();
        mutating.push(disk.mutating_ops - start);
        assert!(disk.pending.is_empty(), "every step ends with a flush");
        images.push(disk.settled());
        tables.push(*service.table());
    }
    let mut checked = 0usize;
    for (index, step) in SCENARIO.iter().enumerate() {
        for crash in 0..=mutating[index] {
            let mut disk = SimDisk::from_image(images[index].clone());
            let mut service = Service::new(EPOCH, tables[index], false);
            disk.crash_at = Some(crash);
            let result = apply(&mut service, &mut disk, *step);
            if crash == mutating[index] {
                assert_eq!(result, Ok(()));
            } else {
                assert!(result.is_err());
            }
            let pending = disk.pending.len();
            let mut variants = vec![(0, None), (pending, None)];
            for keep in [1, pending / 2, pending.saturating_sub(1)] {
                if keep < pending {
                    variants.push((keep, None));
                    variants.push((keep, Some(SECTOR / 3)));
                }
            }
            let mut images: Vec<(usize, Option<usize>, Vec<[u8; SECTOR]>)> = variants
                .into_iter()
                .map(|(keep, tear)| (keep, tear, disk.crash_image(keep, tear)))
                .collect();
            // Unordered persistence of the write cache: odd and even subsets.
            for parity in 0..2 {
                images.push((
                    usize::MAX - parity,
                    None,
                    disk.crash_subset(|i| i % 2 == parity),
                ));
            }
            // Reordered persistence where later writes (e.g. a complete table
            // copy) land while earlier ones (content sectors) are lost.
            for dropped in [0, pending / 2] {
                if dropped < pending {
                    images.push((
                        usize::MAX - 2 - dropped,
                        None,
                        disk.crash_subset(|i| i != dropped),
                    ));
                }
            }
            let suffix = pending.saturating_sub(table::TABLE_SECTORS_USIZE);
            images.push((usize::MAX - 9999, None, disk.crash_subset(|i| i >= suffix)));
            for (keep, tear, image) in images {
                let recovered = common::recover(image.clone())
                    .unwrap_or_else(|e| panic!("step {index} crash {crash} keep {keep}: {e:?}"));
                assert!(
                    recovered == tables[index] || recovered == tables[index + 1],
                    "step {index} crash {crash} keep {keep} tear {tear:?}: unexpected generation {}",
                    recovered.generation
                );
                verify_content(&recovered, image);
                checked += 1;
            }
        }
    }
    assert!(checked > 1000, "only {checked} crash images checked");
}

#[test]
fn recovered_instance_continues_after_interrupted_commit() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    apply(
        &mut service,
        &mut disk,
        Step::Write {
            object: 0,
            length: 9000,
        },
    )
    .unwrap();
    // Interrupt the next commit in the middle of its table write.
    let before = *service.table();
    disk.crash_at = Some(disk.mutating_ops + 3 + 4);
    assert!(apply(
        &mut service,
        &mut disk,
        Step::Write {
            object: 0,
            length: 9000
        }
    )
    .is_err());
    let image = disk.crash_image(disk.pending.len(), Some(100));
    let recovered = common::recover(image.clone()).unwrap();
    assert_eq!(recovered, before);
    // A fresh instance overwrites the torn copy and keeps working.
    let mut disk = SimDisk::from_image(image);
    let mut service = Service::new(EPOCH, recovered, false);
    for _ in 0..5 {
        apply(
            &mut service,
            &mut disk,
            Step::Write {
                object: 0,
                length: 9000,
            },
        )
        .unwrap();
    }
    let mut check = SimDisk::from_image(disk.settled());
    assert_eq!(&table::recover(&mut check).unwrap(), service.table());
}

/// Two interrupted attempts at the same generation whose surviving sectors
/// together form a complete copy (sectors 5-8 from attempt 1, 0-4 from
/// attempt 2). Recovery must treat that as torn and keep the durable state;
/// it must never resurrect attempt 1's rolled-back change.
#[test]
fn mixed_attempts_for_one_generation_are_never_combined() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    for step in [
        Step::Write {
            object: 1,
            length: 10,
        },
        Step::Write {
            object: 9,
            length: 10,
        },
    ] {
        apply(&mut service, &mut disk, step).unwrap();
    }
    let durable = *service.table();
    // Attempt 1: rename object 9 (table sector 5); only sectors 5-8 persist.
    disk.crash_at = Some(disk.mutating_ops + 9);
    assert!(apply(&mut service, &mut disk, Step::Rename { object: 9 }).is_err());
    assert_eq!(disk.pending.len(), 9);
    let image = disk.crash_subset(|index| index >= 5);
    assert_eq!(common::recover(image.clone()).unwrap(), durable);
    // Attempt 2 by a fresh instance: rename object 1 (sector 1); only 0-4 persist.
    let mut disk = SimDisk::from_image(image);
    let mut service = Service::new(EPOCH, durable, false);
    disk.crash_at = Some(9);
    assert!(apply(&mut service, &mut disk, Step::Rename { object: 1 }).is_err());
    let image = disk.crash_subset(|index| index < 5);
    let slot = Table::slot(durable.generation + 1);
    let states: Vec<_> = (0..table::TABLE_SECTORS_USIZE)
        .map(|index| {
            let sector = image[slot as usize * table::TABLE_SECTORS_USIZE + index];
            table::classify(slot, index as u64, &sector)
        })
        .collect();
    // Every sector is individually valid for the next generation ...
    assert!(states.iter().all(
        |state| matches!(state, table::SectorState::Valid(g, _) if *g == durable.generation + 1)
    ));
    // ... yet the copy is a mix of two attempts and recovery keeps `durable`.
    let recovered = common::recover(image).unwrap();
    assert_eq!(recovered, durable);
    assert_eq!(recovered.entries[9].revision, 1);
}
