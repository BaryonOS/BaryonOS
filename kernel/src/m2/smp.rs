//! K06 step 1 (native-smp only): bounded application-processor bring-up.
//!
//! Ownership and concurrency model, see ADR-070:
//! - The BSP alone owns frames, paging, serial, the PIT and the xAPIC ICR. It
//!   prepares everything with IF=0 before the first IPI.
//! - The SIPI page below 1 MiB is a Kernel-owned frame written once through the
//!   firmware identity map before our CR3 switch; it holds 16-bit code and a
//!   CR3 slot only and is never mapped in the kernel tables. 32/64-bit code,
//!   the trampoline GDT and every AP stack live in the identity-mapped kernel
//!   image, so no low page needs an executable mapping (W^X unchanged).
//! - An AP selects its context by its hardware initial APIC ID and claims it
//!   with one `lock cmpxchg` STARTING->CLAIMED. The BSP's timeout uses the same
//!   CAS STARTING->ABANDONED, so a late AP can never run on a stack the BSP has
//!   given up; a claimed-but-silent AP is reset with INIT and bring-up stops.
//! - APs touch only their own stacks, their context/CpuLocal atomics, the
//!   read-only BSP register snapshot and the self-test state. They never enter
//!   the allocator, page-table changes, IPC or the M3 scheduler; since step
//!   5b-2 they map a temporary page only through their own scratch slot (ADR-075). Each AP has its own TSS/IST
//!   (ADR-071); after bring-up it waits with `sti; hlt` for bounded IPIs and
//!   runs two preempted kernel threads with IF=1 (ADR-073), then a per-CPU
//!   run queue of kernel threads in two address spaces (step 5d-1), then
//!   parks with IF=0. Serial is touched only on terminal paths: a CPU exception in the
//!   shared M2 handlers or the unreachable no-anchor check in `smp_preempt`;
//!   both stop the VM, never a resumed or corrupted state.
//! - If an AP that failed cannot be reset with INIT, the boot stops (fail):
//!   an uncontrolled CPU is never left running beside the kernel.
use super::{arch, lock, paging, zero, FRAME_COUNT};
use crate::serial;
use boot_protocol::acpi::{Madt, MadtStatus};
use core::{
    arch::{asm, global_asm},
    hint::spin_loop,
    ptr,
    sync::atomic::{fence, AtomicBool, AtomicU32, AtomicU64, Ordering},
};
use kernel_core::{Frames, Owner, PAGE};
use smp_lock::{
    level,
    sched::{self, RunQueue},
    TicketLock,
};

/// Total CPUs (BSP plus APs) this profile starts; further MADT CPUs are skipped.
pub const MAX_CPUS: usize = paging::MAX_CPUS;
const MAX_APS: usize = MAX_CPUS - 1;

const IDLE: u32 = 0;
const STARTING: u32 = 1;
const CLAIMED: u32 = 2;
const ONLINE: u32 = 3;
const MISMATCH: u32 = 4;
const DONE: u32 = 5;
const ABANDONED: u32 = 6;
const IPI_READY: u32 = 7;
const PARKED: u32 = 8;

const CR0: usize = 0;
const CR4: usize = 2;
const EFER: usize = 3;
const PAT: usize = 4;
const REGISTERS: usize = 5;

/// Offsets 0/4/8 are part of the assembly ABI below.
#[repr(C, align(64))]
pub struct ApContext {
    state: AtomicU32,
    apic_id: AtomicU32,
    stack_top: AtomicU64,
    observed_apic: AtomicU32,
    reserved: AtomicU32,
    observed: [AtomicU64; REGISTERS],
}
const CONTEXT_SIZE: usize = core::mem::size_of::<ApContext>();
const _: () = {
    assert!(core::mem::offset_of!(ApContext, state) == 0);
    assert!(core::mem::offset_of!(ApContext, apic_id) == 4);
    assert!(core::mem::offset_of!(ApContext, stack_top) == 8);
    assert!(CONTEXT_SIZE == 64);
};
impl ApContext {
    const fn new() -> Self {
        Self {
            state: AtomicU32::new(IDLE),
            // No xAPIC or x2APIC CPU has this ID, so unused slots never match.
            apic_id: AtomicU32::new(u32::MAX),
            stack_top: AtomicU64::new(0),
            observed_apic: AtomicU32::new(u32::MAX),
            reserved: AtomicU32::new(0),
            observed: [const { AtomicU64::new(0) }; REGISTERS],
        }
    }
}
#[no_mangle]
static NOS_SMP_AP_CONTEXTS: [ApContext; MAX_APS] = [const { ApContext::new() }; MAX_APS];

#[repr(C, align(4096))]
struct ApStack {
    lower: [u8; 4096],
    data: [u8; 16384],
    upper: [u8; 4096],
}
/// Per AP: main, #DF (IST1) and NMI (IST2) stacks, each with unmapped guards.
const MAIN: usize = 0;
const DOUBLE: usize = 1;
const NMI: usize = 2;
/// Two preemptible kernel threads per AP (K06 step 4).
const THREADS: [usize; 2] = [3, 4];
const STACK_KINDS: usize = 5;
// Kernel image .bss; every guard is left unmapped by m2::run (see is_guard).
static mut AP_STACKS: [[ApStack; STACK_KINDS]; MAX_APS] = [const {
    [const {
        ApStack {
            lower: [0; 4096],
            data: [0; 16384],
            upper: [0; 4096],
        }
    }; STACK_KINDS]
}; MAX_APS];
const STACK_SPAN: u64 = core::mem::size_of::<ApStack>() as u64;
const STACK_TOP: u64 = 20480;

fn stack_base(slot: usize, kind: usize) -> u64 {
    // Address computation only; no reference to the static is formed.
    ptr::addr_of!(AP_STACKS) as u64 + (slot * STACK_KINDS + kind) as u64 * STACK_SPAN
}
pub fn is_guard(page: u64) -> bool {
    guard_pages().any(|guard| guard == page)
}
pub fn guard_pages() -> impl Iterator<Item = u64> {
    (0..MAX_APS * STACK_KINDS).flat_map(|index| {
        let base = stack_base(index / STACK_KINDS, index % STACK_KINDS);
        [base, base + STACK_TOP]
    })
}

/// Per-CPU anchor reachable through IA32_GS_BASE. Offsets 0, 12 and 16 are
/// used by the IPI/shootdown stubs; `slot` identifies the AP without trusting
/// any register. `before`/`after` record what this CPU read through
/// SHOOTDOWN_WINDOW before the remap and at the visibility probe.
#[repr(C, align(64))]
struct CpuLocal {
    self_ptr: AtomicU64,
    slot: AtomicU32,
    ipis: AtomicU32,
    shootdown_ack: AtomicU32,
    before: AtomicU32,
    after: AtomicU32,
    probed: AtomicU32,
    // Preemption (step 4): touched only by this AP's own handler/threads,
    // read by the BSP. saved[0] is the interrupted home loop, [1..] threads.
    ticks: AtomicU32,
    switches: AtomicU32,
    current: AtomicU32,
    preempt: AtomicU32,
    counts: [AtomicU64; 2],
    saved: [AtomicU64; 3],
    /// Set by a thread whose callee-saved registers changed across preemption.
    corrupt: AtomicU32,
    /// Held lock levels (step 5a, `smp_lock::Lockdep`); this CPU only, IF=0.
    lockdep: AtomicU64,
    /// Step 5b spin phase: 1 = about to wait for SPIN_LOCK (IF=0 from here),
    /// 2 = got it and released it again.
    spun: AtomicU32,
    /// Step 5c: timer ticks not switched because this CPU held or awaited a
    /// lock (lockdep non-empty); written by this CPU's handler only.
    deferred: AtomicU32,
    /// Step 5d-1 scheduler: running thread (HOME = the home loop), ARMED /
    /// RESUMED (stopped at home), 0xe8 reschedule IPIs received and CR3
    /// switches. Written by this CPU only, read by the BSP.
    sched_cur: AtomicU32,
    sched_state: AtomicU32,
    resched: AtomicU32,
    cr3_switches: AtomicU32,
    /// Audit F8 ring-3 GS fixture: phase and entries taken from CPL3.
    gs_state: AtomicU32,
    user_ticks: AtomicU32,
    /// Step 5b-2 scratch fixture: 1 once this AP finished its share.
    scratched: AtomicU32,
    /// LD-1 (audit F3): IDT entries this CPU is nested in. Every GS stub
    /// counts it after `gs_entry!` and before `gs_exit!`; this CPU only.
    irq_depth: AtomicU32,
    /// Full lockdep state behind the `lockdep` mirror (audit F4).
    lockdep_state: lock::LockdepCell,
    /// F4 fixture: 1 once this AP finished its `lock_pair` rounds.
    paired: AtomicU32,
}
impl CpuLocal {
    const fn new() -> Self {
        Self {
            self_ptr: AtomicU64::new(0),
            slot: AtomicU32::new(u32::MAX),
            ipis: AtomicU32::new(0),
            shootdown_ack: AtomicU32::new(0),
            before: AtomicU32::new(u32::MAX),
            after: AtomicU32::new(u32::MAX),
            probed: AtomicU32::new(0),
            ticks: AtomicU32::new(0),
            switches: AtomicU32::new(0),
            current: AtomicU32::new(0),
            preempt: AtomicU32::new(0),
            counts: [const { AtomicU64::new(0) }; 2],
            saved: [const { AtomicU64::new(0) }; 3],
            corrupt: AtomicU32::new(0),
            lockdep: AtomicU64::new(0),
            spun: AtomicU32::new(0),
            deferred: AtomicU32::new(0),
            sched_cur: AtomicU32::new(HOME),
            sched_state: AtomicU32::new(0),
            resched: AtomicU32::new(0),
            cr3_switches: AtomicU32::new(0),
            gs_state: AtomicU32::new(0),
            user_ticks: AtomicU32::new(0),
            scratched: AtomicU32::new(0),
            irq_depth: AtomicU32::new(0),
            lockdep_state: lock::LockdepCell::new(),
            paired: AtomicU32::new(0),
        }
    }
}
const _: () = {
    assert!(core::mem::offset_of!(CpuLocal, self_ptr) == 0);
    assert!(core::mem::offset_of!(CpuLocal, ipis) == 12);
    assert!(core::mem::offset_of!(CpuLocal, shootdown_ack) == 16);
};
static CPU_LOCAL: [CpuLocal; MAX_APS] = [const { CpuLocal::new() }; MAX_APS];
/// CPU index 0 (spec §3.1.3): same GS-anchor path as the APs. Kept outside
/// CPU_LOCAL so AP-only lookups (`this_cpu`, IST markers) still miss the BSP.
static BSP_LOCAL: CpuLocal = CpuLocal::new();

/// Shootdown request, published (Release) before the IPI that makes an AP
/// read it: up to two VAs (0 = unused slot is never published here) and a
/// generation that every round increments; the stub acknowledges the
/// generation only after both `invlpg`s.
#[no_mangle]
static NOS_SMP_SHOOTDOWN_VA: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];
#[no_mangle]
static NOS_SMP_SHOOTDOWN_GEN: AtomicU32 = AtomicU32::new(0);
static PROBE: AtomicU32 = AtomicU32::new(0);
/// Frames A (initially behind the window) and B (its replacement).
static SHOOTDOWN_FRAMES: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];
const SHOOTDOWN_VECTOR: usize = 0xe0;
const TIMER_VECTOR: usize = 0xd0;
/// BSP-driven preemption phase: idle -> run -> stop.
static PREEMPT: AtomicU32 = AtomicU32::new(0);
const PREEMPT_RUN: u32 = 1;
const PREEMPT_STOP: u32 = 2;
// Per-AP preempt progress in CpuLocal::preempt.
const ARMED: u32 = 1;
const RESUMED: u32 = 2;
const OLD_PATTERN: u8 = 0xa1;
const NEW_PATTERN: u8 = 0xb2;
// Each slot is written only by the AP that claimed the matching context.
static mut CPU_TABLES: [arch::CpuTables; MAX_APS] = [const { arch::CpuTables::new() }; MAX_APS];

static BSP_REGISTERS: [AtomicU64; REGISTERS] = [const { AtomicU64::new(0) }; REGISTERS];
/// The BSP's IA32_APIC_BASE; an AP must match it except for the BSP flag
/// (bit 8), or its LAPIC_WINDOW accesses would not reach its own xAPIC.
static BSP_APIC_BASE: AtomicU64 = AtomicU64::new(0);
static GO: AtomicBool = AtomicBool::new(false);

/// Every shootdown ack (and the other per-AP handshakes) is awaited this long.
const SHOOTDOWN_DEADLINE_MS: u32 = 100;
/// DG-1 (audit F5): a lock wait longer than this is a deadlock, in every
/// native-smp image.
const LOCK_STUCK_MS: u64 = 4 * SHOOTDOWN_DEADLINE_MS as u64;
/// TSC ticks per millisecond, measured once by the BSP against PIT channel 2
/// before any AP starts; 0 until then (no second CPU can wait before that).
/// shortcut: the PIT is the time base, but a waiter reads the TSC because
/// APs never touch the BSP-owned PIT (ADR-070); assumes a constant-rate TSC
/// (QEMU/TCG, KVM). Replace by the calibrated TSC of step 06.
static TSC_PER_MS: AtomicU64 = AtomicU64::new(0);

fn rdtsc() -> u64 {
    let (low, high): (u32, u32);
    // SAFETY: CPL0 read of the time-stamp counter; no memory is accessed.
    unsafe { asm!("rdtsc", out("eax") low, out("edx") high, options(nomem, nostack)) };
    u64::from(high) << 32 | u64::from(low)
}

/// BSP, IF=0, before the first IPI: 10 ms on the PIT against the TSC.
fn calibrate_tsc() {
    let start = rdtsc();
    if !wait_us(10_000) {
        serial::fail(b"SMP_TSC_CALIBRATION");
    }
    let per_ms = (rdtsc().wrapping_sub(start) / 10).max(1);
    TSC_PER_MS.store(per_ms, Ordering::Release);
}

/// TSC value after which a lock wait that starts now is stuck (DG-1).
pub fn stuck_deadline() -> u64 {
    rdtsc().saturating_add(LOCK_STUCK_MS * TSC_PER_MS.load(Ordering::Acquire))
}

pub fn past(deadline: u64) -> bool {
    rdtsc() > deadline
}

/// F7 fixture (DG-2): lines every CPU writes at once, in fragments, so the
/// host can see each line arrive whole and in order per CPU.
const SERIAL_LINES: u64 = 200;
fn serial_lines() {
    let cpu = cpu_index() as u64;
    for seq in 0..SERIAL_LINES {
        serial::write(b"SMP:SER:");
        decimal(cpu);
        serial::write(b":");
        decimal(seq);
        serial::write(b"\n");
    }
}

/// `test-smp-lock-stuck`: a KERNEL-level lock the BSP takes and never frees.
static STUCK_TEST: lock::IrqLock<()> = lock::IrqLock::new(level::KERNEL, ());
static STUCK_GO: AtomicBool = AtomicBool::new(false);
static PARK: AtomicBool = AtomicBool::new(false);
const IPI_VECTOR: usize = 0xf0;
const SPURIOUS_VECTOR: usize = 0xff;
const IPI_ROUNDS: u32 = 3;

// IPI: count on this CPU (GS-anchored), EOI to this CPU's own xAPIC, return.
// Spurious: architecturally no EOI. Both run with IF cleared by the gate.
// Every stub that uses GS starts with `arch::gs_entry!` and returns through
// `arch::gs_exit!` (audit F8): from CPL3, GS_BASE holds what ring 3 chose.
// Right after the prologue and right before the epilogue each counts its
// nesting in `irq_depth` (LD-1); the interrupted flags are in the CPU frame.
global_asm!(
    ".pushsection .text.nos_smp_ipi,\"ax\"",
    ".globl nos_smp_ipi_entry",
    "nos_smp_ipi_entry:",
    arch::gs_entry!("8"),
    "    inc dword ptr gs:[{depth}]",
    "    push rax",
    "    mov rax, gs:[0]",
    "    lock inc dword ptr [rax + 12]",
    "    mov dword ptr [{eoi}], 0",
    "    pop rax",
    "    dec dword ptr gs:[{depth}]",
    arch::gs_exit!("nos_iret_user_ipi"),
    // Touches neither memory nor GS; a CPL3 return keeps the user GS (lint).
    ".globl nos_smp_spurious_entry",
    "nos_smp_spurious_entry:",
    "    iretq",
    // Shootdown: invalidate both published VAs on THIS CPU, then acknowledge
    // the published generation with a locked xchg (ordered after the
    // serializing invlpg), then EOI. Clobbers nothing visible to the
    // interrupted code.
    ".globl nos_smp_shootdown_entry",
    "nos_smp_shootdown_entry:",
    arch::gs_entry!("8"),
    "    inc dword ptr gs:[{depth}]",
    "    push rax",
    "    push rdx",
    "    mov rax, qword ptr [rip + {va}]",
    "    invlpg [rax]",
    "    mov rax, qword ptr [rip + {va} + 8]",
    "    invlpg [rax]",
    "    mov edx, dword ptr [rip + {generation}]",
    "    mov rax, gs:[0]",
    "    xchg dword ptr [rax + 16], edx",
    "    mov dword ptr [{eoi}], 0",
    "    pop rdx",
    "    pop rax",
    "    dec dword ptr gs:[{depth}]",
    arch::gs_exit!("nos_iret_user_shootdown"),
    ".popsection",
    eoi = const paging::LAPIC_WINDOW + 0xb0,
    va = sym NOS_SMP_SHOOTDOWN_VA,
    generation = sym NOS_SMP_SHOOTDOWN_GEN,
    depth = const core::mem::offset_of!(CpuLocal, irq_depth),
);
extern "C" {
    static nos_iret_user_timer: u8;
    fn nos_smp_ipi_entry();
    fn nos_smp_spurious_entry();
    fn nos_smp_shootdown_entry();
    fn nos_smp_timer_entry();
    fn nos_smp_resched_entry();
}

// Trampoline layout (offsets inside the copied SIPI page):
//   0  cli; short jmp to code      8  CR3 (patched)
//  16  GDTR (limit, base32)       24  far pointer to entry32 (selector 0x08)
//  32  16-bit code, ESI = page linear base for the 32-bit stage
global_asm!(
    ".pushsection .rodata.nos_smp_trampoline,\"a\"",
    ".code16",
    ".balign 4096",
    ".globl nos_smp_trampoline_start",
    "nos_smp_trampoline_start:",
    "    cli",
    "    jmp 1f",
    "    .org nos_smp_trampoline_start + 8",
    "    .long 0",
    "    .org nos_smp_trampoline_start + 16",
    "    .word nos_smp_gdt_end - nos_smp_gdt - 1",
    "    .long nos_smp_gdt",
    "    .org nos_smp_trampoline_start + 24",
    "    .long nos_smp_entry32",
    "    .word 0x08",
    "    .org nos_smp_trampoline_start + 32",
    "1:",
    "    cld",
    "    movw %cs, %ax",
    "    movw %ax, %ds",
    "    xorl %esi, %esi",
    "    movw %ax, %si",
    "    shll $4, %esi",
    "    lgdtl 16",
    "    movl %cr0, %eax",
    "    orl $1, %eax",
    "    movl %eax, %cr0",
    "    ljmpl *24",
    ".globl nos_smp_trampoline_end",
    "nos_smp_trampoline_end:",
    // Accessed bits preset: the CPU never writes this read-only kernel GDT.
    ".balign 16",
    "nos_smp_gdt:",
    "    .quad 0",
    "    .quad 0x00cf9b000000ffff",
    "    .quad 0x00cf93000000ffff",
    "    .quad 0x00af9b000000ffff",
    ".globl nos_smp_gdt_end",
    "nos_smp_gdt_end:",
    ".popsection",
    ".pushsection .text.nos_smp_ap,\"ax\"",
    ".code32",
    "nos_smp_entry32:",
    "    movw $0x10, %ax",
    "    movw %ax, %ds",
    "    movw %ax, %es",
    "    movw %ax, %ss",
    "    xorl %eax, %eax",
    "    movw %ax, %fs",
    "    movw %ax, %gs",
    "    movl %cr4, %eax",
    "    orl $0x20, %eax",
    "    movl %eax, %cr4",
    "    movl 8(%esi), %eax",
    "    movl %eax, %cr3",
    "    movl $0xc0000080, %ecx",
    "    rdmsr",
    "    orl $0x900, %eax",
    "    wrmsr",
    "    movl %cr0, %eax",
    "    andl $0x9fffffff, %eax",
    "    orl $0x80010001, %eax",
    "    movl %eax, %cr0",
    "    ljmpl $0x18, $nos_smp_entry64",
    ".code64",
    "nos_smp_entry64:",
    "    movl $1, %eax",
    "    cpuid",
    "    shrl $24, %ebx",
    "    leaq {contexts}(%rip), %rsi",
    "    movl ${count}, %ecx",
    "2:",
    "    cmpl %ebx, 4(%rsi)",
    "    jne 3f",
    "    movl ${starting}, %eax",
    "    movl ${claimed}, %edx",
    "    lock cmpxchgl %edx, (%rsi)",
    "    jne 3f",
    "    movq 8(%rsi), %rsp",
    "    movq %rsi, %rdi",
    "    call {main}",
    "3:",
    "    addq ${size}, %rsi",
    "    decl %ecx",
    "    jnz 2b",
    "4:",
    "    cli",
    "    hlt",
    "    jmp 4b",
    ".popsection",
    contexts = sym NOS_SMP_AP_CONTEXTS,
    main = sym smp_ap_main,
    count = const MAX_APS,
    starting = const STARTING,
    claimed = const CLAIMED,
    size = const CONTEXT_SIZE,
    options(att_syntax),
);
extern "C" {
    static nos_smp_trampoline_start: u8;
    static nos_smp_trampoline_end: u8;
    static nos_smp_gdt_end: u8;
}
const TRAMPOLINE_CR3: u64 = 8;

/// Outcome of the pre-CR3-switch preparation, reported after KS-03.
pub type Prepared = Result<u64, &'static [u8]>;

/// Call before any paging allocation: the first free ConventionalMemory frame
/// becomes the permanent Kernel-owned SIPI page if it is a valid vector page.
pub fn reserve_trampoline(frames: &mut Frames<FRAME_COUNT>) -> Option<u64> {
    let page = frames.allocate(Owner::Kernel, zero).ok()?;
    // SIPI vectors 0x01..=0x9f; 0xa0.. is legacy VGA/BIOS space.
    if (0x1000..0xa0000).contains(&page) {
        Some(page)
    } else {
        frames.free(page, Owner::Kernel, zero).ok()?;
        None
    }
}

fn rdmsr(msr: u32) -> u64 {
    let (lo, hi): (u32, u32);
    // SAFETY: CPL0 read of an architectural MSR named by the caller.
    unsafe {
        asm!("rdmsr", in("ecx") msr, out("eax") lo, out("edx") hi, options(nostack));
    }
    u64::from(hi) << 32 | u64::from(lo)
}
fn wrmsr(msr: u32, value: u64) {
    // SAFETY: CPL0 write of an architectural MSR with a value copied from the
    // BSP's own validated register state; callers keep IF=0.
    unsafe {
        asm!("wrmsr", in("ecx") msr, in("eax") value as u32, in("edx") (value >> 32) as u32, options(nostack));
    }
}

/// Frames A and B for the shootdown proof, filled while the firmware identity
/// map is active. A is mapped behind SHOOTDOWN_WINDOW and also identity-mapped
/// RW/NX so the BSP can scrub it after CR3 switch; B is mapped only at remap.
fn prepare_shootdown(
    frames: &mut Frames<FRAME_COUNT>,
    paging: &mut paging::Paging,
) -> Result<(), kernel_core::Error> {
    let old = frames.allocate(Owner::Kernel, zero)?;
    let new = frames.allocate(Owner::Kernel, zero)?;
    for (pa, pattern) in [(old, OLD_PATTERN), (new, NEW_PATTERN)] {
        // SAFETY: Fresh exclusive Kernel-owned frames, identity-mapped by the
        // still-active firmware CR3; exactly one page each is written.
        unsafe { ptr::write_bytes(pa as *mut u8, pattern, PAGE as usize) };
    }
    paging.map(frames, paging::SHOOTDOWN_WINDOW, old, true, false);
    paging.map(frames, old, old, true, false);
    SHOOTDOWN_FRAMES[0].store(old, Ordering::Relaxed);
    SHOOTDOWN_FRAMES[1].store(new, Ordering::Relaxed);
    Ok(())
}

fn read_window() -> u32 {
    // SAFETY: SHOOTDOWN_WINDOW is a permanent supervisor RW/NX kernel mapping
    // of a Kernel-owned frame (A or B); a byte read has no side effects. Which
    // frame a CPU reaches depends on its TLB, which is the property under test.
    u32::from(unsafe { (paging::SHOOTDOWN_WINDOW as *const u8).read_volatile() })
}

/// Before `map_tables`/`activate`: validate IA32_APIC_BASE, map its exact page
/// UC/NX and write the SIPI page through the still-active firmware identity map.
pub fn prepare(
    frames: &mut Frames<FRAME_COUNT>,
    paging: &mut paging::Paging,
    trampoline: Option<u64>,
) -> Prepared {
    let page = trampoline.ok_or(b"NO_TRAMPOLINE".as_slice())?;
    let apic_base = rdmsr(0x1b);
    if apic_base & (1 << 8) == 0 {
        return Err(b"NOT_BSP");
    }
    if apic_base & (1 << 11) == 0 {
        return Err(b"LAPIC_DISABLED");
    }
    if apic_base & (1 << 10) != 0 {
        return Err(b"X2APIC_MODE");
    }
    let base = apic_base & 0x000f_ffff_ffff_f000;
    paging
        .map_lapic(frames, base)
        .map_err(|_| b"LAPIC_MAPPING".as_slice())?;
    prepare_shootdown(frames, paging).map_err(|_| b"SHOOTDOWN_FRAMES".as_slice())?;
    // Step 5d-1: one PML4 frame per test address space, identity-mapped RW/NX
    // so the BSP can fill it after the CR3 switch (`schedule`).
    // shortcut: the two frames are never freed; a real process space comes in 5d-2.
    for root in &SPACE_ROOT {
        let pa = frames
            .allocate(Owner::Kernel, zero)
            .map_err(|_| b"SCHED_ROOTS".as_slice())?;
        paging.map(frames, pa, pa, true, false);
        root.store(pa, Ordering::Relaxed);
    }
    if GS_FIXTURE {
        prepare_gs(frames, paging).map_err(|_| b"GS_FIXTURE".as_slice())?;
    }
    let root = paging.root;
    // Address computation of the linked trampoline image only.
    let (start, end) = (
        ptr::addr_of!(nos_smp_trampoline_start) as u64,
        ptr::addr_of!(nos_smp_trampoline_end) as u64,
    );
    let length = end - start;
    // The 32-bit stages address the GDT and entry32 (both below gdt_end in this
    // image) with 32-bit absolute operands; the kernel links below 0x03000000.
    let image_end = ptr::addr_of!(nos_smp_gdt_end) as u64;
    if length > PAGE
        || root >= 1 << 32
        || !root.is_multiple_of(PAGE)
        || end >= 1 << 32
        || image_end >= 1 << 32
    {
        return Err(b"TRAMPOLINE_LAYOUT");
    }
    // SAFETY: `page` is an exclusive Kernel-owned ConventionalMemory frame below
    // 1 MiB, never mapped by our tables; the firmware CR3 that is still active
    // identity-maps it. The source is our own read-only image. Exactly `length`
    // <= one page is copied, then the aligned CR3 slot inside it is written.
    unsafe {
        ptr::copy_nonoverlapping(start as *const u8, page as *mut u8, length as usize);
        ((page + TRAMPOLINE_CR3) as *mut u32).write_volatile(root as u32);
    }
    Ok(page)
}

fn lapic_read(offset: u64) -> u32 {
    // SAFETY: LAPIC_WINDOW is the exact UC/NX supervisor mapping of the validated
    // IA32_APIC_BASE page; offset is a fixed aligned register below 4 KiB.
    unsafe { ((paging::LAPIC_WINDOW + offset) as *const u32).read_volatile() }
}
fn lapic_write(offset: u64, value: u32) {
    // SAFETY: As lapic_read. The window decodes to the *executing* CPU's own
    // xAPIC: the BSP alone writes its ICR, an AP only its own SVR/TPR/LVT/EOI.
    unsafe { ((paging::LAPIC_WINDOW + offset) as *mut u32).write_volatile(value) }
}
fn icr_idle() -> bool {
    (0..1_000_000).any(|_| {
        spin_loop();
        lapic_read(0x300) & (1 << 12) == 0
    })
}
fn send_ipi(apic_id: u32, low: u32) -> bool {
    // Every context/register publication precedes the IPI that makes an AP read
    // it; volatile MMIO is not ordered against atomics by the language alone.
    fence(Ordering::SeqCst);
    if !icr_idle() {
        return false;
    }
    lapic_write(0x310, apic_id << 24);
    lapic_write(0x300, low);
    icr_idle()
}
const IPI_INIT: u32 = 0x0000_4500;
const IPI_STARTUP: u32 = 0x0000_4600;

fn inb(port: u16) -> u8 {
    let value: u8;
    // SAFETY: PIT/port-B access by the sole BSP owner in this profile.
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack));
    }
    value
}
fn outb(port: u16, value: u8) {
    // SAFETY: As inb; native-smp excludes every other PIT/port-B user.
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
    }
}
/// Busy-wait on PIT channel 2 (mode 0, gated through port 0x61). At most
/// 0xffff ticks (~54.9 ms); callers use <= 10 ms. Bounded and fail-closed: a
/// missing timer, or a floating bus that already reads OUT2 high after arming,
/// returns false instead of collapsing the wait or hanging the boot.
fn wait_us(us: u32) -> bool {
    let ticks = (u64::from(us) * 1_193_182)
        .div_ceil(1_000_000)
        .clamp(2, 0xffff) as u16;
    let control = inb(0x61) & 0x0c;
    outb(0x61, control);
    outb(0x43, 0xb0);
    outb(0x42, ticks as u8);
    outb(0x42, (ticks >> 8) as u8);
    // Mode 0 drives OUT2 low on the control word; high now means no real PIT.
    if inb(0x61) & 0x20 != 0 {
        return false;
    }
    outb(0x61, control | 0x01);
    // PAUSE also lets a round-robin hypervisor (single-thread TCG) run the APs.
    let expired = (0..2_000_000u32).any(|_| {
        spin_loop();
        inb(0x61) & 0x20 != 0
    });
    outb(0x61, control);
    expired
}

const ITERATIONS: u64 = 20_000;
static ATOMIC_TOTAL: AtomicU64 = AtomicU64::new(0);
static LOCKED_TOTAL: TicketLock<u64> = TicketLock::new(0);
static INSIDE: AtomicU32 = AtomicU32::new(0);
static VIOLATIONS: AtomicU32 = AtomicU32::new(0);

fn stress() {
    for _ in 0..ITERATIONS {
        ATOMIC_TOTAL.fetch_add(1, Ordering::Relaxed);
        let mut total = LOCKED_TOTAL.lock();
        if INSIDE.fetch_add(1, Ordering::Relaxed) != 0 {
            VIOLATIONS.fetch_add(1, Ordering::Relaxed);
        }
        // Split read/write widens the window a broken lock would lose.
        // SAFETY: Exclusive &mut from the guard; volatile keeps both accesses.
        let value = unsafe { ptr::read_volatile(&*total) };
        spin_loop();
        // SAFETY: Same exclusive location.
        unsafe { ptr::write_volatile(&mut *total, value + 1) };
        INSIDE.fetch_sub(1, Ordering::Relaxed);
        drop(total);
    }
}

fn park() -> ! {
    loop {
        // SAFETY: Terminal AP state: IF=0, nothing is resumed afterwards.
        unsafe { asm!("cli", "hlt", options(nomem, nostack)) };
    }
}

fn control_registers() -> [u64; REGISTERS] {
    let (cr0, cr3, cr4): (u64, u64, u64);
    // SAFETY: CPL0 read-only register queries.
    unsafe {
        asm!("mov {}, cr0", out(reg) cr0, options(nomem, nostack));
        asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
        asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack));
    }
    [cr0, cr3, cr4, rdmsr(0xc000_0080), rdmsr(0x277)]
}
fn initial_apic_id() -> u32 {
    // SAFETY: CPUID leaf 1 is available on every x86-64 CPU.
    unsafe { core::arch::x86_64::__cpuid(1).ebx >> 24 }
}

#[no_mangle]
extern "C" fn smp_ap_main(context: &'static ApContext) -> ! {
    // The assembly scan only passes a pointer into NOS_SMP_AP_CONTEXTS.
    let slot =
        (ptr::from_ref(context) as usize - NOS_SMP_AP_CONTEXTS.as_ptr() as usize) / CONTEXT_SIZE;
    if slot >= MAX_APS {
        park();
    }
    arch::install_ap(
        // slot < MAX_APS checked above; this AP is the slot's sole user
        // because it holds the matching claimed context.
        ap_tables(slot),
        stack_base(slot, MAIN) + STACK_TOP,
        stack_base(slot, DOUBLE) + STACK_TOP,
        stack_base(slot, NMI) + STACK_TOP,
    );
    let local = &CPU_LOCAL[slot];
    local.slot.store(slot as u32, Ordering::Relaxed);
    local
        .self_ptr
        .store(ptr::from_ref(local) as u64, Ordering::Relaxed);
    wrmsr(0xc000_0101, ptr::from_ref(local) as u64);
    let bsp: [u64; REGISTERS] = core::array::from_fn(|i| BSP_REGISTERS[i].load(Ordering::Acquire));
    wrmsr(0x277, bsp[PAT]);
    wrmsr(0xc000_0080, bsp[EFER]);
    // SAFETY: Adopt the BSP's validated CR4/CR0 (PAE/PG/WP and caching) exactly;
    // the trampoline already runs with the same CR3, PAE, LME and NXE.
    unsafe {
        asm!("mov cr4, {}", in(reg) bsp[CR4], options(nostack));
        asm!("mov cr0, {}", in(reg) bsp[CR0], options(nostack));
    }
    let first = ptr::eq(context, &NOS_SMP_AP_CONTEXTS[0]);
    if cfg!(feature = "test-smp-ap-stall") && first {
        // Claimed but never online: the BSP must time out, INIT and stop.
        loop {
            spin_loop();
        }
    }
    if cfg!(feature = "test-smp-ap-mismatch") && first {
        // A real hardware register difference the BSP must reject.
        // SAFETY: Clearing CR0.WP only weakens this AP, which then halts.
        unsafe {
            asm!("mov {tmp}, cr0", "and {tmp}, {mask}", "mov cr0, {tmp}",
                tmp = out(reg) _, mask = in(reg) !(1u64 << 16), options(nostack));
        }
    }
    let observed = control_registers();
    let apic = initial_apic_id();
    for (slot, value) in context.observed.iter().zip(observed) {
        slot.store(value, Ordering::Relaxed);
    }
    context.observed_apic.store(apic, Ordering::Relaxed);
    let apic_base = rdmsr(0x1b);
    let matches = observed == bsp
        && apic == context.apic_id.load(Ordering::Relaxed)
        && apic_base & (1 << 8) == 0
        && apic_base | (1 << 8) == BSP_APIC_BASE.load(Ordering::Acquire);
    context
        .state
        .store(if matches { ONLINE } else { MISMATCH }, Ordering::Release);
    if !matches {
        park();
    }
    while !GO.load(Ordering::Acquire) {
        spin_loop();
    }
    stress();
    serial_lines();
    context.state.store(DONE, Ordering::Release);
    ipi_phase(context, local, first)
}

/// Receive bounded unicast IPIs on vector 0xf0 until PARK: TPR 0, every
/// implemented local LVT masked, spurious vector 0xff. An external (IOAPIC or
/// MSI) source aimed at this APIC ID could still raise another vector; it hits
/// an absent gate (#NP) and the shared handler stops the VM: terminal, not open.
fn ipi_phase(context: &ApContext, local: &CpuLocal, first: bool) -> ! {
    // Version bits 23:16 = highest LVT index: 3 base entries (timer, LINT0/1,
    // error), then performance (>=4), thermal (>=5) and CMCI (>=6).
    let max_lvt = (lapic_read(0x30) >> 16) & 0xff;
    for (lvt, minimum) in [
        (0x320, 0),
        (0x350, 0),
        (0x360, 0),
        (0x370, 0),
        (0x340, 4),
        (0x330, 5),
        (0x2f0, 6),
    ] {
        if max_lvt >= minimum {
            lapic_write(lvt, 1 << 16);
        }
    }
    // TPR 0 accepts every class. The shootdown-deaf test raises it to class 0xe
    // so only the 0xe0 shootdown vector is held back, never the 0xf0 IPIs.
    let tpr = if cfg!(feature = "test-smp-ap-shootdown-deaf") && first {
        0xe0
    } else {
        0
    };
    lapic_write(0x80, tpr);
    lapic_write(0xf0, 0x100 | SPURIOUS_VECTOR as u32);
    // Cache the window's current translation (frame A) in this CPU's TLB.
    local.before.store(read_window(), Ordering::Relaxed);
    context.state.store(IPI_READY, Ordering::Release);
    if cfg!(feature = "test-smp-ap-ipi-deaf") && first {
        // Online but never accepts an interrupt: the BSP must quarantine all.
        park();
    }
    loop {
        // Checked before PARK so the test cannot race into a normal park.
        if cfg!(feature = "test-smp-ap-double-fault")
            && first
            && local.ipis.load(Ordering::Acquire) != 0
        {
            // A real overflow into this AP's lower main guard; #PF cannot push
            // its frame there, so the CPU must switch to this AP's IST1 stack.
            let guard = stack_base(0, MAIN) + 2048;
            // SAFETY: Destructive test-only terminal path; no Rust code resumes
            // on the invalid stack. The #DF handler exits the VM.
            unsafe {
                asm!("mov rsp, {guard}", "push 0", "ud2", guard = in(reg) guard, options(noreturn));
            }
        }
        // Visibility probe: read the window through whatever this CPU's TLB
        // now holds, then publish the result (Release) with the probe number.
        let probe = PROBE.load(Ordering::Acquire);
        if probe != local.probed.load(Ordering::Relaxed) {
            local.after.store(read_window(), Ordering::Relaxed);
            local.probed.store(probe, Ordering::Release);
        }
        // Preemption phase: arm once on RUN; once STOP has brought this CPU
        // back to its home context (current 0 after a tick), report RESUMED.
        match (
            PREEMPT.load(Ordering::Acquire),
            local.preempt.load(Ordering::Relaxed),
        ) {
            (PREEMPT_RUN, 0) => arm_preemption(local, first),
            (PREEMPT_STOP, ARMED)
                if local.current.load(Ordering::Acquire) == 0
                    && local.ticks.load(Ordering::Relaxed) != 0 =>
            {
                local.preempt.store(RESUMED, Ordering::Release);
            }
            _ => {}
        }
        // Scheduler phase (step 5d-1): start from home; the first tick dispatches.
        if SCHED.load(Ordering::Acquire) == PREEMPT_RUN
            && local.sched_state.load(Ordering::Relaxed) == 0
        {
            local.sched_cur.store(HOME, Ordering::Relaxed);
            local.sched_state.store(ARMED, Ordering::Release);
            start_timer();
        }
        // Audit F8 fixture: a one-shot tick from here enters ring 3.
        if GS_FIXTURE
            && GS.load(Ordering::Acquire) == PREEMPT_RUN
            && local.gs_state.load(Ordering::Relaxed) == 0
        {
            local.gs_state.store(GS_ARMED, Ordering::Release);
            lapic_write(TIMER_DIVIDE, 0x3);
            lapic_write(LVT_TIMER, TIMER_VECTOR as u32);
            lapic_write(TIMER_INITIAL, 20_000);
        }
        // Scratch phase (step 5b-2): this AP's share, concurrent with the others.
        if SCRATCH_RUN.load(Ordering::Acquire) && local.scratched.load(Ordering::Relaxed) == 0 {
            scratch_work(local.slot.load(Ordering::Relaxed) as usize + 1);
            local.scratched.store(1, Ordering::Release);
        }
        // Pair phase (audit F4): this AP's lock_pair rounds, concurrent.
        if PAIR_RUN.load(Ordering::Acquire) && local.paired.load(Ordering::Relaxed) == 0 {
            pair_work(local.slot.load(Ordering::Relaxed) as usize + 1);
            local.paired.store(1, Ordering::Release);
        }
        // Spin phase (step 5b): wait for the KERNEL-level lock the BSP holds.
        // IF is 0 from the store of 1 on; only the lock's own window opens it.
        if SPIN.load(Ordering::Acquire) && local.spun.load(Ordering::Relaxed) == 0 {
            local.spun.store(1, Ordering::Release);
            *SPIN_LOCK.lock_irqsave() += 1;
            local.spun.store(2, Ordering::Release);
        }
        if cfg!(feature = "test-smp-lock-stuck") && first && STUCK_GO.load(Ordering::Acquire) {
            drop(STUCK_TEST.lock_irqsave());
        }
        if PARK.load(Ordering::Acquire) {
            break;
        }
        // SAFETY: STI's one-instruction shadow makes `sti; hlt` atomic, so an
        // IPI that becomes pending after the PARK check still wakes this CPU.
        // Local sources are masked; see the function comment for others.
        unsafe { asm!("sti", "hlt", "cli") };
    }
    context.state.store(PARKED, Ordering::Release);
    park()
}

/// Test-only markers. `test-smp-ap-nmi`/`test-smp-ap-double-fault`/
/// `test-smp-gs-nmi-user`: an NMI or #DF that the CPU delivered on an AP's own
/// IST stack; the AP is identified by that stack's address, never by GS (an
/// NMI from CPL3 still runs with the user's GS_BASE). `test-smp-gs-iret-fault`:
/// a faulting `iretq` to CPL3 whose GS was fixed up. `test-smp-gs-user`: a
/// supervisor #PF near address 0 is a GS-relative read through a user GS.
/// `test-smp-shared-scratch`: a supervisor #PF in scratch slot 0.
pub fn ist_fault_marker(
    vector: u64,
    error: u64,
    rip: u64,
    cs: u64,
    cr2: u64,
    frame: *const u64,
) -> Option<&'static [u8]> {
    // Control only: another CPU cleared the shared slot's PTE under this
    // access. That is the crossing too; report it once, never as a fault.
    if cfg!(feature = "test-smp-shared-scratch")
        && vector == 14
        && cs == 8
        && (paging::SCRATCH..paging::SCRATCH + PAGE).contains(&cr2)
    {
        scratch_cross();
    }
    if cfg!(feature = "test-smp-gs-user") && vector == 14 && cs == 8 && cr2 < PAGE {
        serial::write(b"SMP:GS:HIJACK\n");
        return None;
    }
    if cfg!(feature = "test-smp-gs-iret-fault")
        && matches!(vector, 11..=13)
        && cs == 8
        && rip == ptr::addr_of!(nos_iret_user_timer) as u64
        && this_cpu().is_some()
    {
        return Some(b"SMP:GS:IRET_FIXUP:PASS\n");
    }
    let nmi_user = cfg!(feature = "test-smp-gs-nmi-user") && cs & 3 == 3;
    if !(nmi_user
        || cfg!(any(
            feature = "test-smp-ap-nmi",
            feature = "test-smp-ap-double-fault"
        )) && cs == 8)
    {
        return None;
    }
    let at = frame as u64;
    let within = |slot, kind| {
        let base = stack_base(slot, kind);
        at >= base + 4096 && at + 22 * 8 <= base + STACK_TOP
    };
    let slot = (0..MAX_APS).find(|&slot| within(slot, NMI) || within(slot, DOUBLE))?;
    if (nmi_user || cfg!(feature = "test-smp-ap-nmi")) && vector == 2 && within(slot, NMI) {
        return Some(b"SMP:FAULT:AP_NMI_IST:PASS\n");
    }
    if cfg!(feature = "test-smp-ap-double-fault")
        && vector == 8
        && error == 0
        && within(slot, DOUBLE)
    {
        // SAFETY: The normalized exception frame on this live IST stack holds
        // the interrupted RSP at index 20 (15 GPRs, vector, error, RIP, CS, RFLAGS).
        let old_rsp = unsafe { frame.add(20).read() };
        let main = stack_base(slot, MAIN);
        if (main..main + 4096).contains(&old_rsp) {
            return Some(b"SMP:FAULT:AP_DF_IST:PASS\n");
        }
    }
    None
}

fn decimal(mut value: u64) {
    let mut digits = [0u8; 20];
    let mut index = digits.len();
    loop {
        index -= 1;
        digits[index] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    serial::write(&digits[index..]);
}
fn unavailable(reason: &[u8]) {
    serial::write(b"SMP:UNAVAILABLE:");
    serial::write(reason);
    serial::write(b"\n");
}

enum Start {
    Online,
    Timeout,
    Mismatch,
    Ipi,
}

/// Returns the outcome and, for every non-online outcome, whether the CPU was
/// confirmed reset with INIT (false means it may still be running).
fn bring_up(slot: usize, apic_id: u32, vector: u32) -> (Start, bool) {
    let context = &NOS_SMP_AP_CONTEXTS[slot];
    context
        .stack_top
        .store(stack_base(slot, MAIN) + STACK_TOP, Ordering::Relaxed);
    context.apic_id.store(apic_id, Ordering::Relaxed);
    context.state.store(STARTING, Ordering::Release);
    let mut result = Start::Timeout;
    if send_ipi(apic_id, IPI_INIT) && wait_us(10_000) && send_ipi(apic_id, IPI_STARTUP | vector) {
        wait_us(200);
        if context.state.load(Ordering::Acquire) == STARTING
            && !send_ipi(apic_id, IPI_STARTUP | vector)
        {
            result = Start::Ipi;
        }
        wait_us(200);
        for _ in 0..100 {
            match context.state.load(Ordering::Acquire) {
                ONLINE => return (Start::Online, true),
                MISMATCH => {
                    result = Start::Mismatch;
                    break;
                }
                _ => {
                    wait_us(10_000);
                }
            }
        }
    } else {
        result = Start::Ipi;
    }
    // Fail closed: an unclaimed context is retired by the same CAS the AP uses;
    // a claimed one keeps its stack forever. Either way the CPU is reset (INIT).
    // A failed CAS only means the AP already claimed; INIT handles both cases.
    context
        .state
        .compare_exchange(STARTING, ABANDONED, Ordering::AcqRel, Ordering::Acquire)
        .ok();
    let quarantined = send_ipi(apic_id, IPI_INIT);
    context.state.store(ABANDONED, Ordering::Release);
    (result, quarantined)
}

/// After KS-03: report the MADT, start each enabled AP in order and run the
/// bounded synchronization self-test. Every failure keeps a working 1-CPU boot.
pub fn start(
    prepared: Prepared,
    madt: &Madt,
    frames: &mut Frames<FRAME_COUNT>,
    paging: &mut paging::Paging,
) {
    BSP_LOCAL
        .self_ptr
        .store(ptr::from_ref(&BSP_LOCAL) as u64, Ordering::Relaxed);
    wrmsr(0xc000_0101, ptr::from_ref(&BSP_LOCAL) as u64);
    // K-2: every CPU's scratch slot, before any AP can start; from here on
    // page tables are reached through the BSP's slot (index 0).
    paging.begin_scratch(frames);
    // GS-10: without FSGSBASE ring 3 cannot write GS_BASE directly, and
    // without SCE there is no `syscall` entry, which would need its own
    // unconditional swapgs. Every AP adopts exactly these registers and is
    // rejected (MISMATCH) unless it reads them back identically.
    let registers = control_registers();
    if registers[CR4] & (1 << 16) != 0 || registers[EFER] & 1 != 0 {
        serial::write(b"SMP:GS:PRECONDITION\n");
        serial::fail(b"SMP_GS_PRECONDITION");
    }
    match madt.status {
        MadtStatus::Missing => return unavailable(b"NO_MADT"),
        MadtStatus::Invalid => return unavailable(b"INVALID_MADT"),
        MadtStatus::Valid => {}
    }
    calibrate_tsc();
    let cpus = madt.cpus();
    serial::write(b"SMP:MADT:CPUS=");
    decimal(cpus.len() as u64);
    serial::write(b":DISABLED=");
    decimal(u64::from(madt.disabled));
    serial::write(b":HOTPLUG=");
    decimal(u64::from(madt.online_capable));
    serial::write(b"\n");
    let page = match prepared {
        Ok(page) => page,
        Err(reason) => return unavailable(reason),
    };
    if rdmsr(0x1b) & 0x000f_ffff_ffff_f000 != madt.local_apic_base {
        return unavailable(b"LAPIC_BASE_MISMATCH");
    }
    let bsp_id = lapic_read(0x20) >> 24;
    if bsp_id != initial_apic_id() {
        return unavailable(b"BSP_ID");
    }
    if !cpus.iter().any(|cpu| cpu.apic_id == bsp_id) {
        return unavailable(b"BSP_NOT_IN_MADT");
    }
    if !wait_us(1_000) {
        return unavailable(b"TIMER");
    }
    let svr = lapic_read(0xf0);
    if svr & 0x100 == 0 {
        // Software-enable with spurious vector 0xff; IF=0 delivers nothing.
        lapic_write(0xf0, svr | 0x1ff);
    }
    for (slot, value) in BSP_REGISTERS.iter().zip(control_registers()) {
        slot.store(value, Ordering::Release);
    }
    BSP_APIC_BASE.store(rdmsr(0x1b), Ordering::Release);
    // IDT is complete before the first AP can load it.
    arch::smp_gate(IPI_VECTOR, nos_smp_ipi_entry as usize as u64);
    arch::smp_gate(SPURIOUS_VECTOR, nos_smp_spurious_entry as usize as u64);
    arch::smp_gate(SHOOTDOWN_VECTOR, nos_smp_shootdown_entry as usize as u64);
    arch::smp_gate(TIMER_VECTOR, nos_smp_timer_entry as usize as u64);
    arch::smp_gate(RESCHED_VECTOR, nos_smp_resched_entry as usize as u64);
    let vector = (page / PAGE) as u32;
    let (mut online, mut x2apic, mut capacity) = (0usize, 0u64, 0u64);
    for cpu in cpus.iter().filter(|cpu| cpu.apic_id != bsp_id) {
        if cpu.apic_id >= 0xff {
            x2apic += 1;
            continue;
        }
        if online == MAX_APS {
            capacity += 1;
            continue;
        }
        let (outcome, quarantined) = bring_up(online, cpu.apic_id, vector);
        serial::write(b"SMP:AP:");
        decimal(u64::from(cpu.apic_id));
        serial::write(match outcome {
            Start::Online => b":ONLINE\n".as_slice(),
            Start::Timeout => b":TIMEOUT\n",
            Start::Mismatch => b":MISMATCH\n",
            Start::Ipi => b":IPI_ERROR\n",
        });
        if !matches!(outcome, Start::Online) {
            if !quarantined {
                // The AP may still be running; GO must never be published.
                serial::fail(b"SMP_AP_QUARANTINE");
            }
            serial::write(b"SMP:BRINGUP:STOPPED\n");
            break;
        }
        online += 1;
    }
    if x2apic != 0 {
        serial::write(b"SMP:SKIPPED:X2APIC=");
        decimal(x2apic);
        serial::write(b"\n");
    }
    if capacity != 0 {
        serial::write(b"SMP:SKIPPED:CAPACITY=");
        decimal(capacity);
        serial::write(b"\n");
    }
    serial::write(b"SMP:ONLINE:");
    decimal(online as u64 + 1);
    serial::write(b"/");
    decimal(cpus.len() as u64);
    serial::write(b"\n");
    GO.store(true, Ordering::Release);
    stress();
    serial_lines();
    let finished = (0..200).any(|_| {
        NOS_SMP_AP_CONTEXTS[..online].iter().all(finished_stress) || {
            wait_us(10_000);
            false
        }
    });
    let expected = ITERATIONS * (online as u64 + 1);
    if !finished {
        serial::write(b"SMP:SYNC:TIMEOUT:DONE=");
        decimal(
            NOS_SMP_AP_CONTEXTS[..online]
                .iter()
                .filter(|context| finished_stress(context))
                .count() as u64,
        );
        serial::write(b":TOTAL=");
        decimal(ATOMIC_TOTAL.load(Ordering::Acquire));
        serial::write(b"\n");
        serial::fail(b"SMP_SYNC_TIMEOUT");
    }
    if ATOMIC_TOTAL.load(Ordering::Acquire) != expected || *LOCKED_TOTAL.lock() != expected {
        serial::fail(b"SMP_SYNC_LOST_UPDATE");
    }
    if VIOLATIONS.load(Ordering::Acquire) != 0 {
        serial::fail(b"SMP_SYNC_EXCLUSION");
    }
    // Every CPU finished its lines (DONE); wholeness is the host's check.
    serial::write(b"SMP:SERIAL:PASS\n");
    serial::write(b"SMP:SYNC:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    ipis(online, cpus.len(), frames, paging);
}

/// DONE is stored (Release) after the AP's last stress write; the AP may then
/// already have advanced to IPI_READY, which is only reachable through DONE.
fn finished_stress(context: &ApContext) -> bool {
    matches!(context.state.load(Ordering::Acquire), DONE | IPI_READY)
}

const IPI_FIXED: u32 = 0x0000_4000;
const IPI_NMI: u32 = 0x0000_4400;

/// Poll `ready` every millisecond, at most `limit` ms (PIT-bounded).
fn within_ms(limit: u32, ready: impl Fn() -> bool) -> bool {
    (0..limit).any(|_| {
        ready() || {
            wait_us(1_000);
            false
        }
    })
}

/// A CPU that cannot take a directed interrupt cannot join any later
/// shootdown protocol: reset every AP with INIT and continue on the BSP.
fn quarantine_all(online: usize, total: usize) {
    for context in &NOS_SMP_AP_CONTEXTS[..online] {
        if !send_ipi(context.apic_id.load(Ordering::Relaxed), IPI_INIT) {
            serial::fail(b"SMP_AP_QUARANTINE");
        }
        context.state.store(ABANDONED, Ordering::Release);
    }
    // ICR idle is acceptance by the bus, not completion of INIT in the target;
    // allow it to take effect before the BSP reports one CPU. A stray store by
    // a resetting AP only touches its own context, which nothing reads anymore.
    wait_us(1_000);
    serial::write(b"SMP:QUARANTINED:APS=");
    decimal(online as u64);
    serial::write(b"\nSMP:ONLINE:1/");
    decimal(total as u64);
    serial::write(b"\n");
}

/// Unicast only: a broadcast shorthand would also reach CPUs still owned by
/// firmware (capacity-skipped or never started) with unknown interrupt state.
fn ipis(
    online: usize,
    total: usize,
    frames: &mut Frames<FRAME_COUNT>,
    paging: &mut paging::Paging,
) {
    let aps = &NOS_SMP_AP_CONTEXTS[..online];
    if !within_ms(1_000, || {
        aps.iter()
            .all(|context| context.state.load(Ordering::Acquire) == IPI_READY)
    }) {
        serial::write(b"SMP:IPI:NOT_READY\n");
        return quarantine_all(online, total);
    }
    for round in 1..=IPI_ROUNDS {
        for (slot, context) in aps.iter().enumerate() {
            let apic = context.apic_id.load(Ordering::Relaxed);
            if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32)
                || !within_ms(100, || {
                    CPU_LOCAL[slot].ipis.load(Ordering::Acquire) >= round
                })
            {
                serial::write(b"SMP:IPI:TIMEOUT:APIC=");
                decimal(u64::from(apic));
                serial::write(b"\n");
                return quarantine_all(online, total);
            }
            if cfg!(feature = "test-smp-ap-double-fault") && slot == 0 && round == 1 {
                // That AP now faults on its own IST1 stack and ends the VM.
                within_ms(1_000, || false);
                serial::fail(b"SMP_AP_DF_NOT_ON_IST");
            }
        }
    }
    // During the rounds each IPI was delivered exactly once (none lost or
    // duplicated). Later arrivals, e.g. the park IPI, are not counted here.
    if CPU_LOCAL[..online]
        .iter()
        .any(|local| local.ipis.load(Ordering::Acquire) != IPI_ROUNDS)
    {
        serial::fail(b"SMP_IPI_COUNT");
    }
    serial::write(b"SMP:IPI:PASS:ROUNDS=");
    decimal(u64::from(IPI_ROUNDS));
    serial::write(b":CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    if cfg!(feature = "test-smp-lock-stuck") && online != 0 {
        // F5 (DG-1): the first AP must report the wait within LOCK_STUCK_MS.
        let _held = STUCK_TEST.lock_irqsave();
        STUCK_GO.store(true, Ordering::Release);
        send_ipi(
            aps[0].apic_id.load(Ordering::Relaxed),
            IPI_FIXED | IPI_VECTOR as u32,
        );
        within_ms(1_000, || false);
        serial::fail(b"SMP_LOCK_STUCK_MISSED");
    }
    if cfg!(feature = "test-smp-ap-nmi") && online != 0 {
        let apic = aps[0].apic_id.load(Ordering::Relaxed);
        if send_ipi(apic, IPI_NMI) {
            // The AP's IST2 handler must end the VM with its marker.
            within_ms(1_000, || false);
        }
        serial::fail(b"SMP_AP_NMI_NOT_ON_IST");
    }
    if !shootdown(online, total, frames, paging)
        || !preemption(online, total)
        || !spin_window(online, total)
        || !schedule(online, total)
        || !scratch(online, total, frames)
        || !lock_pairs(online, total)
        || !gs_user(online, total)
    {
        return;
    }
    PARK.store(true, Ordering::Release);
    for (slot, context) in aps.iter().enumerate() {
        let apic = context.apic_id.load(Ordering::Relaxed);
        if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32)
            || !within_ms(100, || context.state.load(Ordering::Acquire) == PARKED)
        {
            serial::write(b"SMP:PARK:TIMEOUT:SLOT=");
            decimal(slot as u64);
            serial::write(b"\n");
            return quarantine_all(online, total);
        }
    }
    serial::write(b"SMP:PARKED:APS=");
    decimal(online as u64);
    serial::write(b"\n");
}

/// TLB shootdown proof (ADR-072). Every AP has cached SHOOTDOWN_WINDOW -> A,
/// which also has a bootstrap identity alias. With one shared CR3 any CPU may
/// hold either translation (even speculatively), so the order is:
/// scrub A, clear BOTH PTEs locally (window -> B, identity alias removed),
/// shoot down BOTH VAs on every AP, and only after every ack release both
/// aliases, free A and run the full page-table audit. A missing ack leaves A
/// quarantined forever (owned, aliases held; the audit must then not run, as
/// its metadata deliberately disagrees with the tables) and resets every AP.
/// Returns false after that quarantine. `test-smp-shootdown-skipped` is the
/// control: it omits the IPIs to show that APs then really reach frame A.
fn shootdown(
    online: usize,
    total: usize,
    frames: &mut Frames<FRAME_COUNT>,
    paging: &mut paging::Paging,
) -> bool {
    let locals = &CPU_LOCAL[..online];
    let old = SHOOTDOWN_FRAMES[0].load(Ordering::Relaxed);
    let new = SHOOTDOWN_FRAMES[1].load(Ordering::Relaxed);
    if read_window() != u32::from(OLD_PATTERN)
        || locals
            .iter()
            .any(|local| local.before.load(Ordering::Acquire) != u32::from(OLD_PATTERN))
    {
        serial::fail(b"SMP_SHOOTDOWN_SETUP");
    }
    // SAFETY: `old` is the Kernel-owned frame A with its bootstrap identity
    // RW/NX alias still mapped. Every AP already recorded its pre-remap read
    // (checked above); nothing else reads A, so scrubbing now leaks nothing.
    unsafe { ptr::write_bytes(old as *mut u8, 0, PAGE as usize) };
    if paging.remap_window(frames, new) != Ok(old)
        || paging.clear_identity(old).is_err()
        || read_window() != u32::from(NEW_PATTERN)
    {
        serial::fail(b"SMP_SHOOTDOWN_REMAP");
    }
    let skipped = cfg!(feature = "test-smp-shootdown-skipped");
    if skipped {
        serial::write(b"SMP:SHOOTDOWN:SKIPPED\n");
    } else if let Err(apic) = shootdown_round([paging::SHOOTDOWN_WINDOW, old], online) {
        serial::write(b"SMP:SHOOTDOWN:TIMEOUT:APIC=");
        decimal(u64::from(apic));
        serial::write(b"\nSMP:SHOOTDOWN:FRAME_QUARANTINED\n");
        quarantine_all(online, total);
        return false;
    }
    // Global invalidation of both translations is complete (except, by design,
    // in the control), so A's metadata may be released and the frame reused.
    if paging.release_stale_alias(frames, old).is_err()
        || paging.release_stale_alias(frames, old).is_err()
        // Contents were scrubbed above, before any translation was dropped.
        || frames.free(old, Owner::Kernel, |_| {}).is_err()
    {
        serial::fail(b"SMP_SHOOTDOWN_RECLAIM");
    }
    paging.audit(frames);
    if !skipped {
        serial::write(b"SMP:SHOOTDOWN:PASS:CPUS=");
        decimal(online as u64 + 1);
        serial::write(b"\n");
    }
    probe_visibility(online, total, skipped)
}

/// One shootdown generation for `vas` on every online AP; returns the APIC ID
/// of the first AP that does not acknowledge this exact generation in time.
fn shootdown_round(vas: [u64; 2], online: usize) -> Result<(), u32> {
    for (slot, va) in NOS_SMP_SHOOTDOWN_VA.iter().zip(vas) {
        slot.store(va, Ordering::Release);
    }
    let generation = NOS_SMP_SHOOTDOWN_GEN.fetch_add(1, Ordering::AcqRel) + 1;
    for (context, local) in NOS_SMP_AP_CONTEXTS[..online].iter().zip(&CPU_LOCAL) {
        let apic = context.apic_id.load(Ordering::Relaxed);
        if !send_ipi(apic, IPI_FIXED | SHOOTDOWN_VECTOR as u32)
            || !within_ms(SHOOTDOWN_DEADLINE_MS, || {
                local.shootdown_ack.load(Ordering::Acquire) == generation
            })
        {
            return Err(apic);
        }
    }
    Ok(())
}

/// Every CPU reads the window again through its current TLB; outside the
/// control each must reach frame B.
fn probe_visibility(online: usize, total: usize, skipped: bool) -> bool {
    let locals = &CPU_LOCAL[..online];
    PROBE.store(1, Ordering::Release);
    for (context, local) in NOS_SMP_AP_CONTEXTS[..online].iter().zip(locals) {
        let apic = context.apic_id.load(Ordering::Relaxed);
        if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32)
            || !within_ms(100, || local.probed.load(Ordering::Acquire) == 1)
        {
            serial::write(b"SMP:SHOOTDOWN:PROBE_TIMEOUT\n");
            quarantine_all(online, total);
            return false;
        }
    }
    let stale = locals
        .iter()
        .filter(|local| local.after.load(Ordering::Acquire) != u32::from(NEW_PATTERN))
        .count();
    if skipped {
        serial::write(b"SMP:SHOOTDOWN:STALE:APS=");
        decimal(stale as u64);
        serial::write(b"\n");
    } else if stale != 0 || read_window() != u32::from(NEW_PATTERN) {
        serial::fail(b"SMP_SHOOTDOWN_STALE");
    } else {
        serial::write(b"SMP:SHOOTDOWN:VISIBLE:CPUS=");
        decimal(online as u64 + 1);
        serial::write(b"\n");
    }
    true
}

// Preemption (K06 step 4): the AP's LAPIC timer (vector 0xd0) saves every
// GPR on the interrupted stack, lets `smp_preempt` choose the next stack and
// resumes it. Kernel code is soft-float with no red zone, so the GPRs plus the
// CPU's interrupt frame are the complete thread context. `smp_preempt`
// returns `Resume` in RAX:RDX (SysV, two INTEGER eightbytes). 15 pushes + the
// 5-word frame keep RSP 16-byte aligned for the call (the CPU aligns first).
global_asm!(
    ".pushsection .text.nos_smp_timer,\"ax\"",
    ".globl nos_smp_timer_entry",
    "nos_smp_timer_entry:",
    arch::gs_entry!("8"),
    // 0xe8 joins here, after its own prologue (GS-3: one swapgs per entry).
    // LD-1 counts here for both vectors. A switch resumes another stack, but
    // every exit on this CPU still pairs with one entry on this CPU.
    ".globl nos_smp_timer_body",
    "nos_smp_timer_body:",
    "    inc dword ptr gs:[{depth}]",
    "    push rax", "    push rcx", "    push rdx", "    push rbx", "    push rbp",
    "    push rsi", "    push rdi", "    push r8", "    push r9", "    push r10",
    "    push r11", "    push r12", "    push r13", "    push r14", "    push r15",
    "    cld",
    "    mov rdi, rsp",
    "    call {preempt}",
    "    mov rsp, rax",
    // Audit F1: off the old stack now; let other CPUs resume its thread.
    "    test rdx, rdx",
    "    jz 2f",
    "    mov byte ptr [rdx], 0",
    "2:",
    "    mov dword ptr [{eoi}], 0",
    "    pop r15", "    pop r14", "    pop r13", "    pop r12", "    pop r11",
    "    pop r10", "    pop r9", "    pop r8", "    pop rdi", "    pop rsi",
    "    pop rbp", "    pop rbx", "    pop rdx", "    pop rcx", "    pop rax",
    "    dec dword ptr gs:[{depth}]",
    arch::gs_exit!("nos_iret_user_timer"),
    ".popsection",
    preempt = sym smp_preempt,
    eoi = const paging::LAPIC_WINDOW + 0xb0,
    depth = const core::mem::offset_of!(CpuLocal, irq_depth),
);

const LVT_TIMER: u64 = 0x320;
const TIMER_DIVIDE: u64 = 0x3e0;
const TIMER_INITIAL: u64 = 0x380;
const TIMER_PERIODIC: u32 = 1 << 17;
const MASKED: u32 = 1 << 16;
const MIN_TICKS: u32 = 8;
const MIN_SWITCHES: u32 = 4;

/// This CPU's anchor, identified by IA32_GS_BASE (set once per AP); `None` on
/// the BSP or for any value that is not one of the static anchors.
fn this_cpu() -> Option<&'static CpuLocal> {
    let anchor = rdmsr(0xc000_0101);
    CPU_LOCAL
        .iter()
        .find(|local| ptr::from_ref(*local) as u64 == anchor)
}

/// Any CPU's anchor (BSP or AP) by IA32_GS_BASE; `None` before `start` set it.
fn any_cpu() -> Option<&'static CpuLocal> {
    let anchor = rdmsr(0xc000_0101);
    core::iter::once(&BSP_LOCAL)
        .chain(CPU_LOCAL.iter())
        .find(|local| ptr::from_ref(*local) as u64 == anchor)
}

/// This CPU's index for its scratch slot (K-2): BSP 0, AP slot + 1.
pub fn cpu_index() -> usize {
    anchored_index().unwrap_or_else(|| serial::fail(b"SMP_SCRATCH_ANCHOR"))
}

/// This CPU's index (BSP 0, AP slot + 1), `None` before `start` set its anchor.
pub fn anchored_index() -> Option<usize> {
    any_cpu().map(|local| {
        if ptr::eq(local, &BSP_LOCAL) {
            0
        } else {
            local.slot.load(Ordering::Relaxed) as usize + 1
        }
    })
}

/// True on an AP (its GS anchor is in CPU_LOCAL); false on the BSP.
pub fn on_ap() -> bool {
    this_cpu().is_some()
}

fn anchored() -> &'static CpuLocal {
    any_cpu().unwrap_or_else(|| serial::fail(b"SMP_LOCKDEP_ANCHOR"))
}

/// This CPU's held lock levels (mirror); IF=0, so no handler interleaves.
pub fn lockdep() -> &'static AtomicU64 {
    &anchored().lockdep
}

/// Update this CPU's full lockdep state and its mirror (IF=0 by the caller).
pub fn with_lockdep<R>(f: impl FnOnce(&mut smp_lock::Lockdep) -> R) -> R {
    let local = anchored();
    // SAFETY: this CPU's own cell; every caller (`IrqLock`) runs with IF=0,
    // and `f` is a lockdep method that never re-enters.
    unsafe { local.lockdep_state.update(&local.lockdep, f) }
}

/// LD-1: IDT entries this CPU is nested in (0 = not in a handler).
pub fn irq_depth() -> u32 {
    anchored().irq_depth.load(Ordering::Relaxed)
}

/// Terminal lock-order report (spec §4.4): `SMP:LOCKDEP:ORDER:<held>-><wanted>`.
/// Only the deliberate negative profiles count it as a pass.
pub fn lockdep_order(order: smp_lock::Order) -> ! {
    serial::write(b"SMP:LOCKDEP:ORDER:");
    decimal(u64::from(order.held));
    serial::write(b"->");
    decimal(u64::from(order.wanted));
    serial::write(b"\n");
    if cfg!(any(
        feature = "test-smp-lockdep-order",
        feature = "test-smp-lockpair-order"
    )) {
        serial::exit(0x10);
    }
    serial::fail(b"SMP_LOCKDEP_ORDER")
}

/// Terminal report (LD-1, audit F3): KERNEL/ASPACE wanted inside an interrupt
/// handler, `SMP:LOCKDEP:IN_HANDLER:<level>`.
pub fn lockdep_in_handler(level: u8) -> ! {
    serial::write(b"SMP:LOCKDEP:IN_HANDLER:");
    decimal(u64::from(level));
    serial::write(b"\n");
    if cfg!(feature = "test-smp-lock-in-handler") {
        serial::exit(0x10);
    }
    serial::fail(b"SMP_LOCKDEP_IN_HANDLER")
}

/// Terminal report (audit F2): a guard dropped while a higher level is held,
/// `SMP:LOCKDEP:LIFO:<held>-><released>`. Its saved IF must not be restored.
pub fn lockdep_lifo(order: smp_lock::Order) -> ! {
    serial::write(b"SMP:LOCKDEP:LIFO:");
    decimal(u64::from(order.held));
    serial::write(b"->");
    decimal(u64::from(order.wanted));
    serial::write(b"\n");
    serial::fail(b"SMP_LOCKDEP_LIFO")
}

/// Terminal bounded-spin report (DG-1): `SMP:LOCK:STUCK:<level>:CPU=<n>`,
/// written without waiting (the stuck lock may be `SERIAL` itself).
/// Only the deliberate negative profile counts it as a pass.
pub fn lock_stuck(level: u8) -> ! {
    let mut line = serial::Line::new();
    line.push(b"SMP:LOCK:STUCK:")
        .decimal(u64::from(level))
        .push(b":CPU=");
    match anchored_index() {
        Some(cpu) => line.decimal(cpu as u64),
        None => line.push(b"?"),
    };
    serial::write_terminal(line.push(b"\n").as_bytes());
    if cfg!(feature = "test-smp-lock-stuck") {
        serial::exit(0x10);
    }
    serial::fail(b"SMP_LOCK_STUCK")
}

/// Terminal report (spec §4.4): a context switch with a lock held or awaited.
/// Only the deliberate negative profile counts it as a pass.
fn held_across_switch() -> ! {
    serial::write(b"SMP:LOCKDEP:HELD_ACROSS_SWITCH\n");
    if cfg!(feature = "test-smp-held-across-switch") {
        serial::exit(0x10);
    }
    serial::fail(b"SMP_LOCKDEP_HELD_ACROSS_SWITCH")
}

/// Stack to resume and the address of an `on_cpu` flag (audit F1) the stub
/// clears once it left the old stack; 0 = nothing to clear.
#[repr(C)]
struct Resume {
    rsp: u64,
    left: u64,
}

/// Round-robin over home (0) and the two threads (1, 2). After STOP the timer
/// is masked and the home loop resumes; threads are never run again.
#[no_mangle]
extern "C" fn smp_preempt(rsp: u64) -> Resume {
    let Some(local) = this_cpu() else {
        // A timer interrupt on a CPU without an anchor is impossible by
        // construction (only APs arm it); stop rather than run unknown state.
        if GS.load(Ordering::Acquire) != 0 {
            serial::write(b"SMP:GS:ANCHOR_LOST\n");
        }
        serial::fail(b"SMP_PREEMPT_ANCHOR");
    };
    local.ticks.fetch_add(1, Ordering::Relaxed);
    // LD-1 control: from the first 0xe8 on, this handler takes KERNEL and
    // must stop with SMP:LOCKDEP:IN_HANDLER:10 before spinning.
    if cfg!(feature = "test-smp-lock-in-handler") && local.resched.load(Ordering::Relaxed) != 0 {
        drop(lock::KERNEL.lock_irqsave());
    }
    // Step 5c (INV-5c): a tick can only arrive with locks held or awaited in
    // the 5b `sti; pause; cli` window. Lockdep is per CPU, not per thread, so
    // switching (or STOP's switch home) waits for a later tick.
    let busy = local.lockdep.load(Ordering::Relaxed) != 0;
    if busy && !cfg!(feature = "test-smp-held-across-switch") {
        local.deferred.fetch_add(1, Ordering::Release);
        return Resume { rsp, left: 0 };
    }
    if GS_FIXTURE && GS.load(Ordering::Acquire) != 0 {
        return gs_switch(local, rsp);
    }
    if SCHED.load(Ordering::Acquire) != 0 {
        return sched_switch(local, rsp, busy);
    }
    let current = local.current.load(Ordering::Relaxed) as usize;
    local.saved[current].store(rsp, Ordering::Relaxed);
    let next = if PREEMPT.load(Ordering::Acquire) == PREEMPT_STOP {
        lapic_write(LVT_TIMER, MASKED | TIMER_VECTOR as u32);
        0
    } else if current == 1 {
        2
    } else {
        1
    };
    if next != current {
        if busy {
            held_across_switch();
        }
        local.switches.fetch_add(1, Ordering::Relaxed);
    }
    local.current.store(next as u32, Ordering::Release);
    Resume {
        rsp: local.saved[next].load(Ordering::Relaxed),
        left: 0,
    }
}

/// Each slice parks distinct per-CPU/per-thread values in r12-r15 (and the
/// compiler-chosen input registers), spins long enough to be preempted many
/// times, then checks them: a wrong save/restore in the timer stub would show
/// up as `corrupt`, not just as a passing tick count.
fn thread_body(local: &CpuLocal, index: usize) -> ! {
    let seed = (u64::from(local.slot.load(Ordering::Relaxed)) << 8 | (index as u64 + 1))
        * 0x0101_0101_0101_0101;
    loop {
        let intact: u32;
        // SAFETY: Pure register arithmetic and PAUSE on this thread's own
        // stack; r12-r15 are declared clobbered, no memory is touched.
        unsafe {
            asm!(
                "mov r12, {a}", "mov r13, {b}", "mov r14, {c}", "mov r15, {d}",
                "2:", "pause", "dec {n}", "jnz 2b",
                "xor {ok:e}, {ok:e}",
                "cmp r12, {a}", "jne 3f", "cmp r13, {b}", "jne 3f",
                "cmp r14, {c}", "jne 3f", "cmp r15, {d}", "jne 3f",
                "mov {ok:e}, 1",
                "3:",
                a = in(reg) seed, b = in(reg) seed ^ 0x5555, c = in(reg) seed.rotate_left(17),
                d = in(reg) !seed, n = inout(reg) 2_000u64 => _, ok = out(reg) intact,
                out("r12") _, out("r13") _, out("r14") _, out("r15") _,
                options(nomem, nostack),
            );
        }
        if intact != 1 {
            local.corrupt.store(1, Ordering::Release);
        }
        // Step 5c: wait for the BSP's KERNEL-level lock (IF=1 thread, so the
        // 5b window lets ticks in while lockdep is non-empty).
        if PREEMPT_LOCKED.load(Ordering::Acquire) {
            *PREEMPT_LOCK.lock_irqsave() += 1;
        }
        local.counts[index].fetch_add(1, Ordering::Release);
    }
}
extern "C" fn nos_smp_thread0() -> ! {
    match this_cpu() {
        Some(local) => thread_body(local, 0),
        None => park(),
    }
}
extern "C" fn nos_smp_thread1() -> ! {
    match this_cpu() {
        Some(local) => thread_body(local, 1),
        None => park(),
    }
}

/// Build an initial frame on a fresh thread stack so that the timer stub's
/// pops and `iretq` enter `entry` with IF=1 and a SysV-aligned stack.
fn thread_frame(slot: usize, kind: usize, entry: u64, arg: u64) -> u64 {
    let top = stack_base(slot, kind) + STACK_TOP;
    let frame = initial_frame(top - 8 - 20 * 8, [entry, 0x08, 0x202, top - 8, 0x10], arg);
    // SAFETY: As in `initial_frame`: a zero return address the
    // never-returning entry cannot use, still inside the same thread stack.
    unsafe { ((top - 8) as *mut u64).write(0) };
    frame
}

/// 15 GPRs (rdi = `arg`, slot 8 in pop order) and the 5-word CPU frame at
/// `frame`, as the timer stub pops them.
fn initial_frame(frame: u64, iret: [u64; 5], arg: u64) -> u64 {
    // SAFETY: callers pass 20 aligned u64 slots inside a guarded thread stack
    // (kind THREADS[i] of a claimed slot) that is unused, or whose earlier
    // thread was frozen at its phase's STOP and is never resumed.
    unsafe {
        for i in 0..15 {
            (frame as *mut u64)
                .add(i)
                .write(if i == 8 { arg } else { 0 });
        }
        for (i, value) in iret.into_iter().enumerate() {
            (frame as *mut u64).add(15 + i).write(value);
        }
    }
    frame
}

/// AP side, IF=0 in the home loop: prepare both threads, then arm the
/// periodic timer. The `test-smp-ap-timer-dead` AP never arms it.
fn arm_preemption(local: &CpuLocal, first: bool) {
    let slot = local.slot.load(Ordering::Relaxed) as usize;
    local.saved[1].store(
        thread_frame(slot, THREADS[0], nos_smp_thread0 as usize as u64, 0),
        Ordering::Relaxed,
    );
    local.saved[2].store(
        thread_frame(slot, THREADS[1], nos_smp_thread1 as usize as u64, 0),
        Ordering::Relaxed,
    );
    local.current.store(0, Ordering::Relaxed);
    local.preempt.store(ARMED, Ordering::Release);
    if cfg!(feature = "test-smp-ap-timer-dead") && first {
        return;
    }
    start_timer();
}

fn start_timer() {
    lapic_write(TIMER_DIVIDE, 0x3);
    lapic_write(LVT_TIMER, TIMER_PERIODIC | TIMER_VECTOR as u32);
    lapic_write(TIMER_INITIAL, 20_000);
}

/// BSP: every AP must show real preemption (ticks, switches, both threads
/// advancing), then return to its home loop after STOP. Any AP that does not
/// is treated like a deaf AP: every AP is reset and the BSP continues alone.
fn preemption(online: usize, total: usize) -> bool {
    let aps = &NOS_SMP_AP_CONTEXTS[..online];
    let locals = &CPU_LOCAL[..online];
    PREEMPT.store(PREEMPT_RUN, Ordering::Release);
    for (context, local) in aps.iter().zip(locals) {
        let apic = context.apic_id.load(Ordering::Relaxed);
        let progressed = || {
            local.ticks.load(Ordering::Acquire) >= MIN_TICKS
                && local.switches.load(Ordering::Acquire) >= MIN_SWITCHES
                && local
                    .counts
                    .iter()
                    .all(|count| count.load(Ordering::Acquire) != 0)
        };
        if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32) || !within_ms(500, progressed) || {
            // Both threads must keep advancing between two BSP samples.
            let before = local
                .counts
                .each_ref()
                .map(|count| count.load(Ordering::Acquire));
            !within_ms(500, || {
                local
                    .counts
                    .iter()
                    .zip(before)
                    .all(|(count, old)| count.load(Ordering::Acquire) > old)
            })
        } {
            serial::write(b"SMP:PREEMPT:TIMEOUT:APIC=");
            decimal(u64::from(apic));
            serial::write(b"\n");
            quarantine_all(online, total);
            return false;
        }
    }
    // Without the 5b IF window no tick reaches a lock waiter, so the 5b deaf
    // control skips this proof and must fail later, in `spin_window`.
    if !cfg!(feature = "test-smp-spin-irq-off") {
        preempt_under_lock(locals);
    }
    PREEMPT.store(PREEMPT_STOP, Ordering::Release);
    for (context, local) in aps.iter().zip(locals) {
        let apic = context.apic_id.load(Ordering::Relaxed);
        if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32)
            || !within_ms(200, || local.preempt.load(Ordering::Acquire) == RESUMED)
        {
            serial::write(b"SMP:PREEMPT:STOP_TIMEOUT\n");
            quarantine_all(online, total);
            return false;
        }
    }
    // After STOP nothing may run any more: counters and ticks stay frozen,
    // and no thread saw its registers change across a switch.
    let frozen = |local: &CpuLocal| {
        (
            local.ticks.load(Ordering::Acquire),
            local
                .counts
                .each_ref()
                .map(|count| count.load(Ordering::Acquire)),
        )
    };
    let snapshot: [_; MAX_APS] = core::array::from_fn(|i| (i < online).then(|| frozen(&locals[i])));
    wait_us(10_000);
    for (i, local) in locals.iter().enumerate() {
        if snapshot[i] != Some(frozen(local)) {
            serial::fail(b"SMP_PREEMPT_RAN_AFTER_STOP");
        }
        if local.corrupt.load(Ordering::Acquire) != 0 {
            serial::fail(b"SMP_PREEMPT_CONTEXT_CORRUPT");
        }
    }
    serial::write(b"SMP:PREEMPT:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    true
}

/// Step 5c test lock (KERNEL level), taken by the AP threads while LOCKED.
static PREEMPT_LOCK: lock::IrqLock<u32> = lock::IrqLock::new(level::KERNEL, 0);
static PREEMPT_LOCKED: AtomicBool = AtomicBool::new(false);

/// INV-5c proof: while the BSP holds a KERNEL-level lock, the thread waiting
/// for it on every AP keeps its CPU. Ticks arrive in its IF window and are
/// deferred (`deferred` rises) while `switches` stays put; after the release
/// switching resumes. Runs with every AP's timer proven live, so a missing
/// tick or a switch here is a broken invariant, not a deaf AP: fail closed.
fn preempt_under_lock(locals: &[CpuLocal]) {
    let held = PREEMPT_LOCK.lock_irqsave();
    PREEMPT_LOCKED.store(true, Ordering::Release);
    for local in locals {
        // The first deferred tick means the running thread is now the waiter.
        let start = local.deferred.load(Ordering::Acquire);
        if !within_ms(500, || local.deferred.load(Ordering::Acquire) > start) {
            serial::fail(b"SMP_PREEMPT_LOCK_NO_TICK");
        }
        let (first, switches) = (
            local.deferred.load(Ordering::Acquire),
            local.switches.load(Ordering::Acquire),
        );
        if !within_ms(500, || local.deferred.load(Ordering::Acquire) >= first + 2)
            || local.switches.load(Ordering::Acquire) != switches
        {
            serial::fail(b"SMP_PREEMPT_LOCK_SWITCHED");
        }
    }
    let switches: [u32; MAX_APS] = core::array::from_fn(|i| {
        locals
            .get(i)
            .map_or(0, |l| l.switches.load(Ordering::Acquire))
    });
    drop(held);
    PREEMPT_LOCKED.store(false, Ordering::Release);
    for (local, before) in locals.iter().zip(switches) {
        if !within_ms(500, || local.switches.load(Ordering::Acquire) > before) {
            serial::fail(b"SMP_PREEMPT_LOCK_NOT_RESUMED");
        }
    }
}

/// Step 5b test lock (KERNEL level); used only by the spin phase below.
static SPIN_LOCK: lock::IrqLock<u32> = lock::IrqLock::new(level::KERNEL, 0);
static SPIN: AtomicBool = AtomicBool::new(false);

/// Spin proof (spec §4.3): while the BSP holds a KERNEL-level lock, every AP
/// waits for it with IF=0 and must still acknowledge a shootdown round,
/// through the lock's `sti; pause; cli` window. In `test-smp-spin-irq-off`
/// (no window) the first waiter is deaf: the round times out and every AP is
/// reset before the BSP lets go of the lock, which stays unusable afterwards.
fn spin_window(online: usize, total: usize) -> bool {
    let aps = &NOS_SMP_AP_CONTEXTS[..online];
    let locals = &CPU_LOCAL[..online];
    let held = SPIN_LOCK.lock_irqsave();
    SPIN.store(true, Ordering::Release);
    for (context, local) in aps.iter().zip(locals) {
        let apic = context.apic_id.load(Ordering::Relaxed);
        if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32)
            || !within_ms(100, || local.spun.load(Ordering::Acquire) == 1)
        {
            serial::write(b"SMP:SPIN:TIMEOUT:APIC=");
            decimal(u64::from(apic));
            serial::write(b"\n");
            quarantine_all(online, total);
            return false;
        }
    }
    // Every AP now waits with IF=0; an invlpg of the current window is harmless.
    let window = paging::SHOOTDOWN_WINDOW;
    if let Err(apic) = shootdown_round([window, window], online) {
        serial::write(b"SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=");
        decimal(u64::from(apic));
        serial::write(b"\n");
        quarantine_all(online, total);
        drop(held);
        return false;
    }
    drop(held);
    for local in locals {
        if !within_ms(100, || local.spun.load(Ordering::Acquire) == 2) {
            serial::fail(b"SMP_SPIN_NOT_GRANTED");
        }
        if local.lockdep.load(Ordering::Acquire) != 0 {
            serial::fail(b"SMP_SPIN_LOCKDEP_LEFT");
        }
    }
    if *SPIN_LOCK.lock_irqsave() != online as u32 {
        serial::fail(b"SMP_SPIN_COUNT");
    }
    serial::write(b"SMP:SPIN:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    true
}

// Step 5d-1 (spec §6): a run queue per AP and kernel threads in two address
// spaces. A thread's state word (`smp_lock::sched`) is the only way onto a CPU
// (INV-3); a CR3 switch keeps `ACTIVE` a superset of the CPUs that may cache a
// space (INV-4). Ring-3 processes on APs follow in 5d-2, after 5b-2 (INV-2).
const RESCHED_VECTOR: usize = 0xe8;
const SCHED_THREADS: usize = 2 * MAX_APS;
const HOME: u32 = u32::MAX;
const ROOT_MASK: u64 = !0xfff;
/// 0, then PREEMPT_RUN / PREEMPT_STOP for the scheduler phase.
static SCHED: AtomicU32 = AtomicU32::new(0);
/// Online APs during the phase: thread `tid` runs on AP `tid % aps`, in space
/// 0 if `tid < aps` (READY at start) and space 1 otherwise (BLOCKED, woken).
static SCHED_APS: AtomicU32 = AtomicU32::new(0);
/// Keyed by CPU index (AP slot + 1), the LD-3 order for a RUNQ `lock_pair`.
static RUNQ: [lock::IrqLock<RunQueue<SCHED_THREADS>>; MAX_APS] = {
    let mut queues = [const { lock::IrqLock::new(level::RUNQ, RunQueue::new()) }; MAX_APS];
    let mut slot = 0;
    while slot < MAX_APS {
        queues[slot] = lock::IrqLock::keyed(level::RUNQ, slot as u32 + 1, RunQueue::new());
        slot += 1;
    }
    queues
};
static THREAD_STATE: [AtomicU32; SCHED_THREADS] =
    [const { AtomicU32::new(sched::BLOCKED) }; SCHED_THREADS];
static THREAD_RSP: [AtomicU64; SCHED_THREADS] = [const { AtomicU64::new(0) }; SCHED_THREADS];
/// Audit F1: set while a CPU is on the thread's stack (`sched::claim_stack`),
/// cleared by the timer stub only after it switched away from that stack.
static THREAD_ON_CPU: [AtomicBool; SCHED_THREADS] =
    [const { AtomicBool::new(false) }; SCHED_THREADS];
static PROGRESS: [AtomicU64; SCHED_THREADS] = [const { AtomicU64::new(0) }; SCHED_THREADS];
static SPACE_ROOT: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];
static KERNEL_CR3: AtomicU64 = AtomicU64::new(0);
/// `active_cpus` per space (spec §6.1), bit = CPU index (BSP 0, AP slot + 1).
static ACTIVE: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];
/// Set by a thread that ran under a CR3 or bitmap other than its own space's.
static SCHED_FAULT: AtomicU32 = AtomicU32::new(0);

// Reschedule IPI (spec §6.4): count it on this CPU, then take the timer path,
// which saves the GPRs, may switch, and sends the EOI. RAX is restored first.
global_asm!(
    ".pushsection .text.nos_smp_resched,\"ax\"",
    ".globl nos_smp_resched_entry",
    "nos_smp_resched_entry:",
    arch::gs_entry!("8"),
    "    push rax",
    "    mov rax, gs:[0]",
    "    lock inc dword ptr [rax + {resched}]",
    "    pop rax",
    "    jmp nos_smp_timer_body",
    ".popsection",
    resched = const core::mem::offset_of!(CpuLocal, resched),
);

fn space(tid: u32) -> Option<usize> {
    (tid != HOME).then(|| usize::from(tid >= SCHED_APS.load(Ordering::Relaxed)))
}

/// Terminal report (spec §9.11): a thread was found running on another CPU.
fn double_run() -> ! {
    serial::write(b"SMP:SCHED:DOUBLE_RUN\n");
    if cfg!(feature = "test-smp-double-run") {
        serial::exit(0x10);
    }
    serial::fail(b"SMP_SCHED_DOUBLE_RUN")
}

/// Spec §6.3 step 5: the new space's bit is set before the CR3 load and the
/// old one cleared after it, so `ACTIVE` never misses a CPU that may cache it.
fn switch_space(cpu: usize, from: Option<usize>, to: Option<usize>) {
    let bit = 1u64 << cpu;
    if let Some(to) = to {
        ACTIVE[to].fetch_or(bit, Ordering::SeqCst);
    }
    let root = to.map_or(&KERNEL_CR3, |to| &SPACE_ROOT[to]);
    // SAFETY: the kernel CR3, or a PML4 copied entry for entry from it (same
    // kernel half, no user mappings): code, stacks, GS anchor and the LAPIC
    // window stay mapped. PCID is off, so the load flushes this CPU's TLB.
    unsafe { asm!("mov cr3, {}", in(reg) root.load(Ordering::Relaxed), options(nostack)) };
    if let Some(from) = from {
        ACTIVE[from].fetch_and(!bit, Ordering::SeqCst);
    }
}

/// Context switch on an AP (spec §6.3), IF=0 and lockdep empty on entry. The
/// RUNQ lock covers the state hand-over and the CR3 switch; STOP returns home.
fn sched_switch(local: &CpuLocal, rsp: u64, busy: bool) -> Resume {
    let slot = local.slot.load(Ordering::Relaxed) as usize;
    let cpu = slot + 1;
    let current = local.sched_cur.load(Ordering::Relaxed);
    let stop = SCHED.load(Ordering::Acquire) == PREEMPT_STOP;
    let mut queue = RUNQ[slot].lock_irqsave();
    if current == HOME {
        local.saved[0].store(rsp, Ordering::Relaxed);
    } else {
        THREAD_RSP[current as usize].store(rsp, Ordering::Relaxed);
        if sched::yield_cpu(&THREAD_STATE[current as usize], cpu).is_err()
            || queue.push(current).is_err()
        {
            double_run();
        }
    }
    let mut next = if stop {
        HOME
    } else {
        queue.pop().unwrap_or(HOME)
    };
    if next != HOME && sched::dispatch(&THREAD_STATE[next as usize], cpu).is_err() {
        double_run();
    }
    // Audit F1: the CPU that last ran `next` may still be on its stack. Do not
    // wait for it here; requeue and run home until the next tick.
    if next != HOME && next != current && !sched::claim_stack(&THREAD_ON_CPU[next as usize]) {
        if sched::yield_cpu(&THREAD_STATE[next as usize], cpu).is_err() || queue.push(next).is_err()
        {
            double_run();
        }
        next = HOME;
    }
    if busy && next != current {
        held_across_switch();
    }
    let (from, to) = (space(current), space(next));
    if from != to {
        switch_space(cpu, from, to);
        local.cr3_switches.fetch_add(1, Ordering::Release);
    }
    drop(queue);
    if stop {
        lapic_write(LVT_TIMER, MASKED | TIMER_VECTOR as u32);
        local.sched_state.store(RESUMED, Ordering::Release);
    }
    local.sched_cur.store(next, Ordering::Relaxed);
    let left = if current != HOME && next != current {
        ptr::from_ref(&THREAD_ON_CPU[current as usize]) as u64
    } else {
        0
    };
    Resume {
        rsp: match next {
            HOME => local.saved[0].load(Ordering::Relaxed),
            tid => THREAD_RSP[tid as usize].load(Ordering::Relaxed),
        },
        left,
    }
}

/// Every pass checks, with IF=0, that this CPU runs under the thread's own
/// space root and has its bit in that space's `ACTIVE`, then counts progress.
extern "C" fn nos_smp_sched_thread(tid: u64) -> ! {
    let tid = tid as usize;
    let own = usize::from(tid >= SCHED_APS.load(Ordering::Relaxed) as usize);
    loop {
        let cr3: u64;
        // SAFETY: CLI and a CR3 read in this CPL0 thread; IF is restored below.
        unsafe { asm!("cli", "mov {}, cr3", out(reg) cr3, options(nomem, nostack)) };
        let intact = this_cpu().is_some_and(|local| {
            let bit = 1u64 << (local.slot.load(Ordering::Relaxed) + 1);
            cr3 & ROOT_MASK == SPACE_ROOT[own].load(Ordering::Relaxed)
                && ACTIVE[own].load(Ordering::SeqCst) & bit != 0
        });
        // SAFETY: re-enables interrupts for this preemptible kernel thread.
        unsafe { asm!("sti", options(nomem, nostack)) };
        if !intact {
            SCHED_FAULT.store(1, Ordering::Release);
        }
        PROGRESS[tid].fetch_add(1, Ordering::Release);
        for _ in 0..500 {
            spin_loop();
        }
    }
}

fn sched_timeout(apic: u32, online: usize, total: usize) -> bool {
    serial::write(b"SMP:SCHED:TIMEOUT:APIC=");
    decimal(u64::from(apic));
    serial::write(b"\n");
    quarantine_all(online, total);
    false
}

/// BSP: run 2 threads per AP from per-CPU queues, one of each space, the
/// second woken cross-CPU with 0xe8 (spec §6.4). Every AP must dispatch both
/// and switch CR3 repeatedly; after STOP every AP is home under the kernel CR3
/// with no `ACTIVE` bit left. A silent AP is a deaf AP (quarantine).
fn schedule(online: usize, total: usize) -> bool {
    let aps = &NOS_SMP_AP_CONTEXTS[..online];
    let locals = &CPU_LOCAL[..online];
    if online != 0 {
        let cr3: u64;
        // SAFETY: CPL0 register read.
        unsafe { asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack)) };
        KERNEL_CR3.store(cr3, Ordering::Relaxed);
        for root in &SPACE_ROOT {
            // SAFETY: both PML4 frames are identity-mapped (ours RW by
            // `prepare`, the active one by `map_tables`); ours is Kernel-owned,
            // exclusive and not yet any CPU's CR3; 512 aligned u64 are copied.
            unsafe {
                ptr::copy_nonoverlapping(
                    (cr3 & ROOT_MASK) as *const u64,
                    root.load(Ordering::Relaxed) as *mut u64,
                    512,
                );
            }
        }
        SCHED_APS.store(online as u32, Ordering::Relaxed);
        for tid in 0..2 * online {
            // Stack: slot tid/2's thread stacks, frozen since the step-4 STOP.
            let entry = nos_smp_sched_thread as usize as u64;
            let frame = thread_frame(tid / 2, THREADS[tid % 2], entry, tid as u64);
            THREAD_RSP[tid].store(frame, Ordering::Relaxed);
        }
        for (slot, queue) in RUNQ[..online].iter().enumerate() {
            THREAD_STATE[slot].store(sched::READY, Ordering::Release);
            if queue.lock_irqsave().push(slot as u32).is_err() {
                serial::fail(b"SMP_SCHED_QUEUE");
            }
        }
        if cfg!(feature = "test-smp-double-run") && online >= 2 {
            // Thread 0 queued on a second AP as well: dispatch must refuse.
            let _ = RUNQ[1].lock_irqsave().push(0);
        }
        SCHED.store(PREEMPT_RUN, Ordering::Release);
        for (slot, context) in aps.iter().enumerate() {
            let apic = context.apic_id.load(Ordering::Relaxed);
            if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32)
                || !within_ms(500, || PROGRESS[slot].load(Ordering::Acquire) != 0)
            {
                return sched_timeout(apic, online, total);
            }
        }
        for (slot, (context, local)) in aps.iter().zip(locals).enumerate() {
            let (apic, tid) = (context.apic_id.load(Ordering::Relaxed), online + slot);
            {
                let mut queue = RUNQ[slot].lock_irqsave();
                if sched::wake(&THREAD_STATE[tid]).is_err() || queue.push(tid as u32).is_err() {
                    serial::fail(b"SMP_SCHED_WAKE");
                }
            }
            let (here, woken) = (&PROGRESS[slot], &PROGRESS[tid]);
            if !send_ipi(apic, IPI_FIXED | RESCHED_VECTOR as u32)
                || !within_ms(500, || {
                    local.resched.load(Ordering::Acquire) != 0
                        && woken.load(Ordering::Acquire) != 0
                        && local.cr3_switches.load(Ordering::Acquire) >= MIN_SWITCHES
                })
                || {
                    let before = (here.load(Ordering::Acquire), woken.load(Ordering::Acquire));
                    !within_ms(500, || {
                        here.load(Ordering::Acquire) > before.0
                            && woken.load(Ordering::Acquire) > before.1
                    })
                }
            {
                return sched_timeout(apic, online, total);
            }
        }
        SCHED.store(PREEMPT_STOP, Ordering::Release);
        if !within_ms(200, || {
            locals
                .iter()
                .all(|local| local.sched_state.load(Ordering::Acquire) == RESUMED)
        }) {
            serial::write(b"SMP:SCHED:STOP_TIMEOUT\n");
            quarantine_all(online, total);
            return false;
        }
        let snapshot: [u64; SCHED_THREADS] =
            core::array::from_fn(|tid| PROGRESS[tid].load(Ordering::Acquire));
        wait_us(10_000);
        if (0..SCHED_THREADS).any(|tid| PROGRESS[tid].load(Ordering::Acquire) != snapshot[tid]) {
            serial::fail(b"SMP_SCHED_RAN_AFTER_STOP");
        }
        if ACTIVE.iter().any(|bits| bits.load(Ordering::SeqCst) != 0)
            || locals
                .iter()
                .any(|local| local.lockdep.load(Ordering::Acquire) != 0)
        {
            serial::fail(b"SMP_SCHED_ACTIVE_LEFT");
        }
        if SCHED_FAULT.load(Ordering::Acquire) != 0 {
            serial::fail(b"SMP_SCHED_SPACE");
        }
    }
    serial::write(b"SMP:SCHED:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    true
}

// Step 5b-2 (ADR-075, K-1/K-2): every CPU zeroes, fills and checks its OWN
// frame through its OWN scratch slot, concurrently, then takes KERNEL to bump
// a counter that only the lock protects. `test-smp-shared-scratch` puts every
// CPU in slot 0 (the old global slot) and must report the crossing.
const SCRATCH_ROUNDS: u32 = 1024;
const KERNEL_ROUNDS: u64 = 1000;
static SCRATCH_RUN: AtomicBool = AtomicBool::new(false);
static SCRATCH_FRAMES: [AtomicU64; MAX_CPUS] = [const { AtomicU64::new(0) }; MAX_CPUS];
/// Plain load + store, never an atomic RMW: correct only under KERNEL.
static SCRATCH_COUNT: AtomicU64 = AtomicU64::new(0);
static SCRATCH_CROSSED: AtomicBool = AtomicBool::new(false);

/// Terminal: a CPU saw another CPU in its scratch slot (a busy PTE or a
/// foreign pattern), `SMP:SCRATCH:CROSS:CPU=<index>`. One CPU reports; any
/// other that also detects it stops silently. Pass only in the control.
pub fn scratch_cross() -> ! {
    if SCRATCH_CROSSED.swap(true, Ordering::AcqRel) {
        park();
    }
    serial::write(b"SMP:SCRATCH:CROSS:CPU=");
    decimal(cpu_index() as u64);
    serial::write(b"\n");
    if cfg!(feature = "test-smp-shared-scratch") {
        serial::exit(0x10);
    }
    serial::fail(b"SMP_SCRATCH_CROSS")
}

/// One CPU's share (BSP: 0, AP: slot + 1), IF=0.
fn scratch_work(cpu: usize) {
    let pa = SCRATCH_FRAMES[cpu].load(Ordering::Acquire);
    let byte = 0xa5 ^ cpu as u8;
    let word = u64::from_ne_bytes([byte; 8]);
    for _ in 0..SCRATCH_ROUNDS {
        paging::zero_dynamic(pa);
        // SAFETY: the slot maps exactly this CPU's own Kernel-owned frame;
        // one page is written through the callback's pointer only.
        paging::physical(pa, |p| unsafe { ptr::write_bytes(p, byte, PAGE as usize) });
        let intact = paging::physical(pa, |p| {
            // SAFETY: as above; 512 aligned u64 reads within the one page.
            (0..PAGE as usize / 8)
                .all(|i| unsafe { p.cast::<u64>().add(i).read_volatile() } == word)
        });
        if !intact {
            scratch_cross();
        }
    }
    for _ in 0..KERNEL_ROUNDS {
        let _kernel = lock::KERNEL.lock_irqsave();
        assert!(lock::kernel_held());
        SCRATCH_COUNT.store(SCRATCH_COUNT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
    }
}

/// BSP: give every online CPU its own frame, run all shares concurrently and
/// require every pattern intact, KERNEL_ROUNDS per CPU counted and lockdep
/// empty. APs get 500 ms after the BSP's own share; a silent AP is quarantined.
fn scratch(online: usize, total: usize, frames: &mut Frames<FRAME_COUNT>) -> bool {
    for frame in &SCRATCH_FRAMES[..=online] {
        let pa = frames
            .allocate(Owner::Kernel, paging::zero_dynamic)
            .unwrap_or_else(|_| serial::fail(b"SMP_SCRATCH_FRAMES"));
        frame.store(pa, Ordering::Relaxed);
    }
    let aps = &NOS_SMP_AP_CONTEXTS[..online];
    let locals = &CPU_LOCAL[..online];
    SCRATCH_RUN.store(true, Ordering::Release);
    for context in aps {
        let apic = context.apic_id.load(Ordering::Relaxed);
        if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32) {
            serial::write(b"SMP:SCRATCH:TIMEOUT:APIC=");
            decimal(u64::from(apic));
            serial::write(b"\n");
            quarantine_all(online, total);
            return false;
        }
    }
    scratch_work(0);
    if !within_ms(500, || {
        locals
            .iter()
            .all(|local| local.scratched.load(Ordering::Acquire) == 1)
    }) {
        serial::write(b"SMP:SCRATCH:TIMEOUT\n");
        quarantine_all(online, total);
        return false;
    }
    if cfg!(feature = "test-smp-shared-scratch") {
        serial::fail(b"SMP_SCRATCH_NO_CROSS");
    }
    if SCRATCH_COUNT.load(Ordering::Acquire) != KERNEL_ROUNDS * (online as u64 + 1) {
        serial::fail(b"SMP_SCRATCH_LOST_UPDATE");
    }
    if lockdep().load(Ordering::Relaxed) != 0
        || locals
            .iter()
            .any(|local| local.lockdep.load(Ordering::Acquire) != 0)
    {
        serial::fail(b"SMP_SCRATCH_LOCKDEP_LEFT");
    }
    // Every slot PTE was cleared and invalidated by its own CPU; no CPU uses
    // another slot's VA, so the frames are free to reuse.
    for frame in &SCRATCH_FRAMES[..=online] {
        if frames
            .free(
                frame.load(Ordering::Relaxed),
                Owner::Kernel,
                paging::zero_dynamic,
            )
            .is_err()
        {
            serial::fail(b"SMP_SCRATCH_RECLAIM");
        }
    }
    serial::write(b"SMP:SCRATCH:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    true
}

/// Audit F4 fixture (LD-3): two ASPACE-level locks keyed like pids 1 and 2.
/// Both counters only ever change together, under the pair.
static PAIR: [lock::IrqLock<u64>; 2] = [
    lock::IrqLock::keyed(level::ASPACE, 1, 0),
    lock::IrqLock::keyed(level::ASPACE, 2, 0),
];
static PAIR_RUN: AtomicBool = AtomicBool::new(false);
const PAIR_ROUNDS: u64 = 1000;

/// One CPU's rounds, IF=0. Odd CPU indexes name the pair the other way round,
/// so with two or more APs some always ask in opposite argument order.
fn pair_work(cpu: usize) {
    let (a, b) = if cpu % 2 == 0 {
        (&PAIR[0], &PAIR[1])
    } else {
        (&PAIR[1], &PAIR[0])
    };
    for _ in 0..PAIR_ROUNDS {
        let (mut low, mut high) = a.lock_pair(b);
        if *low != *high {
            serial::fail(b"SMP_LOCKPAIR_TORN");
        }
        *low += 1;
        *high += 1;
    }
}

/// BSP: every online CPU runs PAIR_ROUNDS concurrently; no deadlock, no
/// marker, every round counted in both locks and lockdep empty afterwards.
/// `test-smp-lockpair-order` takes the two loose and descending instead.
fn lock_pairs(online: usize, total: usize) -> bool {
    if cfg!(feature = "test-smp-lockpair-order") {
        let _second = PAIR[1].lock_irqsave();
        let _first = PAIR[0].lock_irqsave();
        serial::fail(b"SMP_LOCKPAIR_ORDER_MISSED");
    }
    let aps = &NOS_SMP_AP_CONTEXTS[..online];
    let locals = &CPU_LOCAL[..online];
    PAIR_RUN.store(true, Ordering::Release);
    for context in aps {
        let apic = context.apic_id.load(Ordering::Relaxed);
        if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32) {
            serial::write(b"SMP:LOCKPAIR:TIMEOUT:APIC=");
            decimal(u64::from(apic));
            serial::write(b"\n");
            quarantine_all(online, total);
            return false;
        }
    }
    pair_work(0);
    if !within_ms(2_000, || {
        locals
            .iter()
            .all(|local| local.paired.load(Ordering::Acquire) == 1)
    }) {
        serial::write(b"SMP:LOCKPAIR:TIMEOUT\n");
        quarantine_all(online, total);
        return false;
    }
    let rounds = PAIR_ROUNDS * (online as u64 + 1);
    let (low, high) = PAIR[0].lock_pair(&PAIR[1]);
    if *low != rounds || *high != rounds {
        serial::fail(b"SMP_LOCKPAIR_LOST_UPDATE");
    }
    drop(high);
    drop(low);
    if lockdep().load(Ordering::Relaxed) != 0
        || locals
            .iter()
            .any(|local| local.lockdep.load(Ordering::Acquire) != 0)
    {
        serial::fail(b"SMP_LOCKPAIR_LOCKDEP_LEFT");
    }
    serial::write(b"SMP:LOCKPAIR:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    true
}

// Audit F8 fixture (test-smp-gs-*, spec §4): one ring-3 thread per AP, whose
// code loads its own GS selector and then spins, so every later kernel entry
// arrives with a GS_BASE that ring 3 chose. A one-shot tick from the home loop
// enters ring 3 (TSS.RSP0 = that thread's stack, CR3 = GS_ROOT); the BSP then
// sends 0xf0/0xe0/0xe8, the 0xe8 path arms the periodic timer, and STOP returns
// home. GS_ROOT shares the kernel half (PDPT[0] and every other PML4 entry)
// and adds one user RX page at USER_CODE in a private PDPT[1].
const GS_FIXTURE: bool = cfg!(any(
    feature = "test-smp-gs-user",
    feature = "test-smp-gs-iret-fault",
    feature = "test-smp-gs-nmi-user"
));
/// 0, then PREEMPT_RUN / PREEMPT_STOP for the GS phase.
static GS: AtomicU32 = AtomicU32::new(0);
const GS_ARMED: u32 = 1;
const GS_USER: u32 = 2;
const GS_PERIODIC: u32 = 3;
const GS_RESUMED: u32 = 4;
const USER_CODE: u64 = 0x4000_0000;
const ENTRY_ADDRESS: u64 = 0x000f_ffff_ffff_f000;
/// Ring-3 code: variant A (`mov ax, 0x1b; mov gs, ax`) at 0, variant B
/// (`xor eax, eax; mov gs, ax`) at 16, each followed by `pause; jmp -4`.
const USER_CODE_BYTES: [u8; 24] = [
    0x66, 0xb8, 0x1b, 0x00, 0x8e, 0xe8, 0xf3, 0x90, 0xeb, 0xfc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc,
    0x31, 0xc0, 0x8e, 0xe8, 0xf3, 0x90, 0xeb, 0xfc,
];
/// GS_ROOT, its PDPT, PD and PT, then the user code frame.
static GS_FRAMES: [AtomicU64; 5] = [const { AtomicU64::new(0) }; 5];

/// Before the CR3 switch: four table frames, identity-mapped RW/NX so the BSP
/// can fill them later, and the code frame, written now through the firmware
/// identity map and never mapped by the kernel (only user RX in GS_ROOT).
// shortcut: the five frames are never freed; real user spaces come in 5d-2.
fn prepare_gs(
    frames: &mut Frames<FRAME_COUNT>,
    paging: &mut paging::Paging,
) -> Result<(), kernel_core::Error> {
    for (index, slot) in GS_FRAMES.iter().enumerate() {
        let pa = frames.allocate(Owner::Kernel, zero)?;
        if index < 4 {
            paging.map(frames, pa, pa, true, false);
        }
        slot.store(pa, Ordering::Relaxed);
    }
    let code = GS_FRAMES[4].load(Ordering::Relaxed);
    // SAFETY: fresh exclusive Kernel-owned frame, identity-mapped by the still
    // active firmware CR3; 24 bytes of our own constant are copied into it.
    unsafe {
        ptr::copy_nonoverlapping(
            USER_CODE_BYTES.as_ptr(),
            code as *mut u8,
            USER_CODE_BYTES.len(),
        );
    }
    Ok(())
}

/// BSP, after `schedule` recorded KERNEL_CR3. False if the kernel already
/// uses PDPT[1] (the user slot), which would make the copy diverge.
fn build_gs_root() -> bool {
    let [root, pdpt, pd, pt, code] = GS_FRAMES.each_ref().map(|f| f.load(Ordering::Relaxed));
    let kernel = (KERNEL_CR3.load(Ordering::Relaxed) & ROOT_MASK) as *const u64;
    // SAFETY: the four table frames are exclusive, Kernel-owned and
    // identity-mapped RW (prepare_gs); the active PML4 and its PDPT are
    // identity-mapped by `map_tables`. No CPU uses GS_ROOT yet.
    unsafe {
        ptr::copy_nonoverlapping(kernel, root as *mut u64, 512);
        let kernel_pdpt = (kernel.read() & ENTRY_ADDRESS) as *const u64;
        ptr::copy_nonoverlapping(kernel_pdpt, pdpt as *mut u64, 512);
        if kernel.read() & 1 == 0 || (pdpt as *const u64).add(1).read() != 0 {
            return false;
        }
        (root as *mut u64).write(pdpt | 7);
        (pdpt as *mut u64).add(1).write(pd | 7);
        (pd as *mut u64).write(pt | 7);
        // Present | User, read-only and executable: the only user leaf.
        (pt as *mut u64).write(code | 5);
    }
    true
}

fn ap_tables(slot: usize) -> *mut arch::CpuTables {
    // Address only; the caller is the AP that owns this slot.
    ptr::addr_of_mut!(CPU_TABLES)
        .cast::<arch::CpuTables>()
        .wrapping_add(slot)
}

/// The GS phase of `smp_preempt` on an AP (IF=0, lockdep empty).
fn gs_switch(local: &CpuLocal, rsp: u64) -> Resume {
    let slot = local.slot.load(Ordering::Relaxed) as usize;
    // SAFETY: `rsp` is the timer stub's frame: 15 GPRs, then RIP and CS.
    let cs = unsafe { (rsp as *const u64).add(16).read() };
    if cs & 3 == 3 {
        // `this_cpu` matched this AP's anchor although ring 3 owned GS.
        local.user_ticks.fetch_add(1, Ordering::Release);
    }
    let same = Resume { rsp, left: 0 };
    match local.gs_state.load(Ordering::Relaxed) {
        GS_ARMED => {
            local.saved[0].store(rsp, Ordering::Relaxed);
            let top = stack_base(slot, THREADS[0]) + STACK_TOP;
            // N2: SS 0x13 is the DPL0 data descriptor with RPL3, so `iretq`
            // raises #GP(0x10) at CPL0, after the exit swapgs. (QEMU-TCG does
            // not check a non-canonical RIP on `iretq`; that #GP came from CPL3.)
            let ss = if cfg!(feature = "test-smp-gs-iret-fault") {
                0x13
            } else {
                0x1b
            };
            let rip = USER_CODE + 16 * (slot as u64 & 1);
            let frame = initial_frame(top - 20 * 8, [rip, 0x23, 0x202, USER_CODE + PAGE, ss], 0);
            arch::ap_rsp0(ap_tables(slot), top);
            // SAFETY: GS_ROOT maps the kernel half exactly like the kernel CR3.
            unsafe {
                asm!("mov cr3, {}", in(reg) GS_FRAMES[0].load(Ordering::Relaxed), options(nostack));
            }
            local.gs_state.store(GS_USER, Ordering::Release);
            Resume {
                rsp: frame,
                left: 0,
            }
        }
        GS_USER => {
            start_timer();
            local.gs_state.store(GS_PERIODIC, Ordering::Release);
            same
        }
        GS_PERIODIC if GS.load(Ordering::Acquire) == PREEMPT_STOP => {
            lapic_write(LVT_TIMER, MASKED | TIMER_VECTOR as u32);
            // SAFETY: back to the kernel CR3 for the home loop (same kernel half).
            unsafe {
                asm!("mov cr3, {}", in(reg) KERNEL_CR3.load(Ordering::Relaxed), options(nostack));
            }
            arch::ap_rsp0(ap_tables(slot), stack_base(slot, MAIN) + STACK_TOP);
            local.gs_state.store(GS_RESUMED, Ordering::Release);
            Resume {
                rsp: local.saved[0].load(Ordering::Relaxed),
                left: 0,
            }
        }
        _ => same,
    }
}

fn gs_timeout(apic: u32, online: usize, total: usize) -> bool {
    serial::write(b"SMP:GS:TIMEOUT:APIC=");
    decimal(u64::from(apic));
    serial::write(b"\n");
    quarantine_all(online, total);
    false
}

/// BSP (spec §4 N1-N3): every AP in ring 3 with a GS it chose must count
/// 0xf0, 0xe0 and 0xe8 and its CPL3 ticks in its OWN anchor, then come home.
fn gs_user(online: usize, total: usize) -> bool {
    if !GS_FIXTURE || online == 0 {
        return true;
    }
    serial::write(b"SMP:GS:CR4=");
    decimal(control_registers()[CR4]);
    serial::write(b"\n");
    if !build_gs_root() {
        serial::fail(b"SMP_GS_FIXTURE");
    }
    let aps = &NOS_SMP_AP_CONTEXTS[..online];
    let locals = &CPU_LOCAL[..online];
    GS.store(PREEMPT_RUN, Ordering::Release);
    for (context, local) in aps.iter().zip(locals) {
        let apic = context.apic_id.load(Ordering::Relaxed);
        if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32)
            || !within_ms(500, || local.gs_state.load(Ordering::Acquire) == GS_USER)
        {
            return gs_timeout(apic, online, total);
        }
        if cfg!(feature = "test-smp-gs-iret-fault") {
            // That AP's first exit to ring 3 faults and its handler ends the VM.
            within_ms(1_000, || false);
            serial::fail(b"SMP_GS_IRET_NO_FIXUP");
        }
    }
    // Each user thread loads its GS selector as its first instruction.
    wait_us(1_000);
    if cfg!(feature = "test-smp-gs-nmi-user") {
        if send_ipi(aps[0].apic_id.load(Ordering::Relaxed), IPI_NMI) {
            within_ms(1_000, || false);
        }
        serial::fail(b"SMP_GS_NMI_NOT_ON_IST");
    }
    for (context, local) in aps.iter().zip(locals) {
        let apic = context.apic_id.load(Ordering::Relaxed);
        let base = local.ipis.load(Ordering::Acquire);
        for round in 1..=IPI_ROUNDS {
            if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32)
                || !within_ms(100, || local.ipis.load(Ordering::Acquire) >= base + round)
            {
                return gs_timeout(apic, online, total);
            }
        }
    }
    let window = paging::SHOOTDOWN_WINDOW;
    if let Err(apic) = shootdown_round([window, window], online) {
        return gs_timeout(apic, online, total);
    }
    for (context, local) in aps.iter().zip(locals) {
        let apic = context.apic_id.load(Ordering::Relaxed);
        let resched = local.resched.load(Ordering::Acquire);
        if !send_ipi(apic, IPI_FIXED | RESCHED_VECTOR as u32)
            || !within_ms(500, || {
                local.resched.load(Ordering::Acquire) > resched
                    && local.gs_state.load(Ordering::Acquire) == GS_PERIODIC
                    && local.user_ticks.load(Ordering::Acquire) >= MIN_TICKS
            })
        {
            return gs_timeout(apic, online, total);
        }
    }
    GS.store(PREEMPT_STOP, Ordering::Release);
    if !within_ms(200, || {
        locals
            .iter()
            .all(|local| local.gs_state.load(Ordering::Acquire) == GS_RESUMED)
    }) {
        serial::write(b"SMP:GS:STOP_TIMEOUT\n");
        quarantine_all(online, total);
        return false;
    }
    if locals
        .iter()
        .any(|local| local.lockdep.load(Ordering::Acquire) != 0)
    {
        serial::fail(b"SMP_GS_LOCKDEP_LEFT");
    }
    serial::write(b"SMP:GS:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    true
}
