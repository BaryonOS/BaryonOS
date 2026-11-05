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
//! - APs touch only their own stack, their context atomics, the read-only BSP
//!   register snapshot and the self-test atomics/spinlock, then halt with IF=0.
//!   They never enter allocator, paging, IPC or scheduler code. Exception: a
//!   CPU exception on an AP enters the shared M2 handlers, which write serial
//!   and exit the VM (IST gates triple-fault: APs have no TSS). That is a
//!   terminal stop, never a resumed or corrupted bring-up.
//! - If an AP that failed cannot be reset with INIT, the boot stops (fail):
//!   an uncontrolled CPU is never left running beside the kernel.
use super::{arch, paging, zero, FRAME_COUNT};
use crate::serial;
use boot_protocol::acpi::{Madt, MadtStatus};
use core::{
    arch::{asm, global_asm},
    cell::UnsafeCell,
    hint::spin_loop,
    ptr,
    sync::atomic::{fence, AtomicBool, AtomicU32, AtomicU64, Ordering},
};
use kernel_core::{Frames, Owner, PAGE};

/// Total CPUs (BSP plus APs) this profile starts; further MADT CPUs are skipped.
pub const MAX_CPUS: usize = 8;
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
const STACK_KINDS: usize = 3;
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

/// Per-CPU anchor reachable through IA32_GS_BASE. Offsets 0 and 12 are used
/// by the IPI stub; `slot` identifies the AP without trusting any register.
#[repr(C, align(64))]
struct CpuLocal {
    self_ptr: AtomicU64,
    slot: AtomicU32,
    ipis: AtomicU32,
}
const _: () = {
    assert!(core::mem::offset_of!(CpuLocal, self_ptr) == 0);
    assert!(core::mem::offset_of!(CpuLocal, ipis) == 12);
};
static CPU_LOCAL: [CpuLocal; MAX_APS] = [const {
    CpuLocal {
        self_ptr: AtomicU64::new(0),
        slot: AtomicU32::new(u32::MAX),
        ipis: AtomicU32::new(0),
    }
}; MAX_APS];
// Each slot is written only by the AP that claimed the matching context.
static mut CPU_TABLES: [arch::CpuTables; MAX_APS] = [const { arch::CpuTables::new() }; MAX_APS];

static BSP_REGISTERS: [AtomicU64; REGISTERS] = [const { AtomicU64::new(0) }; REGISTERS];
static GO: AtomicBool = AtomicBool::new(false);
static PARK: AtomicBool = AtomicBool::new(false);
const IPI_VECTOR: usize = 0xf0;
const SPURIOUS_VECTOR: usize = 0xff;
const IPI_ROUNDS: u32 = 3;

// IPI: count on this CPU (GS-anchored), EOI to this CPU's own xAPIC, return.
// Spurious: architecturally no EOI. Both run with IF cleared by the gate.
global_asm!(
    ".pushsection .text.nos_smp_ipi,\"ax\"",
    ".globl nos_smp_ipi_entry",
    "nos_smp_ipi_entry:",
    "    push rax",
    "    mov rax, gs:[0]",
    "    lock inc dword ptr [rax + 12]",
    "    mov dword ptr [{eoi}], 0",
    "    pop rax",
    "    iretq",
    ".globl nos_smp_spurious_entry",
    "nos_smp_spurious_entry:",
    "    iretq",
    ".popsection",
    eoi = const paging::LAPIC_WINDOW + 0xb0,
);
extern "C" {
    fn nos_smp_ipi_entry();
    fn nos_smp_spurious_entry();
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

/// A minimal test-and-test-and-set lock. Closure-scoped so no guard can leak
/// past the critical section; panics abort the kernel, so none unwinds it.
pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}
// SAFETY: `with` grants &mut T to exactly one CPU at a time (Acquire/Release).
unsafe impl<T: Send> Sync for SpinLock<T> {}
impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }
    pub fn with<R>(&self, operation: impl FnOnce(&mut T) -> R) -> R {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                spin_loop();
            }
        }
        // SAFETY: The successful Acquire CAS makes this CPU the only holder until
        // the Release store below; no reference escapes the closure.
        let result = operation(unsafe { &mut *self.value.get() });
        self.locked.store(false, Ordering::Release);
        result
    }
}

const ITERATIONS: u64 = 20_000;
static ATOMIC_TOTAL: AtomicU64 = AtomicU64::new(0);
static LOCKED_TOTAL: SpinLock<u64> = SpinLock::new(0);
static INSIDE: AtomicU32 = AtomicU32::new(0);
static VIOLATIONS: AtomicU32 = AtomicU32::new(0);

fn stress() {
    for _ in 0..ITERATIONS {
        ATOMIC_TOTAL.fetch_add(1, Ordering::Relaxed);
        LOCKED_TOTAL.with(|total| {
            if INSIDE.fetch_add(1, Ordering::Relaxed) != 0 {
                VIOLATIONS.fetch_add(1, Ordering::Relaxed);
            }
            // Split read/write widens the window a broken lock would lose.
            // SAFETY: Exclusive &mut from the lock; volatile keeps both accesses.
            let value = unsafe { ptr::read_volatile(total) };
            spin_loop();
            // SAFETY: Same exclusive location.
            unsafe { ptr::write_volatile(total, value + 1) };
            INSIDE.fetch_sub(1, Ordering::Relaxed);
        });
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
        // Address only (slot < MAX_APS checked above); this AP is the slot's
        // sole user because it holds the matching claimed context.
        ptr::addr_of_mut!(CPU_TABLES)
            .cast::<arch::CpuTables>()
            .wrapping_add(slot),
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
    let matches = observed == bsp && apic == context.apic_id.load(Ordering::Relaxed);
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
    context.state.store(DONE, Ordering::Release);
    ipi_phase(context, local, first)
}

/// Receive bounded unicast IPIs on vector 0xf0 until PARK; never any other
/// interrupt source: TPR 0, every LVT masked, spurious vector 0xff.
fn ipi_phase(context: &ApContext, local: &CpuLocal, first: bool) -> ! {
    for lvt in [0x320, 0x330, 0x340, 0x350, 0x360, 0x370] {
        lapic_write(lvt, 1 << 16);
    }
    lapic_write(0x80, 0);
    lapic_write(0xf0, 0x100 | SPURIOUS_VECTOR as u32);
    context.state.store(IPI_READY, Ordering::Release);
    if cfg!(feature = "test-smp-ap-ipi-deaf") && first {
        // Online but never accepts an interrupt: the BSP must quarantine all.
        park();
    }
    while !PARK.load(Ordering::Acquire) {
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
        // SAFETY: STI's one-instruction shadow makes `sti; hlt` atomic, so an
        // IPI that becomes pending after the PARK check still wakes this CPU.
        // The only deliverable vectors are the IPI and spurious gates.
        unsafe { asm!("sti", "hlt", "cli") };
    }
    context.state.store(PARKED, Ordering::Release);
    park()
}

/// Test-only (`test-smp-ap-nmi`/`test-smp-ap-double-fault`): recognize an NMI
/// or #DF that the CPU delivered on *this* AP's own IST stack.
pub fn ist_fault_marker(
    vector: u64,
    error: u64,
    cs: u64,
    frame: *const u64,
) -> Option<&'static [u8]> {
    if !cfg!(any(
        feature = "test-smp-ap-nmi",
        feature = "test-smp-ap-double-fault"
    )) || cs != 8
    {
        return None;
    }
    let anchor = rdmsr(0xc000_0101);
    let slot = (0..MAX_APS).find(|&slot| ptr::from_ref(&CPU_LOCAL[slot]) as u64 == anchor)?;
    let at = frame as u64;
    let within = |kind| {
        let base = stack_base(slot, kind);
        at >= base + 4096 && at + 22 * 8 <= base + STACK_TOP
    };
    if cfg!(feature = "test-smp-ap-nmi") && vector == 2 && within(NMI) {
        return Some(b"SMP:FAULT:AP_NMI_IST:PASS\n");
    }
    if cfg!(feature = "test-smp-ap-double-fault") && vector == 8 && error == 0 && within(DOUBLE) {
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
pub fn start(prepared: Prepared, madt: &Madt) {
    match madt.status {
        MadtStatus::Missing => return unavailable(b"NO_MADT"),
        MadtStatus::Invalid => return unavailable(b"INVALID_MADT"),
        MadtStatus::Valid => {}
    }
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
    // IDT is complete before the first AP can load it.
    arch::smp_gate(IPI_VECTOR, nos_smp_ipi_entry as usize as u64);
    arch::smp_gate(SPURIOUS_VECTOR, nos_smp_spurious_entry as usize as u64);
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
    if ATOMIC_TOTAL.load(Ordering::Acquire) != expected
        || LOCKED_TOTAL.with(|total| *total) != expected
    {
        serial::fail(b"SMP_SYNC_LOST_UPDATE");
    }
    if VIOLATIONS.load(Ordering::Acquire) != 0 {
        serial::fail(b"SMP_SYNC_EXCLUSION");
    }
    serial::write(b"SMP:SYNC:PASS:CPUS=");
    decimal(online as u64 + 1);
    serial::write(b"\n");
    ipis(online, cpus.len());
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
    serial::write(b"SMP:QUARANTINED:APS=");
    decimal(online as u64);
    serial::write(b"\nSMP:ONLINE:1/");
    decimal(total as u64);
    serial::write(b"\n");
}

/// Unicast only: a broadcast shorthand would also reach CPUs still owned by
/// firmware (capacity-skipped or never started) with unknown interrupt state.
fn ipis(online: usize, total: usize) {
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
        }
    }
    // Exactly one delivery per IPI: no lost, duplicated or foreign interrupt.
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
    if cfg!(feature = "test-smp-ap-nmi") && online != 0 {
        let apic = aps[0].apic_id.load(Ordering::Relaxed);
        if send_ipi(apic, IPI_NMI) {
            // The AP's IST2 handler must end the VM with its marker.
            within_ms(1_000, || false);
        }
        serial::fail(b"SMP_AP_NMI_NOT_ON_IST");
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
