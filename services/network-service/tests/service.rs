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
        service.poll(2200).unwrap();
        assert!(service.clean());
        assert_eq!(service.counters().sent, 8192);
        assert_eq!(service.counters().received, 8192);
    });
}

#[test]
fn one_owner_port_exhaustion_preserves_the_other_owners_budget() {
    service!(s, configuration(), {
        for turn in 0..12768u64 {
            let opened = rpc(
                &mut s,
                0,
                &request(Operation::OpenTcp, turn * 2 + 1, 0, 300, PEER, 8080, &[]),
                0,
            );
            assert_eq!(opened.status(), Status::Ok);
            assert_eq!(
                rpc(
                    &mut s,
                    0,
                    &request(
                        Operation::CloseTcp,
                        turn * 2 + 2,
                        opened.header().connection,
                        0,
                        [0; 4],
                        0,
                        &[]
                    ),
                    0
                )
                .status(),
                Status::Ok
            );
        }
        assert_eq!(
            rpc(
                &mut s,
                0,
                &request(Operation::OpenTcp, 25537, 0, 300, PEER, 8080, &[]),
                0
            )
            .status(),
            Status::Exhausted
        );
        let b = rpc(
            &mut s,
            1,
            &request(Operation::OpenTcp, 1, 0, 300, PEER, 8090, &[]),
            0,
        );
        assert_eq!(b.status(), Status::Ok);
        assert_eq!(
            rpc(
                &mut s,
                1,
                &request(
                    Operation::State,
                    2,
                    b.header().connection,
                    0,
                    [0; 4],
                    0,
                    &[]
                ),
                0
            )
            .header()
            .argument,
            ConnectionState::Connecting as u32
        );
        assert_eq!(s.counters().opened, 12769);
    });
}

#[test]
fn terminal_channel_failure_retires_one_slot_without_releasing_live_transmission() {
    service!(s, configuration(), {
        for slot in 0..2 {
            assert_eq!(
                rpc(
                    &mut s,
                    slot,
                    &request(
                        Operation::OpenTcp,
                        1,
                        0,
                        300,
                        PEER,
                        8080 + slot as u16 * 10,
                        &[]
                    ),
                    0
                )
                .status(),
                Status::Ok
            );
        }
        s.poll(0).unwrap();
        let frame = s.transmission().unwrap();
        let (id, bytes) = (frame.id, frame.bytes.to_vec());
        s.retire_slot(0, 0).unwrap();
        assert_eq!(s.retired_mask(), 1);
        assert_eq!(s.transmission().unwrap().id, id);
        assert_eq!(s.transmission().unwrap().bytes, bytes);
        assert_eq!(s.complete_transmission(id + 1), Err(Fault::Transport));
        s.complete_transmission(id).unwrap();
        s.poll(1).unwrap();
        assert_eq!(
            rpc(
                &mut s,
                0,
                &request(Operation::OpenTcp, 2, 0, 300, PEER, 8080, &[]),
                1
            )
            .status(),
            Status::Closed
        );
        assert_eq!(
            rpc(
                &mut s,
                1,
                &request(Operation::State, 2, 258, 0, [0; 4], 0, &[]),
                1
            )
            .status(),
            Status::Ok
        );
    });
}

/// One controlled smoltcp peer listening on `PEER:8080` for slot 0.
macro_rules! peer {
    ($dev:ident, $peer:ident, $sockets:ident, $handle:ident, $body:block) => {{
        let (mut prx, mut ptx) = ([0; 1514], [0; 1514]);
        let mut $dev = PacketDevice::new(&mut prx, &mut ptx);
        let mut config = IfaceConfig::new(HardwareAddress::Ethernet(EthernetAddress([
            2, 0, 0, 0, 0, 1,
        ])));
        config.random_seed = 3;
        let mut $peer = Interface::new(config, &mut $dev, Instant::ZERO);
        $peer.update_ip_addrs(|ips| {
            ips.push(IpCidr::new(Ipv4Address::new(192, 0, 2, 1).into(), 24))
                .unwrap()
        });
        let mut pstorage = [SocketStorage::EMPTY];
        let mut $sockets = SocketSet::new(&mut pstorage[..]);
        let (mut pr, mut pt) = ([0; 4096], [0; 4096]);
        let mut socket = Socket::new(
            SocketBuffer::new(&mut pr[..]),
            SocketBuffer::new(&mut pt[..]),
        );
        socket.set_ack_delay(None);
        socket.listen(8080).unwrap();
        let $handle = $sockets.add(socket);
        $body
    }};
}
/// Exchange at most one frame per direction between the service and the peer.
fn step(
    service: &mut Service<'_>,
    dev: &mut PacketDevice<'_>,
    peer: &mut Interface,
    sockets: &mut SocketSet<'_>,
    tick: u64,
    echo: Option<smoltcp::iface::SocketHandle>,
) {
    service.poll(tick).unwrap();
    if let Some(frame) = service.transmission() {
        let id = frame.id;
        dev.push(frame.bytes).unwrap();
        service.complete_transmission(id).unwrap();
    }
    let now = Instant::from_micros(tick as i64 * 10_000);
    peer.poll_maintenance(now);
    peer.poll_ingress_single(now, dev, sockets);
    if let Some(handle) = echo {
        let socket = sockets.get_mut::<Socket>(handle);
        if socket.can_recv() && socket.can_send() {
            let mut buf = [0; 64];
            let n = socket.recv_slice(&mut buf).unwrap();
            assert_eq!(socket.send_slice(&buf[..n]).unwrap(), n);
        }
    }
    peer.poll_egress(now, dev, sockets);
    if let Some(frame) = dev.transmission() {
        let id = frame.id;
        service.ingress(frame.bytes, tick).unwrap();
        dev.complete(id).unwrap();
    }
}
fn tcp_state(s: &mut Service<'_>, id: &mut u64, token: u64, tick: u64) -> u32 {
    *id += 1;
    rpc(
        s,
        0,
        &request(Operation::State, *id, token, 0, [0; 4], 0, &[]),
        tick,
    )
    .header()
    .argument
}
fn tcp_rpc(
    s: &mut Service<'_>,
    id: &mut u64,
    op: Operation,
    token: u64,
    data: &[u8],
    tick: u64,
) -> Packet {
    *id += 1;
    rpc(
        s,
        0,
        &request(op, *id, token, data.len() as u32, [0; 4], 0, data),
        tick,
    )
}
/// A connect timeout is terminal for reads, just as it is for writes. The
/// diagnostic TimedOut state must not turn an aborted socket into a retry loop.
#[test]
fn receive_after_connect_timeout_reports_terminal_closed() {
    service!(s, configuration(), {
        let mut id = 1;
        let token = rpc(
            &mut s,
            0,
            &request(Operation::OpenTcp, id, 0, 3, PEER, 8080, &[]),
            0,
        )
        .header()
        .connection;
        assert_eq!(
            tcp_rpc(&mut s, &mut id, Operation::ReceiveTcp, token, &[], 0).status(),
            Status::Again
        );
        for tick in 3..=5 {
            s.poll(tick).unwrap();
            assert_eq!(
                tcp_state(&mut s, &mut id, token, tick),
                ConnectionState::TimedOut as u32
            );
            let received = tcp_rpc(&mut s, &mut id, Operation::ReceiveTcp, token, &[], tick);
            assert_eq!(received.status(), Status::Closed);
            assert!(received.data().is_empty());
        }
        assert_eq!(s.counters().received, 0);
        assert_eq!(s.counters().rejected, 0);
        assert_eq!(
            tcp_rpc(&mut s, &mut id, Operation::ReceiveTcp, token ^ 1, &[], 5).status(),
            Status::Stale
        );
        // Reopening resets the timeout diagnostic and changes the token. A
        // newly connecting socket still asks the caller to retry its read.
        id += 1;
        let reopened = rpc(
            &mut s,
            0,
            &request(Operation::OpenTcp, id, 0, 300, PEER, 8080, &[]),
            5,
        );
        assert_eq!(reopened.status(), Status::Ok);
        let next_token = reopened.header().connection;
        assert_ne!(token, next_token);
        assert_eq!(
            tcp_rpc(&mut s, &mut id, Operation::ReceiveTcp, token, &[], 5).status(),
            Status::Stale
        );
        assert_eq!(
            tcp_state(&mut s, &mut id, next_token, 5),
            ConnectionState::Connecting as u32
        );
        assert_eq!(
            tcp_rpc(&mut s, &mut id, Operation::ReceiveTcp, next_token, &[], 5).status(),
            Status::Again
        );
    });
}
/// PR #127 regression: the OpenTcp deadline bounds only connection setup. An
/// established connection with ongoing traffic must survive well past it, and
/// an idle one is still ended by the stack's own inactivity timeout.
#[test]
fn established_connection_with_traffic_outlives_connect_deadline() {
    const DEADLINE: u32 = 30;
    service!(s, configuration(), {
        peer!(dev, peer, sockets, handle, {
            let mut id = 1;
            let token = rpc(
                &mut s,
                0,
                &request(Operation::OpenTcp, id, 0, DEADLINE, PEER, 8080, &[]),
                0,
            )
            .header()
            .connection;
            let (mut established_at, mut echoed, mut last_echo) = (None, 0, 0);
            for tick in 0..=200 {
                step(
                    &mut s,
                    &mut dev,
                    &mut peer,
                    &mut sockets,
                    tick,
                    Some(handle),
                );
                let state = tcp_state(&mut s, &mut id, token, tick);
                assert_ne!(state, ConnectionState::TimedOut as u32, "tick {tick}");
                if state != ConnectionState::Established as u32 {
                    assert!(established_at.is_none(), "dropped at tick {tick}: {state}");
                    continue;
                }
                if established_at.is_none() {
                    established_at = Some(tick);
                }
                if tick % 5 == 0 {
                    let sent = tcp_rpc(&mut s, &mut id, Operation::SendTcp, token, b"p", tick);
                    assert_eq!(sent.status(), Status::Ok, "tick {tick}");
                }
                let got = tcp_rpc(&mut s, &mut id, Operation::ReceiveTcp, token, &[], tick);
                if got.status() == Status::Ok {
                    assert!(got.data().iter().all(|b| *b == b'p'));
                    echoed += got.data().len();
                    last_echo = tick;
                }
            }
            assert!(established_at.unwrap() < u64::from(DEADLINE));
            assert!(last_echo > 3 * u64::from(DEADLINE));
            assert!(echoed > 30);
            assert_eq!(
                tcp_state(&mut s, &mut id, token, 200),
                ConnectionState::Established as u32
            );
            // Without traffic the stack inactivity timeout still applies: it is
            // reported as Closed (not a service TimedOut) and sending is terminal.
            let mut closed_at = None;
            for tick in 201..=400 {
                step(
                    &mut s,
                    &mut dev,
                    &mut peer,
                    &mut sockets,
                    tick,
                    Some(handle),
                );
                let state = tcp_state(&mut s, &mut id, token, tick);
                assert_ne!(state, ConnectionState::TimedOut as u32, "tick {tick}");
                if state == ConnectionState::Closed as u32 {
                    closed_at = Some(tick);
                    break;
                }
            }
            let closed_at = closed_at.expect("idle connection must hit stack inactivity timeout");
            assert_eq!(
                tcp_rpc(
                    &mut s,
                    &mut id,
                    Operation::SendTcp,
                    token,
                    b"late",
                    closed_at
                )
                .status(),
                Status::Closed
            );
        });
    });
}
/// PR #127 regression: a reset connection is terminal for SendTcp, analogous
/// to ReceiveTcp, while a still-connecting socket only asks to retry.
#[test]
fn send_after_peer_reset_reports_terminal_closed() {
    service!(s, configuration(), {
        peer!(dev, peer, sockets, handle, {
            let mut id = 1;
            let token = rpc(
                &mut s,
                0,
                &request(Operation::OpenTcp, id, 0, 3000, PEER, 8080, &[]),
                0,
            )
            .header()
            .connection;
            assert_eq!(
                tcp_rpc(&mut s, &mut id, Operation::SendTcp, token, b"early", 0).status(),
                Status::Again
            );
            let mut tick = 0;
            while tcp_state(&mut s, &mut id, token, tick) != ConnectionState::Established as u32 {
                step(&mut s, &mut dev, &mut peer, &mut sockets, tick, None);
                tick += 1;
                assert!(tick < 100, "handshake did not complete");
            }
            sockets.get_mut::<Socket>(handle).abort();
            while tcp_state(&mut s, &mut id, token, tick) != ConnectionState::Closed as u32 {
                step(&mut s, &mut dev, &mut peer, &mut sockets, tick, None);
                tick += 1;
                assert!(tick < 200, "reset was not observed");
            }
            let counters = s.counters();
            for (op, data) in [
                (Operation::SendTcp, &b"x"[..]),
                (Operation::ReceiveTcp, &[][..]),
            ] {
                assert_eq!(
                    tcp_rpc(&mut s, &mut id, op, token, data, tick).status(),
                    Status::Closed
                );
            }
            assert_eq!(s.counters().rejected, counters.rejected);
            assert_eq!(s.counters().sent, counters.sent);
            // Terminal status grants nothing: a foreign token stays stale.
            assert_eq!(
                tcp_rpc(&mut s, &mut id, Operation::SendTcp, token ^ 1, b"x", tick).status(),
                Status::Stale
            );
        });
    });
}

/// N04: a trusted revocation aborts the live connection on the wire and every
/// later open/data operation of that owner fails with Access, while the other
/// owner keeps its own grant and cleanup operations remain available.
#[test]
fn revocation_resets_live_connection_and_denies_further_use() {
    service!(s, configuration(), {
        peer!(dev, peer, sockets, handle, {
            let mut id = 1;
            let token = rpc(
                &mut s,
                0,
                &request(Operation::OpenTcp, id, 0, 3000, PEER, 8080, &[]),
                0,
            )
            .header()
            .connection;
            let mut tick = 0;
            while tcp_state(&mut s, &mut id, token, tick) != ConnectionState::Established as u32 {
                step(&mut s, &mut dev, &mut peer, &mut sockets, tick, None);
                tick += 1;
                assert!(tick < 100, "handshake did not complete");
            }
            assert_eq!(
                tcp_rpc(&mut s, &mut id, Operation::SendTcp, token, b"hi", tick).status(),
                Status::Ok
            );
            while sockets.get_mut::<Socket>(handle).recv_queue() < 2 {
                step(&mut s, &mut dev, &mut peer, &mut sockets, tick, None);
                tick += 1;
                assert!(tick < 200, "data did not reach the peer");
            }
            assert_eq!(s.revoked_mask(), 0);
            s.revoke(0, tick).unwrap();
            assert_eq!(s.revoked_mask(), 1);
            while sockets.get_mut::<Socket>(handle).is_active() {
                step(&mut s, &mut dev, &mut peer, &mut sockets, tick, None);
                tick += 1;
                assert!(tick < 300, "peer never observed the reset");
            }
            assert_eq!(
                tcp_state(&mut s, &mut id, token, tick),
                ConnectionState::Closed as u32
            );
            let rejected = s.counters().rejected;
            let (sent, opened) = (s.counters().sent, s.counters().opened);
            for (op, port, data) in [
                (Operation::SendTcp, 0, &b"x"[..]),
                (Operation::ReceiveTcp, 0, &[][..]),
                (Operation::OpenTcp, 8081, &[][..]),
                (Operation::SendUdp, 9000, &b"u"[..]),
                (Operation::ReceiveUdp, 0, &[][..]),
            ] {
                id += 1;
                let address = if port == 0 { [0; 4] } else { PEER };
                let t = if matches!(op, Operation::SendTcp | Operation::ReceiveTcp) {
                    token
                } else {
                    0
                };
                let argument = if op == Operation::OpenTcp {
                    300
                } else {
                    data.len() as u32
                };
                assert_eq!(
                    rpc(
                        &mut s,
                        0,
                        &request(op, id, t, argument, address, port, data),
                        tick
                    )
                    .status(),
                    Status::Access,
                    "{op:?} after revocation"
                );
            }
            assert_eq!(s.counters().rejected, rejected + 5);
            assert_eq!((s.counters().sent, s.counters().opened), (sent, opened));
            assert!(
                s.transmission().is_none(),
                "denied requests emitted traffic"
            );
            // The other owner's grant is independent of the revocation.
            assert_eq!(
                rpc(
                    &mut s,
                    1,
                    &request(Operation::OpenTcp, 1, 0, 300, PEER, 8090, &[]),
                    tick
                )
                .status(),
                Status::Ok
            );
            for (op, t) in [(Operation::CloseTcp, token), (Operation::Retire, 0)] {
                assert_eq!(
                    tcp_rpc(&mut s, &mut id, op, t, &[], tick).status(),
                    Status::Ok
                );
            }
            assert_eq!(s.retired_mask(), 1);
            assert_eq!(s.revoke(2, tick), Err(Fault::Configuration));
        });
    });
}

/// Revocation also drops a datagram already buffered for the owner and stops
/// accepting new ones; nothing reaches a later ReceiveUdp.
#[test]
fn revocation_drops_buffered_and_later_udp() {
    service!(s, configuration(), {
        s.ingress(&datagram(0, 9000, b"before"), 0).unwrap();
        assert_eq!(s.counters().udp_received, 1);
        s.revoke(0, 0).unwrap();
        let malformed = s.counters().malformed_frames;
        s.ingress(&datagram(0, 9000, b"after"), 0).unwrap();
        assert_eq!(s.counters().udp_received, 1);
        assert_eq!(s.counters().malformed_frames, malformed + 1);
        let reply = rpc(
            &mut s,
            0,
            &request(Operation::ReceiveUdp, 1, 0, 0, [0; 4], 0, &[]),
            0,
        );
        assert_eq!(reply.status(), Status::Access);
        assert!(reply.data().is_empty());
    });
}
