//! Reverse differential for the independent Python oracle (spec 8.2 R3):
//! reads the crafted images that `tests/test_volume_destination_disk.py`
//! (`ReverseDifferential`) writes into `$DEST_REVERSE_IN`, each with the
//! oracle's verdict in `<name>.expect`, mounts every image in this host core
//! by a cold boot and requires the same verdict (mount state, catalog header
//! and records; spec 7 "Cross-check limit"). Without the variable the test
//! reads nothing and passes; the oracle-side corpus test always runs.
mod common;

use common::*;
use destination_core::SECTOR;
use std::{fs, path::Path};

/// Sectors of a crafted image file; a partial last sector is refused, so a
/// truncated corpus fails instead of being padded.
fn sectors_of(bytes: &[u8]) -> Vec<[u8; SECTOR]> {
    let (sectors, rest) = bytes.as_chunks::<SECTOR>();
    assert!(rest.is_empty(), "image is not a whole number of sectors");
    sectors.to_vec()
}

#[test]
fn oracle_crafted_images_mount_with_the_oracle_verdict() {
    let Some(dir) = std::env::var_os("DEST_REVERSE_IN") else {
        return;
    };
    let dir = Path::new(&dir);
    let mut names: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "expect"))
        .collect();
    names.sort();
    // Every image must carry the oracle's verdict: an image without one would
    // otherwise be skipped silently and shrink the corpus unnoticed.
    let images = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "img"))
        .count();
    assert_eq!(
        images,
        names.len(),
        "images without a matching .expect file"
    );
    let (mut mounted, mut refused, mut mismatches) = (0, 0, Vec::new());
    for expect_path in &names {
        let expect = fs::read_to_string(expect_path).unwrap();
        let image = sectors_of(&fs::read(expect_path.with_extension("img")).unwrap());
        let verdict = cold_verdict(&image);
        if verdict.starts_with("state=mounted") {
            mounted += 1;
        } else {
            refused += 1;
        }
        if verdict != expect {
            mismatches.push(format!(
                "{}:\n  oracle: {expect:?}\n  core:   {verdict:?}",
                expect_path.display()
            ));
        }
    }
    // The corpus has more than 100 images with both outcomes; a short or
    // one-sided corpus means the generator did not run as intended.
    assert!(
        names.len() >= 100,
        "only {} images in {}",
        names.len(),
        dir.display()
    );
    assert!(
        mounted > 0 && refused > 0,
        "{mounted} mounted, {refused} refused"
    );
    assert!(
        mismatches.is_empty(),
        "{} disagreements:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}
