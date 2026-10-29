//! K05 native supervisor profile (ADR-042): process management as
//! capabilities. The kernel adds four bounded syscalls (endpoint creation,
//! spawn, kill, status) and keeps every existing rule: a child only receives
//! the reduced capabilities its spawner passes, a `Process` object cannot
//! leave its holder, death reclaims everything through the existing `cleanup`
//! path, and children of a dead spawner end in the same event (no recursion).
//! The kernel knows no user protocol; the acceptance reads app-reported
//! evidence words plus its own counters. Single CPU, IF=0.
use super::*;
use capability_core::{
    KILL, MAX_PROCESS_OBJECTS, OBSERVE, PROCESS_EXITED, PROCESS_FAULTED, PROCESS_KILLED,
    PROCESS_RUNNING, SEND,
};

/// Whole-run budget in 100 Hz ticks; the host bounds wall time separately.
const RUN_TICKS: u64 = 6000;
const IMAGE_WORKER: u32 = 0;
const IMAGE_CLIENT: u32 = 1;
/// Frames a spawn may need beyond the image pages: heap 16, stack 16, kernel
/// stack 4, root/PDPT/PD/PT tables for three user regions.
const SPAWN_FRAME_MARGIN: usize = 64;
const CODE_MASK: u64 = (1 << 56) - 1;

const NORMAL: u64 = 0;
const CRASH: u64 = 1;
const DENIED: u64 = 2;
const PARENT_EXIT: u64 = 3;
const QUOTA: u64 = 4;

/// Supervisor evidence words (startup page offsets) copied at its exit.
const SUP_WORDS: [u64; 15] = [
    40, 240, 400, 408, 416, 424, 432, 440, 448, 456, 464, 472, 480, 488, 496,
];

#[derive(Clone, Copy)]
struct Evidence {
    spawns: u64,
    spawn_denied: u64,
    spawn_quota: u64,
    kills: u64,
    kill_errors: u64,
    status_calls: u64,
    endpoint_creates: u64,
    orphans_reaped: u64,
    child_exits: u64,
    child_faults: u64,
    max_live: u64,
    partial_spawn: bool,
    kernel_tables_at_manifest: usize,
    supervisor_exited: bool,
    supervisor_code: u64,
    sup: [u64; 15],
    image_of: [u32; MAX],
}
static mut EVIDENCE: Evidence = Evidence {
    spawns: 0,
    spawn_denied: 0,
    spawn_quota: 0,
    kills: 0,
    kill_errors: 0,
    status_calls: 0,
    endpoint_creates: 0,
    orphans_reaped: 0,
    child_exits: 0,
    child_faults: 0,
    max_live: 0,
    partial_spawn: false,
    kernel_tables_at_manifest: 0,
    supervisor_exited: false,
    supervisor_code: 0,
    sup: [0; 15],
    image_of: [u32::MAX; MAX],
};
fn evidence() -> &'static mut Evidence {
    // SAFETY: One CPU, IF=0; every borrow ends before scheduling or cleanup.
    unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) }
}

fn case() -> u64 {
    if cfg!(feature = "test-supervisor-crash") {
        CRASH
    } else if cfg!(feature = "test-supervisor-denied") {
        DENIED
    } else if cfg!(feature = "test-supervisor-parent-exit") {
        PARENT_EXIT
    } else if cfg!(feature = "test-supervisor-quota") {
        QUOTA
    } else {
        NORMAL
    }
}
fn marker() -> &'static [u8] {
    match case() {
        NORMAL => b"SUPERVISOR:CASE:NORMAL:PASS\n",
        CRASH => b"SUPERVISOR:CASE:CRASH:PASS\n",
        DENIED => b"SUPERVISOR:CASE:DENIED:PASS\n",
        PARENT_EXIT => b"SUPERVISOR:CASE:PARENT_EXIT:PASS\n",
        _ => b"SUPERVISOR:CASE:QUOTA:PASS\n",
    }
}
fn fail(reason: &[u8]) -> ! {
    serial::write(b"SUPERVISOR:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}
fn put(s: &State, slot: usize, offset: u64, value: u64) {
    write_word(s.processes[slot].physical(DATA + offset), value);
}
fn word(s: &State, slot: usize, offset: u64) -> u64 {
    read_word(s.processes[slot].physical(DATA + offset))
}
fn live_count(s: &State) -> u64 {
    s.processes
        .iter()
        .filter(|p| p.status != Status::Dead)
        .count() as u64
}
fn slot_of(s: &State, pid: u64) -> Option<usize> {
    s.processes
        .iter()
        .position(|p| p.status != Status::Dead && p.pid == pid)
}
fn releasable_frames(s: &State) -> usize {
    let mut total = 0usize;
    for process in &s.processes {
        if process.status == Status::Dead {
            continue;
        }
        let Some(paging) = process.paging.as_ref() else {
            fail(b"MISSING_ADDRESS_SPACE");
        };
        total = total
            .checked_add(process.count)
            .and_then(|value| value.checked_add(process.kernel_frames.len()))
            .and_then(|value| value.checked_add(paging.owned_table_count()))
            .unwrap_or_else(|| fail(b"FRAME_COUNT_OVERFLOW"));
    }
    total
}
fn image_elf(image: u32) -> Option<&'static [u8]> {
    match image {
        IMAGE_WORKER => Some(WORKER_ELF),
        IMAGE_CLIENT => Some(CLIENT_ELF),
        _ => None,
    }
}

pub fn manifest(s: &mut State) {
    if s.amount != MAX
        || s.processes[0].status == Status::Dead
        || s.processes[1..].iter().any(|p| p.status != Status::Dead)
        || s.fault
        || s.exited
        || s.unexpected
    {
        fail(b"MANIFEST_STATE");
    }
    let supervisor = s.processes[0].pid;
    let e = m4::engine();
    e.register(supervisor)
        .unwrap_or_else(|_| fail(b"REGISTER_SUPERVISOR"));
    let worker = e
        .create_launch(supervisor, IMAGE_WORKER)
        .unwrap_or_else(|_| fail(b"LAUNCH_WORKER"));
    let client = e
        .create_launch(supervisor, IMAGE_CLIENT)
        .unwrap_or_else(|_| fail(b"LAUNCH_CLIENT"));
    // Launch objects are typed: they are neither endpoints nor block devices,
    // and they carry no SEND/RECV/block right.
    if e.check_launch(supervisor, worker) != Ok(IMAGE_WORKER)
        || e.check_launch(supervisor, client) != Ok(IMAGE_CLIENT)
        || e.check_endpoint(supervisor, worker, SEND).is_ok()
        || e.check_block(supervisor, worker, capability_core::BLOCK_READ)
            .is_ok()
        || e.check_process(supervisor, worker, OBSERVE).is_ok()
        || e.stats().handles != 2
        || !e.check_invariants()
    {
        fail(b"MANIFEST_ENGINE");
    }
    for (offset, value) in [
        (0, case()),
        (32, 1),
        (40, 0),
        (48, worker),
        (72, client),
        (112, case()),
        (176, supervisor),
        (240, 0),
    ] {
        put(s, 0, offset, value);
    }
    let ev = evidence();
    *ev = Evidence {
        image_of: [u32::MAX; MAX],
        ..*ev
    };
    ev.max_live = 1;
    ev.kernel_tables_at_manifest = s.kernel.owned_table_count();
    s.free_baseline = s
        .free_count()
        .checked_add(releasable_frames(s))
        .unwrap_or_else(|| fail(b"FRAME_BASELINE_OVERFLOW"));
    serial::write(b"SUPERVISOR:MANIFEST:PASS\n");
}

pub fn syscall(s: &mut State, slot: usize, n: u64, a: [u64; 6]) -> (i64, u64) {
    match call(s, slot, n, a) {
        Ok(value) => (0, value),
        Err(code) => (code, 0),
    }
}

fn call(s: &mut State, slot: usize, n: u64, a: [u64; 6]) -> Result<u64, i64> {
    let pid = s.processes[slot].pid;
    match n {
        0x30 => {
            if a.iter().any(|x| *x != 0) {
                return Err(-1);
            }
            let handle = m4::engine().create_endpoint(pid).map_err(|e| e.code())?;
            evidence().endpoint_creates += 1;
            Ok(handle)
        }
        0x50 => spawn(s, slot, a),
        0x51 => {
            if a[1..].iter().any(|x| *x != 0) {
                return Err(-1);
            }
            kill(s, slot, a[0])
        }
        0x52 => {
            if a[1..].iter().any(|x| *x != 0) {
                return Err(-1);
            }
            let view = m4::engine()
                .check_process(pid, a[0], OBSERVE)
                .map_err(|e| e.code())?;
            evidence().status_calls += 1;
            Ok((u64::from(view.state) << 56) | (view.code & CODE_MASK))
        }
        _ => Err(-2),
    }
}

/// Authority and capacity are checked completely before any frame, PID,
/// node or object is taken; after that point a failure is a kernel invariant
/// violation and stops the run rather than leaving a half-built process.
fn spawn(s: &mut State, slot: usize, a: [u64; 6]) -> Result<u64, i64> {
    let pid = s.processes[slot].pid;
    let frames_before = s.free_count();
    let result = spawn_checked(s, pid, a);
    if result.is_err() && s.free_count() != frames_before {
        evidence().partial_spawn = true;
        fail(b"PARTIAL_SPAWN");
    }
    result
}

fn spawn_checked(s: &mut State, pid: u64, a: [u64; 6]) -> Result<u64, i64> {
    let engine = m4::engine();
    let image = match engine.check_launch(pid, a[0]) {
        Ok(image) => image,
        Err(error) => {
            evidence().spawn_denied += 1;
            return Err(error.code());
        }
    };
    let Some(elf) = image_elf(image) else {
        evidence().spawn_denied += 1;
        return Err(-1);
    };
    let mut caps = [(0u64, 0u64); 2];
    let mut count = 0;
    for i in 0..2 {
        let (handle, rights) = (a[2 + 2 * i], a[3 + 2 * i]);
        if handle == 0 {
            if rights != 0 {
                evidence().spawn_denied += 1;
                return Err(-1);
            }
            continue;
        }
        if let Err(error) = engine.check_delegation(pid, handle, rights) {
            evidence().spawn_denied += 1;
            return Err(error.code());
        }
        caps[count] = (handle, rights);
        count += 1;
    }
    let Some(free) = s.processes.iter().position(|p| p.status == Status::Dead) else {
        evidence().spawn_quota += 1;
        return Err(-7);
    };
    let Ok(parsed) = user::parse(elf) else {
        fail(b"EMBEDDED_IMAGE");
    };
    if !engine.can_register()
        || !engine.can_create_process()
        || engine.available_nodes() < 1 + count
        || engine.available_slots(pid) < 1
        || user::issue_pid(s.next_pid).is_err()
        || s.free_count() < parsed.pages + SPAWN_FRAME_MARGIN
    {
        evidence().spawn_quota += 1;
        return Err(-7);
    }
    // Commit: frames and tables first (prechecked), then authority.
    s.spawn_process_from(free, elf);
    let child = s.processes[free].pid;
    let engine = m4::engine();
    engine
        .register(child)
        .unwrap_or_else(|_| fail(b"SPAWN_REGISTER"));
    let handle = engine
        .create_process(pid, child)
        .unwrap_or_else(|_| fail(b"SPAWN_OBJECT"));
    let mut delegated = [0u64; 2];
    for (i, (source, rights)) in caps[..count].iter().enumerate() {
        delegated[i] = engine
            .bootstrap_delegate(pid, *source, child, *rights)
            .unwrap_or_else(|_| fail(b"SPAWN_DELEGATE"));
    }
    if !engine.check_invariants() {
        fail(b"SPAWN_ENGINE");
    }
    for (offset, value) in [
        (0, a[1]),
        (32, 1),
        (56, delegated[0]),
        (64, delegated[1]),
        (176, child),
    ] {
        put(s, free, offset, value);
    }
    s.audit_process(free);
    let ev = evidence();
    ev.spawns += 1;
    ev.image_of[free] = image;
    ev.max_live = core::cmp::max(ev.max_live, live_count(s));
    Ok(handle)
}

fn kill(s: &mut State, slot: usize, handle: u64) -> Result<u64, i64> {
    let pid = s.processes[slot].pid;
    let view = match m4::engine().check_process(pid, handle, KILL) {
        Ok(view) => view,
        Err(error) => {
            evidence().kill_errors += 1;
            return Err(error.code());
        }
    };
    if view.state != PROCESS_RUNNING {
        evidence().kill_errors += 1;
        return Err(-9);
    }
    let Some(target) = slot_of(s, view.child) else {
        fail(b"KILL_WITHOUT_PROCESS");
    };
    if target == slot {
        fail(b"SELF_KILL");
    }
    if !m4::engine().process_died(view.child, PROCESS_KILLED, 0) {
        fail(b"KILL_STATE");
    }
    s.processes[target].status = Status::Dying;
    s.cleanup(target);
    evidence().kills += 1;
    Ok(0)
}

/// Children whose spawner is gone end here, iteratively: each round ends at
/// least one process, so at most `MAX` rounds run on the kernel stack.
fn reap_orphans(s: &mut State) {
    loop {
        let mut out = [0u64; MAX_PROCESS_OBJECTS];
        let n = m4::engine().orphans(&mut out);
        if n == 0 {
            return;
        }
        for &child in &out[..n] {
            let Some(slot) = slot_of(s, child) else {
                fail(b"ORPHAN_WITHOUT_PROCESS");
            };
            if !m4::engine().process_died(child, PROCESS_KILLED, 0) {
                fail(b"ORPHAN_STATE");
            }
            s.processes[slot].status = Status::Dying;
            s.cleanup(slot);
            evidence().orphans_reaped += 1;
        }
    }
}

pub fn observe_exit(s: &mut State, slot: usize, _counter: u64) {
    let pid = s.processes[slot].pid;
    let code = s.last_event.code;
    let ev = evidence();
    if slot == 0 {
        if ev.supervisor_exited {
            fail(b"SUPERVISOR_EXIT_TWICE");
        }
        ev.supervisor_exited = true;
        ev.supervisor_code = code;
        for (i, offset) in SUP_WORDS.iter().enumerate() {
            ev.sup[i] = word(s, 0, *offset);
        }
    } else {
        ev.child_exits += 1;
    }
    m4::engine().process_died(pid, PROCESS_EXITED, code & CODE_MASK);
}

pub fn observe_fault(s: &mut State, slot: usize, context: &[u64; 22], _counter: u64) {
    let pid = s.processes[slot].pid;
    let vector = context[15];
    let ev = evidence();
    if slot == 0 {
        fail(b"SUPERVISOR_FAULT");
    }
    // Only the crash fixture may fault, exactly once, in a worker, with #UD.
    let expected =
        case() == CRASH && vector == 6 && ev.child_faults == 0 && ev.image_of[slot] == IMAGE_WORKER;
    if !expected {
        print_number(b"SUPERVISOR:FAULT_SLOT:", slot as u64);
        print_number(b"SUPERVISOR:FAULT_VECTOR:", vector);
        fail(b"UNEXPECTED_FAULT");
    }
    ev.child_faults += 1;
    m4::engine().process_died(pid, PROCESS_FAULTED, vector);
}

pub fn poll(s: &mut State) -> Option<*const u64> {
    reap_orphans(s);
    if s.unexpected {
        fail(b"UNEXPECTED_USER_EVENT");
    }
    if s.ticks >= RUN_TICKS {
        print_number(b"SUPERVISOR:LIVE_AT_TIMEOUT:", live_count(s));
        fail(b"TIMEOUT");
    }
    let live = live_count(s);
    let ev = evidence();
    ev.max_live = core::cmp::max(ev.max_live, live);
    if !ev.supervisor_exited {
        return None;
    }
    if live != 0 {
        print_number(b"SUPERVISOR:LIVE_AFTER_EXIT:", live);
        fail(b"CHILDREN_SURVIVED_SUPERVISOR");
    }
    finish(s)
}

fn require(condition: bool, reason: &[u8]) {
    if !condition {
        fail(reason);
    }
}

fn finish(s: &mut State) -> ! {
    let ev = *evidence();
    // Supervisor evidence words: see SUP_WORDS.
    let [status, step, restarts, replies, closed, instances, sup_kills, spawns_ok, spawn_quota_errors, killed_confirmed, faulted_observed, denied_matched, denied_total, client_exit_seen, denied_self] =
        ev.sup;
    if ev.supervisor_code != 0 || status != 1 {
        print_number(b"SUPERVISOR:EXIT_CODE:", ev.supervisor_code);
        print_number(b"SUPERVISOR:STATUS_WORD:", status);
        print_number(b"SUPERVISOR:STEP:", step);
        fail(b"SUPERVISOR_RESULT");
    }
    require(!ev.partial_spawn, b"PARTIAL_SPAWN_SEEN");
    for (label, value) in [
        (&b"SUPERVISOR:DIAG:SPAWNS:"[..], ev.spawns),
        (b"SUPERVISOR:DIAG:KILLS:", ev.kills),
        (b"SUPERVISOR:DIAG:KILL_ERRORS:", ev.kill_errors),
        (b"SUPERVISOR:DIAG:SPAWN_DENIED:", ev.spawn_denied),
        (b"SUPERVISOR:DIAG:CHILD_EXITS:", ev.child_exits),
        (b"SUPERVISOR:DIAG:SUP_KILLS:", sup_kills),
        (b"SUPERVISOR:DIAG:KILLED_CONFIRMED:", killed_confirmed),
        (b"SUPERVISOR:DIAG:DENIED_SELF:", denied_self),
        (b"SUPERVISOR:DIAG:DENIED_MATCHED:", denied_matched),
        (b"SUPERVISOR:DIAG:DENIED_TOTAL:", denied_total),
    ] {
        print_number(label, value);
    }
    match case() {
        NORMAL => {
            require(restarts == 1 && instances == 2, b"NORMAL_RESTART");
            require(replies == 24 && closed >= 1, b"NORMAL_CLIENT");
            require(
                ev.spawns == 3 && ev.kills == 2 && sup_kills == 2 && killed_confirmed == 2,
                b"NORMAL_LIFECYCLE",
            );
            require(
                ev.child_exits == 1 && ev.child_faults == 0 && ev.orphans_reaped == 0,
                b"NORMAL_ENDINGS",
            );
            require(client_exit_seen == 1, b"NORMAL_CLIENT_EXIT_OBSERVED");
            require(
                ev.spawn_denied == 0 && ev.spawn_quota == 0,
                b"NORMAL_DENIALS",
            );
        }
        CRASH => {
            require(restarts == 1 && instances == 2, b"CRASH_RESTART");
            require(replies == 24 && closed >= 1, b"CRASH_CLIENT");
            require(
                faulted_observed == 1 && ev.child_faults == 1,
                b"CRASH_FAULT",
            );
            require(
                ev.spawns == 3 && ev.kills == 1 && killed_confirmed == 1 && ev.child_exits == 1,
                b"CRASH_LIFECYCLE",
            );
            require(
                ev.orphans_reaped == 0 && client_exit_seen == 1,
                b"CRASH_ENDINGS",
            );
        }
        DENIED => {
            require(
                denied_total == 17 && denied_matched == denied_total && denied_self == 9,
                b"DENIED_CODES",
            );
            require(ev.spawns == 2 && ev.spawn_denied >= 7, b"DENIED_SPAWNS");
            require(ev.kill_errors >= 3, b"DENIED_KILLS");
            require(ev.kills == 1 && killed_confirmed == 1, b"DENIED_LIFECYCLE");
            require(
                ev.orphans_reaped == 0 && ev.child_faults == 0 && ev.child_exits == 1,
                b"DENIED_ENDINGS",
            );
        }
        PARENT_EXIT => {
            require(replies >= 4, b"PARENT_EXIT_PROGRESS");
            require(ev.spawns == 2 && ev.kills == 0, b"PARENT_EXIT_SPAWNS");
            require(
                ev.orphans_reaped == 2 && ev.child_exits == 0 && ev.child_faults == 0,
                b"PARENT_EXIT_ORPHANS",
            );
        }
        _ => {
            require(spawns_ok == 15 && spawn_quota_errors == 1, b"QUOTA_SPAWNS");
            require(ev.spawns == 15 && ev.spawn_quota == 1, b"QUOTA_KERNEL");
            require(ev.kills == 15 && killed_confirmed == 15, b"QUOTA_KILLS");
            require(ev.max_live == 16 && ev.orphans_reaped == 0, b"QUOTA_LIVE");
        }
    }
    let stats = m4::engine().stats();
    if stats.processes != 0
        || stats.handles != 0
        || stats.nodes != 0
        || stats.endpoints != 0
        || stats.block_objects != 0
        || stats.launch_objects != 0
        || stats.process_objects != 0
        || stats.queued_messages != 0
        || !m4::engine().check_invariants()
    {
        fail(b"ENGINE_NOT_EMPTY");
    }
    // Kernel page tables for the identity map of newly touched RAM are
    // permanent PageTable(0) frames by the M2 contract; they are accounted
    // separately so a real user-frame leak still fails the baseline.
    let grown = s
        .kernel
        .owned_table_count()
        .checked_sub(ev.kernel_tables_at_manifest)
        .unwrap_or_else(|| fail(b"KERNEL_TABLES_SHRANK"));
    if s.free_count().checked_add(grown) != Some(s.free_baseline) {
        print_number(b"SUPERVISOR:FREE_FRAMES:", s.free_count() as u64);
        print_number(b"SUPERVISOR:BASELINE_FRAMES:", s.free_baseline as u64);
        print_number(b"SUPERVISOR:KERNEL_TABLES_GROWN:", grown as u64);
        fail(b"FRAME_BASELINE");
    }
    serial::write(b"SUPERVISOR:READY\n");
    for (label, value) in [
        (&b"SUPERVISOR:SPAWNS:"[..], ev.spawns),
        (b"SUPERVISOR:KILLS:", ev.kills),
        (b"SUPERVISOR:SPAWN_DENIED:", ev.spawn_denied),
        (b"SUPERVISOR:SPAWN_QUOTA:", ev.spawn_quota),
        (b"SUPERVISOR:KILL_ERRORS:", ev.kill_errors),
        (b"SUPERVISOR:STATUS_CALLS:", ev.status_calls),
        (b"SUPERVISOR:ENDPOINT_CREATES:", ev.endpoint_creates),
        (b"SUPERVISOR:ORPHANS_REAPED:", ev.orphans_reaped),
        (b"SUPERVISOR:CHILD_EXITS:", ev.child_exits),
        (b"SUPERVISOR:CHILD_FAULTS:", ev.child_faults),
        (b"SUPERVISOR:MAX_LIVE:", ev.max_live),
        (b"SUPERVISOR:KERNEL_TABLES_GROWN:", grown as u64),
        (b"SUPERVISOR:RESTARTS:", restarts),
        (b"SUPERVISOR:CLIENT_REPLIES:", replies),
        (b"SUPERVISOR:CLIENT_CLOSED:", closed),
        (b"SUPERVISOR:WORKER_INSTANCES:", instances),
        (b"SUPERVISOR:DENIED_MATCHED:", denied_matched),
        (b"SUPERVISOR:DENIED_TOTAL:", denied_total),
        (b"SUPERVISOR:TICKS:", s.ticks),
    ] {
        print_number(label, value);
    }
    serial::write(marker());
    serial::write(b"SUPERVISOR:RESOURCES:PASS\nSUPERVISOR:PASS\nM1:PASS\n");
    serial::exit(0x10)
}
