use super::*;
use crate::pcm::{self, EVENT_BYTES, MAX_PERIODS, S_OK, TX_STATUS_BYTES};
use crate::{negotiate_features, Config, Error as CodecError};

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
    let mut saw_wrap = false;
    // Both 16-bit indices cross 65535 -> 0 at least once.
    for round in 0..(65_536 + 4 * u32::from(QUEUE_SIZE)) {
        let head = (round % u32::from(QUEUE_SIZE)) as u16;
        let boundary = ring.used_index() == u16::MAX;
        if boundary {
            assert_eq!(ring.available_index(), u16::MAX);
            assert_eq!(ring.used_entry(), QUEUE_SIZE - 1);
        }
        let publication = ring.publish(head, 4).unwrap();
        assert_eq!(publication.entry, (round % u32::from(QUEUE_SIZE)) as u16);
        assert_eq!(publication.index, (round + 1) as u16);
        assert_eq!(ring.pending(publication.index), Ok(1));
        if boundary {
            saw_wrap = true;
            assert_eq!(
                publication,
                Publication {
                    entry: QUEUE_SIZE - 1,
                    index: 0
                }
            );
            assert_eq!(ring.pending(0), Ok(1));
            assert_eq!(ring.pending(1), Err(Error::UsedOverrun));
            assert_eq!(ring.pending(u16::MAX), Ok(0));
            assert_eq!(
                ring.pending(u16::MAX - 1),
                Err(Error::UsedOverrun),
                "a backwards move stays a violation across the wrap"
            );
        }
        assert_eq!(ring.complete(u32::from(head), 4), Ok(head));
    }
    assert!(saw_wrap);
    for head in 0..QUEUE_SIZE {
        ring.publish(head, 8).unwrap();
    }
    assert_eq!(ring.outstanding(), QUEUE_SIZE);
    let full = ring.used_index().wrapping_add(QUEUE_SIZE);
    assert_eq!(ring.pending(full), Ok(QUEUE_SIZE));
    assert_eq!(ring.pending(full.wrapping_add(1)), Err(Error::UsedOverrun));
}

#[test]
fn a_full_ring_straddling_the_index_wrap_stays_bounded() {
    let mut ring = Ring::new();
    while ring.used_index() != 65_532 {
        let head = ring.used_entry();
        ring.publish(head, 8).unwrap();
        ring.complete(u32::from(head), 8).unwrap();
    }
    for head in 0..QUEUE_SIZE {
        ring.publish(head, 8).unwrap();
    }
    assert_eq!(ring.available_index(), 4);
    assert_eq!(ring.outstanding(), QUEUE_SIZE);
    assert_eq!(ring.pending(4), Ok(QUEUE_SIZE));
    assert_eq!(ring.pending(5), Err(Error::UsedOverrun));
    assert_eq!(ring.pending(65_531), Err(Error::UsedOverrun));
    for head in [3, 0, 7, 1, 6, 2, 5, 4] {
        assert_eq!(ring.complete(u32::from(head), 8), Ok(head));
    }
    assert_eq!((ring.used_index(), ring.outstanding()), (4, 0));
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
    life.record_snapshot([1, 4, 9], [1, 4, 9]).unwrap();
    life.check_snapshot([1, 4, 9], 0, false).unwrap();
    assert!(!life.may_publish());
    assert_eq!(life.finish_fence([1, 4, 9], 0, false), Ok(1));
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
        life.record_snapshot([1, 4, 9], [1, 4, 9]).unwrap();
        assert_eq!(
            life.check_snapshot(used, status, bme),
            Err(Error::Quarantined)
        );
        assert_eq!(
            life.finish_fence([1, 4, 9], 0, false),
            Err(Error::Quarantined)
        );
    }
    // Deadline overflow and epoch overflow.
    let mut overflow = Lifecycle::new();
    assert_eq!(overflow.begin_fence(u64::MAX, 1), Err(Error::Quarantined));
    let mut epoch = Lifecycle::with_epoch(u64::MAX);
    epoch.begin_fence(0, 10).unwrap();
    assert_eq!(epoch.poll_fence(1, 0, false), FenceStep::Acknowledged);
    epoch.record_snapshot([0; 3], [0; 3]).unwrap();
    epoch.check_snapshot([0; 3], 0, false).unwrap();
    assert_eq!(
        epoch.finish_fence([0; 3], 0, false),
        Err(Error::Quarantined)
    );
    // Quarantine is terminal for every transition.
    for mut life in [Lifecycle::new(), overflow, epoch] {
        life.quarantine();
        assert!(!life.may_publish());
        assert_eq!(life.go_live(), Err(Error::Quarantined));
        assert_eq!(life.begin_fence(0, 10), Err(Error::Quarantined));
        assert_eq!(life.poll_fence(0, 0, false), FenceStep::Quarantined);
        assert_eq!(
            life.record_snapshot([0; 3], [0; 3]),
            Err(Error::Quarantined)
        );
        assert_eq!(life.finish_fence([0; 3], 0, false), Err(Error::Quarantined));
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

/// One completed and consumed element, so `used_index()` is 1 and the ring
/// is idle again.
fn idle_after_one() -> Ring {
    let mut ring = Ring::new();
    ring.publish(0, 8).unwrap();
    ring.complete(0, 8).unwrap();
    ring
}

#[test]
fn publication_rejects_a_used_index_that_moved_while_idle() {
    let pool = pool();
    let chains = [
        control_chain(&pool, 24, 4).unwrap(),
        pcm_info_chain(&pool, 16, 36).unwrap(),
        tx_chain(&pool, 0, FIXTURE_PERIOD_BYTES as usize).unwrap(),
    ];
    let event = event_chain(&pool, 0).unwrap();
    let base = idle_after_one();
    let used = base.used_index();
    for stale in [
        used.wrapping_add(1),
        used.wrapping_add(QUEUE_SIZE),
        used.wrapping_sub(1),
        u16::MAX,
    ] {
        for chain in chains {
            let mut ring = base;
            let refused = ring.publish_checked(stale, &chain);
            assert_eq!(refused, Err(Error::UsedOverrun), "{stale} {chain:?}");
            assert!(refused.unwrap_err().requires_fence());
            assert_eq!(ring, base, "a refused publication changes nothing");
            assert_eq!(
                ring.publish_checked(used, &chain),
                Ok(Publication { entry: 1, index: 2 }),
                "the same head stays publishable"
            );
        }
        let mut ring = base;
        assert_eq!(ring.publish_checked(stale, &event), Err(Error::UsedOverrun));
        assert_eq!(ring, base);
    }
    // An unchanged index on an idle ring publishes normally.
    let mut ring = base;
    assert_eq!(
        ring.publish_checked(used, &chains[2]),
        Ok(Publication { entry: 1, index: 2 })
    );
    assert_eq!(ring.outstanding(), 1);
}

#[test]
fn publication_allows_only_completions_of_requests_already_outstanding() {
    let mut ring = Ring::new();
    ring.publish(2, 8).unwrap();
    ring.publish(4, 8).unwrap();
    let pool = pool();
    let next = tx_chain(&pool, 3, 1920).unwrap();
    for moved in [0, 1, 2] {
        let mut copy = ring;
        assert_eq!(
            copy.publish_checked(moved, &next),
            Ok(Publication { entry: 2, index: 3 }),
            "{moved} unconsumed completions are legitimate"
        );
    }
    let mut copy = ring;
    assert_eq!(copy.publish_checked(3, &next), Err(Error::UsedOverrun));
    assert_eq!(copy, ring);
}

#[test]
fn a_stale_completion_is_caught_only_before_publication() {
    // Regression for the publication order: once a head is published, the
    // same stale used element (id 0, the PCM_INFO alternate length 4, or TX
    // head 0 with length 8) is indistinguishable from its real completion.
    let pool = pool();
    for chain in [
        pcm_info_chain(&pool, 16, 36).unwrap(),
        tx_chain(&pool, 0, 1920).unwrap(),
    ] {
        let reported = if chain.alternate_used_len != 0 {
            chain.alternate_used_len
        } else {
            chain.used_len
        };
        let base = idle_after_one();
        let stale = base.used_index().wrapping_add(1);
        let mut late = base;
        late.publish_chain(chain.head, chain.used_len, chain.alternate_used_len)
            .unwrap();
        assert_eq!(late.pending(stale), Ok(1));
        assert_eq!(
            late.complete(u32::from(chain.head), reported),
            Ok(chain.head)
        );
        let mut checked = base;
        assert_eq!(
            checked.publish_checked(stale, &chain),
            Err(Error::UsedOverrun)
        );
    }
}

fn acknowledged() -> Lifecycle {
    let mut life = Lifecycle::new();
    life.go_live().unwrap();
    life.begin_fence(0, 10).unwrap();
    assert_eq!(life.poll_fence(1, 0, false), FenceStep::Acknowledged);
    life
}

#[test]
fn a_used_index_moving_across_the_acknowledging_read_quarantines() {
    for lane in 0..3 {
        let mut after = [1, 4, 9];
        after[lane] += 1;
        let mut life = acknowledged();
        assert_eq!(
            life.record_snapshot([1, 4, 9], after),
            Err(Error::Quarantined),
            "lane {lane}"
        );
        assert_eq!(life.phase(), Phase::Quarantined);
        assert_eq!(
            life.check_snapshot([1, 4, 9], 0, false),
            Err(Error::Quarantined)
        );
        assert_eq!(
            life.check_snapshot(after, 0, false),
            Err(Error::Quarantined)
        );
        assert_eq!(life.finish_fence(after, 0, false), Err(Error::Quarantined));
        assert_eq!(life.epoch(), 0);
    }
    // An unchanged bracket is the baseline.
    let mut life = acknowledged();
    life.record_snapshot([1, 4, 9], [1, 4, 9]).unwrap();
    life.check_snapshot([1, 4, 9], 0, false).unwrap();
    assert_eq!(life.finish_fence([1, 4, 9], 0, false), Ok(1));
    // A baseline before the acknowledgement, or a second one, quarantines.
    let mut early = Lifecycle::new();
    early.go_live().unwrap();
    early.begin_fence(0, 10).unwrap();
    assert_eq!(
        early.record_snapshot([0; 3], [0; 3]),
        Err(Error::Quarantined)
    );
    assert_eq!(early.phase(), Phase::Quarantined);
    let mut twice = acknowledged();
    twice.record_snapshot([1, 4, 9], [1, 4, 9]).unwrap();
    assert_eq!(
        twice.record_snapshot([1, 4, 9], [1, 4, 9]),
        Err(Error::Quarantined)
    );
    assert_eq!(
        twice.finish_fence([1, 4, 9], 0, false),
        Err(Error::Quarantined)
    );
}

#[test]
fn finish_fence_requires_a_matching_final_observation() {
    for (used, status, bme) in [
        ([1, 4, 10], 0, false),
        ([1, 4, 9], 1, false),
        ([1, 4, 9], 0, true),
    ] {
        // No stability check in between: the final observation alone must
        // still refuse the epoch.
        let mut life = acknowledged();
        life.record_snapshot([1, 4, 9], [1, 4, 9]).unwrap();
        assert_eq!(
            life.finish_fence(used, status, bme),
            Err(Error::Quarantined)
        );
        assert_eq!((life.phase(), life.epoch()), (Phase::Quarantined, 0));
        assert!(!life.may_publish());
        assert_eq!(life.go_live(), Err(Error::Quarantined));
    }
    let mut life = acknowledged();
    life.record_snapshot([1, 4, 9], [1, 4, 9]).unwrap();
    assert_eq!(life.finish_fence([1, 4, 9], 0, false), Ok(1));
    assert_eq!((life.phase(), life.epoch()), (Phase::Offline, 1));
    // Without a baseline, or from any other phase, there is no epoch.
    let mut unrecorded = acknowledged();
    assert_eq!(
        unrecorded.finish_fence([1, 4, 9], 0, false),
        Err(Error::Quarantined)
    );
    let mut fencing = Lifecycle::new();
    fencing.begin_fence(0, 10).unwrap();
    let mut live = Lifecycle::new();
    live.go_live().unwrap();
    for mut other in [Lifecycle::new(), live, fencing] {
        assert_eq!(
            other.finish_fence([0; 3], 0, false),
            Err(Error::Quarantined)
        );
        assert_eq!((other.phase(), other.epoch()), (Phase::Quarantined, 0));
    }
}

#[test]
fn the_largest_accepted_period_fits_one_tx_slot() {
    let mut bytes = [0u8; 4 + 32];
    bytes[..4].copy_from_slice(&S_OK.to_le_bytes());
    bytes[4 + 8..4 + 16].copy_from_slice(&(1u64 << 5).to_le_bytes());
    bytes[4 + 16..4 + 24].copy_from_slice(&(1u64 << 7).to_le_bytes());
    bytes[4 + 25] = 1;
    bytes[4 + 26] = 2;
    let mut config = [0u8; 12];
    config[4..8].copy_from_slice(&1u32.to_le_bytes());
    let config = Config::parse(negotiate_features(1 << 32).unwrap(), &config).unwrap();
    let info = config
        .query_stream(0)
        .unwrap()
        .parse_response(&bytes)
        .unwrap();
    let mut stream = pcm::Stream::new(info.as_slice()[0]).unwrap();
    stream.set_params(MAX_PERIODS, MAX_PERIOD_BYTES).unwrap();
    stream.complete_control(&S_OK.to_le_bytes()).unwrap();
    stream.prepare().unwrap();
    stream.complete_control(&S_OK.to_le_bytes()).unwrap();
    let pool = pool();
    for slot in 0..TX_SLOTS {
        assert!(stream.tx_header(MAX_PERIOD_BYTES as usize).is_ok());
        let chain = tx_chain(&pool, slot, MAX_PERIOD_BYTES as usize).unwrap();
        assert_eq!(chain.entries[0].1.length, SLOT_BYTES);
        for (_, descriptor) in chain.entries {
            assert_descriptor_contained(&pool, descriptor);
        }
    }
    let over = (MAX_PERIOD_BYTES + FRAME_BYTES) as usize;
    assert_eq!(tx_chain(&pool, 0, over), Err(Error::Range));
    assert_eq!(stream.tx_header(over), Err(pcm::Error::Range));
    let mut fresh = pcm::Stream::new(info.as_slice()[0]).unwrap();
    assert_eq!(
        fresh.set_params(1, MAX_PERIOD_BYTES + FRAME_BYTES),
        Err(pcm::Error::Range)
    );
}

#[test]
fn header_only_pcm_info_in_a_zeroed_buffer_fails_closed() {
    // The PCM_INFO alternate used length 4 hands the parser the whole zeroed
    // `4 + 32` buffer; a device that really wrote only the header leaves an
    // all-zero record, which the strict record parser refuses.
    let mut config = [0u8; 12];
    config[4..8].copy_from_slice(&1u32.to_le_bytes());
    let config = Config::parse(negotiate_features(1 << 32).unwrap(), &config).unwrap();
    let query = config.query_stream(0).unwrap();
    let mut buffer = [0u8; 36];
    buffer[..4].copy_from_slice(&S_OK.to_le_bytes());
    assert_eq!(
        query.parse_response(&buffer).err(),
        Some(CodecError::InvalidChannels)
    );
    buffer[..4].copy_from_slice(&pcm::S_BAD_MSG.to_le_bytes());
    assert_eq!(
        query.parse_response(&buffer).err(),
        Some(CodecError::Status(pcm::S_BAD_MSG))
    );
}

#[test]
fn only_event_heads_complete_with_a_short_used_length() {
    let pool = pool();
    let event = event_chain(&pool, 1).unwrap();
    assert!(event.accept_short);
    for short in [0, 4, 7] {
        let mut ring = Ring::new();
        ring.publish_checked(0, &event).unwrap();
        assert_eq!(ring.pending(1), Ok(1));
        assert_eq!(ring.complete(1, short), Ok(1), "{short}");
        assert_eq!((ring.outstanding(), ring.used_index()), (0, 1));
        assert_eq!(
            ring.publish_checked(1, &event),
            Ok(Publication { entry: 1, index: 2 }),
            "the head is free again"
        );
        // A later strict publication of the same head is strict again.
        let mut strict = Ring::new();
        strict.publish_checked(0, &event).unwrap();
        strict.complete(1, short).unwrap();
        strict.publish(1, 8).unwrap();
        assert_eq!(strict.complete(1, short), Err(Error::Length));
    }
    for long in [9, 16, u32::MAX] {
        let mut ring = Ring::new();
        ring.publish_checked(0, &event).unwrap();
        let before = ring;
        assert_eq!(ring.complete(1, long), Err(Error::Length), "{long}");
        assert_eq!(ring, before);
    }
    // Unknown or not outstanding heads stay UnknownId, whatever the length.
    let mut ring = Ring::new();
    ring.publish_checked(0, &event).unwrap();
    assert_eq!(ring.complete(2, 4), Err(Error::UnknownId));
    assert_eq!(ring.complete(u32::MAX, 0), Err(Error::UnknownId));
    // TX, control and PCM_INFO heads never take a short answer.
    for chain in [
        tx_chain(&pool, 0, 1920).unwrap(),
        control_chain(&pool, 24, 4).unwrap(),
    ] {
        assert!(!chain.accept_short);
        for short in [0, 1, chain.used_len - 1] {
            let mut ring = Ring::new();
            ring.publish_checked(0, &chain).unwrap();
            let before = ring;
            assert_eq!(
                ring.complete(u32::from(chain.head), short),
                Err(Error::Length)
            );
            assert_eq!(ring, before);
        }
    }
    let info = pcm_info_chain(&pool, 16, 36).unwrap();
    assert!(!info.accept_short);
    for (reported, expected) in [
        (36, Ok(0)),
        (4, Ok(0)),
        (0, Err(Error::Length)),
        (35, Err(Error::Length)),
    ] {
        let mut ring = Ring::new();
        ring.publish_checked(0, &info).unwrap();
        assert_eq!(ring.complete(0, reported), expected, "{reported}");
    }
    // `publish` and `publish_chain` default to strict lengths.
    let mut plain = Ring::new();
    plain.publish(1, 8).unwrap();
    assert_eq!(plain.complete(1, 4), Err(Error::Length));
    plain.publish_chain(2, 8, 0).unwrap();
    assert_eq!(plain.complete(2, 4), Err(Error::Length));
}

#[test]
fn an_event_poll_handles_at_most_events_per_poll() {
    // A device that completes every re-posted event buffer at once.
    let mut calls = 0u32;
    let handled = bounded_poll::<()>(EVENTS_PER_POLL, || {
        calls += 1;
        Ok(true)
    });
    assert_eq!(handled, Ok(EVENTS_PER_POLL));
    assert_eq!(calls, u32::from(EVENTS_PER_POLL));
    assert_eq!(EVENTS_PER_POLL, EVENT_BUFFERS);
    let mut left = 2;
    assert_eq!(
        bounded_poll::<()>(EVENTS_PER_POLL, || {
            left -= 1;
            Ok(left >= 0)
        }),
        Ok(2)
    );
    let mut errors = 0;
    assert_eq!(
        bounded_poll(EVENTS_PER_POLL, || {
            errors += 1;
            Err::<bool, _>(Error::Length)
        }),
        Err(Error::Length)
    );
    assert_eq!(errors, 1);
    assert_eq!(bounded_poll::<()>(0, || panic!("never called")), Ok(0));
}

/// CHARACTERISATION of the documented F8 limit (TRANSPORT.md F8 row): the
/// idle check is a sample. A stale used element that the device writes after
/// `publish_checked` read the used index, but before the available-index
/// store, has the new chain's head and length and is accepted as its
/// completion. Virtio carries no per-publication tag that could tell the two
/// apart, so this window is accepted only under the fixture's trusted-transport
/// assumption (ADR-H03-SND-FIXTURE-20261119: no IOMMU, no DMA-isolation
/// claim). If this test ever starts failing because the window was closed,
/// update the F8 row instead of deleting the test.
#[test]
fn stale_completion_inside_the_publication_window_is_a_documented_limit() {
    let pool = pool();
    let chain = control_chain(&pool, 24, 4).unwrap();
    let mut ring = idle_after_one();
    let sampled = ring.used_index();
    // Interleaving: sample, publish (outstanding becomes 1), then the device
    // advances the used index by one with head 0 before the driver's poll.
    ring.publish_checked(sampled, &chain).unwrap();
    let stale = sampled.wrapping_add(1);
    assert_eq!(
        ring.pending(stale),
        Ok(1),
        "indistinguishable from a real completion"
    );
    assert_eq!(
        ring.complete(u32::from(chain.head), chain.used_len),
        Ok(chain.head)
    );
    // Outside that window the same stale element is refused (the F8 check).
    let mut idle = idle_after_one();
    assert_eq!(idle.publish_checked(stale, &chain), Err(Error::UsedOverrun));
}
