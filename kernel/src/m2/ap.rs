use core::{arch::global_asm, ptr};

use super::smp::BootstrapData;

const GUARD_SIZE: usize = 4096;
const STACK_SIZE: usize = 16 * 1024;
const MAX_CPUS: usize = boot_protocol::smp::MAX_CPUS;
const AP_CR0_SET: u32 = if cfg!(feature = "fpu-context") {
    (1 << 31) | (1 << 16) | (1 << 5) | (1 << 1)
} else {
    (1 << 31) | (1 << 16) | (1 << 1) | (1 << 2) | (1 << 3)
};
const AP_CR0_CLEAR: u32 = if cfg!(feature = "fpu-context") {
    (1 << 2) | (1 << 3)
} else {
    0
};
const AP_CR0_PROTECTION_MASK: u32 = (1 << 16) | (1 << 2) | (1 << 3);
const AP_CR0_PROTECTION_EXPECTED: u32 = if cfg!(feature = "fpu-context") {
    1 << 16
} else {
    (1 << 16) | (1 << 2) | (1 << 3)
};

#[repr(C, align(4096))]
struct Stack {
    _lower_guard: [u8; GUARD_SIZE],
    _data: [u8; STACK_SIZE],
    _upper_guard: [u8; GUARD_SIZE],
}

impl Stack {
    const fn new() -> Self {
        Self {
            _lower_guard: [0; GUARD_SIZE],
            _data: [0; STACK_SIZE],
            _upper_guard: [0; GUARD_SIZE],
        }
    }
}

static mut STACKS: [Stack; MAX_CPUS] = [Stack::new(), Stack::new(), Stack::new(), Stack::new()];
static mut BOOT_DATA: BootstrapData = BootstrapData::EMPTY;

pub fn stack_top(index: usize) -> u64 {
    stack_bounds(index).1
}

pub fn stack_bounds(index: usize) -> (u64, u64) {
    assert!(index < MAX_CPUS);
    // SAFETY: Forms the end address of a permanent, page-aligned per-CPU stack.
    let stack = unsafe { ptr::addr_of!(STACKS).cast::<Stack>().add(index) as u64 };
    let start = stack + GUARD_SIZE as u64;
    (start, start + STACK_SIZE as u64)
}

pub fn is_guard(address: u64) -> bool {
    (0..MAX_CPUS).any(|index| {
        let (start, top) = stack_bounds(index);
        address == start - GUARD_SIZE as u64 || address == top
    })
}

pub fn initialize(ids: &[u32], root: u64) {
    assert!(!ids.is_empty() && ids.len() <= MAX_CPUS);
    assert!(root != 0 && root <= u32::MAX as u64);
    let mut data = BootstrapData::EMPTY;
    for (index, id) in ids.iter().copied().enumerate() {
        assert!(
            id <= u8::MAX as u32,
            "xAPIC startup supports 8-bit APIC IDs"
        );
        data.apic_ids[index] = id;
        data.stack_tops[index] = stack_top(index);
    }
    data.cr3 = root;
    // SAFETY: Initialized once by the BSP before any AP is released; APs read
    // this immutable data after their trampoline has selected a private stack.
    unsafe {
        ptr::addr_of_mut!(BOOT_DATA).write(data);
    }
    assert_eq!(cpu_count(), ids.len());
}

pub fn cpu_id(index: usize) -> Option<u32> {
    if index >= MAX_CPUS {
        return None;
    }
    // SAFETY: BOOT_DATA is initialized before AP startup and never mutated after.
    let data = unsafe { ptr::addr_of!(BOOT_DATA).read() };
    (index < MAX_CPUS && data.apic_ids[index] != u32::MAX).then_some(data.apic_ids[index])
}

pub fn cpu_count() -> usize {
    // SAFETY: BOOT_DATA is initialized before AP startup and never mutated after.
    let data = unsafe { ptr::addr_of!(BOOT_DATA).read() };
    data.apic_ids
        .iter()
        .take_while(|id| **id != u32::MAX)
        .count()
}

pub fn copy_trampoline(page: u64, entry: u64) {
    assert!(page != 0 && page < boot_protocol::smp::SIPI_LIMIT);
    let start = ptr::addr_of!(__ap_trampoline_start);
    let end = ptr::addr_of!(__ap_trampoline_end);
    let length = end as usize - start as usize;
    assert!(
        length > 0 && length <= 4096,
        "AP trampoline must fit one page"
    );

    // SAFETY: Loader reserves an aligned SIPI page. The source is immutable,
    // linked kernel text; the destination is exclusively owned loader memory.
    unsafe {
        ptr::copy_nonoverlapping(start, page as *mut u8, length);
        write_u32(
            page,
            offset(ptr::addr_of!(__ap_trampoline_boot_data), start),
            ptr::addr_of!(BOOT_DATA) as u64,
        );
        write_u32(
            page,
            offset(ptr::addr_of!(__ap_trampoline_boot_data_operand), start),
            page + offset(ptr::addr_of!(__ap_trampoline_boot_data), start) as u64,
        );
        write_u32(
            page,
            offset(ptr::addr_of!(__ap_trampoline_gdtr_base), start),
            page + offset(ptr::addr_of!(__ap_trampoline_gdt), start) as u64,
        );
        write_u32(
            page,
            offset(ptr::addr_of!(__ap_trampoline_pm_far), start) + 2,
            page + offset(ptr::addr_of!(__ap_trampoline_pm), start) as u64,
        );
        write_u32(
            page,
            offset(ptr::addr_of!(__ap_trampoline_lm_far), start) + 1,
            page + offset(ptr::addr_of!(__ap_trampoline_lm), start) as u64,
        );
        write_u64(
            page,
            offset(ptr::addr_of!(__ap_trampoline_entry), start),
            entry,
        );
    }
}

fn offset(symbol: *const u8, start: *const u8) -> usize {
    let offset = symbol as usize - start as usize;
    assert!(offset < 4096);
    offset
}

unsafe fn write_u32(page: u64, offset: usize, value: u64) {
    assert!(value <= u32::MAX as u64 && offset.checked_add(4).is_some_and(|end| end <= 4096));
    // SAFETY: Caller owns the trampoline page before CR3 activation and supplies
    // an offset within the bounded template; unaligned patch fields are allowed.
    unsafe {
        (page as *mut u8)
            .add(offset)
            .cast::<u32>()
            .write_unaligned(value as u32);
    }
}

unsafe fn write_u64(page: u64, offset: usize, value: u64) {
    assert!(offset.checked_add(8).is_some_and(|end| end <= 4096));
    // SAFETY: Same exclusive page and template-offset contract as write_u32.
    unsafe {
        (page as *mut u8)
            .add(offset)
            .cast::<u64>()
            .write_unaligned(value);
    }
}

extern "C" {
    static __ap_trampoline_start: u8;
    static __ap_trampoline_end: u8;
    static __ap_trampoline_gdt: u8;
    static __ap_trampoline_gdtr_base: u8;
    static __ap_trampoline_pm: u8;
    static __ap_trampoline_pm_far: u8;
    static __ap_trampoline_lm: u8;
    static __ap_trampoline_lm_far: u8;
    static __ap_trampoline_boot_data: u8;
    static __ap_trampoline_boot_data_operand: u8;
    static __ap_trampoline_entry: u8;
}

// AP page walks must honor the NX leaves built by the shared kernel mapper.
global_asm!(
    r#"
.section .text.ap_trampoline,"ax"
.code16
.global __ap_trampoline_start
__ap_trampoline_start:
    cli
    cld
    mov ax, cs
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x0ff0
    lgdt [__ap_trampoline_gdtr_offset]
    mov eax, cr0
    or eax, 1
    mov cr0, eax
.global __ap_trampoline_pm_far
__ap_trampoline_pm_far:
    .byte 0x66, 0xea
    .long 0
    .word 0x08

.code32
.global __ap_trampoline_pm
__ap_trampoline_pm:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    .byte 0x8b, 0x2d
.global __ap_trampoline_boot_data_operand
__ap_trampoline_boot_data_operand:
    .long 0
    mov eax, 1
    cpuid
    shr ebx, 24
    xor ecx, ecx
1:
    cmp ecx, 4
    jae 4f
    cmp ebx, dword ptr [ebp + {apic_ids_offset} + ecx * 4]
    je 2f
    inc ecx
    jmp 1b
2:
    mov esi, ecx
    mov esp, dword ptr [ebp + {stack_tops_offset} + ecx * 8]
    and esp, 0xfffffff0
    mov eax, dword ptr [ebp + {cr3_offset}]
    mov eax, cr4
    or eax, 0x20
    mov cr4, eax
    mov eax, dword ptr [ebp + {cr3_offset}]
    mov cr3, eax
    mov ecx, 0xc0000080
    rdmsr
    or eax, 0x900
    wrmsr
    mov eax, cr0
    and eax, {ap_cr0_clear}
    or eax, {ap_cr0_set}
    mov cr0, eax
.global __ap_trampoline_lm_far
__ap_trampoline_lm_far:
    .byte 0xea
    .long 0
    .word 0x18
4:
    cli
    hlt
    jmp 4b

.code64
.global __ap_trampoline_lm
__ap_trampoline_lm:
    mov eax, cr0
    and eax, {ap_cr0_protection_mask}
    cmp eax, {ap_cr0_protection_expected}
    jne 6f
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov edi, ebx
    mov esi, esi
    mov rax, qword ptr [rip + __ap_trampoline_entry]
    call rax
5:
    cli
    hlt
    jmp 5b
6:
    cli
    hlt
    jmp 6b

.align 8
.global __ap_trampoline_gdt
__ap_trampoline_gdt:
    .quad 0
    .quad 0x00cf9a000000ffff
    .quad 0x00cf92000000ffff
    .quad 0x00af9a000000ffff
__ap_trampoline_gdtr:
    .word 31
.global __ap_trampoline_gdtr_base
__ap_trampoline_gdtr_base:
    .long 0
.global __ap_trampoline_boot_data
__ap_trampoline_boot_data:
    .long 0
.global __ap_trampoline_entry
__ap_trampoline_entry:
    .quad 0
.global __ap_trampoline_end
__ap_trampoline_end:
.set __ap_trampoline_gdtr_offset, __ap_trampoline_gdtr - __ap_trampoline_start
"#,
    apic_ids_offset = const core::mem::offset_of!(BootstrapData, apic_ids),
    stack_tops_offset = const core::mem::offset_of!(BootstrapData, stack_tops),
    cr3_offset = const core::mem::offset_of!(BootstrapData, cr3),
    ap_cr0_clear = const !AP_CR0_CLEAR,
    ap_cr0_set = const AP_CR0_SET,
    ap_cr0_protection_mask = const AP_CR0_PROTECTION_MASK,
    ap_cr0_protection_expected = const AP_CR0_PROTECTION_EXPECTED,
);
