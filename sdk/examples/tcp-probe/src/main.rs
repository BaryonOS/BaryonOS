#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Real ring-3 smoltcp client. The external process is only a packet server.
mod contract;
#[path = "../../system/src/support.rs"]
mod support;
use nieuw_os_net_stack::{Client, Config, Error as StackError, SocketStorage, Stack, State};
use nieuw_os_sdk_runtime::{native, Error, Handle, NetworkSource};

const REQUEST: &[u8] =
    b"GET /nieuw-os-tcp HTTP/1.1\r\nHost: fixture.invalid\r\nConnection: close\r\n\r\n";
const RESPONSE: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Length: 20\r\nConnection: close\r\n\r\nNieuw OS native TCP\n";
const HTTP: u64 = 400;
const REFUSED: u64 = 408;
const RESET: u64 = 416;
const RETRANSMIT: u64 = 424;
const TX: u64 = 432;
const RX: u64 = 440;
const TIMEOUT: u64 = 448;
const BODY_BYTES: u64 = 456;
const START: u64 = 464;
const END: u64 = 472;
const CLOSED_CAP: u64 = 480;
const CLOCK_READS: u64 = 488;
const TIMEOUT_ELAPSED: u64 = 496;

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
    support::put(
        offset,
        support::word(offset)
            .checked_add(1)
            .unwrap_or_else(|| fail(90)),
    );
}
fn clock() -> u64 {
    bump(CLOCK_READS);
    native::monotonic_ticks().unwrap_or_else(|_| fail(2))
}

struct Pending {
    id: u64,
    ticket: u64,
    bytes: usize,
}
struct Bridge {
    source: NetworkSource,
    pending: Option<Pending>,
    rx: [u8; 1514],
}

impl Bridge {
    fn step(&mut self, stack: &mut Stack<'_>, now: u64) {
        if let Some(pending) = &self.pending {
            match native::net_poll_tx(self.source, pending.ticket) {
                Ok(length) => {
                    check(length == pending.bytes.max(60), 10);
                    stack
                        .complete_transmission(pending.id)
                        .unwrap_or_else(|_| fail(11));
                    self.pending = None;
                    bump(TX);
                }
                Err(error) if error.is_again() => {}
                // Never republish a possibly DMA-live frame after an uncertain
                // completion. Native driver reset/quarantine belongs to kernel.
                Err(_) => fail(12),
            }
        }
        if self.pending.is_none() {
            if let Some(frame) = stack.transmission() {
                match native::net_send(self.source, frame.bytes, 100) {
                    Ok(ticket) => {
                        check(ticket != 0, 13);
                        self.pending = Some(Pending {
                            id: frame.id,
                            ticket,
                            bytes: frame.bytes.len(),
                        });
                    }
                    Err(error) if error.is_again() => {}
                    Err(_) => fail(14),
                }
            }
        }
        if stack.can_receive_frame() {
            match native::net_receive(self.source, &mut self.rx) {
                Ok(length) => {
                    check((14..=1514).contains(&length), 15);
                    stack
                        .push_frame(&self.rx[..length])
                        .unwrap_or_else(|_| fail(16));
                    bump(RX);
                }
                Err(error) if error.is_again() => {}
                Err(_) => fail(17),
            }
        }
        stack.poll(now).unwrap_or_else(|_| fail(18));
        bump(8); // Liveness only; never used as network time.
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

fn exercise(stack: &mut Stack<'_>, client: Client, bridge: &mut Bridge, case: Case, index: u16) {
    support::put(240, 30 + u64::from(index));
    let start = clock();
    stack.poll(start).unwrap_or_else(|_| fail(20));
    let timeout = if case == Case::Timeout { 300 } else { 700 };
    stack
        .connect(client, [192, 0, 2, 1], 8080 + index, 40000 + index, timeout)
        .unwrap_or_else(|_| fail(21));
    let deadline = start.checked_add(timeout + 200).unwrap_or_else(|| fail(22));
    let mut established = false;
    let mut eof = false;
    let mut sent = 0usize;
    let mut received = 0usize;
    let mut output = [0; 256];
    loop {
        let now = clock();
        check(now >= start && now <= deadline, 23);
        bridge.step(stack, now);
        let state = stack.state(client).unwrap_or_else(|_| fail(24));
        if state == State::Established {
            established = true;
        }
        if established && state != State::Closed {
            if sent < REQUEST.len() && state == State::Established {
                sent += stack
                    .send(client, &REQUEST[sent..])
                    .unwrap_or_else(|_| fail(25));
            }
            match stack.receive(client, &mut output) {
                Ok(length) if length != 0 => {
                    check(case == Case::Http || case == Case::Retransmit, 26);
                    let end = received.checked_add(length).unwrap_or_else(|| fail(27));
                    check(
                        end <= RESPONSE.len() && output[..length] == RESPONSE[received..end],
                        28,
                    );
                    received = end;
                }
                Ok(_) | Err(StackError::Finished) => {}
                Err(_) => fail(29),
            }
            if state == State::CloseWait {
                check(received == RESPONSE.len() && sent == REQUEST.len(), 30);
                eof = true;
                stack.close(client).unwrap_or_else(|_| fail(31));
            }
        }
        if state == State::Closed && bridge.pending.is_none() && stack.transmission().is_none() {
            match case {
                Case::Http | Case::Retransmit => {
                    check(
                        contract::http_complete(
                            contract::HttpEvidence {
                                established,
                                eof,
                                sent,
                                received,
                                elapsed_ticks: now - start,
                            },
                            REQUEST.len(),
                            RESPONSE.len(),
                            timeout,
                        ),
                        32,
                    );
                    bump(HTTP);
                    support::put(BODY_BYTES, support::word(BODY_BYTES) + 20);
                    if case == Case::Retransmit {
                        // The peer additionally proves an actual repeated SYN.
                        check(now - start >= 100, 37);
                        bump(RETRANSMIT);
                    }
                }
                Case::Refused => {
                    check(
                        !established && sent == 0 && received == 0 && now - start < timeout,
                        33,
                    );
                    bump(REFUSED);
                }
                Case::Reset => {
                    check(
                        established
                            && sent == REQUEST.len()
                            && received == 0
                            && now - start < timeout,
                        34,
                    );
                    bump(RESET);
                }
                Case::Timeout => {
                    check(
                        !established && sent == 0 && received == 0 && now - start >= 300,
                        35,
                    );
                    support::put(TIMEOUT_ELAPSED, now - start);
                    bump(TIMEOUT);
                }
            }
            break;
        }
        check(native::yield_now().is_ok(), 36);
    }
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    main()
}

#[inline(never)]
fn main() -> ! {
    check(native::abi_info() == Ok((1u64 << 32) | 6), 1);
    check(
        nieuw_os_sdk_runtime::clock::MONOTONIC_HZ == nieuw_os_net_stack::MONOTONIC_HZ,
        3,
    );
    for offset in (400..=496).step_by(8) {
        support::put(offset, 0);
    }
    let source =
        NetworkSource::from_handle(Handle::from_raw(support::word(48)).unwrap_or_else(|_| fail(4)));
    let start = clock();
    support::put(START, start);
    let (mut rx_frame, mut tx_frame) = ([0; 1514], [0; 1514]);
    let (mut rx_socket, mut tx_socket) = ([0; 4096], [0; 4096]);
    let mut storage = [SocketStorage::EMPTY];
    let mut stack = Stack::new(
        Config {
            mac: [2, 0, 0, 0, 0, 7],
            ipv4: [192, 0, 2, 2],
            prefix_len: 24,
            random_seed: 0x4e4f_532d_5443_5021, // Clearly scoped synthetic test seed.
        },
        &mut storage,
        &mut rx_frame,
        &mut tx_frame,
        start,
    )
    .unwrap_or_else(|_| fail(5));
    let client = stack
        .add_client(&mut rx_socket, &mut tx_socket)
        .unwrap_or_else(|_| fail(6));
    let mut bridge = Bridge {
        source,
        pending: None,
        rx: [0; 1514],
    };
    for (index, case) in [
        Case::Http,
        Case::Refused,
        Case::Reset,
        Case::Retransmit,
        Case::Timeout,
    ]
    .into_iter()
    .enumerate()
    {
        exercise(&mut stack, client, &mut bridge, case, index as u16);
    }
    check(
        support::word(HTTP) == 2
            && support::word(REFUSED) == 1
            && support::word(RESET) == 1
            && support::word(RETRANSMIT) == 1
            && support::word(TIMEOUT) == 1
            && support::word(BODY_BYTES) == 40,
        40,
    );
    let end = clock();
    check(end > start, 41);
    support::put(END, end);
    check(native::close(source.handle()).is_ok(), 42);
    check(
        native::net_receive(source, &mut bridge.rx) == Err(Error::Kernel(-4)),
        43,
    );
    bump(CLOSED_CAP);
    support::put(240, 0);
    support::put(40, 1);
    native::exit(0)
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99)
}
