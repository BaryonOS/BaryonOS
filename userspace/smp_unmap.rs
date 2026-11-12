#![no_std]
#![no_main]

//! Dedicated ring-3 unmap control. Every process writes its unique pattern at
//! the same private VA. Only slot 1 continuously reads the revoked page.
use core::arch::global_asm;

// SAFETY: Complete executable entry using only the kernel-validated private
// DATA/HEAP mappings and the existing yield/exit ABI. The revoked HEAP read is
// an intentional #PF in the positive isolated QEMU fixture.
global_asm!(
    r#"
    .section .text.entry,"ax",@progbits
    .global _start
    .type _start,@function
_start:
    mov r12, 0x40010000
    mov r13, qword ptr [r12 + 16]
    mov r14, 0x60000000
    mov qword ptr [r14], r13
    cmp qword ptr [r12 + 24], 1
    je .Ltarget
.Lworker:
    cmp qword ptr [r14], r13
    jne .Lbad
    add qword ptr [r12 + 8], 1
    jmp .Lworker
.Ltarget:
    mov rax, qword ptr [r14]
    cmp rax, r13
    jne .Lstale
    mov qword ptr [r12 + 64], rax
    add qword ptr [r12 + 8], 1
    jmp .Ltarget
.Lstale:
    // Free/allocate zeroing can transiently be visible in the deliberately
    // broken profile. Only the new owner's exact pattern is reuse evidence.
    mov r15, 0x7272000000005f5f
    cmp rax, r15
    jne .Ltarget
    mov qword ptr [r12 + 72], rax
    // Allow the BSP to stop this CPU after observing the actual reused value.
    mov eax, 1
    xor edi, edi
    xor esi, esi
    xor edx, edx
    xor r10d, r10d
    xor r8d, r8d
    xor r9d, r9d
    int 0x80
    jmp .Lbad
.Lbad:
    mov eax, 2
    mov edi, 0xe4
    int 0x80
    ud2
    .size _start, . - _start
    .section .data.probe,"aw",@progbits
    .balign 8
    .global probe_data
    .type probe_data,@object
probe_data:
    .zero 32
    .size probe_data, . - probe_data
"#
);

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
