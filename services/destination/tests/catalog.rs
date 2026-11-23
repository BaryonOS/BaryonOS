//! Catalog layout, validator and mount rule (spec 2.6): N4, N8, N30.
//!
//! Crafted copies are built by an encoder written here from the byte offsets
//! of 2.6, independent of the library encoder, so a layout error in the
//! library cannot cancel out. `golden_layout` ties the two together.
mod common;

use common::*;
use destination_core::{
    catalog::{self, Catalog, Mount, Record},
    device, fnv64,
    service::State,
    wire, Status, FNV_OFFSET, SECTOR,
};

/// Independent encoder: (id, first, length, label) records.
struct Raw {
    count: u8,
    id: [u8; 16],
    sequence: u64,
    next_entry: u64,
    next_free: u64,
    formatted: u64,
    records: Vec<(u64, u64, u64, &'static [u8])>,
}

impl Raw {
    fn new() -> Self {
        Self {
            count: 0,
            id: TEST_ID,
            sequence: 1,
            next_entry: 1,
            next_free: 16,
            formatted: SECTORS,
            records: Vec::new(),
        }
    }
    /// Two contiguous entries: 600 bytes (2 sectors) and 80 bytes (1 sector).
    fn two() -> Self {
        let mut raw = Self::new();
        raw.count = 2;
        raw.sequence = 3;
        raw.next_entry = 3;
        raw.next_free = 19;
        raw.records = vec![(1, 16, 600, b"one"), (2, 18, 80, b"two")];
        raw
    }
    fn bytes(&self) -> [u8; 2048] {
        let mut b = [0u8; 2048];
        b[0..8].copy_from_slice(b"NOSDST\x00\x01");
        b[8..10].copy_from_slice(&1u16.to_le_bytes());
        b[10] = self.count;
        b[16..32].copy_from_slice(&self.id);
        b[32..40].copy_from_slice(&self.sequence.to_le_bytes());
        b[40..48].copy_from_slice(&self.next_entry.to_le_bytes());
        b[48..56].copy_from_slice(&self.next_free.to_le_bytes());
        b[56..64].copy_from_slice(&self.formatted.to_le_bytes());
        for (index, (id, first, length, label)) in self.records.iter().enumerate() {
            let at = 64 + index * 80;
            b[at..at + 8].copy_from_slice(&id.to_le_bytes());
            b[at + 8..at + 16].copy_from_slice(&first.to_le_bytes());
            b[at + 16..at + 24].copy_from_slice(&length.to_le_bytes());
            b[at + 24..at + 32].copy_from_slice(&0x1111u64.to_le_bytes());
            b[at + 32..at + 40].copy_from_slice(&(index as u64 + 2).to_le_bytes());
            b[at + 40] = label.len() as u8;
            b[at + 48..at + 48 + label.len()].copy_from_slice(label);
        }
        resum(&mut b);
        b
    }
}

fn resum(b: &mut [u8; 2048]) {
    let sum = fnv64(FNV_OFFSET, &b[..2040]);
    b[2040..2048].copy_from_slice(&sum.to_le_bytes());
}

fn valid(b: &[u8; 2048]) -> bool {
    catalog::decode(b, SECTORS).is_some()
}

#[test]
fn golden_layout() {
    let raw = Raw::two();
    let decoded = catalog::decode(&raw.bytes(), SECTORS).expect("valid crafted copy");
    assert_eq!(decoded.count, 2);
    assert_eq!(decoded.records[1].first, 18);
    assert_eq!(decoded.records[0].label_length, 3);
    assert_eq!(
        decoded.encode(),
        raw.bytes(),
        "library encoder matches 2.6 offsets"
    );
}

/// Validity rules of 2.6, one violation at a time (with
/// `validator_rejects_isolated_rules`). Kills: dropping any non-redundant
/// rule from `catalog::decode`; `next_free >= 16` is implied by
/// `next_free` = end of the extents and is an equivalent mutant.
#[test]
fn validator_rejects_each_rule() {
    let base = Raw::two().bytes();
    assert!(valid(&base));
    type Tweak = Box<dyn Fn(&mut [u8; 2048])>;
    let cases: Vec<(&str, Tweak)> = vec![
        ("magic", Box::new(|b| b[5] ^= 1)),
        ("version", Box::new(|b| b[8] = 2)),
        ("count > 16", Box::new(|b| b[10] = 17)),
        ("count too low", Box::new(|b| b[10] = 1)),
        ("header padding", Box::new(|b| b[13] = 1)),
        ("sequence 0", Box::new(|b| b[32..40].fill(0))),
        ("next entry 0", Box::new(|b| b[40..48].fill(0))),
        (
            "id >= next entry",
            Box::new(|b| b[40..48].copy_from_slice(&2u64.to_le_bytes())),
        ),
        (
            "next_free != end",
            Box::new(|b| b[48..56].copy_from_slice(&20u64.to_le_bytes())),
        ),
        (
            "formatted != scope",
            Box::new(|b| b[56..64].copy_from_slice(&95u64.to_le_bytes())),
        ),
        ("entry id 0", Box::new(|b| b[64..72].fill(0))),
        (
            "ids not ascending",
            Box::new(|b| b[144..152].copy_from_slice(&1u64.to_le_bytes())),
        ),
        (
            "first != 16",
            Box::new(|b| b[72..80].copy_from_slice(&17u64.to_le_bytes())),
        ),
        (
            "gap",
            Box::new(|b| b[152..160].copy_from_slice(&19u64.to_le_bytes())),
        ),
        (
            "overlap",
            Box::new(|b| b[152..160].copy_from_slice(&17u64.to_le_bytes())),
        ),
        (
            "length < 80",
            Box::new(|b| b[160..168].copy_from_slice(&79u64.to_le_bytes())),
        ),
        (
            "length > max",
            Box::new(|b| b[80..88].copy_from_slice(&525_329u64.to_le_bytes())),
        ),
        ("label length 0", Box::new(|b| b[104] = 0)),
        ("label length 33", Box::new(|b| b[104] = 33)),
        ("label padding", Box::new(|b| b[112 + 10] = b'x')),
        ("record padding", Box::new(|b| b[106] = 1)),
        ("unused record", Box::new(|b| b[64 + 2 * 80 + 3] = 1)),
        ("tail padding", Box::new(|b| b[1500] = 1)),
    ];
    for (name, tweak) in cases {
        let mut bytes = base;
        tweak(&mut bytes);
        resum(&mut bytes);
        assert!(!valid(&bytes), "{name} must be invalid");
    }
    let mut bytes = base;
    bytes[2041] ^= 1;
    assert!(!valid(&bytes), "checksum");
}

/// Rules that the single-byte tweaks above cannot isolate, because another
/// rule fires first. Each case breaks exactly one rule.
#[test]
fn validator_rejects_isolated_rules() {
    // `next_free` > formatted count, with `next_free` = end of the extents
    // and formatted count = scope.
    let mut raw = Raw::two();
    raw.formatted = 18;
    assert!(
        catalog::decode(&raw.bytes(), 18).is_none(),
        "next_free > formatted"
    );
    // Length above the largest v1 archive, extents still contiguous.
    let big = |length: u64| {
        let mut raw = Raw::two();
        raw.records[1].2 = length;
        raw.next_free = 18 + length.div_ceil(512);
        raw.formatted = 2000;
        catalog::decode(&raw.bytes(), 2000)
    };
    assert!(big(525_328).is_some(), "largest v1 archive is valid");
    assert!(big(525_329).is_none(), "length > MAX_ARCHIVE");
    // Label length 0 with all label bytes zero.
    let mut raw = Raw::two();
    raw.records[1].3 = b"";
    assert!(!valid(&raw.bytes()), "label length 0");
    // Count 17 over sixteen valid records (would also break `entries()`).
    let mut raw = Raw::new();
    raw.records = (0..16u64).map(|i| (i + 1, 16 + i, 80, &b"l"[..])).collect();
    raw.next_entry = 17;
    raw.next_free = 32;
    raw.count = 16;
    assert!(valid(&raw.bytes()), "sixteen entries are valid");
    raw.count = 17;
    assert!(!valid(&raw.bytes()), "count > 16");
}

/// N4 (crafted catalogs): zero entries with `next_free` < 16, and zero entries
/// with next entry ID 0, are invalid.
/// Kills: a validator that accepts next entry ID 0 when there are no entries.
/// (`next_free` < 16 is also refused by `next_free` = end of the extents, so
/// dropping the explicit rule is an equivalent mutant.)
#[test]
fn n4_crafted_zero_entry_catalogs_invalid() {
    let empty = Raw::new();
    assert!(valid(&empty.bytes()));
    for next_free in [0, 8, 15] {
        let mut raw = Raw::new();
        raw.next_free = next_free;
        assert!(!valid(&raw.bytes()), "next_free {next_free}");
    }
    let mut raw = Raw::new();
    raw.next_entry = 0;
    assert!(!valid(&raw.bytes()));
    let mut raw = Raw::new();
    raw.next_free = SECTORS + 1;
    assert!(!valid(&raw.bytes()));
}

/// N4 (write path): the entry path refuses sectors below 16 and below the
/// committed `next_free`; the catalog path refuses sectors >= 8. Nothing
/// reaches the device.
/// Kills: an entry path that checks only `next_free` (sector 12 with
/// `next_free` 10 is written) or only 16 (sector 30 with `next_free` 40).
#[test]
fn n4_write_paths_refuse_out_of_region() {
    let mut disk = SimDisk::new(SECTORS);
    let data = [0xa5; SECTOR];
    for (next_free, sector) in [(10, 12), (40, 30), (16, 0), (16, 15), (16, SECTORS)] {
        assert_eq!(
            device::write_entry(&mut disk, next_free, SECTORS, sector, &data),
            Err(Status::Access),
            "entry write at {sector} with next_free {next_free}"
        );
    }
    for sector in [8, 15, 16] {
        assert_eq!(
            device::write_catalog(&mut disk, sector, &data),
            Err(Status::Access)
        );
    }
    assert_eq!(disk.writes, 0);
    assert_eq!(
        device::write_entry(&mut disk, 40, SECTORS, 40, &data),
        Ok(())
    );
    assert_eq!(device::write_catalog(&mut disk, 7, &data), Ok(()));
    assert_eq!(disk.writes, 2);
}

/// N4 (service): an export after a sealed entry writes only at or beyond
/// the committed `next_free`; the sealed extent and sectors 0-15 stay
/// byte-identical until SEAL's catalog writes.
#[test]
fn n4_export_never_touches_sealed_extent() {
    let mut h = H::formatted();
    let e1 = h.export(1, &archive(1, 2, 700, 3));
    let before = h.disk.visible();
    let end = h.svc.catalog().find(e1).unwrap().end();
    h.disk.log.clear();
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    h.write_all(1, &archive(2, 1, 900, 9));
    for event in &h.disk.log {
        if let Event::Submit(Op::Write(sector)) = event {
            assert!(*sector >= end, "write at {sector} below next_free {end}");
        }
    }
    let after = h.disk.visible();
    assert_eq!(before[..end as usize], after[..end as usize]);
}

fn two_entry_image() -> (Vec<[u8; SECTOR]>, [u8; 2048], u64, u64) {
    let mut h = H::formatted();
    let e1 = h.export(1, &archive(1, 2, 700, 3));
    let older = copy_of(&h.disk.durable, 1);
    let e2 = h.export(1, &archive(2, 1, 300, 5));
    (h.disk.durable.clone(), older, e1, e2)
}

/// N30: two valid copies with different sequences where the higher does not
/// extend the lower give `Corrupt`; the legitimate pair mounts the higher.
/// Kills: a mount that takes the higher sequence without the extension check.
#[test]
fn n30_mixed_copies() {
    let (image, older, e1, e2) = two_entry_image();
    let newer = catalog::decode(&copy_of(&image, 0), SECTORS).unwrap();
    let lower = catalog::decode(&older, SECTORS).unwrap();

    // Legitimate: crash between barriers 2 and 3.
    let mut legit = image.clone();
    put_copy(&mut legit, 1, &older);
    let mut h = H::cold(legit);
    assert_eq!(h.svc.state(), State::Mounted);
    assert_eq!(h.list_ids(0), (Status::Ok, vec![e1, e2]));

    let variants: Vec<(&str, Catalog)> = vec![
        ("record missing", {
            let mut c = newer;
            c.records[0] = c.records[1];
            c.records[0].first = 16;
            c.records[1] = Record::EMPTY;
            c.count = 1;
            c.next_free = c.records[0].end();
            c
        }),
        ("record changed", {
            let mut c = newer;
            c.records[0].trailer ^= 1;
            c
        }),
        // A `next_free` lower than the lower copy's cannot coexist with an
        // identical record prefix in two valid copies (validity forces
        // `next_free` to the end of the last extent); the reachable twin of
        // that rule is a lower next entry ID.
        ("next entry ID lower", newer),
        ("other destination ID", {
            let mut c = newer;
            c.id[0] ^= 1;
            c
        }),
    ];
    for (name, higher) in variants {
        let mut older = older;
        if name == "next entry ID lower" {
            let mut raised = lower;
            raised.next_entry = newer.next_entry + 4;
            older = raised.encode();
        }
        assert!(higher.sequence > lower.sequence);
        assert!(
            catalog::decode(&higher.encode(), SECTORS).is_some(),
            "{name} valid alone"
        );
        let mut bad = image.clone();
        put_copy(&mut bad, 0, &higher.encode());
        put_copy(&mut bad, 1, &older);
        let mut h = H::cold(bad);
        assert_eq!(h.svc.state(), State::Corrupt, "{name}");
        assert_eq!(h.info().0, Status::Corrupt, "{name}");
        assert_eq!(h.disk.writes, 0);
    }

    // Equal sequences with different bytes.
    let mut same = newer;
    same.records[1].trailer ^= 1;
    let mut bad = image.clone();
    put_copy(&mut bad, 1, &same.encode());
    assert_eq!(H::cold(bad).svc.state(), State::Corrupt);

    // Equal sequences where one copy is a strict extension of the other (one
    // record ahead): still `Corrupt`, in both slot orders, since only the
    // higher sequence may be taken. Kills: equal sequences treated as an
    // ordered pair and then checked with `extends` (M21f).
    let mut behind = lower;
    behind.sequence = newer.sequence;
    let behind = behind.encode();
    let decoded = catalog::decode(&behind, SECTORS).expect("valid alone");
    assert!(catalog::extends(&decoded, &newer));
    for (slot_newer, slot_behind) in [(0, 1), (1, 0)] {
        let mut bad = image.clone();
        put_copy(&mut bad, slot_newer, &newer.encode());
        put_copy(&mut bad, slot_behind, &behind);
        let h = H::cold(bad);
        assert_eq!(
            h.svc.state(),
            State::Corrupt,
            "equal sequence, newer in slot {slot_newer}"
        );
        assert_eq!(h.disk.writes, 0);
    }
}

#[test]
fn mount_rule_single_and_identical_copies() {
    let (image, _, e1, e2) = two_entry_image();
    let mut area = [[0; SECTOR]; 16];
    area.copy_from_slice(&image[..16]);
    assert!(
        matches!(catalog::mount_rule(&area, SECTORS), Mount::Mounted(s) if s.valid == [true, true])
    );
    area[2][7] ^= 1;
    let Mount::Mounted(selection) = catalog::mount_rule(&area, SECTORS) else {
        panic!("one valid copy must mount");
    };
    assert_eq!(selection.valid, [false, true]);
    assert_eq!(selection.target(), 0);
    assert_eq!(selection.catalog.entries().len(), 2);
    let _ = (e1, e2);
}

/// N8: catalog damage, blank candidate and catalog read errors.
/// Kills: a mount that treats a copy with a read error as invalid (selects
/// B), one that treats an unreadable sector as zero (blank candidate), a
/// blank state that answers INFO with `Ok` or BIND with anything but
/// `NotFound`.
#[test]
fn n8_one_copy_damaged_uses_other() {
    let (mut image, _, e1, e2) = two_entry_image();
    image[1][100] ^= 0xff;
    let mut h = H::cold(image);
    assert_eq!(h.svc.state(), State::Mounted);
    assert_eq!(h.list_ids(0), (Status::Ok, vec![e1, e2]));
}

#[test]
fn n8_both_copies_damaged_is_corrupt() {
    let (mut image, _, e1, _) = two_entry_image();
    image[1][100] ^= 0xff;
    image[6][3] ^= 0xff;
    let mut h = H::cold(image);
    assert_eq!(h.svc.state(), State::Corrupt);
    let (status, info, data) = h.info();
    assert_eq!(status, Status::Corrupt);
    assert_eq!((info.aux, info.count), (0, 0));
    assert!(data[..24].iter().all(|b| *b == 0));
    assert_eq!(h.list_ids(0).0, Status::Corrupt);
    assert_eq!(
        h.bind_with(1, wire::BIND_VERIFY, e1, 1, &TEST_ID),
        Status::Corrupt
    );
    assert_eq!(h.control(req(wire::FORMAT), &[]).status, Status::Corrupt);
    assert_eq!(h.disk.writes, 0);
}

#[test]
fn n8_zeroed_catalog_with_entry_data_is_blank_but_unformattable() {
    let (mut image, _, e1, _) = two_entry_image();
    for sector in image.iter_mut().take(16) {
        *sector = [0; SECTOR];
    }
    let mut h = H::cold(image);
    assert_eq!(h.svc.state(), State::Blank);
    let (status, info, data) = h.info();
    assert_eq!(status, Status::NotFound);
    assert_eq!((info.aux, info.count, info.length), (0, 0, 48));
    assert!(
        data[..24].iter().all(|b| *b == 0),
        "destination ID and formatted zero"
    );
    assert_eq!(h.list_ids(0).0, Status::NotFound);
    assert_eq!(
        h.bind_with(1, wire::BIND_VERIFY, e1, 1, &TEST_ID),
        Status::NotFound
    );
    assert_eq!(
        h.bind_with(2, wire::BIND_EXPORT, 0, 1, &[0; 16]),
        Status::NotFound
    );
    h.disk.log.clear();
    assert_eq!(h.control(req(wire::FORMAT), &[]).status, Status::Access);
    assert!(h
        .disk
        .log
        .iter()
        .any(|e| matches!(e, Event::Submit(Op::Read(s)) if *s >= 16)));
    assert_eq!(h.disk.writes, 0);
}

/// A = sequence n+1 {E1, E2}, B = sequence n {E1}, one sector of A unreadable.
fn n8_read_error_case(code: i64, sector: u64) {
    let (mut image, older, e1, e2) = two_entry_image();
    put_copy(&mut image, 1, &older);
    let e2_first = catalog::decode(&copy_of(&image, 0), SECTORS)
        .unwrap()
        .find(e2)
        .unwrap()
        .first;
    let mut disk = SimDisk::from_image(image);
    disk.fault(move |op, _| (op == Op::Read(sector)).then(|| Fault::code(code)));
    let mut h = H::boot(disk);
    assert_eq!(
        h.svc.state(),
        State::Failed { no_device: false },
        "code {code} sector {sector}"
    );
    h.disk.fault = None;
    let (status, info, data) = h.info();
    assert_eq!(status, Status::Io);
    assert_eq!((info.aux, info.count), (0, 0));
    assert!(data[..24].iter().all(|b| *b == 0));
    assert_eq!(h.list_ids(0).0, Status::Io);
    assert_eq!(
        h.bind_with(1, wire::BIND_VERIFY, e1, 1, &TEST_ID),
        Status::Io
    );
    assert_eq!(
        h.bind_with(2, wire::BIND_EXPORT, 0, 1, &TEST_ID),
        Status::Io
    );
    assert_eq!(h.control(req(wire::FORMAT), &[]).status, Status::Io);
    assert_eq!(h.disk.writes, 0, "no device write");
    assert!(h.svc.binding(1).is_none());
    assert!(!h
        .disk
        .log
        .iter()
        .any(|e| matches!(e, Event::Submit(Op::Write(s)) if *s >= e2_first)));
}

#[test]
fn n8_read_error_on_newer_copy_is_failed_mount() {
    for code in [-12, -13, -16] {
        for sector in [0, 3] {
            n8_read_error_case(code, sector);
        }
    }
}

#[test]
fn n8_read_error_in_reserved_area_is_never_blank() {
    for sector in [8, 15] {
        let mut disk = SimDisk::new(SECTORS);
        disk.fault(move |op, _| (op == Op::Read(sector)).then(|| Fault::code(-12)));
        let mut h = H::boot(disk);
        assert_eq!(h.svc.state(), State::Failed { no_device: false });
        assert_eq!(h.info().0, Status::Io);
        assert_eq!(h.control(req(wire::FORMAT), &[]).status, Status::Io);
        assert_eq!(h.disk.writes, 0);
    }
}
