//! K06 step 1 (native-smp only): bounded application-processor bring-up.
//!
//! Ownership and concurrency model, see ADR-070:
//! - The BSP alone owns frames, paging, serial and the PIT. It prepares
//!   everything with IF=0 before the first IPI. Every CPU writes only its own
//!   xAPIC ICR, under its own ICR lock (IQ-1, ADR-079).
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
    mem::MaybeUninit,
    ptr,
    sync::atomic::{fence, AtomicBool, AtomicU32, AtomicU64, Ordering},
};
use kernel_core::{Frames, Kind, Owner, PAGE};
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
    data: [u8; 32768],
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
            data: [0; 32768],
            upper: [0; 4096],
        }
    }; STACK_KINDS]
}; MAX_APS];
const STACK_SPAN: u64 = core::mem::size_of::<ApStack>() as u64;
const STACK_TOP: u64 = 36864;

fn stack_base(slot: usize, kind: usize) -> u64 {
    // Address computation only; no reference to the static is formed.
    ptr::addr_of!(AP_STACKS) as u64 + (slot * STACK_KINDS + kind) as u64 * STACK_SPAN
}
pub fn is_guard(page: u64) -> bool {
    guard_pages().any(|guard| guard == page)
}
pub fn guard_pages() -> impl Iterator<Item = u64> {
    (0..MAX_APS * STACK_KINDS)
        .map(|index| stack_base(index / STACK_KINDS, index % STACK_KINDS))
        .chain((0..BSP_THREADS).map(bsp_stack_base))
        .flat_map(|base| [base, base + STACK_TOP])
}

/// Step 5c-2: the BSP's two preemptible kernel threads (BT-2), guarded like
/// the AP stacks. Only the BSP touches them, in its own phases.
const BSP_THREADS: usize = 2;
static mut BSP_STACKS: [ApStack; BSP_THREADS] = [const {
    ApStack {
        lower: [0; 4096],
        data: [0; 32768],
        upper: [0; 4096],
    }
}; BSP_THREADS];

fn bsp_stack_base(index: usize) -> u64 {
    // Address computation only; no reference to the static is formed.
    ptr::addr_of!(BSP_STACKS) as u64 + index as u64 * STACK_SPAN
}

/// Per-CPU anchor reachable through IA32_GS_BASE. Offsets 0 and 12 are
/// used by the IPI stub; `slot` identifies the AP without trusting
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
    // Preemption (step 4, BSP since 5c-2): touched only by this CPU's own
    // handler/threads. saved[0] is the interrupted home loop, [1..] threads.
    // `tick_count` counts 0xd0 only, `resched_count` 0xe8 only (BT-4, F11).
    tick_count: AtomicU32,
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
    m3_deferred: AtomicU32,
    m3_wait_max: AtomicU64,
    m3_deferred_max: AtomicU32,
    m3_entering: AtomicBool,
    /// Step 5d-1 scheduler: running thread (HOME = the home loop), ARMED /
    /// RESUMED (stopped at home), 0xe8 reschedule IPIs received and CR3
    /// switches. Written by this CPU only, read by the BSP.
    sched_cur: AtomicU32,
    sched_state: AtomicU32,
    resched_count: AtomicU32,
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
    /// IQ-1 (audit F19): ICR high, ICR low and the delivery wait of this CPU.
    icr: lock::IrqLock<()>,
    /// F19 fixture: 0xe8 IPIs this AP sent to its partner, and its result.
    icr_sent: AtomicU32,
    icr_done: AtomicU32,
    /// BT-4 (audit F11): a 0xe8 that was not yet honoured by a switch. Set by
    /// this CPU's handler, kept while INV-5c defers it, cleared by the switch
    /// or by the release of this CPU's last lock (`resched_on_release`).
    need_resched: AtomicBool,
    /// `resched_count` as last seen by `smp_preempt`: a difference means the
    /// entry is a 0xe8 (its stub counted before joining), otherwise a 0xd0.
    resched_seen: AtomicU32,
    /// BT-2/S3: every unmasked interrupt source of this CPU has an IDT gate,
    /// so a KERNEL/ASPACE waiter may open its `sti; pause; cli` window.
    irq_ready: AtomicBool,
    /// Step 5e: 0xe0 rounds this CPU's handler acknowledged (PS-1 target check).
    shootdowns: AtomicU32,
    /// Step 5d-2 (R3-4): top of this CPU's M3 service stack, read by the M3
    /// entry stub through `nos_smp_m3_stack` after its swapgs.
    m3_stack: AtomicU64,
    /// Step 5d-2 (R3-3): this CPU's current M3 FXSAVE image (`fpu::select`).
    m3_fpu: AtomicU64,
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
            tick_count: AtomicU32::new(0),
            switches: AtomicU32::new(0),
            current: AtomicU32::new(0),
            preempt: AtomicU32::new(0),
            counts: [const { AtomicU64::new(0) }; 2],
            saved: [const { AtomicU64::new(0) }; 3],
            corrupt: AtomicU32::new(0),
            lockdep: AtomicU64::new(0),
            spun: AtomicU32::new(0),
            deferred: AtomicU32::new(0),
            m3_deferred: AtomicU32::new(0),
            m3_wait_max: AtomicU64::new(0),
            m3_deferred_max: AtomicU32::new(0),
            m3_entering: AtomicBool::new(false),
            sched_cur: AtomicU32::new(HOME),
            sched_state: AtomicU32::new(0),
            resched_count: AtomicU32::new(0),
            cr3_switches: AtomicU32::new(0),
            gs_state: AtomicU32::new(0),
            user_ticks: AtomicU32::new(0),
            scratched: AtomicU32::new(0),
            irq_depth: AtomicU32::new(0),
            lockdep_state: lock::LockdepCell::new(),
            paired: AtomicU32::new(0),
            icr: lock::IrqLock::new(level::ICR, ()),
            icr_sent: AtomicU32::new(0),
            icr_done: AtomicU32::new(0),
            need_resched: AtomicBool::new(false),
            resched_seen: AtomicU32::new(0),
            irq_ready: AtomicBool::new(false),
            shootdowns: AtomicU32::new(0),
            m3_stack: AtomicU64::new(0),
            m3_fpu: AtomicU64::new(0),
        }
    }
}
const _: () = {
    assert!(core::mem::offset_of!(CpuLocal, self_ptr) == 0);
    assert!(core::mem::offset_of!(CpuLocal, ipis) == 12);
};
static CPU_LOCAL: [CpuLocal; MAX_APS] = [const { CpuLocal::new() }; MAX_APS];
/// CPU index 0 (spec §3.1.3): same GS-anchor path as the APs. Kept outside
/// CPU_LOCAL so AP-only lookups (`this_cpu`, IST markers) still miss the BSP.
static BSP_LOCAL: CpuLocal = CpuLocal::new();

/// PS-2 (step 5e): the published round, written under SHOOTDOWN before the
/// IPIs: the space root (0 = the shared kernel half), the first page and the
/// page count, then the generation (Release). The handler acknowledges every
/// generation it sees, whether or not its CR3 needed the flush.
static ROUND_ROOT: AtomicU64 = AtomicU64::new(0);
static ROUND_START: AtomicU64 = AtomicU64::new(0);
static ROUND_PAGES: AtomicU64 = AtomicU64::new(0);
static ROUND_GEN: AtomicU32 = AtomicU32::new(0);
/// The target set of the last round (CPU index bits), for the PS-1 fixture.
static ROUND_TARGETS: AtomicU64 = AtomicU64::new(0);
/// Above this many pages a round reloads CR3 instead of `invlpg` per page.
const ROUND_INVLPG_MAX: u64 = 32;
static SHOOTDOWN: lock::IrqLock<()> = lock::IrqLock::new(level::SHOOTDOWN, ());
/// CPU index bits of every CPU that takes part in rounds (PS-4): the BSP and
/// the online APs; back to the BSP alone after a quarantine.
static ONLINE_CPUS: AtomicU64 = AtomicU64::new(1);
static PROBE: AtomicU32 = AtomicU32::new(0);
/// Frames A (initially behind the window) and B (its replacement).
static SHOOTDOWN_FRAMES: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];
const SHOOTDOWN_VECTOR: usize = 0xe0;
/// Per-AP ack wait of a shootdown (ADR-072); a kill waits as long (spec §6.4).
const SHOOTDOWN_DEADLINE_MS: u32 = 100;
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

/// DG-1 (audit F5): a lock wait longer than this is a deadlock, in every
/// native-smp image.
const LOCK_STUCK_MS: u64 = 4 * SHOOTDOWN_DEADLINE_MS as u64;

/// BSP, IF=0, before the first IPI: establish TSC_PER_MS on PIT channel 2, so
/// a lock wait during bring-up is bounded too (DG-1). Scheduling delay can
/// only lengthen a sample, so the minimum of ten independent 10 ms samples is
/// the closest upper bound; `bsp_timer_init` repeats it before GO (BT-5).
fn calibrate_tsc() {
    let mut per_ms = u64::MAX;
    for _ in 0..10 {
        let start = rdtsc();
        if !wait_us(10_000) {
            serial::fail(b"SMP_TSC_CALIBRATION");
        }
        per_ms = per_ms.min(rdtsc().wrapping_sub(start) / 10);
    }
    TSC_PER_MS.store(per_ms.max(1), Ordering::Release);
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
    // Shootdown (step 5e): save the caller-saved GPRs and let
    // `smp_shootdown` flush, acknowledge and EOI. Kernel code is soft-float
    // with no red zone; 9 pushes on the CPU-aligned 5-word frame keep RSP
    // 16-byte aligned for the call.
    ".globl nos_smp_shootdown_entry",
    "nos_smp_shootdown_entry:",
    arch::gs_entry!("8"),
    "    inc dword ptr gs:[{depth}]",
    "    push rax", "    push rcx", "    push rdx", "    push rsi", "    push rdi",
    "    push r8", "    push r9", "    push r10", "    push r11",
    "    cld",
    "    call {shootdown}",
    "    pop r11", "    pop r10", "    pop r9", "    pop r8", "    pop rdi",
    "    pop rsi", "    pop rdx", "    pop rcx", "    pop rax",
    "    dec dword ptr gs:[{depth}]",
    arch::gs_exit!("nos_iret_user_shootdown"),
    // BT-1 clock: PIT IRQ0 through the 8259 PIC (ExtINT on the BSP's LINT0;
    // no LAPIC EOI). Touches neither GS nor a lock and arrives only at CPL0
    // on the BSP; an AP that takes it refutes BT-1 and stops the VM. Its
    // iretq is the spurious stub's (lint: one reviewed raw iretq).
    ".globl nos_smp_clock_entry",
    "nos_smp_clock_entry:",
    "    push rax",
    "    mov eax, dword ptr [{lapic_id}]",
    "    shr eax, 24",
    "    cmp eax, dword ptr [rip + {bsp}]",
    "    jne 2f",
    "    push rdx",
    // Bind each counted IRQ0 to its entry TSC. The following locked tick
    // publication makes this preceding store visible to the IF=0 observer.
    "    rdtsc",
    "    shl rdx, 32",
    "    or rax, rdx",
    "    mov qword ptr [rip + {clock_edge_tsc}], rax",
    "    lock inc qword ptr [rip + {clock}]",
    "    mov al, 0x20",
    "    out 0x20, al",
    "    pop rdx",
    "    pop rax",
    "    jmp nos_smp_spurious_entry",
    "2:",
    "    and rsp, -16",
    "    call {on_ap}",
    ".popsection",
    eoi = const paging::LAPIC_WINDOW + 0xb0,
    shootdown = sym smp_shootdown,
    depth = const core::mem::offset_of!(CpuLocal, irq_depth),
    lapic_id = const paging::LAPIC_WINDOW + 0x20,
    bsp = sym BSP_APIC_ID,
    clock = sym CLOCK_TICKS,
    clock_edge_tsc = sym CLOCK_EDGE_TSC,
    on_ap = sym smp_clock_on_ap,
);
extern "C" {
    static nos_iret_user_timer: u8;
    fn nos_smp_clock_entry();
    fn nos_smp_ipi_entry();
    fn nos_smp_spurious_entry();
    fn nos_smp_shootdown_entry();
    fn nos_smp_timer_entry();
    fn nos_smp_resched_entry();
    #[cfg(feature = "native-smp-m3")]
    #[link_name = "m3_reschedule"]
    fn m3_reschedule_entry();
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
    // xAPIC; its ICR only in `send_ipi`, under that CPU's ICR lock.
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
    // IQ-1 (audit F19): a handler on this CPU sends too; between the two
    // writes it would replace our destination. The race control omits it.
    let _icr = (!cfg!(feature = "test-smp-icr-race")).then(|| anchored().icr.lock_irqsave());
    if !icr_idle() {
        return false;
    }
    lapic_write(0x310, apic_id << 24);
    if cfg!(feature = "test-smp-icr-race") {
        icr_race_window();
    }
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
    // BT-2/S3: only 0xd0/0xe0/0xe8/0xf0/0xff can reach this AP, all gated.
    local.irq_ready.store(true, Ordering::Relaxed);
    // Cache the window's current translation (frame A) in this CPU's TLB.
    local.before.store(read_window(), Ordering::Relaxed);
    context.state.store(IPI_READY, Ordering::Release);
    if cfg!(feature = "test-smp-ap-ipi-deaf") && first {
        // Online but never accepts an interrupt: the BSP must quarantine all.
        park();
    }
    if cfg!(feature = "test-smp-quarantine-lock-held") && local.slot.load(Ordering::Relaxed) == 1 {
        // AP2 goes deaf holding KERNEL: IQ-3 must stop the system, not INIT.
        core::mem::forget(lock::KERNEL.lock_irqsave());
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
                    && local.tick_count.load(Ordering::Relaxed) != 0 =>
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
        // ICR phase (audit F19): AP1 and AP2 exchange 0xe8 under their own ticks.
        let slot = local.slot.load(Ordering::Relaxed) as usize;
        if ICR_RUN.load(Ordering::Acquire)
            && slot < 2
            && local.icr_done.load(Ordering::Relaxed) == 0
        {
            icr_work(local, slot);
        }
        // Spin phase (step 5b): wait for the KERNEL-level lock the BSP holds.
        // IF is 0 from the store of 1 on; only the lock's own window opens it.
        if SPIN.load(Ordering::Acquire) && local.spun.load(Ordering::Relaxed) == 0 {
            local.spun.store(1, Ordering::Release);
            *SPIN_LOCK.lock_irqsave() += 1;
            local.spun.store(2, Ordering::Release);
        }
        // Step 5c-2: load for the clock check, then AP1's part of the BSP
        // spin and idle phases (each phase word is reset by the BSP after it).
        let mut clock_work = 0u64;
        let mut clock_mix = slot as u64 + 1;
        while CLOCK_LOAD.load(Ordering::Acquire) {
            // Keep each AP continuously busy, but perform useful dependent
            // integer work between observer/hint steps. TCG exits its CPU
            // loop on every PAUSE; one after only an increment/store would
            // make this observer repeatedly contend for QEMU's global BQL.
            for _ in 0..64 {
                clock_mix = core::hint::black_box(
                    clock_mix
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .rotate_left(17)
                        .wrapping_add(1),
                );
            }
            // This AP is the sole writer of its progress counter. A shared
            // locked RMW would add observer contention to the CPU-load test.
            clock_work = clock_work.wrapping_add(1);
            CLOCK_WORK[slot].0.store(clock_work, Ordering::Relaxed);
            spin_loop();
        }
        if slot == 0 {
            if BSP_SPIN.load(Ordering::Acquire) == STEP_RUN {
                bsp_spin_work();
            }
            match IDLE_PHASE.load(Ordering::Acquire) {
                STEP_RUN => idle_wake_work(),
                IDLE_DEFER => idle_defer_work(),
                _ => {}
            }
        }
        if cfg!(feature = "test-smp-lock-stuck") && first && STUCK_GO.load(Ordering::Acquire) {
            drop(STUCK_TEST.lock_irqsave());
        }
        #[cfg(feature = "native-smp-m3")]
        if M3_GO.load(Ordering::Acquire) {
            m3_ap_enter(local);
        }
        if PARK.load(Ordering::Acquire) {
            break;
        }
        // STI's one-instruction shadow makes `sti; hlt` atomic, so an IPI
        // that becomes pending after the PARK check still wakes this CPU.
        // Local sources are masked; see the function comment for others.
        idle_halt();
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
    // Step 5e: the unmap race's reader lost its translation of space A's page.
    if vector == 14
        && cs == 8
        && error & 1 == 0
        && rip == nos_smp_race_read as usize as u64
        && cr2 >> 12 == SPACE_VA >> 12
        && RACE_RUN.load(Ordering::Acquire)
        && on_ap()
    {
        race_fixup(frame);
    }
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
    // BT-2: the PIC's IRQ0 (the clock) and its spurious IRQ7, remapped to 0x20.
    arch::smp_gate(CLOCK_VECTOR, nos_smp_clock_entry as usize as u64);
    arch::smp_gate(CLOCK_VECTOR + 7, nos_smp_spurious_entry as usize as u64);
    BSP_APIC_ID.store(bsp_id, Ordering::Relaxed);
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
    ONLINE_CPUS.store((1 << (online + 1)) - 1, Ordering::Release);
    serial::write(b"SMP:ONLINE:");
    decimal(online as u64 + 1);
    serial::write(b"/");
    decimal(cpus.len() as u64);
    serial::write(b"\n");
    bsp_timer_init();
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
/// Every AP is checked first (IQ-3): one AP's RUNQ lock may be held by another.
fn quarantine_all(online: usize, total: usize) {
    (0..online).for_each(stop_if_held);
    (0..online).for_each(quarantine_cpu);
    ONLINE_CPUS.store(1, Ordering::Release);
    serial::write(b"SMP:QUARANTINED:APS=");
    decimal(online as u64);
    serial::write(b"\nSMP:ONLINE:1/");
    decimal(total as u64);
    serial::write(b"\n");
}

/// IQ-3 (ADR-079): the lowest lock level CPU index `cpu` holds, over every
/// shared IrqLock in level order. FRAMES is no lock yet: the BSP alone holds
/// `&mut Frames` (spec 07 §5); it must join here once it becomes one.
fn held_by(cpu: usize) -> Option<u8> {
    let cpu = Some(cpu as u8);
    core::iter::once(&lock::KERNEL)
        .chain(ASPACE.iter())
        .map(|lock| (lock.owner(), lock.level()))
        .chain(RUNQ.iter().map(|queue| (queue.owner(), queue.level())))
        .chain(core::iter::once((SHOOTDOWN.owner(), SHOOTDOWN.level())))
        .find_map(|(owner, level)| (owner == cpu).then_some(level))
}

/// IQ-3: a lock held by a CPU that INIT resets stays held forever, and the
/// state under it is unknown: stop (fail-closed), never break the lock.
fn stop_if_held(slot: usize) {
    let Some(level) = held_by(slot + 1) else {
        return;
    };
    serial::write(b"SMP:QUARANTINE:LOCK_HELD:");
    decimal(u64::from(level));
    serial::write(b":CPU=");
    decimal(slot as u64 + 1);
    serial::write(b"\n");
    if cfg!(feature = "test-smp-quarantine-lock-held") {
        serial::exit(0x10);
    }
    serial::fail(b"SMP_QUARANTINE_LOCK_HELD")
}

/// Threads of a quarantined CPU (IQ-3), one bit per tid: READY ones handed to
/// the BSP, the one it ran (or was still on the stack of) crashed.
// shortcut: the BSP has no run queue; step 5c-2 gave it a timer and an idle
// context, not a queue (spec 06 §3). ADOPTED is recorded and not yet run; the
// step that gives the BSP a run queue (5d-2 or 5f) must drain it.
static ADOPTED: AtomicU64 = AtomicU64::new(0);
static CRASHED: AtomicU64 = AtomicU64::new(0);
const _: () = assert!(SCHED_THREADS <= 64);

/// IQ-3/IQ-4: reset one deaf AP, on the BSP only. Its owner fields are checked
/// again once INIT took effect: a deaf AP that still ran may have taken a lock
/// after the first check.
fn quarantine_cpu(slot: usize) {
    if on_ap() {
        serial::fail(b"SMP_QUARANTINE_ON_AP");
    }
    let context = &NOS_SMP_AP_CONTEXTS[slot];
    stop_if_held(slot);
    if !send_ipi(context.apic_id.load(Ordering::Relaxed), IPI_INIT) {
        serial::fail(b"SMP_AP_QUARANTINE");
    }
    context.state.store(ABANDONED, Ordering::Release);
    // ICR idle is acceptance by the bus, not completion of INIT in the target;
    // allow it to take effect. A stray store by a resetting AP only touches its
    // own context, which nothing reads anymore.
    wait_us(1_000);
    stop_if_held(slot);
    {
        let mut queue = RUNQ[slot].lock_irqsave();
        while let Some(tid) = queue.pop(&THREAD_QUEUED) {
            let index = tid as usize;
            if THREAD_STATE[index].load(Ordering::Acquire) == sched::READY
                && !THREAD_ON_CPU[index].load(Ordering::Acquire)
            {
                ADOPTED.fetch_or(1 << tid, Ordering::Release);
            } else {
                crash(tid);
            }
        }
    }
    let current = CPU_LOCAL[slot].sched_cur.load(Ordering::Acquire);
    if current != HOME {
        crash(current);
    }
}

/// The context of a thread on a reset CPU is gone: it dies, never migrates.
// shortcut: these are kernel threads without a process, so the K05 fault route
// has nothing to end; ring-3 threads on APs (step 5d-2) must take that route.
fn crash(tid: u32) {
    sched::kill_request(&THREAD_STATE[tid as usize]).ok();
    CRASHED.fetch_or(1 << tid, Ordering::Release);
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
        || !clock(online, total)
        || !bsp_preempt(online)
        || !bsp_spin(online, total)
        || !idle(online, total)
    {
        return;
    }
    // After the kernel-only audit in `shootdown`: user aliases now exist
    // until `proc_shootdown` exits both spaces and audits again.
    let spaces = build_spaces(frames, paging);
    if !schedule(online, total)
        || !block_wake_kill(online, total)
        || !scratch(online, total, frames)
        || !lock_pairs(online, total)
        || !icr(online, total)
        || !proc_shootdown(online, total, frames, paging, spaces)
        || !gs_user(online, total)
    {
        return;
    }
    if cfg!(feature = "native-smp-m3") {
        // Step 5d-2: the APs stay in their home loop until `m3_go`.
        serial::write(b"SMP:M3:WAITING:APS=");
        decimal(online as u64);
        serial::write(b"\n");
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
    } else if let Err(failed) =
        shootdown_kernel(paging::SHOOTDOWN_WINDOW, 1).and_then(|()| shootdown_kernel(old, 1))
    {
        return frame_quarantined(failed, online, total);
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

/// A round that some target did not acknowledge: its frames stay owned and
/// mapped in the metadata (quarantine, PS-3), then the IQ-3 policy of step 05.
/// Call with no lock held: `quarantine_all` takes the run queues.
fn frame_quarantined(failed: Quarantined, online: usize, total: usize) -> bool {
    serial::write(b"SMP:SHOOTDOWN:TIMEOUT:APIC=");
    decimal(u64::from(failed.0));
    serial::write(b"\nSMP:SHOOTDOWN:FRAME_QUARANTINED\n");
    quarantine_all(online, total);
    false
}

/// A round some target did not acknowledge in time: the APIC ID of the first.
pub struct Quarantined(pub u32);

/// PS-1 (spec §7.1 steps 3-6) for space `space` (the fixture's pid): after the
/// caller changed the PTEs, every CPU that may still cache the old ones is in
/// `ACTIVE[space]` (switch order §6.3 step 5); exactly those, minus this CPU,
/// get the round for `pages` pages from `start`.
pub fn shootdown_process(space: usize, start: u64, pages: u64) -> Result<(), Quarantined> {
    if cfg!(feature = "test-smp-proc-no-shootdown") {
        // Control only: the unmap race must then show the reused frame.
        return Ok(());
    }
    fence(Ordering::SeqCst);
    let targets = ACTIVE[space].load(Ordering::SeqCst) & !(1 << cpu_index());
    round(
        SPACE_ROOT[space].load(Ordering::Relaxed),
        start,
        pages,
        targets,
    )
}

/// PS-4: a change in the shared kernel half reaches every online CPU (the BSP
/// too, for an AP initiator). Every space shares the kernel half through one
/// PDPT entry (`Paging::new_user`), so no space can miss a kernel mapping.
pub fn shootdown_kernel(start: u64, pages: u64) -> Result<(), Quarantined> {
    let targets = ONLINE_CPUS.load(Ordering::Acquire) & !(1 << cpu_index());
    round(0, start, pages, targets)
}

/// M3 caller holds KERNEL and its process ASPACE; process active bits are
/// published before CR3 selection and cleared only after leaving that root.
#[cfg(feature = "native-smp-m3-unmap")]
pub fn shootdown_m3(root: u64, start: u64, pages: u64, active: u64) -> Result<(), Quarantined> {
    assert!(root != 0 && root & (PAGE - 1) == 0);
    assert_eq!(active & !online_mask(), 0);
    if cfg!(feature = "test-smp-m3-no-shootdown") {
        return Ok(());
    }
    fence(Ordering::SeqCst);
    round(root, start, pages, active & !(1 << cpu_index()))
}

/// Mask only this CPU's process timer for the bounded unmap fixture. The
/// process shootdown vector remains unmasked; CR3 stays unchanged until its
/// real page fault (or the deliberately stale-read negative control).
#[cfg(feature = "native-smp-m3-unmap")]
pub fn m3_unmap_timer_mask(masked: bool) {
    assert!(cpu_index() != 0);
    lapic_write(
        LVT_TIMER,
        TIMER_PERIODIC | TIMER_VECTOR as u32 | if masked { MASKED } else { 0 },
    );
}

fn apic_of_cpu(cpu: usize) -> u32 {
    match cpu {
        0 => BSP_APIC_ID.load(Ordering::Relaxed),
        cpu => apic_of(cpu - 1),
    }
}

fn local_of_cpu(cpu: usize) -> &'static CpuLocal {
    match cpu {
        0 => &BSP_LOCAL,
        cpu => &CPU_LOCAL[cpu - 1],
    }
}

/// PS-2: publish one round under SHOOTDOWN, send 0xe0 to each target, wait
/// (IF=0) for each target's ack of this generation. Any CPU may start one:
/// the deadline is calibrated TSC, not the BSP-owned PIT.
fn round(root: u64, start: u64, pages: u64, targets: u64) -> Result<(), Quarantined> {
    let _round = SHOOTDOWN.lock_irqsave();
    ROUND_ROOT.store(root, Ordering::Relaxed);
    ROUND_START.store(start, Ordering::Relaxed);
    ROUND_PAGES.store(pages, Ordering::Relaxed);
    ROUND_TARGETS.store(targets, Ordering::Relaxed);
    let generation = ROUND_GEN.fetch_add(1, Ordering::AcqRel) + 1;
    let cpus = || (0..MAX_CPUS).filter(move |cpu| targets >> cpu & 1 != 0);
    if let Some(cpu) =
        cpus().find(|&cpu| !send_ipi(apic_of_cpu(cpu), IPI_FIXED | SHOOTDOWN_VECTOR as u32))
    {
        return Err(Quarantined(apic_of_cpu(cpu)));
    }
    // AC-3: one named deadline bounds the complete generation. Waiting a
    // fresh interval per target would silently multiply the contract by the
    // number of CPUs.
    let end = rdtsc()
        .saturating_add(u64::from(SHOOTDOWN_DEADLINE_MS) * TSC_PER_MS.load(Ordering::Acquire));
    loop {
        if let Some(cpu) = cpus()
            .find(|&cpu| local_of_cpu(cpu).shootdown_ack.load(Ordering::Acquire) != generation)
        {
            if rdtsc() > end {
                return Err(Quarantined(apic_of_cpu(cpu)));
            }
            spin_loop();
        } else {
            return Ok(());
        }
    }
}

/// The 0xe0 handler (PS-2), IF=0 on this CPU's anchor: flush if the round is
/// for the kernel half or for the space whose root this CPU has loaded (a CR3
/// load without PCID already flushed any other), then always acknowledge.
#[no_mangle]
extern "C" fn smp_shootdown() {
    let Some(local) = any_cpu() else {
        serial::fail(b"SMP_SHOOTDOWN_ANCHOR");
    };
    let generation = ROUND_GEN.load(Ordering::Acquire);
    let (root, start, pages) = (
        ROUND_ROOT.load(Ordering::Relaxed),
        ROUND_START.load(Ordering::Relaxed),
        ROUND_PAGES.load(Ordering::Relaxed),
    );
    if cfg!(feature = "test-smp-proc-shootdown-deaf")
        && RACE_RUN.load(Ordering::Acquire)
        && root == 0
    {
        // Spec 07's deaf control requires a lock-free AP. Arm that fixture
        // through a real, normally acknowledged kernel round before unmap.
        // An already pending timer/reschedule entry must also stay in this
        // context, so publish admission closure before masking the timer.
        assert_eq!(local.lockdep.load(Ordering::Acquire), 0);
        assert!(held_by(cpu_index()).is_none());
        PROC_DEAF_FROZEN.fetch_or(1 << cpu_index(), Ordering::Release);
        lapic_write(LVT_TIMER, MASKED | TIMER_VECTOR as u32);
    }
    // Control: AP1 ignores rounds for space A (no flush, no ack).
    let deaf = (cfg!(feature = "test-smp-proc-shootdown-deaf")
        && local.slot.load(Ordering::Relaxed) == 0
        && root != 0
        && root == SPACE_ROOT[0].load(Ordering::Relaxed))
        || {
            #[cfg(feature = "native-smp-m3")]
            {
                cfg!(feature = "test-smp-m3-recovery")
                    && M3_GO.load(Ordering::Acquire)
                    && cpu_index() == 2
            }
            #[cfg(not(feature = "native-smp-m3"))]
            {
                false
            }
        };
    if !deaf {
        let cr3: u64;
        // SAFETY: CPL0 register read.
        unsafe { asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack)) };
        if root == 0 || cr3 & ROOT_MASK == root {
            if pages > ROUND_INVLPG_MAX {
                // SAFETY: reloads this CPU's own current root; PCID and PGE
                // are off, so this flushes every translation of this CPU.
                unsafe { asm!("mov cr3, {}", in(reg) cr3, options(nostack)) };
            } else {
                for page in 0..pages {
                    // SAFETY: invalidates one TLB entry of this CPU only.
                    unsafe { asm!("invlpg [{}]", in(reg) start + page * PAGE, options(nostack)) };
                }
            }
        }
        local.shootdowns.fetch_add(1, Ordering::Relaxed);
        local.shootdown_ack.store(generation, Ordering::Release);
    }
    lapic_write(0xb0, 0);
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
fn on_ap() -> bool {
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
/// is masked and the home loop resumes; threads are never run again. The BSP
/// (step 5c-2) takes the same entry, deferral and switch rules (BT-2).
#[no_mangle]
extern "C" fn smp_preempt(rsp: u64) -> Resume {
    let Some(local) = any_cpu() else {
        // A timer interrupt on a CPU without an anchor is impossible by
        // construction (anchors precede every arm); stop rather than run
        // unknown state.
        if GS.load(Ordering::Acquire) != 0 {
            serial::write(b"SMP:GS:ANCHOR_LOST\n");
        }
        serial::fail(b"SMP_PREEMPT_ANCHOR");
    };
    let bsp = ptr::eq(local, &BSP_LOCAL);
    // BT-4 (audit F11): the 0xe8 stub counted `resched_count` before joining
    // this body, with IF=0 throughout, so a changed count means this entry is
    // that 0xe8; anything else is a 0xd0 tick. Only a 0xe8 asks to switch.
    let resched = local.resched_count.load(Ordering::Relaxed);
    if local.resched_seen.swap(resched, Ordering::Relaxed) != resched {
        local.need_resched.store(true, Ordering::Relaxed);
        if bsp {
            BSP_WAKE_TSC
                .compare_exchange(0, rdtsc(), Ordering::Relaxed, Ordering::Relaxed)
                .ok();
        }
    } else {
        local.tick_count.fetch_add(1, Ordering::Relaxed);
    }
    // F19 fixture: during the ICR phase every AP entry sends one IPI from the
    // handler (to itself, 0xf0) and resumes the interrupted context; it never
    // switches, so its 0xe8 are only counted.
    if !bsp && ICR_RUN.load(Ordering::Acquire) {
        local.need_resched.store(false, Ordering::Relaxed);
        let own = apic_of(local.slot.load(Ordering::Relaxed) as usize);
        if !send_ipi(own, IPI_FIXED | IPI_VECTOR as u32) {
            ICR_FAULT.store(true, Ordering::Release);
        }
        return Resume { rsp, left: 0 };
    }
    // LD-1 control: from the first 0xe8 on, this handler takes KERNEL and
    // must stop with SMP:LOCKDEP:IN_HANDLER:10 before spinning.
    if cfg!(feature = "test-smp-lock-in-handler")
        && !bsp
        && local.resched_count.load(Ordering::Relaxed) != 0
    {
        drop(lock::KERNEL.lock_irqsave());
    }
    // Step 5c (INV-5c): a tick can only arrive with locks held or awaited in
    // the 5b `sti; pause; cli` window. Lockdep is per CPU, not per thread, so
    // switching (or STOP's switch home) waits for a later tick. A deferred
    // 0xe8 stays in `need_resched` (BT-4).
    let busy = local.lockdep.load(Ordering::Relaxed) != 0;
    if busy && !cfg!(feature = "test-smp-held-across-switch") {
        local.deferred.fetch_add(1, Ordering::Release);
        return Resume { rsp, left: 0 };
    }
    // Every path below is a switch point, so a pending reschedule is honoured.
    local.need_resched.store(false, Ordering::Relaxed);
    if bsp {
        return bsp_switch(local, rsp, busy);
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
    // The BSP's slot is u32::MAX: wrap instead of overflowing.
    let seed = (u64::from(local.slot.load(Ordering::Relaxed)) << 8 | (index as u64 + 1))
        .wrapping_mul(0x0101_0101_0101_0101);
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
    match any_cpu() {
        Some(local) => thread_body(local, 0),
        None => park(),
    }
}
extern "C" fn nos_smp_thread1() -> ! {
    match any_cpu() {
        Some(local) => thread_body(local, 1),
        None => park(),
    }
}

/// Build an initial frame on a fresh thread stack so that the timer stub's
/// pops and `iretq` enter `entry` with IF=1 and a SysV-aligned stack.
fn thread_frame(slot: usize, kind: usize, entry: u64, arg: u64) -> u64 {
    frame_below(stack_base(slot, kind) + STACK_TOP, entry, arg)
}

/// `thread_frame` for the guarded thread stack ending at `top`.
fn frame_below(top: u64, entry: u64, arg: u64) -> u64 {
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
            local.tick_count.load(Ordering::Acquire) >= MIN_TICKS
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
            local.tick_count.load(Ordering::Acquire),
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
    if let Err(failed) = shootdown_kernel(paging::SHOOTDOWN_WINDOW, 1) {
        serial::write(b"SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=");
        decimal(u64::from(failed.0));
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

// Step 5c-2 (spec 06, ADR-080): the BSP gets the PIT clock (BT-1), its own
// 0xd0 timer under the AP rules (BT-2), an idle context (BT-3), reschedule as
// a flag instead of a tick (BT-4) and a calibrated LAPIC/PIT ratio (BT-5).
const CLOCK_VECTOR: usize = 0x20;
/// PIT channel 0 divisor: 1 193 182 Hz / 1193 = 1000.15 Hz (0.99985 ms/tick).
const PIT_DIVISOR: u16 = 1193;
const PIT_HZ: u64 = 1_193_182;
const TIMER_CURRENT: u64 = 0x390;
/// BT-1: PIT IRQ0 ticks since `bsp_timer_init`; written by the IRQ0 stub only.
static CLOCK_TICKS: AtomicU64 = AtomicU64::new(0);
/// TSC captured by the IRQ0 stub immediately before publishing CLOCK_TICKS.
/// This removes only post-interrupt observer latency; every accepted sample
/// still spans actual counted IRQ0 entries and uses the unchanged 1% bound.
static CLOCK_EDGE_TSC: AtomicU64 = AtomicU64::new(0);
static BSP_APIC_ID: AtomicU32 = AtomicU32::new(u32::MAX);
/// TSC ticks per millisecond, measured once by the BSP against PIT channel 2.
/// APs time their bounded waits with it: they never touch the BSP-owned PIT
/// (ADR-070).
// shortcut: assumes a constant-rate TSC that is equal on every CPU (QEMU/TCG,
// KVM); hardware support needs an invariant-TSC check first (H-track).
static TSC_PER_MS: AtomicU64 = AtomicU64::new(0);
/// TSC at the BSP's first 0xe8 of the idle-wake check (0 = none yet).
static BSP_WAKE_TSC: AtomicU64 = AtomicU64::new(0);

fn rdtsc() -> u64 {
    let (low, high): (u32, u32);
    // SAFETY: RDTSC only reads the time-stamp counter into EDX:EAX.
    unsafe { asm!("rdtsc", out("eax") low, out("edx") high, options(nomem, nostack)) };
    u64::from(high) << 32 | u64::from(low)
}

/// Poll `ready` on any CPU for at most `ms` milliseconds of calibrated TSC.
fn within_tsc(ms: u64, ready: impl Fn() -> bool) -> bool {
    let end = rdtsc().saturating_add(ms * TSC_PER_MS.load(Ordering::Acquire));
    loop {
        if ready() {
            return true;
        }
        if rdtsc() > end {
            return ready();
        }
        spin_loop();
    }
}

/// BSP idle context: idle (BT-3) until `ready`, at most `ms` ms of TSC. The
/// clock (IRQ0, ~1 kHz) ends every `hlt`, so the bound is always checked.
fn idle_until(ms: u64, ready: impl Fn() -> bool) -> bool {
    let end = rdtsc().saturating_add(ms * TSC_PER_MS.load(Ordering::Acquire));
    loop {
        if ready() {
            return true;
        }
        if rdtsc() > end {
            return ready();
        }
        idle_halt();
    }
}

/// BT-3: the only `sti; hlt; cli`, in a CPU's idle (home) context. It runs
/// under the kernel CR3 with an empty lockdep set, or the VM stops.
fn idle_halt() {
    let cr3: u64;
    // SAFETY: CPL0 register read.
    unsafe { asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack)) };
    let kernel = KERNEL_CR3.load(Ordering::Relaxed);
    if lockdep().load(Ordering::Relaxed) != 0 || (kernel != 0 && (cr3 ^ kernel) & ROOT_MASK != 0) {
        serial::fail(b"SMP_IDLE_STATE");
    }
    // SAFETY: CPL0 idle context with IF=0 and every unmasked source gated.
    // STI's shadow covers HLT, so an interrupt that became pending before HLT
    // still ends it; handlers may change memory, hence no `nomem`.
    unsafe { asm!("sti", "hlt", "cli") };
}

/// BSP, IF=0, after bring-up and before GO (spec 06 §3, `bsp_timer_init`).
fn bsp_timer_init() {
    let cr3: u64;
    // SAFETY: CPL0 register read.
    unsafe { asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack)) };
    KERNEL_CR3.store(cr3, Ordering::Relaxed);
    // BT-5: the LAPIC timer (divide 16, as on the APs) and the TSC, both
    // against 100 ms of PIT channel 2.
    lapic_write(TIMER_DIVIDE, 0x3);
    lapic_write(LVT_TIMER, MASKED | TIMER_VECTOR as u32);
    lapic_write(TIMER_INITIAL, u32::MAX);
    let mut per_ms = u64::MAX;
    for _ in 0..10 {
        let start = rdtsc();
        if !wait_us(10_000) {
            serial::fail(b"SMP_TIMER_CALIBRATION");
        }
        per_ms = per_ms.min(rdtsc().wrapping_sub(start) / 10);
    }
    let lapic = u64::from(u32::MAX - lapic_read(TIMER_CURRENT));
    lapic_write(TIMER_INITIAL, 0);
    TSC_PER_MS.store(per_ms.max(1), Ordering::Release);
    // LAPIC ticks per PIT IRQ0 tick: 100 ms hold PIT_HZ / 10 / PIT_DIVISOR.
    serial::write(b"SMP:TIMER:RATIO:");
    decimal(lapic * 10 * u64::from(PIT_DIVISOR) / PIT_HZ);
    serial::write(b"\n");
    // BT-2: LINT0 delivers the PIC (ExtINT), LINT1 the NMI (vector 2, gated
    // by the M2 IDT); every other local source but the still-masked timer is
    // masked. TPR 0 and the spurious vector were set in `start`.
    let max_lvt = (lapic_read(0x30) >> 16) & 0xff;
    for (lvt, minimum, value) in [
        (0x350, 0, 0x700),
        (0x360, 0, 0x400),
        (0x370, 0, MASKED),
        (0x340, 4, MASKED),
        (0x330, 5, MASKED),
        (0x2f0, 6, MASKED),
    ] {
        if max_lvt >= minimum {
            lapic_write(lvt, value);
        }
    }
    lapic_write(0x80, 0);
    // BT-1: the 8259 pair remapped to 0x20/0x28 with everything but IRQ0
    // masked; PIT channel 0 as a rate generator. Channel 2 (`wait_us`) keeps
    // its own control word; IRQ0 touches neither.
    for (port, value) in [
        (0x20, 0x11),
        (0xa0, 0x11),
        (0x21, CLOCK_VECTOR as u8),
        (0xa1, CLOCK_VECTOR as u8 + 8),
        (0x21, 4),
        (0xa1, 2),
        (0x21, 1),
        (0xa1, 1),
        (0x21, 0xfe),
        (0xa1, 0xff),
    ] {
        outb(port, value);
    }
    outb(0x43, 0x34);
    outb(0x40, PIT_DIVISOR as u8);
    outb(0x40, (PIT_DIVISOR >> 8) as u8);
    // S3: IRQ0/IRQ7 (0x20/0x27), 0xd0, 0xe0, 0xe8, 0xf0 and 0xff are gated
    // (`start`), so a BSP waiter may now open its window. The control keeps
    // the BSP window shut.
    if !cfg!(feature = "test-smp-bsp-spin-irq-off") {
        BSP_LOCAL.irq_ready.store(true, Ordering::Relaxed);
    }
}

/// BT-1 refuted: an AP took the PIT IRQ0 (terminal, from the IRQ0 stub).
extern "C" fn smp_clock_on_ap() -> ! {
    serial::write(b"SMP:CLOCK:AP_IRQ0\n");
    serial::fail(b"SMP_CLOCK_AP_IRQ0")
}

/// BT-2/S3: this CPU may open a KERNEL/ASPACE wait's interrupt window.
pub fn irq_ready() -> bool {
    anchored().irq_ready.load(Ordering::Relaxed)
}

/// BT-4: a guard released this CPU's last lock and is about to restore IF=1
/// outside a handler (a kernel thread's pseudo-user context, BT-6). A 0xe8
/// that INV-5c deferred meanwhile becomes a self 0xe8, taken right after that
/// `sti`. IF=0 here, so no handler interleaves the swap; the ICR guard of the
/// send restores IF=0 and calls back with the flag already clear.
pub fn resched_on_release() {
    let local = anchored();
    if local.need_resched.swap(false, Ordering::Relaxed)
        && !send_ipi(lapic_read(0x20) >> 24, IPI_FIXED | RESCHED_VECTOR as u32)
    {
        serial::fail(b"SMP_RESCHED_ON_RELEASE");
    }
}

/// BSP phase of `smp_preempt`: 0 none (stay), ROTATE home -> thread0 ->
/// thread1 -> home, DEFER home <-> the deferred-resched thread.
static BSP_PHASE: AtomicU32 = AtomicU32::new(0);
const BSP_ROTATE: u32 = 1;
const BSP_DEFER: u32 = 2;

/// The BSP's switch (IF=0, lockdep empty outside the held-across control).
/// Home is the orchestrator, so unlike an AP it takes part in the rotation.
fn bsp_switch(local: &CpuLocal, rsp: u64, busy: bool) -> Resume {
    let current = local.current.load(Ordering::Relaxed) as usize;
    local.saved[current].store(rsp, Ordering::Relaxed);
    let next = match BSP_PHASE.load(Ordering::Acquire) {
        BSP_ROTATE => (current + 1) % 3,
        BSP_DEFER => usize::from(current == 0),
        _ => current,
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

/// Five one-second IRQ0 samples for the sustained clock check.
const CLOCK_SEGMENT: u64 = 1_000;
const CLOCK_SEGMENTS: usize = 5;
const CLOCK_SPAN: u64 = CLOCK_SEGMENT * CLOCK_SEGMENTS as u64;
static CLOCK_LOAD: AtomicBool = AtomicBool::new(false);
// Keep independent AP progress words on separate 64-byte lines. The load
// remains a continuously busy loop on every AP; no inter-AP synchronization
// is needed to count each AP's own work. KERNEL/ticketlock tests are separate.
#[repr(align(64))]
struct ClockWork(AtomicU64);
const _: () = assert!(core::mem::size_of::<ClockWork>() == 64);
const _: () = assert!(core::mem::align_of::<ClockWork>() == 64);
static CLOCK_WORK: [ClockWork; MAX_CPUS - 1] =
    [const { ClockWork(AtomicU64::new(0)) }; MAX_CPUS - 1];

/// Idle until the clock reaches `target`; snapshot (ticks, IRQ-entry TSC,
/// observer TSC), or None at `bound`. IF=0 keeps IRQ0 from changing either
/// edge value between the reads.
/// Every read checks that the clock never goes back.
fn clock_at(target: u64, bound: u64, last: &mut u64) -> Option<(u64, u64, u64)> {
    loop {
        let now = CLOCK_TICKS.load(Ordering::Acquire);
        if now < *last {
            serial::fail(b"SMP_CLOCK_NOT_MONOTONIC");
        }
        *last = now;
        if now >= target {
            let edge = CLOCK_EDGE_TSC.load(Ordering::Acquire);
            if edge == 0 {
                serial::fail(b"SMP_CLOCK_EDGE_TSC_MISSING");
            }
            return Some((now, edge, rdtsc()));
        }
        if rdtsc() > bound {
            return None;
        }
        idle_halt();
    }
}

/// BT-1 under SMP load (spec 06 §4 "Klok"): every AP spins while the BSP
/// idles through CLOCK_SPAN IRQ0 ticks; on the PIT-calibrated TSC the median
/// one-second segment must last the PIT's own period within 1%, and the clock
/// never goes back. One scheduling delay cannot dominate the result, while
/// sustained drift affects at least three segments. Lost IRQ0s lengthen their
/// segments.
fn clock(online: usize, total: usize) -> bool {
    // This phase runs once per boot, before any AP enters the load loop.
    for work in &CLOCK_WORK {
        work.0.store(0, Ordering::Relaxed);
    }
    CLOCK_LOAD.store(true, Ordering::Release);
    for context in &NOS_SMP_AP_CONTEXTS[..online] {
        let apic = context.apic_id.load(Ordering::Relaxed);
        if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32) {
            CLOCK_LOAD.store(false, Ordering::Release);
            serial::write(b"SMP:CLOCK:TIMEOUT:APIC=");
            decimal(u64::from(apic));
            serial::write(b"\n");
            quarantine_all(online, total);
            return false;
        }
    }
    let per_ms = TSC_PER_MS.load(Ordering::Acquire);
    let bound = rdtsc().saturating_add(4 * CLOCK_SPAN * per_ms);
    let mut last = CLOCK_TICKS.load(Ordering::Acquire);
    let first = last + 1;
    let mut edge = clock_at(first, bound, &mut last);
    let mut samples = [0u64; CLOCK_SEGMENTS];
    let mut ticks = [0u64; CLOCK_SEGMENTS];
    let mut cycles = [0u64; CLOCK_SEGMENTS];
    let mut observer_start_lag = [0u64; CLOCK_SEGMENTS];
    let mut observer_end_lag = [0u64; CLOCK_SEGMENTS];
    for (index, sample) in samples.iter_mut().enumerate() {
        edge = edge.and_then(|start| {
            clock_at(first + (index as u64 + 1) * CLOCK_SEGMENT, bound, &mut last).map(|end| {
                let tick_delta = end.0 - start.0;
                let tsc_delta = end.1 - start.1;
                if tick_delta == 0 {
                    serial::fail(b"SMP_CLOCK_NO_PROGRESS");
                }
                ticks[index] = tick_delta;
                cycles[index] = tsc_delta;
                observer_start_lag[index] = start.2.saturating_sub(start.1);
                observer_end_lag[index] = end.2.saturating_sub(end.1);
                // HLT may return after the target tick. Normalize using the
                // actual IRQ0 count, never the requested count. This does not
                // synthesize lost interrupts: sustained IRQ loss still drifts.
                *sample = (u128::from(tsc_delta) * 1000 * u128::from(CLOCK_SEGMENT)
                    / (u128::from(per_ms) * u128::from(tick_delta)))
                    as u64;
                end
            })
        });
    }
    CLOCK_LOAD.store(false, Ordering::Release);
    if edge.is_none() {
        serial::write(b"SMP:CLOCK:STOPPED\n");
        serial::fail(b"SMP_CLOCK_STOPPED")
    }
    let mut ordered = samples;
    ordered.sort_unstable();
    let us = ordered[CLOCK_SEGMENTS / 2];
    let expected = CLOCK_SEGMENT * u64::from(PIT_DIVISOR) * 1_000_000 / PIT_HZ;
    if us.abs_diff(expected) * 100 > expected {
        serial::write(b"SMP:CLOCK:CALIBRATION:TSC_PER_MS=");
        decimal(per_ms);
        serial::write(b"\n");
        for index in 0..CLOCK_SEGMENTS {
            serial::write(b"SMP:CLOCK:SAMPLE=");
            decimal(index as u64);
            serial::write(b":TICKS=");
            decimal(ticks[index]);
            serial::write(b":TSC=");
            decimal(cycles[index]);
            serial::write(b":NORMALIZED_US=");
            decimal(samples[index]);
            serial::write(b":OBSERVER_START_LAG_TSC=");
            decimal(observer_start_lag[index]);
            serial::write(b":OBSERVER_END_LAG_TSC=");
            decimal(observer_end_lag[index]);
            serial::write(b"\n");
        }
        serial::write(b"SMP:CLOCK:DRIFT:US=");
        decimal(us);
        serial::write(b"\n");
        serial::fail(b"SMP_CLOCK_DRIFT");
    }
    // Prove work on every online AP, rather than allowing one busy AP to
    // satisfy a single shared counter for all of its peers.
    if CLOCK_WORK[..online]
        .iter()
        .any(|work| work.0.load(Ordering::Acquire) == 0)
    {
        serial::fail(b"SMP_CLOCK_NO_LOAD");
    }
    serial::write(b"SMP:CLOCK:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    true
}

/// BT-2 on the BSP (spec 06 §4 "BSP-preëmptie", step 4's proof on its own
/// CPU): two kernel threads with register canaries and the home context
/// share the BSP under its 0xd0 timer; home checks progress in its slices and
/// idles (BT-3) otherwise. Only this CPU is involved, so a stall is a broken
/// invariant, not a deaf AP: fail closed.
fn bsp_preempt(online: usize) -> bool {
    let local = &BSP_LOCAL;
    let entries = [
        nos_smp_thread0 as usize as u64,
        nos_smp_thread1 as usize as u64,
    ];
    for (index, entry) in entries.into_iter().enumerate() {
        let frame = frame_below(bsp_stack_base(index) + STACK_TOP, entry, 0);
        local.saved[index + 1].store(frame, Ordering::Relaxed);
    }
    local.current.store(0, Ordering::Relaxed);
    BSP_PHASE.store(BSP_ROTATE, Ordering::Release);
    start_timer();
    let counts = || {
        local
            .counts
            .each_ref()
            .map(|count| count.load(Ordering::Acquire))
    };
    let progressed = || {
        local.tick_count.load(Ordering::Acquire) >= MIN_TICKS
            && local.switches.load(Ordering::Acquire) >= MIN_SWITCHES
            && counts().iter().all(|&count| count != 0)
    };
    if !idle_until(500, progressed) || {
        let before = counts();
        !idle_until(500, || {
            counts().iter().zip(before).all(|(&now, old)| now > old)
        })
    } {
        serial::fail(b"SMP_BSP_PREEMPT_TIMEOUT");
    }
    // Home runs this, so the threads are off the CPU; mask and stop the timer.
    lapic_write(LVT_TIMER, MASKED | TIMER_VECTOR as u32);
    lapic_write(TIMER_INITIAL, 0);
    BSP_PHASE.store(0, Ordering::Release);
    let frozen = (local.switches.load(Ordering::Acquire), counts());
    wait_us(10_000);
    if frozen != (local.switches.load(Ordering::Acquire), counts()) {
        serial::fail(b"SMP_BSP_PREEMPT_RAN_AFTER_STOP");
    }
    if local.corrupt.load(Ordering::Acquire) != 0 {
        serial::fail(b"SMP_BSP_PREEMPT_CONTEXT_CORRUPT");
    }
    serial::write(b"SMP:BSP_PREEMPT:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    true
}

/// Shared phase words of AP1's parts (BSP_SPIN, IDLE_PHASE) and their outcomes.
const STEP_RUN: u32 = 1;
const STEP_HELD: u32 = 2;
const STEP_ACKED: u32 = 3;
const STEP_TIMEOUT: u32 = 4;
static BSP_SPIN: AtomicU32 = AtomicU32::new(0);

/// AP1, IF=0 in its home loop: take KERNEL, wait until the BSP awaits it
/// (its lockdep records KERNEL before spinning), then shoot down the BSP
/// alone; the ack can only come through the BSP's window. The outcome is
/// published before KERNEL is released, so the BSP reads it under KERNEL.
fn bsp_spin_work() {
    let kernel = lock::KERNEL.lock_irqsave();
    BSP_SPIN.store(STEP_HELD, Ordering::Release);
    let deadline = u64::from(SHOOTDOWN_DEADLINE_MS);
    let bsp_waits = || BSP_LOCAL.lockdep.load(Ordering::Acquire) & (1 << level::KERNEL) != 0;
    // A kernel-half round aimed at the BSP alone (CPU index 0).
    let acked = within_tsc(deadline, bsp_waits) && round(0, paging::SHOOTDOWN_WINDOW, 1, 1).is_ok();
    BSP_SPIN.store(
        if acked { STEP_ACKED } else { STEP_TIMEOUT },
        Ordering::Release,
    );
    drop(kernel);
}

/// Spec 06 §4 "BSP als shootdowndoel": AP1 holds KERNEL and starts a round
/// aimed at the BSP alone while the BSP spins on KERNEL. The ack must arrive
/// within the deadline through the BSP's window, without quarantine. In
/// `test-smp-bsp-spin-irq-off` the BSP window stays shut and the round must
/// time out (terminal marker, no INIT of anything).
fn bsp_spin(online: usize, total: usize) -> bool {
    if online == 0 {
        serial::write(b"SMP:BSP_SPIN:SKIPPED:CPUS=1\n");
        return true;
    }
    let apic = apic_of(0);
    BSP_SPIN.store(STEP_RUN, Ordering::Release);
    if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32)
        || !within_ms(100, || BSP_SPIN.load(Ordering::Acquire) == STEP_HELD)
    {
        serial::write(b"SMP:BSP_SPIN:TIMEOUT:APIC=");
        decimal(u64::from(apic));
        serial::write(b"\n");
        quarantine_all(online, total);
        return false;
    }
    drop(lock::KERNEL.lock_irqsave());
    if BSP_SPIN.swap(0, Ordering::AcqRel) != STEP_ACKED {
        serial::write(b"SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=");
        decimal(u64::from(BSP_APIC_ID.load(Ordering::Relaxed)));
        serial::write(b"\n");
        if cfg!(feature = "test-smp-bsp-spin-irq-off") {
            serial::exit(0x10);
        }
        serial::fail(b"SMP_BSP_SPIN_TIMEOUT");
    }
    if cfg!(feature = "test-smp-bsp-spin-irq-off") {
        serial::fail(b"SMP_BSP_SPIN_WINDOW_OPEN");
    }
    serial::write(b"SMP:BSP_SPIN:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    true
}

/// IDLE_PHASE word: STEP_RUN (wake), IDLE_DEFER, then IDLE_DONE once AP1
/// took its part (CAS, so a later BSP request is never overwritten).
const IDLE_DEFER: u32 = 2;
const IDLE_DONE: u32 = 3;
static IDLE_PHASE: AtomicU32 = AtomicU32::new(0);
static IDLE_HALTING: AtomicBool = AtomicBool::new(false);
static IDLE_SENT_TSC: AtomicU64 = AtomicU64::new(0);
/// AP1's outcome of the deferred-resched part, and the BSP thread's progress.
static IDLE_AP: AtomicU32 = AtomicU32::new(0);
static IDLE_THREAD: AtomicU32 = AtomicU32::new(0);
static IDLE_LOCK: lock::IrqLock<u32> = lock::IrqLock::new(level::KERNEL, 0);

/// AP1, part 1: once the BSP idles, wake it with one 0xe8 and record when.
fn idle_wake_work() {
    if IDLE_PHASE
        .compare_exchange(STEP_RUN, IDLE_DONE, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    if within_tsc(100, || IDLE_HALTING.load(Ordering::Acquire)) {
        IDLE_SENT_TSC.store(rdtsc(), Ordering::Release);
        send_ipi(
            BSP_APIC_ID.load(Ordering::Relaxed),
            IPI_FIXED | RESCHED_VECTOR as u32,
        );
    }
}

/// AP1, part 2, IF=0 in its home loop: hold IDLE_LOCK until the BSP thread
/// waits for it, send the BSP one 0xe8 and see INV-5c defer it, then release.
/// Bounded throughout; the outcome is published before the release.
fn idle_defer_work() {
    if IDLE_PHASE
        .compare_exchange(IDLE_DEFER, IDLE_DONE, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    let guard = IDLE_LOCK.lock_irqsave();
    let deferred = BSP_LOCAL.deferred.load(Ordering::Acquire);
    IDLE_AP.store(STEP_HELD, Ordering::Release);
    let ok = within_tsc(100, || {
        BSP_LOCAL.lockdep.load(Ordering::Acquire) & (1 << level::KERNEL) != 0
    }) && send_ipi(
        BSP_APIC_ID.load(Ordering::Relaxed),
        IPI_FIXED | RESCHED_VECTOR as u32,
    ) && within_tsc(100, || {
        BSP_LOCAL.deferred.load(Ordering::Acquire) > deferred
    });
    IDLE_AP.store(
        if ok { STEP_ACKED } else { STEP_TIMEOUT },
        Ordering::Release,
    );
    drop(guard);
}

/// BSP thread of part 2 (IF=1, pseudo-user, BT-6): wait for IDLE_LOCK while
/// AP1 sends the deferred 0xe8; the release must then switch away (BT-4).
/// Never resumed after that switch, so the code below runs only if it failed.
extern "C" fn nos_smp_bsp_defer() -> ! {
    IDLE_THREAD.store(1, Ordering::Release);
    let guard = IDLE_LOCK.lock_irqsave();
    IDLE_THREAD.store(2, Ordering::Release);
    drop(guard);
    for _ in 0..1 << 20 {
        spin_loop();
    }
    if IDLE_AP.load(Ordering::Acquire) != STEP_ACKED {
        serial::write(b"SMP:IDLE:AP_TIMEOUT\n");
        serial::fail(b"SMP_IDLE_AP_TIMEOUT");
    }
    serial::write(b"SMP:IDLE:RESCHED_LOST\n");
    serial::fail(b"SMP_IDLE_RESCHED_LOST")
}

fn idle_timeout(apic: u32, online: usize, total: usize) -> bool {
    serial::write(b"SMP:IDLE:TIMEOUT:APIC=");
    decimal(u64::from(apic));
    serial::write(b"\n");
    quarantine_all(online, total);
    false
}

/// Spec 06 §4 "Idle-wek" and "Uitgestelde resched" (one marker). Part 1: the
/// BSP idles (timer masked) until AP1's 0xe8 wakes it; the latency from send
/// to handler is logged. Part 2: with the timer still masked, a BSP thread
/// waits for a lock AP1 holds; AP1's 0xe8 is deferred (INV-5c) and only the
/// release of the lock switches the thread away (BT-4). Needs one AP.
fn idle(online: usize, total: usize) -> bool {
    if online == 0 {
        serial::write(b"SMP:IDLE:SKIPPED:CPUS=1\n");
        return true;
    }
    let (apic, local) = (apic_of(0), &BSP_LOCAL);
    let resched = local.resched_count.load(Ordering::Acquire);
    BSP_WAKE_TSC.store(0, Ordering::Relaxed);
    IDLE_PHASE.store(STEP_RUN, Ordering::Release);
    if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32) {
        return idle_timeout(apic, online, total);
    }
    IDLE_HALTING.store(true, Ordering::Release);
    let woken = idle_until(200, || {
        local.resched_count.load(Ordering::Acquire) != resched
    });
    IDLE_HALTING.store(false, Ordering::Release);
    if !woken {
        return idle_timeout(apic, online, total);
    }
    let (sent, woke) = (
        IDLE_SENT_TSC.load(Ordering::Acquire),
        BSP_WAKE_TSC.load(Ordering::Acquire),
    );
    let latency = woke.saturating_sub(sent) * 1000 / TSC_PER_MS.load(Ordering::Acquire);
    serial::write(b"SMP:IDLE:WAKE_US=");
    decimal(latency);
    serial::write(b"\n");
    if sent == 0 || woke == 0 || latency >= 100_000 {
        serial::fail(b"SMP_IDLE_WAKE");
    }
    // Part 2. From here only a 0xe8 can switch the BSP: its timer is masked.
    let frame = frame_below(
        bsp_stack_base(0) + STACK_TOP,
        nos_smp_bsp_defer as usize as u64,
        0,
    );
    local.saved[1].store(frame, Ordering::Relaxed);
    local.current.store(0, Ordering::Relaxed);
    let (switches, deferred) = (
        local.switches.load(Ordering::Acquire),
        local.deferred.load(Ordering::Acquire),
    );
    BSP_PHASE.store(BSP_DEFER, Ordering::Release);
    IDLE_PHASE.store(IDLE_DEFER, Ordering::Release);
    if !send_ipi(apic, IPI_FIXED | IPI_VECTOR as u32)
        || !within_ms(100, || IDLE_AP.load(Ordering::Acquire) == STEP_HELD)
    {
        BSP_PHASE.store(0, Ordering::Release);
        return idle_timeout(apic, online, total);
    }
    if !send_ipi(
        BSP_APIC_ID.load(Ordering::Relaxed),
        IPI_FIXED | RESCHED_VECTOR as u32,
    ) {
        serial::fail(b"SMP_IDLE_SELF_IPI");
    }
    // The pending self 0xe8 switches to the thread in the first idle window;
    // the thread's release switches back here. A stuck thread never returns,
    // but its own waits are bounded and end the VM.
    let returned = idle_until(500, || IDLE_THREAD.load(Ordering::Acquire) == 2);
    BSP_PHASE.store(0, Ordering::Release);
    if !returned
        || local.current.load(Ordering::Acquire) != 0
        || IDLE_THREAD.load(Ordering::Acquire) != 2
        || IDLE_AP.load(Ordering::Acquire) != STEP_ACKED
        || local.deferred.load(Ordering::Acquire) <= deferred
        || local.switches.load(Ordering::Acquire) != switches + 2
        || local.lockdep.load(Ordering::Acquire) != 0
        || local.need_resched.load(Ordering::Acquire)
    {
        serial::fail(b"SMP_IDLE_DEFER");
    }
    IDLE_PHASE.store(0, Ordering::Release);
    serial::write(b"SMP:IDLE:PASS:CPUS=");
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
/// This CPU's idle thread (BW-4): the home loop (`sti; hlt; cli`, spec §6.3
/// step 9), saved in `CpuLocal::saved[0]`, never queued, run under the kernel CR3.
const HOME: u32 = u32::MAX;
const ROOT_MASK: u64 = !0xfff;
/// 0, then PREEMPT_RUN / PREEMPT_STOP for a scheduler phase.
static SCHED: AtomicU32 = AtomicU32::new(0);
/// First thread in space 1: threads below it run in space 0. The 5d-1 phase
/// sets the online AP count (thread `tid` on AP `tid % aps`); the F16 phase 2.
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
/// BW-2: set while the thread is in some run queue (`RunQueue::push`/`pop`).
static THREAD_QUEUED: [AtomicBool; SCHED_THREADS] =
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
    resched = const core::mem::offset_of!(CpuLocal, resched_count),
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

/// Terminal report (BW-2, audit F17): a thread already in a run queue was
/// pushed again. Caught at the push itself, so independent of timing.
fn double_enqueue() -> ! {
    serial::write(b"SMP:SCHED:DOUBLE_ENQUEUE\n");
    if cfg!(feature = "test-smp-double-run") {
        serial::exit(0x10);
    }
    serial::fail(b"SMP_SCHED_DOUBLE_ENQUEUE")
}

fn enqueue(queue: &mut RunQueue<SCHED_THREADS>, tid: u32) {
    if queue.push(tid, &THREAD_QUEUED).is_err() {
        double_enqueue();
    }
}

/// Spec §6.3 step 4 for a thread leaving this CPU: back into the queue when
/// READY (a yield, or READY_PENDING: a wake met the block, BW-1) or dying (a
/// later switch reaps it, BW-3); out of every queue when BLOCKED.
fn leave(queue: &mut RunQueue<SCHED_THREADS>, tid: u32, cpu: usize) {
    let state = &THREAD_STATE[tid as usize];
    let requeue = match sched::yield_cpu(state, cpu) {
        Ok(()) | Err(sched::DEAD_PENDING) => true,
        Err(sched::BLOCKING | sched::READY_PENDING) => match sched::block_commit(state) {
            Ok(requeue) => requeue,
            Err(sched::DEAD_PENDING) => true,
            Err(_) => double_run(),
        },
        Err(_) => double_run(),
    };
    if requeue {
        enqueue(queue, tid);
    }
}

/// Round robin over the queue as it is now. A dying thread is reaped once no
/// CPU is on its stack (BW-3: DEAD only after the stub cleared `on_cpu`), else
/// requeued for a later switch. HOME (idle) when nothing can run.
fn pick(queue: &mut RunQueue<SCHED_THREADS>, current: u32, cpu: usize) -> u32 {
    for _ in 0..queue.len() {
        let Some(tid) = queue.pop(&THREAD_QUEUED) else {
            break;
        };
        let (state, on_cpu) = (&THREAD_STATE[tid as usize], &THREAD_ON_CPU[tid as usize]);
        match sched::dispatch(state, cpu) {
            // Audit F1: the CPU that last ran `tid` may still be on its stack.
            // Do not wait for it here; requeue and run home until the next tick.
            Ok(()) if tid != current && !sched::claim_stack(on_cpu) => {
                leave(queue, tid, cpu);
                return HOME;
            }
            Ok(()) => return tid,
            Err(sched::DEAD_PENDING) => {
                if sched::kill_commit(state, on_cpu).is_err() {
                    enqueue(queue, tid);
                }
            }
            Err(_) => double_run(),
        }
    }
    HOME
}

/// Spec §6.3 step 5 through `sched::switch_space` (host-modelled order): the
/// new space's bit before the CR3 load, the old one cleared after it.
fn switch_space(cpu: usize, from: Option<usize>, to: Option<usize>) {
    let root = to
        .map_or(&KERNEL_CR3, |to| &SPACE_ROOT[to])
        .load(Ordering::Relaxed);
    sched::switch_space(&ACTIVE, cpu, from, to, || {
        // SAFETY: the kernel CR3, or a fixture space (`build_spaces`) whose PDPT
        // entry 0 is the kernel's one PD: code, stacks, GS anchor and the LAPIC
        // window stay mapped. PCID is off, so the load flushes this CPU's TLB.
        unsafe { asm!("mov cr3, {}", in(reg) root, options(nostack)) };
    });
}

/// Context switch on an AP (spec §6.3), IF=0 and lockdep empty on entry. The
/// RUNQ lock covers the state hand-over and the CR3 switch; STOP returns home.
/// BLOCKING/READY_PENDING/DEAD_PENDING are settled in `leave` and `pick`.
fn sched_switch(local: &CpuLocal, rsp: u64, busy: bool) -> Resume {
    let slot = local.slot.load(Ordering::Relaxed) as usize;
    let cpu = slot + 1;
    if cfg!(feature = "test-smp-proc-shootdown-deaf")
        && PROC_DEAF_FROZEN.load(Ordering::Acquire) & (1 << cpu) != 0
    {
        // The setup round closed this test CPU's scheduler admission while
        // lock-free. A pending 0xd0/0xe8 resumes the same stack/root without
        // touching RUNQ; the real process shootdown still receives no ack.
        assert!(!busy);
        return Resume { rsp, left: 0 };
    }
    let current = local.sched_cur.load(Ordering::Relaxed);
    let stop = SCHED.load(Ordering::Acquire) == PREEMPT_STOP;
    let mut queue = RUNQ[slot].lock_irqsave();
    if current == HOME {
        local.saved[0].store(rsp, Ordering::Relaxed);
    } else {
        THREAD_RSP[current as usize].store(rsp, Ordering::Relaxed);
        leave(&mut queue, current, cpu);
    }
    let next = if stop {
        HOME
    } else {
        pick(&mut queue, current, cpu)
    };
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
        SCHED_APS.store(online as u32, Ordering::Relaxed);
        for tid in 0..2 * online {
            // Stack: slot tid/2's thread stacks, frozen since the step-4 STOP.
            let entry = nos_smp_sched_thread as usize as u64;
            let frame = thread_frame(tid / 2, THREADS[tid % 2], entry, tid as u64);
            THREAD_RSP[tid].store(frame, Ordering::Relaxed);
        }
        for (slot, queue) in RUNQ[..online].iter().enumerate() {
            THREAD_STATE[slot].store(sched::READY, Ordering::Release);
            enqueue(&mut queue.lock_irqsave(), slot as u32);
        }
        if cfg!(feature = "test-smp-double-run") && online >= 2 {
            // Thread 0 queued on a second AP as well: the push itself must
            // refuse (BW-2), in every run, before any AP can dispatch it.
            enqueue(&mut RUNQ[1].lock_irqsave(), 0);
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
                if sched::wake(&THREAD_STATE[tid]) != Ok(true) {
                    serial::fail(b"SMP_SCHED_WAKE");
                }
                enqueue(&mut queue, tid as u32);
            }
            let (here, woken) = (&PROGRESS[slot], &PROGRESS[tid]);
            if !send_ipi(apic, IPI_FIXED | RESCHED_VECTOR as u32)
                || !within_ms(500, || {
                    local.resched_count.load(Ordering::Acquire) != 0
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
        if !stop_scheduler(online, total) {
            return false;
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

/// End of a scheduler phase: every AP returns home under the kernel CR3 with
/// no thread running afterwards, no `ACTIVE` bit and no lock left.
fn stop_scheduler(online: usize, total: usize) -> bool {
    let locals = &CPU_LOCAL[..online];
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
    true
}

// Audit F16/F17 (ADR-078): blocking, waking and killing across CPUs. Kernel
// threads A (tid 0) and B (tid 1) hand a turn to each other under KERNEL
// 10 000 times, each blocking until the other wakes it (BW-1); then the BSP
// kills thread K (tid 2, alone in space 1) while it runs (BW-3). A, B and K run
// on AP1, AP2 and AP3; with fewer APs they share (tid % APs).
const WAKE_ROUNDS: u64 = 10_000;
const WAKE_DEADLINE_MS: u32 = 20_000;
const KILLED: usize = 2;
/// Online APs in the F16 phase (thread placement).
static BW_APS: AtomicU32 = AtomicU32::new(0);
/// The "mailbox": whose turn it is (tid 0 or 1). Only touched under KERNEL.
static TURN: AtomicU32 = AtomicU32::new(0);

fn bw_slot(tid: usize) -> usize {
    tid % BW_APS.load(Ordering::Relaxed) as usize
}

fn apic_of(slot: usize) -> u32 {
    NOS_SMP_AP_CONTEXTS[slot].apic_id.load(Ordering::Relaxed)
}

/// Spec §6.4 from CPU index `from`, with KERNEL held (the guard proves it):
/// RUNQ[slot] -> BLOCKED->READY and enqueue -> unlock -> 0xe8 if that is
/// another CPU. A BLOCKING target only becomes READY_PENDING: its own switch
/// requeues it. False if the IPI could not be sent.
fn wake_on(tid: usize, slot: usize, from: usize, _kernel: &lock::IrqGuard<'_, ()>) -> bool {
    let enqueued = {
        let mut queue = RUNQ[slot].lock_irqsave();
        let enqueued = sched::wake(&THREAD_STATE[tid]) == Ok(true);
        if enqueued {
            enqueue(&mut queue, tid as u32);
        }
        enqueued
    };
    !enqueued || slot + 1 == from || send_ipi(apic_of(slot), IPI_FIXED | RESCHED_VECTOR as u32)
}

/// A switch on this CPU now: a self 0xe8, sent with IF=0 so no other thread
/// on this CPU can interleave its own ICR write, and taken right after `sti`.
fn resched_self(apic: u32) {
    // SAFETY: CPL0 kernel thread; IF is restored right after the ICR write.
    unsafe { asm!("cli", options(nomem, nostack)) };
    let sent = send_ipi(apic, IPI_FIXED | RESCHED_VECTOR as u32);
    // SAFETY: as above; the pending 0xe8 is delivered after `sti`'s shadow.
    unsafe { asm!("sti", options(nomem, nostack)) };
    if !sent {
        SCHED_FAULT.store(1, Ordering::Release);
    }
}

/// A and B: take a turn when it is ours, else block (BW-1 phase 1) under the
/// same KERNEL that the waker holds, drop KERNEL (no lock over the switch,
/// BW-5) and switch. K: spin until killed; `test-smp-resched-deaf` first
/// raises TPR to class 0xe so this AP holds back 0xd0 and 0xe8 (deaf to the kill).
extern "C" fn nos_smp_bw_thread(tid: u64) -> ! {
    let tid = tid as usize;
    let Some(local) = this_cpu() else { park() };
    let slot = local.slot.load(Ordering::Relaxed) as usize;
    if tid == KILLED {
        if cfg!(feature = "test-smp-resched-deaf") {
            lapic_write(0x80, 0xe0);
        }
        loop {
            PROGRESS[KILLED].fetch_add(1, Ordering::Release);
            for _ in 0..500 {
                spin_loop();
            }
        }
    }
    let (cpu, partner) = (slot + 1, 1 - tid);
    loop {
        let kernel = lock::KERNEL.lock_irqsave();
        if PROGRESS[tid].load(Ordering::Relaxed) < WAKE_ROUNDS
            && TURN.load(Ordering::Relaxed) == tid as u32
        {
            PROGRESS[tid].fetch_add(1, Ordering::Release);
            TURN.store(partner as u32, Ordering::Relaxed);
            if !wake_on(partner, bw_slot(partner), cpu, &kernel) {
                SCHED_FAULT.store(1, Ordering::Release);
            }
            continue;
        }
        if sched::block_prepare(&THREAD_STATE[tid], cpu).is_err() {
            SCHED_FAULT.store(1, Ordering::Release);
        }
        drop(kernel);
        resched_self(apic_of(slot));
    }
}

fn kill_timeout(apic: u32, online: usize, total: usize) -> bool {
    // INV-5; a deaf CPU that holds a lock stops the system (IQ-3).
    serial::write(b"SMP:KILL:TIMEOUT:APIC=");
    decimal(u64::from(apic));
    serial::write(b"\n");
    quarantine_all(online, total);
    false
}

/// BSP: the F16/F17 phase on a clean scheduler (every AP home since the 5d-1
/// STOP). Ping-pong: both counters reach WAKE_ROUNDS and both threads end
/// BLOCKED. Kill: DEAD within the deadline, then off its stack, in no queue,
/// frozen, and its space active on no CPU.
fn block_wake_kill(online: usize, total: usize) -> bool {
    if online == 0 {
        serial::write(b"SMP:WAKE:SKIPPED:CPUS=1\nSMP:KILL:SKIPPED:CPUS=1\n");
        return true;
    }
    for queue in &RUNQ[..online] {
        let mut queue = queue.lock_irqsave();
        while queue.pop(&THREAD_QUEUED).is_some() {}
    }
    SCHED_APS.store(2, Ordering::Relaxed);
    BW_APS.store(online as u32, Ordering::Relaxed);
    TURN.store(0, Ordering::Relaxed);
    for tid in 0..=KILLED {
        PROGRESS[tid].store(0, Ordering::Relaxed);
        THREAD_STATE[tid].store(sched::BLOCKED, Ordering::Relaxed);
        // Stack: slot tid/2's thread stacks, frozen since the 5d-1 STOP.
        let entry = nos_smp_bw_thread as usize as u64;
        let frame = thread_frame(tid / 2, THREADS[tid % 2], entry, tid as u64);
        THREAD_RSP[tid].store(frame, Ordering::Relaxed);
    }
    for local in &CPU_LOCAL[..online] {
        local.sched_state.store(0, Ordering::Release);
    }
    for tid in 0..KILLED {
        let mut queue = RUNQ[bw_slot(tid)].lock_irqsave();
        THREAD_STATE[tid].store(sched::READY, Ordering::Release);
        enqueue(&mut queue, tid as u32);
    }
    SCHED.store(PREEMPT_RUN, Ordering::Release);
    for slot in 0..online {
        if !send_ipi(apic_of(slot), IPI_FIXED | IPI_VECTOR as u32) {
            return sched_timeout(apic_of(slot), online, total);
        }
    }
    let rounds = |tid: usize| PROGRESS[tid].load(Ordering::Acquire);
    if !within_ms(WAKE_DEADLINE_MS, || {
        rounds(0) == WAKE_ROUNDS && rounds(1) == WAKE_ROUNDS
    }) || !within_ms(100, || {
        (0..KILLED).all(|tid| THREAD_STATE[tid].load(Ordering::Acquire) == sched::BLOCKED)
    }) || SCHED_FAULT.load(Ordering::Acquire) != 0
    {
        serial::write(b"SMP:WAKE:TIMEOUT:A=");
        decimal(rounds(0));
        serial::write(b":B=");
        decimal(rounds(1));
        serial::write(b"\n");
        quarantine_all(online, total);
        return false;
    }
    serial::write(b"SMP:WAKE:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    let (slot, state) = (bw_slot(KILLED), &THREAD_STATE[KILLED]);
    let apic = apic_of(slot);
    {
        let mut queue = RUNQ[slot].lock_irqsave();
        state.store(sched::READY, Ordering::Release);
        enqueue(&mut queue, KILLED as u32);
    }
    if !send_ipi(apic, IPI_FIXED | RESCHED_VECTOR as u32) || !within_ms(500, || rounds(KILLED) != 0)
    {
        return sched_timeout(apic, online, total);
    }
    {
        let mut queue = RUNQ[slot].lock_irqsave();
        match sched::kill_request(state) {
            Ok(sched::BLOCKED) => enqueue(&mut queue, KILLED as u32),
            Ok(_) => {}
            Err(_) => serial::fail(b"SMP_KILL_TWICE"),
        }
    }
    if !send_ipi(apic, IPI_FIXED | RESCHED_VECTOR as u32)
        || !within_ms(SHOOTDOWN_DEADLINE_MS, || {
            state.load(Ordering::Acquire) == sched::DEAD
        })
    {
        return kill_timeout(apic, online, total);
    }
    let last = rounds(KILLED);
    wait_us(10_000);
    if THREAD_ON_CPU[KILLED].load(Ordering::Acquire)
        || THREAD_QUEUED[KILLED].load(Ordering::Acquire)
        || ACTIVE[1].load(Ordering::SeqCst) != 0
        || rounds(KILLED) != last
    {
        serial::fail(b"SMP_KILL_NOT_DEAD");
    }
    serial::write(b"SMP:KILL:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    stop_scheduler(online, total)
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

// Audit F19 (IQ-1): AP1 and AP2 send each other ICR_ROUNDS 0xe8 IPIs from
// thread context while every 0xd0/0xe8 entry on them sends an IPI from the
// handler. A destination replaced between the two ICR writes shows up as a
// count mismatch. `test-smp-icr-race` drops the lock and lets one 0xd0 in.
const ICR_ROUNDS: u32 = 10_000;
const ICR_PASSED: u32 = 1;
const ICR_TIMEOUT: u32 = 2;
static ICR_RUN: AtomicBool = AtomicBool::new(false);
static ICR_FAULT: AtomicBool = AtomicBool::new(false);
static ICR_RACED: AtomicBool = AtomicBool::new(false);

/// Race control only: once, on AP1 in thread context, wait between the two
/// ICR writes until one 0xd0 tick was taken; its handler's own IPI then
/// overwrites the destination. (A one-shot count of 1 stalled QEMU/TCG.)
fn icr_race_window() {
    let Some(local) = this_cpu() else {
        return;
    };
    if ICR_RUN.load(Ordering::Acquire)
        && local.slot.load(Ordering::Relaxed) == 0
        && irq_depth() == 0
        && !ICR_RACED.swap(true, Ordering::Relaxed)
    {
        let before = local.tick_count.load(Ordering::Relaxed);
        for _ in 0..1 << 24 {
            // SAFETY: CPL0 AP thread context of the ICR phase, which holds no
            // lock; every unmasked source has an IDT entry. Test-only window.
            unsafe { asm!("sti", "nop", "pause", "cli") };
            if local.tick_count.load(Ordering::Relaxed) != before {
                break;
            }
        }
    }
}

/// AP side, IF=0 in the home loop. Each send waits for the partner's count to
/// reach it, so at most one 0xe8 is in flight per direction (an IRR bit never
/// merges two) and the counts must match exactly. IQ-4: a missing ack is only
/// reported (`icr_done`); the BSP decides.
fn icr_work(local: &CpuLocal, slot: usize) {
    let (partner, apic) = (&CPU_LOCAL[slot ^ 1], apic_of(slot ^ 1));
    start_timer();
    let mut done = ICR_PASSED;
    for round in 1..=ICR_ROUNDS {
        if !send_ipi(apic, IPI_FIXED | RESCHED_VECTOR as u32) {
            done = ICR_TIMEOUT;
            break;
        }
        local.icr_sent.store(round, Ordering::Release);
        let acked = (0..1 << 16).any(|_| {
            // SAFETY: as in `icr_race_window`; IF is 0 again after `cli`.
            unsafe { asm!("sti", "nop", "pause", "cli") };
            partner.resched_count.load(Ordering::Acquire) >= round
        });
        if !acked {
            done = ICR_TIMEOUT;
            break;
        }
    }
    lapic_write(LVT_TIMER, MASKED | TIMER_VECTOR as u32);
    local.icr_done.store(done, Ordering::Release);
}

/// Terminal verdict of the ICR phase; only the race control counts it a pass.
fn icr_verdict(marker: &[u8]) -> ! {
    serial::write(marker);
    if cfg!(feature = "test-smp-icr-race") {
        serial::exit(0x10);
    }
    serial::fail(b"SMP_ICR")
}

/// BSP: received 0xe8 on each AP = sent by its partner, ICR_ROUNDS each way.
fn icr(online: usize, total: usize) -> bool {
    if online < 2 {
        serial::write(b"SMP:ICR:SKIPPED:CPUS=");
        decimal(online as u64 + 1);
        serial::write(b"\n");
        return true;
    }
    let pair = &CPU_LOCAL[..2];
    for local in pair {
        // Both timers are masked, so no entry interleaves; keep the 0xe8
        // detection of `smp_preempt` (BT-4) in step with the reset count.
        local.resched_count.store(0, Ordering::Relaxed);
        local.resched_seen.store(0, Ordering::Relaxed);
    }
    ICR_RUN.store(true, Ordering::Release);
    if !(0..2).all(|slot| send_ipi(apic_of(slot), IPI_FIXED | IPI_VECTOR as u32))
        || !within_ms(WAKE_DEADLINE_MS, || {
            pair.iter()
                .all(|local| local.icr_done.load(Ordering::Acquire) != 0)
        })
    {
        serial::write(b"SMP:ICR:TIMEOUT\n");
        quarantine_all(online, total);
        return false;
    }
    // Both timers are masked; let a last handler IPI land before counting.
    wait_us(10_000);
    ICR_RUN.store(false, Ordering::Release);
    let got = |slot: usize| pair[slot].resched_count.load(Ordering::Acquire);
    let sent = |slot: usize| pair[slot].icr_sent.load(Ordering::Acquire);
    if got(0) > sent(1) || got(1) > sent(0) {
        icr_verdict(b"SMP:ICR:MISROUTED\n");
    }
    if got(0) != sent(1)
        || got(1) != sent(0)
        || pair.iter().any(|local| {
            local.icr_sent.load(Ordering::Acquire) != ICR_ROUNDS
                || local.icr_done.load(Ordering::Acquire) != ICR_PASSED
        })
        || ICR_FAULT.load(Ordering::Acquire)
    {
        icr_verdict(b"SMP:ICR:LOST\n");
    }
    if cfg!(feature = "test-smp-icr-race") {
        serial::fail(b"SMP_ICR_RACE_MISSED");
    }
    serial::write(b"SMP:ICR:PASS:ROUNDS=");
    decimal(u64::from(ICR_ROUNDS));
    serial::write(b":CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    true
}

// Step 5e (spec 07, PS-1..PS-5): two real spaces with a different frame at the
// same VA. Isolation: thread 0 (space A) on AP1 and thread 1 (space B) on AP2
// (one AP runs both) write and read their own pattern. Unmap race: thread 0
// reads A's page while the BSP unmaps it, shoots it down on ACTIVE[A] only,
// frees the frame and reuses it with NEW_PATTERN; the reader must take the
// controlled #PF (`race_fixup`) and never see the reused contents.
const SPACE_VA: u64 = 0x4000_0000;
const SPACE_PATTERN: [u64; 2] = [0x1111_1111_1111_1111, 0x2222_2222_2222_2222];
const ISO_ROUNDS: u64 = 10_000;
const ISO_DEADLINE_MS: u32 = 20_000;
/// `nos_smp_race_read`'s result once the page is not present (no byte value).
const RACE_FAULT: u32 = 0x100;
/// Spec §4.2 level 20, keyed by the fixture pid (space + 1).
static ASPACE: [lock::IrqLock<()>; 2] = [
    lock::IrqLock::keyed(level::ASPACE, 1, ()),
    lock::IrqLock::keyed(level::ASPACE, 2, ()),
];
static ISO_FAULT: AtomicBool = AtomicBool::new(false);
static RACE_RUN: AtomicBool = AtomicBool::new(false);
static RACE_READS: AtomicU64 = AtomicU64::new(0);
static RACE_FAULTED: AtomicBool = AtomicBool::new(false);
static RACE_STALE: AtomicBool = AtomicBool::new(false);
/// Test-only scheduler admission closure, published by each AP before its
/// valid setup-round ack. It is never used by a normal recovery path.
static PROC_DEAF_FROZEN: AtomicU64 = AtomicU64::new(0);

global_asm!(
    ".pushsection .text.nos_smp_race,\"ax\"",
    // The read is the leaf function's first instruction: on its #PF, [rsp]
    // is the return address `race_fixup` returns to.
    ".globl nos_smp_race_read",
    "nos_smp_race_read:",
    "    movzx eax, byte ptr [rdi]",
    "    ret",
    // Resume a normalized CPL0 exception frame (RDI: 15 GPRs, vector, error,
    // CPU frame) through the spurious stub's reviewed `iretq`. CPL0 only, so
    // GS is the kernel's on entry and exit (GS-2: no swapgs).
    ".globl nos_smp_fault_resume",
    "nos_smp_fault_resume:",
    "    mov rsp, rdi",
    "    pop r15",
    "    pop r14",
    "    pop r13",
    "    pop r12",
    "    pop r11",
    "    pop r10",
    "    pop r9",
    "    pop r8",
    "    pop rdi",
    "    pop rsi",
    "    pop rbp",
    "    pop rbx",
    "    pop rdx",
    "    pop rcx",
    "    pop rax",
    "    add rsp, 16",
    "    jmp nos_smp_spurious_entry",
    ".popsection",
);
extern "C" {
    fn nos_smp_race_read(va: u64) -> u32;
    fn nos_smp_fault_resume(frame: *mut u64) -> !;
}

/// The controlled #PF of the unmap race: `nos_smp_race_read` returns
/// RACE_FAULT to its caller instead of faulting again.
fn race_fixup(frame: *const u64) -> ! {
    let frame = frame.cast_mut();
    // SAFETY: `frame` is the live normalized frame of a CPL0 #PF on this
    // kernel thread's stack (rax at 14, RIP 17, RSP 20). The fault RIP is the
    // first instruction of the leaf `nos_smp_race_read`, so [RSP] is its
    // return address; resuming there with RSP + 8 is that function's `ret`.
    unsafe {
        let rsp = frame.add(20).read();
        frame.add(14).write(u64::from(RACE_FAULT));
        frame.add(17).write((rsp as *const u64).read());
        frame.add(20).write(rsp + 8);
        nos_smp_fault_resume(frame)
    }
}

/// Two real spaces (spec 07 §3): own PML4, PDPT, PD and PT, the kernel half
/// shared through PDPT entry 0 (PS-4), and a different User frame at SPACE_VA.
fn build_spaces(
    frames: &mut Frames<FRAME_COUNT>,
    paging: &paging::Paging,
) -> &'static mut [paging::Paging; 2] {
    let slots = ptr::addr_of_mut!(SPACES).cast::<paging::Paging>();
    for space in 0..2 {
        let user = build_space(frames, paging, space);
        // SAFETY: BSP only, IF=0, called once per boot (`start`): the static
        // is written exactly once per element and borrowed only below.
        unsafe { slots.add(space).write(user) };
    }
    // SAFETY: both elements were initialized above; this is the only borrow.
    unsafe { (*ptr::addr_of_mut!(SPACES)).assume_init_mut() }
}

/// Step 5d-2: the two fixture spaces live here, not on MAIN. As a by-value
/// array they cost ~13 KiB of the 16-KiB MAIN stack under `start` (8.5 KiB
/// for `build_spaces` alone), which the larger M3 build overflowed (#DF).
static mut SPACES: MaybeUninit<[paging::Paging; 2]> = MaybeUninit::uninit();

#[inline(never)]
fn build_space(
    frames: &mut Frames<FRAME_COUNT>,
    paging: &paging::Paging,
    space: usize,
) -> paging::Paging {
    let mut user = paging::Paging::new_user(frames, paging, space as u16 + 1);
    let byte = SPACE_PATTERN[space] as u8;
    let pa = frames
        .allocate(Owner::User(space as u64 + 1, Kind::Data), |pa| {
            // SAFETY: the fresh, unpublished frame through this CPU's
            // scratch slot; exactly one page is written.
            paging::physical(pa, |p| unsafe { ptr::write_bytes(p, byte, PAGE as usize) })
        })
        .unwrap_or_else(|_| serial::fail(b"SMP_SPACE_FRAMES"));
    user.map_user(frames, SPACE_VA, pa, true, false);
    SPACE_ROOT[space].store(user.root, Ordering::Relaxed);
    user
}

extern "C" fn nos_smp_proc_thread(tid: u64) -> ! {
    let tid = tid as usize;
    let own = SPACE_PATTERN[usize::from(tid != 0)];
    loop {
        if tid == 0 && RACE_RUN.load(Ordering::Acquire) {
            race_reader();
        }
        if PROGRESS[tid].load(Ordering::Relaxed) < ISO_ROUNDS {
            // SAFETY: SPACE_VA is the RW data page of this thread's own space,
            // whose root the scheduler loads for it; volatile u64 accesses.
            let seen = unsafe {
                (SPACE_VA as *mut u64).write_volatile(own);
                for _ in 0..50 {
                    spin_loop();
                }
                (SPACE_VA as *const u64).read_volatile()
            };
            if seen != own {
                ISO_FAULT.store(true, Ordering::Release);
            }
            PROGRESS[tid].fetch_add(1, Ordering::Release);
        }
        spin_loop();
    }
}

fn race_reader() -> ! {
    loop {
        // SAFETY: one byte of space A's page, or RACE_FAULT via `race_fixup`
        // once this CPU's translation is gone.
        match unsafe { nos_smp_race_read(SPACE_VA) } {
            RACE_FAULT => {
                RACE_FAULTED.store(true, Ordering::Release);
                loop {
                    spin_loop();
                }
            }
            byte if byte == u32::from(NEW_PATTERN) => RACE_STALE.store(true, Ordering::Release),
            _ => {
                RACE_READS.fetch_add(1, Ordering::Release);
            }
        }
    }
}

/// PS-5 rule: no frame that a live space's PTE names is free.
fn check_live(frames: &Frames<FRAME_COUNT>, live: &[&paging::Paging]) {
    for space in live {
        let leaf = space.leaf(SPACE_VA);
        if leaf & 1 != 0 && frames.owner(leaf & 0x000f_ffff_ffff_f000) == Ok(Owner::Free) {
            serial::fail(b"SMP_LIVE_PTE_FREE_FRAME");
        }
    }
}

/// PS-5 for one fixture space, under KERNEL and its ASPACE: no CPU active,
/// its page unmapped with a round, then a round over the whole user GiB (a
/// CR3 reload on any target, which also drops cached paging-structure
/// entries) before the tables are freed. The protocol runs on an empty set too.
fn exit_space(
    frames: &mut Frames<FRAME_COUNT>,
    user: &mut paging::Paging,
    space: usize,
) -> Result<(), Quarantined> {
    let _kernel = lock::KERNEL.lock_irqsave();
    let _aspace = ASPACE[space].lock_irqsave();
    if ACTIVE[space].load(Ordering::SeqCst) != 0 {
        serial::fail(b"SMP_EXIT_ACTIVE");
    }
    if user.leaf(SPACE_VA) & 1 != 0 {
        let pa = user.unmap_shot(frames, SPACE_VA, space)?;
        let owner = Owner::User(space as u64 + 1, Kind::Data);
        if frames.free(pa, owner, paging::zero_dynamic).is_err() {
            serial::fail(b"SMP_EXIT_FRAME");
        }
    }
    shootdown_process(space, SPACE_VA, 1 << 18)?;
    user.destroy(frames);
    if ACTIVE[space].load(Ordering::SeqCst) != 0 {
        serial::fail(b"SMP_EXIT_ACTIVE");
    }
    Ok(())
}

/// Exit A, check B still names no free frame, exit B, then the kernel audit.
fn exit_spaces(
    online: usize,
    total: usize,
    frames: &mut Frames<FRAME_COUNT>,
    paging: &paging::Paging,
    [a, b]: &mut [paging::Paging; 2],
) -> bool {
    let exited = exit_space(frames, a, 0).and_then(|()| {
        check_live(frames, &[b]);
        exit_space(frames, b, 1)
    });
    if let Err(failed) = exited {
        return frame_quarantined(failed, online, total);
    }
    paging.audit(frames);
    true
}

/// BSP: the step 5e isolation and unmap-race fixture (spec 07 §4).
fn proc_shootdown(
    online: usize,
    total: usize,
    frames: &mut Frames<FRAME_COUNT>,
    paging: &paging::Paging,
    spaces: &mut [paging::Paging; 2],
) -> bool {
    if online == 0 {
        serial::write(b"SMP:ISOLATION:SKIPPED:CPUS=1\nSMP:UNMAP_RACE:SKIPPED:CPUS=1\n");
        return exit_spaces(online, total, frames, paging, spaces);
    }
    let [a, b] = spaces;
    for queue in &RUNQ[..online] {
        let mut queue = queue.lock_irqsave();
        while queue.pop(&THREAD_QUEUED).is_some() {}
    }
    SCHED_APS.store(1, Ordering::Relaxed);
    for tid in 0..2 {
        PROGRESS[tid].store(0, Ordering::Relaxed);
        // Stack: slot 0's thread stacks, frozen since the last STOP.
        let entry = nos_smp_proc_thread as usize as u64;
        THREAD_RSP[tid].store(
            thread_frame(0, THREADS[tid], entry, tid as u64),
            Ordering::Relaxed,
        );
    }
    for local in &CPU_LOCAL[..online] {
        local.sched_state.store(0, Ordering::Release);
    }
    for tid in 0..2 {
        let mut queue = RUNQ[tid % online].lock_irqsave();
        THREAD_STATE[tid].store(sched::READY, Ordering::Release);
        enqueue(&mut queue, tid as u32);
    }
    SCHED.store(PREEMPT_RUN, Ordering::Release);
    for slot in 0..online {
        if !send_ipi(apic_of(slot), IPI_FIXED | IPI_VECTOR as u32) {
            return sched_timeout(apic_of(slot), online, total);
        }
    }
    let rounds = |tid: usize| PROGRESS[tid].load(Ordering::Acquire);
    if !within_ms(ISO_DEADLINE_MS, || {
        rounds(0) >= ISO_ROUNDS && rounds(1) >= ISO_ROUNDS
    }) {
        serial::write(b"SMP:ISOLATION:TIMEOUT\n");
        quarantine_all(online, total);
        return false;
    }
    if ISO_FAULT.load(Ordering::Acquire) {
        serial::fail(b"SMP_ISOLATION");
    }
    serial::write(b"SMP:ISOLATION:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    RACE_RUN.store(true, Ordering::Release);
    if !within_ms(500, || RACE_READS.load(Ordering::Acquire) != 0) {
        return sched_timeout(apic_of(0), online, total);
    }
    if cfg!(feature = "test-smp-proc-shootdown-deaf") {
        // Freeze only scheduler admission, retaining every AP's real current
        // context/CR3/ACTIVE and the running reader. This is a separate valid
        // kernel round, not an ack of the deliberately deaf process round.
        if let Err(failed) = shootdown_kernel(SPACE_VA, 1) {
            return frame_quarantined(failed, online, total);
        }
        let targets = ONLINE_CPUS.load(Ordering::Acquire) & !1;
        assert_eq!(PROC_DEAF_FROZEN.load(Ordering::Acquire), targets);
        assert!((1..=online).all(|cpu| held_by(cpu).is_none()));
        let reads = RACE_READS.load(Ordering::Acquire);
        if !within_ms(500, || {
            RACE_READS.load(Ordering::Acquire) > reads && ACTIVE[0].load(Ordering::SeqCst) == 1 << 1
        }) {
            return sched_timeout(apic_of(0), online, total);
        }
        serial::write(b"SMP:PROC_DEAF:LOCK_FREE:CPUS=");
        decimal(online as u64 + 1);
        serial::write(b"\n");
    }
    let before: [u32; MAX_APS] =
        core::array::from_fn(|slot| CPU_LOCAL[slot].shootdowns.load(Ordering::Acquire));
    let unmapped = {
        let _kernel = lock::KERNEL.lock_irqsave();
        let _aspace = ASPACE[0].lock_irqsave();
        a.unmap_shot(frames, SPACE_VA, 0)
    };
    let old = match unmapped {
        Ok(pa) => pa,
        Err(failed) => return frame_quarantined(failed, online, total),
    };
    // PS-3: every target acknowledged, so the frame may change owner. First
    // fit hands back the same frame: nothing below it was freed since.
    let reused = frames
        .free(old, Owner::User(1, Kind::Data), paging::zero_dynamic)
        .is_ok()
        && frames.allocate(Owner::Kernel, |pa| {
            // SAFETY: the unpublished frame through this CPU's scratch slot.
            paging::physical(pa, |p| unsafe {
                ptr::write_bytes(p, NEW_PATTERN, PAGE as usize)
            })
        }) == Ok(old);
    if !reused {
        serial::fail(b"SMP_UNMAP_RACE_REUSE");
    }
    check_live(frames, &[a, b]);
    if !within_ms(1_000, || {
        RACE_FAULTED.load(Ordering::Acquire) || RACE_STALE.load(Ordering::Acquire)
    }) {
        return sched_timeout(apic_of(0), online, total);
    }
    if RACE_STALE.load(Ordering::Acquire) {
        serial::write(b"SMP:SHOOTDOWN:STALE_READ\n");
        if cfg!(feature = "test-smp-proc-no-shootdown") {
            serial::exit(0x10);
        }
        serial::fail(b"SMP_UNMAP_RACE_STALE");
    }
    if cfg!(feature = "test-smp-proc-no-shootdown") {
        serial::fail(b"SMP_PROC_CONTROL_NO_STALE_READ");
    }
    // PS-1: each AP's handler ran exactly for the rounds it was a target of;
    // with two APs that is AP1 (the reader) alone, never AP2 (space B).
    let targets = ROUND_TARGETS.load(Ordering::Relaxed);
    let miss = (0..online).any(|slot| {
        let hits = CPU_LOCAL[slot].shootdowns.load(Ordering::Acquire) - before[slot];
        hits != u32::from(targets >> (slot + 1) & 1 != 0)
    });
    if miss || (online >= 2 && targets != 1 << 1) {
        serial::fail(b"SMP_SHOOTDOWN_TARGETS");
    }
    if frames
        .free(old, Owner::Kernel, paging::zero_dynamic)
        .is_err()
    {
        serial::fail(b"SMP_UNMAP_RACE_RECLAIM");
    }
    serial::write(b"SMP:UNMAP_RACE:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    if !stop_scheduler(online, total) {
        return false;
    }
    RACE_RUN.store(false, Ordering::Release);
    exit_spaces(online, total, frames, paging, spaces)
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
    if let Err(failed) = shootdown_kernel(paging::SHOOTDOWN_WINDOW, 1) {
        return gs_timeout(failed.0, online, total);
    }
    for (context, local) in aps.iter().zip(locals) {
        let apic = context.apic_id.load(Ordering::Relaxed);
        let resched = local.resched_count.load(Ordering::Acquire);
        if !send_ipi(apic, IPI_FIXED | RESCHED_VECTOR as u32)
            || !within_ms(500, || {
                local.resched_count.load(Ordering::Acquire) > resched
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

// ---- K06 step 5d-2 (R3-1..R3-4, ADR-081): M3 ring-3 processes on every CPU ----
//
// After the self-tests the APs wait in their home loop. `m3_go` (BSP, KERNEL
// held, after the M3 setup) gives each online CPU its M3 service stack, puts
// the M3 tick on 0xd0 and releases M3_GO; each AP then starts its LAPIC timer
// and enters `m3::ap_main`, which never returns. Every M3 entry takes KERNEL
// (R3-4); a process runs only on its home CPU (R3-2), so its translations live
// only there. `m3_stop` ends the M3 phase: every other CPU parks at its next
// entry (kernel CR3, IF=0, halted forever) and leaves ONLINE_CPUS.

#[cfg(feature = "native-smp-m3")]
static M3_GO: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "native-smp-m3")]
static M3_STOP: AtomicBool = AtomicBool::new(false);
/// CPU index bits of the CPUs `m3_park` halted.
#[cfg(feature = "native-smp-m3")]
static M3_PARKED: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "native-smp-m3")]
static M3_RECOVERY: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "native-smp-m3")]
static M3_RECOVERED: AtomicBool = AtomicBool::new(false);

/// Admission and recovery use a SeqCst store/check handshake. A pre-request
/// entrant stays visible until its ticket is released. A post-request AP
/// parks before it can obtain a ticket; INIT cannot strand a future waiter.
#[cfg(feature = "native-smp-m3")]
pub fn m3_entry_admit() {
    anchored().m3_entering.store(true, Ordering::SeqCst);
    if cpu_index() != 0 && M3_RECOVERY.load(Ordering::SeqCst) {
        anchored().m3_entering.store(false, Ordering::SeqCst);
        m3_park();
    }
}

#[cfg(feature = "native-smp-m3")]
pub fn m3_entry_left() {
    anchored().m3_entering.store(false, Ordering::SeqCst);
}

#[cfg(feature = "native-smp-m3")]
pub fn m3_request_recovery() {
    M3_RECOVERY.store(true, Ordering::SeqCst);
    if cpu_index() != 0 && !m3_reschedule(0) {
        serial::fail(b"SMP_M3_RECOVERY_BSP_IPI");
    }
}

#[cfg(feature = "native-smp-m3")]
pub fn m3_recovery_requested() -> bool {
    M3_RECOVERY.load(Ordering::SeqCst) && !M3_RECOVERED.load(Ordering::Acquire)
}

/// BSP/KERNEL, after bounded quiescence. The original mask is retained
/// because m3_stop_wait removes parked CPUs from ONLINE_CPUS.
#[cfg(feature = "native-smp-m3")]
pub fn m3_recovery_reset(original_mask: u64) {
    assert_eq!(cpu_index(), 0);
    assert!(lock::kernel_held() && m3_recovery_requested());
    let total = original_mask.count_ones() as usize;
    assert_eq!(original_mask, (1 << total) - 1);
    for cpu in 1..total {
        let local = &CPU_LOCAL[cpu - 1];
        let held = local.lockdep.load(Ordering::Acquire);
        if local.m3_entering.load(Ordering::SeqCst) || held != 0 {
            serial::write(b"SMP:QUARANTINE:LOCK_HELD:");
            decimal(if held == 0 {
                u64::from(level::KERNEL)
            } else {
                u64::from(held.trailing_zeros())
            });
            serial::write(b":CPU=");
            decimal(cpu as u64);
            serial::write(b"\n");
            serial::fail(b"SMP_M3_RECOVERY_LOCK_HELD_OR_WAITING");
        }
    }
    quarantine_all(total - 1, total);
    // Keep the admission barrier CLOSED through INIT and all metadata repair.
    // No AP is ever restarted in this boot; the BSP may continue alone.
    M3_RECOVERED.store(true, Ordering::Release);
}
/// LAPIC initial count of the AP's M3 tick: ~10 ms at divide 16 on QEMU's
/// 1 GHz APIC timer clock (ADR-080 band), the BSP's PIT M3 tick is 10 ms too.
#[cfg(feature = "native-smp-m3")]
const M3_TICK: u32 = 625_000;
/// Bound for `m3_stop`: an AP in ring 3 enters at its next tick (10 ms).
#[cfg(feature = "native-smp-m3")]
const M3_STOP_MS: u64 = 1_000;

#[cfg(feature = "native-smp-m3")]
pub const M3_DEFERRED_OFFSET: usize = core::mem::offset_of!(CpuLocal, m3_deferred);
#[cfg(feature = "native-smp-m3")]
pub const M3_RESCHED_OFFSET: usize = core::mem::offset_of!(CpuLocal, resched_count);
#[cfg(feature = "native-smp-m3")]
const M3_FAIR_TICKS: u32 = (LOCK_STUCK_MS / 10) as u32;

/// KERNEL held, local IF=0: a coherent snapshot of this CPU's already
/// counted CPL0 timer and 0xe8 events. The periodic PIT/LAPIC setup is unchanged.
#[cfg(feature = "native-smp-m3")]
pub fn m3_schedule_events() -> sched::ScheduleEvents {
    let flags: u64;
    // SAFETY: read-only CPL0 flags observation on this CPU's owned kernel
    // stack; PUSHFQ/POP restore its stack pointer and do not change flags.
    unsafe {
        asm!("pushfq", "pop {}", out(reg) flags, options(preserves_flags));
    }
    assert_eq!(flags & (1 << 9), 0, "M3 event snapshot requires IF=0");
    assert!(lock::kernel_held());
    let local = anchored();
    sched::ScheduleEvents {
        deferred_timer: local.m3_deferred.load(Ordering::Relaxed),
        reschedule: local.resched_count.load(Ordering::Relaxed),
    }
}

#[cfg(feature = "native-smp-m3")]
pub fn m3_wait_start() -> Option<(u64, u32)> {
    M3_GO
        .load(Ordering::Acquire)
        .then(|| (m3_time_ms(), anchored().m3_deferred.load(Ordering::Relaxed)))
}

#[cfg(feature = "native-smp-m3")]
pub fn m3_wait_end(start: Option<(u64, u32)>) {
    let Some(start) = start else {
        return;
    };
    let local = anchored();
    let ms = m3_time_ms().saturating_sub(start.0);
    let deferred = local
        .m3_deferred
        .load(Ordering::Relaxed)
        .wrapping_sub(start.1);
    local.m3_wait_max.fetch_max(ms, Ordering::Relaxed);
    local.m3_deferred_max.fetch_max(deferred, Ordering::Relaxed);
    if ms > LOCK_STUCK_MS || deferred > M3_FAIR_TICKS {
        serial::write(b"SMP:FAIR:FAIL:CPU=");
        decimal(cpu_index() as u64);
        serial::write(b"\n");
        serial::fail(b"SMP_FAIR");
    }
}

/// AC-2: maxima during KERNEL acquisition, using the existing stuck deadline.
#[cfg(feature = "native-smp-m3")]
pub fn m3_fairness_report() {
    for cpu in 0..MAX_CPUS {
        if online_mask() & (1 << cpu) == 0 {
            continue;
        }
        let local = if cpu == 0 {
            &BSP_LOCAL
        } else {
            &CPU_LOCAL[cpu - 1]
        };
        serial::write(b"SMP:FAIR:CPU=");
        decimal(cpu as u64);
        serial::write(b":WAIT_MS=");
        decimal(local.m3_wait_max.load(Ordering::Relaxed));
        serial::write(b":DEFERRED=");
        decimal(u64::from(local.m3_deferred_max.load(Ordering::Relaxed)));
        serial::write(b":LIMIT_MS=");
        decimal(LOCK_STUCK_MS);
        serial::write(b":LIMIT_TICKS=");
        decimal(u64::from(M3_FAIR_TICKS));
        serial::write(b"\n");
    }
}

#[cfg(feature = "native-smp-m3")]
global_asm!(
    ".pushsection .text.nos_smp_m3,\"ax\"",
    // Called by the M3 entry stub after its swapgs (CPL3 origin only, GS-1):
    // RAX = this CPU's M3 service stack top. Clobbers RAX only.
    ".globl nos_smp_m3_stack",
    "nos_smp_m3_stack:",
    "    mov rax, gs:[{stack}]",
    "    ret",
    // R3-3, entry: RAX = this CPU's current FXSAVE image. Clobbers RAX only.
    ".globl nos_smp_m3_fpu",
    "nos_smp_m3_fpu:",
    "    mov rax, gs:[{fpu}]",
    "    ret",
    // R3-3, exit (ADR-045): the FXRSTOR directly before the GPR pops; RSP
    // holds the saved context, so it is reached and left by jumps.
    ".globl nos_smp_m3_fxrstor",
    "nos_smp_m3_fxrstor:",
    "    mov rax, gs:[{fpu}]",
    "    .if {skip} == 0",
    "    fxrstor64 [rax]",
    "    .endif",
    "    jmp m3_fxrstor_done",
    ".popsection",
    stack = const core::mem::offset_of!(CpuLocal, m3_stack),
    fpu = const core::mem::offset_of!(CpuLocal, m3_fpu),
    skip = const cfg!(feature = "test-smp-m3-fpu-skip") as u8,
);

/// R3-3: the image the next M3 exit of this CPU restores (KERNEL held).
#[cfg(feature = "native-smp-m3")]
pub fn set_m3_fpu(image: u64) {
    anchored().m3_fpu.store(image, Ordering::Relaxed);
}

/// BSP, KERNEL held, IF=0, after the M3 setup and before the first resume.
/// `timer` is the M3 0xd0 stub; the BSP keeps the PIT (vector 32) as tick.
#[cfg(feature = "native-smp-m3")]
pub fn m3_go(timer: u64) {
    lapic_write(LVT_TIMER, MASKED | TIMER_VECTOR as u32);
    BSP_LOCAL
        .m3_stack
        .store(arch::service_stack(), Ordering::Relaxed);
    let online = ONLINE_CPUS.load(Ordering::Acquire);
    BSP_LOCAL.resched_count.store(0, Ordering::Relaxed);
    for cpu in 1..MAX_CPUS {
        if online & (1 << cpu) != 0 {
            CPU_LOCAL[cpu - 1].resched_count.store(0, Ordering::Relaxed);
            CPU_LOCAL[cpu - 1]
                .m3_stack
                .store(stack_base(cpu - 1, MAIN) + STACK_TOP, Ordering::Relaxed);
        }
    }
    arch::smp_gate(TIMER_VECTOR, timer);
    arch::smp_gate(RESCHED_VECTOR, m3_reschedule_entry as usize as u64);
    M3_GO.store(true, Ordering::Release);
    for cpu in 1..MAX_CPUS {
        if online & (1 << cpu) != 0 {
            send_ipi(apic_of_cpu(cpu), IPI_FIXED | IPI_VECTOR as u32);
        }
    }
    serial::write(b"SMP:M3:GO:CPUS=");
    decimal(u64::from(online.count_ones()));
    serial::write(b"\n");
}

/// AP home loop after M3_GO: periodic M3 tick, then the M3 scheduler.
#[cfg(feature = "native-smp-m3")]
fn m3_ap_enter(local: &CpuLocal) -> ! {
    if local.lockdep.load(Ordering::Relaxed) != 0 {
        serial::fail(b"SMP_M3_ENTER_LOCKED");
    }
    lapic_write(TIMER_DIVIDE, 0x3);
    lapic_write(LVT_TIMER, TIMER_PERIODIC | TIMER_VECTOR as u32);
    lapic_write(TIMER_INITIAL, M3_TICK);
    super::m3::ap_main()
}

/// The CPU index bits that run M3 processes (bit 0 = BSP).
#[cfg(feature = "native-smp-m3")]
pub fn online_mask() -> u64 {
    ONLINE_CPUS.load(Ordering::Acquire)
}

/// A remote wake is a hint; the target rechecks runnable state under KERNEL.
#[cfg(feature = "native-smp-m3")]
pub fn m3_reschedule(cpu: usize) -> bool {
    cpu < MAX_CPUS
        && online_mask() & (1 << cpu) != 0
        && send_ipi(apic_of_cpu(cpu), IPI_FIXED | RESCHED_VECTOR as u32)
}

#[cfg(feature = "native-smp-m3")]
pub fn m3_record_reschedule() {
    anchored().resched_count.fetch_add(1, Ordering::Relaxed);
}

#[cfg(feature = "native-smp-m3")]
pub fn m3_received_reschedules() -> u64 {
    (0..MAX_CPUS)
        .filter(|cpu| online_mask() & (1 << cpu) != 0)
        .map(|cpu| {
            let local = if cpu == 0 {
                &BSP_LOCAL
            } else {
                &CPU_LOCAL[cpu - 1]
            };
            u64::from(local.resched_count.load(Ordering::Relaxed))
        })
        .sum()
}

/// Calibrated TSC time for bounded SMP fixtures, independent of process ticks.
#[cfg(feature = "native-smp-m3")]
pub fn m3_time_ms() -> u64 {
    rdtsc() / TSC_PER_MS.load(Ordering::Acquire).max(1)
}

/// R3-4: TSS.RSP0 of the executing CPU.
#[cfg(feature = "native-smp-m3")]
pub fn rsp0(top: u64) {
    match cpu_index() {
        0 => arch::process_stack(top),
        cpu => arch::ap_rsp0(ap_tables(cpu - 1), top),
    }
}

/// This CPU's M3 service stack top (the BSP's is the M2 MAIN stack).
#[cfg(feature = "native-smp-m3")]
pub fn m3_stack() -> u64 {
    anchored().m3_stack.load(Ordering::Relaxed)
}

/// End of interrupt for a LAPIC source (the M3 0xd0 tick from CPL3).
#[cfg(feature = "native-smp-m3")]
pub fn eoi() {
    lapic_write(0xb0, 0);
}

#[cfg(feature = "native-smp-m3")]
pub fn m3_stopping() -> bool {
    M3_STOP.load(Ordering::Acquire)
}

/// Caller holds no lock: the end of the M3 phase reached this CPU. Kernel
/// CR3 (no user translation stays reachable), IF=0, halted forever.
#[cfg(feature = "native-smp-m3")]
pub fn m3_park() -> ! {
    if lockdep().load(Ordering::Relaxed) != 0 {
        serial::fail(b"SMP_M3_PARK_LOCKED");
    }
    lapic_write(LVT_TIMER, MASKED | TIMER_VECTOR as u32);
    // SAFETY: the kernel root maps this code and stack like every user root.
    unsafe {
        asm!("mov cr3, {}", in(reg) KERNEL_CR3.load(Ordering::Relaxed), options(nostack));
    }
    M3_PARKED.fetch_or(1 << cpu_index(), Ordering::Release);
    park()
}

/// KERNEL held: ends the M3 phase. Every other CPU checks the flag right
/// after its own `kernel_enter`, so no second CPU can start a stop.
#[cfg(feature = "native-smp-m3")]
pub fn m3_stop_begin() {
    M3_STOP.store(true, Ordering::Release);
}

/// After `m3_stop_begin`, with no lock held: true once every other online
/// CPU parked, which then leaves ONLINE_CPUS (no later round targets a
/// halted CPU). False (and the VM must stop) if one did not park in time.
#[cfg(feature = "native-smp-m3")]
pub fn m3_stop_wait() -> bool {
    let me = 1u64 << cpu_index();
    let others = ONLINE_CPUS.load(Ordering::Acquire) & !me;
    for cpu in 1..MAX_CPUS {
        if others & (1 << cpu) != 0 {
            send_ipi(apic_of_cpu(cpu), IPI_FIXED | IPI_VECTOR as u32);
        }
    }
    let parked = within_tsc(M3_STOP_MS, || {
        M3_PARKED.load(Ordering::Acquire) & others == others
    });
    if parked {
        ONLINE_CPUS.store(me, Ordering::Release);
    }
    parked
}
