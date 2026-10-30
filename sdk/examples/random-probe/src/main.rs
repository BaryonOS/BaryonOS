#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Actual ring-3 RND1 caller. No generator/entropy source or host fallback.
#[path = "../../system/src/support.rs"]
mod support;
use core::cell::Cell;
use nieuw_os_keyring_random_client::{Delivery, Failure, Outcome, Session, Transport};
use nieuw_os_native_random_probe::{contract::*, wire};
use nieuw_os_sdk_runtime::{
    native, EntropySource, Error, Handle, Outgoing, ReceiveEndpoint, Received, Rights, SendEndpoint,
};
use zeroize::Zeroizing;

const POLLS: usize = 50_000;
fn fail(step: u64) -> ! {
    support::put(STEP, step);
    support::put(RESULT, 0x100 + step);
    native::exit(1)
}
fn check(ok: bool, step: u64) {
    if !ok {
        fail(step)
    }
}
fn bump(at: u64) {
    support::put(
        at,
        support::word(at).checked_add(1).unwrap_or_else(|| fail(1)),
    );
}
fn handle(raw: u64) -> Handle {
    Handle::from_raw(raw).unwrap_or_else(|_| fail(2))
}
fn clock() -> u64 {
    bump(CLOCK_READS);
    let now = native::monotonic_ticks().unwrap_or_else(|_| fail(3));
    // Private scratch only; observing a backwards trusted clock is terminal.
    check(now >= support::word(600), 3);
    support::put(600, now);
    now
}
fn yield_now() {
    support::put(8, support::word(8).wrapping_add(1));
    native::yield_now().unwrap_or_else(|_| fail(4));
}
fn deadline() -> u64 {
    clock()
        .checked_add(REQUEST_TICKS)
        .unwrap_or_else(|| fail(5))
}
fn wait_ticks(start: u64, ticks: u64) {
    let end = start.checked_add(ticks).unwrap_or_else(|| fail(6));
    while clock() < end {
        yield_now();
    }
}
fn phase(value: u64) {
    check(support::word(ACK_PHASE) < value || value == SETTLE, 7);
    support::put(REQUEST_PHASE, value);
    let end = deadline();
    while support::word(ACK_PHASE) != value {
        check(clock() < end, 8);
        yield_now();
    }
}
fn send(endpoint: SendEndpoint, frame: &[u8; wire::SIZE]) {
    let outgoing = Outgoing::new(wire::TAG, frame).unwrap();
    let end = deadline();
    for _ in 0..POLLS {
        check(clock() < end, 10);
        match native::send(endpoint, &outgoing) {
            Ok(()) => {
                bump(ENQUEUED);
                return;
            }
            Err(e) if e.is_again() => yield_now(),
            Err(_) => fail(11),
        }
    }
    fail(12)
}
fn exchange(
    endpoint: SendEndpoint,
    receive: ReceiveEndpoint,
    frame: &[u8; wire::SIZE],
    count: usize,
) -> wire::Reply {
    send(endpoint, frame);
    let end = deadline();
    for _ in 0..POLLS {
        check(clock() < end, 13);
        match native::receive(receive) {
            Ok(message) => {
                let parsed = wire::reply(&message, support::word(KEY_PID), frame, count);
                let caps = native::close_received_caps(&message);
                drop(message);
                check(caps.is_ok(), 14);
                bump(REPLIES);
                return parsed.unwrap_or_else(|| fail(15));
            }
            Err(e) if e.is_again() => yield_now(),
            Err(_) => fail(16),
        }
    }
    fail(17)
}
fn positive(
    send: SendEndpoint,
    recv: ReceiveEndpoint,
    id: u64,
    count: usize,
) -> Zeroizing<[u8; wire::MAX]> {
    let frame = wire::request(wire::RANDOM, id, count);
    let reply = exchange(send, recv, &frame, count);
    check(reply.status == 0 && reply.length == count, 18);
    // Zero and repeated single bytes are valid random output. Only a full
    // 32+-byte sample is used for this supplementary all-zero sanity check.
    if count >= 32 {
        check(reply.bytes().iter().any(|b| *b != 0), 19);
    }
    let mut output = Zeroizing::new([0; wire::MAX]);
    output[..count].copy_from_slice(reply.bytes());
    bump(SUCCESSES);
    support::put(BYTES, support::word(BYTES) + count as u64);
    output
}
fn denial(send: SendEndpoint, recv: ReceiveEndpoint, frame: &[u8; wire::SIZE], status: i32) {
    let reply = exchange(send, recv, frame, 0);
    check(reply.status == status && reply.length == 0, 20);
}
fn capability_denials(send: SendEndpoint, recv: ReceiveEndpoint) {
    let empty = Outgoing::new(0, &[]).unwrap();
    for (result, code) in [
        (
            native::send(SendEndpoint::from_handle(recv.handle()), &empty),
            -5,
        ),
        (
            native::receive(ReceiveEndpoint::from_handle(send.handle())).map(|_| ()),
            -5,
        ),
        (
            native::derive(send.handle(), Rights::SEND.union(Rights::DERIVE)).map(|_| ()),
            -5,
        ),
        (native::revoke(send.handle()), -5),
        (
            native::entropy_request(EntropySource::from_handle(send.handle()), 1).map(|_| ()),
            -6,
        ),
        (
            native::entropy_request(EntropySource::from_handle(recv.handle()), 1).map(|_| ()),
            -6,
        ),
    ] {
        check(result == Err(Error::Kernel(code)), 21);
        bump(CAP_DENIED);
    }
}
fn raw_case(mut send_cap: SendEndpoint, recv: ReceiveEndpoint) {
    capability_denials(send_cap, recv);
    let mut previous = Zeroizing::new([0u8; wire::MAX]);
    for (id, count) in [1, 32, 64, 128, 224].into_iter().enumerate() {
        let output = positive(send_cap, recv, id as u64 + 1, count);
        if count >= 32 {
            check(output[..count] != previous[..count], 22);
        }
        previous.copy_from_slice(&*output);
        bump(VARIABLES);
    }
    for op in 1..=9 {
        denial(
            send_cap,
            recv,
            &wire::request(op, 500 + u64::from(op), 0),
            -6,
        );
        bump(LEGACY_DENIED);
    }
    for variant in 0..9 {
        let mut frame = wire::request(wire::RANDOM, 600 + variant, 1);
        match variant {
            0 => frame[22..24].copy_from_slice(&0u16.to_le_bytes()),
            1 => frame[22..24].copy_from_slice(&225u16.to_le_bytes()),
            2 => frame[1] = 1,
            3 => frame[12] = 1,
            4 => frame[20] = 2,
            5 => frame[24] = 1,
            6 => frame[4..12].fill(0),
            7 => frame[2..4].copy_from_slice(&15u16.to_le_bytes()),
            8 => frame[16] = b'X',
            _ => unreachable!(),
        }
        denial(send_cap, recv, &frame, -6);
        bump(INVALID_DENIED);
    }
    wait_ticks(clock(), WINDOW_TICKS);
    for id in 1000..1064 {
        drop(positive(send_cap, recv, id, 1));
    }
    denial(send_cap, recv, &wire::request(wire::RANDOM, 1064, 1), -2);
    bump(QUOTA_DENIED);
    wait_ticks(clock(), WINDOW_TICKS);
    drop(positive(send_cap, recv, 1065, 128));
    wait_ticks(clock(), WINDOW_TICKS);
    for id in 2000..2018 {
        drop(positive(send_cap, recv, id, 224));
    }
    let stale = send_cap;
    native::close(stale.handle()).unwrap_or_else(|_| fail(23));
    phase(REBIND);
    send_cap = SendEndpoint::from_handle(handle(support::word(NEW_SEND)));
    check(send_cap.handle() != stale.handle(), 24);
    bump(REBINDS);
    let message = Outgoing::new(wire::TAG, &wire::request(wire::RANDOM, 9999, 1)).unwrap();
    check(native::send(stale, &message) == Err(Error::Kernel(-4)), 25);
    bump(STALE_DENIED);
    denial(send_cap, recv, &wire::request(wire::RANDOM, 1, 64), -6);
    bump(REPLAY_DENIED);
    let fresh = positive(send_cap, recv, 10_000, 64);
    check(fresh[..64] != previous[..64], 26);
    native::close(recv.handle()).unwrap_or_else(|_| fail(27));
    send(send_cap, &wire::request(wire::RANDOM, 10_001, 32));
    phase(CLOSED_REPLY);
    phase(REVOKE);
    let revoked = Outgoing::new(wire::TAG, &wire::request(wire::RANDOM, 10_002, 32)).unwrap();
    check(
        native::send(send_cap, &revoked) == Err(Error::Kernel(-10)),
        28,
    );
    bump(REVOKED_DENIED);
    native::close(send_cap.handle()).unwrap_or_else(|_| fail(29));
}

struct ProbeTransport {
    unknown: bool,
    sends: Cell<u64>,
    published: Cell<u64>,
    closes: Cell<u64>,
}
impl ProbeTransport {
    fn new(unknown: bool) -> Self {
        Self {
            unknown,
            sends: Cell::new(0),
            published: Cell::new(0),
            closes: Cell::new(0),
        }
    }
}
impl Transport for &ProbeTransport {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), Error> {
        self.sends.set(self.sends.get() + 1);
        let result = native::send(endpoint, message);
        if result.is_ok() {
            self.published.set(self.published.get() + 1);
            if self.unknown {
                return Err(Error::InvalidReturn);
            }
        }
        result
    }
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, Error> {
        native::receive(endpoint)
    }
    fn yield_now(&mut self) -> Result<(), Error> {
        native::yield_now()
    }
    fn close(&mut self, handle: Handle) -> Result<(), Error> {
        let result = native::close(handle);
        if result.is_ok() {
            self.closes.set(self.closes.get() + 1);
        }
        result
    }
}
fn sdk_case(send: SendEndpoint, recv: ReceiveEndpoint, unknown: bool) {
    let transport = ProbeTransport::new(unknown);
    let mut client = Session::new(&transport, send, recv, support::word(KEY_PID), POLLS)
        .unwrap_or_else(|_| fail(30));
    if !unknown {
        let mut previous = Zeroizing::new([0; 128]);
        for count in [1, 32, 64, 128] {
            client.begin(count).unwrap_or_else(|_| fail(31));
            let end = deadline();
            loop {
                check(clock() < end, 32);
                match client.poll() {
                    Ok(Some(Outcome::Bytes(bytes))) => {
                        check(bytes.as_slice().len() == count, 33);
                        if count >= 32 {
                            check(bytes.as_slice().iter().any(|b| *b != 0), 33);
                            check(bytes.as_slice() != &previous[..count], 34);
                        }
                        previous[..count].copy_from_slice(bytes.as_slice());
                        bump(SDK_POSITIVES);
                        support::put(
                            SDK_OUTPUT_BYTES,
                            support::word(SDK_OUTPUT_BYTES) + count as u64,
                        );
                        break;
                    }
                    Ok(None) => {}
                    _ => fail(35),
                }
            }
        }
    }
    client
        .begin(if unknown { 64 } else { 32 })
        .unwrap_or_else(|_| fail(36));
    if unknown {
        let error = client.poll().unwrap_err();
        check(
            error.delivery == Delivery::Unknown
                && error.failure == Failure::Transport(Error::InvalidReturn)
                && error.cleanup.is_none(),
            37,
        );
        bump(SDK_UNKNOWN);
    } else {
        check(client.poll().unwrap_or_else(|_| fail(38)).is_none(), 39);
        client.cancel().unwrap_or_else(|_| fail(40));
        bump(SDK_CANCELLED);
    }
    let before = transport.sends.get();
    check(client.is_closed(), 41);
    check(client.poll().unwrap_err().failure == Failure::Closed, 42);
    check(client.begin(32).unwrap_err().failure == Failure::Closed, 43);
    check(transport.sends.get() == before, 44);
    bump(NO_RETRY);
    support::put(SDK_SEND_CALLS, transport.sends.get());
    support::put(ENQUEUED, transport.published.get());
    support::put(SDK_CLOSES, transport.closes.get());
    check(transport.closes.get() == 2, 45);
    phase(SETTLE);
}
fn wrong_pid(send_cap: SendEndpoint, recv: ReceiveEndpoint) {
    send(send_cap, &wire::request(wire::RANDOM, 1, 32));
    phase(WRONG_PID_CONSUMED);
    let start = clock();
    loop {
        match native::receive(recv) {
            Err(e) if e.is_again() => {}
            _ => fail(46),
        }
        if clock().saturating_sub(start) >= NO_REPLY_TICKS {
            break;
        }
        yield_now();
    }
    bump(PID_DENIED);
    native::close(send_cap.handle()).unwrap_or_else(|_| fail(47));
    native::close(recv.handle()).unwrap_or_else(|_| fail(48));
}

core::arch::global_asm!(
    r#"
.section .text.entry,"ax",@progbits
.global _start
_start:
    cld
    sub rsp, 8
    call random_main
    ud2
"#
);
#[no_mangle]
extern "C" fn random_main() -> ! {
    native::check_abi().unwrap_or_else(|_| fail(49));
    check(
        support::word(32) == 1 && support::word(OWN_PID) != 0 && support::word(KEY_PID) != 0,
        50,
    );
    let send = SendEndpoint::from_handle(handle(support::word(SEND)));
    let recv = ReceiveEndpoint::from_handle(handle(support::word(RECEIVE)));
    check(send.handle() != recv.handle(), 51);
    support::put(START_TICKS, clock());
    match support::word(CASE) {
        RAW => raw_case(send, recv),
        SDK_CANCEL => sdk_case(send, recv, false),
        SDK_UNCERTAIN => sdk_case(send, recv, true),
        WRONG_PID => wrong_pid(send, recv),
        _ => fail(52),
    }
    support::put(END_TICKS, clock());
    support::put(RESULT, 1);
    native::exit(0)
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(255)
}
