//! In-memory TCP algorithm scenarios with explicit model time. These are not
//! native tests: no broker, NIC, actual clock or host network socket is used.
#[path = "../../../sdk/examples/tcp-probe/src/contract.rs"]
mod contract;
use nieuw_os_net_stack::{Config, Error, PacketDevice, SocketStorage, Stack, State};
use smoltcp::{
    iface::{Config as IfaceConfig, Interface, SocketSet},
    socket::tcp::{Socket, SocketBuffer},
    time::Instant,
    wire::{EthernetAddress, HardwareAddress, IpCidr, Ipv4Address},
};

const REQUEST: &[u8] =
    b"GET /nieuw-os-tcp HTTP/1.1\r\nHost: fixture.invalid\r\nConnection: close\r\n\r\n";
const RESPONSE: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Length: 20\r\nConnection: close\r\n\r\nNieuw OS native TCP\n";
#[derive(Clone, Copy, PartialEq, Eq)]
enum Case {
    Normal,
    Refused,
    Reset,
    Retransmit,
    Timeout,
    MissingFinalAck,
}

fn is_tcp_syn(frame: &[u8]) -> bool {
    frame.len() >= 54
        && frame[12..14] == [8, 0]
        && frame[14] == 0x45
        && frame[23] == 6
        && frame[47] & 0x17 == 2
}

fn run(case: Case) -> (u64, usize, usize) {
    let (mut crx, mut ctx, mut srx, mut stx) = ([0; 1514], [0; 1514], [0; 1514], [0; 1514]);
    let (mut crxb, mut ctxb, mut srxb, mut stxb) = ([0; 4096], [0; 4096], [0; 4096], [0; 4096]);
    let (mut cstorage, mut sstorage) = ([SocketStorage::EMPTY], [SocketStorage::EMPTY]);
    let mut client = Stack::new(
        Config {
            mac: [2, 0, 0, 0, 0, 7],
            ipv4: [192, 0, 2, 2],
            prefix_len: 24,
            random_seed: 7,
        },
        &mut cstorage,
        &mut crx,
        &mut ctx,
        0,
    )
    .unwrap();
    let handle = client.add_client(&mut crxb, &mut ctxb).unwrap();
    client
        .connect(handle, [192, 0, 2, 1], 8080, 40000, 300)
        .unwrap();
    let mut device = PacketDevice::new(&mut srx, &mut stx);
    let mut config = IfaceConfig::new(HardwareAddress::Ethernet(EthernetAddress([
        2, 0, 0, 0, 0, 1,
    ])));
    config.random_seed = 1;
    let mut server = Interface::new(config, &mut device, Instant::ZERO);
    server.update_ip_addrs(|addresses| {
        addresses
            .push(IpCidr::new(Ipv4Address::new(192, 0, 2, 1).into(), 24))
            .unwrap()
    });
    let mut sockets = SocketSet::new(&mut sstorage[..]);
    let mut socket = Socket::new(
        SocketBuffer::new(&mut srxb[..]),
        SocketBuffer::new(&mut stxb[..]),
    );
    socket.set_ack_delay(None);
    if case != Case::Refused {
        socket.listen(8080).unwrap();
    }
    let sh = sockets.add(socket);
    let mut received_request = Vec::new();
    let mut received_response = Vec::new();
    let mut sent_request = 0;
    let mut response_sent = false;
    let mut established = false;
    let mut eof = false;
    let mut syns = Vec::new();
    for ticks in 0..800 {
        client.poll(ticks).unwrap();
        if let Some(frame) = client.transmission() {
            let mut drop = false;
            if is_tcp_syn(frame.bytes) {
                syns.push((ticks, frame.bytes[34..].to_vec()));
                drop = case == Case::Timeout || (case == Case::Retransmit && syns.len() == 1);
            }
            if !drop {
                assert!(device.can_receive());
                device.push(frame.bytes).unwrap();
            }
            let id = frame.id;
            // Model-medium delivery, explicitly not a native TX completion.
            client.complete_transmission(id).unwrap();
        }
        let now = Instant::from_micros(ticks as i64 * 10_000);
        server.poll_maintenance(now);
        server.poll_ingress_single(now, &mut device, &mut sockets);
        let socket = sockets.get_mut::<Socket>(sh);
        if socket.can_recv() {
            let mut out = [0; 256];
            let count = socket.recv_slice(&mut out).unwrap();
            received_request.extend_from_slice(&out[..count]);
        }
        if received_request.len() == REQUEST.len() && !response_sent {
            assert_eq!(received_request, REQUEST);
            if case == Case::Reset {
                socket.abort();
            } else {
                assert_eq!(socket.send_slice(RESPONSE).unwrap(), RESPONSE.len());
                socket.close();
            }
            response_sent = true;
        }
        server.poll_egress(now, &mut device, &mut sockets);
        if let Some(frame) = device.transmission() {
            let drop =
                case == Case::MissingFinalAck && client.state(handle).unwrap() == State::LastAck;
            if !drop {
                assert!(client.can_receive_frame());
                client.push_frame(frame.bytes).unwrap();
            }
            let id = frame.id;
            device.complete(id).unwrap();
        }
        let state = client.state(handle).unwrap();
        if state == State::Established {
            established = true;
        }
        if established && state != State::Closed {
            if state == State::Established && sent_request < REQUEST.len() {
                sent_request += client.send(handle, &REQUEST[sent_request..]).unwrap();
            }
            let mut output = [0; 256];
            match client.receive(handle, &mut output) {
                Ok(count) => received_response.extend_from_slice(&output[..count]),
                Err(Error::Finished) => {}
                other => panic!("unexpected receive {other:?}"),
            }
            if state == State::CloseWait {
                eof = true;
                client.close(handle).unwrap();
            }
        }
        if state == State::Closed && client.transmission().is_none() {
            let http_ok = contract::http_complete(
                contract::HttpEvidence {
                    established,
                    eof,
                    sent: sent_request,
                    received: received_response.len(),
                    elapsed_ticks: ticks,
                },
                REQUEST.len(),
                RESPONSE.len(),
                300,
            );
            match case {
                Case::Normal | Case::Retransmit => {
                    assert!(http_ok);
                    assert_eq!(received_response, RESPONSE);
                    if case == Case::Retransmit {
                        assert_eq!(syns.len(), 2);
                        assert_eq!(syns[0].1, syns[1].1);
                        assert!(syns[1].0 - syns[0].0 >= 100);
                    }
                }
                Case::Refused => {
                    assert!(!established);
                    assert!(ticks < 300);
                    assert!(received_response.is_empty());
                }
                Case::Reset => {
                    assert!(established);
                    assert!(ticks < 300);
                    assert!(received_response.is_empty());
                }
                Case::Timeout => {
                    assert!(!established);
                    assert!(ticks >= 300);
                    assert!(received_response.is_empty());
                }
                Case::MissingFinalAck => {
                    assert!(established && eof);
                    assert_eq!(received_response, RESPONSE);
                    assert!(ticks >= 300, "real model timeout closes LastAck");
                    assert!(
                        !http_ok,
                        "timeout-closed LastAck must not count as completed FIN/ACK"
                    );
                }
            }
            return (ticks, syns.len(), received_response.len());
        }
    }
    panic!("model did not complete");
}

#[test]
fn actual_tcp_http_bytes_and_orderly_eof() {
    assert_eq!(run(Case::Normal).2, RESPONSE.len());
}
#[test]
fn refusal_is_observed_before_timeout() {
    run(Case::Refused);
}
#[test]
fn reset_after_request_does_not_produce_http_success() {
    run(Case::Reset);
}
#[test]
fn dropped_syn_retransmits_same_sequence_after_real_model_elapsed_time() {
    run(Case::Retransmit);
}
#[test]
fn unreachable_server_times_out_only_after_model_clock_advances() {
    run(Case::Timeout);
}

#[test]
fn lost_final_ack_cannot_turn_http_eof_into_success_via_timeout() {
    run(Case::MissingFinalAck);
}
