#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Native B01 acceptance app. It holds SEND capabilities to both domains of
//! two Key Service instances and one reply endpoint, nothing else. It never
//! sees a key byte: it drives generate/seal/unseal/revoke, probes the
//! negative paths, and verifies that ordinary app authority cannot export or
//! restore recovery material. Every observation is an evidence word the
//! kernel checks.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("keyring-app is only for the Nieuw OS x86_64-unknown-none target");

mod support;

use keyring_core::{
    wire::{self, Header},
    Status, MAX_SEALED, RECOVERY_BLOB, SECRET_BYTES,
};
use nieuw_os_sdk_runtime::{
    native, EntropySource, Error, Handle, Outgoing, ReceiveEndpoint, SendEndpoint,
};

const LIMIT: usize = 400_000;
const CASE_NORMAL: u64 = 0;
const CASE_RESTART: u64 = 4;

// Evidence words (kernel keyring.rs APP_WORDS).
const EV_GENERATED: u64 = 400;
const EV_SEALED: u64 = 408;
const EV_UNSEALED: u64 = 416;
const EV_TAMPER_DENIED: u64 = 424;
const EV_CROSS_DENIED: u64 = 432;
const EV_REVOKED_DENIED: u64 = 440;
const EV_QUOTA_DENIED: u64 = 448;
const EV_RECOVERY_EXPORT_DENIED: u64 = 456;
const EV_RECOVERY_RESTORE_DENIED: u64 = 464;
const EV_NOT_READY: u64 = 488;
const EV_ENTROPY_DENIED: u64 = 496;
const EV_STATUS_CALLS: u64 = 504;

fn tick() {
    support::put(8, support::word(8).wrapping_add(1));
}

fn fail(step: u64) -> ! {
    support::put(240, step);
    support::put(40, 0x100 + step);
    native::exit(1)
}

fn bump(offset: u64) {
    support::put(offset, support::word(offset) + 1);
}

fn handle(raw: u64) -> Handle {
    Handle::from_raw(raw).unwrap_or_else(|_| fail(1))
}

struct Client {
    reply: ReceiveEndpoint,
    request: u64,
}

struct Target {
    send: SendEndpoint,
    service: u64,
}

impl Client {
    /// One request/reply exchange. The reply must come from the addressed
    /// service, carry no capability, echo the request id and decode cleanly.
    fn call(
        &mut self,
        target: &Target,
        op: u8,
        index: u8,
        body: &[u8],
    ) -> (Status, [u8; wire::SIZE], usize) {
        self.request += 1;
        let request = wire::request(op, index, self.request, body).unwrap_or_else(|| fail(10));
        let message = Outgoing::new(wire::TAG, &request).unwrap_or_else(|_| fail(11));
        let mut sent = false;
        for _ in 0..LIMIT {
            tick();
            match native::send(target.send, &message) {
                Ok(()) => {
                    sent = true;
                    break;
                }
                Err(error) if error.is_again() => {
                    let _ = native::yield_now();
                }
                Err(_) => fail(12),
            }
        }
        if !sent {
            fail(13);
        }
        for _ in 0..LIMIT {
            tick();
            match native::receive(self.reply) {
                Ok(reply) => {
                    if reply.cap_count() != 0 {
                        let _ = native::close_received_caps(&reply);
                        fail(14);
                    }
                    if reply.sender() != target.service || reply.tag() != wire::TAG {
                        fail(15);
                    }
                    let header = Header::decode(reply.payload()).unwrap_or_else(|| fail(16));
                    if header.request != self.request
                        || header.op != op | wire::REPLY
                        || header.index != index
                    {
                        fail(17);
                    }
                    let mut out = [0u8; wire::SIZE];
                    let length = usize::from(header.length);
                    out[..length]
                        .copy_from_slice(&reply.payload()[wire::HEADER..wire::HEADER + length]);
                    return (Status::from_code(header.status), out, length);
                }
                Err(error) if error.is_again() => {
                    let _ = native::yield_now();
                }
                Err(_) => fail(18),
            }
        }
        fail(19)
    }

    fn status(&mut self, target: &Target) -> (u8, u8, [u16; 2]) {
        let (status, body, length) = self.call(target, wire::STATUS, 0, &[]);
        if status != Status::Ok || length != 14 {
            fail(20);
        }
        bump(EV_STATUS_CALLS);
        (
            body[0],
            body[1],
            [
                u16::from_le_bytes([body[2], body[3]]),
                u16::from_le_bytes([body[4], body[5]]),
            ],
        )
    }

    fn generate(&mut self, target: &Target) -> Result<u8, Status> {
        let (status, body, length) = self.call(target, wire::GENERATE, 0, &[]);
        match status {
            Status::Ok if length == 1 => {
                bump(EV_GENERATED);
                Ok(body[0])
            }
            Status::Ok => fail(21),
            Status::NotReady => {
                bump(EV_NOT_READY);
                Err(status)
            }
            other => Err(other),
        }
    }

    fn seal(
        &mut self,
        target: &Target,
        index: u8,
        plaintext: &[u8],
    ) -> Result<([u8; MAX_SEALED], usize), Status> {
        let (status, body, length) = self.call(target, wire::SEAL, index, plaintext);
        if status != Status::Ok {
            return Err(status);
        }
        if length != plaintext.len() + 40 {
            fail(22);
        }
        bump(EV_SEALED);
        let mut out = [0u8; MAX_SEALED];
        out[..length].copy_from_slice(&body[..length]);
        Ok((out, length))
    }

    fn unseal(
        &mut self,
        target: &Target,
        index: u8,
        sealed: &[u8],
    ) -> Result<([u8; wire::SIZE], usize), Status> {
        let (status, body, length) = self.call(target, wire::UNSEAL, index, sealed);
        if status != Status::Ok {
            return Err(status);
        }
        bump(EV_UNSEALED);
        Ok((body, length))
    }

    fn expect_unseal_denied(
        &mut self,
        target: &Target,
        index: u8,
        sealed: &[u8],
        expected: Status,
        counter: u64,
        step: u64,
    ) {
        match self.unseal(target, index, sealed) {
            Err(status) if status == expected => bump(counter),
            _ => fail(step),
        }
    }
}

fn run_normal(client: &mut Client, a0: &Target, a1: &Target, b0: &Target) {
    let plaintext = b"Materiaal: geheim dat alleen de sleuteldienst kan openen.";
    // 1. Both instances are ready with keys = 0.
    let (state_a, reason_a, keys_a) = client.status(a0);
    let (state_b, reason_b, keys_b) = client.status(b0);
    if state_a != 1
        || state_b != 1
        || reason_a != 0
        || reason_b != 0
        || keys_a != [0, 0]
        || keys_b != [0, 0]
    {
        fail(30);
    }
    // 2. Generate, seal, unseal in domain 0 of A.
    let k0 = client.generate(a0).unwrap_or_else(|_| fail(31));
    let k1 = client.generate(a0).unwrap_or_else(|_| fail(32));
    if (k0, k1) != (0, 1) {
        fail(33);
    }
    let (sealed, length) = client.seal(a0, k0, plaintext).unwrap_or_else(|_| fail(34));
    let (plain, n) = client
        .unseal(a0, k0, &sealed[..length])
        .unwrap_or_else(|_| fail(35));
    if n != plaintext.len() || plain[..n] != plaintext[..] {
        fail(36);
    }
    // 3. Tampering: nonce byte, ciphertext byte, tag byte, truncation.
    for position in [0usize, 30, length - 1] {
        let mut bad = sealed;
        bad[position] ^= 0x01;
        client.expect_unseal_denied(a0, k0, &bad[..length], Status::Auth, EV_TAMPER_DENIED, 37);
    }
    client.expect_unseal_denied(
        a0,
        k0,
        &sealed[..length - 1],
        Status::Auth,
        EV_TAMPER_DENIED,
        38,
    );
    // 4. Wrong key of the same domain, and the other domain's key.
    client.expect_unseal_denied(a0, k1, &sealed[..length], Status::Auth, EV_CROSS_DENIED, 39);
    let other = client.generate(a1).unwrap_or_else(|_| fail(40));
    client.expect_unseal_denied(
        a1,
        other,
        &sealed[..length],
        Status::Auth,
        EV_CROSS_DENIED,
        41,
    );
    // Domain 1 has one key: index 1 does not exist there.
    client.expect_unseal_denied(
        a1,
        k1,
        &sealed[..length],
        Status::BadHandle,
        EV_CROSS_DENIED,
        42,
    );
    // 5. Revocation is permanent.
    let (status, _, _) = client.call(a0, wire::REVOKE, k1, &[]);
    if status != Status::Ok {
        fail(43);
    }
    match client.seal(a0, k1, b"x") {
        Err(Status::Revoked) => bump(EV_REVOKED_DENIED),
        _ => fail(44),
    }
    let (status, _, _) = client.call(a0, wire::REVOKE, k1, &[]);
    if status != Status::Revoked {
        fail(45);
    }
    // 6. Quota: 16 keys per domain.
    for _ in 2..16 {
        client.generate(a0).unwrap_or_else(|_| fail(46));
    }
    match client.generate(a0) {
        Err(Status::Quota) => bump(EV_QUOTA_DENIED),
        _ => fail(47),
    }
    // 7. Recovery export/import belongs to the manifest-authorized management
    // ingress, never to an ordinary app's domain SEND capabilities.
    let (status, body, n) = client.call(a0, wire::EXPORT_RECOVERY, 0, &[]);
    if status != Status::Invalid || n != 0 || body.iter().any(|byte| *byte != 0) {
        fail(48);
    }
    bump(EV_RECOVERY_EXPORT_DENIED);
    let recovery = [0u8; SECRET_BYTES + RECOVERY_BLOB];
    let (status, body, n) = client.call(b0, wire::RESTORE, 0, &recovery);
    if status != Status::Invalid || n != 0 || body.iter().any(|byte| *byte != 0) {
        fail(49);
    }
    bump(EV_RECOVERY_RESTORE_DENIED);
}

fn run_blocked(client: &mut Client, a0: &Target, b0: &Target, expected_reason: u8) {
    for target in [a0, b0] {
        let (state, reason, keys) = client.status(target);
        if state != 2 || reason != expected_reason || keys != [0, 0] {
            fail(60);
        }
        match client.generate(target) {
            Err(Status::NotReady) => {}
            _ => fail(61),
        }
        match client.seal(target, 0, b"x") {
            Err(Status::NotReady) => {}
            _ => fail(62),
        }
    }
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    main()
}

fn main() -> ! {
    if native::check_abi().is_err() {
        fail(2);
    }
    if support::word(0) != 2 || support::word(32) != 1 {
        fail(3);
    }
    let case = support::word(112);
    let reply_handle = handle(support::word(56));
    let mut client = Client {
        reply: ReceiveEndpoint::from_handle(reply_handle),
        request: 0,
    };
    let a = support::word(160);
    let b = support::word(168);
    let a0 = Target {
        send: SendEndpoint::from_handle(handle(support::word(64))),
        service: a,
    };
    let a1 = Target {
        send: SendEndpoint::from_handle(handle(support::word(72))),
        service: a,
    };
    let b0 = Target {
        send: SendEndpoint::from_handle(handle(support::word(80))),
        service: b,
    };
    // An app owns no entropy right: the broker must refuse its endpoint handle.
    match native::entropy_request(EntropySource::from_handle(reply_handle), 32) {
        Err(Error::Kernel(-6)) => bump(EV_ENTROPY_DENIED),
        _ => fail(4),
    }
    if matches!(case, CASE_NORMAL | CASE_RESTART) {
        run_normal(&mut client, &a0, &a1, &b0);
    } else {
        // no-device and timeout: Source (1); stuck: Repetition (2).
        run_blocked(&mut client, &a0, &b0, if case == 2 { 2 } else { 1 });
    }
    support::put(40, 1);
    native::exit(0)
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    support::put(40, 0x1ff);
    native::exit(3)
}
