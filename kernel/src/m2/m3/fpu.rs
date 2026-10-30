//! Eager x87/SSE state on the single-CPU profile (no XSAVE/AVX or kernel FP).
//!
//! Instruction/bit contracts: Intel SDM vol.1, "Managing State Using the FXSAVE
//! and FXRSTOR Instructions"; vol.2 CPUID/FXSAVE/FXRSTOR/LDMXCSR; vol.3 CR0/CR4.
//! OSDev FPU W002427 revision29122 is the local secondary cross-reference.
//! The 22-word interrupt frame is deliberately unchanged. Contexts are private
//! supervisor metadata, never user pointers or a user-modifiable restore image.
use super::MAX;
use core::ptr;

#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct Image([u8; 512]);
const _: () = assert!(
    core::mem::size_of::<Image>() == 512 && core::mem::align_of::<Image>() == 16 && MAX == 16
);
static mut IMAGES: [Image; MAX] = [Image([0; 512]); MAX];
#[no_mangle]
static mut m3_fpu_current: u64 = 0;
#[no_mangle]
static mut m3_fpu_saves: u64 = 0;
#[no_mangle]
static mut m3_fpu_restores: u64 = 0;

fn address(slot: usize) -> *mut u8 {
    assert!(slot < MAX);
    // No reference spans user execution or aliases an assembly write.
    ptr::addr_of_mut!(IMAGES)
        .cast::<Image>()
        .wrapping_add(slot)
        .cast()
}
pub(super) fn clear(slot: usize) {
    let image = address(slot);
    // SAFETY: One CPU, IF=0, owned permanent supervisor-only metadata. The user
    // entry save has finished before cleanup; it cannot run during this wipe.
    unsafe { ptr::write_bytes(image, 0, 512) };
}
pub(super) fn initialize(slot: usize) {
    clear(slot);
    let image = address(slot);
    // SAFETY: Aligned owned 512-byte image. FXSAVE layout: FCW=037f, MXCSR=1f80;
    // FTW=0, ST payloads/XMM0..15/FIP/FDP=0. FNINIT alone would retain payloads.
    // The fixed MXCSR has no reserved bits on the required SSE2 CPU. No user can
    // supply a restore image. MXCSR_MASK is ignored by FXRSTOR.
    unsafe {
        image.cast::<u16>().write(0x037f);
        image.add(24).cast::<u32>().write(0x1f80);
    }
}
pub(super) fn select(slot: usize) {
    let image = address(slot);
    assert_eq!(image as usize & 15, 0);
    // SAFETY: IF=0, single-CPU. Set only a validated owned pointer; the terminal
    // assembly path restores this image AFTER all Rust and immediately before
    // pop/iret. Context was initialized or saved by actual FXSAVE64, so its
    // MXCSR is valid. No references to this image survive a user return.
    unsafe {
        ptr::addr_of_mut!(m3_fpu_current).write(image as u64);
    }
}

/// Separate observer fixture; never enables rights or supplies process results.
#[cfg(feature = "native-fpu")]
pub(super) mod fixture {
    use super::*;
    use crate::m2::m3::{print_number, read_word, write_word, State, Status, DATA};
    use crate::serial;

    struct Evidence {
        baseline: usize,
        faults: u64,
        exits: u64,
        reuse: u64,
        loops: u64,
        clock_reads: u64,
        clock_denied: u64,
        clean: u64,
        timer: [u64; 2],
        first_b_loops: u64,
    }
    static mut EVIDENCE: Evidence = Evidence {
        baseline: 0,
        faults: 0,
        exits: 0,
        reuse: 0,
        loops: 0,
        clock_reads: 0,
        clock_denied: 0,
        clean: 0,
        timer: [0; 2],
        first_b_loops: 0,
    };
    fn evidence() -> &'static mut Evidence {
        // SAFETY: This fixture runs only under the single-CPU IF=0 event loop.
        unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) }
    }
    fn require(ok: bool, why: &[u8]) {
        if !ok {
            serial::write(b"FPU:FAIL:");
            serial::write(why);
            serial::write(b"\n");
            serial::exit(0x11);
        }
    }
    fn word(s: &State, slot: usize, offset: u64) -> u64 {
        read_word(s.processes[slot].physical(DATA + offset))
    }
    pub(in crate::m2::m3) fn manifest(s: &State) {
        require(
            s.amount == 2 && s.processes[0].pid == 1 && s.processes[1].pid == 2,
            b"MANIFEST",
        );
        // Kernel page-table containers are intentionally retained. Account every
        // currently owned process/table/stack frame independently of markers.
        evidence().baseline = s.free_count()
            + s.processes[..2]
                .iter()
                .map(|p| {
                    p.count + p.kernel_frames.len() + p.paging.as_ref().unwrap().owned_table_count()
                })
                .sum::<usize>();
        write_word(s.processes[0].physical(DATA + 64), 1);
        write_word(s.processes[1].physical(DATA + 64), 1);
        serial::write(b"FPU:MANIFEST:PASS\n");
    }
    pub(in crate::m2::m3) fn entry(slot: usize, vector: u64) {
        if vector == 32 {
            evidence().timer[slot] += 1;
        }
    }
    fn collect(s: &State, slot: usize) {
        let e = evidence();
        require(
            word(s, slot, 80) == 1 && word(s, slot, 88) >= 8,
            b"STATE_CHECKS",
        );
        require(
            word(s, slot, 104) >= word(s, slot, 96) + 12 && word(s, slot, 112) >= 2,
            b"CLOCK_PROGRESS",
        );
        require(
            word(s, slot, 120) >= 8 && word(s, slot, 128) == 6,
            b"CLOCK_ARGUMENTS",
        );
        require(
            word(s, slot, 136) == 1 && word(s, slot, 144) == 1,
            b"COMPLETION",
        );
        e.clean += 1;
        e.loops += word(s, slot, 88);
        e.clock_reads += word(s, slot, 120);
        e.clock_denied += 6;
    }
    pub(in crate::m2::m3) fn observe_fault(s: &State, slot: usize, c: &[u64; 22], _: u64) {
        let e = evidence();
        require(
            slot == 0 && e.faults == 0 && e.reuse == 0 && word(s, slot, 64) == 1,
            b"FAULT_IDENTITY",
        );
        require(
            s.last_event.vector == 6 && c[16] == 0,
            b"UNSUPPORTED_AVX_UD",
        );
        collect(s, slot);
        evidence().faults = 1;
        evidence().first_b_loops = word(s, 1, 88);
        serial::write(b"FPU:UNSUPPORTED_AVX:PASS\n");
    }
    pub(in crate::m2::m3) fn observe_exit(s: &State, slot: usize, _: u64) {
        if s.last_event.code != 0 {
            print_number(b"FPU:APP_FAILURE_SLOT:", slot as u64);
            print_number(b"FPU:APP_FAILURE_CODE:", s.last_event.code);
            print_number(b"FPU:APP_FAILURE_STEP:", word(s, slot, 152));
        }
        require(s.last_event.code == 0, b"APP_EXIT");
        require(
            if slot == 0 {
                evidence().reuse == 1 && word(s, slot, 64) == 2
            } else {
                evidence().exits == 1
            },
            b"EXIT_ORDER",
        );
        collect(s, slot);
        evidence().exits += 1;
        if slot == 0 {
            write_word(s.processes[1].physical(DATA + 160), 1);
        }
    }
    pub(in crate::m2::m3) fn poll(s: &mut State) {
        require(
            s.ticks < 2000 && s.syscall_calls < 2_000_000,
            b"BOUNDED_RUN",
        );
        if evidence().faults == 1 && evidence().reuse == 0 && s.processes[0].status == Status::Dead
        {
            let survivor = s.processes[1].status == Status::Ready;
            #[cfg(feature = "test-fpu-wait")]
            let survivor = survivor || s.processes[1].status == Status::Waiting;
            require(survivor, b"SURVIVOR");
            if word(s, 1, 88) > evidence().first_b_loops + 8 {
                s.spawn_process(0);
                require(s.processes[0].pid == 3, b"FRESH_PID");
                write_word(s.processes[0].physical(DATA + 64), 2);
                evidence().reuse = 1;
                serial::write(b"FPU:SURVIVOR:PASS\nFPU:SLOT_REUSE:PASS\n");
            }
        }
        if s.processes[..2].iter().all(|p| p.status == Status::Dead) {
            let e = evidence();
            require(
                e.faults == 1 && e.exits == 2 && e.reuse == 1 && e.clean == 3,
                b"LIFECYCLE",
            );
            require(e.timer.iter().all(|x| *x >= 2), b"TIMER_PREEMPTION");
            require(s.free_count() == e.baseline, b"FRAME_RECOVERY");
            // SAFETY: No process is live. All image slots must have been wiped.
            let zero =
                unsafe { core::slice::from_raw_parts(ptr::addr_of!(IMAGES).cast::<u8>(), 8192) }
                    .iter()
                    .all(|x| *x == 0);
            require(zero, b"CONTEXT_WIPE");
            // SAFETY: IF=0, no user is live. Assembly cannot update either
            // permanent single-CPU diagnostic counter during these reads.
            let saves = unsafe { ptr::addr_of!(m3_fpu_saves).read() };
            // SAFETY: Same stopped-user/IF=0 counter ownership as above.
            let restores = unsafe { ptr::addr_of!(m3_fpu_restores).read() };
            require(
                saves == s.timer_interrupts + s.syscall_calls + 1 && restores == saves,
                b"ENTRY_SWITCH_COUNTS",
            );
            for (name, value) in [
                (b"FPU:CLEAN_STARTS:".as_slice(), e.clean),
                (b"FPU:VERIFIED_LOOPS:", e.loops),
                (b"FPU:CLOCK_READS:", e.clock_reads),
                (b"FPU:CLOCK_DENIED:", e.clock_denied),
                (b"FPU:TIMER_A:", e.timer[0]),
                (b"FPU:TIMER_B:", e.timer[1]),
                (b"FPU:TIMER_IRQS:", s.timer_interrupts),
                (b"FPU:SYSCALL_CALLS:", s.syscall_calls),
                (b"FPU:SAVES:", saves),
                (b"FPU:RESTORES:", restores),
                (b"FPU:FAULTS:", e.faults),
                (b"FPU:EXITS:", e.exits),
                (b"FPU:SLOT_REUSES:", e.reuse),
                (b"FPU:FREE_FRAMES:", s.free_count() as u64),
                (b"FPU:CONTEXT_BYTES_WIPED:", 8192),
            ] {
                print_number(name, value);
            }
            #[cfg(feature = "test-fpu-wait")]
            {
                let m = s.wait_metrics;
                require(
                    m.calls == e.loops
                        && m.blocked >= 3
                        && m.wake_timeout == m.blocked
                        && m.immediate_timeout + m.blocked == m.calls
                        && m.immediate_ready == 0
                        && m.wake_ready == 0
                        && m.wake_error == 0
                        && m.idle_irqs > 0,
                    b"REAL_WAIT_LIFECYCLES",
                );
                for (name, value) in [
                    (b"FPU:WAIT_CALLS:".as_slice(), m.calls),
                    (b"FPU:WAIT_BLOCKED:", m.blocked),
                    (b"FPU:WAIT_TIMEOUTS:", m.wake_timeout),
                    (b"FPU:WAIT_IMMEDIATE:", m.immediate_timeout),
                    (b"FPU:WAIT_IDLE_IRQS:", m.idle_irqs),
                ] {
                    print_number(name, value);
                }
                serial::write(b"FPU:WAIT:PASS\n");
            }
            serial::write(b"FPU:CONTEXT_ISOLATION:PASS\nFPU:CLOCK:PASS\nFPU:CLEANUP:PASS\nFPU:PASS\nM1:PASS\n");
            serial::exit(0x10)
        }
    }
}
