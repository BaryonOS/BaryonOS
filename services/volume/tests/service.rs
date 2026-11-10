mod common;

use common::{fill, pattern, recover, Local, Op, SimDisk};
use volume_core::{
    client::{validate_reply, Client},
    service::Service,
    table::{self, SectorState, Table},
    wire::{self, Header},
    Status, CHUNK, CHUNKS, DATA_START, MAX_LENGTH, PER_DOMAIN, QUOTA, SECTOR, TABLE_SECTORS,
    VOLUME_SECTORS,
};

const EPOCH: u64 = 7;

fn client<'a>(service: &'a mut Service, disk: &'a mut SimDisk, domain: usize) -> Client<Local<'a>> {
    Client::new(
        Local {
            service,
            disk,
            domain,
        },
        EPOCH,
        1000 * (domain as u64 + 1),
    )
}

fn read_all(c: &mut Client<Local<'_>>, object: u64, revision: u64, length: usize) -> Vec<u8> {
    let mut out = vec![0u8; length];
    let n = c
        .read(object, revision, length, |offset, bytes| {
            out[offset..offset + bytes.len()].copy_from_slice(bytes)
        })
        .unwrap();
    assert_eq!(n, length);
    out
}

fn expected(object: u64, revision: u64, length: usize) -> Vec<u8> {
    (0..length).map(|i| pattern(object, revision, i)).collect()
}

#[test]
fn empty_volume_recovers_as_generation_zero() {
    let mut disk = SimDisk::new();
    let mut table = Table::EMPTY;
    table::recover(&mut disk, &mut table).unwrap();
    assert_eq!(table, Table::EMPTY);
    assert_eq!(disk.log.len(), VOLUME_SECTORS as usize);
}

#[test]
fn nonzero_unrecognized_data_is_not_admitted_as_an_empty_volume() {
    let mut disk = SimDisk::new();
    disk.durable[VOLUME_SECTORS as usize - 1][0] = 0xb5;
    let mut table = Table::EMPTY;
    assert_eq!(table::recover(&mut disk, &mut table), Err(Status::Corrupt));
    assert_eq!(disk.log.last(), Some(&(Op::Read, VOLUME_SECTORS - 1)));
    assert!(!disk
        .log
        .iter()
        .any(|(operation, _)| matches!(operation, Op::Write | Op::Flush)));
}

#[test]
fn unmarked_nonzero_table_sectors_are_not_admitted_as_empty() {
    for sector in 0..DATA_START {
        let mut disk = SimDisk::new();
        disk.durable[sector as usize][17] = 0xb5;
        let mut table = Table::EMPTY;
        assert_eq!(
            table::recover(&mut disk, &mut table),
            Err(Status::Corrupt),
            "unmarked sector {sector}"
        );
        assert!(!disk
            .log
            .iter()
            .any(|(operation, _)| matches!(operation, Op::Write | Op::Flush)));
    }
}

#[test]
fn format_marker_preserves_recovery_of_a_torn_first_table() {
    let mut disk = SimDisk::new();
    table::write_format_marker(&mut disk).unwrap();
    disk.durable[DATA_START as usize - 1][17] = 0xb5;
    disk.log.clear();
    let mut table = Table::EMPTY;
    assert_eq!(table::recover_with_state(&mut disk, &mut table), Ok(true));
    assert_eq!(table, Table::EMPTY);
    assert!(!disk
        .log
        .iter()
        .any(|(operation, _)| matches!(operation, Op::Write | Op::Flush)));
}

#[test]
fn direct_service_constructor_cannot_claim_a_foreign_volume() {
    for sector in [TABLE_SECTORS, DATA_START - 1, VOLUME_SECTORS - 1] {
        let mut disk = SimDisk::new();
        disk.durable[sector as usize][0] = 0xb5;
        let mut service = Service::new(EPOCH, Table::EMPTY, false);
        let mut c = client(&mut service, &mut disk, 0);
        assert_eq!(
            c.write(0, 0, b"Document", 1, fill(0, 1)),
            Err(Status::Corrupt),
            "unmarked sector {sector}"
        );
        assert!(!disk
            .log
            .iter()
            .any(|(operation, _)| matches!(operation, Op::Write | Op::Flush)));
    }
}

#[test]
fn supplied_generation_does_not_skip_foreign_volume_admission() {
    let mut disk = SimDisk::new();
    disk.durable[VOLUME_SECTORS as usize - 1][0] = 0xb5;
    let mut supplied = Table::EMPTY;
    supplied.generation = 1;
    let mut service = Service::new(EPOCH, supplied, false);
    let mut c = client(&mut service, &mut disk, 0);
    assert_eq!(
        c.write(0, 0, b"Document", 1, fill(0, 1)),
        Err(Status::Corrupt)
    );
    assert!(!disk
        .log
        .iter()
        .any(|(operation, _)| matches!(operation, Op::Write | Op::Flush)));
}

#[test]
fn first_upload_marks_the_empty_volume_before_staging_content() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, 0);
    c.write(0, 0, b"Document", 1000, fill(0, 1)).unwrap();
    let writes: Vec<_> = disk
        .log
        .iter()
        .copied()
        .filter(|(operation, _)| *operation != Op::Read)
        .collect();
    assert_eq!(
        &writes[..3],
        &[(Op::Write, 0), (Op::Flush, 0), (Op::Write, DATA_START)]
    );
}

#[test]
fn committed_generation_one_with_zero_companion_is_preserved() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    client(&mut service, &mut disk, 0)
        .write(0, 0, b"First", 1, fill(0, 1))
        .unwrap();
    let mut legacy = disk.settled();
    legacy[..TABLE_SECTORS as usize].fill([0; SECTOR]);
    assert_eq!(recover(legacy), Ok(*service.table()));
}

#[test]
fn adjacent_generation_checks_do_not_wrap_or_panic() {
    let valid = table::SlotScan {
        states: [SectorState::Valid(2, 7); table::TABLE_SECTORS_USIZE],
        format_marker: false,
        valid: Some(2),
    };
    let mut mixed = table::SlotScan {
        states: [SectorState::Zero; table::TABLE_SECTORS_USIZE],
        format_marker: false,
        valid: None,
    };
    mixed.states[0] = SectorState::Valid(u64::MAX, 8);
    assert_eq!(table::choose([valid, mixed]), Err(Status::Corrupt));
    let maximum = table::SlotScan {
        states: [SectorState::Valid(u64::MAX, 8); table::TABLE_SECTORS_USIZE],
        format_marker: false,
        valid: Some(u64::MAX),
    };
    assert_eq!(table::choose([maximum, maximum]), Err(Status::Corrupt));
    mixed.states.fill(SectorState::Invalid);
    mixed.states[0] = SectorState::Valid(0, 7);
    assert_eq!(table::choose([mixed, maximum]), Err(Status::Corrupt));
}

/// A sealed table with a deleted object at revision `u64::MAX` (all other
/// invariants intact) overflowed `revision + 1` in entry validation: a panic
/// with overflow checks, a silent wrap to 0 in release. It must fail closed.
#[test]
fn deleted_object_revision_from_disk_does_not_overflow() {
    let table = |revision: u64| {
        let mut t = Table::EMPTY;
        t.generation = revision;
        t.request_id = 1;
        t.clocks[0] = revision;
        let entry = &mut t.entries[0];
        entry.revision = revision;
        entry.request_id = 1;
        entry.created = 1;
        entry.modified = 1;
        entry.present = false;
        t
    };
    // Both slots hold a valid table, so `Corrupt` can only come from the
    // newest revision itself, not from a missing companion.
    let image = |newest: u64| {
        let mut sectors = vec![[0u8; SECTOR]; VOLUME_SECTORS as usize];
        for t in [table(newest - 1), table(newest)] {
            let slot = (t.generation % 2) * TABLE_SECTORS;
            for index in 0..TABLE_SECTORS {
                sectors[(slot + index) as usize] = t.encode_sector(index);
            }
        }
        sectors
    };
    let last = u64::MAX - 1;
    assert!(
        recover(image(last)) == Ok(table(last)),
        "control image one revision below the bound must recover"
    );
    assert_eq!(recover(image(u64::MAX)), Err(Status::Corrupt));
}

/// A delete from revision `u64::MAX - 1` would publish a tombstone at
/// `u64::MAX`, which recovery rejects (it bounds the history by revision + 1).
/// The service refuses it as exhausted before writing, via DELETE and TRANSACT,
/// so the durable table stays the recoverable one.
#[test]
fn delete_at_last_tombstone_revision_is_refused_before_publishing() {
    let g = u64::MAX - 1;
    // Object 0 present, last written by the commit of `generation`.
    let table = |generation: u64| {
        let mut t = Table::EMPTY;
        t.generation = generation;
        t.request_id = 1;
        t.clocks[0] = generation;
        let entry = &mut t.entries[0];
        entry.revision = generation;
        entry.request_id = 1;
        entry.created = 1;
        entry.modified = generation;
        entry.present = true;
        entry.current = table::Version {
            revision: generation,
            name_length: 1,
            ..table::Version::EMPTY
        };
        entry.current.name[0] = b'x';
        t
    };
    let current = table(g);
    let mut image = vec![[0u8; SECTOR]; VOLUME_SECTORS as usize];
    for t in [table(g - 1), current] {
        let slot = (t.generation % 2) * TABLE_SECTORS;
        for index in 0..TABLE_SECTORS {
            image[(slot + index) as usize] = t.encode_sector(index);
        }
    }
    assert!(
        recover(image.clone()) == Ok(current),
        "control image must recover"
    );
    let mut disk = SimDisk::from_image(image);
    let mut service = Service::recover(EPOCH, false, &mut disk).unwrap();
    let writes = disk.log.len();
    let mut c = client(&mut service, &mut disk, 0);
    assert_eq!(c.delete(0, g), Err((Status::Full, 0)));
    let sub = wire::SubOp {
        index: 0,
        op: wire::SUB_DELETE,
        collection: 0,
        tags: 0,
        expected: g,
        kind: 0,
    };
    assert_eq!(c.transact(0, &[sub]), Err(Status::Full));
    assert_eq!(c.transport.disk.log.len(), writes);
    assert!(c.transport.service.table() == &current);
    assert!(recover(disk.settled()) == Ok(current));
}

#[test]
fn large_document_roundtrip_beyond_old_128_byte_limit() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, 0);
    for (object, length) in [(0u64, MAX_LENGTH), (1, 129), (2, 0), (3, 4097), (4, 512)] {
        let (revision, id) = c
            .write(object, 0, "Rapport é".as_bytes(), length, fill(object, 1))
            .unwrap();
        assert_eq!(revision, 1);
        let stat = c.stat(object).unwrap();
        assert_eq!(
            (stat.revision, stat.length, stat.request_id),
            (1, length, id)
        );
        assert_eq!(&stat.name[..stat.name_length], "Rapport é".as_bytes());
        assert_eq!(
            read_all(&mut c, object, 1, length),
            expected(object, 1, length)
        );
    }
    // Cold state from disk equals the live state.
    let recovered = recover(disk.settled()).unwrap();
    assert_eq!(&recovered, service.table());
    assert_eq!(recovered.generation, 5);
    // 16 + 1 + 0 + 2 + 1 chunks for the five documents above.
    assert_eq!(recovered.domain_chunks(0), 20);
}

#[test]
fn space_is_reused_far_beyond_record_and_capacity_limits() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut revisions = [0u64; 3];
    // Each object retains its two previous versions, so sizes leave room for
    // three versions of everything plus one staged upload within the quota.
    for round in 0..90u64 {
        let object = round % 3;
        let length = [4 * CHUNK, 3 * CHUNK + 17, 9000][object as usize];
        let mut c = client(&mut service, &mut disk, 0);
        let (revision, _) = c
            .write(
                object,
                revisions[object as usize],
                b"Draft",
                length,
                fill(object, revisions[object as usize] + 1),
            )
            .unwrap();
        revisions[object as usize] = revision;
        assert_eq!(
            read_all(&mut c, object, revision, length),
            expected(object, revision, length)
        );
    }
    assert_eq!(service.counters.commits, 90);
    // Far more chunks were allocated than the volume holds, and more commits
    // happened than the old 32-record journal allowed.
    assert!(service.counters.chunk_allocations > 4 * CHUNKS as u64);
    assert_eq!(service.table().domain_chunks(0), 3 * (4 + 4 + 3));
    let recovered = recover(disk.settled()).unwrap();
    assert_eq!(&recovered, service.table());
    assert_eq!(recovered.generation, 90);
}

#[test]
fn quota_and_full_are_reported_before_any_io() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    {
        let mut c = client(&mut service, &mut disk, 0);
        for object in 0..2 {
            c.write(object, 0, b"Groot", MAX_LENGTH, fill(object, 1))
                .unwrap();
        }
        // 32 of 40 chunks used: another 16-chunk document exceeds the quota.
        let writes = c.transport.disk.log.len();
        assert_eq!(
            c.write(2, 0, b"Te groot", MAX_LENGTH, fill(2, 1)),
            Err(Status::Quota)
        );
        assert_eq!(c.transport.disk.log.len(), writes);
        c.write(2, 0, b"Past", 8 * CHUNK, fill(2, 1)).unwrap();
        assert_eq!(c.transport.service.table().domain_chunks(0), QUOTA);
    }
    {
        let mut c = client(&mut service, &mut disk, 1);
        c.write(8, 0, b"B", 16 * CHUNK, fill(8, 1)).unwrap();
        c.write(9, 0, b"B2", 8 * CHUNK, fill(9, 1)).unwrap();
        // 64 chunks are in use: the volume itself is full for domain 1 even
        // though domain 1 is still under its own quota.
        let writes = c.transport.disk.log.len();
        assert_eq!(c.write(10, 0, b"Vol", 1, fill(10, 1)), Err(Status::Full));
        assert_eq!(c.transport.disk.log.len(), writes);
    }
    {
        // Deleting keeps the content restorable; only PURGE releases the
        // space that the other domain can then use.
        let mut c = client(&mut service, &mut disk, 0);
        assert_eq!(c.delete(1, 1), Ok(2));
        assert_eq!(c.transport.service.table().domain_chunks(0), QUOTA);
        assert_eq!(c.purge(1, 2), Ok(3));
    }
    let mut c = client(&mut service, &mut disk, 1);
    c.write(10, 0, b"Vol", 15 * CHUNK, fill(10, 1)).unwrap();
    assert_eq!(
        read_all(&mut c, 10, 1, 15 * CHUNK),
        expected(10, 1, 15 * CHUNK)
    );
}

#[test]
fn domains_are_isolated_and_listing_is_paged() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, 1);
    for object in [8u64, 10, 11, 13, 15] {
        c.write(
            object,
            0,
            format!("Doc {object}").as_bytes(),
            300,
            fill(object, 1),
        )
        .unwrap();
    }
    let mut entries = [wire::ListEntry::EMPTY; PER_DOMAIN];
    let (count, generation) = c.list(1, &mut entries).unwrap();
    assert_eq!((count, generation), (5, 5));
    assert_eq!(
        entries[..count]
            .iter()
            .map(|e| e.object)
            .collect::<Vec<_>>(),
        vec![8, 10, 11, 13, 15]
    );
    // Cross-domain selectors are rejected before state is consulted.
    for header in [
        Header::new(wire::STAT, 1, EPOCH, 0),
        Header::new(wire::LIST, 2, EPOCH, 0),
        Header::new(wire::DELETE, 3, EPOCH, 3),
    ] {
        let mut h = header;
        if h.operation == wire::DELETE {
            h.revision = 0;
        }
        let (reply, _) = c.exchange(&h, &[]).unwrap();
        assert_eq!(reply.status, Status::Access as i32);
    }
    assert_eq!(c.write(2, 0, b"x", 10, fill(2, 1)), Err(Status::Access));
    let (reply, _) = c
        .exchange(&Header::new(wire::STAT, 9, EPOCH, 16), &[])
        .unwrap();
    assert_eq!(reply.status, Status::Range as i32);
    let mut a = client(&mut service, &mut disk, 0);
    let (count, generation) = a.list(0, &mut entries).unwrap();
    assert_eq!((count, generation), (0, 0));
}

#[test]
fn malformed_requests_are_rejected_without_io() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut good = Header::new(wire::BEGIN, 5, EPOCH, 0);
    good.aux = 10;
    good.length = 3;
    good.name_length = 3;
    let base = wire::encode(&good, b"abc");
    let mut cases: Vec<[u8; wire::SIZE]> = Vec::new();
    let mut m = base;
    m[0] = 4; // version
    cases.push(m);
    let mut m = base;
    m[5] = 1; // reserved
    cases.push(m);
    let mut m = base;
    m[16] ^= 1; // epoch
    cases.push(m);
    let mut m = base;
    m[8..16].fill(0); // request id zero
    cases.push(m);
    let mut m = base;
    m[200] = 1; // nonzero padding beyond data
    cases.push(m);
    let mut m = base;
    m[2] = 0; // operation zero
    cases.push(m);
    let mut m = base;
    m[2] = 10;
    cases.push(m);
    let mut m = base;
    m[64] = b'/'; // invalid name
    cases.push(m);
    let mut m = base;
    m[56..64].copy_from_slice(&(MAX_LENGTH as u64 + 1).to_le_bytes());
    cases.push(m);
    let mut m = base;
    m[44] = 1; // status set in request
    cases.push(m);
    let mut read = Header::new(wire::READ, 6, EPOCH, 0);
    read.count = 161;
    cases.push(wire::encode(&read, &[]));
    read.count = 0;
    cases.push(wire::encode(&read, &[]));
    let mut append = Header::new(wire::APPEND, 6, EPOCH, 0);
    append.length = 0;
    cases.push(wire::encode(&append, &[]));
    for case in &cases {
        let reply = service.handle(0, case, &mut disk);
        let h = wire::header(&reply);
        assert_ne!(h.status, 0, "accepted malformed request");
        assert!(reply[64..].iter().all(|b| *b == 0));
        assert_eq!(h.revision, 0);
    }
    assert!(disk.log.is_empty());
    assert_eq!(service.counters.rejected, cases.len() as u64);
}

#[test]
fn streaming_protocol_rules() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, 0);
    let id = c
        .write_partial(0, 0, b"Half", 1000, 320, fill(0, 1))
        .unwrap();
    // Wrong offset and premature commit.
    let mut append = Header::new(wire::APPEND, id, EPOCH, 0);
    append.aux = 0;
    append.length = 1;
    let (reply, _) = c.exchange(&append, &[1]).unwrap();
    assert_eq!(reply.status, Status::Range as i32);
    let mut commit = Header::new(wire::COMMIT, id, EPOCH, 0);
    commit.aux = 1000;
    let (reply, _) = c.exchange(&commit, &[]).unwrap();
    assert_eq!(reply.status, Status::Range as i32);
    // Abort releases everything; the object never appears.
    c.abort(0, 0, id).unwrap();
    assert_eq!(c.abort(0, 0, id), Err(Status::Invalid));
    assert_eq!(c.stat(0), Err((Status::NotFound, 0)));
    // A second BEGIN replaces an abandoned session of the same domain.
    let first = c
        .write_partial(1, 0, b"Old", 9000, 160, fill(1, 1))
        .unwrap();
    let (revision, _) = c.write(1, 0, b"New", 9000, fill(1, 1)).unwrap();
    assert_eq!(revision, 1);
    let mut stale = Header::new(wire::APPEND, first, EPOCH, 1);
    stale.aux = 160;
    stale.length = 1;
    let (reply, _) = c.exchange(&stale, &[1]).unwrap();
    assert_eq!(reply.status, Status::Invalid as i32);
    // A rename between BEGIN and COMMIT makes the upload stale.
    let id = c.write_partial(1, 1, b"Newer", 10, 10, fill(1, 2)).unwrap();
    assert_eq!(c.rename(1, 1, b"Hernoemd"), Ok(2));
    let mut commit = Header::new(wire::COMMIT, id, EPOCH, 1);
    commit.revision = 1;
    commit.aux = 10;
    let (reply, _) = c.exchange(&commit, &[]).unwrap();
    assert_eq!(reply.status, Status::Stale as i32);
    let stat = c.stat(1).unwrap();
    assert_eq!(
        (stat.revision, &stat.name[..stat.name_length]),
        (2, &b"Hernoemd"[..])
    );
    assert_eq!(read_all(&mut c, 1, 2, 9000), expected(1, 1, 9000));
    // Reads of an older revision are stale, never silently newer bytes.
    let mut read = Header::new(wire::READ, 77, EPOCH, 1);
    read.revision = 1;
    read.count = 10;
    let (reply, _) = c.exchange(&read, &[]).unwrap();
    assert_eq!(reply.status, Status::Stale as i32);
}

#[test]
fn delete_rename_and_tombstones() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, 0);
    c.write(3, 0, b"Brief", 5000, fill(3, 1)).unwrap();
    assert_eq!(c.rename(3, 1, "Brief v2 ✓".as_bytes()), Ok(2));
    assert_eq!(c.rename(3, 1, b"Stale"), Err(Status::Stale));
    assert_eq!(c.delete(3, 2), Ok(3));
    assert_eq!(c.delete(3, 3), Err((Status::NotFound, 3)));
    assert_eq!(c.stat(3), Err((Status::NotFound, 3)));
    assert_eq!(c.rename(3, 3, b"x"), Err(Status::NotFound));
    let (revision, _) = c.write(3, 3, b"Opnieuw", 10, fill(3, 4)).unwrap();
    assert_eq!(revision, 4);
    assert_eq!(&recover(disk.settled()).unwrap(), service.table());
}

#[test]
fn read_only_instance_refuses_mutations() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    client(&mut service, &mut disk, 0)
        .write(0, 0, b"Doc", 700, fill(0, 1))
        .unwrap();
    let table = recover(disk.settled()).unwrap();
    let mut disk = SimDisk::from_image(disk.settled());
    let mut service = Service::new(EPOCH, table, true);
    let mut c = client(&mut service, &mut disk, 0);
    assert_eq!(read_all(&mut c, 0, 1, 700), expected(0, 1, 700));
    assert_eq!(c.write(1, 0, b"x", 1, fill(1, 1)), Err(Status::Access));
    assert_eq!(c.rename(0, 1, b"x"), Err(Status::Access));
    assert_eq!(c.delete(0, 1), Err((Status::Access, 0)));
    assert!(disk.log.iter().all(|(op, _)| *op == common::Op::Read));
}

#[test]
fn corrupted_content_and_tables_fail_closed() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    client(&mut service, &mut disk, 0)
        .write(0, 0, b"Doc", 5000, fill(0, 1))
        .unwrap();
    client(&mut service, &mut disk, 0)
        .write(1, 0, b"Doc2", 100, fill(1, 1))
        .unwrap();
    let image = disk.settled();
    // Flip one content byte: reads of that chunk report Corrupt, others work.
    let chunk = service.table().entries[0].current.chunks[1] as u64;
    let mut bad = image.clone();
    bad[(DATA_START + chunk * 8) as usize][3] ^= 0x40;
    let table = recover(bad.clone()).unwrap();
    let mut disk2 = SimDisk::from_image(bad);
    let mut s2 = Service::new(EPOCH, table, true);
    let mut c = client(&mut s2, &mut disk2, 0);
    let mut read = Header::new(wire::READ, 1, EPOCH, 0);
    read.revision = 1;
    read.aux = 4096;
    read.count = 100;
    let (reply, _) = c.exchange(&read, &[]).unwrap();
    assert_eq!(reply.status, Status::Corrupt as i32);
    read.aux = 0;
    let (reply, _) = c.exchange(&read, &[]).unwrap();
    assert_eq!(reply.status, 0);
    // A newest copy whose sectors are all individually sealed for generation 2
    // but whose content is not canonical is corruption, never a torn write:
    // recovery stops instead of silently using generation 1.
    let newest = Table::slot(2) * TABLE_SECTORS;
    let mut invalid = *service.table();
    invalid.entries[1].current.name[0] = b'/';
    let mut bad = image.clone();
    for index in 0..TABLE_SECTORS {
        bad[(newest + index) as usize] = invalid.encode_sector(index);
    }
    assert_eq!(recover(bad), Err(Status::Corrupt));
    // Residual risk, deliberately pinned by this test: one garbled sector in
    // the newest copy is indistinguishable from an interrupted unflushed
    // rewrite of that slot, so recovery falls back to generation 1. Content
    // that generation 1 references is still checksum-verified on every read.
    let mut bad = image.clone();
    bad[(newest + 1) as usize][7] ^= 1;
    assert_eq!(
        table::classify(Table::slot(2), 1, &bad[(newest + 1) as usize]),
        SectorState::Invalid
    );
    assert_eq!(recover(bad).map(|table| table.generation), Ok(1));
    // Garbage in the older copy is accepted only as a torn sector and the
    // newest complete generation is used.
    let mut bad = image.clone();
    bad[(Table::slot(1) * TABLE_SECTORS + 4) as usize][0] = 0xff;
    assert_eq!(recover(bad).map(|table| table.generation), Ok(2));
    // An older copy that is not generation 1 or 3 is an unexplained state.
    // The forged copy is valid on its own (its clocks add up), so only the
    // generation rule of `choose` can refuse the pair.
    let mut bad = image;
    let mut foreign = *service.table();
    foreign.clocks[0] += 5 - foreign.generation;
    foreign.generation = 5;
    let mut alone = Table::EMPTY;
    assert_eq!(
        Table::decode_from(
            Table::slot(5),
            |index| Ok(foreign.encode_sector(index)),
            &mut alone
        ),
        Ok(())
    );
    for index in 0..TABLE_SECTORS {
        bad[(Table::slot(5) * TABLE_SECTORS + index) as usize] = foreign.encode_sector(index);
    }
    assert_eq!(recover(bad), Err(Status::Corrupt));
}

#[test]
fn certain_io_failures_abort_without_closing() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    client(&mut service, &mut disk, 0)
        .write(0, 0, b"Basis", 2000, fill(0, 1))
        .unwrap();
    let before = *service.table();
    // Fail the 3rd data write of a new upload.
    disk.fail_once_at = Some(disk.mutating_ops + 2);
    let result = client(&mut service, &mut disk, 0).write(1, 0, b"Nieuw", 4000, fill(1, 1));
    assert_eq!(result, Err(Status::Io));
    assert_eq!(service.table(), &before);
    assert!(!service.uncertain());
    // Fail the pre-table flush.
    let mut c = client(&mut service, &mut disk, 0);
    let id = c
        .write_partial(1, 0, b"Nieuw", 600, 600, fill(1, 1))
        .unwrap();
    c.transport.disk.fail_once_at = Some(c.transport.disk.mutating_ops + 1);
    let mut commit = Header::new(wire::COMMIT, id, EPOCH, 1);
    commit.aux = 600;
    let (reply, _) = c.exchange(&commit, &[]).unwrap();
    assert_eq!(reply.status, Status::Io as i32);
    // The instance keeps serving and later commits succeed.
    c.write(1, 0, b"Nieuw", 600, fill(1, 1)).unwrap();
    // Read errors are reported without changing state.
    disk.fail_reads_from = Some(DATA_START);
    let mut c = client(&mut service, &mut disk, 0);
    let mut read = Header::new(wire::READ, 3, EPOCH, 0);
    read.revision = 1;
    read.count = 5;
    let (reply, _) = c.exchange(&read, &[]).unwrap();
    assert_eq!(reply.status, Status::Io as i32);
}

#[test]
fn table_write_failure_is_unknown_and_closes_instance() {
    for failing in 0..=TABLE_SECTORS as usize {
        let mut disk = SimDisk::new();
        let mut service = Service::new(EPOCH, Table::EMPTY, false);
        client(&mut service, &mut disk, 0)
            .write(0, 0, b"Basis", 100, fill(0, 1))
            .unwrap();
        let before = *service.table();
        let mut c = client(&mut service, &mut disk, 0);
        // Upload of 100 bytes: 1 data write + flush, then 9 table writes + flush.
        let id = c
            .write_partial(0, 1, b"Nieuw", 100, 100, fill(0, 2))
            .unwrap();
        c.transport.disk.fail_once_at = Some(c.transport.disk.mutating_ops + 2 + failing);
        let mut commit = Header::new(wire::COMMIT, id, EPOCH, 0);
        commit.revision = 1;
        commit.aux = 100;
        let (reply, _) = c.exchange(&commit, &[]).unwrap();
        assert_eq!(reply.status, Status::Unknown as i32, "failing op {failing}");
        // Every later valid request is Unknown, invalid ones stay Invalid.
        assert_eq!(c.stat(0), Err((Status::Unknown, 0)));
        let (reply, _) = c
            .exchange(&Header::new(wire::STAT, 99, EPOCH, 0), &[])
            .unwrap();
        assert_eq!(reply.status, Status::Unknown as i32);
        let writes = c.transport.disk.mutating_ops;
        assert_eq!(c.write(2, 0, b"x", 1, fill(2, 1)), Err(Status::Unknown));
        assert_eq!(
            c.transport.disk.mutating_ops, writes,
            "no replay or new I/O"
        );
        // Fresh recovery decides: old or new, never anything else.
        let recovered = recover(disk.settled()).unwrap();
        assert!(recovered == before || recovered.entries[0].revision == 2);
        if recovered != before {
            assert_eq!(recovered.entries[0].request_id, id);
        }
    }
}

#[test]
fn shutdown_flushes_once_and_refuses_later_mutations() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, 0);
    c.write(0, 0, b"Doc", 700, fill(0, 1)).unwrap();
    let id = c
        .write_partial(1, 0, b"Half", 900, 320, fill(1, 1))
        .unwrap();
    let before = *c.transport.service.table();
    let counters = c.transport.service.counters;
    let logged = c.transport.disk.log.len();
    assert_eq!(c.shutdown(0), Ok(()));
    assert_eq!(&c.transport.disk.log[logged..], &[(common::Op::Flush, 0)]);
    assert_eq!(c.transport.service.counters.flushes, counters.flushes + 1);
    assert_eq!(
        c.transport.service.counters.aborted_sessions,
        counters.aborted_sessions + 1
    );
    assert!(c.transport.service.session(0).is_none());
    assert_eq!(c.transport.service.table(), &before);
    // The staged upload never reached the table; mutations are now refused.
    let mut append = Header::new(wire::APPEND, id, EPOCH, 1);
    append.aux = 320;
    append.length = 1;
    let (reply, _) = c.exchange(&append, &[1]).unwrap();
    assert_eq!(reply.status, Status::Access as i32);
    assert_eq!(c.write(2, 0, b"x", 1, fill(2, 1)), Err(Status::Access));
    assert_eq!(c.rename(0, 1, b"x"), Err(Status::Access));
    assert_eq!(c.delete(0, 1), Err((Status::Access, 0)));
    assert_eq!(c.shutdown(0), Err(Status::Access));
    assert_eq!(c.stat(0).unwrap().revision, 1);
    assert_eq!(read_all(&mut c, 0, 1, 700), expected(0, 1, 700));
    assert!(c.transport.disk.log[logged + 1..]
        .iter()
        .all(|(op, _)| *op == common::Op::Read));
    assert_eq!(&recover(disk.settled()).unwrap(), &before);
}

#[test]
fn shutdown_flush_failure_is_unknown_and_closes_instance() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, 0);
    c.write(0, 0, b"Doc", 100, fill(0, 1)).unwrap();
    c.transport.disk.fail_once_at = Some(c.transport.disk.mutating_ops);
    assert_eq!(c.shutdown(0), Err(Status::Unknown));
    assert!(c.transport.service.uncertain());
    assert_eq!(c.transport.service.counters.unknown, 1);
    let ops = c.transport.disk.ops;
    let mutating = c.transport.disk.mutating_ops;
    assert_eq!(c.stat(0), Err((Status::Unknown, 0)));
    assert_eq!(c.write(1, 0, b"x", 1, fill(1, 1)), Err(Status::Unknown));
    assert_eq!(c.shutdown(0), Err(Status::Unknown));
    assert_eq!(
        (c.transport.disk.ops, c.transport.disk.mutating_ops),
        (ops, mutating),
        "no replay or new I/O"
    );
}

#[test]
fn shutdown_of_a_read_only_instance_is_refused_without_io() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, true);
    assert_eq!(
        client(&mut service, &mut disk, 0).shutdown(0),
        Err(Status::Access)
    );
    assert!(disk.log.is_empty());
}

#[test]
fn malformed_shutdown_requests_are_rejected_without_io() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let shutdown = |object| Header::new(wire::SHUTDOWN, 5, EPOCH, object);
    let mut cases = vec![(shutdown(1), Status::Invalid)];
    cases.push((shutdown(PER_DOMAIN as u64), Status::Access));
    let mut h = shutdown(0);
    h.revision = 1;
    cases.push((h, Status::Invalid));
    let mut h = shutdown(0);
    h.name_length = 1;
    cases.push((h, Status::Invalid));
    let mut h = shutdown(0);
    h.count = 1;
    cases.push((h, Status::Invalid));
    let mut h = shutdown(0);
    h.aux = 1;
    cases.push((h, Status::Invalid));
    cases.push((Header::new(wire::SHUTDOWN + 1, 5, EPOCH, 0), Status::NoSys));
    for (header, status) in &cases {
        let reply = service.handle(0, &wire::encode(header, &[]), &mut disk);
        assert_eq!(wire::header(&reply).status, *status as i32, "{header:?}");
    }
    let mut h = shutdown(0);
    h.length = 1;
    let reply = service.handle(0, &wire::encode(&h, b"a"), &mut disk);
    assert_eq!(wire::header(&reply).status, Status::Invalid as i32);
    assert!(disk.log.is_empty());
    // Nothing above closed the instance.
    assert_eq!(client(&mut service, &mut disk, 0).shutdown(0), Ok(()));
}

#[test]
fn client_rejects_tampered_replies() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    client(&mut service, &mut disk, 0)
        .write(0, 0, b"Doc", 300, fill(0, 1))
        .unwrap();
    let mut read = Header::new(wire::READ, 50, EPOCH, 0);
    read.revision = 1;
    read.count = 100;
    let reply = service.handle(0, &wire::encode(&read, &[]), &mut disk);
    assert!(validate_reply(&read, &reply, EPOCH).is_ok());
    for (at, value) in [
        (8usize, 1u8),
        (16, 1),
        (24, 1),
        (40, 200),
        (2, 9),
        (223, 1),
        (56, 3),
    ] {
        let mut bad = reply;
        bad[at] ^= value;
        assert_eq!(
            validate_reply(&read, &bad, EPOCH),
            Err(Status::Protocol),
            "byte {at}"
        );
    }
    let mut stat = Header::new(wire::STAT, 51, EPOCH, 0);
    stat.revision = 0;
    let reply = service.handle(0, &wire::encode(&stat, &[]), &mut disk);
    assert!(validate_reply(&stat, &reply, EPOCH).is_ok());
    let mut bad = reply;
    bad[64] = b'/';
    assert_eq!(validate_reply(&stat, &bad, EPOCH), Err(Status::Protocol));
}

#[test]
fn client_rejects_inconsistent_stat_and_list_replies() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, 0);
    for revision in 0..3 {
        c.write(0, revision, b"Doc", 300, fill(0, revision + 1))
            .unwrap();
    }
    let stat = Header::new(wire::STAT, 60, EPOCH, 0);
    let reply = service.handle(0, &wire::encode(&stat, &[]), &mut disk);
    assert!(validate_reply(&stat, &reply, EPOCH).is_ok());
    let put = |body: &mut [u8; wire::SIZE], at: usize, value: u64| {
        body[64 + at..64 + at + 8].copy_from_slice(&value.to_le_bytes())
    };
    // Each case breaks exactly one rule; the other checks still hold.
    let tampered: [(&str, usize, u64); 6] = [
        ("version above object revision", 72, u64::MAX),
        ("oldest history revision zero", 100, 0),
        ("history not older", 80, 3),
        ("history out of order", 100, 2),
        ("created zero", 48, 0),
        ("created after modified", 48, u64::MAX),
    ];
    for (what, at, value) in tampered {
        let mut bad = reply;
        put(&mut bad, at, value);
        assert_eq!(
            validate_reply(&stat, &bad, EPOCH),
            Err(Status::Protocol),
            "{what}"
        );
    }
    let mut bad = reply;
    bad[64 + 88..64 + 92].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(validate_reply(&stat, &bad, EPOCH), Err(Status::Protocol));
    // A version of zero, for an object without history (so no ordering rule
    // applies).
    client(&mut service, &mut disk, 0)
        .write(1, 0, b"Los", 10, fill(1, 1))
        .unwrap();
    let single = Header::new(wire::STAT, 62, EPOCH, 1);
    let reply = service.handle(0, &wire::encode(&single, &[]), &mut disk);
    assert!(validate_reply(&single, &reply, EPOCH).is_ok());
    let mut bad = reply;
    put(&mut bad, 72, 0);
    assert_eq!(validate_reply(&single, &bad, EPOCH), Err(Status::Protocol));
    // A LIST reply with a huge cursor is refused before its entries are
    // inspected (no overflow, also in debug builds). The entry is inside the
    // requested range, so an unchecked cursor would reach the bound sum.
    let list = Header::new(wire::LIST, 61, EPOCH, PER_DOMAIN as u64);
    let mut forged = Header::new(wire::LIST, 61, EPOCH, PER_DOMAIN as u64);
    forged.count = 1;
    forged.aux = u64::MAX;
    forged.length = wire::LIST_ENTRY as u32;
    let mut entry = [0u8; wire::LIST_ENTRY];
    entry[0] = PER_DOMAIN as u8;
    let body = wire::encode(&forged, &entry);
    assert_eq!(validate_reply(&list, &body, EPOCH), Err(Status::Protocol));
}

#[test]
fn client_rejects_forged_collection_stat_replies() {
    let request = Header::new(wire::COLLECTION_STAT, 70, EPOCH, 0);
    let reply = |revision: u64, request_id: u64, count: u32, name: &[u8], name_length: u32| {
        let mut h = Header::new(wire::COLLECTION_STAT, 70, EPOCH, 0);
        h.revision = revision;
        h.aux = request_id;
        h.count = count;
        h.length = name.len() as u32;
        h.name_length = name_length;
        wire::encode(&h, name)
    };
    let check = |body: [u8; wire::SIZE]| validate_reply(&request, &body, EPOCH).map(|_| ());
    // The three shapes the service produces.
    assert_eq!(check(reply(0, 0, 0, b"", 0)), Ok(()), "never created");
    assert_eq!(check(reply(2, 7, 1, b"Werk", 4)), Ok(()), "present");
    assert_eq!(check(reply(3, 8, 0, b"", 0)), Ok(()), "deleted");
    // Each forgery breaks one rule.
    let forged: [(&str, [u8; wire::SIZE]); 7] = [
        ("present without revision", reply(0, 0, 1, b"Werk", 4)),
        ("present without name", reply(2, 7, 1, b"", 0)),
        ("absent with a name", reply(3, 8, 0, b"Werk", 4)),
        ("count above one", reply(3, 8, 2, b"", 0)),
        ("revision without request ID", reply(2, 0, 1, b"Werk", 4)),
        ("name length differs", reply(2, 7, 1, b"Werk", 3)),
        ("invalid name", reply(2, 7, 1, b"a/b", 3)),
    ];
    for (what, body) in forged {
        assert_eq!(check(body), Err(Status::Protocol), "{what}");
    }
}

#[test]
fn table_sector_layout_is_canonical() {
    let mut table = Table::EMPTY;
    table.generation = 3;
    table.request_id = 9;
    table.clocks = [3, 0];
    table.entries[0].revision = 1;
    table.entries[0].request_id = 9;
    table.entries[0].created = 3;
    table.entries[0].modified = 3;
    table.entries[0].present = true;
    table.entries[0].current.revision = 1;
    table.entries[0].current.name_length = 1;
    table.entries[0].current.name[0] = b'a';
    let sectors: [[u8; SECTOR]; TABLE_SECTORS as usize] =
        core::array::from_fn(|i| table.encode_sector(i as u64));
    let decode = |slot: u64, sectors: &[[u8; SECTOR]; TABLE_SECTORS as usize]| {
        let mut out = Table::EMPTY;
        Table::decode_from(slot, |index| Ok(sectors[index as usize]), &mut out).map(|()| out)
    };
    assert_eq!(decode(1, &sectors), Ok(table));
    assert_eq!(decode(0, &sectors), Err(Status::Corrupt));
    let mut swapped = sectors;
    swapped.swap(2, 3);
    assert_eq!(decode(1, &swapped), Err(Status::Corrupt));
    // Domain clocks add up to the generation and bound every stamp of the
    // domain's objects.
    for clocks in [[2, 0], [3, 1], [0, 3]] {
        let mut wrong = table;
        wrong.clocks = clocks;
        let sectors: [[u8; SECTOR]; TABLE_SECTORS as usize] =
            core::array::from_fn(|i| wrong.encode_sector(i as u64));
        assert!(decode(1, &sectors).is_err(), "clocks {clocks:?} accepted");
    }
    // Each bound on its own: the clocks add up and every value fits the
    // generation, only one field exceeds its domain's clock (2).
    let mut modified = table;
    modified.clocks = [2, 1];
    let mut revision = table;
    revision.clocks = [2, 1];
    revision.entries[0].revision = 3;
    revision.entries[0].created = 2;
    revision.entries[0].modified = 2;
    let mut collection = table;
    let foreign = &mut collection.collections[volume_core::COLLECTIONS];
    foreign.revision = 1;
    foreign.request_id = 7;
    for (what, wrong) in [
        ("modified", modified),
        ("revision", revision),
        ("collection", collection),
    ] {
        let sectors: [[u8; SECTOR]; TABLE_SECTORS as usize] =
            core::array::from_fn(|i| wrong.encode_sector(i as u64));
        assert_eq!(decode(1, &sectors), Err(Status::Corrupt), "{what}");
    }
}
