use super::*;
use crate::PHYSICAL_LIMIT;

fn pool() -> DmaPool {
    DmaPool {
        descriptors: 4096,
        available: 8192,
        used: 12288,
        request: 16384,
        bounce: 20480,
    }
}

fn features() -> Features {
    Features::negotiate(VERSION_1 | FLUSH).unwrap()
}

fn plan(operation: Operation) -> Plan {
    Plan::new(features(), operation, pool(), 65536, 128).unwrap()
}

#[test]
fn writable_features_require_modern_flush_and_reject_readonly_or_platform() {
    for offered in [0, VERSION_1, FLUSH, READ_ONLY, ACCESS_PLATFORM] {
        assert_eq!(Features::negotiate(offered), Err(Error::Unsupported));
    }
    for extra in [READ_ONLY, ACCESS_PLATFORM, READ_ONLY | ACCESS_PLATFORM] {
        assert_eq!(
            Features::negotiate(VERSION_1 | FLUSH | extra),
            Err(Error::Unsupported)
        );
    }
    assert_eq!(features().accepted_bits(), VERSION_1 | FLUSH);
    // Neither CONFIG_WCE nor any queue/offload/unknown bit is negotiated.
    let offered = u64::MAX & !(READ_ONLY | ACCESS_PLATFORM);
    assert_eq!(
        Features::negotiate(offered).unwrap().accepted_bits(),
        VERSION_1 | FLUSH
    );
    // The existing M5 profile still requires READ_ONLY and does not accept a
    // writable offer merely because FLUSH is available.
    assert_eq!(
        crate::negotiate_features(VERSION_1 | FLUSH),
        Err(Error::Unsupported)
    );
    assert_eq!(
        crate::negotiate_features(VERSION_1 | READ_ONLY | FLUSH),
        Ok(VERSION_1 | READ_ONLY)
    );
}

#[test]
fn write_header_and_chain_expose_only_status_for_device_writes() {
    let sector = 0x1020_3040_5060_7080;
    let p = Plan::new(
        features(),
        Operation::Write { sector },
        pool(),
        65536,
        u64::MAX,
    )
    .unwrap();
    assert_eq!(p.operation(), Operation::Write { sector });
    assert_eq!(
        p.header(),
        &[1, 0, 0, 0, 0, 0, 0, 0, 0x80, 0x70, 0x60, 0x50, 0x40, 0x30, 0x20, 0x10]
    );
    assert_eq!(
        p.descriptors(),
        &[
            Descriptor {
                address: pool().request,
                length: 16,
                flags: NEXT,
                next: 1,
            },
            Descriptor {
                address: pool().bounce,
                length: 512,
                flags: NEXT,
                next: 2,
            },
            Descriptor {
                address: pool().request + 16,
                length: 1,
                flags: WRITE,
                next: 0,
            },
        ]
    );
    assert_eq!(p.validate(p.header(), p.descriptors()), Ok(()));
}

#[test]
fn flush_has_zero_sector_and_no_data_descriptor_even_on_empty_device() {
    let p = Plan::new(features(), Operation::Flush, pool(), 65536, 0).unwrap();
    assert_eq!(p.operation(), Operation::Flush);
    assert_eq!(
        p.header(),
        &[4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        p.descriptors(),
        &[
            Descriptor {
                address: pool().request,
                length: 16,
                flags: NEXT,
                next: 1,
            },
            Descriptor {
                address: pool().request + 16,
                length: 1,
                flags: WRITE,
                next: 0,
            },
        ]
    );
    assert_eq!(p.validate(p.header(), p.descriptors()), Ok(()));
    // An unused descriptor is excluded rather than passed to the device.
    assert_eq!(p.validate(p.header(), &p.descriptors), Err(Error::Protocol));
}

#[test]
fn single_sector_write_checks_capacity_without_wrapping() {
    for (sector, capacity) in [(0, 0), (1, 1), (128, 128), (u64::MAX, u64::MAX)] {
        assert_eq!(
            Plan::new(
                features(),
                Operation::Write { sector },
                pool(),
                65536,
                capacity
            ),
            Err(Error::Range)
        );
    }
    for (sector, capacity) in [(0, 1), (127, 128), (u64::MAX - 1, u64::MAX)] {
        assert!(Plan::new(
            features(),
            Operation::Write { sector },
            pool(),
            65536,
            capacity
        )
        .is_ok());
    }
}

#[test]
fn both_operations_validate_every_pool_page_and_physical_limit() {
    for operation in [Operation::Write { sector: 0 }, Operation::Flush] {
        let original = pool();
        for index in 0..5 {
            for value in [0, 1, 4097, 65536, PHYSICAL_LIMIT, u64::MAX - 4095] {
                let mut pages = [4096, 8192, 12288, 16384, 20480];
                pages[index] = value;
                let bad = DmaPool {
                    descriptors: pages[0],
                    available: pages[1],
                    used: pages[2],
                    request: pages[3],
                    bounce: pages[4],
                };
                assert_eq!(
                    Plan::new(features(), operation, bad, 65536, 128),
                    Err(Error::Range)
                );
            }
        }
        for i in 0..5 {
            for j in 0..i {
                let mut pages = [4096, 8192, 12288, 16384, 20480];
                pages[i] = pages[j];
                let bad = DmaPool {
                    descriptors: pages[0],
                    available: pages[1],
                    used: pages[2],
                    request: pages[3],
                    bounce: pages[4],
                };
                assert_eq!(
                    Plan::new(features(), operation, bad, 65536, 128),
                    Err(Error::Range)
                );
            }
        }
        for ram_end in [0, 24575, PHYSICAL_LIMIT + 1, u64::MAX] {
            assert_eq!(
                Plan::new(features(), operation, original, ram_end, 128),
                Err(Error::Range)
            );
        }
        assert!(Plan::new(features(), operation, original, 24576, 128).is_ok());
    }
}

#[test]
fn descriptor_and_header_mutations_fail_before_publication() {
    for operation in [Operation::Write { sector: 7 }, Operation::Flush] {
        let p = plan(operation);
        let before = p;
        for index in 0..16 {
            for bit in 0..8 {
                let mut bad = *p.header();
                bad[index] ^= 1 << bit;
                assert_eq!(p.validate(&bad, p.descriptors()), Err(Error::Protocol));
                assert_eq!(p, before);
            }
        }
        for i in 0..p.count {
            for field in 0..4 {
                let width = match field {
                    0 => 64,
                    1 => 32,
                    _ => 16,
                };
                for bit in 0..width {
                    let mut bad = p.descriptors;
                    match field {
                        0 => bad[i].address ^= 1u64 << bit,
                        1 => bad[i].length ^= 1u32 << bit,
                        2 => bad[i].flags ^= 1u16 << bit,
                        _ => bad[i].next ^= 1u16 << bit,
                    }
                    assert_eq!(
                        p.validate(p.header(), &bad[..p.count]),
                        Err(Error::Protocol)
                    );
                    assert_eq!(p, before);
                }
            }
        }
        for count in 0..p.count {
            assert_eq!(
                p.validate(p.header(), &p.descriptors()[..count]),
                Err(Error::Protocol)
            );
        }
        let read_descriptors = crate::descriptor_plan(pool(), 65536).unwrap();
        assert_eq!(
            p.validate(p.header(), &read_descriptors),
            Err(Error::Protocol)
        );
    }
}

#[test]
fn completion_requires_single_status_byte_for_write_and_flush() {
    for operation in [Operation::Write { sector: 7 }, Operation::Flush] {
        let p = plan(operation);
        let before = p;
        assert_eq!(
            p.validate_used(10, 10, u32::MAX, u32::MAX, 255),
            Err(Error::Pending)
        );
        for (old, new) in [(0, 1), (17, 18), (u16::MAX, 0)] {
            for (status, expected) in [
                (0, Completion::Success),
                (1, Completion::Io),
                (2, Completion::Unsupported),
            ] {
                assert_eq!(p.validate_used(old, new, 0, 1, status), Ok(expected));
            }
            for length in [0, 2, 128, 512, 513, 514, u32::MAX] {
                for status in [0, 1, 2] {
                    assert_eq!(
                        p.validate_used(old, new, 0, length, status),
                        Err(Error::Protocol)
                    );
                }
            }
            for id in [1, 7, 8, u32::MAX] {
                assert_eq!(p.validate_used(old, new, id, 1, 0), Err(Error::Protocol));
            }
            for status in [3, 255] {
                assert_eq!(
                    p.validate_used(old, new, 0, 1, status),
                    Err(Error::Protocol)
                );
            }
        }
        for (old, new) in [(0, 2), (0, u16::MAX), (u16::MAX, 1)] {
            assert_eq!(p.validate_used(old, new, 0, 1, 0), Err(Error::Protocol));
        }
        assert_eq!(p, before);
    }
}
