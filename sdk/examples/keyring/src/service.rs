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
const EV_RANDOM_CALLS: u64 = 272;
const EV_RANDOM_BYTES: u64 = 280;
const EV_RANDOM_SEEDS: u64 = 288;
const EV_RANDOM_SEEN: u64 = 296;
const EV_RANDOM_BAD_CALLER: u64 = 304;
const EV_RANDOM_INVALID_OPCODE: u64 = 312;
const EV_RANDOM_RETIRED: u64 = 320;
const EV_RANDOM_PENDING: u64 = 328;
const EV_RANDOM_INVALID_REQUEST: u64 = 336;
const EV_RANDOM_REPLIES: u64 = 344;
const EV_RANDOM_REPLY_CLOSED: u64 = 352;
const EV_RANDOM_REPLY_TIMEOUT: u64 = 360;
const EV_LEGACY_POLLS: u64 = 368;
const EV_LEGACY_POLL_ERRORS: u64 = 376;
// B02 administrative ingress evidence (optional third endpoint).
const EV_ADMIN_SEEN: u64 = 384;
const EV_ADMIN_EXPORTS: u64 = 392;
const EV_ADMIN_RESTORES: u64 = 400;
const EV_ADMIN_DENIED: u64 = 408;
const EV_ADMIN_BAD_CALLER: u64 = 416;
const RANDOM_DESCRIPTOR: u64 = u64::from_le_bytes(*b"NOSRNG01");
const RANDOM_REPLY_POLLS: usize = 4096;
const RANDOM_REPLY_TICKS: u64 = 100;

/// Persistence-only administrative ingress. Only the single manifest-named
/// caller (a supervisor/recovery agent) may use it, and it dispatches
/// exclusively through `wire::dispatch_persistent`: no key, record, recovery
/// or RANDOM operation is reachable here, and the domain endpoints never
/// reach the persistence codec. The caller-held secret arrives in the request
/// body; the service keeps no copy beyond the dispatch call.
struct AdminIngress {
    receive: ReceiveEndpoint,
    reply: SendEndpoint,
    caller: u64,
    seen: u64,
    exports: u64,
    restores: u64,
    denied: u64,
    bad_caller: u64,
}

struct RandomIngress {
    receive: ReceiveEndpoint,
    reply: SendEndpoint,
    caller: u64,
    domain: usize,
    pending: [u8; wire::SIZE],
    has_pending: bool,
    polls: usize,
    deadline: Option<u64>,
    last_tick: u64,
    retired: bool,
    seen: u64,
    bad_caller: u64,
    invalid_opcode: u64,
    invalid_request: u64,
    replies: u64,
    closed: u64,
    timeout: u64,
}

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
    let random = service.random_counters();
    support::put(EV_RANDOM_CALLS, random.calls);
    support::put(EV_RANDOM_BYTES, random.bytes);
    support::put(EV_RANDOM_SEEDS, random.seeds);
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
    let Ok(mut message) = Outgoing::new(wire::TAG, body) else {
        startup_error(2);
    };
    for _ in 0..POLL_LIMIT {
        tick();
        match native::send(endpoint, &message) {
            Ok(()) => break,
            Err(error) if error.is_again() => {
                let _ = native::yield_now();
            }
            Err(_) => break,
        }
    }
    wipe_outgoing(&mut message);
}

fn wipe_outgoing(message: &mut Outgoing) {
    // SAFETY: Exclusive owned, no-cap envelope; all send borrows have ended.
    // A valid empty envelope overwrites its current payload with volatile
    // typed stores, without relying on the SDK's private field layout.
    // No destructor/resource may be silently skipped by this replacement.
    const { assert!(!core::mem::needs_drop::<Outgoing>()) };
    unsafe { core::ptr::write_volatile(message, Outgoing::new(0, &[]).unwrap()) };
}

fn random_ingress(legacy: &[ReceiveEndpoint; 2], reply: SendEndpoint) -> Option<RandomIngress> {
    let magic = support::word(120);
    let words = [
        support::word(128),
        support::word(136),
        support::word(144),
        support::word(152),
    ];
    if magic == 0 {
        if words.iter().any(|value| *value != 0) {
            startup_error(7);
        }
        return None;
    }
    if magic != RANDOM_DESCRIPTOR
        || words[2] == 0
        || words[3] >= keyring_core::random::DOMAINS as u64
    {
        startup_error(7);
    }
    let receive = handle(words[0]);
    let send_reply = handle(words[1]);
    if receive == send_reply
        || receive == reply.handle()
        || send_reply == reply.handle()
        || legacy
            .iter()
            .any(|endpoint| endpoint.handle() == receive || endpoint.handle() == send_reply)
    {
        startup_error(7);
    }
    Some(RandomIngress {
        receive: ReceiveEndpoint::from_handle(receive),
        reply: SendEndpoint::from_handle(send_reply),
        caller: words[2],
        domain: words[3] as usize,
        pending: [0; wire::SIZE],
        has_pending: false,
        polls: 0,
        deadline: None,
        last_tick: 0,
        retired: false,
        seen: 0,
        bad_caller: 0,
        invalid_opcode: 0,
        invalid_request: 0,
        replies: 0,
        closed: 0,
        timeout: 0,
    })
}

fn admin_ingress(legacy: &[ReceiveEndpoint; 2], reply: SendEndpoint) -> Option<AdminIngress> {
    let words = [support::word(96), support::word(104), support::word(168)];
    if words[0] == 0 {
        if words[1] != 0 || words[2] != 0 {
            startup_error(10);
        }
        return None;
    }
    if words[1] == 0 || words[2] == 0 {
        startup_error(10);
    }
    let receive = handle(words[0]);
    let send_reply = handle(words[1]);
    if receive == send_reply
        || receive == reply.handle()
        || send_reply == reply.handle()
        || legacy
            .iter()
            .any(|endpoint| endpoint.handle() == receive || endpoint.handle() == send_reply)
    {
        startup_error(10);
    }
    Some(AdminIngress {
        receive: ReceiveEndpoint::from_handle(receive),
        reply: SendEndpoint::from_handle(send_reply),
        caller: words[2],
        seen: 0,
        exports: 0,
        restores: 0,
        denied: 0,
        bad_caller: 0,
    })
}

fn admin_evidence(ingress: &AdminIngress) {
    for (at, value) in [
        (EV_ADMIN_SEEN, ingress.seen),
        (EV_ADMIN_EXPORTS, ingress.exports),
        (EV_ADMIN_RESTORES, ingress.restores),
        (EV_ADMIN_DENIED, ingress.denied),
        (EV_ADMIN_BAD_CALLER, ingress.bad_caller),
    ] {
        support::put(at, value);
    }
}

/// One administrative request per turn. Sender, capability count, tag and
/// header are checked before any body byte reaches the persistence codec.
fn poll_admin(service: &mut Service, broker: &mut Broker, ingress: &mut AdminIngress) {
    let message = match native::receive(ingress.receive) {
        Ok(message) => message,
        Err(_) => return,
    };
    ingress.seen = ingress.seen.saturating_add(1);
    if message.cap_count() != 0 {
        let _ = native::close_received_caps(&message);
        ingress.denied = ingress.denied.saturating_add(1);
        admin_evidence(ingress);
        return;
    }
    if message.sender() != ingress.caller {
        ingress.bad_caller = ingress.bad_caller.saturating_add(1);
        admin_evidence(ingress);
        return;
    }
    if message.tag() != wire::TAG {
        ingress.denied = ingress.denied.saturating_add(1);
        admin_evidence(ingress);
        return;
    }
    let Some(header) = Header::decode(message.payload()) else {
        ingress.denied = ingress.denied.saturating_add(1);
        admin_evidence(ingress);
        return;
    };
    let body = &message.payload()[wire::HEADER..wire::HEADER + usize::from(header.length)];
    let reply = match header.op {
        wire::EXPORT_PERSISTENT | wire::RESTORE_PERSISTENT => {
            wire::dispatch_persistent(service, broker, &header, body)
        }
        _ => wire::reply(&header, Status::Invalid, &[]),
    };
    let status = Header::decode(&reply).map(|h| h.status);
    if status == Some(Status::Ok.code()) {
        if header.op == wire::EXPORT_PERSISTENT {
            ingress.exports = ingress.exports.saturating_add(1);
        } else {
            ingress.restores = ingress.restores.saturating_add(1);
        }
    } else {
        ingress.denied = ingress.denied.saturating_add(1);
    }
    evidence(service);
    admin_evidence(ingress);
    send(ingress.reply, &reply);
}

fn random_evidence(ingress: &RandomIngress) {
    for (at, value) in [
        (EV_RANDOM_SEEN, ingress.seen),
        (EV_RANDOM_BAD_CALLER, ingress.bad_caller),
        (EV_RANDOM_INVALID_OPCODE, ingress.invalid_opcode),
        (EV_RANDOM_RETIRED, u64::from(ingress.retired)),
        (EV_RANDOM_PENDING, u64::from(ingress.has_pending)),
        (EV_RANDOM_INVALID_REQUEST, ingress.invalid_request),
        (EV_RANDOM_REPLIES, ingress.replies),
        (EV_RANDOM_REPLY_CLOSED, ingress.closed),
        (EV_RANDOM_REPLY_TIMEOUT, ingress.timeout),
    ] {
        support::put(at, value);
    }
}

fn retire_random(ingress: &mut RandomIngress) {
    keyring_core::random::clear_bytes(&mut ingress.pending);
    ingress.has_pending = false;
    ingress.retired = true;
    ingress.deadline = None;
    random_evidence(ingress);
}

/// One nonblocking reply attempt per main-loop turn. A full or closed RANDOM
/// reply endpoint must never hold legacy key/volume RPCs inside a retry loop.
fn send_random(ingress: &mut RandomIngress) {
    if !ingress.has_pending || ingress.retired {
        return;
    }
    if let Some(deadline) = ingress.deadline {
        match native::monotonic_ticks() {
            Ok(now) if now >= ingress.last_tick && now < deadline => ingress.last_tick = now,
            _ => {
                ingress.timeout = ingress.timeout.saturating_add(1);
                retire_random(ingress);
                return;
            }
        }
    }
    let Ok(mut message) = Outgoing::new(wire::TAG, &ingress.pending) else {
        retire_random(ingress);
        return;
    };
    let result = native::send(ingress.reply, &message);
    wipe_outgoing(&mut message);
    match result {
        Ok(()) => {
            ingress.replies = ingress.replies.saturating_add(1);
            keyring_core::random::clear_bytes(&mut ingress.pending);
            ingress.has_pending = false;
            ingress.polls = 0;
            ingress.deadline = None;
        }
        Err(error) if error.is_again() => {
            ingress.polls += 1;
            if ingress.polls >= RANDOM_REPLY_POLLS {
                ingress.timeout = ingress.timeout.saturating_add(1);
                retire_random(ingress);
                return;
            }
            if ingress.deadline.is_none() {
                let deadline = native::monotonic_ticks().ok().and_then(|now| {
                    ingress.last_tick = now;
                    now.checked_add(RANDOM_REPLY_TICKS)
                });
                let Some(deadline) = deadline else {
                    ingress.timeout = ingress.timeout.saturating_add(1);
                    retire_random(ingress);
                    return;
                };
                ingress.deadline = Some(deadline);
            }
        }
        Err(error) => {
            if error == Error::Kernel(-9) {
                ingress.closed = ingress.closed.saturating_add(1);
            }
            retire_random(ingress);
            return;
        }
    }
    random_evidence(ingress);
}

fn poll_random(service: &mut Service, broker: &mut Broker, ingress: &mut RandomIngress) {
    if ingress.retired {
        return;
    }
    if ingress.has_pending {
        send_random(ingress);
        return;
    }
    let message = match native::receive(ingress.receive) {
        Ok(message) => message,
        Err(_) => return,
    };
    ingress.seen = ingress.seen.saturating_add(1);
    random_evidence(ingress);
    if message.cap_count() != 0 {
        let _ = native::close_received_caps(&message);
        return;
    }
    if message.sender() != ingress.caller {
        ingress.bad_caller = ingress.bad_caller.saturating_add(1);
        random_evidence(ingress);
        return;
    }
    if message.tag() != wire::TAG {
        return;
    }
    let Some(header) = Header::decode(message.payload()) else {
        return;
    };
    let body = &message.payload()[wire::HEADER..wire::HEADER + usize::from(header.length)];
    // Validate all format/opcode fields BEFORE consulting a clock or entropy.
    // There is no route from this endpoint to the legacy dispatch below.
    let mut reply = if header.op != wire::RANDOM {
        ingress.invalid_opcode = ingress.invalid_opcode.saturating_add(1);
        wire::reply(&header, Status::Invalid, &[])
    } else if wire::random_length(&header, body).is_err() {
        ingress.invalid_request = ingress.invalid_request.saturating_add(1);
        wire::reply(&header, Status::Invalid, &[])
    } else {
        match native::monotonic_ticks() {
            Ok(now) => wire::dispatch_random(service, broker, ingress.domain, now, &header, body),
            Err(_) => wire::reply(&header, Status::NotReady, &[]),
        }
    };
    ingress.pending.copy_from_slice(&reply);
    keyring_core::random::clear_bytes(&mut reply);
    ingress.has_pending = true;
    ingress.polls = 0;
    ingress.deadline = None;
    // Publish pending before updated output counters become observable. The
    // kernel may preempt between these individual evidence-word stores.
    random_evidence(ingress);
    evidence(service);
    send_random(ingress);
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
        wire::SEAL_RECORD | wire::UNSEAL_RECORD => {
            wire::dispatch_record(service, broker, domain, header, body)
        }
        // Persistence is administrative: never reachable from a domain endpoint.
        wire::EXPORT_PERSISTENT | wire::RESTORE_PERSISTENT => {
            wire::reply(header, Status::Invalid, &[])
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
    let mut random = random_ingress(&ingress, reply_to);
    let mut admin = admin_ingress(&ingress, reply_to);
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
    let (mut legacy_polls, mut legacy_poll_errors) = (0u64, 0u64);
    loop {
        for (domain, endpoint) in ingress.iter().enumerate() {
            tick();
            let received = native::receive(*endpoint);
            match &received {
                Ok(_) => legacy_polls = legacy_polls.saturating_add(1),
                Err(error) if error.is_again() => legacy_polls = legacy_polls.saturating_add(1),
                Err(_) => legacy_poll_errors = legacy_poll_errors.saturating_add(1),
            }
            support::put(EV_LEGACY_POLLS, legacy_polls);
            support::put(EV_LEGACY_POLL_ERRORS, legacy_poll_errors);
            let message = match received {
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
        if let Some(ingress) = &mut random {
            poll_random(&mut service, &mut broker, ingress);
        }
        if let Some(ingress) = &mut admin {
            poll_admin(&mut service, &mut broker, ingress);
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
