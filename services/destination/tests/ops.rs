//! Operation rows: N7, N9, N10, N14, N15, N23, N29, N36.
mod common;

use common::*;
use destination_core::{catalog::COPY, fnv64, service::State, wire, Status, FNV_OFFSET, SECTOR};

/// N7: one byte flipped in a sealed extent: VERIFY `Corrupt`, and the
/// portal's own check over READ (stand-in verifier) refuses it.
/// Kills: a VERIFY that answers from the catalog without reading the extent.
#[test]
fn n7_corrupted_entry() {
    let mut h = H::formatted();
    let bytes = archive(1, 1, 600, 2);
    let e1 = h.export(1, &bytes);
    let first = h.svc.catalog().find(e1).unwrap().first;
    let mut image = h.disk.durable.clone();
    image[first as usize + 1][7] ^= 0x40;
    let mut h = H::cold(image);
    assert_eq!(h.bind(1, wire::BIND_VERIFY, e1), Status::Ok);
    assert_eq!(h.verify_entry(1, e1), Status::Corrupt);
    assert_eq!(h.bind(2, wire::BIND_RESTORE, e1), Status::Ok);
    let read = h.read_entry(2, e1, bytes.len() as u64).unwrap();
    assert_ne!(read, bytes);
    let n = read.len();
    assert_ne!(
        fnv64(FNV_OFFSET, &read[..n - 8]).to_le_bytes(),
        read[n - 8..]
    );
    assert_eq!(
        h.svc.state(),
        State::Mounted,
        "an entry error is not a mount state"
    );
}

/// N9: SEAL with a declared length other than the appended total.
#[test]
fn n9_incomplete_write() {
    let mut h = H::formatted();
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    let bytes = archive(1, 1, 300, 2);
    let entry = h.write_all(1, &bytes);
    assert_eq!(
        h.seal(1, entry, bytes.len() as u64 + 1).status,
        Status::Invalid
    );
    assert_eq!(
        h.seal(1, entry, bytes.len() as u64 - 1).status,
        Status::Invalid
    );
    assert_eq!(h.list_ids(0).1, Vec::<u64>::new(), "nothing listed");
    // Nothing discarded: the correct SEAL still succeeds.
    assert_eq!(h.seal(1, entry, bytes.len() as u64).status, Status::Ok);
}

/// N10: appended bytes that fail verification: SEAL `Corrupt`, nothing
/// listed, `next_free` unchanged after reboot.
#[test]
fn n10_non_archive_bytes() {
    let mut h = H::formatted();
    let before = h.svc.catalog().next_free;
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    let mut bytes = archive(1, 1, 300, 2);
    bytes[70] ^= 1;
    let entry = h.write_all(1, &bytes);
    assert_eq!(h.seal(1, entry, bytes.len() as u64).status, Status::Corrupt);
    assert!(!h.svc.safe_path(), "a failed verify is not a device error");
    assert_eq!(h.list_ids(0).1, Vec::<u64>::new());
    assert_eq!(h.list_ids(1), (Status::Ok, vec![]));
    let cold = h.reboot();
    assert_eq!(cold.svc.catalog().next_free, before);
    // Trailing bytes after a valid archive are not an archive either.
    let mut h = H::formatted();
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    let mut padded = archive(1, 1, 300, 2);
    padded.extend_from_slice(&[0; 5]);
    let entry = h.write_all(1, &padded);
    assert_eq!(
        h.seal(1, entry, padded.len() as u64).status,
        Status::Corrupt
    );
}

/// N14: wrong, foreign or absent device.
/// Kills: a check order that returns `NoDevice` from a request other than
/// BIND, or answers FORMAT in the `Corrupt` state with anything but `Corrupt`.
#[test]
fn n14_wrong_foreign_absent_device() {
    // Catalog Y, grant names X.
    let mut h = H::formatted();
    let ops = h.disk.log.len();
    let mut other = TEST_ID;
    other[0] ^= 1;
    assert_eq!(
        h.bind_with(1, wire::BIND_EXPORT, 0, 1, &other),
        Status::NoDevice
    );
    assert_eq!(h.disk.log.len(), ops + 1, "no I/O (only the Handled event)");
    assert!(h.svc.binding(1).is_none());
    assert_eq!(h.portal(1, req(wire::INFO), &[]).status, Status::Access);
    assert_eq!(h.info().0, Status::Ok);
    assert_eq!(h.control(req(wire::FORMAT), &[]).status, Status::Access);

    // Foreign, non-blank layout.
    let mut image = vec![[0; SECTOR]; SECTORS as usize];
    image[0][..4].copy_from_slice(b"\x7fELF");
    let mut h = H::cold(image);
    assert_eq!(h.svc.state(), State::Corrupt);
    assert_eq!(h.info().0, Status::Corrupt);
    assert_eq!(
        h.bind_with(1, wire::BIND_EXPORT, 0, 1, &TEST_ID),
        Status::Corrupt
    );
    assert_eq!(h.control(req(wire::FORMAT), &[]).status, Status::Corrupt);
    assert_eq!(h.disk.writes, 0);

    // No device: kernel -14 to the mount FLUSH.
    let mut disk = SimDisk::new(SECTORS);
    disk.fault(|_, _| {
        Some(Fault {
            submit: -14,
            ..Fault::default()
        })
    });
    let mut h = H::boot(disk);
    assert_eq!(h.svc.state(), State::Failed { no_device: true });
    assert_eq!(h.info().0, Status::NoDevice);
    assert_eq!(
        h.bind_with(1, wire::BIND_EXPORT, 0, 1, &TEST_ID),
        Status::Io
    );
    assert_eq!(h.control(req(wire::FORMAT), &[]).status, Status::Io);
    assert_eq!(h.list_ids(0).0, Status::Io);
}

/// N15: destination full: `Full` at APPEND, nothing sealed, earlier entries
/// intact, catalog sequence unchanged; CREATE `Full` when no sector is free.
#[test]
fn n15_destination_full() {
    let mut h = H::boot(SimDisk::new(24));
    assert_eq!(h.control(req(wire::FORMAT), &[]).status, Status::Ok);
    let e1_bytes = archive(1, 1, 2000, 1); // 5 sectors: 16..21
    let e1 = h.export(1, &e1_bytes);
    let sequence = h.svc.catalog().sequence;
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    let entry = wire::header(&h.create(1, b"big").reply.unwrap()).entry;
    let big = archive(2, 1, 1700, 2); // 4 sectors > 3 free
    let mut status = Status::Ok;
    let mut offset = 0;
    for chunk in big.chunks(wire::DATA) {
        status = h.append(1, entry, offset, chunk).status;
        if status != Status::Ok {
            break;
        }
        offset += chunk.len() as u64;
    }
    assert_eq!(status, Status::Full);
    assert_eq!(h.list_ids(0).1, vec![e1]);
    assert_eq!(h.svc.catalog().sequence, sequence);
    assert!(!h.svc.safe_path());
    let mut cold = h.reboot();
    let record = *cold.svc.catalog().find(e1).unwrap();
    assert_eq!(extent(&cold.disk.durable, &record), e1_bytes);
    assert_eq!(cold.svc.catalog().sequence, sequence);
    // Fill the last three sectors exactly, then CREATE is Full.
    let fill = archive(3, 1, 1400, 3); // 1480 bytes: 3 sectors
    let e3 = cold.export(2, &fill);
    assert_eq!(cold.svc.catalog().next_free, 24);
    assert_eq!(cold.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    assert_eq!(cold.create(1, b"none").status, Status::Full);
    assert_eq!(cold.list_ids(0).1, vec![e1, e3]);
}

/// N23: catalog sectors zero, data at sector >= 16: FORMAT `Access`, no write.
#[test]
fn n23_data_beyond_catalog_area() {
    for sector in [16, SECTORS - 1] {
        let mut image = vec![[0; SECTOR]; SECTORS as usize];
        image[sector as usize][100] = 1;
        let mut h = H::cold(image);
        assert_eq!(h.svc.state(), State::Blank);
        assert_eq!(h.control(req(wire::FORMAT), &[]).status, Status::Access);
        assert_eq!(h.disk.writes, 0);
        assert_eq!(h.svc.state(), State::Blank);
    }
}

fn resum(copy: &mut [u8; COPY]) {
    let sum = fnv64(FNV_OFFSET, &copy[..2040]);
    copy[2040..].copy_from_slice(&sum.to_le_bytes());
}

/// N29: the pre-write catalog check of 2.7 step 4.
/// Kills: a step 4 that writes without running the validator and the
/// extension check (a catalog sector is written).
#[test]
fn n29_prewrite_check() {
    type Tamper = fn(&mut [u8; COPY]);
    let tampers: [(&str, Tamper); 5] = [
        ("wrong checksum", |c| c[2045] ^= 1),
        ("next_free lower", |c| {
            let free = u64::from_le_bytes(c[48..56].try_into().unwrap());
            c[48..56].copy_from_slice(&(free - 1).to_le_bytes());
            resum(c);
        }),
        ("sequence + 2", |c| {
            let sequence = u64::from_le_bytes(c[32..40].try_into().unwrap());
            c[32..40].copy_from_slice(&(sequence + 1).to_le_bytes());
            resum(c);
        }),
        ("record dropped", |c| {
            // Drop the newest record and rewind next_free to its first sector.
            let count = c[10] as usize;
            let at = 64 + 80 * (count - 1);
            let first: [u8; 8] = c[at + 8..at + 16].try_into().unwrap();
            c[at..at + 80].fill(0);
            c[10] = (count - 1) as u8;
            c[48..56].copy_from_slice(&first);
            resum(c);
        }),
        ("new record sealed above the new sequence", |c| {
            let at = 64 + 80 * (c[10] as usize - 1) + 32;
            let sealed = u64::from_le_bytes(c[at..at + 8].try_into().unwrap());
            c[at..at + 8].copy_from_slice(&(sealed + 1).to_le_bytes());
            resum(c);
        }),
    ];
    for (name, tamper) in tampers {
        let mut h = H::formatted();
        let e1 = h.export(1, &archive(1, 1, 700, 1));
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        let bytes = archive(2, 1, 1200, 2);
        let entry = h.write_all(1, &bytes);
        h.svc.tamper = Some(tamper);
        h.disk.arm();
        assert_eq!(
            h.seal(1, entry, bytes.len() as u64).status,
            Status::Corrupt,
            "{name}"
        );
        h.svc.tamper = None;
        assert_eq!(
            h.disk.counts.catalog_writes, 0,
            "{name}: no catalog sector written"
        );
        assert!(!h.svc.safe_path());
        assert_eq!(
            h.info().0,
            Status::Ok,
            "{name}: not the Corrupt mount state"
        );
        assert_eq!(h.list_ids(0).1, vec![e1]);
        assert_eq!(
            h.append(1, entry, bytes.len() as u64, b"x").status,
            Status::Access
        );
        let e2 = h.export(2, &archive(3, 1, 100, 3));
        assert_eq!(
            h.list_ids(0).1,
            vec![e1, e2],
            "{name}: a later export seals"
        );
    }
}

/// N29 (revision 3, review 4 OP-3): a wrong sealing sequence on the new
/// record that the 2.6 validity rules accept. The committed copy seals its
/// records at 2 and 5 under sequence 9 (valid, last sealing sequence below the
/// sequence); the hook seals the new record at 7 instead of 10. Only the
/// step-4 clause "new record's sealing sequence = new sequence" refuses it.
/// Kills: an extension check without the sealing-sequence clause (writes).
#[test]
fn n29_prewrite_sealing_sequence_accepted_by_validity_rules() {
    let mut h = H::formatted();
    let e1 = h.export(1, &archive(1, 1, 300, 1));
    let e2 = h.export(1, &archive(2, 1, 300, 2));
    let mut image = h.disk.durable.clone();
    let mut catalog = catalog_of(&image, 0, SECTORS).unwrap();
    catalog.records[0].sealed_sequence = 2;
    catalog.records[1].sealed_sequence = 5;
    catalog.sequence = 9;
    let copy = catalog.encode();
    put_copy(&mut image, 0, &copy);
    put_copy(&mut image, 1, &copy);
    let mut h = H::cold(image);
    assert_eq!(
        h.svc.state(),
        State::Mounted,
        "crafted committed copy is valid"
    );
    // The tampered copy itself passes the 2.6 validity rules.
    let mut tampered = catalog;
    tampered.records[2] = tampered.records[1];
    tampered.records[2].id = 3;
    tampered.records[2].first = tampered.records[1].end();
    tampered.records[2].sealed_sequence = 7;
    tampered.count = 3;
    tampered.sequence = 10;
    tampered.next_entry = 4;
    tampered.next_free = tampered.records[2].end();
    assert!(destination_core::catalog::decode(&tampered.encode(), SECTORS).is_some());

    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    let bytes = archive(3, 1, 300, 3);
    let entry = h.write_all(1, &bytes);
    h.svc.tamper = Some(|c| {
        let at = 64 + 80 * 2 + 32;
        c[at..at + 8].copy_from_slice(&7u64.to_le_bytes());
        resum(c);
    });
    h.disk.arm();
    assert_eq!(h.seal(1, entry, bytes.len() as u64).status, Status::Corrupt);
    h.svc.tamper = None;
    assert_eq!(h.disk.counts.catalog_writes, 0, "no catalog sector written");
    assert!(!h.svc.safe_path());
    assert_eq!(h.info().0, Status::Ok, "not the Corrupt mount state");
    assert_eq!(h.list_ids(0).1, vec![e1, e2]);
    assert_eq!(
        h.append(1, entry, bytes.len() as u64, b"x").status,
        Status::Access,
        "writing entry discarded"
    );
    let e3 = h.export(2, &archive(4, 1, 100, 4));
    assert_eq!(h.list_ids(0).1, vec![e1, e2, e3], "a later export seals");
    assert_eq!(h.svc.catalog().find(e3).unwrap().sealed_sequence, 10);
}

/// F1 (review R2): a mounted catalog whose sequence or next entry ID is
/// u64::MAX passes the 2.6 validator, but SEAL cannot increment it. SEAL
/// answers `Corrupt`, writes no catalog sector, discards the writing entry
/// and does not panic in an overflow-checked (test/dev) build.
/// Kills: unchecked `sequence + 1` or `next_entry + 1` in SEAL step 4.
#[test]
fn f1_seal_with_saturated_catalog_counters() {
    type Raise = fn(&mut destination_core::catalog::Catalog);
    let cases: [(&str, Raise); 2] = [
        ("sequence u64::MAX", |c| c.sequence = u64::MAX),
        ("next entry ID u64::MAX", |c| c.next_entry = u64::MAX),
    ];
    for (name, raise) in cases {
        let mut h = H::formatted();
        let e1 = h.export(1, &archive(1, 1, 300, 1));
        let mut image = h.disk.durable.clone();
        let mut catalog = catalog_of(&image, 0, SECTORS).unwrap();
        raise(&mut catalog);
        let copy = catalog.encode();
        put_copy(&mut image, 0, &copy);
        put_copy(&mut image, 1, &copy);
        let mut h = H::cold(image);
        assert_eq!(h.svc.state(), State::Mounted, "{name}: mounts");
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        let bytes = archive(2, 1, 600, 2);
        let entry = h.write_all(1, &bytes);
        let before = h.disk.visible();
        h.disk.arm();
        assert_eq!(
            h.seal(1, entry, bytes.len() as u64).status,
            Status::Corrupt,
            "{name}"
        );
        assert_eq!(h.disk.counts.catalog_writes, 0, "{name}: no catalog write");
        assert_eq!(h.disk.visible()[..8], before[..8], "{name}: catalog intact");
        assert!(!h.svc.safe_path(), "{name}: no device failure");
        assert_eq!(
            h.append(1, entry, bytes.len() as u64, b"x").status,
            Status::Access,
            "{name}: writing entry discarded"
        );
        assert_eq!(h.list_ids(0).1, vec![e1], "{name}");
        // N29 (review 4 RQ-2): such a catalog never commits again; a second
        // export runs on a new binding (lane 1's CREATE is used up).
        let info = h.info();
        assert_eq!(info.0, Status::Ok, "{name}: not the Corrupt mount state");
        assert_eq!(h.bind(2, wire::BIND_EXPORT, 0), Status::Ok);
        let bytes = archive(3, 1, 100, 3);
        let entry = h.write_all(2, &bytes);
        h.disk.arm();
        assert_eq!(
            h.seal(2, entry, bytes.len() as u64).status,
            Status::Corrupt,
            "{name}: second SEAL"
        );
        assert_eq!(h.disk.counts.catalog_writes, 0, "{name}: no catalog write");
        assert_eq!(
            h.append(2, entry, bytes.len() as u64, b"x").status,
            Status::Access,
            "{name}: second writing entry discarded"
        );
        // INFO: status, free sectors, entry count, ID and formatted count
        // (lane generations change with the new binding).
        let after = h.info();
        assert_eq!(
            (after.0, after.1.aux, after.1.count, &after.2[..24]),
            (info.0, info.1.aux, info.1.count, &info.2[..24]),
            "{name}: INFO unchanged"
        );
        assert_eq!(h.list_ids(0).1, vec![e1], "{name}: LIST unchanged");
    }
}

/// N36 (a1), (c): another lane writing: CREATE with a content-invalid label
/// is `Invalid`, then a valid label `Busy`; after the other entry is sealed,
/// CREATE on the same binding is `Ok` (proposed O15: a refused CREATE does
/// not use up the binding; the code follows it).
/// Kills: a CREATE order that reports `Busy` before a bad label; under O15,
/// a refused CREATE that uses up the binding.
#[test]
fn n36_create_order_while_writing() {
    let mut h = H::formatted();
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    assert_eq!(h.bind(2, wire::BIND_EXPORT, 0), Status::Ok);
    let bytes = archive(1, 1, 300, 1);
    let entry = h.write_all(1, &bytes);
    assert_eq!(
        h.create(2, b"..").status,
        Status::Invalid,
        "(a1) label first"
    );
    assert_eq!(h.create(2, b"two").status, Status::Busy, "(a1)");
    assert_eq!(h.seal(1, entry, bytes.len() as u64).status, Status::Ok);
    assert_eq!(h.create(2, b"two").status, Status::Ok, "(c) under O15");
}

/// N36 (a2): nothing writing and no free sector, and separately 16 sealed
/// entries: CREATE with a content-invalid label is `Invalid`, then `Full`.
/// Kills: a CREATE order that reports `Full` before a bad label.
#[test]
fn n36_create_order_when_full() {
    // No free sector: one 8-sector entry fills a 24-sector scope.
    let mut h = H::boot(SimDisk::new(24));
    assert_eq!(h.control(req(wire::FORMAT), &[]).status, Status::Ok);
    h.export(1, &archive(1, 1, 4016, 1));
    assert_eq!(h.svc.catalog().next_free, 24);
    // Sixteen sealed one-sector entries.
    let mut sixteen = H::formatted();
    for index in 0..16 {
        sixteen.export(1, &archive(index + 1, 1, 0, index as u8));
    }
    assert_eq!(sixteen.svc.catalog().count, 16);
    for (name, mut h) in [("no free sector", h), ("16 sealed", sixteen)] {
        assert_eq!(h.bind(2, wire::BIND_EXPORT, 0), Status::Ok);
        assert_eq!(h.create(2, b"..").status, Status::Invalid, "{name}");
        assert_eq!(h.create(2, b"ok").status, Status::Full, "{name}");
    }
}

/// N36 (b) (2.3, revision 3, L8): an APPEND past the largest v1 archive
/// (525 328 bytes) is `Range` -3, appends nothing and leaves the entry
/// writing; a wrong offset is still `Stale` first. With a 1042-sector scope
/// the same APPEND also passes the scope end, so `Range` precedes `Full`;
/// with 1043 sectors the entry can still be completed to the bound.
/// Kills: `Full` instead of `Range` for the archive bound; the scope-end
/// check before the archive bound; a refusal that appends or discards.
#[test]
fn n36_append_past_largest_archive() {
    const APPENDED: u64 = 525_280;
    for scope in [1042, 1043] {
        let mut h = H::boot(SimDisk::new(scope));
        assert_eq!(h.control(req(wire::FORMAT), &[]).status, Status::Ok);
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        let entry = h.write_all(1, &vec![0x5a; APPENDED as usize]);
        let writes = h.disk.writes;
        assert_eq!(
            h.append(1, entry, APPENDED, &[1; 160]).status,
            Status::Range,
            "scope {scope}"
        );
        assert_eq!(h.disk.writes, writes, "scope {scope}: nothing appended");
        assert_eq!(
            h.append(1, entry, APPENDED + 1, &[1; 160]).status,
            Status::Stale,
            "scope {scope}: offset first"
        );
        // Still writing, and the appended count is unchanged.
        let last = h.append(1, entry, APPENDED, &[1; 48]);
        if scope == 1043 {
            assert_eq!(last.status, Status::Ok);
            assert_eq!(wire::header(&last.reply.unwrap()).offset, 525_328);
        } else {
            assert_eq!(last.status, Status::Full, "scope end");
            let mut abort = req(wire::ABORT);
            abort.entry = entry;
            assert_eq!(h.portal(1, abort, &[]).status, Status::Ok, "still writing");
        }
    }
}
