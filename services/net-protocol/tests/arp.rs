//! RFC 826-shaped fixed packets and hostile state transitions; no network I/O.
use nieuw_os_net_protocol::arp::{self, Error, Interface, NeighborSlot, Operation};

const LOCAL: Interface = Interface {
    mac: [2, 0, 0, 0, 0, 1],
    ipv4: [192, 0, 2, 1],
};
const PEER: Interface = Interface {
    mac: [2, 0, 0, 0, 0, 2],
    ipv4: [192, 0, 2, 2],
};
const REQUEST: [u8; 42] = [
    255, 255, 255, 255, 255, 255, 2, 0, 0, 0, 0, 1, 8, 6, 0, 1, 8, 0, 6, 4, 0, 1, 2, 0, 0, 0, 0, 1,
    192, 0, 2, 1, 0, 0, 0, 0, 0, 0, 192, 0, 2, 2,
];
const REPLY: [u8; 42] = [
    2, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 2, 8, 6, 0, 1, 8, 0, 6, 4, 0, 2, 2, 0, 0, 0, 0, 2, 192, 0, 2,
    2, 2, 0, 0, 0, 0, 1, 192, 0, 2, 1,
];

#[test]
fn fixed_request_and_reply_encode_without_exposing_buffer_tail() {
    let packet = arp::parse(&REQUEST).unwrap();
    assert_eq!(packet.operation(), Operation::Request);
    assert_eq!(packet.sender_ipv4(), LOCAL.ipv4);
    assert_eq!(packet.target_ipv4(), PEER.ipv4);
    assert_eq!(packet.sender_mac(), LOCAL.mac);
    let mut request = [0xa5; 64];
    assert_eq!(arp::build_request(LOCAL, PEER.ipv4, &mut request), Ok(60));
    assert_eq!(request[..42], REQUEST);
    assert_eq!(request[42..60], [0; 18]);
    assert_eq!(request[60..], [0xa5; 4]);
    let mut reply = [0xa5; 64];
    assert_eq!(arp::build_reply(PEER, &packet, &mut reply), Ok(60));
    assert_eq!(reply[..42], REPLY);
    assert_eq!(reply[42..60], [0; 18]);
    assert_eq!(reply[60..], [0xa5; 4]);
    assert_eq!(
        arp::parse(&reply[..60]).unwrap().operation(),
        Operation::Reply
    );
}

#[test]
fn truncation_padding_types_and_operation_are_bounded() {
    for cut in 0..42 {
        assert_eq!(arp::parse(&REQUEST[..cut]), Err(Error::Truncated));
    }
    for size in 43..=64 {
        let mut data = REQUEST.to_vec();
        data.resize(size, 0xa5);
        if size == 60 {
            assert!(arp::parse(&data).is_ok());
        } else {
            assert_eq!(arp::parse(&data), Err(Error::InvalidLength));
        }
    }
    for at in 12..20 {
        let mut data = REQUEST;
        data[at] ^= 1;
        assert_eq!(arp::parse(&data), Err(Error::UnsupportedFormat));
    }
    for opcode in [0u16, 3, 256, 0xffff] {
        let mut data = REQUEST;
        data[20..22].copy_from_slice(&opcode.to_be_bytes());
        assert_eq!(arp::parse(&data), Err(Error::UnsupportedOperation));
    }
}

#[test]
fn sender_and_target_mac_must_match_link_headers() {
    let mut data = REPLY;
    data[6] ^= 2;
    assert_eq!(arp::parse(&data), Err(Error::SenderMismatch));
    let mut data = REPLY;
    data[32] ^= 2;
    assert_eq!(arp::parse(&data), Err(Error::TargetMismatch));
    for mac in [[0; 6], [255; 6], [1, 0, 0, 0, 0, 1]] {
        let mut data = REQUEST;
        data[6..12].copy_from_slice(&mac);
        data[22..28].copy_from_slice(&mac);
        assert_eq!(arp::parse(&data), Err(Error::InvalidMac));
    }
    let mut data = REPLY;
    data[..6].fill(255);
    data[32..38].fill(255);
    assert_eq!(arp::parse(&data), Err(Error::InvalidMac));
}

#[test]
fn probes_are_visible_but_do_not_become_reply_or_neighbor() {
    let mut probe = REQUEST;
    probe[28..32].fill(0);
    let packet = arp::parse(&probe).unwrap();
    assert!(packet.is_probe());
    let mut output = [0xa5; 60];
    assert_eq!(
        arp::build_reply(PEER, &packet, &mut output),
        Err(Error::ProbeNotSupported)
    );
    assert_eq!(output, [0xa5; 60]);
    for invalid in [[0; 4], [127, 0, 0, 1], [224, 0, 0, 1], [255; 4]] {
        let mut data = REPLY;
        data[28..32].copy_from_slice(&invalid);
        assert_eq!(arp::parse(&data), Err(Error::InvalidIpv4));
    }
}

#[test]
fn all_encoder_errors_preserve_output() {
    let packet = arp::parse(&REQUEST).unwrap();
    let mut output = [0xa5; 59];
    assert_eq!(
        arp::build_request(LOCAL, PEER.ipv4, &mut output),
        Err(Error::OutputTooSmall)
    );
    assert_eq!(
        arp::build_reply(PEER, &packet, &mut output),
        Err(Error::OutputTooSmall)
    );
    assert_eq!(
        arp::build_request(LOCAL, LOCAL.ipv4, &mut output),
        Err(Error::InvalidIpv4)
    );
    assert_eq!(
        arp::build_request(
            Interface {
                mac: [0; 6],
                ..LOCAL
            },
            PEER.ipv4,
            &mut output
        ),
        Err(Error::InvalidMac)
    );
    assert_eq!(
        arp::build_reply(LOCAL, &packet, &mut output),
        Err(Error::TargetMismatch)
    );
    assert_eq!(
        arp::build_reply(LOCAL, &arp::parse(&REPLY).unwrap(), &mut output),
        Err(Error::NotRequest)
    );
    assert_eq!(output, [0xa5; 59]);
}

#[test]
fn cache_only_accepts_current_solicited_reply_with_full_local_target() {
    let mut slot = NeighborSlot::new(LOCAL).unwrap();
    assert_eq!(
        slot.accept_reply(&REPLY, 0, 10),
        Err(Error::NoPendingRequest)
    );
    slot.begin(PEER.ipv4, 0, 5).unwrap();
    assert_eq!(slot.begin(PEER.ipv4, 1, 5), Err(Error::Busy));
    for (from, to) in [(0, 6), (28, 32), (32, 38), (38, 42)] {
        let mut bad = REPLY;
        bad[from..to].fill(3);
        assert!(slot.accept_reply(&bad, 1, 10).is_err());
        assert_eq!(slot.lookup(PEER.ipv4, 1), None);
    }
    assert_eq!(slot.accept_reply(&REPLY, 2, 10), Ok(PEER.mac));
    assert_eq!(slot.lookup(PEER.ipv4, 11), Some(PEER.mac));
    assert_eq!(slot.lookup(PEER.ipv4, 12), None);
    assert_eq!(slot.lookup([192, 0, 2, 3], 3), None);
    assert_eq!(
        slot.accept_reply(&REPLY, 3, 10),
        Err(Error::NoPendingRequest)
    );
}

#[test]
fn deadline_expiration_cancellation_and_clock_overflow_never_install_mapping() {
    let mut slot = NeighborSlot::new(LOCAL).unwrap();
    assert_eq!(slot.begin(PEER.ipv4, 0, 0), Err(Error::InvalidLifetime));
    assert_eq!(
        slot.begin(PEER.ipv4, u64::MAX, 1),
        Err(Error::ClockOverflow)
    );
    slot.begin(PEER.ipv4, 0, 5).unwrap();
    assert_eq!(slot.accept_reply(&REPLY, 5, 10), Err(Error::Timeout));
    assert_eq!(slot.lookup(PEER.ipv4, 5), None);
    slot.begin(PEER.ipv4, 6, 5).unwrap();
    assert_eq!(slot.accept_reply(&REPLY, 7, 0), Err(Error::InvalidLifetime));
    assert_eq!(
        slot.accept_reply(&REPLY, 7, u64::MAX),
        Err(Error::ClockOverflow)
    );
    slot.cancel();
    assert_eq!(
        slot.accept_reply(&REPLY, 7, 10),
        Err(Error::NoPendingRequest)
    );
    // Expired pending slots can be explicitly replaced without a late reply.
    slot.begin(PEER.ipv4, 10, 1).unwrap();
    slot.begin(PEER.ipv4, 11, 1).unwrap();
    assert_eq!(slot.accept_reply(&REPLY, 11, 1), Ok(PEER.mac));
}
