//! Cross-check of the `seeded-read` image of `tools/volume_disk.py` against
//! volume-core: the Python-encoded table (committed as sparse runs, kept in
//! sync by `tests/test_volume_seeded_read.py`) plus the fixture content mounts
//! read-only, serves STAT and READ in domain 0, refuses a mutation with
//! `Access` and issues no device write or flush.
mod common;

use common::{pattern, Local, SimDisk};
use volume_core::{
    client::Client, service::Service, Status, CHUNK, CHUNK_SECTORS, DATA_START, PER_DOMAIN, SECTOR,
    VOLUME_SECTORS,
};

const EPOCH: u64 = 7;
/// (object, name, length), as `SEED_READ` in tools/volume_disk.py.
const SEED: [(u64, &[u8], usize); 2] = [(0, b"seed-a.txt", CHUNK + 100), (1, b"seed-b.txt", 300)];

fn image() -> Vec<[u8; SECTOR]> {
    let mut image = vec![[0; SECTOR]; VOLUME_SECTORS as usize];
    let runs = include_str!("fixtures/seeded-read/table.txt");
    for line in runs.lines() {
        let mut fields = line.split(' ');
        let sector: usize = fields.next().unwrap().parse().unwrap();
        let offset: usize = fields.next().unwrap().parse().unwrap();
        let hex = fields.next().unwrap();
        assert!(sector < DATA_START as usize, "table runs only");
        for index in 0..hex.len() / 2 {
            image[sector][offset + index] =
                u8::from_str_radix(&hex[2 * index..2 * index + 2], 16).unwrap();
        }
    }
    // Content chunks in seed order, as `seed_read` writes them.
    let mut chunk = 0;
    for (object, _, length) in SEED {
        for start in (0..length).step_by(CHUNK) {
            let first = (DATA_START + chunk * CHUNK_SECTORS) as usize * SECTOR;
            for offset in start..length.min(start + CHUNK) {
                let at = first + offset - start;
                image[at / SECTOR][at % SECTOR] = pattern(object, 1, offset);
            }
            chunk += 1;
        }
    }
    image
}

#[test]
fn seeded_read_mounts_read_only_and_serves_domain_zero() {
    let mut disk = SimDisk::from_image(image());
    let mut service = Service::recover(EPOCH, true, &mut disk).unwrap();
    let mut client = Client::new(
        Local {
            service: &mut service,
            disk: &mut disk,
            domain: 0,
        },
        EPOCH,
        1,
    );
    for (object, name, length) in SEED {
        let stat = client.stat(object).unwrap();
        assert_eq!((stat.revision, stat.length), (1, length));
        assert_eq!(&stat.name[..stat.name_length], name);
        let mut out = vec![0; length];
        let read = client
            .read(object, 1, length, |offset, bytes| {
                out[offset..offset + bytes.len()].copy_from_slice(bytes)
            })
            .unwrap();
        assert_eq!(read, length);
        assert!(out
            .iter()
            .enumerate()
            .all(|(offset, byte)| *byte == pattern(object, 1, offset)));
    }
    for object in 2..PER_DOMAIN as u64 {
        assert_eq!(
            client.stat(object).err(),
            Some((Status::NotFound, 0)),
            "object {object}"
        );
    }
    assert_eq!(client.delete(0, 1), Err((Status::Access, 0)));
    assert_eq!(client.stat(0).unwrap().revision, 1);
    assert_eq!(
        disk.mutating_ops, 0,
        "a read-only mount writes and flushes nothing"
    );
}
