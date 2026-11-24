//! Host tests against the real volume-core service and an in-memory disk.
extern crate std;
use std::vec;
use std::vec::Vec;

use super::*;
use nieuw_os_settings_core::SettingsStore;
use volume_core::{service::Service, table::Table, wire, Disk, SECTOR, VOLUME_SECTORS};

const EPOCH: u64 = 7;

struct Memory(Vec<[u8; SECTOR]>);

impl Disk for Memory {
    fn read(&mut self, sector: u64, buffer: &mut [u8; SECTOR]) -> Result<(), Status> {
        *buffer = *self.0.get(sector as usize).ok_or(Status::Range)?;
        Ok(())
    }
    fn write(&mut self, sector: u64, buffer: &[u8; SECTOR]) -> Result<(), Status> {
        *self.0.get_mut(sector as usize).ok_or(Status::Range)? = *buffer;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Status> {
        Ok(())
    }
}

/// The service as seen from one kernel-bound domain; optionally refuses COMMIT.
struct Local {
    service: Service,
    disk: Memory,
    domain: usize,
    fail_commit: bool,
}

impl Transport for Local {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        if self.fail_commit && wire::header(request).operation == wire::COMMIT {
            return Err(Status::Io);
        }
        Ok(self.service.handle(self.domain, request, &mut self.disk))
    }
}

fn media(domain: usize) -> VolumeMedia<Local> {
    let local = Local {
        service: Service::new(EPOCH, Table::EMPTY, false),
        disk: Memory(vec![[0; SECTOR]; VOLUME_SECTORS as usize]),
        domain,
        fail_commit: false,
    };
    VolumeMedia::new(Client::new(local, EPOCH, 1), domain)
}

/// Same disk and service, as another domain.
fn rebind(media: VolumeMedia<Local>, domain: usize) -> VolumeMedia<Local> {
    let mut local = media.client.transport;
    local.domain = domain;
    VolumeMedia::new(Client::new(local, EPOCH, 50_000), domain)
}

#[test]
fn absent_objects_read_as_blank_and_recover_refuses() {
    let mut media = media(0);
    let mut slot = [0xaa; SLOT_LEN];
    assert_eq!(media.read(0, &mut slot), Ok(()));
    assert_eq!(slot, [0; SLOT_LEN]);
    assert_eq!(media.read(2, &mut slot), Err(MediaError::Io));
    assert_eq!(
        SettingsStore::new().recover(&mut media),
        Err(Error::NotInitialized)
    );
    assert_eq!(media.flush(), Ok(()));
}

#[test]
fn initialize_set_recover_revert_survive_new_stores() {
    let mut media = media(0);
    let mut store = SettingsStore::new();
    assert_eq!(store.initialize(&mut media, first(0)), Ok(1));
    assert_eq!(store.set(&mut media, second(0)), Ok(2));
    let mut again = SettingsStore::new();
    assert_eq!(again.recover(&mut media), Ok(2));
    assert_eq!(again.get(), Ok(second(0)));
    assert_eq!(again.revert(&mut media), Ok(3));
    let mut third = SettingsStore::new();
    assert_eq!(third.recover(&mut media), Ok(3));
    assert_eq!(third.get(), Ok(first(0)));
    // Slot objects are exactly objects 0 and 1 of domain 0.
    let table = media.client.transport.service.table();
    assert_eq!(table.entries[0].revision, 2);
    assert_eq!(table.entries[1].revision, 1);
    assert!(table.entries[2..].iter().all(|e| e.revision == 0));
}

#[test]
fn wrong_length_object_is_refused() {
    let mut media = media(0);
    let short = [1u8; SLOT_LEN - 1];
    media
        .client
        .write(0, 0, NAMES[0], short.len(), |o, b| {
            b.copy_from_slice(&short[o..o + b.len()])
        })
        .unwrap();
    let mut slot = [0; SLOT_LEN];
    assert_eq!(media.read(0, &mut slot), Err(MediaError::Io));
    assert_eq!(
        SettingsStore::new().recover(&mut media),
        Err(Error::Media(MediaError::Io))
    );
}

#[test]
fn failed_commit_poisons_flush_and_later_writes() {
    let mut media = media(0);
    media.client.transport.fail_commit = true;
    let mut store = SettingsStore::new();
    assert_eq!(
        store.initialize(&mut media, first(0)),
        Err(Error::Media(MediaError::Io))
    );
    assert_eq!(media.flush(), Err(MediaError::Io));
    media.client.transport.fail_commit = false;
    assert_eq!(media.write(0, &[0; SLOT_LEN]), Err(MediaError::Io));
    // The abandoned upload was aborted and nothing was committed.
    assert!(media.client.transport.service.session(0).is_none());
    assert_eq!(
        media.client.transport.service.table().entries[0].revision,
        0
    );
}

#[test]
fn domains_are_independent_and_foreign_objects_refused() {
    let mut a = media(0);
    let mut store = SettingsStore::new();
    assert_eq!(store.initialize(&mut a, first(0)), Ok(1));
    let mut b = rebind(a, 1);
    assert_eq!(b.object(0), PER_DOMAIN as u64);
    assert!(matches!(b.client.stat(0), Err((Status::Access, _))));
    let mut other = SettingsStore::new();
    assert_eq!(other.recover(&mut b), Err(Error::NotInitialized));
    assert_eq!(other.initialize(&mut b, first(1)), Ok(1));
    assert_eq!(other.set(&mut b, second(1)), Ok(2));
    let mut a = rebind(b, 0);
    let mut again = SettingsStore::new();
    assert_eq!(again.recover(&mut a), Ok(1));
    assert_eq!(again.get(), Ok(first(0)));
}

#[test]
fn read_only_service_recovers_without_writing() {
    let mut writable = media(0);
    let mut store = SettingsStore::new();
    store.initialize(&mut writable, first(0)).unwrap();
    store.set(&mut writable, second(0)).unwrap();
    let mut local = writable.client.transport;
    let mut disk = local.disk;
    local.service = Service::recover(EPOCH, true, &mut disk).unwrap();
    local.disk = disk;
    let before = local.disk.0.clone();
    let mut media = VolumeMedia::new(Client::new(local, EPOCH, 9), 0);
    let mut cold = SettingsStore::new();
    assert_eq!(cold.recover(&mut media), Ok(2));
    assert_eq!(cold.get(), Ok(second(0)));
    assert!(cold.set(&mut media, first(0)).is_err());
    assert_eq!(media.client.transport.disk.0, before);
}

#[test]
fn report_encodings_are_fixed() {
    assert_eq!(encode(first(0)), 1 | 100 << 16);
    assert_eq!(encode(second(0)), 2 | 1 << 8 | 150 << 16);
    assert_eq!(encode(first(1)), 125 << 16);
    assert_eq!(encode(second(1)), 1 | 1 << 8 | 200 << 16);
    assert_eq!(code(Error::Corrupt), 5);
    assert_eq!(code(Error::UnsupportedVersion), 6);
    assert_eq!(code(Error::ReadbackMismatch), 11);
}
