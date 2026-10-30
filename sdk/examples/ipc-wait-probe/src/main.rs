#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

mod support;

use core::arch::{asm, global_asm};
use nieuw_os_ipc_wait_probe::*;
use nieuw_os_sdk_runtime::{
    decode_return, native, Error, Handle, ReceiveEndpoint, Rights, SendEndpoint, WaitCondition,
};

global_asm!(
    r#"
.section .text.entry,"ax",@progbits
.global _start
_start:
    cld
    mov rax, rsp
    and eax, 15
    cmp eax, 8
    jne .Lwait_bad_alignment
    jmp ipc_wait_main
.Lwait_bad_alignment:
    ud2
"#
);

struct Endpoints {
    root: Handle,
    receive: ReceiveEndpoint,
    peer: SendEndpoint,
    own_send: SendEndpoint,
    peer_pid: u64,
}

fn publish(code: u64) {
    support::put(416, code);
    let elapsed = native::monotonic_ticks()
        .unwrap_or(support::word(440))
        .saturating_sub(support::word(440));
    support::put(424, elapsed);
    support::put(400, 1);
}

fn fail(code: u64) -> ! {
    publish(code);
    native::exit(code.max(1))
}

fn check(condition: bool, code: u64) {
    if !condition {
        fail(code);
    }
    support::put(408, support::word(408) + 1);
}

fn must<T>(value: Result<T, Error>, code: u64) -> T {
    value.unwrap_or_else(|_| fail(code))
}

fn now() -> u64 {
    must(native::monotonic_ticks(), 90)
}

fn after(base: u64, ticks: u64) -> u64 {
    deadline(base, ticks).unwrap_or_else(|| fail(91))
}

fn phase(number: u64) -> u64 {
    support::put(448, number);
    support::put(8, support::word(8) + 1);
    after(now(), PHASE_TICKS)
}

fn complete(bit: u64) {
    support::put(432, support::word(432) | bit);
}

fn wait(conditions: &[WaitCondition], end: u64, code: u64) -> u16 {
    must(native::ipc_wait(conditions, end), code)
}

fn sleep_until(end: u64, code: u64) {
    check(now() < end, code);
    check(wait(&[], end, code) == 0, code);
    check(now() >= end, code);
}

fn send(to: SendEndpoint, packet: Packet, end: u64) {
    let outgoing = must(packet.outgoing(), 92);
    loop {
        check(now() <= end, 100);
        match native::send(to, &outgoing) {
            Ok(()) => return,
            Err(error) if error.is_again() => {
                check(now() < end, 100);
                check(wait(&[WaitCondition::send(to)], end, 101) == 1, 102);
            }
            Err(_) => fail(103),
        }
    }
}

fn receive(e: &Endpoints, end: u64) -> Packet {
    loop {
        check(now() <= end, 111);
        match native::receive(e.receive) {
            Ok(message) => {
                let result = Packet::received(&message, e.peer_pid);
                if message.cap_count() != 0 {
                    let _ = native::close_received_caps(&message);
                }
                return must(result, 110);
            }
            Err(error) if error.is_again() => {
                check(now() < end, 111);
                check(
                    wait(&[WaitCondition::receive(e.receive)], end, 112) == 1,
                    113,
                );
            }
            Err(_) => fail(114),
        }
    }
}

fn expect(e: &Endpoints, packet: Packet, end: u64) {
    check(receive(e, end) == packet, 115);
}

fn empty(e: &Endpoints, code: u64) {
    check(
        matches!(native::receive(e.receive), Err(error) if error.is_again()),
        code,
    );
}

// Kept local until the SDK exposes its raw syscall adapter. The only caller
// below issues 0x22 with bounded scalar arrays; malformed pointers are passed
// to the kernel and never dereferenced by Rust.
fn raw_wait(args: [u64; 6]) -> Result<u64, Error> {
    let mut primary = 0x22u64;
    let mut secondary = args[2];
    // SAFETY: Same int80 register/memory contract as SDK native::ipc_wait.
    unsafe {
        asm!("int 0x80", inout("rax") primary,
            in("rdi") args[0], in("rsi") args[1], inout("rdx") secondary,
            in("r10") args[3], in("r8") args[4], in("r9") args[5],
            options(nostack));
    }
    decode_return(primary as i64, secondary)
}

fn raw_error(args: [u64; 6], expected: i64, code: u64) {
    check(raw_wait(args) == Err(Error::Kernel(expected)), code);
}

fn negatives(e: &Endpoints) {
    let end = phase(1);
    raw_error([0, 17, end, 0, 0, 0], -1, 120);
    raw_error([0, u64::MAX, end, 0, 0, 0], -1, 121);
    raw_error([1, 0, end, 0, 0, 0], -1, 122);
    for index in 3..6 {
        let mut args = [0, 0, end, 0, 0, 0];
        args[index] = 1;
        raw_error(args, -1, 123 + index as u64);
    }
    raw_error([0, 0, after(now(), 2000), 0, 0, 0], -1, 129);
    raw_error([0, 1, end, 0, 0, 0], -3, 130);
    raw_error([0xffff_8000_0000_0000, 1, end, 0, 0, 0], -3, 131);
    raw_error([0x40010ff8, 1, end, 0, 0, 0], -3, 132);
    for interest in [0, 3, u64::MAX] {
        let wire = [e.root.raw(), interest];
        raw_error([wire.as_ptr() as u64, 1, end, 0, 0, 0], -1, 133);
    }
    let denied = [e.own_send.handle().raw(), Rights::RECV.bits()];
    raw_error([denied.as_ptr() as u64, 1, end, 0, 0, 0], -5, 134);
    // Root SEND is ready on the empty own queue, but must not hide a later
    // invalid handle or a missing receive right, in either array order.
    for reversed in [false, true] {
        for invalid in [[0, 2], denied] {
            let ready = [e.root.raw(), Rights::SEND.bits()];
            let wire = if reversed {
                [invalid, ready]
            } else {
                [ready, invalid]
            };
            let error = if invalid[0] == 0 { -4 } else { -5 };
            raw_error([wire.as_ptr() as u64, 2, end, 0, 0, 0], error, 135);
        }
    }
    check(wait(&[], now(), 136) == 0, 136);
    check(wait(&[WaitCondition::send(e.own_send)], end, 137) == 1, 137);
    check(now() <= end, 138);
    complete(NEGATIVES_BIT);
}

fn immediate(e: &Endpoints) {
    let end = phase(2);
    empty(e, 140);
    let packet = Packet::new(SELF, 0xface_cafe, 0x0123_4567_89ab_cdef);
    send(e.own_send, packet, end);
    let conditions = [
        WaitCondition::send(e.own_send),
        WaitCondition::receive(e.receive),
    ];
    check(wait(&conditions, end, 141) == 3, 141);
    check(wait(&conditions, end, 142) == 3, 142);
    let message = must(native::receive(e.receive), 143);
    check(message.sender() != e.peer_pid, 144);
    check(
        must(Packet::received(&message, message.sender()), 145) == packet,
        145,
    );
    empty(e, 146);
    check(
        wait(&[WaitCondition::send(e.own_send); 16], end, 147) == u16::MAX,
        147,
    );
    check(now() <= end, 148);
    complete(IMMEDIATE_BIT);
}

fn fill(e: &Endpoints, end: u64) {
    for sequence in 0..QUEUE_CAPACITY {
        must(
            native::send(
                e.peer,
                &must(Packet::new(ITEM, sequence, 0).outgoing(), 150),
            ),
            151,
        );
        check(now() < end, 152);
    }
    check(
        matches!(
            native::send(
                e.peer,
                &must(Packet::new(ITEM, QUEUE_CAPACITY, 0).outgoing(), 153),
            ),
            Err(error) if error.is_again()
        ),
        154,
    );
}

fn role_zero(e: &Endpoints) {
    negatives(e);
    immediate(e);
    let end = phase(3);
    send(e.peer, Packet::new(EMPTY, 0, 0), end);
    empty(e, 160);
    let before = now();
    check(
        wait(&[WaitCondition::receive(e.receive)], end, 161) == 1,
        161,
    );
    let elapsed = now().saturating_sub(before);
    check(elapsed > 0, 162);
    support::put(464, elapsed);
    expect(
        e,
        Packet::new(DATA, 0xdead_beef, 0x9876_5432_10ab_cdef),
        end,
    );
    empty(e, 163);
    complete(EMPTY_BIT);

    let end = phase(4);
    let idle_end = after(now(), SETTLE_TICKS);
    send(e.peer, Packet::new(IDLE, 0, idle_end), end);
    expect(e, Packet::new(IDLE_ACK, 0, idle_end), end);
    let before = now();
    sleep_until(idle_end, 170);
    support::put(480, now().saturating_sub(before));
    complete(IDLE_BIT);

    let end = phase(5);
    let drain_at = after(now(), SETTLE_TICKS);
    send(e.peer, Packet::new(FULL, 0, drain_at), end);
    expect(e, Packet::new(FULL_ACK, 0, drain_at), end);
    fill(e, drain_at);
    check(after(now(), 2) < drain_at, 180);
    let before = now();
    check(wait(&[WaitCondition::send(e.peer)], end, 181) == 1, 181);
    let elapsed = now().saturating_sub(before);
    check(elapsed > 0, 182);
    support::put(472, elapsed);
    // Exactly one post-wake send: the pre-wait EAGAIN was not replayed.
    must(
        native::send(
            e.peer,
            &must(Packet::new(ITEM, QUEUE_CAPACITY, 0).outgoing(), 183),
        ),
        184,
    );
    expect(e, Packet::new(DRAINED, QUEUE_CAPACITY + 1, 0), end);
    empty(e, 185);
    complete(FULL_BIT);

    let end = phase(6);
    let revoke_at = after(now(), SETTLE_TICKS);
    send(e.peer, Packet::new(REVOKE, 0, revoke_at), end);
    expect(e, Packet::new(REVOKE_ACK, 0, revoke_at), end);
    fill(e, revoke_at);
    check(after(now(), 2) < revoke_at, 190);
    let before = now();
    check(
        native::ipc_wait(&[WaitCondition::send(e.peer)], end) == Err(Error::Kernel(-10)),
        191,
    );
    let elapsed = now().saturating_sub(before);
    check(elapsed > 0, 192);
    support::put(488, elapsed);
    check(
        native::send(
            e.peer,
            &must(Packet::new(ITEM, QUEUE_CAPACITY, 0).outgoing(), 193),
        ) == Err(Error::Kernel(-10)),
        194,
    );
    check(
        native::ipc_wait(&[WaitCondition::send(e.peer)], end) == Err(Error::Kernel(-10)),
        195,
    );
    expect(e, Packet::new(REVOKED, QUEUE_CAPACITY, revoke_at), end);
    complete(REVOKE_BIT);
}

fn role_one(e: &Endpoints) {
    let end = phase(3);
    expect(e, Packet::new(EMPTY, 0, 0), end);
    sleep_until(after(now(), SETTLE_TICKS), 200);
    send(
        e.peer,
        Packet::new(DATA, 0xdead_beef, 0x9876_5432_10ab_cdef),
        end,
    );
    complete(EMPTY_BIT);

    let end = phase(4);
    let idle = receive(e, end);
    check(idle.kind == IDLE && idle.sequence == 0, 201);
    check(idle.at <= end, 202);
    send(e.peer, Packet::new(IDLE_ACK, 0, idle.at), end);
    let before = now();
    sleep_until(idle.at, 203);
    support::put(480, now().saturating_sub(before));
    complete(IDLE_BIT);

    let end = phase(5);
    let full = receive(e, end);
    check(
        full.kind == FULL && full.sequence == 0 && full.at <= end,
        210,
    );
    send(e.peer, Packet::new(FULL_ACK, 0, full.at), end);
    sleep_until(full.at, 211);
    for sequence in 0..=QUEUE_CAPACITY {
        expect(e, Packet::new(ITEM, sequence, 0), end);
    }
    empty(e, 212);
    support::put(496, u64::from(QUEUE_CAPACITY + 1));
    send(e.peer, Packet::new(DRAINED, QUEUE_CAPACITY + 1, 0), end);
    complete(FULL_BIT);

    let end = phase(6);
    let revoke = receive(e, end);
    check(
        revoke.kind == REVOKE && revoke.sequence == 0 && revoke.at <= end,
        220,
    );
    send(e.peer, Packet::new(REVOKE_ACK, 0, revoke.at), end);
    sleep_until(revoke.at, 221);
    check(
        wait(&[WaitCondition::receive(e.receive)], end, 222) == 1,
        222,
    );
    must(native::revoke(e.root), 223);
    // No receives here: uncapped queue contents must survive descendant revoke.
    check(
        wait(&[WaitCondition::receive(e.receive)], end, 224) == 1,
        224,
    );
    check(
        native::ipc_wait(&[WaitCondition::send(e.own_send)], end) == Err(Error::Kernel(-10)),
        225,
    );
    support::put(504, u64::from(QUEUE_CAPACITY));
    send(e.peer, Packet::new(REVOKED, QUEUE_CAPACITY, revoke.at), end);
    // Keep the receiver/root alive while the sender observes the revoke result;
    // do not let receiver exit/cleanup stand in for the root-revoke syscall.
    sleep_until(after(now(), SETTLE_TICKS), 226);
    complete(REVOKE_BIT);
}

#[no_mangle]
pub extern "C" fn ipc_wait_main() -> ! {
    for offset in [
        400, 408, 416, 424, 432, 440, 448, 464, 472, 480, 488, 496, 504,
    ] {
        support::put(offset, 0);
    }
    must(native::check_abi(), 1);
    check(must(native::abi_info(), 2) == (1 << 32) | 10, 2);
    support::put(440, now());
    let root = must(Handle::from_raw(support::word(56)), 3);
    let e = Endpoints {
        root,
        receive: ReceiveEndpoint::from_handle(root),
        peer: SendEndpoint::from_handle(must(Handle::from_raw(support::word(64)), 4)),
        own_send: SendEndpoint::from_handle(must(Handle::from_raw(support::word(72)), 5)),
        peer_pid: support::word(176),
    };
    check(e.peer_pid != 0, 6);
    let expected_mask = match support::word(32) {
        0 => {
            role_zero(&e);
            ROLE_ZERO_MASK
        }
        1 => {
            role_one(&e);
            ROLE_ONE_MASK
        }
        _ => fail(7),
    };
    check(support::word(432) == expected_mask, 8);
    support::put(448, 7);
    publish(0);
    native::exit(0)
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(255)
}
