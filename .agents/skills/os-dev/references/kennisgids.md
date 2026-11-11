# OS Development — thematic knowledge guide

This guide provides navigation and summaries of the sources included in full. It does not replace the source content. Topics are grouped editorially; linked articles retain their own terminology, design choices, examples and revision dates. Uncertainties, possible errors and historical assumptions have not been silently corrected.

## Reading the sources

Use a W number for a wiki page and a B number for a physical PDF page. Find the same number through the page index and complete source files, or open the local source. A source reference identifies a location; it does not guarantee that all code or statements there are correct. In the event of a substantive contradiction, consult the original text as well as this summary.

## Getting started, prerequisites and development order

Begin with the required programming and hardware knowledge, selection of the target platform and the first verifiable boot. The wiki includes both minimal bare-bones examples and more extensive project skeletons. The book proceeds from boot and output through interrupts and memory to user programs, filesystems, system calls and multitasking.

Source entry points: [W001748: Getting Started]; [W003073: Required Knowledge]; [W001879: Beginner Mistakes]; [W001577: What Order Should I Make Things In?]; [W002497: Bare bones]; [W003448: Meaty Skeleton].

Book: p. 7–17, p. 19–22.

## Kernel design and system architecture

The wiki distinguishes monolithic, modular, microkernel, hybrid and exokernel designs, among others. The microkernel page discusses userspace services, IPC, context switches, bootstrap dependencies and explicitly notes that recoverability requires additional design work. A model is a design choice, not automatic evidence of security.

Source entry points: [W002697: Kernel]; [W001411: Microkernel]; [W002298: Monolithic kernel]; [W001906: Modular Kernel]; [W002695: Hybrid Kernel]; [W001465: Exokernel]; [W003680: Object oriented OS].

## Programming languages and runtime

This section brings together language overviews, assembly, freestanding programming, runtime support and standard libraries. The book chooses C with NASM and uses assembly for processor-specific operations; that choice belongs to the source and is not a requirement for another OS.

Source entry points: [W001680: Languages]; [W001425: Assembly]; [W001803: C]; [W001528: C++]; [W003415: Rust]; [W003725: Implications of writing a freestanding C project]; [W003192: C Library].

Book: p. 11–12, p. 19–22, p. 67–68.

## Compilers, assemblers, linkers and builds

The source pages connect cross-compilation with the intended target platform, object files, ABIs, linker scripts and builds. The book shows how an entry symbol, sections and memory addresses together define the kernel file. Preserve the distinction between host tools and the operating system that will execute the resulting file.

Source entry points: [W001448: GCC Cross-Compiler]; [W002968: LLVM Cross-Compiler]; [W002604: Linker Scripts]; [W001759: Calling Conventions]; [W003507: System V ABI]; [W002864: Object Files]; [W001785: Makefile].

Book: p. 13–16, p. 19–22, p. 57–58, p. 61–62.

## Boot process, firmware and bootloaders

The knowledge base contains separate routes for BIOS, UEFI, existing bootloaders, custom loaders and boot protocols. The book uses BIOS, GRUB Legacy, Multiboot and a 32-bit kernel; the UEFI, Limine and 64-bit pages are separate sources alongside that learning route, not implicit replacements for it.

Source entry points: [W003050: Boot sequence]; [W001682: Bootloader]; [W001484: BIOS]; [W002924: UEFI]; [W002803: GRUB]; [W004530: Limine]; [W003350: Multiboot]; [W004562: OVMF].

Book: p. 11–17, p. 47–49.

## x86, processor modes and descriptors

This section covers registers, modes, descriptors, segmentation, processor features and extensions. The book describes the selector, descriptor, base address, limit and linear address for its 32-bit example. The wiki also covers x86-64; always retain the processor mode and data-structure layout intended by each source.

Source entry points: [W001897: IA32 Architecture Family]; [W001447: X86-64]; [W001537: Global descriptor table]; [W001488: Segmentation]; [W002018: Task State Segment]; [W002814: Setting Up Long Mode]; [W001450: CPUID].

Book: p. 33–37, p. 65–66.

## Exceptions, interrupts and timers

The sources cover interrupt tables, handlers, processor exceptions, interrupt controllers and time sources. The book builds an IDT, discusses handler stubs and error codes, and uses the PIC and PIT. The wiki includes additional controller and architecture variants; do not interchange them as though they shared the same registers and rules.

Source entry points: [W001548: Interrupt descriptor table]; [W001719: Interrupt Service Routines]; [W002400: Exceptions]; [W001478: 8259 PIC]; [W002956: Apic]; [W002434: IOAPIC]; [W001561: Programmable interval timer]; [W003378: HPET].

Book: p. 39–45, p. 74–75.

## Memory maps, paging and allocation

Distinguish physical frames, virtual address spaces and smaller heap allocations. The book covers address translation, page tables, access flags, identity mapping, higher-half mapping, reserved areas, frame accounting and temporary mappings. The wiki pages add alternative allocators and architecture-specific details.

Source entry points: [W001536: Memory Management]; [W002278: Memory Map (x86)]; [W002279: Detecting Memory (x86)]; [W002122: X86 Paging]; [W001461: Page Frame Allocation]; [W001460: Memory Allocation]; [W003452: Heap]; [W001868: Higher Half Kernel].

Book: p. 51–59, p. 61–63.

## Processes, threads and scheduling

The process pages distinguish a running program and its resources from its individual execution flows. The book progresses from a loaded module to userspace, then to cooperative and preemptive scheduling. Registers, address space and stacks form part of the state that the implementation must manage.

Source entry points: [W001415: Processes and Threads]; [W004247: Thread]; [W002603: Getting to Ring 3]; [W001418: Context Switching]; [W001477: Scheduling Algorithms]; [W001797: Blocking Process]; [W001888: Multiprocessor Scheduling].

Book: p. 47–49, p. 65–68, p. 73–75.

## Synchronisation, SMP and memory ordering

The wiki groups mutual exclusion, rendezvous, reader/writer access, semaphores, spinlocks, atomic operations and memory barriers. It discusses fairness and starvation as well as correctness. The book does not cover a multicore OS; its explanation of preemption is therefore not a complete SMP guide.

Source entry points: [W001889: Synchronization Primitives]; [W001766: Atomic operation]; [W002494: Spinlock]; [W001752: Semaphore]; [W001729: Deadlock]; [W002499: Memory barriers]; [W002671: Multiprocessing].

Book: p. 73–75.

## System calls and interprocess communication

This section brings together system calls, message passing, shared memory and RPC. The book discusses a syscall API, a software interrupt as the transfer mechanism and a TSS for switching to a kernel stack. The wiki includes other transfer mechanisms and IPC designs; their ABIs remain separate choices.

Source entry points: [W001961: System Calls]; [W003327: Sysenter]; [W001711: Message Passing]; [W003424: IPC Data Copying methods]; [W001765: Shared Memory]; [W001763: Remote Procedure Call]; [W001890: Signals].

Book: p. 71–72.

## Program formats, loaders and libraries

The sources distinguish executable formats, object files, relocation, loaders and libraries. The book begins with a flat binary and leaves ELF support for user programs as a possible extension. The later ELF and loader pages provide information for developing that step specifically.

Source entry points: [W001542: ELF]; [W003429: ELF Tutorial]; [W001954: Executable Formats]; [W001892: PE]; [W002811: COFF]; [W003824: Execution Loader]; [W002367: Loading a Process]; [W004000: Dynamic Linker].

Book: p. 47–49, p. 67–68.

## Device management, buses, DMA and IOMMU

The wiki covers discovery, device interfaces, port I/O, memory-mapped I/O, PCI, DMA and IOMMU variants. The book introduces drivers through display and serial output: other code uses a driver interface instead of directly handling every register.

Source entry points: [W002355: Device Management]; [W002037: Hardware Abstraction Layer]; [W003794: Pci]; [W002829: PCI Express]; [W003135: I/O ports]; [W003198: Memory mapped registers in C/C++]; [W001467: ISA DMA]; [W005001: IOMMU]; [W003632: VirtIO].

Book: p. 23–31.

## Platform discovery, ACPI and power management

Platform tables and firmware descriptions are grouped here: ACPI, AML, relevant tables, devicetree, discovery, shutdown and reboot. This wiki material extends the book; the book chapters do not provide a complete modern power-management or firmware framework.

Source entry points: [W002946: Acpi]; [W003184: AML]; [W001440: RSDP]; [W002712: RSDT]; [W003319: XSDT]; [W002812: FADT]; [W003259: MADT]; [W004422: Devicetree]; [W001845: Shutdown]; [W001844: Reboot].

## Block devices and disk structures

This section separates block I/O and disk layout from filesystems. The wiki covers ATA/ATAPI, AHCI, NVMe, SCSI, sector addressing, partitions and disk images. A filesystem above a device and the protocol used to read blocks are different layers.

Source entry points: [W002239: ATA PIO Mode]; [W002241: ATA/ATAPI using DMA]; [W002674: AHCI]; [W004432: NVMe]; [W004521: SCSI]; [W002970: Logical Block Addressing]; [W002868: Partition table]; [W003126: GPT]; [W001606: Disk Images].

## Filesystems, VFS and consistency

The book begins with a read-only filesystem in a RAM image, then introduces inodes and VFS. The wiki discusses specific formats, free space, naming, metadata, caches, journaling and custom filesystems. Each format remains identifiable in the text: specifications for different formats are not merged.

Source entry points: [W001505: File Systems]; [W001859: File Management]; [W002957: Hierarchical VFS Theory]; [W003194: Initrd]; [W001506: FAT]; [W001509: Ext2]; [W003000: Journaling]; [W003638: Roll Your Own Filesystem].

Book: p. 69–70.

## Network stack, protocols and network cards

The wiki includes both network hardware and the structure of a network stack and individual protocols. They are grouped here for navigation but remain separate source articles. The book has no chapter dedicated to a network stack; this section therefore draws on the wiki.

Source entry points: [W002538: Network Stack]; [W002539: Address Resolution Protocol]; [W002979: Internet Control Message Protocol]; [W002362: Ethernet devices]; [W003751: Socket]; [W002626: 802.11]; [W002789: Intel 8254x]; [W002176: RTL8139].

## USB, keyboard, mouse and serial I/O

The book uses keyboard scancodes, PIC interrupts and serial output. The wiki adds PS/2-controller details, mice, USB and host-controller variants. Scancodes, characters and input interfaces remain distinct concepts in the source structure.

Source entry points: [W001900: PS/2 Keyboard]; [W003157: PS/2 Mouse]; [W003160: I8042 PS/2 Controller]; [W002840: Serial ports]; [W001725: Universal Serial Bus]; [W003309: EXtensible Host Controller Interface]; [W003303: Enhanced Host Controller Interface].

Book: p. 23–31, p. 43–45.

## Video, graphical output, fonts and GUI

The wiki covers text output, linear framebuffers, video modes, fonts, buffering, compositing and GUIs, as well as specific GPUs. The book refers to its VGA text output as a framebuffer; that terminology is retained in the source. A text console and a complete desktop environment represent different stages of development.

Source entry points: [W001697: VGA Hardware]; [W004597: Drawing In a Linear Framebuffer]; [W004512: GOP]; [W002489: Double Buffering]; [W003354: Compositing]; [W004024: Graphics stack]; [W002346: GUI]; [W003999: PC Screen Font].

Book: p. 23–26.

## Audio and sound hardware

Audio is a wiki topic in this collection: general sound output and specific hardware families appear alongside one another. The existence of a driver article does not establish that a target computer contains that chip or that a program for another chip will work without modification.

Source entry points: [W002238: Sound]; [W002248: PC Speaker]; [W003314: AC97]; [W003313: Intel High Definition Audio]; [W002237: Sound Blaster 16]; [W003840: Ensoniq AudioPCI].

## Security, isolation and reliability

The security pages connect privilege boundaries with memory protection and higher-level security choices. The microkernel source emphasises that recovery and data preservation require additional measures. This knowledge base is source material and provides no evidence that a particular implementation is error-free, unbreakable or formally verified.

Source entry points: [W002262: Security]; [W001411: Microkernel]; [W005001: IOMMU]; [W002935: Random Number Generator]; [W002628: Stack Smashing Protector]; [W001452: CPU Bugs].

Book: p. 65–66, p. 71–72.

## Testing, debugging and fault diagnosis

The book develops in small steps and checks the kernel in an emulator; serial output later becomes a logging channel. The wiki covers emulator use, GDB, unit tests and known tutorial errors. Include log output, symbols, the selected machine configuration and the exact source revision in a reproducible fault description.

Source entry points: [W001946: Testing]; [W001918: Unit Testing]; [W001947: Kernel Debugging]; [W001544: GDB]; [W002941: Qemu]; [W001556: Bochs]; [W001540: Troubleshooting]; [W003459: James Molloy's Tutorial Known Bugs].

Book: p. 8–8, p. 16–17, p. 26–31.

## ARM, AArch64, RISC-V and other platforms

Non-x86 architectures remain fully represented, including ARM/AArch64, RISC-V, PowerPC, MIPS, SPARC and S390. Their manuals, interrupt models, memory management and bare-bones projects are not derived from the book’s 32-bit x86 code.

Source entry points: [W002332: ARM Overview]; [W004267: AArch64]; [W004033: RISC-V]; [W003306: Raspberry Pi]; [W001834: PowerPC Overview]; [W001832: MIPS Overview]; [W003045: Sparc Hardware Docs]; [W005085: IBM Mainframe architecture].

## Reference material, tools, porting and community

This section contains reference works, project examples, porting, additional computer science and community information. Links to external manuals are retained as references; the complete external documents are not automatically part of a wiki export.

Source entry points: [W001368: Resources]; [W001572: Books]; [W003903: Academic Papers]; [W001885: Licensing]; [W003405: Code Management]; [W003402: Portability]; [W003628: Cross-Porting Software]; [W001367: Projects].

