//! Host-only independent-volume copy using the existing volume protocol.
mod common;

use common::{fill, pattern, recover, Local, SimDisk};
use volume_core::{
    client::{Client, Stat},
    service::Service,
    table::Table,
    Status, CHUNK, MAX_LENGTH,
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

fn read(c: &mut Client<Local<'_>>, object: u64, stat: &Stat) -> Result<Vec<u8>, Status> {
    let mut bytes = vec![0; stat.length];
    let count = c.read(object, stat.revision, stat.length, |offset, chunk| {
        bytes[offset..offset + chunk.len()].copy_from_slice(chunk);
    })?;
    assert_eq!(count, stat.length);
    Ok(bytes)
}

fn copy(
    source: &mut Client<Local<'_>>,
    destination: &mut Client<Local<'_>>,
    pinned: &Stat,
    target: u64,
    expected: u64,
) -> Result<(u64, u64), Status> {
    // Buffer the complete read before starting a destination upload: write's
    // fill callback cannot propagate a source read error.
    let bytes = read(source, 0, pinned)?;
    destination.write(
        target,
        expected,
        &pinned.name[..pinned.name_length],
        bytes.len(),
        |offset, chunk| chunk.copy_from_slice(&bytes[offset..offset + chunk.len()]),
    )
}

#[test]
fn independent_copy_survives_source_loss_and_cold_destination_recovery() {
    let length = 2 * CHUNK + 173;
    let durable = {
        let mut destination_disk = SimDisk::new();
        let mut destination_service = Service::new(EPOCH, Table::EMPTY, false);
        let mut source_disk = SimDisk::new();
        let mut source_service = Service::new(EPOCH, Table::EMPTY, false);
        let mut source = client(&mut source_service, &mut source_disk, 0);
        source.write(0, 0, b"Earlier", 17, fill(0, 1)).unwrap();
        source
            .write(0, 1, b"Independent", length, fill(0, 2))
            .unwrap();
        let pinned = source.stat(0).unwrap();
        assert_eq!(pinned.revision, 2);
        let mut destination = client(&mut destination_service, &mut destination_disk, 1);
        assert_eq!(
            copy(&mut source, &mut destination, &pinned, 8, 0)
                .unwrap()
                .0,
            1
        );
        assert!(destination_disk.pending.is_empty());
        destination_disk.durable.clone()
    };
    // Both original services, disks, clients and the copy buffer are gone.
    let table = recover(durable.clone()).unwrap();
    let mut disk = SimDisk::from_image(durable);
    let mut service = Service::new(EPOCH + 1, table, true);
    let mut destination = Client::new(
        Local {
            service: &mut service,
            disk: &mut disk,
            domain: 1,
        },
        EPOCH + 1,
        1,
    );
    let stat = destination.stat(8).unwrap();
    assert_eq!(&stat.name[..stat.name_length], b"Independent");
    assert_eq!((stat.revision, stat.length), (1, length));
    let expected: Vec<_> = (0..length).map(|offset| pattern(0, 2, offset)).collect();
    assert_eq!(read(&mut destination, 8, &stat).unwrap(), expected);
}

#[test]
fn stale_source_does_not_start_or_overwrite_destination() {
    let mut source_disk = SimDisk::new();
    let mut source_service = Service::new(EPOCH, Table::EMPTY, false);
    let mut source = client(&mut source_service, &mut source_disk, 0);
    source.write(0, 0, b"Source", 321, fill(0, 1)).unwrap();
    let pinned = source.stat(0).unwrap();
    source.write(0, 1, b"Changed", 321, fill(0, 2)).unwrap();

    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut destination = client(&mut service, &mut disk, 1);
    destination.write(8, 0, b"Keep", 71, fill(8, 1)).unwrap();
    let before = destination.transport.disk.durable.clone();
    let operations = destination.transport.disk.log.len();
    assert_eq!(
        copy(&mut source, &mut destination, &pinned, 8, 1),
        Err(Status::Stale)
    );
    assert_eq!(destination.transport.disk.log.len(), operations);
    assert_eq!(destination.transport.disk.durable, before);
    assert!(destination.transport.disk.pending.is_empty());
    let stat = destination.stat(8).unwrap();
    assert_eq!(&stat.name[..stat.name_length], b"Keep");
    assert_eq!(stat.revision, 1);
    assert_eq!(
        read(&mut destination, 8, &stat).unwrap(),
        (0..71)
            .map(|offset| pattern(8, 1, offset))
            .collect::<Vec<_>>()
    );
}

#[test]
fn quota_and_full_destination_preserve_existing_copy() {
    for full in [false, true] {
        let mut source_disk = SimDisk::new();
        let mut source_service = Service::new(EPOCH, Table::EMPTY, false);
        let mut source = client(&mut source_service, &mut source_disk, 0);
        let length = if full { 1 } else { MAX_LENGTH };
        source.write(0, 0, b"Source", length, fill(0, 1)).unwrap();
        let pinned = source.stat(0).unwrap();

        let mut disk = SimDisk::new();
        let mut service = Service::new(EPOCH, Table::EMPTY, false);
        if full {
            let mut other = client(&mut service, &mut disk, 0);
            other.write(0, 0, b"A", MAX_LENGTH, fill(0, 1)).unwrap();
            other.write(1, 0, b"B", MAX_LENGTH, fill(1, 1)).unwrap();
            other.write(2, 0, b"C", 8 * CHUNK, fill(2, 1)).unwrap();
        }
        let mut destination = client(&mut service, &mut disk, 1);
        destination
            .write(8, 0, b"Keep", MAX_LENGTH, fill(8, 1))
            .unwrap();
        destination
            .write(9, 0, b"Other", 8 * CHUNK, fill(9, 1))
            .unwrap();
        if !full {
            destination
                .write(10, 0, b"More", 8 * CHUNK, fill(10, 1))
                .unwrap();
        }
        let before = destination.transport.disk.durable.clone();
        let operations = destination.transport.disk.log.len();
        assert_eq!(
            copy(&mut source, &mut destination, &pinned, 8, 1),
            Err(if full { Status::Full } else { Status::Quota })
        );
        assert_eq!(destination.transport.disk.log.len(), operations);
        assert_eq!(destination.transport.disk.durable, before);
        assert!(destination.transport.disk.pending.is_empty());
        let stat = destination.stat(8).unwrap();
        assert_eq!(&stat.name[..stat.name_length], b"Keep");
        assert_eq!((stat.revision, stat.length), (1, MAX_LENGTH));
        assert_eq!(
            read(&mut destination, 8, &stat).unwrap(),
            (0..MAX_LENGTH)
                .map(|offset| pattern(8, 1, offset))
                .collect::<Vec<_>>()
        );
    }
}
