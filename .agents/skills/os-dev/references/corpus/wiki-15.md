# OSDev Wiki — volledige bronteksten, deel 15

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w005078"></a>

## W005078 — File:UX20260722-234105.png

Bron-ID: W005078
Namespace: File (6)
Revisie: 30899 | Datum: 2026-07-22T15:54:18Z
Bron: https://wiki.osdev.org/File:UX20260722-234105.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AUX20260722-234105.png&oldid=30899

````text
Uinxed-Kernel-Running
````

<a id="w005079"></a>

## W005079 — File:NeuroSamaOSImg.png

Bron-ID: W005079
Namespace: File (6)
Revisie: 30912 | Datum: 2026-08-19T12:46:10Z
Bron: https://wiki.osdev.org/File:NeuroSamaOSImg.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ANeuroSamaOSImg.png&oldid=30912

````text
NeuroSamaOSImgRun
````

<a id="w005080"></a>

## W005080 — File:NeuroSamaOSRunningPng.png.png

Bron-ID: W005080
Namespace: File (6)
Revisie: 30913 | Datum: 2026-08-19T12:49:51Z
Bron: https://wiki.osdev.org/File:NeuroSamaOSRunningPng.png.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ANeuroSamaOSRunningPng.png.png&oldid=30913

````text
NeuroSamaOSRunning
````

<a id="w005081"></a>

## W005081 — File:NeuroSamaOSRunningPng.png

Bron-ID: W005081
Namespace: File (6)
Revisie: 30914 | Datum: 2026-08-19T12:55:52Z
Bron: https://wiki.osdev.org/File:NeuroSamaOSRunningPng.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ANeuroSamaOSRunningPng.png&oldid=30914

````text

````

<a id="w005082"></a>

## W005082 — File:BoredOSScreenshot26.jpg

Bron-ID: W005082
Namespace: File (6)
Revisie: 30915 | Datum: 2026-08-21T01:01:41Z
Bron: https://wiki.osdev.org/File:BoredOSScreenshot26.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABoredOSScreenshot26.jpg&oldid=30915

````text
A screenshot of the BoredOS DE, Nova.
````

<a id="w005083"></a>

## W005083 — PicoEFI

Bron-ID: W005083
Namespace: Main (0)
Revisie: 31034 | Datum: 2026-10-22T12:32:01Z
Bron: https://wiki.osdev.org/PicoEFI
Revisie-URL: https://wiki.osdev.org/index.php?title=PicoEFI&oldid=31034

````text
[https://github.com/PicoEFI/PicoEFI PicoEFI] is a minimal build environment for [[UEFI]] applications for C and C++. It was originally forked from [[GNU-EFI]] and stripped down to just the relocation stub, the linker scripts, and the EFI header files, discarding GNU-EFI's convenience library (''libefi.a'') and its ''uefi_call_wrapper'' machinery entirely. It has been later rewritten clean-room to strip all code originating or derived from GNU-EFI with the help of LLMs, and wholly relicensed to the 0BSD license (close to public domain in terms).

== Background ==
PicoEFI has been through a couple of names:

* It was forked from '''gnu-efi''' as '''reduced-gnu-efi''' in 2021.
* It was later renamed to '''limine-efi''' in 2022, and then '''nyu-efi''' in 2024.
* In 2025 it was briefly hosted on Codeberg before returning to GitHub. During that period it was renamed again to '''PicoEFI'''.

== Why PicoEFI? ==
Compared to GNU-EFI style setups, MinGW-based approaches, LLVM/lld-link, or EDK2, PicoEFI's maintainers point to a few concrete advantages:

* '''Toolchain portability''': It builds with any ELF toolchain, be it GCC or Clang, links with GNU ld (bfd), lld, or gold, and supports GNU's and LLVM's versions of binutils, in any combination. There is no requirement to install or (worse) build a MinGW cross-toolchain or use LLVM/lld-link just to get a toolchain that natively understands the Microsoft ABI and PE — any ELF toolchain, which is what practically every Unix-like system already ships as their system toolchain, or something like ''<architecture>-elf''-targeted freestanding GCC toolchains, will work out of the box.
* '''No per-toolchain special-casing''': Because it does not depend on a PE-native toolchain or other special tooling, there is no need to maintain separate code paths in the build system or elsewhere for MinGW-based GCC builds vs LLVM/lld-link builds vs EDK2 style builds... — the ELF toolchains that PicoEFI relies on are compatible enough that one simple and streamlined build setup covers them all (see the [https://github.com/PicoEFI/picoefi-c-template PicoEFI C template]).
* '''Reaches targets MinGW does not''': Building natively from ELF means PicoEFI can produce working UEFI binaries for architectures MinGW has no cross-compiler for at all, such as RISC-V and LoongArch.
* '''A correctly zeroed BSS''': Rather than a hardcoded one in the binary like GNU-EFI does.
* '''No library baggage''': No intrusive library beyond pure EFI. No ''Print()'', no ''InitializeLib()'', no ''uefi_call_wrapper''... Pure EFI functions-only are invoked directly from the System Table.
* '''No GNU-EFI bugs''': PicoEFI has fixed several bugs (including some serious ones) that the portions of GNU-EFI it was forked from had.

== What it provides ==
For each supported architecture, PicoEFI provides:

* A PE header and entry point assembly stub providing the entry point that the firmware calls, which in turn calls your ''efi_main''.
* The relocation code needed to fix up the position-independent PE image at load time.
* A linker script for turning the compiler's native ELF output into a PE/COFF image firmware can load.

And globally, for all architectures supported, it provides:

* A set of EFI headers compatible with C and C++ (types, protocols, the System Table, etc.).

=== Supported architectures ===
{| class="wikitable"
|-
! Directory
! Architecture
|-
| ia32 || x86 (32-bit)
|-
| x86_64 || x86-64
|-
| arm || ARM32 (not exercised in the example template)
|-
| aarch64 || ARM64
|-
| riscv32 || RISC-V (32-bit, not exercised in the example template)
|-
| riscv64 || RISC-V (64-bit)
|-
| loongarch32 || LoongArch (32-bit, not exercised in the example template)
|-
| loongarch64 || LoongArch (64-bit)
|}

== License ==
PicoEFI used to be distributed under a mix of licenses requiring careful consideration by users, but after the clean-room rewrite, it is 0BSD-only, allowing embedding in projects using pretty much any license, including proprietary ones.

== Getting started ==
The easiest way to start a project against PicoEFI is the official template, which vendors PicoEFI as a dependency and provides a ready-to-use Makefile:

<syntaxhighlight lang="bash">
$ git clone https://github.com/PicoEFI/picoefi-c-template.git
$ cd picoefi-c-template
$ ./get-deps  # Obtain the dependencies the project relies on, including PicoEFI itself.
$ make clean all run
</syntaxhighlight>

=== Minimal example ===
Since there is no convenience library, even printing a string means going through the System Table's console-output protocol directly:
<syntaxhighlight lang="c">
#include <efi.h>

EFI_STATUS efi_main(EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable) {
    (void)ImageHandle;
    SystemTable->ConOut->OutputString(SystemTable->ConOut, L"Hello world\r\n");
    for (;;);
}
</syntaxhighlight>

== See also ==
=== Articles ===
* [[UEFI]]
* [[GNU-EFI]]
* [[POSIX-UEFI]]
* [[EDK2]]
* [[UEFI App Bare Bones]]

=== External Links ===
* [https://github.com/PicoEFI/PicoEFI PicoEFI source]
* [https://github.com/PicoEFI/picoefi-c-template PicoEFI C template]

[[Category:x86]]
[[Category:x86-64]]
[[Category:ARM]]
[[Category:UEFI]]
````

<a id="w005084"></a>

## W005084 — STSI and ECAG

Bron-ID: W005084
Namespace: Main (0)
Revisie: 31026 | Datum: 2026-10-18T07:40:28Z
Bron: https://wiki.osdev.org/STSI_and_ECAG
Revisie-URL: https://wiki.osdev.org/index.php?title=STSI_and_ECAG&oldid=31026

````text
{{Rating|1}}

On the '''s390x''' (IBM z/Architecture) architecture, hardware discovery is significantly different from architectures like x86 (which relies on `CPUID`) or ARM (which uses system registers and Device Trees / ACPI). 

System identification, CPU capability, multithreading configuration, logical partition (LPAR) status, virtualization nesting, and CPU cache hierarchies are queried primarily through two instructions:
* '''STSI''' (''Store System Information'')
* '''ECAG''' (''Extract CPU Attribute'')

These two instructions are currently the primary standard mechanism for acquiring hardware, topology, and performance metrics on 64-bit z/Architecture systems.

----

== STSI (Store System Information) ==

The <code>STSI</code> instruction retrieves detailed hardware, partition, virtualization, and topology information into a 4096-byte (4 KiB) page-aligned buffer called the '''System-Information Block (SYSIB)'''.

=== Instruction Format & Register Arguments ===

<syntaxhighlight lang="text">
    STSI D2(B2)
</syntaxhighlight>

* '''Operand''': Address of a 4 KiB-aligned memory buffer (SYSIB).
* '''General Register 0 (GR0)''':
** Bits 32–35: '''Function Code (FC)'''
** Bits 36–55: Reserved (must be 0, otherwise a specification exception occurs).
** Bits 56–63: '''Selector 1 (Sel1)'''
* '''General Register 1 (GR1)''':
** Bits 32–47: Reserved (must be 0, otherwise a specification exception occurs).
** Bits 48–63: '''Selector 2 (Sel2)'''

=== Function Codes and Selectors ===

The configuration hierarchy on z/Architecture is defined across three distinct levels:
# '''Level 1 (Basic Machine):''' Bare-metal hardware.
# '''Level 2 (Logical Partition / LPAR):''' PR/SM hypervisor partition.
# '''Level 3 (Virtual Machine / VM):''' Hypervisors like z/VM or KVM.

{| class="wikitable"
! FC !! Sel1 !! Sel2 !! Target / Information Requested !! Output SYSIB Structure
|-
| <code>0</code> || — || — || Query current configuration level number || No SYSIB written; Level (1, 2, or 3) returned in GR0[32..35].
|-
| <code>1</code> || <code>1</code> || <code>1</code> || Basic Machine Configuration (Model, serial, capacity) || <code>SYSIB 1.1.1</code>
|-
| <code>1</code> || <code>2</code> || <code>1</code> || Basic Machine CPU (Executing CPU information) || <code>SYSIB 1.2.1</code>
|-
| <code>1</code> || <code>2</code> || <code>2</code> || Basic Machine CPUs (All physical CPUs / SMT capability) || <code>SYSIB 1.2.2</code>
|-
| <code>2</code> || <code>2</code> || <code>1</code> || LPAR CPU (Executing logical CPU) || <code>SYSIB 2.2.1</code>
|-
| <code>2</code> || <code>2</code> || <code>2</code> || LPAR CPUs (Dedicated/shared CPUs, LPAR name, CAF) || <code>SYSIB 2.2.2</code>
|-
| <code>3</code> || <code>2</code> || <code>2</code> || VM CPUs (Virtual Machine descriptors, hypervisor name) || <code>SYSIB 3.2.2</code>
|-
| <code>15</code> || <code>1</code> || <code>2–6</code> || CPU Configuration Topology (Core nest & grouping) || <code>SYSIB 15.1.x</code>
|}

=== Condition Codes ===
* '''CC 0:''' Information successfully stored (or configuration level returned).
* '''CC 3:''' Function code / selector combination is invalid, the requested level is higher than the current operating level, or the required facility is not installed.

=== Querying the Current Configuration Level (FC = 0) ===

When <code>FC = 0</code>, the operand address is ignored, and the executing CPU writes its current nesting level directly into bits 32–35 of GR0.

<syntaxhighlight lang="c">
#include <stdint.h>

uint8_t stsi_get_current_level(void) {
    register uint64_t r0 __asm__("0") = 0;
    register uint64_t r1 __asm__("1") = 0;
    int cc;

    __asm__ volatile (
        "   stsi 0\n"
        "   ipm  %[cc]\n"
        "   srl  %[cc], 28\n"
        : [cc] "=d" (cc), "+d" (r0)
        : "d" (r1)
        : "cc", "memory"
    );

    if (cc != 0) {
        return 0; // STSI failed or not supported
    }

    // Level number is extracted from bits 32..35 of GR0
    return (uint8_t)((r0 >> 28) & 0x0F);
}
</syntaxhighlight>

=== Fetching a SYSIB Block ===

The following generic function performs a full STSI call against a 4 KiB-aligned page:

<syntaxhighlight lang="c">
#include <stdint.h>
#include <stddef.h>

int stsi_query(void *sysib_page, uint8_t fc, uint8_t sel1, uint16_t sel2) {
    uint64_t r0 = ((uint64_t)(fc & 0x0F) << 28) | (uint64_t)(sel1 & 0xFF);
    uint64_t r1 = (uint64_t)sel2;

    register uint64_t reg0 __asm__("0") = r0;
    register uint64_t reg1 __asm__("1") = r1;
    int cc;

    __asm__ volatile (
        "   stsi %[sysib]\n"
        "   ipm  %[cc]\n"
        "   srl  %[cc], 28\n"
        : [cc] "=d" (cc), [sysib] "=Q" (*(char (*)[4096])sysib_page)
        : "d" (reg0), "d" (reg1)
        : "cc", "memory"
    );

    return cc; // 0 = Success, 3 = Invalid/Unavailable
}
</syntaxhighlight>

----

== ECAG (Extract CPU Attribute) ==

The <code>ECAG</code> instruction retrieves specific processor attributes such as cache topology, cache line sizes, cache sizes, associativity, and CPU clock frequencies.

=== Instruction Format & Operands ===

<syntaxhighlight lang="text">
    ECAG R1, R3, D2(B2)
</syntaxhighlight>

On 64-bit systems, <code>R3</code> is ignored (set to 0), and the second operand address <code>D2(B2)</code> contains an encoded request bitfield. The result is returned in general register <code>R1</code>.

=== Request Format in Second Operand Address ===

The address register contains parameter bits formatted as follows (IBM bit numbering, where 0 is MSB):

* '''Bits 54–55: Attribute Set Indication (ASI)'''
** <code>0</code>: Cache Attributes
** <code>1</code>: CPU Attributes
* '''Bits 56–59: Attribute Indication (AI)'''
** When ASI = 0 (Cache):
*** <code>0</code>: Extract Cache Topology Summary
*** <code>1</code>: Extract Cache Line Size (in bytes)
*** <code>2</code>: Extract Cache Size (in bytes)
*** <code>3</code>: Extract Cache Associativity / Sets
** When ASI = 1 (CPU):
*** <code>0</code>: Extract CPU Speed (Nominal / Dynamic in MHz)
* '''Bits 60–62: Level Indication (LI)''' (When ASI = 0)
** <code>0..7</code>: Cache Level L1 through L8
* '''Bit 63: Type Indication (TI)''' (When ASI = 0)
** <code>0</code>: Data Cache / Unified Cache
** <code>1</code>: Instruction Cache

=== Return Values & Error Handling ===

* On success, the requested scalar value is returned in register <code>R1</code>.
* If a requested attribute, cache level, or property is unavailable, <code>ECAG</code> returns <code>~0ULL</code> (<code>0xFFFFFFFFFFFFFFFF</code>).

=== Code Example: Querying CPU Cache & Frequency ===

<syntaxhighlight lang="c">
#include <stdint.h>
#include <stdbool.h>

#define ECAG_INVALID (~0ULL)

static inline uint64_t ecag_invoke(uint8_t asi, uint8_t ai, uint8_t li, uint8_t ti) {
    uint64_t param = ((uint64_t)(asi & 0x03) << 8) |
                     ((uint64_t)(ai  & 0x0F) << 4) |
                     ((uint64_t)(li  & 0x07) << 1) |
                     ((uint64_t)(ti  & 0x01));

    uint64_t result;
    __asm__ volatile (
        "   ecag %0, 0, 0(%1)\n"
        : "=d" (result)
        : "a" (param)
        : "cc"
    );
    return result;
}

/* Query Cache Line Size for a specific level (0 = L1, 1 = L2, etc.) */
uint64_t get_cache_line_size(uint8_t level, bool is_instruction) {
    uint64_t res = ecag_invoke(0, 1, level, is_instruction ? 1 : 0);
    return (res == ECAG_INVALID) ? 0 : res;
}

/* Query Cache Size in bytes */
uint64_t get_cache_size(uint8_t level, bool is_instruction) {
    uint64_t res = ecag_invoke(0, 2, level, is_instruction ? 1 : 0);
    return (res == ECAG_INVALID) ? 0 : res;
}

/* Query CPU Frequency (MHz) */
void get_cpu_speed(uint32_t *dynamic_mhz, uint32_t *static_mhz) {
    uint64_t res = ecag_invoke(1, 0, 0, 0);
    if (res == ECAG_INVALID) {
        *dynamic_mhz = 0;
        *static_mhz  = 0;
        return;
    }
    *dynamic_mhz = (uint32_t)(res >> 32); /* Current / Turbo speed */
    *static_mhz  = (uint32_t)(res & 0xFFFFFFFF); /* Nominal base speed */
}
</syntaxhighlight>

== See Also ==
* [[CPUID]]
* [[SMP]]

[[Category:S390]]
[[Category:CPU]]
````

<a id="w005085"></a>

## W005085 — IBM Mainframe architecture

Bron-ID: W005085
Namespace: Main (0)
Revisie: 30934 | Datum: 2026-09-01T00:37:40Z
Bron: https://wiki.osdev.org/IBM_Mainframe_architecture
Revisie-URL: https://wiki.osdev.org/index.php?title=IBM_Mainframe_architecture&oldid=30934

````text
{{Architecture Navigation|s390x}}

The '''IBM Mainframe architecture''' (spanning '''System/360''', '''System/370''', '''370-XA''',  '''ESA/370''', '''System/390''', and modern '''z/Architecture''' / '''s390x''') represents the longest continuously evolving computing architecture in history (1964–present). 

Unlike workstation or commodity microcomputer architectures (x86, ARM, RISC-V) which focus primarily on general-purpose compute pipelines, the mainframe architecture is fundamentally '''system-driven'''. It was engineered from the ground up around massive I/O throughput, high-reliability enterprise business workloads, dynamic hardware virtualization, coprocessor offloading, and strict binary backward compatibility across six decades.

----

== The "31-Bit Architecture" Misconception ==

A widespread misconception among software developers is that legacy mainframes were "31-bit architectures" with 31-bit registers.

'''From System/360 (1964) to ESA/390 (1990), the architecture has always had 32-bit General Purpose Registers (GPRs 0–15) and 32-bit ALU operations.'''

What changed over the decades was not the register width, but the '''Storage Addressing Mode (AMODE)''' and physical address bus width:

{| class="wikitable"
! Architecture Generation !! Year !! GPR Width !! Virtual Addressing !! Physical / Real Addressing !! Addressing Mode (AMODE)
|-
| '''System/360''' || 1964 || 32-bit || 24-bit (Model 67: 24/32-bit) || 24-bit (16 MiB max) || AMODE 24
|-
| '''System/370''' || 1970 || 32-bit || 24-bit (16 MiB) || 24-bit (Some models: 26-bit / 64 MiB) || AMODE 24
|-
| '''System/370-XA''' || 1983 || 32-bit || 24-bit / 31-bit (2 GiB) || 31-bit (2 GiB) || Bimodal: AMODE 24, AMODE 31
|-
| '''ESA/370 & ESA/390''' || 1988 / 1990 || 32-bit || 31-bit + Access Registers (16 x 2 GiB) || 31-bit (Later models: 36-bit) || AMODE 24, AMODE 31, AR Mode
|-
| '''z/Architecture (s390x)''' || 2000–Present || '''64-bit''' || 24-bit / 31-bit / 64-bit (16 EiB) || 64-bit || Trimodal: AMODE 24, AMODE 31, AMODE 64
|}

=== Why 31-Bit Addressing? ===
When IBM introduced 370-XA in 1983 to break past the 16 MiB barrier of 24-bit addressing, programs frequently abused the unused top 8 bits of 32-bit address words in memory to store application flags and status tags. 

To preserve compatibility while allowing 2 GiB address spaces, IBM adopted a '''31-bit address scheme'''. Bit 0 (the MSB in IBM notation) of a 32-bit address was reserved as an '''Addressing Mode (AMODE) indicator''':
* If bit 0 was `0`, the CPU operated in 24-bit mode (wrapping at 16 MiB).
* If bit 0 was `1`, the CPU operated in 31-bit mode (allowing access up to 2 GiB).

----

== Chronological Architecture Evolution ==

=== 1. System/360 (1964): The Foundation ===
Introduced by Gene Amdahl, Fred Brooks, and Gerrit Blaauw, the S/360 unified scientific and commercial computing into a single scalable architecture:
* Standardized the '''8-bit byte''', byte-addressable memory, and 2's complement 32-bit integers.
* Introduced 16 General Purpose Registers (GPR0–GPR15), 4 Floating-Point Registers (FPRs), and the '''Program Status Word (PSW)'''.
* Introduced asynchronous '''I/O Channels''' (Selector, Byte Multiplexer).
* '''System/360 Model 67 (1967):''' Added the first dynamic address translation (DAT) hardware for paging and virtual memory under TSS/360 and CP-67 (the ancestor of modern VM hypervisors).

=== 2. System/370 (1970): Virtual Storage Standardization ===
* Standardized [[DAT]] across the entire enterprise product line.
* Introduced dual-space virtual memory and inter-CPU signaling for multiprocessing.
* Introduced the '''Extended Control (EC) Mode PSW'''.
* Extended physical memory bus addressing from 24 bits to 26 bits (Extended Real Addressing / ERA, 64 MiB) on high-end models (3033, 3081).

=== 3. System/370 Extended Architecture / 370-XA (1983) ===
* Introduced '''31-bit virtual and real storage addressing''', breaking the 16 MiB memory limit and expanding addressability to 2 GiB.
* Introduced the '''Channel Subsystem (CSS)''': dedicated channel processors independent of specific CPU attachments.
* Bimodal program execution (seamlessly switching between AMODE 24 and AMODE 31 via `BSM` and `BASSM`).

=== 4. ESA/370 (1988) & ESA/390 (1990) ===
* Introduced '''Enterprise Systems Architecture'''.
* '''Access Registers (AR Mode):''' Added 16 Access Registers (AR0–AR15). When in AR mode, instructions can access data across up to 16 separate 2 GiB virtual address spaces (Data Spaces) simultaneously without swapping CR1 page tables.
* '''PR/SM (Processor Resource/Systems Manager):''' Bare-metal hardware partitioning (LPARs) built directly into the machine firmware.
* '''CMOS Transition (1994):''' Transitioned mainframe hardware from hot, water-cooled Bipolar/ECL chips to air-cooled CMOS microprocessors (IBM 9672 generation).

=== 5. z/Architecture (2000–Present) ===
* Expanded GPRs, FPRs, Control Registers, and addressing from 32 bits to '''full 64 bits'''.
* Trimodal execution (24-bit, 31-bit, and 64-bit applications running concurrently on the same OS kernel).
* Replaced the 8-byte PSW with the '''16-byte Extended PSW''' (`LPSWE`).
* Continual modern extensions: SIMD/Vector facilities, transactional memory (TX), hardware cryptographic acceleration (CPACF), on-chip AI neural processing units (Telum / z16).

----

== Taxonomy: Architecture vs. Hardware Brands vs. OS Triples ==

Because the mainframe ecosystem separates the ISA specification from marketing product lines and software toolchains, the naming conventions can be confusing:

{| class="wikitable"
! Layer !! Name !! Description / Examples
|-
| '''ISA Specification''' || <code>System/360</code>, <code>System/370</code>, <code>370-XA</code>, <code>ESA/370</code>, <code>ESA/390</code>, <code>z/Architecture</code> || The formal architectural standard (defined in the Principles of Operation manuals).
|-
| '''Hardware Brands (Historical)''' || <code>S/360</code>, <code>S/370</code>, <code>303x</code>, <code>308x</code>, <code>3090</code>, <code>ES/9000</code>, <code>System/390</code> (S/390), <code>IBM 9672</code>, <code>Multiprise</code> || Marketing and physical server family names.
|-
| '''Hardware Brands (Modern)''' || <code>eServer zSeries</code> (z900/z990), <code>System z</code> (z9/z10), <code>zEnterprise</code> (z196/zEC12), <code>IBM z13, z14, z15, z16</code> || Physical servers implementing modern 64-bit z/Architecture.
|-
| '''GCC / Linux Target (31-bit)''' || <code>s390-ibm-linux-gnu</code> (or simply <code>s390</code>) || Linux running in legacy 31-bit ESA/390 compatibility mode.
|-
| '''GCC / Linux Target (64-bit)''' || <code>s390x-ibm-linux-gnu</code> (or simply <code>s390x</code>) || Standard modern 64-bit Linux on z/Architecture.
|}

----

== The "System-Driven" Philosophy: Multi-Engine Co-Processors ==

Commodity PC architectures typically utilize symmetric multiprocessors where every CPU core runs operating system kernel code. In contrast, an IBM Mainframe processor drawer is an '''asymmetric, multi-engine computing complex'''.

All physical cores on a modern mainframe processor chip (e.g. z15, z16) share identical silicon, but are designated at initialization (or dynamically reassigned) to specialized operational roles:

=== 1. CP (Central Processor) ===
The standard general-purpose engine. Runs standard operating systems (z/OS, Linux, z/VSE, z/TPF) and executes all standard user/kernel workloads.

=== 2. SAP (System Assist Processor) ===
Dedicated I/O offload processors. 
* The operating system does '''not''' service raw hardware device interrupts or manage physical bus transfers directly.
* When an OS issues an I/O request (`SSCH` - Start Subchannel), the CPU immediately offloads the entire Channel Program (a list of CCWs) to the **SAP**.
* The SAP independently manages the data movement through the **Channel Subsystem (CSS)**, verifies checksums, arbitrates fiber/PCIe links, and only interrupts the OS CP when the transaction is completed.

=== 3. IFL (Integrated Facility for Linux) ===
Dedicated cores reserved exclusively for running Linux and KVM on IBM zSystems. IFLs are physically identical to CPs, but are restricted from running proprietary IBM z/OS workloads, allowing them to be sold and licensed at substantially lower software-maintenance price tiers.

=== 4. zIIP (Integrated Information Processor) ===
Specialized coprocessors designed to run compute-intensive enterprise background workloads (such as Java JVM execution, XML parsing, DB2 indexing, and IPSec encryption) to prevent general CPs from being bogged down.

=== 5. ICF (Internal Coupling Facility) ===
Dedicated hardware engines used for '''Parallel Sysplex''' clustering. They run specialized coupling-control code to synchronize memory caches, distributed locks, and buffer pools across multiple physically separate mainframe servers at microsecond latency.

=== 6. Hardware Accelerators ===
* '''CPACF (Central Processor Assist for Cryptographic Functions):''' On-core hardware crypto accelerators (AES, SHA-1/2/3, DES) executed via dedicated unprivileged machine instructions (e.g., `KM`, `KMC`, `KLMD`).
* '''CEX (Crypto Express):''' Tamper-proof HSM PCIe adapters handling high-security RSA, ECC, and quantum-safe key generation.
* '''On-Chip AI Accelerator:''' Built-in neural network processing pipelines integrated on each core chip (introduced in Telum / z16) providing real-time AI inference during transaction processing.

----

== Core Architectural Concepts ==

=== PR/SM and Hardware Virtualization ===
There is effectively no "bare-metal OS execution" in modern mainframe computing. Every system runs on top of '''PR/SM (Processor Resource/Systems Manager)''', a Type-0 hypervisor implemented in firmware/millicode. PR/SM manages LPARs (Logical Partitions) with guaranteed hardware isolation, dynamic resource allocation, and zero hypervisor latency.

=== Storage Keys and Complex Memory Protection ===
In addition to page-table (DAT) permissions, z/Architecture features a separate, hardware-enforced memory security layer known as '''Storage Keys''':
* Every 4 KiB physical memory frame has an associated 7-bit hardware storage key in the memory controller:
** Bits 0–3: '''Access-Control Key (0–15)'''
** Bit 4: '''Fetch-Protection Bit''' (1 = read access also requires key match)
** Bit 5: '''Reference Bit''' (Set whenever the page is accessed)
** Bit 6: '''Change Bit''' (Set whenever the page is written)
* If the PSW Key does not match the page's storage key (and the PSW Key is not 0), any memory load or store instantly triggers a hardware protection exception (`PGM_PROTECTION`), independent of virtual memory page tables.

=== Channel Subsystem (CSS) and CCWs ===
Mainframes do not use memory-mapped I/O (MMIO) or register-based polling. Instead, all I/O is managed via '''Channel Command Words (CCWs)''':
# The OS builds a linked list of I/O commands (CCWs) in memory.
# The OS issues <code>SSCH</code> (''Start Subchannel'') passing the subchannel ID and ORB (Operation Request Block).
# The '''SAP''' processor processes the entire chain independently.
# Upon completion, an I/O interruption is posted to the OS containing the Final Status (IRB).

----

== Sixty Years of Backward Compatibility ==

A fundamental hallmark of the mainframe architecture is complete, unbroken binary compatibility:
* A non-privileged application binary compiled for an original '''IBM System/360 in 1964''' can run without recompilation or emulation on a modern '''IBM z16 (2022)''' running in 24-bit compatibility mode.
* The instruction set architecture has preserved opcodes, condition codes, arithmetic behaviors, and lowcore conventions while scaling from 16 KiB magnetic core memory to multi-terabyte NUMA complexes.

----

== See Also ==
* [[Program Status Word]]
* [[DAT]]
* [[STSI and ECAG]]
* [https://www.ibm.com/support/pages/zarchitecture-principles-operation IBM z/Architecture Principles of Operation (SA22-7832)]

[[Category:S390]]
[[Category:History]]
````

<a id="w005086"></a>

## W005086 — User:Anotheridiot

Bron-ID: W005086
Namespace: User (2)
Revisie: 30939 | Datum: 2026-09-04T14:39:01Z
Bron: https://wiki.osdev.org/User:Anotheridiot
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAnotheridiot&oldid=30939

````text
Hello! You already know what I am doing here lol. Anyways I work on IridiumKern (https://GitHub.com/IridiumKern) that's about it. Oh and I write some pages about schedulers and stuff. But other than that just check my user page I am writing a LOT of stuff right now.

About me:

- I love C and hate OOP (I will not die on this hill, but someone will)

- I love security (I am trying to design my kernel to be secure but its obviously a WIP and such, though it does have a CSPRNG, using SHA512 as a simple primitive)

- I avoid AI usage (I used to use it though I no longer do)

- I am a freshman in high school! Hi from Ohio!
````

<a id="w005087"></a>

## W005087 — User:Anotheridiot/Threading

Bron-ID: W005087
Namespace: User (2)
Revisie: 30941 | Datum: 2026-09-04T15:01:54Z
Bron: https://wiki.osdev.org/User:Anotheridiot/Threading
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAnotheridiot%2FThreading&oldid=30941

````text
This serves as my writing place for my thoughts on how an OS should implement threading (though it is opinionated to my own preferences and designs obviously).

== How my threading design works ==
A process is spawned with one thread, and a 16KiB stack, the stack size can be increased or decreased by the process as wanted, though most things should be put onto the heap. Regardless a thread can be spawned by tspawn, and exits by texit. tspawn spawns a thread taking in the pointer for its entry code and allocates a 16KiB stack, texit exits the currently running thread (cannot be thread 0, that must call exit). All threads in a process share the PID and have a different TID (thread ID). They also share a CR3 for speed. A process is ran, then it runs once, then the next task runs, and it will loop over and over again going from TID 0 to TID 1 and so on in each task. For example:

- Process 1 runs with Thread 0

- Process 2 runs with Thead 0

- Process 1 runs with Thread 1

- Process 2 runs with Thread 2

- ...

- Process 1 runs its final thread in the list

- Process 2 runs its final thread in the list

- Process 1 runs thread 0.

This is not a perfectly optimized setup though it does stop an application from creating a large amount of threads and just taking over execution.
````

<a id="w005088"></a>

## W005088 — User:Anotheridiot/Kernel Security

Bron-ID: W005088
Namespace: User (2)
Revisie: 31022 | Datum: 2026-10-14T12:25:37Z
Bron: https://wiki.osdev.org/User:Anotheridiot/Kernel_Security
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAnotheridiot%2FKernel_Security&oldid=31022

````text
[[Category:Security]]
{{Stub}}

This is my page dedicated to the development of security features within a kernel. Many features are quite needed for any basic security setup (for example, [[Stack Smashing Protector|SSP]]). Personally I choose to pass the following flags in my makefile for security related things

<syntaxhighlight lang="GNUmakefile">
# NOTE: Some of these flags may only exist on Clang.
# SSP exists on both GCC and Clang, the rest is unclear.
override CFLAGS += \
	-fstack-protector-all \
	-fsanitize=undefined \
	-fsanitize-minimal-runtime \
	-fsanitize=bounds \
	-fsanitize=object-size \
	-fsanitize=alignment \
	-fsanitize=bool \
	-fsanitize=enum \
	-fsanitize=shift \
	-fsanitize=shift-base \
	-fsanitize=shift-exponent \
	-fsanitize=integer-divide-by-zero \
	-fsanitize=unreachable \
	-fsanitize=return \
	-fsanitize=nonnull-attribute \
	-fsanitize=null \
	-fsanitize=implicit-integer-truncation \
	-fsanitize=implicit-integer-sign-change
</syntaxhighlight>

The three most important lines are the first three in these flags, SSP (stack protector) which is used to stop stack overflow attacks taking over the kernel, the undefined behavior sanitizer and of course then there is the fact its told to use the minimal runtime, which makes implementation considerably easier.

The reason to scan for Undefined Behavior and patch it is because if Behavior is undefined it stops C from acting normally and introduces possible bugs and exploits like out of bounds memory writes and reads.

The rest are mostly enabling extra checks in -fsanitize. Though of course some may be redundent depending on the build of clang used.

== The debugging aspect ==

UBSan other than just security, helps stop UB bugs which makes the kernel act more predictable in stock/non debugging builds.
````

<a id="w005089"></a>

## W005089 — Talk:JLoc

Bron-ID: W005089
Namespace: Talk (1)
Revisie: 31015 | Datum: 2026-10-12T11:56:53Z
Bron: https://wiki.osdev.org/Talk:JLoc
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AJLoc&oldid=31015

````text

````

<a id="w005090"></a>

## W005090 — File:Treonos logo.png

Bron-ID: W005090
Namespace: File (6)
Revisie: 30957 | Datum: 2026-09-18T06:35:41Z
Bron: https://wiki.osdev.org/File:Treonos_logo.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ATreonos_logo.png&oldid=30957

````text
Logo for my OS.
````

<a id="w005091"></a>

## W005091 — User:Anotheridiot/Iridium

Bron-ID: W005091
Namespace: User (2)
Revisie: 30997 | Datum: 2026-09-30T17:36:06Z
Bron: https://wiki.osdev.org/User:Anotheridiot/Iridium
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAnotheridiot%2FIridium&oldid=30997

````text
{{stub}}

{{Infobox Project
|name=Iridium

|image=N/A
|caption=Yet to take a photo

|developers=[[User:Anotheridiot|apixeldev]]

|kerneltype=Monolithic
|platform=x86_64-v1
|taskmodel=Preemptive multitasking
|language=C99, assembly

|website=[https://stackunderflow.org stackunderflow.org]
|license=GPLv3
|github=iridiumkern/kernel
}}

Iridium is my custom hobbyist OS written from scratch (outside of using limine). It has a CSPRNG, VMM, and PMM. The CSPRNG is by far one of the more impressive parts of my system, it is only 80 lines of code, however it does rely on an outside hash (SHA512) defined in another file and header to function. All inputs and outputs are wiped from the stack the moment they are no longer needed. The SHA512 implementation was taken from a public domain source and stays under public domain, however the rest is under GPLv3 (except for limine.h which stays under the same license).

The kernel is quite limited in userspace capabilities, while it does technically have a userspace, it is not very effective at its job, as its a single process that can only really print a string and dump a random CSPRNG output to the screen, and the program is a raw binary. Though the limited userspace, it is still under heavy development.
````

<a id="w005092"></a>

## W005092 — User:Softwarewrighter

Bron-ID: W005092
Namespace: User (2)
Revisie: 30973 | Datum: 2026-09-26T22:00:25Z
Bron: https://wiki.osdev.org/User:Softwarewrighter
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASoftwarewrighter&oldid=30973

````text
[https://blog.softwarewrighter.com/about/ About me (Mike Wright)]
````

<a id="w005093"></a>

## W005093 — Template:ProposedDeletion

Bron-ID: W005093
Namespace: Template (10)
Revisie: 30989 | Datum: 2026-09-28T14:52:35Z
Bron: https://wiki.osdev.org/Template:ProposedDeletion
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AProposedDeletion&oldid=30989

````text
<div style="border: 1px solid #cfcfbf; margin: 25px auto 10px auto; background-color: #f0f0ff; text-align: center; padding: 10px; max-width: 80%;">
[[File:Stub.png|alt=Proposed for deletion|link=]] This page or section is proposed for deletion! You can help the wiki by talking about it on its [{{fullurl:{{TALKPAGENAME}}|action=edit}} talk page].
</div><includeonly>[[Category:ProposedDeletion]]</includeonly>
````

<a id="w005094"></a>

## W005094 — Talk:OSKit

Bron-ID: W005094
Namespace: Talk (1)
Revisie: 30991 | Datum: 2026-09-28T14:57:45Z
Bron: https://wiki.osdev.org/Talk:OSKit
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AOSKit&oldid=30991

````text
The reason why I propsed the deletion of this page is because OSKit is legacy and has no actual modern usage. --[[User:Anotheridiot|Anotheridiot]] 14:57, 28 September 2026 (UTC)
````

<a id="w005095"></a>

## W005095 — OS Specific LLVM Toolchain

Bron-ID: W005095
Namespace: Main (0)
Revisie: 31038 | Datum: 2026-10-23T12:18:59Z
Bron: https://wiki.osdev.org/OS_Specific_LLVM_Toolchain
Revisie-URL: https://wiki.osdev.org/index.php?title=OS_Specific_LLVM_Toolchain&oldid=31038

````text
{{Rating|3}}
{{tone}}

Until now you have likely been using a '''cross-compiler''', which is very useful at the beginning as it allows you to easily generate code for a bare metal environment. However, the disadvantage of this approach is that the compiler does not know your operating system.

For example, when using Clang, one would use the triple <code>x86_64-unknown-none</code>. If you are, then you may prefer to be able to use <code>x86_64-unknown-myos</code> instead for many reasons, though a sysroot can replicate many of them.

=== Step 1: Clone LLVM ===
In order to edit LLVM components, you first need to clone [https://github.com/llvm/llvm-project the LLVM project git repository].

=== Step 2: Understanding ===
What we are going to do is making clang, lld and other toolchain components able to generate binaries for our operating system. To know what system it should target, LLVM uses '''triples''', which are simple strings representing a given system. As said above, the triple we want to add is <code>arch-unknown-myos</code>where <code>arch</code>is the computer architecture (e.g. x86_64) and <code>myos</code>is the name of your OS.

For example, type <code>clang --version</code>in your terminal. If you are on a GNU/Linux system, you would see a similar output to the following:<syntaxhighlight lang="text">
<distro> clang version <version>
Target: x86_64-pc-linux-gnu
Thread model: posix
InstalledDir: <directory>
</syntaxhighlight>As you can see, the default target triple is <code>x86_64-pc-linux-gnu</code>. Which means that the compiler would generate binaries for a GNU/Linux system on the x86_64 architecture.

<code>pc</code>is not very important, it is just an additional information.

=== Step 3: <code>TripleName.def</code> ===
The first thing we will do is defining our operating system triple. To do so, we must modify <code>llvm/include/llvm/TargetParse/TripleName.def</code>.

Add the following:<syntaxhighlight lang="c++">
// other calls to TRIPLE_OS()
// ...
TRIPLE_OS(MyOS, "myos", "MyOS")
</syntaxhighlight>

=== Step 4: <code>Triple.h</code> ===
Next, add your OS to <code>llvm::Triple::OSType</code>, which is defined in <code>llvm/include/llvm/TargetParser/Triple.h</code>.<syntaxhighlight lang="c++">
// ...
enum OSType {
    // ...
    MyOS
    LastOSType = MyOS
}
</syntaxhighlight>

=== Step 5: OSTargets.h ===
Now we can add our own target. We can do this by adding a class which inherits from <code>clang::targets::OSTargetInfo</code> to <code>clang/lib/Basic/Targets/OSTargets.h</code>.<syntaxhighlight lang="c++">
template<typename Target>
class LLVM_LIBRARY_VISIBILITY MyosTargetInfo : public OSTargetInfo<Target> {
protected:
    void getOSDefines(const LangOptions &Opts, const llvm::Triple &Triple,
        MacroBuilder &Builder) const override {
        Builder.defineMacro("__myos__");
        Builder.defineMacro("__ELF__"); // depends on your OS executables
        if (Opts.CPlusPlus)
            Builder.defineMacro("_GNU_SOURCE");
        DefineStd(Builder, "unix", Opts);
    }
    
public:
    MyosTargetInfo(const llvm::Triple &Triple, const TargetOptions &Opts)
        : OSTargetInfo<Target>(Triple, Opts) {
        this->WIntType = TargetInfo::UnsignedInt;        
    }
};
</syntaxhighlight>

=== Step 6: <code>Targets.cpp</code> ===
Let's handle the new target. In <code>clang/lib/Basic/Targets.cpp</code>, add in the big switch loop:<syntaxhighlight lang="c++">
// ...
// previous cases
case llvm::triple::MyOS:
    return std::make_unique<MyOSTargetInfo<X86_64TargetInfo>>(Triple, Opts);
// next cases
default:
    return std::make_unique<X86_64TargetInfo>(Triple, Opts);
</syntaxhighlight>Make sure to adapt the section and the code to your architecture.
[[Category:Tutorials]]
````

<a id="w005096"></a>

## W005096 — X86-64 Registers

Bron-ID: W005096
Namespace: Main (0)
Revisie: 31001 | Datum: 2026-10-02T17:05:06Z
Bron: https://wiki.osdev.org/X86-64_Registers
Revisie-URL: https://wiki.osdev.org/index.php?title=X86-64_Registers&oldid=31001
Doorverwijzing: CPU Registers x86-64

````text
#REDIRECT [[CPU_Registers_x86-64]]
````

<a id="w005097"></a>

## W005097 — User:Guac

Bron-ID: W005097
Namespace: User (2)
Revisie: 31006 | Datum: 2026-10-09T18:05:31Z
Bron: https://wiki.osdev.org/User:Guac
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGuac&oldid=31006

````text
hello! i'm guac. i'm not super knowledgeable on operating systems or low-level programming (that's why i'm here!) but i do like writing and fixing typos! you'll probably see me rewriting small chunks for clarity, grammatical accuracy, or consistency. love you, buh bye!
````

<a id="w005098"></a>

## W005098 — Lowcore and Prefixing

Bron-ID: W005098
Namespace: Main (0)
Revisie: 31008 | Datum: 2026-10-10T08:17:44Z
Bron: https://wiki.osdev.org/Lowcore_and_Prefixing
Revisie-URL: https://wiki.osdev.org/index.php?title=Lowcore_and_Prefixing&oldid=31008

````text
The '''Lowcore''' (also called the '''prefix area''' or '''prefix page''') is the 8 KiB block of storage at the bottom of each CPU's real address space that the hardware itself uses for interruption handling, IPL, and status storing. The '''Prefixing''' mechanism is what lets every CPU in a multiprocessor system have its own private Lowcore, even though all of them architecturally "see" it at real address <code>0x0000</code>.

Understanding both is a prerequisite for handling any interruption on z/Architecture (s390x), as the CPU stores and fetches [[Program_Status_Word|PSWs]] from fixed Lowcore locations.

{| class="wikitable" style="float:right; margin-left:1em;"
! colspan="2" | At a glance
|-
| Size (z/Architecture) || 8 KiB (2 × 4 KiB frames)
|-
| Alignment || 8 KiB, in absolute storage
|-
| Real address || <code>0x0000</code>–<code>0x1FFF</code>
|-
| Set with || <code>SPX</code> / <code>SIGP</code> (Set Prefix)
|-
| Read with || <code>STPX</code>
|-
| Size (ESA/390) || 4 KiB
|}

'''Note:''' This page describes the architectural Lowcore as defined by the IBM ''z/Architecture Principles of Operation'' (SA22-7832, tested against the z17 edition), plus a short overview of how the ZXFoundation kernel uses the software-defined portion. It is '''NOT''' the <code>IHAPSA</code> (Prefixed Save Area) mapping used by z/OS. Field names, offsets, and software-use areas '''will not match''' the z/OS macro, and should not be assumed to be interchangeable.

== Terminology ==

{| class="wikitable"
! Term !! Meaning
|-
| '''Absolute address''' || An address into physical main storage, after prefixing has been applied.
|-
| '''Real address''' || The address seen by the program with DAT off (or the result of DAT), ''before'' prefixing is applied.
|-
| '''Prefix register''' || A per-CPU register that holds the absolute address of that CPU's Lowcore.
|-
| '''Lowcore / Prefix area''' || The 8 KiB region designated by the prefix register.
|-
| '''Absolute zero page''' || Absolute storage <code>0x0000</code>–<code>0x1FFF</code>. Only reachable through the prefix swap (see below) or with absolute-addressing instructions.
|}

== The Prefixing Mechanism ==

=== How it works ===

Prefixing is a transformation applied by the hardware to a ''real'' address to produce an ''absolute'' address. Let <code>P</code> be the (8 KiB aligned) value in the prefix register. For a real address <code>A</code>:

{| class="wikitable"
! Real address range !! Resulting absolute address
|-
| <code>0x0000</code> ≤ A < <code>0x2000</code> || <code>A + P</code>
|-
| <code>P</code> ≤ A < <code>P + 0x2000</code> || <code>A − P</code>
|-
| Anything else || <code>A</code> (unchanged)
|}

In other words, the two 8 KiB areas are '''swapped''': a CPU whose prefix is non-zero sees its own Lowcore at real <code>0x0</code>, and sees the original absolute zero page at real address <code>P</code>. Everything else is identity-mapped.

<pre>
        Real address space              Absolute (physical) storage
      (as seen by CPU n, P = 0x40000)
     +-----------------------+        +-----------------------+
0x0  |  CPU n Lowcore (8K)   | ---->  |  P:  CPU n Lowcore    |
     +-----------------------+        +-----------------------+
     |                       |        |                       |
     |      unchanged        | ---->  |      unchanged        |
     |                       |        |                       |
P    |  Absolute zero page   | ---->  |  0x0: Absolute zero   |
     +-----------------------+        +-----------------------+
</pre>

=== Where it sits in the translation pipeline ===

Prefixing is applied '''after''' [[DAT]] and ''only'' to real addresses. The order is therefore:

# Logical (virtual) address → DAT (if PSW bit 5 is set) → '''real address'''
# Real address → '''prefixing''' → '''absolute address'''
# Absolute address → main storage

Consequences worth remembering:
* Interruption handling always operates on the real addresses <code>0x0000</code>–<code>0x1FFF</code>, so the hardware always lands in the ''current CPU's'' Lowcore, regardless of DAT state.
* Absolute addressing (e.g. in some diagnostic and dump paths) bypasses prefixing entirely.

=== The Prefix Register ===

* The prefix is an 8 KiB-aligned address; in z/Architecture it is held in bits 33–50 of the register, which limits the Lowcore location to the first 2 GiB of absolute storage.
* After a '''CPU reset''' (initial or clear) the prefix is zero, so every CPU starts out sharing absolute zero as its Lowcore. Software is expected to give each CPU its own area as early as possible.
* In ESA/390 mode the prefix area is 4 KiB; z/Architecture uses 8 KiB.

{| class="wikitable"
! Instruction !! Mnemonic !! Description
|-
| <code>B210</code> || <code>SPX</code> || Set Prefix on the ''current'' CPU (privileged).
|-
| <code>B211</code> || <code>STPX</code> || Store Prefix of the ''current'' CPU (privileged).
|-
| <code>AE</code> || <code>SIGP</code> || The ''Set Prefix'' order sets the prefix of a ''target'' CPU, which must be in the stopped state.
|}

=== Changing the prefix safely ===

Before issuing <code>SPX</code>, the new Lowcore page should already contain valid <code>New PSW</code> fields, since an interruption may arrive immediately after the swap. A recommended sequence for the boot CPU:

# Allocate an 8 KiB-aligned, 8 KiB block in absolute storage below the 2 GiB boundary.
# Zero it, then populate at least the New PSWs and any stack pointers the handlers will rely on.
# Issue <code>SPX</code>.
# From this point, real <code>0x0</code> refers to the new area. Absolute zero is now reachable only at real address <code>P</code>.

<syntaxhighlight lang="c">
#include <stdint.h>

static inline void set_prefix(uint32_t addr)
{
    __asm__ volatile ("spx %0" : : "Q" (addr) : "memory");
}

static inline uint32_t store_prefix(void)
{
    uint32_t addr;
    __asm__ volatile ("stpx %0" : "=Q" (addr) : : "memory");
    return addr;
}
</syntaxhighlight>

== Lowcore Layout ==

Offsets below are in '''hexadecimal''' (and decimal), measured from the start of the Lowcore. All multi-byte fields are '''big-endian'''. Fields marked ''Arch'' are defined by the Principles of Operation and written by hardware. Fields marked ''SW'' are in storage the architecture leaves for the operating system, and are '''implementation-defined'''.

=== Architecturally defined fields ===

{| class="wikitable"
! Offset (hex) !! Offset (dec) !! Size !! Name !! Type
|-
| <code>0x0000</code> || 0 || 8 || IPL PSW (ESA/390 or short PSW view) || Arch
|-
| <code>0x0008</code> || 8 || 8 || IPL CCW 1 || Arch
|-
| <code>0x0010</code> || 16 || 8 || IPL CCW 2 (also loader size / sysparm block address) || Arch
|-
| <code>0x0080</code> || 128 || 4 || External-interruption parameter || Arch
|-
| <code>0x0084</code> || 132 || 2 || CPU address || Arch
|-
| <code>0x0086</code> || 134 || 2 || External-interruption code || Arch
|-
| <code>0x0088</code> || 136 || 4 || SVC interruption identification (ILC, code) || Arch
|-
| <code>0x008C</code> || 140 || 4 || Program-interruption identification (ILC, code) || Arch
|-
| <code>0x0090</code> || 144 || 4 || Exception code (vector / data exception) || Arch
|-
| <code>0x0094</code> || 148 || 2 || Monitor-class number || Arch
|-
| <code>0x0096</code> || 150 || 2 || PER code || Arch
|-
| <code>0x0098</code> || 152 || 8 || PER address || Arch
|-
| <code>0x00A0</code> || 160 || 1 || Exception access identification (EAID) || Arch
|-
| <code>0x00A1</code> || 161 || 1 || PER access identification || Arch
|-
| <code>0x00A2</code> || 162 || 1 || Operation access identification || Arch
|-
| <code>0x00A3</code> || 163 || 1 || Architectural-mode identification || Arch
|-
| <code>0x00A8</code> || 168 || 8 || Translation-exception identification (TEID) || Arch
|-
| <code>0x00B0</code> || 176 || 8 || Monitor code || Arch
|-
| <code>0x00B8</code> || 184 || 12 || I/O-interruption code || Arch
|-
| <code>0x00C8</code> || 200 || 4 || Facility list (<code>STFL</code>) || Arch
|-
| <code>0x00E8</code> || 232 || 8 || Machine-check interruption code || Arch
|-
| <code>0x00F4</code> || 244 || 4 || External-damage code || Arch
|-
| <code>0x00F8</code> || 248 || 8 || Failing-storage address || Arch
|-
| <code>0x0100</code> || 256 || 8 || Monitor-counter array origin || Arch
|-
| <code>0x0108</code> || 264 || 4 || Monitor-counter array size || Arch
|-
| <code>0x010C</code> || 268 || 4 || Monitor-exception count || Arch
|-
| <code>0x0110</code> || 272 || 8 || Breaking-event address || Arch
|}

=== Interruption PSW slots ===

Each interruption class has a 16-byte Old PSW (written by hardware) and a 16-byte New PSW (loaded by hardware). See [[Program_Status_Word|PSW]] for the format.

{| class="wikitable"
! Class !! Old PSW !! New PSW
|-
| Restart || <code>0x0120</code> || <code>0x01A0</code>
|-
| External || <code>0x0130</code> || <code>0x01B0</code>
|-
| Supervisor Call || <code>0x0140</code> || <code>0x01C0</code>
|-
| Program || <code>0x0150</code> || <code>0x01D0</code>
|-
| Machine Check || <code>0x0160</code> || <code>0x01E0</code>
|-
| I/O || <code>0x0170</code> || <code>0x01F0</code>
|}

<syntaxhighlight lang="cpp">
constexpr uint64_t LC_EXT_OLD_PSW  = 0x0130;
constexpr uint64_t LC_SVC_OLD_PSW  = 0x0140;
constexpr uint64_t LC_PGM_OLD_PSW  = 0x0150;
constexpr uint64_t LC_MCCK_OLD_PSW = 0x0160;
constexpr uint64_t LC_IO_OLD_PSW   = 0x0170;
</syntaxhighlight>

=== Status-save areas (stored by hardware on request) ===

These areas are written by the CPU on <code>STORE STATUS</code> and by the machine-check facility. They are architecturally defined, and are useful for building dump and crash-recovery paths.

{| class="wikitable"
! Offset (hex) !! Size !! Name !! Type
|-
| <code>0x11B0</code> || 8 || Machine-check extended save area designation || Arch
|-
| <code>0x1200</code> || 128 || Floating-point registers (16 × 8) || Arch
|-
| <code>0x1280</code> || 128 || General registers (16 × 8) || Arch
|-
| <code>0x1300</code> || 16 || PSW save area (overlaps the fixed logout area) || Arch
|-
| <code>0x1318</code> || 4 || Prefix register || Arch
|-
| <code>0x131C</code> || 4 || Floating-point control register || Arch
|-
| <code>0x1324</code> || 4 || TOD programmable register || Arch
|-
| <code>0x1328</code> || 8 || CPU timer || Arch
|-
| <code>0x1330</code> || 8 || Clock comparator || Arch
|-
| <code>0x1338</code> || 8 || Last breaking-event address || Arch
|-
| <code>0x1340</code> || 64 || Access registers (16 × 4) || Arch
|-
| <code>0x1380</code> || 128 || Control registers (16 × 8) || Arch
|-
| <code>0x1500</code> || 8 || Crypto-counter designation || Arch
|-
| <code>0x1508</code> || 8 || Adapter-interruption-controls (AIC) designation || Arch
|-
| <code>0x1800</code> || 256 || Transaction diagnostic block (TDB) || Arch
|}

The Lowcore therefore ''spans'' roughly 6400 bytes (<code>0x1900</code>) of defined content, even though it always occupies a full 8 KiB prefix area.

=== Software-use areas ===

The remaining space is available to the operating system. The architecture assigns it no meaning, and hardware never touches it. ZXFoundation uses it for the following categories, '''the exact layout of which is considered kernel-internal and may change between releases''':

{| class="wikitable"
! Region !! Purpose
|-
| <code>0x0200</code>–<code>0x027F</code> || Synchronous-entry register save area, stack protection value, and temporary return PSW storage.
|-
| <code>0x0280</code>–<code>0x02AF</code> || Restart save slot, per-CPU back-pointer, and return PSWs.
|-
| <code>0x02B0</code>–<code>0x032F</code> || Last-break copy, entry/exit accounting timers, interruption clock snapshots, and a clock-comparator shadow (Linux-style CPU-time accounting model).
|-
| <code>0x0330</code>–<code>0x036F</code> || Control-register snapshots, current execution-domain pointer, and the per-context stack pointers (kernel, asynchronous, DAT-off, restart, machine-check).
|-
| <code>0x0370</code>–<code>0x0397</code> || Restart-interruption hand-off (callback, cookie, source, flags) and address-space origin values.
|-
| <code>0x0398</code>–<code>0x03B7</code> || Current context identifier, logical CPU identity, and spinlock bookkeeping values.
|-
| <code>0x0400</code> || Per-CPU data pointer. ''Fixed offset, relied on by early assembly entry code.''
|-
| <code>0x1400</code>–<code>0x14FF</code> || 256-byte kernel crash-record area (ZXFoundation specific).
|}

'''Note:''' The Linux-derived fields use the same ''ideas'' as Linux/s390 (stack-per-context, entry/exit timers, saved return PSW), but offsets and widths are '''not''' guaranteed to match <code>struct lowcore</code> in the Linux kernel.

== Practical Considerations ==

=== Alignment and allocation ===
The prefix must be 8 KiB aligned. Allocate Lowcores from absolute storage below 2 GiB, as the prefix register cannot designate anything higher.

=== The AP (secondary CPU) bring-up ===
Application processors start with a prefix of zero. A typical flow from the boot CPU is:

# Allocate and initialize the AP's Lowcore at a unique 8 KiB-aligned location, filling in at least the restart New PSW and a restart stack.
# Issue <code>SIGP</code> ''Set Prefix'' to the AP (which must be stopped).
# Issue <code>SIGP</code> ''Restart''. The AP loads the Restart New PSW from ''its own'' Lowcore, and starts running with DAT off.
# The AP code enables DAT and joins the kernel.

Since the AP's restart PSW runs with DAT off, its entry point and initial stack must be given as '''real/physical''' addresses.

=== Accessing another CPU's Lowcore ===
The BSP cannot rely on real <code>0x0</code> to look at another CPU's Lowcore, because that address resolves to its ''own''. You need the other CPU's prefix value, which is then accessed through a normal mapping of that physical page (e.g. a direct map).

=== Accessing absolute zero ===
Once the prefix is non-zero, absolute storage <code>0x0</code>–<code>0x1FFF</code> is visible at real address <code>P</code>. This is relevant for dump tooling, and for retrieving data left there by the IPL process.

=== Common pitfalls ===
* '''Interrupts with an uninitialised New PSW''': if an interruption occurs while the relevant New PSW is zero, the CPU will typically enter a loop of program-checks or load a disabled-wait PSW.
* '''Missing New PSW setup before <code>SPX</code>''': the new Lowcore must be fully prepared first.
* '''Confusing real and absolute addresses''' when building page tables or storing a "physical" Lowcore pointer: always record the absolute address.
* '''Sharing a Lowcore between CPUs''': this is possible architecturally, but defeats the purpose of per-CPU stacks and save areas, and will corrupt state on the first simultaneous interruption.
* '''Assuming ESA/390 offsets''': the 8-byte PSW slots at <code>0x18</code>–<code>0x78</code> are not used for interruptions in z/Architecture mode.

== See Also ==
* [[Program_Status_Word]]
* [[DAT]]
* [[Interrupts]]
* [https://www.ibm.com/support/pages/zarchitecture-principles-operation IBM z/Architecture Principles of Operation (SA22-7832)]

[[Category:S390]]
````

<a id="w005099"></a>

## W005099 — DMAR

Bron-ID: W005099
Namespace: Main (0)
Revisie: 31012 | Datum: 2026-10-11T16:55:17Z
Bron: https://wiki.osdev.org/DMAR
Revisie-URL: https://wiki.osdev.org/index.php?title=DMAR&oldid=31012
Doorverwijzing: Intel VT-d

````text
#REDIRECT [[Intel_VT-d]]
````

<a id="w005100"></a>

## W005100 — User:Skeletonz33

Bron-ID: W005100
Namespace: User (2)
Revisie: 31027 | Datum: 2026-10-18T21:08:31Z
Bron: https://wiki.osdev.org/User:Skeletonz33
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASkeletonz33&oldid=31027

````text
I'm a programmer who's interested in systems programming. I love retro hardware like floppy disks, old PCs, VGA, etc. And I love seeing how low-level systems work, like operating systems, file systems, memory allocators, etc.

I mainly write code in C and Assembly, using GCC and NASM. I strongly support GPLv2.

<code>const char* c = "life";</code> (in other words: '''C IS LIFE''')
````

<a id="w005101"></a>

## W005101 — Theia OS

Bron-ID: W005101
Namespace: Main (0)
Revisie: 31028 | Datum: 2026-10-18T22:31:11Z
Bron: https://wiki.osdev.org/Theia_OS
Revisie-URL: https://wiki.osdev.org/index.php?title=Theia_OS&oldid=31028

````text
Theia OS is an operating system under development. Its initial implementation targets the 32-bit x86 architecture and uses a custom bootloader for legacy BIOS systems. The project is written primarily in freestanding C and NASM assembly.

The project includes an interactive command-line console and foundational kernel components. Development is organized into incremental versions, with basic system functionality being implemented before higher-level facilities.

== Architecture ==

The initial implementation targets 32-bit x86 hardware and the legacy BIOS boot process.

The system consists of a custom bootloader, a kernel, and an interactive console. Assembly language is used for low-level startup operations, while freestanding C is used for kernel functionality.

The bootloader is developed as part of Theia OS rather than relying on an external bootloader.

== Boot process ==

The boot process begins with the BIOS loading the first boot stage. This stage loads subsequent boot components, which prepare the environment for transferring control to the kernel.

The bootloader and kernel are separate components of the project. Their implementation is developed incrementally alongside the rest of the system.

== Kernel development ==

Kernel development has proceeded through a series of foundational milestones, including timer functionality and memory management.

The early memory-management work includes memory-map handling, memory-region management, and allocator development. These components provide a foundation for further kernel development.

== Command-line interface ==

Theia OS includes an interactive command-line console that allows users to enter commands and receive responses from the running system.

Implemented commands include:

* <code>ECHO</code> — displays text.
* <code>CLS</code> — clears the console display.
* <code>VERSION</code> — displays version information.
* <code>SYSINFO</code> — displays system information.
* <code>MEMINFO</code> — displays memory information.
* <code>REBOOT</code> — restarts the system.
* <code>SHUTDOWN</code> — provides shutdown-related functionality.

The exact behavior of commands depends on the version being used.

== Development status ==

Version 0.15 is identified by the project as "Basic Closed" and marks the completion of its initial basic-functionality phase.

Filesystem support and application infrastructure are part of the planned development roadmap. They should be considered future work unless documented as implemented in a later release.

== See Also ==

=== External Links ===

* [https://github.com/Ale2323git/Theia-OS Theia OS source repository and project documentation]
````

<a id="w005102"></a>

## W005102 — File:DragonWare-dcp.png

Bron-ID: W005102
Namespace: File (6)
Revisie: 31033 | Datum: 2026-10-21T13:23:09Z
Bron: https://wiki.osdev.org/File:DragonWare-dcp.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ADragonWare-dcp.png&oldid=31033

````text

````
