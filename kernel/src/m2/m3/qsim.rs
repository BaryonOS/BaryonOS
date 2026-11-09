//! Opt-in I02 observer for the fixed ring-3 qsim acceptance executable.
//! No simulator algorithm, app/job policy, allocator, or new authority lives
//! here. All computation remains in two independent userspace processes.
//! KERNEL/IF=0, kernel CR3/service stack: copy only bounded owned words.
//! The SMP profile parks all CPUs before accepting completed reports or freeing
//! frames; its in-flight samples never constitute a completion verdict.
use super::*;

const PROCESS_COUNT: usize = if cfg!(feature = "native-smp-qsim") {
    4
} else {
    2
};
const REPORT_WORDS: usize = 14;
const TIMEOUT_TICKS: u64 = 2000;
const USEFUL_ARENA_BYTES: u64 = 7216;

fn fail(reason: &[u8]) -> ! {
    serial::write(b"QSIM:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}

fn snapshot(process: &Process) -> [u64; REPORT_WORDS] {
    // These 112 bytes are within the kernel-validated private RW report page.
    // The event path has switched to the permanent kernel root and stack;
    // completed SMP reports are validated again after all CPUs have parked.
    core::array::from_fn(|word| read_word(process.physical(DATA + word as u64 * 8)))
}

fn releasable_frames(state: &State) -> usize {
    let mut total = 0usize;
    for process in &state.processes[..state.amount] {
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

pub fn manifest(state: &mut State) {
    if state.amount != PROCESS_COUNT || state.fault || state.exited || state.unexpected {
        fail(b"MANIFEST_STATE");
    }
    #[cfg(feature = "native-smp-qsim")]
    assert_eq!(smp::online_mask(), 0b1111);
    for process in &state.processes[..state.amount] {
        if process.status != Status::Ready
            || process.pid == 0
            || !process
                .paging
                .as_ref()
                .is_some_and(|paging| paging.permissions(DATA).is_some_and(|p| p.1 && !p.2 && p.3))
        {
            fail(b"MANIFEST_PROCESS");
        }
        let report = snapshot(process);
        if report[0] != 0
            || report[1] != 0
            || report[2] != process.pattern
            || report[3..].iter().any(|word| *word != 0)
        {
            fail(b"MANIFEST_REPORT");
        }
    }
    for left in 0..PROCESS_COUNT {
        for right in left + 1..PROCESS_COUNT {
            let first = &state.processes[left];
            let second = &state.processes[right];
            if first.pid == second.pid
                || first.pattern == second.pattern
                || first.paging.as_ref().unwrap().root == second.paging.as_ref().unwrap().root
                || first.physical(DATA) == second.physical(DATA)
                || first.physical(HEAP) == second.physical(HEAP)
                || first.physical(HEAP + 65536 - 8) == second.physical(HEAP + 65536 - 8)
            {
                fail(b"MANIFEST_SHARED_MEMORY");
            }
        }
    }
    state.free_baseline = state
        .free_count()
        .checked_add(releasable_frames(state))
        .unwrap_or_else(|| fail(b"FRAME_BASELINE_OVERFLOW"));
    serial::write(b"QSIM:MANIFEST:PASS\n");
}

fn valid_completed_report(report: &[u64; REPORT_WORDS], pattern: u64) -> bool {
    report[0] == 0
        && report[1] > 0
        && report[2] == pattern
        && report[3] == 0
        && report[4] == 1
        && report[5] == 127
        && report[6] == 2082
        && report[7] == 2014
        && report[8] == 4096
        && report[9] == USEFUL_ARENA_BYTES
        && report[10] == 448
        && report[11] == 4096
        && report[12] == 65536
        && report[13] == 1
}

pub fn poll(state: &mut State) -> Option<*const u64> {
    if state.amount != PROCESS_COUNT || state.fault || state.exited || state.unexpected {
        print_number(b"QSIM:LAST_EVENT_PID:", state.last_event.pid);
        print_number(b"QSIM:LAST_EVENT_VECTOR:", state.last_event.vector);
        print_number(b"QSIM:LAST_EVENT_CODE:", state.last_event.code);
        fail(b"UNEXPECTED_USER_EVENT");
    }
    if state.ticks >= TIMEOUT_TICKS {
        fail(b"TIMEOUT");
    }
    let mut reports = [[0; REPORT_WORDS]; PROCESS_COUNT];
    let mut complete = true;
    for (slot, process) in state.processes[..state.amount].iter().enumerate() {
        if !matches!(process.status, Status::Ready | Status::Running) {
            fail(b"PROCESS_NOT_LIVE");
        }
        let report = snapshot(process);
        if report[0] != 0 || report[2] != process.pattern || report[3] != 0 {
            fail(b"PRIVATE_PATTERN");
        }
        if report[4] > 1 {
            print_number(b"QSIM:FAILED_PID:", process.pid);
            print_number(b"QSIM:FAILED_REPORT_STATUS:", report[4]);
            fail(b"APP_REPORT");
        }
        if report[4] == 1 {
            #[cfg(not(feature = "native-smp-qsim"))]
            if !valid_completed_report(&report, process.pattern) || process.selected == 0 {
                fail(b"RESULT_MISMATCH");
            }
        } else {
            complete = false;
        }
        reports[slot] = report;
    }
    #[cfg(feature = "native-smp-qsim")]
    {
        complete &= (0..4).all(|cpu| state.cpu_ticks[cpu] > 0 && state.cpu_switches[cpu] >= 1);
    }
    if !complete || state.timer_interrupts == 0 {
        // Always bypass the ordinary M3 fixture verdict for this opt-in image.
        // The returned pointer belongs to the selected kernel-owned context;
        // no reference into the copied report survives this scheduling point.
        #[cfg(not(feature = "native-smp-qsim"))]
        return Some(state.select());
        #[cfg(feature = "native-smp-qsim")]
        return None;
    }

    // The stop releases KERNEL while the other CPUs leave their user contexts;
    // no State or userspace borrow is retained across it. Completed status is
    // only a readiness hint until these stable, parked snapshots are checked.
    #[cfg(feature = "native-smp-qsim")]
    let state = super::smp_stop_for_cleanup();
    #[cfg(feature = "native-smp-qsim")]
    for (slot, process) in state.processes[..state.amount].iter().enumerate() {
        reports[slot] = snapshot(process);
        if !valid_completed_report(&reports[slot], process.pattern) || process.selected == 0 {
            fail(b"RESULT_MISMATCH");
        }
    }

    state.audit();
    let released = releasable_frames(state);
    let free_before = state.free_count();
    if free_before.checked_add(released) != Some(state.free_baseline) {
        fail(b"FRAME_BASELINE_CHANGED");
    }
    for (slot, report) in reports.iter().enumerate() {
        print_number(b"QSIM:PROCESS_PID:", state.processes[slot].pid);
        print_number(b"QSIM:PROCESS_CHECKPOINTS:", report[1]);
        print_number(b"QSIM:PROCESS_USEFUL_ARENA:", report[9]);
        print_number(b"QSIM:PROCESS_MAX_WORK_GAP:", report[10]);
    }
    serial::write(b"QSIM:BELL4096:PASS\nQSIM:SIX_QUBIT:PASS\nQSIM:CANCEL:PASS\n");
    serial::write(b"QSIM:DEADLINE:PASS\nQSIM:RESOURCE_LIMITS:PASS\nQSIM:PROCESS_ISOLATION:PASS\n");
    print_number(b"QSIM:TIMER_IRQS:", state.timer_interrupts);
    print_number(b"QSIM:SYSCALL_CALLS:", state.syscall_calls);

    // The event handler already copied the interrupted frame onto its permanent
    // service stack. Retain that root/stack before freeing either user's tables
    // or per-process kernel stack; cleanup performs per-owner zero/free checks.
    arch::activate(state.kernel.root);
    arch::process_stack(arch::service_stack());
    state.current = MAX;
    for slot in 0..state.amount {
        state.processes[slot].status = Status::Dying;
        state.cleanup(slot);
    }
    if state.free_count() != state.free_baseline
        || state.processes[..state.amount].iter().any(|process| {
            process.status != Status::Dead || process.paging.is_some() || process.count != 0
        })
    {
        fail(b"CLEANUP");
    }
    print_number(b"QSIM:CLEANUP_FRAMES:", released as u64);
    #[cfg(feature = "native-smp-qsim")]
    serial::write(b"SMP:K07:QSIM:PASS:CPUS=4\n");
    serial::write(b"QSIM:CLEANUP:PASS\nQSIM:PASS\nM1:PASS\n");
    serial::exit(0x10)
}
