.text
.globl handoff
.p2align 4
handoff:
    cli
    cld
    lidt [rip + empty_idt]
    mov rsp, r8
    and rsp, -16
    xor ebp, ebp
    mov rdi, rdx
    call rcx
    mov dx, 0xf4
    mov eax, 0x11
    out dx, eax
1:
    cli
    hlt
    jmp 1b
.section .rdata,"dr"
empty_idt:
    .short 0
    .quad 0
