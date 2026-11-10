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
    mov rax, qword ptr [r12 + 152]
    test rax, rax
    jnz 1f
    mov rax, 0x100000000
1:
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
    mov qword ptr [r12 + 160], 1
    mov eax, 0xffff
    xor edx, edx
    int 0x80
    cmp rax, -2
    jne .Labi_error
    test rdx, rdx
    jne .Labi_error
    check_preserved 0

    // ABI-info has no arguments: a nonzero unused RSI must be EINVAL=-1.
    mov qword ptr [r12 + 160], 2
    xor eax, eax
    mov esi, 1
    int 0x80
    cmp rax, -1
    jne .Labi_error
    test rdx, rdx
    jne .Labi_error
    check_preserved 1
    xor esi, esi
    mov qword ptr [r12 + 160], 3

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
    cmp rax, 15
    je .Lexit_seven
    cmp rax, 16
    je .Linvalid_return_stack
    cmp rax, 19
    je .Lfpu_canary
    cmp rax, 20
    je .Lfpu_canary
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
    check_preserved 0
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
.Lexit_seven:
    mov edi, 7
    jmp .Lexit
.Linvalid_return_stack:
    // int80 switches to the kernel TSS stack. The kernel must reject this
    // saved return-RSP locally before iretq; no hardware fault is the verdict.
    xor eax, eax
    xor edi, edi
    xor esi, esi
    xor edx, edx
    xor r10d, r10d
    xor r8d, r8d
    xor r9d, r9d
    xor esp, esp
    int 0x80
    ud2

// K06 5d-2 (R3-3) FPU canary, native-smp-m3 only (eager FXSAVE/FXRSTOR):
// st0 = the private pattern (exact in the 64-bit significand), xmm0 = the
// pattern, xmm15 = its complement, loaded once (DATA+128) and checked on
// every loop pass. Another process's or a lost state ends with int3 (#BP).
.Lfpu_canary:
    mov qword ptr [r12 + 160], 4
    cmp rbx, 0x13579bdf
    jne .Labi_error
    cmp rbp, 0x2468ace0
    jne .Labi_error
    cmp r14, 0x11223344
    jne .Labi_error
    cmp r15, 0x55667788
    jne .Labi_error
    mov qword ptr [r12 + 160], 5
    cmp qword ptr [r12 + 128], 0
    jne 1f
    fninit
    fild qword ptr [r12 + 16]
    movq xmm0, r13
    pcmpeqd xmm15, xmm15
    pxor xmm15, xmm0
1:
    movq rax, xmm0
    cmp rax, r13
    jne .Lfpu_corrupt
    movq rax, xmm15
    not rax
    cmp rax, r13
    jne .Lfpu_corrupt
    fld st(0)
    fistp qword ptr [rsp - 8]
    cmp qword ptr [rsp - 8], r13
    jne .Lfpu_corrupt
    // Publish readiness only after initialization AND a complete successful
    // GPR/x87/SSE check. A timer may interrupt any earlier instruction.
    mov qword ptr [r12 + 128], 1
    cmp qword ptr [r12], 20
    je .Lipc
    // Only the migration fixture requests a yield after all canary checks.
    cmp qword ptr [r12 + 136], 0
    jne .Lyield
    jmp .Lloop
.Lfpu_corrupt:
    int3
    ud2

// Mode 20: real cross-process IPC with readiness waits. Private phase and
// sequence survive every syscall and CPU handoff; the kernel owns verdicts.
.Lipc:
    cmp qword ptr [r12 + 104], 0
    jne .Lloop
    cmp qword ptr [r12 + 120], 0
    jne 1f
    mov qword ptr [r12 + 120], 1
1:
    mov rax, qword ptr [r12 + 80]
    xor rax, qword ptr [r12 + 112]
    test rax, rax
    jne .Lipc_recv
.Lipc_send:
    // The kernel stamps received sender identity; a new send cannot supply it.
    mov qword ptr [0x60000000], 0
    mov rax, qword ptr [r12 + 120]
    mov qword ptr [0x60000040], rax
    mov rdi, qword ptr [r12 + 64]
    mov esi, 0x60000000
    mov edx, 320
    xor r10d, r10d
    xor r8d, r8d
    xor r9d, r9d
    mov eax, 0x20
    int 0x80
    cmp rax, -8
    je .Lipc_wait_send
    test rax, rax
    jne .Lipc_send_error
    cmp qword ptr [r12 + 80], 0
    jne .Lipc_round
    mov qword ptr [r12 + 112], 1
    jmp .Lloop
.Lipc_recv:
    mov rdi, qword ptr [r12 + 72]
    mov esi, 0x60000000
    mov edx, 320
    xor r10d, r10d
    xor r8d, r8d
    xor r9d, r9d
    mov eax, 0x21
    int 0x80
    cmp rax, -8
    je .Lipc_wait_recv
    test rax, rax
    jne .Lipc_recv_error
    mov rax, qword ptr [0x60000040]
    cmp rax, qword ptr [r12 + 120]
    jne .Lpattern_error
    cmp qword ptr [r12 + 80], 0
    je .Lipc_round
    mov qword ptr [r12 + 112], 1
    jmp .Lloop
.Lipc_round:
    add qword ptr [r12 + 96], 1
    mov qword ptr [r12 + 112], 0
    add qword ptr [r12 + 120], 1
    mov rax, qword ptr [r12 + 96]
    cmp rax, qword ptr [r12 + 88]
    jb .Lloop
    mov qword ptr [r12 + 104], 1
    jmp .Lloop
.Lipc_wait_send:
    mov rdi, qword ptr [r12 + 64]
    mov qword ptr [0x60000200], rdi
    mov qword ptr [0x60000208], 1
    jmp .Lipc_wait
.Lipc_wait_recv:
    mov rdi, qword ptr [r12 + 72]
    mov qword ptr [0x60000200], rdi
    mov qword ptr [0x60000208], 2
.Lipc_wait:
    mov eax, 3
    xor edi, edi
    xor esi, esi
    xor edx, edx
    xor r10d, r10d
    xor r8d, r8d
    xor r9d, r9d
    int 0x80
    test rax, rax
    jne .Lipc_clock_error
    add rdx, 1000
    mov edi, 0x60000200
    mov esi, 1
    mov eax, 0x22
    int 0x80
    test rax, rax
    jne .Lipc_wait_error
    jmp .Lloop
.Lipc_send_error:
    mov edi, 0xe4
    jmp .Lipc_error
.Lipc_recv_error:
    mov edi, 0xe5
    jmp .Lipc_error
.Lipc_clock_error:
    mov edi, 0xe7
    jmp .Lipc_error
.Lipc_wait_error:
    mov edi, 0xe6
.Lipc_error:
    mov qword ptr [r12 + 144], rax
    jmp .Lexit

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
