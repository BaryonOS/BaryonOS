#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Actual independent ring-3 ARP/UDP/DNS client. The external host process is
//! solely a synthetic packet peer, never a substitute for these OS operations.

#[path = "../../system/src/support.rs"]
mod support;
use core::arch::asm;
use nieuw_os_net_protocol::{
    arp::{self, Interface, NeighborSlot},
    build_udp_ipv4,
    dns::{self, Address, Endpoint, Name, PendingQuery, QueryType, Question},
    parse, UdpIpv4Tx,
};
use nieuw_os_sdk_runtime::{decode_return, native, Error, Handle, NetworkSource};

const LOCAL: Interface = Interface {
    mac: [2, 0, 0, 0, 0, 7],
    ipv4: [192, 0, 2, 2],
};
const PEER_MAC: [u8; 6] = [2, 0, 0, 0, 0, 1];
const PEER_IP: [u8; 4] = [192, 0, 2, 1];
const DNS_IP: [u8; 4] = [192, 0, 2, 53];
const POLLS: usize = 50_000;
const ARP_REPLIES: u64 = 400;
const UDP_ECHOES: u64 = 408;
const DNS_ANSWERS: u64 = 416;
const DENIED: u64 = 424;
const TX_COMPLETIONS: u64 = 432;
const RX_FRAMES: u64 = 440;
const ARP_REJECTS: u64 = 448;
const DNS_REJECTS: u64 = 456;
const RX_TIMEOUTS: u64 = 464;
const CACHE_INTACT: u64 = 472;
const STALE_CAP: u64 = 480;

fn fail(step: u64) -> ! {
    support::put(240, step);
    support::put(40, 0x100 + step);
    native::exit(1)
}
fn check(ok: bool, step: u64) {
    if !ok {
        fail(step);
    }
}
fn bump(offset: u64) {
    support::put(offset, support::word(offset) + 1);
}
fn tick() {
    support::put(8, support::word(8).wrapping_add(1));
}

/// Deliberately raw ABI probes pass invalid integers, never invalid Rust slices.
/// The kernel must reject every malformed request before accessing user memory.
fn raw(number: u64, args: [u64; 6]) -> Result<u64, Error> {
    let mut primary = number;
    let mut secondary = args[2];
    // SAFETY: Nieuw OS int80 ABI preserves GPRs except RAX/RDX. Numeric invalid
    // pointers are not dereferenced here; kernel tests their complete ranges.
    // No nomem flag because a valid syscall may copy bytes through its arguments.
    unsafe {
        asm!("int 0x80", inout("rax") primary, in("rdi") args[0],
             in("rsi") args[1], inout("rdx") secondary, in("r10") args[3],
             in("r8") args[4], in("r9") args[5], options(nostack));
    }
    decode_return(primary as i64, secondary)
}

fn denied(result: Result<u64, Error>, code: i64, step: u64) {
    check(result == Err(Error::Kernel(code)), step);
    bump(DENIED);
}

fn send(source: NetworkSource, frame: &[u8], step: u64) {
    support::put(240, step);
    let mut ticket = None;
    for _ in 0..POLLS {
        tick();
        match native::net_send(source, frame, 100) {
            Ok(value) => {
                ticket = Some(value);
                break;
            }
            Err(error) if error.is_again() => {
                let _ = native::yield_now();
            }
            Err(_) => fail(step),
        }
    }
    let ticket = ticket.unwrap_or_else(|| fail(step));
    check(ticket != 0, step);
    for _ in 0..POLLS {
        tick();
        match native::net_poll_tx(source, ticket) {
            Ok(length) => {
                check(length == frame.len().max(60), step);
                bump(TX_COMPLETIONS);
                return;
            }
            Err(error) if error.is_again() => {
                let _ = native::yield_now();
            }
            Err(_) => fail(step),
        }
    }
    // Never repeat an already-published frame after an uncertain completion.
    fail(step)
}

fn receive(source: NetworkSource, output: &mut [u8; 1514], clock: &mut u64) -> Option<usize> {
    for _ in 0..POLLS {
        *clock += 1;
        tick();
        match native::net_receive(source, output) {
            Ok(length) => {
                check((14..=1514).contains(&length), 20);
                bump(RX_FRAMES);
                return Some(length);
            }
            Err(error) if error.is_again() => {
                let _ = native::yield_now();
            }
            Err(_) => fail(21),
        }
    }
    None
}

fn udp(payload: &[u8], ip: [u8; 4], port: u16, source_port: u16, output: &mut [u8; 1514]) -> usize {
    build_udp_ipv4(
        UdpIpv4Tx {
            destination_mac: PEER_MAC,
            source_mac: LOCAL.mac,
            source_ipv4: LOCAL.ipv4,
            destination_ipv4: ip,
            source_port,
            destination_port: port,
            identification: 0x1632,
            ttl: 64,
            mtu: 1500,
        },
        payload,
        output,
    )
    .unwrap_or_else(|_| fail(22))
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    main()
}

#[inline(never)]
fn main() -> ! {
    check(native::abi_info() == Ok((1u64 << 32) | 5), 1);
    for offset in (400..=480).step_by(8) {
        support::put(offset, 0);
    }
    let handle = Handle::from_raw(support::word(48)).unwrap_or_else(|_| fail(2));
    let source = NetworkSource::from_handle(handle);
    let wrong_type = support::word(56);
    let mut tx = [0; 1514];
    let mut rx = [0; 1514];
    denied(
        raw(0x70, [wrong_type, tx.as_ptr() as u64, 60, 100, 0, 0]),
        -6,
        3,
    );
    denied(raw(0x70, [handle.raw(), 0, 60, 100, 0, 0]), -3, 4);
    denied(
        raw(0x70, [handle.raw(), tx.as_ptr() as u64, 1515, 100, 0, 0]),
        -11,
        5,
    );
    denied(raw(0x72, [handle.raw(), 0, 1514, 0, 0, 0]), -3, 6);
    denied(
        raw(0x72, [handle.raw(), rx.as_mut_ptr() as u64, 1515, 0, 0, 0]),
        -11,
        7,
    );
    denied(raw(0x71, [handle.raw(), 0, 0, 0, 0, 0]), -1, 8);

    let mut clock = 0u64; // Monotonic receive-poll steps, not wall-clock seconds.
    let mut neighbor = NeighborSlot::new(LOCAL).unwrap_or_else(|_| fail(9));
    check(neighbor.begin(PEER_IP, clock, 1_000_000).is_ok(), 10);
    for attempt in 0..3 {
        let length = arp::build_request(LOCAL, PEER_IP, &mut tx).unwrap_or_else(|_| fail(11));
        send(source, &tx[..length], 12);
        let length = receive(source, &mut rx, &mut clock).unwrap_or_else(|| fail(13));
        let reply = neighbor.accept_reply(&rx[..length], clock, 1_000_000);
        if attempt < 2 {
            check(
                reply
                    == Err(if attempt == 0 {
                        arp::Error::SenderMismatch
                    } else {
                        arp::Error::TargetMismatch
                    }),
                14,
            );
            check(neighbor.lookup(PEER_IP, clock).is_none(), 15);
            bump(ARP_REJECTS);
        } else {
            check(reply == Ok(PEER_MAC), 16);
            bump(ARP_REPLIES);
        }
    }
    check(neighbor.lookup(PEER_IP, clock) == Some(PEER_MAC), 17);

    let mut payload = [0; 40];
    let prefix = b"nieuw-os-native/1:";
    payload[..prefix.len()].copy_from_slice(prefix);
    payload[prefix.len()..prefix.len() + 8].copy_from_slice(&support::word(176).to_le_bytes());
    let payload = &payload[..prefix.len() + 8];
    let length = udp(payload, PEER_IP, 9000, 49153, &mut tx);
    send(source, &tx[..length], 23);
    let length = receive(source, &mut rx, &mut clock).unwrap_or_else(|| fail(24));
    let echo = parse(&rx[..length]).unwrap_or_else(|_| fail(25));
    check(
        echo.source_mac() == PEER_MAC
            && echo.destination_mac() == LOCAL.mac
            && echo.source_address() == PEER_IP
            && echo.destination_address() == LOCAL.ipv4
            && echo.source_port() == 9000
            && echo.destination_port() == 49153
            && echo.payload() == payload,
        26,
    );
    bump(UDP_ECHOES);

    let question = Question {
        id: 0x1234,
        name: Name::from_ascii("native.nieuw.test").unwrap_or_else(|_| fail(27)),
        kind: QueryType::A,
    };
    let local = Endpoint {
        address: LOCAL.ipv4,
        port: 49152,
    };
    let server = Endpoint {
        address: DNS_IP,
        port: 53,
    };
    let mut pending =
        PendingQuery::new(question, local, server, clock, 1_000_000).unwrap_or_else(|_| fail(28));
    let mut query = [0; 256];
    let query_len = dns::build_query(question, &mut query).unwrap_or_else(|_| fail(29));
    for attempt in 0..3 {
        let length = udp(&query[..query_len], DNS_IP, 53, 49152, &mut tx);
        send(source, &tx[..length], 30);
        let length = receive(source, &mut rx, &mut clock).unwrap_or_else(|| fail(31));
        let packet = parse(&rx[..length]).unwrap_or_else(|_| fail(32));
        match pending.accept(&packet, clock) {
            Err(error) if attempt < 2 => {
                check(
                    error
                        == if attempt == 0 {
                            dns::Error::WrongId
                        } else {
                            dns::Error::CompressionLoop
                        },
                    33,
                );
                bump(DNS_REJECTS);
            }
            Ok(response) if attempt == 2 => {
                let result = response.resolve().unwrap_or_else(|_| fail(34));
                let mut addresses = result.addresses();
                check(
                    result.canonical_name == question.name
                        && result.cache_ttl == 60
                        && addresses.next() == Some(&Address::Ipv4([198, 51, 100, 42]))
                        && addresses.next().is_none(),
                    35,
                );
                bump(DNS_ANSWERS);
            }
            _ => fail(36),
        }
    }
    let timeout_question = Question {
        id: 0x5678,
        name: Name::from_ascii("timeout.nieuw.test").unwrap_or_else(|_| fail(37)),
        kind: QueryType::A,
    };
    let mut pending = PendingQuery::new(timeout_question, local, server, clock, POLLS as u64)
        .unwrap_or_else(|_| fail(38));
    let query_len = dns::build_query(timeout_question, &mut query).unwrap_or_else(|_| fail(39));
    let length = udp(&query[..query_len], DNS_IP, 53, 49152, &mut tx);
    send(source, &tx[..length], 40);
    check(receive(source, &mut rx, &mut clock).is_none(), 41);
    pending.cancel();
    bump(RX_TIMEOUTS);
    check(neighbor.lookup(PEER_IP, clock) == Some(PEER_MAC), 42);
    bump(CACHE_INTACT);

    check(native::close(handle).is_ok(), 43);
    denied(
        raw(0x70, [handle.raw(), tx.as_ptr() as u64, 60, 100, 0, 0]),
        -4,
        44,
    );
    bump(STALE_CAP);
    check(
        support::word(DENIED) == 7
            && support::word(TX_COMPLETIONS) == 8
            && support::word(RX_FRAMES) == 7,
        45,
    );
    support::put(240, 0);
    support::put(40, 1);
    native::exit(0)
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99)
}
