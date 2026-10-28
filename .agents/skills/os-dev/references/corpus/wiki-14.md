# OSDev Wiki — volledige bronteksten, deel 14

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w004614"></a>

## W004614 — User:Neon/Recursive Paging

Bron-ID: W004614
Namespace: User (2)
Revisie: 26129 | Datum: 2021-06-12T06:17:12Z
Bron: https://wiki.osdev.org/User:Neon/Recursive_Paging
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANeon%2FRecursive_Paging&oldid=26129

````text
= Background =
Let us say that you have set up a simple page table for your kernel. You may even have it identity mapped for now. No biggie, you enable paging and everything appears to be working perfectly. You make a few changes and improve the paging code over time, but then the time comes when you need to create a new page table. Easy right? After all, you already did it right? Just allocate a new page frame, map it into the virtual address space, fill it with data, and now you have to update a page directory entry so it references the page table page frame number. But...lets say we have some virtual address and we need to figure out what needs to be mapped in and created. The virtual address itself is a bit field: it tells us the page directory entry, page table entry, and page offset. So we already know precisely what entries need to be worked with. And no matter what, we always have a virtual address to the root directory already -- all Process Control Blocks should have it. So we use the virtual address to reference the page directory entries. We find the address we are trying to map needs PDE[200]. Lets say we already created this page table...''somewhere.'' PDE[200] stores the page frame number of the page table we need. Except...paging is enabled, we need a ''virtual address to the page table''. Ah shoot, so every page table we create we need to somehow keep track of where all of them are in the virtual address space. And the number of page tables is not static: kernel space page tables are typically ''shared'', however... user mode page tables are not. So any time you create a new user process, you'll end up needing more page tables to map user space. So whatever structure the kernel uses to keep track of the virtual addresses of page tables, it needs to be dynamic. So, how do we keep track of all virtual addresses of the page tables? Keeping in mind paging is enabled, how can we map this dynamic structure when there is no dynamic structure already in place to use? That is, what can we use to manage the virtual addresses of the page tables we create...before being able to have the way to manage it? Its the constant chicken and egg scenario that likes to pop up all the time in memory management. Isn't it fun?

In 64 bit long mode, the virtual address space is so large that you can easily map all of physical memory into kernel space. In 32 bit protected mode however, we introduce a way to get around this chicken and egg scenario: a way so that you can always access, update, and edit paging structures as needed. It is the same method used by mainstream operating systems such as Linux and Windows when targeting 32 bit versions -- recursive paging.

Before getting into details, we need to cover some very important background material related to paging. Recursive paging is not hard to implement, however it can be hard to follow what is happening and why without a strong background. We recommend skimming these sections at least even if you feel strong with the topics.

== How page mapping works ==

Typically we are introduced to paging as a "Page Directory" and "Page Table". However, it does not matter what you would like to call them: in general operating systems, there is only Page Tables. We give different names because they are on different "Layers" of page tables. To keep things simple, let us just look at the lowest layer page tables.

A Page Table consists of 1024 Page Table Entries (PTE)'s. We will focus on 2 fields of the PTE: the Active bit and the Page Frame Number (PFN). When the Active bit is set, the PTE maps to a PFN. Typically you would have a Physical Memory Manager and grab a free PFN to use it. In order to understand how paging works, we need to know what a PTE represents.

A PTE represents...a ''page''. I know, I know...not very surprising. But it doesnt just represent a page...it represents a specific page. Assuming 1 Page Table, PTE[0] is the first page, PTE[1] is the second page ... PTE[1023] is the 1024'th page. If a page is 4096 bytes, this would mean the whole page table represents a 4MB region of the virtual address space. '''Let us go up one layer...''' Lets say we have 1024 page directory entries. If each page directory entry has the PFN of a page table, then PDE[0] is the first page table, PDE[1] is the next page table ... PDE[1023] is the 1024'th page table. Or, in other words, PDE[0] represents the first 4MB of the address space, PDE[1] is the next 4MB region of the address space ... '''PDE[1023] is the last 4MB region of the address space.''' If a page table represents 4MB of the virtual address space, then the whole page directory represents 1024*4MB = 4GB of the virtual address space. Another way of looking at it is that, we can pick any PTE[K]. This represents the 4MB region of the address space starting at (# of PTE's in a page table * size of a page)*K = (1024*4096)*K.

No matter the virtual address, the CPU always goes through the same translation process: look it up to find the PDE and use its PDE.PFN to find the PTE and use its PTE.PFN to find the page.

= Introduction =

So, how do we implement recursive paging?

<pre>
PDE [1023].PageFrameNumber = PageDirectory
</pre>

That is it. No really, it is that simple. Okay, sure, you have to allocate a PFN for the page directory and be able to access it. As such, this is typically done during early initialization when paging structures are being first created. You'll also of course need to still have a separate page table for mapping the current executing code so when you do enable paging it will work. So, typically you do this when enabling paging for the first time so the issue of updating paging structures doesn't exist yet. The actual recursive paging trick though is just that one line.

== Accessing the page directory ==

So...let's see if we can wrap our head around what that line actually does. To do that, we'll be the CPU for the moment. Since the focus here is for updating and modifying page tables, we'll use addresses for them...for now. The magic of this line is how it effects the ''address space''.

So we need to translate an address...let's say 0xfffff000. Note these address are not arbitrary: we'll explain why we are using them later. Okay, so us being the CPU, we need to find the page directory. This part is easy as we already have it in the Page Directory Base Register (CR3). This has the PFN for the page directory. So we go out and see the Page Directory Entries. 0xfffff000 is PDE[1023]. Okay, no biggie, nothing special so far. We get the PFN from PDE[1023] to find the page table.

This is where things get interesting. We set PDE[1023].PFN to be the page directory itself. But now during the translation process, we don't see it as a page directory: we just see it as another page table. In other words, we are looking at the page directory as a page table.

Let us continue... 0xfffff000 is PTE[1023]. So we go out and look at PTE[1023].PFN to get the Page it is mapped to.

Which is, again, the page directory. Except this time, we are just looking at it as a raw page.

So what does this mean? When we recursively set PDE[1023] it ''automagically'' maps 0xfffff000 to the page directory itself. 0xfffff000 maps to a page containing the contents of the page directory. We can edit, update, or modify the page directory by just accessing its PDE's at 0xfffff000.

But...why 0xfffff000?

Let's first consider that we picked ''PDE[1023]''. This represents the last 4MB region of the virtual address space. That is, PDE[1023] only ever maps 0xffc00000 - 0xffffffff. But then...we looked at the page directory as a page table and looked at PDE[1023] again...but this time as PTE[1023]. Recall that PTE[1023] represents the last 4K region of the address space. So we are looking at the last 4K of the last 4MB of the address space... which is 0xfffff000. The page directory is at this address because of that 1023.

== Accessing page tables ==

Let's first reiterate a few important things: we are only considering the last 4MB of the address space from 0xffc00000 - 0xffffffff. This is because we recursively mapped PDE[1023] which represents this region of the address space. We walked through an example of translating an address to find the page directory base address above. Recall that we ended up looking at the Page Directory itself as a Page Table while translating the address. So when we looked at the Page Directory as a Page Table, each PTE referred to some 4K page of the address space. But...keep in mind that when translating an address that uses PDE[1023] we would be looking at just the Page Directory itself as a page table. And each PTE references a 4K region of the address space. But...the PTE's here are PDE's as we are just looking at the Page Directory still... so those PTE references a 4K region of the address space...a page...containing the contents of a Page Table.

<pre>
0xfffff000 is PDE[1023] which is the last page table. We can work backward to find all of the page table virtual addresses:
0xffffe000 is PDE[1022] which is second to last page table.
0xffffd000 is PDE[1021]
...
0xfff00000 is PDE[768] this is typically the kernel page table for higher half kernels at 3GB
...
0xffc00000 is PDE[0]
</pre>

== Recursive mapping on other entries ==

All virtual addresses is based on which PDE you decide to recursively map. We mapped the 1023 entry as it is what we use in our operating system -- although it is also what Linux uses. A problem with recursive paging is that it allocates a range of virtual addresses for the page tables and page directory which of course effects the address space layout of your operating system design. So sometimes it may make sense to select a different entry to recursively map. For a real world example of using a different entry we can look at Windows.

Consider that each PDE represents a 4MB region of the address space. Select a 4MB region that you would like to reserve for page tables and that is the entry for you to recursively map. I.e. If you recursively map PDE[768].PageFrameNumber = Page Directory you would have effectively mapped 0xc0000000 - 0xC0400000 for paging structures - this is method used in Windows.

The method works the same, PTE[768] refers to the page directory which would be located at 0xc0000000+(768*4096) = 0xC0300000.

== Things to think about ==

No matter what, 4MB of the address space gets reserved for paging structures since each PDE represents 4MB and you need to recursively map an entry. The benefit however is that this greatly simplifies paging code as all paging structures will be at specific locations in the address space. However...this means if you need to update the paging structures of some process K you must first switch to the process K address space before you can update the paging structures.

So how does this solve that pesky chicken and egg scenario? We no longer need to manage or think about where paging structures are in the virtual address space: we always know where they are and can easily access them to perform updates.
````

<a id="w004615"></a>

## W004615 — User:Demindiro/RISC-V Notes

Bron-ID: W004615
Namespace: User (2)
Revisie: 26294 | Datum: 2021-08-12T19:28:17Z
Bron: https://wiki.osdev.org/User:Demindiro/RISC-V_Notes
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADemindiro%2FRISC-V_Notes&oldid=26294

````text
== Fixing "Fatal error: invalid -march= option" (AKA you're using the wrong assembler) ==

When trying to build the [[OS Specific Toolchain]] & trying to compile a program you may have encountered this error:

<pre>
Assembler messages:
Fatal error: invalid -march= option: `rv64imafdc'
</pre>

If you run GCC verbosely (i.e. <code>gcc -v ...</code>) you'll see that the wrong assembler is being used. While I still don't know
''why'' this happens, there is a simple fix:

<pre>
./configure
	...
	--with-as=$(PREFIX)/bin/$(TARGET)-as \
	--with-ld=$(PREFIX)/bin/$(TARGET)-ld \
	--with-build-time-tools=$(PREFIX)/bin \
	...
</pre>

I don't know how or why other people's toolchains work without these options and it'll probably remain a mystery to me,
but It Works(tm) now at least.


== RISC-V, PCI and setting valid BARs ==

In my hairloss-inducing adventure of getting PCI to work I've found I missed an important detail
that doesn't seem to be mentioned anywhere, so I figured I'd quickly document it here before I
forget about it.

When enumerating the PCI buses in QEMU you may notice that ''all'' BARs point to 0, for example:

<!-- Please add Rust syntax highlighting (pretty please? :3) -->

<pre>
[kernel/src/main.rs:396] &pci = PCIHeader {
    vendor_id: 6900,
    device_id: 4100,
    command: 0,
    status: 16,
    revision_id: 0,
    prog_if: 0,
    subclass: 0,
    class_code: 1,
    cache_line_size: 0,
    latency_timer: 0,
    header_type: 0,
    bist: 0,
    base_address: [
        1,
        0,
        0,
        0,
        12,
        0,
    ],
    cardbus_cis_pointer: 0,
    subsystem_vendor_id: 6900,
    subsystem_id: 8,
    expansion_rom_base_address: 0,
    capabilities_pointer: 152,
    _reserved_0: 0,
    _reserved_1: 0,
    _reserved_2: 0,
    interrupt_line: 0,
    interrupt_pin: 1,
    min_grant: 0,
    max_latency: 0,
}
</pre>

This means you need to set it yourself. You may naively think that you can set it to whatever you
want, but that is not the case. If you do it anyways, you'll either read garbage or you get a
load/store fault:

<code>Invalid access at addr 0x5400000, size 4, region '(null)', reason: rejected</code>

If you open the QEMU console and enter <code>info pci</code> it'll seem that everything is fine,
so what gives?

<pre>
  Bus  0, device   0, function 0:
    Host bridge: PCI device 1b36:0008
      PCI subsystem 1af4:1100
      id ""
  Bus  0, device   1, function 0:
    SCSI controller: PCI device 1af4:1004
      PCI subsystem 1af4:0008
      IRQ 0, pin A
      BAR0: I/O at 0xffffffffffffffff [0x003e].
      BAR1: 32 bit memory at 0x00000000 [0x00000fff].
      BAR4: 64 bit prefetchable memory at 0x05400000 [0x05403fff].
      id "disk0"
</pre>

If you enter <code>info mtree</code> and inspect the output you may notice something else too:

<pre>
--- snip ---
    0000000010100010-0000000010100017 (prio 0, i/o): fwcfg.dma
    0000000020000000-0000000021ffffff (prio 0, romd): virt.flash0
    0000000022000000-0000000023ffffff (prio 0, romd): virt.flash1
    0000000030000000-000000003fffffff (prio 0, i/o): alias pcie-ecam @pcie-mmcfg-mmio 0000000000000000-000000000fffffff
    0000000040000000-000000007fffffff (prio 0, i/o): alias pcie-mmio @gpex_mmio 0000000040000000-000000007fffffff
    0000000080000000-000000008fffffff (prio 0, ram): riscv_virt_board.ram
    0000000400000000-00000007ffffffff (prio 0, i/o): alias pcie-mmio-high @gpex_mmio 0000000400000000-00000007ffffffff

address-space: I/O
  0000000000000000-000000000000ffff (prio 0, i/o): io

address-space: cpu-memory-0
  0000000000000000-ffffffffffffffff (prio 0, i/o): system
    0000000000001000-000000000000ffff (prio 0, rom): riscv_virt_board.mrom
    0000000000100000-0000000000100fff (prio 0, i/o): riscv.sifive.test
--- snip ---
memory-region: pcie-mmcfg-mmio
  0000000000000000-000000001fffffff (prio 0, i/o): pcie-mmcfg-mmio

memory-region: gpex_mmio
  0000000000000000-ffffffffffffffff (prio 0, i/o): gpex_mmio
    0000000000000000-0000000000000fff (prio 1, i/o): virtio-scsi-pci-msix
      0000000000000000-000000000000003f (prio 0, i/o): msix-table
      0000000000000800-0000000000000807 (prio 0, i/o): msix-pba
    0000000005400000-0000000005403fff (prio 1, i/o): virtio-pci
      0000000005400000-0000000005400fff (prio 0, i/o): virtio-pci-common-virtio-scsi
      0000000005401000-0000000005401fff (prio 0, i/o): virtio-pci-isr-virtio-scsi
      0000000005402000-0000000005402fff (prio 0, i/o): virtio-pci-device-virtio-scsi
      0000000005403000-0000000005403fff (prio 0, i/o): virtio-pci-notify-virtio-scsi

memory-region: system
  0000000000000000-ffffffffffffffff (prio 0, i/o): system
    0000000000001000-000000000000ffff (prio 0, rom): riscv_virt_board.mrom
    0000000000100000-0000000000100fff (prio 0, i/o): riscv.sifive.test
--- snip ---
</pre>

Note the <code>pcie-mmio-high</code> with the range <code>0000000400000000-00000007ffffffff</code>.
If you pick any address that is within that range then it'll stop faulting and automagically work.

The next question then is: how do you get that info without the QEMU console? RISC-V platforms
usually have a FTD and presumably you got the PCI address from the same tree. If you read the
device tree specification (https://github.com/devicetree-org/devicetree-specification/releases/tag/v0.3)
you'll notice it says this:

<blockquote>
<p>2.3.8  ranges</p>

<p>Property name: ranges</p>

<p>Value type: <empty> or <prop-encoded-array> encoded as an arbitrary number of (child-bus-address,parent-bus-address,length) triplets.</p>

<p>Description: The ranges property provides a means of defining a mapping or translation between the address space of the bus (the child
address space) and the address space of the bus node’s parent (the parent address space).</p>
</blockquote>

Checking the DTS I got and lo and behold: <code> ranges = < 0x1000000 0x00 0x00 0x00 0x3000000 0x00 0x10000 0x2000000 0x00 0x40000000 0x00 0x40000000 0x00 0x40000000 0x3000000 0x04 0x00 0x04 0x00 0x04 0x00 > </code>.

Note that <code>#address-cells = < 3 ></code> for the ''pci node'', <code>#address-cells = < 2 ></code> for the ''parent node'' and <code>#size-cells = < 2 ></code> for the ''pci node''
so the triplets are <code>(0x1000000 0x00 0x00, 0x00 0x3000000, 0x00 0x10000), (0x2000000 0x00 0x40000000, 0x00 0x40000000, 0x00 0x40000000), (0x3000000 0x04 0x00, 0x04 0x00, 0x04 0x00)</code>
````

<a id="w004616"></a>

## W004616 — RTL8168

Bron-ID: W004616
Namespace: Main (0)
Revisie: 26147 | Datum: 2021-06-18T02:41:17Z
Bron: https://wiki.osdev.org/RTL8168
Revisie-URL: https://wiki.osdev.org/index.php?title=RTL8168&oldid=26147
Doorverwijzing: RTL8169

````text
#REDIRECT [[RTL8169]]
````

<a id="w004617"></a>

## W004617 — User:Immibis/PinePhone Blinky

Bron-ID: W004617
Namespace: User (2)
Revisie: 26179 | Datum: 2021-07-11T09:42:45Z
Bron: https://wiki.osdev.org/User:Immibis/PinePhone_Blinky
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AImmibis%2FPinePhone_Blinky&oldid=26179

````text
Blinks green, red, blue (in order of pin number - schematic says it should be red, green, blue - is it wired wrong?), then rumbles the motor and returns to the bootloader with the LED red.

code.s:
<pre>.section .stuff, "ax"

	ldr r0, =0x77711177
	ldr r1, =0x01C2086C
	str r0, [r1, #8]
	mov r0, #0x00040000
	str r0, [r1, #16]

	mov r2, #0x3F0000
loop1:
	sub r2, #1
	cmp r2, #0
	bne loop1
	
	mov r0, #0x00080000
	str r0, [r1, #16]
	
	mov r2, #0x3F0000
loop2:
	sub r2, #1
	cmp r2, #0
	bne loop2
	
	mov r0, #0x00100000
	str r0, [r1, #16]
	
	mov r2, #0x3F0000
loop3:
	sub r2, #1
	cmp r2, #0
	bne loop3
	
	mov r0, #0x00100000
	str r0, [r1, #16]
	
	
	ldr r0, =0x77777777
	str r0, [r1]
	
	ldr r0, =0x77777177
	str r0, [r1, #0]
	
	mov r0, #4
	str r0, [r1, #16]
	
	
	mov r2, #0x3F0000
loop4:
	sub r2, #1
	cmp r2, #0
	bne loop4
	
	ldr r0, =0x77777777
	str r0, [r1, #0]
	
	ldr r0, =0x77711177
	str r0, [r1, #8]
	
	mov r0, #0x00080000
	str r0, [r1, #16]
	
	bx lr
.pool</pre>

script.ld:

<pre>SECTIONS
{
	. = 0x00010000;
	.stuff : AT(0) {
		INPUT_SECTION_FLAGS(SHF_ALLOC) *(*)
	}
}
</pre>

run.sh:

<pre>
#!/bin/bash
set -e
arm-none-eabi-as code.s -o code.o -mcpu=cortex-a53
arm-none-eabi-ld code.o -T script.ld -o code.elf
arm-none-eabi-objcopy code.elf code.bin --output-target binary --only-section .stuff
xxd -g4 -e code.bin
sunxi-fel write 0x00010000 code.bin
sunxi-fel execute 0x00010000
</pre>
````

<a id="w004618"></a>

## W004618 — Printer Command Language

Bron-ID: W004618
Namespace: Main (0)
Revisie: 28226 | Datum: 2023-07-10T20:27:46Z
Bron: https://wiki.osdev.org/Printer_Command_Language
Revisie-URL: https://wiki.osdev.org/index.php?title=Printer_Command_Language&oldid=28226

````text
{{stub}}

== Overview ==
Printer Command Language (PCL), is a page description language developed by Hewlett-Packard as a printer protocol and has become an industry standard. Originally developed for early inkjet printers in 1984, PCL has been released in varying levels for thermal, matrix, and page printers.

[[Category:Printing]]
````

<a id="w004619"></a>

## W004619 — User:Drakeor

Bron-ID: W004619
Namespace: User (2)
Revisie: 26197 | Datum: 2021-07-22T09:56:36Z
Bron: https://wiki.osdev.org/User:Drakeor
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADrakeor&oldid=26197

````text
[https://www.drakeor.com My Website]
````

<a id="w004620"></a>

## W004620 — User:VSlezak

Bron-ID: W004620
Namespace: User (2)
Revisie: 29622 | Datum: 2025-08-17T10:37:57Z
Bron: https://wiki.osdev.org/User:VSlezak
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AVSlezak&oldid=29622

````text
https://github.com/VendelinSlezak/BleskOS
````

<a id="w004621"></a>

## W004621 — User:Leapofazzam

Bron-ID: W004621
Namespace: User (2)
Revisie: 26203 | Datum: 2021-07-24T14:45:44Z
Bron: https://wiki.osdev.org/User:Leapofazzam
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALeapofazzam&oldid=26203

````text
Hi, I'm a developer of a hobby OS called [[FaruOS]]
````

<a id="w004622"></a>

## W004622 — User:Egoros7

Bron-ID: W004622
Namespace: User (2)
Revisie: 26206 | Datum: 2021-07-25T19:12:35Z
Bron: https://wiki.osdev.org/User:Egoros7
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AEgoros7&oldid=26206

````text
Hello, i am egor, i am just using my page as sandbox :)
----
== About me ==
I'm from Russia, Nijniy Novgorod.
--[[User:Egoros7|Egoros7]] 14:12, 25 July 2021 (CDT)
````

<a id="w004624"></a>

## W004624 — EDK2

Bron-ID: W004624
Namespace: Main (0)
Revisie: 30304 | Datum: 2026-03-16T20:35:59Z
Bron: https://wiki.osdev.org/EDK2
Revisie-URL: https://wiki.osdev.org/index.php?title=EDK2&oldid=30304

````text
{{In_Progress}}
EDK2 is the official development environment for [[UEFI]] applications. It is developed by the open-source Tianocore project, of which Intel, HP, and Microsoft are the primary contributors. Although it may be bigger than [[GNU-EFI]], it has more features, hence, some OS developers may prefer it over GNU-EFI.

==What is EDK2?==
EDK2 is full on implementation of the [[UEFI]] spec. It contains the Open Virtual Machine Firmware (OVMF) UEFI firmware package, which is primarily targeted at [[QEMU]], and is used by many OS developers to test out their UEFI applications. It also has [[ARM]] and [[AArch64]] firmware packages for QEMU and various boards, and [[RISC-V]] firmware packages for various [[HiFive]] boards. It also has tools for UEFI driver developers, but that is outside the scope of this wiki.

Of most interest the OS developers is the UEFI library and the EDK2 build system. This article will show you the structure of EDK2, how to build firmware for QEMU and real machines, and how to build UEFI applications.

==Structure of EDK2==
EDK2 is made up of multiple repos, all of which can be found at [https://github.com/tianocore https://github.com/tianocore]. The main ones are:

- edk2, which contains the build system, library, OVMF, and ArmVirt firmware

- edk2-platforms, which contains firmware for various real hardware platforms, such as the SiFive U450, Raspberry Pi, and quite a few others

- edk2-non-osi, which contains proprietary binary blobs for various platforms.

EDK2 itself is written in C with some Python, and is licensed under the BSD 2 Clause + Patent license. The blobs are subject to their own licenses.
===Directory structure===
EDK2 is split up into multiple subdirectories, where each one normally contains a "Package". The OvmfPkg subdirectory contains OVMF, the MdePkg subdirectory contains the UEFI library, and so on. Each one of these contains a file ending in .dsc. This file describes the package inside of the folder, and contains things such as dependencies, different components, and so on. Each component is described its own file that contains sources, application type, and so on.
TODO: Describe this in more detail
==Building EDK2's base==
No matter what you are trying to build from EDK2, these steps must be completed. First, you must decide if you want to only build stuff inside of the main EDK2 repo, or if you also want stuff from edk2-platforms. Note that these guides assume you are on a Linux system (Windows Subsystem for Linux is OK)
===Building without edk2-platforms===
If you only want the UEFI library or virtual machine firmware, then this guide will suffice. Make sure you have all the dependencies installed first:
<syntaxhighlight lang="bash">
sudo apt install build-essential git python2 uuid-dev nasm acpica-tools        # Or whatever your package manager is
</syntaxhighlight>
First, we must clone EDK2
<syntaxhighlight lang="bash">
git clone https://github.com/tianocore/edk2.git
</syntaxhighlight>
Note that this is the latest non-stable source. If you want the stable source then run:
<syntaxhighlight lang="bash">
git clone https://github.com/tianocore/edk2.git -b"stable/202011"
</syntaxhighlight>
or whatever is appropriate at the time you are reading this. After that, we now must clone the submodules
<syntaxhighlight lang="bash">
cd edk2
git submodule update --init
</syntaxhighlight>
Then, we need to setup the environment for EDK2
<syntaxhighlight lang="bash">
export EDK_TOOLS_PATH=$PWD/BaseTools
# If you are compiling for ARM, then this is needed
sudo apt install gcc-aarch64-linux-gnu        # Or whatever your package manager is
export GCC5_AARCH64_PREFIX=aarch64-linux-gnu-
# This is needed on RISC-V
sudo apt install gcc-riscv64-linux-gnu        # Or whatever your package manager is
export GCC5_RISCV64_PREFIX=riscv64-linux-gnu-
# Run this no matter what you are compiling for
source edksetup.sh
</syntaxhighlight>
The last step generates various configuration files and sets up some variables. Note that edksetup.sh must be sourced, as its variables must persist.
Now we need to build the base tools
<syntaxhighlight lang="bash">
make -C BaseTools
</syntaxhighlight>
And lastly, we must actually build EDK2 using its Python build script. 
<syntaxhighlight lang="bash">
build -a YOUR_ARCH -t GCC5 -p YOUR_PACKAGE
</syntaxhighlight>
YOUR_ARCH can be IA32, X64, AARCH64, ARM, or RISCV64. Note that not all packages compile for all architectures. YOUR_PACKAGE must point to a package in the source tree. Further guides in this article will tell you what -p option to pass to EDK2's build script.

==== EDK2 since stable release 202602 ====
If you decide to compile from the latest Git source tree (as of 2026-02-27), the build command changes slightly, as the <code>GCC5</code> compiler option has been deprecated. Instead, simply remove the '5', so that the compiler option simply becomes <code>GCC</code>. This also applies when building OVMF and ArmVirt.

===Building with edk2-platforms===
If you actually want firmware that runs on real hardware, then you must perform the following steps instead of the one above

First, we need to create our workspace folder
<syntaxhighlight lang="bash">
mkdir edk2 && cd edk2
export WORKSPACE=$PWD
</syntaxhighlight>
Now we need to clone the source to all repos. edk2-non-osi is optional depending on the platform you are building for, but I would recommend downloading it anyway
<syntaxhighlight lang="bash">
git clone https://github.com/tianocore/edk2.git -b"stable/202011"
git clone https://github.com/tianocore/edk2-platforms.git
git clone https://github.com/tianocore/edk2-non-osi.git
cd edk2 && git submodule update --init
cd ../edk2-platforms && git submodule update --init && cd ..
</syntaxhighlight>
We need to let EDK2's build system know where these folders are. We'll also setup a couple other variables as well
<syntaxhighlight lang="bash">
export EDK_TOOLS_PATH="$PWD/BaseTools"
export PACKAGES_PATH="$PWD/edk2:$PWD/edk2-platforms:$PWD/edk2-non-osi"
</syntaxhighlight>
We now must setup the environment now
<syntaxhighlight lang="bash">
. edk2/edksetup.sh
make -C edk2/BaseTools
</syntaxhighlight>
After that, building is exactly like before. To build for a certain platform in edk2-platforms, simply find the relevant .dsc file.
==Building an EDK2 UEFI application==
Once you have EDK2's base built, building UEFI applications is super easy. As OS developers, the main package will be working with is MdeModulePkg, a complete UEFI wrapper library.

First, we must create a directory that contains our application. We'll create it inside of the main EDK2 directory, like this
<syntaxhighlight lang="bash">
cd edk2
mkdir MyEfiApp
</syntaxhighlight>
Now, create a file named MyEfiApp.inf in the MyEfiApp directory. Give it the following contents
<syntaxhighlight lang="ini">
[Defines]
  INF_VERSION = 1.25
  BASE_NAME = MyEfiApp
  FILE_GUID = # Get a GUID from guidgen.com
  MODULE_TYPE = UEFI_APPLICATION
  VERSION_STRING = 1.0
  ENTRY_POINT = UefiEntry

[Sources]
  UefiMain.c

[Packages]
  MdePkg/MdePkg.dec

[LibraryClasses]
  UefiApplicationEntryPoint
  UefiLib

[Guids]

[Ppis]

[Protocols]

[FeaturePcd]

[Pcd]

</syntaxhighlight>
Create a file called UefiMain.c in the same directory MyEfiApp.inf, and give it the following contents
<syntaxhighlight lang="c">
#include <Uefi.h>
#include <Library/UefiLib.h>
#include <Library/UefiBootServicesTableLib.h>

EFI_STATUS EFIAPI UefiEntry(IN EFI_HANDLE imgHandle, IN EFI_SYSTEM_TABLE* sysTable)
{
    gST = sysTable;
    gBS = sysTable->BootServices;
    gImageHandle = imgHandle;
    // UEFI apps automatically exit after 5 minutes. Stop that here
    gBS->SetWatchdogTimer(0, 0, 0, NULL);
    Print(L"Hello, world!\r\n");
    // Allocate a string
    CHAR16* str = NULL;
    gBS->AllocatePool(EfiLoaderData, 36, (VOID**)&str);
    // Copy over a string
    CHAR16* str2 = L"Allocated string\r\n";
    gBS->CopyMem((VOID*)str, (VOID*)str2, 36);
    Print(str);
    gBS->FreePool(str);
    return EFI_SUCCESS;
}
</syntaxhighlight>

Now run edksetup.sh if need be. After that, find a "Components" section MdeModulePkg/MdeModulePkg.dsc, and add this to it
<syntaxhighlight lang="ini">
[Components]
  ...
  MyEfiApp/MyEfiApp.inf
</syntaxhighlight>
After that, call EDK2's build system like this to build MyEfiApp
<syntaxhighlight lang="bash">
build -a YOUR_ARCH -t GCC5 -p MdeModulePkg/MdeModulePkg.dsc
</syntaxhighlight>
Finally, in the folder Build/MdeModule/DEBUG_GCC5/YOUR_ARCH, there should be a file named MyEfiApp.efi.

==Building OVMF and ArmVirt==
Building OVMF and ArmVirtPkg is easy, after setting up EDK2, run the following for OVMF:
<syntaxhighlight lang="bash">
build -a YOUR_ARCH -t GCC5 -p OvmfPkg/OvmfPkgYOUR_ARCH.dsc
</syntaxhighlight>
YOUR_ARCH can only be IA32 or X64 for OVMF. For ArmVirtPkg, run this instead:
<syntaxhighlight lang="bash">
build -a YOUR_ARCH -t GCC5 -p ArmVirtPkg/ArmVirtQemu.dsc
</syntaxhighlight>
YOUR_ARCH can only be ARM or AARCH64. After this,, the firmware will be output in for OVMF Build/OvmfYOUR_ARCH/DEBUG_GCC5/FV, or for ArmVirtPkg Build/ArmVirtQemu-YOUR_ARCH/DEBUG_GCC5/FV

[[Category:UEFI]]
[[Category:Firmware]]
````

<a id="w004625"></a>

## W004625 — User:Nexos

Bron-ID: W004625
Namespace: User (2)
Revisie: 27232 | Datum: 2022-07-12T19:35:29Z
Bron: https://wiki.osdev.org/User:Nexos
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANexos&oldid=27232

````text
I am nexos, an open source hobbyist software developer. Here are my coding interests:

- Really, anything low level

- RISC-V assembly

- The Unix shell

- UEFI and ACPI

- Memory manager and scheduler design (implementation, well that is coming soon)

- Fast algorithms

- Unix

- Operating systems

- C, C++

- Executable formats

I am currently working on [http://github.com/nexos-dev/nexnix.git NexNix], a Unix like operating system. It will be very fast, safe, and stable, while being a microkernel and investigating some interesting ideas.

My software philosophy is keep it simple. Everything should be made with very little, until more is needed. To quote Einstein:
"Everything should be as simple as possible, but not simpler"
Also, speed is very important to me. Software is slow nowadays, and people just say "modern CPUs make up for that", which is only so true.

My current projects are NexNix, a code editor named NexCode, and refining support for PE across the GNU toolchain.
````

<a id="w004626"></a>

## W004626 — PLIC

Bron-ID: W004626
Namespace: Main (0)
Revisie: 30829 | Datum: 2026-06-29T19:30:08Z
Bron: https://wiki.osdev.org/PLIC
Revisie-URL: https://wiki.osdev.org/index.php?title=PLIC&oldid=30829

````text
{{stub}}

The '''PLIC''' (Platform Level Interrupt Controller) is the RISC-V equivalent of the [[PIC]] for x86 platforms. Without it, no external interrupts can be received (on "standard" platforms, anyways).

== Quickstart ==

First, figure out the start address & size of the PLIC MMIO region & map it. In the [[DTB]] there should be an entry like this:

<syntaxhighlight lang="dts">
plic@c000000 {
	phandle = < 0x03 >;
	riscv,ndev = < 0x35 >;
	reg = < 0x00 0xc000000 0x00 0x210000 >;
	interrupts-extended = < 0x02 0x0b 0x02 0x09 >;
	interrupt-controller;
	compatible = "riscv,plic0";
	#interrupt-cells = < 0x01 >;
	#address-cells = < 0x00 >;
};
</syntaxhighlight>

Then, determine which interrupt source(s) to enable. For example, to receive interrupts from the [[UART]] device, interrupt 0x0A must be enabled:

<syntaxhighlight lang="dts">
uart@10000000 {
	interrupts = < 0x0a >;
	interrupt-parent = < 0x03 >;
	clock-frequency = < 0x384000 >;
	reg = < 0x00 0x10000000 0x00 0x100 >;
	compatible = "ns16550a";
};
</syntaxhighlight>

Interrupts are enabled on a per-context basis. The exact meaning of a context is implementation-dependent but generally even-numbered contexts (0, 2, ...) refer to M-mode of hart 0, 1, ... while odd-numbered contexts (1, 3, ...) refer to S-mode of hart 0, 1, ...

For example, to enable interrupt 0x0A for S-mode in hart 0 the 10th bit at <code>base + 0x2000 + 0x80 * context + source / 32</code> must be toggled. Note that the registers are all 32 bits!

The priority and priority thresholds must be set to actually enable the interrupt. <code>base + 0x0 + 0x4 * source</code> must be set to any non-zero value and <code>base + 0x200000 + 0x1000 * context</code> to any value lower than the priority.

Finally, enable any interrupts on the [[UART]] device:

<pre>
uart_base[0] = 0x00; // Disable DLAB
uart_base[1] = 0x01; // Enable data available interrupts
</pre>

If the trap handler is set up properly, an external interrupt should be triggered when any characters are sent.

== Context ==
Context is an abstract representation of the hart ''x'' under privilege level ''y''. It is the minimal unit of interrupt configuration through PLIC. Each context has an unque ID, and this ID will determine which hart in which privilege mode will the PLIC target when it is delivering interrupt signal. For example, Context 0 map to hart 0 M-mode, Context 1 map to hart 0 S-mode, Context 2 map to hart 1 M-mode, etc...

In this case, when we configure PLIC context 0, we actually configure the external interrupt for Hart 0 when it is in M-mode.

The exact context mapping should always be verified against the target development board's manual.For example, on qemu-virt machine, even numbered contexts (0, 2, 4, ...) map to M mode of hart (0, 1, 2, ...), while odd-numbered contexts map to S mode of hart (0, 1, 2, ...)

== Enable/Disable interrupt ==
There's already a simple example above talked about how to enable 0x0A interrupt in PLIC. The enable register for each interrupt under each context are mapped into memory begin at <code>base + 0x2000</code>. Each bits represent the enable/disable of an interrupt, and each context can hold up to 1024 interrupt sources. Below is a detailed memory map for this area:

{| class="wikitable"
|-
| base + 0x0000_2000
| Enable bits for sources 0-31 on context 0
|-
| base + 0x0000_2004
| Enable bits for sources 32-63 on context 0
|-
| ...
| ...
|-
| base + 0x0000_207C
| Enable bits for sources 992-1023 on context 0
|-
| base + 0x0000_2080
| Enable bits for sources 0-31 on context 1
|-
| base + 0x0000_2084
| Enable bits for sources 32-63 on context 1
|-
| ...
| ...
|-
| base + 0x0000_20FC
| Enable bits for sources 992-1023 on context 1
|-
| base + 0x0000_2100
| Enable bits for sources 0-31 on context 2
|-
| base + 0x0000_2104
| Enable bits for sources 32-63 on context 2
|-
| ...
| ...
|-
| base + 0x0000_217C
| Enable bits for sources 992-1023 on context 2
|-
| ...
| ...
|-
| base + 0x001F_1F80
| Enable bits for sources 0-31 on context 15871
|-
| base + 0x001F_1F84
| Enable bits for sources 32-63 on context 15871
|-
| base + 0x001F_1FFC
| Enable bits for sources 992-1023 on context 15871
|}

== Handling interrupts ==

The PLIC handles interrupts using the following flow: <code>Receive → Pending → Claim → Complete</code>

An interrupt source triggers an interrupt by sending an activation signal to the PLIC. The PLIC then evaluates which contexts are eligible to receive this interrupt based on their enable and threshold settings. For each eligible context, the interrupt is marked as pending, and a signal is sent to the corresponding CPU core. Once the host CPU receives this signal, it saves the current machine state and jumps to a user-defined interrupt handler.

To claim the interrupt, the host processor reads from the claim/complete register associated with its context. This read operation returns the ID of the highest-priority pending interrupt, or 0 if no interrupt is pending for that context.

In a multicore system where the same interrupt may be delivered to multiple contexts, each hart must independently issue a read request to the PLIC. These requests are typically routed through NoC or interconnect. The PLIC ensures that only one hart receives the interrupt ID, while others receive 0. Once an interrupt is successfully claimed, the PLIC clears its pending bit.

After the interrupt has been processed by the handler, the host must signal its completion. This is done by writing the same interrupt ID that was returned by the claim to the claim/complete register. The PLIC does not verify whether the written ID matches the one last read, but it will ignore any completion request for an interrupt ID that is not currently enabled for that context.

For example, to begin handling an interrupt, the interrupt must be claimed by reading <code>base + 0x200000 + 0x4 + 0x1000 * context</code>, which will return the source. This automatically clears the pending bit.

The interrupt <strong>must</strong> be claimed before returning to userspace! Not doing so will not clear the pending bits and it will cause the same interrupt to be triggered again immediately.

<strong>Note:</strong> the interrupt should not be marked as completed until whatever caused it has actually been dealt with, e.g. don't mark an interrupt from UART as completed until all the data has been read.

Here is the memory map for context claim/complete register:

{| class="wikitable"
|-
| base + 0x001F_FFFC
| Reserved
|-
| base + 0x0020_0000
| Priority threshold for context 0
|-
| base + 0x0020_0004
| Claim/complete for context 0
|-
| base + 0x0020_0008
| Reserved
|-
| ...
| ...
|-
| base + 0x0020_0FFC
| Reserved
|-
| base + 0x0020_1000
| Priority threshold for context 1
|-
| base + 0x0020_1004
| Claim/complete for context 1
|-
| base + 0x0020_1008
| Reserved
|-
| ...
| ...
|-
| base + 0x03FF_EFFC
| Reserved
|-
| base + 0x03FF_F000
| Priority threshold for context 15871
|-
| base + 0x03FF_F004
| Claim/complete for context 15871
|-
| base + 0x03FF_F008
| Reserved
|-
| ...
| ...
|-
| base + 0x03FF_FFFC
| Reserved
|}

=== threshold ===
In PLIC, each context offer a configurable threshold number for filter interrupt requests. The PLIC will mask all PLIC interrupts of a priority less
than or equal to threshold. For example, threshold value 0 permits all interrupts with non-zero, while threshold value 7 mute all interrupts.

== See Also ==

=== External Links ===

* [https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic.adoc RISC-V Platform-Level Interrupt Controller Specification]
* [https://github.com/riscv/riscv-isa-manual/releases/download/draft-20181201-5449851/riscv-privileged.pdf The RISC-V Instruction Set Manual, Volume II: Privileged Architecture (draft 2018-12-01)]. Chapter 7 describes the PLIC in more detail. Supposedly it was moved to another document but I'm unable to find it.


[[Category:RISC-V]]
[[Category:Interrupts]]
````

<a id="w004627"></a>

## W004627 — User:Demindiro/PLIC

Bron-ID: W004627
Namespace: User (2)
Revisie: 26293 | Datum: 2021-08-12T19:19:54Z
Bron: https://wiki.osdev.org/User:Demindiro/PLIC
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADemindiro%2FPLIC&oldid=26293
Doorverwijzing: PLIC

````text
#REDIRECT [[PLIC]]
````

<a id="w004628"></a>

## W004628 — User:Demindiro

Bron-ID: W004628
Namespace: User (2)
Revisie: 27178 | Datum: 2022-06-17T17:53:40Z
Bron: https://wiki.osdev.org/User:Demindiro
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADemindiro&oldid=27178

````text
Hello! I'm a hobbyist OS developer. My interest got piqued when someone else showed their OS and since then I've been reading about OS development & toying around with my own OS project.

== Notes & stuff ==

[[User:Demindiro/RISC-V Notes]]

== Wiki pages I'm working on ==

* [https://wiki.osdev.org/User:Demindiro/SBI SBI]

* [https://wiki.osdev.org/User:Demindiro/Porting_Rust_standard_library Porting Rust standard library]

* [https://wiki.osdev.org/User:Demindiro/DisplayPort DisplayPort]

== My OS ==

My OS is [https://norost.com/ Norost B], an object-oriented OS built around a microkernel.

* [https://git.sr.ht/~demindiro/norost-b sourcehut]

* [https://github.com/Demindiro/norost-b Github]

== Graveyard ==

These are my attempts that I have scrapped so far:

=== Norost A ===

The first attempt at writing an OS. I went in pretty much blind as I haven't written an OS nor read much about OSDev in general. I ended up cancelling it as the design was incomplete and I was piling hacks upon hacks to get things to work.

* [https://github.com/Demindiro/norost-a Github]


== See also ==

* [https://demindiro.com/ My website]

* [https://github.com/Demindiro Github]
````

<a id="w004629"></a>

## W004629 — Swift Barebones

Bron-ID: W004629
Namespace: Main (0)
Revisie: 26304 | Datum: 2021-08-12T23:03:32Z
Bron: https://wiki.osdev.org/Swift_Barebones
Revisie-URL: https://wiki.osdev.org/index.php?title=Swift_Barebones&oldid=26304
Doorverwijzing: Swift Bare Bones

````text
#REDIRECT [[Swift Bare Bones]]
````

<a id="w004630"></a>

## W004630 — Ada Bare bones

Bron-ID: W004630
Namespace: Main (0)
Revisie: 26306 | Datum: 2021-08-12T23:08:22Z
Bron: https://wiki.osdev.org/Ada_Bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Ada_Bare_bones&oldid=26306
Doorverwijzing: Ada Bare Bones

````text
#REDIRECT [[Ada Bare Bones]]
````

<a id="w004631"></a>

## W004631 — Talk:Ada Bare bones

Bron-ID: W004631
Namespace: Talk (1)
Revisie: 26308 | Datum: 2021-08-12T23:08:22Z
Bron: https://wiki.osdev.org/Talk:Ada_Bare_bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AAda_Bare_bones&oldid=26308
Doorverwijzing: Talk:Ada Bare Bones

````text
#REDIRECT [[Talk:Ada Bare Bones]]
````

<a id="w004632"></a>

## W004632 — User:Demindiro/SBI

Bron-ID: W004632
Namespace: User (2)
Revisie: 26344 | Datum: 2021-08-16T18:45:56Z
Bron: https://wiki.osdev.org/User:Demindiro/SBI
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADemindiro%2FSBI&oldid=26344

````text
{{stub}}

{{In Progress}}


The RISC-V SBI (Supervisor Binary Interface) defines a common interface for RISC-V platforms for OSes. It hides platform-specific implementation details such that OSes are more portable.

Notably, it adds methods to facilitate IPI (Inter-Processor Interrupts).

The reference implementation is [https://github.com/riscv/opensbi OpenSBI].

== OpenSBI on QEMU ==

To run OpenSBI on QEMU, download the source and compile with <code>make PLATFORM=generic CROSS_COMPILE=riscv64-your-os-</code>. Then add <code>-bios path/to/opensbi/build/platform/generic/firmware/fw_jump.bin</code>

=== fw_jump vs fw_payload vs fw_dynamic ===

* <code>fw_jump</code> should be used in conjunction with QEMU's <code>-kernel</code> option

* <code>fw_payload</code> should directly include the kernel binary.

* <code>fw_dynamic</code> should be used if you use another bootloader that passes more information to OpenSBI.


== Interface ==

The calling convention is the [https://github.com/riscv/riscv-elf-psabi-doc/blob/master/riscv-elf.md RISC-V ELF psABI]. This means you'll need to save caller-saved registers!

The main difference is that the EID (Extension ID) goes into <code>a7</code> and FID (Function ID) into <code>a6</code>.


== Extensions ==

Avoid the legacy extensions (EID 0x00 - 0xFF) as those are deprecated.

=== Timer (0x54494D45) ===

The timer extension has a single function <code>sbi_set_timer</code> with FID 0x00. It takes a single <code>uint64_t</code> argument.

To reset the timer, pass <code>UINT64_MAX</code>. This clears the <code>STIP</code> bit and effectively disables the timer.

{{NoteBox
| <p>QEMU as of 6.1.0-rc3 [https://gitlab.com/qemu-project/qemu/-/issues/493 does not handle <code>UINT64_MAX</code> properly]. To fix this, apply the following patch:</p>
<pre style="text-align:left">
diff --git a/hw/intc/sifive_clint.c b/hw/intc/sifive_clint.c
index 0f41e5ea1c..e65e71e5ec 100644
--- a/hw/intc/sifive_clint.c
+++ b/hw/intc/sifive_clint.c
@@ -61,6 +61,8 @@ static void sifive_clint_write_timecmp(RISCVCPU *cpu, uint64_t value,
     /* back to ns (note args switched in muldiv64) */
     next = qemu_clock_get_ns(QEMU_CLOCK_VIRTUAL) +
         muldiv64(diff, NANOSECONDS_PER_SECOND, timebase_freq);
+    /* ensure next does not overflow, as timer_mod takes a signed value */
+    next = MIN(next, INT64_MAX);
     timer_mod(cpu->env.timer, next);
 }
</pre>
}}

== See Also ==

=== External Links ===

* [https://github.com/riscv/riscv-sbi-doc/blob/master/riscv-sbi.adoc RISC-V Supervisor Binary Interface Specification]



[[Category:RISC-V]]
````

<a id="w004633"></a>

## W004633 — ARM

Bron-ID: W004633
Namespace: Main (0)
Revisie: 26335 | Datum: 2021-08-16T13:21:24Z
Bron: https://wiki.osdev.org/ARM
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM&oldid=26335
Doorverwijzing: ARM Overview

````text
#REDIRECT [[ARM Overview]]
````

<a id="w004635"></a>

## W004635 — File:Bananaos.PNG

Bron-ID: W004635
Namespace: File (6)
Revisie: 26440 | Datum: 2021-10-02T22:11:20Z
Bron: https://wiki.osdev.org/File:Bananaos.PNG
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABananaos.PNG&oldid=26440

````text
Image of Banana OS
````

<a id="w004636"></a>

## W004636 — ATA in x86 RealMode (BIOS)

Bron-ID: W004636
Namespace: Main (0)
Revisie: 26462 | Datum: 2021-10-05T18:52:22Z
Bron: https://wiki.osdev.org/ATA_in_x86_RealMode_(BIOS)
Revisie-URL: https://wiki.osdev.org/index.php?title=ATA_in_x86_RealMode_%28BIOS%29&oldid=26462
Doorverwijzing: Disk access using the BIOS (INT 13h)

````text
#REDIRECT [[Disk access using the BIOS (INT 13h)]]
````

<a id="w004637"></a>

## W004637 — User:Klange/Tables

Bron-ID: W004637
Namespace: User (2)
Revisie: 26470 | Datum: 2021-10-07T13:39:46Z
Bron: https://wiki.osdev.org/User:Klange/Tables
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKlange%2FTables&oldid=26470

````text
{| class="wikitable sortable"
|+ Operating System Projects
|-
! Project
! scope="col" class="unsortable" | Description 
! License
! URL
! Last Update
! scope="col" class="unsortable" | Screenshot
|-
! ToaruOS
| A complete educational operating system for x86-64 PCs, with a bootloader, kernel, libc, window compositor, graphical applications, and Unix-like utilities.
| NCSA/University of Illinois license
| https://toaruos.org/
| 2021-10-07
| [[Image:Toaruos_screenshot.png|200px]]
|-
! Another Project
| Blah blah blah
| BSD
| https://github.com/
| 2020-04-20
|
|}


{{OSProject
|OSName=[[User:Klange/ToaruOS|ToaruOS (とあるOS)]]
|OSDescription=A complete educational operating system for x86-64 PCs, with a bootloader, kernel, libc, window compositor, graphical applications, and Unix-like utilities.
|Contact=#toaruos on libera.chat
|License=Open source (NCSA/University of Illinois license)
|URL=http://github.com/klange/toaruos
|Status=Latest commit: 2021-08-17, is active
|Pic=[[Image:Toaruos_screenshot.png|200px]]
}}
````

<a id="w004638"></a>

## W004638 — ZDSFS

Bron-ID: W004638
Namespace: Main (0)
Revisie: 28011 | Datum: 2023-07-09T17:58:21Z
Bron: https://wiki.osdev.org/ZDSFS
Revisie-URL: https://wiki.osdev.org/index.php?title=ZDSFS&oldid=28011

````text
{{Filesystems}}

'''ZDSFS''' is the normal filesystem used on virtually all [[DASD]] disks.

== VTOC ==

The filesystem uses DSCBs to hold the information of all files in the VTOC table (Volume's Table Of Contents). This table is found after the DASD's IPL which usually means the VTOC is at record 3, cylinder 0 and head 0.

To find the sequence of format 1 entries the OS has to read 0, 0, 3. Place the first 20 bytes into a buffer then obtain the location of the format 1 DSCBs:
<source lang="c">
void GetCHRFromVTOC(void *buffer) {
    uint16_t cyl, head;
    uint8_t rec;

    memcpy(&cyl, buffer + 15, 2); /* 15-16 */
    memcpy(&head, buffer + 17, 2); /* 17-18 */
    memcpy(&rec, buffer + 19, 1); /* 19-19 */
    return;
}
</source>

== Format 1 DSCB ==

{| class="wikitable" 
|-
! Name
! Size
! Usage
|-
| DS1NAM
| 44
| Name of dataset
|-
| DS1FMTID
| 1
| Format Id
|-
| UNUSED
| 80
| 
|-
| EXT_TYPE
| 1
| 
|-
| SEQUENCE_NUMBER<br />
| 1
| 
|-
| START_CC
| 2
| Starting cylinder
|-
| START_HH
| 2
| Starting head
|-
| END_CC
| 2
| Ending cylinder
|-
| END_HH
| 2
| Ending head
|}

This entry describes a dataset (file) general information, including location and name. This is generally what most people will search for.

The entries are ordered in a chain-like structure, where a formatId=0 entry denotes the end of said chain.

<source lang="c">
// NOTE: This code assumes EBCDIC is being used
void CheckDSCB(struct DSCB1 *dscb, const char *name) {
    if(dscb1.ds1fmtid == '1') {
        dscb1.ds1fmtid = ' ';
        if(!memcmp(&dscb1.ds1dsnam, name, strlen(name))) {
            // Dataset found
            cyl = dscb1.start_cc;
            head = dscb1.start_hh;
            rec = 1;

            kprintf("DATASET %s @ CYL=%i,HEAD=%i,RECORD=%i\n", name, (int)cyl, (int)head, (int)rec);

            // The kernel now should use Cyl, Head and the Rec locations provided to read the dataset
            kpanic("TODO: Use cyl, head and rec to read the file\n");
        }
    } else if(dscb1.ds1dsnam[0] == '\0') {
        // Not found
        kpanic("Could not find file %s!", name);
    }
}
</source>

Finding the files is just a matter of reading records until reaching the end of the head, then incrementing heads and after reading the last head in the cylinder, incrementing cylinders. This also has to be done for reading files.

This is mostly specific to DASD devices but it's provided for clarity.
<source lang="c">
// NOTE: This code assumes EBCDIC is being used
void FindFile(const char *name) {
    while(errcnt < 4) {
        r = DasdReadCHR(hdl, &cyl, &head, &rec, &dscb1, sizeof(dscb1));
        if(r < 0) {
            ++errcnt;
            if(errcnt == 1) {
                // If we have an error reading this record skip to the next
                ++rec;
            } else if(errcnt == 2) {
                // Next record failed?, now we need to change heads
                rec = 1;
                ++head;
            } else if(errcnt == 3) {
                // Switch cylinders if we can't read new heads either
                rec = 1;
                head = 0;
                ++cyl;
            }
            continue;
        }

        errcnt = 0;
        if(r >= (int)sizeof(dscb1)) {
            CheckDSCB(&dscb1, name);
        }

        // Go to next record
        ++rec;
    }
}
</source>

== See also ==
=== Source code ===
* https://sourceforge.net/p/pdos/gitcode/ci/master/tree/s370/pdosutil.c#l25 Find a file from ZDSFS
=== External links ===
* https://www.ibm.com/docs/en/zos/2.3.0?topic=components-data-set-control-block-dscb-types DSCB types
* https://www.ibm.com/docs/en/zos/2.3.0?topic=dscbs-how-found Obtaining format 1 entries

[[Category:S390]]
[[Category:Filesystems]]
````

<a id="w004639"></a>

## W004639 — File:Page directory entry.png

Bron-ID: W004639
Namespace: File (6)
Revisie: 26578 | Datum: 2021-10-22T23:09:22Z
Bron: https://wiki.osdev.org/File:Page_directory_entry.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APage_directory_entry.png&oldid=26578

````text

````

<a id="w004640"></a>

## W004640 — File:Page table entry.png

Bron-ID: W004640
Namespace: File (6)
Revisie: 26490 | Datum: 2021-10-14T23:15:58Z
Bron: https://wiki.osdev.org/File:Page_table_entry.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APage_table_entry.png&oldid=26490

````text

````

<a id="w004641"></a>

## W004641 — Command SubSystem

Bron-ID: W004641
Namespace: Main (0)
Revisie: 26503 | Datum: 2021-10-15T18:59:39Z
Bron: https://wiki.osdev.org/Command_SubSystem
Revisie-URL: https://wiki.osdev.org/index.php?title=Command_SubSystem&oldid=26503

````text
The command subsystem is responsible of handling command programs and routing programs and actions to specific devices. It works as if it was a big device which has DMA.

The CSS uses chains of commands instead of a sequential execution: it uses multiple commands at once instead of only one command at a time (ex. in x86, I/O bound instructions can only output 1 element at a time).

The execution of CSS commands is sequential. The commands are executed in the order they appear.

CSS addresses are real, this means that all addresses are physical and are not bound to the current virtual space.

== Command words ==
There are 2 types of command words, the first is used by S360 for SIO/TIO set of instructions, these can be considered mostly obsolete. The second format is more modern and is actively used in most Z11 and z/Arch machines.

<source lang="c">
// Format 0 Control Command Word
struct css_ccw0 {
  uint8_t cmd;
  uint16_t lo_addr;
  uint8_t hi_addr;
  uint8_t flags;
  uint8_t reserved;
  uint16_t count;
} __attribute__((packed));

// Format 1 Control Command Word
struct css_ccw1 {
  uint8_t cmd;
  uint8_t flags;
  uint16_t count;
  uint32_t addr;
} __attribute__((packed));
</source>

== Command types ==
=== SEEK (0x07) ===
Address points to a BBCCHH field (that is, 1 unused halfword, 1 halfword denoting cylinders and another halfword denoting heads). The count should be the size of the BBCCHH (6)

=== SEARCH (0x31) ===
This will perform a search on the disk (something akin to a seek, but done on the track); Address points to a CCHHR field (that is, 1 halfword denoting cylinders, 1 halfword denoting heads and a character denoting record number). The coount should be the size of the CCHHR (5)

=== Transfer In Channel (0x08) ===
Jumps to the address field if an error occurs during a SEARCH on the DASD - when the operation is sucessful this command is skipped.
TIC will loop infinitely when SEARCH fails, when the DASD detects that it has read the entire track it will abnormally exit the channel program.

=== Read key and Data (0x0E) ===
Address is used to point to the destination buffer and length is used to denote the length of data to read.

=== Enable (0x27) ===
Used mostly by telnet terminals to request them to be enabled (start listening/activate send mode).

== Examples ==

=== Start Channel ===
Starts the execution of an ORB which will be executed on the given device.
<source lang="c">
// cc=0 - operational
// cc=3 - not available/device was taken off
int css_start_channel(
    struct css_schid schid,
    struct css_orb *schib)
{
    register struct css_schid r1 __asm__("1") = schid;
    int cc = -1;
    __asm__ __volatile__(
        "ssch 0(%1)\n"
        "ipm %0\n"
        : "+d"(cc)
        : "a"(schib), "d"(r1), "m"(schib)
        : "cc", "memory");
    return cc >> 28;
}
</source>

=== Store Channel ===
<source lang="c">
// cc=0 - operational
// cc=3 - not available/device was taken off
int css_store_channel(
    struct css_schid schid,
    void *schib)
{
    register struct css_schid r1 __asm__("1") = schid;
    int cc = -1;
    __asm__ __volatile__(
        "stsch 0(%2)\n"
        "ipm %0"
        : "+d"(cc), "=m"(*schib)
        : "a"(schib), "d"(r1)
        : "cc", "memory");
    return cc >> 28;
}
</source>

=== Modify Channel ===
Modify channel parameters.
<source lang="c">
// cc=0 - operational
// cc=3 - not available/device was taken off
int css_modify_channel(
    struct css_schid schid,
    struct css_schib *schib)
{
    register struct css_schid r1 __asm__("1") = schid;
    int cc = -1;
    __asm__ __volatile__(
        "msch 0(%2)\n"
        "ipm %0"
        : "+d"(cc), "=m"(*schib)
        : "a"(schib), "d"(r1)
        : "cc", "memory");
    return cc >> 28;
}
</source>

=== Test Channel ===
Usually used to obtain the status of a device after an execution/action.
<source lang="c">
// cc=0 - operational
// cc=3 - not available/device was taken off
int css_test_channel(
    struct css_schid schid,
    struct css_irb *schib)
{
    register struct css_schid r1 __asm__("1") = schid;
    int cc = -1;
    __asm__ __volatile__(
        "tsch 0(%2)\n"
        "ipm %0"
        : "+d"(cc), "=m"(*schib)
        : "a"(schib), "d"(r1)
        : "cc", "memory");
    return cc >> 28;
}
</source>

[[Category:S390]]
````

<a id="w004642"></a>

## W004642 — Program Status Word

Bron-ID: W004642
Namespace: Main (0)
Revisie: 30947 | Datum: 2026-09-13T04:20:21Z
Bron: https://wiki.osdev.org/Program_Status_Word
Revisie-URL: https://wiki.osdev.org/index.php?title=Program_Status_Word&oldid=30947

````text
The '''Program Status Word (PSW)''' on IBM Mainframe architectures (s390x / z/Architecture and ESA/390) is the central execution context register of the CPU. It controls the instruction address, execution state (supervisor vs. user/problem state, wait state), [[DAT]] (paging), interrupt masks, condition code, arithmetic exception masks, storage protection keys, and addressing modes.

Depending on the operational mode and the instruction used, the CPU works with three distinct PSW formats:
# '''Extended PSW (16 bytes / 128 bits):''' The standard native format in 64-bit '''z/Architecture''', commonly referred to as a '''current PSW'''.
# '''Short PSW (8 bytes / 64 bits):''' An 8-byte format used within z/Architecture when executing legacy instructions like <code>LPSW</code> or processing certain I/O/interruption structures.
# '''ESA/390 PSW (8 bytes / 64 bits):''' The native 31-bit format used when the CPU is running in '''ESA/390 mode''' (e.g., before switching to z/Architecture or in 31-bit guest partitions).

== Bit Numbering Convention ==

In standard IBM Principles of Operation (PoP) documentation and hardware specifications, '''Bit 0 is the Most Significant Bit (MSB 0)'''.

* In a 64-bit doubleword: Bit 0 is <code>0x8000000000000000ULL</code>, Bit 63 is <code>0x0000000000000001ULL</code>.
* In a 32-bit word: Bit 0 is <code>0x80000000U</code>, Bit 31 is <code>0x00000001U</code>.

----

== 1. Extended PSW (16-Byte z/Architecture Format) ==

The native z/Architecture format consists of two 64-bit doublewords (16 bytes total, 8-byte aligned) loaded via <code>LPSWE</code>.

=== Doubleword 0: PSW Mask (Bits 0–63) ===

{| class="wikitable"
! Bits (MSB 0) !! Bitmask (64-bit) !! Short !! Name !! Description
|-
| 0 || — || — || Reserved || Must be 0; otherwise triggers a Specification Exception (`PGM_SPECIFICATION`).
|-
| 1 || <code>0x4000000000000000ULL</code> || R || Program Event Recording (PER) || 1 = PER enabled (hardware debugging/trace traps).
|-
| 2–4 || — || — || Reserved || Must be 0.
|-
| 5 || <code>0x0400000000000000ULL</code> || T || Dynamic Address Translation (DAT) || 1 = [[DAT]] enabled (virtual addressing). 0 = Real/Absolute addressing.
|-
| 6 || <code>0x0200000000000000ULL</code> || IO || I/O Interrupt Mask || 1 = Input/Output interruptions unmasked.
|-
| 7 || <code>0x0100000000000000ULL</code> || EX || External Interrupt Mask || 1 = External interruptions (timers, inter-CPU signals) unmasked.
|-
| 8–11 || <code>0x00F0000000000000ULL</code> || Key || PSW Storage Key || 4-bit storage access key (0 = master key).
|-
| 12 || — || — || Architecture Mode || Must be 0 in z/Architecture.
|-
| 13 || <code>0x0004000000000000ULL</code> || M || Machine-Check Mask || 1 = Machine-check interruptions unmasked.
|-
| 14 || <code>0x0002000000000000ULL</code> || W || Wait State || 1 = CPU is halted in wait state.
|-
| 15 || <code>0x0001000000000000ULL</code> || P || Problem State || 0 = Supervisor state (Privileged), 1 = Problem state (User mode).
|-
| 16–17 || <code>0x0000C00000000000ULL</code> || AS || Address-Space Control (ASC) || Determines the active translation table register (CR1, CR7, or CR13).
|-
| 18–19 || <code>0x0000300000000000ULL</code> || CC || Condition Code || Condition code (0, 1, 2, or 3).
|-
| 20–23 || <code>0x00000F0000000000ULL</code> || PM || Program Mask || Arithmetic exception masks (Fixed-point overflow, Decimal overflow, HFP underflow, HFP significance).
|-
| 24–30 || — || — || Reserved || Must be 0.
|-
| 31 || <code>0x0000000100000000ULL</code> || EA || Extended Addressing Mode || Extended addressing mode bit.
|-
| 32 || <code>0x0000000080000000ULL</code> || BA || Basic Addressing Mode || Basic addressing mode bit.
|-
| 33–63 || — || — || Reserved || Must be 0 in the 16-byte PSW mask.
|}

=== Doubleword 1: Instruction Address (Bits 64–127) ===
* Holds the 64-bit virtual/absolute instruction address.
* Bit 127 must be 0 (all s390x instructions are aligned to 2-byte halfwords).

----

== 2. Short PSW (8-Byte z/Architecture Format) ==

In z/Architecture mode, an 8-byte (64-bit) '''Short PSW''' format exists primarily to maintain compatibility with software or instructions that only pass 64-bit descriptors (such as <code>LPSW</code>, legacy channel programs, and certain diagnostics).

Unlike the 16-byte PSW, the Short PSW packs both the control flags and a 31-bit instruction address into a single 64-bit word.

=== Short PSW Layout ===

{| class="wikitable"
! Bits (MSB 0) !! Short !! Name !! Description
|-
| 0 || — || Reserved || Must be 0.
|-
| 1 || R || PER Mask || Program Event Recording.
|-
| 2–4 || — || Reserved || Must be 0.
|-
| 5 || T || DAT Mode || Dynamic Address Translation.
|-
| 6 || IO || I/O Mask || I/O Interrupt Enable.
|-
| 7 || EX || External Mask || External Interrupt Enable.
|-
| 8–11 || Key || Storage Key || 4-bit storage protection key.
|-
| 12 || — || Architecture Mode || Must be 0 in z/Architecture.
|-
| 13 || M || Machine Check || Machine Check Interrupt Enable.
|-
| 14 || W || Wait State || CPU Wait State.
|-
| 15 || P || Problem State || Problem State (0 = Supervisor, 1 = User).
|-
| 16–17 || AS || Address-Space Control || Primary (00), Secondary (01), AR (10), Home (11).
|-
| 18–19 || CC || Condition Code || Condition code.
|-
| 20–23 || PM || Program Mask || Arithmetic masks.
|-
| 24–30 || — || Reserved || Must be 0.
|-
| 31 || EA || Extended Addressing || Extended addressing mode bit.
|-
| 32 || BA || Basic Addressing || Basic addressing mode bit.
|-
| '''33–63''' || '''IA''' || '''Instruction Address''' || '''31-bit instruction address''' (Target PC).
|}

=== Behavior of <code>LPSW</code> in z/Architecture ===

When the <code>LPSW</code> (opcode `82`) instruction is executed in z/Architecture mode:
# Bits 0–32 of the 8-byte operand are loaded directly into bits 0–32 of the CPU's internal 64-bit PSW mask.
# Bits 33–63 of the internal PSW mask are set to 0.
# Bits 0–32 of the internal 64-bit instruction address are set to 0.
# Bits 33–63 of the 8-byte operand are loaded into bits 33–63 of the internal instruction address.

----

== 3. ESA/390 Mode PSW (8-Byte Legacy Format) ==

When the processor is running in '''ESA/390 mode''' (e.g. at initial IPL before sending a `SIGP` architecture-switch signal, or inside a 31-bit legacy virtual machine), the CPU uses the legacy 64-bit ESA/390 Extended Control (EC-mode) PSW.

=== ESA/390 PSW Layout ===

{| class="wikitable"
! Bits (MSB 0) !! Short !! Name !! Description
|-
| 0 || — || Reserved || Must be 0.
|-
| 1 || R || PER Mask || Program Event Recording.
|-
| 2–4 || — || Reserved || Must be 0.
|-
| 5 || T || DAT Mode || Dynamic Address Translation.
|-
| 6 || IO || I/O Mask || I/O Interrupt Enable.
|-
| 7 || EX || External Mask || External Interrupt Enable.
|-
| 8–11 || Key || Storage Key || 4-bit storage key.
|-
| '''12''' || '''A''' || '''Architecture Mode''' || '''Must be 1 in ESA/390 EC-mode'''.
|-
| 13 || M || Machine Check || Machine Check Mask.
|-
| 14 || W || Wait State || Wait State flag.
|-
| 15 || P || Problem State || Problem State flag.
|-
| 16–17 || AS || Address-Space Control || Primary (00), Secondary (01), AR (10), Home (11).
|-
| 18–19 || CC || Condition Code || Condition code.
|-
| 20–23 || PM || Program Mask || Arithmetic masks.
|-
| 24–31 || — || Reserved || Must be 0.
|-
| '''32''' || '''A''' || '''Addressing Mode''' || '''0 = 24-bit AMODE, 1 = 31-bit AMODE'''.
|-
| '''33–63''' || '''IA''' || '''Instruction Address''' || '''31-bit instruction address'''.
|}

=== Differences Between Short PSW and ESA/390 PSW ===

While both formats are 8 bytes long:
* '''Bit 12:''' Is <code>0</code> in z/Architecture Short PSW, but '''must be <code>1</code>''' in ESA/390 EC-mode.
* '''Bit 31:''' Is <code>EA</code> (Extended Addressing) in z/Arch Short PSW, but is a '''reserved 0 bit''' in ESA/390.
* '''Bit 32:''' Is <code>BA</code> (Basic Addressing) in z/Arch Short PSW, but represents the sole 31-bit addressing mode flag in ESA/390.

----

== Addressing Modes (AMODE) ==

In z/Architecture (both 16-byte and Short PSW), addressing modes are controlled by the pair of bits <code>EA</code> (Bit 31) and <code>BA</code> (Bit 32):

{| class="wikitable"
! EA (Bit 31) !! BA (Bit 32) !! Mode !! Address Bits Evaluated
|-
| 0 || 0 || '''24-bit addressing''' || Lower 24 bits (Bits 40–63)
|-
| 0 || 1 || '''31-bit addressing''' || Lower 31 bits (Bits 33–63)
|-
| 1 || 1 || '''64-bit addressing''' || Full 64 bits (Bits 0–63)
|-
| 1 || 0 || ''Invalid'' || Triggers a Specification Exception (`PGM_SPECIFICATION`).
|}

----

== Address Space Control (ASC) Modes ==

When DAT is enabled (Bit 5 = 1), bits 16–17 select the active translation mode:

{| class="wikitable"
! Bits 16–17 !! Mode !! Active ASCE Source !! Usage
|-
| <code>00</code> || '''Primary-space mode''' || Control Register 1 (CR1) || Standard user/kernel virtual address space.
|-
| <code>01</code> || '''Secondary-space mode''' || Control Register 7 (CR7) || Cross-memory communication.
|-
| <code>10</code> || '''Access-register (AR) mode''' || Access Registers (AR0–AR15) || Fine-grained multi-space addressing via ALETs.
|-
| <code>11</code> || '''Home-space mode''' || Control Register 13 (CR13) || Kernel / system-wide execution space.
|}

----

== Interruption Lowcore Vectors ==

In 64-bit z/Architecture, interruptions automatically save the current 16-byte PSW to an '''Old PSW''' field in the Lowcore (the Prefix Page, real offset `0x0000`), and load the execution context from the corresponding 16-byte '''New PSW''':

{| class="wikitable"
! Interruption Class !! Old PSW Offset !! New PSW Offset
|-
| '''Restart''' || <code>0x0120</code> || <code>0x01A0</code>
|-
| '''External''' || <code>0x0130</code> || <code>0x01B0</code>
|-
| '''Supervisor Call (SVC)''' || <code>0x0140</code> || <code>0x01C0</code>
|-
| '''Program Check (Exception)''' || <code>0x0150</code> || <code>0x01D0</code>
|-
| '''Machine Check''' || <code>0x0160</code> || <code>0x01E0</code>
|-
| '''Input/Output (I/O)''' || <code>0x0170</code> || <code>0x01F0</code>
|}

''(Note: In legacy ESA/390 31-bit mode, Old and New PSWs are 8 bytes each and reside at offsets `0x0018`–`0x0078`).''

----

== C Structures & Code Examples ==

=== C Definitions for All Three PSW Types ===

<syntaxhighlight lang="c">
#include <stdint.h>

/* 1. Standard 16-byte z/Architecture Extended PSW */
typedef struct __attribute__((packed, aligned(8))) {
    uint64_t mask;
    uint64_t addr;
} zpsw_t;

/* 2. 8-byte z/Architecture Short PSW (used by LPSW under z/Arch) */
typedef struct __attribute__((packed, aligned(8))) {
    uint32_t mask_flags; /* Bits 0..32: Control flags + EA/BA */
    uint32_t addr_31;    /* Bits 33..63: 31-bit address */
} short_psw_t;

/* 3. 8-byte legacy ESA/390 Mode PSW */
typedef struct __attribute__((packed, aligned(8))) {
    uint32_t mask_flags; /* Bits 0..31: Includes Bit 12 (EC mode = 1) */
    uint32_t addr_31;    /* Bit 32 = AMODE31 flag, Bits 33..63 = 31-bit address */
} esa390_psw_t;

/* Extended PSW Mask Constants (MSB 0) */
#define PSW_MASK_PER         (0x4000000000000000ULL)
#define PSW_MASK_DAT         (0x0400000000000000ULL)
#define PSW_MASK_IO          (0x0200000000000000ULL)
#define PSW_MASK_EXT         (0x0100000000000000ULL)
#define PSW_MASK_KEY(k)      (((uint64_t)((k) & 0x0F)) << 52)
#define PSW_MASK_MCHECK      (0x0004000000000000ULL)
#define PSW_MASK_WAIT        (0x0002000000000000ULL)
#define PSW_MASK_PROBLEM     (0x0001000000000000ULL)

/* Address Space Control Modes */
#define PSW_ASC_PRIMARY      (0x0000000000000000ULL)
#define PSW_ASC_SECONDARY    (0x0000800000000000ULL)
#define PSW_ASC_ACCREG       (0x0000400000000000ULL)
#define PSW_ASC_HOME         (0x0000C00000000000ULL)

/* 64-bit Addressing Mode (EA=1 | BA=1) */
#define PSW_MASK_EA          (0x0000000100000000ULL)
#define PSW_MASK_BA          (0x0000000080000000ULL)
#define PSW_MASK_AMODE64     (PSW_MASK_EA | PSW_MASK_BA)

/* Standard Pre-composed 64-bit Kernel Masks */
#define PSW_KERNEL_BOOT \
    (PSW_MASK_AMODE64 | PSW_ASC_PRIMARY)

#define PSW_KERNEL_RUNNING \
    (PSW_MASK_DAT | PSW_MASK_IO | PSW_MASK_EXT | PSW_MASK_MCHECK | PSW_MASK_AMODE64 | PSW_ASC_HOME)

#define PSW_USER_RUNNING \
    (PSW_MASK_PROBLEM | PSW_MASK_DAT | PSW_MASK_IO | PSW_MASK_EXT | PSW_MASK_MCHECK | PSW_MASK_AMODE64 | PSW_ASC_PRIMARY)
</syntaxhighlight>

=== Loading PSWs ===

==== Loading a 16-Byte Extended PSW (<code>LPSWE</code>) ====
<syntaxhighlight lang="c">
static inline __attribute__((noreturn)) void lpswe(const zpsw_t *psw) {
    __asm__ volatile (
        "lpswe %0\n"
        :
        : "Q" (*psw)
        : "cc", "memory"
    );
    __builtin_unreachable();
}
</syntaxhighlight>

==== Loading an 8-Byte Short PSW (<code>LPSW</code>) ====
<syntaxhighlight lang="c">
static inline __attribute__((noreturn)) void lpsw(const short_psw_t *psw) {
    __asm__ volatile (
        "lpsw %0\n"
        :
        : "Q" (*psw)
        : "cc", "memory"
    );
    __builtin_unreachable();
}
</syntaxhighlight>

=== Extracting Current PSW Mask (<code>EPSW</code>) ===
<syntaxhighlight lang="c">
static inline uint64_t epsw(void) {
    uint32_t hi, lo;
    __asm__ volatile (
        "epsw %0, %1\n"
        : "=d" (hi), "=d" (lo)
    );
    return ((uint64_t)hi << 32) | (uint64_t)lo;
}
</syntaxhighlight>

=== CPU Idle & Panic Idioms ===

==== Enabled Wait (Idle Loop) ====
<syntaxhighlight lang="c">
void cpu_idle(void) {
    zpsw_t wait_pswrest = {
        .mask direct = PSW_MASK_DAT | PSW_MASK_IO | PSW_MASK_EXT | 
                       PSW_MASK_MCHECK | PSW_MASK_WAIT | PSW_MASK_AMODE64 | PSW_ASC_HOME,
        .addr = (uint64_t)&&wake_up
    };

    lpswe(&wait_pswrest);

wake_up:
    return;
}
</syntaxhighlight>

==== Disabled Wait (Panic / Halt) ====
<syntaxhighlight lang="c">
__attribute__((noreturn)) void cpu_halt(uint64_t panic_code) {
    zpsw_t halt_psw = {
        .mask = PSW_MASK_WAIT | PSW_MASK_AMODE64,
        .addr = (panic_code != 0) ? panic_code : 0x0000000000DEAD00ULL
    };

    lpswe(&halt_psw);
}
</syntaxhighlight>

----

== See Also ==
* [[DAT]]
* [[Interrupts]]
* [https://www.ibm.com/support/pages/zarchitecture-principles-operation IBM z/Architecture Principles of Operation (SA22-7832)]

[[Category:S390]]
````

<a id="w004643"></a>

## W004643 — PSA Mapping

Bron-ID: W004643
Namespace: Main (0)
Revisie: 31009 | Datum: 2026-10-10T08:18:52Z
Bron: https://wiki.osdev.org/PSA_Mapping
Revisie-URL: https://wiki.osdev.org/index.php?title=PSA_Mapping&oldid=31009
Doorverwijzing: Lowcore and Prefixing

````text
#REDIRECT [[Lowcore_and_Prefixing]]
````

<a id="w004644"></a>

## W004644 — Memory Map (S390)

Bron-ID: W004644
Namespace: Main (0)
Revisie: 30894 | Datum: 2026-07-14T22:02:15Z
Bron: https://wiki.osdev.org/Memory_Map_(S390)
Revisie-URL: https://wiki.osdev.org/index.php?title=Memory_Map_%28S390%29&oldid=30894

````text
The memory map on the S390 is very simple, the first 1024 (or 8192, if on z/Arch) bytes are used as [[PSA]] area. The [[IPL]] is loaded just after that said PSA.

== Detecting memory ==
<syntaxhighlight lang="c">
/* We are going to read in pairs of 1MiB and when we hit the memory limit we
 * will instantly catch the program exception and stop counting, then it's just
 * a matter of returning what we could count :) */
size_t s390_get_memsize(
    void)
{
    const uint8_t *probe = (const uint8_t *)0x0;
    while(1) {
        int r;
        /* Do a "probe" read */
        r = s390_address_is_valid(probe);
        if(r != 0) {
            kprintf("Done! %p\n", (uintptr_t)probe);
            break;
        }

        kprintf("Memory %p\n", (uintptr_t)probe);

        /* Go to next MiB */
        probe += 1048576;
    }
    return (size_t)probe;
}
</syntaxhighlight>

Now there needs to be a function to "catch" the exceptions recognized by the processor:
<syntaxhighlight lang="c">
/* Check if an address is valid - this only catches program exceptions to
 * determine if it's valid or not */
int s390_address_is_valid(
    volatile const void *probe)
{
    int r = 0;

    S390_PSW_DEFAULT_TYPE old_pc_psw;
#if (MACHINE >= M_ZARCH)
    struct s390x_psw pc_psw = {
        0x00040000 | S390_PSW_AM64, S390_PSW_DEFAULT_AMBIT, 0,
        (uint32_t)&&invalid
    };
#else
    struct s390_psw pc_psw = {
        0x000C0000, (uint32_t)&&invalid + S390_PSW_DEFAULT_AMBIT
    };
#endif

#if (MACHINE >= M_ZARCH)
    memcpy(&old_pc_psw, (void *)S390_FLCEPNPSW, sizeof(struct s390x_psw));
    memcpy((void *)S390_FLCEPNPSW, &pc_psw, sizeof(struct s390x_psw));
#else
    memcpy(&old_pc_psw, (void *)S390_FLCPNPSW, sizeof(struct s390_psw));
    memcpy((void *)S390_FLCPNPSW, &pc_psw, sizeof(struct s390_psw));
#endif

    *((volatile const uint8_t *)probe);
    goto end;
invalid:
    r = -1;
end:
#if (MACHINE >= M_ZARCH)
    memcpy((void *)S390_FLCEPNPSW, &old_pc_psw, sizeof(struct s390x_psw));
#else
    memcpy((void *)S390_FLCPNPSW, &old_pc_psw, sizeof(struct s390_psw));
#endif
    return r;
}
</syntaxhighlight>

Some C compilers are known to crash or even ABEND during compilation if addresses of labels are taken via the && operator, alternatively using direct high-level assembler syntax:

<syntaxhighlight lang="asm">
* HwCheckAddress
* IN:
*    pointer to address to probe
*
         ENTRY @ZHWCHKA
@ZHWCHKA DS 0H
         SAVE (14,12),,@ZHWCHKA
         LR R12,R15
         USING @ZHWCHKA,12
         LR R11,R1
* CATCHPSW address on R1
         L R1,=A(CATCHPSW)
* Save address of TMPSAVE on R2
         L R2,=A(TMPSAVE)
* And FLCPNPSW on R3
         L R3,FLCPNPSW
*
         MVC 0(8,R2),0(R3)
* ... use a new PSW to catch the exceptions
         MVC 0(8,R3),0(R1)
* Probe the address, if it raises a PC exception then
* we will simply catch it and return 1
         L R1,0(R11)
         L R15,0(R1)
* rc = 0
         MVC 0(8,R3),0(R2)
         L R15,=F'0'
         RETURN (14,12),RC=(15)
CATCHPCR DS 0H
* rc = 1
         MVC 0(8,R3),0(R2)
         L R15,=F'1'
         RETURN (14,12),RC=(15)
         LTORG
         DROP 12
*
CATCHPSW DS 0D
         DC X'020E0000'
         DC A(AMBIT+CATCHPCR)
TMPSAVE  DS 1D
</syntaxhighlight>

== See also ===
=== Source code ===
* [https://github.com/udos-project/UDOS/blob/3f446f4117d78fddf181f1df2927a9437fb08035/kernel/s390/cpu.c#L51 Example implementation (C)]

[[Category:S390]]
````

<a id="w004645"></a>

## W004645 — PSA

Bron-ID: W004645
Namespace: Main (0)
Revisie: 26506 | Datum: 2021-10-15T19:03:28Z
Bron: https://wiki.osdev.org/PSA
Revisie-URL: https://wiki.osdev.org/index.php?title=PSA&oldid=26506
Doorverwijzing: PSA Mapping

````text
#REDIRECT[[PSA Mapping]]
````

<a id="w004646"></a>

## W004646 — File:64-bit page tables1.png

Bron-ID: W004646
Namespace: File (6)
Revisie: 26521 | Datum: 2021-10-17T05:43:35Z
Bron: https://wiki.osdev.org/File:64-bit_page_tables1.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3A64-bit_page_tables1.png&oldid=26521

````text
The structure of page map table entries (non-page-sized) for long mode paging.
````

<a id="w004647"></a>

## W004647 — File:64-bit page tables2.png

Bron-ID: W004647
Namespace: File (6)
Revisie: 26522 | Datum: 2021-10-17T05:45:16Z
Bron: https://wiki.osdev.org/File:64-bit_page_tables2.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3A64-bit_page_tables2.png&oldid=26522

````text
The structure for page-sized page map table entries for paging in long mode.
````

<a id="w004648"></a>

## W004648 — Exception

Bron-ID: W004648
Namespace: Main (0)
Revisie: 26525 | Datum: 2021-10-17T08:18:09Z
Bron: https://wiki.osdev.org/Exception
Revisie-URL: https://wiki.osdev.org/index.php?title=Exception&oldid=26525
Doorverwijzing: Exceptions

````text
#REDIRECT [[Exceptions]]
````

<a id="w004649"></a>

## W004649 — Compatibility Mode

Bron-ID: W004649
Namespace: Main (0)
Revisie: 26528 | Datum: 2021-10-17T08:59:11Z
Bron: https://wiki.osdev.org/Compatibility_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Compatibility_Mode&oldid=26528
Doorverwijzing: X86-64

````text
#REDIRECT [[x86-64#Are_there_restrictions_on_32-bit_code_running_in_Legacy_Mode_.3F]]
````

<a id="w004650"></a>

## W004650 — FLATBOOT Barebones

Bron-ID: W004650
Namespace: Main (0)
Revisie: 30280 | Datum: 2026-03-16T17:59:26Z
Bron: https://wiki.osdev.org/FLATBOOT_Barebones
Revisie-URL: https://wiki.osdev.org/index.php?title=FLATBOOT_Barebones&oldid=30280

````text
The S/390 architecture is one of the most interesting ones for kernels, since it involves concepts not seen on standard small computer architectures, such as a dedicated processor for I/O or failure-prone checks.

== Pre-requisites ==
* [https://hercules-390.github.io/html/ Hercules] emulator
* [https://github.com/hercules-390/hyperion Hercules Github Repo]
* [[GCC_Cross-Compiler|GCC ESA S/390 Cross-Compiler]] (s390-linux target)

== Sources ==
The [https://github.com/udos-project/flatboot Flatboot] bootloader is an easy starting point for starting OS-development on the S/390. It partially-implements the [[stivale]] protocol (but it is not fully compliant). Supports ELF loading and BIN loading as well. In general; in the mainframe world having a general purpouse bootloader is not practical, since most IPL bootloaders are dependant on the device they loaded on. Flatboot wastes precious amounts of memory (64KB) which is a lot on the S/360 and the S/370. But it provides an easy starting point for newcomers.

=== kernel.c ===
The central controlling part of the system (runs in an LPAR for any S/390 or z/Arch system):
<syntaxhighlight lang="c">
// Source: https://github.com/limine-bootloader/limine-barebones/blob/master/stivale2-barebones/kernel/kernel.c
#include <stdint.h>
#include <stddef.h>

#define _STIVALE2_SPLIT_64_FORCE 1
#include <stivale2.h>

static uint8_t stack[8192];
static struct stivale2_header_tag_terminal terminal_hdr_tag = {
    .tag = {
        .identifier = STIVALE2_HEADER_TAG_TERMINAL_ID,
        .next = 0
    },
    .flags = 0
};

__attribute__((section(".stivale2hdr"), used))
static struct stivale2_header stivale_hdr = {
    .entry_point = 0,
    .stack = (uintptr_t)stack + sizeof(stack),
    .flags = (1 << 1) | (1 << 2),
    .tags = (uintptr_t)&terminal_hdr_tag
};

// We will now write a helper function which will allow us to scan for tags
// that we want FROM the bootloader (structure tags).
void *stivale2_get_tag(struct stivale2_struct *stivale2_struct, uint64_t id) {
    struct stivale2_tag *current_tag = (void *)stivale2_struct->tags;
    for (;;) {
        // If the tag pointer is NULL (end of linked list), we did not find
        // the tag. Return NULL to signal this.
        if (current_tag == NULL) {
            return NULL;
        }

        // Check whether the identifier matches. If it does, return a pointer
        // to the matching tag.
        if (current_tag->identifier == id) {
            return current_tag;
        }

        // Get a pointer to the next tag in the linked list and repeat.
        current_tag = (void *)current_tag->next;
    }
}

// The following will be our kernel's entry point.
void _start(struct stivale2_struct *stivale2_struct) {
    // Let's get the terminal structure tag from the bootloader.
    struct stivale2_struct_tag_terminal *term_str_tag;
    term_str_tag = stivale2_get_tag(stivale2_struct, STIVALE2_STRUCT_TAG_TERMINAL_ID);

    // Check if the tag was actually found.
    if (term_str_tag == NULL) {
        // It wasn't found, just hang...
        for (;;);
    }

    // Let's get the address of the terminal write function.
    void *term_write_ptr = (void *)term_str_tag->term_write;

    // Now, let's assign this pointer to a function pointer which
    // matches the prototype described in the stivale2 specification for
    // the stivale2_term_write function.
    void (*term_write)(const char *string, size_t length) = term_write_ptr;
    term_write("Hello World\n", 13);
    for (;;);
}
</syntaxhighlight>

=== linker.ld ===
The kernel needs to be loaded above 0x20000 due to Flatboot's strict no-low-64K rule for kernels.
<pre>
ENTRY(_start);
SECTIONS {
    . = 0x20000;

    .text : {
        *(.text);
        *(.text.*);
        *(.stivale2hdr*);
    }
    
    .rodata : {
        *(.rodata);
        *(.rodata.*);
    }

    .data : {
        *(.data);
        *(.data.*);
    }

    .bss : ALIGN(4K) {
        *(COMMON);
        *(.bss);
        *(.bss.*);
    }
}
</pre>

=== hercules.cnf ===
The configuration file for Hercules, enables diagnostic messages, sets the architecture mode to ESA S/390 and uses the IBM-1047 codepage (EBCDIC)
<pre>
MAINSIZE             32
ARCHMODE             ESA/390
CPUSERIAL            000611
CPUMODEL             4381
DIAG8CMD             ENABLE
NUMCPU               1 
PANRATE              MEDIUM
PGMPRDOS             RESTRICTED
CODEPAGE             819/1047

# This is the IPL disk and can be any address
01b9      3390       flat00.cckd
</pre>

=== dasd.txt ===
Describe the files in the DASD disk, this is needed since [[ZDSFS]] has an unique way to order datasets (files) - which includes support for text-files (70-columns) or binaries.
<pre>
FLAT00       3390-1 * stage1.txt
STAGE2.BIN   SEQ stage2.bin          TRK 10 1 0 PS FB 1 18452
SYSVTOC      VTOC                    CYL  2
KERNEL.ELF   SEQ kernel              CYL  1 1 0 PS FB 1 18452
</pre>

== Running the mainframe ==
<syntaxhighlight lang="bash">
#!/usr/bin/bash

# Download headers required for the stivale2 structs
rm stivale2.h
wget https://raw.githubusercontent.com/stivale/stivale/master/stivale2.h

# Download the disk-dependant IPL
wget https://github.com/udos-project/flatboot/releases/download/0.1.2/stage1_S390.txt
mv stage1_S390.txt stage1.txt

# Download the second stage bootloader
wget https://github.com/udos-project/flatboot/releases/download/0.1.2/stage2_S390.bin
mv stage2_S390.bin stage2.bin

CC=s390-linux-gcc
CFLAGS="-fexec-charset=IBM-1047 -fno-stack-protector -pipe -m31 -ffreestanding -mpacked-stack -I. -O2 -g"
LDFLAGS="-Wl,-static,--no-dynamic-linker,-ztext -nostdlib -z max-page-size=0x1000 -static -fno-pic -fno-pie -g"

$CC $CFLAGS -c kernel.c -o kernel.o
$CC $LDFLAGS $CFLAGS -Tlinker.ld kernel.o -o kernel

# Create the DASD disk
dasdload -bz2 dasd.txt flat00.cckd

# Fire up the emulator and concat to a log file
hercules -f hercules.cnf >hercules.log
</syntaxhighlight>

Once the hercules screen appears, the kernel can be loaded with:
<pre>
IPL 1B9
</pre>

== What's next? ==
* Use MVS 3.8/tk4 in hercules to know how a mainframe operates
* Read [https://udos-project.github.io/ this]
* Catch program exceptions
* Get the CSS to be able to do read/write operations on devices
* There is no framebuffer to speak of, no GUI, only terminals - so implement a [[3270]] (Graphical terminal) driver or 2703 (Telnet)
* Implement multiprocessing

[[Category:S390]]
[[Category:Stivale]]
[[Category:Bare bones tutorials]]
````

<a id="w004651"></a>

## W004651 — Control Register

Bron-ID: W004651
Namespace: Main (0)
Revisie: 26549 | Datum: 2021-10-20T05:30:22Z
Bron: https://wiki.osdev.org/Control_Register
Revisie-URL: https://wiki.osdev.org/index.php?title=Control_Register&oldid=26549
Doorverwijzing: CPU Registers x86

````text
#REDIRECT [[CPU_Registers_x86#Control_Registers]]
````

<a id="w004652"></a>

## W004652 — Control registers

Bron-ID: W004652
Namespace: Main (0)
Revisie: 26550 | Datum: 2021-10-20T05:32:06Z
Bron: https://wiki.osdev.org/Control_registers
Revisie-URL: https://wiki.osdev.org/index.php?title=Control_registers&oldid=26550
Doorverwijzing: CPU Registers x86

````text
#REDIRECT [[CPU_Registers_x86#Control_Registers]]
````

<a id="w004653"></a>

## W004653 — Cr2

Bron-ID: W004653
Namespace: Main (0)
Revisie: 26554 | Datum: 2021-10-20T06:06:32Z
Bron: https://wiki.osdev.org/Cr2
Revisie-URL: https://wiki.osdev.org/index.php?title=Cr2&oldid=26554
Doorverwijzing: CPU Registers x86

````text
#REDIRECT[[CPU_Registers_x86#CR2]]
````

<a id="w004654"></a>

## W004654 — Cr3

Bron-ID: W004654
Namespace: Main (0)
Revisie: 26555 | Datum: 2021-10-20T06:06:53Z
Bron: https://wiki.osdev.org/Cr3
Revisie-URL: https://wiki.osdev.org/index.php?title=Cr3&oldid=26555
Doorverwijzing: CPU Registers x86

````text
#REDIRECT [[CPU_Registers_x86#CR3]]
````

<a id="w004655"></a>

## W004655 — Cr8

Bron-ID: W004655
Namespace: Main (0)
Revisie: 26556 | Datum: 2021-10-20T06:07:28Z
Bron: https://wiki.osdev.org/Cr8
Revisie-URL: https://wiki.osdev.org/index.php?title=Cr8&oldid=26556
Doorverwijzing: CPU Registers x86

````text
#REDIRECT [[CPU_Registers_x86#CR8]]
````

<a id="w004656"></a>

## W004656 — Xcr0

Bron-ID: W004656
Namespace: Main (0)
Revisie: 26558 | Datum: 2021-10-20T06:09:10Z
Bron: https://wiki.osdev.org/Xcr0
Revisie-URL: https://wiki.osdev.org/index.php?title=Xcr0&oldid=26558
Doorverwijzing: CPU Registers x86

````text
#REDIRECT [[CPU_Registers_x86#XCR0]]
````

<a id="w004657"></a>

## W004657 — QEMU AArch64 Virt Bare Bones

Bron-ID: W004657
Namespace: Main (0)
Revisie: 30865 | Datum: 2026-07-07T17:30:01Z
Bron: https://wiki.osdev.org/QEMU_AArch64_Virt_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=QEMU_AArch64_Virt_Bare_Bones&oldid=30865

````text
[[QEMU]] is a free and open source hardware emulator that supports multiple architecture, including [[ARM Overview|ARM]]. QEMU has its own generic board mainly for virtualization usages, called virt. virt supports ARM and [[RISC-V]] architecture.

In this tutorial, we'll write a basic kernel for QEMU virt board, specifically Aarch64.

== Prerequisites ==
For this tutorial, you'll need
* An ARM cross-compiler toolchain
* QEMU, of course you'll need QEMU because this tutorial designed for QEMU virt.

== Code ==
Now we'll need to write some code for our little kernel.

=== boot.s ===
<syntaxhighlight lang="asm">
.global _start
_start:
    ldr x30, =stack_top
    mov sp, x30
    bl kmain
    b .
</syntaxhighlight>

=== kernel.c ===
<syntaxhighlight lang="c">
#include <stdint.h>

volatile uint8_t *uart = (uint8_t *) 0x09000000;

void putchar(char c) {
    *uart = c;
}

void print(const char *s) {
    while(*s != '\0') {
        putchar(*s);
        s++;
    }
}

void kmain(void) {
     print("Hello world!\n");
}
</syntaxhighlight>

=== linker.ld ===
<syntaxhighlight lang="c">
ENTRY(_start)
SECTIONS {
    . = 0x40100000; /* RAM starts at 0x40000000 but if we ask to load the kernel there, QEMU will not load a DTB */
    .startup . : { boot.o(.text) }
    .text : { *(.text) }
    .data : { *(.data) }
    .bss : { *(.bss COMMON) }
    . = ALIGN(16); /* two times pointer size */
    . += 0x1000; /* 4kB of stack memory */
    stack_top = .;
}
</syntaxhighlight>

== Compiling the kernel ==
Now, we'll need to compile our kernel so we can boot it.
<syntaxhighlight lang="shell-session">
$ aarch64-elf-as boot.s -o boot.o
$ aarch64-elf-gcc -ffreestanding -c kernel.c -o kernel.o
$ aarch64-elf-ld -nostdlib -Tlinker.ld boot.o kernel.o -o kernel.elf
</syntaxhighlight>

== Booting our kernel ==
Since our kernel is now compiled, we'll boot it with QEMU
<syntaxhighlight lang="shell-session">
$ qemu-system-aarch64 -machine virt -cpu cortex-a57 -kernel kernel.elf -nographic
Hello world!
</syntaxhighlight>

== See also ==
* [[ARM Overview]]
* [[Raspberry Pi Bare Bones]]

[[Category:ARM]]
[[Category:AArch64]]
[[Category:Bare bones tutorials]]
[[Category:QEMU]]
````

<a id="w004658"></a>

## W004658 — PDE

Bron-ID: W004658
Namespace: Main (0)
Revisie: 26569 | Datum: 2021-10-21T16:48:43Z
Bron: https://wiki.osdev.org/PDE
Revisie-URL: https://wiki.osdev.org/index.php?title=PDE&oldid=26569
Doorverwijzing: Paging

````text
#REDIRECT [[Paging#Page_Directory]]
````

<a id="w004659"></a>

## W004659 — PTE

Bron-ID: W004659
Namespace: Main (0)
Revisie: 26570 | Datum: 2021-10-21T16:50:06Z
Bron: https://wiki.osdev.org/PTE
Revisie-URL: https://wiki.osdev.org/index.php?title=PTE&oldid=26570
Doorverwijzing: Paging

````text
#REDIRECT [[Paging#Page_Table]]
````

<a id="w004660"></a>

## W004660 — File:Cyjon v0.1720.png

Bron-ID: W004660
Namespace: File (6)
Revisie: 26581 | Datum: 2021-10-23T20:03:26Z
Bron: https://wiki.osdev.org/File:Cyjon_v0.1720.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACyjon_v0.1720.png&oldid=26581

````text

````

<a id="w004661"></a>

## W004661 — GPF

Bron-ID: W004661
Namespace: Main (0)
Revisie: 26594 | Datum: 2021-10-25T02:30:35Z
Bron: https://wiki.osdev.org/GPF
Revisie-URL: https://wiki.osdev.org/index.php?title=GPF&oldid=26594
Doorverwijzing: Exceptions

````text
#REDIRECT [[Exceptions#General Protection Fault]]
````

<a id="w004662"></a>

## W004662 — ARM RaspberryPi

Bron-ID: W004662
Namespace: Main (0)
Revisie: 26587 | Datum: 2021-10-24T10:48:56Z
Bron: https://wiki.osdev.org/ARM_RaspberryPi
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_RaspberryPi&oldid=26587
Doorverwijzing: Raspberry Pi

````text
#REDIRECT [[Raspberry Pi]]
````

<a id="w004663"></a>

## W004663 — Talk:ARM RaspberryPi

Bron-ID: W004663
Namespace: Talk (1)
Revisie: 26589 | Datum: 2021-10-24T10:48:56Z
Bron: https://wiki.osdev.org/Talk:ARM_RaspberryPi
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AARM_RaspberryPi&oldid=26589
Doorverwijzing: Talk:Raspberry Pi

````text
#REDIRECT [[Talk:Raspberry Pi]]
````

<a id="w004664"></a>

## W004664 — Gpf

Bron-ID: W004664
Namespace: Main (0)
Revisie: 29868 | Datum: 2026-01-12T02:17:04Z
Bron: https://wiki.osdev.org/Gpf
Revisie-URL: https://wiki.osdev.org/index.php?title=Gpf&oldid=29868
Doorverwijzing: Exceptions

````text
#REDIRECT [[Exceptions#General_Protection_Fault]]
````

<a id="w004666"></a>

## W004666 — File:FuzzyOS.gif

Bron-ID: W004666
Namespace: File (6)
Revisie: 26604 | Datum: 2021-10-25T20:40:07Z
Bron: https://wiki.osdev.org/File:FuzzyOS.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AFuzzyOS.gif&oldid=26604

````text
https://github.com/scopeInfinity/FuzzyOS
````

<a id="w004667"></a>

## W004667 — User:0nera

Bron-ID: W004667
Namespace: User (2)
Revisie: 27598 | Datum: 2023-01-17T10:24:28Z
Bron: https://wiki.osdev.org/User:0nera
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3A0nera&oldid=27598

````text
Hello Everyone!
I write my OS https://github.com/0nera/SynapseOS

The useful links:
````

<a id="w004669"></a>

## W004669 — Stivale2 CSharp BareBones

Bron-ID: W004669
Namespace: Main (0)
Revisie: 27028 | Datum: 2022-04-03T17:43:07Z
Bron: https://wiki.osdev.org/Stivale2_CSharp_BareBones
Revisie-URL: https://wiki.osdev.org/index.php?title=Stivale2_CSharp_BareBones&oldid=27028

````text
{{BeginnersWarning}}
{{Rating|2}}
==What is Stivale?==

'''stivale''' means "boot" in Italian. It is a boot protocol designed to overcome shortcomings of common boot protocols used by hobbyist OS developers, such as [[Multiboot]].

There are 2 revisions of the stivale boot protocol, namely: '''stivale''', and '''stivale2'''. Stivale2 makes use of tags for bootloader writers' and kernel writers' convenience, and to make future expandability and revisioning easier. We will use second version.

[https://github.com/stivale/stivale Stivale GitHub repository]

==What is Limine?==
[[Limine]] is a modern, advanced x86 bootloader for [[BIOS]] and [[UEFI]], with support for cutting edge features such as 5-level paging, 64-bit [[Long Mode]], and direct higher half loading thanks to the [[stivale]] boot protocol.

[https://github.com/limine-bootloader/limine Limine GitHub repository]

==Getting started==
It is recommended to check out this project [https://github.com/ilobilo/stivale2-cs-barebones repository] until getting started
===Install required packages===
To compile and run the kernel you will need following programs installed:
* [https://docs.microsoft.com/en-us/dotnet/core/install/linux Dotnet] (C# compiler) should be installed in /usr/share/dotnet/
* Tysila2 (Debian package included in repository)
* [https://www.mono-project.com Mono] (To run tysila2)
* Make (To run everything)
* Nasm (Assembly compiler)
* ld.lld (Linker)
* Xorriso (To build the iso)
* Qemu-system-x86 (To run the OS)
If you are using debian based system you can install most of them with these commands:

 sudo apt install nasm ld.lld xorriso qemu-system-x86 mono-runtime
 // First download tysila2.deb from project's repository
 sudo dpkg -i tysila2.deb

===Actual code===
First of all you need to create following files and directories:

 ├── limine.cfg
 ├── Makefile
 └── source
     ├── Console.cs
     ├── kernel.asm
     ├── kernel.cs
     ├── linker.ld
     ├── Makefile
     └── stivale2.cs

====limine.cfg====
This is [[Limine]] configuration file which tells bootloader what to do:

limine.cfg
 TIMEOUT=0
 VERBOSE=yes
 SERIAL=yes
 
 # Kernel entry name in bootloader menu
 :Stivale2
 # Boot protocol to use
 PROTOCOL=stivale2
 # Kernel elf file path on iso
 KERNEL_PATH=boot:///kernel.elf
 # Disable KASLR
 KASLR=no

====Makefile====
These are simple makefile that will be excecuted when you run make command:

Note: Make sure to use tabs instead of 8 spaces

Makefile:
<source lang="make">
KERNELDIR := $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

all: limine
	mkdir -p $(KERNELDIR)/iso_root
	$(MAKE) -s -C $(KERNELDIR)/source

limine:
	git clone https://github.com/limine-bootloader/limine.git --branch=v2.0-branch-binary --depth=1
	$(MAKE) -C $(KERNELDIR)/limine

clean:
	$(MAKE) -s -C $(KERNELDIR)/source clean

distclean:
	$(MAKE) -s -C $(KERNELDIR)/source clean
	rm -rf $(KERNELDIR)/limine
	rm -rf $(KERNELDIR)/iso_root
</source>

source/Makefile:
<source lang="make">
SOURCEDIR := $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

KERNEL := $(SOURCEDIR)/kernel.elf
KERNELEXE := $(KERNEL:.elf=.exe)
KERNELO := $(KERNEL:.elf=.o)
ISO = $(SOURCEDIR)/../image.iso

LIMINE = $(SOURCEDIR)/../limine/limine-install

QEMU = qemu-system-x86_64
QEMUFLAGS = -enable-kvm -M q35 -cpu max -smp 2 -m 512M -boot d -rtc base=localtime -serial stdio

XORRISO = xorriso
XORRISOFLAGS = -as mkisofs -b limine-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot limine-eltorito-efi.bin -efi-boot-part \
		--efi-boot-image --protective-msdos-label

LD = ld.lld
AS = nasm
AOT = tysila2
DOTNET = /usr/share/dotnet/dotnet
CSC = /usr/share/dotnet/sdk/*/Roslyn/bincore/csc.dll

ASMFLAGS = -f elf64
AOTFLAGS = --arch x86_64-elf64-tysos -fno-rtti -fno-exceptions
CSCFLAGS = -unsafe -target:exe -platform:x86 -nostdlib /r:/usr/share/tysila2/mscorlib.dll

LDFLAGS = -T $(SOURCEDIR)/linker.ld -m elf_x86_64 -z max-page-size=0x1000

CSFILES = $(shell find $(SOURCEDIR)/ -type f -name '*.cs')
ASMFILES = $(shell find $(SOURCEDIR)/ -type f -name '*.asm')
OBJ = $(ASMFILES:.asm=_asm.o)

.PHONY: all
all: $(KERNEL)
	$(MAKE) iso
	$(MAKE) clean run

$(KERNEL): $(OBJ)
	$(DOTNET) $(CSC) $(CSCFLAGS) $(CSFILES) -out:$(KERNELEXE)
	$(AOT) $(AOTFLAGS) $(KERNELEXE) -o $(KERNELO)
	$(LD) $(LDFLAGS) $(INTERNALLDFLAGS) $(OBJ) $(KERNELO) -o $@

%_asm.o: %.asm
	$(AS) $(ASMFLAGS) $^ -o $@

iso:
	cp $(KERNEL) $(SOURCEDIR)/../limine.cfg $(SOURCEDIR)/../limine/limine.sys \
		$(SOURCEDIR)/../limine/limine-cd.bin $(SOURCEDIR)/../limine/limine-eltorito-efi.bin $(SOURCEDIR)/../iso_root/
	$(XORRISO) $(XORRISOFLAGS) $(SOURCEDIR)/../iso_root -o $(ISO)
	$(LIMINE) $(ISO)

clean:
	rm -rf $(KERNEL) $(OBJ) $(KERNELEXE) $(KERNELO) $(SOURCEDIR)/../iso_root/*

run:
	$(QEMU) $(QEMUFLAGS) -cdrom $(ISO)
</source>
====Linker script====
Script that ld.lld will use to link relocatable elf objects

source/linker.ld
<source lang="lscript">
OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)

ENTRY(_start)

PHDRS
{
    null    PT_NULL    FLAGS(0) ;
    text    PT_LOAD    FLAGS((1 << 0) | (1 << 2)) ;
    rodata  PT_LOAD    FLAGS((1 << 2)) ;
    data    PT_LOAD    FLAGS((1 << 1) | (1 << 2)) ;
    dynamic PT_DYNAMIC FLAGS((1 << 1) | (1 << 2)) ;
}

SECTIONS
{
    . = 0xffffffff80200000;

    .text : {
        *(.text*)
    } :text

    . += 0x1000;

    .stivale2hdr : {
        KEEP(*(.stivale2hdr))
    } :rodata

    .rodata : {
        *(.rodata*)
    } :rodata

    . += 0x1000;

    .data : {
        *(.data*)
    } :data

    .dynamic : {
        *(.dynamic)
    } :data :dynamic

    .bss : {
        *(COMMON)
        *(.bss*)
    } :data
}
</source>

====Kernel====
These files contain code that will be executed when kernel runs:

source/kernel.asm
<source lang="asm">
[EXTERN _ZN6kernel6Kernel7ProgramM_0_8RealMain_Rv_P1PV26stivale2#2Bstivale2_struct]

section .data
stivale2_smp_tag:
    dq 0x1ab015085f3273df
    dq 0
    dq 0

stivale2_any_video_tag:
    dq 0xc75c9fa92a44c4db
    dq stivale2_smp_tag
    dq 1

[SECTION .bss]
align 16
stack_bottom:
resb 8192
stack_top:

[SECTION .stivale2hdr]
align 4
stivale_hdr:
    dq kmain
    dq stack_top
    dq (1 << 1)
    dq stivale2_any_video_tag

[SECTION .text]
kmain:
    push rdi
    call _ZN6kernel6Kernel7ProgramM_0_8RealMain_Rv_P1PV26stivale2#2Bstivale2_struct

sthrow:
    hlt
    jmp sthrow
[GLOBAL sthrow]

_ZN6kernel6Kernel7ProgramM_0_4halt_Rv_P0:
    hlt
    jmp _ZN6kernel6Kernel7ProgramM_0_4halt_Rv_P0
[GLOBAL _ZN6kernel6Kernel7ProgramM_0_4halt_Rv_P0]
</source>

source/kernel.cs
<source lang="csharp">
using System.Runtime.CompilerServices;

namespace Kernel
{
    // Class is unsafe so we can use pointers
    public unsafe class Program
    {
        // Fake kernel entry point
        // Do not remove
        public static void Main()
        {
            // These line is required
            // Without it compiler will remove RealMain from kernel
            RealMain(null);
        }

        public static stivale2.stivale2_struct_tag_smp *smp_tag;

        [MethodImpl(MethodImplOptions.InternalCall)]
        public static extern void halt();

        // Real entry
        public static void RealMain(stivale2.stivale2_struct* stiv)
        {
            // If Stivale2 struct was not found halt
            if (stiv == null) halt();

            // Example on how to get Stivale2 structure tags
            smp_tag = (stivale2.stivale2_struct_tag_smp*)stivale2.get_tag(stiv, stivale2.STIVALE2_STRUCT_TAG_SMP_ID);

            // Print text
            Console.WriteLine("Hello, World!");

            // Halt
            halt();
        }
    }
}
</source>
====Console====
This is an example code that provides console write and writeline functions:
source/Console.cs
<source lang="csharp">
namespace Kernel
{
    // Console colours
    public enum ConsoleColour
    {
        Black,
        Blue,
        Green,
        Cyan,
        Red,
        Purple,
        Brown,
        Grey,
        DarkGrey,
        LightBlue,
        LightGreen,
        LightCyan,
        LightRed,
        LightPurple,
        Yellow,
        White,
    };
    public unsafe static class Console
    {
        // VGA text mode address
        public static ushort* vga = (ushort*)0xB8000;
        // Cursor positions
        public static int x = 0, y = 0, lastx = 0;
        // Text colours
        public static ConsoleColour ForegroundColour = ConsoleColour.White;
        public static ConsoleColour BackgroundColour = ConsoleColour.Black;

        public static void Write(string s)
        {
            foreach (char c in s)
            {
                PutChar(c, BackgroundColour, ForegroundColour);
            }
        }

        public static void WriteLine(string s)
        {
            foreach (char c in s)
            {
                PutChar(c, BackgroundColour, ForegroundColour);
            }
            PutChar('\n', BackgroundColour, ForegroundColour);
        }

        static void PutChar(char c, ConsoleColour bgcolour, ConsoleColour fgcolour)
        {
            switch (c)
            {
                // Newline
                case '\n':
                    lastx = x;
                    x = 0;
                    y++;
                    break;
                // Backspace
                case '\b':
                    if (x > 0)
                    {
                        x--;
                        vga[y * 80 + x] = (ushort)(((byte)bgcolour << 12) | ((byte)fgcolour << 8) | ' ');
                    }
                    else
                    {
                        x = lastx;
                        y--;
                    }
                    break;
                // Everything else
                default:
                    vga[y * 80 + x] = (ushort)(((byte)bgcolour << 12) | ((byte)fgcolour << 8) | c);
                    if (x < 80) x++;
                    else
                    {
                        lastx = x;
                        x = 0;
                        y++;
                    }
                    break;
            }
        }
    }
}
</source>

====Stivale2.cs====
Because file is too long its recommended to [https://github.com/ilobilo/stivale2-cs-barebones/blob/main/source/stivale2.cs download] it and place under source/

===Building and running the kernel===
To compile and run the kernel go to main directory (Where limine.cfg, Makefile and source/ are) and run:
 make -j$(nproc --all)

== See Also ==

=== Articles ===

* [[Limine]]

=== External Links ===

* [https://github.com/stivale/stivale/blob/master/STIVALE2.md Stivale2 Specification]
* [https://github.com/limine-bootloader/limine Limine bootloader]
* [https://github.com/ilobilo/stivale2-cs-barebones/ Project's github repository]

[[Category:Bare bones tutorials]]
[[Category:CSharp]]
[[Category:Stivale]]
````

<a id="w004670"></a>

## W004670 — D barebone with ldc2

Bron-ID: W004670
Namespace: Main (0)
Revisie: 26641 | Datum: 2021-11-02T00:27:19Z
Bron: https://wiki.osdev.org/D_barebone_with_ldc2
Revisie-URL: https://wiki.osdev.org/index.php?title=D_barebone_with_ldc2&oldid=26641
Doorverwijzing: D Bare Bones With LDC2

````text
#REDIRECT [[D Bare Bones With LDC2]]
````

<a id="w004671"></a>

## W004671 — Single Kernel Stack

Bron-ID: W004671
Namespace: Main (0)
Revisie: 26658 | Datum: 2021-11-04T00:07:57Z
Bron: https://wiki.osdev.org/Single_Kernel_Stack
Revisie-URL: https://wiki.osdev.org/index.php?title=Single_Kernel_Stack&oldid=26658
Doorverwijzing: Thread

````text
#REDIRECT [[Thread#Single_Kernel_Stack]]
````

<a id="w004672"></a>

## W004672 — Category:CSharp

Bron-ID: W004672
Namespace: Category (14)
Revisie: 27587 | Datum: 2023-01-14T06:08:51Z
Bron: https://wiki.osdev.org/Category:CSharp
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ACSharp&oldid=27587

````text
C# tutorials
[[category:languages]]
````

<a id="w004673"></a>

## W004673 — User:Mrwoggle123

Bron-ID: W004673
Namespace: User (2)
Revisie: 26676 | Datum: 2021-11-13T23:32:46Z
Bron: https://wiki.osdev.org/User:Mrwoggle123
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMrwoggle123&oldid=26676

````text
 
I'm mr_woggle (mrwoggle123 on these forums) If you like developing OSes, please link on: [https://github.com/robinkrens Github]. I think it is a great way to watch other people's progress!
__NOTOC__
== Philosophy ==
I like developing OSes, just because of the mere beauty of it!
I like to keep things simple.
By simple I don't mean that I avoid setting up a MMU or somehow desire that my OS runs in real mode. 
By simple I mean that I don't want to overly rely on IDEs, bloated toolchains, complex libraries and
powerful bootloaders. 

X86 is not the only architecture. Unix is not the only way. Why start with GDT and paging, when you can start off with much simpler and fun stuff?

== Stuff I use and know about ==

* C & Assembly
* Simple makefiles 
* VIM editor

== Projects ==

=== Xinu for STM32 ===
Xinu for STM32 is a port of the arm version of Xinu. Xinu is a small, elegant operating system that supports dynamic process creation, dynamic memory allocation, network communication, local and remote file systems, a shell, and device-independent I/O functions. The small size makes Xinu suitable for embedded environments. <br />
Github: [https://github.com/real-xinu/xinu-stm32 Xinu for STM32]

=== Cortex-M3 from scratch ===
The name says it all: everything from scratch. <br />
Github: [https://github.com/robinkrens/cortex-from-scratch Cortex-M3 from scratch]

== Typedef ==
As you probably can tell, I'm a Lino Commando kind of guy [https://wiki.osdev.org/What_order_should_I_make_things_in]. If I see $ or # or > I'm good to go. I'm not only interested in x86, but also in ARM and RISC-V.
As of late I took an interest in the low powered MCUs. I somehow trying to convert my i386 knowledge to the ARM platform. I have some shitty Chinese development boards.
````

<a id="w004675"></a>

## W004675 — Segment Selector

Bron-ID: W004675
Namespace: Main (0)
Revisie: 31019 | Datum: 2026-10-13T15:25:46Z
Bron: https://wiki.osdev.org/Segment_Selector
Revisie-URL: https://wiki.osdev.org/index.php?title=Segment_Selector&oldid=31019

````text
A '''Segment Selector''' is a 16-bit binary data structure specific to the [[IA32_Architecture_Family|IA-32]] and [[X86-64|x86-64]] architectures. It is used in '''[[Protected Mode]]''' and '''[[Long Mode]]'''. Its value identifies a segment in either the '''[[Global Descriptor Table]]''' or a '''[[Local Descriptor Table]]'''.  It contains three fields and is used in a variety of situations to interact with '''[[Segmentation]]'''.

For more information, see '''Section 3.4.2: Segment Selectors''' and '''Figure 3-6: Segment Selector''' of the Intel Software Developer Manual, Volume 3-A.

== Segment Selector ==

{| class="wikitable"
|+ Segment Selector
!style="width: 82%; text-align: left;" |15 &nbsp;&nbsp;&nbsp;<span style="float: right;">3</span>
!style="width: 6%; text-align: center; vertical-align: bottom;" |2
!style="width: 12%; text-align: left; vertical-align: bottom;" |1&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|-
|'''Index'''
|'''TI'''
|'''RPL'''
|}
* '''Index:''' Bits 3-15 of the Index of the '''GDT''' or '''LDT''' entry referenced by the selector. ''Since '''[[Global Descriptor Table#Segment_Descriptor|Segment Descriptors]]''''' ''are 8 bytes in length, the address calculated from the Index is always 8-byte aligned, and the lowest 3 bits of the Index are ignored by the hardware.''
* '''TI:''' Specifies which descriptor table to use. If clear ('''0''') then the '''GDT''' is used, if set ('''1''') then the current '''LDT''' is used.
* '''RPL:''' The requested '''[[Security#Rings|Privilege Level]]''' of the selector, determines if the selector is valid during permission checks and may set execution or memory access privilege.
For example, a segment selector that references the first entry in the '''GDT''' with ring zero privilege would be 0xF ('''Index''' = 8, '''TI''' = 0, '''RPL''' = 0).

== Uses ==

'''Segment Selectors''' are used in a few locations, including:
* '''[[Segmentation|Segmentation Registers]]'''
* The '''[[Interrupt Descriptor Table]]'''
* The '''[[Task State Segment]]'''

[[Category:X86]]
[[Category:Memory Segmentation]]
````

<a id="w004676"></a>

## W004676 — Talk:Languages

Bron-ID: W004676
Namespace: Talk (1)
Revisie: 26719 | Datum: 2021-11-29T04:49:03Z
Bron: https://wiki.osdev.org/Talk:Languages
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ALanguages&oldid=26719

````text
== Pilot OS inaccuracies ==
The article says that Pilot-OS was the operating system for the Xerox Alto and Star workstations, and that it was written mostly in Smalltalk-80. To my understanding, Pilot-OS was written in Mesa (a different object oriented language, described as being similar to Java), not Smalltalk, and ran only on the Xerox Star. The Alto ran a different unnamed operating system, which was written in BCPL and Assembly. Here are my sources for this: [https://xeroxalto.computerhistory.org/_cd8_/altodocs/.os.press!2.pdf Alto Operating System Reference Manual] [https://www.mcjones.org/paul/pilot/ Pilot] [http://www.bitsavers.org/pdf/xerox/mesa/Alto_Mesa_1974.pdf Alto_Mesa_1974.pdf]. I'm going to go ahead and change what the article says since I'm fairly confident that it's an inaccuracy. I'd also like it if someone could get me a citation on the claim that there have been operating systems written in Smalltalk - to my understanding, if Smalltalk can run without a special machine (virtual or physical), then it's not Smalltalk. The article may be talking about how Smalltalk is implemented as an operating system written in itself, but I'd argue that this doesn't really count as the type of operating system that this Wiki is meant for. --[[User:Spotracite|-1]] 22:49, 28 November 2021 (CST)
````

<a id="w004677"></a>

## W004677 — Zig Bare Bones

Bron-ID: W004677
Namespace: Main (0)
Revisie: 30276 | Datum: 2026-03-16T17:53:53Z
Bron: https://wiki.osdev.org/Zig_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Zig_Bare_Bones&oldid=30276

````text
{{Stub}}
{{TutorialExplain}}
In this tutorial, we'll make a simple hello world kernel in Zig for x86 PCs.

== Prerequisites ==
First off, you'll need:
* The [https://ziglang.org/ Zig] compiler version 0.15.1+
* [[GRUB]] as our bootloader to boot the kernel

== Code ==
If you done setting up all of the prerequisites above, we can now write
some code for our kernel

=== build.zig ===
<syntaxhighlight lang="zig">
const std = @import("std");

pub fn build(b: *std.Build) !void {
    const optimize = b.standardOptimizeOption(.{});

    const Target = std.Target.x86;
    const target = b.resolveTargetQuery(.{
        .cpu_arch = .x86,
        .os_tag = .freestanding,
        .abi = .none,
        // We use software float because we are disabling all SIMD stuff
        .cpu_features_add = Target.featureSet(&.{.soft_float}),
        // Disable all SIMD related stuff because SIMD are problematic in kernel
        .cpu_features_sub = Target.featureSet(&.{ .avx, .avx2, .sse, .sse2, .mmx }),
    });

    const kernel = b.addExecutable(.{
        .name = "kernel.elf",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/main.zig"),
            .target = target,
            .optimize = optimize,
            .code_model = .kernel,
        }),
    });
    kernel.setLinkerScript(b.path("src/linker.ld"));
    b.installArtifact(kernel);

    const kernel_path = kernel.getEmittedBin();
    const qemu_cmd = b.addSystemCommand(&[_][]const u8{
        // zig fmt: off
        "qemu-system-x86_64",
        "-m", "1G",
        "-serial", "stdio",
    });
    // zig fmt: on
    qemu_cmd.addArg("-kernel");
    qemu_cmd.addFileArg(kernel_path);
    qemu_cmd.step.dependOn(b.getInstallStep());

    const run_cmd = b.addRunArtifact(kernel);
    run_cmd.step.dependOn(&qemu_cmd.step);
    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const run_step = b.step("run", "Run kernel with qemu");
    run_step.dependOn(&run_cmd.step);
}
</syntaxhighlight>

=== src/main.zig ===
<syntaxhighlight lang="zig">
const console = @import("console.zig");

const MB_HEADER_MAGIC = 0x1BADB002;
const MB_FLAG_ALIGN = 1 << 0;
const MB_FLAG_MEMINFO = 1 << 1;
const FLAGS = MB_FLAG_ALIGN | MB_FLAG_MEMINFO;

/// https://www.gnu.org/software/grub/manual/multiboot/multiboot.html#Header-layout
const MultibootHeader = packed struct {
    magic: u32 = MB_HEADER_MAGIC,
    flags: u32 = FLAGS,
    checksum: u32,
    padding: u32 = 0,
};

export var multiboot: MultibootHeader align(4) linksection(".multiboot") = .{
    // Here we are adding magic and flags and ~ to get 1's complement and by adding 1 we get 2's complement
    .checksum = ~@as(u32, (MB_HEADER_MAGIC + FLAGS)) + 1,
};

var stack_bytes: [16 * 1024]u8 align(16) linksection(".bss") = undefined;

// We specify that this function is "naked" to let the compiler know
// not to generate a standard function prologue and epilogue, since
// we don't have a stack yet.
export fn _start() callconv(.naked) noreturn {
    // We use inline assembly to set up the stack before jumping to
    // our kernel entry point.
    asm volatile (
        \\ movl %[stack_top], %%esp
        \\ movl %%esp, %%ebp
        \\ call %[kmain:P]
        :
        // The stack grows downwards on x86, so we need to point ESP register
        // to one element past the end of `stack_bytes`.
        //
        // Finally, we pass the whole expression as an input operand with the
        // "immediate" constraint to force the compiler to encode this as an
        // absolute address. This prevents the compiler from doing unnecessary
        // extra steps to compute the address at runtime (especially in Debug mode),
        // which could possibly clobber registers that are specified by multiboot
        // to hold special values (e.g. EAX).
        : [stack_top] "i" (stack_bytes[stack_bytes.len..].ptr),
          // We let the compiler handle the reference to kmain by passing it as an input operand as well.
          [kmain] "X" (&kmain),
    );
}

// We use noinline to make sure it don't get inlined by compiler
noinline fn kmain() callconv(.c) noreturn {
    // Initialize our VGA driver
    console.init();
    // Printing string
    console.print("Hello {s} kernel!", .{"zig"});
    // Loop forever as there is nothing to do
    while (true) {
        asm volatile ("hlt");
    }
}
</syntaxhighlight>

=== src/console.zig ===
<syntaxhighlight lang="zig">
const std = @import("std");
const console = @import("console.zig");

const VGA_WIDTH = 80;
const VGA_HEIGHT = 25;
const VGA_SIZE = VGA_WIDTH * VGA_HEIGHT;

var g_row: usize = 0;
var g_column: usize = 0;
var g_color: Color = .init(.light_gray, .black);
var g_buffer = @as([*]volatile u16, @ptrFromInt(0xB8000));

pub const ColorType = enum(u4) {
    black = 0,
    blue = 1,
    green = 2,
    cyan = 3,
    red = 4,
    magenta = 5,
    brown = 6,
    light_gray = 7,
    dark_gray = 8,
    light_blue = 9,
    light_green = 10,
    light_cyan = 11,
    light_red = 12,
    light_magenta = 13,
    light_brown = 14,
    white = 15,
};

const Color = packed struct(u8) {
    fg: ColorType,
    bg: ColorType,

    pub fn init(fg: ColorType, bg: ColorType) Color {
        return .{ .fg = fg, .bg = bg };
    }

    /// Combine vga color and char. The upper byte will be color and lower byte will be character
    pub fn getVgaChar(self: Color, char: u8) u16 {
        return @as(u16, @as(u8, @bitCast(self))) << 8 | char;
    }
};

/// Initialize VGA
pub fn init() void {
    clear();
}

/// Set Color for VGA
pub fn setColor(fg: Color, bg: Color) void {
    g_color = Color.init(fg, bg);
}

/// Clear the screen
pub fn clear() void {
    @memset(g_buffer[0..VGA_SIZE], Color.getVgaChar(g_color, ' '));
}

/// Print character with color at specific position
pub fn printCharAt(char: u8, color: Color, x: usize, y: usize) void {
    const index = y * VGA_WIDTH + x;
    g_buffer[index] = color.getVgaChar(char);
}

/// INFO: Scrolling is left as an exercise for the reader
fn checkAndScroll() void {
    if (g_row == VGA_HEIGHT) {
        g_row = 0;
    }
}

/// Print character to the VGA
pub fn printChar(char: u8) void {
    switch (char) {
        '\n' => {
            g_column = 0;
            g_row += 1;
            checkAndScroll();
        },
        else => {
            printCharAt(char, g_color, g_column, g_row);
            g_column += 1;
            if (g_column == VGA_WIDTH) {
                g_column = 0;
                g_row += 1;
                checkAndScroll();
            }
        },
    }
}

/// Implementation of std.Io.Writer.vtable.drain function.
/// When flush is called or the writer buffer is full this function is called.
/// This function first writes all data of writer buffer after that it writes
/// the argument data in which the last element is written splat times.
fn drain(w: *std.Io.Writer, data: []const []const u8, splat: usize) !usize {
    // the length of data must not be zero
    std.debug.assert(data.len != 0);

    var consumed: usize = 0;
    const pattern = data[data.len - 1];
    const splat_len = pattern.len * splat;

    // If buffer is not empty write it first
    if (w.end != 0) {
        printString(w.buffered());
        w.end = 0;
    }

    // Now write all data except last element
    for (data[0 .. data.len - 1]) |bytes| {
        printString(bytes);
        consumed += bytes.len;
    }

    // If out patter (i.e. last element of data) is non zero len then write splat times
    switch (pattern.len) {
        0 => {},
        else => {
            for (0..splat) |_| {
                printString(pattern);
            }
        },
    }
    // Now we have to return how many bytes we consumed from data
    consumed += splat_len;
    return consumed;
}

/// Returns std.Io.Writer implementation for this console
pub fn writer(buffer: []u8) std.Io.Writer {
    return .{
        .buffer = buffer,
        .end = 0,
        .vtable = &.{
            .drain = drain,
        },
    };
}

/// Print string to VGA
pub fn printString(str: []const u8) void {
    for (str) |char| {
        printChar(char);
    }
}

/// Print with standard zig format to VGA
pub fn print(comptime fmt: []const u8, args: anytype) void {
    var w = writer(&.{});
    w.print(fmt, args) catch return;
}

</syntaxhighlight>

=== src/linker.ld ===
<syntaxhighlight lang="asm">
/* The bootloader will look at kernel and start execution at the symbol
   designated as the entry point. */
ENTRY(_start)

/* Tell where the various sections of the object files will be put in the
   final kernel image. */
SECTIONS {
    /* It used to be universally recommended to use 1M as a start offset,
       as it was effectively guaranteed to be available under BIOS systems.
       However, UEFI has made things more complicated, and experimental data
       strongly suggests that 2M is a safer place to load. In 2016, a new
       feature was introduced to the multiboot2 spec to inform bootloaders
       that a kernel can be loaded anywhere within a range of addresses and
       will be able to relocate itself to run from such a loader-selected
       address, in order to give the loader freedom in selecting a span of
       memory which is verified to be available by the firmware, in order to
       work around this issue. This does not use that feature, so 2M was
       chosen as a safer option than the traditional 1M. */
    . = 2M;

    /* First put the multiboot header, as it is required to be put very early
       in the image or the bootloader won't recognize the file format. Next
       we'll put the .text section. */
    .text : ALIGN(4K) {
        /* We need to specify KEEP to prevent the linker from removing the
           multiboot section. */
        KEEP(*(.multiboot))
        *(.text)
    }

    /* Read-only data. */
    .rodata : ALIGN(4K) {
        *(.rodata)
    }

    /* Read-write data (initialized) */
    .data : ALIGN(4K) {
        *(.data)
    }

    /* Read-write data (uninitialized i.e. undefined) */
    .bss : ALIGN(4K) {
        *(COMMON)
        *(.bss)
    }

    /* The compiler may produce other sections, by default it will put them
       in a segment with the same name. Simply add stuff here as needed. */
}

</syntaxhighlight>
== Build ==
Now that our kernel code is done, we'll now build our kernel by running
the command below:

<syntaxhighlight lang="bash">
$ zig build
</syntaxhighlight>

== Verifying Multiboot ==
If the header isn't valid, GRUB will give an error that it can't find a Multiboot header when you try to boot it. This code fragment will help you diagnose such cases:<syntaxhighlight lang="bash">
grub-file --is-x86-multiboot zig-out/bin/kernel.elf
</syntaxhighlight><code>grub-file</code> is quiet but will exit 0 (successfully) if it is a valid multiboot kernel and exit 1 (unsuccessfully) otherwise. You can type <code>echo $?</code> in your shell immediately afterwards to see the exit status.

== Booting the Kernel ==
You can easily create a bootable CD-ROM image containing the GRUB bootloader and your kernel using the program <code>grub-mkrescue</code>. You may need to install the GRUB utility programs and the program <code>xorriso</code> (version 0.5.6 or higher). First you should create a file called <code>grub.cfg</code> containing the contents:<syntaxhighlight>
menuentry "Zig Bare Bones" {
	multiboot /boot/kernel.elf
}
</syntaxhighlight>Note that the braces must be placed as shown here. You can now create a bootable image of your operating system by typing these commands:<syntaxhighlight lang="bash">
mkdir -p isodir/boot/grub
cp zig-out/bin/kernel.elf isodir/boot/kernel.elf
cp grub.cfg isodir/boot/grub/grub.cfg
grub-mkrescue -o kernel.iso isodir
</syntaxhighlight>'''Warning:''' GNU GRUB, the bootloader used by <code>grub-mkrescue</code>, is licensed under the GNU General Public License. Your iso file contains copyrighted material under that license and redistributing it in violation of the GPL constitutes copyright infringement. The GPL requires you publish the source code corresponding to the bootloader. You need to get the exact source package corresponding to the GRUB package you have installed from your distribution, at the time <code>grub-mkrescue</code> is invoked (as distro packages are occasionally updated). You then need to publish that source code along with your ISO to satisfy the GPL. Alternative, you can build GRUB from source code yourself. Clone the latest GRUB git from savannah (do not use their last release from 2012, it's severely out of date). Run autogen.sh, ./configure and make dist. That makes a GRUB tarball. Extract it somewhere, then build GRUB from it, and install it in a isolated prefix. Add that to your PATH and ensure its <code>grub-mkrescue</code> program is used to produce your iso. Then publish the GRUB tarball of your own making along with your OS release. You're not required to publish the source code of your OS at all, only the code for the bootloader that's inside the iso.

=== Testing your operating system (QEMU) ===
In this tutorial, we will be using QEMU. You can also use other virtual machines if you please. Simply adding the ISO to the CD drive of an empty virtual machine will do the trick.

Install QEMU from your repositories, and then use the following command to start your new operating system.<syntaxhighlight lang="bash">
qemu-system-x86_64 -cdrom kernel.iso
</syntaxhighlight>Additionally, QEMU supports booting multiboot kernels directly without bootable medium:<syntaxhighlight lang="bash">
qemu-system-x86_64-kernel kernel.elf
</syntaxhighlight>For make testing easier '''run''' step is provided which effectively is equivalent to previous command
[[Category: Bare bones tutorials]] [[Category:Zig]]
````

<a id="w004678"></a>

## W004678 — User:Leapofazzam/Zig Bare Bones

Bron-ID: W004678
Namespace: User (2)
Revisie: 26726 | Datum: 2021-12-01T11:28:43Z
Bron: https://wiki.osdev.org/User:Leapofazzam/Zig_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALeapofazzam%2FZig_Bare_Bones&oldid=26726
Doorverwijzing: Zig Bare Bones

````text
#REDIRECT [[Zig Bare Bones]]
````

<a id="w004679"></a>

## W004679 — File:HistoricalCuriosity.png

Bron-ID: W004679
Namespace: File (6)
Revisie: 26761 | Datum: 2021-12-19T06:45:13Z
Bron: https://wiki.osdev.org/File:HistoricalCuriosity.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AHistoricalCuriosity.png&oldid=26761

````text
Tag timage for [[Template:Historical]]. Derviative of [https://commons.wikimedia.org/wiki/File:Pdp-11-40.jpg], which is under the CC-BY-SA 3.0 license.
````

<a id="w004680"></a>

## W004680 — Template:Historical

Bron-ID: W004680
Namespace: Template (10)
Revisie: 26762 | Datum: 2021-12-19T06:45:36Z
Bron: https://wiki.osdev.org/Template:Historical
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AHistorical&oldid=26762

````text
<center>
{|style="border: 1px solid #8f9f8f; padding: .0em .25em .0em; background-color: #d3cdbb; text-align: center;"
|
[[Image:HistoricalCuriosity.png]]
|
<font color="black">The content of this article or section is considered a historical curiosity and should not be considered for new designs.</font><br />
|
|}
<includeonly>[[Category:Historical Curiosities]]</includeonly>
</center>
<br />
````

<a id="w004681"></a>

## W004681 — Technical Specifications

Bron-ID: W004681
Namespace: Main (0)
Revisie: 30169 | Datum: 2026-03-09T16:31:17Z
Bron: https://wiki.osdev.org/Technical_Specifications
Revisie-URL: https://wiki.osdev.org/index.php?title=Technical_Specifications&oldid=30169

````text

=== Intel 64 and IA-32 Architectures Software Developer Manuals ===

{| class="wikitable"
|-
! Organization !! Short Name !! Title !! Version !! Date !! Link
|-
| Intel || SDM Vol 1 || Volume 1: Basic Architecture || || Jun 2021 || [https://www.intel.com/content/www/us/en/develop/download/intel-64-and-ia-32-architectures-software-developers-manual-volume-1-basic-architecture.html pdf]
|-
| Intel || SDM Vol 2 || Volume 2: Instruction Set Reference, A-Z || || Jun 2021 || [https://www.intel.com/content/www/us/en/develop/download/intel-64-and-ia-32-architectures-sdm-combined-volumes-2a-2b-2c-and-2d-instruction-set-reference-a-z.html pdf]
|-
| Intel || SDM Vol 3 || Volume 3: System Programming Guide || || Jun 2021 || [https://www.intel.com/content/www/us/en/develop/download/intel-64-and-ia-32-architectures-sdm-combined-volumes-3a-3b-3c-and-3d-system-programming-guide.html pdf]
|-
| Intel || SDM Vol 4 || Volume 4: Model-Specific Registers || || Jun 2021 || [https://www.intel.com/content/www/us/en/develop/download/intel-64-and-ia-32-architectures-software-developers-manual-volume-4-model-specific-registers.html pdf]
|}

=== Intel Chips Specifications ===

{| class="wikitable"
|-
! Organization !! Short Name !! Title !! Version !! Date !! Link
|-
| Intel || x2APIC || Intel 64 Architecture x2APIC Specification ||  || Jun 2008 || [https://www.naic.edu/~phil/software/intel/318148.pdf pdf]
|-
| Intel || HPET || IA-PC HPET (High Precision Event Timers) Specification || 1.0a || Oct 2004 || [https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/software-developers-hpet-spec-1-0a.pdf pdf]
|-
| Intel || PIIX4 || 82371AB PCI-TO-ISA / IDE Xcelerator (PIIX4) || || Apr 1997 || [https://web.archive.org/web/20210725161012/https://www.intel.com/assets/pdf/datasheet/290562.pdf pdf]
|-
| Intel || PIIX/PIIX3 || 82371FB (PIIX) and 82371SB (PIIX3) PCI ISA IDE Xcelerator || || Apr 1997 || [http://pdf.datasheetcatalog.com/datasheet/Intel/mXvqwzr.pdf pdf]
|-
| Intel || IOAPIC || 82093AA I/O Advanced Programmable Interrupt Controller (IOAPIC) || || May 1996 || [http://web.archive.org/web/20161130153145/http://download.intel.com/design/chipsets/datashts/29056601.pdf pdf]
|-
| Intel || PIT || 8254 Programmable Interval Timer || || Sep 1993 || [https://www.scs.stanford.edu/10wi-cs140/pintos/specs/8254.pdf pdf]
|-
| Intel || PIC || 8259A Programmable Interrupt Controller (8259A/8259A-2) || || Dec 1988 || [https://pdos.csail.mit.edu/6.828/2005/readings/hardware/8259A.pdf pdf]
|-
| Intel || DMA || 8237/8237-2 High Performance Programmable DMA Controller || || May 1979 || [http://web.archive.org/web/20190819134509/http://zet.aluzina.org/images/8/8c/Intel-8237-dma.pdf pdf]
|}

=== BIOS, UEFI, Boot, and Configuration Specifications ===

{| class="wikitable"
|-
! Organization !! Short Name !! Title !! Version !! Date !! Link
|-
| DMTF || SMBIOS ||  	System Management BIOS (SMBIOS) Reference Specification || 3.5 || Sep 2021 || [https://www.dmtf.org/sites/default/files/standards/documents/DSP0134_3.5.0.pdf pdf]
|-
| UEFI Forum || UEFI || Unified Extensible Firmware Interface || 2.11 || Nov 2024 || [https://uefi.org/sites/default/files/resources/UEFI_Spec_Final_2.11.pdf pdf] [https://uefi.org/specs/UEFI/2.11/ html]
|-
| UEFI Forum || UEFI Shell || UEFI Shell Specification || 2.2 || Jan 2016 || [https://uefi.org/sites/default/files/resources/UEFI_Shell_2_2.pdf pdf]
|-
| UEFI Forum || UEFI PIS || UEFI Platform Initialization Specification || 1.9 || Dec 2024 || [https://uefi.org/sites/default/files/resources/UEFI_PI_Spec_Final_Draft_1.9.pdf pdf] [https://uefi.org/specs/PI/1.9/ html]
|-
| UEFI Forum || ACPI || Advanced Configuration and Power Interface || 6.6 || May 2025 || [https://uefi.org/sites/default/files/resources/ACPI_Spec_6.6.pdf pdf] [https://uefi.org/specs/ACPI/6.6/ html]
|-
| FSF || Multiboot2 || Multiboot2 Specification || 2.0 || 2016 || [https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html html]
|-
| FSF || Multiboot || Multiboot Specification || 0.6.96 || 2010 || [https://www.gnu.org/software/grub/manual/multiboot/multiboot.html html]
|-
| Intel || MPS || MultiProcessor Specification || 1.4 || May 1997 || [https://web.archive.org/web/20121002210153/http://download.intel.com/design/archives/processors/pro/docs/24201606.pdf pdf]
|-
| Intel, Microsoft || APM || Advanced Power Management (APM) BIOS Interface Specification || 1.2 || Feb 1996 || [http://intel-vintage-developer.eu5.org/IAL/POWERMGM/APMV12.PDF pdf]
|-
| Compaq, Phoenix, Intel || BBS || BIOS Boot Specification || 1.01 || Jan 1996 || [https://www.scs.stanford.edu/nyu/04fa/lab/specsbbs101.pdf pdf]
|-
| Phoenix, IBM || El Torito || "El Torito" Bootable CD-ROM Format Specification || 1.0 || Jan 1995 || [https://tc.gts3.org/cs3210/2016/spring/r/boot-cdrom.pdf pdf]
|-
| Compaq, Phoenix, Intel || PNPBIOS || Plug and Play BIOS Specification || 1.0a || May 1994 || [http://www.osdever.net/documents/PNPBIOSSpecification-v1.0a.pdf pdf]
|-
| INCITS T13 || BIOS EDD-3 || BIOS Enhanced Disk Drive Services - 3 || 3 || November 2004 || [https://web.archive.org/web/20200921123019/http://www.t13.org:80/Documents/UploadedDocuments/docs2004/d1572r3-EDD3.pdf pdf]
|}

=== PCI Specifications ===

{| class="wikitable"
|-
! Organization !!Short Name !! Title !! Version !! Date !! Link
|-
! colspan="6" | PCI Express
|-
| PCI-SIG || PCIe || PCI Express Base Specification || 6.0 || Jan 2022 ||
|-
| PCI-SIG || PCIe || PCI Express Base Specification || 5.0 || Jul 2019 ||
|-
| PCI-SIG || PCIe || PCI Express Base Specification || 4.0 || Oct 2017 ||
|-
| PCI-SIG || PCIe || PCI Express Base Specification || 3.1a || Dec 2015 ||
|-
| PCI-SIG || PCIe || PCI Express Base Specification || 3.0 || Nov 2010 ||
|-
| PCI-SIG || PCIe || PCI Express Base Specification || 2.1 || Mar 2009 ||
|-
| PCI-SIG || PCIe || PCI Express Base Specification || 2.0 || Dec 2006 || [https://www.cl.cam.ac.uk/~djm202/pdf/specifications/pcie/PCI_Express_Base_Rev_2.0_20Dec06a.pdf pdf]
|-
| PCI-SIG || PCIe || PCI Express Base Specification || 1.1 || 2005 ||
|-
| PCI-SIG || PCIe || PCI Express Base Specification || 1.0a || 2003 ||
|-
! colspan="6" | PCI-X
|-
| PCI-SIG || PCI-X || PCI-X 2.0 Specification || 2.0 || Jul 2002 ||
|-
| PCI-SIG || PCI-X || PCI-X Addendum to the PCI Local Bus Specification || 1.0 || Sep 1999 || [https://cds.cern.ch/record/551393/files/cer-2308899.pdf pdf]
|-
! colspan="6" | PCI Conventional
|-
| PCI-SIG || PCI || PCI Local Bus Specification || 3.0 || Feb 2004 ||
|-
| PCI-SIG || PCI || PCI Local Bus Specification || 2.3 || Mar 2002 ||
|-
| PCI-SIG || PCI || PCI Local Bus Specification || 2.2 || Dec 1998 || [https://www.ics.uci.edu/~harris/ics216/pci/PCI_22.pdf pdf]
|-
| PCI-SIG || PCI || PCI Local Bus Specification || 2.1 || 1995 ||
|-
| PCI-SIG || PCI || PCI Local Bus Specification || 2.0 || 1993 ||
|-
| Intel || PCI || PCI Local Bus Specification || 1.0 || 1992 ||
|}

=== ATA Specifications ===

{| class="wikitable"
|-
! Organization !! Short Name !! Title !! Version !! Date !! Link
|-
! colspan="6" | Serial ATA [[AHCI]]
|-
| Intel || AHCI || Serial ATA Advanced Host Controller Interface (AHCI) || 1.3.1 || Nov 2013 || [https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/serial-ata-ahci-spec-rev1-3-1.pdf pdf]
|-
| Intel || AHCI || Serial ATA Advanced Host Controller Interface (AHCI) || 1.3 || Jun 2008 || [https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/serial-ata-ahci-spec-rev1_3.pdf pdf]
|-
| Intel || AHCI || Serial ATA Advanced Host Controller Interface (AHCI) || 1.2 || Apr 2007 || [https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/serial-ata-ahci-spec-rev1_2.pdf pdf]
|-
| Intel || AHCI || Serial ATA Advanced Host Controller Interface (AHCI) || 1.1 || Feb 2005 || [https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/serial-ata-ahci-spec-rev1_1.pdf pdf]
|-
| Intel || AHCI || Serial ATA Advanced Host Controller Interface (AHCI) || 1.0 || Apr 2004 || [https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/serial-ata-ahci-spec-rev1_0.pdf pdf]
|-
! colspan="6" | [[SATA|Serial ATA]]
|-
| SATA-IO || SATA 3.5 || Serial ATA Revision 3.5 || 3.5 || Jul 2020 || 
|-
| SATA-IO || SATA 3.4 || Serial ATA Revision 3.4 || 3.4 || Jun 2018 || 
|-
| SATA-IO || SATA 3.3 || Serial ATA Revision 3.3 || 3.3 || Feb 2016 || 
|-
| SATA-IO || SATA 3.2 || Serial ATA Revision 3.2 || 3.2 || Aug 2013 || 
|-
| SATA-IO || SATA 3.1 || Serial ATA Revision 3.1 || 3.1 || Jul 2011 || [https://sata-io.org/system/files/specifications/SerialATA_Revision_3_1_Gold.pdf pdf]
|-
| SATA-IO || SATA 3.0 || Serial ATA Revision 3.0 || 3.0 || Jun 2009 || [http://www.lttconn.com/res/lttconn/pdres/201005/20100521170123066.pdf pdf]
|-
| SATA-IO || SATA 2.6 || Serial ATA Revision 2.6 || 2.6 || Feb 2007 || 
|-
| SATA-IO || SATA 2.5 || Serial ATA Revision 2.5 || 2.5 || Aug 2005 || 
|-
| SATA Working Group || SATA 2.0 || Serial ATA Revision 2.0 || 2.0 || Apr 2004 || 
|-
| SATA Working Group || SATA 1.0 || Serial ATA: High Speed Serialized AT Attachment || 1.0a || Jan 2003 || [https://web.archive.org/web/20161009182351/http://www.ece.umd.edu/courses/enee759h.S2003/references/serialata10a.pdf pdf]
|-
! colspan="6" | Parallel ATA
|-
| INCITS T13 || ATA/ATAPI-8 || AT Attachment with Packet Interface - 8 (ATA/ATAPI-8) || 8 || Nov 2005 || [https://hddguru.com/download/documentation/ATA-ATAPI-standard-8/d1700r2-ATA8-Architecture-Model.pdf Architecture Model] [http://hddguru.com/download/documentation/ATA-ATAPI-standard-8/d1699r2b-ATA8-Command-Set.pdf Command Set]
|-
| INCITS T13 || ATA/ATAPI-7 || AT Attachment with Packet Interface - 7 (ATA/ATAPI-7) || 7 || Apr 2004 || [https://hddguru.com/download/documentation/ATA-ATAPI-standard-7/d1532v1r4b-ATA-ATAPI-7-1.pdf Vol1] [https://hddguru.com/download/documentation/ATA-ATAPI-standard-7/d1532v2r4b-ATA-ATAPI-7-2.pdf Vol2] [http://hddguru.com/download/documentation/ATA-ATAPI-standard-7/d1532v3r4b-ATA-ATAPI-7-3.pdf Vol3]
|-
| INCITS T13 || ATA/ATAPI-6 || AT Attachment with Packet Interface - 6 (ATA/ATAPI-6) || 6 || Jun 2001 || [https://hddguru.com/download/documentation/ATA-ATAPI-standard-6/ATA-ATAPI-6.pdf pdf]
|-
| INCITS T13 || ATA/ATAPI-5 || AT Attachment with Packet Interface - 5 (ATA/ATAPI-5) || 5 || Dec 1999 || [https://hddguru.com/download/documentation/ATA-ATAPI-standard-5/ATA-ATAPI-5.pdf pdf]
|-
| INCITS T13 || ATA/ATAPI-4 || AT Attachment with Packet Interface Extension (ATA/ATAPI-4) || 4 || Aug 1998 || [https://eab.abime.net/attachment.php?attachmentid=29721&d=1320242061&usg=AOvVaw2tCbzIDz4J0u2nwttyZzCl zip]
|-
| INCITS T13 || ATA-3 || AT Attachment-3 Interface (ATA-3) || 3 || Jan 1997 || [https://www.scs.stanford.edu/10wi-cs140/pintos/specs/ata-3-std.pdf pdf]
|-
| X3T10 || ATA-2 || AT Attachment Interface with Extensions (ATA-2) || 2 || Mar 1996 || [https://hddguru.com/download/documentation/ATA-ATAPI-standard-2/d0948r4c_ATA-ATAPI-2.pdf pdf]
|-
| X3T10 || ATA-1 || AT Attachment Interface for Disk Drives || 1 || 1994 || [https://hddguru.com/download/documentation/ATA-ATAPI-standard-1/d0791r4c_ATA-ATAPI-1.pdf pdf]
|}

=== SCSI Specifications ===

{| class="wikitable"
|-
! Organization !! Short Name !! Title !! Version !! Date !! Link
|-
| X3T10 || SCSI-3 || SCSI-3 Parallel Interface (SPI) || || Feb 1996 || [https://stuff.mit.edu/afs/sipb/contrib/doc/specs/protocol/scsi-3/spi-r15a.pdf pdf]
|-
| X3T9 || SCSI-2 || Small Computer System lnterface-2 || || Jan 1994 || [http://www.bitsavers.org/components/ncr_symbios/scsi/SCSI-2_Standard_1994.pdf pdf]
|-
| X3T9 || SCSI-1 || Small Computer System Interface (SCSI) || || Jun 1986 || [https://nvlpubs.nist.gov/nistpubs/Legacy/FIPS/fipspub131.pdf pdf]
|-
| X3T9 || SASI || Shugart Associates System Interface (SASI) || || Apr 1982 || [http://www.bitsavers.org/pdf/ansi/X3T9/X3T9.3_185_RevE_SASI_Apr82.pdf pdf]
|}

=== USB Specifications ===

{| class="wikitable"
|-
! Organization !! Short Name !! Title !! Version !! Date !! Link
|-
| Intel || xHCI || eXtensible Host Controller Interface for USB || 1.2 || Mar 2019 || [https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf pdf]
|-
| Intel || EHCI || Enhanced Host Controller Interface Specification for USB || 1.0 || Mar 2002 || [https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/ehci-specification-for-usb.pdf pdf]
|-
| Compaq, Microsoft, National Semiconductor || OHCI || Open Host Controller Interface Specification for USB || 1.0a || Sep 1999 || [https://composter.com.ua/documents/OHCI_Specification_Rev.1.0a.pdf pdf]
|-
| Intel || UHCI || Universal Host Controller Interface (UHCI) Design Guide || 1.1 || Mar 1996 || [https://web.archive.org/web/20100326195950/http://www.usbman.com/WebDrivers/usbpdffiles/UHCI%20Intel%20Specification.pdf pdf]
|}

=== Video Display Specifications ===

{| class="wikitable"
|-
! Organization !! Short Name !! Title !! Version !! Date !! Link
|-
! colspan="6" | [[GOP]]
|-
| UEFI Forum || GOP || UEFI Graphics Output Protocol || 2.9 || Mar 2021 || [https://uefi.org/sites/default/files/resources/UEFI_Spec_2_9_2021_03_18.pdf#G16.1018772 pdf]
|-
| UEFI Forum || GOP || UEFI Graphics Output Protocol || 2.0 || Jan 2006 || [https://uefi.org/sites/default/files/resources/UEFI_Specification_2_and_Errata_Sept16_08.pdf#page=443 pdf]
|-
| Intel, Microsoft || UGA || EFI Universal Graphics Adapter || 1.1 || Dec 2002 || [https://www.intel.de/content/dam/doc/product-specification/efi-v1-10-specification.pdf#page=318 pdf]
|-
! colspan="6" | [[VBE]]
|-
| VESA || VBE || VESA BIOS Extension (VBE) Core Functions || 3.0 || Sep 1998 || [http://web.archive.org/web/20080302090304/http://www.vesa.org/public/VBE/vbe3.pdf pdf]
|-
| VESA || VBE/AF || VESA BIOS Extension/Accelerator Functions (VBE/AF) || 1.0 || Aug 1996 || [https://web.archive.org/web/20081209121702/http:/www.vesa.org/public/VBE/VBE-AF07.pdf pdf]
|-
| VESA || VBE || VESA BIOS Extension (VBE) Core Functions || 2.0 || Nov 1994 || [http://www.phatcode.net/res/221/files/vbe20.pdf pdf]
|-
| VESA || VBE || Super VGA BIOS Extension || 1.2 || Oct 1991 || [https://web.archive.org/web/20090114055246/http://docs.ruudkoot.nl/vesasp12.txt txt]
|-
| VESA || VBE || Super VGA BIOS Extension || 1.1 || 1990 ||
|-
| VESA || VBE || Super VGA BIOS Extension || 1.0 || Oct 1989 || [https://web.archive.org/web/20131211151957/http://www.gemixtes.de/super-vga-bios-extension-standard-vs891001/ html]
|-
! colspan="6" | VGA (and older)
|-
| IBM || VGA || Video Graphics Array || || May 1987 || [https://ardent-tool.com/docs/pdf/84X1508_PS2_Model_80_Technical_Reference_Apr87.pdf#page=231 pdf]
|-
| IBM || MCGA || Multi-Color Graphics Array || || Jun 1987 || [https://ardent-tool.com/docs/pdf/PS2_Model_25_Technical_Reference_Jun87.pdf#page=49 pdf]
|-
| IBM || EGA || Enhanced Graphics Adapter || || Aug 1984 || [http://minuszerodegrees.net/oa/OA%20-%20IBM%20Enhanced%20Graphics%20Adapter.pdf pdf]
|-
| IBM || CGA || Color/Graphics Monitor Adapter || || Aug 1981 || [http://minuszerodegrees.net/oa/OA%20-%20IBM%20Color%20Graphics%20Monitor%20Adapter%20(CGA).pdf pdf]
|-
| IBM || MDA || Monochrome Display Adapter || || Aug 1981 || [http://minuszerodegrees.net/oa/OA%20-%20IBM%20Monochrome%20Display%20and%20Printer%20Adapter.pdf pdf]
|}

[[Category:Resources]]
````

<a id="w004682"></a>

## W004682 — User talk:Raidtheweb

Bron-ID: W004682
Namespace: User talk (3)
Revisie: 26804 | Datum: 2021-12-27T21:30:08Z
Bron: https://wiki.osdev.org/User_talk:Raidtheweb
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ARaidtheweb&oldid=26804

````text
e
````

<a id="w004683"></a>

## W004683 — User:Raidtheweb

Bron-ID: W004683
Namespace: User (2)
Revisie: 26805 | Datum: 2021-12-27T21:30:17Z
Bron: https://wiki.osdev.org/User:Raidtheweb
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARaidtheweb&oldid=26805

````text
certified idiot who does osdev sometimes
````

<a id="w004685"></a>

## W004685 — S390 Barebones

Bron-ID: W004685
Namespace: Main (0)
Revisie: 26837 | Datum: 2022-01-04T00:47:34Z
Bron: https://wiki.osdev.org/S390_Barebones
Revisie-URL: https://wiki.osdev.org/index.php?title=S390_Barebones&oldid=26837
Doorverwijzing: FLATBOOT Barebones

````text
#REDIRECT [[FLATBOOT Barebones]]
````

<a id="w004686"></a>

## W004686 — User:Sham1

Bron-ID: W004686
Namespace: User (2)
Revisie: 26839 | Datum: 2022-01-04T19:17:51Z
Bron: https://wiki.osdev.org/User:Sham1
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASham1&oldid=26839

````text
I do OSDev sometimes, but I am mostly active at #osdev at Libera Chat.
````

<a id="w004687"></a>

## W004687 — User:Wxwisiasdf/Compile GCCMVS under MVS3.8j

Bron-ID: W004687
Namespace: User (2)
Revisie: 27086 | Datum: 2022-05-13T02:47:28Z
Bron: https://wiki.osdev.org/User:Wxwisiasdf/Compile_GCCMVS_under_MVS3.8j
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWxwisiasdf%2FCompile_GCCMVS_under_MVS3.8j&oldid=27086

````text
{{stub}}

Building GCCMVS is a quite trivial task - depending on who you ask.

'''You need Windows/Wine for this.'''

== Preparation ==

Create a folder called `gccmvs`, this is where `pdos` and `gcc` folders should reside (see below).

Download custom.zip from PDOS.org:
* http://www.pdos.org/custom.zip

Extract gcc-stage-*.zip (* = any version, 320, 270, etc) in the `gccmvs` folder; the extraction should yield a folder named `gcc` (do not extract and make directory!)

Download PDOS and put it on your `gccmvs/` folder, so that the pdos/pdpclib can be accessed via gccmvs/pdos/pdpclib:
* https://sourceforge.net/p/pdos/gitcode/ci/master/tree/s370/

Now your `gccmvs` folder should look roughly like this:
<source lang="asm">
gccmvs/
    pdos/
        pdpclib/
            allmvs.bat
            ...
        src/
        ...
    gcc/
        gcc/
            allmvs.bat
            ...
        ...
</source>

PDPCLIB is required by GCCMVS to properly cross-build.

You need an emulator for this, since MVS/380 does not run under x86, it only runs on S/390. So download the Hercules/380 emulator (it implements an S/380 to allow usage of more than 16 MiB of memory, which to be realistic is pretty cool, it won't run on realiron however (but your compiled programs should do tho)).
* https://sourceforge.net/projects/mvs380/files/hercules380/Hercules-380%20v3.07-5.0/

You'll also need a program called m4 for macro processing, make sure it's in your %PATH% and that you can run it:
* http://gnuwin32.sourceforge.net/packages/m4.htm

MVS/380 has a 2.0 release but it had a malicious file embedded on it - so I made a version without the file; the file in question was located on `utils/URL2FILE.EXE`, it seems to have gotten there purely by accident.
* https://archive.org/details/mvs380-2_0

'''Note:''' [https://technologydribble.info/2008/09/18/url-to-file/ URL2FILE.EXE might be a false positive?] (better be prepared than sorry)

Now, you have code, which is mostly known to be useless if not compiled - so let's get into that.

== Building ==
Open a command prompt in your root directory where you extracted `pdos` and `gcc`. Type in the following:
<source lang="bash">
cd gcc/gcc
allmvs.bat
</source>

You now patiently wait until everything finishes.

If you see an RC=12 or **FLUSH** message from ASMA90 then you have an error - good luck with that!

== I run out of memory! ==
You must use Hercules/380, you can't use normal Hercules.

== GCC ABENDs when trying to compile itself ==
You have AM31 enabled, go to your mvs380 installation and comment out the line on conf/mvs380.conf where it mentions AMODE31/AMODE32

== IEFCOMPR fails ==

'''TODO: someone should attempt to get gccmvs C++ frontend'''
````

<a id="w004688"></a>

## W004688 — Category:Stivale

Bron-ID: W004688
Namespace: Category (14)
Revisie: 27589 | Datum: 2023-01-14T06:21:49Z
Bron: https://wiki.osdev.org/Category:Stivale
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AStivale&oldid=27589

````text
Stivale/Stivale2 boot protocol
[[Category:Bootloaders]]
````

<a id="w004689"></a>

## W004689 — User:Klange/QEMU virt

Bron-ID: W004689
Namespace: User (2)
Revisie: 26906 | Datum: 2022-02-01T00:31:46Z
Bron: https://wiki.osdev.org/User:Klange/QEMU_virt
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKlange%2FQEMU_virt&oldid=26906

````text
These are my personal notes on the QEMU aarch64 <code>-machine virt</code> target.

== Kernel Link Address ==

The <code>virt</code> machine places RAM at <code>0x4000_0000</code> and will happily load your kernel there, but you don't want to do this as that's also where it puts the device tree. If your kernel is too low in RAM, QEMU will quietly skip loading the device tree. Recommendation from geist is <code>0x4010_0000</code>.

== PCIe ==

With the latest virt target, expect PCIe to only be available from the high memory location provided through the device tree. To get the hardcoded low address for the ECAM, use <code>-machine virt-2.12</code> instead.

To find the ECAM offset for a device, the controller QEMU emulates helpfully simplifies things:

<source lang="c">
(bus << 20) | (slot << 15) | (function << 12) | (field)
</source>

== Framebuffer ==

The <code>virt</code> target comes with PCIe and you can happily attach a [[Bochs VBE Extensions|Bochs graphics adapter]] with <code>-device bochs-display</code>. It will provide the MMIO interface, but you'll have to actually assign BARs for it (and the framebuffer) as there's no firmware setting up PCIe for you. You don't need to do anything with the bridge, though.

Note that the Bochs adapter appears to have issues on some accelerators, like HVF; it seems to be trapping on every right, so it is incredible slow. The QEMU docs recommend the <code>virtio-gpu-pci</code> which uses regular guest memory and has flush commands.

== <code>fw-cfg</code> ==

QEMU's <code>fw-cfg</code> interface is available and you can use it to stuff arbitrary data and files into your VM, such as a ramdisk that I can't figure out how to get from the <code>-initrd</code> option despite that not printing any warnings...

To communicate with <code>fw-cfg</code> on ARM64, locate the MMIO address through the DTB (it has a name like <code>fw-cfg@XXXXXX</code>) and then treat it as three MMIO ports, something like this:

<source lang="c">
struct fw_cfg {
  union {
    volatile uint64_t _64;
    volatile uint32_t _32;
    volatile uint8_t  _8;
  } data; /* data is "string-preserving", so if you want a 32-bit integer, it'll be big-endian, but all of it will still be in the first four bytes */
  volatile uint16_t selector;
  uint16_t padding_1; /* data, selector, and dma_addr are 64-bit aligned... not sure why they didn't stick selector at the end to avoid padding confusion since it's always 16-bit... */
  uint32_t padding_2;
  volatile uint64_t dma_addr;
};
</source>

Annoyingly, everything here is big-endian and you need to be careful with your access widths as QEMU will give you as much as you ask for.

To confirm that you are talking to QEMU (or rather, to confirm the fw-cfg device is actually working, obviously you're talking to QEMU if you find one at all...), write 0 to the selector (make sure you write two bytes) and then read at least 4 bytes from the data response (you can do that as a read of a <code>uint32_t</code> or four <code>uint8_t</code>s or even just read a <code>uint64_t</code> as the other bytes will be 0) and confirm you get "QEMU" out of it.

To read the list of selectors, write <code>0x0019</code> ''in big-endian layout'' to the selector field and then read a 32-bit count (also big-endian) from the data port, and then a series of structs:

<source lang="c">
struct fw_cfg_file {
  uint32_t size;
  uint16_t selector;
  uint16_t padding;
  char name[56];
};
</source>

Read the structs as bytes and then endian-swap the size. For the purpose of display while debugging, you may also want to endian-swap the selector, but just remember it needs to be big-endian again when you use it.

Read files by writing the (big-endian) selector you got from the list to the selector field in the MMIO, and then read the bytes up to the <code>size</code> however you want.

You can also use the DMA interface, which is a lot faster. Set up a struct like this:

<source lang="c">
struct fw_cfg_dma {
  volatile uint32_t control;
  volatile uint32_t length;
  volatile uint64_t address;
};
</source>

<code>control</code> is a combination of a bitfield and the selector. Again, all of these are big-endian.

<source lang="c">
struct fw_cfg_dma dma;
dma.control = swizzle((selector << 16) | (1 << 3 /* read */) | (1 << 1 /* from selector */));
dma.length  = swizzle(size);
dma.address = swizzle64(physical_address_to_read_into);
</source>

Once you've set up this struct, write its address (64-bit and big-endian!) to the <code>dma_addr</code> field of the MMIO. QEMU docs say that the write to the latter 4 bytes is what triggers the transaction and that a single 64-bit write should do the write thing, so this should immediately trigger the DMA.

You can then re-read the values in <code>control</code> to check for errors; it should have been 0'd by QEMU on success. QEMU also says that DMA transactions are synchronous so it should either be done already or have errored.

With all of that done, you now have an easy way to get arbitrary blobs and strings into your kernel at runtime.

== RTC ==

QEMU provides a real-time clock. It has a 32-bit counter that gives Unix timestamps in seconds. It will rollover in 2038, so maybe add a check to catch that.

You just need to read <code>0x09010000</code> as a little-endian 32-bit unsigned integer.

Presumably this can move and DTB will tell you where it actually is...

== Serial ==

Write bytes to <code>0x09000000</code>. Weirdly the QEMU docs seem to suggest the MMIO access is 32-bit, but it's little-endian so ints or chars or whatever, doesn't matter. Just spew to that.

Make sure you also ''enable'' the serial output (<code>-serial mon,stdio</code> or something) if you actually want to see it...
````

<a id="w004690"></a>

## W004690 — User:Lugar

Bron-ID: W004690
Namespace: User (2)
Revisie: 26901 | Datum: 2022-01-31T20:19:38Z
Bron: https://wiki.osdev.org/User:Lugar
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALugar&oldid=26901

````text
[[/Setting_Up_A_GDT_In_Long_Mode]]
````

<a id="w004691"></a>

## W004691 — D Bare Bones II

Bron-ID: W004691
Namespace: Main (0)
Revisie: 30285 | Datum: 2026-03-16T18:55:31Z
Bron: https://wiki.osdev.org/D_Bare_Bones_II
Revisie-URL: https://wiki.osdev.org/index.php?title=D_Bare_Bones_II&oldid=30285

````text
{{TutorialExplain}}
{{BeginnersWarning}}
{{Rating|1}}{{Template:Kernel designs}}
In this Tutorial, we will continue to write the kernel in D, making a basic output to the console.

==Overview==
In this tutorial, we will continue to create our kernel on D, namely, we will make a minimal output to the console.
Our file structure will be as follows:
* start.asm
* kernel.main.d
* linker.ld

==start.asm==

<syntaxhighlight lang="asm">

global start
extern kmain        ; Allow kmain() to be called from the assembly code
extern start_ctors, end_ctors, start_dtors, end_dtors

MODULEALIGN        equ        1<<0
MEMINFO            equ        1<<1
FLAGS              equ        MODULEALIGN | MEMINFO
MAGIC              equ        0x1BADB002
CHECKSUM           equ        -(MAGIC + FLAGS)

section .text      ; Next is the Grub Multiboot Header

align 4
MultiBootHeader:
       dd MAGIC
       dd FLAGS
       dd CHECKSUM

STACKSIZE equ 0x4000  ; 16 KiB if you're wondering

static_ctors_loop:
   mov ebx, start_ctors
   jmp .test
.body:
   call [ebx]
   add ebx,4
.test:
   cmp ebx, end_ctors
   jb .body

start:
       mov esp, STACKSIZE+stack

       push eax
       push ebx

       call main

static_dtors_loop:
   mov ebx, start_dtors
   jmp .test
.body:
   call [ebx]
   add ebx,4
.test:
   cmp ebx, end_dtors
   jb .body

    
cpuhalt:
       hlt
       jmp cpuhalt

section .bss
align 32

stack:
      resb      STACKSIZE

</syntaxhighlight>

Assemble that with:

<syntaxhighlight lang="bash">nasm -f elf -o start.o start.asm</syntaxhighlight>

==kernel.main.d==
<syntaxhighlight lang="d">
module kernel.main;
import core.volatile;

extern(C): // We denote that all functions in our file will have the extern(C) flag

// Video memory address
const ubyte* vidmem = cast(ubyte*)0xFFFF_8000_000B_8000;

// Creating variables to indicate the cursor position.
// All global dynamic variables need to be marked with the shared flag, because there is no TLS in our kernel
shared int xpos = 0;
shared int ypos = 0;

// Creating a function to clear the console.
void clear() {
	for (int i = 0; i < 80*25*2; i++) {
		volatileStore(vidmem + i, 0);
	}
}

// Here we output the symbol passed to the function with the color as 0x07(light gray)
void putc_at(char symbol, int x, int y) {
    volatileStore(vidmem + x*2+y*160,symbol&0xFF);
    volatileStore(vidmem + x*2+y*160 + 1,0x07);
}

// Here we output a character with a check to see if it is a newline character.
void putc(char symbol) {
	if(symbol == '\n') {
		xpos=0;
		ypos+=1;
	}
	else {
		putc_at(symbol,xpos,ypos);
		xpos+=1;
		if(xpos==80) {xpos=0; ypos+=1;}
	}
}

// Here we output a string by looping through it
void puts(immutable(char*) str) {
	int counter = 0;
	while(str[counter] != 0) {
		putc(str[counter]);
		counter+=1;
	}
}
 
void kmain(uint magic, uint addr) {
 
	puts("Hello, world".ptr); // Output "Hello, world!" to the console
 
	for (;;) { //Loop forever. You can add your kernel logic here
            
	}
}
</syntaxhighlight>

You then compile that with:

<syntaxhighlight lang="bash">gdc -fno-druntime -m32 -c kernel.main.d -o kernel.main.o -g</syntaxhighlight>
==linker.ld==
<syntaxhighlight lang="c">
OUTPUT_FORMAT(elf32-i386)
ENTRY (start)

SECTIONS{
    . = 0x00100000;

    .text :{
        code = .; _code = .; __code = .;
        *(.text)
        *(.rodata)
    }

    .rodata ALIGN (0x1000) : {
        *(.rodata)
    }

    .data ALIGN (0x1000) : {
        data = .; _data = .; __data = .;
        *(.data)
        start_ctors = .; *(.ctors)   end_ctors = .;
        start_dtors = .; *(.dtors)   end_dtors = .;
    }

    .bss : {
        sbss = .;
        bss = .; _bss = .; __bss = .;
        *(COMMON)
        *(.bss)
        ebss = .;
    }
    end = .; _end = .; __end = .;
}
</syntaxhighlight>

Now finally you can link all of that with:

<syntaxhighlight lang="bash">ld -melf_i386 -T linker.ld -o kernel.bin start.o kernel.main.o</syntaxhighlight>
Your kernel is now kernel.bin, and can now be booted by grub, or run in qemu:

<syntaxhighlight lang="bash">qemu-system-i386 -kernel kernel.bin</syntaxhighlight>

[[Category:Bare bones tutorials|D bare bones]]
[[Category:D]]
````

<a id="w004692"></a>

## W004692 — Talk:File Systems

Bron-ID: W004692
Namespace: Talk (1)
Revisie: 27016 | Datum: 2022-03-28T00:38:58Z
Bron: https://wiki.osdev.org/Talk:File_Systems
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AFile_Systems&oldid=27016

````text
== Inclusion of USTAR ==
I have noticed a recent "edit war" regarding the inclusion of USTAR that lead to the locking of this article. While tar files are an important tool to be aware of and USTAR is a more than adequate file system for tapes and archives, it has some key differences to other file systems, such as FAT, ext2, or ISO 9660. While I do not think that USTAR should be removed, these differences should be noted. Otherwise beginners may be mislead into implementing tars as another file system in their VFS. Specifically, following things should be noted:
* The recommended way of manipulating a tar file is extracting it first into something else: into another file system or into a temporary file system in RAM.
* Due to lack of fragmentation, multiple writes can be extremely slow, they should be combined before writing back.
* For one-time read-only purposes, it's adequate to just load individual files, or the entire tarball, sequentially directly into memory.
In addition, the note "No standard partition type for it, you'll have to pick one on your own" may be misleading, since it suggests that one might have an entire disk partition in the tar format. While this is certainly a possibility, I can't see any adequate use case for that, even when bootstrapping, especially for a bootloader like GRUB, where one can easily load a tar file from the file system as a module. USTAR is an adequate file system for tapes, not disks.

I would have done those changes myself, however the article seems to have been locked. Please note I'm new here, but I have some limited experience on other wikis. However, it seems concerning that articles get permanently protected so easily. I hope this edit protection is not really indefinite and will be lifted eventually. --[[User:Lukflug|Lukflug]] 07:23, 9 February 2022 (CST)
:Your first bullet point is dodgy. I wouldn't included it at all because I've seen bad results from taking recommendations seriously. The computing world is full of recommendations which are fine under most circumstances but harmful in others. Your other bullet points are good. The 2nd point gives a good idea of why and when you might want to extract files into another filesystem and the 3rd explains when you wouldn't. Neither of these burden newbies with recommendations to challenge their confidence. The rest of your comment seems to mix logical advice with opinion, and your opinions seem to be based on the fear that OSDev newbies will do the stupidest thing possible without firm guidance. :p While that's not entirely unfounded, I don't recall too many newbies trying hard to make a writeable tarfs. [[User:Eekee|Eekee]] 10:40, 9 February 2022 (CST)
::Yeah, I agree that I left the first bullet point a little vague and that the phrase "the recommended way" implies that someone authoritative is recommending it, even though that isn't the case. It could be rephrased as "tar files are usually manipulated by extracting it first into another file system", which is less prescriptive and more descriptive. My fear of beginners treating UStar like a disk file system (and yes, even making it writable) comes from my own initial misconception (I'm still very much a noob) and from my activity on the OSDev Discord server, where I saw many other beginners share the same misconceptions. And from what I have seen, these misconceptions stem from this article. Also, some didn't realize that it was the format for tar files, even though the "utility to create tar images" point should've given that away (so that might have been an issue of not reading). Thanks for the constructive criticism! --[[User:Lukflug|Lukflug]] 11:18, 9 February 2022 (CST)
:::I am banned from the OSdev Discord for reasons unknown, so I care very little what troubles people there are experiencing. I've made some edits to make it more clear what USTAR is and what its shortcomings as "filesystem" are, but I will defends its inclusion in a list like this. --[[User:Klange|klange]] 19:21, 11 February 2022 (CST)
:::I've edited the header of the USTAR page itself with some thoughts on why you would or wouldn't want to use it as a filesystem. It's rather shorter than my reasoning here. :) [[User:Eekee|Eekee]] 19:38, 27 March 2022 (CDT)

I also want to partially retract my comment regarding eager edit protection. The protection log (which I should've consulted before publishing that comment), told me this is the first change in protection in 12 years :P. But I still find it weird to fully protect a page without any discussion in the talk pages. In addition, I have just noticed two minor grammatical mistakes in the page, that I can't correct due to the protection:
* "Widely used, utility to create tar images are available for every mainstream OS": "utility" should either be plural, or it should be written as "an utility [...] is available".
* "Implementation Available as static library": I'm also pretty sure "available" should be lower case.
--[[User:Lukflug|Lukflug]] 11:28, 9 February 2022 (CST)
:I unlocked the page again. I primarily locked it to stop the repeated back and forth of reverts - I'm not too familiar with moderation tools on MediaWiki, but we don't seem to have too many options below full locking. That it led to a discussion on the talk page is good to see. I think the main reason we haven't messed with page permissions in a long time is that the wiki has been largely unmoderated - no one's had the necessary permissions until recently. --[[User:Klange|klange]] 23:00, 11 February 2022 (CST)
````

<a id="w004693"></a>

## W004693 — User talk:Revolution

Bron-ID: W004693
Namespace: User talk (3)
Revisie: 26978 | Datum: 2022-02-17T09:36:09Z
Bron: https://wiki.osdev.org/User_talk:Revolution
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ARevolution&oldid=26978

````text
Test

== Change from "LDT" to "Local Descriptor Table" ==

Thanks, Revolution! I agree with what you did. --[[User:Johnburger|John Burger]] 03:36, 17 February 2022 (CST)
````

<a id="w004694"></a>

## W004694 — User:Revolution

Bron-ID: W004694
Namespace: User (2)
Revisie: 26972 | Datum: 2022-02-16T17:37:20Z
Bron: https://wiki.osdev.org/User:Revolution
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARevolution&oldid=26972

````text
== Articles I've Worked On ==

;'''[[Global Descriptor Table]]'''
;'''[[Interrupt Descriptor Table]]'''
;'''[[GDT Tutorial]]'''
;'''[[Segment Selector]]'''
;'''[[Task State Segment]]'''
````

<a id="w004695"></a>

## W004695 — Segment selector

Bron-ID: W004695
Namespace: Main (0)
Revisie: 26939 | Datum: 2022-02-14T07:23:04Z
Bron: https://wiki.osdev.org/Segment_selector
Revisie-URL: https://wiki.osdev.org/index.php?title=Segment_selector&oldid=26939
Doorverwijzing: Segment Selector

````text
#REDIRECT [[Segment Selector]]
````

<a id="w004696"></a>

## W004696 — LDT

Bron-ID: W004696
Namespace: Main (0)
Revisie: 26961 | Datum: 2022-02-16T13:45:55Z
Bron: https://wiki.osdev.org/LDT
Revisie-URL: https://wiki.osdev.org/index.php?title=LDT&oldid=26961
Doorverwijzing: Local Descriptor Table

````text
#REDIRECT [[Local Descriptor Table]]
````

<a id="w004697"></a>

## W004697 — User:Rdrpenguin

Bron-ID: W004697
Namespace: User (2)
Revisie: 26976 | Datum: 2022-02-17T04:21:56Z
Bron: https://wiki.osdev.org/User:Rdrpenguin
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARdrpenguin&oldid=26976

````text
Hello! I'm new here. I'm working on a few OS projects; I'll post updates and favorite articles here.
````

<a id="w004698"></a>

## W004698 — Porting Rust standard library

Bron-ID: W004698
Namespace: Main (0)
Revisie: 29768 | Datum: 2025-11-22T05:27:48Z
Bron: https://wiki.osdev.org/Porting_Rust_standard_library
Revisie-URL: https://wiki.osdev.org/index.php?title=Porting_Rust_standard_library&oldid=29768

````text
{{rating|1}}
{{template:In_Progress}}

== Guide ==

This guide shows how to get the standard library to compile and run for a custom target.

By the end you should be able to compile a project with
<syntaxhighlight lang="bash">
cargo +dev-x86_64-unknown-myos r --target x86_64-unknown-myos
</syntaxhighlight>

===Get sources===

<syntaxhighlight lang="bash">
git clone https://github.com/rust-lang/rust
</syntaxhighlight>

===Configuration===

<syntaxhighlight lang="ini">
# config.toml
profile = "compiler"
change-id = 137215

[build]
host = ["x86_64-unknown-linux-gnu"]
target = ["x86_64-unknown-linux-gnu", "x86_64-unknown-myos"]

[rust]
incremental = true
</syntaxhighlight>

===Adding the target===
Adding a target to the rust compiler takes several files. rustc_target must be told about the new target with a base spec and a target spec. The bootstrap crate must be told that the new target, while not in the downloaded bootstrap compiler, is valid. Also, a test is added for the new target that checks assembly code generation. The test is not strictly necessary, but tidy, the rust compiler's style enforcer, will not pass without it, so it is good practice.


In addition, a <code>_start</code> symbol must be provided to rustc. This can be done in one of two ways. Either rustc can be given a crt0 object file that provides start, or the standard library can be modified to provide it for our target. The CRT option is done in the target files and thus shown here, while the standard library approach is described in the next section.

<syntaxhighlight lang="rust">
// compiler/rustc_target/src/spec/base/myos.rs
// Only if using external CRT to provide _start
use crate::spec::crt_objects;
use crate::spec::{Cc, LinkerFlavor, Lld, RelocModel, StackProbeType, TargetOptions};

pub(crate) fn opts() -> TargetOptions {
    TargetOptions {
        os: "myos".into(),
        linker: Some("rust-lld".into()),
        linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
        stack_probes: StackProbeType::Inline,
        relocation_model: RelocModel::Static,
        // Following two only if using external CRT to provide _start
        pre_link_objects: crt_objects::pre_myos(),
        post_link_objects: crt_objects::post_myos(),
        ..Default::default()
    }
}
</syntaxhighlight>


<syntaxhighlight lang="rust">
// compiler/rustc_target/src/spec/targets/x86_64_unknown_myos.rs
use crate::spec::{base, PanicStrategy, Target, TargetMetadata};

pub(crate) fn target() -> Target {
    let mut base = base::myos::opts();
    base.cpu = "x86-64".into();
    base.disable_redzone = true;
    base.panic_strategy = PanicStrategy::Abort;
    base.features = "-mmx,-sse,+soft-float".into();

    Target {
        llvm_target: "x86_64-unknown-none".into(),
        pointer_width: 64,
        data_layout:
            "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128".into(),
        arch: "x86_64".into(),
        options: base,
        metadata: TargetMetadata {
            description: None,
            tier: None,
            host_tools: None,
            std: None,
        },
    }
}
</syntaxhighlight>


<syntaxhighlight lang="diff">
--- a/compiler/rustc_target/src/spec/base/mod.rs
+++ b/compiler/rustc_target/src/spec/base/mod.rs
@@ -18,6 +18,7 @@
 pub(crate) mod linux_ohos;
 pub(crate) mod linux_uclibc;
 pub(crate) mod msvc;
+pub(crate) mod myos;
 pub(crate) mod netbsd;
 pub(crate) mod nto_qnx;
 pub(crate) mod openbsd;
</syntaxhighlight>


<syntaxhighlight lang="diff">
--- a/compiler/rustc_target/src/spec/mod.rs
+++ b/compiler/rustc_target/src/spec/mod.rs
@@ -1826,6 +1826,8 @@ fn $module() {
 
     ("x86_64-unknown-linux-none", x86_64_unknown_linux_none),
 
+    ("x86_64-unknown-myos", x86_64_unknown_myos),
+
 }
</syntaxhighlight>


<syntaxhighlight lang="diff">
--- a/compiler/rustc_target/src/spec/crt_objects.rs
+++ b/compiler/rustc_target/src/spec/crt_objects.rs
@@ -106,6 +106,14 @@ pub(super) fn post_mingw() -> CrtObjects {
     all("rsend.o")
 }
 
+pub(super) fn pre_myos() -> CrtObjects {
+    all(<PATH TO CRT0 HERE>)
+}
+
+pub(super) fn post_myos() -> CrtObjects {
+    new(&[])
+}
+
</syntaxhighlight>


<syntaxhighlight lang="diff">
--- a/src/bootstrap/src/core/sanity.rs
+++ b/src/bootstrap/src/core/sanity.rs
@@ -40,6 +40,7 @@ pub struct Finder {
 #[cfg(not(feature = "bootstrap-self-test"))]
 const STAGE0_MISSING_TARGETS: &[&str] = &[
     // just a dummy comment so the list doesn't get onelined
+    "x86_64-unknown-myos",
 ];
</syntaxhighlight>

<syntaxhighlight lang="diff">
--- a/src/bootstrap/src/lib.rs
+++ b/src/bootstrap/src/lib.rs
@@ -94,7 +94,7 @@
     (Some(Mode::Std), "no_sync", None),
     /* Extra values not defined in the built-in targets yet, but used in std */
     (Some(Mode::Std), "target_env", Some(&["libnx", "p2"])),
-    (Some(Mode::Std), "target_os", Some(&["visionos"])),
+    (Some(Mode::Std), "target_os", Some(&["myos", "visionos"])),
     (Some(Mode::Std), "target_arch", Some(&["arm64ec", "spirv", "nvptx", "xtensa"])),
     (Some(Mode::ToolStd), "target_os", Some(&["visionos"])),
</syntaxhighlight>


<syntaxhighlight lang="diff">
--- a/tests/assembly/targets/targets-elf.rs
+++ b/tests/assembly/targets/targets-elf.rs
@@ -561,6 +561,9 @@
 //@ revisions: x86_64_unknown_linux_none
 //@ [x86_64_unknown_linux_none] compile-flags: --target x86_64-unknown-linux-none
 //@ [x86_64_unknown_linux_none] needs-llvm-components: x86
+//@ revisions: x86_64_unknown_myos
+//@ [x86_64_unknown_myos] compile-flags: --target x86_64-unknown-myos
+//@ [x86_64_unknown_myos] needs-llvm-components: x86
 //@ revisions: x86_64_unknown_netbsd
 //@ [x86_64_unknown_netbsd] compile-flags: --target x86_64-unknown-netbsd
 //@ [x86_64_unknown_netbsd] needs-llvm-components: x86
</syntaxhighlight>

===Adapt library/std===
In addition to rustc, std must also be modified to support the target. By default, std will error on build if the OS isn't explicitly supported, so we must add our OS to the list of supported OSes. In addition, we must provide a PAL (Platform Abstraction Layer) to tell std how to interact with our OS.
<syntaxhighlight lang="diff">
--- a/library/std/build.rs
+++ b/library/std/build.rs
@@ -48,6 +48,7 @@ fn main() {
         || target_os == "uefi"
         || target_os == "teeos"
         || target_os == "zkvm"
+        || target_os == "myos"
 
         // See src/bootstrap/src/core/build_steps/synthetic_targets.rs
         || env::var("RUSTC_BOOTSTRAP_SYNTHETIC_TARGET").is_ok()
</syntaxhighlight>

Copy <code>library/std/src/sys/pal/unsupported</code> to <code>library/std/src/sys/pal/myos</code>.

<syntaxhighlight lang="diff">
--- a/library/std/src/sys/pal/mod.rs
+++ b/library/std/src/sys/pal/mod.rs
@@ -61,6 +61,9 @@
     } else if #[cfg(target_os = "zkvm")] {
         mod zkvm;
         pub use self::zkvm::*;
+    } else if #[cfg(target_os = "myos")] {
+        mod myos;
+        pub use self::myos::*;
     } else {
         mod unsupported;
         pub use self::unsupported::*;
</syntaxhighlight>If using the standard library to provide <code>_start</code>, you will need to add a module to the PAL containing a <code>#[no_mangle]</code> and <code>extern "C"</code> <code>_start</code> function. The following file works as a basic template. Note that you will have to add a <code>mod start;</code> line to the PAL's <code>mod.rs</code> file.<syntaxhighlight lang="rust">
// library/std/src/sys/pal/myos/start.rs
use crate::ffi::{c_char, c_int};
use crate::ptr;

extern "C" {
    fn main(argc: c_int, argv: *const *const c_char) -> c_int;
}

#[no_mangle]
#[allow(unused)]
pub extern "C" fn _start() {
    unsafe {
        main(0, ptr::null());
    };
}
</syntaxhighlight>
If your kernel does not provide a stack in <code>rsp</code> or provides its CLI arguments in a way impossible to read from a C function, you will have to use [https://github.com/rust-lang/rfcs/blob/master/text/2972-constrained-naked.md naked functions] to provide <code>_start</code> as assembly.

===Add toolchain===

<syntaxhighlight lang="bash">
rustup toolchain link dev-x86_64-unknown-myos ~/Documents/rust/rust/build/x86_64-unknown-linux-gnu/stage2
</syntaxhighlight>

== Making the standard library functional ==
Even though both rustc and std know about the target, not even an empty program will run. This is because the standard library requires two things from the OS to be able to initialize. A memory allocator and [[Thread Local Storage|thread local storage]] (TLS).  

===Memory allocator===
Implementing a memory allocator is done in the PAL's <code>alloc.rs</code> file and requires implementing the GlobalAlloc trait. The following bump allocator will work to let the stdlib initialize.`<syntaxhighlight lang="rust">
// library/std/src/sys/pal/myos/alloc.rs
use crate::{
    alloc::{GlobalAlloc, Layout, System},
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

#[repr(align(4096))]
struct HeapData([u8; 65536]);

static mut HEAP_DATA: HeapData = HeapData([0; 65536]);
static HEAP_USED: AtomicUsize = AtomicUsize::new(0);

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() > 8 {
            return ptr::null_mut();
        }
        let num_blocks = if layout.size() % 8 == 0 {
            layout.size() / 8
        } else {
            (layout.size() / 8) + 1
        };
        HEAP_USED.fetch_add(num_blocks, Ordering::Relaxed);
        let ptr = unsafe { ptr::addr_of_mut!(HEAP_DATA.0[HEAP_USED.load(Ordering::Relaxed) - num_blocks ]) as *mut u8 };
        ptr
    }

    #[inline]
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
</syntaxhighlight>

=== Thread local storage===

====Global statics (easiest, single-threaded only)====
{{Warning | This WILL not work on multi-threaded systems.}}
If your OS is single-threaded or you don't want to implement full TLS before adding multithreading to your PAL, Rust can implement TLS via global statics. 
<syntaxhighlight lang="diff">
--- a/library/std/src/sys/thread_local/mod.rs
+++ b/library/std/src/sys/thread_local/mod.rs
@@ -7,7 +7,7 @@
 // "static" is for single-threaded platforms where a global static is sufficient.
 
 cfg_if::cfg_if! {
-    if #[cfg(any(all(target_family = "wasm", not(target_feature = "atomics")), target_os = "uefi"))] {
+    if #[cfg(any(all(target_family = "wasm", not(target_feature = "atomics")), target_os = "uefi", target_os = "myos"))] {
         #[doc(hidden)]
         mod static_local;
         #[doc(hidden)]
</syntaxhighlight>

====OS APIs (slow, easier than ELF native) ====
This option is slower than the other multi-threaded compatible version, but is easier to implement, requiring only to implement the functions in the PAL's <code>thread_local_key.rs</code> file. The key is a value unique to a TLS variable, though shared amongst threads. This is rustc's default method of doing TLS.

====Native ELF TLS (fastest)====
This option is the fastest way of doing TLS, but requires more complex suport from the OS. To implement it, see the linked wiki article on TLS. Enabling it in rustc is done by adding a target option for your OS.
<syntaxhighlight lang="diff">
--- a/compiler/rustc_target/src/spec/base/myos.rs
+++ b/compiler/rustc_target/src/spec/base/myos.rs
@@ -10,6 +10,7 @@ pub fn opts() -> TargetOptions {
         relocation_model: RelocModel::Static,
         pre_link_objects: crt_objects::pre_myos(),
         post_link_objects: crt_objects::post_myos(),
+        has_thread_local: true,
         ..Default::default()
     }
 }
</syntaxhighlight>

===Basic printing ===
With memory allocation and TLS in place, the Rust stdlib is now functional enough to not crash. However, it's fairly useless as no IO functions work. Adding print output is a natural next step, and will also allow easier debugging of panics.

Adding print support is fairly simple given the architecture of the PAL, and requires filling out the Write implementations of stdout/err in <code>stdio.rs</code>. In addition, the panic_output function should be set to the desired output stream for panic messages, usually stderr. 

Example code is provided below. The given code assumes syscalls are done with interrupt 0x80 with the number in rax and the first parameter in rcx, and syscall 0 takes a byte to write to some text output device. You will probably need to change the syscall number and parameters for your OS, but this should work as a basic starting point.
<syntaxhighlight lang="diff">
--- a/library/std/src/sys/pal/myos/stdio.rs
+++ b/library/std/src/sys/pal/myos/stdio.rs
@@ -1,3 +1,4 @@
+use crate::arch::asm;
 use crate::io;
 
 pub struct Stdin;
@@ -24,6 +25,11 @@ pub const fn new() -> Stdout {
 
 impl io::Write for Stdout {
     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
+        for byte in buf {
+            unsafe {
+                asm!("int 0x80", in("rax") 0, in ("rcx") *byte as u64);
+            };
+        }
         Ok(buf.len())
     }
 
@@ -40,6 +46,11 @@ pub const fn new() -> Stderr {
 
 impl io::Write for Stderr {
     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
+        for byte in buf {
+            unsafe {
+                asm!("int 0x80", in("rax") 0, in ("rcx") *byte as u64);
+            };
+        }
         Ok(buf.len())
     }
 
@@ -54,6 +65,6 @@ pub fn is_ebadf(_err: &io::Error) -> bool {
     true
 }
 
-pub fn panic_output() -> Option<Vec<u8>> {
-    None
+pub fn panic_output() -> Option<impl io::Write> {
+    Some(Stderr::new())
 }
</syntaxhighlight>
Now you should be able to compile and run the default Rust hello world program for your target and see it print to screen.

==Runtime==

===Integrating a crate===

If you use a crate for the runtime (e.g. <code>myos_rt</code>), you can add it as a dependency to the standard library:

<syntaxhighlight lang="toml">
[target.'cfg(target_os = "myos")'.dependencies]
myos_rt = { version = "*", features = ["rustc-dep-of-std"] }
</syntaxhighlight>

The <code>rustc-dep-of-std</code> feature is necessary since <code>libcore</code> et al. don't exist yet.

Modify the <code>Cargo.toml</code> of your runtime library to include the feature and propagate it up, e.g.:

<syntaxhighlight lang="toml">
[dependencies]
cfg-if = "1.0"
core = { version = "1.0.0", optional = true, package = "rustc-std-workspace-core" }
compiler_builtins = { version = "0.1", optional = true }

[features]
rustc-dep-of-std = [
	"core",
	"compiler_builtins",
	"cfg-if/rustc-dep-of-std"
]
</syntaxhighlight>

Do keep in mind that the same crate with different feature flags are seen as [https://github.com/rust-lang/cargo/issues/2363 <strong>different crates</strong> by the compiler]. This means that if you any globals in the runtime crate and have a project that uses both stdlib and your runtime crate there will be two separate sets of those globals. One way to work around this is by giving these globals an explicit name with <code>#[export_name = "__rt_whatever"]</code> and weakly linking them with <code>#[linkage = "weak"]</code>.

==Troubleshooting==

===error[E0463]: can't find crate for `compiler_builtins`===

Add <code>compiler_builtins</code> as a dependency for the crates you use in stdlib, e.g.:

<syntaxhighlight lang="ini">
[dependencies]
core = { version = "1.0.0", optional = true, package = "rustc-std-workspace-core" }
compiler_builtins = { version = "*", optional = true } # <--

[features]
rustc-dep-of-std = [
	"core",
	"compiler_builtins/rustc-dep-of-std", # <--
]
</syntaxhighlight>

[[Category:Rust]]
[[Category:Standard Libraries]]
````

<a id="w004699"></a>

## W004699 — File:BleskOS 28 2 2022.png

Bron-ID: W004699
Namespace: File (6)
Revisie: 26992 | Datum: 2022-02-28T13:51:28Z
Bron: https://wiki.osdev.org/File:BleskOS_28_2_2022.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABleskOS_28_2_2022.png&oldid=26992

````text

````

<a id="w004700"></a>

## W004700 — User:Superleaf1995/uDOS

Bron-ID: W004700
Namespace: User (2)
Revisie: 27010 | Datum: 2022-03-22T15:10:44Z
Bron: https://wiki.osdev.org/User:Superleaf1995/uDOS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASuperleaf1995%2FuDOS&oldid=27010
Doorverwijzing: User:Superleaf1995/UDOS

````text
#REDIRECT [[User:Superleaf1995/UDOS]]
````

<a id="w004701"></a>

## W004701 — File:Banana os 2.jpg

Bron-ID: W004701
Namespace: File (6)
Revisie: 27013 | Datum: 2022-03-27T05:15:47Z
Bron: https://wiki.osdev.org/File:Banana_os_2.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABanana_os_2.jpg&oldid=27013

````text
Banana operating system image
````

<a id="w004702"></a>

## W004702 — Limine Bare Bones

Bron-ID: W004702
Namespace: Main (0)
Revisie: 30950 | Datum: 2026-09-15T06:39:45Z
Bron: https://wiki.osdev.org/Limine_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Limine_Bare_Bones&oldid=30950

````text
{{BeginnersWarning}}
{{Rating|1}}
{{Template:Kernel designs}}

The Limine Boot Protocol is the native boot protocol provided by the [[Limine]] bootloader. It is designed to overcome shortcomings of common boot protocols used by hobbyist OS developers, such as [[Multiboot]].

It provides cutting edge features such as 5-level paging support, 64-bit [[Long Mode]] support, and direct higher half kernel loading.

The Limine boot protocol is firmware and architecture agnostic. It supports x86-64, aarch64, riscv64, and loongarch64.

This article will demonstrate how to write a small Limine-compliant x86-64 kernel in (GNU) [[C]], and boot it using the [[Limine]] bootloader.

Additionally, it is highly recommended to check out [https://github.com/Limine-Bootloader/limine-c-template-x86-64 this repository] as it provides more complete, buildable, portable template code to go along with this guide.

==Overview==

For this example, we will create these 2 files to create the basic directory tree of our project:
* src/main.c
* linker.lds

As one may notice, there is no "entry point" assembly stub, as one is not necessary with the Limine protocol when using a language which can make use of a standard SysV x86 [[Calling Conventions|calling convention]].

Furthermore, we will download the header file <code>limine.h</code> which defines structures and constants that we will use to interact with the bootloader from [https://github.com/Limine-Bootloader/limine-protocol/raw/refs/heads/trunk/include/limine.h here], and place it in the <code>src</code> directory.

Obviously, this is just a bare bones example, and one should always refer to the [https://github.com/Limine-Bootloader/limine-protocol/blob/trunk/PROTOCOL.md Limine protocol specification] for more details and information.

===src/main.c===

This is the kernel "main".

<syntaxhighlight lang="c">
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <limine.h>

// Set the base revision to 6, this is recommended as this is the latest
// base revision described by the Limine boot protocol specification.
// See specification for further info.

__attribute__((used, section(".limine_requests")))
static volatile uint64_t limine_base_revision[] = LIMINE_BASE_REVISION(6);

// The Limine requests can be placed anywhere, but it is important that
// the compiler does not optimise them away, so, usually, they should
// be made volatile or equivalent, _and_ they should be accessed at least
// once or marked as used with the "used" attribute as done here.

__attribute__((used, section(".limine_requests")))
static volatile struct limine_framebuffer_request framebuffer_request = {
    .id = LIMINE_FRAMEBUFFER_REQUEST_ID,
    .revision = 0
};

// Finally, define the start and end markers for the Limine requests.
// These can also be moved anywhere, to any .c file, as seen fit.

__attribute__((used, section(".limine_requests_start")))
static volatile uint64_t limine_requests_start_marker[] = LIMINE_REQUESTS_START_MARKER;

__attribute__((used, section(".limine_requests_end")))
static volatile uint64_t limine_requests_end_marker[] = LIMINE_REQUESTS_END_MARKER;

// GCC and Clang reserve the right to generate calls to the following
// 4 functions even if they are not directly called.
// Implement them as the C specification mandates.
// DO NOT remove or rename these functions, or stuff will eventually break!
// They CAN be moved to a different .c file.

void *memcpy(void *restrict dest, const void *restrict src, size_t n) {
    uint8_t *pdest = dest;
    const uint8_t *psrc = src;

    for (size_t i = 0; i < n; i++) {
        pdest[i] = psrc[i];
    }

    return dest;
}

void *memset(void *s, int c, size_t n) {
    uint8_t *p = s;

    for (size_t i = 0; i < n; i++) {
        p[i] = (uint8_t)c;
    }

    return s;
}

void *memmove(void *dest, const void *src, size_t n) {
    uint8_t *pdest = dest;
    const uint8_t *psrc = src;

    if ((uintptr_t)src > (uintptr_t)dest) {
        for (size_t i = 0; i < n; i++) {
            pdest[i] = psrc[i];
        }
    } else if ((uintptr_t)src < (uintptr_t)dest) {
        for (size_t i = n; i > 0; i--) {
            pdest[i-1] = psrc[i-1];
        }
    }

    return dest;
}

int memcmp(const void *s1, const void *s2, size_t n) {
    const uint8_t *p1 = s1;
    const uint8_t *p2 = s2;

    for (size_t i = 0; i < n; i++) {
        if (p1[i] != p2[i]) {
            return p1[i] < p2[i] ? -1 : 1;
        }
    }

    return 0;
}

// Halt and catch fire function.
static void hcf(void) {
    for (;;) {
        asm ("hlt");
    }
}

// Scale an 8-bit colour channel value to the size the framebuffer gives the
// channel and move it into place within a pixel.
static uint32_t fb_channel(uint8_t value, uint8_t mask_size, uint8_t mask_shift) {
    uint64_t max = ((uint64_t)1 << mask_size) - 1;
    return (uint32_t)((value * max / 255) << mask_shift);
}

// Build a pixel from 8-bit red, green and blue values following the channel
// layout of the framebuffer.
static uint32_t fb_pixel(struct limine_framebuffer *fb, uint8_t red, uint8_t green, uint8_t blue) {
    return fb_channel(red, fb->red_mask_size, fb->red_mask_shift)
         | fb_channel(green, fb->green_mask_size, fb->green_mask_shift)
         | fb_channel(blue, fb->blue_mask_size, fb->blue_mask_shift);
}

// Print a nice pattern to a framebuffer as an example.
static void fb_pattern(struct limine_framebuffer *fb) {
    volatile uint32_t *fb_ptr = fb->address;
    for (size_t y = 0; y < fb->height; y++) {
        for (size_t x = 0; x < fb->width; x++) {
            uint8_t nX = x * 255 / fb->width;
            uint8_t nY = y * 255 / fb->height;
            fb_ptr[y * (fb->pitch / 4) + x] = fb_pixel(fb, 0, nY, nX);
        }
    }
}

// The following will be our kernel's entry point.
// If renaming kmain() to something else, make sure to change the
// linker script accordingly.
void kmain(void) {
    // Ensure the bootloader actually understands our base revision (see spec).
    if (LIMINE_BASE_REVISION_SUPPORTED(limine_base_revision) == false) {
        hcf();
    }

    // Ensure we got a framebuffer.
    if (framebuffer_request.response == NULL
     || framebuffer_request.response->framebuffer_count < 1) {
        hcf();
    }

    // Print the pattern to every framebuffer.
    for (uint64_t i = 0; i < framebuffer_request.response->framebuffer_count; i++) {
        struct limine_framebuffer *framebuffer = framebuffer_request.response->framebuffers[i];

        // Ensure the framebuffer has 32-bit RGB pixels, the only kind we handle.
        if (framebuffer->memory_model != LIMINE_FRAMEBUFFER_RGB || framebuffer->bpp != 32) {
            hcf();
        }

        fb_pattern(framebuffer);
    }

    // We're done, just hang...
    hcf();
}

</syntaxhighlight>

===linker.lds===

This is going to be our linker script describing where our sections will end up in memory.

<syntaxhighlight lang="c">
/* Tell the linker that we want an x86_64 ELF64 output file */
OUTPUT_FORMAT(elf64-x86-64)

/* We want the symbol kmain to be our entry point */
ENTRY(kmain)

/* Define the program headers we want so the bootloader gives us the right */
/* MMU permissions; this also allows us to exert more control over the linking */
/* process. */
PHDRS
{
    limine_requests PT_LOAD;
    text PT_LOAD;
    rodata PT_LOAD;
    data PT_LOAD;
}

SECTIONS
{
    /* We want to be placed in the topmost 2GiB of the address space, for optimisations */
    /* and because that is what the Limine spec mandates. */
    /* Any address in this region will do, but often 0xffffffff80000000 is chosen as */
    /* that is the beginning of the region. */
    . = 0xffffffff80000000;

    /* Define a section to contain the Limine requests and assign it to its own PHDR */
    .limine_requests : {
        KEEP(*(.limine_requests_start))
        KEEP(*(.limine_requests))
        KEEP(*(.limine_requests_end))
    } :limine_requests

    /* Move to the next memory page for .text */
    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .text : {
        *(.text .text.*)
    } :text

    /* Move to the next memory page for .rodata */
    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    /* Add a .note.gnu.build-id output section in case a build ID flag is added to the */
    /* linker command. */
    .note.gnu.build-id : {
        *(.note.gnu.build-id)
    } :rodata

    /* Move to the next memory page for .data */
    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .data : {
        *(.data .data.*)
    } :data

    /* NOTE: .bss needs to be the last thing mapped to :data, otherwise lots of */
    /* unnecessary zeros will be written to the binary. */
    /* If you need, for example, .init_array and .fini_array, those should be placed */
    /* above this. */
    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } :data

    /* Discard .note.* and .eh_frame* since they may cause issues on some hosts. */
    /DISCARD/ : {
        *(.eh_frame*)
        *(.note .note.*)
    }
}
</syntaxhighlight>

==Building the kernel and creating an image==

===GNUmakefile===

In order to build our kernel, we are going to use a Makefile. Since we're going to use
GNU make specific features, we call this file <code>GNUmakefile</code> instead, so only
GNU make will process it.

<syntaxhighlight lang="make">
# Nuke built-in rules.
.SUFFIXES:

# Delete the target of a failed recipe.
.DELETE_ON_ERROR:

# This is the name that our final executable will have.
# Change as needed.
override OUTPUT := myos

# User controllable toolchain and toolchain prefix.
TOOLCHAIN :=
TOOLCHAIN_PREFIX :=
ifneq ($(TOOLCHAIN),)
    ifeq ($(TOOLCHAIN_PREFIX),)
        TOOLCHAIN_PREFIX := $(TOOLCHAIN)-
    endif
endif

# User controllable C compiler command.
ifneq ($(TOOLCHAIN_PREFIX),)
    CC := $(TOOLCHAIN_PREFIX)gcc
else
    CC := cc
endif

# User controllable linker command.
LD := $(TOOLCHAIN_PREFIX)ld

# Defaults overrides for variables if using "llvm" as toolchain.
ifeq ($(TOOLCHAIN),llvm)
    CC := clang
    LD := ld.lld
endif

# User controllable C flags.
CFLAGS := -g -O2 -pipe

# User controllable C preprocessor flags. We set none by default.
CPPFLAGS :=

# User controllable nasm flags.
NASMFLAGS := -g

# User controllable linker flags. We set none by default.
LDFLAGS :=

# Check if CC is Clang.
override CC_IS_CLANG := $(shell ! $(CC) --version 2>/dev/null | grep -q '^Target: '; echo $$?)

# If the C compiler is Clang, set the target as needed.
ifeq ($(CC_IS_CLANG),1)
    override CC += \
        -target x86_64-unknown-none-elf
endif

# Internal C flags that should not be changed by the user.
override CFLAGS += \
    -Wall \
    -Wextra \
    -std=gnu11 \
    -ffreestanding \
    -fno-stack-protector \
    -fno-stack-check \
    -fno-lto \
    -fno-PIC \
    -ffunction-sections \
    -fdata-sections \
    -m64 \
    -march=x86-64 \
    -mabi=sysv \
    -mno-80387 \
    -mno-mmx \
    -mno-sse \
    -mno-sse2 \
    -mno-red-zone \
    -mcmodel=kernel

# Internal C preprocessor flags that should not be changed by the user.
override CPPFLAGS := \
    -I src \
    $(CPPFLAGS) \
    -MMD \
    -MP

# Internal nasm flags that should not be changed by the user.
override NASMFLAGS := \
    -f elf64 \
    $(patsubst -g,-g -F dwarf,$(NASMFLAGS)) \
    -i src/ \
    -Wall

# Internal linker flags that should not be changed by the user.
override LDFLAGS += \
    -m elf_x86_64 \
    -nostdlib \
    -static \
    -z max-page-size=0x1000 \
    -z noexecstack \
    --gc-sections \
    -T linker.lds

# Check if the linker supports -no-pie and enable it if it does.
override LD_HAS_NO_PIE := $(shell ! $(LD) --help 2>/dev/null | grep -qE '(^|[[:space:]])--?no-pie([[:space:]]|$$)'; echo $$?)

ifeq ($(LD_HAS_NO_PIE),1)
    override LDFLAGS += \
        -no-pie
endif

# Use "find" to glob all *.c, *.S, and *.asm files in the tree and obtain the
# object and header dependency file names.
override SRCFILES := $(shell find -L src -type f 2>/dev/null | LC_ALL=C sort)
override CFILES := $(filter %.c,$(SRCFILES))
override ASFILES := $(filter %.S,$(SRCFILES))
override NASMFILES := $(filter %.asm,$(SRCFILES))
override OBJ := $(addprefix obj/,$(CFILES:.c=.c.o) $(ASFILES:.S=.S.o) $(NASMFILES:.asm=.asm.o))
override HEADER_DEPS := $(addprefix obj/,$(CFILES:.c=.c.d) $(ASFILES:.S=.S.d) $(NASMFILES:.asm=.asm.d))

# Default target. This must come first, before header dependencies.
.PHONY: all
all: bin/$(OUTPUT)

# Include header dependencies.
-include $(HEADER_DEPS)

# Link rules for the final executable.
bin/$(OUTPUT): GNUmakefile linker.lds $(OBJ)
	mkdir -p "$(dir $@)"
	$(LD) $(LDFLAGS) $(OBJ) -o $@

# Compilation rules for *.c files.
obj/%.c.o: %.c GNUmakefile
	mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) $(CPPFLAGS) -c $< -o $@

# Compilation rules for *.S files.
obj/%.S.o: %.S GNUmakefile
	mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) $(CPPFLAGS) -c $< -o $@

# Compilation rules for *.asm (nasm) files.
obj/%.asm.o: %.asm GNUmakefile
	mkdir -p "$(dir $@)"
	nasm $(NASMFLAGS) -MD $(@:.o=.d) -MP $< -o $@

# Remove object files and the final executable.
.PHONY: clean
clean:
	rm -rf bin obj
</syntaxhighlight>

===limine.conf===

This file is parsed by Limine and it describes boot entries and other bootloader configuration variables. Further information [https://github.com/Limine-Bootloader/Limine/blob/trunk/CONFIG.md here].

<syntaxhighlight lang="ini">
# Timeout in seconds that Limine will use before automatically booting.
timeout: 5

# The entry name that will be displayed in the boot menu.
/myOS
    # We use the Limine boot protocol.
    protocol: limine

    # Path to the kernel to boot. boot():/ represents the partition on which limine.conf is located.
    path: boot():/boot/myos
</syntaxhighlight>

===Compiling the kernel===

We can now build our example kernel by running <code>make</code>. This command, if successful, should generate, inside the <code>bin</code> directory, a file called <code>myos</code> (or the chosen kernel name). This is our Limine protocol-compliant kernel executable.

===Compiling the kernel on macOS===

''If you are not using macOS, you can skip this section.''

The macOS Xcode toolchain uses Mach-O binaries, and not the ELF binaries required for this Limine-compliant kernel. A solution is to build a [[GCC Cross-Compiler]], or to obtain one from [https://brew.sh homebrew] by installing the <code>x86_64-elf-gcc</code> package. After one of these is done, build using <code>make TOOLCHAIN_PREFIX=x86_64-elf-</code>.

===Creating the image===

We can now create either an ISO or a hard disk/USB drive image with our kernel on it. [[Limine]] can boot on both [[BIOS]] and [[UEFI]] if the image is set up to do so, which is what we are going to do.

====Creating an ISO====

In this example we are going to create a CD-ROM ISO capable of booting on both [[UEFI]] and legacy [[BIOS]] systems.

For this to work, we will need the <code>xorriso</code> utility.

These are shell commands. They can also be compiled into a script or Makefile.

<syntaxhighlight lang="bash">
# Download and extract the latest Limine binary release.
curl -fL -o limine-binary.tar.gz https://github.com/Limine-Bootloader/Limine/releases/latest/download/limine-binary.tar.gz
gunzip < limine-binary.tar.gz | tar -xf -

# Build "limine" utility.
make -C limine-binary

# Create a directory which will be our ISO root.
mkdir -p iso_root

# Copy the relevant files over.
mkdir -p iso_root/boot
cp -v bin/myos iso_root/boot/
mkdir -p iso_root/boot/limine
cp -v limine.conf limine-binary/limine-bios.sys limine-binary/limine-bios-cd.bin \
      limine-binary/limine-uefi-cd.bin iso_root/boot/limine/

# Create the EFI boot tree and copy Limine's EFI executables over.
mkdir -p iso_root/EFI/BOOT
cp -v limine-binary/BOOTX64.EFI iso_root/EFI/BOOT/
cp -v limine-binary/BOOTIA32.EFI iso_root/EFI/BOOT/

# Create the bootable ISO.
xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin \
        -no-emul-boot -boot-load-size 4 -boot-info-table -hfsplus \
        -apm-block-size 2048 --efi-boot boot/limine/limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image --protective-msdos-label \
        iso_root -o image.iso

# Install Limine stage 1 and 2 for legacy BIOS boot.
./limine-binary/limine bios-install image.iso
</syntaxhighlight>

====Creating a hard disk/USB drive image====

In this example, we'll create an [[MBR]] partition table using <code>sgdisk</code>, containing a single FAT partition, also known as the ESP in EFI terminology, which will store our kernel, configs, and bootloader.

For this to work, we will need the <code>sgdisk</code> (usually from the <code>gdisk</code> or <code>gptfdisk</code> packages) and <code>mtools</code> utilities.

This example is more involved and is made up of more steps than creating an ISO image.

These are shell commands. They can also be compiled into a script or Makefile.

<syntaxhighlight lang="bash">
# Size of the image, in MiB.
HDD_SIZE=64

# Geometry to format the partition with. Older mtools require one; 64 heads of
# 32 sectors make a cylinder exactly 1 MiB in size.
HDD_HEADS=64
HDD_SECTORS_PER_TRACK=32
HDD_CYLINDER_SECTORS=$(( HDD_HEADS * HDD_SECTORS_PER_TRACK ))

# Partition layout. sgdisk lays the partition out as GPT before converting the
# table to MBR, so the first and last cylinders are left to the GPT structures.
HDD_PART_START=$HDD_CYLINDER_SECTORS
HDD_PART_SECTORS=$(( (HDD_SIZE - 2) * HDD_CYLINDER_SECTORS ))
HDD_PART_END=$(( HDD_PART_START + HDD_PART_SECTORS - 1 ))
HDD_PART_OFFSET=$(( HDD_PART_START * 512 ))

# Create an empty zeroed-out image file.
dd if=/dev/zero bs=1024k count=0 seek=$HDD_SIZE of=image.hdd

# Create a partition table with a single partition.
PATH=$PATH:/usr/sbin:/sbin sgdisk image.hdd -n 1:$HDD_PART_START:$HDD_PART_END -t 1:ef00 -m 1

# Download and extract the latest Limine binary release.
curl -fL -o limine-binary.tar.gz https://github.com/Limine-Bootloader/Limine/releases/latest/download/limine-binary.tar.gz
gunzip < limine-binary.tar.gz | tar -xf -

# Build "limine" utility.
make -C limine-binary

# Install the Limine BIOS stages onto the image.
./limine-binary/limine bios-install image.hdd

# Format the partition as FAT. mtools are given the partition as a byte offset
# into the image, along with its size and geometry, which older releases cannot
# work out by themselves.
mformat -i image.hdd@@$HDD_PART_OFFSET -T $HDD_PART_SECTORS -h $HDD_HEADS -s $HDD_SECTORS_PER_TRACK ::

# Make relevant subdirectories.
mmd -i image.hdd@@$HDD_PART_OFFSET ::/EFI ::/EFI/BOOT ::/boot ::/boot/limine

# Copy over the relevant files.
mcopy -i image.hdd@@$HDD_PART_OFFSET bin/myos ::/boot
mcopy -i image.hdd@@$HDD_PART_OFFSET limine.conf limine-binary/limine-bios.sys ::/boot/limine
mcopy -i image.hdd@@$HDD_PART_OFFSET limine-binary/BOOTX64.EFI ::/EFI/BOOT
mcopy -i image.hdd@@$HDD_PART_OFFSET limine-binary/BOOTIA32.EFI ::/EFI/BOOT
</syntaxhighlight>

==Conclusions==

If everything above has been completed successfully, you should now have a bootable ISO or hard drive/USB image containing your 64-bit higher half Limine protocol-compliant kernel and Limine to boot it. Once the kernel is successfully booted, you should see a colour gradient filling the screen: black in the top left corner, turning green downwards and blue to the right.

== See Also ==

=== Articles ===

* [[Limine]]
* [[Multiboot]]

=== External Links ===

* [https://github.com/Limine-Bootloader/limine-protocol/blob/trunk/PROTOCOL.md Limine protocol specification]
* [https://github.com/Limine-Bootloader/limine-c-template-x86-64 Buildable Limine protocol based kernel project template in C for x86-64]

[[Category:Bare bones tutorials]]
[[Category:Bootloaders]]
[[Category:C]]
[[Category:Higher Half Barebones]]
````

<a id="w004703"></a>

## W004703 — User:Verticalegg/How To U-Boot

Bron-ID: W004703
Namespace: User (2)
Revisie: 27030 | Datum: 2022-04-07T00:56:23Z
Bron: https://wiki.osdev.org/User:Verticalegg/How_To_U-Boot
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AVerticalegg%2FHow_To_U-Boot&oldid=27030

````text
U-Boot is a messy thing. 

<code>&lt;rant&gt;</code>
It's a bootloader used by Chromebooks, so you'd expect there to be a guide or something, but the only thing I could find was a README on the [https://github.com/u-boot/u-boot GitHub repo] written in '''friggin plain text'''! Come on, it's not 1995 anymore, we can use Markdown.
<code>&lt;/rant&gt;</code>

The point is, I don't want you to have to go through the mess of figuring out how to get it to work. Beyond that, I don't think there's much documentation about U-Boot on the internet, and I wanted a place to compile my random musings so I can find them later. So. U-Boot, here we come.

Note: the code in this article is mainly based around RISC-V. It should be fairly evident what to change based on arch, but you might want to check out the resources if you're confused.

== Getting U-Boot set up ==
First, you're gonna need the source code.
<source lang="text">
git clone https://github.com/u-boot/u-boot.git
cd u-boot
</source>
Configure based on the board you want by plugging it into <code>make</code>. The usable configs are in the configs directory - in my case I want to use the QEMU RISCV board, with S-mode enabled.
<source lang="text">
make qemu-riscv64_smode_defconfig
</source>
It'll spit out a bunch of lines, then you can move onto cross compiling it. By the way, it is '''absolutely critical''' you use the <code>''arch''-linux-gnu</code> target, not the <code>''arch''-unknown-elf</code> target all the other tutorials tell you to use. Otherwise, you get an error completely unrelated that will make you incredibly annoyed with yourself until you figure out your stupid mistake an hour and a half later. Just make sure you use the right compiler, okay?
<source lang="text">
CROSS_COMPILE=riscv64-linux-gnu- make
</source>
Okay, cool. We now have a shiny new U-Boot binary! Now we just have to somehow get it to <del>recognize our kernel</del> run in QEMU. Time for...

== Platform-Specific Setup ==
=== RISC-V ===
We're gonna need a custom build of OpenSBI to get U-Boot working. As above, pull the source from GitHub:
<source lang="text">
git clone https://github.com/riscv-software-src/opensbi.git
cd opensbi
</source>
Unlike U-Boot, OpenSBI doesn't require any configuring before you build it, but on the command line when making. Select a platform from the platforms directory. I'm using QEMU, so picking <code>generic</code>. Make sure to set the cross compiler before doing this, too.
<source lang="text">
CROSS_COMPILE=riscv64-linux-gnu- make PLATFORM=generic
</source>
Cool! Now we can boot our own OpenSBI build, so we can use <code>fw_jump.elf</code> instead of whatever QEMU ships with by default.

== Test Boot with QEMU ==
Alright, moment of truth #1.
<source lang="text">
qemu-system-riscv64  -M virt --bios opensbi/build/platform/generic/firmware/fw_jump.elf --kernel u-boot/u-boot.bin
</source>
It works! You ''should'' get a U-Boot prompt here, but if you don't, recheck your work and try again. As with a lot of OSDev, it's annoying to debug. You could maybe try to connect to QEMU using GDB- let me know if you do that and it works for you.

'''''more to come...'''''

== Resources ==
=== U-Boot ===
* [https://github.com/u-boot/u-boot U-Boot GitHub repo]
* [https://pandysong.github.io/blog/post/run_u-boot_in_qemu/ Blog article about U-Boot under QEMU]
* [https://u-boot.readthedocs.io/en/latest/build/gcc.html U-Boot docs]
=== RISC-V ===
* [https://github.com/riscv-software-src/opensbi OpenSBI GitHub repo]
* [https://riscv.org/wp-content/uploads/2019/06/13.30-RISCV_OpenSBI_Deep_Dive_v5.pdf Info about RISC-V boot process]
````

<a id="w004704"></a>

## W004704 — File:BrutalDemo.png

Bron-ID: W004704
Namespace: File (6)
Revisie: 27033 | Datum: 2022-04-11T13:34:51Z
Bron: https://wiki.osdev.org/File:BrutalDemo.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABrutalDemo.png&oldid=27033

````text
A brutal screenshot showing the system running on QEMU with its own GUI.
````

<a id="w004705"></a>

## W004705 — X87

Bron-ID: W004705
Namespace: Main (0)
Revisie: 27036 | Datum: 2022-04-11T15:49:10Z
Bron: https://wiki.osdev.org/X87
Revisie-URL: https://wiki.osdev.org/index.php?title=X87&oldid=27036
Doorverwijzing: FPU

````text
#redirect [[FPU]]
````

<a id="w004706"></a>

## W004706 — 3270

Bron-ID: W004706
Namespace: Main (0)
Revisie: 27040 | Datum: 2022-04-19T01:17:00Z
Bron: https://wiki.osdev.org/3270
Revisie-URL: https://wiki.osdev.org/index.php?title=3270&oldid=27040

````text
{{stub}}

The '''3270''' is a family of terminals compatible and often used with the S/390 and z/Architecture mainframes. They are relatively simple to code for, and act as a "streaming" terminal (that is, it receives commands and applies them to an state) - so it virtually works the same as a VT100.

== Usage ==
The 3270 operates like a normal terminal - it has PF0-PF15 keys which many programs use to perform operations (especially interactive programs). Receiving data is done through a normal I/O interruption generated by the 3270.

A local non-SNA console known as the SYSG is present on modern day mainframes, to use said console the traditional 3270 commands are mostly deprecated - and instead relies exclusively on the SCLP - this local non-SNA version can't use traditional CCW command chains or CSS addressing.

== Sending commands ==

3270 supports various CCW commands - all of them can be used, most important ones are CMD_WRITE_CR, CMD_WRITE_NOCR and CMD_READ_BUFFER. See [[Command SubSystem]] for more information about CCW commands.

<syntaxhighlight lang="c">
#define X3270_CMD_SELECT 0x0B
#define X3270_CMD_SELECT_WRITE 0x4B
#define X3270_CMD_WRITE_NOCR (CSS_CMD_WRITE) // Write without carriage return
#define X3270_CMD_WRITE_CR 0x09 // Write with carriage return
#define X3270_CMD_READ_BUFFER (CSS_CMD_READ) // Read the buffer
#define X3270_CMD_READ_MODIFIED 0x06 // Read modified
#define X3270_CMD_NOP (CSS_CMD_CONTROL) // No operation
#define X3270_CMD_WSF 0x11 // Write Structured Field
</syntaxhighlight>

== Composing packets ==

Composing packets consists of creating a buffer with the desired 3270 commands (not to be confused with CCW commands). And sending it via a READ CCW.

=== Traditional HLASM ===
<syntaxhighlight lang="cobol">
* USE =A(BUFFER) SINCE WE'RE NOT DECLARING BUFFER AS
* AN EXTERNAL SYMBOL
BUFFER
         $SF   (SKIP)
* TELL THE 3270 WHERE OUR TEXT SHOULD BE PLACED AT
* MOST 3270 TERMINALS CAN USE UP TO 24 ROWS AND 80 COLUMNS
* OTHER MODELS MAY HAVE MORE
         $SBA  (24,5)
* THE HELLO WORLD MESSAGE
         DC    C'HELLO 3270 WORLD!'
* TELL THE 3270 WE'RE ENDING THIS
         $SF   (SKIP)
</syntaxhighlight>

Then an HLASM routine would send the CCW command to take =A(BUFFER) and the 3270 will load said buffer and start interpreting it.

=== C ===

On C there are no HLASM macros to assist in the creation of a buffer, instead it has to be created manually.

<syntaxhighlight lang="c">
enum x3270_order {
    X3270_ORDER_PROGRAM_TAB = 0x05,
    X3270_ORDER_SET_BUFFER_ADDR = 0x11,
    X3270_ORDER_INSERT_CURSOR = 0x13,
    X3270_ORDER_ERASE_UNPROTECTED = 0x12,
    X3270_ORDER_START_FIELD = 0x1D,
    X3270_ORDER_START_FIELD_EXTENDED = 0x29,
    X3270_ORDER_MODIFY_FIELD = 0x2C,
    X3270_ORDER_SET_ATTRIBUTE = 0x28
};
</syntaxhighlight>

=== Addressing ===

The 3270 has a legacy addressing scheme, relying exclusively on EBCDIC characters to represent addresses below a 4096 characters barrier (12-bits of addresses). This is not present on the local non-SNA version.

<syntaxhighlight lang="c">
const unsigned char ebcdic_map[] = {
    0x40, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0x4A, 0x4B,
    0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7,
    0xD8, 0xD9, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F, 0x60, 0x61, 0xE2, 0xE3,
    0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
    0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0x7A, 0x7B,
    0x7C, 0x7D, 0x7E, 0x7F
};

uint16_t x3270_address(uint16_t addr)
{
    uint8_t tmp[2];
    // 12-bit conversion does not need to be done since it's higher than the imposed 4K limit
    if(addr >= 0xfff) {
        tmp[0] = (addr >> 8) & 0x3F;
        tmp[1] = addr & 0x3F;
    } else {
        // Take only in account low 6-bits */
        tmp[0] = ebcdic_map[(addr >> 6) & 0x3F];
        tmp[1] = ebcdic_map[addr & 0x3F];
    }
    return *((uint16_t *)&tmp[0]);
}
</syntaxhighlight>

=== Example write ===
Writing to the 3270 can be, generally broken down into several steps:

* An initial byte known as the Write Control Character (WCC)
* The SBA command to set the address the text is going to be written at
* The text itself, encoded in EBCDIC
* Another final SF to terminate (SKIP)

Some steps are optional, but are provided for completeness.

<syntaxhighlight lang="c">
// the schid depends upon the system
#define X3270_SCHID 0x00010001

void x3270_compose_buffer(uint8_t *buf, size_t n)
{
    const char *msg = "Hello world";
    size_t offset = 0, addr;

    // starting wcc
    buf[offset++] = 0x00;

    // sba will now set the address to 0,0  (top-left of the screen)
    buf[offset++] = X3270_ORDER_SET_BUFFER_ADDR;
    addr = x3270_address(0);
    buf[offset++] = (addr >> 8) & 0xff;
    buf[offset++] = addr & 0xff;

    // the data that is to be written
    memcpy(&buf[offset], msg, strlen(msg));
    offset += strlen(msg);

    // optional terminating sf
    buf[offset++] = X3270_ORDER_START_FIELD;
    buf[offset++] = 0x00;

    // send the buffer to the 3270 via a ccw...
    kernel_css_write(X3270_SCHID, buf, n);
}
</syntaxhighlight>

== See also ==
=== External sources ===
* [https://www.prycroft6.com.au/misc/3270.html 3270 Programming tutorial]
````

<a id="w004707"></a>

## W004707 — User:Superleaf1995/Compile GCCMVS under MVS3.8j

Bron-ID: W004707
Namespace: User (2)
Revisie: 27087 | Datum: 2022-05-13T02:47:28Z
Bron: https://wiki.osdev.org/User:Superleaf1995/Compile_GCCMVS_under_MVS3.8j
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASuperleaf1995%2FCompile_GCCMVS_under_MVS3.8j&oldid=27087
Doorverwijzing: User:Wxwisiasdf/Compile GCCMVS under MVS3.8j

````text
#REDIRECT [[User:Wxwisiasdf/Compile GCCMVS under MVS3.8j]]
````

<a id="w004708"></a>

## W004708 — User talk:Leapofazzam

Bron-ID: W004708
Namespace: User talk (3)
Revisie: 27162 | Datum: 2022-06-08T09:34:42Z
Bron: https://wiki.osdev.org/User_talk:Leapofazzam
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ALeapofazzam&oldid=27162

````text

== Are you Stivationist0Italian? ==

Question --[[User:Wxwisiasdf|Wxwisiasdf]] 12:39, 4 June 2022 (CDT)

No [[User:Leapofazzam|Leapofazzam]] 04:34, 8 June 2022 (CDT)
````

<a id="w004709"></a>

## W004709 — DisplayPort

Bron-ID: W004709
Namespace: Main (0)
Revisie: 28118 | Datum: 2023-07-10T02:06:27Z
Bron: https://wiki.osdev.org/DisplayPort
Revisie-URL: https://wiki.osdev.org/index.php?title=DisplayPort&oldid=28118

````text
{{In Progress}}

DisplayPort is a protocol (link) and bus (physical) standard for transporting video, audio and other data from a "source" device to a "sink" device.

== Link Layer ==

The link layer consists of a Main Link, which is used for transporting video and audio, and an Auxiliary Channel (AUX CH) which is used for configuration & retrieving information such as the EDID.

=== Auxiliary Channel ===

The AUX CH is a packet-based protocol that can transport both [[I²C]] and Native AUX CH packets. The source initiates a request and the sink always sends a response within a timeout. A packet is at most 16 (20?) bytes large.

Each packet starts with a single byte header. Bit 7 indicates whether the packet is an [[I²C]] packet or Native AUX CH packet.

==== Native AUX CH ====

===== Request header format =====

{| class="wikitable"
|-
! Bit(s)
! Description
|-
| 7
| 0 (Native AUX CH transaction)
|-
| 6:4
| Request type (000 = write, 001 = read)
|-
| 3:0
| Padding (must be all zeroes)
|}

===== Reply header format =====

{| class="wikitable"
|-
! Bit(s)
! Description
|-
| 7:6
| Must be zero / ignored
|-
| 5:4
| Native AUX CH reply (00 = ACK, 01 = NACK, 10 = defer)
|-
| 3:0
| Padding (must be all zeroes)
|}

==== I²C ====

A I²C transaction is initiated by sending an initial packet with the address. After receiving an ACK data can be read or written. The source must always specify the address and the amount of data to be sent or read by adding a byte that denotes the length minus 1.

The last packet must have MOT = 0. All other packets must have MOT = 1.

For example, an I²C transaction to retrieve the EDID may go as follows (> denotes source to sink, < denotes sink to source):

<pre>
> 0b0_1_00_0000, 0, 0x50 (I2C, MOT = 1, write, address 0x50)
< 0b00_00_0000 (AUX_ACK, I2C_ACK)
> 0b0_1_00_0000, 0, 0x50, 0, 0 (I2C, MOT = 1, write, address 0x50, length 1, data 0)
< 0b00_00_0000 (AUX_ACK, I2C_ACK)
> 0b0_1_01_0000, 0, 0x50 (I2C, MOT = 1, read, address 0x50)
< 0b00_00_0000 (AUX_ACK, I2C_ACK)
loop 128 times:
> 0b0_1_01_0000, 0, 0x50, 0 (I2C, MOT = 1, read, address 0x50, length 1)
< 0b00_00_0000, x (AUX_ACK, I2C_ACK, data x)
repeat loop
> 0b0_0_01_0000, 0, 0x50 (I2C, MOT = 0, read, address 0x50)
< 0b00_00_0000 (AUX_ACK, I2C_ACK)
</pre>

It is possible to read/write multiple bytes of data in one request which is faster, though this depends on the I²C bus speed. E.g. a 100KHz device can send 1 bit every 10µs, which allows it to send up to 3 bytes per reply (300µs / 10µs/bit = 30 bits > 3 bytes).

A DEFER reply may be sent if the sink fails to deliver a response within 300µs. This may happen with e.g. slow I²C devices. When receiving a DEFER reply a STATUS_REQUEST should be sent at a later point.

===== Request header format =====

{| class="wikitable"
|-
! Bit(s)
! Description
|-
| 7
| 1 (I²C transaction)
|-
| 6
| Middle-of-transaction bit.
|-
| 5:4
| Request type (00 = write, 01 = read, 10 = write status_request)
|-
| 3:0
| Padding (must be all zeroes)
|}


===== Reply header format =====

{| class="wikitable"
|-
! Bit(s)
! Description
|-
| 7:6
| I²C-over-AUX reply (00 = ACK, 01 = NACK, 10 = defer)
|-
| 5:4
| Native AUX CH reply (00 = ACK, 01 = NACK, 10 = defer)
|-
| 3:0
| Padding (must be all zeroes)
|}

== See also ==

* [http://file.yizimg.com/383992/2014090921252964.pdf DisplayPort v1.1a documentation]
* [https://en.wikipedia.org/wiki/DisplayPort Wikipedia article on DisplayPort]

[[Category:Video]]
[[Category:Hardware Interfaces]]
````

<a id="w004710"></a>

## W004710 — User:Demindiro/DisplayPort

Bron-ID: W004710
Namespace: User (2)
Revisie: 27181 | Datum: 2022-06-17T18:11:41Z
Bron: https://wiki.osdev.org/User:Demindiro/DisplayPort
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADemindiro%2FDisplayPort&oldid=27181
Doorverwijzing: DisplayPort

````text
#REDIRECT [[DisplayPort]]
````

<a id="w004711"></a>

## W004711 — User:Demindiro/Porting Rust standard library

Bron-ID: W004711
Namespace: User (2)
Revisie: 27185 | Datum: 2022-06-17T18:29:39Z
Bron: https://wiki.osdev.org/User:Demindiro/Porting_Rust_standard_library
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADemindiro%2FPorting_Rust_standard_library&oldid=27185
Doorverwijzing: Porting Rust standard library

````text
#REDIRECT [[Porting Rust standard library]]
````

<a id="w004712"></a>

## W004712 — Category:Raspberry Pi

Bron-ID: W004712
Namespace: Category (14)
Revisie: 27200 | Datum: 2022-06-21T07:43:08Z
Bron: https://wiki.osdev.org/Category:Raspberry_Pi
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ARaspberry_Pi&oldid=27200

````text
[[Category:ARM]]
````

<a id="w004713"></a>

## W004713 — Talk:Game port

Bron-ID: W004713
Namespace: Talk (1)
Revisie: 27212 | Datum: 2022-07-07T14:25:10Z
Bron: https://wiki.osdev.org/Talk:Game_port
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGame_port&oldid=27212

````text
== Disabling Interrupts ==

asm volatile ("cli") may be reordered by the compiler as part of optimization. It should be changed to
 asm volatile ("cli":::"memory")
[[User:Tree5678|Tree5678]] 09:25, 7 July 2022 (CDT)
````

<a id="w004714"></a>

## W004714 — File:Lotus-Effect v2017.png

Bron-ID: W004714
Namespace: File (6)
Revisie: 27242 | Datum: 2022-07-19T08:05:19Z
Bron: https://wiki.osdev.org/File:Lotus-Effect_v2017.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALotus-Effect_v2017.png&oldid=27242

````text

````

<a id="w004715"></a>

## W004715 — File:Cyjon v1440.png

Bron-ID: W004715
Namespace: File (6)
Revisie: 27244 | Datum: 2022-07-19T08:08:39Z
Bron: https://wiki.osdev.org/File:Cyjon_v1440.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACyjon_v1440.png&oldid=27244

````text

````

<a id="w004716"></a>

## W004716 — User:Abletheabove

Bron-ID: W004716
Namespace: User (2)
Revisie: 27262 | Datum: 2022-08-01T09:13:52Z
Bron: https://wiki.osdev.org/User:Abletheabove
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAbletheabove&oldid=27262

````text
abletheabove, shortened to able, is the lead dev of [http://git.ablecorp.us ableOS]
````

<a id="w004717"></a>

## W004717 — PinePhone

Bron-ID: W004717
Namespace: Main (0)
Revisie: 30203 | Datum: 2026-03-12T15:36:14Z
Bron: https://wiki.osdev.org/PinePhone
Revisie-URL: https://wiki.osdev.org/index.php?title=PinePhone&oldid=30203

````text
{{In Progress}}

The [https://wiki.pine64.org/index.php?title=PinePhone PinePhone] is an open hardware smartphone from Pine64. It features all the stuff you'd expect from a modern smartphone, although don't expect the latest and greatest variants of each. Full schematics are available [https://wiki.pine64.org/index.php?title=PinePhone#PinePhone_board_information,_schematics_and_certifications on the Pine64 wiki]. Unfortunately, open hardware does not mean that full datasheets for all chips are available.

Beware the slight but incompatible changes between different hardware revisions - such as different GPIO pins being used. This page is written about the Community Edition with the v1.2b mainboard, the latest as of writing.

The [[PinePhone Pro]] is a different device with similar but incompatible hardware! (such as an entirely different CPU chip from Rockchip)

== Resources ==

* https://linux-sunxi.org/
* https://linux-sunxi.org/File:Allwinner_A64_User_Manual_V1.1.pdf - includes memory map; full register documentation for several basic peripherals including GPIOs, DMA, timers...
* https://wiki.pine64.org/index.php?title=PinePhone#PinePhone_board_information,_schematics_and_certifications

The CPU is an [https://linux-sunxi.org/A64 Allwinner A64] (a.k.a. "sun50i"), which is also used in some of Pine64's single-board mcomputers. Although Allwinner does not release full documentation, significant reverse-engineered information is available through the [https://linux-sunxi.org/ linux-sunxi project]. This is presumably one reason Pine64 chose this CPU.

== Requirements ==

To write an SD/eMMC image you need the mksunxiboot tool from the u-boot-tools package (may vary by distribution). [TODO: or you could just write the header yourself]

To use FEL you also need the sunxi-fel tool from the sunxi-tools package.

Since the OS controls all signs of whether the phone is powered on, it's probably a good idea to remove the battery, so that you can be sure whether the phone is on or off. Also, some aspects of battery charging are controlled by the OS, so this removes any chance of accidentally damaging it by applying incorrect settings (not sure if this is possible/likely). The cellular modem is powered directly from the battery and won't work if the battery is not inserted - but this won't be a problem until your OS is quite well developed. The CPU seems to work just fine while the battery is removed.

== Booting ==

The CPU first boots from an integrated boot ROM, which then tries to boot from one of these sources:[https://linux-sunxi.org/Boot_Process]

# The microSD card, if present and boot signature (eGON header) is found
# The internal eMMC storage, if boot signature (eGON header) is found
# The FEL mode ROM - a USB slave mode where the host computer can read, write and execute memory.

There is also an FEL button that can theoretically be pressed to force boot in FEL mode, but on the PinePhone it is only connected to a test pad, not to an actual button.

The normal way to prepare an OS for the PinePhone is to flash a microSD card and insert it. The OS may then offer an option to install itself onto the eMMC. However, when getting started with osdev, FEL mode (which allows loading code via USB) should allow for a much faster testing cycle.

For your OS, you can choose to boot via U-boot [TODO] or directly into your own "bootloader".

=== Loading code via microSD/eMMC ===

Section TODO

Boot signature / eGON header starts at address 8192 and occupies (theoretically) up to 32KB. This region is often left unallocated; if you're low on microSD cards, you might be able to share one with a Raspberry Pi project for example, without constantly re-flashing it.

=== Loading code via FEL ===

==== Accessing FEL ====

FEL is executed if:

* You have no valid bootloader installed (eGON header not found) on microSD card nor eMMC
* You press the FEL key (not applicable to Pinephone)
* You call the FEL boot code directly

Third option is possible by writing a tiny stub to a microSD card. This is available in sunxi-tools by the name of fel-sdboot. Run <tt>make fel-sdboot.sunxi</tt>. It makes an 8kB file but only the first 124 bytes are used. Write the file to offset 8192 on any microSD card:

    sudo dd if=fel-sdboot.sunxi of=/dev/NAME_OF_SD_CARD_DEVICE bs=8192 seek=1 count=1

and you can insert this microSD card to trigger FEL mode.

NOTE: When in FEL mode, the phone provides ''absolutely no visible sign'' that it is turned on. You can check by running <tt>lsusb</tt> on the host computer - you should see a USB device like this one:

    Bus 001 Device 012: ID 1f3a:efe8 Allwinner Technology sunxi SoC OTG connector in FEL/flashing mode

and you can verify it by running <tt>sunxi-fel version</tt>:
    sunxi-tools$ ./sunxi-fel version
    AWUSBFEX soc=00001689(A64) 00000001 ver=0001 44 08 scratchpad=00007e00 00000000 00000000

==== Loading code example ====

    sunxi-fel write 0x00010000 code.bin
    sunxi-fel execute 0x00010000

Your code can execute BX LR to return back to the boot loader.

To see if your toolchain works you can try to run [[User:Immibis/PinePhone Blinky]].

Why 0x00010000? According to the [https://linux-sunxi.org/A64/Memory_map memory map] this region 0x00010000-0x00017FFF is where the boot ROM normally loads user code, <s>which means it shouldn't conflict with anything used by the boot ROM (e.g. to support the USB interface).</s> region 0x00010XXX is fine, 0x00011XXX conflicts and breaks USB. Comments in the sunxi-fel tool indicate regions 0x11c00-0x11fff (IRQ stack), 0x15c00-0x16fff (actual stack), and 0x17c00-0x17fff ("something important", probably USB-related data structures or similar) are important. Additionally, the tool uses 0x31400-0x32fff to hold backup copies of these.
It can also use 0x11000-0x113FF to hold code and data for more complex commands.

Maybe 0x18000 (SRAM C) is a good load location. 0x19400 bytes (~48k) should be available.
If using SRAM C, don't set 0x01C202C4 (Bus Clock Register1) bit 0x00000001 (Gating Clock VE) to 0 as this seems to make SRAM C inaccessible. I guess it's connected to the video engine?

Alternatively you could initialize DRAM and then load your code into DRAM, as two separate steps.

== Custom bootloader stuff ==

Skip this section if you are running your OS from U-Boot.

You might notice accessing the 2GB/3GB (depending on model) DRAM area at address 0x40000000 upwards causes the processor to hang, so you only have the SRAM areas (a few hundred kB) to play with. That's because the boot ROM doesn't initialize the DRAM controller. It only uses ROM and SRAM. On a PC, the BIOS would have initialized the memory for you, but this is an embedded system, not a PC. The bootloader is the BIOS, and you are the bootloader.

See [[PinePhone/DRAM initialization]]

== Power distribution ==

As this is a battery-powered device, power is a big issue. The system has no less than ''eighteen'' power supplies which can be turned on and off individually and have adjustable voltages. These are all controlled by the AXP803 power management chip, which also manages the battery detection and charging. There are a few power rails not controlled by the AXP803, such as the USB power output (not input).

Turning off chips you're not using will make the battery last longer! But at the beginning you probably need to turn things ''on''. The display screen is not powered by default.

[https://raw.githubusercontent.com/OLIMEX/OLINUXINO/master/DOCUMENTS/A64-PDFs/AXP803_Datasheet_V1.0.pdf AXP830 datasheet]

The AXP803 can be accessed by the S-TWI interface located on GPIOs PL0 and PL1. The TWI interface block hasn't been explored yet, but you can write bit-banging code using the GPIO. For details how to access those GPIOs see the. Note that I2C is an open-drain protocol, so you should switch the pins between input mode and output-0 mode. Setting them to output 1 has a small chance of damaging either the CPU or the AXP due to a logic level conflict, so don't do it. Because it's open-drain you should also enable pull-up resistors on the CPU side. (Actually it's not clear whether the AXP already has built-in pullups. Having two sets doesn't hurt, but it might waste a little bit of power)

'''Before accessing PL0/PL1''' you must enable bit 0 (0x00000001) in register 0x01F01428. Port PL is on a separate section of the CPU from all the other GPIO ports. This bit enables the clock to this section. Otherwise the section ignores register writes.

The AXP803's I2C address is 0x68 (write) / 0x69 (read) / 0x34 (7-bit address format).

You can write a register by writing the register number and then the value (that's start signal, send 0x68, send register number, send value, stop signal). You can read a register by writing the register number and then reading the value (start signal, send 0x68, send register number, repeated start signal, send 0x69, read value, stop signal). Remember that when changing I2C transfer direction, you should send a repeated start condition and send the device address again (with the new direction). After reading one register, send ACK if you want to read the next register or NACK if you want to finish. If you send ACK and then stop, the chip's I2C interface will be left in a weird state and you'll have to . If you keep reading bytes after the first one, you read the next register, and the next. 

Registers 0x00-0x03 are useful to see if your reading code works correctly; registers 0x04-0x0F are scratch registers with no purpose which you can write and then read to test the writing.

== List of built-in peripherals ==

=== eMMC ===

* CPU's SDC2 port is connected to the eMMC chip.

=== SD card ===

* CPU's SDC0 port (4 bits wide) connected to the SD card.
* GPIO PF6 detects whether a card is inserted.

=== Power management unit (PMU) ===

Chip type: AXP803

* CPU's PL0/PL1 connected to this chip's I2C port for configuration and status
* Resets the CPU when power is not okay
* Triggers NMI on CPU
* No idea why it connects to USB0 data pins

=== WLAN/BT ===

Powered by VCC-PG, VCC-WiFi, DCDC1. 

RTL8723CS chip. 4-bit-wide SD interface connects to CPU's SDC1 interface (Port PG).

* CHIP_EN is controlled by a hardware switch under the back cover.
* CPU's SDC1 interface (4 bits wide) is connected to the SD interface on this chip
* UART (not BT_UART1) connects to CPU UART1. Only TX/RX/CTS. It seems RTS was accidentally not connected on the schematic, despite being labeled?
* BT_UART1 is not connected.
* CPU's PCM1 interface is connected to this chip.
* Additional GPIO connections - labeled as they appear on the schematic: BT-RST-N, BT-WAKE-AP, WL-PMU-EN, AP-WAKE-BT


=== Pogo pin expansion port ===
General purpose low-speed interface exposed via 6 pogo pins (springy pins) inside the back cover.

Power pins: DCIN directly connects to the USB port's power rail. USB-5V is the voltage the phone could output to the USB port, but doesn't have to actually output as there is a switch in the way. Software can turn on a boost converter to get 5V here, otherwise you get slightly lower than the main system voltage (3 to 5V).

Data pins: SDA/SCK/INT connect to GPIOs PE15, PE14 and PL12 respectively. The I2C pins (SDA/SCK) do connect to a hardware I2C block which is dedicated for this port.

=== Audio ===
Audio interfaces built into the CPU.

* PHONEIN, LINEIN, PHONEOUT: Unused - connected to nothing - don't even bother.
* EAROUT: is the "receiver" (top speaker when using as a phone?) Direct connection, no other power supply needed.
* LINEOUT: is the main speaker, amplified via U800. You have to tell the amplifier to actually work by outputting high(?) from GPIO pin PC7. Turn it off when unused, to avoid wasting power.
* HPOUT: to the headphone jack. There's a DETect pin which tells you if something is plugged into the port. When the back cover switch is set to disable headphones, it triggers an analog switch to connect the UART instead of the headphones, and it also overrides the detect signal so it detects nothing plugged in. It doesn't affect the microphone connection, and allegedly the analog switch might not fully isolate the sound signal from the UART, so you probably don't want to play sound.
* MIC2: is the headphone jack microphone port. For headphones without microphone it's just shorted to ground. Microphone power provided by HBIAS from CPU, through a high resistance so there is no problem even if microphone is shorted to ground.
* MIC1: is the microphone below the screen. Microphone power provided by MBIAS from CPU; interrupted by one of the switches under the back cover. Hopefully the microphone doesn't work without this power.

Headphone jack UART: It's UART0 on the CPU, PB8=TX, PB9=RX

=== Camera ===

[not researched, no idea how to make it work]

CSI occupies pins PE0 (PCLK), PE1 (MCLK), PE2 (HSYNC), PE3 (VSYNC), PE4-PE11 (D0-D7), PE12 (SCK), PE13 (SDA).

Front and rear camera both use the same CSI port, at the same time. Activating both cameras presumably causes corrupted data (but won't damage hardware due to the use of resistors for current limiting). Only use one at a time.

Each camera has its own RST and STBY pins which are pulled up (TODO: that means they're off, right?) and connected as follows:

{|
! &nbsp; !! Front camera !! Rear camera
|-
! RST || PE16 || PD3
|-
! STBY || PE17 || PC0
|}

==== Camera flash ====

U1000 controls the flash LED from the battery (therefore probably requires an inserted battery)

GPIO PD24 = PD24-FLASH-TRIGOUT = EN pin<br>
GPIO PC3 = PC3-FLASH-EN = FLASH pin<br>
(names are swapped, yes. They can be either way around according to the schematic. TODO: which way is it really?)

=== Display ===

==== Backlight ====

Theory: GPIO PH10 enables the boost converter U1100 which creates a high voltage to power the backlight (note it's configured as constant current rather than constant voltage).<br>
Pin configuration PH10: write 0x0000X1XX to address 0x01C20900 (upper 16 bits are unused)<br>
Pin control PH10: bit 0x00000400 at address 0x01C2090C - set it true to enable<br>

GPIO PL10 is a PWM signal that gets smoothed out and then alters the feedback signal received by U1100. High feedback signal causes U1100 to believe the output current is higher than it really is, so it decreases it. This is a normal way for these converters to work, and does not cause any problem. However, the author wasn't able to make this work yet, and skipped over it because the backlight did turn on even without touching PL10.<br>
Pin configuration PL10: write 0x000XX1XX (GPIO) or 0x000XX2XX (PWM) to address 0x01F02C04 (upper 12 bits are unused)<br>
Pin control PL10 (GPIO mode): bit 0x00000400 at address 0x01F02C10 - set it false (default value) for full brightness backlight. In reality, writing to this pin didn't do anything.
Actually, the author did not get PL10 to do anything. The backlight was enabled even without touching PL10. When configured as an input, it read as low. Possibly the power rail ALDO2 has to be enabled first?<br>

According to U1100's datasheet, you could also try adjusting the brightness by PWMing PH10, or both at once, however PH10 has no PWM hardware attached, only MIC_CLK (which maybe you could hack to get a 50% PWM). It's also not clear whether the output has enough filtering to make this not flicker. The schematic allows for the possibility to swap PL10 and PH10 (so that EN pin has the PWM hardware) by re-soldering a jumper.

==== Touch panel ====

[not researched, no idea how to make it work]<br>
Also known as CTP (capacitive touch panel). Connected by TWI0 bus and 2x GPIO (interrupt + reset)

==== Video output ====

[not researched, no idea how to make it work]<br>
MIPI-DSI connected to the CPU. Two additional GPIOs: PD7 connects to DSI-TE (?) and PD23 connects to LCD-RESET#

=== Small items ===

==== Front LED and rumble motor ====

Simple on/off switches (GPIOs) controlled by software.

To configure the LED pins for generic output, write 0xXXX111XX to address 0x01C20874. The X are bits that are irrelevant to LEDs. Default value for each X is 7, so write 0x77711177 if you're only using the LEDs and nothing else on this port. To configure the motor pin, write 0xXXXXX1XX to address 0x01C2086C. Then four of the bits in address 0x01C2087C control these features - see below.

Note the R/G/B channel order is mislabeled in the schematic (and in two different ways!) - the order below was actually tested.

{| border
! &nbsp; !! Control bit !! GPIO number
|-
| Red || 0x00080000 || PD19
|-
| Green || 0x00040000 || PD18
|-
| Blue || 0x00100000 || PD20
|-
| Rumble motor || 0x00000004 || PD2
|}

==== Sensors ====

TWI1-SDA/TWI1-SCK is connected to a few sensor chips you'd expect on phones. Some alternative parts may be used depending on what was available when your phone shipped.

All these sensors communicate over I2C bus connected to TWI1-SCK (PH2) and TWI1-SDA (PH3) and are powered by DLDO1; STK3311-A also powered by GPIO0-LDO (as well as DLDO1).

Proximity and ambient light sensor: STK3311-A (U1201) <br>
INT pin connects to GPIO PB0

6-axis (i.e. 3-axis + 3-axis) gyroscope and accelerometer: MPU6050 (U1202) or BMI120 (U1204)<br>
INT pin connects to GPIO PH5

3-axis magnetometer: LIS3MDL (U1200) or AK09911/AF8133J (U1203): ?<br>
DRDY pin (LIS3MDL) or RSTN (AK*) connects to GPIO PB1.

==== Side buttons ====

KEYADC connects to VOL+ and VOL-, forming a resistor divider with a different value depending on which one is pressed. VOL+ has priority. If VOL+ is pressed it's not possible to detect whether VOL- is also pressed. (A shame. It would've cost nothing to give them separate pins, since there are still spare pins available!)

The reset button resets the CPU by pulling down AP-RESET#. Note that it doesn't reset other chips.

The power key goes to a signal called PWRON on the AXP803.

=== 4G cellular modem ===

[Not really researched, no idea how to make it work]

Powered by VDD_EXT (what is that?) and VBAT and DCDC1

VBAT is only connected to the modem when GPIO PL7 is high and the modem is enabled using the switch under the back cover.

* Connected to SIM card slot (nothing else can access the SIM card)
* CPU's PCM0 connects to modem's PCM port (digital audio data, bidirectional)
* CPU's UART3 connects to modem's UART port - well, data lines are UART3 on the CPU but control lines are all over the place and not on UART3.
* CPU's USB1 connects to modem's USB port.
* Other GPIOs: AP_READY, W_DISABLE#, RESET_N, PWRKEY
* A whole lot of unconnected interfaces, including EPHY, SGMII, dual SIM, SDC1, SDC2, WLAN_EN, BT_EN, DBG_


=== USB connector ===

[Not really researched, no idea how to make it work]

The ANX7888 is a big complicated-looking chip with many ports and its own firmware. It appears to control auxiliary functions of the USB port including the DisplayPort Alternate Mode, and possibly cable detection, orientation, and . Note that the main USB D+/D- lines bypass the chip and go directly to the CPU. Likewise the USB power input goes directly to the AXP803 which automatically uses it to power the system. The phone probably will not ''output'' power by default.

* HDMI input from CPU. Also DDC I2C and HHPD pins.
* Control via CSCL/CSDA pins via TWI0-SCK (GPIO PH0) / TWI0-SDA (GPIO PH1) bus
* USB to the USB connector: SSTX, SSRX, AUX and CC wires.
* Other GPIOs: POWER_EN, RESET_N, ALERT_N, CABLE_DET

Special power signals for USB:

* GPIO PD11 tells U1300 to generate some of the power rails for the ANX.
* GPIO PD9 tells U1308 to create 5V for the USB CC1/CC2 signalling pins (the ANX chip controls the pin state).<br>
* GPIO PD8 tells U601 to create 5V for the USB power output. This 5V (USB-5V) is only actually connected to the USB port based on DRVVBUS signal, which is connected to AXP803's N_VBUSEN signal, and the ANX7888's VBUS_CTRL signal (but NOT the CPU pin labelled DRVVBUS). TODO: so when is it active?

[[Category:PinePhone]]
[[Category:ARM]]
````

<a id="w004718"></a>

## W004718 — PinePhone/DRAM initialization

Bron-ID: W004718
Namespace: Main (0)
Revisie: 30202 | Datum: 2026-03-12T15:34:31Z
Bron: https://wiki.osdev.org/PinePhone/DRAM_initialization
Revisie-URL: https://wiki.osdev.org/index.php?title=PinePhone%2FDRAM_initialization&oldid=30202

````text
There are two things we have to do here: (1) enable the DRAM controller module's clock and reset; (2) configure the DRAM controller.

'''License warning''': the sequence of operations needed was derived from [[U-Boot]], but is written here in the author's own code. In the author's opinion, since only the facts and not the code are derived, U-Boot's license (GPL) doesn't carry over. Note there is '''no way''' to know what registers to write, other than copying what Allwinner ultimately provided, detailed hardware reverse-engineering, or an extremely lucky guess.

'''Quality warning''': the code here got DRAM to work for the author, but actual correctness is not guaranteed. Especially since the entire DRAM controller is barely documented (in the comments of U-Boot and Linux drivers), we might be close to some threshold of unreliability without knowing.

== Power up DRAM controller ==

This part is documented in the A64 User Manual so it's pretty safe 

<syntaxhighlight lang="c">
#define U32_REG(addr) (*(volatile uint32_t*)(addr))

#define PLL_DDR0_CTRL_REG U32_REG(0x01C20020)
#define PLL_DDR0_ENABLE_FLAG 0x80000000

#define PLL_DDR1_CTRL_REG U32_REG(0x01C2004C)
#define PLL_DDR1_ENABLE_FLAG 0x80000000
#define PLL_DDR1_UPDATE_FLAG 0x40000000
#define PLL_DDR1_FACTOR_N(n) ((((n) - 1) << 8) & 0x3F00)

#define MBUS_RST_REG U32_REG(0x01C200FC)
#define MBUS_RST_RELEASE 0x80000000 // zero value means reset, this flag means not reset

#define MBUS_CLK_REG U32_REG(0x01C2015C)
#define MBUS_CLK_CLOCK_ENABLE 0x80000000

#define BUS_CLK_GATING_REG0 U32_REG(0x01C20060)
#define BUS_CLK_RESET_RELEASE_REG0 U32_REG(0x01C202C0) // same bitfields
#define BUS_CLK_REG0_DRAM_GATING 0x00004000

#define DRAM_CFG_REG U32_REG(0x01C200F4)
#define DRAM_CFG_RESET_RELEASE 0x80000000
#define DRAM_CFG_SRC_PLL_DDR1 0x00100000
#define DRAM_CFG_UPDATE_FLAG 0x00010000

static void dsb() {
	// wait for all outstanding memory accesses to complete before continuing.
	// Probably not actually the right thing to do in most cases!
	__asm__ __volatile__("dsb" ::: "memory");
}
static void delay() {
	for(volatile int i = 0; i < 0x1000; i++) {}
}

static void delay() {
	int n = 0x1000;
	while(n--) {
		__asm__ __volatile__("" : "=r"(n) : "0"(n) : "memory");
	}
}

void dram_clock_init()
{
	// A64 User Manual 3.3.6.4. says we should always release reset before releasing clock gate. So logically we should do the reverse order when disabling a clock.
	MBUS_CLK_REG &= ~MBUS_CLK_CLOCK_ENABLE; // disable MBUS clock
	BUS_CLK_GATING_REG0 &= ~BUS_CLK_REG0_DRAM_GATING; // disable DRAM clock
	PLL_DDR0_CTRL_REG &= ~PLL_DDR0_ENABLE_FLAG; // disable DRAM clock (maybe)
	PLL_DDR1_CTRL_REG &= ~PLL_DDR1_ENABLE_FLAG; // disable DRAM clock (maybe)
	MBUS_RST_REG &= ~MBUS_RST_RELEASE; // assert MBUS reset
	BUS_CLK_RESET_RELEASE_REG0 &= ~BUS_CLK_REG0_DRAM_GATING; // assert DRAM reset
	// shouldn't we disable the DRAM controller clock?
	DRAM_CFG_REG &= ~DRAM_CFG_RESET_RELEASE; // assert DRAM controller reset
	dsb();

	// N factor calculation:
	// 553MHz DDR clock; *2 because the DRAM controller internal clock apparently runs at DDR (not surprising I guess)
	// divided by the 24MHz base clock which is apparently the input to the PLL
	// gives 553*2/24 = 46.083, rounded to 46 (which gives 552MHz)
	// note the value in register is this -1 (see the macro) so it's 45 to give an actual divisor of 46
	// note the DDR register values are also calculated based on the 553MHz clock speed
	PLL_DDR1_CTRL_REG = PLL_DDR1_ENABLE_FLAG | PLL_DDR1_UPDATE_FLAG | PLL_DDR1_FACTOR_N(46);

	// Then wait for the PLL change to be processed by the hardware. (Does this wait for the PLL to actually lock? Not clear)
	while (PLL_DDR1_CTRL_REG & PLL_DDR1_UPDATE_FLAG) {}

	// there's also a clock divisor in this register; default (zero value) is divide-by-1
	DRAM_CFG_REG = DRAM_CFG_SRC_PLL_DDR1 | DRAM_CFG_UPDATE_FLAG;
	while(DRAM_CFG_REG & DRAM_CFG_UPDATE_FLAG) {}

	// as mentioned, A64 User Manual 3.3.6.4. says we should always release reset before releasing clock gate.
	MBUS_RST_REG |= MBUS_RST_RELEASE; // release MBUS reset
	MBUS_CLK_REG |= MBUS_CLK_CLOCK_ENABLE; // enable MBUS clock
	BUS_CLK_RESET_RELEASE_REG0 |= BUS_CLK_REG0_DRAM_GATING; // release DRAM reset
	BUS_CLK_GATING_REG0 |= BUS_CLK_REG0_DRAM_GATING; // enable DRAM clock

	// apparently that rule does not apply to this one. Perhaps because the clock enable is inside the block (next register)
	DRAM_CFG_REG |= DRAM_CFG_RESET_RELEASE; // release DRAM controller reset

	U32_REG(0x01C6300C) = 0x0000c00e; // enable DRAM controller clock via undocumented register
	// Some kind of readiness check. If we don't wait for this, MCTL_PGSR0_REG&1 never becomes true. u-boot uses a fixed delay, not this register
	while(U32_REG(0x01C63018) == 0) {}
}

</syntaxhighlight>

== Configure DRAM controller ==

According to the A64 User Manual, the DRAM controller initializes itself automatically. ''Compared to some other processors'', this is true. On some other platforms, software has to measure the delay on every wire between the memory chip and the CPU chip, and then decide the optimal delays and tell the DRAM controller how much delay to add to make them all equal. On A64 we don't have to do that, but we still have a bunch of undocumented registers to set...

Here's the sequence - just the raw register writes and magic numbers to make the controller work.
This might be expanded once more is known about the controller. Actually this was distilled down from u-boot, losing a lot of documentation in the process.

<syntaxhighlight lang="c">
// All identifiers and comments in this code were written for the wiki page or based on the A64 manual.
// It is the author's opinion that this code no longer contains any copyrightable material from u-boot.

#define U32_REG(addr) (*(volatile uint32_t*)(addr))
#define MCTL_CR0_REG U32_REG(0x01C62000)
#define MCTL_CR1_REG U32_REG(0x01C62004)
#define MCTL_PIR_REG U32_REG(0x01C63000)
#define MCTL_PGSR0_REG U32_REG(0x01C63010)

static const unsigned char AC_DELAYS[] =
	{5, 5, 13, 10, 2, 5, 3, 3, 0, 3, 3, 3, 1, 0, 0, 0, 3, 4, 0, 3, 4, 1, 4, 0, 1, 1, 0, 1, 13, 5, 4};

static const unsigned short READ_AND_WRITE_DELAYS[4][11] =
	{{0x0010, 0x0010, 0x0010, 0x0010, 0x0011, 0x0010, 0x0010, 0x0011, 0x0010, 0x0f01, 0x0f00},
	 {0x0011, 0x0011, 0x0011, 0x0011, 0x0111, 0x0111, 0x0111, 0x0111, 0x0011, 0x0a01, 0x0a00},
	 {0x0110, 0x0011, 0x0111, 0x0110, 0x0110, 0x0110, 0x0110, 0x0110, 0x0010, 0x0b00, 0x0b00},
	 {0x0111, 0x0011, 0x0011, 0x0111, 0x0111, 0x0111, 0x0111, 0x0111, 0x0011, 0x0c01, 0x0c00}};

static void memcpy32(u32 dst_, const u32 *src, size_t nregs)
{
	u32 *dst = (u32*)dst_;
	while(nregs--) {
		*dst++ = *src++;
	}
}

static void init_dram_controller()
{
	MCTL_CR1_REG = MCTL_CR0_REG = 0x004f19f4;

	memcpy32(0x01C63034, (const u32[]){0xc3, 0xa, 0x2}, 3);
	memcpy32(0x01C63050, (const u32[]){0x0381b009, 0x22a017c4, 0x0d0e180c, 0x00030314, 0x03060d0b, 0x0005500c, 0x07020308, 0x0505050c}, 8);
	U32_REG(0x01C63078) = 0x90006610;
	U32_REG(0x01C63080) = 0x02050102;
	U32_REG(0x01C63090) = 0x0021003a;

	U32_REG(0x01C63100) = 0x04005400;
	U32_REG(0x016C3104) = 0x4680c620; // This is probably a typo for 0x01C63104. This code was tested WITH the typo. Maybe you can fix this typo and see if it changes anything, then add some information here.

	U32_REG(0x01C63208) = 0x0000034A;
	U32_REG(0x01C63108) = 0x000008C0;

	U32_REG(0x01C63100) = 0x00005400; // unlock access to bit delay registers?

	// bytewise bit delay line timing. Low byte delay when reading, high byte delay when writing
	for (int i = 0; i < 4; i++)
		for (int j = 0; j < 11; j++) // DQ0-7, then DM, DQS, DQSN
			U32_REG(0x01C63310 + (i<<7) + (j<<2)) = READ_AND_WRITE_DELAYS[i][j];

	// command/address bit delay line timing
	for (int i = 0; i < 31; i++)
		*(u32*)(0x01C63210 + (i<<2)) = AC_DELAYS[i] << 8;

	__asm__ __volatile__("dsb" ::: "memory"); // not sure if necessary. Isn't cache disabled at this point?
	U32_REG(0x01C63100) = 0x04005400; // re-lock access to bit delay registers?

	U32_REG(0x01C63140) = 0x013b3bdd;

	MCTL_PIR_REG = 0x5F3;
	while(!(MCTL_PGSR0_REG & 1)) {}

	// if any of these bits are set, u-boot does some stuff that doesn't make any sense. Let's skip over that.
	assert((MCTL_PGSR0_REG & 0x0fe00000) == 0);

	while(!(U32_REG(0x01C63018) & 1)) {}

	U32_REG(0x01C6310C) = 0xc0aa0060;
	U32_REG(0x01C63140) = 0x817b7bfc;

	U32_REG(0x01C63120) = 0x00000303;
	U32_REG(0x01C630B8) = 0x0000021f;
	U32_REG(0x01C620D0) = 0x80103040; // if you forget this, accessing DRAM will hang.
}
</syntaxhighlight>

== Detecting memory size ==

Memory address consists of concatenated rank, row, bank, column, and column width.
'''ALL OF THESE''' must be set correctly.
They can also be set too low, but then the memory controller doesn't know about all of your memory.

So e.g. if you have 2 ranks, 12 row bits, 8 banks, and 12 column bits, the physical RAM address looks like this:

<pre>
 3 3 2 2 2 2 2 2 2 2 2 2 2 1 1 1 1 1 1 1 1 1 1 0 0 0 0 0 0 0 0 0 0
 1 0 9 8 7 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|XXXXXXXXX|R|          row          | bank|       column      |wid|
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 ^^^^^^^^^ ^
 ignored   rank
</pre>

Detecting the width of one of these parts consists of configuring the memory controller, writing a value at address N, then a different value at address N+(1&lt;&lt;SHIFT) and detecting whether it overwrote address N. If it overwrote the address, it means the memory controller is trying to use bits that aren't wired up to anything.

For example if we tell the controller there are 11 column bits, but actually there are only 8, then our address layout is like this:
<pre>
 3 3 2 2 2 2 2 2 2 2 2 2 2 1 1 1 1 1 1 1 1 1 1 0 0 0 0 0 0 0 0 0 0
 1 0 9 8 7 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|XXXXXXX|R|          row          | bank|XXXXX|     column    |wid|
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
</pre>

If we write address 0x444 the memory controller will access rank 0, row 0, bank 0, column 0x111, however since there are only 8 column bits in the RAM chip, it will access column 0x11, the same as address 0x044. By detecting that 0x444 and 0x044 access the same byte of memory, we know the RAM chip has less than 11 column bits. Of course, if we correctly configure the controller for 8 column bits, the 0x400 bit will be one of the bank bits, and then we use the same procedure to detect the size of the bank address and then the row address.

There's one other thing to note: each rank can have a different number of row+bank+column bits.

All PinePhones have 2 identical ranks (1 bit), 8 banks (3 bits), and a 4-byte column width (2 bits) which is hard-wired. So you can skip detecting those if you want. Indeed the init code above has the rank detection stripped out.

From the CPU's point of view the RAM starts at CPU address 0x40000000. If the CPU accesses address 0x40000000 it will go to address 0x00000000 on the RAM chip. Address 0x80000000 goes to address 0x40000000 on the RAM chip, 0xFFFFFFFF goes to RAM address 0xBFFFFFFF, etc.



The memory chip is different depending on whether you got the "convergence package" or not.<br>
Non-convergence package: you have the following chip with 2GB of memory. [http://files.pine64.org/doc/datasheet/pinephone/ATL3A1632H12A_mobile_lpddr3_11x11.5_v1.0_1600.pdf Datasheet]<br>
Convergence package: you have an unknown one with 4GB of memory.<br>
You might notice that despite having a 4GB RAM chip you can only access 3GB because the CPU can't go above address 0xFFFFFFFF which corresponds to 3GB-1 on the RAM chip. That's why the convergence package phone is advertised as 3GB despite having a 4GB chip.

According to the datasheet, on the non-convergence package it should be 15 row bits, 10 column bits.<br>
The convergence package seems to also be 15 row bits which means it presumably has 11 column bits.

=== Controller memory size registers ===

<pre>
 
          31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 09 08 07 06 05 04 03 02 01 00
Rank 0   +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
01C62000 |?????????????????|25|?????|BL|21|20|1T|  type  |SQ|?????|FW|    page   |    row    |??|8B|01|DR|
         +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+

Rank 1   +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
01C62004 |???????????????????????????????????????????????????????????|    page   |    row    |??|8B|?????|
         +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+

All ??s should be set to 0 but may have unknown function.

BL: unknown. Full name "BL8". Set to true on the Pinephone.
25: if set, accessing rank 1 hangs the CPU
21, 20: should be 0. Playing with these bits caused random inversions of bit 4 of the address, even after setting them back to the normal 0 value. This condition ended with DRAM controller power-cycle. Also sometimes the CPU hangs.
1T: unknown. Set to 1 on the Pinephone. "1T" is official name. If set to 0, mode is "2T"
type: Select memory type. Set to 7 (LPDDR3) on the Pinephone.
  2: DDR2. 3: DDR3. 6: LPDDR2 (i.e. 2|4). 7: LPDDR3 (i.e. 3|4).
SQ: sequential mode. Set to 0 on the Pinephone.
  Unknown effect
FW: full-width. Set to true on the Pinephone.
  If you set it to false, the controller only accesses the bottom 16 bits out of each 32-bit word,
  and does two accesses to complete a 32-bit access. TODO: figure out how this affects the bit layout of addresses.
page: Number of page bits (column + column width), with some bias (I think raw value 0 means 2 page bits, up to raw value 15 means 17 page bits. Maybe it's actually the number of columns not including the width? This is inconsistent with u-boot)
  Minimum (raw) value is 6, maximum is 12. Values outside this range are treated as raw value 7.
row: Number of row bits, with some bias (I think raw value 0 means 1 row bit, up to 15 means 16 row bits)
  Minimum (raw) value is 10, maximum is unknown, possibly 15. Values outside this range are treated as raw value 15.
8B: set to 1 for 8 banks (3 bank bits), 0 for 4 banks (2 bank bits)
01: if set, accessing rank 1 hangs the CPU.
DR: enables second rank (dual-rank mode).

</pre>

Optimum raw values on the convergence package (discovered by brute force): page=10, row=14.<br>
Without the convergence package it's probably page=9, row=13 but this remains to be verified.

=== Accessing the top 1GB on the convergence package ===

If you bought the convergence package you have 4GB of RAM, but the 3GB address window on the CPU only allows you to use 3/4 of it normally.

But wait - it's divided into two ranks and the memory controller has separate settings for each rank. If we lie to the memory controller and say the lower rank is only 1GB instead of 2GB, now our memory layout divided into GBs looks like this: [internal memory & I/O registers][rank 0 with mangled address pattern][rank 1 first half][rank 1 second half] and we should be able to access the top GB. This wasn't tested yet.

If we do it by shrinking the page size, it probably won't break refresh as refresh is row-wise and the memory controller doesn't handle the data during refresh (so it won't only refresh half of each page). This wasn't confirmed yet. Shrinking anything other than the row bits will mangle the address layout of rank 0 due to deleting address bits from the middle, but shrinking the row bits has a risk of breaking refresh (leading to a debugging nightmare six months down the line when RAM randomly loses data). Data to be transferred to/from the upper 1GB would have to be staged in the first half of rank 1 (or a striped pattern in rank 0).

This could possibly be used as a swap device, or as main memory, depending on how fast and reliable changing the controller settings is.

=== Crossbar priority ===

The controller includes a priority crossbar switch which allocates memory bandwidth to various clients, such as the CPU, GPU, display output. This is left uninitialized here because it doesn't seem to matter if you just want to run the CPU, however this could lead to starvation for some clients - especially for the display output which has to output pixels at a certain rate or else the display presumably just breaks (black pixels or lost synchronization).

[[Category:PinePhone]]
[[Category:ARM]]
````

<a id="w004719"></a>

## W004719 — Timekeeping in virtual machines

Bron-ID: W004719
Namespace: Main (0)
Revisie: 30321 | Datum: 2026-03-17T19:06:47Z
Bron: https://wiki.osdev.org/Timekeeping_in_virtual_machines
Revisie-URL: https://wiki.osdev.org/index.php?title=Timekeeping_in_virtual_machines&oldid=30321

````text
There are several ways to keep track of time in a VM, but they're either very slow (e.g. HPET) or do not work correctly if the VM is migrated (e.g. TSC).

To work around this, VMs such as QEMU/KVM provide several ways keep track of time whilst sacrificing little performance.

== KVM_HC_CLOCK_PAIRING ==

This hypercall is used to get the parameters to calculate a host's clock (KVM_CLOCK_PAIRING_WALLCLOCK for CLOCK_REALTIME).

The host copies the following structure to a physical address given by the guest:

<syntaxhighlight lang="c">
struct kvm_clock_pairing {
    s64 sec;
    s64 nsec;
    u64 tsc;
    u32 flags;
    u32 pad[9];
};
</syntaxhighlight>

A hypercall is performed with the `vmcall` instruction. On KVM, RBX, RCX, RDX and RSI are used for arguments, RAX as the hypercall number and as the return value. No other registers are clobbered (unless explicitly noted).

For example, calling KVM_HC_CLOCK_PAIRING can be done as follows on x86_64:

<syntaxhighlight lang="asm">
; rdi: physical address to copy structure to
; rsi: clock type (KVM_CLOCK_PAIRING_WALLCLOCK = 0)
kvm_hc_clock_pairing:
    mov eax, 9   ; KVM_HC_CLOCK_PAIRING
    mov rbx, rdi
    mov rcx, rsi
    vmcall
    ret
</syntaxhighlight>

== pvclock ==

pvclock is a simple protocol and the fastest way to properly track system time in a VM.

To use it, write a 64-bit 4-byte aligned physical address with bit 0 set to 1 to MSR_KVM_SYSTEM_TIME_NEW (<code>0x4b564d01</code>).
The presence of this MSR is indicated by bit 3 in EAX from leaf 0x4000_0001 of CPUID.

The host will write the following structure to this address:

<syntaxhighlight lang="C">
struct pvclock_vcpu_time_info {
    u32 version;
    u32 pad0;
    u64 tsc_timestamp;
    u64 system_time;
    u32 tsc_to_system_mul;
    s8 tsc_shift;
    u8 flags;
    u8 pad[2];
};
</syntaxhighlight>

The host will automatically update this structure when necessary (e.g. when finishing a migration).

The system time <b>in nanoseconds</b> is calculated as such:

<syntaxhighlight lang="C">
time = rdtsc() - tsc_timestamp
if (tsc_shift >= 0)
    time <<= tsc_shift;
else
    time >>= -tsc_shift;
time = (time * tsc_to_system_mul) >> 32
time = time + system_time
</syntaxhighlight>

The version field is used to detect when the structure has been / is being updated.
If the version is odd an update is in progress and the guest must not read the other fields yet.

== Hyper-V TSC page == 

<syntaxhighlight lang="C">
struct ms_hyperv_tsc_page {
    volatile u32 tsc_sequence;
    u32 reserved1;
    volatile u64 tsc_scale;
    volatile s64 tsc_offset;
    u64 reserved2[509];
};
</syntaxhighlight>

== See also ==

=== References ===

* [https://docs.kernel.org/virt/kvm/x86/hypercalls.html Linux KVM Hypercall]
* [https://docs.kernel.org/virt/kvm/x86/msr.html KVM-specific MSRs]
* [https://opensource.com/article/17/6/timekeeping-linux-vms An introduction to timekeeping in Linux VMs]

[[Category:Time]]
[[Category:Virtualization]]
````

<a id="w004720"></a>

## W004720 — User:Demindiro/Timekeeping in virtual machines

Bron-ID: W004720
Namespace: User (2)
Revisie: 27315 | Datum: 2022-08-29T20:00:40Z
Bron: https://wiki.osdev.org/User:Demindiro/Timekeeping_in_virtual_machines
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADemindiro%2FTimekeeping_in_virtual_machines&oldid=27315
Doorverwijzing: Timekeeping in virtual machines

````text
#REDIRECT [[Timekeeping in virtual machines]]
````

<a id="w004721"></a>

## W004721 — User talk:Bewing

Bron-ID: W004721
Namespace: User talk (3)
Revisie: 27337 | Datum: 2022-09-03T16:14:11Z
Bron: https://wiki.osdev.org/User_talk:Bewing
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ABewing&oldid=27337

````text
= Contributed section removed =
The following section is being removed from [[Partition Table]] as that page is a reference for an existing standard, not a place for proposals. (As an LBA newbie, I found myself confused by it.) As it was contributed by Bewing, it seems polite to paste it here on Bewing's usertalk page rather than remove it entirely.

=="Unofficial" 48 bit LBA Proposed MBR Format==

The two CHS fields are unused in all current drives, leaving only the two 32 bit LBA fields to "do all the work". But there never was any 32 bit
LBA addressing mode -- the current "standard" LBA addressing mode is 48 bits. So it is reasonable to try to redefine the Partition Table to eliminate the
unused CHS fields, and use the extra space to extend the two LBA fields to a full 48 bit size. This would eliminate the impending obsolescence
of the entire MBR scheme.

It seems reasonable to try to preserve (as much as possible) the current Partition Table structure. Therefore, the following alternate
structure for 48 bit LBA Partition Table entries is proposed:

{| {{wikitable}}
! Element (offset)
! Size
! Description
|-
| 0
| byte
| Bitflags field: 1 = not bootable, 0x81 = bootable (or "active")
|-
| 1
| byte
| Signature-1 (0x14)
|-
| 2
| uint16_t
| Partition Start LBA (high 16-bit of 48 bit value)
|-
| 4
| byte
| System ID
|-
| 5
| byte
| Signature-2 (0xeb)
|-
| 6
| uint16_t
| Partition Length (high 16-bit of 48 bit value)
|-
| 8
| uint32_t
| Partition Start LBA (low uint32_t)
|-
| 12
| uint32_t
| Partition Length (low uint32_t)
|}

Note: The basic intent is to use bit #0 (value = 1) in the bitflags field as a "48 bit LBA" indicator, preserve the offsets and functions
of the bitflags and System ID fields, and use the two available aligned 16-bit values to extend the LBA fields.
When detecting a valid partition entry, a little extra verification turns out to be useful, so the "extra" bytes in the table entry should be loaded with signature values.
````

<a id="w004722"></a>

## W004722 — QEMU RISC-V Virt Meaty Skeleton

Bron-ID: W004722
Namespace: Main (0)
Revisie: 30471 | Datum: 2026-04-10T16:14:38Z
Bron: https://wiki.osdev.org/QEMU_RISC-V_Virt_Meaty_Skeleton
Revisie-URL: https://wiki.osdev.org/index.php?title=QEMU_RISC-V_Virt_Meaty_Skeleton&oldid=30471

````text
{{BeginnersWarning}}
{{Rating|1}}
{{Template:Kernel designs}}

This tutorial assumes you have completed [[RISC-V_Bare_Bones|RISC-V Bare Bones]] on the QEMU <code>virt</code> board, or alternatively, [[HiFive-1_Bare_Bones|HiFive-1 Bare Bones]]. If not, you should complete them first for an overview of how to boot your own operating system on RISC-V. This tutorial is deliberately brief on concepts that have already been covered in the bare bones tutorials and their transitive prerequisites.

The bare bones tutorials provide minimal examples that are not structured to enable sustainable mid- to long-term development of the codebase. This tutorial attempts to rectify that by providing a well-structured project that should serve you well through your OSDev journey with the following features:

* Hierarchical project structure with <code>make</code> build system for sustainable mid- to long-term development
* Includes <code>debug</code> target for debugging with [[GDB|GDB]] (requires cross-debugger targeting <code>riscv64-elf</code>)
* Basic console output through NS16550A UART
* Convenience wrappers for powering off and rebooting the device
* Working <code>kprintf</code> supporting base format specifiers (no floating point support; no sub-specifiers; no <code>n</code> specifier) to facilitate <code>printf</code> debugging
* <code>panic</code> function for kernel panics

==RISC-V Bare Bones==

:''Main article:'' [[RISC-V_Bare_Bones|RISC-V Bare Bones]]

It is assumed you have completed RISC-V Bare Bones or another comparable bare bones tutorial. Though not a strict requirement, it is useful for confirming that your development environment works and explaining a few basic things.

We won't be reusing any code from those tutorials though, so throw it away (or save it to an archive) and we'll start over again.

==Building a Cross-Compiler==

:''Main article:'' [[GCC_Cross-Compiler|GCC Cross-Compiler]], [[Why_do_I_need_a_Cross_Compiler%3F|Why do I need a Cross Compiler?]]

You ''must'' use a [[GCC_Cross-Compiler|GCC Cross-Compiler]] in this tutorial as in the [[RISC-V_Bare_Bones|RISC-V Bare Bones]] tutorial, with <code>riscv64-elf</code> as the target.

You ''must'' configure your cross-binutils with the <tt>--with-sysroot</tt> option, otherwise linking will mysteriously fail with the ''this linker was not configured to use sysroots'' error message. If you forgot to configure your cross-binutils with that option, you'll have to rebuild it, but you can keep your cross-gcc.

==Building a Cross-Debugger (optional)==

:''Main article:'' [[GDB|GDB]]

If you wish to debug your kernel with [[GDB|GDB]], you'll need to build it separately with <code>riscv64-elf</code> as the target, since it's not included by default with GCC and Binutils. Otherwise, if <code>printf</code> debugging is your style, you may safely skip this section.

The process is similar to building a cross-GCC or cross-binutils, and you may refer to the [https://www.linuxfromscratch.org/blfs/view/svn/general/gdb.html GDB page in BLFS] for most details sans cross-debugging support, but we'll go through the process in detail here anyway.

Fetch the latest version of GDB through [https://ftp.gnu.org/gnu/gdb/ https://ftp.gnu.org/gnu/gdb/]. The latest version at the time of writing is 12.1:

<syntaxhighlight lang="shell-session">
$ export GDB_VERSION="12.1"
$ wget https://ftp.gnu.org/gnu/gdb/gdb-${GDB_VERSION}.tar.xz
</syntaxhighlight>

Unpack the archive:

<syntaxhighlight lang="shell-session">
$ tar xvf gdb-${GDB_VERSION}.tar.xz
</syntaxhighlight>

Now move into the source directory:

<syntaxhighlight lang="shell-session">
$ pushd gdb-${GDB_VERSION}/
</syntaxhighlight>

Create a build directory and move into it:

<syntaxhighlight lang="shell-session">
$ mkdir build
$ pushd build/
</syntaxhighlight>

Now export a few variables as with building a cross-GCC or cross-binutils:

<syntaxhighlight lang="shell-session">
$ export PREFIX="$HOME/opt/cross"
$ export TARGET=riscv64-elf
$ export PATH="$PREFIX/bin:$PATH"
</syntaxhighlight>

Configuration options are mostly the same as with building cross-GCC or cross-binutils. In particular, you may wish to enable the following features:

* <code>--enable-tui=yes</code>: Enables TUI mode for debugging. Requires development headers for Ncurses to be installed on the build host
* <code>--with-expat</code>: Build with Expat, a library for XML parsing. Requires development headers for Expat to be installed on the build host

<syntaxhighlight lang="shell-session">
$ ../configure --target=$TARGET \
  --prefix="$PREFIX" \
  --with-sysroot \
  --disable-nls \
  --disable-werror \
  --enable-languages=c,c++ \
  --without-headers \
  --enable-tui=yes \
  --with-expat
</syntaxhighlight>

Now build the source code (this can take a while):

<syntaxhighlight lang="shell-session">
$ make -j$(nproc)
</syntaxhighlight>

And install:

<syntaxhighlight lang="shell-session">
$ make -C gdb install
</syntaxhighlight>

If you wish to keep the source tree available for conveniently re-building GDB in the future (e.g. with a different set of options), clean up the build files now:

<syntaxhighlight lang="shell-session">
$ make clean
</syntaxhighlight>

Move one level up to the project root:

<syntaxhighlight lang="shell-session">
$ popd
</syntaxhighlight>

Remove the <code>build/</code> directory we created:

<syntaxhighlight lang="shell-session">
$ rm -rf build/
</syntaxhighlight>

Move up one more level:

<syntaxhighlight lang="shell-session">
$ popd
</syntaxhighlight>

Double check our cross-debugger is properly installed:

<syntaxhighlight lang="shell-session">
$ which -- $TARGET-gdb || echo $TARGET-gdb is not in the PATH
</syntaxhighlight>

Now you may safely remove the source tree and archive, if you wish:

<syntaxhighlight lang="shell-session">
$ rm -rf gdb-${GDB_VERSION}/
$ rm gdb-${GDB_VERSION}.tar.xz
</syntaxhighlight>

If <code>$PREFIX/bin</code> is not in your path already, you may wish to persist it by writing it to your profile:

<syntaxhighlight lang="shell-session">
$ echo "export PATH=\"\$HOME/opt/cross/bin:\$PATH\"" >> $HOME/.profile
$ source $HOME/.profile
</syntaxhighlight>

==Dependencies==

You will need these dependencies in order to complete the tutorial:

* [https://www.qemu.org/docs/master/system/index.html QEMU full-system emulator] for 64-bit RISC-V, of which your distribution-provided package should suffice
* <code>riscv64-elf</code> toolchain, as discussed above
* (Optional, required for debugging with GDB) [https://www.linuxfromscratch.org/blfs/view/svn/general/gdb.html GNU debugger] targeting <code>riscv64-elf</code>, as discussed above

This tutorial requires a GNU/Linux system, or a similar enough system. The BSD systems may almost work. OS X is not supported but can possibly be made to work with some changes. Windows is not supported, but Windows environments like Cygwin and Windows Subsystem For Linux (WSL) might work.


==Acknowledgements==

The initial project setup is based on [https://github.com/twilco/riscv-from-scratch RISC-V from scratch] (C runtime, linker script) and [https://github.com/sgmarz/osblog The Adventures of OS] (overall project structure, Makefile), with the section on building a cross-GDB based on [https://linuxfromscratch.org/ (Beyond) Linux From Scratch].

==Source Code==

Fetch the source code from the <code>v0.0.1</code> release of [https://github.com/DonaldKellett/marvelos https://github.com/DonaldKellett/marvelos] code-named "Meaty Skeleton", using [https://git-scm.com/ Git]:

<syntaxhighlight lang="shell-session">
$ git clone --branch v0.0.1 https://github.com/DonaldKellett/marvelos.git
</syntaxhighlight>

Operating systems development is about being an expert. Take the time to read the code carefully through and understand it. Please seek further information and help if you don't understand aspects of it. This code is minimal and almost everything is done deliberately, often to pre-emptively solve future problems.

===Project Structure===

* <code>Makefile</code>: used for building the project with <code>make</code> build system
* <code>misc/riscv64-virt.dts</code>: device tree of QEMU RISC-V <code>virt</code> board. See [https://twilco.github.io/riscv-from-scratch/2019/04/27/riscv-from-scratch-2.html#finding-our-stack Finding our stack] for details
* <code>src/</code>: source tree containing files used to build the project
** <code>src/asm/crt0.s</code>: minimal C runtime; see [https://twilco.github.io/riscv-from-scratch/2019/04/27/riscv-from-scratch-2.html#stop--hammertime-runtime Stop! Runtime!] for details
** <code>src/lds/riscv64-virt.ld</code>: custom linker script adapted from the output of <code>riscv64-unknown-elf-ld --verbose</code>; see [https://twilco.github.io/riscv-from-scratch/2019/04/27/riscv-from-scratch-2.html#link-it-up Link it up] for details
** <code>src/common/</code>: common utilities and library functions for use across our project
** <code>src/syscon/</code>: [https://elixir.bootlin.com/linux/v4.0/source/Documentation/devicetree/bindings/mfd/syscon.txt syscon] drivers for poweroff and reboot
** <code>src/uart/</code>: UART drivers and I/O-related code
** <code>src/kmain.c</code>: Entry point for our kernel

===Makefile===

The <code>Makefile</code> makes extensive use of environment variables for conciseness and configurability. The following top-level targets are supported (along with sub-targets for building only a specific subsystem):

* <code>all</code>: to build the kernel image
* <code>run</code>: to build the kernel image and run it in QEMU
* <code>debug</code>: to build the kernel image and run it in debug mode to enable debugging with GDB. The "remote" GDB server runs at port <code>$GDB_PORT</code> on the host (default: <code>1234</code>). Requires a cross-debugger to be installed; see above for details
* <code>clean</code>: to clean up build files so the next build won't be affected by the previous build

Run <code>make $TARGET</code> with <code>TARGET</code> set to your desired target to execute the specified target. For example, to build the kernel image and run it in QEMU:

<syntaxhighlight lang="shell-session">
$ make run
</syntaxhighlight>

Additionally, the default target is <code>all</code> if not specified:

<syntaxhighlight lang="shell-session">
$ make
</syntaxhighlight>

<syntaxhighlight lang="make">
# Build
CC=riscv64-elf-gcc
CFLAGS=-ffreestanding -nostartfiles -nostdlib -nodefaultlibs
CFLAGS+=-g -Wl,--gc-sections -mcmodel=medany
RUNTIME=src/asm/crt0.s
LINKER_SCRIPT=src/lds/riscv64-virt.ld
KERNEL_IMAGE=kmain

# QEMU
QEMU=qemu-system-riscv64
MACH=virt
MEM=128M
RUN=$(QEMU) -nographic -machine $(MACH) -m $(MEM)
RUN+=-bios none -kernel $(KERNEL_IMAGE)

# QEMU (debug)
GDB_PORT=1234

all: uart syscon common kmain
	$(CC) *.o $(RUNTIME) $(CFLAGS) -T $(LINKER_SCRIPT) -o $(KERNEL_IMAGE)

uart:
	$(CC) -c src/uart/uart.c $(CFLAGS) -o uart.o

syscon:
	$(CC) -c src/syscon/syscon.c $(CFLAGS) -o syscon.o

common:
	$(CC) -c src/common/common.c $(CFLAGS) -o common.o

kmain:
	$(CC) -c src/kmain.c $(CFLAGS) -o kmain.o

run: all
	$(RUN)

debug: all
	$(RUN) -gdb tcp::$(GDB_PORT) -S

clean:
	rm -vf *.o
	rm -vf $(KERNEL_IMAGE)
</syntaxhighlight>

===Kernel source===

====src/common/common.h====

<syntaxhighlight lang="c">
#ifndef COMMON_H
#define COMMON_H

int toupper(int);
void panic(const char *, ...);

#endif
</syntaxhighlight>

====src/common/common.c====

<syntaxhighlight lang="c">
#include <stdarg.h>
#include "common.h"
#include "../uart/uart.h"

int toupper(int c) {
  return 'a' <= c && c <= 'z' ? c + 'A' - 'a' : c;
}

void panic(const char *format, ...) {
  kputs("Kernel panic!");
  kputs("Reason:");
  va_list arg;
  va_start(arg, format);
  kvprintf(format, arg);
  va_end(arg);
  asm volatile ("wfi");
}
</syntaxhighlight>

====src/syscon/syscon.h====

<syntaxhighlight lang="c">
#ifndef SYSCON_H
#define SYSCON_H

// "test" syscon-compatible device is at memory-mapped address 0x100000
// according to our device tree
#define SYSCON_ADDR 0x100000

void poweroff(void);
void reboot(void);

#endif
</syntaxhighlight>

====src/syscon/syscon.c====

<syntaxhighlight lang="c">
#include <stdint.h>
#include "syscon.h"
#include "../uart/uart.h"

void poweroff(void) {
  kputs("Poweroff requested");
  *(uint32_t *)SYSCON_ADDR = 0x5555;
}

void reboot(void) {
  kputs("Reboot requested");
  *(uint32_t *)SYSCON_ADDR = 0x7777;
}
</syntaxhighlight>

====src/uart/uart.h====

<syntaxhighlight lang="c">
#ifndef UART_H
#define UART_H

#include <stddef.h>
#include <stdarg.h>

// 0x10000000 is memory-mapped address of UART according to device tree
#define UART_ADDR 0x10000000

void uart_init(size_t);
int kputchar(int);
int kputs(const char *);
void kvprintf(const char *, va_list);
void kprintf(const char *, ...);

#endif
</syntaxhighlight>

====src/uart/uart.c====

<syntaxhighlight lang="c">
#include <stddef.h>
#include <stdint.h>
#include <stdarg.h>
#include <limits.h>
#include "uart.h"
#include "../common/common.h"

#define to_hex_digit(n) ('0' + (n) + ((n) < 10 ? 0 : 'a' - '0' - 10))

/*
 * Initialize NS16550A UART
 */
void uart_init(size_t base_addr) {
  volatile uint8_t *ptr = (uint8_t *)base_addr;

  // Set word length to 8 (LCR[1:0])
  const uint8_t LCR = 0b11;
  ptr[3] = LCR;

  // Enable FIFO (FCR[0])
  ptr[2] = 0b1;

  // Enable receiver buffer interrupts (IER[0])
  ptr[1] = 0b1;

  // For a real UART, we need to compute and set the baud rate
  // But since this is an emulated UART, we don't need to do anything
  //
  // Assuming clock rate of 22.729 MHz, set signaling rate to 2400 baud
  // divisor = ceil(CLOCK_HZ / (16 * BAUD_RATE))
  //         = ceil(22729000 / (16 * 2400))
  //         = 592
  //
  // uint16_t divisor = 592;
  // uint8_t divisor_least = divisor & 0xFF;
  // uint8_t divisor_most = divisor >> 8;
  // ptr[3] = LCR | 0x80;
  // ptr[0] = divisor_least;
  // ptr[1] = divisor_most;
  // ptr[3] = LCR;
}

static void uart_put(size_t base_addr, uint8_t c) {
  *(uint8_t *)base_addr = c;
}

int kputchar(int character) {
  uart_put(UART_ADDR, (uint8_t)character);
  return character;
}

static void kprint(const char *str) {
  while (*str) {
    kputchar((int)*str);
    ++str;
  }
}

int kputs(const char *str) {
  kprint(str);
  kputchar((int)'\n');
  return 0;
}

// Limited version of vprintf() which only supports the following specifiers:
//
// - d/i: Signed decimal integer
// - u: Unsigned decimal integer
// - o: Unsigned octal
// - x: Unsigned hexadecimal integer
// - X: Unsigned hexadecimal integer (uppercase)
// - c: Character
// - s: String of characters
// - p: Pointer address
// - %: Literal '%'
//
// None of the sub-specifiers are supported for the sake of simplicity.
// The `n` specifier is not supported since that is a major source of
// security vulnerabilities. None of the floating-point specifiers are
// supported since floating point operations don't make sense in kernel
// space
//
// Anyway, this subset should suffice for printf debugging
void kvprintf(const char *format, va_list arg) {
  while (*format) {
    if (*format == '%') {
      ++format;
      if (!*format)
	return;
      switch (*format) {
      case 'd':
      case 'i':
	{
	  int n = va_arg(arg, int);
	  if (n == INT_MIN) {
	    kprint("-2147483648");
	    break;
	  }
	  if (n < 0) {
	    kputchar('-');
	    n = ~n + 1;
	  }
	  char lsh = '0' + n % 10;
	  n /= 10;
	  char buf[9];
	  char *p_buf = buf;
	  while (n) {
            *p_buf++ = '0' + n % 10;
	    n /= 10;
	  }
	  while (p_buf != buf)
	    kputchar(*--p_buf);
	  kputchar(lsh);
	}
	break;
      case 'u':
        {
	  unsigned n = va_arg(arg, unsigned);
	  char lsh = '0' + n % 10;
	  n /= 10;
	  char buf[9];
	  char *p_buf = buf;
	  while (n) {
            *p_buf++ = '0' + n % 10;
	    n /= 10;
	  }
	  while (p_buf != buf)
	    kputchar(*--p_buf);
	  kputchar(lsh);
	}
	break;
      case 'o':
        {
	  unsigned n = va_arg(arg, unsigned);
	  char lsh = '0' + n % 8;
	  n /= 8;
	  char buf[10];
	  char *p_buf = buf;
	  while (n) {
            *p_buf++ = '0' + n % 8;
	    n /= 8;
	  }
	  while (p_buf != buf)
	    kputchar(*--p_buf);
	  kputchar(lsh);
	}
	break;
      case 'x':
        {
	  unsigned n = va_arg(arg, unsigned);
	  char lsh = to_hex_digit(n % 16);
	  n /= 16;
	  char buf[7];
	  char *p_buf = buf;
	  while (n) {
            *p_buf++ = to_hex_digit(n % 16);
	    n /= 16;
	  }
	  while (p_buf != buf)
	    kputchar(*--p_buf);
	  kputchar(lsh);
	}
	break;
      case 'X':
        {
	  unsigned n = va_arg(arg, unsigned);
	  char lsh = to_hex_digit(n % 16);
	  n /= 16;
	  char buf[7];
	  char *p_buf = buf;
	  while (n) {
            *p_buf++ = to_hex_digit(n % 16);
	    n /= 16;
	  }
	  while (p_buf != buf)
	    kputchar(toupper(*--p_buf));
	  kputchar(toupper(lsh));
	}
	break;
      case 'c':
	kputchar(va_arg(arg, int));
	break;
      case 's':
	kprint(va_arg(arg, char *));
	break;
      case 'p':
	{
          kprint("0x");
	  size_t ptr = va_arg(arg, size_t);
	  char lsh = to_hex_digit(ptr % 16);
	  ptr /= 16;
	  char buf[15];
	  char *p_buf = buf;
	  while (ptr) {
            *p_buf++ = to_hex_digit(ptr % 16);
	    ptr /= 16;
	  }
	  while (p_buf != buf)
	    kputchar(*--p_buf);
	  kputchar(lsh);
	}
	break;
      case '%':
	kputchar('%');
	break;
      default:
	kputchar('%');
	kputchar(*format);
      }
    } else
      kputchar(*format);
    ++format;
  }
}

void kprintf(const char *format, ...) {
  va_list arg;
  va_start(arg, format);
  kvprintf(format, arg);
  va_end(arg);
}
</syntaxhighlight>

====src/kmain.c====

<syntaxhighlight lang="c">
#include "uart/uart.h"
#include "syscon/syscon.h"
#include "common/common.h"

#define ARCH "RISC-V"
#define MODE 'M'

void kmain(void) {
  uart_init(UART_ADDR);

  kprintf("Hello %s World!\n", ARCH);
  kprintf("We are in %c-mode!\n", MODE);

  poweroff();
}
</syntaxhighlight>

==Running the project==

Invoke <code>make run</code> in the project root. You should see the following output:

<pre>
Hello RISC-V World!
We are in M-mode!
Poweroff requested
</pre>

==Final remarks and going further==

This is by no means an end to your OSDev adventures on RISC-V. Make the OS kernel your own! Add support for memory management, interrupt handling, porting newlib ... you name it. Give your own RISC-V OS a creative name! "maRVelOS" / <code>marvelos</code> is already taken by [[User:Donaldsebleung]] though ;-)

Also note that this example RISC-V OS runs in M-mode usually reserved for firmware, rather than the S-mode recommended for RISC-V supervisors (OSes). If you wish to follow the RISC-V conventions closely, you may want to look into RISC-V privilege modes and [https://github.com/riscv-software-src/opensbi OpenSBI] early on and port your OS kernel accordingly. More information about RISC-V privilege modes available on [[RISC-V#Privileges|our wiki]].

==See also==

* [[RISC-V|RISC-V]]
* [[RISC-V_Bare_Bones|RISC-V Bare Bones]]
* [[HiFive-1_Bare_Bones|HiFive-1 Bare Bones]]
* [[Meaty_Skeleton|Meaty Skeleton]]

[[Category:RISC-V]]
[[Category:Bare_bones_tutorials]]
[[Category:C]]
[[Category:QEMU]]
[[Category:Level_1_Tutorials]]

===External links===

* [https://github.com/twilco/riscv-from-scratch RISC-V from scratch]
* [https://github.com/sgmarz/osblog The Adventures of OS]
* [https://linuxfromscratch.org/ Linux From Scratch]
* [https://github.com/riscv-software-src/opensbi OpenSBI]
````

<a id="w004725"></a>

## W004725 — AVX2

Bron-ID: W004725
Namespace: Main (0)
Revisie: 29871 | Datum: 2026-01-12T02:58:53Z
Bron: https://wiki.osdev.org/AVX2
Revisie-URL: https://wiki.osdev.org/index.php?title=AVX2&oldid=29871

````text
{{In Progress}}

= Advanced Vector Extensions =
AVX or (Advanced Vector Extensions) are extensions to the x86 architecture introduced by Intel with the [[https://en.wikipedia.org/wiki/Sandy_Bridge SandyBridge]] micro-architecture. AVX adds 86 instructions to the CPU instruction set, it extends the 128 Bit XMM registers to 256 Bit YMM registers, these registers operate as lower-upper halves meaning that XMMx contains the low 128 bits of YMMx, thus, the AVX instruction set increases the size of memory transfers and parallel floating point computations. Effective usage of these extensions may vastly increase the performance of your program.

The SIMD extensions (MMX/SSE/AVX...) can perform packed and scalar operations on multiple data types : Single Precision Floats (float), Double Precision Floats (double), Bytes, Words, DWORDs and QWORDs.

= Scalar and Packed computations =
=== Scalar computations ===
- in SIMD, a scalar computation (arithmetic, bit manipulations...) acts only on the low value of the SIMD register, scalar computations will not clear the high bits of the register and are only applied to floating point values.
=== Packed computations ===
- Packed computations involve parallel computing of different values across the SIMD register (XMM/YMM/ZMM), the number of values that can be computed at once is decided by the length of the register and the value. For example : You can divide 2 QWORDS at once in an XMM, 4 QWORDS in an YMM, and 4 DWORDS in an XMM.

'''Example :'''
Here we will use AVX to multiply 8 floats at once, the assembly function Mul8floats will do that, it will multiply 8 values in Dest by Src and store the result in Dest.

<pre>
; extern void Mul8floats(float* Dest, float* Src)
Mul8floats:
; RCX contains pointer to Src
; RDX contains pointer of Dest
     vmovups ymm0, [rcx]
     vmovups ymm1, [rdx]
     vmulps ymm0, ymm0, ymm1 ; Packed multiply of 8 floats across ymm0 by 8 floats in ymm1 and store result in ymm0
     vmovups [rcx], ymm0 ; Store the result in memory (float* Dest)
     ret
</pre>

Result on MSVC with intrinsics:
<pre>Mul 134217728 values, normal : 261.955000ms , AVX : 34.871000ms</pre> 

= AVX2 =
AVX2 expands the AVX instruction set, it includes an expansion of the SSE Vector integer instructions to 256 Bits, Gather support, vector shifts and more.


= Integer Arithmetic =
To introduce AVX2, we should see how these registers work :
lets take for example the previous instruction '''VPADDB/W/D/Q'''


<pre>
; Available in AVX
vmulpd ymm0, ymm0, ymm1 ; (Floating point computations)
vpaddb xmm0, xmm1, xmm2 ; (Addition of 16 Byte integers in XMM1 by integers in XMM2 and store result in XMM0)
; Requires AVX2
vpaddb ymm0, ymm1, ymm2 ; (Addition of 32 Byte integers in YMM1 by integers in YMM2 and store result in YMM0)
</pre>

[[Category:X86]]
````

<a id="w004726"></a>

## W004726 — Lingemu

Bron-ID: W004726
Namespace: Main (0)
Revisie: 28097 | Datum: 2023-07-09T22:34:09Z
Bron: https://wiki.osdev.org/Lingemu
Revisie-URL: https://wiki.osdev.org/index.php?title=Lingemu&oldid=28097

````text
{{stub}}
[https://lingemu.org Lingemu] is a machine virtualizer written in the Rust programming language. Alongside from the commands listed [https://lingemu.org/examples.html here], it also has GDB support.

To use the gdb support pass in the argument -gdb xxxx, where "xxxx" is the port number. Lingemu will then wait for you to connect before booting up the operating system.

[[Category:Emulators]]
[[Category:X86]]
````

<a id="w004727"></a>

## W004727 — Mold

Bron-ID: W004727
Namespace: Main (0)
Revisie: 28090 | Datum: 2023-07-09T21:24:24Z
Bron: https://wiki.osdev.org/Mold
Revisie-URL: https://wiki.osdev.org/index.php?title=Mold&oldid=28090

````text
{{stub}}

[https://github.com/rui314/mold Mold] is a new linker written by Rui Ueyama, the creator of the LLVM ld mold tool.

Note it has no support for linker scripts at the time of writing.

[[Category:Linkers]]
````

<a id="w004728"></a>

## W004728 — User:Len

Bron-ID: W004728
Namespace: User (2)
Revisie: 27479 | Datum: 2022-11-03T19:39:33Z
Bron: https://wiki.osdev.org/User:Len
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALen&oldid=27479

````text
Nothing here :)
````

<a id="w004729"></a>

## W004729 — File:ATOS Img2.png

Bron-ID: W004729
Namespace: File (6)
Revisie: 27505 | Datum: 2022-12-01T21:20:52Z
Bron: https://wiki.osdev.org/File:ATOS_Img2.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AATOS_Img2.png&oldid=27505

````text

````

<a id="w004730"></a>

## W004730 — VMWare SVGA-II

Bron-ID: W004730
Namespace: Main (0)
Revisie: 27868 | Datum: 2023-07-09T04:02:56Z
Bron: https://wiki.osdev.org/VMWare_SVGA-II
Revisie-URL: https://wiki.osdev.org/index.php?title=VMWare_SVGA-II&oldid=27868

````text

The VMware SVGA-II device is a graphics card simulated by [[VMware]] and [[QEMU]]. It is a [[PCI]] [[SVGA]]-compatible graphics card that implements 2D and 3D acceleration as well as hardware cursor support.

== Hardware Detection ==
The procedure of detecting its presence is by detecting if there is a PCI device with the vendor ID 0x15AD and the device ID 0x0405 on the bus.

== I/O Ports ==
The base I/O port (SVGA_PORT_BASE) is not fixed. The driver has to substract 1 from the BAR0 in the PCI configuration to get it.
The following port offsets are exposed by the device:
* '''SVGA_INDEX''' (offset 0)
* '''SVGA_VALUE''' (offset 1)
* '''SVGA_BIOS''' (offset 2)
* '''SVGA_IRQSTATUS''' (offset 8)

== Internal Registers ==
Although there are more registers (all shown in the Official Developer Kit linked below), the most important ones are:
* '''SVGA_REG_ID''' (0) - register used to negociate specification ID
* '''SVGA_REG_ENABLE''' (1) - flag set by the driver when the device should enter SVGA mode 
* '''SVGA_REG_WIDTH''' (2) - current screen width
* '''SVGA_REG_HEIGHT''' (3) - current screen height
* '''SVGA_REG_MAX_WIDTH''' (4) - maximum supported screen width
* '''SVGA_REG_MAX_HEIGHT''' (5) - maximum supported screen height
* '''SVGA_REG_BPP''' (7) - current screen bits per pixel
* '''SVGA_REG_FB_START''' (13) - address in system memory of the frame buffer
* '''SVGA_REG_FB_OFFSET''' (14) - offset in the frame buffer to the visible pixel data
* '''SVGA_REG_VRAM_SIZE''' (15) - size of the video RAM
* '''SVGA_REG_FB_SIZE''' (16) - size of the frame buffer
* '''SVGA_REG_CAPABILITIES''' (17) - device capabilities
* '''SVGA_REG_FIFO_START''' (18) - address in system memory of the FIFO
* '''SVGA_REG_FIFO_SIZE''' (19) - FIFO size
* '''SVGA_REG_CONFIG_DONE''' (20) - flag to enable FIFO operation
* '''SVGA_REG_SYNC''' (21) - flag set by the driver to flush FIFO changes
* '''SVGA_REG_BUSY''' (22) - flag set by the FIFO when it's processed

Reading or writing to an internal register is very easy. The driver has to write a long to the port '''SVGA_PORT_BASE''' + '''SVGA_INDEX''' corresponding to the register index then write the data or read it from the port '''SVGA_PORT_BASE''' + '''SVGA_VALUE'''.

== Initialising the Device ==
The procedure of initialisation is:
* Detect the device on the PCI bus (vendor 0x15AD, device 0x0405)
* Enable memory, I/O and bus mastering by ORing the command field (in the PCI configuration) with 0x7; 
* Calculate the I/O base port from substracting 1 from the PCI BAR0.
* Set the '''SVGA_REG_ID''' register to 0x90000002 (indicate latest specification ID)
* Check if it is supported by reading back the register and comparing the result
* Read the frame buffer and FIFO addresses from the registers '''SVGA_REG_FB_START''' and '''SVGA_REG_FIFO_START'''
* Read the frame buffer offset and the capabilities field from the registers '''SVGA_REG_FB_OFFSET''' and '''SVGA_REG_CAPABILITIES'''
* Read the FIFO size from the register '''SVGA_REG_FIFO_SIZE'''
* Identity map the FIFO memory
* Initialise the FIFO (explained in next chapter)
* Write 1 to '''SVGA_REG_ENABLE'''

== Capabilities ==

== Using the FIFO ==
The FIFO is an array of 32-bit elements which is filled by the driver with commands that the device has to execute.

TODO: write something about every command
The supported FIFO commands are:
* '''SVGA_CMD_UPDATE''' (1) -> Update the screen; Operands: startX, startY, width, height
* '''SVGA_CMD_RECT_COPY''' (3) -> Copy a rectangle from one area of the visible frame buffer to another; Operands: srcX, srcY, destX, destY, width, height (Requires '''SVGA_CAP_RECT_COPY''')
* '''SVGA_CMD_DEFINE_CURSOR''' (19)
* '''SVGA_CMD_DEFINE_ALPHA_CURSOR''' (22)
* '''SVGA_CMD_UPDATE_VERBOSE''' (25) -> Just like '''SVGA_CMD_UPDATE''' but adds another operand named reason
* '''SVGA_CMD_FRONT_ROP_FILL''' (29)
* '''SVGA_CMD_FENCE''' (30)
* '''SVGA_CMD_ESCAPE''' (33)
* '''SVGA_CMD_DEFINE_SCREEN'''  (34)
* '''SVGA_CMD_DESTROY_SCREEN''' (35)
* '''SVGA_CMD_DEFINE_GMRFB''' (36)
* '''SVGA_CMD_BLIT_GMRFB_TO_SCREEN''' (37)
* '''SVGA_CMD_BLIT_SCREEN_TO_GMRFB''' (38)
* '''SVGA_CMD_ANNOTATION_FILL''' (39)
* '''SVGA_CMD_ANNOTATION_COPY''' (40)
* '''SVGA_CMD_DEFINE_GMR2''' (41)
* '''SVGA_CMD_REMAP_GMR2''' (42)

To write a command, the driver has to first sequentially write its ID and then the operands to the FIFO.

The FIFO has many registers that the driver can mess with (293). The most important ones are:
* '''SVGA_FIFO_MIN''' (0) - start of command queue
* '''SVGA_FIFO_MAX''' (1) - end of command queue
* '''SVGA_FIFO_NEXT_CMD''' (2) - next command (offset in bytes)
* '''SVGA_FIFO_STOP''' (3) - todo: explain what SVGA_FIFO_STOP does

Writing/reading to/from those is as easy as to write/read data to/from the FIFO address + register index.

To initialise the FIFO, the driver has to:
* Set '''SVGA_FIFO_MIN''', '''SVGA_FIFO_NEXT_CMD''' and '''SVGA_FIFO_STOP''' registers to 293 * 4 (point right after the 293 32-bit registers)
* Set '''SVGA_FIFO_MAX''' to the FIFO size
* Set '''SVGA_REG_CONFIG_DONE''' to 1

To sequentially write data to the FIFO the driver has to:
* Write the data to the FIFO address + value of '''SVGA_FIFO_NEXT_CMD''' (Warning! SVGA_FIFO_NEXT_CMD provides the offset in '''bytes''')
* Increment '''SVGA_FIFO_NEXT_CMD''' by 4

== Setting a new Mode ==
To set a new mode the driver has to:
* Write the new width, height and bits per pixel values to the coresponding registers ('''SVGA_REG_WIDTH''','''SVGA_REG_HEIGHT''','''SVGA_REG_BPP''')
* Read again the frame buffer offset from '''SVGA_REG_FB_OFFSET'''
* Set '''SVGA_REG_ENABLE''' to 1

== 2D Acceleration ==

== 3D Acceleration ==

== Official Developer Kit ==
[https://sourceforge.net/p/vmware-svga/git/ci/master/tree VMware SVGA Device Developer Kit]

[[Category:Video]]
````

<a id="w004731"></a>

## W004731 — User:Abdhdi

Bron-ID: W004731
Namespace: User (2)
Revisie: 27542 | Datum: 2022-12-23T18:11:17Z
Bron: https://wiki.osdev.org/User:Abdhdi
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAbdhdi&oldid=27542

````text
Hi ''''':)'''''.

Github: [https://github.com/Abdulhadi5692HDI2 github]
````

<a id="w004732"></a>

## W004732 — User:Isaccbarker

Bron-ID: W004732
Namespace: User (2)
Revisie: 27682 | Datum: 2023-03-14T22:34:28Z
Bron: https://wiki.osdev.org/User:Isaccbarker
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AIsaccbarker&oldid=27682

````text
Hello! I'm a hobbyist OS developer, currently working on a kernel that can only run bytecode. This could be a bad idea, but I wonder what optimizations I could do with complete control and introspective abilities into program execution.

View my website at http://milobanks.com.
````

<a id="w004733"></a>

## W004733 — User:Klange/Kuroko

Bron-ID: W004733
Namespace: User (2)
Revisie: 27577 | Datum: 2023-01-10T03:51:22Z
Bron: https://wiki.osdev.org/User:Klange/Kuroko
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKlange%2FKuroko&oldid=27577

````text
'''Kuroko''' is a bytecode-compiled interpreted programming language. It is an independent implementation of a dialect of Python with the addition of declaration statements and C-style block scoping.

== History == 

Kuroko was originally developed in late 2020 / early 2021 as an extension language for the Bim code editor.

== Use in Hobby OSes ==

Kuroko is most notably found in [[User:Klange/ToaruOS|ToaruOS]], where it is used as part of the build system, provided as a userspace interpreter, powers a handful of utilities, and is integrated with the Bim text editor.

Kuroko can be built with a very small libc footprint, which makes it suitable as an early port for a new OS. It can also be built to run directly under EFI to run "bare-metal", or compiled to WebAssembly to be hosted in a sandboxed environment.

== Language Features ==

As an implementation of Python, Kuroko has many useful features such as lists, dicts (key-value maps), comprehensions (expressions generating lists and dicts), closures (functions inheriting an environment), lambdas, module imports, "big ints", and mechanisms for asynchronous programming.

Kuroko is garbage collected, with a simple scan-and-sweep collector that does not need to be adapted to new architectures. Kuroko's VM is a straightforward bytecode interpreter with no JIT, and should also be portable to new architectures without additional code changes.

Kuroko's compiler is fast enough that code can be distributed in source form with no need for a bytecode cache. The interpreter also has low cold start costs.

Kuroko can be embedded in host applications as an extension language (both through static linking and as a shared library), and can be extended with C modules to provide wrappers to third-party libraries such as OS-specific windowing interfaces. On platforms without runtime shared library loading, extension modules can be statically built into the interpreter.

== Why would I want Kuroko instead of CPython? ==

If you're looking to run existing Python code, you probably don't want Kuroko. Porting Python code to Kuroko may be possible in the case of code that does not use third-party modules or esoteric metaprogramming tricks, but Kuroko is not intended for this and porting can be a considerable effort of adding variable declarations and tweaking expressions and control flow.

If you are looking to write new code or get an interactive REPL started up early in a new OS, Kuroko may be a viable choice: It is significantly smaller than CPython, while more functional and less reduced than something like MicroPython. It can be built for more restricted environments than CPython, including ones without threading - something that has become a requirement in Python 3.7 and newer.

== Can I write an OS in Kuroko? ==

Probably not without more effort than it is worth. Kuroko is a bytecode-interpreted language, and any interaction with hardware would require additional C code. While Kuroko can be built as an EFI application, extending this to a full OS would be a considerable challenge.
````

<a id="w004734"></a>

## W004734 — User:Rose932

Bron-ID: W004734
Namespace: User (2)
Revisie: 27584 | Datum: 2023-01-13T22:51:57Z
Bron: https://wiki.osdev.org/User:Rose932
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARose932&oldid=27584

````text

== Hi, I'm Rose. ==
I'm a full stack developer working on my Associate's in Computer Programming. I'm working on a basic operating system based off HackNet OS from the PC game HackNet.
````

<a id="w004735"></a>

## W004735 — User:Trinka

Bron-ID: W004735
Namespace: User (2)
Revisie: 27599 | Datum: 2023-01-19T19:44:58Z
Bron: https://wiki.osdev.org/User:Trinka
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATrinka&oldid=27599

````text
I have taken notice of the CC0 license and its intended effects, and hereby declare that all my past and future contributions to the OSDev.org Wiki are licensed under the effect of the CC0.
````

<a id="w004736"></a>

## W004736 — User:Xenobino

Bron-ID: W004736
Namespace: User (2)
Revisie: 27602 | Datum: 2023-01-21T12:09:25Z
Bron: https://wiki.osdev.org/User:Xenobino
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AXenobino&oldid=27602

````text
I am XenoBino (Ahmed Al-Shoqanebi), A novice OS developer. I joined this wiki to enrich it with tutorials and other easy-to-approach content.

I currently study Information Technology in the Collage of Computer Science and Information Technology, [University of Sumer](https://uos.edu.iq/), Iraq.

I am an experienced Full stack web developer of 3 years. On the frontend, I prefer React, Redux, Tailwind.css (or rarely, bootstrap.css). On the backend, I prefer NodeJS and express.js.
````

<a id="w004737"></a>

## W004737 — File:Current.png

Bron-ID: W004737
Namespace: File (6)
Revisie: 27603 | Datum: 2023-01-26T12:48:58Z
Bron: https://wiki.osdev.org/File:Current.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACurrent.png&oldid=27603

````text

````

<a id="w004738"></a>

## W004738 — User:Cyao

Bron-ID: W004738
Namespace: User (2)
Revisie: 27605 | Datum: 2023-01-29T10:04:35Z
Bron: https://wiki.osdev.org/User:Cyao
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACyao&oldid=27605

````text
My current project is https://github.com/cheyao/Achieve-Core/, a risc-v SoC
````

<a id="w004739"></a>

## W004739 — Talk:D Bare Bones With LDC2

Bron-ID: W004739
Namespace: Talk (1)
Revisie: 27607 | Datum: 2023-01-30T04:12:46Z
Bron: https://wiki.osdev.org/Talk:D_Bare_Bones_With_LDC2
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AD_Bare_Bones_With_LDC2&oldid=27607

````text
Where is linker.ld? [[User:Leapofazzam|Leapofazzam]] 22:12, 29 January 2023 (CST)
````

<a id="w004740"></a>

## W004740 — Apple TV 1st Generation

Bron-ID: W004740
Namespace: Main (0)
Revisie: 28468 | Datum: 2023-11-16T15:29:03Z
Bron: https://wiki.osdev.org/Apple_TV_1st_Generation
Revisie-URL: https://wiki.osdev.org/index.php?title=Apple_TV_1st_Generation&oldid=28468

````text
{{Stub}}

The Apple TV 1st Generation is a smart TV box released by Apple in 2007. It contains a 1GHz Intel processor identical in specifications to the Pentium M ULV 723, 256MB of soldered DDR2 RAM, a NVIDIA GeForce Go 7300 [[GPU|graphics card]], and an IDE hard drive.

==Boot Process and Quirks==

Like Intel Macs, the Apple TV 1st Generation uses [[UEFI|EFI]] firmware; however, unlike Intel Macs, the Apple TV can only boot from a single boot.efi, located at either /System/Library/CoreServices/boot.efi or /boot.efi on an HFS-formatted disk.
````

<a id="w004742"></a>

## W004742 — User:Techflash

Bron-ID: W004742
Namespace: User (2)
Revisie: 27649 | Datum: 2023-03-01T23:51:03Z
Bron: https://wiki.osdev.org/User:Techflash
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATechflash&oldid=27649

````text
Hi, I'm Techflash. &nbsp;I'm a hobbyist programmer from Arizona.

You can see [https://github.com/techflashYT/Techflash-OS my OS here]
````

<a id="w004743"></a>

## W004743 — File:Fig15-3.gif

Bron-ID: W004743
Namespace: File (6)
Revisie: 27657 | Datum: 2023-03-06T12:53:21Z
Bron: https://wiki.osdev.org/File:Fig15-3.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AFig15-3.gif&oldid=27657

````text
Stack after exiting V86 mode in x86.
````

<a id="w004744"></a>

## W004744 — User:Isaccbarker/String Formatting

Bron-ID: W004744
Namespace: User (2)
Revisie: 27679 | Datum: 2023-03-14T06:01:21Z
Bron: https://wiki.osdev.org/User:Isaccbarker/String_Formatting
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AIsaccbarker%2FString_Formatting&oldid=27679

````text
Here is an example implementation of some code which formats text according to a format string, and flushes it via a function pointer given in a struct. This code is taken from HenipaOS, and is thus licensed under SPDX: BSD-3-Clause-No-Nuclear-License.

It does not seek to replicate the format seen in libc, but instead formats with a format designed for simple code (no look ahead required, not much logic, easily extensible).

<source lang="c">
typedef int (*pfnStreamWriteBuf)(char*);

enum ParseMode { NORMAL, ARGUMENT, FORMAT_SPECIFIER };

struct Stream {
    size_t buf_len;
    size_t buf_i;
    char *buf;
    pfnStreamWriteBuf pfn_write_all;
};

// Returns 0 on success, non-0 on failure.
static int print_stream_buf(struct Stream *stream) {
  // Ensure we are null terminated. Can never be too safe!
  stream->buf[stream->buf_len - 1] = '\0';

  // Write the buf.
  if ((size_t)stream->pfn_write_all(stream->buf) != strlen(stream->buf)) {
    // We didn't write the entire usable portion of the buffer, for some reason.
    return 1;
  }

  // Clear the buffer.
  memset(stream->buf, 0, stream->buf_len);

  return 0;
}

// If not enough space is avilable, call `print_stream_buf`, and forward the
// return code.
static int push_to_buf(struct Stream *stream, char c) {
  stream->buf[stream->buf_i] = c;
  stream->buf_i += 1;

  // Check if the buffer is full. If so, flush it.
  if (stream->buf_i == stream->buf_len - 2) {
    int err;
    stream->buf_i = 0;

    err = print_stream_buf(stream);
    if (err != 0) {
      // We didn't write the entire buffer, for some reason.
      return err;
    }
  }

  return 0;
}

// If too little space is in the buffer either at the end of pushing the string
// or at the end, it will call `print_stream_buf`. If that fails, it will
// forward its return code.
//
// As such, returns 0 on success, non-0 on failure.
static int push_all_to_buf(struct Stream *stream, char *str) {
  int err;

  for (char c = *str; c != '\0'; c = *++str) {
    err = push_to_buf(stream, c);

    if (err != 0) {
      return err;
    }
  }

  return 0;
}

// To print a signed int this way, first detect if it is negative and then convert it into an
// unsigned type.
static void push_int_to_buf(struct Stream *stream, unsigned int val, unsigned int base) {
    int i = MAXBUF-2;
    static char buf[MAXBUF] = {0};

    for (; val && i; --i, val /= base)
        buf[i] = "0123456789abcdefghijklmnopqrstuvwxyz"[val % base];

    push_all_to_buf(stream, &buf[i+1]);
}

static int int_display_flag_to_base(char flag) {
    switch (flag) {
    case 'd':
        return 10;
    case 'x':
        return 16;
    case 'o':
        return 8;
    default:
        return 10;
    }
}

// The format specifier should follow the below template:
//  {?<arg>.<format code>}
//  note: only include the '?' before the arg if using an arg.
//
// Printing signed number:
//  {?<base>d}
//  note: base can be 'd' for decimal, 'x' for hexidecimal, or 'o' for octal.
//
// Printing unsigned number:
//  {?<base>u}
//  note: base can be 'd' for decimal, 'x' for hexidecimal, or 'o' for octal.
//
// Printing string:
//  {s}
int vfprintf(struct Stream *stream, const char *fmt, va_list args) {
    enum ParseMode parse_mode = NORMAL;
    char arg = '\0';

    for (size_t i = 0; i < strlen(fmt); i++) {
        char cur = fmt[i];

        switch (cur) {
        case '{': {
            if (parse_mode == NORMAL) {
                // Start looking for arguments next, or another '{' for escape.
                parse_mode = FORMAT_SPECIFIER;
                goto char_loop_end;
            } else {
                // We escaped the '{' character.
                parse_mode = NORMAL;
            }

            break;
        }

        case '}': {
            if (parse_mode == FORMAT_SPECIFIER) {
                parse_mode = NORMAL;
                goto char_loop_end;
            }

            break;
        }

        case '?': {
            if (parse_mode == FORMAT_SPECIFIER) {
                parse_mode = ARGUMENT;
                goto char_loop_end;
            }

            break;
        }

        case 'd': {
            if (parse_mode == FORMAT_SPECIFIER) {
                int val = va_arg(args, int);

                if (val < 0)
                    push_to_buf(stream, '-');

                push_int_to_buf(stream, (unsigned int) val, int_display_flag_to_base(arg));

                goto format_spec_end;
            }

            break;
        }

        case 'u': {
            if (parse_mode == FORMAT_SPECIFIER) {
                unsigned int val = va_arg(args, unsigned int);
                push_int_to_buf(stream, val, int_display_flag_to_base(arg));

                goto format_spec_end;
            }

            break;
        }

        case 's': {
            if (parse_mode == FORMAT_SPECIFIER) {
                char *str = va_arg(args, char *);
                push_all_to_buf(stream, str);

                goto format_spec_end;
            }

            break;
        }

        default: {
            if (parse_mode == ARGUMENT) {
                arg = cur;
                parse_mode = FORMAT_SPECIFIER;

                goto char_loop_end;
            }

            break;
        }
        }

        // Add the current character to the buffer.
        push_to_buf(stream, cur);

        goto char_loop_end;

format_spec_end:
        arg = '\0';

char_loop_end:
        continue;
    }

    // At the end, flush the buffer.
    if (print_stream_buf(stream) != 0) {
        // We didn't write the entire buffer, for some reason.
        // TODO: Decide what to do if we ever get here.
    }

    // And of course reset the buffer index.
    stream->buf_i = 0;

    return 0;
}
</source>
````

<a id="w004745"></a>

## W004745 — USB Hubs

Bron-ID: W004745
Namespace: Main (0)
Revisie: 27884 | Datum: 2023-07-09T04:29:59Z
Bron: https://wiki.osdev.org/USB_Hubs
Revisie-URL: https://wiki.osdev.org/index.php?title=USB_Hubs&oldid=27884

````text
{{In Progress}}
[[Image:USBSystem.gif|frame|Figure 2: USB System Illustration]]
'''USB hubs''' are devices that attach multiple USB devices to one port - be it a port on the host controller, or a port on another hub.

__TOC__

== Introduction ==
USB hubs are intimidating to start with - however, as devices, they are in fact quite simple. USB hubs are hubs in the same sense as Ethernet hubs are - they are neither switches nor routers. They operate at the physical layer by connecting the data lines together.

In practice, things are slightly more complex than this, as there are certain tasks that USB hubs must perform. But it's a good conceptual model to begin with.

'''USB 3''' hubs are best treated conceptually as two different hubs in one box - a USB 2 hub, and a SuperSpeed hub. Similarly to [[xHCI]], a USB device is connected to the appropriate logical port for its speed. Note that the hub then appears on both the USB 2 protocol port on its host, and the USB 3 protocol port!

However, if a USB 3 hub is connected to a USB 2 port at any point in the chain back to the host, it will then only operate as USB 2.

The USB 3 hub device is very similar to USB 2, so this device is best described by explaining its differences.

== USB 2 hub ==
A USB hub is a device with two endpoints - the default control endpoint, and the Status change endpoint.

The main responsibility of the hub device is managing the status of its downstream ports.

The first thing you need to do when you've detected a hub is to configure it. Once you have the configuration descriptor as with a normal USB device, you need to set the configuration. [[xHCI]] can get a little interesting, so I'll describe that.

To set the device configuration on xHCI, you first need to issue an '''Configure Endpoint''' TRB. This needs to have the add context flags set to 1 | (EndpointFlags). EndpointFlags will likely be (1<<4), representing Endpoint Context 1 IN. However, this should be set according to the endpoint descriptor from the device.

Bit zero is set because we need to update the slot context. The Context Entries field needs to be set to the ''index'' of the last endpoint (not necessarily the number of entries). And, as this is a hub, the Hub fields need to be initialised. This means setting the Hub bit to 1 and setting the '''Number of Ports'''.

This of course raises the question - how do we know the Number of Ports!?

Well, handily, that's contained in the hub descriptor. Less handily, per the USB 2 spec, 
''"If the hub is not configured, the hub’s response to this request is undefined."''

In practice, USB 2 hubs tend to give you the hub descriptor if you request it. But the correct way of dealing with this is to initialise the Number of Ports to a sensible default (typically 4), then reconfigure the endpoint when you  get the descriptor. This resembles the procedure with MaxPacketSize on full speed devices.

=== Hub Descriptor ===
To get the descriptor (per spec once the device is configured), issue a GET_DESCRIPTOR request, bmRequestType=10100000b, as this is a class descriptor.

wValue is set with a descriptor type of 0x29.

{| align="center" border="1"
!Offset
!Field
!Size
!Type
!Description
|- valign="top"
|align="center" |0
|bLength
|align="center" |1
|align="center" |Number
|Size of this descriptor in bytes
|- valign="top"
|align="center" |1
|bDescriptorType
|align="center" |1
|align="center" |Constant
|HUB Descriptor Type 0x29
|- valign="top"
|align="center" |2
|bNbrPorts
|align="center" |1
|align="center" |Number
|Number of ports
|- valign="top"
|align="center" |3
|wHubCharacteristics
|align="center" |2
|align="center" |Bitmap
|Hub Characteristics
|- valign="top"
|align="center" |5
|bPowerOnGood
|align="center" |1
|align="center" |Number
|Time (*2 ms) from port power on to power good
|- valign="top"
|align="center" |6
|bHubContrCurrent
|align="center" |1
|align="center" |Number
|Maximum current used by hub controller (mA).
|- valign="top"
|align="center" |7
|DeviceRemovable
|align="center" |bNbrPorts bits
|align="center" |Bitmap
|Is port device removable
|- valign="top"
|align="center" |After DeviceRemovable
|PortPwrCtrlMask
|align="center" |bNbrPorts bits
|align="center" |Bitmap
|Should be all ones for anything above USB 1.0
|}
=== Hub Command Set === 
{|border="1" cellpadding="2" style="float:right;clear:right;margin-left:20px;"
|+align="bottom" |'''Hub Class Request Codes'''
!bRequest
!Value
|-
|GET_STATUS
|align="center" |0
|-
|CLEAR_FEATURE
|align="center" |1
|-
|RESERVED
|align="center" |2
|-
|SET_FEATURE
|align="center" |3
|-
|GET_DESCRIPTOR
|align="center" |6
|-
|SET_DESCRIPTOR
|align="center" |7
|-
|CLEAR_TT_BUFFER
|align="center" |8
|-
|RESET_TT
|align="center" |9
|-
|GET_TT_STATE
|align="center" |10
|-
|CSTOP_TT
|align="center" |11
|}
All hub commands have a type of 01b = Class. The recipient is normally 00000b = Device, but '''Port''' or '''TT''' commands target 00011b = Other.


CLEAR_FEATURE, GET_STATUS, and SET_FEATURE can target either the hub or a port.

== See Also ==
*[[USB]]

[[Category:USB]]
[[Category:Hardware]]
````

<a id="w004746"></a>

## W004746 — Git

Bron-ID: W004746
Namespace: Main (0)
Revisie: 30354 | Datum: 2026-03-18T19:40:10Z
Bron: https://wiki.osdev.org/Git
Revisie-URL: https://wiki.osdev.org/index.php?title=Git&oldid=30354

````text
{{In_Progress}}

'''Git''' is a [[Version Control System|version control system]] (VCS) made by Linus Torvalds in 2005 for use with the Linux kernel. It is one of the most well known and popular among version control systems, mainly due to influential Git hosting websites such as GitHub and others.

== Cheatsheet ==
Here's a quick cheatsheet for the most common operations (80%) with Git.

=== Cloning a remote repository ===
<code>git clone repo_address</code>

=== Initializing a new repository ===
This will create an empty local Git repository at the current directory:
<code>git init</code>

=== Committing ===
First the changes must be staged:

<code>git add changed-file-name</code>

Then a commit can be created with a message specified by the <code>-m</code> flag.

<code>git commit -m "Update"</code>

Alternatively, the <code>-m</code> flag can be left out, in which case Git will attempt to open the default set editor to edit the commit message.

As a convenience, <code>git commit</code> provides the <code>-a</code> flag which stages all changes made in the repository and commits them
in one go. The main use for this is when a new repository is initialized and the files are added. Otherwise it is considered to not be good practice, since it is
good to keep commits atomic and self-contained.

=== Selecting hunks (changes) to commit ===
It is also possible to select just specific changes made in file(s), instead of staging the whole file(s):

<code>git add -p</code>

Optionally, a specific file can be specified by including the path to it right after the <code>-p</code> flag.

=== Amending commits ===
If there's an issue with the last made commit (forgot to commit a file, wrong file committed, wrong commit message...), it can be easily amended:

<code>git commit --amend</code>

That command will first: include any additional change now present in the staging area, and second: open the editor with the original commit message for editing. If the commit message is already OK, the <code>--no-edit</code> flag can be used to prevent editing the message.

Note that amending is a form of history-rewriting, which means that it will by definition collide with the remote (in the case the amended commit was already pushed into the remote). In such case, see the next section.

=== Pushing to remote repository ===
After committing, changes made locally can be pushed to the remote repository:

<code>git push</code>

If the history was rewritten, and the touched commits are already in the remote repository, a force push is necessary. First, to play it safe a "force push with lease" is worth attempting (specially if the rewrite is something simple like a commit amendment):

<code>git push --force-with-lease</code>

Otherwise just proceed with a forceful push:

<code>git push -f</code>

=== Updating the local repository from the remote ===
The quickest way is to do a "pull":

<code>git pull</code>

This is equivalent to doing these two commands manually:
<syntaxhighlight lang="bash">
git fetch
git merge
</syntaxhighlight>

If you have uncommitted changes and the remote has also made changes on those files, the merge will fail by default, since otherwise the uncommitted local changes would be destroyed by the merge. See the next section on how to solve this.

=== Stashing ===
Stashing allows to save uncommitted changes in a place called "stash". The stash has stack-like semantics. Changes may be pushed to the stash so they can be either popped, meaning the changes are restored into the files (but not staged or committed) or dropped (deleted).

Stash all uncommitted changes:

<code>git stash</code>

Stash specific hunks (optionally accepts a file path):

<code>git stash -p</code>

Stash with a message:

<code>git stash -m 'stashed change description'</code>

List all stashes:

<code>git stash list</code>

Remove a stash from the top of the stack and apply the changes to the files:

<code>git stash pop</code>

Apply the stash from the top, but don't drop it:

<code>git stash apply</code>

Drop the stash at the top of the stack:

<code>git stash drop</code>

=== Removing files ===
Removes the file from Git tracking and from the file system:

<code>git rm file-to-remove</code>

Remove the file from Git tracking, but keep it in the file system:

<code>git rm --cached file-to-remove</code>

File removal operations are staged, so these have to be committed.

=== Seeing changes ===
List all uncommitted, unstaged changes made in the repo:

<code>git diff</code>

List all uncommitted, '''staged''' changes made:

<code>git diff --cached</code>

=== Repository history ===
Show all the commit history:

<code>git log</code>

Show commit history of file or directory at path:

<code>git log path</code>

Shows who made what change in a file:

<code>git blame file-to-check</code>

=== Branches ===
List all branches in local repository:

<code>git branch</code>

Create a new branch and switch to it:

<code>git checkout -b branch-name</code>

Bring changes made from another branch to the current one:

<code>git checkout branch-name -- path</code>

Push new branch to remote repository:

<code>git push origin branch-name</code>

=== Patching ===
Apply a patch previously made:

<code>git apply diff-file</code>

== Documentation ==
Unix-like systems with Git installed should provide manual pages for Git. The name of the manpage for a specific Git subcommand follow the <code>git-subcmd</code> convention: e.g.: for the branch subcomand, <code>git-branch</code>, so the full command would be <code>man git-branch</code>.

== See Also ==
=== External Links ===
* [https://en.wikipedia.org/wiki/Git Wikipedia Article on Git]
* [https://git-scm.com/docs Git reference (man pages) online]

[[Category:Tools]]
[[Category:Version Control Systems]]
````

<a id="w004747"></a>

## W004747 — User:Kraskaska

Bron-ID: W004747
Namespace: User (2)
Revisie: 27755 | Datum: 2023-04-27T09:54:05Z
Bron: https://wiki.osdev.org/User:Kraskaska
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKraskaska&oldid=27755
Doorverwijzing: Expanded Main Page

````text
#REDIRECT [[Expanded_Main_Page]]

You still here? You don't need my page yet.
````

<a id="w004748"></a>

## W004748 — User:Saper

Bron-ID: W004748
Namespace: User (2)
Revisie: 27757 | Datum: 2023-04-29T17:45:14Z
Bron: https://wiki.osdev.org/User:Saper
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASaper&oldid=27757

````text
[https://bsd.network/@saper saper at bsd.network on Mastodon]
````

<a id="w004749"></a>

## W004749 — File:DuckOS.png

Bron-ID: W004749
Namespace: File (6)
Revisie: 27767 | Datum: 2023-05-16T19:20:33Z
Bron: https://wiki.osdev.org/File:DuckOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ADuckOS.png&oldid=27767

````text
A screenshot of duckOS.
````

<a id="w004750"></a>

## W004750 — Talk:QEMU AArch64 Virt Bare Bones

Bron-ID: W004750
Namespace: Talk (1)
Revisie: 27785 | Datum: 2023-06-02T02:50:22Z
Bron: https://wiki.osdev.org/Talk:QEMU_AArch64_Virt_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AQEMU_AArch64_Virt_Bare_Bones&oldid=27785

````text
Previously, this article suggested linking at 0x40000000. This is the start of RAM on all "virt" targets. It's also where QEMU wants to put the DTB, and if you've already asked to load a kernel there then QEMU will skip the DTB, leaving you unable to discover hardware - or your own command line options! Loading away from the DTB resolves this, and 0x40100000 was chosen based on some investigation of the expected maximum size of a DTB. --[[User:Klange|klange]] 21:50, 1 June 2023 (CDT)
````

<a id="w004751"></a>

## W004751 — User:Auxilus

Bron-ID: W004751
Namespace: User (2)
Revisie: 27796 | Datum: 2023-06-15T16:25:00Z
Bron: https://wiki.osdev.org/User:Auxilus
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAuxilus&oldid=27796

````text
Auxilus
````

<a id="w004752"></a>

## W004752 — User talk:Auxilus

Bron-ID: W004752
Namespace: User talk (3)
Revisie: 27797 | Datum: 2023-06-15T16:25:14Z
Bron: https://wiki.osdev.org/User_talk:Auxilus
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AAuxilus&oldid=27797

````text
Auxilus
````

<a id="w004753"></a>

## W004753 — File:Edge vs level.png

Bron-ID: W004753
Namespace: File (6)
Revisie: 27798 | Datum: 2023-06-16T01:20:12Z
Bron: https://wiki.osdev.org/File:Edge_vs_level.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AEdge_vs_level.png&oldid=27798

````text

````

<a id="w004754"></a>

## W004754 — User:Quadrantal/difficulty icon test template

Bron-ID: W004754
Namespace: User (2)
Revisie: 27828 | Datum: 2023-06-24T18:32:19Z
Bron: https://wiki.osdev.org/User:Quadrantal/difficulty_icon_test_template
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AQuadrantal%2Fdifficulty_icon_test_template&oldid=27828

````text
{{{2|[[Category:Level {{{1|{{{level|0}}}}}} Tutorials]]}}}<table style="font-size:95%; line-height:1.5em; padding:0.25em; float:right; margin: 0 0 8px 15px; clear:right; border:1px solid #aaaaaa; background:#eee; text-align:center;{{{2|}}};"><!--

--><tr><th>Difficulty level</th></tr><tr><td>[[Image:Difficulty {{{1|{{{level|0}}}}}}.png|46px]]<br />{{If|{{Eq|{{{1|{{{level|0}}}}}}|0}}|Not rated|{{If|{{Eq|{{{1|{{{level|0}}}}}}|1}}|Beginner|{{If|{{Eq|{{{1|{{{level|0}}}}}}|2}}|Medium|{{If|{{Eq|{{{1|{{{level|0}}}}}}|3}}|Advanced|Master}}}}}}}}</td></tr><!--

--></table><noinclude>
Usage:
Use the following levels:
* 0 or not specified: Not rated
* 1: Beginner
* 2: Medium
* 3: Advanced
* 4: Master
Add an empty second parameter to prevent the article from being added to the category.
</noinclude>
````

<a id="w004755"></a>

## W004755 — File:5-Difficulty 1.png

Bron-ID: W004755
Namespace: File (6)
Revisie: 27829 | Datum: 2023-06-24T18:36:24Z
Bron: https://wiki.osdev.org/File:5-Difficulty_1.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3A5-Difficulty_1.png&oldid=27829

````text
A test version of the level 1 difficulty icon, for testing before updating the actual image. May be deleted when unneeded.
````

<a id="w004756"></a>

## W004756 — File:SerenityOS.png

Bron-ID: W004756
Namespace: File (6)
Revisie: 27849 | Datum: 2023-07-08T17:35:42Z
Bron: https://wiki.osdev.org/File:SerenityOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASerenityOS.png&oldid=27849

````text

````

<a id="w004757"></a>

## W004757 — File:ReactOS.png

Bron-ID: W004757
Namespace: File (6)
Revisie: 27851 | Datum: 2023-07-08T17:42:43Z
Bron: https://wiki.osdev.org/File:ReactOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AReactOS.png&oldid=27851

````text

````

<a id="w004758"></a>

## W004758 — File:Minix.jpg

Bron-ID: W004758
Namespace: File (6)
Revisie: 27852 | Datum: 2023-07-08T17:53:06Z
Bron: https://wiki.osdev.org/File:Minix.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AMinix.jpg&oldid=27852

````text

````

<a id="w004759"></a>

## W004759 — File:HelenOS.png

Bron-ID: W004759
Namespace: File (6)
Revisie: 27853 | Datum: 2023-07-08T17:56:12Z
Bron: https://wiki.osdev.org/File:HelenOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AHelenOS.png&oldid=27853

````text

````

<a id="w004760"></a>

## W004760 — File:Aros.jpg

Bron-ID: W004760
Namespace: File (6)
Revisie: 27854 | Datum: 2023-07-08T17:58:18Z
Bron: https://wiki.osdev.org/File:Aros.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAros.jpg&oldid=27854

````text

````

<a id="w004761"></a>

## W004761 — File:Access2.png

Bron-ID: W004761
Namespace: File (6)
Revisie: 27859 | Datum: 2023-07-09T02:47:14Z
Bron: https://wiki.osdev.org/File:Access2.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAccess2.png&oldid=27859

````text

````

<a id="w004763"></a>

## W004763 — Category:QEMU

Bron-ID: W004763
Namespace: Category (14)
Revisie: 27876 | Datum: 2023-07-09T04:19:20Z
Bron: https://wiki.osdev.org/Category:QEMU
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AQEMU&oldid=27876

````text
[[Category:Emulators]]
````

<a id="w004764"></a>

## W004764 — Category:Disks

Bron-ID: W004764
Namespace: Category (14)
Revisie: 27886 | Datum: 2023-07-09T04:36:34Z
Bron: https://wiki.osdev.org/Category:Disks
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ADisks&oldid=27886

````text
Articles related to disks and partitioning.
````

<a id="w004765"></a>

## W004765 — Category:Loaders

Bron-ID: W004765
Namespace: Category (14)
Revisie: 27888 | Datum: 2023-07-09T04:37:59Z
Bron: https://wiki.osdev.org/Category:Loaders
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ALoaders&oldid=27888

````text
Articles related to loaders, the component of an operating system responsible for loading executables and shared libraries into memory.
````

<a id="w004766"></a>

## W004766 — Category:Multitasking

Bron-ID: W004766
Namespace: Category (14)
Revisie: 27889 | Datum: 2023-07-09T04:38:33Z
Bron: https://wiki.osdev.org/Category:Multitasking
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AMultitasking&oldid=27889

````text
Articles related to multitasking, task/context switching, scheduling, etc.
````

<a id="w004767"></a>

## W004767 — Category:PCI

Bron-ID: W004767
Namespace: Category (14)
Revisie: 30431 | Datum: 2026-03-27T19:57:11Z
Bron: https://wiki.osdev.org/Category:PCI
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APCI&oldid=30431

````text
Articles related to the PCI bus standard, including PCI Express.

[[Category:Buses]]
````

<a id="w004768"></a>

## W004768 — Category:Paging

Bron-ID: W004768
Namespace: Category (14)
Revisie: 27891 | Datum: 2023-07-09T04:40:13Z
Bron: https://wiki.osdev.org/Category:Paging
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APaging&oldid=27891

````text
Articles related to memory paging and virtual memory.
````

<a id="w004769"></a>

## W004769 — Category:Booting

Bron-ID: W004769
Namespace: Category (14)
Revisie: 27899 | Datum: 2023-07-09T04:50:07Z
Bron: https://wiki.osdev.org/Category:Booting
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ABooting&oldid=27899

````text
Articles related to booting an operating system.
````

<a id="w004770"></a>

## W004770 — Category:S390

Bron-ID: W004770
Namespace: Category (14)
Revisie: 30165 | Datum: 2026-03-09T08:56:53Z
Bron: https://wiki.osdev.org/Category:S390
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AS390&oldid=30165

````text
Articles related to the IBM System/390 mainframe
[[Category:Platforms]]
````

<a id="w004771"></a>

## W004771 — Category:Debugging

Bron-ID: W004771
Namespace: Category (14)
Revisie: 27908 | Datum: 2023-07-09T05:03:20Z
Bron: https://wiki.osdev.org/Category:Debugging
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ADebugging&oldid=27908

````text
Articles related to debugging, debuggers, and debugging tools/techniques.
````

<a id="w004772"></a>

## W004772 — Category:Instruction Set Architecture

Bron-ID: W004772
Namespace: Category (14)
Revisie: 27921 | Datum: 2023-07-09T14:51:52Z
Bron: https://wiki.osdev.org/Category:Instruction_Set_Architecture
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AInstruction_Set_Architecture&oldid=27921

````text
Articles related to Instruction Set Architectures (ISA), such as x86, ARM, RISC-V, etc.
````

<a id="w004774"></a>

## W004774 — Category:Hardware Interfaces

Bron-ID: W004774
Namespace: Category (14)
Revisie: 27925 | Datum: 2023-07-09T14:55:44Z
Bron: https://wiki.osdev.org/Category:Hardware_Interfaces
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AHardware_Interfaces&oldid=27925

````text
Articles related to hardware interfaces, such as IDE/ATA, SCSI, etc.
````

<a id="w004775"></a>

## W004775 — Category:Memory Segmentation

Bron-ID: W004775
Namespace: Category (14)
Revisie: 27937 | Datum: 2023-07-09T15:09:18Z
Bron: https://wiki.osdev.org/Category:Memory_Segmentation
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AMemory_Segmentation&oldid=27937

````text
Articles related to memory segmentation.
````

<a id="w004776"></a>

## W004776 — Category:Hardware Detection

Bron-ID: W004776
Namespace: Category (14)
Revisie: 29978 | Datum: 2026-02-11T17:00:57Z
Bron: https://wiki.osdev.org/Category:Hardware_Detection
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AHardware_Detection&oldid=29978

````text
Articles related to hardware detection, including memory, buses, I/O devices, etc.
````

<a id="w004777"></a>

## W004777 — Category:Standard Libraries

Bron-ID: W004777
Namespace: Category (14)
Revisie: 27961 | Datum: 2023-07-09T16:16:38Z
Bron: https://wiki.osdev.org/Category:Standard_Libraries
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AStandard_Libraries&oldid=27961

````text
Articles related to standard libraries and porting them to an OS.
````

<a id="w004778"></a>

## W004778 — Category:Timers

Bron-ID: W004778
Namespace: Category (14)
Revisie: 27963 | Datum: 2023-07-09T16:17:59Z
Bron: https://wiki.osdev.org/Category:Timers
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ATimers&oldid=27963

````text
Articles related to timer devices, timer interrupts, and their programming.
````

<a id="w004779"></a>

## W004779 — Category:Testing

Bron-ID: W004779
Namespace: Category (14)
Revisie: 27985 | Datum: 2023-07-09T17:28:13Z
Bron: https://wiki.osdev.org/Category:Testing
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ATesting&oldid=27985

````text
Articles related to testing and verification.
````

<a id="w004780"></a>

## W004780 — Category:File Formats

Bron-ID: W004780
Namespace: Category (14)
Revisie: 27987 | Datum: 2023-07-09T17:30:00Z
Bron: https://wiki.osdev.org/Category:File_Formats
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AFile_Formats&oldid=27987

````text
Articles related to file formats, specifications, parsing, and generation.
````

<a id="w004781"></a>

## W004781 — Batch

Bron-ID: W004781
Namespace: Main (0)
Revisie: 27993 | Datum: 2023-07-09T17:36:44Z
Bron: https://wiki.osdev.org/Batch
Revisie-URL: https://wiki.osdev.org/index.php?title=Batch&oldid=27993
Doorverwijzing: Batch File

````text
#REDIRECT [[Batch File]]
````

<a id="w004782"></a>

## W004782 — Category:Build Tools

Bron-ID: W004782
Namespace: Category (14)
Revisie: 30162 | Datum: 2026-03-09T08:52:22Z
Bron: https://wiki.osdev.org/Category:Build_Tools
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ABuild_Tools&oldid=30162

````text
Articles related to build tools, e.g. Make, CMake, etc.
[[Category:Tools]]
````

<a id="w004783"></a>

## W004783 — Category:Printing

Bron-ID: W004783
Namespace: Category (14)
Revisie: 28017 | Datum: 2023-07-09T18:02:04Z
Bron: https://wiki.osdev.org/Category:Printing
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APrinting&oldid=28017

````text
Articles related to printing, printer drivers, printer interfaces, printer job languages, etc.
````

<a id="w004784"></a>

## W004784 — Category:Windows

Bron-ID: W004784
Namespace: Category (14)
Revisie: 28020 | Datum: 2023-07-09T18:05:47Z
Bron: https://wiki.osdev.org/Category:Windows
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AWindows&oldid=28020

````text
Articles related to Microsoft Windows, including its architecture, components, and compilers/tools that run on Windows.
````

<a id="w004785"></a>

## W004785 — Category:Compatibility Layers

Bron-ID: W004785
Namespace: Category (14)
Revisie: 30174 | Datum: 2026-03-09T16:44:16Z
Bron: https://wiki.osdev.org/Category:Compatibility_Layers
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ACompatibility_Layers&oldid=30174

````text
Articles about compatibility layers, e.g. POSIX compatibility.
````

<a id="w004786"></a>

## W004786 — Qemu Monitor

Bron-ID: W004786
Namespace: Main (0)
Revisie: 28056 | Datum: 2023-07-09T20:20:06Z
Bron: https://wiki.osdev.org/Qemu_Monitor
Revisie-URL: https://wiki.osdev.org/index.php?title=Qemu_Monitor&oldid=28056
Doorverwijzing: QEMU Monitor

````text
#REDIRECT [[QEMU Monitor]]
````

<a id="w004787"></a>

## W004787 — Talk:Qemu Monitor

Bron-ID: W004787
Namespace: Talk (1)
Revisie: 28058 | Datum: 2023-07-09T20:20:06Z
Bron: https://wiki.osdev.org/Talk:Qemu_Monitor
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AQemu_Monitor&oldid=28058
Doorverwijzing: Talk:QEMU Monitor

````text
#REDIRECT [[Talk:QEMU Monitor]]
````

<a id="w004788"></a>

## W004788 — Category:Fonts

Bron-ID: W004788
Namespace: Category (14)
Revisie: 28069 | Datum: 2023-07-09T20:41:38Z
Bron: https://wiki.osdev.org/Category:Fonts
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AFonts&oldid=28069

````text
Articles related to fonts and font rendering.
````

<a id="w004789"></a>

## W004789 — Category:Compiler Development

Bron-ID: W004789
Namespace: Category (14)
Revisie: 28075 | Datum: 2023-07-09T20:47:37Z
Bron: https://wiki.osdev.org/Category:Compiler_Development
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ACompiler_Development&oldid=28075

````text
Articles related to compiler and language development.
````

<a id="w004790"></a>

## W004790 — Category:Programming

Bron-ID: W004790
Namespace: Category (14)
Revisie: 28104 | Datum: 2023-07-09T22:38:36Z
Bron: https://wiki.osdev.org/Category:Programming
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AProgramming&oldid=28104

````text
Articles related to programming in general.
````

<a id="w004791"></a>

## W004791 — Category:Linux

Bron-ID: W004791
Namespace: Category (14)
Revisie: 30163 | Datum: 2026-03-09T08:54:14Z
Bron: https://wiki.osdev.org/Category:Linux
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ALinux&oldid=30163

````text
Articles related to the Linux kernel.
[[Category:Operating Systems]]
````

<a id="w004792"></a>

## W004792 — Category:Operating Systems

Bron-ID: W004792
Namespace: Category (14)
Revisie: 28109 | Datum: 2023-07-09T22:41:45Z
Bron: https://wiki.osdev.org/Category:Operating_Systems
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AOperating_Systems&oldid=28109

````text
Articles about actual operating systems available for studying.
````

<a id="w004793"></a>

## W004793 — APIC timer

Bron-ID: W004793
Namespace: Main (0)
Revisie: 28147 | Datum: 2023-07-10T14:11:19Z
Bron: https://wiki.osdev.org/APIC_timer
Revisie-URL: https://wiki.osdev.org/index.php?title=APIC_timer&oldid=28147
Doorverwijzing: APIC Timer

````text
#REDIRECT [[APIC Timer]]
````

<a id="w004794"></a>

## W004794 — Talk:APIC timer

Bron-ID: W004794
Namespace: Talk (1)
Revisie: 28149 | Datum: 2023-07-10T14:11:19Z
Bron: https://wiki.osdev.org/Talk:APIC_timer
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AAPIC_timer&oldid=28149
Doorverwijzing: Talk:APIC Timer

````text
#REDIRECT [[Talk:APIC Timer]]
````

<a id="w004795"></a>

## W004795 — Category:Scheduling

Bron-ID: W004795
Namespace: Category (14)
Revisie: 28156 | Datum: 2023-07-10T14:36:54Z
Bron: https://wiki.osdev.org/Category:Scheduling
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AScheduling&oldid=28156

````text
Articles about scheduling process/threads, scheduling algorithms and data structures, etc.
````

<a id="w004796"></a>

## W004796 — Category:Porting

Bron-ID: W004796
Namespace: Category (14)
Revisie: 28166 | Datum: 2023-07-10T14:50:39Z
Bron: https://wiki.osdev.org/Category:Porting
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APorting&oldid=28166

````text
Articles related to porting libraries (and software in general) to your operating system.
````

<a id="w004797"></a>

## W004797 — Category:System Calls

Bron-ID: W004797
Namespace: Category (14)
Revisie: 28169 | Datum: 2023-07-10T14:55:38Z
Bron: https://wiki.osdev.org/Category:System_Calls
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ASystem_Calls&oldid=28169

````text
Articles related to system calls, their calling conventions, and their design and mechanism in general.
````

<a id="w004798"></a>

## W004798 — Category:History

Bron-ID: W004798
Namespace: Category (14)
Revisie: 28177 | Datum: 2023-07-10T15:50:42Z
Bron: https://wiki.osdev.org/Category:History
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AHistory&oldid=28177

````text
Articles related to historical developments of interest.
````

<a id="w004799"></a>

## W004799 — MACH-O

Bron-ID: W004799
Namespace: Main (0)
Revisie: 28180 | Datum: 2023-07-10T16:51:25Z
Bron: https://wiki.osdev.org/MACH-O
Revisie-URL: https://wiki.osdev.org/index.php?title=MACH-O&oldid=28180
Doorverwijzing: Mach-O

````text
#REDIRECT [[Mach-O]]
````

<a id="w004800"></a>

## W004800 — Template:ACPI

Bron-ID: W004800
Namespace: Template (10)
Revisie: 31013 | Datum: 2026-10-11T16:58:10Z
Bron: https://wiki.osdev.org/Template:ACPI
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AACPI&oldid=31013

````text
{{SmallNavBox
| width:200px;
|Name=[[ACPI]]

|Group0=
|List0=<div>
[[RSDP]]<br />
</div>

|Group1=Fixed Tables
|List1=<div>
[[BGRT]]<br />
[[FADT]]<br />
[[MADT]]<br />
[[MCFG]]<br />
[[RSDT]]/[[XSDT]]<br />
[[SRAT]]<br />
[[WAET]]<br />
[[DMAR]]<br />
</div>

|Group2=Differentiated Tables
|List2=<div>
[[DSDT]]<br />
[[SSDT]]<br />
[[AML]]<br />
</div>

|Group3=Tools/Libs
|List3=<div>
[[ACPICA]]<br />
[[uACPI]]<br />
[[LAI]]<br />
</div>
}}
````

<a id="w004801"></a>

## W004801 — Sol asm

Bron-ID: W004801
Namespace: Main (0)
Revisie: 28204 | Datum: 2023-07-10T18:42:59Z
Bron: https://wiki.osdev.org/Sol_asm
Revisie-URL: https://wiki.osdev.org/index.php?title=Sol_asm&oldid=28204
Doorverwijzing: Solar Assembler

````text
#REDIRECT [[Solar Assembler]]
````

<a id="w004802"></a>

## W004802 — Category:Physical Memory

Bron-ID: W004802
Namespace: Category (14)
Revisie: 28207 | Datum: 2023-07-10T18:48:00Z
Bron: https://wiki.osdev.org/Category:Physical_Memory
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APhysical_Memory&oldid=28207

````text
Articles related to physical memory, cache memory, detection, management, allocation, etc.
````

<a id="w004803"></a>

## W004803 — Category:Virtual Memory

Bron-ID: W004803
Namespace: Category (14)
Revisie: 28208 | Datum: 2023-07-10T18:48:57Z
Bron: https://wiki.osdev.org/Category:Virtual_Memory
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AVirtual_Memory&oldid=28208

````text
Article related to virtual memory, paging, segmentation, address spaces, mapping, management, allocation, etc.
````

<a id="w004804"></a>

## W004804 — File:Windows-Computer-Management.png

Bron-ID: W004804
Namespace: File (6)
Revisie: 28213 | Datum: 2023-07-10T19:01:22Z
Bron: https://wiki.osdev.org/File:Windows-Computer-Management.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AWindows-Computer-Management.png&oldid=28213

````text

````

<a id="w004805"></a>

## W004805 — File:Windows-Create-VHD.png

Bron-ID: W004805
Namespace: File (6)
Revisie: 28214 | Datum: 2023-07-10T19:03:33Z
Bron: https://wiki.osdev.org/File:Windows-Create-VHD.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AWindows-Create-VHD.png&oldid=28214

````text

````

<a id="w004806"></a>

## W004806 — Virtual Monitor

Bron-ID: W004806
Namespace: Main (0)
Revisie: 28247 | Datum: 2023-07-12T16:51:26Z
Bron: https://wiki.osdev.org/Virtual_Monitor
Revisie-URL: https://wiki.osdev.org/index.php?title=Virtual_Monitor&oldid=28247
Doorverwijzing: Virtual-8086 Monitor

````text
#REDIRECT [[Virtual-8086 Monitor]]
````

<a id="w004807"></a>

## W004807 — WAET

Bron-ID: W004807
Namespace: Main (0)
Revisie: 30066 | Datum: 2026-02-23T17:00:05Z
Bron: https://wiki.osdev.org/WAET
Revisie-URL: https://wiki.osdev.org/index.php?title=WAET&oldid=30066

````text
{{ACPI}}

The '''WAET''' (Windows ACPI Emulated Devices Table) is a table in [[ACPI]] running in guest partitions in a virtual machine environment.


== Table Format ==
{| {{wikitable}}
|-
! ACPI Standard Header
! Field
! Byte length
! Byte offset
! Description
|-
| {{Yes}}
| Signature
| 4
| 0x0
| Signature for the WAET
|-
| {{Yes}}
| Length
| 4
| 0x4
| Length, in bytes, of the WAET
|-
| {{Yes}}
| Revision
| 1
| 0x8
| 1
|-
| {{Yes}}
| Checksum
| 1
| 0x9
| Entire table, which must sum to zero
|-
| {{Yes}}
| OEMID
| 6
| 0x10
| Original equipment manufacturer (OEM) identifier (ID)
|-
| {{Yes}}
| OEM Table ID
| 8
| 0x16
| Manufacturer model ID
|-
| {{Yes}}
| OEM Revision
| 4
| 0x24
| OEM revision for supplied OEM table ID
|-
| {{Yes}}
| Creator ID
| 4
| 0x28
| Vendor ID of the ASL compiler utility that created the table
|-
| {{Yes}}
| Creator Revision
| 4
| 0x32
| Revision of the ASL compiler utility that created the table
|-------------------------
| {{No}}
| Emulated Device Flags
| 4
| 0x36
| Container of a bitmask of Windows behavior that this system requires
|}

===Emulated Device Flags===
{| {{wikitable}}
|-
! Bit offset
! Description
|-
| 0
| RTC good Indicates whether the [[RTC]] has been enhanced not to require acknowledgment after it asserts an interrupt. With this bit set, an interrupt handler can bypass reading the RTC register C to unlatch the pending interrupt.
|-
| 1
| ACPI PM timer good Indicates whether the ACPI PM timer has been enhanced not to require multiple reads. With this bit set, only one read of the ACPI PM timer is necessary to obtain a reliable value.
|-
| 31:2
| Reserved; must return 0 when read.
|-
|}

This explanation is a formatted strict copy of the WAET .docx document.

== See Also ==
=== External Links ===
* [https://www.microsoft.com/whdc/system/platform/virtual/WAET.mspx Microsoft's WAET specifications]
* [https://download.microsoft.com/download/7/E/7/7E7662CF-CBEA-470B-A97E-CE7CE0D98DC2/WAET.docx WAET specification document]

[[Category:ACPI]]
````

<a id="w004808"></a>

## W004808 — Talk:A.out

Bron-ID: W004808
Namespace: Talk (1)
Revisie: 28268 | Datum: 2023-08-11T21:26:13Z
Bron: https://wiki.osdev.org/Talk:A.out
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AA.out&oldid=28268

````text
This article is missing a lot of info, so much so, that one can't properly impelement a reader or writer for this file format. To anybody who actually knows the structure of "a.out" files, please provide the following info:

Definitions of the flags found in the a_midmag field of the file header.

Structure of the entries in the Symbol Table.

Structure of the entries in the String Table.

[[User:Videogamer555|Videogamer555]] 16:24, 11 August 2023 (CDT)
````

<a id="w004809"></a>

## W004809 — Talk:NE

Bron-ID: W004809
Namespace: Talk (1)
Revisie: 28269 | Datum: 2023-08-11T21:40:51Z
Bron: https://wiki.osdev.org/Talk:NE
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ANE&oldid=28269

````text
This unfortunately is far from a complete specification for the NE file format. There's no way to implement a file reader or writer for this file format, with the small amount of info provided here. To anybody out there who knows the specifications for this file format, please fill in any missing info in this article.

[[User:Videogamer555|Videogamer555]] 16:40, 11 August 2023 (CDT)
````

<a id="w004810"></a>

## W004810 — XSDP

Bron-ID: W004810
Namespace: Main (0)
Revisie: 28281 | Datum: 2023-08-16T22:21:10Z
Bron: https://wiki.osdev.org/XSDP
Revisie-URL: https://wiki.osdev.org/index.php?title=XSDP&oldid=28281
Doorverwijzing: RSDP

````text
#REDIRECT [[RSDP]]
````

<a id="w004811"></a>

## W004811 — Talk:XSDP

Bron-ID: W004811
Namespace: Talk (1)
Revisie: 28280 | Datum: 2023-08-16T14:47:18Z
Bron: https://wiki.osdev.org/Talk:XSDP
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AXSDP&oldid=28280

````text
TyfoC: In the future, I will update this page and transfer some of the information (about XSDP from the page about RSDP)
````

<a id="w004812"></a>

## W004812 — LAPIC

Bron-ID: W004812
Namespace: Main (0)
Revisie: 28282 | Datum: 2023-08-16T22:23:38Z
Bron: https://wiki.osdev.org/LAPIC
Revisie-URL: https://wiki.osdev.org/index.php?title=LAPIC&oldid=28282
Doorverwijzing: APIC

````text
#REDIRECT [[APIC#Local_APIC_and_IO-APIC]]
````

<a id="w004813"></a>

## W004813 — User:Dbstream/Paging on x86

Bron-ID: W004813
Namespace: User (2)
Revisie: 28403 | Datum: 2023-10-15T16:50:45Z
Bron: https://wiki.osdev.org/User:Dbstream/Paging_on_x86
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADbstream%2FPaging_on_x86&oldid=28403

````text
Paging is a mechanism by which a processor translates virtual addresses to physical addresses. The address space is divided into fixed-size blocks of memory, where one such block is a '''page'''. Usually, translations from virtual to physical addresses are implemented with a kind of recursive data structure, the '''page tables'''. The processor keeps a register that points to a root page table. A page table can be conceptually thought of as an array of pointers to pages or next-level page tables. When a virtual address is to be translated, different parts of it function as indices into the different page table levels.

== Paging modes ==

There are many different paging modes on x86 CPUs that depend on the state of the processor. These are listed below:

{| {{wikitable}}
|-
!CR0.PG
!CR4.PAE
!CR4.LA57
!EFER.LME
!Paging mode
|-
| 0
| (ignored)
| (ignored)
| (ignored)
! paging disabled
|-
| 1
| 0
| (ignored)
| (ignored)
! 32-bit paging
|-
| 1
| 1
| (ignored)
| 0
! PAE paging
|-
| 1
| 1
| 0
| 1
! 4-level paging
|-
| 1
| 1
| 1
| 1
! 5-level paging
|}

This article only deals with long-mode paging (4-level and 5-level paging).

== Paging (long-mode) ==

A page table is a page which contains an array of 512 64-bit values. These values encode permissions and flags, such as whether a page is writable, if it can be cached, but most importantly, a pointer to a page which can be the next level of page tables. In 4-level paging, the '''cr3''' register of the CPU points to a PML4. The PML4 entries point to PML3s, whose entries point to PML2s, whose entries point to PML1s, whose entries point to physical pages of memory. In 5-level paging, there is another level of indirection, in that the ''cr3'' register points to a PML5, whose entries point to PML4s. This enables a bigger effective address space.

WARNING: the Intel and AMD manuals refer to the PML3, PML2 and PML1 as page directory pointer (PDP), page directory (PD) and page table (PT). For simplicity, this article refers to them by different names in a more consistent naming scheme.

The format of a page table entry is as follows:
<pre>
+63----------+62-----------------+MAXPHYADDR---------------------+11-------+8-------+7------------+6------+5---------+4----+3----+2-----+1---------+0--------+
| no-execute |      ignored      | pointer to page or page table | ignored | global | pat or size | dirty | accessed | pcd | pwt | user | writable | present |
+----------63+-------MAXPHYADDR+1+-----------------------------12+--------9+-------8+------------7+------6+---------5+----4+----3+-----2+---------1+--------0+
</pre>

(MAXPHYADDR is the number of bits of physical address space supported. It is a number between 36 and 52.)

Look at the Intel(r) 64 and IA-32 Architecture Software Developer's Manual for information about what each of these bits do.

In long mode, an address looks like this:
<pre>
+63---------+57-----+47-----+38-----+29-----+20-----+11----------------+
| canonical | PML5e | PML4e | PML3e | PML2e | PML1e | offset into page |
+---------58+-----48+-----39+-----30+-----21+-----12+-----------------0+
</pre>

If 4-level paging is used, the PML5e doesn't exist, and is instead canonical. The canonical part of an address is effectively a sign-extension of the rest of the address.

== Detecting support for 5-level paging ==

Support for 5-level paging (LA57, 57-bit linear addresses) can be detected using CPUID leaf 7 subleaf 0. If bit 16 is set in ecx, LA57 is supported.
<source lang="c">
bool is_la57_supported(void)
{
        if(cpuid_a(0) < 7)
                        /* CPUID leaf 7 is not supported. */
                return false;

                /*
                 * NOTE: when implementing 'cpuid_c()', make sure ecx is
                 * zero before the CPUID instruction is called. Otherwise,
                 * information from the wrong subleaf will be returned.
                 */
        return (cpuid_c(7) & (1 << 16)) ? true : false;
}
</source>

Enabling LA57 cannot be done from within long-mode, so there are three options:
1) Have a protected-mode trampoline that enables it.
2) Enable it before your kernel enters long mode.
3) Depend on the bootloader to enable it.

With a modern bootloader like [[Limine]], LA57 will probably be enabled if it exists. For less modern bootloaders and protocols, the second approach may be needed. It is described below:
<pre>
        /* Assumption: we are somewhere in a protected-mode boot stub. */

        /* Detect if we support LA57. */
        xorl %eax, %eax
        cpuid
        cmpl $7, %eax
        jb .Lno_la57

        movl $7, %eax
        xorl %ecx, %ecx
        cpuid
        testl $(1 << 16), %ecx
        jz .Lno_la57

                /* We have LA57. */
        movl %cr4, %eax
        orl $(__CR4_PAE | __CR4_LA57), %eax
        movl $boot_pml5, %edx
        jmp 1f

.Lno_la57:      /* We do not have LA57. */
        movl %cr4, %eax
        orl $__CR4_PAE, %eax
        movl $boot_pml4, %edx

1:      /* %eax holds the %cr4 we want, and %edx holds the %cr3 we want */
        movl %eax, %cr4
        movl %edx, %cr3
</pre>

== Translation lookaside buffer (TLB) ==

TODO: describe the TLB and invalidation

== Page attribute table (PAT) ==

TODO: describe the page attribute table and cache modes.

== Example: Mapping a 4KiB page with 4-level and 5-level paging ==

The code below is an example of how to map a 4KiB page of memory. It maps the page read-write, supervisor-only, executable, and non-global. It assumes the following about your kernel:

- '''alloc_page''' returns the physical address of a page full of zeroed memory.

- '''phys_to_virt''' converts a physical address to a virtual address in some kind of higher-half direct map.

- '''addr_t''' is a typedef that matches uint64_t.

- '''root_page_table''' is the page table in use (pointed to by '''cr3''') and is a '''volatile uint64_t *'''.

- '''use_5level_paging''' is non-zero if 5-level paging is being used, otherwise zero.

<source lang="c">
enum pte_bits {
        PG_PRESENT      = 1 << 0,
        PG_WRITE        = 1 << 1
};

void map_page(addr_t virt, addr_t phys)
{
        /* how many page tables to traverse before we get to the PML1 */
        int l = use_5level_paging ? 4 : 3;

        /* points to the PML5e or PML4e that manages 'virt' */
        volatile uint64_t *table = root_page_table + ((virt >> (12 + 9 * l)) & 0x1ff);

        /*
         * In a loop, reduce the scope of 'table' by traversing downward and
         * allocating new page tables as neccessary.
         */
        while(l) {
                uint64_t value = *table;
                if(value & PG_PRESENT) {
                        /* page table already exists */
                        value &= 0x7ffffffffffff000;
                } else {
                        /* allocate a new page table */
                        value = alloc_page();
                        *table = value | PG_PRESENT | PG_WRITE;
                }

                l--;
                table = (volatile uint64_t *) phys_to_virt(value) + ((virt >> (12 + 9 * l)) & 0x1ff);
        }

        /* 'table' now points to the PTE we're interested in modifying */
        *table = phys | PG_PRESENT | PG_WRITE;
}
</source>
````

<a id="w004814"></a>

## W004814 — PCIE

Bron-ID: W004814
Namespace: Main (0)
Revisie: 28318 | Datum: 2023-09-24T22:37:58Z
Bron: https://wiki.osdev.org/PCIE
Revisie-URL: https://wiki.osdev.org/index.php?title=PCIE&oldid=28318
Doorverwijzing: PCI Express

````text
#REDIRECT [[PCI Express]]
````

<a id="w004815"></a>

## W004815 — File:HhuOS.png

Bron-ID: W004815
Namespace: File (6)
Revisie: 28320 | Datum: 2023-09-25T13:08:56Z
Bron: https://wiki.osdev.org/File:HhuOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AHhuOS.png&oldid=28320

````text
Screenshot of hhuOS after running a few shell commands.
````

<a id="w004816"></a>

## W004816 — User:Hhubsinfo/hhuOS

Bron-ID: W004816
Namespace: User (2)
Revisie: 29329 | Datum: 2024-12-23T19:52:05Z
Bron: https://wiki.osdev.org/User:Hhubsinfo/hhuOS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AHhubsinfo%2FhhuOS&oldid=29329

````text
{{Infobox Project
|name=hhuOS

|image=HhuOS.png
|caption=hhuOS Shell

|developers=[[User:Hhubsinfo|C. Gesse | F. Ruhland]]

|kerneltype=Monolithic
|platform=x86
|taskmodel=Preemptive multitasking
|language=C++, Assembly

|website=[https://github.com/hhuOS/hhuOS/wiki hhuOS Wiki]
|github=hhuOS/hhuOS

|license=GPLv3
}}

'''hhuOS''' is a small operating system for the x86-architecture, build for teaching purposes. The main goal of this project is to show, how different aspects of operating systems theory can be implemented and linked together. The system is not aimed to be a fully featured operating system for daily use. It is implemented in C++ with small parts of assembly where needed.

This is a project by the [https://www.cs.hhu.de/en/research-groups/operating-systems.html Operating Systems group] at the Heinrich Heine University Düsseldorf.

== Features ==

=== Kernel Features ===
* Preemptive multithreading
* User space processes, running in isolated address spaces in ring 3
* Load and run binary ELF-files
* On demand Paging
* Virtual file system with support for FAT (based on [http://elm-chan.org/fsw/ff/00index_e.html FatFs]), Initrd and virtual subfilesystems similar to ''/dev'', ''/proc'', etc.
* Block based memory manager for managing page frames and list based memory manager for kernel/user heaps
* Multiboot2 compliant

=== Hardware support ===
* Runs on BIOS and UEFI systems
* Keyboard and Mouse support
* CGA and Linear frame buffer support, either manual by using VESA BIOS calls or handed over by the bootloader
* Floppy driver
* IDE and AHCI driver for hard disk drives
* UDP/IP-stack and driver for RTL8139 and NE2000 Ethernet cards
* SoundBlaster driver and an application to play .wav-files (no sophisticated sound subsystem yet)
* APIC support with preparations for multicore support done
* FPU/MMX/SSE with multitasking is supported
* PIT, RTC, APIC Timer and HPET support for time measurement
* GDB stub implementation via serial port for debugging on real hardware
* Can use BIOS calls on old hardware by switching to real mode and back to protected mode again

=== User space library ===
* Collection classes like ArrayList, HashMap, etc.
* Graphic library for working with the frame buffer
* Game Engine for simple 2D/3D games (no hardware acceleration)
* File access
* Datagram sockets for sending/receiving UDP packets
* ANSI C compatible standard library
````

<a id="w004817"></a>

## W004817 — File:HhuOS-logo.png

Bron-ID: W004817
Namespace: File (6)
Revisie: 28323 | Datum: 2023-09-25T13:45:26Z
Bron: https://wiki.osdev.org/File:HhuOS-logo.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AHhuOS-logo.png&oldid=28323

````text

````

<a id="w004818"></a>

## W004818 — File:Gloire.png

Bron-ID: W004818
Namespace: File (6)
Revisie: 28328 | Datum: 2023-09-27T03:53:42Z
Bron: https://wiki.osdev.org/File:Gloire.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AGloire.png&oldid=28328

````text
A screenshot of Gloire, to be used in the projects page
````

<a id="w004819"></a>

## W004819 — File:VirtualBox BleskOS 28 09 2023 16 46 49.png

Bron-ID: W004819
Namespace: File (6)
Revisie: 28336 | Datum: 2023-09-28T15:40:33Z
Bron: https://wiki.osdev.org/File:VirtualBox_BleskOS_28_09_2023_16_46_49.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVirtualBox_BleskOS_28_09_2023_16_46_49.png&oldid=28336

````text

````

<a id="w004820"></a>

## W004820 — El Torito

Bron-ID: W004820
Namespace: Main (0)
Revisie: 28353 | Datum: 2023-10-03T09:15:38Z
Bron: https://wiki.osdev.org/El_Torito
Revisie-URL: https://wiki.osdev.org/index.php?title=El_Torito&oldid=28353
Doorverwijzing: El-Torito

````text
#REDIRECT [[El-Torito]]
````

<a id="w004821"></a>

## W004821 — Category:Pages with broken file links

Bron-ID: W004821
Namespace: Category (14)
Revisie: 28365 | Datum: 2023-10-04T11:53:38Z
Bron: https://wiki.osdev.org/Category:Pages_with_broken_file_links
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APages_with_broken_file_links&oldid=28365

````text
Pages containing links to non-existent files.
````

<a id="w004822"></a>

## W004822 — GPU

Bron-ID: W004822
Namespace: Main (0)
Revisie: 28367 | Datum: 2023-10-04T12:07:15Z
Bron: https://wiki.osdev.org/GPU
Revisie-URL: https://wiki.osdev.org/index.php?title=GPU&oldid=28367
Doorverwijzing: Accelerated Graphic Cards

````text
#REDIRECT [[Accelerated Graphic Cards]]
````

<a id="w004828"></a>

## W004828 — User:Dbstream/Booting on x86

Bron-ID: W004828
Namespace: User (2)
Revisie: 28409 | Datum: 2023-10-21T17:50:41Z
Bron: https://wiki.osdev.org/User:Dbstream/Booting_on_x86
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADbstream%2FBooting_on_x86&oldid=28409

````text
== What this article is not ==

This article is not about writing a bootloader. It is also not a tutorial, so don't treat it like one. This article is about writing an operating system kernel that boots, more specifically writing a kernel for a 64-bit x86 machine that boots using an already existing boot protocol such as multiboot2.

This article is not a programming tutorial. It is not an x86 assembler tutorial. Some familiarity with the GNU as syntax is assumed.

There is a much more modern alternative to multiboot2 available, the [[Limine]] boot protocol. But, this article will focus mainly on multiboot2, because that's where most mistakes will be made. A lot of the techniques in this article can be applied in other areas and for other boot protocols as well, but you should carefully consider whether or not there are other alternatives available. For example, I wouldn't do any unneccessary operations involving page tables in an assembler entry stub if I was writing a Limine kernel, and you cannot load a Limine kernel in lower-half memory.

== Relocatable kernel image ==

It is trivial to write a multiboot2 kernel that can only be loaded in one place, but much more awesome to have a fully relocatable kernel image that can be loaded anywhere in the lower 4GiB of RAM. GRUB2 probably supports relocating an ELF64 image, but do we trust GRUB2? Personally, I have my doubts, not to speak of how multiboot2 enters the kernel in 32-bit mode.

This article assumes that you are writing a lower-half 64-bit kernel, which is very effective for a prekernel (which can be thought of as something in-between the bootloader and your "real" kernel, as it extracts the "real" kernel and constructs page tables that map it from the higher-half).

All C code is assumed to be compiled with -mcmodel=large -fPIE, and the kernel is expected to be linked as -pie.

=== Linker script ===

It is a good idea to link the kernel into an ELF64 image and then objcopy it into a binary. This can be done using the following Makefile fragment (please adapt to your own build system):
<source lang="make">
kernel: kernel.elf
        x86_64-elf-objcopy -O binary kernel.elf kernel

# note how 'kernel.lds' is explicitly listed as the first dependency
# of 'kernel.elf', so it will become the argument to '-T'.
kernel.elf: kernel.lds $(kernel-objs)
        x86_64-elf-ld $(LDFLAGS) -T $^ -o $@
</source>

The linker script will link the kernel at virtual address 0, for simplicity. This conflicts with lots of things already in memory at that location, but the kernel image is relocatable and will actually be loaded at some other place in the lower 4GiB of RAM.

The multiboot2 header needs to go very early in the kernel image, so why not place it first?

<pre>
OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

PHDRS
{
        /* not that these permissions actually matter, but it is good practice */
        head    PT_LOAD         FLAGS(7); /* RWX */
        text    PT_LOAD         FLAGS(5); /* R-X */
        rodata  PT_LOAD         FLAGS(4); /* R-- */
        data    PT_LOAD         FLAGS(6); /* RW- */
        dynamic PT_DYNAMIC      FLAGS(6); /* RW- */
}

SECTIONS
{
        . = 0;
        .head           : { *(.head) }                  :head
        .text           : { *(.text .text.*) }          :text
        .rodata         : { *(.rodata .rodata.*) }      :rodata
        .data           : { *(.data .data.*) }          :data
        .dynamic        : { *(.dynamic) }               :data :dynamic
        .bss            : { *(COMMON .bss .bss.*) }     :data
        __image_end = .;

        /DISCARD/ :
        {
                *(.comment)
                *(.eh_frame)
                *(.note .note.*)
        }
}
</pre>

== Header and 32-bit entry stub ==

We need to write the 32-bit entry stub in assembler, because it needs to be able to deal with being relocated. We also need a multiboot2 header. Because we're emitting a flat binary, we need the load address tag, entry point tag, and relocatable image tag. An example of how the multiboot2 header can look is shown below:
<pre>
        .file "head.S"

        /*
         * Sometimes, we want to reference a symbol without generating a
         * relocation entry for it. Use this macro for that. (assumes
         * image_start is at physical address zero)
         */
#define linktime_va(x) (x - image_start)

        .section ".head", "awx"
        .globl image_start
image_start:

mb2_header:
        .long 0xe85250d6                                        /* MB2_MAGIC */
        .long 0                                                 /* MB2_ARCH_X86 */
        .long mb2_header_end - mb2_header                       /* header length */
        .long -(0xe85250d6 + 0 + (mb2_header_end - mb2_header)) /* checksum */
tag_info_req:
        .word 1           /* MB2_TAG_INFO_REQ */
        .word 0           /* flags */
        .long 8           /* size */
tag_load_addr:
        .word 2           /* MB2_TAG_LOAD_ADDR */
        .word 0           /* flags */
        .long 24          /* size */
        .long linktime_va(mb2_header)      /* header_addr */
        .long 0xffffffff                   /* load_addr (-1 means load the entire file) */
        .long 0                            /* load_end_addr (0 means load the entire file */
        .long linktime_va(__image_end)     /* bss_end_addr */
tag_entry_addr:
        .word 3           /* MB2_TAG_ENTRY_ADDR */
        .word 0           /* flags */
        .long 12          /* size */
        .long linktime_va(_start)    /* entry_addr */
        .long 0           /* padding (for 8-byte alignment of tags) */
tag_relocatable:
        .word 10          /* MB2_TAG_RELOCATABLE */
        .word 0           /* flags */
        .long 24          /* size */
        .long 0           /* min_addr */
        .long 0xffffffff  /* max_addr */
        .long 0x1000      /* align */
        .long 0           /* preference (0 means "place me wherever") */
tag_end:
        .word 0           /* MB2_TAG_END */
        .word 0           /* flags */
        .long 8
mb2_header_end:
</pre>

When we enter the kernel, we need to figure out the load offset:
<pre>
        /*
         * 32-bit entry point into the kernel.
         *
         * We are in a very fragile environment. We are in 32-bit mode with
         * paging disabled, and we have been relocated to somewhere in memory
         * that we don't know. Luckily, we do know that %ebx points to the boot
         * information structure, so we can use it as scratch space for a stack,
         * letting us figure out where we are.
         */
 
        .code32

        .balign 16
        .globl _start
        .type _start, @function
_start:
        leal 8(%ebx), %esp          /* use the fixed 'reserved' field as scratch space */
        movl (%esp), %eax           /* save it, if its meaning is to be changed in the future */
        call 1f                     /* the assembler will emit a 'near call', a relative jump */
1:      popl %ebp                   /* %ebp now holds the runtime address of this instruction */
        subl $linktime_va(1b), %ebp /* calculate the load offset */
        movl %eax, (%esp)           /* restore 'reserved' */

        /* %ebp now holds the relocation offset, and we can do stuff with it. */
</pre>

Now that we have a load offset, our code will follow a generic pattern of fixing up pointers and loading stuff. Where we'd normally use <tt>movl $label, %eax</tt>, we instead need to use <tt>leal linktime_va(label)(%ebp), %eax</tt>. In simplified terms, we need to:
* Load a [[Global Descriptor Table]], with entries for 64-bit ring-zero.
* Setup page tables.
* Enter long-mode.
* Relocate ourselves using the _DYNAMIC table.

After determining the relocation offset, we can relocate the hardware datastructures, like aforementioned GDT and page tables:
<pre>
        leal linktime_va(boot_stack_top)(%ebp), %esp      /* setup a small stack, boot_stack_top
                                                             should point to the top of some free
                                                             range of memory.                     */

        subl $8, %esp                        /* allocate space for a 32-bit segment_ptr on the stack */
        movw $gdt_end - gdt - 1, (%esp)      /* segment_ptr::limit */
        leal linktime_va(gdt)(%ebp), %eax
        movl %eax, 2(%esp)                   /* segment_ptr::base */
        lgdt (%esp)
        addl $8, %esp

        /* later in the file */
        .balign 16
gdt:
        .quad 0                  /* offset 0x00: null segment */
        .quad 0x00009a000000ffff /* offset 0x08: 16-bit kernel CS (we don't use this) */
        .quad 0x000093000000ffff /* offset 0x10: 16-bit kernel DS (we don't use this) */
        .quad 0x0000cf9a0000ffff /* offset 0x18: 32-bit kernel CS (we don't use this) */
        .quad 0x0000cf930000ffff /* offset 0x20: 32-bit kernel DS (we don't use this) */
        .quad 0x00af9b000000ffff /* offset 0x28: 64-bit kernel CS */
        .quad 0x00af93000000ffff /* offset 0x30: 64-bit kernel DS */
gdt_end:

        .balign 16
boot_stack:
        .fill 4096, 1, 0         /* this allocates space for a 4KiB stack */
boot_stack_top:                  /* top of the stack */
</pre>

We need to have a GDT with 64-bit kernel entries so that we can switch to long mode later.

TODO: finish writing this article.
````

<a id="w004829"></a>

## W004829 — User talk:Minoto

Bron-ID: W004829
Namespace: User talk (3)
Revisie: 28423 | Datum: 2023-10-22T23:36:48Z
Bron: https://wiki.osdev.org/User_talk:Minoto
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AMinoto&oldid=28423

````text
Hey! Please stop undoing changes to the [[Tutorials]] page.

- pitust

If Blanham wants to remove potentially useful information, providing a little more reasoning for it than "Die!" and "x is crappy" would help his case.

- Minoto

== [[Tutorials]] ==
Hello, there seems to currently be somewhat of an edit war going on that page. I believe that University of Birmingham tutorial should be removed, due to many known issues. It isn't really "potentially useful", since there's other tutorials listed that cover the same stuff much better. Just look at the PDF file yourself, and you'll probably find a lot of problems from the start. For a more thorough explanation, see this critique of a very similar tutorial based on the tutorial being removed from the list: https://github.com/cfenollosa/os-tutorial/issues/269. Some of the problems mentioned there match the ones with this PDF file (and it's explicitly noted there when that is the case). --[[User:Lukflug|Lukflug]] 15:13, 22 October 2023 (CDT)

I lean towards preservation rather than deletion, and at first glance it didn't seem any worse than any other tutorial. But I do seem to be in the wrong here, so I'll apologize and let it go. --[[User:Minoto|Minoto]]

Members of the unaffiliated OSDev Discord server have embarked on a project to improve and modernize content on the wiki - a welcome endeavor given the last time we did such a thing was circa 2014. Among those improvements is removing links to tutorials that are of poor quality, which is something I very much support. I, too, tend to prefer preservation over deletion, but in this case we are driving beginners towards resources that can be actively harmful and that is something we should try to avoid. I have locked [[Tutorials]] for now and will be applying requested edits manually. --[[User:Klange|klange]] 18:36, 22 October 2023 (CDT)
````

<a id="w004830"></a>

## W004830 — User:Arongeo

Bron-ID: W004830
Namespace: User (2)
Revisie: 28428 | Datum: 2023-10-29T12:04:49Z
Bron: https://wiki.osdev.org/User:Arongeo
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AArongeo&oldid=28428

````text
Hi I'm Aron,
I am a waaay too overoptimistic person, who thinks they can accomplish cool things if they work hard enough, so I'm here, cause OS dev sure is a really cool thing. I will probably do a bunch of stupid things, make stupid decisions, and write stupid code, but hopefully I'll learn from them. Anyways, have a good day!

[http://www.arongeo.com my website]
````

<a id="w004831"></a>

## W004831 — File:Mintia.png

Bron-ID: W004831
Namespace: File (6)
Revisie: 28440 | Datum: 2023-11-07T06:22:21Z
Bron: https://wiki.osdev.org/File:Mintia.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AMintia.png&oldid=28440

````text
A screenshot of the MINTIA hobby OS
````

<a id="w004832"></a>

## W004832 — SD Card

Bron-ID: W004832
Namespace: Main (0)
Revisie: 28445 | Datum: 2023-11-07T11:34:30Z
Bron: https://wiki.osdev.org/SD_Card
Revisie-URL: https://wiki.osdev.org/index.php?title=SD_Card&oldid=28445
Doorverwijzing: SD card

````text
#REDIRECT [[SD card]]
````

<a id="w004833"></a>

## W004833 — User talk:Arthurcast18

Bron-ID: W004833
Namespace: User talk (3)
Revisie: 28463 | Datum: 2023-11-15T14:14:47Z
Bron: https://wiki.osdev.org/User_talk:Arthurcast18
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AArthurcast18&oldid=28463

````text
<center><div style="border: solid black 3px; background-color: gray; width: 13cm; height: 7.4cm;"><h2 style="border-bottom: none; height: auto;">Welcome to my discussion page!</h2><div style="height: 6.22cm; background-color: #C0C0C0; border-top: solid black 1px;"><p>Here you can talk with me or ask something to me.<br> If I don't reply you probably:</p><p style="color: red;">I am in Wikipedia editing pages or something else.<br>I am in other site doing something.<br>I am programming.<br>I am sleeping.<br>I am studying.<br>I am using my phone.<br>I am doing something else.<br></p><p>Then if this happen just wait I reply you.</p></div></div></center>

== Discussions ==
Enter your questions below.
````

<a id="w004834"></a>

## W004834 — User:Thewindboi

Bron-ID: W004834
Namespace: User (2)
Revisie: 28482 | Datum: 2023-11-26T18:17:14Z
Bron: https://wiki.osdev.org/User:Thewindboi
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AThewindboi&oldid=28482

````text
== what is love ==
# oh baby dont hurt me
# dont hurt me
# no more
=== baby dont hurt me ===
# dont hurt me
# no more 
# what is love
# yeah
--------
== no ==
# i dont know why youre not fair
# i give you my love
# but you dont care
# so what is right and what is wrong
# gimme a sign
--------
== what is love ==
# oh baby dont hurt me
# dont hurt me
# no more
=== what is love ===
# oh baby dont hurt me
# dont hurt me
# no more
````

<a id="w004835"></a>

## W004835 — File:MaslOS2-demo.png

Bron-ID: W004835
Namespace: File (6)
Revisie: 28536 | Datum: 2023-12-26T18:50:53Z
Bron: https://wiki.osdev.org/File:MaslOS2-demo.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AMaslOS2-demo.png&oldid=28536

````text

````

<a id="w004836"></a>

## W004836 — User:Marceldarcel

Bron-ID: W004836
Namespace: User (2)
Revisie: 28538 | Datum: 2023-12-26T19:41:33Z
Bron: https://wiki.osdev.org/User:Marceldarcel
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMarceldarcel&oldid=28538

````text
'''Marceldarcel'''

Yes this is my profile!

I am marceldarcel, the rocc, marceldobehere or just Marcel.


Programming goofy things is my life.

I program in C#, C++, Java, JS, Python and quite a few other langs. (Even some of my own bc yes)

Most of my projects are on [https://github.com/marceldobehere My Github].


I have written two operating systems: [https://github.com/marceldobehere/MaslOS MaslOS] and [https://github.com/marceldobehere/MaslOS-2 MaslOS2].

Here is a picture of MaslOS2:

[[File:MaslOS2-demo.png|600px]]


I shall post some other things here too.


If you want to contact me, my Discord username is marceldarcel.




Have a nice day!
````

<a id="w004837"></a>

## W004837 — File:Bleskos 2023.png

Bron-ID: W004837
Namespace: File (6)
Revisie: 28545 | Datum: 2024-01-04T17:37:01Z
Bron: https://wiki.osdev.org/File:Bleskos_2023.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABleskos_2023.png&oldid=28545

````text

````

<a id="w004838"></a>

## W004838 — File:ISA Bus pins.svg.png

Bron-ID: W004838
Namespace: File (6)
Revisie: 28557 | Datum: 2024-01-12T01:18:39Z
Bron: https://wiki.osdev.org/File:ISA_Bus_pins.svg.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISA_Bus_pins.svg.png&oldid=28557

````text
A pinout of the ISA 16-bit bus. Not made by me.

Attribution: Fred the Oyster, CC BY-SA 4.0 <https://creativecommons.org/licenses/by-sa/4.0>, via Wikimedia Commons
````

<a id="w004839"></a>

## W004839 — User:Puffer

Bron-ID: W004839
Namespace: User (2)
Revisie: 28626 | Datum: 2024-02-15T18:57:16Z
Bron: https://wiki.osdev.org/User:Puffer
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APuffer&oldid=28626

````text
Puffer is an interesting being.

He used to do iPhone hacking, and web development. 
But he now is a newbie OSDever.

His github:https://github.com/KevinAlavik

His OSDev related projects:

* Paradox(https://github.com/KevinAlavik/Paradox): a crappy kernel
* Nighterm(https://github.com/KevinAlavik/Nighterm): a crappy terminal emulator
* KIF(https://github.com/KevinAlavik/KernelImageFormat): a crappy image format
````

<a id="w004840"></a>

## W004840 — User:Lolguy91

Bron-ID: W004840
Namespace: User (2)
Revisie: 28567 | Datum: 2024-01-17T18:29:26Z
Bron: https://wiki.osdev.org/User:Lolguy91
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALolguy91&oldid=28567

````text
Well, I am a semi-experienced OSDever,

I am OSDevving since spring 2022.

my main project always has been RoidsOS(https://github.com/roidsos).

I procrastinate a lot, so dont expect it to "finish" anytime soon....

Also Im exploring RISC-V, Im a bit disappointed how little info there is on this wiki about RISC-V stuff
````

<a id="w004841"></a>

## W004841 — File:GloireJWM.png

Bron-ID: W004841
Namespace: File (6)
Revisie: 29439 | Datum: 2025-03-15T16:15:22Z
Bron: https://wiki.osdev.org/File:GloireJWM.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AGloireJWM.png&oldid=29439

````text
A screenshot of Gloire, an Ironclad distribution, running some software.
````

<a id="w004842"></a>

## W004842 — User:Tokyo

Bron-ID: W004842
Namespace: User (2)
Revisie: 28580 | Datum: 2024-01-26T16:34:47Z
Bron: https://wiki.osdev.org/User:Tokyo
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATokyo&oldid=28580

````text
Trans girl making an OS in Rust using no external dependencies.

[https://gtihub.com/juls0730/CappuccinOS CappuccinOS]
````

<a id="w004843"></a>

## W004843 — Talk:Loading Icons

Bron-ID: W004843
Namespace: Talk (1)
Revisie: 30000 | Datum: 2026-02-14T01:57:38Z
Bron: https://wiki.osdev.org/Talk:Loading_Icons
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ALoading_Icons&oldid=30000

````text
= PNG loader code wrong =
The PNG loader code using stb_image.h calls stbi__png_load() which I could not find in in stb_image.h. I suspect the statement, "no library linking needed" is essentially wrong, and more files are needed than just stb_image.h. I would like to base my code on the STB image library, but it's going to take a bit of looking around to find the right files. If I find them, I'll edit them in, but I may post partial finds here. [[User:Eekee|Eekee]] 18:28, 28 January 2024 (CST)
: Well this is stb_image.c: https://web.archive.org/web/20070913024502/http://www.nothings.org/stb_image.c [[User:Eekee|Eekee]] 18:31, 28 January 2024 (CST)

: I don't know why someone linked this to the archived version from 2007 when bzt wrote this article much later than that. If you look at an actual current version, you'll find the functions this article references: https://github.com/nothings/stb/blob/master/stb_image.h --[[User:Klange|klange]] 02:22, 31 January 2024 (CST)

== Loading images? ==

This article is basically is about loading images. Shouldn't the article be renamed to reflect that?   -[[User:Tocariimaa|Tocariimaa]] ([[User talk:Tocariimaa|talk]]) 01:57, 14 February 2026 (UTC)
````

<a id="w004844"></a>

## W004844 — Español Llano

Bron-ID: W004844
Namespace: Main (0)
Revisie: 28597 | Datum: 2024-02-03T12:29:18Z
Bron: https://wiki.osdev.org/Espa%C3%B1ol_Llano
Revisie-URL: https://wiki.osdev.org/index.php?title=Espa%C3%B1ol_Llano&oldid=28597

````text
{{stub}}

A Spanish-language equivalent to [[Plain English Programming]]. Details to be discovered. It is likely to be a strongly-typed language with a very small run-time, compiling something like pseudocode directly to machine code.

FIXME: Perhaps a Spanish-speaker might be able to find a link. Perhaps best to email the Osmosian Order for a link. The address is on their blog.
````

<a id="w004845"></a>

## W004845 — Talk:Intel Ethernet i217

Bron-ID: W004845
Namespace: Talk (1)
Revisie: 28602 | Datum: 2024-02-03T18:33:21Z
Bron: https://wiki.osdev.org/Talk:Intel_Ethernet_i217
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AIntel_Ethernet_i217&oldid=28602

````text
Is it normal that startLink() and printMac() doesn't have the source code ?
````

<a id="w004846"></a>

## W004846 — File:Raidou.png

Bron-ID: W004846
Namespace: File (6)
Revisie: 28616 | Datum: 2024-02-09T22:45:50Z
Bron: https://wiki.osdev.org/File:Raidou.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ARaidou.png&oldid=28616

````text
The RaidouOS desktop
````

<a id="w004847"></a>

## W004847 — File:PatienceOS.png

Bron-ID: W004847
Namespace: File (6)
Revisie: 28638 | Datum: 2024-02-27T11:13:03Z
Bron: https://wiki.osdev.org/File:PatienceOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3APatienceOS.png&oldid=28638

````text
QEMU boot splash screen
````

<a id="w004848"></a>

## W004848 — User:Ravenlee

Bron-ID: W004848
Namespace: User (2)
Revisie: 28679 | Datum: 2024-03-01T08:44:25Z
Bron: https://wiki.osdev.org/User:Ravenlee
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARavenlee&oldid=28679

````text

````

<a id="w004849"></a>

## W004849 — Power Macintosh

Bron-ID: W004849
Namespace: Main (0)
Revisie: 28684 | Datum: 2024-03-04T07:10:00Z
Bron: https://wiki.osdev.org/Power_Macintosh
Revisie-URL: https://wiki.osdev.org/index.php?title=Power_Macintosh&oldid=28684

````text
{{In Progress}}

Apple's '''Power Macintosh''' series, which, as its name suggests, use [[PowerPC]] processors, succeeded the earlier Motorola 680x0 CPU-based Macintosh models in 1994 with the [https://en.wikipedia.org/wiki/Power_Macintosh_6100 Power Mac 6100] series. Most Power Macs used [[Open Firmware]] for booting, with the exception of the aforementioned 6100 series.

== Variants == 

== Firmware ==

== Existing Ports ==

== See Also ==
=== Articles ===
=== Reading ===
=== Links ===


== References ==

[[Category:Apple]]
[[Category:PowerPC]]
[[Category:Open Firmware]]
````

<a id="w004850"></a>

## W004850 — Open Firmware

Bron-ID: W004850
Namespace: Main (0)
Revisie: 30971 | Datum: 2026-09-25T17:03:30Z
Bron: https://wiki.osdev.org/Open_Firmware
Revisie-URL: https://wiki.osdev.org/index.php?title=Open_Firmware&oldid=30971

````text
{{In Progress}}

'''Open Firmware''', standardized as IEEE 1275, is a system firmware that originated from Sun Microsystems' ''OpenBoot''. It is implemented in the Forth programming language and utilizes (and originated) [[Devicetree|device trees]] for hardware description. Open Firmware is used in Apple [[Power Macintosh |PowerPC-based Macintosh]] computers (excluding the initial Power Macintosh 6100 series), various IBM and Motorola PowerPC and POWER hardware, SPARC-based Sun Microsystems machines, and the x86-based OLPC project.


== Implementations == 

== Existing Examples of Use ==


== See Also ==
=== Articles ===
=== Reading ===
=== Links ===
* [http://firmworks.com/ Firmworks Home Page]
* [http://firmworks.com/www/featlist.htm Open Firmware feature list (with Firmworks extensions)]
* [https://www.devicetree.org/open-firmware/home.html Open Firmware website (hosted by devicetree)]
* [https://www.devicetree.org/open-firmware/mejohnson/ Mathew John's Introduction to Open Firmware]

== References ==

[[Category:PowerPC]]
[[Category:Sparc]]
[[Category:Open Firmware]]
[[Category:Firmware]]
````

<a id="w004851"></a>

## W004851 — Initramfs

Bron-ID: W004851
Namespace: Main (0)
Revisie: 28686 | Datum: 2024-03-04T16:18:11Z
Bron: https://wiki.osdev.org/Initramfs
Revisie-URL: https://wiki.osdev.org/index.php?title=Initramfs&oldid=28686
Doorverwijzing: Initrd

````text
#REDIRECT [[Initrd]]
````

<a id="w004852"></a>

## W004852 — File:NullTerminator.png

Bron-ID: W004852
Namespace: File (6)
Revisie: 28689 | Datum: 2024-03-11T19:46:10Z
Bron: https://wiki.osdev.org/File:NullTerminator.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ANullTerminator.png&oldid=28689

````text
Image showing a string containing "osdev" with a null terminator
````

<a id="w004853"></a>

## W004853 — Ramfb

Bron-ID: W004853
Namespace: Main (0)
Revisie: 30875 | Datum: 2026-07-09T01:06:40Z
Bron: https://wiki.osdev.org/Ramfb
Revisie-URL: https://wiki.osdev.org/index.php?title=Ramfb&oldid=30875

````text
== Introduction ==
ramfb is a simple way to get graphics on [[QEMU]] via a framebuffer in memory on embedded platforms like ARM or riscv.
It works by adding <code>-device ramfb</code> to your QEMU command line and then configuring ramfb via fw_cfg.
ramfb requires QEMU DMA support which should be available on platforms that have fw_cfg as MMIO (as opposed to x86 IO ports).

== Explanation ==
To initialize the ramfb device we need to write the following structure to the fw_cfg entry with the name <code>etc/ramfb</code> using [[QEMU_fw_cfg#DMA]]:
<syntaxhighlight lang="c">
struct RAMFBCfg {
    uint64_t addr;
    uint32_t fourcc;
    uint32_t flags;
    uint32_t width;
    uint32_t height;
    uint32_t stride;
};
</syntaxhighlight>
The field fourcc is a 4 letter code that identifies the pixelformat expected by the framebuffer. The available fourcc codes can be found in the  [https://github.com/qemu/qemu/blob/master/include/standard-headers/drm/drm_fourcc.h qemu source code]

After that the framebuffer will be mapped at the address specified in the addr field. Writing pixel data to it will immediately show the pixels on screen.

== Example ==
A minimal rust example would be:
<syntaxhighlight lang="rust">
use core::ffi::CStr;
use core::ptr::addr_of;
use core::mem;

#[repr(C)]
struct FWCfgFile {
    size: u32,
    select: u16,
    reserved: u16,
    name: [u8; 56]
}

#[repr(C, packed)]
struct FWCfgDmaAccess {
    control: u32,
    len: u32,
    addr: u64
}

#[repr(C, packed)]
struct RamFBCfg {
    addr: u64,
    fmt: u32,
    flags: u32,
    w: u32,
    h: u32,
    st: u32
}

const QEMU_CFG_DMA_CTL_READ:   u32 = 0x02;
const QEMU_CFG_DMA_CTL_SELECT: u32 = 0x08;
const QEMU_CFG_DMA_CTL_WRITE:  u32 = 0x10;

unsafe fn qemu_dma_transfer (control: u32, len: u32, addr: u64) {
    // Address of the DMA register on the aarch64 virt board
    let fw_cfg_dma: *mut u64 = 0x9020010 as *mut u64; 
    let dma = FWCfgDmaAccess {
        control: control.to_be(),
        len: len.to_be(),
        addr: addr.to_be()
    };
    unsafe {
        fw_cfg_dma.write_volatile((addr_of!(dma) as u64).to_be());
    }
    // Wait until DMA completed or error bit set
}

pub fn setup_ramfb(fb_addr: *mut u8, width: u32, height: u32) {
    let mut num_entries: u32 = 0;
    let fw_cfg_file_directory = 0x19;
    unsafe {
        qemu_dma_transfer((fw_cfg_file_directory << 16
                        | QEMU_CFG_DMA_CTL_SELECT 
                        | QEMU_CFG_DMA_CTL_READ) as u32,
                        mem::size_of::<u32>(),
                        addr_of!(num_entries) as u64);
    }

    // QEMU DMA is BE so need to byte swap arguments and results on LE
    num_entries = num_entries.to_be();

    let ramfb = FWCfgFile {
        size: 0,
        select: 0,
        reserved: 0,
        name: [0; 56]
    };

    for _ in 0..num_entries {
        unsafe {
            qemu_dma_transfer(QEMU_CFG_DMA_CTL_READ,
                            mem::size_of::<FWCfgFile>() as u32,
                            addr_of!(ramfb) as u64);
        }
        let entry = CStr::from_bytes_until_nul(&ramfb.name).unwrap();
        let entry = entry.to_str().unwrap();
        if entry == "etc/ramfb" {
            break;
        }
    }

    // See fourcc:
    // https://github.com/qemu/qemu/blob/54294b23e16dfaeb72e0ffa8b9f13ca8129edfce/include/standard-headers/drm/drm_fourcc.h#L188
    let pixel_format = ('R' as u32) | (('G' as u32) << 8) | 
    (('2' as u32) << 16) | (('4' as u32) << 24);

    // Stride 0 means QEMU calculates from bpp_of_format*width:
    // https://github.com/qemu/qemu/blob/54294b23e16dfaeb72e0ffa8b9f13ca8129edfce/hw/display/ramfb.c#L60
    let ramfb_cfg = RamFBCfg {
        addr: (fb_addr as u64).to_be(),
        fmt: (pixel_format).to_be(),
        flags: (0 as u32).to_be(),
        w: (width as u32).to_be(),
        h: (height as u32).to_be(),
        st: (0 as u32).to_be()
    };

    unsafe {
        qemu_dma_transfer((ramfb.select.to_be() as u32) << 16 
      | QEMU_CFG_DMA_CTL_SELECT 
      | QEMU_CFG_DMA_CTL_WRITE, mem::size_of::<RamFBCfg>() as u32,
        addr_of!(ramfb_cfg) as u64);
    }
}
</syntaxhighlight>

Now you should be able to write to the address you passed as a framebuffer and see the result on screen:
<syntaxhighlight lang="rust">
for x in 0..(stride*height) {
    fb_addr.add(x as usize).write_volatile(0xFF);
}
</syntaxhighlight>

This should produce a white screen.

== See Also ==
[[QEMU_fw_cfg#DMA]]

== External Links ==
[https://www.qemu.org/docs/master/specs/fw_cfg.html QEMU fw_cfg Documentation]

[https://github.com/qemu/qemu/blob/master/include/standard-headers/drm/drm_fourcc.h QEMU fourcc source]

[https://github.com/luickk/qemu-ramfb-aarch64-driver/tree/main Example code in C]

[https://github.com/arrynfr/salmon_salad/blob/efd026052eb214a8fe30159c36c153d4b3fe429e/src/driver/qemu/ramfb.rs Example code in rust]
[[Category:Video]]
````

<a id="w004854"></a>

## W004854 — Apple Screen Info

Bron-ID: W004854
Namespace: Main (0)
Revisie: 30230 | Datum: 2026-03-16T14:59:46Z
Bron: https://wiki.osdev.org/Apple_Screen_Info
Revisie-URL: https://wiki.osdev.org/index.php?title=Apple_Screen_Info&oldid=30230

````text
Before the introduction of [[GOP]] to Apple Macs in 2009, Apple used a proprietary protocol known as Apple Screen Info to serve graphics information. This is the best way to get framebuffer paramaters on any Mac made before 2009.

=== Definitions ===
These should be placed in a header file or in the main EFI program.

<syntaxhighlight lang="c">
#define APPLE_SCREEN_INFO_PROTOCOL_GUID {0xe316e100, 0x0751, 0x4c49, {0x90, 0x56, 0x48, 0x6c, 0x7e, 0x47, 0x29, 0x03}}

typedef struct _APPLE_SCREEN_INFO_PROTOCOL APPLE_SCREEN_INFO_PROTOCOL;

typedef EFI_STATUS (EFIAPI *GetAppleScreenInfo)(APPLE_SCREEN_INFO_PROTOCOL *This, UINT64 *BaseAddress, UINT64 *FrameBufferSize, UINT32 *BytesPerRow, UINT32 *Width, UINT32 *Height, UINT32 *Depth);

struct _APPLE_SCREEN_INFO_PROTOCOL {
    GetAppleScreenInfo GetInfo;
};
</syntaxhighlight>

=== Detection ===
<syntaxhighlight lang="c">
EFI_GUID AppleScreenInfoGuid = APPLE_SCREEN_INFO_PROTOCOL_GUID;
APPLE_SCREEN_INFO_PROTOCOL *AppleScreenInfo;
UINT64 BaseAddress, FrameBufferSize;
UINT32 BytesPerRow, Width, Height, Depth;

Status = uefi_call_wrapper(gBS->LocateProtocol, 3, &AppleScreenInfoGuid, NULL, (VOID **) &AppleScreenInfo); // leave out uefi_call_wrapper if using TianoCore
if (Status != EFI_SUCCESS)
{
    Print(L"Failed to find Apple Screen Info protocol! Status = %d\n", Status);
    return Status;
}
Status = uefi_call_wrapper(AppleScreenInfo->GetInfo, 7, AppleScreenInfo, &BaseAddress, &FrameBufferSize, &BytesPerRow, &Width, &Height, &Depth);
if (Status != UEFI_SUCCESS)
{
    Print(L"Failed to get Apple Screen Information! Status = %d\n", Status);
    return Status;
}
</syntaxhighlight>

With Apple Screen Info, the framebuffer is always assumed to be a 4bpp BGRA framebuffer.

[[Category:Video]]
````

<a id="w004855"></a>

## W004855 — User:Finxx

Bron-ID: W004855
Namespace: User (2)
Revisie: 30542 | Datum: 2026-05-02T03:40:13Z
Bron: https://wiki.osdev.org/User:Finxx
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AFinxx&oldid=30542

````text
Hi. I usually write articles while developing my OS to hopefully help others have an easier time than I did. I natively speak English and have too much free time on my hands, so feel free to ask me to rewrite something for you.

WIP Subpages:
* [[User:Finxx/Device Tree Blob]]
* [[User:Finxx/HFS+]]

WIP Mainpages:
* [[PowerPC Bare Bones]]

Complete:
* [[AArch64 Exceptions]]
````

<a id="w004856"></a>

## W004856 — User:Finxx/HFS+

Bron-ID: W004856
Namespace: User (2)
Revisie: 30543 | Datum: 2026-05-02T03:41:44Z
Bron: https://wiki.osdev.org/User:Finxx/HFS%2B
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AFinxx%2FHFS%2B&oldid=30543

````text
{{Filesystems}}
HFS+ is the successor to [[HFS]], and was introduced by Apple in Mac OS 8.1. It has support for larger disks and allows for more than 65,000 files. It also has support for [[journaling]]. The format is described in Apple's "legacy" documentation, as TN1150.

== Concepts ==
Files in HFS+ as stored in what are called "forks". Data is stored in "allocation blocks".
<syntaxhighlight lang="c">
struct HFSPlusExtentDescriptor {
    UInt32                  startBlock;
    UInt32                  blockCount;
};
typedef struct HFSPlusExtentDescriptor HFSPlusExtentDescriptor;

struct HFSPlusForkData {
    UInt64                  logicalSize;
    UInt32                  clumpSize;
    UInt32                  totalBlocks;
    HFSPlusExtentRecord     extents; // additional extents are in extent overflow
};
typedef struct HFSPlusForkData HFSPlusForkData;
 
typedef HFSPlusExtentDescriptor HFSPlusExtentRecord[8];
</syntaxhighlight>

== Structure ==
{| class="wikitable"
|-
! Offset !! Size !! Description
|-
| 0 || 2 || Reserved
|-
| 2 || 1 || Volume Header
|-
| 3 || ... || Files
|-
| Last - 1 || 1 || Volume Header backup
|-
| Last || 1 || Reserved
|-
|}

<syntaxhighlight lang="c">
struct HFSPlusVolumeHeader {
    UInt16              signature;
    UInt16              version;
    UInt32              attributes;
    UInt32              lastMountedVersion;
    UInt32              journalInfoBlock;
 
    UInt32              createDate;
    UInt32              modifyDate;
    UInt32              backupDate;
    UInt32              checkedDate;
 
    UInt32              fileCount;
    UInt32              folderCount;
 
    UInt32              blockSize;
    UInt32              totalBlocks;
    UInt32              freeBlocks;
 
    UInt32              nextAllocation;
    UInt32              rsrcClumpSize;
    UInt32              dataClumpSize;
    HFSCatalogNodeID    nextCatalogID;
 
    UInt32              writeCount;
    UInt64              encodingsBitmap;
 
    UInt32              finderInfo[8];
 
    HFSPlusForkData     allocationFile;
    HFSPlusForkData     extentsFile;
    HFSPlusForkData     catalogFile;
    HFSPlusForkData     attributesFile;
    HFSPlusForkData     startupFile;
};
typedef struct HFSPlusVolumeHeader HFSPlusVolumeHeader;
</syntaxhighlight>

== External Links ==
*[https://developer.apple.com/legacy/library/technotes/tn/tn1150.html#//apple_ref/doc/uid/DTS10002989 TN1150 - HFS Plus Volume Format]

[[Category:Filesystems]]
````

<a id="w004857"></a>

## W004857 — File:Xeneva2024March 5.png

Bron-ID: W004857
Namespace: File (6)
Revisie: 28718 | Datum: 2024-03-30T03:03:31Z
Bron: https://wiki.osdev.org/File:Xeneva2024March_5.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AXeneva2024March_5.png&oldid=28718

````text
Xeneva desktop screenshot
````

<a id="w004858"></a>

## W004858 — PowerPC Bare Bones

Bron-ID: W004858
Namespace: Main (0)
Revisie: 30383 | Datum: 2026-03-23T13:49:48Z
Bron: https://wiki.osdev.org/PowerPC_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=PowerPC_Bare_Bones&oldid=30383

````text
{{Stub}}
PowerPC is a RISC processor most notably used by Apple between the late 90s and mid 2000s. This article covers PowerPC computers that use the CHRP platform, being Macintosh's and RS/6000's.
== Boot Process ==
OpenFirmware, the BIOS, initializes the system and then starts a Forth console. It looks for a file called <code>/ppc/bootinfo.txt</code> in every partition it can read. OpenBIOS, the default implementation used by QEMU, supports Ext2, HFS, HFS+, and ISO9660. It is worth noting that OpenFirmware uses the Apple Partition Map (APM) instead of an MBR or GPT.
=== /ppc/bootinfo.txt ===
This is an SGML file that tells the BIOS how to load a bootloader on the same partition.
<syntaxhighlight lang=xml>
<CHRP-BOOT>
<DESCRIPTION>My Bootloader</DESCRIPTION>
<OS-NAME>My OS</OS-NAME>
<VERSION>1</VERSION>

<COMPATIBLE>
MacRISC MacRISC3 MacRISC4
</COMPATIBLE>
<BOOT-SCRIPT>
" screen" output
." test string" cr
</BOOT-SCRIPT>
</CHRP-BOOT>
</syntaxhighlight>
This bootinfo.txt should output "test string" to your console.
=== A Simple Infinite Loop ===
OpenFirmware requires a "boot" function to be defined that runs an executable. Add the following line to the boot script:
<pre>
boot &device;:&partition;,\ppc\boot.bin
</pre>
This will run a binary at <code>/ppc/boot.bin</code>. Next, assemble the following file and link it using a binutils targeting <code>powerpc-eabi</code>:
<syntaxhighlight lang=asm>
.globl _start
_start:
	b _start
</syntaxhighlight>
Place the resulting binary at <code>/ppc/boot.bin</code>, and you should get an output similar to this:
<pre>
Welcome to OpenBIOS v1.1 built on Mar 7 2023 22:21
Trying hd:,\\:tbxi...
Trying hd:,\ppc\bootinfo.txt...
test string
>> switching to new context:
</pre>
=== State Upon Entry ===
If you attach a debugger to QEMU and place a breakpoint on _start, you can see the state of the registers. In particular, only 2 have values:
{| class="wikitable"
!Register!!Value
|-
|%r1||Stack Pointer
|-
|%r5||OpenFirmware callback function
|}
We can move the callback around and pass it to a C function like this:
<syntaxhighlight lang=asm>
.globl _start
_start:
	mr %r3,%r5

	bl entry
</syntaxhighlight>
'''WARNING: Do not use main() as a function name in your kernel! GCC will try adding runtime initialization even if you use -ffreestanding and -nostdlib'''

We can then make a simple C file like this:
<syntaxhighlight lang=c>
int entry(int (*ofw)(void*)) {
	while (1) ;
}
</syntaxhighlight>
We now have another infinite loop, but in C this time.
== Interacting with OpenFirmware ==
In order to do anything with the computer, we need to figure out what hardware we have. We do this by asking OpenFirmware through that callback passed to our entry function. By passing the <code>peer</code> command with an argument of <code>0</code>, it gives us a handle to the root of the device tree.
=== Using the callback ===
The callback function takes an array of 32-bit numbers, consisting of a pointer to the command name, parameters, and spots for return values.
{| class="wikitable"
!Name!!Argument Count!!Return Count!!Arguments!!Return Values
|-
|"peer"||0x1||0x1||0x0||0x0
|}
Here is what that looks like in C source:
<syntaxhighlight lang=c>
#include <stdint.h>

extern int (*ofw)(void*);

struct of_peer_t {
	uint32_t name;
	uint32_t nargs;
	uint32_t nrets;
	uint32_t node;
	uint32_t retval;
};
uint32_t OF_peer(uint32_t node) {
	struct of_peer_t cmd = {
		.name = (uint32_t)"peer",
		.nargs = 1,
		.nrets = 1,
		.node = node,
		.retval = 0,
	};
	ofw(&cmd);
	return cmd.retval;
}
</syntaxhighlight>
Call the <code>OF_peer</code> function from your entrypoint, and you will get a handle to the root node. Of course, to do anything with this handle, you will need more OpenFirmware functions.
== External links ==
*[https://github.com/apple-oss-distributions/bootx The source code of the bootloader used by PowerPC version of Mac OS X]
*[https://www.openfirmware.info/data/docs/of1275.pdf The platform-independent OpenFirmware specification]
*[http://www.bitsavers.org/pdf/apple/powerpc/CHRP/chrp1_7a.pdf The PowerPC-ISA Supplement for OpenFirmware]
````

<a id="w004859"></a>

## W004859 — User:Moltony112

Bron-ID: W004859
Namespace: User (2)
Revisie: 28772 | Datum: 2024-04-27T05:06:41Z
Bron: https://wiki.osdev.org/User:Moltony112
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMoltony112&oldid=28772

````text

----

👋
````

<a id="w004860"></a>

## W004860 — User:Shifty

Bron-ID: W004860
Namespace: User (2)
Revisie: 28741 | Datum: 2024-04-17T19:45:41Z
Bron: https://wiki.osdev.org/User:Shifty
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AShifty&oldid=28741

````text
What if an idiot tried OSDev?
````

<a id="w004861"></a>

## W004861 — UACPI

Bron-ID: W004861
Namespace: Main (0)
Revisie: 30834 | Datum: 2026-07-01T00:11:53Z
Bron: https://wiki.osdev.org/UACPI
Revisie-URL: https://wiki.osdev.org/index.php?title=UACPI&oldid=30834

````text
{{ACPI}}

[https://github.com/UltraOS/uACPI '''uACPI'''] is a portable and easy-to-integrate implementation of the Advanced Configuration and Power Interface (ACPI).

Its main focuses are full NT compatibility, safety, and minimization of stack usage by avoiding recursion entirely.

Extra documentation can be found in the project's [https://github.com/UltraOS/uACPI GitHub repo].

== Features ==

* A complete & well-tested AML interpreter
* Operation region subsystem, support for user-defined address space handlers & builtin handlers for SystemMemory/SystemIO/PCI_Config/TableData
* User-defined device notify handlers & dispatch
* A complete resource subsystem. Every resource defined by ACPI 6.5 (last release) is supported
* Sleep API, allowing reset, transition to any sleep state, wake vector programming API 
* An advanced event subsystem, supporting GPE/fixed events, wake, implicit notify, AML handlers
* PCI routing table retrieval & interrupt model API
* Device search API

uACPI has been tested and confirmed to work on many real computers, both new and old, desktops and laptops.
It has also been extensively fuzzed against both real world AML blobs & its own test suite.

The test suite runs on every commit and tests multiple types of configuration, including 64 and 32 bit builds on Windows (MSVC), Linux (GCC), MacOS (AppleClang), and OpenWatcom.

=== Why use uACPI if ACPICA is a thing? ===

This is discussed in great detail in the project's [https://github.com/UltraOS/uACPI?tab=readme-ov-file#why-would-i-use-this-over-acpica README].

== Integrating into a kernel ==

Refer to the project's [https://github.com/UltraOS/uACPI?tab=readme-ov-file#integrating-into-a-kernel README] for detailed instructions on how to do it.

== Initialization ==

Below is an example of basic uACPI initialization sequence that enters ACPI mode, parses tables, brings the event system online, and finally loads & initializes the namespace.

<syntaxhighlight lang="c">
#include <uacpi/uacpi.h>
#include <uacpi/utilities.h>
#include <uacpi/event.h>

int acpi_init(void) {
    /*
     * Start with this as the first step of the initialization. This loads all
     * tables, brings the event subsystem online, and enters ACPI mode. We pass
     * in 0 as the flags as we don't want to override any default behavior for now.
     */
    uacpi_status ret = uacpi_initialize(0);
    if (uacpi_unlikely_error(ret)) {
        log_error("uacpi_initialize error: %s", uacpi_status_to_string(ret));
        return -ENODEV;
    }

    /*
     * Load the AML namespace. This feeds DSDT and all SSDTs to the interpreter
     * for execution.
     */
    ret = uacpi_namespace_load();
    if (uacpi_unlikely_error(ret)) {
        log_error("uacpi_namespace_load error: %s", uacpi_status_to_string(ret));
        return -ENODEV;
    }

    /*
     * Initialize the namespace. This calls all necessary _STA/_INI AML methods,
     * as well as _REG for registered operation region handlers.
     */
    ret = uacpi_namespace_initialize();
    if (uacpi_unlikely_error(ret)) {
        log_error("uacpi_namespace_initialize error: %s", uacpi_status_to_string(ret));
        return -ENODEV;
    }
    
    /*
     * Tell the firmware the interrupt model we're planning to use.
     * (Use UACPI_INTERRUPT_MODEL_PIC if you're planning to use PIC, or any
     *  other value depending on the architecture).
     */
    uacpi_set_interrupt_model(UACPI_INTERRUPT_MODEL_IOAPIC);

    /*
     * Tell uACPI that we have marked all GPEs we wanted for wake (even though we haven't
     * actually marked any, as we have no power management support right now). This is
     * needed to let uACPI enable all unmarked GPEs that have a corresponding AML handler.
     * These handlers are used by the firmware to dynamically execute AML code at runtime
     * to e.g. react to thermal events or device hotplug.
     */
    ret = uacpi_finalize_gpe_initialization();
    if (uacpi_unlikely_error(ret)) {
        log_error("uACPI GPE initialization error: %s", uacpi_status_to_string(ret));
        return -ENODEV;
    }

    /*
     * That's it, uACPI is now fully initialized and working! You can proceed to
     * using any public API at your discretion. The next recommended step is namespace
     * enumeration and device discovery so you can bind drivers to ACPI objects.
     */
    return 0;
}
</syntaxhighlight>

== Code examples ==
=== Namespace Enumeration & Finding Devices ===

There are multiple ways to implement device discovery for an ACPI namespace, below we will discuss the most common ways and their pros and cons.

==== Let Devices Discover Themselves ====

In this approach, we don't use a centralized bus system, but instead write ad-hoc find/discover() function for each supported device, then register it somewhere
so that it's called by kernel code during initialization.

<syntaxhighlight lang="c">
// ps2k.c
#include <uacpi/utilities.h>
#include <uacpi/resources.h>

#define PS2K_PNP_ID "PNP0303"

static uacpi_iteration_decision match_ps2k(void *user, uacpi_namespace_node *node)
{
    // Found a PS2 keyboard! Do initialization below.
    uacpi_resources *kb_res;

    uacpi_status ret = uacpi_get_current_resources(node, &kb_res);
    if (uacpi_unlikely_error(ret)) {
        log_error("unable to retrieve PS2K resources: %s", uacpi_status_to_string(ret));
        return UACPI_ITERATION_DECISION_NEXT_PEER;
    }

    // Parse the resources to find the IRQ and IO ports the keyboard is connected to
    // ...uacpi_for_each_resource()

    ps2k_create_device(...);

    uacpi_free_resources(kb_res);

    return UACPI_ITERATION_DECISION_CONTINUE;
}

void find_ps2_keyboard()
{
    uacpi_find_devices(PS2K_PNP_ID, match_ps2k, NULL);
}
</syntaxhighlight>

<syntaxhighlight lang="c">
// acpi_init.c
void find_acpi_devices(void) {
    find_ps2_keyboard();
    find_ps2_mouse();
    find_i2c();
    find_power_button();
    // ...and more
}
</syntaxhighlight>

As you can see it's a very simple approach, but it has lots of drawbacks:
* Very slow: we have to enumerate the entire namespace every time
* Binary bloat: more devices, more ad-hoc find methods
* Error-prone: more code duplication, more space for errors

==== Treat ACPI Namespace as a Bus ====

In this approach, we treat the ACPI namespace as a bus in our kernel, and let devices provide a way to identify them.

<syntaxhighlight lang="c">
// acpi_bus.h
#include <uacpi/uacpi.h>
#include <uacpi/namespace.h>
#include <uacpi/utilities.h>
#include <uacpi/resources.h>

struct acpi_driver {
    const char *device_name;
    const char *const *pnp_ids;
    int (*device_probe)(uacpi_namespace_node *node, uacpi_namespace_node_info *info);

    struct acpi_driver *next;    
};

void acpi_register_driver(struct acpi_driver *driver);
</syntaxhighlight>

<syntaxhighlight lang="c">
// ps2k.c
#include <acpi_bus.h>

#define PS2K_PNP_ID "PNP0303"

static const char *const ps2k_pnp_ids[] = {
    PS2K_PNP_ID,
    NULL,
};

static int ps2k_probe(uacpi_namespace_node *node, uacpi_namespace_node_info *info)
{
    uacpi_resources *kb_res;

    /* Parse the resources to find the IRQ and IO ports the keyboard is connected to
     * 
     * Note that for a centralized system like that the resources could be passed
     * to the device probe callback from common enumeration code at this point as
     * well!
     */
    uacpi_status st = uacpi_get_current_resources(node, &kb_res);
    if (uacpi_unlikely_error(st)) {
        log_error("unable to retrieve PS2K resources: %s", uacpi_status_to_string(st));
        return -ENODEV;
    }

    // Actually instantiate the device
    int ret = ps2k_create_device(...);

    uacpi_free_resources(kb_res);
    return ret;
}

static acpi_driver ps2k_driver = {
    .device_name = "PS2 Keyboard",
    .pnp_ids = ps2k_pnp_ids,
    .device_probe = ps2k_probe,
};

/*
 * This is called either manually by the kernel, or put in the linker script
 * in some known section, and executed by the kernel as part of driver initcalls.
 * If it's a dynamically loadable module, then this is called on module load.
 */
int ps2k_init(void)
{
    acpi_register_driver(&ps2k_driver);
    return 0;
}
</syntaxhighlight>

<syntaxhighlight lang="c">
// acpi_bus.c
#include <acpi_bus.h>

static struct acpi_driver *acpi_drivers_head

void acpi_register_driver(struct acpi_driver *driver)
{
    struct acpi_driver *next = acpi_drivers_head;
    acpi_drivers_head = driver;
    driver->next = next;
}

static uacpi_iteration_decision acpi_init_one_device(
    void *ctx, uacpi_namespace_node *node, uacpi_u32 node_depth
)
{
    uacpi_namespace_node_info *info;
    (void)node_depth;

    uacpi_status ret = uacpi_get_namespace_node_info(node, &info);
    if (uacpi_unlikely_error(ret)) {
        const char *path = uacpi_namespace_node_generate_absolute_path(node);
        log_error("unable to retrieve node %s information: %s",
                  path, uacpi_status_to_string(ret));
        uacpi_free_absolute_path(path);
        return UACPI_ITERATION_DECISION_CONTINUE;
    }

    struct acpi_driver *drv = NULL;

    if (info->flags & UACPI_NS_NODE_INFO_HAS_HID) {
        // Match the HID against every existing acpi_driver pnp id list
    }

    if (drv == NULL && (info->flags & UACPI_NS_NODE_INFO_HAS_CID)) {
        // Match the CID list against every existing acpi_driver pnp id list
    }

    if (drv != NULL) {
         // Probe the driver and do something with the error code if desired
         drv->device_probe(node, info);
    }

    uacpi_free_namespace_node_info(info);
    return UACPI_ITERATION_DECISION_CONTINUE;
}

void acpi_bus_enumerate()
{
    uacpi_namespace_for_each_child(
        uacpi_namespace_root(), acpi_init_one_device, UACPI_NULL,
        UACPI_OBJECT_DEVICE_BIT, UACPI_MAX_DEPTH_ANY, UACPI_NULL
    );
}
</syntaxhighlight>

As you can see above, this approach is more scalable, faster, and involves way less code duplication.
It does require a lot mode code and design to get going initially though.

=== Shutting Down the System === 
<syntaxhighlight lang="c">
#include <uacpi/sleep.h>

int system_shutdown(void) {
    /*
     * This helper essentially performs a two stage shutdown.
     * 
     * Stage 1:
     * - Runs the \_PTS & \_SST methods, if they exist
     * - Fetches the \_S5 and \_S0 values to make system wake possible later on
     *
     * Stage 2:
     * Actually enter the sleep state by writing to the hardware registers with
     * the values from stage 1. This also disables runtime events and enables
     * only those that are needed for wake.
     */
    ret = uacpi_enter_sleep_state_simple(UACPI_SLEEP_STATE_S5);
    if (uacpi_unlikely_error(ret)) {
        log_error("failed to enter sleep: %s", uacpi_status_to_string(ret));
        return -EIO;
    }

    /*
     * Technically unreachable code, but leave it here to prevent the compiler
     * from complaining.
     */
    return 0;
}
</syntaxhighlight>

=== Hooking Up the Power Button ===

The example below hooks up the power button press using a fixed event callback.

<syntaxhighlight lang="c">
#include <uacpi/event.h>

/*
 * This handler will be called by uACPI from an interrupt context,
 * whenever a power button press is detected.
 */
static uacpi_interrupt_ret handle_power_button(uacpi_handle ctx) {
    /*
     * Shut down right here using the helper we have defined above.
     *
     * Note that it's generally terrible practice to run any AML from
     * an interrupt handler, as it's allowed to allocate, map, sleep,
     * stall, acquire mutexes, etc. So, if possible in your kernel,
     * instead schedule the shutdown callback to be run in a normal
     * preemptible context later.
     */
    system_shutdown();
    return UACPI_INTERRUPT_HANDLED;
}

int power_button_init(void) {
    uacpi_status ret = uacpi_install_fixed_event_handler(
        UACPI_FIXED_EVENT_POWER_BUTTON,
	    handle_power_button, UACPI_NULL
    );
    if (uacpi_unlikely_error(ret)) {
        log_error("failed to install power button event callback: %s", uacpi_status_to_string(ret));
        return -ENODEV;
    }

    return 0;
}
</syntaxhighlight>

Note that some of the more modern hardware routes the power button in a more complicated way, via an embedded controller.

In order to hook that up you will need to:
* Write an [https://uefi.org/specs/ACPI/6.5/12_Embedded_Controller_Interface_Specification.html embedded controller] driver
* Find the EC device in the AML namespace (PNP ID "PNP0C09" or ECDT table), attach an address space handler
* Find a power button object in the namespace, attach a notify handler
* Detect the general purpose event number used by the embedded controller (_GPE method), install a handler for it
* In the event handler, execute the QR_EC command to find out the index of the query requested by the EC
* Run the requested query by executing the corresponding "EC._QXX" control method in AML
* If this query was for a power button press, you will receive a notifications with value 0x80 (S0 Power Button Pressed)

Refer to [https://github.com/managarm/managarm/blob/master/kernel/thor/system/acpi/ec.cpp managarm kernel EC driver] to see an example of how this may be done.

=== More Examples ===
You can look at the [https://github.com/managarm/managarm/tree/master/kernel/thor/system/acpi managarm kernel source] to see more examples of how you might use uACPI to implement various kernel features.

[[Category:ACPI]]
````

<a id="w004862"></a>

## W004862 — User:8infy

Bron-ID: W004862
Namespace: User (2)
Revisie: 28745 | Datum: 2024-04-23T18:31:27Z
Bron: https://wiki.osdev.org/User:8infy
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3A8infy&oldid=28745
Doorverwijzing: UACPI

````text
#REDIRECT [[UACPI]]
````

<a id="w004867"></a>

## W004867 — Talk:EPHERFS

Bron-ID: W004867
Namespace: Talk (1)
Revisie: 28827 | Datum: 2024-05-21T10:45:48Z
Bron: https://wiki.osdev.org/Talk:EPHERFS
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AEPHERFS&oldid=28827

````text
I would ask that you remove your hobby OS' filesystem from the wiki as it would be better to document it in your OS' repo and not on the wiki..

--[[User:Pitust|Pitust]] 03:02, 21 May 2024 (CDT)

I thought about it and realized that the file system is still raw, so I will delete this page and links to it from the wiki
````

<a id="w004869"></a>

## W004869 — SPARC Barebones

Bron-ID: W004869
Namespace: Main (0)
Revisie: 28834 | Datum: 2024-05-23T15:04:33Z
Bron: https://wiki.osdev.org/SPARC_Barebones
Revisie-URL: https://wiki.osdev.org/index.php?title=SPARC_Barebones&oldid=28834
Doorverwijzing: Sparc Barebones

````text
#REDIRECT [[Sparc Barebones]]
````

<a id="w004871"></a>

## W004871 — File:Vinix.png

Bron-ID: W004871
Namespace: File (6)
Revisie: 28871 | Datum: 2024-05-26T17:48:46Z
Bron: https://wiki.osdev.org/File:Vinix.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVinix.png&oldid=28871

````text
Example of Vinix running Xorg.
````

<a id="w004872"></a>

## W004872 — File:AstralFvwm3.png

Bron-ID: W004872
Namespace: File (6)
Revisie: 28873 | Datum: 2024-05-26T18:19:12Z
Bron: https://wiki.osdev.org/File:AstralFvwm3.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAstralFvwm3.png&oldid=28873

````text
Astral running Fvwm3 and other X.org applications
````

<a id="w004873"></a>

## W004873 — File:Merlon os.jpg

Bron-ID: W004873
Namespace: File (6)
Revisie: 28913 | Datum: 2024-06-01T02:44:06Z
Bron: https://wiki.osdev.org/File:Merlon_os.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AMerlon_os.jpg&oldid=28913

````text

````

<a id="w004881"></a>

## W004881 — Category:Wiki administration

Bron-ID: W004881
Namespace: Category (14)
Revisie: 28990 | Datum: 2024-06-06T17:25:18Z
Bron: https://wiki.osdev.org/Category:Wiki_administration
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AWiki_administration&oldid=28990

````text
Articles and categories regarding the administration of the wiki.

[[Category:Categories]]
````

<a id="w004883"></a>

## W004883 — Category:FAQ

Bron-ID: W004883
Namespace: Category (14)
Revisie: 29097 | Datum: 2024-06-07T13:39:26Z
Bron: https://wiki.osdev.org/Category:FAQ
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AFAQ&oldid=29097

````text
A list of Frequently Asked Questions. If your question is not on here, see the posting checklist for typical errors that cause many different problems.

[[Category:OS Development]]
````

<a id="w004885"></a>

## W004885 — User talk:Lukflug

Bron-ID: W004885
Namespace: User talk (3)
Revisie: 29025 | Datum: 2024-06-06T18:06:15Z
Bron: https://wiki.osdev.org/User_talk:Lukflug
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ALukflug&oldid=29025

````text
Please leave a message using the "add topic" button and sign comments with <nowiki>~~~~</nowiki>.

== Vandal complaining about me reverting their vandalism on June 6th 2024 ==

слыш ты верни ата пиво мод даун

<small class="autosigned">—&nbsp;Preceding unsigned comment added by [[User:Strokegmd|Strokegmd]] ([[User talk:Strokegmd#top|talk]] • [[Special:Contributions/Strokegmd|contribs]]) 17:40, 6 June 2024 (UTC)</small>
````

<a id="w004886"></a>

## W004886 — User:Xtex

Bron-ID: W004886
Namespace: User (2)
Revisie: 29030 | Datum: 2024-06-07T00:15:50Z
Bron: https://wiki.osdev.org/User:Xtex
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AXtex&oldid=29030

````text
See also: [https://minecraft.wiki/w/User:Xtex mcw:User:Xtex] and [https://xtexx.eu.org xtexx.eu.org].
````

<a id="w004887"></a>

## W004887 — Poweroff

Bron-ID: W004887
Namespace: Main (0)
Revisie: 29034 | Datum: 2024-06-07T00:26:21Z
Bron: https://wiki.osdev.org/Poweroff
Revisie-URL: https://wiki.osdev.org/index.php?title=Poweroff&oldid=29034
Doorverwijzing: Shutdown

````text
#REDIRECT [[Shutdown]]
````

<a id="w004888"></a>

## W004888 — Expanded Main Page

Bron-ID: W004888
Namespace: Main (0)
Revisie: 30560 | Datum: 2026-05-06T12:56:57Z
Bron: https://wiki.osdev.org/Expanded_Main_Page
Revisie-URL: https://wiki.osdev.org/index.php?title=Expanded_Main_Page&oldid=30560

````text
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
````

<a id="w004889"></a>

## W004889 — Tutorials

Bron-ID: W004889
Namespace: Main (0)
Revisie: 29583 | Datum: 2025-07-05T08:39:04Z
Bron: https://wiki.osdev.org/Tutorials
Revisie-URL: https://wiki.osdev.org/index.php?title=Tutorials&oldid=29583

````text
There are several OS development related tutorials on this wiki. This page is an overview of tutorials that are around, sorted by subject area and difficulty.

== Rating System ==
{{Ratingicon|1}} - Tutorials that are either very basic or easy to follow. Try these first.

{{Ratingicon|2}} - Tutorials on somewhat harder subjects, but still good to do. It may be a good idea to do some easier stuff first.

{{Ratingicon|3}} - Tutorials on advanced subjects. Not recommended for beginners.

{{Ratingicon|4}} - Tutorials on very difficult subjects. A master class. Good luck!

{{Ratingicon}} - Not rated yet.

== Kernel Basics ==

=== Bare Bones ===
These are "bare bones" tutorials that will land you with a basic kernel that's safe enough to use as a starting point for your own.

{{Ratingicon|1}} [[Bare Bones]] - Write a basic 32-bit kernel in C for x86

{{Ratingicon|1}} [[Limine Bare Bones]] - Write a simple 64-bit [[Higher Half Kernel|higher half kernel]] in C using the [[Limine]] bootloader and protocol.

{{Ratingicon|1}} [[User:Zesterer/Bare Bones]] - Write a basic 32-bit kernel in C for x86 (improved tutorial by zesterer)

{{Ratingicon|1}} [[Meaty Skeleton]] - Template operating system

{{Ratingicon|2}} [[Higher Half x86 Bare Bones]] - A tutorial that shows how to write a [[Higher Half Kernel|higher half kernel]]

{{Ratingicon|2}} [[Setting Up Long Mode]] - Switching to long mode

{{Ratingicon|2}} [[Creating a 64-bit kernel]] - An introduction to 64-bit kernels

{{Ratingicon|1}} [[Real mode assembly bare bones]] - A tutorial series on writing a basic assembly language kernel

=== Alternate Programming Languages ===
{{Ratingicon|2}} [[Ada Bare bones]] - A tutorial on writing a basic kernel in Ada

{{Ratingicon|2}} A [[C_Sharp_Bare_Bones|C# Bare Bones]] tutorial, using a language not typically used in OS development.

{{Ratingicon|2}} [[FreeBasic Bare Bones]] - A basic kernel in FreeBasic

{{Ratingicon|2}} [[Pascal Bare Bones]] - A basic kernel in Pascal

{{Ratingicon|3}} A [[Java Primer]] on dealing with languages in general and Java in particular that would normally be unsuitable for OS development.

=== Alternate ISAs and Platforms ===
{{Ratingicon|1}} [[GameBoy Advance Barebones]] - A tutorial on writing a basic GBA kernel

{{Ratingicon|2}} [[SPARC Barebones]] - A basic kernel for SPARCStations

== Basics ==
{{Ratingicon|1}} [[GDT Tutorial]] - A guide about the [[GDT]]

{{Ratingicon|1}} [[Interrupts tutorial]] - How set interrupts from C

{{Ratingicon|1}} [[Creating A Shell]] - A tutorial on how to write a [[Shell|shell]]

{{Ratingicon|2}} [[Going Further on x86]] - A guide that shall cover the basics of kernel internals

{{Ratingicon|3}} [[User:Johnburger/Demo|DEMO]] - A tutorial, in code and prose, on some fundamentals of the '386 (and above) and the PC platform, as an assembly language [[User:Johnburger/Demo|DEMO]]

== Memory Management ==
{{Ratingicon|1}} [[Setting Up Paging]] - A tutorial that deals with setting up and maintaining a system with paging enabled

{{Ratingicon|1}} [[Setting Up Paging With PAE]] - As above, but with PAE enabled

{{Ratingicon|1}} [[Brendan's Memory Management Guide]] - A memory management guide to explain basic concepts

{{Ratingicon|2}} [[Writing a memory manager]] - A tutorial on how to handle the RAM in a computer.

{{Ratingicon|2}} [[Writing A Page Frame Allocator]] - How to write a simple page frame allocator

== Processes and Threads ==
{{Ratingicon|2}} [[Brendan's Multi-tasking Tutorial]] - A kernel-space multitasking tutorial.

{{Ratingicon|3}} [[Cooperative Multitasking]] - How to create a kernel-space multitasking system.

{{Ratingicon|3}} [https://web.archive.org/web/20160326162854/http://xarnze.com/article/Entering%20User%20Mode Getting to User Mode]- How to context switch into user mode.

== Graphics & Video ==
{{Ratingicon|1}} [[Drawing In Protected Mode]] - The basics, how to plot a pixel.

{{Ratingicon|1}} [[Double Buffering]] - A handy way to prevent artifacts.

{{Ratingicon|1}} [[PC Screen Font]] - Displaying text with bitmap fonts.

{{Ratingicon|2}} [[Scalable Screen Font]] - Displaying text with vector fonts.

{{Ratingicon|2}} [[Loading Icons]] - Decoding image files to display icons.

== Booting ==
{{Ratingicon|1}} [[Bootable Disk]] - A tutorial that explains how to create a bootable disk (USB stick) image

{{Ratingicon|1}} [[Bootable CD]] - A tutorial that explains how to create a bootable CD

{{Ratingicon|1}} [[Bootable El-Torito CD with GRUB Legacy]] - A tutorial that explains how to create a bootable GRUB CD

{{Ratingicon|2}} [[Rolling Your Own Bootloader]] - Describes what steps to take when writing a bootloader.

{{Ratingicon|1}} [[Babystep1|Writing a bootloader]] - A basic tutorial on creating a bootloader

{{Ratingicon|2}} [[uefi.inc|Writing a bootloader for UEFI]] - A basic tutorial on creating a bootloader utilising UEFI

{{Ratingicon|3}} [[Writing GRUB Modules]] - A tutorial on writing modules that add custom functionality to GRUB.

== Building ==
{{Ratingicon|1}} [[Makefile]] - A guided demonstration of how Makefiles can be used

{{Ratingicon|3}} [[OS Specific Toolchain]] - A guide on adapting GCC and Binutils to a platform

{{Ratingicon|1}} [[CMake Build System]] - A guide demonstrating adapting KitWare's CMake Build System for building an operating system.

{{Ratingicon|1}} [[User:TheCool1Kevin/VSCode_Debug|VSCode for Debugging]] - Setting up VSCode for debugging your kernel.

== Compilers ==
{{Ratingicon|1}} [[GCC Cross-Compiler]] - A guide that helps build GCC targeting a different platform

{{Ratingicon|1}} [[GDC Cross-Compiler]] - Same as the previous, but this time for the D programming language.

== Executable File Formats == 

{{Ratingicon|2}} [[ELF Tutorial]] - A guide that details the process of loading ELF executables.

{{Ratingicon|3}} [https://web.archive.org/web/20140130143820/http://robinhoksbergen.com/papers/howto_elf.html Manually Creating an ELF Executable] - A guide that demonstrates how ELF binaries work, and how to build one from scratch using only a hex editor.

== Porting Software ==
{{Ratingicon|2}} [[Porting Newlib]] - A guide on porting a common C library to another operating system

{{Ratingicon}} [[Libsupcxx|Using Libsupc++]] - A guide on porting libsupc++ to get more out of the features of C++

{{Ratingicon|4}} [[Porting Python]] - A guide on porting python to another operating system

== Third Party Tutorials ==

'''Note:''' This section lists tutorials often mentioned in discussions, but not affiliated with this wiki. 
Due to the vast number of tutorials available, this list is not comprehensive. 

<span style="color:#FF0000">
'''Caution:''' Quality varies greatly among these tutorials. Since they're external, the wiki authors 
cannot guarantee their accuracy or updates. While they can offer additional insight, it's '''strongly advised'''
to use them as supplementary resources and to approach them with skepticism.
</span>

{{Ratingicon}} [http://jamesmolloy.co.uk/tutorial_html/ James A. Molloy's Kernel Tutorials] - one of the more popular tutorials in the past, it has a number of [[James Molloy's Tutorial Known Bugs|known issues]], and does not seem to be actively updating. [https://github.com/Exclavia/Kernel-Dev/ Tutorials archived] on GitHub due to original website no longer being accessible.

{{Ratingicon}} [http://www.brokenthorn.com/Resources/OSDevIndex.html BrokenThorn Operating System Development Series] - Like the James Molloy series, this is a very well-known tutorial series, but one which is very dated and has a large number of [[Brokenthorn's Known Bugs|known flaws]] that have not been corrected.

{{Ratingicon}} [http://www.osdever.net/tutorials/view/brans-kernel-development-tutorial Bran's Kernel Tutorial] - a very dated, but still often referenced, tutorial from the now-moribund "Bona Fide OS Development" site. Like JAM and BrokenThorn, the code examples have many [[Bran's Known Bugs|known problems]], and much of the material is long out of date.

{{Ratingicon}} [http://mikeos.sourceforge.net/write-your-own-os.html How to write a simple operating system] by Mike Saunders - this is the starting point for those following the MikeOS project, an x86 real-mode system written in assembly language. and focuses on the aspects needed to get going with developing for MikeOS.

{{Ratingicon|1}} [https://www.youtube.com/playlist?list=PLKbvCgwMcH7BX6Z8Bk1EuFwDa0WGkMnrz FlingOS Getting Started video series] - A third party series of video tutorials giving a practical start to writing your first OS (aimed at x86, full examples available in each of ASM, C and C#)

{{Ratingicon}} [https://github.com/littleosbook/littleosbook The Little OS Book] - a third-party OS demonstrator hosted on GitHub. Goes through periods of updating, and known bugs which haven't yet been fixed are listed in the repo.

{{Ratingicon|1}} [https://www.osnews.com/story.php/1482/So_You_Want_to_Write_an_Operating_System So, You Want to Write an Operating System] and [https://www.osnews.com/story/1532/ Climbing the Kernel Mountain] - a now-ancient series of articles from the OS News website, begun in 2002, these were many older developers' introductions to OS dev. They are well-written, but have only cursory coverage of the details, and are primarily of only historical interest today. This is included solely because they are referenced in many older posts in the forum. Note that the author later wrote a [https://www.osnews.com/story/8162 follow-up] in which he argued against developing a new kernel at all.

{{Ratingicon|1}} [[Xv6]] unlike most tutorials in this list, this is a fully functional, yet simple OS. Xv6 is a modernized version of the classic Dennis Richie's and Ken Thompson's UNIX V6, written in ANSI C for the x86 protected mode, keeping the original UNIX philosophy of simplicity.

{{Ratingicon|1}} [https://github.com/gmarino2048/64bit-os-tutorial 64-bit OS Tutorial] - an easy-to-follow, straightforward, third-party tutorial by Guy Marino, Noah Houpt, and Steven Nyeo. The end operating system has a simple kernel with minimal functionality, but is nonetheless a good starting point. However, the abstract concepts aren't explained in much depth. The owner (Guy Marino) is fairly active, and currently committing regularly. It's not as expansive as the resources in this wiki, but if you're really impatient to get something up and running, then feel free!

{{Ratingicon|1}} [https://0xc0ffee.netlify.app/osdev/ Fusion OS - Writing a single-address space operating system in Nim] by Khaled Hammouda. Starts from scratch with a UEFI bootloader, and goes into details of writing a higher half kernel with physical and virtual memory management, interrupts, context switching, system calls, user mode tasks, ELF loading, position independent code, and more.

[[Category:Tutorials]]
````

<a id="w004890"></a>

## W004890 — Module:Flex columns/styles.css

Bron-ID: W004890
Namespace: Module (828)
Revisie: 29134 | Datum: 2024-06-22T22:38:07Z
Bron: https://wiki.osdev.org/Module:Flex_columns/styles.css
Revisie-URL: https://wiki.osdev.org/index.php?title=Module%3AFlex_columns%2Fstyles.css&oldid=29134

````text
.flex-columns-container {
	clear: both;
	width: 100%;
	display: flex;
	flex-wrap: wrap;
}

.flex-columns-container > .flex-columns-column {
	float: left;
	width: 50%;
	min-width: 360px;
	padding: 0 0.5em;
	box-sizing: border-box;
	flex: 1;
	display: flex;
	flex-direction: column;
}

@media screen and (max-width: 393px) {
	.flex-columns-container > .flex-columns-column {
		min-width: 0;
	}
}

.flex-columns-container > .flex-columns-column:first-child {
	padding-left: 0;
}

.flex-columns-container > .flex-columns-column:last-child {
	padding-right: 0;
}

@media screen and (max-width: 720px) {
	.flex-columns-container > .flex-columns-column {
		padding: 0;
		width: 100%;
	}
	
	.flex-columns-container {
		display: block;
	}
}

.flex-columns-container > .flex-columns-column > div {
	flex: 1 0 auto;
}

.flex-columns-container > .flex-columns-column > div.flex-columns-noflex {
	flex: 0;
}
````

<a id="w004891"></a>

## W004891 — Module:Flex columns

Bron-ID: W004891
Namespace: Module (828)
Revisie: 29135 | Datum: 2024-06-22T22:39:18Z
Bron: https://wiki.osdev.org/Module:Flex_columns
Revisie-URL: https://wiki.osdev.org/index.php?title=Module%3AFlex_columns&oldid=29135

````text
local p = {}

local function setCleanArgs(argsTable)
	local cleanArgs = {}
	for key, val in pairs(argsTable) do
		if type(val) == 'string' then
			val = val:match('^%s*(.-)%s*$')
			if val ~= '' then
				cleanArgs[key] = val
			end
		else
			cleanArgs[key] = val
		end
	end
	return cleanArgs
end

p.main = function(frame)
	local parent = frame.getParent(frame)
	local output = p._main(parent.args)
	return frame:extensionTag{
		name='templatestyles', args = { src='Module:Flex columns/styles.css'}
	} .. frame:preprocess(output)
end

p._main = function(_args)
	local args = setCleanArgs(_args)
	local ii = 1
	local container = mw.html.create('div')
	:addClass('flex-columns-container' )
	while args[ii] do
		local column = container:tag('div')
		:addClass('flex-columns-column' )
		:wikitext(args[ii])
		if args['flex'..ii] then
			column:css('flex', args['flex'..ii])
		end
		ii = ii + 1
	end
	return tostring(container)
end

return p
````

<a id="w004892"></a>

## W004892 — Template:Flex columns

Bron-ID: W004892
Namespace: Template (10)
Revisie: 29136 | Datum: 2024-06-22T22:42:20Z
Bron: https://wiki.osdev.org/Template:Flex_columns
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AFlex_columns&oldid=29136

````text
{{#invoke:Flex columns|main}}
````

<a id="w004893"></a>

## W004893 — User:Jamix

Bron-ID: W004893
Namespace: User (2)
Revisie: 30007 | Datum: 2026-02-14T22:55:54Z
Bron: https://wiki.osdev.org/User:Jamix
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJamix&oldid=30007

````text
Hey, I'm Jamix!

The operating system kernel I am currently working on is the [https://github.com/zanxz-lol/neonix.git NEONIX operating system]. It is a 64-bit UEFI-only operating system that aims to be YOUR main operating system! (In the future)

== Recommendations for new OSDevers ==
Please note that these recommendations will definitely not be the best, since at the time of me writing this, I only started a year ago.

* Do NOT develop your Operating System when you're tired. You are at your slowest when you are tired, and it's always good to take breaks every now and then.
* Please, for the love of Jesus Christ, do NOT copy and paste code. I've been there, and it's not fun. You wont know what goes on in your code and if an error happens in the copy-pasted code, you wont know how to fix it!
* Build your operating system on a Unix-like system (for example, Linux, FreeBSD, OpenBSD, etc). It's way harder on a Windows system than a Unix-like system.
* Do not give up. You will eventually find the solution to your problem.
* Never use AI, I saw a post on the OSDev reddit of a dude who (very obviously) used AI to write the kernel. Dude even said, and I quote, "C'mon, ChatGPT can't make a 600 lines complex kernel code."
* Start off creating your OS as 32-bit. It's easier, and doesn't give as much headaches from a 64-bit OS.


That's pretty much all I gotta say for now. 

Cya.
````

<a id="w004894"></a>

## W004894 — PCIe

Bron-ID: W004894
Namespace: Main (0)
Revisie: 29163 | Datum: 2024-06-28T08:24:01Z
Bron: https://wiki.osdev.org/PCIe
Revisie-URL: https://wiki.osdev.org/index.php?title=PCIe&oldid=29163
Doorverwijzing: PCI Express

````text
#REDIRECT: [[PCI Express]]
````

<a id="w004895"></a>

## W004895 — Talk:Limine Bare Bones

Bron-ID: W004895
Namespace: Talk (1)
Revisie: 29169 | Datum: 2024-06-29T15:04:05Z
Bron: https://wiki.osdev.org/Talk:Limine_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ALimine_Bare_Bones&oldid=29169

````text
Hi [[User:Pitust]] any reason for switching the printf from hex to octal? Its a 3 so I get that it doesn't really matter but octal is less commonly used. https://wiki.osdev.org/index.php?title=Limine_Bare_Bones&type=revision&diff=29164&oldid=29123 You said "Fix printf escape code for portability", is octal more portable in some way?
: I believe this is to improve portability. Hexadecimal is not supported everywhere. Apparently Debian's dash doesn't support it. --[[User:Lukflug|Lukflug]] ([[User talk:Lukflug|talk]]) 15:04, 29 June 2024 (UTC)
````

<a id="w004896"></a>

## W004896 — User:Finxx/Device Tree Blob

Bron-ID: W004896
Namespace: User (2)
Revisie: 30227 | Datum: 2026-03-16T14:55:04Z
Bron: https://wiki.osdev.org/User:Finxx/Device_Tree_Blob
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AFinxx%2FDevice_Tree_Blob&oldid=30227

````text
{{Stub}}
{{In_Progress}}
A Device Tree Blob (DTB) is a binary representation of a [[Devicetree|Flattened Device Tree]] that is passed to your kernel.

== Location ==
The location in memory of the DTB is dependent on the motherboard, but it is generally loaded at the start of memory.
{| class="wikitable"
|+ Method of locating the DTB
|-
! CPU !! Motherboard !! Boot Method !! Location
|-
| Arm || Virt || ELF Kernel || Pointer in register R2
|-
| Arm || Virt || Bare-Metal || 0x40000000 
|-
| AArch64 || Virt || ELF Kernel || Pointer in register X0
|-
| AArch64 || Virt || Bare-Metal || 0x40000000
|-
| Risc-V || Virt || ELF Kernel || Pointer in register A1
|}
Please expand this table if you know the correct values!

== Structure ==
All integers are stored in big endian.

<syntaxhighlight lang="c">
typedef struct {
	uint32_t magic; /* 0xD00DFEED */
	uint32_t size; /* includes header and data */
	uint32_t struct_offset; /* offset of struct block */
	uint32_t strings_offset; /* offset of strings block */
	uint32_t memory_offset; /* offset of memory block */
	uint32_t version; /* version of dtb (17) */
	uint32_t min_version; /* minimum version (16) */
	uint32_t cpu_id; /* id of boot cpu */
	uint32_t strings_size; /* size of strings block */
	uint32_t struct_size; /* size of struct block */
} dtb_header;
</syntaxhighlight>
Todo
````

<a id="w004897"></a>

## W004897 — File:Hypnoticos-screenshot.png

Bron-ID: W004897
Namespace: File (6)
Revisie: 29180 | Datum: 2024-07-13T18:08:07Z
Bron: https://wiki.osdev.org/File:Hypnoticos-screenshot.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AHypnoticos-screenshot.png&oldid=29180

````text

````

<a id="w004898"></a>

## W004898 — File:Amethyst-Screenshot.png

Bron-ID: W004898
Namespace: File (6)
Revisie: 29230 | Datum: 2024-08-22T03:26:34Z
Bron: https://wiki.osdev.org/File:Amethyst-Screenshot.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAmethyst-Screenshot.png&oldid=29230

````text
Screenshot of Amethyst operating system by Jack Scott
````

<a id="w004900"></a>

## W004900 — User:Finxx/PowerPC Bare Bones

Bron-ID: W004900
Namespace: User (2)
Revisie: 29267 | Datum: 2024-10-06T18:50:04Z
Bron: https://wiki.osdev.org/User:Finxx/PowerPC_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AFinxx%2FPowerPC_Bare_Bones&oldid=29267
Doorverwijzing: PowerPC Bare Bones

````text
#REDIRECT [[PowerPC Bare Bones]]
````

<a id="w004901"></a>

## W004901 — User:Adamonair

Bron-ID: W004901
Namespace: User (2)
Revisie: 29516 | Datum: 2025-05-14T20:16:31Z
Bron: https://wiki.osdev.org/User:Adamonair
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAdamonair&oldid=29516

````text
I code so good even the compiler thanks me.
````

<a id="w004902"></a>

## W004902 — File:LaylaOS.png

Bron-ID: W004902
Namespace: File (6)
Revisie: 29286 | Datum: 2024-11-01T10:27:26Z
Bron: https://wiki.osdev.org/File:LaylaOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALaylaOS.png&oldid=29286

````text
== Summary ==
LaylaOS screenshot
````

<a id="w004903"></a>

## W004903 — Libreboot

Bron-ID: W004903
Namespace: Main (0)
Revisie: 30563 | Datum: 2026-05-06T13:30:09Z
Bron: https://wiki.osdev.org/Libreboot
Revisie-URL: https://wiki.osdev.org/index.php?title=Libreboot&oldid=30563

````text

[[Category:Firmware]]
[[Category:UEFI]]
[[Category:BIOS]]
[[Category:x86]]
[[Category:ARM]]
[[Category:Stubs]]
{{Stub}}

The '''Libreboot''' project provides free, open source (libre) boot firmware based on coreboot, replacing proprietary [[BIOS]]/[[UEFI]] firmware on specific Intel/AMD x86 and ARM based motherboards.

===A note on AMD support===
AMD is not very well supported by Libreboot and [[Coreboot]] due to its usage of the propriatary [[AGESA]], [[openSIL]] is planned to replace AGESA permanently by the AMD Zen 6 release. OpenSIL is not planned to be officially backported to earlier AMD CPUs, but has been shown to work on systems as old as Zen1.

==See Also==
===External Links===
*[https://libreboot.org/ libreboot homepage] - Contains extra information on Libreboot
*[https://coreboot.org/ coreboot homepage] - What libreboot is based on.
*[https://github.com/openSIL/openSIL openSIL] - The future replacement for AGESA.
````

<a id="w004904"></a>

## W004904 — User:Restingwitchface

Bron-ID: W004904
Namespace: User (2)
Revisie: 29296 | Datum: 2024-11-17T18:56:22Z
Bron: https://wiki.osdev.org/User:Restingwitchface
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARestingwitchface&oldid=29296

````text
OSDever. Currently making a [[Modular Kernel|modular]] [[exokernel]] with a BSD-like userland, but currently fed up with the constant [[Exceptions#General Protection Fault|GPFs]].
````

<a id="w004906"></a>

## W004906 — User:Partisani

Bron-ID: W004906
Namespace: User (2)
Revisie: 29298 | Datum: 2024-11-18T17:13:17Z
Bron: https://wiki.osdev.org/User:Partisani
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APartisani&oldid=29298

````text
A programmer that always gets sidetracked very easily
````

<a id="w004907"></a>

## W004907 — Multithreaded Kernel

Bron-ID: W004907
Namespace: Main (0)
Revisie: 29495 | Datum: 2025-04-27T00:25:46Z
Bron: https://wiki.osdev.org/Multithreaded_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Multithreaded_Kernel&oldid=29495

````text
A multithreaded kernel is an operating system kernel that manages multiple threads, where each thread typically represents a function or application with its own stack and saved registers. When switching between threads, the kernel saves the current thread's state (i.e., registers and stack) and then loads the new thread's state, including the new stack, registers, and the Instruction Pointer, allowing the new thread to resume execution where it left off.{{In Progress}}

== Types of multithreaded kernel ==
There are different types of multithreaded kernels, each relying on the specific design of the system's scheduler and how it handles task switching.

=== Cooperative Kernel ===
This is the type of app execution on macOS System Software 1.x to 9.x, and on Windows 1.0 through 3.1/3.11. Windows 95 through ME used cooperative multitasking for 16-bit apps, but preemptive multitasking for 32-bit ones.

Cooperative multitasking works via giving an app full control over a CPU core, and when it is ready to give control to the next thread, which on single core systems (most computers before 2000) is the full CPU which means any app could on accident not yield control, or do it on purpose as a form of ransomware/malware.

=== Preemptive Multitasking ===
This is the type of app execution on modern operating systems such as macOS 10.0 and later, Windows NT-based systems (32bit), and all versions of Linux.

These systems switch between threads after a set amount of time, for example every 10 milliseconds. Lets say the PIT is used, every 10 ticks (if configured to tick every millisecond) then switch to the next thread, this would be a very simple implementation of cooperative multitasking.
````

<a id="w004908"></a>

## W004908 — User:Gravaera/Building cross compiler with freestanding libsupc++ and libcpp

Bron-ID: W004908
Namespace: User (2)
Revisie: 29337 | Datum: 2025-01-02T19:57:47Z
Bron: https://wiki.osdev.org/User:Gravaera/Building_cross_compiler_with_freestanding_libsupc%2B%2B_and_libcpp
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FBuilding_cross_compiler_with_freestanding_libsupc%2B%2B_and_libcpp&oldid=29337

````text
It's simple really:<syntaxhighlight lang="bash" line="1">
$ ../gcc-14.2.0/configure --target=i586-elf --disable-nls \
    --enable-languages=c,c++ --prefix=$(HOME)/bin/i586-elf \
    --without-headers --with-static-standard-libraries \
    --disable-hosted-libstdcxx --enable-libstdcxx-static-eh-pool \
    --with-libstdcxx-eh-pool-obj-count=64
$ make -j8 all-gcc all-target-libgcc all-target-libstdc++-v3
</syntaxhighlight>

* --with-static-standard-libraries: Tells GCC's configure to build libgcc and libstdc++ as static libs (*.a, *.la).
* --disable-hosted-libstdcxx: This is the magic that tells GCC's build system to build for a freestanding target.
* --enable-libstdcxx-static-eh-pool: This is useful because it makes libsupc++ manage a static pool of memory which it uses for holding exceptions if malloc() fails. I.e: it *should* enable your kernel to use exceptions early on in the boot process, before the memory manager has been initialized.
* --with-libstdcxx-eh-pool-obj-count=64: This tells libsupc++ how much static memory to allocate for early exception handling.

=== Other interesting and potentially useful flags: ===

* --disable-tls --disable-linux-futex
* --disable-libstdcxx-threads
* --disable-libstdcxx-filesystem-ts
* --disable-libstdcxx-backtrace
* --enable-libstdcxx-static-eh-pool
* --enable-version-specific-runtime-libs


But still no libcrt. Working on that now.
````

<a id="w004909"></a>

## W004909 — Talk:Interrupts Tutorial

Bron-ID: W004909
Namespace: Talk (1)
Revisie: 29342 | Datum: 2025-01-21T12:24:41Z
Bron: https://wiki.osdev.org/Talk:Interrupts_Tutorial
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AInterrupts_Tutorial&oldid=29342

````text
* '''Uses MIT license''' - This uses the MIT license but the stuff meant to be on here is supposed to be CC0 that work is copyrighted since this pages code is based on it. -- [[User:Anotheridiot|Anotheridiot]] 07:24 AM, 21 January 2025 (UTC)
````

<a id="w004911"></a>

## W004911 — File:Windows BSOD.png

Bron-ID: W004911
Namespace: File (6)
Revisie: 29346 | Datum: 2025-01-23T16:58:01Z
Bron: https://wiki.osdev.org/File:Windows_BSOD.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AWindows_BSOD.png&oldid=29346

````text
A blue screen of death.
````

<a id="w004912"></a>

## W004912 — User:Meownium

Bron-ID: W004912
Namespace: User (2)
Revisie: 29354 | Datum: 2025-01-29T14:20:29Z
Bron: https://wiki.osdev.org/User:Meownium
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMeownium&oldid=29354

````text
I do stuff from time to time. Right now, I'm writing an OS in Ada.
https://github.com/ATPStorages/ufedmi
````

<a id="w004913"></a>

## W004913 — User:Meownium/Ada Bare Bones

Bron-ID: W004913
Namespace: User (2)
Revisie: 29355 | Datum: 2025-01-29T14:35:03Z
Bron: https://wiki.osdev.org/User:Meownium/Ada_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMeownium%2FAda_Bare_Bones&oldid=29355

````text
{{BeginnersWarning}}
{{Rating|1}}
{{Template:Kernel designs}}
''This draft is for a replacement of the [[Ada Bare Bones]] page, as it is outdated, and does not take advantages modern GCC/GNAT have, such as the ability to cross-compile with the GPR entry '''Target'''. It will be merged into the original article upon it's completion; however: I haven't any experience with making an RPi kernel; I'll have to defer that a bit later.''
````

<a id="w004914"></a>

## W004914 — User:Demindiro/Super Tiny UEFI Hello World

Bron-ID: W004914
Namespace: User (2)
Revisie: 30287 | Datum: 2026-03-16T18:58:02Z
Bron: https://wiki.osdev.org/User:Demindiro/Super_Tiny_UEFI_Hello_World
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADemindiro%2FSuper_Tiny_UEFI_Hello_World&oldid=30287

````text
=== GAS + Python ===

==== build.sh ====

<syntaxhighlight lang="sh">
#!/bin/sh

set -xe

as --64 -o main.o main.s
ld --oformat binary -o main.bin main.o
python3 uefi.py main.bin bootx64.efi

fallocate -l512K efi-fs.img
mformat -i efi-fs.img
mmd     -i efi-fs.img             ::/efi ::/efi/boot
mcopy   -i efi-fs.img bootx64.efi ::/efi/boot/bootx64.efi
#mcopy   -i efi-fs.img edk2/Build/EmulatorX64/DEBUG_GCC/X64/HelloWorld.efi ::/efi/boot/bootx64.efi
mdir    -i efi-fs.img -/ -w -a

fallocate -l1M uefi-test.img
cat <<CMD | /sbin/fdisk uefi-test.img
g

n





t

uefi


w

CMD
/sbin/fdisk -l uefi-test.img

dd if=efi-fs.img of=uefi-test.img bs=512 oseek=34 conv=notrunc

/sbin/fdisk -l uefi-test.img
</syntaxhighlight>

==== main.s ====

<syntaxhighlight lang="asm">
.intel_syntax noprefix
.section .text
.globl _start

.equ EFI_HANDLE.sizeof, 8 # VOID*

# 4.2.1
.equ EFI_TABLE_HEADER.sizeof, 8 + (4 * 4)

# 4.3.1
.equ EFI_SYSTEM_TABLE.Hdr             , 0 # EFI_TABLE_HEADER
.equ EFI_SYSTEM_TABLE.FirmwareVendor  , 24 # CHAR16
.equ EFI_SYSTEM_TABLE.FirmwareRevision, 32 # UINT32
.equ EFI_SYSTEM_TABLE.ConsoleInHandle , 40 # EFI_HANDLE
.equ EFI_SYSTEM_TABLE.ConIn           , 48 # EFI_SIMPLE_TEXT_INPUT_PROTOCOL
.equ EFI_SYSTEM_TABLE.ConsoleOutHandle, 56 # EFI_HANDLE
.equ EFI_SYSTEM_TABLE.ConOut          , 64 # EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL
# there's more but idc

# 12.4.1
.equ EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL.Reset       , 0 # EFI_TEXT_RESET
.equ EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL.OutputString, 8 # EFI_TEXT_STRING

# rcx: EFI_HANDLE (of ourselves, just ignore)
# rdx: EFI_SYSTEM_TABLE*
_start:
	mov rax, [rdx + EFI_SYSTEM_TABLE.ConOut]
	mov rbx, [rax + EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL.OutputString]

	# 12.4.3
	# typedef EFI_STATUS (EFIAPI *EFI_TEXT_STRING) (
	#   IN EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL *This
	#   IN CHAR16 *String
	# );
	# I will _never_ forgive Microsoft for being so stubborn with (not even!) UTF-16
	# instead of at least trying to use the actually sane UTF-8 standard
	mov rcx, rax
	lea rdx, [rip + hello_uefi]
	#call break
	call rbx

2:
	hlt
	jmp 2b

break:
	xor r15, r15
2:
	test r15, r15
	jz 2b
	ret

hello_uefi:
	.word 'H', 'e', 'l', 'l', 'o', ' ', 'U', 'E', 'F', 'I', '!', 0
</syntaxhighlight>

==== uefi.py ====

<syntaxhighlight lang="python">
#!/usr/bin/env python3

import sys, os

# note that no stub program is included!
MSDOS_MZ_HEADER = (b'MZ' + (b'\0' * (64 - 2 - 4))) + \
    b'\x40\x00\x00\x00' # 0x40 = 64 bytes = start of PE header


PE_HEADER = b''.join((
    b'PE\0\0', # magic
    b'\x64\x86', # machine (EFI_IMAGE_MACHINE_X64)
    b'\1\0', # number of sections
    b'\0\0\0\0', # timedatestamp
    b'\0\0\0\0', # pointer to symbol table
    b'\0\0\0\0', # number of symbols
    b'\x70\0', # size of optional header (160 bytes)
    b'\x2e\x00', # characteristics (IMAGE_FILE_EXECUTABLE_IMAGE, IMAGE_FILE_LINE_NUMS_STRIPPED, IMAGE_FILE_LOCAL_SYMS_STRIPPED, IMAGE_FILE_LARGE_ADDRESS_AWARE)
))

def pe32_optional_header(
    *,
    size_of_code,
    size_of_initdata,
    size_of_uninitdata,
    entry_address,
    base_of_code,
    size_of_image,
    size_of_headers,
):
    return b''.join([
        b'\x0b\x02', # magic (PE32+)
        b'\0\0', # linker version
        size_of_code.to_bytes(4, 'little'),
        size_of_initdata.to_bytes(4, 'little'),
        size_of_uninitdata.to_bytes(4, 'little'),
        entry_address.to_bytes(4, 'little'),
        base_of_code.to_bytes(4, 'little'),
        b'\0\0\0\0\0\0\0\0', # image base
        b'\x00\x10\x00\x00', # section alignment (4096)
        #b'\x00\x02\x00\x00', # file alignment (512)
        b'\x00\x10\x00\x00', # file alignment (4096)
        b'\0\0\0\0', # OS version
        b'\0\0\0\0', # image version
        b'\0\0\0\0', # subsystem version
        b'\0\0\0\0', # win32 version
        size_of_image.to_bytes(4, 'little'),
        size_of_headers.to_bytes(4, 'little'),
        b'\0\0\0\0', # checksum (N/A since IMAGE_SCN_LNK_COMDAT is not set)
        b'\x0a\x00', # subsystem
        b'\0\0', # DLL characteristics
        b'\0\0\0\0\0\0\0\0', # size of stack reserve
        b'\0\0\0\0\0\0\0\0', # size of stack commit
        b'\0\0\0\0\0\0\0\0', # size of heap reserve
        b'\0\0\0\0\0\0\0\0', # size of heap commit
        b'\0\0\0\0', # loader flags
        b'\0\0\0\0', # number of RVA and sizes
    ])

def section_header(
    *,
    name,
    virtual_size,
    virtual_addr,
    raw_data_size,
    raw_data_offset,
):
    name = name.encode('utf-8')
    assert len(name) <= 8
    return b''.join([
        name.ljust(8, b'\0'),
        virtual_size.to_bytes(4, 'little'),
        virtual_addr.to_bytes(4, 'little'),
        raw_data_size.to_bytes(4, 'little'),
        raw_data_offset.to_bytes(4, 'little'),
        b'\0\0\0\0', # pointer to relocations
        b'\0\0\0\0', # pointer to line numbers
        b'\0\0', # number of relocations
        b'\0\0', # number of line numbers
        b'\x20\x00\x00\x60', # characteristics (IMAGE_SCN_CNT_CODE, IMAGE_SCN_MEM_EXECUTE, IMAGE_SCN_MEM_READ)
    ])


_, code, out = sys.argv

st = os.stat(code)

with open(out, 'wb') as f:
    f.write(MSDOS_MZ_HEADER)
    f.write(PE_HEADER)
    f.write(pe32_optional_header(
        size_of_code = st.st_size,
        size_of_initdata = 0,
        size_of_uninitdata = 0,
        base_of_code = 0,
        size_of_image = 4096, # OVMF freaks out if you don't do this
        size_of_headers = 64 + 24 + 112 + (1 * 40),
        # we're sticking the code right after the headers
        entry_address = 64 + 24 + 112 + (1 * 40),
    ))
    f.write(section_header(
        name = '.lollmao',
        virtual_size = st.st_size,
        raw_data_size = st.st_size,
        virtual_addr = 64 + 24 + 112 + (1 * 40),
        raw_data_offset = 64 + 24 + 112 + (1 * 40),
    ))
    with open(code, 'rb') as g:
        f.write(g.read())
    # pad to page size
    # OVMF yells otherwise
    pos = f.tell()
    f.write(b'\0' * ((4096 - pos) % 4096))
</syntaxhighlight>

=== FASM ===

==== boot.asm ====

<syntaxhighlight lang="asm">
use64

org 0

IMAGE_FILE_EXECUTABLE_IMAGE    = 1 shl 1
IMAGE_FILE_LARGE_ADDRESS_AWARE = 1 shl 5

IMAGE_SCN_CNT_CODE    = 0x20
IMAGE_SCN_MEM_EXECUTE = 0x20000000
IMAGE_SCN_MEM_READ    = 0x40000000

dos:
        db "MZ"
        times 58 db 0
        dd pe
pe:
        db "PE", 0, 0
        dw 0x8664 ; machine type
        dw 1      ; sections
        dd 0, 0, 0
        dw .opt.end - .opt
        dw IMAGE_FILE_EXECUTABLE_IMAGE or IMAGE_FILE_LARGE_ADDRESS_AWARE
.opt:
        dw 0x20b, 0
        dd text_end     ; size of code
        dd 0, 0         ; size of data
        dd start, start ; entry, code base
        dq 0            ; image base (irrelevant, PIE)
        dd 4096, 4096   ; alignment
        dd 0, 0, 0, 0
        dd text_end     ; image size
        dd pe.end       ; headers size
        dd 0
        dw 10           ; EFI subsystem
        dw 0
        dq 0, 0, 0, 0   ; stack/heap (irrelevant)
        dd 0, 0         ; RVA
.opt.end:
        db ".david", 0, 0
        times 2 \
        dd text_end - start, start  ; virtual/raw size/addr
        dd 0, 0, 0      ; ptr to reloc/lines, num
        dd IMAGE_SCN_CNT_CODE or IMAGE_SCN_MEM_EXECUTE or IMAGE_SCN_MEM_READ
.end:


EFI_HANDLE.sizeof = 8 ; VOID*

; 4.2.1
EFI_TABLE_HEADER.sizeof = 8 + (4 * 4)

; 4.3.1
EFI_SYSTEM_TABLE.Hdr              = 0 ; EFI_TABLE_HEADER
EFI_SYSTEM_TABLE.FirmwareVendor   = 24 ; CHAR16
EFI_SYSTEM_TABLE.FirmwareRevision = 32 ; UINT32
EFI_SYSTEM_TABLE.ConsoleInHandle  = 40 ; EFI_HANDLE
EFI_SYSTEM_TABLE.ConIn            = 48 ; EFI_SIMPLE_TEXT_INPUT_PROTOCOL
EFI_SYSTEM_TABLE.ConsoleOutHandle = 56 ; EFI_HANDLE
EFI_SYSTEM_TABLE.ConOut           = 64 ; EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL
; there's more but idc

; 12.4.1
EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL.Reset        = 0 ; EFI_TEXT_RESET
EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL.OutputString = 8 ; EFI_TEXT_STRING

; rcx: EFI_HANDLE (of ourselves, just ignore)
; rdx: EFI_SYSTEM_TABLE*
start:
        mov rax, [rdx + EFI_SYSTEM_TABLE.ConOut]
        mov rbx, [rax + EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL.OutputString]

        ; 12.4.3
        ; typedef EFI_STATUS (EFIAPI *EFI_TEXT_STRING) (
        ;   IN EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL *This
        ;   IN CHAR16 *String
        ; );
        ; I will _never_ forgive Microsoft for being so stubborn with (not even!) UTF-16
        ; instead of at least trying to use the actually sane UTF-8 standard
        mov rcx, rax
        lea rdx, [hello_uefi]
        ;call _break
        call rbx

@@:     hlt
        jmp @b

_break:
        xor r15, r15
@@:     test r15, r15
        jz @b
        ret

hello_uefi: dw 'H', 'e', 'l', 'l', 'o', ' ', 'U', 'E', 'F', 'I', '!', 0


times ((-$) and 0xfff) db 0

text_end:
</syntaxhighlight>

==== build.sh ====

<syntaxhighlight lang="sh">
#!/bin/sh
src="$(dirname "$0")"
set -xe
mkdir -p $out/
fasm $src/boot.asm $out/bootx64.efi


# https://wiki.osdev.org/User:Demindiro/Super_Tiny_UEFI_Hello_World

fat=$out/efi-fs.img
gpt=$out/root.img
fallocate -l512K $fat
mformat -i $fat
mmd     -i $fat             ::/efi ::/efi/boot
mcopy   -i $fat $out/bootx64.efi ::/efi/boot/bootx64.efi
mdir    -i $fat -/ -w -a

fallocate -l1M $gpt
cat <<CMD | fdisk $gpt
g
n



t
uefi
w
CMD
fdisk -l $gpt

dd if=$fat of=$gpt bs=512 oseek=34 conv=notrunc

fdisk -l $gpt
</syntaxhighlight>

=== qemu.sh ===

<syntaxhighlight lang="sh">
#!/bin/sh

set -xe

qemu=qemu-system-x86_64
ovmf=edk2/Build/OvmfX64/DEBUG_GCC/FV/OVMF.fd
img=uefi-test.img

exec "$qemu" \
	-machine q35 \
	--enable-kvm \
	--bios "$ovmf" \
	-drive format=raw,file="$img" \
	-debugcon file:debug.log -global isa-debugcon.iobase=0x402 \
	-s \
</syntaxhighlight>

[[Category:UEFI]]
[[Category:Firmware]]
[[Category:X86-64]]
````

<a id="w004915"></a>

## W004915 — User:Mid/USB mobile modems

Bron-ID: W004915
Namespace: User (2)
Revisie: 29396 | Datum: 2025-02-11T08:14:00Z
Bron: https://wiki.osdev.org/User:Mid/USB_mobile_modems
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMid%2FUSB_mobile_modems&oldid=29396

````text
''This is a draft page. Once it is ready I intend to move it to the main namespace.''

USB mobile modems are highly diverse. From a programmer's perspective they can be categorized in multiple ways:

* Whether they have ZeroCD and how it is disabled
* How they expose you to the mobile network (through serial, MBIM, QMI, through a local web server or not at all)
* How they expose you to the Internet (through serial, USB CDC, USB RNDIS, QMI MSM or not at all)

Ideally each of these problems would be handled by a unique layer.

== Handling ZeroCD ==

Most dongles first show themselves as a mass storage device to provide Windows drivers to the user. In the best case scenario, sending an "eject" command to the mass storage device is enough for the device to disconnect and reconnect in modem mode, but the method to turning this off varies with almost every single device.

Fortunately, the effort of putting together a database has been done in the usb_modeswitch project (<code>/usr/share/usb_modeswitch/configPack.tar.gz</code>). If a file for your device exists and it contains a <code>MessageContent</code> key, then it is a hex-encoded command block wrapper for the USB BBB mass storage protocol.

== Using the mobile network ==

If your dongle is actually not a USB modem, but more of a USB router, there will be no way to communicate with the mobile network for e.g. sending SMS messages or establishing calls. Said devices usually expose themselves as an Ethernet port over USB CDC, and often contain a web server for configuration at the gateway address, akin to how a regular router works. If only an internet connection is enough for you, then you need not do anything more than a CDC Ethernet driver.

=== AT overview ===

In 1996, the [https://www.etsi.org/deliver/etsi_gts/07/0707/05.00.00_60/gsmts_0707v050000p.pdf GSM 07.07 specification] popularized the use of the AT command set for communicating with the modem. The channel is through a serial interface, which for modern USB dongles implies a USB CDC serial port.

Beware that AT syntax makes no sense whatsoever, and one should not attempt to find a pattern within beyond what is written.

Input to the modem consists of an AT command line, which begins with the characters <code>AT</code>, followed by one or more commands. Commands are delimited by semicolons only when necessary, else they are of fixed size. Extended commands are prefixed with <code>+</code> and must always end with a semicolon. Every extended command can be suffixed with <code>?</code> to testing the command's existence or checking the current state. Every extended command can be suffixed with <code>=?</code> to find the possible values of the corresponding command.

The command line is terminated with a carriage return (ASCII 13) marked with <code>&lt;CR&gt;</code>. Note that the carriage return is not the line feed character (ASCII 10) marked with <code>&lt;LF&gt;</code>.

For each command in the command line, the modem returns each response, followed by <code>&lt;CR&gt;&lt;LF&gt;OK&lt;CR&gt;&lt;LF&gt;</code>. Responses themselves are surrounded by <code>&lt;CR&gt;&lt;LF&gt;</code> on ''both sides''. Here is an example command and response:

 ATCMD1CMD2=12;+CMD1;+CMD2=,,15;+CMD2?;+CMD2=?<CR>

 <CR><LF>+CMD2: 3,0,15,"GSM"<CR><LF>
 <CR><LF>+CMD2: (0-3),(0,1),(0-12,15),("GSM","IRA")<CR><LF>
 <CR><LF>OK<CR><LF>

...

Certain models require you to activate the USB Ethernet port, while for others it is automatic. Try <code>AT+XCEDATA=1,0</code> while ignoring the output. For Quectel models, <code>AT+QCFG="usbnet",1</code>. Reset the modem with <code>AT+CFUN=0</code> then <code>AT+CFUN=1</code> with a delay in between.

<code>AT+CGDCONT</code> ''TBA''

<code>ATDT*99#</code> ''TBA''

== Activating the Internet connection ==

=== Through AT ===
````

<a id="w004917"></a>

## W004917 — Category:Pages using deprecated source tags

Bron-ID: W004917
Namespace: Category (14)
Revisie: 29404 | Datum: 2025-02-17T08:56:49Z
Bron: https://wiki.osdev.org/Category:Pages_using_deprecated_source_tags
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APages_using_deprecated_source_tags&oldid=29404

````text
[[Category:Wiki administration]]
````

<a id="w004918"></a>

## W004918 — KVM

Bron-ID: W004918
Namespace: Main (0)
Revisie: 29407 | Datum: 2025-02-17T09:11:52Z
Bron: https://wiki.osdev.org/KVM
Revisie-URL: https://wiki.osdev.org/index.php?title=KVM&oldid=29407

````text
{{Emulators}}
{{Stub}}

'''KVM''' is a Linux kernel module that provides virtualisation support usable by user-space management tools (such as [[QEMU]] and virt-manager).

KVM is highly used in cloud hosting of virtual machines and other services. It is not as useful for operating system development as it does not provide any emulated hardware (it uses QEMU to emulate hardware).

[[Category:Emulators]]
````

<a id="w004919"></a>

## W004919 — Multikernel

Bron-ID: W004919
Namespace: Main (0)
Revisie: 29437 | Datum: 2025-03-14T19:26:14Z
Bron: https://wiki.osdev.org/Multikernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Multikernel&oldid=29437

````text
{{Template:Kernel designs}}
{{In Progress}}

A multikernel treats a multi-core system as a distributed system, where each core runs a separate instance of the kernel. Each task is local to a single core. Inter-core communication is achieved with message passing. No memory is shared between each kernel (except for the purpose of message passing, as with other devices).

The kernels on each core do not have to be identical: each kernel can be specialized for a specific task.

Multikernels are particularly suitable for systems with multiple incompatible cores, e.g. due to different feature sets (for example, a RISC-V system with one set of cores having 128-bit vectors and another set having 512-bit vectors).

In theory, a multikernel-based system should scale wider more naturally since parallel applications do not share memory and instead rely on message passing, removing reliance on a single system with shared memory.

== See also ==

* [https://barrelfish.org/ Barrelfish], a discontinued research OS
* [https://www.sigops.org/s/conferences/sosp/2009/papers/baumann-sosp09.pdf "The Multikernel: A new OS architecture for scalable multicore systems"]

== References ==

* The [https://raw.githubusercontent.com/milkv-oasis/oasis-files/refs/heads/main/sg2380/SG2380%20Product%20Brief%20V1.4.pdf SG2380] has 12+4 P670 and 4 X280 cores. The P670 and X280 have a very similar feature set but incompatible vector widths, making transparent scheduling of tasks between the two core sets impossible.
````

<a id="w004920"></a>

## W004920 — Template:X86 protected mode interrupts

Bron-ID: W004920
Namespace: Template (10)
Revisie: 29448 | Datum: 2025-04-01T23:09:14Z
Bron: https://wiki.osdev.org/Template:X86_protected_mode_interrupts
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AX86_protected_mode_interrupts&oldid=29448

````text
<noinclude>
Originally created for Wikipedia. [https://en.wikipedia.org/w/index.php?title=Template:X86_protected_mode_interrupts&oldid=1283313564]

Copied from: [https://wiki.jwo.cz/wiki/x86_protected_mode_interrupts]

This table contains information collected from the ''Intel® 64 and IA-32 Architectures Software Developer’s Manual Volume 3 (3A, 3B, 3C & 3D): System Programming Guide'' revision from April&nbsp;2022.

== Interrupt table ==

</noinclude>
{| class="wikitable sortable"
|+ x86 Interrupt descriptor table items<ref>''Intel® 64 and IA-32 Architectures Software Developer’s Manual Volume 3 (3A, 3B, 3C & 3D): System Programming Guide'', April 2022</ref> <span class="plainlinks" style="font-weight: normal">&#x5b;[https://wiki.osdev.org/Template:x86_protected_mode_interrupts?action=edit edit table]]</span>
|-
! class="unsortable" colspan="2" | {{abbr|Int. №|Interrupt number}}
! rowspan="2" | {{abbr|Mnem.|Mnemonic}}
! rowspan="2" | Type
! rowspan="2" | {{abbr|Err. code|Error code}}
! class="unsortable" rowspan="2" | Name
! class="unsortable" rowspan="2" style="text-align: left" | Source
|-
! class="unsortable" | {{abbr|hex|Hexadecimal}}
! {{abbr|dec|Decimal}}
|-
| 0x00
| 0
| #DE 
| Fault 
| {{no|No}}
| Divide Error 
| DIV and IDIV instructions. 
|-
| 0x01
| 1
| #DB 
| Trap
| {{no|No}}
| Debug Exception 
| Instruction, data, and I/O breakpoints; single-step; and others. 
|-
| 0x02
| 2
| NMI
| Interrupt 
| {{no|No}}
| NMI Interrupt 
| Nonmaskable external interrupt. 
|-
| 0x03
| 3
| #BP 
| Trap 
| {{no|No}}
| Breakpoint 
| INT3 instruction. 
|-
| 0x04
| 4
| #OF 
| Trap 
| {{no|No}}
| Overflow 
| INTO instruction. 
|-
| 0x05
| 5
| #BR 
| Fault 
| {{no|No}}
| BOUND Range Exceeded 
| BOUND instruction. 
|-
| 0x06
| 6
| #UD 
| Fault 
| {{no|No}}
| Invalid Opcode (Undefined Opcode) 
| UD instruction or reserved opcode. 
|-
| 0x07
| 7
| #NM 
| Fault 
| {{no|No}}
| Device Not Available (No Math Coprocessor) 
| Floating-point or WAIT/FWAIT instruction. 
|-
| 0x08
| 8
| #DF 
| Abort 
| {{yes|Yes (zero)}}
| Double Fault 
| Any instruction that can generate an exception, an {{abbr|NMI|Non-maskable interrupt}}, or an {{abbr|INTR|external interrupt}}. 
|-
| 0x09
| 9
| {{na}}
| Fault 
| {{no|No}}
| Coprocessor Segment Overrun (reserved) 
| Floating-point instruction.
|-
| 0x0A
| 10
| #TS 
| Fault 
| {{yes|Yes}}
| Invalid TSS 
| Task switch or TSS access. 
|-
| 0x0B
| 11
| #NP 
| Fault 
| {{yes|Yes}}
| Segment Not Present 
| Loading segment registers or accessing system segments. 
|-
| 0x0C
| 12
| #SS 
| Fault 
| {{yes|Yes}}
| Stack-Segment Fault 
| Stack operations and {{abbr|SS|stack segment}} register loads. 
|-
| 0x0D
| 13
| #GP 
| Fault 
| {{yes|Yes}}
| General Protection 
| Any memory reference and other protection checks. 
|-
| 0x0E
| 14
| #PF 
| Fault 
| {{yes|Yes}}
| Page Fault 
| Any memory reference. 
|-
| 0x0F
| 15
| colspan="2" {{na}}
| {{no|No}}
| colspan="2" {{na|Intel reserved. Do not use.|align=left}}
|-
| 0x10
| 16
| #MF 
| Fault 
| {{no|No}}
| x87 FPU Floating-Point Error (Math Fault) 
| x87 FPU floating-point or WAIT/FWAIT instruction. 
|-
| 0x11
| 17
| #AC 
| Fault 
| {{yes|Yes (zero)}}
| Alignment Check 
| Any data reference in memory.
|-
| 0x12
| 18
| #MC 
| Abort 
| {{no|No}}
| Machine Check 
| Error codes (if any) and source are model dependent. 
|-
| 0x13
| 19
| #XM 
| Fault 
| {{no|No}}
| SIMD Floating-Point Exception 
| SSE/SSE2/SSE3 floating-point instructions
|-
| 0x14
| 20
| #VE 
| Fault 
| {{no|No}}
| Virtualization Exception 
| EPT violations
|-
| 0x15
| 21
| #CP 
| Fault 
| {{yes|Yes}}
| Control Protection Exception 
| RET, IRET, RSTORSSP, and SETSSBSY instructions can generate this exception. When CET indirect branch tracking is enabled, this exception can be generated due to a missing ENDBRANCH instruction at target of an indirect call or jump. 
|-
| 0x16<br>⋮<br>0x1f
| 22<br>⋮<br>31
| colspan="3" {{unknown}}
| colspan="2" {{na|Reserved for future use as CPU exception [[Interrupt vector table|vectors]].|align=left}}
|-
| 0x20<br>⋮<br>0xff
| 32<br>⋮<br>255
| {{na}}
| Interrupt
| {{no}}
| {{na}}
| | External interrupts.
|}
````

<a id="w004921"></a>

## W004921 — Template:Abbr

Bron-ID: W004921
Namespace: Template (10)
Revisie: 29449 | Datum: 2025-04-01T23:09:27Z
Bron: https://wiki.osdev.org/Template:Abbr
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AAbbr&oldid=29449

````text
<abbr title="{{{2}}}">{{{1}}}</abbr>
````

<a id="w004922"></a>

## W004922 — Template:Na

Bron-ID: W004922
Namespace: Template (10)
Revisie: 29450 | Datum: 2025-04-01T23:10:35Z
Bron: https://wiki.osdev.org/Template:Na
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ANa&oldid=29450

````text
style="background: #e0d7d7" | {{{1|N/A}}}
````

<a id="w004923"></a>

## W004923 — Template:Unknown

Bron-ID: W004923
Namespace: Template (10)
Revisie: 29451 | Datum: 2025-04-01T23:10:51Z
Bron: https://wiki.osdev.org/Template:Unknown
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AUnknown&oldid=29451

````text
style="background: #ddd" | {{{1|N/A}}}
````

<a id="w004925"></a>

## W004925 — User:Israfiela

Bron-ID: W004925
Namespace: User (2)
Revisie: 29720 | Datum: 2025-10-19T18:08:20Z
Bron: https://wiki.osdev.org/User:Israfiela
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AIsrafiela&oldid=29720

````text

````

<a id="w004927"></a>

## W004927 — User:Callumari

Bron-ID: W004927
Namespace: User (2)
Revisie: 29474 | Datum: 2025-04-17T12:06:34Z
Bron: https://wiki.osdev.org/User:Callumari
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACallumari&oldid=29474

````text
Callumari is an English/Irish kernel hobbyist[https://callumari.com] who enjoys low level programming and correct grammar. He also has a passion for server maintenance[https://callumari.com].
````

<a id="w004929"></a>

## W004929 — "8042" PS/2 Controller

Bron-ID: W004929
Namespace: Main (0)
Revisie: 29478 | Datum: 2025-04-19T23:56:34Z
Bron: https://wiki.osdev.org/%228042%22_PS/2_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=%228042%22_PS%2F2_Controller&oldid=29478
Doorverwijzing: I8042 PS/2 Controller

````text
#REDIRECT [[I8042 PS/2 Controller]]
````

<a id="w004930"></a>

## W004930 — Talk:"8042" PS/2 Controller

Bron-ID: W004930
Namespace: Talk (1)
Revisie: 29480 | Datum: 2025-04-19T23:56:34Z
Bron: https://wiki.osdev.org/Talk:%228042%22_PS/2_Controller
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3A%228042%22_PS%2F2_Controller&oldid=29480
Doorverwijzing: Talk:I8042 PS/2 Controller

````text
#REDIRECT [[Talk:I8042 PS/2 Controller]]
````

<a id="w004931"></a>

## W004931 — Vulkan

Bron-ID: W004931
Namespace: Main (0)
Revisie: 30692 | Datum: 2026-06-04T13:52:46Z
Bron: https://wiki.osdev.org/Vulkan
Revisie-URL: https://wiki.osdev.org/index.php?title=Vulkan&oldid=30692

````text
{{stub}}
'''Vulkan''' is a modern 3D graphics API, very comparable to Direct3D 12.

These APIs are much more multithreading friendly, and do this by offering lower level control of the hardware to applications than previous generation APIs like Open GL or Direct3D 11. In contrast to these APIs, Vulkan avoids global state, state is localised to a command buffer.

Closer mapping to the graphics hardware is able to achieve better performance, at the cost of application complexity. However, this maps to reducing work done by device drivers (and thus, driver complexity).

The existence of projects like ANGLE and DXVK, which implement OpenGL and Direct3D 8-11 respectively, on top of Vulkan, further makes Vulkan an attractive API to target.

== Structure of a Vulkan Application ==
Note that this is not meant to be a tutorial on how to use Vulkan! Rather, it's useful to understand the typical flow of an application.

'''vkCreateInstance''' - Creates an application instance. This is the closest thing that Vulkan has to a global state. However, what's passed in is a list of ''required'' extensions, and some information about the application (name, required API version...).

'''vkEnumerateInstanceExtensionProperties''' can enumerate for ''optional'' extensions.

Now with this initial phase of initialization out of the way, the application can now enumerate graphics adapters.

'''vkEnumeratePhysicalDevices''' - Returns a count, and optionally, fills an array of, <code>VkPhysicalDevice</code>handles.

'''vkGetPhysicalDeviceQueueFamilyProperties''' - Enumerates the queue families supported by a physical device. An application may well need different queues for Compute vs 3D. This is how it finds out.


Finally, the application is able to create a logical device object.

'''vkCreateDevice'''. This takes a physical device to use, a set of queue creation descriptions, and optionally a set of host memory allocator callbacks - allowing e.g. pool allocators instead of default heap.

=== Render Pipeline ===
The application now starts to build a render pipeline. There are numerous stages involved here!<syntaxhighlight lang="cpp">
// Provided by VK_VERSION_1_0
typedef struct VkComputePipelineCreateInfo {
    VkStructureType                    sType;
    const void*                        pNext;
    VkPipelineCreateFlags              flags;
    VkPipelineShaderStageCreateInfo    stage;
    VkPipelineLayout                   layout;
    VkPipeline                         basePipelineHandle;
    int32_t                            basePipelineIndex;
} VkComputePipelineCreateInfo;
</syntaxhighlight><syntaxhighlight lang="cpp">
// Provided by VK_VERSION_1_0
typedef struct VkGraphicsPipelineCreateInfo {
    VkStructureType                                  sType;
    const void*                                      pNext;
    VkPipelineCreateFlags                            flags;
    uint32_t                                         stageCount;
    const VkPipelineShaderStageCreateInfo*           pStages;
    const VkPipelineVertexInputStateCreateInfo*      pVertexInputState;
    const VkPipelineInputAssemblyStateCreateInfo*    pInputAssemblyState;
    const VkPipelineTessellationStateCreateInfo*     pTessellationState;
    const VkPipelineViewportStateCreateInfo*         pViewportState;
    const VkPipelineRasterizationStateCreateInfo*    pRasterizationState;
    const VkPipelineMultisampleStateCreateInfo*      pMultisampleState;
    const VkPipelineDepthStencilStateCreateInfo*     pDepthStencilState;
    const VkPipelineColorBlendStateCreateInfo*       pColorBlendState;
    const VkPipelineDynamicStateCreateInfo*          pDynamicState;
    VkPipelineLayout                                 layout;
    VkRenderPass                                     renderPass;
    uint32_t                                         subpass;
    VkPipeline                                       basePipelineHandle;
    int32_t                                          basePipelineIndex;
} VkGraphicsPipelineCreateInfo;
</syntaxhighlight>Note the basePipeline fields allow optimisations when deriving from a similar pipeline, they can reasonably be ignored here.

=== Shaders ===
An application will create shaders. These are specified in SPIR-V bytecode.

'''vkCreateShaderModule''' - Creates a shader module from an array of SPIR-V bytecode. Note that this is simply a loading operation, it's a thin, opaque, wrapper.

Once shader modules are loaded, they can be specified in a '''VkPipelineShaderStageCreateInfo''' . Note there can be multiple entry points to a shader, shaders can be linked...

'''Vertex Input'''

The format of the vertex buffer is specified. This is how the application will feed raw vertex data to the shaders.

'''Input Assembly'''

An initial processing step, where the toplogogy is specified. The vertex buffer may be individual triangles, triangle strips, points, lines... The input assembly builds the basic mesh based on this.

'''The rest of the pipleine is similar.'''

==== Pipleline Layout ====
'''vkCreatePipelineLayout''' - This is where ''uniforms'' ('''cbuffer''' in HLSL) are specified. A relatively small buffer that effectively paramaterises the shader. For instance, the view transform can be passed to the vertex shader this way.

==== Render Pass ====
'''vkCreateRenderPass -''' Determines how the result is rendered to a framebuffer.

After all that work, we reach '''vkCreateGraphicsPipelines'''

=== Framebuffer ===
'''vkCreateFramebuffer''' creates a framebuffer, which is pretty straightforward. It can be used as a destination for a graphics pipeline.

=== Command Queues ===
Now we're getting somewhere - and this section maps closely to the graphics hardware itself. First, a command pool is created - '''vkCreateCommandPool'''

'''vkAllocateCommandBuffers''' - Allocate command buffers from the pool.

'''vkBeginCommandBuffer -''' Starts a sequence of commands in a buffer.

'''vkCmdBeginRenderPass'''

'''vkCmdBindPipeline'''

'''vkCmdDraw'''

'''vkCmdEndRenderPass'''

'''vkEndCommandBuffer'''

'''vkQueueSubmit -''' submits the recorded command queue asynchronously.

=== Synchronisation ===
All synchronisation is explicit, there are semaphores for sync between GPU processes, and fences for the host CPU.

== See Also ==

=== External Links ===

* [https://docs.vulkan.org/tutorial/latest/00_Introduction.html Vulkan Documentation]

[[Category:Video]]
````

<a id="w004932"></a>

## W004932 — File:Intel Graphics Overview.png

Bron-ID: W004932
Namespace: File (6)
Revisie: 29497 | Datum: 2025-04-28T11:00:33Z
Bron: https://wiki.osdev.org/File:Intel_Graphics_Overview.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIntel_Graphics_Overview.png&oldid=29497

````text
== Summary ==
Overview of intel graphics architecture
````

<a id="w004933"></a>

## W004933 — User:Bellezzasolo/Intel Graphics Family

Bron-ID: W004933
Namespace: User (2)
Revisie: 29509 | Datum: 2025-04-30T15:41:37Z
Bron: https://wiki.osdev.org/User:Bellezzasolo/Intel_Graphics_Family
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABellezzasolo%2FIntel_Graphics_Family&oldid=29509

````text
{{In Progress}}

Intel have been producing integrated graphics devices in their CPUs or Chipsets for approximately 20 years at the time of writing. Integrated Graphics is generally less glamorous than the dedicated graphics cards that are essentially a requirement for playing modern games.

The advantage for Hobby OS development is that Intel make their programming manuals freely available, covering the entire card. In contrast, AMD publish the ISA for their GPUs, but documentation on basic functions like modesetting is hard to come by - [[AMD Atombios]] is probably the place to go for that.

However, Intel have recently entered the discrete GPU market with the ''ARC'' range.

The general architecture of the family hasn't changed between the i965/G35 chipset of 2008 (the earliest documentation the author can find), and current generations.

== Other Sources ==
On the wiki, very incomplete:

[[Intel HD Graphics]]

[[Native Intel graphics]]

'''Documentation'''

https://www.intel.com/content/www/us/en/docs/graphics-for-linux/developer-reference/1-0/overview.html

https://www.x.org/docs/intel

== General Architecture ==
[[File:Intel Graphics Overview.png|alt=Overview image of intel graphics architecture|thumb|General Architecture of Intel Graphics]]Intel GPUs consist of several engines, and a couple of subsidiary components.

=== Display Engine ===
The display engine is responsible for driving the displays. This is the component responsible for setting video modes. It consists of several display '''pipes''', which can be connected to a number of '''Display Digital Interfaces (DDIs).''' These contain transcoders and connect to a physical display (or there may also exist options for wireless display or capture - e.g. hardware screen capture support).

A pipe can only drive one DDI, however, DisplayPort Multistream can allow several pipes to be transmitted on one PHY (i.e, running multiple monitors off one DisplayPort cable).

Pipes effectively map to a monitor, and can perform several transforms, as well as framebuffer flipping.

=== Blitter ===
Variously called the blitter or copy engine, this is effectively the 2D acceleration component of the GPU, and a prime resource for a hobby OS. The command set is reasonably straightforward, but has a wide array of available 2D transformations, utilising '''source''' data, '''pattern''' data, and the '''destination''' data. For instance, this component can be used to render bitmap fonts, in hardware.

=== Video Codec Engine ===
This provides hardware support for decoding and even encoding to video formats like HEVC and AVC1.

=== 3D Engine ===
Split into several subsystems, but this is the meat of the GPU, its 3D rendering capabilities.

=== Memory Interface ===
<div style="clear:both;" class="">The memory interface controls access to Graphics and System memory.
The GPU uses a '''virtual address space''', implemented using paging. However, there are two modes:
'''Global Graphics Translation Table (GGTT)''' - Shared translations
'''Per Process Graphics Translation Table (PPGTT)''' - Translations tied to a specific process.

Certain objects like Display Framebuffers, must use GGTT for simplicity and performance.</div>
==Family==
{| class="wikitable"
|+
!Name
!CPUs
!Codename
!Year
!Description
|-
|'''G965<br />G35'''
|Pentium Dual Core
Core 2 Duo
|Broadwater<br />
Glenwood<br />
Bearlake
|Q2 2006
|Initial version that's documented
|-
|'''G45'''
|Core 2 Duo/Quad
|Eaglelake
|Q2 2008
|
|-
! colspan="5" |Intel HD Graphics
|-
|'''HD'''
|Core i3/i5 1st Gen
|Ironlake
|Q1 2010
|
|-
|'''HD 2000<br />HD 3000'''
|Core 2nd Gen
|Sandy Bridge
|Q1 2011
|
|-
|'''HD 2500<br />HD 4000'''
|Core 3rd Gen
|Ivy Bridge
|Q2 2012
|
|-
|
|
|Haswell
|
|
|-
|
|
|
|
|-
! colspan="5" |Intel UHD Graphics
|-
|'''UHD 6xx'''
|
|Kaby Lake Refresh
|Q4 2017
|-
! colspan="5" |ARC
|-
|'''A3xx<br />A5xx<br />A7xx'''
|Graphics Card
|Alchemist
|Q1 2022
|
|-
|'''B570<br />B580'''
|Graphics Card
|Battlemage
|Q4 2024
|
|}

== Infrastructure ==
This section details the basic principles of the hardware, common to all engines.

=== Detection and Address Spaces ===
The graphics adapter will show up on the PCI bus. Typically this is at 0:02:0, but it could vary by chipset.

The adapter typically has 3 BARs. Again, details '''could''' vary by chipset, however, as far as I can tell, the layout is as follows:

BAR0 - '''GTTMMADR'''. BAR for the global translation tables and MMIO registers. MMIO comes first.

(64 bit BARs consume two slots - see [[PCI]]. I think all devices are 64 bit.

BAR2 - '''GMADR.''' This maps graphics RAM.

BAR4 - '''IOBAR'''. For compatibility with legacy VGA registers.

=== Command Streamers ===
There are a number of command streamers, which issue commands to engines. Typically, each engine has at least one command streamer, and sometimes several. The exact details vary across generations.

'''Before Sandy Bridge''', the Blitter and 3D Render engines shared a command streamer, but are now seperate.

== Memory Interface ==

=== Global GTT ===
The global GTT is a single level page table that is for privileged objects not tied to a specific process. In particular, display framebuffers use this translation scheme.

Page table entries switched from a 32 bit to a 64 bit format at gen8 graphics (Broadwell), and from describing a 2GB virtual address space to a 4GB address space. Note this address space is 32 bit in all SKUs thus far.

The size of the GTT can vary.

==== Haswell and Earlier ====
{| class="wikitable"
|+
!31:12
!11:8
!7:4
!3
!2:1
!0
|-
|PADDR[31:12]
|RSV0
|PADDR[35:32]
|RSV0
|Mapping Type
|Valid
|}
{| class="wikitable"
|+
!Mapping Type
!Description
|-
|0
|Targets main memory
|-
|1-2
|Reserved
|-
|3
|Targets cacheable main memory with snoop
|}
Note that graphics stolen memory is considered main memory here.

==== Broadwell and Later ====
{| class="wikitable"
|+
!63:HAW
!HAW-1:12
!11:2
!1
!0
|-
|Ignored
|PADDR[:12]
|Function Number
|Local
|Valid
|}
HAW is 39 for client hardware, and 46 for server hardware.



'''PADDR -''' Physical address. 4K aligned, matching x86 paging.

'''Function Number''' - Function the page is assigned to if using SRIOV hardware virtualisation. Ignored otherwise.

'''Local''' - Targets local graphics memory instead of system memory. Ignored if there isn't dedicated GRAM.

'''Valid''' - The page is present.

=== Per-Process GTT ===
Address spaces are loaded based in several ways. Earlier models use a mechanism similar to x86's CR3.

===== i965 - Single Level Paging =====
The per-process page table is set using '''PGTBL_CTL2''' (MMIO+20C4). It must be a 4K aligned page table, and does not use snoops.
{| class="wikitable"
|+PGTBL_CTL2
!31:12
!11:8
!7:4
!3:1
!0
|-
|PTAB[31:12]
|RESV0
|PTAB[35:32]
|SIZE
|Enable
|}
'''SIZE -''' The size of the page table, in bytes, is 2^(16+SIZE). Values of size >4 (1MB) are reserved.

The format of page entries is the same as for the GGTT.

===== G45 - Two Level Paging =====
The page directory can be set via '''PP_DIR_BASE''' (MMIO+2518), but should be set via contexts instead.

===== Broadwell - IA32e Style =====
Broadwell cores implement a 4 level paging scheme that closely resembles the format used for IA32e [[Paging]].

== Display Engine ==
This is the most important component to implement, without it, no video for you. Unless you use the firmware provided framebuffer, but if so, why are you here? TODO.

== Blitter ==
Probably the easiest component to understand.

=== Command Set ===
{| class="wikitable"
|+
!Command
!Opcode
|-
|XY_SETUP_BLT
|01h
|-
|
|
|-
|
|
|}
````

<a id="w004934"></a>

## W004934 — User:Thedabbingduck/OStrichFS

Bron-ID: W004934
Namespace: User (2)
Revisie: 30320 | Datum: 2026-03-17T19:05:56Z
Bron: https://wiki.osdev.org/User:Thedabbingduck/OStrichFS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AThedabbingduck%2FOStrichFS&oldid=30320

````text
'''OstrichFS''' (version alpha) is a '''crash-consistent''', '''snapshot-capable''', flash-optimized file system designed for use on SSDs and SD cards. It is a "roll your own" filesystem, designed and written from scratch. It takes various features from other popular filesystem designs, as described below. It was developed as part of the OStrich project by a group of students at the University of Texas at Austin. 

==Overview==
OstrichFS offers:
*'''Crash consistency''': every metadata and data change is logged and committed out-of-place. On reboot it replays only the necessary operations to restore a valid state. This ensures that losing power at any point will not leave the system in an unrecoverable state.
*'''Snapshots''': on-demand checkpoints capture the complete inode-mapping, letting you mount any past state read-only without duplicating file data.
*'''Flash-friendly writes''': avoids small in-place updates by always allocating new blocks, reducing wear and write amplification.
*'''Rich file support''': multi-level inode (15 direct, 10 single-indirect, 2 double-indirect) lets you store very large files.
*'''Simple allocation''': bitmaps for free inodes and blocks keep allocation code minimal and fast.
*'''Journaling area''': a fixed region (e.g. 64 blocks) holds both operation logs and checkpoint chains.

'''Drawbacks & limits''':
*Log/checkpoint area is fixed-size—may overflow if you go too long without checkpointing.
*No automatic garbage collection—old data and inodes accumulate until manually reclaimed.
*Snapshot array capped at 128—older checkpoints will be overwritten.
*No built-in wear-leveling or configurable parameters yet.
*Early development—bug-prone, feature-incomplete and under active development.
*Minimal caching capabilities (only structures like bitmaps / inode tables are cached at the filesystem level, other caching must be implemented by the operating system)

==Design & Key Concepts==
===Disk Regions===
Block 0 is the '''superblock''' (magic, region offsets, free-counts, checkpoint pointers). It contains pointers to the following regions:
*'''Inode region''' – array of fixed-size inodes.
*'''Inode table''' – maps logical inode numbers to physical inode slots (enables copy-on-write updates).
*'''Bitmaps''' – track free inodes and data blocks.
*'''Data blocks''' – hold file and directory contents.
*'''Log/checkpoint area''' – sequential operation log entries plus checkpoint block chains.

===Copy-on-Write & Logging===
Every change (file write, directory update, metadata tweak) follows:
#Modify in memory → write to a new block
#Update parent pointers (indirect blocks or inode) via new blocks
#Log the final inode mapping change before switching the live mapping
On mount, OstrichFS loads the last checkpoint and replays any log entries past that point.

===Snapshotting===
<syntaxhighlight lang="c">
// Mount a snapshot read-only:
fs_req_mount_snapshot(checkpointID);
</syntaxhighlight>
1. Freeze the current inode-table mapping.

2. Serialize all ''<inodeNumber, inodeLocation>'' pairs into checkpoint blocks.

3. Record the checkpoint linked list's first checkpoint block in the superblock.  

4. Mounting that checkpoint reconstructs the old mapping in memory and prohibits writes.

==On-Disk Layout==
OStrichFS utilizes a generally simplified on-disk layout to focus more on user-level features instead of disk-usage or performance optimization. 

The '''superblock''' contains the following fields:
{| class="wikitable"
!Field Name!!Size (bytes)!!Type!!Usage
|-
|'''magic'''||8||uint64_t||Stores the magic number for OStrichFS (0xCA5CADEDBA5EBA11)
|-
|'''size'''||8||uint64_t||Stores the max number of data bytes based on the number of data blocks in the filesystem
|-
|'''totalBlockCount'''||4||uint32_t||Stores the total number of blocks on the partition
|-
|'''dataBlockCount'''||4||uint32_t||Stores the total number of data blocks in the filesystem
|-
|'''inodeCount'''||4||uint32_t||Stores the total number of inodes in the filesystem
|-
|'''freeDataBlockCount'''||4||uint32_t||Stores the number of free data blocks in the filesystem
|-
|'''freeInodeCount'''||4||uint32_t||Stores the number of free inodes in the filesystem
|-
|'''dataBlockBitmap'''||4||uint32_t||Stores the block number at which the data block bitmap starts
|-
|'''dataBlockBitmapSize'''||4||uint32_t||Stores the size of the data block bitmap (in blocks)
|-
|'''inodeBitmap'''||4||uint32_t||Stores the block number at which the inode bitmap starts
|-
|'''inodeBitmapSize'''||4||uint32_t||Stores the size of the inode bitmap (in blocks)
|-
|'''inodeTable'''||4||uint32_t||Stores the block number at which the inode table starts
|-
|'''inodeTableSize'''||4||uint32_t||Stores the size of the inode table (in blocks)
|-
|'''dataBlockRegionStart'''||4||uint32_t||Stores the block number at which the data block region starts
|-
|'''inodeRegionStart'''||4||uint32_t||Stores the block number at which the inode region starts
|-
|'''inodeRegionSize'''||4||uint32_t||Stores the size of the inode region (in blocks)
|-
|'''logAreaStart'''||4||uint32_t||Stores the block number at which the log area starts
|-
|'''logAreaSize'''||4||uint32_t||Stores the size of the log area (in blocks)
|-
|'''systemStateSeqNum'''||8||uint64_t||Stores the sequence number of the system to verify against for consistency on system reboot
|-
|'''latestCheckpointIndex'''||2||uint16_t||Stores the index of the latest checkpoint of the system
|-
|'''checkpointArr'''||512||uint32_t[]||Stores the locations of all checkpoints on the system
|-
|'''readOnly'''||1||bool||Determines whether or not the filesystem is readOnly (mainly used for snapshots in memory, should always be false on disk)
|}

The inode structure is relatively simple with the minimal fields required for a basic filesystem:
{| class="wikitable"
!Field Name!!Size (bytes)!!Type!!Usage
|-
|'''size'''||8||uint64_t||Size of the file/directory (in bytes)
|-
|'''blockCount'''||4||uint32_t||The number of blocks allocated to this file/directory
|-
|'''uid'''||2||uint16_t||UID for user owning the file/directory
|-
|'''gid'''||2||uint16_t||GID for group owning the file/directory
|-
|'''permissions'''||2||uint16_t||Permission bits (POSIX-style) for the file/directory
|-
|'''numFiles'''||2||uint16_t||Number of files in the directory (not used for file inodes)
|-
|'''directBlocks'''||60||uint32_t[]||Locations of direct blocks
|-
|'''indirectBlocks'''||40||uint32_t[]||Location of indirect blocks
|-
|'''doubleIndirectBlocks'''||8||uint32_t[]||Location of double indirect blocks
|}
The inodes are specifically structured to require 128 bytes per inode, allowing for 32 inodes per block. The permission bits of the inode are used for rwx permission for user/group/other and also to determine whether or not a file is a directory. The remaining 6 bits are unused at the moment, but could be utilized for additional flags (like the immutable flag in linux).

===Design Considerations===

The inode bitmap and data block bitmaps referenced by the superblock are on-disk structures to keep track of which blocks are currently in use. The filesystem utilizes a write-through cache for recent bitmap blocks to minimize excess reads (it still always writes to disk when allocating a block/inode for the sake of consistency). 

The inode table provides a level of indirection for inode access. All inodes are referenced via their ''inode number'' which uniquely identifies a file or directory on the system. This number is then mapped to an ''inode location'' via the inode table. When inodes or blocks are edited, the copy of write logic of the filesystem updates (and logs) the inode table to change the ''inode location'' while keeping the ''inode number'' the same. This minimizes the amount of chained copy on writes needed and allows us to mount a snapshot by simply loading an older version of the inode table (since all of the old data blocks and inodes should still be on disk at their original locations). This logic applies to all files, even the root directory (which is provided the ''inode number'' of zero but will be allocated a new location for every change made to it). The inode table also contributes to consistency guarantees because until the inode table is updated, changes made to blocks or inode do not affect the running filesystem. By enforcing the constraint that the inode table is updated last in every file operation, we are guaranteed a level of consistency (assuming the block write operation is considered atomic).

The fields referring to locations or sizes of on-disk regions (e.g., ''dataBlockRegionStart'') are initialized at filesystem creation and expected to remain constant. The filesystem allocates inode and data block regions assuming a ratio of 1 inode per 16KiB of data (which translates to 1 inode for 4 blocks with 4KiB blocks). Using this ratio, the filesystem calculates the size of bitmap and inode table required and stores them in the superblock.

Note that OStrichFS currently uses 32-bit unsigned integers to refer to block numbers which limits the max filesystem size to 2^32 blocks (~16 TiB). While the actual size of the filesystem would be slightly smaller due to the overhead of bitmaps, inodes, and other metadata structures, this limit is significantly higher than any situation which OStrichFS was designed for. Switching the 64 bit unsigned integers would not be a significant design challenge, but it would greatly increase the overhead for metadata structures and thus be a detriment on smaller disks.

==Supported POSIX-style operations==
Operations that aren't listed are not supported at this moment:

{| class="wikitable"
!Syscall!!Status!!Notes
|-
|'''open(path, flags, mode)'''||Partially||Maps to fs_req_open(); returns an inode index rather than a file descriptor; only basic read/write flags; mode used only at creation.
|-
|'''read(fd, buf, count)'''||Supported||Via File::read_at(offset,…); OS must manage file offsets.
|-
|'''write(fd, buf, count)'''||Supported||Via File::write_at(offset,…); OS must manage and update offsets.
|-
|'''stat(path, statbuf)'''||Partially||Inode provides size, uid/gid, permissions; OS must read inode fields directly.
|-
|'''fstat(fd, statbuf)'''||Partially||OS must translate fd→inode_index then read inode.
|-
|'''mkdir(path, mode)'''||Supported||fs_req_create_file(..., is_dir=true); mode stored but no chmod.
|-
|'''rmdir(path)'''||Supported||fs_req_remove_file(); only empties directories.
|-
|'''unlink(path)'''||Supported||fs_req_remove_file(); data blocks not reclaimed until GC.
|-
|'''truncate(path, length)'''||Partially||write_at can extend or overwrite data; no dedicated ftruncate() syscall.
|}

''Note:'' OstrichFS relies on the OS for descriptor management, path parsing, and advanced flag handling.


==Similarities with other file systems==
*'''Copy-on-write metadata''': Like ZFS and Btrfs, OstrichFS never overwrites in-place but allocates new blocks for metadata (indirect blocks, inodes), enabling safe snapshots.
*'''Transaction logging''': Similar to EXT4’s journal, OstrichFS records critical inode mapping changes in a write-ahead log to guarantee atomicity and fast crash recovery.
*'''Checkpoint-based snapshots''': As in NILFS2, OstrichFS can capture point-in-time filesystem states by checkpointing its inode-table mapping, providing read-only mounts of past states.
*'''Inode mapping table (NAT)''': Echoing F2FS’s Node Address Table, OstrichFS centralizes inode number → physical location indirection, simplifying copy-on-write updates without rewriting parent structures.
*'''Multi-level block pointers''': Like ext2/EXT4, OstrichFS inodes support direct, single-indirect, and double-indirect pointers to accommodate large files.
*'''Bitmap allocation''': Free-space management via bitmaps for inodes and data blocks follows the model used by many Unix-style file systems (ext2/EXT4, FAT variants), offering simplicity and speed.


==Development Status==
OstrichFS is in '''alpha'''. It is functional but not battle-tested and lacks many polishing features. You will encounter bugs and missing checks. If you’re an OSDev enthusiast or student looking to learn and contribute, your patches and feedback are welcome.

==Roadmap & Future Work==
*'''Garbage collection''': reclaim orphaned blocks and stale inodes.
*'''fsdebug tool''': create and verify disk images, inspect logs and checkpoints.
*'''Configurable parameters''': make log area size, checkpoint frequency, and bitmap layouts tunable.
*'''Wear-leveling hooks''': integrate with flash controllers for better longevity.

==See Also==
*[[File Systems]]
*[[ZFS]]
*[[Ext2]]

==External Links==
*[https://github.com/AmeyaPurao/OStrich-filesystem OStrichFS Source Code (GitHub)]
*[http://www.osdever.net/ OSDev Wiki home]

<small>Last updated: 2025-04-28</small>
````

<a id="w004935"></a>

## W004935 — User:Notnekodev

Bron-ID: W004935
Namespace: User (2)
Revisie: 29530 | Datum: 2025-05-29T09:04:54Z
Bron: https://wiki.osdev.org/User:Notnekodev
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANotnekodev&oldid=29530

````text
OS Developer from Germany 🇩🇪
Working on https://github.com/purplek2/purplek2

Discord: notcatdev
````

<a id="w004936"></a>

## W004936 — User:Goldside543

Bron-ID: W004936
Namespace: User (2)
Revisie: 29537 | Datum: 2025-06-06T22:25:41Z
Bron: https://wiki.osdev.org/User:Goldside543
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGoldside543&oldid=29537

````text
I made the Goldspace kernel. Check it out.

[[User:Goldside543/goldspace | Here's a link to its wiki page.]]
````

<a id="w004937"></a>

## W004937 — User:Goldside543/goldspace

Bron-ID: W004937
Namespace: User (2)
Revisie: 29543 | Datum: 2025-06-08T21:42:57Z
Bron: https://wiki.osdev.org/User:Goldside543/goldspace
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGoldside543%2Fgoldspace&oldid=29543

````text
Goldspace is a free and open source kernel made by Goldside543.

== Brief description ==

A 32-bit free and open source monolithic kernel for the x86 architecture, written in C and assembly. It has multitasking, around a dozen system calls, ELF support, a PS/2 keyboard driver, and more. Also included is Gash, an in-kernel shell for testing features.

== Links ==
[https://github.com/Goldside543/goldspace Goldspace repository]
````

<a id="w004939"></a>

## W004939 — File:Ux.png

Bron-ID: W004939
Namespace: File (6)
Revisie: 29549 | Datum: 2025-06-14T04:38:44Z
Bron: https://wiki.osdev.org/File:Ux.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AUx.png&oldid=29549

````text
Runtime screenshot of Uinxed-Kernel
````

<a id="w004940"></a>

## W004940 — File:CoolPotOS.png

Bron-ID: W004940
Namespace: File (6)
Revisie: 29562 | Datum: 2025-06-21T10:37:32Z
Bron: https://wiki.osdev.org/File:CoolPotOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACoolPotOS.png&oldid=29562

````text
CoolPotOS Schedule
````

<a id="w004941"></a>

## W004941 — User:Pinkcomputer

Bron-ID: W004941
Namespace: User (2)
Revisie: 29567 | Datum: 2025-06-22T03:31:17Z
Bron: https://wiki.osdev.org/User:Pinkcomputer
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APinkcomputer&oldid=29567

````text
Just an idiot who likes Rust
````

<a id="w004942"></a>

## W004942 — File:Neofetch.png

Bron-ID: W004942
Namespace: File (6)
Revisie: 29570 | Datum: 2025-06-24T17:37:11Z
Bron: https://wiki.osdev.org/File:Neofetch.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ANeofetch.png&oldid=29570

````text
Maestro running Neofetch
````

<a id="w004943"></a>

## W004943 — User:Crumble14

Bron-ID: W004943
Namespace: User (2)
Revisie: 29571 | Datum: 2025-06-24T17:38:32Z
Bron: https://wiki.osdev.org/User:Crumble14
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACrumble14&oldid=29571

````text
Hi!


I am a french software engineer working on Maestro, a Linux-compatible kernel, written in Rust to leverage the safety of the typesystem. Aiming to remove as much bloat and legacy as possible while being usable in most usecases.


It runs on the x86_64 and x86 (32 bit) architectures and is able to run bash, GNU coreutils & binutils, neofetch and gcc/g++, using musl.


Maestro's Github: https://github.com/maestro-os/maestro (a star is always appreciated <3)

My blog: https://blog.lenot.re
````

<a id="w004944"></a>

## W004944 — File:Maestro.png

Bron-ID: W004944
Namespace: File (6)
Revisie: 29572 | Datum: 2025-06-24T16:25:28Z
Bron: https://wiki.osdev.org/File:Maestro.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AMaestro.png&oldid=29572

````text
== Summary ==
Neofetch running on Maestro
````

<a id="w004945"></a>

## W004945 — File:Foton v0.638.png

Bron-ID: W004945
Namespace: File (6)
Revisie: 29579 | Datum: 2025-07-04T20:06:03Z
Bron: https://wiki.osdev.org/File:Foton_v0.638.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AFoton_v0.638.png&oldid=29579

````text

````

<a id="w004946"></a>

## W004946 — User:Cerisesky

Bron-ID: W004946
Namespace: User (2)
Revisie: 29584 | Datum: 2025-07-06T16:47:07Z
Bron: https://wiki.osdev.org/User:Cerisesky
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACerisesky&oldid=29584

````text
== Testing for 32 bit support on x86 ==
I don't know of any official way to test for 32 bit (>386) support. However, I have experimented with different 386 opcodes to determine support for it. A quick warning that this is definitely undefined behaviour though it has been tested on the 8086 using 86box. The way the code works is by attempting to set ds and fs to the same value, and if it can, then 32 bit is supported since fs is only available on 386+. I have tried other opcodes and methods for determining support, but most of them cause some weird issues or cause the cpu to hang on older machines.

<syntaxhighlight lang="asm">
cpu 8086
bits 16

; returns ax = 0 if the processor does not support 32 bit mode.
; returns ax = 1 if it does
; destroys the fs register which should be fine since you aren't
; likely to have used it if you don't know if its even available
test_386:
  push bx

  mov ax, ds
  push ds
  
  cpu 386
  pop fs
  mov bx, fs
  cpu 8086
  
  cmp ax, bx
  je .success
  .fail:
    mov ax, 0
    jmp .end
  .success:
    mov ax, 1
  .end:
    pop bx
    ret
</syntaxhighlight>

== Testing for 186 support ==
Probably not as important but for completeness I also made a test for 186. Works by attempting to push an immediate value to the stack and checking if anything on the stack actually changed. Also undefined behaviour but tested on 8086 in 86box.

<syntaxhighlight lang="asm">
cpu 8086
bits 16

; returns 0 if the processor does not support 186 instructions
; returns 1 if it does
test_186:
  push bx
  push cx
  mov bx, sp

  cpu 186
  push 0
  cpu 8086
  
  mov cx, bx
  sub cx, sp
  mov sp, bx
  
  cmp cx, 2
  je .success
  .fail:
    mov ax, 0
    jmp .end
  .success:
    mov ax, 1
  .end:
    pop cx
    pop bx
    ret
</syntaxhighlight>
````

<a id="w004947"></a>

## W004947 — File:ExectOS.png

Bron-ID: W004947
Namespace: File (6)
Revisie: 29597 | Datum: 2025-07-19T21:45:01Z
Bron: https://wiki.osdev.org/File:ExectOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AExectOS.png&oldid=29597

````text

````

<a id="w004948"></a>

## W004948 — File:Demo.jpg

Bron-ID: W004948
Namespace: File (6)
Revisie: 29606 | Datum: 2025-07-30T07:41:33Z
Bron: https://wiki.osdev.org/File:Demo.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ADemo.jpg&oldid=29606

````text
GUI apps support demonstration
````

<a id="w004949"></a>

## W004949 — User:Klakap

Bron-ID: W004949
Namespace: User (2)
Revisie: 29623 | Datum: 2025-08-17T10:37:57Z
Bron: https://wiki.osdev.org/User:Klakap
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKlakap&oldid=29623
Doorverwijzing: User:VSlezak

````text
#REDIRECT [[User:VSlezak]]
````

<a id="w004950"></a>

## W004950 — User:Mohammedisam

Bron-ID: W004950
Namespace: User (2)
Revisie: 29632 | Datum: 2025-08-28T23:08:26Z
Bron: https://wiki.osdev.org/User:Mohammedisam
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMohammedisam&oldid=29632

````text
Author and maintainer of LaylaOS.

You can find the source code in my GitHub repository: https://github.com/moisam/laylaos.

You can also read the project's [[User:Mohammedisam/laylaos|page on the Wiki]].
````

<a id="w004951"></a>

## W004951 — User:Mohammedisam/laylaos

Bron-ID: W004951
Namespace: User (2)
Revisie: 29631 | Datum: 2025-08-28T23:06:49Z
Bron: https://wiki.osdev.org/User:Mohammedisam/laylaos
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMohammedisam%2Flaylaos&oldid=29631

````text
LaylaOS is a free and open source Operating System made by Mohammedisam.

== Brief description ==

LaylaOS is a hobby operating system with a kernel that is written in C. The project started around 2015 and took almost 9 years to reach a stable point where I felt it could be shared with the world. It is meant as a learning tool to better understand the internal workings of an Operating System's kernel, how system calls work and how to implement a Graphical User Interface (GUI) environment.

== Features ==

The project's features include (from the project's [https://github.com/moisam/laylaos/blob/main/README.md README]):

*   64 bit monolithic multitasking pre-emptive kernel written in C (the 32 bit code is included here but is not fully functional)
*   Symmetric Multiprocessing (SMP), supports up to 32 cores
*   Network stack (TCP, UDP, raw and Unix sockets, IPv4)
*   ATA/ATAPI and AHCI support
*   Basic Intel HDA sound support
*   PS2 keyboard and mouse
*   Virtual consoles
*   Virtual Dynamic Shared Object (vdso)
*   Interprocess Communication (IPC) facilities in the form of shared memory, semaphores and message queues
*   Page cache for demand loading of pages
*   Loopback devices (loopback)
*   Builtin filesystem support for Ext2, FAT12/16 and VFAT
*   ACPI support via ACPICA
*   C library (currently using musl)
*   Ports of 3rd party software (GNU coreutiles, inetutils, bash, several image and audio/video decoding libraries, etc.)
*   Graphical desktop with many (mostly demonstrative) applications
*   SDL2
*   Qt5.12
*   OpenTTD
*   A fork of SDL2 DOOM

== License ==

LaylaOS is released under GPL v3. Ported software and 3rd party packages come with their own licenses. For details, see the [https://github.com/moisam/laylaos/blob/main/README.md README] file.

== Links ==
[https://github.com/moisam/laylaos LaylaOS GitHub repository]
````

<a id="w004952"></a>

## W004952 — Talk:Native Intel graphics

Bron-ID: W004952
Namespace: Talk (1)
Revisie: 29634 | Datum: 2025-08-30T19:58:15Z
Bron: https://wiki.osdev.org/Talk:Native_Intel_graphics
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ANative_Intel_graphics&oldid=29634

````text
== DPLL formulae ==
Are they correct? The mismatching formulae for the clock frequency (<code>DPLL = (reference frequency * (5 * (M1 - 2) + (M2 - 2)) / N) / (P1 * P2)</code> and <code>DPLL = (reference frequency * (5 * (M1 + 2) + (M2 + 2)) / (N + 2)) / (P1 * P2)</code>) are understandable as it's just mismatched definitions between this page and the Intel documentation.

But, the tables of ranges remain unclear. Those definitely don't match (e.g. this page says N can be 1-4 yet it's 3-8 in the doc). Hell, I can't figure out if it's <code>M1 < M2</code> or <code>M1 > M2</code>.

[[User:Mid|Mid]] ([[User talk:Mid|talk]]) 19:58, 30 August 2025 (UTC)
````

<a id="w004953"></a>

## W004953 — Rave Bare Bones

Bron-ID: W004953
Namespace: Main (0)
Revisie: 30335 | Datum: 2026-03-17T20:58:21Z
Bron: https://wiki.osdev.org/Rave_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Rave_Bare_Bones&oldid=30335

````text
{{Disputed|Talk:Rave Bare Bones}}
{{TutorialExplain}}
{{BeginnersWarning}}
{{Rating|1}}{{Template:Kernel designs}}

In this tutorial we will write a bootable kernel in the [[Rave]] programming language.

Tools needed to build the project:

* NASM
* Rave
* binutils (ld)
* QEMU (optional; for testing)


==Overview==

In this tutorial, we will create a simple NASM loader and a kernel capable of outputting the text we need - in this example, we will output "Hello, Rave!".

Our final project will consist of the following files:

* boot.asm
* kernel.rave
* linker.ld

==boot.asm==

<syntaxhighlight lang="asm">

MBALIGN  equ  1 << 0            ; align loaded modules on page boundaries
MEMINFO  equ  1 << 1            ; provide memory map
MBFLAGS  equ  MBALIGN | MEMINFO ; this is the Multiboot 'flag' field
MAGIC    equ  0x1BADB002        ; 'magic number' lets bootloader find the header
CHECKSUM equ -(MAGIC + MBFLAGS) ; checksum of above, to prove we are multiboot
                                ; CHECKSUM + MAGIC + MBFLAGS should be Zero (0)
section .multiboot
align 4
	dd MAGIC
	dd MBFLAGS
	dd CHECKSUM

section .bss
align 16
stack_bottom:
resb 16384 ; 16 KiB is reserved for stack
stack_top:

section .text
global _start:function (_start.end - _start)
_start:
	mov esp, stack_top

	extern kmain
	call kmain
	cli
.hang:	hlt
	jmp .hang
.end:

</syntaxhighlight>

Assemble that with:

<syntaxhighlight lang="bash">nasm -felf32 boot.asm -o boot.o</syntaxhighlight>

==kernel.rave==
<syntaxhighlight lang="d">
namespace terminal {
    (volatile) ushort* memory = 0xB8000;

    alias columns = 80;
    alias rows = 25;

    usize row = 0;
    usize column = 0;

    void clear {
        for (uint y=0; y<terminal::rows; y++) {
            for (uint x=0; x<terminal::columns; x++) terminal::memory[(y * terminal::rows) + x] = 0;
        }
    }

    void putchar(uchar symbol) {
        if (symbol == '\n') {
            terminal::column = 0;
            terminal::row += 1;
        }
        else {
            terminal::memory[(terminal::row * terminal::columns) + terminal::column] = (cast(ushort*)(&[symbol, cast(uchar)0x0F]))[0];
            terminal::column += 1;
        }

        if (terminal::column == terminal::columns) {
            terminal::column = 0;
            terminal::row += 1;

            if (terminal::row == terminal::rows) terminal::row = 0;
        }
    }

    void print(char* cstr) {
        usize i = 0;
        while (cstr[i] != '\0') { terminal::putchar(cstr[i]); i += 1; }
    }
}
 
(C) void kmain {
    terminal::clear();
    terminal::print("Hello, Rave!");

	while (true) {}
}
</syntaxhighlight>

Сompile that with:

<syntaxhighlight lang="bash">rave kernel.rave -o kernel.o -c --noStd --noEntry --noChecks -ffreestanding -t i686-linux</syntaxhighlight>

==linker.ld==
<syntaxhighlight lang="c">
OUTPUT_FORMAT(elf32-i386)

/* The bootloader will look at this image and start execution at the symbol
   designated as the entry point. */
ENTRY(_start)

/* Tell where the various sections of the object files will be put in the final
   kernel image. */
SECTIONS
{
	/* It used to be universally recommended to use 1M as a start offset,
	   as it was effectively guaranteed to be available under BIOS systems.
	   However, UEFI has made things more complicated, and experimental data
	   strongly suggests that 2M is a safer place to load. In 2016, a new
	   feature was introduced to the multiboot2 spec to inform bootloaders
	   that a kernel can be loaded anywhere within a range of addresses and
	   will be able to relocate itself to run from such a loader-selected
	   address, in order to give the loader freedom in selecting a span of
	   memory which is verified to be available by the firmware, in order to
	   work around this issue. This does not use that feature, so 2M was
	   chosen as a safer option than the traditional 1M. */
	. = 2M;

	/* First put the multiboot header, as it is required to be put very early
	   in the image or the bootloader won't recognize the file format.
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
	}

	/* The compiler may produce other sections, by default it will put them in
	   a segment with the same name. Simply add stuff here as needed. */
}
</syntaxhighlight>

After that, you can link all of that with:

<syntaxhighlight lang="bash">ld -melf_i386 -T linker.ld -o myos.bin boot.o kernel.o</syntaxhighlight>

Your kernel is now named as myos.bin. You can try to boot it using QEMU:

<syntaxhighlight lang="bash">qemu-system-i386 -kernel myos.bin</syntaxhighlight>

[[Category:Bare bones tutorials|Rave bare bones]]
[[Category:Rave]]
````

<a id="w004954"></a>

## W004954 — File:ISBN 9780128154120.jpg

Bron-ID: W004954
Namespace: File (6)
Revisie: 29668 | Datum: 2025-09-18T02:33:51Z
Bron: https://wiki.osdev.org/File:ISBN_9780128154120.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_9780128154120.jpg&oldid=29668

````text

````

<a id="w004955"></a>

## W004955 — File:ISBN 1558603204.jpg

Bron-ID: W004955
Namespace: File (6)
Revisie: 29669 | Datum: 2025-09-18T04:00:40Z
Bron: https://wiki.osdev.org/File:ISBN_1558603204.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_1558603204.jpg&oldid=29669

````text
== Summary ==
Advanced Compiler Design & Implementation cover
````

<a id="w004956"></a>

## W004956 — ATI Rage 128

Bron-ID: W004956
Namespace: Main (0)
Revisie: 30877 | Datum: 2026-07-09T01:38:13Z
Bron: https://wiki.osdev.org/ATI_Rage_128
Revisie-URL: https://wiki.osdev.org/index.php?title=ATI_Rage_128&oldid=30877

````text
{{Stub}}

The '''ATI Rage 128''' is a series of graphics cards from the late 1990s. It's noteworthy as a candidate for device driver development as the '''Rage 128 Pro''' is emulated in [[QEMU]].

Models sharing this register programming interface include:

* ATI Rage 128 VR
* ATI Rage 128 GL
* ATI Rage 128 Pro
* ATI Rage 128 Ultra
* ATI Rage Fury MAXX

It has the usual three compatibility programming modes, which won't be discussed further here:
* VGA BIOS
* [[VBE|VESA BIOS Extensions]]
* [[VGA Hardware|VGA Registers]]

Apart from that, it has its own programming mode exposed on the PCI/AGP bus. This is what the rest of this article will focus on.

We will focus on the 2D framebuffer and 2D drawing engine.

== Card Initialisation ==

Before the framebuffer or the 2D or 3D engines can be used, the card must be initialised into a known state. This is complicated by the fact that depending on whether this card is the primary video card in the system it may or may not have had its VGA BIOS (PCI bootable ROM) executed by the BIOS.

Initialising the card involves:
# Detect the card via [[PCI]] and read BAR0 (framebuffer memory) and BAR2 (memory mapped I/O). BAR1 can be ignored as it just provides a subset of BAR2.
# (Optional) enable [[AGP]] speed
# If using paging, map the MMIO, PCI BIOS ROM, and framebuffer into accessible address space.
# Read the memory specifications and clock specifications from the PCI ROM (aty128_get_pllinfo() in the Linux fbdev source is a good start).
# Unless you are certain the VGA BIOS has been executed, program the card's primary GPU clock (XCLK), memory clock (MCLK), and memory information into the card's registers.

== Getting a 2D Framebuffer ==

Prerequisites: PCI Bus Probing, knowledge of VGA signal timing.

Basically, you want to do the following things:

# Determine the mode settings to use (either making assumptions, or using DDC/EDID).
# Enable the accelerator, turn off the CRTC and pixel clock.
# Set the pixel clock settings (if using real hardware, QEMU doesn't care about the pixel clock). This is VCLK in the documentation.
# Turn on the pixel clock and wait for it to stabilise.
# Set the CRTC with VGA timings that will work.
# Turn on CRTC.
# Start drawing pixels

=== Determining Mode Settings ===

See [http://www.tinyvga.com/vga-timing] and the [[Video Signals And Timing]] page.

These timings are in pixels, and should be usable across different video adapters (as long as they are supported by the video card and the monitor, an exercise left to the reader). Video cards will be limited mostly by their maximum pixel clock and memory size.

<syntaxhighlight lang="c">
typedef struct {
    uint16 hRes;
    uint16 vRes;

    uint16 hFront;
    uint16 hSync;
    uint16 hBack;

    /// @brief Horizontal Sync Polarity. 1 is positive.
    bool hSyncPolarity;

    uint16 vFront;
    uint16 vSync;
    uint16 vBack;

    /// @brief Vertical sync polarity. 1 is positive.
    bool vSyncPolarity;

    /// @brief Pixel clock in Hz.
    uint32 pixelClock;

    /// @brief Number of bits per pixel.
    uint8 depth;
} VideoMode;

const VideoMode videoMode1024x768 = {
    1024, 768, 
    24, 136, 160, 0,
    3, 6, 29, 0,
    65000000, 32
};

const VideoMode videoMode640x480 = {
    640, 480,
    16, 96, 48, 0,
    10, 2, 33, 0,
    25175000, 32
};
</syntaxhighlight>

=== CRTC Setup ===

First we determine a number of intermediate variables. For the Rage 128, most horizontal measurements are done in 'characters', blocks of 8 pixels.

<syntaxhighlight lang="c">
// Pick our video mode.
VideoMode mode = videoMode640x480;

uint32 hSyncStart = (mode.hRes + mode.hFront) / 8;
uint32 hSyncWidth = mode.hSync / 8;
uint32 hTotal = (mode.hRes + mode.hFront + mode.hSync + mode.hBack) / 8;
uint32 hEnd = (mode.hRes / 8) -1;

uint32 vSyncStart = (mode.vRes + mode.vFront);
uint32 vSyncWidth = mode.vSync;
uint32 vTotal = (mode.vRes + mode.vFront + mode.vSync + mode.vBack);
uint32 vEnd = (mode.vRes) -1;

uint8 hSyncPol = 1 - mode.hSyncPolarity;
uint8 vSyncPol = 1 - mode.vSyncPolarity;

uint32 depth = 0;

switch(mode.depth) {
    case 32:
        depth = 6;
        break;

    default:
        return ERROR;
}
</syntaxhighlight>

Next we write these values to the CRTC registers. Note that the two sync pulse registers are ignored by QEMU, but are very much required by real hardware.

<syntaxhighlight lang="c">
regs[ATIRAGE128_REGOFFSET_CRTC_H_TOTAL_DISP] = hTotal | (hEnd << 16);
regs[ATIRAGE128_REGOFFSET_CRTC_V_TOTAL_DISP] = vTotal | (vEnd << 16);

regs[ATIRAGE128_REGOFFSET_CRTC_H_SYNC_STRT_WID] = (hSyncStart << 3) | (hSyncWidth << 16) | (hSyncPol << 23);   // QEMU does not use this register at all.

regs[ATIRAGE128_REGOFFSET_CRTC_V_SYNC_STRT_WID] = (vSyncStart) | (vSyncWidth << 16) | (vSyncPol << 23);        // QEMU deso not use this register at all.

// Number of characters the screen is wide.
regs[ATIRAGE128_REGOFFSET_CRTC_PITCH] = (mode.hRes) / 8;
</syntaxhighlight>

=== Setting the Pixel Clock ===

TODO. The pixel clock is completely ignored by QEMU, only used by real hardware.

=== Enabling the CRTC ===
<syntaxhighlight lang="c">
// Enable, 32bpp, extended.
regs[ATIRAGE128_REGOFFSET_CRTC_GEN_CTRL] = (depth << 8) | (1 << 24) | (1 << 25);
</syntaxhighlight>

== See Also ==

=== Hardware Specifications ===
* [http://www.bitsavers.org/components/ati/Rage_128/Rage_128_Reg_Ref_Guide.pdf Rage 128 GL/VR Register Reference Manual]
* [http://www.bitsavers.org/components/ati/Rage_128/RRG-G04500-C_RAGE_128_PRO_Register_Reference_Guide_Rev_1.01_200001.pdf Rage 128 Pro Register Reference Manual]

=== Emulator and Driver Source ===
* [https://github.com/qemu/qemu/blob/master/hw/display/ati.c QEMU Emulator Soure Code]
* [https://gitlab.freedesktop.org/xorg/driver/xf86-video-r128 xFree86 Driver Source Code]
* [https://github.com/torvalds/linux/blob/master/drivers/video/fbdev/aty/aty128fb.c Linux Kernel FBDev Source Code]

=== Other ===
* [https://en.wikipedia.org/wiki/ATI_Rage Wikipedia Page - Useful overview of model range]
* [http://www.tinyvga.com/vga-timing VGA Timing Reference]

[[Category:Video]]
````

<a id="w004957"></a>

## W004957 — Talk:Shutdown

Bron-ID: W004957
Namespace: Talk (1)
Revisie: 29705 | Datum: 2025-10-11T21:27:15Z
Bron: https://wiki.osdev.org/Talk:Shutdown
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AShutdown&oldid=29705

````text
== Shutdown in QEMU not always on port 0x604? ==

For various reasons I'm writing my own x86_64 BIOS for QEMU, and I've found that nothing is present on <code>0x604</code>.
Tracing while running a Linux guest reveals that under normal circumstances <code>'acpi-cnt'</code> should be present there, though in the case of my custom BIOS it does not appear to be there.
Looking at QEMU source code, it calls <code>qemu_system_shutdown_request(SHUTDOWN_CAUSE_GUEST_SHUTDOWN);</code>, so that does look right.
Checking with <code>info mtree</code>, it appears that <code>'acpi-cnt'</code> is at <code>0x4</code>, though masked by <code>'acpi-dma'</code>.
I'm guessing SeaBIOS is somehow telling QEMU to put <code>'acpi-cnt'</code> at <code>0x604</code>.

Looking through SeaBIOS code, I find:
* <code>u16 acpi_pm_base = 0xb000;</code>
* <code>acpi_pm_base = 0x0600;</code>
* <code>acpi_pm1a_cnt = acpi_pm_base + 0x04</code>
... but it doesn't seem to be writing that anywhere? Except:
* <code>pci_config_writel(bdf, PIIX_PMBASE, acpi_pm_base | 1);</code>
I suppose that has to be it, let's see further:
* <code>#define PIIX_PMBASE               0x40</code>
* <code>PORT_PCI_CMD</code> and <code>PORT_PCI_DATA</code> at <code>0xcf8</code> and <code>0xcfc</code> respectively.
* <code>0x80000000 | (bdf << 8) | (addr & 0xfc)</code>
"bdf" is "Bus Device Function" I guess. We should probably enumerate the PCI bus then...

I will post an update once I've got PCI enumeration working.

[[User:Demindiro|Demindiro]] ([[User talk:Demindiro|talk]]) 21:26, 11 October 2025 (UTC)
````

<a id="w004958"></a>

## W004958 — User:Kouosi

Bron-ID: W004958
Namespace: User (2)
Revisie: 29712 | Datum: 2025-10-17T07:31:17Z
Bron: https://wiki.osdev.org/User:Kouosi
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKouosi&oldid=29712

````text
Hi I am kouosi!
````

<a id="w004959"></a>

## W004959 — User:Kerneldestroyer

Bron-ID: W004959
Namespace: User (2)
Revisie: 29726 | Datum: 2025-10-23T20:52:56Z
Bron: https://wiki.osdev.org/User:Kerneldestroyer
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKerneldestroyer&oldid=29726

````text
I use vm because I brick os
````

<a id="w004960"></a>

## W004960 — File:Symbos.gif

Bron-ID: W004960
Namespace: File (6)
Revisie: 29727 | Datum: 2025-10-23T21:16:32Z
Bron: https://wiki.osdev.org/File:Symbos.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASymbos.gif&oldid=29727

````text
== Summary ==
SymbOS screenshot
````

<a id="w004961"></a>

## W004961 — Doors (IPC)

Bron-ID: W004961
Namespace: Main (0)
Revisie: 29745 | Datum: 2025-11-04T18:01:22Z
Bron: https://wiki.osdev.org/Doors_(IPC)
Revisie-URL: https://wiki.osdev.org/index.php?title=Doors_%28IPC%29&oldid=29745

````text
{{stub}}

Doors are a form of IPC pioneered by Spring OS. The essence is to directly "call" a procedure in another process.

== See Also ==

=== External Links ===

* [https://f.osdev.org/viewtopic.php?t=52225 IPC by "process hopping" (forum post)] a different (and worse) idea that happens to be very closely related to the Doors concept.
* [https://docs.oracle.com/en/operating-systems/solaris/oracle-solaris/11.4/prog-interfaces/doors-overview.html Doors Overview]
* [http://kohala.com/start/papers.others/doors.html "Doors" in SolarisTM: Lightweight RPC using File Descriptors]
* [https://web.archive.org/web/20060315090538/http://research.sun.com/features/tenyears/volcd/papers/Mitchell.pdf An Overview of the Spring System] [https://web.archive.org/web/20100116211311/http://research.sun.com/features/tenyears/volcd/papers/mitchell.htm (blurb)]
* [https://web.archive.org/web/20060314222035/http://research.sun.com/techrep/1993/smli_tr-93-14.pdf The Spring Nucleus: A Microkernel for Objects]

[[Category:IPC]]
[[Category:OS theory]]
````

<a id="w004962"></a>

## W004962 — Advanced Interrupt Architecture

Bron-ID: W004962
Namespace: Main (0)
Revisie: 30817 | Datum: 2026-06-24T14:58:21Z
Bron: https://wiki.osdev.org/Advanced_Interrupt_Architecture
Revisie-URL: https://wiki.osdev.org/index.php?title=Advanced_Interrupt_Architecture&oldid=30817

````text
{{stub}}

The '''Advanced Interrupt Architecture''' (AIA) is a specification for handling interrupts on RISC-V platforms. It is more advanced than the [[PLIC]], supporting features such as [[Message Signaled Interrupts|MSI]] which are unsupported by PLIC.

== CSRs ==

AIA adds the following CSRs:

=== Machine-Level Window to Indirectly Accessed Registers ===

{| class="wikitable"
! Number !! Read/Write !! Width (bits) !! Name !! Description
|-
| 0x350
| MRW
| XLEN
| <code>miselect</code>
|
|-
| 0x351
| MRW
| XLEN
| <code>mireg</code>
|
|}

=== Supervisor-Level Window to Indirectly Accessed Registers ===

{| class="wikitable"
! Number !! Read/Write !! Width (bits) !! Name !! Description
|-
| 0x150
| SRW
| XLEN
| <code>siselect</code>
| Supervisor indirect register select
|-
| 0x151
| SRW
| XLEN
| <code>sireg</code>
| Supervisor indirect register alias
|}

=== Supervisor-Level Interrupts ===

{| class="wikitable"
! Number !! Read/Write !! Width (bits) !! Name !! Description
|-
| 0x104
| SRW
| 64
| <code>sie</code>
| Supervisor interrupt-enable bits
|-
| 0x144
| SRW
| 64
| <code>sip</code>
| Supervisor interrupt-pending bits
|-
| 0x15C
| SRW
| SXLEN
| <code>stopei</code>
| Supervisor top external interrupt (only with an IMSIC)
|-
| 0xDB0
| SRO
| SXLEN
| <code>stopi</code>
| Supervisor top interrupt
|-
| 0x114
| SRW
| 32
| <code>sieh</code>
| Upper 32 bits of <code>sie</code> (RV32 only)
|-
| 0x154
| SRW
| 32
| <code>siph</code>
| Upper 32 bits of <code>sip</code> (RV32 only)
|}

== APLIC (Advanced Platform-Level Interrupt Controller) ==

== IMSIC (Incoming Message-Signaled Interrupt Controller) ==

As the name implies, the IMSIC is designed to convert MSI notifications into interrupts.

Each IMSIC is associated with a single hart. An IMSIC consists of one or more ''interrupt files'', at least one for each privilege level (except U-mode(?)). Each interrupt file corresponds to a single interrupt vector, but MSIs from multiple sources can be distinguished by ''identity numbers''.

An interrupt file is always aligned on a 4-KiB (page) boundary. Only two 32-bit registers are specified:

{| class="wikitable"
! Offset !! Name !! Description
|-
| <code>0x000</code>
| <code>seteipnum_le</code>
| For little-endian systems
|-
| <code>0x004</code>
| <code>seteipnum_be</code> 
| For big-endian systems
|}

Writing a value <code>i</code> will set the pending bit of identity number <code>i</code>.

The MSI source can be determined by inspecting the <code>*topei</code> CSR. If it is non-zero, an interrupt has been signaled and the following bits are set:

{| class="wikitable"
! Bits !! Description
|-
| 26:16
| Interrupt identity
|-
| 10:0
| Interrupt priority (same as identity)
|}

=== CSRs ===

These CSRs are ''indirectly'' accessed by first writing <code>*iselect</code> and then reading or writing <code>*ireg</code>.

{| class="wikitable"
! ID !! Name
|-
| <code>0x70</code>
| <code>eidelivery</code>
|-
| <code>0x72</code>
| <code>eithreshold</code>
|-
| <code>0x80</code>
| <code>eip0</code>
|-
| <code>0x81</code>
| <code>eip1</code>
|-
| …
| …
|-
| <code>0xBF</code>
| <code>eip63</code>
|-
| <code>0xC0</code>
| <code>eie0</code>
|-
| <code>0xC1</code>
| <code>eie1</code>
|-
| …
| …
|-
| <code>0xFF</code>
| <code>eie63</code>
|}

== See Also ==

=== External Links ===

* [https://github.com/riscv/riscv-aia RISC-V Advanced Interrupt Architecture (AIA)]

[[Category:RISC-V]]
[[Category:Interrupts]]
````

<a id="w004963"></a>

## W004963 — User:Demindiro/Advanced Interrupt Architecture

Bron-ID: W004963
Namespace: User (2)
Revisie: 29742 | Datum: 2025-11-01T19:33:53Z
Bron: https://wiki.osdev.org/User:Demindiro/Advanced_Interrupt_Architecture
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADemindiro%2FAdvanced_Interrupt_Architecture&oldid=29742
Doorverwijzing: Advanced Interrupt Architecture

````text
#REDIRECT [[Advanced Interrupt Architecture]]
````

<a id="w004964"></a>

## W004964 — User:Demindiro/Doors (IPC)

Bron-ID: W004964
Namespace: User (2)
Revisie: 29746 | Datum: 2025-11-04T18:01:22Z
Bron: https://wiki.osdev.org/User:Demindiro/Doors_(IPC)
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADemindiro%2FDoors_%28IPC%29&oldid=29746
Doorverwijzing: Doors (IPC)

````text
#REDIRECT [[Doors (IPC)]]
````

<a id="w004965"></a>

## W004965 — File:Transmit Ring Structure.png

Bron-ID: W004965
Namespace: File (6)
Revisie: 29750 | Datum: 2025-11-09T18:53:44Z
Bron: https://wiki.osdev.org/File:Transmit_Ring_Structure.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ATransmit_Ring_Structure.png&oldid=29750

````text
Depicts the structure of the transmit ring, for the 8254x series of Network Controllers
````

<a id="w004966"></a>

## W004966 — File:Receive Ring Structure.png

Bron-ID: W004966
Namespace: File (6)
Revisie: 29751 | Datum: 2025-11-09T18:54:49Z
Bron: https://wiki.osdev.org/File:Receive_Ring_Structure.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AReceive_Ring_Structure.png&oldid=29751

````text
This image depicts the structure of the Receive Ring, for the 8254x series of Network Controllers
````

<a id="w004967"></a>

## W004967 — Why function implementations shouldn't be put In header files

Bron-ID: W004967
Namespace: Main (0)
Revisie: 29755 | Datum: 2025-11-11T16:08:06Z
Bron: https://wiki.osdev.org/Why_function_implementations_shouldn%27t_be_put_In_header_files
Revisie-URL: https://wiki.osdev.org/index.php?title=Why_function_implementations_shouldn%27t_be_put_In_header_files&oldid=29755
Doorverwijzing: Why function implementations shouldn't be put in header files

````text
#REDIRECT [[Why function implementations shouldn't be put in header files]]
````

<a id="w004968"></a>

## W004968 — File:Amiga Guru Meditation.gif

Bron-ID: W004968
Namespace: File (6)
Revisie: 29792 | Datum: 2025-12-18T13:40:58Z
Bron: https://wiki.osdev.org/File:Amiga_Guru_Meditation.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAmiga_Guru_Meditation.gif&oldid=29792

````text
== Summary ==
By User:Mooseblaster - https://commons.wikimedia.org/wiki/File:Guru_meditation.gif, Public Domain, https://commons.wikimedia.org/w/index.php?curid=141665312
````

<a id="w004969"></a>

## W004969 — File:ISBN 0132017997.jpg

Bron-ID: W004969
Namespace: File (6)
Revisie: 29799 | Datum: 2025-12-24T22:03:09Z
Bron: https://wiki.osdev.org/File:ISBN_0132017997.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0132017997.jpg&oldid=29799

````text
== Summary ==
the design of the unix operating system
````

<a id="w004970"></a>

## W004970 — User talk:Salimanz23

Bron-ID: W004970
Namespace: User talk (3)
Revisie: 29837 | Datum: 2026-01-07T09:22:15Z
Bron: https://wiki.osdev.org/User_talk:Salimanz23
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ASalimanz23&oldid=29837

````text
== Reverted edits on BeFS and XFS ==

Hey, I reverted your edits to [[BeFS]] and [[XFS]] because they appear to be AI generated. Your edit to BeFS reads like an advertisement, and doesn't give actual on-disk formats. Your edit to XFS gives an incomplete description of the inode structure. LLMs often make mistakes and hallucinate false information, and shouldn't be relied on to generate documentation.

You are, of course, welcome to expand on articles lacking important information, but please make sure the information you add is relevant and correct.

-[[User:Finxx|Finxx]] ([[User talk:Finxx|talk]]) 00:45, 30 December 2025 (UTC)

== Talk Page ==

Please keep the talk page intact and edit the regular user page instead. [[User:Demindiro|Demindiro]] ([[User talk:Demindiro|talk]]) 09:22, 7 January 2026 (UTC)
````

<a id="w004971"></a>

## W004971 — User talk:Finxx

Bron-ID: W004971
Namespace: User talk (3)
Revisie: 29822 | Datum: 2026-01-02T07:52:24Z
Bron: https://wiki.osdev.org/User_talk:Finxx
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AFinxx&oldid=29822

````text
Hello Finxx,

Pardon phrasing. I am ESL. I wrote both articles in Japanese then translated to English using AI. As you see, when I try to write casual it is okay. Try to write tech? Difficult!

I did not intend to cause a problem. I will make small edit, proofread carefully, and if there is problem because then only roll back that part, not entire thing! I worked hard on it!

BeFS was WIP. 

I will on XFS only quote tech docs directly for inode to avoid future issues.

[[User:Salimanz23|Salimanz23]] ([[User talk:Salimanz23|talk]]) 04:52, 31 December 2025 (UTC)

: Ah, that makes sense. Do not worry about this, it is not a large issue. The accuracy of information on OSDev is important, but so long as you make sure everything you add is correct, there won't be any problems. If anything, it is good to finally see work done on the file system articles; those have long been OSDev's weakest spot. --[[User:Finxx|Finxx]] ([[User talk:Finxx|talk]]) 07:52, 2 January 2026 (UTC)
````

<a id="w004972"></a>

## W004972 — Talk:BFS

Bron-ID: W004972
Namespace: Talk (1)
Revisie: 29839 | Datum: 2026-01-08T02:57:30Z
Bron: https://wiki.osdev.org/Talk:BFS
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABFS&oldid=29839

````text
:''From what i see, it also means BFS will have to do nasty things if a file must be extended after some other file has been created'' -- PypeClicker

:''Agreed, but it's not a general-purpose filesystem. One tends not to extend things like the kernel image or modules.'' -- Strib

:''What about updates? Also, why aren't we using the discussion page instead?'' --[[User:Love4boobies|Love4boobies]] 12:06, 7 January 2011 (UTC)
````

<a id="w004973"></a>

## W004973 — User:Exoosdev/ExoOs

Bron-ID: W004973
Namespace: User (2)
Revisie: 30808 | Datum: 2026-06-22T05:02:46Z
Bron: https://wiki.osdev.org/User:Exoosdev/ExoOs
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AExoosdev%2FExoOs&oldid=30808

````text
=== Development ===



Development started on January 5th 2026.  But it has been halted due to a computer change and lack of interest.

Current development is on the bootloader due to the exobition not wanted to go through the trouble of using one like GRUB or Limine.

The ExoOS bootloader uses intel syntax.  Because thats what exobition first learned.


as of aprox. 2026-1-29 development has temporarily halted.


as of 2026-6-21 development may proceed.


as of 2026-1-15 development on ExoOS school repo has been completely abandoned and wont be used (except for archival purposed).
=== Sources ===


[https://github.com/exobition/ExoOS ExoOS Github home updates]

[https://github.com/exodev-source/ExoOS-at-school ExoOS Github school updates]



I own ExoOS copyright so osdev mods dont take this down
````

<a id="w004974"></a>

## W004974 — ExoOs

Bron-ID: W004974
Namespace: Main (0)
Revisie: 29855 | Datum: 2026-01-10T19:13:45Z
Bron: https://wiki.osdev.org/ExoOs
Revisie-URL: https://wiki.osdev.org/index.php?title=ExoOs&oldid=29855
Doorverwijzing: User:Exoosdev/ExoOs

````text
#REDIRECT [[User:Exoosdev/ExoOs]]
````

<a id="w004975"></a>

## W004975 — PA-RISC

Bron-ID: W004975
Namespace: Main (0)
Revisie: 29961 | Datum: 2026-02-10T23:34:36Z
Bron: https://wiki.osdev.org/PA-RISC
Revisie-URL: https://wiki.osdev.org/index.php?title=PA-RISC&oldid=29961

````text
{{In Progress}}
'''PA-RISC''' (Precision Architecture RISC) also known as '''HPPA''' (Hewlett Packard Precision Architecture) is a RISC CPU architecture created by Hewlett Packard in 1985. Its most common use was on the HP 9000 line of servers and workstations. Officially it was succeeded by PA-WideWord, a VLIW-based architecture, later renamed to [[IA-64|Itanium]]. The last CPU was the PA-8900 "Shortfin" released in 2005 and the last HP workstation using a PA-RISC CPU (PA-8800 or PA-8900) was the HP 9000 c8000, introduced in 2004.

== Features ==
* Originally 32-bit for PA1.0, extended to 64-bit in PA2.0
* Thirty-two general purpose integer registers (<code>GR0</code> to <code>GR31</code>):
** <code>GR0</code> to <code>GR2</code> and <code>GR31</code> are special:
*** <code>GR0</code> always yields zero when read, any value that is written to it is discarded
*** <code>GR1</code> holds the result of the <code>addil</code> instruction
*** <code>GR2</code> return address for the branch and link instructions, also called "return pointer". [[GAS]] also names it <code>rp</code>.
*** <code>GR31</code> return address for the "branch external" instructions
*** Additionally, <code>GR30</code> is commonly used as the stack pointer register, with the <code>sp</code> name aliased to it
** [[GAS]] uses the <code>r</code> prefix instead of <code>GR</code>
* Floating point registers, thirty-two in PA2.0 and sixteen in PA1.0 (<code>fr0</code> to <code>fr31</code> in [[GAS]] for the double-word format):
** For PA2.0, these registers can be accessed either as 32 bit registers or 64 bit ones
* [[Endianness|Big-endian]] only in PA1.0, optionally bi-endian for versions ≥1.1
* SIMD extensions (Multimedia Acceleration eXtensions—MAX), added in PA2.0
* Completers, which are used as a way of specifying "options" to the opcode. Some completers are <code>,n</code> for branch nullification, <code>,c</code> for carry, <code>,z</code> for zero...
* Branch delay slot, which can be skipped (nullified) by setting the <code>n</code> bit present on the branch instructions; in assembly this is done using the <code>,n</code> completer (e.g. <code>bv,n r0(rp)</code>)

== Compiler support ==
* [[GCC]], running on Linux and HP-UX, along with glibc (GNU C Standard Library) support<ref>[https://parisc.docs.kernel.org/en/latest/gcc.html GCC - GNU Compiler Collection - Linux on PA-RISC]</ref>
* HP C/aC++ compiler, included with HP-UX

== OS support ==
* HP-UX, HP's commercial proprietary UNIX variant that supported PA-RISC until version 11.31
* NetBSD has a Tier 2 port available
* Linux, from the PA-RISC/Linux project, mainline support merged in version 2.6

== Emulator support ==
* [[QEMU]], can emulate the HP B160L (PA-7100LC, 32-bit, PA1.1) and the HP C3700 workstations (PA-8700, 64-bit, PA2.0) <ref>[https://parisc.docs.kernel.org/en/latest/qemu.html QEMU for PA-RISC overview]</ref>

== References ==
<references/>

== External links ==
* [http://ftp.parisc-linux.org/docs/arch/parisc2.0.pdf PA-RISC 2.0 Architecture] In-depth full description of the architecture
* [https://parisc.docs.kernel.org/en/latest/_downloads/f15883d2c095ac4e7606ac263ab8d883/Pa11_acd.pdf PA-RISC 1.1 Architecture and Instruction Set Reference Manual]
* [https://web.archive.org/web/20150421181801/http://h21007.www2.hp.com/portal/site/dspp/menuitem.1b39e60a9475acc915b49c108973a801?chid=6dc55e210a66a010VgnVCM100000275d6e10RCRD HP References & Manuals] Archive of HP documentation, including PA-RISC
* [https://www.openpa.net/ OpenPA] PA-RISC information archive
* [https://www.openpa.net/pa-risc_processors.html PA-RISC Processors]
* http://ftp.parisc-linux.org/docs/arch/ More documentation on the ABI, the firmware enviroment and the ISA itself
* [http://ftp.parisc-linux.org/docs/arch/pdc.pdf PA-RISC 2.0 Firmware Architecture Reference Specification Version 0.36]
* [http://ftp.parisc-linux.org/docs/arch/pa64rt.pdf 64-Bit Runtime Architecture for PA-RISC 2.0] PA-RISC 2.0 [[ABI]]
* [https://wiki.netbsd.org/ports/hppa/ NetBSD/hppa]
* [https://parisc.docs.kernel.org/en/latest/ Linux on PA-RISC]
* [https://parisc.docs.kernel.org/en/latest/technical_documentation.html Technical Documentation - Linux on PA-RISC] Assortment of technical documentation related to PA-RISC, also includes documentation on HP workstation chipsets, user and service manuals
* [https://parisc.docs.kernel.org/en/latest/qemu.html QEMU for PA-RISC overview] General instructions on how to use QEMU's PA-RISC emulation, not restricted to just running Linux
* [https://www.openpa.net/qemu_pa-risc_emulation.html QEMU PA-RISC Emulation] More on QEMU for this architecture
* [https://web.archive.org/web/20150518101220/http://www.lostcircuits.com/mambo//index.php?option=com_content&task=view&id=42&Itemid=42 Hewlett Packard PA-8800 RISC] Overview of the PA-8800 processor
* [https://kaashif.co.uk/2019/04/18/hp-pa-risc-assembly-crash-course/ HP PA-RISC Assembly Crash Course]
* [https://parisc.docs.kernel.org/en/latest/_downloads/3172fe3d8ffdd58f1cdad4550f3d2395/Pa2c96.pdf 64-bit and Multimedia Extensions in the PA-RISC 2.0 Architecture]

[[Category: PA-RISC]]
[[Category: Platforms]]
[[Category: Instruction Set Architecture]]
````

<a id="w004976"></a>

## W004976 — User:Tocariimaa/PA-RISC

Bron-ID: W004976
Namespace: User (2)
Revisie: 29889 | Datum: 2026-01-12T21:13:59Z
Bron: https://wiki.osdev.org/User:Tocariimaa/PA-RISC
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ATocariimaa%2FPA-RISC&oldid=29889
Doorverwijzing: PA-RISC

````text
#REDIRECT [[PA-RISC]]
````

<a id="w004977"></a>

## W004977 — HPPA

Bron-ID: W004977
Namespace: Main (0)
Revisie: 29890 | Datum: 2026-01-12T21:16:53Z
Bron: https://wiki.osdev.org/HPPA
Revisie-URL: https://wiki.osdev.org/index.php?title=HPPA&oldid=29890
Doorverwijzing: PA-RISC

````text
#REDIRECT [[PA-RISC]]
````

<a id="w004978"></a>

## W004978 — File:Development.png

Bron-ID: W004978
Namespace: File (6)
Revisie: 29903 | Datum: 2026-01-18T07:32:34Z
Bron: https://wiki.osdev.org/File:Development.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ADevelopment.png&oldid=29903

````text
Working prototype of bootloader that lets you choose what to do (set gdt, enable A20 line, start pm and load kernel etc..)
````

<a id="w004979"></a>

## W004979 — Itanium

Bron-ID: W004979
Namespace: Main (0)
Revisie: 29906 | Datum: 2026-01-20T19:52:44Z
Bron: https://wiki.osdev.org/Itanium
Revisie-URL: https://wiki.osdev.org/index.php?title=Itanium&oldid=29906
Doorverwijzing: IA-64

````text
#REDIRECT [[IA-64]]
````

<a id="w004980"></a>

## W004980 — Alpha

Bron-ID: W004980
Namespace: Main (0)
Revisie: 30145 | Datum: 2026-03-05T23:21:38Z
Bron: https://wiki.osdev.org/Alpha
Revisie-URL: https://wiki.osdev.org/index.php?title=Alpha&oldid=30145

````text
{{stub}}
{{In_Progress}}

'''DEC Alpha''' or just '''Alpha''' is a 64-bit [[RISC]] CPU architecture created by Digital Equipment Corporation (DEC) and later developed by Compaq, mainly targeted at the workstation and server market.


== Features ==
* Thirty-two 64-bit general purpose registers, <code>R0</code> to <code>R31</code>
** <code>R31</code> always yields zero when read, writes are discarded
* Thirty-two double-precision (quadword, 64-bit) floating point registers, <code>F0</code> to <code>F31</code>
** <code>F31</code> same as with its integer counterpart, this is always zero
* [[Endianness|Bi-endian]]
* SIMD instruction set extension, the Motion Video Instructions (MVI)

== Compiler support ==
* [[GCC]]
* Compaq Compiler Kit

== Emulator support ==
* [[QEMU]] (<code>qemu-system-alpha</code>)
* [https://github.com/lenticularis39/axpbox AXPbox]
* [https://opensimh.org/ Open SIMH]

== External links ==
* [http://www.alphalinux.org/wiki/index.php/Main_Page AlphaLinux wiki]
* [https://devblogs.microsoft.com/oldnewthing/20170807-00/?p=96766 The Alpha AXP] Review of the Alpha architecture by Raymond Chen

[[Category: DEC Alpha]]
[[Category: Platforms]]
[[Category: Instruction Set Architecture]]
````

<a id="w004981"></a>

## W004981 — DEC Alpha

Bron-ID: W004981
Namespace: Main (0)
Revisie: 29926 | Datum: 2026-01-22T18:16:18Z
Bron: https://wiki.osdev.org/DEC_Alpha
Revisie-URL: https://wiki.osdev.org/index.php?title=DEC_Alpha&oldid=29926
Doorverwijzing: Alpha

````text
#REDIRECT [[Alpha]]
````

<a id="w004982"></a>

## W004982 — UEFI From Scratch

Bron-ID: W004982
Namespace: Main (0)
Revisie: 30313 | Datum: 2026-03-17T18:59:22Z
Bron: https://wiki.osdev.org/UEFI_From_Scratch
Revisie-URL: https://wiki.osdev.org/index.php?title=UEFI_From_Scratch&oldid=30313

````text
{{In_Progress}}

Building a UEFI toolchain from the ground up is wide-scoped work, requiring custom compiler flags, linker scripts, and runtime support. This tutorial is targeted at C/C++ but will provide enough information and be generic enough that you can apply it to any language. This tutorial assumes a solid foundation of standard C/C++ build systems

== Required Reading == 
https://uefi.org/specifications As of this writing. The latest version is 2.11 (Dec 2024). Minimum required readings are the following
# Chapter 2 (Overview) - Provides expected calling conventions, memory layout/ownership, driver/protocol model, and basic typedefs
# Chapter 4 (System Table) - Data structure for the definition of boot & runtime services provided by the firmware
# Chapter 8.4 (SimpleTextOutputProtocol) - Basic character screen output

This tutorial will cover any details described in the specification because you should be following the spec during your implementation. If you are unsure of something or get stuck, double check the specification as its pretty messy and easy to miss stuff.

== Target ==
When building a UEFI toolchain, you will need to pick a specific architecture that you intend to target as each one has subtle differences that need to be accounted for. A unified library is possible but outside the scope of this tutorial. You will also need to choose what kind of EFI binary you are targeting. Whether it be an application, boottime driver, or runtime driver.

You will also need to pick a host environment. UEFI derives alot of its base specification and implementation from windows so other operating systems may require additional additional efforts to get going

The target choosen for this tutorial is an x86_64 Application compiled via linux cross-compiler

== Compiler ==
x86_64 UEFI uses the C calling convention for its firmware calling convention but follows the MSVC ABI over SystemV. This means your compiler either needs to support ms_abi attribute tags, setup an MSVC cross compiler, or create assembly trampolines to manage calls into and out of the UEFI firmware. 

The compiler choosen for this tutorial is a GCC MSVC cross compiler. This significantly reduces headaches and overheads of dealing with a dual ABI program, while allowing easier debugging as it can still emit DWARF debug symbols instead of PDB. 

There are several required compiler flags, mostly to stop the compiler from trying to abuse UB.
* -ffreestanding - No host kernel
* -fshort-wchar - 16 instead of 32bit wchar_t
* -fPIC - Relocatable code
* -fPIE
* -mno-red-zone - Prevents stack abuse

Without the required runtime support, the following compiler flags will also need to be added. It is possible to eventually implement enough runtime to remove all of these, but it is outside the scope of this guide
* -nostdlib - Missing new and delete
* -fno-exceptions - No stack unwinder
* -fno-rtti 
* -fno-stack-protector - Missing glibc intrinsics
* -fno-threadsafe-statics - Single Threaded until MP protocol is called
* -fno-asynchronous-unwind-tables

== Linker ==
UEFI uses the windows PE+ file format, and since UEFI is a non-standard target, we will also need to provide our own linker script. This tutorial does not cover the concept of writing a linker script so one is provided at the end of this section. 

There are also several required linker flags that must be used to properly flag the binary
* -m i386pep - x86_64 bit target
* --subsystem 10 - UEFI Application
* -e _start - Entry point for your CRT before calling EFI_MAIN()
* --image-base=0x400000 - Tells the firmware preferred location. Only useful for debugging and ensuring address stability between reboots

Final compiler + linker commands should look like the following
<syntaxhighlight lang="bash">
x86_64-w64-mingw32-g++ -nostdlib -ffreestanding -c -fno-exceptions -fno-rtti -fno-threadsafe-statics -fno-asynchronous-unwind-tables -fno-stack-protector -mno-red-zone -fPIC -fPIE -fshort-wchar main.cpp -o main.o

x86_64-w64-mingw32-g++ -nostdlib -ffreestanding -Wl,-T linker.ld -Wl,--image-base 0x400000 -Wl,-m i386pep -Wl,--subsystem 10 -Wl,-e _start main.o -o output/BOOTX64.efi
</syntaxhighlight>

<syntaxhighlight lang="linker">
OUTPUT_FORMAT(pei-x86-64)

SECTIONS
{
  /* ------------------------------------------------------------ */
  /* PE HEADERS */
  /* ------------------------------------------------------------ */
  . = SIZEOF_HEADERS;
  . = ALIGN(__section_alignment__);

  /* ------------------------------------------------------------ */
  /* TEXT */
  /* ------------------------------------------------------------ */
  .text __image_base__ + ( __section_alignment__ < 0x1000 ? . : __section_alignment__ ) :
  {
    /* Custom ASSERT infrastructure */
    KEEP(*(.assert_marker))
    KEEP(*(.assert_value))
    KEEP(*(.assert_msg))

    KEEP(*(SORT_NONE(.init)))
    *(.text)
    *(.text.*)
    *(.gnu.linkonce.t.*)

    . = ALIGN(16);
    KEEP(*(SORT_NONE(.fini)))

    PROVIDE(etext = .);
  }

  /* ------------------------------------------------------------ */
  /* C++ CONSTRUCTORS / DESTRUCTORS */
  /* ------------------------------------------------------------ */
  .init_array ALIGN(8) :
  {
    __init_array_start = .;
    KEEP(*(.init_array))
    KEEP(*(SORT(.init_array.*)))
    KEEP(*(.ctors))
    KEEP(*(SORT_BY_NAME(.ctors.*)))
    __init_array_end = .;
  }

  .fini_array ALIGN(8) :
  {
    __fini_array_start = .;
    KEEP(*(.fini_array))
    KEEP(*(SORT(.fini_array.*)))
    KEEP(*(.dtors))
    KEEP(*(SORT_BY_NAME(.dtors.*)))
    __fini_array_end = .;
  }

  /* ------------------------------------------------------------ */
  /* READ-ONLY DATA */
  /* ------------------------------------------------------------ */
  .rdata BLOCK(__section_alignment__) :
  {
    *(.rdata)
    *(.rdata.*)
  }

  /* ------------------------------------------------------------ */
  /* DATA */
  /* ------------------------------------------------------------ */
  .data BLOCK(__section_alignment__) :
  {
    __data_start__ = .;
    *(.data)
    *(.data.*)
    __data_end__ = .;
  }

  /* ------------------------------------------------------------ */
  /* BSS */
  /* ------------------------------------------------------------ */
  .bss BLOCK(__section_alignment__) :
  {
    __bss_start__ = .;
    *(.bss)
    *(COMMON)
    __bss_end__ = .;
  }

  /* ------------------------------------------------------------ */
  /* UNWINDING / STACK TRACING */
  /* ------------------------------------------------------------ */

  /* DWARF CFI (used by your own unwinder / debug output) */
  .eh_frame BLOCK(__section_alignment__) :
  {
    KEEP(*(.eh_frame*))
  }

  /* PE x64 exception metadata (REQUIRED if exceptions enabled) */
  .pdata BLOCK(__section_alignment__) :
  {
    KEEP(*(.pdata*))
  }

  .xdata BLOCK(__section_alignment__) :
  {
    KEEP(*(.xdata*))
  }

  /* ------------------------------------------------------------ */
  /* TLS */
  /* ------------------------------------------------------------ */
  .tls BLOCK(__section_alignment__) :
  {
    ___tls_start__ = .;
    *(.tls)
    *(.tls.*)
    ___tls_end__ = .;
  }

  /* ------------------------------------------------------------ */
  /* RESOURCES */
  /* ------------------------------------------------------------ */
  .rsrc BLOCK(__section_alignment__) : SUBALIGN(4)
  {
    *(.rsrc)
    *(.rsrc.*)
  }

  /* ------------------------------------------------------------ */
  /* RELOCATIONS */
  /* ------------------------------------------------------------ */
  .reloc BLOCK(__section_alignment__) :
  {
    *(.reloc)
  }

  /* ------------------------------------------------------------ */
  /* DEBUG INFORMATION (Runtime Stacktraces) */
  /* ------------------------------------------------------------ */

  /* DWARF 2–5 */
  .debug_aranges     BLOCK(__section_alignment__) : { *(.debug_aranges) }
  .debug_info        BLOCK(__section_alignment__) : { *(.debug_info .gnu.linkonce.wi.*) }
  .debug_abbrev      BLOCK(__section_alignment__) : { *(.debug_abbrev) }
  .debug_line        BLOCK(__section_alignment__) : { *(.debug_line) }
  .debug_frame       BLOCK(__section_alignment__) : { *(.debug_frame*) }
  .debug_str         BLOCK(__section_alignment__) : { *(.debug_str) }
  .debug_loc         BLOCK(__section_alignment__) : { *(.debug_loc) }
  .debug_ranges      BLOCK(__section_alignment__) : { *(.debug_ranges) }

  /* DWARF 5 */
  .debug_addr        BLOCK(__section_alignment__) : { *(.debug_addr) }
  .debug_line_str    BLOCK(__section_alignment__) : { *(.debug_line_str) }
  .debug_loclists    BLOCK(__section_alignment__) : { *(.debug_loclists) }
  .debug_rnglists    BLOCK(__section_alignment__) : { *(.debug_rnglists) }
  .debug_str_offsets BLOCK(__section_alignment__) : { *(.debug_str_offsets) }
  .debug_names       BLOCK(__section_alignment__) : { *(.debug_names) }

  /* ------------------------------------------------------------ */
  /* END SYMBOLS */
  /* ------------------------------------------------------------ */
  .endjunk BLOCK(__section_alignment__) :
  {
    PROVIDE(end = .);
    PROVIDE(_end = .);
    __end__ = .;
  }

  /* ------------------------------------------------------------ */
  /* DISCARD */
  /* ------------------------------------------------------------ */
  /DISCARD/ :
  {
    *(.drectve)
    *(.note.GNU-stack)
    *(.gnu.lto_*)
    *(.comment)
  }
}
</syntaxhighlight>

== Runtime ==
You should now be able to build a bootable and valid image. But without a valid C/C++ runtime environment, none of the standard library is available for you to use and certain language features like global constructors may have to be explicitly called in your code as that is normally handled by it.

=== TODO ===
* Constructors/Destructors
* Asserts
* __cxa_ glibc instrisics
* exit() & panic() handlers
* Accessing firmware protocols and resources

== Debugging ==
If your using the provided linker script and are on linux, then debugging is super easy with QEMU & GDB. Debug symbols are already setup to be included in the image. Simply add the -s -S arguments to tell it to halt the CPU on startup and setup a GDB stub server on localhost:1234. Only thing extra todo is build with debug symbols enabled
* -g
* -gdwarf64
* -gdwarf-5
* -ggdb

If your using OVMF, be warned that it will eat your breakpoints unless you unregister the #BP and #DB interrupt handlers via the DebugSupport protocol. An easy way around this is to use hardware breakpoints instead so you bypass the OVMF interrupt handler

OVMF has to run before your application can, so you will need to set a breakpoint at whatever you define your start point to be and let the firmware run its code first. 

I highly reccomend setting up a .gdbinit as it will save you a ton of time
<syntaxhighlight lang="gdb">
set arch i386:x86-64
set pagination off
target remote localhost:1234
add-symbol-file output/BOOTX64.efi -readnow
break efi_main
set tui compact-source on
tui enable
focus cmd
continue
</syntaxhighlight>

[[Category: UEFI]]
````

<a id="w004983"></a>

## W004983 — Category:Historical Curiosities

Bron-ID: W004983
Namespace: Category (14)
Revisie: 29979 | Datum: 2026-02-11T17:02:43Z
Bron: https://wiki.osdev.org/Category:Historical_Curiosities
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AHistorical_Curiosities&oldid=29979

````text
Articles describing historical, obsolete standards or designs relevant to OS development. These may still be useful for OSes targeting old computers.
````

<a id="w004984"></a>

## W004984 — User:Veganaize

Bron-ID: W004984
Namespace: User (2)
Revisie: 29981 | Datum: 2026-02-11T19:27:57Z
Bron: https://wiki.osdev.org/User:Veganaize
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AVeganaize&oldid=29981

````text
veganaiZe
````

<a id="w004985"></a>

## W004985 — Category:PA-RISC

Bron-ID: W004985
Namespace: Category (14)
Revisie: 30149 | Datum: 2026-03-06T01:55:56Z
Bron: https://wiki.osdev.org/Category:PA-RISC
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APA-RISC&oldid=30149

````text
'''PA-RISC''' (Precision Architecture RISC), also known as '''HPPA''' is a [[RISC]] CPU made by Hewlett-Packard, targeted at the server and workstation market.

[[Category:Platforms]]
````

<a id="w004986"></a>

## W004986 — AArch32

Bron-ID: W004986
Namespace: Main (0)
Revisie: 30026 | Datum: 2026-02-18T16:48:30Z
Bron: https://wiki.osdev.org/AArch32
Revisie-URL: https://wiki.osdev.org/index.php?title=AArch32&oldid=30026
Doorverwijzing: ARM Overview

````text
#REDIRECT [[ARM Overview]]
````

<a id="w004987"></a>

## W004987 — AMD-Vi IOMMU

Bron-ID: W004987
Namespace: Main (0)
Revisie: 30805 | Datum: 2026-06-20T22:26:35Z
Bron: https://wiki.osdev.org/AMD-Vi_IOMMU
Revisie-URL: https://wiki.osdev.org/index.php?title=AMD-Vi_IOMMU&oldid=30805

````text
{{stub}}

AMD-Vi provides a [[IOMMU]] to translate "Device Virtual Addresses" (DVAs) to "System Physical Addresses" (SPAs). Without nested paging, it is equivalent to translating linear/virtual addresses to physical addresses. It is also able to remap interrupt lines.

The IOMMU can be located in the PCI configuration space and is identified by checking class=0x8, subclass=0x6 and progif=0x0. None of the BAR registers are used, instead registers in capability space (id=0xf) must be used.

== Overview ==

The IOMMU requires several in-memory structures to operate.
Communication with the IOMMU is primarily done through ring buffers to improve throughput.

=== Device table ===

The device table describes how to handle transactions for devices behind the IOMMU.
There is one entry for each device, up to 65536 (the maximum for a single PCI segment).

By default, the array must be contiguous in memory. As each entry is 32 bytes it means a maximum of 2MiB is required, though it can be shorter if the start/end range contains no devices. The IOMMU may support a segmented device table to save further memory.

=== I/O page tables ===

The I/O page tables are compatible with [[Paging#64-Bit_Paging|long-mode]] page tables. It uses a number of the AVL bits to set device permissions and other attributes.

=== Interrupt remapping ===

=== Nested translation ===

=== Command buffer ===

=== Event log ===

== Device table ==

{| class="wikitable"
|+ Device Table Entry (DTE)
!
! 31
! 30
! 29
! 28
! 27
! 26
! 25
! 24
! 23
! 22
! 21
! 20
! 19
! 18
! 17
! 16
! 15
! 14
! 13
! 12
! 11
! 10
!  9
!  8
!  7
!  6
!  5
!  4
!  3
!  2
!  1
!  0
|-
! +0
|colspan="20"| Host Page Table Root Pointer[31:12]
|colspan="3" | Mode
|colspan="2" | HAD
|colspan="5" | ''res''
|colspan="1" | TV
|colspan="1" | V
|-
! +4
|colspan="1" | ''res''
|colspan="1" | IW
|colspan="1" | IR
|colspan="3" | GCR3 TRP[14:12]
|colspan="2" style="writing-mode:vertical-rl"| GLX
|colspan="1" | GV
|colspan="1" style="writing-mode:vertical-rl"| GIoV
|colspan="1" style="writing-mode:vertical-rl"| GPRP
|colspan="1" | PPR
|colspan="20"| Host Page Table Root Pointer[51:32]
|-
! +8
|colspan="16"| GCR3 Table Root Pointer[30:15]
|colspan="16"| Domain ID[15:0]
|-
! +12
|colspan="21"| GCR3 Table Root Pointer[51:31]
|colspan="1" style="writing-mode:vertical-rl"| SATS
|colspan="2" style="writing-mode:vertical-rl"| SysMgt
|colspan="1" | EX
|colspan="1" | SD
|colspan="1" style="writing-mode:vertical-rl"| Cache
|colspan="2" style="writing-mode:vertical-rl"| IoCtl
|colspan="1" | SA
|colspan="1" | SE
|colspan="1" | I
|-
! +16
|colspan="26"| Interrupt Table Root Pointer[31:6]
|colspan="1" | IG
|colspan="4" | IntTabLen
|colspan="1" | IV
|-
! +20
|colspan="1" style="writing-mode:vertical-rl"| Lint1Pass
|colspan="1" style="writing-mode:vertical-rl"| Lint0Pass
|colspan="2" style="writing-mode:vertical-rl"| IntCtl
|colspan="1" style="writing-mode:vertical-rl"| HPTMode
|colspan="1" style="writing-mode:vertical-rl"| NMIPass
|colspan="1" style="writing-mode:vertical-rl"| EIntPass
|colspan="1" style="writing-mode:vertical-rl"| InitPass
|colspan="2" style="writing-mode:vertical-rl"| GuestPagingMode
|colspan="2" | ''res''
|colspan="20"| Interrupt Table Root Pointer[51:32]
|-
! +24
|colspan="16"| GDeviceID[15:0]
|colspan="1" style="writing-mode:vertical-rl"| vImuEn
|colspan="15"| ''reserved''
|-
! +28
|colspan="8" | SnoopAttribute
|colspan="1" style="writing-mode:vertical-rl"| Mode0FC
|colspan="1" style="writing-mode:vertical-rl"| AttrV
|colspan="6" | ''reserved''
|colspan="16"| GuestID[15:0]
|}

== I/O page tables ==

{| class="wikitable"
|+ Page Table Entry (PTE)
! 63
! 62
! 61
! 60
! 59
! 58:52
! 51:12
! 11:9
! 8:7
! 6
! 5
! 4:1
! 0
|-
| ''ign''
| IW
| IR
| FC
| U
| ''resv''
| physical address
| next level
| ''ign''
| D
| A
| ''ign''
| P
|}

{| class="wikitable"
|+ Page Directory Entry (PDE)
! 63
! 62
! 61
! 60:52
! 51:12
! 11:9
! 8:6
! 5
! 4:1
! 0
|-
| ''ign''
| IW
| IR
| ''resv''
| physical address
| next level
| ''ign''
| A
| ''ign''
| P
|}

== Commands ==

Each command entry is 128 bits in size.

{| class="wikitable"
|+ Command entry
|-
!colspan="1"  style="width:4em" | Offset
!colspan="1"  style="width:4em" | 63:60
!colspan="15" style="width:60em"| 59:0
|-
| +0
| Opcode
| First opcode dependent operand
|-
| +8
|colspan="2"| Second opcode dependent operand
|}

To send commands, the '''Command Buffer Base Register''' must be programmed. It must point to a ring buffer aligned to a 4KiB boundary.
The '''Command Buffer Head Pointer Register''' and '''Command Buffer Tail Pointer Register''' are automatically reset to 0 when the base register is written to.

{| class="wikitable"
|+ Command Buffer Base Register
!colspan="1"  style="width:4em" | 63:60
!colspan="1"  style="width:4em" | 59:56
!colspan="1"  style="width:4em" | 55:52
!colspan="10" style="width:40em"| 51:12
!colspan="3"  style="width:12em"| 11:00
|-
| /
| Length
| /
|colspan="10"| Base
| /
|-
|}

{| class="wikitable
|+ Length encoding
! Encoding
! Number of entries
! Byte size
|-
| 0-7
|colspan="2"| (reserved)
|-
| 8
| 256 entries
| 4 KiB
|-
| 9
| 512 entries
| 8 KiB
|-
|colspan="3" style="text-align:center"| ...
|-
| 15
| 32768 entries
| 512 KiB
|}

{| class="wikitable"
|+ Command Buffer Head Pointer Register
!colspan="11" style="width:44em"| 63:19
!colspan="4"  style="width:16em"| 18:4
!colspan="1"  style="width:4em" | 3:0
|-
|colspan="11"| /
|colspan="4" | Head pointer
|colspan="1" | /
|}

{| class="wikitable"
|+ Command Buffer Tail Pointer Register
!colspan="11" style="width:44em"| 63:19
!colspan="4"  style="width:16em"| 18:4
!colspan="1"  style="width:4em" | 3:0
|-
|colspan="11"| /
|colspan="4" | Tail pointer
|colspan="1" | /
|}

To notify the IOMMU, the Tail Pointer Register must be incremented, wrapping if necessary (modulo ring buffer size).

The IOMMU will increment the Head Pointer Register when a command has been fetched.
This does not mean the command has been processed! To ensure a command has been processed, use COMPLETION_WAIT.

Commands are only processed when '''CmdBufRun=1'''. Modifying the Base Register or the Head Pointer Register is only allowed when CmdBufRun=0.

=== 1. COMPLETION_WAIT ===

=== 2. INVALIDATE_DEVTAB_ENTRY ===

=== 3. INVALIDATE_IOMMU_PAGES ===

=== 4. INVALIDATE_IOTLB_PAGES ===

=== 5. INVALIDATE_INTERRUPT_TABLE ===

=== 6. PREFETCH_IOMMU_PAGES ===

=== 7. COMPLETE_PPR_REQUEST ===

=== 8. INVALIDATE_IOMMU_ALL ===

=== 9. INSERT_GUEST_EVENT ===

=== 10. RESET_VMMIO ===

== Events ==

=== 1. ILLEGAL_DEV_TABLE_ENTRY ===

=== 2. IO_PAGE_FAULT ===

=== 3. DEV_TAB_HARDWARE_ERROR ===

=== 4. PAGE_TAB_HARDWARE_ERROR ===

=== 5. ILLEGAL_COMMAND_ERROR ===

=== 6. COMMAND_HARDWARE_ERROR ===

=== 7. IOTLB_INV_TIMEOUT ===

=== 8. INVALID_DEVICE_REQUEST ===

=== 9. INVALID_PPR_REQUEST ===

=== 10. EVENT_COUNTER_ZERO ===

=== 11. GUEST_EVENT_FAULT ===

=== 12. VIOMMU_HARDWARE_ERROR ===

=== 13. RMP_PAGE_FAULT ===

=== 14. RMP_HARDWARE_ERROR ===

== Interrupt Remapping ==

=== MSI / MSI-X ===

To remap MSI the address need to be configured to point to <code>FD_F8xx_xxxx</code>.<ref>See Table 3 in specification.</ref>

== Registers ==

=== PCI configuration space ===

All accesses ''must'' be 32-bits and aligned to a 32-bit boundary.
Software should write zeros to reserved bits.

==== Base Capability (id=0xf) ====

{| class="wikitable"
|-
! Offset
! Name
|-
| 0x00
| Capability Header
|-
| 0x04
| Base Address Low Register
|-
| 0x08
| Base Address High Register
|-
| 0x0C
| Range Register
|-
| 0x10
| Miscellaneous Information Register 0
|-
| 0x14
| Miscellaneous Information Register 1
|-
|}

=== MMIO ===

All accesses ''should'' be 64-bits and properly aligned,
but smaller power-of-two accesses (32/16/8-bits) are also permitted.
Larger accesses (128/...-bits) are ''not'' permitted.

{| class="wikitable" style="table-layout:fixed"
! Offset
!style="width:50%"| Name (+4)
!style="width:50%"| Name (+0)
|-
!colspan="3"| 0x0000 to 0x1000
|-
| 0x0000
|colspan="2"| Device Table Base Address Register
|-
| 0x0008
|colspan="2"| Command Buffer Base Address Register
|-
| 0x0010
|colspan="2"| Event Log Base Address Register
|-
| 0x0018
|colspan="2"| IOMMU Control Register
|-
| 0x0020
|colspan="2"| IOMMU Exclusion Base Register / Completion Store Base Register
|-
| 0x0028
|colspan="2"| IOMMU Exclusion Range Limit Register / Completion Store Limit Register
|-
| 0x0030
|colspan="2"| IOMMU Extended Feature Register
|-
| 0x0038
|colspan="2"| PPR Log Base Address Register
|-
| 0x0040
|colspan="2"| IOMMU Hardware Event Upper Register
|-
| 0x0048
|colspan="2"| IOMMU Hardware Event Lower Register
|-
| 0x0050
|colspan="2"| Hardware Event Status Register
|-
| 0x0058
|colspan="2"| (unmapped)
|-
| 0x0060
|colspan="2"| SMI Filter Register 0
|-
| 0x0068
|colspan="2"| SMI Filter Register 1
|-
| 0x0070
|colspan="2"| SMI Filter Register 2
|-
| 0x0078
|colspan="2"| SMI Filter Register 3
|-
| 0x0080
|colspan="2"| SMI Filter Register 4
|-
| 0x0088
|colspan="2"| SMI Filter Register 5
|-
| 0x0090
|colspan="2"| SMI Filter Register 6
|-
| 0x0098
|colspan="2"| SMI Filter Register 7
|-
| 0x00A0
|colspan="2"| SMI Filter Register 8
|-
| 0x00A8
|colspan="2"| SMI Filter Register 9
|-
| 0x00B0
|colspan="2"| SMI Filter Register 10
|-
| 0x00B8
|colspan="2"| SMI Filter Register 11
|-
| 0x00C0
|colspan="2"| SMI Filter Register 12
|-
| 0x00C8
|colspan="2"| SMI Filter Register 13
|-
| 0x00D0
|colspan="2"| SMI Filter Register 14
|-
| 0x00D8
|colspan="2"| SMI Filter Register 15
|-
| 0x00E0
|colspan="2"| Guest Virtual APIC Log Base Address Register
|-
| 0x00E8
|colspan="2"| Guest Virtual APIC Log Tail Address Register
|-
| 0x00F0
|colspan="2"| PPR Log B Base Address Register
|-
| 0x00F8
|colspan="2"| Event Log B Base Address Register
|-
| 0x0100
|colspan="2"| Device Table Segment 1 Base Address Register
|-
| 0x0108
|colspan="2"| Device Table Segment 2 Base Address Register
|-
| 0x0110
|colspan="2"| Device Table Segment 3 Base Address Register
|-
| 0x0118
|colspan="2"| Device Table Segment 4 Base Address Register
|-
| 0x0120
|colspan="2"| Device Table Segment 5 Base Address Register
|-
| 0x0128
|colspan="2"| Device Table Segment 6 Base Address Register
|-
| 0x0130
|colspan="2"| Device Table Segment 7 Base Address Register
|-
| 0x0138
|colspan="2"| Device-Specific Feature Extension (DSFX) Register
|-
| 0x0140
|colspan="2"| Device-Specific Control Extension (DSCX) Register
|-
| 0x0148
|colspan="2"| Device-Specific Status Extension (DSSX) Register
|-
| 0x0150
| MSI Vector Register 1
| MSI Vector Register 0
|-
| 0x0158
| MSI Address Low Register
| MSI Capability Header Register
|-
| 0x0160
| MSI Data Register
| MSI Address High Register
|-
| 0x0168
| IOMMU Performance Optimization Control Register
| MSI Mapping Capability Header Register
|-
| 0x0170
|colspan="2"| XT IOMMU General Interrupt Control Register
|-
| 0x0178
|colspan="2"| XT IOMMU PPR Interrupt Control Register
|-
| 0x0180
|colspan="2"| XT IOMMU GA Log Interrupt Control Register
|-
| 0x0188
|colspan="2"| (reserved)
|-
| 0x0190
|colspan="2"| vIOMMU Status Register
|-
| 0x0198
|colspan="2"| (reserved)
|-
| 0x01A0
|colspan="2"| IOMMU Extended Feature 2 Register
|-
|colspan="3"| ...
|-
| 0x0200
|colspan="2"| MARC Aperture 0 Base Register
|-
| 0x0208
|colspan="2"| MARC Aperture 0 Relocation Register
|-
| 0x0210
|colspan="2"| MARC Aperture 0 Length Register
|-
| 0x0218
|colspan="2"| MARC Aperture 1 Base Register
|-
| 0x0220
|colspan="2"| MARC Aperture 1 Relocation Register
|-
| 0x0228
|colspan="2"| MARC Aperture 1 Length Register
|-
| 0x0230
|colspan="2"| MARC Aperture 2 Base Register
|-
| 0x0238
|colspan="2"| MARC Aperture 2 Relocation Register
|-
| 0x0240
|colspan="2"| MARC Aperture 2 Length Register
|-
| 0x0248
|colspan="2"| MARC Aperture 3 Base Register
|-
| 0x0250
|colspan="2"| MARC Aperture 3 Relocation Register
|-
| 0x0258
|colspan="2"| MARC Aperture 3 Length Register
|-
| 0x0260
|colspan="2"| (reserved)
|-
|colspan="3"| ...
|-
!colspan="3"| 0x1000 to 0x2000
|-
| 0x1000
|colspan="2"| (reserved)
|-
|colspan="3"| ...
|-
| 0x1FF8
|colspan="2"| IOMMU Reserved Register
|-
!colspan="3"| 0x2000 to 0x3000
|-
| 0x2000
|colspan="2"| Command Buffer Head Pointer Register
|-
| 0x2008
|colspan="2"| Command Buffer Tail Pointer Register
|-
| 0x2010
|colspan="2"| Event Log Head Pointer Register
|-
| 0x2018
|colspan="2"| Event Log Tail Pointer Register
|-
| 0x2020
|colspan="2"| IOMMU Status Register
|-
| 0x2028
|colspan="2"| (reserved)
|-
| 0x2030
|colspan="2"| IOMMU PPR Log Head Pointer Register
|-
| 0x2038
|colspan="2"| IOMMU PPR Log Tail Pointer Register
|-
| 0x2040
|colspan="2"| Guest Virtual APIC Log Head Pointer Register
|-
| 0x2048
|colspan="2"| Guest Virtual APIC Log Tail Pointer Register
|-
| 0x2050
|colspan="2"| PPR Log B Head Pointer Register
|-
| 0x2058
|colspan="2"| PPR Log B Tail Pointer Register
|-
| 0x2060
|colspan="2"| Event Log B Head Pointer Register
|-
| 0x2068
|colspan="2"| Event Log B Tail Pointer Register
|-
| 0x2070
|colspan="2"| Event Log B Head Pointer Register
|-
| 0x2078
|colspan="2"| Event Log B Tail Pointer Register
|-
| 0x2080
|colspan="2"| PPR Log Auto Response Register
|-
| 0x2088
|colspan="2"| PPR Log Overflow Early Indicator Register
|-
| 0x2090
|colspan="2"| PPR Log B Overflow Early Indicator Register
|-
| 0x2098
|colspan="2"| (reserved)
|}

== Troubleshooting ==

=== QEMU: no DMA translation ===

DMA translation needs to explicitly enabled:

<pre>
-device amd-iommu,dma-remap=on
</pre>

Interrupt remapping, IOTLB and other features also need to be enabled explicitly. Run with <code>help</code> to see all options:

<pre>
-device amd-iommu,help
</pre>

== See also ==

<references />

=== External Links ===

* [https://docs.amd.com/api/khub/documents/2oKZlc87KNCKiT3B~xJjHg/content AMD I/O Virtualization Technology (IOMMU) Specification (48882-PUB—Rev 3.11—Apr 2026)]

[[Category:X86 Virtualization]]
[[Category:X86-64]]
````

<a id="w004988"></a>

## W004988 — User:Demindiro/AMD-Vi IOMMU

Bron-ID: W004988
Namespace: User (2)
Revisie: 30037 | Datum: 2026-02-19T19:39:03Z
Bron: https://wiki.osdev.org/User:Demindiro/AMD-Vi_IOMMU
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADemindiro%2FAMD-Vi_IOMMU&oldid=30037
Doorverwijzing: AMD-Vi IOMMU

````text
#REDIRECT [[AMD-Vi IOMMU]]
````

<a id="w004989"></a>

## W004989 — User:Demindiro/AMD-Vi IOMMU article scratchpad

Bron-ID: W004989
Namespace: User (2)
Revisie: 30049 | Datum: 2026-02-20T19:57:43Z
Bron: https://wiki.osdev.org/User:Demindiro/AMD-Vi_IOMMU_article_scratchpad
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADemindiro%2FAMD-Vi_IOMMU_article_scratchpad&oldid=30049

````text
Tip for X11 users: <code>xclip</code> + middle mouse button :)

<syntaxhighlight lang="sh">
python3 amd-iommu-registers.py | xclip
</syntaxhighlight>

== Table script ==

=== <code>amd-iommu-registers.txt</code> ===

Format is <code>offset size name*</code>.

<syntaxhighlight>
0x0030 8 IOMMU Extended Feature Register
0x0038 8 PPR Log Base Address Register
0x0040 8 IOMMU Hardware Event Upper Register
0x0048 8 IOMMU Hardware Event Lower Register
0x0050 8 Hardware Event Status Register
0x0058 8 (unmapped)
0x0060 8 SMI Filter Register 0
0x0068 8 SMI Filter Register 1
0x0070 8 SMI Filter Register 2
0x0078 8 SMI Filter Register 3
0x0080 8 SMI Filter Register 4
0x0088 8 SMI Filter Register 5
0x0090 8 SMI Filter Register 6
0x0098 8 SMI Filter Register 7
0x00A0 8 SMI Filter Register 8
0x00A8 8 SMI Filter Register 9
0x00B0 8 SMI Filter Register 10
0x00B8 8 SMI Filter Register 11
0x00C0 8 SMI Filter Register 12
0x00C8 8 SMI Filter Register 13
0x00D0 8 SMI Filter Register 14
0x00D8 8 SMI Filter Register 15
0x00E0 8 Guest Virtual APIC Log Base Address Register
0x00E8 8 Guest Virtual APIC Log Tail Address Register
0x00F0 8 PPR Log B Base Address Register
0x00F8 8 Event Log B Base Address Register
0x0100 8 Device Table Segment 1 Base Address Register
0x0108 8 Device Table Segment 2 Base Address Register
0x0110 8 Device Table Segment 3 Base Address Register
0x0118 8 Device Table Segment 4 Base Address Register
0x0120 8 Device Table Segment 5 Base Address Register
0x0128 8 Device Table Segment 6 Base Address Register
0x0130 8 Device Table Segment 7 Base Address Register
0x0138 8 Device-Specific Feature Extension (DSFX) Register
0x0140 8 Device-Specific Control Extension (DSCX) Register
0x0148 8 Device-Specific Status Extension (DSSX) Register
0x0150 4 MSI Vector Register 0
0x0154 4 MSI Vector Register 1
0x0158 4 MSI Capability Header Register
0x015C 4 MSI Address Low Register
0x0160 4 MSI Address High Register
0x0164 4 MSI Data Register
0x0168 4 MSI Mapping Capability Header Register
0x016C 4 IOMMU Performance Optimization Control Register
0x0170 8 XT IOMMU General Interrupt Control Register
0x0178 8 XT IOMMU PPR Interrupt Control Register
0x0180 8 XT IOMMU GA Log Interrupt Control Register
0x0188 8 (reserved)
0x0190 8 vIOMMU Status Register
0x0198 8 (reserved)
0x01A0 8 IOMMU Extended Feature 2 Register
0x0200 8 MARC Aperture 0 Base Register
0x0208 8 MARC Aperture 0 Relocation Register
0x0210 8 MARC Aperture 0 Length Register
0x0218 8 MARC Aperture 1 Base Register
0x0220 8 MARC Aperture 1 Relocation Register
0x0228 8 MARC Aperture 1 Length Register
0x0230 8 MARC Aperture 2 Base Register
0x0238 8 MARC Aperture 2 Relocation Register
0x0240 8 MARC Aperture 2 Length Register
0x0248 8 MARC Aperture 3 Base Register
0x0250 8 MARC Aperture 3 Relocation Register
0x0258 8 MARC Aperture 3 Length Register
0x0260 8 (reserved)
0x1000 8 (reserved)
0x1FF8 8 IOMMU Reserved Register
0x2000 8 Command Buffer Head Pointer Register
0x2008 8 Command Buffer Tail Pointer Register
0x2010 8 Event Log Head Pointer Register
0x2018 8 Event Log Tail Pointer Register
0x2020 8 IOMMU Status Register
0x2028 8 (reserved)
0x2030 8 IOMMU PPR Log Head Pointer Register
0x2038 8 IOMMU PPR Log Tail Pointer Register
0x2040 8 Guest Virtual APIC Log Head Pointer Register
0x2048 8 Guest Virtual APIC Log Tail Pointer Register
0x2050 8 PPR Log B Head Pointer Register
0x2058 8 PPR Log B Tail Pointer Register
0x2060 8 Event Log B Head Pointer Register
0x2068 8 Event Log B Tail Pointer Register
0x2070 8 Event Log B Head Pointer Register
0x2078 8 Event Log B Tail Pointer Register
0x2080 8 PPR Log Auto Response Register
0x2088 8 PPR Log Overflow Early Indicator Register
0x2090 8 PPR Log B Overflow Early Indicator Register
0x2098 8 (reserved)
</syntaxhighlight>

=== <code>amd-iommu-registers.py</code> ===

<syntaxhighlight lang="python">
entries = []

with open('amd-iommu-registers.txt') as f:
    for l in f:
        l = l.strip()
        offset, size, name = map(str.strip, filter(None, l.split(' ', 2)))
        entries.append((int(offset, 0), int(size), name))

print('{| class="wikitable" style="table-layout:fixed"')
print('! Offset')
print('!style="width:50%"| Name (+4)')
print('!style="width:50%"| Name (+0)')
cur_page = -1
last_offset = 0
i = 0
while i < len(entries):
    offset, size, name = entries[i]
    if offset - last_offset > 8:
        print('|-')
        print('|colspan="3"| ...')
    if cur_page & -0x1000 != offset & -0x1000:
        cur_page = offset & -0x1000
        print('|-')
        print(f'!colspan="3"| 0x{cur_page:04X} to 0x{cur_page + 0x1000:04X}')
    print('|-')
    print(f'| 0x{offset:04X}')
    if size == 8:
        print('|colspan="2"|', name)
        i += 1
    elif size == 4:
        n_offset, n_size, n_name = entries[i + 1]
        assert offset + 4 == n_offset
        assert size == n_size
        print('|', n_name)
        print('|', name)
        i += 2
    else:
        assert 0, 'fix table pls'
    last_offset = offset
print('|}')
</syntaxhighlight>
````

<a id="w004990"></a>

## W004990 — File:AeroSync-r0c4-5.4.9-sample-qemu-boot.png.png

Bron-ID: W004990
Namespace: File (6)
Revisie: 30058 | Datum: 2026-02-22T01:54:29Z
Bron: https://wiki.osdev.org/File:AeroSync-r0c4-5.4.9-sample-qemu-boot.png.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAeroSync-r0c4-5.4.9-sample-qemu-boot.png.png&oldid=30058

````text
the name self-describes the description
````

<a id="w004991"></a>

## W004991 — Iwlwifi

Bron-ID: W004991
Namespace: Main (0)
Revisie: 30192 | Datum: 2026-03-11T02:53:27Z
Bron: https://wiki.osdev.org/Iwlwifi
Revisie-URL: https://wiki.osdev.org/index.php?title=Iwlwifi&oldid=30192

````text
{{Stub}}
{{DISPLAYTITLE:Intel Iwlwifi}}

'''Iwlwifi''' is the Linux kernel driver for wireless Intel Wi-Fi devices.

==Firmware download==
Before usage, a firmware download is required. For many new NIC cards, the firmware is available in [https://git.kernel.org/pub/scm/linux/kernel/git/firmware/linux-firmware.git/tree/ this repository]. [https://www.intel.com/content/www/us/en/support/articles/000005511/wireless.html]


==Writing to registers==
Registers are mapped in MMIO, writel() is used to write 32-bit and 64-bit values safely to the MMIO. In PCI configuration space, the BAR must be used to set up the device registers first. Writel definitions of the Linux source are available [https://billauer.co.il/blog/2014/08/wmb-rmb-mmiomb-effects/ here].


==NIC access==
NIC access is needed to read or write some registers safely.

Grabbing NIC access involves a few steps:

*Disabling [https://lwn.net/Articles/520076/ softirq]
*Test if the transport is alive
*Test if there is no command in flight
*Set the registers
*When release access is called, enable softirq.
<syntaxhighlight lang="c">
// Set bits of the mask
static inline void set_bits_mask(uint32_t reg, uint32_t mask)
{
	uint32_t v = read32(reg);
	v &= ~mask;
	v |= mask;
	write32(reg, v);
}


// Poll bits
int iwl_poll_bits_mask(u32 addr, u32 bits, u32 mask, int timeout)
{
	int t = 0;
	while (t < timeout) {
		if ((read32(trans, addr) & mask) == (bits & mask))
			return 0;
			
		udelay(POLL_INTERVAL);
		t += POLL_INTERVAL;
	}

	return -1;
}


/* Control and status registers */
#define CSR_BASE        (0x000)
#define CSR_GP_CONTROL  (CSR_BASE + 0x024)


/* Grab access */
int grab_nic_access() {
    if (!trans_alive || cmd_in_flight) return -1;
    
    if (family >= family_bz) {
        set_bits_mask(CSR_GP_CONTROL, BIT(21));
        
        if (family >= family_8000) udelay(2);
        
        iwl_poll_bits_mask(CSR_GP_CONTROL, BIT(20), BIT(20), 1500);
        if (ret) {
            // Handle error
            return -1;
        }
    } else {
        set_bits_mask(CSR_GP_CNTRL, 0x08);
        
        if (family >= family_8000) udelay(2);
        
        iwl_poll_bits_mask(CSR_GP_CONTROL, 0x01, 0x11, 1500);
        if (ret) {
            // Handle error
            return -1;
        }
    }

    return 0;
}
</syntaxhighlight><syntaxhighlight lang="c">
// Set bits of the mask
static inline void clear_bits_mask(uint32_t reg, uint32_t mask)
{
	uint32_t v = read32(reg);
	v &= ~mask;
	v |= 0;
	write32(reg, v);
}


#define CSR_BASE        (0x000)
#define CSR_GP_CONTROL  (CSR_BASE + 0x024)


/* Release access */
int release_nic_access() {
    if (cmd_in_flight) return -1;
    
    if (family >= family_bz) {
        clear_bits_mask(CSR_GP_CONTROL, BIT(21));
    } else {
        clear_bits_mask(CSR_GP_CONTROL, 0x08);
    }

    return 0;
}
</syntaxhighlight>

== Keep warm ==
Keep warm is a technique to keep the host DRAM powered on. It utilizes a dummy buffer, in this case a 4K buffer that needs to be passed to the device. The buffer must be 4K aligned and shifted four bits to the right.<syntaxhighlight lang="c">
// Flow handler
#define FH_BASE                         (0x1000)

/* Physical address */
#define KEEP_WARM_ADDR_REG		        (FH_BASE + 0x97C)
#define KEEP_WARM_SIZE                  0x1000


int init_kw() {
    void *kw = allocate_align(KEEP_WARM_SIZE, 0x1000);
    if (!kw) return -1;
    
    write32(KEEP_WARM_ADDR_REG, kw >> 4);
    return 0;
}
</syntaxhighlight>

==Queue==
Intel devices use [https://wiki.osdev.org/Intel_8254x#:~:text=writeH)%3B%0A%7D-,Ring%20setup,-%5Bedit%20%7C circular ring] buffers to send packets. There are many rings which have to be used: TX ring (transmit), RX ring (receive), command rings and a firmware download ring. The firmware download ring is usually only available at the firmware download stage. The command rings are used to change device settings and the other rings are used for networking. The DMA needs base registers to be set to the base address of the descriptor ring. Below is an example for the TX ring.<syntaxhighlight lang="c" line="1">
/*
 * Flow Handler memory block (0x1000 - 0x2000)
 */

// Base registers for TX gen1
#define FH_0_15_LOWER_BOUND         (FH_BASE + 0x9D0)
#define FH_0_15_UPPER_BOUND         (FH_BASE + 0xA10)
#define FH_16_19_LOWER_BOUND        (FH_BASE + 0xBF0)
#define FH_16_19_UPPER_BOUND        (FH_BASE + 0xC00)
#define FH_20_31_LOWER_BOUND        (FH_BASE + 0xB20)
#define FH_20_31_UPPER_BOUND        (FH_BASE + 0xB80)

// 22000 base registers
#define GEN2_TX_BASE_REGS			(0x1C00)

// Get base register
unsigned int get_base_register(int channel) {
    // Generation 2 chips
    if (gen2) {
        // For generation 2, channel should not exceed 64
        throw_error(chnl >= 64);
        return GEN2_TX_BASE_REGS + 8 * chnl;
    }

    if (chnl < 16)
        return FH_MEM_CBBC_0_15_LOWER_BOUND + 4 * chnl;

    if (chnl < 20)
    return FH_MEM_CBBC_16_19_LOWER_BOUND + 4 * (chnl - 16);
    
    // For generation 1, channel should not exceed 32
    throw_error(chnl >= 32);
    return FH_MEM_CBBC_20_31_LOWER_BOUND + 4 * (chnl - 20);
}


// make sure we have NIC control
void write_nic_access32(uint32_t reg, uint32_t val) {
    grab_access();
    write32(reg, val);
    release_access();
}

// make sure we have NIC control
void write_nic_access64(uint32_t reg, uint64_t val) {
    grab_access();
    write32(reg, low_32(val));
    write32(reg, high_32(val));
    release_access();
}


/* Set the base register */
int set_base_register(unsigned int addr, int channel) {
    // it requires a 256-byte aligned address
    if (gen2) {
        write_nic_access64(get_base_register(channel), addr);
    } else {
        write_nic_access32(get_base_register(channel), addr >> 8);
    }
    return 0;
}
</syntaxhighlight>

===TX layout===
For generation 1, every even low address field is unaligned on a 16-bit boundary, so it must be access with a special helper. Each TFD supports up to 20 / 25 buffers in DRAM, with a maximum size of 4K - 4 per buffer and a maximum size of 8K for the whole TFD. Each buffer needs to be a continuous block of memory, which may be scattered in DRAM. A ring has 256 TFDs with no more than 255 TFDs allowed on a queue.
====Generation 1:====
{| class="wikitable"
|+TFD buffer descriptor Gen1
!Size
!Offset
!Field
|-
|4
|0x00
|Low address
|-
|2
|0x04
|High 4 bits and length of buffer
|}
{| class="wikitable"
|+TFD Gen1
!Size
!Offset
!Field
|-
|3
|0x00
|Reserved
|-
|1
|0x03
|Number of TFD buffer descriptors [4:1 (res):3 (padding)]
|-
|6 * 20
|0x04
|TFD buffer descriptor gen1
|-
|4
|0x04 + 6n
|Padding
|-
| colspan="3" |128 bytes
|}


====Generation 2:====
{| class="wikitable"
|+TFD buffer descriptor Gen2
!Size
!Offset
!Field
|-
|2
|0x00
|Length
|-
|8
|0x02
|Address of the buffer
|}
{| class="wikitable"
|+TFD Gen2
!Size
!Offset
!Field
|-
|2
|0x00
|Number of transmit buffers
|-
|10 * 25
|0x02
|TFD buffer descriptor gen2
|-
|4
|0xFC
|Padding
|-
| colspan="3" |256 bytes
|}

=== TX uCode setup ===
The firmware called uCode needs a copy of the base address if the TX ring as it is unable to read the registers. The uCode operates in DRAM and uses commands for configuration. Below is an example for the initialization of the TX ring.<syntaxhighlight lang="c" line="1">
#define TFD_GEN1_NUM_TBS 20

struct tfd_tb_gen1 {
	le32 lo;
	le16 hi_len;
} PACKED;

struct tfd_gen1 {
	uint8_t reserved1[3];
	uint8_t num_tbs;
	struct tfd_tb_gen1 tbs[TFD_GEN1_NUM_TBS];
	le32 __pad;
} PACKED;

#define TFD_GEN2_NUM_TBS 25

struct tfd_tb_gen2 {
	le16 tb_len;
	le64 addr;
} PACKED;

struct tfd_gen2 {
	le16 num_tbs;
	struct tfd_tb_gen2 tbs[TFD_GEN2_NUM_TBS];
	le32 pad;
} PACKED;

void *allocate_tx_tfd(int size, int tid) {
	void *ring = NULL;
	if (gen2) {
		ring = allocate_align(size * sizeof(tfd_gen2), 256);
	} else {
		ring = allocate_align(size * sizeof(tfd_gen1), 256);
	}
	if (!ring) return ring;

	set_base_register(ring, tid);
	return ring;
}


/**
 * struct iwl_bc_tbl_entry - scheduler byte count table entry
 *	base physical address provided by SCD_DRAM_BASE_ADDR
 * For devices up to 22000:
 * @tfd_offset:
 *	For devices up to 22000:
 *		 0-12 - tx command byte count
 *		12-16 - station index
 *	For 22000 and on:
 *		 0-12 - tx command byte count
 *		12-13 - number of 64 byte chunks
 *		14-16 - reserved
 */
struct sched_bc_tbl_entry {
	__le16 tfd_offset;
} __packed;



/* Read with read32 */
#define CSR_HW_REV              (CSR_BASE + 0x028)



struct cmd_header {
	uint8_t cmd_id;
	uint8_t group_id;
	/**
	 * @sequence:
	 * Sequence number for the command.
	 *
	 * The driver sets up the sequence number to values of its choosing.
	 * uCode does not use this value, but passes it back to the driver
	 * when sending the response to each driver-originated command, so
	 * the driver can match the response to the command.  Since the values
	 * don't get used by uCode, the driver may set up an arbitrary format.
	 *
	 * There is one exception:  uCode sets bit 15 when it originates
	 * the response/notification, i.e. when the response/notification
	 * is not a direct response to a command sent by the driver.  For
	 * example, uCode issues REPLY_RX when it sends a received frame
	 * to the driver; it is not a direct response to any driver command.
	 */
	le16 sequence;
} PACKED;


struct rx_packet {
	/*
	 * The first 4 bytes of the RX frame header contain both the RX frame
	 * size and some flags.
	 * Bit fields:
	 * 31:    flag flush RB request
	 * 30:    flag ignore TC (terminal counter) request
	 * 29:    flag fast IRQ request
	 * 28-27: Reserved
	 * 26:    RADA enabled
	 * 25:    Offload enabled
	 * 24:    RPF enabled
	 * 23:    RSS enabled
	 * 22:    Checksum enabled
	 * 21-16: RX queue
	 * 15-14: Reserved
	 * 13-00: RX frame size
	 */
	le32 len_n_flags;
	struct cmd_header hdr;
	uint8_t data[];
} PACKED;


// Up to 20
#define MAX_CMD_TBS_PER_TFD	20

struct iwl_host_cmd {
	const void *data[MAX_CMD_TBS_PER_TFD];
	struct rx_packet *resp_pkt;
	unsigned long _rx_page_addr;
	uint32_t _rx_page_order;

	uint32_t flags;
	uint32_t id;
	uint16_t len[MAX_CMD_TBS_PER_TFD];
	uint8_t dataflags[MAX_CMD_TBS_PER_TFD];
};


struct tx_queue_cfg_cmd {
	uint8_t sta_id;
	uint8_t tid;
	le16 flags;
	le32 cb_size;
	le64 byte_cnt_addr;
	le64 tfdq_addr;
} PACKED; /* TX_QUEUE_CFG_CMD_API_S_VER_2 */


struct tx_queue_cfg_rsp {
	le16 queue_number;
	le16 flags;
	le16 write_pointer;
	le16 reserved;
} PACKED; /* TX_QUEUE_CFG_RSP_API_S_VER_2 */


struct schedular_cfg_cmd {
	le32 operation;
	union {
		struct {
			le32 sta_mask;
			uint8_t tid;
			uint8_t reserved[3];
			le32 flags;
			le32 cb_size;
			le64 bc_dram_addr;
			le64 tfdq_dram_addr;
		} PACKED add; /* TX_QUEUE_CFG_CMD_ADD_API_S_VER_1 */
		struct {
			le32 sta_mask;
			le32 tid;
		} PACKED remove; /* TX_QUEUE_CFG_CMD_REMOVE_API_S_VER_1 */
		struct {
			le32 old_sta_mask;
			le32 tid;
			le32 new_sta_mask;
		} PACKED modify; /* TX_QUEUE_CFG_CMD_MODIFY_API_S_VER_1 */
	} PACKED u; /* TX_QUEUE_CFG_CMD_OPERATION_API_U_VER_1 */
} PACKED; /* TX_QUEUE_CFG_CMD_API_S_VER_3 */




static inline uint32_t rx_packet_payload_len(const struct rx_packet *pkt)
{
	return (le32_to_cpu(pkt->len_n_flags) & FH_RSCSR_FRAME_SIZE_MSK) - sizeof(pkt->hdr);
}


static int txq_alloc_response(struct iwl_host_cmd *hcmd)
{
	struct iwl_tx_queue_cfg_rsp *rsp;

	if (rx_packet_payload_len(hcmd->resp_pkt) != sizeof(*rsp)) {
		goto error;
	}

	rsp = (void *)hcmd->resp_pkt->data;
	int qid = le16_to_cpu(rsp->queue_number); // Store it
	uint32_t wr_ptr = le16_to_cpu(rsp->write_pointer);

	// If qid is unsupported
	if (qid >= ARRAY_SIZE(num_txq)) {
		goto error;
	}

	wr_ptr &= (max_tfd - 1);

	/* Place first TFD at index corresponding to start sequence number */
	txq->read_ptr = wr_ptr;
	txq->write_ptr = wr_ptr;

	free_pages(cmd->_rx_page_addr, cmd->_rx_page_order);
	return qid;

error_free_resp:
	free_pages(cmd->_rx_page_addr, cmd->_rx_page_order);
	return -1;
}

enum CMD_MODE {
	CMD_ASYNC		    = BIT(0),
	CMD_WANT_SKB		= BIT(1),
	CMD_SEND_IN_RFKILL	= BIT(2),
	CMD_BLOCK_TXQS		= BIT(3),
};

enum tx_queue_cfg_actions {
	TX_QUEUE_CFG_ENABLE_QUEUE		= BIT(0),
	TX_QUEUE_CFG_TFD_SHORT_FORMAT		= BIT(1),
};

enum scd_queue_cfg_operation {
	IWL_SCD_QUEUE_ADD = 0,
	IWL_SCD_QUEUE_REMOVE = 1,
	IWL_SCD_QUEUE_MODIFY = 2,
};

enum mvm_command_groups {
	LEGACY_GROUP = 0x0,
	LONG_GROUP = 0x1,
	SYSTEM_GROUP = 0x2,
	MAC_CONF_GROUP = 0x3,
	PHY_OPS_GROUP = 0x4,
	DATA_PATH_GROUP = 0x5,
	SCAN_GROUP = 0x6,
	NAN_GROUP = 0x7,
	LOCATION_GROUP = 0x8,
	BT_COEX_GROUP = 0x9,
	PROT_OFFLOAD_GROUP = 0xb,
	REGULATORY_AND_NVM_GROUP = 0xc,
	DEBUG_GROUP = 0xf,
	STATISTICS_GROUP = 0x10,
};

#define SCD_QUEUE_CONFIG_CMD 0x17

#define TFD_QUEUE_CB_SIZE(x) (ilog2(x) - 3)
#define WIDE_ID(grp, opcode) (((grp) << 8) | (opcode))


/* Allocate tx queue */
int allocate_tx_queue(uint32_t flags, uint32_t sta_mask, uint8_t tid, int size)
{
	union {
		struct iwl_tx_queue_cfg_cmd old;
		struct schedular_cfg_cmd new;
	} cmd;

	struct iwl_host_cmd hcmd = {
		.flags = CMD_WANT_SKB,
	};

	void *schedular = allocate(sizeof(sched_bc_tbl_entry));
	if (!schedular) return -1;

	// Allocate ring
	void *txq = NULL;
	if (family == family_bz && hw_revision == 0x00) {
		size = 4096;
		txq = allocate_tx_tfd(size, tid);
		if (!txq) { free(schedular); return -1; }
	} else {
		do {
			txq = allocate_ring(size, tid);
			if (txq) break;
			size /= 2; // Try smaller sizes
		} while (size >= 16);
		if (!txq) { free(schedular); return -1; }
	}

	// queue_alloc_cmd_ver can be found in the patch header
	if (queue_alloc_cmd_ver == 0) {
		memset(&cmd.old, 0, sizeof(cmd.old));
		cmd.old.tfdq_addr = cpu_to_le64(txq);
		cmd.old.byte_cnt_addr = cpu_to_le64(schedular);
		cmd.old.cb_size = cpu_to_le32(TFD_QUEUE_CB_SIZE(size));
		cmd.old.flags = cpu_to_le16(flags | TX_QUEUE_CFG_ENABLE_QUEUE);
		cmd.old.tid = tid;

		if (__builtin_popcount(sta_mask) != 1) {
			goto error;
		}
		cmd.old.sta_id = __builtin_ffs(sta_mask) - 1;

		hcmd.id = SCD_QUEUE_CFG;
		hcmd.len[0] = sizeof(cmd.old);
		hcmd.data[0] = &cmd.old;
	} else if (queue_alloc_cmd_ver == 3) {
		memset(&cmd.new, 0, sizeof(cmd.new));
		cmd.new.operation = cpu_to_le32(IWL_SCD_QUEUE_ADD);
		cmd.new.u.add.tfdq_dram_addr = cpu_to_le64(txq);
		cmd.new.u.add.bc_dram_addr = cpu_to_le64(schedular);
		cmd.new.u.add.cb_size = cpu_to_le32(TFD_QUEUE_CB_SIZE(size));
		cmd.new.u.add.flags = cpu_to_le32(flags);
		cmd.new.u.add.sta_mask = cpu_to_le32(sta_mask);
		cmd.new.u.add.tid = tid;

		hcmd.id = WIDE_ID(DATA_PATH_GROUP, SCD_QUEUE_CONFIG_CMD);
		hcmd.len[0] = sizeof(cmd.new);
		hcmd.data[0] = &cmd.new;
	} else {
		goto error;
	}

	int ret = iwl_trans_send_cmd(trans, &hcmd);
	if (ret) goto error;

	ret = txq_alloc_response(trans, txq, &hcmd);
	if (ret < 0) goto error;

	return ret;

error:
	free(txq);
	free(schedular);
	return -1;
}
</syntaxhighlight>

== See Also ==
=== Articles ===
=== External Links ===
* https://elixir.bootlin.com/linux/v6.18.6/source/drivers/net/wireless/intel/iwlwifi
* https://git.kernel.org/pub/scm/linux/kernel/git/firmware/linux-firmware.git/tree/
* https://billauer.co.il/blog/2014/08/wmb-rmb-mmiomb-effects/
* https://www.intel.com/content/www/us/en/support/articles/000005511/wireless.html

{{DEFAULTSORT:Intel Iwlwifi}}
[[Category: Network Hardware]]
[[Category: Networking]]
````

<a id="w004992"></a>

## W004992 — File:AeroSync-r0c4-5.4.9-sample-qemu-boot.png

Bron-ID: W004992
Namespace: File (6)
Revisie: 30060 | Datum: 2026-02-22T11:48:47Z
Bron: https://wiki.osdev.org/File:AeroSync-r0c4-5.4.9-sample-qemu-boot.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAeroSync-r0c4-5.4.9-sample-qemu-boot.png&oldid=30060

````text

````

<a id="w004993"></a>

## W004993 — Talk:AMD-Vi IOMMU

Bron-ID: W004993
Namespace: Talk (1)
Revisie: 30069 | Datum: 2026-02-24T18:14:36Z
Bron: https://wiki.osdev.org/Talk:AMD-Vi_IOMMU
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AAMD-Vi_IOMMU&oldid=30069

````text
== QEMU AMD-Vi IOMMU confusion ==

There are several aspects about the IOMMU that I'm uncertain about, so I won't put them in the article while I query qemu-devel. In the meantime, I will document them here:

=== INVALIDATE_IOMMU_ALL doesn't reload DTEs ===

The specification says (emphasis mine):

<blockquote>
The INVALIDATE_IOMMU_ALL command instructs the IOMMU to invalidate all cached information for interrupt remapping and address translation for guest and nested translations, '''including cached portions of the Device Table''', the guest CR3 table, page directory entries, page table entries, and interrupt remapping entries.
</blockquote>

But QEMU doesn't reload any DTEs. It only reloads if INVALIDATE_DEVTAB_ENTRY is issued. I strongly suspect this is an oversight in QEMU, but I will have to confirm.

=== <code>fetch_pte()</code> has incorrect page size for large pages (+ ignores level skipping) ===

As of writing (and excluding the <code>printf</code>s added by me) <code>fetch_pte()</code> uses this loop for walking the page tables:

<syntaxhighlight lang="c">
    printf("PT root (level=%d):  pte=%016"PRIx64"\n", level, *pte);
    do {
        level -= 1;

        printf("PT walk (level=%d):  pte=%016"PRIx64" NextLevel=%llu page_size=0x%"PRIx64"\n", level, *pte, PTE_NEXT_LEVEL(*pte), *page_size);

        /* Update the page_size */
        *page_size = PTE_LEVEL_PAGE_SIZE(level);  // <--- (1)

        /* Permission bits are ANDed at every level, including the DTE */
        perms &= amdvi_get_perms(*pte);
        if (perms == IOMMU_NONE) {
            return 0;
        }

        /* Not Present */
        if (!IOMMU_PTE_PRESENT(*pte)) {
            return 0;
        }

        /* Large or Leaf PTE found */
        if (PTE_NEXT_LEVEL(*pte) == 7 || PTE_NEXT_LEVEL(*pte) == 0) {  // <--- (2)
            /* Leaf PTE found */
            break;
        }

        /*
         * Index the pgtable using the IOVA bits corresponding to current level
         * and walk down to the lower level.
         */
        pte_addr = NEXT_PTE_ADDR(*pte, level, address);
        *pte = amdvi_get_pte_entry(as->iommu_state, pte_addr, as->devfn);

        if (*pte == (uint64_t)-1) {
            /*
             * A returned PTE of -1 indicates a failure to read the page table
             * entry from guest memory.
             */
            if (level == mode - 1) {
                /* Failure to retrieve the Page Table from Root Pointer */
                *page_size = 0;
                return -AMDVI_FR_PT_ROOT_INV;
            } else {
                /* Failure to read PTE. Page walk skips a page_size chunk */
                return -AMDVI_FR_PT_ENTRY_INV;
            }
        }
    } while (level > 0);
</syntaxhighlight>

The first issue is: the page size is too small for large pages (2M becomes 4K, 1G becomes 2M ...). Simply putting (1) below (2) fixes this, but this may not correspond to how real HW works.

The second issue is: this code appears to not implement level skipping.
````

<a id="w004994"></a>

## W004994 — Intel 82599

Bron-ID: W004994
Namespace: Main (0)
Revisie: 30567 | Datum: 2026-05-08T10:32:15Z
Bron: https://wiki.osdev.org/Intel_82599
Revisie-URL: https://wiki.osdev.org/index.php?title=Intel_82599&oldid=30567

````text
[[Image:Intel_X520_SR2.jpg|Intel 82599-based X520 card, with two transceivers installed|thumb|300x300px]]
{{In Progress}}

The Intel 82599 Ethernet controller is a 10 gigabit ethernet controller chipset, notably used on the X520 family of NICs. These NICs are end of life, but for this reason can be acquired on a very tight budget, in comparison to newer models like the X540 or X550. Unlike similar budgets cards the author investigated, documentation is available.

==Hardware==
Unlike most gigabit products you might encounter, 10 GbE products often have a choice of PHY. This is possible on gigabit using SFP, but far more 10 GbE cards have SFP+ ports. These take a PHY transceiver, rendering the card medium independent.

RJ45 connectors for these are often known to run hot. Other options for PHY interface are Fibre Optics, for which a plethora of options exist (single mode, multimode, wavelength, range...). A common modern choice might be 10GBASE-LR over OS2 fibre, which has a range of up to 10 km. Finally, the simplest is DAC, Direct Attach Copper, which works well for short runs, in theory up to 15 metres. This is especially suited to in-rack connections.

The 82599 controller is a PCIe 2.0 x8 device, so has a maximum throughput of 4 GB/s. This isn't actually quite enough to saturate the 2 PHYs it supports, full duplex, aggregating 40 Gbps.

==Virtualization==
This chipset is designed to work with SR-IOV. See also [[IOMMU]].

This means that there is one Physical Function (PF), and a number of Virtual Functions (VFs). Largely, virtual functions mirror the PF, with some differences.

==Operation Modes==
The controller has an double-VLAN capability where it expects at least one (outer) VLAN identifier on received packets.

== Operation ==

=== Initialization ===
The 82599 is initialised with the following sequence:

# Disable Interrupts - this can be achieved through PCI by turning off enable on MSI and MSI-X, and also the legacy pin interrupt enable.
# Issue a global reset by writing RST in the [[Intel 82599#CTRL|CTRL]] register of the PF.
# Wait at least 10 ms
# [Optional] Enable flow control
# Wait for EEPROM auto-read to complete
# Wait for DMA initialization, RDRXCTL.DMAIDONE.
# Set Up the PHY and Link
#* This step depends on the PHY used.
# Enable counters
# Initialize Receive
# Initialize Transmit
# Enable interrupts

=== PHY Initialization ===
As mentioned before, the X520 uses SFP+ pluggable modules. However, SFP+ itself uses an interface call XGMII, but is backwards compatible with Gigabit SFP using SGMII, and 100 MBit SFP using MII.

There are then further differences based on the physical medium, but with so much commonailty too, the result is two initialization modes:

==== SGMII ====
This supports BX PHYs, which is a fibre standard for 100 MBit ethernet, and SGMII, '''''Serial Gigabit Media Independent Interface'''''.

So, this is the 1/0.1 gigabit mode. 

# Electrical link setup. The spec indicates that this is handled by the EEPROM.
# Configure AUTOC.LMS to select the link mode.
# Configure SERDESC register as necessary.
# Verify correct status (link up and speed) in LINKS.

==== XAUI ====
XAUI, or '''''10 Gigabit Attatchment Unit Interface''''', is an extension of XGMII. This mode also supports BX4, CX4, KX, KX4, KR, and SFI backplane standards.

* Electrical link setup. The spec indicates that this is handled by the EEPROM.
* Configure AUTOC.LMS to select the link mode, set 10G_PMA_PMD_PARALLEL and AUTOC2.10G_PMA_PMD_Serial to the appropriate settings.
* Configure SERDESC register as necessary.
** If using KX/KX4/KR mode with autonegotiation, configure the KX_Support field of AUTOC.
* Restart the link by restarting autonegotiation using AUTOC.
* Verify correct status (link up and speed) in LINKS.

=== Statistics ===
Statistics registers (bytes/packets Rx/Tx through specified queues) can be initialized here.

=== Interrupt Initialization ===
Initializing the interrupts is similar between legacy/MSI and MSI-X, with MSI-X being the most modern system.

* Interrupt causes are mapped to vectors by programming the IVAR[n] registers.
* IVAR_MISC supports a couple of additional sources in MSI-X mode.
* Program SRRCTL[n].RDMTS (Split Receive control) for the minimum threshold interrupt.
* Set the EIAC[n] register for interrupt auto-clear Tx and Rx (MSI-X only). This is a performance optimization.
* Set the auto-mask in EIAM and EIAM[n].
* Set interrupt throttling in EITR[n] and GPIE.
* Enable interrupts through EIMS[n]

=== Receive Initialization ===
The 82599 supports 128 Rx and Tx queues.

==Registers==
The 82599s registers are accessed through MMIO at [[PCI]] BAR0. An alternative I/O space access mechanism optionally exists, through an ADDR/DATA pair at BAR2.

===CTRL ===
'''MMIO+0x0000''', mirrored at +0x0004 for compatibility.
{| class="wikitable"
|+Register
!31:27
!26
!25:4
!3
!2
!1:0
|-
| RSVD
|RST
|RSVD
|LRST
|PEMD
|RSVD
|}
{| class="wikitable"
|+Detail
!Field
!Description
|-
|RST
|Device Reset
|-
|LRST
|Link Reset
|-
|PEMD
|PCIe Master Disable
|}

===STATUS===
'''MMIO+0x0008'''
{| class="wikitable"
|+Register
!31:20
!19
!18
!17:10
!9:8
!7
!6:4
!3:2
!1:0
|-
|RSVD
|PMS
|IOVA 
|nVFs
|RSVD
|LU
|RSVD
|LID
|RSVD
|}
{| class="wikitable"
|+Detail
!Field
!Description
|-
|PMS 
|PCIe Master Enable Status
|-
|IOVA
|IOV Active, reflects VFE in IOV capability
|-
|nVFs
| Number of Virtual Functions
|-
|LU
|Link Up
|-
|LID
|LAN ID
|}

===CTRL_EXT===
'''MMIO+0x0018'''
{| class="wikitable"
|+Register
!31:29
!28
!27
!26
!25:18
!17
!16
!15
!14
!13:0
|-
|RSVD
|DRV_LOAD
|RSVD
|xVLAN
|RSVD
|RO_DIS
|NS_DIS
|RSVD
|PFRSTD
|RSVD
|}
{| class="wikitable"
|+Detail
!Field
!Description
|-
|DRV_LOAD
|Driver Loaded
|-
|xVLAN
|Extended VLAN - expects tagged packets 
|-
| RO_DIS
|Relaxed Ordering DIsable
|-
| NS_DIS
|No Snoop Disable
|-
|PFRSTD
|PF Reset Done
|}

===EXVET===
'''MMIO+0x5078'''

Extended VLAN Ether Type.
{| class="wikitable"
|+Register
!31:16
!15:0
|-
|VET EXT 
|RSVD
|}
VET EXT - Outer-VLAN EtherType. This is the VLAN tag protocol identifier. Default value is '''0x8100''' for 802.1Q.

=== AUTOC ===
'''MMIO+0x42A0'''

Autonegotiation control register
{| class="wikitable"
|+AUTOC
!16
!15-13
!12
!11
!10
!9
!8-7
!6-2
!1
!0
|-
|KRS
|LMS
|RAN
|RATD
|D10GMP
|1GPP
|10GPPP
|ANSF
|ANACK2
|FLU
|}

==References==


The full manual can be found here - [https://www.intel.com/content/www/us/en/content-details/331520/intel-82599-10-gigabit-ethernet-controller-datasheet.html Intel® 82599 10 Gigabit Ethernet Controller: Datasheet]

[[Category:Network Hardware]]
````

<a id="w004995"></a>

## W004995 — Intel NICs

Bron-ID: W004995
Namespace: Main (0)
Revisie: 30075 | Datum: 2026-02-26T09:05:44Z
Bron: https://wiki.osdev.org/Intel_NICs
Revisie-URL: https://wiki.osdev.org/index.php?title=Intel_NICs&oldid=30075

````text
{{In Progress}}For convenience, this is a guide to the various different Intel NICs that are available.

==ISA==
{| class="wikitable"
|+
!Device
!Chipset
!Codename
!Speed
!Year
|-
|Intel EtherExpress PRO/10
|82595
|
|10M
|1995
|}

==PCI==
{| class="wikitable"
|+
!Device
!Chipset
!Codename
!Speed
!Year
|-
|Intel EtherExpress PRO/100+
|82558
|
|100M
|1997
|-
|Intel EtherExpress PRO/100+
|82559
|
|100M
|1999
|-
|Intel PRO/1000 S
|[[Intel 8254x|82540]]
|Gainesville
|1G
|2002
|}

==PCI-X==
{| class="wikitable"
|+
!Device
!Chipset
!Codename
!Speed
!Year
|-
|Intel PRO/1000 MF
|[[Intel 8254x|82546]]
|Eldridge
|1G
|2003
|-
|Intel® PRO/10GbE CX4
|82597
|Cibolo
|10G
|2004
|}

==PCIe==
{| class="wikitable"
|+
!Device
!Chipset
!Codename
!Year
|-
| colspan="4" |'''Gigabit'''
|-
|PRO/1000 PT
|82572
|Dewey Beach
|2006
|-
|i350
|i350
|Powerville
|2010
|-
|i210
|i210
|
|2012
|-
|i217
|[[Intel Ethernet i217|i217]]
|
|2013
|-
|i219
|i219
|
|2015
|-
| colspan="4" |'''2.5 GbE'''
|-
|i225
|i225
|
|2020
|-
| colspan="4" |'''10 GbE'''
|-
|EXPX9502CX4
|82598EB
|Cedar Cove
|2008
|-
|X520
|[[Intel 82599|82599]]
|Spring Fountain
|2009
|-
|X540
|X540
|Twinville
|2012
|-
|X550
|X550
|
|2016
|-
|X710
|X710
|
|2014
|-
| colspan="4" |'''40 GbE'''
|-
|X520-QDA1
|[[Intel 82599|82599]]
|
|2014
|-
|XL710
|X710
|
|2014
|-
| colspan="4" |'''25 GbE'''
|-
|XXV710
|X710
|
|2017
|-
| colspan="4" |'''100 GbE'''
|-
|E810
|E810
|
|2020
|-
| colspan="4" |'''200 GbE'''
|-
|E830-CQDA2
|E830
|Emerald Rapids
|2025
|}

[[Category:Network Hardware]]
````

<a id="w004996"></a>

## W004996 — File:Intel X520 SR2.jpg

Bron-ID: W004996
Namespace: File (6)
Revisie: 30081 | Datum: 2026-02-28T13:08:19Z
Bron: https://wiki.osdev.org/File:Intel_X520_SR2.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AIntel_X520_SR2.jpg&oldid=30081

````text
== Summary ==
An Intel X520 SR2 Network Interface Card
````

<a id="w004997"></a>

## W004997 — PnP

Bron-ID: W004997
Namespace: Main (0)
Revisie: 30092 | Datum: 2026-03-01T16:45:35Z
Bron: https://wiki.osdev.org/PnP
Revisie-URL: https://wiki.osdev.org/index.php?title=PnP&oldid=30092
Doorverwijzing: Plug-and-Play

````text
#REDIRECT [[Plug-and-Play]]
````

<a id="w004998"></a>

## W004998 — File:BoredOSscreenshot.jpg

Bron-ID: W004998
Namespace: File (6)
Revisie: 30100 | Datum: 2026-03-02T21:52:57Z
Bron: https://wiki.osdev.org/File:BoredOSscreenshot.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABoredOSscreenshot.jpg&oldid=30100

````text
A screenshot of the BoredOS desktop.
````

<a id="w004999"></a>

## W004999 — Category:Virtual

Bron-ID: W004999
Namespace: Category (14)
Revisie: 30105 | Datum: 2026-03-03T15:03:21Z
Bron: https://wiki.osdev.org/Category:Virtual
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AVirtual&oldid=30105
Doorverwijzing: Category:Virtualization

````text
#REDIRECT [[:Category:Virtualization]]
````

<a id="w005000"></a>

## W005000 — Category:X86 Virtualization

Bron-ID: W005000
Namespace: Category (14)
Revisie: 30110 | Datum: 2026-03-03T15:08:57Z
Bron: https://wiki.osdev.org/Category:X86_Virtualization
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AX86_Virtualization&oldid=30110

````text
[[Category:Virtualization]]
[[Category:X86]]
````

<a id="w005001"></a>

## W005001 — IOMMU

Bron-ID: W005001
Namespace: Main (0)
Revisie: 30116 | Datum: 2026-03-03T15:35:16Z
Bron: https://wiki.osdev.org/IOMMU
Revisie-URL: https://wiki.osdev.org/index.php?title=IOMMU&oldid=30116

````text
{{Stub}}

An '''IOMMU''' (Input Output Memory Management Unit) implements a form of Virtual Memory through [[Paging]], for I/O devices. This means that their [[DMA]] transactions can be access-controlled, and remapped, instead of directly accessing physical memory.

== Uses ==
IOMMUs have more than one use. The most obvious deployment is using a [[Hypervisor]]. Guest OSes in virtual machines can then access virtualized hardware that is passed-through to the guest VM, but the IOMMU will remap Guest Physical Addresses to the real memory backing it, and prevent DMA accesses beyond the bounds of the VM.

However, the same principles may also be applied to application software running in an unprivileged context, too. Some IOMMUs use the same paging format at the CPU, and nested translation too. This means they can translate Virtual Addresses, to Guest Physical Address, to Physical Addresses. Using this, it is possible to allow Ring 3 software to communicate directly with the hardware, and program DMA, without breaking security. Some modern graphics cards support this for performance reasons, for instance.

== Implementations ==

=== X86 ===

* [[AMD-Vi IOMMU]]
* [[Intel VT-d]]

=== ARM ===

* [[ARM SMMU versions 1 and 2]]

[[Category:Virtualization]]
````

<a id="w005002"></a>

## W005002 — Category:ARM Virtualization

Bron-ID: W005002
Namespace: Category (14)
Revisie: 30114 | Datum: 2026-03-03T15:12:24Z
Bron: https://wiki.osdev.org/Category:ARM_Virtualization
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AARM_Virtualization&oldid=30114

````text
[[Category:Virtualization]]
[[Category:ARM]]
````

<a id="w005003"></a>

## W005003 — LBA

Bron-ID: W005003
Namespace: Main (0)
Revisie: 30141 | Datum: 2026-03-05T16:54:34Z
Bron: https://wiki.osdev.org/LBA
Revisie-URL: https://wiki.osdev.org/index.php?title=LBA&oldid=30141
Doorverwijzing: Logical Block Addressing

````text
#REDIRECT [[Logical Block Addressing]]
````

<a id="w005004"></a>

## W005004 — File:Cylinder Head Sector.svg.png

Bron-ID: W005004
Namespace: File (6)
Revisie: 30142 | Datum: 2026-03-05T23:09:48Z
Bron: https://wiki.osdev.org/File:Cylinder_Head_Sector.svg.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ACylinder_Head_Sector.svg.png&oldid=30142

````text
== Summary ==
A diagram of cylinders/tracks and sectors on a platter. Also, a diagram of four (4) stacked platters, and eight (8) read/write heads.

From Wikimedia Commons, released to the public domain.
````

<a id="w005005"></a>

## W005005 — Category:DEC Alpha

Bron-ID: W005005
Namespace: Category (14)
Revisie: 30148 | Datum: 2026-03-06T01:55:31Z
Bron: https://wiki.osdev.org/Category:DEC_Alpha
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ADEC_Alpha&oldid=30148

````text
'''DEC Alpha''' or just '''Alpha''' is a 64-bit RISC CPU architecture developed by Digital Equipment Corporation (DEC).

[[Category:Platforms]]
````

<a id="w005006"></a>

## W005006 — SVM

Bron-ID: W005006
Namespace: Main (0)
Revisie: 30150 | Datum: 2026-03-06T10:57:12Z
Bron: https://wiki.osdev.org/SVM
Revisie-URL: https://wiki.osdev.org/index.php?title=SVM&oldid=30150

````text
{{Stub}}

'''AMD's Secure Virtual Machine''' is AMD's counterpart to Intel's [[VMX]] instructions.

== Programming == 

=== Discovering support ===
After enumerating the features from [[CPUID]], page 0x80000001, bit 2 in ECX indicates support for SVM.

Next, the VM_CR MSR (0xC00010114) needs to be consulted. Bit 4, SVMDIS, must be zero. If set, the CPU supports SVM, but it has been disabled by the platform firmware.


=== Basic environment ===
Software should enable bit 12 of EFER (EFER.SVME) first.

==References==

[[Category:X86 Virtualization]]
````

<a id="w005007"></a>

## W005007 — File:Asuro.png

Bron-ID: W005007
Namespace: File (6)
Revisie: 30156 | Datum: 2026-03-08T22:17:58Z
Bron: https://wiki.osdev.org/File:Asuro.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAsuro.png&oldid=30156

````text
== Summary ==
Asuro operating system preview.
````

<a id="w005008"></a>

## W005008 — Serial ATA

Bron-ID: W005008
Namespace: Main (0)
Revisie: 30170 | Datum: 2026-03-09T16:32:14Z
Bron: https://wiki.osdev.org/Serial_ATA
Revisie-URL: https://wiki.osdev.org/index.php?title=Serial_ATA&oldid=30170
Doorverwijzing: SATA

````text
#REDIRECT [[SATA]]
````

<a id="w005009"></a>

## W005009 — Category:Compatability Layers

Bron-ID: W005009
Namespace: Category (14)
Revisie: 30175 | Datum: 2026-03-09T16:44:16Z
Bron: https://wiki.osdev.org/Category:Compatability_Layers
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ACompatability_Layers&oldid=30175
Doorverwijzing: Category:Compatibility Layers

````text
#REDIRECT [[:Category:Compatibility Layers]]
````

<a id="w005010"></a>

## W005010 — User:Demindiro/Encryption

Bron-ID: W005010
Namespace: User (2)
Revisie: 30180 | Datum: 2026-03-09T19:13:56Z
Bron: https://wiki.osdev.org/User:Demindiro/Encryption
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADemindiro%2FEncryption&oldid=30180

````text
As computers are increasingly interconnected and attacks become more sophisticated it is necessary to secure data both at rest and in transit.
Much of this is done in software on the CPU, but this does not cover all hardware (such as DRAM) and may add substantial overhead.
Modern hardware often includes 

This page documents support for encryption at various layers of common hardware and how to take advantage of it in the design of an OS.

== Storage devices (HDD / SSD) ==

'''Self-Encrypting Drives''' ('''SED'''s) support transparent encryption. The encryption is always on, which allows:

* Very fast "erase" by simply discarding the current key, rendering data permanently unrecoverable.
* Adding a password to an "unencrypted" drive even if it already contains data.

Typical drives use two keys: one for encrypting data and another for authorization. The data key is encrypted with the authorization key.
The authorization key is derived from the password. By using a second key it is possible to change the password without re-encrypting all data.

AES-XTS is commonly used. This algorithm is unable to detect malicious changes to on-disk data.
This is unlikely to be a risk for encryption that occurs fully on-disk but care should be taken.

There are multiple standards. These include [[TCG_OPAL_(Disk_Encryption)|OPAL]].

== DRAM ==

AMD processors support '''Secure Encrypted Virtualization''' ('''SEV'''). It enables transparently encrypting DRAM pages and CPU registers,
preventing even the hypervisor from inspecting the state of a virtual machine.

== PCIe ==

The latest PCIe standards specify link-layer encryption support. Encryption is considered necessary as PCIe starts to scale beyond single computers.

== NICs ==

Advanced network hardware is able to handle high-level protocols typically handled by the (on-CPU) OS. Naturally this includes support for encryption.

[[Category:Security]]
````

<a id="w005011"></a>

## W005011 — Category:Version Control Systems

Bron-ID: W005011
Namespace: Category (14)
Revisie: 30182 | Datum: 2026-03-10T00:52:32Z
Bron: https://wiki.osdev.org/Category:Version_Control_Systems
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AVersion_Control_Systems&oldid=30182

````text
[[Category: Tools]]
````

<a id="w005012"></a>

## W005012 — INT 13h

Bron-ID: W005012
Namespace: Main (0)
Revisie: 30184 | Datum: 2026-03-10T20:48:07Z
Bron: https://wiki.osdev.org/INT_13h
Revisie-URL: https://wiki.osdev.org/index.php?title=INT_13h&oldid=30184
Doorverwijzing: Disk access using the BIOS (INT 13h)

````text
#REDIRECT [[Disk access using the BIOS (INT 13h)]]
````

<a id="w005013"></a>

## W005013 — User:Brain

Bron-ID: W005013
Namespace: User (2)
Revisie: 30201 | Datum: 2026-03-12T00:06:32Z
Bron: https://wiki.osdev.org/User:Brain
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABrain&oldid=30201

````text
I’m Craig Edwards, a UK software developer with a long-standing interest in systems programming and operating systems. I’ve been writing low-level code since the 1990s, starting with 6502 assembly and gradually moving into C and modern x86 systems work. Outside of OS development I’m known for creating '''D++ (DPP)''', a C++ library for building Discord bots, but most of my spare time goes into hobby operating system projects.

My OS project is '''Retro Rocket''', a 64-bit hobby OS with a kernel written in C. It takes inspiration from machines like the BBC Micro: when it boots, you land straight in a BASIC environment that acts as both programming language and shell. Userland programs (and the shell itself) are written in Retro Rocket BASIC.

Retro Rocket is intended as a learning platform for others interested in how operating systems work. The goal is to keep the system understandable so people can explore kernel design, device drivers, networking, and language integration without needing to navigate the complexity of a modern OS.
````

<a id="w005014"></a>

## W005014 — Category:PinePhone

Bron-ID: W005014
Namespace: Category (14)
Revisie: 30205 | Datum: 2026-03-12T15:37:59Z
Bron: https://wiki.osdev.org/Category:PinePhone
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APinePhone&oldid=30205

````text
'''PinePhone''' is an open hardware smartphone from Pine64 which uses an [[ARM]] CPU.

[[Category:ARM]]
````

<a id="w005015"></a>

## W005015 — Category:Pascal

Bron-ID: W005015
Namespace: Category (14)
Revisie: 30259 | Datum: 2026-03-16T16:01:45Z
Bron: https://wiki.osdev.org/Category:Pascal
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3APascal&oldid=30259

````text
Pascal is an imperative, structured programming language.
[[Category:Languages]]
````

<a id="w005016"></a>

## W005016 — Template:Distinguish

Bron-ID: W005016
Namespace: Template (10)
Revisie: 30368 | Datum: 2026-03-22T22:57:00Z
Bron: https://wiki.osdev.org/Template:Distinguish
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ADistinguish&oldid=30368

````text
:''Not to be confused with {{{text}}}''.
````

<a id="w005017"></a>

## W005017 — Linear Executable

Bron-ID: W005017
Namespace: Main (0)
Revisie: 30375 | Datum: 2026-03-22T23:32:50Z
Bron: https://wiki.osdev.org/Linear_Executable
Revisie-URL: https://wiki.osdev.org/index.php?title=Linear_Executable&oldid=30375
Doorverwijzing: LE

````text
#REDIRECT [[LE]]
````

<a id="w005018"></a>

## W005018 — User:Michaelsermir

Bron-ID: W005018
Namespace: User (2)
Revisie: 30444 | Datum: 2026-04-03T23:57:43Z
Bron: https://wiki.osdev.org/User:Michaelsermir
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMichaelsermir&oldid=30444

````text
Low level programming is fun.

I go by he/him.
````

<a id="w005019"></a>

## W005019 — User:Maxvolts

Bron-ID: W005019
Namespace: User (2)
Revisie: 30551 | Datum: 2026-05-05T05:03:04Z
Bron: https://wiki.osdev.org/User:Maxvolts
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMaxvolts&oldid=30551

````text
== '''Introduction:''' ==
I am a software engineer by trade, and have quite a bit of C/C++ experience, as well as some Assembly and Verilog experience, but haven't done many "serious" projects outside of work. I seek to change that.

These are some of my motivations/inspirations for this experiment (most are also on Youtube):

* [https://bvisness.me/high-level/ High Level is the Goal]
* [https://leechplus.net/projects/osakaOS Osaka OS]
* [https://www.mskayyali.com/interface-studies Interface Studies]
* [https://www.github.com/KaiNorberg/PatchworkOS Patchwork OS]
* [https://tirimid.net/ tirimid]
* [https://archive.org/details/lca2020-What_UNIX_Cost_Us What Unix Cost Us]


I develop with Linux for my day job, and every time I use it I ask myself... "Is this really the best it could be? Are we officially declaring this the peak of software engineering?" (Note the lack of even mentioning the other two).

So I want to experiment a little with some thoughts I've had bouncing around. Maybe I'm delusional. I guess time will tell.

I really want to build good abstractions.


If you want to peg me to the developer archetypes, I'm probably Elanore Semaphore (albiet less IPC and synchronization focused) with the downside of Duct Von Tape. 

Mono Lizzy and Alta Lang are surprise tools I'm saving for later.

== '''Blueprints OS''' ==
Status: Pivoted to Serial output for debug info, debugged the multiboot parser, and drew the OS name on the framebuffer.
[[File:SecondStepOfMany.png|thumb|Current Status]]

Current Goal: Get UEFI functioning.  Fix the linking problems preventing 64 bit certainty.

Potential Roadmap (Timescale uncertain): 

*Getting UEFI and ACPI functioning
*A graphical environment with mouse, keyboard, and sound
*File System
*Running Applications
*Multitasking
*LLVM Backend (that'll be worth a new wiki article)
*Developer Utilities (LLVM Port)
*'''''Self-Hosting'''''
*Basic Networking
*Basic Games + Non developer Utilities
*'''''DOOM port?'''''
*Release a version to the public. (tirimid?)

And further down the road I really get into some other experiments.


While I'm officially calling this an experiment and a learning experience, I hope to experiment in this direction:

Motivating Theme: An OS that is the best for both developing on and developing for.

If it seems like I want a lot of applications, it's because that'll motivate me to make this very easy to build applications on. After all, very few people actually care about the OS, mostly the applications it supports.

Potential Purpose: Computer Science Education.

==Practical Notes on "Modern" Operating System Design:==
(Maybe break out into a future wiki article. It'll live here for now.)

Modern hereby being defined as "64-bit with a predominantly graphical environment and UEFI and ACPI support that communicates over PCI". 

No DOS-likes here, and little intention on supporting hardware made before 2003 (when x64 was first created. 2011 brings AArch64 and 2014 brings RV64). 



I knew going into this that trying to make my first operating system use the newer bells and whistles (as well as straying from the usual gnu stack) would make it harder. 

My main hope is that by getting more up-to-date abstractions in place, future work will be made easier. 

Here's some lessons learned and sample code to speed past the initial grind. I'm currently still in said grind, so this will need to be updated.

If it seems like a strange chronology, I'm writing this in the order I'm doing it.

====Tech Stack in use:==== 

*Windows VSCode running the WSL extension (ubuntu) for IDE and build/run environment
*clang as the compiler (but seemingly ld as the underlying linker) (This is done because it's way easier than trying to coax a cross-compiler out of gcc. WSL and gcc compilation do not mix.)
*GRUB2 as the bootloader, grub-mkrescue to create the iso
*QEMU as the emulator
*OVMF as the firmware
*C++ as the main language

We start from bare bones, so make sure you get that working first. 


Swapping gcc for clang is a trivial change for that example. Just make sure to include "-target i686-elf" in your compilation flags, for 32-bit support. 64-bit support is more complicated due to the elf-64 format being different, so hold off on the "-target x86_64-elf" target for now. (he says knowing full well even he hasn't solved this yet).


Swapping C++ for C just needs you to  wrap your kernel_main in extern "C" {  void kernel_main(){} }, to prevent name mangling. 

This gives you an immediate benefit of allowing you to overload functions in the C++. Such as, for example, different terminal_print functions for different things.

====Pivoting to Multiboot2:====
Perhaps the next easiest changes to make from bare bones is to upgrade your bootloader to multiboot2. This should provide us (later down the line) with utilities such as the EFI system pointer, ACPI RSDP, DHCP information, and a real framebuffer.

Below is effectively a different copy of bare bones, but with multiboot2 support. You can yoink the multiboot2 header from here.

https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Example-OS-code

The only details you will need to contribute yourself is a slight modification to your  grub.cfg and your linker script . 

In grub.cfg: change the multiboot command to multiboot2.

The multiboot2 header above is looking for symbols defining the end of the data and bss sections, but the example code does not provide them, nor does it provide a linker script at all.

Here's where you drop the two new symbols you need

(Pretend this code block is configured for ld instead of asm)<syntaxhighlight lang="asm">
/* Read-write data (initialized) */

.data BLOCK(4K) : ALIGN(4K)

{

*(.data)

}

_edata = .;

/* Read-write data (uninitialized) and stack */

.bss BLOCK(4K) : ALIGN(4K)

{

*(COMMON)

*(.bss)

}

_end = .;
</syntaxhighlight>

'''You will ABSOLUTELY need an address tag and an entry address tag to be added. However, try to hold off on the framebuffer tag to preserve your VGA display.'''

As for the information request tag. You can request the following information

Stuff you can acquire immediately:

1: Boot command line (useless right now)

2: Bootloader name (pretty good sanity check to make sure you're parsing it correctly)

4: Basic memory information

6: Memory Map (though get ready for the annoyance of 64 bit pointers while still in 32 bit mode)


And stuff it might just provide entirely unsolicited:

10 - APM table (soon to be wiped out with ACPI)

=====Parsing the multiboot2 results:=====
Parsing multiboot2's provided data is rather simple, even if you don't want to copy their example code. Once you have the multiboot pointer: 

Acquire the 32 bit size at that address.

Advance 64 bits (recall that in C, if you're using a pointer to 32 bit data, adding to it adds in intervals of its data size, so += 2).

Loop until you've reached the end of the base pointer + the size:


Get the 32-bit value at your current address (tag type). 

Here's what each tag type does:

https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Boot-information-format

Switch based on this to determine what to display or do.

*You should expect a load address to always be available (tag 21). Since bare bones uses a relocatable elf file.
*Odds are, your boot command line is just an empty string, so it will have a size of 9.
*Since the Boot command line and bootloader names are literally null-terminated C strings, you can make a char* to their data and print them.

Advance your current address the amount of bytes stored in the 32-bit value after your current address, rounded up to the nearest 8 for alignment purposes.<syntaxhighlight lang="c++">
if(size % 8 != 0){
    size += (8 - (size % 8))
}
</syntaxhighlight>(With 32 bit pointer addition, make sure to divide this value by four)

==== 64 Bit Notes: ====
When trying to compile your kernel 64 bits, you may try to use clang as a driver for your linker.

Do not do this. Clang will try to be too smart about the situation and try some optimizations that will not work, resulting in incredibly cryptic errors like

"relocation R_X86_64_32 against `.stack' can not be used when making a shared object; recompile with -fPIC", even when everything has -fPIC. 

Call the linker directly and the problem will be solved.

Here's also another place for a sample 64 bit kernel:

https://github.com/davidcallanan/os-series/tree/master
==== Getting QEMU logs: ====
Virtually everything we are about to do will wipe out the VGA display we worked so hard to produce in bare bones. In order to get data out of this system, we'll need other routes.

One easy thing to do is to turn on QEMU logging for errors and cpu reset data. Add these flags to your qemu command to get more data.

-d gives the list of things to log, and -D picks the output file.<syntaxhighlight lang="bash">
-d guest_errors,cpu_reset -D ./qemu_log.txt
</syntaxhighlight>

==== Serial Port output: ====
Because VGA text mode will go away with virtually anything we do next, we need another way of getting runtime data out until we can get our Multiboot2-provided framebuffer functional.<syntaxhighlight lang="asm">
/* Print Ok followed by a newline and carriage return */
mov $0x3f8, %dx 
/* This is the COM1 port, which in the qemu interface is 
    tied to the default serial view */

mov $0x4F, %ax /* O */

/* I love magic registers!!1! (Output always comes from ax)*/
outb %dx

mov $0x6B, %ax /* k */
outb %dx

mov $0xA, %ax /* New Line */
outb %dx

mov $0xD, %ax /* We're not in a unix system, so you unironically need to 
    do the windows thing and also carriage return */

outb %dx


</syntaxhighlight>When in your C/C++, use this code from this very wiki:

[[Inline Assembly/Examples#OUTx]]<syntaxhighlight lang="c++">
static inline void outb(uint16_t port, uint8_t val)
{
    __asm__ volatile ( "outb %b0, %w1" : : "a"(val), "Nd"(port) : "memory");
}
/* Set port to 0x3f8 to output to QEMU serial */
</syntaxhighlight>Once you've swapped out bare bones printing to screen to printing to serial, might as well throw out the old code and add the framebuffer tag.

====Customization Sidetrack: ====
Who said we can't have a little sidetrack along the way? This is purely for GRUB2, as part of our tech stack.

Looking online for GRUB configuration data is seemingly a fool's errand, as they all are under the assumption you're trying to configure GRUB on an operating system that already exists.

However, with a little digging, you can customize even with as little GRUB as you have, as a treat.


In your grub.cfg, you can add lines of format<syntaxhighlight lang="bash">
set [VARIABLENAME]=[VALUE]
</syntaxhighlight>(Note the lack of quotes around the value).

Your options for meaningful variables can be found here:

https://www.gnu.org/software/grub/manual/grub/html_node/Special-environment-variables.html


These four let you change the background and text color on GRUB:

color_normal, color_highlight, menu_color_normal, menu_color_highlight

Specifically, they have a format of textcolor/backgroundcolor, where your color options are pretty slim. 

(black, blue, green, cyan, red, magenta, brown, light-gray, dark-gray, light-blue, light-green, light-cyan, light-red, light-magenta, yellow, white)


With a little modification to your menuentry, you can get some neat support.<syntaxhighlight lang="bash">
menuentry "Your OS Name Here" --id yourosnamehere
</syntaxhighlight>


With that you can add these variables:<syntaxhighlight lang="bash">
set default=yourosnamehere
set savedefault=true
set timeout=5
set timeout_style=hidden
</syntaxhighlight>



These will cause GRUB to automatically and quietly boot into your OS after a 5 second period. You can jump to the GRUB menu by hitting escape during this process.

If you set the timeout to 0, it won't even wait, but you will need to be pre-emptively hitting esc.


Theoretically you can go further than that

https://www.gnu.org/software/grub/manual/grub/html_node/Shell_002dlike-scripting.html

https://www.gnu.org/software/grub/manual/grub/html_node/Multi_002dboot-manual-config.html


As for themes. You should be able to apply them, but I haven't figured that part out myself because every theme resource assumes you have all of your linux directories and uses GRUB's automatic configuration tools on /etc/default/grub to create your grub.cfg.

'''In theory''' you should just put another directory on your iso, then:<syntaxhighlight lang="bash">
set theme=/pathtothemes/themename/theme.txt
</syntaxhighlight>


With a properly formatted theme.txt file

(Here's formatting)

https://www.gnu.org/software/grub/manual/grub/html_node/Theme-file-format.html

(Here's a more useful page)

https://web.archive.org/web/20241209100014/http://wiki.rosalab.ru/en/index.php/Grub2_theme_tutorial

And that should allow things to work.


But I can't figure that out fully and this sidetrack is already long enough.



To add a splash screen to QEMU before boot, you can add this to the command line<syntaxhighlight lang="bash">
-boot menu=on,splash=Splash.bmp,splash-time=2500
</syntaxhighlight>
Where Splash.bmp should be 320x240, 640x480 or 800x640. Supposedly it also works with jpeg, but I couldn't see that work.


You can also add your OS name to the window with:<syntaxhighlight lang="bash">
-name YourOSNameHere
</syntaxhighlight>


And can shift your iso file to be quicker to boot by putting it in a non-CD drive with<syntaxhighlight lang="bash">
-drive file=/PathToIso/YourOSNameHere.iso,format=raw
</syntaxhighlight>

====Getting Firmware in place:====
QEMU does not come with a UEFI or ACPI2-supporting Firmware by default. If you were looking for that data in the multiboot tags, you aren't going to find it until you swap the bios out.


We're going to be using the Open Virtual Machine Firmware (OVMF), locatable here: http://www.tianocore.org/tianocore-wiki.github.io/index.html



Something about ubuntu having a signed copy for secure boot purposes. Not yet 100% what that's about, but I'm pretty sure that explains why WSL has a copy of it provided in /usr/share/ovmf.


To be honest, I'm not entirely sure if/how this 100% works. (It also wrecks your GRUB style atm, sorry).


You're going to need grub-efi support on your grub-mkrescue. 

Make sure to sudo apt install grub-efi (or grub-efi-amd64 or grub-efi-amd64-bin, I'm not entirely sure which one).



Once you have that all set up, getting qemu to run with the firmware is as simple as adding <syntaxhighlight lang="bash">
-bios /usr/share/ovmf/OVMF.fd
</syntaxhighlight>to your qemu command line. (Or whatever path you have to this file. This is where my specific stack makes it easy.)

==== UEFI: ====
As for what UEFI is good for. Don't be swayed by the swan song. Once boot services are closed (as any good bootloader should do to give the OS its memory back), like 90% of the specification becomes irrelevant to you.

It's only real applications to an OS are:

* Providing a table of other, more useful, non-uefi tables
*Allowing you to update firmware
*Get/set the system time (with dividing the value up and timezones already handled) and set a wakeup time
*Get the ominous value that increments every single time you boot your system.
*Shutdown/Reboot the system<br /> More info might be coming once I figure out what vendor-defined variables are useful for. The spec doesn't really say.


So while the spec is long and full of tempting abstractions, you can only really use these pages:

https://uefi.org/specs/UEFI/2.10/02_Overview.html

https://uefi.org/specs/UEFI/2.10/04_EFI_System_Table.html

https://uefi.org/specs/UEFI/2.10/08_Services_Runtime_Services.html

Come back to the spec when you're making your own bootloader.
====ACPI:====
You only really need this one very long page.

https://uefi.org/htmlspecs/ACPI_Spec_6_4_html/05_ACPI_Software_Programming_Model/ACPI_Software_Programming_Model.html



==== 64-bit Mode: ====
This code is cobbled together from a few different tutorials on this very wiki (in progress):<syntaxhighlight lang="asm">
/* Check for Required Features */

/* 1: Can it use cpuid? */

pushfd

pushfd

xorl $0x00200000, (%esp)

popfd

pushfd

popl %eax

xorl %eax, (%esp)

popfd

andl $0x00200000, %eax

jz failure

/* 2: does it have the required cpuid extended parameters? */

mov $0x80000000, %eax

cpuid

cmp $0x80000001, %eax

jl failure

/* 3: Is 64 bit mode available? */

mov $0x80000001, %eax

cpuid

and $0x20000000, %edx

jz failure

/* With 64 bit mode confirmed available, we set things up for it */

/* Set the page table's root */

movl $0x1000, %eax

movl %eax, %cr3

/* put 0x1000 registers worth of 0 in*/

xorl %eax, %eax

movl $0x1000, %ecx

rep stosl

/* Establish Table Links (the 3 indicates it is both present and readable)*/

movl %cr3, %edi

movl $0x2003, (0x1000) /* First entry of top table -> second table */

movl $0x3003, (0x2000) /* First entry of second table -> third table */

movl $0x4003, (0x3000) /* First entry of third table -> fourth table */

/* Set up Physical Address Extension */

mov %cr4, %eax

or $0x20, %eax

mov %eax, %cr4

/* Switch to compatibility mode */

mov (0xC0000080), %ecx

rdmsr

or $0x80, %eax

wrmsr

/* Turn on paging and protected mode */

mov %cr0, %eax

or %eax, 0x80000001

mov %eax, %cr0

/* Load GDT */

lgdt GDT

/* Head to kernel main */

call kernel_main
</syntaxhighlight>
````

<a id="w005020"></a>

## W005020 — I²C

Bron-ID: W005020
Namespace: Main (0)
Revisie: 30453 | Datum: 2026-04-05T17:54:47Z
Bron: https://wiki.osdev.org/I%C2%B2C
Revisie-URL: https://wiki.osdev.org/index.php?title=I%C2%B2C&oldid=30453

````text
{{Stub}}

'''I²C''' (Inter-integrated Circuit) is a serial, half-duplex and low-bandwidth communication bus. Its main use is for interconnecting ICs (EPROMs, microcontrollers) and other low-bandwidth devices. I²C is used for carrying [[EDID]] monitor data through VGA, HDMI and DisplayPort cables using the [[DDC]] protocol.

== See Also ==
=== Articles ===
* [[Low Pin Count]]
* [[Serial Peripheral Interface]]

{{Category:Buses}}
````

<a id="w005021"></a>

## W005021 — I2C

Bron-ID: W005021
Namespace: Main (0)
Revisie: 30454 | Datum: 2026-04-05T17:55:29Z
Bron: https://wiki.osdev.org/I2C
Revisie-URL: https://wiki.osdev.org/index.php?title=I2C&oldid=30454
Doorverwijzing: I²C

````text
#REDIRECT [[I²C]]
````

<a id="w005022"></a>

## W005022 — Talk:SSL/TLS

Bron-ID: W005022
Namespace: Talk (1)
Revisie: 30464 | Datum: 2026-04-08T07:36:59Z
Bron: https://wiki.osdev.org/Talk:SSL/TLS
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ASSL%2FTLS&oldid=30464

````text
I am concerned that this page places too much emphasis on implementing TLS yourself, which encourages a "roll your own crypto" approach rather than the far more practical and secure approach of integrating an existing library such as mbedTLS.

Cryptography and TLS in particular is an area where partial understanding is actively dangerous. It is very easy to reach a point where an implementation appears correct, yet is fundamentally insecure due to issues such as timing side channels, padding oracles, or subtle state machine flaws. These are not edge cases, they are well documented classes of real world attacks.

Crypto code is not like typical systems code such as memcpy, where functional correctness is enough. In TLS an implementation can be "correct" and still completely broken from a security perspective.

The statement that "implementing only a few popular cipher suites is enough for most cases" is also no longer accurate in 2026. Modern TLS deployments typically require features such as ECDHE and AEAD ciphers (e.g. GCM or ChaCha20-Poly1305), and often disable the legacy CBC/SHA1-based suites used as examples here. A minimal implementation based on those examples is unlikely to talk to many real world systems.

While the article does note that implementing your own TLS is no guarantee of security, this raises a more fundamental question: if the result is not secure, what is the value of implementing it at all? TLS exists specifically to provide '''security'''; anything less defeats its purpose.

I would suggest restructuring this page to focus primarily on integrating a well-established TLS library (e.g. mbedTLS), which is the approach used in practice. A smaller, clearly marked section could remain for educational purposes, explaining how TLS works at a high level without encouraging incomplete or insecure implementations.

I am happy to contribute a revised version of this page based on integrating mbedTLS, as I have already done this in my own OS. This is not to suggest there is only one correct approach, but it would provide a far safer and more practical starting point than implementing TLS from scratch. -- [[User:Brain|Brain]] ([[User talk:Brain|talk]])

: Hmm, I mean, I do think it's nice to cover some of the underlying theory, but we don't have the same sort of thing for IP or TCP, for instance...

: mbedTLS or something definitely could be useful. I'd keep the existing content, but put it in a section at the end with a stronger warning. [[User:Bellezzasolo|Bellezzasolo]] ([[User talk:Bellezzasolo|talk]]) 07:36, 8 April 2026 (UTC)
````

<a id="w005023"></a>

## W005023 — User:Freezo

Bron-ID: W005023
Namespace: User (2)
Revisie: 30461 | Datum: 2026-04-07T03:29:12Z
Bron: https://wiki.osdev.org/User:Freezo
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AFreezo&oldid=30461

````text
'''hello,'''

i,am Freezo!
````

<a id="w005024"></a>

## W005024 — RISC-V Bare Bones

Bron-ID: W005024
Namespace: Main (0)
Revisie: 30469 | Datum: 2026-04-10T16:08:54Z
Bron: https://wiki.osdev.org/RISC-V_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=RISC-V_Bare_Bones&oldid=30469
Doorverwijzing: QEMU RISC-V Virt Bare Bones

````text
#REDIRECT [[QEMU RISC-V Virt Bare Bones]]
````

<a id="w005025"></a>

## W005025 — RISC-V Meaty Skeleton with QEMU virt board

Bron-ID: W005025
Namespace: Main (0)
Revisie: 30472 | Datum: 2026-04-10T16:14:38Z
Bron: https://wiki.osdev.org/RISC-V_Meaty_Skeleton_with_QEMU_virt_board
Revisie-URL: https://wiki.osdev.org/index.php?title=RISC-V_Meaty_Skeleton_with_QEMU_virt_board&oldid=30472
Doorverwijzing: QEMU RISC-V Virt Meaty Skeleton

````text
#REDIRECT [[QEMU RISC-V Virt Meaty Skeleton]]
````

<a id="w005026"></a>

## W005026 — File:Screenshot of COS.png

Bron-ID: W005026
Namespace: File (6)
Revisie: 30489 | Datum: 2026-04-17T07:14:49Z
Bron: https://wiki.osdev.org/File:Screenshot_of_COS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AScreenshot_of_COS.png&oldid=30489

````text
Screenshot of GUI of Crusader-OS
````

<a id="w005027"></a>

## W005027 — File:SecondStepOfMany.png

Bron-ID: W005027
Namespace: File (6)
Revisie: 30505 | Datum: 2026-04-21T01:56:03Z
Bron: https://wiki.osdev.org/File:SecondStepOfMany.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ASecondStepOfMany.png&oldid=30505

````text
"BLUEPRINTS" written in a QEMU Framebuffer
````

<a id="w005028"></a>

## W005028 — AArch64 Exceptions

Bron-ID: W005028
Namespace: Main (0)
Revisie: 30938 | Datum: 2026-09-03T15:43:31Z
Bron: https://wiki.osdev.org/AArch64_Exceptions
Revisie-URL: https://wiki.osdev.org/index.php?title=AArch64_Exceptions&oldid=30938

````text
Exceptions are events that tell the processor something has happened that it needs to process. They are also called interrupts, because they interrupt whatever the processor was doing.

== Differences from x86 ==
On [[x86]], there are 256 interrupt vectors that can be called by the hardware. Some of these are used by the processor itself to report errors, such as invalid instructions or unpermitted memory accesses. The rest are used by hardware peripherals, usually through a bus like [[PCI]]. On [[AArch64]], there are only 4 exception vectors. Two of these are used for interacting with hardware, and the other two are used for reporting errors. On x86, the vector table (called the [[IDT]]) contains addresses for the processor to jump to. However, AArch64 has instructions directly in the exception table itself, and each vector is 128 bytes long, giving room for 32 instructions.

== Exception Vectors ==

Each vector must be 128-byte aligned, and the table itself must be 2048-byte aligned.

{| class="wikitable"
|-
! Name !! Offset !! Description
|-
| Synchronous || <tt>0x000</tt> || CPU exception occurred due to instruction
|-
| IRQ || <tt>0x080</tt> || Interrupt Request
|-
| FIQ || <tt>0x100</tt> || Fast Interrupt Request
|-
| SError || <tt>0x180</tt> || CPU exception unrelated to instructions
|}

== VBAR and DAIF ==

On [[ARMv7]] and before, the exception table could be placed at one of two places; either at <tt>0x00000000</tt>, or <tt>0xffff0000</tt>. AArch64 allows for much more precise control by introducing the VBAR (Vector Base Address Register). This register holds the location of the vector table, and each exception level (except EL0) has its own base.

The VBAR is accessed just like any other special register, with the MSR or MRS instructions.

<syntaxhighlight lang=asm>
// address to set to in x0
set_vbar_el1:
    msr vbar_el1,x0
    ret
</syntaxhighlight>

In ARM speak, you disable exceptions by masking them so they don't trigger. This is done with the DAIF register. DAIF holds four flags that determine the masking of exceptions. It is effectively analogous to the Interrupt Flag on x86.

{| class="wikitable"
|+ DAIF
|-
| Reserved || D || A || I || F || Reserved
|-
! 10-63 !! 9 !! 8 !! 7 !! 6 !! 0-5
|}
*D - Debug (Synchronous)
*A - SError
*I - IRQ
*F - FIQ

DAIF is also accessed in the same way VBAR is via the MSR and MRS instructions.

<syntaxhighlight lang=asm>
// flags in x0
set_daif:
    msr daif,x0
    ret
</syntaxhighlight>

Setting DAIF to <tt>0x000</tt> will disable exception masking, while setting it to <tt>0x3c0</tt> will mask all exceptions, meaning none will occur.

== Vector Table ==

The vector table contains 4 sets of 4 vectors that determine the code that runs when an exception occurs. Each set of 4 vectors is for a different mode the processor may be operating in when the exception occurs.

{| class="wikitable"
|+ Vector Table
|-
! Mode !! Offset !! Description
|-
| EL1t || <tt>0x000</tt> || EL1 with EL0 stack
|-
| EL1h || <tt>0x200</tt> || EL1 with own stack
|-
| EL0 64 || <tt>0x400</tt> || EL0 64-bit
|-
| EL0 32 || <tt>0x600</tt> || EL0 32-bit
|}

The last entry is provided for compatibility with 32-bit user-mode programs, but the handlers themselves are still 64-bit.

== Syndromes ==

Because [[AArch64]] only has 4 exception vectors, it gives you additional information on why the exception occurred in the form of the Exception Syndrome Register. Upon an exception, this register is populated with information like the exact cause (Exception Class), extra information, and instruction type.

{| class="wikitable"
|+ ESR
|-
| Reserved || ISS2 || EC || IL || ISS
|-
! 37-63 !! 32-36 !! 26-31 !! 25 !! 0-24
|}
*ISS - Instruction Specific Syndrome, format specific to exception class
*IL - Instruction Length, if the instruction was 16-bit (0) or 32-bit (1)
*EC - Reason for the exception
*ISS2 - Register used in failed atomic store (FEAT_LS64 only)

== ELR and SPSR ==

Once an exception has been processed, the processor needs to return back to where it was, and reset the PSTATE to what it was before the exception. The processor automatically stores this information in the Exception Link Register and Saved Program Status Register.

The ELR works in a very similar way to the link register, but it might not point to the next instruction, but rather run it again if it caused an exception or did not finish execution. The SPSR contains various PSTATE values, such as DAIF and ALU flags.

Returning from an exception to the ELR while setting PSTATE is automatically done by the ERET instruction. However, knowing this information is valuable to the kernel so it can be provided to debuggers.

== Example ==

<syntaxhighlight lang=asm>
_start:
    // setup vbar
    ldr x0,=vectors
    msr vbar_el1,x0
    
    // enable exceptions
    msr daif,xzr
    
    // undefined instruction that will trigger a Synchronous exception
    udf #0xdead

// macro that requires alignment and jumps to a label
.macro vector label
.p2align 7
b \label
.endm

// dummy exception handler that just halts forever
vinvalid:
    b .

// vector table must be 2048-byte aligned
.p2align 11
vectors:
    // EL1 keep stack
    vector vinvalid
    vector vinvalid
    vector vinvalid
    vector vinvalid

    // EL1 own stack (kernel mode)
    vector vinvalid
    vector vinvalid
    vector vinvalid
    vector vinvalid

    // EL0->EL1 (user mode)
    vector vinvalid
    vector vinvalid
    vector vinvalid
    vector vinvalid

    // 32-bit EL0->EL1
    vector vinvalid
    vector vinvalid
    vector vinvalid
    vector vinvalid
</syntaxhighlight>

== See Also ==
=== External Links ===
* [https://developer.arm.com/documentation/ddi0487/gb/ The ARMv8 Reference Manual]

[[Category:ARM]]
[[Category:AArch64]]
````

<a id="w005029"></a>

## W005029 — User:Finxx/AArch64 Exceptions

Bron-ID: W005029
Namespace: User (2)
Revisie: 30531 | Datum: 2026-04-29T21:32:32Z
Bron: https://wiki.osdev.org/User:Finxx/AArch64_Exceptions
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AFinxx%2FAArch64_Exceptions&oldid=30531
Doorverwijzing: AArch64 Exceptions

````text
#REDIRECT [[AArch64 Exceptions]]
````

<a id="w005030"></a>

## W005030 — User:Xingji1919810

Bron-ID: W005030
Namespace: User (2)
Revisie: 30535 | Datum: 2026-04-30T09:20:13Z
Bron: https://wiki.osdev.org/User:Xingji1919810
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AXingji1919810&oldid=30535

````text
=Who am I=
A Student of The Affiliated High School of Peking University Chaoyang School Junior High Division (RDFZ Chaoyang School)

XINGJI Studios's leader (www.xingjisoft.com)

Bilibili Uploader (https://space.bilibili.com/1591866703)

XJ380 System Chief Engineer

=About My Team=
We are a team composed of Chinese students aged 10 to 18. We really like programming and we write many programs, such as a browser and an operation system.
````

<a id="w005031"></a>

## W005031 — ZXFL Barebones

Bron-ID: W005031
Namespace: Main (0)
Revisie: 31011 | Datum: 2026-10-11T10:46:49Z
Bron: https://wiki.osdev.org/ZXFL_Barebones
Revisie-URL: https://wiki.osdev.org/index.php?title=ZXFL_Barebones&oldid=31011

````text
{{BeginnersWarning}}
{{Rating|2}}

== About ==

A minimal kernel targeting IBM z/Architecture (s390x), using '''ZXFoundationLoader (ZXFL)''' dual-stage loader.

{{AdmonitionNote|text=This is a minimal barebones intended to get you to a booting kernel as quickly as possible. It deliberately omits explanation of z/Architecture concepts (IPL, DASD, PSW, boot protocol internals, etc.). See the  IBM Principle Of Operations or [https://assembler-0.github.io/ZXFoundation ZXFoundation API documentation] for a thorough treatment of all of these.}}

* '''ZXFoundation source:''' [https://github.com/assembler-0/ZXFoundation GitHub]
* '''Documentation:''' [https://assembler-0.github.io/ZXFoundation assembler-0.github.io/ZXFoundation]

== Why ZXFoundationLoader? ==

Realistically, this is one of the few and only hobbyist IPL loader for the s390x architecture. There are some existing projects like [[FLATBOOT_Barebones|FLATBOOT]] here are its limitation:

* Adapts (and awkwardly implements) x86 protocol stivale2 on a mainframe architecture
* Full EBCDIC end to end (which not all compiler installation supports)
* Doesn't even support ELF64, 32-bit ELFs only
* Abandoned/Inactive (?)


== Prerequisites ==

* <code>s390x-ibm-linux-gnu-gcc</code> cross-compiler (or any <code>s390x-*-gcc</code>)
* <code>s390x-ibm-linux-gnu-ld</code>
* <code>dasdload</code> (Hercules tools)
* ZXFoundation binaries: <code>core.zxfoundationloader00.sys</code>, <code>core.zxfoundationloader01.sys</code> and <code>zxsign</code>

== Compiling ZXFoundation ==

<syntaxhighlight lang="sh">
git clone https://github.com/assembler-0/ZXFoundation.git
cd ZXFoundation
mkdir build && cd build
cmake .. -DZX_TOOLCHAIN_FILE=../cmake/toolchain/zxfoundation-gcc.cmake -DZX_CONFIG_FILE=../cmake/configs/base.cmake -GNinja
ninja
ls -l core.zxfoundationloader00.sys core.zxfoundationloader01.sys zxsign
# now, you can copy the listed files to the barebones sources (i.e. ZXFoundation/docs/barebones/)
</syntaxhighlight>

== Source Files ==

* main.c - kernel main entry point
<syntaxhighlight lang="c">
#include <stdint.h>
#include "zxfl.h"

[[noreturn]] void zxfoundation_global_initialize(zxfl_boot_protocol_t *boot) {
    while (1) __asm__ volatile("nop");
}
</syntaxhighlight>

* head64.S - assembly stub
<syntaxhighlight lang="asm">
// arch/s390x/init/head64.S (ZXFoundation)

#include <arch/s390x/init/zxfl/zxvl.h>
#include <arch/s390x/init/zxfl/zxfl.h>

#define KERNEL_BOOT_STACK_SIZE 0x4000

    .section .text.handshake, "ax"
    .globl __zxfl_handshake_stub
__zxfl_handshake_stub:
    llihf   %r0, ZXVL_SEED_HI
    iilf    %r0, ZXVL_SEED_LO
    xgr     %r2, %r0
    lgr     %r0, %r2
    sllg    %r0, %r0, 17
    srlg    %r1, %r2, 47
    ogr     %r0, %r1
    llihf   %r1, ZXVL_HS_RESPONSE_HI
    iilf    %r1, ZXVL_HS_RESPONSE_LO
    lgr     %r2, %r0
    agr     %r2, %r1
    br      %r14

    .section .text.__zx_start_kernel64, "ax", @progbits
    .globl __zx_start_kernel64
__zx_start_kernel64:
    lgr     %r11, %r2

    larl    %r2, __bss_start
    larl    %r3, __bss_end
    sgr     %r3, %r2                       // R3 = BSS size in bytes
    jz      .Lbss_done
    lgr     %r4, %r2                       // R4 = source (same as dest)
    sgr     %r5, %r5                       // R5 = 0 (source length 0 + pad byte 0x00)
.Lmvcl_retry:
    mvcl    %r2, %r4
    jo      .Lmvcl_retry
.Lbss_done:

    lgr     %r2, %r11
    larl    %r15, __kernel_boot_stack_top
    aghi    %r15, -160                     // 160-byte register save area

    // Enable AFP (bit 45) and Vector (bit 46) for BSP before entering C++ (you can remove this if you dont need these features. though compilers may assume this feature)
    stctg   %c0, %c0, 0(%r15)
    lg      %r1, 0(%r15)
    iilf    %r0, 0x00060000
    ogr     %r1, %r0
    stg     %r1, 0(%r15)
    lctlg   %c0, %c0, 0(%r15)

    // Zero out backchain and r14 save area to terminate unwinder gracefully
    sgr     %r0, %r0
    stg     %r0, 0(%r15)
    stg     %r0, 112(%r15)

    lgr     %r2, %r11
    brasl   %r14, zxfoundation_global_initialize

.Lhalt:
    larl    %r1, .Ldisabled_wait_psw
    lpswe   0(%r1)
    j       .Lhalt

    .align 8
.Ldisabled_wait_psw:
    .quad   PSW_MASK_DISABLED_WAIT, 0x0000000000000000

    // BSP boot stack: kernel-owned, in .bss, zeroed by the entry path above.
    .section .bss.boot_stack, "aw", @nobits
    .align 16
    .globl __kernel_boot_stack
__kernel_boot_stack:
    .skip   KERNEL_BOOT_STACK_SIZE
    .globl __kernel_boot_stack_top
__kernel_boot_stack_top:

    .section .note.GNU-stack, "", @progbits
</syntaxhighlight>

* link.ld - linker script
<syntaxhighlight lang="asm">
/* arch/s390x/init/link.ld — ZXFoundation Kernel Linker Script */

ENTRY(__zx_start_kernel64)

PHDRS {
    hs_seg        PT_LOAD FLAGS(0x00400005);  /* ZXVL_PFLAGS_HS:    PF_X|PF_R + bit22 */
    entry_seg     PT_LOAD FLAGS(0x00800005);  /* ZXVL_PFLAGS_ENTRY: PF_X|PF_R + bit23 */
    nucleus_text  PT_LOAD FLAGS(0x00000005);  /* PF_X|PF_R  — kernel .text             */
    nucleus_ro    PT_LOAD FLAGS(0x00000004);  /* PF_R       — kernel .rodata            */
    nucleus_data  PT_LOAD FLAGS(0x00000006);  /* PF_R|PF_W  — kernel .data             */
    nucleus_bss   PT_LOAD FLAGS(0x00000006);  /* PF_R|PF_W  — kernel .bss              */
    lock_seg      PT_LOAD FLAGS(0x00100006);  /* ZXVL_PFLAGS_LOCK:  PF_R|PF_W + bit20 */
    checksums_seg PT_LOAD FLAGS(0x00200004);  /* ZXVL_PFLAGS_CKSUM: PF_R + bit21      */
}

nucleus_vma_base = 0xFFFF800000100000;

SECTIONS {
    . = nucleus_vma_base;
    __zx_vma_start = .;

    .text.handshake : ALIGN(8) {
        KEEP(*(.text.handshake))
    } :hs_seg

    .text.entry : ALIGN(8) {
        KEEP(*(.text.__zx_start_kernel64))
    } :entry_seg

    .text : ALIGN(8) {
        *(.text .text.*)
    } :nucleus_text

    .rodata : ALIGN(4096) {
        __rodata_start = .;
        *(.rodata .rodata.*)
        __rodata_end = .;
    } :nucleus_ro

    .data : ALIGN(4096) {
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    } :nucleus_data

    .bss : ALIGN(4096) {
        __bss_start = .;
        *(COMMON)
        *(.bss .bss.*)
        __bss_end = .;
    } :nucleus_bss

    .zxfl_lock : ALIGN(4096) {
        LONG(0xCCBBCC35)
        LONG(0x5A58464C)
        . += 0x1000 - 8;
        LONG(0xE5664311)
    } :lock_seg

    .zxvl_checksums : ALIGN(4096) {
        KEEP(*(.zxvl_checksums))
    } :checksums_seg

    __zx_vma_end = .;

    /DISCARD/ : { *(.comment) *(.eh_frame) *(.note.GNU-stack) }
}
</syntaxhighlight>

* zxvl_cksum.c - SHA256 kernel checksum
<syntaxhighlight lang="c">
#include "zxvl.h"

__attribute__((section(".zxvl_checksums"), used))
zxvl_checksum_table_t zxvl_kernel_checksum_table = { 0 };
</syntaxhighlight>

* sysres.conf - dasdload control file
<syntaxhighlight lang="txt">
sysres 3390-2 * core.zxfoundationloader00.sys
core.zxfoundationloader01.sys SEQ core.zxfoundationloader01.sys TRK 5 0 0 PS FB 4096 4096 0
core.zxfoundation.nucleus SEQ core.zxfoundation.nucleus TRK 300 0 0 PS FB 4096 4096 0
etc.zxfoundation.parm SEQ etc.zxfoundation.parm TRK 2 0 0 PS FB 4096 4096 0
</syntaxhighlight>

* hercules.conf - hercules script
<syntaxhighlight lang="txt">
ARCHLVL      z/Arch
CPUSERIAL    000666
CPUMODEL     $(z16)
LPARNUM      1
LPARNAME     ZXF
PLANT        ZZ
MANUFACTURER IBM
MAINSIZE     512M
XPNDSIZE     0
NUMCPU       4
MAXCPU       8
DIAG8CMD     ENABLE
CNSLPORT     3270
0009         3215-C  /
001C         SYSG
001F         3270

0100        3390    sysres.3390

IPL 0100
</syntaxhighlight>

* etc.zxfoundation.parm - parameter file (cmdline)
<syntaxhighlight lang="txt">
syssize=512M sysmodule=
</syntaxhighlight>

== Using with C++ (modules) == 
ZXFL is capable of loading a C++ kernel (like most other loader). With traditional C++, ZXFL protocol (the C header) can easily be adapted. For C++ modules, the protocol [https://github.com/assembler-0/ZXFoundation/blob/master/arch/s390x/init/zxfl/protocol.cxxm module] is not entirely freestanding-compatible, and depends on several simple dependencies:
* ZXFoundation fundamental types (rust-like) (can be trivially replaced by standard C/C++ types)
* STFLE_MAX_DWORDS (just a number)
* CONFIG_ZX_MAX_CPUS (your configuration)
* SHA256_DIGEST_SIZE (also a number)
* string_view-like implementation (can be swapped out for <code>const char*</code>)
Though, is possible to adapt/hard-code for your kernel. 

== Environment and Compilation (from scratch) ==

<syntaxhighlight lang="sh">
# navigate to your source dir
# copy your ZXFoundation binaries here

# get header files
wget https://raw.githubusercontent.com/assembler-0/ZXFoundation/refs/heads/master/arch/s390x/init/zxfl/include/arch/s390x/init/zxfl/zxfl.h
wget https://raw.githubusercontent.com/assembler-0/ZXFoundation/refs/heads/master/arch/s390x/init/zxfl/include/arch/s390x/init/zxfl/zxvl.h

# make sure your sources are here

export CFLAGS="-ffreestanding -nostdlib -msoft-float -mno-vx -m64 -march=z13 -mtune=z13 \
        -pipe -fno-stack-protector -std=c23 -mzarch -static-libgcc -O2 -g0 -I."
export LDFLAGS="-nostdlib -static --no-dynamic-linker -ztext -zmax-page-size=0x1000 \
         --no-pie -g -melf64_s390"

# It is possible to use clang with target triple s390x-{unknown-none,linux-gnu}-elf
s390x-ibm-linux-gnu-gcc $CFLAGS -c head64.S     -o head64.o
s390x-ibm-linux-gnu-gcc $CFLAGS -c main.c       -o main.o
s390x-ibm-linux-gnu-gcc $CFLAGS -c zxvl_cksum.c -o zxvl_cksum.o
s390x-ibm-linux-gnu-ld  -T link.ld $LDFLAGS head64.o main.o zxvl_cksum.o \
        -o core.zxfoundation.nucleus

# Embed checksums required by the ZXFL ZXVerifiedLoad protocol
./zxsign core.zxfoundation.nucleus

# Build the DASD image
dasdload -z sysres.conf sysres.3390
</syntaxhighlight>

The output is <code>sysres.3390</code>, a CKD 3390 disk image bootable under Hercules.

== Running ==

<syntaxhighlight lang="bash-session">
$ hercules -f hercules.cnf
</syntaxhighlight>

== Further Reading ==

For loader internals, boot protocol layout, memory map, and kernel subsystems see the [https://assembler-0.github.io/ZXFoundation full documentation].

[[Category:S390]]
[[Category:Bare bones tutorials]]
[[Category:Higher Half Barebones]]
````

<a id="w005032"></a>

## W005032 — Printing To Screen

Bron-ID: W005032
Namespace: Main (0)
Revisie: 30593 | Datum: 2026-05-21T08:48:12Z
Bron: https://wiki.osdev.org/Printing_To_Screen
Revisie-URL: https://wiki.osdev.org/index.php?title=Printing_To_Screen&oldid=30593
Doorverwijzing: Drawing In a Linear Framebuffer

````text
#REDIRECT [[Drawing In a Linear Framebuffer]]
````

<a id="w005033"></a>

## W005033 — Talk:Printing To Screen

Bron-ID: W005033
Namespace: Talk (1)
Revisie: 30590 | Datum: 2026-05-21T06:16:16Z
Bron: https://wiki.osdev.org/Talk:Printing_To_Screen
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APrinting_To_Screen&oldid=30590
Doorverwijzing: Talk:VGA Text Mode

````text
#REDIRECT [[Talk:VGA Text Mode]]
````

<a id="w005034"></a>

## W005034 — Bad Practices

Bron-ID: W005034
Namespace: Main (0)
Revisie: 30891 | Datum: 2026-07-14T06:07:09Z
Bron: https://wiki.osdev.org/Bad_Practices
Revisie-URL: https://wiki.osdev.org/index.php?title=Bad_Practices&oldid=30891

````text
{{In Progress}}
Many of the articles on this site (and the Internet in general) were written many years ago, and have not been properly updated to keep up with the state of the computers. Worsened by the cross-pollination of the articles, a significant number of them now contain outdated practices and lack nuance. As a result, these articles under-inform beginners and leads to the early deaths and unnecessary rewrites of many new projects.

Some common bad practices include using [[Multiboot|Multiboot 1]] and [[VGA Text Mode]], targeting i686 instead of going straight to [[x86-64]], neglecting the importance of [[Paging|paging]] and [[Higher Half Kernel]] on modern hardware, using legacy interfaces (e.g. [[BIOS]]) and peripherals (e.g. [[PIT]]) that are absent on modern hardware, hard-coding devices at their locations in an emulator instead of [[ACPI|properly discovering them]], and so on.

While all of that might be okay when targeting retro hardware (due to limitations in device discovery) or for a quick "hello world" tutorial, it should not be used as a base for modern designs, generating endless "how do I boot my OS on my PC now" questions and problems.

== See Also ==
* [[Getting Started]]
* [[Beginner Mistakes]]
````

<a id="w005035"></a>

## W005035 — User:Definedbehavior

Bron-ID: W005035
Namespace: User (2)
Revisie: 30611 | Datum: 2026-05-23T15:13:09Z
Bron: https://wiki.osdev.org/User:Definedbehavior
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADefinedbehavior&oldid=30611

````text

````

<a id="w005036"></a>

## W005036 — Template:AdmonitionNote

Bron-ID: W005036
Namespace: Template (10)
Revisie: 30617 | Datum: 2026-05-24T14:48:26Z
Bron: https://wiki.osdev.org/Template:AdmonitionNote
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AAdmonitionNote&oldid=30617

````text
<div style="border-left: 4px solid #f0a500; background:#fff8e6; padding: 8px 12px; margin: 8px 0;">
'''Note:''' {{{text}}}
</div>
````

<a id="w005037"></a>

## W005037 — Template:AdmonitionCaution

Bron-ID: W005037
Namespace: Template (10)
Revisie: 30619 | Datum: 2026-05-24T14:58:49Z
Bron: https://wiki.osdev.org/Template:AdmonitionCaution
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AAdmonitionCaution&oldid=30619

````text
<div style="border-left: 4px solid #f66870; background:#ffbbc0; padding: 8px 12px; margin: 8px 0;">
'''Caution:''' {{{text}}}
</div>
````

<a id="w005038"></a>

## W005038 — Rust Bare Bones (UEFI)

Bron-ID: W005038
Namespace: Main (0)
Revisie: 30624 | Datum: 2026-05-26T04:04:25Z
Bron: https://wiki.osdev.org/Rust_Bare_Bones_(UEFI)
Revisie-URL: https://wiki.osdev.org/index.php?title=Rust_Bare_Bones_%28UEFI%29&oldid=30624

````text
{{BeginnersWarning}}
{{Rating|1}}
{{Template:Kernel designs}}
In this tutorial you will learn how to write a simple x86_64 uefi kernel in rust using the uefi crate and boot it.
This kernel just prints hello world to the screen and halts the cpu. The uefi crate provides a println marco that
prints to the console. The cpu is halted thanks to the cpu instructions cli and hlt. While all std features are not
available in your kernel, the core features still can be used.
== Required crates ==
* uefi crate (Version 0.37.0 used in this tutorial)
The uefi crate provides a safe interface for interaction with boot and runtime services and useful macros.
== Hello world ==
<syntaxhighlight lang="rust">
#![no_main]
#![no_std]

use uefi::{
	Status, entry, println
};
use core::{
	arch::asm,
	panic::PanicInfo
};

#[entry]
fn main() -> Status {
	uefi::helpers::init().unwrap();

	println!("Hello world");

	loop {
		unsafe {
			asm!("cli; hlt");
		}
	}
}

#[panic_handler]
fn panic<'a, 'b>(info: &'a PanicInfo<'b>) -> ! {
	println!("{}", info);
	loop {}
}
</syntaxhighlight>
The panic handler is always required. Instead of the usual main a method with the entry attribute macro shall be defined.
According to the uefi crate documentation, uefi::helpers::init() must be called before the println macro can be used.
before using the prinln macro.
== Building ==
The target x86_64-unknown-uefi must be specified in order to build an UEFI-Application.
<syntaxhighlight lang="bash">cargo build --target x86_64-unknown-uefi</syntaxhighlight>
You can alternatively specify the target in the .cargo/config.toml file.
== Booting ==
=== Booting using QEMU -kernel option ===
On option for booting is by using qemu with the -kernel option to boot the kernel. Since QEMU has a legacy BIOS by default, we need to replace it with
an UEFI-BIOS (OVMF is used in this tutorial).
<syntaxhighlight lang="bash">qemu-system-x86_64 -kernel target/x86_64-unknown-uefi/debug/example.efi -bios /usr/share/ovmf/x64/OVMF.4m.fd</syntaxhighlight>
=== Booting using a FAT image ===
{{Main|Bootable Disk}}
A other option is to make a FAT32 boot-partition and copy the kernel to the location /EFI/BOOT/BOOTX64.EFI.
<syntaxhighlight lang="bash">
fallocate disk.img -l64M
mkfs.vfat -F 32 disk.img
mmd -i disk.img ::/EFI
mmd -i disk.img ::/EFI/BOOT
mcopy -i disk.gpt target/x86_64-unknown-uefi/debug/example.efi ::/EFI/BOOT/BOOTX64.EFI
</syntaxhighlight>
After the kernel has been copied, the image can be booted with:
<syntaxhighlight lang="bash">
qemu-system-x86_64 -hda disk.img -bios /usr/share/ovmf/x64/OVMF.4m.fd
</syntaxhighlight>
=== Booting on real hardware ===
{{Warning|This section assumes the empty USB-Stick is at location /dev/sda. Triple check the path to the hardware. Writing to the wrong disk will cause severe data loss.}}
It is recommended to try out your OS in a virtual machine before trying it on real hardware.
<syntaxhighlight lang="bash">
parted /dev/sda mklabel gpt
parted /dev/sda mkpart BOOT fat32 2048s 64M
mkfs.vfat /dev/sda1 -F 32
mount /dev/sda1 /mnt
mkdir /mnt/EFI
mkdir /mnt/EFI/BOOT
cp target/x86_64-unknown-uefi/debug/example.efi /mnt/EFI/BOOT/BOOTX64.EFI
</syntaxhighlight>
After unmounting the USB-Stick you should be able to boot this kernel on a real machine.
== Exiting boot services and obtaining a memory map ==
At this point your kernel prints "Hello world" to the screen.
To avoid collisions with your drivers and UEFI-drivers you need to exit the boot services. Any boot service or allocator features are unavaiable after exiting boot services.
<syntaxhighlight lang="rust">
use uefi::boot::exit_boot_services;

let memory_map = unsafe {
	exit_boot_services(None)
};
</syntaxhighlight>
== Console ==
Since the println macro will be unavailable after exiting the boot services, you need to write a custom console,
that does not rely on boot services. To do that, you need to get the address of framebuffer.
=== Obtaining the framebuffer using GOP ===
{{Main|GOP}}
To obtain the framebuffers address, you require the GOP (Graphics Output Protocol).
This protocol allows you to read the framebuffer address, the current video mode and set it.
This method returns None if no display is available or GOP is not supported. This method needs to be called before
exiting boot services.
<syntaxhighlight lang="rust">
use uefi::{
	boot,
	proto::console::gop::GraphicsOutput
};

struct Framebuffer {
	frame_buffer: &'static mut [u32],
	width: usize,
	height: usize
}

fn framebuffer() -> Option<Framebuffer> {
	let mut gop_protocol = boot::open_protocol_exclusive::<GraphicsOutput>(
		boot::get_handle_for_protocol::<GraphicsOutput>().ok()?
	).ok()?;

	let (width, height) = gop_protocol.current_mode_info().resolution();
	let mut frame_buffer = gop_protocol.frame_buffer();

	Some(Framebuffer {
		frame_buffer: unsafe {
			core::slice::from_raw_parts_mut(
				frame_buffer.as_mut_ptr() as *mut u32,
				frame_buffer.size()
			)
		},
		width,
		height
	})
}
</syntaxhighlight>
=== Drawing characters ===
This code snippet uses a 8x16 bitmap font. Currently this implementation only shows white characters.
The iterators sometimes require to be reversed if the characters were mirrored vertically or horizontally.
It uses a u128 array to store the font.
<syntaxhighlight lang="rust">
fn draw_char(cx: usize, cy: usize, ch: char, frame_buffer: &mut Framebuffer) {
	let mut bitmask = FONT[ch as usize];
	for y in cy * 16..cy * 16 + 16 {
		for x in cx * 8..cx * 8 + 8 {
			let value = if bitmask & 0x1 == 0x1 { 0xffffffff } else { 0x0 };
			frame_buffer.frame_buffer[y * frame_buffer.width + x] = value;
			bitmask >>= 1;
		}
	}
}
</syntaxhighlight>
=== Printing strings and cursor management ===
Now to support formatted output you can implement the core::fmt::Write trait somewhere and iterate each character.
The write and writeln macro can then be used for formatted output.
<syntaxhighlight lang="rust">
use core::fmt::Write;

struct Console {
	frame_buffer: Framebuffer,
	cursor_pos_x: usize,
	cursor_pos_y: usize
}

impl Console {
	fn new(frame_buffer: Framebuffer) -> Console {
		self.frame_buffer.frame_buffer.fill(0x0);
		Console {
			frame_buffer,
			cursor_pos_x: 0,
			cursor_pos_y: 0
		}
	}

	/*
	 * Clears the console and puts the cursor in the top-left corner.
	 */
	fn clear(&mut self) {
		self.frame_buffer.frame_buffer.fill(0x0);
		self.cursor_pos_x = 0;
		self.cursor_pos_y = 0;

		draw_char(self.cursor_pos_x, self.cursor_pos_y, '_', &mut self.frame_buffer);
	}
}

impl core::fmt::Write for Console {
	fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
		for ch in s.chars() {
			if ch == '\n' {
				draw_char(self.cursor_pos_x, self.cursor_pos_y, ' ', &mut self.frame_buffer);
				self.cursor_pos_x = 0;
				self.cursor_pos_y += 1;
			} else {
				draw_char(self.cursor_pos_x, self.cursor_pos_y, ch, &mut self.frame_buffer);
				self.cursor_pos_x += 1;
			}
		}

		draw_char(self.cursor_pos_x, self.cursor_pos_y, '_', &mut self.frame_buffer);

		Ok(())
	}
}
</syntaxhighlight>
== Moving Forward ==
If your kernel is now working with a custom console, congratulations. Now there are many ways of moving forward,
like extending the console, memory management, interrupts, GDTs, User mode or disk/mouse/keyboards drivers.
=== Extending the console ===
==== Color support ====
A colored terminal is always nicer to look at instead of just a single color. To do that, you just
need to replace the 0xffffffff in the draw_char method with your RGB-color.
==== Scrolling ====
Scrolling is a common feature present in almost every console. To do that, you either clear the screen when
the cursor exceeds the last line or copy the second line to the first line of the frame_buffer, the third to the second and so on.
=== Memory manager ===
{{Main|Memory management}}
Memory management is one of the first things, that needs to be implemented in a kernel. With dynamic memory
management you can write your own implementations of important std features like Vec, Box or String.
=== Higher Half kernel ===
{{Main|Higher Half Kernel}}
If you´re making a higher half kernel, please keep in mind that the function pointers in the vtables of
dyn types point to the memory addresses, where UEFI has put your kernel in physical memory.
=== See Also ===
* [[Going Further on x86]]
* [[Interrupts]]
* [[Global Descriptor Table]]
* [[Getting to Ring 3]]
* [[Rust]]
* [[QEMU]]

[[Category: Bare bones tutorials]]
[[Category: UEFI]]
[[Category: Rust]]
````

<a id="w005039"></a>

## W005039 — Pre-kernel

Bron-ID: W005039
Namespace: Main (0)
Revisie: 30633 | Datum: 2026-05-26T19:23:45Z
Bron: https://wiki.osdev.org/Pre-kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Pre-kernel&oldid=30633

````text
{{Template:Kernel designs}}
A Pre-kernel is a small binary, that (usually) gets loaded by the bootloader, and proceeds to load the kernel to the state it expects, before handling the control to it. While at first glance, having an extra program run before the kernel might seem like an unnecessary complication, it can sometimes actually make the booting process easier (especially when using older boot protocols, like [[Multiboot]]); in addition to that, pre-kernels provide the advantages of smaller kernel size and separation of concerns (which is very useful for [[Microkernel|Microkernels]]), as well as better portability (which might be beneficial in longer run).

== Advantages and common reasons ==
=== Easier initialization of kernel ===
While many newer [[Bootloaders|boot protocols]], such as [[limine]] and [[Ultra protocol]] are more convenient, and do many essential steps to get the kernel to its higher level programming language (C or other [[Languages]]) entry point at its final [[Higher Half Kernel|higher half location]] in virtual memory, the older/simpler ones (multiboot, linux protocol, etc.) require the kernel to do it by itself.

In particular, [[Multiboot]] barely does any initialization, and (on x86) drops the kernel into a protected mode without paging (even the 64 bit kernels). When doing a [[X86-64]] [[Higher Half Kernel]], it requires the kernel to set up its initial page mappings by itself (while the CPU is in protected mode), and then to initialize the long mode by setting up the correct mode, and jumping to the higher half entry point.

While there is nothing that makes it impossible to do it in the kernel binary itself, it requires some specific linker script magic, and mixing 32 and 64 bit code, which also implies doing it in assembly; if done properly, one can expect at least a few hundreds of lines of assembly just to set up the bare minimum page tables and x86 things, and get to higher half long mode. However, if a pre-kernel is used for this, it can run in lower half, requiring very little assembly to get to C (or other higher level language), and can then more easily load the actual kernel to its final location.

=== Portability ===
If using a pre-kernel, the developer has a great opportunity to push the bootloader- and architecture-specific code from the kernel binary into the pre-kernel, abstracting it away. Later, when the kernel is ported to different architectures or (as a result, or to support more hardware in general) to different bootloader protocols, it is usually easier to adapt the small pre-kernel. Namely, if a boot protocol/method requires the kernel executable to have a different executable format (PE for UEFI, ELF for the kernel), memory layout, state on hand-off, size requirements (for embedded devices), or other changes, the small binary is usually easier to adapt, without having to the extra initializations or trampolines in the kernel itself to get it to the expected state.

=== Smaller kernel size and separation of concerns ===
Having a separate binary with just the code that's needed for the kernel's initialization implies that it can be trivially freed when the kernel is booted, implicitly removing all of that from the kernel binary, and resulting in a slightly smaller memory footprint (probably even compared to the kernel discarding some of its sections after initialization). In addition, since using a pre-kernel can be thought of as having a small "personal" bootloader, the kernel also becomes more flexible with what it can do, and the assumptions/requirements imposed by the boot protocol (for example, of physical memory layout, execution formats, hardware initializations, and so on).

This is especially beneficial for microkernels (or other models), which run most of their services and kernel tasks in userspace to achieve a purer design. As a concrete example (which doesn't have to be followed because of the problems that it brings), a more radical microkernel may choose to do almost all memory management (or anything else) in userspace; with a pre-kernel, it can do most of the initialization, before handing the control to the kernel, which will then rely on a userspace server to carry with memory management tasks, being completely oblivious to how it is done, following the pure microkernel philosophy. 

[[Category:Kernel]]
````

<a id="w005040"></a>

## W005040 — Prekernel

Bron-ID: W005040
Namespace: Main (0)
Revisie: 30635 | Datum: 2026-05-26T19:26:19Z
Bron: https://wiki.osdev.org/Prekernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Prekernel&oldid=30635
Doorverwijzing: Pre-kernel

````text
#REDIRECT [[Pre-kernel]]
````

<a id="w005041"></a>

## W005041 — Higher Half x86 Bare Bones

Bron-ID: W005041
Namespace: Main (0)
Revisie: 30654 | Datum: 2026-05-28T12:50:08Z
Bron: https://wiki.osdev.org/Higher_Half_x86_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Higher_Half_x86_Bare_Bones&oldid=30654
Doorverwijzing: Multiboot 1 Higher Half x86 Bare Bones

````text
#REDIRECT [[Multiboot 1 Higher Half x86 Bare Bones]]
````

<a id="w005042"></a>

## W005042 — EBCDIC

Bron-ID: W005042
Namespace: Main (0)
Revisie: 30670 | Datum: 2026-05-29T09:49:19Z
Bron: https://wiki.osdev.org/EBCDIC
Revisie-URL: https://wiki.osdev.org/index.php?title=EBCDIC&oldid=30670

````text
= EBCDIC =

'''EBCDIC''' ('''Extended Binary Coded Decimal Interchange Code''', pronounced ''EBB-see-dick'' or ''EBB-sih-dick'') is an 8-bit character encoding standard developed by IBM, introduced in 1964 alongside the [[IBM System/360]] — the architectural ancestor of the modern [[s390x]] (IBM Z). Unlike [[ASCII]], which was designed for open interchange, EBCDIC was designed for IBM's punched-card based batch-processing ecosystem and remains the '''native character encoding of IBM Z mainframes''' to this day.

If you are writing an operating system or bootloader for [[s390x]], you '''will''' encounter EBCDIC. Firmware interfaces, hardware console I/O, boot records, and low-level IBM interfaces all speak EBCDIC by default.

__TOC__

== History and Background ==

=== Origins ===

EBCDIC evolved from '''BCD''' (Binary Coded Decimal) and '''BCDIC''' (BCD Interchange Code), both of which were used on IBM punched-card equipment such as the IBM 1401. When IBM introduced the System/360 in 1964 — the first family of computers designed as a unified, upward-compatible architecture — they standardized on EBCDIC as the character encoding across the entire line.

The design choices in EBCDIC reflect its punched-card heritage:
* Digits and letters were mapped to correspond to specific punched-card column patterns.
* Uppercase letters are '''not''' contiguous in the code space (a common source of bugs).
* The code includes a rich set of control characters inherited from card and line-printer control.

=== EBCDIC vs ASCII ===

ASCII was standardized by ANSI in 1963, one year before EBCDIC's introduction. Both are 8-bit encodings (ASCII uses 7 bits; the 8th bit was later used for extensions), but they are '''entirely incompatible''' in layout. A raw byte stream of EBCDIC text is meaningless if interpreted as ASCII, and vice versa.

{| class="wikitable"
|+ Key Differences
! Feature !! ASCII !! EBCDIC
|-
| Bit width || 7-bit (8-bit extended) || 8-bit
|-
| Letter 'A' || <code>0x41</code> || <code>0xC1</code>
|-
| Digit '0' || <code>0x30</code> || <code>0xF0</code>
|-
| Alphabet contiguous? || Yes || '''No'''
|-
| Lowercase before uppercase? || Yes (a=0x61, A=0x41) || No (a=0x81, A=0xC1)
|-
| Null || <code>0x00</code> || <code>0x00</code> (same)
|-
| Space || <code>0x20</code> || <code>0x40</code>
|-
| Newline (LF) || <code>0x0A</code> || <code>0x25</code>
|-
| Carriage Return || <code>0x0D</code> || <code>0x0D</code> (same)
|-
| DEL || <code>0x7F</code> || <code>0x07</code> (BEL in ASCII!)
|}

The non-contiguous letter layout is the most notorious EBCDIC quirk. In ASCII, you can test <code>c >= 'A' && c <= 'Z'</code> to check for uppercase letters. In EBCDIC, this range contains non-letter characters (specifically, several punctuation marks lie between the uppercase letter groups). '''Never assume contiguity of the alphabet when writing EBCDIC-aware code.'''

=== Variants ===

There is no single EBCDIC. IBM defined dozens of '''code pages''' — regional and application-specific variants — each with different assignments for characters outside the invariant core. The most commonly encountered variants in practice are:

{| class="wikitable"
! Code Page !! IBM Name !! Common Use
|-
| '''EBCDIC-037''' || CP037 || United States, Canada (most common in North American mainframe shops)
|-
| EBCDIC-273 || CP273 || Germany, Austria
|-
| EBCDIC-500 || CP500 || International / Belgium / Switzerland
|-
| EBCDIC-1047 || CP1047 || Open Systems (z/OS UNIX, Linux on Z) — closest to ASCII mappings
|-
| EBCDIC-1140 || CP1140 || CP037 + Euro sign
|-
| EBCDIC-1148 || CP1148 || CP500 + Euro sign
|}

'''CP1047''' deserves special attention for OS developers: it is the code page used by z/OS UNIX services and [[Linux on Z]] for interoperability, and is the most "ASCII-friendly" EBCDIC variant. If you are writing a kernel that must interoperate with z/OS or handle ELF binaries compiled for s390x Linux, CP1047 is your primary concern.

== Encoding Layout ==

=== Structure of the Code Space ===

EBCDIC divides the 256 code points into several zones. The high nibble (bits 4–7) loosely defines the character class:

{| class="wikitable"
! Range !! Character Class
|-
| <code>0x00–0x3F</code> || Control characters
|-
| <code>0x40</code> || Space
|-
| <code>0x41–0x7F</code> || Special/punctuation characters
|-
| <code>0x80–0x8F</code> || Lowercase letters a–i (with gaps)
|-
| <code>0x90</code> || (unassigned / varies by code page)
|-
| <code>0x91–0x99</code> || Lowercase letters j–r (with gaps)
|-
| <code>0xA0</code> || (unassigned / varies by code page)
|-
| <code>0xA1–0xA9</code> || Lowercase letters s–z + some punctuation
|-
| <code>0xC1–0xC9</code> || Uppercase letters A–I
|-
| <code>0xD1–0xD9</code> || Uppercase letters J–R
|-
| <code>0xE2–0xE9</code> || Uppercase letters S–Z (with a gap at 0xE0–0xE1)
|-
| <code>0xF0–0xF9</code> || Digits 0–9
|}

Note the three separate ranges for lowercase and uppercase letters, each with gaps. The gaps contain punctuation characters that differ between code page variants. This layout directly mirrors the punched-card column encodings from which EBCDIC descended.

=== Critical Characters Quick Reference (CP037 / CP1047) ===

The table below lists the most important characters for an OS developer. Values marked with * differ between CP037 and CP1047.

{| class="wikitable sortable"
! Character !! CP037 (Hex) !! CP1047 (Hex) !! ASCII (Hex)
|-
| NUL || 00 || 00 || 00
|-
| BEL || 2F || 2F || 07
|-
| BS  || 16 || 16 || 08
|-
| HT  || 05 || 05 || 09
|-
| LF  || 25 || 25 || 0A
|-
| VT  || 0B || 0B || 0B
|-
| FF  || 0C || 0C || 0C
|-
| CR  || 0D || 0D || 0D
|-
| SO  || 0E || 0E || 0E
|-
| SI  || 0F || 0F || 0F
|-
| ESC || 27 || 27 || 1B
|-
| Space || 40 || 40 || 20
|-
| <code>[</code> || BA* || AD || 5B
|-
| <code>]</code> || BB* || BD || 5D
|-
| <code>{</code> || C0* || C0 || 7B
|-
| <code>}</code> || D0* || D0 || 7D
|-
| <code>\</code> || E0* || E0 || 5C
|-
| <code>^</code> || B0* || 5F || 5E
|-
| <code>~</code> || A1* || A1 || 7E
|-
| <code>!</code> || 5A || 5A || 21
|-
| <code>"</code> || 7F || 7F || 22
|-
| <code>#</code> || 7B || 7B || 23
|-
| <code>$</code> || 5B || 5B || 24
|-
| <code>%</code> || 6C || 6C || 25
|-
| <code>&</code> || 50 || 50 || 26
|-
| <code>'</code> || 7D || 7D || 27
|-
| <code>(</code> || 4D || 4D || 28
|-
| <code>)</code> || 5D || 5D || 29
|-
| <code>*</code> || 5C || 5C || 2A
|-
| <code>+</code> || 4E || 4E || 2B
|-
| <code>,</code> || 6B || 6B || 2C
|-
| <code>-</code> || 60 || 60 || 2D
|-
| <code>.</code> || 4B || 4B || 2E
|-
| <code>/</code> || 61 || 61 || 2F
|-
| <code>0</code>–<code>9</code> || F0–F9 || F0–F9 || 30–39
|-
| <code>:</code> || 7A || 7A || 3A
|-
| <code>;</code> || 5E || 5E || 3B
|-
| <code><</code> || 4C || 4C || 3C
|-
| <code>=</code> || 7E || 7E || 3D
|-
| <code>></code> || 6E || 6E || 3E
|-
| <code>?</code> || 6F || 6F || 3F
|-
| <code>@</code> || 7C || 7C || 40
|-
| <code>A</code>–<code>I</code> || C1–C9 || C1–C9 || 41–49
|-
| <code>J</code>–<code>R</code> || D1–D9 || D1–D9 || 4A–52
|-
| <code>S</code>–<code>Z</code> || E2–E9 || E2–E9 || 53–5A
|-
| <code>_</code> || 6D || 6D || 5F
|-
| <code>`</code> || 79 || 79 || 60
|-
| <code>a</code>–<code>i</code> || 81–89 || 81–89 || 61–69
|-
| <code>j</code>–<code>r</code> || 91–99 || 91–99 || 6A–72
|-
| <code>s</code>–<code>z</code> || A2–A9 || A2–A9 || 73–7A
|-
| DEL || 07 || 07 || 7F
|}

== Conversion ==

=== Approach 1: Lookup Table ===

The simplest and fastest approach is a 256-entry lookup table. Since the mapping is not mathematically regular, a table is both the most correct and most performant solution for bulk conversion.

<syntaxhighlight lang="c">
/* 
 * ebcdic_to_ascii[n] gives the ASCII byte for EBCDIC byte n.
 * Code page: IBM-1047 (recommended for OS/Linux interoperability).
 * Unmappable bytes are replaced with '?' (0x3F).
 */
static const uint8_t ebcdic1047_to_ascii[256] = {
    /* 0x00 */ 0x00, 0x01, 0x02, 0x03, 0x9C, 0x09, 0x86, 0x7F,
    /* 0x08 */ 0x97, 0x8D, 0x8E, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    /* 0x10 */ 0x10, 0x11, 0x12, 0x13, 0x9D, 0x0A, 0x08, 0x87,
    /* 0x18 */ 0x18, 0x19, 0x92, 0x8F, 0x1C, 0x1D, 0x1E, 0x1F,
    /* 0x20 */ 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x17, 0x1B,
    /* 0x28 */ 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x05, 0x06, 0x07,
    /* 0x30 */ 0x90, 0x91, 0x16, 0x93, 0x94, 0x95, 0x96, 0x04,
    /* 0x38 */ 0x98, 0x99, 0x9A, 0x9B, 0x14, 0x15, 0x9E, 0x1A,
    /* 0x40 */ 0x20, 0xA0, 0xE2, 0xE4, 0xE0, 0xE1, 0xE3, 0xE5,
    /* 0x48 */ 0xE7, 0xF1, 0xA2, 0x2E, 0x3C, 0x28, 0x2B, 0x7C,
    /* 0x50 */ 0x26, 0xE9, 0xEA, 0xEB, 0xE8, 0xED, 0xEE, 0xEF,
    /* 0x58 */ 0xEC, 0xDF, 0x21, 0x24, 0x2A, 0x29, 0x3B, 0x5E,
    /* 0x60 */ 0x2D, 0x2F, 0xC2, 0xC4, 0xC0, 0xC1, 0xC3, 0xC5,
    /* 0x68 */ 0xC7, 0xD1, 0xF6, 0x2C, 0x25, 0x5F, 0x3E, 0x3F,
    /* 0x70 */ 0xF8, 0xC9, 0xCA, 0xCB, 0xC8, 0xCD, 0xCE, 0xCF,
    /* 0x78 */ 0xCC, 0x60, 0x3A, 0x23, 0x40, 0x27, 0x3D, 0x22,
    /* 0x80 */ 0xD8, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67,
    /* 0x88 */ 0x68, 0x69, 0xAB, 0xBB, 0xF0, 0xFD, 0xFE, 0xB1,
    /* 0x90 */ 0xB0, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70,
    /* 0x98 */ 0x71, 0x72, 0xAA, 0xBA, 0xE6, 0xB8, 0xC6, 0xA4,
    /* 0xA0 */ 0xB5, 0x7E, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78,
    /* 0xA8 */ 0x79, 0x7A, 0xA1, 0xBF, 0xD0, 0x5B, 0xDE, 0xAE,
    /* 0xB0 */ 0xAC, 0xA3, 0xA5, 0xB7, 0xA9, 0xA7, 0xB6, 0xBC,
    /* 0xB8 */ 0xBD, 0xBE, 0xDD, 0xA8, 0xAF, 0x5D, 0xB4, 0xD7,
    /* 0xC0 */ 0x7B, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47,
    /* 0xC8 */ 0x48, 0x49, 0xAD, 0xF4, 0xF6, 0xF2, 0xF3, 0xF1, /* note: some entries locale-specific */
    /* 0xD0 */ 0x7D, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50,
    /* 0xD8 */ 0x51, 0x52, 0xB9, 0xFB, 0xFC, 0xF9, 0xFA, 0xFF,
    /* 0xE0 */ 0x5C, 0xF7, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58,
    /* 0xE8 */ 0x59, 0x5A, 0xB2, 0xD4, 0xD6, 0xD2, 0xD3, 0xD1,
    /* 0xF0 */ 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
    /* 0xF8 */ 0x38, 0x39, 0xB3, 0xDB, 0xDC, 0xD9, 0xDA, 0x9F,
};

void ebcdic_to_ascii_buf(uint8_t *dst, const uint8_t *src, size_t len)
{
    for (size_t i = 0; i < len; i++)
        dst[i] = ebcdic1047_to_ascii[src[i]];
}
</syntaxhighlight>

=== Approach 2: iconv ===

If your OS provides a libc (or you are writing a userland tool rather than kernel code), <code>iconv(3)</code> supports EBCDIC code pages. For example:

<syntaxhighlight lang="c">
iconv_t cd = iconv_open("ASCII", "EBCDIC-CP-US");
/* or for CP1047: */
iconv_t cd = iconv_open("UTF-8", "IBM-1047");
</syntaxhighlight>

This is '''not''' suitable for kernel or early-boot code, but is appropriate for userland utilities.

=== Approach 3: s390x TR Instruction ===

The s390x <code>TR</code> (Translate) instruction was literally designed for EBCDIC translation. It translates each byte of a source string by using it as an index into a 256-byte translation table, writing results to a destination buffer. This is the hardware-native approach and is extremely fast.

<syntaxhighlight lang="asm">
/* Translate EBCDIC → ASCII using the TR instruction.
 * R2 = address of source buffer
 * R3 = length - 1 (TR length operand is len-1)
 * R4 = address of 256-byte translation table
 */
    LA    %r1, dest_buffer     /* destination address */
    MVC   0(256,%r1), 0(%r2)   /* copy source to dest first (TR is in-place) */
    TR    0(256,%r1), 0(%r4)   /* translate in place using table at R4 */
</syntaxhighlight>

Note: <code>TR</code> operates in-place on the destination, so copy the source first if you need a separate output buffer. The instruction takes a maximum of 256 bytes per invocation; loop for longer strings. The <code>TRT</code> (Translate and Test) instruction can scan for specific EBCDIC characters, useful for parsing.

== Control Characters ==

EBCDIC control characters are a superset of the ASCII control characters, with some significant differences. The most important for terminal/console handling:

{| class="wikitable"
! EBCDIC (Hex) !! Mnemonic !! ASCII Equiv !! Notes
|-
| 00 || NUL || NUL || Same
|-
| 05 || HT || HT (0x09) || Horizontal Tab
|-
| 0D || CR || CR (0x0D) || Carriage Return — same value!
|-
| 15 || NL || LF (0x0A) || '''New Line''' — EBCDIC's line terminator; maps to ASCII LF
|-
| 25 || LF || LF (0x0A) || Line Feed — also used; some docs say NL is the "true" newline
|-
| 37 || EOT || EOT (0x04) || End of Transmission
|-
| 3C || DC4 || DC4 (0x14) || Device Control 4
|-
| 40 || SP || SP (0x20) || Space — '''0x40, not 0x20'''!
|-
| 0F || SI || SI (0x0F) || Shift In
|-
| 0E || SO || SO (0x0E) || Shift Out
|}

The EBCDIC '''New Line''' (NL, 0x15) is particularly important: z/OS text files use <code>0x15</code> as the line terminator, not <code>0x25</code>. When reading z/OS text datasets, translate <code>0x15</code> → <code>0x0A</code> (LF). This is handled automatically by CP1047 ↔ ISO-8859-1 conversion tables in most iconv implementations.

=== Detecting EBCDIC at Runtime ===

If your code might run on both EBCDIC and ASCII platforms (e.g., a cross-platform bootloader stub), you can detect the encoding at runtime:

<syntaxhighlight lang="c">
static inline int running_on_ebcdic(void)
{
    /* 'A' is 0x41 in ASCII, 0xC1 in EBCDIC */
    return ('A' == 0xC1);
}
</syntaxhighlight>

=== Useful EBCDIC Properties ===

Despite its quirks, EBCDIC has some convenient properties the hardware exploits:

* '''Digit value extraction''': For EBCDIC digit characters (0xF0–0xF9), <code>digit_value = ebcdic_byte & 0x0F</code>.
* '''Case conversion''': For pure alphabetic bytes (verifying they are in the correct ranges first), <code>ebcdic_upper = ebcdic_lower & 0xBF</code> and <code>ebcdic_lower = ebcdic_upper | 0x40</code> — but only if the byte is confirmed to be an alpha character, because the same bits mean different things for non-alpha bytes.
* '''Alphabetic check (hardware)''': The s390x <code>TRT</code> instruction combined with a pre-built classification table enables very fast character classification.

== Code Page Tables ==

=== Full CP1047 Table ===

The following table gives the complete CP1047 (IBM-1047) EBCDIC-to-Unicode mapping for the printable range. The column header is the high nibble and the row header is the low nibble.

''(For the full 256-entry mapping suitable for inclusion in kernel source, see the [[#Approach 1: Lookup Table|lookup table]] in the Conversion section above, or the reference at [https://www.ibm.com/docs/en/zos/latest?topic=guide-ebcdic-unicode-conversion-tables IBM's EBCDIC/Unicode Conversion Tables].)''

== Standards and References ==

* '''IBM z/Architecture Principles of Operation''' (SA22-7832) — The authoritative reference for all s390x architecture details, including character encoding used in firmware interfaces.
* '''IBM z/OS MVS Programming: Authorized Assembler Services Guide''' — Details on SCLP and console interfaces.
* [https://www.ibm.com/docs/en/zos/latest?topic=data-ebcdic-code-pages IBM Documentation: EBCDIC Code Pages] — Official IBM listing of all EBCDIC code pages.
* '''ANSI X3.4-1968''' — Original ASCII standard, useful for comparison.
* '''RFC 877''' — Notes on EBCDIC in network contexts.
* [https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/ Unicode.org IBM Code Page Mappings] — Machine-readable CP→Unicode mapping files.
* [https://pubs.opengroup.org/onlinepubs/9699919799/ POSIX.1-2017] — Defines locale and iconv behavior, relevant for userland tools.

== See Also ==

* [[ASCII]]
* [[Unicode]]
* [[s390x Architecture]]
* [[IBM System/360]]
* [[SCLP (Service Call Logical Processor)]]
* [[IPL (Initial Program Load)]]
* [[z/VM]]
* [[Channel I/O]]
* [[DASD]]
* [[Packed Decimal]]

[[Category:Character Encoding]]
[[Category:s390]]
````

<a id="w005043"></a>

## W005043 — Category:Character Encoding

Bron-ID: W005043
Namespace: Category (14)
Revisie: 30675 | Datum: 2026-05-29T16:49:53Z
Bron: https://wiki.osdev.org/Category:Character_Encoding
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ACharacter_Encoding&oldid=30675

````text
= Character Encoding =
[[Category:Standards]]
````

<a id="w005044"></a>

## W005044 — Unicode

Bron-ID: W005044
Namespace: Main (0)
Revisie: 30674 | Datum: 2026-05-29T13:53:50Z
Bron: https://wiki.osdev.org/Unicode
Revisie-URL: https://wiki.osdev.org/index.php?title=Unicode&oldid=30674

````text
{{In Progress}}

'''Unicode''' is the dominant text encoding standard for representing international strings. Each character from an alphabet is assigned a ''Code Point'', from the range U+0000 to U+10FFFF. This space needs 25 bits to represent it.

The Unicode consortium defines multiple character encodings for Unicode code points, the simplest of which, [[#UTF-32|UTF-32]], is simply the code point padded to 32 bits. [[#UTF-8|UTF-8]], by contrast, is compatible with the base [[ASCII]] encoding (not extended ASCII), as ASCII is a 7-bit encoding, so UTF-8 supports a 1:1 mapping. The unused high bit in ASCII codes denotes a multi-byte encoding in UTF-8.

== UTF-32 ==
Also sometimes called UCS-4. This is the only fixed-width Unicode encoding, and can be represented as <tt>char32_t</tt> since C11.

The Unicode space is broken up into 17 different ''planes''.

=== Planes ===
{| class="wikitable"
|+Unicode Planes
!Plane Number
!0
!1
!2
!3
!4-13
!14
!15
!16
|-
|Code Points
|U+0000-U+FFFF
|U+10000-U+1FFFF
|U+20000-U+2FFFF
|U+30000-U+3FFFF
|U+40000-U+DFFFF
|U+E0000-U+EFFFF
| colspan="2" |U+F0000-U+10FFFF
|-
|Reference
|[[Unicode#BMP|BMP]]
|[[Unicode#SMP|SMP]]
|[[Unicode#SIP|SIP]]
|[[Unicode#TIP|TIP]]
| rowspan="2" |''unassigned''
|[[Unicode#SSP|SSP]]
| colspan="2" |[[Unicode#SPUA|SPUA]]
|-
|Name
|Basic Multilingual
|Supplementary Multilingual
|Supplementary Ideographic
|Tertiary Ideographic
|Supplementary Special-purpose
| colspan="2" |Supplementary Private Use Area
|}

==== BMP ====
The '''Basic Multilingual Plane.'''

The characters of most languages are in this plane. There are also two main reserved areas - UTF-16 surrogates, U+D800-U+DFFF, and the primary ''Private Use Area''  from U+E000-U+F8FF.

'''It is invalid to encode a surrogate in any Unicode encoding.''' They exist entirely to support UTF-16.

==== SMP ====
The '''Supplementary Multilingual Plane.'''

This plane largely contains characters from historic scripts. For instance, Phoenician, which is also intended for representing Early Aramaic and Paleo-Hebrew. 𐤔𐤋𐤅𐤌. Or Egyptian Hieroglyphs. 𓀁

Also in this plane are a generous helping of mathematical symbols. ⨌

==== SIP ====
The '''Supplementary Ideographic Plane.'''

This plane is almost entirely allocated, solely to CJK unified ideographs.

==== TIP ====
The '''Tertiary Ideographic Plane.'''

This plane is only partially allocated, solely to CJK unified ideographs.

==== SSP ====
The '''Supplementary Special-purpose Plane.'''

A largely unused plane, this contains some tags and variation selectors.

==== SPUA ====
The '''Supplementary Private Use Area.''' This can be used by e.g. special fonts.

== UTF-16 ==
UTF-16 is a 16 bit, variable-length encoding. It encodes the [[Unicode#BMP|Basic Multilingual Plane]] identically to UCS-2, directly representing it as one 16 bit value.

On decoding, if a character is in the UTF-16 surrogate region, then per the specification it must be part of a '''surrogate pair''', as these values are invalid to encode alone.

'''Note that the byte encoding of this depends on endianness.''' To assist, the text can be preceeded with a byte-order mark, U+FEFF. U+FFFE is a reserved, invalid, value.

Surrogate pairs are used to encode the other unicode planes as follows:

====== 2-byte encoding ======
U+0000-U+FFFF - xxxxxxxx_xxxxxxxx

====== 4-byte encoding ======
U+010000-U+10FFFF - 110110xx_xxxxxxxx 110111xx_xxxxxxx (0xD800+xh, 0xDC00+xl)

====== Sample conversion ======
<syntaxhighlight lang="c++">
void encode(char32_t codepoint, std::span<char16_t>& result)
{
	size_t bytesWritten;
	if(codepoint <= 0xFFFF)
	{
		result[0] = static_cast<char16_t>(codepoint);
		bytesWritten = 1;
	}
	else
	{
		if(codepoint > 0x10FFFF) throw EncodingException{};
		//Subtract 2^16, implied as it's a surrogate pair now
		codepoint -= 0x10000U;
		//Split into two characters
		char16_t high = 0xD800 + (codepoint >> 10);
		char16_t low = 0xDC00 + (codepoint & 0x3FF);
		
		result[0] = high;
		result[1] = low;
		bytesWritten = 2;
	}
	result = result.subspan(bytesWritten);
}

char32_t decode(std::span<const char16_t>& source)
{
		size_t bytesRead;
		char32_t result;
		char16_t first = source[0];
		if(first < 0xD800 || first > 0xDFFF)
		{
			result = static_cast<char32_t>(first);
			bytesRead = 1;
		}
		else
		{
			char16_t second = source[1];
			
			if(first > 0xDBFF) throw EncodingException{};
			if(second < 0xDC00) throw EncodingException{};
			if(second > 0xDFFF) throw EncodingException{};
			
			uint16_t high = first - 0xD800;
			uint16_t low = second - 0xDC00;
			
			result = static_cast<char32_t>(0x10000U + (high << 10) | low);	
			
			bytesRead = 2;
		}
		source = source.subspan(bytesRead);
		return result;
}
</syntaxhighlight>

==UTF-8 ==
UTF-8 is an 8 bit, variable length encoding. It is in effect a superset of 7-bit [[ASCII]], where ASCII characters directly convert to an 8 bit code unit with a leading zero. A leading 1 denotes a multi-byte encoding.

====== 1-byte encoding ======
U+0000-U+007F - 0xxxxxxx

====== 2-byte encoding ======
U+0080-U+07FF - 110xxxxx 10xxxxxx

====== 3-byte encoding ======
U+0800-U+FFFF - 1110xxxx 10xxxxxx 10xxxxxx

====== 4-byte encoding ======
U+010000-U+10FFFF - 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx

====== Overlong encoding ======
Note that by appending leading zeros, a character can be encoded in a longer encoding than necessary. This can bypass validations that check for characters (e.g. /), and are a security problem. They should be considered an error.

====== Sample conversion ======
<syntaxhighlight lang="c++">
//ENCODING
void encode(char32_t codepoint, std::span<char8_t>& result)
{
	size_t bytesWritten;
	if(codepoint <= 0x7F)
	{
		result[0] = static_cast<char8_t>(codepoint);
		bytesWritten = 1;
	}
	else if(codepoint < 0x7FF)
	{
		result[0] = static_cast<char8_t>(0xC0 + (codepoint >> 6));
		result[1] = static_cast<char8_t>(0x80 + (codepoint & 0x3F));
		bytesWritten = 2;
	}
	else if(codepoint < 0xFFFF)
	{
		result[0] = static_cast<char8_t>(0xE0 + (codepoint >> 12));
		result[1] = static_cast<char8_t>(0x80 + ((codepoint >> 6) & 0x3F));
		result[2] = static_cast<char8_t>(0x80 + (codepoint & 0x3F));
		bytesWritten = 3;
	}
	else if(codepoint < 0x10FFFF)
	{
		result[0] = static_cast<char8_t>(0xF0 + (codepoint >> 18));
		result[1] = static_cast<char8_t>(0x80 + ((codepoint >> 12) & 0x3F));
		result[2] = static_cast<char8_t>(0x80 + ((codepoint >> 6) & 0x3F));
		result[3] = static_cast<char8_t>(0x80 + (codepoint & 0x3F));
		bytesWritten = 4;
	}
	else 
	{
		throw EncodingException{};
	}
	result = result.subspan(bytesWritten);
}

//DECODING
static size_t decode_starting_byte(char8_t start, char32_t& result)
{
		if(first < 0x80)
		{
			result = static_cast<char32_t>(start);
			return 1;
		}
		else if(first < 0xC0)
		{
			//Expected UTF-8 starting byte, not continuation byte
			throw EncodingException{};
		}
		else if(first < 0xE0)
		{
			result = start & 0x1F;
			return 2;
		}
		else if(first < 0xF0)
		{
			result = start & 0x0F;
			return 3;
		}
		else if(first < 0xF8)
		{
			result = start & 0x07;
			return 4;

		}
		else
		{
			//Also invalid
			throw EncodingException{};
		}
}

static void decode_continuation(char8_t continuationByte, char32_t& result)
{
	if((continuationByte & 0xC0) != 0x80) 
		throw EncodingException{};	//Expected a continuation byte, but not
	
	result = (result << 6) | (continuationByte & 0x3F);
}

char32_t decode(std::span<const char8_t>& source)
{
		if(source.empty()) 
			throw EncodingException{};
		
		char32_t result;
		size_t bytesRead = decode_starting_byte(source[0], result);
		
		if(bytesRead > source.size()) 
			throw EncodingException{};
		
		switch(bytesRead)
		{
			case 1:
				break;
			case 2:
				decode_continuation(source[1], result);
				if(result < 0x80) throw EncodingException{};	//Overlong encoding
				break;
			case 3:
				decode_continuation(source[1], result);
				decode_continuation(source[2], result);
				if(result < 0x800) throw EncodingException{};	//Overlong encoding
				break;
			case 4:
				decode_continuation(source[1], result);
				decode_continuation(source[2], result);
				decode_continuation(source[3], result);
				if(result < 0x010000) throw EncodingException{};	//Overlong encoding
				break;
			
		}
		if(result > 0x10FFFF) throw EncodingException{};
		
		source = source.subspan(bytesRead);
		return result;
}
</syntaxhighlight>

== Other Encodings ==
=== UCS-2 ===
UCS-2 is an obsolete predecessor of UTF-16. It uses a fixed 16-bit character width, from before it was obvious that more than 2^16 code points were needed.

=== UTF-EBCDIC ===
This is a modified version of [[#UTF-8|UTF-8]], designed to be compatible with the [[EBCDIC]] encoding used on IBM Z mainframes.

== See Also ==
=== External Links ===
* https://www.unicode.org/charts/ - Character code charts for the entirety of Unicode

[[Category:Standards]]
[[Category:Character Encoding]]
````

<a id="w005045"></a>

## W005045 — Category:Higher Half Barebones

Bron-ID: W005045
Namespace: Category (14)
Revisie: 30680 | Datum: 2026-06-03T11:09:47Z
Bron: https://wiki.osdev.org/Category:Higher_Half_Barebones
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AHigher_Half_Barebones&oldid=30680

````text
This category contains the [[Higher Half Kernel]] tutorials.

[[Category:Bare bones tutorials]]
````

<a id="w005046"></a>

## W005046 — OpenGL

Bron-ID: W005046
Namespace: Main (0)
Revisie: 30702 | Datum: 2026-06-09T13:25:09Z
Bron: https://wiki.osdev.org/OpenGL
Revisie-URL: https://wiki.osdev.org/index.php?title=OpenGL&oldid=30702

````text
{{stub}}
'''OpenGL''' is a graphics standard released in mid 1992. OpenGL is simpler than the popular [[Vulkan]] API, but consequently will on average perform slightly worse than Vulkan. The latest version of OpenGL is version 4.6, released in 2017. Since version 4.6, active development of OpenGL has been dropped in favor of Vulkan.


OpenGL is an API standard, and not a an actual library in it of itself. OpenGL was designed for most if not all of its API to be hardware accelerated, but OpenGL rendering can also be implemented entirely on the CPU. The OpenGL core API is smaller than Vulkan, but OpenGL also as plenty of extensions that can add more functionality.

== See Also ==

=== External Links ===

* [https://www.opengl.org OpenGL Website]
````

<a id="w005047"></a>

## W005047 — User:MrOlsen7905/uBixOS

Bron-ID: W005047
Namespace: User (2)
Revisie: 30700 | Datum: 2026-06-08T16:29:29Z
Bron: https://wiki.osdev.org/User:MrOlsen7905/uBixOS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMrOlsen7905%2FuBixOS&oldid=30700

````text
'''uBixOS''' is a hobby/research operating system for the x86 (i386) platform,
  developed from scratch since 2002. It boots via GRUB2 using the multiboot
  protocol, runs on bare metal or under QEMU, and implements a monolithic kernel
  with a modular subsystem layout, a FreeBSD-derived POSIX userland, and its own
  native copy-on-write filesystem, '''UbixFS'''. A second architecture target,
  '''AArch64''' (ARM64), is currently being brought up.

  == Features ==

  * '''Kernel:''' Preemptive multitasking with an O(1) scheduler (QoS classes,
    I/O boost, starvation aging); SMP support; x86 protected mode (ring 0/3).
  * '''Memory:''' Paged virtual memory with copy-on-write forking and demand-shared
    read-only library pages.
  * '''Processes & threads:''' fork/execve, POSIX signals (sigaction/sigprocmask/
    sigsuspend, SIGSTOP/SIGCONT, ZOMBIE/wait4), kernel threads (clone/rfork, TLS,
    futexes) backing musl pthreads.
  * '''Filesystems:''' UbixFS (native, copy-on-write, pooled — see below), UFS,
    FAT16/32, DevFS, procfs — all behind a VFS abstraction layer.
  * '''Networking:''' lwIP 2.0.3 TCP/IP stack; NE2000, Lance (PCNET), and Intel
    e1000 NIC drivers; DHCP; a TLS/HTTPS client over a BearSSL port.
  * '''IPC:''' Custom MPI message passing, POSIX pipes, semaphores.
  * '''Drivers:''' AT keyboard, PIT, PIC (8259), floppy, RS-232 serial, PCI IDE
    hard disk, PS/2 mouse, USB mass storage.
  * '''GUI:''' Composited window system (''views'') with server-side decorations,
    a taskbar, and a VT100 terminal — apps render with the objGFX library
    (TrueType antialiased fonts) into shared-memory buffers.
  * '''Userland:''' 30+ utilities including tcsh 6.24.16; FreeBSD-derived libc
    (musl-linked runtime); a C++ runtime (libc++ subset); a port of the NetSurf
    web browser (HTML/CSS render, JavaScript via Duktape, images, input).
  * '''ELF loader:''' Loads and runs standard i386 ELF binaries with a runtime
    dynamic linker.

  == UbixFS ==

  uBixOS's native filesystem is a "lite ZFS": a copy-on-write storage '''pool'''
  with '''datasets''' (a POSIX filesystem and a raw block volume for swap),
  per-block checksums (Fletcher4), an uberblock/transaction-group commit model
  (crash-consistent with no journal), POSIX permissions/owners/times, symlinks,
  variable record sizes, and a pluggable compression slot. Snapshots are deferred
  but the on-disk format reserves the hooks for them. The engine is portable C,
  shared between an unprivileged host image tool and the kernel driver.

  == Architectures ==

  * '''i386''' (x86, 32-bit) — the primary, mature target. Boots GRUB2 → kernel →
    init → login shell; runs the full userland and GUI under QEMU and on bare metal.
  * '''AArch64''' (ARM64) — port in progress. Boots headless under QEMU ''virt''
    (HVF-accelerated on Apple Silicon) through to an interactive login and shell,
    with a musl userland over the FreeBSD-style syscall ABI. The source tree is
    organized so architecture-specific code lives under <code>arch/&lt;arch&gt;/</code>
    while the bulk of the kernel stays architecture-agnostic. Real-silicon
    bring-up (Allwinner H618 / RK3566-class boards) is the next milestone.

  == Building ==

  The project uses BSD make (<code>bmake</code>). On macOS the build auto-detects
  an <code>x86_64-elf-gcc -m32</code> cross toolchain; on FreeBSD the host
  toolchain is used directly.

  <source lang="sh">
  bmake                    # build kernel + world (i386)
  bmake image              # build a bootable FAT32 disk image
  bmake run                # launch in QEMU
  bmake kernel TARGET=aarch64      # build the AArch64 kernel
  bmake run-debug TARGET=aarch64   # boot it headless in QEMU virt
  </source>

  == History ==
  
  uBixOS has been in development since 2002. It is written and maintained by
  Christopher W. Olsen. The current release series is the 2.x line
  (v2.3.0-BETA as of this writing).

  [[Category:Operating Systems]]
  [[Category:Hobby Operating Systems]]
  [[Category:C]]
  [[Category:X86]]
  [[Category:ARM]]
````

<a id="w005048"></a>

## W005048 — File:UBixOS.png

Bron-ID: W005048
Namespace: File (6)
Revisie: 30701 | Datum: 2026-06-08T16:20:03Z
Bron: https://wiki.osdev.org/File:UBixOS.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AUBixOS.png&oldid=30701

````text
uBixOS Running Doom
````

<a id="w005049"></a>

## W005049 — Linux Boot Protocol

Bron-ID: W005049
Namespace: Main (0)
Revisie: 30708 | Datum: 2026-06-10T01:43:55Z
Bron: https://wiki.osdev.org/Linux_Boot_Protocol
Revisie-URL: https://wiki.osdev.org/index.php?title=Linux_Boot_Protocol&oldid=30708

````text
{{Stub}}
Linux Boot Protocol is a convention used by the [[Bootloaders]], such as [[GRUB]], [[Limine]] or basically anything else, and the Linux kernel, similarly to other boot protocols, such as [[Multiboot]]. Apart from booting the Linux kernel, the protocol might also be of interest for other operating systems because of its ubiquitousness, especially for the embedded devices or less popular architectures (among OS developers); i.e. everything other than x86.

[[Category:Bootloaders]]
````

<a id="w005050"></a>

## W005050 — Interrupt Translation Service

Bron-ID: W005050
Namespace: Main (0)
Revisie: 30730 | Datum: 2026-06-11T19:18:27Z
Bron: https://wiki.osdev.org/Interrupt_Translation_Service
Revisie-URL: https://wiki.osdev.org/index.php?title=Interrupt_Translation_Service&oldid=30730

````text
The '''Interrupt Translation Service''' is an component of the [[Generic Interrupt Controller versions 3 and 4|ARM Generic Interrupt Controller]] (optional in GICv3, required in GICv4) that is used to route Locality-specific Peripheral Interrupts, such as message-signaled interrupts coming from [[PCI Express]] devices. The ITS manages a series of mapping tables for devices, events, interrupt collections, and other entities in order to redistribute message-signaled interrupts and help compartmentalize interrupts coming from different virtual machines.

If your ARM GIC has an ITS, you must use it to manage and send LPIs - the LPI register interface in the Redistributor will not be implemented.

== Detecting the ITS ==

=== Devicetree ===

When starting your system from a [[Devicetree]], the ITS is listed as a child node of the node describing the GIC:

<pre>
gic: interrupt-controller@0e001000 {
	compatible = "arm,gic-700", "arm,gic-v3";
	#address-cells = <2>;
	#interrupt-cells = <3>;
	#size-cells = <2>;
	single-redist;
	ranges;
	interrupt-controller;
	# ...

	its_pcie: its@0e050000 {
		compatible = "arm,gic-v3-its";
		msi-controller;
		reg = <0x0 0x0e050000 0x0 0x30000>;
	};
};
</pre>

The <code>reg</code> parameter of the ITS node will tell you where its control registers begin.

Note that a single system might have multiple ITS devices, each of which service interrupts from different sources.

=== ACPI ===

If you're booting from ACPI, the ITS will be listed as one of the items in the [[MADT]], specifically as entry type <code>0xF</code>. Its layout is as follows:

<syntaxhighlight lang="c">
struct its_entry {
  uint8_t type;
  uint8_t length;
  uint16_t reserved_0;
  uint32_t gic_its_id; // if there are multiple ITSes, all of these values must be unique
  struct its_control *its_control; // 64-bit address
  uint32_t reserved_1;
}
</syntaxhighlight>

Note again that there might be multiple ITS units.

== Starting the ITS ==

After setting up an enabling the GIC proper, along with initializing the GICR_PROPBASER and GICR_PENDBASER registers with the appropriate LPI tables, you can initialize the ITS. The ITS exposes a series of control registers starting at the base address given in the Devicetree or MADT:

{| class="wikitable"
|+ ITS Control Registers
|-
! Name !! Offset !! Width !! Type !! Description
|-
| GITS_CTLR || 0x0000 || 4 || RW || Control register (activate and read ITS state)
|-
| GITS_IIDR || 0x0004 || 4 || RO || Identification register
|-
| GITS_TYPER || 0x0008 || 8 || RO || Type register (read ITS features)
|-
| GITS_MPAMIDR || 0x0010 || 4 || RO || Support MPAM sizes
|-
| GITS_PARTIDR || 0x0014 || 4 || RW || PARTID register
|-
| GITS_MPIDR || 0x0018 || 4 || RO || ITS affinity (the set of PEs it can send events to)
|-
| GITS_STATUSR || 0x0040 || 4 || RO || (Optional) Error reporting status register
|-
| GITS_UMSIR || 0x0048 || 8 || RO || (Optional) Unmapped MSI register
|-
| GITS_CBASER || 0x0080 || 8 || RW || Base address of command queue
|-
| GITS_CWRITER || 0x0088 || 8 || RW || Write position of command queue
|-
| GITS_CREADR || 0x0090 || 8 || RO || Read position of command queue
|-
| GITS_BASER<n> || 0x0100 + 0x0008 * n || 8 || RW || Translation table descriptors
|}

=== Disabling the ITS first ===

The first thing you should do is ensure the ITS is disabled and not running any commands,
first by clearing the enable bit (bit 0) of GITS_CTLR and then waiting for the quiescent bit
(bit 31) to be set:

<syntaxhighlight lang="c">
its_control->ctlr = its_control->ctlr & 0xfffffffe;
while (!(its_control->ctlr & 0x80000000)) ;
</syntaxhighlight>

=== Allocating ITS tables ===

Then, allocate tables for the GITS_BASER registers. Each GITS_BASER register describes a table that the ITS
uses for internal bookkeeping - you will not be accessing these tables during execution, but you will need to
reserve memory for them and clear them to zero. The layout of the GITS_BASER register is as follows:

{| class="wikitable"
|+ GITS_BASER Register
|-
! Name !! Bit Offsets !! Type !! Description
|-
| Valid || 63 || RW || Starts off clear, set this when you allocate memory for the table.
|-
| Indirect || 62 || RW || If cleared, describes a flat table. If set, describes a two-level table.
|-
| InnerCache || 59-61 || RW || Sets the inner cacheability of memory accesses to the table (e.g. 0b001 for Normal Inner Non-cacheable)
|-
| Type || 56-58 || RO || Describes the type of entity the table concerns itself with (1 for devices, 2 for vPEs, 4 for interrupt collections). In particular, if this field is 0, then the register does not describe a translation table and can be skipped.
|-
| OuterCache || 53-55 || RW || Sets the outer cacheabiilty of memory accesses to the table. 0 indicates that the value from InnerCache should be used.
|-
| EntrySize || 48-52 || RO || The number of bytes in each table entry, minus one (so a value of 0xf means 0x10 bytes).
|-
| PhysicalAddress || 12-47 || RW || Specifies the page-aligned physical address of the table.
|-
| Shareability || 10-11 || RW || Specifies the shareability attributes of the table (Non-shareable, Inner Shareable, Outer Shareable)
|-
| PageSize || 8-9 || RO || A code indicating the size of pages described in Size (0 for 4KiB, 1 for 16KiB, 2 for 64KiB)
|-
| Size || 0-7 || RW || The number of pages allocated for the table minus one. Page_Size specifies the size of each page.
|}

<syntaxhighlight lang="c">
for (int n = 0; n < 8; n++) {
  uint64_t baser_value = its_control->baser[n];
  int table_type = (baser_value & 0x0700000000000000) >> 0x38
  if (table_type != 0) {
    size_t entry_size = ((baser_value & 0x1f000000000000) >> 0x30) + 1;
    size_t entry_count = get_entry_count (table_type);
    size_t needed_size = entry_size * entry_count;
    int page_size_code = baser_value & 0x300;
    size_t page_size = 1000 << (page_size_code >> 7);
    size_t allocation_size = (needed_size + page_size - 1) & ~(page_size - 1)
    size_t size_in_pages = allocation_size / page_size;
    void *table = alloc_aligned_to_0x10000_uncacheable(allocation_size); // whatever your kernel allocation function is
    its_control->baser[n] = baser_value & 0xffff000000000000) | 0x8800000000000000 | (uint64_t) table | (size_in_pages - 1);
  }
}
</syntaxhighlight>

The number of entries you need for each table will depend on what type of entries it stores:

* The Device table stores an entry indexed by the DeviceID of the requesting device. For PCIe, this is the requester ID of the device (the packed tuple of bus, device, and function numbers), meaning that you will likely want to implement 0x10000 entries for this table if this ITS's purpose is to service a PCIe bus.
* The vPE table stores processor identities for virtual interrupts (GICv4 only). If your OS is not using these, allocate 1.
* The Collection table stores mappings for interrupt collections. Collections are a virtual mapping that is primarily intended to link events to specific Redistributors. If you're not doing anything particularly complex with interrupt routing, this table should have one entry for each processor you want to have servicing hardware interrupts.

=== Allocating the command queue ===

The last memory you'll allocate is the command queue, which you'll be using to send commands to the ITS.
This code allocates a page-sized queue:

<syntaxhighlight lang="c">
// set both indices to zero
its_control->cwriter = 0;
its_control->creadr = 0;
// allocate a 1-page buffer and set the base register to that
its_control->cbaser = ((uint64_t) alloc_aligned_to_0x10000_uncacheable(0x1000)) | 0x8800000000000000;
</syntaxhighlight>

=== Enabling the ITS ===

Once you've allocated all the required memory, enabling the ITS is a simple matter of setting
the Enable bit:

<syntaxhighlight lang="c">
its_control->ctlr = its_control->ctlr | 1;
</syntaxhighlight>

== ITS Commands ==

Once you have enabled the ITS, interaction with it will primarily consist of sending commands.
Commands are 32 bytes long and have their contents primarily arranged into four quadwords:

<syntaxhighlight lang="c">
struct its_command = {
  uint64_t qword0;
  uint64_t qword1;
  uint64_t qword2;
  uint64_t qword3;
};
</syntaxhighlight>

To send a command, write it into the location pointed to by <code>cwriter & 0xfffe0 + cbaser & 0xfffffff000</code>,
then increment <code>cwriter</code> by 32, wrapping back around to 0 if needed. The ITS will start processing
the command immediately, and will update the value of <code>creadr</code> accordingly. If a command fails
such that the queue stalls, the low bit of <code>creadr</code> will be set.

The ITS offers quite a few commands to work with, which are listed in the GICv3/4 specification.
Here is a list of the commands that would be necessary for a simplified interrupt setup.

=== MAPC ===

MAPC creates an Interrupt Collection and maps it to a Redistributor. It can also be used
to remove a previously created mapping.

Bit 0x10 (PTA) of GITS_TYPER will state how the Redistributor should be specified:

* If the bit is set, it should be the physical address of the Redistributor.
* Otherwise, it should be the Redistibutor's processor number.

<syntaxhighlight lang="c">
int valid; // whether you are creating or removing a mapping
int icid; // whatever collection ID you want to assign
int pta = its_control->typer & 0x80000;
uint64_t rdbase_field = (pta ? (uint64_t) gicr : (gicr->typer & 0xffff00) << 8)
its_command->qword0 = 9;
its_command->qword1 = 0;
its_command->qword2 = (valid ? 0x80000000000000000 : 0) | rdbase_field | icid;
its_command->qword3 = 0;
</syntaxhighlight>

=== MAPD ===

MAPD creates a Device entry for a given DeviceID and maps it to an Interrupt Translation Table
that you create for it. It can also be used to remove an existing mapping.

The Interrupt Translation Table is another internal bookkeeping table, allocated per device
that you will be servicing interrupts from. The size of entries in this table is given by the ITT_Entry_Size
field in GITS_TYPER.

<syntaxhighlight lang="c">
int valid; // whether you are creating or removing a mapping
int device_id; // the device ID you are mapping
int size_log_2; // the log base 2 of the max number of the event ID you want to support
size_t itt_entry_size = (its_control->typer & 0xf0) >> 4 + 1;
void *itt = alloc_aligned_to_0x1000_uncacheable(itt_entry_size << size_log_2);
its_command->qword0 = 9 | device_id << 0x20;
its_command->qword1 = size_log_2 - 1;
its_command->qword2 = (valid ? 0x80000000000000000 : 0) | (uint64_t) itt;
its_command->qword3 = 0;
</syntaxhighlight>

=== MAPTI ===

MAPTI maps a given DeviceID/EventID tuple to a given LPI on a given ICID.

<syntaxhighlight lang="c">
int device_id; // the device ID you are mapping
int event_id; // the event ID you are mapping
int lpi; // the LPI ID you want to map to
int icid; // the ICID you want to service this interrupt
its_command->qword0 = 0xa | device_id << 0x20;
its_command->qword1 = event_id | lpi << 0x20;
its_command->qword2 = icid;
its_command->qword3 = 0;
</syntaxhighlight>

=== DISCARD ===

DISCARD unmaps an exisitng DeviceID/EventID tuple.

<syntaxhighlight lang="c">
int device_id; // the device ID you are mapping
int event_id; // the event ID you are mapping
its_command->qword0 = 0xf | device_id << 0x20;
its_command->qword1 = event_id;
its_command->qword2 = 0;
its_command->qword3 = 0;
</syntaxhighlight>

=== INV ===

INV ensures that any caching in the Redistributors for the given DeviceID/EventID tuple
matches the LPI configuration tables held in memory (which can be changed by other commands).

<syntaxhighlight lang="c">
int device_id; // the device ID you are mapping
int event_id; // the event ID you are mapping
uint64_t rdbase_field = same_way_you_calculated_rdbase_field_earlier();
its_command->qword0 = 0xc | device_id << 0x20;
its_command->qword1 = event_id;
its_command->qword2 = rdbase_field;
its_command->qword3 = 0;
</syntaxhighlight>

Note that some commands do this invalidation automatically, and you don't need to run this
command. In particular, DISCARD does this invalidation, but the MAP commands do not.

=== SYNC ===

Ensures that all outstanding ITS operations associated with the given Redistributor are globally
observed before any further ITS commands are executed.

<syntaxhighlight lang="c">
uint64_t rdbase_field = same_way_you_calculated_rdbase_field_earlier();
its_command->qword0 = 0x5;
its_command->qword1 = 0;
its_command->qword2 = rdbase_field;
its_command->qword3 = 0;
</syntaxhighlight>

== Receiving Message Signaled Interrupts ==

Once you've used the command queues to set up interrupt mappings on the ITS, you are now ready to send
message signaled interrupts from connected devices.

The destination address for message signaled interrupts is the GITS_TRANSLATER register,
a 4-byte write-only register located at offset 0x10040 from the control register base
(technically, at offset 0x40 in the so-called "ITS translation register map").
You will want to instruct your MSI devices to write the EventID you set to this address:

<syntaxhighlight lang="c">
struct its_translation *its_translation = (struct its_translation *)((uint64_t)its_control + 0x10000);
struct msix_table_entry *entry; // an MSI-X table entry in the device's PCIe config space
msix_table_entry->data = event_id;
msix_table_entry->address = &(its_translation->translater);
msix_table_entry->vector_control = 0;
</syntaxhighlight>

When a device writes to this register, it will also communicate its DeviceID via an auxillary channel, which cannot be spoofed by the interrupt sender but can be read by the ITS. This DeviceID varies based on what kind of devices the ITS is connected to, but for [[PCI Express]], it is the requester ID of the device (<code>bus << 8 | device << 3 | function</code>).

Any DeviceID/EventID combination that is written to this register that has not yet been mapped will either be dropped, or will be reported on the GITS_UMSIR register.

== References ==

* [https://kib.kiev.ua/x86docs/ARM/GIC/IHI0069C_gic_architecture_specification.pdf|ARM GICv3/4 specification (Arm IHI 0069G)]

[[Category:ARM]]
````

<a id="w005051"></a>

## W005051 — MCFG

Bron-ID: W005051
Namespace: Main (0)
Revisie: 30724 | Datum: 2026-06-11T07:47:06Z
Bron: https://wiki.osdev.org/MCFG
Revisie-URL: https://wiki.osdev.org/index.php?title=MCFG&oldid=30724
Doorverwijzing: PCI Express

````text
#REDIRECT [[PCI Express#Enhanced%20Configuration%20Mechanism]]
````

<a id="w005052"></a>

## W005052 — User:Mishakov

Bron-ID: W005052
Namespace: User (2)
Revisie: 30728 | Datum: 2026-06-11T09:33:51Z
Bron: https://wiki.osdev.org/User:Mishakov
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMishakov&oldid=30728

````text
Hi, I'm Misha. I'm a computer engineer and finishing Master's in EE. I develop [https://gitlab.com/mishakov/pmos pmOS] microkernel operating system as a hobby.
````

<a id="w005053"></a>

## W005053 — Page Tables

Bron-ID: W005053
Namespace: Main (0)
Revisie: 30732 | Datum: 2026-06-12T15:53:08Z
Bron: https://wiki.osdev.org/Page_Tables
Revisie-URL: https://wiki.osdev.org/index.php?title=Page_Tables&oldid=30732
Doorverwijzing: X86 Page Tables

````text
#REDIRECT [[X86 Page Tables]]
````

<a id="w005054"></a>

## W005054 — Talk:X86 Page Tables

Bron-ID: W005054
Namespace: Talk (1)
Revisie: 30734 | Datum: 2026-06-12T16:24:55Z
Bron: https://wiki.osdev.org/Talk:X86_Page_Tables
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AX86_Page_Tables&oldid=30734

````text
== Lack of nuance in Recursive mapping section ==

Imo, the recursive mapping section lacks a nuance (the mapping does not have to be in the top level of page tables), as such the used virtual space calculations are a bit shady (especially the claim that 1GB of virtual address space is wasted with PAE; yes, if you have the mapping in PDPT, sure, but if you place it in PD it's only 2MB of memory; with other paging modes you can play with it as well, but the issue with limited virtual adress space is not as problematic). At the same time, it would probably be nice if this section had actually mentioned the advantages/actual problems with recursive mappings, like TLB invalidation, performance (which is potentially better than walking the page tables manually), and so on.

--[[User:Mishakov|Mishakov]] ([[User talk:Mishakov|talk]]) 16:24, 12 June 2026 (UTC)

== Redundant article ==

I feel like, while this page has a bunch of useful information, a lot of it is redundant with [[Paging]] and other articles, though I don't really know what to do with either of them...

--[[User:Mishakov|Mishakov]] ([[User talk:Mishakov|talk]]) 16:24, 12 June 2026 (UTC)
````

<a id="w005055"></a>

## W005055 — Paging

Bron-ID: W005055
Namespace: Main (0)
Revisie: 30736 | Datum: 2026-06-12T16:50:24Z
Bron: https://wiki.osdev.org/Paging
Revisie-URL: https://wiki.osdev.org/index.php?title=Paging&oldid=30736
Doorverwijzing: X86 Paging

````text
#REDIRECT [[X86 Paging]]
````

<a id="w005056"></a>

## W005056 — Talk:Paging

Bron-ID: W005056
Namespace: Talk (1)
Revisie: 30738 | Datum: 2026-06-12T16:50:24Z
Bron: https://wiki.osdev.org/Talk:Paging
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APaging&oldid=30738
Doorverwijzing: Talk:X86 Paging

````text
#REDIRECT [[Talk:X86 Paging]]
````

<a id="w005057"></a>

## W005057 — Identity Paging

Bron-ID: W005057
Namespace: Main (0)
Revisie: 30741 | Datum: 2026-06-12T23:18:57Z
Bron: https://wiki.osdev.org/Identity_Paging
Revisie-URL: https://wiki.osdev.org/index.php?title=Identity_Paging&oldid=30741
Doorverwijzing: Direct Mapping

````text
#REDIRECT [[Direct Mapping]]
````

<a id="w005058"></a>

## W005058 — Talk:Direct Mapping

Bron-ID: W005058
Namespace: Talk (1)
Revisie: 30744 | Datum: 2026-06-13T00:05:53Z
Bron: https://wiki.osdev.org/Talk:Direct_Mapping
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADirect_Mapping&oldid=30744

````text
== Shady example ==

Imo the example code on this page is weird, but I don't really know/can't decide how to rework it, maybe it should be let go... --[[User:Mishakov|Mishakov]] ([[User talk:Mishakov|talk]]) 00:05, 13 June 2026 (UTC)
````

<a id="w005059"></a>

## W005059 — File:Windows9xRunningOutVirtualMemory.jpg

Bron-ID: W005059
Namespace: File (6)
Revisie: 30752 | Datum: 2026-06-15T09:47:57Z
Bron: https://wiki.osdev.org/File:Windows9xRunningOutVirtualMemory.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AWindows9xRunningOutVirtualMemory.jpg&oldid=30752

````text
== Summary ==
Windows 98 error message from trying to map 16GB physical memory into 2GB virtual address space.
````

<a id="w005060"></a>

## W005060 — Interrupts

Bron-ID: W005060
Namespace: Main (0)
Revisie: 30756 | Datum: 2026-06-15T16:30:27Z
Bron: https://wiki.osdev.org/Interrupts
Revisie-URL: https://wiki.osdev.org/index.php?title=Interrupts&oldid=30756
Doorverwijzing: X86 Interrupts

````text
#REDIRECT [[X86 Interrupts]]
````

<a id="w005061"></a>

## W005061 — Talk:Interrupts

Bron-ID: W005061
Namespace: Talk (1)
Revisie: 30758 | Datum: 2026-06-15T16:30:27Z
Bron: https://wiki.osdev.org/Talk:Interrupts
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AInterrupts&oldid=30758
Doorverwijzing: Talk:X86 Interrupts

````text
#REDIRECT [[Talk:X86 Interrupts]]
````

<a id="w005062"></a>

## W005062 — S390 Interrupts

Bron-ID: W005062
Namespace: Main (0)
Revisie: 30759 | Datum: 2026-06-15T16:36:50Z
Bron: https://wiki.osdev.org/S390_Interrupts
Revisie-URL: https://wiki.osdev.org/index.php?title=S390_Interrupts&oldid=30759

````text
An exception on the S390 architecture is recognized depending on the context of the instruction.

=== External Interrupt ===
An external interrupt is produced when the CPU timer or an external devices triggers said interrupt.

=== Supervisor Call Interrupt ===
The supervisor call generates an interrupt which will place the current [[PSW]] into the [[PSA Mapping#FLCSOPSW|FLCSOPSW]] (Service Old PSW). Then load the new PSW from [[PSA Mapping#FLCSNPSW|FLCSNPSW]] (Service New PSW). The location of the SVC code is placed at [[PSA Mapping#FLCESICODE|FLCESICODE]]

=== Input/Output Interrupt ===
This interrupt is generally triggered when a device completes execution of a command program.

=== Program Check Exception ===
A program exception is recognized when an invalid address is accessed or an invalid instruction is executed.

=== Specification Exception ===
Generally produced when the given arguments or registers to an instruction are invalid.

=== Machine Check Exception ===
Produced when part of the equipment reports a malfunction. Usually means that said device should be replaced and it's up to the user to do that.

== See Also ==
* [[Interrupts]]

[[Category:Interrupts]]
[[Category:S390]]
````

<a id="w005063"></a>

## W005063 — Intel VT-d

Bron-ID: W005063
Namespace: Main (0)
Revisie: 30803 | Datum: 2026-06-20T21:39:56Z
Bron: https://wiki.osdev.org/Intel_VT-d
Revisie-URL: https://wiki.osdev.org/index.php?title=Intel_VT-d&oldid=30803

````text
{{stub}}

Intel VT-d provides an [[IOMMU]] to translate device addresses to host addresses, similar to the MMU on the CPU. It is also able to remap interrupt lines.

The location of the configuration registers can be found in the DMAR [[ACPI]] table. Note, DMAR is specific to VT-d.

== DMAR ==
VT-d is found by locating the DMAR ACPI table, much like the [[MADT]].
{| class="wikitable"
|+
!Offset
!0
!1
!2  
!3  
!4
!5
!6
!7
|-
|0x0
| colspan="4" |Signature
| colspan="4" |Length
|-
|0x8
|REV
|CHK
| colspan="6" |OEMID
|-
|0x10
| colspan="8" |OEM Table ID
|-
|0x18
| colspan="4" |OEM Revision
| colspan="4" |Creator ID
|-
|0x20
| colspan="4" |Creator Revision
|HAW
|FLG
| colspan="2" |Resevred
|-
|0x28
| colspan="8" |Reserved
|-
|0x30
| colspan="8" rowspan="4" |DRHD #1
|-
|0x38
|-
|0x40
|-
|0x48
|-
|0x50
| colspan="8" rowspan="3" |DRHD #2
|-
|0x58
|-
|0x60
|-
|0x68
| colspan="8" rowspan="2" |DRHD...
|-
|0x70...
|}
Signature: "DMAR".

== Translation structures ==

There are two translation formats: ''legacy'' and ''scalable''. The legacy format is simpler but lacks useful features. Importantly, it only supports second-stage page tables '''which are not compatible with [[X86 Paging]] tables'''.

This section will only discuss the scalable format, which supports first-stage translation and is directly compatible with 64-bit x86 page tables<ref>Section 3.6: "First-stage translation supports the same paging structures as Intel® 64 processors when operating in
64-bit mode"</ref>.

Note that the tables are all 4KiB in size and alignment. This can be used as a sanity check when defining structures.

{{AdmonitionNote|text=Requests to the <code>0xFEEx_xxxx</code> region do not go through DMA translation tables. See the Interrupt Remapping section.}}

=== Scalable Mode Root Table ===

The root table consists of 256 '''Root Entries''', with each entry corresponding to a single bus. Each entry points to a single ''Context Table''.

==== (Lower/Upper) Context Table ====

The context table is split in two equally sized halves, each half containing 128 entries for a total of 256 entries.

Each entry corresponds to a single device function.

=== Scalable Mode PASID structures ===

The PASID tree consists of a directory and tables, similar to page tables. Each directory entry points to a table.

The [[Process Address Space ID (PCIe)|PASID]] is either a default value ("request-without-PASID") or included in the [[Transaction Layer Packet (PCIe)|TLP]] ("request-with-PASID").

=== First-Stage Page Table Structures ===

These page tables are fully compatible with the CPU page tables. However, some of the otherwise ignored bits may be accessed and interpreted specially by the IOMMU.

==== Extended Access bit ====

== Interrupt Remapping ==

=== MSI / MSI-X ===

Any write requests to the <code>0xFEEx_xxxx</code> region '''do not go through DMA translation tables'''.

== Emulation ==

=== QEMU ===

To use an emulated VT-d IOMMU with support for scalable mode and first-stage translation structures, add the following argument:
<syntaxhighlight lang="sh">
-device intel-iommu,x-scalable-mode=on,x-flts=on
</syntaxhighlight>
(<code>flts</code> is a historical acronym meaning "First-Layer Translation Structures")

==== <code>info mtree</code> ====

The memory regions used by the emulated devices will look like this:

<pre>
memory-region: vtd-1f.2
  0000000000000000-ffffffffffffffff (prio 0, container): vtd-1f.2
    0000000000000000-ffffffffffffffff (prio 0, i/o): vtd-1f.2-dmar
      00000000fee00000-00000000feefffff (prio 1, i/o): alias vtd-ir @vtd-ir 0000000000000000-00000000000fffff
</pre>

Note the <code>vtd-ir</code> range with higher priority than the <code>dmar</code> range.

== See also ==

<references />

=== External Links ===

* [https://cdrdv2-public.intel.com/671081/vt-directed-io-spec.pdf Intel® Virtualization Technology for Directed I/O (June 2022 Revision 4.0)]

[[Category:X86 Virtualization]]
[[Category:X86-64]]
````

<a id="w005064"></a>

## W005064 — X86 Cooperative Multitasking Tutorial

Bron-ID: W005064
Namespace: Main (0)
Revisie: 30764 | Datum: 2026-06-16T15:00:50Z
Bron: https://wiki.osdev.org/X86_Cooperative_Multitasking_Tutorial
Revisie-URL: https://wiki.osdev.org/index.php?title=X86_Cooperative_Multitasking_Tutorial&oldid=30764

````text
{{Rating|2}}
{{In_Progress}}

This tutorial will cover the creation of a co-operative (not pre-emptive) kernel-level multitasking system. That is, kernel threads and processes. The code here is specific to the IA-32 architecture.

== Requirements ==
* [[Paging]]

== ''task.h'' ==
This header defines the function and types used in here. It's code is
<syntaxhighlight lang="c">
#ifndef __TASK_H__
#define __TASK_H__

#include <stdint.h>

extern void initTasking();

typedef struct {
    uint32_t eax, ebx, ecx, edx, esi, edi, esp, ebp, eip, eflags, cr3;
} Registers;

typedef struct Task {
    Registers regs;
    struct Task *next;
} Task;

extern void initTasking();
extern void createTask(Task*, void(*)(), uint32_t, uint32_t*);

extern void yield(); // Switch task frontend
extern void switchTask(Registers *old, Registers *new); // The function which actually switches

#endif /* __TASK_H__ */
</syntaxhighlight>

== ''task.c'' ==
This file defines the actual wrappers that ''switchTask()'' uses, ''createTask()'' and ''yield()'':
<syntaxhighlight lang="c">
#include "task.h"

static Task *runningTask;
static Task mainTask;
static Task otherTask;

static void otherMain() {
    printk("Hello multitasking world!"); // Not implemented here...
    yield();
}

void initTasking() {
    // Get EFLAGS and CR3
    asm volatile("movl %%cr3, %%eax; movl %%eax, %0;":"=m"(mainTask.regs.cr3)::"%eax");
    asm volatile("pushfl; movl (%%esp), %%eax; movl %%eax, %0; popfl;":"=m"(mainTask.regs.eflags)::"%eax");

    createTask(&otherTask, otherMain, mainTask.regs.eflags, (uint32_t*)mainTask.regs.cr3);
    mainTask.next = &otherTask;
    otherTask.next = &mainTask;

    runningTask = &mainTask;
}

void createTask(Task *task, void (*main)(), uint32_t flags, uint32_t *pagedir) {
    task->regs.eax = 0;
    task->regs.ebx = 0;
    task->regs.ecx = 0;
    task->regs.edx = 0;
    task->regs.esi = 0;
    task->regs.edi = 0;
    task->regs.eflags = flags;
    task->regs.eip = (uint32_t) main;
    task->regs.cr3 = (uint32_t) pagedir;
    task->regs.esp = (uint32_t) allocPage() + 0x1000; // Not implemented here
    task->next = 0;
}

void yield() {
    Task *last = runningTask;
    runningTask = runningTask->next;
    switchTask(&last->regs, &runningTask->regs);
}
</syntaxhighlight>

== ''switch.S'' ==
This is the file that actually changes between tasks. It defines a function, ''switchTask()'', which does all the magic. It saves all registers to ''from'' and loads them from ''to''. It is trickier than you might think. Its function prototype is
<syntaxhighlight lang="c">
void switchTask(Registers *from, Registers *to);
</syntaxhighlight>

Its code is
<syntaxhighlight lang="asm">
.section .text
.global switchTask
switchTask:
    pusha
    pushf
    mov %cr3, %eax #Push CR3
    push %eax
    mov 44(%esp), %eax #The first argument, where to save
    mov %ebx, 4(%eax)
    mov %ecx, 8(%eax)
    mov %edx, 12(%eax)
    mov %esi, 16(%eax)
    mov %edi, 20(%eax)
    mov 36(%esp), %ebx #EAX
    mov 40(%esp), %ecx #IP
    mov 20(%esp), %edx #ESP
    add $4, %edx #Remove the return value ;)
    mov 16(%esp), %esi #EBP
    mov 4(%esp), %edi #EFLAGS
    mov %ebx, (%eax)
    mov %edx, 24(%eax)
    mov %esi, 28(%eax)
    mov %ecx, 32(%eax)
    mov %edi, 36(%eax)
    pop %ebx #CR3
    mov %ebx, 40(%eax)
    push %ebx #Goodbye again ;)
    mov 48(%esp), %eax #Now it is the new object
    mov 4(%eax), %ebx #EBX
    mov 8(%eax), %ecx #ECX
    mov 12(%eax), %edx #EDX
    mov 16(%eax), %esi #ESI
    mov 20(%eax), %edi #EDI
    mov 28(%eax), %ebp #EBP
    push %eax
    mov 36(%eax), %eax #EFLAGS
    push %eax
    popf
    pop %eax
    mov 24(%eax), %esp #ESP
    push %eax
    mov 40(%eax), %eax #CR3
    mov %eax, %cr3
    pop %eax
    push %eax
    mov 32(%eax), %eax #EIP
    xchg (%esp), %eax #We do not have any more registers to use as tmp storage
    mov (%eax), %eax #EAX
    ret #This ends all!
</syntaxhighlight>

== Doing it ==
Put this on some source file
<syntaxhighlight lang="c">
#include "task.h"

void doIt() {
    printk("Switching to otherTask... \n");
    yield();
    printk("Returned to mainTask!\n");
}
</syntaxhighlight>

Now, from your ''kernel_main()'' call ''doIt()''!

Congratulations! You've just implement kernel multitasking!

(Note: Article on proper kernel multi-tasking coming soon)

[[Category:C]]
[[Category:Multitasking]]
[[Category:X86 CPU]]
````

<a id="w005065"></a>

## W005065 — Talk:X86 Cooperative Multitasking Tutorial

Bron-ID: W005065
Namespace: Talk (1)
Revisie: 30766 | Datum: 2026-06-16T15:01:23Z
Bron: https://wiki.osdev.org/Talk:X86_Cooperative_Multitasking_Tutorial
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AX86_Cooperative_Multitasking_Tutorial&oldid=30766

````text
 <nowiki>mov 44(%eax), %eax ;CR3</nowiki> seems to be an issue for me . From the registers structure, <nowiki>40(%eax)</nowiki> seems to work
````

<a id="w005066"></a>

## W005066 — Talk:Timer Interrupt Sources

Bron-ID: W005066
Namespace: Talk (1)
Revisie: 30769 | Datum: 2026-06-17T09:40:03Z
Bron: https://wiki.osdev.org/Talk:Timer_Interrupt_Sources
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ATimer_Interrupt_Sources&oldid=30769

````text
== Splitting into arch-specific pages ==
I think it would be nice to split this page into x86 and ARM, as has been recently done with other pages, and make a generic interrupt sources page (and mention this on the Time and Date page), to accommodate/link to other architectures as well
--[[User:Mishakov|Mishakov]] ([[User talk:Mishakov|talk]]) 09:39, 17 June 2026 (UTC)
````

<a id="w005067"></a>

## W005067 — File:LaCarreraLibra.png

Bron-ID: W005067
Namespace: File (6)
Revisie: 30772 | Datum: 2026-06-17T19:35:39Z
Bron: https://wiki.osdev.org/File:LaCarreraLibra.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALaCarreraLibra.png&oldid=30772

````text

````

<a id="w005068"></a>

## W005068 — User:Arsenarsen

Bron-ID: W005068
Namespace: User (2)
Revisie: 30792 | Datum: 2026-06-19T10:08:23Z
Bron: https://wiki.osdev.org/User:Arsenarsen
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AArsenarsen&oldid=30792

````text
putting c++ in kernelspace
````

<a id="w005069"></a>

## W005069 — IPL and Loading your kernel

Bron-ID: W005069
Namespace: Main (0)
Revisie: 30811 | Datum: 2026-06-23T02:07:13Z
Bron: https://wiki.osdev.org/IPL_and_Loading_your_kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=IPL_and_Loading_your_kernel&oldid=30811

````text
{{FirstPerson}}

This article explains how z/Architecture systems perform '''initial program loading''' (IPL) and how an operating-system developer prepares bootable code on channel-attached storage. Architectural behavior described here is taken from the IBM ''Principles of Operation'' (SA22-7832, Thirteenth Edition). For CCW structure, ORB layout, and post-IPL I/O, see [[CCW]].

''Derived from IBM z/Architecture Principles of Operation (SA22-7832).''

== Overview ==

Initial program loading (IPL) is the architecturally defined manual procedure that reads a program from a designated I/O device and starts CPU execution under a new program status word (PSW).

There are two IPL types:

{| class="wikitable"
|-
! Type !! Availability !! Typical osdev use
|-
| '''CCW-type IPL''' || All machine configurations || Boot from DASD or tape via channel command words
|-
| '''List-directed IPL''' || Model-dependent || Console-entered parameters; devices not accessed by CCWs
|}

This article focuses on '''CCW-type IPL''', used for classic DASD and tape paths on real hardware, Hercules, and QEMU.

== Initiating CCW-type IPL ==

CCW-type IPL is started manually:

# Set the '''load-unit-address controls''' to the four-digit hexadecimal device number of the IPL device.
# Activate the '''load-clear''' or '''load-normal''' key.

When the '''alternate-subchannel-set-IPL-device''' facility is installed, an additional hexadecimal digit may select the subchannel set.

=== Reset behavior ===

{| class="wikitable"
|-
! Key !! Reset performed
|-
| '''load-clear''' || '''Clear reset''' on the entire configuration
|-
| '''load-normal''' || '''Initial CPU reset''' on the activating CPU (or lowest-numbered online primary CPU on some models), '''CPU reset''' propagated to all other CPUs, '''subsystem reset''' on the remainder
|}

When the '''configuration-z/Architecture-architectural-mode''' (CZAM) facility is '''not''' installed, either key sets architectural mode to ESA/390 (or ESA/390-compatibility mode). When CZAM '''is''' installed, architectural mode is unchanged.

=== Load state ===

After reset, the affected CPU enters the '''load state'''. The '''load indicator''' is on while the CPU remains in load state.

If IPL succeeds, the CPU leaves load state, the load indicator turns off, and execution continues under the PSW loaded from absolute storage. If IPL fails, the CPU stays in load state and absolute locations 0–7 are unpredictable.

== What the channel subsystem does ==

After reset, the channel subsystem initiates a read from the device designated by the load-unit-address controls. Architecturally this is equivalent to executing '''START SUBCHANNEL''' with:

* The subchannel for the IPL device number, and
* A command-mode '''operation-request block''' (ORB) that is all zeros except '''LPM = 0xFF''' (all logical paths available).

The channel-program address in the ORB is zero. Full ORB field layout is in [[CCW#Operation-request block (ORB)|CCW § ORB]]; the IPL-specific values are summarized below.

=== Implicit IPL ORB ===

Figure 17-2 in the PoP defines the ORB used during CCW-type IPL:

{| class="wikitable"
|-
! ORB word !! Bit(s) !! Field !! IPL value
|-
| 0 || 0–31 || Interruption parameter || 0
|-
| 1 || 0–3 || Subchannel key || 0
|-
| 1 || 4 || S (suspend control) || 0
|-
| 1 || 5–7 || C, M, Y || 0
|-
| 1 || 8 || F (CCW-format control) || 0 → format-0 CCWs
|-
| 1 || 9–15 || P, I, A, U, B, H, T || 0
|-
| 1 || 16–23 || LPM (logical-path mask) || 0xFF
|-
| 1 || 24–30 || L, D, (reserved) || 0
|-
| 1 || 31 || X (ORB extension) || 0
|-
| 2 || 1–31 || CCW address (CPA) || 0
|}

Because '''F = 0''', all CCWs in the IPL channel program (after the implicit first CCW) are '''format-0''' with 24-bit data addresses. IDA is still permitted using format-1 IDAWs below the 2&nbsp;GiB boundary (PoP programming note 7).

=== The implicit first CCW ===

The first CCW is '''not''' fetched from main storage. The channel subsystem behaves as if a format-0 CCW at absolute address '''0''' were executed:

{| class="wikitable"
|-
! Field !! IPL value
|-
| Command || Read (modifier bits zero) → code '''0x02'''
|-
| Data address || 0
|-
| Byte count || 24
|-
| CC (chain-command) || 1
|-
| SLI (suppress incorrect length) || 1
|-
| CD, SKP, PCI, IDA, S, MIDA || 0
|}

As a format-0 byte layout (big-endian, 8 bytes):

<pre>
Offset  Field
------  -----
0       0x02          command (Read)
1–3     0x00 0x00 0x00   data address
4       0x60          flags: CC (0x40) | SLI (0x20)
5–6     (ignored in format-0)
7–8     0x00 0x18     count = 24
</pre>

The read places 24 bytes into absolute storage starting at address 0. The CC flag causes command chaining to the format-0 CCW at absolute address '''8''' (next doubleword). Further CCWs may occupy address '''16''', or a CCW at 16 may use '''TIC''' (transfer-in-channel) to branch elsewhere. See [[CCW#Chaining|TIC and chaining]].

During IPL, the '''PCI''' flag is ignored (PoP §15). Other flags behave normally.

== Lowcore layout after the initial read ==

The 24-byte IPL record maps into absolute storage as follows:

{| class="wikitable"
|-
! Absolute offset !! Size !! Role
|-
| 0–7 || 8 || '''PSW''' loaded when IPL completes successfully (short-PSW form on z/Architecture when CZAM is installed)
|-
| 8–15 || 8 || Optional format-0 '''CCW #2''' (command-chained from the implicit CCW)
|-
| 16–23 || 8 || Optional format-0 '''CCW #3''' (often a TIC, or another read/control CCW)
|-
| 184–187 || 4 || IPL device '''subsystem ID''' (stored on successful completion)
|-
| 188–191 || 4 || Zeros
|}

Additional lowcore locations useful immediately after IPL:

{| class="wikitable"
|-
! Offset !! Content
|-
| 0xB8 (184 decimal for SCHID in some loaders) || Subchannel ID of the IPL device — used by boot loaders to issue further SSCH operations on the same device
|}

Exact post-IPL subchannel identification may also be obtained from the subchannel-information block; loaders commonly cache the SCHID from lowcore early.

=== Successful completion ===

When the last CCW of the IPL channel program completes successfully:

* Subsystem ID is stored at absolute 184–187; 188–191 are cleared.
* A new PSW is loaded from absolute 0–7.
* No I/O interruption is generated for the final IPL operation (unlike normal SSCH).
* The IPL subchannel is left as if START SUBCHANNEL had been executed with the implicit ORB above; save the '''LPUM''' from the SCHIB and use it as ORB LPM until your kernel can verify all channel paths (PoP programming note 6).

=== Failure conditions ===

IPL does not complete when, among other cases (PoP §17):

* No subchannel matches the IPL device number.
* No channel path to the device is available or operational.
* The device presents status other than the allowed combinations (channel end, device end, status modifier, busy, etc.).
* Subchannel status other than PCI is indicated (except some models retry interface-control check on another path).
* The loaded PSW has an early PSW-format error.

On failure, absolute 0–7 are unpredictable. Subsystem ID at 184–187 is unpredictable unless a subchannel was found.

== z/Architecture short PSW at absolute 0 ==

When CZAM is installed, IPL loads a '''short PSW''' (8 bytes) from absolute 0–7. Bit 31 of the first word is the '''EA''' (extended addressing) bit; it must be set for 64-bit addressing. Bit 32 (in the full 64-bit mask after expansion) is '''BA'''.

Minimal C structure for what you place at the start of your IPL record:

<syntaxhighlight lang="c">
/* Short PSW written at absolute 0 for z/Architecture IPL (8 bytes). */
struct ipl_short_psw {
    uint32_t mask;   /* bit 31 (EA) must be 1 for 64-bit mode */
    uint32_t addr;   /* entry point; bit 0 of addr field = 0 */
};

#define PSW_EA_BIT  0x00000001u  /* bit 31 of mask word in short PSW */

/* Example: enter _start at 0x10000, supervisor, DAT off, 64-bit */
struct ipl_short_psw boot_psw = {
    .mask = 0x00000001u,                    /* EA only; expand per your early boot needs */
    .addr = 0x80000000u | 0x00010000u,      /* 0x80000000 = high bit of address half */
};
</syntaxhighlight>

On real z/Architecture, the mask contains additional valid bits (I/O mask, external mask, DAT, wait, problem state, ASC, condition code, program mask). Your first-stage loader normally runs with DAT off, interrupts masked, and EA|BA set once you switch to 64-bit mode. See your platform's PSW conventions.

== Building a classic IPL record (multi-CCW chain) ==

Most introductory s390x examples use a 24-byte record: PSW + two chained CCWs.
=== Step 1 — PSW at 0 ===

Entry at <code>0x10000</code> after the chained read completes:

<syntaxhighlight lang="c">
/* Absolute 0x000: PSW — execution starts here when IPL finishes */
uint8_t abs0_psw[8] = {
    0x00, 0x00, 0x00, 0x01,   /* mask: EA=1 */
    0x80, 0x00, 0x00, 0x00,   /* addr high; OR with entry below */
};
/* Set bytes 4–7 to 0x80000000 | (entry & 0x7FFFFFFF), e.g. entry 0x10000:
   0x80, 0x00, 0x01, 0x00 */
</syntaxhighlight>

=== Step 2 — CCW at 8 (read stage-2 block) ===

<syntaxhighlight lang="c">
/* Format-0 CCW at absolute 0x008 */
struct ccw0 {
    uint8_t  cmd;
    uint8_t  addr[3];   /* 24-bit big-endian */
    uint8_t  flags;     /* CD CC SLI SKP PCI IDA S MIDA from bit 32 */
    uint8_t  _ign[3];
    uint16_t count;
} __attribute__((packed));

#define CCW0_CC  0x40
#define CCW0_SLI 0x20

struct ccw0 abs8_read = {
    .cmd   = 0x02,                    /* Read */
    .addr  = { 0x00, 0x00, 0x10 },    /* data address 0x1000 */
    .flags = CCW0_CC | CCW0_SLI,     /* chain + suppress length */
    .count = 512,
};
/* Bytes: 02 00 00 10 60 00 00 02 00 */
</syntaxhighlight>

=== Step 3 — CCW at 16 (TIC back, or NOP stop) ===

A minimal second CCW that simply ends the chain (control NOP, no CC):

<syntaxhighlight lang="c">
struct ccw0 abs16_nop = {
    .cmd   = 0x03,          /* Control, modifiers zero → NOP */
    .flags = 0,
    .count = 0,
};
</syntaxhighlight>

Alternatively, use '''TIC''' at 16 to loop on a SEARCH/READ sequence (see [[CCW#DASD channel program example (ZXFL)|DASD example]]).

=== Status modifier ===

If the device sets '''status modifier''' during IPL, the channel subsystem skips the CCW at ''current_address + 16'' and uses the CCW at ''current_address + 32'' instead (PoP programming note 2). Layout your record accordingly if the device may assert modifier.

== Loading your kernel ==

The classic way is to embeds multiple IPL/CCW chains across multi stages (zIPL) increasing complexity and more hardcoding addresses. It is possible to skip this by having a single entry and then implement a DASD/VTOC driver.
This approach is used by the '''ZXFoundationLoader'''.

# Stage 1 itself is the IPL and the DASD/VTOC driver.
# Immediately switch to 64 bit addressing mode
# Perform all further I/O with '''format-1 CCWs''' and a full DASD/VTOC driver via SSCH.

This reduces bootstrap assembly code used. Rather than assuming that a stage must be present, it finds and validate the requested dataset, e.g. your next stage in the chain.

=== IPL and switching addressing mode ===

ZXFoundation <code>arch/s390x/init/zxfl/stage1/head.S</code> places at <code>.org 0</code>:

<syntaxhighlight lang="asm">
    .section .text.ipl, "ax"
    .globl __zx_ipl_entry
    .org 0x0

__zx_ipl_entry:
    .long   0x00080000
    .long   0x80000000 + stage1_entry

    .long   0x00000000, 0x00000000
    .long   0x00000000, 0x00000000

    .section .text, "ax"
    .globl stage1_entry
stage1_entry:
    la      %r7, 2                      /* order: z/Arch + restore 64-bit PSWs */
    la      %r6, 0                      /* target CPU address = 0 */
    sigp    %r7, %r6, 0x12              /* SIGP SET ARCHITECTURE */
    bc      8, .Lswitched64             /* branch if accepted (CC=0) */
    la      %r7, 1                      /* fallback: z/Arch without PSW restore */
    sigp    %r7, %r6, 0x12
.Lswitched64:
    sam64                               /* switch addressing mode to 64-bit */
    /* clear BSS  */
    /* Jumps to C code */
</syntaxhighlight>

== Preparing a bootable DASD volume (Hercules) ==

# Compile everything into ELF64
# Convert stage 1 and stage 2 to raw binary
# Convert stage 1 to '''80-byte punch card format (TXT/END)''' (see '''bin2rec''' utility, from ZXFoundation or FLATBOOT)
# Use '''dasdload''' to make a 3390 disk image

The current approach is likely not to work on QEMU s390x, as QEMU demand a zIPL entry (S390EP) as part of the ABI

== See Also ==
=== References ===

* IBM SA22-7832, Thirteenth Edition (September 2019): CCW-type IPL (§17), implicit CCW, ORB Figure 17-2, programming notes 1–7.
* ZXFoundation ZXFL: <code>zxfl/stage1/head.S</code>, <code>zxfl/stage1/entry.c</code>, <code>zxfl/common/dasd_io.c</code>.

[[Category:S390]]
````

<a id="w005070"></a>

## W005070 — RISC-V IOMMU

Bron-ID: W005070
Namespace: Main (0)
Revisie: 30806 | Datum: 2026-06-21T12:21:36Z
Bron: https://wiki.osdev.org/RISC-V_IOMMU
Revisie-URL: https://wiki.osdev.org/index.php?title=RISC-V_IOMMU&oldid=30806

````text
{{stub}}

== Interrupt Remapping ==

=== MSI / MSI-X ===

There are two modes for translating MSIs:

* <code>msiptp.MODE = Off</code>: MSIs are translated through the DMA page tables as if they're regular memory writes.
* <code>msiptp.MODE = Flat</code>: The DMA page tables are ignored iff an IOVA matches <code>(iova & msi_addr_mask) == msi_addr_pattern</code>. It is translated through a ''virtual interrupt file''.

MSIs are expected to be redirected to a virtual [[Advanced Interrupt Architecture#IMSIC (Incoming Message-Signaled Interrupt Controller)|IMSIC]].

== See Also ==

<references />

=== External Links ===

* [https://github.com/riscv-non-isa/riscv-iommu/releases/download/20260222/riscv-iommu.pdf RISC-V IOMMU Architecture Specification (Version v1.0.1, 2026-02-22: Ratified)]

[[Category:RISC-V Virtualization]]
[[Category:RISC-V]]
````

<a id="w005071"></a>

## W005071 — Rump kernel

Bron-ID: W005071
Namespace: Main (0)
Revisie: 30816 | Datum: 2026-06-23T19:34:41Z
Bron: https://wiki.osdev.org/Rump_kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=Rump_kernel&oldid=30816

````text
{{Template:Kernel designs}}
{{stub}}

A '''rump kernel''' is essentially a monolithic kernel with the drivers carved out and put into libraries. This makes it especially interesting to other OS developers as these libraries may be reusable.

== See also ==

=== External Links ===

* [https://rumpkernel.github.io/ Rump Kernels]
* [https://wiki.netbsd.org/rumpkernel/ NetBSD wiki]
* [https://github.com/rumpkernel/wiki/wiki/Info%3A-Who-Uses-Rump-Kernels Who Uses Rump Kernels]
* [https://www.netbsd.org/docs/rump/sptut.html Tutorial On Rump Kernel Servers and Clients]
* [https://archive.fosdem.org/2014/schedule/event/01_uk_rump_kernels/attachments/slides/398/export/events/attachments/01_uk_rump_kernels/slides/398/fosdem2014.pdf Rump Kernels, Just Components (FOSDEM 2014)]
````

<a id="w005072"></a>

## W005072 — DTB

Bron-ID: W005072
Namespace: Main (0)
Revisie: 30826 | Datum: 2026-06-29T19:23:36Z
Bron: https://wiki.osdev.org/DTB
Revisie-URL: https://wiki.osdev.org/index.php?title=DTB&oldid=30826
Doorverwijzing: Devicetree

````text
#REDIRECT [[Devicetree]]
````

<a id="w005073"></a>

## W005073 — DTS

Bron-ID: W005073
Namespace: Main (0)
Revisie: 30827 | Datum: 2026-06-29T19:23:55Z
Bron: https://wiki.osdev.org/DTS
Revisie-URL: https://wiki.osdev.org/index.php?title=DTS&oldid=30827
Doorverwijzing: Devicetree

````text
#REDIRECT [[Devicetree]]
````

<a id="w005074"></a>

## W005074 — User:Ultraslayyy

Bron-ID: W005074
Namespace: User (2)
Revisie: 30837 | Datum: 2026-07-02T10:20:16Z
Bron: https://wiki.osdev.org/User:Ultraslayyy
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AUltraslayyy&oldid=30837

````text
<code>:)</code>
Currently working on [https://github.com/ultraslayyy/I-am-bored/tree/info/os my own OS]
````

<a id="w005075"></a>

## W005075 — Category:AArch64

Bron-ID: W005075
Namespace: Category (14)
Revisie: 30861 | Datum: 2026-07-07T17:27:27Z
Bron: https://wiki.osdev.org/Category:AArch64
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AAArch64&oldid=30861

````text
'''AArch64''' is the 64-bit version of the [[ARM]] architecture, released first released in 2011 as part of the ARMv8 architecture.

[[Category:ARM]]
````

<a id="w005076"></a>

## W005076 — User:Eviessh

Bron-ID: W005076
Namespace: User (2)
Revisie: 30872 | Datum: 2026-07-08T03:03:45Z
Bron: https://wiki.osdev.org/User:Eviessh
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AEviessh&oldid=30872

````text
I'm creating an operating system called [https://codeberg.org/eviessh/aster Aster]. Find me on CodeBerg [https://codeberg.org/eviessh here].
````

<a id="w005077"></a>

## W005077 — Modula-2 Bare Bones

Bron-ID: W005077
Namespace: Main (0)
Revisie: 30902 | Datum: 2026-07-24T17:17:24Z
Bron: https://wiki.osdev.org/Modula-2_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Modula-2_Bare_Bones&oldid=30902

````text
{{BeginnersWarning}}
{{Rating|2}}

This tutorial will give some insights on how to create a Multiboot2 kernel in Modula-2 running on modern x86_64 machines.

==Preface==
Modula-2 is a language which is developed in 1970s to 80s. While it has been used to develop multiple historical OSes like Medos-2 and Excelsior, there is almost no new design. Therefore, this tutorial gives a complete modern design of a Modula-2 kernel, with the latest toolchains, Multiboot2 protocol, and targeting x86_64.

===Prerequisites===
The following are required for building the kernel:

1. A working GNU Modula-2 compiler targeting x86_64 and producing ELF objects. You may compile it from the official GCC source by adding <code>m2</code> to the tag <code>--enable-languages</code>. A cross compiler is not required (and building a cross compiler for Modula-2 requires some efforts, and may fail). Other compilers like ADW Modula-2 are not tested and are likely not going to work.

2. A working x86_64-elf Binutils for the assembler "as" and linker "ld." This should be included in the cross-compiling toolchain. Using the linux-gnu version may produce errors.

3. (Optional) GNU Make if you don't want to write a shell script to deal with complex building steps.

The following are required for running the kernel:

1. A Multiboot2-compatible bootloader, including Grub2 and Limine. 

2. Tools for creating disk or ISO images, like xorriso.

3. An emulator supporting x86_64 like QEMU. Or, you can directly boot the kernel on your computer.

Finally, the following assumptions are made:

1. You have sufficient knowledge about Modula-2. [https://www.modula2.org/tutor/index.php Modula-2 tutorial] can be a good start.

2. You have sufficient knowledge about building a x86_64 kernel with Multiboot2 in C/C++, including how to set-up long mode, paging, GDT, Multiboot Tag, etc. We will not explain too much about that. We will only focus on the differences between a C/C++ kernel and Modula-2 kernel.

==Modula-2 Runtime Libraries==
GNU Modula-2 supplies several runtime libraries: m2pim, m2iso, m2min, m2log, and m2cor. While m2pim is by default included, it requires an operating system for memory and IO operations. Only m2min is usable for bare metal applications. However, it supports only a little features. GCC only provides 3 modules:

1. M2RTS: It includes runtime module management. The functions inside are all stubs that do nothing.
2. SYSTEM: A minimal version of the SYSTEM module. It only provides some type names like ADDRESS, WORD, BYTE, etc., and some constants. No function is defined.
3. libc: Provides two functions: abort and exit, which are infinite loops.

Until GCC 16.1.0, the M2RTS module of m2min library is buggy (has incompatible signatures). So we will create our own M2RTS stub. Also, for bare metal applications, abort and exit are not needed. So we only use the SYSTEM module. This means that, except language features and built-in functions, you can use almost nothing.

==Name Mangling==
GNU Modula-2 will perform name mangling to the functions. Therefore, the actual exported function name is ModuleName_FunctionName. This is important when you write assembly stubs and link object files. To prevent name mangling, define the module for "C" and then export unqualified procedures.

==Files==
1. stub.S: M2RTS stubs.

2. boot.S: Setting up Multiboot2, long mode, paging, etc., and jump to the Modula-2 kernel. It should be exactly the same for different languages. You may refer to the official example of Multiboot2.

3. Multiboot2.def: A Modula-2 translation of multiboot2.h in the official example.

4. Kernel.def & Kernel.mod: The main kernel, setting up the framebuffer, and outputs some color to the screen.

5. libc.def & libc.mod: Implementation of memcpy, memset, memmove, and memcmp. Sometimes gcc compiler will generate calls to these functions even if you don't call them explicitly. 

6. linker.ld: Link script.

==Code Implementation==
* stub.S
<syntaxhighlight lang="asm">
.global m2min_M2RTS_RequestDependant
.global m2min_M2RTS_RegisterModule
.text
m2min_M2RTS_RequestDependant:
ret

m2min_M2RTS_RegisterModule:
ret
</syntaxhighlight>
This file only defines two functions: RequestDependant and RegisterModule, in the m2min library. They do nothing but directly return from the function body. It provides as a runtime stub. In fact, you don't need them in bare bones, but it can make the linker work.

* boot.S
This file is fairly large. And it is almost the same as the C/C++ version except the jump target. Remember to change the jump target to the kernel entry you defined in the kernel. Don't forget about the name mangling. If you want to see the full code, you can refer to the "Example" section.

* libc.def
<syntaxhighlight lang="modula2">
DEFINITION MODULE FOR "C" libc ;

FROM SYSTEM IMPORT ADDRESS ;

EXPORT UNQUALIFIED memcpy, memset, memmove, memcmp;

PROCEDURE memcpy (dest, src: ADDRESS; n: CARDINAL) : ADDRESS ;
PROCEDURE memset (s: ADDRESS; c: INTEGER; n: CARDINAL) : ADDRESS ;
PROCEDURE memmove (dest, src: ADDRESS; n: CARDINAL) : ADDRESS ;
PROCEDURE memcmp (s1, s2: ADDRESS; n: CARDINAL) : INTEGER ;

END libc.
</syntaxhighlight>
Declaring the memory functions (memcpy, memset, memmove, memcmp) that the compiler generates calls to internally.  All symbols are exported unqualified with C linkage so they satisfy both Modula-2 IMPORT and compiler-generated references.

*libc.mod
<syntaxhighlight lang="modula2">
IMPLEMENTATION MODULE libc ;

FROM SYSTEM IMPORT ADDRESS, BYTE, CARDINAL64 ;

TYPE
    BytePtr = POINTER TO BYTE ;


(*
   memcpy — copy n bytes from src to dest (non-overlapping).
*)

PROCEDURE memcpy (dest, src: ADDRESS; n: CARDINAL) : ADDRESS ;
VAR
    d, s: BytePtr ;
    i: CARDINAL ;
BEGIN
    d := VAL (BytePtr, dest) ;
    s := VAL (BytePtr, src) ;
    i := n ;
    WHILE i > 0 DO
        d^ := s^ ;
        d := VAL (BytePtr, VAL (ADDRESS, VAL (CARDINAL64, VAL (ADDRESS, d)) + 1)) ;
        s := VAL (BytePtr, VAL (ADDRESS, VAL (CARDINAL64, VAL (ADDRESS, s)) + 1)) ;
        DEC (i) ;
    END ;
    RETURN dest ;
END memcpy ;


(*
   memset — set n bytes starting at s to byte value c.
*)

PROCEDURE memset (s: ADDRESS; c: INTEGER; n: CARDINAL) : ADDRESS ;
VAR
    p: BytePtr ;
    i: CARDINAL ;
BEGIN
    p := VAL (BytePtr, s) ;
    i := n ;
    WHILE i > 0 DO
        p^ := VAL (BYTE, c) ;
        p := VAL (BytePtr, VAL (ADDRESS, VAL (CARDINAL64, VAL (ADDRESS, p)) + 1)) ;
        DEC (i) ;
    END ;
    RETURN s ;
END memset ;


(*
   memmove — copy n bytes from src to dest (handles overlap).
*)

PROCEDURE memmove (dest, src: ADDRESS; n: CARDINAL) : ADDRESS ;
VAR
    d, s: BytePtr ;
    dAddr, sAddr: CARDINAL64 ;
    i: CARDINAL ;
BEGIN
    dAddr := VAL (CARDINAL64, VAL (ADDRESS, dest)) ;
    sAddr := VAL (CARDINAL64, VAL (ADDRESS, src)) ;
    IF dAddr < sAddr THEN
        (* Copy forward *)
        d := VAL (BytePtr, dest) ;
        s := VAL (BytePtr, src) ;
        i := n ;
        WHILE i > 0 DO
            d^ := s^ ;
            d := VAL (BytePtr, VAL (ADDRESS, VAL (CARDINAL64, VAL (ADDRESS, d)) + 1)) ;
            s := VAL (BytePtr, VAL (ADDRESS, VAL (CARDINAL64, VAL (ADDRESS, s)) + 1)) ;
            DEC (i) ;
        END ;
    ELSE
        (* Copy backward *)
        d := VAL (BytePtr, VAL (ADDRESS, dAddr + VAL (CARDINAL64, n) - 1)) ;
        s := VAL (BytePtr, VAL (ADDRESS, sAddr + VAL (CARDINAL64, n) - 1)) ;
        i := n ;
        WHILE i > 0 DO
            d^ := s^ ;
            d := VAL (BytePtr, VAL (ADDRESS, VAL (CARDINAL64, VAL (ADDRESS, d)) - 1)) ;
            s := VAL (BytePtr, VAL (ADDRESS, VAL (CARDINAL64, VAL (ADDRESS, s)) - 1)) ;
            DEC (i) ;
        END ;
    END ;
    RETURN dest ;
END memmove ;


(*
   memcmp — compare n bytes of s1 and s2.
   Returns 0 if equal, <0 if s1 < s2, >0 if s1 > s2.
*)

PROCEDURE memcmp (s1, s2: ADDRESS; n: CARDINAL) : INTEGER ;
VAR
    p1, p2: BytePtr ;
    i: CARDINAL ;
BEGIN
    p1 := VAL (BytePtr, s1) ;
    p2 := VAL (BytePtr, s2) ;
    i := n ;
    WHILE i > 0 DO
        IF p1^ # p2^ THEN
            IF VAL (CARDINAL, p1^) < VAL (CARDINAL, p2^) THEN
                RETURN -1 ;
            ELSE
                RETURN 1 ;
            END ;
        END ;
        p1 := VAL (BytePtr, VAL (ADDRESS, VAL (CARDINAL64, VAL (ADDRESS, p1)) + 1)) ;
        p2 := VAL (BytePtr, VAL (ADDRESS, VAL (CARDINAL64, VAL (ADDRESS, p2)) + 1)) ;
        DEC (i) ;
    END ;
    RETURN 0 ;
END memcmp ;


END libc.
</syntaxhighlight>
Implementation of memcpy, memset, memmove, and memcmp functions in Modula-2. These should be simple and straightforward. If you are not familiar with Modula-2, you can also use C language to implement these stubs in "libc.c," and then compile to an object file. 

*Multiboot2.def
A direct translation of multiboot2.h to Modula-2. Some hints:

1. Replace "#define" with CONST definitions.

2. Replace "struct" with RECORD definitions.

3. Replace the type names (for example, CARDINAL64 for uint_64, etc.).

If you don't want to build wheels on your own, it is also in the example. Refer to the "Example" section for complete code.

*Kernel.def
<syntaxhighlight lang="modula2">
DEFINITION MODULE Kernel;

PROCEDURE KMain(Magic, InfoAddr: LONGCARD);

END Kernel.
</syntaxhighlight>
Definition of the kernel module. It exports Kernel_KMain function.

*Kernel.mod
<syntaxhighlight lang="modula2">
IMPLEMENTATION MODULE Kernel;

FROM SYSTEM IMPORT BYTE, BITSET8, ADDRESS, CARDINAL8, CARDINAL32, CARDINAL64;
FROM Multiboot2 IMPORT
    MULTIBOOT2_BOOTLOADER_MAGIC, MULTIBOOT_INFO_ALIGN, MULTIBOOT_TAG_ALIGN,
    MULTIBOOT_TAG_TYPE_END, MULTIBOOT_TAG_TYPE_FRAMEBUFFER,
    MULTIBOOT_FRAMEBUFFER_TYPE_RGB,
    MultibootTag, MultibootTagPtr,
    MultibootTagFramebuffer, MultibootTagFramebufferPtr;

CONST
    COM1 = 3F8H;

TYPE
    (* 32-bit BGRA pixel (matches VBE RGB framebuffer layout) *)
    Pixel32 = RECORD
        blue:  BYTE;
        green: BYTE;
        red:   BYTE;
        alpha: BYTE;
    END;
    Pixel32Ptr = POINTER TO Pixel32;

(*
   ByteAnd - returns a bitwise (left AND right)
*)

PROCEDURE ByteAnd (left, right: BYTE) : BYTE ;
BEGIN
   RETURN VAL (BYTE, VAL (BITSET8, left) * VAL (BITSET8, right))
END ByteAnd ;

(* Half and Catch Fire *)
PROCEDURE HCF;
BEGIN
    LOOP
        ASM VOLATILE ("hlt");
    END;
END HCF;

PROCEDURE OUTB(port: SHORTCARD; value: BYTE);
BEGIN
    ASM VOLATILE ("outb %0, %1" : : "a"(value), "Nd"(port));
END OUTB;

PROCEDURE INB(port: SHORTCARD): BYTE;
VAR
    value: BYTE;
BEGIN
    ASM VOLATILE ("inb %1, %0" : "=a"(value) : "Nd"(port));
    RETURN value;
END INB;

PROCEDURE SerialInit;
BEGIN
    OUTB(COM1 + 1, 000H);    (* Disable all interrupts *)
    OUTB(COM1 + 3, 080H);  (* Enable DLAB (set baud rate divisor) *)
    OUTB(COM1 + 0, 003H);  (* Set divisor to 3 (lo byte) 38400 baud *)
    OUTB(COM1 + 1, 000H);  (*                  (hi byte) *)
    OUTB(COM1 + 3, 003H);  (* 8 bits, no parity, one stop bit *)
    OUTB(COM1 + 2, 0C7H);  (* Enable FIFO, clear them, with 14-byte threshold *)
    OUTB(COM1 + 4, 00BH);  (* IRQs enabled, RTS/DSR set *)
END SerialInit;

PROCEDURE SerialWriteChar(c: CHAR);
BEGIN
    WHILE ByteAnd(INB(COM1 + 5), 20H) = 0 DO
        (* Wait for the transmit buffer to be empty *)
    END;
    OUTB(COM1, BYTE(c));
END SerialWriteChar;

PROCEDURE SerialWriteString(s: ARRAY OF CHAR);
VAR
    i: CARDINAL;
BEGIN
    i := 0;
    WHILE (i < HIGH(s)) AND (s[i] # BYTE(0)) DO
        SerialWriteChar(s[i]);
        INC(i);
    END;
END SerialWriteString;

PROCEDURE SerialWriteLongCard(n: LONGCARD);
VAR
    buffer: ARRAY [0..10] OF CHAR;
    i: CARDINAL;
BEGIN
    IF n = 0 THEN
        SerialWriteChar('0');
        RETURN;
    END;
    i := 0;
    WHILE n > 0 DO
        buffer[i] := VAL(CHAR, n MOD 10 + VAL(LONGCARD, ORD('0')));
        INC(i);
        n := n DIV 10;
    END;
    WHILE i > 0 DO
        DEC(i);
        SerialWriteChar(buffer[i]);
    END;
END SerialWriteLongCard;

PROCEDURE SerialWriteLongCardHex(n: LONGCARD);
VAR
    buffer: ARRAY [0..8] OF CHAR;
    i: CARDINAL;
    digit: LONGCARD;
BEGIN
    i := 0;
    WHILE i < 8 DO
        digit := n MOD 16;
        IF digit < 10 THEN
            buffer[7 - i] := VAL(CHAR, digit + VAL(LONGCARD, ORD('0')));
        ELSE
            buffer[7 - i] := VAL(CHAR, digit - 10 + VAL(LONGCARD, ORD('A')));
        END;
        n := n DIV 16;
        INC(i);
    END;
    SerialWriteString("0x");
    FOR i := 0 TO 7 DO
        SerialWriteChar(buffer[i]);
    END;
END SerialWriteLongCardHex;

PROCEDURE KMain(magic, infoAddr: LONGCARD);
VAR
    tag: MultibootTagPtr;
    fbTag: MultibootTagFramebufferPtr;
    pixel: Pixel32Ptr;
    fbAddr: LONGCARD;
    pitch, width, height: CARDINAL32;
    bpp: CARDINAL8;
    row, col: CARDINAL32;
    rowAddr: LONGCARD;
BEGIN
    SerialInit;
    SerialWriteString("Hello, World from Modula-2 Kernel!");
    SerialWriteChar(BYTE(10)); (* Newline *)

    (* Check if the magic number is correct *)
    IF magic # MULTIBOOT2_BOOTLOADER_MAGIC THEN
        SerialWriteString("Error: Invalid magic number! Expected: ");
        SerialWriteLongCardHex(MULTIBOOT2_BOOTLOADER_MAGIC);
        SerialWriteChar(BYTE(10)); (* Newline *)
        HCF;
    END;

    (* Check alignment *)
    IF infoAddr MOD MULTIBOOT_INFO_ALIGN # 0 THEN
        SerialWriteString("Error: Info address is not properly aligned!");
        SerialWriteChar(BYTE(10)); (* Newline *)
        HCF;
    END;

    (* ---- Find the framebuffer tag in the multiboot2 info structure ---- *)
    tag := VAL(MultibootTagPtr, VAL(ADDRESS, infoAddr + 8));
    WHILE (tag # NIL) AND (tag^.tagType # MULTIBOOT_TAG_TYPE_FRAMEBUFFER) DO
        IF tag^.tagType = MULTIBOOT_TAG_TYPE_END THEN
            tag := NIL;
        ELSE
            (* Advance to next tag: align (addr + size) to 8 bytes *)
            tag := VAL(MultibootTagPtr,
                       VAL(ADDRESS,
                           (VAL(LONGCARD, VAL(ADDRESS, tag))
                            + VAL(LONGCARD, tag^.size) + 7) DIV 8 * 8));
        END;
    END;

    IF tag = NIL THEN
        SerialWriteString("Error: No framebuffer tag found!");
        SerialWriteChar(BYTE(10));
        HCF;
    END;

    (* ---- Read framebuffer parameters ---- *)
    fbTag := VAL(MultibootTagFramebufferPtr, VAL(ADDRESS, tag));
    fbAddr := VAL(LONGCARD, fbTag^.framebufferAddr);
    pitch  := fbTag^.framebufferPitch;
    width  := fbTag^.framebufferWidth;
    height := fbTag^.framebufferHeight;
    bpp    := fbTag^.framebufferBpp;

    SerialWriteString("Framebuffer: ");
    SerialWriteLongCard(width);
    SerialWriteString("x");
    SerialWriteLongCard(height);
    SerialWriteString("x");
    SerialWriteLongCard(VAL(LONGCARD, bpp));
    SerialWriteString(", pitch=");
    SerialWriteLongCard(pitch);
    SerialWriteString(", addr=");
    SerialWriteLongCardHex(fbAddr);
    SerialWriteChar(BYTE(10));

    (* ---- Fill the framebuffer with blue (32 bpp only) ---- *)
    IF bpp = 32 THEN
        FOR row := 0 TO height - 1 DO
            rowAddr := fbAddr + VAL(LONGCARD, row) * VAL(LONGCARD, pitch);
            FOR col := 0 TO width - 1 DO
                pixel := VAL(Pixel32Ptr,
                             VAL(ADDRESS, rowAddr + VAL(LONGCARD, col) * 4));
                pixel^.blue := BYTE(0);
                pixel^.green := BYTE(0);
                pixel^.red := BYTE(0);
                IF col < width DIV 3 THEN
                    pixel^.red := BYTE(255);
                ELSIF col < 2 * width DIV 3 THEN
                    pixel^.green := BYTE(255);
                ELSE
                    pixel^.blue := BYTE(255);
                END;
                pixel^.alpha := BYTE(255);
            END;
        END;
        SerialWriteString("Framebuffer filled with RGB!");
    ELSE
        SerialWriteString("Framebuffer is not 32 bpp (got ");
        SerialWriteLongCard(VAL(LONGCARD, bpp));
        SerialWriteString(" bpp) — skipping fill");
    END;
    SerialWriteChar(BYTE(10));

    HCF; (* Halt the CPU after printing the messages *)

END KMain;

END Kernel
</syntaxhighlight>
This kernel initializes the serial output COM1 for debugging, and then set-up the frame buffer. Finally, it prints three colors to the screen: red, green, and blue. It is almost the same as writing a similar kernel in C/C++, except:

1. The language core does not support bit operations. SO YOU NEED TO IMPLEMENT IT ON YOUR OWN! You can refer to the m2log library. But some functions require a runtime. You need to rewrite them for bare bones.

2. Number values require explicit type convertion using <code>VAL</code> function. It does not have implicit integer type convertion has C. This can make some operations, especially pointer arithmetics, especially complex.

*linker.ld
<pre>
/* linker.ld — Linker script for the multiboot2 x86_64 kernel */
OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

SECTIONS
{
    /* Load at 1 MB physical address */
    . = 0x100000;

    .multiboot_header : {
        *(.multiboot_header)
    }

    .text : {
        *(.text)
        *(.text.*)
    }

    .data : {
        *(.data)
        *(.data.*)
    }

    .bss : {
        *(COMMON)
        *(.bss)
        *(.bss.*)
    }

    /DISCARD/ : {
        *(.comment)
        *(.note.*)
        *(.eh_frame)
        *(.debug_*)
    }
}
</pre>
==Building==
Build the kernel object file and libc with:
<syntaxhighlight lang="shell-session">
$ gm2 -fno-exceptions -fno-wideset -mcmodel=large -flibs=min -mno-red-zone -mno-mmx -mno-sse -O2 -c -o FILENAME.o FILENAME.mod
</syntaxhighlight>
Where FILENAME should be libc and Kernel.

Build the assembly files with:
<syntaxhighlight lang="shell-session">
$ x86_64-elf-as -o FILENAME.o FILENAME.S
</syntaxhighlight>
Where FILENAME should be boot and stub.

Finally, link then together with:
<syntaxhighlight lang="shell-session">
$ x86_64-elf-ld -T linker.ld -nostdlib -o kernel.elf boot.o stub.o Kernel.o libc.o
</syntaxhighlight>
If all correct, it will produce kernel.elf, which can be loaded by bootloaders supporting Multiboot2, like GRUB2 or Limine.

==Example==
The following is the minimal kernel I have written for this tutorial: https://github.com/fzhwenzhou/ModOS-kernel/tree/99a2388935ad93745b4ba956a1494db2d508b3d9 . Any file that is not included in this tutorial can be found in this repository.
[[Category:Bare bones tutorials]]
[[Category:Inadvisable]]
````
