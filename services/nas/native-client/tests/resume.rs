use std::{cell::RefCell, rc::Rc};

use nieuw_os_nas_client::{Client, State, PENDING};
use nieuw_os_nas_core::{
    Entropy, Error, Principal, Service, Storage, Workspace, MAX_CIPHERTEXT, MAX_QUOTA, OBJECTS,
    SESSIONS, SNAPSHOT_BYTES,
};

const ADMIN: Principal = Principal([1; 32]);
const OWNER: Principal = Principal([2; 32]);
const WRONG: Principal = Principal([3; 32]);

struct TestEntropy(u8);
impl Entropy for TestEntropy {
    fn fill(&mut self, out: &mut [u8; 32]) -> Result<(), Error> {
        self.0 = self.0.checked_add(1).ok_or(Error::Entropy)?;
        out.fill(self.0);
        Ok(())
    }
}

#[derive(Default)]
struct Disk {
    bytes: Option<Vec<u8>>,
    writes: usize,
    failure: Option<(Error, bool)>,
}

#[derive(Clone, Default)]
struct Memory(Rc<RefCell<Disk>>);
impl Storage for Memory {
    fn load(&mut self, out: &mut [u8; SNAPSHOT_BYTES]) -> Result<Option<usize>, Error> {
        let disk = self.0.borrow();
        let Some(bytes) = &disk.bytes else {
            return Ok(None);
        };
        out.copy_from_slice(bytes);
        Ok(Some(bytes.len()))
    }

    fn replace(&mut self, snapshot: &[u8; SNAPSHOT_BYTES]) -> Result<(), Error> {
        let mut disk = self.0.borrow_mut();
        disk.writes += 1;
        let failure = disk.failure.take();
        if failure.is_none_or(|(_, committed)| committed) {
            disk.bytes = Some(snapshot.to_vec());
        }
        match failure {
            Some((error, _)) => Err(error),
            None => Ok(()),
        }
    }
}

fn fixture(work: &mut Workspace, quota: u32) -> (Service<'_, Memory>, Memory, [u8; 32]) {
    let disk = Memory::default();
    let mut entropy = TestEntropy(10);
    let mut service = Service::initialize(disk.clone(), ADMIN, &mut entropy, work).unwrap();
    let invite = service
        .invite(ADMIN, OWNER, quota, 100, 10, &mut entropy)
        .unwrap();
    service.accept(OWNER, &invite, 100).unwrap();
    (service, disk, invite.namespace)
}

fn publish(client: &mut Client, service: &mut Service<'_, Memory>, slot: usize) -> u64 {
    client.prepare(slot, service, OWNER).unwrap();
    client.commit(slot, service, OWNER).unwrap()
}

#[test]
fn offline_queue_has_no_server_calls_and_preserves_every_rejected_edit() {
    let mut work = Box::new(Workspace::new());
    let (_service, disk, ns) = fixture(&mut work, MAX_QUOTA);
    let writes = disk.0.borrow().writes;
    let mut client = Client::new();
    for object in 0..PENDING {
        assert_eq!(
            client.enqueue(ns, object, 0, &[object as u8; 3]),
            Ok(object)
        );
    }
    assert_eq!(
        client.enqueue([9; 32], 0, 0, b"new").unwrap_err().operation,
        Error::Full
    );
    assert_eq!(
        client
            .enqueue(ns, 0, 0, b"replacement")
            .unwrap_err()
            .operation,
        Error::Conflict
    );
    for object in 0..PENDING {
        assert_eq!(
            client.edit(object).unwrap().local_ciphertext(),
            &[object as u8; 3]
        );
    }
    assert_eq!(disk.0.borrow().writes, writes);
}

#[test]
fn normal_publish_is_acknowledged_only_after_the_existing_core_commit() {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, ns) = fixture(&mut work, MAX_QUOTA);
    let mut client = Client::new();
    let slot = client.enqueue(ns, 0, 0, b"opaque local edit").unwrap();
    let writes = disk.0.borrow().writes;
    client.prepare(slot, &mut service, OWNER).unwrap();
    assert_eq!(client.edit(slot).unwrap().state(), State::Staged);
    assert_eq!(disk.0.borrow().writes, writes);
    assert_eq!(
        client.take(slot).unwrap_err().operation,
        Error::ReopenRequired
    );
    assert_eq!(client.commit(slot, &mut service, OWNER), Ok(1));
    assert_eq!(client.edit(slot).unwrap().state(), State::Published(1));
    assert_eq!(disk.0.borrow().writes, writes + 1);
    assert_eq!(
        client
            .prepare(slot, &mut service, OWNER)
            .unwrap_err()
            .operation,
        Error::ReopenRequired
    );
    assert_eq!(
        client.take(slot).unwrap().local_ciphertext(),
        b"opaque local edit"
    );
    assert_eq!(client.enqueue(ns, 0, 1, b"next"), Ok(slot));
}

#[test]
fn concurrent_writers_preserve_both_sides_without_automatic_overwrite() {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, ns) = fixture(&mut work, MAX_QUOTA);
    let mut a = Client::new();
    let mut b = Client::new();
    let x = a.enqueue(ns, 0, 0, b"writer A").unwrap();
    let y = b.enqueue(ns, 0, 0, b"writer B").unwrap();
    a.prepare(x, &mut service, OWNER).unwrap();
    b.prepare(y, &mut service, OWNER).unwrap();
    assert_eq!(a.commit(x, &mut service, OWNER), Ok(1));
    assert_eq!(
        b.commit(y, &mut service, OWNER).unwrap_err().operation,
        Error::Conflict
    );
    let writes = disk.0.borrow().writes;
    assert_eq!(b.reconcile(y, &mut service, OWNER, 1), Ok(State::Conflict));
    assert_eq!(b.edit(y).unwrap().local_ciphertext(), b"writer B");
    assert_eq!(
        b.edit(y).unwrap().remote_ciphertext(),
        Some((1, b"writer A".as_slice()))
    );
    assert_eq!(
        b.prepare(y, &mut service, OWNER).unwrap_err().operation,
        Error::ReopenRequired
    );
    assert_eq!(disk.0.borrow().writes, writes);
    let retained = b.take(y).unwrap();
    // A real object adapter must re-seal under the NEW revision context.
    let next = b
        .enqueue(ns, 0, 1, b"writer B resealed for revision 2")
        .unwrap();
    assert_eq!(publish(&mut b, &mut service, next), 2);
    assert_eq!(
        retained.remote_ciphertext(),
        Some((1, b"writer A".as_slice()))
    );
}

#[test]
fn every_storage_error_before_and_after_commit_blocks_replay_until_reconcile() {
    let errors = [
        Error::Denied,
        Error::Invalid,
        Error::Full,
        Error::Quota,
        Error::Conflict,
        Error::Expired,
        Error::NotFound,
        Error::Stale,
        Error::Entropy,
        Error::Corrupt,
        Error::Io,
        Error::Unknown,
        Error::ReopenRequired,
    ];
    for error in errors {
        for committed in [false, true] {
            for checkpoint_before_dispatch in [false, true] {
                let mut work = Box::new(Workspace::new());
                let (mut service, disk, ns) = fixture(&mut work, MAX_QUOTA);
                let mut client = Client::new();
                let slot = client.enqueue(ns, 0, 0, b"uncertain opaque bytes").unwrap();
                let before = client.checkpoint();
                client.prepare(slot, &mut service, OWNER).unwrap();
                disk.0.borrow_mut().failure = Some((error, committed));
                let failure = client.commit(slot, &mut service, OWNER).unwrap_err();
                assert_eq!(failure.operation, Error::Unknown);
                assert_eq!(failure.cleanup, Some(Error::ReopenRequired));
                assert_eq!(service.last_storage_error(), Some(error));
                assert_eq!(client.edit(slot).unwrap().state(), State::NeedsReconcile);
                let checkpoint = if checkpoint_before_dispatch {
                    before
                } else {
                    client.checkpoint()
                };
                let mut restored = Client::restore(checkpoint);
                let writes = disk.0.borrow().writes;
                assert_eq!(
                    restored
                        .prepare(slot, &mut service, OWNER)
                        .unwrap_err()
                        .operation,
                    Error::ReopenRequired
                );
                let _old_storage = service.into_storage();
                let mut recovered =
                    Service::recover(disk.clone(), &mut TestEntropy(100), &mut work).unwrap();
                let observed = restored
                    .reconcile(slot, &mut recovered, OWNER, u64::from(committed))
                    .unwrap();
                assert_eq!(
                    observed,
                    if committed {
                        State::Observed(1)
                    } else {
                        State::Ready
                    }
                );
                assert_eq!(disk.0.borrow().writes, writes);
                assert_eq!(
                    restored.edit(slot).unwrap().local_ciphertext(),
                    b"uncertain opaque bytes"
                );
                if committed {
                    assert_eq!(
                        restored
                            .commit(slot, &mut recovered, OWNER)
                            .unwrap_err()
                            .operation,
                        Error::ReopenRequired
                    );
                } else {
                    assert_eq!(publish(&mut restored, &mut recovered, slot), 1);
                    assert_eq!(disk.0.borrow().writes, writes + 1);
                }
            }
        }
    }
}

#[test]
fn lost_ack_with_a_pre_dispatch_checkpoint_does_not_duplicate_a_committed_write() {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, ns) = fixture(&mut work, MAX_QUOTA);
    let mut client = Client::new();
    let slot = client.enqueue(ns, 0, 0, b"already stored").unwrap();
    let checkpoint = client.checkpoint();
    assert_eq!(publish(&mut client, &mut service, slot), 1);
    let writes = disk.0.borrow().writes;
    let _old_storage = service.into_storage();
    let mut recovered = Service::recover(disk.clone(), &mut TestEntropy(100), &mut work).unwrap();
    let mut restored = Client::restore(checkpoint);
    assert_eq!(
        restored.reconcile(slot, &mut recovered, OWNER, 1),
        Ok(State::Observed(1))
    );
    assert_eq!(disk.0.borrow().writes, writes);
}

#[test]
fn partial_upload_and_offline_edits_survive_host_model_recovery_without_live_tickets() {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, ns) = fixture(&mut work, MAX_QUOTA);
    let mut client = Client::new();
    let staged = client.enqueue(ns, 0, 0, b"partly uploaded").unwrap();
    let offline = client.enqueue(ns, 1, 0, b"edited offline").unwrap();
    client.prepare(staged, &mut service, OWNER).unwrap();
    let checkpoint = client.checkpoint();
    let writes = disk.0.borrow().writes;
    let _old_storage = service.into_storage();
    let mut recovered = Service::recover(disk.clone(), &mut TestEntropy(100), &mut work).unwrap();
    let mut restored = Client::restore(checkpoint);
    for slot in [staged, offline] {
        assert_eq!(
            restored
                .commit(slot, &mut recovered, OWNER)
                .unwrap_err()
                .operation,
            Error::ReopenRequired
        );
        assert_eq!(
            restored.reconcile(slot, &mut recovered, OWNER, 0),
            Ok(State::Ready)
        );
    }
    assert_eq!(disk.0.borrow().writes, writes);
    assert_eq!(publish(&mut restored, &mut recovered, staged), 1);
    assert_eq!(publish(&mut restored, &mut recovered, offline), 1);
    assert_eq!(recovered.usage(ADMIN, &ns).unwrap().reserved, 0);
}

#[test]
fn revocation_and_cold_core_recovery_never_restore_old_client_authority() {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, ns) = fixture(&mut work, MAX_QUOTA);
    let mut client = Client::new();
    let slot = client.enqueue(ns, 0, 0, b"keep my local copy").unwrap();
    client.prepare(slot, &mut service, OWNER).unwrap();
    service.revoke(ADMIN, &ns).unwrap();
    assert_eq!(
        client
            .commit(slot, &mut service, OWNER)
            .unwrap_err()
            .operation,
        Error::Stale
    );
    service.evict(ADMIN, &ns).unwrap();
    let checkpoint = client.checkpoint();
    let writes = disk.0.borrow().writes;
    let _old_storage = service.into_storage();
    let mut recovered = Service::recover(disk.clone(), &mut TestEntropy(100), &mut work).unwrap();
    let mut restored = Client::restore(checkpoint);
    assert_eq!(
        restored
            .reconcile(slot, &mut recovered, OWNER, 0)
            .unwrap_err()
            .operation,
        Error::Denied
    );
    assert_eq!(
        restored
            .prepare(slot, &mut recovered, OWNER)
            .unwrap_err()
            .operation,
        Error::ReopenRequired
    );
    assert_eq!(
        restored.edit(slot).unwrap().state(),
        State::Blocked(Error::Denied)
    );
    assert_eq!(
        restored.take(slot).unwrap().local_ciphertext(),
        b"keep my local copy"
    );
    assert_eq!(disk.0.borrow().writes, writes);
    assert_eq!(recovered.usage(ADMIN, &ns).unwrap().used, 0);
}

#[test]
fn wrong_caller_and_revision_hints_never_become_authority() {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, ns) = fixture(&mut work, MAX_QUOTA);
    let mut client = Client::new();
    let slot = client.enqueue(ns, 0, 0, b"private opaque edit").unwrap();
    let writes = disk.0.borrow().writes;
    assert_eq!(
        client
            .prepare(slot, &mut service, WRONG)
            .unwrap_err()
            .operation,
        Error::Denied
    );
    assert_eq!(
        client
            .reconcile(slot, &mut service, WRONG, 0)
            .unwrap_err()
            .operation,
        Error::Denied
    );
    assert_eq!(
        client
            .reconcile(slot, &mut service, OWNER, 99)
            .unwrap_err()
            .operation,
        Error::NotFound
    );
    assert_eq!(
        client.edit(slot).unwrap().local_ciphertext(),
        b"private opaque edit"
    );
    assert_eq!(disk.0.borrow().writes, writes);
    assert_eq!(
        client.reconcile(slot, &mut service, OWNER, 0),
        Ok(State::Ready)
    );
    assert_eq!(publish(&mut client, &mut service, slot), 1);
}

#[test]
fn separate_namespaces_do_not_inherit_the_other_queued_objects_session() {
    let mut work = Box::new(Workspace::new());
    let (mut service, _disk, a) = fixture(&mut work, MAX_QUOTA);
    let invite = service
        .invite(ADMIN, OWNER, MAX_QUOTA, 100, 10, &mut TestEntropy(50))
        .unwrap();
    service.accept(OWNER, &invite, 100).unwrap();
    let mut client = Client::new();
    let x = client.enqueue(a, 0, 0, b"namespace A").unwrap();
    let y = client
        .enqueue(invite.namespace, 0, 0, b"namespace B")
        .unwrap();
    client.prepare(x, &mut service, OWNER).unwrap();
    client.prepare(y, &mut service, OWNER).unwrap();
    assert_eq!(client.commit(y, &mut service, OWNER), Ok(1));
    assert_eq!(client.commit(x, &mut service, OWNER), Ok(1));
    for (ns, expected) in [(a, b"namespace A"), (invite.namespace, b"namespace B")] {
        let session = service.open(OWNER, &ns).unwrap();
        let mut out = [0; MAX_CIPHERTEXT];
        let length = service.read(OWNER, session, 0, 1, &mut out).unwrap();
        assert_eq!(&out[..length], expected);
        service.close(OWNER, session).unwrap();
    }
}

#[test]
fn quota_failures_release_sessions_and_keep_the_pending_local_bytes() {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, ns) = fixture(&mut work, 2);
    let mut client = Client::new();
    let slot = client.enqueue(ns, 0, 0, b"larger than quota").unwrap();
    let writes = disk.0.borrow().writes;
    for _ in 0..SESSIONS * 2 {
        assert_eq!(
            client
                .prepare(slot, &mut service, OWNER)
                .unwrap_err()
                .operation,
            Error::Quota
        );
        assert_eq!(
            client.reconcile(slot, &mut service, OWNER, 0),
            Ok(State::Ready)
        );
    }
    assert_eq!(service.usage(ADMIN, &ns).unwrap().reserved, 0);
    assert_eq!(disk.0.borrow().writes, writes);
    service.set_quota(ADMIN, &ns, 64).unwrap();
    assert_eq!(publish(&mut client, &mut service, slot), 1);
}

#[test]
fn cancelling_staging_frees_quota_and_sessions_without_deleting_the_local_edit() {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, ns) = fixture(&mut work, MAX_QUOTA);
    let mut client = Client::new();
    let slot = client.enqueue(ns, 0, 0, b"keep on cancel").unwrap();
    let writes = disk.0.borrow().writes;
    for _ in 0..SESSIONS * 2 {
        client.prepare(slot, &mut service, OWNER).unwrap();
        assert_eq!(service.usage(ADMIN, &ns).unwrap().reserved, 14);
        client.cancel(slot, &mut service, OWNER).unwrap();
        assert_eq!(client.edit(slot).unwrap().state(), State::Ready);
        assert_eq!(
            client.edit(slot).unwrap().local_ciphertext(),
            b"keep on cancel"
        );
        assert_eq!(service.usage(ADMIN, &ns).unwrap().reserved, 0);
    }
    assert_eq!(disk.0.borrow().writes, writes);
    assert_eq!(publish(&mut client, &mut service, slot), 1);
}

fn misrouted_calls_preserve_tickets(cancel: bool) {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, ns) = fixture(&mut work, MAX_QUOTA);
    let mut client = Client::new();
    let mut base = 0;
    for _ in 0..SESSIONS * 2 {
        let slot = client.enqueue(ns, 0, base, b"owner retained data").unwrap();
        client.prepare(slot, &mut service, OWNER).unwrap();
        let writes = disk.0.borrow().writes;
        let error = if cancel {
            client.cancel(slot, &mut service, WRONG).unwrap_err()
        } else {
            client.commit(slot, &mut service, WRONG).unwrap_err()
        };
        assert_eq!(error.operation, Error::Denied);
        assert_eq!(error.cleanup, None);
        assert_eq!(client.edit(slot).unwrap().state(), State::Staged);
        assert_eq!(
            client.take(slot).unwrap_err().operation,
            Error::ReopenRequired
        );
        assert_eq!(service.usage(ADMIN, &ns).unwrap().reserved, 19);
        assert_eq!(disk.0.borrow().writes, writes);
        if cancel {
            client.cancel(slot, &mut service, OWNER).unwrap();
            assert_eq!(disk.0.borrow().writes, writes);
        } else {
            base += 1;
            assert_eq!(client.commit(slot, &mut service, OWNER), Ok(base));
            assert_eq!(disk.0.borrow().writes, writes + 1);
        }
        assert_eq!(service.usage(ADMIN, &ns).unwrap().reserved, 0);
        let retained = client.take(slot).unwrap();
        assert_eq!(retained.namespace(), &ns);
        assert_eq!(retained.object(), 0);
        assert_eq!(retained.local_ciphertext(), b"owner retained data");
    }
}

#[test]
fn wrong_caller_cannot_commit_or_consume_the_authorized_callers_staging() {
    misrouted_calls_preserve_tickets(false);
}

#[test]
fn wrong_caller_cannot_cancel_or_consume_the_authorized_callers_staging() {
    misrouted_calls_preserve_tickets(true);
}

#[test]
fn older_observed_revision_blocks_retry_without_erasing_either_version() {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, ns) = fixture(&mut work, MAX_QUOTA);
    let mut original = Client::new();
    let initial = original.enqueue(ns, 0, 0, b"remote revision 1").unwrap();
    publish(&mut original, &mut service, initial);
    let mut client = Client::new();
    let slot = client.enqueue(ns, 0, 2, b"local known base 2").unwrap();
    let writes = disk.0.borrow().writes;
    assert_eq!(
        client
            .reconcile(slot, &mut service, OWNER, 1)
            .unwrap_err()
            .operation,
        Error::Stale
    );
    assert_eq!(
        client.edit(slot).unwrap().state(),
        State::Blocked(Error::Stale)
    );
    assert_eq!(
        client.edit(slot).unwrap().remote_ciphertext(),
        Some((1, b"remote revision 1".as_slice()))
    );
    assert_eq!(
        client.edit(slot).unwrap().local_ciphertext(),
        b"local known base 2"
    );
    assert_eq!(disk.0.borrow().writes, writes);
}

#[test]
fn invalid_lengths_indices_and_revision_overflow_are_explicit_and_nonmutating() {
    let mut work = Box::new(Workspace::new());
    let (mut service, disk, ns) = fixture(&mut work, MAX_QUOTA);
    let mut client = Client::new();
    for (namespace, object, base, bytes) in [
        ([0; 32], 0, 0, b"x".as_slice()),
        (ns, OBJECTS, 0, b"x".as_slice()),
        (ns, 0, u64::MAX, b"x".as_slice()),
        (ns, 0, 0, b"".as_slice()),
        (ns, 0, 0, &[1; MAX_CIPHERTEXT + 1]),
    ] {
        assert_eq!(
            client
                .enqueue(namespace, object, base, bytes)
                .unwrap_err()
                .operation,
            Error::Invalid
        );
    }
    let writes = disk.0.borrow().writes;
    for slot in [0, PENDING, usize::MAX] {
        assert_eq!(
            client
                .prepare(slot, &mut service, OWNER)
                .unwrap_err()
                .operation,
            Error::Invalid
        );
        assert_eq!(
            client
                .commit(slot, &mut service, OWNER)
                .unwrap_err()
                .operation,
            Error::Invalid
        );
        assert_eq!(
            client
                .reconcile(slot, &mut service, OWNER, 0)
                .unwrap_err()
                .operation,
            Error::Invalid
        );
        assert_eq!(client.take(slot).unwrap_err().operation, Error::Invalid);
    }
    let slot = client.enqueue(ns, 0, 0, &[7; MAX_CIPHERTEXT]).unwrap();
    assert_eq!(
        client.edit(slot).unwrap().local_ciphertext().len(),
        MAX_CIPHERTEXT
    );
    assert_eq!(disk.0.borrow().writes, writes);
}

#[test]
fn default_debug_never_prints_payloads_or_namespace_identifiers() {
    let mut client = Client::new();
    let slot = client.enqueue([7; 32], 0, 0, b"DO-NOT-LOG-ME").unwrap();
    let text = format!("{:?}", client.edit(slot).unwrap());
    assert_eq!(text, "Edit { state: Ready, .. }");
    assert!(!text.contains("DO-NOT-LOG-ME"));
}

#[test]
#[cfg(not(debug_assertions))]
fn optimized_maximum_queue_and_recovery_fit_a_64_kib_host_stack() {
    use nieuw_os_nas_client::Checkpoint;

    let client_bytes = core::mem::size_of::<Client>();
    let checkpoint_bytes = core::mem::size_of::<Checkpoint>();
    assert!(client_bytes <= 12 * 1024);
    assert!(checkpoint_bytes <= 12 * 1024);
    println!("client={client_bytes} bytes; RAM checkpoint={checkpoint_bytes} bytes");
    // As with the core's existing stack test, its arena is caller-owned heap.
    let mut work = Box::new(Workspace::new());
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(move || {
            let (mut service, disk, ns) = fixture(&mut work, MAX_QUOTA);
            let mut client = Client::new();
            for object in 0..OBJECTS {
                let slot = client
                    .enqueue(ns, object, 0, &[object as u8; MAX_CIPHERTEXT])
                    .unwrap();
                client.prepare(slot, &mut service, OWNER).unwrap();
            }
            assert_eq!(service.usage(ADMIN, &ns).unwrap().reserved, MAX_QUOTA);
            let checkpoint = client.checkpoint();
            let writes = disk.0.borrow().writes;
            let _old_storage = service.into_storage();
            let mut recovered =
                Service::recover(disk.clone(), &mut TestEntropy(100), &mut work).unwrap();
            let mut restored = Client::restore(checkpoint);
            for object in 0..OBJECTS {
                assert_eq!(
                    restored.reconcile(object, &mut recovered, OWNER, 0),
                    Ok(State::Ready)
                );
                assert_eq!(
                    restored.edit(object).unwrap().local_ciphertext().len(),
                    MAX_CIPHERTEXT
                );
                assert_eq!(publish(&mut restored, &mut recovered, object), 1);
            }
            assert_eq!(disk.0.borrow().writes, writes + OBJECTS);
            let usage = recovered.usage(ADMIN, &ns).unwrap();
            assert_eq!(usage.used, MAX_QUOTA);
            assert_eq!(usage.reserved, 0);
        })
        .unwrap()
        .join()
        .unwrap();
}
