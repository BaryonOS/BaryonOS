//! N02 acceptance manifest: the unchanged N01 grants (one raw NIC for the
//! ring-3 network service, IPC-only apps), but the service acquires its
//! address by DHCP on the wire and must withdraw it on DHCPNAK. App A holds a
//! UDP/53 grant to the lease DNS server and resolves names natively; app B
//! does not and is refused.
use super::*;
use capability_core::{PROCESS_EXITED, RECV, SEND};

const EPOCH: u64 = 1;
/// N02-T3b: one QMP link down/up replaces the NAK as the withdrawal cause.
const LINK: bool = cfg!(feature = "test-net-service-link");
/// N02-T6: QEMU's libslirp is the router, DHCP server and DNS forwarder; one
/// lease, no NAK. App A's grant names the libslirp DNS address 192.0.2.3.
const SLIRP: bool = cfg!(feature = "test-net-service-slirp");
const OFFSETS: [u64; 35] = [
    240, 400, 408, 416, 424, 432, 440, 448, 456, 464, 472, 480, 488, 496, 504, 512, 520, 528, 536,
    544, 552, 560, 568, 576, 584, 592, 600, 608, 616, 624, 632, 640, 648, 656, 664,
];
pub(super) const PROCESSES: usize = 3;
struct Evidence {
    snapshots: [Option<[u64; 35]>; 3],
    kernel_tables: usize,
    exits: [u64; 3],
}
static mut EVIDENCE: Evidence = Evidence {
    snapshots: [None; 3],
    kernel_tables: 0,
    exits: [0; 3],
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
        _ => panic!("N02 starts one service and two clients"),
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
    // Exactly the N01 grants: DHCP adds no capability, only the existing
    // NIC and entropy handles of the service are used.
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
        // App A's UDP grant is DNS to the peer, which the lease names as its
        // DNS server; app B keeps a non-DNS port and must be refused.
        (136, 53),
        (144, 9001),
        // 1 = DHCP with the NAK case; 2 = DHCP that follows the link (N02-T3b).
        (176, 1 + u64::from(LINK)),
        // Per-slot destination; 0 keeps the service default 192.0.2.1.
        (184, if SLIRP { 0xc000_0203 } else { 0 }),
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
            (96, if index == 0 { 53 } else { 9001 }),
            (104, if SLIRP { 7 } else { 6 }),
        ] {
            put(s, slot, offset, value);
        }
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
    let mut values = [0; 35];
    for (i, at) in OFFSETS.iter().enumerate() {
        values[i] = word(s, slot, *at);
    }
    let label: &[u8] = match slot {
        0 => b"NETSVC:SERVICE:",
        1 => b"NETSVC:APP_A:",
        _ => b"NETSVC:APP_B:",
    };
    // Snapshot and emit before generic cleanup destroys the process mapping.
    for (at, value) in OFFSETS.iter().zip(values) {
        print_field(label, *at, value);
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
    m4::engine().process_died(s.processes[slot].pid, PROCESS_EXITED, 0);
}
fn print_field(label: &[u8], at: u64, value: u64) {
    serial::write(label);
    let mut number = at;
    let mut bytes = [b'0'; 20];
    for byte in bytes.iter_mut().rev() {
        *byte = b'0' + (number % 10) as u8;
        number /= 10;
    }
    serial::write(&bytes);
    serial::write(b":");
    print_number(b"", value);
}
pub(super) fn observe_fault(_: &mut State, _: usize, _: &[u64; 22], _: u64) {
    fail(b"USER_FAULT")
}
pub(super) fn poll(s: &mut State) -> Option<*const u64> {
    keyring::broker_tick(s);
    network::broker_tick(s);
    require(!s.unexpected, b"UNEXPECTED_EVENT");
    if s.ticks > 6000 {
        for slot in 0..PROCESSES {
            if s.processes[slot].status != Status::Dead {
                print_number(b"NETSVC:TIMEOUT_SLOT:", slot as u64);
                print_number(b"NETSVC:TIMEOUT_STEP:", word(s, slot, 240));
            }
        }
        fail(b"TIMEOUT");
    }
    if evidence().snapshots.iter().any(Option::is_none) || !network::finished() {
        return None;
    }
    let service = evidence().snapshots[0].unwrap();
    if SLIRP {
        check_slirp(&service)
    } else {
        check_dhcp(&service)
    }
    network::print_and_check(service[3], service[4]);
    // The service saw the real down edge before withdrawing, and re-leased
    // only after the real up edge.
    #[cfg(feature = "test-net-service-link")]
    {
        let (down, up) = network::link::check_service();
        require(down <= service[25] && up < service[28], b"LINK_ORDER");
    }
    let (bytes, denied, healthy) = keyring::broker_summary();
    // The 8-byte stack seed plus one 4-byte draw per DHCP random sample.
    require(
        denied == 0 && healthy && Some(bytes) == service[26].checked_mul(4).map(|b| b + 8),
        b"ENTROPY_EVIDENCE",
    );
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
fn subnet_host(address: u64) -> bool {
    address >> 8 == 0xc000_02 && !matches!(address & 0xff, 0 | 1 | 255)
}
/// Two leases, exactly one NAK withdrawal, no app traffic outside a lease and
/// a fresh token after re-acquisition, ordered on one boot-monotonic clock.
fn check_dhcp(service: &[u64; 35]) {
    let [_, a, b] = evidence().snapshots.map(Option::unwrap);
    // UDP is DNS only: service.test on each lease and missing.test once (the
    // cached NXDOMAIN is not re-queried); the two peer spoofs are malformed.
    // Rejected requests, none from polling (Again/Busy/Closed are not counted):
    // A = malformed 1 + Access 1 + NotFound 2 + Stale 4 (two wrong tokens, the
    // withdrawn held token, the old token); B = malformed 1 + Access 1 +
    // refused Resolve 1 + wrong token 1.
    require(
        service[6] == 12
            && service[7] == 3
            && service[8] == 0
            && service[9] == 4
            && service[13] == 3
            && service[14] == 0
            && service[15] == 2
            && service[16] == 0
            && service[18] == 0
            && service[19] == 0
            && service[20] == 0,
        b"DHCP_SERVICE_EVIDENCE",
    );
    require(
        service[21] == 2
            && service[22] == 1
            && subnet_host(service[23])
            && subnet_host(service[24])
            && service[23] != service[24]
            && service[26] > 0
            && service[27] == 0,
        b"DHCP_LEASE_EVIDENCE",
    );
    require(
        a[2] == 2
            && a[3] == 0
            && a[4] == 1
            && a[5] == 3
            && a[6] == 1
            && a[9] == 2
            && a[10] == 1
            && a[11] == 1
            && a[14] == 1
            && a[19] == 1
            && a[30] == 2,
        b"DHCP_APP_A_EVIDENCE",
    );
    require(
        b[2] == 1
            && b[3] == 0
            && b[4] == 1
            && b[5] == 1
            && b[6] == 1
            && b[9] == 1
            && b[10] == 0
            && b[11] == 1
            && b[14] == 0
            && b[19] == 0
            && b[29] == 1
            && b[30..] == [0; 5],
        b"DHCP_APP_B_EVIDENCE",
    );
    // First resolution, held connection live, then lease withdrawn, then the
    // app observing its stale token and Resolve waiting, then the second
    // lease, the second resolution and the fresh exchange finishing no later
    // than app A's recorded exit. `waiting < rebound` is strict: the service
    // records `rebound` at the loop tick that first sees the new lease and
    // handles requests at that same tick or later, and app A records
    // `waiting` after its Again replies, so an Again from a leased Resolve or
    // OpenTcp (a fresh upstream query) would give `waiting >= rebound`.
    let (held, withdrawn, stale, rebound, fresh) = (a[15], service[25], a[20], service[28], a[16]);
    let (resolved, waiting, reresolved) = (a[29], a[31], a[32]);
    require(
        resolved != 0
            && resolved <= held
            && held <= withdrawn
            && withdrawn <= stale
            && stale <= waiting
            && waiting < rebound
            && rebound <= reresolved
            && reresolved <= fresh
            && fresh <= evidence().exits[1],
        b"DHCP_ORDER",
    );
    // Link observations by the service: downs, then ups including the first
    // sample. Outside the link profile the service never queries the link.
    let l = u64::from(LINK);
    require(
        service[33] == l && service[34] == 2 * l,
        b"DHCP_LINK_EVIDENCE",
    );
}
/// N02-T6: exactly one libslirp lease, never withdrawn; app A resolved the
/// public name once (address and count recorded for the harness, which binds
/// them to the DNS answer on the wire) and got NotFound for a reserved
/// .invalid name (the harness requires NXDOMAIN on the wire); app B was
/// refused DNS and finished one HTTP exchange through the libslirp translation.
fn check_slirp(service: &[u64; 35]) {
    let [_, a, b] = evidence().snapshots.map(Option::unwrap);
    // Two UDP datagrams (app A's public and .invalid queries), one TCP
    // connection (app B), no malformed or dropped frame. Rejected: A =
    // malformed 1 + Access 1 + NotFound 1; B = malformed 1 + Access 1 +
    // refused Resolve 1 + wrong token 1.
    require(
        service[6] == 7
            && service[7] == 2
            && service[8] == 0
            && service[9] == 1
            && service[13] == 3
            && service[14] == 0
            && service[15] == 0
            && service[16] == 0
            && service[18] == 0
            && service[19] == 0
            && service[20] == 0,
        b"SLIRP_SERVICE_EVIDENCE",
    );
    require(
        service[21] == 1
            && service[22] == 0
            && subnet_host(service[23])
            && service[24] == service[23]
            && service[26] > 0
            && service[27] == 0
            && service[33] == 0
            && service[34] == 0,
        b"SLIRP_LEASE_EVIDENCE",
    );
    require(
        a[3] == 0
            && a[29] != 0
            && a[29] <= evidence().exits[1]
            && a[33] != 0
            && (1..=8).contains(&a[34]),
        b"SLIRP_APP_A_EVIDENCE",
    );
    // App B received exactly RESPONSE_B (77 bytes) and requested one close.
    require(
        b[2] == 1 && b[8] == 77 && b[9] == 1 && b[29] == 1 && b[33] == 0 && b[34] == 0,
        b"SLIRP_APP_B_EVIDENCE",
    );
}
