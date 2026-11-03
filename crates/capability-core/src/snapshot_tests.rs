//! Exact authority snapshots; Engine clones below are host-only, on a large
//! test stack. Kernel callers reserve AuthoritySnapshot statically instead.
extern crate std;
use super::*;
use std::thread;

fn run(f: impl FnOnce(&mut Engine, &mut AuthoritySnapshot) + Send + 'static) {
    thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let mut engine = Engine::new();
            let mut snapshot = AuthoritySnapshot::new();
            f(&mut engine, &mut snapshot);
        })
        .unwrap()
        .join()
        .unwrap();
}
fn same(engine: &Engine, snapshot: &AuthoritySnapshot) {
    let reference = engine.clone();
    assert!(snapshot.matches(engine));
    assert_eq!(snapshot.state, reference);
    assert_eq!(snapshot.state.stats(), engine.stats());
    assert_eq!(
        snapshot.state.state_digest(),
        engine.state_digest_reference()
    );
}
#[test]
fn snapshots_match_full_clone_and_fresh_oracles_through_real_lifecycle() {
    run(|e, s| {
        e.snapshot_into(s);
        same(e, s);
        e.register(1).unwrap();
        e.register(2).unwrap();
        let root = e.create_endpoint(1).unwrap();
        let send = e.bootstrap_delegate(1, root, 2, SEND).unwrap();
        for byte in 0..20 {
            let mut message = Message::new();
            message.len = 256;
            message.data.fill(byte);
            e.send(2, send, &message).unwrap();
            e.snapshot_into(s);
            same(e, s);
            assert_eq!(e.receive(1, root).unwrap().data, [byte; 256]);
        }
        assert!(!s.matches(e));
        e.snapshot_into(s);
        same(e, s);
        e.close(1, root).unwrap();
        e.exit(2).unwrap();
        e.exit(1).unwrap();
        e.snapshot_into(s);
        same(e, s);
        assert!(e.check_invariants());
    });
}
#[test]
fn every_authority_field_and_inactive_message_is_compared_exactly() {
    run(|e, s| {
        e.snapshot_into(s);
        macro_rules! change {
            ($field:expr, $value:expr) => {{
                let old = $field;
                $field = $value;
                assert!(!s.matches(e), "missed {}", stringify!($field));
                assert_ne!(*e, s.state);
                $field = old;
                assert!(s.matches(e));
            }};
        }
        change!(e.last_pid, 9);
        change!(e.processes[0].pid, 1);
        change!(e.processes[0].live, true);
        change!(e.processes[0].slots[0].generation, 7);
        change!(e.processes[0].slots[0].retired, true);
        change!(e.processes[0].slots[0].node.index, 1);
        change!(e.processes[0].slots[0].node.generation, 7);
        change!(e.nodes[0].generation, 7);
        change!(e.nodes[0].live, true);
        change!(e.nodes[0].retired, true);
        change!(e.nodes[0].revoked, true);
        change!(e.nodes[0].object.index, 1);
        change!(e.nodes[0].object.generation, 7);
        change!(e.nodes[0].parent.index, 1);
        change!(e.nodes[0].parent.generation, 7);
        change!(e.nodes[0].issuer, 1);
        change!(e.nodes[0].rights, 1);
        change!(e.nodes[0].refs, 1);
        change!(e.nodes[0].depth, 1);
        change!(e.endpoints[0].generation, 7);
        change!(e.endpoints[0].live, true);
        change!(e.endpoints[0].retired, true);
        change!(e.endpoints[0].open, true);
        change!(e.endpoints[0].owner, 1);
        change!(e.endpoints[0].nodes, 1);
        change!(e.endpoints[0].head, 1);
        change!(e.endpoints[0].len, 1);
        // Last queue cell of a non-live endpoint must be covered as well.
        change!(e.endpoints[63].queue[15].message.sender_pid, 1);
        change!(e.endpoints[63].queue[15].message.tag, 1);
        change!(e.endpoints[63].queue[15].message.len, 1);
        change!(e.endpoints[63].queue[15].message.cap_count, 1);
        change!(e.endpoints[63].queue[15].message.reserved, 1);
        change!(e.endpoints[63].queue[15].message.caps[0].handle, 1);
        change!(e.endpoints[63].queue[15].message.caps[0].rights, 1);
        change!(e.endpoints[63].queue[15].message.caps[1].handle, 1);
        change!(e.endpoints[63].queue[15].message.caps[1].rights, 1);
        change!(e.endpoints[63].queue[15].message.data[255], 1);
        change!(e.endpoints[63].queue[15].caps[0].index, 1);
        change!(e.endpoints[63].queue[15].caps[0].generation, 7);
        change!(e.endpoints[63].queue[15].caps[1].index, 1);
        change!(e.endpoints[63].queue[15].caps[1].generation, 7);
        change!(e.blocks[0].generation, 7);
        change!(e.blocks[0].live, true);
        change!(e.blocks[0].retired, true);
        change!(e.blocks[0].open, true);
        change!(e.blocks[0].owner, 1);
        change!(e.blocks[0].nodes, 1);
        change!(e.blocks[0].scope.device, 1);
        change!(e.blocks[0].scope.first_sector, 1);
        change!(e.blocks[0].scope.sector_count, 1);
        change!(e.launches[0].generation, 7);
        change!(e.launches[0].live, true);
        change!(e.launches[0].retired, true);
        change!(e.launches[0].open, true);
        change!(e.launches[0].owner, 1);
        change!(e.launches[0].nodes, 1);
        change!(e.launches[0].image, 1);
        change!(e.children[0].generation, 7);
        change!(e.children[0].live, true);
        change!(e.children[0].retired, true);
        change!(e.children[0].open, true);
        change!(e.children[0].owner, 1);
        change!(e.children[0].nodes, 1);
        change!(e.children[0].child, 1);
        change!(e.children[0].state, 1);
        change!(e.children[0].code, 1);
        macro_rules! source {
            ($name:ident) => {
                change!(e.$name[0].generation, 7);
                change!(e.$name[0].live, true);
                change!(e.$name[0].retired, true);
                change!(e.$name[0].open, true);
                change!(e.$name[0].owner, 1);
                change!(e.$name[0].nodes, 1);
            };
        }
        source!(entropy);
        source!(network);
        source!(display);
        source!(input);
    });
}
#[test]
fn snapshots_preserve_failed_operation_and_detect_unexpected_error_write() {
    run(|e, s| {
        e.register(1).unwrap();
        let root = e.create_endpoint(1).unwrap();
        for _ in 0..QUEUE_CAPACITY {
            e.send(1, root, &Message::new()).unwrap();
        }
        e.snapshot_into(s);
        let reference = e.clone();
        assert_eq!(e.send(1, root, &Message::new()), Err(Error::Again));
        assert_eq!(*e, reference);
        same(e, s);
        assert_eq!(e.register(1), Err(Error::Invalid));
        same(e, s);
        assert_eq!(e.receive(2, root), Err(Error::BadHandle));
        same(e, s);
        let mut invalid = Message::new();
        invalid.reserved = 1;
        assert_eq!(e.send(1, root, &invalid), Err(Error::Invalid));
        same(e, s);
        // Simulate a bug after a snapshot: no mutation hook/cache invalidation
        // exists or is required for the error check to notice this write.
        e.nodes[0].refs += 1;
        assert!(!s.matches(e));
    });
}
#[test]
fn snapshot_message_erasure_covers_all_headers_descriptors_and_payload_bytes() {
    run(|e, s| {
        let mut marker = Message::new();
        marker.sender_pid = 9;
        marker.tag = 8;
        marker.len = 7;
        marker.cap_count = 2;
        marker.reserved = 6;
        marker.caps = [Cap {
            handle: 5,
            rights: 4,
        }; 2];
        marker.data.fill(0xa5);
        for endpoint in &mut e.endpoints {
            for queued in &mut endpoint.queue {
                queued.message = marker;
            }
        }
        e.snapshot_into(s);
        assert_eq!(s.messages_mut().count(), MAX_ENDPOINTS * QUEUE_CAPACITY);
        for message in s.messages_mut() {
            *message = Message::new();
        }
        assert!(s.messages_mut().all(|message| *message == Message::new()));
        assert!(e
            .endpoints
            .iter()
            .all(|endpoint| endpoint.queue.iter().all(|q| q.message == marker)));
    });
}
#[test]
fn snapshot_storage_rejects_reentrancy_and_releases_on_scope_exit() {
    let lock = SnapshotLock::new();
    let first = lock.try_claim().unwrap();
    assert!(lock.try_claim().is_none());
    let independent = SnapshotLock::new();
    let other = independent.try_claim().unwrap();
    drop(first);
    assert!(lock.try_claim().is_some());
    drop(other);
    assert!(independent.try_claim().is_some());
}
