//! Packet fixtures and monotonic seconds are host models, not native evidence.
use nieuw_os_net_config::dhcp_client::{Client, Error, Phase};
use nieuw_os_net_protocol::{UdpIpv4Tx, build_udp_ipv4, dhcp, parse};

const MAC: [u8; 6] = [2, 0, 0, 0, 0, 7];
const SERVER: [u8; 4] = [192, 0, 2, 1];
const ADDRESS: [u8; 4] = [192, 0, 2, 42];
const XID: u32 = 0x1234_5678;

fn message(kind: u8, xid: u32, server: [u8; 4]) -> Vec<u8> {
    let mut bytes = vec![0; 240];
    bytes[..3].copy_from_slice(&[2, 1, 6]);
    bytes[4..8].copy_from_slice(&xid.to_be_bytes());
    bytes[16..20].copy_from_slice(&ADDRESS);
    bytes[28..34].copy_from_slice(&MAC);
    bytes[236..240].copy_from_slice(&[99, 130, 83, 99]);
    bytes.extend_from_slice(&[53, 1, kind, 54, 4]);
    bytes.extend_from_slice(&server);
    if kind != 6 {
        bytes.extend_from_slice(&[51, 4, 0, 0, 0, 80, 1, 4, 255, 255, 255, 0]);
        bytes.extend_from_slice(&[3, 4, 192, 0, 2, 1, 6, 4, 192, 0, 2, 53]);
    }
    bytes.push(255);
    bytes
}

fn frame(message: &[u8], source: [u8; 4], destination: [u8; 4]) -> Vec<u8> {
    let mut bytes = vec![0; 1514];
    let length = build_udp_ipv4(
        UdpIpv4Tx {
            source_mac: [2, 0, 0, 0, 0, 1],
            destination_mac: MAC,
            source_ipv4: source,
            destination_ipv4: destination,
            source_port: 67,
            destination_port: 68,
            identification: 1,
            ttl: 64,
            mtu: 1500,
        },
        message,
        &mut bytes,
    )
    .unwrap();
    bytes.truncate(length);
    bytes
}

fn accept(
    client: &mut Client,
    kind: u8,
    xid: u32,
    server: [u8; 4],
    now: u64,
) -> Result<Phase, Error> {
    let bytes = frame(&message(kind, xid, server), server, [255; 4]);
    client.accept(&parse(&bytes).unwrap(), now)
}

fn selecting(start: u64) -> Client {
    let mut client = Client::new(MAC, start).unwrap();
    client.start(XID, start, 120).unwrap();
    let mut out = [0; 576];
    let tx = client.build(&mut out, start).unwrap();
    assert_eq!(tx.source, [0; 4]);
    assert_eq!(tx.destination, [255; 4]);
    assert_eq!(out[242], 1);
    client
}

fn requesting(start: u64) -> Client {
    let mut client = selecting(start);
    assert_eq!(
        accept(&mut client, 2, XID, SERVER, start + 1),
        Ok(Phase::Requesting)
    );
    let mut out = [0; 576];
    client.build(&mut out, start + 2).unwrap();
    assert_eq!(out[242], 3);
    client
}

fn bound(start: u64) -> Client {
    let mut client = requesting(start);
    assert_eq!(
        accept(&mut client, 5, XID, SERVER, start + 3),
        Ok(Phase::CheckingAddress)
    );
    assert_eq!(client.lease(start + 3), Ok(None));
    client.confirm_address_available(start + 4).unwrap();
    client
}

#[test]
fn ack_is_not_configuration_until_conflict_detection_and_lease_uses_request_time() {
    let mut client = requesting(100);
    assert_eq!(
        client.confirm_address_available(102),
        Err(Error::InvalidState)
    );
    assert_eq!(
        accept(&mut client, 5, XID, SERVER, 110),
        Ok(Phase::CheckingAddress)
    );
    assert_eq!(client.lease(110), Ok(None));
    let candidate = client.candidate(110).unwrap().unwrap();
    assert_eq!(candidate.address, ADDRESS);
    assert_eq!(candidate.prefix_len, 24);
    assert_eq!(
        (
            candidate.renew_at,
            candidate.rebind_at,
            candidate.expires_at
        ),
        (142, 172, 182)
    );
    assert_eq!(candidate.routers(), &[SERVER]);
    assert_eq!(candidate.dns_servers(), &[[192, 0, 2, 53]]);
    assert_eq!(client.confirm_address_available(111), Ok(Phase::Bound));
    assert_eq!(client.candidate(111), Ok(None));
    assert_eq!(client.lease(111).unwrap(), Some(candidate));
}

#[test]
fn wrong_endpoint_xid_server_and_message_do_not_consume_exchange() {
    let mut client = requesting(0);
    assert_eq!(
        accept(&mut client, 2, XID, SERVER, 3),
        Err(Error::WrongMessage)
    );
    assert_eq!(
        accept(&mut client, 5, XID ^ 1, SERVER, 3),
        Err(Error::Codec(dhcp::Error::WrongTransaction))
    );
    assert_eq!(
        accept(&mut client, 5, XID, [192, 0, 2, 9], 3),
        Err(Error::WrongServer)
    );
    let valid = message(5, XID, SERVER);
    for (offset, port) in [(34, 68u16), (36, 67u16)] {
        let mut bytes = frame(&valid, SERVER, [255; 4]);
        bytes[offset..offset + 2].copy_from_slice(&port.to_be_bytes());
        bytes[40..42].fill(0);
        assert_eq!(
            client.accept(&parse(&bytes).unwrap(), 3),
            Err(Error::WrongEndpoint)
        );
    }
    for (source, destination) in [([192, 0, 2, 9], [255; 4]), (SERVER, [192, 0, 2, 99])] {
        let bytes = frame(&valid, source, destination);
        assert!(client.accept(&parse(&bytes).unwrap(), 3).is_err());
    }
    let mut bad = frame(&valid, SERVER, ADDRESS);
    bad[..6].copy_from_slice(&[2, 0, 0, 0, 0, 9]);
    assert_eq!(
        client.accept(&parse(&bad).unwrap(), 3),
        Err(Error::WrongEndpoint)
    );
    let mut relay = valid.clone();
    relay[24..28].copy_from_slice(&[192, 0, 2, 9]);
    let bytes = frame(&relay, SERVER, [255; 4]);
    assert_eq!(
        client.accept(&parse(&bytes).unwrap(), 3),
        Err(Error::RelayUnsupported)
    );
    assert_eq!(
        accept(&mut client, 5, XID, SERVER, 3),
        Ok(Phase::CheckingAddress)
    );
}

#[test]
fn renewal_is_unicast_and_rebinding_is_broadcast_without_selected_options() {
    let mut client = bound(0);
    assert_eq!(client.begin_renewal(9, 41), Err(Error::InvalidState));
    assert_eq!(client.tick(42), Ok(Phase::RenewalDue));
    client.begin_renewal(9, 42).unwrap();
    let mut out = [0; 576];
    let tx = client.build(&mut out, 42).unwrap();
    assert_eq!((tx.source, tx.destination), (ADDRESS, SERVER));
    assert_eq!(&out[12..16], &ADDRESS);
    assert_eq!(&out[10..12], &[0, 0]);
    assert_eq!(out[242], 3);
    assert_eq!(&out[4..8], &9u32.to_be_bytes());
    // The entire option layout is known: 53, 61, 55, 57, END; no 50 or 54.
    assert_eq!(
        (out[240], out[243], out[252], out[261], out[265]),
        (53, 61, 55, 57, 255)
    );
    assert_eq!(client.tick(72), Ok(Phase::Rebinding));
    let tx = client.build(&mut out, 72).unwrap();
    assert_eq!((tx.source, tx.destination), (ADDRESS, [255; 4]));
    assert_eq!(client.lease(81).unwrap().unwrap().address, ADDRESS);
    assert_eq!(client.lease(82), Ok(None));
    assert_eq!(client.tick(82), Ok(Phase::Expired));
    assert_eq!(
        accept(&mut client, 5, 9, SERVER, 82),
        Err(Error::InvalidState)
    );
}

#[test]
fn renewal_reply_is_bound_to_fresh_xid_and_server_but_rebinding_can_change_server() {
    let mut client = bound(0);
    client.begin_renewal(99, 42).unwrap();
    let mut out = [0; 576];
    client.build(&mut out, 42).unwrap();
    assert_eq!(
        accept(&mut client, 5, XID, SERVER, 43),
        Err(Error::Codec(dhcp::Error::WrongTransaction))
    );
    assert_eq!(
        accept(&mut client, 5, 99, [192, 0, 2, 9], 43),
        Err(Error::WrongServer)
    );
    assert_eq!(accept(&mut client, 5, 99, SERVER, 44), Ok(Phase::Bound));
    let lease = client.lease(44).unwrap().unwrap();
    assert_eq!(
        (lease.renew_at, lease.rebind_at, lease.expires_at),
        (82, 112, 122)
    );
    client.begin_renewal(100, 113).unwrap();
    let tx = client.build(&mut out, 113).unwrap();
    assert_eq!(tx.destination, [255; 4]);
    assert_eq!(
        accept(&mut client, 5, 100, [192, 0, 2, 9], 114),
        Ok(Phase::Bound)
    );
    assert_eq!(client.lease(114).unwrap().unwrap().server, [192, 0, 2, 9]);
}

#[test]
fn unsolicited_ack_nak_and_unsent_messages_never_configure() {
    let mut client = Client::new(MAC, 0).unwrap();
    assert_eq!(
        accept(&mut client, 5, XID, SERVER, 0),
        Err(Error::InvalidState)
    );
    client.start(XID, 0, 120).unwrap();
    assert_eq!(
        accept(&mut client, 2, XID, SERVER, 0),
        Err(Error::InvalidState)
    );
    let mut out = [0; 576];
    client.build(&mut out, 0).unwrap();
    assert_eq!(
        accept(&mut client, 5, XID, SERVER, 0),
        Err(Error::WrongMessage)
    );
    accept(&mut client, 2, XID, SERVER, 0).unwrap();
    assert_eq!(
        accept(&mut client, 5, XID, SERVER, 0),
        Err(Error::InvalidState)
    );
    client.build(&mut out, 0).unwrap();
    assert_eq!(
        accept(&mut client, 6, XID, SERVER, 0),
        Ok(Phase::Invalidated)
    );
    assert_eq!(client.lease(0), Ok(None));
}

#[test]
fn nack_link_loss_conflict_and_expiry_remove_usable_configuration() {
    let mut client = bound(0);
    let mut out = [0; 576];
    client.begin_renewal(9, 42).unwrap();
    client.build(&mut out, 42).unwrap();
    assert_eq!(
        accept(&mut client, 6, 9, SERVER, 43),
        Ok(Phase::Invalidated)
    );
    assert_eq!(client.lease(43), Ok(None));
    client.start(10, 43, 120).unwrap();
    client.link_down(43).unwrap();
    assert_eq!(client.tick(43), Ok(Phase::Idle));
    assert_eq!(
        accept(&mut client, 5, 10, SERVER, 43),
        Err(Error::InvalidState)
    );
    let mut client = bound(0);
    client.link_down(5).unwrap();
    assert_eq!(client.lease(5), Ok(None));
    let mut client = requesting(0);
    accept(&mut client, 5, XID, SERVER, 3).unwrap();
    client.reject_address_conflict(3).unwrap();
    assert_eq!(client.lease(3), Ok(None));
    assert_eq!(
        client.confirm_address_available(3),
        Err(Error::InvalidState)
    );
    let mut client = requesting(0);
    accept(&mut client, 5, XID, SERVER, 3).unwrap();
    assert_eq!(
        client.confirm_address_available(82),
        Err(Error::InvalidState)
    );
    assert_eq!(client.candidate(82), Ok(None));
}

#[test]
fn clock_rollback_overflow_timeouts_and_failed_builds_are_explicit() {
    assert_eq!(Client::new([0; 6], 0).unwrap_err(), Error::InvalidMac);
    let mut client = Client::new(MAC, 10).unwrap();
    assert_eq!(client.start(XID, 9, 120), Err(Error::ClockWentBackwards));
    assert_eq!(client.start(XID, 10, 0), Err(Error::InvalidTimeout));
    assert_eq!(client.start(XID, 10, 121), Err(Error::InvalidTimeout));
    assert_eq!(client.start(XID, u64::MAX, 1), Err(Error::ClockOverflow));
    client.start(XID, 10, 5).unwrap();
    let mut out = [0xaa; 10];
    assert_eq!(
        client.build(&mut out, 10),
        Err(Error::Codec(dhcp::Error::OutputTooSmall))
    );
    assert_eq!(out, [0xaa; 10]);
    assert_eq!(
        accept(&mut client, 2, XID, SERVER, 11),
        Err(Error::InvalidState)
    );
    assert_eq!(client.tick(9), Err(Error::ClockWentBackwards));
    assert_eq!(client.tick(15), Ok(Phase::TimedOut));
    assert_eq!(client.lease(15), Ok(None));
}

#[test]
fn retransmissions_cannot_extend_the_lease_or_request_timeout() {
    let mut client = requesting(0);
    let mut out = [0; 576];
    client.build(&mut out, 60).unwrap();
    assert_eq!(
        accept(&mut client, 5, XID, SERVER, 81),
        Ok(Phase::CheckingAddress)
    );
    assert_eq!(
        client.confirm_address_available(82),
        Err(Error::InvalidState)
    );
    assert_eq!(client.lease(82), Ok(None));
    let mut client = requesting(0);
    client.build(&mut out, 119).unwrap();
    assert_eq!(
        accept(&mut client, 5, XID, SERVER, 120),
        Err(Error::InvalidState)
    );
    assert_eq!(client.tick(120), Ok(Phase::TimedOut));
}

#[test]
fn invalid_configuration_never_replaces_selected_offer_or_active_lease() {
    let mut client = bound(0);
    let previous = client.lease(4).unwrap().unwrap();
    client.begin_renewal(9, 42).unwrap();
    client.build(&mut [0; 576], 42).unwrap();
    for (offset, value) in [(16, 0), (16, 127), (19, 255), (19, 0), (257, 0)] {
        let mut bytes = message(5, 9, SERVER);
        bytes[offset] = value;
        let bytes = frame(&bytes, SERVER, [255; 4]);
        assert!(client.accept(&parse(&bytes).unwrap(), 43).is_err());
        assert_eq!(client.lease(43).unwrap(), Some(previous));
    }
    assert_eq!(accept(&mut client, 5, 9, SERVER, 44), Ok(Phase::Bound));
}

#[test]
fn clock_overflow_and_infinite_lease_do_not_publish_a_candidate() {
    let start = u64::MAX - 150;
    let mut client = requesting(start);
    let mut bytes = message(5, XID, SERVER);
    bytes[251..255].copy_from_slice(&200u32.to_be_bytes());
    let bytes = frame(&bytes, SERVER, [255; 4]);
    assert_eq!(
        client.accept(&parse(&bytes).unwrap(), start + 3),
        Err(Error::ClockOverflow)
    );
    assert_eq!(client.candidate(start + 3), Ok(None));
    let mut bytes = message(5, XID, SERVER);
    bytes[251..255].copy_from_slice(&u32::MAX.to_be_bytes());
    let bytes = frame(&bytes, SERVER, [255; 4]);
    assert_eq!(
        client.accept(&parse(&bytes).unwrap(), start + 3),
        Err(Error::InfiniteLeaseUnsupported)
    );
    assert_eq!(client.lease(start + 3), Ok(None));
}

#[test]
fn permitted_absent_udp_checksum_does_not_skip_dhcp_identity_checks() {
    let mut client = requesting(0);
    let mut bytes = frame(&message(5, XID ^ 1, SERVER), SERVER, [255; 4]);
    bytes[40..42].fill(0);
    assert_eq!(
        client.accept(&parse(&bytes).unwrap(), 3),
        Err(Error::Codec(dhcp::Error::WrongTransaction))
    );
    let mut bytes = frame(&message(5, XID, SERVER), SERVER, ADDRESS);
    bytes[40..42].fill(0);
    assert_eq!(
        client.accept(&parse(&bytes).unwrap(), 3),
        Ok(Phase::CheckingAddress)
    );
    assert_eq!(client.lease(3), Ok(None));
}
