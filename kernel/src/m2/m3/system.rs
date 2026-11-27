//! FR-I01: composition of the existing process, entropy and block mechanisms.
//! This is an explicit integration acceptance manifest, not the full product.
//! Crypto/volume policy remains in the unchanged ring-3 service binaries.
use super::*;
use capability_core::{
    BlockScope, BLOCK_FLUSH, BLOCK_READ, BLOCK_WRITE, PROCESS_EXITED, PROCESS_FAULTED, RECV, SEND,
};

#[cfg(feature = "native-system-lock")]
#[path = "system_lock.rs"]
mod keylock;
#[cfg(all(
    feature = "native-system-lock",
    not(feature = "test-system-s3-lock-busy")
))]
pub(super) use keylock::lock;
// KL2 (ADR-KEYRING-LOCK-20261126): SUSPEND requires LOCKED; kernel/Cargo.toml
// makes every S3 profile carry LOCK, this pins it.
#[cfg(all(feature = "test-system-s3", not(feature = "native-system-lock")))]
compile_error!("every S3 profile requires native-system-lock (KL2)");
/// H05-T5d A7 (labelled injection, TEST only): init's LOCK attempts while
/// exactly this many kernel entropy requests keep the broker busy.
#[cfg(feature = "test-system-s3-lock-busy")]
const LOCK_BUSY_ATTEMPTS: u64 = 8;
/// H05-T5d A7: before the real 0x92 handler runs, one kernel entropy request is
/// in flight (submitted now if the broker is quiet), so LOCK refuses (-8);
/// anything else stops the VM. SUSPEND then refuses (-25) and init falls back
/// to S5 once `lock_busy_poll` delivered every such request.
#[cfg(feature = "test-system-s3-lock-busy")]
pub(super) fn lock(s: &mut State, slot: usize, a: [u64; 6]) -> i64 {
    // Other callers meet the real handler unchanged (POWER authority, -6).
    if slot != INIT {
        return keylock::lock(s, slot, a);
    }
    require(
        suspend_state().busy_locks < LOCK_BUSY_ATTEMPTS,
        b"S3_LOCK_BUSY_ATTEMPTS",
    );
    if keyring::broker_quiet() {
        let ticket = keyring::kernel_request(s).unwrap_or_else(|_| fail(b"S3_LOCK_BUSY_REQUEST"));
        let p = suspend_state();
        require(p.busy_ticket.is_none(), b"S3_LOCK_BUSY_REQUEST");
        p.busy_ticket = Some(ticket);
        p.busy_requests += 1;
    }
    require(!keyring::broker_quiet(), b"S3_LOCK_BUSY_QUIET");
    let code = keylock::lock(s, slot, a);
    require(code == -8 && !keylock::gone(), b"S3_LOCK_BUSY_GRANTED");
    suspend_state().busy_locks += 1;
    code
}

const INIT: usize = 0;
const KEY: usize = 1;
const VOLUME: usize = 2;
const APP: usize = 3;
const WORDS: [u64; 13] = [
    40, 240, 400, 408, 416, 424, 432, 440, 448, 456, 464, 472, 480,
];
/// B02 persistence evidence of the app: persisted record seals, restored
/// unseals, revoked/tampered refusals after restart, administrative opcodes
/// refused on the domain endpoint, committed object revision.
const PERSIST_WORDS: [u64; 6] = [528, 536, 544, 552, 560, 568];
const RECOVER: bool = cfg!(feature = "test-system-recover");
// ADR-B02-KEY1-RELAY-20261123 (test profile only). The relay kernel grants the
// Key Service a SEND on init's key-state commit ingress; it is never combined
// with a profile that has other probes, restarts, power or a read-only volume.
#[cfg(all(
    feature = "test-system-keystate-relay",
    any(
        feature = "native-system-random",
        feature = "native-system-images",
        feature = "native-system-net",
        feature = "native-system-tcp",
        feature = "native-system-power",
        feature = "native-system-nas-probe",
        feature = "native-system-materiaal",
        feature = "test-system-recover"
    )
))]
compile_error!("the key-state relay is a separate test profile on the normal system flow");
#[cfg(any(
    all(
        feature = "test-system-keystate-relay-not-committed",
        any(
            feature = "test-system-keystate-relay-timeout",
            feature = "test-system-keystate-relay-leak"
        )
    ),
    all(
        feature = "test-system-keystate-relay-timeout",
        feature = "test-system-keystate-relay-leak"
    )
))]
compile_error!("key-state relay cases require separate boots");
/// Relay case: 0 off, 1 normal, 2 NOT_COMMITTED, 3 timeout, 4 manifest leak.
const RELAY: u64 = if cfg!(feature = "test-system-keystate-relay-leak") {
    4
} else if cfg!(feature = "test-system-keystate-relay-timeout") {
    3
} else if cfg!(feature = "test-system-keystate-relay-not-committed") {
    2
} else if cfg!(feature = "test-system-keystate-relay") {
    1
} else {
    0
};
/// Init relay words: received, committed, NOT_COMMITTED, withheld, last
/// generation, FNV sum of the last blob, object revision, BEGIN count; then
/// the case word init was given.
#[cfg(feature = "test-system-keystate-relay")]
const RELAY_INIT_WORDS: [u64; 9] = [760, 768, 776, 784, 792, 800, 808, 816, 208];
/// App relay words: |status| of the two fault-probe GENERATEs, the case word.
#[cfg(feature = "test-system-keystate-relay")]
const RELAY_APP_WORDS: [u64; 3] = [736, 744, 752];
/// H05 power (ADR-076): every POWER_OFF refusal per slot, in order, as the
/// kernel returned it; ring-3 evidence words are not trusted for these codes.
#[cfg(feature = "native-system-power")]
struct Power {
    log: [[i64; 8]; 4],
    count: [usize; 4],
    dropped: [usize; 4],
    granted: bool,
}
#[cfg(feature = "native-system-power")]
static mut POWER: Power = Power {
    log: [[0; 8]; 4],
    count: [0; 4],
    dropped: [0; 4],
    granted: false,
};
#[cfg(feature = "native-system-power")]
fn power_state() -> &'static mut Power {
    // SAFETY: One CPU, IF=0. Each borrow ends before a syscall or scheduling.
    unsafe { &mut *ptr::addr_of_mut!(POWER) }
}
/// Fault fixtures start after init's first POWER_OFF call (made before SHUTDOWN).
#[cfg(feature = "native-system-power")]
pub(super) fn power_requested() -> bool {
    power_state().count[INIT] > 0
}
/// Syscall 0x90 POWER_OFF(power handle, 0, 0, 0, 0, 0). 0 = granted: the caller
/// terminates like exit(0) and S5 follows in `finish` after a second gate check.
/// Order: arguments, authority, plan, then the storage gate (-8 retry, -16 never).
#[cfg(feature = "native-system-power")]
pub(super) fn power(s: &State, slot: usize, a: [u64; 6]) -> i64 {
    let code = if a[1..].iter().any(|value| *value != 0) {
        -1
    } else if let Err(error) = m4::engine().check_power(s.processes[slot].pid, a[0]) {
        error.code()
    } else if !crate::m2::power::ready() {
        -14
    } else if let Err(code) = storage_closed() {
        code
    } else {
        0
    };
    // B01-LOCK fixture: init's grant waits (-8, not logged) until it is done.
    #[cfg(feature = "test-system-lock")]
    if keylock::hold(code) {
        return -8;
    }
    let p = power_state();
    if code == 0 {
        p.granted = true;
    } else if slot < p.count.len() {
        // A caller can exhaust the bounded audit prefix, not kernel availability.
        // Preserve the existing evidence checks for the recorded refusals.
        let recorded = p.count[slot];
        if recorded < p.log[slot].len() {
            p.log[slot][recorded] = code;
            p.count[slot] = recorded + 1;
        } else {
            // Preserve availability, but never let an acceptance fixture pass
            // after losing a refusal that its bounded evidence cannot inspect.
            p.dropped[slot] = p.dropped[slot].saturating_add(1);
        }
    }
    code
}
/// No block request in flight, all durable, no block authority left (nobody
/// can write any more) and the entropy broker idle (no virtio-rng DMA).
#[cfg(feature = "native-system-power")]
fn storage_closed() -> Result<(), i64> {
    m5::power_gate()?;
    if m4::engine().stats().block_objects != 0 || !keyring::broker_summary().2 {
        return Err(-8);
    }
    Ok(())
}
/// H05-T4 (ADR-H05-S3-SYSTEM-20261123): every SUSPEND result per call, in
/// order, as the kernel returned it, kept apart from the POWER_OFF log.
#[cfg(feature = "test-system-s3")]
struct Suspend {
    log: [i64; 8],
    count: usize,
    other: usize,
    granted: bool,
    resumed: bool,
    irqs_at_wake: u64,
    ticked: bool,
    /// Init's entropy/block syscalls after the wake: (number, result).
    lost: [(u64, i64); 4],
    lost_count: usize,
    /// A7: LOCK refusals, kernel requests submitted and delivered, the one
    /// outstanding ticket, and the drained signal (word 640) given to init.
    #[cfg(feature = "test-system-s3-lock-busy")]
    busy_locks: u64,
    #[cfg(feature = "test-system-s3-lock-busy")]
    busy_requests: u64,
    #[cfg(feature = "test-system-s3-lock-busy")]
    busy_delivered: u64,
    #[cfg(feature = "test-system-s3-lock-busy")]
    busy_ticket: Option<u64>,
    #[cfg(feature = "test-system-s3-lock-busy")]
    busy_drained: bool,
    /// Broker (delivered bytes, denied) at the wake, before any post-wake
    /// request, so post-wake activity cannot pad the pre-sleep evidence.
    #[cfg(feature = "test-system-s3-reinit")]
    broker_at_wake: (u64, u64),
    /// Device accesses of both brokers (`device_io`) at the wake (review R3).
    io_at_wake: (u64, u64, u64),
    /// SUSPEND refusals (-8) because the volume broker still held a request.
    held_refusals: u64,
    /// Teardown resets the post-wake latch kept away from a device.
    suppressed: u64,
}
#[cfg(feature = "test-system-s3")]
static mut SUSPEND: Suspend = Suspend {
    log: [0; 8],
    count: 0,
    other: 0,
    granted: false,
    resumed: false,
    irqs_at_wake: 0,
    ticked: false,
    lost: [(0, 0); 4],
    lost_count: 0,
    #[cfg(feature = "test-system-s3-lock-busy")]
    busy_locks: 0,
    #[cfg(feature = "test-system-s3-lock-busy")]
    busy_requests: 0,
    #[cfg(feature = "test-system-s3-lock-busy")]
    busy_delivered: 0,
    #[cfg(feature = "test-system-s3-lock-busy")]
    busy_ticket: None,
    #[cfg(feature = "test-system-s3-lock-busy")]
    busy_drained: false,
    #[cfg(feature = "test-system-s3-reinit")]
    broker_at_wake: (0, 0),
    io_at_wake: (0, 0, 0),
    held_refusals: 0,
    suppressed: 0,
};
#[cfg(feature = "test-system-s3")]
fn suspend_state() -> &'static mut Suspend {
    // SAFETY: One CPU, IF=0, KERNEL held. Each borrow ends before a syscall,
    // scheduling or the sleep itself.
    unsafe { &mut *ptr::addr_of_mut!(SUSPEND) }
}
/// Every device access of both brokers (review R3): entropy requests reaching
/// the driver, block descriptor preparations and publications, polls, resets,
/// reset acknowledgement reads and re-initialisations, each counted at its
/// call site. (keyring, volume, accounted keyring re-init subset; the last
/// is 0 outside test-system-s3-reinit).
#[cfg(feature = "test-system-s3")]
fn device_io() -> (u64, u64, u64) {
    let (keyring, accounted) = keyring::device_accesses();
    // H05-T5c: plus the volume re-init subset (0 outside test-system-s3-blk-reinit).
    (
        keyring,
        m5::device_accesses(),
        accounted + m5::reinit_accounted(),
    )
}
/// H05-T5c labelled injections: each must end in the lane-0 quarantine.
#[cfg(feature = "test-system-s3-blk-reinit")]
const BLK_QUARANTINE_EXPECTED: bool = cfg!(any(
    feature = "test-system-s3-blk-reinit-busy",
    feature = "test-system-s3-blk-reinit-identity",
    feature = "test-system-s3-blk-reinit-capacity",
    feature = "test-system-s3-blk-reinit-no-reset-ack"
));
#[cfg(feature = "test-system-s3")]
pub(super) fn device_access_suppressed() {
    let p = suspend_state();
    p.suppressed = p.suppressed.saturating_add(1);
}
/// test-system-s3-held-completion only: once the quiescence check has refused
/// (-8), skip it on every later SUSPEND until the grant (a labelled gate-defect
/// injection) so that the post-wake latch alone must keep the held request's
/// teardown reset away from the device.
#[cfg(feature = "test-system-s3-held-completion")]
fn held_check_skipped() -> bool {
    let p = suspend_state();
    p.held_refusals > 0 && !p.granted
}
/// Every other S3 profile: the quiescence check is never skipped.
#[cfg(all(
    feature = "test-system-s3",
    not(feature = "test-system-s3-held-completion")
))]
fn held_check_skipped() -> bool {
    false
}
/// Syscall 0x91 SUSPEND(power handle, 0, 0, 0, 0, 0), same validation order
/// as POWER_OFF, then volume broker quiescence (-8: it still holds a request,
/// e.g. a completed one its owner never consumed, whose teardown would reset
/// the device after the wake; review R1). One grant per boot (-14
/// afterwards). 0 = validated wake. Every refusal precedes any I/O.
/// KL2 (H05-T5d): -25 unless LOCKED and erased (F1), before the storage gate.
#[cfg(feature = "test-system-s3")]
pub(super) fn suspend(s: &mut State, slot: usize, a: [u64; 6]) -> i64 {
    let code = if a[1..].iter().any(|value| *value != 0) {
        -1
    } else if let Err(error) = m4::engine().check_power(s.processes[slot].pid, a[0]) {
        error.code()
    } else if !crate::m2::sleep::ready() {
        -14
    } else if {
        // A 0x92 in this boot reaches here only in a later kernel entry, so
        // the exit-path scrub has run (m3_restore, before the CPL3 iretq).
        keylock::latch();
        !(keylock::locked() && keylock::erased())
    } {
        -25
    } else if let Err(code) = storage_closed() {
        code
    } else if !m5::sleep_quiescent() && !held_check_skipped() {
        let p = suspend_state();
        p.held_refusals = p.held_refusals.saturating_add(1);
        -8
    } else {
        0
    };
    let p = suspend_state();
    if slot != INIT {
        p.other = p.other.saturating_add(1);
    } else if p.count < p.log.len() {
        p.log[p.count] = code;
        p.count += 1;
    } else {
        // A lost record can never satisfy the exact log `finish` requires.
        p.other = p.other.saturating_add(1);
    }
    if code != 0 {
        return code;
    }
    p.granted = true;
    crate::m2::sleep::suspend(&mut s.kernel);
    // Validated wake. Latch first: from here no broker path touches a device
    // (test-system-s3-reinit: until the validated re-init, which is accounted).
    let io = device_io();
    let p = suspend_state();
    p.resumed = true;
    p.io_at_wake = io;
    p.irqs_at_wake = s.timer_interrupts;
    #[cfg(feature = "test-system-s3-reinit")]
    {
        let (delivered, denied, _) = keyring::broker_summary();
        p.broker_at_wake = (delivered, denied);
    }
    0
}
/// After a validated wake every device is reset by the platform. Submission
/// paths (keyring 0x60, volume 0x40-0x43) refuse with -14; broker progress
/// (`advance`) and owner teardown (keyring `quarantine_pending`, volume
/// `reset`) skip every device MMIO and PCI config access (review R1). Only
/// test-system-s3-reinit lifts the keyring latch, after its validated re-init.
#[cfg(feature = "test-system-s3")]
pub(super) fn resumed() -> bool {
    suspend_state().resumed
}
/// Every entropy (0x60) and block (0x40..=0x43) result after the wake, as
/// returned. Another caller or an overflow counts in `other`: fail closed.
#[cfg(feature = "test-system-s3")]
pub(super) fn after_wake(slot: usize, number: u64, code: i64) {
    let p = suspend_state();
    if !p.resumed {
        return;
    }
    // An init probe before the proven post-wake tick is unexpected (p.other).
    if slot == INIT && p.ticked && p.lost_count < p.lost.len() {
        p.lost[p.lost_count] = (number, code);
        p.lost_count += 1;
    } else {
        p.other = p.other.saturating_add(1);
    }
}
/// First timer interrupt after the wake: tell init it may continue. With
/// test-system-s3-reinit only once virtio-rng is re-initialised (H05-T5b).
#[cfg(feature = "test-system-s3")]
pub(super) fn tick_after_wake(s: &State) {
    #[cfg(feature = "test-system-s3-reinit")]
    if !keyring::reinit_done() {
        return;
    }
    // H05-T5c: also the volume re-init, validated or quarantined (owner Q4).
    #[cfg(feature = "test-system-s3-blk-reinit")]
    if !m5::blk_reinit_settled() {
        return;
    }
    let p = suspend_state();
    if p.resumed && !p.ticked && s.processes[INIT].status != Status::Dead {
        p.ticked = true;
        put(s, INIT, 616, 1);
    }
}
/// H05-T5d A7: deliver the kernel's own requests as they complete; once init's
/// post-LOCK SUSPEND was refused (-25) and none is left, word 640 lets init
/// continue to S5, so its POWER_OFF meets a closed storage gate at once.
#[cfg(feature = "test-system-s3-lock-busy")]
fn lock_busy_poll(s: &State) {
    if let Some(ticket) = suspend_state().busy_ticket {
        match keyring::kernel_deliver(s, ticket) {
            Ok(32) => {
                let p = suspend_state();
                p.busy_ticket = None;
                p.busy_delivered += 1;
            }
            // Still pending: the broker's own deadline (-13) bounds the wait.
            Err(-8) => return,
            _ => fail(b"S3_LOCK_BUSY_DELIVERY"),
        }
    }
    let p = suspend_state();
    if !p.busy_drained && p.count == 5 && p.log[4] == -25 && keyring::broker_quiet() {
        p.busy_drained = true;
        put(s, INIT, 640, 1);
    }
}
struct Evidence {
    app: Option<[u64; 13]>,
    persist: Option<[u64; 6]>,
    #[cfg(feature = "native-system-clock")]
    clock: Option<[u64; 4]>,
    init: Option<[u64; 13]>,
    faults: u64,
    kernel_tables: usize,
    #[cfg(feature = "native-system-net")]
    net: Option<[u64; 13]>,
    #[cfg(feature = "native-system-net")]
    net_pid: u64,
    #[cfg(feature = "native-system-tcp")]
    tcp: Option<([u64; 2], u64)>, // clock reads, blackhole ticks; actual exit tick
    #[cfg(feature = "test-system-net-link")]
    link: Option<[u64; 13]>,
    #[cfg(feature = "test-system-keystate-relay")]
    relay_init: Option<[u64; 9]>,
    #[cfg(feature = "test-system-keystate-relay")]
    relay_app: Option<[u64; 3]>,
}
static mut EVIDENCE: Evidence = Evidence {
    app: None,
    persist: None,
    #[cfg(feature = "native-system-clock")]
    clock: None,
    init: None,
    faults: 0,
    kernel_tables: 0,
    #[cfg(feature = "native-system-net")]
    net: None,
    #[cfg(feature = "native-system-net")]
    net_pid: 0,
    #[cfg(feature = "native-system-tcp")]
    tcp: None,
    #[cfg(feature = "test-system-net-link")]
    link: None,
    #[cfg(feature = "test-system-keystate-relay")]
    relay_init: None,
    #[cfg(feature = "test-system-keystate-relay")]
    relay_app: None,
};
fn evidence() -> &'static mut Evidence {
    // SAFETY: KERNEL held, IF=0. Each borrow ends before a syscall or scheduling.
    unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) }
}
fn fail(reason: &[u8]) -> ! {
    serial::write(b"SYSTEM:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}
fn require(ok: bool, reason: &[u8]) {
    if !ok {
        fail(reason);
    }
}
fn put(s: &State, slot: usize, offset: u64, value: u64) {
    write_word(s.processes[slot].physical(DATA + offset), value);
}
fn word(s: &State, slot: usize, offset: u64) -> u64 {
    #[cfg(feature = "native-system-lock")]
    if slot == KEY && keylock::gone() {
        return keylock::key_word(offset);
    }
    read_word(s.processes[slot].physical(DATA + offset))
}
#[cfg(not(feature = "native-system-materiaal"))]
pub(super) fn image_elf(image: u32) -> Option<&'static [u8]> {
    if image == 0 {
        Some(SYSTEM_WORKER_ELF)
    } else {
        None
    }
}
#[cfg(feature = "native-system-materiaal")]
pub(super) fn image_elf(_: u32) -> Option<&'static [u8]> {
    None
}
fn channel(e: &mut capability_core::Engine, owner: u64, sender: u64) -> (u64, u64) {
    let root = e.create_endpoint(owner).unwrap();
    let receive = e.derive(owner, root, RECV).unwrap();
    let send = e.bootstrap_delegate(owner, root, sender, SEND).unwrap();
    e.close(owner, root).unwrap();
    (receive, send)
}
pub(super) fn manifest(s: &mut State) {
    let cold = match option_env!("NOS_SYSTEM_COLD_CASE") {
        None => 0,
        Some("keystate") => 1,
        Some("record-tag") => 2,
        Some("swapped-blocks") => 3,
        Some("withheld-secret") => 4,
        Some(_) => fail(b"UNKNOWN_COLD_CASE"),
    };
    require(cold == 0 || RECOVER, b"COLD_REQUIRES_RECOVER");
    put(s, INIT, 576, cold);
    put(s, APP, 576, cold);
    let initial = if cfg!(any(
        feature = "native-system-net",
        feature = "native-system-random",
        feature = "native-system-images"
    )) {
        5
    } else {
        4
    };
    require(
        s.amount == MAX
            && s.processes[..initial]
                .iter()
                .all(|p| p.status != Status::Dead)
            && s.processes[initial..]
                .iter()
                .all(|p| p.status == Status::Dead),
        b"MANIFEST_STATE",
    );
    let ids = [
        s.processes[INIT].pid,
        s.processes[KEY].pid,
        s.processes[VOLUME].pid,
        s.processes[APP].pid,
    ];
    let [init, key, volume, app] = ids;
    let e = m4::engine();
    for pid in ids {
        e.register(pid).unwrap();
    }
    #[cfg(feature = "native-system-net")]
    {
        let pid = s.processes[4].pid;
        evidence().net_pid = pid;
        e.register(pid).unwrap();
        let cap = e.create_network(pid).unwrap();
        let key = e.check_network(pid, cap).unwrap();
        let private = e.create_endpoint(pid).unwrap();
        let wrong_type = e.derive(pid, private, RECV).unwrap();
        e.close(pid, private).unwrap();
        for (offset, value) in [(32, 1), (40, 0), (48, cap), (56, wrong_type), (176, pid)] {
            put(s, 4, offset, value);
        }
        #[cfg(feature = "test-system-net-link")]
        put(s, 4, network::link::MODE_WORD, network::link::MODE);
        network::bind(pid, cap, key);
    }
    #[cfg(feature = "native-system-random")]
    {
        let probe = s.processes[4].pid;
        e.register(probe).unwrap();
        let root = e.create_endpoint(key).unwrap();
        let receive = e.derive(key, root, RECV).unwrap();
        let ancestor = e
            .derive(
                key,
                root,
                SEND | capability_core::TRANSFER | capability_core::REVOKE,
            )
            .unwrap();
        let leaf = e.bootstrap_delegate(key, ancestor, probe, SEND).unwrap();
        let (reply, reply_send) = channel(e, probe, key);
        let case = if cfg!(feature = "test-system-random-sdk") {
            1
        } else if cfg!(feature = "test-system-random-unknown") {
            2
        } else if cfg!(feature = "test-system-random-wrong-caller") {
            3
        } else {
            0
        };
        for (offset, value) in [
            (32, 1),
            (48, leaf),
            (56, reply),
            (112, case),
            (160, key),
            (176, probe),
        ] {
            put(s, 4, offset, value);
        }
        for (offset, value) in [
            (120, u64::from_le_bytes(*b"NOSRNG01")),
            (128, receive),
            (136, reply_send),
            (144, if case == 3 { app } else { probe }),
            (152, 0),
        ] {
            put(s, KEY, offset, value);
        }
        random_fixture::bind(
            s,
            e,
            random_fixture::Binding {
                case,
                key_pid: key,
                probe_pid: probe,
                random_root: receive,
                send_ancestor: ancestor,
                send_leaf: leaf,
                reply_receive: reply,
            },
        );
        // Root authority is no longer needed. Its live receive/REVOKE/SEND
        // leaves keep the object alive and provide the exact test subtree.
        e.close(key, root).unwrap();
    }
    #[cfg(feature = "native-system-images")]
    {
        let probe = s.processes[4].pid;
        e.register(probe).unwrap();
        put(s, 4, 176, probe);
        image_fixture::bind(s, probe, IMAGE_PROBE_ELF);
        print_number(b"SYSTEM:IMAGE_MANIFEST_PID:", probe);
    }
    #[cfg(feature = "native-system-power")]
    {
        let power = e.create_power(init).unwrap();
        // Non-delegable by type: no derive, delegation, spawn grant or transfer.
        let invalid = capability_core::Error::Invalid;
        require(
            e.derive(init, power, capability_core::POWER) == Err(invalid)
                && e.bootstrap_delegate(init, power, app, capability_core::POWER) == Err(invalid)
                && e.check_delegation(init, power, capability_core::POWER) == Err(invalid)
                && e.create_power(init).is_err(),
            b"POWER_DELEGATION",
        );
        // The app learns init's raw handle value only: a number, not authority.
        put(s, INIT, 48, power);
        // 1 = SUSPEND sequence; 2 = it also expects one quiescence -8 first;
        // 3 = A7: LOCK stays -8, SUSPEND -25, then S5 after word 640.
        #[cfg(feature = "test-system-s3")]
        put(
            s,
            INIT,
            592,
            if cfg!(feature = "test-system-s3-held-completion") {
                2
            } else if cfg!(feature = "test-system-s3-lock-busy") {
                3
            } else {
                1
            },
        );
        // H05-T5b: entropy is served again after the wake; init's POWER handle
        // then reaches the authority check (-6) instead of the -14 refusal.
        #[cfg(feature = "test-system-s3-reinit")]
        put(s, INIT, 624, 1);
        // H05-T5c: the expected post-wake block refusal magnitude: authority
        // (-6) after the validated re-init, -24 once lane 0 is quarantined.
        #[cfg(feature = "test-system-s3-blk-reinit")]
        put(
            s,
            INIT,
            632,
            if BLK_QUARANTINE_EXPECTED {
                m5::QUARANTINED.unsigned_abs()
            } else {
                6
            },
        );
        put(s, APP, 104, power);
        // SHUTDOWN is accepted only on domain 1, init's volume ingress.
        put(s, VOLUME, 104, 2);
    }
    let launch = e.create_launch(init, 0).unwrap();
    // Init owns the registry so it can delegate only SEND to each worker.
    let registry = e.create_endpoint(init).unwrap();
    let registry_receive = e.derive(init, registry, RECV).unwrap();
    let app_registry = e.bootstrap_delegate(init, registry, app, SEND).unwrap();
    let (app_control, init_control) = channel(e, app, init);
    let (key_reply, key_send_reply) = channel(e, app, key);
    let (key_ingress, app_key) = channel(e, key, app);
    let key_other_root = e.create_endpoint(key).unwrap();
    let key_other = e.derive(key, key_other_root, RECV).unwrap();
    e.close(key, key_other_root).unwrap();
    // B02: the persistence-only administrative ingress of the Key Service is
    // granted to init alone; the app never holds a SEND to it.
    let (key_admin_ingress, init_key_admin) = channel(e, key, init);
    let (init_key_reply, key_admin_reply) = channel(e, init, key);
    // ADR-B02-KEY1-RELAY: init alone receives the key-state commit ingress and
    // the Key Service holds its only SEND; replies flow back on a second pair.
    // The labelled leak case gives the app one extra SEND (manifest red case).
    #[cfg(feature = "test-system-keystate-relay")]
    let (init_commit, key_commit, key_commit_reply, init_commit_reply, leaked) = {
        let root = e.create_endpoint(init).unwrap();
        let receive = e.derive(init, root, RECV).unwrap();
        let send = e.bootstrap_delegate(init, root, key, SEND).unwrap();
        #[cfg(feature = "test-system-keystate-relay-leak")]
        let leaked = Some(e.bootstrap_delegate(init, root, app, SEND).unwrap());
        #[cfg(not(feature = "test-system-keystate-relay-leak"))]
        let leaked: Option<u64> = None;
        e.close(init, root).unwrap();
        let (reply_receive, reply_send) = channel(e, key, init);
        (receive, send, reply_receive, reply_send, leaked)
    };
    let entropy = e.create_entropy(key).unwrap();
    let (volume_reply, volume_send_reply) = channel(e, app, volume);
    let (volume_ingress, app_volume) = channel(e, volume, app);
    // The second volume domain belongs to init (key-state snapshot), never the app.
    let (volume_other, init_volume) = channel(e, volume, init);
    let (init_volume_reply, volume_reply_other) = channel(e, init, volume);
    let readonly = cfg!(feature = "test-system-recover");
    let block = e
        .create_block_device_with_rights(
            volume,
            BlockScope {
                device: 0,
                first_sector: 4,
                sector_count: 580,
            },
            if readonly {
                BLOCK_READ
            } else {
                BLOCK_READ | BLOCK_WRITE | BLOCK_FLUSH
            },
        )
        .unwrap();
    for (offset, value) in [
        (32, 1),
        (40, 0),
        (48, entropy),
        (56, key_ingress),
        (80, key_other),
        (88, key_send_reply),
        (96, key_admin_ingress),
        (104, key_admin_reply),
        (160, app),
        (168, init),
        (176, key),
    ] {
        put(s, KEY, offset, value);
    }
    #[cfg(feature = "test-system-keystate-relay")]
    {
        let case = if RELAY == 4 { 1 } else { RELAY };
        for (offset, value) in [(496, key_commit), (504, key_commit_reply), (512, init)] {
            put(s, KEY, offset, value);
        }
        for (offset, value) in [(192, init_commit), (200, init_commit_reply), (208, case)] {
            put(s, INIT, offset, value);
        }
        put(s, APP, 752, case);
    }
    for (offset, value) in [
        (32, 1),
        (40, 0),
        (48, block),
        (56, volume_ingress),
        (80, volume_other),
        (88, volume_send_reply),
        (72, volume_reply_other),
        (120, u64::from(readonly)),
        (160, app),
        (168, init),
        (176, volume),
    ] {
        put(s, VOLUME, offset, value);
    }
    for (offset, value) in [
        (32, 1),
        (40, 0),
        (56, launch),
        (64, registry_receive),
        (72, registry),
        (80, init_control),
        (88, init_volume),
        (96, init_volume_reply),
        (104, init_key_admin),
        (112, init_key_reply),
        (120, u64::from(readonly)),
        (160, app),
        (168, key),
        (176, init),
        (184, volume),
    ] {
        put(s, INIT, offset, value);
    }
    for (offset, value) in [
        (32, 1),
        (40, 0),
        (56, key_reply),
        (64, app_key),
        (72, volume_reply),
        (80, app_volume),
        (88, app_registry),
        (96, app_control),
        (112, u64::from(readonly)),
        (160, key),
        (168, volume),
        (176, app),
        (184, init),
    ] {
        put(s, APP, offset, value);
    }
    // B01-LOCK A6: the app keeps its KEY1 session open until after LOCK.
    #[cfg(feature = "test-system-lock")]
    put(s, APP, keylock::APP_PROBE, 1);
    // The app's complete handle table at manifest time. Every app check below
    // iterates this one list; APP_HANDLE_COUNT proves that nothing is outside it.
    #[cfg(not(feature = "test-system-keystate-relay-leak"))]
    let app_handles = [
        key_reply,
        app_key,
        volume_reply,
        app_volume,
        app_registry,
        app_control,
    ];
    // Red case: the leaked commit SEND is listed, so only APP_COMMIT_HANDLE
    // (not APP_HANDLE_COUNT) can be the check that stops this boot.
    #[cfg(feature = "test-system-keystate-relay-leak")]
    let app_handles = [
        key_reply,
        app_key,
        volume_reply,
        app_volume,
        app_registry,
        app_control,
        leaked.unwrap(),
    ];
    for handle in app_handles {
        require(
            e.check_entropy(app, handle).is_err()
                && e.check_block(app, handle, BLOCK_READ).is_err()
                && e.check_launch(app, handle).is_err()
                && e.check_network(app, handle).is_err()
                && e.check_power(app, handle).is_err(),
            b"APP_AUTHORITY",
        );
    }
    // Init's recovery-agent endpoints are plain endpoints: no entropy, block,
    // launch or network authority is hidden behind the administrative ingress.
    for handle in [
        init_key_admin,
        init_key_reply,
        init_volume,
        init_volume_reply,
    ] {
        require(
            e.check_entropy(init, handle).is_err()
                && e.check_block(init, handle, BLOCK_READ).is_err()
                && e.check_launch(init, handle).is_err()
                && e.check_network(init, handle).is_err()
                && e.check_power(init, handle).is_err(),
            b"INIT_AUTHORITY",
        );
    }
    #[cfg(feature = "test-system-keystate-relay")]
    for handle in [init_commit, init_commit_reply] {
        require(
            e.check_entropy(init, handle).is_err()
                && e.check_block(init, handle, BLOCK_READ).is_err()
                && e.check_launch(init, handle).is_err()
                && e.check_network(init, handle).is_err()
                && e.check_power(init, handle).is_err(),
            b"INIT_AUTHORITY",
        );
    }
    // The app cannot reach the administrative ingress or its reply channel by
    // any handle it owns: handle numbers are per process, so compare the
    // endpoint objects behind every app handle with init's admin endpoints.
    let admin_ingress_object = e.endpoint_object(init, init_key_admin).unwrap();
    let admin_reply_object = e.endpoint_object(init, init_key_reply).unwrap();
    for handle in app_handles {
        let object = e.endpoint_object(app, handle).unwrap();
        require(
            object != admin_ingress_object && object != admin_reply_object,
            b"APP_ADMIN_HANDLE",
        );
    }
    #[cfg(feature = "test-system-keystate-relay")]
    {
        let _ = leaked;
        // Same equation as APP_ADMIN_HANDLE for the commit ingress C and its
        // reply endpoint R: no app or volume handle names either object, the
        // Key Service holds exactly one handle on each, and both tables are
        // exhaustive (NULL slot plus the listed handles, see APP_HANDLE_COUNT).
        let commit = e.endpoint_object(init, init_commit).unwrap();
        let commit_reply = e.endpoint_object(init, init_commit_reply).unwrap();
        let on_relay = |pid: u64, handle: u64| matches!(e.endpoint_object(pid, handle).ok(), Some(o) if o == commit || o == commit_reply);
        for handle in app_handles {
            require(!on_relay(app, handle), b"APP_COMMIT_HANDLE");
        }
        let volume_handles = [
            block,
            volume_ingress,
            volume_other,
            volume_send_reply,
            volume_reply_other,
        ];
        for handle in volume_handles {
            require(!on_relay(volume, handle), b"VOLUME_COMMIT_HANDLE");
        }
        let exhaustive = |pid: u64, handles: &[u64]| {
            handles
                .iter()
                .enumerate()
                .all(|(i, handle)| !handles[..i].contains(handle))
                && e.available_slots(pid) == capability_core::MAX_SLOTS - 1 - handles.len()
        };
        require(exhaustive(volume, &volume_handles), b"VOLUME_HANDLE_COUNT");
        let key_handles = [
            entropy,
            key_ingress,
            key_other,
            key_send_reply,
            key_admin_ingress,
            key_admin_reply,
            key_commit,
            key_commit_reply,
        ];
        let named = |object: usize| {
            key_handles
                .iter()
                .filter(|handle| e.endpoint_object(key, **handle).ok() == Some(object))
                .count()
        };
        require(
            named(commit) == 1
                && named(commit_reply) == 1
                && e.endpoint_object(key, key_commit).ok() == Some(commit)
                && e.endpoint_object(key, key_commit_reply).ok() == Some(commit_reply),
            b"KEY_COMMIT_HANDLE",
        );
        require(exhaustive(key, &key_handles), b"KEY_HANDLE_COUNT");
        // SEND-only/RECV-only and non-delegable on both sides (no TRANSFER).
        let only = |pid: u64, handle: u64, right: u64, other: u64| {
            e.check_endpoint(pid, handle, right).is_ok()
                && e.check_endpoint(pid, handle, other).is_err()
                && e.check_delegation(pid, handle, right).is_err()
        };
        require(
            only(key, key_commit, SEND, RECV) && only(key, key_commit_reply, RECV, SEND),
            b"KEY_COMMIT_RIGHTS",
        );
        require(
            only(init, init_commit, RECV, SEND) && only(init, init_commit_reply, SEND, RECV),
            b"INIT_COMMIT_RIGHTS",
        );
    }
    // Exhaustiveness: the loops above only see listed handles, so prove that
    // the app holds no other handle (for example a seventh one to the admin
    // ingress). `Engine::available_slots` counts the free slots in
    // 1..MAX_SLOTS. The terms of the expected value:
    // - NULL_SLOT: slot 0 is the reserved null handle, never counted and
    //   never holding a capability;
    // - app_handles.len(): each listed handle resolved above (unwrap); a
    //   handle names one slot by its index bits and only the slot's current
    //   generation resolves, so pairwise distinct live handles occupy
    //   pairwise distinct slots.
    // Equality therefore means every occupied app slot is a listed, checked
    // handle. A retired or stale-occupied slot only lowers the count and fails
    // closed. The same N holds in every profile that runs this manifest
    // (native-system, test-system-recover, -clock, -net, -tcp, -random*,
    // -images, -display-input, native-cap-digest-bench): their optional
    // fixtures grant handles to their probe in slot 4 or to the Key Service,
    // never to the app. A profile that grants the app more must extend
    // `app_handles`, or this check fails.
    const NULL_SLOT: usize = 1;
    let distinct = app_handles
        .iter()
        .enumerate()
        .all(|(i, handle)| !app_handles[..i].contains(handle));
    require(
        distinct
            && e.available_slots(app) == capability_core::MAX_SLOTS - NULL_SLOT - app_handles.len(),
        b"APP_HANDLE_COUNT",
    );
    require(e.check_invariants(), b"MANIFEST_ENGINE");
    let releasable: usize = s.processes[..initial]
        .iter()
        .map(|p| p.count + p.kernel_frames.len() + p.paging.as_ref().unwrap().owned_table_count())
        .sum();
    s.free_baseline = s.free_count().checked_add(releasable).unwrap();
    evidence().kernel_tables = s.kernel.owned_table_count();
    serial::write(b"SYSTEM:MANIFEST:PASS\n");
}
fn is_net_probe(s: &State, slot: usize) -> bool {
    #[cfg(feature = "native-system-net")]
    return s.processes[slot].pid == evidence().net_pid;
    #[cfg(not(feature = "native-system-net"))]
    {
        let _ = (s, slot);
        false
    }
}
pub(super) fn observe_exit(s: &mut State, slot: usize, _counter: u64) {
    #[cfg(feature = "native-system-random")]
    if random_fixture::observe_exit(s, slot) {
        return;
    }
    #[cfg(feature = "native-system-images")]
    if image_fixture::is_probe(s, slot) {
        image_fixture::observe_exit(s, slot);
        m4::engine().process_died(s.processes[slot].pid, PROCESS_EXITED, 0);
        return;
    }
    require(
        slot == INIT || slot == APP || is_net_probe(s, slot),
        b"UNEXPECTED_EXIT",
    );
    let cold = word(s, slot, 576);
    if RECOVER && cold != 0 {
        require((1..=4).contains(&cold), b"COLD_CASE");
        let expected_slot = if cold == 1 || cold == 4 { INIT } else { APP };
        require(
            slot == expected_slot && s.last_event.code == 0 && word(s, slot, 584) == 1,
            b"COLD_REFUSAL",
        );
        let (published, completed, idle) = m5::broker_summary();
        require(
            idle && published == completed && published[0] > 0 && published[1..] == [0, 0],
            b"COLD_DISK",
        );
        require(
            word(s, KEY, 224) == 0
                && word(s, KEY, 232) == 0
                && word(s, KEY, 240) == 1
                && word(s, KEY, 400) == u64::from(expected_slot == APP),
            b"COLD_CRYPTO",
        );
        serial::write(match cold {
            1 => b"SYSTEM:COLD:REFUSED:keystate\n",
            2 => b"SYSTEM:COLD:REFUSED:record-tag\n",
            3 => b"SYSTEM:COLD:REFUSED:swapped-blocks\n",
            4 => b"SYSTEM:COLD:REFUSED:withheld-secret\n",
            _ => unreachable!(),
        });
        print_number(b"SYSTEM:COLD_PLAINTEXT_REPLIES:", 0);
        print_number(b"SYSTEM:DISK_READS:", published[0]);
        print_number(b"SYSTEM:DISK_WRITES:", published[1]);
        print_number(b"SYSTEM:DISK_FLUSHES:", published[2]);
        serial::exit(0x10)
    }
    let mut words = [0; 13];
    for (i, at) in WORDS.iter().enumerate() {
        words[i] = word(s, slot, *at);
    }
    if s.last_event.code != 0 || words[0] != 1 {
        print_number(b"SYSTEM:ERROR:SLOT:", slot as u64);
        print_number(b"SYSTEM:ERROR:STEP:", words[1]);
        fail(b"APP_OR_INIT_RESULT");
    }
    if slot == INIT {
        evidence().init = Some(words);
        #[cfg(feature = "test-system-keystate-relay")]
        {
            evidence().relay_init = Some(RELAY_INIT_WORDS.map(|at| word(s, slot, at)));
        }
    } else if slot == APP {
        evidence().app = Some(words);
        #[cfg(feature = "test-system-lock")]
        keylock::app_exit(s);
        let mut persist = [0; 6];
        for (i, at) in PERSIST_WORDS.iter().enumerate() {
            persist[i] = word(s, slot, *at);
        }
        evidence().persist = Some(persist);
        #[cfg(feature = "test-system-keystate-relay")]
        {
            evidence().relay_app = Some(RELAY_APP_WORDS.map(|at| word(s, slot, at)));
        }
        #[cfg(feature = "native-system-clock")]
        {
            let first = word(s, slot, 488);
            let second = word(s, slot, 496);
            let quality = word(s, slot, 504);
            let monotonic_before = word(s, slot, 512);
            let monotonic_after = word(s, slot, 520);
            require(
                s.system_clock_calls == 2
                    && first > 0
                    && second >= first
                    && quality == 1
                    && monotonic_after >= monotonic_before,
                b"CLOCK_EVIDENCE",
            );
            evidence().clock = Some([first, second, monotonic_before, monotonic_after]);
        }
    }
    #[cfg(feature = "native-system-net")]
    if is_net_probe(s, slot) {
        evidence().net = Some(words);
        #[cfg(feature = "test-system-net-link")]
        {
            let mut link = [0; 13];
            for (value, at) in link.iter_mut().zip(network::link::PROBE_WORDS) {
                *value = word(s, slot, at);
            }
            evidence().link = Some(link);
        }
        #[cfg(feature = "native-system-tcp")]
        {
            // Snapshot before process cleanup; these are observer inputs, not
            // kernel-supplied client results. The exit tick comes from IRQ0.
            evidence().tcp = Some(([word(s, slot, 488), word(s, slot, 496)], s.ticks));
        }
    }

    m4::engine().process_died(s.processes[slot].pid, PROCESS_EXITED, 0);
}
pub(super) fn observe_fault(s: &mut State, slot: usize, context: &[u64; 22], _counter: u64) {
    #[cfg(feature = "native-system-images")]
    if image_fixture::is_probe(s, slot) {
        image_fixture::fault(s, slot, context);
    }
    #[cfg(feature = "native-system-random")]
    if random_fixture::is_probe(s, slot) {
        fail(b"RANDOM_PROBE_FAULT");
    }
    require(
        supervisor::is_managed_image(slot, 0)
            && !is_net_probe(s, slot)
            && context[15] == 6
            && evidence().faults == 0,
        b"UNEXPECTED_FAULT",
    );
    evidence().faults += 1;
    require(
        m4::engine().process_died(s.processes[slot].pid, PROCESS_FAULTED, 6),
        b"FAULT_PROCESS",
    );
}
pub(super) fn poll(s: &mut State) -> Option<*const u64> {
    #[cfg(feature = "native-system-random")]
    random_fixture::poll(s);
    #[cfg(feature = "test-system-s3-reinit")]
    if resumed() {
        keyring::reinit_poll(s);
    }
    #[cfg(feature = "test-system-s3-lock-busy")]
    lock_busy_poll(s);
    // H05-T5c: lane 0 after the virtio-rng re-init, until settled.
    #[cfg(feature = "test-system-s3-blk-reinit")]
    if resumed() && keyring::reinit_done() && !m5::blk_reinit_settled() {
        m5::blk_reinit_poll(s);
    }
    keyring::broker_tick(s);
    m5::broker_tick(s);
    #[cfg(feature = "native-system-lock")]
    keylock::latch();
    #[cfg(feature = "test-system-lock")]
    keylock::fixture(s);
    #[cfg(feature = "native-system-net")]
    network::broker_tick(s);
    supervisor::reap_orphans(s);
    require(!s.unexpected, b"UNEXPECTED_EVENT");
    if s.ticks >= 20_000 {
        for slot in 0..if cfg!(any(
            feature = "native-system-net",
            feature = "native-system-random",
            feature = "native-system-images"
        )) {
            5
        } else {
            4
        } {
            if s.processes[slot].status != Status::Dead {
                print_number(b"SYSTEM:TIMEOUT:SLOT:", slot as u64);
                print_number(b"SYSTEM:TIMEOUT:STEP:", word(s, slot, 240));
            }
        }
        fail(b"TIMEOUT");
    }
    if evidence().app.is_none() || evidence().init.is_none() {
        return None;
    }
    #[cfg(feature = "native-system-net")]
    if evidence().net.is_none() || !network::finished() {
        return None;
    }
    #[cfg(feature = "native-system-random")]
    if !random_fixture::finished() {
        return None;
    }
    #[cfg(feature = "native-system-images")]
    if !image_fixture::finished() {
        return None;
    }
    finish(s)
}
fn finish(s: &mut State) -> ! {
    let app = evidence().app.unwrap();
    let init = evidence().init.unwrap();
    let (spawns, kills, statuses) = supervisor::mechanism_counts();
    require(
        evidence().faults == 1 && spawns == 2 && kills == 1 && statuses >= 3,
        b"PROCESS_LIFECYCLE",
    );
    require(
        app[2] == 1 && app[3] == 3 && app[4] == 1 && app[5] == 4096 && app[7] == 4 && app[8] == 2,
        b"APP_EVIDENCE",
    );
    require(
        init[2] == 2 && init[3] == 1 && init[4] == 1 && init[5] == 1,
        b"INIT_EVIDENCE",
    );
    require(app[9..13] == [1, 1, 10, 2], b"RECORD_EVIDENCE");
    // B02 persistence: the recovery agent exported once and committed the
    // 120-byte snapshot before the app committed ciphertext (normal), or read
    // it back and restored the exact state before any key work (recover). The
    // wrong secret, the generation floor and a key opcode on the administrative
    // ingress were refused in both runs; generation 3 = generate, revoke, generate.
    let persist = evidence()
        .persist
        .unwrap_or_else(|| fail(b"PERSIST_SNAPSHOT"));
    // Relay: no export/restore at all; the admin GENERATE is still refused.
    let expected_init: [u64; 7] = if RECOVER {
        [0, 1, 1, 1, 1, 120, 3]
    } else if RELAY != 0 {
        [0, 0, 0, 0, 1, 120, 3]
    } else {
        [1, 1, 1, 1, 1, 120, 3]
    };
    require(init[6..13] == expected_init, b"KEYSTATE_EVIDENCE");
    let expected_persist: [u64; 6] = if RECOVER {
        [0, 1, 1, 1, 2, 1]
    } else {
        [1, 0, 0, 0, 2, 1]
    };
    require(persist == expected_persist, b"PERSISTED_RECORD_EVIDENCE");
    // Key Service administrative ingress: every request came from init, each
    // refusal is counted and no persistence opcode succeeded elsewhere.
    let admin_seen = word(s, KEY, 384);
    let admin_exports = word(s, KEY, 392);
    let admin_restores = word(s, KEY, 400);
    let admin_denied = word(s, KEY, 408);
    let admin_bad_caller = word(s, KEY, 416);
    // Relay: SUPPLY_SECRET accepted once; the second supply and the admin
    // GENERATE refused; no EXPORT or RESTORE reached the core.
    let admin_ok = if RELAY != 0 {
        admin_seen == 3
            && admin_exports == 0
            && admin_restores == 0
            && admin_denied == 2
            && admin_bad_caller == 0
            && word(s, KEY, 256) == 0
    } else {
        // B01-LOCK fixture: one more refused request carries the canary.
        let lock_fixture = u64::from(cfg!(feature = "test-system-lock"));
        admin_seen == if RECOVER { 4 } else { 5 } + lock_fixture
            && admin_exports == u64::from(!RECOVER)
            && admin_restores == 1
            && admin_denied == 3 + lock_fixture
            && admin_bad_caller == 0
            && word(s, KEY, 256) == 1
    };
    require(admin_ok, b"ADMIN_INGRESS_EVIDENCE");
    #[cfg(feature = "native-system-clock")]
    {
        let [first, second, monotonic_before, monotonic_after] =
            evidence().clock.unwrap_or_else(|| fail(b"CLOCK_SNAPSHOT"));
        print_number(b"SYSTEM:CLOCK:UTC_FIRST:", first);
        print_number(b"SYSTEM:CLOCK:UTC_SECOND:", second);
        print_number(b"SYSTEM:CLOCK:MONOTONIC_BEFORE:", monotonic_before);
        print_number(b"SYSTEM:CLOCK:MONOTONIC_AFTER:", monotonic_after);
        serial::write(b"SYSTEM:CLOCK:INVALID_ARGS:PASS\n");
    }
    let (entropy, denied, entropy_idle) = keyring::broker_summary();
    // H05-T5b: judge the pre-sleep values against the bounds below, and the
    // post-wake increments exactly: the kernel's 32-byte delivery, init's -6.
    #[cfg(all(
        feature = "test-system-s3-reinit",
        not(feature = "test-system-s3-lock-busy")
    ))]
    let (entropy_before, denied_before) = {
        let (delivered_before, denied_before) = suspend_state().broker_at_wake;
        require(
            entropy.checked_sub(delivered_before) == Some(32)
                && denied.checked_sub(denied_before) == Some(1),
            b"S3_REINIT_BROKER_DELTA",
        );
        (delivered_before, denied_before)
    };
    #[cfg(any(
        not(feature = "test-system-s3-reinit"),
        feature = "test-system-s3-lock-busy"
    ))]
    let (entropy_before, denied_before) = (entropy, denied);
    let (published, completed, block_idle) = m5::broker_summary();
    // Power fault fixtures: exactly the injected SHUTDOWN flush never completed.
    let refused = cfg!(any(
        feature = "test-system-power-flush-error",
        feature = "test-system-power-flush-timeout"
    ));
    let block_state = if refused {
        published[..2] == completed[..2] && published[2] == completed[2] + 1
    } else {
        block_idle && published == completed
    };
    require(
        entropy_before >= 1024
            && denied_before >= 1
            && entropy_idle
            && block_state
            && published[0] > 0,
        b"BROKER_STATE",
    );
    if cfg!(feature = "test-system-recover") {
        require(published[1..] == [0, 0], b"READONLY_MUTATED");
    } else {
        require(published[1] > 0 && published[2] >= 2, b"DISK_NOT_MUTATED");
    }
    // 192: 1 serving, 2 closed after an Ok SHUTDOWN; 376: volume uncertain.
    let (volume_state, uncertain) = if refused {
        (1, 1)
    } else if cfg!(feature = "native-system-power") {
        (2, 0)
    } else {
        (1, 0)
    };
    require(
        word(s, KEY, 192) == 1
            && word(s, VOLUME, 192) == volume_state
            && word(s, VOLUME, 376) == uncertain,
        b"SERVICE_STATE",
    );
    #[cfg(feature = "native-system-power")]
    {
        // Init: early call while the volume still holds BLOCK authority, then
        // its launch handle and a forged generation; refused runs end -16.
        // App: init's raw handle value (BadHandle or Type, depending on what that
        // slot of the app's own table holds by then) and its registry endpoint.
        let p = power_state();
        let init = &p.log[INIT][..p.count[INIT]];
        let tail = &init[init.len().min(3)..];
        let ok = init.len() >= 3
            && init[..3] == [-8, -6, -4]
            && if refused {
                tail.last() == Some(&-16) && tail[..tail.len() - 1].iter().all(|c| *c == -8)
            } else {
                tail.is_empty() && p.granted
            }
            && p.granted != refused
            && p.count[APP] == 2
            && matches!(p.log[APP][0], -4 | -6)
            && p.log[APP][1] == -6
            && p.count[KEY] == 0
            && p.count[VOLUME] == 0
            && p.dropped.iter().all(|count| *count == 0);
        if !ok {
            for slot in [INIT, APP] {
                for code in &p.log[slot][..p.count[slot]] {
                    print_number(b"SYSTEM:POWER:LOG_SLOT:", slot as u64);
                    print_number(b"SYSTEM:POWER:LOG_REFUSAL:", code.unsigned_abs());
                }
            }
            for slot in 0..p.dropped.len() {
                if p.dropped[slot] != 0 {
                    print_number(b"SYSTEM:POWER:DROPPED_SLOT:", slot as u64);
                    print_number(b"SYSTEM:POWER:DROPPED_REFUSALS:", p.dropped[slot] as u64);
                }
            }
            fail(b"POWER_EVIDENCE");
        }
        print_number(b"SYSTEM:POWER_INIT_REFUSALS:", init.len() as u64);
        print_number(b"SYSTEM:POWER_APP_REFUSALS:", p.count[APP] as u64);
        print_number(b"SYSTEM:POWER_NOT_DURABLE:", u64::from(refused));
    }
    // H05-T5d A7: no sleep. LOCK refused every attempt (-8, the instance is
    // still there), SUSPEND refused before and after it (-25), and every
    // kernel request was delivered before init's S5.
    #[cfg(feature = "test-system-s3-lock-busy")]
    {
        let p = suspend_state();
        let ok = !p.granted
            && !p.resumed
            && !p.ticked
            && p.log[..p.count] == [-25, -6, -4, -1, -25]
            && p.lost_count == 0
            && p.other == 0
            && p.held_refusals == 0
            && p.busy_locks == LOCK_BUSY_ATTEMPTS
            && p.busy_requests >= 1
            && p.busy_delivered == p.busy_requests
            && p.busy_ticket.is_none()
            && p.busy_drained
            && !keylock::gone()
            && !keylock::locked();
        if !ok {
            for code in &p.log[..p.count] {
                print_number(b"SYSTEM:SUSPEND:LOG_RESULT:", code.unsigned_abs());
            }
            fail(b"S3_LOCK_BUSY_EVIDENCE");
        }
        print_number(b"SYSTEM:SUSPEND_REFUSALS:", p.count as u64);
        print_number(b"SYSTEM:SUSPEND_GRANTED:", 0);
        print_number(b"SYSTEM:SUSPEND_LOCK_BUSY_REFUSALS:", p.busy_locks);
        print_number(b"SYSTEM:SUSPEND_LOCK_BUSY_REQUESTS:", p.busy_requests);
        print_number(b"SYSTEM:SUSPEND_LOCK_BUSY_DELIVERED:", p.busy_delivered);
    }
    #[cfg(all(feature = "test-system-s3", not(feature = "test-system-s3-lock-busy")))]
    {
        let p = suspend_state();
        // H05-T5b: after re-init init's entropy request reaches the authority
        // check with its POWER handle (-6); block requests stay lost (-14).
        let entropy = if cfg!(feature = "test-system-s3-reinit") {
            -6
        } else {
            -14
        };
        // H05-T5c: block reaches the authority check (-6) after the validated
        // re-init, or the quarantine refusal (-24); -14 in every other profile.
        #[cfg(feature = "test-system-s3-blk-reinit")]
        let block = if BLK_QUARANTINE_EXPECTED {
            m5::QUARANTINED
        } else {
            -6
        };
        #[cfg(not(feature = "test-system-s3-blk-reinit"))]
        let block = -14;
        let ok = p.granted
            && p.resumed
            && p.ticked
            && s.timer_interrupts > p.irqs_at_wake
            && if cfg!(feature = "test-system-s3-held-completion") {
                p.log[..p.count] == [-25, -6, -4, -1, -8, 0, -14] && p.held_refusals == 1
            } else {
                p.log[..p.count] == [-25, -6, -4, -1, 0, -14] && p.held_refusals == 0
            }
            && p.lost[..p.lost_count] == [(0x60, entropy), (0x40, block)]
            && p.other == 0;
        if !ok {
            for code in &p.log[..p.count] {
                print_number(b"SYSTEM:SUSPEND:LOG_RESULT:", code.unsigned_abs());
            }
            for (number, code) in &p.lost[..p.lost_count] {
                print_number(b"SYSTEM:SUSPEND:LOST_CALL:", *number);
                print_number(b"SYSTEM:SUSPEND:LOST_RESULT:", code.unsigned_abs());
            }
            fail(b"SUSPEND_EVIDENCE");
        }
        print_number(b"SYSTEM:SUSPEND_REFUSALS:", (p.count - 1) as u64);
        print_number(b"SYSTEM:SUSPEND_GRANTED:", 1);
        print_number(b"SYSTEM:SUSPEND_TICK_AFTER_WAKE:", 1);
        print_number(b"SYSTEM:SUSPEND_HELD_REFUSALS:", p.held_refusals);
        #[cfg(feature = "test-system-s3-reinit")]
        {
            let (served, stale) = keyring::reinit_evidence().unwrap_or_else(|| fail(b"S3_REINIT"));
            print_number(b"SYSTEM:SUSPEND_REINIT_ENTROPY:", served);
            print_number(b"SYSTEM:SUSPEND_REINIT_STALE_TICKET:", stale.unsigned_abs());
            print_number(
                b"SYSTEM:SUSPEND_REINIT_ENTROPY_AUTH:",
                p.lost[0].1.unsigned_abs(),
            );
        }
        #[cfg(feature = "test-system-s3-blk-reinit")]
        {
            let (quarantined, stale, [master, slave, line], carried) =
                m5::blk_reinit_evidence().unwrap_or_else(|| fail(b"S3_BLK_REINIT"));
            require(
                quarantined == BLK_QUARANTINE_EXPECTED,
                b"S3_BLK_REINIT_OUTCOME",
            );
            print_number(b"SYSTEM:SUSPEND_BLK_QUARANTINED:", u64::from(quarantined));
            print_number(b"SYSTEM:SUSPEND_BLK_STALE_TICKET:", stale.unsigned_abs());
            print_number(b"SYSTEM:SUSPEND_BLK_PIC_MASTER:", u64::from(master));
            print_number(b"SYSTEM:SUSPEND_BLK_PIC_SLAVE:", u64::from(slave));
            print_number(b"SYSTEM:SUSPEND_BLK_INTX_LINE:", u64::from(line));
            // O13: the lane-0 latch table survived the re-init or quarantine.
            require(carried, b"S3_BLK_LATCH_CARRIED");
            print_number(b"SYSTEM:SUSPEND_BLK_LATCH_CARRIED:", u64::from(carried));
        }
        // The logged magnitudes, so a host oracle can tell -14 from 0.
        #[cfg(not(feature = "test-system-s3-reinit"))]
        print_number(b"SYSTEM:SUSPEND_LOST_ENTROPY:", p.lost[0].1.unsigned_abs());
        print_number(b"SYSTEM:SUSPEND_LOST_BLOCK:", p.lost[1].1.unsigned_abs());
    }
    // Seals: legacy 1 + record 1 (+ persisted record in the normal run).
    // Unseals: legacy 1 + record 1 (+ restored record after the cold start).
    // Authentication failures: 9 record/legacy tampers + wrong recovery
    // secret (+ tampered restored record after the cold start).
    // Relay: init never restores with a wrong secret, so one refusal fewer.
    let (seals, unseals, auth_failures) = if RECOVER {
        (2, 3, 11)
    } else if RELAY != 0 {
        (3, 2, 9)
    } else {
        (3, 2, 10)
    };
    require(
        word(s, KEY, 224) == seals
            && word(s, KEY, 232) == unseals
            && word(s, KEY, 240) == auth_failures
            && word(s, KEY, 248) == 1,
        b"CRYPTO_COUNTERS",
    );
    for (label, value) in [
        (&b"SYSTEM:APP_READY:"[..], app[8]),
        (b"SYSTEM:APP_DOCS:", app[2]),
        (b"SYSTEM:APP_DENIED:", app[3]),
        (b"SYSTEM:APP_REVISION:", app[4]),
        (b"SYSTEM:APP_BYTES:", app[5]),
        (b"SYSTEM:APP_CONTENT_SUM:", app[6]),
        (b"SYSTEM:CRYPTO_SEAL:", word(s, KEY, 224)),
        (b"SYSTEM:CRYPTO_UNSEAL:", word(s, KEY, 232)),
        (b"SYSTEM:CRYPTO_TAMPER_DENIED:", word(s, KEY, 240)),
        (b"SYSTEM:RECORD_SEAL:", app[9]),
        (b"SYSTEM:RECORD_UNSEAL:", app[10]),
        (b"SYSTEM:RECORD_CONTEXT_DENIED:", app[11]),
        (b"SYSTEM:RECORD_REVOKED_DENIED:", app[12]),
        (b"SYSTEM:CRYPTO_RESTORES:", word(s, KEY, 256)),
        (b"SYSTEM:KEYSTATE_EXPORTS:", init[6]),
        (b"SYSTEM:KEYSTATE_RESTORES:", init[7]),
        (b"SYSTEM:KEYSTATE_WRONG_SECRET_DENIED:", init[8]),
        (b"SYSTEM:KEYSTATE_FLOOR_DENIED:", init[9]),
        (b"SYSTEM:KEYSTATE_ADMIN_OP_DENIED:", init[10]),
        (b"SYSTEM:KEYSTATE_BYTES:", init[11]),
        (b"SYSTEM:KEYSTATE_GENERATION:", init[12]),
        (b"SYSTEM:ADMIN_REQUESTS:", admin_seen),
        (b"SYSTEM:ADMIN_DENIED:", admin_denied),
        (b"SYSTEM:RECORD_PERSISTED:", persist[0]),
        (b"SYSTEM:RECORD_RESTORED_UNSEAL:", persist[1]),
        (b"SYSTEM:RECORD_RESTORED_REVOKED_DENIED:", persist[2]),
        (b"SYSTEM:RECORD_RESTORED_TAMPER_DENIED:", persist[3]),
        (b"SYSTEM:RECORD_ADMIN_DENIED:", persist[4]),
        (b"SYSTEM:RECORD_OBJECT_REVISION:", persist[5]),
        (b"SYSTEM:WORKER_SPAWNS:", spawns),
        (b"SYSTEM:WORKER_FAULTS:", evidence().faults),
        (b"SYSTEM:WORKER_RESTARTS:", init[4]),
        (b"SYSTEM:WORKER_KILLS:", kills),
        (b"SYSTEM:ENTROPY_BYTES:", entropy),
        (b"SYSTEM:DISK_READS:", published[0]),
        (b"SYSTEM:DISK_WRITES:", published[1]),
        (b"SYSTEM:DISK_FLUSHES:", published[2]),
    ] {
        print_number(label, value);
    }
    #[cfg(feature = "native-system-net")]
    {
        let net = evidence().net.unwrap();
        #[cfg(not(feature = "native-system-tcp"))]
        {
            require(
                net[2..6] == [1, 1, 1, 7] && net[8..13] == [2, 2, 1, 1, 1],
                b"NET_PROTOCOL_EVIDENCE",
            );
            for (label, value) in [
                (&b"SYSTEM:NET_ARP_REPLIES:"[..], net[2]),
                (b"SYSTEM:NET_UDP_ECHOES:", net[3]),
                (b"SYSTEM:NET_DNS_ANSWERS:", net[4]),
                (b"SYSTEM:NET_APP_DENIED:", net[5]),
                (b"SYSTEM:NET_TX_COMPLETIONS:", net[6]),
                (b"SYSTEM:NET_RX_FRAMES:", net[7]),
                (b"SYSTEM:NET_ARP_REJECTS:", net[8]),
                (b"SYSTEM:NET_DNS_REJECTS:", net[9]),
                (b"SYSTEM:NET_RX_TIMEOUTS:", net[10]),
                (b"SYSTEM:NET_CACHE_INTACT:", net[11]),
                (b"SYSTEM:NET_STALE_CAP:", net[12]),
            ] {
                print_number(label, value);
            }
        }
        #[cfg(feature = "native-system-tcp")]
        {
            let (extra, exit_tick) = evidence().tcp.unwrap_or_else(|| fail(b"TCP_SNAPSHOT"));
            require(
                net[2..6] == [2, 1, 1, 1] && net[8] == 1 && net[9] == 40 && net[12] == 1,
                b"TCP_PROTOCOL_EVIDENCE",
            );
            // Real boot-monotonic PIT time, independently bounded by the
            // kernel's observed exit tick. Never substitute loop counts or
            // claim wall/certificate time. Peer validation proves real packets.
            let elapsed = net[11]
                .checked_sub(net[10])
                .unwrap_or_else(|| fail(b"TCP_CLOCK_ORDER"));
            require(
                net[11] <= exit_tick
                    && exit_tick <= s.ticks
                    && exit_tick >= 300
                    && elapsed >= extra[1]
                    && extra[1] >= 300
                    && elapsed < 20_000
                    && extra[0] >= 2,
                b"TCP_CLOCK_EVIDENCE",
            );
            for (label, value) in [
                (&b"SYSTEM:TCP_HTTP:"[..], net[2]),
                (b"SYSTEM:TCP_REFUSED:", net[3]),
                (b"SYSTEM:TCP_RESET:", net[4]),
                (b"SYSTEM:TCP_RETRANSMIT:", net[5]),
                (b"SYSTEM:TCP_TX_COMPLETED:", net[6]),
                (b"SYSTEM:TCP_RX_DELIVERED:", net[7]),
                (b"SYSTEM:TCP_TIMEOUT:", net[8]),
                (b"SYSTEM:TCP_BODY_BYTES:", net[9]),
                (b"SYSTEM:TCP_START_TICKS:", net[10]),
                (b"SYSTEM:TCP_END_TICKS:", net[11]),
                (b"SYSTEM:TCP_STALE_CAP:", net[12]),
                (b"SYSTEM:TCP_CLOCK_READS:", extra[0]),
                (b"SYSTEM:TCP_TIMEOUT_TICKS:", extra[1]),
            ] {
                print_number(label, value);
            }
        }
        network::print_and_check(net[6], net[7]);
        #[cfg(feature = "test-system-net-link")]
        network::link::check_and_print(
            evidence().link.unwrap_or_else(|| fail(b"NETLINK_SNAPSHOT")),
        );
    }
    #[cfg(feature = "native-system-random")]
    random_fixture::check_and_print(s);
    #[cfg(feature = "native-system-images")]
    image_fixture::check_and_print(s);
    #[cfg(feature = "test-system-keystate-relay")]
    relay_check_and_print(s);
    serial::write(b"SYSTEM:BROKERS:PASS\n");
    #[cfg(feature = "native-system-lock")]
    keylock::check_and_print();
    for slot in [KEY, VOLUME] {
        // LOCK already ran the teardown of the Key Service instance.
        #[cfg(feature = "native-system-lock")]
        if slot == KEY && keylock::locked() {
            continue;
        }
        s.processes[slot].status = Status::Dying;
        s.cleanup(slot);
    }
    let stats = m4::engine().stats();
    require(
        stats.processes == 0
            && stats.handles == 0
            && stats.nodes == 0
            && stats.endpoints == 0
            && stats.queued_messages == 0
            && stats.block_objects == 0
            && stats.entropy_objects == 0
            && stats.launch_objects == 0
            && stats.process_objects == 0
            && stats.network_objects == 0
            && stats.power_objects == 0
            && m4::engine().check_invariants(),
        b"ENGINE_CLEANUP",
    );
    let grown = s
        .kernel
        .owned_table_count()
        .checked_sub(evidence().kernel_tables)
        .unwrap();
    require(
        s.free_count().checked_add(grown) == Some(s.free_baseline),
        b"FRAME_CLEANUP",
    );
    print_number(b"SYSTEM:LIVE_PROCESSES_AFTER:", stats.processes as u64);
    print_number(b"SYSTEM:FRAMES_RECOVERED:", 1);
    serial::write(b"SYSTEM:CLEANUP:PASS\n");
    // Review R1/R3, after every owner's teardown: no broker touched a device
    // after the wake outside the accounted re-init path, and the latch
    // suppressed exactly the one teardown reset the held-completion fixture
    // provokes (none otherwise).
    #[cfg(all(feature = "test-system-s3", not(feature = "test-system-s3-lock-busy")))]
    {
        let p = suspend_state();
        let (keyring, volume, accounted) = device_io();
        let (keyring_at_wake, volume_at_wake, accounted_at_wake) = p.io_at_wake;
        let accounted = accounted.saturating_sub(accounted_at_wake);
        let io_after = keyring
            .saturating_add(volume)
            .saturating_sub(keyring_at_wake.saturating_add(volume_at_wake))
            .saturating_sub(accounted);
        let suppressed = u64::from(cfg!(feature = "test-system-s3-held-completion"));
        // Counter liveness: init's pre-sleep block I/O and the services'
        // startup entropy seeds were counted, so a dead counter cannot pass.
        print_number(
            b"SYSTEM:SUSPEND_DEVICE_IO_AT_WAKE_KEYRING:",
            keyring_at_wake,
        );
        print_number(b"SYSTEM:SUSPEND_DEVICE_IO_AT_WAKE_VOLUME:", volume_at_wake);
        print_number(b"SYSTEM:SUSPEND_DEVICE_IO_AFTER_WAKE:", io_after);
        print_number(b"SYSTEM:SUSPEND_DEVICE_SUPPRESSED:", p.suppressed);
        // H05-T5c: the volume re-init used its accounted path, except the busy
        // injection, which must refuse before any device access.
        #[cfg(feature = "test-system-s3-blk-reinit")]
        {
            let volume_accounted = m5::reinit_accounted();
            print_number(
                b"SYSTEM:SUSPEND_DEVICE_IO_ACCOUNTED_VOLUME:",
                volume_accounted,
            );
            require(
                (volume_accounted > 0) != cfg!(feature = "test-system-s3-blk-reinit-busy"),
                b"S3_BLK_ACCOUNTED",
            );
        }
        // The re-init profile must have used its accounted path; no other may.
        let reinit = cfg!(feature = "test-system-s3-reinit");
        require(
            p.resumed
                && keyring_at_wake > 0
                && volume_at_wake > 0
                && io_after == 0
                && p.suppressed == suppressed
                && (accounted > 0) == reinit,
            b"SUSPEND_DEVICE_EVIDENCE",
        );
    }
    #[cfg(feature = "native-system-power")]
    if power_state().granted {
        // Every process is gone; re-check the gate on the final broker state.
        if let Err(code) = storage_closed() {
            print_number(b"SYSTEM:POWER:GATE_LOST:", code as u64);
            fail(b"POWER_GATE");
        }
        // H05-T5d labels: the sleep ran only after this instance was erased;
        // its unwrap secret is the public TEST constant, and nothing here
        // claims that key state survives (O5 open).
        #[cfg(all(feature = "test-system-s3", not(feature = "test-system-s3-lock-busy")))]
        {
            require(
                keylock::locked() && keylock::erased(),
                b"S3_LABEL_NOT_ERASED",
            );
            serial::write(
                b"SYSTEM:S3:LABEL:instance erased; unwrap secret is a public TEST constant\n",
            );
        }
        #[cfg(feature = "test-system-s3-lock-busy")]
        serial::write(b"SYSTEM:S3:LOCK_BUSY:LOCK=-8:SUSPEND=-25:S5\n");
        #[cfg(feature = "test-system-s3")]
        serial::write(b"SYSTEM:S3:LABEL:no key-state durability claimed (O5 open)\n");
        serial::write(if cfg!(feature = "test-system-s3-held-completion") {
            &b"SYSTEM:POWER:GATE:PASS\nSYSTEM:CASE:POWER_S3_HELD_COMPLETION:PASS\nM1:PASS\n"[..]
        } else if cfg!(feature = "test-system-s3-blk-reinit-busy") {
            b"SYSTEM:POWER:GATE:PASS\nSYSTEM:CASE:POWER_S3_BLK_REINIT_BUSY:PASS\nM1:PASS\n"
        } else if cfg!(feature = "test-system-s3-blk-reinit-identity") {
            b"SYSTEM:POWER:GATE:PASS\nSYSTEM:CASE:POWER_S3_BLK_REINIT_IDENTITY:PASS\nM1:PASS\n"
        } else if cfg!(feature = "test-system-s3-blk-reinit-capacity") {
            b"SYSTEM:POWER:GATE:PASS\nSYSTEM:CASE:POWER_S3_BLK_REINIT_CAPACITY:PASS\nM1:PASS\n"
        } else if cfg!(feature = "test-system-s3-blk-reinit-no-reset-ack") {
            b"SYSTEM:POWER:GATE:PASS\nSYSTEM:CASE:POWER_S3_BLK_REINIT_NO_ACK:PASS\nM1:PASS\n"
        } else if cfg!(feature = "test-system-s3-blk-reinit") {
            b"SYSTEM:POWER:GATE:PASS\nSYSTEM:CASE:POWER_S3_BLK_REINIT:PASS\nM1:PASS\n"
        } else if cfg!(feature = "test-system-s3-lock-busy") {
            b"SYSTEM:POWER:GATE:PASS\nSYSTEM:CASE:POWER_S3_LOCK_BUSY:PASS\nM1:PASS\n"
        } else if cfg!(feature = "test-system-s3-reinit") {
            &b"SYSTEM:POWER:GATE:PASS\nSYSTEM:CASE:POWER_S3_REINIT:PASS\nM1:PASS\n"[..]
        } else if cfg!(feature = "test-system-s3") {
            &b"SYSTEM:POWER:GATE:PASS\nSYSTEM:CASE:POWER_S3:PASS\nM1:PASS\n"[..]
        } else if cfg!(feature = "test-system-lock") {
            b"SYSTEM:POWER:GATE:PASS\nSYSTEM:CASE:POWER_LOCK:PASS\nM1:PASS\n"
        } else {
            b"SYSTEM:POWER:GATE:PASS\nSYSTEM:CASE:POWER:PASS\nM1:PASS\n"
        });
        crate::m2::power::off();
        fail(b"POWER_RETURNED");
    } else {
        serial::write(if cfg!(feature = "test-system-power-flush-error") {
            &b"SYSTEM:POWER:REFUSED:NOT_DURABLE\nSYSTEM:CASE:POWER_FLUSH_ERROR:PASS\n"[..]
        } else {
            b"SYSTEM:POWER:REFUSED:NOT_DURABLE\nSYSTEM:CASE:POWER_FLUSH_TIMEOUT:PASS\n"
        });
        serial::write(b"M1:PASS\n");
        serial::exit(0x10)
    }
    #[cfg(not(feature = "native-system-power"))]
    {
        serial::write(match RELAY {
            1 => &b"SYSTEM:CASE:RELAY:PASS\n"[..],
            2 => b"SYSTEM:CASE:RELAY_NOT_COMMITTED:PASS\n",
            3 => b"SYSTEM:CASE:RELAY_TIMEOUT:PASS\n",
            _ if cfg!(feature = "test-system-recover") => b"SYSTEM:CASE:RECOVER:PASS\n",
            _ => b"SYSTEM:CASE:NORMAL:PASS\n",
        });
        serial::write(b"M1:PASS\n");
        serial::exit(0x10)
    }
}
/// ADR-B02-KEY1-RELAY acceptance: exact commit counters of the Key Service,
/// init and the app per case. Every value is a ring-3 observation the kernel
/// checks; the host re-derives the disk oracle independently.
#[cfg(feature = "test-system-keystate-relay")]
fn relay_check_and_print(s: &State) {
    let init = evidence()
        .relay_init
        .unwrap_or_else(|| fail(b"RELAY_SNAPSHOT"));
    let app = evidence()
        .relay_app
        .unwrap_or_else(|| fail(b"RELAY_SNAPSHOT"));
    let [received, committed, refused, withheld, generation, sum, revision, begun, init_case] =
        init;
    let key = |at: u64| word(s, KEY, at);
    // (requests, ok, NOT_COMMITTED, unknown, app statuses) per case.
    let (requests, not_committed, unknown, app_status) = match RELAY {
        1 => (3, 0, 0, 0),
        2 => (5, 2, 0, 1),
        3 => (4, 0, 1, 7),
        _ => fail(b"RELAY_CASE"),
    };
    // wire::COMMIT_TICKS: completed commits stay below it, a timeout reaches it.
    let max_ticks = key(520);
    let latency = if RELAY == 3 {
        (1500..3000).contains(&max_ticks)
    } else {
        // 0 is legal: on a fast (KVM) host every commit can finish within one tick.
        (0..1500).contains(&max_ticks)
    };
    require(
        init_case == RELAY
            && app[2] == RELAY
            && key(440) == requests
            && key(448) == 3
            && key(456) == not_committed
            && key(464) == unknown
            && key(472) == 3
            && key(480) == 1
            && key(488) == 1
            && received == requests
            && committed == 3
            && refused == not_committed
            && withheld == unknown
            && begun == committed
            && revision == 3
            && generation == key(472)
            && sum != 0
            && app[0] == app_status
            && app[1] == app_status
            && latency,
        b"KEYSTATE_RELAY_EVIDENCE",
    );
    for (label, value) in [
        (&b"SYSTEM:KEYSTATE_RELAY_CASE:"[..], RELAY),
        (b"SYSTEM:KEYSTATE_RELAY_REQUESTS:", key(440)),
        (b"SYSTEM:KEYSTATE_RELAY_OK:", key(448)),
        (b"SYSTEM:KEYSTATE_RELAY_NOT_COMMITTED:", key(456)),
        (b"SYSTEM:KEYSTATE_RELAY_UNKNOWN:", key(464)),
        (b"SYSTEM:KEYSTATE_RELAY_LAST_GENERATION:", key(472)),
        (b"SYSTEM:KEYSTATE_RELAY_SECRET_SUPPLIED:", key(480)),
        (b"SYSTEM:KEYSTATE_RELAY_SECRET_REFUSED:", key(488)),
        (b"SYSTEM:KEYSTATE_RELAY_MAX_COMMIT_TICKS:", max_ticks),
        (b"SYSTEM:KEYSTATE_RELAY_INIT_RECEIVED:", received),
        (b"SYSTEM:KEYSTATE_RELAY_INIT_COMMITTED:", committed),
        (b"SYSTEM:KEYSTATE_RELAY_INIT_REFUSED:", refused),
        (b"SYSTEM:KEYSTATE_RELAY_INIT_WITHHELD:", withheld),
        (b"SYSTEM:KEYSTATE_RELAY_INIT_BEGUN:", begun),
        (b"SYSTEM:KEYSTATE_RELAY_INIT_REVISION:", revision),
        (b"SYSTEM:KEYSTATE_RELAY_INIT_GENERATION:", generation),
        (b"SYSTEM:KEYSTATE_RELAY_SUM:", sum),
        (b"SYSTEM:KEYSTATE_RELAY_APP_FIRST:", app[0]),
        (b"SYSTEM:KEYSTATE_RELAY_APP_SECOND:", app[1]),
    ] {
        print_number(label, value);
    }
}
