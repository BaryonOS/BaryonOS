//! Native L02 observer for a userspace caller and qsim job service.
//! The kernel supplies only process-isolated endpoint capabilities; parsing,
//! quota policy and simulation remain in ring 3.
use super::*;
use capability_core::{RECV, SEND};

const PROCESS_COUNT: usize = 2;
const REPORT_WORDS: usize = 15;
const TIMEOUT_TICKS: u64 = 4000;

fn fail(reason: &[u8]) -> ! {
    serial::write(b"QSIMJOB:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}

fn snapshot(process: &Process) -> [u64; REPORT_WORDS] {
    core::array::from_fn(|word| read_word(process.physical(DATA + word as u64 * 8)))
}

fn releasable_frames(state: &State) -> usize {
    let mut total = 0usize;
    for process in &state.processes[..state.amount] {
        let Some(paging) = process.paging.as_ref() else { fail(b"MISSING_ADDRESS_SPACE"); };
        total = total
            .checked_add(process.count)
            .and_then(|value| value.checked_add(process.kernel_frames.len()))
            .and_then(|value| value.checked_add(paging.owned_table_count()))
            .unwrap_or_else(|| fail(b"FRAME_COUNT_OVERFLOW"));
    }
    total
}

pub fn manifest(state: &mut State) {
    if state.amount != PROCESS_COUNT || state.fault || state.exited || state.unexpected {
        fail(b"MANIFEST_STATE");
    }
    let service_pid = state.processes[0].pid;
    let client_pid = state.processes[1].pid;
    if service_pid == 0 || client_pid == 0 || service_pid == client_pid {
        fail(b"MANIFEST_PID");
    }

    let engine = m4::engine();
    if engine.stats().processes != 0 { fail(b"ENGINE_NOT_EMPTY"); }
    engine.register(service_pid).unwrap_or_else(|_| fail(b"REGISTER_SERVICE"));
    engine.register(client_pid).unwrap_or_else(|_| fail(b"REGISTER_CLIENT"));

    let request_root = engine.create_endpoint(service_pid).unwrap_or_else(|_| fail(b"REQUEST_ENDPOINT"));
    let request_send = engine.bootstrap_delegate(service_pid, request_root, client_pid, SEND)
        .unwrap_or_else(|_| fail(b"REQUEST_SEND"));
    let request_recv = engine.derive(service_pid, request_root, RECV)
        .unwrap_or_else(|_| fail(b"REQUEST_RECV"));
    engine.close(service_pid, request_root).unwrap_or_else(|_| fail(b"REQUEST_ROOT_CLOSE"));

    let reply_root = engine.create_endpoint(client_pid).unwrap_or_else(|_| fail(b"REPLY_ENDPOINT"));
    let reply_send = engine.bootstrap_delegate(client_pid, reply_root, service_pid, SEND)
        .unwrap_or_else(|_| fail(b"REPLY_SEND"));
    let reply_recv = engine.derive(client_pid, reply_root, RECV)
        .unwrap_or_else(|_| fail(b"REPLY_RECV"));
    engine.close(client_pid, reply_root).unwrap_or_else(|_| fail(b"REPLY_ROOT_CLOSE"));

    write_word(state.processes[0].physical(DATA), 1);
    write_word(state.processes[0].physical(DATA + 24), client_pid);
    write_word(state.processes[0].physical(DATA + 32), request_recv);
    write_word(state.processes[0].physical(DATA + 40), reply_send);
    write_word(state.processes[1].physical(DATA), 2);
    write_word(state.processes[1].physical(DATA + 24), service_pid);
    write_word(state.processes[1].physical(DATA + 32), request_send);
    write_word(state.processes[1].physical(DATA + 40), reply_recv);

    for process in &state.processes[..state.amount] {
        let report = snapshot(process);
        if report[1] != 0 || report[2] != process.pattern || report[6..].iter().any(|value| *value != 0) {
            fail(b"MANIFEST_REPORT");
        }
    }
    if !engine.check_invariants() || engine.stats().endpoints != 2 || engine.stats().queued_messages != 0 {
        fail(b"MANIFEST_CAPABILITIES");
    }
    state.free_baseline = state.free_count().checked_add(releasable_frames(state))
        .unwrap_or_else(|| fail(b"FRAME_BASELINE_OVERFLOW"));
    serial::write(b"QSIMJOB:MANIFEST:PASS\n");
}

pub fn observe_exit(_state: &mut State, _slot: usize, _counter: u64) {
    fail(b"UNEXPECTED_EXIT");
}
pub fn observe_fault(_state: &mut State, _slot: usize, _context: &[u64; 22], _counter: u64) {
    fail(b"UNEXPECTED_FAULT");
}

pub fn poll(state: &mut State) -> Option<*const u64> {
    if state.amount != PROCESS_COUNT || state.fault || state.exited || state.unexpected {
        fail(b"UNEXPECTED_EVENT");
    }
    if state.ticks >= TIMEOUT_TICKS { fail(b"TIMEOUT"); }

    let service = snapshot(&state.processes[0]);
    let client = snapshot(&state.processes[1]);
    for (slot, report) in [service, client].iter().enumerate() {
        if report[2] != state.processes[slot].pattern || (report[1] == 0 && report[6] != 0) {
            fail(b"PRIVATE_REPORT");
        }
        if report[6] > 1 { fail(b"APP_FAILURE"); }
    }
    if service[6] != 1 || client[6] != 1 || state.timer_interrupts == 0 {
        return Some(state.select());
    }

    if service[0] != 1 || client[0] != 2
        || service[3] != state.processes[1].pid || client[3] != state.processes[0].pid
        || service[4] == 0 || service[5] == 0 || client[4] == 0 || client[5] == 0
        || service[7] != 15 || client[7] != 15
        || service[8] != 7 || client[8] != 7 || service[9] != 7 || client[9] != 7
        || service[10] == 0 || service[11] != 1 || client[11] != 1
        || service[12] != 1 || client[12] != 1 || service[13] != 2 || client[13] != 2
        || !(1..=65536).contains(&service[14]) {
        fail(b"RESULT_MISMATCH");
    }

    let stats = m4::engine().stats();
    if stats.processes != 2 || stats.endpoints != 2 || stats.queued_messages != 0 {
        fail(b"IPC_NOT_QUIESCENT");
    }
    state.audit();
    let released = releasable_frames(state);
    if state.free_count().checked_add(released) != Some(state.free_baseline) {
        fail(b"FRAME_BASELINE_CHANGED");
    }

    print_number(b"QSIMJOB:SERVICE_PID:", state.processes[0].pid);
    print_number(b"QSIMJOB:CLIENT_PID:", state.processes[1].pid);
    print_number(b"QSIMJOB:SERVICE_MAX_WORK:", service[10]);
    print_number(b"QSIMJOB:SERVICE_PEAK_ARENA:", service[14]);
    print_number(b"QSIMJOB:TIMER_IRQS:", state.timer_interrupts);
    print_number(b"QSIMJOB:SYSCALL_CALLS:", state.syscall_calls);
    serial::write(b"QSIMJOB:BELL:PASS\nQSIMJOB:SIX_QUBIT:PASS\n");
    serial::write(b"QSIMJOB:CANCEL:PASS\nQSIMJOB:WORK_BUDGET:PASS\n");
    serial::write(b"QSIMJOB:RESULT_PAGING:PASS\nQSIMJOB:IPC_AUTHORITY:PASS\n");

    arch::activate(state.kernel.root);
    arch::process_stack(arch::service_stack());
    state.current = MAX;
    for slot in 0..state.amount {
        state.processes[slot].status = Status::Dying;
        state.cleanup(slot);
    }
    let after = m4::engine().stats();
    if state.free_count() != state.free_baseline
        || after.processes != 0 || after.handles != 0 || after.nodes != 0
        || after.endpoints != 0 || after.queued_messages != 0 {
        fail(b"CLEANUP");
    }
    print_number(b"QSIMJOB:CLEANUP_FRAMES:", released as u64);
    serial::write(b"QSIMJOB:CLEANUP:PASS\nQSIMJOB:PASS\nM1:PASS\n");
    serial::exit(0x10)
}
