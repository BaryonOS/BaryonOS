//! Real codec/Stack bytes with explicitly modeled NIC completions, not native evidence.
use nieuw_os_net_config::{
    dhcp_client::{Error as ClientError, Phase},
    service::{Error, ProbeTiming, Service},
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
    let mut message = vec![0; 240];
    message[..3].copy_from_slice(&[2, 1, 6]);
    message[4..8].copy_from_slice(&xid.to_be_bytes());
    message[16..20].copy_from_slice(&ADDRESS);
    message[28..34].copy_from_slice(&MAC);
    message[236..240].copy_from_slice(&[99, 130, 83, 99]);
    message.extend_from_slice(&[53, 1, kind, 54, 4]);
    message.extend_from_slice(&SERVER);
    if kind != 6 {
        message.extend_from_slice(&[51, 4, 0, 0, 0, 80, 1, 4, 255, 255, 255, 0]);
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
        assert!(service.transmission(400).unwrap().is_none());
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
        service.poll(4001).unwrap();
        assert!(service.configuration(4001).unwrap().is_none());
        assert!(service.transmission(4001).unwrap().is_none());
    });
}
