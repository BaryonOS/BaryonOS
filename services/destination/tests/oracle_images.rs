//! Cross-check fixture for the independent Python oracle (spec 8.2 R3):
//! writes images made by this host core, each with the state a cold boot of
//! the core reaches, into `$DEST_ORACLE_OUT`, for
//! `tests/test_volume_destination_disk.py` to parse. Without the variable the
//! test writes nothing and passes.
mod common;

use common::*;
use destination_core::{fnv64, service::State, wire, Status, FNV_OFFSET, SECTOR};
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
            let image = seal_cut(k).disk.durable;
            match (
                catalog_of(&image, 0, SECTORS),
                catalog_of(&image, 1, SECTORS),
            ) {
                (Some(a), Some(b)) => a.sequence != b.sequence,
                _ => false,
            }
        })
        .expect("a cut between barriers 2 and 3");
    emit(dir, "between_barriers", seal_cut(k).disk.durable);

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
}
