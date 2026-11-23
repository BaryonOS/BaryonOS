//! Cross-check fixture for the independent Python oracle (spec 8.2 R3):
//! writes images made by this host core, each with the state a cold boot of
//! the core reaches, into `$DEST_ORACLE_OUT`, for
//! `tests/test_volume_destination_disk.py` to parse. Without the variable the
//! test writes nothing and passes.
mod common;

use common::*;
use destination_core::{
    catalog::{self, Catalog, Record, COPY},
    fnv64,
    service::State,
    wire, Status, DATA_START, FNV_OFFSET, LABEL, SECTOR,
};
use std::{fmt::Write as _, fs, path::Path};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}

/// Write `<name>.img` and `<name>.expect` (what a cold boot of the core sees).
fn emit(dir: &Path, name: &str, image: Vec<[u8; SECTOR]>) {
    let h = H::cold(image.clone());
    let state = match h.svc.state() {
        State::Mounted => "mounted",
        State::Blank => "blank",
        State::Corrupt => "corrupt",
        State::Failed { .. } => "failed",
    };
    let mut expect = format!("state={state}\n");
    if h.svc.state() == State::Mounted {
        let c = h.svc.catalog();
        let _ = writeln!(
            expect,
            "id={}\nsequence={}\nnext_entry={}\nnext_free={}\nformatted={}",
            hex(&c.id),
            c.sequence,
            c.next_entry,
            c.next_free,
            c.formatted
        );
        for r in c.entries() {
            let _ = writeln!(
                expect,
                "record={},{},{},{},{},{}",
                r.id,
                r.first,
                r.length,
                r.trailer,
                r.sealed_sequence,
                hex(&r.label[..r.label_length as usize])
            );
        }
    }
    fs::write(dir.join(format!("{name}.img")), image.concat()).unwrap();
    fs::write(dir.join(format!("{name}.expect")), expect).unwrap();
}

fn resum(copy: &mut [u8; COPY]) {
    let sum = fnv64(FNV_OFFSET, &copy[..2040]);
    copy[2040..].copy_from_slice(&sum.to_le_bytes());
}

/// A `scope`-sector image with `copy` in both slots.
fn both_slots(scope: u64, copy: &[u8; COPY]) -> Vec<[u8; SECTOR]> {
    let mut image = vec![[0; SECTOR]; scope as usize];
    put_copy(&mut image, 0, copy);
    put_copy(&mut image, 1, copy);
    image
}

/// A valid two-record copy for a scope of `scope` sectors (entries of
/// `lengths` bytes, labels "one" and "two", sequence 3), then `tweak` at the
/// byte offsets of 2.6 and a renewed checksum: one rule broken, in both slots.
fn crafted(scope: u64, lengths: [u64; 2], tweak: impl Fn(&mut [u8; COPY])) -> Vec<[u8; SECTOR]> {
    let mut c = Catalog::formatted(TEST_ID, scope);
    for (index, length) in lengths.into_iter().enumerate() {
        let mut label = [0; LABEL];
        label[..3].copy_from_slice([b"one", b"two"][index]);
        c.records[index] = Record {
            id: index as u64 + 1,
            first: c.next_free,
            length,
            trailer: 0x1111,
            sealed_sequence: index as u64 + 2,
            label_length: 3,
            label,
        };
        c.count += 1;
        c.next_free = c.records[index].end();
    }
    c.sequence = 3;
    c.next_entry = 3;
    let mut copy = c.encode();
    assert!(catalog::decode(&copy, scope).is_some(), "base copy valid");
    tweak(&mut copy);
    resum(&mut copy);
    assert!(
        catalog::decode(&copy, scope).is_none(),
        "crafted copy invalid"
    );
    both_slots(scope, &copy)
}

/// E1 sealed, E2 written; SEAL of E2 with a power cut at mutating request `k`.
fn seal_cut(k: usize) -> H {
    let mut h = H::formatted();
    h.export(1, &archive(1, 2, 700, 3));
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    let e2 = archive(2, 1, 1020, 9);
    let entry = h.write_all(1, &e2);
    h.disk.arm();
    h.disk.crash_at = Some(k);
    h.seal(1, entry, e2.len() as u64);
    h
}

#[test]
fn oracle_images() {
    let Some(out) = std::env::var_os("DEST_ORACLE_OUT") else {
        return;
    };
    let dir = Path::new(&out);
    fs::create_dir_all(dir).unwrap();

    let mut h = H::formatted();
    emit(dir, "formatted", h.disk.durable.clone());
    h.export(1, &archive(1, 2, 700, 3));
    emit(dir, "one_entry", h.disk.durable.clone());
    h.export(1, &archive(2, 1, 1100, 9));
    emit(dir, "two_entries", h.disk.durable.clone());

    // The first cut point whose durable image holds two valid, different
    // copies: after barrier 2, before barrier 3 (found, not hardcoded).
    let k = (0..16)
        .find(|&k| {
            let image = seal_cut(k).disk.durable.clone();
            match (
                catalog_of(&image, 0, SECTORS),
                catalog_of(&image, 1, SECTORS),
            ) {
                (Some(a), Some(b)) => a.sequence != b.sequence,
                _ => false,
            }
        })
        .expect("a cut between barriers 2 and 3");
    emit(dir, "between_barriers", seal_cut(k).disk.durable.clone());

    // Two mirror writes land, the rest is lost: one valid copy, one torn.
    let image = seal_cut(k + 2).disk.crash_subset(|_| true);
    assert!(catalog_of(&image, 0, SECTORS).is_some() != catalog_of(&image, 1, SECTORS).is_some());
    emit(dir, "mirror_torn", image);

    // FORMAT cut at its first flush with only sector 0 of copy A durable.
    let mut f = H::blank();
    f.disk.arm();
    f.disk.crash_at = Some(4);
    f.control(req(wire::FORMAT), &[]);
    let image = f.disk.crash_subset(|index| index == 0);
    emit(dir, "format_torn", image);

    emit(dir, "blank", vec![[0; SECTOR]; SECTORS as usize]);

    // N34 (revision 3): both copies of the two-entry image crafted to break
    // one record rule (record 0 or 1 at byte `at`), and the valid copy beside
    // non-zero reserved sectors 8-15.
    let valid = h.disk.durable.clone();
    for (name, record, at, value) in [
        ("n34_entry_id_0", 0, 0, 0u64),
        ("n34_label_length_0", 1, 40, 0),
        ("n34_sealing_sequence_1", 0, 32, 1),
        ("n34_sealing_sequence_above_copy", 1, 32, 4),
        ("n34_equal_sealing_sequences", 1, 32, 2),
    ] {
        let mut copy = copy_of(&valid, 0);
        let offset = 64 + record * 80 + at;
        if at == 40 {
            copy[offset] = value as u8;
            copy[offset + 8..offset + 40].fill(0);
        } else {
            copy[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        }
        let sum = fnv64(FNV_OFFSET, &copy[..2040]);
        copy[2040..].copy_from_slice(&sum.to_le_bytes());
        let mut image = valid.clone();
        put_copy(&mut image, 0, &copy);
        put_copy(&mut image, 1, &copy);
        emit(dir, name, image);
    }
    let mut reserved = valid;
    for sector in &mut reserved[8..16] {
        sector.fill(0xa5);
    }
    emit(dir, "n34_reserved_sectors_nonzero", reserved);

    // The other four N34 cases (review F6), each breaking one rule only:
    // lengths within one sector count, and a scope that holds the largest
    // v1 archive for the 525 329-byte case.
    let mut zero_records = Catalog::formatted(TEST_ID, SECTORS).encode();
    zero_records[32..40].fill(0);
    resum(&mut zero_records);
    assert!(catalog::decode(&zero_records, SECTORS).is_none());
    emit(
        dir,
        "n34_zero_records_sequence_0",
        both_slots(SECTORS, &zero_records),
    );
    const R1: usize = 64 + 80;
    let length = |value: u64| {
        move |copy: &mut [u8; COPY]| copy[R1 + 16..R1 + 24].copy_from_slice(&value.to_le_bytes())
    };
    emit(
        dir,
        "n34_length_79",
        crafted(SECTORS, [600, 80], length(79)),
    );
    let big = DATA_START + 2 + 525_328u64.div_ceil(512);
    emit(
        dir,
        "n34_length_525329",
        crafted(big, [600, 525_328], length(525_329)),
    );
    emit(
        dir,
        "n34_label_length_33",
        crafted(SECTORS, [600, 80], |copy| {
            copy[R1 + 40] = 33;
            copy[R1 + 48..R1 + 80].fill(b'x');
        }),
    );
    // O14: label content (2.3 `valid_name`) at mount.
    let labels: [(&str, &[u8]); 13] = [
        ("dot", b"."),
        ("dot_dot", b".."),
        ("invalid_utf8", b"a\xff\xfe"),
        ("surrogate", b"\xed\xa0\x80"),
        ("control", b"a\x07b"),
        ("del", b"a\x7fb"),
        ("u0085", "a\u{85}b".as_bytes()),
        ("slash", b"a/b"),
        ("backslash", b"a\\b"),
        ("u202a", "x\u{202a}".as_bytes()),
        ("u202e", "x\u{202e}".as_bytes()),
        ("u2066", "x\u{2066}".as_bytes()),
        ("u2069", "x\u{2069}".as_bytes()),
    ];
    for (name, label) in labels {
        let image = crafted(SECTORS, [600, 80], |copy| {
            copy[R1 + 40] = label.len() as u8;
            copy[R1 + 48..R1 + 80].fill(0);
            copy[R1 + 48..R1 + 48 + label.len()].copy_from_slice(label);
        });
        emit(dir, &format!("n34_label_{name}"), image);
    }
}
