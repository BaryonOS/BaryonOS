#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Independent app: actual IPC to network-service, no raw NIC/entropy grant.
#[path = "../../system/src/support.rs"]
mod support;
use nieuw_os_catalog_client::NativeTransport;
use nieuw_os_net_client::{Connection, ConnectionState, Error, Read, Session, Status, DATA};
use nieuw_os_network_service::wire::{self, HeaderFields, Operation, Packet};
use nieuw_os_sdk_runtime::{
    native, Handle, NetworkSource, Outgoing, ReceiveEndpoint, SendEndpoint,
};
const REQUEST_A: &[u8] =
    b"GET /app-a HTTP/1.1\r\nHost: fixture.invalid\r\nConnection: close\r\n\r\n";
const REQUEST_B: &[u8] =
    b"GET /app-b HTTP/1.1\r\nHost: fixture.invalid\r\nConnection: close\r\n\r\n";
const RESPONSE_A: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Length: 19\r\nConnection: close\r\n\r\nNieuw OS service A\n";
const RESPONSE_B: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Length: 19\r\nConnection: close\r\n\r\nNieuw OS service B\n";
fn fail(code: u64) -> ! {
    support::put(240, code);
    native::exit(1)
}
fn check(ok: bool, code: u64) {
    if !ok {
        fail(code)
    }
}
fn bump(offset: u64) {
    support::put(offset, support::word(offset).saturating_add(1))
}
fn tick() -> u64 {
    native::monotonic_ticks().unwrap_or_else(|_| fail(2))
}
fn yield_now() {
    native::yield_now().unwrap_or_else(|_| fail(3))
}
fn h(offset: u64) -> Handle {
    Handle::from_raw(support::word(offset)).unwrap_or_else(|_| fail(4))
}
fn again(error: Error) -> bool {
    matches!(error, Error::Service(Status::Again | Status::Busy))
}
fn malformed(send: SendEndpoint, receive: ReceiveEndpoint, peer: u64, epoch: u64) {
    let expected = HeaderFields {
        operation: Operation::State,
        request: 1,
        epoch,
        connection: 257,
        argument: 0,
        address: [0; 4],
        port: 0,
    };
    let mut body = *Packet::request(expected, &[]).unwrap().bytes();
    body[46] = 1;
    let message = Outgoing::new(wire::TAG, &body).unwrap();
    let start = tick();
    loop {
        check(tick() - start < 500, 10);
        match native::send(send, &message) {
            Ok(()) => break,
            Err(e) if e.is_again() => yield_now(),
            Err(_) => fail(11),
        }
    }
    loop {
        check(tick() - start < 500, 12);
        match native::receive(receive) {
            Ok(reply) => {
                check(
                    reply.sender() == peer && reply.tag() == wire::TAG && reply.cap_count() == 0,
                    13,
                );
                let p =
                    Packet::decode_response(reply.payload(), expected).unwrap_or_else(|_| fail(14));
                check(p.status() == Status::Malformed, 15);
                bump(440);
                return;
            }
            Err(e) if e.is_again() => yield_now(),
            Err(_) => fail(16),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Case {
    Http,
    Refused,
    Reset,
    Retransmit,
    Timeout,
}
fn exercise(
    session: &mut Session<NativeTransport>,
    slot: usize,
    port: u16,
    case: Case,
    previous: Option<Connection>,
) -> Connection {
    let start = tick();
    let timeout = if case == Case::Timeout { 300 } else { 700 };
    let connection = loop {
        check(tick() - start < timeout + 300, 20);
        match session.open([192, 0, 2, 1], port, timeout as u32) {
            Ok(c) => break c,
            Err(e) if again(e) => yield_now(),
            Err(_) => fail(21),
        }
    };
    if let Some(old) = previous {
        check(session.state(old) == Err(Error::Service(Status::Stale)), 22);
        bump(432);
        bump(472);
    }
    let wrong = Connection::from_id(connection.id() ^ 3).unwrap();
    check(
        session.state(wrong) == Err(Error::Service(Status::Stale)),
        23,
    );
    bump(432);
    let request = if slot == 0 { REQUEST_A } else { REQUEST_B };
    let response = if slot == 0 { RESPONSE_A } else { RESPONSE_B };
    let mut established = false;
    let mut sent = 0;
    let mut received = 0;
    let mut close_requested = false;
    let mut out = [0; DATA];
    loop {
        let now = tick();
        check(now >= start && now - start < timeout + 300, 24);
        let state = session.state(connection).unwrap_or_else(|_| fail(25));
        if state == ConnectionState::Established {
            established = true;
        }
        if state == ConnectionState::Established && sent < request.len() {
            match session.send(connection, &request[sent..]) {
                Ok(n) => {
                    sent += n;
                    bump(448)
                }
                Err(e) if again(e) => {}
                Err(_) => fail(26),
            }
        }
        if established
            && matches!(
                state,
                ConnectionState::Established
                    | ConnectionState::PeerClosed
                    | ConnectionState::Closing
            )
        {
            match session.receive(connection, &mut out) {
                Ok(Read::Data(n)) => {
                    check(matches!(case, Case::Http | Case::Retransmit), 27);
                    let end = received + n;
                    check(
                        end <= response.len() && out[..n] == response[received..end],
                        28,
                    );
                    received = end;
                    support::put(456, received as u64)
                }
                Ok(Read::End) => {}
                Err(e) if again(e) => {}
                Err(_) => fail(29),
            }
        }
        if state == ConnectionState::PeerClosed && !close_requested {
            check(sent == request.len() && received == response.len(), 30);
            session.close(connection).unwrap_or_else(|_| fail(31));
            close_requested = true;
            bump(464);
        }
        if matches!(state, ConnectionState::Closed | ConnectionState::TimedOut) {
            // The RPC can cross a clock tick/deadline after the loop's first
            // observation. Measure the completed state exchange, not its start.
            let observed = tick();
            check(observed >= now && observed - start < timeout + 300, 24);
            match case {
                Case::Http | Case::Retransmit => {
                    check(
                        established
                            && close_requested
                            && sent == request.len()
                            && received == response.len(),
                        32,
                    );
                    if case == Case::Retransmit {
                        check(observed - start >= 100, 33);
                        bump(504)
                    }
                    bump(408)
                }
                Case::Refused => {
                    check(
                        !established && sent == 0 && received == 0 && observed - start < timeout,
                        34,
                    );
                    bump(512)
                }
                Case::Reset => {
                    check(
                        established
                            && sent == request.len()
                            && received == 0
                            && observed - start < timeout,
                        35,
                    );
                    bump(488)
                }
                Case::Timeout => {
                    check(
                        !established && sent == 0 && received == 0 && observed - start >= timeout,
                        36,
                    );
                    for _ in 0..2 {
                        check(session.receive(connection, &mut out) == Ok(Read::End), 56);
                        bump(520);
                    }
                    bump(496)
                }
            }
            return connection;
        }
        yield_now();
    }
}
/// N04 policy authority: a separate process whose only capability is the
/// service's private control endpoint. It holds no network, entropy or app
/// endpoint. The kernel test driver writes word 104 once app A is live.
fn policy() -> ! {
    let epoch = support::word(40);
    let control = SendEndpoint::from_handle(h(48));
    let target = support::word(72);
    check(epoch != 0 && support::word(64) != 0 && target < 2, 80);
    let mut raw = [0; 1514];
    check(
        native::net_receive(NetworkSource::from_handle(control.handle()), &mut raw).is_err(),
        81,
    );
    support::put(400, 1);
    let start = tick();
    while support::word(104) == 0 {
        check(tick() - start < 3000, 82);
        yield_now();
    }
    check(support::word(104) == 1, 83);
    let mut body = [0; 24];
    body[..8].copy_from_slice(b"NETRVK01");
    body[8..16].copy_from_slice(&epoch.to_le_bytes());
    body[16..].copy_from_slice(&target.to_le_bytes());
    let message = Outgoing::new(u64::from_le_bytes(*b"NETCTL01"), &body).unwrap();
    // Sampled before each attempt: the service cannot handle the message
    // before the successful send, even if this process is preempted after it.
    let sent = loop {
        let at = tick();
        check(at - start < 3000, 84);
        match native::send(control, &message) {
            Ok(()) => break at,
            Err(e) if e.is_again() => yield_now(),
            Err(_) => fail(85),
        }
    };
    support::put(544, 1);
    support::put(552, sent);
    native::close(control.handle()).unwrap_or_else(|_| fail(86));
    support::put(528, 1);
    native::exit(0)
}
/// N04 revoke case, app A only: the policy process withdraws this app's grant
/// while its connection is live. The connection must end promptly and every
/// later network use must be denied without new traffic on the wire.
fn revoked(session: &mut Session<NativeTransport>) -> ! {
    let denied = Err(Error::Service(Status::Access));
    // Another app's destination is never reachable through this app's grant.
    check(
        session.open([192, 0, 2, 1], 8090, 300).map(|_| ()) == denied,
        61,
    );
    bump(424);
    let start = tick();
    let connection = loop {
        check(tick() - start < 1000, 62);
        match session.open([192, 0, 2, 1], 8080, 3000) {
            Ok(c) => break c,
            Err(e) if again(e) => yield_now(),
            Err(_) => fail(63),
        }
    };
    let mut sent = 0;
    while sent < REQUEST_A.len() {
        check(tick() - start < 1000, 64);
        match session.state(connection) {
            Ok(ConnectionState::Established) => {
                match session.send(connection, &REQUEST_A[sent..]) {
                    Ok(n) => {
                        sent += n;
                        bump(448)
                    }
                    Err(e) if again(e) => {}
                    Err(_) => fail(65),
                }
            }
            Ok(ConnectionState::Connecting) => {}
            _ => fail(66),
        }
        yield_now();
    }
    // SendTcp only queued the bytes; give the service time to put them on the
    // wire so the revocation hits a held connection, not a pending send.
    // shortcut: fixed 10-tick settle, upgrade to a send-queue-empty state if one is added.
    let queued = tick();
    while tick() - queued < 10 {
        yield_now();
    }
    // Live and idle: the peer holds the response. Ask for the revocation.
    support::put(544, 1);
    let ready = tick();
    loop {
        // Far below the 3000-tick connection timeout: only revocation ends it.
        check(tick() - ready < 500, 67);
        match session.state(connection) {
            Ok(ConnectionState::Closed) => break,
            Ok(ConnectionState::Established) => yield_now(),
            _ => fail(68),
        }
    }
    let closed = tick();
    support::put(504, closed - ready);
    support::put(552, closed);
    let mut out = [0; DATA];
    for (ok, code) in [
        (session.send(connection, b"x").map(|_| ()) == denied, 69),
        (
            session.receive(connection, &mut out).map(|_| ()) == denied,
            70,
        ),
        (
            session.open([192, 0, 2, 1], 8081, 300).map(|_| ()) == denied,
            71,
        ),
        (
            session
                .send_udp([192, 0, 2, 1], 9000, b"after revoke")
                .map(|_| ())
                == denied,
            72,
        ),
        (session.receive_udp(&mut out).map(|_| ()) == denied, 73),
    ] {
        check(ok, code);
        bump(432);
    }
    session.close(connection).unwrap_or_else(|_| fail(74));
    support::put(544, 2);
    session.retire().unwrap_or_else(|_| fail(75));
    check(session.is_closed(), 76);
    support::put(528, 1);
    native::exit(0)
}
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    run()
}
#[inline(never)]
fn run() -> ! {
    native::check_abi().unwrap_or_else(|_| fail(1));
    if support::word(32) == u64::from_le_bytes(*b"NETPOL01") {
        policy()
    }
    check(support::word(32) == u64::from_le_bytes(*b"NETAPP01"), 5);
    let epoch = support::word(40);
    let send = SendEndpoint::from_handle(h(48));
    let receive = ReceiveEndpoint::from_handle(h(56));
    let peer = support::word(64);
    let slot = usize::try_from(support::word(72)).unwrap_or_else(|_| fail(6));
    let own = support::word(80);
    let first = u16::try_from(support::word(88)).unwrap_or_else(|_| fail(7));
    let udp = u16::try_from(support::word(96)).unwrap_or_else(|_| fail(8));
    check(
        epoch != 0 && peer != 0 && own != 0 && peer != own && slot < 2 && first != 0 && udp != 0,
        9,
    );
    let mut raw = [0; 1514];
    check(
        native::net_receive(NetworkSource::from_handle(send.handle()), &mut raw).is_err(),
        17,
    );
    bump(480);
    malformed(send, receive, peer, epoch);
    let mut session = Session::new(NativeTransport, epoch, peer, send, receive, 100_000)
        .unwrap_or_else(|_| fail(18));
    support::put(400, 1);
    check(
        session.open([192, 0, 2, 1], first + 5, 300) == Err(Error::Service(Status::Access)),
        19,
    );
    bump(424);
    if support::word(104) == 2 {
        check(slot == 0, 60);
        revoked(&mut session)
    }
    if support::word(104) == 1 {
        check(slot == 0, 48);
        let connection = session
            .open([192, 0, 2, 1], first + 4, 700)
            .unwrap_or_else(|_| fail(49));
        let start = tick();
        while tick() - start < 3 {
            yield_now();
        }
        let packet = Packet::request(
            HeaderFields {
                operation: Operation::State,
                request: 3,
                epoch,
                connection: connection.id(),
                argument: 0,
                address: [0; 4],
                port: 0,
            },
            &[],
        )
        .unwrap();
        let message = Outgoing::new(wire::TAG, packet.bytes()).unwrap();
        loop {
            check(tick() - start < 100, 50);
            match native::send(send, &message) {
                Ok(()) => break,
                Err(e) if e.is_again() => yield_now(),
                Err(_) => fail(51),
            }
        }
        // Enqueued request, then a real kernel close of the reply receiver:
        // the service's pending reply must get Closed, never a fake ACK.
        native::close(receive.handle()).unwrap_or_else(|_| fail(52));
        // A reply might have reached the queue before close. Publish one more
        // correlated request to force a send against the now-closed receiver;
        // if the service already retired this ingress, Closed is expected.
        let mut confirm = *packet.bytes();
        confirm[8..16].copy_from_slice(&4u64.to_le_bytes());
        let confirm = Outgoing::new(wire::TAG, &confirm).unwrap();
        loop {
            check(tick() - start < 100, 54);
            match native::send(send, &confirm) {
                Ok(()) => break,
                Err(nieuw_os_sdk_runtime::Error::Kernel(-4 | -9 | -10)) => break,
                Err(e) if e.is_again() => yield_now(),
                Err(_) => fail(55),
            }
        }
        native::close(send.handle()).unwrap_or_else(|_| fail(53));
        support::put(536, 1);
        let closed = tick();
        while tick() - closed < 25 {
            yield_now();
        }
        support::put(528, 1);
        native::exit(0)
    }
    let mut previous = None;
    for (offset, case) in [
        Case::Http,
        Case::Refused,
        Case::Reset,
        Case::Retransmit,
        Case::Timeout,
    ]
    .into_iter()
    .enumerate()
    {
        previous = Some(exercise(
            &mut session,
            slot,
            first + offset as u16,
            case,
            previous,
        ));
    }
    let payload = if slot == 0 {
        b"Nieuw OS UDP app A"
    } else {
        b"Nieuw OS UDP app B"
    };
    let start = tick();
    loop {
        check(tick() - start < 500, 40);
        match session.send_udp([192, 0, 2, 1], udp, payload) {
            Ok(n) => {
                check(n == payload.len(), 41);
                break;
            }
            Err(e) if again(e) => yield_now(),
            Err(_) => fail(42),
        }
    }
    // The peer's bounded four-frame burst exercises malformed/checksum/scope
    // refusal and full-slot preservation while this app does not consume UDP.
    let queued = tick();
    while tick() - queued < 25 {
        yield_now();
    }
    let mut out = [0; DATA];
    loop {
        check(tick() - start < 500, 43);
        match session.receive_udp(&mut out) {
            Ok(n) => {
                check(n == payload.len() && out[..n] == *payload, 44);
                bump(416);
                break;
            }
            Err(e) if again(e) => yield_now(),
            Err(_) => fail(45),
        }
    }
    session.retire().unwrap_or_else(|_| fail(46));
    check(session.is_closed(), 47);
    support::put(528, 1);
    native::exit(0)
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99)
}
