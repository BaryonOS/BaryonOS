//! N01 acceptance manifest. The kernel grants a raw NIC only to one ordinary
//! userspace service. Parsing, sockets, destinations and RPC stay in ring 3.
use super::*;
use capability_core::{PROCESS_EXITED, RECV, SEND};

const EPOCH: u64 = 1;
const OFFSETS: [u64; 20] = [
    240, 400, 408, 416, 424, 432, 440, 448, 456, 464, 472, 480, 488, 496, 504, 512, 520, 528, 536,
    544,
];
/// N04 revoke case: a fourth process is the policy authority for the service.
const REVOKE: bool = cfg!(feature = "test-net-service-revoke");
pub(super) const PROCESSES: usize = if REVOKE { 4 } else { 3 };
struct Evidence {
    snapshots: [Option<[u64; 20]>; 4],
    kernel_tables: usize,
    exits: [u64; 4],
    revoke_requested: u64,
}
static mut EVIDENCE: Evidence = Evidence {
    snapshots: [None; 4],
    kernel_tables: 0,
    exits: [0; 4],
    revoke_requested: 0,
};
fn evidence() -> &'static mut Evidence {
    // SAFETY: One CPU, IF=0; no borrow survives a scheduling point or cleanup.
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
                if index != 0 {
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
    let mut values = [0; 20];
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
    if s.last_event.code != 0 || values[0] != 0 || values[1] != 1 || values[17] != 1 {
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
    if s.ticks > 6000 {
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
    if REVOKE {
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
/// N04: app B keeps its full N01 behaviour; app A saw its live connection end
/// promptly after the policy revocation and every later use denied.
fn check_revoke(service: &[u64; 20]) {
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
    require(
        e.revoke_requested != 0 && e.exits[3] >= e.revoke_requested,
        b"REVOKE_ORDER",
    );
}
fn check_full_app(app: &[u64; 20]) {
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
            && app[19] == 0,
        b"APP_EVIDENCE",
    );
}
fn check_n01(service: &[u64; 20]) {
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
            && service[19] == 0,
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
