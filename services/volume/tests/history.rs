//! S06: collections, metadata, restorable history and atomic transactions.
mod common;

use common::{fill, pattern, recover, Local, SimDisk};
use volume_core::{
    client::Client,
    service::Service,
    table::Table,
    wire::{self, SubOp},
    Status, CHUNK, PER_DOMAIN,
};

const EPOCH: u64 = 9;

fn client<'a>(service: &'a mut Service, disk: &'a mut SimDisk, domain: usize) -> Client<Local<'a>> {
    Client::new(
        Local {
            service,
            disk,
            domain,
        },
        EPOCH,
        500 + 1000 * domain as u64,
    )
}

fn content(
    c: &mut Client<Local<'_>>,
    object: u64,
    revision: u64,
    selector: usize,
    length: usize,
) -> Vec<u8> {
    let mut out = vec![0u8; length];
    c.read_version(object, revision, selector, length, |offset, bytes| {
        out[offset..offset + bytes.len()].copy_from_slice(bytes)
    })
    .unwrap();
    out
}

fn expected(object: u64, revision: u64, length: usize) -> Vec<u8> {
    (0..length).map(|i| pattern(object, revision, i)).collect()
}

fn listed(c: &mut Client<Local<'_>>, domain: usize, filter: usize) -> Vec<u64> {
    let mut entries = [wire::ListEntry::EMPTY; PER_DOMAIN];
    let (count, _) = c.list_filtered(domain, filter, &mut entries).unwrap();
    entries[..count].iter().map(|entry| entry.object).collect()
}

#[test]
fn collections_metadata_and_filtered_listing() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, 0);
    for object in 0..4u64 {
        c.write(
            object,
            0,
            format!("Doc {object}").as_bytes(),
            600,
            fill(object, 1),
        )
        .unwrap();
    }
    assert_eq!(c.collection(0, 1, 0, b"Werk"), Ok(1));
    assert_eq!(c.collection(0, 2, 0, "Privé".as_bytes()), Ok(1));
    assert_eq!(c.collection(0, 1, 0, b"Stale"), Err((Status::Stale, 0)));
    // Unknown collection, then valid moves with tags and kind.
    assert_eq!(c.meta(0, 1, 1, 2, 3), Err((Status::Range, 0)));
    assert_eq!(c.meta(0, 1, 0b101, 2, 1), Ok(2));
    assert_eq!(c.meta(1, 1, 0, 0, 1), Ok(2));
    assert_eq!(c.meta(2, 1, 7, 0, 2), Ok(2));
    assert_eq!(c.meta(2, 1, 7, 0, 2), Err((Status::Stale, 0)));
    let stat = c.stat(0).unwrap();
    assert_eq!((stat.tags, stat.kind, stat.collection), (0b101, 2, 1));
    assert!(stat.created >= 1 && stat.modified > stat.created);
    assert_eq!(listed(&mut c, 0, 0), vec![0, 1, 2, 3]);
    assert_eq!(listed(&mut c, 0, 2), vec![0, 1]);
    assert_eq!(listed(&mut c, 0, 3), vec![2]);
    assert_eq!(listed(&mut c, 0, 1), vec![3]);
    assert_eq!(listed(&mut c, 0, 4), Vec::<u64>::new());
    // A collection in use cannot be deleted; empty it first.
    assert_eq!(c.collection(0, 2, 1, b""), Err((Status::Busy, 0)));
    assert_eq!(c.meta(2, 2, 7, 0, 0), Ok(3));
    assert_eq!(c.collection(0, 2, 1, b""), Ok(2));
    assert_eq!(c.collection(0, 2, 2, b""), Err((Status::NotFound, 2)));
    assert_eq!(c.meta(2, 3, 0, 0, 2), Err((Status::Range, 0)));
    // The other domain sees and changes none of this.
    let mut b = client(&mut service, &mut disk, 1);
    assert_eq!(listed(&mut b, 1, 0), Vec::<u64>::new());
    assert_eq!(b.meta(0, 2, 0, 0, 0), Err((Status::Access, 0)));
    assert_eq!(b.collection(0, 1, 1, b"x"), Err((Status::Access, 0)));
    assert_eq!(b.collection(1, 1, 0, b"Eigen"), Ok(1));
    let recovered = recover(disk.settled()).unwrap();
    assert_eq!(&recovered, service.table());
    assert_eq!(recovered.collection(0, 1).name(), b"Werk");
    assert!(!recovered.collection(0, 2).present);
    assert_eq!(recovered.collection(1, 1).name(), b"Eigen");
}

#[test]
fn history_restore_undelete_and_purge() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, 0);
    let lengths = [5000usize, 9000, 100];
    for (index, length) in lengths.iter().enumerate() {
        let revision = index as u64;
        c.write(
            4,
            revision,
            format!("Versie {}", revision + 1).as_bytes(),
            *length,
            fill(4, revision + 1),
        )
        .unwrap();
    }
    let stat = c.stat(4).unwrap();
    assert_eq!((stat.revision, stat.history_count), (3, 2));
    assert_eq!(
        (stat.history[0].revision, stat.history[0].length),
        (2, 9000)
    );
    assert_eq!(
        (stat.history[1].revision, stat.history[1].length),
        (1, 5000)
    );
    assert_eq!(content(&mut c, 4, 3, 1, 9000), expected(4, 2, 9000));
    assert_eq!(content(&mut c, 4, 3, 2, 5000), expected(4, 1, 5000));
    // Selectors beyond the history and stale object revisions are refused.
    assert_eq!(c.read_version(4, 2, 1, 9000, |_, _| {}), Err(Status::Stale));
    assert_eq!(c.restore(4, 3, 2).map(|_| ()), Ok(()));
    let stat = c.stat(4).unwrap();
    assert_eq!(
        (stat.revision, stat.version_revision, stat.length),
        (4, 4, 5000)
    );
    assert_eq!(&stat.name[..stat.name_length], b"Versie 1");
    assert_eq!((stat.history[0].revision, stat.history[1].revision), (3, 2));
    assert_eq!(content(&mut c, 4, 4, 0, 5000), expected(4, 1, 5000));
    // Undelete: the deleted current version becomes the newest history entry.
    assert_eq!(c.delete(4, 4), Ok(5));
    assert_eq!(c.stat(4), Err((Status::NotFound, 5)));
    assert_eq!(content(&mut c, 4, 5, 1, 5000), expected(4, 1, 5000));
    assert_eq!(c.read_version(4, 5, 0, 1, |_, _| {}), Err(Status::NotFound));
    assert_eq!(c.restore(4, 5, 1), Ok(6));
    let stat = c.stat(4).unwrap();
    // The undeleted version left the history; the older one is kept.
    assert_eq!((stat.length, stat.history_count), (5000, 1));
    assert_eq!(stat.history[0].revision, 3);
    assert_eq!(content(&mut c, 4, 6, 0, 5000), expected(4, 1, 5000));
    let recovered = recover(disk.settled()).unwrap();
    assert_eq!(&recovered, service.table());
    let mut c = client(&mut service, &mut disk, 0);
    // Purge releases the retained versions; nothing remains to restore.
    let before = c.transport.service.table().domain_chunks(0);
    assert_eq!(c.delete(4, 6), Ok(7));
    assert_eq!(c.purge(4, 7), Ok(8));
    assert_eq!(c.transport.service.table().domain_chunks(0), 0);
    assert!(before > 0);
    assert_eq!(c.restore(4, 8, 1), Err((Status::NotFound, 8)));
    assert_eq!(c.purge(5, 0), Err((Status::NotFound, 0)));
    // The freed chunks are reused by the next upload.
    c.write(5, 0, b"Nieuw", 2 * CHUNK, fill(5, 1)).unwrap();
    assert_eq!(
        c.transport.service.table().entries[5].current.chunks[..2],
        [0, 1]
    );
}

#[test]
fn restoring_the_newest_history_version_keeps_the_oldest() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, 0);
    for (revision, length) in [5000usize, 9000, 100].into_iter().enumerate() {
        let revision = revision as u64;
        c.write(4, revision, b"Doc", length, fill(4, revision + 1))
            .unwrap();
    }
    let chunks = c.transport.service.table().domain_chunks(0);
    // Bring back v2 (history slot 1) while v1 is the oldest retained version.
    assert_eq!(c.restore(4, 3, 1), Ok(4));
    let stat = c.stat(4).unwrap();
    assert_eq!((stat.version_revision, stat.length), (4, 9000));
    assert_eq!(stat.history_count, 2);
    assert_eq!((stat.history[0].revision, stat.history[0].length), (3, 100));
    assert_eq!(
        (stat.history[1].revision, stat.history[1].length),
        (1, 5000)
    );
    assert_eq!(content(&mut c, 4, 4, 0, 9000), expected(4, 2, 9000));
    assert_eq!(content(&mut c, 4, 4, 1, 100), expected(4, 3, 100));
    assert_eq!(content(&mut c, 4, 4, 2, 5000), expected(4, 1, 5000));
    // No version was dropped, so no chunk was released or duplicated.
    assert_eq!(c.transport.service.table().domain_chunks(0), chunks);
    let recovered = recover(disk.settled()).unwrap();
    assert_eq!(&recovered, service.table());
}

#[test]
fn transactions_are_all_or_nothing() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, 1);
    for object in 8..12u64 {
        c.write(object, 0, b"Doc", 300, fill(object, 1)).unwrap();
    }
    c.collection(1, 3, 0, b"Archief").unwrap();
    let move_to = |index: u8, expected: u64| SubOp {
        index,
        op: wire::SUB_META,
        collection: 3,
        tags: 1,
        expected,
        kind: 0,
    };
    let delete = |index: u8, expected: u64| SubOp {
        index,
        op: wire::SUB_DELETE,
        collection: 0,
        tags: 0,
        expected,
        kind: 0,
    };
    let snapshot = *c.transport.service.table();
    let writes = c.transport.disk.mutating_ops;
    // One stale precondition rejects the whole transaction without I/O.
    assert_eq!(
        c.transact(1, &[move_to(0, 1), move_to(1, 1), delete(2, 99)]),
        Err(Status::Stale)
    );
    // A missing object or collection likewise.
    assert_eq!(
        c.transact(1, &[move_to(0, 1), delete(6, 0)]),
        Err(Status::NotFound)
    );
    let mut bad = move_to(0, 1);
    bad.collection = 4;
    assert_eq!(c.transact(1, &[move_to(1, 1), bad]), Err(Status::Range));
    // Duplicate objects are structurally invalid.
    assert_eq!(
        c.transact(1, &[move_to(0, 1), delete(0, 1)]),
        Err(Status::Invalid)
    );
    assert_eq!(c.transport.service.table(), &snapshot);
    assert_eq!(c.transport.disk.mutating_ops, writes);
    // The valid transaction changes all three objects in one table commit.
    let generation = c.transport.service.table().generation;
    c.transact(1, &[move_to(0, 1), move_to(1, 1), delete(2, 1)])
        .unwrap();
    let table = *c.transport.service.table();
    assert_eq!(table.generation, generation + 1);
    assert_eq!(
        (table.entries[8].collection, table.entries[9].collection),
        (3, 3)
    );
    assert!(!table.entries[10].present && table.entries[10].history_count == 1);
    assert_eq!(table.entries[11].revision, 1);
    assert_eq!(listed(&mut c, 1, 4), vec![8, 9]);
    // Purge in a transaction as well; cross-domain selectors are refused.
    let purge = SubOp {
        index: 2,
        op: wire::SUB_PURGE,
        collection: 0,
        tags: 0,
        expected: 2,
        kind: 0,
    };
    c.transact(1, &[purge]).unwrap();
    assert_eq!(c.transport.service.table().entries[10].history_count, 0);
    let mut a = client(&mut service, &mut disk, 0);
    assert_eq!(a.transact(1, &[move_to(0, 2)]), Err(Status::Access));
    assert_eq!(&recover(disk.settled()).unwrap(), service.table());
}

#[test]
fn read_only_refuses_every_s06_mutation() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, 0);
    c.write(0, 0, b"Doc", 10, fill(0, 1)).unwrap();
    c.write(0, 1, b"Doc", 10, fill(0, 2)).unwrap();
    c.collection(0, 1, 0, b"Werk").unwrap();
    let table = recover(disk.settled()).unwrap();
    let mut disk = SimDisk::from_image(disk.settled());
    let mut service = Service::new(EPOCH, table, true);
    let mut c = client(&mut service, &mut disk, 0);
    assert_eq!(c.restore(0, 2, 1), Err((Status::Access, 0)));
    assert_eq!(c.purge(0, 2), Err((Status::Access, 0)));
    assert_eq!(c.meta(0, 2, 1, 1, 1), Err((Status::Access, 0)));
    assert_eq!(c.collection(0, 1, 1, b"Nee"), Err((Status::Access, 0)));
    assert_eq!(
        c.transact(
            0,
            &[SubOp {
                index: 0,
                op: wire::SUB_PURGE,
                collection: 0,
                tags: 0,
                expected: 2,
                kind: 0
            }]
        ),
        Err(Status::Access)
    );
    // Reading history remains possible.
    assert_eq!(content(&mut c, 0, 2, 1, 10), expected(0, 1, 10));
    assert!(disk.log.iter().all(|(op, _)| *op == common::Op::Read));
}

#[test]
fn malformed_s06_requests_are_rejected() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut cases = Vec::new();
    let mut meta = wire::Header::new(wire::META, 3, EPOCH, 0);
    meta.aux = wire::meta(0, 0, 5);
    cases.push(wire::encode(&meta, &[]));
    meta.aux = 1 << 60;
    cases.push(wire::encode(&meta, &[]));
    let mut restore = wire::Header::new(wire::RESTORE, 3, EPOCH, 0);
    restore.aux = 3;
    cases.push(wire::encode(&restore, &[]));
    restore.aux = 0;
    cases.push(wire::encode(&restore, &[]));
    let mut collection = wire::Header::new(wire::COLLECTION, 3, EPOCH, 1);
    collection.aux = 1;
    cases.push(wire::encode(&collection, &[]));
    collection.object = 0;
    collection.aux = 5;
    cases.push(wire::encode(&collection, &[]));
    let mut transact = wire::Header::new(wire::TRANSACT, 3, EPOCH, 0);
    transact.count = 5;
    transact.length = 5 * 24;
    cases.push(wire::encode(&transact, &[0; 120]));
    let mut sub = [0u8; 24];
    sub[1] = 9;
    transact.count = 1;
    transact.length = 24;
    cases.push(wire::encode(&transact, &sub));
    let mut sub = [0u8; 24];
    sub[0] = 8;
    sub[1] = wire::SUB_DELETE;
    cases.push(wire::encode(&transact, &sub));
    let mut sub = [0u8; 24];
    sub[1] = wire::SUB_DELETE;
    sub[2] = 1;
    cases.push(wire::encode(&transact, &sub));
    let mut read = wire::Header::new(wire::READ, 3, EPOCH, 0);
    read.count = 10;
    read.name_length = 3;
    cases.push(wire::encode(&read, &[]));
    let mut list = wire::Header::new(wire::LIST, 3, EPOCH, 0);
    list.count = 6;
    cases.push(wire::encode(&list, &[]));
    // COLLECTION_STAT is canonical only on the base object, with an index
    // and no other field.
    let canonical = || {
        let mut stat = wire::Header::new(wire::COLLECTION_STAT, 3, EPOCH, 0);
        stat.aux = 1;
        stat
    };
    let mut stat = canonical();
    stat.object = 1;
    cases.push(wire::encode(&stat, &[]));
    let mut stat = canonical();
    stat.revision = 1;
    cases.push(wire::encode(&stat, &[]));
    let mut stat = canonical();
    stat.count = 1;
    cases.push(wire::encode(&stat, &[]));
    let mut stat = canonical();
    stat.name_length = 1;
    cases.push(wire::encode(&stat, &[]));
    let mut stat = canonical();
    stat.length = 1;
    cases.push(wire::encode(&stat, b"a"));
    for case in &cases {
        let reply = service.handle(0, case, &mut disk);
        assert_eq!(wire::header(&reply).status, Status::Invalid as i32);
    }
    assert!(disk.log.is_empty());
}

/// Object stamps come from the owning domain's clock: no value domain A can
/// read changes when domain B commits.
#[test]
fn stamps_do_not_reveal_other_domain_commits() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    client(&mut service, &mut disk, 0)
        .write(0, 0, b"Eigen", 10, fill(0, 1))
        .unwrap();
    let mut a = client(&mut service, &mut disk, 0);
    assert_eq!(a.meta(0, 1, 1, 0, 0), Ok(2));
    let first = a.stat(0).unwrap();
    let mut listing = [wire::ListEntry::EMPTY; PER_DOMAIN];
    let (_, counter) = a.list(0, &mut listing).unwrap();
    let mut b = client(&mut service, &mut disk, 1);
    for object in 8..13u64 {
        b.write(object, 0, b"Ander", 10, fill(object, 1)).unwrap();
    }
    b.collection(1, 1, 0, b"Prive").unwrap();
    let mut a = client(&mut service, &mut disk, 0);
    assert_eq!(a.list(0, &mut listing).unwrap().1, counter);
    assert_eq!(a.stat(0).unwrap(), first);
    assert_eq!(a.meta(0, 2, 2, 0, 0), Ok(3));
    let second = a.stat(0).unwrap();
    assert_eq!(second.created, first.created);
    assert_eq!(second.modified, first.modified + 1);
    let table = *service.table();
    assert_eq!(table.clocks, [3, 6]);
    assert_eq!(recover(disk.settled()).unwrap(), table);
}

/// Collections are readable over IPC: presence, name, revision and the
/// request that produced it, for the caller's own domain only and also on a
/// read-only mount. That resolves an Unknown COLLECTION commit by reading.
#[test]
fn collection_state_is_readable_and_resolves_unknown() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, 0);
    assert_eq!(c.collection_stat(0, 1), Ok(Default::default()));
    let id = c.next_request_id();
    assert_eq!(c.collection(0, 1, 0, b"Werk"), Ok(1));
    let werk = c.collection_stat(0, 1).unwrap();
    assert_eq!(
        (werk.revision, werk.request_id, werk.present, werk.name()),
        (1, id, true, &b"Werk"[..])
    );
    // Another domain's index 1 is its own collection; its object range and
    // its endpoint are the only way to reach it.
    assert_eq!(c.collection_stat(1, 1), Err(Status::Access));
    assert_eq!(c.collection_stat(0, 0), Err(Status::Invalid));
    assert_eq!(c.collection_stat(0, 5), Err(Status::Invalid));
    assert_eq!(c.collection(0, 1, 1, b""), Ok(2));
    let deleted = c.collection_stat(0, 1).unwrap();
    assert_eq!(
        (deleted.revision, deleted.present, deleted.name_length),
        (2, false, 0)
    );
    // The table write of the next COLLECTION commit fails after all its
    // sectors were written: the outcome is Unknown and the instance closes.
    let attempt = c.next_request_id();
    c.transport.disk.fail_once_at =
        Some(c.transport.disk.mutating_ops + volume_core::table::TABLE_SECTORS_USIZE);
    assert_eq!(c.collection(0, 2, 0, b"Huis"), Err((Status::Unknown, 0)));
    // A fresh read-only instance sees the attempt (the request ID matches),
    // but that answer is provisional: the copy was never flushed and a
    // read-only instance flushes nothing, so power loss can still undo it.
    let mut readonly = Service::recover(EPOCH, true, &mut disk).unwrap();
    let mut r = client(&mut readonly, &mut disk, 0);
    let huis = r.collection_stat(0, 2).unwrap();
    assert_eq!(
        (huis.revision, huis.request_id, huis.present, huis.name()),
        (1, attempt, true, &b"Huis"[..])
    );
    assert_eq!(r.collection(0, 3, 0, b"Nee"), Err((Status::Access, 0)));
    assert!(!disk.pending.is_empty());
    let mut lost = SimDisk::from_image(disk.crash_image(0, None));
    let mut after_loss = Service::recover(EPOCH, true, &mut lost).unwrap();
    assert_eq!(
        client(&mut after_loss, &mut lost, 0).collection_stat(0, 2),
        Ok(Default::default())
    );
    // A writable instance makes its chosen copy durable before it answers,
    // so its answer resolves the Unknown commit for good.
    let mut writable = Service::recover(EPOCH, false, &mut disk).unwrap();
    assert!(disk.pending.is_empty());
    let huis = client(&mut writable, &mut disk, 0)
        .collection_stat(0, 2)
        .unwrap();
    assert_eq!(
        (huis.revision, huis.request_id, huis.present),
        (1, attempt, true)
    );
    let settled = recover(disk.crash_image(0, None)).unwrap();
    assert_eq!(settled.collection(0, 2).request_id, attempt);
}
