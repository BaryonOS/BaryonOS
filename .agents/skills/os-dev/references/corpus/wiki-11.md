# OSDev Wiki — volledige bronteksten, deel 11

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w003774"></a>

## W003774 — CPU Registers x86-64

Bron-ID: W003774
Namespace: Main (0)
Revisie: 30266 | Datum: 2026-03-16T17:44:07Z
Bron: https://wiki.osdev.org/CPU_Registers_x86-64
Revisie-URL: https://wiki.osdev.org/index.php?title=CPU_Registers_x86-64&oldid=30266

````text
==General Purpose Registers==

{| class="wikitable" border="1"
|-
! colspan=5 | Monikers
! rowspan=2 | Description
|-
! 64-bit
! 32-bit
! 16-bit
! 8 high bits of lower 16 bits
! 8-bit
|-
| RAX
| EAX
| AX
| AH
| AL
| Accumulator
|-
| RBX
| EBX
| BX
| BH
| BL
| Base
|-
| RCX
| ECX
| CX
| CH
| CL
| Counter
|-
| RDX
| EDX
| DX
| DH
| DL
| Data (commonly extends the A register)
|-
| RSI
| ESI
| SI
| N/A
| SIL
| Source index for string operations
|-
| RDI
| EDI
| DI
| N/A
| DIL
| Destination index for string operations
|-
| RSP
| ESP
| SP
| N/A
| SPL
| Stack Pointer
|-
| RBP
| EBP
| BP
| N/A
| BPL
| Base Pointer (meant for stack frames)
|-
| R8
| R8D
| R8W
| N/A
| R8B
| General purpose
|-
| R9
| R9D
| R9W
| N/A
| R9B
| General purpose
|-
| R10
| R10D
| R10W
| N/A
| R10B
| General purpose
|-
| R11
| R11D
| R11W
| N/A
| R11B
| General purpose
|-
| R12
| R12D
| R12W
| N/A
| R12B
| General purpose
|-
| R13
| R13D
| R13W
| N/A
| R13B
| General purpose
|-
| R14
| R14D
| R14W
| N/A
| R14B
| General purpose
|-
| R15
| R15D
| R15W
| N/A
| R15B
| General purpose
|}

Note: you cannot access ''AH'', ''BH'', ''CH'' and ''DH'' when using the REX.W instruction prefix.
This prefix is added (automatically by assemblers) when an operand contains a 64-bit register.

==Pointer Registers==

{| class="wikitable" border="1"
|-
! colspan=3 | Monikers
! rowspan=2 | Description
|-
! 64-bit
! 32-bit
! 16-bit
|-
| RIP
| EIP
| IP
| Instruction Pointer
|-
|}

Note: The instruction pointer can only be used in RIP-relative addressing, which was introduced with long mode.

==Segment Registers==

All these are 16 bits long.

{| class="wikitable" border="1"
|-
! Moniker
! Description
|-
| CS
| Code Segment
|-
| DS
| Data Segment
|-
| SS
| Stack Segment
|-
| ES
| Extra Segment (used for string operations)
|-
| FS
| General-purpose Segment
|-
| GS
| General-purpose Segment
|-
|}

Segments of ''CS'', ''DS'', ''ES'', and ''SS'' are treated as if their base was 0 no matter what the segment descriptors in the GDT say.
Exceptions are ''FS'' and ''GS'' which have MSRs to change their base.

Limit checks are disabled for all segments.

==RFLAGS Register==

{| class="wikitable" border="1"
|-
! Bit(s)
! Label
! Description
|-
| 0
| CF
| Carry Flag
|-
| 1
| 1
| Reserved
|-
| 2
| PF
| Parity Flag
|-
| 3
| 0
| Reserved
|-
| 4
| AF
| Auxiliary Carry Flag
|-
| 5
| 0
| Reserved
|-
| 6
| ZF
| Zero Flag
|-
| 7
| SF
| Sign Flag
|-
| 8
| TF
| Trap Flag
|-
| 9
| IF
| Interrupt Enable Flag
|-
| 10
| DF
| Direction Flag
|-
| 11
| OF
| Overflow Flag
|-
| 12-13
| IOPL
| I/O Privilege Level
|-
| 14
| NT
| Nested Task
|-
| 15
| 0
| Reserved
|-
| 16
| RF
| Resume Flag
|-
| 17
| VM
| Virtual-8086 Mode
|-
| 18
| AC
| Alignment Check / Access Control
|-
| 19
| VIF
| Virtual Interrupt Flag
|-
| 20
| VIP
| Virtual Interrupt Pending
|-
| 21
| ID
| ID Flag
|-
| 22-63
| 0
| Reserved
|-
|}

==Control Registers==

====CR0====

{| class="wikitable" border="1"
|-
! Bit(s)
! Label
! Description
|-
| 0
| PE
| Protected Mode Enable
|-
| 1
| MP
| Monitor Co-Processor
|-
| 2
| EM
| Emulation
|-
| 3
| TS
| Task Switched
|-
| 4
| ET
| Extension Type
|-
| 5
| NE
| Numeric Error
|-
| 6-15
| 0
| Reserved
|-
| 16
| WP
| Write Protect
|-
| 17
| 0
| Reserved
|-
| 18
| AM
| Alignment Mask
|-
| 19-28
| 0
| Reserved
|-
| 29
| NW
| Not-Write Through
|-
| 30
| CD
| Cache Disable
|-
| 31
| PG
| Paging
|-
| 32-63
| 0
| Reserved
|-
|}
NOTE that this register is the only control register that can be written and read via 2 ways unlike the other that can be accessed only via the MOV instruction
<syntaxhighlight lang="asm">
;way 1:
;write:
mov cr0,reg32(64)
;read:
mov reg32(64),cr0 
;----------------------
;way 2:
;write:
lmsw reg16(32/64) ; the 'w' in lms(w) stands for word size (16 bit) but the instruction itself can modify the upper 48 bit of cr0 using instruction overrides.

;read:
smsw reg16(32/64) ; SAME as above
</syntaxhighlight>

====CR2====

This control register contains the linear (virtual) address which triggered a page fault, available in the page fault's interrupt handler.

====CR3====

{| class="wikitable" border="1"
|-
! colspan=2 | Bit(s)
! Label
! Description
! Condition
|-
| rowspan=4 | 0-11
| 0-2
| 0
| Reserved
| rowspan=4 | CR4.PCIDE = 0
|-
| 3
| PWT
| Page-Level Write Through
|-
| 5
| PCD
| Page-Level Cache Disable
|-
| 5-11
| 0
| Reserved
|-
| colspan=2 | 0-11
| colspan=2 | PCID
| CR4.PCIDE = 1
|-
| colspan=2 | 12-63
| colspan=2 | Physical Base Address of the PML4
|-
|}
Note that this must be page aligned

====CR4====

{| class="wikitable" border="1"
|-
! Bit(s)
! Label
! Description
|-
| 0
| VME
| Virtual-8086 Mode Extensions
|-
| 1
| PVI
| Protected Mode Virtual Interrupts
|-
| 2
| TSD
| Time Stamp enabled only in ring 0
|-
| 3
| DE
| Debugging Extensions
|-
| 4
| PSE
| Page Size Extension
|-
| 5
| PAE
| Physical Address Extension
|-
| 6
| MCE
| Machine Check Exception
|-
| 7
| PGE
| Page Global Enable
|-
| 8
| PCE
| Performance Monitoring Counter Enable 
|-
| 9
| OSFXSR
| OS support for fxsave and fxrstor instructions
|-
| 10
| OSXMMEXCPT
| OS Support for unmasked simd floating point exceptions
|-
| 11
| UMIP
| User-Mode Instruction Prevention (SGDT, SIDT, SLDT, SMSW, and STR are disabled in user mode)
|-
| 12
| LA57
| 5 level paging
|-
| 13
| VMXE
| Virtual Machine Extensions Enable
|-
| 14
| SMXE
| Safer Mode Extensions Enable
|-
| 15
| 0
| Reserved
|-
| 16
| FSGSBASE
| Enables the instructions RDFSBASE, RDGSBASE, WRFSBASE, and WRGSBASE 
|-
| 17
| PCIDE
| PCID Enable
|-
| 18
| OSXSAVE
| XSAVE And Processor Extended States Enable
|-
| 19
| 0
| Reserved
|-
| 20
| SMEP
| Supervisor Mode Executions Protection Enable
|-
| 21
| SMAP
| Supervisor Mode Access Protection Enable
|-
| 22
| PKE
| Enable protection keys for user-mode pages
|-
| 23
| CET
| Enable Control-flow Enforcement Technology
|-
| 24
| PKS
| Enable protection keys for supervisor-mode pages
|-
| 25-63
| 0
| Reserved
|-
|}

====CR8====

CR8 is a new register accessible in 64-bit mode using the REX prefix. CR8 is used to prioritize external [[interrupts]] and is referred to as the task-priority register (TPR).

The AMD64 architecture allows software to define up to 15 external interrupt-priority classes. Priority classes are numbered from 1 to 15, with priority-class 1 being the lowest and priority-class 15 the highest. CR8 uses the four low-order bits for specifying a task priority and the remaining 60 bits are reserved and must be written with zeros.

System software can use the TPR register to temporarily block low-priority interrupts from interrupting a high-priority task. This is accomplished by loading TPR with a value corresponding to the highest-priority interrupt that is to be blocked. For example, loading TPR with a value of 9 (1001b) blocks all interrupts with a priority class of 9 or less, while allowing all interrupts with a priority class of 10 or more to be recognized. Loading TPR with 0 enables all external interrupts. Loading TPR with 15 (1111b) disables all external interrupts.

The TPR is cleared to 0 on reset.

{| class="wikitable" border="1"
|-
!Bit
!Purpose
|-
|0-3
|Priority
|-
|4-63
|Reserved
|}

====CR1, CR5-7, CR9-15====

Reserved, the cpu will throw a #ud exeption when trying to access them.

==MSRs==

====IA32_EFER====

Extended Feature Enable Register (EFER) is a [[MSR|model-specific register]] added in the AMD K6 processor, to allow enabling the SYSCALL/SYSRET instruction, and later for entering and exiting [[x86-64|long mode]]. This register becomes architectural in [[AMD64]] and has been adopted by Intel. Its MSR number is 0xC0000080.
 
{| class="wikitable"
!Bit
!Name
!Label
!Ops
!Description
|- |-
|0
|SYSCALL Enable
|IA32_EFER.SCE
|R/W
|Enables SYSCALL/SYSRET instructions in 64-bit mode.
|-
|1
| colspan="4" rowspan="7" |Reserved
|-
|2
|-
|3
|-
|4
|-
|5
|-
|6
|-
|7
|-
|8
|IA-32e Mode Enable
|IA32_EFER.LME
|R/W
|Enables IA-32e mode operation.
|-
|9
| colspan="4" | Reserved
|-
|10
|IA-32e Mode Active
|IA32_EFER.LMA
|R
|Indicates IA-32e mode is active when set.
|-
|11
|Execute Disable Bit Enable
| IA32_EFER.NXE
| R/W
|
|-
|⋮
| colspan="4" |Reserved
|}
<blockquote>
Table from https://wiki.jwo.cz/index.php?title=IA32_EFER.</blockquote>

====FS.base, GS.base====
MSRs with the addresses 0xC0000100 (for FS) and 0xC0000101 (for GS) contain the base addresses of the FS and GS segment registers. These are commonly used for thread-pointers in user code and CPU-local pointers in kernel code. Safe to contain anything, since use of a segment does not confer additional privileges to user code.

In newer CPUs, these can also be written with WRFSBASE and WRGSBASE instructions at any privilege level.

====KernelGSBase====
MSR with the address 0xC0000102. Is basically a buffer that gets exchanged with GS.base after a swapgs instruction. Usually used to seperate kernel and user use of the GS register.

==Debug Registers==

====DR0 - DR3 ====

Contain linear addresses of up to 4 breakpoints. If paging is enabled, they are translated to physical addresses.

====DR6====

It permits the debugger to determine which debug conditions have occurred. When an enabled debug exception is triggered, low order bits 0-3 are set before entering debug exception handler.

==== DR7====

{| class="wikitable" border="1"
|-
!Bit
!Description
|-
|0
|Local DR0 Breakpoint
|-
|1
|Global DR0 Breakpoint
|-
|2
|Local DR1 Breakpoint
|-
|3
|Global DR1 Breakpoint
|-
|4
|Local DR2 Breakpoint
|-
|5
|Global DR2 Breakpoint
|-
| 6
| Local DR3 Breakpoint
|-
|7
|Global DR3 Breakpoint
|-
|16-17
|Conditions for DR0
|-
|18-19
|Size of DR0 Breakpoint
|-
|20-21
|Conditions for DR1
|-
|22-23
|Size of DR1 Breakpoint
|-
|24-25
|Conditions for DR2
|-
|26-27
|Size of DR2 Breakpoint
|-
|28-29
|Conditions for DR3
|-
|30-31
|Size of DR3 Breakpoint
|-
|}
A local breakpoint bit deactivates on hardware task switches, while a global does not.<br>
00b condition means execution break, 01b means a write watchpoint, and 11b means an R/W watchpoint. 10b is reserved for I/O R/W (unsupported).

==Test Registers==

{| class="wikitable" border="1"
|-
!Name
!Description
|-
|TR3 - TR5
|Undocumented
|-
|TR6
|Test Command Register
|-
|TR7
|Test Data Register
|-
|}

==Protected Mode Registers==

====GDTR====

{| class="wikitable" border="1"
|-
! colspan="2" |Operand Size
! rowspan="2" |Label
! rowspan="2" |Description
|-
!64-bit
! 32-bit
|-
| colspan="2" style="text-align:center" |Bits 0-15
|Limit
|Size of [[GDT]]
|-
| Bits 16-79
|Bits 16-47
|Base
| Starting Address of GDT
|-
|}

====LDTR====

Stores the segment selector of the [[LDT]].

====TR ====

Stores the segment selector of the [[TSS]].

====IDTR====

{| class="wikitable" border="1"
! colspan="2" |Operand Size
! rowspan="2" |Label
! rowspan="2" |Description
|-
!64-bit
! 32-bit
|-
| colspan="2" style="text-align:center" |Bits 0-15
|Limit
|Size of [[IDT]]
|-
|Bits 16-79
|Bits 16-47
|Base
|Starting Address of IDT
|-
|}

[[Category:CPU Registers]]
[[Category:X86 CPU]]
[[Category:X86-64]]
````

<a id="w003775"></a>

## W003775 — CPU Register x86

Bron-ID: W003775
Namespace: Main (0)
Revisie: 17967 | Datum: 2015-04-30T08:21:30Z
Bron: https://wiki.osdev.org/CPU_Register_x86
Revisie-URL: https://wiki.osdev.org/index.php?title=CPU_Register_x86&oldid=17967
Doorverwijzing: CPU Registers x86

````text
#REDIRECT [[CPU Registers x86]]
````

<a id="w003779"></a>

## W003779 — RFLAGS

Bron-ID: W003779
Namespace: Main (0)
Revisie: 18013 | Datum: 2015-05-02T11:26:18Z
Bron: https://wiki.osdev.org/RFLAGS
Revisie-URL: https://wiki.osdev.org/index.php?title=RFLAGS&oldid=18013
Doorverwijzing: CPU Registers x86-64

````text
#REDIRECT [[CPU Registers x86-64#RFLAGS Register]]
````

<a id="w003781"></a>

## W003781 — User:Robschia

Bron-ID: W003781
Namespace: User (2)
Revisie: 18001 | Datum: 2015-05-01T13:02:13Z
Bron: https://wiki.osdev.org/User:Robschia
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARobschia&oldid=18001

````text
""
````

<a id="w003782"></a>

## W003782 — Category:Humor

Bron-ID: W003782
Namespace: Category (14)
Revisie: 18006 | Datum: 2015-05-01T22:24:34Z
Bron: https://wiki.osdev.org/Category:Humor
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AHumor&oldid=18006

````text
All the pages of this category are works of humor. Don't take them seriously!
````

<a id="w003783"></a>

## W003783 — EFLAGS

Bron-ID: W003783
Namespace: Main (0)
Revisie: 18012 | Datum: 2015-05-02T11:25:54Z
Bron: https://wiki.osdev.org/EFLAGS
Revisie-URL: https://wiki.osdev.org/index.php?title=EFLAGS&oldid=18012
Doorverwijzing: CPU Registers x86

````text
#REDIRECT [[CPU Registers x86#EFLAGS Register]]
````

<a id="w003784"></a>

## W003784 — MacOS

Bron-ID: W003784
Namespace: Main (0)
Revisie: 26140 | Datum: 2021-06-15T14:50:00Z
Bron: https://wiki.osdev.org/MacOS
Revisie-URL: https://wiki.osdev.org/index.php?title=MacOS&oldid=26140

````text
{{stub}}

XNU (acronym for "X is Not Unix") is a [[Hybrid Kernel|hybrid kernel]] based on [[wikipedia:Mach (kernel)|Mach]] 2.5, a hybrid kernel derived from [[wikipedia:Berkeley_Software_Distribution|BSD]] and the [[wikipedia:Accent kernel|Accent]] kernel, which is used in macOS. It was developed for the NeXTStep operating system; when Apple purchased NeXT, they opted to use NeXTStep to replace the obsolete classic Mac OS (Mac OS 9 and older), which had poor memory protection and cooperative multitasking.

macOS is Apple's partly proprietary operating system for Macintosh computers, which is, in fact, able to be run on any modern x86-64 computer; it also has a mobile version, iOS, and other specialised editions: tvOS and watchOS.

Darwin is the term for the core BSD UNIX system which macOS builds atop.

== Features ==

XNU as a direct derivative from 4.3BSD inherits most of its features, as well as some features imported from other BSD systems such as FreeBSD. Features of BSD heritage include:
* POSIX system calls and the various BSD extensions;
* the [[Wikipedia:Kqueue|kqueue]] event multiplexing system;
* [[wikipedia:Mandatory_access_control|Mandatory Access Control (MAC)]].

And XNU and OS X introduce a lot of specific features, including:
* own sandboxing system;
* own file system hierarchy (with [[wikipedia:Filesystem_Hierarchy_Standard|FHS]] preserved);
* Apple Events and FSEvents;
* OpenDirectory, an authentification system;
* SystemConfiguration, a modular configuration mechanism;
* [https://developer.apple.com/library/ios/documentation/System/Conceptual/ManPages_iPhoneOS/man3/asl.3.html Apple System Log (ASL)];
* [[wikipedia:AppleScript|AppleScript]];
* notifyd and distnoted, notification ([[Message_Passing|IPC]]) mechanisms;
* [[wikipedia:launchd|launchd]], a system initialisation and service management system'
* Mach APIs;
* the I/O Kit, an object-oriented drivers framework;
* and more.

OS X is a certified UNIX system.

== Security ==

==== Code signing ====
OS X and iOS support code signing. In OS X it can be disabled, but in iOS it is a heavy protection mechanism, which allows only reviewed apps to be run. However, Apple sells developer and enterprise certificates which can be used to distribute malware; a user only needs to install a profile. There also were a vulnerability which allowed to install outdated profiles by resetting the date. Apple can easily block certificates.

==== Entitlements ====
An entitlement is simply a permission written for a binary; even with root access it is not possible to override them. For example, running a program which uses '''task_for_pid''' Mach call under root without the required entitlements will result in an error. Of course, entitlements are signed.

==== Sandboxing ====
This feature is present in both OS X and iOS, but in iOS it is much harder. In fact, it is a jail, which places applications inside their own environment, from which they can only access their own root. More about app sandbox [https://developer.apple.com/library/mac/documentation/Security/Conceptual/AppSandboxDesignGuide/AboutAppSandbox/AboutAppSandbox.html here].

== Hybrid Kernel Design ==
macOS is derived from the Mach 2.5 operating system, a derivative of the 4.3BSD UNIX system which rearchitected the lowest levels of the BSD kernel. The primary inspiration for this redesign was the Accent kernel developed at Carnegie-Mellon University. Mach made several adaptations to the Accent model to better support UNIX compatibility.

The core abstractions of the Mach kernel are four: tasks, threads, ports, port sets, messages, and memory objects. The BSD kernel is adapted to use these underlying Mach abstractions to provide a complete higher-level BSD UNIX system.

A later version of Mach, version 3.0, was able to detach the new low-level abstraction layer from the BSD kernel, yielding a [[Microkernel]], where the BSD kernel was replaced with userland servers carved out from the BSD kernel code. XNU, however, was forked from Mach 2.5, which had not yet done so. While some improvements were imported from Mach 3.0, Apple opted not to separate the BSD kernel into userland servers. Thus XNU represents the [[Hybrid Kernel]] approach.

== See Also ==

=== Articles ===
* [[Microkernel]]
* [[Hybrid Kernel]]

=== External Links ===
* http://opensource.apple.com - here you can obtain sources of all open source components.
````

<a id="w003785"></a>

## W003785 — Mac OS X

Bron-ID: W003785
Namespace: Main (0)
Revisie: 20687 | Datum: 2017-04-10T17:42:34Z
Bron: https://wiki.osdev.org/Mac_OS_X
Revisie-URL: https://wiki.osdev.org/index.php?title=Mac_OS_X&oldid=20687
Doorverwijzing: MacOS

````text
#REDIRECT [[MacOS]]
````

<a id="w003786"></a>

## W003786 — XNU

Bron-ID: W003786
Namespace: Main (0)
Revisie: 20690 | Datum: 2017-04-10T17:43:31Z
Bron: https://wiki.osdev.org/XNU
Revisie-URL: https://wiki.osdev.org/index.php?title=XNU&oldid=20690
Doorverwijzing: MacOS

````text
#REDIRECT [[MacOS]]
````

<a id="w003787"></a>

## W003787 — User:Alexander/Combining Bran's Kernel Development and the Barebones Tutorials

Bron-ID: W003787
Namespace: User (2)
Revisie: 19540 | Datum: 2016-07-22T22:52:52Z
Bron: https://wiki.osdev.org/User:Alexander/Combining_Bran%27s_Kernel_Development_and_the_Barebones_Tutorials
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAlexander%2FCombining_Bran%27s_Kernel_Development_and_the_Barebones_Tutorials&oldid=19540

````text
{{Template:In Progress}}
<!--{{Rating|1}}-->
This guide is aimed to help you combine the [http://www.osdever.net/tutorials/view/brans-kernel-development-tutorial Bran's Kernel Development] and the [[Barebones]] tutorials. This way you get the more modern gcc arguments, etc. and the clear explanation of interupts and nasm syntax assembly. This guide will also implement the fixes found in [[Bran's Known Bugs]]. It is recommended that you read both tutorials fully first as this guide will only tell you what to change some of the instructions in Bran's into. The sections of this guide are the same as the pages of Bran's tutorial

== Getting Started ==
Make sure you are using a [[GCC Cross Compiler]]. Using DJGPP is not officially supported on newer systems than Windows XP. So sticking with the GCC Cross Compiler(i686) used in the Barebones tutorial is advised.

== The Basic Kernel ==
In your buildscript change:
<pre>
nasm -f aout -o start.o start.asm
</pre>
into:
<pre>
nasm -f elf -o start.o start.asm
</pre>

== Creating Main and Linking C Sources ==
The tutorial wants you to add:
<source lang="asm">
    extern _main
    call _main
</source>
into the correct place in your start.asm file. Use this instead:
<source lang="asm">
    extern main
    call main
</source>
Compiling the C sources in this tutorial should be done with:
<pre>
gcc -I./include -c -o main.o main.c -std=gnu99 -ffreestanding -O2 -Wall -Wextra
</pre>
of course replacing main.o and main.c.

== Printing to the screen ==
I recommend you use the terminal/vga functions from the [[Barebones]] tutorial. You can use Bran's <code>cls()</code>, <code>move_csr()</code> and <code>scroll()</code> functions as long as you replace <code>csr_x</code> and <code>csr_y</code> to <code>terminal_column</code> and <code>terminal_row</code>. You also want to make sure you have a global <code>unsigned short *textmemptr</code> and you set it's adress in your <code>terminal_initialize</code> by adding this line:
<source lang="c">textmemptr = (unsigned short *)0xB8000;</source>

== The GDT ==
In the assembly snippet that you are adding to start.asm replace all identifiers starting with an underscore(<code>_gp</code>, <code>_gdt_flush</code>) to their equivalents without a prefixed underscore.(<code>gp</code>, <code>gdt_flush</code>)

== The IDT ==
In the assembly snippet that you are adding to start.asm replace all identifiers starting with an underscore(<code>_idtp</code>, <code>_idt_load</code>) to their equivalents without a prefixed underscore.(<code>idtp</code>, <code>idt_load</code>)

== Interrupt Service Routines ==
In the assembly snippet that you are adding to start.asm replace all identifiers starting with an underscore(<code>_isr0</code>, <code>_isr1</code>, etc., <code>_fault_handler</code>) to their equivalents without a prefixed underscore.(<code>isr0</code>, <code>isr1</code>, etc., <code>fault_handler</code>)

== IRQs and PICs ==
In the assembly snippet that you are adding to start.asm replace all identifiers starting with an underscore(<code>_irq0</code>, <code>_irq1</code>, etc., <code>_irq_handler</code>) to their equivalents without a prefixed underscore.(<code>irq0</code>, <code>irq1</code>, etc., <code>irq_handler</code>)

== The PIT: A System Clock ==
Make sure you replace:
<source lang="c">
int timer_ticks = 0;
</source>
with:
<source lang="c">
volatile int timer_ticks = 0;
</source>
And that you also replace:
<source lang="c">
irq_install_handler(0, timer_handler);
</source>
with:
<source lang="c">
irq_install_handler(0, (unsigned) timer_handler);
</source>
To preserve power when waiting with the <code>timer_wait()</code> function you can replace:
<source lang="c">
while(timer_ticks < eticks);
</source>
with:
<source lang="c">
while(timer_ticks < eticks) {
	__asm__ __volatile__("hlt");
}
</source>
````

<a id="w003788"></a>

## W003788 — User:Alexander

Bron-ID: W003788
Namespace: User (2)
Revisie: 18030 | Datum: 2015-05-03T09:14:00Z
Bron: https://wiki.osdev.org/User:Alexander
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAlexander&oldid=18030

````text
Take a look at: 

[[User:Alexander/Combining_Bran's_Kernel_Development_and_the_Barebones_Tutorials|Combining Bran's Kernel Development and the Barebones Tutorials]]<br />
and<br />
[[User:Alexander/Designing_Your_Operating_System|Designing Your Operating System]]
````

<a id="w003789"></a>

## W003789 — User:Roman

Bron-ID: W003789
Namespace: User (2)
Revisie: 18028 | Datum: 2015-05-02T17:53:57Z
Bron: https://wiki.osdev.org/User:Roman
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARoman&oldid=18028

````text
[https://rzhikharevich.github.io/ My blog].
````

<a id="w003791"></a>

## W003791 — User:Themav

Bron-ID: W003791
Namespace: User (2)
Revisie: 18090 | Datum: 2015-05-21T18:14:01Z
Bron: https://wiki.osdev.org/User:Themav
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AThemav&oldid=18090

````text
My user page.


What I'm currently working on:

MKOS (A basic system that will eventually have a GUI) 
: Please read the [http://os.themarshallk.com/ MkOS Website/Wiki]



What I'm interested in:

GUIs

Programming

Having fun with my computer

Software freedom/hackability

Just about anything else that uses a computer...
````

<a id="w003793"></a>

## W003793 — Talk:ISRs PIC And Multitasking

Bron-ID: W003793
Namespace: Talk (1)
Revisie: 22158 | Datum: 2018-02-21T03:20:42Z
Bron: https://wiki.osdev.org/Talk:ISRs_PIC_And_Multitasking
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AISRs_PIC_And_Multitasking&oldid=22158

````text
== IRQs in C ==
Why make this overly complicated hack when using assembly remains both shorter and simpler?

In my opinion, this is easier to begin with. It was very useful for me and allowed me to develop faster. But yes, it should be substituted by assembler interrupt handlers, mainly for efficiency so that not so many function calls are made (ISR->dispatcher->handler). [[User:Unsigned|Unsigned]] 03:53, 9 May 2015 (CDT)

I should point out that multiple asm statements even when volatile are not guaranteed to appear in the same order by GCC even if they appear in succession. [[User:Michaelpetch|Michaelpetch]] 21:20, 20 February 2018 (CST)
````

<a id="w003794"></a>

## W003794 — Pci

Bron-ID: W003794
Namespace: Main (0)
Revisie: 18064 | Datum: 2015-05-12T11:28:18Z
Bron: https://wiki.osdev.org/Pci
Revisie-URL: https://wiki.osdev.org/index.php?title=Pci&oldid=18064
Doorverwijzing: PCI

````text
#REDIRECT [[PCI]]
````

<a id="w003795"></a>

## W003795 — User:Max/Preparing Windows as a build environment

Bron-ID: W003795
Namespace: User (2)
Revisie: 27464 | Datum: 2022-10-10T15:30:57Z
Bron: https://wiki.osdev.org/User:Max/Preparing_Windows_as_a_build_environment
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMax%2FPreparing_Windows_as_a_build_environment&oldid=27464

````text

----

'''IMPORTANT:''' Since '''[[Windows Subsystem for Linux]]''' is here, I highly recommend using it instead of Cygwin. It makes your life much easier. Simply follow the tutorials for building a toolchain and do so under WSL.

----


This page explains how to prepare your Windows system to be ready to act as a build environment for building an [[OS Specific Toolchain|OS specific toolchain]] or a [[GCC Cross-Compiler|cross compiler]] on.

The goal is to set up a [https://www.cygwin.com/ Cygwin] environment that is capable of building <code>binutils</code> and <code>gcc</code> for your target system, as this usually seems to be the hardest part when Windows users try to create a cross compiler. This page does not cover building the actual toolchain, but how to prepare your system to be able to do so.

To compile binutils and gcc for a target system, the ''host system'' must have:
* gcc
* make
* autoconf
* automake
* gmp
* mpc
* mpfr

The explanations on this page assume that your PATH is not polluted by other toolchains (remove MinGW etc.). A basic understanding of how to build software with configure scripts and makefiles is also assumed.

== Reference system ==
The steps explained here should work on any Windows version starting from 7. The reference system for this is a fresh Windows 10 installation.

== Installing Cygwin ==
In this most basic step we will set up Cygwin and create the pre-requisites for building autoconf and automake in the next steps.

1. Download the [https://www.cygwin.com/ Cygwin setup] file and start it

2. Select <code>gcc-core</code>, <code>gcc-g++</code>, <code>make</code>, <code>m4</code>, <code>perl</code>

3. Perform installation

Now you have clean setup of Cygwin with only the absolutely required tools. There will now be the <code>Cygwin Terminal</code> installed on your system which you use for the following steps.

== Compiling autoconf & automake ==
Specific versions of GCC depend on specific versions of autoconf/automake/aclocal on your host system. To build gcc 4.9 for example, you need exactly autoconf 2.64 and automake 1.11.1. Adapt the following steps to match the compiler version you want to build later.

1. Download the archives of [http://ftp.gnu.org/gnu/autoconf/ autoconf 2.64] and [http://ftp.gnu.org/gnu/automake/ automake 1.11.1].

2. Build autoconf from within a separate build directory using:
<source lang="bash">../autoconf-2.64/configure --prefix=/versions/autoconf-2.64
make && make install</source>

3. Create a symbolic link to make autoconf available on the path:
<source lang="bash">ln -s /versions/autoconf-2.64/bin/autoconf /bin/autoconf</source>

4. Build automake from within a separate build directory using:
<source lang="bash">../automake-1.11.1/configure --prefix=/versions/automake-1.11.1
make && make install</source>

5. Create symbolic links to make automake available, too:
<source lang="bash">ln -s /versions/automake-1.11.1/bin/automake /bin/automake
ln -s /versions/automake-1.11.1/bin/aclocal /bin/aclocal</source>

With this approach the specific versions of a tool are installed into a <code>"/versions/tool-x.y.z"</code> directory. The binary of the version that should be used is then simply linked into the "/bin" directory.

== Installing GMP, MPFR, MPC ==
For GCC 4.9, the versions mentioned in these points should be suitable. For other versions, consider the respective readmes.

1. Download [https://gmplib.org gmp-6.0.0a], [http://www.mpfr.org mpfr-3.1.2] and [http://multiprecision.org mpc-1.0.3]

2. Build gmp from within a separate build directory using:
<source lang="bash">../gmp-6.0.0/configure --prefix=/usr/local
make && make install</source>

3. Build mpfr from within a separate build directory using:
<source lang="bash">../mpfr-3.1.2/configure --prefix=/usr/local --with-gmp=/usr/local
make && make install</source>

4. Build mpc from within a separate build directory using:
<source lang="bash">../mpc-1.0.3/configure --prefix=/usr/local --with-gmp=/usr/local --with-mpfr=/usr/local --enable-static --disable-shared
make && make install</source>

== Notes for building your toolchain ==
Now you are prepared to build binutils and gcc on your system. The required build tools <code>autoconf</code>, <code>aclocal</code> and <code>automake</code> are in your path, and the required libraries are installed to <code>/usr/local</code>.

Few points to notice:
* When building gcc, it will likely not find the locations of gmp/mpfr/mpc: use <code>--with-*=/usr/local</code> to tell it where each library is
````

<a id="w003796"></a>

## W003796 — User talk:Themav

Bron-ID: W003796
Namespace: User talk (3)
Revisie: 18088 | Datum: 2015-05-21T18:07:46Z
Bron: https://wiki.osdev.org/User_talk:Themav
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AThemav&oldid=18088

````text
Welcome to my talk page. Please keep things clean (no swearing, etc.) and please keep this page organized. Enjoy talking to me! -Themav

==General chat/talk==





==Talk about my [[User:Themav|user page]]==
````

<a id="w003797"></a>

## W003797 — User:Lionel/MSR Draft

Bron-ID: W003797
Namespace: User (2)
Revisie: 18854 | Datum: 2016-01-17T12:11:25Z
Bron: https://wiki.osdev.org/User:Lionel/MSR_Draft
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALionel%2FMSR_Draft&oldid=18854

````text

A prototype of a list of MSR's, lifted straight from the manual.


== MSR's  ==

{| class="wikitable sortable"
|-
! MSR Number
! MSR Name
! Access
! Originator
! class="unsortable" | Description/Comments
! class="unsortable" | Availability
|-
| 0x00000000
| IA32_P5_MC_ADDR (P5_MC_ADDR)
| 
| Intel
| See Section 35.20 of Intel Architecture Manual 3c, “MSRs in Pentium Processors.”
|
; Intel: Family = 05 Model >= 1
|-
| 0x0000001B
| IA32_APIC_BASE (APIC_BASE)
| 
| Intel
| See Intel Architecture Manual 3c.
|
; Intel : Family = 06 Model = 1 and newer<br />Others!
; AMD : Unknown
|-
| 0x0000003A
| IA32_FEATURE_CONTROL
| Read/Write
| Intel
| Control CPU Features
|
;All: If CPUID.01H: ECX[bit 5 or bit 6] = 1
|-
| 0x000000FF
| IA32_SAMPLE_AMD
| 
| AMD
| N/A
|
; AMD: Unknown
|-
| 0x00001107
| FCR
| 
| VIA
| R/W Feature Control Register, eax = FCRValue
|
; VIA: Unknown
|-
| 0x00001108
| FCR2 
| Read/Write
| VIA
| RW Feature Control Register 2, edx = FCR2_Hi, eax = FCRValue
|
; VIA: Unknown
|-
| 0x00001109
| FCR3
| Write Only
| VIA
| Feature Control Register 3, edx = FCR3_Hi, eax = FCRValue
|
; VIA: Unknown
|-
| 0xC0000080
| EFER
| Read/Write
| AMD
| Extended Features Enable Register - notably long mode
|
; AMD: introduced with K6 CPU. Introduced for syscall/sysret enabling.
|}
````

<a id="w003798"></a>

## W003798 — User:Greasemonkey

Bron-ID: W003798
Namespace: User (2)
Revisie: 18148 | Datum: 2015-05-27T05:58:47Z
Bron: https://wiki.osdev.org/User:Greasemonkey
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGreasemonkey&oldid=18148

````text
I worked on AWOS many years ago and wrote the slowest graphics API ever
````

<a id="w003799"></a>

## W003799 — User:Greasemonkey/Intel GenX

Bron-ID: W003799
Namespace: User (2)
Revisie: 18181 | Datum: 2015-06-01T06:09:33Z
Bron: https://wiki.osdev.org/User:Greasemonkey/Intel_GenX
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGreasemonkey%2FIntel_GenX&oldid=18181

````text
The Intel GenX GPU architecture (probably) covers the Intel HD and the later Intel GMA series of GPUs, starting with the Intel 965. Later revisions tend to add and remove and readd and relocate features and instructions over time, but otherwise remain fairly similar.

This will cover Gen4 (963, 965, G35) and later. Earlier GPUs do not have open documentation and are apparently very different.

As there is an awful lot to document, this is probably a better place for tutorials.

{{stub}}

==Systems tested==
* Compaq CQ60-210TU: GM45 DevCTG (GMA 4500MHD; Cantiga) with 1366x768 LVDS panel
* HP Pavilion dv6-6c35tx: i5-2450M DevSNB (HD 3000) with 1366x768 LVDS panel

Anything that doesn't exactly match the specs listed here may differ. Approach at your own risk.

==Finding the GPU==
Tested on:
* GM45 1366x768 CQ60-210TU
* HD3000 1366x768 dv6-6c35tx

Probe the [[PCI]] bus. It should be located at Bus 0, Device 2, Function 0.

GMAs tend to be advertised as at least two functions, so you may wish to check Function 1 as well. HDs tend to be advertised as just one.

Either way, there appears to be no difference between using BAR0 of Function 0 and BAR0 of Function 1, and only Function 0 gives a window into stolen memory, so you might be able to get away with just using Function 0.

With that said, make sure you check the IDs to ensure that they match hardware that you've actually tested. Different GPUs have different bugs and thus require different workarounds - even when they're within the same family.

Here's the information, assuming you are accessing everything through Function 0:
* uint64_t @ PCI 0x10: Location of MMIO registers. (XXX: for devices with a Function 1, this appears to do more than just MMIO registers - Function 1 should be able to provide a space with "just" MMIO registers.)
* uint64_t @ PCI 0x18: Location of stolen memory.

The 64-bit pointers have the lower 4 bits set to 0x4, so remember to mask it out before you use it, and remember to maintain that mask before writing back.

==Getting a display==
Tested on:
* GM45 1366x768 CQ60-210TU
* HD3000 1366x768 dv6-6c35tx

===Proper mode switch methodology===
Consult the appropriate section of your manual. It tends to be in Volume 3 at the start of one of the parts.

* G45: Vol3, pg 28
* SNB: Vol3.2, pg 8

===Simplified version===
This assumes that the BIOS configured the LVDS panel properly and/or in a way that we can just take its values and run with them. If it does, you can avoid an awful lot of pain.

This also assumes that your panel is less than 2048 pixels wide, but that can be fixed by adding a few "if" statements. However, the 1366x768 assumption is dropped from this version.

There is a chance that this will work on systems that don't have an LVDS panel, but you will be limited to whatever resolution the BIOS spews out.

If you want a non-native resolution, enable the panelfitter and adjust PIPEnSRC to suit. Note, the GMA panelfitter does love to blur everything. (HD 3000 seems fine.)

Here's how you get from VGA text mode to native 32bpp full-res BGRX mode the easy way, and by easy we mean this is very much empirical, about as risky, and assumes the BIOS doesn't have stupid bugs in it:

<pre>
	// Set VGA screen off
	outportb(0x3C4, 0x01);
	outportb(0x3C5, inportb(0x3C5)|(1<<5));

	// Wait at least 100us (Gen4.5 only needs 20us, but Gen6 needs 100us)
	wait_us(100);

	// Get correct VGA pipe
	// WARNING: Gen5 changes the location of this register!
	// Pre-Gen5 uses 0x71400, Gen5+ uses 0x41000.
	int real_VGACNTRL = (genx_typ < 0x5000 ? VGACNTRL : VGACNTRL_ILK);
	genx_pipe = (genx_reg32[real_VGACNTRL]>>29)&1;

	// Disable VGA
	genx_reg32[real_VGACNTRL] |= (1<<31);

	// Disable Display n
	genx_reg32[DSPnCNTR(genx_pipe)] &= ~(1<<31);

	// Set PIPEnSRC to screen resolution
	uint32_t vis_w = (genx_reg32[HTOTAL_n(genx_pipe)] & 0xFFFF)+1;
	uint32_t vis_h = (genx_reg32[VTOTAL_n(genx_pipe)] & 0xFFFF)+1;
	genx_reg32[PIPEnSRC(genx_pipe)] = ((vis_w-1)<<16)|(vis_h-1);

	// XXX: Lacking information on DevSNB's panelfitter.
	// Seems to work fine without disabling it.
	// For Gen4.5, however, disabling it is pretty much mandatory,
	// unless you can't stand GPUs that lack antialiasing.
	if(genx_typ >= 0x5000)
	{
		genx_reg32[PIPEnCONF(genx_pipe)] &= ~(1<<31); // Disable Pipe n
		while((genx_reg32[PIPEnCONF(genx_pipe)] & (1<<30))) {} // Wait for Pipe n to stop

		genx_reg32[PFIT_CONTROL] &= ~(1<<31); // Disable panelfitter

		genx_reg32[PIPEnCONF(genx_pipe)] |= (1<<31); // Enable Pipe n
	}

	// Set up Display n and enable
	genx_reg32[DSPnLINOFF(genx_pipe)] = 0x00000000; // linear offset
	genx_reg32[DSPnSTRIDE(genx_pipe)] = 2048*4; // scanline pitch
	genx_reg32[DSPnSURF(genx_pipe)] = 0x00000000; // surface base
	genx_reg32[DSPnCNTR(genx_pipe)] = (genx_reg32[DSPnCNTR(genx_pipe)] & ~(15<<26)) | (6<<26); // bit depth select, 6 = 32bpp BGRX
	genx_reg32[DSPnCNTR(genx_pipe)] = (genx_reg32[DSPnCNTR(genx_pipe)] & ~(3<<20)) | (0<<20); // pixel multiply
	genx_reg32[DSPnCNTR(genx_pipe)] = (genx_reg32[DSPnCNTR(genx_pipe)] & ~(1<<10)) | (0<<10); // tiling flag
	genx_reg32[DSPnCNTR(genx_pipe)] |= (1<<31); // enable
</pre>

With the above config, your framebuffer should be located at the start of stolen memory, and be 2048 32bpp BGRX pixels wide internally.

===Setting monitor timings===
Not tested on the dv6-6c35tx.

If you need to set the monitor timings, this code should be useful for a start. These timings work on the CQ60-210TU's surprisingly tolerant LVDS panel:

<pre>
	uint32_t vis_w = 1366;
	uint32_t vis_stretch_w = vis_w;
	uint32_t vis_sblank_w = 80;
	uint32_t vis_sync_w = 128;
	uint32_t vis_eblank_w = 200;

	uint32_t vis_h = 768;
	uint32_t vis_stretch_h = vis_h;
	uint32_t vis_sblank_h = 3;
	uint32_t vis_sync_h = 5;
	uint32_t vis_eblank_h = 22;

	uint32_t vis_blank_w = vis_sblank_w + vis_sync_w + vis_eblank_w;
	uint32_t vis_blank_h = vis_sblank_h + vis_sync_h + vis_eblank_h;
	genx_reg32[HTOTAL_n(genx_pipe)] = ((vis_w+vis_blank_w-1)<<16) | (vis_w-1);
	genx_reg32[HBLANK_n(genx_pipe)] = ((vis_w+vis_blank_w-1)<<16) | (vis_w-1);
	genx_reg32[HSYNC_n(genx_pipe)] = ((vis_w+vis_sblank_w+vis_sync_w-1)<<16) | (vis_w+vis_sblank_w-1);
	genx_reg32[VTOTAL_n(genx_pipe)] = ((vis_h+vis_blank_h-1)<<16) | (vis_h-1);
	genx_reg32[VBLANK_n(genx_pipe)] = ((vis_h+vis_blank_h-1)<<16) | (vis_h-1);
	genx_reg32[VSYNC_n(genx_pipe)] = ((vis_h+vis_sblank_h+vis_sync_h-1)<<16) | (vis_h+vis_sblank_h-1);
	genx_reg32[PIPEnSRC(genx_pipe)] = ((vis_stretch_w-1)<<16)|(vis_stretch_h-1);
</pre>

==Getting the ring buffer to work==
Tested on:
* GM45 1366x768 CQ60-210TU
* HD3000 1366x768 dv6-6c35tx

===References for RING_BUFFER_* registers===
* G45: Vol1a, pg 238
* HD3000: Vol1p3, pg 39

===References for commands===
* G45: Vol1b
* HD3000: Vol1p3-5

===General notes===

The ring buffer is vital for being able to send commands to the GPE (Geometry Processing Engine) and whatnot. Commands are at least 1 DWord long, but the ring buffer indices are aligned to the nearest QWord.

Note, the tail indicates the end of the buffer, and is where you write your commands to. The head indicates the start of the buffer, and is where the GPU reads from. While the head is DWord-aligned, the tail is QWord-aligned, so you may need to pad your instructions by inserting a MI_NOOP (0x00000000 will do).

It's possible to start the ring buffer and then advance the tail when you have a new command or batch of commands.

RING_BUFFER_START denotes the address relative to that stolen memory space.

RING_BUFFER_HEAD and RING_BUFFER_TAIL need to be given byte offsets, so if you add, say, two DWords, you'd add 8 to RING_BUFFER_TAIL.

Apparently you don't need a GTT to get this working.

If you want to check to see if this is working, NOPID is a useful register. The Gen6 docs seem to be missing the location of this register, however it is in the same location as Gen4.5 (0x02094).

==Using the blitter==
Tested on:
* GM45 1366x768 CQ60-210TU

The HD3000 was tested at some stage but the ring buffer stopped, suggesting an invalid instruction, although there may have also been a GTT issue.

===General notes===

Firstly you'll want to know your "raster op" modes. They are conceptually the same as the Amiga's modes, although probably with different sources.

Here's a list of useful modes:
* 0xF0: Set to pattern (useful for COLOR_BLT)
* 0xCC: Set to source image (useful for SRC_COPY_BLT)
* 0xAA: Set to destination (useful for not much)
* 0x55: Set to opposite of destination (useful for XOR effect)

Top bit determines the result when all of {pattern, source, destination} are 1. Bottom bit determines the result when all are 0.

The rest is All There In The Manual.

'''WARNING:''' When the manual says "pitch in dwords", what they ''really'' mean is "pitch in bytes, aligned to a dword boundary".

There are two sets of blitter commands: The XY_ commands, and the other commands. The other commands are a bit simpler, but there are only two of them.

*COLOR_BLT fills a rectangular area with a solid colour. Useful for clearing the screen like a boss.
*SRC_COPY_BLT copies from one place in memory to another. If you have no GTT, this will be GPU-to-GPU only. Still faster, easier and more powerful than EGA/VGA.

Here's an example of a 32bpp COLOR_BLT used to clear a screen with a nice purple tinge:
<pre>
	// COLOR_BLT
	genx_rb_push(0
		| (0x2<<29) | (0x40<<22)
		| (0x3<<20) // a:rgb mask
		| 0x03
	);
	genx_rb_push(0
		| (3<<24) // bit depth
		| (0xF0<<16) // raster op
		| ((screen_pitch_pixels*4) & 0xFFFF) // pitch in bytes, dword-aligned
	);
	genx_rb_push(((screen_height)<<16)|((screen_width)<<2)); // height in scanlines, width in bytes
	genx_rb_push(screen_stolen_memory_offset);
	genx_rb_push(0x00330066); // XXRRGGBB - HTML colour #330066
</pre>

The XY_ commands allow you to specify ranges using X,Y coordinate pairs and apply clipping based on those pairs.

Here's the XY_COLOR_BLT version of the above:
<pre>
	// XY_COLOR_BLT
	genx_rb_push(0
		| (0x2<<29) | (0x50<<22)
		| (3<<20) // a:rgb mask
		| (0<<11) // tiling enable (tile-X only)
		| 0x04
	);
	genx_rb_push(0
		| (0<<30) // clipping enable
		| (3<<24) // bit depth
		| (0xF0<<16) // raster op
		| ((screen_pitch_pixels*4) & 0xFFFF) // pitch in bytes, dword-aligned
	);
	genx_rb_push((0<<16) | (0)); // Y1:X1 top-left
	genx_rb_push(((screen_height)<<16)|((screen_width))); // Y2:X2 bottom-right
	genx_rb_push(screen_stolen_memory_offset);
	genx_rb_push(0x00330066); // XXRRGGBB - HTML colour #330066
</pre>

Note, if you want clipping, you'll want to run an XY_SETUP_CLIP_BLT command, and then enable the "clipping enable" flag.

==Making the GTT behave==
Tested on:
* GM45 1366x768 CQ60-210TU

===References for GTT page format===
* G45: Vol1a, pg214

===Finding GTTADR===
'''Gen4 only (NOT Gen4.5!)''': 32-bit address "GTTADR" at PCI B0:D2:F0:0x1C. ''(TODO: confirm)''

'''Gen4.5 and above''': 64-bit address "GTTMMADR" at PCI B0:D2:F0:0x18 , then add 2MB (0x200000). ''(TODO: confirm the "above" bit)''
tends to use GTTADR from PCI, which is the 32-bit address at PCI+0x1C.

===Allocating space for the GTT===
====Early Gen4====
Allocate a block of memory in the stolen memory space. 512KB is the largest you can use for the GTT, and allows for a 512MB virtual addressing space. Ensure that the block of memory is aligned with its size.

Once you have it in place, ensure that the graphics pipeline is flushed (if you don't know what this is, it probably already is flushed), then:

<pre>
	genx_reg32[GFX_FLSH_CNTL] = 0;
	genx_reg32[PGTBL_CTL] = 1 | gtt_offset; // GTT: 512KB, enabled
	genx_reg32[PGTBL_CTL2] = 0; // disables the PPGTT
	genx_reg32[GFX_FLSH_CNTL] = 0;
</pre>

We will get to modifying it pretty soon.

*Paging type 0 is for stolen memory.
*Paging type 3 is for main CPU memory. The GPU will snoop the cache for you.

Note that the actual screen is rendered using physical, unmapped "stolen" memory addresses.

Also note that direct access to the stolen memory via GMADR also uses the physical unmapped addresses.

====Gen4 "Bearlake-C" (G35?) and onwards====

Don't allocate it. The chip allocates it for you. Just leave the upper 31:12 in PGTBL_CTL intact when you mess with it. Once you have identity paging in place, set the lower bit.

Of course, if you are paranoid, you can always allocate some memory anyway.

Paging types above are as per pre-Gen6. Gen6 has different paging types, apparently.

===Identity paging===

In this example, genx_gtt32 points to GTTADR as calculated.

The GPU will handle all the caching issues for you if you use GTTADR. To add to this, in Gen6 this is the only way to access the GTT, so instead of learning the older method of writing via system RAM and then flushing the GPU's cache, you should just use this instead.

<pre>
	for(i = 0; i < 512*256; i++)
		genx_gtt32[i] = (((i)<<12) | (3<<1) | (1<<0));
</pre>

===Memory-to-GPU blit (and vice versa)===

This is for a 32bpp blit.

blk_(width|height) denotes the size of the blit to perform.
(src|dest)_gtt are addresses that the GPU will feed through the Global GTT or PPGTT
(src|dest)_pitch are the image pitches in DWords.

You must ensure that the GTT has the correct paging type for the given GPU for each page that this will need to use. For Gen4, use 0 for stolen memory, and 3 for system memory.

<pre>
	// SRC_COPY_BLT
	genx_rb_push((2<<29) | (0x43<<22)
		| (3<<20) // a:rgb mask
		| 0x04
	);
	genx_rb_push(0
		| (0<<30) // reverse X direction
		| (3<<24) // bit depth
		| (0xCC<<16) // raster op
		| (dest_pitch*4) // dest pitch in bytes
	);
	genx_rb_push((blk_height<<16) | (blk_width*4)); // dest dims
	genx_rb_push(dest_gtt);
	genx_rb_push(0
		| (src_pitch*4) // src pitch in bytes
	);
	genx_rb_push(src_gtt); // src addr
	genx_rb_punch();
</pre>

XY_SRC_COPY_BLT and whatnot should also work just fine, including with clipping and the like.

==See Also==
===Articles===
* [[Accelerated Graphic Cards]]

===External Links===
* [https://01.org/linuxgraphics/documentation/driver-documentation-prms Official documentation from Intel]
* [http://www.x.org/docs/intel/ Official documentation on X.org] - covers up to 2012, but also has Vol_1b_G45_core.pdf which is missing from the official Intel PRM list
* Chipset datasheets:
** [http://www.intel.com/Assets/PDF/datasheet/320122.pdf Mobile Intel 4 Series Datasheet] - covers the mobile version of Gen4.5.
* [http://forums.entechtaiwan.com/index.php?topic=2578.0 1366x768 LCD timings] - thread with several different sets of timings
* [http://en.wikipedia.org/wiki/List_of_Intel_graphics_processing_units List of Intel graphics processing units] on Wikipedia - useful for finding out PCI device IDs and exactly what generation you're using.
````

<a id="w003800"></a>

## W003800 — User:Glauxosdev/Tutorials/Bootloader

Bron-ID: W003800
Namespace: User (2)
Revisie: 18173 | Datum: 2015-05-30T11:24:23Z
Bron: https://wiki.osdev.org/User:Glauxosdev/Tutorials/Bootloader
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGlauxosdev%2FTutorials%2FBootloader&oldid=18173

````text
In this tutorial we will learn how to create a bootloader that loads some sectors and executes them.

==Take 0x01==

A bootloader is a [[Real Mode|real mode]] program that resides in the first sector of the disk; let it be on a hard disk or on a USB flash disk. The BIOS loads only the bootsector, so to start we have only 512 bytes, unless we load more sectors ourselves. The bootsector is loaded at address 0x00007C00, so using [[Real Mode|real mode]] [[Segmentation|segmentation]] the address is translated to 0x0000:0x7C00 or 0x07C0:0x0000. So for a start we have this code:

<source lang="asm">
bits 16    ; tell the assembler we want 16 bit code
org 0x7C00 ; tell the assembler to add 0x7C00 offset to labels, segments will be zeroed

start:

    .setup_segments:
        mov ax, 0      ; pass 0 value to ax, as segments can't be set with immediate values
        mov ds, ax     ; ds = ax = 0
        mov es, ax     ; es = ax = 0
        mov fs, ax     ; fs = ax = 0
        mov gs, ax     ; gs = ax = 0
        mov ss, ax     ; ss = ax = 0
    
    .setup_stack:
        mov sp, 0x7C00 ; stack grows downwards from the start of the bootloader

    .hang:
        jmp $          ; jump infinitely to the same instruction

end_boot_sector:

    times 510 - ($ - $$) db 0
    db 0x55
    db 0xAA
</source>

Save this file as "boot.asm" and assemble it with nasm:

<source lang="bash">
nasm -f bin -o boot.bin boot.asm
</source>

Then copy the resulting binary to the first sector of a USB flash disk, where xxx indicates the block device corresponding to the USB flash disk.

<source lang="bash">
sudo dd if=boot.bin of=/dev/xxx
</source>

'''Note:''' Don't try to copy the binary to the hard disk, as it will become unbootable.

Reboot your computer and you will see a blinking cursor. Congratulations, you have made a bootable application. But it doesn't much. So, let's continue.

==Take 0x02==

Let's add some more code so we can load a sector:

<source lang="asm">
bits 16    ; tell the assembler we want 16 bit code
org 0x7C00 ; tell the assembler to add 0x7C00 offset to labels, segments will be zeroed

start:

    .setup_segments:
        mov ax, 0      ; pass 0 value to ax, as segments can't be set with immediate values
        mov ds, ax     ; ds = ax = 0
        mov es, ax     ; es = ax = 0
        mov fs, ax     ; fs = ax = 0
        mov gs, ax     ; gs = ax = 0
        mov ss, ax     ; ss = ax = 0
    
    .setup_stack:
        mov sp, 0x7C00 ; stack grows downwards from the start of the bootloader
    
    .save_boot_device_number:
        mov byte [bootdev], dl ; boot device number is passed in dl by BIOS

    .check_lba_extensions:
        mov ah, 0x41
        mov bx, 0x55AA
        int 0x13
        jc .hang       ; carry flag is set if bios lba extensions don't exist
        cmp bx, 0xAA55 ; bx = 0xAA55 if bios lba extensions exist
        jne .hang

    .load_next_sector:
        mov eax, 1          ; load sector index 1
        mov bx, next_sector ; at address indicated by next_sector label
        mov cx, 1           ; load only 1 sector
        call read_sectors   ; call the routine to load sectors
        jmp 0x7E00          ; jump to newly loaded sector

    .hang:
        jmp $          ; jump infinitely to the same instruction

read_sectors:
    
    .save_registers:
        pusha ; registers are preserved

    .build_disk_address_packet:
        mov si, 0x8000             ; disk address packet is at 0x00008000 (ds = 0)
        mov word [ds:si], 0x10     ; size of packet is 16 bytes
        mov word [ds:si + 2], cx   ; sector count is in cx
        mov word [ds:si + 4], bx   ; memory offset is in bx
        mov word [ds:si + 6], fs   ; memory segment is in fs
        mov dword [ds:si + 8], eax ; low starting lba sector is in eax
        mov dword [ds:si + 12], 0

    .invoke_disk_int:
        mov ah, 0x42           ; bios extended read is used here
        mov dl, byte [bootdev] ; boot device will be read
        int 0x13               ; bios disk interrupt is invoked

    .return:
        popa ; registers are restored
        ret  ; return

variables:

    bootdev db 0

end_boot_sector:

    times 510 - ($ - $$) db 0
    db 0x55
    db 0xAA

; SECTOR 0x01 START ===================================================================

next_sector:
    
    .print_letter_A:
        mov bx, 0xB800       ; video memory resides at 0x000B8000
        mov es, bx
        mov bx, 0            ; print letter "A" at the first row and column
        mov byte [es:bx], 65

    .hang:
        jmp $                ; jump infinitely to the same instruction

end_second_sector:
   
    times 512 - ($ - next_sector) db 0
</source>

If you assemble this file and boot a USB flash disk with the resulting binary, you will see a letter "A" at the top-left corner of the screen, except if your computer is such old so it doesn't support [[BIOS]] [[LBA]] extensions.
````

<a id="w003802"></a>

## W003802 — ARMv7-A Bare Bones

Bron-ID: W003802
Namespace: Main (0)
Revisie: 30234 | Datum: 2026-03-16T15:03:40Z
Bron: https://wiki.osdev.org/ARMv7-A_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=ARMv7-A_Bare_Bones&oldid=30234

````text
{{TutorialExplain}}
{{BeginnersWarning}}
{{Rating|1}}{{Kernel designs}}

In this tutorial, you will write a basic ARM kernel and boot it. You will target the ARM
Versatile Express development board for the Cortex-A15.

== Rationale ==

=== Why ARMv7-A? ===
ARMv7 is likely the last purely 32-bit iteration of the ARM architecture, so
processors based on it are likely to be supported the longest. The 'A' profile is
targeted at complex computing devices like smartphones, and is therefore of the
most interest to OS developers.

=== Why the Cortex-A15? ===
Largely because it's the most powerful ARMv7-A processor supported by QEMU, and it
also has its own development board.

=== Why the Versatile Express? ===
This board was designed by ARM Holdings as a prototyping board, so it makes sense
to target a relatively neutral platform built with the Cortex-A15 in mind.

== Toolchain ==

You will be using the [[GNU Toolchain]] for this, so go read [[GCC Cross-Compiler]] if you
haven't already. The target platform is <code>arm-none-eabi</code>. You will need at
least [[GAS]], [[GCC]], and [[LD]], but it is also suggested to have [[GDB]], [[Objcopy]], and [[Objdump]].

== Code ==

_start.arm:

<syntaxhighlight lang="asm">
.global _start
_start:
    ldr sp, =STACK_TOP
    bl start
1:
    b 1b
.size _start, . - _start
</syntaxhighlight>

start.c:

<syntaxhighlight lang="c">
#include <stdint.h>
#define UART0_BASE 0x1c090000

void start() {
    *(volatile uint32_t *)(UART0_BASE) = 'A';
}
</syntaxhighlight>

linker.ld:

<syntaxhighlight lang="c">
ENTRY(_start)

SECTIONS {
    /* QEMU loads the kernel in Flash here. I strongly suggest you look at the
     * memory map provided in the CoreTile TRM (see below).
     */
    . = 0x80010000;

    /* Make sure our entry stub goes first */
    .stub   : { _start.o(.text) }
    .text   : { *(.text) }
    .rodata : { *(.rodata) }
    .data   : { *(.data) }
    .bss    : { *(.bss COMMON) }

    STACK_BASE = .;
    . += 0x10000;
    STACK_TOP = .;
}
</syntaxhighlight>

== Building and Running ==

<syntaxhighlight lang="bash">
arm-none-eabi-as -march=armv7-a -mcpu=cortex-a15 _start.arm -o _start.o
arm-none-eabi-gcc -ffreestanding -Wall -Wextra -Werror -c start.c -o start.o
arm-none-eabi-ld -T linker.ld _start.o start.o -o kernel.elf
qemu-system-arm -M vexpress-a15 -cpu cortex-a15 -kernel kernel.elf -nographic
</syntaxhighlight>

== See Also ==
=== External Links ===
* [http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.subset.boards.express/index.html ARMv7 Cortex-A Series Programmer's Guide]
* [http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0406c/index.html ARMv7-A Architecture Reference Manual]
* [http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0406c/index.html ARM Cortex-A15 Technical Reference Manual]
* [http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.subset.boards.express/index.html CoreTile Express A-15 Technical Reference Manual]
* [http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.subset.boards.express/index.html Motherboard Express &mu;ATX Technical Reference Manual]

[[Category:ARM]]
[[Category:Bare bones tutorials]]
````

<a id="w003804"></a>

## W003804 — Richard Theseus

Bron-ID: W003804
Namespace: Main (0)
Revisie: 25201 | Datum: 2020-09-25T15:20:26Z
Bron: https://wiki.osdev.org/Richard_Theseus
Revisie-URL: https://wiki.osdev.org/index.php?title=Richard_Theseus&oldid=25201

````text
Richard likes his current OS very much, except for one thing: it's not written by him. Bit by bit, he wants to replace the parts of his current OS with his own code. His ultimate aim is to have an identical copy of his host OS, but with his code. Maybe you prefer a [[What order should I make things in|different approach]]

== Pro's & Con's ==
Since Richard already has a working OS, he can decide which parts of the system he wants to start working with. He could start with writing his own text editor and compiler and other userland binaries, or he could start with the kernel or device drivers. Testing the code is simple, since other parts of the OS are stable and in known working condition.

Once Richard has his new kernel ready, there are many userland programs already available.

Richard can't change the inner workings of his OS much, since it has to be compatible with the host OS. Even if he would notice bad design decisions in his host OS (which he won't, since the host is perfect), there's little he could do about it.

== Subtypes ==

* DOS cloner
The DOS cloner loves real mode, BIOS interrupts and Assembler

* Windows cloner
The Windows cloner loves a GUI and the general Look and Feel and ease of use of Windows

* Unix cloner
The Unix cloner loves standards (for example POSIX, ANSI etc.), the "Everything is a file" philosophy, the shell and its tools and the X Window System. (Note that some people liking the Unix way still make innovations, so they are not simple cloners.)

== Going further than Richard ==
Deviating from the host OS standards and creating something original.

== Richard's bookshelf ==
Richard has to know the inner workings of his host OS extremely well.

== Richard Theseus's opponents position ==
Is an OS made by copying the functionality of another OS actually a new OS at all? Greek philosophers have pondered the same questions back when family of Richard was sailing on the [https://en.wikipedia.org/wiki/Ship_of_Theseus ship sailed by the mythic Greek hero Theseus].

== People and OSes related to Richard Theseus ==
* GNU and xv6 (both cloning Unix)
* ReactOS (cloning Windows)
* FreeDOS (cloning MS-DOS)
* Haiku (cloning BeOS)

[[Category:Developer Archetype]]
````

<a id="w003805"></a>

## W003805 — EDID

Bron-ID: W003805
Namespace: Main (0)
Revisie: 30475 | Datum: 2026-04-11T00:15:02Z
Bron: https://wiki.osdev.org/EDID
Revisie-URL: https://wiki.osdev.org/index.php?title=EDID&oldid=30475

````text
{{FirstPerson}}

'''Extended Display Identification Data''' (EDID) is data about the monitor communicated to the video card's BIOS by a monitor's control hardware. It contains a lot of information, most notably probably being the "preferred" or "native" resolutions that that monitor supports. Where VBE can return a list of all software-supported video modes, EDID will provide special modes supported by the hardware. This usually includes "extended" modes like 1920x1080 or 4K. Getting a "preferred" video mode from the EDID can be an important step towards providing a satisfying desktop experience for an end user, as they can customize their "preferred" mode to their liking and save it for future desktop sessions. Note that the video mode(s) recieved from EDID querying will still need to be converted to a VBE video mode ID through the VBE supported mode list! You won't just be provided a video mode ID like some more simple methods of setting up video.

The first step in understanding the EDID record is to actually, well, GET it. The VESA VBE/DDC extension provides the means by which you will achieve this. For some background, VESA is an organization behind a lot of very important low-level graphics standards, including SVGA, DisplayID, E-EDID, and the VBE family of extensions. You will likely find some kind of VESA compliant video hardware in every video-capable machine introduced since the 90s. However, the specific VESA API this article is going to discuss is the VBE/DDC API, which stands for the Display Data Channel VESA BIOS Extension. This provides a standard way to get detailed video information, including the EDID block. The VBE extension family mostly uses [[BIOS]] interrupt 0x10 with AH=0x4F to communicate, and the DDC extension specifically uses AH=0x4F and AL=0x15. The two functions this article requires are as follows:

<code>int 0x10 AX=0x4F15 BL=0x00 - VBE/DDC - INSTALLATION CHECK</code>

<code>int 0x10 AX=0x4F15 BL=0x01 - VBE/DDC - READ EDID</code>

There is a secret, third DDC function called <code>READ VDIF</code>, though documentation for it is sparse, even in the standard itself; it is meant to give you the format that display information will be passed to you in, but EDID is the only format from that standard that has survived the test of time, so you really do not have to worry about running into more specifications.

== Getting the EDID Structure ==
First, you must check to make sure that the VBE/DDC extension is even supported by your BIOS, some do not support it even if they support SVGA! To do this, the first listed function should be used. While this also returns capability information that you could theoretically parse through, it usually is not particularly useful and can be safely ignored. However, it is a very quick function, so it can be used as an efficient installation check. You could theoretically skip the installation check and just rely on the return value of the <code>READ EDID</code> function, but this runs the risk of running into uninitialized memory and causing problems internally--as far as I know, the installation check is far more widely supported, even on BIOSes that don't have true VBE/DDC support.

In [[NASM]] assembly, you would invoke the installation check like this:
<syntaxhighlight lang="nasm">
mov ax, 0x4F15
xor bx, bx

; While not listed in Ralf Brown's interrupt list, the below sneaky requirements
; ARE listed in the standard, and must be set on strictly-compliant machines.
xor cx, cx ; Controller unit 0 (primary)
; ES:DI MUST be a null pointer, for reasons known only to the standards body.
mov es, bx
xor di, di

int 0x10
cmp al, 0x4F
jne .vbe_ddc_not_supported
cmp ah, 0x00
jne .vbe_ddc_install_check_error
</syntaxhighlight>

If this code executes successfully, you have a VBE/DDC-compatible BIOS. Now, you must actually get the EDID structure. This can be achieved in a very similar way:
<syntaxhighlight lang="nasm">
mov ax, 0x4F15
mov bl, 0x01
xor dx, dx

; If you're storing the EDID on a segment other than the first (0x0000) you
; should set that here.

mov di, edid_block

int 0x10
cmp al, 0x4F
jne .vbe_ddc_get_edid_unsupported
cmp ah, 0x00
jne .vbe_ddc_get_edid_failure
</syntaxhighlight>
After this code segment, you will have your EDID information in the location pointed to by ES:DI. This EDID structure is 128-bytes long, and contains the following information (packed):

{| class="wikitable"
|+EDID Structure Anatomy
!Offset
!Size
!Description
!Notes
|-
|0x00
|8 bytes
|Padding
|Either all 0xFF or 0x00, 0xFF, ..., 0xFF, 0x00.
|-
|0x08
|1 word
|Manufacturer ID
|Big endian, stored as letter "indices" (0x01 = A, 0x02 = B, etc).
|-
|0x0A
|1 word
|Monitor EDID ID
|Sometimes can identify monitor model.
|-
|0x0C
|1 dword
|Serial Number
|Not always provided, and some manufacturers put this in a strange format.
|-
|0x10
|1 byte
|Manufacture Week
|
|-
|0x11
|1 byte
|Manufacture Year
|Add 1990 to this value to get the actual year.
|-
|0x12
|1 byte
|EDID Version
|
|-
|0x13
|1 byte
|EDID Revision
|
|-
|0x14
|1 byte
|Video Input Type
|This is a bitfield; see the "Extra Tables" section.
|-
|0x15
|1 byte
|Maximum Horizontal Size
|In centimeters.
|-
|0x16
|1 byte
|Maximum Vertical Size
|In centimeters.
|-
|0x17
|1 byte
|Gamma Factor
|The monitor's gamma can be calculated as <1.0 + this/100>.
|-
|0x18
|1 byte
|DPMS Flags
|This is a bitfield; see the "Extra Tables" section.
|-
|0x19
|10 bytes
|Chroma Information
|This is a structure; see the "Extra Tables" section.
|-
|0x23
|1 byte
|Established Timings
|This is a bitfield; see the "Extra Tables" section.
|-
|0x24
|1 byte
|2nd Established Timings
|This is a bitfield; see the "Extra Tables" section.
|-
|0x25
|1 byte
|Manufacturer Timing
|Cleared to 0x00 if the manufacturer has not reserved a timing.
|-
|0x26
|8 words
|Standard Timings<sup>1</sup>
|8 standard modes; see the "Extra Tables" section.
|-
|0x36
|18 bytes
|Detailed Timing Description
|This is a structure; continue the article for anatomy.
|-
|0x48
|18 bytes
|Detailed Timing Description
|This is a structure; continue the article for anatomy.
|-
|0x5A
|18 bytes
|Detailed Timing Description
|This is a structure; continue the article for anatomy.
|-
|0x6C
|18 bytes
|Detailed Timing Description
|This is a structure; continue the article for anatomy.
|-
|0x7E
|1 byte
|Unused?
|Unknown what purpose this byte serves. Maybe reserved?
|-
|0x7F
|1 byte
|Checksum
|Calculated via <256 - the low byte of the structure summed up>.
|}
''1:'' The low byte of each mode descriptor is the resolution, which can be used to calculate the X resolution via <(byte + 31) * 8>.

Most of this information is not particularly useful to many smaller operating system projects. However, the main piece of important information within an EDID record is the first "detailed timing description", widely referred to as the "preferred resolution" for the graphics device. This is the resolution that the user or manufacturer has specified as the most preferable graphics mode for general operation. This is structure can come in two forms; one of which is far less useful to us. Typically, this very first record will ONLY be a "non-text" version, which is much easier to parse and contains much better information. That structure is 18 bytes long, and each item within it is exactly one byte long. Below is its specification;
{| class="wikitable"
|+EDID Detailed Timing Descriptor Anatomy
!Offset
!Size
!Description
!Notes
|-
|0x00
|1 byte
|Horizontal Frequency
|In kHz, but if 0x00, the record may be in text form.
|-
|0x01
|1 byte
|Vertical Frequency
|In Hz. This would be the "refresh rate" of your monitor.
|-
|0x02
|1 byte
|Horizontal Active Time
|Measured in pixels, and also referred to as the "X Resolution".
|-
|0x03
|1 byte
|Horizontal Blanking Time
|In Pixels.
|-
|0x04
|1 byte
|Horizontal Time Extensions<sup>1</sup>
|The first four bits are for active, and the second four bits are for blanking.
|-
|0x05
|1 byte
|Vertical Active Time
|Measured in lines, and also referred to as the "Y Resolution".
|-
|0x06
|1 byte
|Vertical Blanking Time
|In lines.
|-
|0x07
|1 byte
|Vertical Time Extensions<sup>1</sup>
|The first four bits are for active, and the second four bits are for blanking.
|-
|0x08
|1 byte
|Horizontal Sync Offset
|In pixels.
|-
|0x09
|1 byte
|Horizontal Sync Pulsewidth
|In pixels.
|-
|0x0A
|1 byte
|Vertical Sync Offset/Pulsewidth<sup>2</sup>
|The first four bits are offset, the second four bits are for pulsewidth.
|-
|0x0B
|1 byte
|Sync Offset 2 / Sync Pulsewidth 2
|This member is ambiguous; it can either be for the vertical or the horizontal.
|-
|0x0C
|1 byte
|Horizontal Image Size
|In milimeters.
|-
|0x0D
|1 byte
|Vertical Image Size
|In milimeters.
|-
|0x0E
|1 byte
|Image Size Extensions
|
|-
|0x0F
|1 byte
|Horizontal Border
|In pixels, you have to double this to get the full border.
|-
|0x10
|1 byte
|Vertical BorderBorder
|In lines, you have to double this to get the full border.
|-
|0x11
|1 byte
|Display Type
|This is a bitfield, see the "Extra Tables" section.
|}
''1:'' Because resolutions can go higher than 255 pixels, an extension must be made to include those pesky widescreen resolutions like...640x480. To get the full resolution number, you should use the four bits of whichever time you're extending (active/blanking) and use it as the top 4 bits of the resulting 12-bit number. To carry out this operation simply, one would do the following procedure;

* Move the full byte into the lower half of a register.
* Move the extension byte into an auxiliary register.
* If you're planning on getting the high half of the byte (active time) you'd shift the auxiliary register right by four. If you're planning on getting the low half, you'd bitwise AND the register by 0x0F.
* Move the auxiliary register into the upper half of the full byte's register.

''2:'' I honestly have zero idea why the standards committee would have possibly decided this was a good idea; thankfully, this information is not very interesting when received from the EDID--better information can be gotten from the SVGA mode descriptor. However, you would theoretically split this byte in half like you would the resolution extension byte<sup>1</sup> and then bitwise AND in the second offset/pulsewidth? Although, the second offset/pulsewidth can be either for the vertical or horizontal measurements, so just be careful, and you'd be much better off simply receiving this from SVGA.

=== Parsing the EDID Structure ===
Parsing the EDID structure is fairly straightforward, especially in assembly. It is very much preferable to define a structural container of some kind at the location where you're loading the EDID, as you will have to rely on magic offsets much less. Below is an example of a valid NASM EDID structure that makes parsing through the EDID data much more readable and easy;<syntaxhighlight lang="nasm">
struc vbe_edid_t
    .padding:                              resb 8
    .manufacturer_id:                      resw 1
    .monitor_id:                           resw 1
    .monitor_serial:                       resd 1
    .manufacturer_week:                    resb 1
    .manufacturer_year:                    resb 1
    .structure_version:                    resb 1
    .structure_revision:                   resb 1
    .video_input_type:                     resb 1
    .max_horizontal_size:                  resb 1
    .max_vertical_size:                    resb 1
    .gamma:                                resb 1
    .video_input_flags:                    resb 1
    .chroma_green_red:                     resb 1
    .chroma_white_blue:                    resb 1
    .chroma_red_y:                         resb 1
    .chroma_red_x:                         resb 1
    .chroma_green_y:                       resb 1
    .chroma_green_x:                       resb 1
    .chroma_blue_y:                        resb 1
    .chroma_blue_x:                        resb 1
    .chroma_white_y:                       resb 1
    .chroma_white_x:                       resb 1
    .timing_table_1:                       resb 1
    .timing_table_2:                       resb 1
    .manufacturer_timing:                  resb 1
    .standard_timing:                      resw 8
    .preferred_horizontal_frequency:       resb 1
    .preferred_vertical_frequency:         resb 1
    .preferred_horizontal_active_pixels:   resb 1
    .preferred_horizontal_blanking_pixels: resb 1
    .preferred_horizontal_pixels_2:        resb 1
    .preferred_vertical_active_lines:      resb 1
    .preferred_vertical_blanking_lines:    resb 1
    .preferred_vertical_lines_2:           resb 1
    .preferred_horizontal_sync_offset:     resb 1
    .preferred_horizontal_sync_pulsewidth: resb 1
    .preferred_vertical_sync_1:            resb 1
    .preferred_vertical_sync_2:            resb 1
    .preferred_horizontal_image_size:      resb 1
    .preferred_vertical_image_size:        resb 1
    .preferred_image_size_2:               resb 1
    .preferred_horizontal_border_pixels:   resb 1
    .preferred_vertical_border_lines:      resb 1
    .preferred_display_type:               resb 1
    .detailed_timing_2:                    resb 18
    .detailed_timing_3:                    resb 18
    .detailed_timing_4:                    resb 18
    .unused:                               resb 1 
    .checksum:                             resb 1 
endstruc
</syntaxhighlight>
For attribution's sake, that code block was taken from LavenderOS's install media bootloader (written by the same author as this article). To specifically get the preferred resolution's width and height, for the reason of checking against VBE mode numbers or the like, you would call a very simple set of instructions like this:

<syntaxhighlight lang="nasm">
; Get the X resolution
mov bl, byte [es:vbe_edid + vbe_edid_t.preferred_horizontal_active_pixels]
mov cl, byte[es:vbe_edid + vbe_edid_t.preferred_horizontal_pixels_2]
shr cl, 0x04
mov bh, cl

; Get the Y resolution
mov dl, byte[vbe_edid + vbe_edid_t.preferred_vertical_active_lines]
mov cl, byte[es:vbe_edid + vbe_edid_t.preferred_vertical_lines_2]
shr cl, 0x04
mov dh, cl
</syntaxhighlight>

Now you have the full EDID record in your program! Make sure to follow the standard's requirements for function interfacing strictly, for even if your emulator is lax, real hardware may not be.

=== Extra Tables ===
{| class="wikitable"
|+Video Input Type Bitfield
!Bit
!Description
!Notes
|-
|0
|Separate Sync
|
|-
|1
|Composite Sync
|
|-
|2
|Sync on Green
|
|-
|3-4
|Unused?
|The purpose of this field is unknown--it is likely reserved.
|-
|5-6
|Voltage Level
|00 = 0.700V/0.300V, 01 = 0.714V/0.286V, 10 = 0.100V/0.400V, 11 = reserved
|-
|7
|Signal Type
|0 = Analog, 1 = Digital
|}
{| class="wikitable"
|+DPMS Flag Bitfield
!Bit
!Description
!Notes
|-
|0-2
|Unused?
|The purpose of this field is unknown--it is likely reserved.
|-
|3
|Display Type
|0 = Non-RGB, 1 = RGB
|-
|4
|Unused?
|The purpose of this field is unknown--it is likely reserved.
|-
|5
|Active Off Supported
|
|-
|6
|Suspend Supported
|
|-
|7
|Standby Supported
|
|}
{| class="wikitable"
|+Chroma Information Anatomy
!Offset
!Size
!Description
!Notes
|-
|0x00
|1 byte
|Green X'/Y'  and Red X'/Y'
|
|-
|0x01
|1 byte
|White X'/Y'  and Blue X'/Y'
|
|-
|0x02
|1 byte
|Red Y
|
|-
|0x03
|1 byte
|Red X
|
|-
|0x04
|1 byte
|Green Y
|
|-
|0x05
|1 byte
|Green X
|
|-
|0x06
|1 byte
|Blue Y
|
|-
|0x07
|1 byte
|Blue X
|
|-
|0x08
|1 byte
|White Y
|
|-
|0x09
|1 byte
|White X
|
|}
{| class="wikitable"
|+Established Timings 1
!Bit
!Timing
!Notes
|-
|0
|720x400 @ 70 Hz
|Supported by IBM PCs, on VGA this actually indicates 640x480.
|-
|1
|720x400 @ 88 Hz
|Supported by XGA2-compatible machines.
|-
|2
|640x480 @ 60 Hz
|Supported by all VGA-compatible machines.
|-
|3
|640x480 @ 67 Hz
|Supported by the Apple Mac II.
|-
|4
|640x480 @ 72 Hz
|Supported by all VESA-compatible machines.
|-
|5
|640x480 @ 75 Hz
|Supported by all VESA-compatible machines.
|-
|6
|800x600 @ 56 Hz
|Supported by all VESA-compatible machines.
|-
|7
|800x600 @ 60 Hz
|Supported by all VESA-compatible machines.
|}
{| class="wikitable"
|+Established Timings 2
!Bit
!Timing
!Notes
|-
|0
|800x600 @ 72 Hz
|Supported by all VESA-compatible machines.
|-
|1
|800x600 @ 75 Hz
|Supported by all VESA-compatible machines.
|-
|2
|832x624 @ 75 Hz
|Supported by the Apple Mac II.
|-
|3
|1024x768 @ 87 Hz
|Interlaced format, supported by the 8514A.
|-
|4
|1024x768 @ 60 Hz
|Supported by all VESA-compatible machines.
|-
|5
|1024x768 @ 70 Hz
|Supported by all VESA-compatible machines.
|-
|6
|1024x768 @ 75 Hz
|Supported by all VESA-compatible machines.
|-
|7
|1280x1024 @ 75 Hz
|Supported by all VESA-compatible machines.
|}
{| class="wikitable"
|+Standard Timing Bitfield
!Bit
!Description
!Notes
|-
|0-5
|Vertical Refresh Frequency
|Add 60 to this to get the measurement in Hertz.
|-
|6-7
|Aspect Ratio
|You can calculate the Y resolution of the standard timing via <X * this>.
0x00 = Unknown, 0x01 = 0.75, 0x10 = 0.8, 0x11 = 0.5625
|}
{| class="wikitable"
|+Display Type Bitfield
!Bit
!Description
!Notes
|-
|0
|Unused?
|The purpose of this field is unknown--it is likely reserved.
|-
|1
|Sync Location / Horizontal Sync Polarity
|Horizontal sync polarity if sync type is digital separate, 0 = negative, 1 = positive.
|-
|2
|Serrate / Vertical Sync Polarity
|Vertical sync polarity if sync type is digital separate, 0 = negative, 1 = positive.
|-
|3-4
|Sync Type
|0x00 = Analog composite, 0x01 = Bipolar analog composite, 0x10 = Digital Composite, 0x11 = Digital separate
|-
|5-6
|Stereo Mode
|0x00 = Normal display, 0x01 = Stereo right sync high, 0x10 = Stereo left sync high, 0x11 = Undefined
|-
|7
|Interlaced
|
|}

== See Also ==
=== Articles ===
* [[VESA Video Modes]]

=== External Links ===
* [https://web.archive.org/web/20080614205229/http://www.vesa.org/public/VBE/VBEDDC11.PDF VBE/DDC 1.1 Standard]

[[Category:Video]]
[[Category:Hardware Detection]]
````

<a id="w003806"></a>

## W003806 — Time

Bron-ID: W003806
Namespace: Main (0)
Revisie: 18226 | Datum: 2015-07-08T14:18:38Z
Bron: https://wiki.osdev.org/Time
Revisie-URL: https://wiki.osdev.org/index.php?title=Time&oldid=18226
Doorverwijzing: Time And Date

````text
#REDIRECT [[Time And Date]]
````

<a id="w003807"></a>

## W003807 — Broken UEFI implementations

Bron-ID: W003807
Namespace: Main (0)
Revisie: 30194 | Datum: 2026-03-11T02:57:51Z
Bron: https://wiki.osdev.org/Broken_UEFI_implementations
Revisie-URL: https://wiki.osdev.org/index.php?title=Broken_UEFI_implementations&oldid=30194

````text
This page serves as a repository of information about any aspect of the system firmware which is broken on [[UEFI]] machines, but especially those directly related to UEFI itself.

== El Torito boot ==

UEFI boot from CD is controlled using [[El Torito]] boot records in the CD headers. Some machines get this wrong. In particular, one common set of known issues stem from early CSM packages which don't correctly interpret multiple El Torito boot catalog entries.  The most common failure is the CSM's parser not recognizing the 0xEF platform ID, and instead of interpreting "I don't know this platform ID" correctly as "This is not for my platform" when there are multiple boot entries, some versions present you with a menu:<pre>1.
2.
Select CD-ROM boot type:</pre>
Which represents the boot catalog entries (cleverly unlabeled).  On some platforms (including some Apple laptops, among others) this happens without setting up any keyboard controller, and you are now stuck.

== BGRT Table ==
* BGRT is an [[ACPI]] table to tell us if and where UEFI firmware has drawn its logo on the screen.  Technically, the BGRT is an ACPI 5 table, but its use corresponds with UEFI 2.4 deployments, and it goes hand in hand with the EFI Graphics Output Protocol and ESRT + UEFI UpdateCapsule and [[https://msdn.microsoft.com/en-us/library/dn917814%28v=vs.85%29.aspx Microsoft's firmware update graphics capsule]].  In theory, "uint16_t version" (offset 0x24) should always be 1, and "uint8_t status" (offset 0x26) with 0x1 set means "valid data" - that is, if the firmware displayed a splash graphic, it sets the values in the table and sets status to 1; if not, status should be 0.

Here are some sample entries.  These are from real machines, but the problems are common across lots of hardware from lots of vendors:
=== Lenovo X1 Carbon (1st gen, 346035U) ===
<blockquote><pre>00000000  42 47 52 54 38 00 00 00  00 00 4c 45 4e 4f 56 4f  |BGRT8.....LENOVO|
00000010  54 50 2d 47 36 20 20 20  00 27 00 00 50 54 4c 20  |TP-G6   .'..PTL |
00000020  02 00 00 00 00 00 01 00  18 b0 7e d3 00 00 00 00  |..........~.....|
00000030  0f 02 00 00 b4 00 00 00                           |........|</pre></blockquote>
This helpfully has version=0x0000 and status=0x01 i.e. some version that doesn't exist but the table claims its information is valid.

=== Intel strawberry mountain (DQ67SW) ===
<blockquote><pre>00000000  42 47 52 54 38 00 00 00  01 fe 49 4e 53 59 44 45  |BGRT8.....INSYDE|
00000010  4d 48 42 20 43 52 42 20  01 00 00 00 41 43 50 49  |MHB CRB ....ACPI|
00000020  00 00 04 00 01 00 00 00  00 d0 fd a7 00 00 00 00  |................|
00000030  fa 00 00 00 4f 00 00 00                           |....O...|</pre></blockquote>
This particular firmware says it's version 1 but that the data is invalid; this implementation /always/ says the data is invalid, whether it displays the logo or not.

== The Lenovo ThinkCentre M92p and maybe some related products ==

The Lenovo ThinkCentre M92p once shipped with a firmware which would only let you select boot entries where the Boot#### variable label was "Microsoft Windows" or "Red Hat Enterprise Linux".  This may have been fixed in firmware version 9SKT61A/9SJT61A , which says:<pre>CHANGES for 9SKT61A/9SJT61A
- Fixes some Linux secure boot issues.
- Updates Microcode M12306A9 to V17.  </pre>
Also the changelog says:
<pre>CHANGES for 9SKT51A/9SJT51A
- Fixes the issue of installing Redhat 6.3.
- Fixes the issue that sometimes the display cannot be full screen in text mode.</pre>
Which was probably when "Red Hat Enterprise Linux" was added to the list.

== Apple ==

Apple firmwares have a couple of funky things going on

=== There's additional UI presented mixed in with BDS ===

The "option key" boot screen and such are basically between BootNext/BootOrder and any other BDS menus and such.  In effect they are the BDS menus, but they do a lot of non-BDS things.  So basically BootNext and BootOrder can override the normal boot selection from that menu without you ever seeing it, but the things that show up in that menu aren't strictly governed by Boot#### variables.

The "option" menu shows files in the following locations on both FAT32 and HFS+ partitions:

- /EFI/BOOT/BOOTX64.EFI (or BOOTIA32.EFI on Macs made from 2006-2007)

- /System/Library/CoreServices/boot.efi

Some machines will also find EFI boot loaders in other locations, but the first two are the only reliable locations.

These files can be placed on any FAT32 or HFS+ partition on USB flash drives or the internal hard drive. However, they will appear in the option menu as "EFI Boot" and do not use the name given by BDS.

=== CDs are wild and crazy ===

CDs can be booted through the option key menu if they're set up properly.  There are also some caveats.

# On some machines, they need to be Apple Partition Map formatted.
# This also means to make a disc that works on EFI+Apple+BIOS as both an ISO image /and/ an image on a USB stick, you need to actually /execute/ the APM header signature on BIOS machines.  Thankfully that's pretty safe, but it's a little squicky in your head.  [http://www.syslinux.org/wiki/index.php/The_Syslinux_Project syslinux]]'s "isohybrid" utility has had this ability to build this sort of image since version 4.05 .
# On some machines, there needs to be a special El Torito image for an HFS+ formatted ESP that conforms to what's above in [http://wiki.osdev.org/index.php?title=Broken_UEFI_implementations#There.27s_additional_UI_presented_mixed_in_with_BDS There's additional UI presented mixed in with BDS] in order to have it show your UEFI-bootable image as anything other than "Windows".  This means there are /three/ El Torito entries - the "Validation Entry" of type 0x00 (80x86), which holds the BIOS code, and then *two* entries of type 0xef - the HFS+ and then the FAT one.  The HFS+ one needs to be last one.  Tools like mkisofs can make ISOs with multiple images with the several boot images using the "-eltorito-alt-boot" and "-e" options.  There's some examples [https://fedoraproject.org/wiki/User:Pjones/BootableCDsForBIOSAndUEFI here].
# The menus will show the BIOS image as "Windows"
# Unfortunately the menus will also show the FAT-based UEFI image as "Windows".
# Some machines will not correctly boot the FAT-based UEFI image.  This means you need to tell your users that when they're trying to boot a CD or USB stick of your product, they need to pick the left-most correctly labeled one with an icon.

It's pretty easy to orchestrate getting all this correct with tools like [https://github.com/rhinstaller/lorax Lorax], which has scripts for doing it on RHEL and Fedora, and can be taught to do it on other things without excessive work.

== GIGABYTE Hybrid EFI ==

These motherboards claim to support other os, however it is not
possible to use efibootmgr to write to nvram. Further, fdisk gpt
partitioned drives with the standard efi system partition type and all
known good default names will not be able to find a stub kernel
image. Strangely they are able to find the /efi/boot/bootx64.efi image
when it is located on a MBR formatted disk (which is how it was
possible to run efibootmgr in the first place). These issues may be
related to issues described here:
http://www.rodsbooks.com/gb-hybrid-efi/.

''Another commenter adds:'' "Hybrid EFI" is gigabyte's name for
running an x64 uefi emulator on top of bios, which does not work at
all well. Some boards with Hybrid EFI offer a beta real EFI upgrade
as a firmware download, which fixes most of the serious issues, but
not all. See
http://www.gigabyte.com/products/product-page.aspx?pid=3979#ov for an
example of such a board. Version F7 is "Hybrid Efi" and doesn't work,
U1G is the real thing but is marked beta.

== Piles and piles of Acer hardware ==

Acer for some reason don't honor the BootOrder variable under most conditions.  This has been an ongoing issue forever, chronicled most recently here: https://github.com/rhinstaller/efibootmgr/issues/19

Typical example (Acer Travelmate B116-M, Insyde BIOS V1.20)

Boots fine from USB UEFI formatted devices, once you disable Secure boot, which can only be done once you've set a bios password.
Installing a Linux-only system shows no errors (including from efibootmgr), but then fails to boot at all. The BIOS shows no boot entries.
Restoring the original windows EFI System Partition from backups "fixes" the problem, on next boot we have a new automagic entry called "Windows Boot Manager". Creating new EFI boot entries after that using efibootmgr silently fails, the system ignores them. The only way to get it to boot something else is to replace '''\EFI\Microsoft\Boot\bootmgfw.efi''' with the distro-provided grubx64.efi.

== MSI motherboards with AMI firmware ==
Some users have reported that on the MSI GE62 2QD Apache Pro (board model: MS-16J2) with AMI firmware, variables set with efibootmgr aren't persistent across reboots.

== Dell Wyse 3040 ==
This thin client will only boot /EFI/BOOT/BOOTX64.efi and ignores which efi file is actually selected in the graphical firmware setup.

== Runtime services accessing boot services memory ==
Many UEFI firmwares have the issue of runtime services accessing boot services memory. There is a series of patches at https://lkml.org/lkml/2014/9/13/103 to detect these bugs without breaking systems. 
(Trivia: https://lkml.org/lkml/2013/11/11/653, which uses a different approach, is written by HP that also locked access to firmware updates to paid customers not long afterwards)
[[Category:UEFI]]
````

<a id="w003808"></a>

## W003808 — User:Moloch

Bron-ID: W003808
Namespace: User (2)
Revisie: 18263 | Datum: 2015-07-28T05:25:46Z
Bron: https://wiki.osdev.org/User:Moloch
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMoloch&oldid=18263

````text
Nothing here...
````

<a id="w003809"></a>

## W003809 — Brendan's Memory Management Guide

Bron-ID: W003809
Namespace: Main (0)
Revisie: 23066 | Datum: 2018-10-31T19:39:58Z
Bron: https://wiki.osdev.org/Brendan%27s_Memory_Management_Guide
Revisie-URL: https://wiki.osdev.org/index.php?title=Brendan%27s_Memory_Management_Guide&oldid=23066

````text

[[Category:Memory management]]
==Overview==

Memory management is something that all OS developers need to deal with sooner or later. It is a very broad topic; and has a tendency to appear very easy at first, but grow into something far more complicated than originally anticipated. This leads to a likelihood of people making design decisions that seem perfectly fine (and work well) initially while the OS is small, where these design decisions become significant problems later on.

In the hope of reducing the chance of design decisions become significant problems later on, "memory management" should be split into 3 separate concepts:
* "Heap" (Dynamic Memory Management)
* Virtual Memory Management
* Physical Memory Management

Please note that this is only a guide, intended to help beginners become aware of various issues and become prepared to handle the complexity that is (eventually) involved. It is not intended to be something that must be followed to the letter, nor is it intended to cover all the possible ways that memory management might be done.

=="Heap" (Dynamic Memory Management)==

This is the highest level layer, and the layer that normal software uses.

In general different processes typically use memory managers designed for whichever language the process is written in (including simple "malloc/free" used by C programs, garbage collection systems in Java, etc). For this reason "heap" typically has nothing to do with the OS itself and is part of a language's run-time. However; in special cases (e.g. where maximum performance/efficiency is needed) a process may bypass the language's generic memory management and implement its own memory management that's custom designed for that application's data structures and usage patterns.

Also; processes tend to allocate and free memory very frequently; and it's a bad idea to call the kernel's API every time an allocation or deallocation is made. For this reason processes allocate a larger amount of virtual memory (from the kernel's virtual memory manager) and then carve it up into smaller pieces as needed.

Essentially; the purpose is to allow processes to use whatever method they like for managing their memory themselves; while avoiding the overhead of excessive use of the kernel's API.

A kernel might also have something for generic dynamic memory management (e.g. "kmalloc()" and "kfree()" that mimics the "malloc/free" that would've been in the C standard library); but might have custom designed allocators for managing specific data structures, or multiple different memory managers for different purposes. However, this is not very different to the memory management done by a process, and the kernel's "heap" would still just allocate larger areas using the virtual memory manager and then carve it up.

Things that tend to complicate this memory manager include:
* satisfying the caller's alignment requirements
* multiple "pools" (e.g. one pool per thread or per CPU, or different pools for different types of data) to improve cache locality/scalability/efficiency
* attributes that effect how memory is allocated (e.g. "allocate fast without caring about fragmentation because it will be freed soon anyway" vs. "take extra time to minimize fragmentation because this is memory will be in use for a long time")
* higher level interface to things like memory mapped files and "memory attributes"
* additional information for debugging and detecting common programmer errors (e.g. freeing the same memory twice)

==Virtual Memory Management==

The purpose of virtual memory management is to manage virtual address spaces; including creating and destroying virtual address spaces and mapping/unmapping "things" into them.

Initially a virtual memory manager can be extremely simple - e.g. when the "heap" for the process (or kernel) asks to make an area of its virtual address space become "usable RAM" the virtual memory manager could just find any pieces of the requested area that aren't already "usable RAM" and allocate physical memory (using the physical memory manager) and map it into the virtual address space; and when the "heap" for the process (or kernel) asks to make an area "not usable RAM" the virtual memory manager could just find any pieces that aren't already "not usable RAM", unmap it, and deallocate it (using the physical memory manager).

However; the virtual memory manager is responsible for a number of "tricks" intended to reduce RAM consumption, improve performance, or provide additional functionality. These tricks may include:
* pretending an area is "usable RAM" when no RAM is allocated initially, and RAM is only allocated if/when the area is actually used (to avoid wasting RAM when the area is allocated by "heap" but not used)
* pretending a file has been loaded from disk into an area, when the file initially wasn't (to improve performance, reduce file IO, and avoid wasting RAM for parts of the file that aren't accessed)
* pretending there's more virtual memory than there is physical memory by transfering data to/from swap space (and discarding pages from memory mapped files where the data can be obtained from the file if needed again) while trying to ensure the most likely to be needed data is in memory (to improve the amount of memory software can use)
* tricks to improve the efficiency of communication between processes (e.g. shared memory areas, moving entire page tables from one virtual address space to another, etc)

Please note that these "tricks" aren't necessarily just for processes alone; and the kernel itself may benefit from them. For example, I use the "pretend area is usable RAM when no RAM is allocated initially" trick a lot in my kernels.

The virtual memory manager is also responsible for maintaining security (ensuring software can't access things it shouldn't be allowed to access, ensuring data that one process freed can't be accessed next time it's allocated, etc).

Things that tend to complicate the virtual memory manager (in addition to the tricks and security issues) include:
* performance and scalability (e.g. lockless/blockless and "O(1)" algorithms)
* lazy TLB invalidation schemes (to reduce overhead on multi-CPU systems)
* support for multiple page sizes
* page/cache coloring and NUMA optimizations (to reduce cache miss costs)
* "compressed data in RAM" as first level swap space (so you can have 5 GiB of data in RAM when there's only 4 GiB of RAM)
* pre-fetching things from disk (memory mapped files and swap space) when RAM becomes free and disks aren't doing much (to reduce overhead later if/when the data is needed)
* tracking statistics (how much memory each process thinks it's using, how much each process is actually using, how many pages fetched from disk per second, etc). This can include how often each page of data is used (used to estimate "most likely to be needed" and decide which page/s to send to swap space)
* notifying other software when RAM is running out (so that things like file system caches, web browser HTML caches, etc. can free their caches) to avoid or reduce the use of swap space
* support for memory mapped IO areas (e.g. mapping video display memory into a virtual address space so a video driver can access it)
* supporting various cachability types (write-through, write-combining, etc) for both normal processes and device drivers (including using PAT to make up for a lack of MTRRs for memory mapped devices)
* higher level interface for special restrictions device drivers may have (e.g. needing buffers of "physically contiguous" memory and/or memory below a certain address)

There is one "beginner mistake" that is common enough to be worth mentioning specifically. The mistake is mapping the physical address space "as is" into kernel space. This prevents the kernel from benefiting from all of the tricks and optimisations that the virtual memory manager does. It can also waste memory for the additional page tables, etc. needed to map things that have no reason to be mapped into kernel space to begin with; and also typically results in "poor locality" (as the data the kernel does need to access is spread throughout data the kernel doesn't need to access). It also creates a massive security risk - any bug in the kernel (or CPU - e.g. Meltdown) may allow an attacker to access everything in RAM (and not just things that actually needed to be mapped in kernel space). Finally it fails when the physical address space is larger than kernel space and results in ugly work-arounds for those cases. Unfortunately, once upon a time a beginner (who didn't know any better at the time) made this mistake, and their kernel grew until it was too late to fix the problem because too many other pieces of their kernel depended on it, and their "still broken" kernel became a well known open source kernel that people look at to understand memory management.

==Physical Memory Management==

The physical memory manager is all about managing the physical address space (in the same way that virtual memory management is for managing virtual address spaces). Part of this is managing free physical RAM; and part of it is managing things that aren't RAM at all (e.g. memory mapped PCI device areas, MTRRs, etc).

By far the most frequent case (and the part that has to be optimized the most) is allocating and freeing individual physical pages of RAM where it makes no difference what the physical address is. I strongly recommend implementing an allocator specifically for this use case, that's built on the concept of fast/"O(1)" free page stack/s.

However, the physical memory manager also has to support special requests (typically from device drivers and nothing else) for physical memory that has specific requirements. These requests for physically contiguous areas, areas that are below a certain physical address, etc. The worst case is for buffers that will be used by legacy/ISA DMA controllers, where the buffer has to be below 0x01000000, must be physically contiguous and must not cross a 64 KiB boundary. Satisfying these requests requires a slower allocator. For this reason it's a good idea to split "usable physical RAM" into 2 or more zones, where some RAM (e.g. RAM below 0x01000000) uses one allocator designed to be able to handle the much less common special requests, and other zones use an allocator designed for much more frequent "allocate/free one page" use case.

Note that these allocators have no need to access the data within free pages, and only allocates/frees them. There is no reason for free pages of RAM to be mapped into any virtual address space.

For managing things that aren't RAM, you need a map of the physical address space that says which areas are used for what. For example, when you're trying to initialize a new PCI device (to configure its "BARs") you want to search for an area that isn't ROM, isn't used by another device and isn't RAM. This information includes "cacheability attributes" and is used to configure (and manage) the CPU's MTRRs. The physical address space map should also contain information for RAM - e.g. is it volatile or non-volatile, is it hot-pluggable, etc.

Things that tend to complicate the physical memory manager (beyond what was mentioned above) include:
* performance and scalability (e.g. lockless/blockless and "O(1)" algorithms)
* support for multiple page sizes
* page/cache coloring and NUMA optimizations (to reduce cache miss costs)
* tracking statistics (how much RAM in each "zone" is used/free)
* fault tolerance (e.g. faulty RAM detection and avoidance)
* power management (e.g. keep all frequently accessed data in some RAM chips, so other RAM chips can spend most of their time in lower power states)
* hot-plug RAM (removal and insertion)
* support for virtual memory ballooning (a technique virtual machines/hypervisors use to reclaim memory guests aren't using)
````

<a id="w003811"></a>

## W003811 — User:Makerimages

Bron-ID: W003811
Namespace: User (2)
Revisie: 18296 | Datum: 2015-08-19T18:49:54Z
Bron: https://wiki.osdev.org/User:Makerimages
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMakerimages&oldid=18296

````text
<div class="floatright">
    http://www.gravatar.com/avatar/496bde6680e9f9f4d8eeffa45baa240d?s=225.png
    <hr>
    <b>GitHub: </b>[https://github.com/makerimages https://github.com/makerimages]
    <b>Twitter: </b> [https://twitter.com/makerimages @makerimages]
    <b>Age: </b> 17
    <b>Location: </b> Estonia
</div>

Ahh, hi there! I'm Makerimages, the creator and main developer of [https://github.com/makerimages/oszin Os Zin]. Work on OS Zin started December 31st, 2014.
````

<a id="w003812"></a>

## W003812 — File:Wataha.png

Bron-ID: W003812
Namespace: File (6)
Revisie: 18297 | Datum: 2015-08-20T06:57:20Z
Bron: https://wiki.osdev.org/File:Wataha.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AWataha.png&oldid=18297

````text

````

<a id="w003813"></a>

## W003813 — TLS

Bron-ID: W003813
Namespace: Main (0)
Revisie: 18302 | Datum: 2015-08-21T12:08:02Z
Bron: https://wiki.osdev.org/TLS
Revisie-URL: https://wiki.osdev.org/index.php?title=TLS&oldid=18302
Doorverwijzing: Thread Local Storage

````text
#REDIRECT [[Thread Local Storage]]
````

<a id="w003814"></a>

## W003814 — Talk:Broken UEFI implementations

Bron-ID: W003814
Namespace: Talk (1)
Revisie: 18308 | Datum: 2015-08-22T01:02:20Z
Bron: https://wiki.osdev.org/Talk:Broken_UEFI_implementations
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABroken_UEFI_implementations&oldid=18308

````text
== Apple ==

I think, that points 2, 3 and 5 are wrong. I've booted EFI loaders without these satisfied. Could that be model-dependent? We need to clarify this and add certain information. --[[User:Roman|Roman]] 20:02, 21 August 2015 (CDT)
````

<a id="w003815"></a>

## W003815 — System Root

Bron-ID: W003815
Namespace: Main (0)
Revisie: 18333 | Datum: 2015-09-06T17:57:56Z
Bron: https://wiki.osdev.org/System_Root
Revisie-URL: https://wiki.osdev.org/index.php?title=System_Root&oldid=18333
Doorverwijzing: Meaty Skeleton

````text
#REDIRECT [[Meaty_Skeleton#System_Root]]
````

<a id="w003816"></a>

## W003816 — User:Catnikita255

Bron-ID: W003816
Namespace: User (2)
Revisie: 18350 | Datum: 2015-09-27T11:54:11Z
Bron: https://wiki.osdev.org/User:Catnikita255
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACatnikita255&oldid=18350

````text
My OS:
[os365dev.tk]
That's all.
````

<a id="w003817"></a>

## W003817 — User:Jarno

Bron-ID: W003817
Namespace: User (2)
Revisie: 18353 | Datum: 2015-09-30T09:48:37Z
Bron: https://wiki.osdev.org/User:Jarno
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJarno&oldid=18353

````text
I'm a student at the University of Ghent, in computer sciences.

I really like to study and experiment with low level software, and have a lot of interest in hardware.

That's it for now, short on time ;)
````

<a id="w003818"></a>

## W003818 — HDAudio

Bron-ID: W003818
Namespace: Main (0)
Revisie: 18390 | Datum: 2015-10-13T13:59:24Z
Bron: https://wiki.osdev.org/HDAudio
Revisie-URL: https://wiki.osdev.org/index.php?title=HDAudio&oldid=18390
Doorverwijzing: Intel High Definition Audio

````text
#REDIRECT [[Intel High Definition Audio]]
````

<a id="w003819"></a>

## W003819 — HD Audio

Bron-ID: W003819
Namespace: Main (0)
Revisie: 18391 | Datum: 2015-10-13T14:00:03Z
Bron: https://wiki.osdev.org/HD_Audio
Revisie-URL: https://wiki.osdev.org/index.php?title=HD_Audio&oldid=18391
Doorverwijzing: Intel High Definition Audio

````text
#REDIRECT [[Intel High Definition Audio]]
````

<a id="w003820"></a>

## W003820 — Fractal Page Mapping

Bron-ID: W003820
Namespace: Main (0)
Revisie: 29409 | Datum: 2025-02-17T09:30:32Z
Bron: https://wiki.osdev.org/Fractal_Page_Mapping
Revisie-URL: https://wiki.osdev.org/index.php?title=Fractal_Page_Mapping&oldid=29409

````text
{{Stub}}
{{TutorialTone}}
A common problem related to paging is "How do you access the paging structures to change mappings?"

Fractal mapping uses the orthogonality of page tables on most architectures (notably x86 and x86_64) to make a region of memory automatically map the paging structures

= How to do it =

On 32-bit x86, all you need to do is set an entry in the page table to the address of the page table (with appropriate access bits). In examples, the last entry is typically used, but in practice any can be used
<syntaxhighlight lang="c">
page_directory[1024 - 1] = get_phys_addr(&page_directory[0]) | PAGE_PRESENT | PAGE_WRITABLE;
</syntaxhighlight>

Now, when you access the last 4MB of the physical address space, the CPU will use the page directory as a page table. This leads to the last 4KB of memory being the page directory.

[[Category:Paging]]
````

<a id="w003821"></a>

## W003821 — User:D3crypt

Bron-ID: W003821
Namespace: User (2)
Revisie: 21025 | Datum: 2017-07-30T20:16:43Z
Bron: https://wiki.osdev.org/User:D3crypt
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AD3crypt&oldid=21025

````text

== Articles I have written ==
Nothing yet :)
````

<a id="w003822"></a>

## W003822 — Hdaudio

Bron-ID: W003822
Namespace: Main (0)
Revisie: 18439 | Datum: 2015-11-03T22:24:07Z
Bron: https://wiki.osdev.org/Hdaudio
Revisie-URL: https://wiki.osdev.org/index.php?title=Hdaudio&oldid=18439
Doorverwijzing: Intel High Definition Audio

````text
#REDIRECT [[Intel High Definition Audio]]
````

<a id="w003823"></a>

## W003823 — Go Bare Bones

Bron-ID: W003823
Namespace: Main (0)
Revisie: 30607 | Datum: 2026-05-22T14:04:10Z
Bron: https://wiki.osdev.org/Go_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Go_Bare_Bones&oldid=30607

````text
{{BadPractice}}
{{BeginnersWarning}}
{{Rating|2}}

In this tutorial you'll learn how to get started using the [[Go]] language to write your own OS. It will be an example of how to create a very minimal system to get text on the screen. It's in no way an example of how you should organize or structure your project.

== Preface ==

It is highly recommended to first read [[Bare Bones]] before following this tutorial because it is based on it. Also please keep in mind that the Go language itself was not designed for operating systems development. As such you'll be fighting the language every step of the way.

This tutorial assumes you are using a Unix-like operating system such as Linux.

For a complete production framework supporting OS development in Go see the [https://github.com/usbarmory/tamago TamaGo project].

== Building a Cross-Compiler ==
{{Main|GCC Cross-Compiler}}

The first thing you should do is set up a GCC Cross Compiler that supports Go. To do this read and follow [[GCC Cross-Compiler]] to the letter with one exception. When configuring the build for GCC we need to enable Go to get the i686-elf-gccgo compiler.

So instead of using: <syntaxhighlight lang="bash">../gcc-x.y.z/configure --target=$TARGET --prefix="$PREFIX" --disable-nls --enable-languages=c,c++ --without-headers</syntaxhighlight> We use: <syntaxhighlight lang="bash">../gcc-x.y.z/configure --target=$TARGET --prefix="$PREFIX" --disable-nls --enable-languages=c,c++,go --without-headers</syntaxhighlight>

== Overview ==

By now, you should have set up your [[GCC Cross-Compiler|cross-compiler]] for i686-elf that supports Go (as described above). This tutorial provides a minimal solution for creating an operating system for x86. It doesn't serve as a recommend skeleton for project structure, but rather as an example of a minimal kernel using another package to put text on the screen. In this simple case, we just need four input files:

* boot.s - kernel entry point that sets up the processor environment
* kernel.go - your actual kernel routines
* terminal.go - a package for printing text to the screen
* linker.ld - for linking the above files

== Bootstrap Assembly ==

We will now create the file called boot.s and discuss its contents a little.

<syntaxhighlight lang="asm">
# Declare constants used for creating a multiboot header.
.set ALIGN,    1<<0             # align loaded modules on page boundaries
.set MEMINFO,  1<<1             # provide memory map
.set FLAGS,    ALIGN | MEMINFO  # this is the Multiboot 'flag' field
.set MAGIC,    0x1BADB002       # 'magic number' lets bootloader find the header
.set CHECKSUM, -(MAGIC + FLAGS) # checksum of above, to prove we are multiboot

# Declare a header as in the Multiboot Standard.
.section .multiboot
.align 4
.long MAGIC
.long FLAGS
.long CHECKSUM

# Currently the stack pointer register (esp) points at anything and using it may
# cause massive harm. Instead, we'll provide our own stack.
.section .bootstrap_stack, "aw", @nobits
stack_bottom:
.skip 16384 # 16 KiB
stack_top:

# The linker script specifies _start as the entry point to the kernel and the
# bootloader will jump to this position once the kernel has been loaded. It
# doesn't make sense to return from this function as the bootloader is gone.
.section .text
.global _start
.type _start, @function
_start:
	# To set up a stack, we simply set the esp register to point to the top of
	# our stack (as it grows downwards).
	movl $stack_top, %esp

	# We are now ready to actually execute Go code. Functions that start with a
	# capital letter are exported as "go.<package>.<function>".
	call go.kernel.Main

	# In case the function returns, we'll want to put the computer into an
	# infinite loop.
	cli
	hlt
.Lhang:
	jmp .Lhang

# Set the size of the _start symbol to the current location '.' minus its start.
# This is useful when debugging or when you implement call tracing.
.size _start, . - _start

# The Go runtime is a big problem when wanting to write a kernel in Go.
# Although not impossible, it's certainly not trivial. At the very least
# you'd need to implement parts of libc that it uses. Porting the runtime
# goes way beyond the scope of this barebone. 
#
# Beware that there are almost no language features you can use at this point.
# The linker will fail with missing symbols when you try to use them.
#
# You have to implement those missing symbols yourself to get things working.
# Sometimes you can get away with declaring them as an empty function. 
# But this won't always work.
#
# After you get this bare bone working, the first priority should be to write
# your own memory allocator. A simple sbrk implementation should suffice for
# symbols like __go_new.
#
# For now we just implement the symbols below as empty functions to get this
# barebone up and running.
#

.global __go_register_gc_roots
.type __go_register_gc_roots, @function
__go_register_gc_roots:
	ret
.size __go_register_gc_roots, . - __go_register_gc_roots

.global __go_runtime_error
.type __go_runtime_error, @function
__go_runtime_error:
	ret
.size __go_runtime_error, . - __go_runtime_error
</syntaxhighlight>

You can then assemble boot.s using:
<syntaxhighlight lang="bash">i686-elf-as boot.s -o boot.o</syntaxhighlight>

== Writing the Terminal package in Go ==
:''Please read [[Printing to Screen]] to understand what this code does.

Now we'll create the file terminal.go. It's the package "terminal" our kernel will depend on for printing text to the screen. 
<syntaxhighlight lang="Go">
package terminal

import "unsafe"

/*
 * Map the text mode video memory into a multi-dimensional array that can be safely
 * used from Go.
 */

func get_vidMem(addr uint32) *[25][80][2]byte {
	buff := (*[25][80][2]byte)(unsafe.Pointer(uintptr(addr)))
	return buff	
}

/*
 * This isn't a tutorial on the language itself so the rest should make sense to you.
 */

const (
	Black        = 0
	Blue         = 1
	Green        = 2
	Cyan         = 3
	Red          = 4
	Magenta      = 5
	Brown        = 6
	LightGrey    = 7
	DarkGrey     = 8
	LightBlue    = 9
	LightGreen   = 10
	LightCyan    = 11
	LightRed     = 12
	LightMagenta = 13
	LightBrown   = 14
	White        = 15
)

var Column, Row int
var Color byte
var vidMem *[25][80][2]byte

func Init() {
	vidMem = get_vidMem(0xB8000)
	Color = MakeColor(LightGrey, Black)
	Column = 0
	Row = 0
}

func MakeColor(fg, bg byte) byte {
	return fg | bg<<4
}

func ScrollUp() {
	for x := 1; x < 25; x++ {
		vidMem[x-1] = vidMem[x]
	}
	Row = 24
}

func Clear() {
	for r := 0; r < 25; r++ {
		for c := 0; c < 80; c++ {
			vidMem[r][c][0] = 32
			vidMem[r][c][1] = Color
		}
	}
}

func Poke(c rune) {
	if c == '\n' {
		Column = 0
		Row++
		if Row > 24 {
			ScrollUp()
		}
	} else {
		vidMem[Row][Column][0] = byte(c)
		vidMem[Row][Column][1] = Color
		Column++
		if Column > 79 {
			Column = 0
			Row++
			if Row > 24 {
				ScrollUp()
			}
		}
	}
}

func Print(s string) {
	for c := 0; c < len(s); c++ {
		Poke(rune(s[c]))
	}
}
</syntaxhighlight>

Compile using: <syntaxhighlight lang="bash">i686-elf-gccgo -static -Werror -nostdlib -nostartfiles -nodefaultlibs -c terminal.go -o terminal.go.o</syntaxhighlight>

== How imported packages are found ==

When you import a package with gccgo it will look for the import data in the following files and will use the first one it finds.

* .gox
* .so
* .a
* .o

A .gox file contains just the import data. If you wanted to extract it from our terminal.go.o object file we use: <syntaxhighlight lang="bash">i686-elf-objcopy -j .go_export terminal.go.o terminal.gox</syntaxhighlight>

== Writing a kernel in Go ==

Now we create the file kernel.go that contains the Main() function called from our bootstrap assembly we've already created and compiled above. It will import the terminal package we also created & compiled. Then we use that package to print text to the screen.

<syntaxhighlight lang="go">
package kernel

import "terminal"

func Main() {
	terminal.Init()                      // Initialize terminal
	terminal.Color = terminal.MakeColor( // Set color to..
		terminal.White, // White text...
		terminal.Blue)  // on a blue background
	terminal.Clear() // Clear screen

	// Center the text a little
	terminal.Row = 11
	terminal.Column = 30

	// Print our Hello, World!
	terminal.Print("Hello, Kernel World!\n")
}
</syntaxhighlight>

Compile using: <syntaxhighlight lang="bash">i686-elf-gccgo -static -Werror -nostdlib -nostartfiles -nodefaultlibs -c kernel.go -o kernel.go.o</syntaxhighlight>

== Linking the Kernel ==

Now we create the file linker.ld which will be used as instructions to link the object files we have into the final kernel program, usable by the bootloader.

<pre>
/* The bootloader will look at this image and start execution at the symbol
   designated as the entry point. */
ENTRY(_start)

/* Tell where the various sections of the object files will be put in the final
   kernel image. */
SECTIONS
{
	/* Begin putting sections at 1 MiB, a conventional place for kernels to be
	   loaded at by the bootloader. */
	. = 1M;

	/* First put the multiboot header, as it is required to be put very early
	   early in the image or the bootloader won't recognize the file format.
	   Next we'll put the .text section. */
	.text BLOCK(4K) : ALIGN(4K)
	{
		*(.multiboot)
		*(.text)
	}

	/* Read-only data. */
	.rodata BLOCK(4K) : ALIGN(4K)
	{
		*(.rodata)
	}

	/* Read-write data (initialized) */
	.data BLOCK(4K) : ALIGN(4K)
	{
		*(.data)
	}

	/* Read-write data (uninitialized) and stack */
	.bss BLOCK(4K) : ALIGN(4K)
	{
		*(COMMON)
		*(.bss)
		*(.bootstrap_stack)
	}

	/* The compiler may produce other sections, by default it will put them in
	   a segment with the same name. Simply add stuff here as needed. */
}
</pre>

You can link your kernel using:
<syntaxhighlight lang="bash">i686-elf-gcc -T linker.ld -o myos.bin -ffreestanding -O2 -nostdlib boot.o terminal.go.o kernel.go.o -lgcc</syntaxhighlight>

The file myos.bin is now your kernel. Note that we are linking against [[libgcc]], which implements various routines that your cross-compiler depends on. Leaving it out will give you problems in the future.

== Testing your operating system (QEMU) ==

Virtual Machines are very useful for developing operating systems, as they allow you to quickly test your code and have access to the source code during the execution. Otherwise, you would be in for an endless cycle of reboots that would only annoy you. They start very quickly, especially combined with small operating systems such as ours.

QEMU supports booting multiboot kernels directly without using a bootable medium:
<syntaxhighlight lang="bash">qemu-system-i386 -kernel myos.bin</syntaxhighlight>

== External Links ==

* [https://www.gnu.org/software/grub/manual/multiboot/multiboot.html Multiboot Specification]
* [https://golang.org/doc/install/gccgo Setting up and using gccgo]

[[Category: Bare bones tutorials]]
[[Category:Inadvisable]]
````

<a id="w003824"></a>

## W003824 — Execution Loader

Bron-ID: W003824
Namespace: Main (0)
Revisie: 27879 | Datum: 2023-07-09T04:22:45Z
Bron: https://wiki.osdev.org/Execution_Loader
Revisie-URL: https://wiki.osdev.org/index.php?title=Execution_Loader&oldid=27879

````text
{{Stub}}

An '''Execution Loader''' or '''program loader''' is a program or operating system service which parses an executable [[Object File]] and loads (or maps) the executable section into memory. It may have additional functions as well, such as:

* Determining if there is a copy of the program already loaded or mapped into memory, and mapping the executable section of the process to share the existing memory footprint;
* Editing the executable image to patch addresses into the code, either to allow relocation, or to map external references to shared (dynamic) libraries,
* Loading shared libraries into memory, and caching them to avoid reloading.

[[Category:Loaders]]
````

<a id="w003825"></a>

## W003825 — Object File

Bron-ID: W003825
Namespace: Main (0)
Revisie: 18452 | Datum: 2015-11-07T02:32:24Z
Bron: https://wiki.osdev.org/Object_File
Revisie-URL: https://wiki.osdev.org/index.php?title=Object_File&oldid=18452
Doorverwijzing: Object Files

````text
#redirect[[Object Files]]
````

<a id="w003826"></a>

## W003826 — File:OVMF shell.png

Bron-ID: W003826
Namespace: File (6)
Revisie: 18511 | Datum: 2015-11-11T14:32:11Z
Bron: https://wiki.osdev.org/File:OVMF_shell.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOVMF_shell.png&oldid=18511

````text

````

<a id="w003827"></a>

## W003827 — File:OVMF browse fs.png

Bron-ID: W003827
Namespace: File (6)
Revisie: 18516 | Datum: 2015-11-11T15:45:53Z
Bron: https://wiki.osdev.org/File:OVMF_browse_fs.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOVMF_browse_fs.png&oldid=18516

````text

````

<a id="w003828"></a>

## W003828 — File:OVMF run app.png

Bron-ID: W003828
Namespace: File (6)
Revisie: 18520 | Datum: 2015-11-11T16:53:32Z
Bron: https://wiki.osdev.org/File:OVMF_run_app.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOVMF_run_app.png&oldid=18520

````text

````

<a id="w003829"></a>

## W003829 — Vagrant

Bron-ID: W003829
Namespace: Main (0)
Revisie: 27971 | Datum: 2023-07-09T16:31:44Z
Bron: https://wiki.osdev.org/Vagrant
Revisie-URL: https://wiki.osdev.org/index.php?title=Vagrant&oldid=27971

````text
{{Stub}}
Vagrant puts standard operating system installs into a container. Vagrant integrates with either VMWare or VirtualBox. Then when a container is downloaded and run for the first time vagrant will perform actions specified in your Vagrantfile. In this way development environments can be easily shared. Once the virtual machine created from the container is started with Vagrant. Vagrant will automatically start an ssh session. The folder in which you start Vagrant has to contain a Vagrantfile. This folder will also be shared with the virtual machine.

== Example ==
<source lang="ruby">
# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure(2) do |config|
  config.vm.box = "ubuntu/trusty32"
  config.vm.provision "shell", inline: <<-SHELL
    sudo apt-get update
    sudo apt-get install -y libgmp3-dev libmpfr-dev libisl-dev libcloog-isl-dev libmpc-dev texinfo make bison flex gcc g++ nasm build-essential grub qemu zip xorriso
    sudo export PREFIX="/opt/cross"
    sudo export TARGET=i686-elf
    sudo export PATH="$PREFIX/bin:$PATH"
    sudo mkdir /tmp/
    sudo mkdir /tmp/build
    sudo cd /tmp/build
    sudo wget -O binutils.tar.gz http://ftp.gnu.org/gnu/binutils/binutils-2.25.tar.gz
    sudo tar -xf binutils.tar.gz
    sudo cd binutils-2.25
    sudo mkdir build-binutils
    sudo cd build-binutils
    sudo ../configure --target=$TARGET --prefix="$PREFIX" --with-sysroot --disable-nls --disable-werror
    sudo make
    sudo make install
    sudo cd ../..
    sudo rm -r binutils-2.25
    sudo wget -O gcc.tar.gz http://ftp.gnu.org/gnu/gcc/gcc-4.9.2/gcc-4.9.2.tar.gz
    sudo tar -xf gcc.tar.gz
    sudo cd gcc-4.9.2
    sudo mkdir build-gcc
    sudo cd build-gcc
    sudo ../configure --target=$TARGET --prefix="$PREFIX" --disable-nls --enable-languages=c,c++ --without-headers
    sudo make all-gcc
    sudo make all-target-libgcc
    sudo make install-gcc
    sudo make install-target-libgcc
    sudo rm -rf /tmp/build
  SHELL
end
</source>
This example Vagrantfile will download a Ubuntu 14.04 32-bit container and install a i686-elf-gcc cross compiler. It will also download several other tools usefull for operating system development.

[[Category:OS Development]]
````

<a id="w003831"></a>

## W003831 — Java For Starters

Bron-ID: W003831
Namespace: Main (0)
Revisie: 27951 | Datum: 2023-07-09T16:06:01Z
Bron: https://wiki.osdev.org/Java_For_Starters
Revisie-URL: https://wiki.osdev.org/index.php?title=Java_For_Starters&oldid=27951

````text
{{Disputed|topic:29802}}

==Introduction==

Java is a very nice language, it is very easy to learn and also it's used in a lot of projects by millions of programmers, so it seems a good idea to show a way of OS development for all these Java developers and other people interested in Java. But before you start it would be nice to familiarize yourself with the basic OS concepts, [[Main_Page | described in this wiki]].

==The problem with OS development in Java==

The main problem for people writing an OS in a language without flexible native tools is the lack of the tools. Also Java program needs Java Runtime Environment (JRE) to be run. But during OS startup there's no JRE and we should somehow manage to run Java program without it. As we can see, there's a need in tools and runtime support now. More information on using different languages in OS development is present [[Languages | here]].

==How to run you without a car?==

Well, it's simple - you can use your legs. But yes, it's not very convenient. And it's better to have some kind of a vehicle to help you to get some speed. That's why it is important to understand what tools we have.

First, it is possible to use some combination of tools, written in C or Assembly, with some source code, written according to the Java syntax, and another part of code, written in C or Assembly. It's actually the way many people prefer, [[Pascal_Bare_Bones | and here is the Pascal example]], which shows such approach. But it should be noted, there's just a small part of the job done in Pascal. Without tools, written in other languages, there will be no Pascal OS. Also, without code in other languages (mostly assembly) there will be no Pascal OS.

Second, it's possible to write absolutely everything in the preferred language. And of course, it means somebody should write the "everything". Because in case of many languages there's no native tools available, it's the OS developer who should write "everything".

And third, most realistic way. An OS can be written using as much tools in the target language, as available, and all the rest can be borrowed from other languages and environments. It's something in between of the first and the second ways.

==Here are some tools available for Java OS developers.==

First, of course, it's the standard tool set [[Getting_Started#Choosing_your_development_environment | as described here]]. It supports the first way of doing things - just some language flavor around the same C and Assembly based environment. And the flavor can be compiled by the [https://gcc.gnu.org/java/ GNU Compiler for the Java] (GCJ), for example. The GCJ was written in C and supports only standard C related conventions. If a developer prefers this way it should be noted this article is not intended to be of a great help for such approach.

The second option (everything in Java) is described below. But it introduces some tools, already written in Java. The tools here are used as an example and there's no attempt to enforce you to use them. Following article shows how to create a simple bootloader in Java. There is a jump in the end of the bootloader code and all your potential Java OS can start after the jump is performed. So, your OS in Java can start with this article and end in some very interesting form, unseen until you'll have it done.

==A bit of theory==

Basically it is required to load an OS for it's code to be executed. Because we are talking about Java then it seems a good idea to understand first how Java can load itself (or an OS written in Java). There is a number of ways.
* First, you can use tools like [https://www.gnu.org/software/grub/ GRUB] and forget about loading OS yourself.
* Second, you can write a bootloader in another language, but not in Java, like assembly, for example.
* And third, you can write a bootloader in Java.

The third option is described here. In fact, when the bootloader is actually written in Java, there's some subtle aspect related to the syntax involved. Somebody can say "it's not Java". Well, it's yes and no. But now you have an opportunity to decide it yourself.

First we have to look in the depth of the software execution. For the software to be executed there should be a processor which is able to understand software's instructions. Usually the code you see in many languages is not understandable by the processor. So, there's a translation layer between the code and the executable software. But the translation can be implemented in many forms. First it was the assembly. People had written short textual commands and software was required to translate the commands in the form the processor understands. Next people understood it's very tedious and time consuming to write software in such a form of short textual commands, so the high level languages were born. And as we know, Java is the one. But the translation level between Java and the actually executed code hides a lot of details. It is done for a reason. It helps a developer to concentrate on the goal instead of gory details. And with the goal achieved faster we, unfortunately, lose some flexibility the low level commands have. And finally people managed to mix the high level with the low level in a consistent manner. It was called "inline assembly". It's not ideal and there's still a way for better compilers with the ability to understand such a mixture without requiring a programmer to jump between two languages. But unfortunately, such compilers are still far away from Java developers. However, the idea of inline assembly works perfectly even in Java.

Now we can try to look at the way Java can inline the assembly. First, of course, there should be [http://jembryos.org/inline.html the assembly for Java]. It's not the only possible implementation, but other variants are far too different from the assembly syntax. Because the assembly syntax is widely used in the hardware documentation and software examples it is a good idea to have the inline version which is as close to the generally accepted assembly syntax as possible.

And as we mentioned above there can be voices like "it's not Java!", we have to clear the subject a bit. If we remember the way a program goes along before being executed we can see some intermediary steps. It's the translation phase. The translation phase hides low level details and produces an executable file. But if we forget about the translation then it really can look like this - we write a program and (after some magic applied) it just runs. Yes, we can forget about translation and be happy with the actually running program. But the OS development is a thing that just requires us to go into the low level details. And if we go then it becomes obvious there's no more "the program" and there are just some bytes, produced by tools and executed by a processor. The actual execution details can be very different from what we expect while looking at the initial program. And the following information (hopefully) can show how things actually work between the layer of the programming language abstraction and the hardware layer. It's not Java or not Java talk, it's just way of doing things in Java. The same is true for every C or assembly program - there just must be some tools for the translation phase to work. And the tools can be written in C or assembly or (yes!) in Java.

==How it works.==

First, it's still a Java program. So, the Java syntax should be respected. But Java syntax allows us a lot of flexibility. And the flexibility was used as such - every assembly instruction was defined in a Java class and instance of this class with the same name, as is used in assembly, is referenced from a successor of the AssemblerProgram class. As a result you can write a standard Java method and use assembly instructions in it.

But it's not the actual "inline" assembly now. What the [http://www.jembryos.org/inline.html inline assembly project] does is the translation '''and''' connection between Java code and the translated assembly code. Here we can see just the translation phase. It's actually the collection of assembly instructions step, which allows to translate the instructions in a binary form later (with the help of the ''getProgram()'' method). And the actual way of connection of translated assembly instructions with the Java code is outlined in the end of the article.

The collection of assembly instructions can look like this:
<source lang="java">
mov.x32(EAX,EDX);
</source>

* Here the "mov" is the class's field name.
* The "x32" is the Java method, but it also is used as a way to show us the instruction operand size (32 bits here).
* The "EAX" and "EDX" are also fields of the ancestor class and represent well known in the assembly syntax registers.

The code line above tells us that we have the 32 bit move assembly instruction with operands in EAX (destination) and EDX (source) registers. hopefully, any developer with assembly knowledge can understand this code. But Java adds here type safeness, which (if you use IDE like Eclipse) can highlight errors like wrong register name for selected operand size right at the moment you are editing the code.

Next it is required to understand how the instruction above can be executed. For it to be possible the translation phase is required. But before the translation step it is usually a good idea to have the actual program. The program actually is just a number of text strings in the form described above. It can be good to separate code fragments intended for different purposes in separate methods like this:
<source lang="java">
protected void writeBootCodeInitialization(short stackAddress, short bootLoaderAddress)
{
	eflags.cld(); // eflags here is the common place for a number of flag manipulaton related instructions
	mov.x16(AX,(short)(bootLoaderAddress>>>4)); // mov WORD AX, immediate
	mov.x16(DS,AX); // ordinary 16 bit mov instruction
	xor.x16(AX,AX);
	mov.x16(ES,AX);
	mov.x16(SS,AX);
	mov.x16(SP,stackAddress);
}
</source>
And the class, where code fragments are supposed to be, can look like this:
<source lang="java">
public class AnAssemblyWriter extends AssemblerProgram
{
	public AnAssemblyWriter(Mode mode, DebugStream debugStream)
	{ super(mode, debugStream); }
	
	... // your code fragment methods
}
</source>
Here we see the class's constructor with two parameters, the mode parameter can be one of the following:
  	CODE16 // Generate 16-bit code
  	CODE32 // Generate 32-bit code
  	CODE64 // Generate 64-bit code

And the debugStream is just a wrapper around an arbitrary OutputStream. It can be used to print the resulting assembly program in textual form. The actual print is performed during AssemblerProgram's getProgram() call.

Next you can add a particular code to the bootloader for it to be able to actually load something. You can write a method like this:
<source lang="java">
protected void loadImageUsingInt13x42(String problemLabel, String diskAddressPacketLabel, int imageSizeInDiskSectors)
{
	String startTransfer="startTransfer", done="doneInt13x42";
	if (imageSizeInDiskSectors>BootstrapConstants.int13x42NumberOfBlocksLimit)
	{
		mov.x16(AX,(short)(imageSizeInDiskSectors-BootstrapConstants.int13x42NumberOfBlocksLimit));
		mov.x16(SI,diskAddressPacketLabel);
		label(startTransfer);
		push.x16(AX);
	}
	else mov.x16(SI,diskAddressPacketLabel);
	mov.x8(AH,(byte)0x42);
	mov.x8(DL,(byte)0x80);
	Int.x((byte)0x13);
	jcc.jc(problemLabel);
	if (imageSizeInDiskSectors>BootstrapConstants.int13x42NumberOfBlocksLimit)
	{
		mov.x16(SI,diskAddressPacketLabel);
		add.x16(i(SI,6),(short)(BootstrapConstants.int13x42NumberOfBlocksLimit<<5));
		add.x32(i(SI,8),BootstrapConstants.int13x42NumberOfBlocksLimit);
		pop.x16(AX);
		sub.x16(AX,(short)BootstrapConstants.int13x42NumberOfBlocksLimit);
		jcc.jae(startTransfer);
		add.x16(AX,(short)BootstrapConstants.int13x42NumberOfBlocksLimit);
		jcc.jz(done);
		mov.x16(i(SI,2),AX);
		xor.x16(AX,AX);
		jmp.near(startTransfer);
		label(done);
	}
}
</source>
The the final program (it's main method) can look like this:
<source lang="java">
private void createBootCode(int imageStartAbsoluteBlockNumber, int imageSizeInDiskSectors,
		short memoryAddressToPlaceBootstrapImageAt, short bootLoaderNewAddress,
		short bootLoaderInitialAddress, short bootLoaderSize)
{
	// define labels in one place for not to copy the same text in many places
	String diskReadProblemLabel="diskProblem", diskAddressPacketLabel="diskAddressPacket";

	// call a few methods to include separate code fragments in the resulting program
	writeBootCodeInitialization(memoryAddressToPlaceBootstrapImageAt,bootLoaderNewAddress);
	loadImageUsingInt13x42(diskReadProblemLabel, diskAddressPacketLabel, imageSizeInDiskSectors);

	mov.x16(AX,memoryAddressToPlaceBootstrapImageAt); // address to jump to
	// in fact this line is just for demonstration, because the actual address should include the segment and developers should keep it in mind
	jmp.near16(AX); // jump to the offset in AX and start loaded image code

	writeProceduresAndData(diskReadProblemLabel,diskAddressPacketLabel,
			imageSizeInDiskSectors,imageStartAbsoluteBlockNumber,memoryAddressToPlaceBootstrapImageAt);
}
</source>
And here you can see how helper procedures and data structures are written together with the actual bootloader's code.
<source lang="java">
private void writeProceduresAndData(String diskReadProblemLabel, String diskAddressPacketLabel,
		int imageSizeInDiskSectors, int imageStartAbsoluteBlockNumber, short memoryAddressToPlaceBootstrapImageAt)
{
	String printMsgProcLabel="printMsgLabel", diskErrorMsgLabel="diskErrorMsg";

	label(diskReadProblemLabel); // assembly label analog
	// it's inline because it's not supposed to be called, only jumps here are expected
	inline_printMessage_haltIfSecondZero(diskErrorMsgLabel,printMsgProcLabel);

	writePrintMessageProcedure(printMsgProcLabel); // it is supposed to ret in the end

	label(diskErrorMsgLabel);
	// the zeros at the end denote the end of string and the halt after shown flag
	writeString("Disk error\0\0"); // just write the string as a sequence of bytes

	label(diskAddressPacketLabel); // int 13*42 data structure follows here
	int imageSize=(imageSizeInDiskSectors>int13x42NumberOfBlocksLimit?int13x42NumberOfBlocksLimit:imageSizeInDiskSectors);
	byte bs[]=new byte[16]; // data structure buffer
	int memoryAddressToPlaceImageAt_offset=memoryAddressToPlaceBootstrapImageAt&0xf;
	int memoryAddressToPlaceImageAt_segment=memoryAddressToPlaceBootstrapImageAt>>4;
	@SuppressWarnings("resource") // this stream requires no close() call
	LittleEndianOutputStream leos=new LittleEndianOutputStream(bs); // it can write bytes to the bs array
	leos.w8(0x10); // write byte
	leos.w8(0);
	leos.w16(imageSize); // write short (16 bit)
	leos.w16(memoryAddressToPlaceImageAt_offset);
	leos.w16(memoryAddressToPlaceImageAt_segment);
	leos.w32(imageStartAbsoluteBlockNumber); // write int (32 bit)
	leos.w32(0);
	writeBytes(bs); // now add the bs buffer to the actual program
}
</source>
After you have all inline assembly instructions in place you can first call the methods with the instructions for them to write their binary representation and next call AssemblerProgram's getProgram() method to get the actual machine code stored in the returned byte array. The resulting bytes can be written to the boot sector using appropriate tools.

And, of course, this example shouldn't be considered as a good bootloader. It's just an example of a very simple bootloader, written in Java, which is able to load in memory a small image of 16-bit code and jump to it (do not forget offset register values).

==What's next.==

For the code above to work it is required to [http://sourceforge.net/projects/jembryos/ download some code]. But the general direction of next step is towards the OS logic being defined as a more familiar Java program. The inline assembly shouldn't be used if there's no other way to define required logic. For it to be the case we need the translation phase. It is possible to compile Java code using GCC (GCJ) compiler, but it leads us to the area of C programmers with it's code formats, call conventions and a lot of compiler settings. The second option is to invent your own compiler. It's not so hard and in it's simplest form can be done just by translation of every Java bytecode in the assembly form. Example of such simple compiler also is included in the download, mentioned above. Beside of the translation the compiler allows to inline some assembly code and to invoke it from pure Java program. Also it worth to note the AssemblerProgramm class is able to produce 32 and 64 bit code and if you look at the package org.jembryos.boot.image._native.x86, and the BootRecordCode class (part of jEmbryoBootstrap project), then it is possible to find some examples that can help you to understand the tool better.

[[Category:Bare bones tutorials]]
[[Category:Java]]
````

<a id="w003832"></a>

## W003832 — Intel I/O Controller Hub

Bron-ID: W003832
Namespace: Main (0)
Revisie: 30196 | Datum: 2026-03-11T04:01:42Z
Bron: https://wiki.osdev.org/Intel_I/O_Controller_Hub
Revisie-URL: https://wiki.osdev.org/index.php?title=Intel_I%2FO_Controller_Hub&oldid=30196

````text
{{Stub}}

The Intel I/O Controller Hub line is a popular family of products that combine several I/O devices into a single chipset, which reduces the cost of designing and manufacturing modern PCs.  They typically include Power Management, a Network Adapter, Storage Controllers, USB Controllers, an SMBus Controller, an Audio Device and a Modem Device.

== References ==

* [https://web.archive.org/web/20080218191605/http://download.intel.com/design/chipsets/datashts/29065503.pdf Intel ICH0 Datasheet (This version is used by VirtualBox)]
* [https://web.archive.org/web/20141209220511/http://www.intel.com/content/www/us/en/chipsets/82801db-io-controller-hub-4-datasheet.html Intel ICH4 Datasheet]

[[Category:Sound]]
[[Category:Network Hardware]]
[[Category:X86]]
````

<a id="w003833"></a>

## W003833 — Debugging UEFI applications with GDB

Bron-ID: W003833
Namespace: Main (0)
Revisie: 30286 | Datum: 2026-03-16T18:57:21Z
Bron: https://wiki.osdev.org/Debugging_UEFI_applications_with_GDB
Revisie-URL: https://wiki.osdev.org/index.php?title=Debugging_UEFI_applications_with_GDB&oldid=30286

````text

== Debugging a POSIX-UEFI Bootloader under QEMU ==

This article describes how to debug a [[POSIX‑UEFI]] bootloader (or any EFI app) under QEMU without hard‑coding a load address. By having your [[UEFI|EFI]] binary write a “magic marker” and its runtime base address into known RAM locations, you can set a watchpoint and then report to GDB exactly where your code lives in memory, then load symbols automatically.

== QEMU Setup ==

* Use the <code>-serial</code> option to have a serial console available for the virtual machine. You will have console logs in your terminal and a possibility to use simple ports writing to output debug tracing to serial console.
* Use the <code>-s</code> option to enable the built-in GDB stub which will wait for connection on TCP port 1234.
* Use the <code>-S</code> option to pause the virtual CPU at startup until a debugger is connected.

Launch QEMU as usual, providing the path to your firmware and disk image. For example:
<syntaxhighlight lang="bash">
qemu-system-x86_64 -drive file=boot.img
-bios firmware/OVMF.fd
-m 256M
-smp 4
-machine q35
-net none
-s -S
-serial mon:stdio
</syntaxhighlight>

QEMU will start and wait for an incoming GDB connection on port 1234. The VM will not begin executing until you connect with GDB and issue a continue command.


If you're using libvirt, the following can be added to the <domain> section of an XML config file in /etc/libvirt/qemu/ to enable the gdb server for a VM:
<syntaxhighlight lang="xml">
  <qemu:commandline>
    <qemu:arg value='-s'/>
    <qemu:arg value='-S'/>
  </qemu:commandline>
</syntaxhighlight>

== Sample application using POSIX UEFI library ==

Debugging UEFI binaries can be challenging because you typically don't know the address where your image will be loaded at runtime, complicating both getting an initial breakpoint and symbol loading.
One workaround is have your application write its loaded base address to a known memory location, together with a marker value, so GDB can watch for it and reload symbols at the correct address.

This example is written using [[POSIX-UEFI]] in a 64 bit environment though the method should be modifiable to apply to any implementation.
<syntaxhighlight lang="c">
int main(int argc, char **argv){
    efi_loaded_image_protocol_t *loaded_image;
    efi_status_t status;
    // Define the GUID variable (cannot pass macro directly)
    efi_guid_t LoadedImageProtocolGUID = EFI_LOADED_IMAGE_PROTOCOL_GUID;

    // Retrieve the Loaded Image Protocol using HandleProtocol
    status = BS->HandleProtocol(IM, &LoadedImageProtocolGUID, (void **)&loaded_image);
    if (EFI_ERROR(status)) {
        printf("HandleProtocol failed: 0x%lx\n", status);
        return status;
    }

    // Print the actual base address of the loaded image
    printf("Image loaded at: 0x%lx\n", (uint64_t)loaded_image->ImageBase);

    // Write image base and marker for GDB
    volatile uint64_t *marker_ptr = (uint64_t *)0x10000;
    volatile uint64_t *image_base_ptr = (uint64_t *)0x10008;
    *image_base_ptr = (uint64_t)loaded_image->ImageBase;  // Store ImageBase
    *marker_ptr = 0xDEADBEEF;   // Set marker

    printf("Hello, world!\n");

    return EFI_SUCCESS;
}
</syntaxhighlight>

When this code is executed, it will write 0xDEADBEEF to address 0x10000 and the image base to address 0x10008, allowing you to detect the moment and address for symbol reloading in GDB.

The next thing required for GDB is executable image with symbols. If you carefully examined build log and Makefiles you should note that when EFI executable is created from ELF shared object file only limited set of sections are copied to the resulted image:
<syntaxhighlight lang="text">
.text .sdata .reloc .data .dynamic .dynsym
</syntaxhighlight>
For having debug symbols we need additionally these sections (in case you have compiled files with "-ggdb" option):
<syntaxhighlight lang="text">
.debug_info .debug_abbrev .debug_loc .debug_aranges .debug_line .debug_macinfo .debug_str etc
</syntaxhighlight>
But if you create EFI binary which additionally contains these sections the EFI firmware will be unable to execute it. Fortunately, we do not need the debug symbols on the target machine since we will use remote debugging with gdb anyway. Instead we can create two EFI binaries - one with only required sections to upload it to target system and another one with debug symbols to use it with GDB. In practice, this just requires running objcopy utility a second time with different set of sections, conveniently provided by the --only-keep-debug flag to copy and different output files. Here is an example with modifying the POSIX-UEFI makefile:

<syntaxhighlight lang="make">
ifneq ($(USE_GCC),)
    $(OBJCOPY) -j .text -j .sdata -j .data -j .dynamic -j .dynsym  -j .rel -j .rela -j .rel.* -j .rela.* -j .reloc --output-target $(EFIARCH) --subsystem=10 $^ $(addprefix $(OUTDIR),$@) || echo target: $(EFIARCH)
    $(OBJCOPY) --only-keep-debug $^ $(addprefix $(OUTDIR),$@.debug)
    @rm $(addprefix $(OUTDIR),$(TARGET).so)
endif
</syntaxhighlight>


Now you can launch GDB. Because your image will be relocated by the firmware, you need to let GDB know the correct addresses.


<syntaxhighlight lang="bash">
$ gdb
(gdb) target remote localhost:1234

# Set a watchpoint for the marker write
(gdb) watch *(unsigned long long*)0x10000 == 0xDEADBEEF
(gdb) continue

# Execution will break as soon as the marker is written.
# Now, fetch the relocated base address:
(gdb) set $base = *(unsigned long long*)0x10008

# Reload symbols for the image at the correct address:
(gdb) add-symbol-file bootloader/main.efi.debug -o $base

# Now you can step, set breakpoints, etc.
(gdb) si
</syntaxhighlight>

<syntaxhighlight lang="text">
$ gdb
GNU gdb (GDB) 16.3
Copyright (C) 2024 Free Software Foundation, Inc.
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.
Type "show copying" and "show warranty" for details.
This GDB was configured as "x86_64-pc-linux-gnu".
Type "show configuration" for configuration details.
For bug reporting instructions, please see:
<https://www.gnu.org/software/gdb/bugs/>.
Find the GDB manual and other documentation resources online at:
    <http://www.gnu.org/software/gdb/documentation/>.

--Type <RET> for more, q to quit, c to continue without paging--
rd".
(gdb) target remote localhost:1234
Remote debugging using localhost:1234
warning: No executable has been specified and target does not support
determining executable automatically.  Try using the "file" command.
0x000000000000fff0 in ?? ()
(gdb) watch *(unsigned long long*)0x10000 == 0xDEADBEEF
Hardware watchpoint 1: *(unsigned long long*)0x10000 == 0xDEADBEEF
(gdb) continue
Continuing.

Thread 1 hit Hardware watchpoint 1: *(unsigned long long*)0x10000 == 0xDEADBEEF

Old value = 0
New value = 1
0x000000000e059518 in ?? ()
(gdb) set $base = *(unsigned long long*)0x10008
(gdb) add-symbol-file bootloader/main.efi.debug -o $base
add symbol table from file "bootloader/main.efi.debug" with all sections offset by 0xe056000
(y or n) y
Reading symbols from bootloader/main.efi.debug...
(gdb) si
0x000000000e05951f  265     printf("Hello, world!\n");
(gdb) list
260     volatile uint64_t *marker_ptr = (uint64_t *)0x10000;
261     volatile uint64_t *image_base_ptr = (uint64_t *)0x10008;
262     *image_base_ptr = (uint64_t)loaded_image->ImageBase;  // Store ImageBase
263     *marker_ptr = 0xDEADBEEF;   // Set marker
264 
265     printf("Hello, world!\n");
266 
267     return EFI_SUCCESS;
268 }
269
(gdb)
</syntaxhighlight>

Or if you want to avoid entering in these commands manually each time, you can also place them in a <code>.gdbinit</code> file in your project directory. GDB will automatically execute these commands when it starts, streamlining your debugging workflow.

For example:
<syntaxhighlight lang="gdb">
target remote localhost:1234
watch *(unsigned long long)0x10000 == 0xDEADBEEF
continue
set $base = *(unsigned long long)0x10008
add-symbol-file bootloader/main.efi.debug -o $base
</syntaxhighlight>

[[Category:UEFI]]
[[Category:Debugging]]
````

<a id="w003834"></a>

## W003834 — Getting VBE Mode Info

Bron-ID: W003834
Namespace: Main (0)
Revisie: 18659 | Datum: 2015-11-28T17:30:37Z
Bron: https://wiki.osdev.org/Getting_VBE_Mode_Info
Revisie-URL: https://wiki.osdev.org/index.php?title=Getting_VBE_Mode_Info&oldid=18659
Doorverwijzing: VESA Video Modes

````text
#REDIRECT [[VESA Video Modes]]
````

<a id="w003835"></a>

## W003835 — Talk:Getting VBE Mode Info

Bron-ID: W003835
Namespace: Talk (1)
Revisie: 18661 | Datum: 2015-11-28T17:30:38Z
Bron: https://wiki.osdev.org/Talk:Getting_VBE_Mode_Info
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGetting_VBE_Mode_Info&oldid=18661
Doorverwijzing: Talk:VESA Video Modes

````text
#REDIRECT [[Talk:VESA Video Modes]]
````

<a id="w003836"></a>

## W003836 — Vesa

Bron-ID: W003836
Namespace: Main (0)
Revisie: 18662 | Datum: 2015-11-28T17:32:40Z
Bron: https://wiki.osdev.org/Vesa
Revisie-URL: https://wiki.osdev.org/index.php?title=Vesa&oldid=18662
Doorverwijzing: VESA Video Modes

````text
#REDIRECT [[VESA Video Modes]]
````

<a id="w003837"></a>

## W003837 — VESA

Bron-ID: W003837
Namespace: Main (0)
Revisie: 18663 | Datum: 2015-11-28T17:38:28Z
Bron: https://wiki.osdev.org/VESA
Revisie-URL: https://wiki.osdev.org/index.php?title=VESA&oldid=18663
Doorverwijzing: VESA Video Modes

````text
#REDIRECT [[VESA Video Modes]]
````

<a id="w003838"></a>

## W003838 — User:Twrl/Unicode

Bron-ID: W003838
Namespace: User (2)
Revisie: 19242 | Datum: 2016-05-18T11:59:49Z
Bron: https://wiki.osdev.org/User:Twrl/Unicode
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATwrl%2FUnicode&oldid=19242

````text
Unicode is a standard way of representing text in many different scripts and languages. As of Unicode version 8.0 the standard covers more than 120,000 characters in 129 different scripts. 

== Structure ==

Unicode itself is an abstract thing. It consists of ''abstract characters'' and ''code points'', their properties, and rules about composing and decomposing them. The actual representation of Unicode text is in one of the various Unicode Transformation Formats (the UTF-*s that you see around the place).

The practical upshot of which is that there are multiple levels of abstraction. There is not a one-to-one correspondence between code points and abstract characters: for example, &eacute; can be represented by the code point U+00E9 <small>(LATIN SMALL LETTER E WITH ACUTE)</small>, or by the sequence U+0065 <small>(LATIN SMALL LETTER E)</small> followed by U+0301 <small>(COMBINING ACUTE ACCENT)</small>, and for almost all purposes these two representations are identical. 

A code point is a number in the range U+0000 to U+10FFFF. These are conventionally written with the prefix U+ rather than 0x to make the semantics clear: they are abstract codes not data values. Each code point has various properties associated with it, such as a name, category, block, case conversions, directionality, and so on. Not every property is defined for every code point.

Code points are organised into planes and blocks. There are 17 planes, numbered 0 to 10 in hex, which is the high order part of the code point's value. The majority of characters are assigned in plane 0 (the Basic Multilingual Plane) or plane 1 (the Supplementary Multilingual Plane)

{| class="wikitable"
|+ Unicode Planes
|-
! Plane 
! Code Range 
! Name 
|-
| 0
| U+0000 to U+FFFF
| Basic Multilingual Plane 
|-
| 1
| U+10000 to U+1FFFF
| Supplementary Multilingual Plane
|-
| 2
| U+20000 to U+2FFFF
| Supplementary Ideographic Plane
|-
| 3
| U+30000 to U+3FFFF
| ''Unused''
|-
| 4
| U+40000 to U+4FFFF
| ''Unused''
|-
| 5
| U+50000 to U+5FFFF
| ''Unused''
|-
| 6
| U+60000 to U+6FFFF
| ''Unused''
|-
| 7
| U+70000 to U+7FFFF
| ''Unused''
|-
| 8
| U+80000 to U+8FFFF
| ''Unused''
|-
| 9
| U+90000 to U+9FFFF
| ''Unused''
|-
| 10
| U+A0000 to U+AFFFF
| ''Unused''
|-
| 11
| U+B0000 to U+BFFFF
| ''Unused''
|-
| 12
| U+C0000 to U+CFFFF
| ''Unused''
|-
| 13
| U+D0000 to U+DFFFF
| ''Unused''
|-
| 14
| U+E0000 to U+EFFFF
| Supplementary Special Use Plane
|-
| 15
| U+F0000 to U+FFFFF
| Supplementary Private Use Area A
|-
| 16
| U+100000 to U+10FFFF
| Supplementary Private Use Area B
|}

As you can see a very large chunk of the code point space is completely unused, and another big chunk is given over to private use. There's also a smaller Private Use Area in the Basic Multilingual Plane, as well as all of planes 15 and 16 - lots of place for people into conlanging and conscripting to define their own (and there's a conscript registry which tries to coordinate use of the PUAs), or more likely for linguists and anthropologists to try out different things.

Blocks roughly correspond to scripts, or script variants. A lot of them are familiar, for example Basic Latin, General Punctuation, Box Drawing, etc. Blocks are useful for organising such a vast array of code points, because each block gives a name to a contiguous range of characters with related usage.

== Encoding ==

There are various ways of encoding Unicode text. These are called Unicode Transformation Formats. The three most common are UTF-8, UTF-16, and UTF-32.

=== UTF-32 ===

UTF-32 is the simplest. Each code point is represented as a 32-bit integer, whose value is the value of the code point. There's no real transformation step. A string in UTF-32 might be represented as 

<source lang="cpp">
char32_t* myString;
</source>

and manipulated just like a regular C-style ASCII string. It is still worth bearing in mind that there is more than one possible sequence of code points to represent some characters.

=== UTF-16 ===

UTF-16 is what vendors have historically meant when they talk about Unicode support. In UTF-16, text is represented as a sequence of 16-bit values. Every code point in the basic multilingual plane is coded as it's own value. Every code point in other planes is represented as sequence of two values called a surrogate pair.

There is a reserved block in the Basic Multilingual Plane, code points U+D800 to U+DFFF, which are unused for representing characters. These values are used by UTF-16 to construct surrogate pairs.

Each surrogate pair consists of a high surrogate (in the range U+D800 to U+DBFF) followed by a low surrogate (in the range U+DC00 to U+DFFF). To encode a code point outside the BMP into UTF-16:
* Subtract 0x10000 from the code point, leaving a 20-bit number 
* Add the 10 most significant bits to the value 0xD800, this give the value of the high surrogate 
* Add the 10 least significant bits to the the value 0xDC00, this is the value of the low surrogate

For example, a simple function (in C++) which reads a UTF-32 stream and writes to a UTF-16 stream might look like this:

<source lang="cpp">
void translate(basic_istream<char32_t>& utf32in, basic_ostream<char16_t>& utf16out) {
    char32_t u32;
    char16_t u16hi, u16lo;

    while (!utf32in.eof()) {
        u32 = utf32in.get();
        if (u32 > 0xFFFF) {
            u32 -= 0x10000;
            u16lo = 0xDC00 + (u32 & 0x3FF);
            u16hi = 0xD800 + ((u32 >> 10) & 0x3FF);
            utf16out.put(u16hi).put(u16lo);
        } else {
            u16lo = static_cast<char16_t>(u32);
            utf16out.put(u16lo);
        }
    }
}
</source>

Obviously there are other things that you can do in a function like this. For example, you can normalise as you go, you can check that the code points are actually defined, etc.

Since virtually all modern text only requires code points in the Basic Multilingual Plane, surrogate pairs are actually quite rare in general use and functions to encode and decode them are often under-tested. On the plus side, the algorithm is relatively simple so there's little to go wrong with it.

=== UTF-8 ===

UTF-8 is probably the most common of the unicode encodings today, and is very widely used in web and email. It has the advantage of being compatible with 7-bit ASCII, which makes it good for applications where there's likely to be legacy data. The down side is that UTF-8 encodes code points as sequences of 1, 2, 3, or 4 bytes, which makes it slightly more complicated to implement than UTF-16.

=== Others ===

There are multiple other encodings for Unicode. The best known are probably UTF-7, UTF-EBCDIC, and GB 18020. Wikipedia has [https://en.wikipedia.org/wiki/Comparison_of_Unicode_encodings a good article on the different encodings].
````

<a id="w003839"></a>

## W003839 — User talk:Twrl/Unicode

Bron-ID: W003839
Namespace: User talk (3)
Revisie: 18679 | Datum: 2015-12-06T19:26:00Z
Bron: https://wiki.osdev.org/User_talk:Twrl/Unicode
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ATwrl%2FUnicode&oldid=18679

````text
This is a good write-up - I think you've got plenty of content to make it the wiki page for Unicode and let others start editing it. [[User:Intx13|Intx13]] 13:26, 6 December 2015 (CST)
````

<a id="w003840"></a>

## W003840 — Ensoniq AudioPCI

Bron-ID: W003840
Namespace: Main (0)
Revisie: 26176 | Datum: 2021-07-06T18:07:08Z
Bron: https://wiki.osdev.org/Ensoniq_AudioPCI
Revisie-URL: https://wiki.osdev.org/index.php?title=Ensoniq_AudioPCI&oldid=26176

````text
The Ensoniq AudioPCI is a low-cost PCI sound card that notably used software to emulate a MIDI synthesizer using a proprietary wave table format.  It also supported up to 48kHz PCM input and output and up to 4 channels in some configurations.  It is also one of the sound card models currently supported in VMWare virtual machines.

After the AudioPCI was released, Ensoniq was acquired by Creative Labs, which re-labeled the AudioPCI as the SoundBlaster PCI 64, and later the SoundBlaster PCI 128.  The PCI Subclass and Revision registers can be used to detect exactly which model is installed, and each model has similar (but not identical) register sets.

== I/O Registers ==

The following I/O mapped registers can be found by reading the BAR0 address of the Ensoniq AudioPCI device.  These registers are used by the ES1371 and later models.  (The earlier ES1370 model did not include an AC '97 Codec Read/Write register, and instead provided its own proprietary Codec register.)

{| {{wikitable}}
! Offset (Hex)
! Description
|-
| 0x00 || Control
|-
| 0x04 || Status
|-
| 0x08 || UART Data
|-
| 0x09 || UART Status/Control
|-
| 0x0a || UART Test Mode
|-
| 0x0c || Memory Page
|-
| 0x10 || Sample Rate Converter Read/Write
|-
| 0x14 || Codec Read/Write
|-
| 0x18 || Legacy
|-
| 0x20 || Serial Interface
|-
| 0x24 || Playback 1 Frame Count
|-
| 0x28 || Playback 2 Frame Count
|-
| 0x2c || Record Frame Count
|-
| 0x30 (Page 0x0c) || Playback 1 Buffer Address
|-
| 0x34 (Page 0x0c) || Playback 1 Buffer Definition
|-
| 0x38 (Page 0x0c) || Playback 2 Buffer Address
|-
| 0x3c (Page 0x0c) || Playback 2 Buffer Definition
|-
| 0x30 (Page 0x0d) || Record Buffer Address
|-
| 0x34 (Page 0x0d) || Record Buffer Definition
|-
| 0x30 (Page 0x0e) || UART FIFO Data
|-
|}

== Playing Audio ==

To play PCM audio on the AudioPCI, the following registers must be configured:

* Ensure that the PCI Bus Master flag is enabled for the device by setting bit 2 on the PCI Command register (0x04).
* Reset the controller by writing 0x20 to the Status register (0x04).  This flag will automatically reset back to 0 when the reset is complete.
* Reset the CODEC by writing 0xff to the Codec register (0x14).  This only applies to the 1371 AC 97 model and later.
* Program the Sample Rate Converter to the appropriate sample rate using register 0x10.  The specific values needed here must be calculated.  See below.
* Set the Master Volume and PCM Out Volume levels on the CODEC.  These are muted by default.
* Set the Memory Page register (0x0c) to 0x0c.  This is the only page needed for audio playback.
* Set the Playback 2 Buffer Address register (0x38) to the physical address of your audio buffer.
* Set the Playback 2 Buffer Definition register (0x3c) to the size of the audio buffer, in DWORDS, minus one.  This gives a maximum buffer size of 256K.
* Set the Playback 2 Frame Count register (0x28) to the number of frames to play, minus one.  After the last frame is played, an IRQ will be raised.
* Set the Serial Interface register (0x20) to 0x0020020C to enable 16-bit stereo, interrupts and looped mode on Playback 2.
* Set the Control register (0x00) to 0x00000020 to enable the Playback 2 DAC.

At this point, the audio should begin playing.  You will receive an IRQ when the last requested frame is played.  However, in looped mode, the audio will continue playing, and will loop around to the start of the audio buffer when the end of the buffer is reached.  By setting the Frame Count register to play half of the audio buffer, you will receive an interrupt when the first half of the buffer has been played.  This will allow you to fill the first half of the buffer while the second half is being played, and vice versa.  After receiving an interrupt and filling the audio buffer, you must re-enable interrupts by clearing the interrupt enable flag in the Serial Interface register, and then setting it again.

The steps above use the Playback 2 channel to play PCM audio.  The Playback 1 channel should work in a similar fashion.  However, according to the documentation, it was designed to be used as a software MIDI synthesizer channel.

== Sample Rate Converter ==

The Sample Rate Converter allows audio data to be converted to 48 kHz on the fly, and must be programmed with the proper values to perform the conversion from lower sample rates (8, 22, 44.1 kHz, etc.).  This process is not well documented in the Ensoniq spec sheets.  However, the formulas used to calculate the proper values can be found in various open source drivers for Linux and Windows CE.

The Sample Rate Converter is programmed using the Sample Rate Controller register (0x10).  This single 32-bit I/O register is used to both read and write values in several registers located in the Sample Rate Converter's memory on the AudioPCI card.

{| {{wikitable}}
! Offset (Hex)
! Description
|-
| 0x6c || Record Volume
|-
| 0x70 || Playback 1 TruncN
|-
| 0x71 || Playback 1 IntRegs
|-
| 0x72 || Playback 1 AccumFrac
|-
| 0x73 || Playback 1 VFreqFrac
|-
| 0x74 || Playback 2 TruncN
|-
| 0x75 || Playback 2 IntRegs
|-
| 0x76 || Playback 2 AccumFrac
|-
| 0x77 || Playback 2 VFreqFrac
|-
| 0x78 || Record TruncN
|-
| 0x79 || Record IntRegs
|-
| 0x7a || Record AccumFrac
|-
| 0x7b || Record VFreqFrac
|-
| 0x7c || Playback 1 Volume
|-
| 0x7e || Playback 2 Volume
|-
|}

The following pseudo-code is used to program the Sample Rate Converter to a specific sample rate.

<pre>
void SetPlayback2SampleRate(short rate)
{
    long frequency = (rate << 16) / 3000;

    SampleRateConverter[0x75] = (frequency >> 6) & 0xfc00;
    SampleRateConverter[0x77] = (frequency >> 1);
}
</pre>

=== External Links ===
* http://www.vogons.org/download/file.php?id=13036&sid=30df81e15e2521deb842a79f451b1161 - ES1371 Spec sheet
* http://www.vogons.org/viewtopic.php?f=46&t=26252&start=100 - Forum thread containing more details
* https://en.wikipedia.org/wiki/Ensoniq_AudioPCI - Wikipedia article
* http://read.pudn.com/downloads127/sourcecode/embed/538289/WAVEDEV/PDD/ES1371/es1371.cpp__.htm - Windows CE Driver Source
* http://read.pudn.com/downloads127/sourcecode/embed/538289/WAVEDEV/PDD/ES1371/es1371.h__.htm - Windows CE Driver Source Header

[[Category:Sound]]
````

<a id="w003841"></a>

## W003841 — AudioPCI

Bron-ID: W003841
Namespace: Main (0)
Revisie: 18684 | Datum: 2015-12-07T05:19:30Z
Bron: https://wiki.osdev.org/AudioPCI
Revisie-URL: https://wiki.osdev.org/index.php?title=AudioPCI&oldid=18684
Doorverwijzing: Ensoniq AudioPCI

````text
#REDIRECT [[Ensoniq AudioPCI]]
````

<a id="w003842"></a>

## W003842 — Common Driver Interface

Bron-ID: W003842
Namespace: Main (0)
Revisie: 28405 | Datum: 2023-10-18T07:37:38Z
Bron: https://wiki.osdev.org/Common_Driver_Interface
Revisie-URL: https://wiki.osdev.org/index.php?title=Common_Driver_Interface&oldid=28405

````text
{{Stub}}
[[Category:Driver Interfaces]]
The Common Driver Interface is a German project made by the people behind Tyndur, a community project of [http://www.lowlevel.eu/wiki/Hauptseite LowLevel], the German equivalent of the OSDev wiki. The interface offers less functionality than [[UDI]] but is a lot easier to implement. The creators have also implemented the interface in their own OS and they have made a nice amount of drivers.

There is an extensive German-language page on the LowLevel wiki, [http://www.lowlevel.eu/wiki/Common_Driver_Interface Common Driver Interface], and a [https://git.tyndur.org/lowlevel/cdi Git repository]. If anyone can find the English-language documentation (if it still exists), please add a link here.
````

<a id="w003843"></a>

## W003843 — File:ChaiOS sine.png

Bron-ID: W003843
Namespace: File (6)
Revisie: 18690 | Datum: 2015-12-13T10:17:44Z
Bron: https://wiki.osdev.org/File:ChaiOS_sine.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AChaiOS_sine.png&oldid=18690

````text
A screenshot of my OS
````

<a id="w003844"></a>

## W003844 — Ich

Bron-ID: W003844
Namespace: Main (0)
Revisie: 18713 | Datum: 2015-12-15T22:53:30Z
Bron: https://wiki.osdev.org/Ich
Revisie-URL: https://wiki.osdev.org/index.php?title=Ich&oldid=18713
Doorverwijzing: Intel I/O Controller Hub

````text
#REDIRECT [[Intel I/O Controller Hub]]
````

<a id="w003845"></a>

## W003845 — File:Shot10.jpg

Bron-ID: W003845
Namespace: File (6)
Revisie: 18722 | Datum: 2015-12-22T23:40:17Z
Bron: https://wiki.osdev.org/File:Shot10.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AShot10.jpg&oldid=18722

````text

````

<a id="w003846"></a>

## W003846 — User talk:Sortie

Bron-ID: W003846
Namespace: User talk (3)
Revisie: 23169 | Datum: 2019-01-05T08:55:26Z
Bron: https://wiki.osdev.org/User_talk:Sortie
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ASortie&oldid=23169

````text

----
Thanks for removing OS365, I'm not developing it now. --[[User:Catnikita255|Catnikita255]] 23:31, 24 December 2015 (CST)

== University of Osdev ==

I don't think everyone gets the joke. [[User:Glauxosdever|Glauxosdever]] 03:29, 16 July 2016 (CDT)
:Honestly, it's obvious. This is just a website for hobbyists. It's obviously not an university. The joke is appropriate because the sentence was about peer-review, which is important, and is commonly done at universities. If someone isn't smart enough to figure it out, they probably aren't smart enough to succeed at this site. Besides, this is my personal page, regular users are not supposed to come here, and if it was an university, the front page would have said so. Regardless, I don't want you specifically to edit my user page in the future. --[[User:Sortie|Sortie]] 07:59, 16 July 2016 (CDT)

----
Can I see your OS? Maybe I'm missing something, but I can't seem to find a link to your work! I'm interested to see what your hobby work looks like! :) --[[User:Ajxs|Ajxs]] 01:53, 14 March 2018 (CDT)
:Sure: I work on [https://sortix.org Sortix]. I updated my user page with that information, thanks.

== Reply to "[edit] Warning: Please stop worsening the project lists" ==

> Hi. I'm a wiki admin and I reverted your changes to Notable_Projects and Projects for the reasons outlined in the edit messages, and now you have restored the changes, starting an edit war.

Sorry, but it is you who fired the first shots in this edit war. In example, just because you think that floppies are not important (maybe just because your OS is too fat to fit there), you have simply removed the floppy availability information without even bringing this issue to the Talk page

> This is an official wiki warning to stop making these wiki pages worse

If you would look at my contributions history, you will see that I spent the huge amount of time to update the very outdated Projects pag. Just look at its miserable state before my arrival at December 2017.

If you will keep insisting that I am making these wiki pages worse, that I am not welcome here, I could simply remove ALL my contributions from this wiki

> The notable projects page is meant to contain the larger, sustained projects, that have a good degree of notability, and serves as useful bases for people to learn osdev from; or projects that are very far and very notable (even if they are currently inactive) ...

Your opinion about the notable projects is obviously biased, because - as a hobby OS author - you have a huge conflict of interest there. Hobby OS authors should not be allowed to modify the Notable Projects page at all (except from their own entry, and only if it has been added by another person). E.g. it could be that all the other people agree with me that your OS is not notable, but you are overpowering everyone simply because somehow you became an admin

Most importantly, I do not agree with Snowdrop OS removal, and Temple OS removal just because you hate its' author

Notability of OS should be decided by the people, not by a biased hobby OS author who is also an admin somehow

> Please don't accuse wiki admins of vandalism for trying to undo your changes that are really difficult to undo because you make many edits

In particular it took me several hours to gather the floppy availability information. You simply removed it because you think "if I don't care about floppies than nobody else is caring" - without even bringing this issue to the talk page. Now you remove it again. How thats not a vandalism? By definition, vandalism is destroying other people's hard work. You are destroying mine

> (I've tried to preserve the edits you made that I agree with).

can't see that - you have simply reverted. all the date updates have been lost

> Floppy support really is an irrelevant part of operating systems, a much more interesting thing would be whether it's a microkernel or monolithic, or whether there's an ARM64 port, or so on.

OK, how about - instead of destroying other people's work - you add some of your own? Do not remove "floppy availability" information - instead, gather the "kernel type" information and add this as a new category. Don't forget to draw a nice icon as well. Lets see how much time it would take, and would you be angry if someone else simply removes your "kernel type" category without even discussion at the Talk pages

>  It seems personal that you removed my Sortix operating system, which should be objectively notable by osdev standards since it's been covered a number of times on various sites

Temple OS has also been covered a number of times, actually its much more famous than Sortix and has a lot of forks with added functionality such as networking. But you hate its' author - and suddenly it is "not notable" as decided by our wise admin

> I will be forced to revoke your wiki edit privileges

Dont forget to remove all my previous contributions from this wiki then, if I am not welcome there

== about your message on my page ==

I'm not "update3" but I value his/her work and have been using update3's archived old version of page (latest before you removed the floppy info) for a long time. Like him/her I am putting these floppies inside the opensource coreboot bios of my laptop. And it is quite convenient to open a page with OS projects and Ctrl+F by "Floppy available" to see the possible floppy candidates to put inside your BIOS. Maybe the other media types are also important, however I don't value them as much and I'm afraid that even if I would spend a lot of time to gather the information for other media types, you can simply remove it without any reasonable discussion, just because you feel like it. 

> There's a lot more important aspects of operating systems to focus on, like whether they're installable

Well, if there's a floppy version of some OS, that means it is "installable" inside of opensource coreboot bios, and that is why this is very important by your own definition!

Will be waiting for your reply, meanwhile - aside from some very minor edits perhaps - I am not going to contribute anything to OSDev wiki because you could simply remove all my edits, while not creating anything of your own

== Would be nice if you could at least give a ==

Would be nice if you could at least give a Yes / No reply,
just interesting to know if you need contributors for OS Dev wiki
or you prefer to leave it as-is or "almost as-is"
````

<a id="w003847"></a>

## W003847 — I Cant Get Interrupts Working

Bron-ID: W003847
Namespace: Main (0)
Revisie: 18731 | Datum: 2015-12-26T12:16:00Z
Bron: https://wiki.osdev.org/I_Cant_Get_Interrupts_Working
Revisie-URL: https://wiki.osdev.org/index.php?title=I_Cant_Get_Interrupts_Working&oldid=18731
Doorverwijzing: I Can't Get Interrupts Working

````text
#REDIRECT [[I Can't Get Interrupts Working]]
````

<a id="w003848"></a>

## W003848 — LGDT

Bron-ID: W003848
Namespace: Main (0)
Revisie: 23545 | Datum: 2019-04-03T19:29:45Z
Bron: https://wiki.osdev.org/LGDT
Revisie-URL: https://wiki.osdev.org/index.php?title=LGDT&oldid=23545
Doorverwijzing: Global Descriptor Table

````text
#REDIRECT [[Global_Descriptor_Table]]
````

<a id="w003849"></a>

## W003849 — Java Primer

Bron-ID: W003849
Namespace: Main (0)
Revisie: 28229 | Datum: 2023-07-10T20:34:16Z
Bron: https://wiki.osdev.org/Java_Primer
Revisie-URL: https://wiki.osdev.org/index.php?title=Java_Primer&oldid=28229

````text
{{Rating|3}}

This is a demonstration tutorial of how you could write an OS using Java. The methods described in here however are quite generic - you can use similar approaches for other bytecoded languages, such as .NET. In addition, this article contains a simple compiler that can be a source of inspiration for any custom languages.

Being a demonstration, the method posted here is exceedingly simple and only does the necessary work to make the bundled Java code run. It can not deal with most things one would expect from Java. It does not do garbage collection in any forms, let alone support strings. The code provided here is purposefully limited, so you can play with it a bit to get a feeling, but it is impossible to extend this code to support more key components of the language. 

Consider this a throwaway prototype - '''it's deliberately written in such a fashion that you're forced to rewrite a new one from scratch.'''

== Supporting a non-native language ==

Since you have much less to leech from existing compilers, you essentially are in charge of the entire toolchain. There are a number of rough steps:
* '''Define your native ABI'''
:: Java and .NET come in bytecode, which are designed for machines that are fundamentally different than your typical register-based hardware. You'll need to understand assembly for your platform, and you'll have to think about how the language structures should be converted to native ones. One of the things this example does is to adhere to the stdcall convention, because it's closer to the virtual java machine than the cdecl that's the default in most compilers. 
* '''Create bytecode-to-native compiler'''
:: Java comes in bytecode form, and we need a method of converting it to native. Because we are writing an OS at the same time, everything must be compiled ahead of time, and not rely on an existing runtime that expects the existing presence of an OS. Therefore, we write our own compiler. As part of the compiler, we include the org.objectweb.asm that can read .class files and save us from writing significant portions of code.
* '''Compile the compiler'''
:: The compiler is of course written in Java as well. We don't need additional languages to add to the bootstrap problem later. Of course this compiler is overly simplistic, and will never be able to compile itself.
* '''Compile managed os to bytecode'''
:: Javac is written in Java, so that's obviously the best choice for later porting.
* '''Compile bytecode to native assembly'''
:: Now that we have the compiler, we can run it on all our OS classes to generate native code.
* '''Create runtime for things that have to be non-native'''
:: Java is memory-safe, which prevents it from implementing a lot of things. A minimal set of startup code is done in native assembly. Everything architecture-specific has to be done in assembly. Some parts you'll need later, such as memory management and exception handling are probably easier to have pure native parts for as well.
* '''Assemble os and runtime'''
:: We have everything in machine language form now. We're still reuseing the parts that we can, and in this example we use yasm. You're free to write an assembler in Java later on.
* '''package the final kernel binary'''
:: This is done using binutils. Note that there is not a single C compiler in the chain, and again, you can build your own linker in your language - it doesn't really have to do that much.


== Compiler ==

In bytecoded languages there are several steps before code can be run. Typically you have "the" compiler, which converts your source files into some portable binary, and you have an interpreter that reads those binaries and runs instructions from them. Modern interpreters turn bytecode into native code, as to avoid the <tt>if(instruction = ...)</tt> that takes several cycles while the instruction you'd actually want to execute would otherwise cost you just one CPU instruction. 

In the case of an OS, we need to take this a step further. We could run an interpreter, but that's slow. We could compile into native on boot but that needs just as much OS as we actually want to run. Instead, the appropriate solution is to compile to native in advance, so we can just run the code directly from the start.

The entirety of a language is still a whole lot to deal with, but for our example it suffices to deal with just integers. That's right, no objects yet!

The Java bytecode uses a stack for operations, and a list of locals. These need not be in the same place, but as the x86 only has one hardware stack, we'll be using it as both local stack, call stack, and operation stack. A few tricks are used to make this compiler easier, and in turn, make it difficult to interface with C. Locals in Java terms include the function arguments, and as a result the locals would be split around return addresses. Since the caller doesn't know the storage needed - it doesn't even get the number of arguments for free - the called function should fix this. We also can't put things past the top of the stack because that'll be a big issue with interrupts later. Basically we copy all arguments to the other side of the return address and then we make some room for locals so that they can be indexed by <tt>EBP - 4 * slot_number</tt> where for instance 0 and 1 would be arguments and 2+ would be true locals.

The fact that getting the number of arguments is convoluted to perform on the caller's side, we do callee-cleanup using <tt>RET imm</tt> instead of the regular <tt>RET</tt>. Locals is a convoluted issue as well, so we just reserve room for 8 because you're not meant to copy this code anyway.

<source lang="java">
// File: compiler/nl/combuster/minijava/Compiler.java
package nl.combuster.minijava;

import org.objectweb.asm.*;
import org.objectweb.asm.tree.*;
import java.util.*;
import java.io.*;

public class Compiler
{
    public static byte[] readEntireFile(String filename)
    {
        File file = new File(filename);
        try 
        {
            FileInputStream input = new FileInputStream(file);
            byte bytes[] = new byte[(int)file.length()];
            input.read(bytes);
            return bytes;
        }
        catch (IOException e)
        {
            throw new RuntimeException("Unable to read file " + file, e);
        }
    }

    public static void writeOutput(String filename, List<String> lines)
    {
        try 
        {
            BufferedWriter writer = new BufferedWriter(new FileWriter(new File(filename)));
            for (String string : lines)
            {
                writer.write(string);
                writer.newLine();
            }
            writer.close();
        }
        catch (IOException e)
        {
            throw new RuntimeException("Unable to write output file " + filename, e);
        }
    }

    public static String decorate(String classname, String method, String signature)
    {
        // make all these assembly-friendly names. Note that the 
        // constructor is for instance called <init>
        return classname.replace("/","_") + "__" + method.replace("<","_").replace(">","_");
    }
    
    public static void jumpgroup(List<String> output, String jumpcode, LabelNode dest)
    {
    	output.add("pop edx");
    	output.add("pop ecx");
    	output.add("cmp ecx, edx");
    	output.add(jumpcode + " .l" + dest.getLabel());
    }

    public static void main(String args[])
    {
        if (args.length != 2) throw new RuntimeException("Usage: compiler input-file output-file");

        ClassNode node = new ClassNode();
        ClassReader reader = new ClassReader(readEntireFile(args[0]));
        reader.accept(node, 0);

        List<String> outputdata = new LinkedList<String>();

        outputdata.add("section .text");

        for (MethodNode method : node.methods)
        {
            method.visitCode();
            String methodname = decorate(node.name, method.name, method.signature);

            if ((method.access & Opcodes.ACC_NATIVE) != 0) continue;

            // prologue
            System.out.println("; attributes: " + method.attrs);
            outputdata.add("global " + methodname);
            outputdata.add(methodname + ":");
            outputdata.add("push ebp");
            outputdata.add("mov ebp, esp");

            int locals = (method.localVariables == null) ? 0 : method.localVariables.size();
            outputdata.add("; locals: + " + locals);
			int arguments = (method.parameters == null) ? 0 : method.parameters.size();
            if ((method.access & Opcodes.ACC_STATIC) == 0) arguments++; // hidden "this"
            outputdata.add("; params: + " + arguments);
            // copy params so that they correspond with java indexing and join with the local numbering
			for (int i = 0; i < arguments; i++)
            {
                outputdata.add("push dword [ebp + " + (8 + 4 * i) + "]");
            }
            // do some frame checking for many variables, locals is not of much use...
            outputdata.add("sub esp, 32");

            Iterator<AbstractInsnNode> iterator = method.instructions.iterator();
            while (iterator.hasNext())
            {     
                AbstractInsnNode insn = iterator.next();
                int opcode = insn.getOpcode() & 0xff;
                outputdata.add("    ; " + opcode + " = " + insn.getClass().getSimpleName());
                //outputdata.add("mov byte [" + (0xb8000 + 156) + "], '0' + " + (opcode % 10));
                //outputdata.add("mov byte [" + (0xb8000 + 154) + "], '0' + " + (opcode / 10)%10);
                //outputdata.add("mov byte [" + (0xb8000 + 152) + "], '0' + " + (opcode / 100));
                
                if (insn instanceof LabelNode)
                {
                    LabelNode labelinsn = (LabelNode)insn;
                    outputdata.add(".l" + labelinsn.getLabel());
                }
                else if (insn instanceof LineNumberNode)
                {
                    // ignore these
                }
                else if (insn instanceof VarInsnNode)
                {
                    // copy a variable
                    VarInsnNode varinsn = (VarInsnNode) insn;
                    switch(varinsn.getOpcode())
                    {
                        case Opcodes.ILOAD:
                        case Opcodes.ALOAD:
                            // todo: verify offset
                            outputdata.add("push dword [ebp - " + (4 + 4 * varinsn.var) + "]");
                            break;
                            
                        case Opcodes.ISTORE:
                        case Opcodes.ASTORE:
                            outputdata.add("pop dword [ebp - " + (4 + 4 * varinsn.var) + "]");
                            break;

                        default:
                            outputdata.add("Can't deal with varinsnnode: " + varinsn.getOpcode());
                    }
                }
                else if (insn instanceof MethodInsnNode)
                {
                    MethodInsnNode methodinsn = (MethodInsnNode) insn;
                    String calledmethod = decorate(methodinsn.owner, methodinsn.name, methodinsn.desc);
                    outputdata.add("extern " + calledmethod);
                    outputdata.add("call " + calledmethod);
                    if (!methodinsn.desc.endsWith("V"))
                    {
                        // not a void return value
                        outputdata.add("push eax");
                    }
                    /*switch (methodinsn.getOpcode())
                    {
                        default:
                            outputdata.add("Can't deal with methodcall: " + methodinsn.getOpcode());
                    }*/
                }
                else if (insn instanceof IntInsnNode)
                {
                	IntInsnNode intinsn = (IntInsnNode)insn;
                    switch(intinsn.getOpcode())
                    {
                        case Opcodes.BIPUSH:
                            outputdata.add("push " + intinsn.operand);
                            break;

                        default:
                            outputdata.add("Can't deal with intinsnnode: " + intinsn.getOpcode());
                    }
                	
                }
                else if (insn instanceof JumpInsnNode)
                {
                	JumpInsnNode jmpinsn = (JumpInsnNode)insn;
                	
                    switch(jmpinsn.getOpcode())
                    {
                    	case Opcodes.IF_ICMPEQ:		jumpgroup(outputdata, "je", jmpinsn.label); 	break; // 159
                    	case Opcodes.IF_ICMPNE:		jumpgroup(outputdata, "jne", jmpinsn.label); 	break; // 160                    	
                    	case Opcodes.IF_ICMPLT:		jumpgroup(outputdata, "jb", jmpinsn.label); 	break; // 161
                    	case Opcodes.IF_ICMPGE:		jumpgroup(outputdata, "jae", jmpinsn.label); 	break; // 162
                    	case Opcodes.IF_ICMPGT:		jumpgroup(outputdata, "ja", jmpinsn.label); 	break; // 163
                    	case Opcodes.IF_ICMPLE:		jumpgroup(outputdata, "jbe", jmpinsn.label);	break; // 164
                    
                        case Opcodes.GOTO: // 167
                            outputdata.add("jmp .l" + jmpinsn.label.getLabel());
                            break;

                        default:
                            outputdata.add("Can't deal with jumpinsnnode: " + jmpinsn.getOpcode());
                    }
                }   
                else if (insn instanceof LdcInsnNode)
                {
                	LdcInsnNode ldcinsn = (LdcInsnNode) insn;
                	if (ldcinsn.cst instanceof Integer)
                	{
                		outputdata.add("push " + ldcinsn.cst.toString());
                	}
                	else
                	{
	                	outputdata.add("Can't deal with data in ldcinsnnode (" + ldcinsn.getOpcode() +"): " + ldcinsn.cst.getClass().getSimpleName());
                	}
                }             
                else if (insn instanceof IincInsnNode)
                {
                	IincInsnNode incinsn = (IincInsnNode) insn;
                	switch (incinsn.getOpcode())
                	{	
                	    case Opcodes.IINC:
                	    	outputdata.add("add dword [ebp - " + (4 + 4 * incinsn.var) + "], " + incinsn.incr);
                	    	break;
                		default:
                			outputdata.add("Can't deal with iincinsnnode: " + incinsn.getOpcode());                			
                	}
                	
                }
                else if (insn.getOpcode() >= Opcodes.ICONST_M1 && insn.getOpcode() <= Opcodes.ICONST_5) // 2...8
                {
                    outputdata.add("push " + (insn.getOpcode() - Opcodes.ICONST_M1 - 1));
                }
                else if (insn.getOpcode() == Opcodes.IADD) // 96
                {
                	outputdata.add("pop edx");
                	outputdata.add("add [esp], edx");
                }
                else if (insn.getOpcode() == Opcodes.IMUL) // 104
                {
                	outputdata.add("pop eax");
                	outputdata.add("pop ecx");
                	outputdata.add("imul ecx"); // eax:edx = eax * ecx
                	outputdata.add("push eax");
                }                
                else if (insn.getOpcode() == Opcodes.I2B) // 145
                {
                	outputdata.add("and dword [esp], 0xff");
                }
                else if (insn.getOpcode() == Opcodes.RETURN)	// 177
                {
                	outputdata.add("xor eax, eax");
                }
                else if (insn instanceof FrameNode)
                {
                    outputdata.add("; framenode");
                }
                else
                {
                    outputdata.add("Can't deal with " + insn.getClass().getSimpleName() + ", fix it (" + insn.getOpcode() + ")");
                }
            }

            // epilogue, stdcall to save complexity on decoding methodinsns
            outputdata.add("leave");
            outputdata.add("ret " + arguments);
        }

        writeOutput(args[1], outputdata);
    }
}

</source>

Yes, that's a compiler in a magic 256 lines of code. The result is Intel syntax assembly, because there already are decent tools for assembly out there that deal with object formats out there. Of course you can write your own later as well.

== HAL and Runtime ==
Almost no programming language has standard constructs for all the things a processor can do, in particular not languages that were designed to be portable and memory-safe. For that reason we are required to add support for that outside of the language. Fortunately, Java does come with one construct specifically design to wrap the language to platform specifics: <tt>native</tt>

<source lang="java">
// File: os/nl/combuster/minijavaos/Hal.java
package nl.combuster.minijavaos;

public class Hal
{
    public static native void poke(int address, byte data);
}
</source>
At some point the OS needs to access memory unsafely, and this is the method we'll use. The native keyword also implies that there is no implementation in java and that it'll come from elsewhere.

There are more things that might be difficult to do in straight Java: this example omits memory management in its entirety, but there is still a quirk: Every object subclasses from java.lang.Object. To make matters more complicated, the compiler does not allow us to write java.lang classes in Java. In this case it's a particular chicken-and-egg problem: how would we even compile a class that extends itself? We are in some way forced to treat java.lang.Object special, and basically use its constructor as a terminator for a constructor chain. At a later stage, we might even want to do something more in there, but for now we don't need to. The first piece of native support code deals with these problems:

<source lang="asm">
; File: os/hal.asm
section .text

global java_lang_Object___init_
global nl_combuster_minijavaos_Hal__poke

java_lang_Object___init_:
	ret

nl_combuster_minijavaos_Hal__poke:
	mov eax, [esp+8]
	mov dl, [esp+4]
	mov [eax], dl
	ret 2

</source>

== A primitive kernel ==
"Hello world!" is a very overrated thing. Or is it? Strings are a full-blown object in Java, and passing objects safely is one of the things skipped in the compiler itself. Instead, we can count our numbers to show what we have actually works:

<source lang="java">
package nl.combuster.minijavaos;

public class Kernel
{
    public Kernel(int arg1)
    {
    }

    public static void kmain()
    {            
    	
        for (int i = 0; i < 10; i++)
        {
            Hal.poke(0xB8000 + 2 * i,     (byte) ('0' + i));
            Hal.poke(0xB8000 + 2 * i + 1, (byte) 0x1F); 
        }                
    }
}
</source>
This is little more than some simple VGA code, but it demonstrates that we can call Java code, do some calculations, and then call our hardware abstraction layer to complete the little bits Java just couldn't do.

== Wrapping it all together ==
The last thing we need to do is to glue all the pieces together and stick a bootloader on top of it. [[Bare Bones]] has a more extensive description of the process. You might have noticed the full filenames in the source files above, basically we have two projects fairly tightly intertwined with each other, residing in the compiler and os subfolders that serve as the Java source root at the same time. A build folder is added so that you can simply delete that folder to start fresh, without having to worry about finding all your intermediates in between the real sources. In particular, we wouldn't be able to tell which .asm file is original and which one is compiler output without it! 

==== Multiboot ====
We use GRUB as a bootloader, so the mechanics are the same. The only real difference is that we have to cope with the name mangling and be unable to call something kmain.
<source lang="asm">
; File: os/multiboot.asm

; Declare constants used for creating a multiboot header.
MBALIGN     equ  1<<0                   ; align loaded modules on page boundaries
MEMINFO     equ  1<<1                   ; provide memory map
FLAGS       equ  MBALIGN | MEMINFO      ; this is the Multiboot 'flag' field
MAGIC       equ  0x1BADB002             ; 'magic number' lets bootloader find the header
CHECKSUM    equ -(MAGIC + FLAGS)        ; checksum of above, to prove we are multiboot
 
; Declare a header as in the Multiboot Standard.
section .multiboot
align 4
	dd MAGIC
	dd FLAGS
	dd CHECKSUM
 
; We'll need a stack
section .bootstrap_stack, nobits
align 4
stack_bottom:
resb 16384
stack_top:
 
; Kernel entry point
section .text
global _start
_start:
	mov esp, stack_top

	extern nl_combuster_minijavaos_Kernel__kmain
	call nl_combuster_minijavaos_Kernel__kmain
 
	cli
.hang:
	;hlt
	jmp .hang
</source>

====Linker script====
Linking also happens with the same script as [[Bare Bones]]

<source lang="c">
/* File: os/linker.ld */
/* The bootloader will look at this image and start execution at the symbol
   designated as the entry point. */
ENTRY(_start)

/* Tell where the various sections of the object files will be put in the final
   kernel image. */
SECTIONS
{
	/* Begin putting sections at 1 MiB, a conventional place for kernels to be
	   loaded at by the bootloader. */
	. = 1M;

	/* First put the multiboot header, as it is required to be put very early
	   early in the image or the bootloader won't recognize the file format.
	   Next we'll put the .text section. */
	.text BLOCK(4K) : ALIGN(4K)
	{
		*(.multiboot)
		*(.text)
	}

	/* Read-only data. */
	.rodata BLOCK(4K) : ALIGN(4K)
	{
		*(.rodata)
	}

	/* Read-write data (initialized) */
	.data BLOCK(4K) : ALIGN(4K)
	{
		*(.data)
	}

	/* Read-write data (uninitialized) and stack */
	.bss BLOCK(4K) : ALIGN(4K)
	{
		*(COMMON)
		*(.bss)
		*(.bootstrap_stack)
	}

	/* The compiler may produce other sections, by default it will put them in
	   a segment with the same name. Simply add stuff here as needed. */
}
</source>

==== GRUB ====
Some configuration files for GRUB legacy. Doing this with a CD is the easiest way, really:
<source lang="bash">
# 
# File: os/grub.cfg
#
default 0
timeout 30

# For booting GNU/Linux
title Java OS 
root (cd)
kernel (cd)/kernel
</source>

==== Build instructions ====
To simplify things, here's the Makefile that glues all the steps together. This one is a bit more elaborate as it covers for pretty much all the intermediate steps needed. Compile the compiler, make an executable jar file, compile the java files to class files, to assembly files, to object files, to an ELF binary. Include the runtime halfway into the process, then build a CD image with GRUB. 

In this example, GRUB is assumed to be preinstalled in /boot where it resides by default on a linux system that has it installed. We also need mkisofs and the binutils step from the [[GCC Cross-Compiler]]. Of course, at some point in time you can decide to write these tools in Java as well
<source lang="make">

COMPILER_J=$(shell find -L compiler -iname '*.java')
COMPILER_C=$(addprefix build/,$(patsubst %.java,%.class,$(COMPILER_J)))

OS_J:=$(shell find -L os -iname '*.java')
OS_C:=$(addprefix build/,$(patsubst %.java,%.class,$(OS_J)))
OS_A:=$(patsubst %.class,%.asm,$(OS_C)) 
RT_A:=$(shell find -L os -iname '*.asm')
OS_O:=$(patsubst %.asm,%.o,$(OS_A))
RT_O:=$(addprefix build/,$(patsubst %.asm,%.o,$(RT_A)))

default: build/compiler.jar build/kernel build/image.iso
	echo done compiling

.PHONY: default

build/.dir: 
	mkdir -p build/compiler
	mkdir -p build/os
	touch build/.dir

build/compiler.jar: $(COMPILER_C) compiler/compiler.manifest
	jar cvfm $@ compiler/compiler.manifest -C build/compiler .

build/compiler/%.class: compiler/%.java build/.dir
	javac -d build/compiler -sourcepath compiler $<

build/os/%.class: os/%.java build/.dir
	javac -d build/os -sourcepath os $<

%.asm:%.class build/compiler.jar
	java -jar build/compiler.jar $< $@

%.o:%.asm
	yasm -f elf -o $@ $<

build/os/%.o: os/%.asm build/.dir
	yasm -f elf -o $@ $<

build/kernel: $(OS_O) $(RT_O)
	i586-elf-ld -o $@ -T os/linker.ld $(OS_O) $(RT_O)

build/image.iso: build/kernel os/grub.cfg Makefile
	-rm -rf build/iso
	mkdir -p build/iso/boot/grub
	cp build/kernel build/iso/
	cp os/grub.cfg build/iso/boot/grub/menu.lst
	cp /boot/grub/stage2_eltorito build/iso/boot/grub/
	mkisofs -R -b boot/grub/stage2_eltorito -no-emul-boot -boot-load-size 4 -boot-info-table -o $@ build/iso

</source>

If you paid attention, you'll notice that the JAR step requires a manifest at <tt>compiler/compiler.manifest</tt>. It only really marks it as runnable so that we can use it easily:
<source lang="bash">
Main-Class: nl.combuster.minijava.Compiler
</source>

Lastly, the [http://forge.ow2.org/project/download.php?group_id=23&file_id=20549 one library] our compiler uses to do most of its magic. You can just copy the source into the compiler folder and the build script will pick it up.

[[Category:Tutorials]]
[[Category:Compilers]]
[[Category:Languages]]
[[Category:Java]]
````

<a id="w003850"></a>

## W003850 — Historical Notes on CISC and RISC

Bron-ID: W003850
Namespace: Main (0)
Revisie: 30409 | Datum: 2026-03-24T22:16:35Z
Bron: https://wiki.osdev.org/Historical_Notes_on_CISC_and_RISC
Revisie-URL: https://wiki.osdev.org/index.php?title=Historical_Notes_on_CISC_and_RISC&oldid=30409

````text
('''NB''': I originally wrote this for some of the other students in an assembly language class I took in Fall 2006, and posted shortly oafterwards to a [http://forums.devshed.com/programming-languages-139/assembly-historical-notes-cisc-vs-risc-427110.html thread on DevShed]. While it is not about OS-dev ''per se'', it may help clarify some of the puzzling aspects of assembly language for some members. - [[User:schol-r-lea]])

It is somewhat ironic that many Assembly language instructors today have found it easier to teach assembly programming in a RISC architecture such as MIPS or ARM, rather than in the ubiquitous but far more complex x86 PC system: historically, it was the CISC (Complex Instruction Set Computer) designs which are generally associated with extensive assembly language programming, while the RISC (Reduced Instruction Set Computer) designs are explicitly intended to make compiling code from a high-level language easier, with the expectation that assembly language would be rarely used except for in the operating system.

To understand why and how this reversal came about, you need to know a bit of history. When stored-program computers based on the Von Neumann architecture were first being developed in the late 1940s and early 1950s, the size and complexity of the hardware (which was much more diverse than today, using such disparate technologies as mercury delay lines, CRT-phosphor memories, magnetic drums, wire and tape recorders, ferromagnetic cores, and the ever-present vacuum tubes) meant that the hardware capacities were extremely limited; it was not uncommon for even a large machine to have only one or two registers and a main memory of a few thousand words (which varied in size from machine to machine). The instruction sets were equally constrained, and often had special-purpose instructions for connecting to the peripherals, taking up the already small instruction space. 

For example, the 18-bit Lincoln Labs TX-0 (one of the first transistor-based systems designed) had a grand total of four primary instructions (though one of these was a kind of escape code that told it to use the following word as an instruction from a different group of special-purpose operations) and two registers, X and Y. Many of these designs had what would today be considered poorly developed instruction sets, full of special cases, lacking useful instructions<sup>[[Historical Notes on CISC and RISC#oisc|1]]</sup>, and including instructions that were of little use. 

Generally speaking, these early designs had only a single special-purpose register, called the accumulator, on which it could perform most of the common operations such as addition or subtraction; the other operand, if any, usually was either in main memory, or in another special-purpose register called the source, and the result would remain in the accumulator (hence the name).

As the designers began to develop more effective architectural principles based on the success and failures of earlier efforts, hardware became smaller and less expensive, which in turn allowed for greater freedom in the CPU design, so the complexity of the systems grew; by 1966, the PDP-6, a typical design of the time, had sixteen 36-bit general-purpose registers, plus a stack register and frame register. The instruction sets became more complex as well; since a significant percentage of programming even into the 1970s was done in assembly language, it was thought that the instruction sets should be 'rich', that is, they should have a special instruction for nearly every common operation the programmer would want to use. 

This culminated with the 32-bit VAX-11 series of minicomputers and mainframes, which had 256 primary instructions, including ones for polynomial evaluation, trigonometric functions, and CRC calculation. Many of these instructions had multiple addressing modes, so that they could operate on registers, on main memory, or some combination thereof; while this increased the convenience of assembly programming, it also meant that there were several special cases the programmer had to be aware of, and it complicated the CPU design substantially.

To provide these instructions, designers began to use 'microcode', which was a set of what could be called firmware-encoded macro instructions that would be handled by the CPU itself. The complex instructions would get broken down into simpler instructions internally, which the programmers wouldn't need to be aware of, and executed as if they were a fixed part of the hardware. Such instructions would often take several system clock cycles to run, and in most processors, there was no pipelining in the modern sense - the system would have to wait until the each instruction was finished before it began processing the next one.

About the time the VAX was being designed, four other things were happening that would change this attitude. First, high-level languages were becoming the primary method of programming, meaning that the baroque instruction sets of the then-current CPUs were becoming unnecessary. 

Second, assembly language programmers were noting that in the majority of programs, whether in assembly or in a compiled language, only a handful of instructions were being used: few programs needed a CRC instruction, and hardly any programmers were familiar enough with the entire VAX instruction set to know that the instruction existed and how to use it. A side aspect of this is that the more complex instruction sets were increasingly difficult to learn and to teach. 

Third, computer hardware design had advanced to the point where graduate courses in CPU design were being offered; such courses naturally enough stuck to very simple and regular designs, which they found actually outperformed comparable complex systems (though professionally-constructed CPUs often used techniques which were patented or trade secrets, skewing the results when comparing the student designs to the commercial ones). They especially noted that the operations that worked directly on memory were generally slower than if one instead loaded the values into registers, performed several operations on those registers, and then stored the results back into memory. 

Finally, the advent of single chip microprocessors meant that it would soon be possible to mass-produce inexpensive computers for a fraction of the cost of minis and mainframes. Early on, these were limited by the capabilities of the hardware in much the same ways the first generation of CPUs were (and some of the design mistakes of the first generation were repeated as well), but the technology soon grew beyond anyone's expectations, and in some ways, beyond the abilities of the designers to predict what they would later need to support.

This last part had some major repercussions, especially regarding some of the design decisions as microprocessors became more powerful. Chief amongst these was the decision by Intel to try and extend the 8-bit [https://en.wikipedia.org/wiki/Intel_8080 8080] design into a new 16-bit design, the [https://en.wikipedia.org/wiki/Intel_8086 8086]; they bent over backwards to make the CPU as familiar as possible to programmers of the older chip, with the result that the design ended up with some unnecessary complications and limitations, especially in how it addresses larger sections of memory<sup>[[Historical Notes on CISC and RISC#segmentation|2]]</sup> Also, because the designers had given the system only a small number of registers, most of the operations have several complicated addressing modes to avoid running out of registers.

The fact that this processor was selected for the IBM PC, which would soon become the dominant platform, meant that these weaknesses were of critical importance to millions of users. This was further exacerbated when they extended the design still further with the [https://en.wikipedia.org/wiki/Intel_80286 80286], which now needed a separate 'protected mode' to access its full abilities while retaining 'normal mode' for backwards compatibility. Some of the design flaws were resolved in the next design, the [https://en.wikipedia.org/wiki/Intel_80386 80386], but at the cost of exponentially increasing complexity both of the chip itself and of assembly programming for it.

Meanwhile, other chip designers were going in other directions. One, Motorola, started over with a classic 'big' design, the [https://en.wikipedia.org/wiki/Motorola_68000 Motorola 68000], which resembled a scaled-down version of the VAX in many ways, with 16 general-purpose registers and a complicated instruction set. This would become the CPU for several successful workstations, as well as the original [https://en.wikipedia.org/wiki/Macintosh Apple Macintosh], [https://en.wikipedia.org/wiki/Atari_ST Atari ST], and [https://en.wikipedia.org/wiki/Amiga Commodore Amiga] home computer lines. Later design extensions would complicate this, though not to the extent that the 80x86 design would be.

Still, it was growing clear that the complex instruction sets were growing counter-productive. Thus, many chip designers decided to go in the opposite direction: minimal instruction sets, no microcoding, load/store architectures with few if any operations working on memory directly, large register sets which could be used to avoid accessing main memory whenever possible, and an emphasis on supporting high-level languages rather than assembly programing. This new idea, which was called RISC (reduced instruction set computer), would be the basis several new CPU designs, including the [https://en.wikipedia.org/wiki/MIPS_instruction_set MIPS] (and it's successor the [https://en.wikipedia.org/wiki/DLX DLX]), the [https://en.wikipedia.org/wiki/SPARC Sun SPARC], the [https://en.wikipedia.org/wiki/ARM_architecture Acorn ARM], the IBM [https://en.wikipedia.org/wiki/IBM_POWER_microprocessors POWER] architecture<sup>[[Historical Notes on CISC and RISC#powerpc|3]]</sup>, and the [https://en.wikipedia.org/wiki/DEC_Alpha DEC Alpha]. Of these, all but the Alpha remain in use today for certain specialized areas of use, and the ARM in particular has become the ''de facto'' standard for mobile computing, though for the most part the domination by the Windows-x86 system has forced them out of the market for home and business systems.

== Footnotes ==

===One Instruction Set Computers===
In principle, only a single operation, 'subtract two memory values and branch if the result is negative' (or several variants on this) is sufficient to allow a Random Access Machine to perform all Turing-computable calculations). There are even (simulated) machines which are designed on this principle. (Such machines are often named after the instruction they implement or, confusingly, OISC.) In practice, of course, such a design is wasteful, especially for the more commonly used operations such as integer arithmetic. This has not stopped the appearance of at least one operating system for such an architecture, complete with its own C compiler: [[Projects#D|Dawn]].

===Segmentation===
The specific issue is the use of 'memory segmentation', which was intended to provide a 20-bit address space while still only using 16-bit addressing for most purposes. It worked by having a separate set of segment registers, which pointed to 64K regions overlapping each other at 16-byte intervals. The addresses are formed by taking the segment address and a 16-bit offset, which are equivalent to the 8080's addresses, adding the two values with a 4-bit displacement to get the 20-bit address. Most instructions could use one of the segment registers as a default value for code or data, and use just the offsets in the instruction stream. The particular overlap was set at 4 bits because using more than 20 address pins was determined to be prohibitively expensive at the time, and would make the Dual-Inline Package design too large. It was assumed that a 1 MiB address space would be sufficient for a CPU meant as a microcontroller with a limited design lifespan rather than a general-purpose system that would be in use 40 years later. This segmentation system persists as 'Real mode' even in current x86 models.  (Segmentation exists even in 32-bit mode, although the segment registers remain 16-bit. Clever uses are occasionally found for it.)

===PowerPC===
PowerPC was a modified POWER architecture, used in Macintoshes from 1994 to 2006 and in some IBM OS/2 systems from 1992 to 1996. The POWER architecture itself is primarily used in larger multiprocessor systems such as [https://en.wikipedia.org/wiki/Deep_Blue_(chess_computer) Deep Blue] and [https://en.wikipedia.org/wiki/Watson_(computer) Watson].

[[Category:Theory]]
[[Category:Instruction Set Architecture]]
````

<a id="w003851"></a>

## W003851 — Register Machine Model of Computation

Bron-ID: W003851
Namespace: Main (0)
Revisie: 26032 | Datum: 2021-05-31T18:06:33Z
Bron: https://wiki.osdev.org/Register_Machine_Model_of_Computation
Revisie-URL: https://wiki.osdev.org/index.php?title=Register_Machine_Model_of_Computation&oldid=26032

````text
This page is intended to present an overview of the theoretical Register Machine model of computation, which is the conceptual basis for most real-world CPU designs. While it is assumed that anyone coming here will be at least passingly familiar with the concepts, it is meant to give a review of the topic to eliminate any misconceptions newcomers may have.

The '''Register Machine Model''' is a ''model of computation'' used to describe the computability of different equations. It is a sub-type of a class of computation models called ''Random Access Machines'' (RAMs), which also includes the related ''Stack Machine'' model. It is one of many highly diverse models, some of which — including many register machines — are ''Turing-Equivalent'', that is to say, they can compute the same set of propositions as a Universal Turing Machine. The relevance of the Register Machine model is that is it the easiest of all models to approximate in electronic hardware, and in fact the model was developed to describe the first programmable computer systems in mathematical terms, in order to demonstrate that they were Turing-equivalent.

== Computational Models and the History of Computing ==
A ''[https://en.wikipedia.org/wiki/Model_of_computation model of computation]'' is a mathematical formalism that describes a mechanical process ('mechanical' in the sense of deterministic and rules-based, not necessarily mechanized or electronic) for performing computations. The idea of such formalisms arose in the early 20th century, when several mathematicians — most notably Alfred Whitehead, David Hilbert, and Bertrand Russell — began to speculate on whether it was possible to develop a mechanical system for deriving mathematical proofs.

=== Early Models: Recursive Functions, λ-Calculus, and the Universal Turing Machine ===
The earliest computational models, both developed in the early 1920s, were ''[https://en.wikipedia.org/wiki/%CE%9C-recursive_function recursive function theory]'' and ''recursive set theory'', both of which were used for developing a set of rules for performing computations and proving theorems.

The original goal of automatic theorem proving was dealt a critical blow in 1933, when [https://en.wikipedia.org/wiki/Kurt_G%C3%B6del Kurt Gödel] demonstrated that any formalism of mathematical logic would have blind spots — either in the form of ''undecidable propositions'', true theorems which could not be proven, or ''invalid propositions'', theorems which could be 'proven' in the system but which were not actually true or contradicted another theorem. While this was a major setback, the proof Gödel used demonstrated that recursive functions were a useful tool in exploring the new area of ''computability''.

The next significant model to arise was the ''[https://en.wikipedia.org/wiki/Lambda_calculus lambda calculus]'', created by Alonzo Church in 1935. This was a simpler model of functions than the existing type of recursive function theory, but was demonstrated to have the same computational power as RFT. Church postulated that this these two models were, in fact, as complete as any sequential model could ever be.

That same year, a graduate student of Church's, Alan Turing, developed yet another model, the ''[https://en.wikipedia.org/wiki/Universal_Turing_machine Turing Machine]'', which he used to demonstrate that a well-known unsolved goal in mathematics — finding a method for [https://en.wikipedia.org/wiki/Halting_problem proving that any arbitrary computation process would go to completion] — was impossible, by showing that the testing process itself would necessarily be ''[https://en.wikipedia.org/wiki/Undecidable_problem undecidable]'' in the general case. 

The Turing Machine, in its basic form as described in the paper "[https://www.cs.virginia.edu/~robins/Turing_Paper_1936.pdf On Computable Numbers, with an Application to the ''Entscheidungsproblem'']", is thought of as a machine consisting of a tape read/writer and an infinitely long paper tape divided into cells containing data - one can think of the cells as boxes drawn on the tape to mark where a particular datum was. For each operation, the Turing Machine would position itself over a cell, read the datum in the cell, and based on that datum, decide whether to write a datum in a cell (whether the same one or a different one), move to another cell, or stop running and eject the tape in its current state as its final output. In his paper, he initially described each machine having a fixed set of responses to the data, stored separately in an ''action table'', and a fixed set of states which the data could put it into. He then proceeded 

In doing so, he showed that it was possible to design a variant of the Turing Machine that could simulate any other possible Turing machine; this ''Universal Turing Machine'' (UTM) demonstrated the same kind of generality as RF and  λ-Calculius, and he expanded on Church's premise that these three models represented the absolute limit of sequential computation, which became known as the ''[https://en.wikipedia.org/wiki/Church%E2%80%93Turing_thesis Church-Turing Thesis]''. While it is not proven, it is generally accepted that it is likely to be correct, with the result that ''[https://en.wikipedia.org/wiki/Turing_completenessTuring Completeness]'' became the benchmark for the computing power of any mechanical model.

It has to be understood that these models were 'mechanical' only in the sense of being built on a set of rules; they were not very useful for developing a physical computing device. Of them all, the UTM was the closest, as it was described in terms of an imaginary mechanism consisting of a reading and writing head and an infinitely long paper tape. While it was not a practical design, it demonstrated that the mechanical calculators of the time had a solid theoretical basis, and more importantly, re-introduced the concept (originally proposed by Charles Babbage) of a ''programmable'' computation device.

=== Early Electromechanical and Electronic Computers ===
Turing would go on to do significant work in the development of one of the first programmable  electronic computers, the ULTRA project's Colossus decryption machine originally conceived by Tom Flowers in early 1943. During this period, from the late 1930s to the early 1950s, several groups of designers seem to have developed similar ideas independently of each other: Konrad Zuse in Germany, John Atanasoff and Clifford Berry at Iowa State University, Presper Eckert and John Mauchly at Princeton, and Howard Aiken at Harvard, all rediscovered the principle of stored program computers between 1941 and 1943, a concept later codified in a white paper by a team led by John Von Neumann. Because of this, designs in which the program and data are stored in a shared memory became known as the ''Von Neumann architecture'', while those which have independent memories for program and data, such as the Zuse Z4 and the Harvard Mark 1, became known as the ''Harvard Architecture''.

=== Linear-Bounded Automata and the Limits of Physical Computation ===
After the war, the question of how electronic computing related to the theoretical models became important, as it wasn't clear if they were equivalent to the UTM, or if they possessed limits that the purely-imaginary models did not. The solution to this came down to an important property of both the Turing Machine and shared by all of the more abstract equivalents: the availability of at least one infinite resource, specifically the tape memory in the case of the Universal Turing Machine.

This led to the creation of a new model which could describe these limitations and allow them to determine what they were. This new model, the ''[https://en.wikipedia.org/wiki/Linear_bounded_automaton Linear-Bounded Automaton]'', a variant of the Turing machine with a finite available memory (it is usually described in terms of the reader having access to a finite, contiguous part of an infinite tape, hence the term 'linear bounded'). This allowed for study of the memory requirements of different classes of computations, and similar models were developed for studying the time requirements.

=== Finite Automata and the Chomsky Hierarchy ===
Meanwhile, work on understanding the more abstract forms continued, and a wide variety of Turing-complete computation models were formulated, along with many provably less powerful models. While the Church-Turing thesis served as a hypothetical limit for these, it remained a conjecture, and it wasn't entirely clear how the different models related to each other - especially those which were not Turing-complete.

In 1955, linguist [https://en.wikipedia.org/wiki/Noam_Chomsky Noam Chomsky] worked out an hierarchy for categorizing [https://en.wikipedia.org/wiki/Formal_grammar formal grammars] in terms of both the [https://en.wikipedia.org/wiki/Generative_grammar grammars capable of generating sentences in them], and the mathematical models for 'recognizing' grammatically-correct sentences mechanically. He demonstrated that not only did the same [Chomsky Hierarchy] apply to both how sentences can be generated mechanically, and how they can be recognized mechanically, but also that the levels of the hierarchy corresponded to certain degrees of computability - and specifically, that his [https://en.wikipedia.org/wiki/Unrestricted_grammar Type-0 grammars] (also called ''unrestricted grammars'') were ''recursively enumerable'', and hence required a Turing-complete mechanism to recognize or generate.

{{stub}}

=== Counters, Accumulators, Stacks, and Registers ===
However, the question of whether the machines could ''in principle'' be Turing-complete if given an infinite resource - memory, presumably - remained. This led to the creation of a set of abstract models for machines similar to those in actual use, but without the memory limits of a LBA.

{{stub}}

== The Random Access Machine model ==
The Random-Access Machine is an abstract model that closely resembles the structure of a conventional computer. It consists of a data memory divided into a set of cells, arranged linearly and monotonically (that is to say, the cells follow each other in a strictly numeric order, and each cell is the same 'size' in the address space - which doesn't ''necessarily'' reflect its capacity to hold data), where the cells can be accessed by a numeric address; an instruction memory, which may or may not be the same as the data memory, but is similarly structured; a set of operations which it can perform; a set of instructions, encoded in a form which could be held in a set of memory cells, which act to direct which operation to perform; and a Program Counter, which holds the address of a cell in the instruction memory which encodes which operation to perform next.


{{stub}}
````

<a id="w003852"></a>

## W003852 — Memory Architectures

Bron-ID: W003852
Namespace: Main (0)
Revisie: 18781 | Datum: 2016-01-07T00:38:42Z
Bron: https://wiki.osdev.org/Memory_Architectures
Revisie-URL: https://wiki.osdev.org/index.php?title=Memory_Architectures&oldid=18781
Doorverwijzing: Memory Architecture

````text

#REDIRECT [[Memory Architecture]]
````

<a id="w003853"></a>

## W003853 — CISC, RISC, and other CPU Architecture Concepts

Bron-ID: W003853
Namespace: Main (0)
Revisie: 19603 | Datum: 2016-07-25T17:40:05Z
Bron: https://wiki.osdev.org/CISC%2C_RISC%2C_and_other_CPU_Architecture_Concepts
Revisie-URL: https://wiki.osdev.org/index.php?title=CISC%2C_RISC%2C_and_other_CPU_Architecture_Concepts&oldid=19603
Doorverwijzing: Instruction Set Architecture

````text
#REDIRECT [[Instruction Set Architecture]]
{{stub}}
````

<a id="w003854"></a>

## W003854 — Instruction Set Architectures

Bron-ID: W003854
Namespace: Main (0)
Revisie: 18783 | Datum: 2016-01-07T00:40:17Z
Bron: https://wiki.osdev.org/Instruction_Set_Architectures
Revisie-URL: https://wiki.osdev.org/index.php?title=Instruction_Set_Architectures&oldid=18783
Doorverwijzing: Instruction Set Architecture

````text
#REDIRECT [[Instruction Set Architecture]]
````

<a id="w003855"></a>

## W003855 — Memory Architecture

Bron-ID: W003855
Namespace: Main (0)
Revisie: 30886 | Datum: 2026-07-12T13:42:29Z
Bron: https://wiki.osdev.org/Memory_Architecture
Revisie-URL: https://wiki.osdev.org/index.php?title=Memory_Architecture&oldid=30886

````text
'''Memory architecture''' describes the hardware and software mechanisms used to organize, address, translate, protect, cache, and access memory in a computer system.

For operating-system development, memory architecture matters because the kernel must convert the hardware-visible memory layout into safe and useful abstractions. These abstractions commonly include a physical page-frame allocator, kernel and user virtual address spaces, protected mappings, shared memory, and DMA buffers.

Memory architecture should not be confused with '''memory management'''. Memory architecture describes the mechanisms and address spaces provided by the machine, while memory management describes the policies and algorithms an operating system builds on top of them.

==Overview==

A processor does not simply read and write a block of memory.

Depending on the architecture and execution mode, a memory access may involve several mechanisms:

* an instruction-fetch or data-access path;
* segmentation or another architecture-specific address-generation stage;
* page-table translation and permission checks;
* a [[TLB|translation lookaside buffer]];
* one or more CPU caches;
* a memory controller, interconnect, or device bus;
* physical RAM, firmware-reserved memory, or a memory-mapped device.

The exact path depends on the architecture.

During early boot, a kernel may run with little or no memory protection. It must then discover the platform memory map, reserve memory already in use, initialize a physical memory allocator, construct page tables, enable or replace address translation, and establish protected kernel and user mappings.

==Instruction and data organization==

Computer systems are often described using one of three broad instruction/data memory organizations: Von Neumann, Harvard, and Modified Harvard.

===Von Neumann architecture===
In a '''Von Neumann architecture''', instructions and data are stored in the same memory system and are addressed through the same memory space. Code is data: the bytes that encode instructions can, in principle, be read or written like other memory.

This model is close to the programmer visible model used by many general purpose systems. Executable files are loaded into memory, the processor fetches instructions from memory, and ordinary load/store instructions access data from the same general address space.

For OS development, this model explains why a kernel can load executable code, relocate it, copy it, map it into an address space, or mark it executable or non executable using page permissions.
===Harvard architecture===
In a '''Harvard architecture''', instruction memory and data memory are separate. Instruction fetches and data accesses use distinct storage or distinct address spaces.

This separation is common in some microcontrollers and embedded systems. It can simplify certain hardware designs and allow instruction fetches and data accesses to occur independently. However, it can also make some operations less direct, such as treating program code as ordinary data.

For OS development, pure Harvard systems require special attention when loading code, modifying code, or implementing executable memory. A mechanism may be required to copy data into instruction memory or to synchronize instruction and data views.  

===Modified Harvard architecture===
A '''Modified Harvard architecture''' combines features of both approaches. A common modern design has separate instruction and data caches near the CPU, while still presenting a mostly unified memory model to software.

For example, a processor may have separate L1 instruction and data caches, but both caches ultimately refer to the same physical memory. This can improve performance while preserving the convenience of a unified address space.

Modified Harvard design should not be confused with virtual memory. Split instruction and data caches concern how memory is fetched and cached. Virtual memory, demand paging, and protection are mainly provided by address translation hardware such as an MMU.

==Address Spaces==
An '''address space''' is the set of addresses meaningful to some component of the system. A modern OS often has to reason about several address spaces at once.
===Physical address space===
The '''physical address space''' is the address space used by the processor, memory controller, and platform hardware after any virtual-to-physical translation has taken place. Physical addresses may refer to RAM or to platform-defined regions such as device memory, firmware data, and reserved areas.

Physical address space must not be confused with installed RAM. A machine may support a larger physical address space than the amount of RAM installed, and some ranges inside that space may not be usable memory. For example, on x86 PCs, memory above the first megabyte is not guaranteed to be contiguous, RAM may be remapped above 4 GiB, and some physical address ranges may be assigned to memory-mapped devices or firmware data structures.

====Address space width====
The size of the physical address space is limited by the number of physical address bits supported by the processor. This limit is independent of the amount of RAM installed.

For example, 32-bit x86 without Physical Address Extension normally uses a 32-bit physical address space. With PAE, x86 can address physical memory beyond 4 GiB. In x86-64 long mode, the architectural physical address space can be up to 52 bits, although individual processors may implement fewer physical address bits.

Physical address width matters because it limits which physical frame numbers can appear in page-table entries and which physical ranges can be covered by identity mappings or direct physical memory mappings. It can also affect device access: some devices can only perform DMA to a smaller address range than the CPU can address, requiring low memory allocations, bounce buffers, or an IOMMU.

====Memory map====
The kernel must not assume that every physical address is usable RAM. During boot, the bootloader or firmware provides a memory map describing which physical ranges are usable and which are reserved.

On BIOS-based x86 systems, the common mechanism is [[Detecting Memory (x86)#Getting an E820 Memory Map|INT 15h, EAX=0xE820]]. On UEFI systems, the firmware provides a [[UEFI|UEFI memory map]]. Boot protocols such as [[Multiboot]], [[Limine]], and [[BOOTBOOT]] may also pass a memory map to the kernel.

Only regions explicitly marked usable should be handed to the physical page-frame allocator. The kernel must also reserve memory occupied by the kernel image, boot modules, firmware tables, framebuffers, page tables, and other data structures created during boot.
===Virtual address space===
A '''virtual address space''' is the address space seen by software when address translation is enabled. When a program executes a load or store, the address in the instruction is a virtual address. The [[Memory Management Unit|MMU]] translates that virtual address into a physical address before the access reaches memory or any memory-mapped device.

Virtual address spaces allow each process to have its own independent view of memory. Two processes may use the same virtual address while mapping it to entirely different physical pages. The kernel may also map the same
physical page at multiple virtual addresses, share pages between processes,
or leave ranges deliberately unmapped so that invalid accesses generate
faults that the kernel can handle.

Virtual memory is a central concept that is used in many other computer system ideas.

Translation results are cached in a [[TLB|translation lookaside buffer]].
When the kernel modifies a mapping it must invalidate the relevant TLB
entries, otherwise stale translations may be used. On SMP systems this requires notifying other CPUs as well.

==== Canonical addresses on x86-64 ====

On x86-64, not every 64-bit value is a valid virtual address. With four-level paging, bits 63 through 48 must be copies of bit 47. With five-level paging, bits 63 through 57 must be copies of bit 56. Addresses satisfying this rule are called '''canonical addresses'''. Accessing a non-canonical address causes a general-protection fault.

This is why many higher-half kernels are placed near the top of the canonical address space, rather than in the middle of the 64-bit range.
===Bus and DMA address space===
A device does not necessarily see memory in the same way as the CPU. Devices that perform direct memory access use DMA addresses, which may be physical addresses or may be translated through an IOMMU.

Some devices can only address part of physical memory. For example, a device may be limited to 32-bit DMA addresses even when the CPU supports a larger physical address space. 
In that case, the kernel must allocate DMA buffers from a reachable range, use bounce buffers, or configure an IOMMU mapping.

This distinction is important because changing CPU page tables does not automatically change what a device can access. 
The CPU virtual address space, the physical address space, and a device DMA-visible address space are related but not identical.
===I/O address spaces===
Some architectures provide special address spaces for device I/O.

On x86, port-mapped I/O uses a separate I/O port address space accessed with instructions such as <code>in</code> and <code>out</code>. This is distinct from ordinary memory accesses.

Many modern devices instead use memory-mapped I/O. In MMIO, device registers appear inside the physical address space and are accessed with ordinary load and store instructions after the kernel maps the relevant physical range. 
MMIO regions usually require appropriate cache attributes, because device registers must not be cached like normal RAM.
==Memory hierarchy==

Real systems contain a hierarchy of storage with different sizes, speeds, and visibility:

* CPU registers;
* L1 instruction and data caches;
* larger shared caches;
* main memory;
* persistent storage;
* device-local memory.

The OS usually does not allocate registers or ordinary CPU cache lines directly, but it must still account for the hierarchy. This is because page size, alignment, cacheability, DMA requirements, and memory ordering can all affect correctness and performance.

For a simple kernel, the memory hierarchy can often be ignored at first. As the kernel grows, cache behavior becomes important for page-table updates, DMA buffers, memory-mapped device registers, atomic operations, and multiprocessing.

For device drivers and multiprocessing kernels, memory hierarchy details become especially important. Device memory may require uncacheable or write-combining mappings. DMA buffers may need alignment, cache flushing, or coherency management. Page-table updates may require TLB invalidation, and atomic operations may require memory-ordering guarantees.

==Memory protection and translation==

===Segmentation===
{{Main|Segmentation}}
Segmentation divides an address space into regions described by segment bases, limits, types, and privilege rules.

On 32-bit x86, segmentation can participate directly in isolation and address translation. Many modern operating systems instead use a mostly flat segmentation model and rely on paging for fine-grained memory management.

In x86-64 long mode, most segment bases and limits are ignored, although FS and GS retain programmable bases and are commonly used for thread-local or per-CPU data.
===Paging===
{{Main|Paging}}
Paging divides virtual and physical memory into fixed-size units called pages and page frames. Page tables describe how virtual pages map to physical frames and what operations are permitted.

Page-table entries commonly encode permissions and attributes such as:

* present or valid;
* readable or writable;
* user-accessible or supervisor-only;
* executable or non-executable;
* accessed or dirty;
* cache or memory type;
* page size.

Paging allows an operating system to implement process isolation, demand paging, copy-on-write, shared memory, memory-mapped files, guard pages, kernel higher-half mappings, and sparse address spaces.

An invalid or disallowed access raises a page fault or architecture-equivalent exception. The kernel must determine whether the fault is recoverable, such as by allocating a page, or fatal to the current process.

===Translation lookaside buffers===
{{Main|TLB}}
A '''translation lookaside buffer''' caches recent address translations and permission information. Without it, many memory accesses would require repeated page-table walks.

When the kernel changes or removes a mapping, it must invalidate any stale TLB entry that could still authorize or redirect an access. On a multiprocessor system, the same translation may be cached by several CPUs, requiring a '''TLB shootdown''' or an architecture-specific address-space synchronization mechanism.

The exact invalidation required depends on the change. Creating a previously absent mapping, tightening permissions, replacing a physical frame, and destroying an address space may have different rules on different architectures.
===IOMMU===
{{Main|IOMMU}}
An '''input-output memory management unit''' translates device DMA addresses and can restrict which memory a device may access.

An IOMMU can provide:

* DMA isolation;
* device-specific address spaces;
* access to high physical memory for limited devices;
* scatter-gather mappings;
* safer device assignment to virtual machines.

An IOMMU does not replace CPU page tables. It performs a separate translation for device-originated accesses, and the kernel must manage both sets of mappings consistently.

===Memory types and ordering===

Memory accesses are not always observed in program order, and not every physical range behaves like normal RAM.

Architectures provide memory types, barriers, fences, and atomic operations so software can control caching and ordering. These mechanisms are especially important for:

* device registers;
* shared data between CPUs;
* lock-free data structures;
* DMA buffers;
* page-table updates;
* self-modifying code.

The required rules are architecture-specific. A portable kernel abstraction must still eventually map each operation to the guarantees provided by the target architecture.

==Typical kernel initialization sequence==

A kernel commonly performs memory initialization in approximately the following order:

# Receive or locate the platform memory map.
# Identify the kernel image, modules, firmware structures, framebuffer, and other occupied regions.
# Establish a temporary or bootstrap allocator.
# Normalize the memory map and reserve every page already in use.
# Initialize the physical page-frame allocator.
# Construct the kernel's page tables.
# Map the kernel image with appropriate read, write, and execute permissions.
# Create mappings for physical memory, devices, and required firmware structures.
# Load or activate the new translation structures.
# Initialize the kernel virtual-memory allocator.
# Install page-fault handling.
# Create per-process address spaces and user mappings.
# Initialize DMA and IOMMU support as required by devices.

The exact order varies by architecture and boot protocol. In particular, enabling paging for the first time, replacing bootloader page tables, or leaving UEFI boot services may impose additional requirements.

==Common mistakes==

Frequent memory-architecture errors in early kernels include:

* assuming installed RAM is contiguous;
* treating the entire physical address space as usable RAM;
* failing to reserve the kernel, modules, page tables, or framebuffer;
* storing physical addresses in pointer types without an explicit mapping;
* confusing virtual, physical, and DMA addresses;
* using an address after the mapping that made it valid has been removed;
* forgetting page alignment or integer-overflow checks;
* modifying page tables without required TLB invalidation;
* mapping writable data as executable;
* mapping MMIO with ordinary write-back caching;
* reclaiming bootloader or firmware memory while references to it remain;
* assuming all CPUs automatically observe page-table or cache changes;
* using the architectural maximum address width instead of the implemented width.

Clear naming helps prevent these errors. Kernels often use distinct types or naming conventions for virtual addresses, physical addresses, page-frame numbers, and DMA addresses.

==See also==

* [[Memory Management]]
* [[Memory Management Unit]]
* [[Paging]]
* [[Segmentation]]
* [[Page Frame Allocation]]
* [[Virtual Memory]]
* [[TLB]]
* [[IOMMU]]
* [[Memory Map]]
* [[Higher Half Kernel]]
* [[Direct Memory Access]]
* [[Memory Mapped Registers]]

[[Category:Memory management]]
[[Category:Theory]]
````

<a id="w003856"></a>

## W003856 — Instruction Set Architecture

Bron-ID: W003856
Namespace: Main (0)
Revisie: 27928 | Datum: 2023-07-09T14:57:55Z
Bron: https://wiki.osdev.org/Instruction_Set_Architecture
Revisie-URL: https://wiki.osdev.org/index.php?title=Instruction_Set_Architecture&oldid=27928

````text
{{stub}}

This page is intended to provide an explanation of some of the various types of instruction sets. The instruction sets are listed starting with the simplest and ending with the most complex instruction set.<br>
See also [[Historical Notes on CISC and RISC]].

==Church-Turing Thesis==

Alan Turing (1912-1954) was highly influential during the early days of computer science. The Church-Turing thesis states that any algorithm computable by humans (without time and memory limitations and without failures) is computable by a Turing machine.<br>
To be Turing complete, a machine must be able to load data, change it according to a set of rules, and store it afterwards. Loading and storing values is self explanatory, the interesting part is the processing.<br>
Common algorithms:
*Logic functions: One might not notice, but e.g. brains use in daily challenges for decisions simple state machines and every state machine can be expressed by logic functions.
*Comparisons: Is this red brighter than this? Answer: Yes or No. See example above. This is implemented by an addition or rather a subtraction. Can be implemented by logical functions.
*Multiplications/divisions: Used f.e. in case of applying weights. Can be implemented by addition and subtraction, and addition can be implemented by logic functions, so ...
*Integration/differentiation: Any physic law may be implemented by using integration and differentiation. Can be implemented by multiplication or rather division, and this can be implemented by arithmetic and logic functions.
As you might have noticed everything can be implemented by logic functions. This is important: To be able to do any logic function means to be Church-Turing mighty.

To be exact, only a single logic function, such as [[wikipedia:Sheffer stroke|NAND]], with two inputs and one output is needed. Other sets of "complete" logic functions can be found, but NAND is the most common.

Any Instruction Set Architecture (ISA) is Church-Turing mighty.

==Flynn's Bottleneck and Fisher's Optimism==

M. J. Flynn (*1934) found, in 1970, a very interesting fact: If one fetches (loads) only one instruction per cycle, one will never get more than one executed instruction per cycle. (This is valid for each physical ALU.)<br>
J. A. Fisher (*1946) argued in 1984 that an array (packed) data structure could achieve more executed instructions per cycle than one.<br>
This will become important speaking when about RISC/CISC dis-/advantages.

==One Instruction Set==

The so called Ultimate Reduced Instruction Set Computer (URISC) or One Instruction Set Computer (OISC) is programmed by only one instruction. This instruction must make it possible to decide, to move data, to calculate, and to jump to different targets in the instruction stream. This is only possible with a complex instruction. 

Applications for a computer programmed by this ISA have huge programs, so this ISA is only of theoretical interest. For further information refer to [[wikipedia:One instruction set computer|One instruction set computer]] on Wikipedia.

==Minimal Instruction Set==

It is defined by less than 32 instructions (one can't really distinguish between MISC and RISC). Mostly, MISCs are Stack machines. Owing to missing security features and the huge program size this ISA is not used anymore, but its instructions are included in more sophisticated ISAs. For further information refer to [[Wikipedia:Minimal instruction set computer|Minimal instruction set computer]]. This type of instruction set has been used on some early computers such as [[wikipedia:ENIAC|ENIAC]].

==Reduced Instruction Set==

A RISC provides fast and simple basic instructions, e.g. conditional jumps, logic functions, addition/subtraction, multiplication/division, etc. It's execution environment is simple, because RISC must not provide complex instructions. This may cause security issues. Oftentimes, Flynn's bottleneck applies, because data can't be processed in advanced structures, as the ISA must be simple by definition. Examples of RISC ISAs include previous [[ARM Overview|ARM]] and [[MIPS Overview|MIPS]] generations.

==Complex Instruction Set==

A CISC instantiates simple and additional complex instructions. It comes mostly with different execution environments and security features. Especially streaming extensions like SSE must be named.
They are the reason why RISC-only processors almost disappeared nowadays. Flynn's bottleneck applies, but by using streaming extensions the chance to reduce its influence and switch to Fisher's optimism grows fast. Other advanced features may reduce memory access, and  will therefore induce idle times in the CPU. Clearly the advantage of the CISC architectures are their capabilities, but that complexity is a disadvantage as well, and can result in serious security issues that it needs to handle. CISC architectures such as [[wikipedia:Motorola 68000 series|M68K]] or [[wikipedia:MOS Technology 6502|6502]] were common in the early days of computing, when optimising compiler technology was less advanced. A CISC architecture still in use is x86 (including x86_64).

==Hybrid Instruction Set==

Modern CPUs are built upon a hybrid RISC-CISC architecture. RISC processors can be built more easily and clocked faster, but lack the advanced instructions of a CISC. To get the best of both worlds, a hybrid is built. A RISC is used as the ALU, and is wrapped by a CISC environment. Any instruction is interpreted by this CISC and is split into one or more sub-instructions, called "micro-opcodes", for the RISC. Additionally, the CISC-wrapping provides security, along with operating system stability and control. Nowadays, only microcontrollers use pure RISC; any other CPU is more or less a hybrid RISC-CISC CPU.


[[Category:Instruction Set Architecture]]
````

<a id="w003857"></a>

## W003857 — Execution Architecture

Bron-ID: W003857
Namespace: Main (0)
Revisie: 22214 | Datum: 2018-03-16T00:31:24Z
Bron: https://wiki.osdev.org/Execution_Architecture
Revisie-URL: https://wiki.osdev.org/index.php?title=Execution_Architecture&oldid=22214

````text
{{stub}}
This page is intended to provide an overview of the various execution architecture principles, including pipelining, branch prediction, out or order operation, etc.
````

<a id="w003858"></a>

## W003858 — Multiprocessing Architecture

Bron-ID: W003858
Namespace: Main (0)
Revisie: 18786 | Datum: 2016-01-07T00:45:11Z
Bron: https://wiki.osdev.org/Multiprocessing_Architecture
Revisie-URL: https://wiki.osdev.org/index.php?title=Multiprocessing_Architecture&oldid=18786

````text
{{stub}}
This page is intended to provide an overview of the various forms of multiprocessing (e.g., SIMD, MIMD, SMP), and the issues relating to them (e.g., vector processing, systolic array processing, non-uniform memory access) and how they relate to different types of multiprocessor/multicore systems.
````

<a id="w003860"></a>

## W003860 — User:Songziming

Bron-ID: W003860
Namespace: User (2)
Revisie: 18809 | Datum: 2016-01-09T11:43:32Z
Bron: https://wiki.osdev.org/User:Songziming
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASongziming&oldid=18809

````text
Hello, my name is Song Ziming, a student of software engineering.

My OS project is [https://github.com/songziming/wheel Wheel], which means "reinvent the wheel". Wheel is going to be an 64 bit mode OS with SMP support, but it's far from finishing.

==My Pages==

# [[User:Songziming/OO]]
# [[User:Songziming/ACPICA]]
# [[User:Songziming/Junk]]
# [[User:Songziming/OSDev]], in Chinese.
````

<a id="w003861"></a>

## W003861 — User:Songziming/OO

Bron-ID: W003861
Namespace: User (2)
Revisie: 23534 | Datum: 2019-04-02T15:18:41Z
Bron: https://wiki.osdev.org/User:Songziming/OO
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASongziming%2FOO&oldid=23534

````text

````

<a id="w003862"></a>

## W003862 — User:Songziming/Junk

Bron-ID: W003862
Namespace: User (2)
Revisie: 18831 | Datum: 2016-01-09T17:00:35Z
Bron: https://wiki.osdev.org/User:Songziming/Junk
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASongziming%2FJunk&oldid=18831

````text
This page contains my junk, you may ignore it.

==OS项目代码组织==

OS一般不是在IDE中的，而是用Makefile这类工具进行管理，所以如何管理工程就需要动一番脑筋了。

目前我的原则是，模块化组织，头文件/源文件共存。

==Printf==

printf是C语言里的输出函数，在内核中如果实现一个类似功能的函数，可以极大方便调试。和printf一起的还有一组函数，它们拥有类似的用法。

在内核中实现printf输出的时候，需要注意的是缓冲区。缓冲区可以由printf函数实现，也可以由目标流实现（stdout、stderr）。对于一个printf调用，如果不执行一遍，就不知道这个字符串格式化之后的长度是多少，因此需要多长的缓冲区也是不确定的。而且，在printf中不适合经常使用动态内存。

使用双缓冲区，当缓冲区发生切换的时候，就将已满的一个缓冲区输出到目标屏幕上。关键是缓冲区的操作并不是一次一个字节的复制，而是一次复制一小段字符串。
````

<a id="w003863"></a>

## W003863 — User:Songziming/ACPICA

Bron-ID: W003863
Namespace: User (2)
Revisie: 18806 | Datum: 2016-01-07T20:36:43Z
Bron: https://wiki.osdev.org/User:Songziming/ACPICA
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASongziming%2FACPICA&oldid=18806

````text
This page contains information of using ACPICA in your own OS. also reference [[ACPICA]].

==About ACPICA==

ACPICA should be compiled inside kernel.

====Prepare ACPICA Source Code====

First download the source code from Github

<source lang="bash">
git clone https://github.com/acpica/acpica.git
</source>

Then create a directory for ACPICA

<source lang="bash">
cd /path/to/myOS
mkdir acpica && cd acpica
</source>

Copy ACPICA files

<source lang="bash">
cp /path/to/acpica/source/include/* .
cp /path/to/acpica/source/components/{dispatcher,events,executer,hardware,namespace,parser,resources,tables,utilities}/* .
</source>

My Makefile will automatically find the source file and headers and compile them, so no need to add Makefile rules.

====Configuration====

The configuration of ACPICA is done via editing header files. One important header file is platform/acenv.h.
````

<a id="w003864"></a>

## W003864 — User:Songziming/OSDev

Bron-ID: W003864
Namespace: User (2)
Revisie: 18812 | Datum: 2016-01-09T13:07:54Z
Bron: https://wiki.osdev.org/User:Songziming/OSDev
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASongziming%2FOSDev&oldid=18812

````text
本文是开发OS的教程，也是本人开发OS过程的总结。OS开发的自由度很高，选择也很多，本文的内容只是其中一种可能。

==目标平台==

OS是计算机软件层次中最底层的部分，直接与硬件交互。因此在开发OS的时候，首先要明确OS的目标平台，也就是这个OS准备运行在哪种硬件上，电脑？手机？平板？树莓派……不同的硬件平台差异很大，因此确定下其中一种才能开始开发。

''关于跨平台'' 不少的OS是跨硬件平台的，也就是支持多种硬件。我认为多硬件支持确实是个好东西，但是应该在其中一种硬件的支持实现了之后再进行考虑。编写OS的过程中，如果有支持多种硬件的计划，应该在OS的设计上予以考虑，例如剥离硬件相关部分，屏蔽底层细节。

本文剩下部分，只考虑AMD64/Intel64的CPU，IBM-PC硬件平台。
````

<a id="w003865"></a>

## W003865 — User:Lukaandjelkovic

Bron-ID: W003865
Namespace: User (2)
Revisie: 19273 | Datum: 2016-05-25T09:00:04Z
Bron: https://wiki.osdev.org/User:Lukaandjelkovic
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALukaandjelkovic&oldid=19273

````text
Blue Desert System: [http://bluedesert.netai.net Link]<br>
First name : Luka<br>
Last name : Andjelkovic<br>
````

<a id="w003866"></a>

## W003866 — User talk:Lukaandjelkovic

Bron-ID: W003866
Namespace: User talk (3)
Revisie: 19233 | Datum: 2016-05-15T14:02:45Z
Bron: https://wiki.osdev.org/User_talk:Lukaandjelkovic
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ALukaandjelkovic&oldid=19233

````text
== Warning ==

Stop spamming the wiki with irrelevant links to your OS, or you're going to be banned. --[[User:Sortie|Sortie]] 08:49, 15 May 2016 (CDT)
:How do you noticed that fast!? -- OK, deleted all of ads for my OS. --[[User:Lukaandjelkovic|Lukaandjelkovic]] 08:52, 15 May 2016 (CDT)
::Normally. There's a bot that posts all wiki edits to #osdev. But don't spam. It's however perfectly OK to add your OS the [[Projects]] page under a real entry, like everything else. It's also OK to link to the source code of your OS in articles, if you have a really good implementation of something that you want people to study and maybe copy. Note that I can't verify that your OS is real from your website, as there's no downloads or source code. I can see that you're going to release it in a couple of days, congratulations on that. Be sure to announce it on the forum if you want people to have a look at it.
````

<a id="w003868"></a>

## W003868 — Talk:CPU Registers x86-64

Bron-ID: W003868
Namespace: Talk (1)
Revisie: 19600 | Datum: 2016-07-25T12:30:31Z
Bron: https://wiki.osdev.org/Talk:CPU_Registers_x86-64
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACPU_Registers_x86-64&oldid=19600

````text
== Most of this page text was copied from [[CPU Registers x86]] ==

The registers in the x86_64 architecture differ from the registers in the x86 architecture. If you know more about the x86_64 architecture, please check the page.<br>[[User:Glauxosdever|Glauxosdever]] 05:22, 10 January 2016 (CST)

:This page is correct. The registers are not execution-environment-specific tagged, but it is correct that x86-64 provides all these registers. You can't use a third of them in 32e-mode (64-bit mode) or with no effect.<br>However, I suggest renaming the page in "registers in long-mode" or something like that or deleting the x86-page. (Its fully covered here.)<br>[[User:New16|New16]] 17:14, 23 July 2016 (CDT)

::I would suggest you reread the page fully. Long mode registers differ from protected mode registers (for example look at GDTR, IDTR - 64-bit versions are not even present in this page).<br>Also, no. The x86 page describes purely 32-bit registers. I'd suggest you don't delete that.<br>[[User:Glauxosdever|Glauxosdever]] 07:16, 24 July 2016 (CDT)

:::Done. However, it was correct, but incomplete. This page was obviously created to cover ALL registers of the x86-64 architecture, including real-mode and protected-mode ([[CPU Registers x86]]) registers.<br>So that one must only differentiate in tables when the corresponding register was introduced. (Just a suggestion. And you said no. Got it.)<br> [[User:New16|New16]] 07:30, 25 July 2016 (CDT)
````

<a id="w003869"></a>

## W003869 — CPU Registers x86 64

Bron-ID: W003869
Namespace: Main (0)
Revisie: 18836 | Datum: 2016-01-10T11:29:17Z
Bron: https://wiki.osdev.org/CPU_Registers_x86_64
Revisie-URL: https://wiki.osdev.org/index.php?title=CPU_Registers_x86_64&oldid=18836
Doorverwijzing: CPU Registers x86-64

````text
#REDIRECT [[CPU_Registers_x86-64]]
````

<a id="w003870"></a>

## W003870 — GCC and Libc++

Bron-ID: W003870
Namespace: Main (0)
Revisie: 28088 | Datum: 2023-07-09T21:21:49Z
Bron: https://wiki.osdev.org/GCC_and_Libc%2B%2B
Revisie-URL: https://wiki.osdev.org/index.php?title=GCC_and_Libc%2B%2B&oldid=28088

````text
For the most part, the information in [[Libsupcxx]] seems to be broken at the moment as GCC has changed a lot since this article was first written. One alternative is to use libc++ instead. libc++ is the c++ library provided by the llvm/clang compiler. Although this could be used with with clang, the following information is with respect to GCC. 

== Prerequisites ==

Before you can compile anything, your going to need a cross compiler. The following assumes that you have created a GCC cross compiler using [[GCC_Cross-Compiler]] with the following:

    export PREFIX="$HOME/opt/cross"
    export TARGET=x86_64-elf
    export PATH="$PREFIX/bin:$PATH"

Note that you might also want to configure GCC to use multilib support so that you can turn off the red zone with respect to x86_64 [[Libgcc_without_red_zone]]. The code in this tutorial assumes that you have done that as it's pretty much needed if you want to run code in your kernel using x86_64. 

Finally you will also need a libc as libc++ uses libc functions. You can create your own libc if you want, or you can use an existing one [[C_Library]]. This article assumes that your using newlib, so you will see some #defines that are specific to newlib which you can ignore if your using your own libc or something else. The main point here is you need to have a libc available of libc++ to use as needed. 

To compile libc++ you will need both libcxx and libcxxabi. Libcxxabi provides all of the abi functions that are needed to make libcxx work. Note that this is similar to libsupc++. 

== Donwload ==

Before we can start, download all of the sources that we need, and setup our build environment

    wget ftp://sourceware.org/pub/newlib/newlib-2.3.0.20160104.tar.gz
    git clone http://llvm.org/git/libcxxabi
    git clone http://llvm.org/git/libcxx

    tar xvf newlib*.tar.gz    
    mv newlib-*/ newlib/

    mkdir build-newlib

== Flags ==

Your going to want to keep everything consistent. The newlib #defines setup newlib to compile for x86_64, as well as enable the portions of newlib that are needed by libc++. For example, libc++ will need functions like strtold, which on x86_64 requires you to enable long double support in newlib (likely you will never use this in a kernel, but it's needed to compile). Libc++ also needs a timer of some sort, and it's likely that you will be providing your own malloc/free. In addition, we disable threading support (also not usually needed in a kernel), and we tell libc++ to use posix timers and enable support for them in newlib.  

    export NEWLIB_DEFINES="-D_HAVE_LONG_DOUBLE -D_LDBL_EQ_DBL -D_POSIX_TIMERS -U__STRICT_ANSI__ -DMALLOC_PROVIDED"
    export LIBCXXABI_DEFINES="-DLIBCXXABI_HAS_NO_THREADS"
    export LIBCXX_DEFINES="-D_LIBCPPABI_VERSION=1002 -D_NEWLIB_VERSION -D_LIBCPP_HAS_NO_THREADS -D_LIBCPP_HAS_NO_MONOTONIC_CLOCK"

With respect to cflags, since we are on x86_64 we need -fpic, and since this is a kernel, we need -mno-red-zone. Since this is a custom GCC cross compiler, we use -ffreestanding to enable GCC's built-in includes like stdint.h etc... And then finally we disable thread statics which prevents GCC from creating calls to thread guards, and disable cxa-atexit in favor of newlib's version (which could be removed if you have your own global constructor / destructor logic that is more c++ friendly). 

    export CFLAGS="-ffreestanding -fpic -mno-red-zone $NEWLIB_DEFINES"
    export CXXFLAGS="-std=c++14 -fno-use-cxa-atexit -fno-threadsafe-statics $CFLAGS"

== Patch Libcxx ==

At the time of writing, there are two issues with libcxx that need to be addressed. The first is libcxx assumes that some cmath functions are actually functions and not macros, while newlib uses macros, so we need to remove the use of "using" for these functions. Finally, libcxx has some c++17 functions that don't have a clean way to #define them out, and GCC currently doesn't have support for these (specifically uncaught exceptions). The following patch cleans these issues up:

    diff -Naur libcxx/include/cmath libcxx_patched/include/cmath
    --- libcxx/include/cmath	2016-01-13 09:54:11.630532663 -0800
    +++ libcxx_patched/include/cmath	2016-01-13 09:53:42.351909907 -0800
    @@ -306,6 +306,7 @@
     
     _LIBCPP_BEGIN_NAMESPACE_STD
     
    +#ifndef _NEWLIB_VERSION
     using ::signbit;
     using ::fpclassify;
     using ::isfinite;
    @@ -326,6 +327,7 @@
     #ifndef _AIX
     using ::abs;
     #endif
    +#endif
     
     #ifndef __sun__
     using ::acos;
    diff -Naur libcxx/src/exception.cpp libcxx_patched/src/exception.cpp
    --- libcxx/src/exception.cpp	2016-01-13 09:54:11.658531346 -0800
    +++ libcxx_patched/src/exception.cpp	2016-01-13 09:52:59.885914438 -0800
    @@ -106,22 +106,22 @@
     
     int uncaught_exceptions() _NOEXCEPT
     {
    -#if defined(__APPLE__) || defined(_LIBCPPABI_VERSION)
    -   // on Darwin, there is a helper function so __cxa_get_globals is private
    -# if _LIBCPPABI_VERSION > 1101
    -    return __cxa_uncaught_exceptions();
    -# else
    -    return __cxa_uncaught_exception() ? 1 : 0;
    -# endif
    -#else  // __APPLE__
    -#   if defined(_MSC_VER) && ! defined(__clang__)
    -        _LIBCPP_WARNING("uncaught_exceptions not yet implemented")
    -#   else
    -#       warning uncaught_exception not yet implemented
    -#   endif
    +// #if defined(__APPLE__) || defined(_LIBCPPABI_VERSION)
    +//    // on Darwin, there is a helper function so __cxa_get_globals is private
    +// # if _LIBCPPABI_VERSION > 1101
    +//     return __cxa_uncaught_exceptions();
    +// # else
    +//     return __cxa_uncaught_exception() ? 1 : 0;
    +// # endif
    +// #else  // __APPLE__
    +// #   if defined(_MSC_VER) && ! defined(__clang__)
    +//         _LIBCPP_WARNING("uncaught_exceptions not yet implemented")
    +// #   else
    +// #       warning uncaught_exception not yet implemented
    +// #   endif
         fprintf(stderr, "uncaught_exceptions not yet implemented\n");
         ::abort();
    -#endif  // __APPLE__
    +// #endif  // __APPLE__
     }
     
== Compile NewLib ==

    pushd build-newlib
    ../newlib/configure --target=$TARGET --prefix=$PREFIX
    make all
    popd

== Compile Libcxxabi ==

    pushd libcxxabi/src
    ~/opt/cross/bin/x86_64-elf-g++ -I../../libcxx/include/ -I../../newlib/newlib/libc/include/ -I../include/ $LIBCXXABI_DEFINES $CXXFLAGS -c *.cpp
    ~/opt/cross/bin/x86_64-elf-ar rcs libcxxabi.a *.o
    popd

== Compile Libcxx ==

    pushd libcxx/src
    ~/opt/cross/bin/x86_64-elf-g++ -I../../libcxxabi/include/ -I../../newlib/newlib/libc/include/ -I../include/ $LIBCXX_DEFINES $CXXFLAGS -c *.cpp
    ~/opt/cross/bin/x86_64-elf-ld -shared *.o -o libcxx.so -L../../libcxxabi/src/ -lcxxabi -L/home/user/sandbox/build-newlib/x86_64-elf/no-red-zone/newlib -lc -L/home/user/opt/cross/lib/gcc/x86_64-elf/5.2.0/no-red-zone/ -lgcc
    popd

== Notes ==

From here you have a lot of options. The above code compiles libc++ as a shared library, but you could compile it as a static library if thats what you need (the command would be similar to how we compiled libcxxabi). Likely these includes / libraries should be copied to your systroot, and then recompile GCC to see your new sysroot using "--with-sysroot=". Keep in mind that the majority of libc++ is a set of headers, and a lot of these headers can be used without compiling any code at all. For example, if all you need is the stdlib containers, you probably could use libc++ without any compilation at all (libcxxabi does this). Also note that, even though we use newlib here, there are still some functions that you will have to implement yourself. An easy way to see what symbols have left to be defined would be to:

    readelf libcxx.so -aW | grep UND

[[Category:C++]]
[[Category:Compilers]]
````

<a id="w003871"></a>

## W003871 — File:CFS64.jpg

Bron-ID: W003871
Namespace: File (6)
Revisie: 18844 | Datum: 2016-01-17T11:47:24Z
Bron: https://wiki.osdev.org/File:CFS64.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACFS64.jpg&oldid=18844

````text
Screenshot of grub cfs64 module - ChaiOS file system.
````

<a id="w003872"></a>

## W003872 — User:Bellezzasolo/TFS

Bron-ID: W003872
Namespace: User (2)
Revisie: 18846 | Datum: 2016-01-17T11:48:31Z
Bron: https://wiki.osdev.org/User:Bellezzasolo/TFS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABellezzasolo%2FTFS&oldid=18846
Doorverwijzing: User:Bellezzasolo/CFS

````text
#REDIRECT [[User:Bellezzasolo/CFS]]
````

<a id="w003873"></a>

## W003873 — User talk:Bellezzasolo/TFS

Bron-ID: W003873
Namespace: User talk (3)
Revisie: 18848 | Datum: 2016-01-17T11:48:31Z
Bron: https://wiki.osdev.org/User_talk:Bellezzasolo/TFS
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ABellezzasolo%2FTFS&oldid=18848
Doorverwijzing: User talk:Bellezzasolo/CFS

````text
#REDIRECT [[User talk:Bellezzasolo/CFS]]
````

<a id="w003874"></a>

## W003874 — Notable Projects

Bron-ID: W003874
Namespace: Main (0)
Revisie: 30216 | Datum: 2026-03-14T22:03:52Z
Bron: https://wiki.osdev.org/Notable_Projects
Revisie-URL: https://wiki.osdev.org/index.php?title=Notable_Projects&oldid=30216

````text
__NOTOC__
This page maintains a list of mature hobby operating system projects considered to be notable. A hobby operating system is considered notable if it made some considerable progress (e.g. has had a full release, is self-hosting, has been reported on outside of the hobby OSdev world, etc.) or if it has achieved such notability within the operating system development community as a useful answer to the question, "I want to see what kind of operating systems you guys have built". It should have a relatively stable kernel and drivers, and some userspace applications. They are usually written by the members of this community, but not limited to.

Everyone is welcome to add their own projects to the regular [[Projects]] list of all hobbyist operating systems.

More advanced and successful alternative operating systems are listed at [[Advanced Projects]] page.

== Notable Hobby Operating Systems ==

{{OSProject
|OSName=BareMetal
|OSDescription=BareMetal is a 64-bit OS for x86-64 based computers. The OS is written entirely in Assembly while applications can be written in Assembly or C/C++. The two main purposes of BareMetal are for educational uses in learning low-level OS programming in 64-bit Assembly and to be used as a base for a high-speed data processing node. Source code is well documented and freely available. As of version 2026.01 BareMetal officially supports multiple processors, memory management, and Ethernet communication.
|Contact=Ian Seyler (ian.seyler at returninfinity.com)
|URL=http://www.returninfinity.com/
|Status=Active
|License=Open source (MIT)
}}

{{OSProject
|OSName=[[User:max/Ghost Kernel|Ghost OS]]
|OSDescription=A homemade operating system with a microkernel for the IA32 (x86) platform. The project is written in C++ and Assembly. Features: multiprocessor- & multitasking support, kernel API library, custom C library, ELF support, IPC (messages, signals, shared memory, pipes), VFS, window server & GUI with homemade toolkit, PS/2 keyboard & mouse driver, VESA video driver and more...
|Contact=Max Schlüssel (lokoxe@gmail.com)
|URL=https://ghostkernel.org/
|Status=Active
|Pic=[[Image:ghost-ui-preview.png|200px]]
|License=GPLv3
}}

{{OSProject
|OSName=Ironclad
|OSDescription=Ironclad is a mostly formally verified hard real-time kernel written in SPARK and Ada. It is made to be 100% free software, free in the sense that it respects the user's freedom. It supports several architectures and features an advanced security model with features like Mandatory Access Control (MAC). The screenshot features Gloire, a distribution of Ironclad.
|Contact=streaksu@ironclad-os.org
|License=GNU GPLv3
|URL=https://ironclad-os.org/
|Pic=[[File:GloireJWM.png|200px]]
|Status=Active
}}

{{OSProject
|OSName=KnightOS
|OSDescription=Open-source operating system for Texas Instruments calculators. Features preemptive multitasking, memory management, etc. Written in z80 assembly.
|Contact=Drew DeVault (sir at cmpwn.com)
|URL=http://knightos.org
|Status=Inactive (2021)
}}

{{OSProject
|OSName=Managarm
|OSDescription=Managarm is a 64-bit OS for x86-64. It’s written in C++ with a custom libc and a GNU like userland on top. Managarm’s main purpose is aiming for Linux compatibility while being completely asynchronous in terms of I/O. The OS is capable of running Weston and kmscon while effort is made into porting (a subset of) Xorg. Furthermore, Managarm supports many modern hardware devices, including USB 3 and has nearly full ACPI support.
|Contact=The official Managarm Discord server https://discord.gg/7WB6Ur3
|URL=http://www.managarm.org
|Status=Active
|Pic=[[File:Managarm-demo.png|200px]]
|License=Open source (MIT)
}}

{{OSProject
|OSName=MikeOS
|OSDescription=MikeOS is an operating system for x86 PCs, written in assembly language. It is a learning tool to show how simple 16-bit, real-mode OSes work, with well-commented code and extensive documentation. It has a BASIC interpreter with 46 instructions, supports over 60 syscalls, could manage a serial terminal connection and output the sound through PC speaker. There is also a file manager, text editor, image viewer and some games
|Contact=Mike Saunders, okachi [at] gmail [dot] com
|License=Open source
|URL=http://mikeos.sourceforge.net/
|Status=Discontinued
|FloppyAvailable=Fits on a floppy! :)
|Pic=[[File:MikeOS.png|200px]]
}}

{{OSProject
|OSName=Pedigree
|OSDescription=Monolithic OS with several backends supported - x86, x64, MIPS32, ARM and PowerPC. Kernel written in C++ with the obvious bits of ASM. Offers a reasonable amount of POSIX support and a tiling GUI and can run Apache, DOSBox, and various other common programs. Planned to also offer a native API alongside POSIX for Pedigree-specific applications.
|Contact=JamesM, bluecode, [http://ideasandcode.blogspot.com/ pcmattman], IRC freenode.net#pedigree
|URL=http://www.pedigree-project.org/
|Status=Discontinued
}}

{{OSProject
|OSName=RDOS
|OSDescription=32-bit segmented operating system for 386 or higher processors. Support for USB (UHCI, OHCI, EHCI, XHCI) and USB 2 hubs, IDE and AHCI for discs, RTL8139 and RTL8169 for networks, and HD audio. It boots with EFI using GPT and BIOS using BIOS disc formats and supports FAT12, FAT16, and FAT32 filesystems. Supports both older CPUs with PIC & PIT and newer with APIC which also includes support for multicore operation. It has a full libc in the OpenWatcom project. OpenWatcom supports building segmented kernel drivers, and 32-bit flat applications, libraries, and DLLs. It has a command-line originally ported from FreeDos, and a GUI the includes widgets for labels, buttons, list boxes, and images (supporting PNG, JPEG, and BMP formats). In the debugging area, it supports remote debugging over TCP/IP from Windows using OpenWatcoms debugger that also can trace into the kernel, a kernel debugger, and a crash debugger. It runs on several thousand commercial installations as a controller and as a payment terminal.
|Contact=Leif Ekblad (leif at rdos.net)
|URL=http://www.rdos.net/rdos/
|Status=Active
}}

{{OSProject
|OSName=SeaOS
|OSDescription=SeaOS is a hybrid kernel with loadable modules that supports ATA, AHCI, EXT2, ELF, and many other fancy acronyms.
It has basic networking support, initial VT-x support, and is self-hosting with a fairly complete unix-like userland. Designed for simplicity.
|Contact=Daniel Bittman (danielbittman1 at gmail.com)
|URL=http://dbittman.github.io/seaos
|Status=Inactive (2016)
}}

{{OSProject
|OSName=SerenityOS
|OSDescription=SerenityOS is a graphical Unix-like OS written in C++. It combines a 1990's style GUI with a modern CLI. Everything is from scratch, including a web browser with JavaScript and HTTPS support. A growing number of 3rd party packages are available as optional ports, including GCC, bash, vim, Python, SDL2, etc. You can watch videos of the system being developed on YouTube: https://www.youtube.com/c/AndreasKling.
|Contact=Andreas Kling, kling [at] serenityos [dot] org
|License=Open source (BSD 2-clause)
|URL=https://github.com/SerenityOS/serenity
|Pic=[[File:SerenityOS-8ea4375.png|200px]]
|Status=Active
}}

{{OSProject
|OSName=skiftOS
|OSDescription=skiftOS is a hobby operating system built for learning and fun targeting the x86 platform. It features a kernel named hjert, a graphical user interface with a compositing window manager, and familiar UNIX utilities.
|Contact=Nicolas Van Bossuyt, nicolas.van.bossuyt [at] skiftos [dot] org
|License=Open source (MIT)
|URL=https://github.com/skiftOS/skift
|Pic=[[File:skiftOS.png|200px]]
|Status=Active
}}

{{OSProject
|OSName=Sortix
|OSDescription=Sortix is a small self-hosting operating-system aiming to be a clean and modern POSIX implementation. It is a hobbyist operating system written from scratch with its own base system, including kernel and standard library, as well as ports of third party software. It has a straightforward installer and can be developed under itself. Releases come with the source code in /src, ready for tinkering.
|Contact=[[User:sortie|sortie]], https://sortix.org/
|License=Open source (ISC)
|Status=Active
|URL=https://sortix.org/
|Pic=[[File:Sortix.png|200px]]
}}

{{OSProject
|OSName=SymbOS
|OSDescription=SymbOS is a multitasking graphical OS for 8-bit computers using the Z80 CPU. It aims to bring a Windows 95-like experience to hardware from the 1980s, including binary software compatibility across a range of originally incompatible computers (Amstrad CPC/PCW/NC, MSX, Enterprise 128, and more). Both original hardware (printers, sound) and modern hobbyist expansions (FAT mass storage, networking) are supported through device drivers. A growing library of userspace applications have been written using Z80 assembly and two native development environments, the Quigs IDE and a self-hosting native C compiler.
|Contact=Jörn Mika, http://www.prodatron.net/
|License=Mixed open/closed source
|URL=http://symbos.org/
|Status=Active
|Pic=[[File:symbos.gif|200px]]
}}

{{OSProject
|OSName=[[wikipedia:TempleOS|TempleOS]]
|OSDescription=TempleOS is a 64 bit lightweight OS with multitasking and multicore support, which is ring-0-only and works in a single address space. It provides the interface for communicating with God: the user has to choose a random number from the constantly changing sequences and then it is converted to the text interpretation. Whole OS with its' software has been single-handedly created during 15 years by Terry A Davis - who also  developed a programming language called Holy C together with a special compiler for it. TempleOS supports the FAT32 and RedSea filesystems (the latter created by Terry) and also the file compression. It doesn't support the networking, partially for ideological reasons, but there are forks available with added functionality
|Contact=Terry A Davis, http://www.templeos.org
|License=Open source (Public Domain)
|URL=http://templeos.org/
|Status=Discontinued
|FloppyAvailable=Could fit on a floppy, but no-one has tried :P
|Pic=[[File:TempleOS.png|200px]]
}}

{{OSProject
|OSName=[[User:Klange/ToaruOS|ToaruOS]]
|OSDescription=64-bit modular kernel written in C. Supports pipes, shared memory, signals, POSIX-like VFS, threading, audio, IPv4. Heavy focus on advanced GUI, including a compositing window system.
|Contact=[[User:klange|klange]], #toaruos on irc.libera.chat
|URL=http://toaruos.org/
|Status=Active
|License=NCSA/University of Illinois License
|Pic=[[Image:Toaruos_screenshot.png|200px]]
}}

{{OSProject
|OSName=Visopsys
|OSDescription=Visopsys (VISual OPerating SYStem) is an alternative operating system for PC-compatible computers, written “from scratch”, and developed primarily by a single hobbyist programmer since 1997. Its primary "useful feature" is a reasonably functional partition management program - the ‘Disk Manager’, which can create, format, delete, resize, defragment, copy, and move partitions, and modify their attributes. It supports both DOS/MBR and UEFI partition tables.  It can also copy hard disks, and has a simple and friendly graphical interface.  A basic version can fit on a bootable floppy disk, or you can use the entire system from a ‘live’ CD/DVD
|Contact=https://visopsys.org/about/
|License=Open source (GNU GPLv2)
|URL=http://visopsys.org/
|Status=Active
|Pic=[[File:Visopsys.png|200px]]
}}

{{OSProject
|OSName=Vanadium OS
|OSDescription=Vanadium OS is a multi-user Unix-like OS. i386, amd64. Fully protected mode. Memory, file and hardware protection. root and unprivileged users. GUI without X, modular kernel, and has a live floppy image with a single user mode! My homepage: http://www.durlej.net/contact ; project pages: http://www.durlej.net/v/ , http://www.durlej.net/nameless , https://github.com/p-durlej/newsys . One of the great features is that you can write C code and instantly compile it to execute
|Contact=Piotr Durlej, http://www.durlej.net/contact
|License=Open source (BSD 2-clause)
|URL=https://github.com/p-durlej/newsys
|Status=Inactive (2018)
|Pic=[[File:VanadiumOS.png|200px]]
}}

[[Category: Resources]]
````

<a id="w003875"></a>

## W003875 — Swift Bare Bones

Bron-ID: W003875
Namespace: Main (0)
Revisie: 26313 | Datum: 2021-08-12T23:34:28Z
Bron: https://wiki.osdev.org/Swift_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Swift_Bare_Bones&oldid=26313

````text
{{BeginnersWarning}}
{{Stub}}
{{In Progress}}
{{Rating|2}}
{{Template:Kernel designs}}

<big><b>WAIT! Have you read [[Getting Started]], [[Beginner Mistakes]], and some of the related [[:Category:OS theory|OS theory]]?</b></big>

== Notes ==
* The easiest way to use Swift is to output LLVM IR and use Clang to target the architecture you want.

== Examples ==
* https://github.com/rzhikharevich/swift-bare-bones
* https://github.com/klange/taylor

[[Category: Bare bones tutorials]]
````

<a id="w003876"></a>

## W003876 — File:U365 0.6.png

Bron-ID: W003876
Namespace: File (6)
Revisie: 18907 | Datum: 2016-02-06T13:32:36Z
Bron: https://wiki.osdev.org/File:U365_0.6.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AU365_0.6.png&oldid=18907

````text
U365 0.6
````

<a id="w003877"></a>

## W003877 — Talk:Cooperative Multitasking

Bron-ID: W003877
Namespace: Talk (1)
Revisie: 30765 | Datum: 2026-06-16T15:01:06Z
Bron: https://wiki.osdev.org/Talk:Cooperative_Multitasking
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACooperative_Multitasking&oldid=30765

````text

````

<a id="w003878"></a>

## W003878 — DWARF

Bron-ID: W003878
Namespace: Main (0)
Revisie: 30292 | Datum: 2026-03-16T19:03:56Z
Bron: https://wiki.osdev.org/DWARF
Revisie-URL: https://wiki.osdev.org/index.php?title=DWARF&oldid=30292

````text
{{In_Progress}}
{{Rating|2}}{{File formats}}

When debugging, chances are you will need debugging information. For instance, you can use GCC utility '''addr2line''' to convert an address into an actual location in your code, but it is much more convenient if your OS can display a stack trace with as much debugging information as possible, e.g.

<pre>
[0x00106F39] process_command (utils/shell.c at line 270)
[0x001077F3] process_char (utils/shell.c at line 477)
[0x001079ED] shell (utils/shell.c at line 526)
[0x00100DD6] main (kernel/kernel.c at line 84)
</pre>

DWARF is a debugging data format designed along with ELF, and allows you to find information like shown above.

==Generating the debug symbols==

You will need to pass the '''-g''' to GCC and to ld to generate DWARF information. GCC also allows you to specify the exact DWARF version, e.g. '''gcc -gdwarf-4'''.

Once this is done, running '''objdump -h''' on your executable will show several .debug sections. Those are standard ELF sections that you access the same way as any ELF binary. Running '''objdump -g''' on your binary will generate a text dump of the debug symbols.

==Relocation==

The addresses provided by DWARF are the addresses provided by ELF. You should take into account any relocation if any.

==Mapping an address to a line number (.debug_line section)==

The '''.debug_line''' section allows to determine the source code file and line number for a given address in memory. The content is broken down by Compilation Units (CU) representing one main source code file.

Each CU has a header:

<syntaxhighlight lang="C">
typedef struct __attribute__((packed)) {
    uint32_t length;
    uint16_t version;
    uint32_t header_length;
    uint8_t min_instruction_length;
    uint8_t default_is_stmt;
    int8_t line_base;
    uint8_t line_range;
    uint8_t opcode_base;
    uint8_t std_opcode_lengths[12];
} DebugLineHeader;
</syntaxhighlight>

followed by a list of directories, a list of files (referencing what directory they are in), and a series of line number statements, e.g.

<pre>
  Offset:                      0x11f
  Length:                      103
  DWARF Version:               2
  Prologue Length:             52
  Minimum Instruction Length:  1
  Initial value of 'is_stmt':  1
  Line Base:                   -5
  Line Range:                  14
  Opcode Base:                 13

 Opcodes:
  Opcode 1 has 0 args
  Opcode 2 has 1 args
  Opcode 3 has 1 args
  Opcode 4 has 1 args
  Opcode 5 has 1 args
  Opcode 6 has 0 args
  Opcode 7 has 0 args
  Opcode 8 has 0 args
  Opcode 9 has 1 args
  Opcode 10 has 0 args
  Opcode 11 has 0 args
  Opcode 12 has 1 args

 The Directory Table (offset 0x13a):
  1	kernel
  2	./lib

 The File Name Table (offset 0x148):
  Entry	Dir	Time	Size	Name
  1	1	0	0	heap.c
  2	2	0	0	libc.h

 Line Number Statements:
  [0x0000015d]  Extended opcode 2: set Address to 0x100ad1
  [0x00000164]  Advance Line by 15 to 16
  [0x00000166]  Copy
  [0x00000167]  Special opcode 48: advance Address by 3 to 0x100ad4 and Line by 1 to 17
  [0x00000168]  Special opcode 50: advance Address by 3 to 0x100ad7 and Line by 3 to 20
  ...
</pre>

In order to decode the information from '''.debug_line''', you will need to implement a small state machine with a few registers. The most important registers are:

* Address: the address in memory
* File: the source code file
* Line: the line number in that file (starts at 1)

Each line number statement starts with a 1-byte opcode, optionally followed by an argument. Each opcode tells what state machine register(s) to change:

* Standard opcodes (1 to 12): see section 6.2.5.2 of the [http://dwarfstd.org/doc/DWARF4.pdf The DWARF Debugging Information Format] to see what those opcodes are doing and what parameters they take
* Extended opcodes (0): the next byte is an unsigned LEB128-encoded number that tells how many bytes after that are used by the extended opcode. The byte after that tells the extended opcode number, followed by any argument. See section 6.2.5.3 of the [http://dwarfstd.org/doc/DWARF4.pdf The DWARF Debugging Information Format]
* Special opcodes (13 or greater): these are one-character instructions that tell to change the Address and Line registers. If the Address can only increase, note that the Line may decrease. The way to compute this is a bit tricky (see section 6.2.5.1 of the [http://dwarfstd.org/doc/DWARF4.pdf The DWARF Debugging Information Format]), but relies on the Line Base, Line Range and Opcode base from the header and goes something line this:

<pre>
Address += ((Opcode - Opcode base) / Line range) * Min instruction length
Line += Line base + (Opcode - Opcode base) % Line range
</pre>

In the example above, Special opcode 48 (stored in the file as 48 + 13 = 0x3D) tells to increase the Address by 48 / 14 = 3 bytes (thus moving from 0x100ad1 to 0x100ad4) and the Line by -5 + 48 % 14 = 1 (thus moving from 16 to 17)

===Binary representation===

The above CU is store as follows in a 32-bit ELF binary:

<pre>
67 00 00 00   02 00 34 00   00 00 01 01   FB 0E 0D 00
01 01 01 01   00 00 00 01   00 00 01 6B   65 72 6E 65
6C 00 2E 2F   6C 69 62 00   00 68 65 61   70 2E 63 00
01 00 00 6C   69 62 63 2E   68 00 02 00   00 00 00 05
02 D1 0A 10   00 03 0F 01   3D3F...
</pre>

* 0x67000000: the size of the CU (after this)
* 0x0200: DWARF version
* 0x34000000: Header Length, which indicates how many bytes after this number does the first opcode start
* 0x01: Minimum Instruction Length
* 0x01: Default is_stmt value
* 0xFB: Line Base (signed int)
* 0x0E: Line Range
* 0x0D: Opcode Base
* 0x00010101 01000000 01000001: the number of arguments for the 12 standard opcodes
* 0x6B65726E 656C00: "kernel" (first directory)
* 0x2E2F6C69 6200: "./lib" (second directory)
* 0x00: end of the directories
* 0x68656170 2E6300: "heap.c" (first file)
* 0x010000: Dir=1 ("kernel"), Time=0, Size=0
* 0x6C696263 2E6800: "libc.h" (second file)
* 0x020000: Dir=2 ("./lib"), Time=0, Size=0
* 0x00: end of the files
* 0x00: Signals an extended opcode
* 0x05: number of bytes after this used by the extended opcode (unsigned LEB128 encoded)
* 0x02: Extended opcode 2 (DW_LNE_set_address)
* 0xD10A1000: opcode argument (address 0x00100ad1)
* 0x03: Standard opcode 3 (DW_LNS_advance_line)
* 0x0F: opcode argument, encoded as a signed LEB128 (15)
* 0x01: Standard opcode 1 (DW_LNS_copy)
* 0x3D: Special opcode 48 (0x3D - 13 = 48)
* 0x3F: Special opcode 50

==Mapping an address to a function (.debug_info section)==

The '''.debug_info''', allows -among others- to determine the address range occupied by various parts of the code. It is complemented by the '''.debug_abbrev''' and '''.debug_str''' sections.

the '''.debug_info''' section is a series of Compilation Units (CU). Each compilation unit starts with a header followed by Debugging Information Entries (DIEs):

<pre>
Contents of the .debug_info section:

  Compilation Unit @ offset 0x0:
   Length:        0x6b7 (32-bit)
   Version:       4
   Abbrev Offset: 0x0
   Pointer Size:  4
 <0>< b>: Abbrev Number: 1 (DW_TAG_compile_unit)
    <c>   DW_AT_producer    : (indirect string, offset: 0x107): GNU C99 5.3.0 -m32 -mtune=generic -march=pentiumpro -g -std=gnu99 -ffreestanding
    <10>   DW_AT_language    : 12	(ANSI C99)
    <11>   DW_AT_name        : (indirect string, offset: 0x1b8): kernel/main.c
    <15>   DW_AT_comp_dir    : (indirect string, offset: 0x158): /usr/opsys
    <19>   DW_AT_low_pc      : 0x1002dd
    <1d>   DW_AT_high_pc     : 0x7f4
    <21>   DW_AT_stmt_list   : 0x0
 <1><25>: Abbrev Number: 2 (DW_TAG_typedef)
    <26>   DW_AT_name        : (indirect string, offset: 0xa5): uint
    <2a>   DW_AT_decl_file   : 2
    <2b>   DW_AT_decl_line   : 4
    <2c>   DW_AT_type        : <0x30>
 <1><30>: Abbrev Number: 3 (DW_TAG_base_type)
    <31>   DW_AT_byte_size   : 4
    <32>   DW_AT_encoding    : 7	(unsigned)
    <33>   DW_AT_name        : (indirect string, offset: 0x23a): unsigned int
</pre>

So the first thing when reading the '''.debug_info''' section is to look at the compilation unit header which contains the following fields:

* unit_length: the size of the unit
* version: the DWARF version
* debug_abbrev_offset: the offset inside the '''.debug_abbrev''' table of the DIE schemas for that CU
* address_size: the pointer size

The header is followed by a series of DIEs. First first byte of each DIE indicates its type number, followed by its concatenated attributes. All the DIEs of the same type have the same schema within the same CU, but the number of types (and their schema) varies for each CU.

So you need to:

* Read the CU header. This will tell you the end of the CU, as well as the Abbrev Offset.
** The Abbrev offset is an offset inside the '''.debug_abbrev''' section
** Read from the '''.debug_abbrev''' section the number of DIE types, as well as the schema for each type (see the next section for further information)
*** The first character of each DIE tells its type. Based on the type you can derive what attribute does the DIE has, and how much space occupies each attribute
*** If the DIE type tag is DW_TAG_subprogram, then look for the attributes DW_AT_NAME (function name), DW_AT_low_pc (the address where the function starts) and DW_AT_high_pc (the amount of memory occupied by the function)
** Repeat until you reach the end of the CU
* Read the next CU header
* Repeat until you reach the end of the .debug_info section

==The .debug_abbrev section==

This section contains the schema for all the DIE types from the '''.debug_info''' section.

<pre>
Contents of the .debug_abbrev section:

  Number TAG (0x0)
   1      DW_TAG_compile_unit    [has children]
    DW_AT_producer     DW_FORM_strp
    DW_AT_language     DW_FORM_data1
    DW_AT_name         DW_FORM_strp
    DW_AT_comp_dir     DW_FORM_strp
    DW_AT_low_pc       DW_FORM_addr
    DW_AT_high_pc      DW_FORM_data4
    DW_AT_stmt_list    DW_FORM_sec_offset
    DW_AT value: 0     DW_FORM value: 0
   2      DW_TAG_typedef    [no children]
    DW_AT_name         DW_FORM_strp
    DW_AT_decl_file    DW_FORM_data1
    DW_AT_decl_line    DW_FORM_data1
    DW_AT_type         DW_FORM_ref4
    DW_AT value: 0     DW_FORM value: 0
   3      DW_TAG_base_type    [no children]
    DW_AT_byte_size    DW_FORM_data1
    DW_AT_encoding     DW_FORM_data1
    DW_AT_name         DW_FORM_strp
    DW_AT value: 0     DW_FORM value: 0
    ...
</pre>

The first byte is the type number followed by its tag. The next byte is whether the type has children or not. Then are stored all the attributes for this type as name / type two-bytes pairs followed by 0x0000. See Figure 21 from the [http://dwarfstd.org/doc/DWARF4.pdf The DWARF Debugging Information Format] for the list of attribute names and their hex codes and Figure 20 for attribute types. The attribute type tells how many bytes will it occupy in the '''.debug_info''' section.

If most attributes have a fixed size, a few attributes need to have some special handling:

* DW_FORM_string attributes are a small zero-terminated string (so of variable length)
* DW_FORM_strp attributes represent the offset inside the '''.debug_str''' section of the desired string
* DW_FORM_exprloc attributes are multibytes blocks (so of variable length) composed of an unsigned LEB128 number which tells the block length followed by the said block

The list of DIE types for a particular Compilation Unit is over when the next type number is 0.

===Binary representation===

The above is encoded as follows:

<pre>
01 11 01 25   0E 13 0B 03   0E 1B 0E 11   01 12 06 10
17 00 00 02   16 00 03 0E   3A 0B 3B 0B   49 13 00 00
03 24 00 0B   0B 3E 0B 03   0E 00 00 04   24 00 0B 0B
...
</pre>

* 0x01 0x11: Type number 1 / DW_TAG_compile_unit
* 0x01: has children
* 0x25 0x0E: DW_AT_producer / DW_FORM_strp
* 0x13 0x0B: DW_AT_language / DW_FORM_data1
* ...
* 0x00 0x00: end of Type number 1
* 0x02 0x16: Type number 2 / DW_TAG_typedef
* 0x00: no children
* ...
* 0x00 0x00: end of Type number n
* 0x00: end of the schema for that Compilation Unit

==LEB128==

DWARF is sometimes storing data using LEB128 (Little-Endian Base 128), a variable-length compression algorithm. You can find multiple implementations online, whose functions will typically look like this:

<syntaxhighlight lang="C">
uint32_t decodeULEB128(const uint8_t *p, uint32_t *n);
int32_t decodeSLEB128(const uint8_t *p, uint32_t *n);
</syntaxhighlight>

The above functions take a pointer to a data stream as an argument, return the decoded value (respectively unsigned and signed LEB128) and store in '''n''' the number of characters used to decompress the data.

==See Also==
=== Articles ===
* [[ELF]]

=== External Links ===
* [http://dwarfstd.org/doc/DWARF4.pdf The DWARF Debugging Information Format]

[[Category:Debugging]]
````

<a id="w003880"></a>

## W003880 — Wiki/PAE

Bron-ID: W003880
Namespace: Main (0)
Revisie: 18958 | Datum: 2016-02-27T16:20:45Z
Bron: https://wiki.osdev.org/Wiki/PAE
Revisie-URL: https://wiki.osdev.org/index.php?title=Wiki%2FPAE&oldid=18958
Doorverwijzing: Physical Address Extension

````text
#REDIRECT [[Physical_Address_Extension]]
````

<a id="w003881"></a>

## W003881 — Wiki/udi

Bron-ID: W003881
Namespace: Main (0)
Revisie: 18959 | Datum: 2016-02-27T20:37:39Z
Bron: https://wiki.osdev.org/Wiki/udi
Revisie-URL: https://wiki.osdev.org/index.php?title=Wiki%2Fudi&oldid=18959
Doorverwijzing: Uniform Driver Interface

````text
#REDIRECT [[Uniform_Driver_Interface]]
````

<a id="w003882"></a>

## W003882 — You know your OS is advanced when

Bron-ID: W003882
Namespace: Main (0)
Revisie: 18966 | Datum: 2016-03-03T19:48:07Z
Bron: https://wiki.osdev.org/You_know_your_OS_is_advanced_when
Revisie-URL: https://wiki.osdev.org/index.php?title=You_know_your_OS_is_advanced_when&oldid=18966
Doorverwijzing: You Know Your OS Is Advanced When

````text
#REDIRECT [[You Know Your OS Is Advanced When]]
````

<a id="w003883"></a>

## W003883 — User:Thefarmingbunny

Bron-ID: W003883
Namespace: User (2)
Revisie: 18968 | Datum: 2016-03-05T11:33:09Z
Bron: https://wiki.osdev.org/User:Thefarmingbunny
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AThefarmingbunny&oldid=18968

````text
I am TheFarmingBunny. I love Assembly, C and C++ and am working on an unnamed operating system on Xubuntu 15.04! I'm also a TF2 addict :P
````

<a id="w003884"></a>

## W003884 — Detecting Hardware

Bron-ID: W003884
Namespace: Main (0)
Revisie: 28187 | Datum: 2023-07-10T17:33:43Z
Bron: https://wiki.osdev.org/Detecting_Hardware
Revisie-URL: https://wiki.osdev.org/index.php?title=Detecting_Hardware&oldid=28187

````text
{{stub}}

One of the main roles of an operating system is to detect hardware that can be used.

==x86==

Typically, on newer [[X86|x86]] hardware there is [[ACPI]], which lets the operating system to detect hardware. If there is no ACPI, then you can't do more than assume that standard buses like [[PCI]] exist.

After parsing the [[ACPI]] structures, you should enumerate whatever is connected to the buses and controllers found in ACPI. Then you should enumerate whatever is connected to whatever found in the first enumeration. This can include [[PCI]], [["8042"_PS/2_Controller|PS/2 controllers]], or whatever.

For example, after scanning the [[ACPI]] structures, you will typically find some form of [[PCI]]. Then you find [[USB]] host controllers connected to PCI. Then you might find external [[ATA]] hard disks, [[SCSI]] flash drives, or printers, or mice, or whatever, which are connected to USB host controllers.

A rule of thumb when initialising devices is to start from those that are less likely to be emulated, for example you need to initialise [[USB]] host controllers before initialising the [["8042"_PS/2_Controller|PS/2 controller]], as the latter is more likely to be emulated through USB Legacy Support.

[[Category:Hardware Detection]]
````

<a id="w003885"></a>

## W003885 — Category:CPU Registers

Bron-ID: W003885
Namespace: Category (14)
Revisie: 19601 | Datum: 2016-07-25T12:34:40Z
Bron: https://wiki.osdev.org/Category:CPU_Registers
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ACPU_Registers&oldid=19601

````text
Registers are small amounts of memory located in the processor. They provide a fast way to temporarily store data. Here are pages describing registers of specific CPU architectures.

[[Category:X86]]
````

<a id="w003886"></a>

## W003886 — Detecting hardware

Bron-ID: W003886
Namespace: Main (0)
Revisie: 19007 | Datum: 2016-03-19T12:37:09Z
Bron: https://wiki.osdev.org/Detecting_hardware
Revisie-URL: https://wiki.osdev.org/index.php?title=Detecting_hardware&oldid=19007
Doorverwijzing: Detecting Hardware

````text
#REDIRECT [[Detecting Hardware]]
````

<a id="w003887"></a>

## W003887 — User:Kjam

Bron-ID: W003887
Namespace: User (2)
Revisie: 19014 | Datum: 2016-03-23T23:35:35Z
Bron: https://wiki.osdev.org/User:Kjam
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKjam&oldid=19014

````text
I'm working on a compiler for compiled statically type programming language, which , I hope, one day will become a heart of revolutionary OS.

I'm using ANTLR3 with custom code generation target for prototyping parser, LLVM for mid-end and back-end, and the heart of the compiler is a semantical analyzer written in C++.
````

<a id="w003889"></a>

## W003889 — Yes Another Unix Clone

Bron-ID: W003889
Namespace: Main (0)
Revisie: 19030 | Datum: 2016-03-30T12:00:32Z
Bron: https://wiki.osdev.org/Yes_Another_Unix_Clone
Revisie-URL: https://wiki.osdev.org/index.php?title=Yes_Another_Unix_Clone&oldid=19030
Doorverwijzing: User:Sortie/Yes Another Unix Clone

````text
#REDIRECT [[User:Sortie/Yes Another Unix Clone]]
````

<a id="w003890"></a>

## W003890 — User:Malloc19

Bron-ID: W003890
Namespace: User (2)
Revisie: 19032 | Datum: 2016-04-01T21:02:29Z
Bron: https://wiki.osdev.org/User:Malloc19
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMalloc19&oldid=19032

````text
I'm just getting started!
````

<a id="w003891"></a>

## W003891 — File:Exdos64.png

Bron-ID: W003891
Namespace: File (6)
Revisie: 19035 | Datum: 2016-04-02T11:30:49Z
Bron: https://wiki.osdev.org/File:Exdos64.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AExdos64.png&oldid=19035

````text

````

<a id="w003893"></a>

## W003893 — File:Win32 lpthandler exe 0000.jpg

Bron-ID: W003893
Namespace: File (6)
Revisie: 19058 | Datum: 2016-04-09T04:51:54Z
Bron: https://wiki.osdev.org/File:Win32_lpthandler_exe_0000.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AWin32_lpthandler_exe_0000.jpg&oldid=19058

````text
Simple example of parallel port manipulation and programming in Windows.
````

<a id="w003894"></a>

## W003894 — Interrupt Service Routine

Bron-ID: W003894
Namespace: Main (0)
Revisie: 19066 | Datum: 2016-04-11T06:54:15Z
Bron: https://wiki.osdev.org/Interrupt_Service_Routine
Revisie-URL: https://wiki.osdev.org/index.php?title=Interrupt_Service_Routine&oldid=19066
Doorverwijzing: Interrupt Service Routines

````text
#REDIRECT [[Interrupt Service Routines]]
````

<a id="w003895"></a>

## W003895 — QEMU and GDB in long mode

Bron-ID: W003895
Namespace: Main (0)
Revisie: 28041 | Datum: 2023-07-09T18:58:52Z
Bron: https://wiki.osdev.org/QEMU_and_GDB_in_long_mode
Revisie-URL: https://wiki.osdev.org/index.php?title=QEMU_and_GDB_in_long_mode&oldid=28041

````text
If you're running in [[Long Mode]] and using [[QEMU]] as your emulator you'll likely encounter the following error trying to use [[QEMU#GDB-stub|QEMU's gdb-stub]]:
<pre>Remote 'g' packet reply is too long: ...</pre>
This error happens when QEMU has switched to long mode. There are two workarounds for this: you can either disconnect and reconnect to force GDB to use the x86_64 architecture or patch GDB to switch the architecture on the fly.

==Workaround 1: Reconnecting==

When you start GDB, connect to QEMU with these commands:
<source lang="text">
$ gdb
(gdb) set arch i386:x86-64:intel
(gdb) target remote localhost:1234
(gdb) symbol-file YOUR_KERNEL_FILE
(gdb) break SOME_FUNCTION_IN_LONG_MODE
(gdb) continue
</source>

When the break point is hit and the "packet reply is too long" happens, run the following:
<source lang="text">
(gdb) disconnect
(gdb) set arch i386:x86-64
(gdb) target remote localhost:1234
</source>

Note that you must use different architecture names in those two "set arch" commands.

==Workaround 2: Patching GDB==

This is a slightly modified version of the patch posted in [[topic:28262|this thread]]. As a bonus, this patch produces a warning when it suspects that the mode switch happened.

[NOTE]: There is an updated version (for GDB 12.1) of this patch at https://github.com/mduft/tachyon3/blob/master/tools/patches/gdb-12.1-archswitch.patch - this is confirmed working with QEMU 7.2.0.

<source lang="diff">
--- gdb/remote.c  	2016-04-14 11:13:49.962628700 +0300
+++ gdb/remote.c	2016-04-14 11:15:38.257783400 +0300
@@ -7181,8 +7181,28 @@
   buf_len = strlen (rs->buf);
 
   /* Further sanity checks, with knowledge of the architecture.  */
+// HACKFIX for changing architectures for qemu. It's ugly. Don't use, unless you have to.
+  // Just a tiny modification of the patch of Matias Vara (http://forum.osdev.org/viewtopic.php?f=13&p=177644)
   if (buf_len > 2 * rsa->sizeof_g_packet)
-    error (_("Remote 'g' packet reply is too long: %s"), rs->buf);
+    {
+      warning (_("Assuming long-mode change. [Remote 'g' packet reply is too long: %s]"), rs->buf);
+      rsa->sizeof_g_packet = buf_len ;
+
+      for (i = 0; i < gdbarch_num_regs (gdbarch); i++)
+        {
+          if (rsa->regs[i].pnum == -1)
+            continue;
+
+          if (rsa->regs[i].offset >= rsa->sizeof_g_packet)
+            rsa->regs[i].in_g_packet = 0;
+          else
+            rsa->regs[i].in_g_packet = 1;
+        }
+
+      // HACKFIX: Make sure at least the lower half of EIP is set correctly, so the proper
+      // breakpoint is recognized (and triggered).
+      rsa->regs[8].offset = 16*8;
+    }
 
   /* Save the size of the packet sent to us by the target.  It is used
      as a heuristic when determining the max size of packets that the

</source>

The patched GDB emits the error as a warning but continues to function across the mode change. This patch has been tested with gdb 7.11 and 7.12.

Alternatively, you can patch QEMU so that you can use gdb unpatched. Be warned, this probably breaks support for debugging 32-bit code (also, compiling QEMU is hour-long task). In order to do this, replace each occurence of <code>if (TARGET_LONG_BITS == 64 && env->hflags & HF_CS64_MASK) {</code> in the file gdbstub.c with <code>if (TARGET_LONG_BITS == 64) {</code>. However, this method originates from [[topic:21836|a thread back in 2011]], and is untested since.

==Workaround 3: Connecting from GDB after long mode has been enabled==

This is an easy solution if you don't care to loose a few seconds of computation at the beginning of the emulation. The problem is that gdb is too dumb to understand that processor execution mode changed, so to workaround it you just connect after long mode has been enabled, and gdb never sees the switch. You can create this debug-with-qemu.sh script (or put this in a make target):

<source lang="text">
    #!/bin/env bash

    # use setsid so that ctrl+c in gdb doesn't kill qemu
    setsid qemu-system-x86_64 -s -boot d -cdrom kernel.iso &
    sleep 5
    gdb kernel.bin -x qemudbg
</source>

where your qemudbg should have something like this:

<source lang="text">
    target remote localhost:1234
</source>

and that's it (as long as you don't switch back and forth between modes).

==Other issues==
Some have experienced [[topic:26224|issues with breakpoints]] (using the <code>break</code> command) not firing. Use <code>hbreak</code> instead.

In case of using gentoo, this error might occur when emerging <code>sys-devel/gdb</code> [https://packages.gentoo.org/packages/sys-devel/gdb] without the <code>xml</code> USE flag.

==See also==
===Threads===
* [[topic:28262|Emulator for Mac OS X]] - Dealing with issues on macOS
* [[topic:30285|Debugging long mode kernel (gdb + qemu)?]] - workaround without patching gdb
* [[topic:21836|GDB and QEMU on Win64]] - patches for either gdb or qemu
* [[topic:26224|GDB not breaking at my breakpoints]] - dealing with 'skipped' breakpoints

[[Category:X86-64]]
[[Category:Debugging]]
[[Category:QEMU]]
[[Category:Troubleshooting]]
[[Category:FAQ]]
````

<a id="w003896"></a>

## W003896 — User:Alyssa

Bron-ID: W003896
Namespace: User (2)
Revisie: 19085 | Datum: 2016-04-22T13:36:06Z
Bron: https://wiki.osdev.org/User:Alyssa
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAlyssa&oldid=19085

````text
Hello! I'm alyssa on freenode.
````

<a id="w003897"></a>

## W003897 — User:Joeeagar

Bron-ID: W003897
Namespace: User (2)
Revisie: 19143 | Datum: 2016-05-07T21:00:36Z
Bron: https://wiki.osdev.org/User:Joeeagar
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJoeeagar&oldid=19143

````text
I'm trying to measure how various common OS conventions (e.g. paging, system calls, etc) affect performance.  It isn't much fun yet (paging! I want paging!).

[[User:Joeeagar/CoherentPosix]]
[[User:Joeeagar/ElfLoading]]
[[User:Joeeagar/TerminalEmulation]]

Software architecture doesn't have a [https://en.wikipedia.org/wiki/Corporatism corporate identity].

== Terminal Emulation Links (evil!) ==
* [http://invisible-island.net/xterm/ctlseqs/ctlseqs.html Xterm Command Codes]
* [http://catb.org/terminfo/ Terminfo Stuff]
* [http://pubs.opengroup.org/onlinepubs/7908799/xcurses/terminfo.html List of terminfo commands]
* [http://vt100.net/docs/vt102-ug/appendixc.html VT-102 command reference]
* [http://bluesock.org/~willg/dev/ansi.html#sequences ANSI command codes]
* [http://www.delorie.com/gnu/docs/termcap/termcap_toc.html#SEC_Contents Termcap documentation]
* [http://tldp.org/HOWTO/Text-Terminal-HOWTO.html#toc21.1 Text terminal HOWTO]
* [http://pubs.opengroup.org/onlinepubs/7908799/xcurses/terminfo.html xcurses terminfo]
````

<a id="w003898"></a>

## W003898 — User:Joeeagar/CoherentUnix

Bron-ID: W003898
Namespace: User (2)
Revisie: 19095 | Datum: 2016-04-25T05:48:13Z
Bron: https://wiki.osdev.org/User:Joeeagar/CoherentUnix
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJoeeagar%2FCoherentUnix&oldid=19095

````text
Unix suffers from one could call "standardized incoherence."  Here are some ideas on how it could be improved.


== Filesystem Hierarchy ==
The standard Unix file hierarchy is both primitive and (on Linux at least) extremely bug-prone.  Alternatives are:

* Application bundles (OSX), where applications contain mini-chroot-ish directory trees (but can pull from shared system libraries).
* Get rid of /usr/local entirely (which is kindof what Windows does if you think about it).

Related issues are:
# ELF does not support manifest files for dynamic linking.  
# Too much of ELF dynamic linking relies on hard-coded paths.

== ELF ==
ELF is a terrible standard.  The problem with ELF is that it is so complicated and so flexible that there are multiple ways to accomplish a given task, and informal standards have arisen on which subset a given shared library system uses (e.g. PIC vs DSO shared libraries, absolute versus PIC vs DSO executables, etc).

Part of the problem is how ELF pushes dynamic linking into the standard C library (what's ironic is that this is an optional feature of ELF that has become mandatory).

== The VT-100 ==

The situation vis-a-vis emulation protocols for controlling Unix terminals is completely absurd.  The VT-100 is something of a de facto  standard, as is xterm. What Unix needs is a single standard.  It can be anything, it just has to exist.

One possibility is to make a database of the control codes supported by xterm, rxvt, linuxconsole, etc, and then filter the list either through a committee or some sort of community voting scheme.

This is not ideal, however, since there a lot of normative questions as to how a terminal protocol '''should''' work.  Someone did a joke presentation a while back on a new type of terminal he'd invented, with all sorts of neat, HTML-like features.  Is this a good idea? Or not? Who knows?  Everyone just takes the current mess for granted.

== Security ==

Yeek!  Run away! Security!  The security scolds are always right, and we all still hate them.  Yet, our code must be secure.  It's a conundrum.  We'll probably all end up with our applications requesting permission from the user for specific system calls, but mercifully not yet.

== Unix Utilities ==

Someday the core utilities will have to be redesigned.  Someday. . .
````

<a id="w003899"></a>

## W003899 — User:Joeeagar/CoherentPosix

Bron-ID: W003899
Namespace: User (2)
Revisie: 19093 | Datum: 2016-04-25T05:45:37Z
Bron: https://wiki.osdev.org/User:Joeeagar/CoherentPosix
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJoeeagar%2FCoherentPosix&oldid=19093
Doorverwijzing: User:Joeeagar/CoherentUnix

````text
#REDIRECT [[User:Joeeagar/CoherentUnix]]
````

<a id="w003900"></a>

## W003900 — User:Joeeagar/ElfLoading

Bron-ID: W003900
Namespace: User (2)
Revisie: 19142 | Datum: 2016-05-07T21:00:03Z
Bron: https://wiki.osdev.org/User:Joeeagar/ElfLoading
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJoeeagar%2FElfLoading&oldid=19142

````text
The Executable Link Format, or ELF, is the format used by most Unixes today.  It's a very versatile format (too much so, but more on that later).

== Basic Information ==

=== Types of ELF Executables ===

There are many ways to link ELF executables:

# Absolute binaries.  These are hard-linked with an absolute base address.  This is the most common way to do Unix binaries, but only works on systems that support virtual memory.

# PIC, or Position Independent Code.  These binaries use relative addressing to avoid relying on a specific base memory address.  Library calls are implemented as jumps into what's called the Procedure Lookup Table, which stores the addresses of external library functions (themselves usually implemented as PIC code).

# DSO, or Dynamic Shared Object. This is basically ELF's version of DLLs.  Rather than using relative addressing to jump to library calls stored in a PLT, library calls are patched on startup.  This avoids the indirect jump, but at the cost of slower (in some cases much slower) start-up time.

=== ELF Binary Interpreters ===

Like I said in the beginning, ELF is a very versatile format.  In fact it's so versatile that there are many ways to do the same thing, and informal standards as to which subset a system uses have sprung up.  Modern Unixes use something called ELF binary interpreters, typically (always?) tied to the C library.  ELF interpreters are special programs that handle dynamic linking of executables (e.g. /lib/ld.linux.so).  The (absolute) path to the interpreter is hardwired into executables at build time.

== Basic Program Loading ==

For now, let's assume our programs are statically linked and don't use an ELF interpreter.  For that we need to load the ELF header and the program headers (note that datatype sizes differ between 32- and 64-bit systems):

<source lang="C">
typedef struct ElfHeader{
  unsigned char e_ident[EI_NIDENT]; //should start with [0x7f 'E' 'L' 'F']
  uint16_t e_type;
  uint16_t e_machine;
  uint32_t e_version;
  uint32_t e_entry;
  uint32_t e_phoff; //start of program headers in file
  uint32_t e_shoff;
  uint32_t e_flags;
  uint16_t e_ehsize;
  uint16_t e_phentsize; //size of each program header
  uint16_t e_phnum; //number of program headers
  uint16_t e_shentsize;
  uint16_t e_shnum;
  uint16_t e_shstrndx;
} ElfHeader;

typedef struct ElfProgramHeader {
  uint32_t p_type;
  uint32_t p_offset; //offset of data in elf image
  uint32_t p_vaddr; //virtual load address
  uint32_t p_paddr; //physical load address, not used
  uint32_t p_filesz; //size of data in elf image
  uint32_t p_memsz; //size of data in memory; any excess over disk size is zero'd
  uint32_t p_flags;
  uint32_t p_align; //alignment
} ElfProgramHeader;

</source>


First read the main header.  Check that e_ident starts with [0x7f, 'E', 'L','F'], or [0x7f, 0x45, 0x4c, 0x46].  Then read the program headers, which are stored in a flat array starting at e_phoff.  Each header has the same size, e_phentsize.  There are e_phnum headers.

The first thing we need to do is calculate the base address.  This is rather annoying, since ELF doesn't store it directly.  Rather, the base address is the smallest virtual address stored in the p_vaddr field in each program header.  Note that supposedly (at least according the documentation I read) p_vaddr must be aligned to be a multiple of p_align before we use it, though this doesn't seem to happen in practice (at least not with binaries generated via GNU binutils).

Here's an example of loading the main header and getting the base address (note that normally one would load the program headers into an array or linked list, which I've omitted to keep things simple):

<source lang="C">

#define MIN2(a, b) ((a) < (b) ? (a) : (b))

int ELF_loadBinary(unsigned char *elfimage) {
    ElfProgramHeader header;
    ProgramHeader ph;
    int i;
    uintptr_t base = UINTPTR_MAX;

    memcpy(&header, elfimage, sizeof(header));
    
    /*remember that strcmp returns zero when two strings are equal*/
    if (strcmp(header.e_ident, "\X7fELF")) {
       printf("error!\n");
       return -1;
    }
    
    /*note that we copy each program header into ph
      to avoid any alignment errors
    */
    
    for (i=0; i<header.e_phnum; i++) {
        memcpy(elfimage + header.e_phoff + header.e_phentsize*i, &ph, sizeof(ph));
        
        base = MIN2(base, ph.p_vaddr);
    }

    return 0;
}

</source>

Now that we have the base address, we need to figure out how much memory to allocate for the program image.  To do this we find the maximum of subtracting each program header's p_vaddr from the base address, then adding e_memsz:

<source lang="C">
    uintptr_t size=0;

    for (i=0; i<header.e_phnum; i++) {
        uintptr_t segment_end;

        memcpy(elfimage + header.e_phoff + header.e_phentsize*i, &ph, sizeof(ph));
        
        segment_end = ph.p_vaddr - base + ph.p_memsz;
        size = MAX2(size, segment_end);
    }
</source>

Next we allocate the program image, and then load the program segments.  There are different types of segments, so we'll need an enumeration of what those types are:

<source lang="C">
enum {
  PT_NULL,
  PT_LOAD,
  PT_DYNAMIC,
  PT_INTERP,
  PT_NOTE,
  PT_SHLIB,
  PT_PHDR,
  PT_LOPROC=0x70000000, //reserved
  PT_HIPROC=0x7FFFFFFF  //reserved
};
</source>

If you compiled an absolute binary (e.g. you didn't pass -fPIC to the compiler),
you'll have to allocate the binary at the calculated base address.  Otherwise you can
put it whereever you want.

After allocating the program image, we need to load the program headers.  In this simple example, our headers should have each have a type (p_type) of PT_LOAD or PT_PHDR.  For each header, calculate where it'll go inside the allocated image by either subtracting base address from p_vaddr (if using PIC code) and then adding the address of the image, or by using p_vaddr directly if you went with absolute binaries.

Next, zero the segment data, using the value in p_memsz.  Technically you only have to zero the difference between p_memsz and p_filesz, but zeroing the whole thing is fine too.  After zeroing, copy the segment data from the file image to the allocated image.  It's located at p_offset, and has size p_filesz.

Here's some code:

<source lang="C">
    unsigned char *finalimage = kmalloc(size); //for PIC
    unsigned char *finalimage = (unsigned char*) base; //for absolute

    for (i=0; i<header.e_phnum; i++) {
        uintptr_t addr;

        memcpy(elfimage + header.e_phoff + header.e_phentsize*i, &ph, sizeof(ph));

        addr = (ph.p_vaddr - base) + (uintptr_t)finalimage;

        memset(finalimage+addr, 0, ph.e_memsz);
        memcpy(finalimage+addr, elfimage+ph.e_offset, elfimage.e_filesz);
    }
</source>

Now we're ready to start the binary!  All we need to do is set up a stack, and jump to the entry point (which is stored in the e_entry field in the main header).

Well, not quite.  You might think the entry point would be the address of the main function, but this isn't the case.  It's a special function (usually called _start) that's provided by the libc.  This function needs a bunch of parameters to start the program with (e.g. the stdin/stdout/stderr file descriptors, argv, the program's environment variables, etc).  You'll need to read up how to pass this data to it, either through the stack or a special system call.  For more information, see the following links:

[https://www.gnu.org/software/libc/manual/html_node/Auxiliary-Vector.html]
[http://pubs.opengroup.org/onlinepubs/9699919799/functions/environ.html]
[http://pubs.opengroup.org/onlinepubs/9699919799/functions/posix_spawn.html]

== See Also ==
=== Articles ===
* [[ELF]]

=== External Links ===
* [http://docs.oracle.com/cd/E23824_01/html/819-0690/chapter6-46512.html#scrolltoc Elf reference from Oracle]
````

<a id="w003901"></a>

## W003901 — User:Joeeagar/TerminalEmulation

Bron-ID: W003901
Namespace: User (2)
Revisie: 19198 | Datum: 2016-05-10T11:58:12Z
Bron: https://wiki.osdev.org/User:Joeeagar/TerminalEmulation
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJoeeagar%2FTerminalEmulation&oldid=19198

````text
Unix text terminals do a number of things: move the cursor around, format text, change colors, clear the screen, etc.  There's a special command language for controlling all of this, originally based on the VT100 series of physical terminals (but over the years a bunch of stuff was added, and now it's a huge mess).

This tutorial covers emulating a (subset of) xterm's command language.  Xterm is something of a de facto standard, so it seemed like a good place to start.

== How terminals work ==

Terminals control how Unix users interact with applications.  A Unix terminal will handle keyboard input, render formatted/colored text, and send signals to host applications when the user presses special control codes, e.g. CTRL-C to abort an application.

Terminals can be implemented in the kernel (linuxconsole) or outside of it (xterm/rxvt and derivatives).  Terminals provide stdin, and are fed stdout.

== Command Reference ==
=== Basic Info on Commands ===

Commands typically start with the ESC key (ascii 27).  They can include a number of arguments,
which are drawn with grey color, and take the form <span style="color: grey">name = $type=default</span>.  For example, <span style="color: grey">ch=$c=1</span> has type c and defaults to '1' (in this case the character '1').

Note that if an argument has a default value, it can be omitted.  Also, control sequences don't have spaces in them.

==== Types ====
* $c        : any printable characters (be careful of numerals)
* $[...]    : any character in set
* $d        : ascii-encoded base-10 integer
* $i        : raw binary integer (0-255)

==== Basic Commands ====
{| class="wikitable"
|+Terminal Control Codes
|-
! scope="col"|Terminfo Code
! scope="col"|Escape Sequence
! scope="col"|Description
! scope="col"|Notes
|-
| cup
| <tt>'''ESC [ <span style="color: grey">y=$d</span> ; <span style="color: grey">x=$d</span> H'''</tt>
| Set cursor position
| origin is at (1, 1), not (0, 0).
|-
| cub
| <tt>'''ESC [ <span style="color: grey">i=$d=1</span> A'''</tt>
| Move cursor up '''''<span style="color: grey">i</span>''''' steps
|
|-
| cud
| <tt>'''ESC [ <span style="color: grey">i=$d=1</span> B'''</tt>
| Move cursor down '''''<span style="color: grey">i</span>''''' steps
|
|-
| cuf
| <tt>'''ESC [ <span style="color: grey">i=$d=1</span> C'''</tt>
| Move cursor right '''''<span style="color: grey">i</span>''''' steps
|
|-
| cub
| <tt>'''ESC [ <span style="color: grey">i=$d=1</span> D'''</tt>
| Move cursor left '''''<span style="color: grey">i</span>''''' steps
|
|-
| clear
| <tt>'''ESC [ 2 J'''</tt>
| Erase entire screen
|
|-
| home
| <tt>'''ESC [ H'''</tt>
| Move cursor to home position.
|
|-
| dch
| <tt>'''ESC [ <span style="color: grey">i=$d=1</span> P'''</tt>
| Delete '''''<span style="color: grey">i</span>''''' preceding characters
|
|-
| decsc
| <tt>'''ESC 7'''</tt>
| Save cursor
|
|-
| decrc
| <tt>'''ESC 8'''</tt>
| Restore cursor
|
|-
| setaf
| <tt>'''ESC [ 3 <span style="color: grey">color=$d=1</span> m'''</tt>
| Set foreground <span style="color: grey">color</span> to a valid [[#SGR_Colors|SGR color]].
| This forms an SGR code.
|-
| smcup
| <tt>'''ESC ? 1 ; 0 ; 4 ; 7 h'''</tt>
| Use alternative screen buffer.
| Full-screen text apps use this to avoid messing up the scroll buffer.
|-
| rmcup
| <tt>'''ESC ? 1 ; 0 ; 4 ; 9 l'''</tt>
| Switch back to normal screen buffer.
| 
|-
| ???
| <tt>'''ESC ? 1 ; 0 ; 4 ; 7 l'''</tt>
| Switch back to normal screen buffer, then clear the screen.
| 
|-
| ???
| <tt>'''ESC ? 1 ; 0 ; 4 ; 9 h'''</tt>
| Save cursor as in decsc, switch to alternate screen buffer, clearing it first.
| 
|}

==== Mouse Tracking (xterm) ====
Source: [[http://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking]]

Xterm supports a number of mouse modes.  For this tutorial we'll focus on the X11 xterm's "normal", "cell" and "any" modes.  "Normal" mode only button press/release events, while "any" sends both button press/release and motion events.  "Cell" mode only sends events if the mouse has moved to another character cell.

{| class="wikitable"
|+ Mouse Mode Set
!Code
!Description
!Notes
|-
|<tt>'''ESC [ 1 ; 0 ; 0 ; 0 h'''</tt>
| Normal mouse mode
| Does not send motion events.
|-
|<tt>'''ESC [ 1 ; 0 ; 0 ; 1 h'''</tt>
| Cell mouse mode
| Does not send motion events.  Only sends events when mouse cursor enters a different text cell.
|-
|<tt>'''ESC [ 1 ; 0 ; 0 ; 2 h'''</tt>
| Any mouse mode
| Sends mouse motion events as well as button press/release.
|-
|<tt>'''ESC [ 1 ; 0 ; 0 ; 2 h'''</tt>
| Send mouse focus events.
|
|}

{| class="wikitable"
|+ Mouse event codes
!Code
!Description
!Notes
|-
|<tt>'''ESC M <span style="color: grey">d=$i x=$i y=$i</span>  '''</tt>
| <span style="color: grey">d</span> is an information bitmask.  <span style="color: grey">x</span> and <span style="color: grey">y</span> are coordinates
|
|-
|<tt>'''ESC [ I'''</tt>
| Got mouse focus
|
|-
|<tt>'''ESC [ O'''</tt>
| Lost mouse focus
| <small>The letter O, not zero</small>
|}


{| class="wikitable"
|+ Information Bitmask
!0
!1
!2
!3
!4
!5
!6
!7
|-
|Button 1 pressed (or 3 if bit 6 is set)
|Button 2 pressed (or 4 if bit 6 is set).  If previous bit is set, interpret this as a button release event <small>(apparently with no info on which button was released?)</small>.
|Shift modifier key
|Meta/Alt modifier key
|Control modifier key
|}


==== SGR Codes ====
SGR codes are used for formatting text.  An SGR code has the form:

<tt>'''ESC [ <span style="color: grey">mode=$d</span> m'''</tt>

<span style="color: grey">mode</span> is a special code specifying either text attributes (e.g. bold) or colors.

{| class="wikitable"
|+ SGR Codes
|-
! Code
! Description
! Notes
|-
| 0
| Reset to default
|
|-
| 1
| Light color mode
|
|-
| 2
| Faint/dark color mode
| Not widely supported (according to wikipedia)
|-
| 3
| Italic: on
|
|-
| 4
| Underline single
|
|-
| 5
| Slow Blink
| Not widely supported (according to wikipedia)
|-
| 6
| Fast Blink
| Not widely supported (according to wikipedia)
|-
| 7
| Swap foreground and background colors
|
|-
| 8
| Conceal.  Temporarily sets foreground color to background color.
| Not widely supported (according to wikipedia)
|-
| 9
| Striketru
|
|-
| 10
| Set default font
|
|-
| 11-19
| Choose font
|
|-
| 20
| Fraktur
| Does anything support this?  Seems like an odd option to have.
|-
| 22
| Normal color, bold off
|
|-
| 23
| Turn off italic and Fraktur
|
|-
| 24
| Turn off underline
|
|-
| 25
| Turn off blink
|
|-
| 26
| Reserved
|
|-
| 27
| Unswap foreground and background colors
|
|-
| 28
| Turn off conceal
|
|-
| 29
| Turn off strikethru
|
|-
| 30-37
| Foreground color
|
|-
| 38
| Terminal-specific custom foreground code
|
|-
| 39
| Default foreground color
|
|-
| 40-47
| Background color
|
|-
| 48
| Terminal-specific custom background code
|
|-
| 49
| Default background color
|
|-
| 50
| Reserved
|
|-
| 51
| Framed
|
|-
| 52
| Encircled
|
|-
| 53
| Overlined
|
|-
| 54
| Turn off frame/encircled
|
|-
| 55
| Turn off overline
|
|}

===== SGR Colors =====
{| class="wikitable"
|+ SGR Colors
! Number
! Default Mode
! Light Mode
! Name
|-
| 0
| style="background-color: black;"|
| style="background-color: grey;"|
| black
|-
| 1
| style="background-color: rgb(150,0,0);"|
| style="background-color: red;"|
| red
|-
| 2
| style="background-color: rgb(0,150,0);"|
| style="background-color: green;"|
| green
|-
| 3
| style="background-color: rgb(150,150,0);"|
| style="background-color: yellow;"|
| yellow
|-
| 4
| style="background-color: rgb(0,0,150);"|
| style="background-color: blue;"|
| blue
|-
| 5
| style="background-color: rgb(150,0,150);"|
| style="background-color: magenta;"|
| magenta
|-
| 6
| style="background-color: rgb(0,150,150);"|
| style="background-color: cyan;"|
| cyan
|-
| 7
| style="background-color: rgb(200, 200, 200);"|
| style="background-color: white;"|
| white
|}

==== The stty command ====

Terminal modes can be controlled with the stty utility (as well as a rather annoying POSIX API).  You can invoke stty from C code with the system command.  For example, to turn off echo and line mode, you could do this:

<code>system("stty raw -echo");</code>

To turn echo and line mode back on:

<code>system("stty -raw echo");</code>

== See Also ==

=== Articles ===

* [[Kernel_Stdio_Theory|Kernel Stdio Theory]]
* [[Text_UI|Text UI]]

=== External Links ===
* [http://invisible-island.net/xterm/ctlseqs/ctlseqs.html Xterm Command Codes]
* [http://catb.org/terminfo/ Terminfo Stuff]
* [http://pubs.opengroup.org/onlinepubs/7908799/xcurses/terminfo.html List of terminfo commands]
* [http://vt100.net/docs/vt102-ug/appendixc.html VT-102 command reference]
* [http://bluesock.org/~willg/dev/ansi.html#sequences ANSI command codes]
* [http://www.delorie.com/gnu/docs/termcap/termcap_toc.html#SEC_Contents Termcap documentation]
* [http://tldp.org/HOWTO/Text-Terminal-HOWTO.html#toc21.1 Text terminal HOWTO]
* [http://pubs.opengroup.org/onlinepubs/7908799/xcurses/terminfo.html xcurses terminfo]
````

<a id="w003902"></a>

## W003902 — SSL/TLS

Bron-ID: W003902
Namespace: Main (0)
Revisie: 30455 | Datum: 2026-04-06T00:09:49Z
Bron: https://wiki.osdev.org/SSL/TLS
Revisie-URL: https://wiki.osdev.org/index.php?title=SSL%2FTLS&oldid=30455

````text
{{In_Progress}}

SSL/TLS is a protocol used to ensure a secure connection in various standard networking protocols (HTTP, FTP, etc.). Even though people talk about SSL, this protocol has been since mostly replaced with TLS (versions 1.0, to 1.3). SSL, and TLS 1.0 through 1.1 should not be used any more as it is not considered secure.

In order to setup an HTTPS connection, SSL/TLS is used between TCP and HTTP. In other word, the HTTP command sent by the Web browser and the HTML returned by the server are encrypted using SSL/TLS.

'''WARNING''': implementing your own TLS layer is no guarantee of security. It is indeed recommended to never even write your own implementation of known, secure cryptographic algorithms as multiple attacks have been known to exploit some faults in the implementation. Writing your own TLS layer is however useful if you want to understand how SSL/TLS works and/or if you want to access websites which are only available through HTTPS and are unable to integrate an existing library.

There are a few tools that can assist you when developing your own TLS layer. First of all, [http://www.wireshark.org Wireshark] is a free tool that captures network traffic and explains in details how the different packets are composed, down to the signification of each byte (save the encrypted parts). Also, Python can be an invaluable tool to prototype and verify your cryptographic algorithms (you might want to write a prototype of a TLS connection in Python first). Python indeed natively supports operations over very large integers like 1024-bit integers, and it has several cryptography libraries such as [https://www.dlitz.net/software/pycrypto/ PyCrypto] or [https://pypi.python.org/pypi/scapy-ssl_tls/1.2.2 Scapy SSL] that allows you to forge SSL packets. These two tools can greatly help testing how TLS works.

==Cryptography recap==

Here are the main types of cryptographic algorithms:

{| {{wikitable}}
|-
! 
! Public/private key
! Secret key
! No key
|-
! Encryption
| Asymmetric encryption
| Symmetric encryption
|
|-
! Verification
| Signing
| Message Authentication Cipher
| Cryptographic hash
|}

* Asymmetric encryption (e.g. RSA): one party generates a private/public key pair and makes the public key readily available. Anybody can encrypt data using that public key, but only the owner of the private key can decrypt it
* Symmetric encryption (e.g. AES): both parties need to use a shared secret key to encrypt and decrypt data
* Signing (e.g. RSA): one party generates a private/public key pair and makes the public key readily available. Only the owner of the private key can sign data, but anybody with the public key can verify that the signature matches the data
* Message Authentication Cipher aka MAC (e.g. HMAC): generates a signature using a secret key
* Cryptographic hash (e.g. SHA1, SHA256): generates a signature of some data, but it is very hard to find another data that would generate the same signature

==Cipher Suites==

A SSL/TLS connection is actually using a whole set of cryptographic algorithms called a cipher suite. On top of that, SSL/TLS does not support one but multiple cipher suites. An SSL/TLS connection might use a completely different cipher suite depending on what the client and server support. Fully supporting TLS would actually require to implement a whole series of cipher suites. Fortunately, implementing only a few popular cipher suites is enough for most cases.

You can get an exhaustive list of the cipher suites available [http://www.thesprawl.org/research/tls-and-ssl-cipher-suites/ here], and use SSL Labs' [https://www.ssllabs.com/ssltest/ SSL test] to check what cipher suites are supported by various Web sites.

A TLS cipher suite is denoted as TLS_[key exchange protocol]_WITH_[encryption]_[authentication]. Because TLS is using symmetric encryption, the two parties first need to determine a common secret key over a non-secure connection. This is where the key exchange protocol is used. Then the data is encrypted. Last but not least, it is authenticated using a MAC, in order to make sure that data was not tampered with.

We will study how things work with TLS version 1.2 when the TLS_DHE_RSA_WITH_AES_128_CBC_SHA cipher suite is used. This cipher suite indicates the algorithms used for the key exchange (DHE, using RSA for verification), for the actual encryption/decryption (AES 128-bit in CBC mode) and verification (HMAC+SHA1). This cipher suite thus requires to implement the following:

* The Diffie-Hellman Ephemeral (DHE) key exchange protocol. This protocols relies on [https://en.wikipedia.org/wiki/Modular_exponentiation modular exponentiation] over very large numbers, although it is possible to get past it if security is not your primary goal
* Encryption and decryption using [https://en.wikipedia.org/wiki/Advanced_Encryption_Standard AES] 128-bit in [https://en.wikipedia.org/wiki/Block_cipher_mode_of_operation#Cipher_Block_Chaining_.28CBC.29 CBC mode]
* The [https://en.wikipedia.org/wiki/SHA-1 SHA1] and [https://en.wikipedia.org/wiki/SHA-2 SHA256] cryptographic hashing algorithm
* [https://en.wikipedia.org/wiki/Hash-based_message_authentication_code HMAC], a Message Authentication Code (MAC). A MAC is similar to a cryptographic hash function except that it requires a secret key
* Optional: if you want to verify the server certificate, you will need to implement the RSA algorithm, which also relies on modular exponentiation as well as SHA1/SHA256/SHA384 (depending on the certificate chain)

Note that you can easily find on the Internet source code for AES, SHA1, SHA256 and HMAC.

This cipher suite is not the strongest available, but is still relatively popular and shows the key mechanisms of a secure TLS interaction. Another cipher suite useful to implement is TLS_RSA_WITH_AES_128_CBC_SHA. The only difference is that the key exchange is using RSA instead of Diffie-Hellman. People interested in implementing a stronger suite can look at TLS_ECDHE_RSA_WITH_AES_128_GCM which requires to implement the Elliptic Curve version of Diffie-Hellman as well as the Galois Counter Mode (GCM) instead of the easier-to-implement CBC mode.

==TLS Packets==

Any communication in TLS starts with a 5-byte TLS Record header:

<source lang="C">
typedef struct __attribute__((packed)) {
	uint8_t content_type;
	uint16_t version;
	uint16_t length;
} TLSRecordHeader;
</source>

This header may be followed by another TLS header, such as a TLS Handshake header or a TLS Application Data header.

=== Record Types ===

{| class="wikitable"
|-
! Value (Hex)
! Description
|-
| 0x14
| Change Cipher Spec
|-
| 0x15
| Alert
|-
| 0x16
| Handshake
|-
| 0x17
| Application Data
|}


=== Handshake Records===

<source lang="C">
typedef struct __attribute__((packed)) {
	uint8_t handshake_type;
	uint8_t[3] length;
} HandshakeRecordHeader;
</source>

{| class="wikitable"
|-
! Value (Hex)
! Description
|-
| 0x00
| Hello Request
|-
| 0x01
| Client Hello
|-
| 0x02
| Server Hello
|-
| 0x0B
| Certificate
|-
| 0x0C
| Server Key Exchange
|-
| 0x0D
| Certificate Request
|-
| 0x0E
| Server Hello Done
|-
| 0x0F
| Certificate Verify
|-
| 0x10
| Client Key Exchange
|-
| 0x14
| Finished
|}

==TLS Handshake==

Like for a TCP connection, a TLS connection starts with a handshake between the client and the server to establish the cipher suite used. See [[TLS Handshake]] for more details.

==TLS Encryption==

Once the handshake has completed, all data sent both way is encrypted using the algorithms and secret keys agreed upon during the TLS Handshake. See [[TLS Encryption]] for more information.

==Math operations on large integers==

Most cipher suites require to perform operations on large integers (e.g. 1024-bit integers) - something that low-level languages such as C/C++ do not handle out of the box. You can either port an existing library (such as [https://gmplib.org/ GMP]) to your operating system or write your own large integer library.

The most common operation (used among others by the RSA and Diffie-Hellman Ephemeral key exchange) is modular exponentiation i.e. computing a<sup>b</sup> mod c. This operation requires to implement multiplication, addition and modulo (which in turns requires to implement comparison, multiplication and subtraction) over large integers. You can find on Wikipedia an [https://en.wikipedia.org/wiki/Modular_exponentiation#Right-to-left_binary_method algorithm] for modular exponentiation which is quite efficient even when using large numbers.

For better performance, it is recommended to use the full power of 32 (or 64-bit) numbers instead of performing operations one bit at a time. Here is an example:

<source lang="C">
typedef struct {
	uint16_t size;  // number of 32-bit words
	uint32_t *data;
} LargeInt;

// Adds a to b and stores the result in a
void LargeInt_add(LargeInt *a, LargeInt *b) {
	uint64_t carry = 0;
	uint32_t *carry32 = (uint32_t*)&carry;
	int size;
	if (a->size > b->size) size = b->size;
	else size = a->size;

	for (int i=0; i< size; i++) {
		carry = (uint64_t)a->data[i] + (uint64_t)b->data[i] + carry;
		a->data[i] = carry32[0];
		carry >>= 32;
	}

	if (size+1 <= a->size)
		a->data[size] = carry32[0];
}
</source>

The same principle can be applied to multiplication: you can use the traditional [https://en.wikipedia.org/wiki/Multiplication_algorithm#Long_multiplication long multiplication algorithm], but instead of multiplying decimal digits (or even worse, bits), you can instead 32-bit "digits" which drastically reduce the number of operations to perform.

===Mind the endian===

Remember that the data sent across the network is in big endian, whereas x86 computers use little endian. Last but not least, the premaster_secret passed to the PRF should be in big endian. Don't forget to perform the necessary conversions.

== See Also ==

* [https://tools.ietf.org/html/rfc5246 The TLS 1.2 Specifications]

[[Category:Network Protocols]]
````

<a id="w003903"></a>

## W003903 — Academic Papers

Bron-ID: W003903
Namespace: Main (0)
Revisie: 31041 | Datum: 2026-10-23T21:22:10Z
Bron: https://wiki.osdev.org/Academic_Papers
Revisie-URL: https://wiki.osdev.org/index.php?title=Academic_Papers&oldid=31041

````text
Note that most of these links are to PDF files, some of which can be downloaded directly.

== Kernel Design ==
* 'The Case for Crypto Protocol Awareness in the Kernel' by Burnside and Keromytis, 2006 [[http://academiccommons.columbia.edu/catalog/ac%3A149447 Academic Commons link page]]

* 'Exokernel: An Operating System Architecture for Application-Level Resource Management', by Engler, Kaashoek and O’Toole, 1995 [[http://citeseer.ist.psu.edu/viewdoc/summary?doi=10.1.1.100.6047 CiteSeer link page]]

* 'Improving IPC by Kernel Design' by Liedtke, 1993 [[https://web.archive.org/web/20170810163050/http://zoo.cs.yale.edu/classes/cs422/2010/bib/liedtke93improving.pdf Yale PDF]]

* 'Operating System Virtualization: Practice and Experience' by Laadan and Nieh, 2008 [[http://academiccommons.columbia.edu/catalog/ac%3A111055 Academic Commons link page]]

* 'An Overview of the Synthesis Operating System' by Pu and Massalin, 1989 [[http://academiccommons.columbia.edu/catalog/ac%3A143176 Academic Commons link page]]

* 'The Protection of Information in Computer Systems' by Saltzer and Schroeder, 1975 [[http://citeseer.ist.psu.edu/viewdoc/summary?doi=10.1.1.126.9257 CiteSeer link page]] 

* 'Protecting Commodity Operating Systems through Strong Kernel Isolation' by Kemerlis, 2015 [[http://academiccommons.columbia.edu/catalog/ac%3A189973 Academic Commons link page]]

* 'Speculative Execution as an Operating System Service' by Locasto and Keromytis, 2006 [[http://academiccommons.columbia.edu/catalog/ac%3A110558 Academic Commons link page]]

* 'Synthesis: An Efficient Implementation of Fundamental Operating System Services' by Massalin, 1992 [[http://citeseerx.ist.psu.edu/viewdoc/download;jsessionid=0B740BDE247580D4D3685C6792DEE535?doi=10.1.1.29.4871&rep=rep1&type=pdf CiteSeer PDF]] [[http://valerieaurora.org/synthesis/SynthesisOS/ch1.html Valerie Aurora HTML]]

* 'The Synthesis Kernel' by Pu, Massalin, and Ioannidis, 1987 [[https://www.usenix.org/legacy/publications/compsystems/1988/win_pu.pdf USENIX PDF]]

* 'Threads and Input/Output in the Synthesis Kernel', by Pu and Massalin, 1989 [[http://www.cs.princeton.edu/courses/archive/fall05/cos518/papers/synthesis.pdf Princeton PDF]]

== File System Design ==
* 'ChunkFS' by Aurora, 2006 [[http://usenix.org/event/hotdep06/tech/prelim_papers/henson/henson.pdf usenix.org PDF]] [[http://usenix.org/event/hotdep06/tech/prelim_papers/henson/henson_html/ usenix.org HTML]] [[http://valerieaurora.org/review/chunkfs.pdf Valerie Aurora PDF]]

* 'Improving File System Reliability and Availability with Continuous Checker and Repair' by Gunawi, 2011 [[http://nma.berkeley.edu/ark:/28722/bk0009d284g UC Berkeley PDF]]

* 'Repair-Driven File System Design' by Aurora, 2006 [[http://valerieaurora.org/review/repair.pdf Valerie Aurora PDF]]

* 'Using Declarative Invariants for Protecting File-System Integrity' Sun, et. al., 2006 [[http://sigops.org/sosp/sosp11/workshops/plos/06-sun.pdf SIGOPS PDF]]

== Security ==
* 'Bro: A System for Detecting Network Intruders in Real-Time' by Paxson, 1999 [[http://citeseer.ist.psu.edu/viewdoc/summary?doi=10.1.1.106.261 CiteSeer link page]]

* 'The Protection of Information in Computer Systems' by Saltzer and Schroeder, 1975 [[http://citeseer.ist.psu.edu/viewdoc/summary?doi=10.1.1.126.9257 CiteSeer link page]] 

* 'Protecting Commodity Operating Systems through Strong Kernel Isolation' by Kemerlis, 2015 [[http://academiccommons.columbia.edu/catalog/ac%3A189973 Academic Commons link page]]

== Distributed Systems ==
* 'An Integrated Experimental Environment for Distributed Systems and Networks', by White et. al., 2002 [[http://citeseer.ist.psu.edu/viewdoc/summary?doi=10.1.1.11.3768 CiteSeer link page]]

* 'Unreliable Failure Detectors for Reliable Distributed Systems' by Chandra and Toueg, 1996 [[http://citeseer.ist.psu.edu/viewdoc/summary?doi=10.1.1.113.498 CiteSeer link page]]

* 'Virtual Time and Global States of Distributed Systems', Mattern, 1988 [[http://citeseer.ist.psu.edu/viewdoc/summary?doi=10.1.1.2.2620 CiteSeer link page]]

== Specific Implementation Techniques ==
* 'PCLSRing: Keeping Process State Modular', Bawden, 1989 (unpublished draft) [http://www.hack.org/mc/texts/pclsr.txt hack.org repository]

[[Category:Resources]]
````

<a id="w003904"></a>

## W003904 — User:Hwpplayer1

Bron-ID: W003904
Namespace: User (2)
Revisie: 19159 | Datum: 2016-05-08T22:29:48Z
Bron: https://wiki.osdev.org/User:Hwpplayer1
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AHwpplayer1&oldid=19159

````text
System programming , Assembly , C , C++ ...
````

<a id="w003905"></a>

## W003905 — User talk:Hwpplayer1

Bron-ID: W003905
Namespace: User talk (3)
Revisie: 19162 | Datum: 2016-05-08T22:32:45Z
Bron: https://wiki.osdev.org/User_talk:Hwpplayer1
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AHwpplayer1&oldid=19162

````text
I study Assembly , C , C++ and System Programming in C and System Programmers Association [http://www.csystem.org].
````

<a id="w003906"></a>

## W003906 — User talk:Vercas

Bron-ID: W003906
Namespace: User talk (3)
Revisie: 19642 | Datum: 2016-08-01T09:50:15Z
Bron: https://wiki.osdev.org/User_talk:Vercas
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AVercas&oldid=19642

````text
Apologies for this, but I had to make sure it works.  
noooo  
yeeeeees

We can edit user pages, but generally it is considered bad form. However, minor changes such as spelling could be acceptable. Talk pages, on the other hand, are fair game. --[[User:Bellezzasolo|Bellezzasolo]] 04:50, 1 August 2016 (CDT)
````

<a id="w003907"></a>

## W003907 — TFS

Bron-ID: W003907
Namespace: Main (0)
Revisie: 19213 | Datum: 2016-05-11T12:57:43Z
Bron: https://wiki.osdev.org/TFS
Revisie-URL: https://wiki.osdev.org/index.php?title=TFS&oldid=19213
Doorverwijzing: User:Bellezzasolo/CFS

````text
#REDIRECT [[User:Bellezzasolo/CFS]]
````

<a id="w003908"></a>

## W003908 — Talk:TFS

Bron-ID: W003908
Namespace: Talk (1)
Revisie: 20693 | Datum: 2017-04-10T17:46:58Z
Bron: https://wiki.osdev.org/Talk:TFS
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ATFS&oldid=20693
Doorverwijzing: User talk:Bellezzasolo/CFS

````text
#REDIRECT [[User talk:Bellezzasolo/CFS]]
````

<a id="w003909"></a>

## W003909 — Scheduler

Bron-ID: W003909
Namespace: Main (0)
Revisie: 19220 | Datum: 2016-05-13T13:01:13Z
Bron: https://wiki.osdev.org/Scheduler
Revisie-URL: https://wiki.osdev.org/index.php?title=Scheduler&oldid=19220
Doorverwijzing: Processes and Threads

````text
#REDIRECT[[Processes_and_Threads]]
````

<a id="w003912"></a>

## W003912 — TLS Handshake

Bron-ID: W003912
Namespace: Main (0)
Revisie: 28023 | Datum: 2023-07-09T18:08:57Z
Bron: https://wiki.osdev.org/TLS_Handshake
Revisie-URL: https://wiki.osdev.org/index.php?title=TLS_Handshake&oldid=28023

````text
{{In_Progress}}

Any TLS communication starts with a TLS handshake, which establishes what protocol will be used. We will focus on this page about what happens when the TLS_DHE_RSA_WITH_AES_128_CBC_SHA cipher suite is used (see [[SSL/TLS]] for more information about what this means)

==Handshake Overview==

Most packets during the communication are of type Handshake (0x16) and are followed by a Handshake packet header:

<source lang="C">
typedef struct __attribute__((packed)) {
	uint8_t content_type;  // 0x16
	uint16_t version;
	uint16_t length;
} TLSRecord;
</source>

This header may be followed by another TLS header, such as a TLS Handshake header. Like for a TCP connection, a TLS connection starts with a handshake between the client and the server:

* The client sends a Client Hello message, including a list of 32-byte list of random data and the list of its supported cipher suites. In our example we only send one supported cipher suite (code 0x0033)
* The server responds with a Server Hello message, telling the client what cipher suite is going to be used as well as its own 32-byte list of random data
* The server sends its certificates. These are used by the client to verify that it is actually talking to the site it thinks it is talking to, as opposed to a malicious site
* The server sends a Server Key Exchange message, initiating the key exchange and signing it with its public key
* The server sends a Server Hello Done message, indicating it is waiting for the client
* The client sends a Client Key Exchange message, containing its part of the key exchange transaction
* The client sends a Change Cipher Spec message
* The client sends a Encrypted Handshake Message
* The server sends a Change Cipher Spec
* The server sends a Encrypted Handshake Message
* The client and the server can communicate by exchanging encrypted Application Data messages

The Change Cipher Spec message tells the other party its is OK with the terms of the handshake.

The Encrypted Handshake messages are the first ones to be sent encrypted. They contain a hash of the initial handshake messages and are here to ensure these were not tampered with.

Any subsequent communication is of type Application Data and encrypted.

==Client Hello Message==

The Client Hello message initiates the TLS handshake. It is composed of a specific header, followed by some (optional) extensions, followed by some optional padding. If some servers such as wikipedia.org are quite forgiving in the types of Client Hello messages they accept, others such as Google require the Client Hello message to be exactly 512 bytes (excluding the TLS Record header) and declare a server_name extension

Here is an example of a Client Hello message:

<source lang="text">
0000   01 00 01 fc 03 03 57 16 ea ce ec 93 89 5c 4a 18
0010   d3 1c 5f 37 9b b3 05 b4 32 08 29 39 b8 3e e0 9f
0020   9a 96 ba be 0a 40 00 00 02 00 33 01 00 01 d1 ff
0030   01 00 01 00 00 00 00 16 00 14 00 00 11 77 77 77
0040   2e 77 69 6b 69 70 65 64 69 61 2e 6f 72 67 00 0d
0050   00 12 00 10 06 01 06 03 05 01 05 03 04 01 04 03
0060   02 01 02 03 00 0b 00 02 01 00 00 0a 00 06 00 04
0070   00 17 00 18 00 15 01 88 00 00 00 00 00 00 00 00
0080   00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
...
01e0   00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
01f0   00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
</source>

* 0x01: Client Hello handshake type
* 0x0001FC: size=508 (512-4)
* 0x0303: TLS 1.2
* 0x5716...0A40: 32-bytes random client data
* 0x00: session ID length=0 (if it was not null, it would be followed by the session ID)
* 0x0002: Cipher Suites Length=2 (1 cipher suite supported)
** 0x0033: TLS_DHE_RSA_WITH_AES_128_CBC_SHA cipher suite code
* 0x01: number of compression methods=1
** 0x00: compression method=null
* 0x01D1: extensions length=465
* 0xFF01: extension type=renegotiation_info
** 0x0001: length=1
** 0x00: renegotiation info extension length=0
* 0x0000: extension type=server_name (the domain name we are trying to contact)
** 0x0016: length=22
** 0x0014: server name list length=20
** 0x00: server name type=host_name
** 0x0011: server name length=17
** 0x7777772E...7267: "www.wikipedia.org"
* 0x000D: extension type=signature_algorithms (the signature algorithms supported)
** 0x0012: length=18
** 0x0010: signature hash algorithms length=16
*** 0x0601: SHA512 + RSA
*** 0x0603: SHA512 + ECDSA
*** 0x0501: SHA384 + RSA
*** 0x0503: SHA384 + ECDSA
*** 0x0401: SHA256 + RSA
*** 0x0403: SHA256 + ECDSA
*** 0x0201: SHA1 + RSA
*** 0x0203: SHA1 + ECDSA
* 0x000B: extension type=ec_point_formats (only used if you use elliptic curve cryptography)
** 0x0002: length=2
** 0x02: EC point formats length=1
** 0x00: EC point format=uncompressed
* 0x000A: extension type=elliptic_curves (Elliptic Curve types supported - only used if you use elliptic curve cryptography)
** 0x0006: length=6
** 0x0004: elliptic curve length=4 (2 curves)
** 0x0017: secp256r1 elliptic curve
** 0x0018: secp384r1 elliptic curve
* 0x0015: extension type=padding (we fill the rest of the 512 bytes with zeros)
** 0x0188: length=392
** 0x0000...0000: 392 bytes of padding data

==Server Hello Message==

The Server Hello message indicates which cipher suite is going to be used and provides some server random data that will be used later on.

Here is a sample Server Hello message:

<source lang="text">
0000   02 00 00 4d 03 03 c8 ca 5c 5f 83 79 eb 8f 8a 16
0010   86 c2 07 d7 42 c7 ee b9 dc 71 b7 f1 71 9f eb 51
0020   66 24 b4 1e 4f 6c 20 e4 02 80 10 1d ea 7f aa ee
0030   5d 4f ac 53 49 29 25 ec 29 a8 b7 23 fa ef 24 d4
0040   47 2e 90 7b 99 36 2b 00 33 00 00 05 ff 01 00 01
0050   00
</source>

* 0x02: Handshake type=Server Hello
* 0x00004D: length=77
* 0x0303: TLS version 1.2
* 0xC8CA...4F6C: 32-bytes server random data
* 0x20: session ID length=32
* 0xE402...362B: session ID (can be used in a future TLS connection to avoid going through the handshake again)
* 0x0033: cipher suite used=TLS_DHE_RSA_WITH_AES_128_CBC_SHA
* 0x00: compression method used=null
* 0x0005: extensions length=5
* 0xFF01: extension type=renegotiation_info
** 0x0001: length=1
** 0x00: renegotiation info extension length=0

==Certificate Message==

The server then sends a Certificate message containing its SSL Certificate chain. The first certificate is the server's SSL certificate. The next certificate is the certificate from a Certificate Authority (CA) which signed the first certificate. The next certificate signs the previous certificate, and so on. The last certificate in the chain should belong to a root CA and is self-signed (each TLS client should have a list of all the root CAs)

Here is how the Certificate Message is encoded:

* 0x0B: handshake type=Certificate
* 0x000C58: length=3160
* 0x000C55: certificates length=3157
** 0x0007E2: certificate #1 Length=2018
*** 0x3082...C0F3: first certificate (ASN.1 encoded)
** 0x00046D: certificate #2 length=1133
*** 0x3080...4998: second certificate (ASN.1 encoded)

==Key Exchange==

TLS encryption is performed using symmetric encryption. The client and server thus need to agree on a secret key. This is done in the key exchange protocol.

In our example, TLS is using the DHE/RSA algorithms: the Diffie-Hellman Ephemeral protocol is used to come up with the secret key, and the server is using the RSA protocol to sign the numbers it sends to the client (the signature is linked to its SSL certificate) to ensure that a third party cannot inject a malicious number. The upside of DHE is that it is using a temporary key that will be discarded afterwards. Key exchange protocols such as DH or RSA are using numbers from the SSL certificate. As a result, a leak of the server's private key (for example through [https://en.wikipedia.org/wiki/Heartbleed Heartbleed]) means that a previously recorded SSL/TLS encryption can be decrypted. Ephemeral key exchange protocols such as DHE or ECDHE offer so-called forward secrecy and are safe even if the server's private key is later compromised.

Diffie-Hellman Ephemeral works as follows:

* The server comes up with a secret number y, with a number g and a modulo p (p typically being a 1024 bit integer) and sends (p, g, pubKey=g<sup>y</sup> mod p) to the client in its "Server Key Exchange" message. It also sends a signature of the Diffie-Hellman parameters (see SSL Certificate section)
* The client comes up with a secret number x and sends pubKey=g<sup>x</sup> mod p to the server in its "Client Key Exchange" message
* The client and server derive a common key premaster_secret = (g<sup>x</sup>)<sup>y</sup> mod p = (g<sup>y</sup>)<sup>x</sup> mod p = g<sup>xy</sup> mod p. If p is large enough, it is extremely hard for anyone knowing only g<sup>x</sup> and g<sup>y</sup> (which were transmitted in clear) to find that key.

Because computing g<sup>xy</sup> mod p using 1024-bytes integers can be tedious in most programming languages, if security is not a concern, one way to avoid this is to use x=1. This way, premaster_secret is just g<sup>y</sup> mod p, a value directly sent by the server. The security in such a case is of course compromised.

premaster_key is however only a first step. Both client and server uses the PRF function to come up with a 48-byte master secret. The PRF function is used once again to generate a 104-bytes series of data which will represent all the secret keys used in the conversation (the length may differ depending on the cipher suite used):

<source lang="python">
# g_y, g and p are provided in the Server Key Exchange message
# The client determines x
premaster_secret = pow(g_y, x, p)

# client_random and sever_random are the 32-bytes random data from the Client Hello and Server Hello messages
master_secret = PRF(premaster_secret, "master secret", client_random + server_random, 48)
keys = PRF(master_secret, "key expansion", server_random + client_random, 104)

# The MAC keys are 20 bytes because we are using HMAC+SHA1
client_write_MAC_key = keys[0:20]
server_write_MAC_key = keys[20:40]
# The client and server keys are 16 bytes because we are using AES 128-bit aka a 128 bit = 16 bytes key
client_write_key = keys[40:56]
server_write_key = keys[56:72]
# The IVs are always 16 bytes because AES encrypts blocks of 16 bytes
client_write_IV = keys[72:88]
server_write_IV = keys[88:104]
</source>

Note how different secret keys are used for the client and for the server, as well as for encryption and to compute the MAC.

==Server Key Exchange Message==

The server then sends its part of the key exchange. In the following case it will send the Diffie-Hellman parameters p, g and pubKey=g<sup>y</sup>

* 0x0C: handshake type=server key exchange
* 0x00030B: length=779
* 0x0100: p length=256
** ..... : 256-bytes p (1024-bit integer)
* 0x0001: g length=1
** 0x02: g = 2
* 0x0100: pubKey length
** ... : 256-bytes pubKey
* 0x0601: signature hash algorithm used to sign the Diffie-Hellman parameters=SHA512+RSA
* 0x0100: signature length=256
** ... : 256-bytes signature

==Server Hello Done Message==

This message indicates that the server is done and is awaiting for client input.

<source lang="C">
typedef struct __attribute__((packed)) {
	uint8_t handshake_type;  // 0x0E
	uint8_t length[3];  // 0x000000
} TLSServerHelloDone;
</source>

==Client Key Exchange Message==

The client then sends its key exchange parameters: pubKey=g<sup>x</sup>

* 0x10: handshake type=client key exchange
* 0x000102: length=258
* 0x0100: pubKey length=256
* ...: 256-bytes pubKey

==Change Cipher Spec Message==

The client sends the Change Cipher Spec message to indicate it has completed its part of the handshake. The next message the server will expect is the Encrypted Handshake Message.

The whole message (including the TLS Record header) is 6 bytes long:

<source lang="C">
typedef struct __attribute__((packed)) {
	uint8_t content_type;  // 0x14
        uint16_t version; // 0x0303 for TLS 1.2
	uint8_t length;  // 0x01
        uint8_t content;  // 0x01
} TLSChangeCipherSpec;
</source>

==Another Key Exchange: Elliptical Curve Diffie Hellman Ephemeral==

If Diffie-Hellman is a very powerful algorithm, it requires very large numbers to be considered secure (1024-bit at minimum). A variant is Elliptical Curve Diffie-Hellman, which is much harder to break even with 256-bit numbers. Numerous TLS cipher suites now rely on the ECDHE_RSA key exchange instead of DHE_RSA, like in the TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA cipher suite.

Elliptic Curve Diffie-Hellman works as follows: consider a point G = (x, y) on a curve y<sup>2</sup> = x<sup>3</sup> + a.x + b mod p. Both parties come up with secret numbers secret1 and secret2, and will send each other G.secret1 and G.secret2 (G.secret1 means multiplying the point G to secret1 using Elliptic Curve point multiplication). The shared secret key is G.secret1.secret2.

TLS can use the ECDHE key exchange to come up with an ephemeral shared secret key the following way:

* The server indicates in the Server Key Exchange message what type of curve is going to be used (secp256r1 is a very common one). This tells what parameters a, b, p and G to use (see [http://www.secg.org/sec2-v2.pdf] to see the domain parameters for each curve)
* The server comes up with a random 256-bit number (or whatever the curve says) server_secret and sends pubKey = G*server_secret in the Server Key Exchange message. pubKey is sent as a 65-bytes block composed of the concatenation 0x04 | Gx | Gy (both numbers being 32-bytes long)
* The client comes up with a random 256-bit number client_secret and sends pubKey = G*client_secret in the Client Key Exchange message. pubKey is sent in the same format as the server's
* Both parties will derive premaster_secret by computing server_pubKey * client_secret = client_pubKey * server_secret = G * client_secret * server_secret. The x coordinate of this result is the premaster_secret
* Once premaster_secret is determined, the rest of the computation works the same regardless of the key exchange protocol used

Regarding how to compute elliptic curve point multiplication, Wikipedia offers [https://en.wikipedia.org/wiki/Elliptic_curve_point_multiplication more details]. Note that, because we are only dealing with integers, you should use [https://en.wikipedia.org/wiki/Modular_multiplicative_inverse modular multiplicative inverse] instead of divisions.

If you want to test Elliptic Curves in Python, [https://pypi.python.org/pypi/tinyec TinyEC] is a very useful package (along with the [https://github.com/alexmgr/tinyec source code] in pure Python):

<source lang="python">
import tinyec.ec as ec
import tinyec.registry as reg

# Get the domain parameters for the named curve specified in the Server Key Exchange message
curve = reg.get_curve("secp256r1")

# Comes up with a random 256-bit (32 bytes) client_secret
# curve.g is a point on the elliptic curve, defined by the domain parameters
# We multiply it with client_secret to obtain the public key
client_pubKey = curve.g * client_secret
# Retrieved from the Server Key Exchange message
server_pubKey = ...

premaster_secret = (server_pubKey * client_secret).x
</source>

==Another Key Exchange: RSA==

The RSA key exchange is by today's standard an old key exchange protocol and does not provide forward secrecy. It is however simpler to implement than ECDHE and still supported by multiple sites (for example, Google supports the RSA key exchange but not the simple Diffie-Hellman key exchange).

With the RSA key exchange, the server does not send any "Server Key Exchange" message. Instead, the client decides the premaster_secret, which is a 48-bytes string composed of a two-bytes TLS version (0x0303 for TLS 1.2) followed by 46 random bytes. It then encrypts that premaster_secret using the [https://tools.ietf.org/html/rfc2313 PKCS #1 protocol] (aka RSA encryption version 1.5) as well as the key from the Web site's certificate as the public key. Because the public key is always the same, a leak of the Web site's private key would allow to decrypt previously recorded TLS conversations. This is why the RSA key exchange protocol offers no forward secrecy.

<source lang="python">
import os
from Crypto.Cipher import PKCS1_v1_5
from Crypto.PublicKey import RSA

# Come up with a random premaster_secret
# It is recommended to use a random generator from a crypto library if possible
premaster_secret = '\x03\x03' + os.urandom(46)

# Retrieve the pubKey from the first certificate. This pubKey value (65-bytes for a 1024-bit public key) is stored in the ASN.1 format
key = RSA.importKey(pubKey)

# Encrypt the premaster_secret
cipher = PKCS1_v1_5.new(key)
encrypted_premaster_secret = cipher.encrypt(premaster_secret)
</source>

Without the Crypto.Cipher.PKCS1_v1_5 package:

<source lang="python">
import os

# Come up with a random premaster_secret
premaster_secret = '\x03\x03' + os.urandom(46)

# Retrieve the pubKey from the first certificate (parse_ASN1 is a fictional function)
RSA_n, RSA_e = parse_ASN1(pubKey)

# Encrypt the premaster_secret
premaster_secret = '\x00\x02' + '\x42' * (256 - 3 - len(premaster_secret)) + '\x00' + premaster_secret
encrypted_premaster_secret = pow(to_int(premaster_secret), RSA_e, RSA_n)
</source>

==SSL Certificate (optional)==

In order to prevent a Man-In-The-Middle attack (MITM), the server will sign the Diffie-Hellman parameters it sent to the client. Because the client may have never contacted the server before (and thus cannot securely obtain its public key), the client and server rely on a trusted third party known as a Certificate Authority (CA).

In order to verify the signature using the RSA algorithm, the client need to do the following:

* Retrieve the Certificate message sent by the server, which contains one or more certificates (look at a such a packet in Wireshark)
* Verify that the first certificate's RDN sequence (signedCertificate / subject:rdnSequence / rdnSequence) contains the Web site the client is trying to contact
* Get the RSA e and n values from the first certificate's public key (signedCertificate / subjectPublicKeyInfo / subjectPublicKey). Those parameters are encoded using the [https://en.wikipedia.org/wiki/Abstract_Syntax_Notation_One ASN.1] format (as a verification, e is very often 65537, or 0x10001)
* Compute the hash of the whole DH parameters (as sent by the server) preceded with the client and server random data. The certificate indicates what type of hash to use (signedCertificate / subjectPublicKeyInfo / algorithm):
* Compute signature<sup>e</sup> mod n, convert it to a string and take the last 20 bytes (or more, depending on the hash function being used)
* Both computations should be the same
* Because this certificate is probably generated by an intermediate CA, the client needs to verify that certificate
* Compute the hash of the whole signedCertificate section and repeat the operation using the next certificate
* Follow the certificate chain up to the end. The last certificate should belong to a root CA (any TLS implementation should contain a list of the root CAs and their public key) and is self-signed

==Encrypted Handshake Message==

The TLS handshake is concluded with the two parties sending a hash of the complete handshake exchange, in order to ensure that a middleman did not try to conduct a downgrade attack.

If your TLS client technically does not have to verify the Encrypted Handshake Message sent by the server, it needs to send a valid Encrypted Handshake Message of its own, otherwise the server will abort the TLS session.

Here is what the client needs to do to create :

* Compute a SHA256 hash of a concatenation of all the handshake communications (or SHA384 if the PRF is based on SHA384). This means the Client Hello, Server Hello, Certificate, Server Key Exchange, Server Hello Done and Client Key Exchange messages. Note that you should concatenate only the handshake part of each TLS message (i.e. strip the first 5 bytes belonging to the TLS Record header)
* Compute PRF(master_secret, "client finished", hash, 12) which will generate a 12-bytes hash
* Append the following header which indicates the hash is 12 bytes: 0x14 0x00 0x00 0x0C
* Encrypt the 0x14 0x00 0x00 0x0C | [12-bytes hash] (see the Encrypting / Decrypting data section). This will generate a 64-bytes ciphertext using AES-CBC and 40 bytes with AES-GCM
* Send this ciphertext wrapped in a TLS Record

The server will use a similar algorithm, with two notable differences:

* It needs to compute a hash of the same handshake communications as the client as well as the decrypted "Encrypted Handshake Message" message sent by the client (i.e. the 16-bytes hash starting with 0x1400000C)
* It will call PRF(master_secret, "server finished", hash, 12)

[[Category:Security]]
````

<a id="w003913"></a>

## W003913 — TLS Encryption

Bron-ID: W003913
Namespace: Main (0)
Revisie: 28485 | Datum: 2023-12-02T12:55:11Z
Bron: https://wiki.osdev.org/TLS_Encryption
Revisie-URL: https://wiki.osdev.org/index.php?title=TLS_Encryption&oldid=28485

````text
{{In_Progress}}

Once the [[TLS Handshake]] has been completed, the two parties can start communicating the way they would normally do. Only now, they do so by encrypting any message and sending a TLS Record instead. We will focus on this page about what happens when the TLS_DHE_RSA_WITH_AES_128_CBC_SHA cipher suite is used (see [[SSL/TLS]] for more information about what this means)

At this point, any TLS packet -as sent on top of TCP- is of type Application Data (save TLS Alert messages) and looks like:

<source lang="C">
typedef struct __attribute__((packed)) {
	uint8_t content_type;  // 0x17 for Application Data
	uint16_t version;      // 0x0303 for TLS 1.2
	uint16_t length;       // length of encrypted_data
        uint8_t encrypted_data[];
} TLSRecord;
</source>

==Encrypting / Decrypting data with AES CBC==

Any encrypted data in this example is using AES 128-bit in CBC mode. AES encrypts 128-bit (16 bytes) blocks of data using a 128, 192 or 256-bit secret key. The CBC mode tells how to use AES to encrypt some plaintext which is not 16-bytes long (and no, you don't want to encrypt each 16-byte block [https://en.wikipedia.org/wiki/Block_cipher_mode_of_operation#Electronic_Codebook_.28ECB.29 independently])

The following steps needs to be implemented:

* Create an intermediary plaintext which concatenates:
** The 8-bytes sequence number. This number is 0 for handshake messages, 1 for the first application data message, 2 for the next application data message, etc.
** The 1-byte content type (0x16 for a handshake message, 0x17 for an application data message)
** The TLS version (0x0303)
** The 2-bytes plaintext length
** The original plaintext
* Compute the MAC on that intermediary plaintext using HMAC+SHA1 and the client/server_write_MAC_key
* The final plaintext will be the concatenation of [original plaintext] + [20-bytes MAC] + [CBC padding]. Because AES-CBC only encrypts data whose size is a multiple of 16, the CBC padding is composed of bytes to fill to 16 (16 full bytes if the plaintext size is already a multiple of 16). The value of each of those padding bytes is the length of the padding + 1. So in the case of a 16-bytes plaintext, the final plaintext would be [16-bytes plaintext] | [20 bytes MAC] | 0x0B0B0B0B0B0B0B0B0B0B0B0B
* Come up with a random 16-bytes IV
* Encrypt the final plaintext using the client/server_write_key and this IV
* The ciphertext is the concatenation of IV + ciphertext

<source lang="python">
from Crypto.Hash import *
from Crypto.Cipher import AES

def to_n_bytes(number, size):
	h = '%x' % number
	s = ('0'*(size*2 - len(h)) + h).decode('hex')
	return s

def encrypt(plaintext, iv, key_AES, key_MAC, seq_num, content_type):
    hmac = HMAC.new(key_MAC, digestmod=SHA)
    plaintext_to_mac = to_n_bytes(seq_num, 8) + to_n_bytes(content_type, 1) + '\x03\x03' + to_n_bytes(len(plaintext), 2) + plaintext
    hmac.update(plaintext_to_mac)
    mac_computed = hmac.digest()

    cipher = AES.new(key_AES, AES.MODE_CBC, iv)
    plaintext += mac_computed
    padding_length = 16 - (len(plaintext) % 16)
    if padding_length == 0:
        padding_length = 16

    padding = chr(padding_length - 1) * padding_length
    ciphertext = cipher.encrypt(plaintext + padding)

    return ciphertext

def decrypt(message, key_AES, key_MAC, seq_num, content_type, debug=False):
    iv = message[0:16]
    cipher = AES.new(key_AES, AES.MODE_CBC, iv)
    decoded = cipher.decrypt(message[16:])

    padding = to_int(decoded[-1:]) + 1
    plaintext = decoded[0:-padding-20]
    mac_decrypted = decoded[-padding-20:-padding]

    hmac = HMAC.new(key_MAC, digestmod=SHA)
    plaintext_to_mac = to_n_bytes(seq_num, 8) + to_n_bytes(content_type, 1) + '\x03\x03' + to_n_bytes(len(plaintext), 2) + plaintext
    hmac.update(plaintext_to_mac)
    mac_computed = hmac.digest()

    if debug:
        print('Decrypted: [' + decoded.encode('hex') + ']')
        print('Plaintext: [' + plaintext.encode('hex') + ']')
        print('MAC (decrypted): ' + to_hex(mac_decrypted))
        print('MAC (computed):  ' + to_hex(mac_computed))
        print('')

    return plaintext
</source>

==Another Encryption: AES GCM (Optional)==

Another popular mode of operation used by TLS in conjunction with AES is the [https://en.wikipedia.org/wiki/Galois/Counter_Mode Galois Counter Mode (GCM)].

The Galois Counter Mode is basically the regular [https://en.wikipedia.org/wiki/Block_cipher_mode_of_operation#Counter_.28CTR.29 Counter Mode] combined with its own authentication tag based on a Galois Field. As a result, GCM contains its own MAC (contrary to the CBC mode). The cryptographic hash mentioned in the cipher suite (e.g. SHA256 in TLS_DHE_RSA_WITH_AES_128_GCM_'''SHA256''') is only used by the PRF and to generate the Encrypted Handshake Message.

As a result, the GCM does not require MAC keys, and needs less data from the key expansion:

<source lang="python">
keys = PRF(master_secret, "key expansion", server_random + client_random, 40)

# We assume a 128-bit AES key
client_write_key = keys[0:16]
server_write_key = keys[16:32]
# The IVs are only 4 bytes long
client_IV = keys[32:36]
server_IV = keys[36:40]
</source>

The GCM algorithm take as input an initialization vector (IV), the plaintext, an AES key as well as authenticated data. While those may vary depending on the use of GCM, TLS is using the following format:

* The IV is a 12-bytes array composed of an implicit part and an explicit part
** The first four bytes (the implicit IV) are the client or server IV (depending on who's writing the message) which was derived from the master secret
** The explicit IV is a 8-bytes random sequence
* The Authenticated data is a concatenation of:
** The sequence number (8 bytes)
** The content type (1 byte)
** The TLS version (0x0303 for TLS 1.2)
** The plaintext size (2 bytes) which should be the final encrypted message size minus 24 (8 for the IV, 16 for the MAC)

The output sent by TLS is the concatenation of:

* The explicit IV (8 bytes). There is no need to send the implicit IV as both sides can derive it from the key expansion
* The ciphertext, generated using the counter mode, and using the IV as an initial counter
* The Authentication Tag (16 bytes), generated by a Galois Field operation (see below). If you technically do not need to verify the Authentication Tag for server messages, you need to compute the correct tag for encrypted data you send to the HTTPS server as it will verify it and ignore the communication if it is incorrect

Here is some pseudo Python code which encrypts a plaintext using the AES-CBC mode (you will need to implement your own GHASH function)

<source lang="python">
def encrypt(plaintext, key_AES, seq_num, content_type):
    iv = client_IV + os.urandom(8)

    # Encrypts the plaintext
    plaintext_size = len(plaintext)
    counter = Counter.new(nbits=32, prefix=iv, initial_value=2, allow_wraparound=False)
    ciphertext = AES.new(AES_key, AES.MODE_CTR, counter=counter).encrypt(plaintext)

    # Compute the Authentication Tag
    H = to_int(AES.new(AES_key, AES.MODE_ECB).encrypt('\x00'*16))
    auth_data = to_n_bytes(seq_num, 8) + to_n_bytes(content_type, 1) + TLS_VERSION + to_n_bytes(plaintext_size, 2)
 
    auth_tag = GHASH(H, auth_data, ciphertext)
    auth_tag ^= to_int(AES.new(self.client_AES_key, AES.MODE_ECB).encrypt(iv + '\x00'*3 + '\x01'))
    auth_tag = to_bytes(auth_tag)

    return iv[4:] + ciphertext + auth_tag
</source>

[[Category:Security]]
````

<a id="w003914"></a>

## W003914 — Go Barebones

Bron-ID: W003914
Namespace: Main (0)
Revisie: 19277 | Datum: 2016-05-27T20:24:27Z
Bron: https://wiki.osdev.org/Go_Barebones
Revisie-URL: https://wiki.osdev.org/index.php?title=Go_Barebones&oldid=19277
Doorverwijzing: Go Bare Bones

````text
#REDIRECT[[Go_Bare_Bones]]
````

<a id="w003915"></a>

## W003915 — ICMP

Bron-ID: W003915
Namespace: Main (0)
Revisie: 19281 | Datum: 2016-05-28T12:20:27Z
Bron: https://wiki.osdev.org/ICMP
Revisie-URL: https://wiki.osdev.org/index.php?title=ICMP&oldid=19281
Doorverwijzing: Internet Control Message Protocol

````text
#REDIRECT[[Internet Control Message Protocol]]
````

<a id="w003916"></a>

## W003916 — Local Descriptor Table

Bron-ID: W003916
Namespace: Main (0)
Revisie: 28315 | Datum: 2023-09-23T22:21:53Z
Bron: https://wiki.osdev.org/Local_Descriptor_Table
Revisie-URL: https://wiki.osdev.org/index.php?title=Local_Descriptor_Table&oldid=28315

````text
A Local Descriptor Table (LDT) is like the [[GDT|Global Descriptor Table]] in that it holds [[Segmentation|Segment]] descriptors for access to memory. The difference is that every Task/thread can have its own LDT, and the OS can change the LDT Register (LDTR) on every Task switch.

That means that every program can have its own list of memory Segment descriptors, and keep them private from other programs:
* The Code, Data and Heap segments can be private in the LDT - separate from other programs, but available to this program;
* Each Task/thread within this program can have its own Stack in the LDT, and yet still be able to access the above Segments;
* Sharing within the program is automatic: just 'know' the correct Descriptor reference;
* If another program (with another LDT) was to attempt to access one of these Segments, it would access its ''own'' LDT's Segment rather than the target Segment.

= What is an LDT? =
The Local Descriptor Table is a list of descriptors for the current program. It is distinct from the GDT in a number of respects:
* It cannot hold many types of descriptors;
* The GDT holds a reference to the LDT (the Base and Limit of the table in memory);
* It is only accessible by Tasks/threads with the same Local Descriptor Table Register (LDTR) value.

== Only certain Descriptors ==
Since the LDT is local to the current Task, there are many things that the LDT should not (and in fact cannot) hold:
* Code for Interrupt handlers;
* Data for system-wide resources (e.g. keyboard buffer);
* System segments ([[TSS|Task State Segment]]s and Local Descriptor Tables).

If you think about it, this makes sense: if an interrupt or task-switch occurs, the current Task cannot be expected to hold ''every'' possible reference for every system structure - that's what the [[GDT]] is for!

== GDT holds reference ==
If the LDT needs to be known to the system, then it needs to be held to the same security ideal as the rest of the system. You wouldn't want a rogue Task to define an LDT that could access sensitive memory! Therefore, the LDT is defined as a 'normal' memory Segment inside the [[GDT]] - simply with a Base memory address and Limit. But the fact that it is identified by a specific Descriptor Type (LDT=0x02) means that the Descriptor cannot be loaded into a standard Segment register (CS,DS,ES,FS,GS,SS): the only way that the Local Descriptor Table Register (LDTR) can be loaded with this value is via the LLDT instruction or via a Task State Segment (TSS) - that's right: if you use the '386's hardware tasking mechanism, the TSS holds the Task's LDT and is automatically loaded on a task switch!

== Only accessible within current Task ==
The whole point of using an LDT is to localize this program's Segments to only certain Tasks. Each program would define its own Code, Data and Heap Segments, and a Stack Segment for each Task within the program. Any attempt to access the Segment from outside the program is futile: it would only access the referencing Task's LDT instead (see Introduction above).

= How are LDTs defined and accessed? =
An LDT is a block of (linear) memory up to 64K in size, just like the [[GDT]]. It can be Paged through the standard mechanism, so is treated just like normal memory in that respect. The difference from the GDT is in the Descriptors that it can store, and the method used to access it.

In Protected Mode, a Segment Selector (CS,DS,ES,FS,GS,SS) is used to index a Descriptor from a Descriptor Table. The 16-bit Segment Selector is made of three fields:

<pre>
 Selector
+=============+=+==+
| Index       |T|PL|
+=============+=+==+
 1111110000000 0 00
 5432109876543 2 10

Index = Index within Descriptor Table
T     = Which Table: 0=GDT, 1=LDT
PL    = Requested Privilege Level (RPL) *
* The RPL is compared against the Privilege Level encoded inside the Descriptor (Descriptor Privilege Level - DPL).
Any mismatches and the request causes a Fault.
</pre>

So:
* 0x0000-0x0003 loaded into a Segment Register all work - but attempting to use that Segment Register causes a General Protection Fault (GPF) - the zero'th entry of the GDT is reserved to allow a NULL Selector;
* 0x0004-0x0007 references the 0x000'th Descriptor in the (current) LDT (with an RPL of 0-3);
* 0x0008 references the 0x001'st Descriptor in the GDT (with an RPL of 0);
* 0x000C references the 0x001'st Descriptor in the (current) LDT (with an RPL of 0);
* 0xF002 references the 0x1E0'th Descriptor in the GDT (with an RPL of 2);
* 0xF006 references the 0x1E0'th Descriptor in the (current) LDT (with an RPL of 2).

[[Category:X86 CPU]]
[[Category:Memory Segmentation]]
````

<a id="w003919"></a>

## W003919 — Going Further on x86

Bron-ID: W003919
Namespace: Main (0)
Revisie: 28742 | Datum: 2024-04-23T14:07:11Z
Bron: https://wiki.osdev.org/Going_Further_on_x86
Revisie-URL: https://wiki.osdev.org/index.php?title=Going_Further_on_x86&oldid=28742

````text
{{Rating|2}} {{Template:Kernel designs}}
You have completed [[Bare Bones]] for [[x86]]. Now what, you may be wondering. Welcome to the OS development world!

The following guide assumes you are doing the things discussed below in order from top to down. Before starting implementing things from this, it is advised that you read it whole to get a broader perspective.

== Preparing For Real ==

Before going any further:
* You should grab a copy of the Intel manuals. Most of the processor-specific things discussed below are best described in the Intel manuals.
* You should ensure you have enough patience and time. Operating system development is one of the most time-consuming projects.

== Design Considerations ==
Designing the operating system as a whole, and part to part, is a tricky, yet important task to accomplish.
<!--
=== Kernel Model ===

You should decide which kernel model will you implement. Will it be a [[Monolithic Kernel|monolithic kernel]]? Will it be a [[Microkernel|microkernel]]? Will it be [[Kernel#Subset Kernel Models|something else]]? Or maybe something entirely new? It's up to you!
-->
=== Code Structuring ===

You should decide how your code should be structured. Consider that you will eventually port your operating system to different architectures, with different assembly instructions, with different initialisation sequences, with different hardware, with different memory structures, etc. You have to ensure you don't mix files from one architecture with files from another architecture. [[Meaty Skeleton]] serves as a minimal example of how to structure your code.

=== Future Proofing ===

Consider that you will eventually want to add new features to the whole range of kernel interfaces. Therefore, you have to make sure it is easy to refactor them without breaking the dependant interfaces.

=== Multithreading ===

Realise that, in some time from now, you will not run your code in a single thread, but it '''will''' be running in parallel with other threads and with other processors, meaning that it will be eventually preempted during a critical operation, and that some other thread will be able to corrupt the saved state.

=== Hardware Abstraction ===

Realise that not every piece of hardware is present in every computer, so you might want to abstract it through a [[Hardware Abstraction Layer]]. For example [[PIT]] and [[HPET]] are two timers, which you might want to abstract into a single timing interface.

=== Algorithms ===

For every task, find the algorithm that gets the most points from every aspect that matters to you (for example simplicity, speed, memory usage, etc).

== Paging ==

[[Paging]] is nice because it lets you map memory however you want and it lets processes see the full address space. It also provides advanced protection capabilities. You may wish to enable it early in <tt>boot.s</tt>.

=== Higher Half ===

You should continue with a [[Higher Half Kernel|higher half kernel]], so user-space programs can be loaded at 4 MiB (or lower if you like) without conflicting with the kernel binary. An important decision to take is where to exactly map the kernel.

Many prefer mapping the kernel to 0x80000000, leaving 2 GiB for kernel data and 2 GiB for processes. This can allow a monolithic kernel to cache large files or filesystem structures.

Others prefer mapping it to 0xC0000000, leaving 1 GiB for kernel data and 3 GiB for processes. Their main argument is that it integrates better with [[PAE]], since the whole kernel space fits exactly in one page directory.

Some others go extreme and map the kernel at 0xE0000000, leaving the tight space of 512 MiB for kernel data and 3.5 GiB for processes. Their main argument is that user-space should be able to use as much memory as possible.

=== Permissions ===

[[Bare Bones]] taught you to use an [[ELF]] binary for the kernel. Do you know what do <tt>.text</tt>, <tt>.data</tt>, <tt>.rodata</tt>, <tt>.bss</tt> mean? Right, they are the sections of the executable. In <tt>.text</tt> the instructions for the processor are stored, in <tt>.data</tt> there is data, in <tt>.rodata</tt> there is read-only data, in <tt>.bss</tt> there is uninitialised data. There may be more sections, but let's focus on these for now.

Each of these sections should have different permissions applied, for example <tt>.text</tt> should be read-only, <tt>.data</tt> should be read-write, <tt>.rodata</tt> should be read-only, and <tt>.bss</tt> should be read-write. For these changes to take effect in kernel-space, you have to set the WP bit in the appropriate register. Make sure you disable execution of non-<tt>.text</tt> sections in [[x86-64]] and [[PAE]] modes.

To facilitate setting permissions on sections you can do the following:
* Tell the linker to align them at 4 KiB boundaries, so sections occupy whole pages.
* Tell the linker to insert symbols indicating the start and end addresses of specific sections, so you can access them from your mapping code.

== More x86 Specific Things ==

An operating system should be self-reliant as much as possible. The bootloader may have left the environment in a "working" state, but it is not convenient in the long run.

=== Segments ===

Create a [[GDT]] before the first segment change, because the one that [[GRUB]] has setup is no longer valid (the entries are simply cached in the processor still, that's why it "works").

You need at least these entries: null segment entry, kernel code segment entry, kernel data segment entry, user code segment entry, user data segment entry, [[Task State Segment|task state segment]] entry.

=== Interrupts ===

Every real operating system handles [[Exceptions|exceptions]] (for example [[Exceptions#Page_Fault|page faults]]), and reads from peripherals only when data is received (instead of polling).

Create an [[IDT]]. Write interrupt handlers. Enable an interrupt controller (for example [[PIC]] or [[APIC]]).

Make sure to save all registers at the start of the interrupt handlers, and restore them at the end of the interrupt handlers. Keep also in mind that some exceptions cause an error code to be pushed to the stack, while some others do not.

=== Timers ===

Initialise a timer to be able to keep track of timing. Consider which timer you would like to support first (most beginners go with [[PIT]], although it is ancient), and how would you like to set it up (most set it up to tick at a convenient interval, like 1 ms or 10 ms). However, make sure you abstract the interface so adding support for more timers is easier.

=== Getting Keyboard Input ===

{{Main|PS/2 Keyboard}}

It is important to allow the user to be able to interact with the operating system. The keyboard can be read using an IO Port, however you will need to set up interrupts for proper keyboard support.

== Memory Management ==

Soon you will need to allocate something whose size is not known at compile time. This is where the [[Memory Management|memory manager]] comes in.

=== Getting the Memory Map ===

You first need to [[Detecting_Memory_(x86)#Memory_Map_Via_GRUB|obtain the memory map]], so you know which physical areas are free. Then you build on top of that.

=== Physical Memory Manager ===

You for sure also need a list of free physical pages, so you know which physical page frames to allocate next.

A common way to do it is to create a linked list, that is to store the physical address of the next free page at the start of the previous free page, so only free memory is used to store it. However, you have paging enabled, so you can't arbitrarily write to every part of memory. You can instead map a page frame at a time, and write to it the address of the next free page. Alternatively, you can have a separate mapping for all physical memory in the higher half: it is especially common in 64-bit kernels because it simplifies the design with very little downsides.

=== Virtual Memory Allocator ===

You will also need a way to allocate virtual pages for mapping physical memory, instead of hard-coded values. Get a way to track which parts of the address space are used, and which aren't.

There are multiple ways to track the address space. Modern operating systems like Linux and Windows use AVL trees, but you can also use whatever data structure you like.

=== Heap Allocator ===

You will certainly want a [[Heap|heap]] as well, or do you want to keep allocating with 4KB granularity at a time? Start by implementing a very simple (but slow) linked list heap. You can then get to the more complicated designs, like separate block sizes for different buckets, etc. You should also keep in mind that eventually your heap will get out of memory, so you need to implement heap expansion.

Or you might opt for another design which involves a [https://en.wikipedia.org/wiki/Slab_allocation Slab Allocator].

== Scheduler ==

No operating system is real if it cannot schedule tasks. Every modern desktop operating system should allow browsing the web, while rendering a 3D scene, while sorting data in a spreadsheet, while writing a large file to the disk. This is taken care by the [[Scheduler|scheduler]].

=== Multiprocessing ===

Be prepared for [[Multiprocessing|multiprocessing]]. A scheduler that is not ready for multiprocessing will probably be fully rewritten.

=== Priority Levels ===

Design the scheduler in a way, so threads can have different priority levels.

=== Thread Lists ===

It is generally recommended there are different lists of threads for each state and priority level. This way, the scheduling code does not have to look through every thread for a high priority one, then possibly fail to find one, then iterate the thread list again to find a somewhat lower priority thread, then possibly fail again, etc. This means the scheduler code runs faster, since the lack of a thread of a specific priority level can be detected immediately, while also not iterating through inactive threads.
<!--
=== Algorithm ===

For the scheduling algorithm you would probably like having:
* A thread state for each thread (saying if it's "running", "ready to run", or blocked for some reason).
* A "switch_to(thread)" function that does nothing if the thread being switched to is currently running or, if the thread is not currently running:
** Saves the currently running thread's register contents, etc.
** Checks if the currently running thread's state is "running" and if it is, puts the currently running thread back into the "ready to run" state, and puts it on whatever data structure the scheduler uses to keep track of "ready to run" threads.
** Sets the state of the thread being switched to to "running", and loads its register contents, etc.
* A "find_thread_to_switch_to()" function that chooses a thread to switch to (using whatever data structure the scheduler uses to keep track of "ready to run" threads), removes the selected thread from the scheduler's data structure, and then calls the "switch_to(thread)" function.
* A "block_thread(reason)" function which sets the currently running thread's state to whatever the reason is, then calls the "find_thread_to_switch_to()" function.
* An "unblock_thread(thread)" function which sets a thread's state to "read to run", then decides if the thread being unblocked should preempt the currently running thread. If the thread being unblocked shouldn't preempt then it puts the thread on whatever data structure the scheduler uses to keep track of "ready to run" threads; and if the thread being unblocked should preempt it calls the "switch_to(thread)" function instead (to cause an immediate task switch, without bothering with the unnecessary overhead of the "find_thread_to_switch_to()" function).
-->

== Conclusion ==

Operating system development is not easy and is not hard. It's pretty hard. The above (incomplete) list is nothing compared to the complexity involved in a mature operating system.

[[Category:Tutorials]]
[[Category:X86]]
````

<a id="w003921"></a>

## W003921 — Talk:Page fault

Bron-ID: W003921
Namespace: Talk (1)
Revisie: 19314 | Datum: 2016-06-02T17:47:55Z
Bron: https://wiki.osdev.org/Talk:Page_fault
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APage_fault&oldid=19314
Doorverwijzing: Talk:Page Fault

````text
#REDIRECT [[Talk:Page Fault]]
````

<a id="w003922"></a>

## W003922 — User:Glauxosdever/Going Further on x86

Bron-ID: W003922
Namespace: User (2)
Revisie: 19333 | Datum: 2016-06-04T18:34:56Z
Bron: https://wiki.osdev.org/User:Glauxosdever/Going_Further_on_x86
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGlauxosdever%2FGoing_Further_on_x86&oldid=19333
Doorverwijzing: Going Further on x86

````text
#REDIRECT [[Going Further on x86]]
````

<a id="w003923"></a>

## W003923 — User:Snasim2002

Bron-ID: W003923
Namespace: User (2)
Revisie: 19337 | Datum: 2016-06-05T12:24:01Z
Bron: https://wiki.osdev.org/User:Snasim2002
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASnasim2002&oldid=19337

````text
Hi, I'm Syed Nasim. And I have built quite a few operating systems; in addition to a C library that can be linked with kernels...

== LibFalcon ==

[https://github.com/warmachine2002/LibFalcon LibFalcon] is a dead simple C library for linking into kernels. It is far from complete, and does not follow any standards. Almost all essential functions required by kernels are provided. (memcpy, strlen, atoi, etc.) However, there are No math functions at this moment. The reason is that I am in 9th standard, and I do not have enough knowledge about math functions. So, if you are creating a not-so-complex kernel, you can give it a try ;-). (Just don't think it will work like magic, it is not well-tested!) Also the license is BSD, so it gives you more freedom than GPL and friends.

PS: Also, if you are interested in it, you could help me make it better!

== Forum Thread ==
[http://forum.osdev.org/viewtopic.php?f=13&p=261139 Testers wanted for LibFalcon]
````

<a id="w003924"></a>

## W003924 — User:Glauxosdever/TODO

Bron-ID: W003924
Namespace: User (2)
Revisie: 19804 | Datum: 2016-09-08T10:09:56Z
Bron: https://wiki.osdev.org/User:Glauxosdever/TODO
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGlauxosdever%2FTODO&oldid=19804

````text
==Missing==

* [[Printing]] - We need a page describing how to interface with printers so you can print text and images.
* [[Scanning]] - We need a page describing how to interface with printers to obtain scanned documents.
* [[Fax]] - We need a page describing how to interface with fax machines, so you can send faxes.
* Tutorial documenting booting from EFI without gnu-efi.

==Incomplete==

* [[CPU_Registers_x86-64]]
* [[Detecting_Hardware]]
* [[User:Glauxosdever/Higher Half x86 Bare Bones]]
* Pages describing [[ACPI]] tables.
````

<a id="w003925"></a>

## W003925 — LPT

Bron-ID: W003925
Namespace: Main (0)
Revisie: 19348 | Datum: 2016-06-11T20:46:58Z
Bron: https://wiki.osdev.org/LPT
Revisie-URL: https://wiki.osdev.org/index.php?title=LPT&oldid=19348
Doorverwijzing: Parallel port

````text
#REDIRECT[[parallel port]]
````

<a id="w003926"></a>

## W003926 — Smaller C

Bron-ID: W003926
Namespace: Main (0)
Revisie: 29901 | Datum: 2026-01-17T16:52:50Z
Bron: https://wiki.osdev.org/Smaller_C
Revisie-URL: https://wiki.osdev.org/index.php?title=Smaller_C&oldid=29901

````text
Smaller C is a simple and small single-pass C compiler, currently supporting most of the C language common between C89/ANSI C and C99 (minus some C89 and plus some C99 features). It generates code for the [[x86]] and [[MIPS]] architectures.

The compiler self-hosts on DOS, Windows, Linux and Mac OS X meaning it can compile under and for all four OSes and it may additionally be used to write bootloaders and other parts OS running in [[Real Mode]], [[Virtual 8086 Mode]], [[Unreal Mode]] and 32-bit [[Protected Mode]].

With some work it should be possible to run Smaller C on any OS, starting by porting [[NASM]], [[YASM]] or [[FASM]] (FASM is probably easier) to the OS and then making appropriate changes in the Smaller C standard library to use that OS native system calls instead of those for DOS, Windows, Linux or Mac OS X.

The core compiler comes with a preprocessor, a linker and a compiler driver (the driver invokes the preprocessor, the core compiler, the assembler, and the linker and supports options similar to those of [[GCC]]). The linker ([[smlrl]]) can produce the following executable formats: [[COM]], [[MZ]], [[PE]], [[ELF]], [[Mach-O]], [[a.out]] and flat executables as well. The produced [[COM]] and [[MZ]] executables may be booted directly with [[BootProg]].

The standard C library is work-in-progress and it's close to completion.

Smaller C can run on [http://retrobsd.org/wiki/doku.php RetroBSD] on a [[MIPS Overview|MIPS]] microcontroller, fitting into the 96KB of user memory.

==See Also==
*[https://wiki.osdev.org/Unreal_Mode#Smaller_C Unreal mode - Compiler Support - Smaller C]
*[[BootProg]]
===External Links===
*[https://github.com/alexfru/SmallerC Smaller C project]
*[https://github.com/alexfru/SmallerC/wiki Smaller C documentation]
*[https://hackaday.io/project/5569-smaller-c Discuss Smaller C]
*[https://github.com/fysnet/FYSOS/tree/master/loader FYSOS bootloader written for/with Smaller C]

[[Category:C]]
[[Category:Compilers]]
````

<a id="w003927"></a>

## W003927 — Smlrl

Bron-ID: W003927
Namespace: Main (0)
Revisie: 29900 | Datum: 2026-01-17T16:50:44Z
Bron: https://wiki.osdev.org/Smlrl
Revisie-URL: https://wiki.osdev.org/index.php?title=Smlrl&oldid=29900

````text
'''The Smaller C Linker''' (smlrl) comes with the [[Smaller C]] compiler.

== About ==
=== Advantages ===
* Free software (BSD 2-Clause License)
* Supported input formats: [[ELF]] (for both 16-bit and 32-bit code)
* Supported output formats: [[COM]], [[MZ]], [[PE]], [[ELF]], [[Mach-O]], [[a.out]], flat binary
* Can set the code address
* Supports .a (ELF) library files

=== Disadvantages ===
* No 64-bit support
* Produces non-relocatable [[ELF]] and [[MACH-O]] for a given or default address (however, [[PE]] and [[a.out]] are relocatable)
* May not work with ELF files produced by anything other than [[NASM]], [[YASM]], [[FASM]]
* Strips debugging information
* Can't create [[Shared library|shared libraries]]

== See Also ==
=== External ===
*[https://github.com/alexfru/SmallerC Smaller C project]
*[https://github.com/alexfru/SmallerC/wiki Smaller C documentation]
*[https://hackaday.io/project/5569-smaller-c Discuss Smaller C]

[[Category:Linkers]]
````

<a id="w003928"></a>

## W003928 — User:Alexfru

Bron-ID: W003928
Namespace: User (2)
Revisie: 19358 | Datum: 2016-06-12T23:38:51Z
Bron: https://wiki.osdev.org/User:Alexfru
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAlexfru&oldid=19358

````text
Author of [[BootProg]] and [[Smaller C]].
````

<a id="w003929"></a>

## W003929 — Multiboot 1 Higher Half x86 Bare Bones

Bron-ID: W003929
Namespace: Main (0)
Revisie: 30681 | Datum: 2026-06-03T11:13:04Z
Bron: https://wiki.osdev.org/Multiboot_1_Higher_Half_x86_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Multiboot_1_Higher_Half_x86_Bare_Bones&oldid=30681

````text
{{Template:Historical}}
{{Rating|2}}
This tutorial explains how to write a basic 32 bit x86 [[Higher Half Kernel]] using the [[Multiboot|Multiboot 1]] protocol. It can be used to as a base for building an operating system targeting obsolete (Intel 386 era) hardware.

Due to the limitations of the Multiboot 1, using a different protocol (at least Multiboot 2, or some other [[Bootloaders|bootloader]]), and also targeting [[x86-64]] should be considered when building an operating system for modern hardware.

== Theory ==

A higher half kernel is a kernel that is mapped to the higher half of the virtual address space, despite being loaded to the lower half of the physical address space. In order to understand the concept, it is recommended that you have a firm grasp of theory at [[Bare Bones | bare bones]] and [[Paging | paging]].

When GRUB passes control to the kernel, paging is disabled. This means that physical and virtual addresses correspond to the same addresses.

When writing a higher-half kernel, the steps required are:
* Reserve some pages for the initial mappings, so it is possible to parse GRUB structures before memory management is enabled.
* Create page tables that contain the page frames of the kernel image.
* Create the boot page directory that contains the aforementioned page tables both in lower half and higher half.
* Enable paging.
* Jump to higher half.
* Remove the lower half kernel mapping.

== Code ==

Here is some sample code for a kernel that is loaded by [[GRUB | the GRUB bootloader]] and then mapped in the upper half of virtual memory. In this case the kernel is loaded at physical address 0x00100000 and mapped to virtual address 0xC0100000.

=== Assumptions ===

This example makes many assumptions:
* It assumes the kernel is smaller than 3 MiB (reserved is only one boot page table).
* It assumes the developer wants to map the kernel at 0xC0000000.
* It assumes every ELF section is both readable and writable (which is not the case for <tt>.text</tt> and <tt>.rodata</tt>)

=== boot.s ===

After GRUB passes control to the kernel, this piece of code is executed, starting from _start. This essentially bootstraps the kernel by performing the aforementioned steps required and calling <tt>main()</tt>.

<syntaxhighlight lang="asm">
# Declare constants for the multiboot header.
.set ALIGN,    1<<0             # align loaded modules on page boundaries
.set MEMINFO,  1<<1             # provide memory map
.set FLAGS,    ALIGN | MEMINFO  # this is the Multiboot 'flag' field
.set MAGIC,    0x1BADB002       # 'magic number' lets bootloader find the header
.set CHECKSUM, -(MAGIC + FLAGS) # checksum of above, to prove we are multiboot

# Declare a multiboot header that marks the program as a kernel.
.section .multiboot.data, "aw"
.align 4
.long MAGIC
.long FLAGS
.long CHECKSUM

# Allocate the initial stack.
.section .bootstrap_stack, "aw", @nobits
stack_bottom:
.skip 16384 # 16 KiB
stack_top:

# Preallocate pages used for paging. Don't hard-code addresses and assume they
# are available, as the bootloader might have loaded its multiboot structures or
# modules there. This lets the bootloader know it must avoid the addresses.
.section .bss, "aw", @nobits
	.align 4096
boot_page_directory:
	.skip 4096
boot_page_table1:
	.skip 4096
# Further page tables may be required if the kernel grows beyond 3 MiB.

# The kernel entry point.
.section .multiboot.text, "a"
.global _start
.type _start, @function
_start:
	# Physical address of boot_page_table1.
	# TODO: I recall seeing some assembly that used a macro to do the
	#       conversions to and from physical. Maybe this should be done in this
	#       code as well?
	movl $(boot_page_table1 - 0xC0000000), %edi
	# First address to map is address 0.
	# TODO: Start at the first kernel page instead. Alternatively map the first
	#       1 MiB as it can be generally useful, and there's no need to
	#       specially map the VGA buffer.
	movl $0, %esi
	# Map 1023 pages. The 1024th will be the VGA text buffer.
	movl $1023, %ecx

1:
	# Only map the kernel.
	cmpl $_kernel_start, %esi
	jl 2f
	cmpl $(_kernel_end - 0xC0000000), %esi
	jge 3f

	# Map physical address as "present, writable". Note that this maps
	# .text and .rodata as writable. Mind security and map them as non-writable.
	movl %esi, %edx
	orl $0x003, %edx
	movl %edx, (%edi)

2:
	# Size of page is 4096 bytes.
	addl $4096, %esi
	# Size of entries in boot_page_table1 is 4 bytes.
	addl $4, %edi
	# Loop to the next entry if we haven't finished.
	loop 1b

3:
	# Map VGA video memory to 0xC03FF000 as "present, writable".
	movl $(0x000B8000 | 0x003), boot_page_table1 - 0xC0000000 + 1023 * 4

	# The page table is used at both page directory entry 0 (virtually from 0x0
	# to 0x3FFFFF) (thus identity mapping the kernel) and page directory entry
	# 768 (virtually from 0xC0000000 to 0xC03FFFFF) (thus mapping it in the
	# higher half). The kernel is identity mapped because enabling paging does
	# not change the next instruction, which continues to be physical. The CPU
	# would instead page fault if there was no identity mapping.

	# Map the page table to both virtual addresses 0x00000000 and 0xC0000000.
	movl $(boot_page_table1 - 0xC0000000 + 0x003), boot_page_directory - 0xC0000000 + 0
	movl $(boot_page_table1 - 0xC0000000 + 0x003), boot_page_directory - 0xC0000000 + 768 * 4

	# Set cr3 to the address of the boot_page_directory.
	movl $(boot_page_directory - 0xC0000000), %ecx
	movl %ecx, %cr3

	# Enable paging and the write-protect bit.
	movl %cr0, %ecx
	orl $0x80010000, %ecx
	movl %ecx, %cr0

	# Jump to higher half with an absolute jump. 
	lea 4f, %ecx
	jmp *%ecx

.section .text

4:
	# At this point, paging is fully set up and enabled.

	# Unmap the identity mapping as it is now unnecessary. 
	movl $0, boot_page_directory + 0

	# Reload crc3 to force a TLB flush so the changes to take effect.
	movl %cr3, %ecx
	movl %ecx, %cr3

	# Set up the stack.
	mov $stack_top, %esp

	# Enter the high-level kernel.
	call kernel_main

	# Infinite loop if the system has nothing more to do.
	cli
1:	hlt
	jmp 1b
</syntaxhighlight>

=== linker.ld ===

This is a little trickier than it was for the [[Bare Bones | bare bones tutorial]], since you need to distinguish between virtual addresses (which will be in the higher half) and physical addresses (which GRUB needs to decide where to put your kernel).

<pre>
ENTRY (_start)

SECTIONS
{
        . = 0x00100000;
	/* The kernel will live at 3GB + 1MB in the virtual address space, */
	/* which will be mapped to 1MB in the physical address space. */
	/* Note that we page-align the sections. */

	_kernel_start = .;
        .multiboot.data : {
            *(.multiboot.data)
        }

       .multiboot.text : {
           *(.multiboot.text)
       }

	. += 0xC0000000;
	/* Add a symbol that indicates the start address of the kernel. */
	.text ALIGN (4K) : AT (ADDR (.text) - 0xC0000000)
	{
		*(.text)
	}
	.rodata ALIGN (4K) : AT (ADDR (.rodata) - 0xC0000000)
	{
		*(.rodata)
	}
	.data ALIGN (4K) : AT (ADDR (.data) - 0xC0000000)
	{
		*(.data)
	}
	.bss ALIGN (4K) : AT (ADDR (.bss) - 0xC0000000)
	{
		*(COMMON)
		*(.bss)
		*(.bootstrap_stack)
	}
	/* Add a symbol that indicates the end address of the kernel. */
	_kernel_end = .;
}
</pre>

=== kernel.c ===

Use [[Bare_Bones#Writing_a_kernel_in_C|kernel.c from Bare Bones]] with the following change. In boot.s we reserved the 1024th page for the VGA text buffer. This corresponds to virtual address 0xC03FF000. Change the initialization of <tt>terminal_buffer</tt> to:
<syntaxhighlight lang="c">
	terminal_buffer = (uint16_t*) 0xC03FF000;
</syntaxhighlight>

[[Category:Kernel]]
[[Category:Bare bones tutorials|Higher Half bare bones]]
[[Category:Higher Half Barebones]]
````

<a id="w003930"></a>

## W003930 — User:Combuster/Java Primer

Bron-ID: W003930
Namespace: User (2)
Revisie: 19397 | Datum: 2016-07-02T21:17:35Z
Bron: https://wiki.osdev.org/User:Combuster/Java_Primer
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FJava_Primer&oldid=19397
Doorverwijzing: Java Primer

````text
#REDIRECT [[Java Primer]]
````

<a id="w003931"></a>

## W003931 — File:Vga mode.gif

Bron-ID: W003931
Namespace: File (6)
Revisie: 19409 | Datum: 2016-07-08T19:25:41Z
Bron: https://wiki.osdev.org/File:Vga_mode.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVga_mode.gif&oldid=19409

````text

````

<a id="w003932"></a>

## W003932 — Talk:Creating A Shell

Bron-ID: W003932
Namespace: Talk (1)
Revisie: 29732 | Datum: 2025-10-25T22:24:10Z
Bron: https://wiki.osdev.org/Talk:Creating_A_Shell
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACreating_A_Shell&oldid=29732

````text
== Consider either deleting this page or improving it ==

Firstly, I want to say that this article is not of the best quality: both technically and stylistically. Specifically, it's not using any proper interpreter techniques, it doesn't even tokenize the input. The "empty" command just looks like a joke, a single space will break its usefulness. I don't understand why such a primitive shell should be described in an article on a wiki about one of the most difficult and complex programming tasks.

On the other hand, a proper article is probably out of place, maybe it would better fit something like a compiler development wiki.

Finally, I see three options: deleting it, moving it to the author's personal wiki page and completely rewriting it with proper practices. What do people think?

--[[User:Roman|Roman]] 16:51, 9 July 2016 (CDT)

== RE: Consider either deleting this page or improving it ==

This article indeed looks like a very simple stub whose code is more than simple. Such simple code is self-explanatory if someone has basic knowledge of programming. We should either remove it or completely rewrite the article; Former, because it has not use at all.
When someone will have the passion to rewrite it, he could explain what a shell does (and don't) in detail (which is absent in this one) and come up with either writing a command line interpreter from scratch or relying on libraries for scripting languages. This is problby closely connected to 'how to port libraries'.
-- [[user:techel|Techel]] 19:34 (CET) 11 July '16

== New draft ==

I've drafted a new version of this page. Maybe someone more familiar with typical ways of reading strings and handling child processes would like to fill in some of the implementation details?

I've tried to make the article as applicable as possible to different kinds of operating systems - maybe I've done too much of this? Should it be more specific to UNIX-like systems?

[[User:Onlyonemac|Onlyonemac]] 03:21, 12 July 2016 (CDT)

I came here from the "Random Page" lottery. My 2¢: shells are very useful but also highly OS- and even purpose-specific.

My first thought after a skim was "this is too UNIX-specific". The introduction mentions it but that doesn't excuse it.

And honestly, if you're competent enough for OSDev then I'm sure you can figure out how to design an appropriate shell. The internet isn't lacking in guides on this topic either.

I'm in favor of deleting it as I see little purpose.

[[User:Demindiro|Demindiro]] ([[User talk:Demindiro|talk]]) 22:24, 25 October 2025 (UTC)
````

<a id="w003933"></a>

## W003933 — OS X

Bron-ID: W003933
Namespace: Main (0)
Revisie: 19416 | Datum: 2016-07-09T22:04:23Z
Bron: https://wiki.osdev.org/OS_X
Revisie-URL: https://wiki.osdev.org/index.php?title=OS_X&oldid=19416
Doorverwijzing: MacOS

````text
#REDIRECT [[MacOS]]
````

<a id="w003934"></a>

## W003934 — User:Tjmonk15

Bron-ID: W003934
Namespace: User (2)
Revisie: 19729 | Datum: 2016-08-30T00:49:48Z
Bron: https://wiki.osdev.org/User:Tjmonk15
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATjmonk15&oldid=19729

````text
== WSL ==
All code is free for anyone else to use in this section. USE AT YOUR OWN RISK!

[[User:Tjmonk15/WSL_Notes]] (WIP)

[[User:Tjmonk15/WSL_Bare_Bones]] (WIP)
````

<a id="w003938"></a>

## W003938 — User:Omarrx024/VESA Tutorial

Bron-ID: W003938
Namespace: User (2)
Revisie: 27508 | Datum: 2022-12-02T14:40:54Z
Bron: https://wiki.osdev.org/User:Omarrx024/VESA_Tutorial
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AOmarrx024%2FVESA_Tutorial&oldid=27508

````text
'''VESA Tutorial'''

Hello. Today, I'm posting specifically to teach people who want graphics all about VESA. That means that in less than 30 minutes, you'll be able to have a high-color graphics world in your OS!
So anyway, VESA is the Video Electronics Standard Association. They are the people who made the VBE, or the VESA BIOS Extensions Standard. Like the name implies, it is an optional BIOS extension. But I think it's safe to say that 99% of PCs built since the late 1990's have VESA support. Note that UEFI GOP has succeeded VESA BIOS Extensions, but UEFI implementations also have a legacy VESA-compaitble BIOS for backwards-compatibility.

Anyway let's have a little history lesson...
In 1991, VESA released VBE version 1.2, which was mostly compatible with the previous 1.0 and 1.1 but had more popularity. Here, VESA defined some "video mode numbers" which are mostly the same idea as VGA mode numbers (e.g 0x03 is text 80x25, 0x13 is graphics 320x200x8bpp, ...) and they also defined "bank switching." The idea worked like this: you call the VESA BIOS and it sets a video mode for you. Then, because 1991 was still considered "DOS days," you write to 0xA000:0x0000 (linear 0xA0000) to draw to the screen. But that buffer only had a maximum of 64 KB because it is a 16-bit segment... So they invented bank switching; in which the BIOS divided the video memory into smaller chunks called banks, and you'll switch banks as you need to while drawing. For example, in a 640x480x8bpp VESA mode, we need 300 KB of video memory. But unfortunately, we can only access the video memory as 64 KB segments... So if we want to draw to the entire screen, first we switch to bank 0, which is the first 64 KB of the screen, then we draw; then we switch to bank 1, which is the second 64 KB of the screen, then we draw to the same address; then we switch to bank 2, and so on...
This had a severe performance penalty, and bank switching is deprecated today, and has been succeeded by linear framebuffers, which are discussed later in this tutorial.

Here are some modes the VESA defined with VBE 1.x:
<pre>MODE    RESOLUTION  BITS PER PIXEL  MAXIMUM COLORS
0x0100  640x400     8               256
0x0101  640x480     8               256
0x0102  800x600     4               16
0x0103  800x600     8               256
0x010D  320x200     15              32k
0x010E  320x200     16              64k
0x010F  320x200     24/32*          16m
0x0110  640x480     15              32k
0x0111  640x480     16              64k
0x0112  640x480     24/32*          16m
0x0113  800x600     15              32k
0x0114  800x600     16              64k
0x0115  800x600     24/32*          16m
0x0116  1024x768    15              32k
0x0117  1024x768    16              64k
0x0118  1024x768    24/32*          16m</pre>

Notes:
* Some BIOSes support 24-bit color, in which each color is an RGB value, in which each color component is 8 bits; giving a total of 16 million available colors. Other BIOSes support 32-bit color, in which color is also an RGB value and each color component is still 8 bits, but there is an empty 8 bits at the top, which are known as the alpha channel; so 32-bit color is often called RGBA color while 24-bit color is often called RGB color. This is done for two reasons: as a stub for software implementing alpha blending, as to speed up memory operations by 32-bit alignment. Note that the values are in little endian.

Then in 1994, VESA defined VBE 2.0 and this was a major improvement, although "most" VBE 2.0+ BIOSes are compatible with VBE 1.x. Anyway, in VBE 2.0, VESA defined the "linear framebuffer" which was a place in high memory (3 to 4 GB) that had a totally contiguous framebuffer; bank switching was deprecated although is still supported in the BIOS for backwards-compatibility. VESA also stated that all the modes that they defined in VBE 1.x are also deprecated, and that hardware manufacturers didn't need to support them, and that anyone can make up any modes they feel like. Most hardware vendors still support the standard VBE 1.x modes for backward-compatibility, but you should NEVER depend on that, because one day you'll find a PC on which your code just won't work.
But wait a minute... How can we set VESA modes without knowing the mode numbers? Well, VESA gave us two things: a function that returns an array of all available mode numbers, and another function that gets the details of a specified mode number (width, height, bpp, linear framebuffer address, etc...) The basic idea is that we query the BIOS for the information of every available mode, and when we find a mode that fits our needs, we can use it. Since mode numbers are not standard, you should NEVER assume mode numbers, width of a mode, height of a mode, or bpp of the mode. For example, my laptop has VESA mode 0x0118 as 1024x768x32 while older software assumed it to be a 24-bit mode. Newer software may assume it to be a 32-bit mode; while Bochs and QEMU emulate it as a 24-bit mode. Anyway, you should NEVER assume VESA modes and should always query the BIOS for what it supports.

That's enough theory for today, I guess. Let's take a look on how to actually use the VESA BIOS Extensions!
VESA put its VBE functions at function 0x4F of BIOS interrupt 0x10. You put the function number 0x4F in AH register, the subfunction number in AL register, parameters in other registers, and call INT 0x10.
All VESA calls return 0x004F in AX on success. Any other return code should be taken as an error.

Here are some useful functions that can be used with VBE 2.0+:

''' FUNCTION: Get VESA BIOS information '''

Function code: 0x4F00

Description: Returns the VESA BIOS information, including manufacturer, supported modes, available video memory, etc...
'''Input:''' AX = 0x4F00

'''Input:''' ES:DI = Segment:Offset pointer to where to store VESA BIOS information structure.

'''Output:''' AX = 0x004F on success, other values indicate that VESA BIOS is not supported.

Anyway, the above function returns the following structure and stores it in ES:DI as they were on entry. On entry, ES:DI should contain a pointer to the following structure:
<source lang="asm">vbe_info_structure:
	.signature		db "VBE2"	; indicate support for VBE 2.0+
	.table_data:		resb 512-4	; reserve space for the table below</source>
After the BIOS call, if it succeeded (AX is 0x004F), then the same structure above now contains the following:
<source lang="c">struct vbe_info_structure {
	char[4] signature = "VESA";	// must be "VESA" to indicate valid VBE support
	uint16_t version;			// VBE version; high byte is major version, low byte is minor version
	uint32_t oem;			// segment:offset pointer to OEM
	uint32_t capabilities;		// bitfield that describes card capabilities
	uint32_t video_modes;		// segment:offset pointer to list of supported video modes
	uint16_t video_memory;		// amount of video memory in 64KB blocks
	uint16_t software_rev;		// software revision
	uint32_t vendor;			// segment:offset to card vendor string
	uint32_t product_name;		// segment:offset to card model name
	uint32_t product_rev;		// segment:offset pointer to product revision
	char reserved[222];		// reserved for future expansion
	char oem_data[256];		// OEM BIOSes store their strings in this area
} __attribute__ ((packed));</source>
Notice that all segment:offset fields are in little-endian, which means the low word is the offset, and the high word is the segment.
Things that might be of interest to you from the above structure: "signature" will be changed from "VBE2" to "VESA". It must be "VBE2" on entry to indicate software support for VBE 2.0. If it contains "VBE2", the BIOS will return the 512 bytes of data for VBE 2.0+. If it contains "VESA", the BIOS will return 256 bytes of data for VBE 1.x. If it is not "VESA" after the call, you should assume that VESA BIOS Extensions are not available. "version" tells you the version of VBE; 0x100 is 1.0, 0x101 is 1.1, 0x102 is 1.2, 0x200 is 2.0, and 0x300 is 3.0 (the latest version). VBE 1.x returns 256 bytes of data in the above structure, VBE 2.0 and 3.0 return 512 bytes of data if the "signature" field contained "VBE2" on entry. "video_modes" is a segment:offset pointer to the list of supported video modes. Each entry in the array is a 16-bit word, and is terminated by a 0xFFFF. If while searching for your mode, you find a 0xFFFF, then the mode is not supported. "video_memory" contains how much VGA RAM the PC has in 64 KB chunks. So, to have it in KB, multiply the value in "video_memory" by 64.
Anyway, about the supported modes array, if the PC supports modes 0x0103, 0x0115, and 0x0118, the array would look like this in a hexdump:
<pre>03 01 15 01 18 01 FF FF</pre>
Notice how it is terminated by a 0xFFFF and all values are in little-endian.

''' FUNCTION: Get VESA mode information '''

Function code: 0x4F01

Description: This function returns the mode information structure for a specified mode. The mode number should be gotten from the supported modes array.

'''Input:''' AX = 0x4F01

'''Input:''' CX = VESA mode number from the video modes array

'''Input:''' ES:DI = Segment:Offset pointer of where to store the VESA Mode Information Structure shown below.

'''Output:''' AX = 0x004F on success, other values indicate a BIOS error or a mode-not-supported error.

Here's the structure returned by this function in ES:DI:
<source lang="c">struct vbe_mode_info_structure {
	uint16_t attributes;		// deprecated, only bit 7 should be of interest to you, and it indicates the mode supports a linear frame buffer.
	uint8_t window_a;			// deprecated
	uint8_t window_b;			// deprecated
	uint16_t granularity;		// deprecated; used while calculating bank numbers
	uint16_t window_size;
	uint16_t segment_a;
	uint16_t segment_b;
	uint32_t win_func_ptr;		// deprecated; used to switch banks from protected mode without returning to real mode
	uint16_t pitch;			// number of bytes per horizontal line
	uint16_t width;			// width in pixels
	uint16_t height;			// height in pixels
	uint8_t w_char;			// unused...
	uint8_t y_char;			// ...
	uint8_t planes;
	uint8_t bpp;			// bits per pixel in this mode
	uint8_t banks;			// deprecated; total number of banks in this mode
	uint8_t memory_model;
	uint8_t bank_size;		// deprecated; size of a bank, almost always 64 KB but may be 16 KB...
	uint8_t image_pages;
	uint8_t reserved0;

	uint8_t red_mask;
	uint8_t red_position;
	uint8_t green_mask;
	uint8_t green_position;
	uint8_t blue_mask;
	uint8_t blue_position;
	uint8_t reserved_mask;
	uint8_t reserved_position;
	uint8_t direct_color_attributes;

	uint32_t framebuffer;		// physical address of the linear frame buffer; write here to draw to the screen
	uint32_t off_screen_mem_off;
	uint16_t off_screen_mem_size;	// size of memory in the framebuffer but not being displayed on the screen
	uint8_t reserved1[206];
} __attribute__ ((packed));</source>

Lots of useless sh*t, I know. The only things that interest us: "attributes" bit 7 (value 0x80) indicates the mode supports a linear frame buffer. "width", "height" are "bpp" are used while searching for the mode we want to use. "framebuffer" is the 32-bit physical pointer to the linear framebuffer. The linear framebuffer must be enabled while setting the VBE mode, which is discussed in the next function. If you are using paging, be sure to map the framebuffer somewhere known in the virtual address space!

''' FUNCTION: Set VBE mode '''

Function code: 0x4F02

Description: This function sets a VBE mode.

'''Input:''' AX = 0x4F02

'''Input:''' BX = Bits 0-13 mode number; bit 14 is the LFB bit: when set, it enables the linear framebuffer, when clear, software must use bank switching. Bit 15 is the DM bit: when set, the BIOS doesn't clear the screen. Bit 15 is usually ignored and should always be cleared.

'''Output:''' AX = 0x004F on success, other values indicate errors; such as BIOS error, too little video memory, unsupported VBE mode, mode doesn't support linear frame buffer, or any other error.

So that means, if VBE mode 0x0118 is 1024x768x32bpp, and we wanted to set this mode and ask the BIOS to clear the screen for us, we can do this:
<source lang="asm">mov ax, 0x4F02	; set VBE mode
mov bx, 0x4118	; VBE mode number; notice that bits 0-13 contain the mode number and bit 14 (LFB) is set and bit 15 (DM) is clear.
int 0x10			; call VBE BIOS
cmp ax, 0x004F	; test for error
jne error

after:
	; ...</source>
Anyway, like I mentioned at least a hundred times, you should first get the mode number from the video modes array. Those mode numbers only have the plain mode numbers (0x0118, 0x0103, etc...) and you should set bit 14 when you set the VBE mode.

''' FUNCTION: Get current VBE mode '''

Function code: 0x4F03

Description: This function returns the current VBE mode.

'''Input:''' AX = 0x4F03

'''Output:''' AX = 0x004F on success, other values indicate errors; maybe the system is not in a VBE mode?

'''Output:''' BX = Bits 0-13 mode number; bit 14 is the LFB bit: when set, the system is using the linear framebuffer, when clear, the system is using bank switching. Bit 15 is the DM bit: when clear, video memory was cleared when the VBE mode was set.

''' FUNCTION: Display Window Control (deprecated) '''

Function code: 0x4F05

Description: Sets/Gets the current bank (deprecated)

'''Input:''' AX = 0x4F05

'''Input:''' BH = 0x00 to set bank, 0x01 to get bank

'''Input:''' BL = 0x00 for window A, 0x01 for window B

'''Input:''' DX = Bank number in window granularity units

'''Output:''' AX = 0x004F on success, other values indicate errors; such as BIOS errors, unusable bank, unpresent bank, not using banked mode...

Because bank switching is deprecated, I will not go into how to calculate bank numbers.

''' FUNCTION: Return protected mode interface '''

Function code: 0x4F0A

Description: Returns the VBE 2.0 protected mode interface

'''Input:''' AX = 0x4F0A

'''Input:''' BL = 0x00

'''Output:''' AX = 0x004F on success, other values indicate errors; such as BIOS errors, protected mode interface not supported, or VBE version is less than 2.0

'''Output:''' ES:DI = Segment:Offset pointer to protected mode table

'''Output:''' CX = Length of table including code in bytes for copying purposes

This function allows software to switch banks (function 0x4F05), set display start (function 0x4F07) and set primary pallette data  (function 0x4F09) from protected mode, without needing to return to real mode. This increases performance somewhat, but is overall deprecated and may not be supported in some BIOSes.
Anyway, the table returned by ES:DI looks like this:

<source lang="c">struct vbe2_pmi_table {
	uint16_t set_window;		// offset in table for protected mode code for function 0x4F05
	uint16_t set_display_start;	// offset in table for protected mode code for function 0x4F07
	uint16_t set_pallette;		// offset in table for protected mode code for function 0x4F09
} __attribute__ ((packed));</source>
That means, to set the bank from protected mode, one should first find the linear address of the set_window function, by doing ES * 0x10 + DI + [ES:DI] and then doing a near CALL, because all protected mode functions end with a near RET. Useless, I know, but perhaps some people want to use bank switching as a fallback when VBE 2.0 is not available, or when the specific mode doesn't support a linear frame buffer.

By now, your understanding of VESA should be quite clear and your implementation of VESA shouldn't be that difficult. Note, that if you use v8086 to call VESA BIOS, many video cards have memory-mapped register in high memory (3-4 GB) and the BIOS will have to switch to protected mode transparently to you during VBE execution, which goes beyond limits of v8086. For this problem, it is recommended to configure VESA in plain 16-bit real mode, before 32-bit/64-bit mode.

'''Need code?'''
Here's a function to set a VESA mode directly from a specified width/height/bpp, copied and pasted from my OS code. Notice that it runs in 16-bit real mode with DS = ES = FS = GS = 0.
<source lang="asm">; vbe_set_mode:
; Sets a VESA mode
; In\	AX = Width
; In\	BX = Height
; In\	CL = Bits per pixel
; Out\	FLAGS = Carry clear on success
; Out\	Width, height, bpp, physical buffer, all set in vbe_screen structure

vbe_set_mode:
	mov [.width], ax
	mov [.height], bx
	mov [.bpp], cl

	sti

	push es					; some VESA BIOSes destroy ES, or so I read
	mov ax, 0x4F00				; get VBE BIOS info
	mov di, vbe_info_block
	int 0x10
	pop es

	cmp ax, 0x4F				; BIOS doesn't support VBE?
	jne .error

	mov ax, word[vbe_info_block.video_modes]
	mov [.offset], ax
	mov ax, word[vbe_info_block.video_modes+2]
	mov [.segment], ax

	mov ax, [.segment]
	mov fs, ax
	mov si, [.offset]

.find_mode:
	mov dx, [fs:si]
	add si, 2
	mov [.offset], si
	mov [.mode], dx
	mov ax, 0
	mov fs, ax

	cmp [.mode], 0xFFFF			; end of list?
	je .error

	push es
	mov ax, 0x4F01				; get VBE mode info
	mov cx, [.mode]
	mov di, mode_info_block
	int 0x10
	pop es

	cmp ax, 0x4F
	jne .error

	mov ax, [.width]
	cmp ax, [mode_info_block.width]
	jne .next_mode

	mov ax, [.height]
	cmp ax, [mode_info_block.height]
	jne .next_mode

	mov al, [.bpp]
	cmp al, [mode_info_block.bpp]
	jne .next_mode

	; If we make it here, we've found the correct mode!
	mov ax, [.width]
	mov word[vbe_screen.width], ax
	mov ax, [.height]
	mov word[vbe_screen.height], ax
	mov eax, [mode_info_block.framebuffer]
	mov dword[vbe_screen.physical_buffer], eax
	mov ax, [mode_info_block.pitch]
	mov word[vbe_screen.bytes_per_line], ax
	mov eax, 0
	mov al, [.bpp]
	mov byte[vbe_screen.bpp], al
	shr eax, 3
	mov dword[vbe_screen.bytes_per_pixel], eax

	mov ax, [.width]
	shr ax, 3
	dec ax
	mov word[vbe_screen.x_cur_max], ax

	mov ax, [.height]
	shr ax, 4
	dec ax
	mov word[vbe_screen.y_cur_max], ax

	; Set the mode
	push es
	mov ax, 0x4F02
	mov bx, [.mode]
	or bx, 0x4000			; enable LFB
	mov di, 0			; not sure if some BIOSes need this... anyway it doesn't hurt
	int 0x10
	pop es

	cmp ax, 0x4F
	jne .error

	clc
	ret

.next_mode:
	mov ax, [.segment]
	mov fs, ax
	mov si, [.offset]
	jmp .find_mode

.error:
	stc
	ret

.width				dw 0
.height				dw 0
.bpp				db 0
.segment			dw 0
.offset				dw 0
.mode				dw 0</source>

'''What's next?'''
Well, that's entirely up to you! Just kidding, here's some tips on implementing a graphics library.
Calculating pixel offset:
<source lang="c">uint32 pixel_offset = y * pitch + (x * (bpp/8)) + framebuffer;</source>
Where "pitch", "bpp" and "framebuffer" are gotten from the structure returned by function 0x4F01. Then, you can plot a pixel by writing a value there.
For 32-bit modes, each pixel value is 0x00RRGGBB in little endian, so to plot a red pixel, we would write 0x00FF0000. In 24-bit mode, each pixel is 0xRRGGBB in little endian. Due to the bad memory alignment, performance is relatively low with 24-bit modes. In 16-bit modes, we have a total of 64K colors, and the color has 5 bits of red, 6 bits of green, and 5 bits of blue. There is more green because according to Wikipedia, the human eye is more sensitive to green than red and blue...
In 15-bit modes, we have a total of 32K colors, 5 bits of red, 5 bits of green and 5 bits of blue. The highest bit (bit 15, value 0x8000) is unused.
Because my OS supports 32-bit/24-bit modes and 16-bit as a fallback, I will focus on these. Anyway if you want something less than 16-bit your limiting yourself to the obsolete technology...
Just by getting the pixel's offset and writing there, you've implemented a put_pixel() routine!
Now, we want some more things: lines, squares, alpha blending, and some hot graphics! It may seem easy to implement a fill_rect() routine: just put_pixel() everything. That's going to work, but it's not going to work fast. For a 32*32 rectangle, you'll be calculating the pixel offset 1024 times! Instead, a better approach is to calculate the pixel offset one time, and then (REP STOSD/REP STOSW) for one line. Then you can do (offset = offset + pitch) just to skip one line down. This way, you can fill a 512x512 rectangle yet calculate the pixel offset only one time. By calling put_pixel() for every pixel, you'd be calculating the pixel offset 262144 times! Using the method I just said, you can make performance 262144 times better.
For drawing lines, you'll probably implement an existing algorithm. Wikipedia has a good article on drawing lines.
As for alpha blending, Wikipedia also has good information on that.
Now that you have a linear framebuffer, a basic put_pixel() routine and a fill_rect() routine, you can implement a basic graphics library. Programming graphics now is just the same as non-OS graphics programming. Enjoy!

'''Keep reading!'''  

[http://www.phatcode.net/res/221/files/vbe20.pdf VBE 2.0 Specification]  

[http://wiki.osdev.org/Drawing_In_Protected_Mode#Drawing_Text Drawing Text]  

[http://wiki.osdev.org/Double_Buffering Double Buffering - a way of increasing graphics performance]  

[https://en.wikipedia.org/wiki/Alpha_compositing Alpha blending]
````

<a id="w003939"></a>

## W003939 — GRUB 2

Bron-ID: W003939
Namespace: Main (0)
Revisie: 19452 | Datum: 2016-07-14T15:14:48Z
Bron: https://wiki.osdev.org/GRUB_2
Revisie-URL: https://wiki.osdev.org/index.php?title=GRUB_2&oldid=19452
Doorverwijzing: GRUB

````text
#REDIRECT [[GRUB]]
````

<a id="w003940"></a>

## W003940 — Talk:GRUB 2

Bron-ID: W003940
Namespace: Talk (1)
Revisie: 19454 | Datum: 2016-07-14T15:14:48Z
Bron: https://wiki.osdev.org/Talk:GRUB_2
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGRUB_2&oldid=19454
Doorverwijzing: Talk:GRUB

````text
#REDIRECT [[Talk:GRUB]]
````

<a id="w003941"></a>

## W003941 — Writing GRUB2 Modules

Bron-ID: W003941
Namespace: Main (0)
Revisie: 19471 | Datum: 2016-07-14T21:09:58Z
Bron: https://wiki.osdev.org/Writing_GRUB2_Modules
Revisie-URL: https://wiki.osdev.org/index.php?title=Writing_GRUB2_Modules&oldid=19471
Doorverwijzing: Writing GRUB Modules

````text
#REDIRECT [[Writing GRUB Modules]]
````

<a id="w003942"></a>

## W003942 — User:Jfjhh

Bron-ID: W003942
Namespace: User (2)
Revisie: 19491 | Datum: 2016-07-18T22:09:23Z
Bron: https://wiki.osdev.org/User:Jfjhh
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJfjhh&oldid=19491

````text
Currently working on [http://www.github.com/jfjhh/os-momo OS Momo] (specifically its ''bootloader'', [http://www.github.com/jfjhh/nana Nana]).
````

<a id="w003943"></a>

## W003943 — User talk:Jfjhh

Bron-ID: W003943
Namespace: User talk (3)
Revisie: 19492 | Datum: 2016-07-18T22:33:19Z
Bron: https://wiki.osdev.org/User_talk:Jfjhh
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AJfjhh&oldid=19492

````text
''I am open to any and all feedback.''
````

<a id="w003944"></a>

## W003944 — Higher Half bare bones

Bron-ID: W003944
Namespace: Main (0)
Revisie: 19518 | Datum: 2016-07-22T14:07:48Z
Bron: https://wiki.osdev.org/Higher_Half_bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Higher_Half_bare_bones&oldid=19518
Doorverwijzing: Higher Half x86 Bare Bones

````text
#REDIRECT [[Higher Half x86 Bare Bones]]
````

<a id="w003947"></a>

## W003947 — Higher Half Bare Bones

Bron-ID: W003947
Namespace: Main (0)
Revisie: 30682 | Datum: 2026-06-03T11:15:19Z
Bron: https://wiki.osdev.org/Higher_Half_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Higher_Half_Bare_Bones&oldid=30682
Doorverwijzing: Category:Higher Half Barebones

````text
#REDIRECT [[Category:Higher Half Barebones]]
````

<a id="w003948"></a>

## W003948 — Talk:AS86

Bron-ID: W003948
Namespace: Talk (1)
Revisie: 28271 | Datum: 2023-08-12T11:39:47Z
Bron: https://wiki.osdev.org/Talk:AS86
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AAS86&oldid=28271

````text
== Deletion ==
I am tempted to delete this. When I searched for as86, the first hit was this article. It doesn't have anything to do with GNU as far as I can see. The last release seems to have been in 1997 or 2002 or 2004 or something. There's no website as far as I can track down. The project seems to also contain a ld86. It also goes by the names of bin86 (name of package in debian [https://packages.debian.org/wheezy/bin86]), dev86, or linux86. It seems to have been developed by a Bruce Evans. The project seems pretty obsolete by now and is only of historical interest. I can't quite imagine how this can be useful to anyone osdeving today. A website for this appears to be [http://v3.sk/~lkundrak/dev86/] with timestamps from this decade. Probably a fork. See the #osdev logs for 2016-07-22 for a discussion with a bit more information. --[[User:Sortie|Sortie]] 09:19, 22 July 2016 (CDT)


At least it doesn't show up as the first hit on google for me. Instead, I stumbled on a man page that documents it as once being the assembler for the Linux kernel. I don't want to get rid of it (especially considering its former glory), so I tried turning it more into a big "go away" sign instead. - [[User:Combuster|Combuster]] 01:40, 25 July 2016 (CDT)



Replying to that comment, I actually would like to see this article expanded, to include a description of the internal structure of the object file as86 outputs (and ld86 takes as input). The reason is, NASM actually can output object files in this format if you use the "-f as86" command line switch. The documentation for NASM describes it as a simple format, but provides no description of the actual internal structure for this file format. Since it sounds fairly simple, I'd like to try to write my own loader or linker for this file format, just as an exercise, but I have no idea what the internal structure of the file format is, so I can't really attempt to make such a program at the current time, unfortunately. So it would be very nice if you actually expanded this article to include a description of the internal structure of this object file format. - [[User:Videogamer555|Videogamer555]] 06:39, 12 August 2023 (CDT)
````

<a id="w003949"></a>

## W003949 — Going further on x86

Bron-ID: W003949
Namespace: Main (0)
Revisie: 19534 | Datum: 2016-07-22T20:57:19Z
Bron: https://wiki.osdev.org/Going_further_on_x86
Revisie-URL: https://wiki.osdev.org/index.php?title=Going_further_on_x86&oldid=19534
Doorverwijzing: Going Further on x86

````text
#REDIRECT [[Going Further on x86]]
````

<a id="w003950"></a>

## W003950 — User:New16

Bron-ID: W003950
Namespace: User (2)
Revisie: 26267 | Datum: 2021-08-06T10:43:25Z
Bron: https://wiki.osdev.org/User:New16
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANew16&oldid=26267

````text
[[Opcode Programming]]

Any beginner should start here.

Before starting assembly or high-level language programming you should know: The processor does only understand opcode. Simply consider this languages named above to be helpful, but they must be interpreted by so called compilers, linkers and assemblers to create the necessary opcode.<br>

So just grab an instance of "<b>Intel® 64 and IA-32 Architectures Software Developer’s Manual</b>", with the subtitle: "<b>Combined Volumes: 1, 2A, 2B, 2C, 3A, 3B, 3C and 3D</b>" (4618 pages in June 2016), and go on with this tutorial.<br>
:(Just type the title in the search at "intel.com" .)

==Basics==

You should read at least the following sections (I recommend this order.):

*[[Introduction]]
*[[BIOS]]
*[[Boot Sequence]]
*[[Real Mode]]
*[[Assembly]]

You might want to run the following code in an emulator. For windows user I recommend [[VirtualBox]]: Set up a new machine with "Typ: Other" and "Version: Other/Unknown (64-bit)" and make sure you find a floppy-controller under section "Storage".<br>
You also need a hex-editor (f.e. "https://hexed.it/").<br>
After editing and saveing the file, named: "something.img", you should close the hex-editor (-website).

==First Program==

Type in the hex editor (without the leading zero and the "h" at the end):

<source lang="asm">

0EBh, 0FEh
;now expand this file with zeros until you reached address 510 (01FEh)
0, 0, ...
;at address 510 (01FEh) insert:
0AAh
;at adress 511 (01FFh) insert:
055h
</source>

Add this file to you floppy controller in your emulator (f.e. VirtualBox).<br>
Run your virtual machine.<br>
And you will see: nothing. But you will be able to notice a massive CPU-usage.<br>
It worked! Whatever you have done...<br>
====Explanation====
Lets start from the beginning, 0xEB and 0xFE is a JMP opcode which makes the computer execute this code over and over.<br>
The last 2 bytes are the boot signature, for the computer to understand that this floppy is a bootable floppy disk.
````

<a id="w003951"></a>

## W003951 — Talk:Instruction Set Architecture

Bron-ID: W003951
Namespace: Talk (1)
Revisie: 19574 | Datum: 2016-07-23T21:48:29Z
Bron: https://wiki.osdev.org/Talk:Instruction_Set_Architecture
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AInstruction_Set_Architecture&oldid=19574

````text
CNN ISA as uncommon alternative + analog computer programming might be interesting - I will add this on demand.; Examples? --[[User:New16|New16]] 16:48, 23 July 2016 (CDT)
````

<a id="w003952"></a>

## W003952 — Talk:Model Specific Registers

Bron-ID: W003952
Namespace: Talk (1)
Revisie: 19576 | Datum: 2016-07-23T22:01:39Z
Bron: https://wiki.osdev.org/Talk:Model_Specific_Registers
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AModel_Specific_Registers&oldid=19576

````text
MTRR-part missing --[[User:New16|New16]] 17:01, 23 July 2016 (CDT)
````

<a id="w003955"></a>

## W003955 — Ps2

Bron-ID: W003955
Namespace: Main (0)
Revisie: 19643 | Datum: 2016-08-01T15:07:17Z
Bron: https://wiki.osdev.org/Ps2
Revisie-URL: https://wiki.osdev.org/index.php?title=Ps2&oldid=19643
Doorverwijzing: PS/2

````text
#REDIRECT[[PS/2]]
````

<a id="w003956"></a>

## W003956 — File:Xbox-Console-Open-Drives.jpg

Bron-ID: W003956
Namespace: File (6)
Revisie: 19646 | Datum: 2016-08-03T08:49:56Z
Bron: https://wiki.osdev.org/File:Xbox-Console-Open-Drives.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AXbox-Console-Open-Drives.jpg&oldid=19646

````text

````

<a id="w003957"></a>

## W003957 — Xbox

Bron-ID: W003957
Namespace: Main (0)
Revisie: 23829 | Datum: 2019-08-25T00:01:01Z
Bron: https://wiki.osdev.org/Xbox
Revisie-URL: https://wiki.osdev.org/index.php?title=Xbox&oldid=23829

````text
This article will be an introduction for you, to the wonderful world of Microsoft's Xbox console. I'm going to post more tutorials here at some later date.

== But it already has an OS? ==
An Xbox does not have an OS per se. The Xbox's firmware consists of its NT-based kernel and the 'dashboard', which, as far as the kernel is concerned, is just a fancy type of game.

== Specifications ==
[[File:Xbox-Console-Open-Drives.jpg|thumb|right|The Xbox uses standard computer DVD and hard drives.]]

:{|
|-
!Length:
| approximately 32cm
|-
!Width:
| approximately 10cm
|-
!Height:
| approximately 26cm
|-
!Mass:
| approximately 3.86 kg
|-
!CPU:
| Custom 733 MHz Intel Pentium III "Coppermine-based" processor
|-
!Memory:
| 64 MB of DDR SDRAM @ 200 MHz
|-
!Storage:
| 8 or 10 GB internal Hard Drive (Formatted to 8 GB with allotted system reserve and MS Dash), 8 MB memory card
|-
!Connectivity:
| 10/100 Mbit ethernet
|-
!Resolutions:
| 480i, 480p, 576i, 576p, 720p, 1080i
|}

===Development Unit===
The Xbox Development unit is the version used by game developers. It has twice the RAM of the retail console, and adds SCSI, USB and RS232 ports to help with debugging.
To accommodate these extra ports, an expansion board was added on top of the hard drive mount, making the console about 2cm taller.

== OS Development ==
Potentially, an Xbox can be targeted as an implementation platform for new Operating Systems, or other technological enhancement/tampering. However, such a thing is still in its infant stage.

[[Category:X86]]
[[Category:Xbox]]
````

<a id="w003959"></a>

## W003959 — File:Beelz 3.png

Bron-ID: W003959
Namespace: File (6)
Revisie: 19650 | Datum: 2016-08-03T09:24:02Z
Bron: https://wiki.osdev.org/File:Beelz_3.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABeelz_3.png&oldid=19650

````text
Testing once again.
````

<a id="w003960"></a>

## W003960 — User:Binox

Bron-ID: W003960
Namespace: User (2)
Revisie: 19666 | Datum: 2016-08-11T09:21:03Z
Bron: https://wiki.osdev.org/User:Binox
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABinox&oldid=19666

````text
== About Me ==

Hi there, I'm BinoX (But the wiki didn't want me to have a capital letter at the end of my username. But either way - it is pronounced Buy-Nocks)

I may have made one of the more unpopular choices during OSDeving - I use Windows as my primary development platform. A lot of this is for convenience, as I am an avid gamer and use Steam as my primary gaming platform (as I have done for many years), so my highest spec machine is a Windows machine and I really didn't want to mess around with VMs (there's too much of that at my actual work)

As with most of the people on this wiki, I am writing an Operating System - currently known as BinOS. Work has been slow on this and I don't really have anything impressive to show. But future plans are good and I am slowly working towards my, somewhat unusual, goal (details on that when I actually get at least a bit of it working)

== Current Work ==

I am playing with the Windows Subsystem for Linux (Beta), that was recently released with the Windows 10 Anniversary Update, and evaluating its uses as an OS Development platform. I am then updating the wiki with my findings.

As for OS Dev - I am working on an unusual HAL and also stripping out a lot of functionality from my OS that I have decided goes contrary to my goals (but was useful as a bit of test code, so no time wasted)
````

<a id="w003961"></a>

## W003961 — File:Cromwell Firmware on an Xbox.jpg

Bron-ID: W003961
Namespace: File (6)
Revisie: 19677 | Datum: 2016-08-12T12:49:04Z
Bron: https://wiki.osdev.org/File:Cromwell_Firmware_on_an_Xbox.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACromwell_Firmware_on_an_Xbox.jpg&oldid=19677

````text

````

<a id="w003962"></a>

## W003962 — User:Halofreak1990/Cromwell

Bron-ID: W003962
Namespace: User (2)
Revisie: 19678 | Datum: 2016-08-12T12:54:11Z
Bron: https://wiki.osdev.org/User:Halofreak1990/Cromwell
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AHalofreak1990%2FCromwell&oldid=19678

````text
[[Image:Cromwell Firmware on an Xbox.jpg|thumb|right|222px|Cromwell boot menu on an Xbox]]

'''Cromwell''' is a replacement firmware for the Microsoft Xbox gaming console that has been developed by the Xbox Linux Project. If programmed onto the onboard flash chip of the Xbox or a modchip, it can boot the [[Linux]] operating system and practically convert the Xbox into a full PC.

It is unusual in that it is a legal firmware (because it does not use any of Microsoft's code to function) and was developed primarily through reverse-engineering of the original Microsoft BIOS and its boot process.  In light of this, many Xbox modchip manufacturers ship this firmware with their chips to avoid litigation and copyright infringement claims.  The main function of this firmware is to load the Linux operating system, although it also supports other features, such as the ability to reprogram the hardware with another firmware image, lock and unlock the hard disk, and change video mode (PAL/NTSC). Unlike the vast majority of Xbox firmware images, it is not able to load Xbox games (either original or otherwise).  This is because the original Microsoft firmware image contains the kernel of the Xbox operating system (widely believed to be a stripped-down Windows 2000 derivative) - that is, the firmware '''is''' the operating system.  As Cromwell does not contain this, it is not able to allow games to load and function.

Cromwell includes code from other open source projects, but combines them in a unique way. This is a list of components that can be found in Cromwell:
* a Linux 2.6-derived USB stack in a standalone version (i.e. it runs without Linux)
* Linux kernel-derived JPEG decompression code
* [[Etherboot]] networking code
* [[GRUB]] filesystem support and bootloader code

===XbeBoot===
XbeBoot is a version of Cromwell packaged as an Xbox Executable (xbe), allowing it to be booted from a 3rd party dashboard as an application.

==External links==
* [http://www.xbox-linux.org/wiki/Cromwell_Manual Cromwell Manual]

[[Category:Xbox]]
[[Category:Bootloaders]]
[[Category:x86]]
````

<a id="w003963"></a>

## W003963 — User talk:Max/Preparing Windows as a build environment

Bron-ID: W003963
Namespace: User talk (3)
Revisie: 19684 | Datum: 2016-08-13T16:21:56Z
Bron: https://wiki.osdev.org/User_talk:Max/Preparing_Windows_as_a_build_environment
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AMax%2FPreparing_Windows_as_a_build_environment&oldid=19684

````text
I think that this need to be putted to wiki, not like user page.
It's big enough to do that.--[[User:Lukand|Lukand]] 11:21, 13 August 2016 (CDT)
````

<a id="w003964"></a>

## W003964 — DPL

Bron-ID: W003964
Namespace: Main (0)
Revisie: 28238 | Datum: 2023-07-10T22:05:40Z
Bron: https://wiki.osdev.org/DPL
Revisie-URL: https://wiki.osdev.org/index.php?title=DPL&oldid=28238

````text
{{Stub}}
DPL is the Descriptor Privilege Level. This is sometimes known as the "Ring" level.

[[Category:X86]]
[[Category:Security]]
````

<a id="w003965"></a>

## W003965 — User:Johnburger/Dingle

Bron-ID: W003965
Namespace: User (2)
Revisie: 21344 | Datum: 2017-08-28T20:54:10Z
Bron: https://wiki.osdev.org/User:Johnburger/Dingle
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDingle&oldid=21344

````text
= Dingle(tm)  =
There are some times when you just want to know that your code worked. And then there are others where, if there's an error, it tells you in '''''NO''''' uncertain terms - but waits.

For these reasons I invented the Dingle(tm). It dingles away in some verifiable area of the screen to show you that it has reached this area - and often, that it won't leave!

The code is easy: increment one specific location on the screen for the user. That will cycle through all the different ASCII codes for that location; and it doesn't matter whether you're using a Virtual Machine or Real Hardware: the location will update!
````

<a id="w003966"></a>

## W003966 — User:Lukand

Bron-ID: W003966
Namespace: User (2)
Revisie: 20075 | Datum: 2016-12-31T15:29:28Z
Bron: https://wiki.osdev.org/User:Lukand
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALukand&oldid=20075

````text
Hi, I'm '''Luka Andjelkovic''', developer of TT-NOS operating system (Nutoak).
Website : [http://nutoak.netau.net Thontelix-free_software].

----

[http://internetometer.com/give/46047 http://internetometer.com/image/46047.png]
== About ==
I am an 11-year old Serb.
I'm interested in remembering how was I stupid few months ago and regret for that. At same time I'm highly optimistic because I have quickly became smarter since then.

I think that this is enough for you to know.
````

<a id="w003967"></a>

## W003967 — User:Tjmonk15/WSL Notes

Bron-ID: W003967
Namespace: User (2)
Revisie: 21661 | Datum: 2017-10-28T02:14:33Z
Bron: https://wiki.osdev.org/User:Tjmonk15/WSL_Notes
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATjmonk15%2FWSL_Notes&oldid=21661

````text

== Random Notes ==

* 100% working on my end for Bare Bone and Meaty Skeleton
* WSL is running Ubuntu 14.04
* WSL does not have an install of GRUB (use grub-pc-bin)
* WSL has a rather out of date version of GCC install by default (4.8.2)
* To build GCC need bison, flex, byacc, as well as the prereqs mentioned on [[GCC_Cross-Compiler#Downloading_the_Source_Code]]
* Need to update pre-built gcc, [http://askubuntu.com/a/581497/353923 Server Fault - How do I use the latest GCC on Ubuntu?] use gcc-6 instead of gcc-4.9
* Recommended list of packages to apt-get before starting
** git
** bison
** flex
** byacc
** build-essential
** gcc-7 (After adding PPA: sudo add-apt-repository ppa:ubuntu-toolchain-r/test)
** libgmp3-dev
** libmpfr-dev
** libmpc-dev
** texinfo
** grub-pc-bin
** xorriso
* Need to come up with steps and test all of the above to make sure it is enough.
* Optional/Helpful packages to apt-get
** genisoimage (provides "isoinfo" which can be helpful for sanity checking iso files)
** nasm (if you prefer Intel syntax over AT&T)
* [[Bare_Bones]] does work with all prereqs mentioned above. Running the kernel (with -kernel) or the iso (with -cdrom) in QEMU works (confirmed by myself +2 on the forums)
* [[Meaty_Skeleton]] works same as the Bare Bones (works for me in qemu, maybe others, though unconfirmed by myself)
* For cleanup purposes, [http://askubuntu.com/questions/17823/how-to-list-all-installed-packages]This is a good way to see all "installed" packages. Adding a "> file.txt" after it will allow you to easily reference and review the list to decide what needs to be removed. And "sudo apt-get autoremove" should be your final command (repeated until nothing shows up if necessary)

<pre>
[19:52] <Kazinsal> should add something about installing a Windows-based X server like Xming
[19:52] <Kazinsal> you can then set up your bash startup scripts to export DISPLAY=0:0
[19:52] <Kazinsal> and then anything X or SDL inside WSL will be forwarded to Xming or whatever
</pre>

== Step by Step ==

# Prerequisites
## A fully up to date Windows 10 x64 install
## [[Required Knowledge]]
# Install WSL
## Open Control Panel -> Program and Features -> Turn Windows features on or off
## Select "Windows Subsystem for Linux (Beta)"
## Hit OK
## Let it install
## Reboot (Yes, you MUST reboot before continuing)
# Installing BASH
## Open an elevated command prompt
## Run "bash" and follow the instructions/prompts provided
# Setup shared folder (Optional)
## Create a folder somewhere in your windows filesystem that you will use for easy access (I created a wsl folder in My Documents)
## "cd ~"
## "ln -s "wsl" "/mnt/c/Users/<WindowsUserName>/Documents/wsl" Replacing paths as you prefer
# Installing and Updating software
## Updating system GCC
### Add PPA for gcc-7: sudo add-apt-repository ppa:ubuntu-toolchain-r/test
### Update apt lists so it can find gcc-7: sudo apt-get update
### Install GCC 7: sudo apt-get install gcc-7
### Point gcc to the new version: sudo update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-7 60 --slave /usr/bin/g++ g++ /usr/bin/g++-7
#### You can see here if you want to have multiple copies of gcc installed and be able to switch between them: https://askubuntu.com/questions/466651/how-do-i-use-the-latest-gcc-on-ubuntu/581497#581497
## Installing/Compiling a cross compiler
### Prereqs
#### sudo apt-get install make bison flex libgmp3-dev libmpfr-dev libmpc-dev texinfo
### Follow [[GCC_Cross-Compiler]]
## Install additional tools
### sudo apt-get install git grub-pc-bin xorriso
### Optional: sudo apt-get install nasm genisoimage
## Clean Up
### sudo apt-get autoremove
#### Repeat til nothing shows up
# Bare Bones
## See [[Bare_Bones]]

== ToDo ==
# Finish Step-by-Step
# Test Step-by-Step + clean-up
# Remove specific version numbers and add instructions to find "current" versions
# Test again
# Seperate instructions and add to appropriate Wiki Pages
## [[GCC_Cross-Compiler]]
## [[Building_GCC]]
## [[Bare_Bones]]
````

<a id="w003968"></a>

## W003968 — VirtualBox Guest Additions

Bron-ID: W003968
Namespace: Main (0)
Revisie: 28067 | Datum: 2023-07-09T20:39:42Z
Bron: https://wiki.osdev.org/VirtualBox_Guest_Additions
Revisie-URL: https://wiki.osdev.org/index.php?title=VirtualBox_Guest_Additions&oldid=28067

````text
This article aims to describe some of the more useful / easier to implement features of the VirtualBox Guest Additions.

== The VirtualBox Guest Device ==

The Guest Additions package operates through a PCI/mmio device provided by the VM. A combination of memory-mapped packets, MMIO port writes, and IRQs allow the guest to communicate its feature support to the VM and for the VM to communicate events such as display changes and mouse movement. The PCI device has the vendor ID '''0x80EE''', same as the VirtualBox implementation of the Bochs display adapter, and a device ID of '''0xCAFE'''. BAR0 is the MMIO port and BAR1 is a memory region that contains some shared state for the device, such as a bit mask for what events the guest wishes to receive. The format for the latter is unimportant for the features described on this page - you need only know that the region can be interpreted as an array of 32-bit unsigned integers and offset 3 contains the IRQ mask.

Communication between the guest and the host happens through packets in memory. These can be anywhere in physical memory, and are relatively short. The MMIO operations that read or populate packets are synchronous, so setting up one page to pass back and forth is feasible. The basic process for sending a message to the VM is to prepare a packet and then write its (physical) address to the MMIO port. Receiving works the same way, as you must prepare a packet with a request type and provide its physical address; after the MMIO port write finishes, the packet will be populated with the appropriate values if the request was successful. Communication from the VM is primarily initiated by an IRQ on the PCI device's interrupt line, which should then be followed up by appropriate packet requests, one of which should be of a special "Acknowledge Events" type.

This page contains some struct definitions which could alternatively be obtained from headers provided by VirtualBox, but those headers are rather heavy and assume they are being used alongside a robust set of system headers, so instead we will define these structs ourselves.

Pseudo-code examples in the following sections will assume the availability of these functions; adjust them to match your environment:

<source lang="c">
// Allocate one page of memory, providing its physical address as an output and virtual address as a return value.
void * allocate_physical_page(uint32_t * physical);
// Map a physical page into the virtual memory space.
void * map_physical_page(uint32_t physical);
// Install an interrupt request handler function.
void install_interrupt_handler(irq, int (*irq_function)(void));
// Write a 32-bit value to an MMIO port.
void outportl(port, value);
// Write a 16-bit value to an MMIO port.
void outports(port, value);
// Return an object describing the requested PCI device, if found.
pci_device_t pci_find(vendor,device);
// Read a field from the given PCI device
uint32_t pci_read_field(pci_device_t device, field, size);
</source>

== Initializing the Device ==

Before we can do anything with the guest device, we need to tell it about ourselves. There are two protocols that current versions of VirtualBox support: 1.03 and 1.04. We will use 1.03, the so-called "Legacy Protocol", as it is slightly simpler.

<source lang="c">
#define VBOX_VENDOR_ID 0x80EE
#define VBOX_DEVICE_ID 0xCAFE
#define VBOX_VMMDEV_VERSION 0x00010003
#define VBOX_REQUEST_HEADER_VERSION 0x10001

#define VBOX_REQUEST_GUEST_INFO 50

/* VBox Guest packet header */
struct vbox_header {
        uint32_t size; /* Size of the entire packet (including this header) */
        uint32_t version; /* Version; always VBOX_REQUEST_HEADER_VERSION */
        uint32_t requestType; /* Request code */
        int32_t  rc; /* This will get filled with the return code from the requset */
        uint32_t reserved1; /* These are unused */
        uint32_t reserved2;
};

/* VBox Guest Info packet (legacy) */
struct vbox_guest_info {
        struct vbox_header header;
        uint32_t version;
        uint32_t ostype;
};

static pci_device_t vbox_pci;
static int vbox_port;
static uint32_t * vbox_vmmdev;

static void vbox_guest_init(void) {
    /* Find the guest device */
    pci_device_t vbox_pci = pci_find(VBOX_VENDOR_ID, VBOX_DEVICE_ID);
    
    /* BAR0 is the IO port. */
    vbox_port = pci_read_field(vbox_pci, PCI_BAR0, 4) & 0xFFFFFFFC;

    /* BAR1 is the memory-mapped "vmmdevmem" area. */
    vbox_vmmdev = map_physical_page(pci_read_field(vbox_pci, PCI_BAR1, 4) & 0xFFFFFFF0);

    /* Allocate some space for our Guest Info packet */
    uint32_t guest_info_phys;
    struct vbox_guest_info * guest_info = allocate_physical_page(&guest_info_phys);

    /* Populate the packet */
    guest_info->header.size = sizeof(struct vbox_guest_info);
    guest_info->header.version = VBOX_REQUEST_HEADER_VERSION;
    guest_info->header.requestType = VBOX_REQUEST_GUEST_INFO;
    guest_info->header.rc = 0;
    guest_info->header.reserved1 = 0;
    guest_info->header.reserved2 = 0;
    guest_info->version = VBOX_VMMDEV_VERSION;
    guest_info->ostype = 0; /* 0 = Unknown (32-bit); we don't need to lie about being another OS here */

    /* And send it to the VM */
    outportl(vbox_port, guest_info_phys);

    /* (We could check the return value here as well) */
}
</source>


== Auto-resize Guest Display ==
The first feature of the Guest Additions we'll look at is "Auto-resize Guest Display", which will allow your OS to be informed of the best resolution to use on the display adapter. The VirtualBox display adapter is itself based on the Bochs/Qemu display adapter, though it uses a different PCI vendor and device ID. It allows for any resolution to be set (in older versions, it required widths to be multiples of 4). With this capability enabled, we can receive interrupts when the host window size changes, and querying the guest device will tell us what resolution we should set the display to.

We'll need to define some new packets, install an interrupt handler, and also set some bits in the VMMDevMem space.

<source lang="c">
#define VBOX_REQUEST_ACK_EVENTS 41
#define VBOX_REQUEST_GET_DISPLAY_CHANGE 51
#define VBOX_REQUEST_SET_GUEST_CAPS 55

/* VBox Guest Capabilities packet */
struct vbox_guest_caps {
        struct vbox_header header;
        uint32_t caps;
};

/* VBox Acknowledge Events packet */
struct vbox_ack_events {
        struct vbox_header header;
        uint32_t events;
};

/* VBox GetDisplayChange packet */
struct vbox_display_change {
        struct vbox_header header;
        uint32_t xres;
        uint32_t yres;
        uint32_t bpp;
        uint32_t eventack;
};

/* We'll use separate pages for our packets for simplicity. */
static uint32_t vbox_display_phys;
static uint32_t vbox_ack_phys;
static vbox_display_change * vbox_display;
static vbox_ack_events * vbox_ack;

/* Adjust as necessary for your interrupt handling. */
static int vbox_irq_handler(struct regs * r) {
    if (!vbox_vmmdev[2]) return 0; /* Nothing to process? Maybe this interrupt was from something sharing the line. */

    vbox_ack->events = vbox_vmmdev[2];
    outportl(vbox_port, vbox_ack_phys); /* Acknowledge events */

    outportl(vbox_port, vbox_display_phys); /* Request display change information. */

    /* vbox_display now has information on our display size. If it changed we can tell our display driver to update. */
    set_new_graphics_mode_maybe(vbox_display->xres, vbox_display->yres, vbox_display->bpp);

    /* You probably want to make sure you have a way of informing your userspace that the
       display resolution is changing, so your window manager or whatever updates. */

    /* Don't forget to acknowledge the interrupt itself if you need to. */
    return 1;
}

static void vbox_guest_init(void) {
    ...

    /* Install an interrupt handler. */
    int irq = pci_read_field(vbox_pci, PCI_INTERRUPT_LINE, 1);
    install_interrupt_handler(irq, vbox_irq_handler);

    ...

    /* We need to tell the VM that we support this capability. The Guest Capabilities request tells the VM about our ability to support seamless and auto-resize modes. */
    uint32_t guest_caps_phys;
    struct vbox_guest_caps * guest_caps = allocate_physical_page(&guest_caps_phys);

    guest_caps->header.size = sizeof(struct vbox_guest_caps);
    guest_caps->header.version = VBOX_REQUEST_HEADER_VERSION;
    guest_caps->header.requestType = VBOX_REQUEST_SET_GUEST_CAPS;
    guest_caps->header.rc = 0;
    guest_caps->header.reserved1 = 0;
    guest_caps->header.reserved2 = 0;
    guest_caps->caps = 1 << 2; /* set bit 2, which indicates we support "graphics" (auto-resize guest display). */
    outportl(vbox_port, guest_caps_phys);

    /* We'll also set up the packets we'll use later for AcknowledgeEvents and GetDisplayChange */
    vbox_ack = allocate_physical_page(&vbox_ack_phys);
    vbox_ack->header.size = sizeof(struct vbox_ack_events);
    vbox_ack->header.version = VBOX_REQUEST_HEADER_VERSION;
    vbox_ack->header.requestType = VBOX_REQUEST_ACK_EVENTS;
    vbox_ack->header.rc = 0;
    vbox_ack->header.reserved1 = 0;
    vbox_ack->header.reserved2 = 0;
    vbox_ack->events = 0;

    vbox_display = allocate_physical_page(&vbox_display_phys);
    vbox_display->header.size = sizeof(struct vbox_display_change);
    vbox_display->header.version = VBOX_REQUEST_HEADER_VERSION;
    vbox_display->header.requestType = VBOX_REQUEST_GET_DISPLAY_CHANGE;
    vbox_display->header.rc = 0;
    vbox_display->header.reserved1 = 0;
    vbox_display->header.reserved2 = 0;
    vbox_display->xres = 0;
    vbox_display->yres = 0;
    vbox_display->bpp = 0;
    vbox_display->eventack = 1;

    /* Finally, we need to enable interrupts for the capabilities we've advertised. We're just going to enable all of them. */
    vbox_vmmdev[3] = 0xFFFFFFFF;
}
</source>

== Mouse Integration ==
Mouse Integration provides mouse position information using an absolute coordinate system. It does not provide information on mouse buttons, though, and that continues to go through the standard PS/2 (or USB) mouse devices. If your OS supports USB devices, mouse integration can implemented through a USB tablet devices instead of the mechanism described in this article.

Mouse Integration operates entirely over the guest device. Once enabled, mouse movements are sent to the guest through mouse packet requests and a corresponding interrupt. It is important to note that the format of the coordinates in these packets is based on a range from 0 to 0xFFFF which needs to be scaled to the display resolution. Why this approach was taken over using actual pixel coordinates (scaled or otherwise) is unknown.

<source lang="c">
#define VBOX_REQUEST_GET_MOUSE 1
#define VBOX_REQUEST_SET_MOUSE 2

/* The Mouse packet is used both to advertise our guest capabilities and to receive mouse movements. */
struct vbox_mouse_absolute {
        struct vbox_header header;
        uint32_t features;
        int32_t x;
        int32_t y;
};

static uint32_t vbox_mouse_phys;
static vbox_mouse_absolute * vbox_mouse;

static int vbox_irq_handler(struct regs * r) {
    ...

    outportl(vbox_port, vbox_mouse_phys);

    ...

    /* The mouse coordinates are scaled to the range (0x0,0xFFFF) independently in each dimension, so let's convert to pixels.
       If you prefer to have a more accurate mouse (subpixel support, etc.) you can convert to something else. */
    unsigned int x = ((unsigned int)vbox_mouse->x * display_resolution_width) / 0xFFFF;
    unsigned int y = ((unsigned int)vbox_mouse->y * display_resolution_height) / 0xFFFF;
    do_something_useful_with_mouse_coordinates(x,y);
    
    ...
}

static void vbox_guest_init(void) {
    ...

    vbox_mouse = allocate_physical_page(&vbox_mouse_phys);
    vbox_mouse->header.size = sizeof(struct vbox_mouse_absolute);
    vbox_mouse->header.version = VBOX_REQUEST_HEADER_VERSION;
    vbox_mouse->header.requestType = VBOX_REQUEST_SET_MOUSE;
    vbox_mouse->header.rc = 0;
    vbox_mouse->header.reserved1 = 0;
    vbox_mouse->header.reserved2 = 0;
    vbox_mouse->features = (1 << 0) | (1 << 4); /* bit 0 says "guest supports (and wants) absolute mouse"; bit 4 says we'll query absolute positions on interrupts */
    vbox_mouse->x = 0;
    vbox_mouse->y = 0;
    outportl(vbox_port, vbox_mouse_phys);

    vbox_mouse->header.requestType = VBOX_REQUEST_GET_MOUSE; /* Change the packet to a Get packet for use in the interrupt handler. */

    ...
}
</source>

== Seamless Desktop ==

The seamless desktop mode in VirtualBox works by sending the virtual machine a list of rectangles representing the non-background visible regions of the desktop. The host creates a borderless, maximized window and shapes it based on the rectangles provided by the guest, yielding the appearance of a seamless desktop. You can tie this into a driver that receives window bounds information from your GUI. Note that seamless works best with (and maybe requires?) Auto-Resize Guest Display support.

<source lang="c">

#define VBOX_REQUEST_SET_VISIBLE_REGION 71

struct vbox_rtrect {
    int32_t xLeft;
    int32_t yTop;
    int32_t xRight;
    int32_t yBottom;
}

struct vbox_visibleregion {
    struct vbox_header header;
    uint32_t count;
    struct vbox_rtrect rect[];
}

static uint32_t vbox_visibleregion_phys;
static vbox_visibleregion * vbox_visibleregion;

...

static void vbox_set_visible_region(uint32_t count, ...) {
    vbox_visibleregion = allocate_physical_page(&vbox_visibleregion_phys);
    vbox_visibleregion->header.size = sizeof(struct vbox_visibleregion) + sizeof(vbox_rtrect) * count;
    vbox_visibleregion->header.version = VBOX_REQUEST_HEADER_VERSION;
    vbox_visibleregion->header.requestType = VBOX_REQUEST_SET_VISIBLE_REGION;
    vbox_visibleregion->header.rc = 0;
    vbox_visibleregion->header.reserved1 = 0;
    vbox_visibleregion->header.reserved2 = 0;
    vbox_visibleregion->count = count;

    /* Set each of the rectangles... */
    vbox_visibleregion->rect[i].xLeft = ...;
    /* etc */
    
    outportl(vbox_port, vbox_visibleregion_phys);
}

</source>

''There is an additional interface for the seamless mode using the (undocumented) host-guest communication interface. In the case of the seamless mode, this functionality only communicates whether the seamless mode has been requested and is not necessary for it to function.''

''There are also hints in the VirtualBox codebase that a direct guest-host window mapping feature was planned but has not been implemented. This would presumably allow the guest to provide separate textures for each window, possibly alongside window title and icon information...''

== Pointer Shape ==

The Pointer Shape functionality allows you to send a mouse cursor texture to the host, saving you the need to draw it yourself in software. This functionality is especially useful in combination with the Seamless mode, as it allows your cursor to be rendered outside of the visible regions. This works with the absolute mouse functionality so there is no need to inform the host of the mouse cursor position as it already knows.

== Shared Folders and Clipboard ==
The virtual Guest Additions PCI device provides access to a service on the host called the Host-Guest Communication Manager (HGCM).  This service is responsible for providing functionality for Shared Folders, Shared Clipboard and making configuration changes to the guest itself.

All requests to the communication manager are made with the same packet header, and there are 5 values used for the packet type field:

{| {{wikitable}}
! Value (decimal)
! Request Type
|-
| 60 || Connect
|-
| 61 || Disconnect
|-
| 62 || Call Function (32-bit)
|-
| 63 || Call Function (64-bit)
|-
| 64 || Cancel Request
|}

After the standard Guest Addition Header, each communication manager packet (except the cancel request packet) has a second header, depending on the request type above.

=== Connect ===

{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 0x20 || 4 || Location Type
|-
| 0x24 || 128 || Location
|-
| 0xA4 || 4 || Client ID
|}

The Location field contains the name of the library to connect.  There are 4 pre-defined library names: VBoxSharedFolders, VBoxSharedClipboard, VBoxGuestPropSvc, VBoxSharedOpenGL

Additional libraries may also be accessed by providing the library name in the Location field.

The Location Type field should be set to 1 to connect to an additional library, and 2 to connect to one of the pre-defined libraries.

The Client ID is filled by the host after the request is complete.  This value should be used in any further calls to this service.

=== Disconnect ===
{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 0x20 || 4 || Client ID
|}

=== Call Function ===

Both the 32-bit and 64-bit call function packets use the same service header, followed by a list of parameter records.  32-bit function parameter records are 12 bytes long, and 64-bit function parameter records are 16 bytes long.

{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 0x20 || 4 || Type
|-
| 0x24 || 4 || Client ID
|-
| 0x28 || 4 || Function Code
|-
| 0x2c || 4 || Parameter Count
|-
| 0x30 || * || Parameters
|}


==== 32-bit Parameter ====
{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 0x00 || 4 || Type
|-
| 0x04 || 8 || Value
|}

==== 64-bit Parameter ====
{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 0x00 || 4 || Type
|-
| 0x04 || 12 || Value
|}

==== Parameter Types ====
{| {{wikitable}}
! Value (decimal)
! Request Type
|-
| 1 || 32-bit Value
|-
| 2 || 64-bit Value
|-
| 3 || Physical Address
|-
| 4 || Linear Address
|-
| 5 || Linear Address (Host-to-Guest only)
|-
| 6 || Linear Address (Guest-to-Host only)
|-
| 7 || Linear Address (Pre-Locked by Guest)
|-
| 8 || Linear Address (Host-to-Guest only, Pre-Locked by Guest)
|-
| 9 || Linear Address (Guest-to-Host only, Pre-Locked by Guest)
|}

Physical and Linear Address parameter values have the following structure:

{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 0x00 || 4 || Buffer Length
|-
| 0x04 || 4/8 || Address (32-Bit/64-Bit)
|}

=== Functions ===
{| {{wikitable}}
! Value (decimal)
! Function
! Parameters
|-
| 1 || Query Mappings || 3
|-
| 2 || Query Map Name || 2
|-
| 3 || Create Handle || 3
|-
| 4 || Close Handle || 2
|-
| 5 || Read || 5
|-
| 6 || Write || 5
|-
| 7 || Lock || 5
|-
| 8 || List Directory || 8
|-
| 9 || Information || 5
|-
| 11 || Delete || 3
|-
| 14 || Rename || 4
|-
| 15 || Flush || 2
|-
| 24 || Set File Size || 3
|}

=== Query Mappings ===
The Query Mappings command returns a list of all of the shared folders configured for this guest.  The buffer passed by the guest will be filled with mapping records, and the Count parameter will be set to the number of records returned.  The Root value for each record will be used to reference this shared folder in future commands.  If the buffer size is insufficient to hold all of the records, the buffer pointer size field will be set to the minimum size necessary to contain the entire list of mapping records.  By default, all strings returned by the host will be in UCS2 encoding.  Passing the UTF8 flag will force all requests to return strings in UTF8 format.  The Automount Only flag will only return shared folders that have been marked as Automount in the guest configuration.

{| {{wikitable}}
! Number
! Parameter
! Type
|-
| 1 || Flags || 32-Bit (0 - UCS2, 1 - UTF8, 2 - Automount Only)
|-
| 2 || Count || 32-Bit
|-
| 3 || Buffer || Pointer
|}

==== Mapping Record ====
{| {{wikitable}}
! Offset (hex)
! Length
! Description
|-
| 0x00 || 4 || Flags (1 - New, 2 - Deleted)
|-
| 0x04 || 4 || Root
|}


=== Query Map Name ===
The Query Map Name command will return the name for the specified shared folder in the guest configuration.

{| {{wikitable}}
! Number
! Parameter
! Type
|-
| 1 || Root || 32-Bit
|-
| 2 || Buffer || Pointer
|}


== Other Stuff ==
VirtualBox also provides a mechanism for writing to its log files, though this is not managed through the guest device. Simple writing bytes to port 0x504 will produce log entries. The capability bit for this functionality is bit 0.


==See Also==

===External Links===
*[https://github.com/klange/toaruos/blob/master/modules/vbox.c ToaruOS VirtualBox guest driver]
*[https://www.virtualbox.org/browser/vbox/trunk/src/VBox/Devices/VMMDev/VMMDev.cpp VirtualBox host implementation of the guest additions]
*[https://www.virtualbox.org/browser/vbox/trunk/src/VBox/Additions VirtualBox guest implementations]
*[https://www.virtualbox.org/wiki/Technical_documentation VirtualBox Technical Documentation (SDK)]

[[Category:Emulators]]
````

<a id="w003969"></a>

## W003969 — User:Klange/VirtualBox/Guest Additions

Bron-ID: W003969
Namespace: User (2)
Revisie: 20685 | Datum: 2017-04-10T17:41:38Z
Bron: https://wiki.osdev.org/User:Klange/VirtualBox/Guest_Additions
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKlange%2FVirtualBox%2FGuest_Additions&oldid=20685
Doorverwijzing: VirtualBox Guest Additions

````text
#REDIRECT [[VirtualBox Guest Additions]]
````

<a id="w003970"></a>

## W003970 — VirtualBox/Guest Additions

Bron-ID: W003970
Namespace: Main (0)
Revisie: 19747 | Datum: 2016-08-30T06:58:53Z
Bron: https://wiki.osdev.org/VirtualBox/Guest_Additions
Revisie-URL: https://wiki.osdev.org/index.php?title=VirtualBox%2FGuest_Additions&oldid=19747
Doorverwijzing: VirtualBox Guest Additions

````text
#REDIRECT [[VirtualBox Guest Additions]]
````

<a id="w003971"></a>

## W003971 — Talk:Meaty Skeleton

Bron-ID: W003971
Namespace: Talk (1)
Revisie: 30810 | Datum: 2026-06-22T20:51:37Z
Bron: https://wiki.osdev.org/Talk:Meaty_Skeleton
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMeaty_Skeleton&oldid=30810

````text
== 2016-09-02 Changes ==

Hello. I'm am modernizing Meaty Skeleton a bunch. The list of changes is too big for the summary field, so here's the list of changes:

* Avoid unportable make -C option and just use cd.
* Don't use unportable rm -v option.
* Fix typo in config.sh.
* Don't rely on PWD variable being set, just use pwd(1) to get it.
* Use upcoming-POSIX != make syntax to run shells and not $(shell).
* Don't overuse := in makefiles.
* Don't locate crtbegin.o and crtend.o when not compiling.
* Switch to portable .SUFFIXES wildcards to get them more known.
* Fix kernel link like order with respect to libk and libgcc.
* Verify multibootness with grub-file.
* Use -MD to properly handle header dependency tracking.
* Replace unportable cp -RTv.
* Pass unportable --preserve=timestamp to cp to avoid -MD issues.
* Align boot.S and linker.ld with bare bones.
* Remove kernel_early abstraction.
* Switch coding style to match bare bones.
* Move vga text mode buffer constants into tty.c.
* Move vga.h from public kernel header to private arch header.
* Prefix all vga.h declarations with vga_.
* Remove _myos from __is_kernel/libc/libk detection macros.
* Don't build libc.a anymore.
* Add EOF to stdio.h to detect putchar errors.
* Simplify printf, handle errors in putchar and error output exceeds INT_MAX.
* Clarify non-libk case in putchar and abort.
* Don't pass -fbuiltin as it's not safe for freestanding environments.

See the history page on the article for the full changes. --[[User:Sortie|Sortie]] 14:59, 2 September 2016 (CDT)

== Reverting makefile changes ==

Unfortunately, Debian and Ubuntu based distros (and probably many, many more) are still on GNU Make 3.8, which does not support the use of !=.  I have added troubleshooting to the bottom of the page to document this problem and how to solve it:

<pre>
DEFAULT_HOST := $(shell ../default-host.sh)
</pre>

[[User:Sigtau|Sigtau]] 00:03, 3 October 2016 (CDT)

== Deprecation of the tutorial ==

For the same reasons, as in the [[Talk:Bare Bones#Deprecation of the tutorial]], I am proposing to deprecate this tutorial, due to the numerous problems that this brings to the unsuspected beginners that might follow it, that are associated with using very outdated stuff, including multiboot1, VGA text, no paging, targeting ia32, and so on.

--[[User:Mishakov|Mishakov]] ([[User talk:Mishakov|talk]]) 08:15, 11 June 2026 (UTC)

:Yeah, I am currently overhauling the article (will be submitted soon) to use Multiboot 2, with framebuffer font rendering and serial line support too. As for paging, you're starting to ask for the impossible.  Do not forget that Bare Bones is the Hello World tutorial. It's supposed to show people how to compile some boot and boot to hello world, with a reasonable amount of knowledge. A full Multiboot 2 kernel, with 64-bit paging, font rendering, and everything else -- it's a LOT of concepts for newcomers to understand. We have these easy 32-bit tutorials without paging, which people can solve, and then move onto the more advanced tutorials, one topic at a time. --[[User:Sortie|Sortie]] ([[User talk:Sortie|talk]]) 20:51, 22 June 2026 (UTC)
````

<a id="w003974"></a>

## W003974 — Prerequisites

Bron-ID: W003974
Namespace: Main (0)
Revisie: 19761 | Datum: 2016-09-03T08:27:18Z
Bron: https://wiki.osdev.org/Prerequisites
Revisie-URL: https://wiki.osdev.org/index.php?title=Prerequisites&oldid=19761
Doorverwijzing: Required Knowledge

````text
#REDIRECT[[Required Knowledge]]
````

<a id="w003975"></a>

## W003975 — Category:Binutils

Bron-ID: W003975
Namespace: Category (14)
Revisie: 30161 | Datum: 2026-03-09T08:51:35Z
Bron: https://wiki.osdev.org/Category:Binutils
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ABinutils&oldid=30161

````text
Binutils (GNU Binary Utilities) is a set of tools for creating and managing binary programs, object files, libraries and assembly source code.
[[Category:Tools]]
````

<a id="w003976"></a>

## W003976 — WhyNeverToPlaceFunctionsInIncludeHeaders

Bron-ID: W003976
Namespace: Main (0)
Revisie: 20655 | Datum: 2017-04-02T14:44:42Z
Bron: https://wiki.osdev.org/WhyNeverToPlaceFunctionsInIncludeHeaders
Revisie-URL: https://wiki.osdev.org/index.php?title=WhyNeverToPlaceFunctionsInIncludeHeaders&oldid=20655
Doorverwijzing: Why function implementations shouldn't be put In header files

````text
#REDIRECT [[Why function implementations shouldn't be put In header files]]
````

<a id="w003977"></a>

## W003977 — User:Glauxosdever/Maintained

Bron-ID: W003977
Namespace: User (2)
Revisie: 19896 | Datum: 2016-10-14T16:02:56Z
Bron: https://wiki.osdev.org/User:Glauxosdever/Maintained
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGlauxosdever%2FMaintained&oldid=19896

````text
Here are pages that are maintained by [[User:Glauxosdever|glauxosdever]].

== Guides ==

* [[Going Further on x86]]

== Tutorials ==

* [[User:Glauxosdever/Higher Half x86 Bare Bones]]

== Miscellaneous ==

* [[Bossy Boots]]
* [[Duct von Tape]]
* [[Mister Perfect]]
````

<a id="w003978"></a>

## W003978 — User talk:Sortie/Naively Closed Source

Bron-ID: W003978
Namespace: User talk (3)
Revisie: 19812 | Datum: 2016-09-12T08:48:14Z
Bron: https://wiki.osdev.org/User_talk:Sortie/Naively_Closed_Source
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ASortie%2FNaively_Closed_Source&oldid=19812

````text
== More misguided fears ==

There are people that fear that an open source OS will be perceived as inferior to a proprietary one.

Maybe consider adding it as a reason?

[[User:Glauxosdever|Glauxosdever]] 03:48, 12 September 2016 (CDT)
````

<a id="w003979"></a>

## W003979 — User:Lucasaiu

Bron-ID: W003979
Namespace: User (2)
Revisie: 19836 | Datum: 2016-09-22T07:30:39Z
Bron: https://wiki.osdev.org/User:Lucasaiu
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALucasaiu&oldid=19836

````text
Hello, I'm [http://ageinghacker.net Luca Saiu].
````

<a id="w003981"></a>

## W003981 — Category:Developer Archetype

Bron-ID: W003981
Namespace: Category (14)
Revisie: 19877 | Datum: 2016-10-12T17:12:32Z
Bron: https://wiki.osdev.org/Category:Developer_Archetype
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ADeveloper_Archetype&oldid=19877

````text
These are the developer archetypes as described in the page [[What order should I make things in]]. The category is simply to make it easier to compare them.
````

<a id="w003982"></a>

## W003982 — Category:Anti-Pattern

Bron-ID: W003982
Namespace: Category (14)
Revisie: 19889 | Datum: 2016-10-12T17:24:47Z
Bron: https://wiki.osdev.org/Category:Anti-Pattern
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AAnti-Pattern&oldid=19889

````text
An '''Anti-Pattern''' is a [[Pattern]] that usually has negative results - the opposite of what a formalized pattern is meant to capture. Just as with positive patterns, there are different kinds of patterns for many different aspects of development - they can be a bad design approach, a harmful behavior on the part of a developer or manager, a poor development model, or some combination of those.
````

<a id="w003983"></a>

## W003983 — Category:Developer Anti-Pattern

Bron-ID: W003983
Namespace: Category (14)
Revisie: 19890 | Datum: 2016-10-12T17:25:20Z
Bron: https://wiki.osdev.org/Category:Developer_Anti-Pattern
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ADeveloper_Anti-Pattern&oldid=19890

````text
[[Category:Anti-Pattern]]
````

<a id="w003984"></a>

## W003984 — User:Glauxosdever/Mister Perfect

Bron-ID: W003984
Namespace: User (2)
Revisie: 19894 | Datum: 2016-10-14T15:58:54Z
Bron: https://wiki.osdev.org/User:Glauxosdever/Mister_Perfect
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGlauxosdever%2FMister_Perfect&oldid=19894
Doorverwijzing: Mister Perfect

````text
#REDIRECT [[Mister Perfect]]
````

<a id="w003985"></a>

## W003985 — User:Gravaera/UDI Direct Memory Access

Bron-ID: W003985
Namespace: User (2)
Revisie: 19904 | Datum: 2016-10-22T08:08:22Z
Bron: https://wiki.osdev.org/User:Gravaera/UDI_Direct_Memory_Access
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FUDI_Direct_Memory_Access&oldid=19904

````text
= Uniform Driver Interface =

== Purpose of this article ==
UDI's DMA abstraction and API is fairly complex, and somewhat difficult to follow if attempted to be scrutinized directly from the text of the UDI Core Specification. This article seeks to take the disjoint, scattered components of the UDI DMA framework, and present them in a cogent, sequential format for easier digestion.

== Recommended reading ==
* UDI Core Specification.

== Key data-structures ==
<source lang="c">
/* UDI buffer handle.
 * Buffer handles to each parent of a device, are passed to that device as part
 * of the UDI_CHANNEL_BOUND event on its parent-bind channel. The driver will receive
 * one udi_buf_t for each of its parents, allowing it to indicate a path to each
 * of its parents.
 */
typedef struct {
	udi_size_t buf_size;
} udi_buf_t;

/* UDI Buffer path: this is used when creating a UDI Buffer, and tells the
 * OS which target driver the buffer is going to be eventually sent to.
 *
 * From this information, the OS can then deduce whether or not it should
 * pro-actively allocate the memory for the buffer in some region of memory
 * that is DMA-compliant for the target-device.
 */
typedef udi_ubit8_t udi_dma_constraints_attr_t;
typedef struct {
	udi_dma_constraints_attr_t attr_type;
	udi_ubit32_t attr_value;
} udi_dma_constraints_attr_spec_t;
typedef <HANDLE> udi_dma_constraints_t;

/* UDI Scatter Gather list: acts as a list of range-descriptors
 * for physical memory that has been allocated in accordance with a
 * DMA Engine's RAM constraints.
 */
typedef struct {
	udi_ubit32_t block_busaddr;
	udi_ubit32_t block_length;
} udi_scgth_element_32_t;
 
typedef struct {
	udi_busaddr64_t block_busaddr;
	udi_ubit32_t block_length;
	udi_ubit32_t el_reserved;
} udi_scgth_element_64_t;

typedef struct {
	udi_ubit16_t scgth_num_elements;
	udi_ubit8_t scgth_format;
	udi_boolean_t scgth_must_swap;

	union {
	    udi_scgth_element_32_t *el32p;
	    udi_scgth_element_64_t *el64p;
	} scgth_elements;

	union {
	    udi_scgth_element_32_t el32;
	    udi_scgth_element_64_t el64;
	} scgth_first_segment;
} udi_scgth_t;
</source>
== Key APIs ==
<source lang="c">
typedef void udi_dma_prepare_call_t (
	udi_cb_t *gcb,
	udi_dma_handle_t new_dma_handle );

void udi_dma_prepare (
	udi_dma_prepare_call_t *callback,
	udi_cb_t *gcb,
	udi_dma_constraints_t constraints,
	udi_ubit8_t flags );

typedef void udi_dma_buf_map_call_t (
	udi_cb_t *gcb,
	udi_scgth_t *scgth,
	udi_boolean_t complete,
	udi_status_t status );

void udi_dma_buf_map (
	udi_dma_buf_map_call_t *callback,
	udi_cb_t *gcb,
	udi_dma_handle_t dma_handle,
	udi_buf_t *buf,
	udi_size_t offset,
	udi_size_t len,
	udi_ubit8_t flags );

typedef void udi_dma_mem_alloc_call_t (
	udi_cb_t *gcb,
	udi_dma_handle_t new_dma_handle,
	void *mem_ptr,
	udi_size_t actual_gap,
	udi_boolean_t single_element,
	udi_scgth_t *scgth,
	udi_boolean_t must_swap );

void udi_dma_mem_alloc (
	udi_dma_mem_alloc_call_t *callback,
	udi_cb_t *gcb,
	udi_dma_constraints_t constraints,
	udi_ubit8_t flags,
	udi_ubit16_t nelements,
	udi_size_t element_size,
	udi_size_t max_gap );

typedef void udi_dma_mem_to_buf_call_t (
	udi_cb_t *gcb,
	udi_buf_t *new_dst_buf );

void udi_dma_mem_to_buf (
	udi_dma_mem_to_buf_call_t *callback,
	udi_cb_t *gcb,
	udi_dma_handle_t dma_handle,
	udi_size_t src_off,
	udi_size_t src_len,
	udi_buf_t *dst_buf );
</source>

= Summary of the UDI DMA design =
== Key abstractions ==
The UDI DMA API works around several key abstractions:

* UDI Buffers, which are abstract notions of a "buffer" that are provided by the UDI environment implementation. These buffers can be mapped to physical memory that corresponds to a scatter-gather list, and act as the abstraction through which user-processes and drivers can both write to scatter-gather lists, or other types of DMA-purposed RAM.
* UDI Scatter-gather lists which represent a range of physical ram that has been allocated for DMA. These scatter-gather lists are allocated according to the parameters specified in a [[UDI Constraints Specification]].
* A [[UDI Constraints Specification]], which tells the environment what kind of memory is needed for a particular DMA Engine's use. Some DMA Engines can only operate on memory in the first 16MiB of RAM, for example. Other DMA Engines may have a 16KiB alignment requirement, and so on.
* A UDI Buffer Path, which is a handle to a structure whose layout is unknown. This Buffer Path is used when allocating UDI Buffers. The Buffer Path tells the environment implementation **where** this memory buffer is likely to be passed to via IPC. It is through this mechanism that UDI enables zero-copy DMA that can be exposed all the way through to userspace. The OS can expose a syscall that allows userspace to allocate buffer memory that it will eventually pass up through a particular "Path", to a particular target driver. Armed with this information, the OS can optimize by allocating RAM that conforms to the target driver's [[UDI Constraints Specification]], and give it directly to the user process. Then, when the userspace process eventually makes a syscall to transfer this buffer to the target driver, there will be no need to copy the data in the buffer to DMA-compliant RAM, since the buffer was mapped to DMA-compliant RAM from the start.

== Benefits ==
* Has the ability to propagate zero-copy data transfer from user-applications down to the device.
* Respects the target DMA Engine controller's endianness.

== Drawbacks ==
* Fairly complex to understand.

INCOMPLETE
````

<a id="w003986"></a>

## W003986 — CompilerDev/Implementing Conditional Statements And Loops

Bron-ID: W003986
Namespace: Main (0)
Revisie: 28080 | Datum: 2023-07-09T20:55:44Z
Bron: https://wiki.osdev.org/CompilerDev/Implementing_Conditional_Statements_And_Loops
Revisie-URL: https://wiki.osdev.org/index.php?title=CompilerDev%2FImplementing_Conditional_Statements_And_Loops&oldid=28080

````text
= General Concepts = 
For a conventional instruction set architecture such as x86, ARM, 8051, MIPS, or most other CPU types in widespread use at this time (2016), conditional statements such as '''if'''/'''elsif'''/'''else''', and loop constructs such as '''for''' or '''while''', are generally implemented through a combination of tests, conditional jumps/branches (jz, beq, etc.) and unconditional branches (jmp, bra, j, etc.). While some common ISAs have special-purpose instructions for repetition or looping (e.g., the '''REP''' and '''LOOP''' instructions on the x86), few compilers use them due to the added work of determining where they can be applied.

Examples below are given for [https://en.wikipedia.org/wiki/X86_assembly_language Intel x86-64], [[ARM Overview| ARM]], and [https://en.wikipedia.org/wiki/MIPS_instruction_set  MIPS R2000 assembly language] instruction sets. It is assumed that the compiler produces assembly language; for compilers that generate an [[Executable Formats| Executable File]] directly, the compiler must handle the housekeeping (e.g., tracking branch targets) that would otherwise be done by the assembler.

== General Conditional Statements ==
=== If ===
A naive implementation of a basic '''if''' statement will generally consist of a conditional branch to the code for the when the condition is true (the ''consequent''), an unconditional branch past the end of the consequent, and the consequent itself:

<source lang="C">if (a == b)
{
    /* do something */
}</source>

The generated code (assuming that '''a''' and '''b''' are in the appropriate registers already) might look like:

{|
! x86-64
! ARM
! MIPS
|+
|<pre>
    cmp rax, rbx
    je main.if0.true
    jmp main.if0.end
main.if0.true:
    ;;; consequent
main.if0.end:
</pre>
|<pre>
    teq r0, r1
    beq main.if0.true
    b main.if0.end
main.if0.true:
    ;;; consequent
main.if0.end:    
</pre>
|<pre>
    beq $t0, $t1, main.if0.true
    j main.if0.end
main.if0.true:
    ### consequent
main.if0.end:
</pre>
|}

Note that the compiler has to generate a unique label or target table listing for each of the branch targets; the simplest implementation of this is to keep a count of the labels, and assign them a separate label name with the count appended to it. For the sake of readability, the example code uses a local label with the function name (main, in this case), the type of expression, and the count of the expressions of this type. 

Thus, for nested '''if'''s, such as

<source lang="C">if (a == b)
{
    if (b <= c)
    {
        /* do the inner clause */
    }
    /* do the rest of the outer clause */
}</source>


it might produce:
{|
! x86-64
! ARM
! MIPS
|+
|<pre>    cmp rax, rbx
    je main.if1.true
    jmp main.if1.end
main.if1.true:
    cmp rbx, rcx
    jle main.if2.true
    jmp main.if2.end
main.if2.true:
    ;;; do the inner clause
main.if2.end:
    ;;; do the rest of the outer clause
main.if1.end:
</pre>
|<pre>
</pre>
|<pre>    bge $t0, $t1, main.if1.true
    j main.if1.end
main.if2.true:
    blt $t0, $t2, main.if2.true
    j main.if2.end
main.if2.true:
    ;;; consequent for inner conditional
main.if2.end:
    ;;; remaining code
main.if1.end:</pre>
|}

==== Compound Boolean Conditionals ====
For compound conditionals, such as logical AND or logical OR, there naive implementation would be to first perform the tests, then use the logical ligatures to produce a testable outcome. 

<source lang="C">if (!(a <= b && b < c))
    /* do something */
}</source>

it could naively generate 

{|
! x86-64
! ARM
! MIPS
|+
|<pre>
</pre>
|<pre>
</pre>
|<pre>
    ble $t0, $t1, main.if3.and0.left.true
    clear $t3
    j main.if3.and0.right
main.if3.and0.left.true:
    move $t3, 1
    j main.if3.and0.right
main.if3.and0.right:
    ble $t0, $t1, main.if3.and0.right.true
    clear $t3
    j main.if3.and0.test
main.if3.and0.right.true:
    move $t3, 1    
    j main.if3.and0.test
main.if3.and0.test:    
    and $t3, $t3, $t4
    not $t5, $t3
    bnez $t5, main.if3.true
    j main.if3.end
main.if3.true:
    ;;; consequent
main.if3.end:</pre> 
|}


While this is a literal translation of the condition, it is clearly less than optimal.

==== Basic Optimizations ====

An obvious, and reasonably simple, optimization of this is to negate or reverse the condition, thus eliminating the need for the unconditional branch:

{|
! x86-64
! ARM
! MIPS
|+
|<pre>
    cmp rax, rbx
    jne main.if0.end
    ;;; consequent
main.if0.end:
</pre>
|<pre>
    teq r0, r1
    bne main.if0.end
    ;;; consequent
main.if0.end:    
</pre>
|<pre>
    bne $t0, $t1, main.if4.end
    ;;; consequent
main.if4.end:</pre>
|}

Similarly, for nested '''if'''s where the inner '''if''' is the first or last clause of the outer loop, then it can remove an extraneous labels:

{|
! x86-64
! ARM
! MIPS
|+
|<pre>
    cmp rax, rbx
    jne main.if1.end
    ;;; if(rax == rbx)
      cmp rbx, rcx
      jg main.if2.end
      ;;; if(rbx <= rcx)
      ;;; do the inner clause
main.if2.end:
    ;;; do the rest of the outer clause
main.if1.end:
</pre>
|<pre>
</pre>
|<pre>
    blt $t0, $t1, main.if5.end
    bge $t0, $t2, main.if6.end
    ;;; inner consequent
main.if6.end:
    ;;; remaining code of outer consequent
main.if2.end:</pre>
|}


For these simple cases, since the mapping of a given operator to its inverse is fixed (e.g., to implement 'less than' you would always substitute 'greater than or equal'), the only change needed is that the code for the test be issued fro the inverse. 

Optimizing compound conditionals is a bit trickier, as you would need to consider which operations invert which other ones, and how to apply things like de Morgan's Law. However, it is still reasonable for most compilers to perform certain eliminations, such as replacing separate tests with short-circuiting branches. In the case of an '''AND''', one can start by short-circuiting the first test to the end of the '''and'' part conditional, so that the second test only gets checked if the first is true. For the '''NOT''', you can get the right result simply by ''not'' inverting the final case:

However, with a little more effort, a better solution might be possible, if we keep a table of boolean functions and their inverses, and are willing to perform a simple analysis of the order of operations.

== Definite Loops ==
The naive implementation of a definite loop is a conditional branch at the start of the loop and an unconditional branch back to that conditional branch at the end of the loop, which is also the naive implementation of an explicit for loop. An example in MIPS assembly code (using pseudo-instructions) might be:

{|
! x86-64
! ARM
! MIPS
|+
|<pre>
        xor rcx, rcx
        mov rbx, for_loop_count
for0.start:
        ; if rcx is greater than or equal 
        ; to rbx, jump past the loop
        cmp rcx, rbx
        jge for1.end
        ;;  body of the loop here
        inc rcx
        jmp for0.start
for0.end:</pre>
|<pre>        mov r0, #0                 
        mov r1, #for_loop_count    
for0.start:
        ; if r1 is greater than or equal 
        ; to r0, jump past the loop     
        cmp r0, r1
        bge for1.end       
        ;;  body of the loop here
        add r0, r0, #1
        b for0.start
for0.end:</pre>
|<pre>        clear $t0                 
        move $t1, for_loop_count    
for0.start:
        ; if t1 is greater than or equal 
        ; to t0, jump past the loop     
        bge $t0, $t1, for1.end        
        ;;  body of the loop here
        addi $t0, $t0, 1
        bra for0.start
for0.end</pre>
|}

However, even a naive compiler will usually do ''loop inversion'' by using an unconditional branch, followed by the loop label, followed by the body, and then put the loop condition as the target of the original unconditional branch with the condition inverted: 

{|
! x86-64
! ARM
! MIPS
|+
|<pre>
</pre>
|<pre>
</pre>
|<pre>   clear $t0
        move $t1, for_loop_count    
        bra for1.test
for1.start:
        ;;  body of the loop here
        addi $t0, $t0, 1
for1.test:
        ; if t1 is less than to t0, 
        ; jump past the loop
        blt $t0, $t1, for1.start         
for1.end</pre>
|}

This means that after the loop entry, the loop has just the unconditional branch, making it faster. For a hand-coded recursive descent compiler, this becomes simply a matter of changing the order in which the parsing-emitting functions are called.

[[Category:Compiler Development]]
````

<a id="w003987"></a>

## W003987 — Talk:CompilerDev/Implementing Conditional Statements And Loops

Bron-ID: W003987
Namespace: Talk (1)
Revisie: 19942 | Datum: 2016-10-30T18:50:38Z
Bron: https://wiki.osdev.org/Talk:CompilerDev/Implementing_Conditional_Statements_And_Loops
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ACompilerDev%2FImplementing_Conditional_Statements_And_Loops&oldid=19942

````text
Why is there a tutorial on implementing loops in some language or another on an OS development wiki? --[[User:Love4boobies|Love4boobies]] 20:52, 26 October 2016 (CDT)

* There is a standing plan to (eventually) create a separate wiki and forum for language design and compiler development. To this end, the OS wiki group established a set of pages for bootstrapping it, to be temporary hosted here. No one else has worked on it for a while, but the subject of how certain constructs came up in the fora, and was relevant to OS dev (because a question arose about efficiency for explicit loops versus gotos - one of the posters was using goto extensively on this basis, and we were able to show that it was ''not'' faster in general), so I thought it would make sense to add this as a permanent part of the CompilerDev wiki project.

** This argument is ridiculous and can very well be used to justify all sorts of nonsense. Implementing anything efficiently may be useful in an OS... or any other software project, for that matter. That doesn't mean it has a place here. For instance, learning how to program is a prerequisite to writing an operating systems. Does that mean we should dedicate pages to teaching basic programming? What about implementing efficient sorting algorithms? Someone on the forum even suggested we have a page describing the MP3 format. We are just making a wiki that is in a sorry state much worse. And, frankly, I am not entirely convinced it has a place even on a wiki about compiler theory. I strongly recommend we delete this page. --[[User:Love4boobies|Love4boobies]] 07:44, 28 October 2016 (CDT)

== Assembly Snippets Architecture ==

Most people are familiar with the x86 syntax. Could we consider converting the MIPS assembly snippets to x86 assembly snippets (preferably in Intel syntax)?

* OK, that sounds reasonable. I was using MIPS mainly because it presents fewer problems for compiler code generation than x86 as a rule (though the expanded register file in the x86-64 makes register painting easier, which is usually the biggest problem for x86 compiler writers), but as you said, it isn't familiar for most of the people using the wiki. If the wiki had a way to show multiple code examples as tabs, however, that would allow us to have examples for several ISAs.

** The vast majority of compilers will implement the branch instructions and perform optimizations on its own intermediate code, rather than doing it at the machine code level.  Thus some sort of pseudocode is probably more appropriate for this article.  [[User:Jnc100|John]] 13:50, 30 October 2016 (CDT)
````

<a id="w003988"></a>

## W003988 — Implementing Conditional Statements And Loops

Bron-ID: W003988
Namespace: Main (0)
Revisie: 19915 | Datum: 2016-10-27T19:16:50Z
Bron: https://wiki.osdev.org/Implementing_Conditional_Statements_And_Loops
Revisie-URL: https://wiki.osdev.org/index.php?title=Implementing_Conditional_Statements_And_Loops&oldid=19915
Doorverwijzing: CompilerDev/Implementing Conditional Statements And Loops

````text
#REDIRECT [[CompilerDev/Implementing Conditional Statements And Loops]]
````

<a id="w003989"></a>

## W003989 — Talk:Implementing Conditional Statements And Loops

Bron-ID: W003989
Namespace: Talk (1)
Revisie: 19917 | Datum: 2016-10-27T19:16:50Z
Bron: https://wiki.osdev.org/Talk:Implementing_Conditional_Statements_And_Loops
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AImplementing_Conditional_Statements_And_Loops&oldid=19917
Doorverwijzing: Talk:CompilerDev/Implementing Conditional Statements And Loops

````text
#REDIRECT [[Talk:CompilerDev/Implementing Conditional Statements And Loops]]
````

<a id="w003990"></a>

## W003990 — Bare Bones Compiler

Bron-ID: W003990
Namespace: Main (0)
Revisie: 19921 | Datum: 2016-10-27T19:25:46Z
Bron: https://wiki.osdev.org/Bare_Bones_Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=Bare_Bones_Compiler&oldid=19921
Doorverwijzing: CompilerDev/Bare Bones Compiler

````text
#REDIRECT [[CompilerDev/Bare Bones Compiler]]
````

<a id="w003991"></a>

## W003991 — User:NunoLava1998

Bron-ID: W003991
Namespace: User (2)
Revisie: 19936 | Datum: 2016-10-29T06:32:24Z
Bron: https://wiki.osdev.org/User:NunoLava1998
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANunoLava1998&oldid=19936

````text
Hi, i'm NunoLava1998.
I'm currently developing WaveSystem.
````

<a id="w003992"></a>

## W003992 — User:Nunolava1998

Bron-ID: W003992
Namespace: User (2)
Revisie: 19937 | Datum: 2016-10-29T06:33:07Z
Bron: https://wiki.osdev.org/User:Nunolava1998
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANunolava1998&oldid=19937

````text
hi
````

<a id="w003993"></a>

## W003993 — Which assembler should I use?(x86)

Bron-ID: W003993
Namespace: Main (0)
Revisie: 28524 | Datum: 2023-12-15T19:31:16Z
Bron: https://wiki.osdev.org/Which_assembler_should_I_use%3F(x86)
Revisie-URL: https://wiki.osdev.org/index.php?title=Which_assembler_should_I_use%3F%28x86%29&oldid=28524

````text
== Introduction ==
An assembler is a program which converts assembly language code to machine code. Even if you are not using assembly language as your primary programming language for your OS you will still have to write some of it. This article discusses advantages and disadvantages of the commonly used assemblers for OS development.

== GAS ==
The GNU Assembler, or [[GAS]], is the assembler included in the GNU binutils. It is also the assembler that is tightly integrated in the GNU toolchain and therefore interacts best with it. GAS uses the AT&T syntax by default though it can use Intel syntax as well. It is commonly coupled with the C preprocessor to enable macro capabilities. GAS can produce code for many different platforms.

== NASM ==
The Netwide Assember, or [[NASM]], is a popular assembler using Intel syntax and an own macro system. It has been written in C but only generates code for x86 and its predecessors.

== FASM == 
The Flat Assembler, or [[FASM]], is a self assembling assembler written entirely in assembly language and uses Intel-syntax with its own macro system. It only produces code for ia32 and amd64 platforms, though a fork for ARM systems exists. Its next generation, FASMG can generate almost for any architecture with its powerful macro system: x86, ARM, Z80 and even [[WebAssembly]].

== Conclusion ==
If you want to write your OS in C, C++ or another high level language you will need some minimal portion of assembly language. In this case, the recommended assembler to use is GAS as it integrates tightly with the GNU toolchain which is commonly used in OSDev. GAS is included in the GNU binutils and therefore does not introduce new dependencies to your project.

If you want to write an project in assembly language entirely, the latter two are usually the tools of choice. However, FASM being self-assembling and only needs a C standard library to work gives it a slight advantage here as it is way easier to port.

See also [[Tool_Comparison#x86_Assemblers|x86 Assemblers Comparison]] for a comparison of features between different assemblers.

[[Category:Assemblers]]
[[Category:Assembly]]
[[Category:x86]]
````

<a id="w003994"></a>

## W003994 — User:Gravaera/Freestanding GNU LibstdC++ v3

Bron-ID: W003994
Namespace: User (2)
Revisie: 22492 | Datum: 2018-07-01T08:35:54Z
Bron: https://wiki.osdev.org/User:Gravaera/Freestanding_GNU_LibstdC%2B%2B_v3
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FFreestanding_GNU_LibstdC%2B%2B_v3&oldid=22492

````text
= Preface =
Work in progress.

= It worked! =

<source lang="bash">
$ ../gcc-5.5.0/libstdc++-v3/configure --target=i586-elf --disable-nls --prefix=/home/user/bin/i586-elf --enable-cxx-flags='-fno-rtti -fno-exceptions -std=c++03' --enable-concept-checks --disable-hosted-libstdcxx --disable-libstdcxx-verbose --with-default-libstdcxx-abi=gcc4-compatible --disable-libstdcxx-filesystem-ts
configure: WARNING: unrecognized options: --with-multisubdir, --with-multisrctop
configure: WARNING: stdbool.h: present but cannot be compiled
configure: WARNING: stdbool.h:     check for missing prerequisite headers?
configure: WARNING: stdbool.h: see the Autoconf documentation
configure: WARNING: stdbool.h:     section "Present But Cannot Be Compiled"
configure: WARNING: stdbool.h: proceeding with the compiler's result
configure: WARNING: stdalign.h: present but cannot be compiled
configure: WARNING: stdalign.h:     check for missing prerequisite headers?
configure: WARNING: stdalign.h: see the Autoconf documentation
configure: WARNING: stdalign.h:     section "Present But Cannot Be Compiled"
configure: WARNING: stdalign.h: proceeding with the compiler's result
configure: WARNING: sys/time.h: present but cannot be compiled
configure: WARNING: sys/time.h:     check for missing prerequisite headers?
configure: WARNING: sys/time.h: see the Autoconf documentation
configure: WARNING: sys/time.h:     section "Present But Cannot Be Compiled"
configure: WARNING: sys/time.h: proceeding with the compiler's result
configure: WARNING: unrecognized options: --with-multisubdir, --with-multisrctop
configure: WARNING: unrecognized options: --with-multisubdir, --with-multisrctop
configure: WARNING: stdbool.h: present but cannot be compiled
configure: WARNING: stdbool.h:     check for missing prerequisite headers?
configure: WARNING: stdbool.h: see the Autoconf documentation
configure: WARNING: stdbool.h:     section "Present But Cannot Be Compiled"
configure: WARNING: stdbool.h: proceeding with the compiler's result
configure: WARNING: stdalign.h: present but cannot be compiled
configure: WARNING: stdalign.h:     check for missing prerequisite headers?
configure: WARNING: stdalign.h: see the Autoconf documentation
configure: WARNING: stdalign.h:     section "Present But Cannot Be Compiled"
configure: WARNING: stdalign.h: proceeding with the compiler's result
configure: WARNING: sys/time.h: present but cannot be compiled
configure: WARNING: sys/time.h:     check for missing prerequisite headers?
configure: WARNING: sys/time.h: see the Autoconf documentation
configure: WARNING: sys/time.h:     section "Present But Cannot Be Compiled"
configure: WARNING: sys/time.h: proceeding with the compiler's result
configure: WARNING: unrecognized options: --with-multisubdir, --with-multisrctop
checking build system type... x86_64-unknown-linux-gnu
checking host system type... x86_64-unknown-linux-gnu
checking target system type... i586-pc-elf
checking for a BSD-compatible install... /usr/bin/install -c
checking whether build environment is sane... yes
checking for a thread-safe mkdir -p... /bin/mkdir -p
checking for gawk... gawk
checking whether make sets $(MAKE)... yes
checking for gcc... gcc
checking for C compiler default output file name... a.out
checking whether the C compiler works... yes
checking whether we are cross compiling... no
checking for suffix of executables... 
checking for suffix of object files... o
checking whether we are using the GNU C compiler... yes
checking whether gcc accepts -g... yes
checking for gcc option to accept ISO C89... none needed
checking for g++... g++
checking whether we are using the GNU C++ compiler... yes
checking whether g++ accepts -g... yes
checking how to run the C preprocessor... gcc -E
checking for grep that handles long lines and -e... /bin/grep
checking for egrep... /bin/grep -E
checking whether ln -s works... yes
checking for as... as
checking for ar... ar
checking for ranlib... ranlib
checking whether to enable maintainer-specific portions of Makefiles... no
configure: CPU config directory is cpu/i486
configure: OS config directory is os/gnu-linux
checking how to print strings... printf
checking for a sed that does not truncate output... /bin/sed
checking for fgrep... /bin/grep -F
checking for ld used by gcc... /usr/bin/ld
checking if the linker (/usr/bin/ld) is GNU ld... yes
checking for BSD- or MS-compatible name lister (nm)... /usr/bin/nm -B
checking the name lister (/usr/bin/nm -B) interface... BSD nm
checking the maximum length of command line arguments... 1572864
checking whether the shell understands some XSI constructs... yes
checking whether the shell understands "+="... yes
checking for /usr/bin/ld option to reload object files... -r
checking for objdump... objdump
checking how to recognize dependent libraries... pass_all
checking for ar... (cached) ar
checking for strip... strip
checking for ranlib... (cached) ranlib
checking command to parse /usr/bin/nm -B output from gcc object... ok
checking for ANSI C header files... yes
checking for sys/types.h... yes
checking for sys/stat.h... yes
checking for stdlib.h... yes
checking for string.h... yes
checking for memory.h... yes
checking for strings.h... yes
checking for inttypes.h... yes
checking for stdint.h... yes
checking for unistd.h... yes
checking for dlfcn.h... yes
checking for objdir... .libs
checking if gcc supports -fno-rtti -fno-exceptions... no
checking for gcc option to produce PIC... -fPIC -DPIC
checking if gcc PIC flag -fPIC -DPIC works... yes
checking if gcc static flag -static works... yes
checking if gcc supports -c -o file.o... yes
checking if gcc supports -c -o file.o... (cached) yes
checking whether the gcc linker (/usr/bin/ld -m elf_x86_64) supports shared libraries... yes
checking whether -lc should be explicitly linked in... no
checking dynamic linker characteristics... GNU/Linux ld.so
checking how to hardcode library paths into programs... immediate
checking for shl_load... no
checking for shl_load in -ldld... no
checking for dlopen... no
checking for dlopen in -ldl... yes
checking whether a program can dlopen itself... yes
checking whether a statically linked program can dlopen itself... no
checking whether stripping libraries is possible... yes
checking if libtool supports shared libraries... yes
checking whether to build shared libraries... yes
checking whether to build static libraries... yes
checking how to run the C++ preprocessor... g++ -E
checking for ld used by g++... /usr/bin/ld -m elf_x86_64
checking if the linker (/usr/bin/ld -m elf_x86_64) is GNU ld... yes
checking whether the g++ linker (/usr/bin/ld -m elf_x86_64) supports shared libraries... yes
checking for g++ option to produce PIC... -fPIC -DPIC
checking if g++ PIC flag -fPIC -DPIC works... yes
checking if g++ static flag -static works... yes
checking if g++ supports -c -o file.o... yes
checking if g++ supports -c -o file.o... (cached) yes
checking whether the g++ linker (/usr/bin/ld -m elf_x86_64) supports shared libraries... yes
checking dynamic linker characteristics... (cached) GNU/Linux ld.so
checking how to hardcode library paths into programs... immediate
configure: Only freestanding libraries will be built
configure: verbose termination messages are disabled
checking for exception model to use... call frame
checking for enabled PCH... no
checking for thread model used by GCC... posix
checking for atomic builtins for bool... no
checking for atomic builtins for short... yes
checking for atomic builtins for int... yes
checking for atomic builtins for long long... yes
checking for ISO/IEC TR 24733 ... yes
checking for __int128... yes
checking for __float128... yes
checking for g++ that supports -ffunction-sections -fdata-sections... yes
checking for underlying I/O to use... stdio
checking for C locale to use... generic
checking for std::allocator base class... new
configure: "C" header strategy set to c_global
checking for enabled long long specializations... yes
checking wchar.h usability... yes
checking wchar.h presence... yes
checking for wchar.h... yes
checking for mbstate_t... yes
checking wctype.h usability... yes
checking wctype.h presence... yes
checking for wctype.h... yes
checking for enabled wchar_t specializations... yes
checking for sin in -lm... yes
checking for ISO C99 support in <math.h>... yes
checking tgmath.h usability... yes
checking tgmath.h presence... yes
checking for tgmath.h... yes
checking complex.h usability... yes
checking complex.h presence... yes
checking for complex.h... yes
checking for ISO C99 support in <complex.h>... yes
checking for ISO C99 support in <stdio.h>... yes
checking for ISO C99 support in <stdlib.h>... yes
checking for ISO C99 support in <wchar.h>... yes
checking for fully enabled ISO C99 support... yes
configure: Debug build flags set to -gdwarf-4 -g3 -O0
checking for additional debug build... no
configure: target-libgomp not built
checking for parallel mode support... no
checking for extra compiler flags for building... -fno-rtti -fno-exceptions -std=c++03
checking for extern template support... yes
checking for custom python install directory... no
checking for -Werror... yes
checking for vtable verify support... no
checking for gets declaration... no
checking for EOWNERDEAD... yes
checking for ENOTRECOVERABLE... yes
checking for ENOLINK... yes
checking for EPROTO... yes
checking for ENODATA... yes
checking for ENOSR... yes
checking for ENOSTR... yes
checking for ETIME... yes
checking for EBADMSG... yes
checking for ECANCELED... yes
checking for EOVERFLOW... yes
checking for ENOTSUP... yes
checking for EIDRM... yes
checking for ETXTBSY... yes
checking for ECHILD... yes
checking for ENOSPC... yes
checking for EPERM... yes
checking for ETIMEDOUT... yes
checking for EWOULDBLOCK... yes
checking for int64_t... yes
checking for int64_t as long... yes
checking for int64_t as long long... checking for LFS support... yes
checking sys/ioctl.h usability... yes
checking sys/ioctl.h presence... yes
checking for sys/ioctl.h... yes
checking sys/filio.h usability... no
checking sys/filio.h presence... no
checking for sys/filio.h... no
checking for poll... yes
checking for S_ISREG or S_IFREG... S_ISREG
checking sys/uio.h usability... yes
checking sys/uio.h presence... yes
checking for sys/uio.h... yes
checking for writev... yes
checking fenv.h usability... yes
checking fenv.h presence... yes
checking for fenv.h... yes
checking for complex.h... (cached) yes
checking for complex.h... (cached) yes
checking for ISO C99 support to TR1 in <complex.h>... yes
checking for ISO C99 support to TR1 in <ctype.h>... yes
checking for fenv.h... (cached) yes
checking for ISO C99 support to TR1 in <fenv.h>... yes
checking for ISO C99 support to TR1 in <stdint.h>... yes
checking for ISO C99 support to TR1 in <math.h>... yes
checking for ISO C99 support to TR1 in <inttypes.h>... yes
checking for wchar_t ISO C99 support to TR1 in <inttypes.h>... yes
checking stdbool.h usability... yes
checking stdbool.h presence... yes
checking for stdbool.h... yes
checking stdalign.h usability... yes
checking stdalign.h presence... yes
checking for stdalign.h... yes
checking for the value of EOF... -1
checking for the value of SEEK_CUR... 1
checking for the value of SEEK_END... 2
checking for gettimeofday... checking sys/time.h usability... yes
checking sys/time.h presence... yes
checking for sys/time.h... yes
checking for gettimeofday... yes
checking for sleep... yes
checking for usleep... yes
checking for tmpnam... yes
checking locale.h usability... yes
checking locale.h presence... yes
checking for locale.h... yes
checking for LC_MESSAGES... yes
checking sys/sysinfo.h usability... yes
checking sys/sysinfo.h presence... yes
checking for sys/sysinfo.h... yes
checking for get_nprocs... yes
checking for unistd.h... (cached) yes
checking for _SC_NPROCESSORS_ONLN... yes
checking for _SC_NPROC_ONLN... no
checking for pthreads_num_processors_np... no
checking for hw.ncpu sysctl... no
checking for suitable sys/sdt.h... no
checking endian.h usability... yes
checking endian.h presence... yes
checking for endian.h... yes
checking execinfo.h usability... yes
checking execinfo.h presence... yes
checking for execinfo.h... yes
checking float.h usability... yes
checking float.h presence... yes
checking for float.h... yes
checking fp.h usability... no
checking fp.h presence... no
checking for fp.h... no
checking ieeefp.h usability... no
checking ieeefp.h presence... no
checking for ieeefp.h... no
checking for inttypes.h... (cached) yes
checking for locale.h... (cached) yes
checking machine/endian.h usability... no
checking machine/endian.h presence... no
checking for machine/endian.h... no
checking machine/param.h usability... no
checking machine/param.h presence... no
checking for machine/param.h... no
checking nan.h usability... no
checking nan.h presence... no
checking for nan.h... no
checking for stdint.h... (cached) yes
checking for stdlib.h... (cached) yes
checking for string.h... (cached) yes
checking for strings.h... (cached) yes
checking sys/ipc.h usability... yes
checking sys/ipc.h presence... yes
checking for sys/ipc.h... yes
checking sys/isa_defs.h usability... no
checking sys/isa_defs.h presence... no
checking for sys/isa_defs.h... no
checking sys/machine.h usability... no
checking sys/machine.h presence... no
checking for sys/machine.h... no
checking sys/param.h usability... yes
checking sys/param.h presence... yes
checking for sys/param.h... yes
checking sys/resource.h usability... yes
checking sys/resource.h presence... yes
checking for sys/resource.h... yes
checking sys/sem.h usability... yes
checking sys/sem.h presence... yes
checking for sys/sem.h... yes
checking for sys/stat.h... (cached) yes
checking for sys/time.h... (cached) yes
checking for sys/types.h... (cached) yes
checking for unistd.h... (cached) yes
checking for wchar.h... (cached) yes
checking for wctype.h... (cached) yes
checking for ld used by gcc... (cached) /usr/bin/ld
checking if the linker (/usr/bin/ld) is GNU ld... (cached) yes
checking for ld version... 22901
checking for ld that supports -Wl,--gc-sections... yes
checking for ld that supports -Wl,-z,relro... yes
checking for sin in -lm... (cached) yes
checking for isinf declaration... yes
checking for isinf... yes
checking for isnan declaration... yes
checking for isnan... yes
checking for finite declaration... yes
checking for finite... yes
checking for sincos declaration... yes
checking for sincos... yes
checking for fpclass declaration... no
checking for _fpclass declaration... no
checking for qfpclass declaration... no
checking for _qfpclass declaration... no
checking for hypot declaration... yes
checking for hypot... yes
checking for float trig functions... yes
checking for acosf... yes
checking for asinf... yes
checking for atanf... yes
checking for cosf... yes
checking for sinf... yes
checking for tanf... yes
checking for coshf... yes
checking for sinhf... yes
checking for tanhf... yes
checking for float round functions... yes
checking for ceilf... yes
checking for floorf... yes
checking for expf declaration... yes
checking for expf... yes
checking for isnanf declaration... yes
checking for isnanf... yes
checking for isinff declaration... yes
checking for isinff... yes
checking for atan2f declaration... yes
checking for atan2f... yes
checking for fabsf declaration... yes
checking for fabsf... yes
checking for fmodf declaration... yes
checking for fmodf... yes
checking for frexpf declaration... yes
checking for frexpf... yes
checking for hypotf declaration... yes
checking for hypotf... yes
checking for ldexpf declaration... yes
checking for ldexpf... yes
checking for logf declaration... yes
checking for logf... yes
checking for log10f declaration... yes
checking for log10f... yes
checking for modff declaration... yes
checking for modff... yes
checking for modf declaration... no
checking for _modf declaration... no
checking for powf declaration... yes
checking for powf... yes
checking for sqrtf declaration... yes
checking for sqrtf... yes
checking for sincosf declaration... yes
checking for sincosf... yes
checking for finitef declaration... yes
checking for finitef... yes
checking for long double trig functions... yes
checking for acosl... yes
checking for asinl... yes
checking for atanl... yes
checking for cosl... yes
checking for sinl... yes
checking for tanl... yes
checking for coshl... yes
checking for sinhl... yes
checking for tanhl... yes
checking for long double round functions... yes
checking for ceill... yes
checking for floorl... yes
checking for isnanl declaration... yes
checking for isnanl... yes
checking for isinfl declaration... yes
checking for isinfl... yes
checking for atan2l declaration... yes
checking for atan2l... yes
checking for expl declaration... yes
checking for expl... yes
checking for fabsl declaration... yes
checking for fabsl... yes
checking for fmodl declaration... yes
checking for fmodl... yes
checking for frexpl declaration... yes
checking for frexpl... yes
checking for hypotl declaration... yes
checking for hypotl... yes
checking for ldexpl declaration... yes
checking for ldexpl... yes
checking for logl declaration... yes
checking for logl... yes
checking for log10l declaration... yes
checking for log10l... yes
checking for modfl declaration... yes
checking for modfl... yes
checking for powl declaration... yes
checking for powl... yes
checking for sqrtl declaration... yes
checking for sqrtl... yes
checking for sincosl declaration... yes
checking for sincosl... yes
checking for finitel declaration... yes
checking for finitel... yes
checking for at_quick_exit declaration... yes
checking for at_quick_exit... yes
checking for quick_exit declaration... yes
checking for quick_exit... yes
checking for strtold declaration... yes
checking for strtold... yes
checking for strtof declaration... yes
checking for strtof... yes
checking for "/dev/random" and "/dev/urandom" for TR1 random_device... yes
checking whether the target supports thread-local storage... yes
checking for __cxa_thread_atexit_impl... yes
checking for ld used by GCC... /usr/bin/ld
checking if the linker (/usr/bin/ld) is GNU ld... yes
checking for shared library run path origin... done
checking for iconv... yes
checking for iconv declaration... install-shextern size_t iconv (iconv_t cd, char * *inbuf, size_t *inbytesleft, char * *outbuf, size_t *outbytesleft);
looking for a compliant stdint.h in stdint.h, checking for uintmax_t... yes
checking for uintptr_t... yes
checking for int_least32_t... yes
checking for int_fast32_t... yes
checking for uint64_t... yes
checking what to include in include/gstdint.h... stdint.h (already complete)
checking for GNU c++filt... /usr/bin/c++filt
checking for shared libgcc... yes
checking whether the target supports .symver directive... yes
configure: versioning on shared library symbols is gnu
checking for size_t as unsigned int... no
checking for ptrdiff_t as int... no
checking whether the target supports hidden visibility... yes
configure: visibility supported: yes
checking for default std::string ABI to use... gcc4-compatible
checking for rdrand support in assembler... yes
checking for unistd.h... (cached) yes
checking for sys/time.h... (cached) yes
checking for sys/resource.h... (cached) yes
checking for RLIMIT_DATA... yes
checking for RLIMIT_RSS... yes
checking for RLIMIT_VMEM... no
checking for RLIMIT_AS... yes
checking for RLIMIT_FSIZE... yes
checking for testsuite resource limits support... yes
checking for setenv declaration... yes
checking for setenv... yes
checking whether it can be safely assumed that mutex_timedlock is available... yes
checking for gthreads library... no
checking fcntl.h usability... yes
checking fcntl.h presence... yes
checking for fcntl.h... yes
checking dirent.h usability... yes
checking dirent.h presence... yes
checking for dirent.h... yes
checking sys/statvfs.h usability... yes
checking sys/statvfs.h presence... yes
checking for sys/statvfs.h... yes
checking utime.h usability... yes
checking utime.h presence... yes
checking for utime.h... yes
checking whether to build Filesystem TS support... no
checking for struct dirent.d_type... yes
checking for realpath... yes
checking for utimensat... yes
checking for struct stat.st_mtim.tv_nsec... yes
checking for fchmod... yes
checking for fchmodat... yes
checking for sendfile that can copy files... no
checking for makeinfo... ${SHELL} /data105/extractors/gcc-5.5.0/missing --run makeinfo
checking for modern makeinfo... no
checking for doxygen... no
checking for dot... no
checking for xsltproc... no
checking for xmllint... yes
checking for docbook stylesheets for documentation creation... no
checking for local stylesheet directory... no
checking for epub3 stylesheets for documentation creation... no
checking for dblatex... no
checking for pdflatex... no
checking for gxx-include-dir... no
checking for --enable-version-specific-runtime-libs... no
checking for install location... ${prefix}/include/c++/${gcc_version}
configure: creating ./config.status
config.status: creating Makefile
config.status: creating scripts/testsuite_flags
config.status: creating scripts/extract_symvers
config.status: creating doc/xsl/customization.xsl
config.status: creating include/Makefile
Adding multilib support to include/Makefile in ../gcc-5.5.0/libstdc++-v3
multidirs=32 x32
with_multisubdir=
config.status: creating libsupc++/Makefile
Adding multilib support to libsupc++/Makefile in ../gcc-5.5.0/libstdc++-v3
multidirs=32 x32
with_multisubdir=
config.status: creating src/Makefile
Adding multilib support to src/Makefile in ../gcc-5.5.0/libstdc++-v3
multidirs=32 x32
with_multisubdir=
config.status: creating src/c++98/Makefile
Adding multilib support to src/c++98/Makefile in ../gcc-5.5.0/libstdc++-v3
multidirs=32 x32
with_multisubdir=
config.status: creating src/c++11/Makefile
Adding multilib support to src/c++11/Makefile in ../gcc-5.5.0/libstdc++-v3
multidirs=32 x32
with_multisubdir=
config.status: creating src/filesystem/Makefile
Adding multilib support to src/filesystem/Makefile in ../gcc-5.5.0/libstdc++-v3
multidirs=32 x32
with_multisubdir=
config.status: creating doc/Makefile
Adding multilib support to doc/Makefile in ../gcc-5.5.0/libstdc++-v3
multidirs=32 x32
with_multisubdir=
config.status: creating po/Makefile
Adding multilib support to po/Makefile in ../gcc-5.5.0/libstdc++-v3
multidirs=32 x32
with_multisubdir=
config.status: creating testsuite/Makefile
Adding multilib support to testsuite/Makefile in ../gcc-5.5.0/libstdc++-v3
multidirs=32 x32
with_multisubdir=
config.status: creating python/Makefile
Adding multilib support to python/Makefile in ../gcc-5.5.0/libstdc++-v3
multidirs=32 x32
with_multisubdir=
config.status: creating config.h
config.status: config.h is unchanged
config.status: executing default-1 commands
Adding multilib support to Makefile in ../gcc-5.5.0/libstdc++-v3
multidirs=32 x32
with_multisubdir=
Running configure in multilib subdirs 32 x32
pwd: /data105/extractors/hi586-elf-libstdcxx-v3
Running configure in multilib subdir 32
pwd: /data105/extractors
checking build system type... x86_64-unknown-linux-gnu
checking host system type... x86_64-unknown-linux-gnu
checking target system type... i586-pc-elf
checking for a BSD-compatible install... /usr/bin/install -c
checking whether build environment is sane... yes
checking for strip... strip
checking for a thread-safe mkdir -p... /bin/mkdir -p
checking for gawk... gawk
checking whether make sets $(MAKE)... yes
checking for gcc... gcc  -m32
checking for C compiler default output file name... a.out
checking whether the C compiler works... yes
checking whether we are cross compiling... no
checking for suffix of executables... 
checking for suffix of object files... o
checking whether we are using the GNU C compiler... yes
checking whether gcc  -m32 accepts -g... yes
checking for gcc  -m32 option to accept ISO C89... none needed
checking whether we are using the GNU C++ compiler... yes
checking whether g++  -m32 accepts -g... yes
checking how to run the C preprocessor... gcc  -m32 -E
checking for grep that handles long lines and -e... /bin/grep
checking for egrep... /bin/grep -E
checking whether ln -s works... yes
checking for as... as
checking for ar... ar
checking for ranlib... ranlib
checking whether to enable maintainer-specific portions of Makefiles... no
configure: CPU config directory is cpu/i486
configure: OS config directory is os/gnu-linux
checking how to print strings... printf
checking for a sed that does not truncate output... /bin/sed
checking for fgrep... /bin/grep -F
checking for ld used by gcc  -m32... /usr/bin/ld
checking if the linker (/usr/bin/ld) is GNU ld... yes
checking for BSD- or MS-compatible name lister (nm)... /usr/bin/nm -B
checking the name lister (/usr/bin/nm -B) interface... BSD nm
checking the maximum length of command line arguments... 1572864
checking whether the shell understands some XSI constructs... yes
checking whether the shell understands "+="... yes
checking for /usr/bin/ld option to reload object files... -r
checking for objdump... objdump
checking how to recognize dependent libraries... pass_all
checking for ar... (cached) ar
checking for strip... (cached) strip
checking for ranlib... (cached) ranlib
checking command to parse /usr/bin/nm -B output from gcc  -m32 object... ok
checking for ANSI C header files... yes
checking for sys/types.h... yes
checking for sys/stat.h... yes
checking for stdlib.h... yes
checking for string.h... yes
checking for memory.h... yes
checking for strings.h... yes
checking for inttypes.h... yes
checking for stdint.h... yes
checking for unistd.h... yes
checking for dlfcn.h... yes
checking for objdir... .libs
checking if gcc  -m32 supports -fno-rtti -fno-exceptions... no
checking for gcc  -m32 option to produce PIC... -fPIC -DPIC
checking if gcc  -m32 PIC flag -fPIC -DPIC works... yes
checking if gcc  -m32 static flag -static works... yes
checking if gcc  -m32 supports -c -o file.o... yes
checking if gcc  -m32 supports -c -o file.o... (cached) yes
checking whether the gcc  -m32 linker (/usr/bin/ld -m elf_i386) supports shared libraries... yes
checking whether -lc should be explicitly linked in... no
checking dynamic linker characteristics... GNU/Linux ld.so
checking how to hardcode library paths into programs... immediate
checking for shl_load... no
checking for shl_load in -ldld... no
checking for dlopen... no
checking for dlopen in -ldl... yes
checking whether a program can dlopen itself... yes
checking whether a statically linked program can dlopen itself... no
checking whether stripping libraries is possible... yes
checking if libtool supports shared libraries... yes
checking whether to build shared libraries... yes
checking whether to build static libraries... yes
checking how to run the C++ preprocessor... g++  -m32 -E
checking for ld used by g++  -m32... /usr/bin/ld -m elf_i386
checking if the linker (/usr/bin/ld -m elf_i386) is GNU ld... yes
checking whether the g++  -m32 linker (/usr/bin/ld -m elf_i386) supports shared libraries... yes
checking for g++  -m32 option to produce PIC... -fPIC -DPIC
checking if g++  -m32 PIC flag -fPIC -DPIC works... yes
checking if g++  -m32 static flag -static works... no
checking if g++  -m32 supports -c -o file.o... yes
checking if g++  -m32 supports -c -o file.o... (cached) yes
checking whether the g++  -m32 linker (/usr/bin/ld -m elf_i386) supports shared libraries... yes
checking dynamic linker characteristics... (cached) GNU/Linux ld.so
checking how to hardcode library paths into programs... immediate
configure: Only freestanding libraries will be built
configure: verbose termination messages are disabled
checking for exception model to use... call frame
checking for enabled PCH... no
checking for thread model used by GCC... posix
checking for atomic builtins for bool... no
checking for atomic builtins for short... no
checking for atomic builtins for int... no
checking for atomic builtins for long long... no
checking for ISO/IEC TR 24733 ... yes
checking for __int128... no
checking for __float128... yes
checking for g++ that supports -ffunction-sections -fdata-sections... yes
checking for underlying I/O to use... stdio
checking for C locale to use... generic
checking for std::allocator base class... new
configure: "C" header strategy set to c_global
checking for enabled long long specializations... yes
checking wchar.h usability... yes
checking wchar.h presence... yes
checking for wchar.h... yes
checking for mbstate_t... yes
checking wctype.h usability... yes
checking wctype.h presence... yes
checking for wctype.h... yes
checking for enabled wchar_t specializations... yes
checking for sin in -lm... no
checking for ISO C99 support in <math.h>... no
checking tgmath.h usability... no
checking tgmath.h presence... no
checking for tgmath.h... no
checking complex.h usability... no
checking complex.h presence... no
checking for complex.h... no
no
checking for ISO C99 support in <stdio.h>... yes
checking for ISO C99 support in <stdlib.h>... no
checking for ISO C99 support in <wchar.h>... yes
checking for fully enabled ISO C99 support... no
configure: Debug build flags set to -gdwarf-4 -g3 -O0
checking for additional debug build... no
configure: target-libgomp not built
checking for parallel mode support... no
checking for extra compiler flags for building... -fno-rtti -fno-exceptions -std=c++03
checking for extern template support... yes
checking for custom python install directory... no
checking for -Werror... yes
checking for vtable verify support... no
checking for gets declaration... no
checking for EOWNERDEAD... yes
checking for ENOTRECOVERABLE... yes
checking for ENOLINK... yes
checking for EPROTO... yes
checking for ENODATA... yes
checking for ENOSR... yes
checking for ENOSTR... yes
checking for ETIME... yes
checking for EBADMSG... yes
checking for ECANCELED... yes
checking for EOVERFLOW... yes
checking for ENOTSUP... yes
checking for EIDRM... yes
checking for ETXTBSY... yes
checking for ECHILD... yes
checking for ENOSPC... yes
checking for EPERM... yes
checking for ETIMEDOUT... yes
checking for EWOULDBLOCK... yes
checking for int64_t... yes
checking for int64_t as long... checking for int64_t as long long... yes
checking for LFS support... no
checking sys/ioctl.h usability... yes
checking sys/ioctl.h presence... yes
checking for sys/ioctl.h... yes
checking sys/filio.h usability... no
checking sys/filio.h presence... no
checking for sys/filio.h... no
checking for poll... no
checking for S_ISREG or S_IFREG... no
checking sys/uio.h usability... yes
checking sys/uio.h presence... yes
checking for sys/uio.h... yes
checking for writev... no
checking fenv.h usability... yes
checking fenv.h presence... yes
checking for fenv.h... yes
checking for complex.h... (cached) no
checking for complex.h... (cached) no
no
checking for ISO C99 support to TR1 in <ctype.h>... yes
checking for fenv.h... (cached) yes
checking for ISO C99 support to TR1 in <fenv.h>... no
checking for ISO C99 support to TR1 in <stdint.h>... yes
checking for ISO C99 support to TR1 in <math.h>... no
checking for ISO C99 support to TR1 in <inttypes.h>... yes
checking for wchar_t ISO C99 support to TR1 in <inttypes.h>... yes
checking stdbool.h usability... no
checking stdbool.h presence... yes
checking for stdbool.h... no
checking stdalign.h usability... no
checking stdalign.h presence... yes
checking for stdalign.h... no
checking for the value of EOF... -1
checking for the value of SEEK_CUR... 1
checking for the value of SEEK_END... 2
checking for gettimeofday... checking sys/time.h usability... no
checking sys/time.h presence... yes
checking for sys/time.h... no
checking for sleep... yes
checking for usleep... yes
checking for tmpnam... no
checking locale.h usability... yes
checking locale.h presence... yes
checking for locale.h... yes
checking for LC_MESSAGES... yes
checking sys/sysinfo.h usability... yes
checking sys/sysinfo.h presence... yes
checking for sys/sysinfo.h... yes
checking for get_nprocs... no
checking for unistd.h... (cached) yes
checking for _SC_NPROCESSORS_ONLN... no
checking for _SC_NPROC_ONLN... no
checking for pthreads_num_processors_np... no
checking for hw.ncpu sysctl... no
checking for suitable sys/sdt.h... no
checking endian.h usability... yes
checking endian.h presence... yes
checking for endian.h... yes
checking execinfo.h usability... yes
checking execinfo.h presence... yes
checking for execinfo.h... yes
checking float.h usability... yes
checking float.h presence... yes
checking for float.h... yes
checking fp.h usability... no
checking fp.h presence... no
checking for fp.h... no
checking ieeefp.h usability... no
checking ieeefp.h presence... no
checking for ieeefp.h... no
checking for inttypes.h... (cached) yes
checking for locale.h... (cached) yes
checking machine/endian.h usability... no
checking machine/endian.h presence... no
checking for machine/endian.h... no
checking machine/param.h usability... no
checking machine/param.h presence... no
checking for machine/param.h... no
checking nan.h usability... no
checking nan.h presence... no
checking for nan.h... no
checking for stdint.h... (cached) yes
checking for stdlib.h... (cached) yes
checking for string.h... (cached) yes
checking for strings.h... (cached) yes
checking sys/ipc.h usability... yes
checking sys/ipc.h presence... yes
checking for sys/ipc.h... yes
checking sys/isa_defs.h usability... no
checking sys/isa_defs.h presence... no
checking for sys/isa_defs.h... no
checking sys/machine.h usability... no
checking sys/machine.h presence... no
checking for sys/machine.h... no
checking sys/param.h usability... yes
checking sys/param.h presence... yes
checking for sys/param.h... yes
checking sys/resource.h usability... yes
checking sys/resource.h presence... yes
checking for sys/resource.h... yes
checking sys/sem.h usability... yes
checking sys/sem.h presence... yes
checking for sys/sem.h... yes
checking for sys/stat.h... (cached) yes
checking for sys/time.h... (cached) no
checking for sys/types.h... (cached) yes
checking for unistd.h... (cached) yes
checking for wchar.h... (cached) yes
checking for wctype.h... (cached) yes
checking for ld used by gcc  -m32... (cached) /usr/bin/ld
checking if the linker (/usr/bin/ld) is GNU ld... (cached) yes
checking for ld version... 22901
checking for ld that supports -Wl,--gc-sections... yes
checking for ld that supports -Wl,-z,relro... yes
checking for sin in -lm... (cached) no
checking for isinf declaration... no
checking for _isinf declaration... no
checking for isnan declaration... no
checking for _isnan declaration... no
checking for finite declaration... no
checking for _finite declaration... no
checking for sincos declaration... no
checking for _sincos declaration... no
checking for fpclass declaration... no
checking for _fpclass declaration... no
checking for qfpclass declaration... no
checking for _qfpclass declaration... no
checking for hypot declaration... no
checking for _hypot declaration... no
checking for float trig functions... no
checking for _float trig functions... no
checking for float round functions... no
checking for _float round functions... no
checking for expf declaration... no
checking for _expf declaration... no
checking for isnanf declaration... no
checking for _isnanf declaration... no
checking for isinff declaration... no
checking for _isinff declaration... no
checking for atan2f declaration... no
checking for _atan2f declaration... no
checking for fabsf declaration... no
checking for _fabsf declaration... no
checking for fmodf declaration... no
checking for _fmodf declaration... no
checking for frexpf declaration... no
checking for _frexpf declaration... no
checking for hypotf declaration... no
checking for _hypotf declaration... no
checking for ldexpf declaration... no
checking for _ldexpf declaration... no
checking for logf declaration... no
checking for _logf declaration... no
checking for log10f declaration... no
checking for _log10f declaration... no
checking for modff declaration... no
checking for _modff declaration... no
checking for modf declaration... no
checking for _modf declaration... no
checking for powf declaration... no
checking for _powf declaration... no
checking for sqrtf declaration... no
checking for _sqrtf declaration... no
checking for sincosf declaration... no
checking for _sincosf declaration... no
checking for finitef declaration... no
checking for _finitef declaration... no
checking for long double trig functions... no
checking for _long double trig functions... no
checking for long double round functions... no
checking for _long double round functions... no
checking for isnanl declaration... no
checking for _isnanl declaration... no
checking for isinfl declaration... no
checking for _isinfl declaration... no
checking for atan2l declaration... no
checking for _atan2l declaration... no
checking for expl declaration... no
checking for _expl declaration... no
checking for fabsl declaration... no
checking for _fabsl declaration... no
checking for fmodl declaration... no
checking for _fmodl declaration... no
checking for frexpl declaration... no
checking for _frexpl declaration... no
checking for hypotl declaration... no
checking for _hypotl declaration... no
checking for ldexpl declaration... no
checking for _ldexpl declaration... no
checking for logl declaration... no
checking for _logl declaration... no
checking for log10l declaration... no
checking for _log10l declaration... no
checking for modfl declaration... no
checking for _modfl declaration... no
checking for powl declaration... no
checking for _powl declaration... no
checking for sqrtl declaration... no
checking for _sqrtl declaration... no
checking for sincosl declaration... no
checking for _sincosl declaration... no
checking for finitel declaration... no
checking for _finitel declaration... no
checking for at_quick_exit declaration... no
checking for quick_exit declaration... no
checking for strtold declaration... no
checking for strtof declaration... no
checking for "/dev/random" and "/dev/urandom" for TR1 random_device... yes
checking whether the target supports thread-local storage... yes
checking for __cxa_thread_atexit_impl... yes
checking for ld used by GCC... /usr/bin/ld
checking if the linker (/usr/bin/ld) is GNU ld... yes
checking for shared library run path origin... done
checking for iconv... yes
checking for iconv declaration... install-shextern size_t iconv (iconv_t cd, char * *inbuf, size_t *inbytesleft, char * *outbuf, size_t *outbytesleft);
looking for a compliant stdint.h in stdint.h, checking for uintmax_t... yes
checking for uintptr_t... yes
checking for int_least32_t... yes
checking for int_fast32_t... yes
checking for uint64_t... yes
checking what to include in include/gstdint.h... stdint.h (already complete)
checking for GNU c++filt... /usr/bin/c++filt
checking for shared libgcc... yes
checking whether the target supports .symver directive... yes
configure: versioning on shared library symbols is gnu
checking for size_t as unsigned int... yes
checking for ptrdiff_t as int... yes
checking whether the target supports hidden visibility... yes
configure: visibility supported: yes
checking for default std::string ABI to use... gcc4-compatible
checking for rdrand support in assembler... yes
checking for unistd.h... (cached) yes
checking for sys/time.h... (cached) no
checking for sys/resource.h... (cached) yes
checking for testsuite resource limits support... no
checking for setenv declaration... no
checking whether it can be safely assumed that mutex_timedlock is available... yes
checking for gthreads library... no
checking fcntl.h usability... yes
checking fcntl.h presence... yes
checking for fcntl.h... yes
checking dirent.h usability... yes
checking dirent.h presence... yes
checking for dirent.h... yes
checking sys/statvfs.h usability... yes
checking sys/statvfs.h presence... yes
checking for sys/statvfs.h... yes
checking utime.h usability... yes
checking utime.h presence... yes
checking for utime.h... yes
checking whether to build Filesystem TS support... no
checking for struct dirent.d_type... no
checking for realpath... no
checking for utimensat... no
checking for struct stat.st_mtim.tv_nsec... no
checking for fchmod... no
checking for fchmodat... no
checking for sendfile that can copy files... no
checking for makeinfo... ${SHELL} /data105/extractors/gcc-5.5.0/missing --run makeinfo
checking for modern makeinfo... no
checking for doxygen... no
checking for dot... no
checking for xsltproc... no
checking for xmllint... yes
checking for docbook stylesheets for documentation creation... no
checking for local stylesheet directory... no
checking for epub3 stylesheets for documentation creation... no
checking for dblatex... no
checking for pdflatex... no
checking for gxx-include-dir... no
checking for --enable-version-specific-runtime-libs... no
checking for install location... ${prefix}/include/c++/${gcc_version}
configure: creating ./config.status
config.status: creating Makefile
config.status: creating scripts/testsuite_flags
config.status: creating scripts/extract_symvers
config.status: creating doc/xsl/customization.xsl
config.status: creating include/Makefile
Adding multilib support to include/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=32
config.status: creating libsupc++/Makefile
Adding multilib support to libsupc++/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=32
config.status: creating src/Makefile
Adding multilib support to src/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=32
config.status: creating src/c++98/Makefile
Adding multilib support to src/c++98/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=32
config.status: creating src/c++11/Makefile
Adding multilib support to src/c++11/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=32
config.status: creating src/filesystem/Makefile
Adding multilib support to src/filesystem/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=32
config.status: creating doc/Makefile
Adding multilib support to doc/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=32
config.status: creating po/Makefile
Adding multilib support to po/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=32
config.status: creating testsuite/Makefile
Adding multilib support to testsuite/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=32
config.status: creating python/Makefile
Adding multilib support to python/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=32
config.status: creating config.h
config.status: config.h is unchanged
config.status: executing default-1 commands
Adding multilib support to Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=32
config.status: executing libtool commands
config.status: executing include/gstdint.h commands
config.status: executing generate-headers commands
make: Nothing to be done for 'all'.
Running configure in multilib subdir x32
pwd: /data105/extractors
checking build system type... x86_64-unknown-linux-gnu
checking host system type... x86_64-unknown-linux-gnu
checking target system type... i586-pc-elf
checking for a BSD-compatible install... /usr/bin/install -c
checking whether build environment is sane... yes
checking for strip... strip
checking for a thread-safe mkdir -p... /bin/mkdir -p
checking for gawk... gawk
checking whether make sets $(MAKE)... yes
checking for gcc... gcc  -mx32
checking for C compiler default output file name... a.out
checking whether the C compiler works... yes
checking whether we are cross compiling... no
checking for suffix of executables... 
checking for suffix of object files... o
checking whether we are using the GNU C compiler... yes
checking whether gcc  -mx32 accepts -g... yes
checking for gcc  -mx32 option to accept ISO C89... none needed
checking whether we are using the GNU C++ compiler... yes
checking whether g++  -mx32 accepts -g... yes
checking how to run the C preprocessor... gcc  -mx32 -E
checking for grep that handles long lines and -e... /bin/grep
checking for egrep... /bin/grep -E
checking whether ln -s works... yes
checking for as... as
checking for ar... ar
checking for ranlib... ranlib
checking whether to enable maintainer-specific portions of Makefiles... no
configure: CPU config directory is cpu/i486
configure: OS config directory is os/gnu-linux
checking how to print strings... printf
checking for a sed that does not truncate output... /bin/sed
checking for fgrep... /bin/grep -F
checking for ld used by gcc  -mx32... /usr/bin/ld
checking if the linker (/usr/bin/ld) is GNU ld... yes
checking for BSD- or MS-compatible name lister (nm)... /usr/bin/nm -B
checking the name lister (/usr/bin/nm -B) interface... BSD nm
checking the maximum length of command line arguments... 1572864
checking whether the shell understands some XSI constructs... yes
checking whether the shell understands "+="... yes
checking for /usr/bin/ld option to reload object files... -r
checking for objdump... objdump
checking how to recognize dependent libraries... pass_all
checking for ar... (cached) ar
checking for strip... (cached) strip
checking for ranlib... (cached) ranlib
checking command to parse /usr/bin/nm -B output from gcc  -mx32 object... ok
checking for ANSI C header files... yes
checking for sys/types.h... yes
checking for sys/stat.h... yes
checking for stdlib.h... yes
checking for string.h... yes
checking for memory.h... yes
checking for strings.h... yes
checking for inttypes.h... yes
checking for stdint.h... yes
checking for unistd.h... yes
checking for dlfcn.h... yes
checking for objdir... .libs
checking if gcc  -mx32 supports -fno-rtti -fno-exceptions... no
checking for gcc  -mx32 option to produce PIC... -fPIC -DPIC
checking if gcc  -mx32 PIC flag -fPIC -DPIC works... yes
checking if gcc  -mx32 static flag -static works... yes
checking if gcc  -mx32 supports -c -o file.o... yes
checking if gcc  -mx32 supports -c -o file.o... (cached) yes
checking whether the gcc  -mx32 linker (/usr/bin/ld -m elf32_x86_64) supports shared libraries... yes
checking whether -lc should be explicitly linked in... no
checking dynamic linker characteristics... GNU/Linux ld.so
checking how to hardcode library paths into programs... immediate
checking for shl_load... no
checking for shl_load in -ldld... no
checking for dlopen... no
checking for dlopen in -ldl... yes
checking whether a program can dlopen itself... yes
checking whether a statically linked program can dlopen itself... no
checking whether stripping libraries is possible... yes
checking if libtool supports shared libraries... yes
checking whether to build shared libraries... yes
checking whether to build static libraries... yes
checking how to run the C++ preprocessor... g++  -mx32 -E
checking for ld used by g++  -mx32... /usr/bin/ld -m elf32_x86_64
checking if the linker (/usr/bin/ld -m elf32_x86_64) is GNU ld... yes
checking whether the g++  -mx32 linker (/usr/bin/ld -m elf32_x86_64) supports shared libraries... yes
checking for g++  -mx32 option to produce PIC... -fPIC -DPIC
checking if g++  -mx32 PIC flag -fPIC -DPIC works... yes
checking if g++  -mx32 static flag -static works... no
checking if g++  -mx32 supports -c -o file.o... yes
checking if g++  -mx32 supports -c -o file.o... (cached) yes
checking whether the g++  -mx32 linker (/usr/bin/ld -m elf32_x86_64) supports shared libraries... yes
checking dynamic linker characteristics... (cached) GNU/Linux ld.so
checking how to hardcode library paths into programs... immediate
configure: Only freestanding libraries will be built
configure: verbose termination messages are disabled
checking for exception model to use... call frame
checking for enabled PCH... no
checking for thread model used by GCC... posix
checking for atomic builtins for bool... no
checking for atomic builtins for short... no
checking for atomic builtins for int... no
checking for atomic builtins for long long... no
checking for ISO/IEC TR 24733 ... yes
checking for __int128... yes
checking for __float128... yes
checking for g++ that supports -ffunction-sections -fdata-sections... yes
checking for underlying I/O to use... stdio
checking for C locale to use... generic
checking for std::allocator base class... new
configure: "C" header strategy set to c_global
checking for enabled long long specializations... yes
checking wchar.h usability... yes
checking wchar.h presence... yes
checking for wchar.h... yes
checking for mbstate_t... yes
checking wctype.h usability... yes
checking wctype.h presence... yes
checking for wctype.h... yes
checking for enabled wchar_t specializations... yes
checking for sin in -lm... no
checking for ISO C99 support in <math.h>... no
checking tgmath.h usability... no
checking tgmath.h presence... no
checking for tgmath.h... no
checking complex.h usability... no
checking complex.h presence... no
checking for complex.h... no
no
checking for ISO C99 support in <stdio.h>... yes
checking for ISO C99 support in <stdlib.h>... no
checking for ISO C99 support in <wchar.h>... yes
checking for fully enabled ISO C99 support... no
configure: Debug build flags set to -gdwarf-4 -g3 -O0
checking for additional debug build... no
configure: target-libgomp not built
checking for parallel mode support... no
checking for extra compiler flags for building... -fno-rtti -fno-exceptions -std=c++03
checking for extern template support... yes
checking for custom python install directory... no
checking for -Werror... yes
checking for vtable verify support... no
checking for gets declaration... no
checking for EOWNERDEAD... yes
checking for ENOTRECOVERABLE... yes
checking for ENOLINK... yes
checking for EPROTO... yes
checking for ENODATA... yes
checking for ENOSR... yes
checking for ENOSTR... yes
checking for ETIME... yes
checking for EBADMSG... yes
checking for ECANCELED... yes
checking for EOVERFLOW... yes
checking for ENOTSUP... yes
checking for EIDRM... yes
checking for ETXTBSY... yes
checking for ECHILD... yes
checking for ENOSPC... yes
checking for EPERM... yes
checking for ETIMEDOUT... yes
checking for EWOULDBLOCK... yes
checking for int64_t... yes
checking for int64_t as long... checking for int64_t as long long... yes
checking for LFS support... no
checking sys/ioctl.h usability... yes
checking sys/ioctl.h presence... yes
checking for sys/ioctl.h... yes
checking sys/filio.h usability... no
checking sys/filio.h presence... no
checking for sys/filio.h... no
checking for poll... no
checking for S_ISREG or S_IFREG... no
checking sys/uio.h usability... yes
checking sys/uio.h presence... yes
checking for sys/uio.h... yes
checking for writev... no
checking fenv.h usability... yes
checking fenv.h presence... yes
checking for fenv.h... yes
checking for complex.h... (cached) no
checking for complex.h... (cached) no
no
checking for ISO C99 support to TR1 in <ctype.h>... yes
checking for fenv.h... (cached) yes
checking for ISO C99 support to TR1 in <fenv.h>... no
checking for ISO C99 support to TR1 in <stdint.h>... yes
checking for ISO C99 support to TR1 in <math.h>... no
checking for ISO C99 support to TR1 in <inttypes.h>... yes
checking for wchar_t ISO C99 support to TR1 in <inttypes.h>... yes
checking stdbool.h usability... no
checking stdbool.h presence... yes
checking for stdbool.h... no
checking stdalign.h usability... no
checking stdalign.h presence... yes
checking for stdalign.h... no
checking for the value of EOF... -1
checking for the value of SEEK_CUR... 1
checking for the value of SEEK_END... 2
checking for gettimeofday... checking sys/time.h usability... no
checking sys/time.h presence... yes
checking for sys/time.h... no
checking for sleep... yes
checking for usleep... yes
checking for tmpnam... no
checking locale.h usability... yes
checking locale.h presence... yes
checking for locale.h... yes
checking for LC_MESSAGES... yes
checking sys/sysinfo.h usability... yes
checking sys/sysinfo.h presence... yes
checking for sys/sysinfo.h... yes
checking for get_nprocs... no
checking for unistd.h... (cached) yes
checking for _SC_NPROCESSORS_ONLN... no
checking for _SC_NPROC_ONLN... no
checking for pthreads_num_processors_np... no
checking for hw.ncpu sysctl... no
checking for suitable sys/sdt.h... no
checking endian.h usability... yes
checking endian.h presence... yes
checking for endian.h... yes
checking execinfo.h usability... yes
checking execinfo.h presence... yes
checking for execinfo.h... yes
checking float.h usability... yes
checking float.h presence... yes
checking for float.h... yes
checking fp.h usability... no
checking fp.h presence... no
checking for fp.h... no
checking ieeefp.h usability... no
checking ieeefp.h presence... no
checking for ieeefp.h... no
checking for inttypes.h... (cached) yes
checking for locale.h... (cached) yes
checking machine/endian.h usability... no
checking machine/endian.h presence... no
checking for machine/endian.h... no
checking machine/param.h usability... no
checking machine/param.h presence... no
checking for machine/param.h... no
checking nan.h usability... no
checking nan.h presence... no
checking for nan.h... no
checking for stdint.h... (cached) yes
checking for stdlib.h... (cached) yes
checking for string.h... (cached) yes
checking for strings.h... (cached) yes
checking sys/ipc.h usability... yes
checking sys/ipc.h presence... yes
checking for sys/ipc.h... yes
checking sys/isa_defs.h usability... no
checking sys/isa_defs.h presence... no
checking for sys/isa_defs.h... no
checking sys/machine.h usability... no
checking sys/machine.h presence... no
checking for sys/machine.h... no
checking sys/param.h usability... yes
checking sys/param.h presence... yes
checking for sys/param.h... yes
checking sys/resource.h usability... yes
checking sys/resource.h presence... yes
checking for sys/resource.h... yes
checking sys/sem.h usability... yes
checking sys/sem.h presence... yes
checking for sys/sem.h... yes
checking for sys/stat.h... (cached) yes
checking for sys/time.h... (cached) no
checking for sys/types.h... (cached) yes
checking for unistd.h... (cached) yes
checking for wchar.h... (cached) yes
checking for wctype.h... (cached) yes
checking for ld used by gcc  -mx32... (cached) /usr/bin/ld
checking if the linker (/usr/bin/ld) is GNU ld... (cached) yes
checking for ld version... 22901
checking for ld that supports -Wl,--gc-sections... yes
checking for ld that supports -Wl,-z,relro... yes
checking for sin in -lm... (cached) no
checking for isinf declaration... no
checking for _isinf declaration... no
checking for isnan declaration... no
checking for _isnan declaration... no
checking for finite declaration... no
checking for _finite declaration... no
checking for sincos declaration... no
checking for _sincos declaration... no
checking for fpclass declaration... no
checking for _fpclass declaration... no
checking for qfpclass declaration... no
checking for _qfpclass declaration... no
checking for hypot declaration... no
checking for _hypot declaration... no
checking for float trig functions... no
checking for _float trig functions... no
checking for float round functions... no
checking for _float round functions... no
checking for expf declaration... no
checking for _expf declaration... no
checking for isnanf declaration... no
checking for _isnanf declaration... no
checking for isinff declaration... no
checking for _isinff declaration... no
checking for atan2f declaration... no
checking for _atan2f declaration... no
checking for fabsf declaration... no
checking for _fabsf declaration... no
checking for fmodf declaration... no
checking for _fmodf declaration... no
checking for frexpf declaration... no
checking for _frexpf declaration... no
checking for hypotf declaration... no
checking for _hypotf declaration... no
checking for ldexpf declaration... no
checking for _ldexpf declaration... no
checking for logf declaration... no
checking for _logf declaration... no
checking for log10f declaration... no
checking for _log10f declaration... no
checking for modff declaration... no
checking for _modff declaration... no
checking for modf declaration... no
checking for _modf declaration... no
checking for powf declaration... no
checking for _powf declaration... no
checking for sqrtf declaration... no
checking for _sqrtf declaration... no
checking for sincosf declaration... no
checking for _sincosf declaration... no
checking for finitef declaration... no
checking for _finitef declaration... no
checking for long double trig functions... no
checking for _long double trig functions... no
checking for long double round functions... no
checking for _long double round functions... no
checking for isnanl declaration... no
checking for _isnanl declaration... no
checking for isinfl declaration... no
checking for _isinfl declaration... no
checking for atan2l declaration... no
checking for _atan2l declaration... no
checking for expl declaration... no
checking for _expl declaration... no
checking for fabsl declaration... no
checking for _fabsl declaration... no
checking for fmodl declaration... no
checking for _fmodl declaration... no
checking for frexpl declaration... no
checking for _frexpl declaration... no
checking for hypotl declaration... no
checking for _hypotl declaration... no
checking for ldexpl declaration... no
checking for _ldexpl declaration... no
checking for logl declaration... no
checking for _logl declaration... no
checking for log10l declaration... no
checking for _log10l declaration... no
checking for modfl declaration... no
checking for _modfl declaration... no
checking for powl declaration... no
checking for _powl declaration... no
checking for sqrtl declaration... no
checking for _sqrtl declaration... no
checking for sincosl declaration... no
checking for _sincosl declaration... no
checking for finitel declaration... no
checking for _finitel declaration... no
checking for at_quick_exit declaration... no
checking for quick_exit declaration... no
checking for strtold declaration... no
checking for strtof declaration... no
checking for "/dev/random" and "/dev/urandom" for TR1 random_device... yes
checking whether the target supports thread-local storage... yes
checking for __cxa_thread_atexit_impl... yes
checking for ld used by GCC... /usr/bin/ld
checking if the linker (/usr/bin/ld) is GNU ld... yes
checking for shared library run path origin... done
checking for iconv... yes
checking for iconv declaration... install-shextern size_t iconv (iconv_t cd, char * *inbuf, size_t *inbytesleft, char * *outbuf, size_t *outbytesleft);
looking for a compliant stdint.h in stdint.h, checking for uintmax_t... yes
checking for uintptr_t... yes
checking for int_least32_t... yes
checking for int_fast32_t... yes
checking for uint64_t... yes
checking what to include in include/gstdint.h... stdint.h (already complete)
checking for GNU c++filt... /usr/bin/c++filt
checking for shared libgcc... yes
checking whether the target supports .symver directive... yes
configure: versioning on shared library symbols is gnu
checking for size_t as unsigned int... yes
checking for ptrdiff_t as int... yes
checking whether the target supports hidden visibility... yes
configure: visibility supported: yes
checking for default std::string ABI to use... gcc4-compatible
checking for rdrand support in assembler... yes
checking for unistd.h... (cached) yes
checking for sys/time.h... (cached) no
checking for sys/resource.h... (cached) yes
checking for testsuite resource limits support... no
checking for setenv declaration... no
checking whether it can be safely assumed that mutex_timedlock is available... yes
checking for gthreads library... no
checking fcntl.h usability... yes
checking fcntl.h presence... yes
checking for fcntl.h... yes
checking dirent.h usability... yes
checking dirent.h presence... yes
checking for dirent.h... yes
checking sys/statvfs.h usability... yes
checking sys/statvfs.h presence... yes
checking for sys/statvfs.h... yes
checking utime.h usability... yes
checking utime.h presence... yes
checking for utime.h... yes
checking whether to build Filesystem TS support... no
checking for struct dirent.d_type... no
checking for realpath... no
checking for utimensat... no
checking for struct stat.st_mtim.tv_nsec... no
checking for fchmod... no
checking for fchmodat... no
checking for sendfile that can copy files... no
checking for makeinfo... ${SHELL} /data105/extractors/gcc-5.5.0/missing --run makeinfo
checking for modern makeinfo... no
checking for doxygen... no
checking for dot... no
checking for xsltproc... no
checking for xmllint... yes
checking for docbook stylesheets for documentation creation... no
checking for local stylesheet directory... no
checking for epub3 stylesheets for documentation creation... no
checking for dblatex... no
checking for pdflatex... no
checking for gxx-include-dir... no
checking for --enable-version-specific-runtime-libs... no
checking for install location... ${prefix}/include/c++/${gcc_version}
configure: creating ./config.status
config.status: creating Makefile
config.status: creating scripts/testsuite_flags
config.status: creating scripts/extract_symvers
config.status: creating doc/xsl/customization.xsl
config.status: creating include/Makefile
Adding multilib support to include/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=x32
config.status: creating libsupc++/Makefile
Adding multilib support to libsupc++/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=x32
config.status: creating src/Makefile
Adding multilib support to src/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=x32
config.status: creating src/c++98/Makefile
Adding multilib support to src/c++98/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=x32
config.status: creating src/c++11/Makefile
Adding multilib support to src/c++11/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=x32
config.status: creating src/filesystem/Makefile
Adding multilib support to src/filesystem/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=x32
config.status: creating doc/Makefile
Adding multilib support to doc/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=x32
config.status: creating po/Makefile
Adding multilib support to po/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=x32
config.status: creating testsuite/Makefile
Adding multilib support to testsuite/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=x32
config.status: creating python/Makefile
Adding multilib support to python/Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=x32
config.status: creating config.h
config.status: config.h is unchanged
config.status: executing default-1 commands
Adding multilib support to Makefile in ../../gcc-5.5.0/libstdc++-v3
with_multisubdir=x32
config.status: executing libtool commands
config.status: executing include/gstdint.h commands
config.status: executing generate-headers commands
make: Nothing to be done for 'all'.
config.status: executing libtool commands
config.status: executing include/gstdint.h commands
config.status: executing generate-headers commands
make: Nothing to be done for 'all'.

$ make
</source>
````

<a id="w003996"></a>

## W003996 — Mouse

Bron-ID: W003996
Namespace: Main (0)
Revisie: 19976 | Datum: 2016-11-29T18:32:26Z
Bron: https://wiki.osdev.org/Mouse
Revisie-URL: https://wiki.osdev.org/index.php?title=Mouse&oldid=19976
Doorverwijzing: Mouse Input

````text
#REDIRECT [[Mouse Input]]
````

<a id="w003997"></a>

## W003997 — File:Tupai-0-1-0.png

Bron-ID: W003997
Namespace: File (6)
Revisie: 19978 | Datum: 2016-11-30T03:26:04Z
Bron: https://wiki.osdev.org/File:Tupai-0-1-0.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ATupai-0-1-0.png&oldid=19978

````text
A screenshot of Tupai 0.1.0
````

<a id="w003998"></a>

## W003998 — Creating a 64-bit kernel using a separate loader

Bron-ID: W003998
Namespace: Main (0)
Revisie: 30494 | Datum: 2026-04-18T15:18:12Z
Bron: https://wiki.osdev.org/Creating_a_64-bit_kernel_using_a_separate_loader
Revisie-URL: https://wiki.osdev.org/index.php?title=Creating_a_64-bit_kernel_using_a_separate_loader&oldid=30494

````text
{{Rating|3}}{{Template:Kernel designs}}
{{In_Progress}}

This page is an extension to [[Creating_a_64-bit_kernel|Creating a 64-bit kernel]], specifically the section [[Creating_a_64-bit_kernel#With_a_separate_loader|Loading with a separate loader]]. If you have not read that, go read it now.<br/>
<b>This may or may not work with your kernel and you may or may not need to tweak it to work with your kernel.</b>

== 64-bit kernel ==
The kernel itself should not contain any multiboot headers, and does not need an assembly bootstrap (you are free to use one if you find it necessary).<br/>
The OUTPUT_FORMAT, as specified in your linker script, should be elf64-x86-64 for it to be loaded correctly by the Loader.<br/>

After your loader has passed control to the kernel, you should set up a 64-bit IDT and paging.<br/>

Compile C files with
<syntaxhighlight lang="bash">
x86_64-elf-gcc -m64 -c <source file> -o <output> -ffreestanding -z max-page-size=0x1000 -mno-red-zone -mno-mmx -mno-sse -mno-sse2 -std=gnu99 -O2 -Wall -Wextra -I <path to your header files>
</syntaxhighlight>
Link the kernel with
<syntaxhighlight lang="bash">
x86_64-elf-gcc -T <linker script> -o <output> <all object files> -ffreestanding -O2 -nostdlib -lgcc
</syntaxhighlight>

Your output should be copied to loader_build/boot/kernel.bin (See [[#GRUB|32-bit loader GRUB]])

== 32-bit loader ==
Your loader will be similar to the kernel made in the [[Bare Bones]] tutorial. 
=== Assembly bootstrap ===
It starts with a multiboot header and an assembly routine to set up a stack and call the C code. <br/>
Additionally it might check eax to see if it contains 0x2BADB002 (which means it was loaded by a multiboot bootloader)
<syntaxhighlight lang="asm">
_start:
    cli                              ; Clear interrupts, we don't want them as long as we are in the loader
    mov esp, stack_top               ; Set up a valid stack
    mov ebp, stack_top

    push ebx                         ; Push pointer to multiboot info structure
    call lmain
</syntaxhighlight>

=== Loader ===
The loader must now set up its own <b>32-bit</b> [[Global Descriptor Table|GDT]] and parse the multiboot structure given to it by the bootstrap.<br/>
A header containing useful multiboot structures can be downloaded [https://www.gnu.org/software/grub/manual/multiboot/html_node/multiboot_002eh.html here]
<syntaxhighlight lang="C">
#include "multiboot.h"

/* This function gets called by the bootloader */
void lmain(const void* multiboot_struct) {
        // Set up GDT
        
        const multiboot_info_t* mb_info = multiboot_struct;            /* Make pointer to multiboot_info_t struct */
        multiboot_uint32_t mb_flags = mb_info->flags;                  /* Get flags from mb_info */

        void* kentry = NULL;                                           /* Pointer to the kernel entry point */

        if (mb_flags & MULTIBOOT_INFO_MODS) {                          /* Check if modules are available */
                multiboot_uint32_t mods_count = mb_info->mods_count;   /* Get the amount of modules available */
                multiboot_uint32_t mods_addr = mb_info->mods_addr;     /* And the starting address of the modules */

                for (uint32_t mod = 0; mod < mods_count; mod++) {
                        multiboot_module_t* module = (multiboot_module_t*)(mods_addr + (mod * sizeof(multiboot_module_t)));     /* Loop through all modules */
                }
        }
}
</syntaxhighlight>

Every multiboot module has a command line associated with it, available from the multiboot_module_t is a pointer to it.<br/>
One might load all the modules available into memory at this point, or check them for a specific command line to load only a specific subset.
<syntaxhighlight lang="C">
        const char* module_string = (const char*)module->cmdline;
        /* Here I check if module_string is equals to the one i assigned my kernel
           you could skip this check if you had a way of determining the kernel module */
        if(strcmp(module_string, kernel_bin_string)){
                kentry = load_elf_module(module->mod_start, module->mod_end);
        }
</syntaxhighlight>

Now, we must parse the kernel ELF file that the bootloader loaded into memory for us.<br/>
This can be done pretty easily using a helper function and a header I wrote following the ELF64 specification (Available [|here]) and function declarations (Available [|here])

<syntaxhighlight lang="C">
        #include "elf64.h" // Also requires elf64.c

        char* kernel_elf_space[sizeof(elf_file_data_t)];
        elf_file_data_t* kernel_elf = (elf_file_data_t*) kernel_elf_space;                                          /* Pointer to elf file structure (remember there is no memory management yet) */

        /* This function parses the ELF file and returns the entry point */
        void* load_elf_module(multiboot_uint32_t mod_start, multiboot_uint32_t mod_end) {
                unsigned long err = parse_elf_executable((void*)mod_start, sizeof(elf_file_data_t), kernel_elf);    /* Parses ELF file and returns an error code */
                if(err == 0){                                                                                       /* No errors occurred while parsing the file */
                        for(int i = 0; i < kernel_elf->numSegments; i++){
                                elf_file_segment_t seg = kernel_elf->segments[i];                                   /* Load all the program segments into memory */
                                                                                                                    /*  if you want to do relocation you should do so here, */
                                const void* src = (const void*) (mod_start + seg.foffset);                          /*  though that would require some changes to parse_elf_executable */
                                memcpy((void*) seg.address, src, seg.flength);
		                }
                        return (void*) kernel_elf->entryAddr;                                                       /* Finally we can return the entry address */
                }
                return NULL;
        }
</syntaxhighlight>

Finally, we can set up long mode and jump to the 64-bit kernel.<br/>
We do this by putting 2 assembly functions at the end of the lmain function

<syntaxhighlight lang="C">
       extern void setup_longmode();
       extern void enter_kernel(void* entry, uint32_t multiboot_info);

       void lmain(const void* multiboot_struct) {
               // Set up GDT
               // Check modules

               setup_longmode();                                                                                    /* Set up long mode and jump to the kernel code */
               enter_kernel(kentry, (uint32_t) mb_info);
       }
</syntaxhighlight>

=== Setting up long mode ===
Before we even look at switching to long mode, we must make sure that it is available on the current CPU.<br/>
* First, we check if the CPUID instruction is available by flipping bit 21 (CPUID) in the eflags register. If the bit was flipped CPUID is available.
* Second, we must check if the extended CPUID functions are avaiable. This is done by checking CPUID function 0x80000000, to see if it is higher than or equals to 0x80000001.
* And last a check if long mode is available, by executing CPUID function 0x80000001 and testing if bit 29 (Long mode) is set.

To set up long mode, many things must be done.
* Disable any paging your bootloader might have set up (clearing bit 31 in CR0)
* Then, set up 64-bit paging (but do <b>not</b> enable it yet). This includes PML4T, PDPT, PDT and PT to identity map any memory you will need before your 64-bit kernel has set up it's own paging.
* Enable PAE (Physical Address Extension) by setting bit 5 of CR4
* Switch to IA32e (compatibility mode), by setting bit 8 (Long Mode Enable) in MSR 0xC0000080
* Enable paging again, by setting bit 31 in CR0
<br/>
Now we are in Compatibility mode, to enter Long mode we have to set up a 64-bit GDT (This will be the GDT your kernel uses, so you have to set up everything here)<br/>
<b>The code below is not optimal in the slightest, and you should try to put together your own alternative.</b>
<syntaxhighlight lang="asm">
global enter_kernel
enter_kernel:
    push     ebp
    mov ebp, esp                ; Set up the stack so the variables passed from the C code can be read

    mov esi, [ebp+8]            ; This is the kernel entry point
    mov [k_ptr], esi

    lgdt [GDT.pointer]          ; Load GDT
	
    mov ax, GDT.data            ; Reload data segment selectors
    mov ss, ax
    mov ds, ax
    mov es, ax
    jmp GDT.code:.jmp_k         ; Reload code selector by jumping to 64-bit code
.jmp_k:
    mov edi, [ebp + 12]         ; 1st argument of kernel_main (pointer to multiboot structure)
    mov eax, [k_ptr]                                                                                              ; This is transformed to mov rax, [k_ptr] and uses the double word reserved below 
    dd  0                       ; Trick the processor, contains high address of k_ptr                             ; as higher half of the address to k_ptr
    jmp eax                     ; This part is plain bad, tricking the processor is not the best thing to do here

section .data
k_ptr:
    dd  0
</syntaxhighlight>

=== Compiling and linking the loader ===
The loader must be compiled and linked as a 32-bit ELF file with the Multiboot header within the first 8 kB (See [[Bare Bones]] Tutorial to see how to do this)<br/>

=== GRUB ===
If you used GRUB this is an example of how to set up the directory structure:

* loader_build/
** boot/
*** grub/
**** grub.cfg
*** kernel.bin         (This is the 64-bit kernel ELF file)
*** loader.bin         (This is the 32-bit loader ELF file)

Your grub.cfg file will contain at least the following
 menuentry "Kernel" {
     multiboot /boot/loader.bin                  // Path to the loader executable
     module /boot/kernel.bin "KERNEL_BIN"        // Path to the kernel executable, the string in "" is your command line
 
     // More modules may be added here in the form 'module <path> "<cmdline>"'
 }

Note: if you are using Multiboot 2, replace the above multiboot/module lines with multiboot2/module2

To build it:
<syntaxhighlight lang="bash">
grub-mkrescue -o os.iso loader_build/
</syntaxhighlight>

== See Also ==
=== Articles ===
* [https://www.uclibc.org/docs/elf-64-gen.pdf ELF-64 specification]
* [https://www.gnu.org/software/grub/manual/multiboot/multiboot.html Multiboot specification v0.6.96]

[[Category:Tutorials|Creating a 64-bit kernel using a separate loader]]
[[Category:X86-64|Creating a 64-bit kernel using a separate loader]]
[[Category:Bootloaders]]
````

<a id="w003999"></a>

## W003999 — PC Screen Font

Bron-ID: W003999
Namespace: Main (0)
Revisie: 30385 | Datum: 2026-03-23T13:53:25Z
Bron: https://wiki.osdev.org/PC_Screen_Font
Revisie-URL: https://wiki.osdev.org/index.php?title=PC_Screen_Font&oldid=30385

````text
On every Linux distribution, console fonts can be found with the extension <code>.psf</code> or <code>.psfu</code>. These can be located by entering <code>whereis consolefonts</code> on a terminal. This article describes how to display those fonts on graphical screen, with the advantage of not having to modify the fonts, instead directly using the ones shipped by the Linux distributions. The other advantage is that
PSF fonts can store the whole Unicode character set, although consolefonts have maximum 512 glyphs.

There are two versions of PSF, PSF 1 and PSF 2. Each of these versions can be detected using their magic number.
This page assumes a PSF 2 font is being used.

== Structure of file ==
A PSF file consists of a header, bitmaps for the glyphs and optionally a Unicode character translation table.

=== File Header ===
It's a fixed chunk at the beginning of the file. The version must be detected before proceeding into parsing the font file.

<syntaxhighlight lang="c">
#define PSF1_FONT_MAGIC 0x0436

/*
    For PSF1 glyph width is always = 8 bits
    and glyph height = characterSize
*/
typedef struct {
    uint16_t magic; // Magic bytes for identification.
    uint8_t fontMode; // PSF font mode.
    uint8_t characterSize; // PSF character size.
} PSF1_Header;


#define PSF_FONT_MAGIC 0x864ab572

/*
    PSF2
*/
typedef struct {
    uint32_t magic;         /* magic bytes to identify PSF */
    uint32_t version;       /* zero */
    uint32_t headersize;    /* offset of bitmaps in file, 32 */
    uint32_t flags;         /* 0 if there's no unicode table */
    uint32_t numglyph;      /* number of glyphs */
    uint32_t bytesperglyph; /* size of each glyph */
    uint32_t height;        /* height in pixels */
    uint32_t width;         /* width in pixels */
} PSF_font;
</syntaxhighlight>

=== Glyphs ===
Each glyph is a bitmap, encoded the same way as [[VGA Fonts]]. For a 8x16 font, each glyph is 16 bytes long, and every byte encodes exactly one row of the glyph.
<pre>
00000000b  byte  0
00000000b  byte  1
00000000b  byte  2
00010000b  byte  3
00111000b  byte  4
01101100b  byte  5
11000110b  byte  6
11000110b  byte  7
11111110b  byte  8
11000110b  byte  9
11000110b  byte 10
11000110b  byte 11
11000110b  byte 12
00000000b  byte 13
00000000b  byte 14
00000000b  byte 15
</pre>For glyphs where the width isn't divisible by 8, there will be padding bits added to make the line divisible on a byte boundary. ￼  As an example inspired from [[/en.wikipedia.org/wiki/PC Screen Font|PC Screen Font - Wikipedia]], the below psf glyph for O is 12x12.  Since 12 isn't divisible by 8, 4 padding bits are added to make each line divisible by a byte.<syntaxhighlight>
 Glyph Data  padding   
|-----------|----|
111111111111 0000
111111111111 0000
110000000011 0000
110000000011 0000
110000000011 0000
110000000011 0000
110000000011 0000
110000000011 0000
110000000011 0000
110000000011 0000
111111111111 0000
111111111111 0000

</syntaxhighlight>

=== Unicode Table ===
If the flags in the PSF header is 1, it indicates that the font has a Unicode table for glyph mapping. Without such a table, Unicode characters and glyphs are mapped identically, so first glyph is for Unicode character 0, second glyph for Unicode character 1 and so forth.

The table is as follows: each glyph has a variable length record. Those are very similar to lines in a text file, only here lines are ended in 0xFF character not '\n' (0x0A). The nth line describes the nth glyph's mappings. Every line contains at least one, but possibly more UTF-8 character sequences.

== Dealing with PSF ==
Although there are lot of PSF editors out there, most of them are broken or hard to use. So it is recommended to use two Perl scripts instead: [https://github.com/talamus/solarize-12x29-psf/ readpsf, writepsf]. These scripts can convert PSF into easily editable ASCII text file or a bitmap image that can be opened with any image editing program.

== Loading the font ==
As described in [[VGA Fonts]], there are several options. For simplicity, in this example the font will be embedded into the kernel executable. Here's how to convert PSF into an [[ELF]] that can be linked with a kernel executable:
<syntaxhighlight lang="bash">
objcopy -O elf64-x86-64 -B i386 -I binary font.psf font.o
readelf -s font.o

Symbol table '.symtab' contains 5 entries:
   Num:    Value          Size Type    Bind   Vis      Ndx Name
     0: 0000000000000000     0 NOTYPE  LOCAL  DEFAULT  UND 
     1: 0000000000000000     0 SECTION LOCAL  DEFAULT    1 
     2: 0000000000000000     0 NOTYPE  GLOBAL DEFAULT    1 _binary_font_psf_start
     3: 0000000000008020     0 NOTYPE  GLOBAL DEFAULT    1 _binary_font_psf_end
     4: 0000000000008020     0 NOTYPE  GLOBAL DEFAULT  ABS _binary_font_psf_size

</syntaxhighlight>
The resulting object exports three symbols that can be referenced as any other variables (using <code>extern</code> in C for example).

Before the font can be used, first the Unicode table has to be decoded. This is tricky and dynamic memory allocation is required (<code>calloc()</code> here), but the good news it is optional if the "first glyph for character 0, second glyph for character 1, etc." scheme is fine.

<syntaxhighlight lang="c">
/* import the font contained in the object file created above */
extern char _binary_font_psf_start;
extern char _binary_font_psf_end;

uint16_t *unicode;

void psf_init()
{
    uint16_t glyph = 0;
    /* cast the address to PSF header struct */
    PSF_font *font = (PSF_font*)&_binary_font_psf_start;
    /* is there a unicode table? */
    if (font->flags == 0) {
        unicode = NULL;
        return; 
    }

    /* get the offset of the table */
    char *s = (char *)(
    (unsigned char*)&_binary_font_psf_start +
      font->headersize +
      font->numglyph * font->bytesperglyph
    );
    /* allocate memory for translation table */
    unicode = calloc(USHRT_MAX, 2);
    while(s < (unsigned char*)&_binary_font_psf_end) {
        uint16_t uc = (uint16_t)((unsigned char *)s[0]);
        if(uc == 0xFF) {
            glyph++;
            s++;
            continue;
        } else if(uc & 128) {
            /* UTF-8 to unicode */
            if((uc & 32) == 0 ) {
                uc = ((s[0] & 0x1F)<<6)+(s[1] & 0x3F);
                s++;
            } else
            if((uc & 16) == 0 ) {
                uc = ((((s[0] & 0xF)<<6)+(s[1] & 0x3F))<<6)+(s[2] & 0x3F);
                s+=2;
            } else
            if((uc & 8) == 0 ) {
                uc = ((((((s[0] & 0x7)<<6)+(s[1] & 0x3F))<<6)+(s[2] & 0x3F))<<6)+(s[3] & 0x3F);
                s+=3;
            } else
                uc = 0;
        }
        /* save translation */
        unicode[uc] = glyph;
        s++;
    }
}

</syntaxhighlight>
== Displaying a character ==
It is assumed that a linear frame buffer has been set up properly and plotting pixels is possible. This example uses 32 bit RGBA format, but can be adapted to other formats easily.
<syntaxhighlight lang="c">
/* the linear framebuffer */
extern char *fb;
/* number of bytes in each line, it's possible it's not screen width * bytesperpixel! */
extern int scanline;
/* import the font contained in the object file created above */
extern char _binary_font_start[];

#define PIXEL uint32_t   /* pixel pointer */
 
void putchar(
    /* note that this is int, not char as it's a unicode character */
    unsigned short int c,
    /* cursor position on screen, in characters not in pixels */
    int cx, int cy,
    /* foreground and background colors, say 0xFFFFFF and 0x000000 */
    uint32_t fg, uint32_t bg)
{
    /* cast the address to PSF header struct */
    PSF_font *font = (PSF_font*)&_binary_font_psf_start;
    /* unicode translation */
    if(unicode != NULL) {
        c = unicode[c];
    }
    /* get the glyph for the character. If there's no
       glyph for a given character, we'll display the first glyph. */
    unsigned char *glyph =
     (unsigned char*)&_binary_font_psf_start +
     font->headersize +
     (c>0&&c<font->numglyph?c:0)*font->bytesperglyph;
    /* calculate the upper left corner on screen where we want to display.
       we only do this once, and adjust the offset later. This is faster. */
    int offs =
        (cy * font->height * scanline) +
        (cx * (font->width + 1) * sizeof(PIXEL));
    
    /* Calculate the number of bytes for a line in a glyph. If the
    glyph width isn't byte aligned, then it is rounded up to be byte aligned. */    
    uint32_t bytesPerGlyphLine = (width + 7) / 8;
    /* finally display pixels according to the bitmap */
    int x, y, line
    for(y = 0; y < font->height; y++){
        /* save the starting position of the line */
        line = offs;
        /* Calulate where the first byte for this line of the glyph is. */
        unsigned char* currentByte = glyph + (bytesPerGlyphLine * y);
        /* Start with a mask at this byte's MSB */
        uint8_t mask = 1<< 7;
        /* display a row */
        for(x = 0; x <font->width; x++){
            *((PIXEL*)(fb + line)) = (*currentByte & mask) ? fg : bg;
            mask >>= 1;
            if (mask == 0){
                /* We have read this byte of the glyph.
                Reset mask and move to next byte */
                mask = 1<<7;
                currentByte += 1;
            }
            /* adjust to the next pixel in framebuffer */
            line += sizeof(PIXEL);
        }
        /* adjust to the next line in framebuffer */
        offs  += scanline;
    }
}
</syntaxhighlight>
With this code, strings can be displayed on a linear frame buffer. Please note that this code is not optimal, it's for demonstration purposes, but it's a good start.

Calculating the offsets with width + 1 is necessary to keep one pixel distance between glyphs on screen. Without some fonts became unreadable. This is also what the original [[VGA Hardware|VGA hardware]] did, it used 8x16 fonts, but actually displayed them as 9x16. Casting the glyph before doing <code>AND</code> with the mask is necessary to support font widths up to 32 pixels.

Using 8 pixel width would require an unsigned char, but 9 pixels an unsigned short. This way the same code can be used even if the required size is unknown at compilation time (as it depends on the font loaded).

It may be tempting to replace <code>PIXEL</code> and line with an array, but that won't work on all hardware, as <code>scanline</code> is given in bytes, and nobody actually said that it must be a multiple of <code>PIXEL</code> (very unlikely, but possible).

== See Also ==
* [[VGA Fonts]]
* [[Scalable Screen Font]] - comes with a small, free ANSI C rendering library
* [[Loading Icons]]

== External Links ==
*http://www.win.tue.nl/~aeb/linux/kbd/font-formats-1.html PC Screen Font format description
*https://en.wikipedia.org/wiki/PC_Screen_Font Wikipedia on PC Screen Font
*https://github.com/talamus/solarize-12x29-psf/blob/master/readpsf Perl script to convert PSF into txt or bmp.
*https://github.com/talamus/solarize-12x29-psf/blob/master/writepsf Perl script to convert txt or bmp into PSF.

[[Category:Graphical UI]]
[[Category:Fonts]]
````

<a id="w004000"></a>

## W004000 — Dynamic Linker

Bron-ID: W004000
Namespace: Main (0)
Revisie: 30293 | Datum: 2026-03-16T19:05:14Z
Bron: https://wiki.osdev.org/Dynamic_Linker
Revisie-URL: https://wiki.osdev.org/index.php?title=Dynamic_Linker&oldid=30293

````text
Sooner or later you'll reach the point where you want shared libraries. Here we won't discuss the difference between static and dynamic linking, you should be already familiar with that.

This article is about [[ELF]] on x86_64 architecture, but can be easily adopted to other systems as the concepts are similar.

== Home work ==
=== Memory Layout ===
I suppose you have created a new, empty address space, and you've already loaded the executable in it. I also assume that you've loaded the libc shared library after the executable's segment. And now you're stuck, because you don't know how to call printf from the executable.

As your executable was loaded by you, you should know the virtual address of ELF magic bytes in the memory. Use that for start.
<syntaxhighlight lang="C">
Elf64_Ehdr *ehdr = (Elf64_Ehdr *)(ptr);

if(!kmemcmp(ehdr->e_ident, ELFMAG,SELFMAG) &&
   ehdr->e_ident[EI_CLASS] == ELFCLASS64 &&
   ehdr->e_ident[EI_DATA] == ELFDATA2LSB &&
   ehdr->e_type == ET_EXEC && ehdr->e_shnum > 0)
{
   // We have a valid image with sections
}
</syntaxhighlight>
First we check the magic bytes and the format of the ELF64. As an extra, we also check whether it's executable and has a non-empty section table (we'll going to need it).

Now let's see what the GNU toolchain does for us to find printf.

=== Segment Local Calls ===
In order to figure that out, first we should know how a segment local call works. For that we'll use a very minimal source.
<syntaxhighlight lang="C">
void localfunction()
{
}

int main(int,char**)
{
    localfunction();
}
</syntaxhighlight>
That compiles to:
<syntaxhighlight lang="bash">
$ objdump -d test
000000000020016f <localfunction>:
  20016f:	55                   	push   %rbp
  200170:	48 89 e5             	mov    %rsp,%rbp
  200173:	90                   	nop
  200174:	5d                   	pop    %rbp
  200175:	c3                   	retq   

0000000000200176 <main>:
  200176:	55                   	push   %rbp
  200177:	48 89 e5             	mov    %rsp,%rbp
  20017a:	b8 00 00 00 00       	mov    $0x0,%eax
  20017f:	e8 eb ff ff ff       	callq  20016f <localfunction>
  200184:	90                   	nop
  200185:	5d                   	pop    %rbp
  200186:	c3                   	retq   
</syntaxhighlight>
That's trivial, a rip relative addressing is used at 20017f.

=== Inter-segment Calls ===
Now let's modify the source a bit to use a libc call:
<syntaxhighlight lang="C">
int main(int,char**)
{
    printf("Hello World");
}
</syntaxhighlight>
Compile and see what's generated.
<syntaxhighlight lang="bash">
000000000020016f <main>:
  20016f:	55                   	push   %rbp
  200170:	48 89 e5             	mov    %rsp,%rbp
  200173:	48 8d 3d b6 01 00 00 	lea    0x1b6(%rip),%rdi        # 200330 <_DYNAMIC+0x110>
  20017a:	b8 00 00 00 00       	mov    $0x0,%eax
  20017f:	e8 8c 00 00 00       	callq  200210 <printf@plt>
  200184:	90                   	nop
  200185:	5d                   	pop    %rbp
  200186:	c3                   	retq   

0000000000200200 <printf@plt-0x10>:
  200200:	ff 35 4a 0e 00 00    	pushq  0xe4a(%rip)        # 201050 <_GLOBAL_OFFSET_TABLE_+0x8>
  200206:	ff 25 4c 0e 00 00    	jmpq   *0xe4c(%rip)        # 201058 <_GLOBAL_OFFSET_TABLE_+0x10>
  20020c:	0f 1f 40 00          	nopl   0x0(%rax)

0000000000200210 <printf@plt>:
  200210:	ff 25 4a 0e 00 00    	jmpq   *0xe4a(%rip)        # 201060 <_GLOBAL_OFFSET_TABLE_+0x18>
  200216:	68 00 00 00 00       	pushq  $0x0
  20021b:	e9 e0 ff ff ff       	jmpq   200200 <main+0x91>
</syntaxhighlight>
What? Two more local functions? What happened here? The GNU toolchain has a concept for lazy run-time linking. That means the address is not resolved until it's referenced. To achieve that, it needs helper functions (generated to the .plt section in the text segment).

When the CPU executes this code, it will first call a normal local function at 20017f. That function is one of the helpers and it's purpose is to load an address from GOT+0x18 and jump to it. By default, the value points to the next instruction, which saves 0 to stack and calls the other helper function at 200200. That one is the reference resolver, and it's job is the replace the address in GOT with a relocated address.

Because the resolver function is not known at link time, it's address is also in GOT at 0x10. What's more it can receive one argument, stored at GOT+0x8. We can also spot that at 200206 the instruction is a jump and not a call, so resolver never returns to this helper, instead it should jump to the relocated address.
 
== Implementing a dynamic linker ==
All that disassembly teach us two things about the dynamic linker:
 1. it has to locate and write the GOT
 2. it has two parts: load time linker and a run time resolver.

The first part runs before the thread is started and saves the second part's address and argument into GOT. On the other hand, the second part runs when the thread is already running, and saves relocated addresses into GOT. As you can see, both parts require the address of GOT. To save resources, one should not locate the GOT twice: this is where the resolver's argument came in. While you can technically insert anything into this slot in the GOT, your best bet will be to store something that will easily identify the requesting program — for instance, the address of the start of the GOT.

To proceed we'll have to locate the GOT in memory and figure out what entries it has.

===Locating the GOT===
Time to peek on what's in the object file.
<syntaxhighlight lang="bash">
$ readelf -a test
Section Headers:
  [Nr] Name              Type             Address           Offset
       Size              EntSize          Flags  Link  Info  Align
  [10] '.got.plt'        PROGBITS         0000000000201048  00001048
       0000000000000020  0000000000000008  WA       0     0     8

Symbol table .symtab contains 23 entries:
   Num:    Value          Size Type    Bind   Vis      Ndx Name
    15: 0000000000201048     0 OBJECT  LOCAL  DEFAULT   10 _GLOBAL_OFFSET_TABLE_
</syntaxhighlight>
Symbol table shows that the GOT is at 201048. We can also see the same value in the section headers at '.got.plt'. That means we don't have to resolve symbols in order to get GOT's address which simplifies the first part. We can also learn that the GOT is 32 (0x20) bytes long in our example. 

Note: the address of the GOT can also be found in the dynamic section for most executable files, under an entry tagged 'DT_PLTGOT.'

===What's in the GOT?===
We already know that
 1. GOT+0x0 entry is unused
 2. GOT+0x8 is an argument to second part
 3. GOT+0x10 is function reference to second part
But what about the rest, starting at 201060 in our example? Here we have only one reference so it's obvious, but what if we have more references? How should we know which symbol is associated to which entry?
<syntaxhighlight lang="bash">
$ readelf -a test
Section Headers:
  [Nr] Name              Type             Address           Offset
       Size              EntSize          Flags  Link  Info  Align
  [ 4] '.rela.plt'       RELA             00000000002001e8  000001e8
       0000000000000018  0000000000000018  AI       2    10     8

Relocation section .rela.plt at offset 0x1e8 contains 1 entries:
  Offset          Info           Type           Sym. Value    Sym. Name + Addend
000000201060  000100000007 R_X86_64_JUMP_SLO 0000000000000000 printf + 0
</syntaxhighlight>
How convenient that another table is also recorded in the section headers. It's called '.rela.plt' and describes exactly that. When the resolver receives control, the second parameter on the stack will be an index into this section, which is the offset (in 16-byte entries on x86-64, or 8-byte entries on 32-bit x86) of an entry like this that points to the symbol being requested. 

===Where are my libraries? ===
So far we assumed that shared libraries are already loaded. It's the case with libc, but how do we know what other shared libraries the executable wants?

<syntaxhighlight lang="bash">
$ readelf -a test
Section Headers:
  [Nr] Name              Type             Address           Offset
       Size              EntSize          Flags  Link  Info  Align
  [ 6] '.dynamic'        DYNAMIC          0000000000200220  00000220
       0000000000000110  0000000000000010  WA       3     0     8

Dynamic section at offset 0x220 contains 12 entries:
  Tag        Type                         Name/Value
 0x0000000000000001 (NEEDED)             Shared library: [libc.so]
</syntaxhighlight>
Not surprising that the answer lies in the section header again. There's a table pointer called '.dynamic'. That table has several records, but what we really are interested in is the ones marked by "NEEDED".

===Symbol look up ===
To find out printf's address we should locate it's symbol first in the shared library.
<syntaxhighlight lang="bash">
$ readelf -a libc.so 
Section Headers:
  [Nr] Name              Type             Address           Offset
       Size              EntSize          Flags  Link  Info  Align
  [ 2] '.dynsym'         DYNSYM           0000000100000218  00000218
       00000000000003c0  0000000000000018   A       3     1     8
  [ 3] '.dynstr'         STRTAB           00000001000005d8  000005d8
       0000000000000124  0000000000000000   A       0     0     1

Symbol table .dynsym contains 40 entries:
   Num:    Value          Size Type    Bind   Vis      Ndx Name
     8: 0000000100000175    93 FUNC    GLOBAL DEFAULT    1 printf
</syntaxhighlight>
Bingo! It is 100000175 in our example.

== Gimme code!==
I've put all the above together in a very simple example, see [https://gitlab.com/bztsrc/osz/-/blob/master/tools/elftool.c elftool.c] on gitlab.

When I run it on the executable it gives:
<syntaxhighlight lang="bash">
$ gcc elftool.c -o elftool
$ ./elftool -d mytestelf.o
Stringtable 000003f0 (118 bytes), symbols 000002b8 (312 bytes, one entry 24)

--- IMPORT ---
Dynamic 00001e20 (464 bytes, one entry 16):
  0. /lib/libc.so.6

GOT 00002000 (104 bytes), Rela 000004d0 (240 bytes, one entry 24):
  0. 00602018 +0 puts
  1. 00602020 +0 fread
  2. 00602028 +0 fclose
  3. 00602030 +0 printf
  4. 00602038 +0 strcmp
  5. 00602040 +0 ftell
  6. 00602048 +0 malloc
  7. 00602050 +0 fseek
  8. 00602058 +0 fopen
  9. 00602060 +0 exit

--- EXPORT ---
</syntaxhighlight>
As you can see here we have an import section, but nothing to be exported. Now let's see a shared library!
<syntaxhighlight lang="bash">
$ ./elftool -d /lib/libc.so.6
Stringtable 00011038 (23041 bytes), symbols 00003d90 (53928 bytes, one entry 24)

--- IMPORT ---
Dynamic 00197b60 (496 bytes, one entry 16):
  0. /lib/ld-linux-x86-64.so.2

GOT 00198000 (88 bytes), Rela 0001f760 (192 bytes, one entry 24):
  0. 00398050 +844d0 
  1. 00398048 +a8560 
  2. 00398040 +7ff80 
  3. 00398038 +867c0 
  4. 00398030 +823f0 
  5. 00398028 +82780 
  6. 00398020 +a8640 
  7. 00398018 +83b50 

--- EXPORT ---
  8. 0008e850 __strspn_c1
  9. 0006aad0 putwchar
 10. 000f8640 __gethostname_chk
 11. 0008e870 __strspn_c2
 12. 0010f210 setrpcent
 13. 0009eda0 __wcstod_l
 14. 0008e8a0 __strspn_c3
 15. 000e8d10 epoll_create
 16. 000d1b50 sched_get_priority_min
 17. 000f8660 __getdomainname_chk
 18. 000e8f20 klogctl
 19. 0002c380 __tolower_l
 20. 0004f440 dprintf
 21. 000b8e00 setuid
 22. 000a3d20 __wcscoll_l
... lot more lines to come ...
</syntaxhighlight>
This time it has hell a lot of functions to export, and also it imports the dynamic linker of Linux with addend offsets in the GOT.

==Summary==
To summarize a dynamic linker should be look like:

 1. load-time linker
   1.1. locates GOT by '.got.plt' section, and saves that address in GOT+0x8
   1.2. stores second part's address at GOT+0x10
   1.3. reads '.dynamic' section to load shared libraries
 2. run-time reference resolver
   2.1. it's called by a helper
   2.2. that helper places index-0x18 and the address of GOT as arguments on the stack
   2.3. it has to locate '.rela.plt' to get the symbol for the reference
   2.4. the symbol is looked up in the shared library's '.dynsym' section to get relocated address
   2.5. that relocated address has to be saved in the GOT (index and base on the stack)
   2.6. clean up stack, restore registers and jump to the relocated address

That's all, hope it helps somebody! Good luck with implementing your own dynamic linker!

[[Category:Linkers]]
[[Category:Loaders]]
````

<a id="w004001"></a>

## W004001 — User:Majicktek

Bron-ID: W004001
Namespace: User (2)
Revisie: 20068 | Datum: 2016-12-30T11:28:02Z
Bron: https://wiki.osdev.org/User:Majicktek
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMajicktek&oldid=20068

````text
Open-Source Software team Majick Tek (magic tech) Creates software that is lightyears ahead. You can catch us at [http://majicktek.blogspot.com My Blog], [http://majick.netai.net My Website], or [https://github.com/MajickTek My GitHub]
````

<a id="w004002"></a>

## W004002 — Raspberry Pi Bare Bones Rust

Bron-ID: W004002
Namespace: Main (0)
Revisie: 28447 | Datum: 2023-11-07T11:38:20Z
Bron: https://wiki.osdev.org/Raspberry_Pi_Bare_Bones_Rust
Revisie-URL: https://wiki.osdev.org/index.php?title=Raspberry_Pi_Bare_Bones_Rust&oldid=28447

````text
{{FirstPerson}}
{{BeginnersWarning}}
{{Rating|3}}

This tutorial directly follows [[Raspberry Pi Bare Bones]] to accomplish the same task using Rust instead of C.

====Prerequisites====

The following instructions assume you have all the tools to, and successfully followed [[Raspberry Pi Bare Bones]] and ran your kernel in QEMU. It also assumes you are familiar with Rust and Cargo.

We will use [https://rustup.rs rustup] to manage Rust.

====Overview====

In the C tutorial, you created a reference to a symbol "kernel_main" in boot.S.

<source lang="asm">
	// Call kernel_main
	ldr r3, =kernel_main
</source>

At the end of that tutorial, the compiled kernel.c object file kernel.o provided a definition for this symbol using its function "kernel_main" to the linker when creating kernel.elf.

We will create a Rust library (.rlib) which defines the symbol kernel_main instead.

====Setup====

For brevity we will assume our environment has no Rust installation.

<source lang="bash"> curl https://sh.rustup.rs -sSf | sh </source>

This installs rustup, which will manage the Rust compiler and Cargo.

<source lang="bash"> rustup component add rust-src </source>

We will need to recompile some of the Rust language later, so we need this component.

<source lang="bash"> cargo install xargo </source>

We will use [https://github.com/japaric/xargo xargo] to build our project, which works just like a drop-in replacement for Cargo. xargo automatically handles the significant task of compiling and linking in components of the Rust language we will need (such as libcore.rs) for our target.

<source lang="bash">
xargo new kernel
cd kernel
rustup override set nightly
</source>

This creates our crate and tells rustup to use the nightly Rust toolchain for it. We will need features of Rust that are not available in stable.

In this directory create a file called arm-none-eabihf.json with this content:

<source lang="python">
{
    "llvm-target": "arm-none-eabihf",
    "target-endian": "little",
    "target-pointer-width": "32",
    "target-c-int-width": "32",
    "os": "none",
    "env": "eabi",
    "vendor": "unknown",
    "arch": "arm",

    "linker-flavor": "gcc",
    "linker": "arm-none-eabi-gcc",
    "data-layout": "e-m:e-p:32:32-i64:64-v128:64:128-a:0:32-n32-S64",
    "executables": true,
    "relocation-model": "static",
    "no-compiler-rt": true
}
</source>

Rust uses LLVM as a backend, which expects this kind of specification file for non-standard (has no triple) compilation targets. Filling out the values of this is beyond the scope of this tutorial; more information can be found in [https://github.com/japaric/rust-cross rust-cross] which covers cross compiling Rust in detail.

In your Cargo.toml, add

<pre>
[profile.dev]
panic = "abort"
</pre>

This tells rustc not to compile in stack unwinding. This allows recovery from panics, but when a kernel panics there is nothing to make a recovery attempt.

Edit src/lib.rs to contain

<pre>
#![no_std]

#[no_mangle]
pub extern fn kernel_main() {}
</pre>

We can build our rlib with a definition of kernel_main now: <source lang="bash"> xargo build --target arm-none-eabihf</source>.

Now, revisit the command which built kernel.elf in the C tutorial, but instead of kernel.o, provide the libkernel.rlib xargo just built in the target directory.

<source lang="bash">arm-none-eabi-gcc -T linker.ld -o myos.elf -ffreestanding -O2 -nostdlib boot.o path/to/libkernel.rlib </source>

This is it! You've built Rust code for bare metal arm that will run on the Raspberry Pi!

====Hello====

The C tutorial used the UART to say hello. The code below accomplishes the same in Rust. It does not configure the UART like the C code does because QEMU's raspi2 machine has a default state that will work. To work on real hardware, take care to configure the UART to your use case. You can read about this in the [https://www.raspberrypi.org/documentation/hardware/raspberrypi/bcm2835/README.md bcm2835 manual]. Put the following in src/lib.rs and rebuild your ELF:

<pre>
#![no_std]
#![feature(core_intrinsics, lang_items)]

use core::intrinsics::abort;
use core::intrinsics::volatile_load;
use core::intrinsics::volatile_store;
use core::panic::PanicInfo;

// raspi2 and raspi3 have peripheral base address 0x3F000000,
// but raspi1 has peripheral base address 0x20000000. Ensure
// you are using the correct peripheral address for your
// hardware.
const UART_DR: u32 = 0x3F201000;
const UART_FR: u32 = 0x3F201018;

fn mmio_write(reg: u32, val: u32) {
    unsafe { volatile_store(reg as *mut u32, val) }
}

fn mmio_read(reg: u32) -> u32 {
    unsafe { volatile_load(reg as *const u32) }
}

fn transmit_fifo_full() -> bool {
    mmio_read(UART_FR) & (1 << 5) > 0
}

fn receive_fifo_empty() -> bool {
    mmio_read(UART_FR) & (1 << 4) > 0
}

fn writec(c: u8) {
    while transmit_fifo_full() {}
    mmio_write(UART_DR, c as u32);
}

fn getc() -> u8 {
    while receive_fifo_empty() {}
    mmio_read(UART_DR) as u8
}

fn write(msg: &str) {
    for c in msg.chars() {
        writec(c as u8)
    }
}

#[no_mangle]
pub extern fn kernel_main() {
    write("Hello Rust Kernel world!");
    loop {
        writec(getc())
    }
}

// These functions below provide definitions for symbols libcore
// expects which are not present on our bare metal target. You
// will not encounter linker errors until you use a part of
// libcore that references them, such as iterators in this program.
// In the future you may need to provide real implementations for
// these functions.

#[no_mangle]
pub extern fn __aeabi_unwind_cpp_pr0() {}

#[lang = "eh_personality"]
pub extern fn eh_personality() {}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { abort() }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn _Unwind_Resume() { loop {} }
</pre>

If you run this you will see your text printed out on the virtual serial port:

<source lang="bash">
qemu-system-arm -M raspi2 -kernel myos.elf -serial stdio
</source>

====External Resources====
# [https://www.rust-lang.org/ Rust programming language]
# [https://rustup.rs rustup, Rust components manager]
# [https://github.com/rust-embedded/rust-raspi3-OS-tutorials rust-raspi3-tutorials] rewrite of bzt's [https://github.com/bztsrc/raspi3-tutorial C tutorials] in the Rust language

[[Category:ARM]]
[[Category:Raspberry Pi]]
[[Category:Bare bones tutorials]]
[[Category:Rust]]
````

<a id="w004003"></a>

## W004003 — User:Turnage

Bron-ID: W004003
Namespace: User (2)
Revisie: 20001 | Datum: 2016-12-17T22:01:30Z
Bron: https://wiki.osdev.org/User:Turnage
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATurnage&oldid=20001

````text
[https://github.com/turnage github]
````

<a id="w004004"></a>

## W004004 — User:Moondeck

Bron-ID: W004004
Namespace: User (2)
Revisie: 20036 | Datum: 2016-12-19T08:36:30Z
Bron: https://wiki.osdev.org/User:Moondeck
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMoondeck&oldid=20036

````text
Hi, i am moondeck

I like OSdev and embedded systems, so thats what my OS (hydrogen) is targeting.
````

<a id="w004005"></a>

## W004005 — User:Sdp

Bron-ID: W004005
Namespace: User (2)
Revisie: 20044 | Datum: 2016-12-23T16:40:19Z
Bron: https://wiki.osdev.org/User:Sdp
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASdp&oldid=20044

````text
sdp is [http://pr4tt.com Simon David Pratt], a PhD student at U. Waterloo.
````

<a id="w004006"></a>

## W004006 — Generic Interrupt Controller

Bron-ID: W004006
Namespace: Main (0)
Revisie: 28096 | Datum: 2023-07-09T21:32:12Z
Bron: https://wiki.osdev.org/Generic_Interrupt_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=Generic_Interrupt_Controller&oldid=28096

````text
==Introduction==

The Generic Interrupt Controller is a standardized component of modern ARM boards, and it provides a solid interrupt handling scheme for embedded systems. Various different versions of the GIC specification exist. The purpose of this article is to provide a quick reference for the ARM Generic Interrupt Controller's behaviour.

This article is current with '''version 2''' of the specification and has not been updated to match '''version 3'''.

===Key points===
* IRQs are divided into secure and non-secure IRQs, pertaining to the secure and non-secure worlds.
* By default all IRQs belong to the secure world, in the same manner that by default, <tt>SCR.NS</tt> is set to 0. Essentially if the running kernel is not secure-world aware, it will not observe any effects from the fact that it is running in secure world.
* GIC supports three types of IRQs: external, "internal" (more later) and inter-processor.
* Both version 1 and 2 GICs support an optional Security Extension feature set.
* Only version 2 GICs support an optional Virtualization Extension feature set.

==Differences between a version 1 GIC and a version 2 GIC==

For a GIC version 1 implementation:
* For a version 1 GIC, "Interrupt grouping" refers only to the grouping of IRQs as being either secure (group0) or non-secure (group1). By default, on #reset of the GIC, if a GIC supports IRQ grouping, all IRQs are assigned to group0, and the FIQ exception request is disabled (section 3.4.4).
* There is no support for GIC virtualization. If the GIC on your board is version 1, then you must trap and emulate all accesses to the GIC from the guest. With GIC version 2, the GIC handles a portion of the emulation in hardware.
* There is no builtin support for disabling IRQ signal bypass in the GIC. If legacy IRQs are connected to the CPU in a bypass  or redundant arrangement, they must be disabled using some implementation defined fashion.

Both version 1 and version 2 support an optional Security Extensions feature. Only version 2 GICs support an optional Virtualization Extensions feature.

For a GIC version 2 implementation:
* Version 2 GIC implementations have a fuller set of features and capabilities added to the "Interrupt grouping" feature. Of note, group0 IRQs can be routed to the CPU as both IRQs and FIQs, whereas group1 IRQs can only be delivered as IRQs.
* Version 2 GICs support an optional Virtualization Extensions feature. This feature supports hardware emulation of the GIC for guest VMs. Along with these extensions comes support for hardware save and restore of GIC state (for supporting multiple guest VMs).
* Version 2 GICs support splitting the handling of IRQs into two stages: "Priority Drop" and "Interrupt Deactivation". The purpose of this split is to allow a Hypervisor to receive a greater number of actual physical IRQs, and dispatch them to guest VMs, and drop the priority of those IRQs the hypervisor has dispatched, without completely deactivating them. See the EOI section below.
* Support for disabling IRQ bypass signals at the GIC CPU Interface.

==IRQ grouping==
According to section 5.1, the ARM model is designed to assume that secure IRQs are delivered as FIQ-signals to the processor, and non-secure IRQs are delivered as IRQ-signals to the processor:
<pr>
In the ARM model for virtualizing Non-secure operation of a processor that implements the ARM Virtualization
Extensions, Secure software on the processor must configure the system as described in Using IRQs and FIQs to
provide Non-secure and Secure interrupts on page 3-68, so that FIQs are used for Secure interrupts, and IRQs for
Non-secure interrupts.
</pr>

==IRQ states==
The specification defines IRQs as being in one of the following states:
* Inactive: not asserted.
* Pending: This IRQ signal has been asserted by its hardware device, but not delivered to a CPU as yet.
* Active: This IRQ signal has been asserted and delivered to a processor, but the processor has not yet finished servicing it.
* Active and pending: This IRQ signal has been asserted and delivered to a processor, but the processor has not yet finished servicing it, and the signal has been re-asserted by a device.

=== Level-triggered IRQs ===

For SGIs, PPIs and SPIs, when a CPU acknowledges an IRQ (Reads from GICC_IAR/GICC_AIAR), the Distributor changes the status of the IRQ from active to pending if it is a level-triggered IRQ and the device has deasserted the level on the line. The Distributor changes the status of the IRQ from active to "active and pending" if it is a level-triggered IRQ and the device has not deasserted the level on the line.

=== Edge-triggered IRQs ===

For SGIs, PPIs and SPIs, when a CPU acknowledges an IRQ (Reads from GICC_IAR/GICC_AIAR), the Distributor changes the status of the IRQ from active to pending if it is an edge-triggered IRQ and the Distributor has not detected another edge on the line. The Distributor changes the status of the IRQ from active to "active and pending" if it is a level-triggered IRQ and the Distributor has detected another edge on the line.

==IRQ types and properties==
Four categories of IRQs exist:

===Peripheral IRQs===
These IRQs are asserted by external or internal devices. External devices are those which exist on the chipset and are not part of the processor itself. Internal devices are devices such as the ARM Generic Timers which are part of the ARM processor specification itself.

External IRQs are called '''Shared Peripheral Interrupts''' by the specification, and internally generated IRQs are called '''Private Peripheral Interrupts''' by the specification.

Both types of peripheral IRQ can be either '''level or edge sensitive'''.

===Software Generated IRQs===
These are ARM's notion of Inter-processor Interrupts (IPIs). They are always '''edge sensitive'''.

When an SGI occurs, the "CPUID" field of the GIC Interrupt Acknowledge Register, or the GIC Aliased Interrupt Acknowledge Register contains the CPUID of the processor that ''requested'' the IRQ.

===Virtual IRQs===
These are IRQs that have been injected by a hypervisor into a guest VM's Virtual-GIC.

===GIC Maintenance IRQs===
These are used to signal changes to the hypervisor, when the guest VM changes the state of its VGIC. Maintenance IRQs are always '''level-sensitive'''.

===IRQ Bypass===
GIC implementations enable the chipset to connect connect IRQs directly to the CPU when the GICC is disabled. In this setup, nIRQ and nFIQ can be directly tied to some other IRQ controller or signal. In order to disable the GIC, the chipset must drive GICCDISABLE to high. This is somewhat analogous to ExtINT on x86 CPUs.


==General==

===SGIs===
For SGIs, an IRQ is identified by the combination of the source CPUID, target CPUID and the IRQ number (0-15 are the IDs that can be used for an SGI). The GIC ignores accesses to its registers from any CPU in the system that is not connected to a CPU interface. In a uniprocessor system, the source of an SGI will always be 0. ARM recommends that system software splits the SGI IRQ IDs into two ranges: 0-7 for secure SGIs and 8-15 for non-secure SGIs.

An SGI can be generated such that it targets multiple processors, but presents itself with the same IRQ ID to each processor it targets. The SGI IRQ configuration registers are banked at the Distributor.

===Spurious IRQs===
A spurious IRQ is one where the GIC determines that an IRQ it has delivered to a processor does not need to be (and should not be) EOId. If this occurs, the GIC will return the value 1023 as the IRQ ID, when the GICC_IAR register is read.

When using the 1-N model for an IRQ, the version 2 GIC must ensure (Section 3.2.3) that only the first CPU to read GICC_IAR gets the IRQ's correct ID, and the rest of the CPUs, if any others got the IRQ, must get the spurious IRQ value. The version 1 GIC does not have this constraint specified, and it may return the correct ID to more than one processor.

A spurious IRQ ID may also be returned when IRQ grouping is being used, and the highest priority pending IRQ is a group0 IRQ, but secure-world software reads GICC_AIAR instead of GICC_IAR. Additionally, in general, any read of the GICC_IAR from a world that does not match the security of the highest priority active IRQ "does not acknowledge any interrupt", and will return:

* 1022 for a Secure read when the highest priority interrupt is Non-secure
* 1023 for a Non-secure read when the highest priority interrupt is Secure.

So if by chance non-secure software gets an IRQ which should be delivered to secure-world, the non-secure software will see it as a spurious IRQ (1023). If secure world gets an IRQ that pertains to non-secure world, it will read a special value (1022) to indicate to it that the IRQ is not for it; If the secure software still wishes to handle the IRQ, it can then acknowledge it by instead reading GICC_AIAR.

ARM "strongly recommends" (section 3.4.3) using a software IRQ handling model where <tt>GICC_CTLR.AckCtl == 0</tt>. In such a setup, every secure world IRQ must be higher priority than every non-secure IRQ.

==Acknowledging EOI==
An IRQ is taken from inactive to pending when it is asserted by its device. The IRQ is then taken from pending to active when the CPU reads the Interrupt Acknowledge Register. Finally, the IRQ is fully retired and the EOI signal is sent, when the CPU writes to the End of Interrupt register.

These steps apply also for Virtual GICs, which behave, from the guest VM's perspective, as if they are no different from a physical GIC. However, for a VGIC, each of these steps may also update the List registers. Additionally, for a virtual IRQ that was delivered to a guest VM through an update of the List registers of the virtual Distributor, the EOI that the guest VM does at the VGIC automatically propagates through to the physical Distributor (Section 5.2):

<quote>
When the virtual machine handles a virtual interrupt, it writes to the virtual CPU interface to indicate when it has
finished this processing. The virtual CPU interface signals this completion to the physical Distributor and the
physical Distributor then deactivates the interrupt.
</quote>

The split IRQ handling feature splits IRQ handling into two stages:
* Priority drop: when a hypervisor receives a physical IRQ, it immediately dispatches a virtual IRQ to the VGIC of the guest VM. Then it sends an EOI by writing to the GICC_EOIR or GICC_AEOIR. The running priority of the physical GIC is lowered by the EOI, so that if the IRQ's priority was high, the IRQ won't continue to block lower priority IRQs from being received by the hypervisor.
* The guest VM then handles the virtual IRQ and sends its own EOI by writing to GICV_EOIR or GICV_AEOIR. If <tt>GICV_CTRLR.EOIMode == 0</tt>, this will automatically cause a write to GICV_DIR as well, which will then cascade a write to GICC_DIR. If <tt>GICV_CTRLR.EOIMode != 0</tt>, the guest VM will have to also issue a write to GICV_DIR to deactivate the virtual IRQ, and this will then also automatically cascade a write to GICC_DIR.

IRQ splitting is enabled by setting GICH_LRn.HW to 1. If GICH_LRn.HW is not set to 1, the hypervisor must manually deactivate the physical IRQ when it has been handled by the guest VM. This can be done with the assistance of an EOI Maintenance IRQ.

==Virtualization Extensions==
The Hypervisor must maintain a set "List" registers in the GICH.

===CPU Interface register access===
In a GIC that supports the Virtualization Extensions, access to the VGIC CPU Interface for each CPU is '''mandated''' by the specification via both the following mechanisms:

* Common base address: Each processor can access its VGIC at the same fixed physical address, and the GIC hardware logic will determine which VGIC register set should be accessed based on the CPUID of the requesting processor.
* Processor-specific base address: Each VGIC is also mapped at its own physical address, and this allows one CPU to access the VGIC of any other CPU.

==See Also==
===External Links===
* [http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ihi0048b/index.html|ARM GIC Specification]
* [http://www.wiki.xilinx.com/GIC|Xilinx GIC Wiki page]
* [http://ssup2.iptime.org/wiki/ARM_Generic_Interrupt_Controller_(GIC)| Another Wiki page]

[[Category:ARM]]
[[Category:Interrupts]]
````

<a id="w004007"></a>

## W004007 — User:Zenzizenzicube

Bron-ID: W004007
Namespace: User (2)
Revisie: 20061 | Datum: 2016-12-28T11:44:43Z
Bron: https://wiki.osdev.org/User:Zenzizenzicube
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AZenzizenzicube&oldid=20061

````text
I'm a happy developer working on an operating system and compiler as a hobby.

I also manage a [https://wiki.compilerdev.org wiki] and [https://forum.compilerdev.org forum] on compiler development with [[User:Glauxosdever]] if you wanted to check that out.

Feel free to ask me anything but I'm not very useful :)
````

<a id="w004008"></a>

## W004008 — User:Bauen1

Bron-ID: W004008
Namespace: User (2)
Revisie: 25082 | Datum: 2020-08-23T15:22:15Z
Bron: https://wiki.osdev.org/User:Bauen1
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABauen1&oldid=25082

````text
Working on [https://gitlab.com/myunix/myunix  myunix (v3)]
````

<a id="w004010"></a>

## W004010 — Talk:Mister Perfect

Bron-ID: W004010
Namespace: Talk (1)
Revisie: 20100 | Datum: 2017-01-03T09:07:30Z
Bron: https://wiki.osdev.org/Talk:Mister_Perfect
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMister_Perfect&oldid=20100

````text
= What is this? =

[[User:Glauxosdever]] at first glance, this seems like passive-aggressive provocation. Is that what it is?

I'm not sure how you got that idea from, but that's not what I intended at all. [[User:Glauxosdever|Glauxosdever]] 03:07, 3 January 2017 (CST)
````

<a id="w004011"></a>

## W004011 — Generic Interrupt Controller versions 3 and 4

Bron-ID: W004011
Namespace: Main (0)
Revisie: 30720 | Datum: 2026-06-11T00:24:39Z
Bron: https://wiki.osdev.org/Generic_Interrupt_Controller_versions_3_and_4
Revisie-URL: https://wiki.osdev.org/index.php?title=Generic_Interrupt_Controller_versions_3_and_4&oldid=30720

````text
= Introduction =
This page seeks to give a quick reference to the behaviour of the version 3 and 4 GIC, especially relative to version 2. It heavily references [[Generic Interrupt Controller]], and recommends that page as prior reading.

Version 3 of the GIC specification is no longer separate from the core ARM specification, but as of version 3, the GICv3 is part of the architectural state of the ARM architecture, from '''ARMv8 and onward'''. In particular, from ARMv8 onwards, access to the GICC_*, GICV_* and GICH_* groups of registers are provided as ARM architected system registers, and not only as memory-mapped registers. In other words, they are accessible as ARM coprocessors. The GICD_*, GICR_* and GICS_* groups of registers however, are always memory mapped.

On an ARMv7 platform, the GIC is an extension that is not architecturally specified. On ARMv8 platforms, the GIC, if implemented, is architecturally specified.

Another key improvement over GICv2 is that CPUIDs in GICv3 are no longer restricted to a maximum of 8 processor IDs, and so GICv3 and upwards supports the delivery of IRQs to more than 8 Processing Elements (PEs).

= Version difference summary =

== Differences between GICv2 and GICv3 ==

== Differences between GICv3 and GICv4 ==

= Interrupt Grouping changes =

In GICv3, the meaning of interrupt Grouping has been expanded a bit; in GICv2, group0 was simply about secure IRQs, and group1 was about non-secure IRQs. This remains the same in GICv2. However, there are slight changes to the way that group1 IRQs are handled. In a system with two security states, an IRQ can either be:
* Group0: ARM expects that these will be handled at EL3 (Monitor mode).
* Secure Group1: ARM expects that these will be handled by Secure PL1.
* Non-secure Group1: ARM expects that these will be handled by non-secure PL2 (Hyp mode) or non-secure PL1 (kernel/supervisor mode). Essentially, if a Hypervisor exists, then the Hypervisor will handle these and re-inject them into the guest VM as virtual IRQs; if there is no hypervisor, then these will be handled by the PL1 kernel directly.

GICD_CTLR.DS indicates if the GIC is configured with one or two security states; or more specifically, GICD_CTLR.DS indicates whether or not the GIC implements a security model compatible with the ARMv8 security model.

In a system where there is only one security state supported (no Security Extensions), IRQs can be configured as either:
* Group0
* Group1

'''LPIs are always Group1''' IRQs. In a GICv3 implementation that has 2 security states, they are specifically '''non-secure Group1''' IRQs.

== Acknowledging and EOIing Group0 IRQs ==
For group0 IRQs, software should attempt to manage them using: ICC_IAR0_EL1, ICC_EOIR0_EL1, ICC_BPR0_EL1, ICC_HPPIR0_EL1, and finally ICC_IGRPEN0_EL1 to enable Group0 IRQs at the CPU interface.

== Acknowledging and EOIing Group1 IRQs ==
For group1 IRQs, software should attempt to manage them using: ICC_IAR1_EL1, ICC_EOIR1_EL1, ICC_BPR1_EL1, ICC_HPPIR1_EL1, and finally ICC_IGRPEN1_EL1 to enable Group1 IRQs at the CPU interface '''for the target security state''' (secure vs non-secure PL1).

= Redistributors =

Redistributors are a new component of the IRI (Interrupt Routing Infrastructure) that are responsible for holding information about all pending physical LPIs (MSI IRQs). In a GICv4 implementation, a Redistributor also holds information about pending virtual LPIs, enabling the injection of virtual MSI IRQs into CPUs (which is not supported in hardware in GICv3; a hypervisor must emulate the injection of a virtual MSI IRQ in GICv3).

= Affinity Routing =

Introduced in GICv3, Affinity routing is a form of specifying PE node IDs in a multiprocessor system using a 32-bit integer that is split into 4 subcomponents: a, b, c and d. If you want to use all 4 levels of addressing, you must be running on an AArch64 processor. AArch32 only supports 3 levels.
* A 4-level identifier is of the form a.b.c.d.
* A 3-level identifier is of the form 0.b.c.d, where the "a" component must be 0.

To enable affinity-based routing in the Distributor for Secure IRQs, set GICD_CTLR.ARE_S to 1. To enable affinity-based routing in the Distributor for non-secure IRQs, set GICD_CTLR.ARE_NS to 1. Changing the value of either or both of these bits from 1 to 0 is <UNPREDICTABLE>. Changing the value from 0 to 1 is <UNPREDICTABLE> unless both groups of interrupts (group0 and group1) are disabled. Specifically:
* If GICD_CTLR.DS == 0, changing the value from 0 to 1 is <UNPREDICTABLE> when:
** GICD_CTLR.EnableGrp0 == 1.
** GICD_CTLR.EnableGrp1S == 1.
** GICD_CTLR.EnableGrp1NS == 1.

* If GICD_CTLR.DS == 1, then changing the value from 0 to 1 is <UNPREDICTABLE> when:
** GICD_CTLR.EnableGrp0 == 1.
** GICD_CTLR.EnableGrp1 == 1.

Software (the kernel) should ensure that the Distributor has commited the effects of these writes before proceeding, by polling GICD_CTLR.RWP before continuing.

Some GIC implementations may have affinity based routing permanently enabled. In such a case, GICD_CTLR.ARE_S and GICD_CTLR.ARE_NS are both RAO/WI.

== Affinity routing mode with SPIs ==
SPIs are routed using the GICD_IROUTER<n>.Interrupt_Routing_Mode bit:
* If GICD_IROUTER<n>.Interrupt_Routing_Mode == 0, SPIs are forwarded to a single target PE, matching a.b.c.d.
* If GICD_IROUTER<n>.Interrupt_Routing_Mode == 1, SPIs are forwarded to all PEs that qualify as a "participating node".

== Affinity routing mode with SGIs ==
SGIs are routed using the ICC_SGI0R_EL1.IRM and ICC_SGI1R_EL1.IRM bit:
* If the IRM bit is set to 1, SGIs are broadcast to all participating PEs in the system, excluding the originating PE (all-but-self).
* If the IRM bit is set to 0, SGIs are routed to a group of PEs, selected according to a.b.d.<TARGET_LIST>. TARGET_LIST is a bitfield for encoding values from 0-15.

== Participating nodes ==
Participating nodes are those PEs which are setup to be ready to receive an interrupt which is being delivered in 1 of N mode. In order to prepare a PE to take a 1 of N routed interrupt, that PE must be configured as follows:

* GICR_WAKER.ProcessorSleep == 0, and the interrupt group of the interrupt must be enabled on that PE.
* GICD_CTLR.E1NWF == 1.
* GICR_TYPER.DPGS == 1. (DPGS = Disable Processor Group Selections). Furthermore, GICR_CTLR.{DPG1S, DPG1NS, DPG0} == 0.

= IntIDs =

Interrupt IDs are the unique identifiers for an IRQ. The number of IntIDs available in hardware at the Distributor '''and Redistributor''' is limited to a 10-bit space if LPIs are not supported. If LPIs are supported, the IntID space is of bit-width <IMPLEMENTATION DEFINED>, between 14 and 24 bits. Read GICD_TYPER to find out the bit-width of the IntID space in this case.

The number of IntIDs supported by the ITS (if LPIs are supported) is defined as a bit-width that is IMPLEMENTATION DEFINED between 14-and 24 bits. Read GITS_TYPER to discover the bit-width of the IntID space. Given that the ITS is used for the purpose of managing LPIs, one can assume that if an ITS exists, then LPIs also exist, but this is just my personal conclusion. Be careful to only program the ITS to forward IRQs whose IntIDs are within the range of the target Redistributor's IntID bit-space.

The number of IntIDs supported by the GICv3 CPU Interface is is '''either''' an IMPLEMENTATION DEFINED 16 bits or 24 bits. Check ICC_CTLR_EL1.IDbits and ICC_CTLR_EL3.IDbits.

ARM imposes the following constraints on GIC implementations:

* PEs (Processor Interfaces) must implement either 16 or 24 bits of IntID. However, a system might have a mixture of PEs that support both sizes!
* The ''Distributor and Redistributors must all implement the same number'' of ID bits. The ID bits for the Distributor and Redistributor must not exceed the minimum IntID size implemented by any single PE in the system. I.e, the Distributor and Redistributor IntID bit-width cannot must be smaller than or equal to the bit width of the smallest PE bit-width.
* If LPIs are ''not'' supported, the Distributor and Redistributors must implement at least 5 bits of IntID, and at most 10 bits of IntID.
* If LPIs are supported, the Distributor and Redistributors must support at least 14 bits of IntID.
* If an ITS exists, the ITS may implement any bit-width up to or equal to the bit-width of the Redistributors; and this bit width must be a minimum of 14 bits.

== IntID space division ==

* ID0-ID15: Software Generated Interrupts (SGIs.)
* ID16-ID31: Private Peripheral Interrupts (PPIs.)
* ID32-ID1019: Shared Peripheral Interrupts (SPIs.)
* ID1020-ID1023: Special interrupt IDs.
* ID1024-ID8191: Reserved.
* ID8192-<IMPLEMENTATION DEFINED MAX BIT WIDTH>: Locality-based Peripheral Interrupts (LPIs.)

Orthogonal insertion: GICv4 assists in making IntIDs unique per-guest VM by providing a vPEID in addition to the IntID space.

== ARM's Recommended default mapping of private peripheral IRQs to PPI IDs ==

This is ARM's recommended mapping; SoC designers do not need to conform to it. Where the SoC implementation differs, the core or SoC or platform documentation will state what the PPI IntIDs are. That being said, ARM recommneds that the following mapping be used:

* ID30: Non-secure Physical Timer IRQ.
* ID29: Secure Physical Timer IRQ.
* ID27: Virtual Timer IRQ.
* ID26: Hypervisor Timer Interrupt.
* ID25: Virtual CPU Interface Maintenance IRQ (VGIC Maintenance IRQ).
* ID24: Cross Trigger Interface IRQ.
* ID23: PMC (Perf Monitor Counter) Overflow IRQ.
* ID22: DCC (Debug Communications Channel) IRQ.

== Special IntIDs (and changes from GICv2) ==

In GICv2, the special IntID 1022 is used to signal to the Secure Monitor that the current highest priority IRQ was meant to be handled in non-secure world (most likely by the hypervisor). In GICv3 onwards, this value is deprecated.

In GICv2 and GICv3 onwards, IntID 1023 is used to signal a Spurious IRQ, or to state to a non-secure world handler that the current highest-priority IRQ is meant to be handled by the secure world. In both cases, the non-secure reader would just ignore the interrupt anyway.

In GICv3 onwards, two new special IntIDs are introduced:
* ID1020: The GIC returns this value when ICC_IAR0_EL1 or ICC_HPPIR0_EL1 is read from EL3 (Secure Monitor Mode), but the current highest priority IRQ is meant to be handled by Secure EL1.
* ID1021: The GIC returns this value when ICC_IAR0_EL1 or ICC_HPPIR0_EL1 is read from EL3 (Secure Monitor Mode), but the current highest priority IRQ is meant to be handled by non-Secure EL1.

**These IntIDs do not require an EOI or deactivation.**

=== Interrupt acknowledgment ===
Interrupt acknowledgement is accomplished through a read from ICC_IAR0_EL1 for group0 IRQs, and it is accomplished via a read from ICC_IAR1_EL1 for group1 IRQs.

=== EOI ===
EOI is accomplished via a write to ICC_EOIR0_EL1 for group0 IRQs, and it is accomplished via a read from ICC_EOIR1_EL1 for group1 IRQs.

= Message-Based IRQ support =

The ITS is one method of supporting MSI IRQs on ARM platforms that implement the GIC architecture. There are two approaches, and a platform must support '''only one''' (GICv3&4 specification, section 6.2):
* The device generates an LPI by writing directly to the Redistributor's GICR_SETLPIR.
* The device generates an LPI via the [[Interrupt Translation Service]], which will generate the LPI on its behalf.

In GICv3 the ITS is optional, one option (ITS vs no-ITS). In GICv4, there must be at least one ITS, in order to support the injection of virtual LPIs into vPEs (so GICv4 must choose the ITS approach, and it does not have the option of using the non-ITS approach).

== Locality-based Peripheral IRQ ==

Support for a new type of IRQ is added from GICv3 onwards: Locality-based Peripheral IRQs (LPIs), which are used for message-based IRQs. LPIs are not routed through the Distributor, but they are routed directly to Redistributors, which are connected directly to PEs. The Distributor manages pin-based IRQs (SPIs, PPIs and SGIs) while the ITS+Redistributors manage MSIs.

LPIs can be generated either by writing to a Redistributor's GICR_SETLPIR, or through the ITS.

'''LPIs are always edge-triggered.''' (GICv3&4 specification, section 1.2.1).
When LPIs are acknowledged (read from the IAR), their pending state goes from pending to not-pending in the '''Redistributor'''. The Redistributor does not maintain an active state for LPIs.

Section 4.2 explains that "In a system where two Security states are enabled, LPIs are always non-secure Group 1 IRQs".

== Interrupt Translation Service ==

{{Main|Interrupt Translation Service}}

Note that the configuration of ITS should have no effect on SPIs, SGI, PPIs (GICv3&4 specification, section 6.2: "An ITS has no effect on SGIs, SPIs or PPIs."). Its purpose is confined to generating IRQ signals from in-band bus messages.

For the GIC architecture, ARM devices that support MSI (message-signaled IRQs) are each assigned a DeviceID. Additionally, each MSI IRQ that the device will generate is assigned an EventID. This combination of DeviceID+EventID are used as inputs to the ITS. The ITS then uses the DeviceID to index into its Device Table, and select that device's Interrupt Translation Table (ITT). The EventID is then used to index into the ITS' ITT for that device, and select the pertinent Interrupt Translation Entry (ITE).

Each ITE then indicates a Collection Table (CT) entry, which gives the target Redistributor for a physical MSI IRQ. For GIVc3, if a device has a physical MSI IRQ, and the OS Hypervisor would like to virtualize that IRQ to a Guest VM, the hypervisor must deliver that IRQ as a standard virtual IRQ (Not 100% sure on this yet).

GICv4 also adds support for delivering virtual MSI IRQs. In this case, the ITE indicates a vPEID, which indicates a vPE table entry (and therefore, the physical Redistributor that hosts the target vPE for the virtual MSI).

These tables may seem tedious, and that is because they are; but thankfully they are not exposed to the hypervisor. The hypervisor is required to simply update the global IPL configuration table, and then issue INVALIDATE (INV and INVALL) commands to the ITS machinery (if it exists), so that the ITS can read the new configuration values. It is not possible to directly access the ITS' internal tables (GICv3&4 specification, section 6.2). The hypervisor should also issue a SYNC command to the ITS if it needs to be sure that the INVALIDATE command has completed before execution continues.

''Where there is no ITS hardware implemented, the hypervisor performs invalidation of LPI configuration via writes to GICR_INVALLR (invalidate all entries) or GICR_INVLPIR (invalidate single entry).''

Caveat: All the ITS' tables are stored in non-secure physical memory.
Author's comment: It appears that the entire GIC MSI IRQ architecture is based on routing of MSI IRQs directly to a Redistributor, or indirectly to a Redistributor through an ITS.

= References =

* [[http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.subset.architecture.gic/index.html|ARM GIC version 3 and 4 specifiction]] (Click on the arrow icon next to "Contents" to get a link to the PDF from ARM.)
* [[http://www.wiki.xilinx.com/GIC|Xilinx GIC Wiki page]]
* [[http://ssup2.iptime.org/wiki/ARM_Generic_Interrupt_Controller_(GIC)| Another Wiki page]]

[[Category:ARM]]
[[Category:Interrupts]]
````

<a id="w004013"></a>

## W004013 — AVX

Bron-ID: W004013
Namespace: Main (0)
Revisie: 20133 | Datum: 2017-01-09T08:53:36Z
Bron: https://wiki.osdev.org/AVX
Revisie-URL: https://wiki.osdev.org/index.php?title=AVX&oldid=20133
Doorverwijzing: SSE

````text
#REDIRECT[[SSE#AVX]]
````

<a id="w004014"></a>

## W004014 — User:Naxnax

Bron-ID: W004014
Namespace: User (2)
Revisie: 20112 | Datum: 2017-01-06T20:08:17Z
Bron: https://wiki.osdev.org/User:Naxnax
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANaxnax&oldid=20112

````text
I like cats.
````

<a id="w004015"></a>

## W004015 — User talk:Naxnax

Bron-ID: W004015
Namespace: User talk (3)
Revisie: 20113 | Datum: 2017-01-06T20:08:46Z
Bron: https://wiki.osdev.org/User_talk:Naxnax
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ANaxnax&oldid=20113

````text
The red link bothered me.
````

<a id="w004016"></a>

## W004016 — Automated Build Using CircleCI

Bron-ID: W004016
Namespace: Main (0)
Revisie: 20119 | Datum: 2017-01-07T19:29:55Z
Bron: https://wiki.osdev.org/Automated_Build_Using_CircleCI
Revisie-URL: https://wiki.osdev.org/index.php?title=Automated_Build_Using_CircleCI&oldid=20119

````text
{{rating|2}}
This tutorial will show you how to set up an automated build environment using [https://circleci.com/ CircleCI], which will automatically build your project on every push to a repository, and make the resulting files available on the web. In the tutorial it is assumed that your kernel / operating system project is hosted at [https://github.com/ GitHub], from where it can be cloned and built using a freestanding [[GCC Cross-Compiler]] toolchain.

== Setting up your project on CircleCI ==

First you will need to sign up at [https://circleci.com/ CircleCI]. Navigate to their homepage and sign in. You will then have the possibility to connect with your GitHub account. GitHub will ask you whether you would like to grant CircleCI access to your account. Once you have confirmed, you can enter your CircleCI settings. Choose a project to build to enable automated builds whenever you push to that project.

== Add a build script for the toolchain ==

The first step towards building your project should be building the toolchain, and thus in particular a [[GCC Cross-Compiler]]. The most convenient way to do this is to create a small script that will download the sources for binutils and gcc (and optionally additional sources, such as newlib or gdb), configure them for your chosen target and build the tools. The following script is an example, which builds a toolchain suitable for using a subset of C++ (only the freestanding part):

<source lang="bash">
#!/bin/sh
set -e

# Set the versions we will be using.
binutils_version="2.27"
gcc_version="6.3.0"
newlib_version="2.4.0"

# This script expects the target triplet (e.g. i786-pc-elf) as command line argument.
target=$1

# The tools will be installed in ~/cross/$target.
prefix=~/cross/$target

# First check whether the toolchain was already built on a previous run of this script.
if [ ! -d $prefix ]
then
	mkdir -p /tmp/toolchain
	cd /tmp/toolchain

	# Download gcc sources if they are not yet downloaded.
	if [ ! -f gcc-$gcc_version.tar.bz2 ]
	then
		wget -c -O gcc-$gcc_version.tar.bz2 ftp://ftp.gnu.org/gnu/gcc/gcc-$gcc_version/gcc-$gcc_version.tar.bz2
		tar -xf gcc-$gcc_version.tar.bz2
	fi

	# Download binutils sources if they are not yet downloaded.
	if [ ! -f binutils-$binutils_version.tar.bz2 ]
	then
		wget -c -O binutils-$binutils_version.tar.bz2 ftp://ftp.gnu.org/gnu/binutils/binutils-$binutils_version.tar.bz2
		tar -xf binutils-$binutils_version.tar.bz2
	fi

	# Optional: download newlib sources if they are not yet downloaded.
	if [ ! -f newlib-$newlib_version.tar.gz ]
	then
		wget -c -O newlib-$newlib_version.tar.gz ftp://sources.redhat.com/pub/newlib/newlib-$newlib_version.tar.gz
		tar -xf newlib-$newlib_version.tar.gz
	fi

	# Create build paths.
	mkdir -p /tmp/toolchain/build-binutils
	mkdir -p /tmp/toolchain/build-gcc
	mkdir -p /tmp/toolchain/build-newlib

	# Build binutils.
	cd /tmp/toolchain/build-binutils
	sudo rm -rf *
	/tmp/toolchain/binutils-$binutils_version/configure --target=$target --prefix=$prefix --disable-nls 2>&1
	make all 2>&1
	make install 2>&1
	sudo rm -rf *

	# Build gcc and libgcc.
	cd /tmp/toolchain/build-gcc
	/tmp/toolchain/gcc-$gcc_version/configure --target=$target --prefix=$prefix --disable-nls --enable-languages=c,c++ --enable-libstdcxx --without-headers 2>&1
	make all-gcc 2>&1
	make install-gcc 2>&1
	make all-target-libgcc 2>&1
	make install-target-libgcc 2>&1

	# Make sure that our cross compiler will be found by creating links.
	# Alternative: Add the $prefix/bin directory to your $PATH.
	sudo ln -s -f $prefix/bin/* /usr/local/bin/

	# Optional: Build newlib. This is necessary only for the next, also optional build step.
	cd /tmp/toolchain/build-newlib
	sudo rm -rf *
	/tmp/toolchain/newlib-$newlib_version/configure --target=$target --prefix=$prefix 2>&1
	make all 2>&1
	make install 2>&1
	sudo rm -rf *

	# Optional: Build libstdc++. This is done in order to install the freestanding headers for using the C++11, C++14, C++17 standards.
	cd /tmp/toolchain/build-gcc
	/tmp/toolchain/gcc-$gcc_version/configure --target=$target --prefix=$prefix --disable-nls --enable-languages=c,c++ --enable-libstdcxx --without-headers --with-newlib 2>&1
	make all-target-libstdc++-v3 2>&1
	make install-target-libstdc++-v3 2>&1
	sudo rm -rf *
fi

# Also if the cross compiler has not been freshly build, link it so that it will be found.
sudo ln -s -f $prefix/bin/* /usr/local/bin/
</source>

Note that this is only an example script. You will most likely have to modify it according to your needs. Also you should test locally whether it actually works and generates the compiler you will use later. Once you have done this, add it to your source repository. Here we will assume that you have created a directory called '''ci''' in the root of your repository, and placed your script as a file named '''toolchain.sh''' in this folder.

== Set up the build ==

Now we need to tell CircleCI which actions it should perform when you push to your repository. By default, CircleCI uses a virtual machine running Ubuntu 14.04 for your build. This is configured using a file called '''circle.yml''' in the root of your repository. The following example explains the sections in this file.

<source lang="yaml">
dependencies:
    pre:
        # Install a few packages which will be needed for building the cross compiler.
        - sudo apt-get update
        - sudo apt-get install grub-pc
        - sudo apt-get install libgmp3-dev
        - sudo apt-get install libmpfr-dev
        - sudo apt-get install libmpc-dev
        - sudo apt-get install texinfo
        - sudo apt-get install libisl-dev
        - sudo apt-get install libcloog-isl-dev

        # Build the cross compiler for your chosen target (replace i786-pc-elf if necessary).
        - bash ci/toolchain.sh i786-pc-elf

    cache_directories:
        # Add the folder with our cross compiler to the cache. This way it will be kept between builds.
        - ~/cross

compile:
    override:
        # Place here whatever commands are necessary to build your project.
        - bash your_build_script.sh

        # Copy any files you want to keep to the folder $CIRCLE_ARTIFACTS.
        - cp my_kernel my_image.iso $CIRCLE_ARTIFACTS/

test:
    override:
        # Place here any automated test you would like to run after the build (unit tests, boot up a VM and collect output...)
        - bash your_test_suite.sh

        # You must place something here, even it it's just testing whether the output exists.
        - test -f $CIRCLE_ARTIFACTS/my_kernel
        - test -f $CIRCLE_ARTIFACTS/my_image.iso
</source>

Ones you have created a custom file of this type, save it as '''circle.yml''' in the root of your repository, commit and push. Now CircleCI should start building your cross compiler, go on with building your project, collect the output files, check for success and inform you whether the build succeeded. The next time you push, all cached directories (and thus your cross compiler) will be restored before the build starts (and so your cross compiler will not be built again on every push).

== Publish outputs on the web ==

Both on CircleCI and on the commits page or your GitHub project you should now see whether your build was successful. If you would also like to show the build status on your project homepage, you can use a link of one of these forms:

<pre>https://circleci.com/gh/<your_user_name>/<your_project_name>.svg?style=svg
https://circleci.com/gh/<your_user_name>/<your_project_name>.svg?style=shield</pre>

You can also create links to the files you save in the $CIRCLE_ARTIFACTS folder in your build. Here the link should look like this:

<pre>https://circleci.com/api/v1/project/<your_user_name>/<your_project_name>/latest/artifacts/0/$CIRCLE_ARTIFACTS/<your_file_name></pre>

This will always link to the latest generated files.

== Links ==

* [https://circleci.com/docs/configuration/ Configuring CircleCI]
* [https://circleci.com/docs/status-badges/ CircleCI build status badges]
* [https://circleci.com/docs/build-artifacts/ CircleCI build artifacts]

[[Category:Automated Builds]]
````

<a id="w004017"></a>

## W004017 — ARMv7 Common Memory System Architecture

Bron-ID: W004017
Namespace: Main (0)
Revisie: 27995 | Datum: 2023-07-09T17:39:04Z
Bron: https://wiki.osdev.org/ARMv7_Common_Memory_System_Architecture
Revisie-URL: https://wiki.osdev.org/index.php?title=ARMv7_Common_Memory_System_Architecture&oldid=27995

````text
= Introduction =
This article seeks to provide a concise summary of the ARM '''version 7''' Common Memory System Architecture, which is a very convoluted name for the ARM Caching and Branch Prediction architecture. ARM has 3 main "Memory Systems Architectures" which form the components of its memory access scheme:
* CMSA: Common Memory System Architecture: Caches and Branch Prediction. CMSA is common to all profiles, with adjustments as needed for the particular implementation and intended use of the processor.
* VMSA: Virtual Memory System Architecture: TLBs and Virtual Memory as a memory protection and isolation, and linearization scheme. VMSA is generally implemented by ARM profile-A processor implementations.
* PMSA: Protected Memory System Architecture: a simplified memory protection scheme, also known in general CPU design parlance as an MPU (Memory Protection Unit), often held in contrast to an MMU (Memory Management Unit). An MPU is generally preferred to the memory and speed overheads of an MMU, in very resource-constrained, or performance critical microcontrollers. PMSA is generally implemented by ARM profile-R processor implementations.

Any particular ARM implementation may have one or more of these "Memory Systems" (CMSA, VMSA and PMSA).

This article is not current with the ARM '''version 8''' CMSA.

'''Strong warning''': The ARM reference manuals are subject to change at the whims and discretion of ARM, and this article does not claim to be a current description of the behaviour of architected ARM features, on any given date. This is a concise summary of critical points for quick recollection, for a person who has '''already''' read the manuals. Do not use this as a primary learning source.

= Components of the CMSA =
== Caches ==

===Cache policies===
ARMv7 CMSA compliant processors have buffering and caching policies which, when combined in a non-LPAE-mode processor, give the following cache policies:
* Non-cacheable
* Write-through cacheable
* Write-back Write-allocate cacheable (aka, read-around or read-behind caching)
* Write-back no-write-allocate cacheable (aka write-around or write-behind caching)

An ARMv7 processor with the LPAE extensions enabled allows for more cache policies, as well as a cache-allocation hint.

===Cache allocation hints===
ARM processors' memory attributes include an allocation hint for the processor. This enables software to cue the processor into whether or not it should allocate cache lines for accesses to that memory area. The ARM specification does not require implementations to respect these hints. The hints are:
* Allocate
* Do-not allocate.

===Cache topology===
In ARM processors, the instruction and data/unified caches can be separately enabled and disabled. The Data/unified cache(s) are enabled as a group, and the instruction cache(s) are separately enabled as their own group. ARMv7 provides a way for software to reason about caches in a uniform manner across implementations. Prior to ARMv7, ARM only architecturally specified one level of cache and support for and management of all other levels of cache was <IMPLEMENTATION DEFINED>. ARMv7 introduces the concepts of '''Level of Unification''' and '''Level of Coherency''' in order for software to interact robustly with diverse caches.

====Level of Unification (verbatim)====
The PoU for a processor is the point by which the instruction and data caches and the translation table walks of that processor are guaranteed to see the same copy of a memory location. In many cases, the point of unification is the point in a uniprocessor memory system by which the instruction and data caches and the translation table walks have merged. The PoU for an Inner Shareable shareability domain is the point by which the instruction and data caches and the translation table walks of all the processors in that Inner Shareable shareability domain are guaranteed to see the same copy of a memory location.Defining this point permits self-modifying software to ensure future instruction fetches are associated with the modified version of the software by using the standard correctness policy of:

* Clean data cache entry by address.
* Invalidate instruction cache entry by address.

The PoU also permits a uniprocessor system that does not implement the Multiprocessing Extensions to use the clean data cache entry operation to ensure that all writes to the translation tables are '''visible to the translation table walk hardware.''' In other words, ARM TLBs walk the page tables from the Level of Unification, so page tables need only be cleaned to the level of Unification, and not the Level of Coherency.

====Level of Coherency (verbatim)====
For a particular MVA, the PoC is the point at which all agents that can access memory are guaranteed to see the same copy of a memory location. In many cases, this is effectively the main system memory, although the architecture does not prohibit the implementation of caches beyond the PoC that have no effect on the coherence between memory system agents.

"All agents" here refers to non-processor microcontrollers such as DMA-capable devices.

===Enumerating processor-controlled caches===

Notice that the title of this section heading is "Enumerating '''processor-controlled''' caches". This is because on any ARM platform, not all caches may be under the control of the processor. In an ARMv7 implementation where some cache is under the control of the processor however, this is how that cache is specified to be interacted with by ARM.

===Architected cache behaviour, caveats and guarantees===

====Properties of a cache system====
* Cache line size: The size of cache lines as allocated and evicted according to the cache policy. ARM allows cache-line sizes from 16Bytes to 2KiB (Section B2.2.2).
* Cache granule: The write-back size of the processor when a write-back policy is in use. The processor may write multiple cache lines back at once, and the size of the burst-transaction that is written at once, is the cache granule.
* Number of levels of caches: The number of cache levels which must be kept coherent in the cache system.
* The cache allocation algorithm: what policy dictates whether or not the processor will allocate a line into the cache for an access to a particular memory location. The cache eviction algorithm is not architecturally specified.
* Behaviour at major junctions in software execution such as exception entry.
* The presence or absence of '''speculative caching''' and what its behaviour is like.

====Cache lockdown====
Due to all the properties of caches (perhaps especially the lattermost), which factor into their behaviour, ARM cannot guarantee:
* Whether or not a particular memory location present in the cache will remain in the cache.
* Whether or not a particular memory location not allocated into the cache will be allocated into the cache.

Instead, ARM provides a "''Locked Entry''" mechanism, which pins a line into the cache. The implementation details behind a particular platform's cache lockdown are <IMPLEMENTATION DEFINED>, and it is not mandatory, and may not be supported by an implementation. Additionally there is no architected control over the feature, but it is to be described by the implementation documentation.
* A memory location cannot be relied upon to remain in the cache unless locked into the cache.
* If an unlocked entry remains in the cache for some duration, it cannot be relied on to remain incoherent with memory; it will be written back eventually, and software should not rely on it remaining dirty (why would anyone do this?).
* If an entry is locked, it can be relied on to remain in the cache; but it cannot be relied on to remain incoherent with memory, and again, software should not rely on it remaining dirty (again, why?).

''In ARMv6 the Cache Type Register held information about the Cache lockdown mechanism; in ARMv7 this is no longer the case.''

====General====
* Even if a memory location is accessible in the current translation scheme (the manual said scheme here, but probably meant "regime") and is accessible in the current privilege level, and is also marked cacheable for the current translation regime, there is "no mechanism that can guarantee" that the memory location '''cannot''' be allocated into an enabled cache at any point. This basically means that there is no architected secondary means of preventing a memory location from being cached if it is marked cachable (this should be obvious).
* If the cache is disabled, it is guaranteed that no new memory location will be allocated into the cache. Stale cache entries may still cause cache hits when the cache is disabled.
* If the cache is enabled, it is guaranteed that no memory location that does not have the Cacheable attribute set, will be allocated into the cache. If a memory location was already in the cache before being marked uncachable, there is no guarantee that it will be evicted.
* If the cache is enabled, it is guaranteed that no memory location that is not accessible to software at the current '''translation regime''' and '''privilege level (or higher)''' will be allocated into the cache. Again, stale entries already in the cache may cause cache hits.
* For data accesses, if a memory location is marked as ''Normal Shareable'', it is guaranteed to be coherent with all masters in that shareability domain.
* The eviction (author's interpretation of "eviction" here is a clean operation, not an invalidate) of a cache line from cache to memory cannot overwrite a memory location written by another observer in the cache system unless the two observers are in the same shareability domain.
* Verbatim: "The allocation of a memory location into a cache cannot cause the most recent value of that memory location to become invisible to an observer, if it had previously been visible to that observer". This appears to be stating that cache coherency is a given.

Additionally, along the same vein as the fact that stale entries in the cache might generate cache hits even with if caches are disabled, the following two caveats are given:
* If a location is marked not-Cacheable, but exists in the cache, there is no guarantee of whether the memory access will be returned from the caches or from memory.
* If a location is in the cache and is marked cacheable, but the cache is disabled.

===Cache state on #RESET===
On #RESET, all caches are disabled and before the Cache system is responsive to the ARMv7 architected cache controls, there may be an <IMPLEMENTATION SPECIFIC> procedure initialization routine that software may need to execute. Furthermore:
* It is <IMPLEMENTATION DEFINED> whether or not an access can cause a cache hit when caches are disabled. If cache hits to stale/garbage cache lines are possible while caches are disabled, the implementation must clearly document the correct way to initialize the platform caches.
* ARM ''recommends'' that invalidation routines conform to the ARM architected form.

If software enables the caches using the ARM architected mechanism before executing the <IMPLEMENTATION SPECIFIC> platform cache initialization routine, the behaviour of the caches is <UNPREDICTABLE>.

''Before ARMv7, caches were specified to be invalidated by the assertion of #RESET''.

== Branch predictors ==
The ARM architecturally visible Branch Prediction architecture consists of the Branch Target Buffer (BTB), the Pipeline, and the Instruction Prefetch Buffer (IPB). The following sections explain why software may need to care about these components. ARM allows the branch predictor to be visible to software and it is not architecturally hidden. Thus software must under certain circumstances perform maintenance operations to ensure desired behaviour.

A branch-predictor invalidate operation has no ''functional'' effect on software execution. The Invalidate BTB by MVA operation must use the address of the branch-target.

If branch prediction is architecturally visible, an Instruction-cache Invalidate-ALL operation also invalidates all branch predictors.

In general, ff for a given translation regime, VMID and ASID (each where appropriate), the instructions at a virtual address change, then invalidation is necessary to ensure that the change is visible to subsequent execution. The following events may require a BTB invalidate:
* Enabling or disabling the MMU (VMSA).
* Writing new mappings to the translation tables (changing page table entries).
* Changes to <tt>TTBR0</tt> ,<tt>TTBR1</tt> and <tt>TTBCR</tt>, except if accompanied by a change of VMID or ContextID.
* Changes to <tt>VTTBR</tt> or <tt>VTTCR</tt>, unless accompanied by a change of VMID.

Failure to perform appropriate BTB maintenance operations may result in <UNPREDICTABLE> behaviour as old stale branches may be executed.

Software does not need to invalidate the BTB in the following cases however:
* When changing the page table entry attributes for a page with instructions, but the change does not cause the virtual address to point to new instructions; i.e, only the permissions are changed, '''for the current translation regime, VMID and ASID'''.
* After changing the ContextID, ASID or FCSE ProcessID.
* After executing an operation that is stated to also invalidate the BTB.

''However, on ARMv6, software must invalidate the BTB after changing the ContextID or FCSE ProcessID''.

===The IPB===

=Translation Regimes=
In ARM lingo, the current translation regime of the processor refers to whether or not the processor is doing single-stage or 2-stage MMU translation. In other words, it refers to whether or not the processor is using two-dimensional address space lookup from Guest virtual addresses to Host physical addresses. In x86 parlance this is called "Extended Page Tables".

2-level translation schemes are used by Hypervisors to support multiple Guest Operating Systems.

[[Category:ARM]]
````

<a id="w004018"></a>

## W004018 — User:Gravaera/ARM Common Memory System Architecture

Bron-ID: W004018
Namespace: User (2)
Revisie: 20141 | Datum: 2017-01-11T11:36:15Z
Bron: https://wiki.osdev.org/User:Gravaera/ARM_Common_Memory_System_Architecture
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FARM_Common_Memory_System_Architecture&oldid=20141
Doorverwijzing: User:Gravaera/ARMv7 Common Memory System Architecture,

````text
#REDIRECT [[User:Gravaera/ARMv7 Common Memory System Architecture,]]
````

<a id="w004019"></a>

## W004019 — 3D Renderer Basics

Bron-ID: W004019
Namespace: Main (0)
Revisie: 23566 | Datum: 2019-04-08T09:15:57Z
Bron: https://wiki.osdev.org/3D_Renderer_Basics
Revisie-URL: https://wiki.osdev.org/index.php?title=3D_Renderer_Basics&oldid=23566

````text
3D graphics is something most operating systems want to support one day. Unfortunately, video drivers that support GPUs are extremely complex to implement, and it's hard to find information that covers the basics of software rendering.

The goal here is to describe the simplest 3D rendering pipeline possible, and then show ideas to improve on it, to give the reader (you) a basic understanding. Please note we do not intend this to be "state-of-the art" and will cover nothing advanced and is also not intended to be maths tutorial and won't include formulas (these can be found with a simple web search).


==Data Structures==

Before a 3D renderer can do anything it needs data describing a scene to render. A scene shall comprise a camera, a world, and objects within that world.


===Camera===

To describe a camera you need the camera's location (x, y, z), which direction it is facing (a "front vector") and which direction is the top of the camera ("up vector"). You can think of a vector as a displacement in 3D space (amount to add to x, amount to add to y, amount to add to z). For the mathematics you'll want later it's best to use unit vectors for the camera's "front vector" and "up vector". A unit vector is a vector where the distance is 1. To convert a vector into a unit vector divide each component by the distance (e.g. "distance = sqrt(x*x + y*y + z*z); x /= distance; y /= distance; z /= distance;").

A camera might also have other data associated with it (e.g. field of view).


===World===

The world is a special kind of object that may be literally nothing (a coordinate system only), or may describe things that can not move (e.g. terrain, sky). The world needs no direction vectors (it is the primary coordinate system that everything else is relative to), but otherwise can be the same as an object.

==Objects==

An object comprises another coordinate system (the object's "local coordinate system"); several values to describe where the object is in the world's coordinate system, one or meshes, plus the data that each mesh needs. For simplicity (initially) you can assume that an object only ever has one mesh.

The values to describe where the object is in the world's coordinate system are a coordinate describing where the object's local coordinate system's origin would end up in the world's coordinate system; plus a "front vector" and "up vector"  to describe the object's orientation within the world's coordinate system.


===Meshes===

A mesh comprises a list of coordinates, then a list of polygons.


====Coordinates====

Coordinates are just a location within a coordinate system, typically described as 3 values "(x, y, z)" with no additional information. For performance it is recommended that coordinates are stored in "structure of arrays" format (and not "array of structures" format). What this means is that you do something like "float x[COORDS]; float y[COORDS]; float z[COORDS];" and not "struct coord myCoords[COORDS];". The reason for this is that it makes it much easier to use SIMD (SSE, AVX) to process many coords.

''Note: Sometime people get "coordinate" and "vertex" confused or use them interchangeably. In my opinion this is wrong. For this wiki page, a vertex is a coordinate with additional information, and is not the same as a coordinate that has no addition information.''


====Polygons====

For simplicity, we will assume there are only 2 kinds of polygons - textured polygons and gradient polygons.

Every polygon has a set of flags, where one/some of the flags are used to determine what type of polygon it is (and the other flags are used for other things described later), and every polygon has a "number of vertices". Everything else depends on the type of polygon.


=====Textured Polygons=====

For textured polygons the polygon also has some kind of "texture reference". Each vertex used by a textured polygon has a reference to the coordinate (in the meshes' list of coordinates) plus a location in the texture (typically a "(u, v)" coordinate within the texture the polygon uses).

Note that for repeating patterns the texture coordinate may be larger than the texture's dimensions. When drawing polygons on the screen later on, the renderer will do something like "u = u % texture_width; v = v % texture_height" for each pixel, so that if the"(u, v)" coordinate within the polygon's vertex is larger than the size of the texture then the texture's ends up being repeated across the polygon. This is very beneficial for things like (e.g.) brick walls, where the same "brick texture" might be repeated over a large area, because that large area can be described with one polygon rather than many polygons.


=====Gradient Polygons=====

For gradient polygons; each vertex has a reference to the coordinate (in the meshes' list of coordinates) plus a colour. When drawing polygons on the screen later on, the renderer will use interpolation to determine a pixel's colour. For example, if the polygon is a triangle and has a top vertex that is blue, a left vertex that is red and a right vertex that is green; then we'd find the points on the left edge and right edge where a horizontal line through the pixel would intersect those edges and use the vertices' colours and the distances between those vertices along the edge to determine a "left colour" and a "right colour"; then use distances that the pixel is between the left and right (and the "left colour" and a "right colour") to determine the pixel's colour.


==Coordinate Systems and Transformations==

There are multiple different coordinate systems (each object's local coordinate system, the world's coordinate system, the camera's coordinate system, then the screen's coordinate system). One of the first things the 3D renderer will need to do is transform all coordinates (from objects and the world) into screen coordinates. This involves is multiple steps (because there multiple coordinate systems).

Most steps involve building a rotation matrix (from the "front vector" and "up vector" of one coordinate system) and a translation matrix (from the "location of origin in the other coordinate system"); and then multiplying each coordinate (in each object's meshes' coordinate list) by each matrix in order. However, this can be optimised by multiplying the matrices together, so you only end up multiplying a coordinate by a single "all steps combined" transformation matrix.

The first step is building a transformation matrix to convert coordinates in the camera's coordinate system into screen coordinates. This is special and unlike the other steps. For this you need a projection matrix (which essentially does "x = x/z; y = y/z"), a scaling matrix (to scale coordinates to suit the screen's resolution, possibly derived from the camera's field of view), and a translation matrix (which subtracts half the screen's horizontal resolution from x values and subtracts half the screen's vertical resolution from y values). These 3 matrices should be combined (multiplied) to form the final "camera to screen transformation matrix".

''Note: This is a simplification used to keep things in a "nicer to understand" order. See the warning in "Viewing Volume Warning" section below about problems involving "z = 0" and division by zero.''

The second step is building a transformation matrix to convert coordinates in the world's coordinate system into the camera's coordinate system. This involves creating a rotation matrix from the camera's "front vector" and "up vector", creating a translation matrix from the camera's location in world space, then combining these 2 matrices with the "camera to screen transformation matrix" from the previous step to get a final "world to screen transformation matrix".

Once you have the "world to screen transformation matrix" it can be applied to world coordinates to convert them to screen coordinates, and this is what makes the world special (because you need one less step to create the transformation matrix for the world's mesh).

The third step is, for each object in the world, use the object's "front vector" and "up vector" to create a rotation matrix, and the object's position to create a translation matrix; then combine the object's 2 matrices with the "world to screen transformation matrix" to get a "object to screen transformation matrix" for the object. Once you have the "object to screen transformation matrix" for an object it can be applied to the object's coordinates to convert them to screen coordinates.

When you've finished all of this (applied the transformation matrices to the world and all objects in the world), all coordinates will be screen coordinates.


==Back Face Culling==

To improve performance we want to discard ("cull") polygons that can't be seen as early as possible (after doing the least unnecessary work for them). For opaque objects that have thickness, some polygons will be on the object's front side and some will be on the object's back side, and the polygons on the object's back side can't be seen (and can be culled). It turns out it's easy to test for this.

If the vertices of a polygon are in a specific order (e.g. clockwise when looking at the polygon), after coordinates are transformed into screen coordinates you can test if the polygon's screen coordinates are in the same specific order (e.g. clockwise) by using any 3 coordinates in sequence (e.g. just the first 3 coordinates) to find 2 vectors ("first to middle" and "middle to last"), and then calculate the vector dot product of those 2 vectors. The sign of the vector dot product will determine if the polygon is a front face or a back face. If it is a back face, then the polygon can be discarded.

Note: You may want to use one of the polygon's flags for "if this flag is set, discard polygon if it's facing backwards". This allows you to disable back face culling for some polygons (e.g. if you want a "zero thickness" fence that looks the same from both sides, then you can use one polygon for both sides instead of having 2 polygons). Also; if you know that the polygon will be a front face for some other reason (e.g. because it's part of the sky and you don't allow the camera to get higher than the sky) you can clear the flag to bypass the overhead of the ("known unnecessary") back face culling test.


==Viewing Volume Culling==

If all of a polygon's z coordinates are negative, then the entire polygon must be behind the camera and impossible to see, and that polygon can be culled. This is (should be) very easy to understand.

Now imagine a "viewing volume" described by a left plane, a top plane, a right plane, a bottom plane, a near plane and a far plane; where everything inside that volume is visible and everything outside that volume is not visible. With screen coordinates, the "near plane" is described by "z = 0" so if all of a polygon's z coordinates are negative the polygon must be outside the viewing volume and the polygon can be culled.

The other planes are similar. The far plane can be described by "z = some far away constant" and if all of a polygon's z coordinates are larger than "some far away constant" then the polygon must be outside the viewing volume and the polygon can be culled.

The left plane can be described by "x = 0" (and if all of a polygon's z coordinates are negative then the polygon must be outside the viewing volume and the polygon can be culled). The right plane can be described by "x = horizontal_resolution". The top plane can be described by "y = 0". The bottom plane can be described by "y = vertical_resolution".


==Viewing Volume Clipping==

If some of a polygon's z coordinates are negative but some aren't; then you can find the points on the polygons edges that intersect with "z = 0" and clip the polygon at those new points. The same applies to all other planes that describe the viewing volume.

To do this, start by finding the intersection points and then calculating the other data that a vertex needs - for a gradient polygon you need to calculate the colour for the vertex at the intersection point, and for a textured polygon you need to calculate the texture coordinate for the new vertex at the intersection point. The new vertices can be inserted into the polygon's list of vertices. Then it starts to get slightly tricky.

There are 2 types of polygon shapes - convex and concave. A convex polygon is a polygon where any line that passes through the polygon can only intersect with 2 points on the polygon's circumference. A concave polygon is a polygon where a line that passes through the polygon might intersect with more than 2 points on the polygon's circumference. Convex polygons complicate things throughout the renderer, and for this reason it might be a good idea to say "all polygons must be convex" (and not bother supporting conc(left, right, top, bottom)ave polygons).

If a polygon is convex, then you will have found (at most) 2 intersection points and created 2 new vertices at those intersection points; and all of the original polygons vertices that fail your clipping test (e.g. have a negative z if you're clipping to "z = 0") can be discarded.

If a polygon is concave, then you may have found 2 or more intersection points and created 2 or more vertices at those intersection points. If you found 2 intersection points then it becomes the same as convex polygons (simply discard any vertices that fail your clipping test). Otherwise you have a choice: you can leave the polygon alone and deal with "incomplete clipping" later (during rasterisation); or you can deal with it during viewing volume clipping. If you do deal with this case during viewing volume clipping then it gets complicated, partly because you can end up with multiple polygons.

''Note 1: It is entirely possible to not do viewing volume clipping at all, and always leave it until later (during rasterisation). However, it's better for performance to clip a smaller number of whole polygons and avoid the need to do clipping on a much larger number of line fragments.''

''Note 2: Depending on what you felt like doing (e.g. if you support convex polygons, etc); it may be a good idea to use a polygon flag to indicate if the polygon was fully clipped or partially clipped, so that (during rasterisation) you don't do a bunch of unnecessary clipping tests for polygons that were fully clipped.''


==Viewing Volume Warning==

Above I've described the near plane (used for viewing volume culling and viewing volume clipping) as "z = 0". In practice this is a bad idea because you end up having to deal with the possibility of division by zero. One of the places where you will have to worry about division by zero (caused by "z = 0") is in the projection matrix mentioned above (in the "Coordinate Systems and Transformations" section), where that projection matrix does something like "x = x/z; y = y/z".

For this reason it's much better to use "z = some small positive value" (instead of "z = 0") for the near plane (used for viewing volume culling and viewing volume clipping); and then apply the projection matrix after viewing volume culling and viewing volume clipping. However this complicates things again, because it makes volume culling and viewing volume clipping complicated for the other (left, right, top, bottom) planes.

The practical solution is to do viewing volume culling and viewing volume clipping for the near plane (and maybe the far plane too); then apply the projection matrix (after you know that z must be a non-zero positive number for all remaining coordinates); then do viewing volume culling and viewing volume clipping for the remaining (left, right, top, bottom, and maybe far) planes after that. 


==Rasterisation==

The idea of rasterisation is to convert each polygon into horizontal line fragments. In the most simplest way, this can be done by finding the coordinates with the highest and lowest y value, then doing a "for each y from lowest to highest" loop where you find the left and right intersections for each horizontal line fragment.

For convex polygons the simplest way isn't very good - it can be improved a lot by keeping track of "left edge" and "right edge" and calculating the intersection from that, switching to the next edge when you y goes beyond an edge's vertex.

For concave polygons it gets complicated again - for each y you can multiple intersections (where the number of intersections is always even). For example, for a shape like the letter "W", near the bottom of the shape you might get 4 pairs of intersections, resulting in 4 horizontal line fragments.

Because we're using 2 different types of polygons (textured polygons and gradient polygons) we will end up with 2 different types of line fragments (textured fragments and gradient fragments).

For textured polygons you want to know the "screen y" for the fragment, which texture it is, the left intersection's x and its texture coordinates (u1, v1), and the right intersection's x and its texture coordinates (u2, v2).

For gradient polygons you want to know the "screen y" for the fragment, the left intersection's x and its colour, and the right intersection's x and its colour.

''Note: To avoid distortion caused by "non-linear depth" you might also want to know the left and right intersection's z for each fragment (if you take that into account later when determining what colour each pixel would be). Because correcting this distortion can be complicated I'd recommend not bothering initially (and worrying about it later, after you've got the renderer working).''


==Fragment Rendering==

Once you have fragments, you can draw them on the screen (sort of). The basic idea is to have a loop that goes from the fragment's left x to the fragment right x; where you find the pixel at the current x (and the fragment's y), then determine the colour for that pixel from the rest of the fragment's data.

For textured fragments you want to use the current x to find the texture coordinate between "left intersection (u1, v1)" and "right intersection (u2, v2)", and then do texture coordinate reduction mentioned earlier for repetition ("u = u % texture_width; v = v % texture_height"), then find the colour of the textel at the resulting texture coordinate and use that for the pixel. 

For gradient fragments you want to use the current x, "left colour" and "right colour" to determine the pixel's colour.


==Occlusion==

If you simply draw fragments, then you're going to get an ugly mess - polygons that are behind other polygons may be drawn as if they were in front instead. Occlusion is all about preventing that problem. There are several different ways to do occlusion, each with their own advantages/disadvantages.


===Painter's Algorithm===

One way to do occlusion sounds simple - just draw everything in "back to front" order. In practice, determining the correct order to draw polygons can be expensive when there's many polygons, and can be very complicated (up to "impossible without splitting polygons up" - e.g. if polygon A is in front of polygon B, polygon B is in front of polygon C, and polygon C is in front of polygon A).


===Z Buffer===

Another way to do occlusion is to use a buffer to store the "current Z for each pixel" that is initially (at the start of rendering a frame) set to "all z coordinates are far away". When you're drawing a fragment you determine the z coordinate for the pixel (for that fragment) and compare it with the z value in the buffer. If the z value in the buffer is smaller then this pixel is behind something and not drawn, and if the z value in the buffer is larger then this pixel is in front of whatever was previously drawn and you set the pixel's colour and also set the z value in the buffer.

There are a few problem with this. The first problem is when you the z value for a fragment's pixel is equal to the z value in the buffer (e.g. due to precision loss/rounding), you can't know if the pixel should be kept or not. This leads to a problem called "z fighting". The second problem is performance - for every pixel of every fragment, you determine if the pixel should be "kept for now" and if the pixel is kept you do work to determine the pixel's colour; but there is no guarantee the pixel won't be overwritten later and the work you did to determine the pixel's colour will be a waste of time.


===Fragment Buffer===

Imagine if, for each horizontal screen line, you have a list of fragments for that y. Initially these lists will all be empty; and during rasterisation you insert fragments into the list for that y.

When you insert a fragment in the list for that y, you can determine if the fragment is in front of or behind any existing fragments for that y. If the new fragment is (completely or partially) behind another fragment then the new fragment can be (completely or partially) discarded, and if other fragments are (completely or partially) behind the new fragment you can (completely or partially) discard the other fragments.

After you've done this for all fragments; you end up with a list of fragments that are not occluded, and then you'd render those fragments (and never determine the colour of a pixel that can't be seen).

This can be a complicated for memory management (e.g. using "malloc()" and "free()" for every fragment would be unusable slow), and takes some cleverness to do the fragment insertion quickly (possibly starting with storing fragments "starting x order" and using binary search).


===Polygon Occlusion===

It is possible to do occlusion before rasterisation, and this can (in theory) be much better for performance because it avoids doing all unnecessary work (in rasteristation and fragment rendering) for everything that's occluded.

This is extremely complex. For each pair of polygons, determine which polygon is closer and which is further away, then create a "masking polygon" by mapping the closer polygon onto the plane of the further polygon, then subtract the masking polygon from the further polygon. Note that polygon subtraction can lead to concave polygons (which most people would've decided to avoid) and "donuts polygons" (where a polygon has an outer circumference and an inner circumference and needs 2 vertex lists to describe).


==Object Meshes Revisited==

Earlier I mentioned that each object might have one or more meshes; and you might be wondering why an object might want multiple meshes. There are a few reasons for this.


===Meshes For Different Distances===

One reason for multiple meshes is that maybe you want to have one detailed mesh (with lots of polygons) for when the object is close to the camera, and then another simpler mesh (with fewer polygons) for when the object is far away from the camera. This improves performance when objects are too far away for the details to matter. You can extend this idea to many meshes at different levels of detail.


===Spherical Mesh Culling===

Imagine an object that has an exterior and an interior (like a space ship, or car, or submarine, or house, or...). When the camera is outside the object there will be polygons in the interior that are impossible to see (and if there's no windows all interior polygons will be impossible to see). When the camera is inside the object there will be polygons on the exterior that are impossible to see (and if there's no windows all exterior polygons will be impossible to see).

Now assume you have 2 spheres centered at the object's origin, where each sphere is stored as its radius. The radius of the first sphere is determined by distance from the object's origin to the furthermost interior polygon coordinate; and if the camera is further from the object's origin than this sphere's radius then the camera must be outside the object.

The radius of the second sphere is determined by the distance from the object's origin to the closest exterior polygon coordinate. If the camera is closer to the object's origin than this sphere's radius then the camera must be inside the object.

Now imagine you have 3 meshes. The first mesh is for polygons that can only be visible when the camera is outside the object; the second mesh is for polygons that can be visible regardless of where the camera is (e.g. exterior polygons that can be seen from the interior through a window); and the third mesh is for polygons that can only be visible when the camera is inside the object.

This gives us 3 cases (camera definitely inside object, camera definitely outside object, and camera may be inside or outside) where you can determine which of these cases it is extremely quickly (by using "distance from camera to object's origin" and comparing it to the radius of each sphere); where for 2 of those cases you can cull entire groups of polygons very early in the renderer (before you do anything else).


==Partial Frame Rendering==

There are a few cases where you might be able to avoid redrawing everything for a frame from scratch.

Note that it may be possible to support all of these, and may be beneficial to have a preliminary "assessment" step that determines how much needs to be rendered when multiple things have changed since last frame.


===Texture Change===

If a texture changed (but camera and objects remain in the same position) you can (at a minimum) start rendering the new frame at the "rasterisation" step. For this to work you need to keep the data used for rasterisation (the "screen coords" and lists of polygons that weren't culled) until after you know if it will/won't be needed for the next frame. This can potentially be improved by determining which area/s of the screen may have been effected by the texture that was changed, and only rendering that area of the screen.

If you used "polygon occlusion"; then you could rasterise and redraw the polygons that used the modified texture and nothing else.

If you used "fragment buffer"; then you can skip redraw the fragments that used the modified texture and nothing else.


===Object Movement===

If an object moves (but camera and textures remain the same); then you can determine the area of the screen that was effected (e.g. use the object's bounding sphere, it's previous location and its new location to determine a bounding rectangle on the screen) and then only render that area of the screen.==


===(Small) Camera Rotation===

For 2 of the 3 possible rotations (excluding "roll"), if the camera rotates (but objects and textures remain the same) then you can scroll the previous frame's pixels in the corresponding direction. This creates "not rendered before" area/s on the left or right of the screen and/or on the top or bottom of the screen; where you only need to render the "not rendered before" area/s.


==Rendering Distance Cube==

You can have a set of 6 textures (north, east, south, west, up, down) that form a cube, and use these textures to represent everything that is past the rendering distance. Because these textures represent "past the rendering distance" they can either be drawn first (so that all polygons occlude them), or they can be drawn last (after rendering everything else, if there's no data for a pixel then get the pixel's colour from the rendering distance cube). The benefit of this is that it can be a relatively cheap way to give the illusion of huge open areas (without the expense of using meshes/polygons to draw things like distant mountains, etc).

These textures can be replaced. For example, if the camera moves a certain distance then maybe you change to a different set of 6 textures.

These textures can be changed. For example, maybe the "top" texture is dynamically generated clouds that move slowly.

These textures can be morphed. For example, maybe you have "midday textures", "dusk textures", "midnight textures" and "dawn textures"; and depending on the time of day you blend 2 textures together (e.g. if the time is 20% of the way between midday and dusk, then you take 80% of a pixel from "midday" and the other 20% from "dusk").

These textures can be dynamically generated. For example, maybe (for the "north" texture) you have mountain in the distance, but there's a large hot air balloon flying over that mountain, so you regenerate the "north texture" every 250 ms by taking the original and drawing a "hot air balloon sprite" into it at a different positions.


==Static Distant Impostors==

For objects in the distance (especially things like trees where it doesn't matter so much which angle you look at them from); you can draw a single static textured polygon (an "impostor") instead of drawing the object itself (many polygons) to improve performance (reduce the number of polygons you have to deal with). When the camera gets close enough, then you switch to the object and stop using the impostor.


==Dynamic Distant Impostors==

For objects in the distance, you can render the object all by itself to dynamically generate a suitable distant impostor (and then use that dynamically generated distant impostor in the same way you would've used a static distant impostor). This takes a little care to determine when to update the distant impostor from the object.


[[Category:Video]]
````

<a id="w004020"></a>

## W004020 — Real Mode OS Warning

Bron-ID: W004020
Namespace: Main (0)
Revisie: 25148 | Datum: 2020-09-10T07:29:58Z
Bron: https://wiki.osdev.org/Real_Mode_OS_Warning
Revisie-URL: https://wiki.osdev.org/index.php?title=Real_Mode_OS_Warning&oldid=25148

````text

Using BIOS functions (and "avoiding" the need to write drivers) can seem easier for beginners that don't know any better, however it makes everything much much harder in the long run. This makes it an alluring trap for the unwary.

The purpose of this page is to forewarn beginners so that they don't become unsuspecting victims of the trap. This page is not intended to prevent people who are familiar with all the pitfalls from intentionally deciding to walk into the trap.

== BIOS Is Deprecated ==

Currently the entire industry (all "80x86 PC" manufacturers all OSs for "80x86 PC") are shifting to UEFI. At the end of this transition (at some point in the future) BIOS will cease to exist. Relying on the existence of BIOS functions is not a long term option. Temporarily relying on the existence of BIOS functions (for the short term) makes it much much harder to port the OS to UEFI later, and also makes it much much harder to port the OS to other platforms (ARM, SPARC, etc) because everything that the BIOS was used for has to be replaced.

''Note: Intel has officially stated that they will cease supporting BIOS (go from "BIOS+UEFI hybrid" to "UEFI only") before the year 2020; and some people expect that Intel's main reason for doing this is so that they can start removing obsolete hardware from their chipsets (starting with A20 gate, and possibly including PIC, PIT, PS/2, legacy ROM areas and VGA emulation).''

== BIOS Does Not Support All Devices ==

BIOS doesn't provide support for sound, network cards (with the exception of one network card if and only if you booted with PXE), modern printers, scanners, (voice, data or fax) modems, GPUs (for hardware accelerated graphics and/or GPGPU), multiple monitors (one monitor only), various sensors (movement, light, location/GPS, etc) or some user input devices (mouse, joystick, steering wheel, head tracking in 3D helmets, motion sensing/Kinect, touch screens, tablets used with stylus, microphones, bar-code scanners, etc).

For any/all of these things, you will need to write your own drivers.

== BIOS Does Not Support Any Device Well ==

Firmware (including BIOS and UEFI) are mostly only intended to provide minimal functionality needed to get an OS started; and for this reason (for devices that they do support) they don't support the devices well. Note that most of the problems apply to both BIOS and UEFI, and most of the problems apply even if you're writing a "single tasking real mode" OS.

=== Time And Timers ===

The "time of day" support only has 1 second precision (and not sub-second precision); and the alarm function is limited to the next 24 hours and also only has 1 second precision. The "elapsed time" support is based on the worst/slowest configuration of the PIT, giving it poor (~55 ms) precision without any support for "call my callback after duration". The BIOS function to obtain elapsed time is based on "ticks since midnight" and has various issues involving roll-over (e.g. trying to determine the amount of time between "just before midnight" and "just after midnight" is a little messy and will bite you if you don't take it into account; and you can't measure anything longer than 1 day without intermittent polling because you don't know how many midnights have passed between "start time" and "end time").

=== Serial Ports ===

The BIOS serial port code uses polling to send and receive bytes (and there is no "FIFO buffers"). This means that it's almost impossible to use the BIOS functions for serial ports to receive data without losing data, and almost impossible to use the BIOS functions for serial ports to send data without huge performance problems.

=== Video ===

The BIOS functions for video (including VBE/VESA BIOS Extensions) don't allow anything other than switching video modes and writing to display memory (no fast bit-blits, no hardware mouse pointer, no vertical sync, etc). For video mode switching the number of modes is typically very limited, and often you can't even set the monitor's preferred/native resolution (leading to a blurry/bad image due to scaling done by the monitor). For writing to display memory the performance is intolerable (hundreds of cycles of extra bloat for every single pixel written).

=== Keyboard ===

The BIOS keyboard support has a tiny buffer and uses polling, so if you don't poll often enough (and the user types fast enough) you lose key-presses. There is also no support for asking if a specific key is currently being held down or not, no support for the keyboard's LEDs, no support for different keyboard layouts, no way to do a reverse lookup (determine the name of the key from the scan code), no support for "non-ASCII" (needed for internationalisation), and no support for "hot-plug" (device removal/insertion, and old KVMs that use a mechanical switch).

=== Disk/Storage Devices ===

BIOS support for these devices is synchronous; which causes severe performance problems. For example, if you want to decompress a file from one device and write the data to another device, you can't have a pipeline of buffers and read from one device while the CPU decompresses while writing decompressed data to the other device - you have to read from the first device (while the second device and CPU do nothing) then decompress (while both devices do nothing) then write to the other device (while the first device and CPU do nothing). You also can't pre-fetch data in the background or implement an effective "write-back" disk cache (where writes are done the background).

For removable media (floppy, CD, tape) there is no notification when media is inserted or removed. This can lead to data corruption - e.g. user removes one floppy disk and inserts another; but OS still thinks the first floppy disk is present and writes to it, accidentally corrupting the second floppy disk. While it is possible to try to avoid this (e.g. always check if first sector is different before doing any read or write) it causes more performance problems.

For hot-plug devices (all SATA disks, all USB devices, etc) there is no support for hot-plug. E.g. if the user unplugs a USB flash device (or SATA hard drive or...) there's no notification (read/writes to the device simply cause an error) and if the user plugs in a USB flash device (or SATA hard drive or...) there's no way for the OS to use it (the BIOS continues to pretend the device doesn't exist).

There is no concept of IO priorities and no way to cancel an "in progress" transfer. This means that if you need to transfer data as soon as possible then that transfer has to wait until any less important "in progress" transfer completes.

There is no support for "trim" (for SSD and USB flash), no support for "secure erase", no support for "eject" (for CD drives), no support for monitoring the health of the device ("Self-Monitoring, Analysis and Reporting Technology"), no support for hard drives with 4 KiB sectors (determining if the drive has 4 KiB sectors or not), and no support for "write barriers" or determining if the data has actually been written to the media and not just buffered by the drive (needed to ensure power failures can't cause file system corruption).

=== CPUs ===

BIOS is only designed to handle single-CPU. You can use multiple CPUs, but you can't safely allow 2 or more CPUs to call BIOS functions at the same time and need a "BIOS lock" to ensure that doesn't happen. There's also a risk that calling BIOS from any CPU other than the BSP ("boot CPU") may cause problems (either due to BIOS bugs or for other reasons).

== BIOS Prevents You From Doing Anything Right ==

The BIOS expects a variety of hardware devices (PIC, PIT, RTC, PS/2 controller, disk controllers, etc) to be in a certain default state. Some of this hardware is deprecated and replaced (and emulated) by more modern devices (PIC replaced with IO APIC, PIT replaced with HPET, etc). An OS can't use modern hardware without tampering with the default state of devices that the BIOS expects, and therefore can't use modern hardware without breaking BIOS.

This makes it extremely difficult for an OS to do anything right (including drivers for devices that the BIOS doesn't support) while still using the BIOS.

== BIOS Prevents You From Learning Anything Useful ==

Eventually (to write an OS that's actually useful, or to learn how to write an OS that's actually useful) an OS developer needs to learn various things; like how devices work, how interrupt controllers can be used, how IO can be done asynchronously, etc. Knowing about these things (even if your OS doesn't do them) is what makes you a good/better OS developer. You learn none of these things by relying on the BIOS.

== BIOS Leads You To Extremely Poor Design ==

For beginners that use BIOS, it is natural (due to lack of experience/knowledge) for them to design their operating system's device driver interfaces to be similar to the interfaces provided by the BIOS, and then it's natural to design the interfaces/APIs for upper layers (e.g. file systems, etc) to suit the driver interfaces, and then it's natural to design the user-space interfaces/APIs to suit both the driver interfaces and the upper layers.

This means that there is an extreme risk of the (very bad and broken) design of the BIOS to permeate the entire OS; such that even if the OS developer manages to overcome all of the other problems caused by using BIOS (e.g. and replace all use of BIOS with their own native drivers, etc) they are still left with a crippled OS that is permanently scarred.

[[Category:Real Mode]]
````

<a id="w004021"></a>

## W004021 — Template:Tone

Bron-ID: W004021
Namespace: Template (10)
Revisie: 22744 | Datum: 2018-09-10T03:18:26Z
Bron: https://wiki.osdev.org/Template:Tone
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ATone&oldid=22744

````text
{{NoteBox|This article's '''tone or style''' may not reflect the encyclopedic tone used throughout the wiki. See [https://en.wikipedia.org/wiki/Wikipedia:Writing_better_articles#Tone Wikipedia's article on tone] for suggestions.}}

<includeonly>[[Category:Tone]]</includeonly>
````

<a id="w004022"></a>

## W004022 — Template talk:Tone

Bron-ID: W004022
Namespace: Template talk (11)
Revisie: 20293 | Datum: 2017-01-26T21:02:48Z
Bron: https://wiki.osdev.org/Template_talk:Tone
Revisie-URL: https://wiki.osdev.org/index.php?title=Template_talk%3ATone&oldid=20293

````text
I have created this template because some articles feel very odd to read and should be cleaned up. --[[User:Matt11235|Matt11235]] 15:02, 26 January 2017 (CST)
````

<a id="w004023"></a>

## W004023 — Intel HD Graphics

Bron-ID: W004023
Namespace: Main (0)
Revisie: 30452 | Datum: 2026-04-05T17:39:15Z
Bron: https://wiki.osdev.org/Intel_HD_Graphics
Revisie-URL: https://wiki.osdev.org/index.php?title=Intel_HD_Graphics&oldid=30452

````text
{{In_Progress}}

== Introduction ==

This page covers implementing driver support for Intel's integrated graphics technology. At the moment this only covers first generation 'Intel HD graphics’, codename Ironlake. Please note that graphics drivers are complex and while Intel's is on the simpler side, it is still a difficult topic not intended for newcomers, some experience writing drivers for other modern hardware is suggested before attempting this. Additionally, the reader is responsible for any damage due to inaccuracies in this documentation.


== Experimenting with the device ==

There are various tools available to study the behavior of the graphics device for simpler tasks in order to gain an understanding of how something is done. One way to do so is to use Intel's graphics debugging utilities (The intel-gpu-tools package), which allow reading and writing to the device registers from the terminal. This can be very useful, for instance, this method proved invaluable when studying the GMBUS. The same can be done using GRUB2’s built in terminal by obtaining the MMIO base address using <code>lspci</code> and then using the read/write commands (read_byte/read_word/read_dword and write_byte/write_word/write_dword) to perform the associated operation on the desired register.

== Register Locations and Definitions ==

Many registers may move between generations or may be replaced completely with a different set of registers, an appropriate note will be made where possible, but it is recommended that this document be read alongside the programming reference manuals for the intended generation. Additionally, the register location definitions for the driver for Cardinal can be found [https://github.com/himanshugoel2797/Cardinal/blob/master/drivers/display/ihd/common/include/ihd_regs.h Cardinal GitHub].


== Recommended reading ==

Some additional reading is highly recommended before tackling graphics/display drivers which will help clarify a lot of the designs and terminology used in the official PRMs. Understanding the following is recommended:
* EDID
* I2C
* DisplayPort
* Intel Integrated graphics Programming Reference Manuals (PRMs)
* OpenGL 4.0+/Vulkan/DirectX 9+: This is only really necessary if you intend to implement 3d acceleration. Understanding how the graphics pipeline functions and is structured at a higher level will help with understanding the hardware level structure exposed the graphics chipset.


== Graphics Northbridge and Southbridge ==

The display controller pipeline structure is split among two components depending on their speed requirements and overall use. The Northbridge is connected directly to the processor, and is used for the built-in display as well as the display pipes and various planes. The Southbridge contains the external ports and the GMBUS (Graphics Management Bus). The GMBUS is an I2C compatible protocol used to communicate with devices to obtain information from them (such as the display's EDID). The two bridges communicate through a transmitter and receiver mechanism; this is done by assigning FDIs to displays on the Southbridge and configuring the display pipes on the Northbridge to receive from the assigned FDI. This system is discussed in detail in the [[#Display Pipeline Structure]] section.

== PCI interface and graphics memory ==

The PCI BARs 0 (offset 0x10) and 2 (offset 0x18) contains the 64bit addresses for the MMIO region and snooped graphics memory respectively.


== Built-in display backlight ==

The display backlight is controlled by a PWM (pulse width modulation) unit. For the built in LVDS display, the PWM initialization registers are present on the south bridge, while the actual control value is specified on a register on the North bridge. 


== Display detection ==

Display detection can be done by either polling or using interrupts. The interrupt method will be discussed here. 


== GMBUS registers and the EDID ==

The '''GMBUS''' (Graphics Management Bus) is an [[I2C]] compatible protocol used to communicate with the attached displays to obtain their information such as the [[EDID]]. This method cannot be used to read the EDID of a [[DisplayPort]] device, instead the AUX CH should be used.

To obtain the [[EDID]] for a device, we want to read 0x80 bytes from the [[I2C]] device index 0 of the desired device from an offset of 0x50. 

The following execution sequence demonstrates how the first 8 bytes can be retrieved:

<syntaxhighlight lang="c">
void igfx_gmbus_reset(igfx_dev_state_t *driver)
{
   igfx_write32(driver, driver->display_mmio_base + IGFX_GMBUS1, 0);
   igfx_gmbus_enable_writeprot(driver);
   igfx_gmbus_disable_writeprot(driver);
}

void igfx_gmbus_wait(igfx_dev_state_t *driver)
{
   while (!(igfx_read32(driver, driver->display_mmio_base + IGFX_GMBUS2) & (1 << 11)))
       ;
}

void igfx_gmbus_stoptransaction(igfx_dev_state_t *driver)
{
   igfx_write32(driver, driver->display_mmio_base + IGFX_GMBUS1, (1 << 30) | (1 << 27));
}

void igfx_gmbus_read(igfx_dev_state_t *driver, uint32_t disp_idx, uint8_t offset,
                    uint16_t sz, uint8_t *buf)
{
   uint32_t gmbus0_val = 0;
   uint32_t gmbus1_val = 0;

   if (driver->device->arch == IGFX_CHERRYTRAIL)
   {
       if (disp_idx == 0)
           gmbus0_val = 3;
   }

   gmbus1_val = (1 << 30) /*SW Ready*/ | (1 << 25) /*WAIT*/ |
                (1 << 26) /*INDEX*/ | (sz & 511) << 16 | ((offset & 127) << 1) |
                1 /*READ*/;
   igfx_gmbus_reset(driver);

   igfx_write32(driver, driver->display_mmio_base + IGFX_GMBUS0, gmbus0_val);
   igfx_write32(driver, driver->display_mmio_base + IGFX_GMBUS1, gmbus1_val);

   for (int i = 0; i < sz; i += 4)
   {
       igfx_gmbus_wait(driver);

       uint32_t bytes = igfx_read32(driver, driver->display_mmio_base + IGFX_GMBUS3);

       buf[i] = bytes & 0xFF;
       if (sz > i + 1)
           buf[i + 1] = (bytes >> 8) & 0xFF;
       if (sz > i + 2)
           buf[i + 2] = (bytes >> 16) & 0xFF;
       if (sz > i + 3)
           buf[i + 3] = (bytes >> 24) & 0xFF;
   }

   igfx_gmbus_stoptransaction(driver);

   print_hexdump(buf, sz);
}
</syntaxhighlight>

=== Getting the EDID from a [[DisplayPort]] device ===

To communicate with a [[DisplayPort]] device you'll need to use the following registers:

{| class="wikitable"
|-
! Register
! Description
|-
| DDI_AUX_CTL
| rowspan=2 | A register to initiate AUX CH requests
|-
| DP_AUX_CTL
|-
| DDI_AUX_DATA
| rowspan=2 | A set of 5 registers to send up to 20 bytes of data
|-
| DP_AUX_DATA
|-
| SRD_AUX_CTL
| A register that must be disabled if using the DDI_AUX registers
|-
|}

Refer to the Intel programming manuals for the register's addresses and potential errata.

To send a request, first copy the AUX CH packet to the data registers starting with the high byte of the first register. Then initiate a request by setting the message length and the send_busy bit. Wait for either the done bit or any of the error bits to be set, then read the message length and the appropriate amount of data from the data registers.

Note that all the other bits in the CTL registers should be preserved, so load the registers before toggling any bits.

=== Display pipeline structure ===


=== Display Pipes ===


=== Display FDI ===


=== Display planes ===

Three types of display planes are available for each pipe, namely:
*Display plane
*Cursor plane
*Video/sprite plane
A VGA plane is also available, allowing for the use of the VGA emulation hardware present in every Intel graphics device, however it is very limiting and its use is discouraged by the author.

None of the planes besides the cursor plane (in pop-up cursor mode) can be enabled alongside the VGA plane.

The display plane provides the high resolution framebuffer and needs to be enabled for any of the other planes to function (besides the VGA plane). 

As the name implies, the cursor plane provides hardware accelerated cursor drawing. Similarly, the video/sprite plane provides a second source which can be drawn on top of the display plane, its purpose is to allow for efficient use of the video decoding hardware by allowing the decoded data to be directly drawn to the window.

== Simple mode set sequence ==

A simple mode set sequence can be used with the default/built-in display, assuming that the boot firmware has configured the display timings correctly. If not, it is necessary to perform a full mode set operation involving a display power cycle.

As this is going to rely on have displays already configured by the firmware, the first step is to figure out the mapping between ports and pipes and their status.

This info is based on the intel_reg_dumper utility's output for integrated graphics for Haswell:

 Ports:
 There are 5 ports: A,B,C,D,E
 These represent connections to displays.
 The EDP port is always port A
 
 The important registers here are:
 PORT_CLK_SEL_[A-E]
 DP_TP_CTL_[A-E]
 DDI_BUF_CTL_A[A-E]
 
 Pipes:
 There are 3 general pipes A,B,C and 1 'specialized' EDP pipe that actually maps to Pipe A. Additionally, the EDP pipe (and DDI A) can only run in DP single stream (SST) mode.
 
 Pipes handle display timings/resolution settings, with the following registers:

 SFUSE_STRAP : This controls overall display capability (can disable all display output - forcing the use of external graphics), VGA port capability (can disable the physical VGA port) and the physical presence of ports on the device (regardless of if the port has something plugged in).

 PIPE_DDI_FUNC_CTL_[A-C,EDP] : Used to control how the system 'talks' to the device (HDMI/DisplayPort SST/DisplayPort MST), bits per channel, VSync and HSync polarities (from EDID) 
 WM_PIPE_[A-C] : These control when the controller generates memory accesses for pixels, the values depend on the timing information as obtained from the EDID, the exact calculation process is described in the Display Watermark section of the PRMs.
 WM_LP{1-3]: Configures the memory access timings for low power mode, the associated WM_LINETIME_[A-C] must be programmed before enabling one of these.
 WM_LINETIME_[A-C]: Configures the line timings for low power mode based on the horizontal resolution

 PIPE_SRCSZ_[A-C]: Contains the horizontal and vertical pixel width of the output.
 PIPE_CONF_[A-C/EDP]: Enable/disable the pipe and some features

 These registers are all based on the desired resolution, and are fairly well documented in the PRM, they rely on the EDID information.
 HTOTAL_[A-C/EDP] 
 HBLANK_[A-C/EDP]
 HSYNC_[A-C/EDP]
 VTOTAL_[A-C/EDP]
 VBLANK_[A-C/EDP]
 VSYNC_[A-C/EDP]
 VSYNCSHIFT_[A-C/EDP]


 Refresh rate timing:
 The calculation for these values is described in the Display chapter, section "Pipe M/N Values":
 Used for normal power mode:
 PIPE_DATA_M1
 PIPE_DATA_N1
 PIPE_LINK_M1
 PIPE_LINK_N1

 Used for low power mode:
 PIPE_DATA_M2
 PIPE_DATA_N2
 PIPE_LINK_M2
 PIPE_LINK_N2

 Once the above are configured as desired, the panels may be configured and used. For display output, the Primary panel is most interesting, followed by the cursor panel if a hardware cursor is desired.
 The EDP pipe uses Pipe A's panels.

 The Primary panel has the following registers:
 PRI_CTL_[A-C]: Enable/Disable the panel, gamma mode, pixel format, tiling, rotation
 PRI_STRIDE_[A-C]: Configure the pixel stride when reading the framebuffer (in multiples of 64-bytes)
 PRI_SURF_[A-C]: Configure the physical address of the framebuffer, this must be mapped via the global GTT
 PRI_OFFSET_[A-C]: Configure the pixel offset from which to start reading

Thus, provided that the display has already been initialized, very dirty mode setting can be performed by directly updating the primary panel registers, then the horizontal and vertical TOTAL/BLANK/SYNC registers, followed by the M/N timings and then finally the WM timings. It may be possible to make the process cleaner by disabling and then re-enabling the pipe, but it is unconfirmed if that would restore the display without having to recallibrate. However, display callibration seems to be handled by the graphics chipset itself, so it may be relatively simple to just do modesetting properly (updates upcoming).

== Full mode set sequence ==

The full mode set sequence is much longer than the simple sequence, but it is guaranteed to work with all connected displays, assuming that the EDID is valid. This sequence involves completely disabling the display, configuring the timings for the FDIs and pipes again and then powering on the display again.

The steps are as follows:

=== Link Training Notes ===
 DDI_BUF_TRANS must be configured with voltage swing information, as specified in the Display chapter, "DDI Buffer" section

 - Set the number of enabled lanes via writes to AUX 0x101, and configure the lanes, 

 First step of training is to set training pattern 1 and try voltage settings from smallest to largest until clock recovery is achieved
 - First set the sink device (via AUX channel/GMBUS) to start receiving training pattern 1 by performing a burst write to AUX 0x102-0x106, setting the lanes to expect training pattern 1 with the currently testing voltage level
 - Set the integrated graphics (DDI_TP_CTL) to send training pattern 1
 - Wait 500us
 - Read offset 0x202 from the AUX channel to get the link status to determine if training was successful.
 - If none of the voltage levels work, reduce the bitrate and try again

 - For eDP, the voltage settings can be precallibrated, allowing skipping the clock recovery process, this is determined by AUX address 0x3 bit 6 (see section 3.5.1.3 of the DisplayPort 1.4 spec)

 Second training step is to determine the pre-emphasis, it's very similar to the first step, but using training pattern 2. Clock recovery may be lost in this step, in which case the clock recovery training must be restarted.
 Once pre-emphasis training is done, the link is ready. 
 
DisplayPort 1.4 spec: [http://file.yizimg.com/383992/2014090921252964.pdf]

=== Graphics virtual memory configuration ===


=== The ring buffer ===


=== Blitter - Block Litigated Transfer engine ===


=== 3D pipeline ===

== See also ==

* [https://www.intel.com/content/www/us/en/docs/graphics-for-linux/developer-reference/1-0/overview.html Intel programming manuals] (PRMs) for Intel HD Graphics devices.

[[Category:Video]]
[[Category:Hardware]]
````

<a id="w004024"></a>

## W004024 — Graphics stack

Bron-ID: W004024
Namespace: Main (0)
Revisie: 28083 | Datum: 2023-07-09T21:10:40Z
Bron: https://wiki.osdev.org/Graphics_stack
Revisie-URL: https://wiki.osdev.org/index.php?title=Graphics_stack&oldid=28083

````text
'''''[NOTE:''' This article's original text was based on [http://forum.osdev.org/viewtopic.php?f=13&t=31328 a post in the forum]. It needs further cleanup and review. --[[User:Schol-r-lea|Schol-r-lea]] 10:19, 4 February 2017 (CST)]''

There is no fixed way to implement video handling, and many hobby operating systems develop their UI as a unit, at least initially. However, for a number of practical reasons, most systems end up separating the details of communicating with the hardware from the process of rendering the image, and that in turn from the management of higher-level concerns such as window layout. In most systems, this leads to a layered approach that could be call the '''''graphics stack''''', analogous to the networking stack.

(NB: this should not be confused with the graphics ''pipeline'', which is the sequence of operations which a GPU applies to an image being rendered.)

{{Stub}}

== A Generalized Graphics Stack ==

There is no fixed model for how to layer the graphics stack, though most are fairly similar. A general model for a graphics stack, from highest to lowest levels, might look like this:

# Application Layer
# Interoperation Layers
## Desktop Management Layer
## Window Management Layer
# Presentation Layers
## Compositing Layer
## Widget Toolkit Layer
## Rendering Layer
# Display Layers
## Device Driver Layer
## Hardware Layer

This is close to most of the well-known graphics stacks, such as Microsoft Windows, MacOS, X Window System, and Wayland, though not identical to any of them. The order of layers may differ, some layers may be missing or merged together, and some may have additional layers or even multiple stacks.

Furthermore, some layers may be more in parallel than sequential. Presentation, in particular, can involve some fairly complex relationships.

==== A note on X Window System ====
X Windows System differs from most of the others regarding its graphics stacks, as it was originally designed as a protocol for networked video, rather than as a specific display manager. X uses separate Client and Server Stacks each with their own Display Layers (even when used for rendering locally, as is the more common use case today), and splits some aspects of the remaining stacks between the Client (the remote program requesting the display being rendered) and the Server (the local system rendering the image - while this might seem to be a reversal of the usual client/server relationship, it makes sense if you view the client as the program requesting a service from a remote system). Further, the Client-Server relationship in X is potentially many-to-many, meaning that their may be several Server stacks rendering for a single client stack, while the Server in turn may be connected to other Clients and have to compose the graphics from each of them into a desktop. Finally, different applications may need different degrees of control, meaning that several details cannot be specified to belong to either the Server or the Client, but need to be negotiated between the two at the start of the operation.

=== Display Layers ===

==== Device Driver Layer ==== 

At the lowest software level, we have the device drivers, which communicate with the actual hardware. These need to be able to work with either the specific display devices - the video memory, the GPU if any, the video signal generators, and even the monitor - or some common subset of it which it shares with disparate adapters. However, this does not mean that the driver must do all the work alone. The VESA VBE/Core defines a standard minimal interface to the hardware as an extension BIOS, which a compliant video adapter should provide as a way of interfacing with the hardware without needing any proprietary details of the adapter.

Somewhere here you would find things like the Mesa driver framework and the Xlib Direct Rendering Manager. This level doesn't have a formal name in most systems, at least not as far as I know of, which is a first abstraction layer which software system (not necessarily the operating system itself) provides to give a uniform model for drawing pixels on the screen, while still exposing the underlying hardware. The split between 2-D and 3-D often starts around here, as a 3-D renderer generally needs a lot more direct hardware access than a 2-D one.

=== Prsentation Layers ===
==== Rendering Layer ====

Then next level is the renderer, which is where individual images or sections of images get created. This is where you really see 3-D becoming a separate thing, as most systems prior to, say, 2007 would have used a strictly 2-D rendering for everything that didn't specifically require 3-D rendering, due to the need for hardware acceleration for practical real-time 3-D rendering at the time. There has been a Cycle of Reincarnation for graphics rendering going between CPU-driven rendering dedicated GPU rendering and back, with current systems often using both for different purposes. Note, however, that the graphics rendering Wheel of Incarnation has been rolling since the very first days of computer graphics in the early 1960s, so it is a good guess that this won't be the last word on the subject.

For example, the Mesa library used in many Linux systems started out in the 1990s as a software 3-D renderer, but currently is used to abstract the rendering process, allowing it to use hardware acceleration while providing software rendering a fallback mode.

This is where you need to decide how you are going to handle the differences between rendering 2-D images such as basic windows and widgets, and the more impressive but also more processing-intensive 3-D rendering. While the fact that you can treat 2-D as a special case of 3-D means it is tempting to use 3-D for everything, that approach has some significant down sides, especially on older hardware. You may need to consider where you can use less general 2-D rendering to avoid a lot of hardware crunching where possible.

You also need to look at how you separate different renderable elements such as glyphs (letters, digits, text symbols, etc.), widgets (window borders, menus, icons, the mouse pointer), 2-D images such as drawings and pictures, 3-D manipulable objects, etc. This relates, and raises the issue in, the next layer of the stack, the compositor. However, before that I need to mention another part of this layer, the widget toolkit.

==== Widget Toolkit Layer ====

The widget toolkit is the set of primitive widgets - window frames, menus, drawing spaces, textboxes, text areas, radio buttons, checkboxes, etc. - that a window manager uses. In most designs, this is not a separate layer from the renderer, but side-by-side with it, and the widgets have to work together with the compositor as well.

==== Compositing Layer ====

The compositor is the part that combines the individual elements being rendered into the instantaneous display state, that is, the screen as it is at a given moment. In a 2-D design, this is usually done by the renderer directly, but 3-D UIs almost always have a separate compositor.

===== Historical Development =====
OK, quick history lesson. Early 2-D windowing systems generally composited in situ, that is, directly into the display. However, while this was feasible with the stroke-vector displays of the 1960s, or on raster displays that used fixed cells drawn from tables of glyphs such as PLATO and the majority of text-oriented terminals, this was problematic for bitmapped video systems even from the outset, as it meant that a large block of memory - often as much as 30% of system memory in the days of the Alto and 128K Macintosh - had to be set aside for the video, and the timing of drawing had to be synced with the vertical refresh in order to avoid flicker.

While double buffering was part of the answer, it ran into issues with time - copying that much data would take longer than the vrefresh, so a workable double buffer needed to be done by hardware. You would have to dedicate two buffer's worth of memory in hardware (one to drive the video, and the other to draw to), and the display would need even more hardware to let it switch which of the video buffers was driving it in order to make it work. Pretty much every video system today supports this as a matter of course. However, this did nothing for when you have to copy a bitmapped image from general memory - something loaded from a file, say - into the drawing video buffer.

In order to cut the time further, developers at Xerox PARC developed a technique called 'Bit BLT', in which a part of the image is prepared as a mask and only the mask is drawn to the video buffer. Other techniques, such as hardware sprites (which were drawn directly to the screen, bypassing the video buffer entirely) were also developed, but were mostly used in dedicated gaming and video editing systems.

I mention all this to get to compositing. Up until 2006 or so, the act of compositing for a window manager was done mainly as a 2-D action, and generally was focused on a) determining what parts of the display have changed, b) determining which parts of the screen were observable, and c) blitting the observable sections of a window that were getting changed to the draw buffer. This was generally easier for a tiling window manager, as there was no z-scaling - no windows overlapped, so everything could be drawn, and you could divide the windows into those which had changed and those which hadn't. Layering windows managers were a little more complicated because some windows might obscure parts of others, but generally it wasn't too difficult. Even so, 2-D hardware acceleration was still very useful for this, even if it wasn't absolutely necessary.

With the introduction of 3-D layered UIs such as Aqua and Aero, the issue of combining things became much more complex, leading to the need for a separate compositor layer. Most major window managers today have a 3-D compositor, and for a time it was almost impossible to get good performance from one without a dedicated GPU, meaning software rendering was out of the question even for the basic GUI, leading to issues that previously were mostly seen in gaming.

However, because this still has a performance cost, there are several Linux desktop managers (see below) which only use 2D compositing, a number of which (e.g., Notion, Awesome) are (or initially were) specifically tiling systems designed to avoid applying significant amounts of compositing. Many of these are also designed to be used keyboard-only, for those who prefer to avoid using the mouse, which is a separate issue but often seen as going hand in hand with the performance matters.

=== Interoperation Layers ===

==== Window Management Layer ====

Getting back on track, we now get to the window manager itself, which is the part that actually decided where to put each rendered component, sets things related to the way widgets interact, and just generally, well, manages the windows. This is what X Window System was from the outset, and it acts as the glue between the lower level aspects of the GUI and the more abstract parts such as the desktop manager.

==== Desktop manager Layer ====

The next layer is the desktop manager, and this is what most people are actually thinking when they talk of a GUI, and of the differences between Windows, Mac, and the various Linux desktops such as KDE, Gnome, Unity, XFCE, Cinnamon, MATE, and so forth.

=== Application Layer ===

The Application Layer is the part which is internal to the program itself.

[[Category:Graphical UI]]
````

<a id="w004026"></a>

## W004026 — Native Intel graphics

Bron-ID: W004026
Namespace: Main (0)
Revisie: 25649 | Datum: 2021-02-08T17:43:25Z
Bron: https://wiki.osdev.org/Native_Intel_graphics
Revisie-URL: https://wiki.osdev.org/index.php?title=Native_Intel_graphics&oldid=25649

````text
==Introduction==

Intel has produced a number of graphics chips that are integrated into their chipsets and processors.
Most notably modern Intel processors often come with Intel HD graphics chips.
Wikipedia contains [https://en.wikipedia.org/wiki/List_of_Intel_graphics_processing_units a list] of all those chips.
This page explains how to drive a subset of those cards. In particular it explains how to change the graphics resolution
and how to manage the hardware provided frame buffers. It does not yet explain how the GPU can be used to accelerate
2D and 3D graphics operations via shader programs.

===Prerequisites===

Before trying to implement a native driver for your OS, make sure you understand the basics first.

* Read the [[VGA Hardware]] page. Especially the part about display timings it is still relevant for modern graphics cards. You should know what horizontal/vertical active, total, sync start/end and blanking start/end values mean; and what a pixel clock is.
* You need to be able to access [[PCI]] configuration space and find MMIO regions that are determined by BARs.

==Getting EDID via DDC==

In order to determine the graphics modes that are supported on a connected monitor it is necessary to parse the monitor's [[EDID]] information. EDID is obtained via [[DDC]] which is a simple protocol over [[I²C]]. Intel graphics chips provide a relative straightforward way to interact with the I²C bus.

Before using the I²C bus a connector has to be selected in the <code>GMBUS0</code> register. This register also sets the bus rate. 100 kHz is a conservative default choice.

In order to send a packet to the bus the following steps suffice:
* Write the first 4 bytes of data to the <code>GMBUS3</code> register.
* Write the destination address, the byte count and the software-ready bit to the <code>GMBUS1</code> register. The simplest way to use the bus is to program the bus cycle selects bits so that no index or STOP is generated and the bus just enters a wait state after the packet is transferred.
* Wait until the hardware-ready bit in <code>GMBUS2</code> is set. Optionally an IRQ can be enabled to avoid busy waiting.
* Write the next 4 bytes to <code>GMBUS3</code> and wait for hardware-ready in <code>GMBUS2</code>. Repeat this last step until all data is transferred.

Receiving a packet works as follows:
* Write the destination address, the byte count, the software-ready bit and the direction bit to the <code>GMBUS1</code> register.
* Wait for hardware-ready in <code>GMBUS2</code>. Read 4 bytes of data from <code>GMBUS3</code>. Repeat this last step until all data is transferred.

After a packet was sent or received the driver should wait until the bus enters a wait state by checking for the wait-phase bit in <code>GMBUS2</code>. If the transaction (possibly consisting of multiple packets) is complete a STOP condition should be generated via the bus cycle select bit in <code>GMBUS1</code>.

==Generation 4 GMA desktop chips (aka Intel G45)==

The G45 is a PCI-Express based graphics chip that was introduced by Intel in 2008.

===Architecture overview===

G45 chips appear as devices on the PCI bus. They are identified a vendor ID of 0x8086 and a model-specific device ID. The PCI configuration space is used to access two BARs: BAR 0 points to a MMIO region that contains all registers of the card. BAR 2 allows access to the graphics memory.

The chip supports two independent graphics pipelines. Each pipeline is made of the following:
* A DPLL (digital phase-locked loop) that generates the pixel clock.
* A display pipe that is responsible for setting the display timings.
* A primary plane and secondary planes. The primary plane supplies the primary framebuffer to the display pipe. Secondary planes are mostly used to implement hardware mouse cursors.

Both pipelines share a set of connectors that are used to attach monitors to the card. G45 supports DAC (digital-to-analog converter, i.e. CRTs via the usual VGA plug) connectors, SDVO (serial digital video out) connectors, LVDS (low-voltage differential signaling) connectors and TV output. SDVO is an internal bus that is usually transcoded to HDMI or DisplayPort. LVDS is used to connect integrated flat panels to the graphics chip of laptops.

===Documentation===

The G45's registers are documented by Intel. However Intel's functional descriptions are quite sparse. The Linux kernel's i915 driver is often a good reference.

* Intel's documentation is available [https://01.org/linuxgraphics/documentation/hardware-specification-prms here]. We are primarily interested in the [https://01.org/sites/default/files/documentation/g45_vol_3_register_0_0.pdf G45: Volume Three: Display Register] that describes the display registers. [https://01.org/sites/default/files/documentation/g45_vol_1a_core_updated.pdf G45: Volume 1a Graphics Core] documents the graphics memory interface and the PCI configuration space. The [https://01.org/sites/default/files/documentation/965_g35_vol_3_display_registers_updated.pdf G35: Volume Three: Display Registers] contains a few registers that are undocumented in the G45 manual but still must be programmed by the driver.
* The [http://lxr.free-electrons.com/source/drivers/gpu/drm/i915 i915 driver] of the Linux kernel. Almost all of the mode setting logic is in [https://elixir.bootlin.com/linux/latest/source/drivers/gpu/drm/i915/display/intel_display.c intel_display.c].

===Mode setting===

Mode setting proceeds in two phases: First the display hardware needs to be deactivated. After that it can be reprogrammed and enabled again in another mode.

More specifically disabling the display hardware consists of the following steps:
* Disable all output connectors.
* Disable all planes. This includes the primary plane and cursor planes.
* Disable the display pipe.
* Disable the DPLL.
* Disable the legacy VGA emulation.

Enabling the display reverses this sequence:
* Program the DPLL to generate a suitable pixel clock and enable it. Wait for the clock to stabilize.
* Setup the display timings of your desired mode and enable the display pipe.
* Set a framebuffer address and stride and enable the primary plane and all secondary planes that you wish to use.
* Enable the output connectors.

====Programming the DPLL====

Before a display pipe can be enabled its DPLL has to be programmed to generate a suitable pixel clock for the desired graphics mode.
The DPLL clock is determined by five integer variables called N, M1, M2, P1 and P2. The relation between the DPLL clock and those variables is given by the formula:

<code>DPLL = (reference frequency * (5 * (M1 - 2) + (M2 - 2)) / N) / (P1 * P2)</code>

There are limits on various terms of this formula. Let <code>M = 5 * (M1 - 2) + (M2 - 2)</code>, <code>P = P1 * P2</code> and <code>VCO = reference frequency * (5 * (M1 - 2) + (M2 - 2)) / N</code>. Then M1 and M2 need to be chosen so that <code>M1 < M2</code>. The following table lists the G45's limits on DPLL variables:
{| class="wikitable"
|-
! scope="row"| Variable
| DPLL
| VCO
| N
| M
| M1
| M2
| P
| P1
| P2
|-
! scope="row"| Min
| style="text-align:right;"| 25,000 kHz
| style="text-align:right;"| 1,750,000 kHz
| style="text-align:right;"| 1
| style="text-align:right;"| 104
| style="text-align:right;"| 17
| style="text-align:right;"| 5
| style="text-align:right;"| 10
| style="text-align:right;"| 1
| style="text-align:right;"| 10
|-
! scope="row"| Max
| style="text-align:right;"| 270,000 kHz
| style="text-align:right;"| 3,500,000 kHz
| style="text-align:right;"| 4
| style="text-align:right;"| 138
| style="text-align:right;"| 23
| style="text-align:right;"| 11
| style="text-align:right;"| 30
| style="text-align:right;"| 3
| style="text-align:right;"| 10
|}

The reference frequency is 96,000 kHz for DAC and SDVO output.

The resulting pixel clock is the quotient between the DPLL clock and a pixel multiplier. The pixel multiplier inserts padding into the SDVO output to ensure that its DPLL always operates at a frequency between 100 MHz and 200 MHz. Note that the pixel multiplier also applies to DAC output.

In order to determine the DPLL parameters one has to:
* Take the desired pixel clock as input.
* Chose a pixel multiplier so that the pixel clock times this multiplier is in the 100 MHz to 200 MHz range. This value is the required DPLL clock.
* Compute N, M1, M2, P1 and P2 from the DPLL clock. This can be done by iterating over all possible N, M1, M2, P1 and P2 values and checking if each combination falls into the allowed limits.

After that the DPLL can be programmed. This is done by programming the N, M1 and M2 values in <code>FPA0</code> register and programming the P1 and P2 values and enabling the DPLL in the <code>DPLLA_CTRL</code> register. Ensure to set the VGA disable bit and select SDVO/DAC mode. The driver should issue a 150μs delay after enabling the DPLL to allow the clock to stabilize.

<div class="toccolours mw-collapsible mw-collapsed">
'''Reference implementations'''
<div class="mw-collapsible-content">
* Linux i915 driver (version 4.9): <code>g4x_find_best_dpll()</code> in [http://lxr.free-electrons.com/source/drivers/gpu/drm/i915/intel_display.c intel_display.c] computes the N, M1, M2, P1 and P2 values for a given DPLL clock. <code>i9xx_compute_dpll()</code> determines the DPLL register values. <code>i9xx_set_pll_dividers()</code> and <code>i9xx_enable_pll()</code> write those values to the registers.
</div></div>

====Programming the display pipes====

Before a display pipe can be enabled the display timings have to be programmed in the <code>HTOTAL_A</code>, <code>VTOTAL_A</code>, <code>HBLANK_A</code>, <code>VBLANK_A</code>, <code>HSYNC_A</code>, <code>VSYNC_A</code> and <code>PIPEASRC</code> registers. The values of the <code>PIPEASRC</code> register should usually match the horizontal and vertical active values.

Display pipes can be enabled and disabled via their <code>PIPEACONF</code> registers.

Pipes should only be enabled while their DPLLs are enabled and warmed up.

<div class="toccolours mw-collapsible mw-collapsed">
'''Reference implementations'''
<div class="mw-collapsible-content">
* Linux i915 driver (version 4.9): <code>intel_set_pipe_timings()</code> and <code>intel_set_pipe_src_size()</code> in [http://lxr.free-electrons.com/source/drivers/gpu/drm/i915/intel_display.c intel_display.c] program the display timings. <code>i9xx_set_pipeconf()</code> and <code>intel_enable_pipe()</code> enable the pipe.
</div></div>

====Handling planes====

The <code>DSPASURF</code> register control the address of the primary framebuffer. The stride of the buffer has to be programmed in the <code>DSPASTRIDE</code> register. G45 requires strides to be multiples of 64. The <code>DSPALINOFF</code> register is usually set to zero.

The primary plane can be enabled and disabled via the <code>DSPACNTR</code> register. This register also sets the pixel format.

Planes should only be enabled while the pipes they are assigned to are enabled.

====Enabling and disabling connectors====

The analog output connector can be enabled and disabled via the <code>ADPA</code> register. This register also specifies the pipe that the connector uses.

Note that connectors must only be enabled or disabled while their respective pipes are enabled.

====Disabling the legacy VGA emulation====

The VGA emulation is controlled via the <code>VGACNTRL</code> register. For non-VGA modes the <code>VGA Display Disable</code> bit should be set. Likewise the <code>VGA Centering Enable</code> bits should be set to 0.

==Intel HD Graphics==

See [[Intel HD Graphics]].

==Debugging tips==

* While debugging a display driver logging messages to the screen is often not possible. Use [[Serial Ports]] instead.
* Because there are no emulators that emulate Intel graphics cards you have to test your driver on a physical machine. Make sure that you can recompile your driver and reboot your test machine with little effort. Things like [[PXE]] might help with that.
* Read the initial values of each register and ensure you understand them before you try to change the registers.

==Implementations in hobby OSs==

* [https://github.com/avdgrinten/managarm managarm] (as of commit 0e9d37c) has a G45 mode setting driver in [https://github.com/avdgrinten/managarm/blob/master/drivers/gfx/intel/src/main.cpp drivers/gfx/intel]. Note that the code makes heavy use of C++ features, particularly operator overloading of the register classes.

[[Category:Video]]
````

<a id="w004027"></a>

## W004027 — User:Gravaera/ARMv7 Virtual Memory System Architecture,

Bron-ID: W004027
Namespace: User (2)
Revisie: 20479 | Datum: 2017-02-16T04:02:24Z
Bron: https://wiki.osdev.org/User:Gravaera/ARMv7_Virtual_Memory_System_Architecture%2C
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FARMv7_Virtual_Memory_System_Architecture%2C&oldid=20479

````text
Coming soon (TM)!
````

<a id="w004028"></a>

## W004028 — User:Gravaera/Generic Interrrupt Controller versions 3 and 4

Bron-ID: W004028
Namespace: User (2)
Revisie: 20481 | Datum: 2017-02-16T04:04:04Z
Bron: https://wiki.osdev.org/User:Gravaera/Generic_Interrrupt_Controller_versions_3_and_4
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FGeneric_Interrrupt_Controller_versions_3_and_4&oldid=20481
Doorverwijzing: Generic Interrrupt Controller versions 3 and 4

````text
#REDIRECT [[Generic Interrrupt Controller versions 3 and 4]]
````

<a id="w004029"></a>

## W004029 — User:Gravaera/Generic Interrrupt Controller

Bron-ID: W004029
Namespace: User (2)
Revisie: 20483 | Datum: 2017-02-16T04:06:31Z
Bron: https://wiki.osdev.org/User:Gravaera/Generic_Interrrupt_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FGeneric_Interrrupt_Controller&oldid=20483
Doorverwijzing: Generic Interrrupt Controller

````text
#REDIRECT [[Generic Interrrupt Controller]]
````

<a id="w004030"></a>

## W004030 — User:Stux

Bron-ID: W004030
Namespace: User (2)
Revisie: 20493 | Datum: 2017-02-18T21:22:54Z
Bron: https://wiki.osdev.org/User:Stux
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AStux&oldid=20493

````text
Here is a link to my main wiki hub:
http://www.hrwiki.org/wiki/User:Stux

For the time being I mostly intend to do minor gnome work (if I notice something I can fix).
````

<a id="w004031"></a>

## W004031 — User talk:Stux

Bron-ID: W004031
Namespace: User talk (3)
Revisie: 20494 | Datum: 2017-02-18T21:23:42Z
Bron: https://wiki.osdev.org/User_talk:Stux
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AStux&oldid=20494

````text
Please leave messages on my main wiki hub page (see my [[User:Stux|user page]].)
````

<a id="w004032"></a>

## W004032 — Category:RISC-V

Bron-ID: W004032
Namespace: Category (14)
Revisie: 20538 | Datum: 2017-02-24T15:05:38Z
Bron: https://wiki.osdev.org/Category:RISC-V
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ARISC-V&oldid=20538

````text
[http://www.riscv.org/ RISC-V] is a completely free and open ISA designed by the [http://www.riscv.org/risc-v-foundation/ RISC-V Foundation] at the EECS departement at the University of California, Berkley.<br/>
It was designed to be simple and directly implementable in real hardware, and consists of a base integer ISA, called RV32I or RV64I for 32 and 64-bit respectivly, it also provides additional standard extensions, as well as room for non-standard (e.g. community/user specified) ones.

[[Category:Platforms]]
````

<a id="w004033"></a>

## W004033 — RISC-V

Bron-ID: W004033
Namespace: Main (0)
Revisie: 28383 | Datum: 2023-10-12T17:54:42Z
Bron: https://wiki.osdev.org/RISC-V
Revisie-URL: https://wiki.osdev.org/index.php?title=RISC-V&oldid=28383

````text
{{Stub}}
<div class="center" style="width: auto; margin-left: auto; margin-right: auto;">Warning: Many parts of RISC-V are not yet final. Things can and will change! Look at the official specification for the most up-to-date information</div>
----
RISC-V is not a single ISA, rather a meta-ISA. It defines basics and boundaries for a family of implementations.
The specification is published under a Creative Commons License and actively developed on [[#External Links|Github]].
RISC-V seeks to address two problems with developing hardware processors: copyright (such as Arm's) hampering
educational and hobbyist processors, and the historical baggage associated with some architectures (such as x86) making those
architectures unwieldy to manufacture and design around.

An implementation consists of one of the [[#Base ISA|Base ISAs]] and zero or more [[#Extensions|Extensions]].

== Architecture ==
The RISC-V ISA has fixed-length 32-bit instructions aligned on their natural boundaries, but is designed to encode variable-length instructions.<br/>
The base ISA operates on a little-endian memory system, but non-standard extensions may add support for big-endian or bi-endian.

=== Hardware Threads ===
The RISC-V ISA specifies hardware threads, called ''harts''. A ''hart'' is defined by its own instruction fetch unit.
A processor may contain multiple ''harts'', at least one.
Each hart has an ID associated with it.

=== Exceptions, Traps and Interrupts ===
In RISC-V the term ''Exception'' refers to an unusual condition at run-time associated with an instruction in the current hardware thread.<br/>
A ''Trap'' is a synchronous transfer of control to a trap handler and is caused by an exceptional condition within a RISC-V thread. The trap handlers usually execute in a more privileged environment.<br/>
An external event that occurs asynchronously to the current thread will cause an ''Interrupt''. When an interrupt occurs, some instruction is selected to experience a trap.

=== Privileges ===
The spec defines 4 privilege modes, of which a valid combination has to be implemented. The modes are:

{| class="wikitable"
|Debug 
| Mode for ''complete'' control, for debuggers
|-
| Machine
| Mode with nearly full control, no debug registers (think firmware), not optional
|-
| Hypervisor
| VM-Hypervisor
|-
| Supervisor
| OS-level
|-
| User
| Application-level, lowest privilege
|}

==== Possible Combinations ====
Valid combinations of privilege levels are:
* One level: Machine mode only for embedded systems
* Two levels: Machine and User mode, small systems
* Three levels: Machine, Supervisor and User mode, complex systems able to run Unix-like operating systems


== Base ISA ==

The base ISA specifies RV32I and RV64I, 32 and 64-bit respectively; most of what is said about RV32I also applies to RV64I. Additionally, there is also RV32E, a reduced version of RV32I for embedded systems, and RV128I, a 128-bit version, which is mostly a placeholder so far.

==== RV32I ====
RV32I offers 31 general-purpose registers (x1-x31) which hold integer values, while the x0 register is hardwired to zero; all registers are 32 bits wide. It specifies a number of logical and arithmetic operations (and, or, xor, shift left and right, addition and subtraction), all of which are available with a source register or an immediate.

==== RV32E ====
RV32E reduces the number of general-purpose registers to 15 (x1-x15), and x0 is still hardwired to constant zero.

==== RV64I ====
RV64I is very similar to its 32-bit counterpart, but offers 64-bit wide registers and can read the CSRs in one operation instead of requiring the programmer to read the upper and lower half separately. Additionally, there are some instructions to work with words (32 bit) instead of double-words.

==== RV128I ====
Like RV64I but with 128bit register length.


== Extensions ==

An extension can be one of the officially defined ones or a vendor-specific one (there is opcode space explicitly reserved for this) and can define additional elements, including opcodes and registers.

The following Standard Extensions are noteworthy here:
==== Zicsr ====
This extensions defines some kind of secondary address space for '''Control and Status Register''', which is used for controlling
things like interrupts, privilege level, hart-local timers and such.
These registers can be accessed via the ''CSR instructions''.

==== RVM (Multiply-Divide Instruction Extension) ====
The RVM Extension adds instructions for multiplying, dividing and computing the remainder of a division.

==== RVA (Atomic Instruction Extension) ====
The RVA Extension adds instruction to work atomically with memory, including reserved load and conditional store.

==== RVF & RVD & RVQ (Floating Point Extensions) ====
Adds additional instructions to work with floating points and also the floating point registers.
The different extensions differ in the length/precision of the floating point numbers.

==== C (Compressed Instructions Extension) ====
Allows 16-bit variants of common instructions with a reduced register set (16 instead of 32) and
is intended to increase code density. It can be '''freely mixed''' with 32-bit instructions.


== Running RISC-V ==
=== Emulator, Simulators, etc ===
For advanced setups (multiple privilege modes, etc.), QEMU is probably the way to go. Other simulators [https://riscv.org/software-status/#simulators exist],
but the author has not yet tried them :-)

=== Real Hardware ===
Some fairly inexpensive SoCs are becoming available, notably VisionFive 2 from StartFive [https://doc-en.rvspace.org/Doc_Center/visionfive_2.html] and Star64 form Pine64 [https://wiki.pine64.org/wiki/STAR64]. 
Both of these are Raspberry Pi like devices based on JH7110 Quad-Core SiFive U74 64-Bit CPU with 
MMU, GPU, GPIO, Ethernet, et al and run Linux.
 
Also note 
[https://www.sifive.com/boards/hifive-unleashed HiFive Unleashed], which is also able to run Linux, has
multiple cores, and is expandable.

The PolarFire SoC Icicle Kit [https://www.crowdsupply.com/microchip/polarfire-soc-icicle-kit] by Microchip also contains cores by SiFive and can run Linux, but is cheaper than the HiFive Unleashed.

Most other hardware implementations (which are for sale for normal consumers) are more in the
category of "microcontrollers" and lack parts like a Memory Management Unit (MMU), I/O for humans
(keyboard, graphics, sound), etc.

The Kendryte K210 (e.g. in the MAIX Bit-board) is a RV64IMACFD_Zicsr_Zifence processor with two harts
and quite cheap.

== See Also ==

=== Articles ===
* [[HiFive-1 Bare Bones]]
* [[RISC-V_Bare_Bones|RISC-V Bare Bones]]
* [[RISC-V_Meaty_Skeleton_with_QEMU_virt_board|RISC-V Meaty Skeleton with QEMU virt board]]

=== External Links ===
* [[Wikipedia:RISC-V|RISC-V on Wikipedia]]
* [https://riscv.org/ riscv.org]
* [https://riscv.org/exchange/software/ RISC-V Software Listing]
* [https://riscv.org/technical/specifications/ RISC-V Specifications]
* [https://github.com/riscv/riscv-asm-manual/blob/master/riscv-asm.md RISC-V Assembly Programmer's Manual]

[[Category:RISC-V]]
[[Category:Instruction Set Architecture]]
````

<a id="w004034"></a>

## W004034 — Duct von Tape

Bron-ID: W004034
Namespace: Main (0)
Revisie: 24785 | Datum: 2020-06-18T12:39:06Z
Bron: https://wiki.osdev.org/Duct_von_Tape
Revisie-URL: https://wiki.osdev.org/index.php?title=Duct_von_Tape&oldid=24785

````text
Duct likes mixing code from beginner tutorials, half-way projects and completed projects. He doesn't stick to one coding style, one naming convention, etc. He doesn't have specific goals that he can achieve by understanding what is he doing, because simply he doesn't understand what is he doing. Anyway his project will be used [[Beginner Mistakes#Assuming It Will Go Nowhere|only by himself]].

==Pros and Cons==

Duct's approach has one pro: he can make use of the open-source nature of many projects. On the other side, con's are much more:
* He mixes incompatible licenses
* He creates many bugs while he is unable to fix any of them
* He makes his code totally unreadable to others and to himself
* He doesn't learn much from it

==Duct's Bookshelf==

None really, every snippet of code, even when irrelevant, can be integrated to his project.

==Opponents Position==

Writing an OS is a hard task that can't be achieved by copying and pasting random snippets of code. One has to know what does he do and how everything works together. In short, Duct is not the appropriate archetype for an OS developer.

==People And OSes Related To Duct==

* ''Insert your names here''

[[Category:Developer Archetype]]
[[Category:Developer Anti-Pattern]]
````

<a id="w004035"></a>

## W004035 — Rules of OSDev

Bron-ID: W004035
Namespace: Main (0)
Revisie: 29520 | Datum: 2025-05-16T08:38:05Z
Bron: https://wiki.osdev.org/Rules_of_OSDev
Revisie-URL: https://wiki.osdev.org/index.php?title=Rules_of_OSDev&oldid=29520

````text
{{Humor}}

* Rule of osdev #1469: never assume memory is zero'd - moondeck
* Rule of osdev #1468: never assume indices are sequential - mniip
* Rule of osdev #1470: don't divide when you can shift - glauxosdever
* Rule of osdev #fbf89521-21f1-4a52-b5cb-a0bf0f761bb4: probably avoid collisions with globally unique identifiers - sortie
* Rule of osdev #1985: Never try to build a GUI ontop of your DOS - bellezzasolo
* Rule of osdev #1986: Always ignore rules and limitations, they "probably" won't hurt you later on. - ''Killaship'' (see rule #1985)
* Rule of osdev #0xB8000: Don't do arithmetic on video pointers. - ''m712''
* Rule of osdev #4200: Always follow rule 0xB8000 instead of learning it the hard way - ''Killaship''
* Rule of osdev #0xdeadbeef: If you think something is done for you, its not. - moondeck
* Rule of osdev #0xF00F: Don't trust Intel, always check that cr1 access causes #UD. - ''aristos''
* Rule of osdev #(uint32_t)&kernel_sbrk: Do not negative-kernel-sbrk past your implementation of kernel-sbrk - ''franchufranchu'' 
* Rule of osdev #0xC10000000: Do not put something big on BSS before your full kernel is mapped (#PF here we come) - ''nexos''
* Rule of osdev #0x1000: Never use any other bootloader than [[Limine]] - ''ilobilo''
* Rule of osdev #995: If you write mouse driver first time, cursor is going to all directions, but not the one you want to - Klakap
* Rule of osdev #1: If you will stuck, do not worry, in future you will stuck again - Klakap
* Rule of osdev #148: Q:How to make yourselve crazy in most efficent way? A:Write 100000 lines of assembly program without comments and read it again - Klakap
* Rule of osdev #0xdeadbeef: Use [[UEFI]] - ''Moldytzu''
* Rule of osdev #1: according to lots of sources, a bootloader-os costs 100$ dollars and everybody is willing to pay for it
* Rule of osdev #0xffffffff80000000: On x86, get as far away from 32-bit mode as possible, as soon as possible. - ''dbstream''
* Rule of osdev #1981: real programmers use assembly language - ''lukflug''
* Rule of osdev #0xdeadc0de: If you hold a strong belief that your code is correct, it is probably not correct. On the other hand, the more doubt you have in your code, the better your code probably is. - ''dbstream''
* Rule of osdev #0xe05b: live in fear of the [[Triple-fault|Triple Fault]] reset - ''restingwitchface''
* Rule of osdev #apic: Do not put a cli instruction at the end of the bootloader and then spend 3 hours wondering why the hardware interrupts don't work. - Vitabytes
* Rule of osdev #boot: Thou shalt not expect it to boot the first time (or the 17th). - Anotheridiot
* Ruls of osdev #reboot: <s>If</s> when you find your architecture lacking for a new feature, it's often easier to tear it all down and rebuild from scratch. -''Bellezzasolo''
* ''Add your rules here - name''
````

<a id="w004036"></a>

## W004036 — Rng

Bron-ID: W004036
Namespace: Main (0)
Revisie: 20566 | Datum: 2017-03-07T01:48:18Z
Bron: https://wiki.osdev.org/Rng
Revisie-URL: https://wiki.osdev.org/index.php?title=Rng&oldid=20566
Doorverwijzing: Random Number Generator

````text
#REDIRECT [[Random Number Generator]]
````

<a id="w004037"></a>

## W004037 — Supervisor Memory Protection

Bron-ID: W004037
Namespace: Main (0)
Revisie: 24076 | Datum: 2019-11-04T08:34:00Z
Bron: https://wiki.osdev.org/Supervisor_Memory_Protection
Revisie-URL: https://wiki.osdev.org/index.php?title=Supervisor_Memory_Protection&oldid=24076

````text
Supervisor Memory Access Protection (SMAP) and Supervisor Memory Execute Protection (SMEP) are two kernel security mechanisms present in Broadwell and later Intel CPUs. SMEP prevents the kernel running in ring 0 from executing code which is user accessible. SMAP prevents the kernel from accessing userspace memory while the AC flag in the RFLAGS register is clear. These features can help harden the kernel against exploitation and prevent certain kinds of memory corruption. SMEP and SMAP are described in Volume 3A, 4.1.1 of the Intel Manual.


== Checking for Feature Availability ==
The CPUID instruction can be used to check if SMEP and SMAP are available. CPUID will report whether SMEP and SMAP are reported in leaf 7, bits 7 and 20 of the EBX register respectively.

== Initializing SMEP and SMAP ==
SMEP and SMAP can be initialized by setting bits 20 and 21 of CR4 respectively.

<source lang="C">
#define CPUID_SMEP 7
#define CPUID_SMAP 20
#define CPU_CR4_SMEP_BIT 20

#define CPU_CR4_SMAP_BIT 21

void supervisor_memory_protection_init(void) {
    uint32_t eax, ebx, ecx, edx;
    eax = 7;
    ecx = 0;
    cpuid(&eax, &ebx, &ecx, &edx);
    if (ebx & CPUID_SMEP) {
        cpu_cr4_set_bit(CPU_CR4_SMEP_BIT);
        log(Log_Info, "SMEP Enabled");
    }

    if (ebx & CPUID_SMAP) {
        cpu_cr4_set_bit(CPU_CR4_SMAP_BIT);
        log(Log_Info, "SMAP Enabled");
    }
}

</source>


== Reading and Writing User Memory with SMAP Enabled ==

Sometimes it may be necessary to modify user space accessible memory from ring 0, such as when setting up the stack for user space code, or performing certain syscalls. In these cases, SMAP protections can be disabled by setting the AC flag. This flag can be set and cleared with the STAC and CLAC instructions. A version of memcpy which allows copying data from ring 3 to ring 0, or vice versa, may look like this:
<source lang="C">
static inline void cpu_flags_set_ac(void) {
    // Set AC bit in RFLAGS register.
    __asm__ volatile ("stac" ::: "cc");
}

static inline void cpu_flags_clear_ac(void) {
    // Clear AC bit in RFLAGS register.
    __asm__ volatile ("clac" ::: "cc");
}

void *user_memcpy(void *destination, const void *source, size_t size) {
    // Disable SMAP protections.
    cpu_flags_set_ac();
    // Perform normal memcpy.
    void *ret = memcpy(destination, source, size);
    // Restore SMAP protections.
    cpu_flags_clear_ac();
    return ret;
}

</source>

[[Category:Security]]
````

<a id="w004038"></a>

## W004038 — Undefined Behavior Sanitization

Bron-ID: W004038
Namespace: Main (0)
Revisie: 28093 | Datum: 2023-07-09T21:29:07Z
Bron: https://wiki.osdev.org/Undefined_Behavior_Sanitization
Revisie-URL: https://wiki.osdev.org/index.php?title=Undefined_Behavior_Sanitization&oldid=28093

````text
Undefined Behavior occurs when running code whose behavior is not described by the programming language specification. Languages like C and C++ have quite a lot of undefined behavior to allow compilers the opportunity to generate efficient code for different architectures. When compilers encounter undefined behavior may generate code which does unexpected things, or may even do nothing at all. Fortunately, GCC can generate code to analyze a program at runtime and catch certain types of undefined behavior using a library called <tt>ubsan</tt>. <tt>ubsan</tt> can catch runtime bugs such as dereferencing NULL or non-canonical addresses, certain undefined overflow errors, shifting or multiplying data which is out of bounds, and other errors. Adding ubsan support for a hobbyist operating system is as simple as defining a few hooks which are called when the kernel detects undefined behavior, and re-compiling with special flags.

== GCC's Libsanitizer <tt>ubsan</tt> Implementation ==
GCC provides a library called Libsanitizer with a C++ implementation of the <tt>ubsan</tt> hooks for userland programs in the [https://github.com/gcc-mirror/gcc/tree/master/libsanitizer/ubsan libsanitizer/ubsan/ directory] of the GCC repository. Attempting to link libsanitizer into a kernel may be problematic, so it is instead recommended to implement ones own hooks by looking at the GCC structure and function definitions and rewriting them. [https://gitlab.com/sortix/sortix/blob/master/libc/ubsan/ubsan.c Sortix] and the Kernel of Truth have good pure C implementations of these hooks as examples.

== Compiling With <tt>ubsan</tt> ==
Compiling a kernel with <tt>ubsan</tt> support is as simple as adding <tt>-fsanitize=undefined</tt> to the flags provided to GCC. Note that adding undefined behavior sanitization will bulk up the kernel and result in lots of inefficient runtime checks. However, it's easy to add a debug specific phony target using a Makefile which builds the kernel with debug information and undefined behavior sanitization. To make a normal build, use <tt>make all</tt> and to build a special debug build use <tt>make debug</tt>
<source lang="C">
CFLAGS := -O2 -Werror -Wall
ASFLAGS := -O2 -Werror -Wall
CC := x86_64-elf-gcc

debug: CFLAGS += -g -fsanitize=undefined
debug: ASFLAGS += -g
debug: all

all:
    # Invoke GCC, GAS, etc.
    $(CC) -c myos.c -o myos.elf $(CFLAGS)
</source>

== <tt>ubsan</tt> Data Structures ==

<tt>ubsan</tt>  defines a few data structures for determining the cause and location of the error. One of the most helpful of these is the structure which holds the source location of the original code. It has a pointer to a C string with the file name which caused the error, as well as the exact line and column where it happened. There is also a type descriptor for types which may have been involved in undefined behavior, which includes a C string representing the type name. Each specific type of behavior, such as type mismatches, overflows, or out of bounds errors, has its own structure type which always begins with an embedded source location structure.
<source lang="C">
struct source_location {
    const char *file;
    uint32_t line;
    uint32_t column;
};

struct type_descriptor {
    uint16_t kind;
    uint16_t info;
    char name[];
};

struct type_mismatch_info {
    struct source_location location;
    struct type_descriptor *type;
    uintptr_t alignment;
    uint8_t type_check_kind;
};

struct out_of_bounds_info {
    struct source_location location;
    struct type_descriptor left_type;
    struct type_descriptor right_type;
};

</source>

== Example Hook Implementation ==

One particularly helpful hook is called <tt>__ubsan_handle_type_mismatch</tt>. As of this writing, this hook handles three different types of errors: NULL pointer access, unaligned memory access, or accessing memory from a pointer whose data is an insufficient size. This also catches non-canonical address dereferences on x86_64. This function takes two arguments, a pointer to a type mismatch info structure, and a pointer to the original data. We can check the data to diagnose the error, and log it for the user, and kernel panic.

<source lang="C">
// Alignment must be a power of 2.
#define is_aligned(value, alignment) !(value & (alignment - 1))

struct type_mismatch_info {
    struct source_location location;
    struct type_descriptor *type;
    uintptr_t alignment;
    uint8_t type_check_kind;
};


const char *Type_Check_Kinds[] = {
    "load of",
    "store to",
    "reference binding to",
    "member access within",
    "member call on",
    "constructor call on",
    "downcast of",
    "downcast of",
    "upcast of",
    "cast to virtual base of",
};

static void log_location(struct source_location *location) {
    logf("\tfile: %s\n\tline: %i\n\tcolumn: %i\n",
         location->file, location->line, location->column);
}


void __ubsan_handle_type_mismatch(struct type_mismatch_info *type_mismatch,
                                  uintptr_t pointer) {
    struct source_location *location = &type_mismatch->location;
    if (pointer == 0) {
        log("Null pointer access");
    } else if (type_mismatch->alignment != 0 &&
               is_aligned(pointer, type_mismatch->alignment)) {
        // Most useful on architectures with stricter memory alignment requirements, like ARM.
        log("Unaligned memory access");
    } else {
        log("Insufficient size");
        logf("%s address %p with insufficient space for object of type %s\n",
             Type_Check_Kinds[type_mismatch->type_check_kind], (void *)pointer,
             type_mismatch->type->name);
    }
    log_location(location);

    kernel_panic();
}


</source>

[[Category:C]]
[[Category:C++]]
````

<a id="w004039"></a>

## W004039 — Category:Rust

Bron-ID: W004039
Namespace: Category (14)
Revisie: 27586 | Datum: 2023-01-14T06:07:57Z
Bron: https://wiki.osdev.org/Category:Rust
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ARust&oldid=27586

````text
{{Cat main|Rust}}
Rust is a programming language sponsored by Mozilla which is potentially suitable for OS development.
[[category:languages]]
````

<a id="w004040"></a>

## W004040 — Category:Ada

Bron-ID: W004040
Namespace: Category (14)
Revisie: 24063 | Datum: 2019-10-15T07:42:31Z
Bron: https://wiki.osdev.org/Category:Ada
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AAda&oldid=24063

````text
Ada is a programming language based on [[Pascal]] developed by the United States Department of Defense which may be suitable for OS development.

[[Category:Languages]]
````

<a id="w004041"></a>

## W004041 — Template:Cat main

Bron-ID: W004041
Namespace: Template (10)
Revisie: 20582 | Datum: 2017-03-14T03:32:00Z
Bron: https://wiki.osdev.org/Template:Cat_main
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ACat_main&oldid=20582

````text
:''The main article for this category is '''[[{{{1}}}]]'''''
````

<a id="w004042"></a>

## W004042 — Category:Tone

Bron-ID: W004042
Namespace: Category (14)
Revisie: 29405 | Datum: 2025-02-17T09:01:30Z
Bron: https://wiki.osdev.org/Category:Tone
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ATone&oldid=29405

````text
This category lists pages which may not fit the appropriate tone for this Wiki, and which may need revision or complete rewriting to fit.

[[Category:Wiki administration]]
````

<a id="w004043"></a>

## W004043 — Category:Xbox

Bron-ID: W004043
Namespace: Category (14)
Revisie: 30166 | Datum: 2026-03-09T08:57:13Z
Bron: https://wiki.osdev.org/Category:Xbox
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AXbox&oldid=30166

````text
{{Cat main|Xbox}}
'''Xbox''' is a video game console developed by Microsoft which can be targeted for OS development.
[[Category:Platforms]]
````

<a id="w004044"></a>

## W004044 — File:Dawn.png

Bron-ID: W004044
Namespace: File (6)
Revisie: 20589 | Datum: 2017-03-15T03:40:10Z
Bron: https://wiki.osdev.org/File:Dawn.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ADawn.png&oldid=20589

````text
Screenshot of Dawn operating system.
````
