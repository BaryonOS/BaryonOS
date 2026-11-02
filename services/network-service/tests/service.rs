//! Controlled userspace algorithm tests, without a native kernel or NIC.
use nieuw_os_net_protocol::{build_udp_ipv4, UdpIpv4Tx};
use nieuw_os_net_stack::{Config, PacketDevice, SocketStorage};
use nieuw_os_network_service::{
    wire::{ConnectionState, HeaderFields, Operation, Packet, Status, DATA},
    Buffers, Configuration, Destination, Fault, Service,
};
use smoltcp::{
    iface::{Config as IfaceConfig, Interface, SocketSet},
    socket::tcp::{Socket, SocketBuffer},
    time::Instant,
    wire::{EthernetAddress, HardwareAddress, IpCidr, Ipv4Address},
};
const LOCAL: [u8; 4] = [192, 0, 2, 2];
const PEER: [u8; 4] = [192, 0, 2, 1];
fn configuration() -> Configuration {
    Configuration {
        epoch: 19,
        pids: [11, 12],
        link: Config {
            mac: [2, 0, 0, 0, 0, 7],
            ipv4: LOCAL,
            prefix_len: 24,
            random_seed: 7,
        },
        destinations: [
            Destination {
                ipv4: PEER,
                tcp_first: 8080,
                tcp_count: 5,
                udp_port: 9000,
            },
            Destination {
                ipv4: PEER,
                tcp_first: 8090,
                tcp_count: 5,
                udp_port: 9001,
            },
        ],
    }
}
macro_rules! service {
    ($s:ident, $config:expr, $body:block) => {{
        let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
        let mut storage = [SocketStorage::EMPTY, SocketStorage::EMPTY];
        let (mut ra, mut rb, mut ta, mut tb) = ([0; 4096], [0; 4096], [0; 4096], [0; 4096]);
        let mut $s = Service::new(
            $config,
            Buffers {
                storage: &mut storage,
                rx_frame: &mut rx,
                tx_frame: &mut tx,
                rx_sockets: [&mut ra, &mut rb],
                tx_sockets: [&mut ta, &mut tb],
            },
            0,
        )
        .unwrap();
        $body
    }};
}
fn request(
    op: Operation,
    id: u64,
    token: u64,
    arg: u32,
    address: [u8; 4],
    port: u16,
    data: &[u8],
) -> Packet {
    Packet::request(
        HeaderFields {
            operation: op,
            request: id,
            epoch: 19,
            connection: token,
            argument: arg,
            address,
            port,
        },
        data,
    )
    .unwrap()
}
fn rpc(s: &mut Service<'_>, slot: usize, p: &Packet, tick: u64) -> Packet {
    s.handle(slot, 11 + slot as u64, 0, p.bytes(), tick)
        .unwrap()
        .unwrap()
}
#[test]
fn real_sender_epoch_request_and_connection_owner_are_separate_fences() {
    service!(s, configuration(), {
        let a = request(Operation::OpenTcp, 1, 0, 300, PEER, 8080, &[]);
        assert!(s.handle(0, 12, 0, a.bytes(), 0).unwrap().is_none());
        assert!(s.handle(0, 11, 1, a.bytes(), 0).unwrap().is_none());
        assert_eq!(s.counters().opened, 0);
        let mut malformed = *a.bytes();
        malformed[46] = 1;
        assert_eq!(
            s.handle(0, 11, 0, &malformed, 0).unwrap().unwrap().status(),
            Status::Malformed
        );
        assert_eq!(s.counters().opened, 0);
        let granted = rpc(&mut s, 0, &a, 0);
        assert_eq!(granted.status(), Status::Ok);
        let token = granted.header().connection;
        assert_eq!(rpc(&mut s, 0, &a, 0).status(), Status::Replay);
        let p = request(Operation::State, 2, token, 0, [0; 4], 0, &[]);
        assert_eq!(rpc(&mut s, 1, &p, 0).status(), Status::Stale);
        let mut stale = *p.bytes();
        stale[16..24].copy_from_slice(&18u64.to_le_bytes());
        assert_eq!(
            s.handle(0, 11, 0, &stale, 0).unwrap().unwrap().status(),
            Status::Stale
        );
        assert_eq!(rpc(&mut s, 0, &p, 0).status(), Status::Ok);
        assert_eq!(s.counters().opened, 1);
    });
}
#[test]
fn whitelist_close_timeout_and_retire_do_not_grant_authority() {
    service!(s, configuration(), {
        for (id, ip, port) in [(1, PEER, 8090), (2, [192, 0, 2, 3], 8080)] {
            assert_eq!(
                rpc(
                    &mut s,
                    0,
                    &request(Operation::OpenTcp, id, 0, 300, ip, port, &[]),
                    0
                )
                .status(),
                Status::Access
            );
        }
        let a = rpc(
            &mut s,
            0,
            &request(Operation::OpenTcp, 3, 0, 3, PEER, 8080, &[]),
            0,
        );
        let token = a.header().connection;
        assert_eq!(
            rpc(
                &mut s,
                0,
                &request(Operation::OpenTcp, 4, 0, 300, PEER, 8081, &[]),
                0
            )
            .status(),
            Status::Busy
        );
        s.poll(3).unwrap();
        let state = rpc(
            &mut s,
            0,
            &request(Operation::State, 5, token, 0, [0; 4], 0, &[]),
            3,
        );
        assert_eq!(state.header().argument, ConnectionState::TimedOut as u32);
        assert_eq!(
            rpc(
                &mut s,
                0,
                &request(Operation::SendTcp, 6, token, 1, [0; 4], 0, b"x"),
                3
            )
            .status(),
            Status::Closed
        );
        assert_eq!(
            rpc(
                &mut s,
                0,
                &request(Operation::Retire, 7, 0, 0, [0; 4], 0, &[]),
                3
            )
            .status(),
            Status::Ok
        );
        assert_eq!(s.retired_mask(), 1);
        assert_eq!(
            rpc(
                &mut s,
                0,
                &request(Operation::OpenTcp, 8, 0, 300, PEER, 8080, &[]),
                3
            )
            .status(),
            Status::Closed
        );
        assert_eq!(s.poll(2), Err(Fault::Clock));
        assert_eq!(s.ingress(&[0; 14], 2), Err(Fault::Clock));
    });
}
fn datagram(slot: usize, source_port: u16, data: &[u8]) -> Vec<u8> {
    let mut frame = [0; 1514];
    let n = build_udp_ipv4(
        UdpIpv4Tx {
            destination_mac: [2, 0, 0, 0, 0, 7],
            source_mac: [2, 0, 0, 0, 0, 1],
            source_ipv4: PEER,
            destination_ipv4: LOCAL,
            source_port,
            destination_port: 45000 + slot as u16,
            identification: 1,
            ttl: 64,
            mtu: 1500,
        },
        data,
        &mut frame,
    )
    .unwrap();
    frame[..n].to_vec()
}
#[test]
fn udp_corruption_scope_and_full_slot_preserve_first_datagram() {
    service!(s, configuration(), {
        let mut broken = datagram(0, 9000, b"first");
        broken[40] ^= 1;
        s.ingress(&broken, 0).unwrap();
        s.ingress(&datagram(0, 9001, b"wrong-owner"), 0).unwrap();
        s.ingress(&datagram(0, 9000, &[1; DATA + 1]), 0).unwrap();
        s.ingress(&datagram(0, 9000, b"first"), 0).unwrap();
        s.ingress(&datagram(0, 9000, b"overwrite"), 0).unwrap();
        s.ingress(&datagram(1, 9001, b"second-app"), 0).unwrap();
        let p = request(Operation::ReceiveUdp, 1, 0, 0, [0; 4], 0, &[]);
        assert_eq!(rpc(&mut s, 0, &p, 0).data(), b"first");
        assert_eq!(rpc(&mut s, 1, &p, 0).data(), b"second-app");
        assert_eq!(
            rpc(
                &mut s,
                0,
                &request(Operation::ReceiveUdp, 2, 0, 0, [0; 4], 0, &[]),
                0
            )
            .status(),
            Status::Again
        );
        let c = s.counters();
        assert_eq!(c.malformed_frames, 3);
        assert_eq!(c.dropped_datagrams, 1);
        assert_eq!(c.udp_received, 2);
    });
}
#[test]
fn subnet_broadcast_and_network_destination_are_rejected() {
    for last in [0, 255] {
        let mut c = configuration();
        c.destinations[0].ipv4[3] = last;
        let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
        let mut storage = [SocketStorage::EMPTY, SocketStorage::EMPTY];
        let (mut ra, mut rb, mut ta, mut tb) = ([0; 4096], [0; 4096], [0; 4096], [0; 4096]);
        assert!(matches!(
            Service::new(
                c,
                Buffers {
                    storage: &mut storage,
                    rx_frame: &mut rx,
                    tx_frame: &mut tx,
                    rx_sockets: [&mut ra, &mut rb],
                    tx_sockets: [&mut ta, &mut tb]
                },
                0
            ),
            Err(Fault::Configuration)
        ));
    }
}
#[test]
fn two_ipc_owners_fill_real_tcp_buffers_recover_backpressure_and_close() {
    service!(service, configuration(), {
        let (mut rx, mut tx) = ([0; 1514], [0; 1514]);
        let mut dev = PacketDevice::new(&mut rx, &mut tx);
        let mut config = IfaceConfig::new(HardwareAddress::Ethernet(EthernetAddress([
            2, 0, 0, 0, 0, 1,
        ])));
        config.random_seed = 3;
        let mut peer = Interface::new(config, &mut dev, Instant::ZERO);
        peer.update_ip_addrs(|ips| {
            ips.push(IpCidr::new(Ipv4Address::new(192, 0, 2, 1).into(), 24))
                .unwrap()
        });
        let mut storage = [SocketStorage::EMPTY, SocketStorage::EMPTY];
        let mut sockets = SocketSet::new(&mut storage[..]);
        let (mut ra, mut rb, mut ta, mut tb) = ([0; 4096], [0; 4096], [0; 4096], [0; 4096]);
        let mut sa = Socket::new(
            SocketBuffer::new(&mut ra[..]),
            SocketBuffer::new(&mut ta[..]),
        );
        sa.set_ack_delay(None);
        sa.listen(8080).unwrap();
        let mut sb = Socket::new(
            SocketBuffer::new(&mut rb[..]),
            SocketBuffer::new(&mut tb[..]),
        );
        sb.set_ack_delay(None);
        sb.listen(8090).unwrap();
        let handles = [sockets.add(sa), sockets.add(sb)];
        let mut ids = [1u64; 2];
        let mut tokens = [0; 2];
        for slot in 0..2 {
            tokens[slot] = rpc(
                &mut service,
                slot,
                &request(
                    Operation::OpenTcp,
                    ids[slot],
                    0,
                    2000,
                    PEER,
                    8080 + slot as u16 * 10,
                    &[],
                ),
                0,
            )
            .header()
            .connection;
            ids[slot] += 1;
        }
        let mut filled = [false; 2];
        let mut backpressure = [false; 2];
        let mut client_read = [Vec::new(), Vec::new()];
        let mut server_read = [Vec::new(), Vec::new()];
        let mut peer_closed = [false; 2];
        let mut closed = [false; 2];
        for tick in 0..2200 {
            service.poll(tick).unwrap();
            if let Some(frame) = service.transmission() {
                let id = frame.id;
                dev.push(frame.bytes).unwrap();
                service.complete_transmission(id).unwrap();
            }
            let now = Instant::from_micros(tick as i64 * 10_000);
            peer.poll_maintenance(now);
            peer.poll_ingress_single(now, &mut dev, &mut sockets);
            for slot in 0..2 {
                let state = rpc(
                    &mut service,
                    slot,
                    &request(Operation::State, ids[slot], tokens[slot], 0, [0; 4], 0, &[]),
                    tick,
                )
                .header()
                .argument;
                ids[slot] += 1;
                if state == ConnectionState::Established as u32 && !filled[slot] {
                    let body = [b'a' + slot as u8; DATA];
                    let mut sent = 0;
                    for _ in 0..24 {
                        let reply = rpc(
                            &mut service,
                            slot,
                            &request(
                                Operation::SendTcp,
                                ids[slot],
                                tokens[slot],
                                DATA as u32,
                                [0; 4],
                                0,
                                &body,
                            ),
                            tick,
                        );
                        ids[slot] += 1;
                        if reply.status() == Status::Again {
                            backpressure[slot] = true;
                            break;
                        }
                        assert_eq!(reply.status(), Status::Ok);
                        sent += reply.header().argument as usize;
                    }
                    assert_eq!(sent, 4096);
                    filled[slot] = true;
                }
                let socket = sockets.get_mut::<Socket>(handles[slot]);
                if socket.can_recv() {
                    let mut buf = [0; 4096];
                    let n = socket.recv_slice(&mut buf).unwrap();
                    server_read[slot].extend_from_slice(&buf[..n]);
                }
                if server_read[slot].len() == 4096 && !peer_closed[slot] {
                    assert!(server_read[slot].iter().all(|b| *b == b'a' + slot as u8));
                    assert_eq!(socket.send_slice(&server_read[slot]).unwrap(), 4096);
                    socket.close();
                    peer_closed[slot] = true;
                }
                if filled[slot] {
                    let reply = rpc(
                        &mut service,
                        slot,
                        &request(
                            Operation::ReceiveTcp,
                            ids[slot],
                            tokens[slot],
                            0,
                            [0; 4],
                            0,
                            &[],
                        ),
                        tick,
                    );
                    ids[slot] += 1;
                    if reply.status() == Status::Ok {
                        client_read[slot].extend_from_slice(reply.data());
                    }
                }
                if state == ConnectionState::PeerClosed as u32
                    && client_read[slot].len() == 4096
                    && !closed[slot]
                {
                    assert_eq!(
                        rpc(
                            &mut service,
                            slot,
                            &request(
                                Operation::CloseTcp,
                                ids[slot],
                                tokens[slot],
                                0,
                                [0; 4],
                                0,
                                &[]
                            ),
                            tick
                        )
                        .status(),
                        Status::Ok
                    );
                    ids[slot] += 1;
                    closed[slot] = true;
                }
            }
            peer.poll_egress(now, &mut dev, &mut sockets);
            if let Some(frame) = dev.transmission() {
                let id = frame.id;
                service.ingress(frame.bytes, tick).unwrap();
                dev.complete(id).unwrap();
            }
            if closed.iter().all(|x| *x)
                && (0..2).all(|slot| {
                    let r = rpc(
                        &mut service,
                        slot,
                        &request(Operation::State, ids[slot], tokens[slot], 0, [0; 4], 0, &[]),
                        tick,
                    );
                    ids[slot] += 1;
                    r.header().argument == ConnectionState::Closed as u32
                })
            {
                break;
            }
        }
        assert_eq!(backpressure, [true; 2]);
        assert_eq!(closed, [true; 2]);
        assert_eq!(client_read, server_read);
        for (slot, id) in ids.into_iter().enumerate() {
            assert_eq!(
                rpc(
                    &mut service,
                    slot,
                    &request(Operation::Retire, id, 0, 0, [0; 4], 0, &[]),
                    2200
                )
                .status(),
                Status::Ok
            );
        }
        assert!(service.clean());
        assert_eq!(service.counters().sent, 8192);
        assert_eq!(service.counters().received, 8192);
    });
}
