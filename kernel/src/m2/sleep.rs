//! H05 step: QEMU-profile ACPI S3 suspend-to-RAM and resume (`native-acpi-s3`,
//! three cycles in the IF0 bootstrap; `test-system-s3`, one cycle granted to
//! init's POWER capability through syscall 0x91, ADR-H05-S3-SYSTEM-20261123).
//!
//! Ownership and trust model (single BSP, IF=0, no APs, no device but COM1):
//! - The wake page below 1 MiB is a Kernel-owned ConventionalMemory frame,
//!   reserved first-fit before any table allocation and written once through
//!   the firmware identity map before the CR3 switch, as the SMP SIPI page.
//!   It holds 16-bit code and a per-cycle arm token. The
//!   kernel tables map it supervisor RW/NX only to update that token; it is
//!   executed solely with paging disabled, so W^X is unchanged.
//! - The 32/64-bit stages, their GDT, the wake stack and the kernel CR3 the
//!   32-bit stage loads live in the identity-mapped kernel image, as in `smp`.
//! - The FACS waking vector is stored only into the FACS extent that
//!   discovery validated (`parse_s3`) and proved to be UEFI ACPI NVS;
//!   `paging::write_firmware` refuses every store outside that extent. It is
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
use nieuw_os_acpi_power::{ExtendedVectorPolicy, Pm1Event, Pm1Setting, WakePlan};

/// Suspend/resume cycles per boot; each uses a fresh nonzero generation.
#[cfg(feature = "native-acpi-s3")]
const CYCLES: u32 = 3;
/// Canary frames rechecked after every wake (256 KiB of the lowest free RAM).
const CANARY_FRAMES: usize = 64;
const WAKE_STACK: usize = 16 * 1024;
/// Bounded spin after SLP_EN; reaching its end means no transition happened.
const NO_TRANSITION_SPIN: u32 = 50_000_000;
// Wake page layout, see the trampoline below.
const PAGE_TOKEN: u64 = 32;
const MSR_EFER: u32 = 0xc000_0080;
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
/// Kernel root the 32-bit stage loads into CR3, written once by `prepare`;
/// kept in the kernel image, not the low wake page, so no wake-page byte
/// selects the page tables used before the ARMED check.
static mut NOS_S3_CR3: u32 = 0;
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
/// isa-debug-exit value of a rejected wake (raw QEMU exit 37), in every S3
/// profile; no other kernel or loader path writes it, so it is neither the
/// pass (0x10, raw 33) nor a generic failure (0x11, raw 35).
const WAKE_REJECTED_EXIT: u32 = 0x12;
/// isa-debug-exit value of a refused S3 plan in `native-acpi-s3` (raw QEMU
/// exit 39): every `resolve` rejection and a cleanly disarmed FACS store
/// failure, none of which performed a PM1 write. No other kernel or loader
/// path writes it, so a refused plan can never end with the pass (0x10).
#[cfg(feature = "native-acpi-s3")]
const PLAN_REJECTED_EXIT: u32 = 0x13;

// Wake page layout (offsets inside the copied page):
//   0  cli; short jmp to code      8  reserved (zero)
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
    // CR3 comes from the kernel image (DS is flat now), not the wake page.
    "    movl {cr3}, %eax",
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
    cr3 = sym NOS_S3_CR3,
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
    // Authority before any restore: the check reads only the token register
    // and ARMED (kernel .bss, mapped by the kernel CR3 the 32-bit stage
    // loaded), never SAVED, so an unarmed wake is reported, not faulted on.
    // The reject path needs only port I/O and the soft-float kernel's integer
    // code, which the stage's CR0 (PG|WP|PE), CR4.PAE and EFER (LME|NXE) run.
    let armed = ARMED.load(Ordering::Acquire);
    if armed == 0 || token != u64::from(armed) {
        serial::init();
        let _ = writeln!(
            Serial,
            "POWER:S3:WAKE:REJECTED:TOKEN:ARMED={armed}:PAGE={token:#x}"
        );
        serial::exit(WAKE_REJECTED_EXIT);
    }
    // SAFETY: `cycle` writes SAVED on this CPU before it arms a generation,
    // and a matching token exists only for an armed one; read-only now.
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

pub type Preparation = Result<Prepared, &'static [u8]>;

/// Before `map_tables`/`activate`: write the wake page through the still-active
/// firmware identity map and map it plus the canary frames RW/NX.
pub fn prepare(
    frames: &mut Frames<FRAME_COUNT>,
    paging: &mut paging::Paging,
    page: Option<u64>,
) -> Preparation {
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
    // is our own read-only image. Exactly `length` <= one page is copied. The
    // two u32 statics are written once here on the single BSP before any
    // wake stage can read them; `root < 4 GiB` was checked above.
    unsafe {
        ptr::copy_nonoverlapping(start as *const u8, page as *mut u8, length as usize);
        ptr::addr_of_mut!(NOS_S3_PAGE32).write(page as u32);
        ptr::addr_of_mut!(NOS_S3_CR3).write(root as u32);
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
        // write_firmware refuses any store outside that recorded extent.
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

/// Everything one suspend cycle needs, resolved once from discovery.
struct Ctx {
    page: u64,
    canary: [u64; CANARY_FRAMES],
    facs: u64,
    length: usize,
    wake: WakePlan,
    disarm: WakePlan,
    pm1a: Pm1Setting,
    event: Pm1Event,
    signature: u32,
}

/// Resolves the S3 plan and prints exactly the markers `run` always printed:
/// the unavailability or rejection reason, or the PLAN line (and a PM1b
/// rejection after it). `None` means no PM1 write may follow.
fn resolve(prepared: Preparation) -> Option<Ctx> {
    let prepared = match prepared {
        Ok(prepared) => prepared,
        Err(reason) => {
            serial::write(b"POWER:S3:UNAVAILABLE:");
            serial::write(reason);
            serial::write(b"\n");
            return None;
        }
    };
    let plan = match power::s3() {
        power::S3State::Ready(plan) => plan,
        power::S3State::Unavailable => {
            serial::write(b"POWER:S3:UNAVAILABLE\n");
            return None;
        }
        power::S3State::Unreadable => {
            serial::write(b"POWER:S3:UNREADABLE\n");
            return None;
        }
        power::S3State::NotNvs => {
            serial::write(b"POWER:S3:REJECTED:FacsNotNvs\n");
            return None;
        }
        power::S3State::Rejected(error) => {
            let _ = writeln!(Serial, "POWER:S3:REJECTED:{error:?}");
            return None;
        }
    };
    let facs = plan.facs();
    // `Ready` is discovery's proof that this whole extent is ACPI NVS
    // (type 10); parse_s3 bound the snapshot's length field to that extent.
    let snapshot = power::facs_bytes();
    let length = u32::from_le_bytes([snapshot[4], snapshot[5], snapshot[6], snapshot[7]]) as usize;
    if !paging::record_firmware_write_extent(facs.source_address(), length) {
        serial::write(b"POWER:S3:REJECTED:FacsExtent\n");
        return None;
    }
    let entry32 = nos_s3_entry32_direct as unsafe extern "C" fn() as usize as u64;
    let wake = match facs.wake_plan(prepared.page, Some(entry32), POLICY) {
        Ok(wake) => wake,
        Err(error) => {
            let _ = writeln!(Serial, "POWER:S3:REJECTED:{error:?}");
            return None;
        }
    };
    let pm1a = plan.pm1a();
    let event = plan.pm1a_event();
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
    if plan.pm1b().is_some() || plan.pm1b_event().is_some() {
        // The QEMU q35 profile has neither; PM1b paths are parser-tested only.
        serial::write(b"POWER:S3:REJECTED:Pm1bUnsupported\n");
        return None;
    }
    Some(Ctx {
        page: prepared.page,
        canary: prepared.canary,
        facs: facs.source_address(),
        length,
        wake,
        disarm: facs.disarmed_plan(POLICY),
        pm1a,
        event,
        signature: facs.hardware_signature(),
    })
}

/// One armed suspend and validated wake. Returns false only for a FACS store
/// failure that was disarmed cleanly before any PM1 write; every failure
/// after SLP_EN fails closed. `frames` is None under M3, whose user mappings
/// the kernel-root alias audit cannot see; the caller audits instead.
fn cycle(
    frames: Option<&Frames<FRAME_COUNT>>,
    paging: &mut paging::Paging,
    ctx: &Ctx,
    cycle: u32,
) -> bool {
    // SAFETY: Exclusive static scratch for FACS read-back, single BSP, IF=0.
    let copy = unsafe { &mut *ptr::addr_of_mut!(FACS_COPY) };
    let (facs, length, event, pm1a) = (ctx.facs, ctx.length, ctx.event, ctx.pm1a);
    // Pre-sleep: clear WAK_STS, fill the canary, arm token and FACS.
    clear_status(event, Pm1Event::WAKE_STATUS);
    canary(&ctx.canary, cycle, true);
    // Negative fixture: SAVED is poisoned (all-ones faults on any restore),
    // no generation is armed and the page token is 0, so token == ARMED and
    // only the `armed == 0` clause in s3_wake stands before a faulting restore.
    let unarmed = cfg!(feature = "test-acpi-s3-unarmed-wake");
    let saved = if unarmed {
        Saved {
            cr0: u64::MAX,
            cr4: u64::MAX,
            efer: u64::MAX,
            pat: u64::MAX,
        }
    } else {
        Saved {
            cr0: cr(false),
            cr4: cr(true),
            efer: rdmsr(MSR_EFER),
            pat: rdmsr(MSR_PAT),
        }
    };
    // SAFETY: Single CPU; SAVED is written before the generation is armed,
    // so s3_wake reads it only for a token this cycle authorized.
    unsafe { ptr::addr_of_mut!(SAVED).write(saved) };
    ARMED.store(if unarmed { 0 } else { cycle }, Ordering::Release);
    let page_token = if cfg!(feature = "test-acpi-s3-stale-token") {
        // Negative fixture: the page carries another generation's token.
        cycle ^ 0x5a5a_0000
    } else if unarmed {
        0
    } else {
        cycle
    };
    set_token(ctx.page, page_token);
    let stored = if cfg!(feature = "test-acpi-s3-facs-end") {
        // Negative fixture: one byte at facs+length, just past the recorded
        // extent, stands in for the wake stores and must be refused.
        // SAFETY: Deliberately outside the caller contract: write_firmware
        // checks the recorded extent before any mapping or store and returns
        // Permission for this span; any other result stops the VM below.
        let refused = unsafe { paging.write_firmware(facs + length as u64, &[0]) };
        if refused != Err(kernel_core::Error::Permission) {
            serial::fail(b"S3_FACS_END_NOT_REFUSED");
        }
        serial::write(b"POWER:S3:STORE:FACS_END:REFUSED:Permission\n");
        false
    } else {
        store_facs(paging, facs, length, ctx.wake, copy)
    };
    if !stored {
        // Never leave a partially armed vector behind.
        ARMED.store(0, Ordering::Release);
        set_token(ctx.page, 0);
        if store_facs(paging, facs, length, ctx.disarm, copy) {
            return false;
        }
        serial::fail(b"S3_FACS_DISARM");
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
    set_token(ctx.page, 0);
    let disarmed = store_facs(paging, facs, length, ctx.disarm, copy);
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
    let signature_same =
        u32::from_le_bytes([copy[8], copy[9], copy[10], copy[11]]) == ctx.signature;
    let intact = canary(&ctx.canary, cycle, false);
    let cr3_same = cr3() == paging.root;
    if let Some(frames) = frames {
        paging.audit(frames);
    }
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
    true
}

/// Runs the bounded S3 cycles after M2 passed; returns only after the last
/// validated wake. A reported rejection that performed no PM1 write stops
/// the VM with `PLAN_REJECTED_EXIT`, never the M2 pass exit.
#[cfg(feature = "native-acpi-s3")]
pub fn run(frames: &Frames<FRAME_COUNT>, paging: &mut paging::Paging, prepared: Preparation) {
    let Some(ctx) = resolve(prepared) else {
        serial::exit(PLAN_REJECTED_EXIT);
    };
    for generation in 1..=CYCLES {
        if !cycle(Some(frames), paging, &ctx, generation) {
            serial::write(b"POWER:S3:REJECTED:FacsStore:DISARMED\n");
            serial::exit(PLAN_REJECTED_EXIT);
        }
    }
    serial::write(b"POWER:S3:PASS\n");
}

/// test-system-s3: the plan resolved once at bootstrap (markers printed
/// then) and whether the single permitted suspend of this boot was used.
#[cfg(feature = "test-system-s3")]
static mut SYSTEM_CTX: Option<Ctx> = None;
#[cfg(feature = "test-system-s3")]
static mut SYSTEM_USED: bool = false;

/// Bootstrap only, before M3 starts: resolve the plan for the system profile.
#[cfg(feature = "test-system-s3")]
pub fn stash(prepared: Preparation) {
    let ctx = resolve(prepared);
    // SAFETY: Single BSP, IF=0, before any process exists; written once here
    // and afterwards read only by the 0x91 handler under the KERNEL lock.
    unsafe { ptr::addr_of_mut!(SYSTEM_CTX).write(ctx) };
}

/// Whether a SUSPEND may still be granted on this boot.
#[cfg(feature = "test-system-s3")]
pub fn ready() -> bool {
    // SAFETY: See `stash`; read under the KERNEL lock with IF=0.
    unsafe { !*ptr::addr_of!(SYSTEM_USED) && (*ptr::addr_of!(SYSTEM_CTX)).is_some() }
}

/// The one granted suspend of this boot. Returns only after a validated
/// wake; every failure after the grant stops the VM (ADR item 2).
#[cfg(feature = "test-system-s3")]
pub fn suspend(paging: &mut paging::Paging) {
    // SAFETY: See `stash`; the flag is set before arming so no path can retry.
    let ctx = unsafe {
        ptr::addr_of_mut!(SYSTEM_USED).write(true);
        (*ptr::addr_of!(SYSTEM_CTX)).as_ref()
    };
    let Some(ctx) = ctx else {
        serial::fail(b"S3_PLAN_LOST")
    };
    if !cycle(None, paging, ctx, 1) {
        serial::fail(b"S3_FACS_STORE");
    }
}

static mut FACS_COPY: [u8; 4096] = [0; 4096];
