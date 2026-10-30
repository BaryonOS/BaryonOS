// Reuse the volume service's established volatile-cache/crash model.
#[path = "../../../volume/tests/common/mod.rs"]
mod common;

use std::{cell::RefCell, rc::Rc};

use common::SimDisk;
use nieuw_os_nas_core::{
    Entropy, Error, NamespaceStatus, Principal, Service as Nas, Storage, Workspace, MAX_QUOTA,
    SNAPSHOT_BYTES,
};
use nieuw_os_nas_volume::VolumeStorage;
use volume_core::{
    client::{Client, Transport},
    service::Service as Volume,
    table::{self, Table},
    wire, Status, DATA_START, SECTOR,
};

const ADMIN: Principal = Principal([1; 32]);
const GUEST: Principal = Principal([2; 32]);
const EPOCH: u64 = 10;

struct TestEntropy(u8);
impl Entropy for TestEntropy {
    fn fill(&mut self, out: &mut [u8; 32]) -> Result<(), Error> {
        self.0 = self.0.checked_add(1).ok_or(Error::Entropy)?;
        out.fill(self.0);
        Ok(())
    }
}

struct Engine {
    volume: Volume,
    disk: SimDisk,
    lose_reply: Option<u16>,
}

#[derive(Clone)]
struct Local {
    engine: Rc<RefCell<Engine>>,
    domain: usize,
}

impl Transport for Local {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        let mut engine = self.engine.borrow_mut();
        let Engine { volume, disk, .. } = &mut *engine;
        let reply = volume.handle(self.domain, request, disk);
        if engine.lose_reply == Some(wire::header(request).operation) {
            engine.lose_reply = None;
            return Err(Status::Timeout);
        }
        Ok(reply)
    }
}

impl Local {
    fn fresh() -> Self {
        Self {
            engine: Rc::new(RefCell::new(Engine {
                volume: Volume::new(EPOCH, Table::EMPTY, false),
                disk: SimDisk::new(),
                lose_reply: None,
            })),
            domain: 0,
        }
    }

    fn recover(image: Vec<[u8; SECTOR]>) -> Self {
        let mut disk = SimDisk::from_image(image);
        let mut table = Table::EMPTY;
        table::recover(&mut disk, &mut table).unwrap();
        Self {
            engine: Rc::new(RefCell::new(Engine {
                volume: Volume::new(EPOCH, table, false),
                disk,
                lose_reply: None,
            })),
            domain: 0,
        }
    }

    fn storage(&self) -> VolumeStorage<Self> {
        VolumeStorage::new(Client::new(self.clone(), EPOCH, 1), 0).unwrap()
    }

    fn history(&self) -> usize {
        self.engine.borrow().volume.table().entries[0].history_count
    }

    fn mutations(&self) -> usize {
        self.engine.borrow().disk.mutating_ops
    }

    fn image(&self) -> Vec<[u8; SECTOR]> {
        let engine = self.engine.borrow();
        assert!(engine.disk.pending.is_empty(), "ACK before flush");
        engine.disk.durable.clone()
    }
}

fn fixture(work: &mut Workspace) -> (Nas<'_, VolumeStorage<Local>>, Local, TestEntropy, [u8; 32]) {
    let local = Local::fresh();
    let mut entropy = TestEntropy(20);
    let mut nas = Nas::initialize(local.storage(), ADMIN, &mut entropy, work).unwrap();
    let invitation = nas
        .invite(ADMIN, GUEST, MAX_QUOTA, 100, 10, &mut entropy)
        .unwrap();
    nas.accept(GUEST, &invitation, 100).unwrap();
    let session = nas.open(GUEST, &invitation.namespace).unwrap();
    let upload = nas
        .stage(GUEST, session, 0, 0, b"synthetic-client-ciphertext")
        .unwrap();
    nas.commit(GUEST, session, upload).unwrap();
    (nas, local, entropy, invitation.namespace)
}

#[test]
fn committed_snapshot_survives_cold_volume_recovery_without_history() {
    let mut work = Box::new(Workspace::new());
    let (nas, local, mut entropy, namespace) = fixture(&mut work);
    assert_eq!(local.history(), 0);
    drop(nas);
    let cold = Local::recover(local.image());
    let before = cold.mutations();
    let mut nas = Nas::recover(cold.storage(), &mut entropy, &mut work).unwrap();
    assert_eq!(cold.mutations(), before, "recovery must be read-only");
    let session = nas.open(GUEST, &namespace).unwrap();
    let mut out = [0; 64];
    assert_eq!(nas.read(GUEST, session, 0, 1, &mut out), Ok(27));
    assert_eq!(&out[..27], b"synthetic-client-ciphertext");
}

#[test]
fn actual_core_volume_outcome_matches_independent_python_probe_oracle() {
    let principal = |pid: u64| {
        let mut bytes = [0; 32];
        bytes[..8].copy_from_slice(b"NOSPROC1");
        bytes[8..16].copy_from_slice(&pid.to_le_bytes());
        Principal(bytes)
    };
    let (admin, guest) = (principal(11), principal(12));
    let local = Local::fresh();
    let mut work = Box::new(Workspace::new());
    let mut entropy = TestEntropy(20);
    let mut nas = Nas::initialize(local.storage(), admin, &mut entropy, &mut work).unwrap();
    let invite = nas
        .invite(admin, guest, 128, 100, 6000, &mut entropy)
        .unwrap();
    nas.accept(guest, &invite, 100).unwrap();
    let session = nas.open(guest, &invite.namespace).unwrap();
    let upload = nas.stage(guest, session, 0, 0, &[0xA5; 96]).unwrap();
    nas.commit(guest, session, upload).unwrap();
    nas.stage(guest, session, 0, 1, &[0x55; 32]).unwrap();
    assert_eq!(
        nas.set_quota(admin, &invite.namespace, 127),
        Err(Error::Quota)
    );
    nas.revoke(admin, &invite.namespace).unwrap();
    nas.evict(admin, &invite.namespace).unwrap();
    drop(nas);
    let image = local.image();
    let cold = Local::recover(image.clone());
    let mut nas = Nas::recover(cold.storage(), &mut entropy, &mut work).unwrap();
    assert!(matches!(
        nas.open(guest, &invite.namespace),
        Err(Error::Denied)
    ));
    nas.evict(admin, &invite.namespace).unwrap();
    assert_eq!(cold.mutations(), 0);
    assert_eq!(cold.image(), image);

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "nieuw-os-nas-oracle-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).unwrap();
    let raw = directory.join("volume.raw");
    let bytes: Vec<u8> = image.iter().flatten().copied().collect();
    std::fs::write(&raw, bytes).unwrap();
    let output = std::process::Command::new("python3")
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native-probe/tests/check_volume_fixture.py"
        ))
        .arg(&raw)
        .output()
        .unwrap();
    for path in [
        raw,
        directory.join("synthetic.img"),
        directory.join("synthetic.json"),
    ] {
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
    }
    std::fs::remove_dir(directory).unwrap();
    assert!(
        output.status.success(),
        "independent host oracle failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn revoke_and_evict_cold_recover_without_restorable_guest_versions() {
    let mut work = Box::new(Workspace::new());
    for evict in [false, true] {
        let (mut nas, local, mut entropy, namespace) = fixture(&mut work);
        if evict {
            nas.evict(ADMIN, &namespace).unwrap();
        } else {
            nas.revoke(ADMIN, &namespace).unwrap();
        }
        assert_eq!(local.history(), 0);
        let cold = Local::recover(local.image());
        let mut nas = Nas::recover(cold.storage(), &mut entropy, &mut work).unwrap();
        let usage = nas.usage(ADMIN, &namespace).unwrap();
        assert_eq!(
            usage.status,
            if evict {
                NamespaceStatus::Evicted
            } else {
                NamespaceStatus::Revoked
            }
        );
        assert_eq!(usage.used, if evict { 0 } else { 27 });
        assert!(matches!(nas.open(GUEST, &namespace), Err(Error::Denied)));
        let mut client = cold.storage().into_client();
        let stat = client.stat(0).unwrap();
        assert_eq!(
            client.restore(0, stat.revision, 1),
            Err((Status::NotFound, stat.revision))
        );
    }
}

#[test]
fn missing_commit_reply_requires_explicit_history_cleanup_before_recovery() {
    let mut work = Box::new(Workspace::new());
    let (mut nas, local, mut entropy, namespace) = fixture(&mut work);
    local.engine.borrow_mut().lose_reply = Some(wire::COMMIT);
    assert_eq!(nas.revoke(ADMIN, &namespace), Err(Error::Unknown));
    assert!(nas.is_poisoned());
    assert_eq!(local.history(), 1);
    let cold = Local::recover(local.image());
    let before = cold.mutations();
    assert!(matches!(
        Nas::recover(cold.storage(), &mut entropy, &mut work),
        Err(Error::ReopenRequired)
    ));
    assert_eq!(cold.mutations(), before);
    let mut storage = cold.storage();
    storage.finish_cleanup().unwrap();
    assert_eq!(cold.history(), 0);
    assert!(matches!(
        storage.replace(&[0; SNAPSHOT_BYTES]),
        Err(Error::ReopenRequired)
    ));
    let mut nas = Nas::recover(storage, &mut entropy, &mut work).unwrap();
    assert_eq!(
        nas.usage(ADMIN, &namespace).unwrap().status,
        NamespaceStatus::Revoked
    );
    assert!(matches!(nas.open(GUEST, &namespace), Err(Error::Denied)));
}

#[test]
fn missing_purge_reply_never_acks_but_clean_current_state_can_recover() {
    let mut work = Box::new(Workspace::new());
    let (mut nas, local, mut entropy, namespace) = fixture(&mut work);
    local.engine.borrow_mut().lose_reply = Some(wire::PURGE);
    assert_eq!(nas.revoke(ADMIN, &namespace), Err(Error::Unknown));
    assert_eq!(local.history(), 0);
    assert_eq!(nas.revoke(ADMIN, &namespace), Err(Error::ReopenRequired));
    let cold = Local::recover(local.image());
    let nas = Nas::recover(cold.storage(), &mut entropy, &mut work).unwrap();
    assert_eq!(
        nas.usage(ADMIN, &namespace).unwrap().status,
        NamespaceStatus::Revoked
    );
}

#[test]
fn cleanup_never_discards_history_when_current_content_is_corrupt() {
    let mut work = Box::new(Workspace::new());
    let (mut nas, local, _, namespace) = fixture(&mut work);
    local.engine.borrow_mut().lose_reply = Some(wire::COMMIT);
    assert_eq!(nas.revoke(ADMIN, &namespace), Err(Error::Unknown));
    let mut image = local.image();
    let first = local.engine.borrow().volume.table().entries[0]
        .current
        .chunks[0];
    let sector = DATA_START + u64::from(first) * volume_core::CHUNK_SECTORS;
    image[sector as usize][0] ^= 1;
    let cold = Local::recover(image);
    let before = cold.mutations();
    assert_eq!(cold.storage().finish_cleanup(), Err(Error::Corrupt));
    assert_eq!(cold.mutations(), before);
    assert_eq!(cold.history(), 1);
}

#[test]
fn another_domain_cannot_initialize_read_repair_or_replace_nas_authority() {
    let mut work = Box::new(Workspace::new());
    let (_, mut local, mut entropy, _) = fixture(&mut work);
    local.domain = 1;
    let before = local.mutations();
    assert!(matches!(
        Nas::recover(local.storage(), &mut entropy, &mut work),
        Err(Error::Denied)
    ));
    assert!(matches!(
        Nas::initialize(local.storage(), ADMIN, &mut entropy, &mut work),
        Err(Error::Denied)
    ));
    assert_eq!(local.storage().finish_cleanup(), Err(Error::Denied));
    assert_eq!(local.mutations(), before);
}

#[test]
fn foreign_deleted_and_malformed_objects_are_never_silently_initialized() {
    let mut work = Box::new(Workspace::new());
    for name in [&b"another-service"[..], &b"nas-authority-v1"[..]] {
        let local = Local::fresh();
        let mut client = local.storage().into_client();
        client
            .write(0, 0, name, 5, |_, out| out.fill(0x55))
            .unwrap();
        let before = local.mutations();
        assert!(matches!(
            Nas::initialize(local.storage(), ADMIN, &mut TestEntropy(1), &mut work),
            Err(Error::Corrupt)
        ));
        assert_eq!(local.mutations(), before);
    }
    let (_, local, mut entropy, _) = fixture(&mut work);
    let mut client = local.storage().into_client();
    let stat = client.stat(0).unwrap();
    client.delete(0, stat.revision).unwrap();
    let before = local.mutations();
    assert!(matches!(
        Nas::initialize(local.storage(), ADMIN, &mut entropy, &mut work),
        Err(Error::Corrupt)
    ));
    assert_eq!(local.mutations(), before);
}

#[test]
fn stale_writer_cannot_replace_a_newer_volume_revision_or_retry_implicitly() {
    let mut work = Box::new(Workspace::new());
    let (_, local, _, _) = fixture(&mut work);
    let mut stale = local.storage();
    let mut bytes = [0; SNAPSHOT_BYTES];
    stale.load(&mut bytes).unwrap();
    let mut client = local.storage().into_client();
    let revision = client.stat(0).unwrap().revision;
    client
        .write(
            0,
            revision,
            b"nas-authority-v1",
            bytes.len(),
            |offset, out| out.copy_from_slice(&bytes[offset..offset + out.len()]),
        )
        .unwrap();
    let before = local.mutations();
    assert_eq!(stale.replace(&bytes), Err(Error::Conflict));
    assert_eq!(stale.replace(&bytes), Err(Error::ReopenRequired));
    assert_eq!(local.mutations(), before);
}

#[test]
fn every_eviction_disk_failure_keeps_a_recoverable_old_or_revoked_state() {
    let mut work = Box::new(Workspace::new());
    let (mut nas, local, _, namespace) = fixture(&mut work);
    let image = local.image();
    let before = local.mutations();
    nas.evict(ADMIN, &namespace).unwrap();
    let operations = local.mutations() - before;
    assert!(operations > 0);
    let base_generation = Local::recover(image.clone())
        .engine
        .borrow()
        .volume
        .table()
        .generation;

    for stop in 0..operations {
        let local = Local::recover(image.clone());
        let mut entropy = TestEntropy(60);
        let mut nas = Nas::recover(local.storage(), &mut entropy, &mut work).unwrap();
        local.engine.borrow_mut().disk.crash_at = Some(stop);
        assert_eq!(
            nas.evict(ADMIN, &namespace),
            Err(Error::Unknown),
            "failure point {stop}"
        );
        assert!(nas.is_poisoned());

        for keep in [0, usize::MAX] {
            let crashed = {
                let engine = local.engine.borrow();
                engine.disk.crash_image(keep, Some(SECTOR / 2))
            };
            let cold = Local::recover(crashed);
            let generation = cold.engine.borrow().volume.table().generation;
            let mut storage = cold.storage();
            if cold.history() != 0 {
                storage.finish_cleanup().unwrap();
            }
            let mut nas = Nas::recover(storage, &mut entropy, &mut work).unwrap();
            let usage = nas.usage(ADMIN, &namespace).unwrap();
            if generation > base_generation {
                assert_ne!(
                    usage.status,
                    NamespaceStatus::Active,
                    "revocation lost at failure point {stop}"
                );
                assert!(matches!(nas.open(GUEST, &namespace), Err(Error::Denied)));
            }
            if usage.status == NamespaceStatus::Evicted {
                assert_eq!(usage.used, 0);
            }
            assert_eq!(cold.history(), 0);
        }
    }
    println!("{operations} eviction disk failure points, two crash images each");
}

#[test]
#[cfg(not(debug_assertions))]
fn optimized_nas_and_volume_client_fit_64_kib_with_separate_service_stack() {
    use std::sync::mpsc::{channel, Receiver, Sender};

    struct Ipc {
        request: Sender<[u8; wire::SIZE]>,
        reply: Receiver<[u8; wire::SIZE]>,
    }
    impl Transport for Ipc {
        fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
            self.request.send(*request).map_err(|_| Status::NoDevice)?;
            self.reply.recv().map_err(|_| Status::NoDevice)
        }
    }

    let (request, requests) = channel();
    let (reply, replies) = channel();
    let server = std::thread::spawn(move || {
        let mut volume = Volume::new(EPOCH, Table::EMPTY, false);
        let mut disk = SimDisk::new();
        while let Ok(request) = requests.recv() {
            reply.send(volume.handle(0, &request, &mut disk)).unwrap();
        }
    });
    let mut work = Box::new(Workspace::new());
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(move || {
            let client = Client::new(
                Ipc {
                    request,
                    reply: replies,
                },
                EPOCH,
                1,
            );
            let storage = VolumeStorage::new(client, 0).unwrap();
            let mut entropy = TestEntropy(10);
            let mut nas = Nas::initialize(storage, ADMIN, &mut entropy, &mut work).unwrap();
            let invite = nas
                .invite(ADMIN, GUEST, MAX_QUOTA, 100, 10, &mut entropy)
                .unwrap();
            nas.accept(GUEST, &invite, 100).unwrap();
            let session = nas.open(GUEST, &invite.namespace).unwrap();
            let upload = nas.stage(GUEST, session, 0, 0, &[7; 1024]).unwrap();
            nas.commit(GUEST, session, upload).unwrap();
            nas.evict(ADMIN, &invite.namespace).unwrap();
            let mut storage = nas.into_storage();
            storage.finish_cleanup().unwrap();
            let nas = Nas::recover(storage, &mut entropy, &mut work).unwrap();
            assert_eq!(
                nas.usage(ADMIN, &invite.namespace).unwrap().status,
                NamespaceStatus::Evicted
            );
        })
        .unwrap()
        .join()
        .unwrap();
    server.join().unwrap();
}
