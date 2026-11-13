//! Isolated ring-3 display/input boundary acceptance observer. User processes
//! perform every hardware syscall; this module grants the fixed manifest and
//! observes real syscall counters/lifecycles without writing display/PS2 ports.
use super::*;
#[path = "../../../../sdk/examples/display-input-probe/src/contract.rs"]
#[allow(dead_code)]
mod contract;
use contract::*;

struct Evidence {
    baseline: usize,
    kernel_tables: usize,
    generation: u64,
    display_marker: bool,
    input_marker: bool,
    exiting: bool,
    exits: u64,
    reuse: u64,
    blits: u64,
    display_denied: u64,
    resets: u64,
    ready: u64,
    samples: u64,
    pointers: u64,
    caps: u64,
    unchanged: u64,
    closed: u64,
    pending: u64,
    old_denied: u64,
    display_epoch: u64,
    input_epoch: u64,
    input_ticket: u64,
    old_sources: [u64; 2],
}
static mut EVIDENCE: Evidence = Evidence {
    baseline: 0,
    kernel_tables: 0,
    generation: 1,
    display_marker: false,
    input_marker: false,
    exiting: false,
    exits: 0,
    reuse: 0,
    blits: 0,
    display_denied: 0,
    resets: 0,
    ready: 0,
    samples: 0,
    pointers: 0,
    caps: 0,
    unchanged: 0,
    closed: 0,
    pending: 0,
    old_denied: 0,
    display_epoch: 0,
    input_epoch: 0,
    input_ticket: 0,
    old_sources: [0; 2],
};
fn evidence() -> &'static mut Evidence {
    // SAFETY: KERNEL held, IF0; sequential fixture callbacks. No borrow survives a
    // scheduler/cleanup call or another call to this accessor.
    unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) }
}
fn require(ok: bool, why: &[u8]) {
    if !ok {
        serial::write(b"DISPINPUT:FAIL:");
        serial::write(why);
        serial::write(b"\n");
        serial::exit(0x11);
    }
}
fn word(s: &State, slot: usize, offset: u64) -> u64 {
    read_word(s.processes[slot].physical(DATA + offset))
}
fn put(s: &State, slot: usize, offset: u64, value: u64) {
    write_word(s.processes[slot].physical(DATA + offset), value);
}
fn configure(s: &State, generation: u64, old_sources: [u64; 2]) {
    let mut sources = [0u64; 2];
    for slot in 0..3 {
        let pid = s.processes[slot].pid;
        m4::engine().register(pid).expect("fixture fresh PID");
        let wrong = m4::engine().create_endpoint(pid).expect("fixture endpoint");
        put(s, slot, WRONG_TYPE, wrong);
        if slot == 1 {
            // Source handles deliberately occupy different slots. Copying the
            // peer integer cannot accidentally name the caller's own source.
            m4::engine()
                .create_endpoint(pid)
                .expect("fixture second endpoint");
        }
        if slot < 2 {
            sources[slot] = if slot == 0 {
                m4::engine()
                    .create_display(pid)
                    .expect("sole display owner")
            } else {
                m4::engine().create_input(pid).expect("sole input owner")
            };
            put(s, slot, SOURCE, sources[slot]);
            put(s, slot, OLD_SOURCE, old_sources[slot]);
        }
        put(s, slot, GENERATION, generation);
    }
    for slot in 0..3 {
        put(s, slot, PEER_DISPLAY, sources[0]);
        put(s, slot, PEER_INPUT, sources[1]);
    }
    let stats = m4::engine().stats();
    require(
        stats.processes == 3
            && stats.handles == 6
            && stats.nodes == 6
            && stats.endpoints == 4
            && stats.open_endpoints == 4
            && stats.queued_messages == 0
            && stats.display_objects == 1
            && stats.input_objects == 1
            && m4::engine().check_invariants(),
        b"MANIFEST_AUTHORITY",
    );
    evidence().old_sources = sources;
}
pub(super) fn manifest(s: &State) {
    require(s.amount == 3 && s.next_pid == 4, b"THREE_NATIVE_ELFS");
    let baseline = s.free_count()
        + s.processes[..3]
            .iter()
            .map(|p| {
                p.count + p.kernel_frames.len() + p.paging.as_ref().unwrap().owned_table_count()
            })
            .sum::<usize>();
    let e = evidence();
    e.baseline = baseline;
    e.kernel_tables = s.kernel.owned_table_count();
    configure(s, 1, [0; 2]);
    serial::write(b"DISPINPUT:MANIFEST:PASS\n");
}
pub(super) fn observe_fault(_: &State, slot: usize, _: &[u64; 22], _: u64) {
    print_number(b"DISPINPUT:FAULT_SLOT:", slot as u64);
    require(false, b"UNEXPECTED_NATIVE_FAULT");
}
pub(super) fn observe_exit(s: &State, slot: usize, _: u64) {
    if s.last_event.code != 0 {
        print_number(b"DISPINPUT:FAILURE_SLOT:", slot as u64);
        print_number(b"DISPINPUT:FAILURE_STEP:", word(s, slot, FAILURE));
    }
    require(
        s.last_event.code == 0 && word(s, slot, DONE) == 1,
        b"ACTUAL_APP_EXIT",
    );
    let generation = word(s, slot, GENERATION);
    require(generation == evidence().generation, b"LIFETIME_GENERATION");
    require(
        word(s, slot, CLOSED) == u64::from(slot < 2),
        b"CLOSED_SOURCE_REFUSAL",
    );
    require(
        word(s, slot, OLD_DENIED) == u64::from(generation == 2 && slot < 2),
        b"OLD_HANDLE_REFUSAL",
    );
    match slot {
        0 => {
            require(
                word(s, slot, BLITS) == 1
                    && word(s, slot, DISPLAY_DENIED) == 11
                    && word(s, slot, CAP_DENIED) == 6
                    && word(s, slot, UNCHANGED) == 3 + generation,
                b"DISPLAY_BOUNDARY_COUNTERS",
            );
            require(
                word(s, slot, WIDTH) == DEV_WIDTH
                    && word(s, slot, HEIGHT) == DEV_HEIGHT
                    && word(s, slot, EPOCH) > evidence().display_epoch,
                b"ACTUAL_GEOMETRY_NEW_EPOCH",
            );
            evidence().display_epoch = word(s, slot, EPOCH);
        }
        1 => {
            require(
                word(s, slot, RESETS) == 1
                    && word(s, slot, READY) == 1
                    && word(s, slot, SAMPLES) == 8
                    && word(s, slot, CAP_DENIED) == 10
                    && word(s, slot, POINTER_DENIED) >= 9
                    && word(s, slot, UNCHANGED) == word(s, slot, POINTER_DENIED) + 5
                    && word(s, slot, PENDING) > 0,
                b"INPUT_BOUNDARY_COUNTERS",
            );
            require(
                word(s, slot, EPOCH) > evidence().input_epoch
                    && word(s, slot, TICKET) > evidence().input_ticket
                    && (1..=8192).contains(&word(s, slot, RESET_OPERATIONS))
                    && word(s, slot, RESET_DISCARDED) <= word(s, slot, RESET_OPERATIONS),
                b"ACTUAL_RESET_NEW_EPOCH",
            );
            for (index, expected) in INPUT_BYTES.into_iter().enumerate() {
                let sequence = word(s, slot, LOG_SEQUENCE + index as u64 * 8);
                let sample = word(s, slot, LOG + index as u64 * 8);
                let status = (sample >> 8) as u8;
                require(
                    sequence == index as u64 + 1
                        && sample <= 0xffff
                        && sample as u8 == expected
                        && status & 0xc1 == 1
                        && (status & 0x20 != 0) == (index >= 2),
                    b"ACTUAL_RAW_TRANSCRIPT",
                );
                print_number(b"DISPINPUT:SEQ:", sequence);
                print_number(b"DISPINPUT:RAW:", sample);
            }
            evidence().input_epoch = word(s, slot, EPOCH);
            evidence().input_ticket = word(s, slot, TICKET);
        }
        2 => require(
            word(s, slot, CAP_DENIED) == 6 && word(s, slot, UNCHANGED) == 4,
            b"UNPRIVILEGED_REFUSALS",
        ),
        _ => require(false, b"NO_EXTRA_PROCESS"),
    }
    let e = evidence();
    e.exits += 1;
    for (offset, total) in [
        (BLITS, &mut e.blits),
        (DISPLAY_DENIED, &mut e.display_denied),
        (RESETS, &mut e.resets),
        (READY, &mut e.ready),
        (SAMPLES, &mut e.samples),
        (POINTER_DENIED, &mut e.pointers),
        (CAP_DENIED, &mut e.caps),
        (UNCHANGED, &mut e.unchanged),
        (CLOSED, &mut e.closed),
        (PENDING, &mut e.pending),
        (OLD_DENIED, &mut e.old_denied),
    ] {
        *total += word(s, slot, offset);
    }
}
fn clean(s: &State) {
    let stats = m4::engine().stats();
    require(
        stats.processes == 0
            && stats.handles == 0
            && stats.nodes == 0
            && stats.endpoints == 0
            && stats.queued_messages == 0
            && stats.display_objects == 0
            && stats.input_objects == 0
            && m4::engine().check_invariants(),
        b"AUTHORITY_CLEANUP",
    );
    let e = evidence();
    let retained = s
        .kernel
        .owned_table_count()
        .checked_sub(e.kernel_tables)
        .unwrap();
    require(
        s.free_count() + retained == e.baseline,
        b"PHYSICAL_FRAME_CLEANUP",
    );
}
pub(super) fn poll(s: &mut State) {
    require(
        s.ticks < 10000 && s.syscall_calls < 2_000_000,
        b"BOUNDED_RUN",
    );
    // No evidence borrow spans spawn/cleanup/configure calls.
    if !evidence().display_marker && s.processes[0].status != Status::Dead && word(s, 0, STAGE) == 1
    {
        evidence().display_marker = true;
        print_number(b"DISPINPUT:DISPLAY_READY:", evidence().generation);
    }
    if !evidence().input_marker && s.processes[1].status != Status::Dead && word(s, 1, STAGE) == 1 {
        evidence().input_marker = true;
        print_number(b"DISPINPUT:INPUT_READY:", evidence().generation);
    }
    if !evidence().exiting
        && (0..3).all(|slot| s.processes[slot].status != Status::Dead)
        && word(s, 0, STAGE) == 1
        && word(s, 1, STAGE) == 2
        && word(s, 2, STAGE) == 1
    {
        require(
            evidence().display_marker && evidence().input_marker,
            b"HOST_COORDINATION_STAGES",
        );
        evidence().exiting = true;
        for slot in 0..3 {
            put(s, slot, CONTROL, 1);
        }
    }
    if !s.processes[..3].iter().all(|p| p.status == Status::Dead) {
        return;
    }
    clean(s);
    if evidence().generation == 1 {
        let old_sources = evidence().old_sources;
        for slot in 0..3 {
            s.spawn_process(slot);
            require(s.processes[slot].pid == slot as u64 + 4, b"FRESH_PID");
        }
        configure(s, 2, old_sources);
        let e = evidence();
        e.generation = 2;
        e.reuse = 3;
        e.display_marker = false;
        e.input_marker = false;
        e.exiting = false;
        serial::write(b"DISPINPUT:REUSE:PASS\n");
        return;
    }
    let e = evidence();
    require(
        e.exits == 6
            && e.reuse == 3
            && e.blits == 2
            && e.display_denied == 22
            && e.resets == 2
            && e.ready == 2
            && e.samples == 16
            && e.caps == 44
            && e.closed == 4
            && e.old_denied == 2
            && e.pointers >= 18
            && e.unchanged == e.pointers + 27
            && e.pending >= 2,
        b"COMBINED_COUNTERS",
    );
    require(
        display::counts() == [2, 512, 35],
        b"DISPLAY_KERNEL_COUNTERS",
    );
    let input = ps2::counts();
    require(
        input[0] == 2 && input[1] == 2 && input[2] == 16 && input[3] == e.pointers + 17,
        b"INPUT_KERNEL_COUNTERS",
    );
    require(s.timer_interrupts >= 2, b"REAL_PREEMPTION");
    for (name, value) in [
        (b"DISPINPUT:BLITS:".as_slice(), e.blits),
        (b"DISPINPUT:PIXELS:", 512),
        (b"DISPINPUT:DISPLAY_DENIED:", e.display_denied),
        (b"DISPINPUT:CAP_DENIED:", e.caps),
        (b"DISPINPUT:POINTER_DENIED:", e.pointers),
        (b"DISPINPUT:OUTPUT_UNCHANGED:", e.unchanged),
        (b"DISPINPUT:RESETS:", e.resets),
        (b"DISPINPUT:READY:", e.ready),
        (b"DISPINPUT:SAMPLES:", e.samples),
        (b"DISPINPUT:RESET_PENDING:", e.pending),
        (b"DISPINPUT:RESET_DISCARDED:", input[4]),
        (b"DISPINPUT:CLOSED:", e.closed),
        (b"DISPINPUT:OLD_DENIED:", e.old_denied),
        (b"DISPINPUT:EXITS:", e.exits),
        (b"DISPINPUT:SLOT_REUSES:", e.reuse),
        (b"DISPINPUT:DISPLAY_EPOCH:", e.display_epoch),
        (b"DISPINPUT:INPUT_EPOCH:", e.input_epoch),
        (b"DISPINPUT:INPUT_TICKET:", e.input_ticket),
        (b"DISPINPUT:TIMER_IRQS:", s.timer_interrupts),
        (b"DISPINPUT:LIVE_PROCESSES:", 0),
        (b"DISPINPUT:FRAMES_RECOVERED:", 1),
    ] {
        print_number(name, value);
    }
    serial::write(b"DISPINPUT:DISPLAY:PASS\nDISPINPUT:INPUT:PASS\nDISPINPUT:ISOLATION:PASS\nDISPINPUT:CLEANUP:PASS\nM1:PASS\n");
    serial::exit(0x10);
}
