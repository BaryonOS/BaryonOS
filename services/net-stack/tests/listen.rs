//! Packet/model tests using two bounded engines and explicit model time.
//! No native clock, NIC, grant, authenticated identity or host socket is used.
use nieuw_os_net_stack::{Config, Error, RemoteEndpoint, SocketStorage, Stack, State};

const REQUEST: &[u8] = b"bounded passive request: 0123456789";
const RESPONSE: &[u8] = b"bounded passive response: 9876543210";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Case {
    Exchange,
    HalfOpen,
    WrongPort,
    LateSyn,
}

fn run(case: Case) {
    let (mut crx, mut ctx, mut srx, mut stx) = ([0; 1514], [0; 1514], [0; 1514], [0; 1514]);
    let (mut crxb, mut ctxb, mut srxb, mut stxb) = ([0; 64], [0; 64], [0; 8], [0; 8]);
    let (mut cstorage, mut sstorage) = ([SocketStorage::EMPTY], [SocketStorage::EMPTY]);
    let mut client = Stack::new(
        Config {
            mac: [2, 0, 0, 0, 0, 1],
            ipv4: [192, 0, 2, 1],
            prefix_len: 24,
            random_seed: 1,
        },
        &mut cstorage,
        &mut crx,
        &mut ctx,
        0,
    )
    .unwrap();
    let mut server = Stack::new(
        Config {
            mac: [2, 0, 0, 0, 0, 7],
            ipv4: [192, 0, 2, 2],
            prefix_len: 24,
            random_seed: 7,
        },
        &mut sstorage,
        &mut srx,
        &mut stx,
        0,
    )
    .unwrap();
    let active = client.add_client(&mut crxb, &mut ctxb).unwrap();
    let passive = server.add_client(&mut srxb, &mut stxb).unwrap();
    server.listen(passive, 8080, 200).unwrap();
    client
        .connect(
            active,
            [192, 0, 2, 2],
            if case == Case::WrongPort { 8081 } else { 8080 },
            40000,
            600,
        )
        .unwrap();
    let mut sent = 0;
    let mut response_sent = 0;
    let mut received = Vec::new();
    let mut response = Vec::new();
    let mut partial = false;
    let mut established = false;
    let mut half_open = false;
    let mut late_syn = false;
    let mut closed_tx = false;
    let mut eof = false;
    for ticks in 0..800 {
        client.poll(ticks).unwrap();
        if let Some(frame) = client.transmission() {
            let tcp =
                frame.bytes.len() >= 54 && frame.bytes[12..14] == [8, 0] && frame.bytes[23] == 6;
            let drop_ack = case == Case::HalfOpen && tcp && frame.bytes[47] & 0x17 == 0x10;
            let delay = case == Case::LateSyn && ticks < 200;
            if !drop_ack && !delay {
                if tcp && frame.bytes[47] & 0x17 == 2 && ticks >= 200 {
                    late_syn = true;
                }
                server.push_frame(frame.bytes).unwrap();
            }
            let id = frame.id;
            // Explicit model delivery, never a native device completion.
            client.complete_transmission(id).unwrap();
        }
        server.poll(ticks).unwrap();
        if let Some(frame) = server.transmission() {
            client.push_frame(frame.bytes).unwrap();
            let id = frame.id;
            server.complete_transmission(id).unwrap();
        }
        let state = server.state(passive).unwrap();
        half_open |= state == State::SynReceived;
        if state == State::Established {
            established = true;
            assert_eq!(
                server.remote_endpoint(passive).unwrap(),
                Some(RemoteEndpoint {
                    ipv4: [192, 0, 2, 1],
                    port: 40000
                })
            );
        }
        match case {
            Case::Exchange => {
                if client.state(active).unwrap() == State::Established && sent < REQUEST.len() {
                    sent += client.send(active, &REQUEST[sent..]).unwrap();
                }
                if established && !matches!(state, State::Closed | State::TimeWait) {
                    let mut out = [0; 8];
                    match server.receive(passive, &mut out) {
                        Ok(length) => received.extend_from_slice(&out[..length]),
                        Err(Error::Finished) => {}
                        other => panic!("unexpected passive receive {other:?}"),
                    }
                    if received.len() == REQUEST.len() && !closed_tx && ticks >= 210 {
                        assert_eq!(received, REQUEST);
                        let remaining = RESPONSE.len() - response_sent;
                        let length = server.send(passive, &RESPONSE[response_sent..]).unwrap();
                        partial |= length < remaining;
                        response_sent += length;
                        if response_sent == RESPONSE.len() {
                            server.close(passive).unwrap();
                            closed_tx = true;
                        }
                    }
                }
                if established && client.state(active).unwrap() != State::Closed {
                    let mut out = [0; 64];
                    match client.receive(active, &mut out) {
                        Ok(length) => response.extend_from_slice(&out[..length]),
                        Err(Error::Finished) => {}
                        other => panic!("unexpected active receive {other:?}"),
                    }
                    if client.state(active).unwrap() == State::CloseWait {
                        eof = true;
                        client.close(active).unwrap();
                    }
                }
                if eof && client.state(active).unwrap() == State::Closed {
                    assert_eq!(sent, REQUEST.len());
                    assert_eq!(received, REQUEST);
                    assert_eq!(response, RESPONSE);
                    assert!(partial, "8-byte server TX enforces partial writes");
                    assert!(
                        ticks > 200,
                        "accept deadline must not kill established transfers"
                    );
                    return;
                }
            }
            Case::HalfOpen if ticks == 200 => {
                assert!(half_open && !established);
                assert_eq!(state, State::Closed);
                assert_eq!(server.remote_endpoint(passive).unwrap(), None);
                assert_eq!(server.send(passive, b"forbidden"), Err(Error::InvalidState));
                server.listen(passive, 8080, 100).unwrap();
                assert_eq!(server.state(passive).unwrap(), State::Listen);
                return;
            }
            Case::WrongPort if client.state(active).unwrap() == State::Closed => {
                assert!(!established && !half_open);
                assert_eq!(state, State::Listen);
                assert_eq!(server.remote_endpoint(passive).unwrap(), None);
                return;
            }
            Case::LateSyn if client.state(active).unwrap() == State::Closed => {
                assert!(!established && !half_open && late_syn);
                assert!(ticks >= 200);
                assert_eq!(state, State::Closed);
                return;
            }
            _ => {}
        }
    }
    panic!("passive model did not complete");
}

#[test]
fn passive_connection_transfers_exact_bytes_under_backpressure_and_closes() {
    run(Case::Exchange);
}

#[test]
fn missing_handshake_ack_expires_half_open_at_absolute_deadline() {
    run(Case::HalfOpen);
}

#[test]
fn wrong_destination_port_does_not_consume_listener() {
    run(Case::WrongPort);
}

#[test]
fn syn_after_accept_deadline_cannot_reopen_expired_listener() {
    run(Case::LateSyn);
}
