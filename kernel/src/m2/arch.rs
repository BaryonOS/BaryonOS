//! x86-64 entry ABI: 15 GPRs, vector, error, CPU iret frame. IF stays clear in M2.
//! Intel SDM vol.3 ch.4-7: CR0.WP, EFER.NXE, IDT/GDT/TSS and IST.
use crate::serial;
use core::{
    arch::{asm, global_asm},
    ptr,
};

#[repr(C, align(4096))]
pub struct Stack {
    lower: [u8; 4096],
    data: [u8; 16384],
    upper: [u8; 4096],
}
impl Stack {
    const fn new() -> Self {
        Self {
            lower: [0; 4096],
            data: [0; 16384],
            upper: [0; 4096],
        }
    }
}
static mut MAIN: Stack = Stack::new();
static mut DOUBLE: Stack = Stack::new();
static mut NMI: Stack = Stack::new();
pub fn stack_top() -> u64 {
    ptr::addr_of!(MAIN) as u64 + 20480
}
pub fn guard_pages() -> [u64; 6] {
    let main = ptr::addr_of!(MAIN) as u64;
    let double = ptr::addr_of!(DOUBLE) as u64;
    let nmi = ptr::addr_of!(NMI) as u64;
    [main, main + 20480, double, double + 20480, nmi, nmi + 20480]
}
pub fn is_guard(address: u64) -> bool {
    guard_pages().contains(&address)
}
pub fn double_fault_probe() -> ! {
    let guard = guard_pages()[0] + 2048;
    // SAFETY: Explicit destructive test-only terminal path. RSP is set inside an
    // unmapped MAIN guard, then an actual push faults. #PF cannot push its CPU
    // frame there, requiring #DF on the separate IST1 stack. No Rust execution
    // may resume on the invalid stack. The feature handler exits the VM.
    unsafe {
        asm!("mov rsp, {guard}","push 0","ud2",guard=in(reg) guard,options(noreturn));
    }
}
#[repr(C, packed)]
struct Pointer {
    limit: u16,
    base: u64,
}
#[repr(C, packed)]
struct Tss {
    reserved: u32,
    rsp: [u64; 3],
    reserved1: u64,
    ist: [u64; 7],
    reserved2: u64,
    reserved3: u16,
    io: u16,
}
static mut TSS: Tss = Tss {
    reserved: 0,
    rsp: [0; 3],
    reserved1: 0,
    ist: [0; 7],
    reserved2: 0,
    reserved3: 0,
    io: 104,
};
static mut GDT: [u64; 7] = [0; 7];
#[derive(Clone, Copy)]
#[repr(C, packed)]
struct Gate {
    lo: u16,
    cs: u16,
    ist: u8,
    flags: u8,
    mid: u16,
    hi: u32,
    reserved: u32,
}
const EMPTY: Gate = Gate {
    lo: 0,
    cs: 0,
    ist: 0,
    flags: 0,
    mid: 0,
    hi: 0,
    reserved: 0,
};
static mut IDT: [Gate; 256] = [EMPTY; 256];
extern "C" {
    static exception_addresses: [u64; 32];
}

pub fn install() {
    // SAFETY: Exclusive single-CPU bootstrap, IF=0, static tables are aligned and
    // live forever. Packed TSS fields use unaligned raw writes. All gate addresses
    // originate in our executable stubs. No user/GDT writes after initialization.
    unsafe {
        ptr::addr_of_mut!(TSS.rsp)
            .cast::<u64>()
            .write_unaligned(stack_top());
        ptr::addr_of_mut!(TSS.ist)
            .cast::<u64>()
            .write_unaligned(ptr::addr_of!(DOUBLE) as u64 + 20480);
        ptr::addr_of_mut!(TSS.ist)
            .cast::<u64>()
            .add(1)
            .write_unaligned(ptr::addr_of!(NMI) as u64 + 20480);
        let base = ptr::addr_of!(TSS) as u64;
        let low = 103 | ((base & 0xffffff) << 16) | (0x89 << 40) | (((base >> 24) & 0xff) << 56);
        ptr::addr_of_mut!(GDT).write([
            0,
            0x00af9a000000ffff,
            0x00cf92000000ffff,
            0x00cff2000000ffff,
            0x00affa000000ffff,
            low,
            base >> 32,
        ]);
        for i in 0..32 {
            let a = ptr::addr_of!(exception_addresses)
                .cast::<u64>()
                .add(i)
                .read();
            let gate = Gate {
                lo: a as u16,
                cs: 8,
                ist: if i == 8 {
                    1
                } else if i == 2 {
                    2
                } else {
                    0
                },
                flags: 0x8e,
                mid: (a >> 16) as u16,
                hi: (a >> 32) as u32,
                reserved: 0,
            };
            ptr::addr_of_mut!(IDT).cast::<Gate>().add(i).write(gate);
        }
        let gdtr = Pointer {
            limit: 55,
            base: ptr::addr_of!(GDT) as u64,
        };
        let idtr = Pointer {
            limit: 4095,
            base: ptr::addr_of!(IDT) as u64,
        };
        asm!("lgdt [{}]",in(reg) &gdtr,options(readonly,nostack));
        asm!("push 8","lea rax, [rip + 2f]","push rax","retfq","2:","mov ax, 16","mov ds, ax","mov es, ax","mov ss, ax","xor eax,eax","mov fs, ax","mov gs, ax",out("rax") _);
        asm!("ltr ax",in("ax") 0x28u16,options(nostack));
        asm!("lidt [{}]",in(reg) &idtr,options(readonly,nostack));
        // Clear FS/GS/KERNEL_GS base; user code cannot set FSGSBASE in this profile.
        for msr in [0xc0000100u32, 0xc0000101, 0xc0000102] {
            asm!("wrmsr",in("ecx") msr,in("eax") 0u32,in("edx") 0u32,options(nostack));
        }
    }
}

/// One AP's private descriptor tables: a GDT with the kernel's code/data
/// descriptors and this CPU's own TSS descriptor, plus that TSS. A busy TSS
/// may be loaded by exactly one CPU, so every AP needs its own. 256-byte
/// alignment keeps the 160-byte payload (56-byte GDT + 104-byte packed TSS,
/// padded to 256 bytes) inside one page.
#[cfg(feature = "native-smp")]
#[repr(C, align(256))]
pub struct CpuTables {
    gdt: [u64; 7],
    tss: Tss,
}
#[cfg(feature = "native-smp")]
const _: () = assert!(
    size_of::<Tss>() == 104
        && core::mem::offset_of!(CpuTables, tss) == 56
        && size_of::<CpuTables>() == 256
        && core::mem::align_of::<CpuTables>() == 256
);
#[cfg(feature = "native-smp")]
impl CpuTables {
    pub const fn new() -> Self {
        Self {
            gdt: [0; 7],
            tss: Tss {
                reserved: 0,
                rsp: [0; 3],
                reserved1: 0,
                ist: [0; 7],
                reserved2: 0,
                reserved3: 0,
                io: 104,
            },
        }
    }
}

/// AP half of `install`: own GDT/TSS (RSP0, IST1=#DF, IST2=NMI exactly as the
/// BSP's gates expect) and the shared permanent IDT. Called once per AP on
/// its own stack with IF=0; `tables` is that AP's exclusive static slot.
#[cfg(feature = "native-smp")]
pub fn install_ap(tables: *mut CpuTables, rsp0: u64, double_top: u64, nmi_top: u64) {
    // SAFETY: `tables` is a permanent, 256-byte aligned static slot owned by this
    // AP alone (selected by the claim CAS); no other CPU reads or writes it. The
    // BSP's GDT descriptors are read-only after install(). Packed TSS fields use
    // unaligned raw writes. The IDT is permanent and complete before any AP runs.
    unsafe {
        let tss = ptr::addr_of_mut!((*tables).tss);
        ptr::addr_of_mut!((*tss).rsp)
            .cast::<u64>()
            .write_unaligned(rsp0);
        let ist = ptr::addr_of_mut!((*tss).ist).cast::<u64>();
        ist.write_unaligned(double_top);
        ist.add(1).write_unaligned(nmi_top);
        let base = tss as u64;
        let low = 103 | ((base & 0xffffff) << 16) | (0x89 << 40) | (((base >> 24) & 0xff) << 56);
        let kernel = ptr::addr_of!(GDT).cast::<u64>();
        let gdt = ptr::addr_of_mut!((*tables).gdt).cast::<u64>();
        for i in 0..5 {
            gdt.add(i).write(kernel.add(i).read());
        }
        gdt.add(5).write(low);
        gdt.add(6).write(base >> 32);
        let gdtr = Pointer {
            limit: 55,
            base: gdt as u64,
        };
        let idtr = Pointer {
            limit: 4095,
            base: ptr::addr_of!(IDT) as u64,
        };
        asm!("lgdt [{}]",in(reg) &gdtr,options(readonly,nostack));
        asm!("push 8","lea rax, [rip + 2f]","push rax","retfq","2:","mov ax, 16","mov ds, ax","mov es, ax","mov ss, ax","xor eax,eax","mov fs, ax","mov gs, ax",out("rax") _);
        asm!("ltr ax",in("ax") 0x28u16,options(nostack));
        asm!("lidt [{}]",in(reg) &idtr,options(readonly,nostack));
        for msr in [0xc0000100u32, 0xc0000101, 0xc0000102] {
            asm!("wrmsr",in("ecx") msr,in("eax") 0u32,in("edx") 0u32,options(nostack));
        }
    }
}

/// BSP-only, before any AP starts: supervisor interrupt gate for the SMP IPI
/// (0xf0) or spurious (0xff) vector. Never a user-callable gate.
#[cfg(feature = "native-smp")]
pub fn smp_gate(vector: usize, address: u64) {
    assert!(matches!(vector, 0xe0 | 0xf0 | 0xff));
    let gate = Gate {
        lo: address as u16,
        cs: 8,
        ist: 0,
        flags: 0x8e,
        mid: (address >> 16) as u16,
        hi: (address >> 32) as u32,
        reserved: 0,
    };
    // SAFETY: Exclusive IF=0 BSP initialization of the permanent IDT before any
    // AP has loaded it; the address is our own executable stub.
    unsafe {
        ptr::addr_of_mut!(IDT)
            .cast::<Gate>()
            .add(vector)
            .write(gate);
    }
}

pub fn protection() {
    // SAFETY: CPUID has no memory effects; own scalar-only kernel target. All CR
    // and MSR operations execute at CPL0 with IF clear. NX checked before NXE.
    unsafe {
        let ext = core::arch::x86_64::__cpuid(0x80000000);
        if ext.eax < 0x80000001 || core::arch::x86_64::__cpuid(0x80000001).edx & (1 << 20) == 0 {
            serial::fail(b"M2_NX_UNAVAILABLE");
        }
        let (mut lo, hi): (u32, u32);
        asm!("rdmsr",in("ecx") 0xc0000080u32,out("eax") lo,out("edx") hi,options(nostack));
        lo |= 1 << 11;
        #[cfg(feature = "fpu-context")]
        {
            lo &= !(1 << 14);
        } // AMD EFER.FFXSR must not skip XMM at CPL0.
        asm!("wrmsr",in("ecx") 0xc0000080u32,in("eax") lo,in("edx") hi,options(nostack));
        #[cfg(feature = "fpu-context")]
        {
            let actual: u32;
            asm!("rdmsr",in("ecx") 0xc0000080u32,out("eax") actual,out("edx") _,options(nostack));
            assert_eq!(actual & (1 << 14), 0);
        }
        let mut cr0: u64;
        let mut cr4: u64;
        asm!("mov {}, cr0",out(reg) cr0,options(nomem,nostack));
        asm!("mov {}, cr4",out(reg) cr4,options(nomem,nostack));
        cr0 |= (1 << 16) | (1 << 1) | (1 << 2) | (1 << 3);
        #[cfg(feature = "fpu-context")]
        {
            // Intel SDM CPUID.01H:EDX: FPU, FXSR, SSE and SSE2. Check before
            // touching OSFXSR or executing an FXSAVE-family instruction.
            let required = (1 << 0) | (1 << 24) | (1 << 25) | (1 << 26);
            if core::arch::x86_64::__cpuid(1).edx & required != required {
                serial::fail(b"M3_FPU_UNAVAILABLE");
            }
            cr0 = (cr0 | (1 << 5)) & !((1 << 2) | (1 << 3));
        }
        cr4 &= !((1 << 7) | (1 << 9) | (1 << 10) | (1 << 16) | (1 << 17) | (1 << 18));
        #[cfg(feature = "fpu-context")]
        {
            // Eager context switching: no lazy TS trap, no XSAVE/AVX state.
            cr4 |= (1 << 9) | (1 << 10);
        }
        asm!("mov cr4, {}",in(reg) cr4,options(nostack));
        asm!("mov cr0, {}",in(reg) cr0,options(nostack));
    }
}
pub fn activate(root: u64) {
    // SAFETY: paging builder validates/retains own aligned tables and maps current
    // code, stacks, GDT, IDT, TSS and all metadata before this serializing switch.
    unsafe {
        asm!("mov cr3, {}",in(reg) root,options(nostack));
    }
    let actual: u64;
    // SAFETY: Read-only CPL0 register query; verifies real switch, not metadata.
    unsafe {
        asm!("mov {}, cr3",out(reg) actual,options(nomem,nostack));
    }
    assert_eq!(actual, root);
}
#[cfg(feature = "milestone-m3")]
pub fn service_stack() -> u64 {
    stack_top()
}
#[cfg(feature = "milestone-m3")]
pub fn process_stack(top: u64) {
    // SAFETY: Single-CPU IF=0; caller supplies mapped supervisor stack with guards.
    unsafe {
        ptr::addr_of_mut!(TSS.rsp)
            .cast::<u64>()
            .write_unaligned(top);
    }
}
#[cfg(feature = "milestone-m3")]
pub fn reset_debug() {
    // SAFETY: CPL0, IF=0. No debugger API exists. Disable all breakpoints first,
    // then clear addresses and status; users cannot write these registers.
    unsafe {
        asm!("mov dr7, {control}","mov dr0, {zero}","mov dr1, {zero}","mov dr2, {zero}","mov dr3, {zero}","mov dr6, {status}",control=in(reg) 0x400u64,zero=in(reg) 0u64,status=in(reg) 0xffff0ff0u64,options(nostack));
    }
}
#[cfg(feature = "milestone-m3")]
pub fn user_gate(vector: usize, address: u64, user: bool) {
    assert!(vector == 32 || vector == 128);
    let gate = Gate {
        lo: address as u16,
        cs: 8,
        ist: 0,
        flags: if user { 0xee } else { 0x8e },
        mid: (address >> 16) as u16,
        hi: (address >> 32) as u32,
        reserved: 0,
    };
    // SAFETY: Exclusive IF=0 initialization of own permanent IDT, valid text stub.
    unsafe {
        ptr::addr_of_mut!(IDT)
            .cast::<Gate>()
            .add(vector)
            .write(gate);
    }
}

#[no_mangle]
extern "C" fn m2_exception(frame: *const u64) -> ! {
    // SAFETY: Stub saves 15 GPRs and normalizes vector/error before CPU frame;
    // it passes its live, aligned kernel/IST stack and never returns. No scheduler
    // or allocator mutation, including NMI/DF paths. Only fixed metadata read.
    let (vector, error, rip, cs) = unsafe {
        (
            frame.add(15).read(),
            frame.add(16).read(),
            frame.add(17).read(),
            frame.add(18).read(),
        )
    };
    let cr2: u64;
    // SAFETY: CPL0 read of fault address; no pointed-to data is followed/logged.
    unsafe {
        asm!("mov {}, cr2",out(reg) cr2,options(nomem,nostack));
    }
    // Test-only AP IST probes: accepted solely on that AP's own IST stack.
    #[cfg(feature = "native-smp")]
    if let Some(marker) = super::smp::ist_fault_marker(vector, error, cs, frame) {
        serial::write(marker);
        serial::exit(0x10);
    }
    if cfg!(feature = "test-m2-double-fault") && vector == 8 && error == 0 && cs == 8 {
        let double = ptr::addr_of!(DOUBLE) as u64;
        let main = ptr::addr_of!(MAIN) as u64;
        let rsp: u64;
        // SAFETY: CPL0 metadata query on the current handler stack; CPU supplied
        // the 64-bit IST frame, whose saved old RSP is at the normalized offset.
        let old_rsp = unsafe {
            asm!("mov {}, rsp",out(reg) rsp,options(nomem,nostack));
            frame.add(20).read()
        };
        let start = frame as u64;
        if rsp >= double + 4096
            && rsp < double + 20480
            && start >= double + 4096
            && start
                .checked_add(22 * 8)
                .is_some_and(|end| end <= double + 20480)
            && old_rsp >= main
            && old_rsp < main + 4096
        {
            serial::write(b"M2:FAULT:DF_IST:PASS\n");
            serial::exit(0x11);
        }
    }
    let supervisor = cs == 8 && error & 4 == 0;
    if cfg!(feature = "test-m2-kernel-fault") && vector == 6 && error == 0 && cs == 8 {
        serial::write(b"M2:FAULT:KERNEL_UD:PASS\n");
    }
    if vector == 14 && supervisor {
        if cfg!(feature = "test-m2-ro") && cr2 == super::text_start() && error & 0x1f == 3 {
            serial::write(b"M2:FAULT:RO:PASS\n");
            serial::exit(0x10);
        }
        if cfg!(feature = "test-m2-nx")
            && cr2 == super::nx_probe()
            && rip == cr2
            && error & 0x1f == 17
        {
            serial::write(b"M2:FAULT:NX:PASS\n");
            serial::exit(0x10);
        }
        if cfg!(feature = "test-m2-absent") && cr2 == 0 && error & 0x1f == 0 {
            serial::write(b"M2:FAULT:ABSENT:PASS\n");
            serial::exit(0x10);
        }
    }
    serial::write(b"M2:FAIL:KERNEL_EXCEPTION\n");
    serial::exit(0x11)
}

// All GPRs are saved before entering Rust. The call stack is SysV aligned and DF
// cleared. No floating point context is used. Non-returning M2 failure path.
global_asm!(
    r#"
.section .text
.macro EX n, err=0
.global m2_exc_\n
m2_exc_\n:
.if \err == 0
 push 0
.endif
 push \n
 jmp m2_exception_common
.endm
EX 0
EX 1
EX 2
EX 3
EX 4
EX 5
EX 6
EX 7
EX 8,1
EX 9
EX 10,1
EX 11,1
EX 12,1
EX 13,1
EX 14,1
EX 15
EX 16
EX 17,1
EX 18
EX 19
EX 20
EX 21,1
EX 22
EX 23
EX 24
EX 25
EX 26
EX 27
EX 28
EX 29,1
EX 30,1
EX 31
m2_exception_common:
 cld
 push rax
 push rcx
 push rdx
 push rbx
 push rbp
 push rsi
 push rdi
 push r8
 push r9
 push r10
 push r11
 push r12
 push r13
 push r14
 push r15
 jmp m2_dispatch
.section .rodata
.align 8
.global exception_addresses
exception_addresses:
.quad m2_exc_0,m2_exc_1,m2_exc_2,m2_exc_3,m2_exc_4,m2_exc_5,m2_exc_6,m2_exc_7
.quad m2_exc_8,m2_exc_9,m2_exc_10,m2_exc_11,m2_exc_12,m2_exc_13,m2_exc_14,m2_exc_15
.quad m2_exc_16,m2_exc_17,m2_exc_18,m2_exc_19,m2_exc_20,m2_exc_21,m2_exc_22,m2_exc_23
.quad m2_exc_24,m2_exc_25,m2_exc_26,m2_exc_27,m2_exc_28,m2_exc_29,m2_exc_30,m2_exc_31
"#
);

#[cfg(not(feature = "milestone-m3"))]
global_asm!(
    r#"
.section .text
m2_dispatch:
 mov rdi,rsp
 and rsp,-16
 call m2_exception
 ud2
"#
);
#[cfg(feature = "milestone-m3")]
global_asm!(
    r#"
.section .text
.global m3_timer
m3_timer:
 push 0
 push 32
 jmp m2_exception_common
.global m3_syscall
m3_syscall:
 push 0
 push 128
 jmp m2_exception_common
m2_dispatch:
 cmp qword ptr [rsp+120],2
 je m3_terminal
 cmp qword ptr [rsp+120],8
 je m3_terminal
 cmp qword ptr [rsp+120],32
 jne m3_user_entry
 cmp qword ptr [rsp+144],8
 jne m3_user_entry
 cmp byte ptr [rip+m3_idle_active],1
 jne m3_terminal
 inc qword ptr [rip+m3_idle_wakeups]
 mov al,0x20
 out 0x20,al
 mov rax,rsp
 jmp m3_restore
m3_user_entry:
 mov rax,[rsp+144]
 and eax,3
 cmp eax,3
 jne m3_terminal
 .if {fpu_context}
 // GPRs already saved. The current owned aligned image is supervisor-only
 // mapped in every user CR3. Save before any Rust or another process runs.
 mov rax,[rip+m3_fpu_current]
 fxsave64 [rax]
 inc qword ptr [rip+m3_fpu_saves]
 .endif
 mov rdi,rsp
 mov rsp,[rip+m3_service_stack]
 and rsp,-16
 call m3_event
.global m3_restore
m3_restore:
 mov rsp,rax
 .if {fpu_context}
 cmp qword ptr [rsp+144],0x23
 jne 9f
 // Final restore occurs after ALL Rust, under the selected user root. These
 // pages are supervisor-only and permanent; only GPR pops/iret follow it.
 mov rax,[rip+m3_fpu_current]
 fxrstor64 [rax]
 inc qword ptr [rip+m3_fpu_restores]
9:
 .endif
 pop r15
 pop r14
 pop r13
 pop r12
 pop r11
 pop r10
 pop r9
 pop r8
 pop rdi
 pop rsi
 pop rbp
 pop rbx
 pop rdx
 pop rcx
 pop rax
 add rsp,16
 iretq
m3_terminal:
 mov rdi,rsp
 and rsp,-16
 call m2_exception
 ud2
"#,
    fpu_context = const cfg!(feature = "fpu-context") as u8,
);
