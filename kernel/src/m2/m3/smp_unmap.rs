//! Opt-in M3 isolation, guarded usercopy and remote unmap/reuse evidence.
//! The negative profile deliberately omits only the process shootdown. CPU 1's
//! local timer is masked during its bounded read loop so an unrelated CR3
//! reload cannot conceal a stale translation; directed shootdowns stay enabled.
use super::*;
use kernel_core::user::PageAccess;

const TARGET: usize = 1;
const REUSE_VA: u64 = HEAP + 16 * PAGE;
const REUSED_PATTERN: u64 = 0x7272_0000_0000_5f5f;
// The control must see the reused frame before the existing 100-ms shootdown
// limit; this bound is not widened after observing a failure.
const CONTROL_MS: u64 = 50;
const FIXTURE_MS: u64 = 1000;
static mut PHASE: u8 = 0;
static mut STARTED: u64 = 0;
static mut FAULT_SEEN: bool = false;

pub(super) fn manifest(state: &mut State) {
    assert!(lock::kernel_held());
    assert!(smp::online_mask() & 3 == 3);
    assert_eq!(state.processes[TARGET].home, 1);
    assert_ne!(
        state.processes[0].physical(HEAP),
        state.processes[TARGET].physical(HEAP)
    );
    for slot in 0..state.amount {
        let _address = ASPACE[slot].lock_irqsave();
        write_word(state.processes[slot].physical(DATA + 24), slot as u64);
    }
    // SAFETY: Fixture state is accessed only under KERNEL, before M3_GO.
    unsafe {
        PHASE = 0;
        STARTED = smp::m3_time_ms();
        FAULT_SEEN = false;
    }
}

pub(super) fn before_dispatch(slot: usize) {
    assert!(lock::kernel_held());
    // SAFETY: KERNEL serializes every read and mutation of fixture state.
    if slot == TARGET && unsafe { PHASE == 0 } {
        assert_eq!(cpu(), 1);
        smp::m3_unmap_timer_mask(true);
    }
}

/// This accepts only the precise post-shootdown user read fault. Any other
/// fault remains subject to the ordinary unexpected-app-event failure path.
pub(super) fn expected_fault(
    state: &State,
    slot: usize,
    vector: u64,
    error: u64,
    cr2: u64,
) -> bool {
    assert!(lock::kernel_held());
    if cfg!(feature = "test-smp-m3-no-shootdown")
        || slot != TARGET || cpu() != 1 || vector != 14 || error != 4 || cr2 != HEAP
        // SAFETY: KERNEL held; no other CPU accesses fixture state concurrently.
        || unsafe { PHASE != 1 || FAULT_SEEN }
    {
        return false;
    }
    assert_eq!(state.processes[slot].active, 0);
    let _address = ASPACE[slot].lock_irqsave();
    assert_eq!(read_word(state.processes[slot].physical(DATA + 72)), 0);
    // SAFETY: KERNEL held; the exact expected fault is recorded once.
    unsafe {
        FAULT_SEEN = true;
    }
    smp::m3_unmap_timer_mask(false);
    serial::write(b"SMP:M3:UNMAP:FAULT:CPU=1\n");
    true
}

pub(super) fn poll(state: &mut State) {
    assert!(lock::kernel_held());
    if cpu() != 0 {
        return;
    }
    // SAFETY: KERNEL held and STARTED was initialized before dispatch.
    if smp::m3_time_ms().saturating_sub(unsafe { STARTED }) >= FIXTURE_MS {
        serial::fail(b"SMP_M3_UNMAP_DEADLINE");
    }
    // SAFETY: This fixture's state has no access outside KERNEL.
    let phase = unsafe { PHASE };
    if phase == 1 {
        // SAFETY: Same serialized access; target cleanup happens on CPU 1
        // after its exact fault and before the BSP can regain KERNEL.
        if unsafe { FAULT_SEEN } && state.processes[TARGET].status == Status::Dead {
            finish();
        }
    } else if phase == 0 {
        let (low, high) = ASPACE[0].lock_pair(&ASPACE[TARGET]);
        let target = &state.processes[TARGET];
        if target.active != 2
            || read_word(target.physical(DATA + 8)) < 32
            || read_word(state.processes[0].physical(DATA + 8)) < 32
        {
            return;
        }
        let old = target.pattern;
        assert_eq!(read_word(target.physical(DATA + 64)), old);
        assert_eq!(
            read_word(state.processes[0].physical(HEAP)),
            state.processes[0].pattern
        );
        assert_eq!(read_word(target.physical(HEAP)), old);
        serial::write(b"SMP:M3:ISOLATION:PASS\n");

        // A real copy uses the live process's page tables and ownership while
        // its ASPACE guard is held. KERNEL currently serializes copy and unmap
        // as well; this does not claim lock-free concurrent kernel execution.
        let plan = user::copy_plan(HEAP, 8, target.pid, false, |va| {
            let (physical, writable, _, user) = target.paging.as_ref()?.permissions(va)?;
            let Owner::User(owner, _) = state.frames.owner(physical & !(PAGE - 1)).ok()? else {
                return None;
            };
            Some(PageAccess {
                physical: physical & !(PAGE - 1),
                owner,
                user,
                writable,
            })
        })
        .expect("live guarded copy");
        assert_eq!(plan.count, 1);
        let chunk = plan.chunks[0];
        // The CopyPlan proves eight bytes belong to this live process;
        // ASPACE and KERNEL remain held through the existing scratch-copy path.
        let copied = read_word(chunk.physical + chunk.offset as u64);
        assert_eq!(copied, old);
        let active = target.active;
        let owner = Owner::User(target.pid, Kind::Data);
        let index = target.mappings().iter().position(|m| m.va == HEAP).unwrap();
        let original = target.mappings()[index];
        assert_eq!(original.kind, Kind::Data);

        let pa = state.processes[TARGET]
            .paging
            .as_mut()
            .unwrap()
            .unmap_m3_shot(state.frames, HEAP, active)
            .unwrap_or_else(|_| serial::fail(b"SMP_M3_UNMAP_SHOOTDOWN"));
        assert_eq!(pa, original.pa);
        assert!(state.processes[TARGET]
            .paging
            .as_ref()
            .unwrap()
            .permissions(HEAP)
            .is_none());
        assert!(
            user::copy_plan(HEAP, 8, state.processes[TARGET].pid, false, |va| {
                let (physical, writable, _, user) =
                    state.processes[TARGET].paging.as_ref()?.permissions(va)?;
                let Owner::User(owner, _) = state.frames.owner(physical & !(PAGE - 1)).ok()? else {
                    return None;
                };
                Some(PageAccess {
                    physical: physical & !(PAGE - 1),
                    owner,
                    user,
                    writable,
                })
            })
            .is_err()
        );
        serial::write(b"SMP:M3:USERCOPY:UNMAP:PASS\n");
        if !cfg!(feature = "test-smp-m3-no-shootdown") {
            serial::write(b"SMP:M3:UNMAP:SHOOTDOWN:PASS\n");
        }
        let target = &mut state.processes[TARGET];
        let last = target.mappings()[target.count - 1];
        // SAFETY: Exclusive Process borrow, KERNEL and its ASPACE held. Only
        // the acknowledged removed leaf leaves the owner's mapping inventory.
        unsafe {
            ptr::addr_of_mut!(USER_MAPS)
                .cast::<Mapping>()
                .add(target.map_slot * MAPS + index)
                .write(last);
            ptr::addr_of_mut!(USER_MAPS)
                .cast::<Mapping>()
                .add(target.map_slot * MAPS + target.count - 1)
                .write(EMPTY_MAP);
        }
        target.count -= 1;

        // Release the kernel alias only after the user round acknowledged.
        // This one-page kernel invalidation deliberately targets pa, not HEAP;
        // the control must retain its missing user invalidation as the hazard.
        let retired_alias = state.kernel.clear_leaf(pa);
        smp::shootdown_kernel(pa, 1).unwrap_or_else(|_| serial::fail(b"SMP_M3_UNMAP_KERNEL_ALIAS"));
        paging::release_leaf(state.frames, retired_alias);
        state.frames.free(pa, owner, paging::zero_dynamic).unwrap();
        let replacement = state
            .frames
            .allocate(
                Owner::User(state.processes[0].pid, Kind::Data),
                paging::zero_dynamic,
            )
            .unwrap();
        assert_eq!(replacement, pa, "the probe must observe actual frame reuse");
        state.kernel.map(state.frames, pa, pa, true, false);
        state.processes[0].paging.as_mut().unwrap().map_user(
            state.frames,
            REUSE_VA,
            pa,
            true,
            false,
        );
        state.processes[0].append_mapping(Mapping {
            va: REUSE_VA,
            pa,
            kind: Kind::Data,
            writable: true,
            executable: false,
            retired_alias: 0,
        });
        smp::shootdown_kernel(pa, 1).unwrap_or_else(|_| serial::fail(b"SMP_M3_UNMAP_REUSE_ALIAS"));
        write_word(pa, REUSED_PATTERN);
        serial::write(b"SMP:M3:UNMAP:REUSED:PASS\n");
        // SAFETY: KERNEL held; publish the phase before allowing the pending
        // target #PF to enter the ordinary process-fault/cleanup route.
        unsafe {
            PHASE = 1;
        }
        if cfg!(feature = "test-smp-m3-no-shootdown") {
            let deadline = smp::m3_time_ms().saturating_add(CONTROL_MS);
            loop {
                let observed = read_word(state.processes[TARGET].physical(DATA + 72));
                if observed == REUSED_PATTERN {
                    serial::write(b"SMP:M3:UNMAP:STALE:CONTROL\n");
                    break;
                }
                if observed != 0 || smp::m3_time_ms() >= deadline {
                    serial::fail(b"SMP_M3_UNMAP_STALE_NOT_OBSERVED");
                }
                core::hint::spin_loop();
            }
            drop(high);
            drop(low);
            finish();
        }
    }
}

fn finish() -> ! {
    assert!(lock::kernel_held());
    assert_eq!(cpu(), 0);
    smp::m3_stop_begin();
    kernel_leave();
    let stopped = smp::m3_stop_wait();
    kernel_enter();
    if !stopped {
        serial::fail(b"SMP_M3_UNMAP_STOP");
    }
    // SAFETY: KERNEL held again; all other CPUs have acknowledged permanent
    // kernel-CR3 parking. No earlier STATE/user-memory reference is used here.
    let state = unsafe { (&mut *ptr::addr_of_mut!(STATE)).assume_init_mut() };
    arch::activate(state.kernel.root);
    state.current = MAX;
    for queue in &RUNQ {
        while queue.lock_irqsave().pop(&QUEUED).is_some() {}
    }
    for slot in 0..state.amount {
        state.processes[slot].active = 0;
        ON_CPU[slot].store(false, Ordering::Release);
        if state.processes[slot].status != Status::Dead {
            state.processes[slot].status = Status::Dying;
            state.cleanup(slot);
        }
    }
    assert_eq!(state.free_count(), state.free_baseline);
    serial::write(b"SMP:M3:UNMAP:CLEANUP:PASS\n");
    if !cfg!(feature = "test-smp-m3-no-shootdown") {
        serial::write(b"SMP:M3:UNMAP:PASS\n");
    }
    serial::write(b"M1:PASS\n");
    serial::exit(0x10)
}
