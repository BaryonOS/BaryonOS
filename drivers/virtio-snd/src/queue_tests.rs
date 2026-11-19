use super::*;
use crate::pcm::{EVENT_BYTES, TX_STATUS_BYTES};

const RAM_END: u64 = 256 * 1024 * 1024;

fn pool() -> Pool {
    Pool::new(
        [0x10_000, 0x11_000, 0x12_000, 0x13_000, 0x14_000, 0x15_000],
        RAM_END,
    )
    .unwrap()
}

/// Device-writable byte ranges inside the response page (§4 of the contract):
/// control response, the event buffers and the TX status words only.
fn writable_ranges(pool: &Pool) -> [(u64, u64); 3] {
    let base = pool.response();
    [
        (base, base + MAX_RESPONSE_BYTES as u64),
        (
            base + 0x800,
            base + 0x800 + u64::from(EVENT_BUFFERS) * EVENT_BYTES as u64,
        ),
        (
            base + 0xc00,
            base + 0xc00 + u64::from(TX_SLOTS) * TX_STATUS_BYTES as u64,
        ),
    ]
}

fn assert_descriptor_contained(pool: &Pool, descriptor: Descriptor) {
    let start = descriptor.address;
    let end = start + u64::from(descriptor.length);
    assert!(descriptor.length > 0);
    if descriptor.flags & F_WRITE != 0 {
        assert!(
            writable_ranges(pool)
                .iter()
                .any(|(low, high)| *low <= start && end <= *high),
            "device-writable descriptor outside the writable areas: {descriptor:?}"
        );
    } else {
        let readable = [pool.request(), pool.pages()[4], pool.pages()[5]];
        assert!(
            readable
                .iter()
                .any(|page| *page <= start && end <= *page + PAGE),
            "device-readable descriptor outside request/PCM pages: {descriptor:?}"
        );
    }
    for page in [pool.pages()[0], pool.used_page()] {
        assert!(
            end <= page || start >= page + PAGE,
            "descriptor reaches a ring page"
        );
    }
}

#[test]
fn pool_pages_are_distinct_aligned_nonzero_ram_pages() {
    let good = [0x1000, 0x2000, 0x3000, 0x4000, 0x5000, 0x6000];
    assert!(Pool::new(good, RAM_END).is_ok());
    for index in 0..Pool::PAGES {
        for bad in [0, 0x1001, RAM_END, !0xfff] {
            let mut pages = good;
            pages[index] = bad;
            assert_eq!(
                Pool::new(pages, RAM_END),
                Err(Error::Range),
                "{index} {bad:#x}"
            );
        }
        let mut duplicate = good;
        duplicate[index] = good[(index + 1) % Pool::PAGES];
        assert_eq!(Pool::new(duplicate, RAM_END), Err(Error::Range));
    }
    assert_eq!(
        pool().pages(),
        [0x10_000, 0x11_000, 0x12_000, 0x13_000, 0x14_000, 0x15_000]
    );
    assert_eq!(pool().device_writable_pages(), [0x11_000, 0x13_000]);
}

#[test]
fn ring_layout_is_aligned_and_rx_has_no_ring() {
    let pool = pool();
    for queue in ACTIVE {
        assert_eq!(pool.descriptors(queue).unwrap() % 16, 0);
        assert_eq!(pool.available(queue).unwrap() % 2, 0);
        assert_eq!(pool.used(queue).unwrap() % 4, 0);
        assert_eq!(
            pool.used(queue).unwrap() + 2,
            pool.used_page() + Pool::used_index_offset(queue).unwrap()
        );
    }
    for queue in [RXQ, QUEUES, u16::MAX] {
        assert_eq!(pool.descriptors(queue), Err(Error::Range));
        assert_eq!(pool.available(queue), Err(Error::Range));
        assert_eq!(pool.used(queue), Err(Error::Range));
        assert_eq!(Pool::used_index_offset(queue), Err(Error::Range));
    }
}

#[test]
fn every_descriptor_stays_inside_its_pool_area() {
    let pool = pool();
    let control = control_chain(&pool, SET_PARAMS_BYTES, MAX_RESPONSE_BYTES).unwrap();
    assert_eq!(control.head, 0);
    assert_eq!(control.used_len, MAX_RESPONSE_BYTES as u32);
    assert_eq!(control.entries[0].1.flags, F_NEXT);
    assert_eq!(control.entries[0].1.next, 1);
    assert_eq!(control.entries[1].1.flags, F_WRITE);
    for (_, descriptor) in control.entries {
        assert_descriptor_contained(&pool, descriptor);
    }
    for buffer in 0..EVENT_BUFFERS {
        let event = event_chain(&pool, buffer).unwrap();
        assert_eq!((event.head, event.used_len), (buffer, 8));
        assert_eq!(event.entries[0].1.flags, F_WRITE);
        assert_descriptor_contained(&pool, event.entries[0].1);
    }
    for slot in 0..TX_SLOTS {
        for len in [4, FIXTURE_PERIOD_BYTES as usize, MAX_SLOT_PCM as usize] {
            let tx = tx_chain(&pool, slot, len).unwrap();
            assert_eq!(tx.head, slot * 2);
            assert_eq!(tx.used_len, 8);
            // PCM is device-readable only; the status is the only writable part.
            assert_eq!(tx.entries[0].1.flags, F_NEXT);
            assert_eq!(tx.entries[0].1.length as usize, 4 + len);
            assert_eq!(tx.entries[1].1.flags, F_WRITE);
            assert_eq!(tx.entries[1].1.length, 8);
            for (index, descriptor) in tx.entries {
                assert!(index < QUEUE_SIZE);
                assert_descriptor_contained(&pool, descriptor);
            }
        }
    }
}

#[test]
fn chain_arguments_outside_their_bounds_publish_nothing() {
    let pool = pool();
    for (request, response) in [
        (0, 4),
        (SET_PARAMS_BYTES + 1, 4),
        (8, 0),
        (8, MAX_RESPONSE_BYTES + 1),
    ] {
        assert_eq!(control_chain(&pool, request, response), Err(Error::Range));
    }
    assert_eq!(event_chain(&pool, EVENT_BUFFERS), Err(Error::Range));
    for (slot, len) in [
        (TX_SLOTS, 4),
        (0, 0),
        (0, 3),
        (0, MAX_SLOT_PCM as usize + 4),
        (0, usize::MAX),
    ] {
        assert_eq!(
            tx_chain(&pool, slot, len),
            Err(Error::Range),
            "{slot} {len}"
        );
    }
}

#[test]
fn descriptor_encoding_is_little_endian_virtq_desc() {
    let bytes = Descriptor {
        address: 0x1122_3344_5566_7788,
        length: 0x99aa_bbcc,
        flags: F_NEXT | F_WRITE,
        next: 0x0102,
    }
    .encode();
    assert_eq!(
        bytes,
        [0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0xcc, 0xbb, 0xaa, 0x99, 3, 0, 2, 1]
    );
}

#[test]
fn ring_accepts_only_published_heads_with_exact_lengths() {
    let mut ring = Ring::new();
    assert_eq!(ring.pending(0), Ok(0));
    assert_eq!(ring.publish(2, 8), Ok(Publication { entry: 0, index: 1 }));
    assert_eq!(ring.publish(4, 8), Ok(Publication { entry: 1, index: 2 }));
    assert_eq!(ring.publish(2, 8), Err(Error::Busy));
    assert_eq!(ring.publish(QUEUE_SIZE, 8), Err(Error::Range));
    assert_eq!(ring.publish(1, 0), Err(Error::Range));
    assert_eq!(ring.outstanding(), 2);
    assert_eq!(ring.pending(2), Ok(2));
    assert_eq!(ring.pending(3), Err(Error::UsedOverrun));
    assert_eq!(ring.pending(u16::MAX), Err(Error::UsedOverrun));
    assert_eq!(ring.complete(0, 8), Err(Error::UnknownId));
    assert_eq!(ring.complete(u32::MAX, 8), Err(Error::UnknownId));
    assert_eq!(
        ring.complete(u32::from(QUEUE_SIZE), 8),
        Err(Error::UnknownId)
    );
    assert_eq!(ring.complete(4, 7), Err(Error::Length));
    assert_eq!(ring.complete(4, 9), Err(Error::Length));
    assert_eq!(ring.used_entry(), 0);
    assert_eq!(ring.complete(4, 8), Ok(4));
    assert_eq!(
        ring.complete(4, 8),
        Err(Error::UnknownId),
        "double completion"
    );
    assert_eq!(ring.complete(2, 8), Ok(2));
    assert_eq!(
        (
            ring.outstanding(),
            ring.used_index(),
            ring.available_index()
        ),
        (0, 2, 2)
    );
    // Idle: any movement of the used index is a violation.
    assert_eq!(ring.pending(2), Ok(0));
    for moved in [3, 4, 1, u16::MAX] {
        assert_eq!(ring.pending(moved), Err(Error::UsedOverrun));
    }
    for error in [Error::UnknownId, Error::Length, Error::UsedOverrun] {
        assert!(error.requires_fence());
    }
    for error in [
        Error::Range,
        Error::Busy,
        Error::State,
        Error::Unstable,
        Error::Quarantined,
    ] {
        assert!(!error.requires_fence());
    }
}

#[test]
fn ring_indices_wrap_and_fill_at_most_queue_size() {
    let mut ring = Ring::new();
    for round in 0..20_000u32 {
        let head = (round % u32::from(QUEUE_SIZE)) as u16;
        let publication = ring.publish(head, 4).unwrap();
        assert_eq!(publication.entry, (round % u32::from(QUEUE_SIZE)) as u16);
        assert_eq!(publication.index, (round + 1) as u16);
        assert_eq!(ring.pending(publication.index), Ok(1));
        assert_eq!(ring.complete(u32::from(head), 4), Ok(head));
    }
    for head in 0..QUEUE_SIZE {
        ring.publish(head, 8).unwrap();
    }
    assert_eq!(ring.outstanding(), QUEUE_SIZE);
    let full = ring.used_index().wrapping_add(QUEUE_SIZE);
    assert_eq!(ring.pending(full), Ok(QUEUE_SIZE));
    assert_eq!(ring.pending(full.wrapping_add(1)), Err(Error::UsedOverrun));
}

#[test]
fn fence_acknowledges_only_status_zero_and_bus_master_off_before_deadline() {
    let mut life = Lifecycle::new();
    assert!(!life.may_publish());
    life.go_live().unwrap();
    assert!(life.may_publish());
    assert_eq!(life.go_live(), Err(Error::State));
    life.begin_fence(100, 50).unwrap();
    assert!(!life.may_publish(), "publication frozen while fencing");
    assert_eq!(life.poll_fence(110, 0x0f, true), FenceStep::Pending);
    assert_eq!(
        life.poll_fence(120, 0, true),
        FenceStep::Pending,
        "BME still on"
    );
    assert_eq!(
        life.poll_fence(130, 0x0f, false),
        FenceStep::Pending,
        "status not 0"
    );
    assert_eq!(life.poll_fence(149, 0, false), FenceStep::Acknowledged);
    assert_eq!(
        life.check_snapshot([1, 4, 9], 0, false),
        Err(Error::Quarantined),
        "no snapshot"
    );
    let mut life = Lifecycle::new();
    life.go_live().unwrap();
    life.begin_fence(100, 50).unwrap();
    assert_eq!(life.poll_fence(149, 0, false), FenceStep::Acknowledged);
    life.record_snapshot([1, 4, 9]).unwrap();
    life.check_snapshot([1, 4, 9], 0, false).unwrap();
    assert!(!life.may_publish());
    assert_eq!(life.finish_fence(), Ok(1));
    assert_eq!((life.phase(), life.epoch()), (Phase::Offline, 1));
    life.go_live().unwrap();
}

#[test]
fn fence_misses_and_post_fence_changes_quarantine() {
    // Deadline reached before both acknowledgements.
    let mut life = Lifecycle::new();
    life.begin_fence(10, 5).unwrap();
    assert_eq!(life.poll_fence(15, 0, false), FenceStep::Quarantined);
    // Zero budget can never acknowledge, even with a quiet device.
    let mut zero = Lifecycle::new();
    zero.go_live().unwrap();
    zero.begin_fence(7, 0).unwrap();
    assert_eq!(zero.poll_fence(7, 0, false), FenceStep::Quarantined);
    // Any used-index, status or BME change after the snapshot.
    for (used, status, bme) in [
        ([1, 4, 10], 0, false),
        ([1, 4, 9], 1, false),
        ([1, 4, 9], 0, true),
    ] {
        let mut life = Lifecycle::new();
        life.go_live().unwrap();
        life.begin_fence(0, 10).unwrap();
        assert_eq!(life.poll_fence(1, 0, false), FenceStep::Acknowledged);
        life.record_snapshot([1, 4, 9]).unwrap();
        assert_eq!(
            life.check_snapshot(used, status, bme),
            Err(Error::Quarantined)
        );
        assert_eq!(life.finish_fence(), Err(Error::Quarantined));
    }
    // Deadline overflow and epoch overflow.
    let mut overflow = Lifecycle::new();
    assert_eq!(overflow.begin_fence(u64::MAX, 1), Err(Error::Quarantined));
    let mut epoch = Lifecycle::with_epoch(u64::MAX);
    epoch.begin_fence(0, 10).unwrap();
    assert_eq!(epoch.poll_fence(1, 0, false), FenceStep::Acknowledged);
    epoch.record_snapshot([0; 3]).unwrap();
    epoch.check_snapshot([0; 3], 0, false).unwrap();
    assert_eq!(epoch.finish_fence(), Err(Error::Quarantined));
    // Quarantine is terminal for every transition.
    for mut life in [Lifecycle::new(), overflow, epoch] {
        life.quarantine();
        assert!(!life.may_publish());
        assert_eq!(life.go_live(), Err(Error::Quarantined));
        assert_eq!(life.begin_fence(0, 10), Err(Error::Quarantined));
        assert_eq!(life.poll_fence(0, 0, false), FenceStep::Quarantined);
        assert_eq!(life.record_snapshot([0; 3]), Err(Error::Quarantined));
        assert_eq!(life.finish_fence(), Err(Error::Quarantined));
        assert_eq!(life.phase(), Phase::Quarantined);
    }
    // An unarmed poll is not an acknowledgement.
    let mut unarmed = Lifecycle::new();
    assert_eq!(unarmed.poll_fence(0, 0, false), FenceStep::Quarantined);
}

#[test]
fn config_read_requires_a_stable_generation() {
    let bytes = [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0];
    assert_eq!(stable_config(|| (4, bytes, 4), 1), Ok(bytes));
    let mut calls = 0;
    let read = || {
        calls += 1;
        if calls < 3 {
            (calls as u8, [0; CONFIG_BYTES], calls as u8 + 1)
        } else {
            (9, bytes, 9)
        }
    };
    assert_eq!(stable_config(read, 3), Ok(bytes));
    assert_eq!(stable_config(|| (0, bytes, 1), 16), Err(Error::Unstable));
    assert_eq!(stable_config(|| (0, bytes, 0), 0), Err(Error::Unstable));
}

#[test]
fn fixture_patterns_are_disjoint_and_never_silent() {
    assert_eq!(Pattern::A.frame(0), [0, 0, 0, 0x80]);
    assert_eq!(Pattern::A.frame(0x1234), [0x34, 0x12, 0x34, 0x92]);
    assert_eq!(Pattern::B.frame(0), [0xff, 0xff, 0xff, 0xdf]);
    assert_eq!(FIXTURE_PERIOD_BYTES, 1920);
    assert_eq!(FIXTURE_TOTAL_PERIODS, 100);
    for n in 0..FIXTURE_FRAMES {
        let a = Pattern::A.frame(n);
        let b = Pattern::B.frame(n);
        let class =
            |f: [u8; 4]| u16::from_le_bytes([f[0], f[1]]) ^ u16::from_le_bytes([f[2], f[3]]);
        assert_eq!(class(a), 0x8000);
        assert_eq!(class(b), 0x2000);
        assert_ne!(a, [0; 4]);
        assert_ne!(b, [0; 4]);
    }
}

#[test]
fn only_pcm_info_accepts_the_header_only_used_length() {
    let pool = pool();
    let info = pcm_info_chain(&pool, 16, 36).unwrap();
    assert_eq!((info.used_len, info.alternate_used_len), (36, 4));
    assert_eq!(info.entries, control_chain(&pool, 16, 36).unwrap().entries);
    // A status-only answer has nothing to tolerate.
    assert_eq!(pcm_info_chain(&pool, 8, 4).unwrap().alternate_used_len, 0);
    for chain in [
        control_chain(&pool, 24, 4).unwrap().alternate_used_len,
        event_chain(&pool, 0).unwrap().alternate_used_len,
        tx_chain(&pool, 0, 1920).unwrap().alternate_used_len,
    ] {
        assert_eq!(chain, 0);
    }
    for (reported, expected) in [
        (36, Ok(0)),
        (4, Ok(0)),
        (0, Err(Error::Length)),
        (5, Err(Error::Length)),
        (35, Err(Error::Length)),
        (37, Err(Error::Length)),
        (u32::MAX, Err(Error::Length)),
    ] {
        let mut ring = Ring::new();
        ring.publish_chain(info.head, info.used_len, info.alternate_used_len)
            .unwrap();
        assert_eq!(ring.complete(0, reported), expected, "{reported}");
    }
    // Without an alternate, the header-only length is a violation.
    let mut strict = Ring::new();
    strict.publish(0, 36).unwrap();
    assert_eq!(strict.complete(0, 4), Err(Error::Length));
    assert_eq!(strict.publish_chain(1, 8, 8), Err(Error::Range));
}
