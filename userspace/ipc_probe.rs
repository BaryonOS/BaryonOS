#![no_std]
#![no_main]

//! Bounded synthetic IPC fixture, independent of the M3 probe. The kernel
//! manifest supplies script rows and initial local capabilities. Only actual
//! int80 results or reads from private received bytes populate result slots.

use core::arch::global_asm;

// SAFETY: Complete CPL3 entry and bounded test interpreter. All internal memory
// operations check their fixed private ranges before access; no Rust references
// are created. Arbitrary syscall arguments remain integers for the kernel to
// validate. No shared user frames, I/O, host calls or kernel verdict markers.
// TSS/kernel stacks provide the int80 boundary; the ordinary user stack is used
// only for local helper return addresses. Deliberate ud2 is an explicit fixture
// operation for receiver-fault lifecycle tests, never an automatic success.
global_asm!(r#"
    .macro resolve dest, offset, bit
    mov \dest, qword ptr [rbp + \offset]
    bt rbx, \bit
    jnc .Lresolved\@
    cmp \dest, 32
    jae .Lfail
    shl \dest, 4
    mov \dest, qword ptr [r12 + \dest + 0xc08]
.Lresolved\@:
    .endm

    .section .text.entry,"ax",@progbits
    .global _start
    .type _start,@function
_start:
    cld
    mov r12d, 0x40010000
    mov r13, qword ptr [r12 + 16]
    xor r14d, r14d
    mov qword ptr [r12 + 48], 0
    mov qword ptr [r12 + 112], 0
    cmp qword ptr [r12 + 88], 32
    ja .Lfail
    xor eax, eax
    xor edi, edi
    xor esi, esi
    xor edx, edx
    xor r10d, r10d
    xor r8d, r8d
    xor r9d, r9d
    add qword ptr [r12 + 8], 1
    int 0x80
    test rax, rax
    jne .Lfail
    mov rcx, 0x100000000
    cmp rdx, rcx
    jne .Lfail
    cmp qword ptr [r12], 0
    je .Lworker

.Lrow:
    cmp r13, qword ptr [r12 + 16]
    jne .Lfail
    cmp r14, qword ptr [r12 + 88]
    jae .Lcomplete
    lea rbp, [r14 + r14*4]
    shl rbp, 4
    add rbp, 0x40010100
    mov rbx, qword ptr [rbp + 72]
    mov rax, 0x0000ffff000001ff
    not rax
    test rax, rbx
    jne .Lfail
    mov r15, rbx
    shr r15, 32
    and r15d, 0xffff
    cmp r15, 1024
    ja .Lfail
    test r15, r15
    jne .Lattempt
    mov r15d, 1

.Lattempt:
    add qword ptr [r12 + 112], 1
    cmp qword ptr [r12 + 112], 32768
    ja .Lfail
    resolve rdi, 8, 0
    resolve rsi, 16, 1
    resolve rdx, 24, 2
    resolve r10, 32, 3
    resolve r8, 40, 4
    resolve r9, 48, 5
    mov rax, qword ptr [rbp]
    cmp rax, 0x100
    jb .Lsyscall
    je .Lwrite
    cmp rax, 0x101
    je .Lfill
    cmp rax, 0x102
    je .Lassert
    cmp rax, 0x103
    je .Lload
    cmp rax, 0x104
    je .Lwait
    cmp rax, 0x105
    je .Lcrash
    jmp .Lfail

.Lsyscall:
    add qword ptr [r12 + 8], 1
    int 0x80
    jmp .Lrecord
.Lwrite:
    mov ecx, 8
    call .Lwritable
    mov qword ptr [rdi], rsi
    jmp .Lzero_result
.Lfill:
    cmp rsi, 255
    ja .Lfail
    cmp rdx, 4096
    ja .Lfail
    mov rcx, rdx
    call .Lwritable
    mov rax, rsi
    cld
    rep stosb
    jmp .Lzero_result
.Lassert:
    mov ecx, 8
    call .Lreadable
    cmp qword ptr [rdi], rsi
    jne .Lfail
    jmp .Lzero_result
.Lload:
    mov ecx, 8
    call .Lreadable
    mov rdx, qword ptr [rdi]
    xor eax, eax
    jmp .Lrecord
.Lwait:
    mov ecx, 8
    call .Lreadable
    cmp qword ptr [rdi], rsi
    je .Lzero_result
    call .Lyield
    jmp .Lattempt
.Lcrash:
    add qword ptr [r12 + 8], 1
    ud2
.Lzero_result:
    xor eax, eax
    xor edx, edx
.Lrecord:
    mov rcx, r14
    shl rcx, 4
    mov qword ptr [r12 + rcx + 0xc00], rax
    mov qword ptr [r12 + rcx + 0xc08], rdx
    mov qword ptr [r12 + 80], rax
    bt rbx, 8
    jnc .Lcompare
    cmp rax, -8
    jne .Lcompare
    call .Lyield
    jmp .Lattempt
.Lcompare:
    cmp rax, qword ptr [rbp + 56]
    jne .Lfail
    bt rbx, 6
    jc .Lmatched
    mov rcx, qword ptr [rbp + 64]
    bt rbx, 7
    jnc .Lexpected_ready
    cmp rcx, 32
    jae .Lfail
    shl rcx, 4
    mov rcx, qword ptr [r12 + rcx + 0xc08]
.Lexpected_ready:
    cmp rdx, rcx
    jne .Lfail
.Lmatched:
    dec r15
    jnz .Lattempt
    inc r14
    mov qword ptr [r12 + 40], r14
    jmp .Lrow

.Lcomplete:
    mov qword ptr [r12 + 48], 1
.Lworker:
    cmp r13, qword ptr [r12 + 16]
    jne .Lfail
    call .Lyield
    jmp .Lworker

.Lyield:
    mov eax, 1
    xor edi, edi
    xor esi, esi
    xor edx, edx
    xor r10d, r10d
    xor r8d, r8d
    xor r9d, r9d
    add qword ptr [r12 + 8], 1
    int 0x80
    test rax, rax
    jne .Lfail
    test rdx, rdx
    jne .Lfail
    ret

// Internal writes can touch result slots or the two RW buffer pages. They
// cannot alter script rows, mode, status, barriers or the read-only page.
.Lwritable:
    mov rax, rdi
    add rax, rcx
    jc .Lfail
    cmp rdi, 0x40010c00
    jb .Lfail
    cmp rdi, 0x40010e00
    jae .Lbuffer_write
    cmp rax, 0x40010e00
    ja .Lfail
    ret
.Lbuffer_write:
    cmp rdi, 0x40011000
    jb .Lfail
    cmp rax, 0x40013000
    ja .Lfail
    ret
.Lreadable:
    mov rax, rdi
    add rax, rcx
    jc .Lfail
    cmp rdi, 0x40010000
    jb .Lfail
    cmp rax, 0x40014000
    ja .Lfail
    ret

.Lfail:
    mov qword ptr [r12 + 80], rax
    mov rax, r14
    add rax, 0x100
    mov qword ptr [r12 + 48], rax
    mov eax, 2
    mov edi, 0xe4
    xor esi, esi
    xor edx, edx
    xor r10d, r10d
    xor r8d, r8d
    xor r9d, r9d
    int 0x80
    ud2
    .size _start, . - _start

    .section .data.ipc,"aw",@progbits
    .balign 4096
    .global ipc_control
ipc_control:
    .zero 0x3000
    .section .rodata.ipc,"a",@progbits
    .balign 4096
    .global ipc_readonly
ipc_readonly:
    .zero 4096
"#);

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop { core::hint::spin_loop(); }
}
