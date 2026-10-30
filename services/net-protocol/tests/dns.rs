//! Synthetic RFC wire fixtures; no resolver, host sockets, or VM are used.
use nieuw_os_net_protocol::dns::{
    self, Address, Endpoint, Error, Name, PendingQuery, QueryType, Question,
};
use nieuw_os_net_protocol::{build_udp_ipv4, parse, UdpIpv4Tx};

fn hex(value: &str) -> Vec<u8> {
    (0..value.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&value[at..at + 2], 16).unwrap())
        .collect()
}

fn question() -> Question {
    Question {
        id: 0x1234,
        name: Name::from_ascii("example.com").unwrap(),
        kind: QueryType::A,
    }
}

// Header, one IN/A question, compressed answer owner, 300-second A result.
// These bytes are written independently of the query/frame implementation.
fn fixed_response() -> Vec<u8> {
    hex("123481800001000100000000076578616d706c6503636f6d0000010001c00c000100010000012c0004c0000207")
}

fn message(answers: u16, authority: u16, additional: u16) -> Vec<u8> {
    let mut data = fixed_response()[..29].to_vec();
    data[6..8].copy_from_slice(&answers.to_be_bytes());
    data[8..10].copy_from_slice(&authority.to_be_bytes());
    data[10..12].copy_from_slice(&additional.to_be_bytes());
    data
}

fn record(message: &mut Vec<u8>, owner: &[u8], kind: u16, ttl: u32, data: &[u8]) {
    message.extend_from_slice(owner);
    message.extend_from_slice(&kind.to_be_bytes());
    message.extend_from_slice(&1u16.to_be_bytes());
    message.extend_from_slice(&ttl.to_be_bytes());
    message.extend_from_slice(&(data.len() as u16).to_be_bytes());
    message.extend_from_slice(data);
}

fn resolved(message: &[u8]) -> Vec<Address> {
    dns::parse_response(message, question())
        .unwrap()
        .resolve()
        .unwrap()
        .addresses()
        .copied()
        .collect()
}

#[test]
fn fixed_query_and_response_match_real_wire_shape() {
    let expected = hex("123401000001000000000000076578616d706c6503636f6d0000010001");
    let mut output = [0xa5; 40];
    assert_eq!(dns::build_query(question(), &mut output), Ok(29));
    assert_eq!(&output[..29], expected);
    assert_eq!(&output[29..], &[0xa5; 11]);
    let data = fixed_response();
    let response = dns::parse_response(&data, question()).unwrap();
    let resolution = response.resolve().unwrap();
    assert_eq!(resolution.canonical_name, question().name);
    assert_eq!(resolution.cache_ttl, 300);
    assert_eq!(
        resolution.addresses().copied().collect::<Vec<_>>(),
        vec![Address::Ipv4([192, 0, 2, 7])]
    );
}

#[test]
fn hostname_boundaries_case_and_root_are_explicit() {
    assert_eq!(Name::from_ascii("ExAmPlE.COM.").unwrap(), question().name);
    assert_eq!(Name::from_ascii(".").unwrap().wire(), [0]);
    for invalid in [
        "",
        ".com",
        "example..com",
        "x..",
        "-x.com",
        "x-.com",
        "a_b.com",
        "é.com",
    ] {
        assert!(Name::from_ascii(invalid).is_err(), "{invalid}");
    }
    assert_eq!(Name::from_ascii(&"a".repeat(64)), Err(Error::InvalidName));
    let largest = format!(
        "{}.{}.{}.{}",
        "a".repeat(63),
        "b".repeat(63),
        "c".repeat(63),
        "d".repeat(61)
    );
    let name = Name::from_ascii(&largest).unwrap();
    assert_eq!(name.wire().len(), 255);
    assert_eq!(Name::from_ascii(&(largest + "d")), Err(Error::NameTooLong));
    let mut output = [0xa5; 28];
    assert_eq!(
        dns::build_query(question(), &mut output),
        Err(Error::OutputTooSmall)
    );
    assert_eq!(output, [0xa5; 28]);
}

#[test]
fn every_truncation_is_rejected_without_partial_resolution() {
    let data = fixed_response();
    for cut in 0..data.len() {
        assert!(
            dns::parse_response(&data[..cut], question()).is_err(),
            "cut {cut}"
        );
    }
    let mut data = data;
    data.push(0);
    assert_eq!(
        dns::parse_response(&data, question()),
        Err(Error::TrailingBytes)
    );
    data.resize(dns::MAX_MESSAGE_BYTES + 1, 0);
    assert_eq!(
        dns::parse_response(&data, question()),
        Err(Error::MessageTooLarge)
    );
}

#[test]
fn header_id_question_and_response_errors_are_never_success() {
    let original = fixed_response();
    let mut data = original.clone();
    data[0] ^= 1;
    assert_eq!(dns::parse_response(&data, question()), Err(Error::WrongId));
    for flags in [0x0180u16, 0x8980, 0x81c0] {
        let mut data = original.clone();
        data[2..4].copy_from_slice(&flags.to_be_bytes());
        assert_eq!(
            dns::parse_response(&data, question()),
            Err(Error::InvalidFlags)
        );
    }
    let mut data = original.clone();
    data[2] |= 2;
    assert_eq!(
        dns::parse_response(&data, question()),
        Err(Error::TruncatedResponse)
    );
    for rcode in 1..=15 {
        let mut data = original.clone();
        data[3] |= rcode;
        assert_eq!(
            dns::parse_response(&data, question()),
            Err(Error::ResponseCode(rcode))
        );
    }
    for (at, value) in [(5, 0), (5, 2), (13, b'x'), (26, 28), (28, 3)] {
        let mut data = original.clone();
        data[at] = value;
        assert_eq!(
            dns::parse_response(&data, question()),
            Err(Error::WrongQuestion)
        );
    }
    let mut data = original;
    data[6..8].copy_from_slice(&33u16.to_be_bytes());
    assert_eq!(
        dns::parse_response(&data, question()),
        Err(Error::TooManyRecords)
    );
}

#[test]
fn compression_rejects_header_forward_self_loop_and_bad_label_encodings() {
    for pointer in [0u16, 11, 29, 40, 0x3fff] {
        let mut data = fixed_response();
        data[29..31].copy_from_slice(&(0xc000 | pointer).to_be_bytes());
        assert_eq!(
            dns::parse_response(&data, question()),
            Err(Error::InvalidPointer)
        );
    }
    let mut data = message(1, 0, 0);
    // The owner has a literal label followed by a backward pointer to itself.
    record(&mut data, &[1, b'x', 0xc0, 29], 1, 1, &[192, 0, 2, 1]);
    assert_eq!(
        dns::parse_response(&data, question()),
        Err(Error::CompressionLoop)
    );
    for length in [0x40, 0x80, 0xbf] {
        let mut data = fixed_response();
        data[29] = length;
        assert_eq!(
            dns::parse_response(&data, question()),
            Err(Error::InvalidName)
        );
    }
    let mut data = message(1, 0, 0);
    data.push(0xc0);
    assert_eq!(
        dns::parse_response(&data, question()),
        Err(Error::Truncated)
    );
}

#[test]
fn malformed_address_and_cname_rdlength_cannot_borrow_next_record() {
    for (kind, content) in [
        (1, vec![1, 2, 3]),
        (28, vec![1; 15]),
        (5, vec![0xc0]),
        (5, vec![1, b'x']),
        (5, vec![0, 0]),
    ] {
        let mut data = message(1, 0, 0);
        record(&mut data, &[0xc0, 12], kind, 1, &content);
        assert!(dns::parse_response(&data, question()).is_err());
    }
    let mut data = message(2, 0, 0);
    record(&mut data, &[0xc0, 12], 5, 1, &[1, b'x']);
    record(&mut data, &[0], 1, 1, &[1, 2, 3, 4]);
    assert_eq!(
        dns::parse_response(&data, question()),
        Err(Error::Truncated)
    );
}

#[test]
fn unrelated_and_additional_addresses_never_supply_resolution() {
    let unrelated = Name::from_ascii("evil.test").unwrap();
    let mut data = message(1, 0, 1);
    record(&mut data, unrelated.wire(), 1, 1, &[203, 0, 113, 1]);
    record(&mut data, question().name.wire(), 1, 1, &[192, 0, 2, 2]);
    assert_eq!(
        dns::parse_response(&data, question()).unwrap().resolve(),
        Err(Error::NoAddress)
    );
    data[6..8].copy_from_slice(&2u16.to_be_bytes());
    data[10..12].fill(0);
    assert_eq!(resolved(&data), vec![Address::Ipv4([192, 0, 2, 2])]);
}

#[test]
fn cname_chain_resolves_out_of_order_with_minimum_ttl() {
    let alias = Name::from_ascii("www.example.com").unwrap();
    let mut data = message(3, 0, 0);
    record(&mut data, alias.wire(), 1, 120, &[192, 0, 2, 7]);
    record(&mut data, &[0xc0, 12], 5, 30, alias.wire());
    record(&mut data, alias.wire(), 1, 300, &[192, 0, 2, 8]);
    let response = dns::parse_response(&data, question()).unwrap();
    let resolution = response.resolve().unwrap();
    assert_eq!(resolution.canonical_name, alias);
    assert_eq!(resolution.cache_ttl, 30);
    assert_eq!(resolution.addresses().count(), 2);
}

#[test]
fn cname_cycles_conflicts_and_depth_limit_fail_closed() {
    let alias = Name::from_ascii("other.test").unwrap();
    let mut cycle = message(2, 0, 0);
    record(&mut cycle, &[0xc0, 12], 5, 1, alias.wire());
    record(&mut cycle, alias.wire(), 5, 1, &[0xc0, 12]);
    assert_eq!(
        dns::parse_response(&cycle, question()).unwrap().resolve(),
        Err(Error::CnameLoop)
    );
    let mut conflict = message(2, 0, 0);
    record(&mut conflict, &[0xc0, 12], 5, 1, alias.wire());
    record(&mut conflict, &[0xc0, 12], 1, 1, &[192, 0, 2, 1]);
    assert_eq!(
        dns::parse_response(&conflict, question())
            .unwrap()
            .resolve(),
        Err(Error::ConflictingAnswers)
    );
    let mut conflict = message(2, 0, 0);
    record(&mut conflict, &[0xc0, 12], 5, 1, alias.wire());
    record(
        &mut conflict,
        &[0xc0, 12],
        5,
        1,
        Name::from_ascii("different.test").unwrap().wire(),
    );
    assert_eq!(
        dns::parse_response(&conflict, question())
            .unwrap()
            .resolve(),
        Err(Error::ConflictingAnswers)
    );
    let mut chain = message(9, 0, 0);
    let mut owner = question().name;
    for index in 0..9 {
        let next = Name::from_ascii(&format!("hop{index}.test")).unwrap();
        record(&mut chain, owner.wire(), 5, 1, next.wire());
        owner = next;
    }
    assert_eq!(
        dns::parse_response(&chain, question()).unwrap().resolve(),
        Err(Error::TooManyAliases)
    );
}

#[test]
fn ttl_zero_high_bit_cap_and_duplicate_addresses_are_handled() {
    for (wire, expected) in [
        (0u32, 0),
        (300, 300),
        (0x8000_0000, 0),
        (0x7fff_ffff, dns::MAX_CACHE_TTL),
    ] {
        let mut data = fixed_response();
        data[35..39].copy_from_slice(&wire.to_be_bytes());
        assert_eq!(
            dns::parse_response(&data, question())
                .unwrap()
                .resolve()
                .unwrap()
                .cache_ttl,
            expected
        );
    }
    let mut data = message(2, 0, 0);
    record(&mut data, &[0xc0, 12], 1, 30, &[192, 0, 2, 1]);
    record(&mut data, &[0xc0, 12], 1, 1, &[192, 0, 2, 1]);
    let result = dns::parse_response(&data, question())
        .unwrap()
        .resolve()
        .unwrap();
    assert_eq!(result.addresses().count(), 1);
    assert_eq!(result.cache_ttl, 1);
    let mut data = message(9, 0, 0);
    for index in 1..=9 {
        record(&mut data, &[0xc0, 12], 1, 30, &[192, 0, 2, index]);
    }
    assert_eq!(
        dns::parse_response(&data, question()).unwrap().resolve(),
        Err(Error::TooManyAddresses)
    );
}

#[test]
fn cname_conflict_with_other_address_family_is_rejected() {
    let mut data = message(2, 0, 0);
    let alias = Name::from_ascii("other.test").unwrap();
    record(&mut data, &[0xc0, 12], 5, 1, alias.wire());
    record(&mut data, &[0xc0, 12], 28, 1, &[1; 16]);
    assert_eq!(
        dns::parse_response(&data, question()).unwrap().resolve(),
        Err(Error::ConflictingAnswers)
    );
}

#[test]
fn compressed_cname_target_and_casefolded_names_resolve() {
    let mut data = message(2, 0, 0);
    record(
        &mut data,
        &[0xc0, 12],
        5,
        20,
        &[3, b'W', b'W', b'W', 0xc0, 12],
    );
    let owner = Name::from_ascii("www.example.com").unwrap();
    record(&mut data, owner.wire(), 1, 10, &[192, 0, 2, 9]);
    let result = dns::parse_response(&data, question())
        .unwrap()
        .resolve()
        .unwrap();
    assert_eq!(result.canonical_name, owner);
    assert_eq!(result.cache_ttl, 10);
    assert_eq!(
        result.addresses().copied().collect::<Vec<_>>(),
        vec![Address::Ipv4([192, 0, 2, 9])]
    );
}

#[test]
fn compressed_names_cannot_exceed_wire_name_or_pointer_work_limits() {
    let mut data = message(1, 0, 0);
    let mut name = Vec::new();
    for _ in 0..4 {
        name.push(63);
        name.extend_from_slice(&[b'x'; 63]);
    }
    name.push(0);
    record(&mut data, &name, 1, 1, &[192, 0, 2, 1]);
    assert_eq!(
        dns::parse_response(&data, question()),
        Err(Error::NameTooLong)
    );

    // Earlier opaque RDATA can contain an acyclic chain of backward pointers.
    // Reject one exceeding the explicit work budget before exposing records.
    let mut data = message(2, 0, 0);
    let start = data.len() + 2 + 10;
    let mut chain = vec![0];
    for index in 0..dns::MAX_POINTERS {
        let target = if index == 0 {
            start
        } else {
            start + 1 + (index - 1) * 2
        };
        chain.extend_from_slice(&(0xc000 | target as u16).to_be_bytes());
    }
    record(&mut data, &[0xc0, 12], 16, 1, &chain);
    let last = start + chain.len() - 2;
    record(
        &mut data,
        &(0xc000 | last as u16).to_be_bytes(),
        1,
        1,
        &[192, 0, 2, 1],
    );
    assert_eq!(
        dns::parse_response(&data, question()),
        Err(Error::CompressionLimit)
    );
}

#[test]
fn unsolicited_edns_is_rejected_instead_of_ignoring_extended_rcode() {
    let mut data = fixed_response();
    data[10..12].copy_from_slice(&1u16.to_be_bytes());
    record(&mut data, &[0], 41, 1 << 24, &[]);
    assert_eq!(
        dns::parse_response(&data, question()),
        Err(Error::UnsupportedEdns)
    );
}

#[test]
fn aaaa_records_resolve_only_for_matching_question_type() {
    let mut data = message(1, 0, 0);
    data[26] = 28;
    let address = [0x20, 1, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    record(&mut data, &[0xc0, 12], 28, 30, &address);
    let q = Question {
        kind: QueryType::Aaaa,
        ..question()
    };
    let result = dns::parse_response(&data, q).unwrap().resolve().unwrap();
    assert_eq!(
        result.addresses().copied().collect::<Vec<_>>(),
        vec![Address::Ipv6(address)]
    );
    data[26] = 1;
    assert_eq!(
        dns::parse_response(&data, question()).unwrap().resolve(),
        Err(Error::NoAddress)
    );
}

const LOCAL: Endpoint = Endpoint {
    address: [192, 0, 2, 1],
    port: 49152,
};
const SERVER: Endpoint = Endpoint {
    address: [192, 0, 2, 53],
    port: 53,
};

fn frame(payload: &[u8], from: Endpoint, to: Endpoint) -> Vec<u8> {
    let tx = UdpIpv4Tx {
        destination_mac: [2, 0, 0, 0, 0, 1],
        source_mac: [2, 0, 0, 0, 0, 53],
        source_ipv4: from.address,
        destination_ipv4: to.address,
        source_port: from.port,
        destination_port: to.port,
        identification: 0,
        ttl: 64,
        mtu: 1500,
    };
    let mut bytes = vec![0; 1514];
    let length = build_udp_ipv4(tx, payload, &mut bytes).unwrap();
    bytes.truncate(length);
    bytes
}

#[test]
fn pending_query_binds_both_endpoints_and_id_without_consuming_unrelated_replies() {
    let mut pending = PendingQuery::new(question(), LOCAL, SERVER, 0, 10).unwrap();
    for (from, to) in [
        (Endpoint { port: 54, ..SERVER }, LOCAL),
        (
            Endpoint {
                address: [192, 0, 2, 54],
                ..SERVER
            },
            LOCAL,
        ),
        (
            SERVER,
            Endpoint {
                port: 49153,
                ..LOCAL
            },
        ),
        (
            SERVER,
            Endpoint {
                address: [192, 0, 2, 2],
                ..LOCAL
            },
        ),
    ] {
        let data = frame(&fixed_response(), from, to);
        assert_eq!(
            pending.accept(&parse(&data).unwrap(), 1),
            Err(Error::WrongEndpoint)
        );
    }
    let mut response = fixed_response();
    response[0] ^= 1;
    let bytes = frame(&response, SERVER, LOCAL);
    assert_eq!(
        pending.accept(&parse(&bytes).unwrap(), 2),
        Err(Error::WrongId)
    );
    let bytes = frame(&fixed_response(), SERVER, LOCAL);
    assert!(pending.accept(&parse(&bytes).unwrap(), 9).is_ok());
    assert_eq!(
        pending.accept(&parse(&bytes).unwrap(), 9),
        Err(Error::NotPending)
    );
}

#[test]
fn pending_deadline_cancel_and_checksum_absence_never_accept() {
    assert_eq!(
        PendingQuery::new(question(), LOCAL, SERVER, 0, 0),
        Err(Error::InvalidLifetime)
    );
    assert_eq!(
        PendingQuery::new(question(), LOCAL, SERVER, u64::MAX, 1),
        Err(Error::ClockOverflow)
    );
    assert_eq!(
        PendingQuery::new(question(), Endpoint { port: 0, ..LOCAL }, SERVER, 0, 10),
        Err(Error::InvalidEndpoint)
    );
    let mut bytes = frame(&fixed_response(), SERVER, LOCAL);
    bytes[40..42].fill(0);
    let packet = parse(&bytes).unwrap();
    let mut pending = PendingQuery::new(question(), LOCAL, SERVER, 0, 10).unwrap();
    assert_eq!(pending.accept(&packet, 1), Err(Error::ChecksumRequired));
    assert_eq!(pending.accept(&packet, 10), Err(Error::Timeout));
    assert_eq!(pending.accept(&packet, 10), Err(Error::NotPending));
    let mut pending = PendingQuery::new(question(), LOCAL, SERVER, 0, 10).unwrap();
    pending.cancel();
    assert_eq!(pending.accept(&packet, 1), Err(Error::NotPending));
}

#[test]
fn all_short_arbitrary_inputs_are_bounded_and_do_not_panic() {
    // Deterministic hostile bytes supplement the independent positive fixtures.
    let mut state = 0x9824_5138u32;
    for length in 0..=512 {
        let mut input = vec![0; length];
        for byte in &mut input {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            *byte = state as u8;
        }
        let _ = dns::parse_response(&input, question());
        // Exercise the record parser with a valid question before hostile data.
        if length >= 29 {
            input[..29].copy_from_slice(&message(1, 0, 0));
            let _ = dns::parse_response(&input, question());
        }
    }
}
