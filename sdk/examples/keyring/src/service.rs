#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Native B01 Key Service. It owns one `Entropy` capability and two
//! kernel-bound ingress endpoints; the endpoint a request arrives on is its
//! only domain authority. All key material and all policy live in
//! `keyring-core`; this file only moves bytes between the kernel broker, the
//! IPC envelopes and the core. Key bytes never leave this process.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("keyring-service is only for the Nieuw OS x86_64-unknown-none target");

mod support;

use keyring_core::{
    wire::{self, Header},
    EntropySource, Service, State, Status, MAX_PLAINTEXT, MAX_SEALED, RECOVERY_BLOB, SECRET_BYTES,
};
use nieuw_os_sdk_runtime::{
    native, EntropySource as EntropyHandle, Error, Handle, Outgoing, ReceiveEndpoint, SendEndpoint,
};

const POLL_LIMIT: usize = 200_000;

// Evidence words read by the kernel (see kernel keyring.rs SERVICE_WORDS).
const EV_STATE: u64 = 192;
const EV_REASON: u64 = 200;
const EV_KEYS_D0: u64 = 208;
const EV_KEYS_D1: u64 = 216;
const EV_SEALED: u64 = 224;
const EV_UNSEALED: u64 = 232;
const EV_AUTH_FAILURES: u64 = 240;
const EV_REVOKED: u64 = 248;
const EV_RESTORES: u64 = 256;
const EV_ENTROPY_BYTES: u64 = 264;

fn tick() {
    support::put(8, support::word(8).wrapping_add(1));
}

/// The kernel broker as seen by keyring-core: one request, one bounded wait.
struct Broker {
    source: EntropyHandle,
}
impl EntropySource for Broker {
    fn read(&mut self, out: &mut [u8]) -> Result<(), Status> {
        // The broker serializes requests across holders: EAGAIN means another
        // instance's request is in flight, not a failing source.
        let mut ticket = None;
        for _ in 0..POLL_LIMIT {
            tick();
            match native::entropy_request(self.source, out.len()) {
                Ok(value) => {
                    ticket = Some(value);
                    break;
                }
                Err(error) if error.is_again() => {
                    let _ = native::yield_now();
                }
                Err(_) => return Err(Status::Entropy),
            }
        }
        let ticket = ticket.ok_or(Status::Entropy)?;
        native::entropy_wait(self.source, ticket, out, POLL_LIMIT).map_err(|_| Status::Entropy)
    }
}

fn evidence(service: &Service) {
    let c = service.counters;
    support::put(EV_STATE, service.state() as u8 as u64);
    support::put(EV_REASON, service.reason() as u8 as u64);
    support::put(EV_KEYS_D0, u64::from(service.keys_created(0)));
    support::put(EV_KEYS_D1, u64::from(service.keys_created(1)));
    support::put(EV_SEALED, c.sealed);
    support::put(EV_UNSEALED, c.unsealed);
    support::put(EV_AUTH_FAILURES, c.auth_failures);
    support::put(EV_REVOKED, c.revoked);
    support::put(EV_RESTORES, c.restores);
    support::put(EV_ENTROPY_BYTES, c.entropy_bytes);
}

fn startup_error(step: u64) -> ! {
    support::put(240, step);
    support::put(EV_STATE, 9);
    loop {
        tick();
        let _ = native::yield_now();
    }
}

fn handle(raw: u64) -> Handle {
    Handle::from_raw(raw).unwrap_or_else(|_| startup_error(1))
}

fn send(endpoint: SendEndpoint, body: &[u8; wire::SIZE]) {
    let Ok(message) = Outgoing::new(wire::TAG, body) else {
        startup_error(2);
    };
    for _ in 0..POLL_LIMIT {
        tick();
        match native::send(endpoint, &message) {
            Ok(()) => return,
            Err(error) if error.is_again() => {
                let _ = native::yield_now();
            }
            Err(_) => return,
        }
    }
}

fn dispatch(
    service: &mut Service,
    broker: &mut Broker,
    domain: usize,
    header: &Header,
    body: &[u8],
) -> [u8; wire::SIZE] {
    let Some((low, high)) = wire::body_bounds(header.op) else {
        return wire::reply(header, Status::Invalid, &[]);
    };
    if body.len() < low || body.len() > high || header.status != 0 {
        return wire::reply(header, Status::Invalid, &[]);
    }
    match header.op {
        wire::STATUS => {
            let status = wire::status_body(
                service.state() as u8,
                service.reason() as u8,
                [service.keys_created(0), service.keys_created(1)],
                service.counters.entropy_bytes,
            );
            wire::reply(header, Status::Ok, &status)
        }
        wire::GENERATE => match service.generate(domain, broker) {
            Ok(index) => wire::reply(header, Status::Ok, &[index]),
            Err(status) => wire::reply(header, status, &[]),
        },
        wire::SEAL => {
            let mut out = [0u8; MAX_SEALED];
            match service.seal(domain, header.index, body, broker, &mut out) {
                Ok(length) => wire::reply(header, Status::Ok, &out[..length]),
                Err(status) => wire::reply(header, status, &[]),
            }
        }
        wire::UNSEAL => {
            let mut out = [0u8; MAX_PLAINTEXT];
            match service.unseal(domain, header.index, body, broker, &mut out) {
                Ok(length) => wire::reply(header, Status::Ok, &out[..length]),
                Err(status) => wire::reply(header, status, &[]),
            }
        }
        wire::REVOKE => match service.revoke(domain, header.index) {
            Ok(()) => wire::reply(header, Status::Ok, &[]),
            Err(status) => wire::reply(header, status, &[]),
        },
        wire::EXPORT_RECOVERY => {
            let mut secret = [0u8; SECRET_BYTES];
            let mut blob = [0u8; RECOVERY_BLOB];
            match service.export_recovery(broker, &mut secret, &mut blob) {
                Ok(()) => {
                    let mut out = [0u8; SECRET_BYTES + RECOVERY_BLOB];
                    out[..SECRET_BYTES].copy_from_slice(&secret);
                    out[SECRET_BYTES..].copy_from_slice(&blob);
                    wire::reply(header, Status::Ok, &out)
                }
                Err(status) => wire::reply(header, status, &[]),
            }
        }
        wire::RESTORE => {
            let mut secret = [0u8; SECRET_BYTES];
            let mut blob = [0u8; RECOVERY_BLOB];
            secret.copy_from_slice(&body[..SECRET_BYTES]);
            blob.copy_from_slice(&body[SECRET_BYTES..]);
            match service.restore(&secret, &blob, broker) {
                Ok(domains) => wire::reply(header, Status::Ok, &[domains as u8]),
                Err(status) => wire::reply(header, status, &[]),
            }
        }
        _ => wire::reply(header, Status::Invalid, &[]),
    }
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    main()
}

fn main() -> ! {
    if native::check_abi().is_err() {
        startup_error(3);
    }
    if support::word(0) != 0 || support::word(32) != 1 {
        startup_error(4);
    }
    let mut broker = Broker {
        source: EntropyHandle::from_handle(handle(support::word(48))),
    };
    let ingress = [
        ReceiveEndpoint::from_handle(handle(support::word(56))),
        ReceiveEndpoint::from_handle(handle(support::word(80))),
    ];
    let reply_to = SendEndpoint::from_handle(handle(support::word(88)));
    let app = support::word(160);
    if app == 0 {
        startup_error(5);
    }
    let mut service = Service::new();
    // Startup entropy and health tests decide READY or BLOCKED; a blocked
    // service keeps answering STATUS and refuses every key operation.
    let _ = service.start(&mut broker);
    evidence(&service);
    if service.state() == State::Starting {
        startup_error(6);
    }
    loop {
        for (domain, endpoint) in ingress.iter().enumerate() {
            tick();
            let message = match native::receive(*endpoint) {
                Ok(message) => message,
                Err(_) => continue,
            };
            if message.cap_count() != 0 {
                let _ = native::close_received_caps(&message);
                continue;
            }
            if message.sender() != app || message.tag() != wire::TAG {
                continue;
            }
            let Some(header) = Header::decode(message.payload()) else {
                continue;
            };
            let body = &message.payload()[wire::HEADER..wire::HEADER + usize::from(header.length)];
            let reply = dispatch(&mut service, &mut broker, domain, &header, body);
            evidence(&service);
            send(reply_to, &reply);
        }
        let _ = native::yield_now();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    support::put(EV_STATE, 8);
    native::exit(3)
}

#[allow(dead_code)]
fn unused(_: Error) {}
