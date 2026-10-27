#![no_std]
#![no_main]

//! Synthetic ring-3 probe only. No port I/O, serial output, kernel markers,
//! host calls, allocator or shared user memory. The kernel owns the test verdict.

use core::arch::global_asm;

// SAFETY: This is the complete executable entry, not callable Rust code. Entry
// uses the contract's user stack/selectors. Ordinary loads/stores touch only
// this process's private data PT_LOAD. Fault-mode accesses below are deliberate
// negative tests, executed exclusively in isolated QEMU guests at CPL3. No Rust
// references are constructed from the supplied target. Interrupts/syscalls must
// preserve r12/r13; the private pattern check exercises that across scheduling.
global_asm!(r#"
    .macro check_preserved expected_rsi
    cmp r12, 0x40010000
    jne .Labi_error
    cmp r13, qword ptr [r12 + 16]
    jne .Labi_error
    cmp rbx, 0x13579bdf
    jne .Labi_error
    cmp rbp, 0x2468ace0
    jne .Labi_error
    cmp r14, 0x11223344
    jne .Labi_error
    cmp r15, 0x55667788
    jne .Labi_error
    cmp rcx, 0x0badc0de
    jne .Labi_error
    cmp r11, 0x76543210
    jne .Labi_error
    cmp rsp, 0x7fffeff8
    jne .Labi_error
    test rdi, rdi
    jne .Labi_error
    cmp rsi, \expected_rsi
    jne .Labi_error
    test r10, r10
    jne .Labi_error
    test r8, r8
    jne .Labi_error
    test r9, r9
    jne .Labi_error
    .endm

    .section .text.entry,"ax",@progbits
    .global _start
    .type _start,@function
_start:
    mov r12, 0x40010000
    mov r13, qword ptr [r12 + 16]
    mov ebx, 0x13579bdf
    mov ebp, 0x2468ace0
    mov r14d, 0x11223344
    mov r15d, 0x55667788
    mov ecx, 0x0badc0de
    mov r11d, 0x76543210

    // ABI v1: no arguments, RAX=0 and RDX=1<<32 on return.
    xor eax, eax
    xor edi, edi
    xor esi, esi
    xor edx, edx
    xor r10d, r10d
    xor r8d, r8d
    xor r9d, r9d
    std
    int 0x80
    test rax, rax
    jne .Labi_error
    mov rax, 0x100000000
    cmp rdx, rax
    jne .Labi_error
    check_preserved 0
    // Kernel entry must clear DF before Rust; sanitized return must clear it
    // again rather than restoring the deliberately hostile user flag.
    pushfq
    pop rdx
    test edx, 0x400
    cld
    jne .Labi_error

    // Unknown syscall: ENOSYS=-2 and no result/argument/register corruption.
    mov eax, 0xffff
    xor edx, edx
    int 0x80
    cmp rax, -2
    jne .Labi_error
    test rdx, rdx
    jne .Labi_error
    check_preserved 0

    // ABI-info has no arguments: a nonzero unused RSI must be EINVAL=-1.
    xor eax, eax
    mov esi, 1
    int 0x80
    cmp rax, -1
    jne .Labi_error
    test rdx, rdx
    jne .Labi_error
    check_preserved 1
    xor esi, esi

.Lloop:
    cmp qword ptr [r12 + 16], r13
    jne .Lpattern_error
    add qword ptr [r12 + 8], 1
    mov rax, qword ptr [r12]
    test rax, rax
    je .Lloop                       // 0: no-yield busy worker
    cmp rax, 1
    je .Lyield
    cmp rax, 2
    je .Lexit_after_eight
    cmp rax, 3
    je .Lread_target
    cmp rax, 4
    je .Lwrite_target
    cmp rax, 5
    je .Lwrite_target               // target supplied separately by kernel
    cmp rax, 6
    je .Lexecute_stack
    cmp rax, 7
    je .Lexecute_heap
    cmp rax, 8
    je .Llower_guard
    cmp rax, 9
    je .Lupper_guard
    cmp rax, 10
    je .Lcli
    cmp rax, 11
    je .Lx87
    cmp rax, 12
    je .Lxmm
    cmp rax, 13
    je .Lymm
    cmp rax, 14
    je .Linvalid_instruction
    mov edi, 0xe2
    jmp .Lexit

.Lyield:
    mov eax, 1
    xor edi, edi
    xor esi, esi
    xor edx, edx
    xor r10d, r10d
    xor r8d, r8d
    xor r9d, r9d
    int 0x80
    test rax, rax
    jne .Labi_error
    test rdx, rdx
    jne .Labi_error
    jmp .Lloop

.Lexit_after_eight:
    cmp qword ptr [r12 + 8], 8
    jb .Lloop
    xor edi, edi
    jmp .Lexit

.Lread_target:
    mov rax, qword ptr [r12 + 24]
    mov rax, qword ptr [rax]
    jmp .Lunexpected_return
.Lwrite_target:
    mov rax, qword ptr [r12 + 24]
    mov qword ptr [rax], 1
    jmp .Lunexpected_return
.Lexecute_stack:
    mov eax, 0x7ffef000
    jmp rax
.Lexecute_heap:
    mov eax, 0x60000000
    jmp rax
.Llower_guard:
    mov eax, 0x7ffee000
    mov qword ptr [rax], 1
    jmp .Lunexpected_return
.Lupper_guard:
    mov eax, 0x7ffff000
    mov qword ptr [rax], 1
    jmp .Lunexpected_return
.Lcli:
    cli
    jmp .Lunexpected_return
.Lx87:
    fninit
    jmp .Lunexpected_return
.Lxmm:
    pxor xmm0, xmm0
    jmp .Lunexpected_return
.Lymm:
    vxorps ymm0, ymm0, ymm0
    jmp .Lunexpected_return
.Linvalid_instruction:
    ud2
    jmp .Lunexpected_return

.Labi_error:
    cld
    mov edi, 0xe0
    jmp .Lexit
.Lpattern_error:
    mov edi, 0xe1
    jmp .Lexit
.Lunexpected_return:
    mov edi, 0xe3
.Lexit:
    mov eax, 2
    xor esi, esi
    xor edx, edx
    xor r10d, r10d
    xor r8d, r8d
    xor r9d, r9d
    int 0x80
    // A returning exit is an ABI failure; do not continue with false progress.
    ud2
    .size _start, . - _start

    .section .data.probe,"aw",@progbits
    .balign 8
    .global probe_data
    .type probe_data,@object
probe_data:
    .quad 0                         // +0: kernel-selected mode
    .quad 0                         // +8: private progress counter
    .quad 0                         // +16: kernel-selected private pattern
    .quad 0                         // +24: kernel-selected negative-test target
    .size probe_data, . - probe_data
"#);

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    // No Rust entry path calls this. Keep the required no_std handler local,
    // scalar and silent; it never produces kernel-owned success evidence.
    loop { core::hint::spin_loop(); }
}
