#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Native B01 Key Service. It owns one `Entropy` capability and two
//! kernel-bound ingress endpoints; the endpoint a request arrives on is its
//! only domain authority. All key material and all policy live in
//! `keyring-core`; this file only moves bytes between the kernel broker, the
//! IPC envelopes and the core. Key bytes never leave this process through the
//! domain or RANDOM endpoints. The optional administrative ingress is the one
//! exception by design: EXPORT_PERSISTENT hands the holder the master key
//! wrapped under a secret that holder chooses, so a SEND to that ingress is
//! equivalent to the whole key namespace and may only belong to the
//! supervisor/recovery principal named by the kernel manifest.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("keyring-service is only for the Nieuw OS x86_64-unknown-none target");

mod support;

use keyring_core::{
    persistent::{KeyStateStore, StoreError, PERSISTENT_BLOB},
    wire::{self, Header, HeldSecret},
    EntropySource, Service, State, Status,
};
use nieuw_os_sdk_runtime::{
    native, supports_ipc_wait, EntropySource as EntropyHandle, Error, Handle, Outgoing,
    ReceiveEndpoint, SendEndpoint, WaitCondition, IPC_WAIT_HORIZON_TICKS,
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
// B01-T2 requester-crash startup words (see kernel keyring.rs CRASH_ROLE).
const CASE_REQUESTER_CRASH: u64 = 5;
const CRASH_ROLE: u64 = 424;
const RELEASE: u64 = 432;
// ADR-B02-KEY1-RELAY-20261123 (test profile): only the kernel's relay branch
// writes these three startup words; all zero keeps every existing path.
const RELAY_COMMIT_SEND: u64 = 496;
const RELAY_REPLY_RECEIVE: u64 = 504;
const RELAY_INIT_PID: u64 = 512;
const EV_COMMIT_REQUESTS: u64 = 440;
const EV_COMMIT_OK: u64 = 448;
const EV_COMMIT_NOT_COMMITTED: u64 = 456;
const EV_COMMIT_UNKNOWN: u64 = 464;
const EV_COMMIT_LAST_GENERATION: u64 = 472;
const EV_SECRET_SUPPLIED: u64 = 480;
const EV_SECRET_REFUSED: u64 = 488;
const EV_COMMIT_MAX_TICKS: u64 = 520;
/// The profile's volume identity (ADR-B02-KEY1-RELAY item 5). A compile-time
/// constant: the first durable bind can only be to this volume.
const RELAY_VOLUME: [u8; wire::VOLUME_ID] = *b"NOS-SYSTEM-VOL01";

/// Write-ahead store backed by init's key-state commit ingress. One commit per
/// call, synchronous and bounded by `wire::COMMIT_TICKS`; every ambiguous
/// result (timeout, transport error, malformed or foreign reply) is `Unknown`.
struct RelayStore {
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    init: u64,
}

fn bump(at: u64) {
    support::put(at, support::word(at).saturating_add(1));
}

impl RelayStore {
    fn exchange(
        &mut self,
        generation: u64,
        blob: &[u8; PERSISTENT_BLOB],
        deadline: u64,
        observed: &mut u64,
    ) -> Result<(), StoreError> {
        let request = wire::commit_request(generation, blob).ok_or(StoreError::Unknown)?;
        let message = Outgoing::new(wire::TAG, &request).map_err(|_| StoreError::Unknown)?;
        // `observed` keeps the last clock reading, so the latency evidence is the
        // same observation that classified the deadline.
        let mut expired = || match native::monotonic_ticks() {
            Ok(now) => {
                *observed = now;
                now >= deadline
            }
            Err(_) => true,
        };
        loop {
            tick();
            match native::send(self.send, &message) {
                Ok(()) => break,
                Err(error) if error.is_again() && !expired() => {
                    let _ = native::yield_now();
                }
                Err(_) => return Err(StoreError::Unknown),
            }
        }
        loop {
            tick();
            // Checked before every receive: a reply already queued after the
            // deadline is never accepted (ADR: every wait is bounded).
            if expired() {
                return Err(StoreError::Unknown);
            }
            match native::receive(self.receive) {
                Ok(reply) => {
                    if reply.cap_count() != 0 {
                        let _ = native::close_received_caps(&reply);
                        return Err(StoreError::Unknown);
                    }
                    if reply.sender() != self.init
                        || reply.tag() != wire::TAG
                        || reply.payload().len() != wire::SIZE
                    {
                        return Err(StoreError::Unknown);
                    }
                    return wire::commit_outcome(generation, reply.payload());
                }
                Err(error) if error.is_again() => {
                    let _ = native::yield_now();
                }
                Err(_) => return Err(StoreError::Unknown),
            }
        }
    }
}

impl KeyStateStore for RelayStore {
    fn commit(&mut self, generation: u64, blob: &[u8; PERSISTENT_BLOB]) -> Result<(), StoreError> {
        bump(EV_COMMIT_REQUESTS);
        let Ok(start) = native::monotonic_ticks() else {
            bump(EV_COMMIT_UNKNOWN);
            return Err(StoreError::Unknown);
        };
        let mut observed = start;
        let result = match start.checked_add(wire::COMMIT_TICKS) {
            Some(deadline) => self.exchange(generation, blob, deadline, &mut observed),
            None => Err(StoreError::Unknown),
        };
        let elapsed = observed.saturating_sub(start);
        if elapsed > support::word(EV_COMMIT_MAX_TICKS) {
            support::put(EV_COMMIT_MAX_TICKS, elapsed);
        }
        match result {
            Ok(()) => {
                bump(EV_COMMIT_OK);
                support::put(EV_COMMIT_LAST_GENERATION, generation);
            }
            Err(StoreError::Failed) => bump(EV_COMMIT_NOT_COMMITTED),
            Err(StoreError::Unknown) => bump(EV_COMMIT_UNKNOWN),
        }
        result
    }
}

/// Relay configuration from the startup page. Any partial, overlapping or
/// unsupported combination stops the service before it serves anything.
fn relay_store(admin: Option<&AdminIngress>, random: bool) -> Option<RelayStore> {
    let words = [
        support::word(RELAY_COMMIT_SEND),
        support::word(RELAY_REPLY_RECEIVE),
        support::word(RELAY_INIT_PID),
    ];
    if words.iter().all(|value| *value == 0) {
        return None;
    }
    let others = [48, 56, 80, 88, 96, 104].map(support::word);
    let Some(admin) = admin else {
        startup_error(12);
    };
    if words.contains(&0)
        || random
        || words[0] == words[1]
        || others.contains(&words[0])
        || others.contains(&words[1])
        || words[2] != admin.caller
        || native::monotonic_ticks().is_err()
    {
        startup_error(12);
    }
    let send = Handle::from_raw(words[0]).unwrap_or_else(|_| startup_error(12));
    let receive = Handle::from_raw(words[1]).unwrap_or_else(|_| startup_error(12));
    Some(RelayStore {
        send: SendEndpoint::from_handle(send),
        receive: ReceiveEndpoint::from_handle(receive),
        init: words[2],
    })
}

/// Recovery and persistence ingress. Only the single manifest-named
/// caller (a supervisor/recovery agent) may use it, and it dispatches
/// through `wire::dispatch_management`; key and record operations are never
/// reachable here, and domain endpoints cannot export/import recovery
/// material. Caller-held secrets are not retained after dispatch.
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
    let Ok(message) = Outgoing::new(wire::TAG, body) else {
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
    // Defense in depth against a manifest mistake: the persistence caller can
    // never be the ordinary key client or this service itself.
    if words[1] == 0
        || words[2] == 0
        || words[2] == support::word(160)
        || words[2] == support::word(176)
    {
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
/// header are checked before request bytes reach the recovery/persistence code.
fn poll_admin(
    service: &mut Service,
    broker: &mut Broker,
    ingress: &mut AdminIngress,
    held: Option<&mut Option<HeldSecret>>,
) {
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
    if let Some(held) = held {
        // Relay mode: one SUPPLY_SECRET per instance; every other opcode,
        // including EXPORT/RESTORE_PERSISTENT, is refused without dispatch.
        let accepted = if header.op == wire::SUPPLY_SECRET && held.is_none() {
            wire::decode_supply_secret(&header, body)
                .ok()
                .map(|secret| *held = Some(secret))
                .is_some()
        } else {
            false
        };
        if accepted {
            support::put(EV_SECRET_SUPPLIED, 1);
        } else {
            if header.op == wire::SUPPLY_SECRET {
                bump(EV_SECRET_REFUSED);
            }
            ingress.denied = ingress.denied.saturating_add(1);
        }
        let status = if accepted {
            Status::Ok
        } else {
            Status::Invalid
        };
        admin_evidence(ingress);
        send(ingress.reply, &wire::reply(&header, status, &[]));
        return;
    }
    let mut reply = wire::dispatch_management(service, broker, &header, body);
    let status = Header::decode(&reply).map(|h| h.status);
    if status == Some(Status::Ok.code()) {
        if matches!(header.op, wire::EXPORT_PERSISTENT | wire::EXPORT_RECOVERY) {
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
    keyring_core::random::clear_bytes(&mut reply);
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
    let Ok(message) = Outgoing::new(wire::TAG, &ingress.pending) else {
        retire_random(ingress);
        return;
    };
    let result = native::send(ingress.reply, &message);
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
    if support::word(112) == CASE_REQUESTER_CRASH {
        if support::word(CRASH_ROLE) == 1 {
            // Labelled B01 fault injection: die with one real request pending.
            if native::entropy_request(broker.source, 256).is_err() {
                startup_error(11);
            }
            // SAFETY: ud2 raises #UD without touching memory, the stack or
            // registers; the kernel terminates this process and never resumes it.
            unsafe { core::arch::asm!("ud2", options(noreturn)) }
        }
        // Service A starts only after the kernel verified the quarantine.
        while support::word(RELEASE) == 0 {
            tick();
            let _ = native::yield_now();
        }
    }
    let mut random = random_ingress(&ingress, reply_to);
    let mut admin = admin_ingress(&ingress, reply_to);
    let mut relay = relay_store(admin.as_ref(), random.is_some());
    let mut held: Option<HeldSecret> = None;
    // The administrative and RANDOM ingresses never share an endpoint handle.
    if let (Some(admin), Some(random)) = (&admin, &random) {
        let admin_handles = [admin.receive.handle(), admin.reply.handle()];
        if admin_handles.contains(&random.receive.handle())
            || admin_handles.contains(&random.reply.handle())
        {
            startup_error(10);
        }
    }
    // RANDOM and administrative ingress retain their existing polling paths.
    let idle_wait = supports_ipc_wait(native::abi_info().unwrap_or_else(|_| startup_error(3)))
        && random.is_none()
        && admin.is_none();
    let conditions = ingress.map(WaitCondition::receive);
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
        // Relay: no domain receive until init supplied the secret.
        if relay.is_some() && held.is_none() {
            if let Some(ingress) = &mut admin {
                poll_admin(&mut service, &mut broker, ingress, Some(&mut held));
            }
            tick();
            let _ = native::yield_now();
            continue;
        }
        let mut received_any = false;
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
                Ok(message) => {
                    received_any = true;
                    message
                }
                Err(error) if idle_wait && !error.is_again() => startup_error(7),
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
            let reply = if let Some(store) = &mut relay {
                let Some(secret) = &held else {
                    startup_error(12);
                };
                let mut durable = wire::Durable {
                    volume: &RELAY_VOLUME,
                    secret: secret.secret(),
                    store,
                };
                wire::dispatch_domain_durable(
                    &mut service,
                    &mut broker,
                    domain,
                    &header,
                    body,
                    &mut durable,
                )
            } else {
                wire::dispatch_domain(&mut service, &mut broker, domain, &header, body)
            };
            evidence(&service);
            send(reply_to, &reply);
        }
        if let Some(ingress) = &mut random {
            poll_random(&mut service, &mut broker, ingress);
        }
        if let Some(ingress) = &mut admin {
            let relay_held = if relay.is_some() {
                Some(&mut held)
            } else {
                None
            };
            poll_admin(&mut service, &mut broker, ingress, relay_held);
        }
        if idle_wait && !received_any {
            let deadline = native::monotonic_ticks()
                .ok()
                .and_then(|now| now.checked_add(IPC_WAIT_HORIZON_TICKS))
                .unwrap_or_else(|| startup_error(7));
            if native::ipc_wait(&conditions, deadline).is_err() {
                startup_error(7);
            }
        } else {
            let _ = native::yield_now();
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    support::put(EV_STATE, 8);
    native::exit(3)
}

#[allow(dead_code)]
fn unused(_: Error) {}
