use super::*;
fn set32(b: &mut [u8; 256], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}
fn pci() -> ([u8; 256], [Bar; 6]) {
    let mut b = [0; 256];
    b[..4].copy_from_slice(&[0xf4, 0x1a, 0x42, 0x10]);
    b[6] = 0x10;
    b[0x34] = 0x40;
    set32(&mut b, 0x10, 0x10000000);
    let offsets = [0, 0x100, 0x200, 0x300];
    let sizes = [56, 32, 1, 8];
    for i in 0..4 {
        let p = 0x40 + i * 20;
        b[p] = 9;
        b[p + 1] = if i == 3 { 0 } else { (p + 20) as u8 };
        b[p + 2] = if i == 1 { 20 } else { 16 };
        b[p + 3] = (i + 1) as u8;
        set32(&mut b, p + 8, offsets[i]);
        set32(&mut b, p + 12, sizes[i]);
    }
    set32(&mut b, 0x54 + 16, 4);
    let mut bars = [Bar::NONE; 6];
    bars[0] = Bar {
        base: 0x10000000,
        size: 4096,
        is_memory: true,
    };
    (b, bars)
}
fn pool() -> DmaPool {
    DmaPool {
        descriptors: 4096,
        available: 8192,
        used: 12288,
        request: 16384,
        bounce: 20480,
    }
}
fn binding() -> Binding {
    Binding {
        pid: 7,
        object_key: (1 << 32) | 64,
    }
}
fn read() -> Read {
    Read {
        sector: 4,
        offset: 0,
        length: 128,
    }
}
fn online() -> Policy {
    let mut p = Policy::new();
    p.reset_completed(true).unwrap();
    p
}

#[test]
fn pci_regions_and_checked_notify_address() {
    let (b, bars) = pci();
    let r = parse_capabilities(&b, &bars).unwrap();
    assert_eq!(r.common.address(&bars), Ok(0x10000000));
    assert_eq!(r.device.length, 8);
    assert_eq!(r.notify_address(&bars, 7), Ok(0x1000011c));
    assert_eq!(r.notify_address(&bars, 8), Err(Error::Range));
    let mut r = r;
    r.notify_multiplier = u32::MAX;
    assert_eq!(r.notify_address(&bars, u16::MAX), Err(Error::Range));
}
#[test]
fn pci_chain_cycle_alignment_length_and_bounds_fail_closed() {
    let (base, bars) = pci();
    for next in [0x40, 0x41, 0x20, 0xfc] {
        let mut b = base;
        b[0x7c + 1] = next;
        assert!(parse_capabilities(&b, &bars).is_err());
    }
    for len in [0, 3, 15, 255] {
        let mut b = base;
        b[0x40 + 2] = len;
        assert!(parse_capabilities(&b, &bars).is_err());
    }
    let mut b = base;
    b[0x40 + 4] = 6;
    assert!(parse_capabilities(&b, &bars).is_err());
    let mut b = base;
    set32(&mut b, 0x40 + 8, u32::MAX);
    assert!(parse_capabilities(&b, &bars).is_err());
    let mut b = base;
    set32(&mut b, 0x40 + 12, u32::MAX);
    assert!(parse_capabilities(&b, &bars).is_err());
    let mut b = base;
    set32(&mut b, 0x40 + 8, 4);
    assert_eq!(parse_capabilities(&b, &bars), Err(Error::Protocol));
    let mut b = base;
    set32(&mut b, 0x54 + 8, 0);
    assert_eq!(parse_capabilities(&b, &bars), Err(Error::Protocol));
    let mut b = base;
    b[0x7c + 3] = 1;
    assert_eq!(parse_capabilities(&b, &bars), Err(Error::Protocol));
    let mut b = base;
    b[0x34] = 0;
    assert_eq!(parse_capabilities(&b, &bars), Err(Error::Unsupported));
}
#[test]
fn pci_bar_io_wide_upper_half_and_absolute_overflow() {
    let (base, bars) = pci();
    let mut b = base;
    set32(&mut b, 0x10, 0x10000001);
    assert!(parse_capabilities(&b, &bars).is_err());
    let mut b = base;
    set32(&mut b, 0x10, 0x10000004);
    assert!(parse_capabilities(&b, &bars).is_ok());
    b[0x40 + 4] = 1;
    assert!(parse_capabilities(&b, &bars).is_err());
    let mut b = base;
    set32(&mut b, 0x10, 0xfffff004);
    set32(&mut b, 0x14, u32::MAX);
    let mut bars = bars;
    bars[0].base = u64::MAX - 4095;
    assert!(parse_capabilities(&b, &bars).is_err());
    let mut high = base;
    set32(&mut high, 0x10, 4);
    set32(&mut high, 0x14, 1 << 20);
    let mut high_bars = bars;
    high_bars[0].base = 1 << 52;
    assert_eq!(parse_capabilities(&high, &high_bars), Err(Error::Range));
    bars[0].base = 0x10000000;
    bars[0].size = 3;
    assert!(parse_capabilities(&base, &bars).is_err());
}
#[test]
fn pci_bounded_single_byte_mutations_never_index_outside_snapshot() {
    let (base, bars) = pci();
    for index in 0..256 {
        for value in [0, 1, 63, 64, 252, 255] {
            let mut b = base;
            b[index] = value;
            let _ = parse_capabilities(&b, &bars);
        }
    }
}
#[test]
fn feature_negotiation_is_readonly_modern_without_platform_claim() {
    assert_eq!(
        negotiate_features(VERSION_1 | READ_ONLY | u32::MAX as u64),
        Ok(VERSION_1 | READ_ONLY)
    );
    for f in [
        0,
        VERSION_1,
        READ_ONLY,
        VERSION_1 | READ_ONLY | ACCESS_PLATFORM,
    ] {
        assert_eq!(negotiate_features(f), Err(Error::Unsupported));
    }
}
#[test]
fn descriptor_chain_is_fixed_and_confined_to_distinct_pinned_page_plan() {
    let p = pool();
    let d = descriptor_plan(p, 65536).unwrap();
    validate_descriptors(&d, p, 65536).unwrap();
    assert_eq!(d[0].address, p.request);
    assert_eq!(d[1].length, 512);
    assert_eq!(d[2].address, p.request + 16);
    assert_eq!(&d[1].encode()[8..12], &512u32.to_le_bytes());
    for i in 0..3 {
        let mut bad = d;
        bad[i].address = 0xe000;
        assert_eq!(validate_descriptors(&bad, p, 65536), Err(Error::Protocol));
        let mut bad = d;
        bad[i].length = u32::MAX;
        assert_eq!(validate_descriptors(&bad, p, 65536), Err(Error::Protocol));
        let mut bad = d;
        bad[i].flags |= 4;
        assert_eq!(validate_descriptors(&bad, p, 65536), Err(Error::Protocol));
        let mut bad = d;
        bad[i].next = 7;
        assert_eq!(validate_descriptors(&bad, p, 65536), Err(Error::Protocol));
    }
    for value in [0, 1, u64::MAX - 4095, 8192] {
        let mut bad = p;
        bad.descriptors = value;
        assert!(descriptor_plan(bad, 65536).is_err());
    }
    assert!(descriptor_plan(p, 20480).is_err());
    assert_eq!(&read_header(4)[8..], &4u64.to_le_bytes());
}
#[test]
fn used_validation_delta_head_length_and_status() {
    assert_eq!(
        validate_used(0, 0, 0, u32::MAX, u32::MAX, 255),
        Err(Error::Pending)
    );
    assert_eq!(
        validate_used(0, u16::MAX, 0, 0, 513, 0),
        Ok(Completion::Success)
    );
    for new in [2, u16::MAX] {
        assert_eq!(validate_used(0, 0, new, 0, 513, 0), Err(Error::Protocol));
    }
    for id in [1, 8, u32::MAX] {
        assert_eq!(validate_used(0, 0, 1, id, 513, 0), Err(Error::Protocol));
    }
    for len in [0, 1, 128, 512, 514, u32::MAX] {
        assert_eq!(validate_used(0, 0, 1, 0, len, 0), Err(Error::Protocol));
    }
    assert_eq!(validate_used(0, 0, 1, 0, 1, 1), Ok(Completion::Io));
    assert_eq!(validate_used(0, 0, 1, 0, 1, 2), Ok(Completion::Unsupported));
    for status in [3, 255] {
        assert_eq!(validate_used(0, 0, 1, 0, 513, status), Err(Error::Protocol));
    }
    assert_eq!(validate_used(8, 0, 1, 8, 513, 0), Err(Error::Protocol));
}
#[test]
fn pending_bindings_and_consume_are_transactional() {
    let mut p = online();
    let r = p.submit(binding(), read(), 8, 10, 20).unwrap();
    let before = p;
    assert_eq!(p.submit(binding(), read(), 8, 10, 20), Err(Error::Busy));
    assert_eq!(p, before);
    assert_eq!(p.peek(binding(), r.ticket), Err(Error::Pending));
    assert_eq!(p, before);
    p.complete(r.epoch, r.ticket, Completion::Success).unwrap();
    let completed = p;
    for b in [
        Binding {
            pid: 8,
            ..binding()
        },
        Binding {
            object_key: (2 << 32) | 64,
            ..binding()
        },
    ] {
        assert_eq!(p.consume(b, r.ticket), Err(Error::WrongTicket));
        assert_eq!(p, completed);
    }
    assert_eq!(p.consume(binding(), r.ticket), Ok(Completion::Success));
    assert_eq!(p.phase(), Phase::Ready);
    assert_eq!(p.consume(binding(), r.ticket), Err(Error::WrongTicket));
}
#[test]
fn request_bounds_and_overflow_do_not_publish() {
    let mut p = online();
    let before = p;
    for r in [
        Read {
            length: 0,
            ..read()
        },
        Read {
            length: 129,
            ..read()
        },
        Read {
            offset: 512,
            ..read()
        },
        Read {
            offset: u64::MAX,
            ..read()
        },
        Read {
            sector: 8,
            ..read()
        },
    ] {
        assert_eq!(p.submit(binding(), r, 8, 0, 20), Err(Error::Range));
        assert_eq!(p, before);
    }
    assert_eq!(
        p.submit(binding(), read(), 8, u64::MAX, 1),
        Err(Error::Range)
    );
    assert_eq!(p, before);
    let r = p
        .submit(
            binding(),
            Read {
                sector: 7,
                offset: 384,
                length: 128,
            },
            8,
            0,
            20,
        )
        .unwrap();
    assert_eq!(r.offset, 384);
}
#[test]
fn timeout_requires_reset_ack_and_never_frees_quarantine() {
    let mut p = online();
    let r = p.submit(binding(), read(), 8, 0, 20).unwrap();
    assert!(!p.timeout(19));
    assert!(p.timeout(20));
    assert_eq!(p.phase(), Phase::Resetting);
    assert_eq!(
        p.complete(r.epoch, r.ticket, Completion::Success),
        Err(Error::Offline)
    );
    assert_eq!(p.reset_completed(false), Err(Error::Offline));
    assert_eq!(p.phase(), Phase::Quarantined);
    assert_eq!(p.request(), Some(r));
    let before = p;
    assert_eq!(p.reset_completed(true), Err(Error::Offline));
    assert_eq!(p.start_reset(), Err(Error::Offline));
    assert_eq!(p, before);
}
#[test]
fn software_epoch_separates_old_tickets_after_acknowledged_reset() {
    let mut p = online();
    let old = p.submit(binding(), read(), 8, 0, 20).unwrap();
    p.start_reset().unwrap();
    p.reset_completed(true).unwrap();
    let next = p.submit(binding(), read(), 8, 21, 20).unwrap();
    assert_ne!(old.ticket, next.ticket);
    assert_ne!(old.epoch, next.epoch);
    let before = p;
    // This is caller-supplied software provenance, NOT a nonce from the device.
    assert_eq!(
        p.complete(old.epoch, old.ticket, Completion::Success),
        Err(Error::StaleEpoch)
    );
    assert_eq!(p, before);
    p.complete(next.epoch, next.ticket, Completion::Io).unwrap();
    assert_eq!(p.consume(binding(), next.ticket), Ok(Completion::Io));
}
#[test]
fn maximum_epoch_and_ticket_do_not_wrap() {
    let mut p = Policy::new();
    p.epoch = u64::MAX;
    assert_eq!(p.reset_completed(true), Err(Error::Exhausted));
    assert_eq!(p.phase(), Phase::Quarantined);
    let mut p = online();
    p.next_ticket = u64::MAX;
    let before = p;
    assert_eq!(p.submit(binding(), read(), 8, 0, 20), Err(Error::Exhausted));
    assert_eq!(p, before);
}

#[test]
fn m6_writable_feature_profile_cannot_weaken_m5_or_assume_flush() {
    assert_eq!(
        negotiate_writable_features(VERSION_1 | FLUSH | (1 << 11)),
        Ok(VERSION_1 | FLUSH)
    );
    for offered in [
        0,
        VERSION_1,
        FLUSH,
        VERSION_1 | FLUSH | READ_ONLY,
        VERSION_1 | FLUSH | ACCESS_PLATFORM,
    ] {
        assert_eq!(
            negotiate_writable_features(offered),
            Err(Error::Unsupported)
        );
    }
    assert_eq!(
        negotiate_features(VERSION_1 | FLUSH),
        Err(Error::Unsupported)
    );
    assert_eq!(
        negotiate_writable_features(VERSION_1 | READ_ONLY),
        Err(Error::Unsupported)
    );
}

#[test]
fn m6_exact_operation_chains_headers_and_all_descriptor_mutations() {
    for op in [
        Operation::Read {
            sector: 7,
            offset: 384,
            length: 128,
        },
        Operation::Write { sector: 7 },
        Operation::Flush,
    ] {
        let plan = descriptor_plan_for(op, pool(), 65536).unwrap();
        validate_descriptors_for(&plan, op, pool(), 65536).unwrap();
        let header = header_for(op);
        assert_eq!(&header[4..8], &[0; 4]);
        assert_eq!(&header[8..], &op.sector().to_le_bytes());
        assert_eq!(plan.entries[0].flags, NEXT);
        match op {
            Operation::Read { .. } => assert_eq!(plan.entries[1].flags, NEXT | WRITE),
            Operation::Write { .. } => {
                assert_eq!(&header[..4], &1u32.to_le_bytes());
                assert_eq!(plan.entries[1].flags, NEXT);
            }
            Operation::Flush => {
                assert_eq!(&header[..4], &4u32.to_le_bytes());
                assert_eq!(plan.count, 2);
                assert_eq!(plan.entries[1].address, pool().request + 16);
                assert_eq!(plan.entries[2].encode(), [0; 16]);
            }
        }
        for index in 0..3 {
            for change in 0..4 {
                let mut bad = plan;
                match change {
                    0 => bad.entries[index].address ^= 4096,
                    1 => bad.entries[index].length ^= 1,
                    2 => bad.entries[index].flags ^= WRITE,
                    _ => bad.entries[index].next ^= 1,
                }
                assert_eq!(
                    validate_descriptors_for(&bad, op, pool(), 65536),
                    Err(Error::Protocol)
                );
            }
        }
        for count in [0, 1, 4, 255] {
            let mut bad = plan;
            bad.count = count;
            assert_eq!(
                validate_descriptors_for(&bad, op, pool(), 65536),
                Err(Error::Protocol)
            );
        }
    }
    let op = Operation::Write { sector: 7 };
    for length in [0, 511, 513, u32::MAX] {
        let mut plan = descriptor_plan_for(op, pool(), 65536).unwrap();
        plan.entries[1].length = length;
        assert_eq!(
            validate_descriptors_for(&plan, op, pool(), 65536),
            Err(Error::Protocol)
        );
    }
}

#[test]
fn m6_used_completion_is_operation_specific_never_decodes_unknown() {
    for op in [Operation::Write { sector: 4 }, Operation::Flush] {
        for status in [0, 1, 2] {
            let expected =
                [Completion::Success, Completion::Io, Completion::Unsupported][status as usize];
            assert_eq!(
                validate_used_for(op, 0, u16::MAX, 0, 0, 1, status),
                Ok(expected)
            );
            for len in [0, 2, 512, 513, u32::MAX] {
                assert_eq!(
                    validate_used_for(op, 0, 0, 1, 0, len, status),
                    Err(Error::Protocol)
                );
            }
        }
        for (idx, id, status) in [
            (2, 0, 0),
            (1, 1, 0),
            (1, u32::MAX, 0),
            (1, 0, 3),
            (1, 0, 255),
        ] {
            assert_eq!(
                validate_used_for(op, 0, 0, idx, id, 1, status),
                Err(Error::Protocol)
            );
        }
    }
}

#[test]
fn m6_prepared_rejection_has_no_publication_and_failures_are_atomic() {
    let mut p = online();
    let r = p
        .prepare(binding(), Operation::Write { sector: 7 }, 8, 0, 20)
        .unwrap();
    assert_eq!(p.phase(), Phase::Prepared);
    assert!(!r.published);
    assert_eq!((r.offset, r.length), (0, 512));
    let before = p;
    assert_eq!(
        p.complete(r.epoch, r.ticket, Completion::Success),
        Err(Error::Offline)
    );
    assert_eq!(p, before);
    assert_eq!(p.publish(r.epoch + 1, r.ticket), Err(Error::StaleEpoch));
    assert_eq!(p.publish(r.epoch, r.ticket + 1), Err(Error::WrongTicket));
    assert_eq!(
        p.reject_prepared(
            Binding {
                pid: 8,
                ..binding()
            },
            r.ticket
        ),
        Err(Error::WrongTicket)
    );
    assert_eq!(p, before);
    p.reject_prepared(binding(), r.ticket).unwrap();
    assert_eq!(p.phase(), Phase::Ready);
    assert_eq!(p.request(), None);
    for op in [
        Operation::Write { sector: 8 },
        Operation::Read {
            sector: 4,
            offset: 0,
            length: 129,
        },
    ] {
        let before = p;
        assert_eq!(p.prepare(binding(), op, 8, 0, 20), Err(Error::Range));
        assert_eq!(p, before);
    }
}

#[test]
fn m6_published_unknown_survives_reset_and_cannot_replay() {
    for op in [Operation::Write { sector: 4 }, Operation::Flush] {
        let mut p = online();
        let r = p.prepare(binding(), op, 8, 0, 20).unwrap();
        p.publish(r.epoch, r.ticket).unwrap();
        assert_eq!(p.phase(), Phase::Published);
        assert!(p.request().unwrap().published);
        let before = p;
        assert_eq!(p.reject_prepared(binding(), r.ticket), Err(Error::Offline));
        assert_eq!(p, before);
        assert!(!p.timeout(19));
        assert!(p.timeout(20));
        assert_eq!(p.request().unwrap().completion, Some(Completion::Unknown));
        p.reset_completed(true).unwrap();
        assert_eq!(p.phase(), Phase::Complete);
        assert_ne!(p.epoch(), r.epoch);
        let before = p;
        assert_eq!(p.prepare(binding(), op, 8, 21, 20), Err(Error::Busy));
        assert_eq!(
            p.complete(r.epoch, r.ticket, Completion::Success),
            Err(Error::StaleEpoch)
        );
        assert_eq!(
            p.consume(
                Binding {
                    object_key: (2 << 32) | 64,
                    ..binding()
                },
                r.ticket
            ),
            Err(Error::WrongTicket)
        );
        assert_eq!(p, before);
        assert_eq!(p.consume(binding(), r.ticket), Ok(Completion::Unknown));
        assert_eq!(p.phase(), Phase::Ready);
        let next = p.prepare(binding(), op, 8, 21, 20).unwrap();
        assert_ne!(next.ticket, r.ticket);
    }
}

#[test]
fn m6_unknown_poll_during_reset_or_quarantine_never_releases_pool() {
    for ack in [None, Some(false)] {
        let mut p = online();
        let r = p
            .prepare(binding(), Operation::Write { sector: 4 }, 8, 0, 20)
            .unwrap();
        p.publish(r.epoch, r.ticket).unwrap();
        p.start_reset().unwrap();
        if let Some(ack) = ack {
            assert_eq!(p.reset_completed(ack), Err(Error::Offline));
        }
        let phase = p.phase();
        assert_eq!(p.consume(binding(), r.ticket), Ok(Completion::Unknown));
        assert_eq!(p.phase(), phase);
        assert!(p.prepare(binding(), Operation::Flush, 8, 21, 20).is_err());
        if ack.is_none() {
            p.reset_completed(true).unwrap();
            assert_eq!(p.phase(), Phase::Ready);
        } else {
            assert_eq!(p.reset_completed(true), Err(Error::Offline));
        }
    }
}

#[test]
fn m6_device_failures_are_unknown_and_completed_results_do_not_expire() {
    for op in [Operation::Write { sector: 4 }, Operation::Flush] {
        for completion in [Completion::Success, Completion::Io, Completion::Unsupported] {
            let mut p = online();
            let r = p.prepare(binding(), op, 8, 0, 20).unwrap();
            p.publish(r.epoch, r.ticket).unwrap();
            p.complete(r.epoch, r.ticket, completion).unwrap();
            let before = p;
            assert!(!p.timeout(u64::MAX));
            assert_eq!(p, before);
            p.start_reset().unwrap();
            p.reset_completed(true).unwrap();
            assert_eq!(
                p.consume(binding(), r.ticket),
                Ok(if completion == Completion::Success {
                    Completion::Success
                } else {
                    Completion::Unknown
                })
            );
        }
    }
}

#[test]
fn m6_owner_exit_discards_results_without_unquarantining_or_implying_rollback() {
    for published in [false, true] {
        let mut p = online();
        let r = p
            .prepare(binding(), Operation::Write { sector: 4 }, 8, 0, 20)
            .unwrap();
        if published {
            p.publish(r.epoch, r.ticket).unwrap();
        }
        let before = p;
        assert_eq!(p.exit(9), None);
        assert_eq!(p, before);
        let exited = p.exit(7).unwrap();
        assert_eq!(
            exited.completion,
            if published {
                Some(Completion::Unknown)
            } else {
                None
            }
        );
        assert_eq!(p.request(), None);
        assert_eq!(
            p.phase(),
            if published {
                Phase::Resetting
            } else {
                Phase::Ready
            }
        );
    }
    let mut p = online();
    let r = p.prepare(binding(), Operation::Flush, 8, 0, 20).unwrap();
    p.publish(r.epoch, r.ticket).unwrap();
    p.start_reset().unwrap();
    p.reset_completed(false).unwrap_err();
    assert_eq!(p.exit(7).unwrap().completion, Some(Completion::Unknown));
    assert_eq!(p.phase(), Phase::Quarantined);
}

#[test]
fn m6_mutation_epoch_exhaustion_retains_unknown_and_ticket_never_wraps() {
    let mut p = online();
    p.epoch = u64::MAX;
    let r = p.prepare(binding(), Operation::Flush, 8, 0, 20).unwrap();
    p.publish(r.epoch, r.ticket).unwrap();
    p.start_reset().unwrap();
    assert_eq!(p.reset_completed(true), Err(Error::Exhausted));
    assert_eq!(p.phase(), Phase::Quarantined);
    assert_eq!(p.consume(binding(), r.ticket), Ok(Completion::Unknown));
    assert_eq!(p.phase(), Phase::Quarantined);
    let mut p = online();
    p.next_ticket = u64::MAX;
    let before = p;
    assert_eq!(
        p.prepare(binding(), Operation::Write { sector: 4 }, 8, 0, 20),
        Err(Error::Exhausted)
    );
    assert_eq!(p, before);
}
