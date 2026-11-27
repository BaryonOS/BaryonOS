//! L01 native SSE2 AI job observer (single CPU; no sleep/wake or SMP claim).
//!
//! Slot 0 runs the capability-free AI job twice (fresh PID on reuse); slot 1 is
//! the `fpu-b` register canary. The observer only reads report words, compares
//! run 1 with run 2 and checks the existing eager FXSAVE64/FXRSTOR64 context
//! accounting. It grants no rights, adds no FPU mechanism and uses no FP itself.
use super::{fpu, print_number, read_word, write_word, State, Status, DATA};
use crate::serial;
use core::ptr;

const STATUS: u64 = 64;
const ITERATIONS: u64 = 72;
const START_TICKS: u64 = 80;
const END_TICKS: u64 = 88;
const MISMATCHES: u64 = 96;
const MACS: u64 = 104;
const ARGMAX: u64 = 112;
const DIGEST: u64 = 120;
const BITS: u64 = 128;
const WORDS: usize = 32;
const MIN_ITERATIONS: u64 = 2000;
const MIN_TICKS: u64 = 50;
/// Timer IRQs that must land on the job itself in each run; the job also
/// yields every 64 iterations, so its self-reported duration alone does not
/// prove preemption. Measured passes see 20-81 per run locally (TCG, idle and
/// under host load) and 8-65 on hosted TCG runners (minimum 8 on 93b6a23e; the
/// yield cadence decides how many ticks land on the job rather than the canary),
/// so the bound is a small absolute count with a 2x margin, not a ratio of `MIN_TICKS`.
const MIN_JOB_TIMER_IRQS: u64 = 4;
/// 100 Hz PIT ticks (20 s); a measured pass needs about 110. The host
/// runner's 60 s timeout is sized above this bound.
const MAX_TICKS: u64 = 2000;

struct Evidence {
    baseline: usize,
    runs: u64,
    reuse: u64,
    canary_exits: u64,
    timer: [u64; 2],
    /// `timer[0]` and kernel ticks when the current job lifetime was spawned.
    timer_mark: u64,
    spawn_ticks: u64,
    first_b_loops: u64,
    macs: u64,
    argmax: u64,
    digest: u64,
    bits: [u64; WORDS],
}
static mut EVIDENCE: Evidence = Evidence {
    baseline: 0,
    runs: 0,
    reuse: 0,
    canary_exits: 0,
    timer: [0; 2],
    timer_mark: 0,
    spawn_ticks: 0,
    first_b_loops: 0,
    macs: 0,
    argmax: 0,
    digest: 0,
    bits: [0; WORDS],
};
fn evidence() -> &'static mut Evidence {
    // SAFETY: This observer runs only under the KERNEL-held IF=0 event loop on
    // the single CPU; no other reference to EVIDENCE outlives one hook call.
    unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) }
}
fn require(ok: bool, why: &[u8]) {
    if !ok {
        serial::write(b"AIJOB:FAIL:");
        serial::write(why);
        serial::write(b"\n");
        serial::exit(0x11);
    }
}
fn word(s: &State, slot: usize, offset: u64) -> u64 {
    read_word(s.processes[slot].physical(DATA + offset))
}

pub(super) fn manifest(s: &State) {
    require(
        s.amount == 2 && s.processes[0].pid == 1 && s.processes[1].pid == 2,
        b"MANIFEST",
    );
    // Same independent frame account as the FPU fixture.
    evidence().spawn_ticks = s.ticks;
    evidence().baseline = s.free_count()
        + s.processes[..2]
            .iter()
            .map(|p| {
                p.count + p.kernel_frames.len() + p.paging.as_ref().unwrap().owned_table_count()
            })
            .sum::<usize>();
    // Only the canary gets its identity word; DATA+64 is the job's own STATUS.
    write_word(s.processes[1].physical(DATA + 64), 1);
    serial::write(b"AIJOB:MANIFEST:PASS\n");
}

pub(super) fn entry(slot: usize, vector: u64) {
    if vector == 32 {
        evidence().timer[slot] += 1;
    }
}

/// The `fpu-b` canary's own report checks, identical to `fpu::fixture::collect`.
fn collect_canary(s: &State) {
    require(
        word(s, 1, 80) == 1 && word(s, 1, 88) >= 8,
        b"CANARY_STATE_CHECKS",
    );
    require(
        word(s, 1, 96)
            .checked_add(12)
            .is_some_and(|floor| word(s, 1, 104) >= floor)
            && word(s, 1, 112) >= 2,
        b"CANARY_CLOCK_PROGRESS",
    );
    require(
        word(s, 1, 120) >= 8 && word(s, 1, 128) == 6,
        b"CANARY_CLOCK_ARGUMENTS",
    );
    require(
        word(s, 1, 136) == 1 && word(s, 1, 144) == 1,
        b"CANARY_COMPLETION",
    );
}

fn print_run(s: &State, run: u64, job_irqs: u64) {
    let digit = b'0' + run as u8;
    let mut label = *b"AIJOB:RUN0:ITERATIONS:";
    label[9] = digit;
    print_number(&label, word(s, 0, ITERATIONS));
    let mut label = *b"AIJOB:RUN0:START_TICKS:";
    label[9] = digit;
    print_number(&label, word(s, 0, START_TICKS));
    let mut label = *b"AIJOB:RUN0:END_TICKS:";
    label[9] = digit;
    print_number(&label, word(s, 0, END_TICKS));
    let mut label = *b"AIJOB:RUN0:TIMER_IRQS:";
    label[9] = digit;
    print_number(&label, job_irqs);
    let mut label = *b"AIJOB:RUN0:MACS:";
    label[9] = digit;
    print_number(&label, word(s, 0, MACS));
    let mut label = *b"AIJOB:RUN0:DIGEST:";
    label[9] = digit;
    print_number(&label, word(s, 0, DIGEST));
    let mut label = *b"AIJOB:RUN0:ARGMAX:";
    label[9] = digit;
    print_number(&label, word(s, 0, ARGMAX));
    for i in 0..WORDS {
        let mut label = *b"AIJOB:RUN0:BITS00:";
        label[9] = digit;
        label[15] = b'0' + (i / 10) as u8;
        label[16] = b'0' + (i % 10) as u8;
        print_number(&label, word(s, 0, BITS + 8 * i as u64));
    }
}

pub(super) fn observe_exit(s: &State, slot: usize, _: u64) {
    if s.last_event.code != 0 {
        print_number(b"AIJOB:APP_FAILURE_SLOT:", slot as u64);
        print_number(b"AIJOB:APP_FAILURE_CODE:", s.last_event.code);
    }
    require(s.last_event.code == 0, b"APP_EXIT");
    let e = evidence();
    if slot == 1 {
        require(e.runs == 2 && e.canary_exits == 0, b"EXIT_ORDER");
        collect_canary(s);
        e.canary_exits = 1;
        return;
    }
    require(
        slot == 0 && e.runs == e.reuse && e.runs < 2 && e.canary_exits == 0,
        b"EXIT_ORDER",
    );
    require(
        s.processes[0].pid == if e.runs == 0 { 1 } else { 3 },
        b"RUN_IDENTITY",
    );
    let (start, end) = (word(s, 0, START_TICKS), word(s, 0, END_TICKS));
    require(word(s, 0, STATUS) == 1, b"JOB_STATUS");
    require(word(s, 0, MISMATCHES) == 0, b"JOB_MISMATCHES");
    require(word(s, 0, ITERATIONS) >= MIN_ITERATIONS, b"JOB_ITERATIONS");
    require(
        e.spawn_ticks <= start && start <= end && end <= s.ticks && end - start >= MIN_TICKS,
        b"JOB_PREEMPTED_DURATION",
    );
    let job_irqs = e.timer[0] - e.timer_mark;
    if job_irqs < MIN_JOB_TIMER_IRQS {
        // Diagnostic only: shows whether ticks went to the canary or were coalesced.
        print_number(b"AIJOB:DIAG:JOB_IRQS:", job_irqs);
        print_number(b"AIJOB:DIAG:CANARY_IRQS:", e.timer[1]);
        print_number(b"AIJOB:DIAG:JOB_TICKS:", end - start);
        print_number(b"AIJOB:DIAG:LIFETIME_TICKS:", s.ticks - e.spawn_ticks);
    }
    require(job_irqs >= MIN_JOB_TIMER_IRQS, b"JOB_TIMER_PREEMPTION");
    if e.runs == 0 {
        e.macs = word(s, 0, MACS);
        e.argmax = word(s, 0, ARGMAX);
        e.digest = word(s, 0, DIGEST);
        for (i, bits) in e.bits.iter_mut().enumerate() {
            *bits = word(s, 0, BITS + 8 * i as u64);
        }
        e.first_b_loops = word(s, 1, 88);
    } else {
        require(
            e.macs == word(s, 0, MACS)
                && e.argmax == word(s, 0, ARGMAX)
                && e.digest == word(s, 0, DIGEST)
                && e.bits
                    .iter()
                    .enumerate()
                    .all(|(i, bits)| *bits == word(s, 0, BITS + 8 * i as u64)),
            b"RUN_BITS_DIFFER",
        );
        // Release the canary only after both AI lifetimes completed.
        write_word(s.processes[1].physical(DATA + 160), 1);
    }
    print_run(s, e.runs + 1, job_irqs);
    e.runs += 1;
}

pub(super) fn observe_fault(_: &State, slot: usize, _: &[u64; 22], _: u64) {
    print_number(b"AIJOB:FAULT_SLOT:", slot as u64);
    require(false, b"FAULT");
}

pub(super) fn poll(s: &mut State) {
    require(
        s.ticks < MAX_TICKS && s.syscall_calls < 2_000_000,
        b"BOUNDED_RUN",
    );
    // A process killed for invalid saved state reaches no observer hook.
    require(!s.fault && !s.unexpected, b"PROCESS_FAULT");
    // A failed job publishes 0x100+code and spins on yield instead of exiting.
    if s.processes[0].status != Status::Dead {
        let status = word(s, 0, STATUS);
        if status >= 0x100 {
            print_number(b"AIJOB:JOB_FAILURE_CODE:", status - 0x100);
            require(false, b"JOB_REPORTED_FAILURE");
        }
    }
    let e = evidence();
    if e.runs == 1 && e.reuse == 0 && s.processes[0].status == Status::Dead {
        require(s.processes[1].status == Status::Ready, b"SURVIVOR");
        if e.first_b_loops
            .checked_add(8)
            .is_some_and(|floor| word(s, 1, 88) > floor)
        {
            e.timer_mark = e.timer[0];
            e.spawn_ticks = s.ticks;
            s.spawn_process(0);
            require(s.processes[0].pid == 3, b"FRESH_PID");
            e.reuse = 1;
            serial::write(b"AIJOB:SURVIVOR:PASS\nAIJOB:SLOT_REUSE:PASS\n");
        }
    }
    if s.processes[..2].iter().all(|p| p.status == Status::Dead) {
        require(
            e.runs == 2 && e.reuse == 1 && e.canary_exits == 1,
            b"LIFECYCLE",
        );
        require(e.timer.iter().all(|x| *x >= 2), b"TIMER_PREEMPTION");
        require(s.free_count() == e.baseline, b"FRAME_RECOVERY");
        require(fpu::images_wiped(), b"CONTEXT_WIPE");
        let (saves, restores) = fpu::counts();
        let user_timer_irqs = e.timer[0] + e.timer[1];
        for (name, value) in [
            (b"AIJOB:RUNS:".as_slice(), e.runs),
            (b"AIJOB:SLOT_REUSES:", e.reuse),
            (b"AIJOB:TIMER_A:", e.timer[0]),
            (b"AIJOB:TIMER_B:", e.timer[1]),
            (b"AIJOB:TIMER_IRQS:", s.timer_interrupts),
            (b"AIJOB:SYSCALL_CALLS:", s.syscall_calls),
            (b"AIJOB:SAVES:", saves),
            (b"AIJOB:RESTORES:", restores),
            (b"AIJOB:BASELINE_FRAMES:", e.baseline as u64),
            (b"AIJOB:FREE_FRAMES:", s.free_count() as u64),
            (b"AIJOB:CONTEXT_BYTES_WIPED:", 8192),
        ] {
            print_number(name, value);
        }
        // No CPL3 fault occurs here, so every save is a user IRQ or syscall entry
        // (the FPU fixture's `+ 1` is its one real #UD entry).
        require(
            s.timer_interrupts == user_timer_irqs
                && saves == user_timer_irqs + s.syscall_calls
                && restores == saves,
            b"ENTRY_SWITCH_COUNTS",
        );
        serial::write(b"AIJOB:CONTEXT_ISOLATION:PASS\nAIJOB:CLEANUP:PASS\nAIJOB:PASS\nM1:PASS\n");
        serial::exit(0x10)
    }
}
