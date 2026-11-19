use nieuw_os_net_protocol::dns::{self, Address, Name, QueryType, Question};
use nieuw_os_net_protocol::resolver::{Error, Resolver, CACHE_CAPACITY, MAX_NEGATIVE_TTL};

fn question(name: &str, kind: QueryType) -> Question {
    Question {
        id: 0x1234,
        name: Name::from_ascii(name).unwrap(),
        kind,
    }
}

fn query() -> Question {
    question("example.com", QueryType::A)
}

// Fixtures encode the DNS wire format independently of the library encoder.
fn message(q: Question, rcode: u8, answers: u16, authority: u16) -> Vec<u8> {
    let mut bytes = vec![0x12, 0x34, 0x81, 0x80 | rcode, 0, 1];
    bytes.extend_from_slice(&answers.to_be_bytes());
    bytes.extend_from_slice(&authority.to_be_bytes());
    bytes.extend_from_slice(&[0, 0]);
    bytes.extend_from_slice(q.name.wire());
    bytes.extend_from_slice(&(q.kind as u16).to_be_bytes());
    bytes.extend_from_slice(&[0, 1]);
    bytes
}

fn record(bytes: &mut Vec<u8>, owner: &[u8], kind: u16, ttl: u32, data: &[u8]) {
    bytes.extend_from_slice(owner);
    bytes.extend_from_slice(&kind.to_be_bytes());
    bytes.extend_from_slice(&[0, 1]);
    bytes.extend_from_slice(&ttl.to_be_bytes());
    bytes.extend_from_slice(&(data.len() as u16).to_be_bytes());
    bytes.extend_from_slice(data);
}

fn positive(q: Question, ttl: u32) -> Vec<u8> {
    let mut bytes = message(q, 0, 1, 0);
    match q.kind {
        QueryType::A => record(&mut bytes, &[0xc0, 12], 1, ttl, &[192, 0, 2, 7]),
        QueryType::Aaaa => record(&mut bytes, &[0xc0, 12], 28, ttl, &[0x20; 16]),
    }
    bytes
}

fn soa(bytes: &mut Vec<u8>, zone: &str, ttl: u32, minimum: u32) {
    let mut data = vec![0xc0, 12, 0xc0, 12];
    for value in [1u32, 3600, 600, 86400, minimum] {
        data.extend_from_slice(&value.to_be_bytes());
    }
    record(bytes, Name::from_ascii(zone).unwrap().wire(), 6, ttl, &data);
}

fn negative(ttl: u32, minimum: u32) -> Vec<u8> {
    let mut bytes = message(query(), 3, 0, 1);
    soa(&mut bytes, "com", ttl, minimum);
    bytes
}

fn lookup(resolver: &mut Resolver, now: u64) -> Option<Result<dns::Resolution, Error>> {
    resolver.lookup(query().name, query().kind, 1, now)
}

#[test]
fn positive_exact_expiry_remaining_ttl_and_canonical_keys() {
    let mut resolver = Resolver::new(1, 10).unwrap();
    let result = resolver
        .accept(&positive(query(), 2), query(), 1, 5)
        .unwrap();
    assert_eq!(
        result.addresses().copied().collect::<Vec<_>>(),
        vec![Address::Ipv4([192, 0, 2, 7])]
    );
    assert_eq!(lookup(&mut resolver, 15).unwrap().unwrap().cache_ttl, 1);
    assert!(resolver
        .lookup(
            Name::from_ascii("EXAMPLE.COM.").unwrap(),
            QueryType::A,
            1,
            24
        )
        .is_some());
    assert!(lookup(&mut resolver, 25).is_none());
    assert!(lookup(&mut resolver, 26).is_none());
}

#[test]
fn zero_and_high_bit_ttls_are_not_cached() {
    for ttl in [0, 0x8000_0000, u32::MAX] {
        let mut resolver = Resolver::new(1, 1).unwrap();
        assert_eq!(
            resolver
                .accept(&positive(query(), ttl), query(), 1, 0)
                .unwrap()
                .cache_ttl,
            0
        );
        assert!(lookup(&mut resolver, 0).is_none());
    }
    assert_eq!(
        Resolver::new(1, 0).err(),
        Some(Error::Dns(dns::Error::InvalidLifetime))
    );
}

#[test]
fn negative_uses_soa_minimum_ttl_and_local_cap_with_exact_expiry() {
    for (ttl, minimum, duration) in [(100, 3, 3), (2, 100, 2), (86400, 86400, MAX_NEGATIVE_TTL)] {
        let mut resolver = Resolver::new(1, 2).unwrap();
        assert_eq!(
            resolver.accept(&negative(ttl, minimum), query(), 1, 7),
            Err(Error::NxDomain)
        );
        assert_eq!(
            lookup(&mut resolver, 7 + 2 * u64::from(duration) - 1),
            Some(Err(Error::NxDomain))
        );
        assert!(lookup(&mut resolver, 7 + 2 * u64::from(duration)).is_none());
    }
}

#[test]
fn absent_zero_or_high_bit_soa_never_fabricates_a_lifetime() {
    let mut messages = vec![message(query(), 3, 0, 0)];
    for (ttl, minimum) in [(0, 50), (50, 0), (u32::MAX, 50), (50, u32::MAX)] {
        messages.push(negative(ttl, minimum));
    }
    for bytes in messages {
        let mut resolver = Resolver::new(1, 1).unwrap();
        assert_eq!(resolver.accept(&bytes, query(), 1, 0), Err(Error::NxDomain));
        assert!(lookup(&mut resolver, 0).is_none());
    }
}

#[test]
fn question_type_is_part_of_key_and_aaaa_is_cached() {
    let mut resolver = Resolver::new(1, 1).unwrap();
    resolver
        .accept(&negative(50, 50), query(), 1, 0)
        .unwrap_err();
    let q = question("example.com", QueryType::Aaaa);
    assert!(resolver.lookup(q.name, q.kind, 1, 0).is_none());
    let result = resolver.accept(&positive(q, 10), q, 1, 0).unwrap();
    assert_eq!(
        result.addresses().copied().collect::<Vec<_>>(),
        vec![Address::Ipv6([0x20; 16])]
    );
    assert!(resolver.lookup(q.name, q.kind, 1, 9).unwrap().is_ok());
    assert_eq!(lookup(&mut resolver, 9), Some(Err(Error::NxDomain)));
}

#[test]
fn epoch_flushes_even_when_new_reply_is_invalid() {
    let mut resolver = Resolver::new(1, 1).unwrap();
    resolver
        .accept(&positive(query(), 50), query(), 1, 0)
        .unwrap();
    assert!(resolver.lookup(query().name, query().kind, 2, 1).is_none());
    resolver
        .accept(&negative(50, 50), query(), 2, 1)
        .unwrap_err();
    assert!(resolver.accept(&[], query(), 3, 2).is_err());
    assert!(resolver.lookup(query().name, query().kind, 3, 2).is_none());
}

#[test]
fn full_cache_evicts_deterministically_and_updates_do_not_evict() {
    let mut resolver = Resolver::new(1, 1).unwrap();
    for index in 0..CACHE_CAPACITY {
        let q = question(&format!("n{index}.com"), QueryType::A);
        resolver.accept(&positive(q, 50), q, 1, 0).unwrap();
    }
    let first = question("n0.com", QueryType::A);
    resolver.accept(&positive(first, 100), first, 1, 1).unwrap();
    resolver
        .accept(&positive(query(), 50), query(), 1, 1)
        .unwrap();
    assert!(resolver.lookup(first.name, first.kind, 1, 1).is_none());
    for index in 1..CACHE_CAPACITY {
        let q = question(&format!("n{index}.com"), QueryType::A);
        assert!(resolver.lookup(q.name, q.kind, 1, 1).is_some());
    }
    let q = question("replacement.com", QueryType::A);
    resolver.accept(&positive(q, 50), q, 1, 1).unwrap();
    assert!(resolver
        .lookup(Name::from_ascii("n1.com").unwrap(), QueryType::A, 1, 1)
        .is_none());
}

#[test]
fn clock_multiplication_and_addition_overflow_leave_cache_unchanged() {
    for (rate, now, ttl) in [(u64::MAX, 0, 2), (1, u64::MAX, 1)] {
        let mut resolver = Resolver::new(1, rate).unwrap();
        resolver
            .accept(&positive(query(), 1), query(), 1, 0)
            .unwrap();
        assert_eq!(
            resolver.accept(&positive(query(), ttl), query(), 1, now),
            Err(Error::Dns(dns::Error::ClockOverflow))
        );
        assert!(lookup(&mut resolver, 0).unwrap().is_ok());
        assert_eq!(
            resolver.accept(&negative(ttl, ttl), query(), 1, now),
            Err(Error::Dns(dns::Error::ClockOverflow))
        );
        assert!(lookup(&mut resolver, 0).unwrap().is_ok());
    }
}

#[test]
fn errors_are_explicit_and_never_replace_valid_entries() {
    for negative_reply in [false, true] {
        let original = if negative_reply {
            negative(50, 50)
        } else {
            positive(query(), 50)
        };
        for (offset, value, error) in [
            (0, 0x99, Error::Dns(dns::Error::WrongId)),
            (13, b'z', Error::Dns(dns::Error::WrongQuestion)),
            (2, 0x83, Error::Dns(dns::Error::TruncatedResponse)),
            (3, 0x82, Error::ServFail),
            (28, 2, Error::Dns(dns::Error::WrongQuestion)),
            (26, 28, Error::Dns(dns::Error::WrongQuestion)),
        ] {
            let mut resolver = Resolver::new(1, 1).unwrap();
            resolver
                .accept(&positive(query(), 50), query(), 1, 0)
                .unwrap();
            let mut bytes = original.clone();
            bytes[offset] = value;
            assert_eq!(resolver.accept(&bytes, query(), 1, 1), Err(error));
            assert_eq!(lookup(&mut resolver, 1).unwrap().unwrap().cache_ttl, 49);
        }
    }
}

#[test]
fn every_negative_truncation_and_trailer_cannot_poison_cache() {
    let bytes = negative(100, 100);
    let mut resolver = Resolver::new(1, 1).unwrap();
    for end in 0..bytes.len() {
        assert!(matches!(
            resolver.accept(&bytes[..end], query(), 1, 0),
            Err(Error::Dns(_))
        ));
        assert!(lookup(&mut resolver, 0).is_none());
    }
    let mut bytes = bytes;
    bytes.push(0);
    assert_eq!(
        resolver.accept(&bytes, query(), 1, 0),
        Err(Error::Dns(dns::Error::TrailingBytes))
    );
    assert!(lookup(&mut resolver, 0).is_none());
}

#[test]
fn malformed_negative_soa_names_fields_and_authority_are_rejected() {
    let mut cases = Vec::new();
    // Backward pointer returns to the label that precedes it, making a loop.
    let mut cycle = message(query(), 3, 0, 1);
    record(&mut cycle, &[1, b'x', 0xc0, 29], 6, 50, &[0; 22]);
    assert_eq!(
        Resolver::new(1, 1).unwrap().accept(&cycle, query(), 1, 0),
        Err(Error::Dns(dns::Error::CompressionLoop))
    );
    cases.push(cycle);
    for data in [vec![0xc0, 29, 0], vec![0, 0], vec![0; 23]] {
        let mut bytes = message(query(), 3, 0, 1);
        record(&mut bytes, &[0xc0, 12], 6, 50, &data);
        cases.push(bytes);
    }
    let mut unrelated = message(query(), 3, 0, 1);
    soa(&mut unrelated, "other.com", 50, 50);
    cases.push(unrelated);
    let mut duplicate = message(query(), 3, 0, 2);
    soa(&mut duplicate, "com", 50, 50);
    soa(&mut duplicate, "com", 50, 50);
    cases.push(duplicate);
    let mut unknown = message(query(), 3, 0, 1);
    record(&mut unknown, &[0xc0, 12], 99, 50, &[0]);
    cases.push(unknown);
    let mut ns_loop = message(query(), 3, 0, 1);
    record(&mut ns_loop, &[0xc0, 12], 2, 50, &[1, b'x', 0xc0, 41]);
    cases.push(ns_loop);
    for bytes in cases {
        let mut resolver = Resolver::new(1, 1).unwrap();
        assert!(matches!(
            resolver.accept(&bytes, query(), 1, 0),
            Err(Error::Dns(_))
        ));
        assert!(lookup(&mut resolver, 0).is_none());
    }
}

#[test]
fn valid_soa_cannot_hide_malformed_additional_records_or_excess_counts() {
    let mut bytes = negative(50, 50);
    bytes[11] = 1;
    record(&mut bytes, &[0xc0, 12], 6, 50, &[0; 21]);
    let mut resolver = Resolver::new(1, 1).unwrap();
    resolver
        .accept(&positive(query(), 50), query(), 1, 0)
        .unwrap();
    assert_eq!(
        resolver.accept(&bytes, query(), 1, 1),
        Err(Error::Dns(dns::Error::InvalidRecord))
    );
    assert!(lookup(&mut resolver, 1).unwrap().is_ok());
    for rcode in [0, 3] {
        let bytes = message(query(), rcode, dns::MAX_RECORDS as u16 + 1, 0);
        assert_eq!(
            resolver.accept(&bytes, query(), 1, 1),
            Err(Error::Dns(dns::Error::TooManyRecords))
        );
        assert!(lookup(&mut resolver, 1).unwrap().is_ok());
    }
}

#[test]
fn only_authority_soa_supplies_lifetime_and_ns_and_root_zone_are_supported() {
    let mut bytes = message(query(), 3, 0, 2);
    record(&mut bytes, &[0xc0, 20], 2, 50, &[0xc0, 12]);
    soa(&mut bytes, ".", 50, 50);
    let mut resolver = Resolver::new(1, 1).unwrap();
    assert_eq!(resolver.accept(&bytes, query(), 1, 0), Err(Error::NxDomain));
    assert_eq!(lookup(&mut resolver, 1), Some(Err(Error::NxDomain)));
    for section in [6, 10] {
        let mut bytes = negative(50, 50);
        bytes[9] = 0;
        bytes[section + 1] = 1;
        let mut resolver = Resolver::new(1, 1).unwrap();
        assert_eq!(resolver.accept(&bytes, query(), 1, 0), Err(Error::NxDomain));
        assert!(lookup(&mut resolver, 0).is_none());
    }
}

#[test]
fn valid_zero_lifetime_replaces_old_positive_or_negative_entries() {
    for replacement in [positive(query(), 0), negative(0, 50)] {
        let mut resolver = Resolver::new(1, 1).unwrap();
        resolver
            .accept(&positive(query(), 50), query(), 1, 0)
            .unwrap();
        let _ = resolver.accept(&replacement, query(), 1, 1);
        assert!(lookup(&mut resolver, 1).is_none());
    }
}

#[test]
fn oversize_wire_name_is_rejected_without_cache_mutation() {
    for rcode in [0, 3] {
        let mut bytes = message(query(), rcode, 1, 0);
        let mut owner = Vec::new();
        for _ in 0..4 {
            owner.push(63);
            owner.extend_from_slice(&[b'x'; 63]);
        }
        owner.push(0);
        record(&mut bytes, &owner, 1, 50, &[192, 0, 2, 7]);
        let mut resolver = Resolver::new(1, 1).unwrap();
        assert_eq!(
            resolver.accept(&bytes, query(), 1, 0),
            Err(Error::Dns(dns::Error::NameTooLong))
        );
        assert!(lookup(&mut resolver, 0).is_none());
    }
}

#[test]
fn cname_chain_ttl_depth_and_cycles_apply_to_positive_and_negative_replies() {
    for rcode in [0, 3] {
        let mut bytes = message(
            query(),
            rcode,
            1 + u16::from(rcode == 0),
            u16::from(rcode == 3),
        );
        let target = Name::from_ascii("alias.com").unwrap();
        record(&mut bytes, &[0xc0, 12], 5, 2, target.wire());
        if rcode == 0 {
            record(&mut bytes, target.wire(), 1, 50, &[192, 0, 2, 7]);
        } else {
            soa(&mut bytes, "com", 50, 50);
        }
        let mut resolver = Resolver::new(1, 1).unwrap();
        let result = resolver.accept(&bytes, query(), 1, 0);
        if rcode == 0 {
            assert_eq!(result.unwrap().cache_ttl, 2);
        } else {
            assert_eq!(result, Err(Error::NxDomain));
        }
        assert!(lookup(&mut resolver, 1).is_some());
        assert!(lookup(&mut resolver, 2).is_none());

        for hops in [dns::MAX_CNAME_HOPS, dns::MAX_CNAME_HOPS + 1] {
            let mut bytes = message(
                query(),
                rcode,
                (hops + usize::from(rcode == 0)) as u16,
                u16::from(rcode == 3),
            );
            let mut owner = query().name;
            for index in 0..hops {
                let target = Name::from_ascii(&format!("alias{index}.com")).unwrap();
                record(&mut bytes, owner.wire(), 5, 50, target.wire());
                owner = target;
            }
            if rcode == 0 {
                record(&mut bytes, owner.wire(), 1, 50, &[192, 0, 2, 7]);
            } else {
                soa(&mut bytes, "com", 50, 50);
            }
            let mut resolver = Resolver::new(1, 1).unwrap();
            let result = resolver.accept(&bytes, query(), 1, 0);
            if hops > dns::MAX_CNAME_HOPS {
                assert_eq!(result, Err(Error::Dns(dns::Error::TooManyAliases)));
                assert!(lookup(&mut resolver, 0).is_none());
            } else {
                assert!(lookup(&mut resolver, 0).is_some());
            }
        }
        let mut bytes = message(query(), rcode, 2, 0);
        record(&mut bytes, &[0xc0, 12], 5, 50, target.wire());
        record(&mut bytes, target.wire(), 5, 50, query().name.wire());
        let mut resolver = Resolver::new(1, 1).unwrap();
        assert_eq!(
            resolver.accept(&bytes, query(), 1, 0),
            Err(Error::Dns(dns::Error::CnameLoop))
        );
        assert!(lookup(&mut resolver, 0).is_none());
    }
}

#[test]
fn nxdomain_with_terminal_addresses_is_not_cached_for_either_type() {
    for kind in [QueryType::A, QueryType::Aaaa] {
        let mut bytes = message(query(), 3, 1, 1);
        record(
            &mut bytes,
            &[0xc0, 12],
            kind as u16,
            50,
            if kind == QueryType::A {
                &[192, 0, 2, 7]
            } else {
                &[0x20; 16]
            },
        );
        soa(&mut bytes, "com", 50, 50);
        let mut resolver = Resolver::new(1, 1).unwrap();
        assert_eq!(
            resolver.accept(&bytes, query(), 1, 0),
            Err(Error::Dns(dns::Error::ConflictingAnswers))
        );
        assert!(lookup(&mut resolver, 0).is_none());
    }
}
