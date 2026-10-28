# Expanded Main Page — lokale leesroute

De oorspronkelijke hoofdpagina bestaat grotendeels uit MediaWiki-templates. Daarom staan hieronder ook de zes inhoudelijke Main2-templates waarnaar zij verwijst. Hun wikitext is ongewijzigd en volledig opgenomen; dit is een leesbare bundeling, geen uitvoering van de MediaWiki-renderer.

## Expanded Main Page [W004888]

<div id="mainpage"></div> __NOTOC__ __NOEDITSECTION__
<!-- For your daily editing, go to http://wiki.osdev.org/Category:Main_Page_Templates - this page is locked mainly to prevent accidental breakage -->
<!-- Welcome to OSDev.org box -->
{{Welcome}}

{{Stylenav}}

{{Flex columns
|1 = 
{{NocatBox|subject=Introduction|page= Main2/Introduction}}
{{NocatBox|subject=Design Considerations|page=Main2/Design Considerations}}
{{NocatBox|subject=Resources|page=Main2/Resources}}
{{NocatBox|subject=Tools|page=Main2/Tools}}
|2 = 
{{NocatBox|subject=Hardware|page=Main2/Hardware}}
{{NocatBox|subject=Reference Information|page=Main2/Reference Information}}
}}
[[de:Hauptseite]]

## Template:Main2/Introduction [W002291]

'''Basic Information'''
* [[Introduction]]
* [[Required Knowledge]]
* [[Beginner Mistakes]]
* [[Getting Started]]
* [[How To Ask Questions]]
* [[GCC Cross-Compiler]]
* [[What Order Should I Make Things In?]]
* [[Licensing]]
* [[Tutorials]]
* '''[[:Category:FAQ|Frequently Asked Questions]]'''
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:OS Development|more...]]</small></div>

'''Environment'''
* [[How kernel, compiler, and C library work together]]
* Using other [[Languages]] than C
* [[UEFI]] and legacy [[BIOS]] (PC)
* [[Inline Assembly]] (including [[Inline Assembly/Examples | examples]])
* [[Object Files]] and relocation


'''Bare Bones / Baby Steps'''
* [[Bare Bones]] (C and C++)
* [[Ada Bare Bones]]
* [[C Sharp Bare Bones]]
* [[D Bare Bones]]
* [[FreeBasic Bare Bones]]
* [[Go Bare Bones]]
* [[Pascal Bare Bones]]
* [[Limine Bare Bones]]
* [[:Category:Babystep | The BabyStep Tutorial]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Bare bones tutorials|more...]]</small></div>

'''Example OS Organization'''
* [[Meaty Skeleton]]


'''Booting and Setup'''
* [[Boot Sequence]]
* [[Diskless Booting]]
* The [[Global_Descriptor_Table | Global Descriptor Table]] and a [[GDT Tutorial]]
* [[Interrupts]]
* [[PS2 Keyboard | Getting Keyboard Input]]


'''Testing and Debugging'''
* [[Troubleshooting]]
* [[Testing]]
* [[Unit Testing]]
* [[How Do I Use A Debugger With My OS]]
* [[Kernel Debugging]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Troubleshooting|more...]]</small></div>

'''Development'''
* [[Going Further on x86]]
* [[Creating an Operating System]]
* [[Code Management]]


'''Compiler Development'''
* [[Compiler|What's a compiler?]]
* [[Making a Compiler | Making a Compiler (Theory)]]
* [[Code Generation and Optimization Techniques]]
** [[Implementing Conditional Statements And Loops]]
** [[Tail Recursion and Tail Call Optimization]]

<noinclude>[[Category:Main Page Templates]]</noinclude>

## Template:Main2/Design Considerations [W002293]

'''Kernel Models'''
* [[Monolithic Kernel]]
* [[Microkernel]]
* [[Exokernel]] and related models
* [[Modular Kernel]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[Kernels|more...]]</small></div>

'''Task Models'''
* [[Monotasking Systems]]
* [[Multitasking Systems]]
* [[Real-Time Systems]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Task Models|more...]]</small></div>

'''Memory and Resource Management'''
* [[Memory management]] - Overview and Introduction
* [[Segmentation]]
* [[Paging]]
* [[Memory Allocation]]
* [[Page Frame Allocation]]
* [[Memory Management Unit]], or in-depth description of recursive mappings
* [[File Management]]
* [[File Systems]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Memory management|more...]]</small></div>

'''System Calls'''
* [[SYSENTER|x86 System Calls]]
* [[ARM_System_Calls|ARM System Calls]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:System Calls|more...]]</small></div>

'''Scheduling'''
* [[Processes and Threads]]
* [[Context Switching]]
* [[Scheduling Algorithms]]
* [[Multiprocessor Scheduling]]
* [[Blocking Process]] - processes sleeping and waiting
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Processes_and_Threads|more...]]</small></div>

'''Process Sychronization'''
* [[Synchronization Primitives]] - Semaphores, mutexes, spinlocks et al.
* [[Signals]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Synchronization|more...]]</small></div>

'''Inter-Process Communication'''
* [[Message Passing]]
* [[Shared Memory]]
* [[Remote Procedure Call]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:IPC|more...]]</small></div>

<noinclude>[[Category:Main Page Templates]]</noinclude>

## Template:Main2/Resources [W002290]

* [[Books]] on OS Development
* [[Academic Papers]] on theory and design
* [[Assembly]]
* [[Disk_Images#Images_with_preinstalled_GRUB | Disk Images with preinstalled GRUB]]
* [[Deprecated]], yet relevant, bits of specifications
* [[Chip Numbers, Acronyms and Things]]
* [[Resources]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Resources|more...]]</small></div>
<noinclude>[[Category:Main Page Templates]]</noinclude>

## Template:Main2/Tools [W002292]

'''Bootloaders'''
* [[Bootloader]] theory
* [[GRUB]]
* [[Limine]]
* [[LILO]]
* [[XOSL]]
* [[System Commander]]
* [[Boot Magic]]
* [[Rolling Your Own Bootloader]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Bootloaders|more...]]</small></div>

'''Compilers'''
* [[GCC]]
** [[Cygwin]]
** [[MinGW]]
** [[Dev-C++]]
** [[DJGPP]]
** '''Building a [[GCC Cross-Compiler]]'''
** [[Cross-Compiler Successful Builds]]
** [[Porting GCC to your OS]]
* [[Calling Conventions]]
** [[System V ABI]]
* [[Watcom]] C/C++
* [[Visual Studio]]
** [[Visual C++ Runtime]]
* [[TCC]]
* [[Smaller C]]
* [[Other Compilers]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Compilers|more...]]</small></div>

'''Assemblers'''
* [[GAS]] (GNU as, part of binutils)
* [[FASM]]
* [[NASM]]
* [[YASM]]
* On the difference between AT&T and Intel [[Opcode syntax]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Assemblers|more...]]</small></div>

'''Linkers'''
* [[LD]] (GNU ld, part of binutils)
* [[Link Archiver]] (GNU ar, part of binutils)
* [[Mold]] (Modern Linker)
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Linkers|more...]]</small></div>

'''Emulators'''
* [[Bochs]]
* [[QEMU]]
* [[Microsoft Virtual PC|Virtual PC]]
* [[VMware]]
* [[VirtualBox]]
* [[SimICS]]
* [[Lingemu]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Emulators|more...]]</small></div>

'''Working with Disk Images'''
* [[Disk Images]] in general, and ...
** [[Loopback Device]] for Linux, *BSD et al. 
** [[hdiutil]] for MacOS X
** [[Windows Tools]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Disk_Image_Utilities|more...]]</small></div>
<noinclude>[[Category:Main Page Templates]]</noinclude>

## Template:Main2/Hardware [W002289]

* The [[:Category:Hardware|Hardware]] Category and [[Resources#Hardware_Specifications|linked resources]]

<big>
'''CPU''' <br/>
</big>

'''General'''<br />
* [[Register Machine Model of Computation]]
* [[Memory Architecture]]
* [[Instruction Set Architecture]]
** [[Historical Notes on CISC and RISC]]
* [[Execution Architecture]]
** [[Multiprocessing Architecture]]

'''X86''' <br/>
<small>
'''Modes'''
</small>
*[[Real Mode]]
*[[Protected Mode]]
* [[Virtual 8086 Mode]]
* [[X86-64|x86-64]] <!--Architecture Family-->
<small>
'''Other'''
</small>
* [[IA32 Architecture Family]]
* [[Model Specific Registers]] (MSRs)
* [[Detecting CPU Speed]]
* [[Inline Assembly/Examples]]
* [[CPU Bugs]]
* [[SSE]]
* [[Where Can I Find Information About Ports]]
* CPU Registers ([[CPU_Registers_x86|x86]], [[CPU_Registers_x86-64|x86-64]])
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:X86 CPU|more...]]</small></div>

'''ESA/390'''
* [[S390 Barebones|Kernel Barebones]]
* [[Program Status Word]]
* [[PSA Mapping|Processing Storage Area Memory Mapping]]
* [[ZDSFS|z/OS DataSet FileSystem]]
* [[Command SubSystem|Batch I/O Channel SubSystem]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:S390|more...]]</small></div>

'''ARM'''
* [[ARM Overview]]
* [[Raspberry Pi Bare Bones]]
* [[AArch64 Exceptions]]

<big>
'''Memory'''
</big>
* [[Detecting Memory (x86)]]
* [[A20 Line]] or "Why can't I access all my memory?"


<big>
'''IRQs and Exceptions, PIC, NMI, APIC, OPIC'''
</big>
* [[I Can't Get Interrupts Working]]
* The [[Interrupt_Descriptor_Table | Interrupt Descriptor Table]] and [[IDT_problems | IDT problems]]
* [[Interrupt Service Routines]] (ISRs)
* [[Interrupts]]
* [[Non Maskable Interrupt]] (NMI)
* [[APIC]]
* [[PIC]]
* [[OPIC]]

<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Interrupts|more...]]</small></div>

<big>
'''Clocks, Timers and Counters'''
</big>
* [[HPET]]
* [[APIC timer]] (per-processor timer)
* [[Programmable Interval Timer]] (PIT)
* The [[CMOS]] clock, and its [[RTC]] chip
* [[Detecting CPU Speed]]


<big>
'''Video'''
</big>
* [[Accelerated Graphic Cards]]
** [[Native Intel graphics]]
* [[Detecting Colour and Monochrome Monitors]]?
* [[Printing To Screen]] in Protected Mode
* [[Text Mode Cursor]]
* [[Drawing In a Linear Framebuffer]]
* [[Kernel Stdio Theory]]
* [[VGA Resources]] and [[VGA Hardware]] documentation
* Drawing characters in graphics mode using [[VGA Fonts]]
* [[Video Signals And Timing]]
* How to program Bochs' and QEMU's [[Bochs VBE Extensions | BGA]] graphics device
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Video|more...]]</small></div>

<big>
'''Audio'''
</big>
* [[Sound]]
* [[PC Speaker]]
* [[Intel High Definition Audio]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Sound|more...]]</small></div>

<big>
'''Storage devices'''
</big>
* [[:Category:ATA|ATA]] (harddisks)
* [[ATAPI]] (CD-ROM)
* [[AHCI]] ([[SATA]] native)
* [[ISA DMA | DMA]]
* [[NVMe]]
* [[Floppy Disk Controller]] (FDC)
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Storage|more...]]</small></div>

<big>
'''Input/Output'''
</big>
* [[PS/2 Keyboard]]
* [[Mouse Input | Mouse]]
* [[Serial Ports]] (primarily RS-232)


<big>
'''Plug and Play'''
</big>
* [[Plug-and-Play]] (PnP)
* [[PNP Calls In Protected Mode|PnP Calls In Protected Mode]]


<big>
'''PCI, USB and things like this ...'''
</big>
* [[PCI]] 
* [[PCI Express]]
* [[Universal Serial Bus | USB]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Buses|more...]]</small></div>

<big>
'''UEFI'''
</big>
*[[UEFI]]
*Systems with [[broken UEFI implementations]]


<big>
'''ACPI'''
</big>
*[[ACPI]]
*[[RSDP]]
*[[AML]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:ACPI|more...]]</small></div>

<big>
'''Network cards and communicating devices'''
</big>
* Ethernet and other wired networking:
** [[3c90x]]
** [[Intel 8254x]]
** [[Ne2000]]
** [[RTL8139]]
** [[RTL8169]]
** [[Intel Ethernet i217]]
** [[AMD PCNET|AMD PCnet]]
* Wireless Networking:
** [[802.11]] (WiFi standard)
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Network Hardware|more...]]</small></div>

<big>
'''Non-x86 Development'''
</big>
* [[:Category:PowerPC|PowerPC]]
* [[:Category:MIPS|MIPS]]
* [[:Category:Sparc|Sparc]]
* [[:Category:ARM|ARM]]
* [[:Category:RISC-V|RISC-V]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Platforms|more...]]</small></div>

<noinclude>[[Category:Main Page Templates]]</noinclude>

## Template:Main2/Reference Information [W002288]

'''Executable File Types'''
* '''64 bit'''
** [[ELF]]
** [[PE]]
** [[COFF]]
* '''32 bit'''
** [[ELF]]
** [[PE]]
** [[COFF]]
* '''16 bit'''
** [[COM]]
** [[MZ]]
** [[NE]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Executable Formats|more...]]</small></div>

<big>'''Filesystems'''</big>
* [[FAT|FAT Details]]
* [[Ext2|Ext2 Details]]
* [[SFS|SFS Details]]
* [[NTFS|NTFS Details]]
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:Filesystems|more...]]</small></div>
<noinclude>[[Category:Main Page Templates]]</noinclude>
