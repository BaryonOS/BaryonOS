//! H05 step: QEMU-profile ACPI S3 suspend-to-RAM and resume (`acpi-s3`).
//!
//! Two callers share `suspend_once`: the IF0 M2 bootstrap (`native-acpi-s3`,
//! three cycles) and, in `native-system-s3`, syscall 0x91 of init's POWER
//! capability (m3/system.rs), which owns everything the wake does not restore
//! (PIC/PIT, devices, scheduler time). Both run on the only CPU with IF=0 and
//! the kernel CR3 active, so nothing runs between arming and SLP_EN.
//!
//! Ownership and trust model (single BSP, IF=0, no APs, no device but COM1):
//! - The wake page below 1 MiB is a Kernel-owned ConventionalMemory frame,
//!   reserved first-fit before any table allocation and written once through
//!   the firmware identity map before the CR3 switch, as the SMP SIPI page.
//!   It holds 16-bit code, the kernel CR3 and a per-cycle arm token. The
//!   kernel tables map it supervisor RW/NX only to update that token; it is
//!   executed solely with paging disabled, so W^X is unchanged.
//! - The 32/64-bit stages, their GDT and the wake stack live in the
//!   identity-mapped kernel image, as in `smp`.
//! - The FACS waking vector is stored only into the FACS extent that
//!   discovery validated (`parse_s3`) and proved to be UEFI ACPI NVS. It is
//!   armed immediately before SLP_EN and cleared right after a validated wake,
//!   and both stores are read back before the kernel relies on them.
//! - Resume authority: the 32-bit stage carries the page token into long mode
//!   and the 64-bit stage accepts it only if it equals the kernel's armed
//!   generation. An unarmed, stale or mismatched wake is reported and the VM
//!   stops; the saved context is never resumed for it.
//! - The firmware resume path is untrusted for memory integrity: a canary of
//!   Kernel-owned frames (first-fit, i.e. the lowest free RAM next to the wake
//!   page), the FACS hardware signature, the page-table audit and CR3 are all
//!   rechecked after every wake.
//!
//! Not covered: AML _PTS/_WAK methods (no interpreter; QEMU q35 profile only),
//! device drivers or services across sleep, wake sources other than the QMP
//! `system_wakeup` request, SMP, and any physical firmware or hardware.
use super::{arch, paging, power, zero, FRAME_COUNT};
use crate::serial;
use core::{
    arch::{asm, global_asm},
    fmt::Write,
    ptr,
    sync::atomic::{AtomicU32, Ordering},
};
use kernel_core::{Frames, Owner, PAGE};
use nieuw_os_acpi_power::{ExtendedVectorPolicy, Pm1Event, S3Plan, WakePlan};

/// `native-acpi-s3` cycles per boot; each uses a fresh nonzero generation.
#[cfg(feature = "native-acpi-s3")]
const CYCLES: u32 = 3;
/// Canary frames rechecked after every wake (256 KiB of the lowest free RAM).
const CANARY_FRAMES: usize = 64;
const WAKE_STACK: usize = 16 * 1024;
/// Bounded spin after SLP_EN; reaching its end means no transition happened.
const NO_TRANSITION_SPIN: u32 = 50_000_000;
// Wake page layout, see the trampoline below.
const PAGE_CR3: u64 = 8;
const PAGE_TOKEN: u64 = 32;
const MSR_EFER: u32 = 0xc000_0080;
const CR4_FSGSBASE: u64 = 1 << 16;
const CR4_OSXSAVE: u64 = 1 << 18;
const MSR_PAT: u32 = 0x277;

/// Control state captured just before SLP_EN and restored on the wake stack.
#[derive(Clone, Copy)]
#[repr(C)]
struct Saved {
    cr0: u64,
    cr4: u64,
    efer: u64,
    pat: u64,
}

#[repr(C, align(16))]
struct Stack([u8; WAKE_STACK]);

static mut NOS_S3_WAKE_STACK: Stack = Stack([0; WAKE_STACK]);
static mut NOS_S3_SAVED_RSP: u64 = 0;
/// Wake page base for the extended entry, which has no CS-derived base.
static mut NOS_S3_PAGE32: u32 = 0;
/// Which firmware transfer reached the kernel: 1 real-mode vector, 2 the
/// 32-bit extended vector; cleared by the kernel before every sleep.
static mut NOS_S3_ENTRY_PATH: u32 = 0;
/// See `ExtendedVectorPolicy::Edk2Version0Extended` for why this fixed
/// QEMU/OVMF profile also arms the 32-bit entry on a version-0 FACS.
const POLICY: ExtendedVectorPolicy = ExtendedVectorPolicy::Edk2Version0Extended;
static mut SAVED: Saved = Saved {
    cr0: 0,
    cr4: 0,
    efer: 0,
    pat: 0,
};
/// The armed generation; 0 means no wake is authorized.
static ARMED: AtomicU32 = AtomicU32::new(0);

// Wake page layout (offsets inside the copied page):
//   0  cli; short jmp to code      8  CR3 (patched once)
//  16  GDTR (limit, base32)       24  far pointer to entry32 (selector 0x08)
//  32  arm token (patched per cycle)
//  48  16-bit code, ESI = page linear base for the 32-bit stage
// ACPI transfers control in real mode with CS:IP = (vector >> 4):(vector & 0xf);
// the page-aligned vector therefore enters at offset 0 with CS = page >> 4.
global_asm!(
    ".pushsection .rodata.nos_s3_wake,\"a\"",
    ".code16",
    ".balign 4096",
    ".globl nos_s3_wake_start",
    "nos_s3_wake_start:",
    "    cli",
    "    jmp 1f",
    "    .org nos_s3_wake_start + 8",
    "    .long 0",
    "    .org nos_s3_wake_start + 16",
    "    .word nos_s3_gdt_end - nos_s3_gdt - 1",
    "    .long nos_s3_gdt",
    "    .org nos_s3_wake_start + 24",
    "    .long nos_s3_entry32",
    "    .word 0x08",
    "    .org nos_s3_wake_start + 32",
    "    .long 0",
    "    .org nos_s3_wake_start + 48",
    "1:",
    "    cld",
    "    movw %cs, %ax",
    "    movw %ax, %ds",
    "    xorl %esi, %esi",
    "    movw %ax, %si",
    "    shll $4, %esi",
    "    lgdtl 16",
    "    movl $1, %ebp",
    "    movl %cr0, %eax",
    "    orl $1, %eax",
    "    movl %eax, %cr0",
    "    ljmpl *24",
    ".globl nos_s3_wake_end",
    "nos_s3_wake_end:",
    // Accessed bits preset: the CPU never writes this read-only kernel GDT.
    ".balign 16",
    "nos_s3_gdt:",
    "    .quad 0",
    "    .quad 0x00cf9b000000ffff",
    "    .quad 0x00cf93000000ffff",
    "    .quad 0x00af9b000000ffff",
    ".globl nos_s3_gdt_end",
    "nos_s3_gdt_end:",
    "nos_s3_gdtr:",
    "    .word nos_s3_gdt_end - nos_s3_gdt - 1",
    "    .long nos_s3_gdt",
    ".popsection",
    ".pushsection .text.nos_s3_wake,\"ax\"",
    ".code32",
    // Extended (32-bit protected mode, paging off) entry: load our GDT and CS,
    // then join the real-mode path with ESI = wake page base.
    ".balign 16",
    ".globl nos_s3_entry32_direct",
    "nos_s3_entry32_direct:",
    "    cli",
    "    lgdtl nos_s3_gdtr",
    "    ljmpl $0x08, $5f",
    "5:",
    "    movl {page32}, %esi",
    "    movl $2, %ebp",
    "nos_s3_entry32:",
    // Known state for both entries: firmware defines neither EFLAGS nor CR4
    // for the waking vector. No stack is trusted here, so only `cld` (IF is
    // already 0); CR4 = PAE only until s3_wake restores the saved value.
    "    cld",
    "    movw $0x10, %ax",
    "    movw %ax, %ds",
    "    movw %ax, %es",
    "    movw %ax, %ss",
    // FS/GS are not used before `arch::load_tables` reloads every segment.
    "    movl %ebp, {path}",
    // The token is read while paging is off; the page is not executable later.
    "    movl 32(%esi), %edi",
    "    movl $0x20, %eax",
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
    "    ljmpl $0x18, $nos_s3_entry64",
    ".code64",
    "nos_s3_entry64:",
    "    movl %edi, %edi",
    "    leaq {stack}(%rip), %rsp",
    "    addq ${stack_size}, %rsp",
    // RFLAGS = 2 (IF=0, DF=0, no TF/AC/NT) on our own stack before Rust runs.
    "    pushq $2",
    "    popfq",
    "    call {wake}",
    "    movq %rax, %rsp",
    "    popq %r15",
    "    popq %r14",
    "    popq %r13",
    "    popq %r12",
    "    popq %rbx",
    "    popq %rbp",
    "    movl $1, %eax",
    "    ret",
    // nos_s3_suspend(pm1a_port, pm1a_value, pm1b_port_or_0, pm1b_value) -> u64:
    // 1 after a validated wake, 0 when SLP_EN did not stop the CPU.
    ".globl nos_s3_suspend",
    "nos_s3_suspend:",
    "    pushq %rbp",
    "    pushq %rbx",
    "    pushq %r12",
    "    pushq %r13",
    "    pushq %r14",
    "    pushq %r15",
    "    movq %rsp, {saved_rsp}(%rip)",
    "    movq %rdx, %r8",
    "    wbinvd",
    // ACPICA order: SLP_TYP with SLP_EN clear to PM1a then PM1b, then SLP_EN.
    "    movl %edi, %edx",
    "    movl %esi, %eax",
    "    andl $0xdfff, %eax",
    "    outw %ax, %dx",
    "    testq %r8, %r8",
    "    jz 2f",
    "    movl %r8d, %edx",
    "    movl %ecx, %eax",
    "    andl $0xdfff, %eax",
    "    outw %ax, %dx",
    "2:",
    "    movl %edi, %edx",
    "    movl %esi, %eax",
    "    outw %ax, %dx",
    "    testq %r8, %r8",
    "    jz 3f",
    "    movl %r8d, %edx",
    "    movl %ecx, %eax",
    "    outw %ax, %dx",
    "3:",
    "    movl ${spin}, %ecx",
    "4:",
    "    pause",
    "    decl %ecx",
    "    jnz 4b",
    "    popq %r15",
    "    popq %r14",
    "    popq %r13",
    "    popq %r12",
    "    popq %rbx",
    "    popq %rbp",
    "    xorl %eax, %eax",
    "    ret",
    ".popsection",
    stack = sym NOS_S3_WAKE_STACK,
    page32 = sym NOS_S3_PAGE32,
    path = sym NOS_S3_ENTRY_PATH,
    stack_size = const WAKE_STACK,
    wake = sym s3_wake,
    saved_rsp = sym NOS_S3_SAVED_RSP,
    spin = const NO_TRANSITION_SPIN,
    options(att_syntax),
);
extern "C" {
    static nos_s3_wake_start: u8;
    static nos_s3_wake_end: u8;
    static nos_s3_gdt_end: u8;
    fn nos_s3_suspend(pm1a: u64, pm1a_value: u64, pm1b: u64, pm1b_value: u64) -> u64;
    fn nos_s3_entry32_direct();
}

/// Runs on the wake stack with the trampoline GDT, the firmware IDT and the
/// kernel CR3. Returns the saved suspend stack only for the armed generation.
extern "C" fn s3_wake(token: u64) -> u64 {
    // SAFETY: SAVED was written by this CPU before SLP_EN and is read-only now.
    let saved = unsafe { *ptr::addr_of!(SAVED) };
    // SAFETY: CPL0, IF=0. CR4/CR0/EFER/PAT are restored to the exact values
    // read just before sleep; PAE, PG and LME are already set identically.
    unsafe {
        asm!("mov cr4, {}", in(reg) saved.cr4, options(nostack));
        asm!("mov cr0, {}", in(reg) saved.cr0, options(nostack));
    }
    wrmsr(MSR_EFER, saved.efer);
    wrmsr(MSR_PAT, saved.pat);
    serial::init();
    let armed = ARMED.load(Ordering::Acquire);
    if armed == 0 || token != u64::from(armed) {
        let _ = writeln!(
            Serial,
            "POWER:S3:WAKE:REJECTED:TOKEN:ARMED={armed}:PAGE={token:#x}"
        );
        serial::exit(0x10);
    }
    arch::reload_after_resume();
    // SAFETY: Written by nos_s3_suspend on this CPU for this armed generation.
    unsafe { *ptr::addr_of!(NOS_S3_SAVED_RSP) }
}

struct Serial;
impl Write for Serial {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        serial::write(text.as_bytes());
        Ok(())
    }
}

fn rdmsr(msr: u32) -> u64 {
    let (lo, hi): (u32, u32);
    // SAFETY: CPL0 read of an architectural MSR named by this module.
    unsafe {
        asm!("rdmsr", in("ecx") msr, out("eax") lo, out("edx") hi, options(nostack));
    }
    u64::from(hi) << 32 | u64::from(lo)
}

fn wrmsr(msr: u32, value: u64) {
    // SAFETY: CPL0 write restoring this CPU's own pre-sleep EFER/PAT value.
    unsafe {
        asm!("wrmsr", in("ecx") msr, in("eax") value as u32, in("edx") (value >> 32) as u32, options(nostack));
    }
}

fn cr(read_cr4: bool) -> u64 {
    let value: u64;
    // SAFETY: Read-only CPL0 control-register query.
    unsafe {
        if read_cr4 {
            asm!("mov {}, cr4", out(reg) value, options(nomem, nostack));
        } else {
            asm!("mov {}, cr0", out(reg) value, options(nomem, nostack));
        }
    }
    value
}

fn cr3() -> u64 {
    let value: u64;
    // SAFETY: Read-only CPL0 control-register query.
    unsafe { asm!("mov {}, cr3", out(reg) value, options(nomem, nostack)) };
    value
}

/// Call before any paging allocation: the first free ConventionalMemory frame
/// becomes the permanent Kernel-owned wake page if it is real-mode reachable.
pub fn reserve_wake_page(frames: &mut Frames<FRAME_COUNT>) -> Option<u64> {
    let page = frames.allocate(Owner::Kernel, zero).ok()?;
    if (0x1000..0xa0000).contains(&page) {
        Some(page)
    } else {
        frames.free(page, Owner::Kernel, zero).ok()?;
        None
    }
}

pub struct Prepared {
    page: u64,
    canary: [u64; CANARY_FRAMES],
}

/// Written once by `prepare` on the single BSP (IF=0, before the CR3 switch)
/// and read-only afterwards; `Err` names why no sleep can be offered.
static mut PREPARED: Result<Prepared, &'static [u8]> = Err(b"NOT_PREPARED");

/// Before `map_tables`/`activate`: write the wake page through the still-active
/// firmware identity map and map it plus the canary frames RW/NX.
pub fn prepare(frames: &mut Frames<FRAME_COUNT>, paging: &mut paging::Paging, page: Option<u64>) {
    let result = prepare_frames(frames, paging, page);
    // SAFETY: Single bootstrap writer with IF=0; no reader exists before this
    // store, and nothing writes PREPARED again.
    unsafe { ptr::addr_of_mut!(PREPARED).write(result) };
}

fn prepared() -> Result<&'static Prepared, &'static [u8]> {
    // SAFETY: Written once by `prepare` before any caller; read-only from then on.
    match unsafe { &*ptr::addr_of!(PREPARED) } {
        Ok(prepared) => Ok(prepared),
        Err(reason) => Err(reason),
    }
}

fn prepare_frames(
    frames: &mut Frames<FRAME_COUNT>,
    paging: &mut paging::Paging,
    page: Option<u64>,
) -> Result<Prepared, &'static [u8]> {
    let page = page.ok_or(b"NO_WAKE_PAGE".as_slice())?;
    let root = paging.root;
    // Address computation of the linked trampoline image only.
    let (start, end) = (
        ptr::addr_of!(nos_s3_wake_start) as u64,
        ptr::addr_of!(nos_s3_wake_end) as u64,
    );
    let length = end - start;
    let image_end = ptr::addr_of!(nos_s3_gdt_end) as u64;
    if length > PAGE
        || root >= 1 << 32
        || !root.is_multiple_of(PAGE)
        || end >= 1 << 32
        || image_end >= 1 << 32
    {
        return Err(b"WAKE_LAYOUT");
    }
    // SAFETY: `page` is an exclusive Kernel-owned ConventionalMemory frame below
    // 1 MiB; the firmware CR3 that is still active identity-maps it. The source
    // is our own read-only image. Exactly `length` <= one page is copied, then
    // the aligned CR3 slot inside it is written.
    unsafe {
        ptr::copy_nonoverlapping(start as *const u8, page as *mut u8, length as usize);
        ((page + PAGE_CR3) as *mut u32).write_volatile(root as u32);
        ptr::addr_of_mut!(NOS_S3_PAGE32).write(page as u32);
    }
    paging.map(frames, page, page, true, false);
    let mut canary = [0u64; CANARY_FRAMES];
    for slot in canary.iter_mut() {
        *slot = frames
            .allocate(Owner::Kernel, zero)
            .map_err(|_| b"CANARY_FRAMES".as_slice())?;
        paging.map(frames, *slot, *slot, true, false);
    }
    Ok(Prepared { page, canary })
}

fn pattern(generation: u32, frame: usize, word: usize) -> u64 {
    // splitmix64 over (generation, frame, word): every word differs per cycle.
    let mut z = (u64::from(generation) << 40 | (frame as u64) << 20 | word as u64)
        .wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn canary(frames: &[u64; CANARY_FRAMES], generation: u32, fill: bool) -> bool {
    let words = (PAGE / 8) as usize;
    let mut intact = true;
    for (index, &frame) in frames.iter().enumerate() {
        for word in 0..words {
            let expected = pattern(generation, index, word);
            let address = (frame + (word * 8) as u64) as *mut u64;
            // SAFETY: Exclusive Kernel-owned canary frame, identity-mapped
            // supervisor RW/NX by `prepare`; aligned u64 access inside it.
            unsafe {
                if fill {
                    address.write_volatile(expected);
                } else if address.read_volatile() != expected {
                    intact = false;
                }
            }
        }
    }
    intact
}

fn set_token(page: u64, token: u32) {
    // SAFETY: The wake page is Kernel-owned and mapped supervisor RW/NX at its
    // identity address; offset 32 is the aligned token slot of the layout.
    unsafe { ((page + PAGE_TOKEN) as *mut u32).write_volatile(token) };
}

/// Stores one FACS waking-vector plan, then reads the FACS back and checks
/// that exactly the planned fields hold the planned values.
fn store_facs(
    paging: &mut paging::Paging,
    facs: u64,
    length: usize,
    plan: WakePlan,
    copy: &mut [u8],
) -> bool {
    let mut stores: [Option<(usize, [u8; 8], usize)>; 3] = [
        Some((
            WakePlan::LEGACY_VECTOR_OFFSET,
            u64::from(plan.legacy_vector()).to_le_bytes(),
            4,
        )),
        plan.extended_vector()
            .map(|value| (WakePlan::EXTENDED_VECTOR_OFFSET, value.to_le_bytes(), 8)),
        plan.ospm_flags().map(|flags| {
            (
                WakePlan::OSPM_FLAGS_OFFSET,
                u64::from(flags).to_le_bytes(),
                4,
            )
        }),
    ];
    for (offset, bytes, len) in stores.iter_mut().flatten() {
        // SAFETY: `facs..facs+length` is the FACS extent discovery validated
        // with parse_s3 and proved to be UEFI ACPI NVS; the offsets are the
        // OSPM-owned waking-vector fields of the plan for this FACS version.
        if unsafe { paging.write_firmware(facs + *offset as u64, &bytes[..*len]) }.is_err() {
            return false;
        }
    }
    let copy = &mut copy[..length];
    // SAFETY: Same validated NVS extent, read through the RO/NX scratch leaf.
    if unsafe { paging.read_firmware(facs, copy) }.is_err() {
        return false;
    }
    stores
        .iter()
        .flatten()
        .all(|(offset, bytes, len)| copy[*offset..*offset + *len] == bytes[..*len])
}

fn clear_status(event: Pm1Event, bits: u16) {
    if bits != 0 {
        power::out16(event.status(), bits);
    }
}

/// Why one sleep request ended without a transition. Neither variant wrote
/// PM1 control (SLP_TYP/SLP_EN); the kernel reported the reason on the serial
/// line (only the silent `available` probe and the generation-0 guard do not).
/// Every other outcome stops the VM.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Unavailable {
    /// Refused from the read-only plan before any I/O: no PM1 status or
    /// control, FACS, wake-page or canary access.
    Refused,
    /// Refused after the pre-sleep writes: WAK_STS was cleared in PM1a_STS,
    /// the canary filled, ARMED and the page token set, and then the FACS arm
    /// store or its read-back failed. ARMED and the token were cleared again
    /// and the disarmed FACS plan was stored and read back intact
    /// (`POWER:S3:REJECTED:FacsStore:DISARMED`); a failed disarm stops the VM
    /// (`S3_FACS_DISARM`) instead of returning.
    FacsRestored,
}

/// The constant, validated inputs of every sleep, derived from the read-only
/// discovery snapshot. `report` writes the reason for a refusal; neither path
/// touches PM1, the FACS or the wake page.
fn plan(report: bool) -> Option<(&'static Prepared, S3Plan, WakePlan, WakePlan)> {
    let say = |text: &[u8]| {
        if report {
            serial::write(text);
        }
    };
    let prepared = match prepared() {
        Ok(prepared) => prepared,
        Err(reason) => {
            say(b"POWER:S3:UNAVAILABLE:");
            say(reason);
            say(b"\n");
            return None;
        }
    };
    let plan = match power::s3() {
        power::S3State::Ready(plan) => plan,
        power::S3State::Unavailable => {
            say(b"POWER:S3:UNAVAILABLE\n");
            return None;
        }
        power::S3State::Unreadable => {
            say(b"POWER:S3:UNREADABLE\n");
            return None;
        }
        power::S3State::NotNvs => {
            say(b"POWER:S3:REJECTED:FacsNotNvs\n");
            return None;
        }
        power::S3State::Rejected(error) => {
            if report {
                let _ = writeln!(Serial, "POWER:S3:REJECTED:{error:?}");
            }
            return None;
        }
    };
    let facs = plan.facs();
    let entry32 = nos_s3_entry32_direct as unsafe extern "C" fn() as usize as u64;
    let wake = match facs.wake_plan(prepared.page, Some(entry32), POLICY) {
        Ok(wake) => wake,
        Err(error) => {
            if report {
                let _ = writeln!(Serial, "POWER:S3:REJECTED:{error:?}");
            }
            return None;
        }
    };
    Some((prepared, plan, wake, facs.disarmed_plan(POLICY)))
}

/// True when `suspend_once` would arm the FACS and write PM1: the wake page
/// and canary were prepared, discovery validated an S3 plan in ACPI NVS, the
/// wake plan exists and the profile has no PM1b. No I/O and no output.
#[cfg(feature = "native-system-s3")]
pub fn available() -> bool {
    plan(false).is_some_and(|(_, plan, _, _)| plan.pm1b().is_none() && plan.pm1b_event().is_none())
}

/// One validated S3 cycle with the nonzero `generation` as arm token. The
/// caller holds the only CPU with IF=0 and the kernel CR3 active. Returns
/// only after a validated wake (`post` ran on the resumed tables first) or
/// after a reported refusal without any PM1 control (SLP_EN) write; see
/// `Unavailable` for the pre-sleep writes the `FacsRestored` refusal already
/// made and undid. Every other outcome stops the VM.
pub fn suspend_once(
    paging: &mut paging::Paging,
    generation: u32,
    post: impl FnOnce(&paging::Paging),
) -> Result<(), Unavailable> {
    if generation == 0 {
        // 0 is the "nothing armed" value of ARMED and the wake page.
        return Err(Unavailable::Refused);
    }
    let (prepared, plan, wake, disarm) = plan(true).ok_or(Unavailable::Refused)?;
    let facs = plan.facs();
    let pm1a = plan.pm1a();
    let event = plan.pm1a_event();
    let snapshot = power::facs_bytes();
    let length = u32::from_le_bytes([snapshot[4], snapshot[5], snapshot[6], snapshot[7]]) as usize;
    let signature = facs.hardware_signature();
    if generation == 1 {
        let _ = writeln!(
            Serial,
            "POWER:S3:PLAN:PM1A={:#06x}:SLP_TYP={}:PM1A_STS={:#06x}:FACS_VERSION={}:REAL={:#x}:PM32={:#x}:POLICY={:?}",
            pm1a.address(),
            pm1a.sleep_type(),
            event.status(),
            facs.version(),
            wake.legacy_vector(),
            wake.extended_vector().unwrap_or(0),
            POLICY
        );
    }
    if plan.pm1b().is_some() || plan.pm1b_event().is_some() {
        // The QEMU q35 profile has neither; PM1b paths are parser-tested only.
        serial::write(b"POWER:S3:REJECTED:Pm1bUnsupported\n");
        return Err(Unavailable::Refused);
    }
    // The wake path restores the saved CR4 verbatim: user-writable GS bases
    // (FSGSBASE) or XSAVE state would bypass the GS discipline and the
    // no-FPU-context profiles. `arch::protection` clears both; recheck here.
    if cr(true) & (CR4_FSGSBASE | CR4_OSXSAVE) != 0 {
        serial::fail(b"S3_CR4");
    }
    // The wake page resumes on `paging.root`; no other address space (in
    // particular no user CR3) may be live when the transition is armed.
    if cr3() != paging.root {
        serial::fail(b"S3_CR3");
    }
    // SAFETY: Exclusive static scratch for FACS read-back, single BSP, IF=0.
    let copy = unsafe { &mut *ptr::addr_of_mut!(FACS_COPY) };
    let cycle = generation;
    // Pre-sleep: clear WAK_STS, fill the canary, arm token and FACS.
    clear_status(event, Pm1Event::WAKE_STATUS);
    canary(&prepared.canary, cycle, true);
    ARMED.store(cycle, Ordering::Release);
    let page_token = if cfg!(any(
        feature = "test-acpi-s3-stale-token",
        feature = "test-system-s3-stale-token"
    )) {
        // Negative fixture: the page carries another generation's token.
        cycle ^ 0x5a5a_0000
    } else {
        cycle
    };
    set_token(prepared.page, page_token);
    if !store_facs(paging, facs.source_address(), length, wake, copy) {
        // Not a refusal before I/O: WAK_STS, the canary, ARMED and the token
        // were already written. Undo the arm and never leave a partially
        // armed vector behind; report the restored FACS, or stop the VM.
        ARMED.store(0, Ordering::Release);
        set_token(prepared.page, 0);
        if store_facs(paging, facs.source_address(), length, disarm, copy) {
            serial::write(b"POWER:S3:REJECTED:FacsStore:DISARMED\n");
            return Err(Unavailable::FacsRestored);
        }
        serial::fail(b"S3_FACS_DISARM");
    }
    // SAFETY: Single CPU; SAVED is read only by s3_wake after this point.
    unsafe {
        ptr::addr_of_mut!(SAVED).write(Saved {
            cr0: cr(false),
            cr4: cr(true),
            efer: rdmsr(MSR_EFER),
            pat: rdmsr(MSR_PAT),
        });
    }
    // SAFETY: Single CPU; only the wake stages write this before s3_wake.
    unsafe { ptr::addr_of_mut!(NOS_S3_ENTRY_PATH).write_volatile(0) };
    let current = power::in16(pm1a.address());
    let sci_before = current & 1;
    let _ = writeln!(Serial, "POWER:S3:ENTER:CYCLE={cycle}:SCI_EN={sci_before}");
    // SAFETY: The PM1a port and value come from the validated S3 plan;
    // the suspend routine saves the callee-saved registers and stack and
    // returns 1 only through s3_wake's armed-generation check.
    let resumed = unsafe {
        nos_s3_suspend(
            u64::from(pm1a.address()),
            u64::from(pm1a.apply_to(current)),
            0,
            0,
        )
    };
    // Post-wake (or failed transition): disarm before anything else.
    ARMED.store(0, Ordering::Release);
    set_token(prepared.page, 0);
    let disarmed = store_facs(paging, facs.source_address(), length, disarm, copy);
    if resumed != 1 {
        // A requested sleep that never happened is a failure, not a
        // handled rejection: report the disarm outcome, then fail closed.
        serial::write(if disarmed {
            b"POWER:S3:NO_TRANSITION:DISARMED\n"
        } else {
            b"POWER:S3:NO_TRANSITION:STORE_FAILED\n"
        });
        serial::fail(b"S3_NO_TRANSITION");
    }
    // SAFETY: Written by the wake stage of this CPU before s3_wake ran.
    let path = unsafe { ptr::addr_of!(NOS_S3_ENTRY_PATH).read_volatile() };
    let status = power::in16(event.status());
    clear_status(event, status);
    // Firmware should restore SCI_EN; edk2/OVMF does not. Like Linux
    // ("bug compatibility"), restore the pre-sleep ACPI mode ourselves.
    let control = power::in16(pm1a.address());
    let sci = match (sci_before, control & 1) {
        (_, 1) => "KEPT",
        (1, _) => {
            power::out16(pm1a.address(), (control | 1) & !(1 << 13));
            if power::in16(pm1a.address()) & 1 == 1 {
                "RESTORED"
            } else {
                "LOST"
            }
        }
        _ => "OFF",
    };
    let signature_same = u32::from_le_bytes([copy[8], copy[9], copy[10], copy[11]]) == signature;
    let intact = canary(&prepared.canary, cycle, false);
    let cr3_same = cr3() == paging.root;
    post(paging);
    let _ = writeln!(
        Serial,
        "POWER:S3:RESUMED:CYCLE={cycle}:ENTRY={}:PM1_STS={status:#06x}:WAK_STS={}:SCI_EN={sci}:HWSIG={}:FACS={}:CANARY={}:CR3={}",
        match path {
            1 => "REAL",
            2 => "PM32",
            _ => "UNKNOWN",
        },
        u8::from(status & Pm1Event::WAKE_STATUS != 0),
        if signature_same { "SAME" } else { "CHANGED" },
        if disarmed { "DISARMED" } else { "STORE_FAILED" },
        if intact { "PASS" } else { "FAIL" },
        if cr3_same { "SAME" } else { "CHANGED" },
    );
    if status & Pm1Event::WAKE_STATUS == 0
        || !(1..=2).contains(&path)
        || sci == "LOST"
        || !signature_same
        || !disarmed
        || !intact
        || !cr3_same
    {
        serial::fail(b"S3_RESUME_CHECK");
    }
    Ok(())
}

/// `native-acpi-s3`: the bounded S3 cycles after M2 passed; returns only
/// after the last validated wake or after a reported rejection without any
/// PM1 control (SLP_EN) write (`Unavailable` lists what each one wrote).
#[cfg(feature = "native-acpi-s3")]
pub fn run(frames: &Frames<FRAME_COUNT>, paging: &mut paging::Paging) {
    for cycle in 1..=CYCLES {
        if suspend_once(paging, cycle, |paging| paging.audit(frames)).is_err() {
            return;
        }
    }
    serial::write(b"POWER:S3:PASS\n");
}

static mut FACS_COPY: [u8; 4096] = [0; 4096];
