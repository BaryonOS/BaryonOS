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
struct Evidence {
    app: Option<[u64; 13]>,
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
    // SAFETY: One CPU, IF=0. Each borrow ends before a syscall or scheduling.
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
    let initial = if cfg!(feature = "native-system-net") {
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
    let entropy = e.create_entropy(key).unwrap();
    let (volume_reply, volume_send_reply) = channel(e, app, volume);
    let (volume_ingress, app_volume) = channel(e, volume, app);
    // The unused second volume domain belongs to init, never the app.
    let (volume_other, _) = channel(e, volume, init);
    let (_, volume_reply_other) = channel(e, init, volume);
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
        (160, app),
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
        (160, app),
        (176, init),
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
    for handle in [
        key_reply,
        app_key,
        volume_reply,
        app_volume,
        app_registry,
        app_control,
    ] {
        require(
            e.check_entropy(app, handle).is_err()
                && e.check_block(app, handle, BLOCK_READ).is_err()
                && e.check_launch(app, handle).is_err()
                && e.check_network(app, handle).is_err(),
            b"APP_AUTHORITY",
        );
    }
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
    keyring::broker_tick(s);
    m5::broker_tick(s);
    #[cfg(feature = "native-system-net")]
    network::broker_tick(s);
    supervisor::reap_orphans(s);
    require(!s.unexpected, b"UNEXPECTED_EVENT");
    if s.ticks >= 20_000 {
        for slot in 0..if cfg!(feature = "native-system-net") {
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
    let (entropy, denied, entropy_idle) = keyring::broker_summary();
    let (published, completed, block_idle) = m5::broker_summary();
    require(
        entropy >= 1024
            && denied >= 1
            && entropy_idle
            && block_idle
            && published == completed
            && published[0] > 0,
        b"BROKER_STATE",
    );
    if cfg!(feature = "test-system-recover") {
        require(published[1..] == [0, 0], b"READONLY_MUTATED");
    } else {
        require(published[1] > 0 && published[2] >= 2, b"DISK_NOT_MUTATED");
    }
    require(
        word(s, KEY, 192) == 1 && word(s, VOLUME, 192) == 1 && word(s, VOLUME, 376) == 0,
        b"SERVICE_STATE",
    );
    require(
        word(s, KEY, 224) == 2
            && word(s, KEY, 232) == 2
            && word(s, KEY, 240) == 9
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
    serial::write(if cfg!(feature = "test-system-recover") {
        b"SYSTEM:CASE:RECOVER:PASS\n"
    } else {
        b"SYSTEM:CASE:NORMAL:PASS\n"
    });
    serial::write(b"M1:PASS\n");
    serial::exit(0x10)
}
