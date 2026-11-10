# The little book about OS development — reading guide

**Source:** Helin, E., & Renberg, A. (2015, January 19). *The little book about OS development*. Supplied PDF, 78 pages, commit fe83e27dab3c39930354d2dea83f6d4ee2928212.

The following chapter descriptions are summaries. The complete, unabridged page transcription is in [corpus/boek.md](corpus/boek.md), with source IDs B001 through B078. The original PDF remains available at [assets/book.pdf](../assets/book.pdf) for typography, tables, exact code presentation and images.

## 1. Introduction — pages 7–10

The book presents itself as a practical guide to creating an x86 operating system, not as a comprehensive theoretical OS textbook. The early chapters give explicit instructions; later chapters leave more design work to the reader. The authors describe incremental development, manual testing after small steps and their experience of collaborating on code. Prerequisites include C, systems programming and a UNIX-like development environment. Source credits, project information and licensing are included in this chapter.

Complete source: [B007](corpus/boek.md#b007) through [B010](corpus/boek.md#b010).

## 2. First Steps — pages 11–18

The example development environment consists of C/GCC, NASM, Make and Bochs. The chapter distinguishes BIOS, bootloader and kernel. GRUB loads an ELF kernel containing a Multiboot header. The first program places a recognisable value in a register. A linker script determines the entry point, sections and placement; a bootable ISO image is then assembled. Verification consists of booting in the emulator and inspecting the register value in the log. The specific packages, paths and GRUB Legacy procedure belong to the environment described.

Complete source: [B011](corpus/boek.md#b011) through [B018](corpus/boek.md#b018).

## 3. Getting to C — pages 19–22

A stack must be available before C code is called. The example reserves it in the bss section and sets the stack pointer. The chapter explains the cdecl calling convention, covers packed structs for hardware layouts and gives compiler and linker options for an environment without an available standard library. A Makefile connects compilation, linking, ISO construction and starting Bochs. The complete example code is retained unchanged in the source pages.

Complete source: [B019](corpus/boek.md#b019) through [B022](corpus/boek.md#b022).

## 4. Output — pages 23–32

Output is developed through memory-mapped I/O and I/O ports. The chapter covers VGA text cells, colours, cursor position and wrappers for in and out instructions. It then discusses serial-port configuration, baud rate, line control, FIFOs, modem control and transmitting data. Serial output is directed to a file in Bochs. Write functions and, later, printf-style output with log levels are proposed as an interface. This also introduces a driver as an abstraction layer.

Complete source: [B023](corpus/boek.md#b023) through [B032](corpus/boek.md#b032).

## 5. Segmentation — pages 33–38

A logical address is described by a segment selector and offset. A descriptor supplies a base address and limit, from which a linear address is formed. The chapter covers segment registers, the GDT, segment types and privilege levels. The minimal example uses overlapping code and data segments; segmentation primarily distinguishes privileges there. Loading the descriptor table and refreshing the segment registers and CS are discussed. Figure 5.1 on page 34 is described separately and retained as the original image.

Complete source: [B033](corpus/boek.md#b033) through [B038](corpus/boek.md#b038).

## 6. Interrupts and Input — pages 39–46

The chapter distinguishes hardware interrupts, processor exceptions and software interrupts. The IDT associates an interrupt number with a handler. It covers the state placed on the stack by the processor, error codes, saving registers, a common handler and returning from an interrupt. PIC configuration, remapping and acknowledgement follow, together with reading keyboard scancodes. The examples remain source examples; they have not been compiled or certified as error-free in this knowledge base.

Complete source: [B039](corpus/boek.md#b039) through [B046](corpus/boek.md#b046).

## 7. The Road to User Mode — pages 47–50

GRUB modules allow an external program to be loaded into memory without first building a storage driver and filesystem. Module information is passed to the kernel through the Multiboot data. The chapter shows how a small flat-binary program is created and started. It emphasises that, at this stage, the program still runs at the same privilege level as the kernel; the ability to load a program does not yet establish isolated userspace.

Complete source: [B047](corpus/boek.md#b047) through [B050](corpus/boek.md#b050).

## 8. A Short Introduction to Virtual Memory — pages 51–52

Virtual memory is introduced as an abstraction for memory addressing, process isolation and managing available physical memory. The text compares segmentation with paging and discusses the problems of contiguous physical allocation, growth and fragmentation. The chapter notes that the role of segmentation differs substantially in x86-64. The paging implementation itself follows in the next chapter.

Complete source: [B051](corpus/boek.md#b051) through [B052](corpus/boek.md#b052).

## 9. Paging — pages 53–60

The 32-bit model described distinguishes the directory, page table and frame. The chapter covers table layout, address decomposition, alignment, access permissions, identity mapping, enabling paging and TLB-related operations. The kernel is then moved into a higher-half address space with different virtual and physical placement. The bootstrap code must remain executable as the translation changes. Consequences for framebuffer addresses and boot information are also covered. Figure 9.1 and the complete scripts remain available with the source.

Complete source: [B053](corpus/boek.md#b053) through [B060](corpus/boek.md#b060).

## 10. Page Frame Allocation — pages 61–64

The frame allocator tracks which physical pages are available or occupied. The book uses memory information supplied by the bootloader and discusses reserving kernel memory and loaded modules. Linker symbols indicate the kernel boundaries. Bitmaps are among the mechanisms mentioned for this accounting. A new physical page is not automatically accessible through a usable virtual address; temporary mappings break the dependency between creating new page tables and being able to write to them. A kernel heap is then built above the frame allocator.

Complete source: [B061](corpus/boek.md#b061) through [B064](corpus/boek.md#b064).

## 11. User Mode — pages 65–68

The chapter adds user segments and explains why segment selectors alone are insufficient to protect kernel memory. Code, data, the user stack and page tables are prepared for a user process. In the example, the transition is constructed using a return frame. Entry points for C programs, flat binaries versus ELF and the first steps towards a C library are also covered. The precise addresses, selectors and code examples belong to the 32-bit configuration described.

Complete source: [B065](corpus/boek.md#b065) through [B068](corpus/boek.md#b068).

## 12. File Systems — pages 69–70

A read-only RAM filesystem is constructed as data with metadata, assembled on the development machine. Files and directories receive descriptive structures; this simplicity limits support for mutable files. The text then introduces inodes for writable systems and a VFS as an abstraction above specific filesystems. It identifies several ways to expose devices through that layer. The chapter provides direction rather than a complete implementation of a crash-resistant filesystem.

Complete source: [B069](corpus/boek.md#b069) through [B070](corpus/boek.md#b070).

## 13. System Calls — pages 71–72

System calls provide the interface through which user programs request kernel services. The chapter discusses API design and an implementation using software interrupts. A privilege transition requires a suitable kernel stack; the example uses a TSS and its associated descriptor for this purpose. The text refers to further processor manuals and syscall examples. The knowledge base preserves the API choices described without treating them as a mandatory interface for Baryon OS.

Complete source: [B071](corpus/boek.md#b071) through [B072](corpus/boek.md#b072).

## 14. Multitasking — pages 73–76

The source distinguishes execution on multiple processors from rapid alternation on one processor, and chooses the latter. It first discusses process creation and cooperative scheduling with yield. A timer interrupt then introduces preemptive scheduling. Registers and other process context must be saved and restored during a switch. Separate kernel stacks and differences between interruption in kernel mode and user mode are important considerations. This chapter does not cover a complete multiprocessor design.

Complete source: [B073](corpus/boek.md#b073) through [B076](corpus/boek.md#b076).

## Bibliography — pages 77–78

The original reference list is included in full as [B077](corpus/boek.md#b077) and [B078](corpus/boek.md#b078). An external document mentioned there is not automatically included in this knowledge base.

## The three images

### Page 12

Figure 2.1 — Example of the boot process. The four blocks and arrows read exactly: BIOS → GRUB1 → GRUB2 → OS. The caption states that each block is a program. This describes the source's boot example; the labels are not renamed or equated with another boot route.

![Original figure, page 12](../assets/figures/boek-p012-figuur1.png)

### Page 34

Figure 5.1 — Translation of logical addresses to linear addresses. At the top are a 16-bit Segment selector (bits 15–0) and a 32-bit Offset (Effective address) (bits 31–0). The selector points to a Segment descriptor. The Base address from that descriptor and the offset lead to an addition symbol. The output is a 32-bit Linear address (bits 31–0). This description follows the visible figure; checks discussed in the surrounding text are not additional blocks in the drawing.

![Original figure, page 34](../assets/figures/boek-p034-figuur1.png)

### Page 54

Figure 9.1 — Translating virtual addresses (linear addresses) to physical addresses. A 32-bit address is divided into Directory (bits 31–22; 10 bits), Table (bits 21–12; 10 bits) and Offset (bits 11–0; 12 bits). CR3 points to the Page directory using a 20-bit address field. The directory index selects a PDE (PS=0); a 20-bit address field points to the Page table. The table index selects the PTE. The 20-bit address field from the PTE and the 12-bit offset determine the Physical addr position within the illustrated 4KB Page frame. The page also discusses 4 MB pages; the drawing itself shows the path with PDE PS=0 and a separate page table.

![Original figure, page 54](../assets/figures/boek-p054-figuur1.png)
