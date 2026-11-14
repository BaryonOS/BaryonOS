// SPDX-License-Identifier: MPL-2.0
//! M3 recovery after a failed kernel-half shootdown. Saved contexts migrate;
//! contexts that did not finish entry are retired with their frames retained.
//! INIT requires the SeqCst admission barrier plus zero entering/lockdep on
//! every former AP. A lock-owner snapshot without that barrier is insufficient.
use super::*;

#[cfg(feature = "test-smp-m3-recovery")]
static mut FIXTURE_STAGE: u8 = 0;
#[cfg(feature = "test-smp-m3-recovery")]
static mut FIXTURE_MIGRATED: u64 = 0;
#[cfg(feature = "test-smp-m3-recovery")]
static mut FIXTURE_PROGRESS: [u64; MAX] = [0; MAX];

/// Exercise the real retirement path rather than replacing its result: CPU 0
/// kills slot 3 while CPU 3 owns it. The profile's post-M3_GO CPU 2 deaf control
/// then withholds a kernel shootdown acknowledgement. Only saved AP contexts
/// that subsequently execute their user canaries on the BSP count as migrated.
#[cfg(feature = "test-smp-m3-recovery")]
pub(super) fn poll(state: &mut State) -> bool {
    assert!(lock::kernel_held());
    // SAFETY: Fixture metadata has one KERNEL owner; plain reads/writes retain
    // no reference across any kernel_leave or user return.
    let stage = unsafe { ptr::addr_of!(FIXTURE_STAGE).read() };
    if stage == 3 {
        return true;
    }
    if cpu() != 0 {
        return false;
    }
    if stage == 0 {
        assert_eq!(smp::online_mask(), 0b1111);
        let target = &state.processes[3];
        if target.status != Status::Running
            || target.active != 1 << 3
            || !ON_CPU[3].load(Ordering::Acquire)
            || state.cpu_ticks[3] == 0
            || read_word(target.physical(DATA + 8)) == 0
        {
            return false;
        }
        assert_eq!(target.home, 3);
        assert_eq!(SCHED_STATE[3].load(Ordering::Acquire), sched::running(3));
        // SAFETY: serialized fixture metadata; publish before the kill request.
        unsafe { ptr::addr_of_mut!(FIXTURE_STAGE).write(1) };
        state.retire_process(3, None);
        serial::write(b"SMP:M3:RECOVERY:REQUEST:FROM=0:TARGET=3\n");
        return false;
    }
    if stage == 1 {
        // A normal successful cleanup would make this negative control vacuous.
        assert!(state.processes[3].status != Status::Dead);
        return false;
    }
    assert_eq!(stage, 2);
    assert_eq!(smp::online_mask(), 1);
    assert!(state.processes[3].quarantined);
    // SAFETY: Copy only; coordinator published the mask under the same lock.
    let migrated = unsafe { ptr::addr_of!(FIXTURE_MIGRATED).read() };
    assert!(migrated != 0);
    for slot in 0..state.amount {
        if migrated & (1 << slot) == 0 {
            continue;
        }
        let process = &state.processes[slot];
        assert!(!process.quarantined && process.home == 0);
        // SAFETY: as above, a bounded read from initialized fixture metadata.
        let before = unsafe { ptr::addr_of!(FIXTURE_PROGRESS[slot]).read() };
        if read_word(process.physical(DATA + 8)) <= before.saturating_add(8) {
            return false;
        }
    }
    // SAFETY: terminal fixture phase under KERNEL; no repeated PASS marker.
    unsafe { ptr::addr_of_mut!(FIXTURE_STAGE).write(3) };
    let mut line = serial::Line::new();
    line.push(b"SMP:M3:RECOVERY:RESUMED=")
        .decimal(u64::from(migrated.count_ones()))
        .push(b":CPU=0\n");
    serial::write(line.as_bytes());
    serial::write(b"SMP:M3:RECOVERY:PASS\n");
    true
}

/// Retained frames belong to the failed retirement, with their mapping arena
/// and page-table ownership intact. They are never reclaimed or scheduled.
/// The caller holds KERNEL and, where required, the process ASPACE lock.
pub(super) fn audit_quarantined(state: &State, slot: usize) -> usize {
    assert!(lock::kernel_held());
    let process = &state.processes[slot];
    assert!(process.quarantined);
    assert!(process.status == Status::Dying);
    assert_eq!(process.map_slot, slot);
    assert!(process.pid != 0);
    assert_eq!(process.active, 0);
    assert!(!ON_CPU[slot].load(Ordering::Acquire));
    assert!(!QUEUED[slot].load(Ordering::Acquire));
    assert_eq!(SCHED_STATE[slot].load(Ordering::Acquire), sched::DEAD);
    let tables = process.paging.as_ref().expect("retained quarantine tables");
    assert!(tables.owned_table_count() > 0);
    let owner = Owner::PageTable(u16::try_from(slot + 1).unwrap());
    assert_eq!(state.frames.owner(tables.root), Ok(owner));
    for mapping in process.mappings() {
        assert_eq!(
            state.frames.owner(mapping.pa),
            Ok(Owner::User(process.pid, mapping.kind))
        );
        // Failed cleanup has already removed these leaves; a context lost at
        // the admission barrier still owns its original mappings. Neither may
        // become runnable or donate any frame to another address space.
        if let Some((physical, writable, executable, user)) = tables.permissions(mapping.va) {
            assert_eq!(physical, mapping.pa);
            assert_eq!(writable, mapping.writable);
            assert_eq!(executable, mapping.executable);
            assert!(user);
        }
    }
    for frame in process.kernel_frames {
        assert_eq!(state.frames.owner(frame), Ok(Owner::Kernel));
    }
    let owned_tables = (0..FRAME_COUNT)
        .filter(|index| state.frames.owner(*index as u64 * PAGE) == Ok(owner))
        .count();
    assert_eq!(owned_tables, tables.owned_table_count());
    process.count + process.kernel_frames.len() + owned_tables
}

/// Called instead of selecting/resuming a thread, under KERNEL with no other
/// guard held, after the old user entry saved its full context and published
/// ON_CPU=false. No State reference may be used after this call by its caller.
pub(super) fn recover() -> *const u64 {
    assert!(lock::kernel_held());
    assert!(smp::m3_recovery_requested());
    let this_cpu = cpu();
    let original_online = smp::online_mask();
    {
        // SAFETY: KERNEL is held. This borrow ends before releasing KERNEL;
        // only initialized static State is accessed from its permanent stack.
        let state = unsafe { (&mut *ptr::addr_of_mut!(STATE)).assume_init_mut() };
        // Entry/idle code must already have left the old process context. Do
        // not manufacture quiescence by clearing an unsaved ON_CPU flag here.
        // SAFETY: CPU_CURRENT is exclusively observed under KERNEL.
        assert_eq!(unsafe { ptr::addr_of!(CPU_CURRENT[this_cpu]).read() }, MAX);
        assert!(state.processes[..state.amount]
            .iter()
            .all(|process| process.active & (1 << this_cpu) == 0));
        state.current = MAX;
        arch::activate(state.kernel.root);
        set_rsp0(service_stack());
    }
    if this_cpu != 0 {
        kernel_leave();
        smp::m3_park();
    }

    kernel_leave();
    // M3_STOP stays false. Already admitted APs complete their kernel entry;
    // later entrants park before taking a ticket and retain an unsaved context.
    let parked = smp::m3_stop_wait();
    kernel_enter();
    // Root contract: flag publication and entering/flag reads are SeqCst;
    // entering stays set through guard Drop. Verify entering=false AND
    // lockdep=0 on the original set, then INIT that set before clearing the
    // request. Any admitted lock holder/waiter is a terminal failure. With
    // that barrier, an unparked CPU may be reset but its context is never used.
    smp::m3_recovery_reset(original_online);
    assert_eq!(smp::online_mask(), 1);
    assert!(!smp::m3_recovery_requested());

    // SAFETY: fresh exclusive borrow after KERNEL was reacquired; INIT has
    // taken effect on every original AP, so even a partial FXSAVE has stopped.
    let state = unsafe { (&mut *ptr::addr_of_mut!(STATE)).assume_init_mut() };
    let free_before = state.free_count();
    let mut crashed = 0;
    for slot in 0..state.amount {
        let process = &mut state.processes[slot];
        if process.quarantined {
            continue;
        }
        if ON_CPU[slot].load(Ordering::Acquire)
            || process.status == Status::Running
            || process.active != 0
        {
            assert!(process.home != 0 && original_online & (1 << process.home) != 0);
            assert_eq!(process.active & 1, 0, "BSP must enter recovery quiescent");
            assert!(!QUEUED[slot].load(Ordering::Acquire));
            // Clear only after INIT, and never reinterpret the partially saved
            // register/FPU image as a resumable context.
            ON_CPU[slot].store(false, Ordering::Release);
            process.active = 0;
            if SCHED_STATE[slot].load(Ordering::Acquire) != sched::DEAD_PENDING {
                sched::kill_request(&SCHED_STATE[slot]).expect("retire lost context");
            }
            sched::kill_commit(&SCHED_STATE[slot], &ON_CPU[slot])
                .expect("INIT completed before retirement");
            retire_authority(slot, process.pid);
            process.status = Status::Dying;
            process.quarantined = true;
            crashed += 1;
        }
    }
    for cpu in 0..smp::MAX_CPUS {
        // SAFETY: KERNEL held; original APs were reset and cannot publish again.
        unsafe { ptr::addr_of_mut!(CPU_CURRENT[cpu]).write(MAX) };
    }
    let mut queued = [false; MAX];
    let mut order = [0u32; MAX];
    let mut amount = 0;
    for (cpu, runq) in RUNQ.iter().enumerate() {
        let mut queue = runq.lock_irqsave();
        while let Some(tid) = queue.pop(&QUEUED) {
            let slot = tid as usize;
            assert!(slot < state.amount && !queued[slot] && amount < MAX);
            assert_eq!(state.processes[slot].home, cpu);
            assert!(!state.processes[slot].quarantined);
            queued[slot] = true;
            order[amount] = tid;
            amount += 1;
        }
    }
    let mut migrated = 0;
    let mut retained = 0;
    #[cfg(feature = "test-smp-m3-recovery")]
    {
        // SAFETY: coordinator owns KERNEL and all APs have been reset.
        assert_eq!(unsafe { ptr::addr_of!(FIXTURE_STAGE).read() }, 1);
        unsafe { ptr::addr_of_mut!(FIXTURE_MIGRATED).write(0) };
    }
    for (slot, was_queued) in queued.iter().enumerate().take(state.amount) {
        assert!(!ON_CPU[slot].load(Ordering::Acquire));
        assert!(!QUEUED[slot].load(Ordering::Acquire));
        assert_eq!(state.processes[slot].active, 0);
        if state.processes[slot].quarantined {
            assert!(!was_queued);
            retained += audit_quarantined(state, slot);
            continue;
        }
        let process = &mut state.processes[slot];
        let status = SCHED_STATE[slot].load(Ordering::Acquire);
        match process.status {
            Status::Ready => assert!(*was_queued && status == sched::READY),
            Status::Waiting => assert!(!was_queued && status == sched::BLOCKED),
            Status::Dying => assert!(*was_queued && status == sched::DEAD_PENDING),
            Status::Dead => {
                assert!(!was_queued && status == sched::DEAD);
                continue;
            }
            Status::Running => serial::fail(b"SMP_M3_RECOVERY_UNSAVED_CONTEXT"),
        }
        if process.home != 0 {
            migrated += 1;
            #[cfg(feature = "test-smp-m3-recovery")]
            if process.status == Status::Ready {
                let before = read_word(process.physical(DATA + 8));
                // SAFETY: KERNEL owns the bounded fixture arrays; a copied
                // counter records actual user work before BSP dispatch.
                unsafe {
                    ptr::addr_of_mut!(FIXTURE_PROGRESS[slot]).write(before);
                    let mask = ptr::addr_of!(FIXTURE_MIGRATED).read();
                    ptr::addr_of_mut!(FIXTURE_MIGRATED).write(mask | (1 << slot));
                }
            }
        }
        process.home = 0;
    }
    {
        let mut bsp = RUNQ[0].lock_irqsave();
        for tid in &order[..amount] {
            bsp.push(*tid, &QUEUED)
                .expect("single migration queue owner");
        }
    }
    // RUNQ release/acquire and each ON_CPU Acquire preserve the old CPU's
    // completed FXSAVE before the BSP's existing restore path sees the image.
    assert_eq!(state.free_count(), free_before);
    state.current = MAX;
    let mut line = serial::Line::new();
    line.push(b"SMP:M3:RECOVERY:CPUS=")
        .decimal(u64::from(original_online.count_ones()))
        .push(b":MIGRATED=")
        .decimal(migrated)
        .push(b":CRASHED=")
        .decimal(crashed)
        .push(b":PARKED=")
        .decimal(u64::from(parked))
        .push(b":RETAINED_FRAMES=")
        .decimal(retained as u64)
        .push(b":FREE=")
        .decimal(free_before as u64)
        .push(b"\n");
    serial::write(line.as_bytes());
    #[cfg(feature = "test-smp-m3-recovery")]
    {
        // SAFETY: publish only after every saved context belongs to the BSP.
        unsafe { ptr::addr_of_mut!(FIXTURE_STAGE).write(2) };
    }
    state.select()
}
