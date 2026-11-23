use nieuw_os_network_protocol::{
    ConnectionState, Error, HeaderFields, Operation, Packet, Status, DATA, SIZE,
};
fn header(op: Operation) -> HeaderFields {
    HeaderFields {
        operation: op,
        request: 7,
        epoch: 19,
        connection: 0,
        argument: 0,
        address: [0; 4],
        port: 0,
    }
}
#[test]
fn payload_and_reserved_boundaries_are_checked_before_use() {
    for length in 0..=DATA {
        let mut h = header(Operation::SendTcp);
        h.connection = 257;
        h.argument = length as u32;
        let body = [0x61; DATA];
        let p = Packet::request(h, &body[..length]).unwrap();
        assert_eq!(p.data(), &body[..length]);
        let mut bytes = *p.bytes();
        if length < DATA {
            bytes[48 + length] = 1;
            assert!(matches!(
                Packet::decode_request(&bytes),
                Err(Error::Reserved)
            ));
        }
    }
    let mut h = header(Operation::SendTcp);
    h.argument = DATA as u32 + 1;
    assert!(matches!(
        Packet::request(h, &[0; DATA + 1]),
        Err(Error::Argument)
    ));
    for size in [0, 47, 255, 257] {
        assert!(matches!(
            Packet::decode_request(&[0; SIZE + 1][..size]),
            Err(Error::Length)
        ));
    }
}
#[test]
fn every_reply_is_bound_to_the_full_request() {
    let mut h = header(Operation::OpenTcp);
    h.argument = 300;
    h.address = [192, 0, 2, 1];
    h.port = 8080;
    let mut granted = h;
    granted.connection = 257;
    let p = Packet::response(granted, Status::Ok, ConnectionState::Connecting as u32, &[]);
    assert!(Packet::decode_response(p.bytes(), h).is_ok());
    for at in [6, 8, 16, 40, 44] {
        let mut bytes = *p.bytes();
        bytes[at] ^= 1;
        assert!(Packet::decode_response(&bytes, h).is_err());
    }
    let mut expected = header(Operation::State);
    expected.connection = 257;
    let p = Packet::response(expected, Status::Ok, 2, &[]);
    let mut bytes = *p.bytes();
    bytes[24] ^= 1;
    assert!(matches!(
        Packet::decode_response(&bytes, expected),
        Err(Error::Binding)
    ));
}
#[test]
fn negative_replies_cannot_smuggle_data_or_unknown_state() {
    let h = header(Operation::ReceiveUdp);
    let p = Packet::response(h, Status::Again, 0, &[]);
    assert!(Packet::decode_response(p.bytes(), h).is_ok());
    let mut b = *p.bytes();
    b[48] = 1;
    assert!(Packet::decode_response(&b, h).is_err());
    let h = header(Operation::State);
    let p = Packet::response(h, Status::Ok, 6, &[]);
    assert!(Packet::decode_response(p.bytes(), h).is_err());
    let mut b = *p.bytes();
    b[32..36].copy_from_slice(&100u32.to_le_bytes());
    assert!(Packet::decode_response(&b, h).is_err());
}
#[test]
fn requests_cannot_set_result_status_or_open_with_a_token() {
    let mut h = header(Operation::OpenTcp);
    h.argument = 300;
    h.address = [192, 0, 2, 1];
    h.port = 8080;
    h.connection = 257;
    assert!(Packet::request(h, &[]).is_err());
    h.connection = 0;
    let p = Packet::request(h, &[]).unwrap();
    let mut b = *p.bytes();
    b[32] = 1;
    assert!(Packet::decode_request(&b).is_err());
    b = *p.bytes();
    b[46] = 1;
    assert!(Packet::decode_request(&b).is_err());
    for timeout in [0, 6001, u32::MAX] {
        h.argument = timeout;
        assert!(Packet::request(h, &[]).is_err());
    }
}
#[test]
fn resolve_names_and_address_replies_are_strictly_bounded() {
    use nieuw_os_network_protocol::{MAX_NAME, MAX_RESOLVED};
    let resolve = |name: &[u8]| {
        let mut h = header(Operation::Resolve);
        h.argument = name.len() as u32;
        (h, Packet::request(h, name))
    };
    let (h, p) = resolve(b"Example-1.org.");
    assert_eq!(p.unwrap().data(), b"Example-1.org.");
    let long = [b'a'; MAX_NAME + 1];
    let label = [b"a".repeat(64), b".org".to_vec()].concat();
    let fit = [b"a".repeat(63), b".".to_vec()].concat().repeat(3);
    let fit = [fit, b"d".repeat(MAX_NAME - 192)].concat();
    assert_eq!(fit.len(), MAX_NAME);
    assert!(resolve(&fit).1.is_ok());
    for bad in [
        &b""[..],
        b".",
        b"a..org",
        b"-a.org",
        b"a-.org",
        b"a_b.org",
        b"a b",
        b"a.org\0",
        &label,
    ] {
        assert!(matches!(resolve(bad).1, Err(Error::Argument)), "{bad:?}");
    }
    // Oversize lengths are refused before any body byte is read.
    let mut big = header(Operation::Resolve);
    big.argument = long.len() as u32;
    assert!(Packet::request(big, &long).is_err());
    let mut bytes = *resolve(b"a.org").1.unwrap().bytes();
    bytes[36..40].copy_from_slice(&(DATA as u32 + 1).to_le_bytes());
    assert!(matches!(
        Packet::decode_request(&bytes),
        Err(Error::Argument)
    ));
    // Address, port and connection are reserved: the service picks the server.
    for at in [24, 40, 44] {
        let mut bytes = *resolve(b"a.org").1.unwrap().bytes();
        bytes[at] = 1;
        assert!(matches!(
            Packet::decode_request(&bytes),
            Err(Error::Reserved)
        ));
    }
    let addresses = [7; 4 * MAX_RESOLVED + 4];
    for n in [4, 8, 4 * MAX_RESOLVED] {
        let r = Packet::response(h, Status::Ok, n as u32, &addresses[..n]);
        assert_eq!(
            Packet::decode_response(r.bytes(), h).unwrap().data().len(),
            n
        );
    }
    for n in [0, 3, 5, 4 * MAX_RESOLVED + 4] {
        let r = Packet::response(h, Status::Ok, n as u32, &addresses[..n]);
        assert!(matches!(
            Packet::decode_response(r.bytes(), h),
            Err(Error::Argument)
        ));
    }
    for status in [Status::NotFound, Status::ServerFailure] {
        let r = Packet::response(h, status, 0, &[]);
        assert_eq!(
            Packet::decode_response(r.bytes(), h).unwrap().status(),
            status
        );
    }
}
