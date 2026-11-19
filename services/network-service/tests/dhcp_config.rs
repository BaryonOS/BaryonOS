//! Host model: real DHCP/ARP/TCP bytes, modeled NIC completions, no native evidence.
use nieuw_os_net_config::service::{DriveOutcome, ProbeTiming, RandomSource};
use nieuw_os_net_protocol::{arp, build_udp_ipv4, parse, UdpIpv4Tx};
use nieuw_os_net_stack::{LinkConfig, SocketStorage, FRAME_BYTES};
use nieuw_os_network_service::{
    wire::{HeaderFields, Operation, Packet, Status},
    Buffers, Destination, DhcpConfiguration, Fault, Service,
};
use std::collections::VecDeque;

const MAC: [u8; 6] = [2, 0, 0, 0, 0, 7];
const SERVER_MAC: [u8; 6] = [2, 0, 0, 0, 0, 1];
const ADDRESS: [u8; 4] = [192, 0, 2, 42];
const SERVER: [u8; 4] = [192, 0, 2, 1];
// Off-link .255 is a valid host under a remote /16; the local /24 cannot reject it.
const REMOTE: [u8; 4] = [198, 51, 100, 255];

fn with_service(f: impl FnOnce(&mut Service<'_>)) {
    let mut storage = [SocketStorage::EMPTY, SocketStorage::EMPTY];
    let (mut rx, mut tx) = ([0; FRAME_BYTES], [0; FRAME_BYTES]);
    let (mut ra, mut rb, mut ta, mut tb) = ([0; 4096], [0; 4096], [0; 4096], [0; 4096]);
    let mut service = Service::new_dhcp(
        DhcpConfiguration {
            epoch: 19,
            pids: [11, 12],
            link: LinkConfig {
                mac: MAC,
                random_seed: 7,
            },
            destinations: [SERVER, REMOTE].map(|ipv4| Destination {
                ipv4,
                tcp_first: 8080,
                tcp_count: 1,
                udp_port: 9000,
            }),
        },
        Buffers {
            storage: &mut storage,
            rx_frame: &mut rx,
            tx_frame: &mut tx,
            rx_sockets: [&mut ra, &mut rb],
            tx_sockets: [&mut ta, &mut tb],
        },
        ProbeTiming::new(0, 100, 100).unwrap(),
        0,
    )
    .unwrap();
    f(&mut service);
}

struct Entropy(VecDeque<u32>);
impl RandomSource for Entropy {
    type Error = ();
    fn random_u32(&mut self) -> Result<u32, ()> {
        self.0.pop_front().ok_or(())
    }
}
fn random(xid: u32) -> Entropy {
    Entropy([901_000, xid, 101_000, 101_000, 101_000, 201_000, 201_000].into())
}

// Same real BOOTP/options + IPv4/UDP construction as net-config/tests/service.rs.
fn reply(kind: u8, xid: u32, dns: bool, gateway: [u8; 4]) -> Vec<u8> {
    let mut message = vec![0; 240];
    message[..3].copy_from_slice(&[2, 1, 6]);
    message[4..8].copy_from_slice(&xid.to_be_bytes());
    message[16..20].copy_from_slice(&ADDRESS);
    message[28..34].copy_from_slice(&MAC);
    message[236..240].copy_from_slice(&[99, 130, 83, 99]);
    message.extend_from_slice(&[53, 1, kind, 54, 4]);
    message.extend_from_slice(&SERVER);
    if kind != 6 {
        message.extend_from_slice(&[51, 4, 0, 0, 0, 80, 1, 4, 255, 255, 255, 0, 3, 4]);
        message.extend_from_slice(&gateway);
        if dns {
            message.extend_from_slice(&[6, 4, 192, 0, 2, 53]);
        }
    }
    message.push(255);
    udp(SERVER, [255; 4], 67, 68, &message)
}
fn udp(
    source: [u8; 4],
    destination: [u8; 4],
    source_port: u16,
    destination_port: u16,
    bytes: &[u8],
) -> Vec<u8> {
    let mut frame = vec![0; FRAME_BYTES];
    let n = build_udp_ipv4(
        UdpIpv4Tx {
            source_mac: SERVER_MAC,
            destination_mac: MAC,
            source_ipv4: source,
            destination_ipv4: destination,
            source_port,
            destination_port,
            identification: 1,
            ttl: 64,
            mtu: 1500,
        },
        bytes,
        &mut frame,
    )
    .unwrap();
    frame.truncate(n);
    frame
}
fn complete(s: &mut Service<'_>, now: u64) -> Vec<u8> {
    let tx = s.transmission_at(now).unwrap().unwrap();
    let (id, bytes) = (tx.id, tx.bytes.to_vec());
    s.complete_transmission_at(id, now).unwrap();
    bytes
}
fn rpc(
    s: &mut Service<'_>,
    slot: usize,
    op: Operation,
    request: u64,
    token: u64,
    now: u64,
) -> Packet {
    let data: &[u8] = if matches!(op, Operation::SendTcp | Operation::SendUdp) {
        b"private"
    } else {
        &[]
    };
    let packet = Packet::request(
        HeaderFields {
            operation: op,
            request,
            epoch: 19,
            connection: token,
            argument: if op == Operation::OpenTcp {
                300
            } else {
                data.len() as u32
            },
            address: if !matches!(op, Operation::OpenTcp | Operation::SendUdp) {
                [0; 4]
            } else if slot == 0 {
                SERVER
            } else {
                REMOTE
            },
            port: if op == Operation::SendUdp {
                9000
            } else if op == Operation::OpenTcp {
                8080
            } else {
                0
            },
        },
        data,
    )
    .unwrap();
    s.handle(slot, 11 + slot as u64, 0, packet.bytes(), now)
        .unwrap()
        .unwrap()
}
fn acquire(s: &mut Service<'_>, start: u64, xid: u32, dns: bool) -> u64 {
    s.link_up(start).unwrap();
    let mut rng = random(xid);
    assert_eq!(
        s.drive_dhcp(start, &mut rng),
        Ok(DriveOutcome::WaitingUntil(start + 100))
    );
    let now = start + 100;
    assert!(matches!(
        s.drive_dhcp(now, &mut rng),
        Ok(DriveOutcome::Published(_))
    ));
    let discover = complete(s, now);
    let packet = parse(&discover).unwrap();
    assert_eq!(packet.source_address(), [0; 4]);
    assert_eq!(&packet.payload()[240..243], &[53, 1, 1]);
    s.ingress(&reply(2, xid, dns, SERVER), now).unwrap();
    assert!(matches!(
        s.drive_dhcp(now, &mut rng),
        Ok(DriveOutcome::Published(_))
    ));
    let request = complete(s, now);
    assert_eq!(&parse(&request).unwrap().payload()[240..243], &[53, 1, 3]);
    s.ingress(&reply(5, xid, dns, SERVER), now).unwrap();
    assert!(s.dhcp_configuration(now).unwrap().is_none());
    for tick in [now, now + 100, now + 200] {
        s.poll(tick).unwrap();
        let tx = s.transmission_at(tick).unwrap().unwrap();
        let id = tx.id;
        assert_eq!(&tx.bytes[12..22], &[8, 6, 0, 1, 8, 0, 6, 4, 0, 1]);
        assert_eq!(&tx.bytes[28..32], &[0; 4]);
        assert_eq!(&tx.bytes[38..42], &ADDRESS);
        assert!(s.dhcp_configuration(tick).unwrap().is_none());
        // Neither stale nor legacy completion can advance a DHCP probe.
        assert!(s.complete_transmission_at(id + 1, tick).is_err());
        assert_eq!(s.complete_transmission(id), Err(Fault::Configuration));
        assert!(s.transmission().is_none());
        assert!(s.dhcp_configuration(tick).unwrap().is_none());
        complete(s, tick);
    }
    s.poll(now + 399).unwrap();
    assert!(s.transmission_at(now + 399).unwrap().is_none());
    s.poll(now + 400).unwrap();
    assert!(s.dhcp_configuration(now + 400).unwrap().is_none());
    let announcement = complete(s, now + 400);
    assert_eq!(&announcement[28..32], &ADDRESS);
    let lease = s.dhcp_configuration(now + 400).unwrap().unwrap();
    assert_eq!(lease.address, ADDRESS);
    assert_eq!(lease.routers(), &[SERVER]);
    assert_eq!(
        lease.dns_servers(),
        if dns { &[[192, 0, 2, 53]][..] } else { &[] }
    );
    s.poll(now + 600).unwrap();
    complete(s, now + 600);
    now + 600
}

#[test]
fn dhcp_probes_gate_same_transport_and_dns_has_no_fallback() {
    with_service(|s| {
        assert_eq!(
            rpc(s, 0, Operation::OpenTcp, 1, 0, 0).status(),
            Status::Again
        );
        let now = acquire(s, 0, 0x1234, false);
        assert_eq!(
            rpc(s, 0, Operation::OpenTcp, 2, 0, now).status(),
            Status::Ok
        );
        s.poll(now).unwrap();
        let arp = complete(s, now);
        assert_eq!(&arp[28..32], &ADDRESS);
        assert_eq!(&arp[38..42], &SERVER);
        let mut frame = [0; 60];
        let n = arp::build_reply(
            arp::Interface {
                mac: SERVER_MAC,
                ipv4: SERVER,
            },
            &arp::parse(&arp).unwrap(),
            &mut frame,
        )
        .unwrap();
        s.ingress(&frame[..n], now).unwrap();
        s.poll(now).unwrap();
        let syn = complete(s, now);
        assert_eq!(&syn[26..30], &ADDRESS);
        assert_eq!(syn[23], 6);
        assert_eq!(syn[47] & 2, 2);
        // Fixed PID and destination authority still apply after DHCP.
        let wrong = Packet::request(
            HeaderFields {
                operation: Operation::OpenTcp,
                request: 1,
                epoch: 19,
                connection: 0,
                argument: 300,
                address: SERVER,
                port: 8080,
            },
            &[],
        )
        .unwrap();
        assert!(s.handle(1, 11, 0, wrong.bytes(), now).unwrap().is_none());
        assert_eq!(
            s.handle(1, 12, 0, wrong.bytes(), now)
                .unwrap()
                .unwrap()
                .status(),
            Status::Access
        );
    });
}

#[test]
fn nak_expiry_and_link_down_withdraw_and_reconnect_never_reuses_tokens() {
    for loss in 0..3 {
        with_service(|s| {
            let now = acquire(s, 0, 0x1234, true);
            let opened = rpc(s, 0, Operation::OpenTcp, 1, 0, now);
            let old = opened.header().connection;
            assert_eq!(opened.status(), Status::Ok);
            s.ingress(&udp(SERVER, ADDRESS, 9000, 45000, b"private"), now)
                .unwrap();
            s.poll(now).unwrap();
            let pending = s.transmission_at(now).unwrap().unwrap().id;
            let lost = if loss == 1 { 8100 } else { 4100 };
            if loss == 0 {
                // Actual renewal REQUEST followed by server NAK.
                s.complete_transmission_at(pending, lost).unwrap();
                let mut rng = Entropy([0x5678, 101_000].into());
                s.drive_dhcp(lost, &mut rng).unwrap();
                complete(s, lost);
                s.ingress(&reply(6, 0x5678, true, SERVER), lost).unwrap();
            } else if loss == 2 {
                s.link_down(lost).unwrap();
            }
            // Expiry is observed by the RPC, without requiring a preceding poll.
            assert_eq!(
                rpc(s, 0, Operation::OpenTcp, 2, 0, lost).status(),
                Status::Again
            );
            assert_eq!(
                rpc(s, 0, Operation::SendTcp, 3, old, lost).status(),
                Status::Stale
            );
            assert_eq!(
                rpc(s, 0, Operation::ReceiveUdp, 4, 0, lost).status(),
                Status::Again
            );
            assert!(s.dhcp_configuration(lost).unwrap().is_none());
            assert!(s.transmission_at(lost).unwrap().is_none());
            if loss != 0 {
                // Withdrawal hid the real pending frame but did not release its slot.
                s.link_up(lost).unwrap();
                let mut rng = random(0xabcd);
                assert_eq!(s.drive_dhcp(lost, &mut rng), Ok(DriveOutcome::Backpressure));
                s.complete_transmission_at(pending, lost).unwrap();
            }
            let ready = acquire(s, lost, 0xabcd, false);
            let new = rpc(s, 0, Operation::OpenTcp, 5, 0, ready);
            assert_eq!(new.status(), Status::Ok);
            assert_ne!(new.header().connection, old);
            assert_eq!(
                rpc(s, 0, Operation::SendTcp, 6, old, ready).status(),
                Status::Stale
            );
            assert_eq!(
                rpc(s, 0, Operation::ReceiveUdp, 7, 0, ready).status(),
                Status::Again
            );
            assert_eq!(
                rpc(s, 0, Operation::OpenTcp, 5, 0, ready).status(),
                Status::Replay
            );
        });
    }
}

#[test]
fn reconnect_preserves_revoked_and_retired_authority() {
    with_service(|s| {
        let now = acquire(s, 0, 0x1234, true);
        s.revoke(0, now).unwrap();
        s.retire_slot(1, now).unwrap();
        s.link_down(now).unwrap();
        let ready = acquire(s, now, 0x5678, false);
        assert_eq!(s.revoked_mask(), 1);
        assert_eq!(s.retired_mask(), 2);
        assert_eq!(
            rpc(s, 0, Operation::OpenTcp, 1, 0, ready).status(),
            Status::Access
        );
        assert_eq!(
            rpc(s, 1, Operation::OpenTcp, 1, 0, ready).status(),
            Status::Closed
        );
    });
}

#[test]
fn leased_gateway_routes_udp_without_changing_destination_authority() {
    with_service(|s| {
        let now = acquire(s, 0, 0x1234, false);
        assert_eq!(
            rpc(s, 1, Operation::SendUdp, 1, 0, now).status(),
            Status::Again
        );
        let probe = complete(s, now);
        assert_eq!(&probe[38..42], &SERVER);
        let mut reply = [0; 60];
        let n = arp::build_reply(
            arp::Interface {
                mac: SERVER_MAC,
                ipv4: SERVER,
            },
            &arp::parse(&probe).unwrap(),
            &mut reply,
        )
        .unwrap();
        s.ingress(&reply[..n], now).unwrap();
        assert_eq!(
            rpc(s, 1, Operation::SendUdp, 2, 0, now).status(),
            Status::Ok
        );
        let frame = complete(s, now);
        let packet = parse(&frame).unwrap();
        assert_eq!(packet.destination_mac(), SERVER_MAC);
        assert_eq!(packet.destination_address(), REMOTE);
        assert_eq!(packet.source_address(), ADDRESS);
        assert_eq!(packet.payload(), b"private");
    });
}

#[test]
fn renewal_preserves_tokens_only_while_route_epoch_is_unchanged() {
    with_service(|s| {
        let now = acquire(s, 0, 0x1234, true);
        let old = rpc(s, 0, Operation::OpenTcp, 1, 0, now).header().connection;
        let renew = |s: &mut Service<'_>, tick, xid| {
            let mut rng = Entropy([xid, 101_000].into());
            assert!(matches!(
                s.drive_dhcp(tick, &mut rng),
                Ok(DriveOutcome::Published(_))
            ));
            let request = complete(s, tick);
            assert_eq!(&parse(&request).unwrap().payload()[240..243], &[53, 1, 3]);
        };
        renew(s, 4100, 0x5678);
        s.ingress(&reply(5, 0x5678, false, SERVER), 4100).unwrap();
        assert!(s
            .dhcp_configuration(4100)
            .unwrap()
            .unwrap()
            .dns_servers()
            .is_empty());
        assert_eq!(
            rpc(s, 0, Operation::State, 2, old, 4100).status(),
            Status::Ok
        );
        renew(s, 8100, 0xabcd);
        // Keep the last old-route transport frame genuinely outstanding.
        s.poll(8100).unwrap();
        let pending = s.transmission_at(8100).unwrap().unwrap().id;
        let gateway = [192, 0, 2, 9];
        s.ingress(&reply(5, 0xabcd, true, gateway), 8100).unwrap();
        assert!(s.dhcp_configuration(8100).unwrap().is_none());
        assert_eq!(
            rpc(s, 0, Operation::OpenTcp, 3, 0, 8100).status(),
            Status::Again
        );
        assert_eq!(
            rpc(s, 0, Operation::State, 4, old, 8100).status(),
            Status::Stale
        );
        s.poll(8100).unwrap();
        assert!(s.dhcp_configuration(8100).unwrap().is_none());
        s.complete_transmission_at(pending, 8100).unwrap();
        s.poll(8100).unwrap();
        let lease = s.dhcp_configuration(8100).unwrap().unwrap();
        assert_eq!(lease.routers(), &[gateway]);
        assert_eq!(lease.dns_servers(), &[[192, 0, 2, 53]]);
        assert_eq!(
            rpc(s, 0, Operation::State, 5, old, 8100).status(),
            Status::Stale
        );
        let new = rpc(s, 0, Operation::OpenTcp, 6, 0, 8100);
        assert_eq!(new.status(), Status::Ok);
        assert_ne!(new.header().connection, old);
    });
}

#[test]
fn udp_and_tx_entry_points_observe_expiry_without_a_poll() {
    for operation in [Operation::SendUdp, Operation::ReceiveUdp] {
        with_service(|s| {
            let now = acquire(s, 0, 0x1234, true);
            assert_eq!(
                rpc(s, 0, Operation::SendUdp, 1, 0, now).status(),
                Status::Again
            );
            let probe = complete(s, now);
            let mut reply = [0; 60];
            let n = arp::build_reply(
                arp::Interface {
                    mac: SERVER_MAC,
                    ipv4: SERVER,
                },
                &arp::parse(&probe).unwrap(),
                &mut reply,
            )
            .unwrap();
            s.ingress(&reply[..n], now).unwrap();
            s.ingress(&udp(SERVER, ADDRESS, 9000, 45000, b"private"), now)
                .unwrap();
            assert_eq!(rpc(s, 0, operation, 2, 0, 8100).status(), Status::Again);
            assert!(s.transmission_at(8100).unwrap().is_none());
            assert!(s.dhcp_configuration(8100).unwrap().is_none());
        });
    }
    with_service(|s| {
        let now = acquire(s, 0, 0x1234, true);
        assert_eq!(
            rpc(s, 0, Operation::SendUdp, 1, 0, now).status(),
            Status::Again
        );
        let pending = s.transmission_at(now).unwrap().unwrap().id;
        // Publication itself observes expiry; no app request or poll precedes it.
        assert!(s.transmission_at(8100).unwrap().is_none());
        assert_eq!(s.complete_transmission(pending), Err(Fault::Configuration));
        s.complete_transmission_at(pending, 8100).unwrap();
    });
}

#[test]
fn withheld_unsubmitted_frame_is_discarded_and_dhcp_resumes() {
    with_service(|s| {
        let now = acquire(s, 0, 0x1234, true);
        assert_eq!(
            rpc(s, 0, Operation::SendUdp, 1, 0, now).status(),
            Status::Again
        );
        let pending = s.transmission_at(now).unwrap().unwrap().id;
        // The adapter never submitted `pending`; link-down withdraws it.
        let lost = 4100;
        s.link_down(lost).unwrap();
        assert!(s.transmission_at(lost).unwrap().is_none());
        assert_eq!(s.withheld_transmission_at(lost).unwrap(), Some(pending));
        // Without recovery the slot stalls DHCP.
        s.link_up(lost).unwrap();
        let mut rng = random(0xabcd);
        assert_eq!(s.drive_dhcp(lost, &mut rng), Ok(DriveOutcome::Backpressure));
        // A wrong identity is refused and changes nothing.
        assert!(s
            .discard_withheld_transmission_at(pending + 1, lost)
            .is_err());
        assert_eq!(s.withheld_transmission_at(lost).unwrap(), Some(pending));
        s.discard_withheld_transmission_at(pending, lost).unwrap();
        assert_eq!(s.withheld_transmission_at(lost).unwrap(), None);
        // Not a completion: the discarded identity cannot also be completed.
        assert!(s.complete_transmission_at(pending, lost).is_err());
        assert!(s.discard_withheld_transmission_at(pending, lost).is_err());
        let ready = acquire(s, lost, 0xabcd, false);
        assert!(s.dhcp_configuration(ready).unwrap().is_some());
    });
}

#[test]
fn offered_frame_cannot_be_discarded() {
    with_service(|s| {
        let now = acquire(s, 0, 0x1234, true);
        assert_eq!(
            rpc(s, 0, Operation::SendUdp, 1, 0, now).status(),
            Status::Again
        );
        let pending = s.transmission_at(now).unwrap().unwrap().id;
        assert_eq!(s.withheld_transmission_at(now).unwrap(), None);
        assert!(s.discard_withheld_transmission_at(pending, now).is_err());
        assert_eq!(s.transmission_at(now).unwrap().unwrap().id, pending);
    });
}
