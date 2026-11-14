//! N01 acceptance manifest. The kernel grants a raw NIC only to one ordinary
//! userspace service. Parsing, sockets, destinations and RPC stay in ring 3.
use super::*;
use capability_core::{PROCESS_EXITED, RECV, SEND};

const EPOCH: u64 = 1;
const OFFSETS: [u64; 21] = [
    240, 400, 408, 416, 424, 432, 440, 448, 456, 464, 472, 480, 488, 496, 504, 512, 520, 528, 536,
    544, 552,
];
/// N04 revoke case: a fourth process is the policy authority for the service.
const REVOKE: bool = cfg!(feature = "test-net-service-revoke");
const DEVICE_RESET: bool = cfg!(feature = "test-net-service-device-reset");
const EXHAUSTION: bool = cfg!(feature = "test-net-service-exhaustion");
// Full 12,768-pair workload: CI QEMU 8.2 reached 10,867 pairs by tick 6,000.
// This aggregate fixture deadline does not change any socket or IPC deadline.
const EXHAUSTION_TICK_DEADLINE: u64 = 12_000;
pub(super) const PROCESSES: usize = if REVOKE { 4 } else { 3 };
struct Evidence {
    snapshots: [Option<[u64; 21]>; 4],
    kernel_tables: usize,
    exits: [u64; 4],
    revoke_requested: u64,
    exhaustion_released: u64,
    #[cfg(feature = "test-net-service-device-reset")]
    reset_armed: bool,
}
static mut EVIDENCE: Evidence = Evidence {
    snapshots: [None; 4],
    kernel_tables: 0,
    exits: [0; 4],
    revoke_requested: 0,
    exhaustion_released: 0,
    #[cfg(feature = "test-net-service-device-reset")]
    reset_armed: false,
};
fn evidence() -> &'static mut Evidence {
    // SAFETY: KERNEL held, IF=0; no borrow survives a scheduling point or cleanup.
    unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) }
}
fn fail(reason: &[u8]) -> ! {
    serial::write(b"NETSVC:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}
fn require(ok: bool, reason: &[u8]) {
    if !ok {
        fail(reason)
    }
}
fn put(s: &State, slot: usize, at: u64, value: u64) {
    write_word(s.processes[slot].physical(DATA + at), value);
}
fn word(s: &State, slot: usize, at: u64) -> u64 {
    read_word(s.processes[slot].physical(DATA + at))
}
pub(super) fn process_elf(slot: usize) -> &'static [u8] {
    match slot {
        0 => NETWORK_SERVICE_ELF,
        1 | 2 => NETWORK_CLIENT_ELF,
        3 if REVOKE => NETWORK_CLIENT_ELF,
        _ => panic!("N01 starts one service and two clients (plus N04 policy)"),
    }
}
// No launch capability is issued in this manifest.
pub(super) fn image_elf(_: u32) -> Option<&'static [u8]> {
    None
}
fn channel(e: &mut capability_core::Engine, receiver: u64, sender: u64) -> (u64, u64) {
    let root = e.create_endpoint(receiver).unwrap();
    let receive = e.derive(receiver, root, RECV).unwrap();
    let send = e.bootstrap_delegate(receiver, root, sender, SEND).unwrap();
    e.close(receiver, root).unwrap();
    (receive, send)
}
pub(super) fn manifest(s: &mut State) {
    require(
        s.amount == MAX
            && s.processes[..PROCESSES]
                .iter()
                .all(|p| p.status != Status::Dead)
            && s.processes[PROCESSES..]
                .iter()
                .all(|p| p.status == Status::Dead),
        b"MANIFEST_STATE",
    );
    let ids = [s.processes[0].pid, s.processes[1].pid, s.processes[2].pid];
    let [service, app_a, app_b] = ids;
    let e = m4::engine();
    for pid in ids {
        e.register(pid).unwrap();
    }
    if REVOKE {
        let policy = s.processes[3].pid;
        e.register(policy).unwrap();
        // The policy process's only capability: send on the service's private
        // control endpoint. The service authenticates its kernel PID.
        let (control, order) = channel(e, service, policy);
        put(s, 0, 152, control);
        put(s, 0, 160, policy);
        for (at, value) in [
            (32, u64::from_le_bytes(*b"NETPOL01")),
            (40, EPOCH),
            (48, order),
            (64, service),
            (72, 0),
        ] {
            put(s, 3, at, value);
        }
        require(
            e.check_network(policy, order).is_err()
                && e.check_entropy(policy, order).is_err()
                && e.check_launch(policy, order).is_err(),
            b"POLICY_AUTHORITY",
        );
    }
    let network = e.create_network(service).unwrap();
    let key = e.check_network(service, network).unwrap();
    network::bind(service, network, key);
    let entropy = e.create_entropy(service).unwrap();
    for (at, value) in [
        (32, u64::from_le_bytes(*b"NETSVC01")),
        (40, EPOCH),
        (48, network),
        (104, entropy),
        (112, service),
        (120, 8080),
        (128, 8090),
        (136, 9000),
        (144, 9001),
        (168, u64::from(DEVICE_RESET)),
    ] {
        put(s, 0, at, value);
    }
    for (index, app) in [app_a, app_b].into_iter().enumerate() {
        let slot = index + 1;
        let (ingress, send) = channel(e, service, app);
        let (reply, response) = channel(e, app, service);
        let at = 56 + index as u64 * 24;
        for (offset, value) in [(at, ingress), (at + 8, response), (at + 16, app)] {
            put(s, 0, offset, value);
        }
        for (offset, value) in [
            (32, u64::from_le_bytes(*b"NETAPP01")),
            (40, EPOCH),
            (48, send),
            (56, reply),
            (64, service),
            (72, index as u64),
            (80, app),
            (88, 8080 + index as u64 * 10),
            (96, 9000 + index as u64),
            (
                104,
                if EXHAUSTION {
                    4
                } else if DEVICE_RESET {
                    3
                } else if index != 0 {
                    0
                } else if REVOKE {
                    2
                } else {
                    u64::from(cfg!(feature = "test-net-service-client-exit"))
                },
            ),
        ] {
            put(s, slot, offset, value);
        }
        // Bootstrap proves app endpoints convey only message authority. The
        // actual negative NIC syscall is additionally exercised by each app.
        for handle in [send, reply] {
            require(
                e.check_network(app, handle).is_err()
                    && e.check_entropy(app, handle).is_err()
                    && e.check_launch(app, handle).is_err()
                    && e.check_block(app, handle, capability_core::BLOCK_READ)
                        .is_err(),
                b"APP_AUTHORITY",
            );
        }
        require(
            e.check_network(app, network).is_err() && e.check_entropy(app, entropy).is_err(),
            b"FOREIGN_HANDLE",
        );
    }
    require(e.check_invariants(), b"ENGINE");
    let releasable: usize = s.processes[..PROCESSES]
        .iter()
        .map(|p| p.count + p.kernel_frames.len() + p.paging.as_ref().unwrap().owned_table_count())
        .sum();
    s.free_baseline = s.free_count().checked_add(releasable).unwrap();
    evidence().kernel_tables = s.kernel.owned_table_count();
    serial::write(b"NETSVC:MANIFEST:PASS\n");
}
pub(super) fn observe_exit(s: &mut State, slot: usize, _: u64) {
    require(
        slot < PROCESSES && evidence().snapshots[slot].is_none(),
        b"EXIT_SLOT",
    );
    if DEVICE_RESET {
        require(
            s.last_event.code == u64::from(slot == 0),
            b"RESET_EXIT_CODE",
        );
    }
    let mut values = [0; 21];
    for (i, at) in OFFSETS.iter().enumerate() {
        values[i] = word(s, slot, *at);
    }
    let label: &[u8] = match slot {
        0 => b"NETSVC:SERVICE:",
        1 => b"NETSVC:APP_A:",
        2 => b"NETSVC:APP_B:",
        _ => b"NETSVC:POLICY:",
    };
    // Snapshot and emit before generic cleanup destroys the process mapping.
    for (at, value) in OFFSETS.iter().zip(values) {
        serial::write(label);
        let mut number = *at;
        let mut bytes = [b'0'; 20];
        for byte in bytes.iter_mut().rev() {
            *byte = b'0' + (number % 10) as u8;
            number /= 10;
        }
        serial::write(&bytes);
        serial::write(b":");
        print_number(b"", value);
    }
    let expected_reset_failure = DEVICE_RESET
        && slot == 0
        && s.last_event.code == 1
        && matches!(values[0], 12 | 15)
        && values[1] == 1
        && values[17] == 0;
    if !expected_reset_failure
        && (s.last_event.code != 0 || values[0] != 0 || values[1] != 1 || values[17] != 1)
    {
        exhaustion_diagnostics(s);
        print_number(b"NETSVC:EXIT_SLOT:", slot as u64);
        fail(b"USER_RESULT");
    }
    evidence().snapshots[slot] = Some(values);
    evidence().exits[slot] = s.ticks;
    if slot == 1 {
        print_number(b"NETSVC:APP_A_EXIT_TICKS:", s.ticks);
    }
    if slot == 2 {
        print_number(b"NETSVC:APP_B_EXIT_TICKS:", s.ticks);
    }
    // This records status for runtime-spawned children. The bootstrap
    // processes deliberately have no Process object, so `false` is expected.
    m4::engine().process_died(s.processes[slot].pid, PROCESS_EXITED, 0);
}
pub(super) fn observe_fault(_: &mut State, _: usize, _: &[u64; 22], _: u64) {
    fail(b"USER_FAULT")
}
pub(super) fn poll(s: &mut State) -> Option<*const u64> {
    keyring::broker_tick(s);
    network::broker_tick(s);
    require(!s.unexpected, b"UNEXPECTED_EVENT");
    // The second owner may allocate only after the first owner has received
    // the real Exhausted reply. Neither this signal nor the observer alters
    // the service allocator, endpoint authority or production port budget.
    if EXHAUSTION
        && evidence().exhaustion_released == 0
        && s.processes[1].status != Status::Dead
        && s.processes[2].status != Status::Dead
        && word(s, 1, 544) == 1
    {
        require(
            word(s, 1, 408) == 12_768 && word(s, 1, 464) == 12_768,
            b"EXHAUSTION_PAIR_COUNT",
        );
        evidence().exhaustion_released = s.ticks.max(1);
        put(s, 2, 120, 1);
        print_number(b"NETSVC:EXHAUSTION_RELEASE_TICKS:", s.ticks.max(1));
    }
    if EXHAUSTION && evidence().exhaustion_released != 0 && s.processes[1].status != Status::Dead {
        let survivor_closed = if let Some(app) = evidence().snapshots[2] {
            app[19] == 1
        } else {
            s.processes[2].status != Status::Dead && word(s, 2, 544) == 1
        };
        if survivor_closed {
            // Keep the exhausted owner registered until the other owner has
            // actually allocated, observed Connecting and closed its socket.
            put(s, 1, 120, 2);
        }
    }
    #[cfg(feature = "test-net-service-device-reset")]
    {
        if !evidence().reset_armed
            && s.processes[1].status != Status::Dead
            && s.processes[2].status != Status::Dead
            && word(s, 1, 544) == 1
            && word(s, 2, 400) == 1
        {
            network::arm_device_reset();
            evidence().reset_armed = true;
            put(s, 1, 112, 1);
        }
        // A synchronization signal only: clients must still execute a real
        // syscall and observe their dead ingress capability being rejected.
        if evidence().snapshots[0].is_some() {
            for slot in 1..3 {
                if s.processes[slot].status != Status::Dead {
                    put(s, slot, 112, 2);
                }
            }
        }
    }
    // Test driver: once app A reports a live, idle connection, ask the policy
    // process to revoke. The authority itself is the policy's endpoint.
    if REVOKE
        && evidence().revoke_requested == 0
        && s.processes[1].status != Status::Dead
        && s.processes[3].status != Status::Dead
        && word(s, 1, 544) == 1
    {
        put(s, 3, 104, 1);
        evidence().revoke_requested = s.ticks.max(1);
        print_number(b"NETSVC:REVOKE_TICKS:", s.ticks);
    }
    let timed_out = if EXHAUSTION {
        s.ticks >= EXHAUSTION_TICK_DEADLINE
    } else {
        s.ticks > 6000
    };
    if timed_out {
        exhaustion_diagnostics(s);
        for slot in 0..PROCESSES {
            if s.processes[slot].status != Status::Dead {
                print_number(b"NETSVC:TIMEOUT_SLOT:", slot as u64);
                print_number(b"NETSVC:TIMEOUT_STEP:", word(s, slot, 240));
            }
        }
        fail(b"TIMEOUT");
    }
    if evidence().snapshots[..PROCESSES]
        .iter()
        .any(Option::is_none)
        || !network::finished()
    {
        return None;
    }
    let service = evidence().snapshots[0].unwrap();
    if DEVICE_RESET {
        #[cfg(feature = "test-net-service-device-reset")]
        check_device_reset(&service);
    } else if EXHAUSTION {
        check_exhaustion(&service);
    } else if REVOKE {
        check_revoke(&service);
    } else {
        check_n01(&service);
    }
    network::print_and_check(service[3], service[4]);
    let (bytes, denied, healthy) = keyring::broker_summary();
    require(bytes == 8 && denied == 0 && healthy, b"ENTROPY_EVIDENCE");
    let stats = m4::engine().stats();
    require(
        stats.processes == 0
            && stats.handles == 0
            && stats.nodes == 0
            && stats.endpoints == 0
            && stats.queued_messages == 0
            && stats.network_objects == 0
            && stats.entropy_objects == 0
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
    print_number(b"NETSVC:ENTROPY_BYTES:", bytes);
    serial::write(b"NETSVC:CLEANUP:PASS\nNETSVC:PASS\nM1:PASS\n");
    serial::exit(0x10)
}

#[cfg(feature = "test-net-service-device-reset")]
fn check_device_reset(service: &[u64; 21]) {
    let e = evidence();
    let (fault_tick, ticket, rx_errors) = network::device_reset_evidence();
    require(
        service[0] == if rx_errors == 1 { 15 } else { 12 },
        b"RESET_FAILURE_SITE",
    );
    require(service[16] == 12 * rx_errors, b"RESET_RX_ERROR");
    require(
        e.reset_armed && matches!(service[0], 12 | 15) && service[1] == 1,
        b"RESET_SERVICE_EXIT",
    );
    require(
        service[14] == service[0] && service[18] >= fault_tick,
        b"RESET_FAILURE_CAUSE",
    );
    require(
        service[19] == 12 && service[20] == ticket,
        b"RESET_TERMINAL_TICKET",
    );
    require(
        service[17] == 0 && service[13] == 0,
        b"RESET_NOT_ORDERLY_RETIRE",
    );
    for slot in 1..3 {
        let app = e.snapshots[slot].unwrap();
        require(
            app[0] == 0 && app[1] == 1 && app[17] == 1,
            b"RESET_APP_FINISHED",
        );
        require(
            app[4] == 1 && app[6] == 1 && app[11] == 1,
            b"RESET_APP_AUTHORITY",
        );
        require(matches!(app[18], 4 | 9 | 10), b"RESET_DEAD_INGRESS");
        require(
            app[14] >= service[18] && app[14] <= fault_tick + 500,
            b"RESET_APP_LATENCY",
        );
        require(e.exits[slot] >= app[14], b"RESET_APP_EXIT_ORDER");
        if slot == 1 {
            require(
                app[7] == 2 && app[19] == 1 && app[20] > 0 && app[20] <= fault_tick,
                b"RESET_LIVE_THEN_PUBLISHED",
            );
        } else {
            require(
                app[7] == 0 && app[19] == 0 && app[20] == 0,
                b"RESET_OTHER_APP_SCOPE",
            );
        }
    }
}
/// N04: app B keeps its full N01 behaviour; app A saw its live connection end
/// promptly after the policy revocation and every later use denied.
fn check_revoke(service: &[u64; 21]) {
    let e = evidence();
    require(
        service[7] == 1
            && service[8] == 1
            && service[9] == 6
            && service[13] == 3
            && service[14] == 0
            && service[15] >= 2
            && service[16] >= 1
            && service[18] == 0
            && service[19] == 1,
        b"REVOKE_SERVICE_EVIDENCE",
    );
    let [_, a, b, policy] = e.snapshots.map(Option::unwrap);
    require(
        a[2] == 0
            && a[3] == 0
            && a[4] == 2
            && a[5] == 5
            && a[6] == 1
            && a[7] >= 1
            && a[9] == 0
            && a[11] == 1
            && a[14] < 500
            && a[18] == 0
            && a[19] == 2,
        b"REVOKED_APP_EVIDENCE",
    );
    check_full_app(&b);
    require(
        policy[1] == 1
            && policy[17] == 1
            && policy[19] == 1
            && policy[2..17].iter().all(|v| *v == 0)
            && policy[18] == 0,
        b"POLICY_EVIDENCE",
    );
    // Causal order on one boot-monotonic clock: kernel request, policy send,
    // service revocation, then app A observing its connection closed.
    let (requested, sent, handled, closed) = (e.revoke_requested, policy[20], service[20], a[20]);
    require(
        requested != 0 && requested <= sent && sent <= handled && handled <= closed,
        b"REVOKE_ORDER",
    );
}
fn exhaustion_diagnostics(s: &State) {
    if !EXHAUSTION {
        return;
    }
    print_number(b"NETSVC:EXHAUSTION_FAILED_TICK:", s.ticks);
    if s.processes[1].status != Status::Dead {
        print_number(b"NETSVC:EXHAUSTION_A_OPENED:", word(s, 1, 408));
        print_number(b"NETSVC:EXHAUSTION_A_CLOSED:", word(s, 1, 464));
    }
    if s.processes[0].status != Status::Dead {
        print_number(b"NETSVC:EXHAUSTION_SERVICE_REQUESTS:", word(s, 0, 432));
        print_number(b"NETSVC:EXHAUSTION_SERVICE_OPENED:", word(s, 0, 464));
        print_number(b"NETSVC:EXHAUSTION_SERVICE_CLOSED:", word(s, 0, 488));
    }
}

fn check_exhaustion(service: &[u64; 21]) {
    require(
        service[2] > 0
            && service[4] == 0
            && service[5] == 25_544
            && service[6] == 4
            && service[9] == 12_769
            && service[12] == 12_769
            && service[13] == 3
            && [7, 8, 10, 11, 14, 15, 16, 18, 19, 20]
                .iter()
                .all(|i| service[*i] == 0),
        b"EXHAUSTION_SERVICE",
    );
    for slot in 1..3 {
        let app = evidence().snapshots[slot].unwrap();
        let pairs = if slot == 1 { 12_768 } else { 1 };
        require(
            app[2] == pairs
                && app[3] == u64::from(slot == 2)
                && app[4] == 1
                && app[6] == 1
                && app[9] == pairs
                && app[11] == 1
                && app[19] == 1
                && (slot == 2 || app[14] == 0)
                && [5, 7, 8, 10, 12, 13, 15, 16, 18]
                    .iter()
                    .all(|i| app[*i] == 0),
            b"EXHAUSTION_APP",
        );
    }
    let exhausted = evidence().snapshots[1].unwrap()[20];
    let survivor = evidence().snapshots[2].unwrap()[20];
    let closed = evidence().snapshots[2].unwrap()[14];
    let released = evidence().exhaustion_released;
    require(
        exhausted > 0
            && exhausted <= released
            && released <= survivor
            && survivor <= closed
            && closed <= evidence().exits[2]
            && closed <= evidence().exits[1],
        b"EXHAUSTION_CAUSALITY",
    );
}

fn check_full_app(app: &[u64; 21]) {
    require(
        app[2] == 2
            && app[3] == 1
            && app[4] == 1
            && app[5] >= 5
            && app[6] == 1
            && app[9] == 2
            && app[10] == 4
            && app[11] == 1
            && app[12] == 1
            && app[13] == 1
            && app[14] == 1
            && app[15] == 1
            && app[18] == 0
            && app[19] == 0
            && app[20] == 0,
        b"APP_EVIDENCE",
    );
}
fn check_n01(service: &[u64; 21]) {
    let departing = cfg!(feature = "test-net-service-client-exit");
    let udp = if departing { 1 } else { 2 };
    require(
        service[7] == udp
            && service[8] == udp
            && service[9] == if departing { 6 } else { 10 }
            && service[13] == 3
            && service[14] == 0
            && service[15] >= 2 * udp
            && service[16] >= udp
            && service[18] == u64::from(departing)
            && service[19] == 0
            && service[20] == 0,
        b"SERVICE_EVIDENCE",
    );
    for (index, app) in evidence().snapshots[1..].iter().flatten().enumerate() {
        if departing && index == 0 {
            require(
                app[2] == 0
                    && app[3] == 0
                    && app[4] == 1
                    && app[6] == 1
                    && app[11] == 1
                    && app[18] == 1,
                b"DEPARTED_APP_EVIDENCE",
            );
        } else {
            check_full_app(app);
        }
    }
    if departing {
        require(
            evidence().exits[2] > evidence().exits[1],
            b"SURVIVOR_PROGRESS",
        );
    }
}
