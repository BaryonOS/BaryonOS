//! Real codec/Stack bytes with explicitly modeled NIC completions, not native evidence.
use nieuw_os_net_config::{
    dhcp_client::{Error as ClientError, Phase},
    service::{DriveError, DriveOutcome, Error, ProbeTiming, RandomSource, Service},
};
use nieuw_os_net_protocol::{UdpIpv4Tx, build_udp_ipv4, parse};
use nieuw_os_net_stack::{Error as StackError, FRAME_BYTES, LinkConfig, SocketStorage, Stack};

const MAC: [u8; 6] = [2, 0, 0, 0, 0, 7];
const SERVER_MAC: [u8; 6] = [2, 0, 0, 0, 0, 1];
const ADDRESS: [u8; 4] = [192, 0, 2, 42];
const SERVER: [u8; 4] = [192, 0, 2, 1];
const XID: u32 = 0x1234_5678;

fn with_service(f: impl FnOnce(&mut Service<'_>)) {
    let mut storage = [SocketStorage::EMPTY];
    let mut rx = [0; FRAME_BYTES];
    let mut tx = [0; FRAME_BYTES];
    let stack = Stack::new_unconfigured(
        LinkConfig {
            mac: MAC,
            random_seed: 7,
        },
        &mut storage,
        &mut rx,
        &mut tx,
        0,
    )
    .unwrap();
    let mut service = Service::new(stack, ProbeTiming::new(0, 100, 100).unwrap(), 0).unwrap();
    f(&mut service);
}

fn with_transport(f: impl FnOnce(&mut Service<'_>, nieuw_os_net_stack::Client)) {
    let mut storage = [SocketStorage::EMPTY];
    let mut rx = [0; FRAME_BYTES];
    let mut tx = [0; FRAME_BYTES];
    let mut tcp_rx = [0; 512];
    let mut tcp_tx = [0; 512];
    let stack = Stack::new_unconfigured(
        LinkConfig {
            mac: MAC,
            random_seed: 7,
        },
        &mut storage,
        &mut rx,
        &mut tx,
        0,
    )
    .unwrap();
    let mut service = Service::new(stack, ProbeTiming::new(0, 100, 100).unwrap(), 0).unwrap();
    let client = service
        .add_transport_client(&mut tcp_rx, &mut tcp_tx, 0)
        .unwrap();
    f(&mut service, client);
}

fn reply(kind: u8, xid: u32, dns_last: u8) -> Vec<u8> {
    reply_with_router(kind, xid, dns_last, SERVER)
}

fn reply_with_router(kind: u8, xid: u32, dns_last: u8, gateway: [u8; 4]) -> Vec<u8> {
    reply_with_times(kind, xid, dns_last, gateway, None)
}

fn reply_with_times(
    kind: u8,
    xid: u32,
    dns_last: u8,
    gateway: [u8; 4],
    timers: Option<(u32, u32)>,
) -> Vec<u8> {
    reply_with_lease(kind, xid, dns_last, gateway, 80, timers)
}

fn reply_with_lease(
    kind: u8,
    xid: u32,
    dns_last: u8,
    gateway: [u8; 4],
    lease: u32,
    timers: Option<(u32, u32)>,
) -> Vec<u8> {
    let mut message = vec![0; 240];
    message[..3].copy_from_slice(&[2, 1, 6]);
    message[4..8].copy_from_slice(&xid.to_be_bytes());
    message[16..20].copy_from_slice(&ADDRESS);
    message[28..34].copy_from_slice(&MAC);
    message[236..240].copy_from_slice(&[99, 130, 83, 99]);
    message.extend_from_slice(&[53, 1, kind, 54, 4]);
    message.extend_from_slice(&SERVER);
    if kind != 6 {
        message.extend_from_slice(&[51, 4]);
        message.extend_from_slice(&lease.to_be_bytes());
        message.extend_from_slice(&[1, 4, 255, 255, 255, 0]);
        message.extend_from_slice(&[3, 4]);
        message.extend_from_slice(&gateway);
        message.extend_from_slice(&[6, 4, 192, 0, 2, dns_last]);
        if let Some((renew, rebind)) = timers {
            message.extend_from_slice(&[58, 4]);
            message.extend_from_slice(&renew.to_be_bytes());
            message.extend_from_slice(&[59, 4]);
            message.extend_from_slice(&rebind.to_be_bytes());
        }
    }
    message.push(255);
    let mut frame = vec![0; FRAME_BYTES];
    let length = build_udp_ipv4(
        UdpIpv4Tx {
            source_mac: SERVER_MAC,
            destination_mac: MAC,
            source_ipv4: SERVER,
            destination_ipv4: [255; 4],
            source_port: 67,
            destination_port: 68,
            identification: 1,
            ttl: 64,
            mtu: 1500,
        },
        &message,
        &mut frame,
    )
    .unwrap();
    frame.truncate(length);
    frame
}

fn offered(service: &mut Service<'_>, ticks: u64) -> (u64, Vec<u8>) {
    let tx = service.transmission(ticks).unwrap().unwrap();
    (tx.id, tx.bytes.to_vec())
}

fn complete(service: &mut Service<'_>, ticks: u64) -> Vec<u8> {
    let (id, bytes) = offered(service, ticks);
    service.complete_transmission(id, ticks).unwrap();
    bytes
}

fn complete_decline(service: &mut Service<'_>, ticks: u64) {
    service.poll(ticks).unwrap();
    let bytes = complete(service, ticks);
    let packet = parse(&bytes).unwrap();
    assert_eq!(packet.source_address(), [0; 4]);
    assert_eq!(packet.destination_address(), [255; 4]);
    assert_eq!(packet.destination_mac(), [255; 6]);
    assert_eq!(&packet.payload()[240..243], &[53, 1, 4]);
}

fn candidate(service: &mut Service<'_>) {
    candidate_at(service, XID, 0);
}

fn candidate_at(service: &mut Service<'_>, xid: u32, ticks: u64) {
    service.start(xid, ticks, 30).unwrap();
    service.send_dhcp(ticks).unwrap();
    let discover = complete(service, ticks);
    assert_eq!(parse(&discover).unwrap().destination_port(), 67);
    service.receive_frame(&reply(2, xid, 53), ticks).unwrap();
    service.send_dhcp(ticks).unwrap();
    complete(service, ticks);
    service.receive_frame(&reply(5, xid, 53), ticks).unwrap();
    assert_eq!(service.phase(ticks), Ok(Phase::CheckingAddress));
    assert!(service.configuration(ticks).unwrap().is_none());
}

fn bound(service: &mut Service<'_>) {
    candidate(service);
    for ticks in [0, 100, 200] {
        service.poll(ticks).unwrap();
        let probe = complete(service, ticks);
        assert_eq!(&probe[12..22], &[8, 6, 0, 1, 8, 0, 6, 4, 0, 1]);
        assert_eq!(&probe[..6], &[255; 6]);
        assert_eq!(&probe[22..28], &MAC);
        assert_eq!(&probe[28..32], &[0; 4]);
        assert_eq!(&probe[32..38], &[0; 6]);
        assert_eq!(&probe[38..42], &ADDRESS);
        assert!(service.configuration(ticks).unwrap().is_none());
    }
    service.poll(399).unwrap();
    assert!(service.transmission(399).unwrap().is_none());
    assert!(service.configuration(399).unwrap().is_none());
    service.poll(400).unwrap();
    assert!(service.configuration(400).unwrap().is_none());
    let announcement = complete(service, 400);
    assert_eq!(&announcement[28..32], &ADDRESS);
    assert_eq!(&announcement[38..42], &ADDRESS);
    assert_eq!(service.phase(400), Ok(Phase::Bound));
    let lease = service.configuration(400).unwrap().unwrap();
    assert_eq!(lease.address, ADDRESS);
    assert_eq!(lease.routers(), &[SERVER]);
    assert_eq!(lease.dns_servers(), &[[192, 0, 2, 53]]);
    service.poll(600).unwrap();
    let second = complete(service, 600);
    assert_eq!(&second[28..32], &ADDRESS);
    service.poll(601).unwrap();
    assert!(service.transmission(601).unwrap().is_none());
}

#[test]
fn address_and_dns_bind_only_after_three_completed_probes_wait_and_announcement() {
    with_service(bound);
}

#[test]
fn pending_probe_backpressure_wrong_completion_and_repeated_ticks_do_not_prove_a_probe() {
    with_service(|service| {
        candidate(service);
        service.poll(0).unwrap();
        let (id, bytes) = offered(service, 0);
        for _ in 0..10 {
            service.poll(0).unwrap();
            assert_eq!(offered(service, 0), (id, bytes.clone()));
            assert_eq!(
                service.complete_transmission(id + 1, 0),
                Err(Error::Stack(StackError::StaleTransmission))
            );
            assert!(service.configuration(0).unwrap().is_none());
        }
        assert_eq!(service.send_dhcp(0), Err(Error::Stack(StackError::Busy)));
        service.poll(7999).unwrap();
        assert!(service.configuration(7999).unwrap().is_none());
        assert_eq!(offered(service, 7999).0, id);
        assert!(service.transmission(8000).unwrap().is_none());
        assert_eq!(service.phase(8000), Ok(Phase::Expired));
        service.complete_transmission(id, 8000).unwrap();
        service.start(XID + 1, 8000, 30).unwrap();
        assert!(service.send_dhcp(8000).unwrap() > id);
    });
}

#[test]
fn foreign_probe_or_reply_to_zero_target_conflicts_but_own_echo_does_not() {
    for reply_to_zero in [false, true] {
        with_service(|service| {
            candidate(service);
            service.poll(0).unwrap();
            let (id, mut frame) = offered(service, 0);
            service.receive_frame(&frame, 0).unwrap();
            assert_eq!(service.phase(0), Ok(Phase::CheckingAddress));
            frame[6..12].copy_from_slice(&SERVER_MAC);
            frame[22..28].copy_from_slice(&SERVER_MAC);
            if reply_to_zero {
                frame[21] = 2;
                frame[..6].copy_from_slice(&MAC);
                frame[28..32].copy_from_slice(&ADDRESS);
                frame[32..38].copy_from_slice(&MAC);
                frame[38..42].fill(0);
            }
            assert_eq!(
                service.receive_frame(&frame, 0),
                Err(Error::AddressConflict)
            );
            assert!(service.configuration(0).unwrap().is_none());
            assert!(service.transmission(0).unwrap().is_none());
            service.complete_transmission(id, 0).unwrap();
            complete_decline(service, 0);
            assert_eq!(
                service.start(XID + 1, 999, 30),
                Err(Error::ConflictRateLimit)
            );
            service.start(XID + 1, 1000, 30).unwrap();
            service.send_dhcp(1000).unwrap();
        });
    }
}

#[test]
fn renewal_and_rebinding_keep_route_dns_coherent_and_nak_withdraws_pending_tx() {
    with_service(|service| {
        bound(service);
        assert_eq!(service.phase(4000), Ok(Phase::RenewalDue));
        service.begin_renewal(XID + 1, 4000).unwrap();
        service.send_dhcp(4000).unwrap();
        let request = complete(service, 4000);
        let packet = parse(&request).unwrap();
        assert_eq!(packet.destination_address(), SERVER);
        assert_eq!(packet.destination_mac(), SERVER_MAC);
        service.receive_frame(&reply(5, XID + 1, 54), 4000).unwrap();
        assert_eq!(
            service.configuration(4000).unwrap().unwrap().dns_servers(),
            &[[192, 0, 2, 54]]
        );
        assert_eq!(service.phase(8000), Ok(Phase::RenewalDue));
        service.begin_renewal(XID + 2, 8000).unwrap();
        assert_eq!(service.phase(11000), Ok(Phase::Rebinding));
        service.send_dhcp(11000).unwrap();
        let (id, request) = offered(service, 11000);
        assert_eq!(parse(&request).unwrap().destination_address(), [255; 4]);
        service
            .receive_frame(&reply(6, XID + 2, 54), 11000)
            .unwrap();
        assert!(service.configuration(11000).unwrap().is_none());
        assert!(service.transmission(11000).unwrap().is_none());
        assert_eq!(service.phase(11000), Ok(Phase::Invalidated));
        service.complete_transmission(id, 11000).unwrap();
    });
}

#[test]
fn malformed_stale_and_unsolicited_messages_cannot_bind_or_replace_dns() {
    with_service(|service| {
        service.start(XID, 0, 30).unwrap();
        assert_eq!(
            service.receive_frame(&reply(5, XID, 99), 0),
            Err(Error::Client(ClientError::InvalidState))
        );
        assert!(service.receive_frame(&reply(2, XID, 53)[..20], 0).is_err());
        service.send_dhcp(0).unwrap();
        complete(service, 0);
        assert!(service.receive_frame(&reply(2, XID + 1, 53), 0).is_err());
        assert_eq!(service.phase(0), Ok(Phase::Selecting));
        assert!(service.configuration(0).unwrap().is_none());
    });
}

#[test]
fn linkdown_reconnect_and_unknown_never_inherit_the_old_address_or_completion() {
    with_service(|service| {
        bound(service);
        service.link_down(602).unwrap();
        assert!(service.configuration(602).unwrap().is_none());
        service.start(XID + 1, 602, 30).unwrap();
        let id = service.send_dhcp(602).unwrap();
        assert_eq!(
            service.transport_uncertain(602),
            Err(Error::TransportUncertain)
        );
        assert_eq!(
            service.complete_transmission(id, 602),
            Err(Error::TransportUncertain)
        );
        assert_eq!(
            service.transmission(602).unwrap_err(),
            Error::TransportUncertain
        );
        assert_eq!(
            service.start(XID + 2, 603, 30),
            Err(Error::TransportUncertain)
        );
    });
}

#[test]
fn clock_fault_fences_and_is_terminal_even_if_the_clock_later_catches_up() {
    for fault in [600, u64::MAX] {
        with_service(|service| {
            bound(service);
            let error = service.configuration(fault).unwrap_err();
            assert!(matches!(error, Error::ClockFault { .. }));
            assert_eq!(service.configuration(602), Err(error));
            assert_eq!(service.begin_renewal(XID + 1, 4000), Err(error));
        });
    }
}

#[test]
fn configured_address_answers_a_foreign_zero_source_probe_on_the_same_slot() {
    with_service(|service| {
        bound(service);
        let mut frame = vec![0; 60];
        frame[..6].fill(255);
        frame[6..12].copy_from_slice(&SERVER_MAC);
        frame[12..22].copy_from_slice(&[8, 6, 0, 1, 8, 0, 6, 4, 0, 1]);
        frame[22..28].copy_from_slice(&SERVER_MAC);
        frame[38..42].copy_from_slice(&ADDRESS);
        service.receive_frame(&frame, 602).unwrap();
        service.poll(602).unwrap();
        let response = complete(service, 602);
        assert_eq!(&response[..6], &SERVER_MAC);
        assert_eq!(response[21], 2);
        assert_eq!(&response[28..32], &ADDRESS);
        assert_eq!(&response[32..38], &SERVER_MAC);
        assert_eq!(&response[38..42], &[0; 4]);
        assert!(service.configuration(602).unwrap().is_some());
    });
}

#[test]
fn changed_gateway_and_dns_wait_for_old_request_completion_before_new_epoch() {
    with_service(|service| {
        bound(service);
        service.begin_renewal(XID + 1, 4000).unwrap();
        let id = service.send_dhcp(4000).unwrap();
        service
            .receive_frame(&reply_with_router(5, XID + 1, 54, [192, 0, 2, 254]), 4000)
            .unwrap();
        assert!(service.configuration(4000).unwrap().is_none());
        assert!(service.transmission(4000).unwrap().is_none());
        service.poll(4001).unwrap();
        assert!(service.configuration(4001).unwrap().is_none());
        service.complete_transmission(id, 4001).unwrap();
        service.poll(4001).unwrap();
        let lease = service.configuration(4001).unwrap().unwrap();
        assert_eq!(lease.routers(), &[[192, 0, 2, 254]]);
        assert_eq!(lease.dns_servers(), &[[192, 0, 2, 54]]);
    });
}

#[test]
fn late_probe_window_conflict_and_ongoing_duplicate_claim_both_withdraw() {
    with_service(|service| {
        candidate(service);
        let mut frame = Vec::new();
        for ticks in [0, 100, 200] {
            service.poll(ticks).unwrap();
            frame = complete(service, ticks);
        }
        frame[6..12].copy_from_slice(&SERVER_MAC);
        frame[22..28].copy_from_slice(&SERVER_MAC);
        assert_eq!(
            service.receive_frame(&frame, 399),
            Err(Error::AddressConflict)
        );
        service.poll(400).unwrap();
        let (_, decline) = offered(service, 400);
        assert_eq!(&parse(&decline).unwrap().payload()[240..243], &[53, 1, 4]);
        assert!(service.configuration(400).unwrap().is_none());
    });
    with_service(|service| {
        bound(service);
        let mut frame = vec![0; 60];
        frame[..6].fill(255);
        frame[6..12].copy_from_slice(&SERVER_MAC);
        frame[12..22].copy_from_slice(&[8, 6, 0, 1, 8, 0, 6, 4, 0, 1]);
        frame[22..28].copy_from_slice(&SERVER_MAC);
        frame[28..32].copy_from_slice(&ADDRESS);
        frame[38..42].copy_from_slice(&SERVER);
        assert_eq!(
            service.receive_frame(&frame, 602),
            Err(Error::AddressConflict)
        );
        assert!(service.configuration(602).unwrap().is_none());
    });
}

#[test]
fn ten_conflicts_rate_limit_fresh_acquisition_for_sixty_seconds() {
    with_service(|service| {
        for n in 0..10 {
            let ticks = u64::from(n) * 1000;
            candidate_at(service, XID + n, ticks);
            service.poll(ticks).unwrap();
            let (id, mut frame) = offered(service, ticks);
            frame[6..12].copy_from_slice(&SERVER_MAC);
            frame[22..28].copy_from_slice(&SERVER_MAC);
            assert_eq!(
                service.receive_frame(&frame, ticks),
                Err(Error::AddressConflict)
            );
            service.complete_transmission(id, ticks).unwrap();
            complete_decline(service, ticks);
        }
        assert_eq!(
            service.start(XID + 10, 14999, 30),
            Err(Error::ConflictRateLimit)
        );
        service.start(XID + 11, 15000, 30).unwrap();
        service.send_dhcp(15000).unwrap();
    });
}

#[test]
fn lease_deadlines_beyond_stack_clock_range_fail_closed_before_probing() {
    with_service(|service| {
        let ticks = nieuw_os_net_stack::MAX_MONOTONIC_TICKS - 100;
        service.start(XID, ticks, 30).unwrap();
        service.send_dhcp(ticks).unwrap();
        complete(service, ticks);
        service.receive_frame(&reply(2, XID, 53), ticks).unwrap();
        service.send_dhcp(ticks).unwrap();
        complete(service, ticks);
        let error = service
            .receive_frame(&reply(5, XID, 53), ticks)
            .unwrap_err();
        assert!(matches!(error, Error::ClockFault { .. }));
        assert_eq!(service.configuration(ticks), Err(error));
        assert_eq!(service.poll(ticks), Err(error));
    });
}

#[test]
fn selecting_request_preserves_discover_secs_without_moving_the_lease_start() {
    with_service(|service| {
        service.start(XID, 0, 30).unwrap();
        service.send_dhcp(100).unwrap();
        let discover = complete(service, 100);
        service.receive_frame(&reply(2, XID, 53), 500).unwrap();
        service.send_dhcp(500).unwrap();
        let request = complete(service, 500);
        assert_eq!(
            &parse(&request).unwrap().payload()[8..10],
            &parse(&discover).unwrap().payload()[8..10]
        );
        service.receive_frame(&reply(5, XID, 53), 500).unwrap();
        assert_eq!(service.phase(8499), Ok(Phase::CheckingAddress));
        assert_eq!(service.phase(8500), Ok(Phase::Expired));
    });
}

#[test]
fn delayed_renewal_ack_applies_new_route_and_dns_even_when_new_t1_is_already_due() {
    with_service(|service| {
        bound(service);
        service.begin_renewal(XID + 1, 4000).unwrap();
        service.send_dhcp(4000).unwrap();
        complete(service, 4000);
        service
            .receive_frame(
                &reply_with_times(5, XID + 1, 54, [192, 0, 2, 254], Some((1, 70))),
                4100,
            )
            .unwrap();
        assert_eq!(service.phase(4100), Ok(Phase::RenewalDue));
        assert!(service.configuration(4100).unwrap().is_none());
        service.poll(4100).unwrap();
        let lease = service.configuration(4100).unwrap().unwrap();
        assert_eq!(lease.routers(), &[[192, 0, 2, 254]]);
        assert_eq!(lease.dns_servers(), &[[192, 0, 2, 54]]);
        assert_eq!(lease.renew_at, 41);
        assert_eq!(lease.expires_at, 120);
    });
}

#[test]
fn conflict_decline_waits_for_retired_tx_and_real_own_completion_before_reacquiring() {
    with_service(|service| {
        candidate(service);
        service.poll(0).unwrap();
        let (old_id, mut frame) = offered(service, 0);
        frame[6..12].copy_from_slice(&SERVER_MAC);
        frame[22..28].copy_from_slice(&SERVER_MAC);
        assert_eq!(
            service.receive_frame(&frame, 0),
            Err(Error::AddressConflict)
        );
        service.poll(0).unwrap();
        assert!(service.transmission(0).unwrap().is_none());
        assert_eq!(
            service.start(XID + 1, 1000, 30),
            Err(Error::Stack(StackError::Busy))
        );
        service.complete_transmission(old_id, 1000).unwrap();
        service.poll(1000).unwrap();
        let (decline_id, bytes) = offered(service, 1000);
        assert!(decline_id > old_id);
        assert_eq!(&parse(&bytes).unwrap().payload()[240..243], &[53, 1, 4]);
        service.poll(1000).unwrap();
        assert_eq!(offered(service, 1000), (decline_id, bytes));
        assert_eq!(
            service.start(XID + 1, 2000, 30),
            Err(Error::Stack(StackError::Busy))
        );
        assert_eq!(
            service.complete_transmission(old_id, 2000),
            Err(Error::Stack(StackError::StaleTransmission))
        );
        service.complete_transmission(decline_id, 2000).unwrap();
        service.start(XID + 1, 2000, 30).unwrap();
        assert!(service.send_dhcp(2000).unwrap() > decline_id);
    });
}

#[test]
fn configured_transport_uses_the_same_arp_and_tcp_slot_and_expiry_gates_each_operation() {
    with_transport(|service, client| {
        assert_eq!(
            service.connect(client, (SERVER, 443), 40000, 300, 0),
            Err(Error::Stack(StackError::NotConfigured))
        );
        bound(service);
        service
            .connect(client, (SERVER, 443), 40000, 300, 602)
            .unwrap();
        service.poll(602).unwrap();
        let arp_request = complete(service, 602);
        let packet = nieuw_os_net_protocol::arp::parse(&arp_request).unwrap();
        assert_eq!(packet.target_ipv4(), SERVER);
        let mut reply = [0; 60];
        nieuw_os_net_protocol::arp::build_reply(
            nieuw_os_net_protocol::arp::Interface {
                mac: SERVER_MAC,
                ipv4: SERVER,
            },
            &packet,
            &mut reply,
        )
        .unwrap();
        service.receive_frame(&reply, 603).unwrap();
        service.poll(603).unwrap();
        let (id, syn) = offered(service, 603);
        assert_eq!(&syn[12..14], &[8, 0]);
        assert_eq!(syn[23], 6);
        assert_eq!(syn[47] & 0x17, 2);
        assert_eq!(&syn[26..30], &ADDRESS);
        assert_eq!(
            service.state(client, 603),
            Ok(nieuw_os_net_stack::State::SynSent)
        );
        assert_eq!(service.send_dhcp(603), Err(Error::Stack(StackError::Busy)));
        assert_eq!(
            service.send(client, b"expired", 8000),
            Err(Error::Stack(StackError::NotConfigured))
        );
        assert!(service.transmission(8000).unwrap().is_none());
        assert_eq!(
            service.state(client, 8000),
            Ok(nieuw_os_net_stack::State::Closed)
        );
        let mut output = [0xa5; 16];
        assert_eq!(
            service.receive(client, &mut output, 8000),
            Err(Error::Stack(StackError::NotConfigured))
        );
        assert_eq!(output, [0xa5; 16]);
        assert_eq!(
            service.close(client, 8000),
            Err(Error::Stack(StackError::NotConfigured))
        );
        service.complete_transmission(id, 8000).unwrap();
        service.cancel(client, 8000).unwrap();
    });
}

#[test]
fn unchanged_lease_renewal_keeps_live_transport_but_new_route_retires_it() {
    with_transport(|service, client| {
        bound(service);
        service
            .connect(client, (SERVER, 443), 40000, 6000, 602)
            .unwrap();
        service.begin_renewal(XID + 1, 4000).unwrap();
        let old_request = service.send_dhcp(4000).unwrap();
        service.receive_frame(&reply(5, XID + 1, 54), 4000).unwrap();
        assert_eq!(
            service.state(client, 4000),
            Ok(nieuw_os_net_stack::State::SynSent)
        );
        assert_eq!(
            service.configuration(4000).unwrap().unwrap().dns_servers(),
            &[[192, 0, 2, 54]]
        );
        service.complete_transmission(old_request, 4000).unwrap();
        service.begin_renewal(XID + 2, 8000).unwrap();
        let request = service.send_dhcp(8000).unwrap();
        service
            .receive_frame(&reply_with_router(5, XID + 2, 55, [192, 0, 2, 254]), 8000)
            .unwrap();
        assert_eq!(
            service.state(client, 8000),
            Ok(nieuw_os_net_stack::State::Closed)
        );
        assert_eq!(
            service.send(client, b"old route", 8000),
            Err(Error::Stack(StackError::NotConfigured))
        );
        service.complete_transmission(request, 8000).unwrap();
        service.poll(8000).unwrap();
        assert_eq!(
            service.configuration(8000).unwrap().unwrap().routers(),
            &[[192, 0, 2, 254]]
        );
    });
}

#[test]
fn foreign_claim_during_pending_gateway_change_cancels_reapplication() {
    with_service(|service| {
        bound(service);
        service.begin_renewal(XID + 1, 4000).unwrap();
        let id = service.send_dhcp(4000).unwrap();
        service
            .receive_frame(&reply_with_router(5, XID + 1, 54, [192, 0, 2, 254]), 4000)
            .unwrap();
        assert!(service.configuration(4000).unwrap().is_none());

        let mut claim = vec![0; 60];
        claim[..6].fill(255);
        claim[6..12].copy_from_slice(&SERVER_MAC);
        claim[12..22].copy_from_slice(&[8, 6, 0, 1, 8, 0, 6, 4, 0, 1]);
        claim[22..28].copy_from_slice(&SERVER_MAC);
        claim[28..32].copy_from_slice(&ADDRESS);
        claim[38..42].copy_from_slice(&SERVER);
        assert_eq!(
            service.receive_frame(&claim, 4000),
            Err(Error::AddressConflict),
            "the accepted lease is still the pending address epoch"
        );

        service.complete_transmission(id, 4001).unwrap();
        // The conflict now has a legitimate DHCPDECLINE path. Completing it
        // must not reactivate the rejected renewal configuration.
        complete_decline(service, 4001);
        assert!(service.configuration(4001).unwrap().is_none());
        service.poll(4001).unwrap();
        assert!(service.configuration(4001).unwrap().is_none());
        assert!(service.transmission(4001).unwrap().is_none());
    });
}

struct Numbers {
    values: std::collections::VecDeque<u32>,
    calls: usize,
}

impl Numbers {
    fn new(values: &[u32]) -> Self {
        Self {
            values: values.iter().copied().collect(),
            calls: 0,
        }
    }
}

impl RandomSource for Numbers {
    type Error = &'static str;
    fn random_u32(&mut self) -> Result<u32, Self::Error> {
        self.calls += 1;
        self.values.pop_front().ok_or("entropy unavailable")
    }
}

fn auto_candidate(service: &mut Service<'_>) {
    let mut random = Numbers::new(&[90100, XID, 10100, 10100, 10100, 20100, 20100]);
    service.automatic_link_up(0).unwrap();
    assert_eq!(
        service.drive(0, &mut random),
        Ok(DriveOutcome::WaitingUntil(100))
    );
    assert!(matches!(
        service.drive(100, &mut random),
        Ok(DriveOutcome::Published(_))
    ));
    complete(service, 100);
    service.receive_frame(&reply(2, XID, 53), 100).unwrap();
    assert!(matches!(
        service.drive(100, &mut random),
        Ok(DriveOutcome::Published(_))
    ));
    complete(service, 100);
    service.receive_frame(&reply(5, XID, 53), 100).unwrap();
}

#[test]
fn automatic_loop_uses_randomized_exponential_backoff_without_resetting_xid() {
    with_service(|service| {
        let mut random = Numbers::new(&[90100, XID, 10100, 10100, 10100, 20100, 20100]);
        assert_eq!(service.drive(0, &mut random), Ok(DriveOutcome::LinkDown));
        assert_eq!(random.calls, 0);
        service.automatic_link_up(0).unwrap();
        assert_eq!(
            service.drive(0, &mut random),
            Ok(DriveOutcome::WaitingUntil(100))
        );
        assert_eq!(
            service.drive(99, &mut random),
            Ok(DriveOutcome::WaitingUntil(100))
        );
        assert_eq!(random.calls, 1);
        let id = match service.drive(100, &mut random).unwrap() {
            DriveOutcome::Published(id) => id,
            other => panic!("unexpected {other:?}"),
        };
        let (_, first) = offered(service, 100);
        service.complete_transmission(id, 100).unwrap();
        assert_eq!(
            service.drive(399, &mut random),
            Ok(DriveOutcome::WaitingUntil(400))
        );
        assert!(matches!(
            service.drive(400, &mut random),
            Ok(DriveOutcome::Published(_))
        ));
        let second = complete(service, 400);
        assert_eq!(&parse(&first).unwrap().payload()[4..8], &XID.to_be_bytes());
        assert_eq!(&parse(&second).unwrap().payload()[4..8], &XID.to_be_bytes());
        assert_eq!(
            service.drive(1099, &mut random),
            Ok(DriveOutcome::WaitingUntil(1100))
        );
        assert_eq!(random.calls, 7);
    });
}

#[test]
fn automatic_loop_never_treats_backpressure_as_delivery_or_consumes_entropy_for_it() {
    with_service(|service| {
        let mut random = Numbers::new(&[90100, XID, 10100, 10100, 10100, 20100]);
        service.automatic_link_up(0).unwrap();
        service.drive(0, &mut random).unwrap();
        service.drive(100, &mut random).unwrap();
        let (id, frame) = offered(service, 100);
        let calls = random.calls;
        for _ in 0..10 {
            assert_eq!(
                service.drive(1000, &mut random),
                Ok(DriveOutcome::Backpressure)
            );
            assert_eq!(offered(service, 1000), (id, frame.clone()));
            assert_eq!(random.calls, calls);
        }
        service.link_down(1000).unwrap();
        assert_eq!(service.drive(1000, &mut random), Ok(DriveOutcome::LinkDown));
        service.automatic_link_up(1000).unwrap();
        assert_eq!(
            service.drive(1000, &mut random),
            Ok(DriveOutcome::Backpressure)
        );
        service.complete_transmission(id, 1000).unwrap();
        assert_eq!(
            service.drive(1000, &mut random),
            Err(DriveError::Random("entropy unavailable"))
        );
        assert!(service.configuration(1000).unwrap().is_none());
    });
}

#[test]
fn automatic_loop_rejects_broken_or_missing_entropy_without_publishing_a_frame() {
    with_service(|service| {
        service.automatic_link_up(0).unwrap();
        let mut absent = Numbers::new(&[]);
        assert_eq!(
            service.drive(0, &mut absent),
            Err(DriveError::Random("entropy unavailable"))
        );
        assert!(service.transmission(0).unwrap().is_none());
        let mut stuck = Numbers::new(&[0; 8]);
        assert_eq!(
            service.drive(0, &mut stuck),
            Err(DriveError::RandomRejected)
        );
        assert_eq!(stuck.calls, 8);
        assert!(service.transmission(0).unwrap().is_none());
        let mut duplicate = Numbers::new(&[90100, 0, 0, 0, 0, 0, 0, 0, 0]);
        service.drive(0, &mut duplicate).unwrap();
        assert_eq!(
            service.drive(100, &mut duplicate),
            Err(DriveError::RandomRejected)
        );
        assert!(service.transmission(100).unwrap().is_none());
    });
}

#[test]
fn automatic_loop_drives_probes_t1_t2_and_reacquires_after_expiry_without_stale_config() {
    with_service(|service| {
        auto_candidate(service);
        let mut random = Numbers::new(&[]);
        for ticks in [100, 200, 300, 500, 700] {
            service.drive(ticks, &mut random).unwrap();
            complete(service, ticks);
        }
        assert_eq!(service.phase(700), Ok(Phase::Bound));
        assert_eq!(random.calls, 0);
        let mut renewal = Numbers::new(&[XID + 1, 20100, 20100, 90100]);
        assert!(matches!(
            service.drive(4100, &mut renewal),
            Ok(DriveOutcome::Published(_))
        ));
        let request = complete(service, 4100);
        assert_eq!(parse(&request).unwrap().destination_address(), SERVER);
        assert_eq!(
            &parse(&request).unwrap().payload()[4..8],
            &(XID + 1).to_be_bytes()
        );
        assert_eq!(
            service.drive(7100 - 1, &mut renewal),
            Ok(DriveOutcome::WaitingUntil(7100))
        );
        assert!(matches!(
            service.drive(7100, &mut renewal),
            Ok(DriveOutcome::Published(_))
        ));
        let request = complete(service, 7100);
        assert_eq!(parse(&request).unwrap().destination_address(), [255; 4]);
        assert_eq!(
            service.drive(8100, &mut renewal),
            Ok(DriveOutcome::WaitingUntil(8200))
        );
        assert!(service.configuration(8100).unwrap().is_none());
        assert!(service.transmission(8100).unwrap().is_none());
    });
}

fn auto_bound(service: &mut Service<'_>) {
    auto_candidate(service);
    let mut random = Numbers::new(&[]);
    for ticks in [100, 200, 300, 500, 700] {
        service.drive(ticks, &mut random).unwrap();
        complete(service, ticks);
    }
    assert_eq!(service.phase(700), Ok(Phase::Bound));
}

#[test]
fn renewal_retry_schedule_is_bound_to_the_new_transaction_not_the_old_lease() {
    // Rebinding -> Rebinding: the ACK for XID+1 (lease 39 s, T1 1 s, T2 30 s
    // from request_at 41 s) is already past its new T2 when it arrives, so
    // drive starts XID+2 in the same phase. Its first REQUEST must go out now
    // and its retry deadline must be bounded by the NEW expiry (80 s), not the
    // old transaction's deadline at 81 s, which lies beyond the new lease.
    with_service(|service| {
        auto_bound(service);
        let mut random = Numbers::new(&[XID + 1, 20100, 20100, XID + 2, 20100]);
        assert!(matches!(
            service.drive(4100, &mut random),
            Ok(DriveOutcome::Published(_))
        ));
        complete(service, 4100);
        assert!(matches!(
            service.drive(7100, &mut random),
            Ok(DriveOutcome::Published(_))
        ));
        complete(service, 7100);
        let ack = reply_with_lease(5, XID + 1, 53, SERVER, 39, Some((1, 30)));
        service.receive_frame(&ack, 7300).unwrap();
        assert_eq!(service.phase(7300), Ok(Phase::RenewalDue));
        assert!(service.configuration(7300).unwrap().is_some());
        let outcome = service.drive(7300, &mut random);
        assert!(
            matches!(outcome, Ok(DriveOutcome::Published(_))),
            "new transaction inherited an old deadline: {outcome:?}"
        );
        let request = complete(service, 7300);
        assert_eq!(parse(&request).unwrap().destination_address(), [255; 4]);
        assert_eq!(
            &parse(&request).unwrap().payload()[4..8],
            &(XID + 2).to_be_bytes()
        );
        assert_eq!(service.phase(7300), Ok(Phase::Rebinding));
        assert_eq!(
            service.drive(7301, &mut random),
            Ok(DriveOutcome::WaitingUntil(8000))
        );
        assert_eq!(random.calls, 5);
    });
    // Renewing -> Renewing: the ACK for XID+1 (lease 80 s, T1 1 s, T2 60 s)
    // is due immediately; XID+2 must not inherit the old 71 s rebinding
    // deadline and its retry is bounded by the new T2 at 101 s.
    with_service(|service| {
        auto_bound(service);
        let mut random = Numbers::new(&[XID + 1, 20100, XID + 2, 20100]);
        assert!(matches!(
            service.drive(4100, &mut random),
            Ok(DriveOutcome::Published(_))
        ));
        complete(service, 4100);
        let ack = reply_with_lease(5, XID + 1, 53, SERVER, 80, Some((1, 60)));
        service.receive_frame(&ack, 4200).unwrap();
        assert_eq!(service.phase(4200), Ok(Phase::RenewalDue));
        let outcome = service.drive(4200, &mut random);
        assert!(
            matches!(outcome, Ok(DriveOutcome::Published(_))),
            "new transaction inherited an old deadline: {outcome:?}"
        );
        let request = complete(service, 4200);
        assert_eq!(parse(&request).unwrap().destination_address(), SERVER);
        assert_eq!(
            &parse(&request).unwrap().payload()[4..8],
            &(XID + 2).to_be_bytes()
        );
        assert_eq!(service.phase(4200), Ok(Phase::Renewing));
        assert_eq!(
            service.drive(4201, &mut random),
            Ok(DriveOutcome::WaitingUntil(10100))
        );
        assert_eq!(random.calls, 4);
    });
}

#[test]
fn automatic_backoff_doubles_to_measured_sixty_four_second_cap_and_timeout_restarts() {
    with_service(|service| {
        let mut random = Numbers::new(&[
            90100, XID, 10100, 10100, 10100, 20100, 20100, 20100, 20100, 10100, 10100, 90100,
        ]);
        service.automatic_link_up(0).unwrap();
        service.drive(0, &mut random).unwrap();
        let published = [100, 400, 1100, 2600, 5700, 12000];
        for (index, ticks) in published.into_iter().enumerate() {
            assert!(matches!(
                service.drive(ticks, &mut random),
                Ok(DriveOutcome::Published(_))
            ));
            let bytes = complete(service, ticks);
            assert_eq!(&parse(&bytes).unwrap().payload()[4..8], &XID.to_be_bytes());
            if let Some(next) = published.get(index + 1) {
                assert_eq!(
                    service.drive(*next - 1, &mut random),
                    Ok(DriveOutcome::WaitingUntil(*next))
                );
                assert!(*next - ticks <= 6400);
            }
        }
        assert_eq!(
            service.drive(12100, &mut random),
            Ok(DriveOutcome::WaitingUntil(12200))
        );
        assert_eq!(service.phase(12100), Ok(Phase::TimedOut));
        assert!(service.configuration(12100).unwrap().is_none());
    });
}

#[test]
fn automatic_startup_delay_checks_both_exact_uniform_range_endpoints() {
    for (sample, expected) in [(90100, 100), (900, 1000)] {
        with_service(|service| {
            service.automatic_link_up(0).unwrap();
            let mut random = Numbers::new(&[sample]);
            assert_eq!(
                service.drive(0, &mut random),
                Ok(DriveOutcome::WaitingUntil(expected))
            );
            assert_eq!(
                service.drive(expected - 1, &mut random),
                Ok(DriveOutcome::WaitingUntil(expected))
            );
            assert_eq!(random.calls, 1);
        });
    }
}
