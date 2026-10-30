use std::{cell::RefCell, rc::Rc};

use nieuw_os_nas_core::{
    Entropy, Error, Invitation, NamespaceStatus, Principal, Service, Storage, Workspace,
    MAX_CIPHERTEXT, MAX_QUOTA, NAMESPACES, SESSIONS, SNAPSHOT_BYTES, UPLOADS,
};

const ADMIN: Principal = Principal([1; 32]);
const ALICE: Principal = Principal([2; 32]);
const BOB: Principal = Principal([3; 32]);

/// Deterministic host-test input only; never used by a production adapter.
struct TestEntropy(u8);
impl Entropy for TestEntropy {
    fn fill(&mut self, bytes: &mut [u8; 32]) -> Result<(), Error> {
        self.0 = self.0.checked_add(1).ok_or(Error::Entropy)?;
        bytes.fill(self.0);
        Ok(())
    }
}

#[derive(Default)]
struct Disk {
    bytes: Option<Vec<u8>>,
    writes: usize,
    fail_at: Option<usize>,
    commit_on_failure: bool,
}

#[derive(Clone, Default)]
struct Memory(Rc<RefCell<Disk>>);

impl Storage for Memory {
    fn load(&mut self, out: &mut [u8; SNAPSHOT_BYTES]) -> Result<Option<usize>, Error> {
        let disk = self.0.borrow();
        let Some(bytes) = &disk.bytes else {
            return Ok(None);
        };
        if bytes.len() <= out.len() {
            out[..bytes.len()].copy_from_slice(bytes);
        }
        Ok(Some(bytes.len()))
    }

    fn replace(&mut self, bytes: &[u8; SNAPSHOT_BYTES]) -> Result<(), Error> {
        let mut disk = self.0.borrow_mut();
        disk.writes += 1;
        if disk.fail_at == Some(disk.writes) {
            if disk.commit_on_failure {
                disk.bytes = Some(bytes.to_vec());
            }
            return Err(Error::Unknown);
        }
        disk.bytes = Some(bytes.to_vec());
        Ok(())
    }
}

fn fixture(work: &mut Workspace) -> (Service<'_, Memory>, Memory, TestEntropy) {
    let disk = Memory::default();
    let mut entropy = TestEntropy(10);
    let service = Service::initialize(disk.clone(), ADMIN, &mut entropy, work).unwrap();
    (service, disk, entropy)
}

fn pair(
    service: &mut Service<'_, Memory>,
    entropy: &mut TestEntropy,
    peer: Principal,
) -> Invitation {
    let invitation = service
        .invite(ADMIN, peer, MAX_QUOTA, 100, 10, entropy)
        .unwrap();
    service.accept(peer, &invitation, 100).unwrap();
    invitation
}

fn fail_next(disk: &Memory, later: usize, commit: bool) {
    let mut d = disk.0.borrow_mut();
    d.fail_at = Some(d.writes + later);
    d.commit_on_failure = commit;
}

#[test]
fn explicit_initialization_never_replaces_existing_or_corrupt_storage() {
    let mut work = Box::new(Workspace::new());
    let (_, disk, mut entropy) = fixture(&mut work);
    let writes = disk.0.borrow().writes;
    assert!(matches!(
        Service::initialize(disk.clone(), ADMIN, &mut entropy, &mut work),
        Err(Error::Conflict)
    ));
    disk.0.borrow_mut().bytes = Some(vec![0xff; SNAPSHOT_BYTES]);
    assert!(matches!(
        Service::initialize(disk.clone(), ADMIN, &mut entropy, &mut work),
        Err(Error::Conflict)
    ));
    assert!(matches!(
        Service::recover(disk.clone(), &mut entropy, &mut work),
        Err(Error::Corrupt)
    ));
    assert_eq!(disk.0.borrow().writes, writes);
}

#[test]
fn absent_truncated_and_oversized_storage_are_not_empty_authority() {
    let mut work = Box::new(Workspace::new());
    for length in [0, 1, SNAPSHOT_BYTES - 1, SNAPSHOT_BYTES + 1] {
        let disk = Memory::default();
        disk.0.borrow_mut().bytes = Some(vec![0; length]);
        assert!(matches!(
            Service::recover(disk.clone(), &mut TestEntropy(1), &mut work),
            Err(Error::Corrupt)
        ));
        assert_eq!(disk.0.borrow().writes, 0);
    }
    assert!(matches!(
        Service::recover(Memory::default(), &mut TestEntropy(1), &mut work),
        Err(Error::Corrupt)
    ));
}

#[test]
fn invitation_is_bound_to_recipient_and_expires_at_exact_boundary() {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, mut entropy) = fixture(&mut work);
    let invitation = service
        .invite(ADMIN, ALICE, 100, 100, 10, &mut entropy)
        .unwrap();
    let writes = disk.0.borrow().writes;
    assert_eq!(service.accept(BOB, &invitation, 100), Err(Error::Denied));
    let mut forged = invitation;
    forged.recipient = BOB;
    assert_eq!(service.accept(BOB, &forged, 100), Err(Error::Denied));
    forged = invitation;
    forged.nonce[0] ^= 1;
    assert_eq!(service.accept(ALICE, &forged, 100), Err(Error::Denied));
    forged = invitation;
    forged.expires += 1;
    assert_eq!(service.accept(ALICE, &forged, 100), Err(Error::Denied));
    assert_eq!(service.accept(ALICE, &invitation, 110), Err(Error::Expired));
    assert_eq!(disk.0.borrow().writes, writes);
}

#[test]
fn accepted_invitation_cannot_replay_even_after_recovery() {
    let mut work = Box::new(Workspace::new());
    let (mut service, _, mut entropy) = fixture(&mut work);
    let invitation = pair(&mut service, &mut entropy, ALICE);
    assert_eq!(service.accept(ALICE, &invitation, 100), Err(Error::Denied));
    let mut service = Service::recover(service.into_storage(), &mut entropy, &mut work).unwrap();
    assert_eq!(service.accept(ALICE, &invitation, 100), Err(Error::Denied));
    assert!(service.open(ALICE, &invitation.namespace).is_ok());
}

#[test]
fn admin_and_guest_authorities_do_not_substitute_for_each_other() {
    let mut work = Box::new(Workspace::new());
    let (mut service, _, mut entropy) = fixture(&mut work);
    let invitation = pair(&mut service, &mut entropy, ALICE);
    assert!(matches!(
        service.open(ADMIN, &invitation.namespace),
        Err(Error::Denied)
    ));
    assert!(matches!(
        service.invite(ALICE, BOB, 100, 0, 1, &mut entropy),
        Err(Error::Denied)
    ));
    assert!(matches!(
        service.usage(ALICE, &invitation.namespace),
        Err(Error::Denied)
    ));
    assert_eq!(
        service.set_quota(ALICE, &invitation.namespace, 100),
        Err(Error::Denied)
    );
    assert_eq!(
        service.revoke(ALICE, &invitation.namespace),
        Err(Error::Denied)
    );
    assert_eq!(
        service.evict(ALICE, &invitation.namespace),
        Err(Error::Denied)
    );
}

#[test]
fn sessions_uploads_and_reads_are_bound_to_actual_caller_and_namespace() {
    let mut work = Box::new(Workspace::new());
    let (mut service, _, mut entropy) = fixture(&mut work);
    let alice = pair(&mut service, &mut entropy, ALICE);
    let bob = pair(&mut service, &mut entropy, BOB);
    let a = service.open(ALICE, &alice.namespace).unwrap();
    let b = service.open(BOB, &bob.namespace).unwrap();
    let upload = service.stage(ALICE, a, 0, 0, b"opaque-alice").unwrap();
    assert_eq!(service.commit(BOB, b, upload), Err(Error::Stale));
    assert_eq!(service.commit(BOB, a, upload), Err(Error::Stale));
    assert_eq!(service.commit(ALICE, a, upload), Ok(1));
    let mut out = [0xaa; 64];
    assert_eq!(service.read(BOB, a, 0, 1, &mut out), Err(Error::Stale));
    assert_eq!(service.read(BOB, b, 0, 1, &mut out), Err(Error::NotFound));
    assert_eq!(out, [0xaa; 64]);
    assert_eq!(service.read(ALICE, a, 0, 1, &mut out), Ok(12));
    assert_eq!(&out[..12], b"opaque-alice");
    assert_eq!(&out[12..], &[0xaa; 52]);
}

#[test]
fn quota_counts_all_reservations_and_cancel_releases_only_its_upload() {
    let mut work = Box::new(Workspace::new());
    let (mut service, _, mut entropy) = fixture(&mut work);
    let invite = pair(&mut service, &mut entropy, ALICE);
    service.set_quota(ADMIN, &invite.namespace, 16).unwrap();
    let session = service.open(ALICE, &invite.namespace).unwrap();
    let a = service.stage(ALICE, session, 0, 0, &[1; 8]).unwrap();
    let b = service.stage(ALICE, session, 1, 0, &[2; 8]).unwrap();
    assert!(matches!(
        service.stage(ALICE, session, 2, 0, &[3]),
        Err(Error::Quota)
    ));
    assert_eq!(
        service.set_quota(ADMIN, &invite.namespace, 15),
        Err(Error::Quota)
    );
    service.cancel(ALICE, session, a).unwrap();
    assert_eq!(service.commit(ALICE, session, a), Err(Error::Stale));
    assert_eq!(service.usage(ADMIN, &invite.namespace).unwrap().reserved, 8);
    service.commit(ALICE, session, b).unwrap();
    let usage = service.usage(ADMIN, &invite.namespace).unwrap();
    assert_eq!((usage.used, usage.reserved), (8, 0));
    assert_eq!(
        service.set_quota(ADMIN, &invite.namespace, 7),
        Err(Error::Quota)
    );
}

#[test]
fn commit_copies_input_and_rejects_competing_writer_without_losing_winner() {
    let mut work = Box::new(Workspace::new());
    let (mut service, _, mut entropy) = fixture(&mut work);
    let invite = pair(&mut service, &mut entropy, ALICE);
    let a = service.open(ALICE, &invite.namespace).unwrap();
    let b = service.open(ALICE, &invite.namespace).unwrap();
    let mut input = [9; 32];
    let first = service.stage(ALICE, a, 0, 0, &input).unwrap();
    let second = service.stage(ALICE, b, 0, 0, &[8; 32]).unwrap();
    input.fill(0);
    assert_eq!(service.commit(ALICE, a, first), Ok(1));
    assert_eq!(service.commit(ALICE, b, second), Err(Error::Conflict));
    assert_eq!(service.commit(ALICE, b, second), Err(Error::Stale));
    let mut out = [0; 32];
    assert_eq!(service.read(ALICE, a, 0, 1, &mut out), Ok(32));
    assert_eq!(out, [9; 32]);
    assert_eq!(service.usage(ADMIN, &invite.namespace).unwrap().reserved, 0);
}

#[test]
fn close_releases_reservations_and_cannot_be_used_to_close_another_session() {
    let mut work = Box::new(Workspace::new());
    let (mut service, _, mut entropy) = fixture(&mut work);
    let invite = pair(&mut service, &mut entropy, ALICE);
    let a = service.open(ALICE, &invite.namespace).unwrap();
    let b = service.open(ALICE, &invite.namespace).unwrap();
    let upload = service.stage(ALICE, a, 0, 0, &[7; 10]).unwrap();
    assert_eq!(service.close(BOB, a), Err(Error::Stale));
    service.close(ALICE, a).unwrap();
    assert_eq!(service.commit(ALICE, a, upload), Err(Error::Stale));
    assert_eq!(service.usage(ADMIN, &invite.namespace).unwrap().reserved, 0);
    assert!(service.stage(ALICE, b, 0, 0, &[7]).is_ok());
}

#[test]
fn revoke_is_durable_without_guest_consent_and_full_quota_cannot_block_it() {
    let mut work = Box::new(Workspace::new());
    let (mut service, _, mut entropy) = fixture(&mut work);
    let invite = pair(&mut service, &mut entropy, ALICE);
    service.set_quota(ADMIN, &invite.namespace, 8).unwrap();
    let session = service.open(ALICE, &invite.namespace).unwrap();
    let upload = service.stage(ALICE, session, 0, 0, &[4; 8]).unwrap();
    service.commit(ALICE, session, upload).unwrap();
    service.revoke(ADMIN, &invite.namespace).unwrap();
    assert_eq!(
        service.read(ALICE, session, 0, 1, &mut [0; 8]),
        Err(Error::Stale)
    );
    let mut service = Service::recover(service.into_storage(), &mut entropy, &mut work).unwrap();
    assert!(matches!(
        service.open(ALICE, &invite.namespace),
        Err(Error::Denied)
    ));
    let usage = service.usage(ADMIN, &invite.namespace).unwrap();
    assert_eq!(usage.status, NamespaceStatus::Revoked);
    assert_eq!((usage.epoch, usage.used, usage.reserved), (2, 8, 0));
    service.revoke(ADMIN, &invite.namespace).unwrap();
    assert_eq!(service.usage(ADMIN, &invite.namespace).unwrap().epoch, 2);
}

#[test]
fn eviction_removes_committed_and_pending_bytes_and_preserves_tombstone() {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, mut entropy) = fixture(&mut work);
    let invite = pair(&mut service, &mut entropy, ALICE);
    let session = service.open(ALICE, &invite.namespace).unwrap();
    let bytes = b"synthetic-opaque-ciphertext-for-purge";
    let upload = service.stage(ALICE, session, 0, 0, bytes).unwrap();
    service.commit(ALICE, session, upload).unwrap();
    let pending = service.stage(ALICE, session, 0, 1, b"pending").unwrap();
    service.evict(ADMIN, &invite.namespace).unwrap();
    assert_eq!(service.commit(ALICE, session, pending), Err(Error::Stale));
    assert!(!disk
        .0
        .borrow()
        .bytes
        .as_ref()
        .unwrap()
        .windows(bytes.len())
        .any(|w| w == bytes));
    let mut service = Service::recover(service.into_storage(), &mut entropy, &mut work).unwrap();
    let usage = service.usage(ADMIN, &invite.namespace).unwrap();
    assert_eq!(usage.status, NamespaceStatus::Evicted);
    assert_eq!((usage.used, usage.reserved), (0, 0));
    assert!(matches!(
        service.open(ALICE, &invite.namespace),
        Err(Error::Denied)
    ));
    assert_eq!(service.accept(ALICE, &invite, 100), Err(Error::Denied));
    let next = pair(&mut service, &mut entropy, ALICE);
    assert_ne!(next.namespace, invite.namespace);
}

#[test]
fn restart_preserves_ciphertext_but_never_sessions_staging_or_old_uploads() {
    let mut work = Box::new(Workspace::new());
    let (mut service, _, mut entropy) = fixture(&mut work);
    let invite = pair(&mut service, &mut entropy, ALICE);
    let session = service.open(ALICE, &invite.namespace).unwrap();
    let committed = service.stage(ALICE, session, 0, 0, &[8; 100]).unwrap();
    service.commit(ALICE, session, committed).unwrap();
    let pending = service.stage(ALICE, session, 0, 1, &[9; 50]).unwrap();
    let mut service = Service::recover(service.into_storage(), &mut entropy, &mut work).unwrap();
    assert_eq!(service.commit(ALICE, session, pending), Err(Error::Stale));
    assert_eq!(
        service.read(ALICE, session, 0, 1, &mut [0; 100]),
        Err(Error::Stale)
    );
    let fresh = service.open(ALICE, &invite.namespace).unwrap();
    let mut out = [0; 100];
    assert_eq!(service.read(ALICE, fresh, 0, 1, &mut out), Ok(100));
    assert_eq!(out, [8; 100]);
    assert_eq!(service.usage(ADMIN, &invite.namespace).unwrap().reserved, 0);
}

#[test]
fn uncertain_publication_never_acks_replays_or_keeps_serving() {
    let mut work = Box::new(Workspace::new());
    for committed in [false, true] {
        let (mut service, disk, mut entropy) = fixture(&mut work);
        let invite = pair(&mut service, &mut entropy, ALICE);
        let session = service.open(ALICE, &invite.namespace).unwrap();
        let upload = service.stage(ALICE, session, 0, 0, &[5; 30]).unwrap();
        fail_next(&disk, 1, committed);
        assert_eq!(service.commit(ALICE, session, upload), Err(Error::Unknown));
        let writes = disk.0.borrow().writes;
        assert!(service.is_poisoned());
        assert_eq!(
            service.commit(ALICE, session, upload),
            Err(Error::ReopenRequired)
        );
        assert!(matches!(
            service.usage(ADMIN, &invite.namespace),
            Err(Error::ReopenRequired)
        ));
        assert_eq!(disk.0.borrow().writes, writes);
        let mut service =
            Service::recover(service.into_storage(), &mut entropy, &mut work).unwrap();
        let session = service.open(ALICE, &invite.namespace).unwrap();
        assert_eq!(
            service.read(ALICE, session, 0, 1, &mut [0; 30]),
            if committed {
                Ok(30)
            } else {
                Err(Error::NotFound)
            }
        );
    }
}

#[test]
fn crash_at_each_eviction_commit_preserves_only_possible_durable_outcome() {
    let mut work = Box::new(Workspace::new());
    for step in [1, 2] {
        for committed in [false, true] {
            let (mut service, disk, mut entropy) = fixture(&mut work);
            let invite = pair(&mut service, &mut entropy, ALICE);
            let session = service.open(ALICE, &invite.namespace).unwrap();
            let upload = service.stage(ALICE, session, 0, 0, &[4; 80]).unwrap();
            service.commit(ALICE, session, upload).unwrap();
            fail_next(&disk, step, committed);
            assert_eq!(service.evict(ADMIN, &invite.namespace), Err(Error::Unknown));
            assert_eq!(
                service.read(ALICE, session, 0, 1, &mut [0; 80]),
                Err(Error::ReopenRequired)
            );
            let mut service =
                Service::recover(service.into_storage(), &mut entropy, &mut work).unwrap();
            let expected = match (step, committed) {
                (1, false) => NamespaceStatus::Active,
                (2, true) => NamespaceStatus::Evicted,
                _ => NamespaceStatus::Revoked,
            };
            assert_eq!(
                service.usage(ADMIN, &invite.namespace).unwrap().status,
                expected
            );
            if expected != NamespaceStatus::Active {
                assert!(matches!(
                    service.open(ALICE, &invite.namespace),
                    Err(Error::Denied)
                ));
            }
            // Explicit idempotent administrator retry, never an automatic
            // replay based on the previous ambiguous response.
            service.evict(ADMIN, &invite.namespace).unwrap();
            assert_eq!(
                service.usage(ADMIN, &invite.namespace).unwrap().status,
                NamespaceStatus::Evicted
            );
        }
    }
}

#[test]
fn resource_limits_and_invalid_inputs_leave_durable_state_unchanged() {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, mut entropy) = fixture(&mut work);
    for (quota, now, lifetime) in [
        (0, 0, 1),
        (MAX_QUOTA + 1, 0, 1),
        (1, 0, 0),
        (1, u64::MAX, 1),
    ] {
        assert!(matches!(
            service.invite(ADMIN, ALICE, quota, now, lifetime, &mut entropy),
            Err(Error::Invalid)
        ));
    }
    let invite = pair(&mut service, &mut entropy, ALICE);
    let session = service.open(ALICE, &invite.namespace).unwrap();
    for _ in 1..SESSIONS {
        service.open(ALICE, &invite.namespace).unwrap();
    }
    assert!(matches!(
        service.open(ALICE, &invite.namespace),
        Err(Error::Full)
    ));
    for data in [&[][..], &[0; MAX_CIPHERTEXT + 1][..]] {
        assert!(matches!(
            service.stage(ALICE, session, 0, 0, data),
            Err(Error::Invalid)
        ));
    }
    assert!(matches!(
        service.stage(ALICE, session, usize::MAX, 0, &[1]),
        Err(Error::Invalid)
    ));
    assert!(matches!(
        service.stage(ALICE, session, 0, u64::MAX, &[1]),
        Err(Error::Conflict)
    ));
    let writes = disk.0.borrow().writes;
    for _ in 0..UPLOADS {
        service.stage(ALICE, session, 0, 0, &[1]).unwrap();
    }
    assert!(matches!(
        service.stage(ALICE, session, 0, 0, &[1]),
        Err(Error::Full)
    ));
    assert_eq!(disk.0.borrow().writes, writes);
    for _ in 1..NAMESPACES {
        let invite = pair(&mut service, &mut entropy, ALICE);
        service.evict(ADMIN, &invite.namespace).unwrap();
    }
    assert!(matches!(
        service.invite(ADMIN, ALICE, 1, 0, 1, &mut entropy),
        Err(Error::Full)
    ));
}

#[test]
fn read_errors_never_modify_output_and_revision_is_checked() {
    let mut work = Box::new(Workspace::new());
    let (mut service, _, mut entropy) = fixture(&mut work);
    let invite = pair(&mut service, &mut entropy, ALICE);
    let session = service.open(ALICE, &invite.namespace).unwrap();
    let upload = service
        .stage(ALICE, session, 0, 0, &[0x55; MAX_CIPHERTEXT])
        .unwrap();
    assert_eq!(service.commit(ALICE, session, upload), Ok(1));
    let mut out = [0xaa; MAX_CIPHERTEXT - 1];
    for (object, revision, expected) in [
        (0, 0, Error::Conflict),
        (1, 1, Error::NotFound),
        (usize::MAX, 1, Error::Invalid),
        (0, 1, Error::Invalid),
    ] {
        assert_eq!(
            service.read(ALICE, session, object, revision, &mut out),
            Err(expected)
        );
        assert!(out.iter().all(|v| *v == 0xaa));
    }
}

#[test]
fn malformed_snapshot_fields_are_rejected_without_writes_or_panics() {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, mut entropy) = fixture(&mut work);
    pair(&mut service, &mut entropy, ALICE);
    let original = disk.0.borrow().bytes.clone().unwrap();
    // Magic/version/reserved, generation/admin, namespace ID/status/epoch/
    // quota/owner, noncanonical invitation, object length and padding.
    for (offset, replacement) in [
        (0, 0),
        (8, 2),
        (10, 1),
        (56, 1),
        (64 + 32, 99),
        (64 + 33, 1),
        (64 + 52, 1),
        (64 + 88, 1),
        (64 + 128 + 9, 255),
        (64 + 128 + 10, 1),
        (64 + 128 + 16, 1),
    ] {
        let mut corrupt = original.clone();
        corrupt[offset] = replacement;
        disk.0.borrow_mut().bytes = Some(corrupt);
        assert!(matches!(
            Service::recover(disk.clone(), &mut entropy, &mut work),
            Err(Error::Corrupt)
        ));
    }
    for range in [16..24, 24..56, 64..96, 104..112, 112..116, 120..152] {
        let mut corrupt = original.clone();
        corrupt[range].fill(0);
        disk.0.borrow_mut().bytes = Some(corrupt);
        assert!(matches!(
            Service::recover(disk.clone(), &mut entropy, &mut work),
            Err(Error::Corrupt)
        ));
    }
}

#[test]
fn entropy_failure_or_repeating_namespace_never_publishes_an_invitation() {
    let mut work = Box::new(Workspace::new());
    struct Broken;
    impl Entropy for Broken {
        fn fill(&mut self, _: &mut [u8; 32]) -> Result<(), Error> {
            Err(Error::Entropy)
        }
    }
    assert!(matches!(
        Service::initialize(Memory::default(), ADMIN, &mut Broken, &mut work),
        Err(Error::Entropy)
    ));
    let (mut service, disk, mut entropy) = fixture(&mut work);
    let invitation = pair(&mut service, &mut entropy, ALICE);
    struct Repeated([u8; 32]);
    impl Entropy for Repeated {
        fn fill(&mut self, bytes: &mut [u8; 32]) -> Result<(), Error> {
            *bytes = self.0;
            Ok(())
        }
    }
    let writes = disk.0.borrow().writes;
    for bytes in [[0; 32], invitation.namespace] {
        assert!(matches!(
            service.invite(ADMIN, BOB, 1, 0, 1, &mut Repeated(bytes)),
            Err(Error::Entropy)
        ));
    }
    assert_eq!(disk.0.borrow().writes, writes);
}

#[test]
#[cfg(not(debug_assertions))]
fn optimized_core_fits_the_existing_64_kib_native_stack() {
    println!(
        "caller workspace={} bytes; service={} bytes",
        core::mem::size_of::<Workspace>(),
        core::mem::size_of::<Service<'_, Memory>>()
    );
    let mut work = Box::new(Workspace::new());
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(move || {
            let (mut service, _, mut entropy) = fixture(&mut work);
            let invite = pair(&mut service, &mut entropy, ALICE);
            let session = service.open(ALICE, &invite.namespace).unwrap();
            let upload = service
                .stage(ALICE, session, 0, 0, &[7; MAX_CIPHERTEXT])
                .unwrap();
            service.commit(ALICE, session, upload).unwrap();
            service.revoke(ADMIN, &invite.namespace).unwrap();
            service.evict(ADMIN, &invite.namespace).unwrap();
            let service =
                Service::recover(service.into_storage(), &mut entropy, &mut work).unwrap();
            assert_eq!(
                service.usage(ADMIN, &invite.namespace).unwrap().status,
                NamespaceStatus::Evicted
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
