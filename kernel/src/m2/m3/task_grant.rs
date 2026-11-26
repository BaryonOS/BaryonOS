//! A03 native acceptance: a ring-3 grant broker authorizes each request by the
//! kernel-supplied IPC sender PID. Three processes: broker (slot 0), owner app
//! (slot 1) and a hostile helper (slot 2, same client ELF as the owner).
//! The kernel grants endpoint rights only, never block or volume rights.
//!
//! FIXTURE: the bootstrap grant words in the broker's DATA page stand in for
//! the trusted permission dialog (U-owned, not built). `test-task-grant-denied`
//! omits them. FIXTURE lifecycle notice: on each client exit the kernel writes
//! the exited PID into the broker's private DATA page (as `nas.rs` does for its
//! admin), standing in for a product exit-notification path. No client can
//! write that page, so the notice cannot be forged over IPC.
//! This module replaces `sdk_app` via a path attribute in m3.rs.
use super::*;
use capability_core::{RECV, SEND};

const PROCESS_COUNT: usize = 3;
const TIMEOUT_TICKS: u64 = 2400;
const DENIED: bool = cfg!(feature = "test-task-grant-denied");

// Wire values shared with sdk/examples/task-grant-broker/src/lib.rs.
const OP_READ: u64 = 1;
const OP_WRITE: u64 = 2;
const OP_CLOSE: u64 = 3;
const OP_RETIRE: u64 = 4;
const OK: u64 = 1;
const NO: u64 = 2;
const GO: u64 = 4;
const RESUME: u64 = 5;
/// Object 1 content, little-endian words as the owner stores them.
const CONTENT: &[u8; 12] = b"A03 grant ok";

#[derive(Clone, Copy)]
struct ClientEvidence {
    exited: bool,
    pid: u64,
    code: u64,
    counter: u64,
    statuses: [u64; 10],
    content: [u64; 2],
}
const NONE: ClientEvidence = ClientEvidence {
    exited: false,
    pid: 0,
    code: 0,
    counter: 0,
    statuses: [0; 10],
    content: [0; 2],
};
static mut EVIDENCE: [ClientEvidence; 2] = [NONE; 2];

fn evidence() -> &'static mut [ClientEvidence; 2] {
    // SAFETY: KERNEL held, IF=0; serialized kernel service-stack access and
    // the borrow ends before scheduling.
    unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) }
}
fn fail(reason: &[u8]) -> ! {
    serial::write(b"TASKGRANT:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}
fn put(state: &State, slot: usize, offset: u64, value: u64) {
    write_word(state.processes[slot].physical(DATA + offset), value);
}
fn word(state: &State, slot: usize, offset: u64) -> u64 {
    read_word(state.processes[slot].physical(DATA + offset))
}
fn releasable_frames(state: &State) -> usize {
    let mut total = 0usize;
    for process in &state.processes[..state.amount] {
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
fn content_words() -> [u64; 2] {
    let mut bytes = [0u8; 16];
    bytes[..CONTENT.len()].copy_from_slice(CONTENT);
    let mut first = [0; 8];
    let mut second = [0; 8];
    first.copy_from_slice(&bytes[..8]);
    second.copy_from_slice(&bytes[8..]);
    [u64::from_le_bytes(first), u64::from_le_bytes(second)]
}

pub fn manifest(state: &mut State) {
    if state.amount != 2 || state.fault || state.exited || state.unexpected {
        fail(b"MANIFEST_STATE");
    }
    // The sdk-app profile boots two processes; the hostile helper is a second
    // instance of the client ELF, spawned through the existing loader path.
    state.amount = PROCESS_COUNT;
    state.spawn_process_from(2, SDK_APP_ELF);
    state.audit();
    let broker = state.processes[0].pid;
    let owner = state.processes[1].pid;
    let helper = state.processes[2].pid;
    if !(broker < owner && owner < helper) {
        fail(b"PID_ORDER");
    }
    let engine = m4::engine();
    for pid in [broker, owner, helper] {
        engine.register(pid).unwrap_or_else(|_| fail(b"REGISTER"));
    }

    // One request endpoint owned by the broker; both clients may only send.
    let request_root = engine
        .create_endpoint(broker)
        .unwrap_or_else(|_| fail(b"REQUEST_ENDPOINT"));
    let request_recv = engine
        .derive(broker, request_root, RECV)
        .unwrap_or_else(|_| fail(b"REQUEST_RECV"));
    let mut request_send = [0u64; 2];
    for (index, client) in [owner, helper].into_iter().enumerate() {
        request_send[index] = engine
            .bootstrap_delegate(broker, request_root, client, SEND)
            .unwrap_or_else(|_| fail(b"REQUEST_SEND"));
    }
    if engine.bootstrap_delegate(broker, request_root, helper, RECV)
        != Err(capability_core::Error::Access)
    {
        fail(b"REQUEST_RECV_TRANSFER");
    }
    engine
        .close(broker, request_root)
        .unwrap_or_else(|_| fail(b"REQUEST_ROOT_CLOSE"));

    // One private reply endpoint per client; only the broker may send to it.
    let mut reply_send = [0u64; 2];
    let mut reply_recv = [0u64; 2];
    for (index, client) in [owner, helper].into_iter().enumerate() {
        let root = engine
            .create_endpoint(client)
            .unwrap_or_else(|_| fail(b"REPLY_ENDPOINT"));
        reply_send[index] = engine
            .bootstrap_delegate(client, root, broker, SEND)
            .unwrap_or_else(|_| fail(b"REPLY_SEND"));
        reply_recv[index] = engine
            .derive(client, root, RECV)
            .unwrap_or_else(|_| fail(b"REPLY_RECV"));
        engine
            .close(client, root)
            .unwrap_or_else(|_| fail(b"REPLY_ROOT_CLOSE"));
    }

    for (index, client) in [owner, helper].into_iter().enumerate() {
        if engine
            .check_endpoint(client, request_send[index], SEND)
            .is_err()
            || engine
                .check_endpoint(client, reply_recv[index], RECV)
                .is_err()
            || engine
                .check_endpoint(broker, reply_send[index], SEND)
                .is_err()
        {
            fail(b"RIGHTS_MISSING");
        }
        if engine
            .check_endpoint(client, request_send[index], RECV)
            .is_ok()
            || engine
                .check_endpoint(client, reply_recv[index], SEND)
                .is_ok()
            || engine
                .check_block(client, request_send[index], capability_core::BLOCK_READ)
                .is_ok()
            || engine
                .check_block(client, reply_recv[index], capability_core::BLOCK_READ)
                .is_ok()
        {
            fail(b"RIGHTS_WIDENED");
        }
    }
    if engine.check_endpoint(broker, request_recv, RECV).is_err()
        || engine.check_endpoint(broker, request_recv, SEND).is_ok()
    {
        fail(b"BROKER_RIGHTS");
    }
    // Broker: receive + two reply sends; each client: request send + reply receive.
    if engine.stats().handles != 7
        || !engine.check_invariants()
        || engine.stats().queued_messages != 0
    {
        fail(b"MANIFEST_ENGINE");
    }

    for slot in 0..PROCESS_COUNT {
        put(state, slot, 32, 1);
        put(state, slot, 40, 0);
        for offset in (96..512).step_by(8) {
            put(state, slot, offset, 0);
        }
    }
    put(state, 0, 0, 0);
    put(state, 0, 56, request_recv);
    put(state, 0, 64, reply_send[0]);
    put(state, 0, 72, reply_send[1]);
    put(state, 0, 176, owner);
    put(state, 0, 184, helper);
    if !DENIED {
        // FIXTURE standing in for the trusted dialog: owner, object 1,
        // revision 1, READ only, until close.
        put(state, 0, 96, 1);
        put(state, 0, 104, owner);
        put(state, 0, 112, 1);
        put(state, 0, 120, 1);
        put(state, 0, 128, 1);
        put(state, 0, 136, 1);
        // FIXTURE: the same decision again, applied by the broker after the
        // owner's close so that the owner holds a fresh grant when it exits.
        for (index, value) in [1, owner, 1, 1, 1, 1].into_iter().enumerate() {
            put(state, 0, 360 + index as u64 * 8, value);
        }
    }
    for (index, slot) in [1usize, 2].into_iter().enumerate() {
        put(state, slot, 0, slot as u64);
        put(state, slot, 56, reply_recv[index]);
        put(state, slot, 64, request_send[index]);
        put(state, slot, 176, broker);
        // The value both clients put in the claimed actor field.
        put(state, slot, 184, owner);
    }

    state.free_baseline = state
        .free_count()
        .checked_add(releasable_frames(state))
        .unwrap_or_else(|| fail(b"FRAME_BASELINE_OVERFLOW"));
    *evidence() = [NONE; 2];
    serial::write(b"TASKGRANT:MANIFEST:PASS\n");
}

pub fn observe_exit(state: &mut State, slot: usize, counter: u64) {
    if !(1..=2).contains(&slot) || evidence()[slot - 1].exited {
        fail(b"UNEXPECTED_EXIT");
    }
    let mut statuses = [0u64; 10];
    for (index, status) in statuses.iter_mut().enumerate() {
        *status = word(state, slot, 192 + index as u64 * 8);
    }
    evidence()[slot - 1] = ClientEvidence {
        exited: true,
        // Captured here: the slot's PID is cleared once exit cleanup runs.
        pid: state.processes[slot].pid,
        code: state.last_event.code,
        counter,
        statuses,
        content: [word(state, slot, 320), word(state, slot, 328)],
    };
    // FIXTURE lifecycle notice into the broker's private page: PID first, then
    // the count the broker polls. The PID is kernel bookkeeping, not app data.
    let notices = word(state, 0, 448);
    if notices >= 2 {
        fail(b"EXIT_NOTICES");
    }
    put(state, 0, 456 + notices * 8, state.processes[slot].pid);
    put(state, 0, 448, notices + 1);
}

pub fn observe_fault(_state: &mut State, _slot: usize, _context: &[u64; 22], _counter: u64) {
    fail(b"UNEXPECTED_FAULT");
}

fn entry(pid: u64, op: u64, object: u64, status: u64) -> u64 {
    (pid << 32) | (op << 24) | (object << 16) | status
}

pub fn poll(state: &mut State) -> Option<*const u64> {
    if state.ticks >= TIMEOUT_TICKS || state.fault || state.unexpected {
        fail(b"TIMEOUT_OR_FAULT");
    }
    let [owner_evidence, helper_evidence] = *evidence();
    let done = word(state, 0, 40);
    if done > 1 {
        fail(b"BROKER_ERROR");
    }
    if !owner_evidence.exited || !helper_evidence.exited || done != 1 {
        // The m3 event/select loop alone schedules; idle observers must not recurse.
        return None;
    }
    if state.processes[1].status != Status::Dead
        || state.processes[2].status != Status::Dead
        || state.processes[0].status == Status::Dead
    {
        fail(b"LIFECYCLE");
    }
    if owner_evidence.code != 0 || helper_evidence.code != 0 {
        fail(b"CLIENT_EXIT_CODE");
    }

    let owner = owner_evidence.pid;
    let helper = helper_evidence.pid;
    let (first, after_helper, after_close) = if DENIED { (NO, NO, NO) } else { (OK, OK, OK) };
    let (fresh, after_forged) = if DENIED { (NO, NO) } else { (OK, OK) };
    // The broker's own decision log, in serving order. Helper entries sit
    // between the owner's phases, so the helper's CLOSE precedes the re-read,
    // its forged RETIRE precedes the owner's last read, and its final READ
    // follows the owner's exit and the broker's retirement.
    let expected = [
        entry(owner, OP_READ, 1, first),
        entry(owner, OP_WRITE, 1, NO),
        entry(owner, OP_READ, 2, NO),
        entry(helper, OP_READ, 1, NO),
        entry(helper, OP_CLOSE, 1, NO),
        entry(owner, OP_READ, 1, after_helper),
        entry(owner, OP_CLOSE, 1, after_close),
        entry(owner, OP_READ, 1, NO),
        entry(owner, OP_READ, 1, fresh),
        entry(helper, OP_RETIRE, 1, NO),
        entry(owner, OP_READ, 1, after_forged),
        entry(helper, OP_READ, 1, NO),
    ];
    for (index, value) in expected.into_iter().enumerate() {
        let actual = word(state, 0, 256 + index as u64 * 8);
        if actual != value {
            print_number(b"TASKGRANT:LOG_INDEX:", index as u64);
            print_number(b"TASKGRANT:LOG_ACTUAL:", actual);
            print_number(b"TASKGRANT:LOG_EXPECTED:", value);
            fail(b"BROKER_LOG");
        }
    }
    let (read_ok, denied, close_ok, fixtures, held) = if DENIED {
        (0, 12, 0, 0, 0)
    } else {
        (4, 7, 1, 2, 1)
    };
    if word(state, 0, 192) != read_ok
        || word(state, 0, 200) != denied
        || word(state, 0, 208) != close_ok
        // All four helper requests claimed the owner's PID; the owner's were truthful.
        || word(state, 0, 216) != 4
        || word(state, 0, 224) != 0
        || word(state, 0, 144) != fixtures
    {
        fail(b"BROKER_COUNTERS");
    }
    // Owner grants after the forged RETIRE; then before retirement, retired
    // and after, all derived from the broker's grant table, not from requests.
    if word(state, 0, 160) != held {
        fail(b"FORGED_RETIRE");
    }
    if word(state, 0, 152) != 1
        || word(state, 0, 232) != held
        || word(state, 0, 240) != held
        || word(state, 0, 248) != 0
        || word(state, 0, 448) != 2
        || word(state, 0, 456) != owner
        || word(state, 0, 464) != helper
    {
        fail(b"EXIT_RETIREMENT");
    }
    if owner_evidence.statuses
        != [
            first,
            NO,
            NO,
            RESUME,
            after_helper,
            after_close,
            NO,
            fresh,
            RESUME,
            after_forged,
        ]
        || helper_evidence.statuses != [GO, NO, NO, GO, NO, GO, NO, 0, 0, 0]
    {
        fail(b"CLIENT_STATUSES");
    }
    let content = if DENIED { [0, 0] } else { content_words() };
    if owner_evidence.content != content || helper_evidence.content != [0, 0] {
        fail(b"CONTENT");
    }

    if DENIED {
        serial::write(b"TASKGRANT:DENIED:PASS\nTASKGRANT:HELPER_DENIED:PASS\n");
    } else {
        serial::write(
            b"TASKGRANT:OWNER_READ:PASS\nTASKGRANT:OWNER_LIMITS:PASS\n\
TASKGRANT:HELPER_DENIED:PASS\nTASKGRANT:CLOSE_REVOKES:PASS\n\
TASKGRANT:FORGED_RETIRE_DENIED:PASS\nTASKGRANT:EXIT_RETIRES:PASS\n",
        );
    }

    // Both clients completed exit cleanup. Keep the established address-space
    // audit for the still-live broker before releasing it.
    state.audit_process(0);
    let released = releasable_frames(state);
    if state.free_count().checked_add(released) != Some(state.free_baseline) {
        fail(b"FRAME_BASELINE_CHANGED");
    }
    state.processes[0].status = Status::Dying;
    state.cleanup(0);
    let stats = m4::engine().stats();
    if state.free_count() != state.free_baseline
        || stats.processes != 0
        || stats.handles != 0
        || stats.nodes != 0
        || stats.endpoints != 0
        || stats.queued_messages != 0
    {
        fail(b"CLEANUP");
    }
    print_number(b"TASKGRANT:OWNER_COUNTER:", owner_evidence.counter);
    print_number(b"TASKGRANT:HELPER_COUNTER:", helper_evidence.counter);
    serial::write(b"TASKGRANT:CLEANUP:PASS\nTASKGRANT:PASS\nM1:PASS\n");
    serial::exit(0x10)
}
