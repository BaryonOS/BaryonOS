//! FR-I01: composition of the existing process, entropy and block mechanisms.
//! This is an explicit integration acceptance manifest, not the full product.
//! Crypto/volume policy remains in the unchanged ring-3 service binaries.
use super::*;
use capability_core::{
    BlockScope, BLOCK_FLUSH, BLOCK_READ, BLOCK_WRITE, PROCESS_EXITED, PROCESS_FAULTED, RECV, SEND,
};

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
    // The app's complete handle table at manifest time. Every app check below
    // iterates this one list; APP_HANDLE_COUNT proves that nothing is outside it.
    let app_handles = [
        key_reply,
        app_key,
        volume_reply,
        app_volume,
        app_registry,
        app_control,
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
    } else if slot == APP {
        evidence().app = Some(words);
        let mut persist = [0; 6];
        for (i, at) in PERSIST_WORDS.iter().enumerate() {
            persist[i] = word(s, slot, *at);
        }
        evidence().persist = Some(persist);
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
    keyring::broker_tick(s);
    m5::broker_tick(s);
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
    let expected_init: [u64; 7] = if RECOVER {
        [0, 1, 1, 1, 1, 120, 3]
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
    require(
        admin_seen == if RECOVER { 4 } else { 5 }
            && admin_exports == u64::from(!RECOVER)
            && admin_restores == 1
            && admin_denied == 3
            && admin_bad_caller == 0
            && word(s, KEY, 256) == 1,
        b"ADMIN_INGRESS_EVIDENCE",
    );
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
        entropy >= 1024 && denied >= 1 && entropy_idle && block_state && published[0] > 0,
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
    // Seals: legacy 1 + record 1 (+ persisted record in the normal run).
    // Unseals: legacy 1 + record 1 (+ restored record after the cold start).
    // Authentication failures: 9 record/legacy tampers + wrong recovery
    // secret (+ tampered restored record after the cold start).
    let (seals, unseals, auth_failures) = if RECOVER { (2, 3, 11) } else { (3, 2, 10) };
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
    }
    #[cfg(feature = "native-system-random")]
    random_fixture::check_and_print(s);
    #[cfg(feature = "native-system-images")]
    image_fixture::check_and_print(s);
    serial::write(b"SYSTEM:BROKERS:PASS\n");
    for slot in [KEY, VOLUME] {
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
    #[cfg(feature = "native-system-power")]
    if power_state().granted {
        // Every process is gone; re-check the gate on the final broker state.
        if let Err(code) = storage_closed() {
            print_number(b"SYSTEM:POWER:GATE_LOST:", code as u64);
            fail(b"POWER_GATE");
        }
        serial::write(b"SYSTEM:POWER:GATE:PASS\nSYSTEM:CASE:POWER:PASS\nM1:PASS\n");
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
        serial::write(if cfg!(feature = "test-system-recover") {
            b"SYSTEM:CASE:RECOVER:PASS\n"
        } else {
            b"SYSTEM:CASE:NORMAL:PASS\n"
        });
        serial::write(b"M1:PASS\n");
        serial::exit(0x10)
    }
}
