// Native architecture acceptance fixture. Direct int80 is intentional: no
// compiler may use/clobber the register bank whose survival is being measured.
// Syscall ABI: rax=number,rdi/rsi/rdx/r10/r8/r9=args; rax=status,rdx=value.
.section .text.entry,"ax"
.global _start
_start:
 jmp probe
.section .app.startup,"aw"
.balign 16
.global app_startup
app_startup:
 .zero 1024
.section .data,"aw"
.balign 16
snapshot:
 .zero 512
expected:
 .zero 512
.section .rodata,"a"
.balign 16
vectors:
 .quad ({role} * 4096 + 0), ~({role} * 4096 + 0)
 .quad ({role} * 4096 + 1), ~({role} * 4096 + 1)
 .quad ({role} * 4096 + 2), ~({role} * 4096 + 2)
 .quad ({role} * 4096 + 3), ~({role} * 4096 + 3)
 .quad ({role} * 4096 + 4), ~({role} * 4096 + 4)
 .quad ({role} * 4096 + 5), ~({role} * 4096 + 5)
 .quad ({role} * 4096 + 6), ~({role} * 4096 + 6)
 .quad ({role} * 4096 + 7), ~({role} * 4096 + 7)
 .quad ({role} * 4096 + 8), ~({role} * 4096 + 8)
 .quad ({role} * 4096 + 9), ~({role} * 4096 + 9)
 .quad ({role} * 4096 + 10), ~({role} * 4096 + 10)
 .quad ({role} * 4096 + 11), ~({role} * 4096 + 11)
 .quad ({role} * 4096 + 12), ~({role} * 4096 + 12)
 .quad ({role} * 4096 + 13), ~({role} * 4096 + 13)
 .quad ({role} * 4096 + 14), ~({role} * 4096 + 14)
 .quad ({role} * 4096 + 15), ~({role} * 4096 + 15)
// Distinct virtual operand addresses make stale/cross-process FDP observable.
.if {role} == 29
 .quad 0
.endif
integer: .quad {role}, ({role}+1), ({role}+2), ({role}+3), ({role}+4), ({role}+5), ({role}+6), ({role}+7)
control: .word {control}
mxcsr_value: .long {mxcsr}

.section .text,"ax"
probe:
 mov qword ptr [app_startup+152],1
 // Inspect initial hardware state BEFORE FNINIT, LDMXCSR or any SIMD op.
 fxsave64 [snapshot]
 cmp word ptr [snapshot],0x037f
 jne failure
 cmp word ptr [snapshot+2],0
 jne failure
 cmp byte ptr [snapshot+4],0
 jne failure
 cmp word ptr [snapshot+6],0
 jne failure
 cmp dword ptr [snapshot+24],0x1f80
 jne failure
 // FIP/FDP must not disclose firmware or a previous process.
 cmp qword ptr [snapshot+8],0
 jne failure
 cmp qword ptr [snapshot+16],0
 jne failure
 lea rdi,[snapshot+32]
 mov ecx,8
clean_st:
 mov rax,[rdi]
 or ax,[rdi+8]
 test rax,rax
 jne failure
 add rdi,16
 loop clean_st
 lea rdi,[snapshot+160]
 mov ecx,32
 xor eax,eax
 repe scasq
 jne failure
 mov qword ptr [app_startup+80],1
 mov qword ptr [app_startup+152],2
 // Confirm this exact qemu64 fixture has no AVX before using the negative case.
 mov eax,1
 cpuid
 test ecx,1<<28
 jne failure
 // Start all six syscall argument registers clean, then test each independently.
 xor edi,edi
 xor esi,esi
 xor edx,edx
 xor r10d,r10d
 xor r8d,r8d
 xor r9d,r9d
 xor eax,eax
 int 0x80
 test rax,rax
 jne failure
 mov rax,0x100000006
 cmp rdx,rax
 je abi_ready
 mov rax,0x10000000a
 cmp rdx,rax
 jne failure
 mov qword ptr [app_startup+168],1
abi_ready:
 xor edi,edi
 xor esi,esi
 xor edx,edx
 xor r10d,r10d
 xor r8d,r8d
 xor r9d,r9d
 mov rdi,1
 mov eax,3
 int 0x80
 cmp rax,-1
 jne failure
 test rdx,rdx
 jne failure
 inc qword ptr [app_startup+128]
 xor edi,edi
 xor esi,esi
 xor edx,edx
 xor r10d,r10d
 xor r8d,r8d
 xor r9d,r9d
 mov rsi,1
 mov eax,3
 int 0x80
 cmp rax,-1
 jne failure
 test rdx,rdx
 jne failure
 inc qword ptr [app_startup+128]
 xor edi,edi
 xor esi,esi
 xor edx,edx
 xor r10d,r10d
 xor r8d,r8d
 xor r9d,r9d
 mov rdx,1
 mov eax,3
 int 0x80
 cmp rax,-1
 jne failure
 test rdx,rdx
 jne failure
 inc qword ptr [app_startup+128]
 xor edi,edi
 xor esi,esi
 xor edx,edx
 xor r10d,r10d
 xor r8d,r8d
 xor r9d,r9d
 mov r10,1
 mov eax,3
 int 0x80
 cmp rax,-1
 jne failure
 test rdx,rdx
 jne failure
 inc qword ptr [app_startup+128]
 xor edi,edi
 xor esi,esi
 xor edx,edx
 xor r10d,r10d
 xor r8d,r8d
 xor r9d,r9d
 mov r8,1
 mov eax,3
 int 0x80
 cmp rax,-1
 jne failure
 test rdx,rdx
 jne failure
 inc qword ptr [app_startup+128]
 xor edi,edi
 xor esi,esi
 xor edx,edx
 xor r10d,r10d
 xor r8d,r8d
 xor r9d,r9d
 mov r9,1
 mov eax,3
 int 0x80
 cmp rax,-1
 jne failure
 test rdx,rdx
 jne failure
 inc qword ptr [app_startup+128]
 xor edi,edi
 xor esi,esi
 xor edx,edx
 xor r10d,r10d
 xor r8d,r8d
 xor r9d,r9d
 mov qword ptr [app_startup+152],3
 // Distinct instruction addresses make stale/cross-process FIP observable.
 .if {role} == 29
 .rept 11
 nop
 .endr
 .endif
 fldcw word ptr [control]
 ldmxcsr dword ptr [mxcsr_value]
 fild qword ptr [integer+0]
 fild qword ptr [integer+8]
 fild qword ptr [integer+16]
 fild qword ptr [integer+24]
 fild qword ptr [integer+32]
 fild qword ptr [integer+40]
 fild qword ptr [integer+48]
 fild qword ptr [integer+56]
 movdqa xmm0,[vectors+0]
 movdqa xmm1,[vectors+16]
 movdqa xmm2,[vectors+32]
 movdqa xmm3,[vectors+48]
 movdqa xmm4,[vectors+64]
 movdqa xmm5,[vectors+80]
 movdqa xmm6,[vectors+96]
 movdqa xmm7,[vectors+112]
 movdqa xmm8,[vectors+128]
 movdqa xmm9,[vectors+144]
 movdqa xmm10,[vectors+160]
 movdqa xmm11,[vectors+176]
 movdqa xmm12,[vectors+192]
 movdqa xmm13,[vectors+208]
 movdqa xmm14,[vectors+224]
 movdqa xmm15,[vectors+240]
 fxsave64 [expected]
 mov qword ptr [app_startup+136],1
 call getclock
 mov [app_startup+96],rdx
 mov [app_startup+104],rdx
main_loop:
 mov qword ptr [app_startup+152],4
 call verify
 // Syscall/yield has to preserve the entire FP bank and control state.
 xor edi,edi
 xor esi,esi
 xor edx,edx
 xor r10d,r10d
 xor r8d,r8d
 xor r9d,r9d
 cmp qword ptr [app_startup+168],0
 je yield_call
 // The previous verified clock value gives an absolute, bounded deadline.
 mov rdx,[app_startup+104]
 add rdx,2
 mov eax,0x22
 jmp schedule_call
yield_call:
 mov eax,1
schedule_call:
 int 0x80
 test rax,rax
 jne failure
 test rdx,rdx
 jne failure
 call verify
 mov ecx,50000
busy:
 pause
 dec ecx
 jne busy
 call verify
 call getclock
 cmp rdx,[app_startup+104]
 jb failure
 je clock_same
 inc qword ptr [app_startup+112]
clock_same:
 mov [app_startup+104],rdx
 inc qword ptr [app_startup+88]
 inc qword ptr [app_startup+8]
 mov rax,rdx
 sub rax,[app_startup+96]
 cmp rax,12
 jb main_loop
 cmp qword ptr [app_startup+88],8
 jb main_loop
 mov qword ptr [app_startup+144],1
 .if {role} == 13
 cmp qword ptr [app_startup+64],1
 jne successful_exit
 mov qword ptr [app_startup+152],5
 // AVX is outside this profile, with CR4.OSXSAVE clear. The real instruction
 // must fault at CPL3. No simulated fault or privileged test call is used.
 vxorps ymm0,ymm0,ymm0
 jmp failure
 .else
 cmp qword ptr [app_startup+160],1
 jne main_loop
 .endif
successful_exit:
 call verify
 xor edi,edi
 xor esi,esi
 xor edx,edx
 xor r10d,r10d
 xor r8d,r8d
 xor r9d,r9d
 mov eax,2
 int 0x80
 ud2
getclock:
 xor edi,edi
 xor esi,esi
 xor edx,edx
 xor r10d,r10d
 xor r8d,r8d
 xor r9d,r9d
 mov eax,3
 int 0x80
 test rax,rax
 jne failure
 inc qword ptr [app_startup+120]
 ret
verify:
 fxsave64 [snapshot]
 // Compare FCW/FSW/FTW, FOP/FIP/FDP and MXCSR; ignore reserved bytes and mask.
 mov eax,[expected]
 cmp [snapshot],eax
 jne failure
 mov al,[expected+4]
 cmp [snapshot+4],al
 jne failure
 mov ax,[expected+6]
 cmp [snapshot+6],ax
 jne failure
 mov rax,[expected+8]
 cmp [snapshot+8],rax
 jne failure
 mov rax,[expected+16]
 cmp [snapshot+16],rax
 jne failure
 mov eax,[expected+24]
 cmp [snapshot+24],eax
 jne failure
 lea rdi,[snapshot+32]
 lea rsi,[expected+32]
 mov ecx,8
verify_st:
 mov rax,[rsi]
 cmp [rdi],rax
 jne failure
 mov ax,[rsi+8]
 cmp [rdi+8],ax
 jne failure
 add rsi,16
 add rdi,16
 loop verify_st
 lea rdi,[snapshot+160]
 lea rsi,[expected+160]
 mov ecx,32
 repe cmpsq
 jne failure
 ret
failure:
 mov rdi,[app_startup+152]
 add rdi,100
 xor esi,esi
 xor edx,edx
 xor r10d,r10d
 xor r8d,r8d
 xor r9d,r9d
 mov eax,2
 int 0x80
 ud2
