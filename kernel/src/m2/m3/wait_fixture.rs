//! Native two-process wait acceptance; no product bootstrap authority.
use super::*;
use capability_core::SEND;

#[derive(Clone, Copy)]
struct Evidence {
    exited: [bool; 2],
    assertions: [u64; 2],
    ticks: [u64; 2],
    masks: [u64; 2],
}
static mut EVIDENCE: Evidence = Evidence {
    exited: [false; 2],
    assertions: [0; 2],
    ticks: [0; 2],
    masks: [0; 2],
};
fn evidence() -> &'static mut Evidence {
    // SAFETY: One CPU, IF=0; this borrow ends before scheduling.
    unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) }
}
fn fail(reason: &[u8]) -> ! {
    serial::write(b"IPCWAIT:FAIL:");
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
    state.processes[..state.amount]
        .iter()
        .filter(|p| p.status != Status::Dead)
        .map(|p| p.count + p.kernel_frames.len() + p.paging.as_ref().unwrap().owned_table_count())
        .sum()
}
pub fn manifest(state: &mut State) {
    if state.amount != 2 || state.fault || state.exited || state.unexpected {
        fail(b"MANIFEST_STATE");
    }
    #[cfg(feature = "native-tpm-crb")]
    super::tpm_fixture::manifest(state);
    let e = m4::engine();
    let pids = [state.processes[0].pid, state.processes[1].pid];
    for pid in pids {
        e.register(pid).unwrap_or_else(|_| fail(b"REGISTER"));
    }
    let roots = [
        e.create_endpoint(pids[0]).unwrap_or_else(|_| fail(b"ROOT")),
        e.create_endpoint(pids[1]).unwrap_or_else(|_| fail(b"ROOT")),
    ];
    for slot in 0..2 {
        let peer = 1 - slot;
        let send = e
            .bootstrap_delegate(pids[peer], roots[peer], pids[slot], SEND)
            .unwrap_or_else(|_| fail(b"PEER_SEND"));
        let own_send = e
            .derive(pids[slot], roots[slot], SEND)
            .unwrap_or_else(|_| fail(b"OWN_SEND"));
        put(state, slot, 32, slot as u64);
        put(state, slot, 56, roots[slot]);
        put(state, slot, 64, send);
        put(state, slot, 72, own_send);
        put(state, slot, 176, pids[peer]);
        state.audit_process(slot);
    }
    if !e.check_invariants() || e.stats().queued_messages != 0 {
        fail(b"MANIFEST_ENGINE");
    }
    state.free_baseline = state
        .free_count()
        .checked_add(releasable_frames(state))
        .unwrap_or_else(|| fail(b"FRAME_OVERFLOW"));
    *evidence() = Evidence {
        exited: [false; 2],
        assertions: [0; 2],
        ticks: [0; 2],
        masks: [0; 2],
    };
    serial::write(b"IPCWAIT:MANIFEST:PASS\n");
}
pub fn observe_exit(state: &mut State, slot: usize, _counter: u64) {
    if slot >= 2 {
        fail(b"EXIT_SLOT");
    }
    if evidence().exited[slot] || state.last_event.code != 0 {
        print_number(b"IPCWAIT:EXIT_SLOT:", slot as u64);
        print_number(b"IPCWAIT:EXIT_CODE:", state.last_event.code);
        print_number(b"IPCWAIT:APP_FAILURE:", word(state, slot, 416));
        print_number(b"IPCWAIT:APP_PHASE:", word(state, slot, 448));
        fail(b"EXIT");
    }
    if word(state, slot, 400) != 1
        || word(state, slot, 408) == 0
        || word(state, slot, 416) != 0
        || word(state, slot, 432) != [0x3f, 0x3c][slot]
        || word(state, slot, 448) != 7
        || word(state, slot, 480) == 0
    {
        fail(b"APP_EVIDENCE");
    }
    if slot == 0 {
        if word(state, slot, 464) == 0
            || word(state, slot, 472) == 0
            || word(state, slot, 488) == 0
            || state.processes[1].status == Status::Dead
            || m4::engine().stats().queued_messages != 16
        {
            fail(b"QUEUE_WAKE_EVIDENCE");
        }
        serial::write(b"IPCWAIT:REVOKED_FULL_QUEUE:PASS\n");
    } else if word(state, slot, 496) != 17 || word(state, slot, 504) != 16 {
        fail(b"FIFO_OR_REVOKE");
    }
    let e = evidence();
    e.exited[slot] = true;
    e.assertions[slot] = word(state, slot, 408);
    e.ticks[slot] = word(state, slot, 424);
    e.masks[slot] = word(state, slot, 432);
    state.audit_process(slot);
}
pub fn observe_fault(_state: &mut State, _slot: usize, _context: &[u64; 22], _counter: u64) {
    fail(b"FAULT")
}
pub fn poll(state: &mut State) -> Option<*const u64> {
    if state.ticks >= 2000 || state.fault || state.unexpected {
        fail(b"TIMEOUT_OR_FAULT");
    }
    let e = *evidence();
    if !e.exited.iter().all(|exited| *exited) {
        return None;
    }
    let stats = m4::engine().stats();
    if state.processes[..2]
        .iter()
        .any(|p| p.status != Status::Dead)
        || state.free_count() != state.free_baseline
        || stats.processes != 0
        || stats.handles != 0
        || stats.nodes != 0
        || stats.endpoints != 0
        || stats.queued_messages != 0
        || !m4::engine().check_invariants()
    {
        fail(b"CLEANUP");
    }
    let m = state.wait_metrics;
    if m.blocked == 0
        || m.wake_ready == 0
        || m.wake_timeout == 0
        || m.wake_error == 0
        || m.idle_irqs == 0
    {
        fail(b"NO_NATIVE_WAIT");
    }
    #[cfg(feature = "native-tpm-crb")]
    super::tpm_fixture::finish();
    for slot in 0..2 {
        print_number(b"IPCWAIT:ASSERTIONS:", e.assertions[slot]);
        print_number(b"IPCWAIT:TICKS:", e.ticks[slot]);
        print_number(b"IPCWAIT:PHASE_MASK:", e.masks[slot]);
    }
    print_number(b"IPCWAIT:BLOCKED:", m.blocked);
    print_number(b"IPCWAIT:WAKE_READY:", m.wake_ready);
    print_number(b"IPCWAIT:WAKE_TIMEOUT:", m.wake_timeout);
    print_number(b"IPCWAIT:WAKE_ERROR:", m.wake_error);
    print_number(b"IPCWAIT:IDLE_IRQS:", m.idle_irqs);
    serial::write(b"IPCWAIT:NATIVE:PASS\nIPCWAIT:CLEANUP:PASS\nIPCWAIT:PASS\nM1:PASS\n");
    serial::exit(0x10)
}
