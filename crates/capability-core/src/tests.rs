extern crate std;
use super::*;
use std::{thread, vec::Vec};

// Large fixed tables deliberately live on a large host-test thread stack. The
// kernel uses const static initialization and never constructs them on a stack.
fn run(f: impl FnOnce(&mut Engine) + Send + 'static) {
    thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let mut e = Engine::new();
            f(&mut e);
            assert!(e.check_invariants());
        })
        .unwrap()
        .join()
        .unwrap();
}
fn setup(e: &mut Engine, n: u64) {
    for pid in 1..=n {
        e.register(pid).unwrap();
    }
}
fn assert_atomic<T: core::fmt::Debug>(
    e: &mut Engine,
    expected: Error,
    f: impl FnOnce(&mut Engine) -> Result<T, Error>,
) {
    let before = e.clone();
    assert_eq!(f(e).unwrap_err(), expected);
    assert!(
        *e == before,
        "failure changed authority, resources or generations"
    );
    assert!(e.check_invariants());
}
fn cap_message(handle: u64, rights: u64) -> Message {
    let mut m = Message::new();
    m.len = 1;
    m.data[0] = 42;
    m.cap_count = 1;
    m.caps[0] = Cap { handle, rights };
    m
}
fn destination(e: &mut Engine) -> (u64, u64) {
    let root = e.create_endpoint(2).unwrap();
    let send = e.bootstrap_delegate(2, root, 1, SEND).unwrap();
    (root, send)
}

#[test]
fn m6_explicit_write_flush_roots_preserve_legacy_rights_and_type_boundary() {
    run(|e| {
        setup(e, 2);
        assert_eq!(ALL_RIGHTS, 31);
        assert_eq!(BLOCK_READ_RIGHTS, 29);
        assert_eq!(BLOCK_RIGHTS, 125);
        let legacy = e.create_block_device(1, block_scope()).unwrap();
        for required in [BLOCK_WRITE, BLOCK_FLUSH, BLOCK_WRITE | BLOCK_FLUSH] {
            assert_atomic(e, Error::Access, |e| e.check_block(1, legacy, required));
        }
        let root = e
            .create_block_device_with_rights(1, block_scope(), BLOCK_RIGHTS)
            .unwrap();
        let endpoint = e.create_endpoint(1).unwrap();
        assert_atomic(e, Error::Type, |e| e.check_block(1, endpoint, BLOCK_WRITE));
        assert_atomic(e, Error::Type, |e| e.check_endpoint(1, root, SEND));
        assert_atomic(e, Error::Invalid, |e| e.derive(1, endpoint, BLOCK_WRITE));
        let rwf = BLOCK_READ | BLOCK_WRITE | BLOCK_FLUSH;
        assert_eq!(rwf, 97);
        let service = e.bootstrap_delegate(1, root, 2, rwf).unwrap();
        e.close(1, root).unwrap();
        assert_eq!(
            e.check_block(2, service, rwf).unwrap().scope(),
            block_scope()
        );
        assert_atomic(e, Error::Access, |e| e.derive(2, service, rwf));
        assert_atomic(e, Error::Access, |e| e.derive(2, service, 128));
    });
}

#[test]
fn m6_manifest_masks_and_subset_failures_are_transactional() {
    run(|e| {
        setup(e, 2);
        for rights in [RECV, 128, u64::MAX] {
            assert_atomic(e, Error::Invalid, |e| {
                e.create_block_device_with_rights(1, block_scope(), rights)
            });
        }
        let root = e
            .create_block_device_with_rights(1, block_scope(), BLOCK_RIGHTS)
            .unwrap();
        for rights in 0..256 {
            if rights & !BLOCK_RIGHTS == 0 {
                let derived = e.derive(1, root, rights).unwrap();
                assert_eq!(
                    e.check_block(1, derived, rights).unwrap().scope(),
                    block_scope()
                );
                e.close(1, derived).unwrap();
            } else {
                assert_atomic(e, Error::Invalid, |e| e.derive(1, root, rights));
            }
        }
        let read = e.derive(1, root, BLOCK_READ | DERIVE | TRANSFER).unwrap();
        for rights in [BLOCK_WRITE, BLOCK_FLUSH] {
            assert_atomic(e, Error::Access, |e| e.derive(1, read, rights));
            assert_atomic(e, Error::Access, |e| {
                e.bootstrap_delegate(1, read, 2, rights)
            });
        }
    });
}

#[test]
fn m6_queued_write_authority_revoke_exit_and_generation_remain_bounded() {
    run(|e| {
        setup(e, 3);
        let (recv, send) = destination(e);
        let root = e
            .create_block_device_with_rights(1, block_scope(), BLOCK_RIGHTS)
            .unwrap();
        let key = e.check_block(1, root, BLOCK_WRITE).unwrap().object_key();
        e.send(1, send, &cap_message(root, BLOCK_WRITE | BLOCK_FLUSH))
            .unwrap();
        let m = e.receive(2, recv).unwrap();
        let transferred = m.caps[0].handle;
        assert_eq!(
            e.check_block(2, transferred, BLOCK_WRITE | BLOCK_FLUSH)
                .unwrap()
                .object_key(),
            key
        );
        assert_atomic(e, Error::Access, |e| {
            e.check_block(2, transferred, BLOCK_READ)
        });
        e.send(1, send, &cap_message(root, BLOCK_WRITE)).unwrap();
        e.revoke(1, root).unwrap();
        assert_atomic(e, Error::Revoked, |e| {
            e.check_block(2, transferred, BLOCK_WRITE)
        });
        assert_atomic(e, Error::Again, |e| e.receive(2, recv));
        e.close(2, transferred).unwrap();
        e.close(1, root).unwrap();
        let replacement = e
            .create_block_device_with_rights(3, block_scope(), BLOCK_RIGHTS)
            .unwrap();
        assert_ne!(
            e.check_block(3, replacement, BLOCK_FLUSH)
                .unwrap()
                .object_key(),
            key
        );
        assert_atomic(e, Error::BadHandle, |e| e.check_block(1, root, BLOCK_WRITE));
        let delegated = e
            .bootstrap_delegate(3, replacement, 2, BLOCK_WRITE)
            .unwrap();
        e.exit(3).unwrap();
        assert_atomic(e, Error::Revoked, |e| {
            e.check_block(2, delegated, BLOCK_WRITE)
        });
        e.close(2, delegated).unwrap();
    });
}

#[test]
fn m6_writable_objects_do_not_reduce_endpoint_or_separate_block_quota() {
    run(|e| {
        setup(e, 16);
        for pid in 1..=16 {
            for _ in 0..4 {
                e.create_endpoint(pid).unwrap();
            }
        }
        for _ in 0..MAX_BLOCK_DEVICES {
            e.create_block_device_with_rights(1, block_scope(), BLOCK_RIGHTS)
                .unwrap();
        }
        assert_eq!(e.stats().endpoints, 64);
        assert_eq!(e.stats().block_objects, 4);
        assert_atomic(e, Error::Quota, |e| {
            e.create_block_device_with_rights(2, block_scope(), BLOCK_RIGHTS)
        });
    });
}

#[test]
fn ks10_wire_exact_offsets_canonical_fields_and_misaligned_byte_source() {
    let mut m = Message::new();
    m.tag = u64::MAX;
    m.len = 256;
    m.cap_count = 2;
    m.data = [0xa5; 256];
    m.caps = [
        Cap {
            handle: u64::MAX,
            rights: ALL_RIGHTS,
        },
        Cap {
            handle: 9,
            rights: 0,
        },
    ];
    let wire = m.encode();
    assert_eq!(wire.len(), 320);
    assert_eq!(&wire[8..16], &u64::MAX.to_le_bytes());
    assert_eq!(&wire[16..20], &256u32.to_le_bytes());
    assert_eq!(&wire[32..40], &u64::MAX.to_le_bytes());
    let mut misaligned = [0u8; 321];
    misaligned[1..].copy_from_slice(&wire);
    let aligned_bytes: &[u8; 320] = misaligned[1..].try_into().unwrap();
    assert_eq!(Message::decode(aligned_bytes), Ok(m));
    for (offset, bytes, expected) in [
        (16, 257u32.to_le_bytes(), Error::MessageSize),
        (20, 3u32.to_le_bytes(), Error::MessageSize),
    ] {
        let mut b = Message::new().encode();
        b[offset..offset + 4].copy_from_slice(&bytes);
        assert_eq!(Message::decode(&b), Err(expected));
    }
    for offset in [0, 24, 32, 40, 48, 56, 64, 319] {
        let mut b = Message::new().encode();
        b[offset] = 1;
        assert_eq!(Message::decode(&b), Err(Error::Invalid));
    }
    // Huge integer fields are bounded before a slice/index operation is used.
    for offset in [16, 20] {
        let mut b = Message::new().encode();
        b[offset..offset + 4].fill(255);
        assert_eq!(Message::decode(&b), Err(Error::MessageSize));
    }
}

#[test]
fn monotone_process_identity_quota_and_process_row_reuse() {
    run(|e| {
        setup(e, 16);
        assert_atomic(e, Error::Quota, |e| e.register(17));
        e.exit(1).unwrap();
        e.register(65536).unwrap();
        assert_atomic(e, Error::Invalid, |e| e.register(1));
        assert_atomic(e, Error::Invalid, |e| e.register(0));
        assert_atomic(e, Error::Invalid, |e| e.register(65536));
        assert_eq!(e.stats().processes, 16);
    });
}

#[test]
fn ks12_process_local_numeric_collision_and_stale_handles() {
    run(|e| {
        setup(e, 2);
        let a = e.create_endpoint(1).unwrap();
        let b = e.create_endpoint(2).unwrap();
        assert_eq!(a, b);
        // The equal numeric handle resolves only to A's own endpoint.
        e.send(1, b, &Message::new()).unwrap();
        assert_eq!(e.receive(1, a).unwrap().sender_pid, 1);
        assert_atomic(e, Error::Again, |e| e.receive(2, b));
        let other = e.derive(2, b, SEND).unwrap();
        assert_atomic(e, Error::BadHandle, |e| e.check_endpoint(1, other, SEND));
        let old = e.derive(1, a, SEND).unwrap();
        e.close(1, old).unwrap();
        let fresh = e.derive(1, a, SEND).unwrap();
        assert_ne!(old, fresh);
        assert_atomic(e, Error::BadHandle, |e| e.send(1, old, &Message::new()));
        for h in [
            0,
            1,
            u64::MAX,
            (1u64 << 32) | 256,
            (1u64 << 32) | u64::from(u32::MAX),
        ] {
            assert_atomic(e, Error::BadHandle, |e| e.close(1, h));
        }
    });
}

#[test]
fn ks13_attenuation_transfer_and_zero_rights() {
    run(|e| {
        setup(e, 3);
        let a = e.create_endpoint(1).unwrap();
        let (b, dest) = destination(e);
        assert_atomic(e, Error::Invalid, |e| e.derive(1, a, 32));
        let send = e.derive(1, a, SEND).unwrap();
        let zero = e.derive(1, a, 0).unwrap();
        assert_atomic(e, Error::Access, |e| e.derive(1, send, SEND));
        // Existing capability authority is checked before the requested fields.
        assert_atomic(e, Error::Access, |e| e.derive(1, send, 32));
        assert_atomic(e, Error::Access, |e| e.bootstrap_delegate(1, send, 3, 32));
        assert_atomic(e, Error::Access, |e| e.send(1, zero, &Message::new()));
        let derive = e.derive(1, a, DERIVE).unwrap();
        assert_atomic(e, Error::Access, |e| e.derive(1, derive, SEND));
        assert_atomic(e, Error::Access, |e| {
            e.send(1, dest, &cap_message(send, SEND))
        });
        assert_atomic(e, Error::Access, |e| {
            e.send(1, dest, &cap_message(send, 32))
        });
        assert_atomic(e, Error::Access, |e| e.send(1, dest, &cap_message(a, RECV)));
        assert_atomic(e, Error::Access, |e| e.bootstrap_delegate(1, a, 3, RECV));
        assert_atomic(e, Error::Invalid, |e| e.send(1, dest, &cap_message(a, 32)));
        e.send(1, dest, &cap_message(a, 0)).unwrap();
        let m = e.receive(2, b).unwrap();
        assert_eq!(m.caps[0].rights, 0);
        assert_atomic(e, Error::Access, |e| {
            e.check_endpoint(2, m.caps[0].handle, SEND)
        });
        assert!(e.check_endpoint(1, a, RECV).is_ok());
    });
}

#[test]
fn ks10_message_rejection_and_endpoint_priority_are_atomic() {
    run(|e| {
        setup(e, 2);
        let (_root, dest) = destination(e);
        let mut invalid = Message::new();
        invalid.len = 257;
        assert_atomic(e, Error::BadHandle, |e| e.send(1, 0, &invalid));
        assert_atomic(e, Error::MessageSize, |e| e.send(1, dest, &invalid));
        for field in 0..5 {
            let mut m = Message::new();
            match field {
                0 => m.sender_pid = 1,
                1 => m.reserved = 1,
                2 => m.caps[0].handle = 1,
                3 => m.caps[1].rights = 1,
                _ => m.data[255] = 1,
            };
            assert_atomic(e, Error::Invalid, |e| e.send(1, dest, &m));
        }
    });
}

#[test]
fn ks11_snapshot_fifo_and_queue_quota() {
    run(|e| {
        setup(e, 2);
        let (root, dest) = destination(e);
        for tag in 0..16 {
            let mut m = Message::new();
            m.tag = tag;
            m.len = 1;
            m.data[0] = tag as u8;
            e.send(1, dest, &m).unwrap();
            m.tag = 999;
            m.data[0] = 255;
            assert_eq!(m.tag, 999);
        }
        assert_atomic(e, Error::Again, |e| e.send(1, dest, &Message::new()));
        for tag in 0..16 {
            let m = e.receive(2, root).unwrap();
            assert_eq!(m.sender_pid, 1);
            assert_eq!(m.tag, tag);
            assert_eq!(m.data[0], tag as u8);
            assert!(m.data[1..].iter().all(|x| *x == 0));
            assert_eq!(m.caps, [Cap::EMPTY; 2]);
        }
        assert_atomic(e, Error::Again, |e| e.receive(2, root));
        // Exercise ring wrap rather than only head=0.
        for tag in 16..40 {
            let mut m = Message::new();
            m.tag = tag;
            e.send(1, dest, &m).unwrap();
            assert_eq!(e.receive(2, root).unwrap().tag, tag);
        }
    });
}

#[test]
fn ks15_receive_reserves_both_slots_before_commit() {
    run(|e| {
        setup(e, 2);
        let a = e.create_endpoint(1).unwrap();
        let (b, dest) = destination(e);
        let mut m = cap_message(a, SEND);
        m.cap_count = 2;
        m.caps[1] = Cap {
            handle: a,
            rights: 0,
        };
        e.send(1, dest, &m).unwrap();
        let mut handles = Vec::new();
        while let Ok(h) = e.derive(2, b, 0) {
            handles.push(h);
        }
        assert_eq!(handles.len(), 254);
        assert_atomic(e, Error::Quota, |e| e.receive(2, b));
        e.close(2, handles.pop().unwrap()).unwrap();
        assert_atomic(e, Error::Quota, |e| e.receive(2, b));
        e.close(2, handles.pop().unwrap()).unwrap();
        let received = e.receive(2, b).unwrap();
        assert_ne!(received.caps[0].handle, received.caps[1].handle);
        assert_eq!(received.caps[0].rights, SEND);
        assert_eq!(received.caps[1].rights, 0);
        e.check_endpoint(2, received.caps[0].handle, SEND).unwrap();
        e.check_endpoint(1, a, SEND).unwrap();
    });
}

#[test]
fn ks15_two_cap_send_global_node_exhaustion_has_no_partial_node() {
    run(|e| {
        setup(e, 5);
        let mut roots = [0u64; 5];
        for pid in 1..=5 {
            roots[(pid - 1) as usize] = e.create_endpoint(pid).unwrap();
        }
        let dest = e.bootstrap_delegate(2, roots[1], 1, SEND).unwrap();
        for pid in 1..=5 {
            while e.nodes.iter().filter(|n| n.live).count() < 1023 {
                if e.derive(pid, roots[(pid - 1) as usize], 0).is_err() {
                    break;
                }
            }
        }
        assert_eq!(e.nodes.iter().filter(|n| n.live).count(), 1023);
        let mut m = cap_message(roots[0], 0);
        m.cap_count = 2;
        m.caps[1] = m.caps[0];
        assert_atomic(e, Error::Quota, |e| e.send(1, dest, &m));
        e.send(1, dest, &cap_message(roots[0], 0)).unwrap();
        assert_eq!(e.nodes.iter().filter(|n| n.live).count(), 1024);
        assert_atomic(e, Error::Quota, |e| e.derive(5, roots[4], 0));
    });
}

#[test]
fn ks15_depth_eight_parent_retention_and_reclamation() {
    run(|e| {
        setup(e, 2);
        let root = e.create_endpoint(1).unwrap();
        let mut h = root;
        let mut chain = Vec::new();
        for _ in 0..8 {
            h = e.derive(1, h, ALL_RIGHTS).unwrap();
            chain.push(h);
        }
        assert_atomic(e, Error::Quota, |e| e.derive(1, h, 0));
        assert_atomic(e, Error::Quota, |e| e.bootstrap_delegate(1, h, 2, SEND));
        e.close(1, root).unwrap();
        for parent in &chain[..7] {
            e.close(1, *parent).unwrap();
        }
        assert_eq!(e.stats().nodes, 9);
        e.check_endpoint(1, h, RECV).unwrap();
        e.close(1, h).unwrap();
        assert_eq!(e.stats().nodes, 0);
        assert_eq!(e.stats().endpoints, 0);
    });
}

#[test]
fn ks13_all_transfer_authority_precedes_any_depth_resource_failure() {
    run(|e| {
        setup(e, 2);
        let a = e.create_endpoint(1).unwrap();
        let (_b, dest) = destination(e);
        let mut deep = a;
        for _ in 0..8 {
            deep = e.derive(1, deep, ALL_RIGHTS).unwrap();
        }
        let shallow = e.derive(1, a, SEND).unwrap();
        let mut m = cap_message(deep, 0);
        m.cap_count = 2;
        m.caps[1] = Cap {
            handle: 0,
            rights: 0,
        };
        assert_atomic(e, Error::BadHandle, |e| e.send(1, dest, &m));
        m.caps[1] = Cap {
            handle: shallow,
            rights: 0,
        };
        assert_atomic(e, Error::Access, |e| e.send(1, dest, &m));
        m.caps[1] = Cap {
            handle: a,
            rights: 0,
        };
        assert_atomic(e, Error::Quota, |e| e.send(1, dest, &m));
    });
}

#[test]
fn ks14_revoke_is_not_array_order_dependent_and_preserves_own_node() {
    run(|e| {
        setup(e, 3);
        let spare = e.create_endpoint(3).unwrap();
        let root = e.create_endpoint(1).unwrap();
        let child = e
            .bootstrap_delegate(1, root, 2, SEND | TRANSFER | REVOKE)
            .unwrap();
        e.close(3, spare).unwrap();
        let grandchild = e.bootstrap_delegate(2, child, 3, SEND).unwrap();
        let g = e.resolve(3, grandchild).unwrap();
        let c = e.resolve(2, child).unwrap();
        assert!(g < c, "fixture must put child before parent in array");
        e.revoke(1, root).unwrap();
        e.check_endpoint(1, root, SEND).unwrap();
        assert_atomic(e, Error::Revoked, |e| e.check_endpoint(2, child, SEND));
        assert_atomic(e, Error::Revoked, |e| e.check_endpoint(3, grandchild, SEND));
        e.close(2, child).unwrap();
        e.close(3, grandchild).unwrap();
        assert_eq!(e.stats().nodes, 1);
    });
}

#[test]
fn ks14_revoke_cancels_whole_queued_transfer_but_preserves_fifo() {
    run(|e| {
        setup(e, 3);
        let root = e.create_endpoint(1).unwrap();
        let foreign = e.create_endpoint(3).unwrap();
        let cap = e
            .bootstrap_delegate(3, foreign, 1, SEND | TRANSFER)
            .unwrap();
        let (b, dest) = destination(e);
        let mut doomed = cap_message(root, SEND);
        doomed.tag = 1;
        doomed.cap_count = 2;
        doomed.caps[1] = Cap {
            handle: cap,
            rights: SEND,
        };
        e.send(1, dest, &doomed).unwrap();
        let mut plain = Message::new();
        plain.tag = 2;
        e.send(1, dest, &plain).unwrap();
        e.revoke(1, root).unwrap();
        assert_eq!(e.stats().queued_messages, 1);
        let good = e.receive(2, b).unwrap();
        assert_eq!(good.tag, 2);
        assert_eq!(good.cap_count, 0);
        assert_atomic(e, Error::Again, |e| e.receive(2, b));
        e.check_endpoint(1, cap, SEND).unwrap();
    });
}

#[test]
fn ks14_revocation_after_delivery_and_close_do_not_recall_bytes() {
    run(|e| {
        setup(e, 2);
        let a = e.create_endpoint(1).unwrap();
        let (b, dest) = destination(e);
        e.send(1, dest, &cap_message(a, SEND)).unwrap();
        let delivered = e.receive(2, b).unwrap();
        e.revoke(1, a).unwrap();
        assert_eq!(delivered.data[0], 42);
        assert_atomic(e, Error::Revoked, |e| {
            e.check_endpoint(2, delivered.caps[0].handle, SEND)
        });
        e.close(2, delivered.caps[0].handle).unwrap();
        e.check_endpoint(1, a, RECV).unwrap();
    });
}

#[test]
fn ks16_last_receive_close_tombstone_and_revoked_before_closed() {
    run(|e| {
        setup(e, 2);
        let root = e.create_endpoint(1).unwrap();
        let recv = e.derive(1, root, RECV).unwrap();
        let send = e.bootstrap_delegate(1, root, 2, SEND).unwrap();
        e.close(1, root).unwrap();
        e.send(2, send, &Message::new()).unwrap();
        e.close(1, recv).unwrap();
        assert_eq!(e.stats().queued_messages, 0);
        assert_atomic(e, Error::Closed, |e| e.send(2, send, &Message::new()));
        assert_eq!(e.stats().endpoints, 1);
        e.close(2, send).unwrap();
        assert_eq!(e.stats().nodes, 0);
        assert_eq!(e.stats().endpoints, 0);
        let root = e.create_endpoint(1).unwrap();
        let send = e.bootstrap_delegate(1, root, 2, SEND).unwrap();
        e.revoke(1, root).unwrap();
        e.close(1, root).unwrap();
        assert_atomic(e, Error::Revoked, |e| e.check_endpoint(2, send, SEND));
        e.close(2, send).unwrap();
    });
}

#[test]
fn ks16_exit_revokes_received_and_issued_descendants_and_queued_caps() {
    run(|e| {
        setup(e, 3);
        let root = e.create_endpoint(1).unwrap();
        let b = e.bootstrap_delegate(1, root, 2, SEND | TRANSFER).unwrap();
        let c = e.bootstrap_delegate(2, b, 3, SEND).unwrap();
        let inbox = e.create_endpoint(3).unwrap();
        let to_c = e.bootstrap_delegate(3, inbox, 2, SEND).unwrap();
        e.send(2, to_c, &cap_message(b, SEND)).unwrap();
        assert_eq!(e.stats().queued_messages, 1);
        e.exit(2).unwrap();
        assert_eq!(e.stats().queued_messages, 0);
        assert_atomic(e, Error::Revoked, |e| e.check_endpoint(3, c, SEND));
        e.check_endpoint(1, root, RECV).unwrap();
        e.close(3, c).unwrap();
        e.exit(1).unwrap();
        e.exit(3).unwrap();
        assert_eq!(e.stats().nodes, 0);
        assert_eq!(e.stats().endpoints, 0);
        assert_eq!(e.stats().handles, 0);
    });
}

#[test]
fn ks15_slot_node_object_max_generation_retire_without_aba() {
    run(|e| {
        setup(e, 1);
        e.processes[0].slots[1].generation = u32::MAX;
        e.nodes[0].generation = u32::MAX;
        e.endpoints[0].generation = u32::MAX;
        let old = e.create_endpoint(1).unwrap();
        e.close(1, old).unwrap();
        assert!(e.processes[0].slots[1].retired && e.nodes[0].retired && e.endpoints[0].retired);
        let new = e.create_endpoint(1).unwrap();
        assert_ne!(old, new);
        assert_atomic(e, Error::BadHandle, |e| e.check_endpoint(1, old, SEND));
        assert_eq!(e.stats().retired_slots, 1);
        assert_eq!(e.stats().retired_nodes, 1);
        assert_eq!(e.stats().retired_endpoints, 1);
    });
}

#[test]
fn endpoint_owner_and_global_quotas_recover_after_close() {
    run(|e| {
        setup(e, 16);
        let mut all = Vec::new();
        for pid in 1..=16 {
            for _ in 0..4 {
                all.push((pid, e.create_endpoint(pid).unwrap()));
            }
        }
        assert_eq!(e.stats().endpoints, 64);
        assert_atomic(e, Error::Quota, |e| e.create_endpoint(1));
        for (pid, h) in all {
            e.close(pid, h).unwrap();
        }
        assert_eq!(e.stats().endpoints, 0);
        e.create_endpoint(1).unwrap();
    });
}

#[test]
fn closed_object_tombstones_hold_global_capacity_without_retargeting() {
    run(|e| {
        setup(e, 16);
        let mut foreign = Vec::new();
        for pid in 1..=16 {
            for _ in 0..4 {
                let root = e.create_endpoint(pid).unwrap();
                foreign.push((pid, e.bootstrap_delegate(pid, root, 16, SEND).unwrap()));
            }
        }
        e.exit(1).unwrap();
        e.register(17).unwrap();
        // New owner has no objects of its own: this is the global object bound,
        // not the independent four-object owner limit.
        assert_atomic(e, Error::Quota, |e| e.create_endpoint(17));
        let old = foreign[0].1;
        assert_atomic(e, Error::Revoked, |e| e.check_endpoint(16, old, SEND));
        e.close(16, old).unwrap();
        let fresh = e.create_endpoint(17).unwrap();
        e.check_endpoint(17, fresh, RECV).unwrap();
        assert_atomic(e, Error::BadHandle, |e| e.check_endpoint(16, old, SEND));
    });
}

#[test]
fn exact_public_error_numbers_match_the_syscall_contract() {
    for (i, error) in [
        Error::Invalid,
        Error::NoSys,
        Error::Fault,
        Error::BadHandle,
        Error::Access,
        Error::Type,
        Error::Quota,
        Error::Again,
        Error::Closed,
        Error::Revoked,
        Error::MessageSize,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(error.code(), -(i as i64) - 1);
    }
}

fn block_scope() -> BlockScope {
    BlockScope {
        device: 0,
        first_sector: 4,
        sector_count: 4,
    }
}
#[test]
fn m5_block_type_scope_and_admin_close_are_enforced() {
    run(|e| {
        setup(e, 1);
        let ep = e.create_endpoint(1).unwrap();
        let root = e.create_block_device(1, block_scope()).unwrap();
        assert_atomic(e, Error::Type, |e| e.check_block(1, ep, BLOCK_READ));
        assert_atomic(e, Error::Type, |e| e.check_block(1, ep, RECV));
        assert_atomic(e, Error::Type, |e| e.check_endpoint(1, root, SEND));
        assert_atomic(e, Error::Type, |e| e.check_endpoint(1, root, 32));
        let read = e.derive(1, root, BLOCK_READ).unwrap();
        let auth = e.check_block(1, read, BLOCK_READ).unwrap();
        assert_eq!(auth.scope(), block_scope());
        assert_eq!(auth.sector(0), Ok(4));
        assert_eq!(auth.sector(2), Ok(6));
        assert_eq!(auth.sector(4), Err(Error::Invalid));
        assert_atomic(e, Error::Invalid, |e| e.derive(1, root, RECV));
        assert_atomic(e, Error::Access, |e| e.derive(1, read, BLOCK_READ));
        e.close(1, root).unwrap();
        assert_eq!(e.check_block(1, read, BLOCK_READ), Ok(auth));
        assert_atomic(e, Error::Access, |e| e.revoke(1, read));
        e.close(1, read).unwrap();
        assert_eq!(e.stats().block_objects, 0);
        assert_eq!(e.stats().endpoints, 1);
    });
}
#[test]
fn m5_block_transfer_revoke_and_owner_exit_use_common_lifetimes() {
    run(|e| {
        setup(e, 2);
        let root = e.create_block_device(1, block_scope()).unwrap();
        let (ep, dest) = destination(e);
        assert_atomic(e, Error::Invalid, |e| {
            e.bootstrap_delegate(1, root, 2, RECV)
        });
        assert_atomic(e, Error::Invalid, |e| {
            e.send(1, dest, &cap_message(root, RECV))
        });
        e.send(1, dest, &cap_message(root, BLOCK_READ)).unwrap();
        let received = e.receive(2, ep).unwrap();
        let h = received.caps[0].handle;
        assert_eq!(
            e.check_block(2, h, BLOCK_READ).unwrap().scope(),
            block_scope()
        );
        e.revoke(1, root).unwrap();
        assert_atomic(e, Error::Revoked, |e| e.check_block(2, h, BLOCK_READ));
        e.close(2, h).unwrap();
        let child = e.bootstrap_delegate(1, root, 2, BLOCK_READ).unwrap();
        e.exit(1).unwrap();
        assert_atomic(e, Error::Revoked, |e| e.check_block(2, child, BLOCK_READ));
        e.close(2, child).unwrap();
        e.exit(2).unwrap();
        assert_eq!(e.stats().nodes, 0);
        assert_eq!(e.stats().block_objects, 0);
    });
}

#[test]
fn m5_revoked_queued_block_cap_cancels_complete_mixed_type_message() {
    run(|e| {
        setup(e, 2);
        let block = e.create_block_device(1, block_scope()).unwrap();
        let endpoint = e.create_endpoint(1).unwrap();
        let (inbox, send) = destination(e);
        let mut m = cap_message(block, BLOCK_READ);
        m.cap_count = 2;
        m.caps[1] = Cap {
            handle: endpoint,
            rights: SEND,
        };
        e.send(1, send, &m).unwrap();
        e.revoke(1, block).unwrap();
        assert_atomic(e, Error::Again, |e| e.receive(2, inbox));
        e.check_block(1, block, BLOCK_READ).unwrap();
        e.check_endpoint(1, endpoint, SEND).unwrap();
        assert_eq!(e.stats().queued_messages, 0);
        assert_eq!(e.stats().block_objects, 1);
    });
}
#[test]
fn m5_block_quota_is_separate_from_all_64_endpoint_objects() {
    run(|e| {
        setup(e, 16);
        for pid in 1..=16 {
            for _ in 0..4 {
                e.create_endpoint(pid).unwrap();
            }
        }
        let mut roots = [0; 4];
        for h in &mut roots {
            *h = e.create_block_device(1, block_scope()).unwrap();
        }
        assert_eq!(e.stats().endpoints, 64);
        assert_eq!(e.stats().block_objects, 4);
        assert_atomic(e, Error::Quota, |e| e.create_block_device(2, block_scope()));
        e.close(1, roots[0]).unwrap();
        e.create_block_device(2, block_scope()).unwrap();
        assert_eq!(e.stats().endpoints, 64);
    });
}
#[test]
fn m5_block_scope_invalid_masks_error_order_and_generation_retirement() {
    run(|e| {
        setup(e, 1);
        for scope in [
            BlockScope {
                sector_count: 0,
                ..block_scope()
            },
            BlockScope {
                first_sector: u64::MAX,
                sector_count: 1,
                ..block_scope()
            },
        ] {
            assert_atomic(e, Error::Invalid, |e| e.create_block_device(1, scope));
        }
        e.blocks[0].generation = u32::MAX;
        let root = e.create_block_device(1, block_scope()).unwrap();
        let old = e.check_block(1, root, BLOCK_READ).unwrap();
        assert_atomic(e, Error::Invalid, |e| e.check_block(1, root, RECV));
        // M6 assigns bit32 to WRITE. It remains absent from the legacy root;
        // bit128 is still unknown and must retain the invalid-mask rejection.
        assert_atomic(e, Error::Access, |e| e.derive(1, root, BLOCK_WRITE));
        assert_atomic(e, Error::Invalid, |e| e.derive(1, root, 128));
        let read = e.derive(1, root, BLOCK_READ).unwrap();
        assert_atomic(e, Error::Access, |e| e.derive(1, read, RECV));
        e.close(1, root).unwrap();
        e.close(1, read).unwrap();
        assert!(e.blocks[0].retired);
        let new = e.create_block_device(1, block_scope()).unwrap();
        assert_ne!(
            e.check_block(1, new, BLOCK_READ).unwrap().object_key(),
            old.object_key()
        );
        assert_atomic(e, Error::BadHandle, |e| e.check_block(1, root, BLOCK_READ));
    });
}

#[test]
fn bounded_transaction_state_machine_fuzz_preserves_error_atomicity() {
    run(|e| {
        setup(e, 4);
        let mut handles = Vec::new();
        for pid in 1..=4 {
            handles.push((pid, e.create_endpoint(pid).unwrap()));
        }
        let mut word = 0x9385a1u64;
        for _ in 0..800 {
            word = word
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let (pid, h) = handles[(word as usize) % handles.len()];
            let target = (word >> 8) % 4 + 1;
            let rights = (word >> 16) & 63;
            let before = e.clone();
            let result = match (word >> 32) % 7 {
                0 => e.derive(pid, h, rights).map(|n| {
                    handles.push((pid, n));
                }),
                1 => e.bootstrap_delegate(pid, h, target, rights).map(|n| {
                    handles.push((target, n));
                }),
                2 => e.revoke(pid, h),
                3 => e.close(pid, h),
                4 => e.send(pid, h, &Message::new()),
                5 => e.receive(pid, h).map(|_| ()),
                _ => e.check_endpoint(target, h, SEND),
            };
            if result.is_err() {
                assert!(*e == before, "failed transaction mutated state");
            }
            assert!(e.check_invariants());
        }
    });
}

#[test]
fn ks10_bounded_wire_bit_fuzz_never_panics_or_mutates_on_reject() {
    run(|e| {
        setup(e, 2);
        let (_root, dest) = destination(e);
        for bit in 0..2560 {
            let mut bytes = Message::new().encode();
            bytes[bit / 8] ^= 1 << (bit % 8);
            if let Ok(m) = Message::decode(&bytes) {
                let before = e.clone();
                let result = e.send(1, dest, &m);
                if result.is_err() {
                    assert!(*e == before);
                } else {
                    let root = (1u64 << 32) | 1;
                    e.receive(2, root).unwrap();
                }
            }
        }
    });
}

// ---- K05: Launch and Process objects (ADR-042) ----

fn launch_pair(e: &mut Engine) -> (u64, u64) {
    // pid 1 is the spawner with one launch capability; pid 2 is its child.
    let launch = e.create_launch(1, 0).unwrap();
    let child = e.create_process(1, 2).unwrap();
    (launch, child)
}

#[test]
fn k05_rights_constants_and_known_mask() {
    assert_eq!(LAUNCH, 128);
    assert_eq!(OBSERVE, 256);
    assert_eq!(KILL, 512);
    assert_eq!(ENTROPY, 1024);
    assert_eq!(NETWORK, 2048);
    assert_eq!(DISPLAY, 4096);
    assert_eq!(INPUT, 8192);
    assert_eq!(KNOWN_RIGHTS, 16383);
    assert_eq!(NETWORK_RIGHTS & (DERIVE | TRANSFER | REVOKE), 0);
    assert_eq!(ENTROPY_RIGHTS & (DERIVE | TRANSFER | REVOKE), 0);
    assert_eq!(LAUNCH_RIGHTS & (DERIVE | TRANSFER | REVOKE), 0);
    assert_eq!(PROCESS_RIGHTS & (DERIVE | TRANSFER | REVOKE), 0);
}

#[test]
fn k05_launch_is_typed_non_derivable_non_transferable_and_closable() {
    run(|e| {
        setup(e, 2);
        let launch = e.create_launch(1, 3).unwrap();
        assert_eq!(e.check_launch(1, launch), Ok(3));
        assert_eq!(e.stats().launch_objects, 1);
        let endpoint = e.create_endpoint(1).unwrap();
        assert_atomic(e, Error::Type, |e| e.check_launch(1, endpoint));
        assert_atomic(e, Error::Type, |e| e.check_endpoint(1, launch, SEND));
        assert_atomic(e, Error::Type, |e| e.check_block(1, launch, BLOCK_READ));
        assert_atomic(e, Error::Type, |e| e.check_process(1, launch, OBSERVE));
        // No derive, revoke or transfer exists for this object type at all.
        assert_atomic(e, Error::Invalid, |e| e.derive(1, launch, LAUNCH));
        assert_atomic(e, Error::Invalid, |e| e.revoke(1, launch));
        assert_atomic(e, Error::Invalid, |e| {
            e.bootstrap_delegate(1, launch, 2, LAUNCH)
        });
        assert_atomic(e, Error::Invalid, |e| e.check_delegation(1, launch, LAUNCH));
        // Another process cannot use the numeric value (its table is empty).
        assert_atomic(e, Error::BadHandle, |e| e.check_launch(2, launch));
        let (_, send) = destination(e);
        assert_atomic(e, Error::Invalid, |e| {
            e.send(1, send, &cap_message(launch, LAUNCH))
        });
        // After pid 2 gained its own slot 1, the same number names only pid 2's
        // own endpoint, never pid 1's launch object (KS-12 collision rule).
        assert_atomic(e, Error::Type, |e| e.check_launch(2, launch));
        e.close(1, launch).unwrap();
        assert_eq!(e.stats().launch_objects, 0);
        assert_eq!(e.stats().retired_launch_objects, 0);
        assert_atomic(e, Error::BadHandle, |e| e.check_launch(1, launch));
    });
}

#[test]
fn k05_launch_quota_and_owner_exit_close_objects() {
    run(|e| {
        setup(e, 2);
        let mut handles = Vec::new();
        for image in 0..MAX_LAUNCH_OBJECTS as u32 {
            handles.push(e.create_launch(1, image).unwrap());
        }
        assert_atomic(e, Error::Quota, |e| e.create_launch(1, 0));
        assert_eq!(e.stats().launch_objects, MAX_LAUNCH_OBJECTS);
        e.exit(1).unwrap();
        assert_eq!(e.stats().launch_objects, 0);
        assert_eq!(e.stats().nodes, 0);
        for handle in handles {
            assert_atomic(e, Error::BadHandle, |e| e.check_launch(1, handle));
        }
    });
}

#[test]
fn k05_process_object_reports_lifecycle_to_its_spawner_only() {
    run(|e| {
        setup(e, 3);
        let (_, child) = launch_pair(e);
        let view = e.check_process(1, child, OBSERVE).unwrap();
        assert_eq!(
            view,
            ProcessView {
                child: 2,
                state: PROCESS_RUNNING,
                code: 0
            }
        );
        assert!(e.check_process(1, child, KILL).is_ok());
        assert_atomic(e, Error::BadHandle, |e| e.check_process(2, child, OBSERVE));
        assert_atomic(e, Error::BadHandle, |e| e.check_process(3, child, OBSERVE));
        assert_atomic(e, Error::Invalid, |e| e.check_process(1, child, SEND));
        assert_atomic(e, Error::Invalid, |e| e.derive(1, child, OBSERVE));
        assert_atomic(e, Error::Invalid, |e| e.revoke(1, child));
        assert_atomic(e, Error::Invalid, |e| {
            e.bootstrap_delegate(1, child, 3, OBSERVE)
        });
        // Child dies with an exit code: the object keeps the report.
        assert!(e.process_died(2, PROCESS_EXITED, 7));
        e.exit(2).unwrap();
        assert_eq!(
            e.check_process(1, child, OBSERVE).unwrap(),
            ProcessView {
                child: 2,
                state: PROCESS_EXITED,
                code: 7
            }
        );
        // A second death report for the same child changes nothing.
        assert!(!e.process_died(2, PROCESS_KILLED, 0));
        assert_eq!(e.check_process(1, child, OBSERVE).unwrap().code, 7);
        assert_eq!(e.stats().process_objects, 1);
        e.close(1, child).unwrap();
        assert_eq!(e.stats().process_objects, 0);
        // Only the launch handle of the spawner remains.
        assert_eq!(e.stats().nodes, 1);
    });
}

#[test]
fn k05_invalid_death_states_and_duplicate_children_are_rejected() {
    run(|e| {
        setup(e, 2);
        let (_, child) = launch_pair(e);
        assert!(!e.process_died(2, 0, 0));
        assert!(!e.process_died(2, PROCESS_RUNNING, 0));
        assert!(!e.process_died(2, 9, 0));
        assert!(!e.process_died(5, PROCESS_EXITED, 0));
        assert_eq!(
            e.check_process(1, child, OBSERVE).unwrap().state,
            PROCESS_RUNNING
        );
        assert_atomic(e, Error::Invalid, |e| e.create_process(1, 2));
        assert_atomic(e, Error::Invalid, |e| e.create_process(1, 1));
        assert_atomic(e, Error::BadHandle, |e| e.create_process(1, 9));
        assert_atomic(e, Error::BadHandle, |e| e.create_process(9, 2));
    });
}

#[test]
fn k05_running_child_without_handles_stays_an_orphan_candidate() {
    run(|e| {
        setup(e, 3);
        let (_, child) = launch_pair(e);
        let mut out = [0; MAX_PROCESS_OBJECTS];
        assert_eq!(e.orphans(&mut out), 0);
        // The spawner drops its handle while the child still runs: the link
        // survives so a later spawner exit can still end the child.
        e.close(1, child).unwrap();
        assert_eq!(e.stats().process_objects, 1);
        assert_eq!(e.stats().nodes, 1);
        assert_eq!(e.orphans(&mut out), 0);
        e.exit(1).unwrap();
        assert_eq!(e.orphans(&mut out), 1);
        assert_eq!(out[0], 2);
        // The kernel ends the orphan and reports the kill; then it retires.
        assert!(e.process_died(2, PROCESS_KILLED, 0));
        assert_eq!(e.orphans(&mut out), 0);
        assert_eq!(e.stats().process_objects, 0);
        e.exit(2).unwrap();
        assert_eq!(e.stats().processes, 1);
    });
}

#[test]
fn k05_spawner_exit_ends_grandchildren_iteratively() {
    run(|e| {
        setup(e, 4);
        let a = e.create_process(1, 2).unwrap();
        let b = e.create_process(2, 3).unwrap();
        let c = e.create_process(3, 4).unwrap();
        let mut out = [0; MAX_PROCESS_OBJECTS];
        assert_eq!(e.orphans(&mut out), 0);
        e.exit(1).unwrap();
        assert_eq!(e.orphans(&mut out), 1);
        assert_eq!(out[0], 2);
        assert!(e.process_died(2, PROCESS_KILLED, 0));
        e.exit(2).unwrap();
        assert_eq!(e.orphans(&mut out), 1);
        assert_eq!(out[0], 3);
        assert!(e.process_died(3, PROCESS_KILLED, 0));
        e.exit(3).unwrap();
        assert_eq!(e.orphans(&mut out), 1);
        assert_eq!(out[0], 4);
        assert!(e.process_died(4, PROCESS_KILLED, 0));
        e.exit(4).unwrap();
        assert_eq!(e.orphans(&mut out), 0);
        let s = e.stats();
        assert_eq!((s.processes, s.nodes, s.process_objects), (0, 0, 0));
        for handle in [a, b, c] {
            assert_atomic(e, Error::BadHandle, |e| e.check_process(1, handle, OBSERVE));
        }
    });
}

#[test]
fn k05_delegation_precheck_matches_bootstrap_rules() {
    run(|e| {
        setup(e, 2);
        let root = e.create_endpoint(1).unwrap();
        assert_eq!(e.check_delegation(1, root, SEND), Ok(()));
        assert_atomic(e, Error::Access, |e| e.check_delegation(1, root, RECV));
        assert_atomic(e, Error::Invalid, |e| e.check_delegation(1, root, 1024));
        let send_only = e.derive(1, root, SEND).unwrap();
        assert_atomic(e, Error::Access, |e| e.check_delegation(1, send_only, SEND));
        assert_atomic(e, Error::BadHandle, |e| e.check_delegation(1, 0, SEND));
        assert_atomic(e, Error::BadHandle, |e| e.check_delegation(2, root, SEND));
        let (_, child) = launch_pair(e);
        assert_atomic(e, Error::Invalid, |e| e.check_delegation(1, child, OBSERVE));
        let before = e.clone();
        assert_eq!(e.available_nodes(), MAX_NODES - 4);
        assert_eq!(e.available_slots(1), MAX_SLOTS - 1 - 4);
        assert_eq!(e.available_slots(9), 0);
        assert!(e.can_register());
        assert!(e.can_create_process());
        assert_eq!(*e, before);
    });
}

#[test]
fn k05_process_quota_and_capacity_probes() {
    run(|e| {
        setup(e, MAX_PROCESSES as u64);
        assert!(!e.can_register());
        for child in 2..=MAX_PROCESSES as u64 {
            e.create_process(1, child).unwrap();
        }
        assert_eq!(e.stats().process_objects, MAX_PROCESSES - 1);
        // Every object slot is bound to a live child, so a 17th cannot exist.
        assert_atomic(e, Error::Invalid, |e| e.create_process(1, 2));
        e.exit(1).unwrap();
        let mut out = [0; MAX_PROCESS_OBJECTS];
        assert_eq!(e.orphans(&mut out), MAX_PROCESSES - 1);
        assert!(e.can_register());
    });
}

#[test]
fn b01_entropy_capability_is_typed_exclusive_and_bounded() {
    run(|e| {
        setup(e, 2);
        let entropy = e.create_entropy(1).unwrap();
        assert_eq!(e.check_entropy(1, entropy), Ok(()));
        assert_eq!(e.stats().entropy_objects, 1);
        let endpoint = e.create_endpoint(1).unwrap();
        assert_atomic(e, Error::Type, |e| e.check_entropy(1, endpoint));
        assert_atomic(e, Error::Type, |e| e.check_endpoint(1, entropy, SEND));
        assert_atomic(e, Error::Type, |e| e.check_block(1, entropy, BLOCK_READ));
        assert_atomic(e, Error::Type, |e| e.check_launch(1, entropy));
        assert_atomic(e, Error::Type, |e| e.check_process(1, entropy, OBSERVE));
        assert_atomic(e, Error::Invalid, |e| e.derive(1, entropy, ENTROPY));
        assert_atomic(e, Error::Invalid, |e| e.revoke(1, entropy));
        assert_atomic(e, Error::Invalid, |e| {
            e.bootstrap_delegate(1, entropy, 2, ENTROPY)
        });
        assert_atomic(e, Error::BadHandle, |e| e.check_entropy(2, entropy));
        let (_, send) = destination(e);
        assert_atomic(e, Error::Invalid, |e| {
            e.send(1, send, &cap_message(entropy, ENTROPY))
        });
        for _ in 1..MAX_ENTROPY_OBJECTS {
            e.create_entropy(1).unwrap();
        }
        assert_atomic(e, Error::Quota, |e| e.create_entropy(1));
        e.close(1, entropy).unwrap();
        assert_eq!(e.stats().entropy_objects, MAX_ENTROPY_OBJECTS - 1);
        assert_atomic(e, Error::BadHandle, |e| e.check_entropy(1, entropy));
        e.exit(1).unwrap();
        assert_eq!(e.stats().entropy_objects, 0);
        assert_eq!(e.stats().nodes, 1);
    });
}

#[test]
fn raw_network_authority_is_typed_private_and_cannot_be_delegated() {
    run(|e| {
        setup(e, 2);
        let net = e.create_network(1).unwrap();
        assert!(e.check_network(1, net).is_ok());
        let entropy = e.create_entropy(1).unwrap();
        let endpoint = e.create_endpoint(1).unwrap();
        assert_atomic(e, Error::Type, |e| e.check_network(1, endpoint));
        assert_atomic(e, Error::Type, |e| e.check_network(1, entropy));
        assert_atomic(e, Error::Type, |e| e.check_entropy(1, net));
        assert_atomic(e, Error::Type, |e| e.check_endpoint(1, net, SEND));
        assert_atomic(e, Error::Type, |e| e.check_block(1, net, BLOCK_READ));
        assert_atomic(e, Error::Type, |e| e.check_launch(1, net));
        assert_atomic(e, Error::Type, |e| e.check_process(1, net, OBSERVE));
        assert_atomic(e, Error::BadHandle, |e| e.check_network(2, net));
        assert_atomic(e, Error::Invalid, |e| e.derive(1, net, NETWORK));
        assert_atomic(e, Error::Invalid, |e| e.revoke(1, net));
        assert_atomic(e, Error::Invalid, |e| {
            e.bootstrap_delegate(1, net, 2, NETWORK)
        });
        let (_, send) = destination(e);
        assert_atomic(e, Error::Invalid, |e| {
            e.send(1, send, &cap_message(net, NETWORK))
        });
        assert_eq!(e.stats().network_objects, 1);
    });
}

#[test]
fn raw_network_close_exit_and_reissue_do_not_resurrect_authority() {
    run(|e| {
        setup(e, 1);
        let old = e.create_network(1).unwrap();
        let old_object = e.check_network(1, old).unwrap();
        for _ in 1..MAX_NETWORK_OBJECTS {
            e.create_network(1).unwrap();
        }
        assert_atomic(e, Error::Quota, |e| e.create_network(1));
        e.close(1, old).unwrap();
        assert_atomic(e, Error::BadHandle, |e| e.check_network(1, old));
        let new = e.create_network(1).unwrap();
        assert_ne!(e.check_network(1, new).unwrap(), old_object);
        assert_ne!(new, old);
        assert_atomic(e, Error::BadHandle, |e| e.check_network(1, old));
        e.exit(1).unwrap();
        assert_eq!(e.stats().network_objects, 0);
        assert_eq!(e.stats().nodes, 0);
        assert_eq!(e.stats().handles, 0);
    });
}

#[test]
fn display_input_sources_are_sole_typed_manifest_authorities() {
    run(|e| {
        setup(e, 2);
        let display = e.create_display(1).unwrap();
        let input = e.create_input(1).unwrap();
        assert_ne!(
            e.check_display(1, display).unwrap(),
            e.check_input(1, input).unwrap()
        );
        assert_atomic(e, Error::Quota, |e| e.create_display(2));
        assert_atomic(e, Error::Quota, |e| e.create_input(2));
        assert_atomic(e, Error::BadHandle, |e| e.check_display(2, display));
        assert_atomic(e, Error::BadHandle, |e| e.check_input(2, input));
        assert_atomic(e, Error::Type, |e| e.check_display(1, input));
        assert_atomic(e, Error::Type, |e| e.check_input(1, display));
        for source in [display, input] {
            assert_atomic(e, Error::Type, |e| e.check_entropy(1, source));
            assert_atomic(e, Error::Type, |e| e.check_network(1, source));
            assert_atomic(e, Error::Type, |e| e.check_endpoint(1, source, SEND));
            assert_atomic(e, Error::Type, |e| e.check_block(1, source, BLOCK_READ));
            assert_atomic(e, Error::Type, |e| e.check_launch(1, source));
            assert_atomic(e, Error::Type, |e| e.check_process(1, source, OBSERVE));
        }
        let network = e.create_network(1).unwrap();
        assert_atomic(e, Error::Type, |e| e.check_display(1, network));
        assert_atomic(e, Error::Type, |e| e.check_input(1, network));
        assert!(e.check_network(1, network).is_ok());
        assert_eq!(e.stats().display_objects, 1);
        assert_eq!(e.stats().input_objects, 1);
    });
}

#[test]
fn display_input_sources_cannot_derive_transfer_revoke_or_escalate() {
    run(|e| {
        setup(e, 2);
        let display = e.create_display(1).unwrap();
        let input = e.create_input(1).unwrap();
        let (_, send) = destination(e);
        for (handle, right) in [(display, DISPLAY), (input, INPUT)] {
            for rights in [
                right,
                right | DERIVE,
                right | TRANSFER,
                right | REVOKE,
                SEND,
                ALL_RIGHTS,
            ] {
                assert_atomic(e, Error::Invalid, |e| e.derive(1, handle, rights));
                assert_atomic(e, Error::Invalid, |e| {
                    e.bootstrap_delegate(1, handle, 2, rights)
                });
                assert_atomic(e, Error::Invalid, |e| {
                    e.send(1, send, &cap_message(handle, rights))
                });
            }
            assert_atomic(e, Error::Invalid, |e| e.revoke(1, handle));
        }
    });
}

#[test]
fn display_input_close_reuse_exit_never_restore_stale_source() {
    run(|e| {
        setup(e, 2);
        let d = e.create_display(1).unwrap();
        let i = e.create_input(1).unwrap();
        let dk = e.check_display(1, d).unwrap();
        let ik = e.check_input(1, i).unwrap();
        e.close(1, d).unwrap();
        e.close(1, i).unwrap();
        assert_atomic(e, Error::BadHandle, |e| e.check_display(1, d));
        assert_atomic(e, Error::BadHandle, |e| e.check_input(1, i));
        assert_eq!(e.stats().nodes, 0);
        assert_eq!(e.stats().handles, 0);
        assert_eq!(e.stats().display_objects, 0);
        assert_eq!(e.stats().input_objects, 0);
        let d2 = e.create_display(2).unwrap();
        let i2 = e.create_input(2).unwrap();
        assert_ne!(dk, e.check_display(2, d2).unwrap());
        assert_ne!(ik, e.check_input(2, i2).unwrap());
        e.exit(2).unwrap();
        assert_eq!(e.stats().display_objects, 0);
        assert_eq!(e.stats().input_objects, 0);
        assert_eq!(e.stats().nodes, 0);
        assert_eq!(e.stats().handles, 0);
        assert!(e.create_display(1).is_ok());
        assert!(e.create_input(1).is_ok());
    });
}

#[test]
fn display_input_generation_exhaustion_retires_sole_object_without_wrap() {
    run(|e| {
        setup(e, 1);
        e.display[0].generation = u32::MAX;
        e.input[0].generation = u32::MAX;
        let display = e.create_display(1).unwrap();
        let input = e.create_input(1).unwrap();
        e.close(1, display).unwrap();
        e.close(1, input).unwrap();
        assert!(e.display[0].retired && e.input[0].retired);
        assert_atomic(e, Error::Quota, |e| e.create_display(1));
        assert_atomic(e, Error::Quota, |e| e.create_input(1));
        assert_atomic(e, Error::BadHandle, |e| e.check_display(1, display));
        assert_atomic(e, Error::BadHandle, |e| e.check_input(1, input));
        assert_eq!(e.stats().retired_display_objects, 1);
        assert_eq!(e.stats().retired_input_objects, 1);
    });
}

#[test]
fn display_input_creation_checks_slot_node_and_pid_quota_atomically() {
    run(|e| {
        setup(e, 1);
        assert_atomic(e, Error::BadHandle, |e| e.create_display(2));
        assert_atomic(e, Error::BadHandle, |e| e.create_input(2));
        for slot in &mut e.processes[0].slots[1..] {
            slot.retired = true;
            slot.generation = u32::MAX;
        }
        assert_atomic(e, Error::Quota, |e| e.create_display(1));
        assert_atomic(e, Error::Quota, |e| e.create_input(1));
    });
    run(|e| {
        setup(e, 1);
        for node in &mut e.nodes {
            node.retired = true;
            node.generation = u32::MAX;
        }
        assert_atomic(e, Error::Quota, |e| e.create_display(1));
        assert_atomic(e, Error::Quota, |e| e.create_input(1));
    });
}

#[test]
fn manifest_capacity_queries_count_only_reusable_objects_without_mutation() {
    run(|e| {
        setup(e, 2);
        e.endpoints[MAX_ENDPOINTS - 1].generation = u32::MAX;
        e.endpoints[MAX_ENDPOINTS - 1].retired = true;
        e.children[MAX_PROCESS_OBJECTS - 1].generation = u32::MAX;
        e.children[MAX_PROCESS_OBJECTS - 1].retired = true;
        let root = e.create_endpoint(1).unwrap();
        let sender = e.bootstrap_delegate(1, root, 2, SEND).unwrap();
        let process = e.create_process(1, 2).unwrap();
        e.close(1, root).unwrap();
        let before = e.clone();
        assert_eq!(e.available_processes(), MAX_PROCESSES - 2);
        assert_eq!(e.available_process_objects(), MAX_PROCESS_OBJECTS - 2);
        assert_eq!(e.available_endpoints(), MAX_ENDPOINTS - 2);
        assert_eq!(e.available_nodes(), MAX_NODES - e.stats().nodes);
        assert_eq!(*e, before);
        e.close(2, sender).unwrap();
        assert_eq!(e.available_endpoints(), MAX_ENDPOINTS - 1);
        e.process_died(2, PROCESS_EXITED, 0);
        e.exit(2).unwrap();
        e.close(1, process).unwrap();
        assert_eq!(e.available_processes(), MAX_PROCESSES - 1);
        assert_eq!(e.available_process_objects(), MAX_PROCESS_OBJECTS - 1);
    });
}

#[test]
fn endpoint_replacement_preflight_keeps_remote_leaves_and_ignores_stale_handles() {
    run(|e| {
        setup(e, 2);
        let root = e.create_endpoint(1).unwrap();
        let receiver = e.derive(1, root, RECV).unwrap();
        let sender = e.bootstrap_delegate(1, root, 2, SEND).unwrap();
        e.close(1, root).unwrap();
        let unrelated: Vec<_> = (1..MAX_OWNED_ENDPOINTS)
            .map(|_| e.create_endpoint(1).unwrap())
            .collect();
        let before = e.clone();
        assert!(!e.can_create_owned_endpoints_after_closing(1, &[receiver], 1, &[]));
        assert!(e.can_create_owned_endpoints_after_closing(1, &[receiver], 1, &[2]));
        assert!(!e.can_create_owned_endpoints_after_closing(1, &[receiver], 2, &[2]));
        assert!(!e.can_create_owned_endpoints_after_closing(1, &[receiver], 1, &[1, 2]));
        assert!(!e.can_create_owned_endpoints_after_closing(99, &[], 0, &[]));
        assert_eq!(*e, before);

        // The planned teardown really frees exactly one object. Reusing the
        // local slot must not let its old handle excuse the replacement object.
        e.close(1, receiver).unwrap();
        e.close(2, sender).unwrap();
        let replacement = e.create_endpoint(1).unwrap();
        assert_ne!(replacement, receiver);
        let before = e.clone();
        assert!(!e.can_create_owned_endpoints_after_closing(1, &[receiver], 1, &[2]));
        assert!(e.can_create_owned_endpoints_after_closing(1, &[replacement], 1, &[2]));
        assert!(e.can_create_owned_endpoints_after_closing(1, &[unrelated[0]], 1, &[]));
        assert_eq!(*e, before);
    });
}

#[test]
fn endpoint_replacement_preflight_conservatively_retains_queued_capabilities() {
    run(|e| {
        setup(e, 2);
        let root = e.create_endpoint(1).unwrap();
        let (_destination, sender) = destination(e);
        e.send(1, sender, &cap_message(root, SEND)).unwrap();
        for _ in 1..MAX_OWNED_ENDPOINTS {
            e.create_endpoint(1).unwrap();
        }
        let before = e.clone();
        // Exit(2) will eventually flush the queue. Preflight deliberately does
        // not anticipate cancellation or weaken the closed-tombstone rule.
        assert!(!e.can_create_owned_endpoints_after_closing(1, &[root], 1, &[2]));
        assert_eq!(*e, before);
    });
}

#[test]
fn source_recreation_preflight_checks_owner_and_terminal_generation() {
    run(|e| {
        setup(e, 2);
        assert!(e.can_create_display_after_exit(99));
        assert!(e.can_create_input_after_exit(99));
        let display = e.create_display(1).unwrap();
        let input = e.create_input(2).unwrap();
        let before = e.clone();
        assert!(e.can_create_display_after_exit(1));
        assert!(!e.can_create_display_after_exit(2));
        assert!(e.can_create_input_after_exit(2));
        assert!(!e.can_create_input_after_exit(1));
        assert_eq!(*e, before);
        e.exit(2).unwrap();
        assert!(e.can_create_input_after_exit(99));
        e.close(1, display).unwrap();
        assert!(e.can_create_display_after_exit(99));
        assert_atomic(e, Error::BadHandle, |e| e.check_input(2, input));
    });
    run(|e| {
        setup(e, 1);
        e.display[0].generation = u32::MAX;
        e.input[0].generation = u32::MAX;
        e.create_display(1).unwrap();
        e.create_input(1).unwrap();
        let before = e.clone();
        assert!(!e.can_create_display_after_exit(1));
        assert!(!e.can_create_input_after_exit(1));
        assert_eq!(*e, before);
        e.exit(1).unwrap();
        assert!(!e.can_create_display_after_exit(1));
        assert!(!e.can_create_input_after_exit(1));
        e.register(2).unwrap();
        assert_atomic(e, Error::Quota, |e| e.create_display(2));
    });
}
