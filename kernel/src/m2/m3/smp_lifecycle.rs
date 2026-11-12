// SPDX-License-Identifier: MPL-2.0
//! L0 step 5f: retire a real ring-3 process on CPU 3 from the BSP.
//! KERNEL serializes this observer. No frame is freed by the fixture; the
//! ordinary remote-retirement entry and K05 cleanup paths remain the oracle.
use super::*;

const TARGET: usize = 3;
const SURVIVOR: usize = 7;
// The existing process-shootdown/remote-kill acknowledgement budget. A late
// acknowledgement is a failed case, never a reason to enlarge the deadline.
const ACK_LIMIT_MS: u64 = 100;

struct Evidence {
    phase: u8,
    pid: u64,
    requested_ms: u64,
    free_before: usize,
    frames: usize,
    survivor_before: u64,
    reschedules_before: u64,
    entered: bool,
    ipi_entered: bool,
}

static mut EVIDENCE: Evidence = Evidence {
    phase: 0,
    pid: 0,
    requested_ms: 0,
    free_before: 0,
    frames: 0,
    survivor_before: 0,
    reschedules_before: 0,
    entered: false,
    ipi_entered: false,
};

/// Called immediately before the normal cleanup of a Dying process, after the
/// entry stub saved GPR/FPU state and event() left its CR3 and process stack.
pub(super) fn entry(state: &State, slot: usize, vector: u64) {
    assert!(lock::kernel_held());
    if slot != TARGET {
        return;
    }
    // SAFETY: Every access is under KERNEL, IF=0. The static is aligned and
    // lives for the entire boot; no reference escapes this call.
    let evidence = unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) };
    assert_eq!(evidence.phase, 1);
    assert!(!evidence.entered);
    assert_eq!(cpu(), 3);
    assert!(
        matches!(vector, 128 | 0xd0 | 0xe8),
        "kill must not hide a fault"
    );
    assert_eq!(state.processes[slot].pid, evidence.pid);
    assert_eq!(state.processes[slot].active, 0);
    assert!(!ON_CPU[slot].load(Ordering::Acquire));
    assert_eq!(
        SCHED_STATE[slot].load(Ordering::Acquire),
        sched::DEAD_PENDING
    );
    let elapsed = smp::m3_time_ms().saturating_sub(evidence.requested_ms);
    if elapsed >= ACK_LIMIT_MS {
        serial::fail(b"SMP_M3_REMOTE_KILL_ACK_TIMEOUT");
    }
    evidence.entered = true;
    evidence.ipi_entered = vector == 0xe8;
    let mut line = serial::Line::new();
    line.push(b"SMP:M3:REMOTE_KILL:ACK:CPU=3:VECTOR=")
        .decimal(vector)
        .push(b":WAIT_MS=")
        .decimal(elapsed)
        .push(b":LIMIT_MS=")
        .decimal(ACK_LIMIT_MS)
        .push(b"\n");
    serial::write(line.as_bytes());
}

/// Immediately after the ordinary cleanup returns, while the target CPU still
/// holds KERNEL: bind the free-frame delta and deadline to the actual retirement.
pub(super) fn retired(state: &State, slot: usize) {
    assert!(lock::kernel_held());
    if slot != TARGET {
        return;
    }
    // SAFETY: KERNEL/IF=0 gives this static one exclusive borrower, as above.
    let evidence = unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) };
    assert_eq!(evidence.phase, 1);
    assert!(evidence.entered);
    assert_eq!(cpu(), 3);
    assert!(state.processes[TARGET].status == Status::Dead);
    assert_eq!(SCHED_STATE[TARGET].load(Ordering::Acquire), sched::DEAD);
    assert!(!ON_CPU[TARGET].load(Ordering::Acquire));
    assert!(!QUEUED[TARGET].load(Ordering::Acquire));
    assert_eq!(state.processes[TARGET].active, 0);
    assert!(state.processes[TARGET].paging.is_none());
    assert_eq!(state.free_count(), evidence.free_before + evidence.frames);
    if smp::m3_time_ms().saturating_sub(evidence.requested_ms) >= ACK_LIMIT_MS {
        serial::fail(b"SMP_M3_REMOTE_KILL_ACK_TIMEOUT");
    }
    evidence.survivor_before = read_word(state.processes[SURVIVOR].physical(DATA + 8));
    let mut line = serial::Line::new();
    line.push(b"SMP:M3:REMOTE_KILL:CLEANUP:BEFORE=")
        .decimal(evidence.free_before as u64)
        .push(b":RELEASED=")
        .decimal(evidence.frames as u64)
        .push(b":AFTER=")
        .decimal(state.free_count() as u64)
        .push(b":ACTIVE=0\n");
    serial::write(line.as_bytes());
    evidence.phase = 2;
}

/// Return true only after remote retirement, exact cleanup, IPI delivery and
/// subsequent useful progress of a different process on the target CPU.
pub(super) fn poll(state: &mut State) -> bool {
    assert!(lock::kernel_held());
    // SAFETY: As in entry(), KERNEL gives this static one exclusive borrower.
    // retire_process does not release KERNEL or invoke this observer.
    let evidence = unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) };
    if evidence.phase == 3 {
        return true;
    }
    if cpu() != 0 {
        return false;
    }
    assert_eq!(smp::online_mask(), 0b1111);
    assert!(state.amount > SURVIVOR);
    if evidence.phase == 0 {
        let target = &state.processes[TARGET];
        if target.status != Status::Running
            || target.active != 1 << 3
            || !ON_CPU[TARGET].load(Ordering::Acquire)
            || state.cpu_ticks[3] == 0
            || read_word(target.physical(DATA + 8)) == 0
        {
            return false;
        }
        assert_eq!(target.home, 3);
        assert_eq!(
            SCHED_STATE[TARGET].load(Ordering::Acquire),
            sched::running(3)
        );
        assert!(!QUEUED[TARGET].load(Ordering::Acquire));
        assert_eq!(state.processes[SURVIVOR].home, 3);
        evidence.pid = target.pid;
        evidence.frames = target.count
            + target.kernel_frames.len()
            + target.paging.as_ref().unwrap().owned_table_count();
        evidence.free_before = state.free_count();
        evidence.reschedules_before = smp::m3_received_reschedules();
        evidence.requested_ms = smp::m3_time_ms();
        evidence.phase = 1;
        state.retire_process(TARGET, None);
        assert!(state.processes[TARGET].status == Status::Dying);
        assert_eq!(
            SCHED_STATE[TARGET].load(Ordering::Acquire),
            sched::DEAD_PENDING
        );
        serial::write(b"SMP:M3:REMOTE_KILL:REQUEST:FROM=0:TO=3\n");
        return false;
    }
    if evidence.phase == 1 {
        assert!(state.processes[TARGET].status == Status::Dying);
        if smp::m3_time_ms().saturating_sub(evidence.requested_ms) >= ACK_LIMIT_MS {
            serial::fail(b"SMP_M3_REMOTE_KILL_ACK_TIMEOUT");
        }
        return false;
    }
    if read_word(state.processes[SURVIVOR].physical(DATA + 8)) <= evidence.survivor_before + 8
        || (!evidence.ipi_entered && smp::m3_received_reschedules() <= evidence.reschedules_before)
    {
        return false;
    }
    evidence.phase = 3;
    serial::write(b"SMP:M3:REMOTE_KILL:PASS:CPU=3\n");
    true
}
